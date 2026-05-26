//! MIG-056 §C — Auto-migrate cUniverse schema-drift on first federated attach.
//!
//! **Highest-risk step in the MIG-056 cascade.** Writes to a search.db
//! belonging to a different universe than the active one — an explicit
//! deviation from Constellation's normal per-universe ownership model.
//! Architect §5.3 (Boss-locked) authorizes this with **four safeguards**,
//! all implemented here:
//!
//! 1. **Lock check** — `is_cuniverse_open_elsewhere(db)` tries
//!    `BEGIN EXCLUSIVE` against the cUniverse's search.db. If the lock
//!    can't be acquired (another process is holding read or write
//!    locks), bail with `MigrationError::CUniverseLocked`. The
//!    federation skips this cUniverse + surfaces a clear warning
//!    ("close it in the other window first").
//!
//! 2. **Pre-migration backup** — `fs::copy(db, db.pre-mig-056.bak)`.
//!    If the backup fails (disk full, permission denied), bail BEFORE
//!    touching the source DB. The backup is the only recovery path
//!    if migration corrupts the source.
//!
//! 3. **Atomic via backup** — `crate::search::init_db(path)` runs the
//!    full schema setup against the cUniverse's DB. init_db is
//!    idempotent + step-by-step; it CAN fail mid-migration if a
//!    specific schema change throws. On failure, we restore from
//!    the backup. The restore is the atomic boundary (not a single
//!    SQL transaction, which isn't possible across multiple
//!    migration steps).
//!
//! 4. **Audit log** — every migrate attempt writes a structured line
//!    to `{parent_universe}/.constellation/federation-audit.log`:
//!    ```
//!    2026-05-26T19:32:18Z  AUTO_MIGRATE         cuniverse=...  result=OK
//!    2026-05-26T19:42:18Z  AUTO_MIGRATE_FAILED  cuniverse=...  result=<msg>
//!    ```
//!    Per Architect §5.3: the parent universe's audit log records what
//!    was done so the user can audit cross-universe writes.
//!
//! ## Why init_db (not custom migrations)
//!
//! `crate::search::init_db` is the canonical schema setup function. It's
//! already idempotent (runs every boot on the active universe). Reusing
//! it for cUniverse auto-migrate guarantees:
//!
//! - Schema parity between active universe + auto-migrated cUniverses
//! - No drift between "init via boot" + "init via auto-migrate" paths
//! - Future schema changes apply to cUniverses without dual maintenance
//!
//! Trade-off: we don't have surgical "just upgrade from version X to Y";
//! init_db runs whatever steps are needed. For typical drift cases
//! (cUniverse skipped a release; missing some columns), this is fine
//! — init_db detects missing pieces + adds them.

use super::failure::MigrationError;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

/// Run schema migrations on a cUniverse's `search.db`, bringing it
/// to the current schema. Per Architect §5.3.
///
/// `cu_db_path` — full path to the cUniverse's `search.db` file.
/// `parent_universe_root` — the ACTIVE (parent) universe's root path,
/// used to locate the audit log destination.
///
/// On success: cUniverse's schema is brought to current; an audit
/// log entry is written; `Ok(())`. On failure: source DB is
/// restored from backup; failure logged; `Err(MigrationError)`.
pub fn run_migrations_on(
    cu_db_path: &Path,
    parent_universe_root: &Path,
) -> Result<(), MigrationError> {
    // ─── Safeguard 1: lock check ───
    if is_cuniverse_open_elsewhere(cu_db_path) {
        // Don't write the audit log for this case — nothing was
        // attempted. The federation warning surfaces the reason.
        return Err(MigrationError::CUniverseLocked);
    }

    // ─── Safeguard 2: pre-migration backup ───
    let backup_path = backup_path_for(cu_db_path);
    fs::copy(cu_db_path, &backup_path)
        .map_err(|e| MigrationError::BackupFailed(format!("copy to {:?} failed: {}", backup_path, e)))?;

    // ─── Run migration via init_db ───
    let migration_outcome: Result<(), String> = (|| {
        let conn = crate::search::init_db(cu_db_path)
            .map_err(|e| format!("init_db failed: {}", e))?;
        // Explicitly drop the connection to release the file lock
        // before we return — otherwise the file stays held when
        // run_migrations_on returns, blocking attach_with_safety's
        // re-open in the caller.
        drop(conn);
        Ok(())
    })();

    match migration_outcome {
        Ok(()) => {
            // ─── Safeguard 4 (success path): audit log ───
            write_audit_log(parent_universe_root, cu_db_path, "AUTO_MIGRATE", "OK")
                .map_err(MigrationError::AuditLogFailed)?;
            // Backup stays on disk for ~24h-style retention. Future
            // enhancement: GC backups older than N days. For v1, the
            // backup is permanent until the user removes it.
            Ok(())
        }
        Err(migration_err) => {
            // ─── Safeguard 3 (failure path): restore from backup ───
            let restore_result = fs::copy(&backup_path, cu_db_path);
            let audit_action = if restore_result.is_ok() {
                "AUTO_MIGRATE_FAILED_RESTORED"
            } else {
                "AUTO_MIGRATE_FAILED_RESTORE_FAILED"
            };
            // Audit-log the failure regardless of restore outcome.
            // If audit also fails, the original migration error wins
            // for reporting purposes (it's the more useful diagnostic).
            let _ = write_audit_log(
                parent_universe_root,
                cu_db_path,
                audit_action,
                &migration_err,
            );

            match restore_result {
                Ok(_) => Err(MigrationError::MigrationFailed(migration_err)),
                Err(restore_err) => Err(MigrationError::BackupRestoreFailed {
                    migration_error: migration_err,
                    restore_error: restore_err.to_string(),
                }),
            }
        }
    }
}

