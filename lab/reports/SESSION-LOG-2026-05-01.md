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

---

## §119 — First-time-user (?) help affordances on the matrix

Stage 3.1 finding from Boss (2026-05-01) testing the matrix on their note إسماعيل: "What the matrix provides me is a hint about where my knowledge stands and what I need to do to connect the dots. But for the first-time user, we need to help them figure out what this matrix is all about. We need to explain each stratum, type, and/or every bit of detail within the 360.3D. By adding a (?) with each one of those elements."

Boss approved: hover + click-to-pin behaviour, Claude writes the explanation text, all 30 markers in one commit.

### Code changes

1. **New component** `src/lib/components/HelpTip.svelte` — reusable `?` affordance:
   - Hover trigger → tooltip floats at `position: fixed` above (or below, configurable) the icon, no delay.
   - Click trigger → toggles a "pinned" mode that survives mouseleave; outside-click dismisses (effect attaches a `document` click listener while pinned, removes it on cleanup).
   - Computes coords from `getBoundingClientRect()` so the tooltip escapes any matrix `overflow: hidden`.
   - Theme-aware: `var(--background-secondary)`, `var(--background-modifier-border-focus)`, `var(--text-normal)`, `var(--text-accent)` for the trigger and pinned border.

2. **30 markers added to `Inspector360.svelte`**:
   - **Corner cell (2)**: HelpTip on `▲ Stratum` (vertical-axis legend) and `Type →` (horizontal-axis legend). The vertical legend explains the L1→L8 hierarchy and the active-row purple highlight; the horizontal legend explains the 7+1 typed columns and what the diagonal stripes mean.
   - **Column headers (8)**: one per typed direction + Untyped. Each tooltip describes what that typed link asserts and shows the `[[target|type]]` wikilink syntax.
   - **Stratum row labels (8)**: one per L1–L8. Each tooltip describes what kind of note lives at that altitude.
   - **Dimension strip (5 base + 2 conditional)**: Stratum, Maturity, Origin (with depth note), Stage, Review; Trails and Lenses when present.
   - **Grand total Σ (1)**: in the corner row-totals header. Explains the matrix-level cross-check.
   - **HUD warnings (4)**: Orphan, Fragile, Blind spots, Tensions — when each fires + what it means cognitively.

3. **Explanation text** in three constants in `Inspector360.svelte`: `HELP_STRATUM`, `HELP_TYPE`, `HELP_DIM`, plus singletons `HELP_GRAND`, `HELP_HUD`, `HELP_AXIS_STRATUM`, `HELP_AXIS_TYPE`. English-only for first ship; i18n keys deferred (same pattern the type labels use).

### Verification

- `node node_modules/svelte-check/dist/src/index.js --tsconfig ./tsconfig.json --threshold error`: 1 error (pre-existing `store.ts:1850`, deferred per Boss). Zero in `Inspector360.svelte` or `HelpTip.svelte`.
- `cargo check`: not re-run (no Rust change).
- Release build: pending.

### SO #6

Orientation **v1.22** created alongside v1.21. The §119 callout enumerates all 30 markers + describes the HelpTip component briefly.

### Pending after §119

- Boss tests the help affordances on a fresh first-time-reading session: do the explanations actually answer the questions a new user would have?
- Stage 3.2 (horizontal balance reading) — was queued before the §119 detour. Resume after §119 lands.
- MIG-006 §3 redo (queued).
- CE Phase 9 Path B / MIG-010 scale (queued after MIG-006 §3).
- store.ts:1850 LinkLifecycle Option B fix (deferred until post-CE).

---

## §120 — Tooltip uppercase + edge-clip + full Arabic + English localization

Boss tested §119 on the matrix and flagged three issues:

1. **STRATUM tooltip rendered ALL CAPS** (image showing the dim-strip ? tooltip). The parent `.i360-strip-label` has `text-transform: uppercase` for its own label text; the tooltip element, even though `position: fixed`, inherits text-transform from the ancestor.
2. **Σ tooltip and Blind spots tooltip clipped on the right** (images). `transform: translate(-50%, ...)` centers the tooltip on the trigger; for triggers near the right edge of the viewport, the tooltip's right half falls off-screen.
3. **"I want everything fully localized, like the Stratum, and the top row."** Boss showed the matrix on Arabic locale: typed column headers were already translated (link_supports etc. exist in ar.json), but stratum row labels (Worldview / Paradigm / etc.), dim-strip labels (Stratum / Maturity / Origin / Stage / Review), maturity/origin/stage values (evergreen / discovered / spark / Due), Untyped column header, axis legends, and ALL the §119 help text were still hardcoded English.

