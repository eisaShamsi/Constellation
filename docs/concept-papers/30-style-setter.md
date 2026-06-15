# 30 — Style Setter (Concept Paper)

> Serves [00-Constellation](00-Constellation-Core-Concept-Paper.md). A satellite of the **Editor gate** ([01](01-Note-Editor.md)): it restyles the surfaces knowledge is read/written in, but never owns content. Boss-named **Style Setter** (MIG-070; the "full-center-zone preview" rule, CLAUDE.md).

## 1. Function in hand
The **Constellation Style Setter (CSS)** — `src/lib/components/StyleSetter.svelte`, a full-page "design studio" mounted once at the top level (`src/routes/+layout.svelte`), driven by the `styleSetterOpen` store (`src/lib/stores/styleSetter.ts`). Your real interface sits in the centre; click any part to style it; edits go to a draft, **Keep** persists. The colour catalog + apply path live in `src/lib/theme/constellationStyleSettings.ts`; named looks reuse `src/lib/libraries/stylePresets.ts`.

## 2. Purpose
Let one person shape *how their whole knowledge environment looks* — every chrome surface, every Markdown element, Sky View, CNS — visually, by clicking the real thing and editing it, then saving the look per-Universe. It does not advance one of the Five Acts directly; it is **enabling infrastructure** — it tunes the legibility of the surfaces where Observation and Synthesis happen (a serif body, a calmer accent, a larger heading make reading and writing better). Honest test (§8 of the core): it serves the Acts *indirectly*, by making the Editor and its satellites more readable — it must justify itself on that ground, not pretend to be a knowledge Act. It exists because *File-Over-App* keeps look out of the `.md` file: appearance is a per-Universe preference, never note content.

## 3. What it is NOT
- **Not** a theme engine — the old MIG-069/071 theme subsystem is gone; the Setter is the single styling path, built clean to avoid the old "gallery of heavy self-portrait cards" main-thread freeze (LL-032).
- **Not** a content surface — it never reads or writes a note's body; it writes only CSS-variable overrides.
- **Not** a per-note or per-file setting — the look is **per-Universe** (`appSettings.styleOverride`), applied app-wide.
- **Not** a second writer of `<body>` vars — `+layout`'s single `$effect` is the only writer (the BUG-015 guard).

