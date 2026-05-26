//! MIG-055 §B — Lens YAML parser.
//!
//! Parses the YAML text from a ` ```base ` code block (or a standalone
//! `.base` file) into a `LensDefinition`. Parsing failures surface as
//! `LensError::Parse` with the raw serde_yaml error message so the
//! `LensBlock.svelte` renderer (§D) can show a meaningful error to
//! the user (e.g., `"Failed to parse lens: missing field \`columns\`"`).

use super::definition::LensDefinition;

/// Errors from the lens pipeline (parse + validate).
#[derive(Debug, Clone, PartialEq)]
pub enum LensError {
    /// YAML parse failed (malformed syntax / missing required field /
    /// wrong type for a known field).
    Parse(String),
    /// Schema validation failed (unknown dimension / unsupported op /
    /// schema-version mismatch / empty columns / etc.). See §B validator.
    Validate(String),
}

impl std::fmt::Display for LensError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LensError::Parse(msg) => write!(f, "Failed to parse lens: {}", msg),
            LensError::Validate(msg) => write!(f, "Lens validation failed: {}", msg),
        }
    }
}

impl std::error::Error for LensError {}

/// Parse a YAML string into a `LensDefinition`.
///
/// Validation is a separate step (call `super::validator::validate`
/// on the returned `LensDefinition`) — parsing here only checks YAML
/// syntax + serde-shape conformance, not the dimension registry.
pub fn parse_lens_yaml(yaml: &str) -> Result<LensDefinition, LensError> {
    serde_yaml::from_str::<LensDefinition>(yaml)
        .map_err(|e: serde_yaml::Error| LensError::Parse(e.to_string()))
}

// ─── §B parser tests ───

#[cfg(test)]
mod tests {
    use super::super::definition::{
        FederationMode, LensColumn, LensFilter, LensSort, LensView, LibrariesSelector,
        SortDirection,
    };
    use super::*;

    /// The canonical Recent Captures fixture (per Architect §6).
    const RECENT_CAPTURES_YAML: &str = r#"
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

    #[test]
    fn parse_recent_captures_round_trip() {
        let def = parse_lens_yaml(RECENT_CAPTURES_YAML).expect("Recent Captures should parse");
        assert_eq!(def.schema, 1);
        assert_eq!(def.lens, "Recent Captures");
        assert_eq!(def.template, Some("five-acts.observation".to_string()));
        assert_eq!(def.scope.libraries, LibrariesSelector::All);
        assert_eq!(def.scope.federation, FederationMode::Auto);
        assert_eq!(def.where_clauses.len(), 1);
        assert_eq!(def.where_clauses[0].dimension, "note.created_at");
        assert_eq!(def.where_clauses[0].op, "after");
        assert_eq!(def.where_clauses[0].value, "now - 14 days");
        assert_eq!(def.order.len(), 1);
        assert_eq!(def.order[0].dimension, "note.created_at");
        assert_eq!(def.order[0].direction, SortDirection::Desc);
        assert_eq!(def.columns.len(), 2);
        assert_eq!(def.columns[0].dimension, "note.name");
        assert_eq!(def.columns[1].dimension, "note.headline");
        assert_eq!(def.view, LensView::List);
    }

    #[test]
    fn parse_template_field_is_optional() {
        let yaml = r#"
schema: 1
lens: "Test"
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("template-less lens should parse");
        assert_eq!(def.template, None);
    }

    #[test]
    fn parse_libraries_subset() {
        let yaml = r#"
schema: 1
lens: "Test"
scope:
  libraries:
    - Lib1
    - Lib2
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("subset libraries should parse");
        match def.scope.libraries {
            LibrariesSelector::Subset(v) => {
                assert_eq!(v, vec!["Lib1".to_string(), "Lib2".to_string()]);
            }
            other => panic!("expected Subset, got {:?}", other),
        }
    }

    #[test]
    fn parse_libraries_all_keyword() {
        let yaml = r#"
schema: 1
lens: "Test"
scope:
  libraries: all
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("libraries: all should parse");
        assert_eq!(def.scope.libraries, LibrariesSelector::All);
    }

