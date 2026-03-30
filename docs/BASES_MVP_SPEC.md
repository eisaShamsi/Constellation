# Constellation Bases — MVP Specification (v0.1)

**Codename: Star Charts**

> "If a Star is a note and a Universe is a library, then a **Star Chart** is a structured view of your stars — filtered, sorted, grouped, and displayed the way your mind organizes them."

---

## Grounding Principles

Every decision in this spec is constrained by Constellation's four non-negotiable principles:

| Principle | Implication for Bases |
|---|---|
| **You own everything** | `.base` files are plain YAML/JSON stored in the library. No proprietary binary formats. |
| **All local** | All queries, filtering, and sorting happen on-device in Rust. Zero network calls. |
| **Read and write in place** | Editing a cell writes directly to the note's YAML frontmatter. No shadow database. |
| **Non-destructive** | Delete Constellation → `.base` files are inert text files. Your notes are untouched. |

**Additional constraint**: Bases must be **RTL-native from day one**. Not retrofitted. Arabic column headers, right-to-left table flow, and bidirectional cell content are first-class requirements, not afterthoughts.

---

## What Bases Is (and Is Not)

**Bases IS:**
- A structured view layer over your existing notes and their YAML frontmatter properties
- A visual query builder — no code, no query language
- A way to create, filter, sort, and group notes by their properties
- An interactive editor — changing a cell changes the note's frontmatter

**Bases IS NOT:**
- A separate database engine (no SQLite, no IndexedDB)
- A replacement for notes (every "row" is still a .md file)
- A Notion clone (no blocks-as-rows, no pages-as-databases)
- A Dataview replacement (Bases is visual-first; Dataview is code-first)

---

## File Format: `.base`

A `.base` file lives in the library like any other file. It is plain YAML that defines what to show and how.

```yaml
# books.base
version: 1
name: "📚 مكتبتي"       # Display name (supports Arabic/emoji)
source:
  type: folder           # folder | tag | library | all
  path: "Books"          # Folder path (relative to library root)
  # tag: "#research"     # Alternative: filter by tag
  # library: "Islamic Sources"  # Alternative: specific library
  includeSubfolders: true

columns:
  - property: "title"
    label: "العنوان"      # Custom column label (optional)
    width: 250
    visible: true
  - property: "author"
    label: "المؤلف"
    width: 180
  - property: "status"
    width: 120
  - property: "rating"
    width: 80
  - property: "date_read"
    width: 120

filters:
  - property: "status"
    operator: "is"        # is | is_not | contains | gt | lt | is_empty | is_not_empty
    value: "reading"

sorts:
  - property: "rating"
    direction: "desc"     # asc | desc

# groupBy:               # Phase 2
#   property: "status"

view: table               # table | card | list
direction: auto           # auto | rtl | ltr (auto = detect from content)
```

### Why YAML?
- Human-readable in any text editor
- Standard format (consistent with frontmatter)
- Diffable in git
- Non-destructive: if Constellation is removed, it's just a text file

---

## Architecture

### Data Flow

```
.base file (YAML)
       ↓ parse
Base Definition (TypeScript)
       ↓ query
Rust Backend: scan folder → read each note's frontmatter → return rows
       ↓ filter + sort (Rust, fast)
Filtered/Sorted Row Data
       ↓ render
Svelte 5 Component (reactive table/card/list)
       ↓ edit cell
Write YAML frontmatter change → Rust save_note → file watcher triggers refresh
```

### Key Design Decisions

1. **Rust does the heavy lifting.** Scanning 10,000 notes and extracting frontmatter properties is a Rust task, not a TypeScript task. The IPC returns structured row data.

2. **No shadow state.** There is no in-memory database. Every query re-scans the source (with filesystem caching in Rust for performance). The library files are the single source of truth.

3. **Incremental indexing.** The Rust file watcher already exists. When a note changes, only that row is re-read, not the entire Base.

4. **Cross-library support.** Since Constellation's core differentiator is multi-library, Bases can query across libraries: `source.type: all` scans every registered library.

---

## MVP Scope (v0.1)

### In Scope

| Feature | Priority |
|---|---|
| `.base` file format (YAML) | Must |
| Table view (sortable columns) | Must |
| Card view (grid of cards) | Must |
| List view (compact rows) | Must |
| Filter by property (visual builder) | Must |
| Sort by property (multi-sort) | Must |
| Inline cell editing → writes to YAML | Must |
| Create new note from Base (with pre-filled properties) | Must |
| RTL/Bidi: full table direction, per-column override | Must |
| Column visibility, width, reorder | Must |
| File tree integration (`.base` files show with icon) | Must |
| Embed Base in note (`![[books.base]]`) | Should |
| Source types: folder, tag | Must |
| Source types: library, all (cross-library) | Should |
| Column types: text, number, date, checkbox, list, link | Must |
| Empty state with "Create your first Base" wizard | Must |

