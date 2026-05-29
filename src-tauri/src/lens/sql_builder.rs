//! MIG-055 §C — SQL builder.
//!
//! Translates a validated `LensDefinition` + the resolved federated
//! library set into a SQL string + parameter list ready to execute
//! against the search DB.
//!
//! Per Architect §5.1: queries `note_meta` (+ JOINs added per dimensions
//! used by the lens). MIG-065 §D: also reads `note_meta.properties_json`
//! via `json_extract` for raw frontmatter `prop.<key>` columns — the
//! unified Base's familiar table. (Scalar frontmatter is faithful per the
//! §B characterization; list/nested fidelity is a deferred parser upgrade.)

use super::definition::{LensDefinition, LensFilter, LensSort, SortDirection};
use super::dimensions::{resolve_dim, DimensionKind, ResolvedDim};
use std::collections::{HashSet, BTreeMap};
use std::time::{SystemTime, UNIX_EPOCH};

/// Output of `build_sql`: a SQL string + parameter list +
/// the column index map (for the materializer in `query.rs`).
pub struct BuiltQuery {
    /// The parameterized SQL string.
    pub sql: String,
    /// Parameters to bind in the same order as `?` placeholders in `sql`.
    pub params: Vec<rusqlite::types::Value>,
    /// Map: SELECT column index → dimension name (in lens declaration order).
    /// Used by the materializer to populate `LensRow.dimensions`.
    /// The implicit columns (path, name, library_name) occupy indices 0, 1, 2.
    /// Lens-declared columns start at index 3.
    pub dimension_index_map: BTreeMap<usize, String>,
}

/// Build a parameterized SQL query for a validated lens.
///
/// `allowed_libraries` is the resolved set of library names this lens
/// is allowed to see (already filtered by `scope.libraries` + federation
/// at the call site).
pub fn build_sql(
    def: &LensDefinition,
    allowed_libraries: &[String],
) -> Result<BuiltQuery, String> {
    let mut select_parts: Vec<String> = Vec::new();
    let mut joins: HashSet<&'static str> = HashSet::new();
    let mut where_parts: Vec<String> = Vec::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();
    let mut dimension_index_map: BTreeMap<usize, String> = BTreeMap::new();

    // 1. Implicit SELECT columns (always present — populated into LensRow's
    //    top-level fields, not into dimensions HashMap).
    //
    //    index 0: note_meta.path        → LensRow.note_path
    //    index 1: note_meta.name        → LensRow.name (also dimension `note.name` if requested)
    //    index 2: note_meta.library_name → LensRow.library_name
    select_parts.push("note_meta.path".to_string());
    select_parts.push("note_meta.name".to_string());
    select_parts.push("note_meta.library_name".to_string());

    // 2. Lens-declared columns. Each adds:
    //    - One entry to select_parts (SQL expression)
    //    - Its required JOIN (if any) to the joins set
    //    - A mapping into dimension_index_map for the materializer
    let mut col_index = 3;
    for col in &def.columns {
        let dim = resolve_dim(&col.dimension)
            .ok_or_else(|| format!("internal: unknown dimension `{}` in columns (should have been caught by validator)", col.dimension))?;
        select_parts.push(dim.sql_expression.to_string());
        if let Some(join) = dim.requires_join {
            joins.insert(join);
        }
        dimension_index_map.insert(col_index, col.dimension.clone());
        col_index += 1;
    }

    // 3. Library scope. If allowed_libraries is empty, force 1=0 (empty result).
    if allowed_libraries.is_empty() {
        where_parts.push("1=0".to_string());
    } else {
        let placeholders: Vec<&str> = (0..allowed_libraries.len()).map(|_| "?").collect();
        where_parts.push(format!(
            "note_meta.library_name IN ({})",
            placeholders.join(", ")
        ));
        for lib in allowed_libraries {
            params.push(rusqlite::types::Value::Text(lib.clone()));
        }
    }

    // 4. Filters from `where:` — each adds a clause + binds parameter(s) +
    //    pulls in any JOIN the dimension needs.
    for filter in &def.where_clauses {
        let (clause, mut filter_params, filter_joins) = build_filter_clause(filter)?;
        where_parts.push(clause);
        params.append(&mut filter_params);
        for j in filter_joins {
            joins.insert(j);
        }
    }

    // 5. ORDER BY from `order:`.
    let order_sql = build_order_clause(&def.order)?;

    // 6. Assemble.
    let mut sql = format!("SELECT {} FROM note_meta", select_parts.join(", "));
    for join in &joins {
        sql.push(' ');
        sql.push_str(join);
    }
    if !where_parts.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_parts.join(" AND "));
    }
    if let Some(order_sql) = order_sql {
        sql.push(' ');
        sql.push_str(&order_sql);
    }

    Ok(BuiltQuery {
        sql,
        params,
        dimension_index_map,
    })
}

