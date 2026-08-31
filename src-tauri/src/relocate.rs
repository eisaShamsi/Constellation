//! PJ-435 — repair a MOVED universe's index, safely, on one click.
//!
//! **Concept (the horse):** a universe carries its own mind with it — moving the folder must not
//! cost the user anything they built. The `.md` files and `earned.jsonl` already travel; what
//! stays behind is the INDEX's addressing: every path column still records the old root, plus
//! two things only the index holds — each link's `created` date and the path-keyed
//! `review_schedule`. A rebuild (Full re-read) destroys both, which is why the docs now warn
//! against it. This module repairs by REWRITE instead: same rows, new prefixes, everything
//! earned and everything dated carried through untouched.
//!
//! **No new engine.** This drives `mig108`'s proven rewrite — verified backup, one transaction,
//! in-transaction conservation proof — with a ONE-ENTRY journal. Three deliberate isolations
//! from the machinery it borrows:
//!   * its own journal file (`relocation-journal.json`) — landing in `mig108-journal.json`
//!     would make the boot resume machinery present a crashed relocation as a half-finished
//!     library unification;
//!   * its own backup directory (`relocation-backup`) — the default would have ROTATED the
//!     user's existing multi-GB `mig108-backup` aside;
//!   * no `run_move_phase` — the OS performed the move; the journal enters at `Phase::Moved`.
//!
//! **Idempotent by the fast path.** A crash between the DB commit and the record deletion
//! leaves the moved-notice armed; the user clicks again; the persisted journal proves both
//! rewrite stages committed and zero rows still carry the old prefix, so the second run
//! disarms WITHOUT re-running the engine — which also means the backup generation is never
//! rotated by a re-click (safety sweep 2026-08-30: three full re-runs would have rotated the
//! only genuine pre-repair backup to destruction). A crash EARLIER than the DB commit re-runs
//! the engine, where the conditional destination purge (spares rows with no old counterpart)
//! keeps that safe too. Re-clicking is the whole recovery story, by construction.

use std::path::Path;

/// What the repair did, for the receipt line and the frontend.
#[derive(serde::Serialize, Default)]
pub struct RelocateReport {
    pub old_root: String,
    pub new_root: String,
    /// Rows rewritten across all path columns (the engine's own count is per-table; this is
    /// the note_meta count, the number the user can check against the status bar).
    pub notes: i64,
    pub backup_dir: String,
}

pub const RELOCATE_BACKUP_DIR: &str = "relocation-backup";
pub const RELOCATE_JOURNAL: &str = "relocation-journal.json";

/// Remove the relocation record, and SAY SO if it cannot be removed. A swallowed
/// `let _ = remove_file` here used to leave the notice armed with now-false text while
/// suppressing the drift and phantom rows (safety sweep 2026-08-30). On this toolchain the
/// read-only attribute alone cannot cause that — std 1.94's `remove_file` deletes read-only
/// files (probe-verified 2026-08-30) — the attribute-clearing retry below is a belt for other
/// toolchains; the realistic failure is a sharing violation (a sync tool or AV holding the
/// file), which retrying once also gives a second chance to clear.
pub(crate) fn disarm_relocation(reloc_path: &Path) -> Result<(), String> {
    if std::fs::remove_file(reloc_path).is_ok() {
        return Ok(());
    }
    if let Ok(md) = std::fs::metadata(reloc_path) {
        let mut p = md.permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        p.set_readonly(false);
        let _ = std::fs::set_permissions(reloc_path, p);
    }
    std::fs::remove_file(reloc_path)
        .map_err(|e| format!("could not remove the relocation record: {e}"))
}

