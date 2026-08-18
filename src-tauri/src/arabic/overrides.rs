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
//! ## Federation — layered stores (M8b-v2)
//!
//! A Universe may declare `cUniverse` children (see `UniverseMeta::children`
//! in `universe.rs`) — linked Universes whose libraries surface in the
//! active Universe's federated view. Each of those child Universes may
//! have authored its own `arabic-overrides.json`. The active store is
//! therefore a **stack of layers**:
//!
//! ```text
//! layers[0]   = active Universe (sovereign — user's own overrides)
//! layers[1..] = cUniverse children, in declaration order
//! ```
//!
//! `lookup` walks the stack in order and returns on the first hit —
//! **the parent's override always wins** on conflict. This matches the
//! sovereignty semantics in CLAUDE.md's Knowledge Formulation principles:
//! the user's intent in the active Universe is the highest-authority
//! signal. Children contribute for surfaces the parent hasn't overridden.
//!
//! CRUD (`insert` / `remove` / `save_to_path`) touches **only** the
//! sovereign layer. Child layers are read-only from the active Universe's
//! perspective — each child owns its own `arabic-overrides.json`, edited
//! when that child is the active Universe.
//!
//! A non-federated Universe (no `children`) produces a one-layer stack —
//! byte-for-byte the same hot-path behaviour as pre-M8b-v2.
//!
//! ## What this module does NOT do (today)
//!
//! - **No FTS re-index on change**: if the user edits an override, the
//!   existing FTS rows still hold the pre-override stem. A forthcoming
//!   task (tracked in SESSION-LOG as M8c) will emit a reindex signal
//!   when overrides change. For now, overrides take effect on newly
//!   written notes only — and on the next full reindex of the library.
//! - **No normalizer-dependency flattening**: we call
//!   `crate::arabic::normalizer::normalize` at lookup time to match the
//!   stripped form of the input. If the user's override was authored on
//!   a different normalizer version, it still matches — normalization is
//!   idempotent on the relevant code points.

use super::types::{Analysis, AnalysisOrigin, Lang, PartOfSpeech};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
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

/// In-memory override store for a single Universe, optionally layered
/// across its cUniverse children.
///
/// Internal shape is a **stack** of layers:
///
/// - `layers[0]`    — the active (sovereign) Universe's overrides
/// - `layers[1..]`  — cUniverse children's overrides, in declaration order
///
/// A non-federated Universe (no `UniverseMeta::children`) produces a
/// one-entry stack that behaves byte-for-byte like the pre-M8b-v2
/// single-layer store. An uninitialized store (before any Universe is
/// activated, or after `clear_active`) has `layers.is_empty()` — this
/// is a valid "no overrides anywhere" state and short-circuits cleanly
/// via `is_empty()`.
///
/// Construction:
/// - `OverrideStore::new()` — empty (zero layers), for tests or brand-
///   new Universes.
/// - `OverrideStore::load_from_path(&p)` — single-layer load from one
///   file. Returns an empty (zero-layer) store if the file doesn't exist.
/// - `OverrideStore::from_layered_paths(&[parent, children...])` — load
///   multiple files into a layered stack. Used by the Universe switcher
///   to combine parent + cUniverse children's overrides.
///
/// Lookup:
/// - `OverrideStore::lookup(&norm_surface)` — walks layers in order;
///   the first hit wins. Parent always wins on conflict.
///
/// Mutation (sovereign-only — child layers are read-only from the
/// active Universe's perspective):
/// - `OverrideStore::insert(o)` — upsert into `layers[0]` (creates it
///   if the stack was empty).
/// - `OverrideStore::remove(&surface)` — remove from `layers[0]` only.
/// - `OverrideStore::save_to_path(&p)` — atomic write of the sovereign
///   layer only, via `.tmp` + rename. Children are not touched.
#[derive(Debug, Default, Clone)]
pub struct OverrideStore {
    /// One HashMap per layer. Key = normalized surface (from
    /// `normalizer::normalize(...).stripped`). `layers[0]` is the active
    /// Universe's sovereign layer; `layers[1..]` are cUniverse children
    /// probed after the parent misses.
    ///
    /// Invariant: every layer's keys are normalized consistently via
    /// `normalize_key` so that a surface authored in any layer can be
    /// found by a lookup using any equivalent normalized form.
    layers: Vec<HashMap<String, UserOverride>>,
}

impl OverrideStore {
    /// New empty store with zero layers. Returned from `load_from_path`
    /// on a missing file, from `new()` for tests / fixtures, and from
    /// `clear_active` when the active Universe is closed.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Total number of overrides across **all** layers. Useful for boot
    /// diagnostics ("loaded N overrides across K Universes") and for the
    /// Settings UI summary count.
    pub fn len(&self) -> usize {
        self.layers.iter().map(|l| l.len()).sum()
    }

    /// True when no overrides are configured in any layer. Handy short-
    /// circuit for `analyze_with_overrides` to skip the layer walk
    /// entirely when both the parent and every child have no entries.
    pub fn is_empty(&self) -> bool {
        self.layers.iter().all(|l| l.is_empty())
    }

    /// Iterate every override across all layers (sovereign first, then
    /// children in declaration order). Order within a single layer is
    /// unspecified (HashMap iteration); callers that need a stable total
    /// order must sort the result.
    ///
    /// If you only want the editable (sovereign) entries — e.g. the
    /// Settings UI listing — prefer `sovereign_iter` instead, so you
    /// don't show the user child-Universe entries they can't modify
    /// from this Universe's editor.
    pub fn iter(&self) -> impl Iterator<Item = &UserOverride> {
        self.layers.iter().flat_map(|l| l.values())
    }

    /// Iterate only the sovereign (parent's own) overrides — the entries
    /// backed by the active Universe's own `arabic-overrides.json`. The
    /// Settings UI editing flow uses this to avoid listing read-only
    /// child-Universe entries.
    pub fn sovereign_iter(&self) -> impl Iterator<Item = &UserOverride> {
        self.layers.first().into_iter().flat_map(|l| l.values())
    }

    /// Number of layers currently installed. Diagnostic only; the hot
    /// path does not branch on this. Useful for the Settings UI to
    /// render "parent + N child Universe contributions".
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Look up by *normalized* surface. Walks layers in order — parent
    /// wins on conflict.
    ///
    /// The caller is responsible for running the normalizer on the user
    /// input first — this keeps the hot path cheap (one HashMap probe
    /// per layer, no re-normalization inside the store).
    /// `analyze_with_overrides` does this correctly.
    ///
    /// Typical layer count today is 1–5 (parent + a handful of
    /// children). With a small Vec the loop is branch-prediction-friendly
    /// and each missing probe is a single hash + bucket check. The hot-
    /// path cost is dominated by the single `Arc::clone` at the call
    /// site (~5ns on Windows), not the layer walk.
    pub fn lookup(&self, normalized_surface: &str) -> Option<&UserOverride> {
        for layer in &self.layers {
            if let Some(v) = layer.get(normalized_surface) {
                return Some(v);
            }
        }
        None
    }

