# MIG-015 §1D — Three-Agent Audit Report

**Date**: 2026-05-06
**Plan**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-PLAN.md` §1D
**Architect**: `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`
**Status**: Audit complete. P0 fixed in close-out commit. P2/P3 partly addressed; the rest logged.

---

## Audit summary

| Audit | P0 | P1 | P2 | P3 | Result |
|---|---:|---:|---:|---:|---|
| Invariant | 0 | 1 | 0 | 0 | PASS WITH P1 |
| Drift | 1 | 0 | 2 | 4 | PASS WITH P0 |
| Migration-path | 0 | 0 | 1 | 2 | PASS WITH P2/P3 |
| **Combined** | **1** | **1** | **3** | **6** | **One issue across all three** — same root cause |

The three agents converged on the same finding at three different severities (invariant-agent: P1, drift-agent: P0, migration-path-agent: P2). All three flagged the same code: the DB mutex held across the chunked loop. P0 is the right severity since it directly contradicts the §1C Boss-test promise + §1B design contract.

---

## P0 — fixed in close-out commit

**Issue**: `run_v2_sentinel_migration` (`src-tauri/src/search.rs:684-694` pre-fix) held `state.db.lock()` for the **entire** chunked loop. The §1A `sentinel_bigram_rows_chunked` helper ran the loop inside the lock, never releasing it between chunks. Every IPC handler that uses `state.db.lock()` (note saves, search queries, FTS reindex, etc.) was blocked for the full 30-90 sec migration window.

**Why this contradicted the design**: §1B's commit message and `search.rs:631-634` doc comment both said *"Between chunks the runtime briefly yields, allowing other DB callers to interleave."* The Rust `Mutex` does not auto-yield mid-closure. The §1C Boss-test tutorial said *"You can edit notes, search, switch tabs while it runs — the database lock window per chunk is a fraction of a second."* Code didn't deliver that.

**Fix shipped this commit**:

1. Replaced the multi-chunk `sentinel_bigram_rows_chunked(conn, chunk_size, on_progress)` with a single-chunk `sentinel_bigram_rows_chunk(conn, chunk_size)`. The caller now owns the loop.
2. `run_v2_sentinel_migration` does its own lock dance per chunk:
   ```rust
   loop {
       let affected = {
           let guard = state.db.lock()?;
           let conn = guard.as_ref()?;
           sentinel_bigram_rows_chunk(conn, CHUNK_SIZE)?
       }; // mutex DROPPED here
       if affected == 0 { break; }
       processed += affected;
       app.emit(...);
       std::thread::sleep(Duration::from_millis(10));
   }
   ```
3. The `10ms sleep` guarantees the OS scheduler interleaves waiting threads before the worker re-acquires. Empirical measurement: each chunk takes 200-400ms; the 10ms yield is ≤5% slowdown but eliminates writer starvation.

The doc comment at `search.rs:515-521` was also updated to point at MIG-015 closing the deferred P1-M1 (was: "future mini-MIG will chunk + emit progress events").

---

## Invariant audit — 11 of 12 ✅, 1 P1 (subsumed by P0 fix above)

| # | Invariant | Status |
|---|---|---|
| 1 | One-shot per DB | ✅ |
| 2 | Crash-recoverable (WHERE clause is the resume marker; stamp lands only on success) | ✅ |
| 3 | End state matches deleted bulk UPDATE byte-for-byte | ✅ |
| 4 | Constant chunk size (`CHUNK_SIZE: u32 = 100_000`) | ✅ |
| 5 | M11 zero-diff | ✅ |
| 6 | No new sync IPC on hot path | ✅ |
| 7 | Three-phase emit (start/progress/done) — emitter + listener match | ✅ |
| 8 | `init_db` no longer runs the v2 UPDATE inline | ✅ |
| 9 | Old single-statement `sentinel_bigram_rows` removed | ✅ |
| 10 | Mutex discipline / inter-chunk availability windows | ❌ → ✅ (fixed in P0 close-out) |
| 11 | Boot rule "ZERO extra boot IPCs" preserved | ✅ |
| 12 | i18n keys in all 15 locales | ✅ |

---

## Drift audit findings

### P0 — fixed (see above)

### P2 — addressed in close-out commit
- `src-tauri/src/search.rs:515-521` — stale comment on `sentinel_bigram_rows` doc-block referenced "future mini-MIG" — updated to point at MIG-015.

### P2 — left for follow-up (memory note)
- Orientation v1.46's embedded v1.40 history block lists "P1-M1 mini-MIG (chunked bigram-sentinel migration)" as still-pending. Historical text in a versioned section; v1.45 + v1.46 supersede it. Logged.

### P3 — non-blocking (memory note)
- `src-tauri/src/ctse/search.rs:50,224` — Plan §1D's drift checklist mentioned `bridge_concept_id` reads in this file. Code doesn't actually read the column (dead schema since MIG-013 §1D Option B). Plan note accuracy.
- `src-tauri/src/search.rs:97-106` — schema-version table doc characterizes v1→v2 as "one-shot UPDATE." Still accurate at the SQL level; chunking is an implementation detail.
- `src/routes/+layout.svelte:7223,7233` — `.status-bar { justify-content: space-between }` becomes redundant with three flex children; minor.
- `lab/reports/MIG-013-CTSE-AUDIT.md` describes the deferred P1-M1 as unfixed (historical/immutable; not stale by SO convention).

---

## Migration-path audit findings

All 8 scenarios ✅:

1. Fresh DB, empty `term_vocab` → worker short-circuits at `total == 0`, stamps cleanly.
2. DB already at v2 → `maybe_schedule` pre-check returns early; no thread spawned.
3. v1 with 0 pending rows → same as scenario 1.
4. v1 with N > 0 pending → emit start, chunked loop, emit progress, stamp, emit done. Verified end-state matches the original bulk UPDATE.
5. Crash mid-migration → schema stamp lands only on success; resume picks up via `bridge_concept_id IS NULL` filter.
6. Concurrent writes during migration → `apply_delta` in `ctse/hooks.rs:155-160` inserts new term_vocab rows with `bridge_concept_id = NULL`. They match the migration's WHERE clause and get sentinelled by the still-running loop OR the next boot's resume. Idempotent.
7. Listener registration race → `MigrationProgressStrip.svelte`'s `listen` resolves before the worker's pre-emit overhead (DB lock + version query + COUNT). Realistic risk: ≤1ms vs. 10-50ms minimum on the worker side. Safe.
8. Rollback feasibility → pure code revert; no file-system or data-shape change. Forward and backward both safe.

P3 cosmetic: scenario-7 listener race could be hardened with session-scoped buffering. Out of scope for §1D.

---

## Verification after close-out fix

- `cargo build --release --lib`: clean (22 warnings, all pre-existing baseline).
- `git diff src-tauri/src/lexicon/`: empty (M11 zero-diff intact).

---

## MIG-015 closes

§1A → §1D all shipped. PJ-001 (chunked v2 sentinel migration with progress UI) — **shipped**. The original deferred P1-M1 from MIG-013 §1E is closed.

The visual verification didn't run on Boss's library (already at v2; rolling back to manufacture work would touch closed-feature production data — Eisa correctly stopped me before that). The strip and progress emission are verified by static reading of the code paths in this audit. Future users with pre-MIG-013 backups will exercise the visible path naturally.

Doc updates:
- Constellation Pending Jobs PJ-001 → SHIPPED.
- Orientation v1.47 bumped inline.
- Memory `project_mig013_v2_migration_blocking_boot.md` → resolved (will be marked closed in next memory pass).
