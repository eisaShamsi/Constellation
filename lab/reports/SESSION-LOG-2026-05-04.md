# Session Log — 2026-05-04

## Phase A — Cooccurrence IPC bug fix (`4a45b10`)

`libraries::read_cooccurring_terms` existed and was wired from frontend (`store.ts:2429`) but never registered in `tauri::generate_handler!`. Index chip-strip cooccurrence panel was silently broken. One-line fix in `lib.rs`. Verified compiles, no behaviour change otherwise.

---

## Trivial cleanup pre-MIG-010 (`3d1b224`)

While scoping the M11/M9/dead-code work-streams, found three stale doc comments still referencing the M10/early-M11 seed ("49 hand-curated concepts", "2,500+ across 63 shards") even though the M11-data v2 producer had shipped — `lexicon_v1.tsv` is 20,015 lines, 499 shards. Plus `ExpansionResult::total()` had zero callers. Three comment fixes + one dead-code deletion. `cargo check` clean.

---

## MIG-010 — Lexical Bridge integration into the Index panel

**Boss directive 2026-05-04**: "What I want is to finish and implement the Index function. So, whatever is related to it has to be done."

### Phase 1 — Architect (`204bd29`)

Mapped current Index read paths (libraries.rs `read_term_mentions`, `read_index_entries`, `read_cooccurring_terms`), current Bridge wiring (search.rs `expanded_match_query`, `LexicalExpansion`, `find_match_via`), current Settings shape. Three design axes enumerated (IPC param shape, visibility for bridge helpers, badge UX, Settings tab placement). 11 invariants defined. **Boss-approved Option β**: exact-match by default + Settings toggle for cross-language expansion. New "Index" tab in Settings.

### Phase 2 — Plan (`c46d413`)

5 build steps + simplify checkpoint + audit. Each step landable as one commit, each with verification clause.

### Phase 3 — Build

