# MIG-041 — Remove `term_vocab` bigrams (DB shrink + complexity retirement)

**Status:** Architect + Plan (Phase 1–2). Awaiting Eisa approval before Build.
**Date:** 2026-05-21
**Lineage:** PJ #26 (parked). Follows MIG-013 (CTSE), MIG-015 (v2 bigram-sentinel boot-fix).

---

## 1. Goal

Remove the ~5.19M **bigram rows** from the `term_vocab` shadow table — they are
redundant dead weight that nothing reads — then reclaim the freed disk. Stop the
write path from re-creating them. As a side effect, retire the MIG-015 "v2
sentinel" migration that exists *only* to service those bigram rows (and which
still blocks boot on old pre-MIG-013 backups — the parked `project_mig013`
issue).

**Payoff (honest, today's numbers):**
- DB **2.35 GB → ~1.75 GB** (~0.6 GB / **26%**). *(Earlier "1.7 GB / 70%" was an overestimate — corrected.)* Helps the occasional **cold** first-boot; warm boots are already ~0.4 s.
- Every note-save stops maintaining 5.19M useless rows (slightly cheaper saves).
- Retires a whole migration's worth of code + a known boot-blocker on old backups.
- **The real win is hygiene/simplification**, not the gigabyte.

---

## 2. Territory map (verified against current code, 2026-05-21)

### 2.1 Two separate bigram stores — THE key distinction

Bigrams (two same-script stems joined by `BIGRAM_SEP = 0x1F`) exist in **two
unrelated places**. Conflating them is the trap (a stale comment at
`search.rs:524` says "the Index panel … consume[s]" bigrams — true, but from the
*other* store):

| Store | What it is | Who reads it | Touch in MIG-041? |
|---|---|---|---|
| **`notes_fts` + `notes_vocab`** | FTS5 full-text index + its `fts5vocab(notes_fts,'row')` dictionary view | **Index panel** (`read_index_entries` → `SELECT term,cnt FROM notes_vocab`), phrase search, Arabic matching (`read_term_mentions` MATCH) | **NO — untouched** |
| **`term_vocab`** | CTSE shadow/ledger table `(term, doc_count, total_count, bridge_concept_id)` | **only** `ctse/search.rs` query-time expansion — which **explicitly skips bigrams** (`search.rs:201`) | **YES — delete its bigram rows** |

The bigrams in `term_vocab` duplicate signal that the FTS5 index already holds.
`ctse/search.rs` filters them out on read (`token.contains(BIGRAM_SEP) → skip`,
then `SELECT term FROM term_vocab WHERE term IN (single-stems)`), so the 5.19M
rows are pure dead weight.

### 2.2 Write path (how bigrams enter `term_vocab`)

`ctse::hooks::on_note_indexed` (every save) →
`token_counts(body)` (`hooks.rs:92`) →
`fts5_tokenizer::tokenize_to_vec()` (`fts5_tokenizer.rs:434`, emits **stems + bigrams** via `emit_word_collect`) →
`compute_delta(old,new)` →
`apply_delta()` (`hooks.rs:140`, UPSERTs every term — incl. bigrams — into `term_vocab`).

`tokenize_to_vec` is the **`term_vocab`-only twin** of the FTS5 tokenizer. The
real FTS5 index uses a *different* code path (`emit_word` → FTS5 cursor,
`fts5_tokenizer.rs:504+`). So filtering bigrams out of the `term_vocab` write
path does **not** affect `notes_fts`.

### 2.3 Obsoleted machinery (retire after bigrams gone)

- `count_pending_v2_sentinel_rows` / `sentinel_bigram_rows_chunk` / the v2-sentinel scheduling (`search.rs:520–641`, `4517`) — a background chunked `UPDATE term_vocab SET bridge_concept_id='-' WHERE term LIKE '%CHAR(31)%'`. Exists only to sentinel bigram rows. Becomes a no-op once they're deleted. **This is the `project_mig013` boot-blocker.**
- `bridge_concept_id` column — dead schema (`search.rs:50`). Optional drop.

---

## 3. Invariants that MUST NOT break

1. **`notes_fts` / `notes_vocab` unchanged** → Index panel, phrase search, Arabic matching unaffected.
2. **`term_embeddings` (104,823 rows) unchanged** → `≈ similar` / cross-language bridge unaffected.
3. **Single-stem `term_vocab` rows (538,648) unchanged** → query-time concept expansion (`WHERE term IN (…)`) unaffected.
4. **`ctse/search.rs` read path unchanged** (already skips bigrams).
5. **FTS5 tokenizer still emits bigrams to `notes_fts`** (phrase queries still match) — only the `term_vocab` twin stops.
6. **Boot never blocked** by the delete or the VACUUM (background, post-paint, resumable).
7. **Crash/interrupt-safe + resumable** (mirror MIG-015's chunk pattern).
8. The **crown-jewel four** (cross-language, `≈ similar`, Arabic, phrase) all keep working — verified each reads a store in invariants 1–3, none of which we touch.

---

## 4. Design options

**(a) Stop writing bigrams to `term_vocab`:**
- **A1 — filter in `token_counts` (CHOSEN).** After `tokenize_to_vec`, drop tokens containing `BIGRAM_SEP` before counting. Surgical; one consumer; the shared tokenizer + FTS5 path untouched; same predicate `search.rs` already uses on read (symmetric). Effort: tiny. Risk: minimal.
- A2 — stems-only variant of `tokenize_to_vec`. More code, same effect. Rejected.

**(b) Delete existing 5.19M bigram rows:**
- **B1 — chunked background worker (CHOSEN).** Reuse the MIG-015 pattern: `DELETE FROM term_vocab WHERE rowid IN (SELECT rowid … WHERE term LIKE '%'||CHAR(31)||'%' LIMIT N)`, drop+reacquire the DB mutex between chunks, progress events + status strip, resumable by construction. Replaces the v2-sentinel worker. Effort: moderate (mostly repurposing existing code). Risk: low.
- B2 — single big DELETE. Blocks/locks; rejected.

**(c) Reclaim freed space:**
- **C1 — one-time background `VACUUM` after the delete (CHOSEN).** Required to return pages to the OS (a plain delete only frees them to the freelist *inside* the 2.35 GB file → no cold-boot win). Run once, off the boot path, status strip, guarded by a sentinel so it never repeats. Pre-check free disk (`VACUUM` needs ~2× DB size temporarily) and skip-with-log if insufficient. Risk: the exclusive lock during VACUUM (~tens of seconds on 2.35 GB) blocks search/index IPCs briefly — acceptable for a one-time post-boot maintenance op; `busy_timeout=5000` already set.
- C2 — `VACUUM INTO` + file swap. Can't easily swap a live open DB on Windows; rejected.
- C3 — `auto_vacuum=INCREMENTAL`. Needs a full VACUUM to take effect anyway; could be set *during* C1 so future deletes auto-reclaim. Nice-to-have.

---

## 5. Plan (each phase = one commit + verification clause)

> **Phase A — Stop writing bigrams to `term_vocab`.**
> Filter `BIGRAM_SEP` tokens out in `ctse::hooks::token_counts`. Add a unit test: a two-same-script-word body yields stems-only counts (no `0x1F` key).
> *Verify:* `cargo test` (hooks) green; save a note in a dev build → no new bigram rows appear in `term_vocab` (before/after `COUNT(*) WHERE term LIKE '%CHAR(31)%'`).

> **Phase B — One-time chunked delete of existing bigrams (background, resumable).**
> Repurpose the MIG-015 v2-sentinel worker into a "delete bigrams" worker (chunked DELETE, mutex released between chunks, `migration:term_vocab` progress events, `MigrationProgressStrip`). Schedule from `ensure_search_db_ready`, gated on "bigram rows remain?".
> *Verify (on a COPY of the real 2.35 GB DB first):* bigram rows → 0; single stems intact (538,648); `≈ similar`, phrase, Arabic, cross-language all work; Index panel populates (reads `notes_vocab`, unaffected). Then on the live DB.

> **Phase C — Reclaim space (one-time background VACUUM).**
> After the delete worker reports 0 remaining, run one `VACUUM` (background, disk-space pre-check, run-once sentinel). Optionally set `auto_vacuum=INCREMENTAL`.
> *Verify:* DB file ~2.35 GB → ~1.75 GB; `PRAGMA integrity_check` = ok; all search features work; boot not blocked (cold boot timed before/after).

> **Phase D — Retire obsolete machinery (/simplify).**
> Remove `count_pending_v2_sentinel_rows`, `sentinel_bigram_rows_chunk`, the v2-sentinel scheduling, and (optionally) the dead `bridge_concept_id` column. Resolves the `project_mig013` boot-blocker.
> *Verify:* `cargo build`/`clippy` clean; no dangling references; first-boot + existing-DB boot both clean.

> **Phase E — Audit (Migration Rule Phase 4).**
> Three agents in parallel: (1) invariants §3 hold; (2) drift — any new guard/path the plan didn't know about; (3) migration path — fresh DB, existing 2.35 GB DB, mid-delete interrupt/resume, pre-MIG-013 backup, low-disk VACUUM skip, rollback.

---

## 6. Migration-path matrix

| Scenario | Behavior |
|---|---|
| **Fresh DB** | Phase A → no bigrams ever written; B/C no-op (nothing to delete/reclaim). |
| **Existing 2.35 GB DB (Eisa)** | A stops new bigrams; B deletes 5.19M in background; C VACUUMs once → ~1.75 GB. |
| **Mid-delete interrupt** | Resumable — WHERE clause excludes already-deleted; re-entry continues. |
| **Pre-MIG-013 backup** | Gets the delete worker instead of the slow v2-sentinel UPDATE → **boot-blocker resolved**. |
| **Low disk for VACUUM** | Pre-check; skip with a log; delete still done (growth stopped, pages freelisted); retry VACUUM later. |
| **Code rollback** | DB simply has fewer bigrams; `search.rs` skips them anyway → no corruption; old write path would slowly re-add them. Safe. |

---

## 7. Risk summary

**Low.** The bigrams are verified-dead (read path skips them; Index panel/phrase/Arabic/≈similar all read other stores; proven safe on a real-DB copy earlier). The only operationally-touchy step is the one-time VACUUM lock, mitigated by running it off the boot path with a disk pre-check and status strip. No schema-breaking change; rollback-safe.

---

## 8. Build outcome + the concurrency bug (2026-05-22)

Phases A–E built + audited cleanly. **First live run STALLED at exactly 600k rows** and sat frozen ~7 hours.

**Root cause:** the chunked purge worker (shared `state.db` connection, `busy_timeout`=5s) was **fatal-on-error**, and the **WAL checkpoint daemon** (`spawn_wal_checkpoint_daemon` — own connection, `wal_checkpoint(TRUNCATE)` every 5 min) **collided** with it. As the purge's deletions filled the WAL, the daemon's TRUNCATE grew slow; a collision returned `SQLITE_BUSY` past the worker's 5s timeout, the `?` propagated, and the worker thread exited — dead for the session. **The isolated copy-test could not reveal this** (no app, no daemon, no concurrency). The Phase-E drift audit checked daemon-vs-VACUUM but not daemon-vs-purge-worker — a gap.

**Four-part fix** (committed after the architect phases, Boss-approved):
1. **Pause the daemon during the migration** — `MIGRATION_ACTIVE: AtomicBool`; the daemon stands down (polls every 15s) while set; a drop-guard clears it on every worker exit path.
2. **Retry, don't die** — the chunk DELETE retries transient `SQLITE_BUSY`/`locked` (≤120× with backoff) before giving up (then the next boot resumes).
3. **Self-checkpoint** — the worker TRUNCATE-checkpoints every 1M rows to bound the WAL while the daemon is paused.
4. **Log to `diagnostics.log`** — start / progress / retry / done / VACUUM / errors, so a future stall is directly diagnosable, never inferred.

**Validated end-to-end on the live 2.35 GB DB** (test #2, resumed from 600k): purge `deleted 4591533 rows in 799.9s` → `term_vocab_bridge` stamped 3 → VACUUM `done in 339.6s` (freelist 125,061 pages) → `term_vocab_vacuum` stamped 1. Final: **term_vocab 538,648 · bigrams 0 · stems 538,648 · integrity ok · file 2.35 → 1.75 GB (26%)**. Boss-confirmed.

**Lesson (→ migration checklist):** a one-time chunked DB worker MUST be (a) resilient to `SQLITE_BUSY` and (b) coordinated with any other background DB user (here, the WAL daemon). **Test migrations under live app concurrency, not just on an isolated copy.**
