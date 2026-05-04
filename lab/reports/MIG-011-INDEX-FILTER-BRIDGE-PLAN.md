# MIG-011 — Phase 2: Build Plan

**Companion to**: `MIG-011-INDEX-FILTER-BRIDGE-ARCHITECT.md`
**Phase**: 2 (Plan) → cascades to Phase 3 (Build) on Boss approval
**Build steps**: 5 commits + simplify checkpoint + Phase 4 audit doc.

---

## §0 · Reading guide

Each Build step lands as one commit with format `§MIG-011 §Build.N — short title`. Each step has a **verification clause** — what observably proves the step worked. Boss-testable steps pause for tutorial-style test instructions per the Testing Instructions Rule; internal-only steps cascade.

The plan reuses MIG-010's surface as much as possible: the `Settings → Index → Expand mentions cross-language` toggle, the `via {lemma}` badge style, the i18n keys (`indexPanel.viaLemma` / `viaLemmaTooltip`). MIG-011 adds one new IPC and three new bits of frontend state.

---

## §Build.1 — Rust IPC `lexicon_expand_for_filter`

**Surface**: `src-tauri/src/lexicon/mod.rs` (or a new top-level Tauri command file in libraries.rs/search.rs — TBD during build, file-placement is implementation detail).

**Changes**:
1. New `#[tauri::command] pub fn lexicon_expand_for_filter(query: String) -> Result<Option<FilterExpansion>, String>`.
2. New struct `FilterExpansion { source_lemma: String, source_lang: Lang, lemmas: Vec<FilterLemma> }` where `FilterLemma { lemma_lower: String, lang: Lang }`.
3. Logic: normalize query (same `normalize_stripped` MIG-010 used) → `lexicon::detect_source_lang` → if Some, `lexicon::expand` with default opts → flatten to `Vec<FilterLemma>` filtered to **non-source-language** lemmas (M13 same-language exclusion rule), each `lemma.to_lowercase()` for substring matching.
4. Return `None` when: query is empty, normalization yields empty, no source lang detected, or the expansion has no cross-language terms.
5. Register in `tauri::generate_handler!` (lib.rs).
6. **Tests** (added in same commit): 4 unit tests modeling the cases — out-of-corpus returns None, in-corpus returns FilterExpansion with non-source lemmas, source language is correctly detected for English vs Arabic queries, all lemmas are pre-lowercased.

