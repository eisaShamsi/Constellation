# Constellation Editor — Full Specification

## Mission

Build a custom, high-performance WYSIWYG Markdown editor from scratch for the Constellation desktop app (Tauri v2 + SvelteKit/Svelte 5). No dependency on TipTap, ProseMirror, CodeMirror, or any third-party editor engine. The editor must be the fastest, most responsive Markdown editor available — surpassing Obsidian, Typora, and Notion.

---

## 1. Architecture

### 1.1 Core Principles

- **Markdown is the source of truth** — the document is always plain Markdown text, never HTML or a DOM tree
- **Zero conversion** — no markdown→HTML→DOM→HTML→markdown round-trips. Ever.
- **Typing is instant** — keystroke handling must be <1ms. No processing in the input path.
- **Rendering is lazy** — only render what's visible in the viewport
- **Decorations over mutations** — visual formatting is achieved by decorating the text, not by transforming it

### 1.2 Layer Architecture

```
┌──────────────────────────────────────────────┐
│  UI Layer (Svelte 5)                         │
│  Toolbar, context menus, dropdowns, modals   │
├──────────────────────────────────────────────┤
│  Decoration Layer                            │
│  WYSIWYG rendering: hide syntax, show        │
│  formatted output, inline widgets            │
├──────────────────────────────────────────────┤
│  Markdown Parser (incremental)               │
│  Tokenize only changed regions               │
├──────────────────────────────────────────────┤
│  Edit Engine                                 │
│  Commands, undo/redo, selections, clipboard  │
├──────────────────────────────────────────────┤
│  Text Buffer (Piece Table)                   │
│  O(log n) insert/delete, snapshot for undo   │
├──────────────────────────────────────────────┤
│  Input Handler                               │
│  Keyboard, IME, composition, bidi            │
├──────────────────────────────────────────────┤
│  Viewport Renderer                           │
│  Virtual scrolling, line layout, bidi        │
└──────────────────────────────────────────────┘
```

---

## 2. Text Buffer

### 2.1 Data Structure: Piece Table

The piece table is the optimal structure for text editors — used by VS Code and many modern editors.

**How it works:**
- Original file content is stored in a read-only "original" buffer
- All insertions go into an append-only "add" buffer
- A table of "pieces" describes the document as ordered slices of these two buffers
- Insert = split one piece into two + add a new piece pointing to the add buffer
- Delete = adjust piece boundaries

