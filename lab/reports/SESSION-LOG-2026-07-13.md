# Session Log — 2026-07-13

## PJ-092 — the rename-cascade fix — REVERTED and RE-OPENED

**Function in hand:** the rename wikilink cascade (`handleRenameComplete` → `flushAllTabsInLibrary` → `updateLinksOnRename` → `reloadTabsFromDisk`).

### What happened
- PJ-092 shipped 2026-07-12 as commit `0a605f02` — a **dirty-guard** in `reloadTabsFromDisk` (skip reseeding a still-dirty note). **It was committed to `main` WITHOUT a Boss live test** ("deterministically proven by the store test"). That was the mistake.
- The Boss live-tested it: a **deterministic FREEZE** in the exact scenario the fix protects. A note whose flush fails during a rename is left dirty + disk-mismatched; the editor never remounts (the clobber path bumps `reloadVersion` and converges — the skip path does not), so the Svelte reactive layer hangs. A focused-fix investigation (`general-purpose` trace agent) refuted the watcher-loop / reindex-deadlock / double-reload and pinned it to the reactive re-entrancy.
- A follow-up **flush-outcome-gate** rework (mirror `renameItem`'s `renameFlushOk`: `flushAllTabsInLibrary` returns the not-durably-flushed paths → the cascade skips them; sibling callers gate on the flush outcome; revert the dirty-guard) tested as **still not acceptable** to the Boss.
- **Boss ruling: "FIX IT, don't patch it, or revert PJ-092." → REVERTED entirely.** Code restored to `fd6008bc` (pre-PJ-092): `src/lib/libraries/store.ts` (`reloadTabsFromDisk` + `flushAllTabsInLibrary`), `src/routes/+layout.svelte` (cascade), `src/lib/lens/store.ts`, `src/lib/editor/noteSession.ts`, `vitest.config.ts`. All PJ-092 code (dirty-guard, sibling gates, LOCKTEST/SHOWBUG live-test hooks) + tests removed. **svelte-check 0 · vitest 335** (pre-PJ-092 count). Boss live-tested the reverted build → **PASS** (normal rename works, no freeze, app responsive). THEN committed (the commit gated on the Boss pass — the new standing rule).

### Two durable lessons
1. **The Boss Test is MANDATORY on every build (new TOP STANDING ORDER, Boss-mandated 2026-07-13).** The commit is the LAST step, gated on the Boss's pass. No "backend-only" / "proven by tests" exceptions. The PJ-092 freeze reached `main` precisely because I committed `0a605f02` without a Boss test. (Memory: `feedback_boss_test_every_build_mandatory`.)
2. **PJ-092 should have gone through `/migration`, not a focused fix.** It touches the rename cascade + the editor reactive lifecycle across Rust↔Svelte — a textbook migration change. The band-aid → freeze → re-patch cycle is exactly what the Migration Rule + Solve-the-Class exist to prevent. **The freeze was invisible to the store-level vitest** (no real watcher/reindex/remount) — the "vitest is not runtime verification for editor-lifecycle bugs" gap.

### Status
- **PJ-092 (bug) — RE-OPENED, Group 1.** The rare rename-cascade edit-loss (a note open + dirty + `.md` locked at the instant of a rename). To be redone via the full `/migration` (Architect → Boss picks approach → Plan → Build → Audit), Reproduce-First on the RUNNING app, before any code.
- **Close (SO#9):** Pending Jobs **v1.24** (PJ-092 reverted → re-opened; ► Next action = PJ-089); Orientation **v3.45**; Charter revert-note appended. PJ-094/095/096 (from the PJ-092 sweeps) remain valid, independent of the revert.
- **Also this session (2026-07-12, carried):** PJ-091 (accept-merge) shipped `fd6008bc`; PJ-071 (bulk-accept RMW) shipped `7daaf946` — both remain (not touched by the revert).
