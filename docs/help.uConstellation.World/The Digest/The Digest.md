---
aliases:
  - The Digest
  - Universe Digest
  - Digest
  - Digest pane
description: The Universe Digest is a left-dock pane that shows every note in your knowledge base at summary-headline level — tiered Library → Folder → Note — so you can skim the whole Universe without opening anything. Click a row to expand and see the full summary inline. Filter narrows the whole list; sort toggles between recency (default) and alphabetical. Reads the same summaries you see everywhere else; no extra computation; no extra disk space.
---

# The Universe Digest

> *Think of the Digest as a table of contents for your mind — not a list of files, a list of ideas.*

The **Universe Digest** is the place to skim your whole knowledge base at the level of *meaning*. Instead of a file tree (names only) or the Sky View (shapes only), the Digest shows you, under every note, the **one sentence that says what the note is about**. Tap a row and the full multi-sentence summary expands inline. You can read fifty notes' worth of substance in a minute, without ever opening one.

It lives in your **left dock**, alongside the File tree, the Notes Navigator, and the Sky View — one of the four ways Constellation lets you navigate.

---

## Why the Digest exists

A file tree tells you what you *have*. A search tells you what you *asked for*. The Digest tells you what you *know*.

When your Universe grows past a few hundred notes, "opening each one to remember what it says" becomes impossible. You need a way to read the **gist** of each note at the speed of scrolling — and a way to expand any gist into the full summary the moment you want to think about it more carefully. That's the Digest.

It's the third pillar of the **Note Summary Creator (NSC)** Core Plug-in:
- **Pillar 1**: a summary engine (Phase 1 / MIG-043).
- **Pillar 2**: a service that puts the summary wherever a note shows up (Phase 2 / MIG-044 — Cataloger, Search results, Editor band, Backlinks, Outgoing links, the Index, Sky View hover).
- **Pillar 3**: this view — the Universe Digest (Phase 3 / MIG-045).

---

## Opening the Digest

In the **left sidebar**, click the **Universe Digest icon** (a small list with a circle in the corner) — it's the fourth icon in the row, next to File tree / Notes Navigator / Sky View. The sidebar switches to the Digest pane.

To switch back, click any of the other three icons (or press **Escape**).

---

## What you see

From top to bottom:

1. **Toolbar.** A search input + a small clock icon (the sort toggle, default "by recency").
2. **Library headers.** Uppercase purple bars — one per library in your Universe. Each shows the library's name and a count of how many notes it contains.
3. **Folder headers.** Small muted labels — one per folder *that contains notes*. Notes that live at the library root don't get a folder header.
4. **Note rows.** Each row has:
   - A chevron (▶) on the left — click it to expand the row.
   - The **note name** in interactive-accent color — click it to **open the note** in the editor.
   - A faint italic line beneath the name — the **summary headline** (the same one that appears in every other Phase 1/2 surface).

---

## Expanding a row to read the full summary

Click the **chevron** (▶) at the left of a row — or click the **headline italic line** itself. The chevron rotates to ▼ and the **full multi-sentence summary** appears inline beneath the headline, wrapping naturally across as many lines as it needs.

Click the chevron (or headline) again to collapse.

The "click the chevron to expand, click the name to open" split keeps the two gestures distinct: you can expand to *read about* a note, then keep scrolling past it; only when you click the name does the note actually open and take focus.

---

## Filtering

Type into the **search input** at the top. The list narrows as you type — only notes whose **name, headline, or full summary** contains your query stay visible. Library headers and folder headers with zero matching notes disappear entirely (no empty headers).

Clear the input (× button or backspace) to restore the full list.

The filter is **instant** — Constellation doesn't hit your disk or the database. It reads the summaries already in memory, so even a 10,000-note Universe filters at typing speed.

---

## Sort: recency or alphabetical

Click the **clock icon** in the toolbar to toggle between two sort modes:

- **Recency** (default) — within each folder, notes appear in order of **creation time, newest first**. Folders within a library are sorted by the most-recent note they contain (so the most active folder appears first). This is the default because it surfaces *what you've been working on lately*.
- **Alphabetical** — folders sorted by name, notes within each folder sorted by name. Click again to return to recency.

The toggle is per-session; close and reopen the Digest and it goes back to recency.

---

## Federation: Linked Universes appear inline

If your Universe has **Linked Universes**, every library from a Linked Universe appears in the Digest as **its own peer Library header**, alongside your own Universe's libraries. The Digest is a unified view of everything reachable from this Universe, not just the libraries that physically live here.

(A future Constellation update will add an on/off toggle to hide Linked-Universe libraries from the Digest temporarily; for now they always appear.)

---

## How the Digest stays fast on huge Universes

The Digest is **virtualized**: it renders only the rows currently visible in your scrollport, not the entire tree. A 10,000-note Universe scrolls as smoothly as a 50-note one. As rows scroll into view, their summaries are fetched in batches from Constellation's in-memory cache (the same cache that powers every other Phase 1/2 surface — no separate work, no separate storage).

The Digest never re-reads your notes from disk. It never re-computes summaries. It is a **read** view onto the same `note_summaries` table that the engine populates from Phase 1.

---

## Common workflows

**"I want to see what I worked on this week."**
Open the Digest with sort = Recency (default). The most-recently-created notes appear at the top of each library/folder. Scan the headlines.

**"I'm looking for a half-remembered note about X."**
Open the Digest. Type X (a word that would appear in the note's title, headline, or full summary). The list narrows to candidates. Click chevrons to read full summaries; click the name to open the winner.

**"I want to write a top-down review of my Library."**
Open the Digest, sort = Alphabetical. Walk the headlines in order. Click chevrons to read fuller summaries when something catches you. Use this as the spine of a new MOC (Map of Content) note.

**"I'm exploring a Linked Universe for the first time."**
Open the Digest. Scroll past your own libraries to the Linked Universe's libraries — they're peer rows. Read the headlines to learn what the Linked Universe contains, without opening anything from it.

---

## What's NOT in the Digest

- **Right-click context menu** on rows — opening in a new tab, archiving, etc. (For v1, the primary actions are click-name-to-open and click-chevron-to-expand. A future update will add a context menu.)
- **Custom groupings** — Library → Folder is the only tiering for v1. (No "group by tag" or "group by stage" yet.)
- **Drag-to-reorder** — the Digest is read-only; sort comes from rules, not manual ordering.
- **Cataloger-like classification controls** — the Digest is a *browse* view; classification lives in the **Cataloger** (separate pane).

---

## Related topics

- **Note Summaries** — where summaries come from, the precedence rule (yours wins), and the full list of surfaces that show them.
- **The Cataloger** — the home of *Build all summaries* (pre-compute every summary in your Library at once so the Digest fills instantly).
- **Sky View** — the *shape* view of your knowledge (bubbles + links); the Digest is its complementary *meaning* view.
- **Knowledge Formulation** — why Constellation organizes knowledge by *connection* and *summary*, not just file storage.
