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

// ─── MIG-111 §1.2/A5 — the ambient readers are GONE ──────────────────────
//
// `is_known_type` and `is_structural_type` used to answer "is this a link type?" by reading the
// process-global registry at CALL time. They were deleted, not deprecated, because a deprecated
// function still compiles: every caller now has to name the registry it means, and an eleventh
// ambient reader cannot appear without a compile error. Use `LinkTypeRegistry::is_known` /
// `::is_structural` on a registry you were HANDED — from a `WriteScope` for a routed write, or
// from `active_universe_vocabulary()` at a named, commented call site for the active universe.
//
// This is LL-047's structural fix: the question "which vocabulary?" stops being a question about
// WHEN the call happened and becomes a question about WHICH VALUE the caller was given.

/// PJ-065 — the lowercased wikilink targets declared under a STRUCTURAL (parent/TOC)
/// frontmatter key, given the note's frontmatter region (the caller passes
/// `&content[..fm_len]`). The filesystem content-scanners (strata.rs,
/// inspector360.rs) call this — with a byte-offset guard so only the frontmatter
/// occurrence is skipped — to keep a structural placement from being miscounted as a
/// cognitive outgoing link. ONE shared implementation (no divergent per-file
/// frontmatter parser — the registry DRY rule). Active since §5 — a fast no-op only
/// if no structural type is registered. Block-aware: tracks the current top-level key so `- ` list items
/// attribute to the right property (mirrors `extract_frontmatter_typed_links`).
pub fn structural_frontmatter_targets(
    reg: &LinkTypeRegistry,
    frontmatter: &str,
) -> std::collections::HashSet<String> {
    use std::sync::OnceLock;
    static WL: OnceLock<regex::Regex> = OnceLock::new();
    let wl = WL.get_or_init(|| regex::Regex::new(r"\[\[([^\[\]]+)\]\]").unwrap());
    let mut out = std::collections::HashSet::new();
    if reg.structural_ids().is_empty() {
        return out; // no structural type ⇒ nothing to skip
    }
    let mut current_structural = false;
    for line in frontmatter.lines() {
        let trimmed = line.trim_start();
        // PJ-182 — a column-0 `- "[[Chapter One]]"` is a list item under the key above.
        // Testing indentation alone made this return an EMPTY set for a zero-indent
        // `contains:` block, so the guard at `search.rs` FAILED OPEN and the structural
        // TOC targets were counted as cognitive outgoing links — while the sibling
        // extractor was simultaneously missing the real structural edge. Both directions
        // wrong on the same note.
        let scan_text: Option<&str> = if crate::yaml_lines::is_top_level_key_line(line) {
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

/// A clone of the **ACTIVE universe's** registry, for off-lock reads (parser / SQL gen).
///
/// MIG-111 §1.2/A5 renamed this from `snapshot()`. The old name described the mechanism (a
/// clone taken under the lock) and said nothing about the question it answers, which is the
/// one that matters at every call site: *whose vocabulary is this?* Every remaining caller is
/// asserting that the ACTIVE universe's answer is the right one for what it is about to do —
/// which is true for a note in the active universe and false for a note in a linked one.
pub fn active_universe_vocabulary() -> LinkTypeRegistry {
    cell()
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| LinkTypeRegistry::seeds_only())
}



// ─── Per-universe persistence (the property-types.json pattern) ────────

fn link_types_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(crate::universe::active_constellation_dir(app)?.join("link-types.json"))
}

/// The vocabulary file inside an EXPLICIT universe root — the same path
/// `link_types_path` produces for the active universe, without asking which
/// universe is active. Every routed (non-active) read and write goes through here.
pub fn link_types_file_in(universe_root: &std::path::Path) -> std::path::PathBuf {
    crate::universe::constellation_dir(universe_root).join("link-types.json")
}

/// Write vocabulary deltas to an EXPLICIT path — the production writer's body,
/// extracted so a caller (and a test) can exercise the real serialization and the
/// real atomic write without an `AppHandle` and without touching the global.
///
/// `save_universe_link_types` is this plus "and make it the active vocabulary".
/// The split matters for MIG-111: writing another universe's vocabulary must NOT
/// make it active, and a test that hand-rolls `serde_json::to_string` is testing
/// its own format rather than the one the app actually stores (LL-048).
pub(crate) fn write_link_types_at(
    path: &std::path::Path,
    deltas: &[LinkTypeDef],
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(deltas).map_err(|e| e.to_string())?;
    crate::universe::atomic_write(path, json.as_bytes())
        .map_err(|e| format!("Failed to save link types: {}", e))
}

/// MIG-111 §1.2/A1 — **the vocabulary of a universe that is not the active one.**
///
/// This is the door the routed write path needs and the codebase did not have: a
/// way to answer "what does THAT universe call its link types?" without making it
/// active. Before this, the only answers available were `snapshot()` and the three
/// ambient readers — all of which report the ACTIVE universe's vocabulary, which is
/// the wrong answer for a note that lives somewhere else (LL-047).
///
/// **Strict on purpose — it does not share `read_deltas`' fallback.** `read_deltas`
/// returns the 8 seeds for an unreadable file so a broken vocabulary can never break
/// the link grammar at boot; that is right when the alternative is an app that will
/// not start. It is wrong here. Falling back to the seeds for a *routed* write does
/// not degrade gracefully — it writes one universe's data using another universe's
/// answer, silently, with every row count still correct. That is precisely the
/// failure the harness pins. So: absent ⇒ the 8 seeds (a universe that has never
/// customized its vocabulary genuinely has the seeds); unreadable, empty, or corrupt
/// ⇒ **refuse, naming the universe** (Boss ruling 2, 2026-08-17).
pub fn registry_for_root(universe_root: &std::path::Path) -> Result<LinkTypeRegistry, String> {
    let path = link_types_file_in(universe_root);
    let deltas = crate::universe::read_persisted_json::<Vec<LinkTypeDef>>(&path)
        .map_err(|e| {
            format!(
                "Cannot read the link vocabulary of the universe \"{}\" — {}",
                universe_root
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| universe_root.display().to_string()),
                e.message()
            )
        })?
        .unwrap_or_default();
    Ok(LinkTypeRegistry::merge(deltas))
}

