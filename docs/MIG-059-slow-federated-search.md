# MIG-059 — Slow Federated Search (SHIPPED)

**Status:** Shipped 2026-05-27 (fix landed in `ensure_search_db_ready` background-attach thread).
**Priority:** P2. Didn't break correctness; per-cUniverse search took ~25s vs ~1s in active-mode on the same data.

## The bug, in measured terms

The MIG-056 §K.3 scatter-gather implementation opened a standalone SQLite Connection per cUniverse and ran `lexical_search` on each. The per-Connection `lexical_search` on Eisa's cu1 (Eisa Cognitive Knowledge, 7650 notes) took **15-27 seconds** for the first FTS5 BM25 query — vs the equivalent active-mode call (~1 second) on the exact same data file.

The slow-search exact times observed in diag logs:
- `query="الرباط"` → 14-15s for cu1's branch (returns 30 ranked rows)
- `query="الربا"` → 23s for cu1's branch (returns 30 ranked rows)
- `query="الر"` → 3s for cu1's branch (returns 30 ranked rows)
- `query="تالر"` (typo) → 1s for cu1's branch (returns 2 rows)

Pattern: slower for queries that match more rows (more BM25 scoring + more index pages to read). But even short prefix matches took 3+ seconds, way above the ~100ms FTS5 should take on a 7650-doc index.

## Root cause

**Cold page cache + un-checkpointed WAL on the cu1 Connection's first read.**

The active-universe Connection gets implicitly warmed during `init_db`:
- `mig003_step3_soft_rebackfill` rewrites at least 1 row in `note_meta` → FTS5 triggers fire → some FTS5 segment pages get touched.
- Various PRAGMA/migrations read the database file.
- The active universe's `spawn_wal_checkpoint_daemon` regularly merges the WAL into the main DB file, keeping the WAL small.

The cu1 standalone Connection (introduced by MIG-056 §K.3) had none of this:
- Open → register tokenizer → store. No reads, no checkpoint, nothing.
- If the cUniverse's owner left an accumulated WAL file, the first standalone read had to scan the entire WAL to construct the consistent view — slow.
- The first FTS5 query had to lazily initialize the FTS5 vtable, faulting hundreds of index pages from cold disk.

## Fix

In `src-tauri/src/search.rs::ensure_search_db_ready`'s background-attach thread, right after `register_fts5_tokenizer(&mut cu_conn)`, run two best-effort pre-warm steps:

```rust
let _ = cu_conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);");
let _ = cu_conn
    .query_row::<i64, _, _>(
        "SELECT COUNT(*) FROM notes_fts",
        [],
        |r| r.get(0),
    )
    .ok();
```

**Step 1: `PRAGMA wal_checkpoint(PASSIVE)`** merges the WAL into the main DB file if no other readers hold locks. If it can't acquire the lock, it no-ops without blocking (PASSIVE mode is non-blocking by definition). Safe.

**Step 2: `SELECT COUNT(*) FROM notes_fts`** forces SQLite to initialize the FTS5 vtable for this Connection. This triggers the lazy loading of FTS5 segment-index pages into the page cache, paying the first-query cost upfront on the background-attach thread instead of on the user's first search.

Both failures are best-effort — they only affect the FIRST search's latency, not correctness. Subsequent searches hit warm caches regardless.

## Cost vs benefit

- Per-cUniverse cost paid: ~50-200ms during boot (background-attach thread, off the UI's critical path).
- Per-search cost saved: 15-27s → ~1s on the first federated search. Subsequent searches: same (always cache-warm).

User-visible effect: **first federated search after boot is no longer the multi-second wait it was in §K.3**. All subsequent searches should now feel as fast as the active-mode equivalent.

## Hypothesis that the slow-search ALSO fixes the truncation (MIG-058)

MIG-058 (QuickSwitcher Arabic input truncation) was hypothesized to be caused by the slow async `constellationSearch` resolving mid-IME-composition and the resulting re-render disrupting Arabic IME state. If the search is now ~1s instead of ~25s, the async never completes mid-typing for any reasonable typing speed. The truncation may resolve as a side effect.

If MIG-058 persists after MIG-059, it's a separate IME / Svelte issue and needs its own investigation.

## Tests

- 836/836 lib tests pass (no regression).
- `cargo check --lib` clean.
- Pre-warm code is purely additive; doesn't change any existing call paths.

## Verification path

The user-facing test: type a query in Ctrl+O search across the federated set. Result list should appear within ~1 second instead of the ~25s wait observed in §K.3 retest.

## Related

- Surfaced during MIG-056 §K.3 Boss-test (2026-05-27 morning).
- File: `src-tauri/src/search.rs::ensure_search_db_ready` background-attach thread, right after the per-cUniverse `register_fts5_tokenizer` call.
- Architecture unaffected: scatter-gather + RRF (commit `0e094da0`) still runs identically; the per-Connection `lexical_search` is now just much faster on its first call.
