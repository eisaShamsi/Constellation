# Handover — 2026-07-17 (PJ-106 cycle close + 3 app-killer fixes)

**Read `docs/Constellation Orientation & Onboarding v3.56.md` first** (highest version), then this
file, then `git pull origin main` + `git log --oneline -12`.

## HEAD
`main` — synced with `origin/main` at the cross-platform-docs commit (the last commit of this
session). The 3 app-killer fixes + the cycle-close bundle are all pushed. **Working tree carries
the parked #3 code** (3 files, intentionally uncommitted — see below) + the runtime
`.claude/scheduled_tasks.lock` deletion (leave it).

## What shipped (all Boss-validated live, each its own commit)
**PJ-106 CYCLE CLOSED.** The per-cycle whole-app sweep (`wf_776dbce6-a50`, 82 agents, 62 confirmed)
ran a day early (Boss direction) = the migration's C3 boundary; the §B4 post-gate edge was fixed.
- **#2 `317b2512`** — Alt+←/→ history-nav cloned an open note (two NoteModels for one file → silent
  clobber of saved edits + net-recovery bypass). `loadTabHistoryEntry` now mirrors `openNoteTab`'s
  B1 dedup + `resolveNoteContent`. RED→GREEN `tests/mig-076/historyNavDedup.test.ts`.
- **#1 `baae4533`** — PropertyEditor onDestroy flush wrote the OUTGOING note's frontmatter onto the
  INCOMING note (BUG-023 via the props channel). Fixed with a mount-time identity snapshot (mirrors
  NotePane `mountedFilePath`).
- **#4 `b6310479`** — §B4 gesture flipped on a Ctrl+Shift+toolbar-click. Fix = window-level
  capture-phase mousedown+wheel disarm belt (ViewPlugin) + **ignore OS key auto-repeat** (`e.repeat`
  — the root cause the Boss's before/after repro exposed).
- **Bundle `8bf9ae60`** — register + ledger v1.35 + orientation v3.56 + Charter + session log + MoCh.
- **Cross-platform docs** (this session's last commit) — the new standing rule in CLAUDE.md + v3.56.

## PARKED — do NOT lose (ships with PJ-114)
**Sweep #3 (FocusPane readOnly-during-cascade) is parked UNCOMMITTED** in 3 working-tree files:
`FocusPane.svelte`, `CascadeFreezeOverlay.svelte`, `src/routes/+layout.svelte`. It was found NOT
user-reachable (Focus mode hides the tree/tabs + auto-exits on nav → no rename can fire while in
Focus; the sweep's premise was wrong — SO#8 + Reproduce-First caught it). It becomes reachable only
once PJ-114 ships the link→Rename right-click action. **Keep these edits; they ship with PJ-114.**

## ► NEXT ACTION — PJ-114: the Focus-mode right-click menu
Boss-directed (2026-07-17): design the **complete right-click context list for Focus mode** (a
right-click on a `[[link]]` → Rename etc., without leaving Focus). It makes the parked #3 protection
reachable + testable. **Concept-first** (it touches the *Focus = minimal / plain-text / no
decorations* principle — Editor Parity exception); **Art Director & Team** design pass; reuse the
**banked Obsidian RC targets** (Note/Folder/Link/editor-empty — see `project_rightclick_obsidian_targets`).
Folds in **PJ-116** (FocusPane never wires `ontitlechange`). **Then PJ-110** (recovery-net durability).

## Standing rules — do NOT regress
- **Boss tests EVERY build before commit** (mandatory) · **Reproduce-First on the running app** ·
  **NEW: Cross-Platform by Design** — consider macOS in every coding/build decision (CLAUDE.md
  Architecture Principles; the §B4/PJ-106 Ctrl gestures need a macOS ⌘Cmd keymap pass later) ·
  SO#6/8/9 · Art Director & Team own UX/UI · `npm run build` BEFORE `cargo build --release`.

## Environment
Release binary `src-tauri\target\release\constellation.exe` built 2026-07-17 **15:42** contains all
4 fixes (incl. the parked #3). A hung ghost `constellation.exe` (PID 36172) left a
`constellation.exe.zombie-locked` file in `target/release/` — harmless, clears on Eisa's next
reboot. Boss's active universe root = `E:\Cognitive Knowledge`. One location:
`E:\مشاريع كلاود\Constellation`, branch `main`.
