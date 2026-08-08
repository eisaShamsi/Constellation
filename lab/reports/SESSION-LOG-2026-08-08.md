# SESSION LOG — 2026-08-08

**PJ-207 §9, Boss testing and close.** Build log: `SESSION-LOG-2026-08-07.md`.

## 1 - Boss test round 1: the DETECTION is right, the PLACEMENT was broken

Boss ran Stage 1 and stopped at Step 1 with a screenshot: **"Fix the UX first."**

**What the screenshot proves works.** The notice rendered with exactly the text and exactly the
numbers predicted: *"19 notes changed on disk while Constellation was closed, so search may not
show their latest text. 825 notes in your libraries are not in the search index, so searching will
not find them."* Two independent measurements - my read of a copy of his database, and the app's own
live pass - agree to the digit. The mechanism, the counting, the two-sentence split, the plural
rendering and the amber tone all landed.

**What was broken.** The bar was pushed BELOW the status bar and outside the window, and the sidebar
and content area had swapped places. Cause, and it is the risk flagged in the 2026-08-07 log §8 (honest open item) as unverifiable
without a browser: `.app` is a CSS grid whose four columns are **exactly saturated** by its four
in-flow children (dock, sidebar, main-area, right-sidebar). Any additional in-flow child has no cell
to occupy, so auto-placement pushes it - and everything after it - into implicit rows, past
`height: 100vh` and under `overflow: hidden`.

**This was never a §9 defect alone.** `.tpl-err` (a dropped user action) and `.store-err` (a store
that is not saving) have shipped with the identical property since 2026-08-01 and earlier. They fire
rarely enough that nobody had seen it. §9 made a latent breakage constant.

**The fix (Whole-Ecosystem: one concern, three surfaces).** A third grid row - `auto 1fr statusbar`,
so it is exactly 0px when there is nothing to say - and one `.notice-band` grid item spanning every
column at row 1, holding all three bars as flex children so two conditions stack instead of
colliding in one cell. The four workspace panes move to row 2, the status bar to row 3. **Every
in-flow child is now explicitly placed, so nothing auto-places at all** and a fourth bar can be added
later without touching the grid again.

**The lesson, and it is the one this project keeps paying for.** I said in writing that I could not
verify the placement and would not guess - which was right, and it is why the test asked him to
*report* the position rather than confirm one I had asserted. But "I cannot verify it" is a reason
to make the thing verifiable, not a reason to ship it unverified. A saturated four-column grid with
an unplaced child is a fact readable from the CSS; I had the two numbers in front of me and did not
put them together. **The honest hedge in the test caught it at the cheapest possible moment - one
launch - but the defect should never have reached the launch.**

## 2 - Boss test round 2: layout fixed and confirmed; the "failed" step was §8 working

Boss re-ran Stage 1 on the rebuilt binary (09:29). Screenshots:

- **Step 1 PASS.** The notice renders as a full-width band across the top, workspace intact below it
  - the three-row grid fix confirmed on the running app. Text and numbers exact: 19 / 825.
- **Step 4 PASS.** Search for `plarnwick` -> "No results found".
- **Step 3: the count stayed 19 - and the investigation (NOT theorized; verified in both databases
  and on disk) shows it is correct behaviour my tutorial failed to anticipate:**
  - The Boss created "Throughaway note" IN-APP at 09:35:23 inside **كون عيسى** - a LINKED universe
    offered seamlessly by the sidebar. The save wrote its row into **Eisa Universe's** index (body
    WITHOUT plarnwick) - PJ-219's designed behaviour, live-reproduced by accident. كون عيسى's own
    index got **no row**.
  - He closed the app and added `plarnwick test edit` in Notepad at 09:36:40 (file mtime verified).
  - On relaunch, the drift walk visits OWN roots only (§8) - كون عيسى's directory is never stat'ed,
    so `drifted` stayed 19 and the new row landed in the un-rendered `foreign_rows`.
  - Step 4's "nothing found" is doubly correct: Eisa's row holds the pre-edit body; كون عيسى's
    index has no row for the note at all.

**The tutorial gap: "open any one note" in a federated universe includes linked universes' notes,
which §9 is blind to BY DESIGN.** Three inspection rounds verified every UI claim and none of us -
auditor, inspector, me - carried §8's scoping into the note-choice constraint. Round-2 follow-up
(inspector-gated) directs the edit at **المساعد الذكي**, verified this morning as 21 indexed / 21
on disk - the +1 is guaranteed there or the failure is real.

