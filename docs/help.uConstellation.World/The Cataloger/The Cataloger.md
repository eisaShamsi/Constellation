---
aliases:
  - The Cataloger
  - Cataloger
  - Classify notes
  - Classification home
  - CECE home
  - Scan library
description: The Cataloger is the universe-wide home for classifying your notes. It is the full-page dock view where you run the Constellation Epistemic Content Engine (CECE) over your Library, classify any note on demand, build note summaries, and work through the review queue. If Source Review is the card you act on, the Cataloger is the room you do it in.
---

# The Cataloger

> *"Classify each note by its kind of knowledge and its source."*

The **Cataloger** is the universe-wide home for classification. It is a full-page view, opened from the left dock, that gathers everything you need to read your notes through Constellation's knowledge taxonomy in one place: a control to scan the whole Library, a way to classify any single note on demand, a button to build note summaries, and the live review queue where you Accept, Edit, Reject, or disambiguate each suggestion.

If you have used the **Source Review** panel in the right sidebar, you already know the cards. The Cataloger is the same engine and the same cards, promoted out of a narrow sidebar tab and given the whole window — plus two things the sidebar tab never had: a note-picker and a "Build all summaries" button.

---

## "The Cataloger" vs "the catalogers" — a quick word on names

These two names look alike on purpose, but they mean different things:

- **The Cataloger** (capital *T*, this view) is the *place* — the full-page room where classification happens.
- **the catalogers** (lowercase, plural) are the *six lenses* inside the engine — frontmatter, citations, wordstems, linked notes, similar notes, and AI judgment — each of which reads a note and votes. Five of the six are active today; the sixth (AI judgment) is built but not yet switched on.

So: you open **the Cataloger**, and inside it **the catalogers** do the reading. The six-lens machinery is explained in full in the **Source Review** topic — this topic is about the room.

---

## What it is

The Cataloger answers one question: **"How is each note in my universe classified — and what still needs my decision?"**

It is built around four things stacked top to bottom:

1. **A header with three actions** — *Classify a note…*, *Build all summaries*, and *Start scan*.
2. **A progress strip** — appears only while a Library scan is running, showing how far it has got.
3. **The review queue** — the same Accept / Edit / Reject / Disambiguate cards as the Source Review panel, now full-width.
4. **A note summary under each card** — a short, plain-language précis of the note so you can decide without opening it (see *Note Summaries* below, and the dedicated **Note Summaries** topic).

Everything runs **on your device**. No note ever leaves Constellation.

---

## Why it matters

Classification is how Constellation turns a pile of `.md` files into a *shaped* body of knowledge — every note placed on two axes (where the knowledge came from, and what kind of knowledge it is). That shape is what powers **Constellation Sight**, the **Epistemic Metadata** panel, and taxonomy-aware search.

But classification is a decision-heavy job. When you have hundreds of unclassified notes, doing it from a thin sidebar tab — one note at a time, with no way to summon a specific note — is slow. The Cataloger exists to make the job *sittable*: open it once, give it the whole screen, and work through your Library in a single focused session. The note-picker lets you pull in any note by name; the summaries let you judge a card without leaving the room; the scan control seeds the queue in bulk.

---

## How to open it

1. In the **left dock** (the vertical strip of icons at the far edge of the window), click the **stacked-cards icon** — three small cards layered on top of each other. It sits among the other workspace icons such as the Sight eye and the Nervous System neuron.
2. The Cataloger opens as a **full-page view**, taking over the content area.
3. To close it: click the **(×)** at the top-right of the header, or press **Esc**. You return to wherever you were.

> **Note on Esc:** if the *Classify a note…* search popover is open, pressing **Esc** closes just the popover and leaves the Cataloger open. Press **Esc** again (with the popover closed) to close the Cataloger itself.

---

## What you see

### The header — three actions

At the top of the Cataloger, three controls sit side by side:

| Control | What it does |
|---|---|
| **Classify a note…** | Opens a small search box. Type a few letters of any note's title, pick it from the results, and the engine classifies it on the spot — no need to open the note first. The new suggestion appears in the queue below. |
| **Build all summaries** | Pre-computes a short summary for every note that doesn't already have one. Runs quietly in the background; progress shows in the status bar at the bottom of the window; you can cancel any time. (Detailed in the **Note Summaries** topic.) |
| **Start scan** | Runs the engine across your **whole active Library** at once, queuing a suggestion for every note that isn't yet classified. While it runs, the button reads *Running…* and a progress strip appears below the header. |

### The progress strip

Directly under the header, a thin strip appears **only while a Library scan is running**. It shows how many notes have been processed and lets you watch the scan complete. When no scan is running, the strip is hidden and the queue sits right under the header.

### The review queue

