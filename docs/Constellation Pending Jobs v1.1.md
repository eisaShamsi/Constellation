# Constellation Pending Jobs

**Version 1.1 | 2026-05-06**

> **What changed in v1.1** (Boss-directed 2026-05-06): elevates the **Stable Reference Numbers** rule from the Appendix to the front of the doc — `PJ-NNN` IDs are reference numbers used across session logs, commit messages, and cross-doc references; they are never reused, never recycled, and never renumbered, even when a job is rejected, cancelled, merged into another, or split into siblings. Adds **Rejected** and **Cancelled** as explicit terminal statuses (alongside **Done**) that retire a number permanently with the entry preserved. Updates **PJ-007** from `Open · Boss design call` to `Confirmed · In-Progress` with the chosen baseline (Living Link 6-stage) and the proposed-defaults all approved. PJ-007's closure plan is now a focused MIG; PJ-006 P3 is unblocked by this confirmation.

**Version 1.0 | 2026-05-05**

> **What this is.** A durable, versioned project backlog. Every open job that isn't actively shipping right now lives here. The list is reviewed at the start of every work session and updated whenever a job opens, closes, or changes priority. Like the orientation and Laws docs, this file is versioned: a new version (`v1.1`, `v1.2`, …) is written as a NEW file alongside the previous one whenever the structure changes (new job added / existing job moves status / batch of jobs closes). Older versions stay as historical record so the trail of decisions is durable.
>
> **What this is NOT.** It is not the session log. The session log records what *happened today*. This doc records what's *open across the project*. Jobs flow from this doc into a session log entry when work begins; back into this doc as Done when work closes.
>
> **Audience.** Primary: every future Claude session. Secondary: the Boss reviewing what's outstanding. Tertiary: any future contributor.
>
> ### Stable Reference Numbers (foundational rule, v1.1)
>
> Each job has a stable `PJ-NNN` identifier that **acts as its permanent reference number** — like a ticket ID. The rules:
>
> - **Numbers are unique and never repeated.** PJ-007 means PJ-007 forever. If PJ-007 is rejected, cancelled, or merged into another job, **the number PJ-007 is retired with its entry**; no future job ever reuses it.
> - **Renumbering is forbidden.** When jobs close, are rejected, or split, their numbers stay where they are. New jobs always take the **next unused number** (PJ-028, PJ-029, …), regardless of what's happened to earlier ones.
> - **Splitting preserves the parent number with sibling suffixes.** If PJ-006 splits into PJ-006a and PJ-006b, the parent PJ-006 stays in the doc as a header pointing at its children. PJ-006 itself never disappears.
> - **Merging redirects.** If PJ-008 + PJ-009 + PJ-010 merge into one MIG, all three numbers stay in the doc; two of them point at the surviving entry with `merged into PJ-NNN`.
> - **Why this matters.** Session logs cite jobs by number. Commit messages cite jobs by number. The Pending Jobs doc itself cross-references by number (e.g. PJ-006 depends on PJ-007). If numbers got reused, every historical reference would silently break.
>
> ### Status vocabulary
>
> | Status | Meaning | Number behavior |
> |---|---|---|
> | **Open** | Not yet started; queued for future work | active |
> | **In-Progress** | Work has started; tracked in current session log | active |
> | **Confirmed** | Boss has decided the design / scope; ready to start | active |
> | **Blocked** | Cannot proceed until a dependency lands | active |
> | **Deferred** | Decided not to ship now; revisit later | active |
> | **On-hold** | Conditional on a future trigger (e.g. user feedback) | active |
> | **Done** | Shipped; commit hash recorded | retired (number reserved) |
> | **Rejected** | Boss decided not to do this | retired (number reserved) |
> | **Cancelled** | Started but abandoned; entry kept for the record | retired (number reserved) |
> | **Merged** | Folded into another job; the surviving job's number is referenced | retired (number reserved, points to survivor) |
>
> All terminal statuses (Done, Rejected, Cancelled, Merged) move the entry to **§7 Done** but the number stays referenced from its original spot via a one-line stub if useful for navigation.

---

## Quick reference — top of the queue

The first three rows are what's queued to start *next*; the rest are sequenced by priority within their category.

| ID | Job | Status | Severity | Effort |
|---|---|---|---|---|
| PJ-007 | Note-stage taxonomy: Living Link 6-stage baseline + extensible | **Confirmed · Ready to start** | P1 | Single focused MIG |
| PJ-006 | Living Link Architecture P2–P5 implementation | **Open** (PJ-007 dependency unblocked) | P1 | Multi-MIG |
| PJ-001 | MIG-013 P1-M1 — chunk the v2 sentinel migration | Open | P1 | Mini-MIG |