/// MIG-056 §E — Build a federated UNION ALL query across the active
/// universe (schema alias `main`) + each attached cUniverse (`cu0`…).
///
/// Each per-schema SELECT pushes its WHERE clauses down per
/// Architect §7.2 (predicate-pushdown contract / Agent 3's Citus
/// lesson). The outer ORDER BY references a sort column by ORDINAL
/// POSITION (sidesteps SQL alias-name resolution across UNION ALL
/// branches).
///
/// `federated_schemas` MUST include `"main"` as the first entry +
/// every cUniverse alias from `FederationContext.attached`. Empty
/// or single-entry lists fall back to the single-schema `build_sql`.
pub fn build_federated_sql(
    def: &LensDefinition,
    allowed_libraries: &[String],
    federated_schemas: &[&str],
) -> Result<BuiltQuery, String> {
    // Degenerate cases — defer to single-schema builder so we keep
    // one source of truth for the simple path.
    if federated_schemas.len() <= 1 {
        return build_sql(def, allowed_libraries);
    }

    let mut all_parts: Vec<String> = Vec::new();
    let mut all_params: Vec<rusqlite::types::Value> = Vec::new();
    let mut dimension_index_map: BTreeMap<usize, String> = BTreeMap::new();

    // Build the dimension_index_map (same across all schemas).
    // Implicit cols at 0/1/2 (path/name/library_name); declared cols at 3+.
    let mut col_index = 3;
    for col in &def.columns {
        dimension_index_map.insert(col_index, col.dimension.clone());
        col_index += 1;
    }

    // ORDER BY columns are appended to the SELECT list after declared
    // columns (so the outer ORDER BY can reference them by ordinal).
    // Same shape across all branches.
    let order_dims: Vec<&str> = def.order.iter().map(|s| s.dimension.as_str()).collect();
    let order_start_index = col_index;

    for schema in federated_schemas {
        let (part_sql, mut part_params) =
            build_per_schema_body(def, allowed_libraries, schema, &order_dims)?;
        all_parts.push(part_sql);
        all_params.append(&mut part_params);
    }

    let mut sql = all_parts.join(" UNION ALL ");

    // Outer ORDER BY using ordinal positions of the appended sort cols.
    // 1-based in SQLite. The first appended sort col is at position
    // `order_start_index + 1` (1-based from 0-based index).
    if !def.order.is_empty() {
        let mut order_parts: Vec<String> = Vec::new();
        for (i, sort) in def.order.iter().enumerate() {
            let dim = resolve_dim(&sort.dimension).ok_or_else(|| {
                format!(
                    "internal: unknown dimension `{}` in order (should have been caught by validator)",
                    sort.dimension
                )
            })?;
            let dir = match sort.direction {
                SortDirection::Asc => "ASC",
                SortDirection::Desc => "DESC",
            };
            let collate = match dim.kind {
                DimensionKind::Text => " COLLATE NOCASE",
                _ => "",
            };
            // 1-based ordinal: implicit cols (3) + declared cols + i
            let ordinal = order_start_index + i + 1;
            order_parts.push(format!("{}{} {}", ordinal, collate, dir));
        }
        sql.push_str(" ORDER BY ");
        sql.push_str(&order_parts.join(", "));
    }

    Ok(BuiltQuery {
        sql,
        params: all_params,
        dimension_index_map,
    })
}