### Out of Scope (v0.2+)

| Feature | Phase |
|---|---|
| Grouping and sub-grouping | v0.2 |
| Kanban view | v0.2 |
| Calendar view | v0.2 |
| Relational columns (note links as typed relations) | v0.3 |
| Rollup / aggregation columns | v0.3 |
| Formula columns | v0.3 |
| Gallery / image view | v0.3 |
| Timeline view | v0.4 |
| Task aggregation (body-level queries) | v0.4 |
| Conditional formatting | v0.4 |
| Publish integration | v0.5 |

---

## Implementation Plan

### Step 1: Rust Backend — Base Query Engine

**File: `src-tauri/src/bases.rs`** (new)

```rust
// Core Tauri commands:

#[tauri::command]
fn parse_base_file(file_path: String) -> Result<BaseDefinition, String>
// Reads and parses a .base YAML file

#[tauri::command]
fn query_base(definition: BaseDefinition, vault_paths: Vec<String>) -> Result<BaseQueryResult, String>
// Scans source (folder/tag/library), extracts frontmatter from each note,
// applies filters, applies sorts, returns structured rows

#[tauri::command]
fn update_note_property(file_path: String, key: String, value: String) -> Result<(), String>
// Writes a single property change to a note's YAML frontmatter
// (reuse existing frontmatter write logic from store)
```

**Data structures:**
```rust
struct BaseDefinition {
    version: u32,
    name: String,
    source: BaseSource,
    columns: Vec<ColumnDef>,
    filters: Vec<FilterRule>,
    sorts: Vec<SortRule>,
    view: String,           // "table" | "card" | "list"
    direction: String,      // "auto" | "rtl" | "ltr"
}

struct BaseRow {
    file_path: String,
    file_name: String,
    vault_name: String,
    properties: HashMap<String, String>,
    modified: u64,          // unix timestamp
}

struct BaseQueryResult {
    rows: Vec<BaseRow>,
    total_count: usize,
    query_time_ms: u64,     // for performance monitoring
}
```

**Performance target:** < 100ms for 5,000 notes with 10 properties each.

### Step 2: Base File Format Parser

**File: `src/lib/bases/parser.ts`** (new)

- Parse `.base` YAML into TypeScript `BaseDefinition` type
- Validate schema (version, required fields)
- Provide defaults for optional fields (direction: auto, view: table)

### Step 3: Base View Component

**File: `src/lib/components/BaseView.svelte`** (new)

The main component that renders a Base. Receives a `BaseDefinition` and orchestrates the view.

```
BaseView.svelte
├── BaseToolbar.svelte        (view switcher, filter/sort controls, new note button)
├── BaseTableView.svelte      (table with sortable headers, editable cells)
├── BaseCardView.svelte       (responsive grid of property cards)
├── BaseListView.svelte       (compact row list)
├── BaseFilterBuilder.svelte  (visual filter rule builder)
└── BaseSortBuilder.svelte    (visual sort order builder)
```

### Step 4: Table View (Core)

The table view is the most complex and most important view. Requirements:

- **Column headers** with click-to-sort, drag-to-reorder, resize handles
- **Cell types** rendered by property type:
  - `text` → editable text input
  - `number` → number input with locale formatting
  - `date` → date display with date picker on click
  - `checkbox` → clickable toggle
  - `list` → comma-separated tags with pills
  - `link` → clickable wikilink that opens the note
- **Row click** → opens the note in a new tab
- **RTL mode**: columns flow right-to-left, text aligns right, scroll direction inverts
- **Sticky first column** (note name) during horizontal scroll
- **Virtual scrolling** for performance with 1,000+ rows

### Step 5: Card View

A responsive CSS Grid of cards. Each card shows:
- Note title (header)
- 3-4 configurable property values
- Library color indicator
- Click to open note

### Step 6: List View

A compact single-line-per-note view:
- Note title + selected properties inline
- Checkbox support for quick task completion
- Compact enough for sidebar embedding

### Step 7: Inline Editing

When a user edits a cell:
1. Cell enters edit mode (input/select/datepicker based on type)
2. On blur/Enter, call `update_note_property` Rust command
3. Rust reads the note, modifies the YAML frontmatter, writes it back
4. File watcher detects change → row refreshes reactively

**Critical**: Use the existing `parseFrontmatter` / `reconstructFrontmatter` logic from `store.ts` as reference. The Rust implementation must produce identical YAML output.

### Step 8: File Tree Integration

- `.base` files appear in the file tree with a distinct icon (grid/table icon)
- Clicking a `.base` file opens it in BaseView (not as raw text)
- Right-click context menu: "Open as Base view" / "Open as source"
- "New Base" option in folder context menu

