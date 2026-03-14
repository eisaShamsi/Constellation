---
aliases:
  - Dataview
  - Dataview queries
  - DQL
  - Inline queries
  - Data queries
description: Learn how to use Dataview queries in Constellation to dynamically display lists and tables of notes based on their properties, tags, and folders.
---

# Dataview

Dataview lets you write queries inside your notes that dynamically display lists and tables of matching notes. Queries are written in **Dataview Query Language (DQL)** inside fenced code blocks, and results update automatically from your vault's [[Properties|frontmatter properties]].

> [!tip] Built-in feature
> Dataview is a native Constellation feature. Unlike the Obsidian plugin, queries run on the Rust backend for fast performance across large vaults and multi-vault universes.

---

## Writing a Query

Create a fenced code block with the language set to `dataview`:

````markdown
```dataview
TABLE status, priority FROM "Projects" WHERE status != "done" SORT priority ASC
```
````

When you view the note in reading mode, the code block is replaced with an interactive table showing matching notes.

In the editor (Live Preview), the block collapses into a compact **Dataview** label showing a preview of the query. Click the label to expand and edit the query.

---

## Query Types

### TABLE

Displays results as a table with the file name in the first column and additional property columns.

````markdown
```dataview
TABLE author, date, status FROM "Books" SORT date DESC
```
````

| File | author | date | status |
|------|--------|------|--------|
| The Great Gatsby | F. Scott Fitzgerald | 1925 | read |
| Dune | Frank Herbert | 1965 | reading |

### LIST

Displays results as a simple list of note links.

````markdown
```dataview
LIST FROM #project WHERE status = "active"
```
````

---

## Query Syntax

A DQL query follows this structure:

```
TYPE [properties] FROM source WHERE condition SORT property ORDER LIMIT number
```

### FROM — Source

| Syntax | Description |
|--------|-------------|
| `FROM "FolderName"` | Notes from a specific folder |
| `FROM #tag` | Notes with a specific tag |
| *(omit FROM)* | All notes across all vaults |

### WHERE — Filter

| Operator | Example | Description |
|----------|---------|-------------|
| `=` | `WHERE status = "done"` | Equals |
| `!=` | `WHERE status != "done"` | Not equals |
| `>` | `WHERE priority > 3` | Greater than |
| `<` | `WHERE priority < 3` | Less than |
| `>=` | `WHERE priority >= 3` | Greater than or equal |
| `<=` | `WHERE priority <= 3` | Less than or equal |
| `CONTAINS` | `WHERE tags CONTAINS "work"` | Value contains text |
| `IS_EMPTY` | `WHERE status IS_EMPTY` | Property is empty or missing |

### SORT — Order

```
SORT property ASC    — Ascending (A-Z, 0-9)
SORT property DESC   — Descending (Z-A, 9-0)
```

### LIMIT — Cap Results

```
LIMIT 20    — Show at most 20 results
```

### GROUP BY

```
GROUP BY status    — Group results by a property value
```

---

## Examples

### All notes modified recently
````markdown
```dataview
TABLE modified FROM "" SORT modified DESC LIMIT 10
```
````

### Tasks by priority
````markdown
```dataview
TABLE priority, due FROM #task WHERE status != "done" SORT priority ASC
```
````

### Books I'm reading
````markdown
```dataview
LIST FROM "Reading" WHERE status = "reading"
```
````

### Notes without a status property
````markdown
```dataview
TABLE title FROM "Projects" WHERE status IS_EMPTY
```
````

---

## Interactive Features

- **Clickable file links** — Click any file name in the results to open that note
- **Collapse/expand** — Click the toggle arrow in the header to collapse the results
- **Refresh** — Hover over the header and click the refresh icon to re-run the query
- **Result count** — The footer shows the number of matching notes and query time

---

## Multi-Vault Queries

Dataview queries run across **all vaults** in your active universe. If you use `FROM "Projects"`, it will find the `Projects` folder in any vault. Results include the vault name so you can distinguish notes from different vaults.

---

## Tips

> [!tip] Property names are case-sensitive
> `WHERE Status = "done"` is different from `WHERE status = "done"`. Use the exact property name as it appears in your frontmatter.

> [!tip] Quoted strings
> Always wrap text values in double quotes: `WHERE status = "done"`. Folder names in `FROM` must also be quoted: `FROM "My Folder"`.

> [!tip] Performance
> Queries run on the Rust backend and are typically fast even with thousands of notes. Use `LIMIT` to cap large result sets for faster rendering.

---

## RTL and Arabic Support

Dataview tables automatically detect RTL content using Constellation's bidirectional text detection. Property values in Arabic, Hebrew, Urdu, or Farsi will display with the correct text direction.

---

## Comparison with Obsidian Dataview Plugin

| Feature | Constellation | Obsidian Plugin |
|---------|--------------|-----------------|
| Query engine | Rust (native, fast) | JavaScript |
| Multi-vault | Yes (all vaults in universe) | Single vault only |
| DQL syntax | TABLE, LIST | TABLE, LIST, TASK, CALENDAR |
| Inline queries | Not yet | Yes (`= this.property`) |
| DataviewJS | Not supported | Yes |
| Custom output | Not yet | Yes (via JS) |

> [!info] Coming soon
> TASK and CALENDAR query types will be available in future updates as part of the Tasks and Calendar features.
