# Ready-to-paste prompt — new session for MIG-111 Phase 1.2

Copy everything inside the block below into a fresh Claude Code session.

---

```
Start MIG-111 Phase 1.2 — the routed context pool.

Before anything else, read in this order:
1. lab/reports/HANDOVER-2026-08-17-mig111-phase-1.2.md
2. docs/Constellation Orientation & Onboarding v3.98.md
3. docs/Constellation Pending Jobs v1.91.md  (the "► Next action" line names this job)
4. docs/LESSONS-LEARNED.md — LL-047 and LL-048 specifically
5. src-tauri/src/federation/vocab_harness.rs — the whole module doc
6. src-tauri/src/federation/owner.rs — the module doc

Then state the function in hand in one line before writing any code.

The job: an operation on a note that lives in a linked universe must do its bookkeeping in
THAT universe's database, using THAT universe's link vocabulary — not the active universe's.
Phase 0 and Phase 1.1 are closed; owner resolution already works (federation/owner.rs).

Definition of done — not a claim in a commit message:
  federation/vocab_harness.rs::routed_write_must_match_the_owners_vocabulary
  is currently #[ignore]d. Removing that attribute and having it pass IS Phase 1.2.
  The TODO inside it names the two assertions to write.

Two constraints the harness has already established — do not re-litigate them, read LL-047:

  1. You may NOT implement this by swapping the process-global link_types::REGISTRY.
     It is read at CALL time by all 26 of its call sites across 11 files, so the debounced
     save, a backfill tick or the watcher lands in the swap window and computes with the
     wrong vocabulary — with every row count still correct, so nothing surfaces it.
     A routed write must carry its vocabulary explicitly (threaded through the call, or
     bound to the connection). Those 26 call sites are the migration.

  2. Diff aggregate VALUES, never row counts. aggregates_for() exists for this.

Test through the form the app actually produces, not hand-built values. Phase 1.1 shipped
with nine green tests over an entry point that returned the inverted answer, because every
test drove the pure function with hand-built raw paths. That is the recurring failure of
this whole migration — proving a property over the sample you happened to look at.

Standing obligations that apply to every step:
  · Per-build diff-scoped inspection before every commit:
    Workflow({ name: 'safety-inspection', args: { files: [<changed files>] } })
    Every confirmed finding is fixed BEFORE the commit.
  · Any Boss test goes tutorial-auditor -> ui-inspector -> Boss. Never direct.
  · The Boss tests and passes every build before it is committed.
  · SO#9 — reconcile the Pending Jobs ledger at the close, in the same commit.

Start with the Architect step: map how a vocabulary reaches each of the 26 call sites today,
and give me the options for carrying it explicitly, with speed/effort/risk and the invariants
that must not break. Do not write implementation code until I approve the approach.
```
