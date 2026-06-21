# MIG-083 — Architect: Review Pulse → Rule-8 Write-Time Schedule

> **Concept (the horse):** Review Pulse resurfaces a note at the right moment so the user re-confronts a held position ("still true? supersede it?") — driving **Tension → Conviction**. Not flashcards. See `docs/concept-papers/22-review-pulse.md`.
> **Why this migration:** the Boss ruled (2026-06-21) *fix Rule 8 first, then build §F's full-page reviewer.* This Architect doc precedes the Plan. Phase 1 of the `/migration` four-phase process.

## 1. Territory — how it works today, and where Rule 8 breaks

- **The read path recomputes everything.** `get_due_notes` (`review.rs:43`) calls `scan_due_recursive` (`review.rs:191`) on **every** call: a full library `read_dir` recursion, `fs::metadata` per never-reviewed note, and — for checkpoints — a **full `fs::read_to_string` + regex `#(assumption|model)` of EVERY `.md`** (`review.rs:269`). Nothing is persisted as a "due list."
- **`review-pulse.json` stores only user ACTIONS** (`review.rs:19`) — path-keyed `last_reviewed` / `snoozed` / `interval` / `dismissed`. It is the durable, File-Over-App, syncable source of truth for what the user *did*; it does **not** store the computed queue.
- **Budget unmet:** the concept paper's <100 ms-on-7,600-notes target is unachievable while the read walks the filesystem. The §F full-page reviewer (`+layout.svelte:7315/7527`) and the palette (`:2140`) both call the heavy `get_due_notes`.
- **The precedent already exists in-repo — `tag_counts` (MIG-079 §C.1):** a derived SQLite table maintained by a **write-time delta inside `index_note`'s `BEGIN IMMEDIATE` transaction** (`search.rs:4586-4648`), **gated on `schema_versions`**, built by a **post-paint resumable back-fill** (`search.rs:7334`), and **self-healed by reconcile**. Crucially, `index_note` already has the note's tags in `note_meta.tags_json` (`search.rs:3792`) and its `modified`/`created_at` — **zero extra `.md` reads**. Six back-fills already register at `search.rs:7315-7361`; `sky_backfill` is the resumable-cursor template.

### Quirks to preserve *byte-for-byte* on the swap (LL-023 — replicate ACTUAL, not the docs)
The current code's *real* behavior differs from the concept paper / docs. The swap must reproduce the **actual** behavior so the only change is performance; each "fix" below is a **separate Boss decision**, not part of the swap:
- `stratum` is **faked to `2u8`** (not a real maturity read).
- The declared `stale` reason (`review.rs:36`) is **never emitted**.
- The interval ladder is **1 → 2 → 4 → 8 → 16 → 30** (doubling, cap 30), **not** the docs' 1 → 3 → 7 → 14 → 30.
- `record_note_visit` (`review.rs:135`) is registered but **never called** (§F wires it from `openNoteTab`).

## 2. Options (speed / effort / risk)

