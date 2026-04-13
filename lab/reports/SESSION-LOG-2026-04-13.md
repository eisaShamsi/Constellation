# Session Log — 2026-04-13

## Phase: Sight2 Complete Redesign

### Key Design Decisions
1. **Dropped Louvain community detection** — communities were algorithmic noise, not cognitive knowledge. User's own libraries are more meaningful groupings.
2. **Nodes colored by library** — user's organization, not algorithm's guess.
3. **Neighborhood highlight** — single-click a node to see all connections (nervous system metaphor). More useful than static community boundaries.
4. **All links solid** — dashed lines raised questions ("what does this shape mean?"). Confidence shown by thickness instead.
5. **Multiple direction arrows** on typed links — 3 arrows on long links, 2 on medium, 1 on short.
6. **Simplicity principle** — "The simplicity should come from understanding what you see at first sight. NOT to raise more questions."

### Commits
- `5b11d11` — Sight2: Sector wedge communities + full search integration + legend
- `3f40403` — Sight2: Fix 5 test findings — sectors, badges, arrows, link controls, perf
- `e6a54e5` — Revert hexagon/boundary experiments — back to 3f40403
- `04f0308` — Sight2: Restore exact 3f40403 layout
- `289e4e5` — Sight2: Drop Louvain — library colors + neighborhood highlight
- `d87b804` — Sight2: Bigger badges, pointer arrow, persistent settings, visible arrows
- `d6f2f8e` — Sight2: Solid links + multiple direction arrows + bigger pointer

### Boundary Issue Investigation
- Spent significant time debugging nodes/links escaping outer ring
- Root cause: D3 forceCollide pushed nodes outward after positioning, no post-simulation clamp
- Research: d3-force-limit, d3-force-boundary, Mbostock bounded force gist
- Solution: custom boundary force registered as last D3 force + jitter reduction
- Multiple failed approaches: clip(), render-time skipping, chord-based layout
- Final state: 4-ring layout with boundary force for collision avoidance

### What Works Now
- 4-ring gravity-well layout (centrality → distance from center)
- Library-based angular sectors and node colors
- Search: 6 scopes, syntax chips, history, category badges (T/C/#/P/S), pointer arrow
- Neighborhood highlight: single-click shows connections, double-click opens note
- Settings: link stroke/opacity/arrow sliders, persist across remounts
- Legend: node sizes, library colors, link types, confidence=thickness, arrows=direction
- All links solid, multiple direction arrows for typed links

### Open Items
- Pointer arrow for current search result — may need debugging if not appearing
- SightPanel (insight/analytics sidebar) — pending
- Help file updates for Sight changes
