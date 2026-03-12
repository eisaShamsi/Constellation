---
aliases:
  - Glossary
  - Word Index
  - Concordance
  - Term index
description: An automatic word index that extracts every word from all notes across all vaults, showing occurrence counts and source notes.
---

# Glossary

The Glossary is an automatic word index. It extracts every word from every note across all your vaults, counts how many times each word appears, and shows you which notes contain it. Click any note name to jump straight to it.

You can open the Glossary from the **book icon** in the left ribbon, or via the command palette (**Ctrl+P** > "Open glossary").

---

## How it works

The glossary scans all `.md` files in all registered vaults and builds a word index:

1. **Strip markdown syntax** — Removes links, bold/italic markers, code blocks, HTML tags, headings markers, and other formatting so only the actual text content is indexed.
2. **Extract every word** — Splits the cleaned text into individual words.
3. **Count occurrences** — Tracks how many times each word appears across all notes (total count) and which notes contain it (one entry per note, deduplicated).

### Word filtering

To keep the index useful, very short or purely numeric tokens are excluded:

- **Latin-script words**: must be 3 or more characters.
- **Arabic / CJK words**: must be 2 or more characters.
- **Pure numbers** (e.g. `42`, `2024`) are skipped.

---

## Using the glossary panel

### Opening the panel

- Click the **book icon** in the left ribbon.
- Or press **Ctrl+P** and type "Open glossary".

The glossary replaces the file explorer in the left sidebar. Click the folder icon to return to the file explorer.

### Filtering words

Use the search input at the top of the panel to filter words by name. The filter updates instantly as you type.

### Browsing words

Words are grouped alphabetically by their first letter with sticky letter headers (A, B, C...). Each word row shows:

| Element | Description |
|---------|-------------|
| Word | The indexed word (bold) |
| Count | Total number of occurrences across all notes |

### Viewing source notes

Click a word to expand its source list. Note names appear in a comma-separated list. Click any note name to open it in the editor.

---

## Auto-updating

The glossary updates automatically when:

- A vault is added or removed.
- Files are created, modified, or deleted in any watched vault.
- The app starts up.

Changes are detected by the file watcher with a short debounce delay (~1.5 seconds) to avoid excessive rescanning during rapid edits.

---

## Multi-vault merging

When you have multiple vaults, the glossary merges word data across all of them:

- The same word found in different vaults has its counts summed.
- Source notes are deduplicated — each note appears only once per word.
- All words are sorted alphabetically in a single unified index.

---

## Disabling the glossary

If you don't need the glossary, you can disable it:

1. Open **Settings** (Ctrl+,).
2. Navigate to **Core Plugins**.
3. Toggle **Glossary** off.

This hides the glossary ribbon button and stops word scanning.

---

## Tips

> [!tip] Arabic and multilingual support
> The glossary supports words in any language including Arabic, Hebrew, and other RTL scripts. Arabic words are grouped under the letter "ع" in the alphabetical index.

> [!tip] Use the filter for large vaults
> In vaults with many notes, the word index can be very large. Use the filter input to quickly find specific words you're looking for.
