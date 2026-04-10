# Canonical Filename Architecture — Design Document

**Status**: Design  
**Date**: 2026-04-10

---

## 1. Overview

Every file Constellation manages — notes, images, attachments, links, templates, and any future type — receives a **canonical filename** that serves as its immutable primary key. Users never see or type these filenames; Constellation presents human-chosen titles everywhere.

### Format

```
YYYYMMDDTHHMMSSZ_KIND_XXXX.ext
```

### Example

```
20260410T153045Z_NOTE_7F3A.md       ← user sees "Agriculture System"
20250315T120000Z_IMG_E5F6.png       ← user sees "Farm diagram"
20260401T091200Z_LINK_A1B2.md       ← user sees "Agriculture → Irrigation"
```

---

## 2. File Kind Registry

### 2.1 Core Kinds (ship with Constellation)

| Code | Kind | Extensions | Description |
|------|------|-----------|-------------|
| `NOTE` | Note | `.md` | The atomic knowledge unit |
| `BASE` | Base | `.md` | Structured database / records |
| `TMPL` | Template | `.md` | Reusable note structure |
| `LINK` | Link | `.md` | Typed connection between knowledge objects |
| `IMG` | Image | `.png` `.jpg` `.jpeg` `.gif` `.svg` `.webp` `.bmp` `.ico` `.tiff` | Visual media |
| `AUD` | Audio | `.mp3` `.wav` `.ogg` `.m4a` `.flac` `.aac` `.wma` | Sound files |
| `VID` | Video | `.mp4` `.webm` `.mov` `.mkv` `.avi` `.wmv` | Video files |
| `ATT` | Attachment | `.pdf` `.docx` `.xlsx` `.pptx` `.epub` `.zip` `.rar` | Non-native documents |
| `CANVAS` | Canvas | `.canvas` `.json` | Visual node-based maps |
| `DRAW` | Drawing | `.excalidraw` | Sketches, diagrams |
| `MARK` | Bookmark | `.md` | Saved URL with metadata/annotations |
| `CLIP` | Clip | `.md` | Web clipping, captured excerpt |

### 2.2 Auto-Generated Kinds

When a user adds a file with an unknown extension:

```
1. Take the extension (without dot)
2. Uppercase it
3. Truncate to 6 characters
4. If collision with existing code → append sequential digit
5. Store mapping in library config
```

Example: `.blend` → `BLEND`, `.fcpxml` → `FCPXML`, `.r3d` → `R3D`

### 2.3 Kind Registry Storage

Per-universe config at `.constellation/file_kinds.json`:

```json
{
  "version": 1,
  "custom_kinds": {
    "BLEND": { "extensions": [".blend"], "description": "Blender 3D file" },
    "R3D": { "extensions": [".r3d"], "description": "RED camera footage" }
  }
}
```

Core kinds are hardcoded in Rust — never stored in config. Custom kinds extend the registry at runtime.

---

## 3. Classification Engine

### 3.1 Architecture

Three-layer classifier, executed in order. First match wins.

```
classify(file_path) → KindCode
    │
    ├── Layer 1: Extension → Binary kind (IMG, AUD, VID, ATT, CANVAS, DRAW)
    │   Fast lookup table. No file content read required.
    │
    ├── Layer 2: Markdown content analysis → Text kind (NOTE, BASE, TMPL, LINK, MARK, CLIP)
    │   Reads first 4KB of file. Parses YAML frontmatter + structural signals.
    │
    └── Layer 3: Unknown extension → Auto-generate code
        Creates new entry in kind registry.
```

### 3.2 Layer 1 — Extension Mapping

Static `HashMap<&str, &str>` compiled into the binary:

