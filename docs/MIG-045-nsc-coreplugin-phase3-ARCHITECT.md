# MIG-045 — NSC Core Plug-in, Phase 3: the Universe Digest left-dock view

**Status:** Architect + Plan. Eisa pre-approved the cascade ("Proceed through all the remaining phases" + "PCS-3 > MIG-045 Phase 3").
**Date:** 2026-05-24
**Lineage:** Phase 3 of the NSC Core Plug-in roadmap (Concept Paper v2.0 §10). Builds on Phases 1 (engine `headline` + shared store + 2 surfaces) and 2 (5 more surfaces). This is the second pillar of the Core Plug-in — the **dock view that lets the user skim the whole knowledge base at summary level without opening notes**.

---

## 1. Goal

Ship the **Universe Digest** as a left-dock pane: a tiered, scrollable browse of every note in the Universe (and its cUniverse children) at summary-headline level. Default tiering is **Library → Folder → Headline**, sorted by recency within each tier. The user can expand any headline row to see the full multi-sentence summary inline, and can search/filter the whole Digest by headline / note name / library.

The Digest IS the second pillar of the Concept Paper v2.0 vision ("a left-dock view to skim the whole knowledge base at summary level without opening notes"). Phases 1–2 made the summary service available to every surface; Phase 3 builds the surface that's purely about summaries.

---

## 2. Locked design decisions (from the 2026-05-22 Concept Paper §9 Q&A)

These were chosen by Eisa during the Concept Paper drafting and ratified at Phase 1 ship. They are **not in scope for re-litigation** here — Phase 3 implements them:

| Decision | Choice | Source |
|---|---|---|
| Dock-view name | **"Digest"** | Concept Paper §9 |
| Headline storage | **Stored** `headline` column on `note_summaries` | Concept Paper §9 + MIG-043 (already shipped) |
| cUniverse federation | **In scope v1** — Digest spans child universes (same tiering, child-universe nodes appear as their own top-level Library rows) | Concept Paper §9 |
| Mode | **Extractive only** for v1; abstractive (LLM rewrite) is a future upgrade | Concept Paper §9 |
| Default sort | **Recency** within Library → Folder | Concept Paper §9 |
| Granularity | **Tiered Library → Folder → 1-line headline**, expandable to full summary | Concept Paper §9 |
| Shape | Both service + dock view | Concept Paper §9 (Phase 1–2 = service; this phase = dock) |
| Service reach | All surfaces | Phase 2 shipped this |

---

## 3. Territory — what's IN, what's OUT of Phase 3

**IN:**
- New left-dock pane component (`DigestPane.svelte` or similar — name TBD in Plan Step A).
- Mount in `+layout.svelte` left dock alongside existing panes (File tree, Outline, etc.).
- Top-bar (title + filter input + sort toggle + cUniverse toggle).
- Tiered list (Library header → Folder sub-header → Note row with headline).
- Per-row expansion (click → show full multi-sentence summary inline; click again → collapse).
- Recency sort (default): within each Library, folders sorted by max(child note mtime); within each folder, notes sorted by mtime desc.
- Alternative sort: alphabetical (one toggle in the top-bar).
- Filter: substring match on (note name, headline, full summary) — same `.includes()` shape as the Index panel's filter.
- Virtualization: the existing `VirtualList.svelte` component (already used by IndexPanel for the 7k+-note universe) — Digest rows are typed (`'library-header' | 'folder-header' | 'note' | 'expanded-summary'`) and the list is flattened the same way IndexPanel does.
- cUniverse federation: child-universe libraries appear inline at the top level (the existing `resolve_libraries_recursive` flattening already gives a federated library list).
- i18n: full string-set in all 15 locales.
- Help topic: new `docs/help.*/The Digest/The Digest.md` topic in all 15 locales.

**OUT (defer to future MIGs):**
- Drag-to-reorder rows (the Digest is read-only — sort comes from rules).
- Per-row context menu (open in new tab is the primary action; right-click can come later if Eisa wants it).
- Custom user-defined groupings (Library → Folder is the only tiering for v1).
- Abstractive / LLM-rewritten headlines (locked OUT of v2.0 per the Concept Paper).
- Map disabled per MIG-038 (no Digest entry for Map).
- Second-screen mount (Digest is a primary-window pane; second-screen comes later if Eisa wants it, per `feedback_display_not_domain`).

