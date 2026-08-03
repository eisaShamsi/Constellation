# PJ-207 — The Index Repair · `/migration` Phase 1 (Architect)

**Date:** 2026-08-03 · **Status:** awaiting the Boss's option pick
**Evidence:** `lab/reports/PJ-207-REPRODUCTION-2026-08-03.md` (reproduction + the first timings ever taken)
**Method:** a 58-agent whole-ecosystem sweep (285 confirmed surfaces, 256 must-bring) followed by a
3-design / 3-judge panel. Every claim below carries a `file:line` someone actually read.

---

## 1. The concept (the horse)

> **"Did my notes change while Constellation wasn't watching — and can I make the index agree with
> them again?"**

Write-Time Derivation is correct while the app is running, and **structurally blind to the interval
when it is not**. Nothing hooks a write that never came through Constellation — a `git pull`, a
Syncthing round, an edit in Obsidian, a file dropped in Explorer. This feature answers that interval
and nothing else. The honest name for it is **reconciliation**, not rebuild.

## 2. Where we actually are

- **60 of 7,824 notes** in the live universe are newer on disk than in the index; **57** hold words
  that are consequently unsearchable. Largest drift **55 days**.
- The pass that would fix it, `reconcile_filesystem` (`search.rs:10468`), has **no user-reachable
  route**: its only frontend door is gated on a completely empty index (`+layout.svelte:2892`) or a
  library being added (`:4690`, `:5971`, `:5984`). `cache_reconcile` (`cache.rs:1511`) is registered
  at `lib.rs:506` with **zero callers**.
- The health message names **"Settings → Rebuild Index"** in **all 15 locales**. No such control
  exists — only orphan CSS (`SettingsModal.svelte:2868`) left by the button MIG-013 removed.
- Measured writer-lock hold of the pass's two unbatched tail passes: **13.2 s** + **20.6 s**. The
  walk's connection waits 30 s for the lock; every user save waits **5**. A save landing inside
  either window fails *and* freezes the window, because it holds the one mutex 71 call sites need.

## 3. Boss rulings already made (constraints, not options)

1. **Placement** — a permanent control in **Settings → Index** *and* a **Repair now** action on the
   health alert bar. `docs/concept-papers/29-settings.md` is amended (wording in §7 below).
2. **What it repairs** — default **catch-up** (only files whose contents changed on disk), plus a
   separately-confirmed **Full re-read**.
3. **Detection** — a post-paint **stat-only drift check** (measured **160–590 ms**, no file reads,
   no writes) that surfaces a notice *only when drift exists*. This is "Criterion 4"
   (`lab/boot-perf/BOOT-BUDGET.md:101`, *"Still not implemented"*), specified 2026-04-15.

## 4. Options

| # | Option | Speed | Effort | Risk |
|---|---|---|---|---|
| **A** | **Repair note-by-note behind one exclusive gate.** The existing bulk walker keeps its four callers and gains no new ones; the new doors drive the same per-note primitive the watcher already uses. | check 160–590 ms · catch-up on 60 notes ≈ seconds (per-note cost **unmeasured**) · full re-read **unmeasured** | **6 commits** | **Medium** — deletes nothing, drops no trigger, holds the writer one note at a time |
| B | **Re-found the walker.** Remove it as a callable symbol; one guarded runner; the derived-view tail collapsed into one body a compiler token protects. | similar | 10 commits | **High** — makes trigger *creation* conditional on an in-process flag (`search.rs:4831`, `mig108.rs:2181`, `search.rs:1889`) and disables WAL checkpointing for the whole run (`search.rs:9730`) |
| C | **Persisted work-queue ledger** — chunked, resumable across sessions; deletes stale rows; removes the five-pass self-heal. | similar | 9 commits | **High** — uncapped delete of federated rows that hold earned link data (`search.rs:10855`); removes a self-heal its own default mode does not replace |
| D | Keep deferring (the 2026-05-04 memo) | — | 0 | **Unacceptable** — the drift is measured, and LL-027 is violated verbatim |

**Judge scores** (0–10, adversarial, per lens):

| | safety | honesty | half-sweep resistance |
|---|---|---|---|
| **A** | **7** | 6 | 5 |
| B | (lowest) | **7** | **8** |
| C | 5 | 5 | 7 |

### Recommendation: **A**, with grafts

A is the only design that **destroys nothing**. It wins safety because every other design earns its
elegance by deleting something load-bearing: B disarms trigger creation app-wide behind a flag whose
failure mode is silent (live saves keep running while `outgoing_*` quietly freezes) and pauses WAL
checkpointing for the whole run; C queues federated index rows for deletion **uncapped**, and those
rows carry the weight / confidence / traversal data `CLAUDE.md` names `search.db` the system of
record for — then the boot reconcile re-adopts them next launch, so it oscillates forever.

Graft into A:
- from **C** — a **50 ms yield between chunks** (the discipline already written at
  `classifier/scan_job.rs:33-38`); a **persisted queue** so an interrupted run resumes by fact rather
  than by a boot heal; a **re-stat immediately before each note**;
- from **B** — a **typed refusal** (`Started | AlreadyRunning | Blocked`), because all four existing
  callers swallow errors with `.catch(() => {})`; a per-family report **generated** from outcomes
  (`converged(n) | skipped(reason) | failed(msg)`) and never hardcoded, so a stamp-gated skip can
  never render as a whole repair; a **run-owned crash marker**.

