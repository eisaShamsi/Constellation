# MIG-070 §C — Unify Styling: Phase-2 Implementation Plan (APPROVED PENDING)

Single styling surface = the standalone `StyleSetter.svelte` overlay. Persistence = theme base **+** per-Universe `appSettings.styleOverride`, merged in the one shared `+layout` apply `$effect`. Approach locked per `MIG-070-style-merge-AUDIT.md` §5 (A1/B1/C1/D1).

**Spine:** once **Phase 0** lands, adding an element to the Setter's `ELEMENTS` **auto-persists** — no per-element wiring. That is why Phase 0 is first.

**Higher-risk steps (write each yourself, verify before commit):** 0.3, 1.2, 6.2, 7.4, 8.1.

---

## PHASE 0 — Persistence foundation *(⚠ shared apply path; ~4 commits)*
- **0.1** Add `styleOverride: Record<string,string>` to `AppSettings` (store.ts ≈L3216) + `{}` in `DEFAULT_SETTINGS`. *Verify:* boots; key round-trips; nothing visual changes.
- **0.2** Confirm round-trip + rollback (`applyParsedSettings` L3874 spreads `...parsed`, so unknown/older key survives). *Verify:* hand-add key → survives a save cycle. **(rollback guarantee for the whole migration.)**
- **0.3** ⚠ Merge `styleOverride` ON TOP in the theme apply `$effect` (+layout L1554–1647), after derivation, registering keys into `_lastStyleSettingsKeys` (L1551). Override = last writer. *Verify [BOSS TEST]:* set one override in settings.json, relaunch, switch theme back/forth → override persists; delete key → clears.
- **0.4** Add debounced `setStyleOverride`/`clearStyleOverride` (store.ts near `updateSettings` L4060, reuse 300ms timer L4053 — no IPC hammer on 7,600 notes). *Verify:* devtools call → lands, persists, applies.

## PHASE 1 — Setter writes persistently *(⚠ behaviour change; ~3 commits)*
- **1.1** Seed Setter `draft` from `appSettings.styleOverride` on open (not blank, L166). *Verify [BOSS TEST]:* swatches reflect the live look.
- **1.2** ⚠ `apply()` (L222–242) writes via `setStyleOverride` (incl. the accent-decomposed vars), not session-only `body.style`. *Verify [BOSS TEST]:* edit accent + body-text colour, Apply, quit+relaunch → both stick.
- **1.3** `resetDraft()` (L243) clears the affected overrides. *Verify:* Reset → theme default, stays after relaunch.

## PHASE 2 — Catalog parity for the ~17 new vars (B1) *(additive; ~2 commits)*
- **2.1** Add the ~17 truly-new vars to `constellationStyleSettings.ts` (`bold-color/-weight`, `italic-color`, `strikethrough-color/-thickness`, `blockquote-text-color`, `h1..h6-color`, `ft-master-font-family`, `ft-row-radius`, `ft-border-width/-style/-color`, `ft-{library,folder,cuniverse}-*`, `universe-bar-*`, `font-{interface,text,monospace}-theme`). **Skip the ~8 already derived** (`--text-accent`, `--interactive-accent-hover`, `--accent-h/s/l`, `--background-*-alt`, `--text-muted/-faint`, `--editor-text-color`). *Verify:* Style Settings tab renders, no dup; existing values unchanged.
- **2.2** Reconcile `ELEMENTS` var names ↔ catalog ids (no-op audit). *Verify:* Setter H1-size and Style-Settings H1-size move the same `--h1-size`.

## PHASE 3 — Gap controls: Colors/Typography/Shape/Components *(additive; ~3–4 commits)*
- **3.1** "Colors" + "Typography" elements (modifier-*, text-muted/faint/on-accent/error/warning/success, interactive-accent-hover, text-accent; font-interface-size, line-heights, paragraph-spacing). **[BOSS TEST]** muted + line-height persist.
- **3.2** "Shape" element (radius-s/m/l, border-width, shadow-s/-l, file-line-width, file-margins). *Verify:* radius + reading width persist.
- **3.3** "Components" category (sidebar width/bg; dock; sidebar toolbar; layout bar; topbar+tabs; right sidebar; tab-radius; buttons; tags/callouts). *Verify:* dock/tab knobs apply to live chrome + persist.

## PHASE 4 — Fonts into the Setter *(~2–3 commits)*
- **4.1** Typeface pickers → `--font-{interface,text,monospace}-theme`. *Verify:* chrome font changes + persists.
- **4.2** Per-script fonts + font-theme + numerals via `updateSettings` (reuse font `$effect` L1657). **[BOSS TEST]** Typewriter + Arabic secondary + Hindi numerals, RTL intact, persists.

## PHASE 5 — Link styling *(~2 commits)*
- **5.1** "Links" category: 8 typed-link colours + add/delete/reset via the shared `saveLinkTypes` registry (same rail as `applyPreset`). *Verify:* recolour → panels update live; frozen link-colours preset still applies.
- **5.2** Display toggles (`colourTypedLinks`, `showTypedLinkLabels`) + pill shape (`linkPills.shape`). **[BOSS TEST]** colour + pill radius update live + persist.

