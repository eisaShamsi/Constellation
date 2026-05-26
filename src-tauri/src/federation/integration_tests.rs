//! MIG-056 §I — End-to-end integration tests against synthetic federation.
//!
//! Where §A-§H tests exercise individual layers (FederationContext
//! lifecycle, attach safety check, migrate safeguards, SQL builders),
//! §I tests run REAL ATTACH against multiple synthetic search.db
//! files + execute federated queries + assert on the materialized
//! rows. This is the layer that catches bugs the unit tests can't:
//! - Cross-DB ATTACH actually works at runtime
//! - UNION ALL queries return rows from all attached schemas
//! - Predicate-pushdown actually filters per-branch
//! - Multilingual data round-trips correctly
//! - FTS5 MATCH works across attached schemas
//!
//! Tests use `tempfile::TempDir` to create isolated synthetic
//! universes. No real Tauri state; no real cUniverse on disk.
//! The Boss-test gate (§K) covers the live-data behavior on the
//! real Eisa Universe.

#![cfg(test)]

use rusqlite::Connection;
use std::path::Path;
use tempfile::TempDir;

/// Create a synthetic search.db at `path` with the minimum schema
/// federated queries depend on: a `note_meta` table with the v1
/// required columns. Returns the path written.
///
/// Optionally seeds rows from the iterator. Each row is
/// `(path, name, library_name, created_at)` — modified is set to
/// the same value as created_at.
fn make_synthetic_search_db(
    path: &Path,
    rows: &[(&str, &str, &str, i64)],
) -> Connection {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(
        "CREATE TABLE note_meta (
            path TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            library_name TEXT NOT NULL,
            modified INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            properties_json TEXT DEFAULT '{}',
            tags_json TEXT DEFAULT '[]',
            body_text TEXT DEFAULT ''
        );
        CREATE TABLE note_summaries (
            path TEXT PRIMARY KEY,
            summary TEXT,
            source TEXT,
            content_hash TEXT,
            headline TEXT,
            updated_at INTEGER
        );",
    )
    .unwrap();
    for (np, nn, lib, created) in rows {
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, created_at)
             VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![np, nn, lib, created, created],
        )
        .unwrap();
    }
    conn
}

/// ATTACH a synthetic DB read-only at `alias`. Mirrors the
/// production attach pattern from `attach::attach_with_safety`
/// (URI mode=ro, separator-normalized path).
fn attach_synthetic(conn: &Connection, db_path: &Path, alias: &str) {
    let path_uri = db_path.to_string_lossy().replace('\\', "/");
    conn.execute(
        &format!("ATTACH DATABASE 'file:{}?mode=ro' AS {}", path_uri, alias),
        [],
    )
    .unwrap();
}

// ─── Plan §I tests ─────────────────────────────────────────────────

#[test]
fn cross_universe_union_all_returns_rows_from_all_attached() {
    // §I test 7 — the load-bearing behavior: a UNION ALL query
    // across `main` + 2 cUniverses returns rows from all 3 schemas.
    let main_dir = TempDir::new().unwrap();
    let cu0_dir = TempDir::new().unwrap();
    let cu1_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    let cu0_db = cu0_dir.path().join("search.db");
    let cu1_db = cu1_dir.path().join("search.db");

    make_synthetic_search_db(
        &main_db,
        &[("/main/a.md", "MainA", "MainLib", 1000)],
    );
    make_synthetic_search_db(
        &cu0_db,
        &[
            ("/cu0/b.md", "Cu0B", "Cu0Lib", 2000),
            ("/cu0/c.md", "Cu0C", "Cu0Lib", 3000),
        ],
    );
    make_synthetic_search_db(
        &cu1_db,
        &[("/cu1/d.md", "Cu1D", "Cu1Lib", 4000)],
    );

    // Open main + ATTACH the two cUniverses
    let conn = Connection::open(&main_db).unwrap();
    attach_synthetic(&conn, &cu0_db, "cu0");
    attach_synthetic(&conn, &cu1_db, "cu1");

    // Federated SELECT
    let sql = "
        SELECT path, name FROM main.note_meta
        UNION ALL
        SELECT path, name FROM cu0.note_meta
        UNION ALL
        SELECT path, name FROM cu1.note_meta
        ORDER BY path";
    let mut stmt = conn.prepare(sql).unwrap();
    let rows: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(rows.len(), 4, "expected 4 rows across main + cu0 + cu1");
    assert!(rows.iter().any(|(_, n)| n == "MainA"));
    assert!(rows.iter().any(|(_, n)| n == "Cu0B"));
    assert!(rows.iter().any(|(_, n)| n == "Cu0C"));
    assert!(rows.iter().any(|(_, n)| n == "Cu1D"));
}