/// The one-click repair. `(async)` — this holds the writer lock for one large transaction and
/// must never run on the IPC dispatch thread (the note-open-freeze class).
#[tauri::command(async)]
pub fn repair_moved_universe(app: tauri::AppHandle) -> Result<RelocateReport, String> {
    use tauri::Manager as _;

    let root = crate::universe::active_universe_dir(&app)?;
    let cdir = root.join(".constellation");
    let reloc_path = cdir.join("relocation.json");
    let raw = std::fs::read_to_string(&reloc_path)
        .map_err(|_| "No relocation is recorded for this universe — nothing to repair.".to_string())?;
    let record: crate::universe::RelocationRecord =
        serde_json::from_str(&raw).map_err(|e| format!("The relocation record could not be read: {e}"))?;

    // Safety sweep 2026-08-30 (HIGH): the record must describe THIS folder. A copied
    // moved-but-unrepaired universe inherits a record whose `new_root` is the SOURCE folder —
    // consuming it verbatim would aim the whole rewrite at another universe's living root and
    // report success. (Activation now removes such a record too; this is the belt to that
    // suspender, because the record is read fresh here.)
    let root_str = root.to_string_lossy().to_string();
    let ours = crate::mig108::norm_under(&record.new_root, &root_str)
        && crate::mig108::norm_under(&root_str, &record.new_root);
    if !ours {
        // Re-inspection 2026-08-30: route through the reporting helper and never claim
        // "removed" unless it actually was.
        let removed = disarm_relocation(&reloc_path).is_ok();
        crate::reconcile::maybe_schedule(app.clone());
        return Err(format!(
            "The relocation record describes a different folder ({}) — this universe is at {}. {} \
             Nothing else was changed.",
            record.new_root,
            root_str,
            if removed {
                "The record was removed."
            } else {
                "The record could not be removed and may need deleting by hand \
                 (.constellation\\relocation.json)."
            }
        ));
    }

    let db_path = crate::search::db_path(&app)?;
    let state = app.state::<crate::search::SearchState>();
    // Hold the writer for the whole repair — the same freeze envelope the mig108 command uses.
    // Everything below is one transaction plus file copies; a search that waits is better than
    // a search that reads half a rewrite.
    let guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = guard
        .as_ref()
        .ok_or_else(|| "The search index is not open yet — try again in a moment.".to_string())?;

    // How many index rows still carry the old prefix — counted with the ENGINE'S OWN matcher
    // so this number cannot disagree with what the rewrite would do. It feeds the receipt
    // (honest observability: "remapped N" vs "nothing needed remapping") and the fast path.
    let stale_rows: i64 = {
        let mut stmt = conn.prepare("SELECT path FROM note_meta").map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.filter_map(Result::ok)
            .filter(|p| crate::mig108::norm_under(p, &record.old_root))
            .count() as i64
    };

    // Safety sweep 2026-08-30 (LOW×2): the already-repaired fast path. A re-click after a
    // crashed or failed disarm must not re-run the engine — `take_snapshot` rotates backup
    // generations, so a third full run would destroy the only genuine pre-repair backup.
    // The persisted journal reaching `JsonRewritten`/`Done` proves both rewrite stages
    // committed; zero stale rows proves the DB carries no old prefix. Housekeeping + disarm
    // is all that remains.
    let journal_done = std::fs::read_to_string(cdir.join(RELOCATE_JOURNAL))
        .ok()
        .and_then(|s| serde_json::from_str::<crate::mig108::Journal>(&s).ok())
        .map(|j| matches!(j.phase, crate::mig108::Phase::JsonRewritten | crate::mig108::Phase::Done))
        .unwrap_or(false);
    if stale_rows == 0 && journal_done {
        let _ = crate::search::init_db(&db_path); // idempotent; covers a crashed trigger window
        disarm_relocation(&reloc_path)?;
        let notes: i64 = conn
            .query_row("SELECT COUNT(*) FROM note_meta", [], |r| r.get(0))
            .unwrap_or(0);
        crate::search::diag_log(
            &db_path,
            &format!(
                "[relocate] PJ-435: re-click after a completed repair ({} → {}) — disarmed only; \
                 engine not re-run, backup generation untouched",
                record.old_root, record.new_root
            ),
        );
        drop(guard);
        crate::reconcile::maybe_schedule(app.clone());
        return Ok(RelocateReport {
            old_root: record.old_root,
            new_root: record.new_root,
            notes,
            backup_dir: RELOCATE_BACKUP_DIR.to_string(),
        });
    }

    // One entry: the universe root itself. The OS already moved it; the journal enters past
    // the move phase.
    let mut journal = crate::mig108::Journal {
        version: 1,
        universe_root: record.new_root.clone(),
        phase: crate::mig108::Phase::Moved,
        snapshot_db: None,
        json_backups: Vec::new(),
        entries: vec![crate::mig108::JournalEntry {
            library_id: "pj435-relocation".to_string(),
            library_name: "universe".to_string(),
            old_path: record.old_root.clone(),
            new_path: record.new_root.clone(),
            action: "move".to_string(),
            moved: true,
            started: true,
            copied: false,
        }],
        baseline: None,
        last_error: None,
        json_rewritten: Vec::new(),
        journal_file: Some(RELOCATE_JOURNAL.to_string()),
    };

    // Verified backup FIRST — into this repair's OWN directory.
    let (db_backup, json_backups, baseline) =
        crate::mig108::take_snapshot(conn, &db_path, &cdir, RELOCATE_BACKUP_DIR)?;
    journal.snapshot_db = Some(db_backup);
    journal.json_backups = json_backups;
    journal.baseline = Some(baseline);
    journal.save(&cdir)?;

    // The proven rewrite: one transaction, conservation verified inside it, then the eight
    // JSON stores (collections, review-pulse, workspaces, sessions, settings, bookmarks …).
    crate::mig108::run_db_rewrite(conn, &mut journal, &cdir)?;
    crate::mig108::run_json_rewrites(&mut journal, &cdir)?;

    let notes: i64 = conn
        .query_row("SELECT COUNT(*) FROM note_meta", [], |r| r.get(0))
        .unwrap_or(0);

    // Recreate the triggers the rewrite dropped — the same post-commit step mig108's own
    // command performs. Retried once; a repeat failure is written to the diagnostics log
    // (safety sweep 2026-08-30: a bare eprintln left the live session silently running
    // without the sky/outgoing triggers until restart — saves in that window skip derived
    // maintenance and the next boot does not replay them).
    if let Err(e) = crate::search::init_db(&db_path) {
        if let Err(e2) = crate::search::init_db(&db_path) {
            crate::search::diag_log(
                &db_path,
                &format!(
                    "[relocate] PJ-435: post-repair init_db failed twice ({e}; then {e2}) — \
                     derived-surface triggers are absent until the app restarts; notes saved \
                     in this session may need the next boot's reconcile to catch up"
                ),
            );
        }
    }

    // Disarm ONLY after everything committed. A crash before this line leaves the notice
    // armed and the next click takes the fast path above (disarm-only, backup untouched).
    // A FAILED disarm must not fail the command — the repair itself succeeded and the
    // failure text would falsely say nothing was changed; log it and let the fast path
    // finish the disarm on the next click.
    if let Err(e) = disarm_relocation(&reloc_path) {
        crate::search::diag_log(
            &db_path,
            &format!(
                "[relocate] PJ-435: repair committed but the record could not be removed ({e}) — \
                 the moved notice will re-appear; the next click disarms without re-running"
            ),
        );
    }
    crate::search::diag_log(
        &db_path,
        &format!(
            "[relocate] PJ-435: index repaired after a universe move: {} → {} ({} notes, {} rows \
             remapped; backup in {})",
            record.old_root, record.new_root, notes, stale_rows, RELOCATE_BACKUP_DIR
        ),
    );
    drop(guard);

    // A fresh walk at the new paths replaces the report — the moved row disarms, and the drift
    // row has nothing left to miscount.
    crate::reconcile::maybe_schedule(app.clone());

    Ok(RelocateReport {
        old_root: record.old_root,
        new_root: record.new_root,
        notes,
        backup_dir: RELOCATE_BACKUP_DIR.to_string(),
    })
}

