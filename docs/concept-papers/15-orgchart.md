# 15 — OrgChart (Concept Paper)

> Attaches to the Editor (the gate) as a **navigator**: it draws the Universe → Library → Folder → Note hierarchy and hands a clicked note back to the Editor to open. Root paper: [00-Constellation](00-Constellation-Core-Concept-Paper.md). This function is the live subject of **MIG-078** (persist the tree, Rule 8) — read §7 first.

## 1. Function in hand
The **OrgChart** — `src/lib/components/OrgChart.svelte`. Two surfaces in one component: a **sidebar tree** (embedded, structural, `read_library_tree`) and a **fullscreen visual org chart** (pannable/zoomable boxes with per-note metrics, `constellation_map_universe`). The Settings feature toggle and the fullscreen header both name it "OrgChart" / "Organization Chart" (`en.json` `orgChart.title`); the sidebar header reads `$t('orgChart.title')`.

## 2. Purpose
Show *where a piece of knowledge sits* in the structural hierarchy and let the user jump to it. It serves the **Connection** act at the structural (not semantic) layer: it answers *"how is my Universe organized, and where does this note live in it?"* — and it is a navigator into the Editor (click a note → it opens). It justifies itself as the one view that renders the four-level Constellation hierarchy (Universe → Library → Folder → Note) plus the federation layer (cUniverse) in a single picture; the file tree shows one library at a time, OrgChart shows the whole Universe.

## 3. What it is NOT
- **Not** the semantic graph (that is Sky View's bubbles / the Map's sunburst — see [feedback_skyview_vs_map_vocabulary]). OrgChart draws **containment**, not links.
- **Not** the file tree — it is read-mostly navigation across the *whole* Universe, not the per-library inline-edit tree.
- **Not** an editor or a derivation engine — it computes no knowledge; it displays structure and dispatches a note-open.
- **Not** the source of truth for tree mutations — move/rename/delete are performed by the **host** (`+layout`), never by OrgChart itself (it owns only local expand/collapse state).

## 4. Wiring
- **Inputs (IPC):** `constellation_map_universe(universeName, maxDepth:20)` → `MapNode` tree (fullscreen); `read_library_tree(path, maxDepth:20)` per library (sidebar); `getChildUniverses` + `read_child_universe_libraries` (federation). Reads stores: `$libraries`, `$appSettings`, `$dir`, `libraryColorMap`, `selectedPath` (`$bindable`).
- **Inputs (props):** `refreshKey` — bumped by the host after any move/delete/rename/create so the cached fullscreen tree reloads (MIG-077 A3-R3); `embedded` / `fullscreen` mode switches.
- **Outputs (drag-drop):** sidebar drag→drop calls `invoke('move_item', …)` then `loadData()` (the one place OrgChart writes to disk directly).
- **Outputs (callbacks):** `onNoteClick(path, name)` → host opens the note in the Editor; `onNodeMenuAction(action, target)` → host dispatcher `handleOrgNodeMenuAction` performs open / rename / move / add-tag / delete / reveal-in-tree / suggest-sources; `onClose`.
- **Consumers:** `+layout.svelte` is the sole host; `selectedPath` is shared with Sky View for cross-surface highlight. The fullscreen `MapNode` shape is shared with `ConstellationMap.svelte` (D3 sunburst).
- **Connection to the Editor (the gate):** OrgChart never edits — it dispatches `onNoteClick` and the host opens the note in the Editor. Every structural mutation it offers is routed back to the host's existing handlers (which fire `write_note`/`rename_item`/reindex), so the Editor's write-path invariants stay the single source of truth. OrgChart is downstream of the gate, not a second writer.

## 5. Right-click / context menu
- **Fullscreen mode: YES — shared `<ContextMenu>` + `buildContextMenu` (MIG-077 A3-R). Good — not hand-rolled.** `oncontextmenu` on each box/note-row → `handleContextMenu` → `getOrgNodeMenuItems` maps `node_type` to a `ContextTarget` and builds items via the shared builder. **Notes:** open, open-in-new-tab, rename, move, add-tag, copy-path, copy-name, reveal-in-tree, suggest-sources, delete. **Folders:** new-note/folder/base, toggle-expand, rename, move, reveal-in-tree, delete. **Libraries:** new-note/folder/base + toggle-expand only (management lives in the Library Manager). Expand/Collapse is OrgChart-local; every other action dispatches to the host. **Reveal-in-tree and Suggest-sources are effectively right-click-only** here.
- **Sidebar mode: NO context menu — GAP.** The embedded tree (lines ~1066–1217) supports left-click (open/toggle) and **drag-and-drop move only**; there is no `oncontextmenu` handler. A user in the sidebar cannot rename/delete/add-tag from OrgChart — they must switch surfaces. **Flag:** the sidebar tree should expose the same shared `buildContextMenu` set as fullscreen (one source of truth), or this asymmetry is documented as intentional in bring-up.

