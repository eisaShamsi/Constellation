---
aliases:
  - Properties
  - Frontmatter
  - YAML properties
  - Metadata
  - Note properties
description: Learn how to view, edit, and manage YAML frontmatter properties in Constellation, including type detection, auto-dates, templates, and property search.
---

# Properties

Properties are YAML frontmatter metadata stored at the top of each note. Constellation reads, displays, and lets you edit properties inline, with full support for Arabic keys, type persistence, and auto-date population.

You can view and edit properties in the **Properties** tab of the right sidebar, or in the properties section at the top of a note pane.

---

## Supported property types

Constellation supports eight property types, each with a dedicated editor:

| Type | Icon | Editor |
|------|------|--------|
| Text | `T` | Plain text input |
| Number | `#` | Numeric input |
| Date | Calendar | Native date picker with locale display |
| Datetime | Clock | Native datetime-local picker |
| List | List | Tag-style chips with inline add/remove |
| Link | Chain | Wikilink input, clickable to navigate |
| Checkbox | Check | Toggle switch |
| Nested fields | Branch | Read-only summary of the fields it contains (see below) |

---

## Automatic type detection

When a property is loaded, Constellation automatically detects its type based on the key name and value format.

### Key-based detection

Certain key names are recognized automatically:

| Type | Recognized keys |
|------|----------------|
| Date | `date`, `created`, `updated`, `modified`, `due`, `start`, `end`, `deadline`, `completed_date`, `published`, `أنشئ`, `حُدث`, `تاريخ`, `تعديل`, `موعد`, `بداية`, `نهاية` |
| List | `tags`, `aliases`, `cssclasses`, `cssclass`, `related`, `categories`, `group`, `الوسم`, `وسوم`, `المجموعة`, `ذات صلة`, `أسماء بديلة`, `تصنيفات` |
| Checkbox | `done`, `completed`, `draft`, `publish`, `published`, `pinned`, `archived`, `starred`, `todo`, `favorite`, `featured`, `hidden`, `مكتمل`, `منشور`, `مسودة`, `مثبت`, `مؤرشف`, `مميز`, `مخفي` |

### Value-based detection

If the key is not recognized, the value format is checked:

- **YYYY-MM-DD** or **DD/MM/YYYY** patterns are detected as Date
- **YYYY-MM-DDTHH:MM** patterns are detected as Datetime
- **`true`** or **`false`** values are detected as Checkbox
- **Numeric** values are detected as Number
- **YAML arrays** are detected as List
- **`[[wikilink]]`** values are detected as Link
- Everything else defaults to Text

### Date format normalization

Dates entered in **DD/MM/YYYY** format (common in many locales) are automatically normalized to **YYYY-MM-DD** for storage, ensuring compatibility with other Markdown tools and Lens.

---

## Type selector

Click the type icon next to any property to open the type selector dropdown. This lets you manually override the detected type for any property.

### How to use

1. Click the type icon (e.g. `T` for text) to the left of the property key.
2. A dropdown appears with all seven types, each showing its icon and translated name.
3. Click the desired type to apply it.
4. Press **Escape** or click outside to close without changing.

### Type persistence

When you change a property's type, the choice is remembered library-wide. The same key will be recognized as that type in all notes within the same library. Type assignments persist across sessions via local storage.

---

## Editing properties

### Inline editing

Click any property value to edit it inline. The editor adapts to the property type:

- **Text / Number**: Direct input field
- **Date**: Native date picker, with a locale-formatted label displayed alongside (e.g. "17 September 2025" or "17 سبتمبر 2025")
- **Datetime**: Native datetime-local picker
- **Checkbox**: Click to toggle between true and false
- **List**: Tag-style chips. Type in the input and press **Enter** or **comma** to add items. Click the **x** on a chip to remove it.
- **Link**: Text input for wikilink names. The value is displayed as a clickable link that opens the target note.

### Empty placeholders

When a property value is blank, a muted italic placeholder is shown:

- **Text / Link / Number**: "Empty" (or "فارغ" in Arabic)
- **Date**: "dd/mm/yyyy"
- **List**: "Empty" (or "فارغ" in Arabic)

### Properties that contain other fields

Most properties hold a single value — `stage: growth-seed`. A few hold a **block of
their own fields**:

```yaml
source:
  title: Muqaddimah
  author: Ibn Khaldun
  year: 1377
```

Here `source` doesn't have a value of its own; it has three fields underneath it. The
inner `title` belongs to the *source*, not to the note.

**How it appears.** The row shows the names of the fields it contains as small chips —
*title*, *author*, *year* — followed by a faint **read-only** label. Hovering that label
explains why: nested fields are kept exactly as written in the file and cannot be edited
in this panel.

