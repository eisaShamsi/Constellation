# Session Log — 2026-03-30

## Session Summary
Started a new major work stream: the Constellation Cognitive Engine (CE).
Derived from the "The Cognitive Engine v2.1" architecture paper.

---

## Work Completed

### 0. Context & Planning
- Read `docs/constellation_cognitive_engine_v2.1.pdf` (15 pages)
- Assessed paper: sound two-layer architecture, genuine epistemological basis (Philosophy of Knowledge paper), original Provenance Chain / isnad concept
- Mapped existing codebase against CE spec:
  - Foundation already in place: GraphMind, Wikilinks, Backlinks, Tags, AI engine, Frontmatter, FocusPane (= Fleeting stage), Dataview/Bases
  - Missing: Typed Links, Knowledge Strata, Maturity Lifecycle, Tension Detector, Provenance Chain, and 8 more tools

### 1. Documents Created
- `docs/cognitive-engine-roadmap.md` — master roadmap with all 16 phases, build sequence, progress table
- `docs/CE-spec.md` — comprehensive specification for all 16 phases (equivalent depth to NotePane-spec.md): architecture, test plans, GO/NO-GO criteria, files to change per phase
- `docs/phases/CE-phase-01-typed-links.md` — detailed Phase 1 spec

### 2. CE Phase 1: Typed Links — IMPLEMENTED
**Commit**: `d7edc6d`

**What was built**:
- Rust parser: `[[note|causes]]` — direct pipe-syntax link types, no `type:` prefix required. `KNOWN_LINK_TYPES` constant validates 7 types.
- `store.ts`: `getBacklinks()` now passes `linkType` to UI
- `BacklinksPanel.svelte`: colored badge per link type (7 colors)
- `livePreview.ts`: typed links show note name in type-specific color; `|type` hidden from view; 7 CSS classes added to editor theme
- `completions.ts`: `createTypedLinkCompletion()` — triggers on `[[note|`, offers 7 link types with descriptions
- `NotePane.svelte`: `typedLinkCompletion` wired (highest priority in completion stack)
- `graphEngine.ts`: typed links render in type color (normal state at reduced opacity + hover); `contradicts` = bidirectional; `causes` = thicker stroke; `TYPED_LINK_COLORS` constant added

**Link types implemented**:
| Type | Color | Meaning |
|---|---|---|
| supports | #4A9EFF blue | Evidence for a claim |
| contradicts | #FF4A4A red | Tension / opposition |
| causes | #FF8C42 orange | Causal relationship |
| exemplifies | #4AFF88 green | Instance-of |
| generalizes | #A44AFF purple | Abstraction |
| derives-from | #FFD700 gold | Provenance / source |
| part-of | #AAAAAA gray | Compositional hierarchy |

**Rust check**: `cargo check` → clean
**Svelte check**: 1 pre-existing error in vite.config.js (unrelated)

### 3. CE Spec Committed
**Commit**: `5df2ae3`
`docs/CE-spec.md` — 890 lines covering all 16 phases

---

---

## NotePane Regression — Bug Fixes

### Bug #1: New Tab opens NotePane editor instead of home screen

**Symptom**: Clicking `+` tab button opened a full NotePane with "New tab" as the heading.

**Root cause**: In `+layout.svelte` the condition `{:else if $activeTab}` rendered NotePane for ALL active tabs, including empty ones where `path: ''`.

**Fix** (`src/routes/+layout.svelte`):
- Split the condition into two branches:
  - `{:else if $activeTab && !$activeTab.path}` → shows **new tab home screen**
  - `{:else if $activeTab && $activeTab.path}` → renders **NotePane** (unchanged)
- New tab home screen: centered layout with star icon, "Open note" (quick switcher) button, and "New note" button
- Added `.new-tab-screen`, `.nt-btn`, `.nt-actions`, `.nt-hint` CSS classes

**Status**: Fixed, svelte-check clean (no new errors)

---

