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
        Ok((0, 0, 0)) => {}
        Ok((relocated, readopted, removed)) => diag(
            &app,
            &format!(
                "[reconcile] healed index drift: {} relocated + {} re-adopted (by cid_cn), {} removed (note truly gone)",
                relocated, readopted, removed
            ),
        ),
        Err(e) => diag(&app, &format!("[reconcile] FAILED (non-fatal): {}", e)),
    });
}

/// Returns `(relocated, readopted, removed)`.
fn run(app: &tauri::AppHandle) -> Result<(usize, usize, usize), String> {
    // 1. Accessible library roots (name, path). If NONE are accessible (e.g. the
    //    universe drive is offline), do nothing — never touch rows on a bad mount.
    let libs = crate::libraries::load_all_libraries(app);
    let roots: Vec<(String, String)> = libs
        .iter()
        .filter(|l| Path::new(&l.path).is_dir())
        .map(|l| (l.name.clone(), l.path.clone()))
        .collect();
    if roots.is_empty() {
        return Ok((0, 0, 0));
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
        return Ok((0, 0, 0)); // empty index — the initial reindex owns population.
    }
    let known: HashSet<String> = rows.iter().map(|(p, _)| norm(p)).collect();

    // 3. Dead rows — LOCK-FREE per-path stat. Stat each note_meta path INDIVIDUALLY
    //    (never infer "dead" from a walk's completeness — a read_dir error on one
    //    subdir would then make its files look dead and get removed). Only rows
    //    under an accessible root are candidates (never touch a bad mount).
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

    // 4. Orphan files — walk the accessible roots (lock-free) for `.md` files NOT in
    //    note_meta: the surviving half of a lost-tail rename whose dead row a prior
    //    reconcile already removed. Directory listing is cheap; frontmatter (the
    //    cid) is read only for orphans.
    let mut orphans: Vec<(String, String)> = Vec::new(); // (actual path, cid_cn)
    let mut seen: HashSet<String> = HashSet::new();
    let mut walk_complete = true; // false if any subtree failed to list (→ don't remove)
    for (_, root) in &roots {
        // Walk only TOP-LEVEL roots — skip a root nested under another (universe_notes
        // at the root + a sub-folder library): the parent walk already covers it, so
        // we don't read_dir the overlap twice. lib_for still attributes via ALL roots.
        let rn = norm(root);
        if roots.iter().any(|(_, other)| { let on = norm(other); on != rn && under(&rn, &on) }) {
            continue;
        }
        collect_md(Path::new(root), &known, &mut orphans, &mut seen, &mut walk_complete, 0);
    }

    if stale.is_empty() && orphans.is_empty() {
        return Ok((0, 0, 0)); // index matches disk — nothing to do.
    }

    // 5. Safety caps (WA#4) — a suspiciously large set in EITHER direction means a
    //    transient mount/sync or a mid-initial-index race, not steady-state drift.
    let cap = MAX_STALE_ABSOLUTE.max((total as f64 * MAX_STALE_FRACTION) as usize);
    if stale.len() > cap {
        diag(app, &format!("[reconcile] ABORTED: {} of {} rows look stale (> cap {}). Refusing to touch — offline drive or sync in progress.", stale.len(), total, cap));
        return Ok((0, 0, 0));
    }

    // 6. cid_cn → orphan path (first wins), for relocating a STILL-present dead row
    //    onto its current file.
    let mut orphan_by_cid: HashMap<String, String> = HashMap::new();
    for (p, cid) in &orphans {
        if !cid.is_empty() {
            orphan_by_cid.entry(cid.clone()).or_insert_with(|| p.clone());
        }
    }
    let mut consumed: HashSet<String> = HashSet::new(); // orphan paths taken by a relocate

    // 7. Relocate each dead row whose cid_cn has a live orphan file (preserves the
    //    row's aux data — review history, links). A relocate that FAILS is LEFT for
    //    next boot — NEVER falls to remove: falling to remove would destroy exactly
    //    the aux relocate exists to preserve, for a note that still exists. [audit]
    let mut relocated = 0usize;
    let mut relocate_failed = 0usize;
    let mut remove: Vec<String> = Vec::new();
    for (dead, cid) in &stale {
        // Empty-cid rows have no identity to relocate by, so they land in
        // `remove`. PJ-153 (MIG-105 C6): the init_db boot healer now INJECTS
        // cid_cn into every knowledge note that lacks one (and it runs before
        // this reconcile — proven boot order), so the only rows that can still
        // arrive here empty are kind-template rows (empty BY DESIGN, MIG-TPL
        // §1 — a mold's identity IS its content; remove + re-adopt is lossless
        // for it) and genuinely-deleted notes. A knowledge note can no longer
        // be dropped here for lacking an identity.
        let target = if cid.is_empty() { None } else { orphan_by_cid.get(cid).cloned() };
        match target {
            Some(new_path) => {
                let res = {
                    let guard = state.db.lock().map_err(|e| e.to_string())?;
                    let conn = guard.as_ref().ok_or("DB not initialized")?;
                    relocate_row(conn, dead, &new_path)
                };
                match res {
                    Ok(()) => {
                        let np = norm(&new_path);
                        // Reindex the new path to refresh name/body (re-locks internally,
                        // so it runs AFTER the relocate lock is released).
                        if let Some(lib_name) = lib_for(&roots, &np) {
                            let _ = reindex_single_note(&state, &new_path, lib_name);
                        }
                        consumed.insert(np); // this orphan is the relocated row — don't re-adopt it
                        relocated += 1;
                    }
                    Err(e) => {
                        // PJ-151 (2026-07-26): this arm discarded the error for ~3 weeks
                        // while asserting "target busy/contended" — wrong in 100% of the
                        // 1,591 logged cases (live data shows NO row at any target path).
                        // Surface the REAL error so the failing class can be named; keep
                        // the dead row + its aux for retry next boot. Never fall to remove.
                        relocate_failed += 1;
                        if relocate_failed <= 20 {
                            let kind = match e {
                                // relocate_row's two sentinels, distinguished so the log
                                // says WHICH invariant stopped the heal.
                                rusqlite::Error::InvalidQuery => "target OCCUPIED (guard)",
                                rusqlite::Error::StatementChangedRows(0) => {
                                    "cascade moved NOTHING (see [migrate_note_db_paths] lines above)"
                                }
                                _ => "DB error",
                            };
                            diag(app, &format!(
                                "[reconcile] relocate FAILED ({kind}) {dead} -> {new_path}: {e:?} — kept for retry"
                            ));
                        }
                    }
                }
            }
            None => remove.push(dead.clone()), // no orphan with this cid — removal CANDIDATE
        }
    }
    if relocate_failed > 20 {
        diag(app, &format!(
            "[reconcile] …plus {} more relocate failures this boot (first 20 detailed above)",
            relocate_failed - 20
        ));
    }

    // 8. De-index the truly-gone — but ONLY when the walk was COMPLETE (an
    //    incomplete walk could hide a renamed note's moved file, turning a relocate
    //    into a destructive remove) AND a fresh re-stat still shows the file gone
    //    (guards a transient stat error that falsely marked a live note dead). Both
    //    guard against destroying review history for a note that isn't actually gone.
    //    [audit HIGH + MED]
    let mut removed = 0usize;
    if walk_complete {
        for p in &remove {
            if Path::new(p).exists() {
                continue; // transient stat earlier — the file is there; keep the row.
            }
            match reindex_delete_note(&state, p) {
                Ok(_) => removed += 1,
                Err(e) => diag(app, &format!("[reconcile] failed to remove {}: {}", p, e)),
            }
        }
    } else if !remove.is_empty() {
        diag(app, &format!("[reconcile] walk INCOMPLETE (a subtree failed to list) — skipping {} removal(s) to protect aux; phantoms left for a clean pass.", remove.len()));
    }

    // 9. RE-ADOPT orphans NOT consumed by a relocate — index the file fresh. Its
    //    note_meta row was already deleted by a prior reconcile, so there was
    //    nothing to relocate; the file on disk is the source of truth (File-Over-
    //    App). Capped: a huge orphan set is a mid-initial-index race, not drift —
    //    the initial reindex owns that, so skip re-adopt there.
    let mut readopted = 0usize;
    let mut readopt_failed = 0usize;
    if orphans.len() <= cap {
        for (p, _cid) in &orphans {
            let np = norm(p);
            if consumed.contains(&np) {
                continue;
            }
            if let Some(lib_name) = lib_for(&roots, &np) {
                match reindex_single_note(&state, p, lib_name) {
                    Ok(_) => readopted += 1,
                    Err(e) => {
                        // PJ-154 (2026-07-26): this Err was 100% silent — an orphan that
                        // can never index (e.g. a cid_cn UNIQUE collision with a dead row)
                        // stayed invisible to search with no trace. Surface it, bounded.
                        readopt_failed += 1;
                        if readopt_failed <= 20 {
                            diag(app, &format!("[reconcile] re-adopt FAILED {}: {}", p, e));
                        }
                    }
                }
            }
        }
    } else {
        diag(app, &format!("[reconcile] {} orphan files (> cap {}) — skipping re-adopt (a full reindex is the right tool).", orphans.len(), cap));
    }

    // PJ-151 (2026-07-26): an all-deferred boot used to be COMPLETELY invisible —
    // the (0,0,0) tuple looked like "nothing to do" while every relocate failed.
    // Any failure now forces a boot summary regardless of the healed counts.
    if relocate_failed > 0 || readopt_failed > 0 {
        diag(app, &format!(
            "[reconcile] boot summary: {} relocated, {} re-adopted, {} removed — {} relocate FAILURES, {} re-adopt failures (details above)",
            relocated, readopted, removed, relocate_failed, readopt_failed
        ));
    }

    Ok((relocated, readopted, removed))
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
    // PJ-149 B / Stage-0 C5 (2026-07-26): this was a DUPLICATE 5-table cascade that
    // had already drifted from the canonical one (no note_body/summaries/history/
    // layout/shape/suggestions — the relocated note's earned aux stayed stranded at
    // the dead path). Delegate to the ONE shared cascade so the two surfaces can
    // never drift again (the Whole-Ecosystem law). The helper's note_meta destination
    // pre-delete is a no-op here — the occupied-guard above already proved the
    // destination row-free. Accepted trade (build-spec §2-C5): per-statement error
    // propagation becomes logged-best-effort inside this still-atomic envelope.
    crate::libraries::migrate_note_db_paths(conn, old, new);
    // VERIFY, then report. The shared cascade is best-effort by contract (one failed
    // statement must never abort a user's rename), so it cannot signal failure to us —
    // and on 2026-07-26 that turned this function into a liar: FK enforcement refused
    // every parent-path UPDATE, the cascade logged and moved on, this returned Ok, and
    // reconcile reported "14 relocated" on a boot where NOTHING moved. A success this
    // function reports must be a fact it checked.
    let moved: bool = conn
        .query_row("SELECT 1 FROM note_meta WHERE path = ?1", [new], |_| Ok(true))
        .unwrap_or(false);
    if !moved {
        let _ = conn.execute_batch("ROLLBACK");
        return Err(rusqlite::Error::StatementChangedRows(0));
    }
    conn.execute_batch("COMMIT")?;
    Ok(())
}

