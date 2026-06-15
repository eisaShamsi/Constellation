# 05 — Outline Panel (Concept Paper)

> Serves [00-Constellation-Core](00-Constellation-Core-Concept-Paper.md): the Editor is the gate; every derived surface should be write-time-maintained (Rule 8); `.md` is the source of truth. Match template per [01-Note-Editor](01-Note-Editor.md).

## 1. Function in hand
The **Outline** panel — the heading list inside the **Properties** right-sidebar tab. Not a standalone component: it is inline markup in `src/routes/+layout.svelte` (~lines 6908–6923), labelled `{$t('panels.outline')}`, rendering `sidebarHeadings` which is fed by `extractHeadings()` in `src/lib/libraries/store.ts` (line 1102).

## 2. Purpose
Show the heading hierarchy (`#`…`######`) of the **active note** as a clickable, indented list, and jump the editor to a heading on click. Its one job: **navigation within a single long note** — orient and move inside the document. It serves **Observation** (see the note's own structure at a glance) and is a convenience for the Editor surface, not a knowledge operation in itself. Honest justification: it is a small, legitimate reading aid; it earns its place only because long notes exist. If a note has no headings it correctly shows "No headings" and costs nothing.

## 3. What it is NOT
- **Not** an editor of structure — clicking scrolls; it never reorders, folds, renames, or rewrites headings on disk.
- **Not** a universe-wide outline (no cross-note table of contents) — it is scoped to the one active note only.
- **Not** a derived knowledge surface (not backlinks/tags/graph) — it carries no link type, weight, or confidence; it is pure within-document navigation.

## 4. Wiring
- **Inputs:** the active note's `body` (the `sidebarTab` content), read inside the debounced sidebar `$effect` (`src/routes/+layout.svelte` ~line 1313–1315) → `sidebarHeadings = extractHeadings(body)`. Container inherits `dir={noteDir}` from `detectDir(body)`.
- **Outputs:** none persisted. On click it calls `document.getElementById(h.id)` and `scrollIntoView({ behavior:'smooth' })` — a pure DOM scroll, no IPC, no event, no write.
- **Consumers:** none. Nothing downstream depends on `sidebarHeadings`; it is a leaf view.
- **Connection to the Editor (the gate):** indirect/read-only. It reads the same `body` the Editor owns (via the active tab), and scrolls the rendered editor DOM by element `id`. It does **not** dispatch to the Editor or mutate the model — consistent with the gate being the sole content authority.

## 5. Right-click / context menu
- **None.** Each `rs-heading` row has only `onclick` (scroll-to-heading). A repo-wide grep for `oncontextmenu`/`contextmenu` in `+layout.svelte` finds matches only on library headers and tabs — **none on the outline rows or its section**.
- Shared `<ContextMenu>` (MIG-077) is **not** wired here; nothing is hand-rolled either.
- **No action is reachable only by right-click** (the only action is left-click scroll).
- **Gap (low priority):** a right-click on a heading could plausibly offer "Copy heading link", "Collapse/expand section", or "Promote/demote level" — but those touch note structure and belong to the Editor, not a read-only navigator. If any heading-level action is ever added, it must go through the shared `<ContextMenu>`, not a new hand-rolled menu. Flag for bring-up: confirm whether the Boss wants any right-click here at all; default is "none — ok" given its read-only navigation role.

## 6. Multilingual
- **Chrome strings:** both user-facing labels — `panels.outline` ("Outline") and `panels.noHeadings` ("No headings") — flow through `$t()` and exist in `en.json` (lines 887–888). Verify all 15 locales (ar de en es fa fr he hi ja ko pt ru tr ur zh) carry both keys during bring-up; no hardcoded English string was found in the outline markup.
- **Heading text:** rendered verbatim from the note body (`h.text`), so it is in whatever language the user wrote — localized by definition.
- **RTL:** the enclosing `div.rs-inner` carries `dir={noteDir}` (`detectDir(body)`), so Arabic/Hebrew outlines lay out RTL; indentation uses `padding-inline-start` (logical, flips correctly). No mixed-script font handling is applied to the rows themselves — **verify in bring-up** that long mixed-script headings render with the right per-script font (currently inherits sidebar font).

## 7. Boot behavior
- **Runs at boot?** No universe-wide work. The outline materializes only when a note is open and the Properties tab is shown; `extractHeadings` runs inside the **debounced** sidebar effect, not at app start. No IPC of its own.
- **Rule 8 status:** ⚠️ **RECOMPUTES-on-read.** `extractHeadings` re-scans the full note body with a per-line regex every time the sidebar effect fires (tab change, edit, link bump). The headings are **not** persisted and **not** maintained write-time. Scope is one note, so the cost is small — but it is technically a Rule 8 deviation, not a write-time-derived surface.
- **Cost:** estimated O(lines) per fire, sub-millisecond on a typical note (estimated — not measured). Debounced with the rest of the sidebar derivation, so it does not run per keystroke. Not a boot or typing bottleneck. **Verify-in-bring-up** the actual ms on a very large note (10k+ lines).

## 8. Flag / gate & bring-up position
- **Gate today:** none of its own. It lives inside the `properties` right-sidebar tab, which is gated only by `panelPlacements.properties === 'right-sidebar'` (default on) and an open note. No `enabledFeatures.X` / `SIGHT_*` flag. If a minimal-mode shell is built, the outline should ride along with the Properties tab — it needs **no new gate**.
- **Bring-up phase:** **1 (rides with the Properties sidebar / Editor satellites).** Depends on: the Editor (active tab + body), `detectDir`, and the `extractHeadings` helper. No backend dependency.

## 9. Budget
- **Boot budget:** zero — does nothing until a note is open.
- **Interaction budget:** outline rebuild must stay inside the existing debounced sidebar effect — **never** add an `extractHeadings` call on the keystroke hot path; click-to-scroll must be instant (it is — pure DOM, no IPC).
- **Regression guard:** open a 10k-line note, confirm typing stays instant (Rule 1) and the outline still updates after the debounce; switch tabs and confirm the outline tracks the active note with no stale rows.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** headings list matches the note; click scrolls to the right place; "No headings" shows when there are none.
- [ ] **Serves Constellation's core purpose:** stays a read-only within-note navigator (Observation aid); never silently edits structure (File-Over-App).
- [ ] **Wires correctly to the Editor:** reads the active note's body only; scroll targets resolve by `id`; no write, no IPC, no model mutation.
- [ ] **Right-click present + correct:** decision logged — either "none (ok, read-only)" or, if added, routed through the shared `<ContextMenu>` (MIG-077), never hand-rolled.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** `panels.outline` and `panels.noHeadings` present in all 15 locales; RTL outline lays out correctly; mixed-script headings render readably.
- [ ] **Within budget:** no `extractHeadings` on the keystroke path; 10k-line note stays instant; click-scroll instant.
- [ ] **Obeys Rule 8:** *currently RECOMPUTES-on-read* — either accept the deviation (single-note scope, cheap) with a written waiver, or persist headings write-time. Bring-up must rule explicitly.
- [ ] **Holds its invariants:** outline always reflects the **active** note (no cross-note bleed); empty state correct; indentation flips in RTL.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (unmeasured; estimated cheap)**
Notes: Not a separate component — inline in `+layout.svelte`'s Properties tab, fed by `extractHeadings` (`store.ts:1102`). Two open items for bring-up: (1) **Rule 8** — recomputes-on-read; decide waiver vs. persist; (2) **right-click** — none today; confirm whether any heading action is wanted and, if so, route it through the shared `<ContextMenu>`. Mixed-script font on rows is "unknown — verify in bring-up."
