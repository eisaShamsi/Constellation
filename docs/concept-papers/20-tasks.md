# 20 — Tasks & Global Tasks (Concept Paper)

> Per-function paper. Must trace to [00-Constellation](00-Constellation-Core-Concept-Paper.md) (the Five Acts; File-Over-App; Rule 8; the Editor is the gate) and attach to [01-Note-Editor](01-Note-Editor.md).

## 1. Function in hand
**Tasks** — two surfaces over the same data: the **Tasks** right-sidebar panel (`src/lib/components/TasksPanel.svelte`, scoped to the *active note*) and **Global Tasks** (`src/lib/components/GlobalTasksView.svelte`, a full-page view across *all libraries*). Both render Markdown checkbox lines (`- [ ]` / `- [x]`) parsed in Rust (`src-tauri/src/tasks.rs`). Right-sidebar tab id `tasks`; full-page command `global-tasks` (`☑️`, View category).

## 2. Purpose
**Boss-ruled 2026-06-15: REFRAME (not remove).** Surface the **open epistemic loops** a note left behind — the `- [ ]` lines that, in a *formulation* system, are not chores but **unfinished knowledge work**: an unanswered question, an unverified claim, a source still to read, a commitment to investigate, a tension not yet resolved. Each loop is the *gap* between an Observation/Connection and a held position; closing it (`- [x]`) records that the gap was closed. So Tasks serves **Tension → Synthesis → Conviction** — it is the unfinished business of knowledge formulation, not a productivity to-do app, planner, or file-manager affordance (the framing the core paper rejects). Data unchanged (Markdown `- [ ]` with optional due `📅`/`due::`, priority `⏫🔼🔽`, tags, created/done dates); the reframe is a **purpose + naming** shift: the bring-up may surface it as **"Open Loops" / "Open Questions"** and bias its parsing toward `#question` / `#assumption` / `#claim` loops, making the formulation framing concrete. It stays a first-class function — its debt (Rule 8, gate-bypass, missing menu/gate) is paid down in its bring-up phase, not grounds for removal.

## 3. What it is NOT
- **Not** a task *database* — tasks are never stored; they are re-derived from `.md` text on demand (today). There is no task entity, no task ID, no persistence.
- **Not** an editor — it toggles one checkbox character via `toggle_task` and otherwise sends the user *into* the Editor (`openNoteTab`) to change task text.
- **Not** a planner/scheduler — no recurrence, no reminders, no notifications.
- **Not** a second domain — Global Tasks is a display; all writes go through the WriteGate, never a private save path.

## 4. Wiring
- **Inputs (IPC, read):** `scan_note_tasks(file_path, library_name, library_path)` — TasksPanel, the active note only; `scan_library_tasks(library_path, library_name)` per library — GlobalTasksView, fanned out with `Promise.all` over `get(libraries)`. Stores: `libraries`, `libraryColorMap`.
- **Outputs (IPC, write):** `toggle_task(file_path, line_number)` → flips `[ ]`↔`[x]`, stamps/strips `✅ YYYY-MM-DD`, and persists **through `crate::write_gate::gate_write` (MIG-076 §A2 — serialized, atomic, journaled)**. Returns the new file content.
- **Outputs (events):** on toggle, the host (`+layout.svelte`) patches the matching open tab's `content`, re-scans, and `broadcastNoteSaved(filePath)` to the second screen.
- **Consumers:** the open-tab list (content patch after toggle), the second screen, `CalendarPanel` (shares the sibling `scan_library_note_dates` scanner in `tasks.rs`).
- **Connection to the Editor (the gate):** **partial / out-of-band.** `toggle_task` writes the file in Rust *directly via the WriteGate*, bypassing the Editor's own `write_note` + reindex dispatch. The host compensates by hand-patching tab content and broadcasting — but search/backlinks/tags are **not** reindexed by a task toggle. This is a gate-bypass to resolve in bring-up: a checkbox flip is a content edit and should flow through the Editor's write path so downstream derivations stay consistent (per 00 §4, "no silent reads/writes").

## 5. Right-click / context menu
- **NONE.** Neither `TasksPanel.svelte` nor `GlobalTasksView.svelte` contains `oncontextmenu` / `contextmenu` / `<ContextMenu>` / `buildContextMenu`. The host tab button for `tasks` also has no context menu.
- Task actions exist but are reachable only by **left-click** (checkbox = toggle; file-link = open) and a **modifier-click** convention (`ctrl`/`meta`/middle-click on the file-link opens in a new tab). The "open in new tab" affordance is therefore *discoverable only by accident* — exactly the kind of action that should live on a right-click menu.
- **Gap to flag (debt):** per 00 §5, "right-click should include every aspect of the app." A task row should expose, via the shared `buildContextMenu()` (MIG-077), at minimum: Open / Open in new tab / Open in split, Toggle complete, Set priority, Set due date, Copy task text, Reveal in file. Building these hand-rolled would be a second violation — they must route through the shared `<ContextMenu>`.

## 6. Multilingual
- **Strings:** clean. Every user-facing string flows through `$t()` (`tasksPanel.*`, `globalTasks.*`); all 15 locale files present and the Arabic block (`src/lib/i18n/ar.json`) is genuinely translated, not English-stubbed.
- **RTL:** `GlobalTasksView` sets `dir={$dir}` on its root. **`TasksPanel` does NOT set `dir`** — verify in bring-up that the sidebar panel inherits direction correctly for Arabic/Hebrew/Farsi/Urdu task text and meta badges.
- **Date formatting:** `formatDueDate` builds strings like `"3d"`, `"2d "+overdue` by concatenating digits with a lowercased `$t()` fragment — borderline for non-Latin locales; verify the `Nd` / `Nd overdue` composition reads correctly in all 15 (candidate for a fully-interpolated key).
- **Parsing:** tag extraction includes `\p{Arabic}` in the Rust regex, so Arabic hashtags parse. Priority/due **emoji and `::` syntax are language-neutral**; no hardcoded English in the parser.