/// Build one per-schema SELECT body for the federated UNION ALL.
/// All table refs are schema-qualified. WHERE clauses are inlined
/// (predicate-pushdown). Sort columns are appended to the SELECT list
/// so the outer ORDER BY can reference them by ordinal.
fn build_per_schema_body(
    def: &LensDefinition,
    allowed_libraries: &[String],
    schema: &str,
    order_dims: &[&str],
) -> Result<(String, Vec<rusqlite::types::Value>), String> {
    let mut select_parts: Vec<String> = Vec::new();
    let mut joins: HashSet<String> = HashSet::new();
    let mut where_parts: Vec<String> = Vec::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    // Implicit cols (schema-qualified)
    select_parts.push(format!("{}.note_meta.path", schema));
    select_parts.push(format!("{}.note_meta.name", schema));
    select_parts.push(format!("{}.note_meta.library_name", schema));

    // Declared lens columns (schema-qualified by substituting the static
    // dimension expression's table prefix).
    for col in &def.columns {
        let dim = resolve_dim(&col.dimension).ok_or_else(|| {
            format!(
                "internal: unknown dimension `{}` in columns (should have been caught by validator)",
                col.dimension
            )
        })?;
        select_parts.push(qualify_expr(&dim.sql_expression, schema));
        if let Some(join) = dim.requires_join {
            joins.insert(qualify_join(join, schema));
        }
    }

    // Append sort columns (for outer ORDER BY).
    for dim_name in order_dims {
        let dim = resolve_dim(dim_name).ok_or_else(|| {
            format!(
                "internal: unknown dimension `{}` in order (should have been caught by validator)",
                dim_name
            )
        })?;
        select_parts.push(qualify_expr(&dim.sql_expression, schema));
        if let Some(join) = dim.requires_join {
            joins.insert(qualify_join(join, schema));
        }
    }

    // Library scope.
    if allowed_libraries.is_empty() {
        where_parts.push("1=0".to_string());
    } else {
        let placeholders: Vec<&str> = (0..allowed_libraries.len()).map(|_| "?").collect();
        where_parts.push(format!(
            "{}.note_meta.library_name IN ({})",
            schema,
            placeholders.join(", ")
        ));
        for lib in allowed_libraries {
            params.push(rusqlite::types::Value::Text(lib.clone()));
        }
    }

    // Filters (predicate-pushdown: each branch's WHERE includes them).
    for filter in &def.where_clauses {
        let (clause, mut filter_params, filter_joins) = build_filter_clause(filter)?;
        // Schema-qualify the filter's column reference.
        where_parts.push(qualify_expr(&clause, schema));
        params.append(&mut filter_params);
        for j in filter_joins {
            joins.insert(qualify_join(j, schema));
        }
    }

    // Assemble per-schema body (NO ORDER BY — applied at outer level).
    let mut sql = format!(
        "SELECT {} FROM {}.note_meta",
        select_parts.join(", "),
        schema
    );
    for join in &joins {
        sql.push(' ');
        sql.push_str(join);
    }
    if !where_parts.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&where_parts.join(" AND "));
    }

    Ok((sql, params))
}

/// Substitute unqualified table refs (`note_meta`, `note_summaries`)
/// with the given schema prefix. Safe because:
/// - Constellation's dimension `sql_expression` fields use only these
///   two table names (verified by `dimensions.rs`'s REGISTRY)
/// - Neither name appears as a substring in column names or string
///   literals in these expressions
///
/// Future dimensions that introduce new table refs must extend this
/// list (and a test in `dimensions.rs` should pin the table-name set).
fn qualify_expr(expr: &str, schema: &str) -> String {
    expr.replace("note_meta", &format!("{}.note_meta", schema))
        .replace("note_summaries", &format!("{}.note_summaries", schema))
}