/// Try to acquire an exclusive lock on the cUniverse's search.db.
/// If we succeed, no other process is holding read or write locks →
/// safe to migrate. If we fail, something else has it open.
///
/// This is a best-effort check. WAL mode + multiple readers can
/// produce false positives (we think it's locked when only readers
/// hold it). For v1's expected single-user-single-process pattern,
/// this is sufficient. Future enhancements could inspect the `-shm`
/// file for active connections.
pub(crate) fn is_cuniverse_open_elsewhere(db_path: &Path) -> bool {
    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        // If we can't even open the file, we can't migrate either
        // — but that's not "locked elsewhere"; the safety check in
        // run_migrations_on's backup step will catch it differently.
        Err(_) => return false,
    };

    // Set a SHORT busy_timeout so BEGIN EXCLUSIVE fails fast if locked.
    let _ = conn.busy_handler(None);
    let _ = conn.execute_batch("PRAGMA busy_timeout = 100"); // 100ms

    match conn.execute_batch("BEGIN EXCLUSIVE; ROLLBACK;") {
        Ok(_) => false,
        Err(_) => true,
    }
}

/// Generate the backup path for a cUniverse's search.db.
/// `{search.db}` → `{search.db.pre-mig-056.bak}` (in the same dir).
fn backup_path_for(db_path: &Path) -> PathBuf {
    let mut backup = db_path.to_path_buf();
    let new_name = match db_path.file_name() {
        Some(n) => format!("{}.pre-mig-056.bak", n.to_string_lossy()),
        None => "search.db.pre-mig-056.bak".to_string(),
    };
    backup.set_file_name(new_name);
    backup
}

/// Append a structured audit entry to the parent universe's
/// `federation-audit.log`. Format: tab-separated timestamp + action +
/// cuniverse path + result. Per Architect §5.3 safeguard 4.
fn write_audit_log(
    parent_universe_root: &Path,
    cu_db_path: &Path,
    action: &str,
    result: &str,
) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let log_dir = parent_universe_root.join(".constellation");
    fs::create_dir_all(&log_dir)
        .map_err(|e| format!("create audit log dir failed: {}", e))?;
    let log_path = log_dir.join("federation-audit.log");

    let timestamp = chrono::Utc::now().to_rfc3339();
    let line = format!(
        "{}\t{}\tcuniverse={}\tresult={}\n",
        timestamp,
        action,
        cu_db_path.display(),
        result
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("open audit log failed: {}", e))?;
    file.write_all(line.as_bytes())
        .map_err(|e| format!("write audit log failed: {}", e))?;
    Ok(())
}

