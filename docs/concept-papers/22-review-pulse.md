# 22 — Review Pulse (Concept Paper)

> Per-function paper. Must trace to [00-Constellation](00-Constellation-Core-Concept-Paper.md) (the Five Acts, File-Over-App, Rule 8) and attach to [01 — Note Editor](01-Note-Editor.md), the gate. Code: `src/lib/components/ReviewPulsePanel.svelte` (UI) + `src-tauri/src/review.rs` (Cognitive Engine Phase 7, "نبض المراجعة").

## 1. Function in hand
**Review Pulse** — the right-sidebar **Review** panel (`ReviewPulsePanel.svelte`), a spaced-resurfacing / staleness queue of notes "due for review." Three sections: **Due for Review** (resurfacing), **Mental Model Checkpoints**, **Never Reviewed**.

## 2. Purpose
The ONE job: **resurface a note at the right moment so the user re-examines a held position** — "still relevant? still true? link it? archive it?" (`review.rs` header). It is **not flashcards**; it is a revisit prompt. It serves **Tension** (a 30-day `#assumption`/`#model` checkpoint asks "do you still hold this view?" — forcing a held belief back into question) and feeds **Conviction** (re-affirm or supersede). Justification: in a PKF system a conviction left untouched silently rots; Review Pulse is the only surface that *schedules* the act of re-confronting it. It earns its place — but see §7: its current implementation contradicts Rule 8.

## 3. What it is NOT
- **Not flashcards / SRS quizzing** — no question/answer, no grading; it surfaces the note, the Editor does the rest.
- **Not a writer of note content** — it persists schedule state to `.constellation/review-pulse.json` only; it never edits `.md` bodies (File-Over-App preserved).
- **Not Source Review** — a *separate* `sourceReview` tab/panel (MIG-021) handles classifier/source triage; do not conflate.

## 4. Wiring
- **Inputs (IPC read):** `get_due_notes(libraryPath)` — called on the **Review tab click** and from the command palette ("Review due notes"). Returns `DueNote[]` (path, name, reason, days_overdue, stratum, last_reviewed). Reads `review-pulse.json`.
- **Outputs (IPC write):** `mark_reviewed(notePath)` (doubles interval, cap 30d), `snooze_note(notePath, days=7)`, `dismiss_note(notePath)`. All write `review-pulse.json`. A registered `record_note_visit` exists **but is never called from the frontend** (verified: no call site in `src/`) — so `last_reviewed` advances only via the explicit ✓ button.
- **Consumers:** the sidebar tab badge (`dueNotes.length`); nothing else depends on it.
- **Connection to the Editor (the gate):** `onNoteClick(path,name)` → `openNoteTab(...)` — opening a due note routes through the gate correctly (no re-implemented load). The three actions bypass the Editor by design (they touch schedule JSON, not note content).

## 5. Right-click / context menu
- **None.** Grep of the component finds no `oncontextmenu` / `contextmenu` / `<ContextMenu>` / `buildContextMenu`. The per-row actions (Reviewed ✓ / Snooze 👁 / Dismiss 🗄️) are **hand-rolled inline icon `<button>`s**, not a shared menu — and they are *not* duplicated as a right-click path either.
- **Gap to flag:** per the core paper §5 ("right-click should include every aspect of the app", MIG-077), a due-note row **should** offer a shared `<ContextMenu>` (Open / Open in new surface / Reviewed / Snooze… / Dismiss / Reveal in tree). Bring-up action: add a shared `buildContextMenu` for due-note rows; do **not** hand-roll a second menu. No action is currently reachable *only* by right-click (there is no right-click), so nothing is lost today — but the surface is non-conformant.

## 6. Multilingual
- **Mostly localized:** all section/label/action strings flow through `$t('reviewPanel.*')` and the key block exists in **all 15 locales** (ar de en es fa fr he hi ja ko pt ru tr ur zh — verified). Palette/tab strings `commands.reviewDueNotes`, `panels.review` exist in en.json.
- **Hardcoded English — FLAG:** the per-row detail strings are inline and untranslated — `{note.days_overdue}d overdue` (line 80) and `{note.days_overdue}d old` (line 133). The "d overdue" / "d old" suffixes never pass through `$t()`. Fix: add `reviewPanel.daysOverdue` / `reviewPanel.daysOld` with a count placeholder, ×15.
- **RTL:** the panel handles its own chevron flip (`:global([dir="rtl"]) .rp-chevron`), but note **names have no `dir`/`detectDir()`** — an Arabic/Hebrew title in the list won't get per-name bidi like the Editor gives it. Bring-up: apply `dir="auto"` (or `detectDir()`) to `.rp-name`.