```rust
const EXTENSION_MAP: &[(&str, &str)] = &[
    // Images
    ("png", "IMG"), ("jpg", "IMG"), ("jpeg", "IMG"), ("gif", "IMG"),
    ("svg", "IMG"), ("webp", "IMG"), ("bmp", "IMG"), ("ico", "IMG"), ("tiff", "IMG"),
    // Audio
    ("mp3", "AUD"), ("wav", "AUD"), ("ogg", "AUD"), ("m4a", "AUD"),
    ("flac", "AUD"), ("aac", "AUD"), ("wma", "AUD"),
    // Video
    ("mp4", "VID"), ("webm", "VID"), ("mov", "VID"),
    ("mkv", "VID"), ("avi", "VID"), ("wmv", "VID"),
    // Documents
    ("pdf", "ATT"), ("docx", "ATT"), ("xlsx", "ATT"), ("pptx", "ATT"),
    ("epub", "ATT"), ("zip", "ATT"), ("rar", "ATT"), ("7z", "ATT"),
    // Canvas / Drawing
    ("canvas", "CANVAS"), ("excalidraw", "DRAW"),
];
```

If extension matches here → return immediately. No content read.

### 3.3 Layer 2 — Markdown Content Heuristics

For `.md` and `.markdown` files, read the first 4KB and classify:

```
Priority order (first match wins):

1. EXPLICIT TYPE — frontmatter has `type:` or `kind:` field
   → Map directly: "template" → TMPL, "link" → LINK, "base" → BASE, etc.

2. LINK — frontmatter has `from:` AND `to:` fields
   → This is a typed relationship between two knowledge objects.

3. TMPL — content contains template syntax
   → Detect: `<% ... %>` (Templater), `{{ ... }}` (Handlebars/Mustache),
     `{< ... >}` (Hugo), or frontmatter `template: true`

4. MARK — frontmatter has `url:` or `bookmark:` field, body is short (< 500 chars)
   → Saved web bookmark with metadata.

5. CLIP — frontmatter has `source:` field AND body contains blockquotes (> ...)
   → Web clipping with preserved source attribution.

6. BASE — frontmatter has `schema:` or `fields:` or `database:` field,
   OR body contains Dataview/DataviewJS blocks
   → Structured data / database note.

7. CANVAS — body is valid JSON with `nodes` and `edges` arrays
   → Visual map (some systems use .md extension for canvas data).

8. NOTE — default
   → Standard knowledge note. Anything that doesn't match above.
```

### 3.4 Layer 3 — Auto-Generation

```rust
fn auto_generate_kind(extension: &str, registry: &mut KindRegistry) -> String {
    let code = extension.to_uppercase();
    let code = if code.len() > 6 { &code[..6] } else { &code };
    let code = code.to_string();

    // Check collision with existing codes
    if registry.has_code(&code) {
        // Append digit: BLEND → BLEND1
        for i in 1..=9 {
            let candidate = format!("{}{}", code, i);
            if !registry.has_code(&candidate) {
                registry.register(candidate.clone(), extension);
                return candidate;
            }
        }
    }

    registry.register(code.clone(), extension);
    code
}
```

---

## 4. Canonical Filename Generator

### 4.1 Timestamp Source

| Scenario | Timestamp source |
|----------|-----------------|
| New note created in Constellation | `Utc::now()` |
| Imported file | File's **creation date** from filesystem metadata |
| Fallback (no creation date available) | File's **modification date** |
| Last resort | Import time |

### 4.2 Hex Suffix Generation

4-character uppercase hex (`0000`–`FFFF`), generated from:

```rust
fn generate_suffix() -> String {
    use rand::Rng;
    let n: u16 = rand::thread_rng().gen();
    format!("{:04X}", n)
}
```

### 4.3 Collision Check

Before finalizing a canonical filename, verify it doesn't exist in the target directory. If collision:

```
Attempt 1: 20260410T153045Z_NOTE_7F3A.md
Collision → regenerate suffix
Attempt 2: 20260410T153045Z_NOTE_B2C1.md
```

Maximum 10 attempts, then increment the timestamp by 1 second.

### 4.4 Format Function

