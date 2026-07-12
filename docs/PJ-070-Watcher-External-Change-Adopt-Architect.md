# PJ-070 — Watcher External-Change Adopt — Architect

*/migration Phase 1 · 2026-07-12 · workflow `wf_2c3313ab-542` (12 agents: 4 census + WA#5 prior art + 3 competing designs + 3 adversarial refuters + synthesis)*

## Concept (the horse)
When someone edits an open note from *outside* Constellation (git-pull, Syncthing, Obsidian), the note you're looking at must quietly become the new truth — so your next keystroke builds on their edit instead of silently erasing it.

## The bug (mechanism — read off the code)
Under Single-Ownership (MIG-076) the open editor's source of truth is the note **model** (`noteModel.ts`), not the store's `tab.content`. The main-window watcher flush at `+layout.svelte:3220-3230` reads the changed file and updates **only** `{ ...t, content }` in `openTabs` — it never calls `adoptDisk` into the model and never bumps `tab.reloadVersion`. The `NoteEditor` `{#key}` is `tab.id|tab.path|(reloadVersion ?? 0)` (`NoteEditor.svelte:454`) and NotePane seeds **only at mount** via `seedBody` — so a content-only store update with no `reloadVersion` change does **not** remount and does **not** re-seed. Both the model *and* the mounted editor keep the **stale** body; the next keystroke marks the stale model dirty and the debounced `editor_save` durably **overwrites the external edit**, then re-indexes so search agrees with the stomp.

Deterministic reproduction: `tests/mig-076/runtimeHarness.test.ts` **Recipe O** (3/3) — RED loses the external edit; GREEN (`externalChange`/`adoptDisk`) preserves both; DIRTY shows a mid-edit model correctly refusing the adopt (local wins). Running-app write-journal reproduction: `lab/reports/PJ-070-reproduction.md`.

## Prior art (WA#5) — the industry-dominant pattern is **adopt-if-clean, prompt/keep-both-if-dirty**
Every mature editor uses the same two-branch reconcile keyed on the in-memory buffer's dirty flag:
- **VS Code** — external edit + clean buffer → silent auto-reload; dirty buffer → never clobbers, surfaces a conflict. ([issue 12452](https://github.com/microsoft/vscode/issues/12452))
- **JetBrains IntelliJ** — "Synchronize files on frame activation" (default on): clean = silent refresh; dirty external change = File Cache Conflict resolution. ([File Cache Conflict](https://www.jetbrains.com/help/idea/file-cache-conflict.html))
- **Sublime Text** — clean → silent reload; unsaved → "file changed on disk, reload?" modal. ([forum](https://forum.sublimetext.com/t/the-file-has-changed-on-disk-should-reload/33926))
- **Obsidian (NEGATIVE example — closest peer, *same bug as PJ-070*)** — a local-first file-over-app Markdown editor that does **not** reload an open note on external change; the pane keeps the stale copy and the next save clobbers. This is the documented data-loss cautionary tale we are fixing. ([forum](https://forum.obsidian.md/t/obsidian-doesnt-reload-the-current-file-when-it-is-c))

Constellation already ships this exact mirror **read-only** in `adoptFreshDiskIntoSS` (`SecondScreenPage.svelte:735`) — PJ-070 is the main-window twin.

## Options

| Option | Mechanism | Speed | Effort | Risk | Harness RED→GREEN |
|---|---|---|---|---|---|
| **A — Inline in `+layout`** | Guarded adopt inside the watcher loop (`drainCidEnsure` shape) | Fast | Moderate | **Medium** | **No** — logic in a Svelte file vitest can't drive |
| **B — Extract `adoptExternalChangeIntoTabs()` in `store.ts`** ⭐ | One freshness-gated store fn both ingress paths (watcher + `onNoteSaved`) call | Fast | Moderate | **Medium** | **Yes** — importable, testable at the store boundary |
| **C — `mode:'force'/'freshness'` flag on `reloadTabsFromDisk`** | Overload the shipped force-adopt primitive | Fast | Medium | **High blast radius** — widens a primitive the rename-cascade + task-toggle depend on | Partial |

Dirty-conflict policy is identical across all three (clean=adopt, dirty=refuse+signal) — it is a separate Boss ruling below, not an option differentiator.

## Invariants that must not break
1. **App's-own-write echo suppression** — keep `wasRecentlyWritten` (`+layout.svelte:3245`), Rust `watcher_suppress`, and `adoptDisk`'s `composeModel===diskContent` guard (`noteModel.ts:269`). Three backstop layers; our own saves trigger neither adopt nor remount.
2. **Dirty model never clobbered** — route through `adoptDisk` (refuses when dirty, `noteModel.ts:268`); **never** `reloadTabsFromDisk`/`openNoteModel` force-adopt on this path (APP-KILLER #2 / LL-014).
3. **Cascade path unchanged** — `cascade:rewrote` (`+layout.svelte:3277`) and `toggleTaskReconciled` (`store.ts:698`) stay force-adopt; the watcher adopt must skip any path inside the cascade window (`isCascading`).
4. **Both halves move together** — `adoptDisk` (model fresh + left clean) **and** a `reloadVersion` bump so the `{#key}` remounts NotePane and reseeds via `seedBody`.
5. **Bump only tabs that actually adopted** — no needless teardown / CM6-undo loss.
6. **⚠ Teardown must not re-stale** *(refutation-confirmed break in A & B as naively specified)* — the mark-cascading gate must span the **async** `{#key}` teardown. Svelte 5 fires the teardown on a LATER microtask; a synchronous `clearCascading` runs first, so `NotePane.onDestroy→doFlush→handleFlush` misses `if(isCascading)return` (`NoteEditor.svelte:301`) and flushes the stale text via `editBody` (`setBody` has no staleness guard). **Clear cascading AFTER the flush microtask (`await tick()`), not synchronously.**
7. **⚠ Focus mode is first-class** *(refutation-confirmed break — the 2026-06-12 corruption site)* — FocusPane is **not** under the `{#key}` and ignores `value` after mount (`FocusPane.svelte:229`), so a `reloadVersion` bump does nothing to it; and the naive "set `focusMode=false` to reseed" **unmounts** FocusPane → `onDestroy→flushNow()` durably writes the **stale** body. Focus needs its own suppressed reseed/exit path that gates `commitFocusSave` (`+layout.svelte:7881`).
8. **No keystroke-hot-path IPC** — reuse the `read_note` already on the 300 ms watcher flush.
9. **Burst scaling O(open ∩ changed)** — intersect with open tabs before adopting; explicit per-read `.catch` so a deleted file can't reject the batch; don't block the background reindex (`BURST_AWAIT_CAP=250`).
10. **`clearWriteAhead` only on ADOPTED paths** — a dirty tab that refused the adopt still holds unsaved edits in its write-ahead recovery buffer; clearing it would destroy the net.
11. **Preserve** reindex/tree/stats side-effects (`+layout.svelte:3193-3216`).

## In-pass scope (fix-what-you-discover / WA#6)
- **Sibling gap** — `onNoteSaved` (`+layout.svelte:3377-3391`, the SS→main save path) adopts via `externalChangeNoteModel` but **omits the `reloadVersion` bump**, so an SS save leaves the main model fresh yet the on-screen NotePane stale until some other remount. Fold it through the same shared helper — closes it in-pass, one home, no drift.
- **Stale citations** — `SecondScreenPage.svelte:730-733` cites `+layout.svelte:3320`/`:3223` for the onNoteSaved adopt / cascade reload; today they are `:3388`/`:3278`. Correct in the same commit.

## Residual (needs a Boss ruling — see below)
The **dirty-model + rapid-edit-during-the-300 ms-debounce** cases: `adoptDisk` correctly refuses (local work never clobbered), but the external edit is then **silently lost** on the next save (last-writer-wins). The rapid-edit window can be **narrowed** by doing the tab-adopt *before* the reindex/loadAllStats awaits. The full loss can be **eliminated** only by a conflict policy (below). The "§E conflict dialog" `noteModel.ts:256` points to **does not exist** (verified vaporware) — per WA#6 this cannot ship as "documented."

## Migration / rollback
- **Feature flag** `WATCHER_ADOPT_ENABLED` (default on): false = today's content-only update — a one-line revert.
- **First boot / schema** — none; pure in-memory + IPC read, no persisted-state change; rollback leaves nothing behind.
- **Mid-burst interrupt** — the flush is idempotent (echo guard makes a re-run a no-op); a partial batch re-fires on the next event.

## Recommendation
**Option B.** It is the industry-dominant reload-if-clean/refuse-if-dirty rule (VS Code / IntelliJ / Sublime converge; Obsidian's *not*-reloading is exactly our bug), Constellation already ships the read-only mirror in `adoptFreshDiskIntoSS`, and B makes the logic an **importable, harness-testable** store function (closes Recipe O's "did the wiring actually adopt" residual) while fixing the `onNoteSaved` sibling gap in the same pass — one home, no drift. It beats C because C widens a load-bearing primitive the rename-cascade depends on (a flipped default there silently undoes a rename cascade — an APP-KILLER on a currently-safe path; "secure what's achieved, never muddle"). Mandatory in the Plan for **all** options (refutation-confirmed, running-app Editor-Surface-Gate items, not static-check-provable): invariant #6 (deferred `clearCascading`) and invariant #7 (Focus-mode suppression + reseed).

**Boss decision to ratify — the dirty-model conflict policy** (see next message).