fn qualify_join(join: &str, schema: &str) -> String {
    qualify_expr(join, schema)
}

/// Build the SQL clause for one filter. Returns (clause_sql, params, joins_needed).
fn build_filter_clause(
    filter: &LensFilter,
) -> Result<(String, Vec<rusqlite::types::Value>, Vec<&'static str>), String> {
    let dim = resolve_dim(&filter.dimension)
        .ok_or_else(|| format!("internal: unknown dimension `{}` in where (should have been caught by validator)", filter.dimension))?;

    // For v1, all filterable dimensions are Timestamp (only `note.created_at`).
    // Future phases extend to Text / Number / Enum / Bool filtering.
    match dim.kind {
        DimensionKind::Timestamp => build_timestamp_filter(&dim, filter),
        // MIG-065 §D — raw frontmatter `prop.<key>` columns are Text.
        DimensionKind::Text => build_text_filter(&dim, filter),
        other => Err(format!(
            "internal: dimension `{}` has kind {:?} which is not filterable in v1",
            filter.dimension, other
        )),
    }
}

fn build_timestamp_filter(
    dim: &ResolvedDim,
    filter: &LensFilter,
) -> Result<(String, Vec<rusqlite::types::Value>, Vec<&'static str>), String> {
    let joins: Vec<&'static str> = dim.requires_join.into_iter().collect();
    let expr = dim.sql_expression.as_str();
    match filter.op.as_str() {
        "after" => {
            let ts = parse_time_value(&filter.value)?;
            Ok((
                format!("{} >= ?", expr),
                vec![rusqlite::types::Value::Integer(ts)],
                joins,
            ))
        }
        "before" => {
            let ts = parse_time_value(&filter.value)?;
            Ok((
                format!("{} <= ?", expr),
                vec![rusqlite::types::Value::Integer(ts)],
                joins,
            ))
        }
        "between" => {
            // Value format: "<start> .. <end>"
            let parts: Vec<&str> = filter.value.split("..").collect();
            if parts.len() != 2 {
                return Err(format!(
                    "filter `between` expects value `<start> .. <end>`, got `{}`",
                    filter.value
                ));
            }
            let start_ts = parse_time_value(parts[0].trim())?;
            let end_ts = parse_time_value(parts[1].trim())?;
            Ok((
                format!("{} BETWEEN ? AND ?", expr),
                vec![
                    rusqlite::types::Value::Integer(start_ts),
                    rusqlite::types::Value::Integer(end_ts),
                ],
                joins,
            ))
        }
        "within" => {
            // "within 7 days" → after (now - 7 days). Value format: "<N> <unit>".
            let synthetic = format!("now - {}", filter.value.trim());
            let ts = parse_time_value(&synthetic)?;
            Ok((
                format!("{} >= ?", expr),
                vec![rusqlite::types::Value::Integer(ts)],
                joins,
            ))
        }
        other => Err(format!("unsupported timestamp filter op: {}", other)),
    }
}