```rust
fn canonical_filename(timestamp: DateTime<Utc>, kind: &str, suffix: &str, ext: &str) -> String {
    format!(
        "{}_{}_{}{}",
        timestamp.format("%Y%m%dT%H%M%SZ"),
        kind,
        suffix,
        if ext.starts_with('.') { ext.to_string() } else { format!(".{}", ext) }
    )
}
// → "20260410T153045Z_NOTE_7F3A.md"
```

---

## 5. Frontmatter Contract

### 5.1 For Markdown Files (NOTE, BASE, TMPL, LINK, MARK, CLIP)

Every `.md` file under Constellation management receives this frontmatter:

```yaml
---
title: Agriculture System
cid: 20260410T153045Z_NOTE_7F3A
kind: note
created: 2026-04-10T15:30:45Z
aliases:
  - Agriculture System
---
```

| Field | Required | Mutable | Description |
|-------|----------|---------|-------------|
| `title` | Yes | Yes | Human-facing display name. User can change freely. |
| `cid` | Yes | **No** | Constellation ID = canonical filename stem. Immutable PK. |
| `kind` | Yes | **No** | File kind (lowercase). Immutable — what the file IS. |
| `created` | Yes | **No** | ISO 8601 UTC creation timestamp. |
| `aliases` | No | Yes | Alternative names for wikilink resolution. |

### 5.2 Additional Fields per Kind

**LINK:**
```yaml
---
title: Agriculture → Irrigation Systems
cid: 20260410T153045Z_LINK_A1B2
kind: link
created: 2026-04-10T15:30:45Z
from: 20260410T140000Z_NOTE_7F3A
to: 20260312T091500Z_NOTE_C3D4
link_type: relates_to
---
```

**MARK (Bookmark):**
```yaml
---
title: Regenerative Agriculture Guide
cid: 20260410T153045Z_MARK_D4E5
kind: bookmark
created: 2026-04-10T15:30:45Z
url: https://example.com/regen-ag
saved: 2026-04-10T15:30:45Z
---
```

**CLIP:**
```yaml
---
title: Key findings on soil health
cid: 20260410T153045Z_CLIP_F6G7
kind: clip
created: 2026-04-10T15:30:45Z
source: https://example.com/soil-research
author: Dr. Smith
clipped: 2026-04-10T15:30:45Z
---
```

### 5.3 For Non-Markdown Files (IMG, AUD, VID, ATT)

No frontmatter possible. Use a **sidecar `.meta.json`** file:

```
20250315T120000Z_IMG_E5F6.png
20250315T120000Z_IMG_E5F6.meta.json
```

```json
{
  "title": "Farm diagram",
  "cid": "20250315T120000Z_IMG_E5F6",
  "kind": "img",
  "created": "2025-03-15T12:00:00Z",
  "original_filename": "Pasted image 20250315.png",
  "aliases": ["Farm diagram", "Pasted image 20250315"],
  "referenced_by": ["20260410T153045Z_NOTE_7F3A"]
}
```

The `.meta.json` suffix (not just `.json`) avoids conflicts with actual JSON data files.

---

## 6. Wikilink Resolution

### 6.1 The Principle

**Wikilinks remain human-readable. Always.**

```markdown
See [[Agriculture System]] for details.
```

NOT:

```markdown
See [[20260410T153045Z_NOTE_7F3A]] for details.
```

### 6.2 Resolution Order

When Constellation encounters `[[Agriculture System]]`:

```
1. Search index: title == "Agriculture System"         → exact match
2. Search index: aliases CONTAINS "Agriculture System"  → alias match
3. Search index: original_filename == "Agriculture System.md" → legacy match
4. Not found → mark as broken link (red in editor)
```

### 6.3 Rename Safety

When a user renames a note from "Agriculture System" to "Farming Systems":

```
1. Update frontmatter: title → "Farming Systems"
2. Add old title to aliases: ["Agriculture System"]
3. Canonical filename does NOT change
4. All existing [[Agriculture System]] links still resolve via aliases
5. Optionally: scan-and-replace [[Agriculture System]] → [[Farming Systems]] across library
```

