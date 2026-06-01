# MIG-070 — Constellation Style Setter (CSS) — PLAN

**Status:** Plan (Phase 2 of /migration). Awaiting Boss approval. Date 2026-06-01.
**Architect:** `docs/MIG-070-constellation-style-setter-ARCHITECT.md`.

## Boss decisions (ratified 2026-06-01)
1. **Edit model = Draft, then Apply.** The Setter edits a draft scoped to a wrapper; the live workspace changes only on Apply/Save.
2. **Preview fidelity = static representative samples** for heavy surfaces (editor, Sky View, OrgChart, Index, Cataloger); chrome rendered live-mini.
3. **Replace.** The Setter becomes THE theming UI; the current Appearance → Themes grid + Style Settings tab fold into it.
4. **Unify.** Per-Universe Themes + app-global Styles (MIG-069) merge into ONE app-global, named, switchable, exportable concept that the Setter produces.

## Reuse (the engine already exists — see Architect §1)
- Token registry: `src/lib/theme/constellationStyleSettings.ts` (+ `styleSettings.ts` types).
- Live-apply: `generateStyleSettingsCSS()` + `deriveThemeVariables()` → `setProperty()`.
- Controls UI: `StyleSettingsPanel.svelte` (re-grouped by surface).
- Theme model + export/import: `ConstellationTheme` (`store.ts:3054`).
- MIG-069 plumbing: `stylePresets.ts` / `style_presets.rs` (app-global store), the card.

## The unified model (the crux of "Replace + Unify")
A **Style** (provisional name — Boss to confirm "Style" vs "Theme") is an **app-global, named look**:
`{ id, name, type: light|dark, colors{5}, styleSettingsValues{}, fonts{}, linkColors[], pillShape{}, sections-included }`
— i.e. the MIG-069 Style sections fused with the `ConstellationTheme` fields, stored app-globally (extend the MIG-069 `style-presets.json` store). Applying a Style sets the current look (reusing the live-apply path). **Open sub-decisions for Boss (Q below).**

---

## Phases (each = one landable commit + a Boss-testable verification)

### §A — Unified model + NON-DESTRUCTIVE migration  *(foundation, highest risk)*
- Define the unified `Style` shape; app-global storage (extend `style_presets.rs` / `stylePresets.ts`).
- **Migrate, additively:** pull every existing per-Universe `customThemes` entry + every existing MIG-069 Style into the unified store (de-duped by id/name). **Keep the originals untouched** until verified — no deletion in §A.
- Adapt the apply path so applying a unified Style reuses `deriveThemeVariables` + `generateStyleSettingsCSS`.
- **Verify (Boss):** every theme/style you already have (Eisa Default, Eisa Chocolate, Eisa Styles 01/02, the 6 built-ins) appears in one list and applies correctly; nothing is lost; the app looks identical after migration.

### §B — Setter shell + draft scope + Apply/Save/Revert
- New full-page Setter surface (overlay, like Sky View / Cataloger), reachable from the Appearance area.
- `.css-draft` wrapper + a `$state` draft token-store seeded from the Style being edited; one `$effect` applies the draft via `generateStyleSettingsCSS` **to the wrapper** (not `:root`).
- **Apply** (promote draft → live `:root` / active Style) · **Save** (persist as a named Style) · **Revert** (discard draft).
- **Verify (Boss):** open the Setter; change a colour → only the preview changes; Apply → the real app changes; Save → it persists as a named Style; Revert → discards cleanly; closing without Apply leaves the app untouched.

### §C — Controls grouped by surface
- Reuse `StyleSettingsPanel` control renderers; group the catalog **by surface** (Editor · File sidebar · Ribbon · Tabs · Right panels · Status bar · Sky View · …). A surface selector drives both the controls shown and the preview's active surface.
- **Verify (Boss):** picking a surface shows its controls; changing any control updates the draft preview live.

### §D — Live preview: chrome
- The sampler renders live-mini **ribbon · tabs · status bar · file sidebar · Backlinks/Outgoing panels** under `.css-draft`, with stub data (zero IPC).
- **Verify (Boss):** the chrome preview looks like the real app and reflects your draft changes instantly; differently-themed Styles look visibly different.

### §E — Surface navigation + static heavy-surface samples
- Surface switcher; **static representative** samples for the editor (PagePreview-style rendered note), Sky View (snapshot/SVG + legend), OrgChart, Index, Cataloger — each styled by the draft vars.
- **Verify (Boss):** switching to each surface shows a faithful static sample that re-colours/re-fonts with your draft.

### §F — Save/manage/export/import + card thumbnail + RETIRE old tabs *(the "Replace")*
- Unified Style management: name / rename / duplicate / delete; export / import (`.constellation-style.json` and/or a `.css` custom-property block).
- The **card thumbnail** = a small render of the sampler per saved Style (replaces MIG-069's abstract card; the Styles panel becomes the gallery the Setter feeds).
- **Retire** the old Appearance → Themes grid + Style Settings tab; the Appearance entry now opens the Setter. (Predecessor Lookup logged: the old theme picker + `StyleSettingsPanel` mount in `SettingsModal.svelte:2519`.)
- **Verify (Boss):** save/rename/dup/delete/export/import all work; the old Themes + Style Settings tabs are gone/redirected; no orphaned theming UI; nothing that used the old path is broken.

### §G — i18n + docs
- All new strings via `$t()` in **15 locales**; RTL-correct preview sample (mixed-script).
- Update help (**Appearance and Themes** → rewrite for the Setter) + User Manual + orientation (v-bump in the §F or §G commit, per SO #6).

### §H — Audit + PCS
- 3-agent audit (invariants / drift / migration-path — esp. the §A data migration: first-boot, mid-migration interrupt, rollback, a no-themes universe).
- `/simplify` the diff; milestone tag + ZIP; help/manual/orientation final pass.

---

## Invariants (from Architect §6 — must hold every phase)
1. **§A migration is non-destructive + reversible** — originals kept until verified; existing looks survive.
2. **No boot-time regression** — applying a Style stays N `setProperty` calls; no new walk.
3. **appSettings fonts + per-library `libraryAppearances` cascade preserved** (deliberate layering, no collision).
4. **Second screen stays a display** — re-applies via `notifySettingsChanged()`.
5. **Draft scope** — the live workspace is untouched until Apply/Save.
6. **Zero `invoke()` on the edit/drag path**; persistence debounced.
7. **Editor Parity / reuse** — the preview mounts real components, never re-implements them.

## Open sub-decisions for Boss (can be answered at §A)
- **Q-a (apply scope):** when you Apply/activate a Style, does it set the look **app-wide** (every Universe shows it) or **per-Universe** (just the current one, like MIG-069 today)? *Default proposal: the Style library is app-global; Apply sets the current look, and the active choice is remembered per-Universe (so different Universes can wear different Styles).*
- **Q-b (name):** call the unified concept **"Style"** (matches "Style Setter") or **"Theme"** (your word "save as a theme")? *Default proposal: "Style".*

## Effort / risk
- **§A** — Medium-High effort, **highest risk** (data migration). Gets the most care + the audit's focus.
- **§B–§E** — Medium-High (new UI + preview); low engine risk (reuse).
- **§F** — Medium (management + the Replace retirement).
- This is a **multi-session** migration; each phase lands + is Boss-tested before the next.
