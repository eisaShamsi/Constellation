# 12 — Sky View (Concept Paper)

> Per-function paper. Traces to [00-Constellation](00-Constellation-Core-Concept-Paper.md): Sky View serves the **Connection** Act, obeys **Rule 8** (write-time derivation), and attaches downstream of the Editor (the gate). Template per [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3, matching [01 — Note Editor](01-Note-Editor.md).

## 1. Function in hand
**Sky View** — the full-window force-directed graph of the Universe, rendered as PIXI **bubbles** (NOT the sunburst arcs — that is Constellation Map). Component: `src/lib/components/GraphMindView.svelte` (the "GraphMind" Svelte wrapper), driven by the PIXI/worker engine `src/lib/graph/graphEngine.ts`. A small companion "local star" view (`src/lib/components/LocalSkyView.svelte`, a 2D-canvas active-note-at-center renderer) is the right-sidebar embed; the named candidate `SkyView.svelte` / `FullSkyView.svelte` are dead code (no static importer — confirmed in-source).

## 2. Purpose
Show the user the **shape of their knowledge graph** — every note as a bubble, every link as an edge — so they can *see* connections (and clusters, hubs, orphans) that no single note reveals. It serves the **Connection** Act directly: it is the surface where the topology of the Universe becomes visible, the entry point to traversing one note to the next. Justified: *awareness of connections* is the core purpose ([00](00-Constellation-Core-Concept-Paper.md) §2); a graph is the canonical instrument for it. It is also the launch pad into a note (click a bubble → open in inspect mode).

## 3. What it is NOT
- **Not** Constellation Map (D3 sunburst arcs) — Sky View is PIXI bubbles. The two are not interchangeable.
- **Not** Sight (whole-universe epistemic dome) — different scope and primitive.
- **Not** an editor — it opens notes into the Editor; it never reads/writes note content itself (one read-on-demand for semantic enrichment aside).
- **Not** the owner of node positions/hover/simulation — those live in `GraphEngine` + the force worker; the Svelte layer owns only UI controls and stats.

## 4. Wiring
- **Inputs:** `nodes={skyNodes}` / `links={skyLinks}` props (the persisted sky snapshot in `+layout.svelte`); `skyViewSettings` (`$appSettings.skyView`); `libraryColorMap`; `highlightPath` (selected folder/library); the Style Setter palette via `resolveSkyPalette()` + `linkTypesStore`; on-demand `invoke('read_file')` only when the user runs **semantic links** enrichment.
- **Outputs:** callbacks `onNodeClick(path, libraryName, highlightTerm?)`, `onNodeHover(node|null)`, `onRequestEnrichment()`. No `write_note`, no `emit`, no direct save IPC — it dispatches nothing to disk.
- **Consumers:** `+layout.svelte` (opens the clicked note → `skyViewInspectMode`, "Return to Sky View" pill); second screen mirrors via the same `skyNodes`/`skyLinks` (`SecondScreenPage.svelte`).
- **Connection to the Editor (the gate):** Sky View is **read-only downstream** of the Editor. The graph reflects what the Editor's saves wrote (links/tags reindex → `note_links` → sky triggers). Clicking a bubble routes back *through* the layout to open that note in the Editor — Sky View never re-implements load/save (display, not domain).

## 5. Right-click / context menu
- **Has one — but HAND-ROLLED.** `GraphMindView` renders its own `.gm-context-menu` div (lines ~1154–1161), not the shared `<ContextMenu>` / `buildContextMenu()` (MIG-077). **Flag this as debt:** core paper §5 mandates one shared builder and "never a hand-rolled per-surface menu."
- **Items (per graph node):** Open (`graphView.open`), Focus (`graphView.focus`), Pin/Unpin (`graphView.pin`/`graphView.unpin`), Hide (`graphView.hide`) — all localized via `$t()`, menu has `dir="auto"` + RTL-aware left/right anchoring.
- **Actions reachable ONLY by right-click:** **Focus**, **Pin**, **Hide** — left-click opens the node; these three engine operations have no other entry point in the graph surface (verify in bring-up whether any toolbar duplicates them).
- **Bring-up action:** fold these four items into the shared `buildContextMenu()` so Sky View's node menu matches every other surface (one source of truth).

## 6. Multilingual
- Uses `$t()` for menu items, stats labels, search/legend chrome; imports `dir`, `isRTL`, `detectDir`, `getSearchOps`; RTL-aware directional symbols (`arrowIncoming`/`arrowOutgoing`/`breadcrumbSep`) and RTL menu/tooltip anchoring; tooltips carry `dir="auto"`.
- **Hardcoded English found (flag):** the semantic-enrichment progress strings — `'Loading AI model...'`, `` `Embedding notes: ${current}/${total}` ``, `'Computing similarities...'` (GraphMindView ~lines 381–383) — and the `MOCs` stats token (~line 1170) are not routed through `$t()`. These must move to `$t()` across all 15 locales (ar de en es fa fr he hi ja ko pt ru tr ur zh) before re-enable. Engine-internal label strings in `graphEngine.ts` — verify in bring-up.

## 7. Boot behavior
- **Runs at boot?** Data yes, render no. `+layout.svelte` fires `cache_boot_snapshot_sky` during boot to populate `skyNodes`/`skyLinks`; the PIXI canvas only mounts when the user opens Sky View (`showSkyView`). Mounting also spins a force-simulation Web Worker.
- **Rule 8 status: ✅ READS-PERSISTED.** `sky_nodes`/`sky_links` are persisted tables (MIG-001) kept current at write time by the `note_meta_sky_ai/ad/au` + `note_links_sky_ai/ad/au` triggers; `cache_boot_snapshot_sky` (cache.rs:514) reads them and gates on the back-fill stamp (`schema_versions.sky`, returns `is_ready=false` until complete, resumable via `sky_backfill.rs`). `buildSkyData()` is the **recompute fallback only** when the stamp is absent/IPC errors — not the steady-state path. The legacy "skyNodes/skyLinks rebuilt on every boot" audit item (CLAUDE.md Rule 8) is **resolved** by MIG-001; verify the fallback is never the hot path on a large Universe.
- **Cost:** snapshot read is a fast indexed table scan (IPC `timings_ms` instrumented; not separately measured here — **measure in bring-up**). PIXI mount + worker sim cost is paid on first open, not boot. Mark both **estimated — verify in bring-up**.

## 8. Flag / gate & bring-up position
- **Gate today:** `enabledFeatures.skyView` (defaults ON — `!== false` / `?? true`). Both the dock button and the inspect-mode pill are behind it.
- **Bring-up phase:** **Phase 4 (visualization).** Depends on: the persisted sky snapshot + triggers (Rule 8 spine, already shipped), `skyPalette`/Link-Types registry, and the Editor (the gate) for node→note open. Re-enable only after §10 passes.

## 9. Budget
- **Boot budget:** the snapshot IPC must not regress `paint_ms`/`hydrated_ms` — measure on a 7,600+ note Universe before/after (the fallback `buildSkyData` iterates the full link set on the main thread, so it must stay off the steady-state path).
- **Interaction budget:** zero `invoke()` on hover/pan/zoom/render (the only IPC is the user-initiated semantic enrichment, which streams progress); pan/zoom and hover must stay 60fps; PIXI teardown is deferred (engine note ~line 830) to avoid a ~100ms close stall.
- **Regression guard:** open Sky View on the large Universe, pan/zoom/hover with no stutter (Rule 3/7); confirm no leaked observers/workers on close (Rule 4); confirm the snapshot path (not the fallback) serves the graph.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** the graph renders the Universe; bubbles = notes, edges = links; clusters/hubs/orphans are visible.
- [ ] **Serves Constellation's core purpose:** advances the **Connection** Act — the user can *see* and traverse connections.
- [ ] **Wires correctly to the Editor:** clicking a bubble opens that note in the Editor (inspect mode); "Return to Sky View" restores; Sky View writes nothing to disk.
- [ ] **Right-click present + correct (shared, not hand-rolled):** node menu (Open/Focus/Pin/Hide) folded into shared `buildContextMenu()` — **currently hand-rolled, must fix.**
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** semantic-progress + `MOCs` strings moved to `$t()`; menu/stats/legend localized; RTL anchoring holds.
- [ ] **Within budget:** large-Universe boot un-regressed; pan/zoom/hover stutter-free; no `invoke()` on the interaction hot path.
- [ ] **Obeys Rule 8:** serves from persisted `sky_nodes`/`sky_links` (triggers current); `buildSkyData` never the steady-state path.
- [ ] **Holds its invariants:** GraphEngine owns positions/hover/sim; Svelte owns UI only; second screen mirrors the same snapshot (display, not domain); no leaked worker/observers on close.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (unmeasured — verify in bring-up)**
Notes: Real component is `GraphMindView.svelte` (PIXI bubbles) + `graphEngine.ts`; `LocalSkyView.svelte` is the sidebar companion; `SkyView.svelte`/`FullSkyView.svelte` are dead code. Two carry-over fixes before re-enable: (1) **hand-rolled context menu** → shared `buildContextMenu()`; (2) **hardcoded English** semantic-progress + `MOCs` strings → `$t()` ×15. Rule 8 is satisfied via MIG-001 (persisted sky tables + triggers), resolving the legacy "rebuilt on every boot" audit item — confirm the recompute fallback never fires on the large Universe.