---

## §1 · Mini-MIG candidates (focused, 1–3 days each)

### PJ-001 — MIG-013 P1-M1: chunk the v2 sentinel migration with progress UI

**Status.** Open · **Severity.** P1 · **Effort.** Mini-MIG

The MIG-013 §1E audit (`lab/reports/MIG-013-CTSE-AUDIT.md` §3) found that the v2 bigram-sentinel migration's bulk UPDATE blocks boot for 30–90 sec on pre-MIG-013 DBs (~5.7M bigram rows) with zero user feedback. Boss's library has already migrated, but new pre-MIG-013 backups would hit it once.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §3, `project_mig013_v2_migration_blocking_boot.md`.

**Acceptance.** On a 5.7M-row pre-MIG-013 DB, boot proceeds to first paint within 5 seconds. A status-bar strip shows `Migrating term index — N / M` and disappears 4 seconds after `done`. Pre-UPDATE `diag_log` records the matching row count for diagnostics even without UI.

---

### PJ-002 — Pre-§140 `cid_cn` collision scrub utility

**Status.** Open · **Severity.** P1 · **Effort.** Mini-MIG

One-time fix for libraries with corrupted `cid_cn` values from before MIG-003 §140's hardening. Boss self-healed his own affected note (Hub v6) by delete + recreate. Users restoring old backups would still hit the collision silently.

**Source.** Orientation v1.30, session log 2026-05-03.

**Acceptance.** A boot-time scan walks all `note_meta` rows; any duplicate `cid_cn` triggers a status-bar prompt offering "Re-canonicalize duplicates" with a preview list. Run is opt-in, idempotent, and logs every change to a session-scoped report.

---

### PJ-003 — Rename-collision popup (Override / Rename / Cancel)

**Status.** Open · **Severity.** P1 · **Effort.** Mini-MIG

Today `create_note` / `rename_item` silently refuse a rename when the target filename already exists. Boss expects a system-style dialog with three actions: Override the existing file, Rename to something else, or Cancel.

**Source.** `project_rename_collision_popup_wanted.md` (logged 2026-04-28 from MIG-003 Stage 5 test).

**Acceptance.** When `rename_item` hits a collision, frontend shows a `ConfirmDialog`-style popup with three buttons. Override copies properties from the renamed file before deleting the existing one (preserves `cid_cn`). Rename re-prompts for a new name. Cancel dismisses. Localized in en + ar; placeholders in 13 others.

---

### PJ-004 — NSIS bundling lock workaround

**Status.** Open · **Severity.** P2 · **Effort.** Investigation + small fix

Recurring `os error 32` when Constellation is running during `npm run tauri build`. The NSIS bundle stage tries to write `Constellation_X.Y.Z_x64-setup.exe` into a directory that the running binary holds open. MSI succeeds; NSIS doesn't. We work around by using the MSI; a real fix would let CI bundle reliably.

**Source.** Orientation v1.30, hit again on every build during MIG-013.

**Acceptance.** `npm run tauri build` produces both MSI and NSIS bundles cleanly even when an old binary is running. Likely fix: change the NSIS output path or add a kill-running-instance pre-build hook.

---

## §2 · Larger MIG candidates

### PJ-005 — MIG-007: Links Settings tab

**Status.** Open · **Severity.** P1 · **Effort.** Single MIG

Consolidate every link-related Settings control into one "Links" tab. Currently the controls are scattered: Auto-update Links toggle is misplaced under Sky View & Links (per `project_autoupdatelinks_toggle_placement.md`); the link-confidence backfill button lives in a generic Maintenance section; Living Link lifecycle preferences (when introduced) need a home.

**Source.** `project_links_settings_tab.md`, `project_autoupdatelinks_toggle_placement.md`.

**Acceptance.** A new `Links` tab in Settings that aggregates: Auto-update Links toggle, link-confidence backfill button, Living Link lifecycle decay rate (if exposed), typed-link visibility preferences, link-archival display toggle. Localized in en + ar. Old toggle locations removed without breaking deep-link bookmarks (settings-section anchors stable).

---

### PJ-006 — Living Link Architecture P2–P5 implementation

**Status.** Open · **Severity.** P1 · **Effort.** Multi-MIG