## PHASE 6 — Unify Themes + Presets into one "Styles" gallery (C1) *(⚠ 6.2; ~3 commits)*
- **6.1** Replace Setter `THEMES` (L154) + "My themes" rail with `unifiedStyleList(savedStyles)` (stylePresets.ts L271) + `stylePreview` swatches. *Verify:* built-ins + customs + saved styles in one gallery; nothing rewritten.
- **6.2** ⚠ Card-click → `applyPreset` (L189, non-destructive merge); "+ new" → `newPresetFromCurrent`. **[BOSS TEST]** apply a built-in Style + Save current, relaunch → both present; earlier override still rides on top.
- **6.3** Fold theme CRUD/Obsidian-import into the Setter (call existing handlers; old tab still present). *Verify:* create/import/edit/delete from Setter == old tab.

## PHASE 7 — The 4 no-UI gaps + missing per-library apply path *(⚠ 7.4; ~5 commits)*
- **7.1** Accent picker → `accentColor`. *Verify:* applies + persists.
- **7.2** Dark/Light/System toggle → `colorScheme`. **[BOSS TEST]** System → OS dark-mode flip flips the app.
- **7.3** Custom-CSS editor → `theme.customCSS` (injected L1601). *Verify:* paste → live; clear → removed.
- **7.4** ⚠ NEW per-library apply `$effect` in +layout (keyed on active library + `libraryAppearances`, store.ts L2247/2256): apply accent + per-library fonts + css_theme, scoped after the theme/override effect, keys registered for clean clear-down on library switch. *Verify:* switch into/out of a library applies/clears (no bleed); composes with override (define precedence).
- **7.5** Per-library appearance editor in the Setter. **[BOSS TEST]** distinct accent for Library A → switching A/B flips it; persists.

## PHASE 8 — Second-screen full-style sync *(⚠ 8.1; ~2 commits)*
- **8.1** ⚠ `SecondScreenPage.svelte` theme-sync `$effect` (L377) also runs `deriveThemeVariables` + `generateStyleSettingsCSS` + applies `styleOverride` (mirror +layout L1589–1646). *Verify:* full colour/style parity main↔second.
- **8.2** Live re-sync on `screen:settings-changed` (emit L2064). **[BOSS TEST]** change accent + style var + override → second screen updates live.

## PHASE 9 — RETIRE old tabs (LAST, only at parity) *(deletions; ~4 commits)*
- **9.1** Parity gate (no code): walk audit §2.A–G; confirm 1:1 mapping. Only then proceed.
- **9.2** Remove the Style Settings tab (`SettingsModal.svelte` tab L270 + body ≈L2475–2548); keep `StyleSettingsPanel.svelte` + catalog (still the apply path). *Verify:* Settings opens; existing values still apply.
- **9.3** Remove Appearance styling controls (L2181–2473); **keep behavioural toggles** (audit §H). *Verify:* nothing lost.
- **9.4** Remove the font-styling controls moved in Phase 4 (Language tab L1244–1456; keep date formats etc.). **[BOSS TEST]** full pass — every style reachable + persistent from the Setter alone; old tabs gone; older build still boots ignoring `styleOverride`.

---

## Risk → Step mitigation
| Invariant / risk | Step | Verify |
|---|---|---|
| Existing themes load | 0.3 (after derivation), 2.1 (add-only ids) | boot each theme; colours correct |
| `styleSettingsValues` apply unchanged | 2.1 reuse derived; apply loop L1629 untouched | type in Style Settings → still applies |
| Frozen MIG-069 presets work | 6.1/6.2 use `unifiedStyleList`/`applyPreset` (read-time, non-destructive); 5.1 reuses `saveLinkTypes` | apply a preset → unchanged |
| Look survives theme switch | 0.3 registers keys + re-applies after derivation | [BOSS TEST] 0.3 / 6.2 |
| No boot/typing/IPC regression (7,600 notes) | 0.4 debounced; var-writes not scans; zero keystroke IPC | type in large Universe w/ Setter open |
| Second screen | 8.1/8.2 build the missing mirror + live re-sync | [BOSS TEST] 8.2 |
| Per-library apply | 7.4 new apply path w/ clean clear-down; 7.5 editor | [BOSS TEST] 7.5 |
| RTL / i18n | 4.2 via `SCRIPT_UNICODE_RANGES` + font `$effect` | [BOSS TEST] 4.2 |
| FocusPane plain-text | no step changes Focus; verify after P0/P3 | enter Focus → exception holds |
| BUG-015 race (delete old first) | both paths write same values P1–P8; deletions only P9 after gate 9.1 | parity checklist |
| LL-023 (new guard steals state) | 7.4 only new write path — registers keys, scoped, clears on switch | library switch leaves no stale vars |

## Rollback
- `styleOverride` is one additive key; older builds ignore + preserve it (`applyParsedSettings` L3874). No data loss on downgrade.
- Each step = one `§N` commit; revert any phase → old tabs still write the same values → working app. Reverting 0.3 just stops reading `styleOverride` (values stay dormant in JSON).
- Keep both old tabs + Setter live through Phase 8; run Phase 9 only after the 9.1 parity gate. Drift after 9.x → revert that 9.x commit to restore the old tab (no schema change needed).
- Frozen MIG-069 never modified — rollback cannot corrupt presets.

**Commit estimate ≈ 30–32:** P0~4 · P1~3 · P2~2 · P3~3–4 · P4~2–3 · P5~2 · P6~3 · P7~5 · P8~2 · P9~4.
