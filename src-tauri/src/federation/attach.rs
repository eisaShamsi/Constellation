//! MIG-056 §B — `attach_all` + per-cUniverse safety wrapper.
//!
//! Boot-time logic that ATTACHes each cUniverse's `search.db` to the
//! active universe's connection as a read-only schema. Failures
//! become `FederationWarning`s in the `FederationContext` per the
//! skip_unavailable model (Architect §5.2 / Boss-locked).
//!
//! ## Safety wrapper (`attach_with_safety`)
//!
//! For each cUniverse:
//! 1. Resolve the cUniverse's `search.db` path
//! 2. If missing → warn + skip
//! 3. ATTACH read-only via URI mode `?mode=ro`
//! 4. Verify `note_meta` table exists with the columns the federation
//!    queries need (path, name, library_name, created_at, modified)
//!    via `PRAGMA {schema}.table_info(note_meta)`
//! 5. If schema-incomplete → DETACH + warn + skip
//!    (in §C, this path becomes "auto-migrate to floor" per §5.3)
//! 6. Tune `PRAGMA {schema}.cache_size = -512` (≈512 KB) per
//!    Architect §7.1 — avoids per-attachment cache bloat
//! 7. Add to `FederationContext.attached`
//!
//! ## ATTACH cap
//!
//! Per Architect §5.4 (Boss-locked): ATTACH cap is 25. cUniverses
//! beyond 25 are warned + skipped. (The SQLite compile-time
//! `SQLITE_MAX_ATTACHED` bump from 10 → 25 lands in §L's PCS, since
//! it requires rebuilding the bundled SQLite. Until then, the
//! runtime cap is whatever the bundled SQLite was compiled with —
//! the v1 enforcement here is a soft cap for design correctness.)
//!
//! ## Lifecycle integration
//!
//! Called from `ensure_search_db_ready` AFTER `init_db` + the §E
//! `init_five_acts_system_notes` hook. Wrapped in a background
//! thread so the boot path completes before background-attach
//! begins (Architect §3.3 — no boot-perf regression).

use super::failure::FederationError;
use super::FederationContext;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Boss-locked v1 ATTACH cap (Architect §5.4).
const ATTACH_CAP_V1: usize = 25;

/// Schema columns the federation queries depend on. Missing any of
/// these → skip the cUniverse with a "schema_incomplete" warning.
const REQUIRED_NOTE_META_COLUMNS: &[&str] = &[
    "path",
    "name",
    "library_name",
    "created_at",
    "modified",
];

/// Build the schema alias for the i-th attached cUniverse. Format:
/// `cu0`, `cu1`, …, `cu24`. Alphanumeric only — safe to interpolate
/// into SQL identifier positions.
fn schema_alias(i: usize) -> String {
    format!("cu{}", i)
}

/// Resolve the federated cUniverse roots from the active universe's
/// federation manifest, de-duplicated by physical path. Order
/// follows `resolve_universe_libraries`'s walk order (own libraries
/// first, then cUniverses in declared order, recursive).
///
/// The active universe's OWN libraries are NOT included — they
/// belong to the `main` schema, not to an attached cUniverse.
pub(crate) fn unique_cuniverse_roots(
    libs: &[crate::libraries::LibraryInfo],
    active_universe_root: &Path,
) -> Vec<PathBuf> {
    let active_canon = active_universe_root.to_string_lossy().to_lowercase();
    let mut seen: Vec<String> = Vec::new();
    let mut out: Vec<PathBuf> = Vec::new();

    for lib in libs {
        // The cUniverse root is the directory containing the library —
        // for our purposes, the cUniverse root is whatever directory
        // contains a `.constellation/search.db` file ancestral to the
        // library path. We approximate by walking parents until we
        // find a `.constellation/universe.json` OR run out — that's
        // the cUniverse's root.
        let lib_path = Path::new(&lib.path);
        if let Some(cu_root) = find_universe_root(lib_path) {
            let key = cu_root.to_string_lossy().to_lowercase();
            // Exclude the active universe (it's `main`, not a cUniverse).
            if key == active_canon {
                continue;
            }
            if !seen.contains(&key) {
                seen.push(key);
                out.push(cu_root);
            }
        }
    }
    out
}

