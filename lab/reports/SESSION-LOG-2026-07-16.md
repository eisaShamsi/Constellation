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

## PJ-106 §B4 — the paragraph direction switch (Boss directed: proceed ahead of Jul 18)

**Working on:** Right-Ctrl+Shift → caret's paragraph 100% RTL, Left-Ctrl+Shift → 100% LTR,
persisted as an invisible RLM/LRM at each content line's content-start (the Boss-approved
Round-3 design). New `src/lib/editor/paragraphDir.ts` (pure change-computer + arm/disarm/fire
gesture); `bidiPlugin.ts` mark-precedence + shared `BLOCK_PREFIX_RE` + content-based same-frame
rebuild; wired into NotePane/FocusPane/ConflictMergeView behind `RTL_MOTION_ENABLED`.

**The adversarial review (`wf_34d75a00`, 24 agents, 4 refute-first lenses + per-finding
skeptics — standing in for the rate-limited automated inspection): 16 confirmed findings.**
Headline: **[APP-KILLER]** `domEventHandlers` never sees keymap-consumed chords, so releasing
Ctrl+Shift+S (§B3!) would have force-flipped the paragraph → the gesture now uses
`domEventObservers` (always run). **[HIGH]** the merge view edits the FULL file → YAML keys
would get marked (`‏title:` ≠ `title:`, silent metadata/typed-link loss) → frontmatter-aware
skip + caret-in-frontmatter no-ops entirely; **[HIGH]** a mark before a line-leading `#tag`
kills the tag in index/tasks/Obsidian → tag-leading lines skipped. Plus: CommonMark-aware
fence parity (opener char+length matched; quoted/listed fences; indented-code lines never
marked); `![[note#heading]]` fragment identity normalized against marks (livePreview.ts +
store.ts extractHeadings + libraries.rs get_note_headings); checked-task lines (`- [x]`) read
past the `x` in detectLineDir (structured prefix strip); undo/redo/paste/adopt get the
same-frame rebuild via content-based mark detection (replaces the effect-only trigger);
caret maps AFTER the inserted mark (assoc 1); `isolateHistory` keeps the flip one undo step;
AltGr chords belted; digit-only lines are content (no half-flipped blocks); link-ref/footnote
definition lines skipped. **15 of 16 fixed in code**; the 16th — the Windows Ctrl+Shift
layout-hotkey collision — is Word's own shipped convention and is a Boss live observation +
ruling (folded into the staged test). Documented limitation (LOW, accepted): a line starting
with `_emphasis_` renders literal underscores in some EXTERNAL renderers when marked.

Gates: 30/30 §B4 recipes (incl. one contract catch — caret on a YAML line now targets
nothing), vitest 425, svelte-check 0, fresh release binary 14:49.

**Boss live-test — PASS (all stages).** Steps 1–5: English→RTL flip, LTR return + one-press
undo, the two-line Arabic paragraph forced LTR and back, Ctrl+Shift+S/L still select without
flipping (the app-killer check), persistence across note-switch AND full app restart. Step 6
(the OS-collision ruling): the Boss switches language with **Win+Space** — the gesture doesn't
switch his language, and the 30-second disambiguation confirmed an already-written Arabic
paragraph **stays put** on a language switch (the earlier observation was Part-A's correct
auto-direction on newly-typed text, not a mark write). **Ruling: no collision → §B4 ships
as-is.** Committed `<this>`. **PJ-106 Part B is COMPLETE** (§B1/§B2/§B3/§B4 all Boss-validated).
Remaining in PJ-106: §A4 (Reproduce-First-gated on the callout repro), C1 (CM6 bump, optional),
C2 (docs/help/User Manual ×15), C3 (Phase-4 audit + the per-cycle sweep — the automated
inspection resets Jul 18 and doubles as the §B4 post-gate confirmation).

## PJ-106 CLOSE — C2 + C3(audit) + §A4 gate + the callout fixes

**C2 (docs ×15) — DONE.** NEW help topic "Writing in Arabic and Mixed Scripts" in all 15 locales
(English master hand-written; 14 native translations via `wf_e588176b`, every agent found its
manual's RTL section at its own drifted number — §19/§20 — and extended it in place; folder/file
names stay English per repo convention). English User Manual §18 extended (select-by-unit +
forcing direction). `LESSONS-LEARNED.md` gained **LL-034** (bidi has TWO engines — render fixes
without the motion facet ship half the recipe; sweep plain-text marks against every text parser).

**C3 (Phase-4 audit, `wf_97aab837`, 3 lenses) — verdicts:** invariants **PASS-WITH-NOTES** (all
6 INVs HOLD); migration-path **PASS-WITH-NOTES** (all 5 items PASS; flag-off contract now
pinned in rtlFlag.ts + the plan CLOSE NOTES); drift **FAIL → FIXED same-pass** (WA#6): a §B4
mark before `[!` would sever a callout (all 7 callout parsers mark-blind) → callout HEADERS are
now in §B4's skip list; and — converging with the Boss's independent split-box report — a
callout header now takes its DIRECTION from its visible TITLE, not the hidden `[!note]` keyword
(`detectLineDir` strips the type token; an Arabic-titled callout renders as one coherent RTL
box). Audit paper-trail also fixed: FocusPane:192 stale comment, detectDir mark-blind-by-design
comment, rtlFlag.ts exact-scope comment, plan CLOSE NOTES §1–6. The remaining audit notes fold
into the **Jul-18 sweep** (window-level mousedown disarm edge for a toolbar Ctrl+Shift+click)
and the ledger (**PJ-109** — A5's optional Mod-Arrow Windows word-hop, never landed; polish).

**§A4 — CLOSED, SUBSUMED BY PART A.** The Reproduce-First gate ran on the Boss's live app:
End/Home + arrows inside an Arabic callout (pure + bilingual lines) — **"callout caret pass."**
No repro fired → per the standing rule, §A4 is not built.

**Gates at close:** vitest **427** (33 files; +2 callout recipes), svelte-check **0**, fresh
release binary. **PJ-106 is CLOSED** pending only the Jul-18 per-cycle sweep (which doubles as
the §B4 post-gate). INV-1 note: the Boss's live typing validation at every increment stands in
for the two recorded burst numbers (flagged to the Boss at close for the explicit ruling).

## Housekeeping

- Test notes `PJ108 Target.md` / `PJ108 Linker.md` remain in `Eisa Test` pending tab-close →
  to be moved to the session scratchpad once the Boss closes their tabs.
- The lock script + sentinel live in the session scratchpad (`pj108-lock.ps1`); the lock process
  exits on release — nothing left running.
- Pending for the session-close PCS: User Manual + help topic (×15 locales) for the new selection
  commands (B1/B2/B3) — the PJ-106 C2 step, landing with the migration close or the PCS, whichever
  comes first.
