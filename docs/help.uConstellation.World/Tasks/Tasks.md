---
aliases:
  - Tasks Panel
  - Task Management
  - Global Tasks
description: Create, manage, and track tasks across all your libraries with due dates, priorities, and interactive checkboxes.
---

# Tasks

Constellation includes a built-in task management system that scans your notes for task items (`- [ ]` and `- [x]`), extracts metadata like due dates and priorities, and provides interactive panels for managing them.

## Creating Tasks

Tasks use standard Markdown checkbox syntax:

| Syntax | Result |
|--------|--------|
| `- [ ] Buy groceries` | Unchecked task |
| `- [x] Send email` | Completed task |

### Adding Metadata

You can enrich tasks with metadata using emoji markers or inline fields:

| Metadata | Emoji Format | Inline Field Format |
|----------|-------------|-------------------|
| Due date | `- [ ] Task text 📅 2026-03-20` | `- [ ] Task [due:: 2026-03-20]` |
| High priority | `- [ ] Task ⏫` | `- [ ] Task [priority:: high]` |
| Medium priority | `- [ ] Task 🔼` | `- [ ] Task [priority:: medium]` |
| Low priority | `- [ ] Task 🔽` | `- [ ] Task [priority:: low]` |
| Created date | `- [ ] Task ➕ 2026-03-14` | `- [ ] Task [created:: 2026-03-14]` |
| Tags | `- [ ] Task #work #urgent` | Same |

When you complete a task, a completion date is automatically added: `✅ 2026-03-14`.

## Tasks Panel (Right Sidebar)

The Tasks tab in the right sidebar shows all tasks from the **currently active note**.

### Features

- **Filter**: Toggle between All, Incomplete, and Completed tasks
- **Sort**: Order by default, due date, or priority
- **Interactive checkboxes**: Click to toggle completion directly from the sidebar
- **Due date badges**: Color-coded (red = overdue, yellow = due today, gray = upcoming)
- **Priority icons**: Visual indicators for task priority
- **File links**: Click the file name to navigate to the note

> [!tip]
> When you toggle a task in the sidebar, the note content updates automatically in the editor.

## Global Tasks View

Access the full-page Global Tasks view to see tasks across **all libraries**:

- Click the **☑ checkbox icon** in the left ribbon
- Or use **Mission Control** (`Ctrl+P`) and search for "Global Tasks"

### Filtering & Sorting

| Filter | Options |
|--------|---------|
| Status | All, Incomplete, Completed |
| Due Date | All dates, Overdue, Today, This week, No date |
| Library | All libraries or a specific library |
| Priority | All, High, Medium, Low |
| Search | Free-text search across task text, file names, and tags |

### Grouping

Group tasks by:
- **File** - Tasks grouped under their source note
- **Library** - Tasks grouped by library
- **Priority** - High, Medium, Low, No priority
- **Due date** - Overdue, Today, and by date

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/task` | Insert a task item (in editor) |
| `Ctrl+Enter` | Toggle checkbox on current line (in editor) |
| `Escape` | Close Global Tasks view |

> [!tip] RTL Support
> Tasks panels fully support right-to-left languages. Due date badges, priority icons, and filter controls all respect the current text direction.


---

## The universe task agenda (left dock)

The right-sidebar **Tasks** tab shows the open note's tasks. To see **every** task across all your libraries at once, open the **Tasks** button in the **left dock** — a full-page agenda with filter chips (All / Incomplete / Completed), dropdowns (by date, by library, by priority), *Group by file*, *Sort by due date*, and search.

**Appearance.** The agenda follows your current universe's theme automatically. To fine-tune it, open **Style Setter → Global Tasks** — colour controls (background, text, accent, borders, overdue / due-today dates) plus a **Text size** slider that scales the agenda's text.

**Natural-language due dates.** While typing a task, type **`@today`**, **`@tomorrow`**, **`@yesterday`**, **`@next week`**, **`@next month`**, a weekday name (**`@monday`**), or **`@in 3 days`** / **`@in 2 weeks`** and accept the autosuggestion — it pins a fixed real date (for example `📅 2026-06-25`). You don't have to add the `📅` yourself. Turn the feature on or off in **Settings → Editor → Natural-language task dates**.