    /// Upsert an override into the sovereign layer. Creates `layers[0]`
    /// if the stack was empty. The record's `surface` is normalized via
    /// the engine's normalizer to produce the HashMap key, so a later
    /// lookup on either the raw surface or a vocalized form finds it.
    ///
    /// Returns the previous value if one was replaced in the sovereign
    /// layer (HashMap::insert semantics). A duplicate surface in a child
    /// layer is NOT overwritten — children own their own files.
    pub fn insert(&mut self, override_: UserOverride) -> Option<UserOverride> {
        if self.layers.is_empty() {
            self.layers.push(HashMap::new());
        }
        let key = Self::normalize_key(&override_.surface);
        self.layers[0].insert(key, override_)
    }

    /// Remove by verbatim surface from the sovereign layer only. Returns
    /// the removed record, or None if no sovereign override existed for
    /// this surface (even if a child layer has one — child entries are
    /// read-only from the parent's perspective).
    pub fn remove(&mut self, surface: &str) -> Option<UserOverride> {
        if self.layers.is_empty() {
            return None;
        }
        let key = Self::normalize_key(surface);
        self.layers[0].remove(&key)
    }

    /// The canonical path for a Universe's override file. Separate helper
    /// so Tauri command handlers and tests agree on the location.
    pub fn path_in_universe(universe_dir: &Path) -> PathBuf {
        universe_dir.join(".constellation").join("arabic-overrides.json")
    }

    /// Load a single-layer store from one file. A missing file yields
    /// a zero-layer store (this is the common case for a freshly-created
    /// Universe that hasn't authored any overrides yet); only true I/O
    /// or parse errors bubble up.
    ///
    /// Used by the CRUD Tauri commands, which always operate on a single
    /// file (the active Universe's own overrides).
    pub fn load_from_path(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let layer = read_layer(path)?;
        Ok(Self { layers: vec![layer] })
    }

    /// Build a layered store from an ordered list of paths. `paths[0]`
    /// is the sovereign (active Universe's) override file; `paths[1..]`
    /// are cUniverse children's override files probed after a parent
    /// miss, in the order given.
    ///
    /// Missing files become empty layers — a fresh child Universe that
    /// hasn't authored overrides contributes nothing but does not abort
    /// the load. An existing-but-malformed file propagates the parse
    /// error (callers can log it and install an empty store as fallback).
    pub fn from_layered_paths(paths: &[PathBuf]) -> std::io::Result<Self> {
        let mut layers = Vec::with_capacity(paths.len());
        for p in paths {
            let layer = if p.exists() {
                read_layer(p)?
            } else {
                HashMap::new()
            };
            layers.push(layer);
        }
        Ok(Self { layers })
    }

