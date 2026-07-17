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

## The close verification — Boss PASS (2026-07-16, post-722f3f97 binary)

The Boss relaunched and verified the callout coherence fix on his own split-box test case:
**"Pass"** — the Arabic-titled callout renders as one coherent RTL box (title + icon right,
accent bar consistent). No performance complaint raised across the whole arc's live typing →
the INV-1 ruling stands as recorded: live validation at every increment sufficed. PJ-106's
close verification is complete; the migration is DONE pending the Jul-18 sweep ritual.

## Session close (PCS)

Full PCS completed 2026-07-16: all commits pushed (`c0d668fc`…`c6a8e2a8` + this close commit);
help/User Manual ×15 shipped in C2; orientation bumps rode their feature commits (v3.52/v3.53/
v3.54); MoCh ×3 for the day's blocks (12:15 / 13:45 / 15:30); handover
`HANDOVER-2026-07-16-pj106-close.md`; next-session prompt `NEXT-PROMPT-2026-07-18.md` (carries
the Jul-18 sweep reminder). **PJ ledger reviewed at close — no change beyond v1.33** (the pass
record touched no backlog item; ► Next stands: the Jul-18 sweep, then PJ-103).

## Housekeeping

- Test notes `PJ108 Target.md` / `PJ108 Linker.md` remain in `Eisa Test` pending tab-close →
  to be moved to the session scratchpad once the Boss closes their tabs.
- The lock script + sentinel live in the session scratchpad (`pj108-lock.ps1`); the lock process
  exits on release — nothing left running.
- Pending for the session-close PCS: User Manual + help topic (×15 locales) for the new selection
  commands (B1/B2/B3) — the PJ-106 C2 step, landing with the migration close or the PCS, whichever
  comes first.

---

# Session 2 (evening) — PJ-103 opened (the app-close flush APP-KILLER)

**Function in hand:** PJ-103 — app close never flushes dirty note models; the `session:final-flush`
listener (`+layout.svelte:3444`) persists only session.json before acking Rust's 700 ms close hold.

## Session start
- `git pull` clean — `HEAD == origin/main` at `53e39675`. Orientation v3.54 + handover read.
- **Boss rulings ×3** (AskUserQuestion): (1) start PJ-103 NOW (§B4 precedent — stand-in adversarial
  review; the Jul-18 whole-app sweep re-covers the diff); (2) schedule the Jul-18 sweep;
  (3) PJ-108 fixture tabs are closed — move the files.

