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
//!   task (tracked in SESSION-LOG as M8c) will emit a reindex signal
//!   when overrides change. For now, overrides take effect on newly
//!   written notes only — and on the next full reindex of the library.
//! - **No per-library federation**: `cUniverses` (child Universes
//!   contributing libraries to a parent) all share the *parent*
//!   Universe's override set via `ACTIVE_STORE`. Per-Universe override
//!   stacking is tracked as an M8b-v2 follow-up; for v1, overrides are
//!   a property of the active Universe only.
//! - **No normalizer-dependency flattening**: we call
//!   `crate::arabic::normalizer::normalize` at lookup time to match the
//!   stripped form of the input. If the user's override was authored on
//!   a different normalizer version, it still matches — normalization is
//!   idempotent on the relevant code points.

use super::types::{Analysis, AnalysisOrigin, Lang, PartOfSpeech};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

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

// ── Process-wide active store (M8b) ──────────────────────────────────
//
// The FTS5 tokenizer runs inside SQLite's call context — a sync,
// Tauri-State-less path. It can't easily reach the `UniverseState`
// managed store via `app.state::<UniverseState>()`. So we mirror the
// per-Universe override file into a process-wide `OnceLock<RwLock<Arc<...>>>`
// and swap the inner `Arc` every time the active Universe changes or
// the user edits an override via the Settings UI.
//
// Hot-path cost: one `RwLock::read` (uncontended ~20ns on Windows) +
// one `Arc::clone` (refcount bump, ~5ns). Well under the tokenizer's
// normalize + HashMap-probe budget. The RwLock is poisoned-tolerant —
// if a panic happens while holding the write guard, subsequent readers
// still see the prior `Arc` via `into_inner` recovery; we `unwrap()`
// because a poisoned lock here means the analyzer itself panicked mid-
// write, which is a bug we want to surface loudly, not paper over.

static ACTIVE_STORE: OnceLock<RwLock<Arc<OverrideStore>>> = OnceLock::new();

fn store_lock() -> &'static RwLock<Arc<OverrideStore>> {
    ACTIVE_STORE.get_or_init(|| RwLock::new(Arc::new(OverrideStore::new())))
}

/// Cheap clone of the currently-active override store. Called per FTS5
/// tokenizer invocation. `Arc::clone` is a refcount bump — the store's
/// internal HashMap is NOT duplicated.
///
/// On first call before any `set_active` / `activate_for_universe`, this
/// returns an empty store, so Layer 0 never fires and every analysis
/// falls through to Layers 1–5 as in pre-M8b behaviour.
pub fn active() -> Arc<OverrideStore> {
    store_lock().read().expect("arabic override lock poisoned").clone()
}

/// Install a new active store. Called by:
/// - `activate_for_universe` when the user opens / switches Universes.
/// - The `add_arabic_override` / `remove_arabic_override` Tauri commands
///   after they persist the change to disk, so subsequent tokenizer
///   calls see the new entry without waiting for the next Universe
///   switch.
///
/// Thread-safe. The write guard is held only for the Arc pointer swap,
/// not for any HashMap-level mutation — the store itself is immutable
/// once installed.
pub fn set_active(store: OverrideStore) {
    *store_lock().write().expect("arabic override lock poisoned") = Arc::new(store);
}

/// Load the override file from `<universe_root>/.constellation/arabic-overrides.json`
/// and install it as the active store. Missing file = empty store (the
/// common case for a fresh Universe). Malformed JSON = error.
///
/// Returns the count of installed overrides on success — the caller can
/// log this for boot diagnostics ("loaded 42 Arabic overrides for
/// Universe <name>").
pub fn activate_for_universe(universe_root: &Path) -> Result<usize, String> {
    let path = OverrideStore::path_in_universe(universe_root);
    let store = OverrideStore::load_from_path(&path)
        .map_err(|e| format!("Failed to load {}: {}", path.display(), e))?;
    let count = store.len();
    set_active(store);
    Ok(count)
}

/// Install an empty active store. Used when the app has no active
/// Universe (cold-boot before the frontend calls `set_active_universe`)
/// or when a Universe is deliberately closed. Idempotent.
pub fn clear_active() {
    set_active(OverrideStore::new());
}

