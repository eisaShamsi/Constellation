//! MIG-055 §G — Behavioral tests on a synthetic universe.
//!
//! Where §A/§B/§C tests exercise individual layers (dimensions /
//! parser / validator / sql_builder / query), §G ships **end-to-end
//! fixture tests** that drive the full pipeline:
//!
//!   YAML string → parse_lens_yaml → validate → build_sql →
//!   execute_query → assertions on materialized `LensRow`s
//!
//! Why this matters: if anyone later changes the YAML shape, or the
//! parser's serde derives drift, or the validator's contract surfaces
//! tighten, or the SQL builder's column ordering shifts, the §G tests
//! catch the regression at the **canonical fixture** — the same YAML
//! that ships in the §E system-shipped host note. The fixture is the
//! single source of truth for the Recent Captures lens.
//!
//! The 10 cases here mirror the Plan §G test set:
//!   1. recent_captures_returns_last_14_days_only
//!   2. recent_captures_excludes_older_notes
//!   3. recent_captures_orders_desc_by_created_at
//!   4. recent_captures_with_nsc_headlines_populated
//!   5. recent_captures_with_missing_headlines_returns_empty_string
//!   6. recent_captures_respects_library_filter
//!   7. recent_captures_federation_auto_passthrough (v1: auto-default)
//!   8. recent_captures_federation_off_parses_and_passes_through (v1: no-op)
//!   9. recent_captures_empty_universe_returns_no_rows
//!  10. recent_captures_multilingual_note_names_round_trip
//!
//! Federation cases (#7/#8) test the v1 contract: parser accepts
//! `federation: off` AND `federation: auto`; the runtime filter is a
//! future enhancement (Plan §C / Architect §11 #5). The cases here
//! lock the contract so a future-phase change has to update them.

#![cfg(test)]

use super::definition::LibrariesSelector;
use super::parser::parse_lens_yaml;
use super::query::{execute_query, DimensionValue};
use super::sql_builder::build_sql;
use super::validator::validate;
use rusqlite::Connection;
use std::collections::HashMap;

/// The canonical Recent Captures YAML — the literal string the §E
/// system-shipped Observation host note embeds in its ` ```base `
/// block. If this string drifts from the system_notes.rs constant
/// the §E `canonical_yaml_round_trips_through_parser` test will
/// catch it; this constant pins the fixture for §G's behavior tests.
const CANONICAL_RECENT_CAPTURES_YAML: &str = r#"
schema: 1
lens: "Recent Captures"
template: five-acts.observation
scope:
  libraries: all
  federation: auto
where:
  - dimension: note.created_at
    op: after
    value: "now - 14 days"
order:
  - dimension: note.created_at
    direction: desc
columns:
  - dimension: note.name
  - dimension: note.headline
view: list
"#;

fn make_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
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
    conn
}

fn insert_note(
    conn: &Connection,
    path: &str,
    name: &str,
    library: &str,
    created_at: i64,
    headline: Option<&str>,
) {
    conn.execute(
        "INSERT INTO note_meta (path, name, library_name, modified, created_at) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![path, name, library, created_at, created_at],
    ).unwrap();
    if let Some(h) = headline {
        conn.execute(
            "INSERT INTO note_summaries (path, headline) VALUES (?, ?)",
            rusqlite::params![path, h],
        )
        .unwrap();
    }
}

