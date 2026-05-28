# MIG-058 + MIG-059 — Combined fix (SHIPPED)

**Status:** Shipped 2026-05-28 (commit `c426af7e` + supporting commits).
**Resolved by:** Options **C + G + H** combined, after 8 iterations that pruned 7 incorrect hypotheses.

## What both MIGs were about

- **MIG-058** — QuickSwitcher Arabic input truncation: when the user typed Arabic at normal pace (300-400ms per char), the input box showed only the first few characters. Pasting or typing very fast worked.
- **MIG-059** — Federated search latency: 15-25 seconds per first federated FTS5 search on Eisa's cu1 (Eisa Cognitive Knowledge, 7650 notes), vs ~1 second when the same data was the active universe. Same file, same query, same FTS5 engine.

Both were caused by the same root issue: federated `lexical_search` was slow, and the slow async resolve was blocking the IPC/event loop long enough to buffer Arabic keystrokes in WebView2. Fix the speed, both close together.

## The actual root cause

SQLite FTS5's native `snippet()` function — used in the SQL to extract highlighted excerpts — does the following for EACH matching result row:

1. Fetch `body_text` from the external content table (`note_meta`).
2. **Re-tokenize the entire body_text** via the registered tokenizer to find positions of the matched tokens.
3. Extract a window around the best match, with `<mark>` tags around hits.

Constellation uses a custom `constellation` FTS5 tokenizer that does Arabic Unicode normalization, diacritic stripping, stopword filtering, and bigram emission. Running this expensive tokenizer over 30 result rows × kilobytes of body_text per row = ~16 seconds.

The cost scales with **result count** (30 rows), not with **segment count** or **page cache state** — which is why Options E (PRAGMAs), F (page-cache pre-warm), and G (FTS5 segment merge) couldn't move the needle: they all addressed Connection / index state, but the dominant cost was per-row tokenizer work.

