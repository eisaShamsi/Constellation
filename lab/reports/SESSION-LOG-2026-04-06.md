# Session Log — 2026-04-06

## Phase: SS Architecture Redesign — Monitor Detection, Panel Migration, Focus-First

### Context
The previous session (2026-04-04) defined the SS redesign principles: hardware-first (2+ monitors), no auto-restore, focus by default (main window = clean writing space), panels migrate to SS, context-aware, user in control. This session implements those principles.

### Commit: `1c407a9`

### What Was Built

#### 1. Monitor Detection (Rust + TypeScript)
- **Rust**: `list_monitors` command returns all connected displays with position, size, scale factor, and primary flag
- **Rust**: `open_second_screen_on_monitor` auto-positions SS centered on secondary monitor (80% of display area)
- **TypeScript**: `listMonitors()`, `hasMultipleMonitors()`, `openSecondScreenSmart()` — smart open that auto-detects and positions

#### 2. Right Sidebar Auto-Hide (Main Window)
- When SS opens: main window's right sidebar automatically hides, stores previous state
- When SS closes: right sidebar restores to its pre-SS state
- `$effect` in `+layout.svelte` watches `secondScreenOpen` and toggles `rightSidebarOpen`

#### 3. Editor Panels Companion (SS)
- New event: `emitEditorPanels` / `onEditorPanels` with `EditorPanelsData` type
- Main window emits editor panel data on tab switch (notePath, content, libraryName)
- SS receives and loads: Properties (frontmatter), Backlinks, Forward Links, Tags, Local Star
- Reuses existing `.split-companion` / `.sc-*` CSS classes — no new UI chrome
- Panel tabs: Properties, Backlinks, Tags, Star, Tasks

#### 4. No Auto-Restore
- Workspace restore no longer opens SS — `if (screen?.open)` block replaced with close-only logic
- SS always starts closed — user opens deliberately

#### 5. Smart Open Everywhere
- `handleToggleSecondScreen` and `handleSendToSecondScreen` both use `openSecondScreenSmart()`

### Architecture / Key Decisions
- **"Screens Are Displays, Not Domains"** — SS doesn't re-implement panel data loading; it uses the same `scanLibraryLinks`, `parseFrontmatter`, `buildStarData` functions
- **No new UI paradigm** — editor panels companion reuses the existing split companion CSS
- **Context-aware auto-switching** — editor panels mode activates when main window emits panel data, resets when other companion modes activate (map, index, dashboard)

### Files Modified
- `src-tauri/src/lib.rs` — `list_monitors`, `open_second_screen_on_monitor` commands
- `src/lib/secondScreen.ts` — `MonitorInfo`, `EditorPanelsData`, smart open functions, event emitters/listeners
- `src/lib/components/SecondScreenPage.svelte` — editor panels companion mode, panel data loading, monitor detection
- `src/routes/+layout.svelte` — right sidebar auto-hide, editor panels emission, smart open, no auto-restore
- `src/lib/i18n/*.json` (15 files) — `editorPanels`, `forwardLinks` keys

### Test Results
- `npx vite build` — clean ✓

### Open Items
- Test with actual 2-monitor setup (monitor auto-positioning)
- Constellation Map Phase 2: maturity inference + drill-down animation
- CE Layer 3: Constellation Lens
- Consider gating SS button visibility when only 1 monitor detected

---

## Test Tutorial: SS Architecture Redesign

### What It Is
The Second Screen is Constellation's companion window. When you have two monitors, it automatically positions itself on the secondary display and takes over the role of the right sidebar — showing Properties, Backlinks, Tags, Star, and Tasks for whatever note you're editing. The main window becomes a clean writing space.

### Why It Exists
Writers and researchers need focus. The main window should be for writing, not for navigating context panels. With a second monitor, all context moves to the Second Screen. Without a second monitor, the right sidebar works as before.

### Why It Matters
Constellation is an extension of the mind. The main window is where you think. The Second Screen is where context lives — invisible infrastructure that supports thinking rather than demanding attention. Simple by default, powerful on demand.

### Testing — Step by Step

#### Test 1: Opening with Monitor Detection
1. Connect a second monitor (or test with single monitor).
2. Click the **monitor icon** (rectangle with a stand) in the **bottom dock bar**.
3. **Two monitors**: The Second Screen should appear **centered on your secondary monitor**, filling about 80% of that display.
4. **Single monitor**: The window opens at normal size on the primary display.
5. Look at the Second Screen — it should show the editor panels header with the active note name.
6. If 2+ monitors are detected, a small "2 displays" badge should appear in the header.

#### Test 2: Right Sidebar Auto-Hide
7. Before opening SS, ensure the **right sidebar is open** (click the sidebar toggle if needed). Note which panel tab is active (Properties, Backlinks, etc.).
8. Open the Second Screen.
9. The **right sidebar in the main window should automatically hide**.
10. Close the Second Screen (click × in its toolbar or use Ctrl+Shift+2).
11. The **right sidebar should reappear** in the same state it was before.

#### Test 3: Editor Panels on SS
12. Open a note that has frontmatter properties, tags, and wiki-links to other notes.
13. Open the Second Screen.
14. The SS should show **"Panels"** header with the note name.
15. The **Properties tab** should be active by default, showing frontmatter key-value pairs.
16. Click the **Backlinks tab** (🔗) — should show notes that link to this note, each with a colored library dot.
17. Click the **Tags tab** (🏷) — should show frontmatter tags as rounded badges.
18. Click the **Star tab** (⭐) — should show a local graph of the note and its connections.
19. Click the **Tasks tab** (☑) — should show tasks from the note (or "No tasks" if none).

#### Test 4: Panel Updates on Tab Switch
20. In the main window, switch to a **different note tab**.
21. The SS panels should **update automatically** to show data for the new active note.
22. The Properties tab should show the new note's frontmatter.
23. Backlinks should show the new note's backlinks.
24. Tags should show the new note's tags.

#### Test 5: Clicking Links in SS
25. On the SS Backlinks tab, click a **backlink note name**.
26. That note should **open in the main window** (not in the SS).
27. The SS panels should then update to show data for the newly opened note.

#### Test 6: No Auto-Restore
28. Open the Second Screen. Open several notes.
29. Save a **workspace** via the Workspace Manager.
30. Close the Second Screen.
31. **Restore** the saved workspace.
32. The Second Screen should **NOT reopen** — it stays closed.
33. All other workspace state (tabs, sidebars) should restore normally.

#### Test 7: Context-Aware Mode Switching
34. With SS open, open the **Constellation Map** (click its icon in the dock).
35. The SS should switch from editor panels to **Map companion** mode (grid of library sunbursts).
36. Close the Map.
37. The SS should switch back to **editor panels** mode showing the active note's data.
38. Open the **Index** (book icon in the dock). Click a term.
39. The SS should switch to **Index companion** mode.
40. Close the Index.
41. The SS should return to **editor panels** mode.

### Summary of Expected Results

| Test | Pass Criteria |
|------|---------------|
| 1. Monitor detection | Auto-positions on secondary monitor; display badge |
| 2. Right sidebar auto-hide | Hides on SS open, restores on SS close |
| 3. Editor panels | Properties, Backlinks, Tags, Star, Tasks all render correctly |
| 4. Tab switch | Panels update when switching notes in main window |
| 5. Link clicks | Backlink/forward link clicks open note in main window |
| 6. No auto-restore | Workspace restore does not reopen SS |
| 7. Context-aware | SS switches between editor/map/index companions automatically |
