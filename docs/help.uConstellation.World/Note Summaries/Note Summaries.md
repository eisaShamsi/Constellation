---
aliases:
  - Note Summaries
  - Note Summary
  - Summary
  - NSC
  - Note Summary Creator
  - Build all summaries
description: Note Summaries give you a short, plain-language précis of a note so you can judge it without opening it. Constellation always honors a summary you wrote yourself — in frontmatter or a summary callout — and only generates one when you haven't. Generated summaries are extractive (the note's own most-central sentences), read-only (never written back into your file), and computed entirely on your device. Summaries appear across the app wherever a note shows up — the Cataloger, Search results, the Editor band, the Backlinks panel, the Outgoing links panel, the Index, and the Sky View hover.
---

# Note Summaries

> *If you wrote a summary, Constellation uses yours. It only writes one when you haven't — and even then, never into your file.*

A **Note Summary** is a short précis of a note — a few sentences that tell you what the note is about at a glance. Summaries are produced by the **Note Summary Creator (NSC)**. You'll see them **across the whole app, wherever a note shows up**: under each card's title in the **Cataloger** / **Source Review** queue (where you decide how to classify a card without opening the note behind it), as a faint italic line under each hit in **Search results** (telling you what a note is *about* alongside the snippet that shows why it matched), as a thin band above the note when you open it in the **Editor** (so the note's gist is in-context as you read or write), under each linked source in the **Backlinks** panel and each linked target in the **Outgoing links** panel (so a long list of related notes scans as ideas rather than just titles), under each note-mention when you expand a term in the **Index** (so a term's notes are recognizable at a glance), and inside the **Sky View** hover tooltip when you point at a bubble (so a busy graph stays readable without clicking through).

This topic explains where summaries come from, the strict order of precedence that always prefers *your* words over the machine's, how the generated summaries are built, and how to pre-compute them for a whole Library at once.

---

## Why summaries exist

When you are working through a review queue of hundreds of cards, the title alone often isn't enough to remember what a note actually says. Opening each note to refresh your memory breaks your flow. A one-glance summary under the title fixes that: you read three sentences, you remember the note, you make the call, you move on.

But a summary is also a small act of authorship. If you have already distilled a note in your own words — in a `summary:` field or a `> [!summary]` callout — then *that* is the summary that should show, not a machine's guess. Constellation's first rule for summaries is therefore a rule about respect for your writing: **yours wins.**

---

## Where a summary comes from — the order of precedence

For any note, Constellation picks the summary by walking down this list and stopping at the first one that exists:

1. **Your frontmatter summary.** If the note's properties contain a `summary:`, `description:`, `abstract:`, or `excerpt:` field (checked in that order), its text is used **exactly as you wrote it**.
2. **Your summary callout.** If the note body contains a `> [!summary]`, `> [!abstract]`, or `> [!tldr]` callout, its text is used **exactly as you wrote it** — including diacritics and punctuation, preserved verbatim.
3. **A generated summary.** Only if you wrote neither of the above does Constellation generate one — by reading the note and extracting its most central sentences (see below).
4. **An opening-text fallback.** For a note the engine can't split into sentences (for example, text in a script without clear sentence punctuation), it shows the note's opening lines instead of a ranked summary.

> **The one rule that matters most:** steps 1 and 2 mean a summary you wrote is *never* overwritten. If you see a generated summary on a note you thought you'd summarized, it means the engine didn't find your summary where it looks — check that your frontmatter field is one of the four names above, or that your callout is one of the three types above.

---

## How a generated summary is built

