# MIG-082 — Plan (phase-by-phase)

Architect: `docs/MIG-082-Architect-Calendar-Interactions.md`. Each step lands as one commit with a verification clause; Boss-testable steps pause for a staged test. **Plan-approval = build-approval** (cascade; stop only at Boss-test points + genuine architectural surprise).

## §A — Clickable calendar + Indian/Buddhist (the interaction win + the easy calendars)

**§A.1 — Cell restructure + per-item dots + daily-note distinction (frontend + a tiny Rust flag).**
- Rust: add `is_daily: bool` to `NoteDateEntry`; `scan_library_note_dates` takes `daily_format`+`daily_folder` and sets it (the `get_daily_note_path` match). (Additive; existing callers pass the values or get `false`.)
- `+layout.svelte`: keep `NoteDateEntry[]`/`TaskItem[]` per date (stop the count-collapse); de-dup notes by `file_path` per date.
- `CalendarPanel.svelte`: cell → `role="gridcell"` container = full-bleed bg button (empty→`onDayClick`) + `pointer-events:none` number overlay + per-dot `<button>`s (`stopPropagation`); daily-note dot distinct via `--cal-daily-dot`; `--cal-dot-size`; ≥14px hit areas; RTL; keyboard reachable. A **popover** for 2+ items (single → open directly).
- Style-Setter: add `--cal-daily-dot` + `--cal-dot-size` to the calendar category (+ EN label; ×15 in §C).
- **Verify (Boss):** clicking empty space opens/creates the daily note; a note dot opens that note (popover when several; daily note distinct colour); counts still correct; RTL ok. **Editor-Surface Gate light** (no content/save change — notes open via the existing path).

**§A.2 — Open-at-line (task dot → open at the task's line).** *(Editor-Surface Gate — full.)*
- `openNoteTab` gains optional `{ targetLine }`; reuse the NotePane `initialCursorPos` mount site (line→offset `doc.line(n).from`, clamped) + `scrollIntoView`; already-open-same-note → imperative branch (verify the live view) with `reloadVersion` fallback. Line jump = selection-only (no `{changes}`, no save).
- Wire task-dot click → open at `task.line_number`; route `GlobalTasksView.openTask` through it too (reuse).
- **Verify (Boss):** clicking a task dot opens the note at the task line; the Tasks panel also jumps to the line; **the full Editor-Surface Gate** (type-burst persists, Focus round-trip, tab switch, rename probe-pair, body intact) — proven on the running app (Reproduce-First).

**§A.3 — Task toggle/add from the calendar + single-ownership reconcile.** *(Editor-Surface Gate — full.)*
- Toggle complete from the task popover (reuse `toggle_task`) → **`reloadTabsFromDisk`** if the note is open (also fixes the latent `GlobalTasksView.handleToggle` gap). "Add a task for this date" → new gate-write Rust `append_task_line(file,text,due_date)` → daily note → `reloadTabsFromDisk`.
- **Verify (Boss):** toggling a task on an OPEN note persists (no revert on next save); adding a task lands in that date's daily note; gate clean.

**§A.4 — Indian + Buddhist + Persian era fix.**
- `calendarMath`: add `'indian'|'buddhist'` to the type + `TEMPORAL_CAL` + `ensureCalendarEngines`; era suffix; **fix the Persian double-era** (`monthLabel` = name+year, controlled suffix). Settings options; i18n system-name keys.
- **Verify (Boss):** switch to Indian / Buddhist → correct native month names + grid; Persian pill no longer shows "AP SH"; the 4 existing systems unchanged.

## §B — Chinese + Korean (lunisolar, Intl-only branch)

**§B.1 — The lunisolar branch.**
- `calendarMath`: a new Intl-driven path for `'chinese'|'dangi'` (no Temporal): ISO-anchored month boundaries + leap-month-correct grid/step/today via `Intl.formatToParts` + `isoShift`; native leap labels (闰二月 / 윤2월) from `Intl`. Settings options; i18n; Korean year per Boss choice.
- **Verify (Boss):** Chinese + Korean render with correct (incl. leap) months; prev/next traverses a 13-month year without skip/dup; the 6 other systems unchanged. (Node + on-app checks against the 2023 leap-2 boundary.)

## §C — Close-out
- `calendarSecondarySystem`: per Boss — activate (show a second date alongside) or remove the dead wiring.
- Cultural-date frontmatter (`hijri:`/`jalali:`/…): per Boss — implement (Rust `get_daily_note_path` cultural-date param + write) or defer.
- ×15 i18n for all new strings; 3-agent migration audit (invariants / drift / migration-path + the full Editor-Surface Gate); `/simplify`; full PCS (orientation v-bump, session log, MoCh, handover, manual).

## Defaults locked unless Boss changes
Add-task → the daily note; daily-note dot → a colour token; popover only for 2+; new calendars are lazy; filenames stay Gregorian ISO.
