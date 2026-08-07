# Constellation Pending Jobs

**Version 1.67 | 2026-08-03**

> **What changed in v1.67** (**PJ-207 is HALF SHIPPED — §1–§7 of a 15-step migration, all seven Boss-tested and passed. And the Boss installed a two-agent gate between me and his test instructions, after I invented a screen. Ultracode**):
>
> **► NEXT ACTION — PJ-207 §8**, *the index stops adopting notes that belong to a linked universe* (Charter **W2-9**, OPEN, HIGH). **Read the Architect doc's §8 correction first — scoping only the walk does NOT close it.** Everything after §8 is unstarted; **§11 is the door** — the step that finally makes the repair reachable. Plan: `docs/PJ-207-Index-Repair-Plan.md`; Architect: `docs/PJ-207-Index-Repair-Architect.md`. Ready-to-paste prompt: `docs/NEXT-SESSION-PROMPT-2026-08-04.md`.
>
> ### ✅ PJ-207 §1–§7 — SHIPPED and Boss-validated
> The defect: the app's authoritative index self-heal had **no user-reachable route**, and its error message named a "Settings → Rebuild Index" control that has never existed, in **all 15 languages**.
>
> **Reproduced on the live universe before a line was written** (`lab/reports/PJ-207-REPRODUCTION-2026-08-03.md`): **60 of 7,824 notes** had disk content newer than the index; **57** held body words absent from `note_meta.body_text`; `notes_fts` is `content=note_meta`, so those words were unfindable and nothing in the app could fix it. Largest drift **55 days**.
>
> - **§1 `3c0dc84b`** — *a judgement is earned data too.* `index_note`'s preserve predicate checked traversal, weight and archived status but **not confidence**, so a link promoted to evidence/established but never clicked came back as `hypothesis` with `created` reset — on an ordinary save. The hand-mirrored copy of the predicate at `search.rs:338` (which would have kept five tests green through a production-only widen) is deleted; both sites share one function. **First test of `index_note` in either suite.**
> - **§2 `aae51aff`** — the two dead doors, 15 orphan CSS rules (the last physical trace of the button the app kept advertising), and a dead settings key. Plus the two rollback flags.
> - **§3 `eaafe240`** — the indexer stops reporting success it did not earn. `IndexOutcome` + `WalkTally`: a `read_dir` failure used to return from an entire subtree with **no trace**, and the only number available was `COUNT(*)`, identical whether the walk indexed 7,800 notes or zero.
> - **§4 `98bca820`** — one covering index. Measured first, because the plan required the planner to actually choose it: `SCAN note_meta USING COVERING INDEX`. **1.6 MB read instead of 270.3 MB — 167× fewer bytes**, zero extra disk (absorbed by the freelist).
> - **§5 `23f6cb99`** — the review rebuild: **2.5 s → 109 ms** worst hold, **260 MB → 30 MB** resident. **The plan told me to window the orphan sweep wrongly** — an orphan can sort past every note path, where a note-derived window never reaches; RED-proven (`left: [..., "/lib/zzz-after.md"]`).
> - **`1948090b`** — Boss-found mid-test: **Manage libraries → + Add library silently did nothing** for a folder outside the universe. Rust returned a ready-made message; the frontend discarded it in a bare catch. The sidebar's flow was already correct — one concern, two implementations, one right. The wrong one is deleted, not repaired.
> - **§6 `250d9892`** — one place where derived views are rebuilt. Five hand-assembled recomputes → one, **sealed by a token whose private field the compiler enforces** (proven by a deliberate `E0603`, not asserted). §5's marker got its reader: the boot heal now runs **five** families, not three.
> - **§7 `ee2191ed`** — only one thing may walk the library. Two independent walkers with seven entry points → one submit-point with a typed outcome; `reconcile_filesystem` lost its `pub`, `reindex_library` was absorbed, the boot fan-out collapsed, the bring-in double-fire went, mutual exclusion landed both ways with defrag and MIG-108, and the walk gained the per-note cancel / universe-switch / checkpoint gate it never had. Its safety review found **ten**, two HIGH — see below.
>
> ### ⚖️ THE PLAN WAS WRONG THREE TIMES, AND EACH TIME IT WAS THE SAME SHAPE
> §5's orphan sweep · §6's incoming back-fill (routing it through the gated path would have made it a **permanent no-op** — it recomputes then stamps, so the gate it would check is the stamp it is about to write) · §6's tag-count back-fill (builds **and stamps** in one transaction, deliberately). **A builder is not a healer.** A plan verified against the code still needs verifying against the code.
>
> ### ☠️ §7's SAFETY REVIEW FOUND TWO HIGH FINDINGS IN MY OWN WORK
> Both were safety nets I disarmed *while building them*. (1) I moved the trigger drop/recreate into RAII but left the old marker-clearing code downstream reading a now-always-`None` variable — so a **failed** recreate cleared the marker it had deliberately kept, disarming the boot heal on its own failure mode, and returned `Ok`. (2) **No RAII on the single-flight flag**, against this codebase's own `mig108::RunningGuard` precedent, which says in its comment why one is needed: a panic would have leaked it for the session — no repair, no cold start, and (because defrag now defers to it) **no compaction ever**. Plus 8 more, all fixed.
>
> ### 🛡️ NEW: the test pipeline — auditor → inspector → Boss (LAW)
> Boss-mandated after I told him to look for "bands/levels" in Sky View (a 3D cloud with no bands) and to add a link type in **Settings → Links** (a page whose own text says they live in the Style Setter). His verdict: *"This test is unrealistic at all! I have designed the application, and don't know what this test is about!!!"*
> - **`.claude/agents/tutorial-auditor.md`** — BUILDS the test.
> - **`.claude/agents/ui-inspector.md`** — GATES it. **Default verdict REJECTED.**
> - `CLAUDE.md` gains **Never Describe the App Without Looking At It** (LAW) and **The Test Pipeline** (LAW).
> **It paid for itself immediately:** across §6 and §7 the two agents caught **16 findings in test material I would have sent after one read-through** — including a step whose opening screen was inverted for his actual settings, a "not a failure" clause that would have told him to ignore the exact regression under test, an instruction to watch for an error that cannot occur, and an expected result that would have made him report a false failure.
>
> ### 🆕 FILED
> - **PJ-213** — **surface a link's age.** Boss-ruled 2026-08-03: *"I want the link's age to be surfaced."* `created` is one of the eight Living-Link properties and the basis of decay, is **not** in the earned ledger, and until §1 shipped was being silently reset. Both the Outgoing **and** Backlinks panels (Whole-Ecosystem), a date format, i18n ×15, RTL.
> - **PJ-214** — **a search term filtered to nothing returns zero results with no explanation.** Boss hit it live: `run7` → tokenises to `run` → an English stopword → unfindable by construction. Same silence class as PJ-207.
> - **PJ-215** — **`statusBar.indexing` ("Indexing") is translated into all 15 locales and rendered nowhere.** The entire `statusBar.` namespace has zero code references. The app does multi-second work and says nothing.
> - **PJ-216** — **`TagsPanel.svelte` is dead code.** The component with tag counts and an A→Z / by-count sort is imported by no file. (Counts live on the Dashboard.)
> - **PJ-217** — **same-second mtime residual.** §3 narrowed the save-during-walk window to sub-second; `modified` has second resolution, so a save in the same second still wins. Closing it needs content hashing on the walk path — a write-path change, its own job.
> - **PJ-218** — **Charter W2-14**: the save-path incoming diff keys on names only, so re-typing a link never recomputes the target's aggregates. The repair now heals it; the write-path fix is its own job.
>
> ### 📌 STILL OPEN, unchanged
> The 2026-07-30 inspection lost **25 candidates to server errors** — never triaged, still owed. The 2026-08-03 whole-app sweep's **40 confirmed findings** (`lab/reports/inspection-2026-08-03-pj207-s1.md`) need merging with the 31-item register; **it escalated triage item #11 to APP-KILLER** (`loadWorkspaces` refuses a successful EMPTY read, so universe A's layouts stay live in universe B and the first save overwrites B's file — collections, settings and property-types all got that reset; workspaces is the sibling that never did).
> **PJ-166 — ELEVENTH strike.** The safety inspection was invoked diff-scoped with `args.files` and returned `mode: "whole-app"` again.
>
> **Gates at close:** Rust **1355/0** · vitest **900/900** (76 files) · svelte-check **0** · i18n **15/15 ✓**.
>
> ---

**Version 1.66 | 2026-08-03**

> *(See `Constellation Pending Jobs v1.66.md` — the trail is durable, never overwritten.)*
