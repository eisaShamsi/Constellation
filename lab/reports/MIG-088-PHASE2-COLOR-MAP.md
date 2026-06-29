# MIG-088 Phase 2 — Semantic colour map (wf_1c77386c-019)

## maturity  → proposed `--maturity-{state}`

**seed**
  - Sky (LocalSkyView/skyPalette) — `src/lib/graph/skyPalette.ts:91` — 0x999999 (hex #999999) (--skyview-maturity-seed)
  - StyleSetter (Sky preview) — `src/lib/components/StyleSetter.svelte:1547-1550` — var(--skyview-maturity-seed) fallback #999999 in palette (--skyview-maturity-seed)
  - Map (ConstellationMap maturity mode) — `src/lib/components/ConstellationMap.svelte:141` — #d1d5db (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:102` — #9ca3af (none (hardcoded))
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:not_present` — none (none)
  - ⚠️ CONFLICT: 4 distinct colours

**sapling**
  - file-tree (NavFileItem/FileTree maturity border) — `src/lib/components/FileTree.svelte:216` — #4ade80 (none (hardcoded))
  - tabs (+layout tab maturity dot) — `src/routes/+layout.svelte:8737` — #4ade80 (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:102` — #4ade80 (none (hardcoded))
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:not_used` — none (none)
  - Sky (LocalSkyView/skyPalette) — `src/lib/graph/skyPalette.ts:92` — 0x4ade80 (hex #4ade80) (--skyview-maturity-sapling)
  - StyleSetter (Sky preview) — `src/lib/components/StyleSetter.svelte:1547` — var(--skyview-maturity-sapling, #4ade80) (--skyview-maturity-sapling)
  - Map (ConstellationMap maturity mode) — `src/lib/components/ConstellationMap.svelte:142` — #86efac (none (hardcoded))
  - ⚠️ CONFLICT: 4 distinct colours

**evergreen**
  - file-tree (NavFileItem/FileTree maturity border) — `src/lib/components/FileTree.svelte:217` — #16a34a (none (hardcoded))
  - tabs (+layout tab maturity dot) — `src/routes/+layout.svelte:8738` — #16a34a (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:102` — #16a34a (none (hardcoded))
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:not_used` — none (none)
  - Sky (LocalSkyView/skyPalette) — `src/lib/graph/skyPalette.ts:93` — 0x16a34a (hex #16a34a) (--skyview-maturity-evergreen)
  - StyleSetter (Sky preview) — `src/lib/components/StyleSetter.svelte:1548` — var(--skyview-maturity-evergreen, #16a34a) (--skyview-maturity-evergreen)
  - Map (ConstellationMap maturity mode) — `src/lib/components/ConstellationMap.svelte:143` — #16a34a (none (hardcoded))
  - ⚠️ CONFLICT: 3 distinct colours

**canonical**
  - file-tree (NavFileItem/FileTree maturity border) — `src/lib/components/FileTree.svelte:218` — #f59e0b (none (hardcoded))
  - tabs (+layout tab maturity dot) — `src/routes/+layout.svelte:8739` — #f59e0b (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:102` — #f59e0b (none (hardcoded))
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:not_used` — none (none)
  - Sky (LocalSkyView/skyPalette) — `src/lib/graph/skyPalette.ts:94` — 0xf59e0b (hex #f59e0b) (--skyview-maturity-canonical)
  - StyleSetter (Sky preview) — `src/lib/components/StyleSetter.svelte:1549` — var(--skyview-maturity-canonical, #f59e0b) (--skyview-maturity-canonical)
  - Map (ConstellationMap maturity mode) — `src/lib/components/ConstellationMap.svelte:144` — #f59e0b (none (hardcoded))
  - ⚠️ CONFLICT: 3 distinct colours

**wilting**
  - file-tree (NavFileItem/FileTree maturity border) — `src/lib/components/FileTree.svelte:219` — rgba(22, 163, 74, 0.4) (none (hardcoded))
  - tabs (+layout tab maturity dot) — `src/routes/+layout.svelte:8740` — #16a34a with opacity: 0.4 (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:102` — #16a34a80 (RGBA with 50% alpha) (none (hardcoded))
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:not_used` — none (none)
  - Sky (LocalSkyView/skyPalette) — `src/lib/graph/skyPalette.ts:95` — 0x16a34a (hex #16a34a, opaque) (--skyview-maturity-wilting)
  - StyleSetter (Sky preview) — `src/lib/components/StyleSetter.svelte:1550` — var(--skyview-maturity-wilting, #16a34a) (--skyview-maturity-wilting)
  - Map (ConstellationMap maturity mode) — `src/lib/components/ConstellationMap.svelte:145` — #a3e635 (none (hardcoded))
  - ⚠️ CONFLICT: 6 distinct colours

_notes:_ CRITICAL CONFLICTS IDENTIFIED: The maturity states show major colour conflicts across surfaces that must be resolved during consolidation. Most severe: Wilting state uses #16a34a (opaque) in Sky (skyPalette), but FileTree/Tabs use rgba(22,163,74,0.4) (faded), Map uses #a3e635 (lime), and Inspector360 uses #16a34a80 (50% alpha). Sapling shows #4ade80 everywhere EXCEPT Map which uses #86efac. Seed is uncontrolled across surfaces: Sky palette #999999, Map #d1d5db, Inspector360 #9ca3af. The consolidation must decide a canonical colour for each state (recommend dark mode compatible palette), then apply uniformly. Sky already has CSS vars wired (--skyview-maturity-*), but FileTree, Tabs, Inspector360, and Map all use hardcoded hex values with no existing CSS var infrastructure. KnowledgeHealth does not use maturity colours at all. StyleSetter preview classes (.ssn-mat-*) use CSS vars only as fallbacks in borders, not for primary display.


## confidence  → proposed `--confidence-{state}`

**hypothesis**
  - ConfidencePicker — `src/lib/components/ConfidencePicker.svelte:118` — color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent) (none (color-mix expression))
  - KnowledgeHealth confidence bars — `src/lib/components/KnowledgeHealthDashboard.svelte:119` — #94a3b8 (none (hardcoded))
  - backlinks/outgoing traversal chips — `src/lib/components/BacklinksPanel.svelte:401` — color-mix(in srgb, var(--interactive-accent, #7c3aed) 14%, transparent) (none (color-mix expression))
  - ⚠️ CONFLICT: 2 distinct colours

**evidence**
  - ConfidencePicker — `src/lib/components/ConfidencePicker.svelte:119` — color-mix(in srgb, var(--interactive-accent, #7c3aed) 40%, transparent) (none (color-mix expression))
  - KnowledgeHealth confidence bars — `src/lib/components/KnowledgeHealthDashboard.svelte:119` — #3b82f6 (none (hardcoded))
  - ⚠️ CONFLICT: 2 distinct colours

**established**
  - ConfidencePicker — `src/lib/components/ConfidencePicker.svelte:120` — var(--interactive-accent, #7c3aed) (--interactive-accent)
  - KnowledgeHealth confidence bars — `src/lib/components/KnowledgeHealthDashboard.svelte:119` — #16a34a (none (hardcoded))
  - backlinks/outgoing traversal chips tier-established — `src/lib/components/BacklinksPanel.svelte:414` — color-mix(in srgb, var(--interactive-accent, #7c3aed) 26%, transparent) (none (color-mix expression))
  - ⚠️ CONFLICT: 3 distinct colours

**contested**
  - ConfidencePicker — `src/lib/components/ConfidencePicker.svelte:121` — #d97706 (none (hardcoded))
  - KnowledgeHealth confidence bars — `src/lib/components/KnowledgeHealthDashboard.svelte:119` — #ef4444 (none (hardcoded))
  - backlinks/outgoing traversal chips tier-stale — `src/lib/components/BacklinksPanel.svelte:423` — color-mix(in srgb, #d97706 14%, transparent) (none (color-mix expression))
  - ⚠️ CONFLICT: 3 distinct colours

_notes:_ CRITICAL CONFLICT: ConfidencePicker and KnowledgeHealth use entirely different colour palettes for the same confidence states. ConfidencePicker relies on opacity variations of --interactive-accent (#7c3aed) for hypothesis/evidence/established, while KnowledgeHealth uses hard-coded state-specific colours (#94a3b8 for hypothesis, #3b82f6 for evidence, #16a34a for established). Both systems also disagree on contested (#d97706 vs #ef4444). The Sight v6 mini-domes use alpha-opacity encoding rather than colour. Consolidation must select ONE canonical palette and update all surfaces to match byte-identically with the new shared vars.


## origin  → proposed `--origin-{state}`

**received**
  - Inspector360 — `src/lib/components/Inspector360.svelte:105` — #4A9EFF (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:357` — ORIGIN_COLORS[data.origin_type] → #4A9EFF (none (hardcoded in ORIGIN_COLORS object))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:29` — #4A9EFF (none (hardcoded))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:51` — originColor(chain.origin_type) → #4A9EFF (none (hardcoded in originColor function))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:69` — #4A9EFF (for ancestor with external_source) (none (hardcoded))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:120` — #4A9EFF (none (hardcoded in .prov-external-tag color))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:121` — #4A9EFF40 (with 25% alpha) (none (hardcoded in .prov-external-tag border))
  - ⚠️ CONFLICT: 5 distinct colours

**discovered**
  - Inspector360 — `src/lib/components/Inspector360.svelte:105` — #FFB347 (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:357` — ORIGIN_COLORS[data.origin_type] → #FFB347 (none (hardcoded in ORIGIN_COLORS object))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:29` — #FFB347 (none (hardcoded))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:51` — originColor(chain.origin_type) → #FFB347 (none (hardcoded in originColor function))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:69` — #FFB347 (for ancestor without external_source) (none (hardcoded))
  - ⚠️ CONFLICT: 4 distinct colours

**mixed**
  - Inspector360 — `src/lib/components/Inspector360.svelte:105` — #A78BFA (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:357` — ORIGIN_COLORS[data.origin_type] → #A78BFA (none (hardcoded in ORIGIN_COLORS object))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:29` — #A78BFA (none (hardcoded))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:51` — originColor(chain.origin_type) → #A78BFA (none (hardcoded in originColor function))
  - ⚠️ CONFLICT: 3 distinct colours

**none**
  - Inspector360 — `src/lib/components/Inspector360.svelte:105` — #9ca3af (none (hardcoded))
  - Inspector360 — `src/lib/components/Inspector360.svelte:357` — ORIGIN_COLORS[data.origin_type] ?? '#999' → #9ca3af (none (hardcoded in ORIGIN_COLORS object))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:29` — #9ca3af (none (hardcoded))
  - Provenance panel — `src/lib/components/ProvenancePanel.svelte:51` — originColor(chain.origin_type) → #9ca3af (none (hardcoded in originColor function))
  - ⚠️ CONFLICT: 3 distinct colours

_notes:_ COLOUR CONFLICTS: None detected. All four origin states (received, discovered, mixed, none) use IDENTICAL hex values across both surfaces (Inspector360 and Provenance panel), so consolidation will achieve byte-perfect colour consistency. Inspector360 line 357 has a fallback pattern `?? '#999'` but the ORIGIN_COLORS object itself uses #9ca3af for none. Provenance panel's external-source tag (line 120-121) uses #4A9EFF (received colour) for all external sources regardless of origin_type. No existing CSS variables found for origin colours in either component or StyleSetter (checked StyleSetter thoroughly, lines 1-500+). Glow effects for Sky View exist (--skyview-glow-received #4a9eff, --skyview-glow-discovered #ffb347 in StyleSetter line 414-415) but those are visually distinct from the origin-state dots in Provenance/Inspector360 and serve the graph node glow, not the origin badges themselves.


## stage  → proposed `--stage-{state}`

**spark**
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:125-175` — #a78bfa (none (hardcoded))
  - note stage badge (PropertyEditor/NotePane) — `src/lib/components/NotePane.svelte:1221-1476` — var(--text-muted) [via parent --text-muted] (none (uses --text-muted for all stages))
  - ⚠️ CONFLICT: 2 distinct colours

**birth**
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:125-175` — #94a3b8 (none (hardcoded))
  - note stage badge (PropertyEditor/NotePane) — `src/lib/components/NotePane.svelte:1221-1476` — var(--text-muted) [via parent --text-muted] (none (uses --text-muted for all stages))
  - ⚠️ CONFLICT: 2 distinct colours

**growth**
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:125-175` — #16a34a (none (hardcoded))
  - note stage badge (PropertyEditor/NotePane) — `src/lib/components/NotePane.svelte:1221-1476` — var(--text-muted) [via parent --text-muted] (none (uses --text-muted for all stages))
  - ⚠️ CONFLICT: 2 distinct colours

**maturity**
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:125-175` — #7c3aed (none (hardcoded))
  - note stage badge (PropertyEditor/NotePane) — `src/lib/components/NotePane.svelte:1221-1476` — var(--text-muted) [via parent --text-muted] (none (uses --text-muted for all stages))
  - ⚠️ CONFLICT: 2 distinct colours

**dormancy**
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:125-175` — #f59e0b (none (hardcoded))
  - note stage badge (PropertyEditor/NotePane) — `src/lib/components/NotePane.svelte:1221-1476` — var(--text-muted) [via parent --text-muted] (none (uses --text-muted for all stages))
  - ⚠️ CONFLICT: 2 distinct colours

**archival**
  - KnowledgeHealth — `src/lib/components/KnowledgeHealthDashboard.svelte:125-175` — #ef4444 (none (hardcoded))
  - note stage badge (PropertyEditor/NotePane) — `src/lib/components/NotePane.svelte:1221-1476` — var(--text-muted) [via parent --text-muted] (none (uses --text-muted for all stages))
  - ⚠️ CONFLICT: 2 distinct colours

_notes:_ THREE colour sets exist for lifecycle stages across the codebase: (1) KnowledgeHealth hardcoded stageColors in .svelte (2) STAGE_META reference in store.ts with different link-stage colours (3) Note badge uses generic --text-muted with no stage-specific colour. For wiring, KnowledgeHealth's hardcoded palette (spark #a78bfa, birth #94a3b8, growth #16a34a, maturity #7c3aed, dormancy #f59e0b, archival #ef4444) should be the fallback values. Note stage badge currently has NO stage-specific colour — it renders text in --text-muted for all stages, so the proposed --stage-{state} vars would be NEW styling for that surface, not a consolidation. Recommend keeping KnowledgeHealth's palette as the canonical set and wiring both surfaces to shared --stage-{state} vars.


## match-category — Search-match / category states: title, content, tag, wikilink, property, semantic, structured  → proposed `--match-category-{state}`
- already-controlled vars: --skyview-badge-title, --skyview-badge-content, --skyview-badge-tag, --skyview-badge-property, --skyview-badge-wikilink, --skyview-badge-semantic, --skyview-badge-structured

**title**
  - editor-search highlight (NotePane) — `src/lib/components/NotePane.svelte:504` — #3b82f6 (none (hardcoded))
  - Sky badges (skyPalette) — `src/lib/graph/skyPalette.ts:108` — 0x3b82f6 (both dark/light) (--skyview-badge-title)
  - OrgChart category badges — `src/lib/components/OrgChart.svelte:1442` — #3b82f6 (none (hardcoded))
  - ConstellationMap search categories (CAT_COLORS) — `src/lib/components/ConstellationMap.svelte:81` — #3b82f6 (none (hardcoded))
  - ConstellationSight2 search categories (CAT_COLORS) — `src/lib/components/ConstellationSight2.svelte:90` — #3b82f6 (none (hardcoded))
  - SearchHub category badges — `src/lib/components/SearchHub.svelte:85` — #3b82f6 (none (hardcoded))
  - StyleSetter preview (Sky badges) — `src/lib/components/StyleSetter.svelte:1565` — var(--skyview-badge-title, #3b82f6) (--skyview-badge-title)
  - ⚠️ CONFLICT: 3 distinct colours

**content**
  - editor-search highlight (NotePane) — `src/lib/components/NotePane.svelte:505` — #16a34a (none (hardcoded))
  - Sky badges (skyPalette) — `src/lib/graph/skyPalette.ts:109` — 0x16a34a (both dark/light) (--skyview-badge-content)
  - OrgChart category badges — `src/lib/components/OrgChart.svelte:1443` — #16a34a (none (hardcoded))
  - ConstellationMap search categories (CAT_COLORS) — `src/lib/components/ConstellationMap.svelte:81` — #16a34a (none (hardcoded))
  - ConstellationSight2 search categories (CAT_COLORS) — `src/lib/components/ConstellationSight2.svelte:90` — #16a34a (none (hardcoded))
  - SearchHub category badges — `src/lib/components/SearchHub.svelte:85` — #16a34a (none (hardcoded))
  - StyleSetter preview (Sky badges) — `src/lib/components/StyleSetter.svelte:1566` — var(--skyview-badge-content, #16a34a) (--skyview-badge-content)
  - ⚠️ CONFLICT: 3 distinct colours

**tag**
  - editor-search highlight (NotePane) — `src/lib/components/NotePane.svelte:506` — #f472b6 (none (hardcoded))
  - Sky badges (skyPalette) — `src/lib/graph/skyPalette.ts:110` — 0xf472b6 (both dark/light) (--skyview-badge-tag)
  - OrgChart category badges — `src/lib/components/OrgChart.svelte:1444` — #f472b6 (none (hardcoded))
  - ConstellationMap search categories (CAT_COLORS) — `src/lib/components/ConstellationMap.svelte:81` — #f472b6 (none (hardcoded))
  - ConstellationSight2 search categories (CAT_COLORS) — `src/lib/components/ConstellationSight2.svelte:90` — #f472b6 (none (hardcoded))
  - SearchHub category badges — `src/lib/components/SearchHub.svelte:85` — #f472b6 (none (hardcoded))
  - StyleSetter preview (Sky badges) — `src/lib/components/StyleSetter.svelte:1567` — var(--skyview-badge-tag, #f472b6) (--skyview-badge-tag)
  - ⚠️ CONFLICT: 3 distinct colours

**property**
  - editor-search highlight (NotePane) — `src/lib/components/NotePane.svelte:507` — #f59e0b (none (hardcoded))
  - Sky badges (skyPalette) — `src/lib/graph/skyPalette.ts:111` — 0xf59e0b (both dark/light) (--skyview-badge-property)
  - OrgChart category badges — `src/lib/components/OrgChart.svelte:1446` — #f59e0b (none (hardcoded))
  - ConstellationMap search categories (CAT_COLORS) — `src/lib/components/ConstellationMap.svelte:81` — #f59e0b (none (hardcoded))
  - ConstellationSight2 search categories (CAT_COLORS) — `src/lib/components/ConstellationSight2.svelte:90` — #f59e0b (none (hardcoded))
  - SearchHub category badges — `src/lib/components/SearchHub.svelte:85` — #f59e0b (none (hardcoded))
  - StyleSetter preview (Sky badges) — `src/lib/components/StyleSetter.svelte:1568` — var(--skyview-badge-property, #f59e0b) (--skyview-badge-property)
  - ⚠️ CONFLICT: 3 distinct colours

**wikilink**
  - editor-search highlight (NotePane) — `src/lib/components/NotePane.svelte:508` — #60a5fa (none (hardcoded))
  - Sky badges (skyPalette) — `src/lib/graph/skyPalette.ts:112` — 0x60a5fa (both dark/light) (--skyview-badge-wikilink)
  - OrgChart category badges — `src/lib/components/OrgChart.svelte:1445` — #60a5fa (none (hardcoded))
  - ConstellationMap search categories (CAT_COLORS) — `src/lib/components/ConstellationMap.svelte:81` — #94a3b8 (mapped as 'W') (none (hardcoded))
  - ConstellationSight2 search categories (CAT_COLORS) — `src/lib/components/ConstellationSight2.svelte:90` — #94a3b8 (mapped as 'W') (none (hardcoded))
  - SearchHub category badges — `src/lib/components/SearchHub.svelte:85` — #60a5fa (none (hardcoded))
  - StyleSetter preview (Sky badges) — `src/lib/components/StyleSetter.svelte:1569` — var(--skyview-badge-wikilink, #60a5fa) (--skyview-badge-wikilink)
  - ⚠️ CONFLICT: 4 distinct colours

**semantic**
  - Sky badges (skyPalette) — `src/lib/graph/skyPalette.ts:113` — 0x7c3aed (both dark/light) (--skyview-badge-semantic)
  - ConstellationMap search categories (CAT_COLORS) — `src/lib/components/ConstellationMap.svelte:81` — #7c3aed (mapped as 'S') (none (hardcoded))
  - ConstellationSight2 search categories (CAT_COLORS) — `src/lib/components/ConstellationSight2.svelte:90` — #7c3aed (mapped as 'S') (none (hardcoded))
  - SearchHub category badges — `src/lib/components/SearchHub.svelte:85` — #7c3aed (none (hardcoded))
  - StyleSetter preview (Sky badges) — `src/lib/components/StyleSetter.svelte:1570` — var(--skyview-badge-semantic, #7c3aed) (--skyview-badge-semantic)
  - ⚠️ CONFLICT: 4 distinct colours

**structured**
  - Sky badges (skyPalette) — `src/lib/graph/skyPalette.ts:114` — 0x94a3b8 (both dark/light) (--skyview-badge-structured)
  - OrgChart category badges (via 'content') — `src/lib/components/OrgChart.svelte:920` — #16a34a (mapped same as content) (none (hardcoded))
  - SearchHub category badges — `src/lib/components/SearchHub.svelte:87` — #ef4444 (none (hardcoded))
  - StyleSetter preview (Sky badges) — `src/lib/components/StyleSetter.svelte:1571` — var(--skyview-badge-structured, #94a3b8) (--skyview-badge-structured)
  - ⚠️ CONFLICT: 4 distinct colours

_notes:_ CONFLICT DETECTED on wikilink state: ConstellationMap and ConstellationSight2 use #94a3b8 for wikilink badge (W), while editor search highlight (NotePane) and SearchHub use #60a5fa. OrgChart uses #60a5fa. Sky badges use #60a5fa. CONFLICT on structured state: SearchHub uses #ef4444, while Sky and StyleSetter fallback use #94a3b8. Sky badges already controlled by --skyview-badge-* vars (7 total). Editor search uses hardcoded hex. Map/Sight CAT_COLORS are hardcoded in JS. SearchHub uses hardcoded hex. OrgChart uses hardcoded hex. Recommend wikilink resolution toward #60a5fa (majority) and structured toward #94a3b8 (Sky/structured intent).

