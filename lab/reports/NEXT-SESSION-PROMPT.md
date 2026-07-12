# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session after PJ-070 closed, "Show copy" was fixed, and the PJ-088 merge view shipped (backlog reconciled to Pending Jobs v1.20). Copy everything in the box.

---

Read docs/Constellation Orientation & Onboarding v3.41.md first (highest version — PJ-088 the merge view is in its "What changed in v3.41" preamble; PJ-070 + the class fix + the PJ-072 lead are in v3.40 below it). Then read the handover lab/reports/HANDOVER-2026-07-12-pj088-merge-closed.md. Then git pull origin main and skim git log --oneline -12.

State: three deliverables closed this session — PJ-070 (watcher external-change adopt, /migration CLOSED + Boss-validated), the "Show copy" reveal fix, and PJ-088 (conflict-resolution side-by-side merge view, Boss-validated). The backlog is reconciled — docs/Constellation Pending Jobs v1.20.md — with a standing "► Next action" line; PJ-088 closed + PJ-089/PJ-090 (two HIGH silent-loss findings from the PJ-088 sweep) filed. SO#9 makes that ledger the living backlog, reconciled FIRST at every job-close (it pairs with SO#8's cross-check-before-tackling).

Boss ruling holds: continue Group 1, top-down. Next item = PJ-071.

PJ-071 — the bulk Accept-All unlocked read-modify-write race: accept_one reads unlocked then gate_write (sources/bulk_ops.rs:305), the exact race the proven gate_rmw pattern prevents — per-card accept was migrated to it, bulk wasn't — so a concurrent editor save in the window is silently overwritten. It is a write-path change.

Do this:
- Cross-check PJ-071 against the orientation body + recent session logs (SO#8) — confirm the bulk_ops.rs:305 site + the gate_rmw call sites still exist as described.
- Reproduce-First: instrument + reproduce the RMW race (a concurrent editor save during a bulk accept losing the edit) before designing the fix.
- Scope check: this is likely a focused write-path fix (route bulk accept through the existing gate_rmw), NOT necessarily a full /migration — confirm against the gate_rmw call sites; escalate to /migration only if it crosses subsystem boundaries.
- Build with the standing discipline: diff-scoped safety-inspection (write path), /simplify, svelte-check 0/0, cargo build --release after npm run build, verify binary mtime.
- At close (SO#9): reconcile the PJ ledger FIRST — close PJ-071 with evidence, file anything surfaced, re-rank the "► Next action", bump to Pending Jobs v1.20, in the same commit as the work.

Standing reminders: Reproduce-First is a top principal (no write-path fix ships before the bug is reproduced on demand). Test tutorials are staged, one stage at a time, tutorial-style. Open investigation carried forward: PJ-072 (the "Eisa Cognitive Knowledge" universe root is now known to be E:\Cognitive Knowledge\; a diagnostic build is still wanted for WHERE the name→root mapping persists).
