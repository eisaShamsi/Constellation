# MIG-044 — NSC Core Plug-in, Phase 2: full service reach (remaining surfaces)

**Status:** Architect + Plan (Phase 1–2). Eisa pre-approved the cascade ("Proceed through all the remaining phases").
**Date:** 2026-05-23
**Lineage:** Phase 2 of the NSC Core Plug-in roadmap (Concept Paper v2.0 §10). Builds on Phase 1 / MIG-043 (engine `headline` + shared frontend summary store + 2 first surfaces).

---

## 1. Goal

Wire **NSC summary headlines** into every remaining enabled note-displaying surface, using the same shared store (`src/lib/nsc/summaryStore.ts`) Phase 1 established. This completes the "summary service feeding every Constellation function" half of the Concept Paper's vision. (Phase 3 / MIG-045 builds the Universe Digest left-dock view itself.)

---

## 2. Territory map — remaining enabled surfaces (verified 2026-05-23)

| Surface | Component | Render mode | Action |
|---|---|---|---|
| **Backlinks panel** | `src/lib/components/BacklinksPanel.svelte` | list of inbound links per active note | Add headline under each linked-note row |
| **Outgoing-links panel** | `src/lib/components/OutgoingLinksPanel.svelte` | list of outbound links per active note | Add headline under each linked-note row |
| **Index panel** | `src/lib/components/IndexPanel.svelte` | term browser — expanding a term shows note rows | Add headline under each note-mention row |
| **Sky View bubble inspector** | `src/lib/components/SkyView.svelte` / `FullSkyView.svelte` | PIXI bubbles + on-click inspector | Add headline in the inspector for the focused node |

**Explicitly OUT of Phase 2 scope:**
- **Map** — disabled (MIG-038). Skip.
- **Hover / wikilink-previews** — no such surface exists in current code (grep-confirmed). If one is added later, wire it under MIG-046 or fold into the relevant feature.
- **SecondScreenPage / LocalSkyView** — the second-screen display window is a *display surface*, not a primary interaction surface; defer unless Eisa flags it. (Per memory `feedback_display_not_domain`: additional screens are displays, not domains.)

---

## 3. Invariants that MUST NOT break (same as Phase 1)

1. **Cache-first + batched everywhere** — every surface uses `getSummariesFor(visiblePaths)` (batched, no per-item IPC on render). Surfaces with >50 rows virtualize and request only visible.
2. **No hot-path heavy work** — `$effect`s gate on the *visible note set* changing, not on every keystroke/scroll frame.
3. **No reactive loops (CLAUDE.md Rule 2)** — every new `$effect` reads its trigger reactive vars and writes a *different* `$state` (with a `changed` guard where it merges into a Map).
4. **No boot regression** — surfaces are lazy-mounted; no boot-time fetch.
5. **Author authority** — headline reflects author-summary when present (already encoded in the engine via `first_sentence` of author text, Phase 1).
6. **No new IPC** — every surface reuses `nsc_get_summaries_for_notes` via the shared store. Zero Rust changes (this is a frontend-only MIG).
7. **No new schema, no new write path** — nothing changes on disk.
8. **Existing Cataloger / Source Review / search results / editor band** all keep working unchanged.

---

## 4. Design options

### A. Render granularity per surface

For each surface: do we show the **1-line headline** or the **full 2–3 sentence summary**? Each surface has a different vertical-space cost profile:

- **A1 — Headline (1-line) everywhere (CHOSEN).** Consistent density: under each row, a faint italic single line. Matches Phase 1's search-results + editor-band style. Single design language across the app. Users learn it once.
- A2 — Full summary in Sky View inspector (more vertical space available), headline in compact panels. Rejected — inconsistent UX; the inspector "feels different" without clear gain (the user can open the note for the full summary).

### B. Backlinks / Outgoing — placement

These panels already render a small "source/target note name" row per link, sometimes with a snippet of context. The headline goes **under the existing row** as a second line (italic, muted), same shape as SearchHub's `.sh-item-headline`. **Chosen** for consistency.

### C. Index panel — placement

`IndexPanel.svelte` is a term browser: pick a term, expand to see the notes that mention it. Each note-mention row already shows note title + library + context-snippet. Headline goes **under the existing row**. **Chosen.**

### D. Sky View inspector — placement

When the user clicks a node bubble in SkyView/FullSkyView, an inspector panel appears with the note's metadata (title, library, stratum, etc.). Headline goes **under the title in the inspector**, italic + muted. **Chosen.**

### E. Shared utility vs. per-component logic

Each surface needs: `$state` for summaries + `$effect` to fetch + render. The boilerplate is small (~15-25 lines per surface). **Not** worth factoring into a shared helper for Phase 2 (the variations in *visible-path computation* per surface make a shared helper awkward). If Phase 3's Digest view ends up needing the same pattern, we could extract then. **Chosen: per-component for Phase 2.**

---

## 5. Plan (each step = one commit or one bundled commit per surface)

> **Step A — Backlinks panel.** Add `summaryHeadlines: Map<path, string>` `$state`; `$effect` over the visible backlinks → `getSummariesFor` → merge. Render `.bp-item-headline` (or namespaced) under each row. CSS.
> *Verify:* svelte-check 0 new; open a note with backlinks → headlines appear under each link row.

> **Step B — Outgoing-links panel.** Mirror Step A. Same pattern, namespaced CSS.
> *Verify:* same; open a note with outgoing links → headlines appear.

> **Step C — Index panel.** Add summary fetch in the term-expansion path (when a term is expanded → visible note-mention rows → fetch their summaries). Render headline under each mention row.
> *Verify:* svelte-check 0 new; expand any term → headlines appear under the mentions.

> **Step D — Sky View bubble inspector.** Add `activeHeadline` `$state` + `$effect` on the focused-node path → `getSummaryFor` → render headline in the inspector. (Single-note case; mirrors NoteEditor's pattern from Phase 1.)
> *Verify:* svelte-check 0 new; click any bubble → inspector shows headline.

> **Step E — `/simplify` + Phase-D audit.** `/simplify` the diff. 3 agents: invariants (§3), drift (any unmapped consumer, any `$effect` loop), migration-path (nothing to migrate — pure UI).

> **Step F — SO + docs + 15-locale help additions + PCS-3.**
> Update the Note Summaries help in all 15 locales: the surface list grows from 3 to ~7 (Cataloger / Source Review / Search results / Editor + Backlinks / Outgoing / Index / Sky View inspector). Orientation v2.28. SESSION-LOG-2026-05-23 §2. MoCh update. Commit + push.

---

## 6. Migration-path matrix

| Scenario | Behavior |
|---|---|
| Fresh DB / existing DB | No schema change — purely additive UI. |
| Rollback to MIG-043 (Phase 1) | Surfaces revert to no-headline rendering; nothing breaks. |
| Mid-backfill | Surfaces render what the cache has; empty headlines hidden (consistent with Phase 1). |
| Universe with empty NSC cache | BUG-022 auto-rebuild fires on open (Phase 1); once filled, surfaces show headlines. |

---

## 7. Risk summary

**Very low.** No new IPC, no schema change, no engine change. Pure frontend additive integrations of an established pattern. The only attention point is virtualization where rows can be many (Index panel can have many mentions per term; Sky View has one focused node — trivial). Pattern matches Phase 1 (which Boss-validated cleanly), so quality + perf risk is well-understood.

---

*Phase 2 of the NSC Core Plug-in roadmap. On completion: PCS-3 + orientation v2.28, then MIG-045 (Phase 3 — the Universe Digest itself) cascades next per Eisa's "proceed through all" directive.*
