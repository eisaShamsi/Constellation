# 31 — Quick Capture (Concept Paper)

> Per-function paper. Serves [00-Constellation](00-Constellation-Core-Concept-Paper.md): every function must advance one of the Five Acts and wire to the Editor (the gate). Quick Capture is the fastest possible **Observation** entry point.

## 1. Function in hand
**Quick Capture** — the command `quick-capture` (palette label `$t('commands.quickCapture')` = "Quick Capture", icon ⚡, shortcut `Ctrl+Shift+N` per `src/lib/utils.ts`). It is **not a component**: it is the handler `handleQuickCapture()` in `src/routes/+layout.svelte` → the store wrapper `quickCapture()` in `src/lib/libraries/store.ts` → the Rust IPC `quick_capture` in `src-tauri/src/libraries.rs`. No pop-up window, no dedicated panel.

## 2. Purpose
The **one job**: create a fresh, timestamped, empty note in the inbox folder and drop the user straight into editing it — zero dialogs, zero naming, zero folder-picking. It serves **Observation** (the first Act): get a raw thought onto disk before it evaporates. It exists because the friction of "new note → name it → choose a folder" is exactly the friction that loses fleeting ideas. (`en.json` maps the Stage value *Fleeting* to "quick capture" — Quick Capture is the capture half of the Fleeting→Permanent lifecycle.) Justified: without a frictionless inbox, Observation has a tax; *File Over App* makes the captured `.md` an immediate first-class file.

## 3. What it is NOT
- **NOT** a separate pop-up inbox window or modal — it creates a note and opens a normal tab in the main window.
- **NOT** a note *type* — the output is an ordinary `.md` (frontmatter `created:` + empty body); nothing distinguishes it from a hand-made note except its location and timestamp name.
- **NOT** a renamer/organizer — it never moves or files anything later; it only creates at the inbox path (default `+`).
- **NOT** the Editor — it hands off to the Editor immediately and owns no content state.

## 4. Wiring
- **Inputs (events/stores read):** command-palette invocation or `Ctrl+Shift+N`; reads `$libraries[0]` (always targets the **first** library), `$appSettings.inboxFolder` (default `'+'`), `libraryColorMap[lib.name]`.
- **Outputs (IPC/writes):** `invoke('quick_capture', { libraryPath, inboxFolder })` → Rust creates `inbox/YYYY-MM-DD HH-MM.md` (deduped `… N.md` up to 100) via `gate_create_exclusive` (MIG-076 §A2 create-exclusive: refuses on concurrent-create race instead of overwriting); content is `---\ncreated: YYYY-MM-DD\n---\n\n`. Then `refreshLibraryTree(lib.id)`, `openNoteTab(newPath, …)`, and `toggleEditMode(tab.id)`.
- **Consumers:** the file tree (refreshed), the tab bar (new tab), the Editor (receives the open + edit-mode toggle).
- **Connection to the Editor (the gate):** Quick Capture does **not** write content or manage lifecycle — it creates an empty file on disk, then calls `openNoteTab` + `toggleEditMode`, handing the note to the Editor. All subsequent typing, saving, reindexing, and downstream derivation flow through the Editor exactly as for any note. Quick Capture is a *create-and-hand-off* shim, correctly a display/dispatch concern, never a domain.

## 5. Right-click / context menu
**None.** Quick Capture is a command/handler with no surface of its own, so there is nothing to right-click — no `oncontextmenu`, no `ContextMenu`, no `buildContextMenu` in any of its three files (verified by grep). The note it *produces* appears in the file tree and tab bar, which carry the **shared `<ContextMenu>`/`buildContextMenu` (MIG-077)** menus for note/tab targets — so the captured note is right-clickable through those surfaces, not through Quick Capture itself. **No gap:** a command-trigger needs no menu; the standard note/tab right-click already covers the output.

## 6. Multilingual
- The palette label is `$t('commands.quickCapture')` — wired through `$t()` — **but the value is the literal "Quick Capture" in all 15 locale files** (ar de en es fa fr he hi ja ko pt ru tr ur zh all verified to contain `"quickCapture": "Quick Capture"`). This is the brand-name pattern (like "Sky View"). **Flag:** per the *full-localization* top-principal (memory `feedback_full_localization_everything`), when the UI language switches, this label currently does **not** adapt to a native equivalent. Whether it *should* (brand-name exception vs. localize-everything) is a Boss call to record in bring-up.
- The note's body is empty and the only stored string is an ISO date — no language assumption in the output; content the user then types round-trips faithfully via the Editor's per-line bidi.
- The inbox folder name default `'+'` is script-neutral.
- No `detectDir`/RTL concern in the handler itself (no rendered chrome).

