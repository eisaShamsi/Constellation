# Experiment: Phase 6 — Live Preview

## Hypothesis
Importing the shared `livePreviewPlugin` and `livePreviewTheme` from `$lib/editor/livePreview` will give eNotePane full WYSIWYG-like decorations without any measurable typing latency increase.

## Spec Reference
- Section 3.3: Phase 6 — Live Preview (Incremental)
- Section 4.4: No Feature Shall Slow Typing
- Editor Parity Rule: all note views must share the same CM6 extensions

## Implementation
- **Reused existing shared plugin** — `src/lib/editor/livePreview.ts` (already proven in CodeMirrorEditor)
- Added `livePreviewCompartment` for dynamic enable/disable
- Added `livePreviewEnabled` $state (default: true)
- Added toggle $effect with guard variable (no loops)
- Added "Source mode" / "Live Preview" toggle to more menu (matches NotePane pattern)
- Created `src/lib/editor/markdownHighlight.ts` — custom MarkdownExtension for `==highlight==` syntax (adds Highlight/HighlightMark nodes to syntax tree)
- Added capture-phase mousedown listener for checkbox click-to-toggle (reliable: fires before CM6, uses coordsAtPos for line matching)
- Added `notePane.sourceMode` i18n key to all 15 locale files

### All 10 sub-features (from shared plugin):
1. Headings: hide `#` marks, apply font size (H1-H6)
2. Bold/Italic: hide `**`/`_` marks, apply style
3. Strikethrough: hide `~~` marks, apply line-through
4. Highlights: hide `==` marks, apply background
5. Inline code: hide backticks, apply monospace style
6. Links: style `[text](url)`
7. Wikilinks: hide `[[` `]]`, show display text, style as link
8. Checkboxes: replace `[ ]`/`[x]` with interactive checkbox
9. Horizontal rules: style `---`
10. Tags: style `#tag` with accent background

## Files Changed
- `src/lib/components/eNotePane.svelte` — MODIFIED (import, compartment, extensions, toggle effect, menu item, checkbox handler)
- `src/lib/editor/markdownHighlight.ts` — NEW (custom MarkdownExtension for ==highlight== syntax)
- `src/lib/i18n/*.json` — all 15 locales (added `notePane.sourceMode` key)

## Bugs Fixed During Testing
1. **Highlight not rendering** — base markdown parser doesn't recognize `==`. Created custom `MarkdownExtension` with `parseInline` for `==` delimiters.
2. **Checkbox click not persisting** — CM6 processes mousedown before `domEventHandlers`, destroying the widget. Fixed with capture-phase `addEventListener` on the editor element + `coordsAtPos` for line matching.
3. **Toggle label** — showed "Editing mode" instead of "Source mode". Added new i18n key `notePane.sourceMode`.

## Audit Results

| Agent | Verdict | Notes |
|---|---|---|
| Performance (PA) | PASS | Reuses proven ViewPlugin. Processes only visibleRanges. Maps decorations on docChanged (~0.05ms), full rebuild deferred 300ms via rAF |
| Architecture (AA) | PASS | Compartment pattern for toggle. Guard variable prevents $effect loops. Shared plugin = Editor Parity Rule satisfied |
| Memory (MA) | PASS | Plugin has destroy() that clears rebuildTimer. Capture listener on editorEl (destroyed with component) |
| Spec Compliance (SCA) | PASS | All 10 sub-features implemented. Highlight required custom MarkdownExtension |
| RTL/Bidi (RA) | PASS | Decorations are text styling only — no layout direction impact |
| UX (UXA) | PASS | Cursor-aware: marks visible on cursor line, hidden elsewhere. Toggle in more menu shows "Source mode" / "Live preview" |
| Code Quality (CQA) | PASS | ~60 lines added to eNotePane. 48-line markdownHighlight.ts. Reuses 570-line shared plugin |
| Environment (EA) | PASS | No new store updates, no new IPC |

## Testing Protocol (user-tested 2026-03-28)

| Test | Result |
|---|---|
| Heading `# Title` — hides `#`, applies large font | PASS |
| Bold `**text**` — hides `**`, applies bold | PASS |
| Italic `*text*` — hides `*`, applies italic | PASS |
| Strikethrough `~~text~~` — hides `~~`, applies line-through | PASS |
| Highlight `==text==` — hides `==`, applies background | PASS |
| Inline code `` `code` `` — hides backticks, applies monospace | PASS |
| Link `[text](url)` — styled as link | PASS |
| Wikilink `[[note]]` — hides brackets, styled as link | PASS |
| Checkbox `- [ ] task` — shows interactive checkbox | PASS |
| Checkbox click toggles `[ ]`/`[x]` | PASS |
| Horizontal rule `---` — styled | PASS |
| Tag `#tag` — styled with accent background | PASS |
| Cursor on line → raw markdown visible | PASS |
| Cursor off line → marks hidden (WYSIWYG) | PASS |
| Toggle: More menu → "Source mode" disables decorations | PASS |
| Toggle: More menu → "Live Preview" re-enables decorations | PASS |
| Rapid typing (10 chars) → zero lag | PASS |
| Long note → scroll smooth | PASS |

## Decision
- [x] APPROVED — all 8 auditors pass, all 18 user tests pass
- [ ] REJECTED
- [ ] NEEDS WORK

## Date
2026-03-28