Also measured this morning (fresh copy): تخطيط الدولة 274/276, الكون المعرفي 162/164,
Constellation PKM 1 row vs 798+ files. And the federated-note asymmetry deserves a sentence in the
PJ-219 design discussion: an external edit to a linked note is invisible to the parent's drift check
AND to the child's until the child universe is opened - the child's boot reconcile will then
re-adopt or drift-count it.

## 3 - Boss PASS, and the step closes

Round 3: the Boss edited a note in **المساعد الذكي** (own library, verified 21 indexed / 21 on
disk) in Notepad with the app closed, relaunched — **"The number reads 20 now."** The +1 landed
exactly where the mechanism says it must. With Step 1 (band + numbers), Step 4 (search still blind
— correct until §11) and the layout fix confirmed on screen, **§9 is Boss-passed.**

The round-2 follow-up itself went through the ui-inspector (APPROVED, 10 claims — it re-derived my
21/21 and 798/799 counts itself rather than citing them, traced the cleanup claim through
`constellation_search_reindex` → `reindex_single_note` → `index_note(force: true)`, and grounded
the PJ-219 sentence in `libraries.rs` + the ledger).

**Marker-word bookkeeping for future tests:** `plarnwick` is now ON DISK in two notes — the
المساعد الذكي test note and `كون عيسى\Throughaway note.md` — and in NO index. `zarquon`,
`blorptide`, `vandrasil` were already burned. The count reads **20** until the Boss does the
optional cleanup (delete the line, save in-app → next launch 19).

**Docs note:** help files / User Manual deliberately NOT updated here — the plan schedules every
doc correction at §12, landing the same day as §11. Recorded so the skip is a decision, not a miss.

Committed with: orientation v3.85, PJ ledger v1.69, MoCh, BOOT-BUDGET Criterion-4 status — one
commit, per SO#6/SO#9.

---

# PJ-207 §10 — one progress strip instead of three copies

**Boss-passed 2026-08-08 (~13:30), committed with this entry.**

## 1 · What was built

The plan's claim verified first: `ClassifierScanProgressStrip.svelte` and
`NscBackfillProgressStrip.svelte` were both exactly 159 lines and byte-equivalent modulo six
identifiers (event, two commands, i18n prefix, CSS prefix, console tag). Both Rust contracts are
field-identical (`ScanStatus` / `NscBackfillStatus`; the event payloads likewise).

- **`jobProgressCore.ts`** — the ONE state machine (recover-on-mount, 4 s linger, cancel), plain TS
  because the repo has NO component-mount harness (vitest only; no jsdom/testing-library — adding a
  test stack is its own decision, not §10's). **9 vitest pins**, including the preserved
  progress-clears-cancelling quirk (pinned-not-endorsed) and the no-start-no-flash rule.
- **`JobProgressStrip.svelte`** — the thin shell; `$state.raw` (replace-only snapshot contract);
  props `{eventName, statusCommand, cancelCommand, labelPrefix}`. The dead `$t(k) || 'fallback'`
  idiom (fallback unreachable — $t returns the key) removed; keys verified ×15.
- Consumers re-pointed (+layout `.sb-center`, CatalogerView); both clones DELETED. Net 3 → 2 strip
  components, three consumers planned, one implementation. `MigrationProgressStrip` stays (different
  contract).
- **WA#6 fix while here:** both ORIGINAL strips carried a mount/destroy race — async onMount means a
  component destroyed before `listen()` resolves registers a listener nothing removes, for the
  session. Closed with a `destroyed` guard in the new shell.
- **§11 seam documented in the core's header:** `index_repair_status` plugs in as-is (superset);
  the EVENT side is §11's Rust obligation (a progress event in `JobProgressEvent` shape from
  `note_progress`) — not a licence to widen §10.

## 2 · Reviews + inspection

