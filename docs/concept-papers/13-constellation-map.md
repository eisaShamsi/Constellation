# 13 — Constellation Map (Concept Paper)

> A satellite of the Editor (the gate, [01](01-Note-Editor.md)). Currently **force-disabled** (MIG-038, 2026-05-19) pending detachment into a standalone "Constellation Wings" plug-in. This paper is its re-enable gate.

## 1. Function in hand
The **Constellation Map** — `src/lib/components/ConstellationMap.svelte` (the dock 🗺️ button / command-palette "Constellation Map"). A radial **D3 sunburst** (`d3.partition()`) over the whole Universe's structure — Universe → cUniverse → Library → Folder → Note arcs, colored by maturity / stratum / library. **NOT bubbles** — Sky View has the PIXI bubbles; the Map has the D3 sunburst arcs. Backed by `src-tauri/src/map.rs` (`constellation_map_universe`, `constellation_map_data`).

## 2. Purpose
Give one **whole-Universe overview** of knowledge structure and density: where the mass of notes sits, which folders/libraries are heavy, and (via color) how mature or layered the knowledge is. It serves **Connection** — a survey instrument that surfaces the shape of the corpus so the mind can see clusters and gaps it would never notice note-by-note. Honest caveat: it is *adjacent* to the Editor's write path, not on it; its existence must be re-justified against Constraint-as-Design at re-enable, since Sky View (the bubble graph) and the file tree already answer overlapping "where is everything?" questions. If it cannot show something those two cannot, it doesn't earn its place.

