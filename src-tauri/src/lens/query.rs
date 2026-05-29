//! MIG-055 §C — `execute_lens` Tauri command.
//!
//! The single entrypoint frontend code (the §D LensBlock.svelte
//! renderer) uses to render a lens. Pipeline:
//!
//!   1. parse_lens_yaml — YAML → LensDefinition
//!   2. validate       — semantic check against the §A dimension registry
//!   3. resolve_libs   — federated library set from universe.rs
//!   4. build_sql      — SQL string + parameters (§C sql_builder)
//!   5. execute        — query the search DB
//!   6. materialize    — column-position → LensRow.dimensions HashMap

use super::definition::{FederationMode, LensDefinition, LensSort, LensView, LibrariesSelector};
use super::dimensions::resolve_dim;
use super::parser::parse_lens_yaml;
use super::sql_builder::{build_federated_sql, build_sql, BuiltQuery};
use super::validator::validate;
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;
use tauri::Manager;

/// Returned to the frontend by `execute_lens`.
#[derive(Debug, Clone, Serialize)]
pub struct LensResult {
    pub rows: Vec<LensRow>,
    pub total_count: usize,
    pub query_time_ms: u64,
    pub lens_name: String,
    pub template: Option<String>,
    /// MIG-065 §F — render shape ("list" | "table"); the frontend renders
    /// accordingly.
    pub view: String,
    /// MIG-065 §F — declared column dimension names, in order. The table
    /// renders headers in this order (the per-row dimensions map is unordered).
    pub columns: Vec<String>,
    /// MIG-065 §G.2 — the active sort clauses (`order:` in the `.base`), so the
    /// table can render sort arrows + cycle direction on header click without
    /// re-parsing the YAML. Empty = unsorted.
    pub order: Vec<LensSort>,
}

/// One row of a lens result.
#[derive(Debug, Clone, Serialize)]
pub struct LensRow {
    /// Note's filesystem path (always present, regardless of `columns:`).
    pub note_path: String,
    /// Note's name as stored in note_meta (always present).
    pub name: String,
    /// Library this note belongs to (always present).
    pub library_name: String,
    /// Library's filesystem path (resolved from the universe's libraries.json).
    pub library_path: String,
    /// Dimensions the lens declared in `columns:`, keyed by dimension name.
    pub dimensions: HashMap<String, DimensionValue>,
}

/// A single dimension value, serde-tagged untagged so the frontend
/// receives natural JSON shapes.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum DimensionValue {
    Text(String),
    Number(f64),
    Timestamp(i64),
    Bool(bool),
    Null,
}