## 4. Wiring
- **Inputs:** `styleSetterOpen` / `styleSetterInspectRequest` / `styleSetterCategoryRequest` stores; `appSettings.styleOverride` (seeded into the draft on open); `systemFonts` (installed fonts); saved Styles via `loadStylePresets()`. Live values read from `getComputedStyle(document.body)`.
- **Outputs:** `mergeStyleOverride()` / `clearAllStyleOverride()` (per-Universe save on **Keep**/**Reset**); `setLiveStyleDraft()` / `clearLiveStyleDraft()` (transient live layer while open on a 2-zone category); `updateSettings()` for settings-backed controls (font sizes, link-pill shape, typed-link toggles, per-script fonts); `saveStylePresets()` for named Styles. **No `invoke()` / no IPC** — pure frontend CSS-var writes.
- **Consumers:** `+layout`'s style `$effect` (writes the vars to `<body>`), and therefore every surface that reads those vars — Editor/NotePane (`livePreviewTheme`), file tree, chrome, Sky View (`skyPalette.ts`), CNS, second screen, Links panels.
- **Connection to the Editor (the gate):** it is a **display-only satellite**. It changes how the Editor's surface looks (CSS vars consumed by `livePreview.ts`) without touching the Editor's save/load/edit path — it adds no `write_note`, no reindex. The Editor remains the sole content authority; the Setter only re-skins the window.

## 5. Right-click / context menu
- **Has none.** Grep of `StyleSetter.svelte` for `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` finds **no menu** — only comments noting that an *accidental* right-click delete on saved-colour swatches was deliberately **replaced** by a two-step button confirm (✕ → Remove/Cancel), Eisa 2026-06-07.
- Discoverability is instead by **left-click**: click any preview part, or use **Inspect** mode (⌖) to click the real app and jump to that element's controls (`data-style-target`).
- **Gap to weigh in bring-up:** per the core §5 ("right-click should include every aspect of the app"), a saved-Style row and a swatch arguably *should* expose Apply / Rename / Delete / Export via the shared `<ContextMenu>` (MIG-077) rather than only inline icon buttons. Today those actions are inline-only; **flag**: decide in bring-up whether to add a shared context menu. If added, it MUST be the shared builder, never hand-rolled.

## 6. Multilingual
- **Mostly localized (×15) — but with hardcoded-English leaks.** ~307 control/group/category labels render through the `L()` helper → `styleSetter.labels.<slug>`, and that block is present and translated in **all 15 locales** (verified: en/ar/de/zh/fa/he each carry 307 keys; ar values are native, e.g. `"3d_gizmo": "أداة 3D"`). `L()` falls back to the English source on a key miss, so nothing renders as a raw key.
- **Hardcoded English found (flag):** the Inspect banner (`⌖ Inspecting — click a part…`), the saved-Style apply tooltip (`'Apply ' + p.name`), two empty-state lines (`Pick an element on the left…`, `Click any part of the interface to style it…`), the per-script hint line, and the **dock button** `title`/`aria-label` in `+layout.svelte` (`Style Setter — click any element…`) bypass `L()`. These must be routed through `$t()`/`L()` and added to all 15 locales before re-enable.
- **RTL:** saved-Style rows, rename inputs, and swatch-name inputs use `dir="auto"`; the per-script preview uses an explicit `dir="rtl"` Arabic sample. (Preview *sample* text — "The quick brown fox", "Heading", "نص عربي" — is intentional visual mimicry, not chrome, and is exempt.)

## 7. Boot behavior
- **Runs at boot?** The component mounts at app start (top-level in `+layout`) but renders nothing until `styleSetterOpen` is true. On mount it calls `loadStylePresets()` (saved Styles) + `ensureSystemFonts()` — lazy, post-paint, only the user's saved list. **No `invoke()`, no reindex, no Universe walk.**
- **Rule 8 status: ✅ reads-persisted.** The saved look is `appSettings.styleOverride`; `+layout`'s `$effect` reads it and assigns the vars to `<body>`. Nothing is recomputed on boot or panel-open — the persisted override IS the derived view. No `scan_*` / `rebuild_*` shape.
- **Cost:** negligible at boot (the component is inert until opened). When open, edits are CSS-var assignments on `<body>` (no layout recompute beyond the browser's own). Sky/CNS previews draw a small canvas from the draft. *Estimated* — not separately measured; mark for bring-up if it ever renders while the heavy graph is mounted.

## 8. Flag / gate & bring-up position
- **Gate today: none.** Grep finds **no** `enabledFeatures.*` / `SIGHT_*` guard around the Setter — it mounts unconditionally and is reached via the dock inspect button + Settings + the Links hub deep-link. To bring it up under the staged program it **needs a new gate** (e.g. `enabledFeatures.styleSetter`) so minimal mode can ship without it.
- **Bring-up phase:** a **late satellite** — it depends on the Editor gate (Phase 1) and on every surface whose vars it edits being present (chrome, then Sky View / CNS / Links). Bring up *after* those surfaces are re-enabled, so its inspect targets and previews are real.

## 9. Budget
- **Boot budget:** ~0 ms — inert until opened; no boot IPC. Must not add any boot-time work when gated off.
- **Interaction budget:** a control edit is one object-spread into `draft` + one CSS-var write; **no `invoke()` on any edit path** (Perf Rule 3 honoured). Live-preview layer is a single `$effect` writing tracked vars; the resize uses pointer listeners cleaned up on release. Sky/CNS canvas redraw must stay off the typing path (it isn't — it's draft-driven, on the Setter only).
- **Regression guard:** open the Setter on a 7,600-note Universe; edit accent/heading/font rapidly — the live app must not stutter; Keep/Discard/Reset must leave `<body>` with exactly the persisted vars and no orphans (the `_lastStyleSettingsKeys` cleanup). Confirm the Setter never renders `BUILTIN_THEMES` anywhere (LL-032 freeze).

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** click-to-style works; Keep persists per-Universe; Discard/Reset revert cleanly; the real app is the preview.
- [ ] **Serves Constellation's core purpose:** justified as legibility infrastructure for the Editor's surfaces (not a knowledge Act) — kept only on that honest ground.
- [ ] **Wires to the Editor correctly:** changes only CSS vars `livePreview.ts` consumes; adds **no** `write_note`/reindex; the Editor stays the sole content authority.
- [ ] **Right-click present + correct:** decide the saved-Style/swatch context-menu gap (§5) — if added, via shared `<ContextMenu>`/`buildContextMenu` (MIG-077), never hand-rolled.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** route the Inspect banner, empty-states, tooltips, per-script hint, and the `+layout` dock button through `$t()`/`L()` and translate in all 15 locales; keep `dir="auto"` on inputs.
- [ ] **Within budget:** rapid edits on a 7,600-note Universe show no live-app stutter; no `invoke()` on any edit path; no orphaned `<body>` vars after Keep/Discard/Reset.
- [ ] **Obeys Rule 8:** reads persisted `styleOverride`; recomputes no view on boot/open; never renders `BUILTIN_THEMES`.
- [ ] **Holds its invariants:** `+layout`'s `$effect` stays the single writer of `<body>` vars (BUG-015 guard); the live layer always clears on close.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—** (estimated, not separately measured)
Notes: Code is intact and clean-slate (MIG-070 §C). Three honest flags for bring-up: (1) **no gate** — needs a new `enabledFeatures` guard; (2) **partial i18n** — ~6 chrome strings hardcoded in English despite full 307-label localization; (3) **no right-click** — saved-Style/swatch actions are inline-only, decide whether the shared MIG-077 menu should cover them. Rule 8 ✅, single-writer invariant ✅, no IPC ✅.
