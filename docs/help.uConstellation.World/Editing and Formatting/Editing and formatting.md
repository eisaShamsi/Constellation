---
aliases:
  - Editing
  - Formatting
  - Editor
  - Editor settings
  - Markdown editing
  - Keyboard shortcuts
  - Live preview
  - Formatting toolbar
  - Table editing
  - Code folding
  - Multiple cursors
  - Indentation guides
  - Auto-pair
  - Properties in document
description: Learn how to edit and format notes in Constellation, including keyboard shortcuts, the formatting toolbar, table editing, live preview mode, code folding, multiple cursors, and editor settings.
---

# Editing and formatting

Constellation offers two editor modes, selectable in **Settings > Editor**:

| Mode | Engine | Best for |
|------|--------|----------|
| **Markdown** (default) | CodeMirror 6 | Power users who prefer direct Markdown syntax control |
| **Document** | TipTap (ProseMirror) | Word-processor experience — WYSIWYG editing like Google Docs or Microsoft Word |

Both modes save notes as standard Markdown files. The Document editor converts between HTML and Markdown transparently — your files remain portable regardless of which mode you use.

### Switching editor mode

Go to **Settings > Editor > Editor type** and choose **Markdown editor** or **Document editor**.

---

## Document editor (TipTap)

The Document editor provides a familiar word-processor toolbar with buttons for:

| Button | Action |
|--------|--------|
| **Undo / Redo** | Step through edit history |
| **H1 / H2 / H3** | Heading levels |
| **B / I / U / S** | Bold, Italic, Underline, Strikethrough |
| **Highlight** | Background highlight |
| **Align** | Left, Center, Right text alignment |
| **Lists** | Bullet list, Numbered list, Task list |
| **Blockquote** | Indented quote block |
| **Code block** | Fenced code block |
| **Horizontal rule** | Divider line |
| **Table** | Insert/manage tables (rows, columns, delete) |
| **Link** | Insert hyperlink |
| **Image** | Insert image |

All standard keyboard shortcuts (Ctrl+B, Ctrl+I, Ctrl+Z, etc.) work in the Document editor.

> [!tip]
> The Document editor is ideal for users who prefer not to learn Markdown syntax. Everything you type is rendered immediately as formatted text.

---

## Markdown editor (CodeMirror)

The Markdown editor gives you full control over the raw Markdown syntax with live preview, formatting toolbar, and smart editing features.

---

## Keyboard shortcuts

### Text formatting

| Shortcut | Action | Markdown |
|----------|--------|----------|
| **Ctrl+B** | Bold | `**text**` |
| **Ctrl+I** | Italic | `_text_` |
| **Ctrl+Shift+S** | Strikethrough | `~~text~~` |
| **Ctrl+Shift+H** | Highlight | `==text==` |
| **Ctrl+`** | Inline code | `` `text` `` |
| **Ctrl+K** | Insert wikilink | `[[text]]` |

All formatting shortcuts wrap the current selection. If no text is selected, they insert the markers at the cursor position.

### Editing shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+D** | Select next occurrence of the current selection |
| **Ctrl+Shift+D** | Duplicate the current line |
| **Ctrl+/** | Toggle comment (`%%...%%`) |
| **Ctrl+Shift+V** | Paste as plain text |
| **Ctrl+Z** | Undo |
| **Ctrl+Shift+Z** | Redo |
| **Tab** | Indent (or navigate to next table cell) |
| **Shift+Tab** | Outdent (or navigate to previous table cell) |

### Folding shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+Shift+[** | Fold the section at cursor |
| **Ctrl+Shift+]** | Unfold the section at cursor |

You can also use the **Fold all sections** and **Unfold all sections** commands from Mission Control (**Ctrl+P**).

### Navigation shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+O** | Star Jump |
| **Ctrl+P** | Mission Control |
| **Ctrl+Shift+F** | Search library |
| **Ctrl+G** | Open Sky View |
| **Ctrl+N** | New note |

---

## Formatting toolbar

When you select text in the editor, a floating toolbar appears above your selection with quick formatting buttons:

| Button | Action |
|--------|--------|
| **B** | Bold |
| *I* | Italic |
| ~~S~~ | Strikethrough |
| ==H== | Highlight |
| `<>` | Inline code |
| Link | Wikilink |
| H | Heading (dropdown for H1-H6) |

### How it works

1. Select any text in the editor.
2. The toolbar appears above your selection.
3. Click a button to apply formatting.
4. The toolbar disappears when the selection is cleared.

The heading button opens a dropdown where you can choose heading levels 1 through 6. Selecting a heading level replaces the line prefix with the appropriate number of `#` characters.

> [!tip]
> The toolbar buttons preserve your text selection, so you can apply multiple formats in sequence without re-selecting.

---