| Commit | Step | Highlight |
|---|---|---|
| `e5bde4b` | §Build.1 | `pub(crate)` on `expanded_match_query`, `LexicalExpansion`; new `find_match_via_marked` parameterized helper (handles STX/ETX vs `<mark>`). 12/12 M13 tests pass. |
| `97bac88` | §Build.2 | `read_term_mentions` extended; new `via_lemma` field; `build_term_match_clause` helper + 4 unit tests. |
| `8279e25` | §Build.3 | Settings UI: new "Index" section + toggle. i18n in 15 locales (en + ar full; 13 placeholders per backfill workstream). G2 Boss-test gate. |
| `6665d96` | §Build.4 | IndexPanel reads setting, renders badge. G3 Boss-test gate. **First Boss test surfaced 4 issues.** |
| `4024e34` | §Build.4-fix | G2 cosmetics fixed (off-state contrast, RTL slider mirror) + TS field-name `viaLemma` → `via_lemma` (Tauri doesn't auto-convert struct fields). |
| `23957bb` | §Build.4-fix2 | Defensive backend (try expanded → fallback to exact phrase on error or 0 rows) + frontend error catch in `ensureMentionsLoaded`. |
| `ece6090` | §Build.4-fix3 | **Real G3 root cause**: cache-invalidation `$effect` read `mentionsCache.size` → tracked the cache as a dependency → infinite-clearing on every fetch. Wrapped cache reads in `untrack()`. CLAUDE.md Rule 2 violation. |

**G3 PASS confirmed by Boss screenshot**: Arabic-titled notes with `via علم` badges + Spanish "Ada Lovelace" with `via conocimiento` badge. 7,600-note mixed Arabic/English library now genuinely browseable by concept across languages.

### Phase 3.5 — `/simplify` (`40412ae`)

Three review agents in parallel (code-reuse, code-quality, efficiency). Triage: 1 Tier-1, 6 Tier-2, multiple Tier-3 actionable. Fixes:
- Tier 1: IndexPanel prop renamed `expandCrossLanguage` → `cacheKey?: unknown` (decouples child from parent setting semantics).
- Tier 2: `LexicalExpansion::into_parts()` accessor (fields private again); `fts_quote_phrase` helper (DRY); flatten 3-arm match in `read_term_mentions`; `prepare()` → `prepare_cached()`; `eprintln!` gated behind `cfg!(debug_assertions)`; `clone()` → borrow on `bridge_terms_lower`.
- Tier 3: magic-pixel derivation comment in toggle CSS.

Tier 3 noted-only items (unbounded `mentionsCache`, composite cache key, `find_match_via` thin shim) deferred with rationale.

### Phase 4 — Audit (`c4c4bf4`)

`lab/reports/MIG-010-AUDIT.md`. All 11 invariants verified PASS. Drift audit: minor naming drift (Plan said `find_bridge_lemma_in_snippet`; code is `find_match_via_marked`); significant scope expansion at §Build.4 driven by Boss test feedback (3 follow-up commits). Migration path: no action needed (additive only, settings spread-merge).

---

## Phase D — Defer Index boot load (`d4f99da`)

`+layout.svelte` $effect was firing `readIndexEntries()` on `graphReady` regardless of whether the user opened the panel. Gated on `showIndex`. Boot becomes free; cost paid on first Index open.

## Phase E — Help docs (`7206655`)

Three documentation surfaces:
- `docs/User Manual.md` §7 — appended cross-language mentions subsection.
- `docs/help.ar/User Manual.md` §8 — same in Arabic (Boss's daily).
- `docs/help.uConstellation.World/Index/Index.md` — new dedicated help page (the Index didn't have one).

13 other locale User Manuals queued in existing backfill workstream.

## Phase G — Index guidance teaching doc (`faf9a99`)

`docs/help.uConstellation.World/Index/Index Guidance — How to Read Your Vocabulary.md`. Boss-pattern teaching doc (parallel to the queued 360.3D matrix guidance). Six sections: framing, three diagnostic reads, common patterns + readings, writing prompts, what-the-Index-won't-tell-you, weekly practice ritual.

## Phase F — Standing Order (this commit)

Orientation v1.32 created (copied v1.31, prepended `v1.32` change block). This session log written. PCS pending.

---

## Memory files added today

- `project_index_search_engine_history_semantic.md` — Boss-requested MIG-012 territory: search history + semantic search.
- `project_index_script_filter_all_hides_arabic.md` — pre-existing bug observed during G3 testing.
- `project_index_rebuild_button_decision.md` — Rebuild button explicitly deferred per Rule 8.

---

## Lessons logged

**LL-024** (added in MIG-010 audit, this session log): `$effect` body must declare its dependencies explicitly. The §Build.4-fix3 root cause was a CLAUDE.md Rule 2 violation that I shipped without an end-to-end IPC trace. **New rule for cross-subsystem `$effect` work**: run a console-level trace BEFORE the Boss test cycle. The Boss DevTools diagnostic was what caught this — Working Agreement #4 demands I run that kind of trace upfront.

---

## State of standing (close-of-session)

- **Verified-shipped**: 11 MIG-010 commits + Phase D + Phase E + Phase G + this Phase F (orientation/log) + the Phase A bug fix and pre-MIG cleanup.
- **Branch state**: `main` ahead of `origin/main` by ~17 commits. PCS push pending.
- **Boss-approved follow-on workstreams**: MIG-011 (cross-lang Index filter), MIG-012 (Index search engine: history + semantic), pre-existing script-filter bug.
- **Boss-deferred**: Rebuild Index button (per Rule 8).
- **Backlog (pre-existing, unchanged)**: 13-locale User Manual backfill, reserved-Windows-name + trailing-dot/space hardening, collision popup, pre-§140 cid_cn collision scrub utility, Outgoing Links display case fix, Unlinked Mentions panel cleanups (alias bleed + double-count), `LinkLifecycle 'fresh'` Boss-deferred TS error.

**MIG-010 closes here. Ready for MIG-011 when Boss is.**