Active mode (when EC Knowledge is the user's active universe) is fast because `state.db` is the only Connection serving everything, and FTS5's `snippet()` benefits from incidental warming via other operations earlier in the session. Federated mode opens a fresh path where nothing has warmed the per-row callback path.

## The shipped fix

Three commits, building on each other, all in production code on `main`:

### Option C — Per-schema queries on the warm `federated_conn` (`fb83797e`)

**Architecture cleanup.** Drop the §K.3 per-cUniverse standalone-Connection pool entirely. Use the ATTACH-based `federated_conn` (which has every cUniverse mounted as `cu0` / `cu1` / ...) for ALL federation paths: libraryStats, lens, AND federated FTS5 search.

The §G/§K.2 prohibition on `bm25(schema.notes_fts, ...)` in UNION ALL queries does NOT apply to single-schema queries with `FROM cu1.notes_fts`. In that context, unqualified `bm25(notes_fts, ...)` correctly resolves to the FROM-clause table — verified by 4 new unit tests in `mig056_federated_search::option_c_*`.

The scatter-gather coordinator runs one single-schema query per attached cUniverse, all on the SAME `federated_conn`, merged via RRF in Rust. One Connection, one warm cache, no pool.

### Option G — Background FTS5 segment merge (`4cbdd56a`)

**Periodic FTS5 maintenance, but never run for cUniverses.** The active universe's `init_db` writes 1 row to `note_meta` every boot via `mig003_step3_soft_rebackfill`. Those writes incrementally merge FTS5 segments. cUniverses never see that boot-time write, so their FTS5 indexes accumulate segments without ever being merged.

After federation attaches (and after `federated_conn` is saved to state, so federation visibility isn't blocked), spawn a background thread `federation_prewarm` that opens a throwaway Connection to each cUniverse's `search.db` and runs `INSERT INTO notes_fts(notes_fts) VALUES('optimize')` — the FTS5-documented segment merge command.

Cost: ~30-60s of background work on first invocation per cUniverse (for a 7650-doc index). Persistent: the merged state survives across boots. Subsequent invocations are 0ms (already merged). Idempotent.

**Important note on the contribution of Option G:** Option G alone did NOT fix the perf issue (Eisa's boss-test showed segment merge ran successfully but search timing didn't improve). However, Eisa noted that search RESULT QUALITY improved after Option G — likely because BM25 ranking is more accurate on a non-fragmented index. We kept Option G in the shipped state because it (a) costs nothing on subsequent boots, (b) improves ranking quality, and (c) reduces FTS5 maintenance debt for cUniverses generally.

### Option H — Bypass FTS5 `snippet()` in federated mode (`c426af7e`)

**The decisive fix.** Added a `skip_fts5_snippet: bool` parameter to `lexical_search_in_schema`. Federated mode (`true`) selects raw `body_text` instead of calling `snippet()`. Rust then computes the snippet via `synth_snippet_for_body` — a UTF-8 char-boundary-safe substring scan that finds the query (or a bridge term from the lexicon expansion) in body_text and extracts a ±40-character window with `<mark>` tags.

Active mode (`false`) still uses FTS5's native `snippet()`, which is fast on the warm `state.db` Connection.

Federated snippets are slightly less precise than FTS5 native:
- FTS5 matches stemmed/inflected tokens (it knows what the tokenizer would emit).
- Rust matches literal substrings.

In practice: query for "trees" highlights "trees" in federated mode, may highlight "tree" in active mode. The COUNT and RANKING of results is unchanged — only `<mark>` placement differs.

## What was rejected (and what it taught us)

| Option | Hypothesis | Why it failed | Lesson |
|---|---|---|---|
| §K.1 | Tokenizer not registered on federation Connection | Necessary but not sufficient — Stage 4 still failed | Tokenizer is per-Connection in FTS5; needed to register but doesn't address the perf issue |
| §K.2 | Cross-schema UNION ALL with bm25/snippet was the bug | True for correctness; dropping aux funcs fixed PREPARE-time crashes but lost relevance ranking | FTS5 aux funcs CAN'T be schema-qualified in UNION ALL |
| §K.3 | Per-cUniverse standalone Connection enables bm25 | Worked for correctness but standalone Connections were 15-25× slower than active | Cold FTS5 segment pages on a fresh Connection |
| Option E | PRAGMA mmap_size + cache_size on federated_conn | 18s — REGRESSED. mmap on ATTACH bypasses the OS page cache that libraryStats had been warming | Counter-intuitive but empirically clear |
| Option F | Pre-warm OS page cache via MATCH on throwaway Connection | Returned 0 matches (stopword filter stripped 'a OR e OR i' tokens), 16s | Verify warm-up queries actually iterate the FTS5 cursor |
| Option G | FTS5 segment merge ('optimize') | Ran successfully — but timing didn't change. Improved RESULT QUALITY though. | Fragmentation wasn't the dominant cost; the per-row tokenizer cost was |

## Verification

Eisa's Boss-test for the final shipped state (Options C + G + H combined):

| Stage | Pre-fix | Post-fix |
|---|---|---|
| Federation status bar | 8751 notes + ⚠ 1 | Unchanged (8751 + ⚠ 1) |
| First federated search (paste `الرباط`) | 16-25 seconds | **Almost instantly** |
| Second federated search (`الربا`) | 25 seconds | **Under a second** |
| Arabic slow-typing (`الرباط` at 300-400ms/char) | Truncated to `الرب` or `الربا`, results 30+ seconds late | **Full word lands; results sub-second** |

## Architectural state post-resolution

- `SearchState.federated_conn` is the SOLE Connection for every federated query path (libraryStats / lens / search).
- `SearchState.federated_search_conns` (the §K.3 standalone pool) is gone.
- `federation_prewarm` background thread runs FTS5 `optimize` per cUniverse after each attach.
- `lexical_search_in_schema` takes a `skip_fts5_snippet` flag; federated mode passes `true`, active mode passes `false`.
- `synth_snippet_for_body` is the Rust-side snippet generator.
- 4 new unit tests in `mig056_federated_search::option_c_*` lock in the single-schema-attached-bm25 contract.
- 840/840 lib tests pass.

## Future considerations

- The `[federation-prewarm]` background thread logs to `diagnostics.log`. The MAX(segid) diagnostic query is broken (FTS5 shadow tables don't have a `segid` column under that name); the optimize itself works regardless. Could be cleaned up in a future PCS pass.
- Federated `<mark>` placement may differ from active-mode placement for stemmed/inflected query terms. If users notice + report, the fix is to make `synth_snippet_for_body` also try stemmed variants. Not a current concern.
- The Option G `INSERT INTO notes_fts(notes_fts) VALUES('optimize')` runs on every boot per cUniverse. The first-boot-per-cUniverse cost is ~30-60s; subsequent boots are 0ms (idempotent). If users have many (10+) cUniverses, first-launch federation visibility might lag by a few minutes total. Could add a `schema_versions` stamp to skip optimize when already-recently-optimized, but the idempotent 0ms cost makes this low-priority.

## Sources

The investigation used these primary sources:

- [SQLite FTS5 docs §11 (optimize)](https://sqlite.org/fts5.html)
- [SQLite Forum: ANALYZE fix 170s → 0.259s](https://sqlite.org/forum/info/509bdbe534f58f20)
- [SQLite Forum: Bad query plans from FTS5](https://sqlite.org/forum/info/e0e30e9eb1998e3c9305aea26957bec804615283969d11c1f9326a6b787526eb)
- [SQLite mmap.html](https://sqlite.org/mmap.html)
- [SQLite shared-cache (obsolete)](https://sqlite.org/sharedcache.html)
- [Lucene IndexReaderWarmer](https://lucene.apache.org/core/7_3_1/core/org/apache/lucene/index/IndexWriter.IndexReaderWarmer.html)
- [Solr Caches and Query Warming](https://solr.apache.org/guide/solr/latest/configuration-guide/caches-warming.html)
- [Svelte 5 `bind_value` source — focused-input guard](https://github.com/sveltejs/svelte/blob/main/packages/svelte/src/internal/client/dom/elements/bindings/input.js)
- [Win32 Arabic 101 keyboard (no IME)](http://kbdlayout.info/KBDA1/)

Plus four parallel research agents on 2026-05-27 covering: SQLite FTS5 cross-Connection perf, federated-search per-shard patterns from production, SQLite multi-Connection same-file behavior, and Svelte/IME/Arabic input.
