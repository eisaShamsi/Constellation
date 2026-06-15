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
//! **Safety (Working Agreement #4 — never ship a risky bulk DB mutation):**
//!   1. A row is a deletion candidate ONLY if it sits under a library root that
//!      is *currently accessible* (the root directory exists). If a drive is
//!      unmounted at boot, that library's rows match no accessible root and are
//!      skipped — never mass-deleted.
//!   2. A hard **safety cap**: if the candidate set exceeds 10 % of all rows or
//!      200 rows (whichever is larger), the pass ABORTS without deleting and logs
//!      a warning. A transient sync glitch that hides many files cannot cause a
//!      catastrophic purge; the few-row steady-state cleanup still runs.
//!   3. The disk existence checks run **lock-free** (the DB mutex is released
//!      while statting), so the scan never blocks user saves or other IPC.

use std::path::Path;
use std::thread;
use tauri::Manager;

use crate::search::{reindex_delete_note, SearchState};

/// Abort the pass if more than this fraction of all rows look stale.
const MAX_STALE_FRACTION: f64 = 0.10;
/// …or more than this many absolute rows (whichever bound is larger).
const MAX_STALE_ABSOLUTE: usize = 200;

/// Schedule the reconcile on a background thread. Returns immediately.
/// Called from `ensure_search_db_ready` after the connection is live.
pub fn maybe_schedule(app: tauri::AppHandle) {
    thread::spawn(move || match run(&app) {
        Ok(0) => {}
        Ok(n) => diag(&app, &format!("[reconcile] removed {} stale note_meta rows (file missing on disk)", n)),
        Err(e) => diag(&app, &format!("[reconcile] FAILED (non-fatal): {}", e)),
    });
}

fn run(app: &tauri::AppHandle) -> Result<usize, String> {
    // 1. Accessible library roots. If NONE are accessible (e.g. the universe
    //    drive is offline), do nothing — we must never delete on a bad mount.
    let libs = crate::libraries::load_all_libraries(app);
    let roots_norm: Vec<String> = libs
        .iter()
        .map(|l| l.path.clone())
        .filter(|p| Path::new(p).is_dir())
        .map(|p| p.replace('\\', "/").to_lowercase())
        .collect();
    if roots_norm.is_empty() {
        return Ok(0);
    }

    // 2. Snapshot all note paths under a brief lock, then release it.
    let state = app.state::<SearchState>();
    let all_paths: Vec<String> = {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB not initialized")?;
        let mut stmt = conn
            .prepare("SELECT path FROM note_meta")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.flatten().collect()
    };
    let total = all_paths.len();
    if total == 0 {
        return Ok(0);
    }

    // 3. Compute the stale set LOCK-FREE (disk stats outside the mutex). A row is
    //    a candidate only when it lives under an accessible library root and its
    //    file is missing.
    let mut stale: Vec<String> = Vec::new();
    for p in &all_paths {
        if p.is_empty() {
            continue;
        }
        let pn = p.replace('\\', "/").to_lowercase();
        let under_accessible = roots_norm
            .iter()
            .any(|r| pn == *r || pn.starts_with(&format!("{}/", r)));
        if !under_accessible {
            continue;
        }
        if !Path::new(p).exists() {
            stale.push(p.clone());
        }
    }
    if stale.is_empty() {
        return Ok(0);
    }

    // 4. Safety cap — refuse a suspiciously large purge (transient mount/sync).
    let cap = MAX_STALE_ABSOLUTE.max((total as f64 * MAX_STALE_FRACTION) as usize);
    if stale.len() > cap {
        diag(
            app,
            &format!(
                "[reconcile] ABORTED: {} of {} rows look stale (> cap {}). Refusing to purge — likely an offline drive or sync in progress.",
                stale.len(), total, cap
            ),
        );
        return Ok(0);
    }

    // 5. De-index each stale path via the canonical delete path (triggers cascade
    //    to FTS / sky; CTSE term cleanup). Per-row locking is fine for the small
    //    steady-state set the cap permits.
    let mut removed = 0usize;
    for p in &stale {
        match reindex_delete_note(&state, p) {
            Ok(_) => removed += 1,
            Err(e) => diag(app, &format!("[reconcile] failed to remove {}: {}", p, e)),
        }
    }
    Ok(removed)
}

/// Write a line to the universe's diagnostics log (mirrors `links_backfill::diag`).
fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}