**Zero broken links on rename.** The alias list is the safety net. The canonical filename never moves.

### 6.4 Title Collision

Two notes with the same title in the same library:

```
20260410T153045Z_NOTE_7F3A.md  →  title: "Meeting Notes"
20260412T090000Z_NOTE_B2C1.md  →  title: "Meeting Notes"
```

Resolution: if `[[Meeting Notes]]` is ambiguous, Constellation shows a disambiguation popup (like Wikipedia). The user picks which one. Alternatively, use path-qualified links: `[[Projects/Meeting Notes]]`.

---

## 7. Import Pipeline

### 7.1 Supported Sources

| Source | Detection | Special handling |
|--------|-----------|-----------------|
| Obsidian vault | `.obsidian/` folder present | Strip `.obsidian/`, `.trash/`, plugin data |
| Plain markdown folder | Any folder with `.md` files | Direct classification |
| Notion export | UUID-suffixed filenames | Strip UUIDs from names, fix broken links |
| Evernote (.enex) | XML with `<note>` tags | Convert ENML to markdown |
| Bear export | `.md` files with `{BearID:...}` | Strip Bear IDs |
| Joplin export | `.md` files with hex-named resources | Map resource IDs to filenames |
| HTML files | `.html` / `.htm` | Convert to markdown |
| CSV | `.csv` / `.tsv` | One row = one note |
| Plain text | `.txt` | Wrap in markdown |

### 7.2 Pipeline Phases

```
Phase 1: SCAN
  ├── Walk source directory recursively
  ├── Skip excluded dirs (.obsidian, .git, .trash, node_modules, __MACOSX)
  ├── Collect every file with path + metadata (size, created, modified)
  └── Output: Vec<SourceFile>

Phase 2: CLASSIFY
  ├── For each file, run Classification Engine (Layer 1 → 2 → 3)
  ├── For Notion: strip UUID suffix to recover original title
  ├── For Bear: strip {BearID:...} to recover clean content
  ├── For Evernote: convert ENML to markdown
  └── Output: Vec<ClassifiedFile { source, kind, title, content }>

Phase 3: GENERATE
  ├── For each classified file:
  │   ├── Extract creation timestamp (filesystem metadata)
  │   ├── Generate canonical filename: YYYYMMDDTHHMMSSZ_KIND_XXXX.ext
  │   └── Check for collisions, regenerate suffix if needed
  ├── Build rename map: HashMap<original_path, canonical_path>
  └── Output: RenameMap + Vec<CanonicalFile>

Phase 4: ENRICH
  ├── For .md files:
  │   ├── Inject/update frontmatter (title, cid, kind, created, aliases)
  │   ├── Preserve existing frontmatter fields (tags, custom metadata)
  │   └── Add original_filename to aliases for backward compatibility
  ├── For media files:
  │   └── Create .meta.json sidecar
  └── Output: enriched file contents ready to write

Phase 5: WRITE
  ├── Write all enriched files to destination with canonical filenames
  ├── Create folder structure (preserve source folders)
  ├── Write sidecar .meta.json files
  └── This is a COPY operation — source is never modified

Phase 6: INDEX
  ├── Build title index: cid → title, aliases
  ├── Build link index: scan all wikilinks, resolve via title index
  ├── Report broken links (unresolvable [[references]])
  └── Trigger full library reindex (search, embeddings)
```

### 7.3 Preview Before Execute

Before Phase 5, the user sees a preview:

```
Import Preview: Obsidian Vault "My Research"
─────────────────────────────────────────────
Files to import: 1,247
  Notes:       892   (.md)
  Images:      234   (.png, .jpg)
  Attachments:  89   (.pdf, .docx)
  Templates:    12   (.md with template syntax)
  Bookmarks:    15   (.md with url: field)
  Clips:         5   (.md with source: field)

Title collisions: 3
  "Meeting Notes" (2 files) → will add folder path to aliases
  "Index" (2 files) → will add folder path to aliases

Broken wikilinks detected: 7
  [[Missing Note]] referenced by 3 files
  ...

[Cancel]  [Import]
```

