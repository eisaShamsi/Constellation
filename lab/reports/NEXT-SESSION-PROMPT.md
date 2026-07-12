# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session after MIG-100 closed and the backlog was re-prioritized (Pending Jobs v1.18) + Standing Order #9 was added. Copy everything in the box.

---

Read `docs/Constellation Orientation & Onboarding v3.39.md` first (highest version — the outstanding-work sweep, the re-prioritized backlog Pending Jobs v1.18, and the new Standing Order #9 are in its "What changed in v3.39" preamble). Then read the handover `lab/reports/HANDOVER-2026-07-12-mig100-closed-pj-reprioritized.md`. Then `git pull origin main` and skim `git log --oneline -10`.

State: MIG-100 (Auto-Restore Tabs) is CLOSED + Boss-validated. The whole backlog was re-verified and re-prioritized into five groups — `docs/Constellation Pending Jobs v1.18.md` — with a standing "► Next action" line. A new Standing Order #9 makes that ledger the living backlog, reconciled at the close of every job (it is the FIRST file you open at any job-close; it pairs with SO#8's cross-check-before-tackling).

Boss ruling: **start Group 1, top-down. First item = PJ-070.**

PJ-070 — the watcher external-change APP-KILLER: an external `.md` edit on an OPEN note (git-pull / Syncthing / Obsidian) is silently clobbered by the user's next keystroke, because the watcher flush updates only `tab.content` and never adopts the change into the single-ownership note model (`+layout.svelte:3172`). It is the main-window mirror of the closed G3 cross-window class — the fix reuses the proven `SecondScreenPage.adoptFreshDiskIntoSS` pattern (adopt disk into the model + bump `reloadVersion` for the `{#key}` remount). This is a write-path / lifecycle change.

Do this:
1. Cross-check PJ-070 against the orientation body + recent session logs (SO#8) — confirm the `+layout.svelte:3172` site and the `adoptFreshDiskIntoSS` pattern still exist as described; it was filed 2026-07-12 so staleness risk is low.
2. Reproduce-First: instrument the write journal and reproduce the external-edit clobber on the running app BEFORE designing the fix (svelte-check/vitest are NOT runtime verification for editor-lifecycle bugs).
3. Run the `/migration` four-phase workflow: Architect (census the external-change adopt paths + the write model; design the main-window adopt reusing the G3 pattern; list invariants) → Boss picks the option → Plan → Build (each step: diff-scoped safety-inspection since it touches a write/lifecycle path, /simplify, svelte-check 0/0, npm run build before cargo build --release, verify binary mtime) → Audit.
4. At close (SO#9): reconcile the PJ ledger FIRST — close PJ-070 with evidence, file anything it surfaced, re-rank the "► Next action", bump to Pending Jobs v1.19, in the same commit as the work.

Standing reminders: Reproduce-First is a top principal (no editor-lifecycle fix ships before the bug is reproduced on demand). Test tutorials are staged, one stage at a time, tutorial-style. The Art Director & Team own UX/UI (not relevant here — PJ-070 is backend/lifecycle). Open investigation carried forward: PJ-072 (universes booting from an unfound registry — a diagnostic build, when you get to it).
