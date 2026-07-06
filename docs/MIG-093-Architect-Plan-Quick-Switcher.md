# MIG-093 — Architect + Plan: the Quick Switcher (Ctrl+O), the pure title jumper

**Date:** 2026-07-06 · **Branch:** `main` · **Status:** Plan approved (Boss "go", 2026-07-06)
**Reproduce-First record:** Boss traces ×2 on the instrumented build (`52ea2e2d`):
`knowledge` → local 3.5ms (20 hits) / rust 29.3s; `islam` → local 3.6ms / rust 25.0s —
each showing TWO serialized searches (stale runs never cancelled, queued on the
`state.db` lock: `db_lock_wait` 2.4s/5.9s) on the `lexical(single,no-federation)`
path. Mechanism read off the code: FTS5 native `snippet()` on an external-content
table re-tokenizes the FULL body of EVERY matching row (SQLite materializes the
SELECT list before ORDER BY+LIMIT); a common term + M12 cross-language expansion
matches thousands of notes → ~20-27s per keystroke-pause. Federated mode dodged
this in MIG-058 Option H; the single-schema path never got the fix. `include_snippet`
exists in the protocol but was never read.

---

## 1. The concept (the horse)

> **Ctrl+O answers exactly one question — "take me to the note I can name" — instantly,
> from titles and aliases alone.** Content search is the Search Hub's owned question;
> the switcher never runs a content engine.

**Boss ruling (2026-07-06):** *"Ctrl+O should focus only on the titles. Do your research
on how other apps are processing this kind of plain search."* The embedded content
search wasn't just slow — it was a second search engine growing inside the switcher
(the same disease the Workbench Intent Bar was rejected for). The whole-entity map
(MIG-090 paper §1) already assigned the Quick Switcher exactly "jump by name."

## 2. The research (WA#5 cross-check — workflow `wf_7786efda-5db`)

- **Obsidian Ctrl+O:** names + frontmatter aliases ONLY, never content (official help);
  fuzzy subsequence scoring; empty query = recent notes; Enter-creates-note; content
  search is the separate Ctrl+Shift+F. (Its documented anti-pattern: a silent
  algorithm downgrade at 10k items.)
- **VS Code Ctrl+P:** file names only; `fuzzyScorer.ts` uses hard score BANDS —
  identity 1<<18, label-prefix 1<<17, other-label 1<<16 — so no fuzzy hit can ever
  outrank an exact title; recently-opened render first; content = Ctrl+Shift+F.
- **fzf/fzy/Sublime:** subsequence prefilter + affine-gap DP scoring (start/word-boundary/
  consecutive bonuses, gap penalties, shorter-candidate preference); 100k items in ms.
- **Notion/Logseq/Roam** (the mixing camp): content always DEMOTED below title hits
  (ranking, badges, "title only" filter) + a create row.
- **Local gaps found:** (a) the title cache (`cache.rs::read_notes`, line ~1212) reads
  ONLY the main schema — **cUniverse titles invisible → violates the ONE-universe
  ruling**; (b) aliases live in a separate graph-snapshot map, unsearchable by the
  switcher; (c) no shared frontend Arabic normalizer (an inline copy sits in
  IndexPanel.svelte:429); (d) a recents list already exists (`recentNotes.ts`).

## 3. Predecessor Lookup

| Predecessor | Disposition |
|---|---|
| QuickSwitcher's embedded content search (`constellationSearch` call, MIG-058/059 wiring incl. the stale-guard for 10s+ searches) | **DELETE** — replaced by the "Search ‹q› in Search Hub" escape row (same place, new hand-off) |
| The local substring filter (name OR path, cache-order, cap 20) | **REPLACE in place** — banded ranking over titles+aliases; path demoted to tie-break context only |
| `IndexPanel.svelte:429` inline `normalizeArabicForFilter` | **EXTRACT** to the shared `$lib` fold utility; IndexPanel re-points (same behavior) |
| `cache.rs::read_notes` main-schema-only SELECT | **FEDERATE** via the existing `get_federated_schemas` pattern (cache.rs:260) |
| QS-speed instrumentation (`LAST_SEARCH_TRACE`, `get_last_search_trace`, the panel diag line) | **REMOVE at §E** (after Boss validation) |

## 4. Invariants

- **Zero `invoke()` on the keystroke path** (Rule 3 / IPC contract): every keystroke is
  an in-memory scan; the ONLY IPCs left in the switcher are open-note / create-note /
  open-Search-Hub on explicit user action.
- **ONE-universe:** titles span active + federated cUniverses after §A.
- **Fold-for-matching, display-raw:** normalization never changes what's rendered;
  highlights map back to raw titles; RTL-safe (`detectDir`).
- **Parity:** the TS fold utility matches Rust `normalize_arabic_for_search` decisions
  (documented side-by-side) so Ctrl+O and the Search Hub agree on what matches.
- **No boot regression:** the federated title read is the same indexed SELECT per
  schema; measured before/after on the 7,600-note universe.
- **No note writes** (create routes through the existing create path + MIG-076 §E1b
  collision dialog). Content-integrity harness not triggered; full suite gates.
- **No silent scale cliff:** cap rendered rows (50, virtualized); the scan itself is
  O(n·query) over ~8k titles = single-digit ms; if universes grow 10×, adopt chunked
  scan — never Obsidian's silent downgrade.

## 5. The Plan (each § = one commit + verification)

- **§D Search Hub engine two-phase** *(first — restores compilation; the same engine
  serves Search Hub content queries which exhibit the identical 20-27s pathology).*
  `lexical_search_in_schema` = PHASE 1 rank (index-only bm25, no join/aux) → PHASE 2
  fetch details for ONLY the ≤limit winners; snippets synthesized in Rust (the proven
  Option-H path); FTS5 native `snippet()` eliminated; `want_snippet` honored end-to-end
  (`include_snippet=false` skips body_text entirely). *Verify: cargo build + tests;
  Search Hub `islam` content query sub-second on the Boss universe (was ~25s).*
- **§A Federated titles.** `read_notes` iterates main + attached schemas. *Verify:
  cargo test; count = Σ schemas; boot re-measured.*
- **§B Shared fold utility.** `$lib/utils/searchFold.ts` (case + Latin diacritics +
  Arabic fold, parity-documented); IndexPanel re-pointed. *Verify: unit tests EN+AR;
  svelte-check.*
- **§C The switcher rewrite.** Titles+aliases only; banded ranking (exact > prefix >
  word-boundary > fuzzy-subsequence) with tie-breakers (recency → compactness →
  shorter → collator); 1-2-char queries skip fuzzy; multi-word = all pieces must
  match; empty query = recents; cap 50 virtualized; pinned rows "Create note ‹q›"
  + "Search ‹q› in Search Hub"; RTL-safe highlights. Content search DELETED.
  *Verify: ranking unit tests (pinned case: query `islam` ranks title `Islam` #1,
  above "Abraham in Islam"); zero-IPC-per-keystroke traced; svelte-check.*
- **→ Boss test** (staged tutorial; instrumentation still visible so the numbers are
  readable).
- **§E Close-out.** Remove instrumentation; i18n ×15 for the new rows; docs + manual +
  Orientation bump; /simplify; boot + latency re-measure; log + PCS.

## 6. Cost & risk

~5 commits + close-out, one session. Risk is LOW: §D is a proven in-repo pattern
(Option H) restructured to the standard rank-then-fetch shape; §C is pure in-memory
frontend. Alias availability races (graph snapshot late) degrade gracefully — titles
always work. The M12 cross-language expansion still applies in the Search Hub
(content), NOT in Ctrl+O (titles are matched by folding, not by translation).
