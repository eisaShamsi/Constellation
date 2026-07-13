# Next-session kickoff prompt — paste this to resume

> Ready-to-paste prompt for the session after PJ-092 was fixed properly via /migration (both this-arc app-killers, PJ-091 + PJ-092, now closed). Copy everything in the box.

---

Read `docs/Constellation Orientation & Onboarding v3.46.md` first (highest version — PJ-092's /migration fix is in its "What changed in v3.46" preamble; PJ-091 + the PJ-092 revert are in v3.45/earlier below it). Then read the handover `lab/reports/HANDOVER-2026-07-13-pj092-migration.md`. Then `git pull origin main` and skim `git log --oneline -12`.

State: last session closed **both** APP-KILLERs of the arc — **PJ-091** (accept now MERGES, never truncates your manual sources/type) and **PJ-092** (rename cascade never loses a linked note's unsaved edits, never freezes — fixed *properly* via the full `/migration` after a band-aid froze the app and was reverted). Also shipped earlier this arc: PJ-070 (watcher adopt), the Show-copy fix, PJ-088 (conflict Merge view), PJ-071 (bulk-accept race). `main` = `f1f96ade`, clean.

**Two standing process rules are now in force — do NOT regress them:**
1. **The Boss Test is MANDATORY on every build.** The commit is the LAST step, gated on Eisa's live-test pass on the running app. No "backend-only" or "proven-by-tests" exceptions. (This is why PJ-092's freeze reached main — it was committed without a Boss test.)
2. **The Safety Inspection reviews the DESIGN (Architect/Plan), not just the code** — run a design-stage adversarial inspection on the Plan of any `/migration` BEFORE writing code.

Boss ruling holds: continue Group 1, top-down. **► Next action = PJ-089** — the Index-panel preview mounts a SECOND writable editor for an already-open note (not deduped against the open store tabs) → two writable models for one path → last-writer-wins silent clobber, no `.conflict` sidecar (`+layout.svelte`). It is a content-integrity / write-path issue. Read the current backlog: `docs/Constellation Pending Jobs v1.25.md`.

Do this for PJ-089:
- Cross-check PJ-089 against the orientation §4.x BODY + recent session logs (SO#8) — confirm the two-writable-model site still exists as described.
- **Reproduce-First on the RUNNING app** — instrument + reproduce the clobber (an Index-preview edit lost when the main tab saves, or vice versa) before designing the fix. For this content-integrity/editor-lifecycle class, the running app is the verification, NOT svelte-check/vitest alone.
- Scope check: does it cross subsystem boundaries (Rust↔Svelte, write-path↔read-path)? If yes → `/migration` (Architect → Boss picks approach → design-stage inspection on the Plan → Build → /simplify → Audit). If it's a focused single-surface dedup → a focused fix with the standing discipline.
- **Build with the mandatory gates:** diff-scoped safety-inspection, /simplify, svelte-check 0, `npm run build` before `cargo build --release`, verify the binary mtime — then **a staged, tutorial-style Boss live-test, and COMMIT ONLY AFTER Eisa passes it.**
- At close (SO#9): reconcile the PJ ledger FIRST — close PJ-089 with evidence, file anything surfaced, re-rank the ► Next action, bump to Pending Jobs v1.26, in the same commit as the work; then Orientation v-bump + session log + MoCh + handover + this prompt.

Standing reminders: Reproduce-First is a top principal. Tests are staged, one stage at a time, tutorial-style. The Art Director & Team own UX/UI. Open investigation carried forward: **PJ-072** (the "Eisa Cognitive Knowledge" display-name → `E:\Cognitive Knowledge\` root mapping — a diagnostic build is still wanted for WHERE that mapping is persisted). Newly-filed follow-ups to not lose: **PJ-094/095/096/097**.
