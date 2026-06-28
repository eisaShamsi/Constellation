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
    /// PJ-065 — `true` for the structural (parent/TOC) lane: a NON-cognitive
    /// compositional-spine relation, locked like a seed but EXCLUDED from the
    /// cognitive apparatus (maturity/strata/360/centrality/tension/health/sky)
    /// and from the cognitive UI enumerators. Default `false`, so every
    /// pre-PJ-065 def (the 8 seeds + each custom type, and any old
    /// `link-types.json`) deserializes unchanged.
    #[serde(default)]
    pub structural: bool,
}

/// The 8 canonical seed ids, in derived order (Concept Paper §7). `associative`
/// is the null/untyped synonym — NOT a type you add, handled separately.
pub const SEED_IDS: &[&str] = &[
    "supports", "contradicts", "causes", "exemplifies",
    "generalizes", "derives-from", "part-of", "supersedes",
];

/// PJ-065 — the structural (parent/TOC) seed ids: a NON-cognitive lane, locked
/// like the 8 but flagged `structural`. **Registered by §5: `parent` /
/// `contains`** — the safe-order rule: every cognitive-exclusion filter installed
/// in §3/§4 derives from the `structural` flag and is a no-op until these ids
/// exist, so registration (which makes `is_known_type` true and starts edge
/// emission) lands LAST, after the blinders are all in place.
pub const STRUCTURAL_SEED_IDS: &[&str] = &["parent", "contains"];

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
        structural: false,
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
        // PJ-065 — the structural (parent/TOC) lane: the compositional spine, a
        // distinct NON-cognitive KIND (NOT a 9th act, NOT the cognitive `part-of`).
        // Locked like the 8 but flagged `structural`, so every cognitive subsystem
        // excludes it (no weight / confidence / decay / topology). Teal #14B8A6
        // (distinct from all 8). Two inverse faces: `contains` (parent→child, carries
        // the `seq` order) + `parent` (child→parent). Ordered after the 8, own group.
        LinkTypeDef { id: "contains".to_string(), label: "Contains".to_string(), parent: None, color: "#14B8A6".to_string(), order: 9, builtin: true, emoji: None, desc: Some("Structural: the ordered children of this work (table of contents)".to_string()), structural: true },
        LinkTypeDef { id: "parent".to_string(), label: "Parent".to_string(), parent: None, color: "#14B8A6".to_string(), order: 10, builtin: true, emoji: None, desc: Some("Structural: this note's place under a parent work".to_string()), structural: true },
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
        // PJ-065 — the structural lane lock set (registered since §5).
        let structural_set: std::collections::HashSet<&str> =
            STRUCTURAL_SEED_IDS.iter().copied().collect();
        let mut by_id: std::collections::BTreeMap<String, LinkTypeDef> =
            seeds().into_iter().map(|d| (d.id.clone(), d)).collect();
        for mut d in deltas {
            d.id = sanitize_id(&d.id);
            if d.id.is_empty() {
                continue;
            }
            d.parent = d.parent.map(|p| sanitize_id(&p)).filter(|p| !p.is_empty());
            if seed_set.contains(d.id.as_str()) {
                d.builtin = true;
                d.parent = None;
                d.structural = false; // cognitive seed — never structural
                if let Some(seed) = by_id.get(&d.id) {
                    if d.label.trim().is_empty() {
                        d.label = seed.label.clone();
                    }
                    if d.desc.is_none() {
                        d.desc = seed.desc.clone();
                    }
                }
            } else if structural_set.contains(d.id.as_str()) {
                // PJ-065 — structural seed: locked like the 8 (id/existence/parent
                // immutable) but flagged `structural` (the non-cognitive parent/TOC
                // lane). Coerces a stray `{id:'parent', structural:false}` delta back
                // to the locked structural form. Active since §5 (would be dormant
                // only if STRUCTURAL_SEED_IDS were empty).
                d.builtin = true;
                d.parent = None;
                d.structural = true;
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
                d.structural = false; // custom types are cognitive
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
    /// True if `id` is a recognized stored `note_links.link_type` value — a typed
    /// act in the registry OR the null/default `associative`. The analytics
    /// surfaces (strata / tension / libraries link-type filters) historically
    /// counted `associative` alongside the 8, so they use this (not `is_known`,
    /// which is the 8 typed acts only) to stay byte-identical while still picking
    /// up custom types. Snapshot once per call site, then check in the loop — no
    /// per-link lock. MIG-067 §D.
    pub fn is_link_type_value(&self, id: &str) -> bool {
        id == "associative" || self.is_known(id)
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

    /// SQL `(...)` membership list of all known type ids (single-quote-escaped).
    /// Ids are slug-sanitized at merge, so this is injection-safe.
    pub fn sql_in_list(&self) -> String {
        let parts: Vec<String> = self
            .types
            .iter()
            .map(|t| format!("'{}'", t.id.replace('\'', "''")))
            .collect();
        format!("({})", parts.join(","))
    }

    /// SQL `CASE link_type WHEN 'id' THEN <rank> ... END` — 1-based flattened rank
    /// (canonical order; custom types after/under the 8). The materialization sort key.
    pub fn sql_rank_case(&self) -> String {
        let mut s = String::from("CASE link_type ");
        for (i, t) in self.types.iter().enumerate() {
            s.push_str(&format!("WHEN '{}' THEN {} ", t.id.replace('\'', "''"), i + 1));
        }
        s.push_str("END");
        s
    }

    // ─── PJ-065 — the structural-lane partition (cognitive vs structural) ──────
    // Every cognitive-scoring query reads the COGNITIVE variant so the structural
    // (parent/TOC) lane is invisible to maturity/strata/360/centrality/tension/
    // health/sky. All return "all known types" while no structural type exists
    // before §5; §5 registered the lane, so the §3/§4 filters are now live.

    /// Ids flagged structural (the parent/TOC lane). Registered since §5.
    pub fn structural_ids(&self) -> Vec<String> {
        self.types.iter().filter(|t| t.structural).map(|t| t.id.clone()).collect()
    }

    /// True if `id` is a structural (non-cognitive) type in this registry.
    pub fn is_structural(&self, id: &str) -> bool {
        self.types.iter().any(|t| t.id == id && t.structural)
    }

    /// Cognitive ids = every known type that is NOT structural (the 8 + customs).
    pub fn cognitive_ids(&self) -> Vec<String> {
        self.types.iter().filter(|t| !t.structural).map(|t| t.id.clone()).collect()
    }

    /// SQL `(...)` membership of COGNITIVE ids only (structural excluded) — for the
    /// aggregate breakdown filters that must ignore the structural lane. Identical
    /// to `sql_in_list` while no structural type exists.
    pub fn sql_in_list_cognitive(&self) -> String {
        let parts: Vec<String> = self
            .types
            .iter()
            .filter(|t| !t.structural)
            .map(|t| format!("'{}'", t.id.replace('\'', "''")))
            .collect();
        format!("({})", parts.join(","))
    }

    /// Rank CASE over COGNITIVE ids only (1-based, canonical order; structural
    /// excluded). Identical to `sql_rank_case` while no structural type exists.
    pub fn sql_rank_case_cognitive(&self) -> String {
        let mut s = String::from("CASE link_type ");
        let mut i = 0;
        for t in self.types.iter().filter(|t| !t.structural) {
            i += 1;
            s.push_str(&format!("WHEN '{}' THEN {} ", t.id.replace('\'', "''"), i));
        }
        s.push_str("END");
        s
    }

    /// The no-op-safe structural exclusion fragment — the single chokepoint every
    /// cognitive count/edge query appends to drop the structural lane. Returns `""`
    /// when no structural type exists (so callers append NOTHING — never the
    /// SQL-error `NOT IN ()`), else ` AND <col> NOT IN ('parent','contains')`.
    pub fn structural_not_in_clause(&self, col: &str) -> String {
        let ids: Vec<String> = self
            .types
            .iter()
            .filter(|t| t.structural)
            .map(|t| format!("'{}'", t.id.replace('\'', "''")))
            .collect();
        if ids.is_empty() {
            String::new()
        } else {
            format!(" AND {} NOT IN ({})", col, ids.join(","))
        }
    }

    /// Sentinel rank when a note has no canonical typed links = `len()+1`.
    pub fn sentinel_rank(&self) -> usize {
        self.types.len() + 1
    }

    /// PJ-065 — sentinel for the COGNITIVE top-rank (no cognitive typed link) =
    /// cognitive count + 1. Distinct from `sentinel_rank` (which counts ALL types,
    /// incl. the structural lane) so the cognitive aggregates' "no typed link"
    /// sentinel stays stable (8 cognitive → 9) regardless of how many structural
    /// types exist. The cognitive aggregates use THIS.
    pub fn cognitive_sentinel_rank(&self) -> usize {
        self.cognitive_ids().len() + 1
    }

    /// Version-stable fingerprint of the vocabulary (ordered ids) — gates the
    /// re-materialization of `note_meta` when the vocabulary changes. FNV-1a so it
    /// is identical across binary upgrades (a hash that drifts would force a
    /// spurious recompute on every update).
    pub fn fingerprint(&self) -> i64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for t in &self.types {
            for b in t.id.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
            h ^= 0xff; // id separator
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        (h >> 1) as i64 // 63-bit, fits i64 positively
    }
}

/// Reduce a user id to a safe slug: lowercase ASCII alphanumerics + hyphen.
/// Ids flow into generated SQL (the IN-list + rank CASE) and into stored
/// `note_links.link_type` + `note.link.<id>` column names, so this is the
/// defensive floor (the §G editor slugifies on input).
fn sanitize_id(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
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

/// PJ-065 — true if `id` is a structural (non-cognitive parent/TOC) type in the
/// active registry. Falls back to `STRUCTURAL_SEED_IDS` on a poisoned lock.
/// (TS mirror: linkTypeRegistry.ts::isStructuralLinkType.)
pub fn is_structural_type(id: &str) -> bool {
    cell()
        .read()
        .map(|r| r.is_structural(id))
        .unwrap_or_else(|_| STRUCTURAL_SEED_IDS.contains(&id))
}

/// PJ-065 — the lowercased wikilink targets declared under a STRUCTURAL (parent/TOC)
/// frontmatter key, given the note's frontmatter region (the caller passes
/// `&content[..fm_len]`). The filesystem content-scanners (strata.rs,
/// inspector360.rs) call this — with a byte-offset guard so only the frontmatter
/// occurrence is skipped — to keep a structural placement from being miscounted as a
/// cognitive outgoing link. ONE shared implementation (no divergent per-file
/// frontmatter parser — the registry DRY rule). Active since §5 — a fast no-op only
/// if no structural type is registered. Block-aware: tracks the current top-level key so `- ` list items
/// attribute to the right property (mirrors `extract_frontmatter_typed_links`).
pub fn structural_frontmatter_targets(frontmatter: &str) -> std::collections::HashSet<String> {
    use std::sync::OnceLock;
    static WL: OnceLock<regex::Regex> = OnceLock::new();
    let wl = WL.get_or_init(|| regex::Regex::new(r"\[\[([^\[\]]+)\]\]").unwrap());
    let mut out = std::collections::HashSet::new();
    let reg = snapshot();
    if reg.structural_ids().is_empty() {
        return out; // no structural type ⇒ nothing to skip
    }
    let mut current_structural = false;
    for line in frontmatter.lines() {
        let trimmed = line.trim_start();
        let is_indented = line.len() != trimmed.len();
        let scan_text: Option<&str> = if !is_indented {
            current_structural = false;
            match trimmed.find(':') {
                Some(colon) if reg.is_structural(&trimmed[..colon].trim().to_lowercase()) => {
                    current_structural = true;
                    Some(&trimmed[colon + 1..])
                }
                _ => None,
            }
        } else if current_structural {
            Some(trimmed)
        } else {
            None
        };
        if let Some(text) = scan_text {
            for cap in wl.captures_iter(text) {
                let inner = cap.get(1).map_or("", |m| m.as_str());
                let (before, after) = match inner.split_once('|') {
                    Some((b, a)) => (b, Some(a)),
                    None => (inner, None),
                };
                let (target, _) = resolve_wikilink_type(&reg, before, after, true);
                if !target.is_empty() {
                    out.insert(target.to_lowercase());
                }
            }
        }
    }
    out
}

/// Resolve `(target, Option<type>)` from a wikilink's regex capture groups —
/// `before_pipe` = group 1 (text before any `|`), `after_pipe` = group 2 (the
/// optional alias / legacy type after `|`). Understands BOTH link orders:
///   - predicate-FIRST  `[[type::target]]`  → type from the `type::` prefix on
///     `before_pipe`, target = the remainder;
///   - predicate-LAST   `[[target|type]]`   → type from `after_pipe`, target =
///     `before_pipe`.
/// `include_associative` chooses the membership: `true` accepts the null
/// `associative` alongside the typed acts (the Tension / Strata / library scans,
/// which historically counted it); `false` recognizes only the 8 cognitive acts
/// (the 360.3D matrix, where `associative` is "untyped"). Unknown / absent ⇒
/// untyped, target = `before_pipe`. Target keeps its case; callers lowercase.
///
/// MIG-067 — the single predicate-first-aware parser the content re-readers share
/// so they can never drift from the indexed `note_links` again (the §A switch to
/// `[[type::target]]` left these re-readers parsing the old order → everything
/// read as untyped; this is the fix).
pub fn resolve_wikilink_type(
    reg: &LinkTypeRegistry,
    before_pipe: &str,
    after_pipe: Option<&str>,
    include_associative: bool,
) -> (String, Option<String>) {
    let known = |c: &str| {
        if include_associative { reg.is_link_type_value(c) } else { reg.is_known(c) }
    };
    // Predicate-first: a known `type::` prefix on the pre-pipe segment.
    if let Some(idx) = before_pipe.find("::") {
        let candidate = before_pipe[..idx].trim().to_lowercase();
        if known(&candidate) {
            return (before_pipe[idx + 2..].trim().to_string(), Some(candidate));
        }
    }
    // Predicate-last: a known type in the alias slot. Also accepts the legacy
    // explicit `type:` prefix (`[[note|type:supports]]`) that strata/libraries
    // recognized, so folding them onto this helper loses no old form.
    if let Some(a) = after_pipe {
        let raw = a.trim().to_lowercase();
        let lower = raw.strip_prefix("type:").map(|r| r.trim().to_string()).unwrap_or(raw);
        if known(&lower) {
            return (before_pipe.trim().to_string(), Some(lower));
        }
    }
    (before_pipe.trim().to_string(), None)
}

/// Replace the active registry from user deltas (8 seeds + these).
pub fn set_active(deltas: Vec<LinkTypeDef>) {
    if let Ok(mut g) = cell().write() {
        *g = LinkTypeRegistry::merge(deltas);
    }
}

/// MIG-075 follow-up — the ONE definition of "a null type": ids that mean
/// untyped / the open question rather than a typed cognitive act.
/// `associative` is the canonical null id (MIG-067), `relates` the legacy
/// one, `""` the defensive empty. Callers decide what null MEANS for them
/// (default weight, untyped tint, …); membership is defined here once.
/// (TS mirror: linkTypeRegistry.ts::isNullLinkType.)
pub fn is_null_type(id: &str) -> bool {
    matches!(id, "associative" | "relates" | "")
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
    set_active(deltas); // reflect immediately (parser + SQL generators see it now)
    // MIG-067 §B — re-materialize the outgoing-link aggregates under the new
    // vocabulary: recreate the triggers (so live edge writes use the new rank) and
    // schedule the background re-materialize of existing rows (gated, batched,
    // never blocks). No-op cost when the vocabulary did not actually change.
    crate::search::on_link_vocabulary_changed(&app);
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
            structural: false,
        }
    }

    #[test]
    fn seeds_are_the_eight_cognitive_then_the_structural_lane() {
        let r = LinkTypeRegistry::seeds_only();
        // The 8 cognitive acts in canonical order, then the PJ-065 structural lane.
        assert_eq!(
            r.ids(),
            vec!["supports", "contradicts", "causes", "exemplifies",
                 "generalizes", "derives-from", "part-of", "supersedes",
                 "contains", "parent"]
        );
        assert_eq!(r.rank("supports"), 1);
        assert_eq!(r.rank("supersedes"), 8);
        assert_eq!(r.rank("contains"), 9);
        assert_eq!(r.rank("parent"), 10);
        assert!(r.ordered().iter().all(|t| t.builtin && t.parent.is_none()));
        // The 8 cognitive ids exclude the structural lane; structural_ids() is exactly it.
        assert_eq!(r.cognitive_ids().len(), 8);
        assert_eq!(r.structural_ids(), vec!["contains".to_string(), "parent".to_string()]);
        assert!(r.is_structural("parent") && r.is_structural("contains"));
        assert!(!r.is_structural("part-of"), "cognitive part-of is NOT structural");
        // The exclusion clause is non-empty now (and never the SQL-error NOT IN ()).
        assert_eq!(r.structural_not_in_clause("link_type"), " AND link_type NOT IN ('contains','parent')");
    }

    #[test]
    fn structural_seed_delta_is_coerced_back_to_locked_structural() {
        // A stray delta trying to un-structural / un-lock `parent` is coerced back.
        let r = LinkTypeRegistry::merge(vec![LinkTypeDef {
            id: "parent".into(), label: "Mine".into(), parent: Some("supports".into()),
            color: "#000000".into(), order: 1, builtin: false, emoji: None, desc: None,
            structural: false,
        }]);
        let p = r.ordered().iter().find(|t| t.id == "parent").expect("parent present");
        assert!(p.builtin && p.structural && p.parent.is_none(),
            "structural seed stays locked + structural + top-level");
    }

    #[test]
    fn resolve_wikilink_type_handles_both_orders() {
        let reg = LinkTypeRegistry::seeds_only();
        let s = |x: &str| Some(x.to_string());
        // predicate-FIRST (the canonical form; the Boss-reported supersedes case)
        assert_eq!(resolve_wikilink_type(&reg, "supersedes::apple", None, false), ("apple".into(), s("supersedes")));
        assert_eq!(resolve_wikilink_type(&reg, "supports::apple", None, false), ("apple".into(), s("supports")));
        // predicate-LAST (bare alias) + the legacy `type:` prefix
        assert_eq!(resolve_wikilink_type(&reg, "apple", Some("supports"), false), ("apple".into(), s("supports")));
        assert_eq!(resolve_wikilink_type(&reg, "apple", Some("type:supports"), false), ("apple".into(), s("supports")));
        // untyped: no alias, or a display alias that isn't a type
        assert_eq!(resolve_wikilink_type(&reg, "apple", None, false), ("apple".into(), None));
        assert_eq!(resolve_wikilink_type(&reg, "apple", Some("My Note"), false), ("apple".into(), None));
        // a real "::" in a note name (unknown prefix) is left intact
        assert_eq!(resolve_wikilink_type(&reg, "C++::vector", None, false), ("C++::vector".into(), None));
        // associative: untyped for the matrix (false), a type for the analytics (true)
        assert_eq!(resolve_wikilink_type(&reg, "associative::apple", None, false), ("associative::apple".into(), None));
        assert_eq!(resolve_wikilink_type(&reg, "associative::apple", None, true), ("apple".into(), s("associative")));
    }

    #[test]
    fn custom_top_level_appended_after_the_eight() {
        let r = LinkTypeRegistry::merge(vec![custom("inspires", None, 11)]);
        assert!(r.is_known("inspires"));
        // After the 8 cognitive (1-8) AND the 2 structural seeds (contains=9, parent=10).
        assert_eq!(r.rank("inspires"), 11, "custom top-level sits after the 8 cognitive + structural lane");
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
        assert_eq!(r.ids().len(), 10, "blank-id delta dropped; the 8 cognitive + 2 structural seeds remain");
    }
}