## 7. Boot behavior
- **Runs at boot?** No automatic boot IPC — `get_due_notes` fires lazily on first Review-tab click / palette command. (The tab **badge count** only populates once the tab is opened, so a "5 due" badge is not shown at cold boot.)
- **Rule 8 status: RECOMPUTES-on-read — VIOLATION.** `get_due_notes` runs `scan_due_recursive` over the **entire library filesystem** every call: `read_dir` recursion, `fs::metadata` per note, and for checkpoints a full `fs::read_to_string` + regex `#(assumption|model)` scan of each `.md`. Nothing is persisted as a derived "due list"; `review-pulse.json` stores only user *actions*, not the computed queue. This is exactly the Rule-8 anti-pattern the core paper forbids (a `scan_*` that re-walks the Universe on read).
- **Cost:** unmeasured — **estimate** O(N notes) disk reads per open, with a *full content read per note that carries `#assumption`/`#model`*. On a 7,600-note Universe this is the read-time recompute Rule 8 exists to eliminate. **Mark: measure in bring-up.**

## 8. Flag / gate & bring-up position
- **Gate today:** none semantic — only a *placement* switch, `$appSettings.panelPlacements?.review ?? 'right-sidebar'`. There is **no `enabledFeatures.X` / `SIGHT_*` flag**; it cannot be cleanly disabled for minimal mode. **Needs a new gate** (e.g. `enabledFeatures.reviewPulse`).
- **Bring-up phase:** **Phase 5 (curation), gated.** Depends on: (a) a **Rule-8 redesign** — persist a `review_schedule` table maintained at write time (note save / tag change / dismiss updates the due-set via trigger or hook), so reads are cheap lookups, with a resumable first-time back-fill; (b) the new feature gate; (c) the Editor (gate) for opening due notes — already wired.

## 9. Budget
- **Boot budget:** zero — must not fire any IPC at boot; the badge may stay empty until first open, or be hydrated from the persisted schedule (post-Rule-8 fix) cheaply.
- **Interaction budget:** opening the Review tab must return in <100 ms on a 7,600-note Universe — **only achievable after the Rule-8 fix** (today it does a full FS walk + content reads).
- **Regression guard:** measure `get_due_notes` latency before/after on a large Universe; assert no full-content scan on the read path; confirm ✓/Snooze/Dismiss persist and the row leaves the queue.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** a `#assumption`/`#model` note resurfaces at 30 days; ✓ doubles the interval; Snooze hides 7 days; Dismiss removes permanently.
- [ ] **Serves Constellation's core purpose:** advances **Tension → Conviction** (re-confront a held position) — traces to [00](00-Constellation-Core-Concept-Paper.md).
- [ ] **Wires to the Editor (the gate):** clicking a due note opens it via `openNoteTab` (no re-implemented load); actions touch only `review-pulse.json`, never `.md` content.
- [ ] **Right-click present + correct:** due-note rows expose a **shared** `<ContextMenu>` (not hand-rolled); actions reachable consistently.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** `daysOverdue`/`daysOld` moved to `$t()` ×15; note names get `dir`/`detectDir()`.
- [ ] **Within budget:** Review-tab open <100 ms on a 7,600-note Universe.
- [ ] **Obeys Rule 8:** due-set is **persisted and write-time-maintained**; no full FS/content recompute on read.
- [ ] **Holds its invariants:** dismissed/snoozed never reappear early; intervals cap at 30d; no `.md` body is ever modified.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (unmeasured; Rule-8 recompute makes large-Universe budget unlikely as-is)**
Notes: **Two blockers before re-enable** — (1) **Rule 8 violation**: `get_due_notes` re-walks the library filesystem (with per-note content reads for checkpoints) on every read; must become a persisted, write-time-maintained schedule. (2) **No shared right-click** + **hardcoded "d overdue"/"d old"** strings + **no per-name RTL**. Also: `record_note_visit` is dead (registered, never called) — decide whether visiting a note should count as a review, or remove the command.
