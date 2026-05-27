# MIG-058 + MIG-059 — Combined fix (SHIPPED)

**Status:** Shipped 2026-05-27 (commit `05f0e474`).
**Priority:** P2. Closes two issues from the MIG-056 §K Boss-test: federation search latency + Arabic input truncation.

## Why combined

Two issues, two different root causes, but the fix surface is small for both and they share one verification cycle (rebuild + Eisa retest). The first MIG-059 attempt was a guess (pre-warm via `SELECT COUNT(*) FROM notes_fts`) that broke federation; it got reverted. This time the design is sourced from four parallel research agents covering SQLite FTS5 internals, federated-search per-shard patterns from production systems, SQLite multi-Connection same-file behavior, and Svelte 5 + IME + WebView2 interactions.

## MIG-059 — Federated search ~25s → ~1s

### Root cause (sourced)

**Query-planner-driven, not page-cache-driven.** SQLite FTS5 uses static cost estimates by default and picks catastrophic plans for OR-of-MATCH expressions — exactly our 9-term lexicon-expanded multilingual query. Documented fix on the SQLite Forum:

- Dan Kennedy's thread "JOINs with FTS5 are very slow" — running `ANALYZE` cut a similar query from 170s to 0.259s (660× speedup). [Source](https://sqlite.org/forum/info/509bdbe534f58f20).
- Hipp's measurement: ANALYZE on a 93MB DB takes ~161ms.
- Per the SQLite docs: *"The query planner loads the content of the statistics tables into memory when the schema is read."* [Source](https://sqlite.org/lang_analyze.html).

`Connection A` (active-mode) was fast because `init_db`'s migrations implicitly populated `sqlite_stat1` and Connection A loaded those stats at schema-parse. `Connection B` (per-cUniverse standalone, opened in §K.3) opens the same file but the planner falls back to static estimates → bad OR-of-MATCH plan → 25× slowdown.

**Bonus:** SQLite multi-Connection page caches are SEPARATE 64 MB allocations by default (not shared). `PRAGMA mmap_size` moves the page cache to OS-shared mmap — eliminates the duplicate-cache asymmetry between the ATTACH-based `federated_conn` (warm) and the standalone per-cUniverse Connection (cold). [Source](https://sqlite.org/mmap.html).

### Backend changes (`src-tauri/src/search.rs`)

1. **`init_db`: `PRAGMA optimize;`** at end. Populates `sqlite_stat1` on the active universe's `search.db`. <1ms when stats already current; ~160ms when ANALYZE actually runs. Subsequent Connections opening the same file inherit good planner stats.

2. **Per-cUniverse Connection: `PRAGMA mmap_size=268435456` (256 MB)** added to the PRAGMA batch. OS-shared page mapping eliminates the cold-cache asymmetry.

3. **Per-cUniverse Connection: `PRAGMA optimize`** after tokenizer registration. Refreshes planner stats for the standalone Connection's view. Documented <1ms if stats current.

### What was rejected (and why)

- ❌ The reverted `SELECT COUNT(*) FROM notes_fts` pre-warm stays reverted. Agent 3's research confirmed `COUNT(*)` is O(n) on FTS5 segments (must query each individually and merge), not O(1) metadata read — which is why the previous attempt hung federation. [Source: Fedor Indutny's FTS5 structure write-up](https://darksi.de/13.sqlite-fts5-structure/).
- ❌ The `PRAGMA wal_checkpoint(PASSIVE)` is gone. PASSIVE "doesn't block" other connections but itself runs slowly when nothing is cached. Wrong tool here.

## MIG-058 — Arabic input truncation in QuickSwitcher

### Root cause (sourced)

**NOT what was hypothesized.** Agent 4's research found:

- **Svelte 5's `bind_value` source explicitly guards against the value-rewrite-during-typing race** I assumed. The source has `if (input === document.activeElement) return` — never rewrites a focused input. AND `if (value !== input.value)` — never rewrites a no-op. The "async resolve breaks IME composition" hypothesis was a phantom. [Source: bind_value source on github main](https://github.com/sveltejs/svelte/blob/main/packages/svelte/src/internal/client/dom/elements/bindings/input.js).

- **Arabic 101 keyboard on Windows is a DIRECT keyboard layout, not an IME.** No `compositionstart`/`compositionend` events fire for Arabic. So the React #34485 / Vue v-model "gate on composition events" fix pattern (which solves CJK input bugs) does NOT apply here. [Source: kbdlayout.info/KBDA1](http://kbdlayout.info/KBDA1/).

The actual cause is **synchronous main-thread pressure**: every keystroke triggered the `filtered` `$derived.by` rebuild (walk 1101 notes + lowercase + substring-match + slice + merge with `extendedResults` + slice) AND the keyed `{#each filtered ... (note.path)}` re-render. Under WebView2 main-thread pressure, Arabic keystrokes can drop at slow typing speeds. Same family of bugs as [CodeMirror discuss #9741](https://discuss.codemirror.net/t/chinese-ime-punctuation-input-loses-every-other-keypress-requires-2-presses-per-character/9741) and [Tauri #3136](https://github.com/tauri-apps/tauri/discussions/3136).

### Frontend changes (`src/lib/components/QuickSwitcher.svelte`)

1. **`filtered` changed from `$derived.by(...)` to `$state<...[]>([])`.** Typing no longer triggers synchronous filter + re-key. The list updates only once per 300ms debounce window. Substring-match logic moved INSIDE the same debounced `$effect` that calls `constellationSearch`.

2. **`selectedIndex = 0` reset moved INSIDE the debounced effect.** The old separate `$effect(() => { if (filtered) selectedIndex = 0; })` fired on every filtered change including async resolves; that contributed to mid-typing reactive churn.

3. **`oncompositionstart`/`oncompositionend`** added to the input. Sets a `composing` flag; the debounced effect skips while composing. Free insurance for CJK / Indic / any IME-composed input. No effect for Arabic 101 (no composition events), no harm anywhere.

### What was rejected

- ❌ Switch from `bind:value` to manual `oninput` — the Svelte 5 source proves `bind:value` isn't at fault.
- ❌ "Gate on composition events" as the PRIMARY fix — Arabic 101 doesn't fire these events; the gate would be a no-op for the actual reported bug.

## Tests

- 836/836 lib tests pass (no regression on the backend changes).
- `svelte-check`: 0 new errors in `QuickSwitcher.svelte`. 3 pre-existing baseline errors unchanged (LinkLifecycle 'fresh' + 2× PropertyEditor type narrowing).

## Verification path (Boss-test)

1. **Federation status bar** should show federated note count within ~5 seconds of boot — no longer blocked on the reverted pre-warm.
2. **First federated search** for `الرباط` should return in ~1 second instead of ~25 seconds, with the same result quality as MIG-057's verified output (الرباط at rank 1-2, geography cluster following).
3. **Arabic slow-typing** should land all characters when typed at 300-400ms per character. Type `الرباط` slowly; input should show the full 6-character word post-typing.

## Open questions deferred

- Whether WebView2 fires `compositionstart` for Arabic specifically wasn't definitively answered by primary sources. Agent 4 recommended a 30-second devtools test, but Constellation release builds disable devtools (per `feedback_devtools_dev_only.md`). If the Boss-test for Arabic slow-typing still fails after this commit, that test becomes the next investigation step (likely from a dev build).

## Sources

**SQLite side:**
- [SQLite Forum: JOINs with FTS5 very slow → ANALYZE fixed 170s→0.259s](https://sqlite.org/forum/info/509bdbe534f58f20)
- [SQLite Forum: Bad query plans from FTS5 (static cost estimates)](https://sqlite.org/forum/info/e0e30e9eb1998e3c9305aea26957bec804615283969d11c1f9326a6b787526eb)
- [SQLite ANALYZE docs](https://sqlite.org/lang_analyze.html)
- [SQLite PRAGMA docs (`optimize`, `mmap_size`)](https://sqlite.org/pragma.html)
- [SQLite mmap.html](https://sqlite.org/mmap.html)
- [SQLite shared-cache (obsolete)](https://sqlite.org/sharedcache.html)
- [Fedor Indutny: FTS5 structure](https://darksi.de/13.sqlite-fts5-structure/)

**Federated-search patterns:**
- [Lucene IndexReaderWarmer](https://lucene.apache.org/core/7_3_1/core/org/apache/lucene/index/IndexWriter.IndexReaderWarmer.html)
- [Solr Caches and Query Warming](https://solr.apache.org/guide/solr/latest/configuration-guide/caches-warming.html)
- [Elasticsearch CCS gateway pool](https://www.elastic.co/docs/explore-analyze/cross-cluster-search)
- [Postgres FDW keep_connections](https://www.postgresql.org/docs/current/postgres-fdw.html)
- [Citus slow-start adaptive executor](https://docs.citusdata.com/en/stable/performance/performance_tuning.html)

**Svelte / IME / Arabic input:**
- [Svelte 5 `bind_value` source (focused-input guard + no-op short-circuit)](https://github.com/sveltejs/svelte/blob/main/packages/svelte/src/internal/client/dom/elements/bindings/input.js)
- [Svelte issue #13196 (bind:value + composition)](https://github.com/sveltejs/svelte/issues/13196)
- [MDN compositionstart](https://developer.mozilla.org/en-US/docs/Web/API/Element/compositionstart_event)
- [Win32 Arabic 101 keyboard layout (no IME)](http://kbdlayout.info/KBDA1/)
- [CodeMirror discuss 9741 — Chinese IME keypress loss in WebView2](https://discuss.codemirror.net/t/chinese-ime-punctuation-input-loses-every-other-keypress-requires-2-presses-per-character/9741)
- [Tauri discussion #3136 — broken accents/diacritics](https://github.com/tauri-apps/tauri/discussions/3136)
- [Vue forms guide (v-model IME handling)](https://vuejs.org/guide/essentials/forms.html)