### Step 9: Note Creation from Base

When clicking "New Note" in a Base:
1. Create a new `.md` file in the Base's source folder
2. Pre-populate frontmatter with all columns defined in the Base
3. If the Base has active filters, pre-fill those filter values (e.g., if filtered to `status: active`, new note gets `status: active`)
4. If a template folder is configured, apply the template
5. Open the new note for editing

### Step 10: Base Embedding

Support `![[books.base]]` transclusion syntax:
- In reading mode, render the Base view inline (read-only, compact)
- In editing mode, show a placeholder with "Click to open Base"
- Respect the `.base` file's view type and filters

---

## RTL Implementation

RTL is not a feature flag — it's woven into every component:

| Component | RTL Behavior |
|---|---|
| Table headers | Flow right-to-left; sort arrows mirror |
| Table cells | Text aligns right by default; numbers stay LTR |
| Filter builder | Labels and dropdowns mirror; text inputs are RTL |
| Card grid | Cards flow right-to-left |
| Toolbar | Buttons mirror; search input is RTL |
| Column reorder | Drag handles on the correct side |

**Direction detection:**
- `direction: auto` → detect from the Base's `name` field (if Arabic/Hebrew → RTL)
- `direction: rtl` / `ltr` → explicit override
- Per-column `direction` override for mixed-language Bases

---

## Property Type System

Bases leverages the existing property type registry (`src/lib/libraries/propertyTypeRegistry.ts`). The type system:

| Type | Cell Renderer | Cell Editor | Sort Logic |
|---|---|---|---|
| `text` | Plain text | Text input | `localeCompare` (Arabic-aware) |
| `number` | Formatted number | Number input | Numeric |
| `date` | Locale-formatted date | Date picker | Chronological |
| `datetime` | Locale-formatted datetime | Datetime picker | Chronological |
| `checkbox` | ✓ / ✗ toggle | Click toggle | Boolean |
| `list` | Tag pills | Comma-separated input | By first item |
| `link` | Clickable wikilink | Note autocomplete | Alphabetical |

**Arabic-aware sorting**: All text sorts use `localeCompare` with the appropriate locale parameter. Arabic names sort correctly by default.

---

## Performance Budget

| Metric | Target | Method |
|---|---|---|
| Base file parse | < 5ms | YAML parse in TypeScript |
| Query 1,000 notes | < 50ms | Rust parallel frontmatter scan |
| Query 10,000 notes | < 200ms | Rust with filesystem cache |
| Render 100 rows | < 16ms (one frame) | Virtual scrolling, Svelte 5 reactivity |
| Cell edit → save | < 100ms | Direct Rust file write |
| Cross-library query (5 libraries, 50K notes) | < 500ms | Parallel library scan |

---

## File Structure (New Files)

```
src/
├── lib/
│   ├── bases/
│   │   ├── parser.ts           # .base YAML parser + validator
│   │   ├── types.ts            # TypeScript interfaces
│   │   └── utils.ts            # Formatting, sorting helpers
│   └── components/
│       ├── BaseView.svelte     # Main Base container
│       ├── BaseToolbar.svelte  # View switcher, filters, sorts
│       ├── BaseTableView.svelte
│       ├── BaseCardView.svelte
│       ├── BaseListView.svelte
│       ├── BaseFilterBuilder.svelte
│       ├── BaseSortBuilder.svelte
│       └── BaseCellEditor.svelte  # Type-aware cell editor
src-tauri/
└── src/
    ├── bases.rs                # Query engine, property writer
    └── lib.rs                  # Register new commands
```

---

## Success Criteria

The MVP is complete when:

1. A user can create a `.base` file (via UI wizard or manually) that queries a folder
2. The Base renders as an interactive table with sortable, filterable columns
3. Editing a cell in the table updates the source note's YAML frontmatter
4. Card and List views work with the same data
5. An Arabic-language library can create an RTL Base with Arabic headers and content that renders correctly without any CSS workarounds
6. Creating a new note from a Base pre-populates the frontmatter
7. Performance meets the budget above
8. `.base` files are plain YAML readable in any text editor

---

## What This Enables (v0.2+ Vision)

Once the MVP ships, the foundation supports rapid iteration:

- **v0.2**: Grouping (rows collapse under group headers), Kanban (group-by as columns with drag-and-drop)
- **v0.3**: Relations (link columns that reference other notes), Rollups (aggregate across relations), Gallery view
- **v0.4**: Calendar view (date properties plotted on a grid), Timeline, Task aggregation, Formula columns
- **v0.5**: Conditional formatting, Publish integration

Each version builds on the same `.base` format (backward-compatible), the same Rust query engine (extended, never rewritten), and the same Svelte component architecture.

---

*This spec is a living document. Update it as implementation decisions are made.*
