# Session Log — 2026-07-06 (Quick Switcher speed · reproduce-first)

**Branch:** `main` · **Predecessor:** MIG-092 DONE (`a617def4`; see SESSION-LOG-2026-07-05.md).

## Function in hand: the Quick Switcher (Ctrl+O) — retrieval speed

**Boss symptom (2026-07-06):** typing is fine, but **getting results takes 2+ MINUTES
with heavy thrashing.** That is a pathology, not inherent slowness.

**Reproduce-First:** no fix designed/shipped until the delay is reproduced under
instrumentation and the mechanism is read off the trace.

### Path mapped (code-read, not yet the diagnosis)
`QuickSwitcher.svelte` (fed the in-memory `allNotes` cache): 300ms debounce →
instant local substring filter → for ≥3-char queries **`await constellationSearch`**
(mode `lexical` via `parseSearchQuery`) → `search.rs::constellation_search`
(`(async)`, holds `state.db.lock()` for the duration) →
`federated_lexical_search_or_fallback` → sequential per-schema FTS5 branches
(`main` + every attached cUniverse) on the shared `federated_conn` → RRF merge.
**The instant local title hits are held back behind the awaited federated search.**
MIG-058's own comment records "10+s on cold federated FTS5" — the Boss sees 2+ min.

**Candidate mechanisms the trace must separate:** (a) `state.db` lock WAIT
(contention/pile-up — matches the thrashing), (b) per-branch federated FTS5 cost
(which cUniverse), (c) merge/other.

### Instrumentation shipped (TEMPORARY) — commit `52ea2e2d`
- `search.rs`: `LAST_SEARCH_TRACE` phase log per `constellation_search` —
  `db_lock_wait`, `execute`, `federated_conn_lock_wait`, `branch:main`,
  `branch:cuN`…, + the fallback paths. New `get_last_search_trace` command.
- `QuickSwitcher.svelte`: per-run on-screen diagnostic line (devtools is OFF in
  release): `run #N · local Xms (hits) · rust Ys [phase trace]`; stale runs
  report too (pile-up visibility).
- No retrieval behavior changed. cargo check + svelte-check clean.

### Reproduction + diagnosis (Boss traces ×2)
- `knowledge`: local 3.5ms (20) · rust 29.3s; `islam`: local 3.6ms (20) · rust 25.0s.
  Both show TWO serialized runs (stale searches never cancelled; `db_lock_wait`
  2.4s/5.9s) on `lexical(single,no-federation)`. **Mechanism (code-confirmed):**
  FTS5 native `snippet()` on the external-content table re-tokenizes the FULL body
  of EVERY matching row (SELECT list materialized before ORDER BY+LIMIT); common
  term + M12 expansion → thousands of full-body tokenizer passes per keystroke-pause.
  `include_snippet` declared but never read. Federated mode dodged this (Option H);
  the single-schema path never got the fix.
- **Boss rulings:** *"Ctrl+O is useless… should focus ONLY on the titles. Do your
  research on how other apps process this kind of plain search."* Stop-On-Correction
  honored (in-flight §1 threading frozen; state summarized; research run).
- **Expected-hit finding:** the wanted note had the term in its TITLE — the old local
  filter (name+path, cache order, cap 20) buried/dropped exact title matches
  (library literally named "Eisa Cognitive Knowledge" floods path matches).

### Research (workflow `wf_7786efda-5db`, Boss-directed)
Obsidian Ctrl+O = names+aliases ONLY (never content; Enter-creates; empty=recents) ·
VS Code Ctrl+P = names only, banded fuzzyScorer (identity 1<<18 / prefix 1<<17 /
label 1<<16) · fzf/fzy = subsequence + position bonuses + gap penalties · peers that
mix (Notion/Logseq/Roam) always DEMOTE content. **Local gaps:** allNotes read main
schema only (ONE-universe violation); aliases unsearchable; no shared frontend fold;
recents list already exists.

