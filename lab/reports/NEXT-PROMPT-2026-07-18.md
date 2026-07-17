# Ready-to-paste next-session prompt (rewritten 2026-07-17 at the PJ-106 cycle close)

---

Read `docs/Constellation Orientation & Onboarding v3.56.md` first (highest version — the PJ-106
cycle close is in its preamble). Then read the handover
`lab/reports/HANDOVER-2026-07-17-pj106-cycle-close.md`. Then `git pull origin main` and skim
`git log --oneline -12`.

State: last session CLOSED the PJ-106 cycle. The per-cycle whole-app safety sweep ran a day early
(`wf_776dbce6-a50`, 82 agents, 62 confirmed; register
`lab/reports/SWEEP-REGISTER-2026-07-18-wf_776dbce6-a50.md`) and its 3 reachable app-killers were
fixed + Boss-validated live + committed separately: **#2** back-button clone (`317b2512`,
loadTabHistoryEntry dedup + net recovery), **#1** properties bleed (`baae4533`, PropertyEditor
mount-time identity guard), **#4** §B4 toolbar flip (`b6310479`, window disarm belt + ignore OS key
auto-repeat). The **3rd app-killer (#3, FocusPane rename-cascade) was found NOT reachable** (Focus
mode hides the tree/tabs + auto-exits on nav) and REFRAMED into PJ-114; its readOnly-during-cascade
code is **parked UNCOMMITTED** in the working tree (`FocusPane.svelte`, `CascadeFreezeOverlay.svelte`,
`+layout.svelte`) to ship WITH PJ-114 — keep those edits. Ledger v1.35, orientation v3.56, Charter,
session log, MoCh all pushed. A NEW standing rule landed: **Cross-Platform by Design** (consider
macOS in every coding/build decision — CLAUDE.md Architecture Principles).

► NEXT ACTION — **PJ-114: the Focus-mode right-click menu.** Boss-directed: design the complete
right-click context list for Focus mode — a right-click on a `[[link]]` → Rename (and the other
essential actions) WITHOUT leaving Focus. This is the affordance that makes the parked sweep-#3
protection reachable + testable. Approach: **concept-first** (State the concept — it touches the
*Focus = minimal / plain-text / no-decorations* principle, the Editor Parity exception; detecting a
wikilink under the cursor is a line-regex, not a full parser, so it can stay parser-free). Then the
**Art Director & Team** design pass (multi-agent workflow — they own UX/UI), reusing the **banked
Obsidian right-click menus** as the target (Note/Folder/Link/editor-empty — see memory
`project_rightclick_obsidian_targets`). Build macOS-aware from the first line (Ctrl-vs-Cmd, the new
standing rule). Bring the concept + design options to the Boss to approve BEFORE any build. Folds in
**PJ-116** (FocusPane never wires `ontitlechange` → a title typed in Focus is silently discarded).

Then PJ-110 (recovery-net durability, /migration), then the Group-1 queue per ledger v1.35 (new
HIGHs filed this cycle: PJ-117 adopt stale-snapshot TOCTOU · PJ-118 ConflictMergeView
stale-generation · PJ-119 CE malformed-frontmatter classification loss · PJ-115 reloadTabsFromDisk
case/NFC skip · PJ-120→072 libraries.json registry-wipe).

Don't lose: parked #3 code (3 uncommitted files) · the `constellation.exe.zombie-locked` file in
`target/release/` clears on reboot (harmless) · Boss-test-every-build + Reproduce-First-on-running-app
+ Cross-Platform-by-Design are all in force.
