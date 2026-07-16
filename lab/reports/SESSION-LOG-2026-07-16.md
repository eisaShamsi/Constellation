# Session Log — 2026-07-16

Continues `SESSION-LOG-2026-07-15.md` (same working session: PJ-106 Part-B selection half B1/B2/B3
shipped + Boss-validated; PJ-106 §B4 deferred to Jul 18 by Boss ruling; PJ-108 fix built + gated,
pending Boss test — this log records the live validation and the close).

---

## PJ-108 — the live crash-recovery Boss test (full arc) — PASS

Boss chose the **full crash-recovery test** over accepting the deterministic proof alone. Run on the
real release binary against the `Eisa Test` library with a REAL write-blocking lock (FileShare.Read
held by a background pwsh process — the PJ-102 method) on `PJ108 Target.md`.

**What driving the test taught us (recorded honestly):**
- The everyday SS view — the **Knowledge Cockpit** (lens graph) — was never the danger: its node
  clicks delegate to the main window (`sendNoteToMain`). The Boss's screenshot confirmed only the
  node is clickable there.
- The save-health banner **blocks navigation away from an unflushable note** (PJ-102/PJ-091 arc
  behavior, correct) — which also blocked the original Linker-note test path. Test rerouted.
- The ledger's third trigger path — **SS workspace auto-restore** (`SecondScreenPage.svelte:717`) —
  is **DEAD in the shipped app**: `sendWorkspaceRestore` (`secondScreen.ts:182`) has zero callers,
  and `+layout.svelte:8758` says "SS always starts closed — never auto-restore from workspace."
  The listener stays fixed regardless (the flag covers it if ever wired).
- The one LIVE-reachable vulnerable door: the **split-view Panels Companion → Tasks tab → task
  file-link** (`TasksPanel.svelte:87`), reached via main-window Split View + Ctrl+Shift+2.
  (The Boss's UI labels it "Panels Companion" — the `secondScreen.splitCompanion` locale string;
  "Split Comparison" is only the code fallback.)

**The run (all stages Boss-driven on the running app):**
1. Baseline note reset (`original line — do not lose me` + a `- [ ] PJ108 sample task` line);
   lock armed and verified (writes blocked, reads pass). One false start recorded: the first lock's
   60-min window expired before the Boss typed → his first edit SAVED (no banner, nothing lost);
   lock re-armed with a 240-min window.
2. Boss typed ` — EDIT THREE` in the main window → **red save-failure banner** (the net now held
   the only copy).
3. Boss clicked the task's **PJ108 Target** file-link in the SS Tasks tab — the exact
   `openNoteTab`-in-SS-context call that pre-fix ran `resolveNoteContent` → `clearWriteAhead`.
4. I force-killed `constellation.exe` (PID 44100) — disk verified at crash time: **no EDIT THREE
   anywhere on disk**. Lock released.
5. Boss relaunched, opened PJ108 Target → **screen showed `— EDIT THREE`** (recovered from the net
   that survived the SS open). Disk at that moment still held the baseline — expected: the recovered
   delta is unsaved work (PJ-102 born-dirty contract) and persists on the next natural trigger.
6. Boss typed a line (`Testing`) → durable save → **disk now holds `— EDIT THREE`**. Loop closed.

**Verdict: on the pre-fix build, step 5 would have shown the baseline (net destroyed at step 3).
The fix held through the exact app-killer scenario. PJ-108 CLOSED.**

## The commit (this one)

- **Code:** `store.ts` — `displayOnlyWindow` flag (`setDisplayOnlyWindow`) + `openNoteTab`'s
  trailing `preserveNet` param defaulting to it (`preserveNet ?? displayOnlyWindow` →
  `resolveNoteContent`); `SecondScreenPage.svelte` — `setDisplayOnlyWindow()` at script-init;
  `NoteEditor.svelte` — `handleLinkClick` passes `readOnly` as `preserveNet` (belt for main-window
  read-only mounts, e.g. the Index peek) + never `createNote`s from a read-only display.
- **Reproduce-First:** `tests/mig-076/readonlyLinkPreservesNet.test.ts` (Recipe RO, 5 tests:
  RO1 wound / RO2 preserveNet / RO3 inert / RO4 window-flag Solve-the-Class / RO5 precedence) —
  RO2 confirmed RED pre-fix. vitest 395, svelte-check 0.
- **Docs in-commit (SO#6/SO#9):** Pending Jobs **v1.31** (PJ-108 closed; PJ-106 Part-B selection
  half recorded; ► Next re-pointed), Orientation **v3.52**, MoCh ×2, this log.

## Housekeeping

- Test notes `PJ108 Target.md` / `PJ108 Linker.md` remain in `Eisa Test` pending tab-close →
  to be moved to the session scratchpad once the Boss closes their tabs.
- The lock script + sentinel live in the session scratchpad (`pj108-lock.ps1`); the lock process
  exits on release — nothing left running.
- Pending for the session-close PCS: User Manual + help topic (×15 locales) for the new selection
  commands (B1/B2/B3) — the PJ-106 C2 step, landing with the migration close or the PCS, whichever
  comes first.
