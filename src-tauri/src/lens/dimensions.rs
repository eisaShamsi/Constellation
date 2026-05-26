//! MIG-055 §A — Dimension registry for the lens system.
//!
//! Each `DimensionDef` names a cognitive dimension a lens can reference
//! in its `where` / `order` / `columns` clauses. The dimension's SQL
//! expression + optional JOIN tell the `sql_builder` (§C) how to fetch
//! the value from the search DB.
//!
//! ## v1 scope (per Architect §4.1 / §5.2 / Plan §A)
//!
//! Four dimensions only. Future phases extend.
//!
//! | Name              | Kind      | Sortable | Filterable | Source                                        |
//! |-------------------|-----------|----------|------------|-----------------------------------------------|
//! | `note.name`       | Text      | yes      | no         | `note_meta.name`                              |
//! | `note.path`       | Text      | no       | no         | `note_meta.path`                              |
//! | `note.created_at` | Timestamp | yes      | yes        | `note_meta.created_at`                        |
//! | `note.headline`   | Text      | no       | no         | `note_summaries.headline` via LEFT JOIN       |
//!
//! Filter ops for `note.created_at`: `after` / `before` / `between` / `within`.

/// The kind of value a dimension yields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionKind {
    /// String values (e.g., note name, NSC headline).
    Text,
    /// Numeric values (e.g., future word counts, link weights).
    #[allow(dead_code)]
    Number,
    /// Unix-second timestamps (e.g., created_at, modified).
    Timestamp,
    /// Boolean flags (e.g., future is_orphan, is_fragile).
    #[allow(dead_code)]
    Bool,
    /// Comma-joined list values (e.g., future tags).
    #[allow(dead_code)]
    List,
}

/// One entry in the dimension registry.
///
/// All fields are `&'static` so the registry is a `const` array — zero
/// runtime cost; the dimension set is fixed at compile time per phase.
#[derive(Debug, Clone, Copy)]
pub struct DimensionDef {
    /// The dimension's canonical name (e.g., `"note.created_at"`).
    /// Used as the lookup key + the key in `LensRow.dimensions`.
    pub name: &'static str,

    /// The kind of value this dimension yields.
    pub kind: DimensionKind,

    /// The SQL expression that produces this dimension's value.
    /// Referenced from the FROM clause built by `sql_builder` (§C).
    pub sql_expression: &'static str,

    /// If `Some`, the SQL FROM clause must include this JOIN fragment
    /// when this dimension appears in the query. `sql_builder` (§C)
    /// deduplicates JOINs across all dimensions used by a lens.
    pub requires_join: Option<&'static str>,

    /// `true` if this dimension can appear in a lens's `order:` clause.
    pub sortable: bool,

    /// `true` if this dimension can appear in a lens's `where:` clause.
    pub filterable: bool,

    /// The filter operators this dimension supports (when filterable).
    /// Empty when `filterable == false`.
    pub filter_ops: &'static [&'static str],
}

/// The v1 dimension registry.
///
/// Order matters only for stable iteration (e.g., docs / debugging);
/// lookup is by name.
const REGISTRY: &[DimensionDef] = &[
    DimensionDef {
        name: "note.name",
        kind: DimensionKind::Text,
        sql_expression: "note_meta.name",
        requires_join: None,
        sortable: true,
        filterable: false,
        filter_ops: &[],
    },
    DimensionDef {
        name: "note.path",
        kind: DimensionKind::Text,
        sql_expression: "note_meta.path",
        requires_join: None,
        sortable: false,
        filterable: false,
        filter_ops: &[],
    },
    DimensionDef {
        name: "note.created_at",
        kind: DimensionKind::Timestamp,
        sql_expression: "note_meta.created_at",
        requires_join: None,
        sortable: true,
        filterable: true,
        filter_ops: &["after", "before", "between", "within"],
    },
    DimensionDef {
        name: "note.headline",
        kind: DimensionKind::Text,
        sql_expression: "note_summaries.headline",
        requires_join: Some(
            "LEFT JOIN note_summaries ON note_summaries.path = note_meta.path",
        ),
        sortable: false,
        filterable: false,
        filter_ops: &[],
    },
];

