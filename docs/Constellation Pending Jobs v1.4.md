# Constellation Pending Jobs

**Version 1.4 | 2026-05-07**

> **What changed in v1.4** (Boss-directed 2026-05-07; closes the MIG-016 cycle and frames the Sight v3 trajectory):
>
> **Closed in v1.4:**
> - **PJ-034** (MIG-016: Sight instant-toggle perf) — **Cancelled (partial-shipped)**. §1A instrumentation + §1B edges-on-hover gate shipped (commits `a0babbb` → `7e76b17` → `62718f7`). §1C (Web Worker offload), §1D (post-paint prewarm), §1E (SQLite `sight_cache`) **abandoned mid-flight** because v2 Sight is being disabled under MIG-017 (PJ-039) as a known-good fallback while v3 is built fresh. Original goal — "instant first-toggle on a 30k-edge universe" — not met for v2; designed-in for v3 from the start. Audit close-out at `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`.
>
> **New PJs allocated:**
> - **PJ-035** — Sight content-similarity TF-IDF edges (the InfraNodus-defining mechanic; not in v2; **inheritable into v3**).
> - **PJ-036** — Sight layer peeling (hide top-N centrality nodes and recompute; not in v2; **inheritable into v3**).
> - **PJ-037** — Map ↔ Sight integration (cross-surface filtering and selection sync; **inheritable into v3**).
> - **PJ-038** — **Sight v3 build with own dedicated Concept Paper.** Multi-MIG. Star-chart aesthetic per Boss's design north star (Suwaidi northern-hemisphere chart reference). Inherits the Rust analytics from v2; rebuilds the visualization layer entirely.
> - **PJ-039** — **MIG-017: Disable v2 Sight.** Mini-MIG, single session. Hides v2 Sight's user-visible surface (dock button, modal, Settings entry) while preserving the v2 Svelte component and IPCs as a known-good fallback. Precondition for PJ-038.
>
> **Top of queue rotates** (PJ-039 + PJ-038 are the current Sight track; PJ-005 / PJ-002 / PJ-008 carry over from v1.3 as the non-Sight queue):
> 1. PJ-039 (MIG-017) — disable v2 Sight (next-up, mini-MIG)
> 2. PJ-038 — Sight v3 build with own Concept Paper (after MIG-017)
> 3. PJ-005 — MIG-007: Links Settings tab
> 4. PJ-002 — cid_cn collision scrub
> 5. PJ-008 — Outgoing Links typed-link dedupe
>
> **Done count after v1.4**: 6 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027). **Cancelled count**: 1 (PJ-034 — partial-shipped).
>
> **New papers landed alongside this version**:
> - `docs/Constellation-Sight-Concept-Paper-v1.1.md` — markdown port of Eisa's April 2026 v1.0 PDF, refreshed with truth-status, Principle 6, and v3 forward-look.
> - `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` — scope-narrowed audit for the partial-shipped MIG-016.

**Version 1.3 | 2026-05-06**

> **What changed in v1.3** (same day as v1.2; deeper cross-check after PJ-006 catch): the v1.2 cross-check agent missed **PJ-006 — Living Link Architecture P2–P5** because my own instructions told it to read only the "What changed in vX.Y" preambles, not orientation BODIES. Orientation v1.40 §4.4 has been titled *"The Living Link Architecture (P0–P5 all shipped + user-validated)"* since 2026-05-05. PJ-006 was already done.
>
> **Eisa's response**: codified as **Standing Order #8** in CLAUDE.md (also memory feedback note `feedback_pj_crosscheck_before_tackle.md`): cross-check any PJ before tackling — read orientation BODIES (§4.x subsystem sections) and session logs, not just preambles. Then re-ran the cross-check correctly.
>
> **Outcome of the deeper cross-check** (orientation v1.49 bodies + session logs 2026-05-01 → 2026-05-06):
> - **1 entry flipped to SHIPPED**: PJ-006 (Living Link P2–P5).
> - **All 27 other entries confirmed unchanged** from v1.2 status. No further stale entries.
> - **No new PJ-NNN allocations needed**.
> - **Scope rewrites already correct** in v1.2 (PJ-010, PJ-014, PJ-021 stay as written).
>
> **Top of queue rotates**: PJ-005 (Links Settings tab) → PJ-002 (cid_cn collision scrub) → PJ-008 (Outgoing Links dedupe).
>
> **Done count after v1.3**: 6 (PJ-001, PJ-006, PJ-007, PJ-025, PJ-026, PJ-027).

**Version 1.2 | 2026-05-06**