## 7. Boot behavior
- **Runs at boot?** No. Quick Capture fires only on explicit user command; it registers no boot IPC and computes nothing at startup. Its handler is defined but dormant until invoked.
- **Rule 8 status:** **N/A — reads-persisted / no derived view.** It creates a single file on demand; it never recomputes a universe-wide view at read or boot. No violation.
- **Cost:** one filesystem `create_dir_all` + one gated file write + a tree refresh — single-note IO, sub-millisecond order (estimated; not separately measured). Negligible.

## 8. Flag / gate & bring-up position
- **Gate today:** **none.** Unlike Sky View / Map / CCS, the `quick-capture` palette entry is **not** wrapped in any `enabledFeatures.X` check — it is unconditional core (verified: no `enabledFeatures` guard around the command at `+layout.svelte:1809`). Minimal mode would keep it on.
- **Bring-up phase:** **Phase 1 / early — a thin satellite on the core spine.** Depends only on: the Editor (the gate, Phase 1), the file tree refresh, and `$appSettings.inboxFolder`. No semantic/index/Sight dependency.
- **Known wiring debt to settle in bring-up:** (a) it always targets `$libraries[0]` — behavior when the focused library is *not* the first is unverified; (b) `inboxFolder` has a default (`'+'`) but **no Settings UI exposes it** (no grep hit for `inboxFolder` in any `.svelte` settings surface) — it is editable only via raw settings. Flag both for the Boss.

## 9. Budget
- **Boot budget:** zero — does not run at boot.
- **Interaction budget:** capture→editable note should feel instant (≤ ~50 ms perceived) — one file create + tree refresh + tab open; no `invoke()` loop, no heavy work. After hand-off, the Editor's own keystroke budget (Rule 1, instant) governs.
- **Regression guard:** invoke `Ctrl+Shift+N` 5× rapidly → 5 distinct deduped notes, no overwrite (the `gate_create_exclusive` path), each opening editable; confirm no stray write to a *different* note (content-integrity class). Re-measure if the create path or `openNoteTab`/`toggleEditMode` sequence changes.

## 10. Acceptance checklist (the gate to re-enabled)
- [ ] **Serves its purpose:** `Ctrl+Shift+N` creates a timestamped inbox note and lands the cursor in it, no dialog.
- [ ] **Serves Constellation's core purpose:** advances **Observation** — frictionless idea entry (see [00-Constellation](00-Constellation-Core-Concept-Paper.md)).
- [ ] **Wires to the Editor:** hands off via `openNoteTab` + `toggleEditMode`; never writes body content or manages lifecycle itself; downstream reindex flows through the Editor's save.
- [ ] **Right-click present + correct:** N/A for the trigger; the produced note/tab carry the **shared** `<ContextMenu>` (not hand-rolled) — verified, not assumed.
- [ ] **Multilingual ×15 + RTL + no hardcoded English:** label is `$t()`-wired but the value is identical English in all 15 locales — **Boss to rule** whether it stays brand-English or localizes; output body is language-neutral.
- [ ] **Within budget:** capture→editable feels instant; rapid repeat creates distinct deduped notes.
- [ ] **Obeys Rule 8:** creates one file; recomputes no derived view at read/boot.
- [ ] **Holds its invariants:** create-exclusive refuses on concurrent-create race (no overwrite); always targets a valid library inbox; path-validated in Rust.
- [ ] **Boss-tested** per the Testing Instructions Rule.

## 11. Status
Concept paper: **draft** · Enabled in bring-up: **no** · Budget met: **—** (unmeasured; estimated negligible) · Notes: Not a pop-up inbox — a create-and-hand-off command (handler in `+layout.svelte`, store wrapper in `store.ts`, IPC `quick_capture` in `libraries.rs`). Three open items for bring-up: (1) the "Quick Capture" label is the same English in all 15 locales — Boss-ruling needed on localize-vs-brand; (2) `inboxFolder` has no Settings UI; (3) it hard-targets `$libraries[0]` rather than the focused library. None block the function; all should be recorded before re-enable.
