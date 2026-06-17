# MIG-079 §C.2c — SME panel findings + the sound solution (no more patching)

> Produced 2026-06-17 after Boss ruling "Enough patching → employ SME agents to research, validate, audit, and provide sound solutions." Four parallel SME agents: (1) diagnostic, (2) virtualization design, (3) WA#5 best-practices, (4) correctness audit. Their findings converge.

## Root cause (SME-1 diagnostic — evidence-based, not guessed)

The switch-lag patches were aimed at the wrong layer. The hub churn ("fighting for memory", scroll not smooth) is **the un-virtualized RENDER**, ranked:

1. **DOMINANT — rendering every row.** `BacklinksPanel`/`OutgoingLinksPanel` render the FULL list (`{#each filteredBacklinks}`, no virtualization). A hub like ISBN (~5,358 backlinks) builds **~55,000 DOM nodes + thousands of reactive `LinkTypePill` component subscribers synchronously, and tears them all down + rebuilds on every note switch.** This is the memory churn + scroll stutter.
2. **CLOSE SECOND, compounding — the NSC summary `$effect` fetches for ALL rows.** It collects every path in `filteredBacklinks` (all 5,358), fires one 5,358-path IPC, and on resolve triggers a **second full render pass** (the `summaryHeadlines` write invalidates every row). `OutgoingLinksPanel` is worse: a per-target `resolveWikilinkCrossLibrary` IPC **per target**.
3. Secondary — `getBacklinks` sort/map/dedupe over N (~tens of ms; an order of magnitude below the DOM cost).
4. **NOT a factor** — the editor ×N re-decoration (`linkTraversalMap` ← `activeOutgoingRows`) is O(visible viewport), driven by the small outgoing list, capped by `view.visibleRanges`.

**Verdict: virtualize the panels → both (1) and (2) collapse to O(visible rows).** This is the CLAUDE.md Rule-3 line ("virtualize every list that can exceed 50 items: …backlinks") that was never applied to these two panels.

## WA#5 validation (SME-3 — the approach is the proven standard, by negative example)

Virtualize + fetch detail only for visible rows + coalesce switches is **the** industry pattern. Validated hard by counter-example: Roam (100 refs → 10–60 s, 850 MB→2.1 GB), Logseq (editor "unusable" on pages with a few references), Obsidian (Backlink Cache plugin exists precisely because the native pane degrades) — all the SAME freeze Constellation hit, all from rendering/recomputing references at read time WITHOUT windowing. Refinements the field adds: (a) **dynamic/measured row height, estimate biased HIGH**; (b) feed the window from a persisted index (we have `note_links` + §C.2a — done); (c) **overscan/prefetch + per-row cache + cancel-on-scroll-past**; (d) tighten the switch coalesce toward **~100–120 ms** (Nielsen's "instant" ceiling) OR keep ~180 ms paired with an immediate skeleton (our `panelLoading` + retained previous rows already is the skeleton).

## The sound solution (SME-2 design + SME-4 audit), built as ONE unit

**Reuse `VirtualList.svelte`** exactly as `IndexPanel.svelte` does. Steps (each landable + Boss-testable):

- **C.2c-3a — BacklinksPanel virtualized.** Hoist the row markup into a shared `{#snippet backlinkRow(bl)}` used by BOTH the `{#each}` (small notes, <50 rows → byte-identical to today) AND `<VirtualList>` (≥50). `ROW_*` height constants + `getItemHeight(bl)` accounting for the optional annotation/headline lines (context is empty for the per-note path); re-derive on `summaryHeadlines.size` (IndexPanel's signal). **Bounded-height (the one real constraint, SME-2):** VirtualList needs a bounded `clientHeight`, but the panels currently grow to natural height (the flank/sidebar scrolls). **Option A (chosen, minimal-risk):** wrap the list in `.bl-vlist-wrap { display:flex; flex-direction:column; max-height:60vh; min-height:0 }` — no host-layout change. (Option B — panel owns full height — is `/migration`-tier; rejected for this pass.) Filter input + section header stay OUTSIDE the VirtualList. `scrollResetKey={filterQuery}` (NOT summaryHeadlines — a late headline must not jump scroll).
- **C.2c-3b — NSC headline fetch → visible window only.** Ship the **head-cap** first (fetch headlines for the first ~120 rows; rows below render without a headline — a soft enhancement) — self-contained, no shared-component change. Optional fast-follow: add an additive optional `onVisibleRange` prop to VirtualList for true visible-window fetch.
- **C.2c-3c — OutgoingLinksPanel virtualized** (mirror; no filter/unlinked) + **cap the per-target `resolveWikilinkCrossLibrary` IPC** (the heavier one) to the visible/head window.
- **C.2c-3d — fix the SME-4 P1 + nit** (below), and **tighten the switch coalesce to ~120 ms**.

## SME-4 audit — correctness (mostly PASS; two items folded into the solution)

- **PASS:** alias-aware match == old getBacklinks (rehearsal-proven); confidence/archive nonce refresh; stale-result guard; Editor-Surface Gate (read-path only); flag-OFF rollback intact; federation (single + cUniverse, graceful skip of older schema); no timer leak (the 180 ms `setTimeout` is cleared in the `$effect` cleanup).
- **P1 — ×N chips vanish in NON-FOCUSED split panes (flag-on).** `linkTraversalMap` derives from `activeOutgoingRows` = the FOCUSED tab's outgoing only; split view renders an editor per open tab, each needing its OWN note's outgoing. Single-pane is fine; split panes lose ×N chips. **Fix:** maintain outgoing rows per OPEN tab (a small `Map<path, NoteLink[]>`, a few cheap fetches) and union into `linkTraversalMap` — folded into C.2c-3d.
- **nit — `status='active'` (new row queries) vs `status!='archived'` (legacy) predicate drift.** Identical today (all rows active); diverges only if a third lifecycle state appears. Align to one convention (`status != 'archived'`) in C.2c-3d. (Also noted: `NoteLink` has no `status` field, so the JS `l.status!=='archived'` re-filter is a dead no-op — server-side filtering is authoritative.)

## Process lesson (logged)
§C.2c-2 shipped the fetch-swap, then the perf was patched live 3× on the switch-timing — the wrong layer. The render was never the timing; it was the un-windowed DOM. Designing the COMPLETE panel (fetch + coalesce + virtualized render + visible-window detail) as one proven unit — what this doc is — is the Solve-the-Class discipline that the patching violated.
