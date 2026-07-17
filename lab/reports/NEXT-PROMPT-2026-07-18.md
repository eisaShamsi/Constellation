# Ready-to-paste next-session prompt (rewritten 2026-07-17 at the PJ-103 close)

---

Read `docs/Constellation Orientation & Onboarding v3.55.md` first (highest version — the PJ-103
close is in its preamble). Then read the handover
`lab/reports/HANDOVER-2026-07-17-pj103-close.md`. Then `git pull origin main` and skim
`git log --oneline -12`.

State: last session CLOSED PJ-103 (the app-close data-loss APP-KILLER) — Boss-validated live: the
close handshake now flushes every dirty note durably to disk before the window dies (5s cap,
instant when clean), with per-id save serialization, a re-pass, journal markers, and awaited FTS
reindex. The arc REFUTED the filed switch-drop mechanism (PJ-086 needs re-examination) and proved
the localStorage recovery net NON-DURABLE (leveldb orphan-wipe, evidence banked) → PJ-110 filed as
the new Group-1 top. Ledger v1.34; orientation v3.55; Charter updated (incl. the Jul-14 drift fix);
manuals ×15 updated.

► NEXT ACTION — the Jul-18 4:00 am whole-app sweep should have fired AUTOMATICALLY (scheduled task
`pj106-cycle-close-sweep-jul18`, register-only). Check `lab/reports/` for
`SWEEP-REGISTER-2026-07-18-*.md`: if present, triage it — fix every confirmed finding before
declaring the PJ-106 cycle closed (WA#6), APP-KILLERs first, each fix Reproduce-First + Boss-tested.
If absent (the Claude app was closed at 4am), run `Workflow({ name: 'safety-inspection' })` now.
The sweep also post-gates PJ-106 §B4 and checks the toolbar-Ctrl+Shift-click disarm edge.

Then PJ-110 (localStorage net durability — full /migration: Rust-side atomic_write persistence for
the write-ahead net; the hard-kill recovery class PJ-102/108 still rides on it). Then the Group-1
queue per ledger v1.34.

Don't lose: PJ103 A/B test notes in Eisa Test (move once tabs closed) · PJ-104 fresh evidence
(wrong-universe boot ×2, 2026-07-16) · PJ-086 re-examine-first flag · PJ-107 parked · Group 3
SS-Cockpit paused (resume Plan §6).
