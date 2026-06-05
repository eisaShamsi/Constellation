# MIG-070 §C — HANDOVER (Unify all styling into the Style Setter)

**Written:** 2026-06-05 (updated post-handover) · **Latest commit:** `3f01ce1b` (docs) · **Branch:** `main` · **Working tree:** ⚠ **NOT clean** — Phase 5 §5.1 is in-flight + uncommitted (see §4); run `git status` before assuming anything.
**Binary:** `src-tauri/target/release/constellation.exe` mtime `2026-06-04 20:22:27` — reflects committed code only; the in-flight Phase 5 §5.1 edits are **NOT built**. Rebuild before testing §5.1.

This is the single file a fresh session reads to resume MIG-070 §C cold. It is self-contained; the authoritative companions are:
- **Architect + audit + invariants + rollback:** `docs/MIG-070-style-merge-AUDIT.md`
- **The 10-phase Plan (line anchors per step):** `docs/MIG-070C-PLAN.md`
- **Detailed commit trail (this run):** `lab/reports/SESSION-LOG-2026-06-02.md` (+ state-of-standing in `SESSION-LOG-2026-06-05.md`)
- **Orientation (read first, always):** `docs/Constellation Orientation & Onboarding v2.52.md`

---

## 1. What this migration is (one paragraph)

Constellation had **THREE styling surfaces with three storage models** — the Style Settings catalog (`styleSettingsValues`, per-theme), the standalone **Style Setter** overlay (was session-only body vars), and the **frozen MIG-069 Style Presets** (app-global JSON). MIG-070 §C unifies all styling so the **Style Setter is the ONLY styling surface**, with persistence = **theme base + per-Universe `appSettings.styleOverride`** (override merged last in the shared `+layout` apply `$effect`). Scope = everything in one formal `/migration`. The frozen MIG-069 presets are NEVER modified — they ride on top, read-time, non-destructive.

---

## 2. The resume prompt (copy-paste into the next session)

> Resume **MIG-070 §C — unify all styling into the Style Setter**. Location `E:\مشاريع كلاود\Constellation`, branch `main`. First: `git pull origin main`, then read `docs/Constellation Orientation & Onboarding v2.52.md`, then `docs/MIG-070C-HANDOVER.md` (this file), then `docs/MIG-070C-PLAN.md`. We are at commit `b49658e1`; Phases 0–4.2 + saved swatches + focused per-element previews are **shipped and Boss-validated**. The plan was approved (Plan Approval = Build Approval — cascade autonomously, stop only at `[BOSS TEST]` clauses, architectural surprise, or plan completion). **Next is Phase 5 — link colours** (`docs/MIG-070C-PLAN.md` §5.1–5.2). Before editing, state the function-in-hand, write the Predecessor→Replacement entry into `lab/reports/SESSION-LOG-2026-06-05.md`, and verify the Phase-5 save path in `src/lib/components/LinkTypesEditor.svelte` + `src/lib/libraries/linkTypeRegistry.ts`. Build with the running app **stopped** (`Get-Process constellation | Stop-Process -Force; Start-Sleep -Seconds 2`) then `npm run tauri build -- --no-bundle`; confirm binary mtime advances before any Boss test. Stage tests as tutorials (define feature → click-by-click). Do NOT touch the frozen MIG-069 presets. BASIC RULE applies — no invented paths/anchors.

---

## 3. Shipped & Boss-validated (DO NOT REGRESS)

