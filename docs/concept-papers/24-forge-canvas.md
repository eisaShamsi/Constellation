# 24 — Expression Forge & Sense-Making Canvas (Concept Paper)

> Two satellites of the Editor (the gate, [01](01-Note-Editor.md)) — the **pre-write** and **post-write** ends of composition. Both are **palette-only today** (no flag, no dock button, no `enabledFeatures` gate). This paper is their re-enable gate. Covered together because they share the same role (authoring surfaces that *feed* the Editor, never replace it) and the same two debts (no shared right-click; hardcoded English in one dialog).

## 1. Function in hand
- **Expression Forge** — `src/lib/components/ExpressionForge.svelte` (command palette "Expression Forge", icon ✨, CE Phase 10). A two-pane composer: a left note-browser (strata-filtered) and a right ordered stack of **blocks** (one per chosen note) with free-text **transitions** between them; "Export as Note" writes the assembled markdown as one new note.
- **Sense-Making Canvas** — `src/lib/components/SenseMakingCanvas.svelte` (command palette "Sense-Making Canvas", icon 🎨, CE Phase 11; Rust `src-tauri/src/canvas.rs`). An infinite pan/zoom board of Post-it items laid over the four **Cynefin** quadrants (Clear / Complicated / Complex / Chaotic); items can be **promoted** into real notes. Persisted as `.canvas` JSON files in the library.

## 2. Purpose
- **Forge** serves **Synthesis** — its one job is to *assemble existing mature notes into a single linear argument* with the user's connective tissue (the transition text) and emit it as a new note. It is the bridge from a set of established claims to a written composition.
- **Canvas** serves **Observation → Connection** (the pre-structural end) — its one job is *to hold ambiguous, half-formed ideas in spatial relationship before they are notes*, then graduate the ones that harden (`promote` → real note with `stage: growth`). Cynefin is the framing lens for "how settled is this thought?"
- **Honest caveat (Constraint-as-Design):** both overlap surfaces that already exist. Forge's "stitch notes together" is one step beyond what transclusion/embeds + the Editor already do; Canvas overlaps a whiteboard the file tree + Sky View don't provide but Obsidian-style `.canvas` does. Each must re-justify at bring-up: if Forge produces nothing the Editor + embeds can't, or Canvas is a board nobody promotes from, it doesn't earn its place.

## 3. What it is NOT
- **Neither is the Editor.** They do not own a note's in-memory content. Forge reads note bodies read-only to preview them; Canvas owns only its own `.canvas` JSON, never note content.
- **Forge is NOT a live multi-note editor** — block content is a read-only preview; edits happen in the real note via the Editor. Its only write is the one-shot export.
- **Canvas is NOT a graph/map** — no links between items, no derived structure; it's a free spatial board (not Sky View bubbles, not the Map sunburst).
- **Canvas `.canvas` files are NOT notes** — they're a separate JSON artifact; only *promoted* items become `.md`.

