# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session after PJ-070 (Watcher External-Change Adopt) closed and the backlog was reconciled to Pending Jobs v1.19. Copy everything in the box.

---

Read docs/Constellation Orientation & Onboarding v3.40.md first (highest version — the PJ-070 close + the class fix + the PJ-072 lead are in its "What changed in v3.40" preamble). Then read the handover lab/reports/HANDOVER-2026-07-12-pj070-closed.md. Then git pull origin main and skim git log --oneline -10.

State: PJ-070 (Watcher External-Change Adopt) is CLOSED + Boss-validated (Stages 1+2). The backlog is reconciled — docs/Constellation Pending Jobs v1.19.md — with a standing "► Next action" line and 5 new PJs filed from the close (PJ-083..087). SO#9 makes that ledger the living backlog, reconciled FIRST at every job-close (it pairs with SO#8's cross-check-before-tackling).

Boss ruling holds: continue Group 1, top-down. Next item = PJ-071.

PJ-071 — the bulk Accept-All unlocked read-modify-write race: accept_one reads unlocked then gate_write (sources/bulk_ops.rs:305), the exact race the proven gate_rmw pattern prevents — per-card accept was migrated to it, bulk wasn't — so a concurrent editor save in the window is silently overwritten. It is a write-path change.

Do this:
- Cross-check PJ-071 against the orientation body + recent session logs (SO#8) — confirm the bulk_ops.rs:305 site + the gate_rmw call sites still exist as described.
- Reproduce-First: instrument + reproduce the RMW race (a concurrent editor save during a bulk accept losing the edit) before designing the fix.
- Scope check: this is likely a focused write-path fix (route bulk accept through the existing gate_rmw), NOT necessarily a full /migration — confirm against the gate_rmw call sites; escalate to /migration only if it crosses subsystem boundaries.
- Build with the standing discipline: diff-scoped safety-inspection (write path), /simplify, svelte-check 0/0, cargo build --release after npm run build, verify binary mtime.
- At close (SO#9): reconcile the PJ ledger FIRST — close PJ-071 with evidence, file anything surfaced, re-rank the "► Next action", bump to Pending Jobs v1.20, in the same commit as the work.

Standing reminders: Reproduce-First is a top principal (no write-path fix ships before the bug is reproduced on demand). Test tutorials are staged, one stage at a time, tutorial-style. Open investigation carried forward: PJ-072 (the "Eisa Cognitive Knowledge" universe root is now known to be E:\Cognitive Knowledge\; a diagnostic build is still wanted for WHERE the name→root mapping persists).