### Commit `3fdf55b` — New tab screen
- `src/routes/+layout.svelte`: guard NotePane on `$activeTab.path`; new-tab branch renders plain Obsidian-style command list
- All 15 i18n files: added `tabs.closeTab` and `tabs.openNote` keys
- **Status**: pushed, user-confirmed working ✅

---

---

## NotePane Regression — Bug #2: "Graph View" label leak

**Symptom**: "Graph View" text still appeared throughout the app despite renaming to Sky View.

**Fix** (commit `e59d057`):
- All 15 i18n locale files: updated `plugins.graphView`, `commands.graphView`, `ribbon.graphView`, `sidebar.graphViewTitle`, `secondScreen.graphView` → "Sky View"
- `src/lib/graph/types.ts`: fixed code comment

**Fix** (commit `5f716f0`):
- 6 doc/spec files: Star View.md, Vault management.md, NotePane-spec.md, DEVELOPMENT_PLAN.md, FEATURE_REFERENCE.md, README.md

**Status**: ✅ Closed

---

## NotePane Regression — Bug #3: "vault" terminology leak in docs

**Symptom**: `Vault management.md`, `Vaults/` folder, and "vault" references throughout docs violated the Library naming rule.

**Fix** (commit `e8c08dc`):
- Renamed `docs/help.uConstellation.World/Vaults/` → `Libraries/`
- Renamed `Vault management.md` → `Library management.md`
- Replaced all vault→library in: Library management.md, BASES_MVP_SPEC.md, User Manual.md, FEATURE_REFERENCE.md, DEVELOPMENT_PLAN.md
- Preserved proper nouns: FileVault (Apple), Obsidian vault (import compat)

**Status**: ✅ Closed

---

---

## NotePane Regression — Bug #4: Sky View freezes app on node click or close

**Symptom**: Clicking any node or the close button in Sky View caused the application to stop responding completely.

**Root cause**: `app.destroy({ children: true, texture: true })` in `GraphEngine.destroy()` synchronously called `.destroy()` on every Pixi.js `Graphics` object in the stage — one per node. For a 1000-node graph, that's 1000+ WebGL buffer disposals in a tight loop on the main thread, blocking it for several seconds.

**Fix** (`src/lib/graph/graphEngine.ts`, commit `adf48fc`):
- Before calling `app.destroy()`, manually clear the display hierarchy:
  - `nodeContainer.removeChildren()` — detaches all node Graphics (O(1), no WebGL calls)
  - `app.stage.removeChildren()` — detaches remaining stage children
  - `nodeGfx = []` — releases engine references
- Call `app.destroy(true, { children: false, texture: true })` — destroys only the renderer, not children
- Orphaned Graphics objects are GC'd by the JS runtime (WebGL buffers released lazily)
- Also explicitly destroy `gizmoLabels` (previously leaked)
- Result: `destroy()` completes in O(1) regardless of graph size

**Status**: ✅ Closed

---

## Open Items / Next Session

1. **Phase 1 GO/NO-GO**: User has not yet tested Phase 1 (Typed Links). Must test before proceeding to Phase 2.
2. **Phase 2: Knowledge Strata** — next to implement after GO/NO-GO on Phase 1
3. **Session log for 2026-03-27** — still needs Phase 1 entry (was pre-existing)

## Test Plan for Phase 1 (user to conduct)
Key tests:
- Type `[[note|supports]]` → save → reopen → link preserved
- Type `[[note|` → autocomplete list appears with 7 types
- Open BacklinksPanel on a linked note → colored badge shows
- Open GraphMind → typed links show in their colors
- Existing untyped `[[links]]` → zero regression

---

## Architecture Notes
- The CE is designed to be invisible: complexity in the system, simplicity for the user
- Layer 1 (12 tools) must work fully offline — zero AI dependency
- Layer 2 (5 AI tools) uses existing `ai_send_message` Tauri command; local LLM supported
- Build order is strict: Phase 1 (Typed Links) unlocks all of Phases 2–5 and 12
