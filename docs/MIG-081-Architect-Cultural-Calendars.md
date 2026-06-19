# MIG-081 — Architect: Cultural Calendars + Calendar Settings

**Status:** Architect (Phase 1 of the /migration). Awaiting Boss review → Plan → approval → Build → Audit.
**Date:** 2026-06-17. **Branch:** `main`. **Boss directive (2026-06-17):** "build #2+#3 now as a /migration" — #2 = Calendar settings (don't exist); #3 = "since Constellation supports many languages, the calendar should provide those cultures' own calendars, integrated or standalone."
**Function in hand:** the **Calendar** subsystem — the §A Daily-Note launcher + the §A.2 full-page Calendar (`CalendarPanel.svelte`, `openDailyNote`), and a NEW **Calendar settings** section.
**WA#5 cross-check:** done (agent `a01e938727b623301`) — the proven approach is settled (Intl display + Temporal grid math); cited below.

---

## 1. Contract
Let users view and navigate the Calendar in **their own culture's calendar system** — Gregorian, **Hijri** (Islamic), **Solar Hijri** (Persian/Jalali), **Hebrew** — either **standalone** (the whole month grid switches) or **integrated** (a secondary cultural date shown alongside Gregorian). Multilingual-by-default, top-principal. **The note's on-disk identity never changes** — daily-note filenames stay Gregorian ISO.

