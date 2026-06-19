# Session Log — 2026-06-19 (MIG-081 — the Calendar's 4 new functions)

> **Function in hand:** MIG-081 — the Calendar's 4 new functions (Boss-requested 2026-06-17), in Eisa's order:
> (1) Month Correction, (2) Calculation Mode, (3) Style-Setter "Calendar" tab, (4) retire the Daily-Note dock button.
> Branch `main`. Picks up from `lab/reports/HANDOVER-2026-06-17-mig081-rich-calendar.md`. Continues MIG-081 (already an open
> migration; these fold under its Architect §9 phasing like §C.2a–c did — additive, frontend-only, no new Rust).

## Session-start ritual
- `git pull origin main` → already up to date (through `1c4f420f`).
- Read orientation v2.90 pointers + the handover + SESSION-LOG-2026-06-17 (the §C.2b "Perfect!" close + the 4 new asks).

## Familiarization (grounded, not guessed — verified against the real code)
- **Engine API (`src/lib/calendar/hijri.js`):** `setMode('astronomical'|'tabular')` sets `currentMode` (does NOT persist); `_saveMode()` writes localStorage `hijri-mode`; `getMode()`; `MODE_NAMES` exposed. `setCorrection(y,m,off)` (auto-saves; off 0 deletes), `getCorrection`, `clearCorrections` (auto-saves), `getAllCorrections()` → `{ "Y-M": off }`. All in the export block (3743 / 3755). **Verified the unified `hijriToGregorian`/`gregorianToHijri`/`daysInMonth` route through `_engine()` (mode switch) AND `_getCumulativeCorrection` (corrections)** — exactly the functions `calendarMath.buildMonthGrid` uses → setting mode/corrections changes the rendered grid. (Task 2's "verify the public setter" requirement: DONE — `setMode` exists + exported; no engine edit needed.)
- **Style-Setter (`StyleSetter.svelte`):** catalog = `ELEMENTS` (element→controls→CSS-var) + `CATEGORIES` (key/name/surface/elements); `twoZone` derived excludes editor/sky/cns (three-zone centre preview); `draft`→`mergedDraft()`→`mergeStyleOverride`→`appSettings.styleOverride`→`+layout` writes it to `<body>` (the BUG-015 single-writer). Themable components consume `var(--x, fallback)` inline (CNS `--cns-bg` precedent), NOT a local declaration.
- **Daily launcher (`+layout.svelte`):** `showDailyLauncher`/`dailyLauncherToday` + dock button/popover (~5524) + `.daily-launcher-*`/`.dl-*` CSS; `openDailyNote`/`handleOpenDailyNote` (command palette + Calendar page) must stay.

## Boss decisions (AskUserQuestion)
- **Persistence = appSettings (synced).** Corrections + mode live in the universe's appSettings (travels with the synced universe, survives reinstall, consistent across iOS/macOS/Windows), pushed into the singleton engine on load — NOT the engine's per-device localStorage.
- **Correction UI = Settings only.** All correction controls (year/month/offset picker, the list, Clear All) in Calendar Settings. The Calendar page stays display-only.

## §C.2f — Month Correction + Calculation Mode (Tasks 1+2) — BUILT + Claude-verified (pending Boss test)
- **`store.ts`:** added `calendarCorrections: Record<string,number>` (default `{}`) + `calendarCalculationMode: 'astronomical'|'tabular'` (default `'astronomical'`) to the appSettings type + DEFAULT_SETTINGS.
- **`calendarMath.ts`:** `applyCalendarPrefs(corrections, mode)` (await engine → `clearCorrections` → replay each `setCorrection` → always `setMode`) + `hijriMonthNames(locale)` (12 localized names) + `CalculationMode` type. appSettings is the source of truth; the engine's localStorage is overridden on every load.
- **`CalendarPanel.svelte`:** new props `corrections` + `calculationMode`; the engine-load `$effect` now calls `applyCalendarPrefs` before anchoring today (re-runs on prefs change → re-anchor → grid re-derives via the `enginesReady` toggle). `+layout` Calendar page passes both from appSettings.
- **`SettingsModal.svelte`:** a **Hijri calendar** subsection in the Calendar category — a **Calculation method** select (Astronomical (Lunar Conjunction) / Tabular (al-Tawfīqāt al-Ilhāmiyyah)) + a **Month correction** picker (Hijri year input + month select + ±2/±1 offset + Set) + the corrections **list** (localized "Muharram 1448 AH · +1 day" rows with × remove) + **Clear all**. Engine loads lazily when the Calendar section opens; defaults the picker to the current Hijri month.
- **`en.json`:** 19 new `settings.calendar.*` keys (hijriHeading/hijriIntro/calcMode/calcModeDesc/calcAstronomical/calcTabular/correction*/ahSuffix/day/days). ×15 rides §C.2e (EN-key gotcha handled — EN present → all locales render English via the fallback chain).

## §C.2d — Style-Setter "Calendar" category (Task 3) — BUILT + Claude-verified (pending Boss test)
- **`CalendarPanel.svelte`:** moved all `--cal-*` defaults off the `.cal-root` declaration block into inline `var(--cal-X, default)` fallbacks (so a body-level styleOverride / the preview draft INHERITS and wins — a local declaration would block it). Added a themable `--cal-font` and `data-style-target="calendar"` (inspect-to-style). Layout-only `--cal-wk-col` stays local.
- **`StyleSetter.svelte`:** a `calendar` element (24 controls: font + 23 colours covering every `--cal-*`); a `calendar` CATEGORY (surface `calendar`); added to the three-zone set (`twoZone` exclusion); a **full-center-zone** centre preview that renders the REAL `<CalendarPanel>` under the `.ss` draft wrapper (reuse, not a mockup — Style-Setter Preview Rule honored) — one click-target selects `calendar`, header nav still scrubs months.
- Apply path verified: `mergedDraft()` = `{...draft}` (no filter) → `mergeStyleOverride` → `styleOverride` → body → `var(--cal-*)` picks it up live.

## §C.2g — Retire the Daily-Note dock button (Task 4) — BUILT + Claude-verified (pending Boss test)
- Removed the launcher dock button + popover + backdrop + date input (`+layout.svelte`), the `showDailyLauncher`/`dailyLauncherToday` state, and the `.daily-launcher-*`/`.dl-*` CSS. Left a retirement comment. **Kept** `openDailyNote` + `handleOpenDailyNote` (command palette `:2077` + the Calendar page day-click). Left dock now: Calendar button only for daily notes.

## Verification (Claude-side)
- **svelte-check: 0 errors / 315 pre-existing warnings** after all three sub-steps.
- **WA#4 independent review** (general-purpose agent on the diff): **no P0/P1.** One P2 found + **FIXED**: a `--cal-grid-border` consumer (`.cal-wk` background) had dropped the `var(--border)` indirection → would diverge in dark themes; restored to match the other two consumers (+ aligned the `.cal-cell:hover` `--cal-cell-bg` fallback for consistency). Cleared: every other `--cal-*` default byte-identical; no `$effect` loops; singleton-engine idempotent (no race); launcher removal clean (zero orphans); en.json valid.
- **Editor-Surface Gate:** all three sub-steps are settings/styling/launcher changes — they touch NO note content/save/lifecycle code (content-integrity class structurally untouched). Boss test still exercises the gate as belt-and-suspenders.
- Frontend rebuilt (`npm run build`); binary `cargo build --release` (from `src-tauri`) — rebuild in progress.

## ⚑ Boss validation + refinements (2026-06-19)
- **Stage 1 (§C.2f corrections + mode): "All pass"**, with two Boss refinements:
  1. **Offset dropdown** → default **0**, ordered `+2 / +1 / 0 / −1 / −2`; now reflects the chosen month's existing correction (a reflect `$effect`: reads year/month/corrections, writes only `corrOffset` — no loop). Built + re-validated.
  2. **Subtitle = the OTHER calendar** (not a repeat of the primary). Added `hijriRange()` in calendarMath; renamed `RichMonthGrid.gregorianRange` → `subtitleRange` (cross-ref: Gregorian range when primary≠Gregorian, **Hijri** range when primary=Gregorian). Node-verified: Gregorian June 2026 → "Dhul-Hijjah 1447 – Muharram 1448 AH". CalendarPanel reads `grid.subtitleRange`. Built + re-validated ("All pass").
- **Stage 2 (§C.2d Style-Setter) + Stage 3 (§C.2g retired button): "All pass."**
- **Text-size control:** first shipped a single master `--cal-font-size` scale (em-based). **Boss rejected** → wants **each element its own sizing + colouring**. Reverted the master scale; gave 9 textual elements their own size token (`--cal-{day,subdate,pill,subtitle,weekday,week,moon,today,nav}-size`, each keeping its original rem default; `.cal-suffix` stays `0.75em` so it tracks the pill). StyleSetter calendar element rebuilt: **font + 23 colours + 9 sizes = 33 controls**, each element's colour + size paired. Boss: **"What a milestone. All passed perfectly."**
- **All four asks + refinements + per-element sizing/colouring SHIPPED + Boss-validated.** (Flat paired control list kept; offered grouped sections — Boss didn't request, left flat.)

## §C.2e — close-out (DONE)
- **Fonts: DONE (no code).** Amiri is already bundled (`static/fonts/Amiri-Regular/Bold.ttf`) AND declared via `@font-face` in `static/fonts/fonts.css` → `--cal-font: 'Amiri', 'Cairo', var(--text-font)` already renders in Amiri (confirmed by the §C.2b "Perfect!" look). Cairo not shipped — harmless fallback after Amiri.
- **i18n ×15 COMPLETE.** Two fan-out Workflows (one native translator per locale):
  1. `settings.calendar` (34 keys) + `calendarPanel.weekAbbrev` — were EN-only across all 14 non-EN locales. Translated + written + validated (clean additions-only diff via `json.dump(ensure_ascii=False, indent='\t')` — round-trip-confirmed). Format token `%Y-%m-%d` preserved everywhere; the date-pattern notation `YYYY-MM-DD` localized per locale (JJJJ-MM-TT / AAAA-MM-DD / …) per the full-localization principle.
  2. The 34 **StyleSetter Calendar labels** (the audit's P2-1) — also EN-only; translated + written ×15 into `styleSetter.labels`.
  - **Workflow lessons:** (a) `args` as a top-level ARRAY arrives non-iterable → hardcode lists in the script. (b) a 35-named-property object schema came back numeric-keyed; an array schema is cleaner but does NOT guarantee ORDER (pt block-rotated 5 items) → verified each locale via a size-label parity check before mapping; pt fixed by deterministic rotation (translations were correct, only repositioned). **Never positionally-map an LLM array without verification** (fabrication-class risk).
- **3-agent migration audit (Migration Rule Phase 4) — CLEAN.** Invariants **7/7 HOLD** (INV-1 filenames stay Gregorian ISO; INV-2 dots/onDayClick key on ISO; INV-3 appSettings back-compat; INV-4 singleton-engine idempotent/convergent; INV-5 default look byte-identical; INV-6 Editor-Surface Gate untouched; INV-7 no $effect loops). Migration **5/5 SAFE** (first-boot defaults, localStorage-override, rollback, i18n fallback, preview lazy-load). **No P0/P1.** Three P2s — all FIXED/documented:
  - **P2-1** StyleSetter Calendar labels not localized → FIXED (34 labels ×15, above).
  - **P2-2** orphaned `ribbon.dailyNote` (launcher-only, now unreferenced) → REMOVED ×15 (confirmed `commands.dailyNote` is the live palette key; no dynamic `ribbon.*` construction).
  - **P2-3** applyCalendarPrefs mirrors corrections into localStorage as a discarded side-effect → documented with a code comment (no functional change; appSettings stays the single synced source).
- **Docs:** orientation **v2.91** (NEW file, v2.90 preamble retained); User Manual §14 (EN — the new calendar features); MoCh `docs/MoCh/MoCh-2026-06-19-1800.md`; handover `lab/reports/HANDOVER-2026-06-19-mig081-complete.md`; next-prompt updated. (Manual ×14 translations ride the standing debt.)

## SESSION CLOSE
- **MIG-081 COMPLETE + Boss-validated** ("What a milestone. All passed perfectly."). The 4 Calendar functions + 2 refinements + per-element Style-Setter theming + ×15 localization + clean audit, all on `main`.
- Final verification: svelte-check **0 errors / 315 pre-existing warnings**; `cargo build --release` clean (52 pre-existing dead-code warnings); i18n re-embed confirmed in `build/` (`calcTabular` + Arabic `التوفيقات`).
- **NEXT:** MIG-080 §B–§F right-rail note-context cascade (Plan approved) + the deferred inspector360 Settings-UI bug.

## Pending (after §C.2e close-out)
- Full PCS: commit, orientation v-bump (SAME commit per SO #6), MoCh, handover, next-session prompt, User Manual.
- Still queued after MIG-081: the MIG-080 §B–§F right-rail note-context cascade (Plan approved: `docs/MIG-080-Plan.md`) + the deferred inspector360 Settings-UI bug.