The Living Link Architecture spec (`docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`) defines five implementation phases. P0 (note_links table, `extract_typed_links`, 19,062 links indexed) and P1 (7 cognitive search operators in 15 languages, chips in SearchHub + Sky View) are done. P2 through P5 are sketched but not built:

- **P2 — Traversal tracking.** Increment `last_traversed` + `traversal_count` on every link click. Wire into the click-through paths in NotePane, Sky View, OrgChart, Index, Map.
- **P3 — Weight accumulation + lifecycle stages.** Weight grows logarithmically with traversal, decays 5%/month without use. Lifecycle stages: `spark → birth → growth → maturity → dormancy → renewal/archival`. SQL trigger maintenance + scheduled decay job.
- **P4 — Formulation analysis queries.** Surface dormant links, contested links, orphaned references, "5 Acts of Knowledge Creation" structure analysis.
- **P5 — Knowledge health dashboard.** Visual surface for P4 queries; trends over time; alerts for stagnant areas.

**Dependency.** PJ-007 (note-stage taxonomy decision) should be resolved before P3 — if Boss picks Path C (unify on Living Link lifecycle for both notes and links), the lifecycle vocabulary needs to be consistent across the Note + Link models.

**Source.** `project_ce_philosophy.md`, `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`.

**Acceptance.** Each phase ships as its own MIG (Architect → Plan → Build → Audit). P2 + P3 likely combine into one MIG; P4 + P5 each their own.

---

### PJ-007 — Note-stage taxonomy: Living Link 6-stage baseline + extensible custom stages

**Status.** **Confirmed · Ready to start** · **Severity.** P1 · **Effort.** Single focused MIG

**Boss decision (2026-05-06)**: Notes use the **Living Link 6-stage lifecycle** as the canonical baseline (`spark / birth / growth / maturity / dormancy / archival`). The taxonomy is **closed but user-extensible** — users can add their own stages and Constellation accommodates them. The Zettelkasten 4-stage taxonomy currently in `PropertyEditor.svelte:481-484` is replaced. The note remains liquid: the system tracks usage, surfaces signals, and stays out of the user's way on classification.

**Decision rationale (Boss manifesto, 2026-05-05/06)**: "The note is the heart and mind of Constellation system. It is the Key to Cognition & Knowledge. It is not solid, but flexible and liquid. It can be formed in any way, shape, or method." The closed-baseline-with-extensibility model honors both the Constellation-native vocabulary (Living Link as the system's own framework, already specified in `CONSTELLATION-KNOWLEDGE-FORMULATION.md` §IV and i18n'd in 15 locales via `c95a0e6`) AND the user's right to bring their own method.

**Approved scope**:

| Decision | Approved value |
|---|---|
| Baseline taxonomy | Living Link 6-stage (`spark / birth / growth / maturity / dormancy / archival`) |
| Extensibility | Users can add their own stages; system accommodates them everywhere downstream |
| Where custom stages persist | Per-Universe in `universe.json` — new `custom_stages: string[]` field. Different vocabularies allowed in different Universes. |
| How to add a custom stage | Inline in PropertyEditor — type a new value in the dropdown's free-text mode, hit Enter, joins the user's `custom_stages` list automatically. Plus Settings → Notes → "Manage custom stages" panel for renaming / deleting. |
| i18n | Baseline stages localized via `notePane.stage.<key>` in 15 locales (already shipped in `c95a0e6`). Custom stages shown verbatim — user's own words, user's own language. |
| Emoji | Baseline gets fixed emoji per stage. Custom stages get default `🏷️` and a picker to choose. |
| Dropdown ordering | Baseline first (in canonical lifecycle order), then user's custom in chronological-add order. |
| Migration of Boss's existing `stage: birth` note | No migration needed — `birth` is already a baseline value. |
| Living Link 6-stage on links | Stays as currently specified; this work doesn't change link-side semantics. |

**Implementation plan** (focused MIG, follows the Migration Rule §1–§4):

1. **Architect** — Architect doc enumerating the schema change, PropertyEditor refactor, Settings panel scaffold, i18n confirmation, User Manual section.
2. **Plan** — Phase-by-phase commits with verification clauses; Boss approves before any code edit.
3. **Build** — Implementation per the approved plan; `/simplify` after.
4. **Audit** — Three parallel agents (invariants / drift / migration-path) per Migration Rule §4.

**Dependency chain**:
- Unblocks PJ-006 P3 (Living Link weight + lifecycle on links): the link-side lifecycle work proceeds in parallel; both surfaces share the 6-stage vocabulary.
- No blocker upstream of PJ-007 anymore.

