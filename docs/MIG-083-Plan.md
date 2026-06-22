# MIG-083 — Plan: Review Pulse → Rule-8 Write-Time Schedule

> **Phase 2 of `/migration`.** Architect: [MIG-083-Architect-Review-Pulse-Rule8.md](MIG-083-Architect-Review-Pulse-Rule8.md). Concept paper: [22-review-pulse.md](concept-papers/22-review-pulse.md).
> **Concept (horse):** resurface a note to re-confront a held position → Tension → Conviction.
> **Design (Option A):** a derived `review_schedule` SQLite table (indexed `due_days`), maintained **write-time in `index_note`** (the `tag_counts`/MIG-079 §C.1 pattern), gated on `schema_versions.review`, built by a resumable post-paint back-fill. Reads become `SELECT … WHERE due_days <= today`. `review-pulse.json` stays the **only** durable action source (File-Over-App).

## Locked decisions (Architect §6)
- **Checkpoint = `tags_json`** (frontmatter + inline `#` tags). Corpus-verified: 0 `#assumption`/`#model` notes in the live 7,611-note Universe today → zero-risk; future-correct (catches Properties-tagged checkpoints the old `#`-only regex misses); free at write time (`index_note` already builds `tags_json`, `search.rs:3792`).
- **Fix all 3 quirks IN the swap** (so the new behavior is the *corrected* one): (a) **stratum** = the note's real maturity from `sky_nodes.stratum` (MIG-002, SQL-trigger-maintained), not the faked `2u8`; (b) **interval ladder** = the documented **1 → 3 → 7 → 14 → 30** (cap 30), not `1·2·4·8·16·30`; (c) **`stale` (Mode 2) — now a DEFINED, WA#5-validated concept** (concept paper §12): a note is stale when a **load-bearing OUT-dependency** (`supports`/`contradicts`/`derives-from`/`part-of`/`supersedes`; NOT `associative`) had its **content change** (not a touch) **after its last explicit review**; all maturities; 1-hop; ranked+explained; a **separate lens** from Mode 1. The rehearsal harness asserts the indexed read == the **corrected** reference (not parity with the old buggy scan).
- **`last_reviewed` = explicit "✓ Reviewed" only** — opening/editing does NOT count. **Consequence: the old §F `record_note_visit → openNoteTab` wiring is dropped; `record_note_visit` is removed/unwired.** Both Mode-1 resurfacing and Mode-2 staleness key off the explicit-✓ `last_reviewed`.
- **Content-change signal:** maintain a per-note **`content_changed_at`** bumped at write-time ONLY when the content hash actually changes (autosave/sync/cid_cn touches must NOT bump it). Mode-2's JOIN uses `dependency.content_changed_at > last_reviewed`, with `mtime` as the cheap indexed pre-filter. (If no content hash exists yet, reuse the cid_cn/FTS path — verify in §B.)

## Invariants (must hold every phase — Architect §4, I1–I8)
File-Over-App (review-pulse.json authoritative; table rebuildable; no `.md` write) · action semantics exact (dismissed permanent; snoozed strict `> today`; interval cap 30) · Rule-8 read path (indexed only; zero `read_dir`/`metadata`/`read_to_string`/regex on read; <100 ms on 7,600) · write-time via the `index_note` seam (no trigger/effect loop; body-only save no-op; no `invoke` on hot path; no boot/typing/IPC regression) · consumers identical (badge, §F status, §F reviewer, `record_note_visit`) · migration gated on `schema_versions.review` (first-boot inert; crash mid-back-fill resumes; old binary ignores the table) · back-fill background/post-paint/resumable/status-bar.

---

## Phase §A — Schema + the corrected pure scheduling logic *(no behavior change)*
**Changes**
- `review.rs`: define the **corrected** scheduling as pure, testable functions (no I/O): `next_interval(prev) → 1·3·7·14·30 cap 30`; `due_day(last_reviewed, interval)`; `reason_for(note_state) → never_reviewed | interval_due | stale | checkpoint`; `is_checkpoint(tags_json)`; `stratum_of(note)` (real maturity). A `ScheduleRow` struct (`path, reason, due_days, is_checkpoint, last_reviewed, stratum`).
- `search.rs init_db`: `CREATE TABLE IF NOT EXISTS review_schedule(path TEXT PRIMARY KEY, reason TEXT, due_days INTEGER, is_checkpoint INTEGER, last_reviewed TEXT, stratum INTEGER)` + `CREATE INDEX idx_review_due ON review_schedule(due_days)`. Inert until stamped. Register `schema_versions` key `review` (mirror `tag_counts::is_stamped`).
**Verify (one commit):** `cargo test review` — unit tests pin the corrected ladder (1→3→7→14→30, cap 30), the strict-snooze boundary, `is_checkpoint(tags_json)`, real stratum, and the `stale` path. Table created but **unread/unwritten** → zero runtime behavior change. `svelte-check` 0; build clean; boot unchanged (table inert).