## 6. Multilingual
- **Good:** chrome flows through `$t()` (title, libraries, notes, reset, search placeholder, syntax chips, prev/next/clear, common.loading); `dir={isRTL?'rtl':'ltr'}` on both roots; per-node `dir={detectDir(node.name)}` and `dir="auto"` on labels/inputs/history so mixed-script note names render correctly; RTL chevron/flip handled via `isRTL`.
- **Hardcoded English — FLAG (fullscreen view):** the header stat `{note_count} {$t('secondScreen.dashboard.notes')} · {words} words` ("words" literal); per-box meta `"{n} notes"` and the `"w"` word-count suffix (`formatWordCount`); the **maturity chip + filter labels render the raw English keys** `seed / sapling / evergreen / canonical / wilting` (`{m}` printed directly, not `$t()`); aria-label `"Knowledge hierarchy"`; the Fit-to-screen `title="Fit to screen"`; the search category badges (`title`/`content`/`tag`…). These must be moved to `$t()` across all 15 locales (ar de en es fa fr he hi ja ko pt ru tr ur zh) and given native equivalents (per `feedback_full_localization_everything`) before re-enable.

## 7. Boot behavior
- **Runs at boot?** No — neither IPC fires until the user opens OrgChart. Sidebar `loadData()` runs on mount only when *not* fullscreen; fullscreen `loadFullscreenData()` fires on first open via `$effect`. Boot is not on its critical path.
- **Rule 8 status: ⚠️ RECOMPUTES-on-read — a known violation, actively being fixed by MIG-078.** `constellation_map_universe` (`src-tauri/src/map.rs:566`) **re-assembles the whole-Universe tree on every open**: even on the indexed path `build_tree` calls `fs::read_dir` on every directory, and the cUniverse/cold-index fallback (`collect_notes_recursive`) reads **every `.md` file's full content**. MIG-077 reduced the constant factor (read `note_meta` once via `load_note_records` instead of all files) but did **not** remove the read-time recomputation. The MIG-078 Architect doc names this the exact shape Rule 8 forbids and proposes Option B: a persisted `tree_node` adjacency list + `folder_stats` rollups maintained by write-path triggers (mirroring the `notes_fts` / `sky_nodes` exemplars).
- **Cost (measured, MIG-078 Architect §1):** indexed warm ≈ 1.4–1.5 s (`load_note_records` ~634 ms + `build_tree` readdir ~761 ms); indexed cold dominated by ~600 readdir/stat calls under AV; **fallback cold ≈ 40–60 s** (7,600 × `read_to_string`). The uncommitted regression that removed the incidental FS-cache warm-up can push a cold federated open past 2 minutes.

## 8. Flag / gate & bring-up position
- **Gate today:** `$appSettings.enabledFeatures?.orgChart` — the toolbar button and overlay are guarded by `enabledFeatures?.orgChart !== false` (`+layout.svelte:5198`). No new gate needed.
- **Bring-up phase:** **Phase 4 (visualization / navigation satellite)** — depends on the Editor (gate, for note-open), `$libraries` + the index (`note_meta`), and the federation IPCs. **Hard dependency: it must NOT be re-enabled on the current recompute-on-read path** — it should land behind / after MIG-078 (persisted tree) so opening is instant per Rule 8. Until then it is correctness-OK but budget-failing on a large Universe.

## 9. Budget
- **Boot budget:** zero — must add nothing to the boot critical path (does not today; keep it that way post-MIG-078: first back-fill runs after first paint, resumable, status-bar progress).
- **Interaction budget:** **open must be sub-second on a 7,600-note Universe** (the MIG-078 target). Pan/zoom/expand are pure client state and already cheap; search debounces and runs Rust-side (`constellationSearch`), never per-keystroke IPC on the typing path.
- **Regression guard:** measure cold + warm open on the 7,600-note Universe before/after any `map.rs` or OrgChart change; assert no `fs::read_to_string` fallback fires for an indexed library; assert a move/rename reflects without a full O(Universe) rebuild.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** the full Universe→Library→Folder→Note hierarchy (incl. cUniverse) renders; clicking a note opens it in the Editor; counts are correct.
- [ ] **Serves Constellation's core purpose:** structural Connection — it shows where knowledge sits, and routes back to the Editor without becoming a second writer.
- [ ] **Wires correctly to the Editor:** `onNoteClick` opens via the host; every mutation routes to host handlers (no direct disk write except the sidebar `move_item`, reviewed).
- [ ] **Right-click present + correct (shared, not hand-rolled):** fullscreen uses shared `buildContextMenu` ✓; **sidebar tree gains the same shared menu (or the asymmetry is signed off).**
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** maturity labels, `"notes"/"words"/"w"`, aria-labels, and "Fit to screen" moved to `$t()` with native equivalents; bidi/`detectDir` verified.
- [ ] **Within budget:** sub-second open on the 7,600-note Universe; no `read_to_string` fallback on the indexed path.
- [ ] **Obeys Rule 8:** reads a persisted, write-time-maintained tree (post-MIG-078) — no whole-Universe disk walk on open.
- [ ] **Holds its invariants:** aggregate correctness; federation included; rename/move/delete reflected without full rebuild; File-Over-App self-heal on external change.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (fails open-latency on large/federated Universe; recompute-on-read)** · Notes: The central bring-up issue is **Rule 8** — OrgChart recomputes the tree on every open (`constellation_map_universe` walks the Universe), which **MIG-078** is converting to a persisted `tree_node` + `folder_stats` maintained by write-path triggers; OrgChart should re-enable behind that work. Right-click is **shared/correct in fullscreen** but **missing in the sidebar tree** (gap). Several fullscreen strings are **hardcoded English** (maturity labels, "notes/words/w", "Fit to screen", "Knowledge hierarchy"). Unknown — verify in bring-up: whether empty (note-less) folders must appear in the Map (MIG-078 Option A′ caveat), and whether the sidebar context-menu gap is intentional.