/// The Tauri command. Frontend calls this with the YAML text from a
/// ` ```base ` code block.
#[tauri::command]
pub fn execute_lens(app: tauri::AppHandle, lens_yaml: String) -> Result<LensResult, String> {
    let start = Instant::now();

    // 1. Parse.
    let def = parse_lens_yaml(&lens_yaml).map_err(|e| e.to_string())?;

    // 2. Validate.
    validate(&def).map_err(|e| e.to_string())?;

    // 3. Resolve federated library set.
    let all_libs = crate::universe::resolve_universe_libraries(app.clone())?;
    let allowed_libs: Vec<String> = match &def.scope.libraries {
        LibrariesSelector::All => all_libs.iter().map(|l| l.name.clone()).collect(),
        LibrariesSelector::Subset(subset) => all_libs
            .iter()
            .filter(|l| subset.contains(&l.name))
            .map(|l| l.name.clone())
            .collect(),
    };
    // Note: federation `auto` vs `off` would filter the all_libs set differently;
    // resolve_universe_libraries already includes cUniverse children, so for
    // `federation: off` we'd need to filter to the current universe's libraries
    // only. v1 ships `auto` as the default per Architect §11 #5; the `off` path
    // is a future enhancement (the parser already accepts the value).
    let lib_path_map: HashMap<String, String> = all_libs
        .iter()
        .map(|l| (l.name.clone(), l.path.clone()))
        .collect();

    // 4. Decide: federated path (MIG-056) or single-schema path?
    // MIG-056 §E: when scope.federation == Auto AND the federation
    // context is ready AND has cUniverses attached, use the federated
    // UNION ALL query against state.federated_conn. Otherwise fall
    // back to the existing single-schema path against state.db.
    let federation_ready_and_auto =
        def.scope.federation == FederationMode::Auto && federation_has_attached(&app);

    // Try federated; on any failure (race during universe switch leaves
    // federated_conn = None; transient lock issues; etc.) fall back to
    // single-schema. Matches the sibling consumer pattern in
    // `libraries::aggregate_library_counts` and
    // `search::federated_lexical_search_or_fallback`. Per the §J drift
    // audit's P1-1 finding — without this fallback the lens errors out
    // instead of degrading gracefully (the skip_unavailable semantic).
    let rows = (|| -> Result<Vec<LensRow>, String> {
        if federation_ready_and_auto {
            let attached_aliases = federation_attached_aliases(&app);
            let mut schemas: Vec<&str> = vec!["main"];
            for alias in &attached_aliases {
                schemas.push(alias.as_str());
            }
            let built = build_federated_sql(&def, &allowed_libs, &schemas)?;
            match execute_federated_query(&app, &built, &def, &lib_path_map) {
                Ok(rows) => return Ok(rows),
                Err(e) => {
                    // Race during universe switch, transient lock, etc.
                    // Log + fall through to single-schema path.
                    eprintln!("[lens] federated query failed; falling back to single-schema: {}", e);
                }
            }
        }
        // Single-schema fallback (existing MIG-055 behavior).
        let built = build_sql(&def, &allowed_libs)?;
        let db_path = crate::search::db_path(&app)
            .map_err(|e| format!("Failed to resolve search DB path: {}", e))?;
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open search DB: {}", e))?;
        execute_query(&conn, &built, &def, &lib_path_map)
    })()?;

    let total_count = rows.len();
    let query_time_ms = start.elapsed().as_millis() as u64;

    Ok(LensResult {
        rows,
        total_count,
        query_time_ms,
        lens_name: def.lens.clone(),
        template: def.template.clone(),
        view: match def.view {
            LensView::Table => "table".to_string(),
            LensView::List => "list".to_string(),
        },
        columns: def.columns.iter().map(|c| c.dimension.clone()).collect(),
        order: def.order.clone(),
    })
}

/// Execute the built SQL + materialize LensRows. Pulled out so tests
/// can drive it with an in-memory Connection.
pub(crate) fn execute_query(
    conn: &Connection,
    built: &BuiltQuery,
    def: &LensDefinition,
    lib_path_map: &HashMap<String, String>,
) -> Result<Vec<LensRow>, String> {
    let mut stmt = conn.prepare(&built.sql)
        .map_err(|e| format!("Failed to prepare lens SQL: {}\nSQL: {}", e, &built.sql))?;

    let row_count = stmt.column_count();

    let mut rows: Vec<LensRow> = Vec::new();

    let mut query_iter = stmt
        .query(rusqlite::params_from_iter(built.params.iter()))
        .map_err(|e| format!("Failed to execute lens SQL: {}", e))?;

    while let Some(row) = query_iter.next()
        .map_err(|e| format!("Failed to read lens row: {}", e))?
    {
        let note_path: String = row.get(0).unwrap_or_default();
        let name: String = row.get(1).unwrap_or_default();
        let library_name: String = row.get(2).unwrap_or_default();
        let library_path = lib_path_map.get(&library_name).cloned().unwrap_or_default();

        // Populate dimensions HashMap from the index map.
        let mut dimensions: HashMap<String, DimensionValue> = HashMap::new();
        for (col_idx, dim_name) in &built.dimension_index_map {
            if *col_idx >= row_count {
                continue; // safety
            }
            // MIG-065 §E — resolve_dim covers both registered dimensions and
            // `prop.<key>` frontmatter columns (Text kind). Using lookup_dimension
            // here would error on every prop.* column.
            let kind = resolve_dim(dim_name)
                .map(|d| d.kind)
                .ok_or_else(|| {
                    format!("internal: column references unknown dimension `{}`", dim_name)
                })?;
            let value = read_dimension_value(row, *col_idx, kind);
            dimensions.insert(dim_name.clone(), value);
        }

        // Also populate the IMPLICIT name/path/library_name fields. These are NOT
        // in dimensions HashMap — they're top-level on LensRow. The lens can
        // declare them as columns separately (e.g., `columns: [note.name]`) and
        // they appear in dimensions too in that case; no conflict.

        rows.push(LensRow {
            note_path,
            name,
            library_name,
            library_path,
            dimensions,
        });
    }

    // Suppress unused warning until §D uses def for diagnostics
    let _ = def;

    Ok(rows)
}