| Area | What landed | Where |
|---|---|---|
| **Persistence spine (Phase 0/1)** | per-Universe `styleOverride` merged on top of theme in the shared apply `$effect`; survives theme switch + clears cleanly; Setter `apply()`→`mergeStyleOverride`, `resetDraft()`→`clearAllStyleOverride`, seeds draft on open | `store.ts`, `+layout.svelte`, `StyleSetter.svelte` |
| **Every Markdown element (§3A)** | H1–H6 (own colour+size, shared weight), bold, italic, strikethrough (line colour+thickness), inline code (bg/text/size), blockquote text — all read CSS vars | `NotePane.svelte` `markdownHighlightStyle`, `livePreview.ts` `livePreviewTheme` |
| **Interface elements (§3B)** | file tree per-row-type (Library/Folder/cUniverse over a master), Status bar, Universe bar (+ text size); tab text + library label + breadcrumb follow **interface**; note title+body follow **note** (`--editor-text-color` note-scoped default) | `FileTree.svelte`, `+layout.svelte`, `deriveThemeVariables` |
| **Categories rail** | left rail grouped into Surfaces (Interface · Components · Editor · Global · Sky View · OrgChart · Index · Cataloger · Shell) | `StyleSetter.svelte` `CATEGORIES`/`CATEGORY_OF` |
| **Global category (Phase 3)** | backgrounds/text-shades/status/accent shades; type & rhythm (interface size, line-heights, paragraph spacing); shape (radii, border, reading width, margins) | `StyleSetter.svelte` |
| **Components category** | dock · sidebar toolbar · layout bar · top bar & tabs · right sidebar · buttons · tags & callouts · sidebar shell | `StyleSetter.svelte` |
| **Saved colour swatches** | per-Universe palette; auto-save on settle; click→apply to active control, right-click→remove (cap 24) | `styleSwatches`, `addStyleSwatch`/`removeStyleSwatch` |
| **Focused per-element preview (#4)** | centre replicates ONLY the selected element (note / tree rows / switcher / status / each chrome widget / global composite) — no more always-on mini-app box | `StyleSetter.svelte` `pk` derived |
| **Fonts (Phase 4.1/4.2)** | 14 curated Latin typeface stacks + Code font; **per-script fonts** (Arabic/Hebrew/CJK/Devanagari/Cyrillic) language-smart — an Arabic note shows its Arabic font even in an English interface; chrome follows interface font | `perScriptFonts`, `setPerScriptFont`, `+layout` font `$effect` (`CnSetterText`/`CnSetterUI` virtual families) |

**Two decisions locked (Phase 4.2 follow-ups):**
1. **Interface-language selector stays in Settings → Language** (removed from the Setter — it's a locale setting, not styling; per-script *fonts* stay in the Setter). Done in `b49658e1`.
2. **Setter UI localization (15 languages) happens AFTER all content is final** (so the ~100-string set is translated once). Queued as the penultimate step, before Phase 9 retire.

---

## 4. Next up — Phase 5: Link colours (~2 commits)

> ⚠ **In-flight (2026-06-05):** §5.1 has **already been started in this working tree (uncommitted)** by a parallel session — `LinkTypesEditor.svelte` gains an `embedded` prop, `StyleSetter.svelte` gains the "Links" category embedding `<LinkTypesEditor embedded/>`. **Do not revert or re-implement** — `git status` and read the working-tree diff first. §5.2 (display toggles + pill shape) not started. Detailed Phase-5 prep (verified save paths, Predecessor→Replacement entry) is in `lab/reports/SESSION-LOG-2026-06-05.md`.

From `docs/MIG-070C-PLAN.md` §5:
- **5.1** New **"Links" category** in the Setter: the **8 typed-link colours** + add/delete/reset, written through the **shared link-type save path** (so the panels — Backlinks/Outgoing/pills — recolour live). *Verify:* recolour → panels update live; the frozen link-colours preset still applies.
- **5.2** Display toggles **`colourTypedLinks`** (store.ts L3242), **`showTypedLinkLabels`** (L3244) + pill shape **`linkPills`** (L3337) via `updateSettings`. **[BOSS TEST]** colour + pill radius update live + persist.

**Reuse, don't reinvent:** the existing link-type editor is `src/lib/components/LinkTypesEditor.svelte`; the registry is `src/lib/libraries/linkTypeRegistry.ts`; the pill renderer is `src/lib/components/LinkTypePill.svelte`. **Confirm the exact save function at Phase-5 start** — the plan calls it `saveLinkTypes`, but verify the real name/location in `linkTypeRegistry.ts` before wiring (it is NOT in `store.ts`).

**Watch:** `note_links.link_type` is globally `'relates'` (project memo `project_note_links_link_type_relates_bug.md`) — a foundational, separate bug. Don't try to fix it inside Phase 5; just don't build Phase 5 in a way that assumes link_type is populated correctly.

---

## 5. Remaining after Phase 5

- **Phase 6** — unify Themes + MIG-069 Presets into ONE "Styles" gallery (`unifiedStyleList`/`applyPreset`, read-time non-destructive). ⚠ 6.2 [BOSS TEST].
- **Phase 7** — the 4 no-UI gaps: accent picker · dark/light/system toggle · custom-CSS editor · **per-library appearance + its MISSING apply path** (⚠ 7.4 — a NEW per-library apply `$effect` in `+layout`, keyed on active library + `libraryAppearances`, with clean clear-down per LL-023). [BOSS TEST] 7.2, 7.5.
- **Phase 8** — second-screen full-style sync: `SecondScreenPage.svelte` must also run `deriveThemeVariables` + `generateStyleSettingsCSS` + apply `styleOverride` (mirror `+layout`), plus live re-sync on `screen:settings-changed`. ⚠ 8.1. [BOSS TEST] 8.2.
- **Setter UI localization** — all 15 languages (decision #2 above). After content final, before retire.
- **Phase 9** — RETIRE the old Appearance + Style-Settings tabs **only at a 9.1 parity gate**; also do the deferred **Phase 2** (catalog parity for the ~17 Setter-only vars) here. [BOSS TEST] 9.4.

**Deferred small items:** swatch *rename* (memory #2); full installed-fonts enumeration (currently a curated list).

---

## 6. Key files & anchors (verified 2026-06-05 — line numbers drift, re-grep)

**`src/lib/libraries/store.ts`**
- `AppSettings` new keys: `styleOverride: Record<string,string>`, `styleSwatches: string[]`, `perScriptFonts: Record<string,string>` (all in `DEFAULT_SETTINGS` as `{}`/`[]`/`{}`).
- Helpers (all debounced via `saveSettings`, emit `screen:settings-changed`): `setStyleOverride` (L4084), `clearStyleOverride`, `mergeStyleOverride` (L4105), `setPerScriptFont` (L4112), `clearAllStyleOverride` (L4123), `addStyleSwatch` (L4130), `removeStyleSwatch`.
- Link toggles: `colourTypedLinks` (L3242), `showTypedLinkLabels` (L3244), `linkPills` (L3337).
- `deriveThemeVariables` (~L3100) emits `--editor-text-color: text` (note-scoped default).

**`src/routes/+layout.svelte`**
- Theme apply `$effect` (~L1554): collect-then-apply; derived theme vars (untrack) + Style-Settings vars + **`styleOverride` last writer**, all tracked in `_lastStyleSettingsKeys` for clean clear-down; applies in BOTH theme and no-theme paths.
- Font `$effect` (~L1655): per-script composition — Latin base from `styleOverride` font vars; per-script via `@font-face` unicode-range using virtual families **`CnSetterText`/`CnSetterUI`** (DO NOT override `--font-text-theme` wholesale — it bypasses the per-script engine and breaks Arabic).

**`src/lib/components/StyleSetter.svelte`** — `ELEMENTS` map, `CATEGORIES` + `CATEGORY_OF`, `pk` derived (focused preview), control types `color`/`select`/`range`/`scriptfont`, `apply()`→`mergeStyleOverride`, `resetDraft()`→`clearAllStyleOverride`, draft seed `$effect` from `appSettings.styleOverride`, swatches (`addStyleSwatch`/`removeStyleSwatch`, `activeColorVar`, `applySwatch`).

**Markdown rendering (Editor Parity Rule — both must match):** `src/lib/components/NotePane.svelte` (`markdownHighlightStyle` reads the per-element vars) + `src/lib/editor/livePreview.ts` (`livePreviewTheme`).

---

## 7. How to build & test (process discipline)

1. **Stop the running app first** (Windows holds the exe): `Get-Process constellation | Stop-Process -Force; Start-Sleep -Seconds 2`.
2. Build: `npm run tauri build -- --no-bundle` (~2 min). Output: `src-tauri/target/release/constellation.exe`.
3. **Verify binary mtime advanced** before any Boss test (Stage 0 — memory `feedback_verify_binary_before_testing`). If mtime pre-dates the change, STOP and rebuild.
4. **Boss tests are tutorials** (Testing Instructions Rule): define the feature → walk click-by-click → pre-state/action/post-state → failure modes. **Stage them** — send Stage 1, wait, then Stage 2 (memory `feedback_staged_tests`). Eisa prefers **.exe** test installs.
5. **Devtools are DEV-only** — Boss tests must rely on observable UI, never `console` (memory `feedback_devtools_dev_only`).

---

## 8. Invariants & gotchas (the "don't break this" list)

- **Frozen MIG-069 presets** — never modified; Phases 5/6 use read-time non-destructive merge only.
- **Rollback** — `styleOverride`/`styleSwatches`/`perScriptFonts` are additive keys; older builds ignore + preserve them (`applyParsedSettings` spread). No data loss on downgrade.
- **Per-script font engine** — feed it via `CnSetterText`/`CnSetterUI`; never clobber `--font-text-theme`.
- **Both old tabs (Appearance + Style Settings) stay LIVE** through Phase 8 — they write the same values; retire only at the Phase 9.1 parity gate (avoids the BUG-015 race; deletions last).
- **LL-023** — any new apply path (Phase 7.4 per-library) must register its keys for clean clear-down on switch, or it steals state.
- **Editor Parity Rule** — any markdown styling change lands in BOTH `NotePane` highlight + `livePreview` theme (FocusPane exempt — plain text).
- **Performance** — zero `invoke()` on the keystroke path; helpers debounced; var-writes not scans; measure on the 7,600-note Universe before commit.
- **BASIC RULE** — no invented file paths, line numbers, or function names. Re-grep before citing.

---

## 9. Standing-order pointers for the next session

- Log each `§N` commit to `lab/reports/SESSION-LOG-2026-06-05.md` as it lands (SO).
- Orientation v-bump lands **in the same commit** as any SO #6 trigger (TOP PRINCIPAL) — a phase shipping is a trigger; bump to v2.53 when Phase 5 ships if it's structural.
- Write a MoCh every ~3h of direct chat (`docs/MoCh/MoCh-YYYY-MM-DD-HHMM.md`).
- State the **function-in-hand** before any edit; write the **Predecessor→Replacement** entry before removing/relocating anything; **Stop-On-Correction** if Eisa says "wrong target / no / we're working on X".