## 7. Boot behavior
- **Runs at boot?** Not at app boot directly. TasksPanel scans (debounced 100 ms) **whenever the Tasks sidebar tab becomes visible / the active note changes**; GlobalTasks scans **on every mount and every Refresh**.
- **Rule 8 status: ❌ RECOMPUTES-ON-READ — a violation to fix (the central finding).** There is **no persisted task index and no trigger**. `scan_library_tasks` walks the *entire library directory tree*, reads every `.md` file, and re-parses every line on each open — the exact `scan_*` anti-pattern Rule 8 forbids ("Don't write a `scan_*_library` … that re-walks the Universe to produce a derived view"). The right shape (per 00 §7 / CLAUDE.md Rule 8): a persisted `note_tasks` table maintained by the same write-path hook that fires the FTS5 reindex, with the panels doing cheap lookups; a resumable background back-fill for first-time population.
- **Cost:** `scan_note_tasks` is one file read + line parse — cheap (≈1–3 ms, estimated). `scan_library_tasks` is **O(all files in all libraries)** per open; the view surfaces its own `scanTime` in the header (e.g. `… · {scanTime}ms`) — **measure on the 7,600-note Universe in bring-up; unmeasured here**. This is the cost Rule 8 exists to eliminate.

## 8. Flag / gate & bring-up position
- **Gate today:** **none dedicated.** There is no `enabledFeatures.tasks`. The sidebar tab renders whenever the `tasks` panel placement is in the sidebar; `global-tasks` is always in the command palette (no `enabledFeatures` guard, unlike `constellationMap` / `ccs` / `index`). **Needs a new gate** (`enabledFeatures.tasks`) so it can be flipped off in minimal mode like its peers.
- **Bring-up phase:** **late (satellite).** Should remain **OFF** until (a) §2 self-justification is decided, (b) the Rule 8 persisted-index rebuild lands, and (c) the gate-bypass in §4 is resolved. Depends on: the Editor's write path (the gate), the `libraries` registry, the WriteGate, and `libraryColorMap`.

## 9. Budget
- **Boot budget:** **zero at boot** — nothing may scan before paint. Panel scans only on its own tab activation; Global Tasks only on explicit open.
- **Interaction budget:** opening the panel and toggling a checkbox must feel instant; once the persisted index lands, panel open is a single cheap lookup, not a tree walk. No `invoke()` on any keystroke path (the panels have none today).
- **Regression guard:** measure `scan_library_tasks` total time on the 7,600-note Universe before/after the Rule 8 rework; assert a task toggle still round-trips through the WriteGate and that on-screen === on-disk for the toggled line.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** parses `- [ ]`/`- [x]` with due/priority/tags; toggling completes in place and stamps the date.
- [x] **Serves Constellation's core purpose:** RULED 2026-06-15 — reframed as "open epistemic loops" driving Tension→Synthesis→Conviction (not a to-do app). Stays first-class; debt paid in bring-up.
- [ ] **Wires correctly to the Editor:** a toggle flows through the Editor's write path (or an equivalent that **also fires the reindex**) — no silent out-of-band write that leaves search/backlinks stale.
- [ ] **Right-click present + correct:** task rows expose actions via the **shared** `buildContextMenu()` / `<ContextMenu>` (MIG-077) — not hand-rolled; "open in new tab" no longer modifier-click-only.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** `TasksPanel` sets `dir`; date-composition strings read correctly in all 15; no hardcoded English (currently clean in strings).
- [ ] **Within budget:** zero scan at boot; `scan_library_tasks` time measured on the 7,600-note Universe and acceptable.
- [ ] **Obeys Rule 8:** tasks read from a **persisted** index maintained by a write-path hook/trigger — no tree-walk recompute on open; first-time back-fill is background + resumable.
- [ ] **Holds its invariants:** toggle is reversible; completion date added on check / removed on uncheck; trailing newline preserved; on-screen === on-disk after toggle; second screen stays in sync.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (unmeasured; Rule 8 rework required first)**
Notes: Two real components verified (`TasksPanel.svelte`, `GlobalTasksView.svelte`) over `src-tauri/src/tasks.rs`. **Three bring-up blockers:** (1) **Rule 8 violation** — both surfaces recompute by re-walking the disk; needs a persisted `note_tasks` index + write-path trigger + resumable back-fill. (2) **No right-click menu** at all — gap vs. 00 §5; must be the shared menu, not hand-rolled. (3) **Editor gate-bypass** — `toggle_task` writes via the WriteGate directly without firing the Editor's reindex, so search/backlinks/tags can drift after a toggle. Also: no `enabledFeatures.tasks` gate (add one); `TasksPanel` is missing a `dir` attribute (GlobalTasksView has it). The §2 question is **RULED (Boss, 2026-06-15): REFRAME, keep** — Tasks = "open epistemic loops" serving Tension→Synthesis→Conviction; candidate rename "Open Loops / Open Questions" + loop-biased parsing in bring-up. The three bring-up blockers (Rule 8, gate-bypass, no menu) remain the work; removal is off the table.