fn read_dimension_value(
    row: &rusqlite::Row<'_>,
    col_idx: usize,
    kind: super::dimensions::DimensionKind,
) -> DimensionValue {
    use super::dimensions::DimensionKind::*;
    match kind {
        Text => row
            .get::<_, Option<String>>(col_idx)
            .ok()
            .flatten()
            .map(DimensionValue::Text)
            .unwrap_or(DimensionValue::Null),
        Number => row
            .get::<_, Option<f64>>(col_idx)
            .ok()
            .flatten()
            .map(DimensionValue::Number)
            .unwrap_or(DimensionValue::Null),
        Timestamp => row
            .get::<_, Option<i64>>(col_idx)
            .ok()
            .flatten()
            .map(DimensionValue::Timestamp)
            .unwrap_or(DimensionValue::Null),
        Bool => row
            .get::<_, Option<bool>>(col_idx)
            .ok()
            .flatten()
            .map(DimensionValue::Bool)
            .unwrap_or(DimensionValue::Null),
        List => row
            .get::<_, Option<String>>(col_idx)
            .ok()
            .flatten()
            .map(DimensionValue::Text) // v1 — lists stored as JSON; future renders properly
            .unwrap_or(DimensionValue::Null),
    }
}

// ─── MIG-056 §E — Federation helpers ───

/// Check whether the federation context has attached cUniverses
/// AND is ready. False when:
/// - Boot hasn't reached the background-attach stage yet
/// - No cUniverses are linked
/// - `attach_all` failed (all cUniverses landed in warnings)
fn federation_has_attached(app: &tauri::AppHandle) -> bool {
    let state = app.state::<crate::search::SearchState>();
    let guard = state.federation.lock();
    match guard {
        Ok(g) => g.is_ready() && !g.attached().is_empty(),
        Err(_) => false,
    }
}

