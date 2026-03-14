---
aliases:
  - Import Notes
  - Note Importer
  - Migration
description: Import notes from other apps like Notion, Evernote, Bear, or plain files into your Constellation vaults.
---

# Importer

Constellation's built-in Importer lets you bring notes from other apps into your vaults. It supports multiple formats and converts them to Obsidian-compatible Markdown.

## Supported Formats

| Format | Source | What It Does |
|--------|--------|--------------|
| **Markdown** | Any app | Copies `.md` files preserving folder structure and attachments (images, PDFs) |
| **Notion** | Notion export | Cleans Notion-style filenames (removes hex IDs), converts Notion links to `[[wikilinks]]` |
| **Bear** | Bear export | Imports Bear's markdown export folder |
| **Evernote** | `.enex` file | Parses ENML content, extracts tags and dates into YAML frontmatter |
| **HTML** | `.html` files | Converts headings, links, lists, code blocks, and formatting to Markdown |
| **CSV** | `.csv`/`.tsv` file | Creates one note per row, with columns as frontmatter properties |
| **Plain Text** | `.txt` files | Wraps text content in Markdown with a heading |

## How to Use

1. Open the Importer via:
   - **Command palette** → "Import notes"
   - **Ribbon** → Import button (download arrow icon)
2. **Select format** — choose the source format
3. **Choose target vault** and optional subfolder (default: "Imported")
4. **Select source** — pick the file(s) or folder
5. **Preview** — review what will be imported
6. **Import** — click "Import now" to begin

## Import Details

### Markdown / Bear

- Copies all `.md` files preserving the original folder structure
- Also copies images and attachments (PNG, JPG, GIF, SVG, WebP, PDF)
- Skips files that already exist in the destination (no overwriting)
- Ignores hidden folders (starting with `.`)

### Notion

- Removes the 32-character hex ID that Notion appends to filenames
- Converts `[Page Title](notion.so/...)` links to `[[Page Title]]` wikilinks
- Preserves all other Markdown formatting

### Evernote (.enex)

- Extracts individual notes from the `.enex` XML file
- Converts ENML (Evernote Markup Language) to Markdown
- Creates YAML frontmatter with:
  - `created` date from Evernote's timestamp
  - `tags` array from Evernote tags

### HTML

- Converts headings (`<h1>`–`<h6>`) to `#` syntax
- Converts bold, italic, links, images, lists, code blocks, and blockquotes
- Strips remaining HTML tags
- Decodes HTML entities

### CSV / TSV

- First row is used as column headers
- First column becomes the note title
- Remaining columns become frontmatter properties
- Auto-detects comma vs tab separator

### Plain Text

- Each `.txt` file becomes a `.md` file
- The filename is used as the heading
- Content is preserved as-is

## Safety

- **No overwriting**: Existing files are never overwritten. Duplicates are skipped and counted.
- **Vault validation**: The target path is validated to ensure it's within a registered vault.
- **Error reporting**: Any files that fail to import are listed with their error messages.

> [!tip]
> Create a dedicated subfolder like "Imported" to keep your imported notes organized and separate from your existing notes.

> [!tip] RTL Support
> The Importer interface works correctly in both LTR and RTL modes.
