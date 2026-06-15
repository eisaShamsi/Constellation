# 26 — Second Screen (Concept Paper)

> A per-function paper under the bring-up program — template from [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3, serving the core charter [00-Constellation](00-Constellation-Core-Concept-Paper.md). The Second Screen is **function #27, Phase 6** — re-enabled **last** on purpose: it is the thinnest display surface and the historical double-init culprit.

## 1. Function in hand
The **Second Screen** — `SecondScreenPage.svelte` (`src/lib/components/SecondScreenPage.svelte`), the companion display rendered in the separate `second-screen` Tauri window (`tauri.conf.json` → `url: screen.html`, entry `src/screen-entry.ts`). It mirrors and extends the main window onto a second monitor: a note editor, plus companion modes (Sky View, Constellation Map, Index term/compare, Dashboard, split-comparison panels, the migrated right-sidebar panels).

## 2. Purpose
Give the user **more room** — show on a second display what the main window can't fit: the active note alongside its links/graph/tags, two notes compared side by side, a term's mentions, the universe dashboard. It is a *display* of work the Editor (the gate) is already doing, so it serves **Connection** and **Tension** (seeing a note next to its backlinks / a contradicting note next to it) without owning any new knowledge act. It justifies itself only as a window — never as a second source of truth. (Per **Display-Not-Domain**: additional screens mount core components and display them; they must never re-implement save/load/edit.)

## 3. What it is NOT
- **NOT** a second domain. It must not own content, save logic, tab management, or universe activation. It mounts `NoteEditor`/`NotePane` and lets the core editor handle all operations.
- **NOT** a place to recompute the universe. Companion panels read what the Editor/index already produced.
- **NOT** required. A one-monitor setup is complete; the second screen is opt-in and stays hidden until opened.

## 4. Wiring
- **Inputs (events, via `$lib/secondScreen`):** `onNoteToScreen`, `onNoteSaved`, `onUniverseSwitch`, `onSettingsChanged`, `onStateRequest`/`onWorkspaceRestore`, `onContextChanged`, `onSkyViewHover`/`onSkyViewClick`, `onSidebarModeChanged`/`onSplitModeChanged`, `onDashboardOpenNote`/`onDashboardTagSelected`, `onIndexTermSelected`/`onIndexCompare`, `onMapCompanion`, `onEditorPanels`; plus the Tauri `library-changed` / `screen-hidden` events and `localStorage` (`constellation-recent-*`) shared with the main window.
- **Inputs (IPC):** `listUniverses`, `read_note`, `collect_library_notes`, `notes_by_tag`, `read_child_universe_libraries`, `list_monitors`; store helpers `loadSettings`/`loadLibraries`/`scanLibraryLinks`/`scanLibraryTags`/`loadAllStats`.
- **Outputs (events/IPC):** `sendNoteToMain` (push a clicked note back to the main window — the correct direction), `sendScreenState`/`emitScreenReady`/`notifyScreenClosed`, `close_second_screen`. ⚠️ Also calls `setActiveUniverse(universes[0].id)` at **line 923** in `onMount` — a **Display-Not-Domain violation** (a display re-activating the universe; the documented double-init culprit). Correct form: read-only `getActiveUniversePath` / `get_active_universe_path` (both exist), never re-activate.
- **Consumers:** the main window (receives `sendNoteToMain` and `screen-*` state); no other surface depends on the second screen.
- **Connection to the Editor (the gate):** it **mounts** `NoteEditor`/`NotePane` directly (one shared editor, no re-implemented save path) and learns of changes only via `onNoteSaved`/`library-changed` events the gate fires. It must remain a pure mount — the `setActiveUniverse` call is the one place it breaches that contract today.

## 5. Right-click / context menu
**Has none.** No `oncontextmenu` / `<ContextMenu>` / `buildContextMenu` anywhere in `SecondScreenPage.svelte` (grep clean). Companion items (notes, links, tags, sky/map nodes) are activated by **left-click only** — they call `sendNoteToMain`, `openNoteTab`, or `loadPeekPreview`. **Gap flagged:** the file-tree / tabs / panels in the main window expose right-click actions through the shared `<ContextMenu>` (MIG-077); the second screen's note lists and link lists arguably should offer the same shared menu (open / open-in-main / reveal) rather than dead-ending on a single left-click. No hand-rolled menu to fold — the work is to **add the shared `<ContextMenu>`**, not to flag debt. Verify scope in bring-up.

## 6. Multilingual
- Root container honours `dir={$dir}`; per-item `dir={detectDir(...)}` / `dir="auto"` is used widely (note names, tag names, term columns) — RTL handling is mostly present.
- Most chrome strings flow through `$t(...)` (`secondScreen.*`, `panels.*`, `statusBar.notes`, `constellationMap.*`).
- **Hardcoded English found — flag:** `"Screen 2"` / `(Screen 2)` (toolbar title, ~952–954); `"Map companion"` (~1227); `"2nd Display"` badge (~1368); and in the Sky View companion block the bare labels `Backlinks`, `No backlinks`, `Forward links`, `No forward links`, `Tags` (~1546–1593); plus numerous `title=` tooltips (`"Close"`, `"Back"`, `"Forward"`, `"Pinned"`). Several `$t(...)` calls also carry English `|| 'fallback'` literals — acceptable as fallbacks but the **key must exist in all 15 locales** (ar de en es fa fr he hi ja ko pt ru tr ur zh). Bring-up: replace the bare literals with `$t()` keys and confirm 15-locale coverage + RTL on the toolbar/status bar.

## 7. Boot behavior
- **Runs at boot?** The `second-screen` window is **pre-declared** (`tauri.conf.json`, `visible: false`) and exists from app start; `SecondScreenPage` `onMount` runs when the window is created. On mount it registers ~18 listeners, then calls `listUniverses` → **`setActiveUniverse`** → `loadAllData` (`collect_library_notes` per library) → `loadDashboardData` (`loadAllStats` + child-universe libs + per-library `scanLibraryTags`).
- **Rule 8 status: RECOMPUTES-on-read — VIOLATION.** Companion data is rebuilt at read time, not read from a persisted derived view: `scanLibraryLinks` + `buildSkyData` on every Sky View / panel load, `scanLibraryTags` merged across libraries on dashboard load, link/backlink derivation per note. This mirrors the same boot-graph recompute MIG-079 targets (Phase 3). The second screen should consume the **persisted** snapshots, not re-walk the universe.
- **Cost:** not separately measured here. Estimated: per-library `collect_library_notes` + `scanLibraryTags` over a 7,600-note universe is the dominant cost, in the same order as the main window's boot graph (tens of ms to seconds). **Estimated — measure in bring-up.**

## 8. Flag / gate & bring-up position
- **Gate today:** the master inventory names the gate `secondScreen` for function #27, but **no `enabledFeatures.secondScreen` flag exists yet** (grep clean) — so this is a **NEW gate to add** in bring-up (wrap window-open + `onMount` work). The window-open IPCs (`open_second_screen`, `open_second_screen_on_monitor`, `close_second_screen`, `is_second_screen_open`) are unconditional today.
- **Bring-up phase: 6 (Federation, second screen, infra)** — re-enabled **last**. Depends on: the Editor (Phase 1, the gate) being proven; the companion modes' source surfaces (Sky View / Map / Index / Dashboard, Phases 2–4) being on; and **Federation (#26)** for the cUniverse/child-universe dashboard paths (`getChildUniverses`, `read_child_universe_libraries`). Sequencing rule (charter §6): the second screen must come up as a **pure display** — read-only `get_active_universe`, never re-activates (the `setActiveUniverse` fix is part of MIG-079's single-owner activation).