Reject: B's trigger flag and WAL pause; C's deletes. Orphan rows route to the existing capped,
archive-first sweep (`reconcile.rs:57-60`).

## 5. Invariants that must not break

1. **No `.md` file is ever written.** This is an index-only operation.
2. **No archived link is resurrected and no confidence promotion is reset.** `index_note`'s preserve
   predicate (`search.rs:7115`) is `(traversal > 0 || weight != 1.0 || status != "active")` — it
   omits **confidence**, so a link the user promoted but never traversed is re-inserted as
   `hypothesis` with `created` reset. **This must be widened first, alone, with its own test**,
   before anything makes the walk easier to trigger.
3. **One index job process-wide.** No single-flight guard exists anywhere today.
4. **A library added mid-repair is queued, never refused** — the boot fan-out at
   `+layout.svelte:2859-2864` submits N parallel `reindex_library` calls with `.catch(() => 0)`; a
   naive gate would silently refuse N−1 and re-open the LL-027 / BUG-022 cold-start gap.
5. **The writer lock is held one note at a time**, with a yield between chunks.
6. **Boot stays walk-free.** The drift check reads no file bytes.
7. **Nothing is deleted** except through the existing capped, archive-first path.
8. **Every failure is counted and shown.** The health bar clears only on **zero** failures.
9. **All 15 locale strings name a control that exists.**
10. **No write lands in a universe other than the one that was active when the run started.**

## 6. Back-fill / migration / rollback

- **Flag-gated.** Removing the flag leaves the shipped code inert.
- **The first run is the back-fill.** There is no separate migration step and no schema rewrite of
  existing data (a small `repair_queue` table is additive).
- **Cancel or crash leaves a partly-repaired index** — which is a correct state: the next drift check
  finds the remainder. This is the resumability contract the scorecard already advertises
  (`SettingsModal.svelte:2525-2543`, Criterion 5).
- **Rollback** is a revert; no user data shape changes.

## 7. The concept-paper amendment (exact wording)

`docs/concept-papers/29-settings.md` currently forbids this in four places (§3, §7, §9, §10).
The distinction that resolves it is **a press is not a change**:

- **§3, added:** *"**Not a maintenance engine.** Settings HOSTS the controls for maintenance actions
  the user explicitly asks for (Index → Repair), but it neither owns nor performs the work: the
  button submits a request to the one repair runner and renders its report. Settings still edits no
  note and derives no view of its own."*
- **§7, replacing the Rule-8 line:** *"Rule 8 status: ✅ reads-persisted. Settings derives nothing and
  re-walks nothing of its own. It hosts one explicitly-pressed maintenance action that submits to the
  index repair runner; PJ-207 added **no new walker** — it added a door to one that has shipped since
  2026-04-08 — and reduced the user-reachable walker count from two to one."*
- **§9, replacing the regression guard:** *"Confirm that CHANGING A SETTING never fires a note write
  or a reindex. The only reindex Settings may cause is one the user EXPLICITLY REQUESTED by pressing
  a maintenance button, which is an action, not a setting change. A control that starts work on
  toggle — rather than on an explicit, separately-labelled action press — is forbidden."*
- **§10, amending the Editor-wiring acceptance box:** *"…with no note write, no reindex, no model
  touch; explicitly-pressed maintenance actions are exempt from the 'no reindex' clause **and from no
  other clause** — they still never touch note content, the save path, or the Editor's in-memory
  model."*

**Rule 8** is satisfied on its own terms: it mandates write-time derivation, and write-time
derivation *has no write to hook* when the change did not come through Constellation. Rule 8 is
silent on out-of-band change. The **2026-05-04 memo** that deferred this button set its own reopen
condition — *"Reopen if Boss flags index desync"* — which has now fired and is measured (60/7,824).
Its three demands (rebuild semantics; a cancellable streaming-progress IPC; a decided home in
Settings) are answered by Mode A/B, the `scan_job` chassis, and Settings → Index. It is overturned in
writing, in the same commit.

**LL-027** is honoured on both counts: the recovery path is **shipped and verified** rather than
promised in a comment (its canonical violation was three claimed triggers of which two never
existed — *including this very button*), and its stated preference for gated-automatic over manual is
met by the post-paint drift detection.

## 8. Two corrections Phase 2 must carry

1. **Scoping one line does not close Charter W2-9.** Changing `reconcile_filesystem` to walk the
   non-recursive library set does **not** stop foreign cUniverse notes entering the active index —
   the boot reconcile re-adopts any `.md` under an accessible root (`reconcile.rs:89`, `:286`), and
   its roots come from the recursive set. The scoping decision must cover both passes or it is
   itself a half sweep.
2. **The per-note primitive reports success while its maintenance fails silently.**
   `reindex_single_note` calls `ctse::hooks::on_note_indexed` (`search.rs:11122`),
   `maintain_incoming_after_save` (`:11136`) and `maintain_sky_after_save` (`:11150`) — all three are
   `eprintln`-only and still return `Ok(())`. A per-note repair loop built on it can print
   "Repaired 60 · 0 problems" while 60 term-index deltas silently failed.

## 9. Open questions for the Boss

1. **Full re-read may take tens of minutes** and has never been measured. Ship it in this migration,
   or hold it until it is measured on the real universe?
2. **Foreign rows from linked universes** are already in the index (Charter W2-9). Report them only
   (my default — they carry earned link data), or offer removal behind its own confirmation?
3. **The 34-second freeze** also hits the four *existing* callers today. Fix it inside PJ-207, or
   file it as its own job?
