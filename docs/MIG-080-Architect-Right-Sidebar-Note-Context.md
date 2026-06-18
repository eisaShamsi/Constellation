# MIG-080 — Architect: Right Sidebar → Note-Context-Only

**Status:** Architect (Phase 1 of the /migration). Awaiting Boss review → then Plan → approval → Build → Audit.
**Date:** 2026-06-17. **Branch:** `main`. **Seed:** `docs/Right-Sidebar-Note-Context-Design-Decision.md` (2026-06-16, commit `bfa66f0a`) + Boss confirmations 2026-06-17 (the **Tasks contextual split** refinement).
**Function in hand:** the **right sidebar** (`+layout.svelte:7032–7350`) — its 11 tabs, gated by `appSettings.panelPlacements.<panel>` and `NOTE_SCOPED_TABS` (`:389`).

---

## 1. The contract
The right sidebar is the **live, note-context extension of the open note** — every tab answers *"tell me about THIS note."* No whole-universe / whole-library aggregate renders there. (Form-Aligns-To-Purpose applied to a region; the dominant convention across VS Code / Obsidian / Notion / Roam / Logseq per the 06-16 WA#5 research `wf_e09ede1a`.)

The bug this fixes is visible today: clicking **Calendar** shows "No note selected" — a universe function wrongly jammed into the note-scoped rail (`calendar` ∈ `NOTE_SCOPED_TABS`).

## 2. Disposition (Boss-confirmed)
| Tab | Disposition | End-state |
|---|---|---|
| Properties (+Outline) | **STAY** | unchanged (already note-scoped) |
| Backlinks (+Outgoing) | **STAY** | unchanged |
| Tags — "This note" | **STAY** | the "All tags" toggle is removed from here |
| Sky-local (star) | **STAY** | unchanged |
| Provenance | **STAY** | unchanged |
| 360.3D (inspector360) | **STAY** | unchanged |
| Knowledge Health | **SPLIT** | right = this note's tensions/health (distinct from 360.3D); universe → **Dashboard** (reuse `KnowledgeHealthDashboard`) |
| Review Pulse | **SPLIT** | right = this note's review status; universe queue → **NEW full-page reviewer** + a "N due" Dashboard card |
| Source Review | **SPLIT** | right = this note's sources/suggestions; universe → **Cataloger** (reuse `CatalogerView`, left dock) |
| Tasks | **SPLIT** (Boss 2026-06-17: *contextual*) | right = the OPEN note's own task list (reuse `scanNoteTasks`); universe agenda → **left**, with the Calendar |
| Calendar | **RELOCATE OUT** | → left launcher → daily note (reuse the `:5487` Daily-Note dock launcher); fixes wrong-date (done §D-1) + wrong-library |
| Tags "All tags" | **RELOCATE OUT** | → **Dashboard** card (reuse the existing tags section) + the existing `#tag` → Search Hub routing; reads `tag_counts` (§C.1) |

## 3. Per-function design (current → end-state; reuse vs build)

### 3.1 Tasks (split) — *mostly exists*
- Current: right-rail tab (`:7055/7230`) renders `TasksPanel`; the populate `$effect` (`:1667`) already calls **`scanNoteTasks(tab.path)`** for the open note. So the right-rail Tasks is **already note-contextual** — it shows the open note's tasks. ✅ Little to build for the note side: keep it, ensure it's purely note-scoped (it is), keep `tasks` ∈ `NOTE_SCOPED_TABS`.
- Universe agenda → **left** (new section, `scanLibraryTasks` + the per-date grouping the Calendar dot-scan already computes). Lands next to the Calendar launcher.
- **Defect fixed (§7.E.2):** `toggle_task` (tasks.rs:408) writes via `gate_write` but never re-indexes → search/backlinks/tags drift. Fix: route the toggle through `reindex_note` (or call it after the gate write), so the index stays consistent. *(Editor-Surface Gate applies — this touches the note write path.)*

### 3.2 Calendar (relocate out)
- Remove the right-rail tab + its `NOTE_SCOPED_TABS` membership.
- **Left launcher:** the left dock already has a Daily-Note button (`:5487 handleOpenDailyNote` → today). Extend the launcher to a small date picker / optional month-grid (reuse `CalendarPanel`) → opens the picked day's daily note via `get_daily_note_path(date)` (the §D-1 date param — **reused groundwork**).
- **Defect fixed (wrong-library):** today `onDayClick` always uses `libraries[0]`. At the launcher, resolve the daily-note's library explicitly (the configured daily-note home, not "first library"). 
- §D-1 disposition: the Rust `get_daily_note_path` date param + `store.ts` wrapper **carry over** (the launcher needs them); the in-place right-rail `onDayClick` wiring + the CalendarPanel are reused at the launcher or retired with the tab.

### 3.3 Tags "All tags" (relocate out)
- Remove the "All tags" toggle from the right-rail Tags tab; keep "This note".
- Universe All-tags → the **Dashboard** (it already renders a tags section, `DashboardView:312`), upgraded to read the cheap **`tag_counts`** table (not the per-library `scanAllLibraryTags` filesystem walk). The existing tag-click → `#tag` Search Hub routing (`:4387`) stays as the query path. *(A dedicated Search-Hub tag-facet sidebar is optional polish, not required — flagged as a Boss decision.)*

### 3.4 Knowledge Health (split)
- Right (note): NEW `detect_note_tensions(libraryPath, notePath)` — reuses `detect_tensions` machinery (tension.rs, DB-backed per MIG-075) + a post-filter to tensions touching the open note. `TensionPanel` gains an optional `noteContext` prop. Stays **distinct from 360.3D** (Boss).
- Universe → **reuse `KnowledgeHealthDashboard`** (exists, full-page, cached `constellation_knowledge_health_snapshot`); add a summary card on the Dashboard.

### 3.5 Review Pulse (split) — *the heaviest item*
- Right (note): NEW `get_note_review_status(notePath)` — O(1) lookups in `review-pulse.json` (last reviewed / next due / interval). NEW compact `ReviewStatusPanel` (reuses the mark/snooze/dismiss actions).
- Universe → **NEW full-page reviewer** (`ReviewerView` — the decision's "1 new overlay"): stepped card UI over `get_due_notes`, dispatching `record_note_visit` on advance. + a "N due" Dashboard card.
- **Defect fixed (§7.E.4):** `record_note_visit` (review.rs:134) exists but is never called → notes resurface forever. Fix: call it from `openNoteTab` (note opened = visited) AND in the reviewer.

### 3.6 Source Review (split)
- Right (note): NEW `sources_list_pending_for_note(notePath)` — one `WHERE note_path = ?` on `sources_suggestions` (sources/mod.rs:687). `SourceReviewPanel` already takes `activeNotePath` — wire it to call the per-note IPC when set (currently it loads the full queue regardless — the gap to close). *(Note: `sources_get_for_note` already exists at sources/mod.rs:543 — verify whether it already serves this before adding a new command.)*
- Universe → **reuse `CatalogerView`** (exists, left dock).

## 4. Structural levers
- **`panelPlacements`** (`store.ts:3309` PanelId; `:3982` defaults; slots `left-of-note|right-of-note|right-sidebar|hidden`). Settings UI: `SettingsModal:2105`. **Bug to fix in passing:** `inspector360` is in the schema + tab strip but **missing from the Settings UI list** — add it.
- **`NOTE_SCOPED_TABS`** (`:389`) — reconcile to the true note-scoped set after the splits: `{properties, backlinks, star, tags(this-note), provenance, inspector360, health, review, tasks, sourceReview}`; **drop `calendar`** (relocated). (Each split's note panel is note-scoped → in the set.)
- **Settings migration (existing users):** no schema-version field; migrate inline in `applyParsedSettings` (`:4045`) — e.g. a saved `calendar='right-sidebar'` becomes `hidden` (calendar left the rail); the universe-Tasks placement maps to the new model. Workspace-restore `rightSidebarTab='calendar'` is caught by the existing safety `$effect` (resets to properties) — no breakage.
- **Second screen:** `EditorPanelsData` (secondScreen.ts:349) carries only note identity, no per-panel placement — **out of scope** for MIG-080 (display-not-domain holds).

## 5. Defects fixed by the moves (not patched in place — per the decision)
1. **Calendar wrong-library** → resolved at the left launcher (explicit daily-note home).
2. **Review `record_note_visit` never called** → wired at `openNoteTab` + the new reviewer.
3. **Tasks `toggle_task` no reindex** → routed through `reindex_note` (the Tasks subsystem is being touched anyway; content-integrity-adjacent).

## 6. Invariants (Audit will verify)
- **INV-1 — Note-context contract:** no universe/library aggregate renders in the right rail after the migration. Every right-rail tab is scoped to the open note (or the shared note-scoped empty-state).
- **INV-2 — No function lost:** every relocated function is reachable at its new home (left launcher / Dashboard / Cataloger / full-page reviewer / Search Hub), with parity of capability.
- **INV-3 — Content integrity (Editor-Surface Gate):** the `toggle_task` reindex fix touches the note write path → full gate (Focus round-trip, tab switch, body intact, reindex correctness). The per-note read IPCs (tensions/review/sources) are read-only.
- **INV-4 — Settings migration safe:** existing users' saved `panelPlacements` + workspace `rightSidebarTab` migrate without breakage or lost access.
- **INV-5 — i18n ×15 + RTL + right-click:** every new/moved surface follows the concept-paper checklist (strings in all 15 locales, native equivalents, RTL, shared `<ContextMenu>` where a menu is added).
- **INV-6 — Rule 8:** the relocated All-tags reads `tag_counts` (no scan); the note-scoped reads are bounded per-note queries; no new boot recompute.
- **INV-7 — Boot/latency not regressed:** no new boot IPC; the right rail's per-note reads stay on-demand.

## 7. Proposed phasing (each phase landable + Boss-testable)
This is a large migration; phased so each lands and is testable:
- **§A — Structural spine:** extend `panelPlacements`/`NOTE_SCOPED_TABS` for the note/universe split model; settings migration; fix the `inspector360` Settings-UI bug. *(No visible relocation yet; the scaffolding.)*
- **§B — Calendar → left launcher** (+ wrong-library fix; reuse §D-1 date param). Remove the Calendar right-rail tab.
- **§C — Tasks split:** confirm/keep note-contextual right rail; build the left agenda; fix `toggle_task` reindex (Editor-Surface Gate).
- **§D — Tags "All tags" → Dashboard card** (read `tag_counts`); remove the right-rail toggle.
- **§E — The 3 universe→note splits:** §E1 Source Review (lightest — IPC + panel wiring + reuse Cataloger); §E2 Knowledge Health (per-note IPC + TensionPanel prop + reuse KH Dashboard); §E3 Review Pulse (per-note IPC + ReviewStatusPanel + NEW full-page reviewer + `record_note_visit` fix — heaviest).
- **§F — Dashboard cards** (universe Health + "N reviews due") + `NOTE_SCOPED_TABS` final reconcile.
- **§G — Audit** (3 agents: invariants / drift / migration-path) + the Editor-Surface Gate for §C.

## 8. Open decisions for Boss
1. **Search Hub tag-facet:** the decision doc says "Search Hub **+** Dashboard" for All-tags. The `#tag` → Search Hub *query* routing already exists; a dedicated tag-facet *sidebar* in Search Hub is extra build. **Recommend: Dashboard card now (cheap, reuses existing); defer the Search-Hub facet** unless you want the browse-all-tags entry point inside Search Hub too.
2. **Calendar left form:** a minimal **launcher** (date picker → daily note, smallest) vs the full **month-grid widget** on the left (reuses `CalendarPanel`, shows dots). Recommend launcher first; month-grid as opt-in.
3. **Phasing granularity:** ship as the §A–§G sequence above (recommended — each testable), or bundle some (e.g. the 3 splits together)?
4. **§D-1 commit:** keep (the `get_daily_note_path` date param is reused by §B) — no revert needed. Confirm.
