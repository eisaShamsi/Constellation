# MIG-090 — Plan: the All-Notes Base (Notes Navigator restructure)

> **SUPERSEDED (2026-07-05, Boss correction after §2):** *"if I want to have something similar I would use the Bases, the way I wanted. Let's re-write the Navigator concept, based on the original 'Note Navigator'… upgrade it to be smart, fast, and to launch from where others ended."* — Option B revoked. **§1 (the three lens dimensions) KEPT** (engine upgrade for all Bases, Boss-ruled); **§2 REVERTED** (`d03e2fd0` — the old Navigator restored, both mounts); §3–§5 cancelled. The rework restarts research-first from the ORIGINAL two-pane paradigm — frontier research `wf_50227623-e2e`; new concept paper to follow. This document remains as the historical record of the revoked direction.

**Date:** 2026-07-05 · **Phase:** 2 of 4 (Plan) · **Architect:** [MIG-090-Architect-Notes-Navigator.md](MIG-090-Architect-Notes-Navigator.md)
**Boss rulings (2026-07-05, all banked):** Direction = **Option B** (rebuild the list mode on the Base engine) · Batch ops = **PORT** through proper write paths · SS Navigator branch = **RETIRE in this MIG** (single reopened PJ-068 line; rest of PJ-068 stays parked).

**Concept (the horse):** *the corpus as a working set — every note in the universe, its standing, and what it needs next, one lens away.*

---

## Design decisions carried into the plan (recommended defaults, approved with this plan)

