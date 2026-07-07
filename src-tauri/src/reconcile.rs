//! MIG-078 §A′.2 — Reconcile `note_meta` against disk (File-Over-App self-heal).
//!
//! The Map/OrgChart tree is now assembled from `note_meta` (MIG-078 §A′), so any
//! row whose `.md` file no longer exists on disk shows up as a *phantom* note.
//! The old disk-walk masked these because it only emitted notes it found on disk;
//! reading the index directly exposes the drift. Such drift accumulates from
//! out-of-app changes (a rename/delete via Explorer, git, Syncthing) and from
//! historical bugs that left orphan rows.
//!
//! This module removes those stale rows in the background, after first paint,
//! using the SAME canonical de-index path a normal delete uses
//! (`reindex_delete_note` → drops `note_links` + `note_meta`, fires the FTS /
//! sky triggers, runs CTSE term cleanup). `.md` files on disk remain the source
//! of truth; a stale row is just an index entry pointing at a file that is gone,
//! and a future re-index re-adds any note that actually exists.
//!
//! Scheduled by `ensure_search_db_ready` (runs once per universe-open). Operates
//! only on the ACTIVE universe's `note_meta`; child universes self-heal when they
//! are themselves the active universe.
//!
//! **MIG-097 — rename-drift RELOCATE (2026-07-07).** A rename writes the file
//! immediately (gated) but updates the index in a *detached, best-effort* tail
//! (§B2-4, to avoid a freeze on large libraries). On a busy 2 GB library that
//! tail can be starved/lost, and because gated renames deliberately suppress the
//! watcher, nothing heals it — the row is left at the OLD (now-dead) path with
//! the OLD name, while the file lives at a NEW path with the SAME `cid_cn`.
//! Boss-reproduced 2026-07-07 (Reviewer rename → row reverted to old name on
//! reopen, opening it hit the dead path → empty Dashboard; disk was correct).
//! Removing the dead row (the MIG-078 behaviour) would drop the note — AND its
//! review history / links — from the index until a future reindex. So this pass
//! now first tries to **relocate** each dead row to its current file, matched by
//! the stable `cid_cn`, preserving the row's aux data; only rows whose note is
//! genuinely gone (no file with that cid) fall back to removal.
//!
//! **Safety (Working Agreement #4 — never ship a risky bulk DB mutation):**
//!   1. A row is a candidate ONLY if it sits under a library root that is
//!      *currently accessible* (the root directory exists). If a drive is
//!      unmounted at boot, that library's rows match no accessible root and are
//!      skipped — never mass-touched.
//!   2. A hard **safety cap**: if the candidate set exceeds 10 % of all rows or
//!      200 rows (whichever is larger), the pass ABORTS without touching anything
//!      and logs a warning. A transient sync glitch that hides many files cannot
//!      cause a catastrophic purge/relocate; the few-row steady-state heal runs.
//!   3. The disk existence checks + the orphan walk run **lock-free** (the DB
//!      mutex is released while statting), so the scan never blocks user IPC.
//!   4. Relocation never overwrites an existing row (orphans have none by
//!      definition; guarded anyway) and runs in a transaction (all-or-nothing).

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::thread;
use tauri::Manager;

use crate::search::{extract_frontmatter_cid_cn, reindex_delete_note, reindex_single_note, SearchState};

/// Abort the pass if more than this fraction of all rows look stale.
const MAX_STALE_FRACTION: f64 = 0.10;
/// …or more than this many absolute rows (whichever bound is larger).
const MAX_STALE_ABSOLUTE: usize = 200;

fn norm(p: &str) -> String {
    p.replace('\\', "/").to_lowercase()
}

/// `true` when `path` sits at or under `root` (bounded at a separator, so
/// "…/Research" never matches "…/Research Notes"). Both args already normalized.
fn under(path_norm: &str, root_norm: &str) -> bool {
    path_norm == root_norm || path_norm.starts_with(&format!("{}/", root_norm))
}