/// MIG-111 Stage B4 — the vocabulary for the universe that OWNS `path`.
///
/// The read-side twin of `WriteScope`'s resolution: a read that CLASSIFIES links on
/// behalf of a path that may live in a Linked Universe (the 360 Inspector's typed-act
/// scan and gap list, the strata walk, `scan_library_links`) must classify with the
/// OWNER's vocabulary, not the active one. Active owner ⇒ the in-memory active
/// registry — the same value every active-arm reader uses. Linked owner ⇒ that
/// universe's own disk, through the STRICT reader. Unknown owner, or an unreadable /
/// corrupt vocabulary ⇒ `Err` naming the universe — never a silent fall-back to the
/// active vocabulary (the misclassification this migration exists to remove), and
/// never the seeds (a guess).
pub(crate) fn registry_for_owner_of(
    app: &tauri::AppHandle,
    path: &str,
) -> Result<LinkTypeRegistry, String> {
    match crate::federation::owner::resolve_owner(app, path) {
        Ok(owner) if owner.is_active => {
            // Whose vocabulary is this? The ACTIVE universe's — the owner IS the active universe.
            Ok(active_universe_vocabulary())
        }
        Ok(owner) => registry_for_root(&owner.root),
        Err(e) => {
            // `resolve_owner` is root-containment over {active} ∪ {federation} — it cannot
            // see an OWN library registered at an EXTERNAL path (the pre-MIG-108 legacy
            // layout, still live until a universe accepts its unification proposal). Such a
            // library's vocabulary IS the active universe's — its registry lists it, which is
            // a fact, not a guess. Checked ONLY here in the Err branch: run first, the
            // prefix resolver would hand a NESTED linked universe to `universe_notes` (whose
            // path IS the active root) — the exact trap documented on `require_own_library_in`.
            // STRICT own-set read: an unreadable registry is a refusal, not an empty pass.
            let own = crate::libraries::try_load_libraries(app)
                .map_err(|le| format!("{e} (and the library registry could not be read: {le})"))?;
            if crate::libraries::owning_own_library_name_in(&own, path).is_some() {
                return Ok(active_universe_vocabulary());
            }
            Err(e)
        }
    }
}

