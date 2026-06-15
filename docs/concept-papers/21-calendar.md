# 21 — Calendar (Concept Paper)

> One of the right-sidebar panels. Attaches to the Editor (the gate) by opening a daily note as a tab. Phase 5 (Knowledge curation & analysis) in the [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) bring-up.

## 1. Function in hand
The **Calendar** panel — `src/lib/components/CalendarPanel.svelte`, the right-sidebar tab (`rightSidebarTab === 'calendar'`, button title `panels.calendar`) that renders a month grid and, on a day click, opens that day's **daily note**. Mounted and wired in `src/routes/+layout.svelte` (~`:6998`).

## 2. Purpose
Give a **temporal entry point** into the Universe: pick a date, land in its daily note. The ONE question it answers: *"what did I think on this day — take me there."* It serves the **Observation** Act — the daily note is the capture surface where raw observations enter before they become linked knowledge. The dot markers (notes-on-this-date, tasks-due-on-this-date) are a *navigational hint*, not a derived knowledge view. Justification (Constraint as Design): thin but real — the daily-note pattern is a load-bearing PKM idiom (Obsidian/Logseq), and Constellation already ships `get_daily_note_path` + `dailyNotes` settings; the panel is the date-shaped front door to that file. If the dots cannot be made cheap (see §7), the panel still justifies itself as a pure date→daily-note launcher *without* the scan.

## 3. What it is NOT
- **Not** a knowledge-derivation surface — it does not compute backlinks, tags, or graph; the dots are decoration over a filesystem date scan.
- **Not** an events/agenda/reminder calendar — there is no event model, no recurrence, no notifications.
- **Not** a task manager — task-due dots are read-only mirrors of the Tasks subsystem (`scan_library_tasks`); editing tasks lives in the Tasks panel (#21 in the inventory).
- **Not** a multi-calendar (Hijri/lunar) surface today — `Intl.DateTimeFormat` localizes labels, but the grid is Gregorian, Sunday-first.

## 4. Wiring
- **Inputs (props/stores read):** `noteDates`, `taskDueDates` (passed down as `calendarNoteDates` / `calendarTaskDates`, `$state` in `+layout.svelte:682-683`); `$t`, `$dir` (i18n). The counts are filled by an `$effect` (`+layout.svelte:1477`) that, while the panel is visible, debounces 200 ms then calls **`scanLibraryNoteDates(path, name)`** + **`scanLibraryTasks`** across every library in `get(libraries)`.
- **Outputs (IPC, on day click):** `get_daily_note_path(libraryPath, format=dailyNoteFormat, folder=dailyNoteFolder)` → then `openNoteTab(dailyPath, lib.name, color)`. Uses **`libraries[0]`** (first library) as the daily-note home.
- **Inputs (IPC, for dots):** `scan_library_note_dates` (Rust `tasks.rs:476`, full `scan_dates_recursive` walk; date source = filesystem **`modified`** timestamp) and `scan_library_tasks`.
- **Consumers:** none downstream — the Calendar is a leaf consumer. Its only effect on the rest of the system is opening a tab in the Editor.
- **Connection to the Editor (the gate):** day click → `onDayClick(dateStr)` → resolve daily-note path → `openNoteTab(...)`. It attaches by **handing a path to the Editor**; it never reads/writes note content itself. **CONFIRMED BUG (verified in `libraries.rs:4318-4346`):** `get_daily_note_path` derives the filename from `chrono::Local::now()` and takes **no date parameter** — the clicked `dateStr` is computed in `onDayClick` and then **discarded**. Clicking *any* day always opens/creates **today's** daily note. This breaks the panel's core promise ("take me to this day"). Must be fixed for §10's first checkbox.

## 5. Right-click / context menu
- **Has one? NO.** The day cell is a `<button onclick={...}>` with only an `onclick`; grep finds **no** `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` in the component.
- **SHOULD it? YES — flagged gap.** `lab/reports/MIG-077-RIGHTCLICK-CONTEXT-MENUS-ARCHITECT.md:29` explicitly lists **"Calendar day cells (`CalendarPanel.svelte`)"** under *"Genuinely missing (verified)"*. Natural right-click actions on a day: *Open daily note · Create daily note here · Reveal in file tree · Copy date · (if dots) list the N notes / M tasks on this day*. None are reachable today.
- **Bring-up action:** add a right-click menu via the **shared `<ContextMenu>` / `buildContextMenu()`** (MIG-077) — **not** hand-rolled. No action is reachable only by right-click today (because there is no menu), so there is nothing to preserve — just add it correctly.

## 6. Multilingual
- **Chrome strings:** localized — `calendarPanel.today` / `prevMonth` / `nextMonth` flow through `$t()` (keys present in all 15 locales: ar de en es fa fr he hi ja ko pt ru tr ur zh). Panel-tab title uses `panels.calendar`.
- **RTL:** handled — root `<div class="calendar-panel" dir={$dir}>`; the grid is `display: grid`, so column order follows `dir`. (Chevrons are literal `‹` / `›` glyphs that mirror naturally under RTL.)
- **Date/weekday labels:** localized via `Intl.DateTimeFormat(undefined, …)` (month name + narrow weekday) — picks up the OS/app locale, native month names included.
- **Hardcoded-English gap — FLAGGED:** the day-cell tooltip is `title={cell.noteCount > 0 ? \`${cell.noteCount} notes\` : ''}` (line 156) — a **hardcoded English "notes"**. The locale files already define `calendarPanel.notesCount` (`"{count} notes"`) and `calendarPanel.tasksCount` — they exist but are **not wired**. Bring-up: replace the literal with `$t('calendarPanel.notesCount', { count })` and add the tasks tooltip. Also: `calendarPanel.createDailyNote` exists in locales but no element uses it yet (reserved for the missing right-click menu).

## 7. Boot behavior
- **Runs at boot? NO** — the dot-scan `$effect` is guarded by `rightSidebarOpen && rightSidebarTab === 'calendar'`, so nothing fires unless the user opens the Calendar tab. It is **on-demand**, not a boot IPC.
- **Rule 8 status: RECOMPUTES-on-read (VIOLATION).** Every time the panel becomes visible it re-walks **every library's whole tree** via `scan_library_note_dates` (`scan_dates_recursive`) to count notes-per-date from filesystem `modified` times, plus `scan_library_tasks` for due-dates. This is exactly the `scan_*_library` shape Rule 8 forbids ("re-walks the Universe to produce a derived view"). The 200 ms debounce caps re-fire rate but not the cost per fire. The correct end-state: persist per-date note/task counts (maintained on the write path, like `tag_counts` in MIG-079's plan) and have the panel read them.
- **Cost:** **estimated, not measured.** On the 7,653-note baseline a full `scan_dates_recursive` + `scan_library_tasks` per library is plausibly hundreds of ms to low seconds on first open (file-stat-bound). Measure on the live Universe before re-enabling.

