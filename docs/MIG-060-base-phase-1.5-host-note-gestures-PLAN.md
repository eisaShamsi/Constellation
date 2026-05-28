# MIG-060 — Constellation Base Phase 1.5: Host-Note Threading Gestures (Plan)

**Date:** 2026-05-28
**Architect:** `docs/MIG-060-base-phase-1.5-host-note-gestures-ARCHITECT.md` (v1.0, locked)
**Plan:** v1.0 — cascades through Build per Plan-Approval-Equals-Build-Approval.

## Goal

Wire three threading gestures from every lens row to the deep-read surfaces 360.3D / CNS / Cataloger, completing the four-surface workflow Constellation uniquely enables.

## Architecture summary (from Architect locks)

- **UI:** three small icon buttons appended to each lens row (right side). Always visible. 12px icons, color-hinted per target surface.
- **Event:** single `constellation:open-note-in-surface` custom event with `detail.surface` discriminator.
- **Navigation:** click → open host note in active pane → toggle target surface flag.
- **CNS gating:** hide the CNS gesture when `enabledFeatures.constellationSight === false`.
- **i18n:** 15 locales × 3 tooltip strings (45 new entries).

## Steps

### §A — i18n: 45 new tooltip strings across 15 locales (1 commit)

Three new keys under `lensBlock.*`:
- `lensBlock.openIn360Tooltip` — e.g. "Open in 360.3D" / "افتح في 360.3D"
- `lensBlock.openInCNSTooltip` — e.g. "Open in CNS" / "افتح في نظام الأعصاب"
- `lensBlock.openInCatalogerTooltip` — e.g. "Open in The Cataloger" / "افتح في المُصنِّف"

For RTL locales (ar, fa, he, ur): use the native equivalent. For non-RTL locales: literal English with the surface name kept in English (since "360.3D" / "CNS" / "Cataloger" are brand-like internal names that don't translate cleanly across all locales, AND Eisa's `feedback_full_localization_everything.md` memo specifies "Cataloger" → "المُصنِّف" in Arabic).

**Verification clause:** `npx svelte-check` reports no missing-translation warnings; spot-check each locale's JSON for valid syntax + presence of all three keys.

### §B — LensBlockWidget._renderRow extension (1 commit)

In `src/lib/editor/livePreview.ts::LensBlockWidget._renderRow` (line 832), after appending the headline span (or after the name button if no headline), append a `<div class="cm-lens-row-actions">` container holding three icon `<button>` elements:

```typescript
const actions = document.createElement('div');
actions.className = 'cm-lens-row-actions';

// 360.3D button — always shown
const btn360 = document.createElement('button');
btn360.type = 'button';
btn360.className = 'cm-lens-row-action cm-lens-row-action-360';
btn360.innerHTML = '<svg ...>...</svg>'; // small 12px icon
btn360.title = get(t)('lensBlock.openIn360Tooltip') || 'Open in 360.3D';
btn360.addEventListener('click', (e) => {
    e.stopPropagation();
    window.dispatchEvent(new CustomEvent('constellation:open-note-in-surface', {
        detail: {
            surface: '360.3d',
            path: row.note_path,
            libraryName: row.library_name,
            libraryPath: row.library_path,
        },
    }));
});
actions.appendChild(btn360);

// CNS button — gated by enabledFeatures.constellationSight
if (get(appSettings).enabledFeatures?.constellationSight !== false) {
    // ... same pattern with surface: 'cns'
}

// Cataloger button — always shown
// ... same pattern with surface: 'cataloger'

li.appendChild(actions);
```

The `e.stopPropagation()` is critical — it prevents the click from bubbling up to the row name button's handler and double-firing.

The action container has its own dir handling for RTL: in RTL contexts, actions appear on the LEFT (visual right of the text); in LTR contexts, they appear on the RIGHT.

**Verification clause:** rebuild and visually confirm three icons appear on each lens row in the "Observation — Recent Captures" five-acts note. Click each — note opens AND surface opens.

### §C — `+layout.svelte` event listener (1 commit)

Add a listener for `constellation:open-note-in-surface` in `+layout.svelte`. The handler:

1. Dispatches `constellation:open-note` with the path+library detail (existing flow — opens the host note in the active pane, makes it the active `sidebarTab`).
2. Awaits one tick (`await tick()` from `svelte`) so the reactive cascade settles and `sidebarTab` is updated.
3. Switches on `surface`:
   - `'360.3d'`: clears other full-page flags + sets `showInspector360 = true`.
   - `'cns'`: clears other full-page flags + calls `toggleLens()` (the existing function that activates CNS / `lensActive`).
   - `'cataloger'`: clears other full-page flags + sets `showCataloger = true`.

Use the existing "clear other surfaces before opening one" pattern already established in the dock-button onclick handlers.

**Verification clause:** trigger the event from devtools (`window.dispatchEvent(new CustomEvent('constellation:open-note-in-surface', { detail: { surface: '360.3d', path: ..., libraryName: ..., libraryPath: ... } }))`) and confirm the host note opens + the surface activates.

### §D — CSS for `.cm-lens-row-actions` (1 commit)

