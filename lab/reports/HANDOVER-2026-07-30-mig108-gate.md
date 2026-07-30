# HANDOVER — 2026-07-30 session close: MIG-108 at the Stage-B gate

## Where we stand (verified, not remembered)

**MIG-108 "One Universe, One Location" is built, Boss-validated at Stage-A, and rehearsed
three times — everything is done except the two items hard-gated on the Aug-1 weekly-limit
reset.** Branch `main` @ `91dd0509`, all pushed. Gates at close: Rust **1305/0** ·
vitest **67/715** (+ Sight 84 in the PJ-172 serial lane) · svelte-check **0**.

- Slices 0–6 + Stage-A's three caught defects (sky-trigger timing → 455 s measured;
  copy-class rows; stale registry cache) all landed, RED-proven where a harness exists.
- Slice 5 made the law standing: `add_library` rejects external paths; `bring_in_library`
  Copy/Move per use; importer own-only; "Bring In a Library" ×15; tagline de-vaulted.
- Slice 8's docs half: CLAUDE.md invariant REPEALED+amended, manuals ×15, orientation
  v3.79, ledger v1.62, MoCh ×2.
- PJ-192 CLOSED. MIG-105 priority RAISED (Boss's own Stage-A connection).
- The rehearsal world at `E:\Constellation Universes\MIG108 Rehearsal` (+ ` External`) is
  POST-migration (Done). Rebuild fresh with `lab/tools/mig108_make_rehearsal.py` (refuses
  if dirs exist; clean via the long-path-safe scratchpad script pattern).

## THE GATE (do these IN ORDER, next session, after Aug 1 04:00 Dubai)

1. **Complete safety inspection** — `Workflow({ name: 'safety-inspection' })` whole-app.
   The 2026-07-30 run covered only **3 of 14 scopes** (11 agents died on the weekly limit);
   a truncated inspection reads exactly like a clean one — that is WHY the gate exists.
   Fix every confirmed finding before proceeding (WA#6).
2. **MIG-108 Phase-4 audit** — three agents per the Migration Rule: invariants I1–I10
   against shipped code · drift (new guards the system doesn't know) · migration path
   (first-boot, foreign-skip, mid-journal interrupt, VerifyFailed resume, rollback).
3. **Slice 7 — Stage-B**: rebuild the binary if anything changed, then the Boss opens the
   REAL universe → proposal (18 externals; he flips PJ-065-test-book to **Copy in**) →
   Unify → the §7-invariant validation walk (tutorial per the Testing Instructions Rule).
   The snapshot stays until the Boss declares pass. THEN close MIG-108 (ledger/orientation)
   and proceed to **MIG-104 Slice 8 + 8b** (archive hook BEFORE the `DELETE FROM note_meta`
   at search.rs:9845 — FK CASCADE fires there; `tests_stage0_delete_order_defect`).

## Standing cautions
- The transient `LNK1104` cargo-link failure: always retry once before diagnosing.
- Bash heredocs eat backslashes in this environment — write scripts via the Write tool.
- The post-Unify reload is WEBVIEW-ONLY; anything cached process-lifetime must be
  invalidated explicitly (the class behind Stage-A finding #3).
- PJ-187's remaining register (19 M-cost + the 27-item feed) still owes a Boss triage.

## Ready-to-paste next-session prompt

> Read `docs/Constellation Orientation & Onboarding v3.79.md`, `docs/Constellation Pending
> Jobs v1.62.md`, and `lab/reports/HANDOVER-2026-07-30-mig108-gate.md`. The next job is the
> MIG-108 Stage-B gate, in order: (1) the COMPLETE whole-app safety-inspection workflow —
> the 2026-07-30 run covered only 3 of 14 scopes before the weekly limit; fix every
> confirmed finding; (2) the MIG-108 Phase-4 three-agent audit; (3) Slice 7 / Stage-B — the
> live unification of `E:\Constellation Universes\Eisa Cognitive Knowledge` (18 externals,
> PJ-065-test-book flips to Copy in), with the validation walk as a tutorial. Nothing
> touches the real universe before (1) and (2) are green. Then MIG-104 Slice 8 + 8b.
