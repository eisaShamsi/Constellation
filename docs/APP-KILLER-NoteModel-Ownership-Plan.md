# APP-KILLER #2 — NoteModel-Ownership Nav-Loss — Build Plan (Phase 2)

*Follows `docs/APP-KILLER-NoteModel-Ownership-Architect.md`. Boss ruling: **A3** (full scope) + **B1** (jump to existing tab, unconditional).*

## Why incremental commits are safe here (vs the §CB rule)

Solve-the-Class warns against *live half-migrations* — the §CB failure was **swapping the
content-ownership storage** live, so every intermediate state was half-old/half-new = corruption.
This migration is **not** a swap: single-content-ownership (MIG-076 §C) is already stable and
untouched. Every step here **adds a guard on top of it** (flush-before-replace, a path-guard on
mark-clean, an abort-on-fail). Each guard is additive and independently safe, and the whole change
lands behind two consts (`NAV_FLUSH_ENABLED`, `DEDUP_ALL_TABS_ENABLED`, both default `true`) so it
is one-line revertible. The reproduction harness proves the **primitives**; the 8-item Editor-
Surface Gate Boss test proves the **component wiring** (the one residual the logic harness can't see,
per `runtimeHarness.test.ts`'s own header).

---

## §1 — Reproduce-First: red harness cases *(commit `§1 — nav-loss reproduction (RED)`)*

Extend `tests/mig-076/runtimeHarness.test.ts` with three logic-level recipes that FAIL against
today's code (they call primitives that don't exist yet / assert the guard that isn't there):

- **Recipe I — nav-away-while-dirty flushes the outgoing model.** `S.open('t', A)`, `editBody`
  dirty, then `await S.flushIfDirty('t', writer)` → assert disk(A) has the edit AND result `ok`;
  then `S.open('t', B)` (the replace) → `bodyForView` is B, disk(A) intact.
- **Recipe J — failed nav-flush ABORTS.** `flushIfDirty` with a throwing writer → result `ok:false`,
  model **stays dirty**, net retained, nothing written; the caller must not replace.
- **Recipe K — late save cannot poison a swapped-in model.** open A, `compose` A@vN, swap the id
  to B via `openModel`, then `markSaved('t', vN, /*expectPath*/A)` must **no-op** (B.savedVersion
  stays 0, B reports dirty after one edit).

**Verify:** the 3 new recipes are RED now; the existing 16 stay GREEN. (Proves the recipes capture
the real defect before any fix — Reproduce-First.)

## §2 — Core primitives: make §1 GREEN in isolation *(commit `§2 — flushIfDirty + markSaved path-guard`)*

`src/lib/editor/noteModel.ts`:
- `markSaved(id, version, expectPath?)` — add the optional path-guard: no-op if
  `expectPath !== undefined && m.path !== expectPath` (mirror the `compose`/`setBody` identity
  guards). Fixes the swap-poison (Architect §3.4).

`src/lib/editor/noteSession.ts`:
- `save(...)` — pass the composed path: `M.markSaved(id, r.version, r.path)` (was `r.version` only).
- **New** `flushIfDirty(id, env, origin='nav_flush'): Promise<SaveOutcome>` — the ONE nav-flush
  choke point: reads the old path **from the model** (`M.getModel(id).path`, not the tab — fixes
  Architect §3.3); returns `{ok:true}` immediately if no model or not dirty; else a **bounded
  loop** (`MAX=4`) of `await save(id, m.path, env, origin)` re-checking `isDirty` each pass (closes
  the await-window keystroke — Architect §3.5); returns the write failure on `!ok` (abort signal),
  or `{ok:false, reason:'still_dirty'}` if unresolved after the bound (abort). Never touches
  `openModel` — the store orchestrates flush → update → replace.

**Verify:** Recipes I/J/K GREEN; all 16 prior recipes GREEN; `npm run check` (svelte-check) clean.

## §3 — Wire the two nav sites *(commit `§3 — flush-before-replace at the nav sites (NAV_FLUSH_ENABLED)`)*

`src/lib/libraries/store.ts`, behind `const NAV_FLUSH_ENABLED = true`:
- **`openNoteTab` reuse branch** (before the `openTabs.update` at `:1766`): capture the nav token
  (shared `_navTokens` map — the same one `loadTabHistoryEntry` uses, so click-nav and Alt-nav
  supersede each other, Architect §3.5); `const f = await flushIfDirty(currentTab.id,
  standardSaveEnv({origin:'nav_flush', name, onSaved:(p)=>{ broadcastNoteSaved(p); reindex(p);
  embedIfEnabled(p) }}), 'nav_flush'); if (!f.ok) return; if (_navTokens superseded) return;` — the
  `onSaved` gives the flush the reindex + broadcast a bare flush would skip (Architect §3.2).
- **`loadTabHistoryEntry`** (before the `openTabs.update` at `:1004`): same `flushIfDirty` guard,
  reusing its existing `_navTokens`.
- **Do NOT touch `reloadTabsFromDisk` (`:605`)** — it is an *adopt-disk* path; flushing there would
  clobber cascade-authored disk (Architect §3.1).

**Verify (Boss-testable):** Gate 1 — type in A, click a `[[wikilink]]`/another note → A's edit is
on disk, B opens. Gate 3 — Alt+←/→ while mid-typing preserves the edit. Locked-file case — the
save-health banner appears and you stay on A (nav aborts).

## §4 — Focus-mode cousin *(commit `§4 — exit Focus on nav (path change)`)*

`src/routes/+layout.svelte`: the `$effect` at `:1473` resets `focusMode` on active-tab **id**
change; extend it to also reset on active-tab **path** change (track `_focusModePath`). In-place
reuse keeps the id but changes the path, so Focus now exits on nav → the new note renders in
NotePane and its keystrokes are no longer refused (Architect §2 Focus, §3).

**Verify (Boss):** Gate 2/4 — enter Focus on A, Quick-Switch to B → lands in NotePane on B; type →
persists; A intact; no spurious write at Focus enter.

## §5 — Rename durability cousin *(commit `§5 — rename flush honors the write outcome`)*

`src/lib/libraries/store.ts` `renameItem`: capture the `rename_flush` `SaveOutcome` (`:2632`); on
`!ok` (write failed) **skip** the unconditional `clearWriteAhead` (`:2664/:2665`) for that path and
**skip** the stale-disk re-seed (`openNoteModel` at `:2677`) — `repath` only, keep the dirty model +
its net. Success path unchanged (Gate-6 behavior preserved). Fixes the "rename on a locked file loses
the edit" hole (Architect §3, Invariant 8).

**Verify (Boss):** edit a note, make its file momentarily read-only, rename it → banner shows, the
edit is **not** lost (recovers on retry/reopen). Linked probe pair (A links B, rename B): both
identities intact (Gate 6).

## §6 — Duplicate-tab dedup (B1) *(commit `§6 — one path → one tab (DEDUP_ALL_TABS_ENABLED)`)*

`src/lib/libraries/store.ts` `openNoteTab`, behind `const DEDUP_ALL_TABS_ENABLED = true`, before
both the reuse and new-tab branches: scan `get(openTabs)` for any tab with `path === filePath`; if
found, set `activeTabId`/`focusedTabId` to it, apply `highlightTerm`, `setPendingLineJump(found.id,
targetLine)`, and return. Applies to `newTab` too (Ctrl+click focuses the existing tab). Guarantees
one path → one model per window, which also makes the first-match `onNoteSaved` reconcile correct.

**Verify (Boss):** Gate 6 — open a note in a tab, then open/Ctrl+click it again → jumps to the
existing tab, no second tab, no clobber. A wikilink with `#heading` to an already-open background
tab activates it and lands on the heading.

## §7 — Simplify + diff-scoped safety sweep *(commit `§7 — simplify + safety-inspection fixes`)*

Run `/simplify` on the full `§1..§6` diff. Run the diff-scoped safety inspection:
`Workflow({name:'safety-inspection', args:{files:[store.ts, noteSession.ts, noteModel.ts,
+layout.svelte, NoteEditor.svelte, NotePane.svelte, runtimeHarness.test.ts]}})`. **Fix every
confirmed finding before this commit** (WA#6).

**Verify:** full harness GREEN; `npm run check`; `vitest run`; Rust suite unaffected;
safety-inspection returns no confirmed app-killer.

---

## Rollback

Per step: revert the single commit. Whole change: set `NAV_FLUSH_ENABLED=false` +
`DEDUP_ALL_TABS_ENABLED=false` (one line each) → prior behavior, zero data migration (models are
ephemeral). The write-ahead net (`localStorage constellation-wab`) is the safety floor throughout.

## After the build (Phase 4)

Per-cycle **whole-app** `safety-inspection` sweep → append register to the Charter. Deliver the
**8-item Editor-Surface Gate** Boss test as a tutorial. Docs in the same close: orientation v-bump
(SO #6), `SESSION-LOG-2026-07-08`, MoCh, help/manual (Saving-and-Recovery note), memory update.

## Test surface summary (what proves what)

- **Logic harness** (`runtimeHarness.test.ts`): Recipes I/J/K — flush-before-replace, abort-on-fail,
  markSaved swap-guard. Re-runs on every future editor-content change.
- **Boss test** (8-item Gate): the component wiring — the openTabs-ordering, the Focus $effect, the
  nav-token, the dedup activation — that the logic harness cannot mount.