/// The doc-block above promises Phase enters at `Moved` and never touches the move phase; pin
/// the two structural facts a refactor would most plausibly break.
#[cfg(test)]
mod tests_pj435_relocate {
    /// Truncate at the test module AND build the forbidden token by concatenation — a negative
    /// assertion whose own literal contains the token is green forever (this self-matching trap
    /// has now been caught three times in this codebase; twice by reverting a fix and finding
    /// the test still passing, once — here — before first run).
    #[test]
    fn the_command_isolates_itself_from_mig108s_resume_machinery() {
        let src = include_str!("relocate.rs");
        let code = &src[..src.find("#[cfg(test)]").unwrap_or(src.len())];
        assert!(
            code.contains("journal_file: Some(RELOCATE_JOURNAL.to_string())"),
            "the relocate journal must NEVER land in mig108-journal.json"
        );
        assert!(
            code.contains("take_snapshot(conn, &db_path, &cdir, RELOCATE_BACKUP_DIR)"),
            "the snapshot must go to the relocate backup dir, not rotate mig108's aside"
        );
        let forbidden = concat!("run_move_", "phase(");
        assert!(
            !code.contains(forbidden),
            "the OS performed the move; running the move phase would re-move directories"
        );
        // Safety sweep 2026-08-30: the two guards that must never be refactored away — the
        // record must describe THIS folder (a copied record aims the rewrite at another
        // universe's living root), and a re-click must take the disarm-only fast path (a full
        // re-run rotates the only genuine pre-repair backup toward destruction).
        assert!(
            code.contains("norm_under(&record.new_root, &root_str)"),
            "the repair must refuse a record that describes a different folder"
        );
        assert!(
            code.contains("if stale_rows == 0 && journal_done"),
            "a completed repair must disarm without re-running the engine"
        );
    }

    /// The disarm must remove the record even with a read-only attribute set (sync/backup
    /// tools stamp one). On Rust 1.94 std's `remove_file` already handles the attribute
    /// (probe-verified 2026-08-30), so this pins the guarantee rather than the mechanism —
    /// the helper's own retry covers toolchains where std does not.
    #[test]
    fn disarm_clears_a_readonly_attribute_and_removes_the_record() {
        let td = tempfile::tempdir().unwrap();
        let p = td.path().join("relocation.json");
        std::fs::write(&p, b"{}").unwrap();
        let mut perms = std::fs::metadata(&p).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&p, perms).unwrap();

        super::disarm_relocation(&p).expect("read-only must not defeat the disarm");
        assert!(!p.exists(), "the record must actually be gone");
    }
}
