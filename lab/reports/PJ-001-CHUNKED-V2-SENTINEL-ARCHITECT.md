# PJ-001 Architect — Chunked v2 sentinel migration with progress UI

**Status**: Draft for Eisa's approval · 2026-05-06
**Predecessor**: MIG-013 §1E audit P1-M1 (`lab/reports/MIG-013-CTSE-AUDIT.md` §3) + memory `project_mig013_v2_migration_blocking_boot.md`
**Migration ID**: this is **MIG-015** (the next sequential migration after MIG-014).

---

## §1 — Function in hand

The MIG-013 §1D Option B work installed a v2 schema migration on `term_vocab` that converts every bigram row's `bridge_concept_id` from NULL to `'-'` (a sentinel). The conversion is one statement:

```sql
UPDATE term_vocab
   SET bridge_concept_id = '-'
 WHERE bridge_concept_id IS NULL
   AND term LIKE '%' || CHAR(31) || '%'
```

On a fresh DB this matches zero rows. On Boss-equivalent libraries (~5.7M term_vocab rows) the UPDATE has to scan every row, evaluate the leading-wildcard `LIKE`, and write to ~5.68M of them inside one implicit transaction. Wall-clock: 30–90 seconds. The boot path is fully synchronous — the user sees a frozen splash for the duration with zero feedback.

Boss has already migrated past it; new users with pre-MIG-013 backups remain at risk. PJ-001 closes that gap.

---

## §2 — Predecessor lookup (per Law 3.2)

| Predecessor | Replacement |
| --- | --- |
| `sentinel_bigram_rows(conn)` at `src-tauri/src/search.rs:526` — single bulk UPDATE. | **Same location, different algorithm.** New `sentinel_bigram_rows_chunked(conn, on_progress, db_path)` runs the UPDATE in N-row chunks with a progress callback. |
| Caller: `init_db` at `src-tauri/src/search.rs:1325` — synchronous call inside the v0→v2 schema migration block. | **Different shape.** `init_db` no longer runs the UPDATE inline. It only **detects** that v2 is pending and emits a "v2 migration deferred" diag log. The actual UPDATE runs on a background task spawned from the Tauri app's main entry, after first paint, with progress events emitted to the frontend. |
| `schema_versions.term_vocab_bridge` stamp at line 1336. | **Same logic, different timing.** Stamp moves out of `init_db` and into the deferred-task completion handler. |

**What's cut:** the synchronous `sentinel_bigram_rows` call from the boot critical path.
**What's kept:** the schema-version gating logic. The v0→v1 column-add step (`ensure_term_vocab_bridge_column`) stays in `init_db` — it's cheap (a `PRAGMA` + at most one `ALTER TABLE`).
**What's new:** a `migration:term_vocab_v2` Tauri event channel and a `<MigrationProgressStrip>` UI component in the status bar.

Verified against `src-tauri/src/search.rs:472-535, 1290-1346` and `src/routes/+layout.svelte:6075-6105`. Read against the orientation v1.44, not memory.

---

## §3 — Invariants that must not break