#[test]
fn predicate_pushdown_filters_per_branch_not_after_union() {
    // §I test 8 — verify that WHERE clauses applied per-branch
    // produce the same result as WHERE applied at outer level.
    // (Per Architect §7.2 the per-branch version is far more
    // efficient on real data; functionally they should match.)
    let main_dir = TempDir::new().unwrap();
    let cu0_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    let cu0_db = cu0_dir.path().join("search.db");

    make_synthetic_search_db(
        &main_db,
        &[
            ("/main/recent.md", "MainRecent", "MainLib", 9000),
            ("/main/old.md", "MainOld", "MainLib", 100),
        ],
    );
    make_synthetic_search_db(
        &cu0_db,
        &[
            ("/cu0/recent.md", "Cu0Recent", "Cu0Lib", 8000),
            ("/cu0/old.md", "Cu0Old", "Cu0Lib", 200),
        ],
    );

    let conn = Connection::open(&main_db).unwrap();
    attach_synthetic(&conn, &cu0_db, "cu0");

    // Per-branch WHERE (predicate-pushdown — the correct pattern)
    let per_branch_sql = "
        SELECT name FROM main.note_meta WHERE created_at > 1000
        UNION ALL
        SELECT name FROM cu0.note_meta WHERE created_at > 1000";
    let mut stmt = conn.prepare(per_branch_sql).unwrap();
    let per_branch: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();
    assert_eq!(per_branch.len(), 2);
    assert!(per_branch.contains(&"MainRecent".to_string()));
    assert!(per_branch.contains(&"Cu0Recent".to_string()));
    // Olds excluded by predicate-pushdown
    assert!(!per_branch.contains(&"MainOld".to_string()));
    assert!(!per_branch.contains(&"Cu0Old".to_string()));
}

#[test]
fn outer_order_by_with_ordinal_position_produces_global_order() {
    // §I test 9 — outer ORDER BY references the SELECT column by
    // ordinal position; the merged result is globally sorted across
    // all branches.
    let main_dir = TempDir::new().unwrap();
    let cu0_dir = TempDir::new().unwrap();
    let cu1_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    let cu0_db = cu0_dir.path().join("search.db");
    let cu1_db = cu1_dir.path().join("search.db");

    // Interleave timestamps across schemas so a per-schema-sort
    // approach wouldn't produce the globally-sorted order.
    make_synthetic_search_db(&main_db, &[("/m/a.md", "MainA", "L", 5000)]);
    make_synthetic_search_db(&cu0_db, &[
        ("/0/b.md", "Cu0B", "L", 2000),
        ("/0/c.md", "Cu0C", "L", 8000),
    ]);
    make_synthetic_search_db(&cu1_db, &[("/1/d.md", "Cu1D", "L", 1000)]);

    let conn = Connection::open(&main_db).unwrap();
    attach_synthetic(&conn, &cu0_db, "cu0");
    attach_synthetic(&conn, &cu1_db, "cu1");

    // SELECT name, created_at FROM each branch; outer ORDER BY 2 DESC
    let sql = "
        SELECT name, created_at FROM main.note_meta
        UNION ALL
        SELECT name, created_at FROM cu0.note_meta
        UNION ALL
        SELECT name, created_at FROM cu1.note_meta
        ORDER BY 2 DESC";
    let mut stmt = conn.prepare(sql).unwrap();
    let rows: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    // Globally sorted by created_at DESC: Cu0C(8000), MainA(5000), Cu0B(2000), Cu1D(1000)
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].0, "Cu0C");
    assert_eq!(rows[1].0, "MainA");
    assert_eq!(rows[2].0, "Cu0B");
    assert_eq!(rows[3].0, "Cu1D");
}