/// MIG-065 §D — Text-column filters for raw frontmatter `prop.<key>` columns
/// (the familiar Obsidian/Notion operator set). `is_empty` / `is_not_empty`
/// bind no parameter; the others bind the filter value once.
fn build_text_filter(
    dim: &ResolvedDim,
    filter: &LensFilter,
) -> Result<(String, Vec<rusqlite::types::Value>, Vec<&'static str>), String> {
    let joins: Vec<&'static str> = dim.requires_join.into_iter().collect();
    let expr = dim.sql_expression.as_str();
    let val = || rusqlite::types::Value::Text(filter.value.clone());
    match filter.op.as_str() {
        "is" => Ok((format!("{} = ?", expr), vec![val()], joins)),
        "is_not" => Ok((
            format!("({0} IS NULL OR {0} != ?)", expr),
            vec![val()],
            joins,
        )),
        "contains" => Ok((
            format!("{} LIKE '%' || ? || '%'", expr),
            vec![val()],
            joins,
        )),
        "does_not_contain" => Ok((
            format!("({0} IS NULL OR {0} NOT LIKE '%' || ? || '%')", expr),
            vec![val()],
            joins,
        )),
        "is_empty" => Ok((format!("({0} IS NULL OR {0} = '')", expr), vec![], joins)),
        "is_not_empty" => Ok((
            format!("({0} IS NOT NULL AND {0} != '')", expr),
            vec![],
            joins,
        )),
        other => Err(format!("unsupported text filter op: {}", other)),
    }
}

fn build_order_clause(sorts: &[LensSort]) -> Result<Option<String>, String> {
    if sorts.is_empty() {
        return Ok(None);
    }
    let parts: Result<Vec<String>, String> = sorts
        .iter()
        .map(|sort| {
            let dim = resolve_dim(&sort.dimension).ok_or_else(|| {
                format!(
                    "internal: unknown dimension `{}` in order (should have been caught by validator)",
                    sort.dimension
                )
            })?;
            let dir = match sort.direction {
                SortDirection::Asc => "ASC",
                SortDirection::Desc => "DESC",
            };
            // For Text dimensions, COLLATE NOCASE so casing is ignored.
            // For Timestamp, raw numeric sort.
            let collate = match dim.kind {
                DimensionKind::Text => " COLLATE NOCASE",
                _ => "",
            };
            Ok(format!("{}{} {}", dim.sql_expression, collate, dir))
        })
        .collect();
    let parts = parts?;
    Ok(Some(format!("ORDER BY {}", parts.join(", "))))
}

/// Parse a time-value string into a Unix-second timestamp.
///
/// Accepts:
/// - `"now"` → current time
/// - `"now - <N> <unit>"` / `"now + <N> <unit>"` → relative; units:
///   second / seconds / minute / minutes / hour / hours /
///   day / days / week / weeks
/// - RFC 3339 / ISO 8601 timestamp (e.g., `"2026-01-01T00:00:00Z"`)
pub(crate) fn parse_time_value(s: &str) -> Result<i64, String> {
    let s = s.trim();

    if s == "now" {
        return Ok(current_unix_seconds());
    }

    if let Some(rest) = s.strip_prefix("now") {
        let rest = rest.trim();
        let (sign, n_and_unit) = if let Some(r) = rest.strip_prefix('-') {
            (-1i64, r.trim())
        } else if let Some(r) = rest.strip_prefix('+') {
            (1i64, r.trim())
        } else {
            return Err(format!("malformed time value `{}` (expected `now + N units` / `now - N units`)", s));
        };
        let parts: Vec<&str> = n_and_unit.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(format!("malformed time value `{}`", s));
        }
        let n: i64 = parts[0]
            .parse()
            .map_err(|_| format!("not a number in time value: `{}`", parts[0]))?;
        let unit_secs: i64 = match parts[1] {
            "second" | "seconds" => 1,
            "minute" | "minutes" => 60,
            "hour" | "hours" => 3600,
            "day" | "days" => 86400,
            "week" | "weeks" => 604800,
            other => return Err(format!("unknown time unit: `{}`", other)),
        };
        return Ok(current_unix_seconds() + sign * n * unit_secs);
    }

    // RFC 3339 / ISO 8601
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp());
    }

    Err(format!("could not parse time value: `{}`", s))
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ─── §C sql_builder unit tests ───

#[cfg(test)]
mod tests {
    use super::super::definition::{
        LensColumn, LensDefinition, LensFilter, LensScope, LensSort, LensView, SortDirection,
    };
    use super::*;