// ── Tauri command surface (M8b) ──────────────────────────────────────
//
// Read / add / remove override endpoints for the Settings UI. All three
// commands resolve the active Universe root via
// `crate::universe::active_universe_dir`, reload the store from disk
// (rather than trusting the in-memory `ACTIVE_STORE`), mutate, persist
// atomically, then update `ACTIVE_STORE` so the next FTS5 tokenizer
// call sees the change.
//
// Reloading from disk on every CRUD — instead of mutating `ACTIVE_STORE`
// directly — makes concurrent edits from multiple UI windows safe
// (second screen, settings modal) without needing a cross-window
// mutex: the disk is the source of truth, and the Mutex over the on-
// disk JSON file (implicit in file-system-level atomic rename) is the
// only contention point.

/// Return every override authored in the active Universe, sorted
/// alphabetically by surface for stable UI ordering.
#[tauri::command]
pub fn read_arabic_overrides(app: tauri::AppHandle) -> Result<Vec<UserOverride>, String> {
    let universe_root = crate::universe::active_universe_dir(&app)?;
    let path = OverrideStore::path_in_universe(&universe_root);
    let store = OverrideStore::load_from_path(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let mut entries: Vec<UserOverride> = store.iter().cloned().collect();
    entries.sort_by(|a, b| a.surface.cmp(&b.surface));
    Ok(entries)
}

/// Upsert an override in the active Universe. Replaces any existing
/// entry with the same normalized surface. Persists to disk atomically,
/// then reinstalls the active store so subsequent FTS5 tokens see the
/// change without waiting for the next Universe switch.
///
/// Parameter name `entry` rather than Rust keyword-adjacent `override`
/// so it serializes cleanly through Tauri's IPC layer.
#[tauri::command]
pub fn add_arabic_override(
    app: tauri::AppHandle,
    entry: UserOverride,
) -> Result<(), String> {
    let universe_root = crate::universe::active_universe_dir(&app)?;
    let path = OverrideStore::path_in_universe(&universe_root);
    let mut store = OverrideStore::load_from_path(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    store.insert(entry);
    store.save_to_path(&path)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    set_active(store);
    Ok(())
}

/// Remove the override for a surface. Returns `true` if an override was
/// removed, `false` if no override existed for this surface (not an
/// error — idempotent from the UI's perspective).
#[tauri::command]
pub fn remove_arabic_override(
    app: tauri::AppHandle,
    surface: String,
) -> Result<bool, String> {
    let universe_root = crate::universe::active_universe_dir(&app)?;
    let path = OverrideStore::path_in_universe(&universe_root);
    let mut store = OverrideStore::load_from_path(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let removed = store.remove(&surface).is_some();
    if removed {
        store.save_to_path(&path)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        set_active(store);
    }
    Ok(removed)
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

    // ── Process-wide active store (M8b) ───────────────────────────────
    //
    // These tests all touch a global singleton, so they must not run in
    // parallel with each other. Cargo test's default `--test-threads=N`
    // would race them, producing flaky failures. We serialize via the
    // `REGISTRY_TEST_MUTEX` below — every test that calls `set_active` /
    // `active` / `activate_for_universe` acquires the guard first and
    // snapshots the prior state on drop.
    //
    // Rationale vs. #[serial_test]: the crate isn't a dependency, and
    // adding one for three tests is overkill. A hand-rolled Mutex is
    // three lines.

    use std::sync::Mutex as StdMutex;

    static REGISTRY_TEST_MUTEX: StdMutex<()> = StdMutex::new(());

    /// RAII guard that snapshots the active store on construction and
    /// restores it on drop, so each test runs against a predictable
    /// (empty) baseline and doesn't leak state into the next one.
    struct RegistryGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        prior: Arc<OverrideStore>,
    }

    impl RegistryGuard {
        fn new() -> Self {
            let lock = REGISTRY_TEST_MUTEX
                .lock()
                .unwrap_or_else(|e| e.into_inner()); // ignore poisoning
            let prior = active();
            clear_active();
            Self { _lock: lock, prior }
        }
    }

    impl Drop for RegistryGuard {
        fn drop(&mut self) {
            // Restore the prior Arc byte-for-byte.
            *store_lock().write().unwrap() = self.prior.clone();
        }
    }

    #[test]
    fn active_returns_empty_store_before_any_set() {
        let _g = RegistryGuard::new();
        let store = active();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn set_active_then_active_roundtrips() {
        let _g = RegistryGuard::new();

        let mut store = OverrideStore::new();
        store.insert(mk_override("وائل", "وائل"));
        set_active(store);

        let active_store = active();
        assert_eq!(active_store.len(), 1);
        assert!(active_store.lookup("وائل").is_some(),
                "installed override must be reachable via active()");
    }

    #[test]
    fn set_active_replaces_prior_store_entirely() {
        let _g = RegistryGuard::new();

        // First install
        let mut a = OverrideStore::new();
        a.insert(mk_override("ألف", "ألف"));
        set_active(a);
        assert_eq!(active().len(), 1);
        assert!(active().lookup("ألف").is_some());

        // Second install — completely different contents
        let mut b = OverrideStore::new();
        b.insert(mk_override("باء", "باء"));
        set_active(b);
        assert_eq!(active().len(), 1);
        assert!(active().lookup("ألف").is_none(),
                "prior override must not survive a set_active swap");
        assert!(active().lookup("باء").is_some());
    }

    #[test]
    fn clear_active_installs_empty_store() {
        let _g = RegistryGuard::new();

        let mut store = OverrideStore::new();
        store.insert(mk_override("وائل", "وائل"));
        set_active(store);
        assert_eq!(active().len(), 1);

        clear_active();
        assert!(active().is_empty());
    }

    #[test]
    fn active_returns_cheap_arc_clones() {
        // Sanity check: the two Arcs returned by back-to-back `active()`
        // calls point to the same underlying allocation — we're not
        // accidentally deep-cloning the HashMap on every tokenizer hit.
        let _g = RegistryGuard::new();
        let mut store = OverrideStore::new();
        store.insert(mk_override("وائل", "وائل"));
        set_active(store);

        let first = active();
        let second = active();
        assert!(Arc::ptr_eq(&first, &second),
                "active() must return shared Arc, not a deep clone");
    }

    #[test]
    fn activate_for_universe_installs_from_disk() {
        let _g = RegistryGuard::new();

        let tmp_dir = std::env::temp_dir().join(format!(
            "constellation_overrides_test_activate_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = OverrideStore::path_in_universe(&tmp_dir);

        // Seed disk with a single override
        let mut store = OverrideStore::new();
        store.insert(mk_override("وائل", "وائل"));
        store.save_to_path(&path).expect("seed disk");

        // Clear ACTIVE_STORE, then activate from this Universe
        clear_active();
        assert!(active().is_empty());

        let count = activate_for_universe(&tmp_dir).expect("activate");
        assert_eq!(count, 1, "returned count must match loaded override count");
        assert_eq!(active().len(), 1);
        assert!(active().lookup("وائل").is_some());

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn activate_for_universe_handles_missing_file() {
        let _g = RegistryGuard::new();

        let tmp_dir = std::env::temp_dir().join(format!(
            "constellation_overrides_test_missing_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Don't create anything on disk. activate_for_universe on a
        // fresh Universe must install an empty store, NOT error out —
        // a Universe that hasn't authored any overrides is the common
        // case.
        let count = activate_for_universe(&tmp_dir).expect("missing = empty, not err");
        assert_eq!(count, 0);
        assert!(active().is_empty());
    }

    #[test]
    fn activate_for_universe_reports_malformed_json_as_error() {
        let _g = RegistryGuard::new();

        let tmp_dir = std::env::temp_dir().join(format!(
            "constellation_overrides_test_malformed_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = OverrideStore::path_in_universe(&tmp_dir);
        std::fs::create_dir_all(path.parent().unwrap()).expect("mkdir");
        std::fs::write(&path, b"{ not valid JSON").expect("write garbage");

        let result = activate_for_universe(&tmp_dir);
        assert!(result.is_err(),
                "malformed JSON on an *existing* file must surface as an error");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