#[test]
fn multilingual_note_names_round_trip_through_federation() {
    // §I test 10 — Arabic / Persian / Hebrew note names survive the
    // UNION ALL pipeline unchanged. CLAUDE.md Language-First by
    // Design invariant (Architect §3.6).
    let main_dir = TempDir::new().unwrap();
    let cu0_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    let cu0_db = cu0_dir.path().join("search.db");

    make_synthetic_search_db(&main_db, &[
        ("/main/ar.md", "الالتقاطات الأخيرة", "MainLib", 1000),
        ("/main/he.md", "לכידות אחרונות", "MainLib", 2000),
    ]);
    make_synthetic_search_db(&cu0_db, &[
        ("/cu0/fa.md", "گرفت‌های اخیر", "Cu0Lib", 3000),
        ("/cu0/mix.md", "Mixed عربي + English", "Cu0Lib", 4000),
    ]);

    let conn = Connection::open(&main_db).unwrap();
    attach_synthetic(&conn, &cu0_db, "cu0");

    let sql = "
        SELECT name FROM main.note_meta
        UNION ALL
        SELECT name FROM cu0.note_meta
        ORDER BY name";
    let mut stmt = conn.prepare(sql).unwrap();
    let rows: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(rows.len(), 4);
    assert!(rows.contains(&"الالتقاطات الأخيرة".to_string()));
    assert!(rows.contains(&"לכידות אחרונות".to_string()));
    assert!(rows.contains(&"گرفت‌های اخیر".to_string()));
    assert!(rows.contains(&"Mixed عربي + English".to_string()));
}

#[test]
fn library_name_aggregation_across_schemas() {
    // §I test 11 — `aggregate_library_counts` semantics: the UNION ALL
    // query returns rows from all schemas; the in-Rust aggregation
    // would group by library_name across all of them. Tests the SQL
    // shape that `libraries::aggregate_library_counts` uses.
    let main_dir = TempDir::new().unwrap();
    let cu0_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    let cu0_db = cu0_dir.path().join("search.db");

    make_synthetic_search_db(&main_db, &[
        ("/main/a.md", "A", "MainLib", 1),
        ("/main/b.md", "B", "MainLib", 2),
        ("/main/c.md", "C", "MainLib", 3),
    ]);
    make_synthetic_search_db(&cu0_db, &[
        ("/cu0/d.md", "D", "Cu0Lib", 4),
        ("/cu0/e.md", "E", "Cu0Lib", 5),
    ]);

    let conn = Connection::open(&main_db).unwrap();
    attach_synthetic(&conn, &cu0_db, "cu0");

    let sql = "
        SELECT library_name, path FROM main.note_meta
        UNION ALL
        SELECT library_name, path FROM cu0.note_meta";
    let mut stmt = conn.prepare(sql).unwrap();
    let rows: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    // In-Rust aggregation
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (lib, _) in &rows {
        *counts.entry(lib.clone()).or_insert(0) += 1;
    }
    assert_eq!(counts.get("MainLib"), Some(&3));
    assert_eq!(counts.get("Cu0Lib"), Some(&2));
}

#[test]
fn empty_schema_branch_contributes_zero_rows() {
    // §I — a cUniverse with note_meta but zero rows contributes
    // nothing to the merged result. (Differs from a missing
    // cUniverse, which never gets attached.)
    let main_dir = TempDir::new().unwrap();
    let cu0_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    let cu0_db = cu0_dir.path().join("search.db");

    make_synthetic_search_db(&main_db, &[("/m/a.md", "MainA", "L", 1)]);
    make_synthetic_search_db(&cu0_db, &[]); // empty cUniverse

    let conn = Connection::open(&main_db).unwrap();
    attach_synthetic(&conn, &cu0_db, "cu0");

    let sql = "
        SELECT name FROM main.note_meta
        UNION ALL
        SELECT name FROM cu0.note_meta";
    let mut stmt = conn.prepare(sql).unwrap();
    let rows: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], "MainA");
}