/// Look up a dimension by its canonical name.
/// Returns `None` if the name is not registered.
pub fn lookup_dimension(name: &str) -> Option<&'static DimensionDef> {
    REGISTRY.iter().find(|d| d.name == name)
}

/// All registered dimensions (for iteration / docs / debugging).
#[allow(dead_code)] // Used by future phases + the §G test harness.
pub fn all_dimensions() -> &'static [DimensionDef] {
    REGISTRY
}

/// Canonical names of all registered dimensions.
/// Useful for error messages (`Unknown dimension: X. Known: [...]`).
pub fn dimension_names() -> Vec<&'static str> {
    REGISTRY.iter().map(|d| d.name).collect()
}

// ─── §A unit tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_registry_includes_4_v1_dimensions() {
        let names = dimension_names();
        assert_eq!(names.len(), 4, "v1 dimension registry must contain exactly 4 dimensions");
        assert!(names.contains(&"note.name"));
        assert!(names.contains(&"note.path"));
        assert!(names.contains(&"note.created_at"));
        assert!(names.contains(&"note.headline"));
    }

    #[test]
    fn dimension_registry_lookup_by_name() {
        let d = lookup_dimension("note.created_at").expect("note.created_at registered");
        assert_eq!(d.kind, DimensionKind::Timestamp);
        assert_eq!(d.sql_expression, "note_meta.created_at");
        assert!(d.sortable);
        assert!(d.filterable);
    }

    #[test]
    fn dimension_registry_unknown_returns_none() {
        assert!(lookup_dimension("note.frobnitz").is_none());
        assert!(lookup_dimension("").is_none());
        assert!(lookup_dimension("note.name.suffix").is_none());
        assert!(lookup_dimension("NOTE.NAME").is_none(), "lookup is case-sensitive");
    }

    #[test]
    fn note_created_at_filter_ops_includes_after_before_between_within() {
        let d = lookup_dimension("note.created_at").unwrap();
        assert!(d.filter_ops.contains(&"after"));
        assert!(d.filter_ops.contains(&"before"));
        assert!(d.filter_ops.contains(&"between"));
        assert!(d.filter_ops.contains(&"within"));
    }

    #[test]
    fn note_name_is_sortable() {
        let d = lookup_dimension("note.name").unwrap();
        assert!(d.sortable);
        assert!(!d.filterable, "note.name not filterable in v1");
    }

    #[test]
    fn note_headline_is_not_sortable_not_filterable_in_v1() {
        let d = lookup_dimension("note.headline").unwrap();
        assert!(!d.sortable, "headlines are display-only in v1");
        assert!(!d.filterable);
        assert!(d.filter_ops.is_empty());
    }

    #[test]
    fn note_headline_requires_join() {
        let d = lookup_dimension("note.headline").unwrap();
        let join = d.requires_join.expect("headline source requires a JOIN");
        assert!(join.contains("note_summaries"));
        assert!(join.contains("path"));
        assert!(join.contains("LEFT JOIN"));
    }

    #[test]
    fn note_path_is_registered_but_not_sortable_not_filterable() {
        // note.path is in the registry so users CAN reference it as a column;
        // sorting / filtering on path isn't useful in v1 (paths are arbitrary
        // strings without natural sort semantics for the user).
        let d = lookup_dimension("note.path").unwrap();
        assert_eq!(d.sql_expression, "note_meta.path");
        assert!(!d.sortable);
        assert!(!d.filterable);
        assert_eq!(d.kind, DimensionKind::Text);
    }

    #[test]
    fn all_v1_dimensions_use_note_prefix() {
        // Per Architect §11 #1 lock — v1 dimensions all use the `note.` prefix
        // (none are link.X or note.cns.X or note.cece.X in v1).
        for d in all_dimensions() {
            assert!(
                d.name.starts_with("note."),
                "v1 dimension `{}` does not use the `note.` prefix",
                d.name
            );
        }
    }

    #[test]
    fn registry_iteration_is_stable() {
        // The registry is a const array — order is deterministic across runs.
        // This test pins the order so future readers + diff reviewers spot
        // re-ordering immediately.
        let names: Vec<&str> = all_dimensions().iter().map(|d| d.name).collect();
        assert_eq!(names, vec!["note.name", "note.path", "note.created_at", "note.headline"]);
    }
}
