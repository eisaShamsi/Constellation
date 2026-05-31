//! MIG-067 §A — The Link-Type Registry ("The Living Vocabulary").
//!
//! Single source of truth for link types. The 8 canonical acts (Living-Link
//! Concept Paper §6–7) are built-in **seeds** — their ids, semantics, and derived
//! order are immutable (the grammar of inquiry). Users grow their own vocabulary
//! on top: **top-level** types (peers of the 8) or **children** nested under one
//! of the 8 (v1 = one level), stored as deltas in
//! `<universe>/.constellation/link-types.json`. Every surface — parser,
//! materialization, editor, Base — reads this registry, so the vocabulary can
//! change without touching code (and the pre-existing drift across ~25 hardcoded
//! lists collapses to one source). The 8 are a living seed, not a cage.

use serde::{Deserialize, Serialize};
use std::sync::{OnceLock, RwLock};

/// One link type — a built-in seed (the 8) or a user-defined type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkTypeDef {
    /// Slug id (lowercase-hyphen). For seeds, one of `SEED_IDS` (immutable).
    pub id: String,
    /// Display label. The 8 localize via i18n; custom types use the user's label.
    pub label: String,
    /// `None` = top-level; else a canonical-8 id (a sub-type refining that act).
    #[serde(default)]
    pub parent: Option<String>,
    /// Hex color.
    pub color: String,
    /// Position within its tier (top-level among top-level; child among siblings).
    pub order: i64,
    /// `true` for the 8 seeds — id + semantics + existence locked (only color /
    /// order / label may be overridden).
    #[serde(default)]
    pub builtin: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
}

/// The 8 canonical seed ids, in derived order (Concept Paper §7). `associative`
/// is the null/untyped synonym — NOT a type you add, handled separately.
pub const SEED_IDS: &[&str] = &[
    "supports", "contradicts", "causes", "exemplifies",
    "generalizes", "derives-from", "part-of", "supersedes",
];

/// The 8 built-in seeds (ids + derived order + canonical colors from
/// `DEFAULT_SETTINGS.linkPills`). The English `label`/`desc` are fallbacks; the
/// frontend localizes builtins via i18n.
pub fn seeds() -> Vec<LinkTypeDef> {
    let mk = |id: &str, label: &str, color: &str, order: i64, desc: &str| LinkTypeDef {
        id: id.to_string(),
        label: label.to_string(),
        parent: None,
        color: color.to_string(),
        order,
        builtin: true,
        emoji: None,
        desc: Some(desc.to_string()),
    };
    vec![
        mk("supports", "Supports", "#4A9EFF", 1, "Evidence for a claim"),
        mk("contradicts", "Contradicts", "#FF4A4A", 2, "Tension / opposition"),
        mk("causes", "Causes", "#FF8C42", 3, "Causal relationship"),
        mk("exemplifies", "Exemplifies", "#4AFF88", 4, "Instance-of"),
        mk("generalizes", "Generalizes", "#A44AFF", 5, "Abstraction"),
        mk("derives-from", "Derives From", "#FFD700", 6, "Provenance / source"),
        mk("part-of", "Part Of", "#AAAAAA", 7, "Compositional hierarchy"),
        mk("supersedes", "Supersedes", "#5B7A8A", 8, "Replaces an earlier stance"),
    ]
}

/// The resolved registry: the 8 seeds merged with user deltas, in flattened
/// canonical order (top-level by `order`, each followed by its children).
#[derive(Debug, Clone)]
pub struct LinkTypeRegistry {
    types: Vec<LinkTypeDef>,
}

impl LinkTypeRegistry {
    pub fn seeds_only() -> Self {
        Self { types: flatten_ordered(seeds()) }
    }