/simplify (2 agents, reuse+simplification / efficiency+altitude): CLEAN, three fixes applied
($state.raw; the listen-race; the core doc's over-present-tense §11 sentence). Measured emit cadence
justifies the design (classifier ≤~7 events/s, NSC ≤~1.3/s — Rust already batches). Diff-scoped
safety inspection (LF scriptPath, hardcoded files — PJ-220 workaround): **0 confirmed findings.**

## 3 · The Boss test — two stages, five pipeline rounds

- **Stage 1 pipeline:** one wasted round from MY mishap (pointed the inspector at the agent's task
  .output file, which the harness leaves EMPTY — the draft travels in the notification; lesson:
  write drafts to scratchpad myself). Then REJECTED with 3 real findings: an IMPOSSIBLE failure mode
  (a 0/0 flash the code cannot render — visible starts false), an OVERSTATED cross-strip claim (the
  un-clicked strip never shows "Cancelling…" — cancelling is controller-local; it jumps straight to
  "Classification cancelled" on the broadcast event), and a missing near-empty-queue not-a-bug line
  — answered with DATA: 721 notes pending classification, measured with enumerate_pending's exact
  SQL on a fresh copy. Round 3: one ambiguous pronoun ("its label" → the strip's leading text, not
  the button caption). APPROVED round 4.
- **Stage 1 result:** strip appeared bottom-centre, exact wording, total **721 — the digit-exact
  measured prediction** — counted to completion (screenshot at 710/721 98%). The scan finished
  before a natural cancel, so cancel/linger moved to Stage 2; the Cataloger recover-on-mount step
  was correctly VOIDED (a finished job is never resurrected — designed + pinned) and gets its live
  demonstration on §11's minutes-long repair.
- **Stage 2 pipeline:** REJECTED once, 2 findings — both MY prose overstating: "never touches your
  .md files" (the summariser READS the file for an author callout when frontmatter has none;
  writes-never is what's true) and "every background job" (migrations keep their own strip).
  APPROVED round 2. Measured basis: **1,619 notes pending summaries** (v2 predicate, fresh copy);
  the inspector re-derived it independently.
- **Stage 2 result: PASS.** Boss screenshot mid-run: status bar "Building note summaries… —
  75 / 1,568 (4%)" + Cancel, Cataloger button flipped to "Building note summaries…" (live 1,568 vs
  1,619 morning measure — within the stated allowance). Step 4 cancel sequence PASS; Step 5 4-s
  linger + vanish PASS. (Linger on 'done' wasn't separately confirmed on Stage 1; same single timer
  path, pinned by test — noted for honesty.)

## 4 · Gates at close

vitest **909/909** (9 new) · svelte-check **0 errors** · i18n **15/15 ✓** (no key changes) · Rust
**untouched by §10** (suite green from §9's close). Binary 12:17:10 embeds the final frontend.

Help files / User Manual: deliberately untouched — §10 changes no user-visible surface or label
(that is the point), and the plan lands all doc corrections at §12.

---

# PJ-207 §11 — THE DOOR (build log; Boss test pending)

**The step that makes the repair reachable.** Three doors, one runner: "Repair now" on the §9
drift band, Settings → Index → "Index repair" (+ the verbatim per-family report), and the old
automatic doors unchanged. Progress through §10's shared strip (`index-repair:progress`, Full runs
only for start/progress; terminal phase for every scope — a no-op on a hidden strip by pinned
test, and it closes the eventless-adopt window the inspection traced). Completion refreshes every
derived surface without a restart, and the drift notice RE-DERIVES from a fresh scan — the band
updates or clears from facts, never from `ok:true`.

## Ground truth that shaped the build
- `ConvergeReport` existed with exactly the plan's shape (Converged/Skipped/Failed per family) and
  was DISCARDED inside `reconcile_filesystem` — produced, consumed for failure handling, never
  reaching the frontend. §11 serializes it and threads it walk → RunCompletion → RepairState +
  the done event (`RepairReport`, shape pinned by test).
- `initSearchIndex()` already returned the typed SubmitOutcome — my first cut added an
  `index_repair_start` command and DELETED it the same hour: a duplicate door, caught by the map.
- The plan's §11 anchors were stale in five places (Settings :2030-2090 → :2033-2092; the
  storeHealthError render :7442 → :7563; the Index-panel gate :5838 → :5951; ensureSky :1082 →
  :1126; the "Repair now" HOST — the plan drafted it onto the storeHealthError bar, §9's record
  already ruled the drift band is the home).

