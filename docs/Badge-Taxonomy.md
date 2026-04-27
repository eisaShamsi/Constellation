# Constellation Badge Taxonomy

> Canonical reference for the colored letter/symbol badges that appear next to search results in **Constellation Map** and **Constellation Sight**. Each badge tells the user **where the match was found** in the note (or what kind of relationship the link represents). One result can carry multiple badges.

## Where badges appear

| Surface | Component | Renders via |
|---|---|---|
| Constellation Map — search panel | `src/lib/components/ConstellationMap.svelte` | `CAT_COLORS` map at lines 80–84; rendered at line 660 (current result) and line 711 (result list). |
| Constellation Sight — graph view | `src/lib/components/ConstellationSight2.svelte` | `CAT_COLORS` map at lines 79–83. |

Both components ship the **same letter set** with **identical colors**. Keep them in sync when adding or changing badges.

## Confirmed badges

### Content / structural matches

These badges indicate **where in the note the search query matched**.

| Badge | Meaning | Color | Hex |
|---|---|---|---|
| **T** | Title | Blue | `#3b82f6` |
| **C** | Content (body text) | Green | `#16a34a` |
| **P** | Property (frontmatter key/value) | Amber | `#f59e0b` |
| **S** | Semantic (embedding similarity) | Purple | `#7c3aed` |
| **W** | Wikilink (`[[target]]`) | Grey | `#94a3b8` |
| **#** | Tag / Hashtag (`#tag` or YAML `tags:`) | Pink | `#f472b6` |
| **∅** | Empty / Null result | Slate | `#64748b` |

### Link relationship badges

These appear when a result is matched **by virtue of how it links to / from** another note.

| Badge | Meaning | Color | Hex |
|---|---|---|---|
| **LT** | Link Target (this note links *to* the queried note) | Green | `#16a34a` |
| **LF** | Link From (this note is linked *from* the queried note) | Red | `#ef4444` |
| **⇄** | Bidirectional (mutual link in both directions) | Violet | `#8b5cf6` |
| **LB** | Link Back (backlink hit) | Light blue | `#0ea5e9` |
| **LA** | Link Alias (matched via the link's display alias rather than its target) | Pink | `#d946ef` |
| **M** | Mutual link (the queried note links *to* the source AND the source links *back*) | Cyan | `#06b6d4` |

## Deprecated / superseded

| Badge | Status |
|---|---|
| **G** | Earlier identifier for Tag/Hashtag. Superseded by **#**. Not present in current code; documented here so future readers wondering "where is `G`?" find the answer. |

## Adding a new badge

When introducing a new badge:

1. Add it to **both** `CAT_COLORS` maps (`ConstellationMap.svelte:80-84` and `ConstellationSight2.svelte:79-83`) with the same hex color in both.
2. Add a row to the appropriate section of this document.
3. If the badge is set somewhere outside these two components (e.g. emitted by the search Rust backend), grep for `searchCats` to find the producer and document what conditions cause the badge to be applied.
4. Update §13.1 of the current orientation doc (`docs/Constellation Orientation & Onboarding v1.X.md`) so the orientation summary stays truthful.

## Source-of-truth invariants

- The two `CAT_COLORS` maps **must** agree on letter → color. Drift between Map and Sight produces the same badge in two different colors, which confuses users.
- Letter identifiers should be **single uppercase ASCII** (T, C, P, S, W, M) **or** standard symbols (#, ∅, ⇄), **or** short two-letter pair codes for relationships (LT, LF, LB, LA). Avoid lowercase or multi-character mnemonics — they break the visual rhythm of the badge row in dense search-result lists.
- A badge meaning, once shipped, should not be **silently** redefined. If superseded, document the supersession (as `G` → `#` is documented above).

## Provenance

- 2026-04-15: Document created. T/C/P/S/W/#/∅/LT/LF/⇄/LB/LA confirmed by project owner. **W** clarified as Wikilink (was unresolved). **G** confirmed deprecated in favor of **#**. **M** remains pending.
- 2026-04-27: **M = Mutual link** confirmed by project owner. Moved out of Unresolved into the Link-relationship badges table. No more pending letters.
