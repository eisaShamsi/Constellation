# Session Log — 2026-06-22 (MIG-083 §D — Review Pulse read swap + Mode-2 staleness)

> **Function in hand:** the **Review Pulse read path** — `get_due_notes` (the data source behind the right-sidebar **Review** panel). §D swaps it from a full-filesystem-walk recompute to a cheap indexed read of the write-time-maintained `review_schedule` table, and adds the **Mode-2 staleness** lens.
> **Concept (the horse):** resurface a note so the user re-confronts a held position → Tension → Conviction. Mode-2 = *a held position becomes suspect when something it depends on has changed* ("is this still consistent with what moved?" — Truth-Maintenance / changed-dependency invalidation), the complement of Mode-1's forgetting-curve.
> Branch `main`. Picks up from `HANDOVER-2026-06-22-mig083-b2.md` ("RESUME HERE — §D"). §A–§C committed + INERT (gated on `schema_versions.review`).

## Session-start ritual
- `git pull origin main` → already up to date (HEAD `540e1465`).
- Read orientation v3.00, `docs/MIG-083-Plan.md` §D, concept paper `22-review-pulse.md` §12 (Mode-2 staleness), and the handover.
- Mapped the code before touching it: `note_meta.content_hash` is a **dead column** (never written by `index_note`, no readers — confirmed by grep) → repurpose for Mode-2; `note_meta.name` is **already the display title** (set at index time) → the read needs no `.md` access; Mode-2 join key = `note_links.target_cid_cn → note_meta.cid_cn` (both UNIQUE-indexed — `target_path` is unset for fresh links).

## Work done — §D built in 4 landable, committed, verified pieces

### §D-1 — Mode-2 content-change signal (`content_changed_at`, gated) — `625ee6d6`
- `review::content_hash` — stable **FNV-1a 64-bit** hex of the body. Chosen over `std` `DefaultHasher` because the hash is **persisted** + compared across restarts/Rust-toolchain upgrades (the std algorithm is explicitly unstable across releases → would silently false-fire every dependent on a bump).
- `note_meta` gains `content_changed_at INTEGER` (idempotent `ensure_note_meta_review_columns`); `content_hash` also ensured for old DBs.
- `index_note` (gated on `is_stamped(review)`, inert until §C): bump `content_changed_at = modified` **only** when the FNV body hash differs from the stored one — a touch/sync/cid_cn/frontmatter-only save is a pure read+compare, zero writes.
- No back-fill needed: the §D JOIN uses `COALESCE(content_changed_at, modified)`.
- **13 review unit tests** (incl. `content_hash` stability vectors — empty == `cbf29ce484222325`).

### §D-2 — the read swap: `get_due_notes` → indexed SELECT ∪ Mode-2 JOIN — `8f024de7`
- `get_due_notes` takes the **Rule-8 fast path** once `review` is stamped (zero filesystem access), else falls back to the legacy scan (so the panel is never empty mid-back-fill).
- `query_due_notes_indexed` = **two distinct lenses** (Boss: never merged into one score): Lens 1 (Mode 1/3) `SELECT FROM review_schedule WHERE due_days <= today AND reason != 'dismissed'`, library-scoped (char-indexed `substr` prefix), name from `note_meta`; Lens 2 (Mode-2 staleness) reviewed-set × load-bearing OUT-links × `COALESCE(content_changed_at, modified)` changed on a **later calendar day** than `last_reviewed`, one row per stale note citing its most consequential changed dep.
- `DueNote` gains `stale_trigger_name/type/changed_on` (None for Mode 1/3); the transitional panel already folds `reason='stale'` into "Due for Review" (🥀) so stale is visible now — the full two-lens UI is §F. TS interface synced. svelte-check 0.
- **15 review unit tests** (added: two-lens scope/filters/associative-exclusion/COALESCE-fallback; dedup-to-most-consequential).

### §D-3 — `get_note_review_status` (O(1) PK lookup for the §F Review tab) — `19beb84d`
- Read-only single-row fetch (`reason, due_days, last_reviewed, never_reviewed, is_checkpoint`); clean "never reviewed" status when no row. Registered in `lib.rs`. Fixed a stale `mark_reviewed` doc comment.

