//! MIG-055 §B — Lens schema validator.
//!
//! Validates a parsed `LensDefinition` against the §A dimension registry.
//! Catches semantic errors (unknown dimensions, unsupported filter ops,
//! schema version mismatches, empty columns).
//!
//! Parse-time errors (malformed YAML / missing required fields) surface
//! from `parser::parse_lens_yaml` BEFORE validation runs. The validator
//! assumes a syntactically-valid `LensDefinition` and checks the
//! semantic contract.

use super::definition::{LensColumn, LensDefinition, LensFilter, LensSort};
use super::dimensions::lookup_dimension;
use super::parser::LensError;

/// The schema version this build of Constellation understands.
const CURRENT_SCHEMA: u32 = 1;

/// Validate a parsed lens definition against the §A dimension registry.
///
/// Returns `Ok(())` on success. On failure, returns `LensError::Validate`
/// with a human-readable message naming the offending field + (where
/// applicable) the registered alternatives.
pub fn validate(def: &LensDefinition) -> Result<(), LensError> {
    // 1. Schema version match.
    if def.schema != CURRENT_SCHEMA {
        return Err(LensError::Validate(format!(
            "schema version mismatch: lens declares schema {}, this build understands schema {}",
            def.schema, CURRENT_SCHEMA
        )));
    }

    // 2. Lens name must be non-empty (parser would have rejected absence; this
    //    catches `lens: ""` which serde happily parses).
    if def.lens.trim().is_empty() {
        return Err(LensError::Validate(
            "lens name must not be empty (set the `lens:` field)".to_string(),
        ));
    }

    // 3. At least one column.
    if def.columns.is_empty() {
        return Err(LensError::Validate(
            "lens must declare at least one column (set the `columns:` list)".to_string(),
        ));
    }

    // 4. Every column references a registered dimension.
    for col in &def.columns {
        validate_column(col)?;
    }

    // 5. Every `where:` filter references a registered + filterable dimension
    //    with a supported operator.
    for filter in &def.where_clauses {
        validate_filter(filter)?;
    }

    // 6. Every `order:` sort references a registered + sortable dimension.
    for sort in &def.order {
        validate_sort(sort)?;
    }

    Ok(())
}

fn validate_column(col: &LensColumn) -> Result<(), LensError> {
    lookup_dimension(&col.dimension).ok_or_else(|| {
        LensError::Validate(format!(
            "columns: unknown dimension `{}`",
            col.dimension
        ))
    })?;
    Ok(())
}

fn validate_filter(filter: &LensFilter) -> Result<(), LensError> {
    let dim = lookup_dimension(&filter.dimension).ok_or_else(|| {
        LensError::Validate(format!(
            "where: unknown dimension `{}`",
            filter.dimension
        ))
    })?;
    if !dim.filterable {
        return Err(LensError::Validate(format!(
            "where: dimension `{}` is not filterable in this build (filterable dimensions: {})",
            filter.dimension,
            filterable_dimension_names().join(", ")
        )));
    }
    if !dim.filter_ops.contains(&filter.op.as_str()) {
        return Err(LensError::Validate(format!(
            "where: unsupported operator `{}` for dimension `{}` (supported: {})",
            filter.op,
            filter.dimension,
            dim.filter_ops.join(", ")
        )));
    }
    Ok(())
}

fn validate_sort(sort: &LensSort) -> Result<(), LensError> {
    let dim = lookup_dimension(&sort.dimension).ok_or_else(|| {
        LensError::Validate(format!(
            "order: unknown dimension `{}`",
            sort.dimension
        ))
    })?;
    if !dim.sortable {
        return Err(LensError::Validate(format!(
            "order: dimension `{}` is not sortable in this build (sortable dimensions: {})",
            sort.dimension,
            sortable_dimension_names().join(", ")
        )));
    }
    Ok(())
}

fn filterable_dimension_names() -> Vec<&'static str> {
    super::dimensions::all_dimensions()
        .iter()
        .filter(|d| d.filterable)
        .map(|d| d.name)
        .collect()
}

