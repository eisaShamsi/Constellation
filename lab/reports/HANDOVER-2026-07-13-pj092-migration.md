# Handover — 2026-07-13 (session close)

**Read `docs/Constellation Orientation & Onboarding v3.46.md` first** (highest version), then this file, then `git pull origin main` + `git log --oneline -12`.

## HEAD
`main` = **`f1f96ade`** — "PJ-092 redo — rename-cascade edit-loss/freeze FIXED via /migration". Working tree clean, `HEAD == origin/main`. Milestone tags this session: `milestone/pj-091-accept-merge`, `milestone/pj-092-flush-gate-exclude` (the buggy `pj-092-rename-cascade-dirty-guard` tag was deleted with its revert).

## What shipped this session (all committed + Boss-validated)
1. **PJ-070** — watcher external-change adopt (/migration) — `b1a3e388`/`cd5e53fd`.
2. **"Show copy"** reveal-in-explorer fix (spaced-path `raw_arg`).
3. **PJ-088** — conflict-resolution side-by-side Merge view — `bc6a1e43`+.
4. **PJ-071** — bulk Accept-All read-modify-write race → `gate_rmw` — `7daaf946`.
5. **PJ-091** — accept silently truncated manual multi-value frontmatter → **merge at every accept seam** (Boss ruling: never lose a manual value) — `fd6008bc`.
6. **PJ-092** — rename-cascade edit-loss/freeze APP-KILLER. **Arc: band-aid `0a605f02` (FROZE) → revert `cfdb75a3` → proper /migration `f1f96ade` (flush-gate-exclude).** Boss live-tested (A1/A2/B1/B2 + clean sanity) — all PASS.

**Both this-session APP-KILLERs (PJ-091, PJ-092) are closed.** The rename cascade is the strongest it's been.

## Two durable process changes (both Boss-mandated — do NOT regress)
1. **The Boss Test is MANDATORY on every build.** The commit is the LAST step, gated on the Boss's live-test pass. No "backend-only" / "proven-by-tests" exceptions. (Memory: `feedback_boss_test_every_build_mandatory`.) PJ-092's freeze reached `main` because `0a605f02` was committed without a Boss test — that must never recur.
2. **The Safety Inspection reviews the DESIGN, not just the code.** A `/migration` now runs a **design-stage safety inspection on the Plan** (adversarial, refute-first) BEFORE any code — it caught PJ-092's 5 hazards for free. Pairs with the per-build (diff) + per-cycle (whole-app) inspections. (Recorded in Pending Jobs v1.25 + Orientation v3.46 + the Charter.)

## Backlog — `docs/Constellation Pending Jobs v1.25.md` (SO#9 reconciled)
**► Next action: PJ-089** (Index-panel preview mounts a SECOND writable editor for an already-open note → last-writer-wins silent clobber, no `.conflict` sidecar; `+layout.svelte`). Group-1 top.
Then: PJ-090 (SS Tasks toggle no-broadcast) · PJ-093 (reindex-skip when db None) · PJ-086 (switchTab flush gap) · PJ-085/073 (frontmatter/YAML) · PJ-074 (durable rename + folder cascade) · PJ-083 · PJ-087/075/076 · PJ-077 · PJ-094/095/096/097 · PJ-072/002.

## Newly filed this session (don't lose these)
- **PJ-094** — `moveItem` no flush-before-repath (MED). **PJ-095** — `NoteEditor` `saving` single-flight drops a debounced save (MED). **PJ-096** — dirty-note `.conflict` sidecar write-failure swallowed (LOW).
- **PJ-097** *(NEW, from the PJ-092 audit)* — **FocusPane isn't covered by the `CascadeFreezeOverlay` during a rename cascade** (unlike NotePane), so a keystroke typed into a *rewritten* backlink's Focus view mid-cascade can be discarded by the `focusReseed` remount. PRE-EXISTING; contrived trigger; PJ-092's H3 reseed is an improvement over the prior silent stale-revert. Fix = add a FocusPane freeze-overlay + an Editor-Surface-Gate #4 harness assertion.

## Outstanding docs debt
- **Help translation backfill (PJ-014):** the new save-safety/external-edit/rename section was added to the **English** `docs/help.uConstellation.World/Notes Management/Notes Management.md` this session; the 14 other locales + the User Manual translations are the standing PJ-014 debt.
- **PJ-081** — orientation BODY refresh (§3/§4.x still lag the stacked preambles) + the §12 doc-drift batch.

## Open investigation carried forward
- **PJ-072** — the "Eisa Cognitive Knowledge" universe display-name resolves to on-disk root `E:\Cognitive Knowledge\` (confirmed via the write journal), but WHERE that display-name→root mapping is persisted is still unknown (not in the findable `universes.json`). A diagnostic build logging the resolved `app_data_dir` + registry path at boot is still wanted.

## Test/verify state
svelte-check 0 · vitest **338** · Rust: sources 32 / classifier 15 / write_gate 22 / cascade_walker 16 — all green. The `LOCKTEST` live-test harness was TEMPORARY and is removed from `main` (the automated tests are the permanent regression net).
