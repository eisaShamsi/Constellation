# PJ-069 Note-Lists Cluster — Right-Click on Every Computed List — Concept Paper

**Date:** 2026-07-07 · **Status:** **DRAFT for Boss ratification.** PJ-069's biggest *form*-duplication cluster (note-lists ×26), triggered by the Boss's Stage-1 ask: *"Enabling RC [right-click] functions on the 'Reviewer', and in all functions that list notes as a result of their computation."* Concept-first per the bring-up method.

**Basis:** the PJ-069 re-audit (`wf_2ae0f8c0-d59`) — 26 live hand-rolled note-list surfaces, the shipped `NoteRow`/`NoteList` primitive (1 adopter), the shared `ContextMenu`/`buildContextMenu`, and the MIG-077 "SAFE right-click subset" precedent. The Boss banked Obsidian's right-click menus as the target (`docs/concept-papers/Right-Click-Reference-Obsidian.md`).

---

## 1. The horse

> **Every list of notes the app *computes* is a working set the user wants to act on — so every such list carries the same right-click menu, sourced once (the shared note row + the shared context menu), never re-implemented per surface.**

The Boss's right-click ask and PJ-069's note-lists dedup are the **same move**: today 26 surfaces hand-roll their own note-row markup, and almost none carry a right-click menu. Adopting the shared `NoteRow` — which *carries* the shared menu — gives every computed list its right-click **and** collapses the 26-way drift into one home, in a single act. That is "one home per capability" for the note row, and it delivers exactly what the Boss asked for.

## 2. The gap — which computed lists lack right-click today

Right-click exists on a few surfaces: the File Explorer tree, Search Hub results, Collections (via `NoteRow`), and Base table rows (the MIG-077 §B2 SAFE subset). It is **absent** from almost every *computed* note-list:

- **The Reviewer** (the Boss's headline) — its master queue rows are hand-rolled, no menu.
- **Around a note:** Backlinks, Outgoing, 360° lists, Suggested Connections, Index term-mentions.
- **Attention:** Tasks / Open Loops, Calendar day lists, the Tension/Health lists.
- **The whole picture:** Knowledge Health worry-lists, CCS rows, Cataloger queue, Digest, Structure/Provenance, OrgChart per-node lists, Dashboard recents/tag-notes, the sidebar Starred + Five-Acts lists, ExpressionForge, the Second-Screen companion lists.

A user who finds a note through *any* of these computations can't act on it in place — they have to leave the surface. Every one of these is a set the computation surfaced *because it matters*; not being able to right-click it is a gap in exactly the moments the app is being most useful.

## 3. The design — the shared row carries the menu

**One primitive, one menu, adopted everywhere:**

1. **`NoteRow` gains the shared right-click** — it wires `oncontextmenu` to the host's `buildContextMenu`, rendering the shared `ContextMenu.svelte`. Because `NoteRow` is self-contained (per the pill-drift lesson), the menu is identical on every surface that mounts it.
2. **The menu is the SAFE subset for computed lists** (MIG-077 §B2 precedent): **Open · Open in new tab · Reveal in Explorer · Star/Unstar · Add to collection ▸ · Copy link/name**. It deliberately excludes **Rename / Move / Delete** — those mutate the file and would leave a *computed* list (which doesn't re-run on mutation) showing a stale row pointing at a moved/renamed/deleted note. (The File Explorer keeps the full menu because it *is* the file system and refreshes.)
3. **Adopt `NoteRow` across the 26 surfaces** — starting with the Reviewer — which simultaneously (a) delivers the right-click and (b) retires each hand-rolled row, shrinking the cluster from 26 homes to 1. Surfaces that virtualize already sit on the shared `VirtualList`; `NoteList` bundles `VirtualList` + `NoteRow`'s height contract, so adoption is drop-in for those.

This is the "one list, N mounts" pattern (the sanctioned `RelatedCandidates` model) applied to the note row itself.

## 4. Precedents (this is not new invention)

- **MIG-077 §B2 SAFE right-click subset** — the exact "no rename/move/delete from a non-refreshing computed list" ruling; already shipped on Base rows. This cluster generalizes it.
- **The shared `ContextMenu.svelte` + `buildContextMenu`** — the one menu renderer; leaf surfaces forward `(entry, x, y)` to the host, which mounts it. Already used by the tree, Search Hub, OrgChart, Index, NotePane.
- **`NoteRow`/`NoteList`** (MIG-090/092) — the shipped shared row, built explicitly (its header cites PJ-069) for this adoption; today mounted only by Collections.
- **Obsidian's right-click menus** (Boss-banked reference) — the target vocabulary for the actions.

## 5. Scope & plan sketch (pending ratification)

Each adoption is a small, independently-testable swap (the row's data/actions stay host-owned; only the row *rendering* + the menu move to the shared primitive), so this pipelines cleanly:

- **Step A — the shared row-with-menu:** give `NoteRow` the SAFE-subset right-click via `buildContextMenu`; one localized menu, ×15.
- **Step B — the Reviewer** (the Boss's headline): adopt `NoteRow` in the Reviewer's master list. First Boss test.
- **Step C — the "around a note" panels:** Backlinks, Outgoing, 360°, Suggested Connections, Index mentions (they already share `VirtualList`).
- **Step D — attention + whole-picture lists:** Tasks, Calendar, Tension/Health, KH, CCS, Digest, Structure, Provenance, Cataloger, Dashboard, sidebar Starred/Five-Acts, ExpressionForge, OrgChart.
- **Step E — the Second-Screen companion lists** (display-not-domain: mount the same row).

Every step obeys the hard constraint: virtualize lists >50 rows (they already do), zero new per-keystroke IPC, per-title RTL (NoteRow already carries it).

## 6. Boss decisions

1. **Ratify the horse** (§1) — the right-click and the note-lists dedup are one move via the shared row.
2. **The menu contents** (§3.2) — confirm the SAFE subset (Open / Open-in-new-tab / Reveal / Star / Add-to-collection / Copy), *excluding* Rename/Move/Delete on computed lists. Add/remove any action?
3. **Scope of the first pass** — the full 26-surface adoption, or the Boss's priority order (Reviewer first, then the highest-value lists)?
4. **One migration or fold into PJ-069's note-lists cluster?** — this IS the note-lists cluster; ship it as that cluster's `/migration` (it doubles as the dedup), or a narrower "right-click-only" pass first?

---

*This session's deliverable stops at this concept paper. On ratification: Architect (per-surface adoption inventory + the SAFE-menu contract) → Plan → build (Reviewer first, staged Boss tests).*
