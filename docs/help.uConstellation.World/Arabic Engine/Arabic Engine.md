# Arabic Engine

Constellation analyses Arabic text with a five-layer morphological engine built from the ground up for this app. It is not a port of an existing stemmer — it is a native instrument that understands Arabic roots, patterns, proper nouns, loanwords, and your own terminology. You never configure the engine itself; it runs silently beneath every search, every link, every index entry. What you *can* configure — and what this help topic covers — is the single place the engine invites your judgement: the **Arabic Engine Overrides** panel in Settings.

---

## Why the engine exists

Arabic is a templatic language. A single root like ك‑ت‑ب ("to write") generates dozens of surface forms — كاتب (writer), مكتوب (written), كتاب (book), يكتب (he writes), كتبنا (we wrote) — all of which should collapse to the same semantic core when you search. A naïve stemmer either mangles these forms (over-stripping وائل into ائل, for example) or misses the connection between them entirely. Constellation's engine avoids both failures by running every Arabic word through five layers in strict priority order:

1. **Layer 0 — User Overrides** (this is the one you control)
2. **Layer 2 — Protected List** (~1,200 hand-curated proper nouns, places, loanwords, and function words that must never be touched)
3. **Layer 3 — Generative FST** (a compiled finite-state transducer that maps ~7,000 roots × 158 patterns to their full surface vocabulary)
4. **Layer 3b — Cascade** (phonological repairs: assimilation, weak roots, hamza placement)
5. **Layer 5 — Heuristic** (the graceful fallback — a conservative affix stripper that only fires when every other layer declined to answer)

A ranking step (Layer 4) picks the single best analysis when more than one layer produces a reading. The ranking puts your overrides above everything else.

---

## Feature: Arabic Engine Overrides

### What it is

The Overrides panel is a small table in Settings where you tell the engine, in your own words, how to analyse specific Arabic surfaces. Each override has:

- **Surface** — the Arabic word exactly as you type it (e.g. وائل).
- **Lemma** — the canonical form the engine should return (e.g. وائل).
- **Root** — optional. Three or four consonants if the word has a classical root.
- **Pattern** — optional. A free-text label (e.g. `فاعل`) if you want to record the morphological template.
- **Part of speech** — Proper noun / Noun / Adjective / Adverb / Verb / Particle / Foreign / Unknown.
- **Note** — optional. A line of context for your future self.

### Why it matters

Every knowledge network has terms the engine cannot know from a dictionary: your own coinages, names from your local town, acronyms you use in your field, loanwords your colleagues prefer spelled a specific way. Without overrides, the engine would apply its generic analysis to these surfaces and your search results would fragment around slight variations. An override is the sovereign answer — it wins over the generative FST, the cascade, and the heuristic fallback. Layer 4's ranking gives overrides the top origin and a confidence of 1.0, so they are never discarded in favour of another analysis.

Overrides live in a single JSON file at `<your Universe>/.constellation/arabic-overrides.json`. The file is plain text, sorted alphabetically, and written atomically (via a `.tmp` + rename pair) so a power loss mid-edit cannot corrupt it. It is yours — you can version-control it, diff it, or share it across devices.

### How to use it

**Step 1: Open the panel**

Click the gear icon in the top-right toolbar (or press `Ctrl + ,` / `Cmd + ,`) to open Settings. In the left sidebar, select **Arabic Overrides** — it sits next to **Language**. If you do not see it, scroll the sidebar.

**Step 2: Add your first override**

Click **Add override**. A form appears with six fields (surface, lemma, root, pattern, part of speech, note). Type the surface exactly as you write it in your notes — the engine normalises diacritics and alef variants internally, so you do not need to worry about matching them precisely. Fill in the lemma you want returned. Leave the root and pattern blank if you do not know them; the engine will still use the override. Choose a part of speech from the dropdown, or leave it at **Unknown**. Click **Save**.

**Step 3: Watch the reindex banner**

The moment you save, the panel shows **Reindexing…** and the engine sweeps every note in the active Universe whose text contains that surface. Each matching note is re-tokenised under the new override verdict. When the sweep finishes — usually within a second on a typical Universe — the banner turns to **Reindexed N note(s)** and auto-clears after three seconds. You do not need to restart the app and you do not need to rebuild any index.

**Step 4: Verify in search**

Open the Search hub (`Ctrl + K` / `Cmd + K`) and type the surface. The matches should now reflect the lemma you specified: queries for the lemma find the surface, and queries for the surface find other inflections of the lemma.

**Step 5: Remove an override**

Click the **×** button on the override's row. The entry is removed from disk immediately, and the same reindex sweep runs in reverse — the notes that contained the surface are re-tokenised under the engine's generic analysis. The banner reports how many notes were touched.

### Interaction with the Protected List

The Protected List (Layer 2) already contains ~1,200 common surfaces that must never be stripped — names like وائل, places like فلسطين, loanwords like إنترنت. You do not need to add these yourself; the engine ships with them. Use the Overrides panel for surfaces that are *personal* to your Universe — your own terminology, local names, field-specific loanwords, or cases where you disagree with the engine's automatic reading.

### Interaction across Universes

Each Universe has its own overrides file. Switching Universes swaps the active override set in memory — the engine reloads the JSON from the new Universe's `.constellation/` folder. If the file is missing (a fresh Universe), the engine treats the override set as empty. If the file is malformed, the engine logs a warning and falls back to an empty set rather than refusing to load.

### What happens if you edit the file by hand

You can. The file format is:

```json
[
  {
    "surface": "وائل",
    "lemma": "وائل",
    "root": null,
    "pattern": null,
    "pos": "ProperNoun",
    "note": "Personal name — never strip"
  }
]
```

Keep entries sorted alphabetically by surface for git-friendly diffs. The engine re-sorts on every save, so manual reorderings will not survive a UI-driven edit.

---

## Glossary

- **Surface** — an Arabic word as written, including any attached clitics (e.g. الكتاب, بالكتاب, كتبنا).
- **Lemma** — the citation form of a word, stripped of inflection (e.g. كتاب).
- **Root** — the 3- or 4-consonant semantic core shared by a family of words (e.g. ك‑ت‑ب).
- **Pattern** — the vowel-and-affix template that combines with a root to produce a surface (e.g. فاعل → كاتب).
- **FST** — a finite-state transducer. The engine uses one to map roots × patterns to their full surface vocabulary efficiently.
- **Cascade** — the phonological repair layer that handles assimilation, weak consonants, and hamza placement.
- **Override** — your own verdict for how a specific surface should be analysed; wins over every other layer.