## 8. Flag / gate & bring-up position
- **Gate today:** the day-click action is gated by `enabledFeatures.dailyNotes` (default `true`, `store.ts:~3531/3840`); the panel's *presence* is a sidebar-tab placement (`rightSidebarTab === 'calendar'`), not its own `enabledFeatures` boolean. The dot-scan has **no independent flag** — it rides the tab-visibility `$effect`. Inventory row: **#22, gate `(panel) · dailyNotes`**.
- **Bring-up phase:** **Phase 5 — Knowledge curation & analysis.** Depends on: the Editor + Tab bar (to receive the opened daily note), the Tasks subsystem (`scan_library_tasks` for due-date dots), and — for Rule-8 compliance — a persisted per-date count source (MIG-079-style write-time maintenance). Re-enable the date→daily-note launcher first; gate the dots on the persisted-count fix.

## 9. Budget
- **Boot budget:** **0 ms** — must not touch boot at all (already on-demand; keep it that way).
- **Interaction budget:** day click → daily note open ≤ Editor's per-tab open (~1–3 ms disk + mount). Panel-open dot-scan target: ≤ a few ms once reading persisted counts; the current full-tree scan must not block paint of the grid (grid should render immediately; dots fill in async — which it does).
- **Regression guard:** open Calendar on the 7,653-note Universe and measure time-to-dots before/after the WTD fix; confirm month-nav (`prev`/`next`) never re-scans (it doesn't — `viewMonth`/`viewYear` are pure local `$state`, no IPC).

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** clicking a day opens *that day's* daily note — **today blocked by the confirmed §4 bug** (`get_daily_note_path` ignores the date); fix first; grid renders instantly.
- [ ] **Serves Constellation's core purpose:** the daily note it opens is a real Observation-capture surface feeding the Five Acts ([00-Constellation](00-Constellation-Core-Concept-Paper.md)).
- [ ] **Wires correctly to the Editor:** day click opens exactly one tab via `openNoteTab`; no content read/write inside the panel.
- [ ] **Right-click present + correct:** day-cell menu added via shared `<ContextMenu>`/`buildContextMenu` (MIG-077), **not** hand-rolled; items (open/create daily note, reveal, copy date) work.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** the `\`${count} notes\`` literal is replaced with `$t('calendarPanel.notesCount', …)`; task tooltip localized; grid mirrors under RTL; native month/weekday names render.
- [ ] **Within budget:** zero boot cost; dot-scan does not stall the grid; month-nav is IPC-free.
- [ ] **Obeys Rule 8:** dot counts read from a **persisted, write-time-maintained** source — no `scan_*_library` re-walk on panel open.
- [ ] **Holds its invariants:** read-only over notes/tasks (never mutates content); opening a daily note creates it via the sanctioned `get_daily_note_path` path only.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—**
Notes: Two concrete debts to clear before re-enable — (1) **Rule 8 violation:** `scan_library_note_dates` + `scan_library_tasks` re-walk every library on panel open; fold the per-date counts into a write-time-maintained source (MIG-079 family). (2) **Right-click gap:** Calendar day cells are on MIG-077's "genuinely missing" list — add the shared context menu. (3) **Confirmed functional bug:** every day click opens *today's* daily note, not the clicked day's — `get_daily_note_path` (`libraries.rs:4318`) uses `chrono::Local::now()` and ignores the date; the panel computes `dateStr` then drops it. Minor: wire the unused `calendarPanel.notesCount`/`tasksCount`/`createDailyNote` locale keys (a hardcoded English `"notes"` tooltip exists today).