**Source.** `project_note_stage_taxonomy_decision.md`; Boss manifesto 2026-05-05/06; this commit's session log.

**Acceptance.** PropertyEditor renders the Living Link 6-stage baseline + the user's `custom_stages` in a single dropdown with optional inline-add. `universe.json` schema extended with `custom_stages` field (idempotent migration). Settings → Notes → "Manage custom stages" panel ships. User Manual + Index help updated (en + ar; 13 others queued via PJ-014). Boss-tests on real notes confirm round-trip: type custom stage → save → reopen → see it persisted + autocompleting on next note.

---

## §3 · Bug fixes / quality

### PJ-008 — Outgoing Links panel typed-link duplication

**Status.** Open · **Severity.** P2 · **Effort.** Single-file fix

Outgoing Links panel renders typed-link aliases twice — once as a typed-link badge (e.g. `supports`) and once as a plain text row. Same root pattern as PJ-009.

**Source.** `project_outgoing_typedlink_duplication.md`.

**Acceptance.** Each typed-link target appears exactly once in the panel, with its type as a badge.

---

### PJ-009 — Backlinks panel typed-link duplication

**Status.** Open · **Severity.** P2 · **Effort.** Single-file fix

Backlinks panel duplicates source notes when the same source uses both a regular wikilink and a typed wikilink to the same target. Lunch Plan shows twice for Apple Tree Fruit (regular + supports).

**Source.** `project_backlinks_typed_link_duplication.md`.

**Acceptance.** Each source note appears once per target, with all link types accumulated as badges on the row.

---

### PJ-010 — Unlinked Mentions panel double-count + canonical filenames

**Status.** Open · **Severity.** P2 · **Effort.** Small refactor

Two issues bundled: (a) double-counts typed-link references; (b) shows canonical filenames (`20260427T155436Z_NOTE_3a8f.md`) instead of human titles. Pair with PJ-008 / PJ-009 — same de-duplication pattern across all link panels.

**Source.** `project_unlinked_mentions_double_count.md`. There's also a related `project_unlinked_mentions_alias_bleed.md` (frontmatter aliases surfacing as "unlinked mentions") that should compose with this fix.

**Acceptance.** Each unlinked mention row appears once; rows show human titles via the same `read_index_entries` path the Index panel uses.

---

### PJ-011 — Constellation Map open issues

**Status.** Open · **Severity.** P2 · **Effort.** Single MIG

Three issues bundled (logged 2026-04-27):

- Performance / memory leak in the D3 sunburst rendering on large libraries.
- Tooltip shows canonical filename instead of human title (same root as PJ-010).
- Search doesn't highlight matched arcs.

**Source.** `project_constellation_map_backlog.md`.

**Acceptance.** Map renders cleanly on 7,600-note libraries (no leak across navigation). Tooltips show human titles. Search highlights matching arcs with a visible style.

---

### PJ-012 — `LinkLifecycle.fresh` TS error

**Status.** Deferred · **Severity.** P2 · **Effort.** 2-line fix

Pre-existing svelte-check error at `store.ts:2212`: `Property 'fresh' is missing in type '{emerging, established, load-bearing, stale}'`. Option B approved 2026-05-01 but deferred until post-CE: add `fresh: 1`, shift `stale: 0`, `fresh: 1`, `emerging: 2`, `established: 3`, `load-bearing: 4`. Runtime impact silent; this is a type-completeness fix.

**Source.** `project_link_lifecycle_dedupe_fix.md`.

**Acceptance.** `npm run check` produces zero errors (currently shows this one + warnings).

**Composes with.** PJ-006 P3 — LinkLifecycle taxonomy is the same vocab the Living Link lifecycle uses; fix it as part of P3 if not done sooner.

---

### PJ-013 — `lenses::apply_lens` dead-code decision

**Status.** Open · **Severity.** P2 · **Effort.** Decision + small fix

`lenses.rs::apply_lens` is dead code (zero frontend callers, verified 2026-04-27). Settings can still create + save lens definitions but they're never applied. Two paths: delete the function + the orphaned Settings UI, or re-wire it for CE Phase 9 (whatever that turns out to be).

**Source.** `project_lenses_apply_lens_dead_code.md`.

**Acceptance.** Either deleted (along with `list_lenses` / `save_lenses` if those are also unused) and the Settings lens-builder UI removed, or re-wired into a frontend consumer that exercises it.

---

