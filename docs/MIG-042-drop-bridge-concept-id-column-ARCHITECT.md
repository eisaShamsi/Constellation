# MIG-042 — Drop the dead `term_vocab.bridge_concept_id` column

**Status:** Architect + Plan (Phase 1–2). Awaiting Eisa approval before Build.
**Date:** 2026-05-22
**Lineage:** The deferred "optional cleanup" from MIG-041 §2.3 / Phase D. Follows
MIG-013 (CTSE, which introduced the column), MIG-041 (bigram shrink, which made
it fully dead).

---

## 1. Goal

Remove the `bridge_concept_id` column from the `term_vocab` shadow table — the
last surviving piece of the abandoned "document-side concept tagging" design —
and the machinery that maintains it (the `ensure_term_vocab_bridge_column` add-
path + the `idx_term_vocab_bridge_concept_id` index + the always-NULL write in
`apply_delta`).

**Payoff (honest):**
- **NOT a disk or speed win.** The column is 100% NULL on every row, so it costs
  ~a few hundred KB across 538,648 rows plus one small all-NULL index. Dropping
  it reclaims a negligible amount. *Do not market this as a size win.*
- **The win is pure hygiene / simplification:** removes dead schema, a ~25-line
  function (`ensure_term_vocab_bridge_column`), an index, and a misleading
  column from the table that every future session otherwise has to re-establish
  is dead. It also lets the schema-version doc stop carrying "forward-compat for
  a feature that was abandoned" caveats.
- This is genuinely optional. It is worth doing for cleanliness; it has zero
  user-visible effect.

---

## 2. Territory map (verified against current code, 2026-05-22)

### 2.1 Every place the column is touched (complete consumer list)

| # | Location | Role | MIG-042 action |
|---|---|---|---|
| 1 | `search.rs:1755` base `CREATE TABLE term_vocab` | **Does NOT define the column** — proves a fresh DB has no column until step 2 runs | unchanged (already correct) |
| 2 | `search.rs:511` `ensure_term_vocab_bridge_column()` + call at `:1779` | ADDs the column via `ALTER … ADD COLUMN` + creates the index, **every boot** (idempotent) | **REMOVE** function + call. This is what would otherwise re-add the column after a drop. |
| 3 | `search.rs:531` index `idx_term_vocab_bridge_concept_id` | index on the column | **DROP** in the migration (SQLite refuses `DROP COLUMN` on an indexed column) |
| 4 | `hooks.rs:167` `apply_delta` INSERT `(…, bridge_concept_id) VALUES (…, NULL)` | the only write — always NULL | **MODIFY** → drop the column from the INSERT |
| 5 | `hooks.rs:245` test helper `make_term_vocab` | creates the column in unit-test DBs | **MODIFY** → drop the column (so tests exercise the post-drop schema) |
| 6 | `search.rs:103–135` `TERM_VOCAB_BRIDGE_SCHEMA_VERSION` doc/const | schema lineage doc | **UPDATE** doc; the constant stays 3 (purge). The drop gets its own gate (§4). |
| 7 | Comments: `hooks.rs:16-35`, `search.rs:497-510`, `ctse/search.rs:32-50`, `ctse/mod.rs:24-33` | explain "dead schema, kept for forward-compat" | **UPDATE** → "removed in MIG-042" |
| 8 | **Frontend (`src/`)** | — | **NONE** — grep-confirmed zero references. Backend-only change. |

**No live SQL reads the column anywhere** (no SELECT / WHERE / JOIN on
`bridge_concept_id`) — verified by grep across `src-tauri/src` and `src`. The
only references are the write at #4, the add-path at #2, and comments.

### 2.2 Why removal is safe

- The column is **verified-dead**: MIG-041 already established (and the
  `ctse/search.rs:50` comment states) that nothing reads it. Cross-language
  search runs entirely at query time (`ctse::search`), never touching this
  column.
- The base table never defined it → removing the add-path means **fresh DBs
  simply never have it** (no migration needed for new users).