/// Walk parent directories until we find one containing
/// `.constellation/universe.json` — that's the universe root.
/// Returns `None` if no such ancestor is found.
fn find_universe_root(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_dir() {
        Some(start.to_path_buf())
    } else {
        start.parent().map(|p| p.to_path_buf())
    };
    while let Some(dir) = current {
        if dir.join(".constellation").join("universe.json").exists() {
            return Some(dir);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

/// Public entrypoint. Builds a `FederationContext` by ATTACHing every
/// cUniverse's `search.db` to `conn`. Per Architect §6.2.
///
/// Called from `ensure_search_db_ready` (search.rs) inside a
/// background thread AFTER the main `init_db` returns.
pub fn attach_all(
    conn: &mut Connection,
    app: &tauri::AppHandle,
) -> Result<FederationContext, FederationError> {
    let active_universe_root = crate::universe::active_universe_dir(app)
        .map_err(FederationError::ResolveFailed)?;

    let libs = crate::universe::resolve_universe_libraries(app.clone())
        .map_err(FederationError::ResolveFailed)?;

    let cu_roots = unique_cuniverse_roots(&libs, &active_universe_root);

    let mut ctx = FederationContext::new();

    for (i, cu_root) in cu_roots.iter().enumerate() {
        if i >= ATTACH_CAP_V1 {
            ctx.warn(
                cu_root.clone(),
                format!(
                    "ATTACH cap reached ({} cUniverses; v1 limit is {}). Federation skipped for this and any subsequent cUniverses.",
                    cu_roots.len(),
                    ATTACH_CAP_V1
                ),
            );
            // Don't break — keep emitting warnings for the rest so
            // the user sees every skipped cUniverse, not just the
            // first over-cap one.
            continue;
        }

        let alias = schema_alias(i);
        let cu_db_path = cu_root.join(".constellation").join("search.db");

        if !cu_db_path.exists() {
            ctx.warn(cu_root.clone(), "search.db missing");
            continue;
        }

        match attach_with_safety(conn, &cu_db_path, &alias) {
            Ok(()) => {
                ctx.add_attached(alias, cu_root.clone());
            }
            Err(reason) if reason.starts_with("schema_incomplete") => {
                // MIG-056 §C — auto-migrate path (Architect §5.3).
                // Try to bring the cUniverse's schema up to current
                // and retry the attach. Failures during migrate
                // become warnings (skip_unavailable model).
                match super::migrate::run_migrations_on(&cu_db_path, &active_universe_root) {
                    Ok(()) => {
                        // Retry attach after successful migration.
                        match attach_with_safety(conn, &cu_db_path, &alias) {
                            Ok(()) => {
                                ctx.add_attached(alias, cu_root.clone());
                            }
                            Err(re_reason) => {
                                ctx.warn(
                                    cu_root.clone(),
                                    format!(
                                        "auto-migration succeeded but post-migrate attach still failed: {}",
                                        re_reason
                                    ),
                                );
                            }
                        }
                    }
                    Err(mig_err) => {
                        ctx.warn(
                            cu_root.clone(),
                            format!("auto-migration declined: {}", mig_err),
                        );
                    }
                }
            }
            Err(other) => {
                ctx.warn(cu_root.clone(), other);
            }
        }
    }

    ctx.set_ready(true);
    Ok(ctx)
}

/// ATTACH a single cUniverse's `search.db` read-only + verify the
/// schema is federation-ready. Returns `Err(String)` with a
/// human-readable reason on any failure; the caller turns that into
/// a `FederationWarning` (skip_unavailable model).
fn attach_with_safety(
    conn: &mut Connection,
    db_path: &Path,
    alias: &str,
) -> Result<(), String> {
    // Step 1: ATTACH read-only.
    // URI mode=ro opens read-only; immutable=0 keeps file-watcher
    // semantics intact (the cUniverse's owner can modify it; we
    // detect changes on next attach cycle).
    //
    // The path's separator-sensitivity matters on Windows. SQLite
    // URI form accepts forward slashes; we normalize via to_string_lossy
    // then replace backslashes.
    let path_uri = db_path
        .to_string_lossy()
        .replace('\\', "/");
    let attach_sql = format!(
        "ATTACH DATABASE 'file:{}?mode=ro' AS {}",
        path_uri, alias
    );
    conn.execute(&attach_sql, [])
        .map_err(|e| format!("ATTACH failed: {}", e))?;

    // Step 2: verify required schema columns exist.
    if let Err(reason) = verify_schema(conn, alias) {
        // Detach to keep the connection clean. If DETACH fails, log
        // but propagate the original schema error (which is the more
        // useful diagnostic).
        let _ = conn.execute(&format!("DETACH DATABASE {}", alias), []);
        return Err(reason);
    }

    // Step 3: tune cache_size per Architect §7.1.
    // -512 = 512 KB target (negative values are KiB; positive are pages).
    if let Err(e) = conn.execute(
        &format!("PRAGMA {}.cache_size = -512", alias),
        [],
    ) {
        // Cache tuning failure isn't fatal — log and continue.
        // The cUniverse stays attached at SQLite's default cache.
        eprintln!(
            "[federation] cache_size tune failed for {} ({}): {}",
            alias,
            db_path.display(),
            e
        );
    }

    Ok(())
}

/// Confirm the attached schema has the required `note_meta` columns.
/// On any missing/extra-state, return a human-readable diagnostic
/// the caller surfaces as a `FederationWarning`.
fn verify_schema(conn: &Connection, alias: &str) -> Result<(), String> {
    // PRAGMA `{schema}.table_info('note_meta')` returns one row per column.
    let sql = format!("PRAGMA {}.table_info(note_meta)", alias);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("table_info prepare failed: {}", e))?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("table_info query failed: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    if columns.is_empty() {
        return Err(
            "schema_incomplete: note_meta table is missing or empty (search.db may be uninitialized or corrupt)"
                .to_string(),
        );
    }

    for required in REQUIRED_NOTE_META_COLUMNS {
        if !columns.iter().any(|c| c == required) {
            return Err(format!(
                "schema_incomplete: note_meta is missing required column `{}` (cUniverse may need to be opened in Constellation as the active universe to upgrade its schema)",
                required
            ));
        }
    }

    Ok(())
}

// ─── §B tests ───
// These extend the §A scaffold. Cover the happy path + the 3
// failure-mode paths per Plan §B verification clause.

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_cuniverse_with_search_db(dir: &Path) -> PathBuf {
        // Create the universe directory + .constellation/ + universe.json
        // + a working search.db with a note_meta table that has all the
        // required columns.
        let cdir = dir.join(".constellation");
        std::fs::create_dir_all(&cdir).unwrap();
        std::fs::write(
            cdir.join("universe.json"),
            r#"{"name":"Test","created":"2026-01-01T00:00:00Z","version":2,"children":[],"notes_folder":null}"#,
        )
        .unwrap();
        let db_path = cdir.join("search.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                library_name TEXT NOT NULL,
                modified INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );",
        )
        .unwrap();
        db_path
    }

    fn make_test_cuniverse_with_incomplete_schema(dir: &Path) -> PathBuf {
        let cdir = dir.join(".constellation");
        std::fs::create_dir_all(&cdir).unwrap();
        std::fs::write(
            cdir.join("universe.json"),
            r#"{"name":"Incomplete","created":"2026-01-01T00:00:00Z","version":2,"children":[],"notes_folder":null}"#,
        )
        .unwrap();
        let db_path = cdir.join("search.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                name TEXT NOT NULL
                -- missing library_name, created_at, modified
            );",
        )
        .unwrap();
        db_path
    }

    #[test]
    fn schema_alias_is_alphanumeric_safe() {
        assert_eq!(schema_alias(0), "cu0");
        assert_eq!(schema_alias(24), "cu24");
        // Verify no special chars that could break SQL identifier escaping.
        for i in 0..25 {
            let a = schema_alias(i);
            for ch in a.chars() {
                assert!(
                    ch.is_ascii_alphanumeric(),
                    "alias `{}` contains non-alphanumeric char `{}`",
                    a,
                    ch
                );
            }
        }
    }

    #[test]
    fn attach_with_safety_succeeds_on_healthy_cuniverse() {
        let tmp = TempDir::new().unwrap();
        let cu_db = make_test_cuniverse_with_search_db(tmp.path());

        let mut conn = Connection::open_in_memory().unwrap();
        let result = attach_with_safety(&mut conn, &cu_db, "cu_test");
        assert!(result.is_ok(), "attach_with_safety should succeed: {:?}", result);

        // Confirm we can query the attached schema.
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM cu_test.note_meta", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn attach_with_safety_fails_on_missing_required_columns() {
        let tmp = TempDir::new().unwrap();
        let cu_db = make_test_cuniverse_with_incomplete_schema(tmp.path());

        let mut conn = Connection::open_in_memory().unwrap();
        let result = attach_with_safety(&mut conn, &cu_db, "cu_test");
        assert!(result.is_err(), "should detect incomplete schema");
        let reason = result.unwrap_err();
        assert!(
            reason.contains("schema_incomplete"),
            "reason should mention schema_incomplete; got: {}",
            reason
        );

        // Confirm DETACH cleaned up — the alias should no longer be queryable.
        let probe = conn.query_row(
            "SELECT COUNT(*) FROM cu_test.note_meta",
            [],
            |r| r.get::<_, i64>(0),
        );
        assert!(probe.is_err(), "alias should be detached after schema check failure");
    }

    #[test]
    fn unique_cuniverse_roots_excludes_active_and_dedupes() {
        let tmp = TempDir::new().unwrap();
        let active = tmp.path().join("ActiveUniverse");
        let cu1 = tmp.path().join("CUniverse1");
        std::fs::create_dir_all(&active).unwrap();
        std::fs::create_dir_all(&cu1).unwrap();
        std::fs::create_dir_all(active.join(".constellation")).unwrap();
        std::fs::create_dir_all(cu1.join(".constellation")).unwrap();
        std::fs::write(
            active.join(".constellation").join("universe.json"),
            "{}",
        )
        .unwrap();
        std::fs::write(
            cu1.join(".constellation").join("universe.json"),
            "{}",
        )
        .unwrap();

        let libs = vec![
            // Active universe library (should be excluded)
            crate::libraries::LibraryInfo {
                id: "1".into(),
                name: "active_lib".into(),
                path: active.to_string_lossy().into_owned(),
                is_universe_notes: true,
                canonical_mode: "native".into(),
            },
            // cUniverse library 1
            crate::libraries::LibraryInfo {
                id: "2".into(),
                name: "cu1_lib".into(),
                path: cu1.to_string_lossy().into_owned(),
                is_universe_notes: true,
                canonical_mode: "native".into(),
            },
            // Duplicate of cu1 (should be deduped)
            crate::libraries::LibraryInfo {
                id: "3".into(),
                name: "cu1_lib_dup".into(),
                path: cu1.to_string_lossy().into_owned(),
                is_universe_notes: false,
                canonical_mode: "native".into(),
            },
        ];

        let roots = unique_cuniverse_roots(&libs, &active);
        assert_eq!(roots.len(), 1, "should exclude active + dedupe cu1");
        assert_eq!(roots[0], cu1);
    }
}