## §4 · Doc backlog

### PJ-014 — 13-locale User Manual backfill

**Status.** Open · **Severity.** P2 · **Effort.** Translation work

Every MIG since 008 has shipped en + ar User Manual updates while deferring the 13 other locales (de / es / fa / fr / he / hi / ja / ko / pt / ru / tr / ur / zh). The queue is now MIGs 008, 010, 011, 012, 013 worth of additions.

**Source.** `project_user_manual_13_locales_backfill.md`.

**Acceptance.** All 13 locale User Manuals receive the deferred sections. Done as one batch translation pass; can be split per locale if Boss has translator capacity for a few at a time.

---

### PJ-015 — 360.3D Stratification Matrix guidance doc

**Status.** Blocked · **Severity.** P2 · **Effort.** Single doc (~2000 words)

Boss-requested teaching doc on how to read / interpret the 360.3D Stratification Matrix. Three reads (Position / Profile / Absence), mental shapes catalogue, matrix → action examples. Modeled after the Index Guidance doc.

**Blocked on:** 360.3D Stage 3 closing + matrix UX stabilizing — writing the guidance before the UI settles produces stale doc.

**Source.** `project_360_3d_matrix_guidance_doc.md`.

**Acceptance.** New doc at `docs/help.uConstellation.World/360.3D/Stratification Matrix Guidance.md`. Translated to ar (en first, others queued via PJ-014 backfill pattern).

---

## §5 · Cleanup / hygiene

### PJ-016 — Drop `term_vocab.bridge_concept_id` column

**Status.** Open · **Severity.** P2 · **Effort.** Schema migration

Dead schema after MIG-013 §1D Option B. Nothing reads or writes the column on the live CTSE path. Forward-compat preserved deliberately, but a future cleanup migration can drop it (along with the v1 / v2 schema gates and the `sentinel_bigram_rows` helper).

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2.

**Acceptance.** Schema v3 migration drops the column, the index, and all the dead helper code. M11 zero-diff invariant still holds.

**Defer.** Wait at least 2–3 sessions after MIG-013 close to confirm nothing reactivates the column.

---

### PJ-017 — Drop orphaned `term_embeddings` table on existing DBs

**Status.** Open · **Severity.** P2 · **Effort.** Schema migration

Leftover from MIG-012 on pre-MIG-013 universes. Tens of MB of dead disk per Universe. No correctness issue.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §3 Notes.

**Acceptance.** A schema migration runs `DROP TABLE IF EXISTS term_embeddings`. Bundle with PJ-016 into a single cleanup migration.

---

### PJ-018 — Drop `index.semanticSearchEnabled` settings flag

**Status.** Open · **Severity.** P2 · **Effort.** 2-line fix

Kept for forward-compat after §1D-B; zero readers in `src/`. Bundle with the rest of the MIG-013 cleanup.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2 Notes.

**Acceptance.** Removed from `DEFAULT_SETTINGS` and the `IndexSettings` interface in `store.ts`.

---

### PJ-019 — Drop `searchHub.concept` / `searchBadges.concept` i18n keys

**Status.** Open · **Severity.** P2 · **Effort.** 2-key delete × 15 locales

Kept after the SearchHub `concept` category was reverted in §1D-D; zero callers.

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §2 Notes.

**Acceptance.** Both keys removed from all 15 locale JSONs. Bundle with PJ-016 / PJ-017 / PJ-018 into one MIG-013 cleanup commit.

---

### PJ-020 — Optional `≈ similar` kill-switch

**Status.** On-hold · **Severity.** P3 · **Effort.** Settings toggle + gating

The CTSE `≈ similar` feature is currently always-on with no Settings toggle. Add only if Boss reports noise (irrelevant terms surfacing in the Index dropdown).

**Source.** MIG-013 close-out note.

**Acceptance.** Add a toggle to Settings → Index ("Cross-language `≈ similar` matches"). Default ON. When OFF, the `ctseSearchTermsByConcept` effect short-circuits.

**On-hold.** No action until Boss reports the feature is producing noise.

---

## §6 · Standing-Order audit — Write-Time Derivation (CLAUDE.md Rule 8)

CLAUDE.md Rule 8 ("Every computed view in Constellation is maintained at write time, not read time") explicitly names these surfaces as needing audit. Each currently rebuilds at boot or on tab focus instead of being maintained at write-time via triggers/hooks. Each is its own focused MIG.

### PJ-021 — Sky View (`skyNodes` / `skyLinks`)

