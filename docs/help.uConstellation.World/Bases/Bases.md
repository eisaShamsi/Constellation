---
aliases:
  - Bases
  - Constellation Base
  - Note tables
  - Structured views
  - Base files
description: Learn how to use the Constellation Base — a live table of your notes, one row per note and one column per property, that you can sort, edit, and reshape without ever moving a file.
---

# Bases

A **Base** turns a set of your notes into a live table: **one row per note, one column per property**. Nothing is copied or moved — the table reads your notes in place and reflects them as they are right now.

> [!tip] Strong yet Simple, by default
> A Base opens looking familiar and uncluttered — just your notes' names and the fields you care about. Constellation's deeper, cognitive columns are always **one click away**, but they never crowd the first screen. You decide how much structure to pull in.

> [!info] Non-destructive
> A Base never changes your notes on its own. It is a small `.base` file holding a query — "show these notes, with these columns, in this order." Your Markdown files stay exactly where they are.

---

## Two ways to use a Base

**1. As a full tab.** Open a `.base` file and it fills the tab as an interactive table.

**2. Inside a note.** Drop a fenced code block into any note and it renders inline:

````markdown
```base
view: table
```
````

Both are powered by the same engine, so they behave identically.

---

## Creating a Base

Use **New Base** from the sidebar (the "+" / New Base action). Constellation writes a small **YAML** `.base` file for you:

```yaml
schema: 1
lens: My Notes
scope:
  libraries: all
  federation: auto
columns:
  - dimension: note.name
view: table
```

| Field | Meaning |
|-------|---------|
| `schema` | Format version (currently `1`). |
| `lens` | The name shown at the top of the table. |
| `scope.libraries` | `all`, or a list of specific libraries to include. |
| `scope.federation` | `auto` — also include notes from any linked Universes (cUniverses). |
| `columns` | The columns to show. A new Base starts with just the note **Name**. |
| `view` | `table` (the table is the Base view). |

You rarely need to edit this by hand — the table's own controls (below) write every change back to the file for you.

---

## The table

- **Name column** — always first. Click a note's name to open it.
- **Every matching note becomes a row.** There is **no row limit**. The table is *virtualized* — it only draws the rows currently on screen — so a Base over thousands of notes opens instantly and scrolls smoothly.
- **Per-cell direction** — each value detects its own left-to-right or right-to-left script, so mixed-language tables read correctly.
- The footer shows how long the query took.

---

## Columns — add, remove, reorder

### Add a column

Click **+ Add column**. The picker is grouped in two:

- **Your fields** — the frontmatter properties Constellation found in your notes (for example `status`, `maturity`, `author`). These are *your* data.
- **Constellation** — built-in fields the app always knows: **Name**, **Path**, **Created**, and **Summary**.

Start typing to filter the list. Fields already in the table are marked so you don't add them twice.

### Remove a column

Hover a column header and click the **×**.

### Reorder columns

**Press and drag a column header sideways.** The whole column lifts (it dims and the header shows a grab outline), and a vertical line marks where it will drop. Release to move it. The Name column stays fixed as the first column.

Every add, remove, and reorder is saved back to the `.base` file automatically.

---

## Sorting

**Click a column header to sort by it.** Each click cycles **ascending → descending → off** (an arrow shows the current direction).

For sorting by more than one column, open the **Sort** panel:

- Add several columns — the first is the primary sort, the next break ties.
- Flip any level between ascending and descending.
- Move levels up or down to change priority, or remove them.

---

## Editing a note from the table

Double-click a cell in one of **your** frontmatter columns to edit it:

- **Free-text fields** — type the new value; **Enter** saves, **Escape** cancels.
- **List-type fields** (like `maturity`) — a **dropdown** appears with the valid values **in their natural order** (for `maturity`: *seed → sapling → evergreen → canonical*). Pick one, or type your own.

The change is written straight to that note's YAML frontmatter on disk, and the table updates in place.

> [!note] Read-only columns
> **Name** and **Created** (and the other built-in Constellation columns) are computed for you, so they aren't editable. Only your own frontmatter fields can be changed here.

---

## Opening an older Base

If you switch from Obsidian, or from an earlier version of Constellation, your existing `.base` files use an older format.

**Your file is never touched.** When Constellation opens one, it shows a calm notice explaining the format is older, and offers a **Convert to Constellation Base** button. Conversion happens **only when you click it** — it upgrades the file in place to the new YAML format (carrying over what it can: the name, the columns, and simple text filters). Until you choose to convert, the original file is left exactly as it was.

---

## Federation

A Base is Universe-aware. With `federation: auto`, it includes notes from any linked Universes (cUniverses) alongside your own. Notes that live in a linked Universe are read-only — you can view and sort them in the Base, but editing is reserved for notes you own.

---

## Local-first & file-over-app

Bases hold no data of their own. Every value you see comes from a real `.md` file on your disk, read live. Delete the `.base` file and your notes are completely unaffected — a Base is just a lens you point at notes you already have.

---

## Keyboard & mouse

| Action | What it does |
|--------|--------------|
| **Click** a column header | Sort by it (ascending → descending → off) |
| **Drag** a column header | Reorder that column |
| **Click** the × on a header | Remove that column |
| **Double-click** a frontmatter cell | Edit it (dropdown for list fields) |
| **Enter** | Save the edit |
| **Escape** | Cancel the edit |
| **Click** a note's name | Open the note |