## 4. Wiring
- **Forge — Inputs:** props `notes={skyNodes}` (the host's already-computed Sky View node list — Forge does **not** recompute it), `activeTrail` (CE Phase 8 trail → seeds the block backbone via `$effect`), `libraryPath`/`libraryName`. IPC: `read_note(filePath)` per block (lazy body load). **Outputs:** `createNote` + `writeNote(path, content, 'expression_forge')` on export → the new note enters the normal write/reindex path; `onClose` callback. **Consumers:** none downstream beyond the note it creates (which the search index then picks up like any save).
- **Canvas — Inputs:** IPC `list_canvases(libraryPath)` (picker), `read_canvas(canvasPath)` (open), `resolve_universe_libraries` (promote target picker). **Outputs:** `write_canvas(canvasPath, data)` (debounced 1000 ms) for board edits; on promote → `createNote` + `writeNote(path, content, 'canvas_export')` (frontmatter `stage: growth`, `canvas_origin`, optional `canvas_quadrant`) then `openNoteTab` (opens the new note in the Editor) and rewrites the item to a `[[wikilink]]`. **Consumers:** the search index (via the promoted note's save); the second screen is **not** wired (no companion emit).
- **Connection to the Editor (the gate):** both attach **loosely and correctly** as *producers*. They never re-implement save/load/edit of a note — they call `createNote`/`writeNote`/`openNoteTab` and let the Editor + write path own everything thereafter (File-Over-App preserved). Forge's export and Canvas's promote both funnel back into the one write path; the Editor remains the single content authority.

## 5. Right-click / context menu
- **Forge: none.** Grep for `oncontextmenu` / `contextmenu` / `<ContextMenu>` / `buildContextMenu` returns **zero matches**. Block actions (move ↑/↓, collapse, remove) and note-add are left-click buttons only.
- **Canvas: none.** Same grep, **zero matches**. Item actions (promote 🔗, edit ✏️, delete ×) are hover-revealed left-click buttons; add-item is double-click; pan is Shift-drag.
- **Gap — flag both.** Per the core paper §5, these are exactly the surfaces where right-click belongs: a Forge note-row should offer "open in Editor / remove from composition"; a Canvas item should offer "promote / open promoted note / change quadrant / delete / duplicate". All currently **unreachable**. When re-enabled, add via the **shared** `buildContextMenu()` / `<ContextMenu>` (MIG-077) — **do not** hand-roll. Exact item sets: **unknown — verify in bring-up** against the file-tree / Sky-View menus for consistency.

## 6. Multilingual
- **Forge:** all chrome strings flow through `$t('expressionForge.*')`; keys exist and are translated in **all 15 locales** (verified `en.json` + `ar.json` → `titlePlaceholder: "عنوان التأليف..."`). `dir="auto"` on title, search, note names, block names, block content, transitions. **One hardcoded fallback English literal** survives the export-default title (`'New Composition'` / `' — Composition'`, lines ~65) — flag for `$t()`.
- **Canvas:** chrome flows through `$t('senseMakingCanvas.*')`; keys exist ×15; `dir` on the root from `isRTL`, `dir="auto"` on items/inputs. **Hardcoded English — flag these (must move to existing `$t()` keys before re-enable):** the entire **promote dialog** is hardcoded — `"Promote to Note"`, `"Note name"`, `"Select library"`, `"Create Note"`, `"Cancel"` (lines ~404–425) — even though `senseMakingCanvas.noteName` / `selectLibrary` / `promoteConfirm` keys **already exist** in the locale files and are simply not wired. The `🎨` header title prefix and quadrant tooltips are localized; the promote dialog is the live gap.

## 7. Boot behavior
- **Runs at boot? No (good).** Both are lazy — `{:else if showExpressionForge}` / `{:else if showSenseMakingCanvas}` in `+layout.svelte`; neither mounts nor fires IPC until toggled from the palette.
- **Rule 8 status:**
  - **Forge — N/A (does not recompute a derived view).** It consumes the host's `skyNodes` (computed once for Sky View) and reads note bodies on demand; it persists nothing of its own and maintains no derived surface. No violation, but no persistence either — it's a transient assembler.
  - **Canvas — reads-persisted (compliant in spirit).** The board IS its own source of truth (`.canvas` JSON on disk); `read_canvas` is a cheap file load, not a recompute. **One watch-point:** `list_canvases` runs `scan_canvases_recursive` — a **disk walk of the library tree on every picker open** (not boot). Cheap on small libraries; **measure on the 7,600-note Universe** and, if slow, source from the index instead (the Rule-8-shaped follow-up).
- **Cost:** unmeasured for both — *estimate only, measure in bring-up*. Forge: ~1 disk read per block on open (cheap). Canvas: 1 directory walk per picker-open + 1 JSON parse per canvas-open.

## 8. Flag / gate & bring-up position
- **Gate today: none.** Unlike Constellation Map (`enabledFeatures.constellationMap`), neither Forge nor Canvas has a flag, a dock button, or a default-off override. They are reachable **only** via command-palette entries (`expression-forge`, `sense-making-canvas`), unconditionally. **Bring-up needs a new gate** — minimal mode must be able to flip these off; recommend `enabledFeatures.expressionForge` / `enabledFeatures.senseMakingCanvas`, default off, mirroring the Map pattern.
- **Bring-up phase: 5 (Curation / composition satellites)** — after the Editor (1), the index (2), and the read-only panels. Depends on: the Editor gate (for export/promote → `openNoteTab`); the write/reindex path; the shared `<ContextMenu>` builder (for the §5 gap); Sky View's `skyNodes` (Forge's input). Canvas additionally depends on `canvas.rs` IPC being registered.

## 9. Budget
- **Boot budget: zero** — both must stay lazy; neither may fire an IPC before first open (regression guard: confirm no `list_canvases` / `read_note` at boot).
- **Interaction budget:** Forge — add/remove/reorder blocks < 16 ms (pure state, no IPC); block body load is one async `read_note`, off the interaction path. Canvas — pan/zoom/drag at 60 fps (pure CSS transform, no IPC per frame); board save debounced ≥ 1000 ms (never per pointer-move); promote is one explicit `createNote`+`writeNote`.
- **Regression guard:** open each on the large Universe; confirm (a) no boot IPC, (b) first-open measured + within budget, (c) Canvas drag stays smooth with 50+ items, (d) Forge export fires exactly one `write_note` and the note round-trips intact.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** Forge assembles notes + transitions into one exported note that reads coherently; Canvas holds items spatially and promotes them into real notes — and each shows something the Editor + embeds (Forge) / file tree + Sky View (Canvas) cannot (else cut it — Constraint as Design).
- [ ] **Serves Constellation's core purpose:** Forge advances **Synthesis**, Canvas advances **Observation → Connection** — traceable to [00](00-Constellation-Core-Concept-Paper.md)'s Five Acts.
- [ ] **Wires correctly to the Editor:** Forge export and Canvas promote both go through `createNote`/`writeNote` and open in the Editor; neither re-implements save/load; the promoted/exported note enters the normal reindex path.
- [ ] **Right-click present + correct:** Forge rows and Canvas items expose actions via the **shared** `buildContextMenu()` / `<ContextMenu>` (MIG-077) — **not** hand-rolled; item sets verified consistent with file tree / Sky View.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** the Canvas **promote dialog** rewired to the existing `senseMakingCanvas.*` keys; the Forge default-title literals moved to `$t()`; RTL + `dir="auto"` confirmed in both.
- [ ] **Within budget:** no boot IPC; first-open measured on 7,600 notes; Canvas drag smooth with 50+ items; `list_canvases` disk-walk cost measured (index it if slow).
- [ ] **Obeys Rule 8:** Canvas reads persisted `.canvas` JSON (✓); Forge recomputes no universe-wide derived view (✓); the `list_canvases` directory walk is confirmed cheap or moved to the index.
- [ ] **Holds its invariants:** Forge export loses no block content and writes valid frontmatter; Canvas promote leaves a `[[wikilink]]` back-reference, writes `stage: growth` + `canvas_origin`, and never corrupts the source `.canvas`; promoting to a *different* library targets the right path.
- [ ] **Boss-tested** per the Testing Instructions Rule (define each feature, then walk Forge export and Canvas promote click-by-click).

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** (palette-only, no flag — needs a new gate) · Budget met: **— (unmeasured)**
Notes: Both code paths intact and reachable from the command palette today; no `enabledFeatures` flag exists yet for either. Three shared blocking items before re-enable: **(1) a gate** (`enabledFeatures.expressionForge` / `.senseMakingCanvas`, default off); **(2) right-click** — add the missing shared context menu to both; **(3) the Canvas promote dialog's hardcoded English** — wire the already-present locale keys. Lesser items: Forge's default-title English literals; the `list_canvases` disk-walk-on-open (measure, index if slow); Canvas has no second-screen companion (decide if it needs one). Neither is on the Editor's write path — both are *producers* that feed it, which is the correct shape.
