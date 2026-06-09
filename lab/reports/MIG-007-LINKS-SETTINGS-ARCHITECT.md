# MIG-007 — Links Settings Tab · Architect (Phase 1)

**Date:** 2026-06-09 · **Status:** options presented, **Eisa picked Option A** (Behaviour + hub links).
**Origin:** PJ-005 / the 2026-04-27 decision ("a new Links Settings tab consolidates every link-related
control"; the Auto-update-Links toggle is misplaced under Sky View & Links). Frontend-only.

## Problem

Link-related controls are scattered across Settings, and the 2026-04-27 framing predates MIG-067/070 —
which made the **Style Setter** the home for link **types + colours**. So "consolidate *every* link control"
can no longer mean what it meant without re-opening the MIG-070/071 *single-styling-home* invariant.

## Territory (verified, file:line)

| Control | Current home | Nature |
|---|---|---|
| Auto-update links on rename (`$appSettings.autoUpdateLinks`) | Settings **'skyview'** block, `SettingsModal.svelte:~1293` | behaviour |
| Link decay on/off + half-life (`$appSettings.linkLifecycle.decayEnabled` / `.halfLifeDays`) | Settings **appearance** keys, `SettingsModal.svelte:~2012-2030` | behaviour (sort) |
| Links-panel visibility | Settings **'panels'** list, `SettingsModal.svelte:~1956` | layout |
| Link-Type editor (names · 8 acts · custom · nesting · colours) | **Style Setter** — `StyleSetter.svelte:1149` `<LinkTypesEditor embedded />` | vocabulary + styling |
| Typed-link styling (pill, toggles) | **Style Setter** | styling |
| Link lifecycle (archive · traversal · confidence) | **right-sidebar 'links' tab** → `LinkDashboard` (`+layout.svelte:6588`) | instances |

Open mechanisms: Style Setter via the global store `$lib/stores/styleSetter` (`openStyleSetter()`); Link
Dashboard via `rightSidebarTab='links'` + open the right sidebar.

## Options considered

| # | Option | Speed | Effort | Risk |
|---|---|---|---|---|
| **A ✅** | **Behaviour + hub links.** New 'links' section gathers auto-update + decay (+ a pointer to panel visibility) and deep-links to the Style-Setter type editor + the Link Dashboard. Types/colours/styling stay in the Setter. | Fast | Low (≈1 component + i18n) | **Low** — honours single-styling-home; no settings migration |
| B | Also move the type **editor** (vocabulary) into the Links tab; only colours stay in the Setter. | Med | Med | Med — splits one editor across two surfaces; partly re-opens MIG-070 |
| C | Links tab = full link home (behaviour + editor incl. colours); Setter keeps a styling view on the same registry. | Slow | High | High — two editors on one registry (single-source risk) |

**Eisa picked A** (2026-06-09).

## Invariants that must not break

- **I1 — Single-styling-home (MIG-070/071):** the Links tab renders **no styling control**; it only *links* to the Style Setter. Types/colours/pill stay in the Setter.
- **I2 — No settings renamed/lost:** moved controls keep their exact `$appSettings` paths; existing user settings render unchanged in the new section.
- **I3 — `autoUpdateLinks` cascade intact:** the rename→link-rewrite path (`+layout.svelte:~3807`, gated on `$appSettings.autoUpdateLinks`) is untouched; only the toggle's UI location moves.
- **I4 — i18n parity:** every new string in all 15 locales, English-fallback inline (`|| 'Links'`) so an untranslated locale never blanks.
- **I5 — No duplicate controls:** a moved control is *removed* from its old section, not mirrored (one UI source per setting).
- **I6 — No perf regression:** pure markup relocation + 2 hub buttons; no new `$effect`, no new `invoke()`.

## Migration / rollback concerns

Effectively none — **no schema, no persisted-state change**. The `$appSettings` keys are identical before
and after; only the rendering section changes. Rollback (revert the commits) simply renders the controls in
their old sections again — zero data loss. This is why Phase-4 Audit 4C (migration-path) is light here.

## Audit sizing (Phase 4, proportionate)

Frontend-only → run **4A Invariant** (I1–I6) + **4B Drift** (no second writer / no leftover copy of a moved
control). **4C Migration-path** reduces to "existing settings still render + rollback is clean" (no
schema/data path). Plus `/simplify` on the diff.