### §D-4 — live-copy rehearsal harness + the perf fix it surfaced — `63ea58c4`
- `review_rehearse.rs`: a gated harness (`#[cfg(test)]`, runs only with `REVIEW_REHEARSE_DB`) that **copies** the live universe DB (never touches the original), migrates + back-fills the copy, then asserts (1) parity vs an independent Rust-loop reference, (2) `get_due_notes` < 100 ms, (3) Mode-2 fires on a seeded real-graph fixture.
- Run against the live **7,660-note "Eisa Cognitive Knowledge"** universe (233,998 `note_links`), the harness **FOUND a budget breach: 232 ms**. `EXPLAIN QUERY PLAN`: the single-JOIN Lens 2 drove from `note_links.status='active'`, scanning all ~234k active links, because `last_reviewed` looked unindexed on the freshly-built table.
- **Fixes (both verified by the harness):** (a) partial index `idx_review_last_reviewed ON review_schedule(last_reviewed) WHERE last_reviewed IS NOT NULL`; (b) Lens 2 restructured into two steps (fetch the tiny reviewed set, then probe each note's out-links with a prepared statement reused per note — every call rides `idx_link_source`; `lr_day` computed in Rust). Extracted `review::schedule_for` as the shared pure `(reason, due_days)` core so write + reference can't drift.
- **Result: 232 ms → 17.79 ms (release), parity exact, Mode-2 fires.** Zero-`.md`-syscall is structural + corroborated (a 7,660-note FS walk could never be <18 ms). **16 review unit tests pass.**

## Verification before Boss test — adversarial review workflow (5 dimensions × find→verify, 33 agents)
**24 confirmed findings (1 P1, 10 P2, 13 P3) → ~9 distinct issues** (dimensions overlapped). Triage + disposition (WA#6: fix discovered defects; surface genuine design/scope decisions, never silently park):

**Fixed in §D-fix-1 (query correctness):**
- **A [P1] — Mode-2 fired on file mtime, not content.** `COALESCE(content_changed_at, modified)` fell back to mtime during the NULL window (the *default* state of a freshly-stamped universe), so a sync/touch/cid_cn/frontmatter save falsely flagged dependents — the exact false-positive class §D exists to prevent; §12 violation. Fix: back-fill **baselines** `content_hash` from `note_meta.body_text` (no `.md` read); Lens-2 **drops the mtime fallback** (requires `content_changed_at IS NOT NULL`). Harness reference matched + touch-test added.
- **D [P2] — library scope sibling-bleed** (raw char-prefix, no boundary). Fix: separator-terminated prefix (empty = match-all passthrough for the universe-wide rehearsal).
- **F [P2] — UTC(dep)-vs-local(last_reviewed) day off-by-one.** Fix: compute the dependency day with `local_day()` (chrono::Local), matching the local `last_reviewed`; comparison moved to Rust (also resolves **H** div-truncation divergence).
- **I [P3] — self-links** could flag a note stale by itself. Fix: `dep.path != source`.
- **E [P3] — malformed `last_reviewed` → day 0 → spurious stale.** Fix: `parse_day()` (Option); skip unparseable.
- **G [P3] — non-deterministic "most consequential" tie-break.** Fix: `ORDER BY weight DESC, content_changed_at DESC, jl.id DESC`.

**Fixed in §D-fix-2 (snooze):**
- **C/E [P2] — snooze didn't suppress the Stale lens (#3/#10) + snooze lost on re-index (#7).** Fix: `snoozed_until` column on `review_schedule`, maintained by the snooze/mark actions + back-fill, preserved by `upsert_schedule_row`, and excluded from Lens-2. (Lens-1 already excluded via the pushed `due_days`.) Snooze-suppresses-stale flagged for Boss confirmation in the test.

**Harness strengthened (L/#13):** independent Lens-1 reference (no shared `schedule_for`); touch-test; self-link/snooze mirrored.

**Surfaced to Boss (genuine design/scope — not parked):**
- **B [P2] — content hash is over `plain_body`** (markdown-stripped + Arabic-normalized), so pure formatting / URL / **Arabic-diacritic** edits don't fire staleness (false *negatives*). Fixing cleanly needs the back-fill to read raw `.md` bodies (a Rule-8 departure for the one-time population) — a Boss-weighable call. Prose edits ARE caught.
- **J [P3] — rename loses review state** (`review-pulse.json` is path-keyed, not migrated on rename). **Pre-existing** (true of the legacy scan too); needs rename-cascade integration — separate follow-up.
- **K [P3] — §C reconcile self-heal** (planned, verify it shipped). Belongs to §E's migration-path audit.

## Open / next
- Boss test (Testing Instructions Rule), then §E (remove `scan_due_recursive`, `/simplify`, 3-agent audit, orientation v-bump, help/manual), then **MIG-080 §F** (note-context Review tab + two-lens reviewer over the now-cheap read) → §G.