    fn base_def() -> LensDefinition {
        LensDefinition {
            schema: 1,
            lens: "Test".to_string(),
            template: None,
            scope: LensScope::default(),
            where_clauses: vec![],
            order: vec![],
            columns: vec![LensColumn { dimension: "note.name".to_string() }],
            view: LensView::List,
        }
    }

    #[test]
    fn build_sql_minimal() {
        let def = base_def();
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.starts_with("SELECT"));
        assert!(built.sql.contains("note_meta.path"));
        assert!(built.sql.contains("note_meta.name"));
        assert!(built.sql.contains("note_meta.library_name"));
        assert!(built.sql.contains("FROM note_meta"));
        assert!(built.sql.contains("note_meta.library_name IN (?)"));
        assert_eq!(built.params.len(), 1);
        // dimension_index_map should have note.name at index 3 (after the 3 implicit columns)
        assert_eq!(built.dimension_index_map.get(&3), Some(&"note.name".to_string()));
    }

    #[test]
    fn build_sql_with_headline_includes_join() {
        let mut def = base_def();
        def.columns.push(LensColumn { dimension: "note.headline".to_string() });
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("LEFT JOIN note_summaries"));
        // headline should be at index 4
        assert_eq!(built.dimension_index_map.get(&4), Some(&"note.headline".to_string()));
    }

    #[test]
    fn build_sql_empty_libraries_returns_1_eq_0() {
        let def = base_def();
        let built = build_sql(&def, &[]).unwrap();
        assert!(built.sql.contains("1=0"));
        assert_eq!(built.params.len(), 0);
    }

    #[test]
    fn build_sql_multiple_libraries() {
        let def = base_def();
        let built = build_sql(&def, &["A".to_string(), "B".to_string(), "C".to_string()]).unwrap();
        assert!(built.sql.contains("IN (?, ?, ?)"));
        assert_eq!(built.params.len(), 3);
    }

    #[test]
    fn build_sql_after_filter() {
        let mut def = base_def();
        def.where_clauses = vec![LensFilter {
            dimension: "note.created_at".to_string(),
            op: "after".to_string(),
            value: "now - 14 days".to_string(),
        }];
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("note_meta.created_at >= ?"));
        // 2 params: 1 library + 1 timestamp
        assert_eq!(built.params.len(), 2);
    }

    #[test]
    fn build_sql_order_clause_desc() {
        let mut def = base_def();
        def.order = vec![LensSort {
            dimension: "note.created_at".to_string(),
            direction: SortDirection::Desc,
        }];
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("ORDER BY note_meta.created_at DESC"));
    }

    #[test]
    fn build_sql_sort_text_uses_collate_nocase() {
        let mut def = base_def();
        def.order = vec![LensSort {
            dimension: "note.name".to_string(),
            direction: SortDirection::Asc,
        }];
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("note_meta.name COLLATE NOCASE ASC"));
    }

    #[test]
    fn parse_time_value_now() {
        let now = parse_time_value("now").unwrap();
        let real_now = current_unix_seconds();
        assert!((now - real_now).abs() <= 1);
    }

    #[test]
    fn parse_time_value_relative_minus_14_days() {
        let then = parse_time_value("now - 14 days").unwrap();
        let now = current_unix_seconds();
        let expected = now - 14 * 86400;
        assert!((then - expected).abs() <= 2);
    }

    #[test]
    fn parse_time_value_relative_units() {
        let now = current_unix_seconds();
        assert!((parse_time_value("now - 1 minute").unwrap() - (now - 60)).abs() <= 2);
        assert!((parse_time_value("now - 1 hour").unwrap() - (now - 3600)).abs() <= 2);
        assert!((parse_time_value("now - 1 day").unwrap() - (now - 86400)).abs() <= 2);
        assert!((parse_time_value("now - 1 week").unwrap() - (now - 604800)).abs() <= 2);
    }

    #[test]
    fn parse_time_value_iso_8601() {
        // 2026-01-01T00:00:00Z = 1767225600
        let ts = parse_time_value("2026-01-01T00:00:00Z").unwrap();
        assert_eq!(ts, 1767225600);
    }

    #[test]
    fn parse_time_value_unknown_unit_rejected() {
        assert!(parse_time_value("now - 1 fortnight").is_err());
    }

    #[test]
    fn parse_time_value_malformed_rejected() {
        assert!(parse_time_value("yesterday").is_err());
        assert!(parse_time_value("now -").is_err());
        assert!(parse_time_value("").is_err());
    }

    #[test]
    fn build_sql_between_filter() {
        let mut def = base_def();
        def.where_clauses = vec![LensFilter {
            dimension: "note.created_at".to_string(),
            op: "between".to_string(),
            value: "now - 30 days .. now".to_string(),
        }];
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("BETWEEN ? AND ?"));
        // 3 params: 1 library + 2 timestamps
        assert_eq!(built.params.len(), 3);
    }

    #[test]
    fn build_sql_within_filter() {
        let mut def = base_def();
        def.where_clauses = vec![LensFilter {
            dimension: "note.created_at".to_string(),
            op: "within".to_string(),
            value: "7 days".to_string(),
        }];
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("note_meta.created_at >= ?"));
        assert_eq!(built.params.len(), 2);
    }

    // ─── MIG-065 §D — raw frontmatter `prop.<key>` columns ───

    #[test]
    fn build_sql_property_column_uses_json_extract() {
        let mut def = base_def();
        def.columns.push(LensColumn { dimension: "prop.status".to_string() });
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("json_extract(note_meta.properties_json"));
        assert!(built.sql.contains("\"status\""));
        // declared after note.name (index 3) → prop.status at index 4
        assert_eq!(built.dimension_index_map.get(&4), Some(&"prop.status".to_string()));
    }

    #[test]
    fn build_sql_property_contains_filter() {
        let mut def = base_def();
        def.where_clauses = vec![LensFilter {
            dimension: "prop.status".to_string(),
            op: "contains".to_string(),
            value: "done".to_string(),
        }];
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("LIKE '%' || ? || '%'"));
        assert_eq!(built.params.len(), 2); // 1 library + 1 value
    }

    #[test]
    fn build_sql_property_is_empty_binds_no_value() {
        let mut def = base_def();
        def.where_clauses = vec![LensFilter {
            dimension: "prop.status".to_string(),
            op: "is_empty".to_string(),
            value: String::new(),
        }];
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("IS NULL OR"));
        assert_eq!(built.params.len(), 1); // only the library param
    }

    #[test]
    fn build_sql_property_sort_uses_collate_nocase() {
        let mut def = base_def();
        def.order = vec![LensSort {
            dimension: "prop.status".to_string(),
            direction: SortDirection::Asc,
        }];
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.contains("COLLATE NOCASE ASC"));
        assert!(built.sql.contains("json_extract(note_meta.properties_json"));
    }

    #[test]
    fn build_federated_sql_property_column_qualified_per_schema() {
        let mut def = base_def();
        def.columns.push(LensColumn { dimension: "prop.status".to_string() });
        let built =
            build_federated_sql(&def, &["Lib1".to_string()], &["main", "cu0"]).unwrap();
        assert!(built.sql.contains("UNION ALL"));
        assert!(built.sql.contains("json_extract(main.note_meta.properties_json"));
        assert!(built.sql.contains("json_extract(cu0.note_meta.properties_json"));
    }

    #[test]
    fn build_sql_table_view_parses_like_list() {
        // view: table returns the same SQL shape as list (rendering differs only
        // on the frontend). This pins that the engine is view-agnostic.
        let mut def = base_def();
        def.view = LensView::Table;
        let built = build_sql(&def, &["Lib1".to_string()]).unwrap();
        assert!(built.sql.starts_with("SELECT"));
        assert!(built.sql.contains("FROM note_meta"));
    }
}
