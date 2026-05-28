# MIG-060 — Constellation Base Phase 1.5: Host-Note Threading Gestures (Architect)

**Date:** 2026-05-28
**MIG class:** UI + cross-subsystem wiring (lens block → deep-read surfaces)
**Status:** Architect — open questions locked TBD by Eisa

## What this MIG ships

Per Concept Paper v1.4 §7.5 and orientation v2.36's roadmap, Phase 1.5 adds **threading gestures from every Bases (lens-block) row** to the three deep-read surfaces:

| Gesture | Routes to | What it shows | Internal flag |
|---|---|---|---|
| Open in **360.3D** | Per-note cognitive standing | Stratification Matrix — Position / Connection Profile / Absence | `showInspector360` |
| Open in **CNS** | Per-note network neighborhood | Universe Health, Communities, Top Bridges, Blind Spots | `lensActive` (the historical name for the surface; user-facing is "CNS" or "Constellation Sight" depending on locale) |
| Open in **The Cataloger** | Per-note epistemic classification | Source × Content-type card, per-cataloger reasoning trail, disambiguation chips | `showCataloger` |

Plus the existing default click-to-open-host-note gesture (which already works via `constellation:open-note`).

**The four-surface workflow Constellation uniquely enables:**
- Bases = comparison of many notes (surveying)
- 360.3D = cognitive depth of one note
- CNS = network depth of one note
- The Cataloger = epistemic depth of one note

No other PKM has all four. No other PKM threads them together with single-click row gestures.

## Territory

### Current state (verified by code read)

**`src/lib/editor/livePreview.ts::LensBlockWidget._renderRow`** (line 832): each lens row is a `<li class="cm-lens-row">` containing:
- `<button class="cm-lens-row-name">` (clicking dispatches `constellation:open-note` → opens host note in active pane)
- Optional `<span class="cm-lens-row-sep">—</span>` + `<span class="cm-lens-row-headline">` (when `note.headline` is in the lens dimensions)

**`src/routes/+layout.svelte`** owns the three surfaces:
- `showInspector360: $state(false)` — full-page 360.3D view, lazy-mount (`inspector360EverOpened` sticky), `inspector360Data` populated from `sidebarTab.path`. Dock-button to toggle.
- `lensActive: $state(false)` — full-page CNS view (`ConstellationSight2.svelte`). Dock-button gated by `SIGHT_V2_ENABLED && enabledFeatures.constellationSight !== false`.
- `showCataloger: $state(false)` — full-page Cataloger view (`CatalogerView.svelte`), lazy-mount. Dock-button to toggle.

### Cross-cutting pattern: "open this note in surface X"

The existing pattern (host-note open): `window.dispatchEvent(new CustomEvent('constellation:open-note', { detail: { path, libraryName, libraryPath } }))`. The layout listens, opens the note in the active pane, the note becomes the active `sidebarTab`. The three surfaces read from `sidebarTab` to know which note to display.

So "open note X in surface Y" is mechanically:
1. Dispatch `constellation:open-note` for path X → note becomes active
2. Set surface Y's flag to `true` → surface renders for the active note

Two events, or one custom event with a `targetSurface` field. Design choice (locked in this Architect).

## Invariants (must not break)

1. **Default click still opens the host note.** The existing `cm-lens-row-name` button must continue to dispatch `constellation:open-note` and open the host note. The three new gestures are ADDITIVE.
2. **RTL parity.** Lens rows render with `dir=detectDir(row.name)`. New gestures must inherit the same RTL handling so Arabic rows align correctly.
3. **CNS surface gating.** When `enabledFeatures.constellationSight === false`, the "Open in CNS" gesture must either be hidden or disabled (not just dispatch the event and silently fail). User-visible discoverability follows the user's settings.
4. **No new keystroke-path costs.** Lens rows render at scroll time / on lens-result arrival. The new gestures add three DOM elements per row (or one trigger element); render budget per row remains low (< 1ms per row total).
5. **Performance: lens with N rows.** A typical Five Acts lens caps at 30-50 rows visible. Adding three buttons per row = ~150 extra DOM nodes max. Trivial.
6. **Existing four-pane / split / second-screen behaviors continue to work.** Opening a note then toggling a surface is the same flow as today's dock-button-then-active-note pattern; no new edge cases for split panes.

## Open design questions (need Eisa lock)

### Q1 — UI affordance: how does the user invoke the three gestures per row?

**Option A: Three icon buttons appended to the row** (always visible, right side after the headline).

```
[ note name ] — headline  [360] [CNS] [Cataloger]
```

Pros: discoverable on first sight, one click per gesture, no hover/menu interaction.
Cons: visual clutter on dense lens lists; three icons × N rows might feel busy.

**Option B: Single "actions" trigger (⋮ or ⋯) that opens a popup menu** with the three options.

```
[ note name ] — headline  ⋯
                          └→ [Open in 360.3D]
                              [Open in CNS]
                              [Open in Cataloger]
```

Pros: clean rest state, all interaction localized to one icon.
Cons: two clicks per gesture, less discoverable, popup positioning needs care in RTL contexts.

**Option C: Icons appear ON HOVER only** (right side of the row).

Pros: clean rest state, one click on hover.
Cons: doesn't work on touch / accessibility issue for keyboard users; users may not discover them.

**Option D: Right-click context menu only** (no visible affordance; users right-click the row).

Pros: zero visual change to the row.
Cons: completely invisible; users have to know the gesture exists; conflicts with browser context menu unless preventDefault'd.