### Option A — `review_schedule` SQLite table, indexed `due_days`, write-time-maintained  ✅ RECOMMENDED
- **How:** new table `review_schedule(path PK, reason, due_days INT, is_checkpoint INT, last_reviewed)` + `INDEX(due_days)`. Store the **due day**, compute `days_overdue` at read (never an `is_due` flag — clocks roll at midnight). Maintain it in `index_note`'s existing `BEGIN IMMEDIATE` txn at the `tag_counts` apply-delta site: `is_checkpoint` from `note_meta.tags_json` (no `.md` read), never-reviewed age from `note_meta.modified` (no `stat`), interval from `review-pulse.json`. Removal in `reindex_delete_note`. **Rust-in-`index_note`, not a pure SQL trigger** (it joins `review-pulse.json` facts). Actions (`mark/snooze/dismiss/record_note_visit`) mutate `review-pulse.json` **and** upsert/delete the row. `get_due_notes` keeps the `DueNote` shape = `SELECT … WHERE due_days <= today AND not dismissed ORDER BY stratum desc, overdue desc`; **new** `get_note_review_status(path)` = a PK lookup. Back-fill = `review_backfill.rs` on the `sky_backfill` resumable cursor (idempotent `INSERT OR REPLACE`, 1000-row batches, crash-resume cursor, post-paint, status-bar progress, stamp on completion); reconcile self-heals.
- **Speed:** indexed scan **< 100 ms** vs the full FS walk; each save writes a per-note no-op delta. **Effort:** medium, ~600–900 LOC templated on `tag_counts` + `sky_backfill`. **Risk:** low-med (the checkpoint-definition gap, I6).
- **Verdict:** fits Rule 8, File-Over-App, the existing `tag_counts` seam, and the proven Anki/job-queue "indexed `due` column" pattern (WA#5).

### Option B — extend `note_meta` with review columns  ❌ REJECT
Re-widens the **hottest boot table** that MIG-078 BL is actively slimming; boot indexes would exclude them; every save touches them. A side table gets the same speed without re-widening.

### Option C — cache the due-list inside `review-pulse.json`  ❌ REJECT
No indexed query (parse+scan every open); **violates File-Over-App** (mixes a derived cache into the user-action source); goes stale on any mtime/tag change and at midnight. The concept paper §8 already prescribes Option A.

## 3. WA#5 — the proven pattern
Mature SRS systems (Anki's `cards` table with an indexed `due`/`ivl`/`factor`, queried `WHERE due <= today`; SM-2 / FSRS schedule storage) **persist the due value and query it by index — they never re-scan the note corpus to find what's due.** Option A is that pattern; B/C deviate.

## 4. Invariants (must not break)
- **I1 File-Over-App** — `review-pulse.json` is the ONLY authoritative action store; `review_schedule` is derived/rebuildable; no `.md` write. *Verify: delete the table + restart → reconstructs; corrupt + reconcile → restores.*
- **I2 Action semantics exact** — dismissed permanent; snoozed excluded while `snooze_until > today` (strict, `review.rs:219`); interval doubles from 1, cap 30; reasons `never_reviewed`/`interval_due`/`checkpoint`. *Verify old == new.*
- **I2b Replicate ACTUAL, not aspirational (LL-023)** — stratum `2`, `stale` never emitted, interval `1 2 4 8 16 30`. Byte-equivalent swap; each fix a **separate Boss item**.
- **I3 Rule-8 read path** — indexed SQL only; **zero** `read_dir`/`metadata`/`read_to_string`/regex on read; checkpoint from `tags_json`; age from `note_meta.modified`. *Verify zero `.md` syscalls; <100 ms on 7,600.*
- **I4 Write-time hook** — via the `index_note` seam, same txn as `tag_counts`; no `$effect`/trigger loop; body-only save is a no-op; no `invoke` on the hot path; no boot/typing/IPC regression. *Verify type-burst unchanged; `mark` drops the row with no walk.*
- **I5 Consumers identical** — sidebar badge (`layout.svelte:441`), `get_note_review_status` (§F), the §F reviewer, `record_note_visit`. *Verify badge == old scan; an opened due note leaves the queue with no rescan.*
- **I6 Checkpoint definition reconciled BEFORE swap** — the old scan uses **inline `#(assumption|model)` regex on `.md` content**; the new path uses **`note_meta.tags_json`** (deduped, includes frontmatter tags, may miss inline-only). *Diff on the live corpus; Boss pins the canonical definition; harness enforces.*
- **I7 Migration gated on `schema_versions.review`** — first-boot inert (legacy scan until stamped); un-upgraded cUniverse scans; crash mid-back-fill → no half-stamp (cursor + stamp-on-completion); rollback = bump version, reconcile self-heals. *Verify kill mid-back-fill resumes clean; old binary ignores the table.*
- **I8 Back-fill** — background, post-paint, resumable, status-bar progress; must not regress boot/typing/IPC on 7,600. *Measure before/after.*

## 5. The verification harness (before any swap)
`rehearse_against_live_copy`: on a **COPY** of the live 7,600-note DB, assert the new indexed read == the old FS scan for the full trio (dismissed; snoozed-tomorrow-vs-today strict; cap ladder; never+interval+checkpoint). Resolve **I6 first** by diffing raw-regex vs `tags_json` on the corpus so the Boss pins the checkpoint definition before the rehearsal runs.

## 6. Open questions — Boss decisions before the Plan
1. **Checkpoint definition (I6) — RESOLVED via live-corpus diff (2026-06-21).** Scanned all **7,609** notes of `E:/Cognitive Knowledge` (the live 7,611-note Universe): **inline `#assumption`/`#model` = 0; frontmatter `assumption`/`model` tag ≈ 0; total checkpoints = 0 under either definition.** (The `#tag` feature *is* used — 97 notes carry inline tags — just not these two.) So the difference is **zero on today's data**, and the per-note `read_to_string` scan reads all 7,611 files to find nothing. **Decision: pin `tags_json`** — the complete superset (frontmatter + inline), risk-free now and future-correct (catches Properties-tagged checkpoints the old `#`-only regex misses). `index_note` already builds `tags_json` from both sources (`search.rs:3792`), so it is free at write time.
2. **The quirks (I2b):** recommended = **replicate ACTUAL for the swap** (stratum `2`, no `stale`, interval `1 2 4 8 16 30`), and treat each fix (real stratum / add `stale` / change the ladder to `1 3 7 14 30`) as a **separate item after** the Rule-8 swap lands. Confirm, or fold any fix into the swap.
3. **Scope confirmations:** `review-pulse.json` stays the durable action log; rename path-key fragility is out of scope; `record_note_visit → openNoteTab` wiring lands in §F (after this).

---
*Architect phase complete. Next: Boss decides §6 → Plan (phase-by-phase, each landable + verifiable) → Build → Audit (3 agents).*
