---
title: Search
aliases: [Search Engine, Find Notes, Hybrid Search]
description: Constellation's hybrid multilingual search engine with FTS5, structured queries, accessible badges, pinned results, and search history.
---

# Search

Constellation's search engine is a hybrid multilingual system built on SQLite FTS5 with BM25 ranking, structured query filters, and Arabic-optimized text normalization. It powers search across the sidebar, Sky View, OrgChart, Constellation Sight, and Quick Switcher.

## Activating Search

- Click the **search icon** in the sidebar toolbar
- Press **Ctrl+Shift+F** (Windows/Linux) or **Cmd+Shift+F** (macOS)
- Results appear after a 300ms debounce as you type

## Search Syntax

| Syntax | Example | What it finds |
|--------|---------|---------------|
| Free text | `project management` | Notes containing those words in title or body |
| Tag filter | `#research` | Notes tagged with `#research` |
| Property filter | `status=active` | Notes where frontmatter property `status` equals `active` |
| Wikilink filter | `links to [[Climate]]` | Notes that contain a link to `[[Climate]]` |
| Library scope | `in:MyLibrary` | Restricts results to a specific library |
| Combined | `#research status=active economy` | All filters applied together |

Filters can be combined freely. Free text is processed with FTS5 BM25 ranking for relevance scoring.

## Match-Type Badges

Each search result displays a colored character badge showing how the match was found. The badge letter is localized to your interface language for accessibility (color-blind safe).

| Badge | Color | Meaning |
|-------|-------|---------|
| **T** (en) / **ع** (ar) | Blue | **Title match** -- search term appears in the note's name |
| **C** (en) / **م** (ar) | Green | **Content match** -- search term appears in the note's body text |
| **S** (en) / **د** (ar) | Purple | **Semantic match** -- conceptually related (requires embedding model) |
| **P** (en) / **خ** (ar) | Amber | **Property match** -- matched via frontmatter property filter |
| **#** | Pink | **Tag match** -- matched via tag filter |
| **W** (en) / **ر** (ar) | Light blue | **Wikilink match** -- matched via wikilink filter |

Badges use both color and shape+letter to ensure accessibility for color-blind users.

## Pinned Results

Search results remain visible after you click one. The currently open note is highlighted in the result list. Click another result to navigate to it without re-searching. This lets you browse through multiple results from a single query.

To clear the search and return to the file tree, press **Escape** or click the **x** button.

## Keyboard Navigation

| Key | Action |
|-----|--------|
| Arrow Down | Select the next result in the list |
| Arrow Up | Select the previous result |
| Enter | Open the currently selected result |
| Escape | Clear search and return to the file tree |

## Search Term Highlighting

When you open a note from search results, all occurrences of your search term are highlighted in the editor. This includes Arabic-aware diacritic-insensitive matching -- searching for "ادارة" will highlight "إدارة" and all diacritical variants.

## Search History

Click on the search field when it is empty to see your recent searches (last 20 queries). Each entry shows the query text and how long ago it was used (e.g., "2m", "3h", "1d"). Click any entry to re-run that search instantly. Use the **Clear history** link at the bottom to erase all history.

Search history is stored locally on your device (in browser localStorage) and persists across app restarts.

## Arabic and Multilingual Support

The search engine includes Arabic Light10 stemming and normalization:
- Diacritics (tashkeel) are stripped for matching
- Alef variants (أ إ آ) are normalized to bare Alef (ا)
- Teh marbuta (ة) is normalized to Heh (ه)
- All languages are supported simultaneously with per-line bidirectional text