When Constellation has to generate a summary (because you didn't write one), it does **extractive** summarization — it selects sentences that are already in your note, rather than inventing new prose. The method is a well-established one (TextRank, Mihalcea & Tarau 2004):

1. **Split into sentences.** The note body is segmented into sentences using the Unicode standard for sentence boundaries, so it works across languages and scripts.
2. **Read each sentence's meaning.** Each sentence is turned into a small numeric "meaning fingerprint" (an embedding) using a compact on-device model.
3. **Rank by centrality.** Sentences that are most similar in meaning to the most *other* sentences score highest — these are the sentences that best represent the note as a whole.
4. **Take the top three, in order.** The three highest-ranked sentences are shown **in the order they appear in the note**, so the summary reads naturally rather than out of sequence.

Very long notes are handled gently — the engine caps how much of the body it scans and how many sentences it ranks, so summarizing a huge note never slows the app down or risks a crash.

Because it is extractive, a generated summary is always made of sentences you actually wrote. It will never put words in your mouth.

---

## Summaries are read-only — File-Over-App

Constellation **never writes a generated summary back into your note.** Your `.md` files are the source of truth; the summary you see on a card is computed on the fly and cached separately, not saved into the file's text or frontmatter.

This is deliberate, and it follows Constellation's *File-Over-App* principle: the app is a window onto your files, not an editor that quietly changes them. If you want a summary to live *in* the note, write one yourself (a `summary:` field or a `[!summary]` callout) — and then, by the precedence rule above, Constellation will show yours and stop generating.

Everything is computed **on your device.** No note text is ever sent anywhere to be summarized.

---

## Where summaries appear, and how they fill in

Summaries surface across Constellation wherever a note shows up:

- **Cataloger / Source Review queue** — under each card's title (the original surface — see *The Cataloger*).
- **Search results** — a faint italic line under each hit, beneath the snippet. The snippet shows *why* a result matched your query; the summary line shows what the note is *about*. Together they let you scan results without opening anything.
- **Editor** — a thin muted band above the note body when you open a note, so the note's gist is in-context as you read or write. The band hides itself when there's no summary yet (a brand-new note, or one whose summary is still being computed).
- **Backlinks panel** — under each source row that links to the note you're reading. A long list of inbound mentions becomes scannable: you read the linking note's gist as a single italic line beneath its title, without having to open each one to remember what it is.
- **Outgoing links panel** — under each target row that the note you're reading links out to. Same shape as Backlinks; you see at a glance what each outgoing connection is about.
- **Index** — when you expand a term, each note that uses the term shows the summary as a faint line beneath its title (and beneath the matched-context snippet, when one is present). A term that appears in dozens of notes becomes a list of ideas rather than just filenames.
- **Sky View hover** — when you hover a bubble in the **Sky View** graph, the floating tooltip shows the note's name on the first line and its summary headline on a second italic line, so you can read what a bubble *means* without leaving the graph.

By default summaries fill in **lazily and gently**: as cards scroll into view, as search results appear, as you open a note, or as you expand a term / hover a bubble, Constellation computes the missing summaries a few at a time, pausing whenever a Library classification scan is running so the two never compete for resources. This keeps the app responsive — you may briefly see a card / result / opened note / row / tooltip before its summary appears, then the summary pops in a moment later.

If you'd rather have every summary ready ahead of time — so every surface shows summaries instantly — use **Build all summaries**.

---

## Build all summaries — pre-computing the whole Library

The **Build all summaries** button (in the **Cataloger** header) pre-computes a summary for **every note that doesn't already have a current one**, so cards show their summary instantly instead of filling in as you scroll.

**To use it:**

1. Open the **Cataloger** (the stacked-cards icon in the left dock).
2. Click **Build all summaries** in the header. The button changes to *Building note summaries…*.
3. Progress appears in the **status bar** at the bottom of the window — you can keep working while it runs.
4. To stop early, use the **Cancel** control on the status-bar progress strip. A partial run is fine; it picks up where it left off next time.

A few things worth knowing:

- It runs **only when you ask** — it never starts on its own, so it can never slow down app startup.
- It runs **in the background** on a separate thread; typing and navigation stay instant.
- It is **resumable** — if you cancel it, or close the app mid-run, the next run continues from where it stopped rather than starting over.
- It only computes summaries that are **missing or out of date** — notes whose summary is already current are skipped, so a second run is fast.

---

## Making sure your own summary is used

On a card, the summary appears under a single **Summary** label — the card does not badge whether the text came from you or from the engine. What decides that is the precedence above: if a note has one of the frontmatter fields or one of the summary callouts, Constellation shows *that* and never generates one.

So if a note shows a summary that reads like the machine chose it, that note has neither a frontmatter summary nor a summary callout — and the fix is to add one:

- Add a `summary:` (or `description:` / `abstract:` / `excerpt:`) field to the note's frontmatter, **or**
- Add a `> [!summary]` (or `[!abstract]` / `[!tldr]`) callout to the body.

The next time that note's summary is computed — when its card next loads, or after you run **Build all summaries** — your words take over.

---

## Common workflows

**"A note shows a machine summary, but I wrote one."**
Constellation didn't find your summary where it looks. Make sure your frontmatter field is named `summary`, `description`, `abstract`, or `excerpt`, **or** that your callout is `[!summary]`, `[!abstract]`, or `[!tldr]`. Then re-open the Cataloger (or click *Build all summaries*) to refresh.

**"I want every card to show its summary the instant I open the Cataloger."**
Click **Build all summaries** once and let it finish. After that, summaries are pre-computed and appear immediately.

**"I want the summary to be part of the note itself, on disk."**
Write it yourself — add a `summary:` frontmatter field or a `> [!summary]` callout. Constellation will then show your version (and stop generating one), and your words live in the file where any other app can read them too.

---

## Related topics

- **The Cataloger** — the full-page home where summaries appear under each card, and where *Build all summaries* lives.
- **Source Review** — the classification cards the summaries sit on.
- **Properties** — the `summary:` / `description:` / `abstract:` / `excerpt:` frontmatter fields, and how to add them.
- **Editing and Formatting** — how to write a `> [!summary]` callout in a note.