**Performance:**
- Insert: O(log n) with balanced tree of pieces
- Delete: O(log n)
- Memory: original text is never copied, only referenced
- Undo: snapshot the piece table (cheap — it's just pointers)

### 2.2 Requirements

- Support documents up to 500K lines without lag
- Line index for O(1) line-number-to-offset mapping
- Character offset to line/column mapping
- UTF-16 aware (for JavaScript string compatibility) but internally track Unicode code points
- Change event with precise affected range (for incremental re-parse)

---

## 3. Input Handling

### 3.1 Keyboard Events

- Use `beforeinput` events (modern, gives intent like "insertText", "deleteContentBackward")
- Fall back to `keydown` for shortcuts and special keys
- Never use deprecated `keypress`
- Handle `Ctrl+Z/Y`, `Ctrl+B/I/U`, `Ctrl+Shift+S` etc. via a configurable keymap

### 3.2 IME (Input Method Editor)

Critical for Arabic, Chinese, Japanese, Korean:

- Use `compositionstart`, `compositionupdate`, `compositionend` events
- During composition: show the composing text in a temporary overlay
- On `compositionend`: commit the final text to the buffer
- Never interrupt composition with cursor moves or formatting
- Arabic reshaping must work seamlessly during composition

### 3.3 Clipboard

- `Ctrl+C`: copy selected text as plain Markdown
- `Ctrl+V`: paste with Markdown detection (if clipboard has HTML, convert to Markdown first)
- `Ctrl+Shift+V`: paste as plain text
- `Ctrl+X`: cut selected text
- Support pasting images (insert as `![](path)` after saving to attachments folder)

---

## 4. Bidirectional Text (RTL/LTR)

### 4.1 Requirements

Constellation is used heavily in Arabic. Bidi support is non-negotiable.

- Implement the Unicode Bidirectional Algorithm (UAX #9) or use the browser's built-in bidi
- Support mixed Arabic/English text in a single line
- Cursor movement must follow visual order (not logical order) for arrow keys
- Text selection must follow visual order
- The `dir` attribute on the editor container controls base direction
- Auto-detect direction per paragraph based on first strong character
- Toolbar mirrors in RTL mode (flex-direction: row-reverse)

### 4.2 Approach

Leverage the browser's native bidi by using `<div>` elements with `dir="auto"` for each paragraph/line. The browser handles character reordering. We handle:
- Cursor positioning relative to bidi boundaries
- Selection painting across direction changes
- Arrow key navigation at direction boundaries

---

## 5. Markdown Parser

### 5.1 Incremental Parsing

Full document re-parse on every keystroke is too slow. The parser must be incremental:

- On text change: identify the affected block (paragraph, list item, code block, etc.)
- Re-parse only that block and its immediate neighbors
- Output: array of tokens with positions, types, and nesting levels
- Token types: heading, paragraph, bold, italic, strikethrough, underline, highlight, code, codeBlock, link, wikilink, image, list, taskList, blockquote, callout, table, horizontalRule, frontmatter, footnote, math

### 5.2 Block Structure

```
Document
├── Frontmatter (--- ... ---)
├── Heading (# ## ### etc.)
├── Paragraph (text with inline marks)
├── List (ordered, unordered, task)
│   └── ListItem
│       └── Paragraph / nested List
├── Blockquote
│   └── Callout ([!type])
├── CodeBlock (``` ... ```)
├── Table
│   ├── TableHeader
│   └── TableRow
├── HorizontalRule (---)
├── Image (standalone)
└── Math block ($$ ... $$)
```

### 5.3 Inline Marks

Within any paragraph or heading:
- **Bold**: `**text**` or `__text__`
- **Italic**: `*text*` or `_text_`
- **Strikethrough**: `~~text~~`
- **Highlight**: `==text==`
- **Underline**: `<u>text</u>` (HTML in Markdown)
- **Inline code**: `` `code` ``
- **Link**: `[text](url)` or `[[wikilink]]`
- **Image**: `![alt](src)`
- **Math**: `$expr$`
- **Subscript**: `<sub>text</sub>`
- **Superscript**: `<sup>text</sup>`
- **Font span**: `<span style="font-family: 'Name', fallback">text</span>`
- **Color span**: `<span style="color: #hex">text</span>`

---

## 6. Decoration Layer (WYSIWYG)

### 6.1 Concept

Decorations are visual overlays on the plain text. They don't change the underlying Markdown — they change how it looks.

**Types of decorations:**
1. **Hide** — make characters invisible (e.g., hide `**` around bold text)
2. **Style** — apply CSS to a text range (e.g., bold, italic, heading size)
3. **Widget** — replace a range with a custom DOM element (e.g., render an image, interactive checkbox)
4. **Line** — apply styling to an entire line (e.g., heading background, blockquote border)

### 6.2 Cursor Reveal

When the cursor enters a decorated region, the decoration is temporarily removed to reveal the Markdown syntax. This allows direct editing of the source.

**Rules:**
- Cursor on `**bold**` → show `**bold**` with visible asterisks
- Cursor leaves → show **bold** (decorated)
- The transition must be smooth (no layout jump)
- Only reveal the innermost decoration at cursor position

### 6.3 Decoration Map

After parsing, build a decoration map:

```
Position 0-15:  heading level 1 (hide "# ", style h1)
Position 20-28: bold (hide "**", style font-weight:700)
Position 30-45: wikilink (hide "[[" "]]", style link, make clickable)
Position 50-80: code block (style monospace, syntax highlight)
```

Decorations are rebuilt incrementally when the parser outputs new tokens.

---

## 7. Virtual Scrolling / Viewport Renderer

### 7.1 Requirements

- Only render lines visible in the viewport + small buffer above/below
- As user scrolls, create/recycle DOM elements
- Maintain a line height cache for accurate scroll position calculation
- Support variable-height lines (headings, images, code blocks)

### 7.2 Approach

- Maintain a flat array of line objects: `{ offset, length, height, element }`
- A "viewport window" tracks which lines are visible
- On scroll: compute new visible range, create/remove DOM elements
- Line height is measured after first render and cached
- For lines not yet measured, use an estimated height

---

## 8. Edit Engine

### 8.1 Commands

Every editing operation goes through a command:

```typescript
interface EditCommand {
  execute(buffer: PieceTable, selection: Selection): ChangeSet;
  undo(buffer: PieceTable, changeSet: ChangeSet): void;
}
```

**Built-in commands:**
- `insertText(text)` — insert at cursor
- `deleteSelection()` — delete selected text
- `toggleMark(syntax)` — wrap/unwrap selection with `**`, `*`, `~~`, etc.
- `setHeading(level)` — change line to heading
- `toggleList(type)` — toggle bullet/ordered/task list
- `insertTable(rows, cols)` — insert markdown table
- `insertLink(url, text)` — insert `[text](url)`
- `insertImage(src, alt)` — insert `![alt](src)`
- `insertCodeBlock(language)` — insert fenced code block
- `insertCallout(type)` — insert blockquote with `[!type]`
- `indent()` / `outdent()` — adjust indentation
- `setTextAlign(align)` — insert alignment marker or HTML attribute
- `setFontFamily(font)` — wrap selection in `<span style="font-family:...">`
- `setColor(color)` — wrap selection in `<span style="color:...">`

### 8.2 Undo/Redo

- Operation-based: each command produces a `ChangeSet` (list of inserts/deletes with positions)
- Undo stack: push change sets
- Redo stack: push undone change sets
- Group rapid keystrokes into a single undo step (by time threshold, e.g., 300ms pause)

### 8.3 Selection

- Support single cursor (caret) and range selection
- Selection is stored as `{ anchor: number, head: number }` (offsets into the buffer)
- Multiple cursors (future): array of selections
- Selection must work correctly with bidi text

---

## 9. Toolbar

### 9.1 Design

Follow Google Docs' toolbar order and grouping:

```
[Undo] [Redo] | [Heading ▾] [Font ▾] [Size ▾] | [B] [I] [U] [S] [Highlight] [Color ▾] |
[Align L] [Align C] [Align R] | [Bullet] [Numbered] [Task] | [Indent] [Outdent] |
[Link] [Image] [Table ▾] [Code] [Quote] [Callout ▾] [HR] | [Sub] [Sup] | [Clear] [Find] | [words]
```

### 9.2 Performance

**Critical: the toolbar must NOT cause re-renders on typing.**

- Toolbar buttons are static DOM elements
- Active state updates happen via direct DOM class manipulation (`classList.toggle`)
- A single `requestAnimationFrame` callback reads editor state and updates button classes
- Svelte reactivity is NOT used for button active states
- Result: zero Svelte re-renders during typing

### 9.3 RTL

- Toolbar has `dir` attribute matching the editor's direction
- In RTL: toolbar items flow right-to-left
- Dropdowns open in the correct direction

---

## 10. Context Menu (Right-Click)

### 10.1 Base Menu

Always shown:
- Cut (Ctrl+X)
- Copy (Ctrl+C)
- Paste (Ctrl+V)
- Paste as plain text (Ctrl+Shift+V)
- Select All (Ctrl+A)
- Separator
- Bold, Italic, Underline, Strikethrough, Highlight, Inline Code, Clear Formatting
- Separator
- Paragraph, Heading 1-4
- Separator
- Bullet List, Numbered List, Task List
- Blockquote, Code Block, Callout, Horizontal Rule
- Link, Image

### 10.2 Contextual Items

When cursor is inside specific elements, add relevant items:

- **In table**: Add Row ↑/↓, Add Column ←/→, Delete Row, Delete Column, Toggle Header, Delete Table
- **On link**: Edit Link, Open Link, Remove Link
- **On image**: Edit Alt Text, Resize, Remove
- **In code block**: Change Language, Convert to Plain Text
- **On callout**: Change Type submenu

### 10.3 Layout

- Always LTR internally (label left, shortcut right)
- Uses interface font
- Animated appearance (subtle fade-in)
- Positioned at click coordinates, adjusted to stay within viewport

---

## 11. Font System

### 11.1 Font Sets

Constellation has a font set system in Settings:
- **Universal mode**: one font set for all text
- **Per-language mode**: different font sets per script (Latin, Arabic, Hebrew, CJK, Devanagari, Cyrillic)

The editor applies fonts via CSS custom properties and `@font-face` with `unicode-range`.

### 11.2 Explicit Font Choice

Users can apply a specific font to selected text via the toolbar Font dropdown. This inserts:
```markdown
<span style="font-family: 'Font Name', fallback">text</span>
```

In the editor, this renders with the chosen font. In other Markdown apps, the text is still readable (HTML spans are valid in Markdown).

### 11.3 Font Size

Support font size changes via toolbar. Stored as:
```markdown
<span style="font-size: 18px">text</span>
```

---

## 12. Keyboard Shortcuts

### 12.1 Formatting

| Shortcut | Action |
|----------|--------|
| Ctrl+B | Bold |
| Ctrl+I | Italic |
| Ctrl+U | Underline |
| Ctrl+Shift+S | Strikethrough |
| Ctrl+Shift+H | Highlight |
| Ctrl+E | Inline code |
| Ctrl+K | Insert link |
| Ctrl+Shift+I | Insert image |

### 12.2 Blocks

| Shortcut | Action |
|----------|--------|
| Ctrl+1-6 | Heading 1-6 |
| Ctrl+0 | Paragraph |
| Ctrl+Shift+B | Bullet list |
| Ctrl+Shift+O | Ordered list |
| Ctrl+Shift+T | Task list |
| Ctrl+Shift+Q | Blockquote |
| Ctrl+Shift+C | Code block |

### 12.3 Editing

| Shortcut | Action |
|----------|--------|
| Ctrl+Z | Undo |
| Ctrl+Y / Ctrl+Shift+Z | Redo |
| Ctrl+A | Select all |
| Ctrl+F | Find |
| Ctrl+H | Find & Replace |
| Tab | Indent |
| Shift+Tab | Outdent |
| Ctrl+] | Indent |
| Ctrl+[ | Outdent |

---

## 13. Special Features

### 13.1 Wikilinks

`[[Note Name]]` renders as a clickable link:
- In WYSIWYG: hides `[[` `]]`, shows styled link
- Ctrl+Click: opens the linked note
- Autocomplete: type `[[` to trigger note name suggestions

### 13.2 Tables

- Interactive table editing in WYSIWYG mode
- Click cells to edit
- Tab to move between cells
- Toolbar button to insert with grid picker
- Context menu for row/column operations
- Underlying format is standard Markdown tables

### 13.3 Checkboxes

`- [ ]` and `- [x]` render as interactive checkboxes:
- Click to toggle
- Updates the underlying Markdown

### 13.4 Code Blocks

- Syntax highlighting (use a lightweight highlighter like Shiki or custom regex-based)
- Language label shown above the block
- Copy button
- Line numbers (optional)

### 13.5 Callouts

Obsidian-style callouts:
```markdown
> [!tip] Title
> Content here
```
Render with colored border, icon, and collapsible content.

### 13.6 Images

- Inline rendering (show the actual image)
- Drag to resize
- Alt text editing on click
- Support local paths and URLs

### 13.7 Math

- Inline `$expr$` renders as formatted math (use KaTeX)
- Block `$$ expr $$` renders as display math
- Click to edit source

### 13.8 Frontmatter

- `---` blocks at the top render as a properties panel (already built in Constellation)
- The editor should pass frontmatter to NotePane's existing property editor

---

## 14. Integration with Constellation

### 14.1 Component Interface

```typescript
interface ConstellationEditorProps {
  value: string;              // Markdown content
  dir: 'ltr' | 'rtl' | 'auto';
  placeholder?: string;
  onchange?: (markdown: string) => void;
  readonly?: boolean;
  allNotes?: { name: string; path: string }[];  // For wikilink autocomplete
  allTags?: string[];          // For tag autocomplete
}
```

### 14.2 Events

- `onchange(markdown)` — fired after edits (debounced, configurable)
- `onsave()` — fired on Ctrl+S
- `onlinkclick(target)` — fired when a wikilink is Ctrl+clicked

### 14.3 Drop-in Replacement

The editor must be a drop-in replacement for both `TipTapEditor.svelte` and `CodeMirrorEditor.svelte`. Same props interface, same event contract. NotePane should need minimal changes.

---

## 15. Performance Targets

| Metric | Target |
|--------|--------|
| Keystroke to screen | <2ms |
| Document load (10K lines) | <100ms |
| Document load (100K lines) | <500ms |
| Scroll (continuous) | 60fps |
| Memory (10K lines) | <50MB |
| Memory (100K lines) | <200MB |
| Initial render | <50ms |
| Toolbar update | <1ms (direct DOM) |

---

## 16. i18n

All user-facing strings must go through Constellation's `$t()` i18n system. The editor itself has no hardcoded strings. All 15 locales must be supported:
ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh

---

## 17. Technology Stack

- **Language**: TypeScript (strict mode)
- **Framework integration**: Svelte 5 component wrapper
- **Rendering**: DOM-based (not Canvas) for accessibility and CSS flexibility
- **Testing**: Vitest for unit tests, Playwright for integration
- **Bundle**: Tree-shakeable ES modules

---

## 18. File Structure

```
src/lib/editor/
├── ConstellationEditor.svelte   ← Svelte 5 wrapper component
├── core/
│   ├── PieceTable.ts            ← Text buffer
│   ├── Selection.ts             ← Cursor and selection
│   ├── EditEngine.ts            ← Commands, undo/redo
│   ├── InputHandler.ts          ← Keyboard, IME, clipboard
│   └── History.ts               ← Undo/redo stack
├── parser/
│   ├── MarkdownParser.ts        ← Incremental tokenizer
│   ├── tokens.ts                ← Token type definitions
│   └── InlineParser.ts          ← Inline mark parsing
├── render/
│   ├── ViewportRenderer.ts      ← Virtual scrolling, line rendering
│   ├── DecorationEngine.ts      ← WYSIWYG decorations
│   ├── LineLayout.ts            ← Line measurement, height cache
│   └── widgets/                 ← Custom widgets
│       ├── ImageWidget.ts
│       ├── TableWidget.ts
│       ├── CheckboxWidget.ts
│       ├── CodeBlockWidget.ts
│       ├── CalloutWidget.ts
│       └── MathWidget.ts
├── ui/
│   ├── Toolbar.svelte           ← Formatting toolbar
│   ├── ContextMenu.svelte       ← Right-click menu
│   ├── FindReplace.svelte       ← Search & replace
│   ├── TablePicker.svelte       ← Grid picker for table insertion
│   ├── LinkDialog.svelte        ← Link editing dialog
│   └── ColorPicker.svelte       ← Color selection
├── bidi/
│   ├── BidiResolver.ts          ← Bidi algorithm helpers
│   └── CursorMotion.ts          ← Visual cursor movement
├── extensions/
│   ├── WikilinkAutocomplete.ts  ← [[note]] suggestions
│   ├── TagAutocomplete.ts       ← #tag suggestions
│   ├── SyntaxHighlight.ts       ← Code block highlighting
│   └── MathRenderer.ts          ← KaTeX integration
└── theme/
    ├── editor.css               ← Base editor styles
    └── decorations.css          ← WYSIWYG decoration styles
```

---

## 19. Lessons from TipTap (What NOT to Do)

1. **Never convert markdown↔HTML on every keystroke** — the round-trip is inherently slow and lossy
2. **Never use Svelte reactivity for toolbar updates** — direct DOM manipulation only
3. **Never store the document as HTML/DOM** — markdown must be the source of truth
4. **Never use `tick` counters to force re-renders** — this defeats virtual DOM optimization
5. **Handle font-family quotes carefully** — single quotes inside double-quoted style attributes
6. **Test with Arabic text from day one** — bidi bugs are hard to fix retroactively
7. **The toolbar must be independent of the editor's update cycle** — zero coupling

---

## 20. Development Phases

### Phase 1: Core Engine
- Piece table buffer
- Basic input handling (insert, delete, arrow keys)
- Selection (caret and range)
- Undo/redo
- Viewport renderer (virtual scrolling)
- Basic paragraph rendering

### Phase 2: Markdown Parser + Basic Decorations
- Incremental block parser (headings, paragraphs, lists, code blocks)
- Inline parser (bold, italic, code, links)
- Hide/show syntax decorations
- Cursor reveal behavior

### Phase 3: WYSIWYG Features
- Headings rendered as actual headings
- Bold/italic/strikethrough visual rendering
- Links rendered as clickable
- Images rendered inline
- Checkboxes rendered as interactive
- Code blocks with syntax highlighting

### Phase 4: Advanced Features
- Tables (interactive editing)
- Callouts (colored, collapsible)
- Math rendering (KaTeX)
- Wikilink autocomplete
- Tag autocomplete
- Find & Replace

### Phase 5: UI & Polish
- Toolbar (Google Docs style)
- Context menu (right-click)
- Font family/size support
- Color picker
- RTL toolbar mirroring
- Keyboard shortcuts
- Accessibility

### Phase 6: Integration
- Drop-in replacement for NotePane
- Settings integration (font sets, editor preferences)
- Second screen support
- Performance optimization
- Testing across large documents

---

## 21. Constellation-Specific Context

### Current Codebase
- **App**: Tauri v2 desktop app (Rust + SvelteKit/Svelte 5)
- **Path**: `E:\مشاريع كلاود\Constellation`
- **Current editors**: `TipTapEditor.svelte` (WYSIWYG), `CodeMirrorEditor.svelte` (source)
- **Editor host**: `NotePane.svelte` — switches between editors based on `appSettings.editorType`
- **i18n**: 15 locales via `$t()` from `$lib/i18n`
- **RTL**: Full Arabic/Hebrew/Persian/Urdu support with `detectDir()` from `$lib/utils`
- **Fonts**: Font set system in `store.ts` (BUILTIN_FONT_SETS, SCRIPT_UNICODE_RANGES)
- **Settings**: `appSettings` store with `editorType`, `fontMode`, `activeFontSetId`, `languageFontSets`

### Terminology
- Use "Library" (never "vault")
- Use Svelte 5 runes: `$state`, `$derived`, `$effect`, `$props`
- All user-facing strings through `$t()`
- Update all 15 locale files for new strings

---

*This specification is the blueprint for the Constellation Editor. The goal is to build the fastest, most capable Markdown WYSIWYG editor available — purpose-built for Constellation.*
