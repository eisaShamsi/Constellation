# Sky View Phase 3 — Interaction Features

## Already Implemented (No work needed)
- Context menu (open, focus, pin, hide)
- Focus mode with depth slider (1-4 hops BFS)
- Local graph mode (Space bar toggle)
- Pin & hide with indicators
- Keyboard shortcuts: Ctrl+F (search), Space (local graph), Escape (dismiss)
- Double-click zoom to node
- Search-to-highlight (dims non-matches)

## New Features to Implement

### 1. Layout Modes + Ctrl+L Cycling
**Files:** `forceWorker.ts`, `graphEngine.ts`, `GraphMindView.svelte`

Add 3 layout modes switchable via Ctrl+L:
- **Organic** (current) — force-directed, clusters emerge naturally
- **Hierarchical** — DAG top-down layout using depth-first ordering from MOC nodes
- **Temporal** — nodes arranged on horizontal time axis by creation date

Implementation:
- Add `layoutMode: 'organic' | 'hierarchical' | 'temporal'` to EngineConfig
- Hierarchical: compute topological sort from MOC nodes, assign y-levels, spread x within levels
- Temporal: use note creation dates (passed via StarNode), map to x-axis, y scattered to avoid overlap
- Both computed inside the Worker as alternative position arrays
- Animated transition: lerp between current positions and target positions over 20 frames in the engine's draw loop
- Ctrl+L in Svelte wrapper cycles mode and calls `engine.setLayoutMode()`

### 2. Directional Filter in Focus Mode
**Files:** `graphEngine.ts`, `GraphMindView.svelte`

Add direction toggle to focus bar: All / Incoming / Outgoing
- Modify `rebuildFocusSet()` to filter by link direction
- Need directed neighbor maps (currently undirected): add `incomingMap` and `outgoingMap`
- UI: 3 small buttons in the focus bar (↔ ← →)

### 3. Navigation Breadcrumb Trail
**Files:** `graphEngine.ts`, `GraphMindView.svelte`

As user clicks through nodes in focus mode, record the path:
- Array of `{ id, name }` shown as horizontal breadcrumb at bottom of viewport
- Clicking a breadcrumb item re-centers on that node
- Back button pops the last item
- Clear when exiting focus mode
- Max 8 items visible (scroll if more)

### 4. Temporal Replay Scrubber
**Files:** `graphEngine.ts`, `GraphMindView.svelte`, StarNode needs `createdAt` field

Timeline slider at bottom of viewport:
- Range from earliest to latest note creation date
- Dragging left hides notes created after that date
- Nodes fade in/out as scrubber moves
- Shows date label at scrubber position
- Requires: pass `createdAt` from file metadata through StarNode

**Dependency:** StarNode currently has no date field. Need to add `createdAt?: number` (epoch ms) populated from file system metadata in Rust `scan_library_links`.

## Execution Order
1. Layout modes + Ctrl+L (biggest visual impact)
2. Directional filter (small addition to existing focus mode)
3. Breadcrumb trail (enhances navigation)
4. Temporal replay (requires Rust backend change for dates)

## Architecture Rules (Unchanged)
- All position computation in Worker (Layer 3)
- All rendering in GraphEngine (Layer 2) — plain variables only
- Svelte wrapper (Layer 1) only manages UI controls
- Hover NEVER reaches the Worker
