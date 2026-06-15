# 08 — Quick Switcher (Concept Paper)

> A navigation satellite. It attaches to the **Editor** (the gate) by handing it a path to open — it reads the boot snapshot, owns no content, and writes nothing. Template per [00-MASTER](00-MASTER-Bring-Up-Charter-and-Checklist.md) §3; serves [00-Constellation](00-Constellation-Core-Concept-Paper.md).

## 1. Function in hand
The **Quick Switcher** — `src/lib/components/QuickSwitcher.svelte`, the modal "Find a note…" palette opened with **Ctrl+O** (`'quick-switch'` in `src/lib/utils.ts:732`) or from the Command Palette. Type a fragment, arrow/enter to open. Mounted unconditionally at `src/routes/+layout.svelte:7101` (`{#if showQuickSwitcher}`).

## 2. Purpose
The ONE job: **jump to any note by name, fast, from the keyboard** — type a fragment, hit Enter, the note opens in the Editor. It serves **Observation** (the first Act): get to the note you want to look at without leaving the keyboard or hunting the tree. It justifies itself — in a 7,600-note Universe the file tree is not a navigation tool; name-fuzzy jump is. It does not create knowledge; it *reaches* it, which is the precondition for every other Act.

## 3. What it is NOT
- **Not** a search engine — it is name/path substring matching over the cached note list (plus an optional federated FTS5 augment for queries ≥3 chars). It answers "which note is called X", not "which notes discuss X".
- **Not** a command runner — that is the **Command Palette** (`CommandPalette.svelte`), a separate surface. The Switcher only opens notes.
- **Not** an owner of content — it never reads or writes a note body; it emits a path and closes.
- **Not** the Library Switcher (`LibrarySwitcher.svelte`) — that changes the active library; this opens a note.

## 4. Wiring
- **Inputs:** `notes` prop = `allSwitcherNotes` = `$derived(allNotes)` (`+layout.svelte:1526`), itself populated once from the boot snapshot `core.notes` (`cache_boot_snapshot_core` → `note_meta`). On query ≥3 chars it calls `constellationSearch(parseSearchQuery(q))` (federated FTS5) via `$lib/libraries/store`. Strings via `$t()`.
- **Outputs:** none persisted. `onSelect(path, libraryName)` → `handleQuickSwitchSelect` (`+layout.svelte:4091`) → `openNoteTab(...)`. `onClose()` flips `showQuickSwitcher = false`. No IPC writes, no events emitted, no disk mutation.
- **Consumers:** none depend on it — it is a leaf. It is a *consumer* of the boot snapshot and the search index.
- **Connection to the Editor (the gate):** it attaches by **opening a tab**. `handleQuickSwitchSelect` → `openNoteTab(path, libraryName, color)`; the Editor then becomes the single authority for that note. The Switcher hands off and is gone — exactly the satellite→gate shape: it never touches content, only routes the user into the Editor.

## 5. Right-click / context menu
**Has none.** Grep of the component for `oncontextmenu` / `contextmenu` / `ContextMenu` / `buildContextMenu` returns nothing — the only handlers are `onclick`, `onkeydown`, `onmouseenter`, `oncompositionstart/end`. Each result row is a `<button>` with left-click-to-open only.
- **Gap (flag, low priority):** a per-result right-click menu would be a reasonable affordance — "Open in new pane", "Open in second screen", "Reveal in file tree", "Copy wikilink". These actions exist elsewhere (the file-tree shared `<ContextMenu>`, MIG-077) but are **not reachable from the Switcher**. If added in bring-up, it MUST use the shared `<ContextMenu>` / `buildContextMenu` (MIG-077) — never hand-rolled. Today: no right-click, and no action is reachable *only* by right-click (there are none).

## 6. Multilingual
- **Localized.** The two user-facing strings both flow through `$t()`: `quickSwitcher.placeholder` and `quickSwitcher.noResults`. Both keys are present in **all 15 locale files** (ar, de, en, es, fa, fr, he, hi, ja, ko, pt, ru, tr, ur, zh) — verified the `quickSwitcher` block in each. The command label uses `$t('commands.quickSwitcher')`. **No hardcoded English** in the component.
- **RTL / bidi:** the component sets **no `dir`** attribute and does not call `detectDir()`. Result names render in the inherited document direction; CSS uses logical `text-align: start`, which is RTL-safe for layout, but mixed-script note titles get no per-item `dir="auto"`. **Flag (bring-up):** add `dir="auto"` (or `detectDir()`) on the input and on `.qs-name` so an Arabic/Hebrew title inside an LTR UI (or vice-versa) renders with correct base direction — same polish noted for NotePane badges.