**What you can and cannot do.** The whole row is read-only: you can't type into it, the
field name itself isn't editable, and there's no remove button. Right-clicking offers
**Copy value** (which copies the field names) and **Copy name**, but not *Remove
property*. Everything else on the note — adding tags, changing the stage, renaming the
note — works normally and leaves the block untouched.

**Why it's locked, for now.** Constellation guarantees that a block it can't fully edit
is a block it won't damage. Editing the row would mean replacing the whole block with a
single value, so instead the panel shows you what's there and stays out of the way. Your
file keeps the nested fields exactly as you wrote them.

> [!note] This is temporary
> Being able to edit nested fields directly in the panel is planned. Until then the rule
> is simple and safe: Constellation shows you the block, and never rewrites it.

**To edit them today**, open the `.md` file in any text editor — the fields are ordinary
YAML and nothing in Constellation will overwrite your changes.

### Adding a new property

Click the **+ Add property** button below the properties list. A new row appears with an empty key and value. Type the key name and press **Tab** to move to the value.

### Removing a property

Click the **trash icon** that appears on hover at the end of each property row.

### Reordering properties

Drag a property row by its **grip handle** (left edge) to reorder. Properties are saved in the order they appear.

---

## Property key suggestions

When typing a property key, a suggestion dropdown appears with common keys in both English and Arabic:

| English | Arabic |
|---------|--------|
| tags | الوسم |
| aliases | أسماء بديلة |
| date | تاريخ |
| created | أنشئ |
| updated | حُدث |
| author | المؤلف |
| source | المصدر |
| status | الحالة |
| type | النوع |
| category | الفئة |
| description | الوصف |
| image | الصورة |
| cover | الغلاف |
| cssclasses | — |
| publish | منشور |
| permalink | — |

The list filters as you type. Press **Up/Down** arrows to navigate, **Enter** to select, or **Escape** to dismiss.

---

## Clickable links

Properties detected as **Link** type display their value as a clickable link. Clicking the link opens the target note in a new tab:

- The link is resolved as a wikilink across all libraries (cross-library linking).
- If the target note exists, it opens directly.
- Link values are styled as clickable text with an underline on hover.

---

## Auto-populated dates

Constellation automatically manages date properties when creating and saving notes.

### On note creation

When you create a new note, a `created` property is automatically added with the current date in `YYYY-MM-DD` format.

### On note save

When you save a note that already contains an `updated`, `modified`, `حُدث`, or `تعديل` property, that property is automatically updated to the current date.

> [!tip]
> Auto-dates only update properties that already exist in the note. If you want `updated` to be tracked, add it once and Constellation will keep it current.

---

## Templates

When creating a new note, Constellation checks for a default template at `{library}/Templates/default.md`. If found:

- The template's YAML frontmatter is merged with the auto-generated properties (like `created`).
- The template's body content is used as the initial note content.

This lets you define a standard set of properties and boilerplate content for all new notes in a library.

> [!tip]
> Create a `Templates` folder in your library root and add a `default.md` file with your preferred frontmatter and content structure.

---

## Property search

You can search notes by their property values using the special `[key:value]` syntax in the search bar.

### How to use

1. Open the search panel (**Ctrl+Shift+F** or the magnifying glass icon).
2. Type a query in the format `[key:value]`.
3. Constellation searches all libraries for notes whose frontmatter contains a matching key-value pair.

### Examples

| Query | Finds |
|-------|-------|
| `[tags:journal]` | Notes with "journal" in their tags |
| `[status:draft]` | Notes where status is "draft" |
| `[المؤلف:أحمد]` | Notes where المؤلف contains "أحمد" |
| `[created:2025-09-17]` | Notes created on that date |

The search matches partial values — `[tags:jour]` will match notes tagged "journal".

---

## RTL and multilingual support

Properties fully support right-to-left (RTL) languages:

- Arabic, Hebrew, Urdu, and Persian property keys are displayed correctly.
- The property editor respects the document direction.
- Key suggestions include both English and Arabic common keys.
- Type names in the dropdown are translated into the current interface language.

---

## Keyboard shortcuts

| Action | Shortcut |
|--------|----------|
| Save properties | Automatic on change |
| Add new property | Click **+ Add property** |
| Navigate key suggestions | Up / Down arrows |
| Select key suggestion | Enter |
| Dismiss dropdown | Escape |
| Add list item | Enter or Comma |
| Remove list item | Click x on chip |

---

## Tips

> [!tip] Markdown tools compatibility
> Properties are stored as standard YAML frontmatter, fully compatible with any Markdown tool that reads frontmatter.

> [!tip] Type persistence saves time
> Once you set a property type (e.g. making "status" a List), it applies library-wide. You won't need to set it again for other notes.

> [!tip] Use templates for consistency
> Create a `Templates/default.md` in your library to ensure every new note starts with the same properties structure.
