//! Layer 0 — user overrides (per-Universe learning).
//!
//! The sovereign layer. When the user explicitly pins the analysis of a
//! surface ("treat خليفة as a ProperNoun, not a verbal noun"), that choice
//! must win over everything else the engine would otherwise infer — the
//! protected list, the generative FST, the cascade, the heuristic
//! fallback, all of it. This module is the ingress: a simple, persistent,
//! per-Universe HashMap from normalized-surface to a user-authored
//! `UserOverride` record.
//!
//! ## Where this layer sits in the five-layer pipeline
//!
//! ```text
//! Input surface
//!    ↓
//! [L0 user override]    ← THIS MODULE. If hit, return immediately.
//!    ↓
//! [L1 normalizer]       ← tashkeel removal, script detection
//!    ↓
//! [L2 protected list]   ← proper nouns, places, loanwords, function words
//!    ↓
//! [L3 generative FST]   ← root × pattern combinations
//!    ↓
//! [L3b cascade]         ← affix-peel and retry L3
//!    ↓
//! [L4 disambiguator]    ← rank remaining candidates
//!    ↓
//! [L5 heuristic]        ← last-resort surface fallback
//! ```
//!
//! Per the 2026-04-18 spec, L0 runs on the **normalized** surface — same
//! input the protected list sees — so a user override for `وائل` also
//! catches `وَائِل` (fully vocalized) without requiring two entries.
//!
//! ## Storage
//!
//! One JSON file per Universe:
//! `<universe>/.constellation/arabic-overrides.json`.
//!
//! Rationale for JSON (not SQLite, not TSV):
//! - Human-inspectable. Users can hand-edit with a plain-text editor if
//!   they want, no schema migrations to worry about.
//! - Small. A few hundred overrides fit easily in a few KB.
//! - Round-trips through `serde` cleanly — no custom parser needed.
//! - Git-friendly. If the user syncs their Universe with Git/Syncthing,
//!   overrides diff cleanly.
//!
//! Schema (forward-compatible; unknown fields are ignored on load):
//! ```json
//! {
//!   "version": 1,
//!   "overrides": [
//!     {
//!       "surface": "خليفة",
//!       "lemma": "خليفة",
//!       "root": "خ-ل-ف",
//!       "pattern_label": "user:ProperNoun",
//!       "pos": "ProperNoun",
//!       "note": "Caliph name, not a verbal noun",
//!       "created_at": "2026-04-18T10:30:00Z"
//!     }
//!   ]
//! }
//! ```
//!
//! ## What this module does NOT do (today)
//!
//! - **No FTS re-index on change**: if the user edits an override, the
//!   existing FTS rows still hold the pre-override stem. A forthcoming
//!   task (tracked in SESSION-LOG as M8b) will emit a reindex signal
//!   when overrides change. For now, overrides take effect on newly
//!   written notes only.
//! - **No Tauri command surface**: the `#[tauri::command]`-decorated
//!   CRUD endpoints that the Settings UI will call live in a separate
//!   forthcoming module. This layer is the pure data / lookup core.
//! - **No normalizer-dependency flattening**: we call
//!   `crate::arabic::normalizer::normalize` at lookup time to match the
//!   stripped form of the input. If the user's override was authored on
//!   a different normalizer version, it still matches — normalization is
//!   idempotent on the relevant code points.

use super::types::{Analysis, AnalysisOrigin, Lang, PartOfSpeech};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One user-authored override.
///
/// `surface` is the user's verbatim input, preserved for display/export.
/// The engine keys lookups on the *normalized* form (stripped) which is
/// computed on load; that's what the `OverrideStore`'s internal HashMap
/// is indexed by.
///
/// `lemma`, `root`, `pattern_label`, `pos` mirror the `Analysis` fields
/// they seed — that way `to_analysis` below is a thin copy, no translation
/// layer.
///
/// `note` is an optional free-text explanation the user can attach to
/// remind themselves *why* this override exists. Never consumed by the
/// engine; purely a user-memory aid, shown in the Settings UI.
///
/// `created_at` is ISO-8601 UTC. Purely informational; the engine never
/// sorts by it or reads it back. A human-readable timestamp is easier
/// for the user to make sense of than a Unix epoch integer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserOverride {
    pub surface: String,
    pub lemma: String,
    #[serde(default)]
    pub root: String,
    #[serde(default = "default_pattern_label")]
    pub pattern_label: String,
    #[serde(default = "default_pos")]
    pub pos: PartOfSpeech,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub created_at: String,
}

fn default_pattern_label() -> String {
    "user:override".to_string()
}

fn default_pos() -> PartOfSpeech {
    PartOfSpeech::Unknown
}

