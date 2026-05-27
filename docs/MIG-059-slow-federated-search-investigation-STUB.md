# MIG-059 — Slow Federated Search Investigation (STUB)

**Status:** Open — pending investigation.
**Opened:** 2026-05-27 (post-MIG-056 §K.3).
**Priority:** P2. Doesn't break correctness, but ~25s per federated search is a real UX regression vs single-schema (~1s).

## The bug, in measured terms

The MIG-056 §K.3 scatter-gather implementation opens a standalone SQLite Connection per cUniverse and runs `lexical_search` on each. The per-Connection `lexical_search` on Eisa's cu1 (Eisa Cognitive Knowledge, 7650 notes) takes **20-27 seconds** for a single FTS5 BM25 query.

By comparison, when Eisa Cognitive Knowledge is the active universe (single-schema, same data file, same `lexical_search` function), the same query completes in ~1 second.

### Diagnostic evidence

From `lab/reports/SESSION-LOG-2026-05-27.md` and the in-context diag log:

```
[1779819360] federated_lexical_search: query="الربا"
[1779819360] probe main: ... MATCH hits=11
[1779819363] probe cu1: ... MATCH hits=298  <-- 3 seconds for the COUNT probe
```

The 27-second figure was measured for the per-Connection `lexical_search` call. The diag-log COUNT probe takes ~3 seconds on the same Connection. Both are dramatically slower than the equivalent active-mode call.

## Suspected root causes (in order of plausibility)

1. **FTS5 cold-start / vtable initialization.** The first FTS5 query on a Connection materializes the vtable's internal data structures (segment files, dictionary, etc.). For a 7650-doc FTS5 index this could be hundreds of MB of mmap-faulting + segment merging. The active-universe Connection went through this cost at boot via `init_db` warm-up; the §K.3 standalone Connection opens but doesn't pre-warm. First query pays the cold-start cost.
2. **Lock contention with the ATTACH-based `federated_conn`.** Both Connections read the same `search.db` file. Even with WAL mode allowing concurrent readers, there may be some SHARED-lock acquisition cost or page-cache contention.
3. **Per-Connection FTS5 state divergence.** The active-universe Connection runs through `init_db` which includes the `mig003_step3_soft_rebackfill` (the "repaired note_meta=1" line in boot logs). If this modifies note_meta rows, the FTS5 triggers update the index. The cUniverse's search.db hasn't gone through that on this boot. Reading it might involve some FTS5 self-recovery / integrity-check pass that's slow.
4. **Missing PRAGMA / cache_size tuning.** §K.3 sets `cache_size=-65536` (64 MB) to match active-mode, but maybe FTS5 needs more for prefix queries. The active-mode Connection might have a hotter page cache by the time the user searches.

## Why this is its own MIG

The §K.3 scatter-gather architecture is correct — just slow on first query. The fix is a perf optimization on the per-Connection setup, not a re-architecture. Could ship at any time without touching the federation contract.

## Proposed investigation steps

1. **Add per-branch timing instrumentation** (temporary, like the §K.3.A diagnostic). Log `lexical_search` wall-time per branch. Confirm cu1 is the slow one and quantify it precisely (mean / p95 / p99 over 10 queries).
2. **Pre-warm the FTS5 vtable** in the background-attach thread by running a no-op query (`SELECT COUNT(*) FROM notes_fts`) right after opening + tokenizer registration. If first-query cost drops to ~1s, root cause is cold-start.
3. **Compare with `OpenFlags::SQLITE_OPEN_READ_ONLY`** — open the standalone Connection read-only. SQLite can use optimizations that don't apply to read-write opens. If this fixes it, lock contention was the cause.
4. **Run an FTS5 integrity check** (`INSERT INTO notes_fts(notes_fts) VALUES('integrity-check')`) on the standalone Connection after open. If it reports corruption, that explains slowness.
5. **Profile a per-branch query** with `EXPLAIN QUERY PLAN` + `sqlite3_profile` to see exactly where time goes.

## Verification clauses (once the fix is identified)

- [ ] Per-branch `lexical_search` on cu1 completes in < 1s on Eisa's 7650-note data (post-warm-up).
- [ ] First search after boot completes in < 2s total (including warm-up if any).
- [ ] No regression in 833/833 lib tests.
- [ ] Boss-test for `الرباط` returns same results as today (just faster).

## Related

- Surfaced during MIG-056 §K.3 Boss-test (2026-05-27).
- File: `src-tauri/src/search.rs::federated_lexical_search_or_fallback` + the background-attach Connection setup in `ensure_search_db_ready`.
- Possibly tied to: cUniverse's FTS5 index "freshness" relative to active-mode init_db side effects.
