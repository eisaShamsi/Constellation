# Constellation Pending Jobs

**Version 1.54 | 2026-07-27**

> **What changed in v1.54** (**MIG-104 Slice 7 shipped — the ledger's load is now bounded. The build's safety inspection found an app-killer in Slice 7's OWN new code and it was fixed before the commit; the sweep also returned a 50-finding whole-app register.**):
>
> ### ★ THE RE-PRIORITIZED BACKLOG
>
> **► NEXT ACTION — MIG-104 Slice 8 + 8b**, *the Boss's time machine substrate*: **archive-before-purge**, and the hook MUST go **BEFORE the `DELETE FROM note_meta`** (`search.rs:9845`) — FK enforcement makes the CASCADE fire there, so a hook at the later explicit purge archives **NOTHING** (proven by `tests_stage0_delete_order_defect`). **8b = the note BODY** (Boss decision #6), so the machine survives an emptied Recycle Bin. Then Slice 9 (continuous note-history mirror, Boss decision #4) · 10 (cascade pre-delete archive) · 11 (restore rejoin) · **12 = PJ-164/C8** · 13 (gated `index_note` overlay) · **14 (adjacent defects — now also absorbs PJ-173)** · 15 (docs ×15). **Then MIG-105 Phase 2 Plan.** MIG-106 (LINK authoring surface) is open → PJ-169.
>
> **CLOSED THIS JOB:**
> - **MIG-104 Slice 7 — the snapshot + compactor.** `earned.snapshot.jsonl` (one line per earned link + one per note decision) + compaction on a **2 MB byte threshold, never a timer**: unique temp → fsync → persist → **rename the tail aside, never delete**. Load = snapshot + tail, both bounded. `note-history.jsonl` is structurally excluded — `maybe_compact` has **no stream parameter**, so it cannot be pointed at the stream whose records ARE the payload. Live store unchanged (6,222 B, ~340× below the threshold); Rust **1261/0**.
> - **The Slice-6 clause that never shipped** — *"restores review priority too."* `priority` records had been appended **and fsync'd** since Slice 4 and **never read back** (the fold's key function required a target, so every one was dropped). Losing `search.db` still cost the user every review priority they had set. Now folded, snapshotted and restored, with `-1` → SQL `NULL` and the same one-row-or-skip rule that governs an ambiguous link.
>
> ### ★★ THE APP-KILLER THIS JOB CAUGHT IN ITS OWN NEW CODE
> **`link_life.rs:592` — unguarded TOCTOU in the compactor**, confirmed by three independent verifiers on the per-build safety inspection, ~1 hour after the code was written and while 52 tests were green. `maybe_compact` folded the tail, spent tens of milliseconds writing+fsyncing a multi-MB snapshot, then renamed the tail aside — and `append` took **no lock**, while `constellation_link_traverse` / `record_decision` / `set_review_priority` all append from Tauri command threads after deliberately dropping the DB guard. Records landing in that window were moved into `earned.tail-*.jsonl`, **which nothing reads back**. Worse than absence: the restore treats the ledger as authoritative for *decisions*, so the next boot writes the **pre-decision value back over the DB** — silently un-retiring a link or reverting a priority, every step logging success. **Fixed** with a module-level `FILE_LOCK`. **RED proven over 3 runs (666/730 and 1,110/1,168 decisions lost per run); GREEN over 3.**
>
> **Two further dead guards fixed in the same pass (WA#6):** `refuse_write` was set ONLY inside `quarantine`, which returns its *own* report — so the restore's *"do NOT write from a store we could not read"* was **structurally unable to fire** while reading as live protection (the LL-035 shape). The reader now OBSERVES the quarantine on disk. And the restore's log line ended `— stamped` while the pass is deliberately **unstamped** — a false success claim, removed.
>
> **NEWLY FILED:**
> - **PJ-173** — `ConfidencePicker.svelte:61,70` wraps the ONLY user entry to trust/retire in `catch { /* ignore */ }`, discarding the error from a Rust path that deliberately **fails closed**. The user clicks *Contested*, the menu closes, nothing happens and nothing is said. Needs an error surface + 15 locales + a design call → **fold into MIG-104 Slice 14**.
> - **PJ-174** — **the 2026-07-27 whole-app sweep register**: 50 unique confirmed (**3 APP-KILLER · 20 HIGH · 24 MED · 4 LOW**), full text in `lab/reports/SAFETY-INSPECTION-2026-07-27-whole-app.md`, summarised in the Charter. **Owed a per-CYCLE triage pass with the Boss — deliberately NOT absorbed into MIG-104.** Headline: **APP-KILLER `+layout.svelte:6779`** (the rename cascade's freeze/flush protection sets are snapshots taken *before* a multi-second walk, so a tab opened DURING the walk is never frozen, never flushed, yet still force-adopted from disk) and **two APP-KILLERs in `PropertyEditor.svelte:851/852`**.
> - **PJ-175** — `link_life::quarantine` has **no production caller**: nothing yet decides a store is "structurally unusable". Its *effect* is now observable and both guards honour it, and the compactor's own "the fold is empty while the tail is not" refusal covers the shape that actually occurs — so there is **no silent-loss exposure today** — but the detector itself is an unfinished Slice-3 mechanism. Boss ruling wanted rather than silently parked.
>
> **PJ-166 STRUCK A FOURTH TIME.** The inspection was invoked diff-scoped and returned `mode: "whole-app"` — `args.files` ignored again. The sweep's value was real (it is what caught the TOCTOU), but **the per-build gate the standing order requires still does not exist**, and a 31-minute / 10.1 M-token whole-app run cannot serve as one. This is now the single biggest gap in the Safety Inspection standing order; it should be fixed before the next build rather than worked around a fifth time.
>
> **STILL OPEN:** **MIG-104** (Slices 8–15) · **PJ-145 / MIG-105** (Plan → Build → Audit) · **PJ-164** (C8 = Slice 12) · PJ-150 · PJ-152 (`custom_stages` destroyed by rename/attach/detach) · PJ-158 · PJ-159 (939 MB orphan DB) · PJ-160 · PJ-162 · PJ-163 · PJ-166 (**4th strike**) · PJ-167 · PJ-168 · PJ-169 (MIG-106) · PJ-170 · PJ-171 (CI runs zero tests) · **PJ-172** (Sight perf tests still need a permanent serial lane — run manually as one this job) · PJ-140 (~37) · PJ-142/143/144/146/147/148 · PJ-137 · PJ-135 · PJ-125–139.
>
> ---

**Version 1.53 | 2026-07-27**

> *(See `Constellation Pending Jobs v1.53.md` — the trail is durable, never overwritten.)*