## 7. Boot behavior
- **Runs at boot?** **No.** The component mounts only on user action (Ctrl+O / palette). It runs **no IPC of its own at boot**.
- **Rule 8 status:** **reads-stored — compliant.** Its data source (`allNotes`) is the boot snapshot read `SELECT name, path, library_name FROM note_meta` inside `cache_boot_snapshot_core` (`cache.rs:167`, Phase 3) — a read of the persisted SQLite index, **not** a filesystem walk and **not** a recompute. Per-query it does an in-memory substring filter (debounced 300 ms, capped 30 rows) plus an optional FTS5 `MATCH` (also a stored-index read). It recomputes **no universe-wide derived view** on open. No violation.
- **Cost:** zero at boot (not mounted). Per-open: slice top-30 of an in-memory array — sub-millisecond (estimated). Per-keystroke: nothing synchronous (the MIG-058 fix moved filtering off the keystroke tick into a 300 ms debounce); the federated `constellationSearch` is async and guarded against stale results (`if (q !== query) return`). Federated FTS5 latency measured up to ~10 s on cold boot per the in-code MIG-059 note — masked by the debounce + stale-discard, not on the typing path.

## 8. Flag / gate & bring-up position
- **Gate today:** **none.** Unlike Constellation Map / Sight / CCS (gated by `enabledFeatures.*` / `SIGHT_*_ENABLED`), the Switcher mount is an unconditional `{#if showQuickSwitcher}` driven only by the user. It is de-facto navigation core.
- **Bring-up phase:** **2 (navigation satellites)** — depends on Phase 1: the Editor (`openNoteTab`) and `cache_boot_snapshot_core` (the `allNotes` source). The ≥3-char federated augment additionally depends on the search index being ready; without it, the local substring path still works (graceful degrade — the `try/catch` falls through to local-only).
- **Bring-up decision:** could re-enable in minimal mode unconditionally (cheap, leaf, no derived state), OR add a `enabledFeatures.quickSwitcher` gate for parity with other satellites — **needs Boss decision in bring-up.**

## 9. Budget
- **Boot budget:** **0 ms** — must remain un-mounted at boot; no boot IPC. Regression if any `invoke()` is added to module load.
- **Interaction budget:** open ≤16 ms (one paint); **every keystroke instant** (Rule 1) — no synchronous filter on the keystroke tick; filtering debounced ≥300 ms; zero `invoke()` on the keystroke path (the federated call is debounced + async + stale-guarded, per Rule 3 IPC rules).
- **Regression guard:** type a 10-char Arabic burst into the input with a 1,100+ note cache — no dropped/truncated characters (the MIG-058 report). Open on a 7,600-note Universe — list appears without jank. **Virtualization note (Rule 3):** the list is *capped* at 30 rows, not virtualized; the cap keeps it within budget today but if the cap is ever raised past ~50, it must move to a virtualized list.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** Ctrl+O → type fragment → Enter opens the matching note in the Editor; arrow keys move selection; Esc closes.
- [ ] **Serves Constellation's core purpose:** keyboard-first **Observation** — reaching any note in a large Universe without the tree.
- [ ] **Wires correctly to the Editor:** `onSelect` → `openNoteTab` opens exactly one tab; the Editor becomes the note's authority; the Switcher writes nothing.
- [ ] **Right-click present + correct:** currently absent; if added, uses shared `<ContextMenu>`/`buildContextMenu` (MIG-077), **not** hand-rolled — or explicitly ruled "none-ok" by the Boss.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** `placeholder` + `noResults` localized in all 15 locales (✓ verified); **add `dir="auto"`/`detectDir()` on input + result rows** before sign-off.
- [ ] **Within budget:** 10-char Arabic burst drops no characters; open is jank-free on 7,600 notes; no `invoke()` on the keystroke path.
- [ ] **Obeys Rule 8:** reads `allNotes` (the persisted boot snapshot) + FTS5; recomputes no universe-wide view on open (✓).
- [ ] **Holds its invariants:** stale federated results are discarded (`q !== query` guard); the debounce timer is cleared on each keystroke; selection index resets on result change.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **— (un-measured; per-keystroke path proven by MIG-058 fix, federated latency un-budgeted)**
Notes: Rule 8 **clean** — reads the persisted `note_meta` snapshot, recomputes nothing. Two flags for bring-up, both non-blocking: (1) **no right-click menu** — gap, if filled use the shared MIG-077 `<ContextMenu>`; (2) **no `dir`/`detectDir()`** on input or result rows — mixed-script titles get no per-item base-direction. The MIG-058/059 work already hardened the Arabic-input path (debounced `$state` filter, IME `composing` guard, stale-result discard). Gate today: **none** — de-facto navigation core; bring-up decides whether to add an `enabledFeatures.quickSwitcher` flag for satellite parity.
