//! MIG-055 §B — Lens definition data shape.
//!
//! Parsed from the YAML in a ` ```base ` code block (per v1.4 §7
//! host-note assemblage) OR from a standalone `.base` file. Per
//! Architect §4.1, schema v1 supports:
//!
//! - `schema: 1` (required)
//! - `lens: "Display name"` (required)
//! - `template: <id>` (optional — names a Five Acts system template)
//! - `scope: { libraries: all|[...], federation: auto|off }`
//! - `where:` list of filter clauses
//! - `order:` list of sort clauses
//! - `columns:` list of dimension columns (required, at least one)
//! - `view: list` (v1 only ships list)

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A complete lens definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LensDefinition {
    /// Schema version. v1 expects exactly `1`; mismatches are validator errors.
    pub schema: u32,

    /// Display name (or template ID when `template` is also set).
    pub lens: String,

    /// Optional Five Acts template identifier (e.g., `"five-acts.observation"`).
    /// Present implies "this is a known shape, not a user composition".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Scope: which notes the lens looks at.
    #[serde(default)]
    pub scope: LensScope,

    /// Filter clauses (lens INCLUDES these). Empty = no filter (all rows in scope).
    /// Serialized as the YAML key `where:` (Rust reserved keyword).
    #[serde(default, rename = "where")]
    pub where_clauses: Vec<LensFilter>,

    /// Sort clauses. Empty = arbitrary order.
    #[serde(default)]
    pub order: Vec<LensSort>,

    /// Columns to render. Validator requires at least one.
    pub columns: Vec<LensColumn>,

    /// Render shape. v1 ships only `list`.
    #[serde(default)]
    pub view: LensView,
}

/// `scope:` block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LensScope {
    /// Library selection — `"all"` keyword or an explicit subset list.
    #[serde(default)]
    pub libraries: LibrariesSelector,

    /// Federation behavior across cUniverse children.
    #[serde(default)]
    pub federation: FederationMode,
}

impl Default for LensScope {
    fn default() -> Self {
        Self {
            libraries: LibrariesSelector::default(),
            federation: FederationMode::default(),
        }
    }
}

/// `scope.libraries` accepts either the literal `"all"` (= all visible libraries
/// in the federated set) or a list of library names (an explicit subset).
#[derive(Debug, Clone, PartialEq)]
pub enum LibrariesSelector {
    /// `libraries: all` — every library in the federated set.
    All,
    /// `libraries: [Lib1, Lib2]` — explicit subset.
    Subset(Vec<String>),
}

impl Default for LibrariesSelector {
    fn default() -> Self {
        LibrariesSelector::All
    }
}

impl Serialize for LibrariesSelector {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self {
            LibrariesSelector::All => ser.serialize_str("all"),
            LibrariesSelector::Subset(v) => v.serialize(ser),
        }
    }
}

impl<'de> Deserialize<'de> for LibrariesSelector {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let v = serde_yaml::Value::deserialize(de)?;
        match v {
            serde_yaml::Value::String(s) => {
                if s == "all" {
                    Ok(LibrariesSelector::All)
                } else {
                    Err(D::Error::custom(format!(
                        "scope.libraries: expected \"all\" or a list of library names, got string \"{}\"",
                        s
                    )))
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                let names: Result<Vec<String>, _> = seq
                    .into_iter()
                    .map(|el| match el {
                        serde_yaml::Value::String(s) => Ok(s),
                        other => Err(D::Error::custom(format!(
                            "scope.libraries: list entries must be strings, got {:?}",
                            other
                        ))),
                    })
                    .collect();
                names.map(LibrariesSelector::Subset)
            }
            other => Err(D::Error::custom(format!(
                "scope.libraries: expected \"all\" or a list, got {:?}",
                other
            ))),
        }
    }
}

/// `scope.federation` behavior.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FederationMode {
    /// Federation auto-on per v1.4 §10.6 — cUniverse children included.
    Auto,
    /// Federation explicitly disabled for this lens.
    Off,
}

impl Default for FederationMode {
    fn default() -> Self {
        FederationMode::Auto
    }
}

/// One filter clause in the `where:` list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LensFilter {
    /// Dimension being filtered (must be in the registry + filterable).
    pub dimension: String,

    /// Filter operator (must be in the dimension's `filter_ops` list).
    pub op: String,

    /// Filter value (string in v1; future phases may add numeric / list types).
    pub value: String,
}

/// One sort clause in the `order:` list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LensSort {
    /// Dimension to sort by (must be in the registry + sortable).
    pub dimension: String,

    /// Sort direction.
    pub direction: SortDirection,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// One column in the `columns:` list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LensColumn {
    /// Dimension to render in this column (must be in the registry).
    pub dimension: String,
}

/// Render shape. v1 ships `list` only; future phases earn other shapes
/// per v1.4 §5.1 Form-Aligns-To-Purpose.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LensView {
    List,
}

impl Default for LensView {
    fn default() -> Self {
        LensView::List
    }
}