impl UserOverride {
    /// Render this override as an `Analysis`. Called by `analyze()` when
    /// the Layer 0 lookup hits. Confidence is pinned at 1.0 — user intent
    /// is the highest-authority signal we have.
    pub fn to_analysis(&self, original_surface: &str) -> Analysis {
        Analysis {
            surface: original_surface.to_string(),
            lemma: self.lemma.clone(),
            root: self.root.clone(),
            pattern_label: self.pattern_label.clone(),
            pos: self.pos,
            prefixes: Vec::new(),
            suffixes: Vec::new(),
            confidence: 1.0,
            origin: AnalysisOrigin::UserOverride,
            equivalents: HashMap::new(),
            lang: Lang::Ar,
        }
    }
}

/// The on-disk JSON envelope. Wraps the override list plus a version
/// integer so future schema changes can migrate gracefully.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct OverrideFile {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default)]
    overrides: Vec<UserOverride>,
}

fn default_version() -> u32 {
    1
}

/// In-memory override store for a single Universe.
///
/// Construction:
/// - `OverrideStore::new()` — empty, for tests or brand-new Universes.
/// - `OverrideStore::load_from_path(&p)` — parse the JSON file. Returns
///   `Ok(empty)` if the file doesn't exist (a fresh Universe hasn't
///   authored any overrides yet), `Err` only on malformed JSON or I/O
///   errors on an *existing* file.
///
/// Lookup:
/// - `OverrideStore::lookup(&norm_surface)` — O(1) HashMap hit on the
///   pre-normalized surface. This is what `analyze_with_overrides` calls.
///
/// Mutation:
/// - `OverrideStore::insert(o)` — upsert (replace on duplicate surface).
/// - `OverrideStore::remove(&surface)` — remove by verbatim surface;
///   returns the removed record for audit.
/// - `OverrideStore::save_to_path(&p)` — atomic write via `.tmp` +
///   rename (no partial-file poisoning on crash).
#[derive(Debug, Default, Clone)]
pub struct OverrideStore {
    /// Key = normalized surface (from `normalizer::normalize(...).stripped`).
    /// Values own the full `UserOverride`; there's no need for lifetimes
    /// here because the store is always held by the `analyze()` call
    /// site at the layer above.
    entries: HashMap<String, UserOverride>,
}

impl OverrideStore {
    /// New empty store. Test/fixture constructor and the return value
    /// from `load_from_path` when the file doesn't exist yet.
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Number of overrides in the store. Useful for the Settings UI
    /// "N overrides configured" indicator.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no overrides are configured. Handy short-circuit for
    /// `analyze_with_overrides` to skip the HashMap probe entirely.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate all overrides. Order is unspecified (HashMap iteration);
    /// callers that need a stable order must sort the result.
    pub fn iter(&self) -> impl Iterator<Item = &UserOverride> {
        self.entries.values()
    }

    /// Look up by *normalized* surface.
    ///
    /// The caller is responsible for running the normalizer on the user
    /// input first — this keeps the hot path cheap (one HashMap probe,
    /// no re-normalization inside the store). `analyze_with_overrides`
    /// does this correctly.
    pub fn lookup(&self, normalized_surface: &str) -> Option<&UserOverride> {
        self.entries.get(normalized_surface)
    }

    /// Upsert an override. The record's `surface` is normalized via the
    /// engine's normalizer to produce the HashMap key, so a later lookup
    /// on either the raw surface or the normalized form finds it.
    ///
    /// Returns the previous value if one was replaced (HashMap::insert
    /// semantics), so callers can audit "this edit overwrote an existing
    /// override authored at {created_at}".
    pub fn insert(&mut self, override_: UserOverride) -> Option<UserOverride> {
        let key = Self::normalize_key(&override_.surface);
        self.entries.insert(key, override_)
    }

    /// Remove by verbatim surface. Returns the removed record, or None
    /// if no override existed for this surface.
    pub fn remove(&mut self, surface: &str) -> Option<UserOverride> {
        let key = Self::normalize_key(surface);
        self.entries.remove(&key)
    }

    /// The canonical path for a Universe's override file. Separate helper
    /// so Tauri command handlers and tests agree on the location.
    pub fn path_in_universe(universe_dir: &Path) -> PathBuf {
        universe_dir.join(".constellation").join("arabic-overrides.json")
    }

    /// Load from disk. A missing file yields an empty store (this is the
    /// common case for a freshly-created Universe); only true I/O or
    /// parse errors bubble up.
    pub fn load_from_path(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let bytes = std::fs::read(path)?;
        let file: OverrideFile = serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut store = Self::new();
        for o in file.overrides {
            store.insert(o);
        }
        Ok(store)
    }

