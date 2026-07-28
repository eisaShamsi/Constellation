# Constellation Pending Jobs

**Version 1.55 | 2026-07-28**

> **What changed in v1.55** (**Boss-ruled DIVERT from the MIG-104 cascade to the three confirmed APP-KILLERs. AK-1 CLOSED — and it was FOUR holes, three of them found by inspecting the fix for the first. One new law, LL-038.**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — AK-2/3 (PJ-174): the props-ownership APP-KILLERs** (`PropertyEditor.svelte:851/:852`). **They are ONE root cause, not two:** props have no single owner in the UI layer — each PropertyEditor keeps its own `editableProps` derived from `tab.content` (a projection the model-based writers deliberately never update or notify), **two instances mount for the same tabId** (NotePane-embedded + right-sidebar), and every save REPLACES `model.props` wholesale from that snapshot while mutating `tab.content` directly ("no store.update = no cascade"), so nothing ever re-seeds. This is the **content-integrity class**, whose three-strike law is already spent (LL-014; BUG-012 / 015 / 019 / 023) — so **Solve-the-Class governs: the fix is single content ownership for PROPS** (the model as the authority every reader and writer goes through), designed whole, proven against the reproduction harness, landed as ONE validated swap behind a toggle. MIG-076 did exactly this for the note BODY and stopped there. **Likely `/migration`-sized — design to be brought to the Boss before it is written.** Then resume **MIG-104 Slice 8 + 8b**.
>
> **CLOSED THIS JOB — AK-1 / PJ-174 #1, Boss-validated in two stages:**
> - **#1 — the protection sets were PRE-WALK SNAPSHOTS.** Freeze, save-gate and flush list were all built from `tabsInLibrary(lib.path)` before a multi-second walk, while the sidebar stays clickable. A note opened mid-walk was in none of them. **A snapshot cannot be repaired by taking it later** — the predicate is now scoped to the **library** (`markCascadingLibrary`), so it is true for tabs that do not exist at mark time.
> - **#2 — `reloadTabsFromDisk` force-adopted over a dirty model**, while its own docstring said a dirty path must never reach it and "the guard lives UPSTREAM at every caller" — upstream being the stale snapshot. Now enforced at the point of damage, routing genuine conflicts to the SAME `.conflict` sidecar + banner the watcher uses. **Force-discard is opt-in by name** (`discardLocalEdits`), because the WA#4 consumer sweep found exactly one of nine callers depends on it (PJ-102c "Discard my changes") — a blanket refusal would have silently broken that feature.
> - **#3 — the freeze overlay had the same hole**, so the pane most at risk had no overlay. `cascadeFreeze` now holds library ROOTS through a shared `isPathFrozen`, collapsing two representations of one window into one concept.
> - **#1b — APP-KILLER found by the inspection ON the #1 fix.** `renameItem` re-seeded the model from disk after its awaits with **no dirty re-check**, having cleared the write-ahead net three lines earlier. `markCascading` gates disk writes, **not** `onDocChange → editBody`, and the freeze is raised only after `renameItem` returns — so "rename the title, press Enter, keep writing" (the caret is placed in the body *by the app*) silently destroyed the typing. **My miss:** the ecosystem sweep ran `grep openNoteModel | grep -v libraries/store.ts`, excluding the file being edited; that file holds seven call sites. Fixed by mirroring the sibling `drainCidEnsure`, which already carried the guard *with a comment explaining it*.
> - **#1c — a fix that would have become a regression.** `saveTabContent`'s cascade gate returned **before** the model push, so a property edited during a cascade was neither written nor kept. Making the gate live (#1) **widened that window to the whole library**, so shipping without this would have made an existing silent loss fire far more often. The gate now sits below the model push; a control test asserts it still blocks the disk write.
>
> **Every fix RED-proven then GREEN, each with a control that it does not over-block.** Gates: vitest **54 files / 619 tests** (was 52/607) · Sight perf in a **SERIAL lane** (PJ-172) 31/31 · svelte-check **0** · Rust **1261/0**. Boss-validated: Stage 1 (typing survives a rename) and Stage 2 ("Updating links…" seen on a note opened mid-walk).
>
> ### ★★ THE LAW THIS JOB PRODUCED
> - **LL-038** — *A guard built from a SNAPSHOT protects only what existed when it was taken; scope the predicate to the CONTAINER. Never delegate a destructive primitive's invariant to its callers — enforce it at the point of damage and make the destructive path opt-in by name. Never exclude the file you are editing from an ecosystem sweep. And WIDENING a guard is a behaviour change for everything the guard drops.*
>
> **NEWLY FILED (same family as AK-1, NOT absorbed into it):**
> - **PJ-176** — **`moveItem` (`store.ts:3913`) has neither `markCascading` nor a pre-move flush** — the exact two-part gate its sibling `renameItem` carries — so a save firing during the move IPC writes to the old path. Third surface of the same concern.
> - **PJ-177** — **`deleteWithSetting` (`store.ts:3986`) never calls `closeNoteModel`**, so a deleted note's model survives and the unmounting editor's teardown flush can durably re-create the file.
>
> **PJ-166 STRUCK A FIFTH TIME.** Invoked diff-scoped, returned `mode: "whole-app"` again (84 agents, ~10.3 M tokens, 32 min) → 46 unique confirmed (**3 APP-KILLER · 10 HIGH · 27 MED · 6 LOW**), register at `lab/reports/SAFETY-INSPECTION-2026-07-28-ak1-build.md`. **The sweep is what caught #1b and #1c**, so its value is not in doubt — but the per-build gate the standing order requires still does not exist, and a 32-minute run cannot be one. **This is now the single biggest gap in the Safety Inspection standing order and should be fixed before it is worked around a sixth time.**
>
> **STILL OPEN:** **PJ-174 AK-2/3** (► next) · **MIG-104** (Slices 8–15, paused at the 7/8 boundary) · **PJ-145 / MIG-105** · PJ-164 (= MIG-104 Slice 12) · PJ-150 · PJ-152 · PJ-158 · PJ-159 · PJ-160 · PJ-162 · PJ-163 · **PJ-166 (5th strike)** · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 · PJ-172 · **PJ-173** (folds into MIG-104 Slice 14) · **PJ-174** (the remaining sweep register) · **PJ-176 · PJ-177** · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-137 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.54 | 2026-07-27**

> *(See `Constellation Pending Jobs v1.54.md` — the trail is durable, never overwritten.)*
