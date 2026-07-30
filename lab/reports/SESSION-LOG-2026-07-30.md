# SESSION LOG — 2026-07-30

Continues 2026-07-29 (PJ-187 S-sweep, uncommitted, Boss Stage-1 in progress).

## §1 — Pre-commit safety inspection: an APP-KILLER in the sweep's own fix, plus three more

Run `wf_860b6505-c41` (85 agents, ~10.1M tokens): **34 confirmed**. Triaged against the diff:
**6 in the sweep's own changes — all fixed + RED-proven before commit**; 27 pre-existing → the
triage feed `lab/reports/pj187-inspection-2026-07-30-remaining.md` (1 APP-KILLER, 10 HIGH,
13 MED, 3 LOW). 25 verify agents were lost to server 500s — those candidates died UNVERIFIED,
so the register is an under-count, not an all-clear.

### §1.1 The APP-KILLER (store.ts:624) — LL-040's shape, fourth time

The sweep's cascade gate in `flushOutgoing` refused to write a dirty model mid-cascade —
correct — but returned `{ok:true}`, which the FlushResult contract (noteSession.ts:266)
defines as "safe to proceed with the nav/replace". Every departure site then destroyed the
model: nav re-seeded it, closeTab/universe-sweep disposed it. The edit existed nowhere — not
on disk (refused), not in the net (NoteEditor's stash sites sit BELOW its own cascade gate),
no banner (nothing "failed"). Reachable by: rename a heavily-linked note, edit another note's
property in the right sidebar (not covered by the freeze overlay), close/switch mid-walk.

**Fix at the choke point:** stash the model's composed content into the write-ahead net
(unflagged — real work, so PJ-181's stale-snapshot check never discards it), THEN return
`{ok:false, reason:'cascading'}`. Nav sites abort (user stays on the note); close/sweep
proceed but the net now holds the content for reopen. Proof:
`tests/pj-187/cascadeDepartureLoss.test.ts` (4 cases incl. a no-cascade control), RED-proven:
under the broken version the edit vanishes from the net and the nav destroys the model.

### §1.2 Three more in-diff findings, fixed + proven

- **store.ts:1820** — `collectionsLoaded` was never reset, so a universe SWITCH whose read
  failed kept the latch TRUE and the store holding the PREVIOUS universe's collections: the
  next star wrote universe A's list over universe B's file. Fix: reset latch + list at the
  top of `loadCollections` (no subscriber auto-saves, verified). RED-proven.
- **store.ts:1833** — `saveCollections` snapshotted its payload at call time; two rapid
  toggles could land save1(old) → save2(new) → save1-RETRY(old), dropping the newer star.
  Fix: single-flight chain, payload read at WRITE time. Proven (blocked-first-write harness).
- **store.ts:3098** — `drainCidEnsure` writes the note's permanent cid through the gate
  (watcher-suppressed) and never reindexed — the exact gap the sweep closed in `openNoteTab`,
  missed in this sibling. The Whole-Ecosystem law violated INSIDE the sweep, second instance
  (first: move_to_trash). Fix: same fire-and-forget reindex.
- **store.ts:1737 / :2865** — both nav paths consume the INCOMING note's write-ahead entry
  (resolveNoteContent) BEFORE the flush-abort/supersede checks; an aborted nav destroyed the
  only copy of a failed save's recovery. Fix: re-stash the consumed entry (unflagged) on every
  abort after the read. RED-proven via the cascade-abort path.

### §1.3 Gates after the fixes

vitest **67 files / 717** (+ Sight **5/84** in the PJ-172 serial lane) · svelte-check **0** ·
Rust untouched since 1287/0. Binary rebuild running; Test 5 instructions held until the
chain (sources → build/ → exe) verifies per the standing rule.

### §1.4 PJ-166 — NINTH strike

Invoked diff-scoped with `args.files` (17 files); returned `mode: "whole-app"` again.
85 agents / ~36 min for what should be a 17-file gate. It earned its cost AGAIN (§1.1 is
real), but the per-build gate the standing order requires still does not exist.

## §2 — Stage 1 COMPLETE: Boss all-Pass; the sweep lands

Test 5, all seven steps: **Pass** (typing latency, save+tab round-trip, property round-trip,
linked rename incl. the mid-cascade typing recipe, move-with-unsaved-typing, Alt-nav, restart
restore). With Tests 1–4 that is the full Stage-1 matrix green. Per the Boss-test standing
order the commit gate is satisfied; landing as a thematic commit series (Rust safety · Rust
silent-failures · core store/layout · panels+i18n · docs/reports/ledger), then push.

Books written in this close: MoCh-2026-07-29-2100 · Orientation **v3.78** · PJ ledger **v1.61**
(files PJ-193 trash-browser UI; supersedes the "Consolidate trash" button into the migration
back-fill; records PJ-166's ninth strike). `tests/pj-187/zz-out.txt` (debug scratch) removed.

## §3 — MIG-108 Phase 1 (Architect) — "One Universe, One Location"

Boss: "Go." Territory mapped by a 7-slice workflow + completeness critic (run
`wf_1e0af182-c53`, 8/8 agents, ~1.24M tokens): sqlite (16 path-bearing tables, trigger
gating, FK hazards), json-state (17 stores ranked), frontend-state (localStorage/boot/
watcher/SS), move-machinery (gate_rename reuse; NAME-based links → zero body rewrites,
proven), import-linking (add_library the choke point; cUniverse out of scope by
construction), trash-settings (collapse mechanics §D-F), prior-art (Lightroom re-link
shape; index-only-data horror stories = our search.db exposure). Critic findings: the
reconcile hard-abort cap invalidates it as a crash net (journal required); WAL-blind
snapshot hazard; cross-registry entanglement; basename collisions; two walkers without
nested-library exclusion (canonical.rs, embeds.rs); SS outside every freeze channel;
earned.jsonl path-fallback claim REFUTED from code (cid/name only).

**Architect doc:** `docs/MIG-108-One-Universe-One-Location-Architect.md` — concept,
measured scope, 12 hazards → mitigations, run design (P/S/F/M/R/T/V/W), standing
constraints, 10 audit invariants, rehearsal protocol (full-copy dry run first), 4 Boss
decision points (layout D1, copy-vs-move D2, PJ-065-test-book D3, timing D4). Maps
archived in scratchpad; key excerpts inline in the doc.

Awaiting Boss rulings on D1–D4 → Phase 2 (Plan).