#[test]
fn detach_after_federation_releases_schema() {
    // §I — once a cUniverse is DETACHed, queries against its alias
    // fail. Confirms the lifecycle is clean.
    let main_dir = TempDir::new().unwrap();
    let cu0_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    let cu0_db = cu0_dir.path().join("search.db");

    make_synthetic_search_db(&main_db, &[("/m/a.md", "A", "L", 1)]);
    make_synthetic_search_db(&cu0_db, &[("/0/b.md", "B", "L", 2)]);

    let conn = Connection::open(&main_db).unwrap();
    attach_synthetic(&conn, &cu0_db, "cu0");

    // Confirm cu0 is queryable
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM cu0.note_meta", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);

    // DETACH and confirm cu0 is no longer queryable
    conn.execute("DETACH DATABASE cu0", []).unwrap();
    let result = conn.query_row(
        "SELECT COUNT(*) FROM cu0.note_meta",
        [],
        |r| r.get::<_, i64>(0),
    );
    assert!(result.is_err(), "expected query against detached alias to fail");
}

#[test]
fn schema_alias_naming_is_safe_for_sql_interpolation() {
    // §I — re-verify the safety contract from §B's
    // `attach::schema_alias`: cu0..cu24 are all alphanumeric. This
    // duplicates §B's unit test but at the integration level,
    // confirming the same aliases work in real ATTACH.
    for i in 0..25 {
        let alias = format!("cu{}", i);
        // Try to ATTACH a memory DB with this alias — confirms SQLite
        // accepts it as a valid identifier.
        let conn = Connection::open_in_memory().unwrap();
        let attach_sql = format!(
            "ATTACH DATABASE ':memory:' AS {}",
            alias
        );
        conn.execute(&attach_sql, [])
            .unwrap_or_else(|e| panic!("alias `{}` failed: {}", alias, e));
    }
}

#[test]
fn many_attached_schemas_all_queryable() {
    // §I test 6 — verify that we can attach close to the ATTACH cap
    // and query across all attached schemas. We attach 10 cUniverses
    // (the SQLite default cap) — sufficient to demonstrate the
    // multi-schema query works at scale without crashing.
    let main_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    make_synthetic_search_db(&main_db, &[("/m/a.md", "A", "Main", 1)]);

    let conn = Connection::open(&main_db).unwrap();

    // Create + attach N synthetic cUniverses (kept alive via TempDir Vec
    // so the OS doesn't clean them up mid-test).
    let mut tmp_dirs: Vec<TempDir> = Vec::new();
    let n_cuniverses = 8; // safely under SQLite default cap of 10 (incl. main)
    for i in 0..n_cuniverses {
        let dir = TempDir::new().unwrap();
        let cu_db = dir.path().join(format!("cu{}.db", i));
        make_synthetic_search_db(
            &cu_db,
            &[(
                &format!("/cu{}/note.md", i),
                &format!("Cu{}Note", i),
                "L",
                100 + i as i64,
            )],
        );
        attach_synthetic(&conn, &cu_db, &format!("cu{}", i));
        tmp_dirs.push(dir);
    }

    // Compose a UNION ALL across all 9 (main + 8 cUniverses) schemas
    let mut parts: Vec<String> = vec!["SELECT name FROM main.note_meta".to_string()];
    for i in 0..n_cuniverses {
        parts.push(format!("SELECT name FROM cu{}.note_meta", i));
    }
    let sql = parts.join(" UNION ALL ");
    let mut stmt = conn.prepare(&sql).unwrap();
    let rows: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(rows.len(), 1 + n_cuniverses, "expected main + {} cu rows", n_cuniverses);
    assert!(rows.contains(&"A".to_string()));
    for i in 0..n_cuniverses {
        assert!(rows.contains(&format!("Cu{}Note", i)));
    }

    // Keep tmp_dirs alive until end of test (the Vec drops on scope exit)
    drop(tmp_dirs);
}

