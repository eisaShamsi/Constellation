//! MIG-055 §C — SQL builder.
//!
//! Translates a validated `LensDefinition` + the resolved federated
//! library set into a SQL string + parameter list ready to execute
//! against the search DB.
//!
//! Per Architect §5.1: queries `note_meta` (+ JOINs added per
//! dimensions used by the lens) — never `properties_json` (which has
//! pre-existing parser bugs irrelevant to v1's curated dimensions).

use super::definition::{LensDefinition, LensFilter, LensSort, SortDirection};
use super::dimensions::{lookup_dimension, DimensionKind};
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
        let dim = lookup_dimension(&col.dimension)
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

/// Build the SQL clause for one filter. Returns (clause_sql, params, joins_needed).
fn build_filter_clause(
    filter: &LensFilter,
) -> Result<(String, Vec<rusqlite::types::Value>, Vec<&'static str>), String> {
    let dim = lookup_dimension(&filter.dimension)
        .ok_or_else(|| format!("internal: unknown dimension `{}` in where (should have been caught by validator)", filter.dimension))?;

    // For v1, all filterable dimensions are Timestamp (only `note.created_at`).
    // Future phases extend to Text / Number / Enum / Bool filtering.
    match dim.kind {
        DimensionKind::Timestamp => build_timestamp_filter(dim, filter),
        other => Err(format!(
            "internal: dimension `{}` has kind {:?} which is not filterable in v1",
            filter.dimension, other
        )),
    }
}

fn build_timestamp_filter(
    dim: &super::dimensions::DimensionDef,
    filter: &LensFilter,
) -> Result<(String, Vec<rusqlite::types::Value>, Vec<&'static str>), String> {
    let joins: Vec<&'static str> = dim.requires_join.into_iter().collect();
    let expr = dim.sql_expression;
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

fn build_order_clause(sorts: &[LensSort]) -> Result<Option<String>, String> {
    if sorts.is_empty() {
        return Ok(None);
    }
    let parts: Result<Vec<String>, String> = sorts
        .iter()
        .map(|sort| {
            let dim = lookup_dimension(&sort.dimension).ok_or_else(|| {
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
}