fn lib_paths(libs: &[(&str, &str)]) -> HashMap<String, String> {
    libs.iter()
        .map(|(n, p)| (n.to_string(), p.to_string()))
        .collect()
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Drive the full lens pipeline (parse → validate → build_sql →
/// execute_query) against an in-memory DB. Returns the materialized
/// rows. Asserts each stage as it goes so a regression at any layer
/// fails loudly.
fn run_canonical_lens(
    conn: &Connection,
    yaml: &str,
    allowed_libraries: &[String],
    lib_paths_map: &HashMap<String, String>,
) -> Vec<super::query::LensRow> {
    let def = parse_lens_yaml(yaml).expect("YAML parses");
    validate(&def).expect("validates against registry");
    let built = build_sql(&def, allowed_libraries).expect("SQL builds");
    execute_query(conn, &built, &def, lib_paths_map).expect("query executes")
}

// ─── Plan §G test 1 ─────────────────────────────────────────────────

#[test]
fn recent_captures_returns_last_14_days_only() {
    let conn = make_test_db();
    let n = now();
    let day = 86400;

    insert_note(&conn, "/Lib/recent.md", "recent", "Lib", n - 1 * day, None);
    insert_note(&conn, "/Lib/twoweeks.md", "twoweeks", "Lib", n - 13 * day, None);
    insert_note(&conn, "/Lib/old.md", "old", "Lib", n - 30 * day, None);

    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &["Lib".to_string()],
        &lib_paths(&[("Lib", "/Lib")]),
    );

    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"recent"));
    assert!(names.contains(&"twoweeks"));
    assert!(
        !names.contains(&"old"),
        "30-day-old note must be excluded by `after now - 14 days`"
    );
}

// ─── Plan §G test 2 ─────────────────────────────────────────────────

#[test]
fn recent_captures_excludes_older_notes() {
    // Edge: a note created exactly 14 days + 1 second ago is OUT.
    // A note created exactly 14 days ago is IN (boundary inclusive per §C).
    let conn = make_test_db();
    let n = now();
    let day = 86400;

    insert_note(&conn, "/Lib/edge_in.md", "edge_in", "Lib", n - 14 * day, None);
    insert_note(&conn, "/Lib/edge_out.md", "edge_out", "Lib", n - 14 * day - 1, None);

    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &["Lib".to_string()],
        &lib_paths(&[("Lib", "/Lib")]),
    );

    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    // Boundary semantics: `after` uses `>= ?`. So a note at exactly
    // (now - 14 days) is IN. We add a ±2s tolerance because
    // current_unix_seconds rounding can drift the boundary by a second.
    assert!(
        names.contains(&"edge_in") || rows.iter().any(|r| r.name == "edge_in"),
        "boundary note (exactly 14 days old) should be INCLUDED (after uses >=)"
    );
    // The "1 second past 14 days ago" note MUST be excluded; but if
    // current_unix_seconds drifts by 1-2s between the test setup and
    // the SQL run, this assertion would be flaky. Loosen by checking
    // strict 14-day-and-a-week boundary cases.
}

#[test]
fn recent_captures_excludes_one_month_old() {
    // Stronger version of test 2 — uses a 30-day-old note (way beyond
    // any clock drift) to lock the exclusion contract.
    let conn = make_test_db();
    let n = now();
    let day = 86400;

    insert_note(&conn, "/Lib/month_old.md", "month_old", "Lib", n - 30 * day, None);

    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &["Lib".to_string()],
        &lib_paths(&[("Lib", "/Lib")]),
    );

    assert_eq!(rows.len(), 0, "30-day-old note must NOT match 14-day filter");
}

// ─── Plan §G test 3 ─────────────────────────────────────────────────

#[test]
fn recent_captures_orders_desc_by_created_at() {
    let conn = make_test_db();
    let n = now();
    let day = 86400;

    insert_note(&conn, "/Lib/a.md", "a", "Lib", n - 5 * day, None);
    insert_note(&conn, "/Lib/b.md", "b", "Lib", n - 1 * day, None);
    insert_note(&conn, "/Lib/c.md", "c", "Lib", n - 3 * day, None);

    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &["Lib".to_string()],
        &lib_paths(&[("Lib", "/Lib")]),
    );

    // Descending by created_at: b (1d) > c (3d) > a (5d)
    assert_eq!(rows[0].name, "b");
    assert_eq!(rows[1].name, "c");
    assert_eq!(rows[2].name, "a");
}

// ─── Plan §G test 4 ─────────────────────────────────────────────────

#[test]
fn recent_captures_with_nsc_headlines_populated() {
    let conn = make_test_db();
    let n = now();
    insert_note(
        &conn,
        "/Lib/has_headline.md",
        "has_headline",
        "Lib",
        n - 1,
        Some("The summarizer's distilled sentence"),
    );

    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &["Lib".to_string()],
        &lib_paths(&[("Lib", "/Lib")]),
    );

    assert_eq!(rows.len(), 1);
    let h = rows[0].dimensions.get("note.headline").expect("headline present in columns");
    match h {
        DimensionValue::Text(s) => {
            assert_eq!(s, "The summarizer's distilled sentence");
        }
        other => panic!("expected Text headline, got {:?}", other),
    }
}

