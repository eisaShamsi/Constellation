# Session Log — 2026-05-01

Continuation of the §92-onward Inspector 360.3D arc that started in `SESSION-LOG-2026-04-29.md`. New file because the calendar date has rolled over.

---

## §115 — Stage 1 + Stage 2 retest follow-ups bundled

Boss walked Stage 1 (S1.1 → S1.6) and Stage 2 (S2.1 → S2.6) of the matrix tutorial in sequence. All sub-stages PASSED structurally. Six refinements surfaced across the walk; bundled into one rebuild as §115 per Boss's "implement after we finish the test" directive.

### Findings collected

**Stage 1**:
- S1.3.5 — "It cannot manage a higher number of dots, and you can't collapse back. I think the practical way is, instead of displaying more untitled dots, to display a list of the respective note titles in that box."

**Stage 2**:
- S2.3 (general) — "It belongs to S1. Since the note title is already visible at the top, we don't need to have it again within the respective Stratum."
- S2.5 (general) — "The Type Row text is hard to read because of its background color."
- S2.6 — "We need to make the Grand total number visible."

### Code changes

`src/lib/components/Inspector360.svelte` — frontend only.

1. **Expanded typed cell now renders as a list of titles.** Cell switches layout from `flex-wrap center` (dots) to `flex-direction: column` (list). Each item is a clickable button: dot bullet (type-coloured) + truncated note name. Click navigates to that note via `onNoteClick`. Untyped column still excluded — typically 100s of connections, would balloon the matrix.
2. **Always-visible `×` collapse button** absolutely-positioned at top-right of the expanded list. Replaces §114's `−` chip placed at the end of the dots, which was hard to find when the cell had scrolled. New placement: `position: absolute; top: 4px; inset-inline-end: 4px; z-index: 3`.
3. **`max-height: 240px` + `overflow-y: auto`** on the inner list-scroll container. Cells with 49 connections at one (stratum, type) intersection no longer push the matrix row past the canvas. List scrolls inside the cell.
4. **Active-note chip removed from row label.** Was: row header showed `L7 Paradigm [Abu Bakr]` chip. Now: just `L7 Paradigm`. The active row's purple highlight and accented row number still signal "this is the active stratum"; the note's name is in the matrix header where it always was.
5. **Column header contrast fix.** Background gradient tint reduced from 22 % type-colour to 10 %. Text colour switched from full type colour to `color-mix(in srgb, var(--col-color) 55%, var(--text-normal))` so type-coding remains while contrast against the tinted background improves. Bottom border keeps full-strength colour as the type signal.
6. **Grand total displayed in top-right corner.** `matrix` derived now also returns `grandTotal` (sum across the deduped cells matrix). Top-right cell — formerly just `Σ` — now shows `Σ` stacked over the grand-total number (e.g. `Σ` / `926`). Layout: `flex-direction: column; align-items: center; gap: 2px`. Confirms at a glance that column-totals sum equals row-totals sum equals this grand total (fail signal: any of the three diverge).

### Verification

- `cargo check`: not re-run (no Rust change).
- `node node_modules/svelte-check/dist/src/index.js --tsconfig ./tsconfig.json --threshold error`: 1 error (pre-existing `store.ts:1850`, out of scope). Zero in `Inspector360.svelte`.
- Release build: pending.

### SO #6

Orientation **v1.18** created alongside v1.17. v1.17's content demoted to its own subsection. The §115 callout enumerates the six fixes inline with Boss's exact phrasing where applicable.

### Pending after §115

- Boss verification round on the §115 binary: re-test S1.3.5 (list-of-titles), spot-check S2.3 (chip removed), S2.5 (column header contrast), S2.6 (grand total visible).
- Stage 3 plan (TBD) — moving from "matrix renders, matrix reads" to "matrix interpretation" once the visual is settled.
- MIG-006 §3 redo (queued).
- CE Phase 9 Path B / MIG-010 scale (queued after MIG-006 §3).

---

## §116 — Verification A retest fixes (nav-reset + Untyped expandable)

Boss tested the §115 list-of-titles via Verification A. Three findings:

- Step 1 (expand) = Pass.
- Step 2 (click navigates) = Pass, but: "when we move to the clicked node, the list is still expanded in the new node, which is not logical. It should collapse by default when we move to another node."
- Step 3 (× collapse) finding tied to Step 2: "When we are back, it should collapse automatically."
- Step 4 (Untyped not expandable) = "Let's have the 'untyped' expandable like the other type."

The two underlying behaviours collapsed into two fixes.

### Code changes

`src/lib/components/Inspector360.svelte` — frontend only.