**Verification**: `cargo check` clean. New tests pass. Existing 12/12 M13 tests still pass (the function reuses M13's filter primitives).

**Boss-testable**: No — backend-only. Cascade to §Build.2.

---

## §Build.2 — Frontend bridge wrapper + debounce + cache

**Surface**:
- `src/lib/libraries/store.ts` — new types `FilterLemma`, `FilterExpansion`. New wrapper `lexiconExpandForFilter(query: string): Promise<FilterExpansion | null>`.
- `src/lib/components/IndexPanel.svelte` — new state `bridgeExpansion: $state<FilterExpansion | null>(null)`. Debounced effect on `filterQuery` (300ms) that calls the IPC when toggle is on, caches results in a session-scoped Map.

**Changes**:
1. Wrapper: `invoke('lexicon_expand_for_filter', { query })`.
2. Cache: `Map<string, FilterExpansion | null>` keyed by lowercased query. Empty queries skip.
3. Debounce: 300ms. Cancel in-flight on next keystroke.
4. Toggle-aware: when toggle is off, no IPC, `bridgeExpansion` stays null.
5. Cache cleared when toggle flips (reuses the existing `cacheKey` invalidation pattern — extend it to clear `bridgeExpansion` too, OR add a separate `$effect` that watches the toggle directly via `cacheKey` — choose during build).

**Verification**: `cargo` + svelte-check clean. Manual: open dev console; type rapidly in the filter; verify only one IPC fires per ~300ms pause.

**Boss-testable**: No (no UI surface yet — bridge data is fetched but not yet rendered). Cascade to §Build.3.

---

## §Build.3 — Filter loop extended with bridge lemma matching

**Surface**: `src/lib/components/IndexPanel.svelte` — `filteredEntries: $derived.by(...)`.

**Changes**:
1. When `bridgeExpansion !== null`, the per-entry filter loop ALSO checks: does `entry.term.toLowerCase()` equal or contain any of `bridgeExpansion.lemmas[].lemma_lower`?
2. On match, annotate the entry with the source lemma that bridged it (the `bridgeExpansion.source_lemma`). This is per-render annotation; we don't mutate the original entries array. Use a parallel Map `bridgeAnnotations: Map<string, string>` keyed by entry.term, populated during the filter pass.
3. Sort: direct substring matches first, then bridge matches. (Within each group, existing sort applies.)

**Verification**: With toggle on and the IPC primed, type "knowledge" in the filter — list now includes Arabic terms like "معرفة" (substring "معرفة" doesn't contain "knowledge", but "knowledge" expands to Arabic equivalents and "معرفة" is one of them).

**Boss-testable**: No (badges not yet rendered — entries appear in the list but identical to direct matches visually). Cascade to §Build.4.

---

## §Build.4 — Render `via {lemma}` badge on cross-language filter entries

**Surface**: `src/lib/components/IndexPanel.svelte` — the entry-row template.

**Changes**:
1. When rendering an entry, look up `bridgeAnnotations.get(entry.term)`. If present, render the same `via {lemma}` chip used in MIG-010's mentions list — same CSS class `.gp-ref-via` (need to verify the class is reachable from the term-row scope; if not, hoist).
2. Reuse `$t('indexPanel.viaLemma')` and `$t('indexPanel.viaLemmaTooltip')` — already shipped in MIG-010 for 15 locales.
3. Visual: small chip after the term name, before the count. Same muted-purple style.

**Verification**: G-test gate. Boss verifies:
- Toggle on → type "knowledge" in filter → English terms appear with no badge, Arabic terms (`معرفة`, `علم`) appear with `via knowledge` badge.
- Toggle off → type "knowledge" → only English terms (substring match), no badges.
- Type Arabic term with toggle on → English terms surface with `via معرفة` badge.

**Boss-testable**: Yes — full end-to-end. **Tutorial test instructions emitted at this step.**

---

## §Build.5 — `/simplify` checkpoint

Run three review agents on the §Build.1 → §Build.4 diff: code-reuse, code-quality, efficiency. Address Tier 1 + Tier 2 findings inline; surface Tier 3 for Boss.

**Verification**: agent reports clean OR all Tier 1+2 findings resolved + Tier 3 explicitly deferred.

**Boss-testable**: No — internal review pass.

---

## §Audit — Phase 4 (separate doc: `MIG-011-AUDIT.md`)

Three audit lenses:
1. **Invariant verification** (F1–F11 from Architect) — each checked against shipped code + Boss test result.
2. **Drift audit** — compare shipped to plan; flag deviations.
3. **Code surface check** — Rust diff size; frontend diff size; locale coverage (no new keys; reuses MIG-010's). Migration path: no schema/format change.

---

## §X · Boss approval gates

| Gate | What Boss approves | When |
|---|---|---|
| **G1 — Plan approval** | This plan as written; treats §Build.1 → §Build.5 + §Audit as one autonomous cascade. | Now (Phase 2 → Phase 3). |
| **G2 — End-to-end (§Build.4)** | Cross-language filter results appear with correct `via {lemma}` badges. RTL clean. | After §Build.4 commit lands. |
| **G3 — Closure** | Audit doc verified; MIG-011 marked closed in memory + orientation. | After §Audit. |

Stops at G2 only (single Boss-test gate). §Build.1, .2, .3, .5 cascade without pause.

---

## §Y · Out-of-scope (deferred follow-ons)

- **Search history** — queued as MIG-012 (`project_index_search_engine_history_semantic.md`).
- **Semantic search** — MIG-012.
- **Boot-time bridge corpus dump** (option 3.B from Architect) — explicitly rejected; only revisit if Audit shows a perf regression.

---

**Phase 2 closes here. Plan-Approval = Build-Approval applies. Cascading to §Build.1 immediately upon Boss approval — and Boss has already said "Proceed all," which I'm reading as approval. Will pause only at G2 (after §Build.4) for the Boss test.**
