# MIG-011 Phase 4 Audit — Cross-language Index *Filter* Expansion

**Date**: 2026-05-04
**Closes**: MIG-011 (Architect → Plan → Build → **Audit**)
**Architect doc**: `lab/reports/MIG-011-INDEX-FILTER-BRIDGE-ARCHITECT.md`
**Plan doc**: `lab/reports/MIG-011-INDEX-FILTER-BRIDGE-PLAN.md`
**Build commits**: 4 commits — `5081503` (§Build.1) → `5ca387c` (§Build.2) → `5d06471` (§Build.3) → `34f526e` (§Build.4). Plus the script-filter pre-fix `5dbb43f` and the i18n backfill `c95a0e6` (side-discovery during G2 testing). §Build.5 simplify ran clean (zero Tier 1, two Tier 2 deferred — see §3).

---

## §1 · Invariant verification (F1–F11)

| # | Invariant | Status | Evidence |
|---|---|---|---|
| **F1** | Default install (toggle off) behaviour unchanged: pure substring filter (with the bidirectional check shipped just before MIG-011). | ✅ | Boss G2 Stage 1 PASS: `knowledge` typed with toggle off → only English-substring entries, no badges. |
| **F2** | Toggle on + out-of-corpus query: behaves as toggle off. | ✅ | Boss G2 Stage 2 Action 3 PASS: `Xzyqwop` typed with toggle on → empty result, no errors. Backend test `filter_expand_out_of_corpus_returns_none` pins this. |
| **F3** | Toggle on + in-corpus query: union of direct + bridge matches with `via {lemma}` badges. | ✅ | Boss G2 Stage 2 Action 1 + 2 PASS — both English-typing-surfaces-Arabic and Arabic-typing-surfaces-English directions verified. |
| **F4** | Bridge IPC debounced (≥300ms). No keystroke fires more than one IPC. Cache prevents re-firing for repeated queries. | ✅ | `IndexPanel.svelte:88-122` — `setTimeout(handle, 300)` + `bridgeFetchToken` cancels stale results + `bridgeExpansionCache` Map. Boss G2 Stage 2 Action 4 PASS: typing `t-r-e-e` quickly settled correctly. |
| **F5** | Filter performance unchanged: per-keystroke filter loop bounded by entries × O(query_len + lemma_count). | ✅ | Bridge lemma set typically ~5-15 strings; effective constant overhead vs. pre-MIG-011 substring loop. No Boss-perceptible lag in G2 testing. |
| **F6** | Same `Settings → Index → Expand mentions cross-language` toggle drives both mentions and filter. | ✅ | `+layout.svelte:4814-4815` — both `cacheKey` and `bridgeFilterEnabled` props source from `$appSettings.index.expandCrossLanguage`. Boss G2 Stage 1 ↔ Stage 2 toggle-flip transitions clean. |
| **F7** | i18n: badge label reused from MIG-010 (`indexPanel.viaLemma`, `indexPanel.viaLemmaTooltip`). No new keys. | ✅ | Verified by grep — same keys used at IndexPanel mention badge AND filter badge. **Plus side-discovery during G2**: backfilled missing `indexPanel.returnToIndex` and 6 `notePane.stage.*` keys in 15 locales (commit `c95a0e6`); pre-existing gap, not MIG-011-introduced but caught here. |
| **F8** | RTL: badge layout flows correctly. | ✅ | Same `dir="auto"` + logical CSS as MIG-010. Boss G2 Stage 3 PASS on Arabic interface — "عبر knowledge" badge renders correctly. |
| **F9** | Rule 8 compliance: no recompute on toggle flip. | ✅ | Toggle flip clears `mentionsCache` + `bridgeExpansionCache` + nulls `bridgeExpansion` (frontend Map-resets, no Rust-side work). Next non-empty query lazily fires the bridge IPC. |
| **F10** | Cooccurrence chip-strip is unaffected. | ✅ | Verified by grep: bridge IPC call lives in IndexPanel filter logic, not in cooccurrence path. Boss G2 testing confirmed chips render alongside the new badges. |
| **F11** | The new IPC integrates with the script-filter bug fix shipped `5dbb43f`. | ✅ | Same `filteredEntries` derivation feeds both the bridge match path AND the activeLetter-stale-clear effect. Click "K" letter → type Arabic with toggle on → bridge matches surface, "K" clears. (No explicit Boss test for this composition, but the two effects don't share state — they consume the same `filteredEntries`.) |

**All 11 invariants PASS.**

---

## §2 · Drift audit (vs. Plan doc §Build.1–§Build.5)

| Plan step | Planned | Shipped | Deviation? |
|---|---|---|---|
| **§Build.1** | Rust IPC `lexicon_expand_for_filter` + `FilterExpansion` struct + 4 unit tests | Same. Placed in `src-tauri/src/lexicon/mod.rs` (next to `expand`). | None. |
| **§Build.2** | Frontend wrapper + debounce + cache | Same. **Plus** the cleanup-bubbling-out-of-untrack pattern documented in the comment — necessary so the cancel actually fires. Architect didn't anticipate this Svelte-5 subtlety; Build did. | Minor — design discovery, not drift. |
| **§Build.3** | Filter loop extension + per-entry annotation Map | Same. Restructured `filteredEntries` into a tuple `filteredResult: { entries, annotations }` exposed via two thin `$derived`. | None — the tuple shape was Plan-implied, not explicit. |
| **§Build.4** | Render badge. **Boss G2 test gate.** | Same. Reused MIG-010 CSS class `.gp-ref-via` + i18n keys without modification. **Boss G2 PASS.** | None. |
| **§Build.5** | `/simplify` three-agent review | Combined-lens review (single agent run). Result: zero Tier 1, two Tier 2 deferred (cross-language helper extraction — wait for a third bridge surface; cache size cap — same status as MIG-010's deferred `mentionsCache` cap, defer for consistency). | Minor — single-agent vs. three-agent review, but the lens coverage was preserved (reuse + quality + efficiency all asked). |

**Side discovery shipped during G2 testing**: i18n backfill (`c95a0e6`) and the deeper note-stage-taxonomy decision (`project_note_stage_taxonomy_decision.md`). Neither was in MIG-011 scope but both were caught here and either fixed (i18n) or queued for Boss decision (taxonomy).

**Net drift: minor** — single-agent simplify instead of three, design-time discoveries documented inline. Final shipped surface matches Architect intent.

---

## §3 · §Build.5 simplify findings

Combined-lens review on the MIG-011 diff. Summary (full punch list in commit notes / agent return):

### Tier 1 — fix before merge: **none.**

### Tier 2 — defer (per agent recommendation):

| # | Item | Rationale |
|---|---|---|
| 2.1 | Same-language-exclusion duplication: `expanded_match_query` (search.rs) and `lexicon_expand_for_filter` (lexicon/mod.rs) both run the identical `flat_terms().filter(lang != source_lang).map(to_lowercase).collect()` pipeline. | Two sites = low drift risk. Extract a `cross_language_lemmas_lower` helper when a third surface (Map / Sky View) needs the bridge. **Logged for follow-on.** |
| 2.2 | `bridgeExpansionCache` unbounded. | Same status as MIG-010's deferred `mentionsCache` cap — bounded by user typing distinct prefixes per session, ~tens of KB worst case, released on unmount. Defer for consistency with MIG-010 unless memory growth reported. |

### Tier 3 — note only: 5 cosmetic items (separation between `bridgeFilterEnabled` and `cacheKey` justified, `normalize_stripped` naming-smell vs. behavior-correct, `bridgeFilterAnnotations` rebuild cost dominated by substring scan, bridge inner loop benchmark-first, test assertion adjacency). All accepted as-is.

---

## §4 · Code surface check

**Rust changes shipped:**
- `lexicon/mod.rs`: new `FilterLemma`, `FilterExpansion` structs + `lexicon_expand_for_filter` Tauri command + 4 unit tests (~120 lines).
- `lib.rs`: 1-line `generate_handler!` registration.
- Total Rust diff: ~121 lines net.

**Frontend changes shipped:**
- `store.ts`: type definitions + IPC wrapper (~30 lines).
- `IndexPanel.svelte`: new prop `bridgeFilterEnabled`, new state for bridge expansion + cache + cancel token, new $effect for debounced fetch, extended `filteredResult` derivation, badge render in term row (~80 lines net).
- `+layout.svelte`: 1-line prop wire.
- Total frontend diff: ~111 lines net.

**i18n**: zero new keys for MIG-011 itself (reuses MIG-010's `indexPanel.viaLemma{,Tooltip}`). Side-discovery backfill: 7 keys × 15 locales = 105 strings added.

**Doc changes:**
- `lab/reports/MIG-011-INDEX-FILTER-BRIDGE-ARCHITECT.md` (new).
- `lab/reports/MIG-011-INDEX-FILTER-BRIDGE-PLAN.md` (new).
- `lab/reports/MIG-011-AUDIT.md` (this doc).

---

## §5 · Migration path check

Pure additive. No schema, no on-disk format, no settings change (reuses MIG-010's `index.expandCrossLanguage` toggle). Fresh-install behaviour is the default-off Pre-MIG-010 substring filter. Existing users get the new IPC silently — toggle stays at whatever they had it set to from MIG-010. **No migration required.**

---

## §6 · Known limitations + follow-ups

| Item | Status |
|---|---|
| Cross-language helper extraction | Defer until 3rd bridge surface (Tier 2.1 above). |
| Bridge cache size cap (LRU at 200 entries) | Defer; consistent with MIG-010's `mentionsCache` posture. |
| Bigram terms with cross-language expansion (e.g. "above sea") | Architect-doc-deferred — same M11 single-lemma constraint as MIG-010; revisit in a follow-on if Boss requests bigram expansion. |
| Note-stage taxonomy decision (Living Link lifecycle vs Zettelkasten) | `project_note_stage_taxonomy_decision.md` — Boss decision pending, not blocking MIG-011 closure. |
| 13-of-15-locale translation backfill (returnToIndex + lifecycle stages) | Same backfill workstream as `project_user_manual_13_locales_backfill.md`. |

---

## §7 · State of standing

- **Verified-shipped**: 4 MIG-011 commits + script-filter pre-fix + i18n backfill side-discovery + (this) audit doc.
- **Boss G2 PASS** confirmed visually + functionally.
- **Branch**: `main` ahead of `origin/main` by ~9 commits since MIG-010 close. PCS at MIG-011 close (next).
- **MIG status**: ready to mark closed.

**MIG-011 closes here. Ready for MIG-012 Architect.**