## 3. What it is NOT
- **Not** Sky View — that's the PIXI bubble/link graph; this is the D3 sunburst hierarchy (never conflate the two).
- **Not** a single-note view (that's Inspector 360 / 360.3D); the Map is whole-Universe scope only.
- **Not** an editor — clicking a note arc *opens* it in the Editor; the Map never reads/writes note content itself.
- **Not** a write-time-maintained derived view today — it **recomputes** on open (see §7).

## 4. Wiring
- **Inputs (IPC):** `constellation_map_universe(universeName, maxDepth)` (the universe tree; `constellation_map_data` is the single-library variant). Reads `note_meta` (path, name, word_count, outgoing_links, modified, created), `note_aliases` (alias→canonical), and each cUniverse's own `search.db`. Search box reuses the shared engine: `universalSearch` / `constellationSearch` / `embedText` + `readSearchHistory`.
- **Inputs (props):** `universeName`, library color map, `initialData` (for the second-screen mini-map), `initialColorMode`.
- **Outputs (callbacks, no IPC writes):** `onNoteClick` (→ host opens the note tab), `onDrillDown`, `onColorModeChange`, `onClose`. The host (`+layout.svelte`) bridges these to `openNoteTab` and to the second screen via `emitMapCompanion`. The Map performs **no** note writes — File-Over-App preserved.
- **Consumers:** the second screen (companion mini-map via `emitMapCompanion`/listen); `OrgChart.svelte` shares the same `constellation_map_universe` backend.
- **Connection to the Editor (the gate):** loose and correct — the Map is a **read-only display**. It attaches by *opening* arcs into the Editor (`onNoteClick` → `openNoteTab`); it never re-implements save/load/edit (Additional-screens-are-displays rule). It does **not** subscribe to `note-saved` to refresh, so its tree can go stale after edits until reopened — note this for bring-up.

## 5. Right-click / context menu
- **None.** Grep for `oncontextmenu` / `contextmenu` / `<ContextMenu>` / `buildContextMenu` in `ConstellationMap.svelte` returns **zero matches**. Interaction is left-click (drill into folder / open note), double-click, hover tooltip, zoom/pan, and an Escape handler.
- **Gap — flag it.** Per the core paper §5 ("right-click should include every aspect of the app"), an arc *should* offer right-click actions (open in new tab / open in second screen / reveal in file tree / copy title / drill here). These are currently **unreachable**. When re-enabled, add them via the **shared** `buildContextMenu()` / `<ContextMenu>` (MIG-077) — **do not** hand-roll a menu. Exact item set: **unknown — verify in bring-up** against the file-tree/Sky-View menus for consistency.

## 6. Multilingual
- Chrome strings flow through `$t('constellationMap.*')` and the keys exist + are translated in **all 15 locales** (verified: `ar.json` → `title: "خريطة المعرفة"`, `maturity: "النضج"`, etc.); shared keys (`lens.*`, `searchHub.*`, `sightPanel.*`) reused for toolbar/search. `dir="auto"` is applied to library name, breadcrumb, search input, history, and result rows; resize math handles `$dir === 'rtl'`.
- **Hardcoded English — flag these (must move to `$t()` before re-enable):**
  - `Depth: {depthLimit}` settings label (line ~687) — not localized.
  - SVG `<title>` type labels `'cUniverse' | 'Library' | 'Folder' | 'Note'` (lines ~270–273) — user-visible native tooltip, hardcoded.
  - Maturity legend labels render the **raw keys** `seed/sapling/evergreen/canonical/wilting` (line ~777) and the tooltip prints raw `tooltip.node.maturity` (line ~761) — English regardless of locale. Needs `$t()` maturity-name map.
  - Many `$t(...) || 'English'` fallbacks are fine *as fallbacks* since the keys exist, but the four above have **no** key at all.

## 7. Boot behavior
- **Runs at boot? No (good).** The overlay is **lazy-mounted** — `{#if mapEverOpened}` in `+layout.svelte` (LL-022). It mounts (and fires `constellation_map_universe`) only after the user first opens it. *(The Rust doc comment at `map.rs` lines ~537–564 still describes the old "always-mounted at boot" state that caused `core_queue_ms≈20,693`; LL-022 + the `async` command fixed that. Doc-drift — update in bring-up.)*
- **Rule 8 status: ⚠️ RECOMPUTES-on-read (violation to fix).** `constellation_map_universe` rebuilds the entire tree every open — inbound-link aggregation, `compute_maturity`, `compute_simple_stratum`, `compute_weight`, and the hierarchy assembly all run at *read* time. MIG-077/MIG-078 (`MAP_TREE_FROM_INDEX=true`, `build_tree_from_records`) cut the **cost** by sourcing `note_meta` instead of walking 7,600 files on disk — but the *derivation is still read-time*, not persisted/trigger-maintained. The code's own comment (map.rs ~560) names this the "Rule 8 follow-up (tracked separately)." The correct end-state: persist the map tree, maintained by note-save triggers; open becomes a cheap lookup.
- **Cost:** pre-MIG-077 = tens of seconds (full disk walk, ~419 MB / 7,664 files) on every open + reload. Post-MIG-077/078 = index-driven, **much** cheaper but **unmeasured for the sunburst path** — *estimate only; measure in bring-up* on the 7,600-note Universe.

## 8. Flag / gate & bring-up position
- **Gate today:** `enabledFeatures.constellationMap`. Default is `false`, **and** `store.ts` (~line 4038, MIG-038) hard-overrides it to `false` on every settings load — so even a user who previously enabled it is force-disabled. Dock button + palette entry both gate on `=== true`. Reversible by deleting the override (the Wings detachment).
- **Bring-up phase:** **4 (Visualization satellites)** — alongside Sky View / Org Chart. Depends on: the search index (`note_meta`) being current; the shared `<ContextMenu>` builder (for the §5 gap); the Editor gate (for `onNoteClick` → open). Re-enable only after the Rule-8 persistence question is answered (persist-or-justify-staying-read-time) and the §10 checklist passes.

## 9. Budget
- **Boot budget:** **zero** — must stay lazy (`mapEverOpened`); it may **never** fire an IPC at boot (LL-022 regression guard).
- **Interaction budget:** first open within ~1 s on the 7,600-note Universe (index-sourced); drill-down / color-mode / depth-slider re-render < 100 ms (pure D3, no IPC) — except the depth slider, which re-invokes `loadData()` (acceptable on explicit drag).
- **Regression guard:** open the Map on the large Universe and confirm (a) no boot IPC before first open, (b) first-open time measured + within budget, (c) the recompute is index-only (no disk walk) — watch for the `collect_notes_recursive` fallback firing.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** the sunburst shows whole-Universe structure/density at a glance; drill-down + color modes (maturity/stratum/library) work; it shows something Sky View + file tree do not (else cut it — Constraint as Design).
- [ ] **Serves Constellation's core purpose:** advances **Connection** (survey of the corpus's shape) traceable to [00](00-Constellation-Core-Concept-Paper.md).
- [ ] **Wires correctly to the Editor:** an arc click opens the note in the Editor; the Map writes nothing; second-screen mini-map mirrors via `emitMapCompanion`.
- [ ] **Right-click present + correct:** arcs expose actions via the **shared** `buildContextMenu()` / `<ContextMenu>` (MIG-077) — **not** hand-rolled; item set verified consistent with file tree / Sky View.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** the four hardcoded strings (Depth label, type tooltips, maturity legend + tooltip names) moved to `$t()`; all 15 locales updated; RTL layout + `dir="auto"` confirmed.
- [ ] **Within budget:** no boot IPC; first-open measured on 7,600 notes and within §9.
- [ ] **Obeys Rule 8:** either the map tree is persisted + trigger-maintained (preferred), or a Boss-approved written justification for staying read-time-derived is logged.
- [ ] **Holds its invariants:** alias-aware inbound counts; human titles (never canonical filenames) in labels; no stale-after-edit surprise (refresh-on-reopen documented or a `note-saved` refresh added).
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** (MIG-038 force-disabled; `enabledFeatures.constellationMap` overridden to `false`) · Budget met: **— (unmeasured on the sunburst path post-MIG-077/078)**
Notes: Code intact; flag off, awaiting detachment into a standalone **Constellation Wings** plug-in. The two blocking items before re-enable: **(1) Rule 8** — persist the derived tree or get a logged justification to stay read-time; **(2) right-click** — add the missing shared context menu. Lesser items: four hardcoded English strings; the stale-Rust-doc-comment about "always mounted at boot" (corrected by LL-022); no `note-saved` refresh. `OrgChart.svelte` shares the same backend, so the Rule-8 fix benefits both.
