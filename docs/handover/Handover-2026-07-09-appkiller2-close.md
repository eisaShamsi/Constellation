# Handover — 2026-07-09 (APP-KILLER #2 shipped → next: build G3 second-screen)

**Read first each session:** `docs/Constellation Orientation & Onboarding v3.34.md` (highest version). This handover is the fast pick-up for the *next* session.

---

## What shipped this session (Boss-validated, on `origin/main`, tree clean)

**APP-KILLER #2 — NoteModel-Ownership silent nav-loss: FIXED + Boss-validated 10/10 + CLOSED.** The 3rd/last worst-case silent-loss bug. A navigation is a DEPARTURE, and a departure flushes: whatever you were typing reaches disk before the editor is handed to the next note. Full four-phase `/migration` (Architect `wf_0862e784-829` → Plan+Boss-approval A3+B1 → Build §1–§7 → per-cycle sweep `wf_415a7214-4ad`). Commits `c43923ce..d84b1fb4`. Orientation **v3.34**.

- **Fix:** `noteSession.flushIfDirty` (the one nav-flush choke point; old path from the model; bounded flush-while-dirty loop; failed flush ABORTS the nav) + `store.ts flushOutgoing` at the **3 departure sites** — `openNoteTab` reuse, `loadTabHistoryEntry`, **`closeTab`** (the sweep-caught 3rd) — + `markSaved(id,ver,expectPath?)` swap-poison guard + Focus-exit-on-nav + rename-on-locked-file durability + one-path-one-tab dedup (B1). Behind `NAV_FLUSH_ENABLED` + `DEDUP_ALL_TABS_ENABLED`.
- **Safety sweep** (whole-app, 44 agents, 23 confirmed): diff CLEAN of new app-killers; fixed the `closeTab` APP-KILLER + FocusPane window-close HIGH in-build (WA#6). Register in the Charter.
- **Boss-validated 10/10** on the running release binary (Stage 1 nav cases + Stage 2 rename/switch/properties/restart/locked-file).

Also: harness `tests/mig-076/runtimeHarness.test.ts` Recipes I/J/K/M (22/22). User Manual "Saving and Recovery" made explicit + a one-note-one-tab note. MoCh `docs/MoCh/MoCh-2026-07-08-2245.md`.

---

## NEXT — BUILD G3 (Architect + Plan DONE + Boss-APPROVED; just build it)

**G3 — Second-Screen Cross-Window Sync `/migration`.** The 2 remaining HIGH from the APP-KILLER #2 sweep. The second screen (a separate Tauri webview / JS realm) is blind to a main-window rename cascade AND never adopts main→SS saves into its writable editor tabs → a stale SS edit stomps the rewrite / clobbers main's committed edits.

**Boss ruling (2026-07-09):** **read-only by DEFAULT + a Settings toggle to make it editable.** Read-only default fixes both HIGH by construction (the SS never writes); the editable-toggle path is made safe too (WA#6 — never ship a known-unsafe toggle).

**The approved Plan (`docs/G3-SecondScreen-CrossWindow-Plan.md`) — build these:**
- **§1** `readOnly?: boolean` prop on `NoteEditor` (+ `NotePane`: CM6 `EditorState.readOnly` + `EditorView.editable.of(false)`); `appSettings.secondScreenEditable` (default **false**) propagated via `screen:settings-changed`; wire all **7** SS `<NoteEditor>` mounts to `readOnly={!$appSettings.secondScreenEditable}`; Settings toggle "Make the second screen editable" + i18n ×15.
- **§2** SS `onNoteSaved` (`SecondScreenPage.svelte:723`): after the echo guard, for EVERY SS tab (openTabs editor tabs + companion tabs) with `path === saved path`, re-read disk + `externalChange(tab.id, content)` + bump `reloadVersion`. Freshness-gated; **all-tabs** not first-match.
- **§3** SS `listen('cascade:rewrote')` (already broadcast via `app.emit`) → adopt/reload the rewritten paths into matching SS tabs (mirror `+layout.svelte:3223`).
- **§4** editable-mode cross-window **freeze**: main emits `screen:cascade-freeze {paths,active}` around its `markCascading`/`clearCascading`; SS raises/clears its OWN realm `markCascading` so its autosave is gated during a main cascade (+ a stuck-freeze auto-clear timer).
- **§5** harness two-sessions-one-path recipe (adopt-when-clean / refuse-when-dirty / reload-on-cascade) + `/simplify` + diff-scoped `safety-inspection` + **Editor-Surface Gate item 7** TWO-WINDOW Boss test (the real proof — a single-realm harness can't reach it).

**Architect doc** `docs/G3-SecondScreen-CrossWindow-Architect.md` (workflow `wf_a6e3b69b-da7` — the 4 mappers verified the territory; the schema-strict synthesis agent failed, so the synthesis was hand-authored from the mappers' findings). Frontend-only, no schema; rollback = flags + revert.

## THEN (queued)
**Auto-restore-tabs-on-relaunch feature** (Boss-wanted 2026-07-09) — a **Settings toggle, DEFAULT ON** ("Restore my tabs on startup"). Open tabs are NOT persisted across restart today (only manual named workspaces `restoreWorkspace`/`persistWorkspaces`). Persist openTabs+activeTabId on change (debounced) + restore safely on boot (never overwrite a note with a stale restored copy). Touches persisted-JSON + boot-ordering (audit-sensitive) → likely its own small `/migration`. Then the standing **G2–G8 backlog** (Charter).

## Where to read (fresh session)
`docs/Constellation Orientation & Onboarding v3.34.md` → `docs/G3-SecondScreen-CrossWindow-Plan.md` (the approved build) + `-Architect.md` → `lab/reports/SESSION-LOG-2026-07-08.md` (§ APP-KILLER #2 + § G3) → `docs/Constellation-Safety-Audit-CHARTER.md` (full register). Memory: `project_appkiller2_navloss_shipped`, `project_safety_audit_active`.

---

## Ready-to-paste next-session prompt

> Build the **G3 second-screen cross-window sync** `/migration` — the Architect + Plan are DONE and Boss-approved (`docs/G3-SecondScreen-CrossWindow-Plan.md`; read `docs/handover/Handover-2026-07-09-appkiller2-close.md` + orientation v3.34 first). Boss ruling: **second screen read-only by default + a Settings toggle to make it editable**, and the editable path must be safe (WA#6). Cascade the approved plan: §1 `readOnly` prop on `NoteEditor` + the `secondScreenEditable` Settings toggle (default off) wired to all 7 SS editor mounts (+ i18n ×15) · §2 SS `onNoteSaved` adopts main→SS saves into ALL same-path SS tabs (`externalChange` + `reloadVersion`, freshness-gated) · §3 SS `listen('cascade:rewrote')` → adopt/reload rewritten paths · §4 editable-mode cross-window `screen:cascade-freeze` so the SS autosave can't stomp a main cascade · §5 extend `tests/mig-076/runtimeHarness.test.ts` with a two-sessions-one-path recipe + `/simplify` + diff-scoped `safety-inspection` + the **Editor-Surface Gate item-7 two-window Boss test**. Stop at §1–§4 for the two-window Boss tests. Then the per-cycle sweep + close-out (orientation v-bump, MoCh, manual, PCS). THEN queued: the auto-restore-tabs feature (Settings toggle, default ON).