### 7.4 Atomicity

The import is **atomic at the library level**:
- Write to a temporary `.importing/` directory first
- On success, move everything to the final location
- On failure, delete `.importing/` — source untouched

---

## 8. New Note Creation Flow

When a user creates a new note in Constellation:

```
1. User clicks "New Note" or presses shortcut
2. Constellation generates:
   - timestamp: Utc::now()
   - kind: NOTE (default)
   - suffix: random 4-char hex
   - canonical: 20260410T153045Z_NOTE_7F3A.md
3. File is created on disk with frontmatter:
   ---
   title: Untitled
   cid: 20260410T153045Z_NOTE_7F3A
   kind: note
   created: 2026-04-10T15:30:45Z
   ---
4. User types a title → frontmatter `title:` updates
5. Canonical filename never changes
```

### Creating Other Kinds

```
New Base      → ...T...Z_BASE_XXXX.md      (kind: base)
New Template  → ...T...Z_TMPL_XXXX.md      (kind: template)
New Bookmark  → ...T...Z_MARK_XXXX.md      (kind: bookmark)
New Link      → created automatically when user creates a typed connection
New Canvas    → ...T...Z_CANVAS_XXXX.json   (kind: canvas)
```

---

## 9. Existing Library Migration

For libraries already in Constellation (with human filenames):

```
Phase 1: User opts into "Canonicalize Library" (settings or command)
Phase 2: Classification runs on all existing files
Phase 3: Preview shows all proposed renames
Phase 4: User confirms
Phase 5: For each file:
  a. Inject cid + kind into frontmatter (if .md)
  b. Create .meta.json sidecar (if media)
  c. Add current filename to aliases
  d. Rename to canonical filename
Phase 6: Reindex
```

This is **opt-in, never automatic**. Users who prefer human filenames can keep them — Constellation works either way. The `cid` in frontmatter still provides a stable ID even without renaming.

---

## 10. Rust Module Structure

### New files:

```
src-tauri/src/
  file_kinds.rs      ← Kind registry, classification engine
  canonical.rs       ← Canonical filename generation, frontmatter injection
```

### Modified files:

```
src-tauri/src/
  importers.rs       ← Integrate classification + canonical naming into import pipeline
  libraries.rs       ← Wikilink resolution via title/alias index
  search.rs          ← Index cid, title, aliases, kind fields
  lib.rs             ← Register new modules
```

### New Tauri commands:

```rust
#[tauri::command]
fn classify_file(path: String) -> Result<String, String>  // → kind code

#[tauri::command]
fn generate_canonical_name(kind: String, created: Option<String>) -> Result<String, String>

#[tauri::command]
fn canonicalize_library(app: AppHandle, library_path: String) -> Result<MigrationResult, String>

#[tauri::command]
fn import_with_canonical(app: AppHandle, source: String, format: String, target: String) -> Result<ImportResult, String>
```

---

## 11. Design Principles

1. **The canonical filename is the primary key.** It never changes after creation.
2. **The title is the human interface.** Users name, rename, and search by title.
3. **Wikilinks use titles, not filenames.** `[[My Note]]` resolves via the title index.
4. **Aliases are the safety net.** Old titles are preserved as aliases — zero broken links.
5. **Classification is deterministic.** Same file → same kind, every time.
6. **Unknown types are welcomed.** New extensions auto-generate codes — no gatekeeping.
7. **Migration is opt-in.** Existing libraries work as-is. Canonicalization is a choice.
8. **The file is self-contained.** Open any `.md` in Notepad — the title, ID, and kind are right there in frontmatter.
9. **Sidecar metadata keeps media self-describing.** The `.meta.json` file travels with the asset.
10. **Import never modifies the source.** Always copy, never move or rename originals.
