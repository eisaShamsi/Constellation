---
title: Constellation Knowledge Hierarchy
aliases: [Knowledge Hierarchy, Directory System, CKH]
description: The five-level organizational structure that makes Constellation the most comprehensive PKM system.
---

# Constellation Knowledge Hierarchy

Constellation organizes knowledge in a five-level hierarchy. No other PKM system provides this depth of organization.

## The Five Levels

```
Universe
  └── Child Universe (cUniverse)
       └── Library
            └── Folder
                 └── Note
```

## Universe

The **Universe** is the top-level container — the root of everything in Constellation. Each Constellation instance has one active Universe. The Universe holds:

- All registered Libraries
- Child Universes (federated knowledge domains)
- Settings, Bases, Bookmarks, and Search Index
- Universe-level metadata (`universe.json`)

The Universe is stored as a directory on your local filesystem. It is portable and can be moved, backed up, or synced across devices.

## Child Universe (cUniverse)

A **Child Universe** is a linked Universe that contributes its libraries to a parent Universe. This enables:

- **Federation**: View notes from multiple independent Universes in one window
- **Domain separation**: Keep work, personal, and project knowledge in separate Universes
- **Collaboration**: Share a cUniverse with others while keeping your parent Universe private

Child Universes appear grouped in the sidebar, with their libraries nested inside.

## Library

A **Library** is the fundamental unit of knowledge in Constellation — equivalent to an Obsidian vault, a Notion workspace, or a project repository. A Library is:

- A **complete, self-contained knowledge base** on disk (a folder of Markdown files)
- Identified by its own **color**, **appearance settings**, **font preferences**
- Indexed independently for **tags**, **links**, **search**, and **graph visualization**
- Registered in the Universe's `libraries.json` — never copied, always read in place

### Library vs. Folder

| Aspect | Library | Folder |
|--------|---------|--------|
| **Identity** | Has its own name, color, settings | Just a directory name |
| **Scope** | Complete knowledge base | Subset within a Library |
| **Index** | Own tag namespace, link graph, search index | Inherits from parent Library |
| **Creation** | Created via "New Library" button or linked from disk | Created inside an existing Library |
| **Portability** | Can be moved, shared, synced independently | Moves with its Library |

A Library is a first-class citizen. A Folder is organizational structure within a Library.

### Creating a Library

1. Click the **New Library** button (book icon with plus) in the sidebar toolbar
2. Choose one of two options:
   - **Create New Library**: Enter a name, Constellation creates a new folder and registers it
   - **Link Existing Library**: Browse for an existing Markdown folder on disk to register

Libraries can also be added via the Command Palette (`New library`) or from Settings > Universe & Libraries.

## Folder

A **Folder** is a subdirectory within a Library. Folders provide:

- Organizational structure for notes
- Nesting (folders inside folders)
- Drag-and-drop management in the sidebar file tree

Folders have no identity beyond their name and path. They are purely organizational.

## Note

A **Note** is the atomic unit of knowledge — a single Markdown file (`.md`) with optional YAML frontmatter. Notes contain:

- **Content**: Markdown text, headings, lists, callouts, code blocks
- **Properties**: YAML frontmatter (key-value metadata)
- **Links**: Wikilinks `[[to other notes]]` with optional typed relationships
- **Tags**: Inline `#hashtags` or YAML `tags:` arrays
- **Attachments**: Images, PDFs, and other files referenced from the note

## Why This Hierarchy Matters

The Constellation Knowledge Hierarchy enables:

1. **Scale**: Manage thousands of notes across multiple domains without chaos
2. **Independence**: Each Library is self-contained — sync, share, or archive independently
3. **Federation**: Child Universes let you combine knowledge from different sources
4. **Discovery**: The Search Hub, Sky View, and Constellation Sight operate across the entire Universe — finding connections between Libraries that no single-vault system can see
5. **Multilingual**: Each Library can have its own language, font, and directionality preferences