    /// Atomic write of the sovereign (parent's) layer only. Children
    /// are never written through the parent's path — each child owns
    /// its own `arabic-overrides.json`, edited when that child is the
    /// active Universe.
    ///
    /// Writes through `universe::atomic_write` — unique temp, fsync, then rename.
    /// Creates parent directories as needed.
    pub fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Stable iteration order for the on-disk JSON so diffs stay small
        // when the user edits with Git: sort by surface.
        let sovereign = self.layers.first().cloned().unwrap_or_default();
        let mut overrides: Vec<UserOverride> = sovereign.values().cloned().collect();
        overrides.sort_by(|a, b| a.surface.cmp(&b.surface));
        let file = OverrideFile { version: 1, overrides };
        let json = serde_json::to_vec_pretty(&file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        // PJ-207 §15 — this was the last persisted-STATE writer still doing a bare
        // write+rename. Two defects, both already named in `universe::atomic_write`'s own
        // comment two files away: no fsync before the rename, so power loss can commit the
        // rename while the data blocks are still unflushed and publish zeros under the FINAL
        // name; and a FIXED `.json.tmp`, which two writers of the same file share, letting one
        // publish the other's half-written bytes.
        //
        // Why it mattered here in particular: `arabic-overrides.json` is the ONLY on-disk
        // record of these overrides — nothing rebuilds them. And the loader's failure path
        // (`activate_layered_for_universe`, universe.rs) catches the parse error, prints an
        // `eprintln!` a release build has no console for, and installs an EMPTY store. So the
        // loss never surfaces as an error; it surfaces as Arabic search quietly tokenizing
        // differently than the user taught it to.
        crate::universe::atomic_write(path, &json)?;
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

/// Parse a single override file off disk into the HashMap shape used
/// per layer. Shared between `load_from_path` (single-layer) and
/// `from_layered_paths` (multi-layer) so the on-disk schema and the
/// normalization of keys stay identical regardless of entry point.
fn read_layer(path: &Path) -> std::io::Result<HashMap<String, UserOverride>> {
    let bytes = std::fs::read(path)?;
    let file: OverrideFile = serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let mut map = HashMap::new();
    for o in file.overrides {
        let key = OverrideStore::normalize_key(&o.surface);
        map.insert(key, o);
    }
    Ok(map)
}

// ── Process-wide active store (M8b, layered in M8b-v2) ───────────────
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
//
// M8b-v2 additions:
// - `activate_layered_for_universe` installs a multi-layer store,
//   one layer per (parent + cUniverse child) combination.
// - `set_sovereign_layer` replaces only `layers[0]` of the current
//   active store, preserving any child layers — used by the CRUD
//   commands so editing a parent override doesn't clobber the child
//   contributions the Universe switcher installed.

static ACTIVE_STORE: OnceLock<RwLock<Arc<OverrideStore>>> = OnceLock::new();

/// Fast-path snapshot of "is the active store empty?". Maintained by
/// every mutator of `ACTIVE_STORE` (`set_active`, `set_sovereign_layer`,
/// `clear_active`). Readers on the FTS5 hot path check this first via
/// `active_if_non_empty` — when `true`, we skip the `RwLock::read` +
/// `Arc::clone` and return `None` directly, saving ~25 ns per token on
/// the overwhelmingly common case (fresh install, no authored overrides).
///
/// Starts `true` because the default `ACTIVE_STORE` (before any activate
/// call) is an empty `OverrideStore::new()` — so the fast-path kicks in
/// immediately on cold boot.
///
/// # Ordering discipline
///
/// Writers:
/// - **Transitioning to non-empty** (new store has entries): update the
///   atomic to `false` **before** the RwLock swap. Readers who see
///   `false` take the slow path and will observe either the old or new
///   store via the RwLock — both at least non-empty (or if we're going
///   empty→non-empty, reading the old empty store one more time is
///   harmless: the caller does an extra HashMap probe that misses and
///   falls through, semantically identical to the fast-path None).
/// - **Transitioning to empty** (new store has no entries): update the
///   atomic to `true` **after** the RwLock swap. Readers who see `true`
///   will skip the lock — and the store they'd have read is now empty,
///   so skipping is correct. If a reader sees `false` (stale) during
///   the transition, they take the slow path and read the post-swap
///   empty store, which just routes through L1–L5 via `None` inside
///   `analyze_with_overrides`. Never incorrect.
///
/// This gives us the invariant: **if `ACTIVE_STORE_EMPTY == true`, the
/// active store is guaranteed empty.** The reverse is allowed to be
/// stale (over-report non-empty is safe — just a one-off extra clone).
///
/// Readers use `Ordering::Acquire` on the atomic + `Ordering::Relaxed`
/// on the miss path — cheap enough that even the Acquire is below the
/// HashMap probe that would follow.
static ACTIVE_STORE_EMPTY: AtomicBool = AtomicBool::new(true);

fn store_lock() -> &'static RwLock<Arc<OverrideStore>> {
    ACTIVE_STORE.get_or_init(|| RwLock::new(Arc::new(OverrideStore::new())))
}

/// Cheap clone of the currently-active override store. Called per FTS5
/// tokenizer invocation. `Arc::clone` is a refcount bump — the store's
/// internal layer Vec and its HashMaps are NOT duplicated.
///
/// On first call before any `set_active` / `activate_for_universe`, this
/// returns an empty store, so Layer 0 never fires and every analysis
/// falls through to Layers 1–5 as in pre-M8b behaviour.
///
/// # When to use which
///
/// - FTS5 hot path → [`active_if_non_empty`]. Returns `None` on the
///   common empty case without touching the RwLock.
/// - Diagnostic / admin code (tests, Settings UI) → `active()`. Always
///   returns a concrete Arc even for an empty store, so callers can
///   `.iter()` / `.len()` / `.layer_count()` without branching.
pub fn active() -> Arc<OverrideStore> {
    store_lock().read().expect("arabic override lock poisoned").clone()
}

/// Fast-path variant of [`active`] for the FTS5 tokenizer hot path.
///
/// Returns `None` when the active store is empty (no authored overrides
/// in the sovereign layer and no child layers with content) — the common
/// case for fresh installs and the overwhelming majority of Universes
/// that never author an override. Callers pass the resulting
/// `Option<&OverrideStore>` directly to
/// [`analyze_with_overrides`] / [`analyze_with_overrides_best`].
///
/// **Performance**: the empty path is a single `AtomicBool::load`
/// (~2 ns) versus `active()`'s RwLock-read + Arc::clone (~25 ns). On
/// production Arabic-heavy notes this is called once per token, so the
/// savings scale linearly with document size. At 100K tokens per note
/// that's ~2.3 ms trimmed off the indexer per note.
///
/// **Correctness**: follows the ordering discipline on
/// [`ACTIVE_STORE_EMPTY`]. A stale read showing "empty" is impossible
/// (the bit is only set `true` after a swap to an empty store has
/// completed). A stale read showing "non-empty" is possible during a
/// transitioning-to-empty swap — the caller gets an `Some` Arc to an
/// empty store, does one extra HashMap-miss probe, then falls through to
/// L1–L5 just as if we'd returned `None`. Not a correctness issue.
pub fn active_if_non_empty() -> Option<Arc<OverrideStore>> {
    if ACTIVE_STORE_EMPTY.load(Ordering::Acquire) {
        return None;
    }
    // Slow path — RwLock read + Arc::clone. Still cheap (~25 ns) but
    // paid only when there's likely real work to do.
    Some(store_lock().read().expect("arabic override lock poisoned").clone())
}

/// Install a new active store, replacing all layers. Called by:
/// - `activate_for_universe` / `activate_layered_for_universe` when the
///   user opens / switches Universes.
/// - `clear_active` on Universe close.
/// - Tests that need deterministic baseline state.
///
/// CRUD commands (`add_arabic_override` / `remove_arabic_override`)
/// should use `set_sovereign_layer` instead, so they don't wipe out
/// child-Universe contributions when the user edits a parent override.
///
/// Thread-safe. The write guard is held only for the Arc pointer swap,
/// not for any HashMap-level mutation — the store itself is immutable
/// once installed. Maintains [`ACTIVE_STORE_EMPTY`] in the orderings
/// documented on that static — empty→non-empty transitions flip the
/// bit **before** the swap; non-empty→empty transitions flip the bit
/// **after** the swap.
pub fn set_active(store: OverrideStore) {
    // PJ-307 — the bit and the swap are maintained under ONE held write guard, exactly as
    // `set_sovereign_layer` already does.
    //
    // This function used to take the guard inside the swap statement (so it was released at
    // the end of that statement) and store the bit OUTSIDE it, while its sibling writer held
    // its guard across both. Two writers maintaining one invariant under two different
    // disciplines do not serialise against each other, and the interleaving is reachable:
    // `set_active_universe` is `#[tauri::command(async)]` and reaches here on a runtime worker,
    // while `add_arabic_override` / `remove_arabic_override` are sync commands reaching
    // `set_sovereign_layer` on the main thread. `switch_lock` serialises switch-vs-switch only.
    //
    // Either order leaves `ACTIVE_STORE_EMPTY == true` over a NON-EMPTY store, which is
    // precisely the invariant documented above ("if true, the active store is guaranteed
    // empty"). `active_if_non_empty` then returns None on the atomic alone, so the FTS5
    // tokenizer path stems every Arabic token as though the user had authored no overrides —
    // silently, with `active()` still reporting the correct store, so every len()/layer_count()
    // diagnostic and the Settings panel look healthy while the index diverges.
    let mut guard = store_lock().write().expect("arabic override lock poisoned");
    publish_under(&mut guard, store);
}

/// **PJ-307 — the ONE place `ACTIVE_STORE_EMPTY` and the store are published together.**
///
/// Both writers (`set_active` and `set_sovereign_layer`) route through here, so the ordering
/// discipline documented on [`ACTIVE_STORE_EMPTY`] exists in exactly one place and the two
/// cannot drift apart. They had drifted: `set_active` took the write guard inside its swap
/// statement — releasing it at the end of that statement — and stored the bit OUTSIDE it, while
/// `set_sovereign_layer` held its guard across both. Two writers maintaining one invariant under
/// two disciplines do not serialise, and the interleaving is reachable: `set_active_universe` is
/// `#[tauri::command(async)]` and lands here on a runtime worker, while `add_arabic_override` /
/// `remove_arabic_override` are sync commands on the main thread, and `switch_lock` serialises
/// switch-vs-switch only.
///
/// Either order can leave the bit `true` over a NON-EMPTY store — the exact negation of the
/// documented invariant. `active_if_non_empty` then returns `None` from the atomic alone, so the
/// FTS5 tokenizer stems every Arabic token as though no override existed: silently, with
/// `active()` still returning the correct store, so `len()`/`layer_count()` diagnostics and the
/// Settings panel all look healthy while the index diverges.
///
/// Extracting this is the fix rather than repeating the ordering in both writers, because "both
/// callers remember the discipline" is the promise that was already broken once.
fn publish_under(
    guard: &mut std::sync::RwLockWriteGuard<'_, Arc<OverrideStore>>,
    store: OverrideStore,
) {
    let is_empty = store.is_empty();
    // Empty -> non-empty: bit BEFORE the swap. A reader who sees `false` falls through to the
    // RwLock and reads either the old or new store - both safe to route through the slow path.
    if !is_empty {
        ACTIVE_STORE_EMPTY.store(false, Ordering::Release);
    }
    **guard = Arc::new(store);
    // Non-empty -> empty: bit AFTER the swap, so a reader seeing `true` is guaranteed the store
    // it would have read is empty - safe to skip the lock.
    if is_empty {
        ACTIVE_STORE_EMPTY.store(true, Ordering::Release);
    }
}

/// Replace only the sovereign (parent) layer of the currently-active
/// store, preserving any cUniverse child layers underneath.
///
/// Called by `add_arabic_override` / `remove_arabic_override` after the
/// CRUD command has persisted the change to disk. The command loads a
/// single-layer view from the parent's `arabic-overrides.json`, mutates
/// it, saves, and then calls this function to slot that layer back into
/// position 0 of the active store — without re-reading child override
/// files (no disk I/O) and without losing the child contributions the
/// Universe switcher originally installed.
///
/// Takes a single-layer `OverrideStore` (what `load_from_path` returns).
/// If the passed store has more than one layer, only `layers[0]` is
/// adopted as the new sovereign; additional layers are ignored.
pub fn set_sovereign_layer(sovereign: OverrideStore) {
    let new_layer_0 = sovereign.layers.into_iter().next().unwrap_or_default();
    let mut guard = store_lock().write().expect("arabic override lock poisoned");
    // Build the replacement by cloning out the prior child layers, then
    // slotting the new sovereign in front. Cheap: HashMaps are Arc-aware
    // through the wrapping Arc<OverrideStore>, but the layers Vec itself
    // isn't Arc'd, so we do one shallow clone per child — typically 0.
    let prior = guard.clone();
    let mut new_layers: Vec<HashMap<String, UserOverride>> = Vec::with_capacity(prior.layers.len().max(1));
    new_layers.push(new_layer_0);
    for child in prior.layers.iter().skip(1) {
        new_layers.push(child.clone());
    }
    let new_store = OverrideStore { layers: new_layers };
    // PJ-307b — publish UNDER the guard this function already holds.
    //
    // The first version of this fix did `drop(guard); publish(new_store);` and justified it in a
    // comment: "the gap is safe because a concurrent writer landing in it publishes a fully
    // coherent state of its own." **That comment was false**, and it shipped in a673a548. Their
    // state is coherent; this function then OVERWRITES it with `new_layers`, which were built
    // from a `prior` read before the gap. A universe switch landing in that window has its whole
    // layered store — including every child-universe layer it just loaded — clobbered by a store
    // derived from the previous universe's.
    //
    // Stated directionally rather than as a severity ranking, because "strictly worse" would not
    // survive a challenge — both defects need the same microsecond-wide collision. The honest
    // statement: **the fix took a function that was already safe and made it unsafe**, in order to
    // correct a flag-ordering problem in its neighbour.
    //
    // The read-modify-write (`prior` -> `new_layers` -> swap) must be ATOMIC, which it was before
    // this fix touched it. Passing the held guard keeps that atomicity AND keeps the bit/swap
    // ordering in exactly one place, which was the whole point of extracting it.
    publish_under(&mut guard, new_store);
}

/// Load a single override file from a Universe and install it as the
/// active store (single-layer). Kept for non-federated callers and
/// backwards-compatible tests. A Universe with cUniverse children
/// should use `activate_layered_for_universe` so the children's
/// overrides are also consulted.
///
/// Missing parent file = empty store (the common case for a fresh
/// Universe). Malformed JSON = error.
///
/// Returns the count of installed overrides on success.
pub fn activate_for_universe(universe_root: &Path) -> Result<usize, String> {
    activate_layered_for_universe(universe_root, &[])
}

/// Layered activation — install the parent's overrides as the sovereign
/// layer and each cUniverse child's overrides as additional layers
/// probed in declaration order on parent miss.
///
/// Called by `universe::set_active_universe` after reading
/// `UniverseMeta::children` and resolving each entry to a Universe root
/// path. The order of `child_universes` is preserved in the lookup
/// order — the caller decides priority among children (today: the order
/// the user added them in the federation settings).
///
/// Missing child files become empty layers (a fresh child Universe
/// contributes nothing but does not abort the load). Parent-file errors
/// propagate, because the active Universe's overrides failing to load
/// is a user-visible problem worth surfacing; child errors propagate
/// too, since a malformed child file is a bug the Settings UI should
/// show rather than silently swallow.
///
/// Returns the total override count across all layers, for boot
/// diagnostics ("loaded 42 overrides across parent + 3 child Universes").
pub fn activate_layered_for_universe(
    universe_root: &Path,
    child_universes: &[PathBuf],
) -> Result<usize, String> {
    let mut paths: Vec<PathBuf> = Vec::with_capacity(1 + child_universes.len());
    paths.push(OverrideStore::path_in_universe(universe_root));
    for cu in child_universes {
        paths.push(OverrideStore::path_in_universe(cu));
    }
    let store = OverrideStore::from_layered_paths(&paths)
        .map_err(|e| format!("Failed to load layered overrides: {}", e))?;
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

/// Process-wide test mutex for every cargo-test path that mutates
/// `ACTIVE_STORE`.
///
/// Declared at module (not test-module) scope so integration tests in
/// other crate modules — notably `search::tests::m8c_*` — can lock the
/// same mutex and serialize against the tests in this file. Without
/// this, the default `--test-threads` concurrency would race
/// `set_active` / `active` calls across suites, producing flaky
/// failures.
///
/// `#[cfg(test)]`-gated so it contributes zero bytes to release builds.
#[cfg(test)]
pub(crate) static TEST_OVERRIDE_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
/// then reinstalls the active store's sovereign layer so subsequent
/// FTS5 tokens see the change without waiting for the next Universe
/// switch.
///
/// Parameter name `entry` rather than Rust keyword-adjacent `override`
/// so it serializes cleanly through Tauri's IPC layer.
///
/// Uses `set_sovereign_layer` (not `set_active`) so any cUniverse
/// children installed at Universe-switch time remain probed on parent
/// miss — the user editing a parent override doesn't silently drop
/// child-Universe contributions.
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
    set_sovereign_layer(store);
    Ok(())
}

/// Remove the override for a surface. Returns `true` if an override was
/// removed, `false` if no override existed for this surface (not an
/// error — idempotent from the UI's perspective).
///
/// Only touches the sovereign (active Universe's) layer — a surface
/// that exists only in a cUniverse child's overrides file cannot be
/// removed from the parent Universe. To remove a child's override, the
/// user must switch to that child Universe first.
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
        set_sovereign_layer(store);
    }
    Ok(removed)
}