/// Snapshot of the federation's attached schema aliases.
/// Returns owned `String`s so the caller doesn't hold the Mutex.
fn federation_attached_aliases(app: &tauri::AppHandle) -> Vec<String> {
    let state = app.state::<crate::search::SearchState>();
    let guard = state.federation.lock();
    match guard {
        Ok(g) => g.attached().iter().map(|(alias, _)| alias.clone()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Execute a federated query against `SearchState.federated_conn`.
/// This connection has `main` + each cUniverse alias attached (from
/// the §B/§B.1 background-thread attach). If the connection is None
/// (federation not yet ready or attach failed), returns Err and the
/// caller falls back to the single-schema path.
fn execute_federated_query(
    app: &tauri::AppHandle,
    built: &BuiltQuery,
    def: &LensDefinition,
    lib_path_map: &HashMap<String, String>,
) -> Result<Vec<LensRow>, String> {
    let state = app.state::<crate::search::SearchState>();
    let guard = state.federated_conn.lock();
    match guard {
        Ok(g) => match g.as_ref() {
            Some(conn) => execute_query(conn, built, def, lib_path_map),
            None => Err(
                "federation: federated_conn is None (background-attach not complete)".to_string(),
            ),
        },
        Err(_) => Err("federation: federated_conn Mutex poisoned".to_string()),
    }
}

// ─── MIG-065 §E — frontmatter-key discovery (for the add-column picker) ───

/// Enumerate the distinct frontmatter keys present across the active universe
/// (+ federated cUniverses). Feeds the unified Base's "+ Add column" picker
/// "Your fields" tier. Cheap: one `json_each` pass over `note_meta`.
#[tauri::command]
pub fn discover_base_properties(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    // Federated path: union keys across main + each attached cUniverse schema.
    // Falls back to single-schema on any federation hiccup (mirrors execute_lens).
    if federation_has_attached(&app) {
        let mut schemas: Vec<String> = vec!["main".to_string()];
        schemas.extend(federation_attached_aliases(&app));
        if let Ok(keys) = discover_keys_federated(&app, &schemas) {
            return Ok(keys);
        }
        // any hiccup → fall through to single-schema
    }
    let db_path = crate::search::db_path(&app)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open search DB: {}", e))?;
    discover_keys(&conn, &["main"])
}

/// MIG-065 §G — persist a new column list to a standalone `.base` file.
///
/// The add-column picker / remove-column gesture computes the new ordered
/// column list (each entry a registered dimension name like `note.created_at`
/// OR a `prop.<key>` frontmatter reference). This round-trips the file through
/// `LensDefinition` — preserving `scope` / `where` / `order` / `view` — and
/// rewrites only `columns:`. Returns the re-serialized YAML so the caller
/// (`BaseTab.svelte`) re-renders without a second read.
///
/// Security: only `.base` files inside the active universe or a registered
/// library (reuses `bases::validate_base_path`). Rejects an empty column list
/// (the validator requires ≥1) and any column that doesn't resolve, so the
/// file on disk stays valid + queryable.
#[tauri::command]
pub fn update_base_columns(
    app: tauri::AppHandle,
    file_path: String,
    columns: Vec<String>,
) -> Result<String, String> {
    crate::bases::validate_base_path(&app, &file_path)?;
    if !file_path.ends_with(".base") {
        return Err("Not a .base file.".to_string());
    }
    if columns.is_empty() {
        return Err("A base needs at least one column.".to_string());
    }
    for c in &columns {
        if resolve_dim(c).is_none() {
            return Err(format!("Unknown column dimension: {}", c));
        }
    }

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read base file: {}", e))?;
    let mut def = parse_lens_yaml(&content).map_err(|e| e.to_string())?;
    def.columns = columns
        .into_iter()
        .map(|d| super::definition::LensColumn { dimension: d })
        .collect();
    // Re-validate the modified definition before persisting.
    validate(&def).map_err(|e| e.to_string())?;

    let yaml = serde_yaml::to_string(&def)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    std::fs::write(&file_path, &yaml)
        .map_err(|e| format!("Failed to write base file: {}", e))?;
    Ok(yaml)
}

/// MIG-065 §G.2 — persist the sort order to a standalone `.base` file. The
/// click-header / multi-sort gesture computes the new ordered `order:` list
/// (each entry a `{dimension, direction}`); this round-trips the file through
/// `LensDefinition` (preserving columns/scope/where/view) and rewrites only
/// `order:`. `validate` rejects a non-sortable dimension, so the file stays
/// valid. Returns the re-serialized YAML for an immediate re-render.
#[tauri::command]
pub fn update_base_order(
    app: tauri::AppHandle,
    file_path: String,
    order: Vec<LensSort>,
) -> Result<String, String> {
    crate::bases::validate_base_path(&app, &file_path)?;
    if !file_path.ends_with(".base") {
        return Err("Not a .base file.".to_string());
    }
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read base file: {}", e))?;
    let mut def = parse_lens_yaml(&content).map_err(|e| e.to_string())?;
    def.order = order;
    validate(&def).map_err(|e| e.to_string())?;
    let yaml = serde_yaml::to_string(&def)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    std::fs::write(&file_path, &yaml)
        .map_err(|e| format!("Failed to write base file: {}", e))?;
    Ok(yaml)
}

/// Federated key discovery against `SearchState.federated_conn`. `state` is
/// bound at function scope (not in an `if let`), so the lock guard is valid
/// throughout — same structure as `execute_federated_query`.
fn discover_keys_federated(
    app: &tauri::AppHandle,
    schemas: &[String],
) -> Result<Vec<String>, String> {
    let schema_refs: Vec<&str> = schemas.iter().map(|s| s.as_str()).collect();
    let state = app.state::<crate::search::SearchState>();
    let guard = state
        .federated_conn
        .lock()
        .map_err(|_| "federation: federated_conn Mutex poisoned".to_string())?;
    match guard.as_ref() {
        Some(conn) => discover_keys(conn, &schema_refs),
        None => Err("federation: federated_conn is None".to_string()),
    }
}

/// Distinct top-level frontmatter keys across the given schemas, sorted
/// case-insensitively. `schemas` is `["main"]` for single-universe or
/// `["main", "cu0", …]` for federated. `properties_json` is always valid JSON
/// by construction (serde-serialized), so `json_each` never errors here.
fn discover_keys(conn: &Connection, schemas: &[&str]) -> Result<Vec<String>, String> {
    let selects: Vec<String> = schemas
        .iter()
        .map(|s| {
            format!(
                "SELECT je.key AS k FROM {0}.note_meta AS nm, json_each(nm.properties_json) AS je",
                s
            )
        })
        .collect();
    let sql = format!(
        "SELECT DISTINCT k FROM ({}) ORDER BY k COLLATE NOCASE",
        selects.join(" UNION ALL ")
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("discover_base_properties prepare failed: {}", e))?;
    let mapped = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| format!("discover_base_properties query failed: {}", e))?;
    Ok(mapped.flatten().collect())
}

// ─── §C / §G — Integration tests against in-memory SQLite ───

#[cfg(test)]
mod tests {
    use super::super::definition::{
        LensColumn, LensDefinition, LensFilter, LensScope, LensSort, LensView, LibrariesSelector,
        SortDirection,
    };
    use super::*;
    use rusqlite::Connection;

    /// Create an in-memory note_meta + note_summaries schema for tests.
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

    fn recent_captures_def() -> LensDefinition {
        LensDefinition {
            schema: 1,
            lens: "Recent Captures".to_string(),
            template: Some("five-acts.observation".to_string()),
            scope: LensScope::default(),
            where_clauses: vec![LensFilter {
                dimension: "note.created_at".to_string(),
                op: "after".to_string(),
                value: "now - 14 days".to_string(),
            }],
            order: vec![LensSort {
                dimension: "note.created_at".to_string(),
                direction: SortDirection::Desc,
            }],
            columns: vec![
                LensColumn { dimension: "note.name".to_string() },
                LensColumn { dimension: "note.headline".to_string() },
            ],
            view: LensView::List,
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

    #[test]
    fn recent_captures_returns_last_14_days_only() {
        let conn = make_test_db();
        let now_ = now();
        let day = 86400;

        insert_note(&conn, "/Lib/a.md", "a", "Lib", now_ - 1 * day, Some("recent A"));
        insert_note(&conn, "/Lib/b.md", "b", "Lib", now_ - 10 * day, Some("recent B"));
        insert_note(&conn, "/Lib/c.md", "c", "Lib", now_ - 20 * day, Some("old C"));

        let def = recent_captures_def();
        let built = build_sql(&def, &["Lib".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib", "/Lib")])).unwrap();

        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
        assert!(!names.contains(&"c"), "20-day-old note should be excluded");
    }

    #[test]
    fn recent_captures_orders_desc_by_created_at() {
        let conn = make_test_db();
        let now_ = now();
        let day = 86400;

        insert_note(&conn, "/Lib/a.md", "a", "Lib", now_ - 5 * day, None);
        insert_note(&conn, "/Lib/b.md", "b", "Lib", now_ - 1 * day, None);
        insert_note(&conn, "/Lib/c.md", "c", "Lib", now_ - 3 * day, None);

        let def = recent_captures_def();
        let built = build_sql(&def, &["Lib".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib", "/Lib")])).unwrap();

        // Order: b (1d), c (3d), a (5d) — descending by created_at
        assert_eq!(rows[0].name, "b");
        assert_eq!(rows[1].name, "c");
        assert_eq!(rows[2].name, "a");
    }

    #[test]
    fn recent_captures_headline_populated() {
        let conn = make_test_db();
        let now_ = now();

        insert_note(&conn, "/Lib/a.md", "a", "Lib", now_ - 1, Some("the headline"));
        insert_note(&conn, "/Lib/b.md", "b", "Lib", now_ - 2, None);

        let def = recent_captures_def();
        let built = build_sql(&def, &["Lib".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib", "/Lib")])).unwrap();

        let a = rows.iter().find(|r| r.name == "a").unwrap();
        let b = rows.iter().find(|r| r.name == "b").unwrap();
        assert!(matches!(a.dimensions.get("note.headline"), Some(DimensionValue::Text(_))));
        assert!(matches!(b.dimensions.get("note.headline"), Some(DimensionValue::Null)));
    }

    #[test]
    fn library_subset_filter() {
        let conn = make_test_db();
        let now_ = now();

        insert_note(&conn, "/Lib1/a.md", "a", "Lib1", now_ - 1, None);
        insert_note(&conn, "/Lib2/b.md", "b", "Lib2", now_ - 1, None);

        let mut def = recent_captures_def();
        def.scope.libraries = LibrariesSelector::Subset(vec!["Lib1".to_string()]);
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib1", "/Lib1"), ("Lib2", "/Lib2")])).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "a");
    }

    #[test]
    fn empty_universe_returns_zero_rows() {
        let conn = make_test_db();
        let def = recent_captures_def();
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib1", "/Lib1")])).unwrap();
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn multilingual_note_names_round_trip() {
        let conn = make_test_db();
        let now_ = now();

        insert_note(&conn, "/Lib/أ.md", "أ", "Lib", now_ - 1, Some("ملخص"));
        insert_note(&conn, "/Lib/فيلسوف.md", "فيلسوف", "Lib", now_ - 2, Some("Arabic headline"));

        let def = recent_captures_def();
        let built = build_sql(&def, &["Lib".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib", "/Lib")])).unwrap();

        assert_eq!(rows.len(), 2);
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"أ"));
        assert!(names.contains(&"فيلسوف"));
    }

    #[test]
    fn dimensions_map_contains_declared_columns() {
        let conn = make_test_db();
        let now_ = now();
        insert_note(&conn, "/Lib/a.md", "a", "Lib", now_ - 1, Some("h"));

        let def = recent_captures_def();
        let built = build_sql(&def, &["Lib".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib", "/Lib")])).unwrap();

        let row = &rows[0];
        assert!(row.dimensions.contains_key("note.name"));
        assert!(row.dimensions.contains_key("note.headline"));
        // Implicit fields are top-level, not in dimensions
        assert!(!row.dimensions.contains_key("note.path"));
    }

    #[test]
    fn library_path_resolved_via_map() {
        let conn = make_test_db();
        insert_note(&conn, "/Lib/a.md", "a", "MyLib", now() - 1, None);

        let def = recent_captures_def();
        let built = build_sql(&def, &["MyLib".to_string()]).unwrap();
        let rows = execute_query(
            &conn,
            &built,
            &def,
            &lib_paths(&[("MyLib", "/path/to/MyLib")]),
        )
        .unwrap();

        assert_eq!(rows[0].library_path, "/path/to/MyLib");
    }

    #[test]
    fn note_at_exact_boundary_included_after() {
        // A note with created_at == (now - 14 days) — should match `after now - 14 days`
        // because the SQL is `>= ?`.
        let conn = make_test_db();
        let now_ = now();
        let day = 86400;

        insert_note(&conn, "/Lib/boundary.md", "boundary", "Lib", now_ - 14 * day, None);

        let def = recent_captures_def();
        let built = build_sql(&def, &["Lib".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib", "/Lib")])).unwrap();

        // Exact-boundary should be included (allow ±2s for current_unix_seconds rounding)
        assert!(rows.iter().any(|r| r.name == "boundary"));
    }

    #[test]
    fn dimension_index_map_offsets_correctly() {
        // Verify that the dimension_index_map starts at 3 (after the 3 implicit columns)
        // and increments by 1 per column.
        let def = LensDefinition {
            schema: 1,
            lens: "Test".to_string(),
            template: None,
            scope: LensScope::default(),
            where_clauses: vec![],
            order: vec![],
            columns: vec![
                LensColumn { dimension: "note.name".to_string() },
                LensColumn { dimension: "note.created_at".to_string() },
                LensColumn { dimension: "note.headline".to_string() },
            ],
            view: LensView::List,
        };
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert_eq!(built.dimension_index_map.get(&3), Some(&"note.name".to_string()));
        assert_eq!(built.dimension_index_map.get(&4), Some(&"note.created_at".to_string()));
        assert_eq!(built.dimension_index_map.get(&5), Some(&"note.headline".to_string()));
    }

    // ─── MIG-065 §E — prop.* materialization + key discovery ───

    #[test]
    fn prop_column_materializes_through_execute_query() {
        let conn = make_test_db();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, created_at, properties_json) VALUES (?,?,?,?,?,?)",
            rusqlite::params![
                "/Lib/a.md", "a", "Lib", now(), now(),
                r#"{"status":"done","author":"Eisa"}"#
            ],
        ).unwrap();

        let def = LensDefinition {
            schema: 1,
            lens: "T".to_string(),
            template: None,
            scope: LensScope::default(),
            where_clauses: vec![],
            order: vec![],
            columns: vec![
                LensColumn { dimension: "note.name".to_string() },
                LensColumn { dimension: "prop.status".to_string() },
            ],
            view: LensView::Table,
        };
        let built = build_sql(&def, &["Lib".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib", "/Lib")])).unwrap();
        assert_eq!(rows.len(), 1);
        match rows[0].dimensions.get("prop.status") {
            Some(DimensionValue::Text(s)) => assert_eq!(s, "done"),
            other => panic!("expected Text(\"done\"), got {:?}", other),
        }
    }

    #[test]
    fn prop_contains_filter_through_execute_query() {
        let conn = make_test_db();
        for (p, st) in [("/a.md", "in-progress"), ("/b.md", "done"), ("/c.md", "blocked")] {
            conn.execute(
                "INSERT INTO note_meta (path, name, library_name, modified, created_at, properties_json) VALUES (?,?,?,?,?,?)",
                rusqlite::params![p, p, "Lib", now(), now(), format!(r#"{{"status":"{}"}}"#, st)],
            ).unwrap();
        }
        let def = LensDefinition {
            schema: 1,
            lens: "T".to_string(),
            template: None,
            scope: LensScope::default(),
            where_clauses: vec![LensFilter {
                dimension: "prop.status".to_string(),
                op: "contains".to_string(),
                value: "progress".to_string(),
            }],
            order: vec![],
            columns: vec![LensColumn { dimension: "note.name".to_string() }],
            view: LensView::Table,
        };
        let built = build_sql(&def, &["Lib".to_string()]).unwrap();
        let rows = execute_query(&conn, &built, &def, &lib_paths(&[("Lib", "/Lib")])).unwrap();
        assert_eq!(rows.len(), 1, "only the in-progress note should match `contains progress`");
        assert_eq!(rows[0].name, "/a.md");
    }

    #[test]
    fn discover_keys_returns_distinct_sorted() {
        let conn = make_test_db();
        conn.execute(
            "INSERT INTO note_meta (path,name,library_name,modified,created_at,properties_json) VALUES (?,?,?,?,?,?)",
            rusqlite::params!["/a.md", "a", "L", 1, 1, r#"{"status":"x","author":"y"}"#],
        ).unwrap();
        conn.execute(
            "INSERT INTO note_meta (path,name,library_name,modified,created_at,properties_json) VALUES (?,?,?,?,?,?)",
            rusqlite::params!["/b.md", "b", "L", 1, 1, r#"{"status":"z","priority":"1"}"#],
        ).unwrap();
        conn.execute(
            "INSERT INTO note_meta (path,name,library_name,modified,created_at,properties_json) VALUES (?,?,?,?,?,?)",
            rusqlite::params!["/c.md", "c", "L", 1, 1, "{}"],
        ).unwrap();

        let keys = discover_keys(&conn, &["main"]).unwrap();
        assert!(keys.contains(&"status".to_string()));
        assert!(keys.contains(&"author".to_string()));
        assert!(keys.contains(&"priority".to_string()));
        // DISTINCT: `status` (in two notes) appears once.
        assert_eq!(keys.iter().filter(|k| k.as_str() == "status").count(), 1);
    }
}
