# Index

The Index is Constellation's living vocabulary catalog — every meaningful word across every library you've added, sorted alphabetically with occurrence counts. Think of it as the index at the back of a book, except it's live and covers your entire knowledge system at once.

## Opening

- **Dock button** — click the Index icon (book) in the left dock.
- **Command Palette** — `Ctrl+P` (or `Cmd+P` on Mac) → type "Index".

## What you see

When you open the Index, three things appear:

1. **A search filter at the top** — narrows the term list as you type.
2. **A term list** — every word in your library, alphabetically sorted, with the occurrence count next to each entry.
3. **An "Also appears with" chip strip** — shows up below an expanded term, listing related vocabulary that co-occurs in the same notes.

## Browsing

- **Language tabs**: switch between **All**, **Arabic**, **Hebrew**, **English**, or **#** (numbers / special characters). Each tab narrows the term list to the corresponding script.
- **Alphabet bar**: click a letter to filter to terms starting with it. The term count above updates to reflect the filtered set. Click the same letter again to clear.
- **Sort modes**: alphabetical (default) or by frequency (most common first).

## Expanding a term

Click the **▸** triangle next to any term to expand its **mentions** — the list of notes that contain that term. Each mention shows:

- The note name (clickable — opens the note).
- A short context snippet around where the term appears.
- The term itself highlighted in the snippet.

Below the mentions list, a chip strip labeled **"Also appears with"** shows the most-common co-occurring terms — words that frequently appear in the same notes as the one you expanded. Click a chip to compare term sets.

## Cross-language Mentions

Constellation's **Lexical Bridge** knows that "knowledge" in English, "معرفة" / "علم" in Arabic, "connaissance" in French, "知识" in Chinese, and so on across all 15 supported languages — refer to the same concept. The bridge ships with the app as a 20,000-concept multilingual dictionary.

By default, the Index treats each term **literally** — clicking "knowledge" shows only notes that contain those exact letters. If you want the Index to surface notes about the **same concept in other languages**, turn on the toggle:

1. Open **Settings → Index**.
2. Flip on **Expand mentions cross-language**.
3. Return to the Index. Click any term again — the mentions list now includes notes in any language that mention an equivalent concept.

Cross-language matches carry a small **"via {lemma}"** badge next to the note name. The badge tells you which translation caused the row to surface:

- Click "tree" → an Arabic-titled note appears with **"via شجرة"** because that note contains "شجرة" (Arabic for "tree").
- Click "معرفة" → an English-titled note appears with **"via knowledge"** because that note contains the English equivalent.

Direct same-language matches still appear with no badge. The toggle is **off by default** so the Index works the same as before for users who want their vocabulary strictly per-language.

**Note**: today the toggle affects mentions only — the **filter box** at the top of the Index still does plain substring matching, so typing "knowledge" finds only index terms containing those letters. Cross-language filtering (typing "knowledge" also finding "معرفة" in the term list) is a planned next step.

## Editing from the Index

Click any note in a term's mentions to open it as a split preview pane alongside the Index. The preview is a full editor — you can edit, save, change properties, promote stages. The clicked term is highlighted in the note and scrolled to automatically.

`Ctrl+Click` (or `Cmd+Click`) opens the note as a regular tab instead. A **"Return to Index"** button appears in the tab bar — click it to jump back to exactly where you left off in the Index.

## Second Screen

When the Second Screen window is open:

- **Click a term** in the Index → Second Screen displays all notes containing that term in a split view (note list on one side, editor on the other).
- **Ctrl+Click multiple terms** → Second Screen shows compare mode, with each term's notes in its own column side-by-side.

## How it stays current

The Index reads directly from Constellation's full-text search index, which is maintained in real time as you edit notes. There's nothing to "rebuild" — the moment you save a note, its terms are reflected in the Index. Boot is free; the Index opens to a live view.

## Multilingual NLP

Behind the Index, every word is tokenized through a language-aware pipeline:

- **Arabic**: Constellation Arabic Engine (CAE) — root-pattern morphology, definite-article peeling, hamza unification, broken-plural recognition.
- **Hebrew**: Prefix removal (ב/ל/מ/ה/ו/כ/ש).
- **English**: Porter-style stemming.
- **French / Spanish / Portuguese / German / Italian**: Suffix-based stemming.
- **Russian / Turkish / Hindi / Persian / Urdu**: Morphological suffix removal.
- **Stop words**: filtered from all 15 languages so the Index focuses on meaningful vocabulary.

This means "knowledge" and "knowledges" merge to one entry; "معرفة" and "المعرفة" merge to one entry. You see concepts, not surface forms.