**Status.** Open · **Severity.** P1 · **Effort.** Single MIG (could be the next CE phase)

`skyNodes` and `skyLinks` are rebuilt on every boot. Should be persisted in SQLite with triggers that maintain them on every `note_meta` / `note_links` change.

**Acceptance.** New schema + triggers; boot reads `skyNodes` directly without rebuild. Cold-start time on 7,600-note library drops measurably.

---

### PJ-022 — Backlinks panel

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Currently recomputed on tab focus by walking `note_links` per-target. Should be a cached table or a materialized view.

**Acceptance.** Tab focus shows backlinks instantly even on 100-link nodes.

---

### PJ-023 — Outgoing Links panel

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Currently recomputed on tab focus. Same shape as PJ-022; pair into one MIG.

**Acceptance.** Same as PJ-022.

---

### PJ-024 — Tag browser

**Status.** Open · **Severity.** P2 · **Effort.** Cache layer

Scanned on open. Should read from a maintained tag→notes index.

**Acceptance.** Tag browser opens instantly on libraries of any size.

---

### PJ-025 — Sight dashboard

**Status.** Open · **Severity.** P2 · **Effort.** Verify

Status unverified. Claude needs to read the Sight dashboard read path and confirm whether it's already maintained at write-time or needs the audit.

**Acceptance.** Either confirmed write-time-derived (close as Done) or migrated.

---

### PJ-026 — Sidebar star counts

**Status.** Open · **Severity.** P2 · **Effort.** Verify

Status unverified. Same shape as PJ-025.

---

### PJ-027 — Map

**Status.** Open · **Severity.** P2 · **Effort.** Verify

Server-side `map.rs` is the data source. Verify the cache path (likely already write-time-derived; pair with PJ-011 perf investigation).

---

## §7 · Done

(Empty — items move here from the categories above when they close. Format: `~~PJ-NNN — title~~ · closed YYYY-MM-DD · commit hash`.)

---

## Appendix — How to amend this document

1. **Adding a job.** Append to the appropriate `§N` section with the **next unused** `PJ-NNN` ID — never reuse a number from a Done / Rejected / Cancelled / Merged entry. Bump the version. Commit + push as a new file (`v1.2.md` etc.).
2. **Updating a job.** Edit the existing entry (status, severity, source, acceptance). Bump the version if the change is structural (status transition, scope change); same version if just refining wording.
3. **Closing a job.** Move the entry to `§7 Done` keeping its `PJ-NNN`, strikethrough the title, append closing date and commit hash. Bump the version. The number is retired with the entry — never recycled.
4. **Rejecting / cancelling a job.** Same shape as closing — move to `§7 Done` with status `Rejected` or `Cancelled`, keep the number, record the date and reason. Number retired.
5. **Splitting a job.** Keep the parent ID. Add `PJ-NNNa`, `PJ-NNNb`, etc. as siblings. The parent entry stays as a header pointing at the children. Cross-reference in both directions.
6. **Merging jobs.** All merged numbers stay in the doc. Survivor keeps its number; merged entries point at the survivor with `merged into PJ-NNN`. All merged-in numbers are retired.
7. **Renumbering. Strictly forbidden.** PJ-NNN is a permanent reference identity. Session logs, commits, and other docs cite jobs by number. Renumbering would silently break every historical reference.
8. **Filename convention.** Same as orientation + Laws docs + NotePane Specs. New version = new file alongside the previous. Older versions stay as historical record.

---

## Appendix — Cross-references

This doc is read alongside:

- `CLAUDE.md` — operational rules. Several jobs cite specific top-principals or rules.
- `docs/Constellation Orientation & Onboarding v1.40.md` (current) — project's operating state, with predecessor versions back to v1.0. Jobs cite orientation versions where they were first surfaced.
- `docs/Constellation Development Laws v1.1.md` (current) — higher-order law statements. Jobs are the *surfaces* the Laws operate on.
- `lab/reports/SESSION-LOG-YYYY-MM-DD.md` — daily engineering record. Jobs are surfaced into a session log entry when work begins; entry ID + commit hash flow back into Done.
- `lab/reports/MIG-NNN-*.md` — Architect / Plan / Audit docs for each MIG. Several jobs cite specific audit findings.
- Auto-memory at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\` — every `project_*.md` file is a candidate source for a Pending Job entry.

---

**End of v1.0.** Twenty-seven jobs across six categories. Boss has indicated PJ-006 + PJ-007 are next; the cascade for those starts after this doc commits.
