# MIG-080 — Plan (Phase 2): Right Sidebar → Note-Context-Only

**Companion to:** `docs/MIG-080-Architect-Right-Sidebar-Note-Context.md`.
**Approach:** the right sidebar becomes the open note's context only. 6 tabs stay; 4 split (note version stays right / universe version relocates); 2 relocate out. Defaults locked (Boss 2026-06-17): All-tags → **Dashboard card only** (Search-Hub facet deferred); Calendar → **launcher** first (month-grid opt-in later); **keep §D-1** (`get_daily_note_path` date param reused by §A).
**Gate:** Boss approves this Plan, then Plan-Approval = Build-Approval — cascade phase by phase, stopping at each phase's Boss-test clause and on genuine architectural surprise.
**Ordering rationale:** Calendar first (the Boss-flagged item + cleanest standalone), then the light relocations, then the 3 splits easiest→heaviest. Each phase reconciles **its own** `NOTE_SCOPED_TABS` membership *together with* its note-scoped implementation (so no tab is gated note-scoped before it can actually show note data).

---

## §A — Calendar → left launcher (+ wrong-library fix, + settings migration, + inspector360 Settings-UI bug)
**Why first:** the Boss-flagged "No note selected" tab; the cleanest standalone relocation; reuses §D-1.
**Changes:**
- **Remove** the right-rail Calendar tab (`+layout.svelte:7060` button + `:7256` render branch) and **drop `'calendar'`** from `NOTE_SCOPED_TABS` (`:389`).
- **Left launcher:** extend the existing Daily-Note dock button (`:5487 handleOpenDailyNote`) into a launcher — a date picker (and/or the reused `CalendarPanel` month-grid as an opt-in left widget) → opens the picked day's daily note via `getDailyNotePath(libPath, fmt, folder, date)` (§D-1 param).
- **Wrong-library fix:** resolve the daily-note's library from the configured daily-note home (not `libraries[0]`).
- **Settings migration** (`store.ts applyParsedSettings ~:4045`): a saved `panelPlacements.calendar` → drop/`hidden`; the workspace `rightSidebarTab='calendar'` is caught by the existing safety `$effect`.
- **Fold in:** add `inspector360` to the `SettingsModal` panel-placement list (`:2105` — it's in the schema + tab strip but missing from the UI; a pre-existing bug).
**Verify:** svelte-check 0; build; **Boss-test** — Calendar no longer in the right rail; the left launcher opens the *clicked* day's daily note in the *correct* library; "open today" still works; existing-user settings migrate cleanly (no lost panels); the inspector360 placement now appears in Settings. (Read/launch path only — no note-content write.)

## §B — Tags "All tags" → Dashboard card
**Changes:**
- Remove the "All tags" toggle from the right-rail Tags tab (`:7108–7142`); keep "This note" (default the tab to note-tags only).
- **Dashboard** (`DashboardView.svelte:312`): upgrade the existing tags section to read the cheap **`tag_counts`** table (new `get_all_tags()` reading `tag_counts`, or reuse the boot `allLibraryTags`) instead of the per-library `scanAllLibraryTags` filesystem walk. Keep the existing tag-click → `#tag` Search Hub routing (`:4387`).
**Verify:** svelte-check 0; build; **Boss-test** — right-rail Tags shows only the open note's tags; the Dashboard shows the universe tag list with counts; clicking a tag still opens the `#tag` Search Hub query; (Rule 8) the Dashboard tag list reads the table, not a scan.

## §C — Tasks split (note-contextual right rail + left agenda + `toggle_task` reindex fix)
**Changes:**
- Right rail: confirm/keep the note-contextual Tasks (`scanNoteTasks(tab.path)`, `:1667/7230`) — it already shows the open note's tasks; keep `'tasks'` ∈ `NOTE_SCOPED_TABS`.
- **Left agenda:** new left-sidebar section (the `.library-section` pattern, `:5737`) listing the universe/library tasks (`scanLibraryTasks`), date-grouped (reuse the Calendar dot-scan's per-date grouping), placed with the §A Calendar launcher.
- **Defect §7.E.2 fix:** route `toggle_task` (tasks.rs:408) through `reindex_note` after the gate write so search/backlinks/tags don't drift on a checkbox toggle.
**Verify:** svelte-check 0; cargo test; build; **Boss-test + Editor-Surface Gate** — right-rail Tasks = the open note's tasks; the left agenda lists universe tasks; toggling a task persists AND the note's index/search/backlinks stay correct (toggle a task, then search for changed text / check backlinks — no drift); Focus round-trip + tab switch + body intact. *(Touches the note write path → full gate.)*

## §D — Source Review split (note IPC + panel wiring; reuse Cataloger)
**Changes:**
- **New IPC** `sources_list_pending_for_note(notePath)` — `WHERE note_path = ?` on `sources_suggestions` (sources/mod.rs:687). *(First verify whether the existing `sources_get_for_note` (`:543`) already serves this.)*
- `SourceReviewPanel` (`:701`): when `activeNotePath` is set (right-rail), call the per-note IPC instead of the full queue (close the current "ignores activeNotePath" gap).
- Universe Source Review → **reuse `CatalogerView`** (left dock, exists). Add `'sourceReview'` to `NOTE_SCOPED_TABS`.
**Verify:** svelte-check 0; cargo test; build; **Boss-test** — the right-rail Source Review shows only the open note's pending suggestions; the Cataloger (left) shows the full universe queue; no note open → note empty-state. (Read-only.)

## §E — Knowledge Health split (note IPC + TensionPanel prop; reuse KH Dashboard)
**Changes:**
- **New IPC** `detect_note_tensions(libraryPath, notePath)` — reuses `detect_tensions` (tension.rs), post-filtered to tensions touching the open note.
- `TensionPanel` gains an optional `noteContext` prop; the right-rail Health tab passes it (shows this note's tensions/health, **distinct from 360.3D**).
- Universe Health → **reuse `KnowledgeHealthDashboard`**; add a summary card on the Dashboard.
**Verify:** svelte-check 0; cargo test; build; **Boss-test** — right-rail Health = this note's tensions only; the Dashboard/KH-Dashboard shows the universe health; distinct from 360.3D. (Read-only.)

## §F — Review Pulse split (note status tab + NEW full-page two-lens reviewer)
**The heaviest phase.** **UPDATED 2026-06-22 to reflect MIG-083 (shipped) + the Boss placement ruling.** MIG-083 changed the foundations this phase builds on:
- `get_note_review_status(notePath)` is **already built** (MIG-083 §D-3) — an O(1) PK lookup on the `review_schedule` **table** (NOT `review-pulse.json`). Returns `{ reason, due_days, last_reviewed, never_reviewed, is_checkpoint }`. §F just consumes it.
- **`record_note_visit` is DROPPED — locked decision (opening ≠ review).** The ONLY thing that advances `last_reviewed` is the explicit **✓ Reviewed** action. So the old §7.E.4 "call `record_note_visit` from `openNoteTab`" and "dispatch `record_note_visit` on advance" are **obsolete — do NOT build them.**
- `get_due_notes` now returns **TWO lenses** (Due-for-Review + Stale); stale `DueNote`s carry `stale_trigger_name`/`stale_trigger_type`/`stale_changed_on` for the per-row "why."

**Changes:**
- **New `ReviewStatusPanel`** (compact: this note's status from `get_note_review_status` + ✓/Snooze/Dismiss, reusing `ReviewPulsePanel`'s actions) for the **right-rail Review tab**; add `'review'` to `NOTE_SCOPED_TABS`. This makes the right-sidebar Review tab genuinely **note-scoped** (resolves the "Review Pulse isn't note-related" mismatch).
- **New full-page two-lens reviewer (`ReviewerView`)** over `get_due_notes`: **Due-for-Review** and **Stale** as two distinct sections, each row rendering its "why" (stale → "stale because {type} {name} changed on {date}"). **Placement — Boss ruling 2026-06-22: a LEFT-DOCK core surface** (a clock nav icon → the full-page reviewer), matching Sky View / Index / Map / Calendar. The universe-wide queue leaves the note-context right sidebar entirely; the left dock is where universe-wide full-page views live. Optionally + a "N due" Dashboard card.
- **Retire the transitional right-sidebar universe `ReviewPulsePanel`** (its universe queue moves to the left-dock reviewer; its per-note slot becomes `ReviewStatusPanel`). Honor the Predecessor Lookup Rule for the `panelPlacements.review` wiring.
- **Help / User Manual ×15 (the SO #2 item deferred from MIG-083):** write the Review-Pulse help topic + manual section here, documenting the FINAL surface — the note-status tab, the left-dock two-lens reviewer + per-row why, ✓/Snooze/Dismiss, and Settings → Review staleGraceDays.
**Verify:** svelte-check 0; cargo test; build; **Boss-test** — the right-rail Review tab shows THIS note's status; the left-dock clock icon opens the full-page reviewer with Due-for-Review + Stale lenses (each row's why correct); ✓/Snooze/Dismiss persist + the row leaves; grace-period Setting observable. (Review state only — not note content.)

## §G — Final reconcile + Audit
- Confirm the final `NOTE_SCOPED_TABS` = `{properties, backlinks, star, tags, provenance, inspector360, health, review, tasks, sourceReview}` (no `calendar`).
- `/simplify` on the full MIG-080 diff.
- **Audit (3 agents):** invariants (INV-1…7), drift (the split model, the relocated surfaces, the settings migration), migration-path (existing users' `panelPlacements` + workspace `rightSidebarTab`; first-boot; rollback). Re-run the Editor-Surface Gate for §C.
- SO #6: orientation v-bump (new file) folding MIG-080; help/manual updates (the new homes); session log; MoCh.

## Notes
- **Second screen:** out of scope (carries only note identity; display-not-domain holds).
- **i18n ×15 + RTL + right-click:** each phase that adds/moves a surface follows the concept-paper checklist (strings in all 15 locales, native equivalents, shared `<ContextMenu>` where a menu is added) — verified per phase, not batched.
- **§D-1 commit `a4152934`:** kept; its `get_daily_note_path` date param is consumed by §A. No revert.
