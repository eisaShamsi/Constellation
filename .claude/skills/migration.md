# /migration — Major Change Workflow

Use this skill for any change that touches **schema, core data flow, cross-surface invariants, or multiple subsystems** — examples: adding a persistent derived view, swapping a read/write path, introducing a new trigger family, refactoring a shared store, migrating state across files.

Single-file tweaks and bug fixes do not need `/migration`. They use `/simplify`.

## The Four Phases

Every migration runs through four agents in sequence. Each phase gates the next. Do not skip.

### Phase 1 — Architect

Launch one Agent in parallel with two questions:
1. What is the current state of the territory being changed? (files, schemas, call sites, invariants)
2. What are the design options, their costs, their risks, their failure modes?

Agent deliverable: under 600 words. Option table with **speed estimate**, **effort estimate**, **risk class** per option. Explicit list of invariants that must not break. Explicit list of back-fill / migration / rollback concerns. No code.

After the agent returns, **present the options to the user and get a pick before moving on**. Do not guess.

### Phase 2 — Plan

After the user picks an option, launch one Agent to produce a phase-by-phase implementation plan:

- Ordered steps, each small enough to land as one commit
- Test surface per step (what to verify before moving on)
- Explicit risk-mitigation for each risk flagged in Phase 1
- Rollback plan: what happens if we ship this and discover drift in production

Agent deliverable: step list with file paths, function names, and line references where known. Each step has a **verification clause** ("after this step, X still works"). No code yet.

Present the plan to the user. Edit it based on feedback.

### Phase 3 — Build

Implement the plan step-by-step. For each step:

1. Write the code yourself (do not delegate).
2. Run the verification clause from Phase 2.
3. Commit with a `§N — <step name>` message tied to the plan.

You may use `Agent` calls during Build for focused exploration (e.g. "find all writers of `note_links`") but writing the change is your job, not an agent's.

After the final step, run `/simplify` on the full diff range.

### Phase 4 — Audit

Launch **three agents in parallel**, each receiving the full commit range and the original Phase 2 plan:

#### Agent 4A — Invariant Check
Walk the invariant list from Phase 1. For each invariant, report: still holds / regressed / cannot determine. Concrete evidence from the code, not opinions.

#### Agent 4B — Drift Check
For every trigger, hook, or write-path the migration introduced or touched, list every caller that bypasses the expected flow. This is the LL-023 failure mode — a new guard the system doesn't know about, silently stealing state.

#### Agent 4C — Migration Path
Run through the exact sequence a user on an existing universe will experience: first boot after update, schema version mismatch, mid-backfill interruption, rollback to previous version. Flag any step that can silently corrupt data or lose user work.

Aggregate findings. Fix each real issue before declaring done. Do not argue with findings; skip only clear false-positives and note why.

## The Principle Rule (for CLAUDE.md)

Major changes follow this workflow. The four phases are not ceremony — they are the verification protocol that keeps Constellation from shipping a regression that takes three sessions to undo. Cost of running the four phases: ~30 minutes of agent time. Cost of skipping them: the entire iteration that built the feature that broke.

## When NOT to use /migration

- Single-file refactors → `/simplify`
- Bug fixes with known root cause → just fix it
- Documentation updates → just update
- New feature that touches one component → just build it

When in doubt: does the change cross subsystem boundaries (Rust ↔ Svelte, schema ↔ code, write path ↔ read path)? If yes, `/migration`. If no, don't.

## Additional Focus

$ARGUMENTS
