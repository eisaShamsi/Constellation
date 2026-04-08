---
aliases:
  - Table Editor
  - Table Formatting
  - Table Formulas
description: Edit, sort, format, and calculate markdown tables with an interactive toolbar and formula support.
---

# Advanced Tables

Constellation includes a built-in advanced table editor that makes working with markdown tables fast and intuitive. When your cursor is inside a table, a floating toolbar appears with actions for editing, sorting, moving, and calculating.

## Table Toolbar

Place your cursor inside any markdown table to reveal the floating toolbar above it.

### Basic Actions

| Button | Action |
|--------|--------|
| **+ Row** | Add a new row below the current row |
| **+ Col** | Add a new column after the current column |
| **- Row** | Delete the current row (cannot delete header) |
| **- Col** | Delete the current column |

### Alignment

| Button | Action |
|--------|--------|
| Align Left | Set current column to left alignment |
| Align Center | Set current column to center alignment |
| Align Right | Set current column to right alignment |

### More Actions Menu

Click the **...** button to access additional features:

#### Move

| Button | Action |
|--------|--------|
| Move Row Up | Swap current row with the row above |
| Move Row Down | Swap current row with the row below |
| Move Col Left | Swap current column with the column to the left |
| Move Col Right | Swap current column with the column to the right |

> [!tip]
> The header row cannot be moved. Move operations only apply to data rows.

#### Sort

| Button | Action |
|--------|--------|
| A→Z | Sort all data rows by the current column in ascending order |
| Z→A | Sort all data rows by the current column in descending order |

Sorting is smart: numeric values are compared numerically, text values are compared alphabetically. The header row always stays in place.

#### Formulas

| Button | Action |
|--------|--------|
| =SUM | Insert a SUM formula at the current cell |
| Eval | Evaluate all formulas in the table, replacing them with calculated values |

## Formulas

Constellation supports spreadsheet-style formulas in table cells.

### Supported Functions

| Function | Description | Example |
|----------|-------------|---------|
| `=SUM(range)` | Sum of values | `=SUM(A1:A5)` |
| `=AVG(range)` | Average of values | `=AVG(B1:B3)` |
| `=COUNT(range)` | Count of numeric values | `=COUNT(A1:A10)` |
| `=MIN(range)` | Minimum value | `=MIN(C1:C5)` |
| `=MAX(range)` | Maximum value | `=MAX(C1:C5)` |

### Cell References

- **Single cell:** `A1`, `B3`, `C10` (column letter + row number)
- **Range:** `A1:A5` (all cells from A1 to A5), `A1:C3` (rectangular range)
- Row numbers start at 1 (first data row after header)
- Column letters: A = first column, B = second, etc.

### Example

```markdown
| Item   | Price |
| ------ | ----- |
| Apple  | 3     |
| Banana | 2     |
| Cherry | 5     |
| Total  | =SUM(B1:B3) |
```

Click **Eval** in the toolbar to calculate:

```markdown
| Item   | Price |
| ------ | ----- |
| Apple  | 3     |
| Banana | 2     |
| Cherry | 5     |
| Total  | 10    |
```

## Tab Navigation

Press **Tab** to move to the next cell, **Shift+Tab** to move to the previous cell. When you Tab past the last cell in the last row, a new row is automatically added.

## Auto-Formatting

Tables are automatically formatted with consistent column widths when you make any change through the toolbar. This keeps your markdown clean and readable.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Tab` | Move to next cell |
| `Shift+Tab` | Move to previous cell |

## Slash Command

Type `/table` at the beginning of a line to insert a new 2-column table template.

> [!tip] RTL Support
> Tables work correctly in both LTR and RTL editing modes. The toolbar and all operations respect the current text direction.
