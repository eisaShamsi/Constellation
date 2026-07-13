# Session Log — 2026-07-13

## PJ-092 — the rename-cascade fix — REVERTED and RE-OPENED

**Function in hand:** the rename wikilink cascade (`handleRenameComplete` → `flushAllTabsInLibrary` → `updateLinksOnRename` → `reloadTabsFromDisk`).

### What happened
- PJ-092 shipped 2026-07-12 as commit `0a605f02` — a **dirty-guard** in `reloadTabsFromDisk` (skip reseeding a still-dirty note). **It was committed to `main` WITHOUT a Boss live test** ("deterministically proven by the store test"). That was the mistake.
- The Boss live-tested it: a **deterministic FREEZE** in the exact scenario the fix protects. A note whose flush fails during a rename is left dirty + disk-mismatched; the editor never remounts (the clobber path bumps `reloadVersion` and converges — the skip path does not), so the Svelte reactive layer hangs. A focused-fix investigation (`general-purpose` trace agent) refuted the watcher-loop / reindex-deadlock / double-reload and pinned it to the reactive re-entrancy.
- A follow-up **flush-outcome-gate** rework (mirror `renameItem`'s `renameFlushOk`: `flushAllTabsInLibrary` returns the not-durably-flushed paths → the cascade skips them; sibling callers gate on the flush outcome; revert the dirty-guard) tested as **still not acceptable** to the Boss.
- **Boss ruling: "FIX IT, don't patch it, or revert PJ-092." → REVERTED entirely.** Code restored to `fd6008bc` (pre-PJ-092): `src/lib/libraries/store.ts` (`reloadTabsFromDisk` + `flushAllTabsInLibrary`), `src/routes/+layout.svelte` (cascade), `src/lib/lens/store.ts`, `src/lib/editor/noteSession.ts`, `vitest.config.ts`. All PJ-092 code (dirty-guard, sibling gates, LOCKTEST/SHOWBUG live-test hooks) + tests removed. **svelte-check 0 · vitest 335** (pre-PJ-092 count). Boss live-tested the reverted build → **PASS** (normal rename works, no freeze, app responsive). THEN committed (the commit gated on the Boss pass — the new standing rule).

### Two durable lessons
1. **The Boss Test is MANDATORY on every build (new TOP STANDING ORDER, Boss-mandated 2026-07-13).** The commit is the LAST step, gated on the Boss's pass. No "backend-only" / "proven by tests" exceptions. The PJ-092 freeze reached `main` precisely because I committed `0a605f02` without a Boss test. (Memory: `feedback_boss_test_every_build_mandatory`.)
2. **PJ-092 should have gone through `/migration`, not a focused fix.** It touches the rename cascade + the editor reactive lifecycle across Rust↔Svelte — a textbook migration change. The band-aid → freeze → re-patch cycle is exactly what the Migration Rule + Solve-the-Class exist to prevent. **The freeze was invisible to the store-level vitest** (no real watcher/reindex/remount) — the "vitest is not runtime verification for editor-lifecycle bugs" gap.

### Status
- **PJ-092 (bug) — RE-OPENED, Group 1.** The rare rename-cascade edit-loss (a note open + dirty + `.md` locked at the instant of a rename). To be redone via the full `/migration` (Architect → Boss picks approach → Plan → Build → Audit), Reproduce-First on the RUNNING app, before any code.
- **Close (SO#9):** Pending Jobs **v1.24** (PJ-092 reverted → re-opened; ► Next action = PJ-089); Orientation **v3.45**; Charter revert-note appended. PJ-094/095/096 (from the PJ-092 sweeps) remain valid, independent of the revert.
- **Also this session (2026-07-12, carried):** PJ-091 (accept-merge) shipped `fd6008bc`; PJ-071 (bulk-accept RMW) shipped `7daaf946` — both remain (not touched by the revert).

---

## PJ-092 REDO — rename-cascade edit-loss/freeze — FIXED via /migration

After the revert, redone properly through the four `/migration` phases + a NEW design-stage safety inspection. Approach: **flush-gate-exclude** (Boss-picked). Full record: `docs/PJ-092-Rename-Cascade-Redo-Migration.md`.

- **Architect** (workflow `wf_f9b2c823-1fe`): 4 options, 2 dropped as freeze-unsafe; Boss picked flush-gate-exclude (structural no-loss — never write the file).
- **Plan**: 9 steps; sharp edge = the JS↔Rust path-normalization contract.
- **Design-stage safety inspection** (`wf_f922a5cc-f78`, Boss-requested) — 5 hazards caught BEFORE code: H1 Arabic-NFC path-match (→ file-identity `canonicalize`+NFC + fail-closed belt), H2 await-window race (→ bounded re-flush loop), H3 focus-blind reseed (→ focusReseed), H4 4 sibling callers (→ Boss ruled fix-in-scope, shared `flushOpenTabOrAbort`), H5 alias-refresh.
- **Build**: all amendments folded in. `/simplify` caught 2 more contract gaps (belt-not-NFC, siblings-not-bounded) → fixed. Per-build safety-inspection: only the temporary LOCKTEST harness + pre-existing backlog. Reproduce-First on the running app via a temporary content-based LOCKTEST harness (removed before commit).
- **Audit** (`wf_abf7f854-5cd`): 11/11 invariants HOLD · migration-path PASS · 1 drift (cascade:rewrote listener bypassed the belt) → FIXED (shared `renameCascadeExcludedKeys`).
- **Verify**: `renameCascadeExclude.test.ts` (3) + Rust `cascade_walker_tests` (NFC/NFD identity, separator-mismatch exclude, empty-exclude rollback); svelte-check 0, vitest 338, cargo walker 16.
- **Boss live-test** (per the mandatory Boss-test-before-commit rule): A1 normal rename, A2 locked-note-protected + others-update (real Arabic-root universe), B1 Focus mode, B2 restart recovery, + clean-binary sanity — **ALL PASS**.
- **Close (SO#9)**: Pending Jobs **v1.25** (PJ-092 Done; PJ-097 filed — FocusPane freeze-overlay follow-up); Orientation **v3.46**; migration record + Charter register. **NEW STANDING PROCESS: the Safety Inspection reviews the DESIGN (Architect/Plan), not just the code** — Boss-endorsed; it caught PJ-092's 5 hazards for free.

---

## PJ-089 — Index-panel preview two-writable-model silent clobber (STARTED)

**Function in hand:** the **Index panel preview editor** — the split pane inside the sidebar Index overlay that shows a term's source note next to the term browser (`handleIndexNoteClick` → the standalone `indexNoteTab` `$state` → `<NoteEditor tab={indexNoteTab}>` at `+layout.svelte:7346`).

**Concept (the horse):** the Index preview is a **peek** into a term's source note without leaving your train of thought — *not a second desk*. A peek must never be able to overwrite the note you have open on your main desk. The function (a preview editor) must serve that: **one writable owner per note, always.**

### SO#8 cross-check — the two-writable-model site STILL EXISTS as described ✔
- **Mount:** `handleIndexNoteClick` (`+layout.svelte:6458`) reads the note FRESH from disk (`read_note`) and builds a standalone tab `indexNoteTab` with a **unique id** `index_preview_${Date.now()}` (`:6489`). Rendered at `:7346` as `<NoteEditor tab={indexNoteTab} … />` with **NO `readOnly` prop** → the preview editor is **writable** (`handleSave`/`handleFlush` in `NoteEditor.svelte` DO write to disk through the durability gate).
- **Why two models:** the single-ownership `models` Map (`noteModel.ts:93`) is keyed by **`id`**, not path. `ensure(id,path,content)` (`noteSession.ts:64-68`) only early-returns for the SAME id on the same path. The preview's unique `index_preview_*` id ≠ any main tab's id → a **second, independent NoteModel for the same path** (own `version`/`savedVersion`/`body`/`base`/`diskBaseline`).
- **Why no reconciliation:** the watcher-adopt (`adoptExternalChangeIntoTabs`, `store.ts:778`) filters to `openTabs` paths only (`:788-790`) and iterates `openTabs` (`:816`). The preview tab is a **standalone `$state`** (`+layout.svelte:627`), **never in `openTabs`** → its model is invisible to adopt. The store's own comment (`store.ts:812`) states the adopt design **assumes** "path↔id is 1:1 under DEDUP" — the preview breaks that assumption.
- **Result:** open note N in a main tab (model A) + preview N in the Index (model B seeded from disk). Edit A → A saves to disk. Edit B (whose seed predates A's save; B never learns of A's save) → B saves to disk → **A's edit silently gone**, no `.conflict` sidecar. Symmetric the other way. This is the content-integrity / write-path class (Solve-the-Class: single content ownership).
- Orientation §4.x confirms the Index panel is a **term browser** (the "diagnostic instrument"); the split preview is a reading peek, not a designed editing surface. Editing-in-preview is incidental.

### Predecessor → Replacement (Predecessor Lookup Rule — pre-code)
- **Where it lives now:** `handleIndexNoteClick` + the `indexNoteTab` render block, `src/routes/+layout.svelte` (`:6458` builder, `:7339-7350` render). Introduced pre-MIG history as the Index split-pane preview; filed as PJ-089 by the PJ-088 sweep (`wf_c0dac305-85e`).
- **Where the replacement will live:** **the same place** (`+layout.svelte`, the same mount). No relocation.
- **What gets cut / kept:** the fix removes the preview's *writability* (or shares the open note's model) — either way the second independent writable model is eliminated. The preview's read/display path is kept; the Ctrl/middle-click "open in a real tab" escape hatch (`:6460`) is kept as the edit affordance. Approach (read-only peek vs share-model dedup) → Boss ruling below, after Reproduce-First.

### Reproduce-First (top principal — in progress)
Harness reproduction of the two-writable-model clobber through the REAL `noteSession`/`noteModel` path (two ids, one path, independent saves through a fake disk) → RED. Running-app Boss live-test is the final verification (vitest is necessary, not sufficient, for this class).

- **`tests/mig-076/indexPreviewClobber.test.ts` — Recipe Q (2 tests, both reproduce the mechanism, GREEN=characterizes the bug):**
  - RED: main tab (id `main`) saves `main edit B`; the preview (id `index_preview_1`) — seeded fresh from disk BEFORE edit-B, never told about it (outside `openTabs`) — saves its stale-seeded tweak → **`main edit B` silently gone from disk, no `.conflict`**. `screen(main) !== disk`.
  - RED-2: the preview model is **clean + eligible** to adopt but `externalChange` is **never wired** for it in production (standalone `$state`, not in `openTabs`) → a main save can never reach it.
- **Mechanism off the trace:** `models` keyed by id (`noteModel.ts:93`) → preview's unique `index_preview_*` id = 2nd model → mounts writable (no `readOnly`, `+layout.svelte:7346`) → outside `openTabs` so `adoptExternalChangeIntoTabs` (`store.ts:788-790`) skips it → last-writer-wins.
- **Solve-the-Class check:** the ONLY writable NoteEditor mount on a standalone (non-`openTabs`) tab in the whole app is the index preview (`:7346`). Split-view (`:7882`) iterates `$openTabs` (deduped, watcher-covered); `$activeTab` (`:8042`) is a real tab; every SecondScreen mount is `readOnly`. Single-surface instance.

### Design fork → Boss ruling (the /migration "Boss picks approach" step)
- **A — Read-only peek (recommended):** mount the preview `readOnly` (the proven Display-not-Domain primitive the second screen uses) + a one-click "Open to edit". Removes a writer → structurally cannot clobber; single-surface frontend fix; no ownership-model surgery; matches the concept. **Focused fix.**
- **B — Share-one-live-copy dedup:** reuse the open tab's model id so both views back one model. Preserves edit-in-preview but is a content-ownership-model change (two CM6 views on one id diverge on screen until remount; the not-open→then-opened race) — **/migration**, the burned content-integrity class.

### Boss ruling + build + Stage 1 test
- **Eisa picked A (read-only peek).** Focused single-surface frontend fix — no /migration.
- **Built:** `readOnly={true}` on the preview mount; a lifecycle-owned `$effect` disposal (keyed to `indexNoteTab?.id`) frees the preview's model on any change/clear (structural leak fix); a shared `leaveIndexForNote` helper; an **"Open to edit"** button (promotes the peek to a real single-owner tab via `openNoteTab` path-dedup); `indexPanel.openToEdit` ×15 locales; CSS.
- **Gates:** svelte-check 0 · vitest 341 · **diff-scoped safety inspection = 0 in-diff findings** (the 14 whole-app findings are pre-existing backlog + 4 new to file at close — PJ-098–101) · `/simplify` applied (4 of 5: lifecycle-owned disposal, shared helper, test cleanups, redundant-title; skipped the shared-code readOnly seed-model note) · binary freshness-verified.
- **Boss Stage 1 test: PASS**, with one UX report — clicking a `[[wikilink]]` inside the read-only preview did "nothing in the preview but opened a tab in the background" (the default NoteEditor link handler opening a real tab hidden under the Index overlay; pre-existing, exposed by the peek).

### Link-fix increment (WA#6 — fix what surfaced in the Boss test)
- **Change:** `NoteEditor` gains an optional `onLinkClick?` override prop (default unchanged → `onlinkclick={onLinkClick ?? handleLinkClick}`); the read-only Index preview passes `handleIndexPreviewLinkClick`. Links in the peek now behave like the Index note-list: **plain-click → the preview follows the link** (navigate the peek, stay in the Index); **Ctrl/middle-click → open a real tab + leave the Index** (Return-to-Index). A peek never authors → an unresolvable link is inert (no note creation). **No background tabs anywhere in the peek.**
- **Verify:** focused adversarial review — SAFE on all 6 vectors (no write/second-model/clobber; strictly removes the default handler's `createNote` write path; no other mount affected by the `??` fallback). svelte-check 0 · vitest 341. Rebuilt frontend + binary.

### Boss live-test — ALL PASS → committed
- **Stage 1** (read-only peek + "Open to edit"): **PASS** (with the link report → link-fix increment above).
- **Link re-test** (plain-click follows in peek · Ctrl-click opens a real tab + leaves Index): **PASS.**
- **Stage 2** (same note in a tab + preview → no silent overwrite; "Open to edit" no duplicate; the two close buttons): **PASS.**
- **Committed after the pass** (the standing rule — commit is the LAST step, gated on the Boss). **Close (SO#9):** Pending Jobs **v1.26** (PJ-089 Done; PJ-098–101 filed from the per-cycle whole-app sweep `wf_ca0d3aa9-3d6`); Orientation **v3.47**; English Index help updated (read-only peek + Open-to-edit + link-follow — the 14 locale backfill is standing PJ-014 debt); Charter register appended; MoCh `MoCh-2026-07-13-1400.md`. ► Next action = **PJ-090** (SS Tasks-panel toggle no-broadcast clobber).