**Recommendation (mine): Option A.** Constellation is a power-user tool; visual density is acceptable; discoverability + one-click matters most. Icons can be small (12px) and color-coded with subtle hue cues (360.3D = blue-violet, CNS = teal, Cataloger = amber, mirroring their dock-button colors if any).

### Q2 — Event mechanism: one custom event with `targetSurface`, or three distinct events?

**Option A: One generic event** `constellation:open-note-in-surface` with `detail.surface: '360.3d' | 'cns' | 'cataloger'`.

Pros: one listener, easy to extend (Phase 1.5 might add more surfaces later).
Cons: string union in the detail; type safety relies on the listener pattern-matching.

**Option B: Three distinct events** `constellation:open-note-in-360`, `constellation:open-note-in-cns`, `constellation:open-note-in-cataloger`.

Pros: explicit; grep-friendly; each event has its own handler.
Cons: three events to maintain.

**Recommendation (mine): Option A.** The string union is fine for an internal contract; extending to a Phase 2 "Open in Knowledge Health" or "Open in Sky View" is trivial.

### Q3 — When the user clicks a gesture, does the host note's NotePane also open?

**Option A: Yes — open the host note in the active pane FIRST, then toggle the surface.** This is the "stay grounded in the note" pattern.

Pros: the user has the note open for reference while exploring the surface; matches today's behavior where surfaces operate on the active sidebarTab.
Cons: more state churn per gesture click (open tab + toggle surface).

**Option B: No — only toggle the surface, leave the active pane alone.** Surface targets the lens row's note as a one-shot view.

Pros: less state churn; cleaner "I want to see THIS note in THIS surface" mental model.
Cons: requires each surface to accept a "target note path" prop independent of `sidebarTab`. That's a refactor of each of the three surfaces.

**Recommendation (mine): Option A.** The three surfaces are already wired to read from `sidebarTab`; we get correct behavior by just opening the note first. Option B is a larger refactor for marginal UX gain.

### Q4 — CNS gating: hide or disable when `enabledFeatures.constellationSight === false`?

**Option A: Hide the "Open in CNS" gesture** when CNS is disabled in settings.

Pros: rows show only working gestures.
Cons: inconsistent rendering between users with/without CNS enabled.

**Option B: Show but disable** the gesture (greyed out, tooltip explains why).

Pros: consistent layout; user can see the feature exists and learn to enable it.
Cons: visual noise for users who'll never enable CNS.

**Recommendation (mine): Option A.** The legacy Sight/CNS gate is documented as user-toggleable; respecting the toggle by hiding is consistent with the rest of the dock (the CNS dock-button is hidden when disabled too).

### Q5 — i18n: what are the localized labels for the three gestures (15 locales)?

Need new i18n keys:
- `lensBlock.openIn360` (e.g. "Open in 360.3D" / "افتح في 360.3D")
- `lensBlock.openInCNS` (e.g. "Open in CNS" / "افتح في CNS")
- `lensBlock.openInCataloger` (e.g. "Open in Cataloger" / "افتح في المُصنِّف")

Question: are these tooltip labels (Option A icons) or menu item labels (Option B menu)? Either way, 15 locales × 3 strings = 45 new i18n entries.

**Recommendation (mine):** Use the existing localized names where they already exist in i18n. For Arabic: 360.3D → "360.3D"; CNS → already has localization; Cataloger → "المُصنِّف" per memory `feedback_full_localization_everything.md`.

## Proposed plan structure (assuming all my recommendations lock)

Phase-by-phase steps for the Plan doc (subject to Eisa's adjustments):

- **§A** — i18n entries (45 strings × 15 locales). Mechanical.
- **§B** — `LensBlockWidget._renderRow` extension: add three icon buttons after the headline, dispatch the generic `constellation:open-note-in-surface` event. RTL-aware positioning.
- **§C** — `+layout.svelte` listener: on `constellation:open-note-in-surface`, run the existing `constellation:open-note` flow first (open host note in active pane), then `await tick()`, then toggle the target surface flag.
- **§D** — CNS gating: lens row only renders the CNS gesture when `enabledFeatures.constellationSight !== false`.
- **§E** — CSS for the three icons (size, color hint, RTL positioning).
- **§F** — Behavioral test fixtures (synthetic lens, click each gesture, assert correct surface opens).
- **§G** — Boss-test gate (Eisa's tutorial).

Each step lands as one commit with a verification clause.

## Locks (Eisa, 2026-05-28)

| # | Question | Lock |
|---|---|---|
| Q1 | UI affordance | **A — Three icons, always visible.** Small (12px) icons appended to the right of each lens row. One click per gesture. |
| Q2 | Event mechanism | **A — One generic event** `constellation:open-note-in-surface` with `detail.surface: '360.3d' \| 'cns' \| 'cataloger'`. Extensible for Phase 2 surfaces. |
| Q3 | NotePane navigation | **A — Open host note + toggle surface.** Click dispatches `constellation:open-note` first (note becomes active in pane), then toggles the surface flag. Surfaces read from active `sidebarTab` so no surface refactoring needed. |
| Q4 | CNS gating | **A — Hide the gesture entirely** when `enabledFeatures.constellationSight === false`. Consistent with the dock-button which is also hidden in that state. |
| Q5 | i18n labels | Accept proposed forms. Arabic: 360.3D → "360.3D"; CNS → existing localization; Cataloger → "المُصنِّف" per `feedback_full_localization_everything.md`. |

All locked with my recommendations. Cascading to Plan + Build per Plan-Approval-Equals-Build-Approval.