## 2. Systems (Boss's established vocabulary — reuse, don't reinvent)
`appSettings.sight.calendarSystems` already defines Eisa's set: **`gregorian` | `hijri` | `solar-hijri` | `hebrew`** (the Sight dome's rings — a *separate* feature, but the same vocabulary + i18n labels `settings.sight.calendarSystems.*` exist in all 15 locales, and PJ-014 tracks full month-name localization). **MIG-081 adopts the same 4 systems** for the Calendar.
- **Map to Intl/Temporal identifiers:** `gregorian`→`gregory`, `hijri`→**`islamic-umalqura`** (NOT bare `islamic` — MDN: undocumented/warns), `solar-hijri`→`persian`, `hebrew`→`hebrew`.
- **Deferred (Form-Aligns-To-Purpose):** Indian/Buddhist (regular, easy to add later) and **Chinese/Korean** (lunisolar leap-months don't fit a fixed 6×7 grid). Out of scope; documented.

## 3. Technical approach (WA#5-confirmed)
- **Display** → `Intl.DateTimeFormat` with `{ calendar, numberingSystem }` (or `-u-ca-…-nu-…`). In-browser, zero deps. Used for month names, day labels, the secondary-date label.
- **Grid math** → **Temporal** (`Temporal.PlainDate.withCalendar(cal)`, `.daysInMonth`, `.dayOfWeek`). Plain JS `Date` CANNOT compute Hijri/Hebrew/Persian month lengths/starts — Temporal is required for a standalone non-Gregorian grid. **Ship `@js-temporal/polyfill`** (~20 KB gz), **lazy-loaded into the Calendar chunk only** (Perf Rule 6 — never the boot/editor path); feature-detect native `globalThis.Temporal` and prefer it when present (Chromium ≥144).
- **No Rust / no Tauri command:** calendar conversion is a pure display transform of an ISO date, computed at render. Rule 8 doesn't apply (nothing to persist).

## 4. UX (primary + secondary; per-locale defaults)
A new **Calendar settings** section (in Settings) with:
- **Primary calendar** (switches the whole grid = standalone). Default seeded per-locale, overridable.
- **Secondary calendar** ("show alongside" — a small second date in each cell/header = integrated). Default `none`.
- The existing **daily-note** controls grouped here: format, folder, template. + **Week start** (Sun/Mon/locale).
- **Per-locale defaults** (suggestion seeded from the active UI locale, never a lock): `ar`→primary `hijri`; `fa`→`solar-hijri`; `he`→`hebrew`; others→`gregorian`.
- **Numerals:** Intl `numberingSystem` — `ar`→`arab` (٠١٢), `fa`→`arabext` (۰۱۲) — *different glyphs*; never hardcode digit substitution. **Grid direction** from the UI locale via the existing `detectDir()` — NOT coupled to the calendar choice.

## 5. Storage rule (hard invariant)
Daily-note **filenames stay Gregorian ISO `YYYY-MM-DD`** — always (File-Over-App + sync stability; reuses the §D-1 `get_daily_note_path(date)` param, which already takes `YYYY-MM-DD`). The cultural date is **display-only**; optionally recorded as a non-authoritative `hijri:`/`jalali:`/`hebrew:` frontmatter field (display sugar, never the key). A click on a cultural-calendar cell resolves that cell → its Gregorian ISO date → `openDailyNote(isoDate)`.

## 6. Code shape
- **`CalendarPanel.svelte` rewrite:** `calendarDays` (currently JS-`Date` Gregorian math, `:47–102`) → build the 6×7 grid in the **primary** calendar via Temporal (month length from `daysInMonth`, first-cell weekday from `dayOfWeek`), each cell carrying BOTH its display label (primary) + its Gregorian ISO `dateStr` (for `onDayClick` + the note/task dot lookups, which key on ISO). Month name + weekday headers via `Intl` in the primary calendar + locale. Optional secondary-date label per cell. `weekStart` shifts the column order. Props gain `primaryCalendar`, `secondaryCalendar`, `weekStart`, `numberingSystem` (or read settings directly).
- **`openDailyNote`** (`+layout.svelte`): unchanged — already takes a Gregorian ISO `dateStr`. The grid hands it ISO regardless of the display calendar.
- **Settings:** new `appSettings.calendar.{ primarySystem, secondarySystem, weekStart }` (or top-level `calendarPrimarySystem` etc. to match the flat `dailyNote*` style — TBD in Plan); group the existing `dailyNoteFormat/Folder/Template` under the new Calendar settings UI section. Per-locale default resolved at first run / when unset.
- **Temporal loader:** a lazy `import('@js-temporal/polyfill')` (or native) inside the Calendar chunk; a tiny `calendarMath` helper module wrapping it.

## 7. Invariants (Audit will verify)
- **INV-1 — Filename stability:** daily-note filenames are Gregorian ISO under every calendar setting. Switching calendars never renames/re-keys a note. (Editor-Surface-adjacent: the create path is the §D-1 `gate_create_exclusive` with an ISO date.)
- **INV-2 — Correct grid math:** Hijri/Hebrew/Persian months show correct lengths + first-weekday (Temporal, not `Date`). Today's cell is correct in every system.
- **INV-3 — Multilingual ×15 + RTL + numerals:** all new strings in 15 locales (native equivalents); grid direction from `detectDir()`; correct numbering system per locale (`arab` vs `arabext`). No hardcoded English.
- **INV-4 — Perf:** Temporal polyfill lazy-loaded into the Calendar chunk only — zero boot/editor-hot-path cost (Perf Rules 3/6). Calendar open stays snappy.
- **INV-5 — Note/task dots still correct:** the highlighted-event dots key on the Gregorian ISO date (the scan is ISO-based), independent of the display calendar.
- **INV-6 — No regression to §A/§A.2:** the launcher + the full-page Calendar still work; "today" + day-click → daily note unaffected.

## 8. Migration-path / edge cases (Audit)
- Existing users (no calendar settings saved) → default `gregorian` primary (or the per-locale seed) — identical to today's behavior. No migration breakage.
- Native Temporal absent (older WebView2) → polyfill loads. Native present → used.
- Hijri ±1-day vs local moon-sighting: documented; not a religious-sighting authority (help-text note).
- Hebrew leap month (Adar I/II): read month count from Temporal, never assume 12.

## 9. Proposed phasing
- **§A — Temporal infra + `calendarMath` helper** (lazy loader, feature-detect, the 4 system mappings) + unit-ish verification (known-date conversions).
- **§B — Settings:** new Calendar settings section (primary/secondary/week-start + grouped daily-note format/folder/template) + schema + per-locale defaults + ×15 i18n.
- **§C — `CalendarPanel` rewrite** to render the primary calendar via Temporal (grid math, headers, today, numerals, RTL) + the §A.2 page + launcher consume the settings. Dots stay ISO-keyed.
- **§D — Secondary "show alongside"** label (integrated mode).
- **§E — Audit** (3 agents) + Editor-Surface check on the create path + Boss-test per system (Hijri/Persian/Hebrew/Gregorian) incl. RTL + numerals + filename-stays-ISO.

## 10. Open decisions for Boss
1. **Systems:** confirm the 4 (Gregorian, Hijri, Solar-Hijri/Persian, Hebrew) — matching the Sight vocabulary — and that Indian/Buddhist/Chinese are deferred.
2. **Default mode:** primary-switch (standalone) as the main control, with secondary "show alongside" optional? (Recommended — matches Google/Apple + your "integrated or standalone".)
3. **Per-locale auto-default** (ar→Hijri, fa→Persian, he→Hebrew) on first run — yes, or always default Gregorian and let the user choose? (Recommended: per-locale seed, overridable.)
4. **Secondary frontmatter** (`hijri:` etc.) written into new daily notes — include, or display-only (no frontmatter)? (Recommended: display-only first; frontmatter later if wanted.)
