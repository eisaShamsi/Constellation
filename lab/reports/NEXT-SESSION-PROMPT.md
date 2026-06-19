# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session that picks up MIG-081's 4 new Calendar functions. Copy everything in the box.

---

Working on: MIG-081 — the Calendar's 4 new functions (Boss-requested 2026-06-17), then the rest of MIG-081 + the MIG-080 right-rail cascade.

First read docs/Constellation Orientation & Onboarding v2.90.md, then lab/reports/HANDOVER-2026-06-17-mig081-rich-calendar.md, then lab/reports/SESSION-LOG-2026-06-17.md.

Context (all SHIPPED + on main, through 9e6ae75f): MIG-079 §C.2d (Sky read off boot, validated); MIG-080 §A/§A.2 (Calendar → left dock launcher + full-page view, validated); MIG-081 §A/§B/§C/§C.2a/§C.2b — Eisa's astronomical Hijri engine vendored (src/lib/calendar/hijri.js, pinned @a06be50e) + @js-temporal/polyfill + calendarMath.ts (buildRichMonthGrid) + a Calendar Settings category + a RICH CalendarPanel ported from Eisa's hijri-calendar app (ornate header, gold AH/sacred pill, Gregorian-range subtitle, dual-date cells, moon glyphs, event dots, Wk column). Boss: "Perfect!" Cultural calendars work; daily-note filenames stay Gregorian ISO.

THE TASK — Eisa's 4 new asks (in order):
1. Month Correction (±1 day per Hijri month + corrections list + Clear All). The engine ALREADY exposes setCorrection/getCorrection/clearCorrections/getAllCorrections — wire them into the Calendar page (correct the viewed month) and/or Calendar settings. Decide localStorage (engine's own) vs appSettings mirror (cross-device) — Boss call.
2. Calculation Mode selector — Astronomical (Lunar Conjunction) ⟷ Tabular (al-Tawfīqāt al-Ilhāmiyyah). Engine default 'astronomical'. VERIFY the public mode setter in the vendored hijri.js (internal currentMode + localStorage 'hijri-mode'; confirm/add an exported setter). Put it in Calendar settings.
3. Style Setter → new "Calendar" tab — wire the --cal-* CSS variables already in CalendarPanel.svelte into the Style Setter catalog + apply path, as a Calendar category. HONOR the full-center-zone preview rule (CLAUDE.md). (This was the planned §C.2d.)
4. Retire the Daily-Note dock button — the Calendar fully serves daily notes. Remove the §A launcher dock button + popover (+layout.svelte showDailyLauncher/.daily-launcher-*); KEEP handleOpenDailyNote/openDailyNote (command palette + the Calendar page use openDailyNote). Left dock = the Calendar button only.

Then: MIG-081 §C.2e (Amiri/Cairo fonts + ×15 i18n for settings.calendar.*/calendarPanel.weekAbbrev + 3-agent audit), then the MIG-080 §B–§F right-rail note-context cascade (Plan approved: docs/MIG-080-Plan.md) — Tags→Dashboard, Tasks/Source/Health/Review splits, + the deferred inspector360 Settings-UI bug.

Standing orders that bit last session — honor them: measure-don't-guess; WA#5 cross-check (and check Eisa's own repos/intent first — he has his own engines); SO #8 cross-check a deferred item against orientation BODY + session logs before tackling; Stop-On-Correction; the i18n EN-key gotcha ($t returns the key for missing → always add EN keys); Edit-tool whitespace mismatches on tab files → use Python string-replace; test instructions LITERAL (exact click/type + expected); git pull first; close the app before cargo build --release; frontend change → npm run build THEN cargo, grep build/ for a new string. Do the full closing PCS + handover + next prompt at session end.
