//! MIG-003 Step 4 — Filesystem migration: canonical → human filenames.
//!
//! Walks every library declared in libraries.json. For every `.md` whose
//! filename matches the canonical pattern (`YYYYMMDDTHHMMSSZ_KIND_HEX`),
//! computes a human-readable target filename from the note's frontmatter
//! title, resolves collisions, renames the file on disk, and cascades
//! the path change to every dependent table inside a per-library
//! transaction.
//!
//! Frontmatter aliases append (preserving the canonical stem so external
//! systems referencing the old name still resolve) is DEFERRED to §89 —
//! it's a separate concern and lets us validate the rename pass first.
//!
//! Idempotent: already-renamed files (non-canonical) are skipped via
//! `is_canonical_filename`. Re-running the migration after a partial
//! crash picks up only the unrenamed survivors.
//!
//! Gated by `schema_versions.mig003_step4`; runs once on first boot
//! after the binary lands, then short-circuits on every subsequent boot.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rusqlite::{params, Connection};

pub(crate) const MIG003_STEP4_VERSION: i64 = 1;

struct Library {
    name: String,
    path: String,
}

#[derive(Debug)]
struct RenameOutcome {
    old_path: String,
    new_path: String,
    title: String,
    cid_cn: String,
}

fn parse_libraries_json(json_path: &Path) -> Result<Vec<Library>, String> {
    let content =
        fs::read_to_string(json_path).map_err(|e| format!("read libraries.json: {}", e))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("parse libraries.json: {}", e))?;
    let arr = parsed.as_array().ok_or("libraries.json not an array")?;
    let mut libs = Vec::new();
    for entry in arr {
        let name = entry
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let path = entry
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if !path.is_empty() {
            libs.push(Library { name, path });
        }
    }
    Ok(libs)
}

fn collect_canonical_md(library_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(library_root, &mut out, 0);
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, depth: u32) {
    if depth > 30 {
        return;
    }
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in read.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            walk(&path, out, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false)
            && crate::canonical::is_canonical_filename(&path)
        {
            out.push(path);
        }
    }
}

/// Look up title and cid_cn for a given path from note_meta. If the row
/// is missing (e.g. a stray canonical file the indexer never reached),
/// return None — caller skips it.
fn lookup_title_and_cid(conn: &Connection, path: &str) -> Option<(String, String)> {
    conn.query_row(
        "SELECT name, cid_cn FROM note_meta WHERE path = ?1",
        params![path],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )
    .ok()
}