fn sortable_dimension_names() -> Vec<&'static str> {
    super::dimensions::all_dimensions()
        .iter()
        .filter(|d| d.sortable)
        .map(|d| d.name)
        .collect()
}

// ─── §B validator tests ───

#[cfg(test)]
mod tests {
    use super::super::definition::{
        FederationMode, LensColumn, LensDefinition, LensFilter, LensScope, LensSort, LensView,
        LibrariesSelector, SortDirection,
    };
    use super::*;

    fn base_definition() -> LensDefinition {
        LensDefinition {
            schema: 1,
            lens: "Test".to_string(),
            template: None,
            scope: LensScope {
                libraries: LibrariesSelector::All,
                federation: FederationMode::Auto,
            },
            where_clauses: vec![],
            order: vec![],
            columns: vec![LensColumn { dimension: "note.name".to_string() }],
            view: LensView::List,
        }
    }

    #[test]
    fn validate_canonical_recent_captures_passes() {
        let def = LensDefinition {
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
        };
        assert!(validate(&def).is_ok());
    }

    #[test]
    fn validate_unknown_column_dimension_rejected() {
        let mut def = base_definition();
        def.columns = vec![LensColumn {
            dimension: "note.frobnitz".to_string(),
        }];
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("note.frobnitz"));
        assert!(err.contains("columns"));
    }

    #[test]
    fn validate_unknown_filter_dimension_rejected() {
        let mut def = base_definition();
        def.where_clauses = vec![LensFilter {
            dimension: "note.frobnitz".to_string(),
            op: "after".to_string(),
            value: "x".to_string(),
        }];
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("note.frobnitz"));
        assert!(err.contains("where"));
    }

    #[test]
    fn validate_unsupported_filter_op_rejected() {
        let mut def = base_definition();
        def.where_clauses = vec![LensFilter {
            dimension: "note.created_at".to_string(),
            // `gt` is not a Timestamp operator (the registry declares
            // after/before/between/within). `gt` is a Number operator
            // we'll add when numeric dimensions ship.
            op: "gt".to_string(),
            value: "100".to_string(),
        }];
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("gt"));
        assert!(err.contains("note.created_at"));
        // Error should suggest supported ops
        assert!(err.contains("after") || err.contains("supported"));
    }

    #[test]
    fn validate_filter_on_non_filterable_dimension_rejected() {
        let mut def = base_definition();
        // note.name is in the registry but not filterable in v1.
        def.where_clauses = vec![LensFilter {
            dimension: "note.name".to_string(),
            op: "is".to_string(),
            value: "anything".to_string(),
        }];
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("note.name"));
        assert!(err.contains("not filterable") || err.contains("filterable"));
    }

    #[test]
    fn validate_sort_on_non_sortable_dimension_rejected() {
        let mut def = base_definition();
        // note.headline is not sortable in v1.
        def.order = vec![LensSort {
            dimension: "note.headline".to_string(),
            direction: SortDirection::Asc,
        }];
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("note.headline"));
        assert!(err.contains("not sortable") || err.contains("sortable"));
    }

    #[test]
    fn validate_unknown_sort_dimension_rejected() {
        let mut def = base_definition();
        def.order = vec![LensSort {
            dimension: "note.frobnitz".to_string(),
            direction: SortDirection::Asc,
        }];
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("note.frobnitz"));
        assert!(err.contains("order"));
    }

    #[test]
    fn validate_schema_version_other_than_1_rejected() {
        let mut def = base_definition();
        def.schema = 2;
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("schema") && err.contains("version"));
        assert!(err.contains("2") && err.contains("1"));
    }

    #[test]
    fn validate_empty_columns_rejected() {
        let mut def = base_definition();
        def.columns = vec![];
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("columns") || err.contains("column"));
    }

    #[test]
    fn validate_empty_lens_name_rejected() {
        let mut def = base_definition();
        def.lens = "".to_string();
        let err = validate(&def).unwrap_err().to_string();
        assert!(err.contains("lens") || err.contains("name"));
    }

    #[test]
    fn validate_whitespace_only_lens_name_rejected() {
        let mut def = base_definition();
        def.lens = "   \t  ".to_string();
        assert!(validate(&def).is_err());
    }
}