1. **One-shot per DB.** After successful migration, `schema_versions.term_vocab_bridge = 2` and the UPDATE is skipped on subsequent boots.
2. **Crash-recoverable.** If the user kills the app or the OS crashes mid-migration, the next boot picks up where it left off. Already-sentinelled rows stay sentinelled (the WHERE clause excludes `bridge_concept_id IS NOT NULL`); remaining NULLs are processed in the next attempt.
3. **Net result equivalent to the single UPDATE.** Every `term_vocab` row where `term LIKE '%' || CHAR(31) || '%' AND bridge_concept_id IS NULL` ends with `bridge_concept_id = '-'`.
4. **Boot is non-blocking.** `appReady` flips to `true` and first paint completes BEFORE the UPDATE starts. Status-bar strip is the only signal the user gets.
5. **No new boot IPCs.** The migration uses Tauri's `emit` (Rust → frontend push); the frontend `listen`s. No additional `invoke()` calls on the boot path. (Per CLAUDE.md "ZERO extra boot IPCs" rule.)
6. **M11 zero-diff.** `git diff src-tauri/src/lexicon/` returns empty.
7. **Reads of `term_vocab.bridge_concept_id` during the migration window are tolerant.** The column is dead schema (no code reads it for behaviour — it's forward-compat only). Mixed-state reads are acceptable. **Verified**: a grep for `bridge_concept_id` returns only the migration code itself.
8. **No interference with the v0→v1 column-add step.** That step is idempotent and stays in the boot path.
9. **Status-bar strip respects existing space.** It piggybacks on `.status-bar` (line 6079 of `+layout.svelte`); no new container, no layout shift.

---

## §4 — Design options

### Option A — Same-thread chunked with emit (rejected)

Replace the bulk UPDATE with a loop in `init_db` itself: `UPDATE … LIMIT 100000` repeated until 0 rows affected. Emit progress between chunks.

- **Pros**: preserves the synchronous `init_db` contract; simplest diff; no Tauri-task plumbing.
- **Cons**: still blocks `init_db` for the same total wall-clock; just visible progress. The user still waits 30–90 seconds before first paint, just with a moving indicator. Misses the "boot proceeds to first paint within 5 seconds" acceptance clause.

### Option B — Deferred background task, no UI (rejected)

`init_db` detects v2 pending, returns immediately. A `tauri::async_runtime::spawn` task does the UPDATE post-boot. No progress UI — just a `diag_log` start + `diag_log` done.

- **Pros**: boot is instant.
- **Cons**: misses the explicit acceptance clause "status-bar strip shows `Migrating term index — N / M`". User has no awareness the migration is happening; if they trigger a heavy term-vocab read during the window, they get confusing partial results.

### Option C — Deferred background task with status-bar strip (recommended)

`init_db` detects v2 pending, returns immediately, queues a one-shot async task. The task:

1. Runs `count_pending_v2_sentinel_rows(conn)` once → emits initial `migration:term_vocab_v2 { phase: 'start', total: N }`.
2. Loops `UPDATE term_vocab SET bridge_concept_id = '-' WHERE bridge_concept_id IS NULL AND term LIKE … LIMIT 100000`. After each chunk, emits `migration:term_vocab_v2 { phase: 'progress', completed: M, total: N }`.
3. On the chunk that returns 0 rows affected, stamps `schema_versions.term_vocab_bridge = 2` and emits `migration:term_vocab_v2 { phase: 'done', total: N }`.
4. Frontend listens, renders the status-bar strip on `start`, updates on `progress`, hides 4 seconds after `done`.

- **Pros**: meets every acceptance clause; user sees clean progress + closure; chunk size is tunable.
- **Cons**: most surface area — but the surface is bounded (one Rust function, one Svelte component, two i18n keys, one Tauri event channel).

---

## §5 — Recommendation

**Option C.** The Mini-MIG framing fits exactly: one Rust function (chunked + emitting), one frontend component (status-bar strip), wired via one Tauri event channel.

Chunk size: **100,000 rows**. Empirical SQLite UPDATE throughput on commodity SSD is ~250k–500k rows/sec for narrow updates, so each chunk completes in 200–400 ms — well under the 16-ms-frame-budget threshold for jank, and gives 50–100 progress updates over the full 5.7M-row migration (one update every ~600 ms — perceptible motion without flooding the event channel).

Total wall-clock under Option C is the same 30-90 seconds as the single UPDATE, but the user is interacting with a fully-painted UI throughout. The migration drives a status-bar strip that disappears on completion.

---

## §6 — Cross-check (per Working Agreement #5)

- **SQLite chunked migrations**: textbook. `UPDATE … LIMIT N` with `WHERE bridge_concept_id IS NULL` natural-pagination keeps each chunk small. PostgreSQL's `pg_repack`, MySQL's `pt-online-schema-change`, and SQLite community guides all use this shape.
- **Tauri progress events for long boot tasks**: documented pattern. The Constellation codebase already uses Tauri `emit` (`boot:hydrated`, search-reindex events, link-decay events). No new infrastructure needed.
- **Status-bar progress strips**: already in the codebase (see `update-progress-bar` and `semantic-progress-bar` in `SettingsModal.svelte`). The strip element is a thin progress bar plus a label — small SVG/CSS.
- **Crash-recoverable bulk migrations**: SQLite's WAL + the `WHERE bridge_concept_id IS NULL` clause make this trivial. No journal table, no checkpointing — the WHERE clause IS the resume marker.

No reinvention.

---

## §7 — Plan summary (full plan in §2 doc)

| Phase | Scope                                                                | Visible? | Boss test? |
| ----- | -------------------------------------------------------------------- | -------- | ---------- |
| §1A   | Rust — `count_pending_v2_sentinel_rows` helper + `sentinel_bigram_rows_chunked(conn, on_progress)`. Internal refactor. | No       | No         |
| §1B   | Rust — `init_db` defers v2 step. Spawn-task wiring in `lib.rs::run`. Tauri event channel. | No       | No         |
| §1C   | Frontend — `MigrationProgressStrip.svelte`. Listener wired into `+layout.svelte` status bar. i18n strings (en + ar). | **Yes**  | **Yes**    |
| §1D   | Three-agent audit (invariants / drift / migration-path).             | No       | No         |

Boss test for §1C: install on a fresh dev DB (or simulate a v2-pending state by manually setting `schema_versions.term_vocab_bridge = 1` then booting). Verify boot to first paint < 5 sec, status-bar strip appears with `Migrating term index — N / M`, progresses smoothly, disappears 4 seconds after `done`.

---

## §8 — Open questions for Eisa

1. **Chunk size — 100,000 rows**, or do you want a different value? (Smaller = more progress updates + more transaction overhead; larger = fewer updates + tighter DB lock windows.)
2. **Status-bar strip placement** — left side or right side of the status bar? Status bar currently has `.sb-left` (lib + note name) and `.sb-right` (counts + universe). I'd put it left, replacing the lib+note momentarily during migration. Or insert as a third center group.
3. **i18n strings**: en + ar now, 13 others queued via PJ-014. Acceptable?
4. **MIG numbering**: this is MIG-015. Confirm.

---

**Awaiting Eisa's "Architect approved" before writing the Plan.**
