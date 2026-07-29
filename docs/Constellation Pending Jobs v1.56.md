# Constellation Pending Jobs

**Version 1.56 | 2026-07-29**

> **What changed in v1.56** (**MIG-104 Slice 7 shipped · the three PJ-174 APP-KILLERs CLOSED · MIG-107 built and shipped end-to-end in one session · three new laws. Six defects were caught by the Boss in live testing, three of them mine.**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — PJ-182** (below): a **reproduced** content-loss bug that silently deletes list items from the user's `.md`. It is small, it is proven, and it destroys authored data — it goes before resuming MIG-104. Then **PJ-181** (app-killer). Then **MIG-104 Slice 8 + 8b** (the time machine): the archive hook must go **BEFORE `DELETE FROM note_meta`** (`search.rs:9845`) because FK enforcement fires the CASCADE there — a hook at the later purge archives **nothing** (`tests_stage0_delete_order_defect`). 8b adds the note BODY (Boss decision #6). Then Slices 9–15, then **MIG-105 Phase 2**.
>
> **CLOSED THIS SESSION:**
> - **MIG-104 Slice 7** — `earned.snapshot.jsonl` + the 2 MB compactor. Load is now bounded by what the user has EARNED, not by how long they have used the app. Boss-validated via the in-pass discovery that **review priorities were written and fsync'd since Slice 4 and never read back** — losing `search.db` still cost the user every priority they had set.
> - **PJ-174 #1 (AK-1)** — the rename cascade's unprotected mid-walk tab. **Four holes**, three found by inspecting the fix for the first. Boss-validated in two stages.
> - **PJ-174 AK-2/AK-3 → MIG-107** — props single ownership, all six slices, five Boss-validated. Both reproductions flipped from `it.fails` to green.
> - **PJ-178** — a blank Properties row no longer reaches the file as `"": ""` (closed structurally by `addProp` refusing an empty key).
> - **PJ-179** — the stage picker now opens on the note's CURRENT stage, not the top of the list.
>
> ### ★★ THE THREE LAWS THIS SESSION PRODUCED
> - **LL-037** — *a SEQUENCING argument is not an EXCLUSION argument; and a race test must SPAN the window, not sample it.* (My comment claimed one thread made a race impossible; my first regression test passed WITHOUT the fix.)
> - **LL-038** — *a guard built from a SNAPSHOT protects only what existed when it was taken; scope the predicate to the CONTAINER. Never delegate a destructive primitive's invariant to its callers. Never exclude the file you are editing from an ecosystem sweep. WIDENING a guard is a behaviour change for everything it drops.*
> - **LL-038 rule 6** — *never hand-maintain a list that must be COMPLETE to be correct — derive it.* (`touchedKeys` was hand-marked at **3 of 16** mutation sites and silently dropped every tag edit. Boss-found.)
>
> **NEWLY FILED:**
> - **PJ-182 — ► NEXT. Content-loss, `store.ts:2009`, REPRODUCED and independently verified.** A **zero-indent** YAML block list (`tags:` newline `- alpha`) — valid YAML, what PyYAML emits, common in imported vaults — projects as an **EMPTY list**. The panel shows it as empty, and the next property write replaces the whole block: **every item is deleted from the `.md`**, with no error, and the result re-parses cleanly so nothing notices. Same for **`aliases`**, which silently breaks every backlink through that alias. `addTagToNote` is a trigger; the batch tagger multiplies it.
> - **PJ-181 — APP-KILLER, `store.ts:2448`.** The write-ahead net is restored on a `cid_cn` match with **no freshness check against disk**, and a net entry is stashed for merely-VIEWED notes. View a note → close it → it is edited externally (Syncthing / second device / git pull; the watcher ignores it because the note is closed) → reopen and the **stale** content is shown with the model born dirty → the first tab switch writes it over the newer file. `restoreSessionTabs` already solves this on the sibling path.
> - **PJ-180** — MIG-107's altitude follow-ups: a by-name `setPropByName` intent; a generic `noteProp(id, key)` read facade (which is what lets the `onStageChanged` push channel be **deleted** rather than shadowed); splitting `saveTabContent`'s two modes; a `propsCommit` draft handle; and `buildFullContent` being a **lossier** composer than the `compose()` every other writer uses.
> - **PJ-176** — `moveItem` (`store.ts:3913`) has neither `markCascading` nor a pre-move flush — the exact two-part gate its sibling `renameItem` carries.
> - **PJ-177** — `deleteWithSetting` (`store.ts:3986`) never calls `closeNoteModel`, so a deleted note's model survives and a teardown flush can re-create the file.
>
> **CARRIED, and now the biggest process gap: PJ-166 struck SEVEN times this session.** Every inspection was invoked diff-scoped and returned `mode: "whole-app"` — ~32 min and ~10 M tokens each. The sweeps earned their cost (they caught the Slice-7 TOCTOU, the AK-1 `renameItem` app-killer, and PJ-181/182), but **the per-build gate the standing order actually requires still does not exist**, seven attempts in. Fix it before it is worked around an eighth time.
>
> **STILL OPEN:** **PJ-182** (► next) · **PJ-181** · **MIG-104** Slices 8–15 · **PJ-145 / MIG-105** · PJ-164 (= MIG-104 Slice 12) · PJ-150 · PJ-152 · PJ-158 · PJ-159 · PJ-160 · PJ-162 · PJ-163 · **PJ-166 (7th strike)** · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 · PJ-172 · PJ-173 (→ MIG-104 Slice 14) · **PJ-174** (the remaining sweep register) · PJ-176 · PJ-177 · **PJ-180** · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-137 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.55 | 2026-07-28**

> *(See `Constellation Pending Jobs v1.55.md` — the trail is durable, never overwritten.)*