## 9. Budget
- **Boot budget:** opening the second screen must not regress the **main** window's boot (paint / hydrated envelope). Its own first-paint should be within the hydrated envelope of the universe it reflects; with Rule 8 honoured (read persisted), companion-load should be ≤ a few hundred ms.
- **Interaction budget:** companion updates on hover/click/save must be perceptibly instant; the editor it mounts keeps the Editor's keystroke budget (zero `invoke()` on the keystroke path) — the second screen adds none.
- **Regression guard:** open the second screen on a 7,600-note universe; confirm main-window boot + typing latency are unchanged; type a burst in a note mounted on the second screen and confirm the save lands once (no double-write); switch universes and confirm the second screen does **not** re-activate (no double-init).

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** shows the active note + a companion (links/graph/tags/compare) on a second display; clicking an item routes back to the main window via `sendNoteToMain`.
- [ ] **Serves Constellation's core purpose:** it is a *display* of Connection/Tension, owning no new act; **Display-Not-Domain** holds.
- [ ] **Wires correctly to the Editor (the gate):** mounts the shared `NoteEditor`; no re-implemented save/load/tab logic; learns of changes only via gate-fired events. **`setActiveUniverse` removed** → read-only `get_active_universe`.
- [ ] **Right-click present + correct:** the (currently missing) shared `<ContextMenu>` is added to note/link lists per MIG-077 — not hand-rolled — or its absence is explicitly ruled acceptable.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** `"Screen 2"`, `"Map companion"`, `"2nd Display"`, the Sky View `Backlinks/Forward links/Tags` labels, and `title=` tooltips moved to `$t()`; keys present in all 15 locales; toolbar/status bar correct in RTL.
- [ ] **Within budget:** opening the second screen does not regress main-window boot or typing latency.
- [ ] **Obeys Rule 8:** companion data reads persisted derived views, not `scanLibraryLinks`/`scanLibraryTags` re-walks on each load.
- [ ] **Holds its invariants:** Display-Not-Domain (no domain ops); no double-init on universe switch; mounted editor passes the **Editor-Surface Gate** (on-screen === on-disk after every transition, including second-screen edit + sync, item #7 of that gate).
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—**
Notes: Function **#27, Phase 6** — re-enabled **last** (thinnest display + double-init culprit). **Three things must be fixed before re-enable:** (1) the `setActiveUniverse` Display-Not-Domain violation at `SecondScreenPage.svelte:923` → read-only `get_active_universe` (part of MIG-079 single-owner activation); (2) Rule-8 recompute on companion load → read persisted; (3) hardcoded English + missing right-click menu. Depends on Federation (#26) for cUniverse dashboard paths. A `secondScreen` gate must be **created** (none exists today). All `unknown — verify in bring-up` items: per-load cost measurement; exact 15-locale key coverage; final right-click scope ruling.