/// Schedule the reconcile on a background thread. Returns immediately.
/// Called from `ensure_search_db_ready` after the connection is live.
pub fn maybe_schedule(app: tauri::AppHandle) {
    thread::spawn(move || match run(&app) {
        Ok((0, 0)) => {}
        Ok((relocated, removed)) => diag(
            &app,
            &format!(
                "[reconcile] healed index drift: {} row(s) relocated to their current file (by cid_cn), {} stale row(s) removed (note truly gone)",
                relocated, removed
            ),
        ),
        Err(e) => diag(&app, &format!("[reconcile] FAILED (non-fatal): {}", e)),
    });
}

/// Returns `(relocated, removed)`.
fn run(app: &tauri::AppHandle) -> Result<(usize, usize), String> {
    // 1. Accessible library roots (name, path). If NONE are accessible (e.g. the
    //    universe drive is offline), do nothing — never touch rows on a bad mount.
    let libs = crate::libraries::load_all_libraries(app);
    let roots: Vec<(String, String)> = libs
        .iter()
        .filter(|l| Path::new(&l.path).is_dir())
        .map(|l| (l.name.clone(), l.path.clone()))
        .collect();
    if roots.is_empty() {
        return Ok((0, 0));
    }
    let roots_norm: Vec<String> = roots.iter().map(|(_, p)| norm(p)).collect();

    // 2. Snapshot (path, cid_cn) under a brief lock, then release it.
    let state = app.state::<SearchState>();
    let rows: Vec<(String, String)> = {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB not initialized")?;
        let mut stmt = conn
            .prepare("SELECT path, COALESCE(cid_cn, '') FROM note_meta")
            .map_err(|e| e.to_string())?;
        let r = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        r.flatten().collect()
    };
    let total = rows.len();
    if total == 0 {
        return Ok((0, 0));
    }
    let known: HashSet<String> = rows.iter().map(|(p, _)| norm(p)).collect();

    // 3. Compute the stale set LOCK-FREE (disk stats outside the mutex). A row is a
    //    candidate only when it lives under an accessible root and its file is gone.
    let mut stale: Vec<(String, String)> = Vec::new();
    for (p, cid) in &rows {
        if p.is_empty() {
            continue;
        }
        let pn = norm(p);
        if !roots_norm.iter().any(|r| under(&pn, r)) {
            continue;
        }
        if !Path::new(p).exists() {
            stale.push((p.clone(), cid.clone()));
        }
    }
    if stale.is_empty() {
        return Ok((0, 0));
    }

    // 4. Safety cap — refuse a suspiciously large set (transient mount/sync).
    let cap = MAX_STALE_ABSOLUTE.max((total as f64 * MAX_STALE_FRACTION) as usize);
    if stale.len() > cap {
        diag(
            app,
            &format!(
                "[reconcile] ABORTED: {} of {} rows look stale (> cap {}). Refusing to touch — likely an offline drive or sync in progress.",
                stale.len(), total, cap
            ),
        );
        return Ok((0, 0));
    }

    // 5. Build cid_cn → current-file map from ORPHAN files (on disk, not in
    //    note_meta) — the other half of a lost-tail rename. Walk only now (drift
    //    exists); read frontmatter only for orphan files (few). Lock-free.
    let mut orphan_by_cid: HashMap<String, String> = HashMap::new();
    for (_, root) in &roots {
        collect_orphans(Path::new(root), &known, &mut orphan_by_cid, 0);
    }

    // 6. Relocate each dead row whose cid_cn has a live orphan file (preserves the
    //    row's aux data — review history, links); collect the rest for removal.
    let mut relocated = 0usize;
    let mut remove: Vec<String> = Vec::new();
    for (dead, cid) in &stale {
        let target = if cid.is_empty() { None } else { orphan_by_cid.get(cid).cloned() };
        match target {
            Some(new_path) => {
                let ok = {
                    let guard = state.db.lock().map_err(|e| e.to_string())?;
                    let conn = guard.as_ref().ok_or("DB not initialized")?;
                    relocate_row(conn, dead, &new_path).is_ok()
                };
                if ok {
                    // Reindex the new path to refresh name/body/etc. (re-locks
                    // internally, so it runs AFTER the relocate lock is released).
                    let np = norm(&new_path);
                    if let Some((lib_name, _)) = roots.iter().find(|(_, rp)| under(&np, &norm(rp))) {
                        let _ = reindex_single_note(&state, &new_path, lib_name);
                    }
                    relocated += 1;
                } else {
                    remove.push(dead.clone());
                }
            }
            None => remove.push(dead.clone()),
        }
    }

    // 7. De-index the truly-gone via the canonical delete path (FTS / sky cascade,
    //    CTSE term cleanup). Per-row locking is fine for the capped set.
    let mut removed = 0usize;
    for p in &remove {
        match reindex_delete_note(&state, p) {
            Ok(_) => removed += 1,
            Err(e) => diag(app, &format!("[reconcile] failed to remove {}: {}", p, e)),
        }
    }
    Ok((relocated, removed))
}