> **What changed in v1.2** (same day as v1.1; cross-check audit + v2 batch close): Eisa-directed cross-check of v1.1 against orientation v1.0 → v1.47 to verify which jobs are still applicable.
>
> **Closed in v1.2 cycle:**
> - **PJ-001** (chunk v2 sentinel migration) — SHIPPED via MIG-015 (commits `0ca7e64` → `877e46e`).
> - **PJ-007** (Note-stage taxonomy) — SHIPPED via MIG-014 (commits `c3b9454` → `339d65b`).
>
> **Marked OBSOLETE in v1.2** (already addressed; numbers retired per stable-reference-numbers rule):
> - **PJ-025** (Sight dashboard) — Sight is on-demand via `toggleLens()` in `+layout.svelte:3354`, not boot-rebuilt. Cached after first compute. The "rebuilds on every boot" framing was incorrect.
> - **PJ-026** (sidebar star counts) — `loadAllStats` is cache-fast (Rust per-library parallelism, fire-and-forget per `+layout.svelte:1939-1941`). Not a boot-blocking gap.
> - **PJ-027** (Map) — `src-tauri/src/map.rs:300` documents the Map data is "maintained by triggers on note-save". Already write-time derived.
>
> **Scope rewritten in v1.2:**
> - **PJ-010** (Unlinked Mentions) — narrowed to the frontmatter alias-bleed half (the "double-count typed-link" half was already fixed in v1.5 §90's `scan_unlinked_mentions` rewrite that skips ALL wikilink forms before plain-text scanning).
> - **PJ-014** (13-locale User Manual backfill) — reflected that MIG-014 + MIG-015 shipped all 15 locales upfront for their new strings; remaining queue is the User Manual / help-doc body content (Stages section §18.6, Cognitive Engine help, etc.) that needs translation in the 13 non-en/ar locales.
> - **PJ-021** (Sky View persistence) — narrowed: `sky_backfill.rs` and `cache_boot_snapshot_sky` already provide partial persistence; the gap is whether the full Rule 8 audit (write-time triggers on every note_meta / note_links change) is complete. Verify-then-narrow.
>
> **New PJ-028 → PJ-033** allocated to the MIG-014 §2F audit P2/P3 follow-ups (six edge-case items from `project_mig014_audit_p2_p3_followups.md`).

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
> | **Cancelled (partial-shipped)** | Started, some phases shipped, remainder abandoned (added in v1.4) | retired (number reserved) |
> | **Merged** | Folded into another job; the surviving job's number is referenced | retired (number reserved, points to survivor) |
>
> All terminal statuses (Done, Rejected, Cancelled, Merged) move the entry to **§7 Done** but the number stays referenced from its original spot via a one-line stub if useful for navigation.

---

## Quick reference — top of the queue

The first five rows are what's queued to start *next*; the rest are sequenced by priority within their category.

| ID | Job | Status | Severity | Effort |
|---|---|---|---|---|
| PJ-039 | MIG-017: Disable v2 Sight (precondition for v3 build) | **Open** (next-up) | P1 | Mini-MIG (single session) |
| PJ-038 | Sight v3 build with own dedicated Concept Paper | **Confirmed** (after PJ-039) | P1 | Multi-MIG |
| PJ-005 | MIG-007: Links Settings tab | Open | P1 | Single MIG |
| PJ-002 | Pre-§140 `cid_cn` collision scrub utility | Open | P1 | Mini-MIG |
| PJ-008 | Outgoing Links typed-link duplication | Open | P2 (pair w/ PJ-009) | Single-file fix |

---

## §1 · Mini-MIG candidates (focused, 1–3 days each)

### PJ-001 — MIG-013 P1-M1: chunk the v2 sentinel migration with progress UI

**Status.** **SHIPPED 2026-05-06 via MIG-015** · **Severity.** P1 · **Effort.** Mini-MIG (4 phases)

The MIG-013 §1E audit (`lab/reports/MIG-013-CTSE-AUDIT.md` §3) found that the v2 bigram-sentinel migration's bulk UPDATE blocks boot for 30–90 sec on pre-MIG-013 DBs (~5.7M bigram rows) with zero user feedback. Boss's library has already migrated, but new pre-MIG-013 backups would hit it once.

**What shipped (MIG-015 §1A → §1D)**: the migration moved off the boot critical path. `init_db` only detects pending; a worker thread spawned from `ensure_search_db_ready` runs the chunked migration (100,000 rows per chunk) with the DB mutex acquired+dropped per chunk + a 10ms yield between chunks so other IPC callers can interleave. Tauri event channel `migration:term_vocab_v2` emits start/progress/done phases. Frontend `MigrationProgressStrip.svelte` listens and renders a status-bar strip in a new `.sb-center` group: `Migrating term index — N / M`, then `Term index migration complete`, hidden 4 seconds later. i18n covers all 15 locales upfront.

**Acceptance**: ALL met.
- ✅ Boot proceeds to first paint without waiting on the migration.
- ✅ Status-bar strip shows running counts; hides 4 sec after `done`.
- ✅ Crash-recoverable by construction (WHERE clause is the resume marker).
- ✅ 15 locales translated.
- ✅ Three-agent audit clean after one P0 fix (DB mutex held across loop → split per chunk).

**Visual Boss test skipped** per Eisa's directive: Boss's library is already at v2 from earlier MIG-013 testing, and rolling back to manufacture migration work would touch closed-feature production data (Index closed 2026-05-04 per session log; Working Agreement #4 forbids "let's see what happens" on closed-feature data). Static audit verifies behaviour; future users with pre-MIG-013 backups will exercise the visible path naturally.

**Closed-out commit chain**: `0ca7e64` (§1A) → `df0bf87` (§1B) → `62d3b4a` (§1C) → close-out commit (§1D + P0 fix + audit + orientation v1.47).

**Source.** `lab/reports/MIG-013-CTSE-AUDIT.md` §3 (original deferred); `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-ARCHITECT.md`; `lab/reports/PJ-001-CHUNKED-V2-SENTINEL-PLAN.md`; `lab/reports/MIG-015-CHUNKED-V2-SENTINEL-AUDIT.md`.

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

### PJ-039 — MIG-017: Disable v2 Sight (next-up)

**Status.** **Open · next-up** · **Severity.** P1 · **Effort.** Mini-MIG (single session)

Disable v2 Sight (`ConstellationSight2.svelte` + the v2 dock button + the v2 modal + the Settings entry) as a **known-good fallback** while v3 is built fresh under PJ-038. The Rust analytics IPCs (`constellation_sight_centrality`, `constellation_sight_communities`, etc.) and the v2 Svelte component are **kept on disk** — they are the proven baseline if v3 fails.

**Decision context (2026-05-07)**: Eisa's directive — "secure what's achieved, never muddle." Continuing perf work on v2 (the abandoned MIG-016 §1C / §1D / §1E phases) is wasted effort because the visualization layer is being replaced. Disable v2 cleanly, freeze it as a fallback, build v3 from scratch on the star-chart aesthetic.

**Mechanism (proposed)**: feature flag `sight.engine: 'v2-disabled' | 'v2' | 'v3'`, default `'v2-disabled'` for production. Hide:
- The Sight dock-button in the dock toolbar (`+layout.svelte` toggle handler).
- The `{#if lensActive}` mount block for `ConstellationSight2.svelte` (gate it on `sight.engine !== 'v2-disabled'`).
- The "Sight" entry in Settings (gate the same way).

Mark the help docs (`docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md`) as "Sight is being rebuilt — v2 reference preserved below" with a banner.

**Acceptance.**
- v2 Sight is unreachable from the running app's UI in default config.
- The v2 component code, IPCs, and tests stay in the repo.
- A developer can flip the flag to `'v2'` and bring v2 back as-is for diagnostics.
- Help doc shows the "being rebuilt" banner; existing v2-documentation paragraphs preserved beneath.
- Three-agent audit clean (invariants / drift / migration-path).

**Source.** Boss decision 2026-05-07 (logged in `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`).

**Composes with.** PJ-038 — runs immediately after PJ-039 closes.

---

## §2 · Larger MIG candidates

### PJ-005 — MIG-007: Links Settings tab

**Status.** Open · **Severity.** P1 · **Effort.** Single MIG

Consolidate every link-related Settings control into one "Links" tab. Currently the controls are scattered: Auto-update Links toggle is misplaced under Sky View & Links (per `project_autoupdatelinks_toggle_placement.md`); the link-confidence backfill button lives in a generic Maintenance section; Living Link lifecycle preferences (when introduced) need a home.

**Source.** `project_links_settings_tab.md`, `project_autoupdatelinks_toggle_placement.md`.

**Acceptance.** A new `Links` tab in Settings that aggregates: Auto-update Links toggle, link-confidence backfill button, Living Link lifecycle decay rate (if exposed), typed-link visibility preferences, link-archival display toggle. Localized in en + ar. Old toggle locations removed without breaking deep-link bookmarks (settings-section anchors stable).

---

### PJ-006 — Living Link Architecture P2–P5 implementation

**Status.** **SHIPPED (closed in v1.3 cross-check, 2026-05-06)** · **Severity.** P1 · **Effort.** Multi-phase (delivered incrementally)

The Living Link Architecture's five implementation phases (P0 through P5) are all live and user-validated. The v1.2 entry's "Open · Multi-MIG" framing was stale — the work was done in slices over the prior weeks (CE Phase commits in the §90-§142 range), and orientation v1.40 §4.4 has been titled *"The Living Link Architecture (P0–P5 all shipped + user-validated)"* since 2026-05-05.

**What's actually live (verified 2026-05-06 deeper cross-check):**

| Phase | Verified shipping |
|---|---|
| P0 | `note_links` SQLite table, `extract_typed_links`, 19,062 links indexed |
| P1 | 7 cognitive search operators in 15 languages, chips in SearchHub + Sky View |
| **P2 — Traversal tracking** | `constellation_link_traverse` IPC at `src-tauri/src/search.rs:3516`; frontend caller at `src/lib/libraries/store.ts:1094` (called on wikilink click) |
| **P3 — Weight + lifecycle + decay** | `LinkLifecycle` type in `src/lib/libraries/store.ts:1521`; `_link_decay`, `_link_dormant`, `_link_set_confidence`, `_link_backfill_confidence`, `_link_archive`/`_unarchive`/`_archived` IPCs (orientation v1.49 §4.4 line 1190 enumerates them) |
| **P4 — Formulation analysis** | `formulationAnalysis` wrapper at `src/lib/libraries/store.ts:1505` calling `constellation_formulation_analysis` IPC |
| **P5 — Knowledge health dashboard** | `src/lib/components/KnowledgeHealthDashboard.svelte`, mounted at `src/routes/+layout.svelte:5975`. Reads from P2-P4's data via `formulationAnalysis` |

**Decay formula** (display-only): `effectiveWeight = rawWeight × exp(−ln(2) × daysSinceTraversal / halfLifeDays)`. Default half-life: 60 days. (Orientation v1.49 §4.4.)

**Auto-promote on traversal**: confidence escalates `hypothesis → evidence` at traversal_count ≥3, `evidence → established` at ≥10. Manual override via right-click in Link Dashboard. (Orientation v1.49 §4.4.)

**Why this entry was missed in v1.1 / v1.2**: the v1.2 cross-check agent read only orientation "What changed in vX.Y" preambles per my instructions. Orientation §4.4 BODY says "all shipped"; the preamble of any individual version doesn't necessarily restate that. SO #8 codifies the lesson.

**Source.** `project_ce_philosophy.md`, `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md`, orientation v1.40 → v1.49 §4.4 body, `lab/reports/SESSION-LOG-2026-05-*.md`.

**Acceptance.** Already met. Living Link Architecture is the canonical link-side model; PJ-007 (note-side dependency) shipped same-day via MIG-014; the two now share the lifecycle vocabulary cleanly.

---

### PJ-007 — Note-stage taxonomy: Living Link 6-stage baseline + extensible custom stages

**Status.** **SHIPPED 2026-05-06 via MIG-014 (per-note dash-encoded model)** · **Severity.** P1 · **Effort.** Single focused MIG (delivered as a 2-iteration migration: §1A–§1D flat-list iteration record, §2A–§2F shipped model)

**What actually shipped**: the **per-note dash-encoded model** from `Stages-Concept-Paper-v1.2.md` and Plan v4. Iteration 1 (§1A → §1D, flat custom-stage list with per-Universe `custom_stages: Vec<CustomStage>` + 5 IPC commands + emoji picker) was built then proven wrong in Boss test — it didn't scale (long promote chain), the matrix was wrong (Eisa: "It is allowed only one custom term"), and it broke the Single-Source-of-Truth principle (three local mirrors of the stage value drifted across surfaces).

Iteration 2 (§2A → §2F, the model that ships):

- **6 fixed lifecycle stages** form the canonical chain: spark → birth → growth → maturity → dormancy → archival.
- **Per-note custom term** as a dash suffix in the on-disk frontmatter `stage:` value (e.g. `stage: spark-concept`). No Universe-wide setting.
- **PropertyEditor combobox** is a 6-entry mode-flip dropdown: Mode A (input empty / matches a fixed name) → 6 baselines; Mode B (custom word in input or dash suffix) → 6 paired stages (`Spark-Concept`, `Birth-Concept`, …).
- **Breadcrumb promote/demote** walks the 6-baseline chain; suffix carried verbatim across the chain. Single-source-of-truth (Law 2.7) — `currentStage` is `$derived` from the prop, never a local `$state` mirror.
- **No emoji per custom term**: emoji follows the lifecycle phase.
- **Old Zettelkasten values** (`fleeting / literature / permanent / synthesis`) preserved verbatim on disk; render via `LEGACY_ZETTELKASTEN_EMOJI` for back-compat. They aren't promoteable in the new chain.

**Acceptance**: ALL met.
- ✅ Single combobox in Properties (Mode A / Mode B).
- ✅ On-disk frontmatter is the single canonical source — no Universe-level state.
- ✅ Promote/demote chain length always 6.
- ✅ User Manual + Cognitive Engine help updated (en + ar; PJ-014 queues 13 others).
- ✅ Boss tests passed: combobox + per-note scope + cross-track navigation + boundary cases (Spark/Archival).
- ✅ Three-agent audit clean (invariants / drift / migration-path), audit report at `lab/reports/MIG-014-NOTE-STAGE-AUDIT.md`.

**Generalisation produced**: Law 2.7 (Constellation Development Laws v1.4) — every first-class data property has one canonical owner; UI surfaces are subfunctions that derive, never hold their own copy. Triggered by the §2C+§2D stage-sync patch cycle (Eisa: "Enough patching").

**Closed-out commit chain**: `c3b9454` (§1A) → `8a9ab3d` (§1B) → `17bf474` (§1C) → `9973e65` (§1C.5) → `f4eef3e` (§1C.5 fix + §1D) [iteration record] → `2f58b8a` (§2A) → `59ed95c` (§2B) → `432076c` (§2C) → `2c58bda` (§2D) → `bb7a6ef` (§2C+D fix) → `e3a97a1` (Law 2.7 architectural fix) → `a50463c` (§2E) → `339d65b` (§2F closes).

**Source.** Stages Concept Paper v1.0 → v1.2; MIG-014 Plan v1 → v4; MIG-014 §2F audit report.

---

### PJ-034 — MIG-016: Sight instant-toggle perf

**Status.** **Cancelled (partial-shipped) — closed 2026-05-07** · **Severity.** P1 · **Effort.** Was scoped as a 6-phase MIG; closed early after §1B.

**What shipped (§1A + §1B):**
- **§1A — `performance.mark` instrumentation** around `toggleLens()` in `+layout.svelte` and `ConstellationSight2.svelte` mount path. Marks: `sight:rust-centrality`, `sight:louvain`, `sight:structural-gaps`, `sight:universe-health`, `sight:stratum-weighted`, `sight:top-bridges`, `sight:community-profiles`, `sight:bridge-suggestions`, `sight:toggle:total`. Initial alerts/clipboard fallback added then removed in §1B; `console.log` + `performance.mark` retained as no-op-in-production. Commits: `a0babbb` (§1A) + `7e76b17` (§1A clipboard fix).
- **§1B — Edges-on-hover gate** (Principle 6 of the Sight Concept Paper v1.1). `needsEdgeDraw = hoveredNode || selectedNode || searchActive || hoveredLink`; `focusOnly` short-circuit at the top of `drawLinks()` skips non-incident edges in O(1) per link. Drops per-frame edge iteration to zero in the resting case (no hover, no selection, no search). Pre-built `neighborMap: Map<string, Set<string>>` populated once per `buildSimData()` call. Commit: `62718f7`. Boss test: PASSED.

**What was abandoned (§1C / §1D / §1E):**
- **§1C — `sightWorker.ts` extraction** (Louvain + gaps + profiles + bridges off main thread): **Cancelled.** Wasted work on a disabled view.
- **§1D — Post-paint prewarm** (`requestIdleCallback` after first paint to cache results before user toggles): **Cancelled.** Same reason.
- **§1E — SQLite `sight_cache`** (cross-session persistence, mirroring the `sky_backfill` pattern): **Deferred to PJ-038.** v3 will compute identical analytical outputs (centrality, communities, gaps, health) and benefit from the same cross-session persistence pattern. The design knowledge from MIG-016 Plan v1 carries forward.

**Why it closes early**: Eisa's directive 2026-05-07 — "secure what's achieved, never muddle." v2 Sight is being disabled under PJ-039 (MIG-017) as a known-good fallback while v3 is built fresh under PJ-038. Continuing perf work on a view that's about to be shelved is wasted effort — except for §1E's design knowledge, which transfers to v3.

**Original goal**: "instant first-toggle on a 30k-edge universe." **Met for v2?** No (the §1A data showed mount is fast at 175-367 ms, but the toggle pipeline's compute is what would have been targeted by §1C/§1D — not addressed). **Designed-in for v3?** Yes — the star-chart aesthetic (Sight Concept Paper v1.1 §13) makes Principle 6 (reveal-on-demand) the visual default, not an add-on.

**Audit close-out**: `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` (scope-narrowed). 0 P0, 0 P1, 1 P3 logged (mousemove handler iterating `simLinks` for link-annotation hover detection; moot once v2 disabled under PJ-039).

**Source.** `lab/reports/MIG-016-ARCHITECT.md`, `lab/reports/MIG-016-PLAN-v1.md`, `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`.

**Inheritance into v3 (PJ-038)**:
- §1B's reveal-on-demand pattern → v3's constellation-line idiom (lines render only inside the focused constellation territory).
- §1E's SQLite cache design → v3's projection-position cache (the projection math is deterministic per-universe-snapshot — caching is a clean win).

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

### PJ-010 — Unlinked Mentions panel: frontmatter alias bleed (scope rewritten in v1.2)

**Status.** Open · **Severity.** P2 · **Effort.** Small refactor

**Scope rewrite — v1.2 (2026-05-06)**: the original v1.1 description bundled two issues — (a) double-counts typed-link references, and (b) canonical filenames instead of human titles. Cross-check against orientation v1.5 §90 confirmed both were fixed by the `scan_unlinked_mentions` rewrite in commit `5cf779a` ("§90 — BUG-005 fix: autosave writeNote bypassed constellation_search_reindex" cycle). That rewrite skips ALL wikilink forms (regular + embed + typed + aliased) before plain-text scanning, AND uses the canonical-filename helper for human-title display. So both v1.1 bullets are stale.

**Remaining genuine gap**: frontmatter aliases (e.g. `aliases: [Foo, Bar]`) still surface as "unlinked mentions" because the alias-bleed fix never landed. A note with `aliases: [Bar]` in its frontmatter and a body that says "Bar" appears in the Unlinked Mentions panel as a separate row, even though the alias on the SAME note's frontmatter is what's matching. Memory note `project_unlinked_mentions_alias_bleed.md` (2026-04-29) describes the case.

**Source.** `project_unlinked_mentions_alias_bleed.md`.

**Acceptance.** A note's frontmatter aliases do NOT surface as "unlinked mentions" against the same note's body (the body word should be parsed as a self-alias-match and excluded). All other panel behaviour stays as v1.5 §90 made it.

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

**Composes with.** PJ-039 (MIG-017): may resolve as part of disabling v2 Sight if the orphaned Settings UI is removed at the same time.

---

## §4 · Doc backlog

### PJ-014 — 13-locale User Manual backfill (scope updated in v1.2)

**Status.** Open · **Severity.** P2 · **Effort.** Translation work

**Scope update — v1.2 (2026-05-06)**: MIG-014 + MIG-015 broke the "en+ar first, others queued" pattern by shipping all 15 locales upfront for their *new* string keys (Eisa-directed). So the i18n .json files don't lag for those two MIGs.

**Remaining queue is the User Manual / help-doc body content** that DOES lag in 13 locales:

- **MIG-014 §2E**: Stages model rewrite in `docs/User Manual.md` §18.6 (en) + `docs/help.ar/User Manual.md` §18.6 (ar) + `docs/help.uConstellation.World/Cognitive Engine/Cognitive Engine.md` Feature 6 (en). 13 other locale User Manuals still carry the old Externalization-Engine 4-stage description.
- **Older deferrals** still queued: MIGs 008, 010, 011, 012, 013 deferred sections.

**Source.** `project_user_manual_13_locales_backfill.md`.

**Acceptance.** All 13 locale User Manuals receive the deferred sections (Stages model + earlier-MIG deferrals). Done as one batch translation pass; can be split per locale if Boss has translator capacity for a few at a time.

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

### PJ-021 — Sky View (`skyNodes` / `skyLinks`) — scope updated in v1.2

**Status.** Open (verify-then-narrow) · **Severity.** P2 · **Effort.** Verify + targeted MIG

**Scope update — v1.2 (2026-05-06)**: cross-check found that `src-tauri/src/sky_backfill.rs` and `cache.rs::cache_boot_snapshot_sky` already provide partial persistence of `sky_nodes` / `sky_links` (since MIG-001 §136 / §142 timeframe). The original v1.1 description ("rebuilt on every boot") is partly outdated.

**Verify-then-narrow plan**:
1. Open `src-tauri/src/sky_backfill.rs` + `cache.rs::cache_boot_snapshot_sky` and confirm what's already persisted vs. what still rebuilds.
2. If full Rule 8 (write-time triggers on every `note_meta` / `note_links` change) is in place: close as Done.
3. If partial: narrow this PJ to "the gap that remains" (likely a specific trigger that's missing or an edge-case where the cache invalidates wholesale instead of incrementally).

**Acceptance.** Either confirmed Rule 8-clean (close) or narrowed to a specific bounded gap with new acceptance criteria.

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

### PJ-025 — Sight dashboard (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: Sight is invoked from the frontend's `toggleLens()` handler (`+layout.svelte:3354` calls `constellation_sight_centrality`). It runs on-demand when the user toggles the Sight overlay; results are cached and reused while fresh. Boot path does NOT invoke Sight. The "rebuilds on every boot" framing in the original PJ description was incorrect — there was no boot-time Sight rebuild to migrate away from.

The PJ-025 number is retired per the stable-reference-numbers rule. No further action.

---

### PJ-026 — Sidebar star counts (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: `loadAllStats` walks the per-library star count at boot, but per `+layout.svelte:1939-1941` comment: *"loadAllStats remains because its Rust side is already cache-fast (metadata-only walk + per-library thread parallelism). It's fire-and-forget so the sidebar star counts populate without blocking anything."* This is a cached-fast path, not a Rule-8 violation. Not boot-blocking, not full filesystem walk. Original PJ-026 framing was incorrect.

PJ-026 number retired.

---

### PJ-027 — Map (OBSOLETE in v1.2)

**Status.** **Obsolete (closed 2026-05-06)** · **Severity.** P2 · **Effort.** Verify

**Verified obsolete (v1.2 cross-check)**: `src-tauri/src/map.rs:300` documents that the Map data is *"maintained by triggers on note-save, so even an explicit open is..."* — already write-time derived per Rule 8. No Rebuild on open / boot.

(PJ-011's separate Map open issues — perf/leak, tooltip showing canonical filename, search-highlight — remain open as P2; those are panel UX bugs unrelated to the persistence question.)

PJ-027 number retired.

---

## §7 · MIG-014 §2F audit follow-ups (PJ-028 → PJ-033 — carried from v1.2)

Six edge-case items found by the MIG-014 §2F three-agent audit (2026-05-06) but logged for later, not blocking close. Memory note: `project_mig014_audit_p2_p3_followups.md`. All non-blocking — graceful degradation in every case. P2/P3 severity.

### PJ-028 — `splitStage` and a leading dash

**Status.** Open · **Severity.** P2 · **Effort.** 2-line fix

`stage: -concept` (no lifecycle prefix) splits to `lifecycle=''`, `suffix='concept'`. Renders as `-Concept` with empty emoji and no promote/demote arrows.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 1; MIG-014 §2F audit.

**Acceptance.** `splitStage` treats empty lifecycle as "no stage" — returns both empty when no valid lifecycle prefix is present. Edge-case display normalizes cleanly.

---

### PJ-029 — Concept Paper §6.1 vs `commitStage` multi-dash drift

**Status.** Open · **Severity.** P2 · **Effort.** Decision + 2-line code fix OR doc fix

Stages Concept Paper v1.2 §6.1 says suffix may not contain `-`. `commitStage` at `PropertyEditor.svelte:199` doesn't enforce. Multi-dash values like `stage: spark-foo-bar` are accepted. Either tighten code (reject) or update doc (allow). Doc-vs-code drift; not a runtime bug.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 2.

**Acceptance.** Doc and code agree on whether multi-dash suffixes are allowed. Whichever direction, the surviving rule is enforced by tests.

---

### PJ-030 — Stale `custom_stages: [...]` from §1A-era testing in `universe.json`

**Status.** Deferred · **Severity.** P3 · **Effort.** None (acceptable)

Serde silently ignores the unknown field; gone on next read-modify-write cycle of `universe.json`. Affects only Boss-equivalent dev-build users (none reported). Acceptable graceful self-healing.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 3.

**Acceptance.** Self-heals on next universe-meta write. No active fix needed unless surfaces as a bug.

---

### PJ-031 — Trailing-dash on disk (`stage: spark-`)

**Status.** Deferred · **Severity.** P3 · **Effort.** None (acceptable)

`splitStage` returns `suffix=''`; nextStage carries no suffix. Display correct. The trailing dash stays on disk verbatim until the user re-commits via promote. Acceptable graceful self-healing.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 4.

**Acceptance.** Self-heals on next promote/demote. No active fix needed.

---

### PJ-032 — Uppercase on disk (`stage: SPARK-CONCEPT`)

**Status.** Deferred · **Severity.** P3 · **Effort.** Optional 2-line fix

`LIVING_LINK_BASELINE.findIndex` is case-sensitive → returns -1. No emoji, no arrows. Display falls back to verbatim render. User must re-pick to recover. Could be normalized in `splitStage` (lowercase the lifecycle component) but the current behavior is acceptable graceful degradation.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 5.

**Acceptance.** Either lowercase normalization in `splitStage`, or status quo if Boss prefers strict canonical form.

---

### PJ-033 — NotePane stage badge `<span>` has no `dir="auto"`

**Status.** Open · **Severity.** P3 · **Effort.** 1-line fix

`src/lib/components/NotePane.svelte:951`. A long Arabic suffix in an LTR UI may render slightly off due to Chromium's bidi defaults. Easy polish — add `dir="auto"` to the badge span.

**Source.** `project_mig014_audit_p2_p3_followups.md` item 6.

**Acceptance.** `dir="auto"` on the stage-badge `<span>`. Mixed-script suffixes render with proper directionality in any UI direction.

---

## §8 · Sight v3 trajectory (NEW in v1.4 — PJ-035 → PJ-038)

The Sight Concept Paper v1.1 (`docs/Constellation-Sight-Concept-Paper-v1.1.md`) §12 truth-status matrix surfaced three implementation gaps in v2 Sight relative to the paper's analytical promise. Each is now an inheritable PJ.

### PJ-035 — Sight content-similarity TF-IDF edges

**Status.** Open · **Severity.** P2 · **Effort.** Multi-step (vector compute + cache + integration)

**The InfraNodus-defining mechanic.** v2 Sight wires explicit-wikilink edges (weight 1.0) and shared-tag edges (weight 0.6) into its graph build. The third edge type from the Concept Paper §3.3 — **content similarity (weight 0.3, TF-IDF cosine)** — is not implemented. This is the mechanic that lets Sight surface *latent* connections — notes that talk about the same topic without being explicitly linked. Without it, Sight cannot detect structural gaps that span un-linked-but-related clusters.

**Source.** `docs/Constellation-Sight-Concept-Paper-v1.1.md` §3.3 + §12 truth-status row.

**Acceptance.** TF-IDF vectors computed per note (incremental on changed notes only, cached). Cosine similarity computed lazily between notes above a configurable threshold. Edges merged into the Sight graph build with weight 0.3 (toggleable in Settings). For non-English content, configurable per-language stemmer pipeline (Arabic via ISRIStemmer or equivalent). Graceful degradation if NLP for a given language isn't loaded — tag/link edges still work.

**Inheritable into v3 (PJ-038)**: in v3's star-chart aesthetic, content-similarity edges become the **Milky Way band** (a diffuse density wash, not extra edge lines competing with constellation connectors). The compute layer is identical; only the visualization changes.

---

### PJ-036 — Sight layer peeling

**Status.** Open · **Severity.** P3 · **Effort.** Single feature (compute + UI toggle)

**The "remove the obvious to reveal the subtle" mechanic.** v2 Sight does not implement the Concept Paper §2.2 mechanic 5 — *layer peeling* — where the user temporarily hides the top-N centrality nodes (typically MOC / index notes) and the analytics recompute on the residual graph. This reveals secondary structure beneath the dominant nodes.

**Source.** `docs/Constellation-Sight-Concept-Paper-v1.1.md` §2.2 + §3.2 + §12 truth-status row.

**Acceptance.** A "Peel layer" toggle in Sight's sidebar. When activated, hide top-N nodes (N user-configurable, default 10) by centrality and re-run Brandes / Louvain / structural-gaps / universe-health on the residual graph. Secondary clusters and bridges become visible; user can iterate (peel again to reveal a third layer).

**Inheritable into v3 (PJ-038)**: in v3's star-chart aesthetic, layer peeling becomes a **"hide brightest stars"** toggle — visually obvious instead of buried in a menu. Same compute, cleaner UX.

---

### PJ-037 — Map ↔ Sight integration

**Status.** Open · **Severity.** P2 · **Effort.** Single MIG (cross-surface wiring)

**The "Map diagnoses, Sight prescribes" loop.** Concept Paper §7 frames Map and Sight as complementary tools that should inform each other:
- Click a Map segment → Sight opens filtered to the notes in that knowledge branch.
- Sight's selected community / bridge node visualized on the Map (which segment does this cluster live in?).

Today the two surfaces are independent — clicking a Map segment does *not* filter Sight, and Sight selections are not visualized on the Map.

**Source.** `docs/Constellation-Sight-Concept-Paper-v1.1.md` §7 + §12 truth-status row.

**Acceptance.** Map → Sight: clicking a Map segment passes that segment's note-set as a filter into the Sight overlay; Sight computes its analytics scoped to that subset. Sight → Map: when Sight selects a community or bridge node, the corresponding segment(s) on the Map highlight. Bidirectional selection cursor.

**Inheritable into v3 (PJ-038)**: the cleanest implementation in v3 is a **two-up panel** where the Map (sunburst) and Sight (sky chart) share a selection cursor — the same selection state drives both views simultaneously.

---

### PJ-038 — Sight v3 build with own dedicated Concept Paper

**Status.** **Confirmed** (after PJ-039 closes) · **Severity.** P1 · **Effort.** Multi-MIG

**Decision** (Eisa, 2026-05-07): rebuild Sight from scratch on the **star-chart aesthetic** — a 2D polar projection where star magnitude maps to centrality, constellation territories map to Louvain communities, the Milky Way band maps to content-similarity density, and a calendar rim maps to time. Reference image: 19th-century-style northern-hemisphere chart (Suwaidi reference; sample owned by the Boss). This is the design north star articulated in `docs/Constellation-Sight-Concept-Paper-v1.1.md` §13.

**What v3 inherits from v2** (the analytical pipeline is preserved as-is):
- Brandes' betweenness centrality (`constellation_sight_centrality` IPC).
- Louvain community detection (`constellation_sight_communities` IPC).
- Structural gap detection (`constellation_sight_structural_gaps` IPC).
- Universe-health metric (`constellation_sight_universe_health` IPC: M + D + E + C).
- Reveal-on-demand (Principle 6) — already shipped in v2's MIG-016 §1B.

**What v3 absorbs (the deferred PJs)**:
- **PJ-035** content-similarity edges → Milky Way band (diffuse density wash).
- **PJ-036** layer peeling → "hide brightest stars" toggle.
- **PJ-037** Map ↔ Sight integration → two-up panel with shared selection cursor.

**What v3 rebuilds entirely** (the visualization layer):
- Force-directed Pixi.js simulation → 2D polar projection (likely Lambert azimuthal equal-area or similar; specified in v3's own Concept Paper).
- D3-style force layout → stable astronomy-style projection math.
- Edge-render hot path → constellation-line idiom (lines render only inside the focused constellation territory).

**Mandatory deliverable: own dedicated Concept Paper** (Boss directive 2026-05-07). The v3 paper is the canonical reference for *what v3 looks like and how it is built*. The v1.1 paper continues as the canonical reference for *what Sight is for* (the analytical foundations both versions share). They are read side-by-side.

**Source.** Boss decision 2026-05-07; Sight Concept Paper v1.1 §13 + §14; MIG-016 audit §6 ("inheritance into v3").

**Acceptance.** v3 ships behind feature flag (`sight.engine: 'v3'`), default ON in production once Boss-test passes. Star-chart projection renders the user's full universe at one glance. Constellation territories visually distinct. Reveal-on-demand for connector lines (Principle 6 baked into the visual grammar). Universe-health metric readable in one glance from dome-balance. Three deferred PJs (PJ-035 / PJ-036 / PJ-037) integrated as the design intends. Three-agent audit clean. Own Concept Paper delivered alongside the build.

**Composes with**:
- PJ-039 (precondition: v2 disabled before v3 starts).
- PJ-035 / PJ-036 / PJ-037 (absorbed as v3 features rather than v2 add-ons).
- PJ-013 (`apply_lens` dead code may be cleaned up as part of the v3 cutover).

---

## §9 · Done

Items move here from the categories above when they close. Format preserved per stable-reference-numbers rule: the original entry stays in its source section with its closure status; this section provides a quick chronological index.

| PJ-NNN | Title | Status | Closed | Commit chain |
|---|---|---|---|---|
| PJ-001 | Chunk the v2 sentinel migration with progress UI | Done | 2026-05-06 (via MIG-015) | `0ca7e64` → `df0bf87` → `62d3b4a` → `877e46e` |
| PJ-006 | Living Link Architecture P2–P5 (all phases verified shipped & user-validated) | Done | 2026-05-06 (closed in v1.3 cross-check; shipped earlier in CE Phase §90-§142 commit range) | (multi-commit; orientation §4.4 line 1167+ for canonical state) |
| PJ-007 | Note-stage taxonomy: per-note dash-encoded model | Done | 2026-05-06 (via MIG-014) | `c3b9454` → `8a9ab3d` → `17bf474` → `9973e65` → `f4eef3e` → `2f58b8a` → `59ed95c` → `432076c` → `2c58bda` → `bb7a6ef` → `e3a97a1` → `a50463c` → `339d65b` |
| PJ-025 | Sight dashboard (Obsolete — verified write-time-derived) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-026 | Sidebar star counts (Obsolete — verified cache-fast) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-027 | Map (Obsolete — verified trigger-maintained) | Obsolete | 2026-05-06 (v1.2 audit) | n/a |
| PJ-034 | MIG-016: Sight instant-toggle perf | **Cancelled (partial-shipped)** | 2026-05-07 (§1A + §1B shipped; §1C/§1D cancelled; §1E deferred to PJ-038) | `a0babbb` → `7e76b17` → `62718f7` |

---

## Appendix — How to amend this document

1. **Adding a job.** Append to the appropriate `§N` section with the **next unused** `PJ-NNN` ID — never reuse a number from a Done / Rejected / Cancelled / Merged entry. Bump the version. Commit + push as a new file (`v1.5.md` etc.).
2. **Updating a job.** Edit the existing entry (status, severity, source, acceptance). Bump the version if the change is structural (status transition, scope change); same version if just refining wording.
3. **Closing a job.** Move the entry to `§9 Done` keeping its `PJ-NNN`, strikethrough the title, append closing date and commit hash. Bump the version. The number is retired with the entry — never recycled.
4. **Rejecting / cancelling a job.** Same shape as closing — move to `§9 Done` with status `Rejected` or `Cancelled` (or `Cancelled (partial-shipped)` if some phases shipped before abandonment), keep the number, record the date and reason. Number retired.
5. **Splitting a job.** Keep the parent ID. Add `PJ-NNNa`, `PJ-NNNb`, etc. as siblings. The parent entry stays as a header pointing at the children. Cross-reference in both directions.
6. **Merging jobs.** All merged numbers stay in the doc. Survivor keeps its number; merged entries point at the survivor with `merged into PJ-NNN`. All merged-in numbers are retired.
7. **Renumbering. Strictly forbidden.** PJ-NNN is a permanent reference identity. Session logs, commits, and other docs cite jobs by number. Renumbering would silently break every historical reference.
8. **Filename convention.** Same as orientation + Laws docs + NotePane Specs. New version = new file alongside the previous. Older versions stay as historical record.

---

## Appendix — Cross-references

This doc is read alongside:

- `CLAUDE.md` — operational rules. Several jobs cite specific top-principals or rules.
- `docs/Constellation Orientation & Onboarding v1.55.md` (current at v1.4) — project's operating state, with predecessor versions back to v1.0. Jobs cite orientation versions where they were first surfaced.
- `docs/Constellation Development Laws v1.4.md` (current) — higher-order law statements. Jobs are the *surfaces* the Laws operate on.
- `docs/Constellation-Sight-Concept-Paper-v1.1.md` (NEW in v1.4) — Sight's analytical foundation. PJ-035 / PJ-036 / PJ-037 / PJ-038 all derive from its §3 / §12 / §13 / §14.
- `lab/reports/SESSION-LOG-YYYY-MM-DD.md` — daily engineering record. Jobs are surfaced into a session log entry when work begins; entry ID + commit hash flow back into Done.
- `lab/reports/MIG-NNN-*.md` — Architect / Plan / Audit docs for each MIG. Several jobs cite specific audit findings.
- `lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md` (NEW in v1.4) — partial-shipped audit closing PJ-034.
- Auto-memory at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\` — every `project_*.md` file is a candidate source for a Pending Job entry.

---

**End of v1.4.** Thirty-two jobs across nine sections. Cancelled-with-partial-ship (PJ-034) closes the MIG-016 cycle; PJ-039 (MIG-017) is the precondition for PJ-038 (Sight v3 build with own Concept Paper). The Sight v3 trajectory is laid out in §8.