---

## 4. Invariants that MUST NOT break

1. **No new IPC.** The Digest reads ENTIRELY through the existing `getSummariesFor(paths)` shared store. The list of notes to feed it comes from the already-loaded `skyNodes` / `libraries` state (the same arrays the Sky View consumes). Zero Rust changes.
2. **No new schema.** `note_summaries.headline` already exists (MIG-043). No new tables, no new columns.
3. **Cache-first + batched.** Every visible row's summary comes from the store's cache. The Digest fetches in viewport-sized batches (e.g. 50 paths per IPC), gated on scroll position — never one IPC per row.
4. **Virtualized.** Universes with 10k+ notes must scroll smoothly. `VirtualList.svelte` (already used by IndexPanel) is the reference implementation; copy the same pattern.
5. **No reactive loops (CLAUDE.md Rule 2).** Every new `$effect` reads its trigger reactive vars and writes a *different* `$state` (use `untrack` for cache writes per the existing IndexPanel pattern).
6. **No boot regression.** The Digest is lazy-mounted (only when its dock slot is open). On first mount it reads the already-loaded `skyNodes` / `libraries` — no boot-time fetch.
7. **No hot-path heavy work.** Filter input debounced ≥150ms (matches IndexPanel). Sort operations on 7k+ items use chunked sort or single-pass key extraction.
8. **Author authority preserved** (already encoded in the engine via Phase 1 — `headline` is `first_sentence` of author text when present).
9. **cUniverse federation parity.** The same federation that powers Sky View / Cataloger lights up the Digest. Child-universe libraries appear inline (no separate "linked universes" section in v1 — they're peers).
10. **Rollback safe.** Removing the Digest dock entry reverts cleanly; no schema change to roll back, no IPC consumers stranded.

---

## 5. Design options

### A. Dock placement

- **A1 — Left dock, alongside File tree / Outline (CHOSEN).** Matches "a left-dock view to skim the whole knowledge base" from the Concept Paper §10. Users already think of the left dock as "the place where I navigate the knowledge base," and the Digest is a navigation view at a different granularity (summary-level instead of file-level). Toggle via a new dock-icon in the left rail (next to Files / Outline / etc.).
- A2 — Right dock, next to Backlinks / Outgoing. Rejected — those are *contextual* (about the current note); the Digest is *global* (about the whole Universe).
- A3 — Bottom dock / new full-page mode. Rejected — Digest is browsing UX, not full-page reading UX. Bottom dock would compete with status/console; full-page would duplicate what the Cataloger already does for card-level review.

### B. Row expansion UX

- **B1 — Inline click-to-expand on headline row (CHOSEN).** Click anywhere on a headline row → the row's height expands to show the full multi-sentence summary inline. Click again → collapses. Matches IndexPanel's term-expansion UX exactly. The user already learned this gesture there.
- B2 — Floating popup on hover. Rejected — too many accidental triggers on scroll; popup positioning gets complex with virtualization.
- B3 — Side-panel detail when row clicked. Rejected — splits the Digest UX into "list and detail," doubles the visual complexity, and the full summary is short enough (2–3 sentences) to fit inline.

### C. Federation rendering

- **C1 — Inline federation: child-universe libraries appear as peer top-level rows (CHOSEN).** The existing `resolve_libraries_recursive` (in `universe.rs`) already flattens the federation tree into a single library list — the Digest just consumes that list. A small badge / icon next to library-header rows that come from child universes (e.g. "↗ from {cUniverse name}") so the user knows the origin without it being visually separate.
- C2 — Grouped section "Linked universes" below the parent universe's libraries. Rejected — adds tier complexity for marginal clarity; the inline-peer model matches how Sky View / Cataloger already federate.
- C3 — Per-universe tabs. Rejected — single-universe at a time defeats the "Universe Digest" framing; the whole point is a unified view.

### D. Sort & filter toolbar shape

- **D1 — Compact top-bar with filter input + sort dropdown + cUniverse toggle (CHOSEN).** Mirrors IndexPanel's top-bar pattern (filter input + script tabs + alpha/freq sort toggle + actions). Users know the shape. Filter input takes most of the width; sort + cUniverse-on toggle are small icon-buttons.
- D2 — Separate filter sidebar. Rejected — adds visual complexity for a panel that already has tight horizontal space.
- D3 — No in-panel filter; rely on global search. Rejected — the Digest is a browse UX; needing to leave it to filter would break the flow.

### E. Initial data source

- **E1 — Reuse `skyNodes` and `libraries` from `+layout.svelte` (CHOSEN).** Both are already populated at boot (skyNodes for the Sky View) and reactive. The Digest reads them via `$derived` to build its tiered tree. Zero new fetches, zero new IPC, zero boot delay.
- E2 — New IPC `digest_get_universe_tree`. Rejected — invariant #1 (no new IPC). And it would duplicate data that's already in memory.
- E3 — Lazy-load per-library. Rejected — for the recency sort to work across libraries, we need all paths upfront; lazy-loading would force a multi-stage sort that's worse perf-wise than reading the already-populated `skyNodes`.

### F. Headline-fetching strategy

- **F1 — Viewport-batched via the shared store (CHOSEN).** As rows scroll into the virtualized window, the visible paths are collected and `getSummariesFor(visiblePaths)` is called in batches (e.g. one IPC per scroll-induced refresh, capped at 50 paths per batch). Cache-first means re-scrolling is free. Same shape as BacklinksPanel / IndexPanel from Phase 2.
- F2 — Preload all summaries on first mount. Rejected — for a 10k+ note Universe that's a 200-IPC barrage on dock-open; we cap it via Phase 1's "lazy and gentle" fill anyway.
- F3 — Run NSC backfill on Digest open. Rejected — `Build all summaries` already exists in the Cataloger for that purpose; making the Digest a trigger would surprise users.

---

## 6. Plan (each step = one commit or one bundled commit per concern)

> **Step A — `DigestPane.svelte` skeleton.** New component file under `src/lib/components/`. Props: `nodes: SkyNode[]`, `libraries: Library[]`, `cUniverseChildren?: ...`, `onNoteClick`. Empty top-bar + empty virtualized list scaffolding (use `VirtualList.svelte` from `$lib/components/`). `let dockOpen = $state(false)` plumbing in `+layout.svelte`. New i18n keys for `digest.title`, `digest.empty`, `digest.filterPlaceholder`.
> *Verify:* svelte-check 0 new; pane mounts and shows "Empty" placeholder when toggled.

> **Step B — Tiered tree derivation.** `$derived` `treeRows: VRow[]` from `nodes + libraries`. VRow types: `'library-header' | 'folder-header' | 'note' | 'expanded-summary'`. Flatten in sort order (Library asc → Folder asc → Note recency desc by default). Library headers carry a "from cUniverse" badge if `library.universe_id !== current`.
> *Verify:* svelte-check 0 new; pane shows the correct tier structure on a small universe (the dev `eisa-cognitive-knowledge` library — verify a known Library → Folder → Note path appears in the right order).

> **Step C — Headline fetch + render.** State `summaryHeadlines: Map<path, string>` + state `expanded: Set<path>` + state `fullSummaries: Map<path, string>`. `$effect` over the visible window's paths → `getSummariesFor(visiblePaths)` → merge into `summaryHeadlines` with the same `changed` guard pattern as Phase 2. Render: each note row shows `name` + `headline` inline; expanded rows show full `summary` from the entry. Inline expansion handler toggles `expanded`.
> *Verify:* svelte-check 0 new; on the dev universe, headlines fill in within a frame or two of opening the dock; clicking a row expands to show the full multi-sentence summary; collapsing works.

> **Step D — Sort + filter + cUniverse toggle.** Sort dropdown: "Recency" (default) / "Alphabetical." Filter input (debounced 150ms; `.includes()` on `note.name + headline + full summary`). cUniverse toggle: when off, filter `libraries` to only the current universe's. All three update the `treeRows` derivation.
> *Verify:* svelte-check 0 new; filter typing instantly narrows the list (no IPC); sort switch reorders correctly; cUniverse toggle hides/shows child-universe libraries.

> **Step E — Virtualization wiring + row heights.** Hook `VirtualList`. Per-row heights: `library-header = 30`, `folder-header = 24`, `note = 36` (name + headline lines), `expanded-summary = computed by line-count + 8 padding`. `scrollResetKey` includes filter + sort.
> *Verify:* svelte-check 0 new; smooth scroll on a 7k+ note universe (target: 60fps wheel scroll on a Library with 2k+ rows expanded).

> **Step F — `/simplify` + Phase E audit.** `/simplify` the diff. 3 agents in parallel: invariants (§4), drift (any new consumer of summaryStore not via the canonical pattern; any $effect loop), migration-path (rollback / fresh DB / mid-backfill / empty NSC cache).

> **Step G — SO + 15-locale help additions + PCS-4.**
> - **New help topic in English:** `docs/help.uConstellation.World/The Digest/The Digest.md` — what the Digest is, when to use it (vs. Cataloger / Search / Sky View), how the tiering + sort + filter work, federation note.
> - **14-locale translation** of the new help topic via background sub-agent using each locale's established native term ("Resumen del Universo" / "ملخص الكون" / etc. — TBD; agent picks the locale's idiomatic equivalent).
> - **Note Summaries help** in all 15 locales: add the Digest as the 8th surface in the existing "Where summaries appear" list.
> - **Orientation v2.29** (new file alongside v2.28).
> - **Session log §5** appended (this MIG's ship details).
> - **MoCh fresh file** for the Phase 3 work.
> - PCS-4 commit + push (Boss explicit go required).

---

## 7. Migration-path matrix

| Scenario | Behavior |
|---|---|
| Fresh DB / new universe | No schema change. Digest is empty until libraries / notes exist; then it renders cards as they're discovered, with headlines filling in gently. |
| Existing DB | Reads existing `note_summaries.headline` (populated by Phase 1's `headline` column + any backfill). Universes whose NSC cache is empty render note rows with empty-headline placeholders; the Cataloger's "Build all summaries" populates them. |
| Mid-backfill | Headlines appear as the backfill completes. No competing IPC — the Digest uses the same `getSummariesFor` store as everything else. |
| Universe switch | `clearAll()` not called (store is path-keyed; new universe's paths just don't exist in cache yet). Digest re-derives `treeRows` from the new `skyNodes` + `libraries` via reactivity. |
| Rollback to MIG-044 | Pulling out `DigestPane.svelte` + its dock entry + i18n keys + help topic reverts cleanly. No schema, no IPC, no data state to migrate. |
| cUniverse added/removed at runtime | `libraries` array updates → `treeRows` re-derives → child-universe rows appear/disappear inline. No special handling. |

---

## 8. Risk summary

**Low-to-moderate.** The wiring shape is well-understood from Phase 2 (cache-first + batched + virtualized + i18n). The main risks:

- **Perf on 10k+ universes.** Mitigated by virtualization + viewport-batched fetches; verify in Step E with explicit scroll-fps measurement on the dev "Eisa Cognitive Knowledge" universe.
- **Tier-sort correctness with cUniverse federation.** Mitigated by reusing the proven `resolve_libraries_recursive` output; verify in Step B with a federation-having universe.
- **Help topic discovery.** A new help topic needs the User Manual / help-index to surface it. Step G picks this up — verify the help search finds "Digest" after the topic ships.
- **Naming consistency.** The DOCK ENTRY label needs to be localized to "Digest" in EN and to the chosen native equivalent in each locale. Step A's i18n keys + Step G's translation are the contract.

No cross-subsystem risk (frontend-only, no Rust, no schema). No new write paths. No new IPC.

---

*Phase 3 of the NSC Core Plug-in roadmap — the dock view that completes the Concept Paper v2.0 vision. On completion: PCS-4 + orientation v2.29 + new help topic in 15 locales. This is the last of the three NSC MIGs; what comes after is at Eisa's direction.*