## The reviews + inspection (9 findings fixed pre-commit, 1 HIGH)
- **HIGH (inspection):** my D2 fix over-corrected — clearing `indexHealthError` on ANY note's
  successful reindex masked a DIFFERENT note's surfaced divergence the moment the user typed a
  character elsewhere. Now path-gated: only the note the bar names can take the bar down.
- The Full-only gate on the done event's `report` (a boot ColdStart's empty receipt could clear
  the red bar it did nothing to earn AND blank the modal's rendered report — three findings, one
  gate).
- The Settings "blocked" reason was written into `testStatus` — the AI section's sentinel that
  renders only 'success'/'failed': the refusal displayed NOWHERE. Own `$state` + rendered row now.
- `tParams` deleted ($t(key, params) already interpolates — the third hand-rolled interpolator);
  `settings.index.repair.blocked` key deleted ×15 (indexDrift.repairBlocked is the one string);
  `submitRepair()` extracted (three doors, one submit decision); `scanStageMaturityForLibrary`
  extracted (toggleLibrary + post-repair refresh, no copy-paste); the precount's boundary test now
  goes through the shared `path_is_under_any`; done-listener heavy refreshes gated on
  "changed anything" (ensureSky is the boot profile's heaviest read); doc drift fixed in 3 files.

## Also in this step
- `record_drift_report` always-emits (the render gate lives frontend-side; a clean rescan must be
  able to REPLACE stale counts — that is how the band clears).
- Post-repair: `reconcile::maybe_schedule` re-runs (warm ~20 ms) then `maybe_schedule_defrag`.
- D2 + storeHealth.index rewritten ×15 — the string that promised "Settings → Rebuild Index" in
  15 languages for a control that never existed now names the door that IS there.
- Concept paper 29-settings.md amended §3/§7/§9/§10 with the Architect §7 wording verbatim.
- Second screen: display-only debounced reload on done.
- i18n: indexRepair.* (7) + indexDrift.repairNow/repairing/repairBlocked + settings.index.repair.*
  (13) + storeHealth.index rewrite — ×15, parity 15/15 ✓.

**Gates:** Rust 1372/0 · vitest 909/909 · svelte-check 0 · parity 15/15. Boss test staged through
the pipeline; commit only after the Boss passes.

## §11 Boss test — Stage 1 PASS (all six steps, digit-exact)

Binary 16:23:27. Four screenshots. The Boss's report against the send-time live measurement
(21 drifted / 830 missing / 2,100 files / 1,249 unchanged, plus his fresh `brimsloe` edit):

| Step | Expected | Boss result |
|---|---|---|
| 1 band | ~22 changed / 830 not in index | **22 / 830** — exact |
| 2 repair | strip to ~2,100, minutes | total 2,100 shown, **"It took 90 seconds"** |
| 3 search | `brimsloe` FOUND | FOUND — Welcome / المساعد الذكي, highlighted |
| 4 band | gone or smaller (fresh look) | **"The band is gone entirely"** |
| 5 receipt | ~851 re-read · 1,249 unchanged · 0 failed | **852 · 1,248 · 0** (= 851+1 / 1,249−1 — his edit moved one note between piles) |
| 6 Sky View | populated, no restart | Pass |

Receipt families: outgoing **2,721** · backlinks **2,721** · sky **1,732** · tags **3,689** ·
review **0**. PJ-223's class is repaired on the live universe: 830 missing-from-index → band gone.

**Discovered at the pass (WA#6 — fix in flight, not deferred):** "Review schedule — updated 0" is
a hard-coded placeholder — `review::recompute_all_in` returns `Result<(), _>` (no count) and
`converge.rs:257` maps success to `Converged(0)`, unlike `tag_counts` / incoming which return real
counts. The receipt said "nothing needed doing" about a family that rebuilt every row. Fix:
return the pass-2 rebuilt-row count (sibling shape). Per the tested-build-before-commit rule the
exact passed build commits FIRST; the fix rides §11 Stage 2's Boss verification (Arabic RTL +
mid-run Cancel), where the receipt must show a real non-zero review count.

PCS: orientation **v3.87** (new file, preamble + Criterion-4 row) · Pending Jobs **v1.71** (new
file; PJ-223 noted repaired-live, formal close at §15) · MoCh-2026-08-08-1330 · this record.