## MIG-093 — Ctrl+O the pure title jumper (Boss "go") — docs/MIG-093-Architect-Plan-Quick-Switcher.md

### §D — two-phase lexical search — commit `32550fd0`
Rank-first (index-only bm25, no join/aux) → fetch details for ONLY the ≤limit
winners; snippets Rust-synthesized (Option-H path); FTS5 native snippet() GONE;
`include_snippet` honored end-to-end. **Verify:** cargo suite **1012/1012**.

### §A — federated titles — commit `1f0fe817`
`read_notes` → `read_notes_in_schema` looped over `get_federated_schemas` on
`federated_conn` (graceful main-only when the boot races the attach); the
`federation:ready` handler now re-fetches the core snapshot into `allNotes`
(shrink-overwrite guarded). **Verify:** cargo check + svelte-check clean.

### §B — shared fold utility — commit `407af8e6`
`$lib/searchFold.ts` (`foldForMatch` + `stemArabicLight10`, Rust-parity documented);
IndexPanel's 55-line inline copy re-pointed. **Verify:** tests 10/10.

### §C — the switcher rewrite — commit `02579dde`
`$lib/switcherRank.ts` banded ranking (exact>prefix>word-boundary>fuzzy; recency/
compactness/shorter/collator tie-breaks; alias penalty+dedupe; 1-2-char fuzzy skip;
multi-word all-match) + QuickSwitcher.svelte rewritten (titles+aliases only, folded
once per cache refresh, MIG-058 debounce kept at 100ms, empty=recents, pinned
Create-note + Search-in-Hub rows, per-title dir). Content search DELETED from the
switcher. +layout: create/search-hub hand-offs + aliases prop.
**Verify:** svelte-check 0 errors; full suite **245/245** incl. the pinned Boss case
(query `islam` → exact title "Islam" #1).

### Boss test — ALL PASS (with one §D-2 discovery)
- **Ctrl+O measured by the Boss:** `islam` **2.7ms · 50 hits · 9,161 candidates**
  (was 25.0s + heavy thrashing); `knowledge` **1.6ms** (was 29.3s). Exact title
  "Islam" #1. Arabic fold works (`الزراعة` → alias row `الزِراعة → الزراعة المستدامة`).
  Recents, Create-note row, Search-Hub hand-off: all pass.
- **Test 5 discovery:** the Search Hub was STILL slow + light thrashing — its
  default mode calls `constellation_search_universal`, a DIFFERENT command whose
  `search_contents` had the identical materialize-snippets-before-LIMIT disease
  (amplified by limit=200). **§D-2** (commit `bb46ea81`): same two-phase cure;
  NO production native-snippet() path remains in search.rs (grep-verified).
  **Boss re-test: "Much better. Pass."**

### §E — close-out
- **§E-1** (commit `1ffb9076`): instrumentation removed end-to-end (Rust trace +
  command + registration; the QS diag line + CSS); `quickSwitcher.recent/
  createNote/searchInHub` ×15 locales (workflow `wf_05140239-9b9`, all validated);
  User Manual §5 "Quick Switcher (Ctrl+O)" section; **Orientation v3.27** (new
  file: preamble + §8 Migrations row).
- **§E-2** (commit `3366e0ca`): /simplify (4 agents) — reuse/simplification/
  altitude clean (altitude's one flag was a false positive: the §A federation:ready
  re-fetch already shipped at +layout:3021); efficiency: Intl.Collator hoisted to
  module scope (APPLIED); documented skip: hasExactMatch second scan
  (~2ms/keystroke at a 100ms debounce; not worth the pure-module API churn).
- **Measures record (honest):** keystroke latency Boss-measured 1.6-2.7ms over
  9,161 federated candidates (the Rule-3 budget met with ~40× headroom). Boot: no
  separate formal measure taken; the §A read adds one indexed name/path/library
  scan per attached schema (the graph payload's existing shape) and the Boss
  booted 6+ times through testing with no perceived regression. FileTree/typing
  untouched.

### MIG-093 — DONE. Shipped + Boss-validated + closed out.
Before/after: Ctrl+O 25-29s+thrash → 1.6-2.7ms · Search Hub content ~25s → ~1s
("much better", calm) · exact titles rank #1 · federated titles visible ·
aliases searchable · recents on empty · create + search hand-off rows.

---

## PJ-069 — The whole-entity deduplication pass (concept-paper-first)

### Kickoff — SO#8 cross-check + adversarial re-audit
- **SO#8 cross-check ran BEFORE any work** (workflow `wf_2ae0f8c0-d59`, 18 agents:
  3 context readers + 7 cluster finders + 7 adversarial verifiers + 1 completeness
  critic; ~2.3M tokens). Every 2026-07-05-map count was re-verified against the LIVE
  tree — the map was drawn the same day the Navigator was deleted and predates
  MIG-092/093, so it was stale.
- **Recomposed counts (map → today):** tags 6→6 (a *different* six), folders 4→5,
  recents 3→2 hand-rolls, orphan/fragile 5→9 surfaces / 5 divergent definitions,
  hubs 3→6 in-degree substrates, note-lists 9→26 live (NoteRow/NoteList shipped, 1
  adopter), **confidence 2→0** (already unified by the shared ConfidencePicker,
  MIG-077 §F `fa98bf6b`, six days before the map).
- **Load-bearing corrections:** "Sight is disabled" is over-broad — CNS v2
  (ConstellationSight2 + SightPanel) is LIVE core; only Sight v3/v4/v6/v7 + the
  Constellation *Map* frontend are off. `map.rs`'s hierarchy builder is LIVE via the
  default-on OrgChart (the reachable OrgChart runs `constellation_map_universe`, not
  `read_library_tree`). Confidence cluster already at 0.

### Concept paper + ratification
- **Concept paper:** `docs/concept-papers/PJ-069-Whole-Entity-Deduplication-Concept-Paper.md`.
- **Boss rulings (2026-07-06):** horse = **"one home per capability"** (structural
  framing, not the semantic single-answer one) · priority = **answer-duplication
  first** · scope = **the 7 clusters + the 9 newly-found cross-cluster families** ·
  first step = **the dead-code cull**.
- **Pending Jobs → v1.17** (`docs/Constellation Pending Jobs v1.17.md`; v1.16
  preserved below the new preamble): the PJ-069 entry recomposed, its four stale
  parts corrected (dead surfaces in the counts, obsolete MIG-090 coordination clause,
  overstated confidence-menu ×2, events it predates).

### Step 0 — dead-code cull (verified before any deletion)
- **Deletion-safety verification** (workflow `wf_4a972c1a-736`, 9 adversarial
  verifiers + an assembler) produced the exact bounded edit set per item, the
  do-not-touch live-sibling list, and flagged **two Boss gates** (TagsPanel
  delete-vs-revive; the `lenses.rs` full-delete that executes the parked 2026-05-09
  Option-A decision). Caught name-collisions that look deletable but are live
  (`crate::lens` Bases engine vs dead `crate::lenses`; live `store.ts::scanLibraryTags`
  vs dead `tagUtils::scanAllLibraryTags`) and 3 cascade-dead files.

- **§0a — frontend dead components + orphan i18n:**
  - DELETED `src/lib/components/NoteGrid.svelte` (259 lines, never mounted) + its dead
    import at `+layout.svelte:95`.
  - Removed the dead `scanLibraryTags` import token at `+layout.svelte:24` (imported,
    never called; siblings kept).
  - `tagUtils.ts`: removed `scanAllLibraryTags()` (fs-walk retired by MIG-080 §B) + its
    now-unused imports; KEPT the `DashboardTag` interface (used by DashboardView).
  - `utils.ts`: removed the zero-caller `extractTags()`.
  - `libraries.rs`: corrected the stale `scan_library_tags` docstring (its boot-path
    claim about `DashboardView.onMount → scanAllLibraryTags` is false since MIG-080 §B).
  - i18n ×15: removed `secondScreen.{searchNotes,allLibraries,sortName,sortLibrary,
    noResults,noNotes}` (NoteGrid-owned), `lens.tagEdges` (the verifier mislabeled its
    parent as `skyview` — corrected on the fly), `panels.{tagsAll,tagsThisNote}` (the
    removed All/This-note toggle). Minimal diffs (files are `stringify(obj,null,2)+\n`).
  - **Verify:** `npm run check` → 0 ERRORS (324 pre-existing unused-CSS warnings); all
    15 locales parse; grep of the removed symbols/keys = 0 in the primary tree.

- **§0b — /libraries route + NavBar cascade + orphaned store exports + nav i18n:**
  - DELETED the unreachable legacy `/libraries` route (`src/routes/libraries/+page.svelte`
    + the dir), `NavBar.svelte` (imported by zero components), and `LanguageSwitcher.svelte`
    (imported only by NavBar) — a 3-file cascade the MIG-091 §retire sweep missed.
  - Removed the 4 store exports whose ONLY consumer was that route: `timeAgo`,
    `selectedNote` (derived), `searchResults` (writable), `searchAllStars` (+ its
    `_searchStarsSeq` guard). Re-verified route-only first: every other `searchResults`
    hit is a component-local `$state` in ConstellationMap/Sight2 (name-collision, not the
    store import); the live `relativeTime()` in searchHistory.ts is a different symbol, kept.
  - i18n ×15: removed the whole `nav` namespace (home/libraries/skills, NavBar-owned).
  - **Discovery (WA#6):** removing `searchAllStars` orphaned the `search_stars` Rust
    command (its only invoker) → folded into §0c.
  - **Verify:** `npm run check` → 0 ERRORS (warnings 324→320 = the route's CSS gone;
    file count −3); all 15 locales parse.

- **§0c — Rust dead IPC (map.rs zombie + bases.rs live-scan path + search_stars):**
  - `map.rs`: removed the registered-but-never-invoked single-library `constellation_map_data`
    (101 lines) + its `lib.rs` registration. KEPT `constellation_map_universe` (LIVE via the
    default-on OrgChart) and every shared helper (`build_library_node`, `build_tree`,
    `collect_notes_recursive`, `compute_*`, `load_alias_map`, `MapNode`); fixed 2 stale
    comments that named the deleted command.
  - `bases.rs`: removed the retired live-scan cluster — `query_base` (unregistered) +
    `scan_folder`/`scan_by_tag`/`apply_filters`/`apply_sorts_fixed` + the now-orphaned
    `BaseRow`/`BaseQueryResult` structs (307 lines). The live replacement is
    `lens::query::execute_lens` (MIG-065 §I); the frontend already calls it. KEPT the LIVE
    `parse_frontmatter`, `parse_base_file`, `minimal_base_yaml`, and every registered command.
  - `libraries.rs`: removed `search_stars` (orphaned by §0b's `searchAllStars` removal —
    WA#6) + its private `search_notes_recursive` helper (117 lines) + the `lib.rs`
    registration. KEPT `StarInfo` (LIVE via `notes_by_tag` + the `recent_stars` field) and
    `safe_truncate` (5 live callers); fixed a stale `query_base` comment.
  - **Verify:** `cargo check` green (39.5s); **zero NEW dead-code warnings** — none of the
    54 pre-existing warnings names any deleted symbol (proves the whole dead cluster went, no
    orphan left). Frontend already clean from §0b; no `invoke` of any deleted command remains.
  - **§0c orphan caught in §0d compile:** `query_base` was the only user of `use std::time::
    Instant` (bases.rs:5) → removed the now-unused import (folded into §0d).

- **§0d — boot_bundle bookmarks dead fetch (the migration command KEPT):**
  - `boot_bundle.rs`: removed the `BootBundle.bookmarks` field, its `read_universe_bookmarks`
    fetch, and its struct-literal entry — the value nothing reads post-MIG-092 (Bookmarks
    unified into the Starred collection).
  - Frontend parity: dropped `bookmarks: unknown[]` from the local `BootBundle` type +
    updated the two comments.
  - `universe.rs`: corrected the `read_universe_bookmarks` docstring (loadCollections is now
    its sole reader).
  - **KEPT (verified):** the `read_universe_bookmarks` command + its `lib.rs:567` registration
    + the `readUniverseBookmarks` wrapper + `store.ts:1046`'s migration invoke — the one-time
    Bookmarks→Starred migration still needs them.
  - **Verify:** `cargo check` green (11.7s, Instant warning gone); `npm run check` → 0 ERRORS.

- **§0e — bases dead-command sweep (WA#6, the pending 2026-05-29 physical sweep):**
  The §0d compile surfaced 4 known-dead unregistered `bases.rs` commands (SESSION-LOG-2026-05-29
  §337 listed them as "uncallable but present (cargo warns). Physical sweep pending"; MIG-055
  replaced the family). This §0 cull IS that sweep. Removed `parse_base_file`, its exclusive
  helper `parse_base_yaml`, `save_base_file`, and `parse_workspace_base` (62 lines) — all
  unregistered, zero frontend invoke. KEPT the shared helpers verified live: `validate_base_path`
  (shared with the live `convert_base`), `workspace_bases_dir` (5 live commands), `format_yaml_value`
  (used by `convert_base`), `minimal_base_yaml`, `convert_filter_op`.
  - **Verify:** `cargo check` green (31.2s); all 4 dead-command warnings gone, zero new warnings.

### Step 0 — Boss gate rulings (2026-07-06)
- **TagsPanel.svelte → KEEP** (Boss) as the tags-cluster seed. It's dead now (zero imports since
  2026-06-20) but is the only hierarchical tag-tree builder left; the tags /migration decides its
  fate. NOT deleted.
- **lenses.rs → DELETE confirmed** (Boss) — execute the parked 2026-05-09 Option-A ruling.

- **§0f — lenses.rs full CE-Phase-9 Multi-Lens delete (Boss-confirmed gate):**
  - Rust: DELETED `src-tauri/src/lenses.rs` (276 lines) + `mod lenses` + the `list_lenses`/
    `save_lenses`/`apply_lens` registrations. `cargo check` green (38.7s).
  - Frontend `+layout.svelte`: removed the never-populated lens view (`availableLenses`/
    `activeLensId`/`lensGroups`/`lensEntries` derived + the `list_lenses` invoke + the dead
    sidebar render branch — if/else chain rejoined to `{:else}` (Bookmarks) + the create-lens
    command-palette entry + the `.lens-select` CSS).
  - Frontend `SettingsModal.svelte`: the Lenses UI was the **entire** "Knowledge Management"
    settings section → removed the whole section + its nav tab (an empty tab would be a defect;
    flagged to Boss). Removed the lens state + `loadLenses`/`saveLensItem`/`deleteLens`.
  - i18n ×15: removed `lensPanel.*`, `commands.createLens`, `settings.sections.knowledge`, and
    `settings.knowledge.*` — **KEPT `settings.knowledge.create`** (shared with SenseMakingCanvas).
  - **Line drawn (verified, untouched):** the LIVE `lens/` Bases engine (`src/lib/lens/`,
    `LensBlockWidget`, `bases.rs` imports `crate::lens::definition`) and `sight.rs` (CNS analytics)
    — both share the word "lens" but have zero dependency on the deleted `lenses.rs`.
  - Memory `project_lenses_apply_lens_dead_code` updated to "executed."
  - **Verify:** `cargo check` green; `npm run check` → 0 ERRORS (warnings 320→316 = lens CSS gone).

### Step 0 — DONE (§0a–§0f, all build-green). Cull total: −1,992 lines of dead code, 5 files deleted.