// ─── §C tests ───

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a cUniverse directory with a search.db that has only a
    /// minimal note_meta schema (missing required columns). This is
    /// the "drift" scenario that triggers auto-migrate.
    fn make_drifted_cuniverse(dir: &Path) -> PathBuf {
        let cdir = dir.join(".constellation");
        fs::create_dir_all(&cdir).unwrap();
        let db_path = cdir.join("search.db");
        let conn = Connection::open(&db_path).unwrap();
        // Minimal schema — missing required cols
        conn.execute_batch("CREATE TABLE note_meta (path TEXT PRIMARY KEY);").unwrap();
        db_path
    }

    /// Create a parent universe directory with .constellation/.
    fn make_parent_universe(dir: &Path) -> PathBuf {
        let cdir = dir.join(".constellation");
        fs::create_dir_all(&cdir).unwrap();
        dir.to_path_buf()
    }

    #[test]
    fn backup_path_for_appends_suffix() {
        // Compare on file_name only — full-path comparison is fragile
        // across Windows (mixed separators after set_file_name) vs Unix.
        let p = Path::new("E:/X/Y/.constellation/search.db");
        let b = backup_path_for(p);
        assert_eq!(
            b.file_name().unwrap().to_string_lossy(),
            "search.db.pre-mig-056.bak"
        );
        assert_eq!(b.parent(), p.parent());
    }

    #[test]
    fn is_cuniverse_open_elsewhere_false_for_unlocked_db() {
        let tmp = TempDir::new().unwrap();
        let db = tmp.path().join("test.db");
        // Create + close the DB
        {
            let _ = Connection::open(&db).unwrap();
        }
        assert!(!is_cuniverse_open_elsewhere(&db));
    }

    #[test]
    fn is_cuniverse_open_elsewhere_true_for_locked_db() {
        let tmp = TempDir::new().unwrap();
        let db = tmp.path().join("test.db");
        // Open the DB and start an exclusive transaction. The lock
        // check should see we can't acquire BEGIN EXCLUSIVE.
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch("BEGIN EXCLUSIVE;").unwrap();

        assert!(is_cuniverse_open_elsewhere(&db));

        // Clean up the held lock
        conn.execute_batch("ROLLBACK;").unwrap();
    }

    #[test]
    fn audit_log_creates_dir_and_appends_line() {
        let tmp = TempDir::new().unwrap();
        let parent = make_parent_universe(tmp.path());

        write_audit_log(&parent, Path::new("/some/cu/search.db"), "AUTO_MIGRATE", "OK").unwrap();
        // Second write to verify append (not overwrite)
        write_audit_log(&parent, Path::new("/other/cu/search.db"), "AUTO_MIGRATE_FAILED", "X").unwrap();

        let log_path = parent.join(".constellation").join("federation-audit.log");
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("AUTO_MIGRATE\tcuniverse=/some/cu/search.db\tresult=OK"));
        assert!(content.contains("AUTO_MIGRATE_FAILED\tcuniverse=/other/cu/search.db\tresult=X"));
        // Two entries total
        assert_eq!(content.lines().count(), 2);
    }

    #[test]
    fn backup_failure_bails_before_touching_source() {
        // Use a cuniverse path with a non-existent source — fs::copy
        // will fail at the backup step.
        let tmp = TempDir::new().unwrap();
        let parent = make_parent_universe(tmp.path());
        let nonexistent_db = tmp.path().join("does_not_exist").join("search.db");

        let result = run_migrations_on(&nonexistent_db, &parent);
        assert!(result.is_err());
        match result.unwrap_err() {
            MigrationError::BackupFailed(_) => {}
            other => panic!("expected BackupFailed, got {:?}", other),
        }

        // Source still doesn't exist (we didn't accidentally create it)
        assert!(!nonexistent_db.exists());
    }

    #[test]
    fn drift_too_severe_falls_back_to_backup_restore() {
        // Pathological "drift" — a note_meta with only PRIMARY KEY column
        // and no others. This isn't a realistic real-world drift (real
        // drift is between adjacent schema versions where all earlier
        // columns are present); it represents the worst-case corruption.
        //
        // init_db's later migration steps (MIG-022 etc.) reference
        // columns that the pathological schema doesn't have, so init_db
        // returns Err. run_migrations_on must:
        //   (a) Detect the failure
        //   (b) Restore the source from the backup
        //   (c) Surface MigrationError::MigrationFailed
        //   (d) Write a FAILED audit log entry
        //
        // This locks the safeguard 3 (atomic-via-backup-restore) path.
        let parent_tmp = TempDir::new().unwrap();
        let cu_tmp = TempDir::new().unwrap();
        let parent = make_parent_universe(parent_tmp.path());
        let cu_db = make_drifted_cuniverse(cu_tmp.path());

        // Capture pre-migration source bytes for restore verification.
        let pre_bytes = fs::read(&cu_db).unwrap();

        let result = run_migrations_on(&cu_db, &parent);
        assert!(result.is_err(), "expected failure on pathological drift");
        match result.unwrap_err() {
            MigrationError::MigrationFailed(_) => {}
            other => panic!("expected MigrationFailed, got {:?}", other),
        }

        // Source restored from backup — bytes match pre-migration state.
        let post_bytes = fs::read(&cu_db).unwrap();
        assert_eq!(
            pre_bytes, post_bytes,
            "source must be byte-identical to pre-migration backup after restore"
        );

        // Failure audit log entry written
        let log_path = parent.join(".constellation").join("federation-audit.log");
        assert!(log_path.exists());
        let log_content = fs::read_to_string(&log_path).unwrap();
        assert!(
            log_content.contains("AUTO_MIGRATE_FAILED_RESTORED"),
            "expected FAILED_RESTORED audit entry; got: {}",
            log_content
        );
    }

    #[test]
    fn migration_on_completely_empty_db_runs_full_init() {
        // Truly empty: a 0-byte file pretending to be search.db.
        // init_db should handle this (SQLite open on 0-byte file
        // creates a new empty DB).
        let parent_tmp = TempDir::new().unwrap();
        let cu_tmp = TempDir::new().unwrap();
        let parent = make_parent_universe(parent_tmp.path());
        let cdir = cu_tmp.path().join(".constellation");
        fs::create_dir_all(&cdir).unwrap();
        let cu_db = cdir.join("search.db");
        // Create an empty file
        fs::write(&cu_db, "").unwrap();

        let result = run_migrations_on(&cu_db, &parent);
        assert!(
            result.is_ok(),
            "expected migration on empty DB to succeed; got: {:?}",
            result
        );

        // Confirm note_meta now exists
        let conn = Connection::open(&cu_db).unwrap();
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='note_meta'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1);
    }
}