## Table editing

Constellation includes a built-in table editor that makes working with Markdown tables easy.

### Table toolbar

When your cursor is inside a Markdown table, a floating toolbar appears with table-specific actions:

| Button | Action |
|--------|--------|
| **+ Row** | Add a new row below the current row |
| **+ Col** | Add a new column to the right |
| **- Row** | Delete the current row |
| **- Col** | Delete the current column |
| Align left | Set column alignment to left |
| Align center | Set column alignment to center |
| Align right | Set column alignment to right |

### Cell navigation

- **Tab** moves the cursor to the next cell in the table.
- **Shift+Tab** moves the cursor to the previous cell.
- When you reach the last cell and press Tab, a new row is automatically created.

### Table formatting

When you modify a table using the toolbar, Constellation automatically formats the table with aligned pipes and consistent column widths for clean, readable Markdown.

### Creating a table

Type a Markdown table manually or use the `/table` slash command to insert a starter table:

```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
```

---

## Live preview

Live preview mode renders Markdown formatting inline as you type, hiding syntax characters when your cursor is not on that line.

### Enabling live preview

- Click the **book icon** button next to the edit/read mode toggle in the note header.
- Or use Mission Control: **Toggle live preview** (**Ctrl+P**, then search for it).

### What renders in live preview

| Markdown | Rendered as |
|----------|------------|
| `# Heading` | Large, bold heading (H1-H6 sizes) |
| `**bold**` | **Bold text** |
| `_italic_` | *Italic text* |
| `~~strikethrough~~` | ~~Strikethrough text~~ |
| `==highlight==` | Highlighted text |
| `` `code` `` | Monospace code |
| `[[link]]` | Accent-colored clickable link |
| `- [ ]` / `- [x]` | Clickable checkbox |
| `> quote` | Styled blockquote |
| `---` | Horizontal rule |

### Cursor-aware rendering

When your cursor is on a line, the full Markdown syntax is shown so you can edit it. When you move to a different line, the syntax markers are hidden and the formatted result is displayed. This gives you the best of both worlds: readable formatting and full control over the raw Markdown.

> [!tip]
> Live preview only processes visible lines for performance. Even in very long notes, the editor stays responsive.

---

## Section folding

You can fold (collapse) sections of your note to focus on specific parts.

### Heading folding

Click the fold arrow in the gutter next to any heading to collapse everything until the next heading of the same or higher level. For example, folding an `## H2` heading hides all content up to the next `## H2` or `# H1`.

### List folding

Nested list items can be folded to hide their children. Click the fold arrow next to a list item that has indented sub-items.

### Commands

- **Fold all sections**: Collapses all foldable sections in the note.
- **Unfold all sections**: Expands all folded sections.

Both commands are available from Mission Control (**Ctrl+P**).

---

## Multiple cursors

Constellation supports multiple cursors for editing several locations simultaneously.

### Adding cursors

- **Alt+Click** places an additional cursor at the clicked location.
- **Ctrl+D** selects the next occurrence of the current selection, adding a new cursor.
- **Shift+Alt+Drag** creates a rectangular (column) selection.

### Using multiple cursors

Once you have multiple cursors, any text you type is inserted at all cursor positions. Formatting shortcuts (Bold, Italic, etc.) also apply to all selections simultaneously. Press **Escape** to collapse back to a single cursor.

---

## Smart editing features

### Smart lists

When you press **Enter** at the end of a list item, Constellation automatically continues the list:

- Unordered lists (`-`, `*`, `+`) continue with the same marker.
- Ordered lists (`1.`, `2.`) increment the number.
- Pressing **Enter** on an empty list item removes the marker and exits the list.

### Auto-pair brackets

The editor automatically inserts matching closing characters:

| You type | Editor inserts |
|----------|---------------|
| `(` | `()` |
| `[` | `[]` |
| `{` | `{}` |
| `"` | `""` |

### Auto-pair Markdown syntax

When enabled, Markdown formatting symbols are also auto-paired:

| You type | Editor inserts |
|----------|---------------|
| `` ` `` | ` `` ` |
| `*` | `**` |
| `_` | `__` |
| `~` | `~~` |
| `=` | `==` |

When text is selected, typing any of these characters wraps the selection. For example, selecting a word and pressing `*` wraps it as `*word*`. Pressing `*` again upgrades it to `**word**`.

### Slash commands

Type `/` at the beginning of a line to open the slash command menu with quick insertions:

| Command | Inserts |
|---------|---------|
| `/heading1` - `/heading6` | Heading |
| `/bold`, `/italic`, `/strikethrough` | Text formatting |
| `/code` | Code block |
| `/quote` | Blockquote |
| `/link` | Wikilink |
| `/image` | Image |
| `/table` | Table template |
| `/hr` | Horizontal rule |
| `/task` | Task checkbox |
| `/callout` | Callout block |
| `/date` | Current date |
| `/time` | Current time |

### Wikilink autocomplete

Type `[[` to open the note autocomplete. Start typing a note name and matching notes from all libraries appear. Press **Enter** to insert the link. You can also link to specific headings with `[[note#heading]]`.

### Tag autocomplete

Type `#` to open the tag autocomplete. Existing tags from your libraries are suggested as you type.

---

## Renaming a note from its title line

Every note carries its title above the text, and you can rename the note by typing straight into
it. However you leave that field, the new title is kept:

- **Press Enter** — the cursor returns to the body and the rename is applied.
- **Click anywhere else** — the rename is applied as you leave.
- **Press Escape** — this is the "take me back to writing" key, and it now applies the rename too.

Previously Escape was the one exit that threw the new title away without a word, so a note you had
just renamed kept its old name. If you want to abandon a rename, put the old title back before you
leave the field.

## Editor settings

You can customize the editor behavior in **Settings > Editor**. Settings are organized into three sections:

### General

| Setting | Description | Default |
|---------|-------------|---------|
| Always focus new tabs | When you open a link in a new tab, switch to it immediately | On |
| Restore tabs on relaunch | Reopen the tabs from your last session when the app starts. Turning this off also deletes the remembered session | On |
| Default view for new tabs | Reading or editing view when opening a new note | Reading view |
| Default editing mode | Live Preview or Source mode when entering edit mode | Live Preview |

#### Restore tabs on relaunch

With this on (the default), closing and reopening Constellation puts your desk back the way you left it: the same tabs in the same order, the same active tab, and the split view if you had one. The memory is per-Universe and updates quietly about a second after you open, close, or rearrange tabs — a crash or force-kill loses at most the last second of arrangement, never note content. A note that was moved or deleted while the app was closed is simply skipped; the rest still return. Named Workspaces are unaffected — they stay your deliberate, hand-saved snapshots.

### Display

| Setting | Description | Default |
|---------|-------------|---------|
| Readable line length | Limit maximum line length for comfortable reading | On |
| Properties in document | How properties are shown at the top of notes — Visible, Hidden, or Source (raw YAML) | Visible |
| Fold heading | Lets you fold all content under a heading | On |
| Fold indent | Lets you fold part of an indentation, such as lists | On |
| Line numbers | Show line numbers in the gutter | On |
| Indentation guides | Show vertical relationship lines between nested list items | Off |

### Behavior

| Setting | Description | Default |
|---------|-------------|---------|
| Spellcheck | Turn on the spellchecker | Off |
| Auto-pair brackets | Pair brackets `()`, `[]`, `{}` and quotes `""`, `''` automatically | On |
| Auto-pair Markdown syntax | Pair symbols automatically for bold `**`, italic `_`, code `` ` ``, strikethrough `~~`, and highlight `==` | On |
| Smart lists | Automatically set indentation and place list items correctly on Enter | On |
| Indent using tabs | Use tabs to indent by pressing the Tab key. Turn this off to indent using spaces | On |
| Tab size | Number of spaces a tab character will render as (2 or 4) | 4 |

> [!tip] Properties display mode
> Set **Properties in document** to **Hidden** if you want a cleaner editing experience without frontmatter distractions. Set it to **Source** to see and edit the raw YAML directly.

---

## Enhanced Toolbar

The toolbar includes a toggle button (≡) to show/hide all buttons. New formatting options:

- **Underline** (`<u>`) — renders underlined in Live Preview
- **Subscript** (`<sub>`) — for chemical formulas, footnotes
- **Superscript** (`<sup>`) — for exponents, ordinals
- **Text Alignment** — Align Start, Center, End (direction-aware for RTL)
- **Clear Formatting** — removes all markdown and HTML marks from selection
- **Find & Replace** — opens search panel (also Ctrl+F)

The toolbar is RTL-aware: when your cursor is on an Arabic or Hebrew line, the toolbar flips to match the text direction. Undo/redo arrows and alignment icons mirror automatically.

---

## Tips

> [!tip] Formatting shortcuts work on selections
> Select text first, then press **Ctrl+B** to bold it. If nothing is selected, the markers are inserted at the cursor and you can type between them.

> [!tip] Use Ctrl+D for quick renaming
> Select a word, then press **Ctrl+D** repeatedly to select each occurrence. Type the replacement and all occurrences update at once.

> [!tip] Tab navigates tables
> When your cursor is inside a table, **Tab** and **Shift+Tab** move between cells instead of inserting indentation. This makes table editing fast and natural.

> [!tip] Live preview + editing mode
> Live preview is a sub-mode of editing mode. You get inline rendering while retaining full edit capabilities. Toggle it on for a cleaner writing experience.
