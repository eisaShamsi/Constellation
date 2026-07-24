---
aliases:
  - Templater
  - Template Engine
  - Template Variables
description: Create and use templates with dynamic variables for consistent note creation.
---

# Templates

Constellation's built-in template engine lets you create reusable note templates with dynamic variables that are automatically replaced when inserted.

## Setting Up Templates

1. Go to **Settings > Templates**
2. Set the **Template folder** (default: `Templates`)
3. Create `.md` files inside this folder — each file is a template

## Template Variables

Use double curly braces to insert dynamic content:

| Variable | Replaced With | Example Output |
|----------|--------------|----------------|
| `{{date}}` | Current date | 2026-03-14 |
| `{{date:FORMAT}}` | Custom date format | See formats below |
| `{{time}}` | Current time (HH:MM) | 14:30 |
| `{{title}}` | Note title (without .md) | My New Note |
| `{{folder}}` | Parent folder name | Projects |
| `{{library}}` | Library name | Personal |
| `{{cursor}}` | Cursor position after insert | *(removed, cursor placed here)* |

### Date Format Tokens

Use `{{date:FORMAT}}` with these tokens:

| Token | Meaning | Example |
|-------|---------|---------|
| `YYYY` | 4-digit year | 2026 |
| `YY` | 2-digit year | 26 |
| `MMMM` | Full month name | March |
| `MMM` | Short month name | Mar |
| `MM` | Zero-padded month | 03 |
| `DD` | Zero-padded day | 14 |
| `dddd` | Full weekday | Saturday |
| `ddd` | Short weekday | Sat |
| `HH` | Hours (24h) | 14 |
| `mm` | Minutes | 30 |
| `ss` | Seconds | 05 |

**Example:** `{{date:YYYY/MM/DD dddd}}` produces `2026/03/14 Saturday`

## Using Templates

### Method 1: Template Picker (Ctrl+T)

1. Press **Ctrl+T** or use Mission Control ("Insert from template")
2. Search and select a template
3. Template content is inserted at your cursor position with all variables processed

### Method 2: Slash Command

1. Type `/template` at the beginning of a line in the editor
2. Select from the autocomplete dropdown
3. The Template Picker opens for selection

### Method 3: New Note from Template

Instead of inserting a template into a note you already have, you can start a
brand-new note from one:

1. Open Mission Control and run **New note from template**. (This is different
   from **Start from a template…**, which appears on an empty note you already
   created and fills *that* note — no location question there, the note already
   has one.)
2. Pick the template.
3. A small dialog asks for the new note's **title** — the template's name is
   pre-filled; accept it or type your own.
4. The same dialog **shows where the note will be created** — the folder line
   under the title reads like the sidebar does, e.g. `Personal / Projects /
   Ideas`. Constellation proposes the folder of the note you are currently
   working in.
5. If that's not where you want it, press **Change…** — a folder tree of your
   whole Universe opens (every library, every folder, searchable). Pick the
   destination and you're back at the title dialog, now showing your choice.
6. Press **OK** — the note is created in the shown folder and opens in a new
   tab.

If **no note is open**, there is nothing to propose — so the folder tree opens
*first*: choose where the note should live, then confirm its title. The note is
never placed somewhere you didn't see.

### Method 4: New Note Default Template

1. Create a file named `default.md` in your template folder
2. Every new note (Ctrl+N) will automatically use this template

### Method 5: Daily Note Template

1. Go to **Settings > Templates**
2. Set **Daily note template** to the file name (e.g., `Daily`)
3. When creating a daily note, this template is applied automatically

## Example Template

Create a file `Templates/Meeting.md`:

```markdown
---
description: Meeting notes template
---

# {{title}}

**Date:** {{date}}
**Time:** {{time}}

## Attendees
-

## Agenda
{{cursor}}

## Notes

## Action Items
- [ ]
```

> [!tip]
> The template's own frontmatter (like `description`) is stripped when inserted. Only the body content is used.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | Open Template Picker |
| `/template` | Slash command in editor |
| `Ctrl+N` | New note (uses `default.md` template if present) |

> [!tip] RTL Support
> Template variables work correctly in both LTR and RTL notes. The Template Picker respects the current text direction.