1. **The All-Notes view is a real `.base` file** (file-over-app): a per-universe **`All Notes.base`**, auto-provisioned on first use if missing, scope = all libraries (federated included — the ONE-universe ruling holds by construction in `execute_lens`). The user can rename/edit/restyle it like any Base; re-provisioning never overwrites an existing file.
2. **The sidebar "list" button opens it as a center TAB** (opens or focuses the All-Notes base tab), not a squeezed sidebar panel — a corpus table needs the center zone (Full-Center-Zone rule); this also kills the Navigator's forced-450px sidebar dance.
3. **Rollback** = the existing `enabledFeatures.notesNavigator` flag hides the mode button entirely; code rollback is git-revert. The old component chain does NOT linger behind the flag (the dead-fossil lesson: one validated swap, no parallel layer).
4. `search_by_property` retires with its sole caller (SearchHub's FTS5 `properties` category is the successor). `scan_library_tags` / `read_library_tree` survive (other consumers).

## The steps (each = one commit + verification clause)

### §0 — Predecessor → Replacement entries (no code)
Session-log entries per the Predecessor Lookup Rule for every retired surface: the sidebar list mode (predecessor: NotebookNavigator two-pane, MIG-lineage: pre-MIG; replacement: the All-Notes Base tab, same button), batch ops (NavBatchBar → BaseTab batch bar), property search (Navigator pane → SearchHub `properties`), SS branch (mount → retired per PJ-068 ruling), `collect_library_notes_with_metadata` (+ wrapper + `NoteWithMeta`) → `execute_lens`. Evidence rider: one runtime confirmation of the Folders-mode defect on the running app (Reproduce-First hygiene; evidential only — the pane is being deleted).

### §1 — Lens engine: three additive dimensions
`dimensions.rs` registry + `sql_builder` + `tableModel.ts` rendering + column labels (reuse existing i18n slugs where they exist; new ones ×15):
- **`note.library`** — Text, `note_meta.library_name`, sortable, filterable(eq).
- **`note.modified`** — Timestamp, `note_meta.modified`, sortable, filterable (same ops as `note.created_at`).
- **`note.tags`** — Text from `note_meta.tags_json`, rendered as chips (reuse the tag-chip rendering conventions); non-sortable v1 (sorting a JSON array string is noise — Form-Aligns-To-Purpose); filterable `contains` v1 if cheap (`tags_json LIKE '%"?"%'` — the established pattern at search.rs:6445), else defer filtering.
**Verify:** cargo tests (registry/sql_builder suites) green; a scratch Base renders the three columns; sort by modified works; svelte-check 0.

### §2 — The All-Notes Base + the swap (Boss test)
- `ensureAllNotesBase()`: on list-button click, create `All Notes.base` at the universe root if missing (content: scope all libraries; columns name · library · modified · tags · outgoing_count; order modified desc), then open/focus it as a base tab. Localized display name handling = whatever Bases already do with filenames (no new mechanism).
- Rewire the sidebar list button: same button, same flag, new action (open the base tab). Remove the 450px width-force + `preTreeWidth` dance. Delete the main-window `{#if sidebarMode === 'list'}` branch.
- **Retire the SS branch** (SecondScreenPage list branch removed; `SidebarMode` event contract keeps firing — the SS simply no longer branches on 'list'; no dangling branch, three-site lockstep honored).
**Verify:** click the button on the 7,600-note universe → the table opens **instantly** (no fs walk fires — journal/instrument check); search + letter rail + RC work (already-validated machinery); SS unaffected when open. **Boss test clause (tutorial per the Testing Instructions Rule).**

### §3 — Multi-select + batch bar on BaseTab (Boss test; Editor-Surface aware)
- Selection column (virtualization-safe: selection keyed by path, not row index) + a batch bar (visible when ≥1 selected): **Tag · Move · Delete**.
- **Batch tag:** per selected note, the SHARED single-note tag-add routine (locate the FileTree RC's add-tag handler; if not yet a shared function, extract it — reuse rule). Open-note reconciliation required (the `toggleTaskReconciled` precedent): a tag-add to an open note must adopt into its model, never be clobbered by the next autosave. NO YAML string-splicing anywhere.
- **Batch move:** shared folder picker + `move_item` per note (already gated).
- **Batch delete:** `deleteWithSetting` per note (trash-backed) + the existing styled ConfirmDialog + `$tn` plurals.
- Because batch-tag touches note content: run `npx vitest run tests/mig-076/` + the Editor-Surface Gate items 1/3/5 recipes locally before the Boss test; the Boss test includes tagging an OPEN, freshly-edited note.
**Verify:** svelte-check 0; harness green; **Boss test clause** (batch tag/move/delete incl. the open-note scenario).

### §4 — Deletions + i18n sweep
Delete: `NotebookNavigator.svelte` + `NavBrowserPane` + `NavFileList` + `NavFileItem` + `NavBatchBar` (1,033 lines) + dead `navigator/data.svelte.ts` (253) + `collect_library_notes_with_metadata` (Rust + lib.rs registration + store wrapper + `NoteWithMeta`) + `search_by_property` (sole-caller verified again at build time) + `navLoadSeq`/`propSearchSeq` guards (die with the component) + orphaned `navigator.*` i18n keys ×15 (KEEP the sidebar-tab keys consumed by +layout) + the `.nav-*` styles.
**Verify:** grep-clean (zero references to deleted symbols); cargo + svelte-check green; boot the big universe — boot budget unchanged (criteria 1–2); type-burst test (Rule 7).

### §5 — /simplify + Audit (Phase 4 of /migration)
/simplify on the full diff; then 3 audit agents: **invariants** (boot budget, Editor-Surface Gate, ONE-universe resolution, displays-not-domains), **drift** (new guards/registries the system doesn't know — LL-023), **migration path** (first boot with no `All Notes.base`; existing universe upgrade; flag-off state; SS event contract; base-file deleted-by-user mid-session). Orientation v-bump + help files + User Manual (×15) + Pending Jobs close-out in the same commit per SO #6.

## Out of scope (explicitly)
- Folder facet on the Base (waits for MIG-078 Phase B write-time folder stats — no second tree source).
- Stage/maturity lens columns (MIG-068's scope; the registry gains them there — the All-Notes Base will pick them up for free).
- Dataview's sibling live-scan shape (flagged to the Rule-8 audit list, not this MIG).
- Engine-side LIMIT for `execute_lens` (queued lens-engine item).

## Cost & risk
~5 commits + audit. Additive engine work (§1) and swap (§2) are low-risk; §3 is the care-point (content-touching batch-tag → harness-gated). Wall-clock: one focused session.
