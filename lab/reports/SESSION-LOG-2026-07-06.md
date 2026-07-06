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
