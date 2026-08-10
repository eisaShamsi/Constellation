# Safety-sweep and performance records

Structured output from the `safety-inspection` workflow and the performance investigations, kept
**in the repo** rather than in a session scratchpad.

## Why these files exist here

The 2026-08-09/10 orientation bump (v3.94) surfaced the gap that created this folder: the sweeps
had written ~100 confirmed findings to a session-local scratchpad **outside** the repository. Every
one of them was real, adversarially refuted, and anchored to a `file:line` — and all of it would
have vanished with the session, leaving the next reader a summary paragraph and no way to check it.

A finding that only a summary remembers is a finding nobody can act on. These are the raw records.

## The files

| File | What it is |
|---|---|
| `SWEEP-2026-08-09-second-whole-app.json` | 32 confirmed findings, whole-app sweep run against the code AFTER the first round of §15 fixes. All 32 were subsequently fixed. |
| `SWEEP-2026-08-10-third-whole-app.json` | 37 confirmed findings, whole-app sweep run after the second round. 8 fixed (2 app-killers + the 5 Boss-ruled held items + 3 self-inflicted); **~29 remain open** and are filed in the Pending Jobs ledger. |
| `PERF-2026-08-10-create-rename.json` | 29 measured findings from the create/rename performance investigation — the note create at 54 s and the rename at 50 s. Includes the negative results (suspects that were CLEARED by measurement), which are as useful as the positives. |

## Shape of each record

Each finding carries: `file`, `line`, `summary`, the failure `scenario`, the adversarial `verdict`
with its reasoning, and a severity that survived refutation. The performance records carry
`evidence` (what was RUN and what it returned), a `cost_estimate` with its derivation, and a
`confidence` of `measured` / `strong` / `weak`.

**Read the `evidence` field, not the `summary`.** The summaries are compressed; the evidence is what
was actually observed, and several of these findings were downgraded or refuted outright once their
evidence was checked.

## Before acting on one

These are snapshots of the code at the moment each sweep ran. Verify a finding is still true before
working it — several in the second sweep were fixed within hours of being filed, and the ledger
(`docs/Constellation Pending Jobs vX.Y.md`) is the live record of what is still open. Standing Order
#8 applies: cross-check against the current orientation body and the session logs first.
