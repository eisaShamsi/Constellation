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
- `a8a24d9` — Fixed pointer arrow (ID comparison, not object identity) + arrows on ALL links + zoom 3x
- Pointer arrow: CONFIRMED WORKING (amber triangle above current result)
- Direction arrows: CONFIRMED WORKING ("like a highway" — user approved)
- Category badges: CONFIRMED WORKING (T/P/S badges enlarged 2x)
- Settings persistence: CONFIRMED WORKING

### Critical Bug Fix: Missing Graph Links
- `scan_library_links` used file stem as source_name for ALL files
- `collect_library_notes` used frontmatter title for canonical files
- Mismatch meant `buildSkyData` could never match link sources to nodes
- Result: ZERO links in Sky View and Sight for canonical libraries
- Fix: `scan_library_links` now uses frontmatter title for canonical files
- Impact: ALL graph visualizations now show correct link connections

### Structured Search in Sight
- Wired `parseSearchQuery` → `constellationSearch` for advanced operators
- `links to [[X]]`, `orphans`, `mutual`, cognitive types all work
- Target node added to match set so connecting links stay visible
- Search-highlighted links: green (inward→target), red (outward←target), 3× bold

### Constellation Map Enhancements
- Full search engine (same as Sight: structured + free text + semantic)
- Search results sidebar with category badges + resizable drag handle
- Fit to Screen button (reset to root)
- Settings panel (arc opacity, depth limit, persistent)
- Zoom + pan (D3 zoom behavior)
- SVG hover tooltips showing "Name (Type)"
- Double-click sidebar result to open note
- Hover respects search dimming state
- Library color legend added
- Legend enlarged 2x
- Error display localized

### Distinct Category Badges (All Operators)
- LT=links to, LF=links from, LA=links all, LB=links between
- ⇄=mutual, M=mentions, ∅=orphan
- T=title, C=content, #=tag, P=property, S=semantic, W=wikilink
- Backend updated to return distinct match_types per operator

### Critical Bug Fix: Missing Graph Links
- `scan_library_links` used file stem for canonical files
- `collect_library_notes` used frontmatter title
- Mismatch → zero links in graph for canonical libraries
- Fix: both now use frontmatter title for canonical files

### Settings Restructure
- "Keyboard" → "Hotkeys"
- "Features" → "Plug-Ins"
- New "Templates" tab (future-ready for multiple templates)
- Constellation Map: OFF by default (opt-in plug-in)
- Constellation Sight: ON by default
- Ribbon + command palette gated by plug-in flags
- All 15 locales updated

### Notes Navigator Fix
- Root cause: duplicate note paths from Rust backend → Svelte each_key_duplicate crash
- Fix: deduplicate by path + parallel library loading with timeouts

### Settings Restructure (Final)
- Keyboard → Hotkeys
- Features → Core Plug-Ins (renamed from Plug-Ins)
- New Templates tab
- All plug-in toggles wired to ribbon buttons:
  Sky View, Constellation Sight, Constellation Map, OrgChart,
  Index, Daily Notes, AI Skills, Notes Navigator, Second Screen
- Global Tasks ribbon button removed
- Folders/By Stage/By Topic lens dropdown removed
- Theme toggle moved from ribbon to Settings > Appearance
- Import Notes moved from ribbon to Settings > Universe

### Remaining
- Theme Settings system (plan approved, not yet implemented)
- Map help documentation file
- Help file updates for settings restructure