/// MIG-111 Stage B4 — the vocabulary for a federated schema alias (`main`, `cu0`, …)
/// over the active federated connection. The per-schema readers in `cache.rs`
/// concatenate each universe's own rows; each schema's rows are classified with that
/// universe's own vocabulary. `main` is the ACTIVE universe; each `cuN` is the Linked
/// Universe attached at that alias (`SearchState.federation`), read STRICTLY from its
/// own disk. Unknown alias or unreadable vocabulary ⇒ `Err` naming it — the same
/// fail-closed rule as `registry_for_owner_of` above.
pub(crate) fn registry_for_schema(
    app: &tauri::AppHandle,
    schema: &str,
) -> Result<LinkTypeRegistry, String> {
    if schema == "main" {
        // Whose vocabulary is this? The ACTIVE universe's — `main` IS its schema.
        return Ok(active_universe_vocabulary());
    }
    let attached: Vec<(String, std::path::PathBuf)> = {
        use tauri::Manager;
        let state = app.state::<crate::search::SearchState>();
        let fed = state
            .federation
            .lock()
            .map_err(|e| format!("federation lock poisoned: {}", e))?;
        fed.attached().to_vec()
    };
    registry_for_attached_in(&attached, schema)
}

/// The decision half of `registry_for_schema`, free of `AppHandle` so the test
/// drives THIS function with the exact list `attach_all` builds (LL-048: the pure
/// function is fed the form production supplies — canonicalized roots included).
pub(crate) fn registry_for_attached_in(
    attached: &[(String, std::path::PathBuf)],
    schema: &str,
) -> Result<LinkTypeRegistry, String> {
    let root = attached
        .iter()
        .find(|(alias, _)| alias == schema)
        .map(|(_, root)| root.clone())
        .ok_or_else(|| {
            format!(
                "Unknown federated schema \"{}\" — not in the attached Linked Universe list.",
                schema
            )
        })?;
    registry_for_root(&root)
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

/// 2026-08-02 triage concern #1 — **the command must be STRICT even though `read_deltas` is
/// lenient.** `read_deltas` falls back to the 8 seeds so a broken file can never break the
/// grammar in-process; that is right for boot. It is wrong for the editor: the frontend
/// received `[]`, showed only the built-ins, and the user's next save wrote that list back
/// over their entire custom vocabulary. Surfacing the error instead means the editor can say
/// "we could not read your link types" rather than quietly presenting an empty one.
#[tauri::command]
pub fn read_universe_link_types(app: tauri::AppHandle) -> Result<Vec<LinkTypeDef>, String> {
    let path = link_types_path(&app)?;
    Ok(crate::universe::read_persisted_json::<Vec<LinkTypeDef>>(&path)?.unwrap_or_default())
}

// MIG-088 (Boss 2026-07-02, ~10s freeze on colour reset): `(async)` so the command
// runs off the IPC dispatch thread — a writer-lock wait (a background reindex/embed
// holding it) no longer freezes the UI (the PJ-066 rule: multi-second/lock-touching
// commands must be async, never SYNC on the IPC thread). The `#[tauri::command]`
// entry was the last piece of the reset path still able to block the whole app.
#[tauri::command(async)]
pub fn save_universe_link_types(app: tauri::AppHandle, deltas: Vec<LinkTypeDef>) -> Result<(), String> {
    let path = link_types_path(&app)?;
    // 2026-08-02 triage — this was a plain `fs::write`: truncate-then-write, so an
    // interruption mid-write leaves the user's link vocabulary partial, which the loader
    // then reads as "no custom types". Every other persisted-state file in the app already
    // goes through `atomic_write` (temp + fsync + rename); this one was missed. The write
    // itself now lives in `write_link_types_at` so a routed write can reuse it without
    // also making the vocabulary active (MIG-111 §1.2/A1).
    write_link_types_at(&path, &deltas)?;
    let before_fp = active_universe_vocabulary().fingerprint();
    set_active(deltas); // reflect immediately (parser + SQL generators see it now)
    // MIG-067 §B — re-materialize the outgoing-link aggregates ONLY when the VOCABULARY
    // (ordered ids) actually changed. `fingerprint()` is over ids, so a colour/label edit
    // (recolour, or "reset colours to default") leaves it identical → the triggers' rank
    // CASE + IN-list and the materialized aggregates are already correct, and we SKIP the
    // trigger recreation + backfill schedule that each grab the writer lock. This is what
    // made a colour-only save free; the earlier code claimed "no-op when unchanged" but had
    // no such guard, so every recolour needlessly took (and could wait on) the writer lock.
    if active_universe_vocabulary().fingerprint() != before_fp {
        crate::search::on_link_vocabulary_changed(&app);
    }
    Ok(())
}

/// The resolved registry (8 seeds + custom, ordered + nested) for the frontend.
///
/// **STRICT, and this is the one that matters.** The 2026-08-02 audit caught the first version
/// of this fix landing on `read_universe_link_types` — a command registered in `lib.rs` with
/// ZERO frontend callers. The editor reads through *here*, and this went through `load_active`
/// → `read_deltas`, which falls back to an empty delta list on an unreadable or corrupt file.
///
/// The loss, end to end: `link-types.json` is held for a second by a sync tool or antivirus →
/// this returns the 8 built-in seeds → the Links editor renders as though the user has no
/// custom types → the user recolours anything → `save_universe_link_types` writes the list the
/// frontend is holding → every custom link type is gone, now written atomically.
///
/// `load_active` stays lenient on purpose: at boot, falling back to the seeds means a broken
/// file can never break the link grammar. That reasoning holds for a read; it does not survive
/// a read the user will write back from. Same split as `load_registry` /
/// `load_registry_for_update` in `universe.rs`.
#[tauri::command]
pub fn list_link_types(app: tauri::AppHandle) -> Result<Vec<LinkTypeDef>, String> {
    let path = link_types_path(&app)?;
    let deltas = crate::universe::read_persisted_json::<Vec<LinkTypeDef>>(&path)?.unwrap_or_default();
    set_active(deltas);
    Ok(active_universe_vocabulary().ordered().to_vec())
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

    /// PJ-182 — the structural-link guard must see a ZERO-INDENT `contains:` block.
    ///
    /// It returned an EMPTY set for that shape, so the exclusion at `search.rs` had
    /// nothing to exclude and FAILED OPEN: a structural table-of-contents placement was
    /// counted as a cognitive outgoing link, in `note_meta.outgoing_links_json` and every
    /// view fed from it. The sibling extractor was missing the real structural edge on the
    /// very same note — both directions wrong at once.
    #[test]
    fn pj182_structural_targets_see_a_zero_indent_block() {
        let fm = "title: Part I\ncontains:\n- \"[[Chapter One]]\"\n- \"[[Chapter Two]]\"\n";
        let mut got: Vec<String> = structural_frontmatter_targets(&LinkTypeRegistry::merge(vec![custom("contains", None, 1)]), fm).into_iter().collect();
        got.sort();
        assert_eq!(got, vec!["chapter one".to_string(), "chapter two".to_string()]);

        // CONTROL — the indented form is unchanged.
        let indented = "title: Part I\ncontains:\n  - \"[[Chapter One]]\"\n  - \"[[Chapter Two]]\"\n";
        let mut got2: Vec<String> = structural_frontmatter_targets(&LinkTypeRegistry::merge(vec![custom("contains", None, 1)]), indented).into_iter().collect();
        got2.sort();
        assert_eq!(got2, got);

        // A COGNITIVE key's zero-indent block must NOT be treated as structural.
        let cognitive = "supports:\n- \"[[Alpha]]\"\n";
        assert!(structural_frontmatter_targets(&LinkTypeRegistry::merge(vec![custom("contains", None, 1)]), cognitive).is_empty());
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


    // ─── MIG-111 §1.2 / A5 — the ambient-read census ──────────────────────

    /// **Every place that still reads the ACTIVE universe's vocabulary, counted and named.**
    ///
    /// A5 deleted `is_known_type` and `is_structural_type` outright and turned
    /// `structural_frontmatter_targets` into a registry-taking function, so the parser can no
    /// longer reach the process-global at all. What remains is a set of call sites that read it
    /// *deliberately* — a backfill over the active database, DDL generated for the active
    /// database, a read-side analytic over the active database. For those, "the active
    /// universe's vocabulary" is the correct answer and the function name now says so.
    ///
    /// **This test exists because "correct answer" is a claim that decays.** Nothing stops a
    /// future edit from adding a thirty-eighth call in a routed write path, where the active
    /// universe's vocabulary is exactly the wrong answer and nothing would fail — not a type
    /// check, not a row count, not a test. That is LL-047's whole shape.
    ///
    /// So the census is pinned. Adding or removing a call site turns this red, and the fix is to
    /// look at the new site and answer the question out loud: *whose vocabulary is this, and why
    /// is the active universe's the right one here?* Then update the map. **Updating the map
    /// without answering that question is the failure this test is trying to prevent** — it costs
    /// one line and buys nothing.
    #[test]
    fn the_ambient_vocabulary_reads_are_census_ed() {
        // (file, count) — the source of truth is the source tree, not this list.
        // Each entry was looked at and asked "whose vocabulary is this?" — the answer is
        // recorded beside it. **A5/A7 grew this list on purpose:** the generators and the
        // maintenance pass now TAKE a registry, so the sites below are the callers that supply
        // one, and they say which they mean instead of the callee reaching for it.
        const CENSUS: &[(&str, usize)] = &[
            // cache.rs is ABSENT since B4: its three per-schema readers (backlink /
            // outgoing / boot-links) take a `&LinkTypeRegistry` resolved per schema by
            // `registry_for_schema` — a Linked Universe's rows are classified with its
            // OWN vocabulary, read from its own disk, and the "main" arm's active read
            // lives in that helper (counted under link_types.rs below).
            ("federation/migrate.rs", 2),      // schema migration of a foreign DB — see PJ-302
            ("federation/vocab_harness.rs", 2),// the harness asserting what the ACTIVE one is
            ("federation/write_scope.rs", 3),  // the active ARM of the scope — correct by definition
            ("incoming_links_backfill.rs", 5), // backfill over the active DB
            // inspector360.rs is ABSENT since B4 (its 2026-08-21 annotation said "B4 must
            // thread this", and B4 did): `get_360_view` accepts Linked-Universe paths, so
            // it resolves the OWNER's registry once (`registry_for_owner_of`) for both the
            // walk and the gap list.
            ("libraries.rs", 1),               // the rename rewriter's caller — **B5/B6: a rename
                                               //   inside a Linked Universe needs the OWNER's.**
                                               //   (Was 2: `scan_links_recursive` was threaded by
                                               //   B4 — owner-resolved, once per walk, ending its
                                               //   per-directory re-read.)
            ("link_types.rs", 7),              // the registry's own lock plumbing (4), plus the
                                               //   B4 resolvers' ACTIVE arms: `registry_for_owner_of`
                                               //   (owner IS the active universe — 2 arms: the
                                               //   resolve_owner hit, and the pre-MIG-108 legacy
                                               //   external-own-library fallback, whose membership
                                               //   is read STRICTLY from the own registry) and
                                               //   `registry_for_schema` ("main" IS the active
                                               //   schema) — each correct by definition, and the
                                               //   single place the active answer enters a
                                               //   scope-resolving read.
            ("links_backfill.rs", 8),          // backfill + trigger DDL over the active DB.
                                               //   (+1 in B4: `recompute_sky_range` now reads
                                               //   explicitly what the expr generators used to
                                               //   read for it invisibly. **B1 threads the
                                               //   recompute_* functions to their callers'
                                               //   pinned scope.**)
            // name_fold_backfill.rs is DELIBERATELY ABSENT. It used to be here with 1. The
            // 2026-08-21 safety inspection found that its connection is pinned to one universe's
            // search.db while its vocabulary was read from the global eighty lines later — a
            // switch in that window recomputed one universe's aggregates with another's rank CASE
            // and then stamped the module complete. It now uses `registry_for_root` on the same
            // root it opened, so it reads the global not at all — and since B4 the stratum /
            // maturity generators take that same pinned `&vocab`, so the last hidden global
            // read inside this module's call graph is gone. **This is the shape every entry
            // above should eventually reach.**
            ("search.rs", 16),                 // trigger DDL + backfills + the index tail.
                                               //   B4: the stratum/maturity generators no longer
                                               //   read the global; each caller answers at its own
                                               //   line — the DDL + PJ-334 restore arms are
                                               //   `owns`-gated (PJ-232) so the active answer is
                                               //   right there (B2 threads the DDL layer); the
                                               //   save/de-index tails hoist ONE read shared by
                                               //   their incoming+sky pair.
                                               //   **Phase 1.3 must revisit the index tail:
                                               //   `maintain_incoming_after_save`'s caller is on
                                               //   the write path a routed note will travel.**
            ("sight.rs", 1),                   // active by CONSTRUCTION: no path parameter, reads
                                               //   only `state.db` under one lock. Federated Sight
                                               //   is the reserved MIG-063 family.
            ("sky_backfill.rs", 1),            // active universe's own DB (PJ-332 pinned conn);
                                               //   B4 made it ONE read per batch shared by the
                                               //   sx + stratum/maturity (was three moments).
                                               //   **B1 threads it from the pinned root.**
            // strata.rs is ABSENT since B4: `compute_note_strata` is called for EVERY
            // federated library by the Sky enrichment loop, so it resolves the OWNER's
            // registry once per walk (`registry_for_owner_of`).
            ("tension.rs", 1),                 // active by SCOPE since B4: `detect_tensions` now
                                               //   genuinely refuses non-own libraries (the
                                               //   refusal its comment had always claimed), so
                                               //   the rows are always the active universe's own.
        ];

        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut actual: Vec<(String, usize)> = Vec::new();
        let mut stack = vec![src.clone()];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
                    let Ok(text) = std::fs::read_to_string(&p) else { continue };
                    let n = text
                        .lines()
                        // the definition itself is not a call site
                        .filter(|l| !l.contains(concat!("pub fn active_universe_", "vocabulary")))
                        // The needle is assembled so this file does not textually contain it —
                        // otherwise the census counts its own matcher and can never agree.
                        .map(|l| l.matches(concat!("active_universe_", "vocabulary()")).count())
                        .sum::<usize>();
                    if n > 0 {
                        let rel = p
                            .strip_prefix(&src)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/");
                        actual.push((rel, n));
                    }
                }
            }
        }
        actual.sort();
        let expected: Vec<(String, usize)> =
            CENSUS.iter().map(|(f, n)| (f.to_string(), *n)).collect();

        assert_eq!(
            actual, expected,
            "\n\nThe set of places reading the ACTIVE universe's vocabulary changed.\n\
             Before updating the list above, answer this at the new call site:\n\
             **whose vocabulary is this, and why is the active universe's the right one here?**\n\
             If the code runs on behalf of a note that might live in a Linked Universe, the answer \
             is NO — take a `&LinkTypeRegistry` (from a `WriteScope`) instead.\n"
        );
    }

    // ─── MIG-111 §1.2 / A1 — the routed vocabulary door ───────────────────
    //
    // These run over REAL directories, and the file they read is written by the
    // production writer (`write_link_types_at`, the body of the save command), not
    // by a hand-rolled literal. A test that writes its own JSON proves its own
    // format round-trips; it says nothing about the format the app stores (LL-048).

    fn tmp_universe(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "constellation_lt_a1_{}_{}",
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(crate::universe::constellation_dir(&d)).expect("tmp universe");
        d
    }

    #[test]
    fn a_universe_that_never_customized_its_vocabulary_has_the_seeds() {
        let root = tmp_universe("absent");
        let reg = registry_for_root(&root).expect("absent file is not an error");
        assert_eq!(
            reg.ids(),
            LinkTypeRegistry::seeds_only().ids(),
            "no link-types.json means the universe genuinely has the seeds —              the ONE case where falling back is the truth and not a guess"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn registry_for_root_reads_back_what_the_production_writer_wrote() {
        let root = tmp_universe("roundtrip");
        let path = link_types_file_in(&root);
        write_link_types_at(&path, &[custom("refutes", None, 9)]).expect("write");

        let reg = registry_for_root(&root).expect("read back");
        assert!(reg.is_known("refutes"), "the custom type survives the real writer's format");
        assert!(reg.is_known("supports"), "and the seeds are still there");
        assert!(
            !LinkTypeRegistry::seeds_only().is_known("refutes"),
            "guard: `refutes` is genuinely custom, so this test cannot pass on the seeds alone"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    // ─── MIG-111 Stage B4 — per-schema resolution over the attached list ──────
    //
    // The clause layer cannot prove "each universe's own types": A2 pins the
    // structural lane to {contains, parent} in every constructible registry, so
    // `structural_not_in_clause` is registry-invariant and two universes' SQL is
    // byte-identical. The observable difference between vocabularies is their
    // CUSTOM types — so that is the marker these tests use, and the proof lands
    // at the resolution layer: a cu-alias resolves to the CHILD's own disk.

    #[test]
    fn a_cu_alias_resolves_to_that_linked_universes_own_vocabulary() {
        let child_root = tmp_universe("attached_child");
        write_link_types_at(&link_types_file_in(&child_root), &[custom("refutes", None, 9)])
            .expect("write child vocab via the production writer");
        // LL-048 — the exact form `attach_all` builds: `unique_cuniverse_roots`
        // canonicalizes, so the attached list holds the OS's form, not a hand-built one.
        let attached = vec![(
            "cu0".to_string(),
            std::fs::canonicalize(&child_root).expect("canonicalize child root"),
        )];

        let reg = registry_for_attached_in(&attached, "cu0").expect("resolve cu0");
        assert!(
            reg.is_known("refutes"),
            "cu0 must resolve to the CHILD's own disk — its custom type is the marker"
        );
        assert!(
            !LinkTypeRegistry::seeds_only().is_known("refutes"),
            "guard: the marker is genuinely custom, so this cannot pass on the seeds alone"
        );
        let _ = std::fs::remove_dir_all(&child_root);
    }

    #[test]
    fn an_unknown_schema_alias_refuses_instead_of_guessing() {
        let err = registry_for_attached_in(&[], "cu7")
            .expect_err("an alias with no attached root cannot resolve to ANY vocabulary");
        assert!(err.contains("cu7"), "the refusal names the alias; got: {err}");
    }

    #[test]
    fn a_cu_alias_whose_vocabulary_is_corrupt_refuses_naming_the_universe() {
        let child_root = tmp_universe("attached_corrupt");
        std::fs::write(link_types_file_in(&child_root), b"[{\"id\": ").expect("write partial");
        let attached = vec![(
            "cu0".to_string(),
            std::fs::canonicalize(&child_root).expect("canonicalize"),
        )];
        let err = registry_for_attached_in(&attached, "cu0")
            .expect_err("corrupt is a refusal, never a silent fallback to the active vocabulary");
        assert!(
            err.contains(child_root.file_name().unwrap().to_str().unwrap()),
            "the refusal must name the universe; got: {err}"
        );
        let _ = std::fs::remove_dir_all(&child_root);
    }

    /// The three refusals. Each asserts the message NAMES the universe (Boss ruling 2,
    /// 2026-08-17): a routed write that cannot read its vocabulary must say WHICH
    /// universe it could not read, because the user has several and the parent is not
    /// the one at fault.
    #[test]
    fn registry_for_root_refuses_a_truncated_file() {
        let root = tmp_universe("empty");
        std::fs::write(link_types_file_in(&root), b"").expect("write empty");
        let err = registry_for_root(&root).expect_err("a zero-length file is not 'no custom types'");
        assert!(
            err.contains(root.file_name().unwrap().to_str().unwrap()),
            "the refusal must name the universe; got: {err}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn registry_for_root_refuses_corrupt_json() {
        let root = tmp_universe("corrupt");
        std::fs::write(link_types_file_in(&root), b"[{\"id\": ").expect("write partial");
        let err = registry_for_root(&root).expect_err("half a JSON array is not an empty one");
        assert!(
            err.contains(root.file_name().unwrap().to_str().unwrap()),
            "the refusal must name the universe; got: {err}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn registry_for_root_refuses_a_file_it_cannot_read() {
        // A directory where the file should be: an I/O error that is NOT NotFound, on
        // both Windows (access denied) and Unix (is-a-directory) — the shape a sync
        // tool or antivirus holding the file produces, without needing to hold one.
        let root = tmp_universe("unreadable");
        std::fs::create_dir_all(link_types_file_in(&root)).expect("dir in the file's place");
        let err = registry_for_root(&root).expect_err("unreadable is not empty");
        assert!(
            err.contains(root.file_name().unwrap().to_str().unwrap()),
            "the refusal must name the universe; got: {err}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    // ─── MIG-111 §1.2 / A2 — the merge invariance pin ─────────────────────

    /// **A declared contract, not a property of a function body.** Everything the
    /// cognitive surfaces exclude is derived from the `structural` flag, and `merge`
    /// is the only place that flag is decided: a custom type is forced cognitive, a
    /// cognitive seed is forced non-structural, and only the two structural seeds
    /// come out structural — no matter what the deltas on disk claim.
    ///
    /// This matters for MIG-111 specifically: a linked universe's `link-types.json`
    /// is a file the parent did not write and does not control. If a delta could
    /// flip `structural`, a child's vocabulary file could make a *cognitive* type
    /// invisible to every maturity / strata / tension / sky query the parent runs
    /// over it. It cannot — and this test goes red the day that stops being true.
    #[test]
    fn merge_decides_the_structural_lane_no_matter_what_the_deltas_claim() {
        let structural_delta = |id: &str, order: i64| LinkTypeDef {
            id: id.into(),
            label: id.into(),
            parent: None,
            color: "#123456".into(),
            order,
            builtin: false,
            emoji: None,
            desc: None,
            structural: true, // the claim under test
        };
        let mut cognitive_seed_claiming_structural = structural_delta("supports", 1);
        cognitive_seed_claiming_structural.builtin = true;
        let mut structural_seed_claiming_cognitive = structural_delta("parent", 2);
        structural_seed_claiming_cognitive.structural = false;

        let reg = LinkTypeRegistry::merge(vec![
            custom("refutes", None, 9),                 // a plain custom type
            structural_delta("toc-like", 10),           // a custom type CLAIMING structural
            cognitive_seed_claiming_structural,         // a seed override claiming structural
            structural_seed_claiming_cognitive,         // a structural seed claiming cognitive
        ]);

        let mut structural = reg.structural_ids();
        structural.sort();
        assert_eq!(
            structural,
            vec!["contains".to_string(), "parent".to_string()],
            "only the two structural seeds are structural — a delta cannot add to or              remove from the lane"
        );
        assert!(reg.is_known("toc-like"), "the custom type is still ADDED, just not structural");
        assert!(
            reg.cognitive_ids().contains(&"toc-like".to_string()),
            "a custom type is cognitive regardless of what it claims"
        );
        assert!(
            reg.cognitive_ids().contains(&"supports".to_string()),
            "a seed override cannot hide a cognitive seed from the cognitive surfaces"
        );
    }
}
