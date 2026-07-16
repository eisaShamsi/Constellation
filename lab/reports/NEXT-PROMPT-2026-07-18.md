# Ready-to-paste next-session prompt (written 2026-07-16 at session close)

---

Read `docs/Constellation Orientation & Onboarding v3.54.md` first (highest version — the PJ-106
close is in its preamble). Then read the handover
`lab/reports/HANDOVER-2026-07-16-pj106-close.md`. Then `git pull origin main` and skim
`git log --oneline -12`.

State: last session shipped and Boss-validated PJ-106 Part B COMPLETE (§B1 paragraph nav ·
§B2 select line/paragraph · §B3 select sentence · §B4 the Right/Left-Ctrl+Shift paragraph
direction switch via invisible RLM/LRM marks), fixed PJ-108 (the second-screen recovery-net
APP-KILLER, proven by a live crash-recovery test), and CLOSED the PJ-106 migration (C2 docs
×15 · Phase-4 audit 6/6 INVs + 5/5 migration-path, drift FAIL fixed same-pass · §A4 subsumed
via the live callout gate · the Boss's callout split-box fixed at the root). `main` clean at
`c6a8e2a8`.

Standing rules — do NOT regress:
1. The Boss Test is MANDATORY on every build — commit is the LAST step, gated on Eisa's live
   PASS on the running release binary. Tests are staged, one stage at a time, tutorial-style.
2. Reproduce-First for editor-lifecycle/content-integrity bugs — the running app is the
   verification. `npm run build` BEFORE `cargo build --release`; verify the binary mtime.
3. SO#8 cross-check any PJ before tackling it; SO#9 ledger reconcile at every job close;
   SO#6 orientation bump rides the feature commit.

► NEXT ACTION — the safety-inspection weekly limit reset (Jul 18, 4:00 am Asia/Dubai):
run the per-cycle WHOLE-APP sweep — `Workflow({ name: 'safety-inspection' })`. It is three
things at once: the PJ-106 cycle-close ritual, the promised §B4 post-gate, and the check on
the audit's open edge (the §B4 gesture's arm survives a mousedown OUTSIDE the editor — a
focus-preserving toolbar Ctrl+Shift+click could fire the flip on release; add a window-level
disarm if confirmed). Fix every confirmed finding before declaring the cycle closed (WA#6).

Then PJ-103 (APP-KILLER, Group 1): app close never flushes dirty BACKGROUND models — the
MIG-100 `session:final-flush` listener (`+layout.svelte:3436`) persists only session.json, so
a note edited then switched away from can lose up to ~30 s of typing at quit. Likely fix:
`flushAllDirtyTabs('final_flush')` inside the final-flush listener before the ack.
Reproduce-First on the running app, then the Group-1 queue per ledger v1.33.

Don't lose: PJ-107 (imported-note Home-caret invisible — PARKED, needs an instrumented dev
build) · PJ-109 (optional Mod-Arrow word-hop, LOW) · the two PJ108 test notes still in
`E:\Cognitive Knowledge\Eisa Test\` (move to scratchpad once Eisa closes their tabs — never
delete under open tabs) · Group 3: the PAUSED SS-Cockpit Parts B–F (resume Plan §6).