    /// Merge built-in seeds with user deltas. A delta whose id is a seed id
    /// OVERRIDES that seed's color/order/label only (never its id/semantics/
    /// existence; `parent` forced None, `builtin` forced true). A delta with a new
    /// id ADDS a custom type (`builtin` false; an invalid/non-seed parent is dropped
    /// to top-level — v1 nests only under the 8).
    pub fn merge(deltas: Vec<LinkTypeDef>) -> Self {
        let seed_set: std::collections::HashSet<&str> = SEED_IDS.iter().copied().collect();
        let mut by_id: std::collections::BTreeMap<String, LinkTypeDef> =
            seeds().into_iter().map(|d| (d.id.clone(), d)).collect();
        for mut d in deltas {
            if d.id.trim().is_empty() {
                continue;
            }
            if seed_set.contains(d.id.as_str()) {
                d.builtin = true;
                d.parent = None;
                if let Some(seed) = by_id.get(&d.id) {
                    if d.label.trim().is_empty() {
                        d.label = seed.label.clone();
                    }
                    if d.desc.is_none() {
                        d.desc = seed.desc.clone();
                    }
                }
            } else {
                d.builtin = false;
                if let Some(p) = &d.parent {
                    if !seed_set.contains(p.as_str()) {
                        d.parent = None; // v1: children only under the 8
                    }
                }
            }
            by_id.insert(d.id.clone(), d);
        }
        Self { types: flatten_ordered(by_id.into_values().collect()) }
    }

    pub fn is_known(&self, id: &str) -> bool {
        self.types.iter().any(|t| t.id == id)
    }
    pub fn ordered(&self) -> &[LinkTypeDef] {
        &self.types
    }
    /// 1-based rank in the flattened order (the materialization sort key); 0 if unknown.
    pub fn rank(&self, id: &str) -> usize {
        self.types.iter().position(|t| t.id == id).map(|i| i + 1).unwrap_or(0)
    }
    pub fn ids(&self) -> Vec<String> {
        self.types.iter().map(|t| t.id.clone()).collect()
    }
}

/// Order: top-level (parent None) by `order` then id; each immediately followed by
/// its children (parent == this) by `order` then id. v1 = one nesting level.
fn flatten_ordered(types: Vec<LinkTypeDef>) -> Vec<LinkTypeDef> {
    let mut tops: Vec<LinkTypeDef> = types.iter().filter(|t| t.parent.is_none()).cloned().collect();
    tops.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.id.cmp(&b.id)));
    let mut out = Vec::with_capacity(types.len());
    for top in tops {
        let top_id = top.id.clone();
        out.push(top);
        let mut kids: Vec<LinkTypeDef> = types
            .iter()
            .filter(|t| t.parent.as_deref() == Some(top_id.as_str()))
            .cloned()
            .collect();
        kids.sort_by(|a, b| a.order.cmp(&b.order).then_with(|| a.id.cmp(&b.id)));
        out.extend(kids);
    }
    out
}

// ─── Global active-universe registry ──────────────────────────────────
// The active universe's vocabulary. The parser + materialization read it; it's
// reloaded at boot / universe-switch / vocabulary edit. Defaults to the 8 seeds
// before any load, so first-boot + tests behave like today.

static REGISTRY: OnceLock<RwLock<LinkTypeRegistry>> = OnceLock::new();

fn cell() -> &'static RwLock<LinkTypeRegistry> {
    REGISTRY.get_or_init(|| RwLock::new(LinkTypeRegistry::seeds_only()))
}

/// True if `id` is a known type in the active registry (seeds + custom). Used by
/// the parser. Falls back to the 8 seed ids if the lock is poisoned.
pub fn is_known_type(id: &str) -> bool {
    cell()
        .read()
        .map(|r| r.is_known(id))
        .unwrap_or_else(|_| SEED_IDS.contains(&id))
}

/// Replace the active registry from user deltas (8 seeds + these).
pub fn set_active(deltas: Vec<LinkTypeDef>) {
    if let Ok(mut g) = cell().write() {
        *g = LinkTypeRegistry::merge(deltas);
    }
}

/// A clone of the active registry for off-lock reads (parser snapshot / SQL gen).
pub fn snapshot() -> LinkTypeRegistry {
    cell()
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| LinkTypeRegistry::seeds_only())
}

// ─── Per-universe persistence (the property-types.json pattern) ────────

fn link_types_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(crate::universe::active_constellation_dir(app)?.join("link-types.json"))
}