/// MIG-003 Step 4 entry point. Runs unconditionally; caller (init_db)
/// gates on schema_versions.
///
/// `db_path` is the path to `search.db` itself (matches diag_log's
/// signature). The migration's working directory is its parent (i.e.
/// `<universe>/.constellation/`) — that's where libraries.json lives
/// and where the audit log is written.
pub(crate) fn run(conn: &mut Connection, db_path: &Path) -> rusqlite::Result<()> {
    let t = Instant::now();
    let db_dir: PathBuf = db_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let libraries_json = db_dir.join("libraries.json");

    let libraries = match parse_libraries_json(&libraries_json) {
        Ok(l) => l,
        Err(e) => {
            crate::search::diag_log(
                db_path,
                &format!("[mig003-step4] libraries.json parse failed: {}", e),
            );
            // Return an error so init_db DOES NOT stamp the migration
            // as complete. Next boot will retry. Without this, a stale
            // libraries.json (or any read failure) would leave the
            // user's canonical files un-renamed but the migration
            // marked done.
            return Err(rusqlite::Error::ExecuteReturnedResults);
        }
    };

    crate::search::diag_log(
        db_path,
        &format!(
            "[mig003-step4] starting; {} libraries to scan",
            libraries.len()
        ),
    );

    // Build per-library snapshots up front (parallel-safe; no shared state).
    let mut by_library: HashMap<String, (String, Vec<PathBuf>)> = HashMap::new();
    let mut total_canonical = 0usize;
    for lib in &libraries {
        let lib_root = Path::new(&lib.path);
        if !lib_root.exists() {
            crate::search::diag_log(
                db_path,
                &format!(
                    "[mig003-step4] library '{}' path not found, skipping: {}",
                    lib.name, lib.path
                ),
            );
            continue;
        }
        let files = collect_canonical_md(lib_root);
        total_canonical += files.len();
        crate::search::diag_log(
            db_path,
            &format!(
                "[mig003-step4] library '{}': {} canonical files",
                lib.name,
                files.len()
            ),
        );
        by_library.insert(lib.name.clone(), (lib.path.clone(), files));
    }

    crate::search::diag_log(
        db_path,
        &format!(
            "[mig003-step4] total canonical .md files to rename: {}",
            total_canonical
        ),
    );

    // Audit-log accumulator. TSV header.
    let mut audit_lines = String::from("old_path\tnew_path\ttitle\tcid_cn\n");
    let mut total_renamed = 0usize;
    let mut total_skipped = 0usize;
    let mut total_errors = 0usize;

    for (lib_name, (_, files)) in &by_library {
        if files.is_empty() {
            continue;
        }
        let lib_t = Instant::now();
        let mut lib_renamed = 0usize;
        let mut lib_skipped = 0usize;
        let mut lib_errors = 0usize;

        let tx = conn.transaction()?;
        {
            let mut upd_meta = tx.prepare("UPDATE note_meta SET path = ?2 WHERE path = ?1")?;
            let mut upd_links_src =
                tx.prepare("UPDATE note_links SET source_path = ?2 WHERE source_path = ?1")?;
            let mut upd_links_tgt =
                tx.prepare("UPDATE note_links SET target_path = ?2 WHERE target_path = ?1")?;
            let mut upd_aliases =
                tx.prepare("UPDATE note_aliases SET path = ?2 WHERE path = ?1")?;
            let mut upd_embed =
                tx.prepare("UPDATE note_embeddings SET path = ?2 WHERE path = ?1")?;

            for old_path in files {
                let old_path_str = old_path.to_string_lossy().to_string();
                let parent = match old_path.parent() {
                    Some(p) => p,
                    None => {
                        lib_errors += 1;
                        continue;
                    }
                };

                // Look up title from note_meta. If row is missing, skip.
                let (title, cid_cn) = match lookup_title_and_cid(&tx, &old_path_str) {
                    Some(t) => t,
                    None => {
                        lib_skipped += 1;
                        continue;
                    }
                };
                if title.is_empty() {
                    lib_skipped += 1;
                    continue;
                }

                // Compute target filename.
                let safe_stem = crate::libraries::note_display_filename(&title);
                let new_filename = match crate::libraries::resolve_filename_collision(
                    parent,
                    &safe_stem,
                    ".md",
                    true,
                ) {
                    Ok(n) => n,
                    Err(e) => {
                        crate::search::diag_log(
                            db_path,
                            &format!(
                                "[mig003-step4] collision resolution failed for '{}': {}",
                                old_path_str, e
                            ),
                        );
                        lib_errors += 1;
                        continue;
                    }
                };
                let new_path = parent.join(&new_filename);
                let new_path_str = new_path.to_string_lossy().to_string();

                // Defensive guard: if the target somehow equals the
                // source (shouldn't — old is canonical, new is human),
                // skip.
                if new_path_str == old_path_str {
                    lib_skipped += 1;
                    continue;
                }

                // fs::rename FIRST. If it fails (file locked, perms,
                // long path), log and skip — DB updates won't fire.
                if let Err(e) = fs::rename(&old_path_str, &new_path_str) {
                    crate::search::diag_log(
                        db_path,
                        &format!(
                            "[mig003-step4] fs::rename failed: {} → {}: {}",
                            old_path_str, new_path_str, e
                        ),
                    );
                    lib_errors += 1;
                    continue;
                }

                // Cascade DB updates. note_meta first (the AU trigger
                // fires here, propagating to sky_nodes/sky_links). The
                // explicit dependent-table UPDATEs cover note_links /
                // note_aliases / note_embeddings — those have no
                // trigger of their own.
                if upd_meta
                    .execute(params![&old_path_str, &new_path_str])
                    .is_err()
                {
                    // DB write failed AFTER fs::rename succeeded —
                    // the on-disk state is now ahead of DB. Reindex
                    // on next boot will rebuild via cid_cn in
                    // frontmatter (Step 1 invariant). Log and move on.
                    crate::search::diag_log(
                        db_path,
                        &format!(
                            "[mig003-step4] note_meta UPDATE failed for {} (file already renamed)",
                            new_path_str
                        ),
                    );
                    lib_errors += 1;
                    continue;
                }
                let _ = upd_links_src.execute(params![&old_path_str, &new_path_str]);
                let _ = upd_links_tgt.execute(params![&old_path_str, &new_path_str]);
                let _ = upd_aliases.execute(params![&old_path_str, &new_path_str]);
                let _ = upd_embed.execute(params![&old_path_str, &new_path_str]);

                lib_renamed += 1;
                let outcome = RenameOutcome {
                    old_path: old_path_str.clone(),
                    new_path: new_path_str.clone(),
                    title: title.clone(),
                    cid_cn: cid_cn.clone(),
                };
                audit_lines.push_str(&format!(
                    "{}\t{}\t{}\t{}\n",
                    outcome.old_path, outcome.new_path, outcome.title, outcome.cid_cn
                ));

                if lib_renamed % 200 == 0 {
                    crate::search::diag_log(
                        db_path,
                        &format!(
                            "[mig003-step4] library '{}': {} renamed so far...",
                            lib_name, lib_renamed
                        ),
                    );
                }
            }
        }
        tx.commit()?;
        crate::search::diag_log(
            db_path,
            &format!(
                "[mig003-step4] library '{}' DONE: renamed={} skipped={} errors={} in {:?}",
                lib_name,
                lib_renamed,
                lib_skipped,
                lib_errors,
                lib_t.elapsed()
            ),
        );
        total_renamed += lib_renamed;
        total_skipped += lib_skipped;
        total_errors += lib_errors;
    }

    // Persist audit log to disk.
    let audit_path = db_dir.join("mig003-step4-renames.tsv");
    if let Err(e) = fs::write(&audit_path, &audit_lines) {
        crate::search::diag_log(
            db_path,
            &format!("[mig003-step4] audit log write failed: {}", e),
        );
    } else {
        crate::search::diag_log(
            db_path,
            &format!(
                "[mig003-step4] audit log written: {}",
                audit_path.to_string_lossy()
            ),
        );
    }

    crate::search::diag_log(
        db_path,
        &format!(
            "[mig003-step4] DONE — total renamed={} skipped={} errors={} elapsed={:?}",
            total_renamed,
            total_skipped,
            total_errors,
            t.elapsed()
        ),
    );

    Ok(())
}
