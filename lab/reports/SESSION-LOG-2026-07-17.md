# Session Log — 2026-07-17

## ~05:11 — PJ-103 Boss test PASS + close (the session's arc lives in SESSION-LOG-2026-07-16.md, session 2)

The PJ-103 close-flush fix was Boss-validated live this morning: the MARKER-FOUR gesture (type +
instant ✕) landed on disk at the close instant; clean close instant; typing burst clean. Commit +
PCS followed. Full arc, evidence, and the review register: SESSION-LOG-2026-07-16.md (session 2)
+ the Charter's PJ-103 close-cycle register + ledger v1.34.

**► Standing next:** the Jul-18 4:00 am whole-app sweep — fires automatically (scheduled task
`pj106-cycle-close-sweep-jul18`, register-only). Then PJ-110 (Group-1 top).

## Session close (PCS)

Full PCS complete: the PJ-103 close commit `4b3f217c` (code + ledger v1.34 + orientation v3.55 +
Charter + manuals ×15 + evidence + MoCh + handover + next-prompt) pushed; this close commit pins
the hash into the handover. **PJ ledger reviewed at close — no change beyond v1.34** (nothing
surfaced after the reconcile; ► Next stands: the Jul-18 4am auto-sweep, then PJ-110). The PJ103 A/B
test fixtures stay in `Eisa Test` until the Boss confirms their tabs are closed. Next-session
prompt: `lab/reports/NEXT-PROMPT-2026-07-18.md`.

---

## Continuation (afternoon) — PJ-106 CYCLE CLOSE: the Jul-18 sweep + 3 app-killer fixes

**Boss ran the Jul-18 whole-app sweep a day early.** `wf_776dbce6-a50`, 82 agents, **62 confirmed**
(3 APP-KILLER · 11 HIGH · 38 MED · 10 LOW). Register `lab/reports/SWEEP-REGISTER-2026-07-18-wf_776dbce6-a50.md`.
The 4am scheduled task `pj106-cycle-close-sweep-jul18` disabled as superseded. §B4 post-gate: the
toolbar-Ctrl+Shift-click disarm edge CONFIRMED REACHABLE (independent code trace). PJ103 A/B
fixtures moved out of Eisa Test to scratchpad (Boss confirmed tabs closed).

**Boss ruling:** fix the app-killers most-dangerous-first. Each Reproduce-First + Boss-tested-live +
committed separately.

- **#2 (APP-KILLER) — `317b2512`.** `loadTabHistoryEntry` no B1 dedup + raw read → two models/one
  file (clobber of saved edits) + no net recovery. Fix mirrors `openNoteTab` (dedup-switch +
  resolveNoteContent + born-dirty). RED→GREEN `historyNavDedup.test.ts`; adversarial review "ships
  as-is." Boss: dedup-switch + Alt+Back-in-Focus PASS (Focus exits on nav — by design).
- **#1 (APP-KILLER, NEW) — `baae4533`.** PropertyEditor onDestroy flush used live tabId/filePath +
  stale editableProps → A's frontmatter onto B. Fix: mount-time identity snapshot (mirrors NotePane
  `mountedFilePath`). Boss: Two's props stay clean PASS.
- **#4 — `b6310479`.** §B4 gesture flipped on a Ctrl+Shift+toolbar-click. Fix: window capture
  mousedown+wheel disarm belt (ViewPlugin) + **ignore `e.repeat`** (root cause — held-modifier
  auto-repeat re-armed after the click-cancel; Boss's before/after repro exposed it). Boss: Test A
  (no flip) + Test B (real gesture still flips) PASS. *(Needed a 2nd build — the first belt-only
  version still flipped; the auto-repeat guard is the real fix.)*
- **#3 (was APP-KILLER) — PARKED, NOT reachable.** Focus mode hides the tree/tabs + auto-exits on
  nav → no rename can fire while in Focus (the sweep's premise was wrong; SO#8 + Reproduce-First
  caught it). Reframed as **PJ-114** (Focus-mode right-click menu); readOnly-during-cascade code
  parked uncommitted to ship with it.

**Build saga:** the release binary was locked by a HUNG ghost `constellation.exe` (PID 36172,
un-killable, held its own image file). Workaround: renamed the locked exe aside (Windows allows
in-use rename on same volume) → build wrote a fresh `constellation.exe`. The `.zombie-locked` file
clears on next reboot.

**Gates:** svelte-check 0 · vitest 429 · diff safety-inspection = zero findings in the edits (all 12
it re-confirmed are pre-existing backlog). **PJ-106 cycle CLOSED.** Ledger v1.35, Orientation v3.56.
**► Next:** PJ-114 (Focus-mode RC menu, Boss-directed), then PJ-110.
