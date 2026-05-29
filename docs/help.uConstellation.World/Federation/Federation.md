---
aliases:
  - Federation
  - cUniverse
  - Child Universe
  - Federated Universe
  - Cross-Universe
description: Federation lets one Universe show the content of other Universes (added as cUniverses) without copying or merging their data. Search, the gravity well (CNS), Sky View, Backlinks, Outgoing Links, the tag list, Five Acts, and Workspace Bases all read across your federated cUniverses at runtime. Each cUniverse keeps its own files; detaching one leaves it fully intact and able to stand alone.
---

# Federation (cUniverses)

## What Is It?

A **Universe** in Constellation is a complete, self-contained knowledge base — its own folder on disk with its own notes, libraries, links, and index. **Federation** lets you add one Universe to another as a **cUniverse** (child Universe), so the parent's windows can show the child's content alongside its own.

The key idea — and Constellation's firm rule: **federation reads, it never merges.** When Universe A is added as a cUniverse of Universe B, nothing is copied, moved, or rewritten. B simply *reads* A's data at runtime and displays it. A's files stay exactly where they are, untouched.

## Why Does It Matter?

Most tools force a choice: keep knowledge bases separate (and lose the ability to see across them), or merge them into one (and lose their independence). Federation gives you both:

- **See across** — your whole intellectual world in one window: search finds notes from every cUniverse; the gravity well shows the full connected graph; tags, Five Acts, and bases aggregate across universes.
- **Stay independent** — each cUniverse remains a real, standalone Universe. Detach it at any time and it works on its own, with all its data intact, exactly as before it was federated.

This is the "the wheel is already there" principle: each Universe already has everything it needs. Federation just lets a parent borrow the view, never the data.

## What Federates (reads across cUniverses)

When you're in a Universe that has cUniverses, these surfaces show federated content:

- **Search** — finds notes across the parent + all cUniverses.
- **Constellation Nervous System (CNS)** — the gravity well shows every connected note across the federation.
- **Sky View** — the same federated node set, as bubbles.
- **Backlinks** and **Outgoing Links** — show links across the federation (each universe's links resolve within that universe).
- **Tag list** (in the Notes Navigator) — aggregates tag counts across universes.
- **Five Acts** — the sidebar's Five Acts section lists each cUniverse's "Observation — Recent Captures" under a collapsible group labelled with the cUniverse's name.
- **Workspace Bases** — each cUniverse's saved bases appear under a collapsible per-universe group.
- **Library count / note count** (status bar) — totals span the federation.

### Collapsible per-universe groups

In the **Five Acts** and **Workspace Bases** sidebar sections, your active Universe's entries show directly. Each cUniverse's entries appear below, grouped under a collapsible header with the cUniverse's name and a count. Click the header to expand or collapse that universe's entries. They start collapsed so the sidebar stays tidy.

cUniverse bases are **open-only** — you can open them to view, but you cannot delete or rename a cUniverse's base from the parent's sidebar. That protection is deliberate: the parent never modifies a cUniverse's files.

## What Stays Per-Universe (by design)

A few things are intentionally *not* federated, because they belong to the universe you're actively in:

- The **per-note Tags panel** (right sidebar Tags tab, *This note* mode) shows the tags of the note you have open — it's about the open note, not the universe. Flip the same tab to **All tags** for the federated, universe-wide **Tag Browser**: a sorted, filterable tree of every tag across the active Universe *and* its cUniverses (sort by A→Z, Z→A, or count; the header, sort bar, and filter stay frozen as you scroll). Click any tag to search it everywhere.
- **Bookmarks** are scoped to the active Universe.
- The **`Five Acts/` folder** is hidden from the file tree (in every Universe) because its note is surfaced through the dedicated Five Acts section instead — this avoids showing the same note twice. The file is never deleted; it stays on disk.

## How Detaching Works

Because federation only ever reads, detaching a cUniverse is lossless:

1. Remove the cUniverse link from the parent (or open the cUniverse directly as its own Universe).
2. The (former) cUniverse has all its notes, links, Five Acts, bases, and tags exactly as they were.
3. When you switch into it as the active Universe, its Five Acts shows by default in the Five Acts section, its bases in the Bases section, and so on — it behaves as a complete, standalone Universe.

Nothing about being federated changes the cUniverse's own data. It was always its own Universe; federation just let a parent look in.

## Related

- **Knowledge Hierarchy** — Universe → Library → Folder → Note, and where cUniverses sit.
- **Constellation Nervous System** — the federated gravity well.
- **Search** — federated search across cUniverses.