/// Targeted FTS5 re-tokenization for every note whose body or name
/// mentions `surface`.
///
/// Fired by the Settings UI after `add_arabic_override` or
/// `remove_arabic_override` so the on-disk FTS index reflects the new
/// Layer 0 verdict without waiting for a full Universe rebuild. Returns
/// the number of rows re-tokenized — zero is a valid outcome (no indexed
/// note contains the surface) and not an error.
///
/// The heavy lifting lives in `search::reindex_notes_matching_text`;
/// this wrapper just exposes it through the Tauri IPC surface.
// Note-open-freeze Batch-2 §B2-2 (2026-07-03): `(async)` — off the IPC dispatch thread.
// Discovery-verified async-only-safe: DB-only / mutex-covered body, no note-file writes,
// all callers await. See SESSION-LOG-2026-07-03 (Architect findings).
#[tauri::command(async)]
pub fn reindex_arabic_overrides(
    app: tauri::AppHandle,
    surface: String,
) -> Result<u32, String> {
    use tauri::Manager;
    // PJ-308 — this was the ONE DB-touching command here without `ensure_search_db_ready`,
    // and its absence made a skipped reindex indistinguishable from a completed one.
    //
    // `reindex_notes_matching_text` returns `Ok(0)` when `state.db` is None (search.rs:13777),
    // byte-identical to the legitimate "no indexed note contains this surface" result — and the
    // Settings panel renders any `Ok` as a green success. `invalidate_search_state` NULLs
    // `state.db` on every universe switch and it stays None until the next
    // `ensure_search_db_ready`, while the shell paints before the boot fan-out installs the
    // connection. So: add an override, be told the reindex completed, and every already-indexed
    // Arabic note keeps its pre-override stems forever, because nothing re-runs this.
    //
    // Every sibling DB-touching command already carries this line (libraries.rs:1712, :1753,
    // :2634, :2709), each added by an earlier inspection against this exact hazard.
    crate::search::ensure_search_db_ready(&app)?;
    let state = app.state::<crate::search::SearchState>();
    crate::search::reindex_notes_matching_text(&state, &surface)
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

    // **PJ-307 — why there is no test here, stated rather than left as an absence.**
    //
    // A concurrency test WAS written for the fast-path-bit invariant and has been deleted,
    // for two independently sufficient reasons:
    //
    //   1. **It did not reproduce the defect.** Run against the pre-fix code — `set_active`
    //      restored to storing the bit outside the write guard — it PASSED. The interleaving
    //      needs a preemption inside a few-instruction window, and 40 rounds x 120 writes
    //      never hit it. It was a settled-state guard, not a red->green.
    //   2. **It broke its neighbours.** Hammering the process-global store from two threads
    //      for the duration of the test is exactly the hazard LL-047/LL-049 describe, and it
    //      failed `set_active_replaces_prior_store_entirely`,
    //      `set_active_then_active_roundtrips`, `set_sovereign_layer_on_empty_active_creates_
    //      single_layer` and `set_sovereign_layer_preserves_child_layers` in 6 of 8 suite runs.
    //      A test that mutates shared state for a duration is the very thing this file's fix
    //      is about; writing one to guard that fix was the same mistake one layer out.
    //
    // What justifies the fix instead is structural and checkable by reading: the bit-and-swap
    // discipline lives in exactly ONE function (`publish`) that both writers call, so "two
    // writers, two disciplines" is no longer expressible. If someone later wants a genuine
    // red->green here, it needs a test-only hook that widens the window inside `publish` —
    // a deliberate change to production code, and a decision to be taken openly rather than
    // smuggled in behind a green test that proves nothing.

    // ── persistence ───────────────────────────────────────────────────

    /// PJ-305 — a temp path unique to this process AND this call.
    ///
    /// The persistence tests below used FIXED names (`constellation-overrides-atomic`,
    /// `-roundtrip`, `-sorted`, …) and each opens with `remove_dir_all` on it, so two
    /// concurrent `cargo test` processes on one machine delete each other's fixture
    /// mid-test. Observed directly on 2026-08-17: running two suites at once produced
    /// `save_then_load_roundtrip`, `save_is_atomic_no_leftover_tmp_on_success`,
    /// `save_sorts_entries_alphabetically_for_git_friendly_diffs` and
    /// `load_rejects_malformed_json` failing in shifting combinations.
    ///
    /// The unique-path idiom already existed further down this same file (the
    /// `constellation_overrides_test_activate_{nanos}` sites); these tests simply were
    /// not using it. Same class as LL-049: a test sharing a mutable resource.
    fn unique_tmp(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "constellation-overrides-{}-{}-{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn load_missing_file_yields_empty_store() {
        let tmp = unique_tmp("missing").with_extension("json");
        let _ = std::fs::remove_file(&tmp); // ensure missing
        let store = OverrideStore::load_from_path(&tmp).expect("load must not error on missing");
        assert!(store.is_empty());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let tmp_dir = unique_tmp("roundtrip");
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
        let tmp = unique_tmp("malformed").with_extension("json");
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
        let tmp = unique_tmp("unknown-fields").with_extension("json");
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
        let tmp_dir = unique_tmp("atomic");
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
        let tmp_dir = unique_tmp("sorted");
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

    /// RAII guard that snapshots the active store on construction and
    /// restores it on drop, so each test runs against a predictable
    /// (empty) baseline and doesn't leak state into the next one.
    ///
    /// Locks `crate::arabic::overrides::TEST_OVERRIDE_MUTEX` — the
    /// crate-wide test mutex — so any concurrent `cargo test` thread
    /// that touches `ACTIVE_STORE` (including `search::tests::m8c_*`)
    /// serializes against us.
    struct RegistryGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        prior: Arc<OverrideStore>,
    }

    impl RegistryGuard {
        fn new() -> Self {
            let lock = super::TEST_OVERRIDE_MUTEX
                .lock()
                .unwrap_or_else(|e| e.into_inner()); // ignore poisoning
            let prior = active();
            clear_active();
            Self { _lock: lock, prior }
        }
    }

    impl Drop for RegistryGuard {
        fn drop(&mut self) {
            // Restore the prior Arc byte-for-byte, and keep
            // `ACTIVE_STORE_EMPTY` in sync with the restored state.
            // Without this sync the next test could observe a stale
            // fast-path bit — e.g. see `empty=true` while the RwLock
            // actually holds a non-empty store, causing
            // `active_if_non_empty` to return `None` and bypass real
            // overrides. Mirrors the ordering discipline of
            // `set_active` for consistency, though under the mutex the
            // transitional window is not externally observable.
            let is_empty = self.prior.is_empty();
            if !is_empty {
                ACTIVE_STORE_EMPTY.store(false, Ordering::Release);
            }
            *store_lock().write().unwrap() = self.prior.clone();
            if is_empty {
                ACTIVE_STORE_EMPTY.store(true, Ordering::Release);
            }
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

    // ── M8b-v2: layered federation ────────────────────────────────────
    //
    // Helper that authors a minimal `arabic-overrides.json` at the
    // canonical path under `universe_dir` and returns the path, so
    // end-to-end activation tests can exercise the on-disk → layered-
    // store pipeline without re-implementing the schema per test.

    fn seed_universe_with_override(universe_dir: &Path, o: UserOverride) -> PathBuf {
        let path = OverrideStore::path_in_universe(universe_dir);
        let mut single = OverrideStore::new();
        single.insert(o);
        single.save_to_path(&path).expect("seed universe overrides");
        path
    }

    /// Build a layered store in-memory (no disk I/O) for unit-level
    /// tests of lookup / iter / len semantics. The first layer is the
    /// sovereign; subsequent layers are children in declaration order.
    fn layered_store(layers: Vec<Vec<UserOverride>>) -> OverrideStore {
        let built_layers: Vec<HashMap<String, UserOverride>> = layers
            .into_iter()
            .map(|v| {
                let mut m = HashMap::new();
                for o in v {
                    let key = OverrideStore::normalize_key(&o.surface);
                    m.insert(key, o);
                }
                m
            })
            .collect();
        OverrideStore { layers: built_layers }
    }

    #[test]
    fn layered_lookup_returns_parent_on_conflict() {
        // Both parent and child have an override for the same surface —
        // the parent's (layer 0) must win. This is the core sovereignty
        // guarantee of M8b-v2.
        let store = layered_store(vec![
            vec![mk_override("خليفة", "from_parent")],
            vec![mk_override("خليفة", "from_child")],
        ]);
        let norm = super::super::normalizer::normalize("خليفة").stripped;
        let hit = store.lookup(&norm).expect("must hit");
        assert_eq!(hit.lemma, "from_parent",
            "parent's override must win on conflict, not child's");
    }

    #[test]
    fn layered_lookup_falls_through_to_child_on_parent_miss() {
        // Parent has nothing; child contributes a surface. Lookup must
        // find the child entry rather than returning None, otherwise
        // federated overrides are invisible to the tokenizer.
        let store = layered_store(vec![
            vec![], // empty parent
            vec![mk_override("خليفة", "from_child")],
        ]);
        let norm = super::super::normalizer::normalize("خليفة").stripped;
        let hit = store.lookup(&norm).expect("must hit via child");
        assert_eq!(hit.lemma, "from_child");
    }

    #[test]
    fn layered_lookup_walks_children_in_declaration_order() {
        // Parent misses; two children each have a distinct surface.
        // Both surfaces must be reachable, and a surface present in
        // an earlier child must not be shadowed by a later child.
        let store = layered_store(vec![
            vec![], // empty parent
            vec![mk_override("ألف", "from_child_1")],
            vec![mk_override("باء", "from_child_2"), mk_override("ألف", "from_child_2_shadow")],
        ]);
        let alif = super::super::normalizer::normalize("ألف").stripped;
        let ba = super::super::normalizer::normalize("باء").stripped;
        assert_eq!(store.lookup(&alif).unwrap().lemma, "from_child_1",
            "earlier child's entry must win over a later child's duplicate");
        assert_eq!(store.lookup(&ba).unwrap().lemma, "from_child_2");
    }

    #[test]
    fn layered_len_sums_across_layers() {
        let store = layered_store(vec![
            vec![mk_override("ألف", "alif"), mk_override("باء", "ba")],
            vec![mk_override("جيم", "jim")],
            vec![mk_override("دال", "dal"), mk_override("هاء", "ha"), mk_override("واو", "waw")],
        ]);
        assert_eq!(store.len(), 2 + 1 + 3);
        assert_eq!(store.layer_count(), 3);
        assert!(!store.is_empty());
    }

    #[test]
    fn layered_is_empty_when_all_layers_empty() {
        // A stack with layers that all happen to be empty is still
        // "empty" for the short-circuit path. The tokenizer's
        // `if store.is_empty()` guard should skip the lookup entirely.
        let store = layered_store(vec![vec![], vec![], vec![]]);
        assert!(store.is_empty(), "stack of empty layers must report empty");
        assert_eq!(store.layer_count(), 3,
            "but the layers themselves are preserved");
    }

    #[test]
    fn layered_iter_yields_entries_from_every_layer() {
        let store = layered_store(vec![
            vec![mk_override("ألف", "alif")],
            vec![mk_override("باء", "ba"), mk_override("جيم", "jim")],
        ]);
        let surfaces: Vec<String> = store.iter().map(|o| o.surface.clone()).collect();
        assert_eq!(surfaces.len(), 3);
        assert!(surfaces.contains(&"ألف".to_string()));
        assert!(surfaces.contains(&"باء".to_string()));
        assert!(surfaces.contains(&"جيم".to_string()));
    }

    #[test]
    fn layered_sovereign_iter_yields_only_parent_layer() {
        // Settings UI contract: the editable overrides list is the
        // sovereign layer only — child-Universe entries must not appear
        // in the parent's editing UI (the user can't modify them from
        // this Universe).
        let store = layered_store(vec![
            vec![mk_override("ألف", "alif"), mk_override("باء", "ba")],
            vec![mk_override("جيم", "jim_from_child")],
        ]);
        let sovereign: Vec<String> = store.sovereign_iter().map(|o| o.lemma.clone()).collect();
        assert_eq!(sovereign.len(), 2);
        assert!(sovereign.contains(&"alif".to_string()));
        assert!(sovereign.contains(&"ba".to_string()));
        assert!(!sovereign.contains(&"jim_from_child".to_string()),
            "sovereign_iter must exclude child-Universe entries");
    }

    #[test]
    fn insert_into_layered_only_touches_sovereign_layer() {
        // Editing a parent override must not rewrite any child layer's
        // entries — child files are owned by child Universes and must
        // stay byte-for-byte untouched.
        let mut store = layered_store(vec![
            vec![mk_override("ألف", "alif_parent")],
            vec![mk_override("باء", "ba_child")],
        ]);
        store.insert(mk_override("جيم", "new_parent_entry"));

        // Parent now has 2 entries; child still has exactly 1.
        assert_eq!(store.layers[0].len(), 2);
        assert_eq!(store.layers[1].len(), 1);

        // And the new entry must resolve to the parent, never shadowing
        // into the child.
        let jim = super::super::normalizer::normalize("جيم").stripped;
        assert_eq!(store.lookup(&jim).unwrap().lemma, "new_parent_entry");
    }

    #[test]
    fn remove_from_layered_cannot_touch_child_entries() {
        // A surface that only exists in a child layer is NOT removable
        // via remove() — CRUD only touches the sovereign layer. The
        // Settings UI surfaces this by showing child entries as read-only.
        let mut store = layered_store(vec![
            vec![mk_override("ألف", "alif_parent")],
            vec![mk_override("باء", "ba_child")],
        ]);

        // Remove something that only exists in child → returns None,
        // child layer untouched.
        let removed = store.remove("باء");
        assert!(removed.is_none(),
            "surface only in child layer must not be removable from parent");
        assert_eq!(store.layers[1].len(), 1, "child layer unchanged");

        // Remove something that exists in parent → works, child still
        // untouched.
        let removed = store.remove("ألف");
        assert!(removed.is_some());
        assert_eq!(store.layers[0].len(), 0);
        assert_eq!(store.layers[1].len(), 1, "child layer still untouched");
    }

    #[test]
    fn save_to_path_writes_only_sovereign_not_children() {
        // When the parent's overrides file is saved, the on-disk JSON
        // must contain only sovereign entries — child contributions
        // must not leak into the parent's file.
        let tmp_dir = std::env::temp_dir().join(format!(
            "constellation_overrides_save_layered_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = OverrideStore::path_in_universe(&tmp_dir);

        let store = layered_store(vec![
            vec![mk_override("ألف", "alif_parent")],
            vec![mk_override("باء", "ba_child"), mk_override("جيم", "jim_child")],
        ]);
        store.save_to_path(&path).expect("save");

        let contents = std::fs::read_to_string(&path).expect("read back");
        assert!(contents.contains("ألف"), "parent entry must be saved");
        assert!(!contents.contains("باء"), "child entry must NOT leak into parent file");
        assert!(!contents.contains("جيم"), "child entry must NOT leak into parent file");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn from_layered_paths_builds_correct_stack() {
        let tmp_root = std::env::temp_dir().join(format!(
            "constellation_overrides_from_layered_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let parent_dir = tmp_root.join("parent");
        let child_a_dir = tmp_root.join("child_a");
        let child_b_dir = tmp_root.join("child_b");

        let parent_path = seed_universe_with_override(
            &parent_dir,
            UserOverride {
                surface: "ألف".into(), lemma: "alif_parent".into(),
                root: String::new(), pattern_label: "user:override".into(),
                pos: PartOfSpeech::ProperNoun, note: String::new(),
                created_at: String::new(),
            },
        );
        let child_a_path = seed_universe_with_override(
            &child_a_dir,
            UserOverride {
                surface: "باء".into(), lemma: "ba_child_a".into(),
                root: String::new(), pattern_label: "user:override".into(),
                pos: PartOfSpeech::ProperNoun, note: String::new(),
                created_at: String::new(),
            },
        );
        let child_b_path = seed_universe_with_override(
            &child_b_dir,
            UserOverride {
                surface: "جيم".into(), lemma: "jim_child_b".into(),
                root: String::new(), pattern_label: "user:override".into(),
                pos: PartOfSpeech::ProperNoun, note: String::new(),
                created_at: String::new(),
            },
        );

        let store = OverrideStore::from_layered_paths(&[
            parent_path, child_a_path, child_b_path,
        ]).expect("layered load");

        assert_eq!(store.layer_count(), 3);
        assert_eq!(store.len(), 3);

        let alif = super::super::normalizer::normalize("ألف").stripped;
        let ba = super::super::normalizer::normalize("باء").stripped;
        let jim = super::super::normalizer::normalize("جيم").stripped;

        assert_eq!(store.lookup(&alif).unwrap().lemma, "alif_parent");
        assert_eq!(store.lookup(&ba).unwrap().lemma, "ba_child_a");
        assert_eq!(store.lookup(&jim).unwrap().lemma, "jim_child_b");

        let _ = std::fs::remove_dir_all(&tmp_root);
    }

    #[test]
    fn from_layered_paths_tolerates_missing_child_files() {
        // A freshly-created cUniverse child that hasn't authored any
        // overrides yet must not abort the layered load — its layer
        // just becomes empty and the active Universe continues working.
        let tmp_root = std::env::temp_dir().join(format!(
            "constellation_overrides_missing_child_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let parent_dir = tmp_root.join("parent");
        let parent_path = seed_universe_with_override(
            &parent_dir,
            UserOverride {
                surface: "ألف".into(), lemma: "alif_parent".into(),
                root: String::new(), pattern_label: "user:override".into(),
                pos: PartOfSpeech::ProperNoun, note: String::new(),
                created_at: String::new(),
            },
        );
        // Point at a non-existent child path — simulates a child
        // Universe that has no `arabic-overrides.json` authored yet.
        let missing_child_path = tmp_root.join("ghost_child")
            .join(".constellation").join("arabic-overrides.json");

        let store = OverrideStore::from_layered_paths(&[
            parent_path, missing_child_path,
        ]).expect("missing child must not abort");

        assert_eq!(store.layer_count(), 2);
        assert_eq!(store.layers[1].len(), 0, "ghost child layer is empty");
        assert_eq!(store.len(), 1);

        let _ = std::fs::remove_dir_all(&tmp_root);
    }

    #[test]
    fn from_layered_paths_propagates_malformed_child_error() {
        // An existing-but-malformed child file is a bug the Settings
        // UI should surface, not a silent degradation. The error must
        // propagate so the caller can log/display it.
        let tmp_root = std::env::temp_dir().join(format!(
            "constellation_overrides_malformed_child_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let parent_dir = tmp_root.join("parent");
        let parent_path = seed_universe_with_override(
            &parent_dir,
            UserOverride {
                surface: "ألف".into(), lemma: "alif_parent".into(),
                root: String::new(), pattern_label: "user:override".into(),
                pos: PartOfSpeech::ProperNoun, note: String::new(),
                created_at: String::new(),
            },
        );
        let child_dir = tmp_root.join("broken_child");
        let child_path = OverrideStore::path_in_universe(&child_dir);
        std::fs::create_dir_all(child_path.parent().unwrap()).expect("mkdir child");
        std::fs::write(&child_path, b"{ corrupted json").expect("write garbage");

        let result = OverrideStore::from_layered_paths(&[parent_path, child_path]);
        assert!(result.is_err(),
            "malformed child JSON must surface as error, not silent drop");

        let _ = std::fs::remove_dir_all(&tmp_root);
    }

    #[test]
    fn activate_layered_for_universe_end_to_end() {
        let _g = RegistryGuard::new();

        let tmp_root = std::env::temp_dir().join(format!(
            "constellation_overrides_activate_layered_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let parent_dir = tmp_root.join("parent_universe");
        let child_dir = tmp_root.join("child_universe");

        seed_universe_with_override(&parent_dir, UserOverride {
            surface: "خليفة".into(), lemma: "from_parent".into(),
            root: String::new(), pattern_label: "user:override".into(),
            pos: PartOfSpeech::ProperNoun, note: String::new(),
            created_at: String::new(),
        });
        seed_universe_with_override(&child_dir, UserOverride {
            surface: "إمام".into(), lemma: "from_child".into(),
            root: String::new(), pattern_label: "user:override".into(),
            pos: PartOfSpeech::ProperNoun, note: String::new(),
            created_at: String::new(),
        });

        let count = activate_layered_for_universe(&parent_dir, &[child_dir.clone()])
            .expect("layered activation");
        assert_eq!(count, 2, "total count includes both parent and child entries");

        let store = active();
        assert_eq!(store.layer_count(), 2);

        let khalifa = super::super::normalizer::normalize("خليفة").stripped;
        let imam = super::super::normalizer::normalize("إمام").stripped;
        assert_eq!(store.lookup(&khalifa).unwrap().lemma, "from_parent",
            "parent surface resolves to parent layer");
        assert_eq!(store.lookup(&imam).unwrap().lemma, "from_child",
            "child surface resolves via fall-through to child layer");

        let _ = std::fs::remove_dir_all(&tmp_root);
    }

    #[test]
    fn activate_for_universe_is_equivalent_to_zero_children() {
        // Backward-compatibility contract: the pre-M8b-v2 API
        // `activate_for_universe(p)` must now be observationally
        // identical to `activate_layered_for_universe(p, &[])`. A
        // non-federated Universe with no children gets exactly one
        // layer, same as before the refactor.
        let _g = RegistryGuard::new();

        let tmp_dir = std::env::temp_dir().join(format!(
            "constellation_overrides_backcompat_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        seed_universe_with_override(&tmp_dir, UserOverride {
            surface: "وائل".into(), lemma: "وائل".into(),
            root: String::new(), pattern_label: "user:ProperNoun".into(),
            pos: PartOfSpeech::ProperNoun, note: String::new(),
            created_at: String::new(),
        });

        let count = activate_for_universe(&tmp_dir).expect("activate");
        assert_eq!(count, 1);

        let store = active();
        assert_eq!(store.layer_count(), 1,
            "single-arg activate_for_universe must produce exactly one layer");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn set_sovereign_layer_preserves_child_layers() {
        // The critical CRUD invariant: editing a parent override must
        // not drop any cUniverse child contributions. This is the whole
        // reason `set_sovereign_layer` exists alongside `set_active`.
        let _g = RegistryGuard::new();

        // Install a 2-layer store: parent + one child.
        let initial = layered_store(vec![
            vec![mk_override("ألف", "alif_v1")],
            vec![mk_override("باء", "ba_from_child")],
        ]);
        set_active(initial);
        assert_eq!(active().layer_count(), 2);

        // Simulate a CRUD operation: load a single-layer view from disk,
        // mutate, call set_sovereign_layer.
        let mut new_sovereign = OverrideStore::new();
        new_sovereign.insert(mk_override("ألف", "alif_v2_edited"));
        new_sovereign.insert(mk_override("جيم", "jim_new_entry"));
        set_sovereign_layer(new_sovereign);

        // After: parent layer has the edited content; child layer
        // survives untouched.
        let after = active();
        assert_eq!(after.layer_count(), 2,
            "child layer must survive sovereign replacement");

        let alif = super::super::normalizer::normalize("ألف").stripped;
        let ba = super::super::normalizer::normalize("باء").stripped;
        let jim = super::super::normalizer::normalize("جيم").stripped;

        assert_eq!(after.lookup(&alif).unwrap().lemma, "alif_v2_edited",
            "parent's edited override must be visible");
        assert_eq!(after.lookup(&jim).unwrap().lemma, "jim_new_entry",
            "parent's new override must be visible");
        assert_eq!(after.lookup(&ba).unwrap().lemma, "ba_from_child",
            "child override must still be reachable after CRUD");
    }

    #[test]
    fn set_sovereign_layer_on_empty_active_creates_single_layer() {
        // Edge case: no Universe activated yet (ACTIVE_STORE is empty
        // by default). set_sovereign_layer on an empty store must
        // produce a 1-layer store with just the sovereign content,
        // mirroring what set_active would do.
        let _g = RegistryGuard::new();
        clear_active();
        assert!(active().is_empty());
        assert_eq!(active().layer_count(), 0);

        let mut sov = OverrideStore::new();
        sov.insert(mk_override("وائل", "وائل"));
        set_sovereign_layer(sov);

        let after = active();
        assert_eq!(after.layer_count(), 1);
        assert_eq!(after.len(), 1);
        let norm = super::super::normalizer::normalize("وائل").stripped;
        assert!(after.lookup(&norm).is_some());
    }

    // ── M9-hotpath (a): active_if_non_empty fast path ────────────────

    #[test]
    fn active_if_non_empty_returns_none_on_default_empty_store() {
        // Fresh baseline — no set_active has been called, so the
        // ACTIVE_STORE is the default empty one and ACTIVE_STORE_EMPTY
        // is `true`. The fast path must short-circuit to None without
        // touching the RwLock.
        let _g = RegistryGuard::new();
        assert!(active_if_non_empty().is_none(),
            "empty active store must take the fast-path None branch");
    }

    #[test]
    fn active_if_non_empty_returns_some_after_installing_nonempty_store() {
        // After set_active(store_with_entries), the fast path must
        // transition to returning `Some` and the entries must be
        // reachable via lookup.
        let _g = RegistryGuard::new();
        assert!(active_if_non_empty().is_none());

        let mut store = OverrideStore::new();
        store.insert(mk_override("وائل", "وائل"));
        set_active(store);

        let opt = active_if_non_empty();
        assert!(opt.is_some(),
            "non-empty store must be reachable via active_if_non_empty");
        let arc = opt.unwrap();
        let norm = super::super::normalizer::normalize("وائل").stripped;
        assert!(arc.lookup(&norm).is_some(),
            "the Some(Arc) must expose the installed entries");
    }

    #[test]
    fn active_if_non_empty_returns_none_after_clear_active() {
        // Transition back to empty: install non-empty, then clear_active.
        // The atomic must flip back to true so subsequent fast-path
        // readers skip the RwLock.
        let _g = RegistryGuard::new();

        let mut store = OverrideStore::new();
        store.insert(mk_override("وائل", "وائل"));
        set_active(store);
        assert!(active_if_non_empty().is_some());

        clear_active();
        assert!(active_if_non_empty().is_none(),
            "clear_active must restore the fast-path empty short-circuit");
    }

    #[test]
    fn active_if_non_empty_tracks_set_sovereign_layer_transitions() {
        // set_sovereign_layer is the CRUD path — editing / removing a
        // parent override. It must maintain ACTIVE_STORE_EMPTY
        // correctly so the fast path doesn't desync from the actual
        // store shape.
        let _g = RegistryGuard::new();

        // Start: empty. Fast path = None.
        assert!(active_if_non_empty().is_none());

        // Install a non-empty sovereign layer via set_sovereign_layer
        // (not set_active) — the CRUD-style entry point. Fast path
        // must switch to Some.
        let mut sov = OverrideStore::new();
        sov.insert(mk_override("وائل", "وائل"));
        set_sovereign_layer(sov);
        assert!(active_if_non_empty().is_some(),
            "set_sovereign_layer with content must flip the fast-path bit");

        // Replace with an empty sovereign — the "user removed the last
        // override" case. Fast path must return None again.
        set_sovereign_layer(OverrideStore::new());
        assert!(active_if_non_empty().is_none(),
            "set_sovereign_layer with empty content must restore fast-path None");
    }

    #[test]
    fn active_if_non_empty_with_child_layers_returns_some_even_when_sovereign_empty() {
        // Edge case: the sovereign layer is empty but a cUniverse child
        // has overrides. The active store is NOT empty as a whole, so
        // the fast path must return Some and the caller must consult
        // the child layer on lookup. This guards against a future
        // regression where someone mistakenly implements "empty" as
        // "sovereign only has no entries".
        let _g = RegistryGuard::new();

        let mixed = layered_store(vec![
            vec![], // empty sovereign
            vec![mk_override("خليفة", "from_child")], // non-empty child
        ]);
        set_active(mixed);

        let opt = active_if_non_empty();
        assert!(opt.is_some(),
            "non-empty child layer must keep the store observably non-empty");
        let arc = opt.unwrap();
        let norm = super::super::normalizer::normalize("خليفة").stripped;
        assert_eq!(arc.lookup(&norm).unwrap().lemma, "from_child");
    }

    #[test]
    fn active_if_non_empty_is_coherent_with_is_empty() {
        // Property-ish: for every state the active store can be in,
        // active_if_non_empty's Some/None return must agree with
        // active().is_empty() — they're two views of the same truth.
        let _g = RegistryGuard::new();

        // State 1: default empty.
        assert_eq!(active_if_non_empty().is_some(), !active().is_empty());

        // State 2: non-empty via set_active.
        let mut s = OverrideStore::new();
        s.insert(mk_override("وائل", "وائل"));
        set_active(s);
        assert_eq!(active_if_non_empty().is_some(), !active().is_empty());

        // State 3: cleared.
        clear_active();
        assert_eq!(active_if_non_empty().is_some(), !active().is_empty());

        // State 4: set_sovereign_layer with content.
        let mut s2 = OverrideStore::new();
        s2.insert(mk_override("ألف", "ألف"));
        set_sovereign_layer(s2);
        assert_eq!(active_if_non_empty().is_some(), !active().is_empty());
    }

    #[test]
    fn active_if_non_empty_some_branch_returns_same_arc_as_active() {
        // When the store is non-empty, both APIs return pointers into
        // the same underlying allocation — no deep clones, no parallel
        // stores.
        let _g = RegistryGuard::new();

        let mut s = OverrideStore::new();
        s.insert(mk_override("وائل", "وائل"));
        set_active(s);

        let fast = active_if_non_empty().expect("non-empty after set_active");
        let slow = active();
        assert!(Arc::ptr_eq(&fast, &slow),
            "active_if_non_empty() and active() must share the Arc");
    }
}
