# APP-KILLER #1 — Save Durability (mark-clean-before-durable-write) — Architect

**Status:** Architect complete → awaiting Plan approval
**Opened:** 2026-07-08 · Safety Audit **G2** (save / model-ownership) · Owner: Claude · Boss: Eisa
**Analysis:** workflow `wf_16260085-719` (4 agents): save-site census · durability/recovery machinery · WA#5 (VS Code / Obsidian / POSIX) · adversarial design (8 refutation points).

---

## Concept (the horse)

> **One note has one in-memory authority; "clean" is a fact about disk, so it may only become true *after* the disk write is proven durable.** Every save site earns the right to mark clean by going through the single primitive that awaits the write, keeps the note **dirty + buffered + loud** on failure, and clears the recovery net only when the bytes it wrote are still the bytes buffered.

## 1. The gap (confirmed from code)

`noteSession.ts:84 save()` is **already correct** — `compose → await write → markSaved` (markSaved is skipped if the awaited write throws). Four store.ts flush sites use it (`:549 :698 :2535 :3498`). But the **five component save sites bypass it** with an inlined `markSaved`-**before**-write, and swallow the error:

| Site | markSaved before write? | write-ahead net? | error surfaced? |
|---|---|---|---|
| `NoteEditor.handleSave` :233 (debounced) | **yes (bug)** | **NO** | swallowed `.catch(()=>{})` :280 — **APP-KILLER** |
| `NoteEditor.handleFlush` :297 (teardown) | yes | yes (:308) | swallowed :329 |
| `NoteEditor.handlePromote` :194 (stage) | yes | **NO** | swallowed :207; **also no reindex** |
| `store.ts saveTabContent` :765 (props/tags) | yes | **NO** | propagates → callers swallow/console.error |
| `+layout commitFocusSave` :1490 (focus) | yes | yes (:1495) | console.error only (invisible in release) |

**The APP-KILLER chain (confirmed end-to-end):** a transient `.md` lock (Syncthing / OneDrive / Defender) rejects the debounced `handleSave` write → `markSaved` already ran so the model is **falsely clean** (`isDirty=false`), the edit lives only in memory, there is **no write-ahead net** on this path, and the error is **swallowed**. On tab close `doFlush` passes `needsDiskSave = dirty = false` → nothing written. Worse: a later wikilink rename's `flushAllTabsInLibrary` (`store.ts:695 if (!isNoteDirty) continue`) **skips the falsely-clean tab**, and the cascade rewrites the **stale disk content** — the exact **F2** pre-cascade-staleness loss the code claims to prevent, defeated by the false `isDirty`.

**Machinery facts:** the write-ahead buffer (WAB) is localStorage-backed (`constellation-wab`), survives restart, and is recovered on tab reopen / boot via `resolveNoteContent` (fail-closed: cid_cn must match). Only `handleFlush` + `commitFocusSave` populate it → the other three paths have **no net**. Under `SINGLE_OWNERSHIP=true` the cascade gates on `isNoteDirty`, **not** the WAB, so the net doesn't rescue the cascade — the false-clean is the root. **No user-surfaced save error exists anywhere** (all swallow or console.error; devtools off in release → invisible). Doc drift: `+layout:1483` claims the focus error is "surfaced (W1-5)" — it only console.errors.

**Rust write is already durable/atomic** (verified this session): `write_note` → `write_gate::gate_write` → `atomic_write` = same-dir temp + `sync_all` fsync + atomic rename. So a resolved write is genuinely durable (no torn file); the frontend fix is the missing half.

## 2. WA#5 — the universal rule

VS Code is the reference: an explicit dirty bit; **model cleared to clean only when the write promise resolves**; on rejection the model **stays dirty**, a save-error notice is surfaced with Retry, and a **hot-exit backup journal** independently persists dirty buffers so a failure can never mean silent loss. Obsidian: retain buffer + surface "failed to save". POSIX: clean trails fsync. **Silently swallowing a save error is a data-loss defect everywhere.** This fix deletes five divergences from a pattern the app already implements correctly at four sites — nothing inventive.

## 3. The hardened primitive (`noteSession.ts`, replacing 84–95)

`save()` grows into the full durability contract, still a **pure module** (all side effects injected → the headless harness drives the real path): `compose → setNet(BEFORE write) → try await write → catch: onError + return {write_failed} (NO markSaved, net RETAINED) → markSaved → clearNetIf(compare-and-clear) → onSuccess`. Supporting `store.ts` additions: `clearWriteAheadIf(path, content)` (compare-and-clear — a newer net is never wiped by an older completed write); `saveHealth` writable Map + `reportSaveFailure`/`clearSaveFailure` (path-keyed → coalesced, one entry per note, never a per-tick toast storm); `standardSaveEnv({tab, cursor, scroll, reindex, embed, cece, origin})` factory wiring the env — **every component site calls this; no site hand-rolls ordering again.**

## 4. Invariants (audit checklist)

