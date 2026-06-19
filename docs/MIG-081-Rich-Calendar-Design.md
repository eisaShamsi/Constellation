# MIG-081 — Rich Calendar Design (port Eisa's app UI + Style-Setter)

**Status:** Design (Boss-directed "study your app's UI, then design", 2026-06-17). Supersedes the plain §C grid. Awaiting Boss review of the one open decision → build.
**Source studied:** `eisaShamsi/hijri-calendar` (`index.html`, `app.js`, `style.css`, the vendored `hijri.js`) via a 3-agent reverse-engineering pass. Target = the Boss screenshot (the app's rich month view).

## 1. What the engine already gives us (the key finding)
The vendored `hijri.js` exposes a **whole-grid** API — we barely build grid logic ourselves:
- **`getMonthData(year, month)`** → `{ year, monthName, gregorianRange ("June – July 2026"), orderedDayNames (week-start applied), days[ { jdn, hijriDay, gregorian:{y,m,d}, dayOfWeek, weekNumber, isToday, isOtherMonth } ] }`.
- **`getMoonPhase(gY,gM,gD)`** → `{ phase 0–7, symbol (●◗◑◕○◔◐◖), name/nameAr/nameEn, age, illumination }` (Meeus; astronomical → valid for ANY Gregorian date, so usable for every system).
- **`getEvent(hMonth,hDay)`** → `{ type: 'holiday'|'observance'|'special', name }` from the 21-entry `ISLAMIC_EVENTS` table. **`isSacredMonth(m)`**, **`toArabicNumerals(n)`**, **`getEclipseInfo`**, **`getTale3`** (Durur season markers — optional).
- Week number = `weekOfYear(jdn, hijriYear)` (Hijri ISO week, week-start configurable) — already in `day.weekNumber`.

## 2. The rich UI (from the app)
- **Header:** a `nav-bar` — circular ‹ › buttons + a circular "Today" button; centre `month-info` = the gold **pill** "`{Month} {Year} AH`" (papyrus-cream gradient `#f5e6c8→#eedbb5`, gold border `#c49440` when **sacred month**, else the green theme) + the **Gregorian-range subtitle** ("June – July 2026"). Header bg = green gradient `#14553f→#1a6b4f`.
- **Grid:** 8 columns = **Wk** (42px) + Mon..Sun (`orderedDayNames`); each row's first cell = the week number. Day cell (min-height 62px): **Hijri day** (Amiri, 1.2rem, `#0d3b2e`) + **Gregorian day** beneath (small, `#0e7490` teal) + a **moon glyph** top-corner (`#374151`) + an **event dot** (holiday red `#ef4444` / observance gold `#d4a017` / special purple `#8b5cf6`). **Today** cell = gold gradient `#b8860b→#d4a017`, white text. Other-month cells muted.
- **Fonts:** Amiri (400/700) + Cairo (the repo ships the woff2; Constellation has per-script fonts — reuse or vendor Amiri/Cairo for the calendar).

## 3. Port plan
- **`calendarMath` — add a RICH grid builder** returning, per cell: `{ iso, primaryDayLabel, gregDayLabel, weekNumber, isToday, inCurrentMonth, moonSymbol, moonName, eventType?, eventName? }` + header `{ monthLabel, suffix (AH/—), gregorianRange, isSacredMonth }`. **Hijri** path = `getMonthData` + `getMoonPhase` + `getEvent` + `isSacredMonth` (verbatim from the engine — "the same one"). **Persian/Hebrew/Gregorian** path = Temporal (existing) + `getMoonPhase` (universal) + a week-number calc; NO Islamic events / AH / sacred pill (see §5).
- **`CalendarPanel` — rich render:** the ornate header (pill + suffix + Gregorian range + Today/‹›), the optional **Wk column**, dual-date cells with moon glyph + event dot, sacred-month pill variant, Arabic numerals when locale=ar (`toArabicNumerals`/`localeNum`), RTL via `dir`. All colors/sizes via **CSS variables** (Style-Setter tokens).
- **Settings:** add a **"Show week numbers"** toggle (Boss-requested) to the Calendar settings (`calendarShowWeekNumbers`, default on). (Primary/secondary/week-start already in §B; "secondary" is now subsumed by the always-shown Gregorian sub-number — see §5.)
- **Style Setter:** a new **Calendar** token group (~18 tokens) wired to the existing theming engine (catalog + apply path), exposing the palette/fonts as CSS vars: `--cal-header-from/to`, `--cal-pill-bg/-sacred/-border/-text`, `--cal-today-from/to/-text`, `--cal-hijri-color`, `--cal-greg-color`, `--cal-weekday-color`, `--cal-othermonth-bg/-text`, `--cal-event-holiday/-observance/-special`, `--cal-moon-color`, `--cal-grid-border`, `--cal-font`, `--cal-font-display`. Defaults = the extracted app palette.
- **Fonts:** vendor Amiri + Cairo woff2 (from the repo) for the calendar display, or map to Constellation's existing script-font system.

## 4. Phasing
- **§C.2a — rich grid in `calendarMath`** (Hijri via `getMonthData`; the rich cell shape) + Node verify.
- **§C.2b — rich `CalendarPanel`** (header pill + Gregorian range + dual-date cells + moon glyphs + event dots + sacred pill + CSS vars) — the visible Boss-test.
- **§C.2c — Wk-column toggle** setting.
- **§C.2d — Style-Setter Calendar token group** (catalog + apply + a preview per the full-center-zone rule).
- **§C.2e — fonts** (Amiri/Cairo) + ×15 i18n + audit.

## 5. Open decision for Boss
**Non-Hijri systems (Persian/Hebrew) — how much of the rich treatment applies?** The Islamic enrichments — the **AH pill**, **sacred-month** gold, **Islamic-event dots** — are intrinsically Hijri. **Moon phases, dual Hijri/Gregorian dates, week numbers, and the ornate theme are universal.** Recommended: the rich *shell* (ornate header, dual dates, moon glyphs, week numbers, Style-Setter theme) applies to **all** systems; the Islamic-specific bits (AH suffix, sacred pill, Islamic-event dots) show **only for Hijri** (Persian shows "SH"/Jalali suffix + Persian month names; Hebrew shows its months; no Islamic events). Confirm — or should the rich design be **Hijri-only** (other systems keep a simpler grid)?

> Note: "two calendars doesn't work" (Boss) → resolved by the rich cell showing **both** dates inline (Hijri primary + Gregorian sub-number), not a separate secondary-calendar toggle. The §B `calendarSecondarySystem` setting is dropped/repurposed.
