# MIG-082 — Architect: Clickable Calendar + 4 New Calendars + Open-at-Line

**Opened:** 2026-06-19. **Status:** Architect (Phase 1) complete; Plan in `docs/MIG-082-Plan.md`; pending Boss approval before build.
**Function in hand:** the full-page **Calendar** (`CalendarPanel.svelte`, mounted from `+layout.svelte`'s `.calendar-overlay`) — MIG-081 shipped its rich rendering; MIG-082 makes its day cells *interactive* and adds four calendar systems.

Source: 4-agent Architect workflow (`wf_f0c6d756-1f3`) building on the verified recon (`wf_c51636b5-6f9`).

## Boss decisions (2026-06-19)
1. **Calendars:** add Indian + Buddhist **and** Chinese + Korean (incl. the hard lunisolar leap-month handling).
2. **Note dot:** any note edited that day, but **visually distinguish the daily note from the other (conventionally-created) notes** — by colour.
3. **Dots clickable:** note/task dot → open that specific item (popover when several); **empty cell space → create/open the daily note**.
4. **Task dot:** open **and** jump to the task's line to add/edit. Event dot = static tooltip only. **Filenames stay Gregorian ISO.**

## The duplicate question — answered (verified, no code change needed)
Clicking a date whose daily note exists **OPENS it; never duplicates.** Filename is deterministic per Gregorian ISO (format + folder); Rust guards `if !exists` + the write gate refuses-on-exists; the tab layer reuses the active tab. (Caveat: changing the daily-note folder/format later resolves old dates to a new path → a fresh file there; config-relocation, not a same-config duplicate — a future migration if ever needed.)

## Key empirical finding (Node-verified against the installed `@js-temporal/polyfill@0.5.1`)
- **persian, hebrew, indian, buddhist** all work via the polyfill (12-month solar; `Intl` long names — "Jyaistha 1947 Śaka", "June 2568 BE"). → **Indian + Buddhist are the SMALL path** (reuse the existing Temporal branch).
- **chinese & dangi (Korean)** — the polyfill **THROWS on every op** ("Unexpected leap month suffix: Mo2bis"): a known polyfill-vs-newer-ICU break. But the **host `Intl` (ICU)** fully supports them incl. leap months (en "Second Monthbis", zh "闰二月", ko "윤2월"); a complete leap-month grid was driven from `Intl.formatToParts` + ISO day-walking (the existing `isoShift`), correctly detecting the 2023 leap-2 boundary at 29 days. → **Chinese/Korean get an Intl-only lunisolar branch** — no Temporal, self-contained, cannot regress the 4 shipped systems. (Supersedes the WA#5 agent's web-guess that suggested Temporal for chinese/dangi + a bespoke Indian converter — the Node run is authoritative.)

## Design

### Part A — Clickable cell + dots (frontend; CalendarPanel + +layout)
- **Cell restructure (a11y constraint: no nested `<button>`).** The cell becomes a `role="gridcell"` container holding: a full-bleed transparent **background button** (`inset:0; z-index:0`) → `onDayClick(iso)` (empty-space → daily note); the day-number/moon/sub-date as `pointer-events:none` overlay (clicks fall through to the bg); and **per-dot `<button>`s** (`z-index:2`, `stopPropagation`) → open the item / popover. Dot hit-area ≥14px (transparent padded button, visible 6px inner dot). RTL via `inset:0` + logical positioning.
- **Keep per-item arrays** (Rust already returns them; the count-collapse at `+layout.svelte:1755`/`:1766` is the only thing discarding them). New state `calendarNoteEntries: Record<string, NoteDateEntry[]>` + `calendarTaskEntries: Record<string, TaskItem[]>`; **de-dup NoteDateEntry by `file_path` per date** (the scanner emits up to 3 entries/file: modified + frontmatter `date:` + `created:`). The count badge = `entries.length`.
- **Daily-note distinction:** add a Rust flag **`is_daily: bool`** to `NoteDateEntry`, computed by the same logic `get_daily_note_path` uses (rendered `dailyNoteFormat` stem + `dailyNoteFolder` match) — NOT a frontend strftime re-implementation (reuse rule). New Style-Setter tokens **`--cal-daily-dot`** (distinct colour, default gold to echo "today") + **`--cal-dot-size`** (themeable). Daily-note dot rendered distinctly.
- **Popover** (multiple items): cap visible dots ~2-3, overflow → a popover (FullCalendar's default pattern) listing the date's notes (daily note pinned/highlighted) + tasks; each row wires its OWN click handler (popover clicks don't reliably bubble). Single item → open directly (no popover). Reuse the shared task-row renderer where possible.

### Part B — Open-at-line + task add/edit (editor open path — Editor-Surface Gate applies)
- **Open-at-line:** extend `openNoteTab` with an optional trailing `{ targetLine }` (do NOT reorder its 6 positional params). It reuses the EXISTING cursor-placement site (the `initialCursorPos` mount block in NotePane) — line→offset via `view.state.doc.line(n).from` (1-indexed, clamped) + `EditorView.scrollIntoView`. The already-open-same-note case = an imperative branch that verifies the live view is the right note before dispatching (else bump `reloadVersion` for a clean remount). **Line jump = a selection dispatch only — never `{changes}`, never a save** (Gate #2).
- **Reuse:** route the existing `GlobalTasksView.openTask` through the same capability so the Tasks panel ALSO jumps to the line (one source of truth).
- **Task toggle/append + single-ownership reconciliation (the landmine):** `toggle_task` writes to disk. If the note is OPEN, the in-memory single-ownership model must adopt the change via **`reloadTabsFromDisk`** — else the next debounced save reverts the toggle. (This also fixes a *latent* bug: `GlobalTasksView.handleToggle` doesn't reconcile today.) **Add a task** = a new gate-write Rust `append_task_line(file, text, due_date)` → writes `- [ ] {text} 📅 {YYYY-MM-DD}` → `reloadTabsFromDisk`. **Destination = the daily note for that date** (deterministic, collision-free).

### Part C — Calendars
- **Indian + Buddhist (SMALL):** add to `CalendarSystem`, `TEMPORAL_CAL`, the `ensureCalendarEngines` Temporal predicate, the Settings options, i18n names ×15, era suffix — the generic Temporal branch handles the grid/step/today unchanged. **+ fix the Persian double-era glitch** (build `monthLabel` as name+year, control the suffix ourselves) — fold in so Indian doesn't reproduce it.
- **Chinese + Korean (MEDIUM, Intl-only branch):** a new lunisolar path in `calendarMath` keyed off `Intl.DateTimeFormat(calendar:'chinese'|'dangi')` + ISO day-walking; leap-month-correct (a leap month is its own page with its native label; prev/next neither skips nor duplicates it). Month names + leap labels come from `Intl` (never hand-rolled "闰"/"bis"). Navigation routes through a relatedYear+ordinal model, never integer month±1. Korean year display = a Boss decision (Dangi era 4357 vs Gregorian-year + Korean names).

## Options (effort / risk)
| Area | Recommended | Effort | Risk |
|---|---|---|---|
| Cell restructure + per-item arrays + daily-note flag + popover | Option D | Medium | Low-Med (pure UI; a11y care) |
| Open-at-line via `openNoteTab {targetLine}` + shared `goToLine` | Option A | Medium | Low-Med (Editor-Surface Gate; reuses proven cursor site) |
| Task toggle/append + `reloadTabsFromDisk` reconcile | Option C | Medium | Med (single-ownership; must reconcile) |
| Indian + Buddhist (Temporal) + Persian era fix | — | Small | Very low |
| Chinese + Korean (Intl-only lunisolar branch) | — | Medium (~2-3 d) | Med (own verification surface) |

## Invariants (must not break)
1. Every `GridCell.iso` stays Gregorian ISO; grid stays 6×7; daily-note filename/dots/openDailyNote consume only `iso`. New calendars affect **display labels + month grouping only**.
2. The 4 existing systems render byte-identical after MIG-082 (Indian/Buddhist reuse the existing branch; Chinese/Korean are a separate branch → no regression by construction; snapshot before/after).
3. **Editor-Surface Gate** (all 8 checks + on-screen===disk) for every open-path/toggle/append change; **Reproduce-First** (svelte-check/vitest/cargo do NOT catch these — prove on the running app, esp. the open-note-task-toggle + linked-probe-pair recipes).
4. **Single content ownership:** any disk write to an open note routes through `reloadTabsFromDisk`; line-jump is selection-only (no spurious write).
5. Engines stay lazy (Perf 3/6); the calendar scan stays the on-open debounced read (Rule 8 — no boot re-walk); virtualize the popover if a date can exceed ~50 items.
6. Lunisolar navigation never uses integer month±1; month names always from `Intl`; all new strings ×15; RTL preserved.

## Open questions for the Boss (resolved as defaults unless changed)
- **Phasing:** split into **§A** (clickable dots + open-at-line + Indian/Buddhist + Persian fix) then **§B** (Chinese/Korean lunisolar)? *(Recommended — the lunisolar pair is a distinct verification surface that shouldn't gate the interaction work.)*
- **Korean year:** Dangi era (4357) vs Gregorian-year + Korean month names? *(Genuine cultural choice — ask.)*
- **Add-task destination:** the daily note for that date. *(Default — collision-free.)*
- **Daily-note dot distinction:** a dedicated colour token `--cal-daily-dot`. *(Default — matches "by colours" + Style-Setter themable.)*
- **Popover threshold:** single item → open directly; 2+ → popover. *(Default.)*
- **`calendarSecondarySystem`** (dead wiring): activate "show a second date alongside", or remove it? *(Ask — adjacent product decision.)*
- **Cultural-date frontmatter** (`hijri:`/`jalali:`/… in new daily notes; MIG-081 Boss decision #4, never implemented): do it in MIG-082 or defer? *(Ask.)*
- FocusPane line-jump scope; second-screen parity — handle by default (verify in build).
