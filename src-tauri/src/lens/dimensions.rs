//! MIG-055 §A — Dimension registry for the lens system.
//!
//! Each `DimensionDef` names a cognitive dimension a lens can reference
//! in its `where` / `order` / `columns` clauses. The dimension's SQL
//! expression + optional JOIN tell the `sql_builder` (§C) how to fetch
//! the value from the search DB.
//!
//! ## Registry (MIG-055 §A v1 + MIG-066 §B Living-Links)
//!
//! | Name                  | Kind      | Sortable | Filterable | Source                                  |
//! |-----------------------|-----------|----------|------------|-----------------------------------------|
//! | `note.name`           | Text      | yes      | no         | `note_meta.name`                        |
//! | `note.path`           | Text      | no       | no         | `note_meta.path`                        |
//! | `note.created_at`     | Timestamp | yes      | yes        | `note_meta.created_at`                  |
//! | `note.headline`       | Text      | no       | no         | `note_summaries.headline` via LEFT JOIN |
//! | `note.outgoing_count` | Number    | yes      | no         | `note_meta.outgoing_count` (MIG-066 §A) |
//! | `note.link_types`     | Text      | yes      | no         | `note_meta.outgoing_link_types` (§A)    |
//! | `note.library`        | Text      | yes      | yes        | `note_meta.library_name` (MIG-090 §1)   |
//! | `note.modified`       | Timestamp | yes      | yes        | `note_meta.modified` (MIG-090 §1)       |
//! | `note.tags`           | List      | no       | no         | `note_meta.tags_json` (MIG-090 §1)      |
//!
//! Filter ops for `note.created_at` / `note.modified`: `after` / `before` / `between` / `within`.

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
    // ─── MIG-066 §B — Living-Links columns (the "Connection" question) ───
    // Both read the write-time-materialized note_meta columns from §A (plain
    // column reads — Rule 8). Outgoing-side only in v1 (backlinks → v2; the
    // columns are honestly outgoing-only). `note.link_types` is the canonical-
    // ordered string of distinct outgoing typed relations; its rank-aware sort
    // key (outgoing_top_rank) is wired in §D so sorting it is canonical, not
    // alphabetical.
    DimensionDef {
        name: "note.outgoing_count",
        kind: DimensionKind::Number,
        sql_expression: "note_meta.outgoing_count",
        requires_join: None,
        sortable: true,
        filterable: false,
        filter_ops: &[],
    },
    DimensionDef {
        name: "note.link_types",
        kind: DimensionKind::Text,
        sql_expression: "note_meta.outgoing_link_types",
        requires_join: None,
        sortable: true,
        filterable: false,
        filter_ops: &[],
    },
    // ─── MIG-090 §1 — the All-Notes Base columns ───
    // Plain note_meta column reads (Rule 8): the working-set list needs the
    // note's home library, its last-touch time, and its tags.
    DimensionDef {
        name: "note.library",
        kind: DimensionKind::Text,
        sql_expression: "note_meta.library_name",
        requires_join: None,
        sortable: true,
        filterable: true,
        filter_ops: &["is", "is_not", "contains", "does_not_contain", "is_empty", "is_not_empty"],
    },
    DimensionDef {
        name: "note.modified",
        kind: DimensionKind::Timestamp,
        sql_expression: "note_meta.modified",
        requires_join: None,
        sortable: true,
        filterable: true,
        filter_ops: &["after", "before", "between", "within"],
    },
    DimensionDef {
        name: "note.tags",
        kind: DimensionKind::List,
        sql_expression: "note_meta.tags_json",
        requires_join: None,
        // v1: not sortable (ordering by a JSON array string is noise —
        // Form-Aligns-To-Purpose) and not filterable (exact-tag matching
        // needs a custom `tags_json LIKE '%\"?\"%'` op — deferred).
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

// ─── MIG-065 §C/§D — raw frontmatter property columns ───

/// Prefix marking a raw frontmatter-property column reference.
/// `prop.<key>` → `json_extract(note_meta.properties_json, '$."<key>"')`.
/// The unified Base's familiar table references a note's own frontmatter
/// fields this way, alongside the registered cognitive dimensions. The prefix
/// keeps the namespace convention consistent with `note.` / future `link.`.
pub const PROP_PREFIX: &str = "prop.";

/// Prefix marking a per-type outgoing-link COUNT column. MIG-067 §F:
/// `note.link.<typeid>` → `COALESCE(json_extract(note_meta.outgoing_link_types_json,
/// '$."<id>"'), 0)` — how many active outgoing links of that type the note has as a
/// source (MIG-066 §B materialised the `{type:count}` JSON write-time; this surfaces
/// it as a sortable Base column). Read-only Number; an absent type ⇒ 0.
pub const LINK_PREFIX: &str = "note.link.";

/// Filter operators available on a raw frontmatter (Text) property column —
/// the familiar Obsidian/Notion operator set.
const PROP_FILTER_OPS: &[&str] = &[
    "is", "is_not", "contains", "does_not_contain", "is_empty", "is_not_empty",
];

/// A column/filter/sort reference resolved to its SQL form: either a registered
/// cognitive dimension (static expression) or a dynamic `prop.<key>` frontmatter
/// property (`json_extract`). `sql_expression` is owned because the property
/// case is built per-key at runtime.
#[derive(Debug, Clone)]
pub struct ResolvedDim {
    pub kind: DimensionKind,
    pub sql_expression: String,
    pub requires_join: Option<&'static str>,
    pub sortable: bool,
    pub filterable: bool,
    pub filter_ops: Vec<&'static str>,
    /// `true` when resolved from `prop.<key>` (a read-only frontmatter column);
    /// `false` for a registered cognitive dimension.
    pub is_property: bool,
}

/// Resolve a column reference to its SQL form. Accepts a registered dimension
/// name (e.g. `note.created_at`) OR a `prop.<key>` frontmatter reference.
/// Returns `None` for anything else (the validator turns that into an error).
pub fn resolve_dim(name: &str) -> Option<ResolvedDim> {
    if let Some(d) = lookup_dimension(name) {
        return Some(ResolvedDim {
            kind: d.kind,
            sql_expression: d.sql_expression.to_string(),
            requires_join: d.requires_join,
            sortable: d.sortable,
            filterable: d.filterable,
            filter_ops: d.filter_ops.to_vec(),
            is_property: false,
        });
    }
    if let Some(key) = name.strip_prefix(PROP_PREFIX) {
        let key = key.trim();
        if key.is_empty() {
            return None;
        }
        let safe = sanitize_json_key(key);
        if safe.is_empty() {
            return None;
        }
        return Some(ResolvedDim {
            kind: DimensionKind::Text,
            sql_expression: format!(
                "json_extract(note_meta.properties_json, '$.\"{}\"')",
                safe
            ),
            requires_join: None,
            sortable: true,
            filterable: true,
            filter_ops: PROP_FILTER_OPS.to_vec(),
            is_property: true,
        });
    }
    // MIG-067 §F — per-type outgoing-link count column (`note.link.<typeid>`).
    if let Some(id) = name.strip_prefix(LINK_PREFIX) {
        let id = id.trim();
        if id.is_empty() {
            return None;
        }
        let safe = sanitize_json_key(id);
        if safe.is_empty() {
            return None;
        }
        return Some(ResolvedDim {
            kind: DimensionKind::Number,
            sql_expression: format!(
                "COALESCE(json_extract(note_meta.outgoing_link_types_json, '$.\"{}\"'), 0)",
                safe
            ),
            requires_join: None,
            sortable: true,
            // v1: a count column is sortable but not filterable (no "supports > 5"
            // operator yet); filtering is a future phase.
            filterable: false,
            filter_ops: vec![],
            // Not an editable frontmatter property — a read-only computed aggregate.
            is_property: false,
        });
    }
    None
}

/// Strip characters that would break the embedded JSON-path string literal
/// (the path is inlined into SQL, not bound). Frontmatter keys are normally
/// simple identifiers; quotes / backslashes / control chars are dropped
/// defensively against injection.
fn sanitize_json_key(key: &str) -> String {
    key.chars()
        .filter(|c| *c != '"' && *c != '\\' && !c.is_control())
        .collect()
}

// ─── §A unit tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_registry_includes_v1_plus_links_dimensions() {
        let names = dimension_names();
        assert_eq!(names.len(), 9, "registry = 4 MIG-055 v1 dims + 2 MIG-066 Living-Links dims + 3 MIG-090 All-Notes dims");
        assert!(names.contains(&"note.name"));
        assert!(names.contains(&"note.path"));
        assert!(names.contains(&"note.created_at"));
        assert!(names.contains(&"note.headline"));
        assert!(names.contains(&"note.outgoing_count"));
        assert!(names.contains(&"note.link_types"));
        assert!(names.contains(&"note.library"));
        assert!(names.contains(&"note.modified"));
        assert!(names.contains(&"note.tags"));
    }

    #[test]
    fn all_notes_dimensions_read_note_meta_columns() {
        // MIG-090 §1 — plain note_meta column reads (Rule 8), no JOIN.
        let l = lookup_dimension("note.library").expect("note.library registered");
        assert_eq!(l.kind, DimensionKind::Text);
        assert_eq!(l.sql_expression, "note_meta.library_name");
        assert!(l.requires_join.is_none());
        assert!(l.sortable && l.filterable);

        let m = lookup_dimension("note.modified").expect("note.modified registered");
        assert_eq!(m.kind, DimensionKind::Timestamp);
        assert_eq!(m.sql_expression, "note_meta.modified");
        assert!(m.requires_join.is_none());
        assert!(m.sortable && m.filterable);
        assert_eq!(m.filter_ops, &["after", "before", "between", "within"]);

        let t = lookup_dimension("note.tags").expect("note.tags registered");
        assert_eq!(t.kind, DimensionKind::List);
        assert_eq!(t.sql_expression, "note_meta.tags_json");
        assert!(t.requires_join.is_none());
        assert!(!t.sortable && !t.filterable);
    }

    #[test]
    fn link_dimensions_read_materialized_columns_and_sort() {
        // MIG-066 §B — both Living-Links dims resolve to the write-time-materialized
        // note_meta columns (plain reads, Rule 8), no JOIN, sortable, not filterable.
        let c = lookup_dimension("note.outgoing_count").expect("note.outgoing_count registered");
        assert_eq!(c.kind, DimensionKind::Number);
        assert_eq!(c.sql_expression, "note_meta.outgoing_count");
        assert!(c.requires_join.is_none());
        assert!(c.sortable && !c.filterable);

        let t = lookup_dimension("note.link_types").expect("note.link_types registered");
        assert_eq!(t.kind, DimensionKind::Text);
        assert_eq!(t.sql_expression, "note_meta.outgoing_link_types");
        assert!(t.requires_join.is_none());
        assert!(t.sortable && !t.filterable);
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
        assert_eq!(
            names,
            vec![
                "note.name", "note.path", "note.created_at", "note.headline",
                "note.outgoing_count", "note.link_types",
                "note.library", "note.modified", "note.tags",
            ]
        );
    }

    // ─── MIG-065 §C/§D — resolve_dim (registered + prop.<key>) ───

    #[test]
    fn resolve_dim_registered_matches_lookup() {
        let r = resolve_dim("note.created_at").unwrap();
        assert_eq!(r.sql_expression, "note_meta.created_at");
        assert_eq!(r.kind, DimensionKind::Timestamp);
        assert!(!r.is_property);
        assert!(r.filterable && r.sortable);
    }

    #[test]
    fn resolve_dim_property_builds_json_extract() {
        let r = resolve_dim("prop.status").unwrap();
        assert!(r.is_property);
        assert_eq!(r.kind, DimensionKind::Text);
        assert!(r.sql_expression.contains("json_extract(note_meta.properties_json"));
        assert!(r.sql_expression.contains("\"status\""));
        assert!(r.sortable && r.filterable);
        assert!(r.filter_ops.contains(&"contains"));
        assert!(r.filter_ops.contains(&"is_empty"));
    }

    #[test]
    fn resolve_dim_link_count_builds_coalesced_json_extract() {
        // MIG-067 §F — note.link.<typeid> → a sortable Number count column.
        let r = resolve_dim("note.link.supports").unwrap();
        assert_eq!(r.kind, DimensionKind::Number);
        assert!(r.sql_expression.contains("json_extract(note_meta.outgoing_link_types_json"));
        assert!(r.sql_expression.contains("\"supports\""));
        assert!(r.sql_expression.starts_with("COALESCE("), "absent type ⇒ 0: {}", r.sql_expression);
        assert!(r.sortable, "count columns sort");
        assert!(!r.filterable, "v1: count columns are not filterable");
        assert!(!r.is_property, "a computed aggregate, not editable frontmatter");
        // a custom (hyphenated) id resolves the same way
        assert!(resolve_dim("note.link.evidence-for").unwrap().sql_expression.contains("\"evidence-for\""));
        // empty / blank id → None
        assert!(resolve_dim("note.link.").is_none());
        assert!(resolve_dim("note.link.   ").is_none());
    }

    #[test]
    fn resolve_dim_property_sanitizes_quotes() {
        // a key with a quote must not break out of the JSON-path literal.
        let r = resolve_dim("prop.ev\"il").unwrap();
        assert!(!r.sql_expression.contains("ev\"il"));
        assert!(r.sql_expression.contains("evil"));
    }

    #[test]
    fn resolve_dim_unknown_and_empty_rejected() {
        assert!(resolve_dim("note.frobnitz").is_none());
        assert!(resolve_dim("prop.").is_none());
        assert!(resolve_dim("prop.   ").is_none());
        assert!(resolve_dim("").is_none());
    }
}