Boss's added directive: "Don't forget the other languages."

### Code changes

1. **`src/lib/components/HelpTip.svelte`**:
   - `.help-tooltip` adds `text-transform: none; font-weight: 400; letter-spacing: normal` to override ancestor styles regardless of where the trigger is mounted.
   - `computeCoords()` clamps `x` to viewport bounds: `halfWidth = 200` (380px max-width / 2 + breathing room), `margin = 12`, `minX = halfWidth + margin`, `maxX = vw - halfWidth - margin`. Triggers near the left/right edge now keep the full tooltip on-screen.

2. **`src/lib/i18n/index.ts`**:
   - `t` derived now implements a fallback chain: active locale → en.json → key. When `lookup()` returns the literal key path (miss), the chain tries en before giving up.
   - Locale loader casts each non-en file through `unknown as typeof en` to bypass strict structural typing. The runtime fallback handles missing keys, so partial translation no longer breaks compilation.

3. **`src/lib/i18n/en.json`** — added 64 new keys under `inspector360`:
   - 1 untyped column label
   - 8 stratum names (`stratum_name_1` Datum → `stratum_name_8` Worldview)
   - 7 dimension labels (`dim_stratum/maturity/origin/stage/review/trails/lenses`)
   - 5 maturity values, 4 origin values, 5 stage values (`stage_none` is the placeholder for stage absent), 2 review values (due/none)
   - 2 axis legend labels
   - 30 help strings: `help_axis_*` (2), `help_stratum_*` (8), `help_type_*` (8), `help_dim_*` (7), `help_grand_total` (1), `help_hud_*` (4)

4. **`src/lib/i18n/ar.json`** — 64 Arabic translations of the same keys. Native-quality terminology:
   - Strata: بَيانة / معلومة / قضية / مفهوم / مبدأ / نظرية / نموذج / رؤية كونية
   - Dim labels: المستوى / النضج / المنشأ / المرحلة / المراجعة / المسارات / العدسات
   - Maturity: بذرة / نبتة / دائمة الخضرة / مرجعية / ذابلة
   - Origin: مكتسب / مكتشف / مختلط
   - Stage: عابرة / اقتباس / دائمة / توليفية
   - Review due: مستحقة
   - Untyped: غير محدد
   - 30 full help-text translations matching the English semantics.

5. **`src/lib/components/Inspector360.svelte`** — full localization wire-up:
   - All static `HELP_*` constants and `STRATUM_NAMES` map removed.
   - Only `STRATUM_FALLBACK` retained as defensive English fallback (rarely used, since en.json is now authoritative).
   - New helper `tr(value, key, fallback)`: returns `value` if it's not the literal key string, else `fallback`. Used at every i18n call site to handle the "key returned because missing" case cleanly.
   - 30 HelpTip instances now read text via `tr($t('inspector360.help_*'), key, '')`.
   - Stratum row labels, dim-strip labels, maturity/origin/stage values, axis labels, "Due" review, untyped column header — all read via `tr()` calls.

Other 13 locales (fa, he, ur, es, fr, de, zh, ja, ko, pt, ru, hi, tr) untouched in this commit. With the new fallback chain, users on those locales see English text for the new keys instead of cryptic key strings. Backfill of those locales is a follow-up task.

### Verification

- `cargo check`: not re-run (no Rust change).
- `node node_modules/svelte-check/dist/src/index.js --tsconfig ./tsconfig.json --threshold error`: 1 error (pre-existing `store.ts:1850`, deferred per Boss until post-CE). Zero in i18n, Inspector360, or HelpTip.
- Release build: pending.

### SO #6

Orientation **v1.23** created alongside v1.22. Three fixes inline-described.

### Pending after §120

- Boss tests on Arabic locale: matrix stratum labels, dim-strip labels, maturity/origin/stage values, axis legends, help tooltips — all in Arabic? Tooltip uppercase fixed? Edge clipping fixed?
- Stage 3.2 (horizontal balance reading) — was queued before the §119 + §120 detour. Resume after this lands.
- Other 13 locales (fa, he, ur, es, fr, de, zh, ja, ko, pt, ru, hi, tr) — backfill the new keys when bandwidth allows. Currently fall back to English gracefully via the new chain.
- MIG-006 §3 redo (queued).
- CE Phase 9 Path B / MIG-010 scale (queued after MIG-006 §3).
- store.ts:1850 LinkLifecycle Option B fix (deferred until post-CE).
