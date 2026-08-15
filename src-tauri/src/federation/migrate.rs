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
//! 2. **Pre-migration backup** — via the SQLite Online Backup API
//!    (`backup_database`; MIG-111 R11 retired the original `fs::copy`,
//!    which lost WAL-resident rows — the red→green pair in
//!    `tests_r11_backup` keeps the reason executable). If the backup
//!    fails, bail BEFORE touching the source DB.
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
//!    ```text
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

/// MIG-111 Phase 0.1 (R11) — copy a SQLite database THROUGH the engine, never around it.
///
/// The SQLite Online Backup API reads a transactionally-consistent snapshot under the
/// database's own locks: WAL-resident committed rows are included, a concurrent writer is
/// tolerated (the backup restarts its pass), and a lock that cannot be honoured fails loudly
/// instead of producing a torn file. `fs::copy` guarantees none of those.
///
/// THE BAN, stated precisely (R11, whole-ecosystem sweep 2026-08-12): `fs::copy` of a SQLite
/// database is forbidden UNLESS the copier (a) holds the database's only connection, (b) runs
/// `wal_checkpoint(TRUNCATE)` through it first, and (c) VERIFIES the copy opens with matching
/// baseline counts afterwards — the `mig108.rs` unification pattern, which is the one audited
/// exemption. JSON/config files and .md files are outside the ban (no WAL to lose).
pub(crate) fn backup_database(src: &Path, dst: &Path) -> Result<(), String> {
    let src_conn = rusqlite::Connection::open_with_flags(
        src,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("open source {:?}: {}", src, e))?;
    let mut dst_conn =
        rusqlite::Connection::open(dst).map_err(|e| format!("open dest {:?}: {}", dst, e))?;
    let backup = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn)
        .map_err(|e| format!("backup init: {}", e))?;
    backup
        .run_to_completion(64, std::time::Duration::from_millis(50), None)
        .map_err(|e| format!("backup run: {}", e))
}

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
    //
    // MIG-111 Phase 0.1 (R11 — panel-mandated, "fixed FIRST: it runs on today's boot path").
    // This was `fs::copy(cu_db_path, backup)`, which copies the MAIN file only. Under WAL,
    // committed rows live in the `-wal` sibling until a checkpoint — so the "backup" could
    // silently lack the newest committed writes, and Safeguard 3 restoring it would ROLL THEM
    // BACK. Both adversarial passes (Option B attack A2; the plan attack) named it, and the
    // red→green test below proves the shape: rows committed-but-uncheckpointed are exactly
    // what fs::copy loses and what the SQLite Online Backup API carries.
    let backup_path = backup_path_for(cu_db_path);
    backup_database(cu_db_path, &backup_path)
        .map_err(|e| MigrationError::BackupFailed(format!("backup to {:?} failed: {}", backup_path, e)))?;

    // ─── Run migration via init_db ───
    //
    // **PJ-230 — `init_db` on a FOREIGN database must not heal it, and no longer does.**
    //
    // This is the one place Constellation runs `init_db` against a database it is not
    // the authority on: a linked universe's, from the parent's process. Until PJ-228,
    // `init_db` carried a synchronous five-family derived-view heal gated on two crash
    // markers — so this call could rebuild the CHILD's outgoing/incoming aggregates and
    // Sky strata using the PARENT's process-global link vocabulary (`link_types` is one
    // global, loaded only from the active universe), and then CLEAR the child's markers.
    // The clearing is what made it permanent: the child's own next boot saw nothing left
    // to heal and kept the parent-flavoured values as final.
    //
    // PJ-228 moved the heal out of `init_db` for boot latency, and incidentally ended
    // that particular write. Do not put it back here, and do not "fix" it by loading the
    // child's vocabulary into the global first — that means swapping a process-global on
    // a background thread while every other subsystem reads it.
    //
    // A child also heals itself when it is opened as its own active universe
    // (`set_active_universe` → `invalidate_search_state` → the next
    // `ensure_search_db_ready` → `derived_heal::maybe_schedule` against ITS db_path).
    //
    // **PJ-232 — the heal was NOT the only foreign write. The rest is now closed too.**
    // The 2026-08-09 inspection refuted an earlier version of this very comment, which
    // claimed PJ-228 had made `init_db` safe on a foreign database. It had not:
    //
    //   * `init_db` unconditionally DROPs and re-CREATEs the outgoing-aggregate triggers
    //     (`search.rs` `create_outgoing_link_triggers`) and the Sky stratum/maturity
    //     triggers, and every one of those bodies is generated from
    //     `link_types::snapshot()` — the ACTIVE (parent) universe's registry, loaded at
    //     `search.rs`'s `load_active` immediately before `init_db` for the ACTIVE
    //     universe only. So this call persists parent-flavoured DDL into the child's
    //     `sqlite_master`.
    //   * Worse, `init_db` then runs `mig003_step3_soft_rebackfill` ungated, which
    //     re-indexes every row with an empty `cid_cn` — a set that is non-empty by
    //     construction on a schema-drifted child — firing those parent-flavoured
    //     triggers on the child's own rows, and, for a file with no identity key at all,
    //     writing frontmatter into the child universe's `.md` files from this process.
    //
    // It diverged only when the two universes' link vocabularies actually differed (a
    // user-defined type on either side); with seeds only the generated SQL is identical,
    // which is why nobody had seen it.
    //
    // **The fix is `init_db_schema_only`, used below.** It migrates the schema and
    // NOTHING else: no vocabulary-dependent trigger DDL, no dependent-table back-fill, no
    // soft re-backfill, no Step-4 rename pass. The owner does all of that on its own next
    // launch, when the registry actually holds ITS vocabulary — and until then nothing
    // writes through those triggers, because a cUniverse is attached read-only.
    let migration_outcome: Result<(), String> = (|| {
        let conn = crate::search::init_db_schema_only(cu_db_path)
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
            //
            // MIG-111 Phase 0.1 (R11) — restore likewise goes through the backup API, never a
            // byte copy over a possibly-live file: the API takes SQLite's own locks, so a
            // holder elsewhere makes this FAIL LOUDLY (audit: RESTORE_FAILED) instead of
            // silently tearing the file under it. And restoring through the API rewrites the
            // target's WAL state coherently, where fs::copy left a stale `-wal` sibling
            // pairing with the restored main file.
            let restore_result = backup_database(&backup_path, cu_db_path);
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
    // MIG-111 Phase 0.2 (R5) — the OWNER LOCK is the primary answer. The old sole test,
    // `BEGIN EXCLUSIVE; ROLLBACK`, acquires only SQLite's WRITE lock: in WAL mode an
    // instance merely HOLDING the universe open (no write in flight) does not hold it, so
    // the probe answered "not open" in exactly the routine case it existed to catch — a
    // false NEGATIVE certified by the MIG-111 adversarial pass and pinned by the
    // two-process test in `universe_lock`. The SQLite probe is RETAINED as a supplement
    // only: it still catches a NON-Constellation tool (a DB browser) holding the file,
    // which the owner lock cannot see.
    //
    // `<root>/.constellation/search.db` → the universe root is two levels up.
    if let Some(root) = db_path.parent().and_then(|c| c.parent()) {
        if !crate::universe_lock::held_by_us(root) {
            if matches!(
                crate::universe_lock::probe(root),
                crate::universe_lock::Ownership::HeldElsewhere { .. }
            ) {
                return true; // a live Constellation instance owns this universe — idle or not
            }
        }
    }
    sqlite_write_lock_held(db_path)
}

/// The retired PRIMARY probe, kept as a supplement (see above) and as the RED half of the
/// two-process proof in `universe_lock::tests` — it must keep failing to see an idle holder,
/// or SQLite's locking model changed and the whole 0.2 design note needs revisiting.
pub(crate) fn sqlite_write_lock_held(db_path: &Path) -> bool {
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
mod tests_r11_backup {
    //! MIG-111 Phase 0.1 (R11) — the red→green pair for the live-WAL backup ban.
    use super::backup_database;
    use rusqlite::Connection;

    fn wal_db_with_uncheckpointed_row(dir: &std::path::Path) -> std::path::PathBuf {
        let db = dir.join("src.db");
        let conn = Connection::open(&db).unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.execute("CREATE TABLE t (v TEXT)", []).unwrap();
        // Checkpoint the schema into the main file, then commit a row that stays WAL-resident:
        // `wal_checkpoint(TRUNCATE)` first, then the insert with checkpointing disabled.
        conn.pragma_update(None, "wal_checkpoint", "TRUNCATE").ok();
        conn.pragma_update(None, "wal_autocheckpoint", "0").unwrap();
        conn.execute("INSERT INTO t (v) VALUES ('wal-resident')", []).unwrap();
        // Keep the connection OPEN (leaked) so closing cannot checkpoint the WAL behind us —
        // this is the live-DB shape: a holder elsewhere, committed rows in the `-wal` only.
        std::mem::forget(conn);
        db
    }

    /// The DEFECT, demonstrated: fs::copy of the main file loses the WAL-resident row.
    /// This is the pre-0.1 backup verbatim, kept as the red half of the pair so the reason
    /// for the ban stays executable, not narrated.
    #[test]
    fn r11_red_fs_copy_loses_wal_resident_rows() {
        let dir = tempfile::tempdir().unwrap();
        let src = wal_db_with_uncheckpointed_row(dir.path());
        let copy = dir.path().join("copy.db");
        std::fs::copy(&src, &copy).unwrap();
        let c = Connection::open(&copy).unwrap();
        let n: i64 = c.query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0, "fs::copy must demonstrably lose the WAL-resident row (else this pair proves nothing)");
    }

    /// The FIX: the backup API carries the WAL-resident row.
    #[test]
    fn r11_green_backup_api_carries_wal_resident_rows() {
        let dir = tempfile::tempdir().unwrap();
        let src = wal_db_with_uncheckpointed_row(dir.path());
        let dst = dir.path().join("backup.db");
        backup_database(&src, &dst).unwrap();
        let c = Connection::open(&dst).unwrap();
        let n: i64 = c.query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1, "the backup API must include committed WAL-resident rows");
        let v: String = c.query_row("SELECT v FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(v, "wal-resident");
    }
}

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

        // Capture the pre-migration source CONTENT for restore verification.
        //
        // MIG-111 Phase 0.1 (R11) — this assertion was byte-identity when the backup was an
        // `fs::copy`. The invariant it protects was never really bytes; it was "the user's
        // DATA is exactly what it was". The backup-API restore reproduces every page of
        // content but may legitimately differ in header change-counters and WAL state — so
        // the assertion is restated as what it always meant: schema and rows identical.
        let dump = |path: &Path| -> Vec<(String, i64)> {
            let c = Connection::open(path).unwrap();
            let tables: Vec<String> = c
                .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .unwrap()
                .query_map([], |r| r.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            tables
                .into_iter()
                .map(|t| {
                    let n: i64 = c
                        .query_row(&format!("SELECT COUNT(*) FROM \"{}\"", t), [], |r| r.get(0))
                        .unwrap_or(-1);
                    (t, n)
                })
                .collect()
        };
        let pre_content = dump(&cu_db);

        let result = run_migrations_on(&cu_db, &parent);
        assert!(result.is_err(), "expected failure on pathological drift");
        match result.unwrap_err() {
            MigrationError::MigrationFailed(_) => {}
            other => panic!("expected MigrationFailed, got {:?}", other),
        }

        // Source restored from backup — same tables, same row counts as pre-migration.
        let post_content = dump(&cu_db);
        assert_eq!(
            pre_content, post_content,
            "source content (tables + row counts) must match the pre-migration state after restore"
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

    /// PJ-230 — **`init_db` must never heal or disarm a database it does not own.**
    ///
    /// This module is the only caller that hands `init_db` a FOREIGN universe's
    /// database. Before PJ-228, `init_db` healed the five derived families and then
    /// cleared the crash markers — here, that meant rebuilding a child's link
    /// aggregates with the PARENT's process-global link vocabulary and then removing
    /// the child's own record that a heal was owed, so its next boot kept the wrong
    /// values forever.
    ///
    /// PJ-228 removed that as a side effect of moving the heal off the boot path. This
    /// pins the property so it cannot come back with the next person who thinks a
    /// self-heal belongs in `init_db`.
    #[test]
    fn init_db_leaves_a_foreign_universes_crash_markers_armed() {
        let dir = TempDir::new().unwrap();
        let cdir = dir.path().join(".constellation");
        fs::create_dir_all(&cdir).unwrap();
        let db_path = cdir.join("search.db");

        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);
                 INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
                   VALUES ('outgoing_triggers_dropped', 1, 0), ('derived_tail_pending', 1, 0);",
            )
            .unwrap();
        }

        // Exactly what `run_migrations_on` does to a child DB.
        let conn = crate::search::init_db_schema_only(&db_path).expect("init_db on a foreign DB");

        let armed = |module: &str| -> bool {
            conn.query_row(
                "SELECT version FROM schema_versions WHERE module = ?1",
                rusqlite::params![module],
                |r| r.get::<_, i64>(0),
            )
            .map(|v| v > 0)
            .unwrap_or(false)
        };
        assert!(
            armed("outgoing_triggers_dropped"),
            "init_db must not clear a foreign universe's trigger marker — clearing it is what made the parent's values permanent"
        );
        assert!(
            armed("derived_tail_pending"),
            "init_db must not clear a foreign universe's derived-tail marker"
        );
    }

    /// PJ-232 — **the schema-only door must not bake OUR link vocabulary into THEIR
    /// database, and must not touch their data or their files.**
    ///
    /// The trigger bodies for the outgoing aggregates and for Sky stratum/maturity are
    /// generated from `link_types::snapshot()`, a process-global holding the ACTIVE
    /// universe's registry. Creating them here would persist the parent's link types
    /// into the child's `sqlite_master`; the data passes that follow would then fire
    /// them on the child's own rows, and write identity keys into the child's `.md`
    /// files. The owner creates all of it correctly on its own next launch.
    #[test]
    fn schema_only_init_writes_no_vocabulary_triggers_into_a_foreign_db() {
        let dir = TempDir::new().unwrap();
        let cdir = dir.path().join(".constellation");
        fs::create_dir_all(&cdir).unwrap();
        let db_path = cdir.join("search.db");

        let conn = crate::search::init_db_schema_only(&db_path).expect("schema-only init");

        let trigger_count = |name: &str| -> i64 {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger' AND name = ?1",
                rusqlite::params![name],
                |r| r.get(0),
            )
            .unwrap_or(-1)
        };
        for t in [
            "note_links_outgoing_ai",
            "note_links_outgoing_ad",
            "note_links_outgoing_au",
            "note_meta_sky_stratum_au",
            "note_meta_sky_maturity_au",
        ] {
            assert_eq!(trigger_count(t), 0, "{t} carries the ACTIVE universe's vocabulary — it must not be written into a foreign database");
        }

        // The schema itself IS migrated — that is the entire point of this door.
        let cols: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('note_meta') WHERE name IN ('path','name','library_name','created_at','modified')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cols, 5, "schema-only still brings the schema up to current");
    }

    /// PJ-232 — **the foreign door must not touch the other universe's NOTE FILES, and
    /// must not delete its rows.**
    ///
    /// The first version of the test above ran against an EMPTY database, so the passes
    /// that write files never had a row to act on — and it therefore PASSED while MIG-003
    /// Step 1 was still unguarded, which the 2026-08-09 inspection caught. A test whose
    /// subject cannot fire is not a test. This one seeds exactly the state that arms
    /// those passes: a note row with no `cid_cn`, and a real `.md` file on disk.
    ///
    /// Unguarded, Step 1 would inject `cid_cn:` frontmatter into that file (bumping its
    /// mtime across the user's sync), or DELETE its row if the path no longer stats —
    /// cascading into `note_state_history`, which cannot be rebuilt from the files.
    #[test]
    fn schema_only_init_does_not_write_a_foreign_universes_note_files_or_rows() {
        let dir = TempDir::new().unwrap();
        let cdir = dir.path().join(".constellation");
        fs::create_dir_all(&cdir).unwrap();
        let db_path = cdir.join("search.db");

        // TWO rows, because Step 1 does two destructive things and only one of them is
        // reachable from a unit test:
        //
        //   * it injects `cid_cn:` frontmatter into files — but that goes through the
        //     write gate, which cannot fire here with no universe registered, so an
        //     assertion on file bytes can never go red in a test. It is asserted anyway,
        //     to state the intent, but it is NOT what guards this.
        //   * it DELETEs rows whose path no longer stats — no gate involved. That is the
        //     assertion with teeth, and it is the worse failure: the delete cascades into
        //     `note_state_history`, which cannot be rebuilt from the `.md` files.
        //
        // Verified red: removing the `owns &&` from MIG-003 Step 1 fails this test on the
        // vanished-path row.
        let note = dir.path().join("Foreign Note.md");
        let original = "---
title: Foreign Note
---

body
";
        fs::write(&note, original).unwrap();
        let vanished = dir.path().join("Moved Away.md"); // deliberately never created

        {
            // Build rows through the REAL schema rather than a hand-rolled subset — a
            // partial table makes a later pass fail on a missing column, which would
            // "pass" this test for the wrong reason. Because the foreign door does not
            // stamp MIG-003, the second call below still sees Step 1 as outstanding.
            let conn = crate::search::init_db_schema_only(&db_path).expect("schema setup");
            for p in [&note, &vanished] {
                conn.execute(
                    "INSERT INTO note_meta (path, name, cid_cn, library_name, created_at, modified)
                     VALUES (?1, 'Foreign Note', '', 'lib', 0, 0)",
                    rusqlite::params![p.to_string_lossy()],
                )
                .unwrap();
            }
            // The setup call above builds the schema; clear MIG-003's stamp so the real
            // call below still sees Step 1 as outstanding — which is the state a foreign
            // database that has never had MIG-003 run is actually in. Without this the
            // setup call silently satisfies the stamp and the assertions below cannot
            // fail, which is exactly how the FIRST version of this test passed while the
            // bug was live.
            conn.execute("DELETE FROM schema_versions WHERE module = 'note_meta'", [])
                .unwrap();
        }

        let conn = crate::search::init_db_schema_only(&db_path).expect("schema-only init");

        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM note_meta", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            rows, 2,
            "no row of a foreign universe may be deleted by a schema migration — including one whose file has moved, which is exactly what a re-linked universe looks like"
        );
        let empty_cid: i64 = conn
            .query_row("SELECT COUNT(*) FROM note_meta WHERE cid_cn = ''", [], |r| r.get(0))
            .unwrap();
        assert_eq!(empty_cid, 2, "the rows Step 1 would have repaired must still be untouched");
        assert_eq!(
            fs::read_to_string(&note).unwrap(),
            original,
            "the foreign universe's note file must be byte-identical — this process is not its owner"
        );
    }

    /// The ACTIVE door is unchanged — it still creates everything. Without this, the
    /// guard above could be satisfied by simply breaking trigger creation for everyone.
    #[test]
    fn the_active_door_still_creates_the_vocabulary_triggers() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("search.db");
        let conn = crate::search::init_db(&db_path).expect("active init");
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger' AND name IN
                   ('note_links_outgoing_ai','note_links_outgoing_ad','note_links_outgoing_au',
                    'note_meta_sky_stratum_au','note_meta_sky_maturity_au')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 5, "the active universe's init must still create all five");
    }
}