    /// Atomic write: stage to `.tmp`, rename on success. No partial-file
    /// poisoning on crash mid-write. Creates parent directories as needed.
    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Stable iteration order for the on-disk JSON so diffs stay small
        // when the user edits with Git: sort by surface.
        let mut overrides: Vec<UserOverride> = self.entries.values().cloned().collect();
        overrides.sort_by(|a, b| a.surface.cmp(&b.surface));
        let file = OverrideFile { version: 1, overrides };
        let json = serde_json::to_vec_pretty(&file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Internal: normalize a surface into the HashMap lookup key.
    ///
    /// Calls the engine's normalizer so that the key computed at write
    /// time (`insert`) matches the key computed at read time
    /// (`analyze_with_overrides`). If we ever swap normalizer versions
    /// in a way that changes the stripped output, old override keys in
    /// memory remain valid; on next disk reload the keys are recomputed
    /// from the stored `surface` field — so a normalizer bump is a
    /// zero-user-intervention migration.
    fn normalize_key(surface: &str) -> String {
        super::normalizer::normalize(surface).stripped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_override(surface: &str, lemma: &str) -> UserOverride {
        UserOverride {
            surface: surface.to_string(),
            lemma: lemma.to_string(),
            root: "خ-ل-ف".to_string(),
            pattern_label: "user:ProperNoun".to_string(),
            pos: PartOfSpeech::ProperNoun,
            note: "test override".to_string(),
            created_at: "2026-04-18T00:00:00Z".to_string(),
        }
    }

    // ── UserOverride → Analysis shape ────────────────────────────────

    #[test]
    fn to_analysis_produces_user_override_origin() {
        let o = mk_override("خليفة", "خليفة");
        let a = o.to_analysis("خليفة");
        assert!(matches!(a.origin, AnalysisOrigin::UserOverride));
        assert_eq!(a.confidence, 1.0);
        assert_eq!(a.lemma, "خليفة");
        assert_eq!(a.root, "خ-ل-ف");
        assert_eq!(a.pos, PartOfSpeech::ProperNoun);
    }

    #[test]
    fn to_analysis_preserves_original_surface() {
        // The surface field holds the user's verbatim input (pre-
        // normalization), not the stored surface from the override.
        let o = mk_override("خليفة", "خليفة");
        let a = o.to_analysis("خَلِيفَة");
        assert_eq!(a.surface, "خَلِيفَة", "surface round-trips verbatim");
    }

    // ── OverrideStore lookup ─────────────────────────────────────────

    #[test]
    fn empty_store_finds_nothing() {
        let store = OverrideStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.lookup("خليفة").is_none());
    }

    #[test]
    fn insert_then_lookup_roundtrip() {
        let mut store = OverrideStore::new();
        store.insert(mk_override("خليفة", "خليفة"));
        assert_eq!(store.len(), 1);
        let norm = super::super::normalizer::normalize("خليفة").stripped;
        let found = store.lookup(&norm).expect("must find override");
        assert_eq!(found.lemma, "خليفة");
    }

    #[test]
    fn vocalized_surface_finds_bare_override() {
        // Critical semantic contract: an override authored for `وائل`
        // must also catch `وَائِل` (fully vocalized). The normalizer
        // strips tashkeel so both collapse to the same HashMap key.
        let mut store = OverrideStore::new();
        store.insert(UserOverride {
            surface: "وائل".to_string(),
            lemma: "وائل".to_string(),
            root: String::new(),
            pattern_label: "user:ProperNoun".to_string(),
            pos: PartOfSpeech::ProperNoun,
            note: String::new(),
            created_at: String::new(),
        });
        let vocalized = super::super::normalizer::normalize("وَائِل").stripped;
        assert!(
            store.lookup(&vocalized).is_some(),
            "vocalized surface must hit bare override via normalizer"
        );
    }

    #[test]
    fn insert_replaces_on_duplicate_surface() {
        let mut store = OverrideStore::new();
        store.insert(mk_override("خليفة", "first"));
        let prev = store.insert(mk_override("خليفة", "second"));
        assert!(prev.is_some(), "replaced value must be returned");
        assert_eq!(prev.unwrap().lemma, "first");
        let norm = super::super::normalizer::normalize("خليفة").stripped;
        assert_eq!(store.lookup(&norm).unwrap().lemma, "second");
    }

    #[test]
    fn remove_returns_the_removed_override() {
        let mut store = OverrideStore::new();
        store.insert(mk_override("خليفة", "خليفة"));
        let removed = store.remove("خليفة");
        assert!(removed.is_some());
        assert!(store.is_empty());
    }

    #[test]
    fn remove_nonexistent_returns_none() {
        let mut store = OverrideStore::new();
        assert!(store.remove("does-not-exist").is_none());
    }

    #[test]
    fn iter_exposes_all_entries() {
        let mut store = OverrideStore::new();
        store.insert(mk_override("خليفة", "خليفة"));
        store.insert(mk_override("أبو بكر", "أبو بكر"));
        let lemmas: Vec<String> = store.iter().map(|o| o.lemma.clone()).collect();
        assert_eq!(lemmas.len(), 2);
    }

    // ── persistence ───────────────────────────────────────────────────

    #[test]
    fn load_missing_file_yields_empty_store() {
        let tmp = std::env::temp_dir().join("constellation-overrides-missing.json");
        let _ = std::fs::remove_file(&tmp); // ensure missing
        let store = OverrideStore::load_from_path(&tmp).expect("load must not error on missing");
        assert!(store.is_empty());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let tmp_dir = std::env::temp_dir().join("constellation-overrides-roundtrip");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let path = tmp_dir.join(".constellation").join("arabic-overrides.json");

        let mut store = OverrideStore::new();
        store.insert(mk_override("خليفة", "خليفة"));
        store.insert(mk_override("أبو بكر", "أبو بكر"));
        store.save_to_path(&path).expect("save must succeed");

        let reloaded = OverrideStore::load_from_path(&path).expect("reload must succeed");
        assert_eq!(reloaded.len(), 2);
        let norm = super::super::normalizer::normalize("خليفة").stripped;
        assert_eq!(reloaded.lookup(&norm).unwrap().lemma, "خليفة");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn load_rejects_malformed_json() {
        let tmp = std::env::temp_dir().join("constellation-overrides-malformed.json");
        std::fs::write(&tmp, b"{not valid json").expect("write test fixture");
        let err = OverrideStore::load_from_path(&tmp).expect_err("malformed must error");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn load_tolerates_unknown_fields_in_json() {
        // Forward-compat: future schema versions adding fields must not
        // break older Constellation builds that read the same file. serde
        // ignores unknown fields by default; this test pins that contract.
        let tmp = std::env::temp_dir().join("constellation-overrides-unknown-fields.json");
        let json = r#"{
            "version": 2,
            "future_field": "ignored",
            "overrides": [
                {
                    "surface": "خليفة",
                    "lemma": "خليفة",
                    "root": "خ-ل-ف",
                    "pattern_label": "user:ProperNoun",
                    "pos": "ProperNoun",
                    "note": "",
                    "created_at": "",
                    "future_per_entry_field": 42
                }
            ]
        }"#;
        std::fs::write(&tmp, json).expect("write fixture");
        let store = OverrideStore::load_from_path(&tmp).expect("load must tolerate unknown fields");
        assert_eq!(store.len(), 1);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn save_is_atomic_no_leftover_tmp_on_success() {
        let tmp_dir = std::env::temp_dir().join("constellation-overrides-atomic");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let path = tmp_dir.join(".constellation").join("arabic-overrides.json");

        let mut store = OverrideStore::new();
        store.insert(mk_override("خليفة", "خليفة"));
        store.save_to_path(&path).expect("save");

        // On success the .tmp must be renamed-not-copied; no leftover.
        let tmp_path = path.with_extension("json.tmp");
        assert!(!tmp_path.exists(), "no leftover .tmp file after atomic rename");
        assert!(path.exists(), "target file exists");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn save_sorts_entries_alphabetically_for_git_friendly_diffs() {
        let tmp_dir = std::env::temp_dir().join("constellation-overrides-sorted");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let path = tmp_dir.join(".constellation").join("arabic-overrides.json");

        let mut store = OverrideStore::new();
        // Insert out of alphabetic order.
        store.insert(mk_override("ياء", "ياء"));
        store.insert(mk_override("ألف", "ألف"));
        store.insert(mk_override("ميم", "ميم"));
        store.save_to_path(&path).expect("save");

        let contents = std::fs::read_to_string(&path).expect("read back");
        let alif = contents.find("ألف").expect("ألف present");
        let mim = contents.find("ميم").expect("ميم present");
        let ya = contents.find("ياء").expect("ياء present");
        assert!(alif < mim && mim < ya, "alphabetic order: ألف < ميم < ياء");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn path_in_universe_points_to_dot_constellation_subdir() {
        let universe = std::path::PathBuf::from("/tmp/MyUniverse");
        let p = OverrideStore::path_in_universe(&universe);
        assert!(p.ends_with(".constellation/arabic-overrides.json") ||
                p.ends_with(r".constellation\arabic-overrides.json"),
                "path must land in .constellation/, got {:?}", p);
    }
}