The bulk of the Cataloger is the **review queue** — the same cards you see in the Source Review panel, just full-width. Each card shows a note, the engine's read of how it fits your taxonomy (Source × Content Type), the six small cataloger dots, and the actions you can take:

- **Accept** — write the engine's suggestion to the note and clear the card.
- **Edit** — pick the values yourself from a tree.
- **Reject** — clear the card without writing anything.
- **Disambiguate** — on a "split" card, pick the right value from the candidate chips.

The full mechanics of the cards — the colored dots, the confidence regimes, sibling disambiguation, the queue filter chips, "Approve all", and per-Library calibration — are documented in the **Source Review** topic. The Cataloger uses that exact panel; nothing about the cards changes between the sidebar and the full-page view.

### The note summary under each card

Under each card's title sits a short **Summary** line — a few sentences that tell you what the note is about, so you can judge the card without opening the note. This is produced by the **Note Summary Creator (NSC)**; see the next section and the **Note Summaries** topic.

---

## Classifying a single note — the note-picker

The *Classify a note…* button solves a simple problem: in the sidebar tab, you could only classify the note you currently had open. The Cataloger has no "open note", so it gives you a way to summon any note by name.

**To classify one note:**

1. Click **Classify a note…**. A search box drops down with the placeholder *Search notes…*.
2. Start typing the note's title. After a brief pause, matching notes appear in a list (up to ten).
3. Click the note you want. The engine classifies it, the popover closes, and a fresh card for that note appears in the queue below.
4. If something goes wrong (a rare engine error), the message appears inside the popover so you know the classification didn't run.

You don't have to open the note, and you don't lose your place in the queue. This is the fastest way to classify a specific note you have in mind.

---

## Note Summaries (NSC) inside the Cataloger

Every card in the queue carries a short **Summary** of its note, shown under the title. The summary is produced by the **Note Summary Creator (NSC)** and follows one firm rule: **if you wrote a summary, the engine uses yours; it only generates one when you haven't.**

The order of precedence is:

1. **Your frontmatter summary** — a `summary:`, `description:`, `abstract:`, or `excerpt:` field in the note's properties. Used exactly as you wrote it.
2. **Your summary callout** — a `> [!summary]`, `> [!abstract]`, or `> [!tldr]` block in the note body. Used exactly as you wrote it, diacritics and all.
3. **A generated summary** — only if you wrote neither of the above. Constellation reads the note, finds its most central sentences, and shows the top three in their original order.

The engine **never writes a generated summary back into your note** — your `.md` files are the source of truth and the Cataloger only ever *reads* them.

The **Build all summaries** button pre-computes summaries for the whole Library in the background, so cards show their summary instantly instead of filling in as you scroll. Full detail — including how the generated summaries are produced and what to do if a summary looks wrong — is in the **Note Summaries** topic.

---

## What the Cataloger does *not* do

- **It does not classify automatically in the background by default.** Scans are something you *start*. (There is an optional background mode in Settings → Intelligence → CECE, off by default — see **Source Review**.)
- **It does not call any cloud service.** The five active catalogers are heuristic and local. The sixth lens (AI judgment, a local language model) is built into the design but not switched on yet, so it stays silent on every card today.
- **It does not change your notes' wording.** Accepting a card writes classification *properties* (the `sources:` and `content_type:` frontmatter fields). It never edits your prose, and it never writes a generated summary into the file.

---

## Common workflows

**"I just opened the Cataloger for the first time — where do I start?"**
Click **Start scan** to queue a suggestion for every unclassified note in the Library. Watch the progress strip fill. Then work down the queue, accepting the ones the engine got right and disambiguating the split ones. The summaries under each card let you decide quickly.

**"I want to classify one specific note, not the whole Library."**
Click **Classify a note…**, type its title, click it. A card appears in the queue. Accept or edit it.

**"My cards take a moment to show their summaries."**
Click **Build all summaries** once. It pre-computes every note's summary in the background (progress in the status bar). After it finishes, summaries appear instantly.

**"The queue has hundreds of cards — how do I focus?"**
Use the filter chips above the queue (documented in **Source Review**): start with *Catalogers agreed* and *Approve all* to clear the easy ones, then tackle the split cards.

---

## Related topics

- **Source Review** — the cards themselves: the six catalogers, the colored dots, confidence regimes, sibling disambiguation, queue filters, "Approve all", and per-Library calibration. The Cataloger embeds this panel.
- **Note Summaries** — how the Summary line under each card is produced, the author-first precedence, and the *Build all summaries* backfill.
- **Cognitive Engine** — the broader knowledge-formulation philosophy classification fits into.
- **Epistemic Metadata** — the `sources:` and `content_type:` properties classification writes, and how to read them.
- **Constellation Sight** — the spatial view that the Source × Content Type classification powers.
