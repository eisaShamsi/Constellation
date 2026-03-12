# Plan: Graph View Controls Panel

## Goal
Add an Obsidian-style controls panel to the existing Graph View with **Filters**, **Groups**, **Display**, and **Forces** sections — matching the Obsidian screenshot the user provided.

## Current State
- `GraphView.svelte` (694 lines): D3 force simulation on Canvas with vault clustering, legend, zoom/pan, drag, tooltips
- Hardcoded force values: charge=-80, distance=60, collision=8, clusterStrength=0.05, alphaDecay=0.06
- Hardcoded display values: node radius 4-14, link width 0.6-2
- No search filter, no groups, no animate toggle, no user-configurable controls
- Legend drawn on canvas (vault color dots with counts)

## Implementation

### Step 1: Add Controls State + HTML Panel to `GraphView.svelte`

Add a collapsible HTML side panel (left, ~280px) overlaying the canvas. Four collapsible `<details>` sections:

**Filters:**
- `filterQuery: string` — search input (filters nodes by name/path)
- `showTags: boolean` (default: off) — toggle tag nodes
- `showAttachments: boolean` (default: off) — toggle attachments
- `existingOnly: boolean` (default: off) — hide ghost/unresolved nodes
- `showOrphans: boolean` (default: on) — show unlinked notes

**Groups:**
- `groups: Array<{ query: string; color: string }>` — color-coded search groups
- "New group" button adds a row with query input + color picker + delete button
- Nodes matching a group query get that group's color override

**Display:**
- `showArrows: boolean` (default: off) — draw arrowheads on links
- `textFadeThreshold: number` (range 0–5, default: 1.5) — zoom level for label visibility
- `nodeSize: number` (range 1–10, default: 4) — node scale multiplier
- `linkThickness: number` (range 1–5, default: 1) — link width multiplier
- `animate: boolean` (default: true) — toggle force animation; when off, simulation runs to completion instantly

**Forces:**
- `centerForce: number` (range 0–1, default: 0.5)
- `repelForce: number` (range 0–300, default: 80)
- `linkForce: number` (range 0–1, default: 1)
- `linkDistance: number` (range 10–500, default: 60)

Panel features:
- Close (×) and Reset (↻) buttons in the header
- Toggle button (gear icon) on the graph toolbar to show/hide
- Scrollable, styled to match the app theme

### Step 2: Wire Controls to D3 Simulation

- **Filters**: Applied in `renderGraph()` when building nodeData/linkData
  - `filterQuery` → filter nodes by name/path substring match
  - `showOrphans` → filter nodes with linkCount === 0
  - `existingOnly` → would filter ghost nodes (currently graph only shows linked notes, so minimal impact)
- **Groups**: In `getNodeColor()`, check if node matches any group query; first match wins
- **Display**:
  - `showArrows` → draw arrowhead triangles at link endpoints in `draw()`
  - `textFadeThreshold` → show labels when `currentTransform.k >= threshold`
  - `nodeSize` → multiply `getNodeRadius()` result
  - `linkThickness` → multiply link lineWidth
  - `animate` → when toggled off, tick simulation to alpha=0 instantly
- **Forces**: Use `$effect` to dynamically update `simulation.force(...)` parameters without full re-render, then reheat with `simulation.alpha(0.3).restart()`

### Step 3: i18n (15 locale files)
Add keys under `graphView.controls`:
```
filters, search, tags, attachments, existingOnly, orphans,
groups, newGroup,
display, arrows, textFade, nodeSize, linkThickness, animate,
forces, centerForce, repelForce, linkForce, linkDistance,
reset
```

### Step 4: Help Docs
Create/update `docs/help.notesconstellation.com/Graph View/Graph View.md`

## Files to Modify
1. `src/lib/components/GraphView.svelte` — Controls panel UI + wiring (main change)
2. `src/lib/i18n/*.json` (15 files) — Graph control translations
3. `docs/help.notesconstellation.com/Graph View/Graph View.md` — Help documentation
4. `src/routes/+layout.svelte` — Add toggle button for controls panel in graph header