1. **Auto-reset cell expansion on note navigation.** New `$effect` watches `data?.note_path`; when it changes (forward navigation via title-click → parent's `onNoteClick` fires → +layout updates `data`; or backward via back-bar → onBack restores prior `data`), the effect resets `expandedCells = new Set()`. The current expansion state thus belongs to the active note alone.

2. **Untyped exclusion removed.** `toggleCellExpand` no longer early-returns on `type === 'untyped'`. The template `{#if expanded}` branch is the same for typed and untyped. The collapsed `+N` overflow indicator is now a `i360-overflow-btn` clickable button uniformly; the previous `i360-overflow` non-clickable text span path was removed. The expanded list-of-titles renders identically — Untyped just uses the dark-grey colour for its bullets and benefits from the same `max-height: 240px` scroll cap.

### Verification

- `cargo check`: not re-run.
- `node node_modules/svelte-check/dist/src/index.js --tsconfig ./tsconfig.json --threshold error`: 1 error (pre-existing `store.ts:1850`, out of scope). Zero in `Inspector360.svelte`.
- Release build: pending.

### SO #6

Orientation **v1.19** created alongside v1.18. Session log §116 entry appended (this entry).

### Pending after §116

- Boss verification re-run on §116 binary: confirm cells collapse on navigation + Untyped now expandable.
- Verification B (chip removed, header contrast, grand total visible) — still owed.
- Stage 3 plan (TBD).

---

## §117 — Column header tint 10 % → 5 % (Verification B Check-2 follow-up)

Boss tested §115's column-header contrast fix via Verification B. Check 1 (chip removed) and Check 3 (grand total visible) passed. Check 2 (column header text contrast): "Lower the tinted background more."

§115 had reduced tint from §113's 22 % to 10 %. §117 lowers it again to 5 %. Text colour (`color-mix(--col-color 55%, --text-normal)`) and the full-strength bottom border kept as-is — those carry the type-coding signal.

### Code change

`src/lib/components/Inspector360.svelte` — single-line CSS:

```diff
 .i360-col-header {
     background:
         linear-gradient(180deg,
-            color-mix(in srgb, var(--col-color, currentColor) 10%, transparent),
+            color-mix(in srgb, var(--col-color, currentColor) 5%, transparent),
             var(--background-primary-alt) 90%);
 }
```

### Verification

- `cargo check`: not re-run.
- `node node_modules/svelte-check/dist/src/index.js --tsconfig ./tsconfig.json --threshold error`: not re-run for a 1-line CSS change; structurally identical AST.
- Release build: pending.

### SO #6

Orientation **v1.20** created alongside v1.19. Single callout for the one-liner.

### Pending after §117

- Boss re-test Check 2 specifically: does the column-header text now read clearly against the 5 %-tinted background?
- If still too tinted, drop to 3 % or 0 (text colour + bottom border alone carry the type signal).
- After Check 2 settles, all of Stage 1 + Stage 2 + Verifications A and B are closed.
- Stage 3 plan (matrix interpretation) — TBD.

---

## §118 — Sky View inspect-mode lockout recovery

**Bug Boss reported (2026-05-01)**: In Sky View, click a node → app opens that note as a tab. Close the tab via its own × button (instead of the dismiss pill). Result: every panel locked. Sidebar toggle buttons no longer open the file tree or any other panel. Editor area shows the empty "Select a note from the sidebar" state. Only recovery is restarting the app.

### Root cause

`handleSkyNodeClick` ([+layout.svelte:3429](src/routes/+layout.svelte:3429)) does three things when entering inspect mode:
1. Snapshots the current sidebar state via `pushSidebars('skyInspect', ...)`.
2. Hides both sidebars: `sidebarOpen = false; rightSidebarOpen = false;`.
3. Sets `skyViewInspectMode = true`.

The intended exit is the "Return to Sky View" pill at [+layout.svelte:4439-4453](src/routes/+layout.svelte:4439). Clicking its body returns to Sky View; clicking the `×` dismiss button calls `popSidebars('skyInspect')` and `skyViewInspectMode = false` — restoring the pre-SV sidebar layout.

**The trap**: that pill only renders while `$activeTab?.path` is truthy (`{:else if skyViewInspectMode && $activeTab?.path && ...}`). And the global sidebar toggles are guarded by `!skyViewInspectMode` ([+layout.svelte:1660-1661](src/routes/+layout.svelte:1660)).

If the user closes the tab via its own × button (which calls `closeTab()` in `store.ts:540`), `$activeTabId` becomes `null` (when the closed tab was the only active one). The pill disappears with the tab. `skyViewInspectMode` stays `true`. Sidebar toggles ignore clicks. Lockout.

### Fix

Single `$effect` added in [+layout.svelte:586-590](src/routes/+layout.svelte:586), right after the existing sticky-flag effects (`mapEverOpened`, etc):

```js
$effect(() => {
    if (skyViewInspectMode && $activeTabId === null) {
        popSidebars('skyInspect');
        skyViewInspectMode = false;
    }
});
```

When the active tab goes null mid-inspect, mirror the dismiss pill's cleanup. Both reactive reads (`skyViewInspectMode` and `$activeTabId`) are dependencies; on the next change, the effect re-runs with `skyViewInspectMode === false` and the if-condition fails, so no infinite loop.

The dismiss pill itself is unchanged for users who use the intended exit path.

### Verification

- `cargo check`: not re-run (no Rust change).
- `node node_modules/svelte-check/dist/src/index.js --tsconfig ./tsconfig.json --threshold error`: 1 error (pre-existing `store.ts:1850`, out of scope). Zero new errors in `+layout.svelte`.
- Release build: pending.

### SO #6

Orientation **v1.21** created alongside v1.20. Bug + root cause + fix described inline so a future reader can match the symptom (locked sidebars after closing an SV-opened tab) to the fix without trawling git.

### Pending after §118

- Boss reproduces the original lockout sequence on the §118 binary, then confirms tab-close-via-× now exits inspect mode cleanly.
- Stage 3 of the matrix tutorial — moving from "matrix renders / matrix reads" to "matrix interpretation" — once this bug is closed.
- MIG-006 §3 redo (queued).
- CE Phase 9 Path B / MIG-010 scale (queued after MIG-006 §3).
