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

### Re-verification pass (4 agents on the fix diff) — `all_fixes_correct: true` + 2 NEW findings (both fixed)
The fixes were re-verified adversarially; the 6 fixes are correct with no regressions in them. Two NEW issues surfaced + fixed (WA#6):
- **[P3] back-fill-window note keeps `content_hash` NULL → first post-stamp touch false-bumps** (residual of A). A note created during an inter-batch window at a path below the cursor is never baselined. Fix: `index_note` now treats a first-observation (`old_content_hash IS NULL`) as a **baseline** — writes the hash, does NOT set `content_changed_at`. Hardens against *any* never-baselined note.
- **[P2 — NEW §D regression] rename orphans the old-path `review_schedule` row → phantom queue entry** (dead path). The legacy FS scan could never list a nonexistent path. Two-layer fix: (a) `rename_item` migrates the row (`DELETE` stale new-path row, then `UPDATE … SET path`) — kills the orphan AND carries ✓ history to the new path (closes J's rename case); (b) Lens-1 + Lens-2 changed `LEFT JOIN note_meta` → **INNER JOIN**, so an orphan from *any* source can never surface. Unit test `indexed_read_excludes_orphan_rows` added.

**20 review unit tests pass; live rehearsal: touch-test 0, parity exact, Mode-2 fires, 16.73 ms.**

**Boss rulings (2026-06-22, via AskUserQuestion):**
- **B → "Substantive edits only (ship now)"** — keep hashing `plain_body`; pure formatting/diacritic/URL-only edits don't trip staleness (prose edits do). No code change; diacritic-sensitivity is a documented fast-follow if wanted later.
- **C → "No — keep the lenses fully separate"** — REVERSED my snooze-suppresses-stale: snooze hides a note from **Due-for-Review (Lens-1) only**; a snoozed note can STILL surface as **Stale (Lens-2)**. Removed the Lens-2 snooze clause + the reference skip; kept the `snoozed_until` column (Lens-1 hide + the finding-E re-index preservation). Test renamed → `snooze_hides_from_due_not_from_stale`.

**Still surfaced / deferred (not parked):**
- **J [P3] — rename loses review state.** The `review_schedule`-row half is now FIXED (rename migrates the row, carrying ✓ history). The `review-pulse.json` path-keyed half remains (pre-existing) — separate follow-up.
- **K [P3] — §C reconcile self-heal** — confirmed NOT shipped (`reconcile_filesystem` has no review recompute). Self-heals on a note's next save; → §E's migration-path audit implements it.

## Boss test — PASSED (2026-06-22)
Built release binary (16:45), Boss-tested in two stages:
- **Stage 1 PASS** — Review panel opens instantly; ✓/Snooze/Dismiss persist + remove the row. (Empty universe correctly shows only "Never Reviewed" — Due/Checkpoints are earned.)
- **Stage 2 PASS (Mode-2 live)** — built a self-contained "Review Demo" scratch universe (Claim --derives-from--> Evidence) + `seed_review_demo` harness (seeds the app's OWN DB at its real paths, resolves the link, Claim reviewed-10d + Evidence changed-today). Boss reopened → **🥀 Claim under "Due for Review"** = stale. Mode-2 confirmed end-to-end on real files.
- **Demo-plumbing lessons:** (a) a forward link to a not-yet-indexed target resolves `target_cid_cn` to NULL until a later re-index (pre-existing link-system behavior; self-heals); (b) seed the app's OWN DB (not a pre-built one) to avoid path-form mismatch; (c) open the seed connection via `init_db` (registers the custom FTS tokenizer). Scratch universe at `E:\Constellation Universes\Review Demo` (delete at §E cleanup unless kept).

## §D-grace — SHIPPED (`b366e654`)
Configurable **staleness grace period** Setting (days, min 1, default 1). Mode-2 fires when `local_day(dep.content_changed_at) − last_reviewed_day ≥ grace`. `get_due_notes` gains an optional `stale_grace_days`; `appSettings.review.staleGraceDays` (merged safely); new **Review** Settings section (clock icon) with a min-1 number field; 4 i18n keys × **15 locales** (native, all validated). Unit test `stale_grace_period_gates_by_days`. Grace binary built 18:13.

## §E — migration close
- **§E-1 SHIPPED (`5ca1d8aa`):** retired `scan_due_recursive` (the Rule-8-violating FS-walk fallback). Unstamped `get_due_notes` now kicks the back-fill (idempotent) + returns empty. No `read_dir`/`metadata`/`read_to_string`/regex anywhere on the read path.
- **§E-2 SHIPPED (`6...`):** `/simplify` (4-agent) — applied scope_clause helper (single-sourced finding-D predicate), `staleness_types_sql()`, `date_to_days→parse_day`, `epoch_2020()`, `today_str` reuse, and a **per-batch transaction** in the back-fill (≈2N commits → 1). Skipped riskier refactors of verified functions. 22 tests + rehearsal green.
- **§E-3 SHIPPED (`...`):** the `/migration` Phase-4 audit (3 agents — invariants I1–I8 / drift LL-023 / migration-path). All real findings fixed per WA#6: **the P1 missing reconcile self-heal** → `review::recompute_all_in` (orphan sweep + rebuild from note_meta + review-pulse.json) wired into `reconcile_filesystem` gated on `is_stamped`, mirroring `tag_counts` (also heals the P2 stratum-drift + the back-fill-window missed-note); extracted `review::backfill_one` (shared by §C back-fill + recompute + the harness — kills the triple copy); the **maybe_schedule re-entrancy** (the §E lazy kick could spawn concurrent back-fills) → a `static AtomicBool RUNNING` one-shot guard; `REVIEW_SCHEMA_VERSION` const + version-aware `is_stamped` (sibling rollback pattern); fixed the stale `ensure_note_meta_review_columns` COALESCE comment. **23 review unit tests** (incl. `recompute_all_in_sweeps_orphans_and_rebuilds`); live rehearsal green.

**MIG-083 §A–§E is COMPLETE + Boss-validated + triple-reviewed (24-finding review + 4-agent re-verify + Phase-4 audit) + all findings fixed.** Final release binary built 18:49.

## SO close-out
- **Orientation v-bump (SO #6):** `docs/Constellation Orientation & Onboarding v3.01.md` (new file alongside v3.00) — v3.01 preamble (MIG-083 shipped end-to-end) + the Review Pulse §4.3 storage row flipped to ✅ write-time.
- **Help / User Manual ×15 — deliberately tied to MIG-080 §F (NOT skipped).** The Review-Pulse *surface* is mid-transition: the current panel is the transitional one (the code itself notes "§F will split this panel"), and §F builds the **note-context Review tab + the full-page two-lens reviewer** — the FINAL surface a user manual should describe (incl. the "stale because {dep} changed" line, which §D produces but the panel doesn't yet render). Documenting the transitional panel now + re-documenting in §F is wasted motion. **§F's first task: the Review-Pulse help topic + User Manual section ×15** (the instant panel, the Due-for-Review vs Stale two lenses with per-row "why", the ✓/Snooze/Dismiss actions, AND the Settings → Review grace-period control). The grace Setting + Stale concept are stable and fold cleanly into that §F pass.

## Boss ruling (2026-06-22, post-close) — Review Pulse placement for §F
Boss asked whether Review Pulse stays a right-sidebar tab or becomes a left-dock core plugin (astutely noting it's universe-scoped, not note-related, while the right sidebar is now note-context). **Ruling (AskUserQuestion): the universe-wide reviewer → a LEFT-DOCK core surface** (clock nav icon → the full-page two-lens reviewer), matching Sky View / Index / Map / Calendar. The note's OWN status stays a right-sidebar **note-scoped** tab (`ReviewStatusPanel` on `get_note_review_status`). The transitional right-sidebar universe `ReviewPulsePanel` is retired in §F. **Pinned into `docs/MIG-080-Plan.md` §F** (which was STALE — it still said `get_note_review_status` reads review-pulse.json [it reads the table, built in §D-3] and that `record_note_visit` gets wired to `openNoteTab` [DROPPED — opening ≠ review]; both corrected). Memory `project_review_pulse_split_placement` written.

## Open / next (after §E close)
- **MIG-080 §F** (now UNBLOCKED): note-context **Review tab** (`get_note_review_status`, O(1)) + the **LEFT-DOCK** full-page **two-lens reviewer** (Due-for-Review + Stale, each with per-row "why") over the now-cheap `get_due_notes`; retire the transitional right-sidebar universe panel; help/manual ×15. Drop the old `record_note_visit→openNoteTab` wiring (already removed). → then **§G** closes MIG-080.

---

## MIG-080 §F — Review Pulse SPLIT (built; awaiting Boss test)
The split is built, reviewed, and shipped behind the running binary (rebuilt 20:04 — frontend rebuilt + embedded, verified via `build/` strings + fresh mtime; the `.exe` itself compresses assets so a binary grep is a false-negative).

- **§F-0** `217a59a5` — backend: `get_note_review_status` gained per-note Mode-2 staleness; extracted the shared `note_stale_status` probe (single-sources Lens-2 of `query_due_notes_indexed` + the note tab).
- **§F-1/2** `0c0b55f9` — components: `ReviewStatusPanel` (note-scoped status: due/checkpoint/never + the per-note 🥀 stale callout + ✓/Snooze/Dismiss), `ReviewerView` (left-dock full-page wrapper), `ReviewPulsePanel` enhanced (Stale is now its own lens with the per-row "why").
- **§F-3** `4d054216` — `+layout` integration: right-sidebar Review tab → NOTE-SCOPED (`'review'` ∈ NOTE_SCOPED_TABS, renders `ReviewStatusPanel`); NEW left-dock clock icon → full-page `ReviewerView`; `fullPageActive` += `showReviewer` + a guard `$effect` (mirrors the Calendar guard); command palette opens the reviewer; retired the right-sidebar universe panel + the orphaned `dueNotes` state and its 3 load sites.
- **Adversarial review** (4 dims × find→verify, 17 agents) → **11 confirmed findings, ALL fixed** in **§F-4** `<pending hash>` (Fix-What-You-Discover, none deferred):
  - **P1** i18n: the `$t(key)||fallback` idiom is DEAD here (a miss returns the truthy key-path) → the UI would have shown raw `reviewStatus.dueIn`. Added the 11 new keys (`reviewStatus.*` ×9 + `reviewPanel.stale`/`staleBecause`) to **all 15 locales**, native translations, tokens preserved+validated.
  - **P1** reviewer overlay stacked with the editor/dashboard → added `showReviewer` to `.content-area` `content-hidden`.
  - **P2** reviewer mutual-exclusion: dock button + palette now clear the FULL settable peer set (else the guard closed the reviewer the instant it opened over OrgChart/Health/SearchHub/Forge/Canvas/Calendar).
  - **P2** `ReviewStatusPanel` "due in N" used UTC vs Rust's LOCAL day (±1) → anchored to local midnight.
  - **P3** request-token guard on `load()`; `aria-label` via `$t`; restored prepare-once for the bulk Lens-2 probe (`note_stale_status_with_stmt`) + fixed the stale comment; rehearsal parity oracle made grace-aware.
- **Verification**: 21/21 review unit tests pass post-refactor; svelte-check 0; frontend rebuilt + embedded.

### Still pending in §F
- **Boss test** (staged): Stage 1 = the note-scoped Review tab; Stage 2 = the left-dock reviewer.
- **§F help/manual ×15** (the SO #2 docs pass) — deferred to AFTER Boss UI validation (final strings/behavior).
- Then **§G** closes MIG-080.

---

## MIG-084 — The Rich Reviewer (build, 2026-06-23)
After §F shipped the split, Eisa: the left-dock Reviewer is "a thin list on a vast empty page" — make it a RICH decision surface + add a Reviewer Style Setter (text resize). Cross-checked against the core plugins (the "prescription"); corrected my "orphan is disposable" error (an orphan is an ALARM). Rulings: master-detail; self-explanatory law; SIX lenses (Stale·Due·Checkpoints·🔗Orphan·⚠Fragile·Never); priority slider in BOTH the detail pane + the note tab; dedicated Reviewer Style Setter category.

- **§A** `<hash>` — Architect doc (`docs/MIG-084-Rich-Reviewer-ARCHITECT.md`).
- **§B** `<hash>` — DueNote enriched with incoming/outgoing counts + maturity (read-time via the shared maturity::compute_state; the planned §E maturity migration COLLAPSED into this reuse — Rule 8).
- **§C** `<hash>` — Orphan + Fragile lenses (from note_meta.incoming_count/word_count + a cheap derives-from count; reuse inspector360 thresholds; dismissed-excluded).
- **§D** — Priority (the ONE schema change). **WA#4 impact note:** `review_priority INTEGER NOT NULL DEFAULT 50` goes on **note_meta** (NOT review_schedule) — verified index_note's `ON CONFLICT(path) DO UPDATE SET` lists specific columns and does NOT touch review_priority, so it survives re-indexing; review-recompute only touches review_schedule, so it survives that too; every note (incl. orphans with no schedule row) has one; the lenses already JOIN note_meta. Default 50 ⇒ no back-fill. New cmd `set_review_priority`; sort ranks by priority first. Cuts: nothing.