- **INV-1** Single durability gate — `markSaved` on a save path is reachable only inside `save()`'s resolved branch (grep-enforceable).
- **INV-2** Net-before-write — every disk write preceded by `setNet` with the same content; net retained on failure.
- **INV-3** Compare-and-clear — net cleared only when its content still equals what this save persisted.
- **INV-4** Dirty-until-durable — `isDirty` stays true from first edit until the composed version's write resolves.
- **INV-5** No silent swallow — every save failure calls `onError → reportSaveFailure` (user-visible). Zero bare `.catch(()=>{})` on a save write.
- **INV-6** Identity guard preserved — `compose` still refuses `path_mismatch`.
- **INV-7** Derived-surface-on-success-only — reindex/embed/broadcast/CECE run only in the resolved branch — now including **handlePromote** (ships no reindex today; WA#6).

## 5. Adversarial refutation (all 8 survive — see workflow output)

1. **Version semantics under typing during await:** compose snapshots V; a mid-write `setBody` → V+1; `markSaved(V)` on success leaves `isDirty=(V+1>V)=true` → correctly still dirty. Not a regression (the ≤1.5 s pre-crash window is universal). 2. **`setNet(path, content, 0, 0)` on the debounced path:** `resolveNoteContent` treats content as load-bearing, cursor/scroll as best-effort (already defaults 0,0) → content intact, note recovers at top. Safe. 3. **double/stale net:** closed by `clearWriteAheadIf` (compare-and-clear) — the one genuinely new hardening. 4. **saving-guard/debounce:** keeping `isDirty=true` until resolve is exactly the F2 repair; failure → dirty + `saving` released in `finally` → next debounce is a natural retry. 5. **cross-window:** SS mounts the shared NoteEditor → inherits the fix; independent WAB Maps + shared localStorage origin handled by compare-and-clear. 6. **error surface:** must be built (no toast infra) — Boss decision #1. 7. **harness:** RED (inlined shape falsely-clean) → GREEN (routed) + type-during-await + compare-and-clear + F2-chain. 8. **full `/migration`:** yes (feedback_bug023).

## 6. Phased plan (each step = one commit + Reproduce-First verification)

**Phase 1 — primitive, proven headless (additive; app behavior unchanged).**
- **Step 1** — new `save()`/`SaveEnv`/`SaveOutcome`; `clearWriteAheadIf`; `saveHealth`+report/clear; `standardSaveEnv`; migrate the four already-correct sites to the env signature (gain net + surface, keep order). **Verify:** the 6-case harness (`tests/mig-076/runtimeHarness.test.ts`) — RED proven then GREEN.

**Phase 2 — reroute the five wrong sites (one commit each + its Editor-Surface Gate test).**
- **Step 2** `handleSave` (delete inlined markSaved + swallowed catch; route via `standardSaveEnv`) — Gate 1. **Step 3** `handleFlush` (net exists; markSaved→success, `clearWriteAheadIf`) — Gate 3. **Step 4** `handlePromote` (route + add missing reindex, INV-7) — Gate 5. **Step 5** `saveTabContent` (keep auto-date `editNoteProps`, then delegate; drop markNoteSaved-before-write) — Gate 5 + tag-add. **Step 6** `commitFocusSave` (route; fix the false :1483 comment via `saveHealth`) — Gates 2 + 4.

**Phase 3 — surface, cross-window, full gate.**
- **Step 7** `saveHealth` banner in main + SS `+layout`; i18n ×15; Retry wired — Gates 7 + 8. **Step 8** `/simplify` + diff-scoped `safety-inspection` + full 8-item Editor-Surface Gate run + `/migration` Audit (invariants/drift/migration-path).

## 7. Editor-Surface Gate coverage

1 NotePane burst → Step 2 · 2 Focus enter/type/exit (no spurious enter write) → Step 6 · 3 tab switch-away+return (NotePane+Focus) → Steps 3,6 · 4 tab switch while in Focus → Step 6 · 5 PropertyEditor (embedded+standalone)+promote → Steps 4,5 · 6 rename linked-probe-pair (F2) → Step 2 + F2 harness (Step 1) · 7 second-screen edit+sync → Step 7 · 8 restart/restore → Step 7 (WAB). Harness asserts on-screen === disk after every transition.

## 8. Decisions for Eisa (defaults in **bold**)

1. **Error-surface UX (blocking Step 7).** No toast/notification infra exists. Proposed: a persistent, path-coalesced **"save health" banner** (Obsidian pattern), non-blocking, auto-dismiss on next success, i18n ×15, with a **Retry** button. Confirm: top banner **(recommended)** vs status-bar chip; Retry ships now vs later.
2. **Bounded-retry policy.** The 1500 ms debounce auto-retries only while the user keeps typing; if they walk away after a failure the dirty edit sits until the next interaction. **Recommend a lightweight timer re-attempting a failed save every ~10 s while it stays dirty** (drive comes back → it persists unattended; one timer, `onDestroy`-cleaned). Approve N=10 s, or decline (rely on next-interaction retry).
3. **Rust `write_note` atomicity — RESOLVED (no action).** Verified: already atomic + fsync-durable via `write_gate::atomic_write` (temp + `sync_all` + rename). The frontend fix sits on a durable write.
