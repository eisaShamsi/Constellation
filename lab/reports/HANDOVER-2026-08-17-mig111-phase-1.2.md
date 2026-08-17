# Handover — MIG-111 Phase 1.2 (the routed context pool)

**Written 2026-08-17.** Read this, then `docs/Constellation Orientation & Onboarding v3.98.md` and
`docs/Constellation Pending Jobs v1.91.md`.

---

## Function in hand

**MIG-111 Phase 1.2 — the routed context pool.** An operation on a note that lives in a *linked*
universe must do its bookkeeping in **that** universe's database, using **that** universe's link
vocabulary — not the active universe's.

## Where things stand

**Phase 0 is CLOSED** (0.1 live-WAL copy ban · 0.2 owner lock · 0.3 ledger cross-process lock · 0.4
the writers on the federation boundary).

**Phase 1.1 is CLOSED.** `src-tauri/src/federation/owner.rs` answers *which universe owns this
path?* — longest-match wins, unknown is an `Err` and never the active universe, roots from the
federation tree. 12 tests.

**Phase 1.2 has not started.** Its acceptance condition is already committed.

## Your definition of done, and it is not a claim in a commit message

`src-tauri/src/federation/vocab_harness.rs` contains:

```rust
#[test]
#[ignore = "MIG-111 Phase 1.2 — the routed context pool does not exist yet; this is its red→green"]
fn routed_write_must_match_the_owners_vocabulary() { … }
```

**Removing that `#[ignore]` and having it pass is what "Phase 1.2 is done" means.** The TODO inside
it names the two assertions to write. The harness already proves the hazard is real
(`two_vocabularies_disagree_on_the_same_note`) and is deterministic under a lock.

## The two things the harness already decided for you

**1. You may NOT implement this by swapping the process-global.**

The obvious design — *open the child's connection, `link_types::set_active` the child's vocabulary,
write, restore* — is **ruled out**, and not on taste. `link_types::REGISTRY` is a process-global read
at **call time** by all 26 of its call sites across 11 files. Anything landing in that window (the
1500 ms debounced save, a backfill tick, the file watcher's adopt path) computes with whichever
vocabulary happens to be installed, and stores the answer in the child's rows **with every row count
still correct**. Nothing surfaces it.

This is not theory — the harness's determinism test failed exactly this way on its first run, from a
sibling test. Pinned by `a_vocabulary_swap_reaches_back_into_an_already_open_database`, which asserts
the coupling *is present*: if you make the vocabulary a genuine property of the connection, that test
will fail loudly, and that is the signal to re-read LL-047 rather than to delete the test.

**A routed write must carry its vocabulary explicitly** — threaded through the call, or bound to the
connection. Expect this to be the bulk of the work: the 26 call sites are the migration.

**2. Diff aggregate VALUES, never row counts.** A vocabulary mismatch leaves every count identical
and changes what the rows *say* (`link_type` collapses to the null type, `incoming_link_types` loses
a member). `aggregates_for` exists for this.

## Read these first, in this order

1. `docs/LESSONS-LEARNED.md` — **LL-047** (shared mutable state makes "which vocabulary?" a question
   about *when*) and **LL-048** (a test that drives the pure function has not tested the caller).
2. `src-tauri/src/federation/vocab_harness.rs` — the whole module doc.
3. `src-tauri/src/federation/owner.rs` — the module doc, including what the inspection found in it
   *after* it was reported clean.

## Traps this migration has already fallen into — all four are the same trap

Every one is **proving a property over the sample I happened to look at**:

- §0.4 — a guard tested by its **source position** instead of its behaviour. It was a dead no-op on
  every input, green for a whole round.
- §0.4 — a test counting `gate_rmw` in **one file** while the file next door bypassed it.
- §0.4 — a test asserting the writers the **plan named**; there were seven, not five.
- §1.1 — nine tests driving the **pure function** with hand-built raw paths, green over an app entry
  point that returned the inverted answer for the default federation shape.

**For 1.2 specifically:** the routed path must be tested through the form the app actually produces.
If you test the pool with a hand-built child root, you have tested nothing — that is precisely how
1.1 shipped broken.

## Standing obligations

- Per-build diff-scoped inspection: `Workflow({ name: 'safety-inspection', args: { files: [...] } })`.
  Every confirmed finding is fixed **before** the commit. It has caught a defect in the diff that
  introduced it on essentially every step of this migration.
- Boss tests go `tutorial-auditor` → `ui-inspector` → Boss. Never direct. Default verdict is
  REJECTED.
- **The Boss tests and passes every build before commit.**
- SO#9: reconcile `docs/Constellation Pending Jobs vX.Y.md` at the close, in the same commit.

## Open items not in Phase 1.2's path

- **PJ-288** — a Boss ruling is owed.
- **PJ-300** — the federation cache can hold a degraded resolve for a session, silently turning every
  §0.4 guard into a no-op. Group 1, needs its own pass.
- **PJ-301** — `universe_lock::canon` can return two identities for one universe; not fixed because
  it also builds the lock path and verbatim is what bypasses MAX_PATH. Needs a measured decision.
- **PJ-284** — the freeze-and-leaks sweep scope is still unrun.