- Existing DBs (Eisa's: column present, all-NULL) need a one-time DROP.

---

## 3. Invariants that MUST NOT break

1. **`term_vocab` (term, doc_count, total_count) rows unchanged** — 538,648
   single-stem rows keep their counts; only the dead column leaves.
2. **CTSE query-time concept expansion unaffected** (`WHERE term IN (…)` reads
   `term`, never `bridge_concept_id`).
3. **`notes_fts` / `notes_vocab` untouched** → Index panel, phrase, Arabic
   matching unaffected (they were never related to this column).
4. **The crown-jewel four** (cross-language `via`, `≈ similar`, Arabic, phrase)
   all keep working — none read this column.
5. **Boot never blocked** — the DROP runs in the existing background worker
   (post-paint), never on the boot critical path. Hard boot-perf constraint held.
6. **Crash/interrupt-safe** — `DROP COLUMN` is an atomic table rewrite (rolls
   back fully if interrupted); the gate is unstamped until success, so the next
   boot simply retries. No partial state possible.
7. **Concurrency-safe** — reuses the MIG-041 fix's `MIGRATION_ACTIVE` daemon
   pause + transient-lock retry + self-checkpoint. (The whole point of last
   session's lesson.)
8. **The `term_vocab` write hook keeps working** — `apply_delta` still
   UPSERTs (term, doc_count, total_count); only the NULL column reference goes.

---

## 4. Design decision — WHERE to run the DROP

`DROP COLUMN` takes an **exclusive lock** and **rewrites the whole table**.
That is precisely the operation class that collided with the WAL daemon in
MIG-041 and stalled for 7 hours. So placement is the key decision.

**Option A — synchronous DROP in `init_db`.** Runs before the daemon + worker
exist (concurrency-free), but **blocks boot** by the table-rewrite time. Worse:
on a pre-MIG-041 backup (column present + 5.73M un-purged rows) the synchronous
rewrite would be huge (10–30 s frozen splash). Would need an awkward "only if
already purged, else defer to next boot" gate. Rejected.

**Option B — DROP as "Part 3" of the existing `run_bigram_purge` worker
(CHOSEN).** The worker already (a) pauses the WAL daemon (`MIGRATION_ACTIVE`),
(b) retries transient `SQLITE_BUSY`/locked, (c) self-checkpoints, (d) logs to
`diagnostics.log`. Adding the DROP there reuses **every** concurrency safeguard
MIG-041 earned. It runs post-paint (no boot block), and it naturally orders
**after** the purge → the table is already small (538k rows) when the column is
dropped, so the rewrite is sub-second. Handles every DB state in one pass:
- Fresh DB → no column → no-op + stamp.
- Eisa's DB (purged) → only the drop runs.
- Pre-MIG-041 backup → purge → VACUUM → drop, in order.

**Gate:** a dedicated `schema_versions` module **`term_vocab_dropcol`** (= 1 when
dropped), mirroring how `term_vocab_vacuum` is separate from `term_vocab_bridge`.
This avoids overloading the bridge constant's stamp value (Part 1 still stamps
`term_vocab_bridge` = 3; Part 3 stamps `term_vocab_dropcol` = 1).

### 4.1 The DROP sequence (Part 3 body)

```text
if term_vocab_dropcol < 1:
    if column "bridge_concept_id" absent (PRAGMA table_info):   # fresh / already-clean DB
        stamp term_vocab_dropcol = 1; done (no-op)
    else:
        retry-on-busy:
            DROP INDEX IF EXISTS idx_term_vocab_bridge_concept_id;   # must precede DROP COLUMN
            ALTER TABLE term_vocab DROP COLUMN bridge_concept_id;    # atomic rewrite
        PRAGMA wal_checkpoint(TRUNCATE);   # bound the WAL before the daemon resumes (best-effort)
        stamp term_vocab_dropcol = 1
        log to diagnostics.log
```

**Crash matrix:** killed between DROP INDEX and DROP COLUMN → index gone, column
remains, gate unstamped → next boot: `DROP INDEX IF EXISTS` no-ops, `DROP COLUMN`
runs, stamp. Killed mid-rewrite → SQLite rolls back → column remains, gate
unstamped → next boot retries. No corruption either way.

### 4.2 Wake the worker for the drop-only case

`maybe_schedule_bigram_purge` pre-check today: `bridge < 3 || vacuum < 1`. On
Eisa's DB both are satisfied → worker never spawns. Add `|| dropcol < 1` so the
worker wakes to do Part 3 alone. To keep **fresh/clean DBs from spawning a
worker for nothing**, `init_db` pre-stamps `term_vocab_dropcol = 1` when it
observes the column is already absent (`PRAGMA table_info` after the base
`CREATE TABLE`). Net: the worker spawns exactly once on DBs that still carry the
column, never again afterward.

---

## 5. Plan (each phase = one commit + verification clause)

> **Phase A — Stop maintaining the column (write path + add path + tests).**
> - `hooks.rs`: `apply_delta` INSERT → `(term, doc_count, total_count) VALUES (?1,?2,?3)` (drop the NULL column); update the test helper `make_term_vocab` to omit the column; update the module doc comment (#7).
> - `search.rs`: delete `ensure_term_vocab_bridge_column()` + its call at `:1779`; update the `TERM_VOCAB_BRIDGE_SCHEMA_VERSION` doc block to record that v-lineage ends at 3 + the column is dropped by `term_vocab_dropcol`.
> - Update the `ctse/search.rs` + `ctse/mod.rs` comments (#7).
> *Verify:* `cargo test -p constellation` (hooks tests green, incl. the existing `bigrams_are_not_written` + counts tests — now against a column-free schema); `cargo clippy` clean. **No DB has been touched yet** — this phase only stops *future* maintenance + fresh-DB creation of the column.

> **Phase B — One-time DROP migration (Part 3 of the worker + gate + scheduler).**
> - Add `TERM_VOCAB_DROPCOL_SCHEMA_VERSION = 1` + `term_vocab_dropcol` handling.
> - Add Part 3 to `run_bigram_purge` per §4.1 (column-presence guard, DROP INDEX IF EXISTS, DROP COLUMN, retry-on-busy via the existing `is_transient_lock`, final checkpoint, stamp, diag-log).
> - Extend `maybe_schedule_bigram_purge` pre-check with `|| dropcol < 1`; add the `init_db` pre-stamp-when-absent optimization (§4.2).
> *Verify (on a COPY of the real 1.75 GB DB first):* worker runs Part 3 → `PRAGMA table_info(term_vocab)` shows no `bridge_concept_id`; index gone; row count still 538,648; `(term, doc_count, total_count)` intact; `PRAGMA integrity_check` = ok; the crown-jewel four all work; Index panel populates. Then re-open → worker does NOT re-spawn (gate stamped). **Then Eisa tests on the live DB.**

> **Phase C — /simplify pass.**
> Run `/simplify` on the full diff: confirm no dangling `bridge_concept_id` references remain (grep), no dead helper left behind, comments accurate.
> *Verify:* `cargo build` + `svelte-check` (frontend untouched → expect the 3 pre-existing, 0 new); grep `bridge_concept_id` returns only historical doc/MoCh/session-log mentions, zero in `src-tauri/src` + `src`.

> **Phase D — Audit (Migration Rule Phase 4).**
> Three agents in parallel: (1) invariants §3 hold; (2) drift — any new guard/path the plan didn't map (esp. anything that reads `term_vocab.*` by `SELECT *` and would now see a narrower row); (3) migration path — fresh DB, Eisa's purged DB, pre-MIG-041 backup, mid-drop interrupt/resume, rollback.

> **Phase E — SO + docs.**
> Session log; Orientation **v2.25** (SO #6 — a shipped MIG); the LL-025 "test under live concurrency" entry can ride along here (it was the deferred formalization, and MIG-042 is the natural place to cite it as the rule this migration was built to honor). Help files: **no user-facing change** → no help/User-Manual edit needed (note this explicitly in the session log so the docs-sync rule is satisfied by exception, not omission).

---

## 6. Migration-path matrix

| Scenario | Behavior |
|---|---|
| **Fresh DB** | Base table has no column; add-path removed; `init_db` pre-stamps `dropcol=1` → worker never spawns for the drop. |
| **Eisa's DB (purged, bridge=3/vacuum=1)** | Worker wakes for Part 3 only; column present → DROP INDEX + DROP COLUMN (sub-second, 538k rows) → stamp. |
| **Pre-MIG-041 backup (bridge<3, column present)** | Worker: Part 1 purge → Part 2 VACUUM → Part 3 drop, in order. Table small by the time Part 3 runs. |
| **Mid-drop interrupt** | Atomic rollback or partial (index-only) → gate unstamped → next boot retries idempotently (§4.1 crash matrix). |
| **Code rollback after drop** | DB has no column; the rolled-back code's `ensure_term_vocab_bridge_column` would simply re-ADD it (all-NULL) on next boot, and `apply_delta` would resume writing NULL. Fully safe — the column is inert either way. |
| **SQLite version** | rusqlite 0.31 `bundled` ⇒ SQLite ≥ 3.45 ⇒ `DROP COLUMN` (needs 3.35) supported. (Confirm `SELECT sqlite_version()` in Phase B if desired.) |

---

## 7. Risk summary

**Very low.** The column is verified-dead (no reader anywhere, frontend included)
and the base table never defined it. (Correction: it is **not** 100% NULL — on
the live DB ~24,827 of 538,648 rows / ~4.6% carry a stale value from the original
§1C eager-tagging fast-path. Still dead — nothing reads it — so dropping it is
safe; the earlier "all-NULL" claim was wrong.) `DROP COLUMN` is atomic +
crash-safe; the drop runs inside the already-hardened MIG-041 worker (daemon
pause + retry + self-checkpoint), so the one concurrency hazard is already
mitigated. Rollback is safe in both directions. The only "cost" is a one-time
sub-second background table rewrite (measured **0.37 s** on the real 538k-row
table) on DBs that still carry the column. **No user-visible effect; the value
is schema hygiene.**

---

## 8. Build discovery (2026-05-22) — the orphaned `_ad` trigger (→ BUG-020)

The plan said "verify on a COPY of the real DB first." That copy-test **caught a
blocker the unit tests never could** — vindicating the MIG-041 lesson again.

**What broke:** `ALTER TABLE term_vocab DROP COLUMN` failed on the real-DB copy
with `error in trigger sight_v5_layout_invalidate_ad: no such table:
main.sight_v5_layout`. `DROP COLUMN` re-validates the **entire** schema, and an
orphaned trigger referenced a table that no longer exists.

**Root cause (a pre-existing latent bug, not MIG-042's):** MIG-028 (Sight v5
retirement, 2026-05-18) dropped the `sight_v5_layout` table + its AFTER-UPDATE
trigger (`_au`) but **missed the AFTER-DELETE trigger `_ad`** (on `note_meta`).
With the table gone and `_ad` surviving, **every `DELETE FROM note_meta` failed**
("no such table"). In `reindex_delete_note` the error is swallowed (`let _ =`),
so **deleted notes silently ghosted in the index** since ~2026-05-18; any
`?`-propagating delete path errored outright. Confirmed: `sight_v5_layout_invalidate_ad`
is the **sole** `sight_v5` leftover on the live DB.

**Fix (folded into MIG-042, Eisa-approved):** add `DROP TRIGGER IF EXISTS
sight_v5_layout_invalidate_ad` to the existing MIG-028 cleanup batch in `init_db`
(synchronous, idempotent, instant — fixes note deletion on boot AND clears the
schema before the worker's Part 3 column drop runs). Tracked as **BUG-020**.

**Definitive copy-test (exact shipped DDL on a fresh copy of the live 1.63 GB
DB):** Step 1 (init_db cleanup) → `_ad` gone, `DELETE FROM note_meta` works;
Step 2 (worker Part 3) → `DROP COLUMN` 0.37 s, column + bridge index gone,
`total_count` index survives, **538,648 → 538,648 rows (zero loss)**,
`integrity_check`/`quick_check` = ok. **RESULT: PASS.**

**Lesson reinforced:** test DB migrations on a copy of the *real* DB, not a clean
synthetic one — the orphaned trigger (and the latent note-deletion bug it caused)
existed only in the field, never in an in-memory test fixture.