## Housekeeping (done)
- `PJ108 Target.md` + `PJ108 Linker.md` moved from `E:\Cognitive Knowledge\Eisa Test\` to the
  session scratchpad (`pj108-fixtures\`, 200+165 bytes intact — moved, never deleted).
- **Scheduled task `pj106-cycle-close-sweep-jul18`** created: fires ONCE at 2026-07-18T04:00+04:00,
  runs `Workflow({name:'safety-inspection'})` whole-app, writes the register to `lab/reports/`,
  commits/fixes NOTHING (Boss-test rule — fixes belong to the live session). Caveat: runs only
  while the Claude desktop app is open; else fires on next launch.

## PJ-103 — SO#8 cross-check + Understand phase (workflow `wf_ef19a089-dc0`, 6 readers, all facts cited to current code)
- **Confirmed live, not stale.** The listener at `+layout.svelte:3444` calls only `persistSessionNow()`
  (arrangement-only session.json, signature-guarded) then `session_flush_ack`. No note-model flush
  exists anywhere on the close path.
- **The Rust guillotine:** `lib.rs:661-692` — prevent_close → emit → `tokio::time::timeout(700ms, notified())`
  with the Result DISCARDED (ack and timeout indistinguishable) → destroy both windows. No RunEvent
  hook, no IPC drain; in-flight `write_note` is protected only by atomic_write's fsync+ReplaceFileW
  (old-or-new, never torn — but a cut-off save is silently lost).
- **The net does NOT cover the canonical scenario:** the write-ahead net (localStorage `constellation-wab`)
  is written inside the save gate (`noteSession.ts:132`) — per keystroke it is NOT touched
  (`NotePane.svelte:495-504`). A background-dirty model that never reached a save attempt has NO net
  entry → nothing to recover at next boot.
- **The mechanism (sweep register verbatim, re-verified):** type in A → switch to B inside the 1.5 s
  debounce → outgoing pane's teardown flush dropped by the staleness guard (`NoteEditor.svelte:305`,
  tab prop already points at B during the `{#key}` teardown) → model A dirty in RAM, no disk write,
  no net, no timer (all died with the pane) → quit loses everything since A's last durable save
  (the "~30 s" = NotePane's 30 s idle belt, inference — never derived in any doc).
- **The active tab is exposed too:** sub-1.5 s keystrokes rely on DOM `beforeunload` firing under
  `win.destroy()` — the codebase's own comment (`lib.rs:669-671`) says that is "NOT proven to survive
  webview teardown". Load-bearing unknown → part of the live reproduction.
- **The fix-shape tension nobody analyzed:** `flushAllDirtyTabs` (`store.ts:2319`) = sequential awaited
  durable writes (fsync + up to 5 lock retries each) vs the 700 ms window. A synchronous net-stash of
  every dirty model (compose + localStorage, zero awaits) is the only guaranteed-complete step.
- **Paper-trail drift found:** the 2026-07-14 sweep register was NEVER appended to the Charter although
  PJ-102–105 are marked "Open · Charter" in the ledger (fix at the ledger reconcile). The register file's
  own scenario lines are truncated mid-sentence. PJ-086 (switch-time flush) = the same class's other
  half; a close-time sweep covers its graceful-quit exposure.

## Reproduce-First — in progress
- Release binary verified current (built 16/07 17:36, post-PJ-106-close). App launched, PID 46652,
  path-verified `target\release\constellation.exe`.
- Victim/target notes created: `Eisa Test\PJ103 A.md` (106 bytes baseline, mtime 18:17:27, no marker)
  + `PJ103 B.md`.
- Computer-use request DENIED by Boss → reproduction handed to the Boss as a staged tutorial
  (WA#1's own carve-out: the running GUI is the Boss's domain). **Stage 1 delivered:** type
  `MARKER-ONE` in A → switch to B inside ~1 s → hands-off 15 s → I disk-check → Boss closes via X →
  disk-check + reopen-recovery check. Awaiting the Boss's "done waiting".

## PJ-103 Reproduce-First — COMPLETE (the loss fired live; root cause deeper than filed)

**The named recipe (fires on demand):** hover the mouse over the window ✕ → type into the active
note → click ✕ within the 1.5 s debounce → the typed tail NEVER reaches the .md file.
Boss-executed 2026-07-16 ~19:13: `MARKER-THREE` typed, app closed, disk verified marker-less
(`PJ103 A.md` last write 19:13:04 = the pre-typing Enter debounce; 172 bytes, no marker).

**Mechanism, proven step by step on the release binary:**
1. **Attempts 1+2 REFUTED the filed scenario** — a plain tab switch (paste+instant click; then
   type+pre-aimed instant click) PERSISTED the outgoing note both times (`MARKER-ONE`/`-TWO` on
   disk). The sweep register's "staleness guard drops the switch-away teardown flush" claim does
   not fire on the live app. (PJ-086 as filed inherits this doubt — flagged for re-examination.)
2. **The close cut-off CONFIRMED** — beforeunload DID fire under `win.destroy()` (settling the
   codebase's own "unproven" comment, lib.rs:669): its synchronous compose+setWriteAhead stashed
   the full marker content into localStorage at 19:13:46 (bytes recovered forensically from the
   WebView2 leveldb log — evidence preserved at scratchpad `pj103-leveldb-evidence\`). The ASYNC
   disk write it launched was cut off by process exit. Disk stale, net current.
3. **The recovery net FAILED cross-session — TOTAL SILENT LOSS.** On reopen, leveldb's own LOG:
   `Reusing MANIFEST-000001 / Recovering log #873 / Delete type=0 #3` — the manifest never
   registered the test session's log (its 18:56 open logged "Creating DB since it was missing"),
   so recovery replayed a STALE log and DELETED `000003.log` — the file holding the stash —
   as an orphan. `getWriteAhead` then found nothing; the MIG-100 restore (journal:
   `session_restore_begin 2 tabs → 2/2 restored` at 19:18:29) honestly served stale disk.
   Boss confirmed: the note ends at MARKER-TWO. **localStorage is NOT a durable medium** —
   its survival depends on Chromium leveldb internals (async browser-process commits,
   orphan-log deletion). For the LAST copy of user knowledge that is a File-Over-App violation.
4. Also observed live: the app relaunched into the WRONG universe (كون عيسى instead of Eisa
   Cognitive Knowledge) — PJ-104 territory, timestamped evidence ~19:16.

**The fix (Boss ruling: "up to 5 s, instant when clean"):**
- `+layout.svelte` final-flush listener: `await flushAllDirtyTabs('final_flush')` BEFORE
  `persistSessionNow()` before the ack — dirty models reach the .md files through the proven
  bounded durability gate. Fail-open per step (close never hangs on a failed write).
- `lib.rs` close arm: ack cap 700 ms → **5000 ms**; timeout expiry now writes a
  `final_flush_timeout_5s` journal marker (no silent guillotine — Charter class-1).
- Instant-when-clean: zero dirty models ⇒ the flush is a sync no-op scan ⇒ close unchanged.
- Keystrokes live in the model from the first character (per-keystroke editBody), so the
  sub-debounce tail is exactly what the close-flush persists.

**Gates:** svelte-check 0 errors · cargo check clean · vitest 427/427.
**In flight:** stand-in adversarial review (`wf_5bb5c713-220`, 4 refute lenses + verdict — the
§B4 precedent, safety-inspection rate-limited until Jul 18) · npm run build → then
cargo build --release (needs the running app closed — binary file lock) → Boss test (MARKER-FOUR
recipe) → commit gated on the Boss PASS.
**To file at the ledger reconcile:** the localStorage-net durability PJ (move the net's persistent
layer to a Rust-side atomic_write file — needs its own migration); PJ-104 evidence; PJ-086
re-examination; the 2026-07-14 register→Charter append drift; sweep-register mechanism correction
for PJ-103.

## PJ-103 — review-gated build + Boss PASS + CLOSE (rolls into 2026-07-17, ~05:11)

**The stand-in adversarial review** (`wf_5bb5c713`, 4 refute-first lenses, 770k tokens): 12 findings.
Every one fixed pre-commit or filed (WA#6):
- FIXED: the post-flush typing window (bounded RE-PASS in the new `flushAllForAppClose`) · unserialized
  same-id saves (per-id chain in `noteSession.save`; APP-KILLER-PLAUSIBLE closed unconditionally) ·
  journal-invisible flush failure (`final_flush_residual_dirty` marker, awaited) · the false timeout
  marker (renamed `final_flush_no_ack_5s` with honest semantics) · the boot-window 5s stall (listener
  registered at the TOP of onMount — instant no-op ack during boot) · index↔disk divergence at close
  (AWAITED FTS reindex of flushed notes; embeds stay async → PJ-113) · arrangement starvation
  (persist-FIRST ordering) · the stale `session_flush_notify` contract doc · the updater `relaunch()`
  bypass (SettingsModal flushes+persists before restart).
- FILED: PJ-110 / PJ-111 / PJ-112 / PJ-113 (ledger v1.34).
- **The recipes earned their keep:** my first serialization draft broke save()'s synchronous
  compose+setNet prefix (the beforeunload-stash contract) — caught by 2 MIG-076 recipes
  (type-during-await, compare-and-clear), fixed with the unchained fast path + an eager chained stash.
- Workflow-authoring lesson: the review's verdict-synthesis prompt had an UNINTERPOLATED template
  (my escaped `$`); the synthesizer agent correctly REFUSED to fabricate a verdict (Don't-Make-
  Things-Up honored under delegation) — I synthesized from the four raw registers myself.

**Gates:** svelte-check 0 · vitest 427/427 (after the fast-path fix) · cargo check clean ·
`final_flush_repass` literal verified in `build/` · release binary rebuilt 20:25:36 (npm build first).

**Boss test (staged, on the fresh binary):** Stage 1 — the MARKER-FOUR gesture (type + instant ✕,
the exact recipe that killed MARKER-THREE): marker ON DISK at the close instant (05:11:22 Jul-17),
no net involvement, no reopen needed. Stage 2 — typing burst clean · tab round-trip clean · clean
close instant. **PASS — commit gated on this.**

**Close paperwork:** ledger v1.34 (PJ-103 closed · PJ-110/111/112/113 filed · PJ-086 re-examine
flag · register-correction note) · orientation v3.55 · Charter: PJ-103 close-cycle register +
the 2026-07-14 drift fixed · User Manual close-flush paragraph EN + ×14 locales + the
Notes-Management help topic (`wf_c422cf79`) · evidence `lab/reports/pj103-evidence-000003.log` ·
MoCh-2026-07-16-1810.