/// Read custom-type deltas from `.constellation/link-types.json`. Empty when the
/// file is absent/corrupt (a pristine or broken file ⇒ the 8 seeds — never breaks
/// the grammar).
pub fn read_deltas(app: &tauri::AppHandle) -> Vec<LinkTypeDef> {
    let Ok(path) = link_types_path(app) else { return Vec::new(); };
    let Ok(data) = std::fs::read_to_string(&path) else { return Vec::new(); };
    serde_json::from_str::<Vec<LinkTypeDef>>(&data).unwrap_or_default()
}

/// Load the active-universe registry from disk into the global static. Call at
/// boot + universe-switch. Idempotent.
pub fn load_active(app: &tauri::AppHandle) {
    set_active(read_deltas(app));
}

#[tauri::command]
pub fn read_universe_link_types(app: tauri::AppHandle) -> Result<Vec<LinkTypeDef>, String> {
    Ok(read_deltas(&app))
}

#[tauri::command]
pub fn save_universe_link_types(app: tauri::AppHandle, deltas: Vec<LinkTypeDef>) -> Result<(), String> {
    let path = link_types_path(&app)?;
    let json = serde_json::to_string_pretty(&deltas).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to save link types: {}", e))?;
    set_active(deltas); // reflect immediately
    Ok(())
}

/// The resolved registry (8 seeds + custom, ordered + nested) for the frontend.
#[tauri::command]
pub fn list_link_types(app: tauri::AppHandle) -> Result<Vec<LinkTypeDef>, String> {
    load_active(&app);
    Ok(snapshot().ordered().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom(id: &str, parent: Option<&str>, order: i64) -> LinkTypeDef {
        LinkTypeDef {
            id: id.into(), label: id.into(), parent: parent.map(|s| s.into()),
            color: "#123456".into(), order, builtin: false, emoji: None, desc: None,
        }
    }

    #[test]
    fn seeds_are_the_eight_in_canonical_order() {
        let r = LinkTypeRegistry::seeds_only();
        assert_eq!(
            r.ids(),
            vec!["supports", "contradicts", "causes", "exemplifies",
                 "generalizes", "derives-from", "part-of", "supersedes"]
        );
        assert_eq!(r.rank("supports"), 1);
        assert_eq!(r.rank("supersedes"), 8);
        assert!(r.ordered().iter().all(|t| t.builtin && t.parent.is_none()));
    }

    #[test]
    fn custom_top_level_appended_after_the_eight() {
        let r = LinkTypeRegistry::merge(vec![custom("inspires", None, 9)]);
        assert!(r.is_known("inspires"));
        assert_eq!(r.rank("inspires"), 9, "custom top-level sits after the 8");
        assert!(!r.ordered().iter().find(|t| t.id == "inspires").unwrap().builtin);
    }

    #[test]
    fn custom_child_nests_directly_under_its_parent() {
        let r = LinkTypeRegistry::merge(vec![custom("empirically-supports", Some("supports"), 1)]);
        let ids = r.ids();
        // child sits immediately after `supports`, before `contradicts`.
        let i_sup = ids.iter().position(|s| s == "supports").unwrap();
        let i_child = ids.iter().position(|s| s == "empirically-supports").unwrap();
        let i_con = ids.iter().position(|s| s == "contradicts").unwrap();
        assert_eq!(i_child, i_sup + 1);
        assert!(i_child < i_con);
    }

    #[test]
    fn seed_delta_overrides_presentation_not_grammar() {
        // a delta with id "supports" recolors it but stays builtin + top-level.
        let mut d = custom("supports", Some("contradicts"), 99);
        d.color = "#000000".into();
        let r = LinkTypeRegistry::merge(vec![d]);
        let s = r.ordered().iter().find(|t| t.id == "supports").unwrap();
        assert_eq!(s.color, "#000000", "color override applies");
        assert!(s.builtin, "still a protected seed");
        assert!(s.parent.is_none(), "a seed can't be reparented");
    }

    #[test]
    fn invalid_parent_falls_back_to_top_level() {
        let r = LinkTypeRegistry::merge(vec![custom("foo", Some("not-a-seed"), 9)]);
        assert!(r.ordered().iter().find(|t| t.id == "foo").unwrap().parent.is_none());
    }

    #[test]
    fn empty_id_ignored() {
        let r = LinkTypeRegistry::merge(vec![custom("", None, 9)]);
        assert_eq!(r.ids().len(), 8, "blank-id delta dropped; only the 8 remain");
    }
}