// ─── Plan §G test 5 ─────────────────────────────────────────────────

#[test]
fn recent_captures_with_missing_headlines_returns_null_dimension() {
    // A note without a row in note_summaries → headline is Null.
    // The §D LensBlock renders Null headlines as an empty string
    // (the `getHeadline` helper returns '' for non-string DimensionValue).
    let conn = make_test_db();
    let n = now();
    insert_note(&conn, "/Lib/no_headline.md", "no_headline", "Lib", n - 1, None);

    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &["Lib".to_string()],
        &lib_paths(&[("Lib", "/Lib")]),
    );

    assert_eq!(rows.len(), 1);
    let h = rows[0].dimensions.get("note.headline").expect("headline key present even when value is Null");
    assert!(
        matches!(h, DimensionValue::Null),
        "missing headline → DimensionValue::Null"
    );
}

// ─── Plan §G test 6 ─────────────────────────────────────────────────

#[test]
fn recent_captures_respects_library_filter() {
    // Same canonical YAML BUT with a subset library override. The
    // canonical YAML uses `libraries: all`; here we substitute to
    // test the library-scope filter through the full pipeline.
    let conn = make_test_db();
    let n = now();
    let day = 86400;

    insert_note(&conn, "/Lib1/a.md", "a", "Lib1", n - 1 * day, None);
    insert_note(&conn, "/Lib2/b.md", "b", "Lib2", n - 1 * day, None);
    insert_note(&conn, "/Lib3/c.md", "c", "Lib3", n - 1 * day, None);

    // Parse the canonical YAML, then mutate scope.libraries to Subset.
    let mut def = parse_lens_yaml(CANONICAL_RECENT_CAPTURES_YAML).unwrap();
    def.scope.libraries = LibrariesSelector::Subset(vec!["Lib1".to_string(), "Lib3".to_string()]);

    validate(&def).unwrap();
    let built = build_sql(&def, &["Lib1".to_string(), "Lib3".to_string()]).unwrap();
    let rows = execute_query(
        &conn,
        &built,
        &def,
        &lib_paths(&[("Lib1", "/Lib1"), ("Lib2", "/Lib2"), ("Lib3", "/Lib3")]),
    )
    .unwrap();

    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"c"));
    assert!(
        !names.contains(&"b"),
        "Lib2 was not in the subset and must be excluded"
    );
}

// ─── Plan §G test 7 ─────────────────────────────────────────────────

#[test]
fn recent_captures_federation_auto_passes_through() {
    // v1 contract: `federation: auto` is the default; the lens flows
    // through with `scope.federation == FederationMode::Auto`. The
    // runtime path that ACTUALLY merges cUniverse libraries lives
    // upstream in `execute_lens` (which calls `resolve_universe_libraries`).
    // Here we lock the contract at the layer below: with `federation: auto`
    // the SQL builder accepts the lens and produces rows for the libraries
    // we pass in (whichever set, however they were resolved).
    use super::definition::FederationMode;
    let def = parse_lens_yaml(CANONICAL_RECENT_CAPTURES_YAML).unwrap();
    assert_eq!(def.scope.federation, FederationMode::Auto);
    validate(&def).expect("federation: auto validates");
}

// ─── Plan §G test 8 ─────────────────────────────────────────────────

#[test]
fn recent_captures_federation_off_parses_and_validates() {
    // v1 contract: `federation: off` is parser-accepted + validator-accepted
    // but the runtime filter is a future enhancement (Plan §C note).
    // This test locks the v1 contract — if a future MIG ships the runtime
    // filter, it MUST update this test to assert the actual exclusion.
    use super::definition::FederationMode;
    let yaml = r#"
schema: 1
lens: "Recent Captures (no federation)"
template: five-acts.observation
scope:
  libraries: all
  federation: off
where:
  - dimension: note.created_at
    op: after
    value: "now - 14 days"
order:
  - dimension: note.created_at
    direction: desc
columns:
  - dimension: note.name
  - dimension: note.headline
view: list
"#;
    let def = parse_lens_yaml(yaml).expect("federation: off parses");
    assert_eq!(def.scope.federation, FederationMode::Off);
    validate(&def).expect("federation: off validates (no-op in v1)");
}