/// The most-specific (longest-path) accessible library whose root contains the
/// normalized path `np`, or None. Longest wins so a note in a nested library is
/// attributed to THAT library, not its parent (e.g. universe_notes at the root).
fn lib_for<'a>(roots: &'a [(String, String)], np: &str) -> Option<&'a str> {
    roots
        .iter()
        .filter(|(_, rp)| under(np, &norm(rp)))
        .max_by_key(|(_, rp)| rp.len())
        .map(|(name, _)| name.as_str())
}

/// Recursively walk `.md` files under `dir`, pushing `(path, cid_cn)` for files
/// NOT in `known` (note_meta) to `orphans` (→ relocate a surviving dead row, or
/// re-adopt). Frontmatter (for the cid) is read only for orphan files. Skips
/// hidden entries (`.trash`, `.constellation`).
///
/// `seen` dedupes across OVERLAPPING roots (universe_notes at the root + a nested
/// registered library) so a file is visited once. `complete` is set false on ANY
/// read_dir error or depth cutoff — the caller must NOT remove dead rows from an
/// incomplete walk (a hidden subtree could hold a renamed note's moved file, and
/// removing its row would destroy aux the walk simply failed to surface). [audit]
fn collect_md(
    dir: &Path,
    known: &HashSet<String>,
    orphans: &mut Vec<(String, String)>,
    seen: &mut HashSet<String>,
    complete: &mut bool,
    depth: u32,
) {
    if depth > 20 {
        *complete = false; // truncated — a deeper file is unseen; don't trust removal.
        return;
    }
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => {
            *complete = false; // this subtree is unseen; don't trust removal for it.
            return;
        }
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_md(&path, known, orphans, seen, complete, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let ps = path.to_string_lossy().to_string();
            let pn = norm(&ps);
            if !seen.insert(pn.clone()) {
                continue; // already visited via an overlapping root
            }
            if !known.contains(&pn) {
                // Orphan — read its cid_cn (empty for a cid-free note).
                let cid = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|c| extract_frontmatter_cid_cn(&c))
                    .unwrap_or_default();
                orphans.push((ps, cid));
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

    /// The disk walk pushes (path, cid) for files NOT in `known` to `orphans`
    /// (→ relocate / re-adopt), skipping already-indexed files. This is what lets
    /// the reconcile recover a note whose dead row a prior pass already removed.
    #[test]
    fn collect_md_finds_orphans_skips_indexed() {
        let dir = std::env::temp_dir().join(format!("mig098_md_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let orphan = dir.join("relocated note.md");
        std::fs::write(&orphan, "---\ntitle: Relocated\ncid_cn: CIDNEW\nkind: note\n---\nbody").unwrap();
        let known_file = dir.join("already indexed.md");
        std::fs::write(&known_file, "---\ntitle: Known\ncid_cn: CIDOLD\nkind: note\n---\nbody").unwrap();

        let mut known = HashSet::new();
        known.insert(norm(&known_file.to_string_lossy()));
        let mut orphans: Vec<(String, String)> = Vec::new();
        let mut seen = HashSet::new();
        let mut complete = true;
        collect_md(&dir, &known, &mut orphans, &mut seen, &mut complete, 0);

        assert!(complete, "a clean walk reports complete");
        assert_eq!(orphans.len(), 1, "only the unindexed file is an orphan");
        assert_eq!(orphans[0].1, "CIDNEW", "orphan carries its cid_cn for relocate/re-adopt");
        assert_eq!(norm(&orphans[0].0), norm(&orphan.to_string_lossy()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `lib_for` attributes a path to the MOST-SPECIFIC (longest) containing root,
    /// so a note in a nested library isn't mis-attributed to the universe_notes root.
    #[test]
    fn lib_for_prefers_the_nested_library() {
        let roots = vec![
            ("universe_notes".to_string(), "E:/U".to_string()),
            ("Nested".to_string(), "E:/U/Nested".to_string()),
        ];
        assert_eq!(lib_for(&roots, &norm("E:/U/Nested/note.md")), Some("Nested"));
        assert_eq!(lib_for(&roots, &norm("E:/U/top.md")), Some("universe_notes"));
        assert_eq!(lib_for(&roots, &norm("E:/Other/x.md")), None);
    }
}
