---
aliases:
  - Bases
  - Database views
  - Note databases
  - Structured views
  - Base files
description: Learn how to create and use Bases in Constellation to view, filter, sort, and edit your notes as structured databases with table, card, and list views.
---

# Bases

Bases let you view your notes as structured databases. A Base collects notes from a folder (or by tag), reads their [[Properties|frontmatter properties]], and displays them in a dynamic table, card grid, or list view — all without copying or moving your files.

> [!tip] Core principle
> Bases are **non-destructive**. Your notes stay exactly where they are. A `.base` file is just a small JSON query definition that tells Constellation which notes to show and how to display them.

---

## Creating a Base

To create a Base, create a file with the `.base` extension in your library. The file contains a JSON definition:

```json
{
  "version": 1,
  "name": "My Projects",
  "source": {
    "type": "folder",
    "path": "Projects",
    "includeSubfolders": true
  },
  "columns": [],
  "filters": [],
  "sorts": [],
  "view": "table",
  "direction": "auto"
}
```

When you open a `.base` file in Constellation, it renders as an interactive database view instead of a text editor.

---

## Source types

The `source` field defines which notes to include:

| Source type | Description | Example |
|-------------|-------------|---------|
| `folder` | Notes in a specific folder | `"type": "folder", "path": "Projects"` |
| `tag` | Notes with a specific tag | `"type": "tag", "tag": "project"` |
| `library` | All notes in a library | `"type": "library"` |

### Subfolders

When using `folder` source, set `"includeSubfolders": true` to include notes in nested folders.

---

## Views

Bases support three view modes, switchable from the toolbar:

### Table view

The default view. Displays notes as rows in a spreadsheet-like table with:

- **Sortable columns** for each detected property
- **Resizable columns** — drag the column edge to resize
- **Reorderable columns** — drag column headers to rearrange
- **Inline editing** — double-click any cell to edit the value
- **Type-aware cells** — checkboxes toggle, links are clickable, tags show as pills

### Card view

A responsive grid of cards, ideal for visual browsing. Each card shows the note name and up to 6 properties.

### List view

A compact, single-line-per-note view showing the note name and up to 4 properties. Supports inline editing by double-clicking values.

---

## Columns

Columns determine which properties are displayed and in what order.

- If no columns are defined, Constellation **auto-detects** columns from the properties found in your notes
- You can drag column headers in table view to reorder them
- Column order is saved to the `.base` file automatically

### Column definition

```json
{
  "property": "status",
  "label": "Status",
  "width": 150,
  "visible": true
}
```

| Field | Description |
|-------|-------------|
| `property` | The frontmatter key to display |
| `label` | Display name (defaults to property name) |
| `width` | Column width in pixels (table view) |
| `visible` | Whether to show the column |

---

## Filtering

Click the filter button in the toolbar to open the filter builder. Filters narrow down which notes are displayed.

### Filter operators

| Operator | Description |
|----------|-------------|
| is | Exact match |
| is not | Excludes exact match |
| contains | Value contains the text |
| does not contain | Value does not contain the text |
| greater than | Numeric/date comparison |
| less than | Numeric/date comparison |
| is empty | Property has no value |
| is not empty | Property has a value |

You can add multiple filters — they are combined with AND logic (all filters must match).

> [!tip] Pre-filled new notes
> When you create a new note from a Base with active "is" filters, the new note's frontmatter is pre-populated with those filter values.

---

## Sorting

Click the sort button in the toolbar to open the sort builder. You can:

- Sort by any property
- Choose ascending or descending order
- Add multiple sort levels (first sort takes priority)
- Reorder sort levels with the up/down arrows

Constellation automatically detects numeric values and sorts them numerically rather than alphabetically.

---

## Editing notes from a Base

You can edit property values directly from table and list views:

- **Text, number, date**: Double-click the cell to edit, press Enter to save, Escape to cancel
- **Checkbox**: Click to toggle between true/false
- **Links**: Click to navigate to the linked note

Changes are saved immediately to the note's frontmatter on disk.

---

## Toolbar

The Base toolbar provides quick access to all features:

| Button | Action |
|--------|--------|
| View switcher (table/card/list icons) | Switch between view modes |
| Filter funnel | Toggle filter builder (badge shows active filter count) |
| Sort arrows | Toggle sort builder (badge shows active sort count) |
| Plus (+) | Create a new note in the Base's source folder |
| Refresh | Re-run the query to pick up external changes |

---

## The `.base` file format

A `.base` file is a plain JSON file with the following structure:

```json
{
  "version": 1,
  "name": "Display Name",
  "source": { "type": "folder", "path": "..." },
  "columns": [],
  "filters": [],
  "sorts": [],
  "view": "table",
  "direction": "auto"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `version` | number | Schema version (currently 1) |
| `name` | string | Display name shown in the toolbar |
| `source` | object | Which notes to query |
| `columns` | array | Column definitions (empty = auto-detect) |
| `filters` | array | Active filter rules |
| `sorts` | array | Active sort rules |
| `view` | string | Current view mode: `table`, `card`, or `list` |
| `direction` | string | Text direction: `auto`, `ltr`, or `rtl` |

---

## RTL support

Bases fully support right-to-left layouts:

- Set `"direction": "rtl"` for Arabic/Hebrew content
- Set `"direction": "auto"` to detect direction from the Base name
- Individual cell values auto-detect their text direction
- Column headers, filter/sort builders, and all UI elements respect the direction

---

## Performance

Bases query your notes directly from disk each time they load. The query time is displayed in the footer (e.g., "42 results in 12ms").

> [!tip] Tips for fast queries
> - Use `folder` source instead of `library` when possible
> - Keep your Base focused on specific folders rather than entire libraries
> - The query engine skips notes without YAML frontmatter automatically

---

## Cross-library support

Bases work across multiple libraries. The library name is displayed alongside each note in card and list views, helping you identify which library a note belongs to.

---

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| Double-click cell | Start editing (table/list view) |
| Enter | Confirm edit |
| Escape | Cancel edit |
| Click checkbox | Toggle value |
| Click note name | Open note |
