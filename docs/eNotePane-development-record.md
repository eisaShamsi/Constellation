# eNotePane Development Record

> This document records the complete development story of eNotePane — from experimental prototype to production NotePane. Preserved for historical reference.

---

## Background

**Date:** 2026-03-26 to 2026-03-29

**Problem:** The original NotePane used CodeMirrorEditor.svelte, a massive component with accumulated performance and architecture problems:
- `$effect` echo loops caused cursor jumping and typing lag
- Store updates during autosave triggered full reactivity cascades across the 3873-line `+layout.svelte`
- No clean separation between editor state, save logic, toolbar, and properties
- Arabic text typing was particularly affected by lag

**Solution:** Build a new editor from scratch — eNotePane (experimental NotePane) — following a rigorous phase-by-phase spec with 8 audit agents and user testing at every gate.

---

## The Spec

`docs/NotePane-spec.md` (originally `docs/eNotePane-spec.md`) defined:
- Philosophy: speed > features > appearance
- 5ms keystroke latency target
- 9 phases (0-8) each requiring all 8 auditors to PASS
- Editor Parity Rule: all note views share the same CM6 extensions
- File Over App: `.md` on disk is the source of truth

---

## Phase Summary

| Phase | What | Commit | Tests | Rounds |
|---|---|---|---|---|
| 0 | Skeleton (desk + paper + title) | `a14923a` | 7 | 2 (BLOCKING-001 discovered) |
| 1 | Bare editor (CM6, zero plugins) | `2c8b76b` | 10 | 1 |
| 2 | Save & restore (WAB, idle save) | `c72b2f8` | 6 | **13** (hardest phase) |
| 3 | Breadcrumb & properties | `59777e1` | 8 | 2 |
| 4 | Toolbar (formatting buttons) | `df3b24b` | 9 | 1 |
| 5 | Syntax highlighting (custom colors) | `bea9a39` | 7 | 2 (defaultHighlightStyle was too subtle) |
| 6 | Live preview (10 decoration types) | `1632a88` | 18 | 4 (checkbox, highlight, toggle label) |
| 7 | Advanced features (callouts, code blocks, images) | `f98ff25` | 10 | 2 (callout chevron) |
| 7b | Table toolbar (add/remove/sort/Tab nav) | `652817c` | 12 | 4 (positioning, getCursorColumn bug) |
| 8 | Knowledge infrastructure (autocomplete) | `27feda3` | 12 | 2 (extra brackets from closeBrackets) |

**Total: 99 user tests across 33 testing rounds.**

---

## Blocking Issues Discovered

### BLOCKING-001: Layout Reactivity Storm
- **Symptom:** 3s+ freeze on title click, 10s+ hangs
- **Root cause:** 6 synchronous `$derived` blocks in `+layout.svelte` (parseFrontmatter, extractHeadings, detectDir, getBacklinks, getOutgoingLinks, activeNoteTags) firing on every state change
- **Fix:** Debounced sidebar derived chain, fixed idleTimer memory leak

### BLOCKING-002: Re-entrant Store Update
- **Symptom:** App freeze on tab close
- **Root cause:** `onflush` called `updateTabContent()` (which calls `openTabs.update()`) inside `onDestroy`, during an already-running Svelte update cycle from `closeTab()`
- **Fix:** Direct object mutation instead of store.update()

### BLOCKING-003: Multiple Store Cascades
- **Symptom:** Freeze on tab close/reopen
- **Root cause:** `closeTab()` did 3 separate store updates: `openTabs.set()`, `editingTabIds.update()`, `activeTabId.set()` — each triggering a full 3873-line layout re-render
- **Fix:** Batched updates, non-reactive cleanup first

---

## Key Architecture Decisions

### 1. Zero IPC During Typing
After 13 rounds of Phase 2 testing, discovered that ANY Rust `invoke()` call causes perceptible lag. Final architecture: content stays in JS memory during typing, writes to disk only on idle (30s), blur, or close.

### 2. Write-Ahead Buffer (WAB)
Synchronous in-memory Map + localStorage backup ensures content survives tab close/reopen and app crash without waiting for async disk writes.

### 3. Direct Mutation Over Store Updates
Store mutations trigger the layout reactivity cascade. Direct object mutation (`tab.content = ...`) is invisible to Svelte's reactivity — safe during `onDestroy`.

### 4. Capture-Phase Event Listeners
CM6's `domEventHandlers` run after CM6 processes events. For replacement widgets (checkboxes, callout chevrons), the widget is destroyed before the handler fires. Solution: capture-phase `addEventListener` on the editor DOM element.

### 5. Shared Plugins
All editor extensions live in `src/lib/editor/` — livePreview, calloutPlugin, lineDecoPlugin, completions, markdownHighlight, tableUtils. Both NotePane and CodeMirrorEditor import from the same source.

---

## Post-Phase Work

| Item | Commit | What |
|---|---|---|
| SO1: Simplify review | `2b5ad67` | O(n)→O(1) checkbox, early-exit autocomplete, listener cleanup |
| SO2: Table cleanup | `da31fae` | $derived for toolbar visibility, removed wrapper div |
| Editor parity | `59597d3` | Shared completions.ts, HighlightExt in CodeMirrorEditor |
| Image previews | `bd60372` | Tauri asset protocol + CSP for external + local images |
| Performance | `be70087` | Merged 3 line-scan loops, increased rebuild delay |
| Docs sync | `c444874` | User Manual + 14 translations updated |
| Lessons Learned | `bdab789` | 13 rules (LL-001 to LL-013) |

---

## Promotion to Production

**Date:** 2026-03-29

eNotePane was renamed to NotePane and became the default note editor:
- `eNotePane.svelte` → `NotePane.svelte`
- Old NotePane archived to `archive/NotePane-legacy.svelte`
- `eNotePane` i18n keys merged into `notePane` section
- `eNotePane-spec.md` → `NotePane-spec.md`
- Import in `+layout.svelte` updated

The "e" (experimental) prefix was removed. The experiment succeeded.

---

*Recorded: 2026-03-29*
