# MIG-007 — Links Settings Tab · Plan (Phase 2)

**Scope:** Option A (Behaviour + hub links). Frontend-only; no Rust, no schema, no persisted-state change.
**Architect:** `lab/reports/MIG-007-LINKS-SETTINGS-ARCHITECT.md`. Each step = one `§N` commit + a verification
clause. **Boss-Test Gate after §4** (functional whole), localization spot-check after §5.

**Predecessor → Replacement** (Predecessor Lookup Rule — relocation approved by Eisa 2026-06-09 via the
scope pick; default-same-place overridden to the new 'links' section):

| Control | Predecessor | Replacement | Binding (UNCHANGED) |
|---|---|---|---|
| Auto-update links on rename | Settings 'skyview' (`~1293`) | Settings **'links'** | `$appSettings.autoUpdateLinks` |
| Link decay on/off + half-life | Settings appearance (`~2012-2030`) | Settings **'links'** | `$appSettings.linkLifecycle.decayEnabled` / `.halfLifeDays` |
| Links-panel visibility | Settings 'panels' (`~1956`) | **STAYS** in 'panels' (uniform list) + a pointer line in 'links' | (unchanged) |
| Link-Type editor | Style Setter (`StyleSetter.svelte:1149`) | **STAYS**; hub deep-link from 'links' | — |
| Link Dashboard | right-sidebar 'links' tab (`+layout:6588`) | **STAYS**; hub deep-link from 'links' | — |

---

## Steps

### §1 — Scaffold the 'links' section (empty + intro)
- `SettingsModal.svelte`: add `{ id: 'links', label: $t('settings.sections.links') || 'Links', icon: 'link' }` to the `sections` `$derived` array — placed **right after 'skyview'** (adjacency: auto-update is leaving "Sky View & Links").
- Add the `{:else if activeSection === 'links'}` body with a `section-intro` paragraph.
- **Verify:** Settings opens; a new "Links" tab appears after Sky View; clicking it shows the intro; all other sections unaffected; no console errors.

### §2 — Relocate the auto-update-links toggle (skyview → links)
- Move the toggle markup (`~1293-1298`) out of the 'skyview' block into 'links'. Keep the exact binding (`$appSettings.autoUpdateLinks` + `updateSettings({autoUpdateLinks})`).
- **Verify (I3):** toggle now in Links, gone from Sky View; toggling persists; renaming a note still cascades `[[Old]]→[[New]]` when ON and does not when OFF (cascade at `+layout:~3807` unchanged).

### §3 — Relocate the link-decay settings (appearance → links)
- Move `decayEnabled` + `halfLifeDays` controls (`~2012-2030`) from the appearance block into 'links'. Keep the exact `$appSettings.linkLifecycle.*` bindings.
- **Verify:** decay controls now in Links, gone from Appearance; changing half-life still re-sorts Backlinks / Outgoing by effective weight; raw counts untouched.

### §4 — Hub deep-links + panel pointer
- In the 'links' block add a "Related" subsection:
  - **"Edit link types & colours →"** → `import { openStyleSetter } from '$lib/stores/styleSetter'`; on click: `openStyleSetter()` + close the Settings modal. *(Sub-step §4a, optional: add a `styleSetterScrollTarget` writable mirroring `styleSetterInspectRequest` so the Setter scrolls to the Link-Types editor on open. If it adds risk, ship §4 without it and the Setter just opens at top.)*
  - **"Open Link Dashboard →"** → close Settings + set `rightSidebarTab='links'` and open the right sidebar. Mechanism: a callback/event prop from `SettingsModal` to `+layout` (confirm SettingsModal's existing close/callback surface during build; reuse it).
  - A one-line pointer: "Show/hide the Links panel in **Panels**."
- **Verify:** "Edit link types" closes Settings and opens the Style Setter (at the Link-Types editor if §4a shipped); "Open Link Dashboard" closes Settings and opens the right-sidebar Links tab.

### §4 → BOSS-TEST GATE (functional whole, English)
Tutorial-style test: open Settings → Links; confirm the moved controls work; confirm the two hub buttons navigate. Articulated per the Testing Instructions Rule when sent.

### §5 — i18n sweep (15 locales)
- Add new keys to all 15 `src/lib/i18n/*.json`: `settings.sections.links`, `settings.links.intro`, the two hub-button labels, the panel pointer. (Moved controls keep their existing keys — `settings.files.autoUpdateLinks`, `settings.appearance.decayEnabled/halfLifeDays`; consider aliasing to `settings.links.*` later, non-blocking.) English-fallback inline so a lagging locale never blanks.
- **Verify:** switch to ar / zh → the Links tab, intro, and hub buttons render translated; RTL (ar) lays out correctly.

### §6 — /simplify + Phase-4 Audit
- `/simplify` on the §1–§5 diff.
- **Audit (proportionate):** 4A Invariant (I1–I6 — esp. I1 no styling control in Links; I2 bindings unchanged); 4B Drift (no moved control left duplicated in its old section; no second writer); 4C light (existing settings still render; rollback clean).

## Rollback
Revert §1–§5 → controls render in their original sections again. No data/schema change, so nothing to undo beyond the UI.

## Risks & mitigations
- **R1 (I1 regression):** accidentally rendering a colour/pill control in Links. *Mitigation:* Links holds only behaviour + hub links; §6/4A checks.
- **R2 (lost binding):** a moved control silently rebinds/typos its `$appSettings` path. *Mitigation:* copy bindings verbatim; §2/§3 verify persistence + behaviour.
- **R3 (SettingsModal→+layout wiring):** the Link-Dashboard deep-link needs a parent action. *Mitigation:* reuse SettingsModal's existing close/callback surface (confirm in build); if absent, the Style-Setter link (global store, no wiring) ships first.
