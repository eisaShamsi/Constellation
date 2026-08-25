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

## Cross-language Filter — `≈ similar`

The Index filter at the top of the panel does **three layers** of matching as you type. Each layer adds a different kind of result:

1. **Literal substring** (always on). Typing `know` surfaces every term in your vocabulary containing those letters: `knowledge`, `known`, `knowing`, etc. This is the fastest layer and what you've always used.
2. **Cross-language bridge** — adds `via {lemma}` results when **Settings → Index → "Expand mentions cross-language"** is on. Typing `knowledge` ALSO surfaces Arabic terms whose dictionary translation is "knowledge" (`معرفة`, `علم`, …), each marked with the small **"via knowledge"** badge.
3. **Cross-language concept (`≈ similar`)** — always on, no setup. Typing `knowledge` ALSO surfaces terms whose **M11 concept** is the same — even when there's no direct dictionary translation lemma in your library. These rows carry the **`≈ similar`** badge.

How layer 3 works in plain terms: when you type `knowledge`, Constellation embeds that word once into a 384-dimension semantic space (one-time, ~50 milliseconds), looks up the ten nearest concepts in M11's 20,000-concept dictionary, expands each concept into all the languages it covers, and shows you which of *your* vocabulary terms map to those concepts. So if your library has Arabic notes that use `معرفة`, the stem `معرف` will appear in the dropdown with the `≈ similar` badge — even if you never turned the cross-language bridge toggle on.

The first time you type any query in a fresh session, expect a 2–5 second wait while the embedding model loads. Every query after that runs in ~80 milliseconds; the panel stays responsive while you type.

Misses are normal. The M11 dictionary covers 20,000 common-vocabulary concepts. Specialized jargon, proper nouns, and rare regional variants will often miss `≈ similar` — they'll still appear if they match the literal substring (layer 1) or the bridge (layer 2). Misses are not bugs.

## Previewing from the Index

Click any note in a term's mentions to open it as a split **preview pane** alongside the Index. The clicked term is highlighted in the note and scrolled to automatically.

The preview is a **read-only peek** — you can read the note (and follow its links, see below), but you can't type in it, change its properties, or rename it. This is deliberate: the preview is a quick look at where a term is used, *not* a second editing surface. Because it can never write, it can never overwrite the same note if you also have it open in a normal tab.

**To edit the note you're peeking at**, click the **"Open to edit"** button at the top of the preview. It opens the note as a normal, editable tab (and if that note is already open, it simply jumps you to the existing tab — never a duplicate). A **"Return to Index"** button appears in the tab bar so you can hop back to exactly where you left off.

- `Ctrl+Click` (or `Cmd+Click`) a note in the list opens it straight as a regular editable tab (same as "Open to edit"), leaving the Index with a "Return to Index" button.
- **Following links inside the preview:** click a `[[wikilink]]` in the peek and the preview *follows* it — it navigates to the linked note and you stay in the Index, so you can trace connections without leaving. `Ctrl+Click` (or `Cmd+Click`) a link instead to open that note as a real editable tab. (A link to a note that doesn't exist yet does nothing — a peek never creates notes.)

## Second Screen

When the Second Screen window is open:

- **Click a term** in the Index → Second Screen displays all notes containing that term in a split view (note list on one side, editor on the other).
- **Ctrl+Click multiple terms** → Second Screen shows compare mode, with each term's notes in its own column side-by-side.

## How it stays current

The Index reads directly from Constellation's full-text search index, which is maintained in real time **while Constellation is running** — the moment you save a note, its terms are reflected in the Index. Boot is free; the Index opens to a live view.

The one interval nothing can watch is the time the app is **closed**. A note edited by another device, a sync tool, or `git pull` while Constellation was shut down is not in the index until it is read again. Constellation checks for exactly that just after it opens, and if it finds anything it says so in a band across the top of the window, with a **Repair now** button. The same repair is always available from **Settings → Index → Index repair**. It re-reads what changed, indexes anything never seen before, and rebuilds the derived views — without ever writing to your note files. See the User Manual, "If your notes changed while Constellation was closed".

That same band may also report a different kind of leftover: entries that point at notes which no longer exist on disk, belong to no library of this universe, and carry none of your work — the reason a search result can occasionally open nothing. That sentence deliberately carries **no Repair now button**, because the repair reaches notes by walking your libraries and re-reading their files, and one of these leftovers has neither. Removing them is a separate, explicit action: **Settings → Index → Remove stale index entries**, which shows the count, asks you to confirm, and writes a record of each entry into your universe folder before it goes — readable afterwards in **Settings → Universe & Libraries → Deleted notes**. That record is a log, not an undo, so treat the removal itself as permanent — and it keeps the note's text **as the search index held it**, which is a stripped version rather than your file: properties, headings, code-block contents, link addresses and Arabic vowel marks are all absent from it. The links a removed entry carried are not recorded at all. Your note files are never touched. The count is conservative — if a drive holding a library is unavailable it reports nothing rather than risk mistaking your real notes for leftovers — and zero is a normal, healthy answer.

## Multilingual NLP

Behind the Index, every word is tokenized through a language-aware pipeline:

- **Arabic**: Constellation Arabic Engine (CAE) — root-pattern morphology, definite-article peeling, hamza unification, broken-plural recognition.
- **Hebrew**: Prefix removal (ב/ל/מ/ה/ו/כ/ש).
- **English**: Porter-style stemming.
- **French / Spanish / Portuguese / German / Italian**: Suffix-based stemming.
- **Russian / Turkish / Hindi / Persian / Urdu**: Morphological suffix removal.
- **Stop words**: filtered from all 15 languages so the Index focuses on meaningful vocabulary.

This means "knowledge" and "knowledges" merge to one entry; "معرفة" and "المعرفة" merge to one entry. You see concepts, not surface forms.
