---
aliases:
  - Federation
  - Linked Universe
  - Linked Universes
  - Federated Universe
  - Cross-Universe
description: Federation lets one Universe show the content of other Universes (added as Linked Universes) without copying or merging their data. The gravity well (CNS), Sky View, Backlinks, Outgoing Links, the tag list, Five Acts, and Workspace Bases all read across your Linked Universes at runtime. Search is the exception today — see "A Known Limit" below. Each Linked Universe keeps its own files; detaching one leaves it fully intact and able to stand alone.
---

# Federation (Linked Universes)

## What Is It?

A **Universe** in Constellation is a complete, self-contained knowledge base — its own folder on disk with its own notes, libraries, links, and index. **Federation** lets you add one Universe to another as a **Linked Universe**, so the parent's windows can show the linked universe's content alongside its own.

The key idea — and Constellation's firm rule: **federation reads, it never merges.** When Universe A is added as a Linked Universe of Universe B, nothing is copied, moved, or rewritten. B simply *reads* A's data at runtime and displays it. A's files stay exactly where they are, untouched. The one refinement: when you perform an operation *on* one of A's own notes — renaming it, say — that operation does its work inside A, with A's own settings and vocabulary, and its bookkeeping lands in A's own records. B never rewrites A's notes because of something that happened in B.

## Why Does It Matter?

Most tools force a choice: keep knowledge bases separate (and lose the ability to see across them), or merge them into one (and lose their independence). Federation gives you both:

- **See across** — your whole intellectual world in one window: search finds notes from every Linked Universe; the gravity well shows the full connected graph; tags, Five Acts, and bases aggregate across universes.
- **Stay independent** — each Linked Universe remains a real, standalone Universe. Detach it at any time and it works on its own, with all its data intact, exactly as before it was federated.

This is the "the wheel is already there" principle: each Universe already has everything it needs. Federation just lets a parent borrow the view, never the data.

## What Federates (reads across Linked Universes)

When you're in a Universe that has Linked Universes, these surfaces show federated content:

- **Search** — **does NOT currently reach a Linked Universe's notes.** This is the one surface in
  this list that stops at the boundary, and it returns *no results* rather than saying it did not
  look. See "A Known Limit" below for exactly what is and is not searched. (Corrected 2026-08-23:
  this entry claimed the opposite, and the claim was wrong.)
- **Constellation Nervous System (CNS)** — the gravity well shows every connected note across the federation.
- **Sky View** — the same federated node set, as bubbles.
- **Backlinks** and **Outgoing Links** — show links across the federation (each universe's links resolve within that universe).
- **Tag list** (in the Notes Navigator) — aggregates tag counts across universes.
- **Five Acts** — the sidebar's Five Acts section lists each Linked Universe's "Observation — Recent Captures" under a collapsible group labelled with the Linked Universe's name.
- **Workspace Bases** — each Linked Universe's saved bases appear under a collapsible per-universe group.
- **Library count / note count** (status bar) — totals span the federation.

### Collapsible per-universe groups

In the **Five Acts** and **Workspace Bases** sidebar sections, your active Universe's entries show directly. Each Linked Universe's entries appear below, grouped under a collapsible header with the Linked Universe's name and a count. Click the header to expand or collapse that universe's entries. They start collapsed so the sidebar stays tidy.

Linked Universe bases are **open-only** — you can open them to view, but you cannot delete or rename a Linked Universe's base from the parent's sidebar. That protection is deliberate: the parent never modifies a Linked Universe's files.

## What Stays Per-Universe (by design)

A few things are intentionally *not* federated, because they belong to the universe you're actively in:

- The **per-note Tags panel** (right sidebar Tags tab, *This note* mode) shows the tags of the note you have open — it's about the open note, not the universe. Flip the same tab to **All tags** for the federated, universe-wide **Tag Browser**: a sorted, filterable tree of every tag across the active Universe *and* its Linked Universes (sort by A→Z, Z→A, or count; the header, sort bar, and filter stay frozen as you scroll). Click any tag to search it everywhere.
- **Bookmarks** are scoped to the active Universe.
- The **`Five Acts/` folder** is hidden from the file tree (in every Universe) because its note is surfaced through the dedicated Five Acts section instead — this avoids showing the same note twice. The file is never deleted; it stays on disk.

## The Folder a Linked Universe Lives In

The link between two universes is recorded as a **location on disk**. If a Linked Universe
physically sits inside a folder of the universe you have open, that folder is now protected:
renaming, moving or deleting it is **refused**, with the linked universe named in the message, and
nothing is changed.

Without that guard the recorded location would stop existing and the linked universe would simply
stop appearing — no error, no explanation. If you want to reorganise the folder, unlink that
universe first, or open it directly and work inside it.

This only applies to a universe nested inside another's folder. Linked Universes kept side by side
are untouched by it.

## A Known Limit: Ordinary Search Does Not Span Linked Universes Yet

Worth knowing so you do not misread a result: **searching from Search Hub looks only inside the
universe you have open.** A note that lives in a Linked Universe will not appear — and it comes back
as *no results*, not as a message telling you it was not searched, which is the misleading part.

This applies to plain text and to nearly every written search form: `#tag`, `in:Library`,
`property=value`, `links to [[Note]]`, the typed-link searches and `orphans` all look only in the
open universe. (The one partial exception is a search that mixes free text WITH one of those
operators, where the free-text half alone reaches further — which is not a distinction you should
have to know, and is part of why this is filed as a gap.)

This is a gap, not a design decision — federation exists so that knowledge connects across
universes, and a search that stops at the boundary is incomplete. It is recorded and will be closed.

## How Detaching Works

Because federation only ever reads, detaching a Linked Universe is lossless:

1. Remove the link from the parent (or open the Linked Universe directly as its own Universe).
2. The (former) Linked Universe has all its notes, links, Five Acts, bases, and tags exactly as they were.
3. When you switch into it as the active Universe, its Five Acts shows by default in the Five Acts section, its bases in the Bases section, and so on — it behaves as a complete, standalone Universe.

Nothing about being federated changes the Linked Universe's own data. It was always its own Universe; federation just let a parent look in.

## Related

- **Knowledge Hierarchy** — Universe → Library → Folder → Note, and where Linked Universes sit.
- **Constellation Nervous System** — the federated gravity well.
- **Search** — see the known limit above: searching from Search Hub does not currently reach
  Linked Universes.