## Phase §B — Write-time maintenance (`index_note` upsert + the action writers)
**Changes**
- `index_note` (inside the existing `BEGIN IMMEDIATE` txn, at the `tag_counts` apply-delta site, gated on `review` stamped): `review_schedule::upsert_row(note)` from the §A logic — `is_checkpoint` from `note_meta.tags_json` (zero extra read), never-reviewed age from `note_meta.modified` (no `stat`), `last_reviewed`/`interval` from `review-pulse.json`, real `stratum`. Removal in `reindex_delete_note`.
- The action writers `mark_reviewed`/`snooze_note`/`dismiss_note`: each mutates `review-pulse.json` (source) **and** upserts/deletes the note's `review_schedule` row, using the **corrected 1·3·7·14·30 ladder**. **`record_note_visit` is removed/unwired** (opening ≠ review, per the locked decision).
- **Mode-2 content signal:** maintain a per-note **`content_changed_at`** (column on `note_meta` or `sky_nodes`, whichever already carries the content hash — verify), bumped in `index_note` ONLY when the freshly-computed content hash differs from the stored one (reuse the cid_cn/FTS hash; a touch/whitespace/frontmatter-only/sync edit must NOT bump it).
**Verify (one commit):** `cargo test` — saving a note upserts the correct row; `mark_reviewed` advances `due_days` by the corrected ladder; `dismiss` drops the row; a body-only edit is a no-op delta; **`content_changed_at` bumps on a real content edit but NOT on a touch/frontmatter-only save**. `svelte-check` 0; build. **Reads still use the old scan** (not yet stamped) → no user-visible change yet. Typing-latency unchanged.

## Phase §C — Resumable back-fill + reconcile self-heal
**Changes**
- `review_backfill.rs` (template: `sky_backfill`): populate `review_schedule` from `review-pulse.json` + `note_meta` in **1000-row batches**, a `review_schedule_cursor` for crash-resume, `INSERT OR REPLACE` (idempotent), a 50 ms inter-batch sleep, a dedicated WAL connection, **post-paint** when `review` is unstamped, **status-bar progress** via a Tauri event, **stamp `review` on completion**.
- `reconcile`: `recompute_all_in` self-heals the table on a background reconcile.
**Verify (one commit):** a test fills the table from a seeded `review-pulse.json` + `note_meta`; **kill mid-back-fill → resumes clean from the cursor, no half-stamp**. Manual: on the live 7,611-note copy the back-fill runs after paint with status-bar progress; boot/typing unaffected. Old binary opening the DB ignores `review_schedule` (forward-compat).

## Phase §D — The read swap + `get_note_review_status` + the rehearsal harness *(Boss-testable)*
**Changes**
- `rehearse_against_live_copy` (test/dev harness): on a **COPY** of the live 7,611-note DB, assert the indexed read == the **corrected reference** full computation for the full trio (dismissed; snoozed-tomorrow-vs-today strict; the 1·3·7·14·30 ladder; the never+interval+checkpoint+stale set).
- Swap `get_due_notes`: when `review` is stamped → the **union of two lenses**: (1) Mode 1/3 from the table — `SELECT … FROM review_schedule WHERE due_days <= today AND reason != dismissed`; (2) **Mode 2 staleness JOIN** — `review_schedule rs` ⋈ `note_links jl` (where `jl.source = rs.path` AND `jl.type IN (supports,contradicts,derives-from,part-of,supersedes)`) ⋈ `note_meta dep ON dep.path = jl.target` WHERE `dep.content_changed_at > rs.last_reviewed` — out-dependency, load-bearing, content-change, 1-hop. Each `DueNote` carries its `reason` + (for stale) the triggering neighbour + link type + date (ranked by link weight/confidence). Order: stratum DESC, then days_overdue DESC. Else (unstamped) → the legacy scan. **Measure the JOIN on the 7,611-note copy; if >100 ms, escalate to a trigger-maintained `neighbourhood_max_changed` column.**
- **New** `get_note_review_status(notePath) → { last_reviewed, due_days, reason, never_reviewed }` — a PK lookup (the §F note-context Review tab consumes this).
**Verify (one commit):** `rehearse_against_live_copy` passes; `get_due_notes` returns **<100 ms** on the 7,611-note copy with **zero `.md` syscalls** on the read path (assert no `read_dir`/`read_to_string`); the sidebar badge count matches the corrected reference. **Boss test** (Testing Instructions Rule): the Review tab opens instantly; ✓/Snooze/Dismiss persist and the row leaves the queue; the documented ladder is observable.

## Phase §E — Cleanup + Audit
**Changes**
- Remove the dead `scan_due_recursive` read path (kept through §D as the unstamped fallback) once the swap is proven; `/simplify` on the full MIG-083 diff.
- **Audit (3 agents in parallel):** invariants (I1–I8); drift (the new gate/table/back-fill vs what the system records — LL-023); migration-path (first-boot, schema mismatch, mid-back-fill interrupt, rollback, un-upgraded cUniverse). Re-run the read-path Rule-8 assertion.
**Verify (one commit):** audit clean; `/simplify` applied; orientation v-bump (SO #6) folding MIG-083; help/manual updated; session log; MoCh.

---

## After MIG-083 → MIG-080 §F (now unblocked)
With the cheap schedule live: §F builds the note-context **Review tab** (`get_note_review_status`, O(1)) + adds `'review'` to `NOTE_SCOPED_TABS`; and the new **full-page reviewer** (`ReviewerView`) over the now-cheap `get_due_notes`, presenting **Due-for-review** and **Stale** as two distinct lenses with per-row "why." **The old `record_note_visit → openNoteTab` wiring is dropped** (opening ≠ review — locked decision); the only thing that advances `last_reviewed` is the explicit **✓ Reviewed** action. Then §G closes MIG-080.

*Plan complete. Awaiting Boss approval → Build (§A→§E, cascading per Plan-Approval=Build-Approval, stopping at each Boss-testable verify clause).*
