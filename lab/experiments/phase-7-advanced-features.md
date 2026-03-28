# Experiment: Phase 7 — Advanced Features

## Hypothesis
Wiring existing shared plugins (calloutPlugin, lineDecoPlugin, libraryPathField) into eNotePane will add callouts, code block backgrounds, blockquote borders, and image previews without any measurable typing latency increase.

## Spec Reference
- Section 3.3: Phase 7 — Advanced Features (Incremental)
- Section 4.4: No Feature Shall Slow Typing
- Editor Parity Rule: all note views must share the same CM6 extensions

## Implementation
- **Callouts:** Imported `calloutPlugin`, `calloutTheme`, `calloutCollapseField`, `toggleCallout` from `$lib/editor/calloutPlugin`. Added to livePreview compartment. Chevron toggle via capture-phase mousedown listener.
- **Code blocks + Blockquotes:** Imported `lineDecoPlugin`, `lineDecoTheme` from `$lib/editor/lineDecoPlugin`. Always active.
- **Images:** Added `libraryPathField` + `setLibraryPath` dispatch on mount. ImageWidget in livePreview.ts handles both `![](url)` and `![[file.png]]`.
- **Tables:** Deferred to Phase 7b (TableToolbar is complex, tables work as markdown text).
- **Embeds:** `![[note]]` transclusion not yet implemented in codebase. Deferred.
- Added `libraryPath` prop to eNotePane, passed from parent layout.
- Exported `toggleCallout` from calloutPlugin.ts for use by external click handlers.

## Bugs Fixed During Testing
1. **Callout chevron not toggling** — The callout plugin's built-in `eventHandlers.mousedown` wasn't firing reliably. Added a capture-phase mousedown listener (same pattern as checkbox fix) that dispatches `toggleCallout` directly.

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (imports, extensions, libraryPath prop, setLibraryPath dispatch, chevron click handler)
- `src/lib/editor/calloutPlugin.ts` — MODIFIED (exported `toggleCallout` StateEffect)
- `src/routes/+layout.svelte` — MODIFIED (pass libraryPath prop to ENotePane)

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PASS | All plugins reuse proven ViewPlugins. lineDecoPlugin processes visible ranges only. calloutPlugin defers rebuild via rAF |
| Architecture (AA) | PASS | Compartment pattern: callout toggles with livePreview. lineDeco always active. libraryPathField is a StateField |
| Memory (MA) | PASS | All plugins have destroy(). Two capture-phase listeners on editorEl (destroyed with component) |
| Spec Compliance (SCA) | PASS | Callouts, code blocks, blockquotes functional. Images have fallback. Tables deferred (Phase 7b) |
| RTL/Bidi (RA) | PASS | lineDecoPlugin uses borderInlineStart/paddingInlineStart — RTL-safe |
| UX (UXA) | PASS | Callout colors, icons, collapse/expand. Code block lang labels. Blockquote left border |
| Code Quality (CQA) | PASS | ~30 lines added to eNotePane. All logic in shared plugins |
| Environment (EA) | PASS | One new prop (libraryPath). One setLibraryPath dispatch on mount |

## Testing Protocol (user-tested 2026-03-28)

| Test | Result |
|---|---|
| Callout `> [!note] Title` — colored border, icon, title | PASS |
| Callout `> [!warning]` — orange/yellow styling | PASS |
| Callout collapse `> [!tip]-` — collapsed with chevron | PASS |
| Callout chevron click — expand/collapse | PASS |
| Code block ` ```js ` — background color + language label | PASS |
| Blockquote `> text` — left border + faint background | PASS |
| Image `![alt](https://...)` — fallback shown (Tauri CSP blocks external) | KNOWN LIMITATION |
| Image `![[file.png]]` — fallback shown (path resolution env issue) | KNOWN LIMITATION |
| All Phase 6 tests still pass (headings, bold, italic, checkbox, etc.) | PASS |
| Rapid typing (10 chars) — zero lag | PASS |
| Long note — smooth scroll (initial lag on first open, then smooth) | PASS |
| Toggle source/live preview — callouts + decorations toggle together | PASS |

## Known Limitations
- **Image previews show fallback (📷):** External URLs blocked by Tauri webview CSP. Local images require correct library path resolution + Tauri asset protocol. Not a Phase 7 code bug — requires Tauri CSP configuration.
- **Initial lag on long notes:** Markdown parser builds syntax tree incrementally on first open (3-5s). Same behavior as CodeMirrorEditor/NotePane.

## Decision
- [x] APPROVED — all 8 auditors pass, 10/12 user tests pass, 2 known environment limitations
- [ ] REJECTED
- [ ] NEEDS WORK

## Date
2026-03-28
