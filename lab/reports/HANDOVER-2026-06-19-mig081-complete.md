# Handover — next session (MIG-081 COMPLETE → MIG-080 §B–§F right-rail cascade)

**Prepared:** end of session 2026-06-19. **Branch:** `main` (all pushed). **Active universe:** "Eisa Cognitive Knowledge" (~7,650 notes, 1.97 GB). **Binary rebuild:** frontend change → `npm run build` THEN `cargo build --release` **from `src-tauri/`** (Cargo.toml lives there — running cargo from the repo root fails); close the app first (it locks the .exe); a frontend change → grep `build/` for a new string to confirm re-embed.

**Read first:** the latest orientation (`docs/Constellation Orientation & Onboarding v2.91.md`), then `lab/reports/SESSION-LOG-2026-06-19.md`, then this file. Plan for the next work: `docs/MIG-080-Plan.md` (+ `docs/MIG-080-Architect-Right-Sidebar-Note-Context.md`).

## SHIPPED + Boss-validated this session (MIG-081 COMPLETE)
All four Boss asks (2026-06-17) + two refinements + a per-element redesign, each behind a staged Boss test:
- **§C.2f — Month Correction + Calculation Mode** in Settings → Calendar → "Hijri calendar (Islamic)". Calc-method select (Astronomical/Tabular); month-correction editor (year/month + offset default **0**, ordered +2/+1/0/−1/−2, reflects the chosen month's existing value; removable list; Clear all). Stored in **appSettings (synced)**, pushed into the singleton engine via `calendarMath.applyCalendarPrefs`. Boss: "All pass."
- **§C.2d — Style-Setter "Calendar" category.** `--cal-*` moved to inline `var(--x,default)` fallbacks; a `calendar` element + category (three-zone) + a full-center-zone live preview of the REAL CalendarPanel. **Per-element colour + size** (font + 23 colours + 9 size tokens = 33 controls). Boss: "What a milestone. All passed perfectly."
- **§C.2g — Daily-Note launcher dock button RETIRED** (Calendar serves daily notes; `openDailyNote`/`handleOpenDailyNote` kept for the command palette + the Calendar page).
- **Subtitle = the cross-reference calendar** (refinement): Gregorian primary → Hijri range; non-Gregorian → Gregorian range (`subtitleRange` + `hijriRange()`).
- **§C.2e close-out:** Fonts — Amiri already bundled + `@font-face`'d (no code). **i18n ×15 COMPLETE** — `settings.calendar` (34) + `calendarPanel.weekAbbrev` + the 34 StyleSetter Calendar labels, all 14 non-EN locales (native); orphan `ribbon.dailyNote` removed ×15. **3-agent migration audit: invariants 7/7, migration 5/5, no P0/P1**; 3 P2s fixed/documented.

Files: `store.ts` (+2 appSettings fields), `calendarMath.ts` (applyCalendarPrefs/hijriMonthNames/hijriRange/subtitleRange), `CalendarPanel.svelte` (props + var refactor + per-element sizes), `SettingsModal.svelte` (Hijri subsection), `StyleSetter.svelte` (calendar element/category/preview), `+layout.svelte` (launcher removal + prefs prop), 15 i18n JSONs. No Rust.

## NEXT — MIG-080 §B–§F (the right-rail note-context cascade; Plan approved `docs/MIG-080-Plan.md`)
The right sidebar becomes the OPEN NOTE's context only; universe functions relocate. §A/§A.2 already shipped (Calendar→left). Remaining:
- **§B** Tags "All tags" → Dashboard (reuse §C.1 `tag_counts`).
- **§C** Tasks split (contextual right-rail open-note tasks + left agenda; `toggle_task` reindex-gate fix).
- **§D** Source Review split (note-scoped right rail + universe Cataloger on the left).
- **§E** Knowledge Health split (note tensions right rail | universe → Dashboard).
- **§F** Review Pulse split (note review status right rail | universe → full-page reviewer; `record_note_visit` fix).
- **§G** reconcile + 3-agent audit + the **deferred inspector360 Settings-UI bug** (missing from the Panels placement list; needs ×15 i18n) + ×15.

## Standing reminders that bit this session
- **cargo runs from `src-tauri/`** (root has no Cargo.toml) — I burned one build on this.
- **Workflow `args` must be an OBJECT, not a top-level ARRAY** (a `["a","b"]` arrives non-iterable → `args.map` throws). Hardcode lists in the script body, or wrap in `{list:[...]}`.
- **Structured-output schemas:** a 35-named-property object schema came back numeric-keyed; an **array schema** is cleaner but does NOT guarantee order (one locale block-rotated 5 items) → always **verify order** (size-label parity check) before mapping; or use `{english,translation}` pairs (self-verifying). Never positionally-map an LLM array without verification (fabrication-class risk).
- **i18n writer:** `json.dump(ensure_ascii=False, indent='\t')+"\n"` round-trips the locale files losslessly → diff = additions only. Format tokens (`%Y-%m-%d`) must survive translation; date-pattern notation (YYYY-MM-DD) is localizable per the full-localization principle.
- **Full-localization is a TOP PRINCIPAL** — the audit caught that StyleSetter labels (not just Settings) needed ×15. When adding ANY new UI labels, add them to `styleSetter.labels` / the relevant i18n block ×15, not just EN.
- Staged Boss tests (one stage, wait, next); Stop-On-Correction (the master-Text-size → per-element pivot); measure-don't-guess (Node-verified the cross-ref range + the engine correction/mode flow).
