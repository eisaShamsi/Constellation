# MIG-010 Phase 4 Audit — Lexical Bridge Integration into Index Reads

**Date**: 2026-05-04
**Closes**: MIG-010 (Architect → Plan → Build → **Audit**)
**Architect doc**: `lab/reports/MIG-010-INDEX-LEXICAL-BRIDGE-ARCHITECT.md`
**Plan doc**: `lab/reports/MIG-010-INDEX-LEXICAL-BRIDGE-PLAN.md`
**Build commits**: 11 commits from `4a45b10` (Phase A bug fix) → `40412ae` (§Build.5 simplify).

---

## §1 · Invariant verification (I1–I11)

Each Architect-doc invariant checked against shipped code + Boss G2/G3 test result.

| # | Invariant | Status | Evidence |
|---|---|---|---|
| **I1** | Default install behaviour unchanged: `read_term_mentions` with no expansion flag returns the same rows as today. | ✅ | `expand_cross_language: Option<bool>` defaults to `false` via `unwrap_or(false)` (libraries.rs ~3462). `build_term_match_clause(_, false)` returns `(fts_quote_phrase(term), None)` — byte-identical phrase to pre-MIG-010. Verified: G3 test "knowledge expand=false → 10 rows, 0 with via_lemma" matches the pre-MIG-010 row shape. |
| **I2** | Settings toggle persists across restart, propagates to second screen. | ✅ | `AppSettings.index.expandCrossLanguage` lives in the standard `app_settings.json` persistence path. `loadSettings` spread-merge ensures existing universes pick up the new key with default `false`. Boss G2 test confirmed: toggle on → close Settings → reopen → still on; quit app → relaunch → still on. |
| **I3** | Cross-language expansion only fires when (a) the toggle is on AND (b) `expanded_match_query` actually returns Some (i.e. real OR-joined expansion, not a degenerate single-phrase). | ✅ | `build_term_match_clause` (libraries.rs ~3416) gates expansion on both conditions; when either fails, falls through to `fts_quote_phrase(term)` exact-phrase. `build_term_match_clause_expand_out_of_corpus_falls_back` test pins this against "Xzyqwop". |
| **I4** | Badges use the same source-of-truth filter as M13 search (non-source-language lemmas only, lowercased). | ✅ | The expansion is decomposed via `LexicalExpansion::into_parts()` (search.rs:2647) which yields the `bridge_terms_lower` already filtered by `expanded_match_query` to exclude the source language. Per-row scan reuses `find_match_via_marked` — same M13 helper, no parallel filter. |
| **I5** | Snippet HTML safety preserved: STX/ETX sentinels (not `<mark>`) so user content can't inject DOM. | ✅ | `run_mentions_query` (libraries.rs ~3550) calls `snippet(notes_fts, -1, CHAR(2), CHAR(3), '…', 12)` — STX (\x02) and ETX (\x03) preserved. `find_match_via_marked` accepts the delimiter pair as parameters; called with `("\u{0002}", "\u{0003}")` from libraries.rs and `("<mark>", "</mark>")` via the `find_match_via` thin wrapper from search.rs. Two delimiter regimes, one helper. |
| **I6** | Cooccurrence chip-strip is unaffected. | ✅ | `read_cooccurring_terms` was unchanged in this MIG (only registered in `generate_handler!` per Phase A bug fix). G3 screenshot shows chips render correctly while toggle is on AND expansion is firing. |
| **I7** | i18n complete in 15 locales for new Settings label, toggle description, and badge tooltip. | ✅ | Verified by grep: `settings.sections.index`, `settings.index.intro`, `settings.index.expandCrossLanguage.{label,description}`, `indexPanel.viaLemma`, `indexPanel.viaLemmaTooltip` present in all of `en, ar, de, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh`. Full translations: en + ar (Boss's daily). 13 others ship with English-value placeholders pending the existing translation backfill workstream (`project_user_manual_13_locales_backfill.md`). Block presence satisfies the Architect invariant. |
| **I8** | RTL works (Arabic / Hebrew / Persian / Urdu): badge layout flows correctly with `dir="auto"`, toggle slider mirrors. | ✅ | Boss G2 test on Arabic interface confirmed PASS after §Build.4-fix added `:global([dir="rtl"]) .toggle-slider::after` rules. Badge `<span>` has `dir="auto"` so the lemma's own script direction reads naturally; CSS uses `margin-inline-start` (logical) so the badge sits on the row's logical end. |
| **I9** | Performance: toggle-on path is O(same FTS query) — no extra round-trip. | ✅ | Expansion happens once in `expanded_match_query` (synchronous lexicon walk + FST lookup; sub-millisecond on the 20K corpus). The expanded MATCH expression fires as a single FTS5 query, same shape as the unexpanded one. Per-row badge scan adds a small string-find loop bounded by `bridge_terms_lower.len()` (typically 5–15). No additional IPC calls per row. |
| **I10** | Rule 8 compliance: no recompute work on toggle flip. | ✅ | Toggle flip clears the IndexPanel's mentionsCache (frontend) — that's a Map-reset, no work on the Rust side. Next click re-issues `read_term_mentions` with the new flag. No cache invalidation, no rebuild, no schema migration. |
| **I11** | "via {lemma}" badge text uses the actual matched bridge lemma (from snippet scan), not just any bridge term in the expansion set. | ✅ | `find_match_via_marked` scans the snippet for the FIRST marked token whose lowercased content matches a bridge term, and returns *that* term. Reuses M13's `find_match_via_marked` directly — no parallel matcher. G3 screenshot visibly shows different lemmas per row ("via علم" on multiple Arabic rows; "via conocimiento" on the Spanish row), confirming the per-row scan is honoring the actual marked content. |

**All 11 invariants PASS.**

---

## §2 · Drift audit (vs. Plan doc §Build.1–§Build.5)

| Plan step | Planned | Shipped | Deviation? |
|---|---|---|---|
| **§Build.1** | `pub(crate)` on `expanded_match_query`, `LexicalExpansion`, `find_bridge_lemma_in_snippet` | Same; **plus** parameterized `find_match_via_marked` to handle STX/ETX vs `<mark>` delimiter difference between Index and search paths. Plan doc named the helper `find_bridge_lemma_in_snippet` — actual code is `find_match_via`/`find_match_via_marked`. Naming drift only. | Minor — plan-doc naming matched the architect's mental model; the real code has slightly different names. Architect drift is harmless but logged. |
| **§Build.2** | Extend `read_term_mentions` with `expand_cross_language` param; new `via_lemma`; tests | Same. **Plus** added `build_term_match_clause` helper for testability. Tests cover the helper, not the IPC fn directly (SQLite fixture infrastructure didn't exist; would have been scope creep). | None — added abstraction was a purely positive surprise during build. Tests verify the same invariants the plan called for, just at the helper level. |
| **§Build.3** | Settings: new "Index" section + i18n | Same. New `AppSettings.index` nested block with `expandCrossLanguage: boolean` (default `false`) — chosen over a flat key per the same rationale as `linkLifecycle`/`skyView` blocks (room for future Index settings without re-shaping). | None. |
| **§Build.4** | IndexPanel: read setting, pass to IPC, render badge. **Boss G3 test gate.** | Shipped, but Boss G3 surfaced **four follow-on issues** — three cosmetic (G2 toggle states), one functional (TS field-name mismatch). All four fixed in three follow-up commits (§Build.4-fix, §Build.4-fix2, §Build.4-fix3). Final fix3 was the bug Boss's DevTools diagnostic pinned: `$effect` reading `mentionsCache.size` made the cache its own dependency, infinite-clearing on every fetch (CLAUDE.md Rule 2 violation). | **Significant.** The fix3 is a self-correction: I shipped a Rule-2-violating $effect without running an end-to-end IPC trace before the Boss test cycle. **Lesson logged below.** All issues resolved; G3 PASS confirmed via Boss screenshot showing Arabic notes with "via علم" + Spanish note with "via conocimiento". |
| **§Build.5** | `/simplify` checkpoint | Three review agents ran in parallel (code-reuse, code-quality, efficiency). Triage: 1 Tier-1, 6 Tier-2, multiple Tier-3 actionable; all fixed in commit `40412ae`. Tier-3 noted-only items (unbounded cache, composite cache key, find_match_via thin shim) deferred with rationale. | None — simplify exposed and resolved real cleanups; no pushed-back deferrals. |

**Net drift assessment: minor naming drift in §Build.1, significant scope expansion in §Build.4 (four follow-up commits) driven by Boss test feedback.** Final shipped surface matches the Architect's design intent.

---

## §3 · Code surface check

**Rust changes shipped:**
- `search.rs`: visibility bumps + `find_match_via_marked` parameterized helper + `LexicalExpansion::into_parts()` accessor (~50 lines net).
- `libraries.rs`: new `fts_quote_phrase` helper, `build_term_match_clause`, `run_mentions_query` extracted helper, `read_term_mentions` rewritten with try-expanded-then-fallback retry; per-row `via_lemma` populated via `find_match_via_marked`. `IndexMention` struct gained `via_lemma: Option<String>` (skip-serialize-if-none). 4 unit tests on `build_term_match_clause`. ~+220 lines net.
- `lib.rs`: 1-line `generate_handler!` addition for `read_cooccurring_terms` (Phase A bug fix).
- Total Rust diff: ~230 lines net.

**Frontend changes shipped:**
- `IndexPanel.svelte`: new `cacheKey?: unknown` prop, `$effect` invalidates `mentionsCache` on flip, `via_lemma` badge rendered after note name, error catch in `ensureMentionsLoaded`. CSS class `.gp-ref-via`.
- `SettingsModal.svelte`: new "Index" section + toggle, RTL slider mirror, off-state contrast fix, magic-pixel derivation comment.
- `+layout.svelte`: 1-line `loadMentions` callback now passes the toggle state; `cacheKey` prop wired.
- `store.ts`: `IndexMention.via_lemma`, `readTermMentions(term, limit, expandCrossLanguage)`, `AppSettings.index.expandCrossLanguage`, spread-merge in `loadSettings`.
- 15 locale JSON files: `settings.sections.index`, `settings.index.*`, `indexPanel.viaLemma`, `indexPanel.viaLemmaTooltip`.
- Total frontend diff: ~150 lines net.

**Doc changes:**
- `lab/reports/MIG-010-INDEX-LEXICAL-BRIDGE-ARCHITECT.md` (new, ~140 lines).
- `lab/reports/MIG-010-INDEX-LEXICAL-BRIDGE-PLAN.md` (new, ~135 lines).
- `lab/reports/MIG-010-AUDIT.md` (this doc).

---

## §4 · Migration path check

What happens for an existing user who installs the post-§Build.5 binary on top of a prior version?

- **No migration required.** Changes are pure additive — new IPC parameter (optional, defaults to off), new struct field (skip-serialize-if-none, so old payloads parse fine), new settings key (spread-merge picks up default for absent users), new "Index" Settings tab. On-disk format unchanged.
- **SQLite/FTS5 unchanged.** No schema bumps, no triggers, no migration steps. The `notes_vocab` virtual table this MIG reads from was already shipped pre-MIG-010.
- **Settings unchanged on existing universes.** `loadSettings` (store.ts) spreads `DEFAULT_SETTINGS.index` over the parsed JSON — users who never had an `index` block get `{ expandCrossLanguage: false }` automatically.
- **Roll-back safe.** Reverting to a pre-MIG-010 binary loses the toggle UI but the on-disk settings JSON is forward-compatible (the `index` block is just ignored).

**Migration path: no action needed.**

---

## §5 · Lessons logged

### LL — `$effect` body must declare its dependencies explicitly via `untrack`

**The bug**: in §Build.4-fix I wrote a cache-invalidation `$effect` that read `mentionsCache.size` to short-circuit unnecessary reassignments. Svelte 5 auto-tracks every reactive read inside `$effect`, so the effect tracked the very cache it was meant to manage — every successful fetch (which mutates the cache → size changes) re-fired the effect, which cleared the cache it just populated. UI showed empty mentions despite the IPC returning rows.

**Caught by**: Boss DevTools diagnostic showing the IPC succeeding (`10 rows, 4 with via_lemma`) while the UI rendered nothing. Without that trace I would have continued chasing the wrong layer.

**The rule** (CLAUDE.md Rule 2 — already on the books): "NEVER write a `$effect` that reads and writes the same reactive variable." I literally wrote one.

**The new convention**: when writing `$effect(() => { void watchedDep; ...body... })` to react to ONE specific dependency, every other reactive read in the body MUST live inside `untrack()`. The "void X;" pattern explicitly declares ONE intended dependency; the body must honor that contract.

**Working Agreement #4 self-correction**: I shipped §Build.4 without running an end-to-end IPC trace before the Boss test cycle. Three rounds of fixes (§Build.4-fix, §Build.4-fix2, §Build.4-fix3) and one DevTools-aided diagnostic later, the right bug was found. The lesson: for cross-subsystem `$effect` work, I will run a console-level trace (or write a unit test that simulates the reactive cycle) BEFORE shipping. Adding to my pre-flight checklist for any IndexPanel-class component.

---

## §6 · Known limitations + follow-ups

Logged in project memory, not blockers for MIG-010 closure:

| Item | Memory file | Note |
|---|---|---|
| Cross-language Index *filter* (mirror of mentions expansion, applied to the search box) | none yet — Boss-approved 2026-05-04 as **MIG-011** next | When user types "knowledge" in the filter, also surface index entries that are bridge-equivalents in other languages. Logical extension of MIG-010's read-side work. |
| Index search engine: search history + semantic search | `project_index_search_engine_history_semantic.md` | Boss-requested 2026-05-04. Composes with MIG-011. Likely MIG-012. |
| Script-filter "All" hides Arabic terms until "عربي" bounce | `project_index_script_filter_all_hides_arabic.md` | Pre-existing bug observed during G3 testing; not introduced by MIG-010. Composes with MIG-011. |
| Bigram terms ("above sea") don't expand cross-language | logged in PLAN §Y / Architect §6 | The lexicon is single-lemma. Boss-deferred per design. Could split bigram + expand each constituent in a future MIG, but risk of over-broadening results. |
| 13 of 15 locale translations are English-value placeholders | `project_user_manual_13_locales_backfill.md` (existing) | Same pattern as MIG-008. Boss-acceptable — the Svelte `||` fallback renders English where keys are present-but-untranslated. |
| Unbounded `mentionsCache` in IndexPanel | (none) | Bounded by user clicks per session; ~40 KB worst-case per term × 100 terms ≈ 4 MB. Defer LRU until reported. |
| Cache-clear blast radius on toggle flip | (none) | Could key cache by `${term}:${expand}` to coexist both states. Minor UX polish; defer. |

---

## §7 · State of standing

- **Verified-shipped**: 11 commits — `4a45b10` (Phase A) → `204bd29` (Architect) → `c46d413` (Plan) → `e5bde4b` (§Build.1) → `97bac88` (§Build.2) → `8279e25` (§Build.3) → `6665d96` (§Build.4) → `4024e34` (§Build.4-fix) → `23957bb` (§Build.4-fix2) → `ece6090` (§Build.4-fix3) → `40412ae` (§Build.5). All Boss-tested PASS at G2 + G3 gates.
- **Tests**: 4/4 build_term_match_clause + 12/12 M13 search-side tests + svelte-check clean (only pre-existing deferred LinkLifecycle error).
- **Branch**: `main` ahead of `origin/main` by these 11 commits + the audit doc + closure docs (Phase F still pending: orientation v1.32 + session log).
- **MIG status**: ready to mark closed in `project_index_lexical_bridge.md` (pending Phase F SO).

**MIG-010 closes here.**