Add CSS in `livePreview.ts`'s style injection (or a sibling location):

```css
.cm-lens-row {
    display: flex;
    align-items: center;
    gap: 4px;
}
.cm-lens-row-actions {
    margin-inline-start: auto; /* pushes to right in LTR, left in RTL */
    display: flex;
    gap: 2px;
    opacity: 0.6;
    transition: opacity 0.15s;
}
.cm-lens-row:hover .cm-lens-row-actions {
    opacity: 1;
}
.cm-lens-row-action {
    width: 18px;
    height: 18px;
    padding: 0;
    background: transparent;
    border: none;
    color: var(--color-base-50);
    cursor: pointer;
    border-radius: 3px;
}
.cm-lens-row-action:hover {
    background: var(--color-base-15);
    color: var(--color-base-90);
}
.cm-lens-row-action svg {
    width: 12px;
    height: 12px;
}
```

Subtle color-hint per surface via CSS class:
- `.cm-lens-row-action-360 { color: var(--color-violet-50); }` on hover
- `.cm-lens-row-action-cns { color: var(--color-teal-50); }` on hover
- `.cm-lens-row-action-cataloger { color: var(--color-amber-50); }` on hover

(Use existing color variables; fall back to neutral if specific hue tokens don't exist.)

**Verification clause:** rebuild; visually check that actions are right-aligned in LTR rows and visually-right (left in DOM) in RTL rows; hover changes opacity and per-surface hue.

### §E — Behavioral unit tests (1 commit)

Add tests in `lens::tests` (or new module `lens::host_note_gestures::tests`):

1. **Event payload shape** — when the lens row renders, the click handler dispatches a CustomEvent with `detail.surface ∈ {'360.3d', 'cns', 'cataloger'}` + the correct `path`/`libraryName`/`libraryPath`.
2. **CNS gating** — when `enabledFeatures.constellationSight === false`, the CNS button is not rendered (other two still render).
3. **i18n tooltip presence** — for each of the 15 locales, the three new keys resolve to non-empty strings.

These are pure-shape tests; no SQLite or universe setup needed.

**Verification clause:** `cargo test --lib lens::host_note_gestures` (or equivalent) returns N passed.

### §F — Boss-test gate (Eisa's tutorial) (1 commit)

Write `docs/MIG-060-BOSS-TEST.md` with three stages:

1. **Visual** — open "Observation — Recent Captures" lens; confirm three small icons appear on each row.
2. **360.3D gesture** — click the leftmost icon (or rightmost in RTL); confirm note opens + 360.3D Inspector view activates.
3. **CNS gesture** — only if Eisa has CNS enabled in settings. Click the CNS icon; confirm CNS opens for the clicked note.
4. **Cataloger gesture** — click the Cataloger icon; confirm Cataloger opens for the clicked note.
5. **RTL parity** — confirm icons are positioned correctly for Arabic-named notes.

### §G — PCS (orientation v2.40 + MoCh + 15-locale help-doc + milestone tag) (1 commit)

After Eisa verifies §F:
- Orientation v2.40 captures MIG-060 in the preamble.
- New MoCh entry for the design conversation + cascade.
- Help docs (English + 15 locales) get a "Threading Gestures from Lens Rows" section in the Lens / Bases help file.
- Milestone tag `milestone/mig-060-base-phase-1.5-shipped`.
- ZIP backup.

## Verification (cumulative across all steps)

After §G:
- 840+ lib tests pass (no regression).
- `svelte-check` no new errors.
- Boss-test verified all four pass criteria.
- Eisa can open any lens row's host note in any of the three surfaces with one click.
- The "four-surface workflow" claim in marketing is now true in product.

## Risks (catalogued for Audit phase)

- **R1: CodeMirror widget render performance with three new DOM elements per row.** Mitigated by §D's CSS (display: flex; static layout; no transitions on render). Lens lists cap at ~30 rows so DOM node count rises by ~90.
- **R2: RTL layout edge cases.** Mitigated by `margin-inline-start: auto` (logical property auto-flips); ensure devtools check both Eisa Universe (mixed) and an Arabic-only lens.
- **R3: Event listener leak in `+layout.svelte`.** Existing `constellation:open-note` listener is mounted at component root; same pattern for the new event. Add cleanup in `onDestroy` (matches Rule 4 — Memory Leaks).
- **R4: `tick()` await race during fast gesture clicks.** If user clicks gesture for note A then quickly clicks gesture for note B, the awaits could interleave. Treat as single-user single-click flow; out of scope.

## Out of scope (deferred)

- **Phase 2** — Living Link Columns. Separate MIG.
- **Phase 2.5+** — Bridges (360.3D / CNS / Cataloger as lens DIMENSIONS, not just gestures). Separate MIGs.
- **Additional surfaces** — Phase 1.5 ships exactly 360.3D + CNS + Cataloger. Adding "Open in Index" or "Open in Sky View" is a Phase 8+ scope.

## Approval

Once Eisa approves this Plan, Build cascades through §A → §G with verification stops at §B, §C (per-step rebuild + visual check) and §F (Boss-test).