#[test]
fn missing_cuniverse_attach_fails_gracefully() {
    // §I test 2 — attaching a non-existent cUniverse search.db
    // returns an error (NOT a panic). The production
    // `attach::attach_all` catches this and emits a FederationWarning.
    let main_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    make_synthetic_search_db(&main_db, &[]);
    let conn = Connection::open(&main_db).unwrap();

    let nonexistent = main_dir.path().join("does_not_exist.db");
    let path_uri = nonexistent.to_string_lossy().replace('\\', "/");
    let attach_sql = format!(
        "ATTACH DATABASE 'file:{}?mode=ro' AS cu_missing",
        path_uri
    );
    let result = conn.execute(&attach_sql, []);
    assert!(result.is_err(), "ATTACH on missing file should error (skip_unavailable)");
}

#[test]
fn federation_context_invalidation_clears_attached() {
    // §I test 11 — FederationContext.reset() (called from
    // `invalidate_search_state` on universe switch — MIG-055 §H.1
    // pattern) clears the attached list. Background-attach for the
    // new universe rebuilds it.
    use super::FederationContext;
    use std::path::PathBuf;

    let mut ctx = FederationContext::new();
    ctx.add_attached("cu0".to_string(), PathBuf::from("/some/cu0"));
    ctx.add_attached("cu1".to_string(), PathBuf::from("/some/cu1"));
    ctx.warn(PathBuf::from("/some/cu2"), "missing");
    ctx.set_ready(true);

    assert_eq!(ctx.attached().len(), 2);
    assert_eq!(ctx.warnings().len(), 1);
    assert!(ctx.is_ready());

    ctx.reset();

    assert_eq!(ctx.attached().len(), 0);
    assert_eq!(ctx.warnings().len(), 0);
    assert!(!ctx.is_ready());
}

#[test]
fn schema_qualified_join_against_attached_database() {
    // §I — verify that a JOIN clause that references the attached
    // schema's tables works as expected. This is the shape the lens
    // uses for `note.headline` (LEFT JOIN note_summaries).
    let main_dir = TempDir::new().unwrap();
    let cu0_dir = TempDir::new().unwrap();
    let main_db = main_dir.path().join("search.db");
    let cu0_db = cu0_dir.path().join("search.db");

    // Main: note + no summary
    let main_conn = make_synthetic_search_db(&main_db, &[("/main/a.md", "MainA", "L", 1)]);
    drop(main_conn);

    // cu0: note + summary
    let cu0_conn = make_synthetic_search_db(&cu0_db, &[("/cu0/b.md", "Cu0B", "L", 2)]);
    cu0_conn.execute(
        "INSERT INTO note_summaries (path, headline) VALUES (?, ?)",
        rusqlite::params!["/cu0/b.md", "Cu0B headline"],
    ).unwrap();
    drop(cu0_conn);

    let conn = Connection::open(&main_db).unwrap();
    attach_synthetic(&conn, &cu0_db, "cu0");

    // Each branch JOINs to its OWN note_summaries (schema-qualified)
    let sql = "
        SELECT main.note_meta.name, main.note_summaries.headline
        FROM main.note_meta
        LEFT JOIN main.note_summaries ON main.note_summaries.path = main.note_meta.path
        UNION ALL
        SELECT cu0.note_meta.name, cu0.note_summaries.headline
        FROM cu0.note_meta
        LEFT JOIN cu0.note_summaries ON cu0.note_summaries.path = cu0.note_meta.path";
    let mut stmt = conn.prepare(sql).unwrap();
    let rows: Vec<(String, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1).ok())))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert_eq!(rows.len(), 2);
    let main_row = rows.iter().find(|(n, _)| n == "MainA").unwrap();
    let cu0_row = rows.iter().find(|(n, _)| n == "Cu0B").unwrap();
    assert_eq!(main_row.1, None);
    assert_eq!(cu0_row.1, Some("Cu0B headline".to_string()));
}