/// Migrate a `note_meta` row + its path-keyed aux rows from `old` to `new` — a
/// lost-tail rename left the row at a dead path while the file moved to `new`
/// with the SAME cid_cn. Mirrors `rename_item_db_tail`'s path cascade; the caller
/// reindexes `new` afterward to refresh name/body. Transactional (all-or-nothing);
/// never overwrites an existing row at `new`.
fn relocate_row(conn: &rusqlite::Connection, old: &str, new: &str) -> rusqlite::Result<()> {
    let occupied: bool = conn
        .query_row("SELECT 1 FROM note_meta WHERE path = ?1", [new], |_| Ok(true))
        .unwrap_or(false);
    if occupied {
        return Err(rusqlite::Error::InvalidQuery);
    }
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let res = (|| -> rusqlite::Result<()> {
        conn.execute("UPDATE note_meta SET path = ?2 WHERE path = ?1", [old, new])?;
        conn.execute("UPDATE note_links SET source_path = ?2 WHERE source_path = ?1", [old, new])?;
        conn.execute("UPDATE note_aliases SET path = ?2 WHERE path = ?1", [old, new])?;
        conn.execute("UPDATE note_embeddings SET path = ?2 WHERE path = ?1", [old, new])?;
        // review_schedule carries the note's ✓ history; migrate it (gated on the
        // stamp, mirroring rename_item_db_tail). Clear any stale row at `new` first.
        if crate::review::is_stamped(conn) {
            conn.execute("DELETE FROM review_schedule WHERE path = ?1", [new])?;
            conn.execute("UPDATE review_schedule SET path = ?2 WHERE path = ?1", [old, new])?;
        }
        Ok(())
    })();
    match res {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

/// Recursively collect `.md` files under `dir` that are NOT already in `known`
/// (note_meta) — the orphan half of a lost-tail rename — mapping cid_cn → path.
/// Reads frontmatter only for orphans (files already indexed are skipped).
fn collect_orphans(dir: &Path, known: &HashSet<String>, out: &mut HashMap<String, String>, depth: u32) {
    if depth > 20 {
        return;
    }
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_orphans(&path, known, out, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let ps = path.to_string_lossy().to_string();
            if known.contains(&norm(&ps)) {
                continue; // already indexed — not an orphan
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(cid) = extract_frontmatter_cid_cn(&content) {
                    if !cid.is_empty() {
                        out.entry(cid).or_insert(ps); // first file for a cid wins
                    }
                }
            }
        }
    }
}

/// Write a line to the universe's diagnostics log (mirrors `links_backfill::diag`).
fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Minimal schema covering every table `relocate_row` migrates. (`is_stamped`
    /// returns false without `schema_versions`, so `review_schedule` is skipped.)
    fn schema(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT, cid_cn TEXT);
             CREATE TABLE note_links (source_path TEXT, target_name TEXT);
             CREATE TABLE note_aliases (path TEXT, alias_lower TEXT);
             CREATE TABLE note_embeddings (path TEXT, vec BLOB);",
        )
        .unwrap();
    }

    /// MIG-097 — a lost-tail rename leaves the row at a dead path; relocating it to
    /// the note's current file (by cid_cn) must migrate note_meta + every aux row,
    /// preserving the stable cid_cn (and thus review history / links).
    #[test]
    fn relocate_row_migrates_note_and_aux_by_path() {
        let conn = Connection::open_in_memory().unwrap();
        schema(&conn);
        let dead = "E:/lib/التجربة الثانية_إعادة تسمية.md";
        let new = "E:/lib/التجربة الثانية ن2.md";
        conn.execute("INSERT INTO note_meta(path,name,cid_cn) VALUES (?1,'التجربة الثانية','CID8878')", [dead]).unwrap();
        conn.execute("INSERT INTO note_links(source_path,target_name) VALUES (?1,'Foo')", [dead]).unwrap();
        conn.execute("INSERT INTO note_aliases(path,alias_lower) VALUES (?1,'x')", [dead]).unwrap();
        conn.execute("INSERT INTO note_embeddings(path,vec) VALUES (?1, x'00')", [dead]).unwrap();

        relocate_row(&conn, dead, new).unwrap();

        let cnt_dead: i64 = conn.query_row("SELECT COUNT(*) FROM note_meta WHERE path=?1", [dead], |r| r.get(0)).unwrap();
        let cnt_new: i64 = conn.query_row("SELECT COUNT(*) FROM note_meta WHERE path=?1", [new], |r| r.get(0)).unwrap();
        assert_eq!(cnt_dead, 0, "dead note_meta row removed");
        assert_eq!(cnt_new, 1, "note_meta relocated to the current file");
        let cid: String = conn.query_row("SELECT cid_cn FROM note_meta WHERE path=?1", [new], |r| r.get(0)).unwrap();
        assert_eq!(cid, "CID8878", "stable cid_cn preserved across relocate");
        let lnk: String = conn.query_row("SELECT source_path FROM note_links", [], |r| r.get(0)).unwrap();
        assert_eq!(lnk, new, "note_links.source_path migrated");
        let al: String = conn.query_row("SELECT path FROM note_aliases", [], |r| r.get(0)).unwrap();
        assert_eq!(al, new, "note_aliases.path migrated");
        let em: String = conn.query_row("SELECT path FROM note_embeddings", [], |r| r.get(0)).unwrap();
        assert_eq!(em, new, "note_embeddings.path migrated");
    }

    /// Never overwrite an existing row (orphans have none by definition; guard it).
    /// A refused relocate must leave BOTH rows intact (no data loss).
    #[test]
    fn relocate_row_refuses_occupied_target() {
        let conn = Connection::open_in_memory().unwrap();
        schema(&conn);
        conn.execute("INSERT INTO note_meta(path,name,cid_cn) VALUES ('a.md','A','C1')", []).unwrap();
        conn.execute("INSERT INTO note_meta(path,name,cid_cn) VALUES ('b.md','B','C2')", []).unwrap();
        assert!(relocate_row(&conn, "a.md", "b.md").is_err(), "must refuse an occupied target");
        let cnt: i64 = conn.query_row("SELECT COUNT(*) FROM note_meta", [], |r| r.get(0)).unwrap();
        assert_eq!(cnt, 2, "no row lost on refused relocate");
    }

    /// The orphan walk maps cid_cn → path for on-disk files NOT already indexed,
    /// and skips known (already-indexed) files.
    #[test]
    fn collect_orphans_maps_unknown_md_by_cid() {
        let dir = std::env::temp_dir().join(format!("mig097_orphan_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let orphan = dir.join("relocated note.md");
        std::fs::write(&orphan, "---\ntitle: Relocated\ncid_cn: CIDNEW\nkind: note\n---\nbody").unwrap();
        let known_file = dir.join("already indexed.md");
        std::fs::write(&known_file, "---\ntitle: Known\ncid_cn: CIDOLD\nkind: note\n---\nbody").unwrap();

        let mut known = HashSet::new();
        known.insert(norm(&known_file.to_string_lossy()));
        let mut out = HashMap::new();
        collect_orphans(&dir, &known, &mut out, 0);

        assert_eq!(out.get("CIDNEW").map(|p| norm(p)), Some(norm(&orphan.to_string_lossy())), "orphan mapped by cid");
        assert!(!out.contains_key("CIDOLD"), "already-indexed file skipped");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