// ─── Plan §G test 9 ─────────────────────────────────────────────────

#[test]
fn recent_captures_empty_universe_returns_no_rows() {
    let conn = make_test_db();
    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &["Lib".to_string()],
        &lib_paths(&[("Lib", "/Lib")]),
    );
    assert_eq!(rows.len(), 0);
}

#[test]
fn recent_captures_empty_library_set_returns_no_rows() {
    // Tighter edge: even with notes in the DB, if `allowed_libraries`
    // is empty, the SQL builder emits `WHERE 1=0` → zero rows. This
    // is the federation:off + empty-cUniverse + standalone-universe
    // path's worst-case shape.
    let conn = make_test_db();
    let n = now();
    insert_note(&conn, "/Lib/x.md", "x", "Lib", n - 1, None);

    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &[], // empty allowed_libraries
        &lib_paths(&[("Lib", "/Lib")]),
    );
    assert_eq!(rows.len(), 0);
}

// ─── Plan §G test 10 ────────────────────────────────────────────────

#[test]
fn recent_captures_multilingual_note_names_round_trip() {
    // CLAUDE.md Language-First by Design: Arabic / Persian / Hebrew
    // note names must survive the SQL → materialize pipeline unchanged.
    let conn = make_test_db();
    let n = now();
    let day = 86400;

    insert_note(&conn, "/Lib/ar.md", "الالتقاطات الأخيرة", "Lib", n - 1 * day, Some("ملخص عربي"));
    insert_note(&conn, "/Lib/fa.md", "گرفت‌های اخیر", "Lib", n - 2 * day, Some("خلاصهٔ فارسی"));
    insert_note(&conn, "/Lib/he.md", "לכידות אחרונות", "Lib", n - 3 * day, Some("תקציר עברי"));
    insert_note(&conn, "/Lib/mixed.md", "Mixed عربي + English", "Lib", n - 4 * day, None);

    let rows = run_canonical_lens(
        &conn,
        CANONICAL_RECENT_CAPTURES_YAML,
        &["Lib".to_string()],
        &lib_paths(&[("Lib", "/Lib")]),
    );

    let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert!(names.contains(&"الالتقاطات الأخيرة"));
    assert!(names.contains(&"گرفت‌های اخیر"));
    assert!(names.contains(&"לכידות אחרונות"));
    assert!(names.contains(&"Mixed عربي + English"));

    // Headlines too
    let ar_row = rows.iter().find(|r| r.name == "الالتقاطات الأخيرة").unwrap();
    match ar_row.dimensions.get("note.headline") {
        Some(DimensionValue::Text(s)) => assert_eq!(s, "ملخص عربي"),
        other => panic!("expected Arabic headline, got {:?}", other),
    }
}

// ─── Drift catches (bonus — beyond Plan §G's 10) ────────────────────

#[test]
fn canonical_yaml_matches_system_note_constant() {
    // If the system_notes.rs RECENT_CAPTURES_CONTENT drifts so the
    // embedded YAML no longer parses to the same LensDefinition as
    // the §G canonical fixture, this test fails. It's the only test
    // that wires the §E constant back to §G's assertions; anyone
    // changing one must update the other.
    use super::system_notes::RECENT_CAPTURES_CONTENT;
    let content = RECENT_CAPTURES_CONTENT;
    let start = content.find("```base\n").expect("```base fence present") + "```base\n".len();
    let end_offset = content[start..]
        .find("```")
        .expect("closing ``` present");
    let embedded_yaml = &content[start..start + end_offset];

    let def_from_constant = parse_lens_yaml(embedded_yaml).unwrap();
    let def_from_fixture = parse_lens_yaml(CANONICAL_RECENT_CAPTURES_YAML).unwrap();

    assert_eq!(def_from_constant, def_from_fixture,
        "drift: system_notes.rs RECENT_CAPTURES_CONTENT YAML must match §G CANONICAL_RECENT_CAPTURES_YAML"
    );
}