    #[test]
    fn parse_libraries_other_string_rejected() {
        let yaml = r#"
schema: 1
lens: "Test"
scope:
  libraries: some_random_string
columns:
  - dimension: note.name
"#;
        let result = parse_lens_yaml(yaml);
        assert!(result.is_err(), "non-\"all\" string for libraries should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("scope.libraries") || err.contains("libraries"),
            "error should mention libraries field; got: {}",
            err
        );
    }

    #[test]
    fn parse_federation_off() {
        let yaml = r#"
schema: 1
lens: "Test"
scope:
  libraries: all
  federation: off
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("federation: off should parse");
        assert_eq!(def.scope.federation, FederationMode::Off);
    }

    #[test]
    fn parse_defaults_when_scope_omitted() {
        let yaml = r#"
schema: 1
lens: "Test"
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("scope-less lens should parse with defaults");
        assert_eq!(def.scope.libraries, LibrariesSelector::All);
        assert_eq!(def.scope.federation, FederationMode::Auto);
    }

    #[test]
    fn parse_empty_where_and_order_are_valid() {
        let yaml = r#"
schema: 1
lens: "Test"
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("lens with no filters/sorts should parse");
        assert_eq!(def.where_clauses.len(), 0);
        assert_eq!(def.order.len(), 0);
    }

    #[test]
    fn parse_missing_lens_name_rejected() {
        let yaml = r#"
schema: 1
columns:
  - dimension: note.name
"#;
        let result = parse_lens_yaml(yaml);
        assert!(result.is_err(), "missing `lens` field should fail parse");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("lens"), "error should mention the missing field; got: {}", err);
    }

    #[test]
    fn parse_missing_columns_field_rejected() {
        let yaml = r#"
schema: 1
lens: "Test"
"#;
        let result = parse_lens_yaml(yaml);
        assert!(result.is_err(), "missing `columns` field should fail parse");
    }

    #[test]
    fn parse_multilingual_lens_name_arabic() {
        let yaml = r#"
schema: 1
lens: "الالتقاطات الأخيرة"
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("Arabic lens name should parse");
        assert_eq!(def.lens, "الالتقاطات الأخيرة");
    }

    #[test]
    fn parse_view_list_default() {
        let yaml = r#"
schema: 1
lens: "Test"
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("default view should parse");
        assert_eq!(def.view, LensView::List);
    }

    #[test]
    fn parse_view_table_currently_rejected_by_enum() {
        // v1 LensView enum only has `List`. `view: table` should fail at serde level
        // (the validator would also reject, but parse catches it first).
        let yaml = r#"
schema: 1
lens: "Test"
columns:
  - dimension: note.name
view: table
"#;
        let result = parse_lens_yaml(yaml);
        assert!(result.is_err(), "v1 only supports `view: list`; table should fail");
    }

    #[test]
    fn parse_sort_direction_asc_desc() {
        let yaml = r#"
schema: 1
lens: "Test"
order:
  - dimension: note.name
    direction: asc
  - dimension: note.created_at
    direction: desc
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("two-sort lens should parse");
        assert_eq!(def.order[0].direction, SortDirection::Asc);
        assert_eq!(def.order[1].direction, SortDirection::Desc);
    }

    #[test]
    fn parse_time_relative_filter_values_round_trip() {
        // The parser stores filter values verbatim as strings; the SQL builder (§C)
        // is responsible for translating "now - 14 days" / "now" / ISO timestamps
        // into Unix-second comparisons. The parser must not mangle the string.
        let yaml = r#"
schema: 1
lens: "Test"
where:
  - dimension: note.created_at
    op: after
    value: "now - 14 days"
  - dimension: note.created_at
    op: before
    value: "2026-01-01T00:00:00Z"
columns:
  - dimension: note.name
"#;
        let def = parse_lens_yaml(yaml).expect("time-relative filters should parse");
        assert_eq!(def.where_clauses[0].value, "now - 14 days");
        assert_eq!(def.where_clauses[1].value, "2026-01-01T00:00:00Z");
    }
}
