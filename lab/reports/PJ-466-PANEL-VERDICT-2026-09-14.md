# PJ-466 — Panel verdict, 2026-09-14

**Read-only design panel.** Workflow `wf_6d74a307-646` — four lenses (exits · engine · ecosystem · voice), two adversaries each, one chair. 13 agents, 280 tool calls, 2.11M subagent tokens. Nothing was written to the app by this panel.


## The concept (the horse)

> The end-of-run screen is the Mold Repair Door's **receipt** — the run's own account of itself: before the user closes it, it must say which template files were changed, which were left alone with nothing owed, which still carry the false birth date and why, and which were never attempted — so that closing the dialog leaves him knowing the true state of his own files.


## Established (survived attack)

- FOUR exits reach the summary, not the ledger's three: MoldRepairDialog.svelte:173 (delicate-batch halt), :182 (between-batches blocker), :194 (normal completion), :198 (the catch). I re-read all four.
- `summaryTitle` is static and sits OUTSIDE the `{#if report}` guard — MoldRepairDialog.svelte:329 vs :330 — so "Templates fixed" (en.json moldRepair.summaryTitle) renders over every one of the four exits, including one where zero files were fixed.
- `failed` is computed by subtraction and conflates two incompatible things: mold_repair.rs:463-467 `let repaired = outcomes.iter().filter(|o| o.ok).count(); ... failed: outcomes.len() - repaired`. The halt at MoldRepairDialog.svelte:171 `if (r1.failed > 0)` therefore fires on a benign refusal exactly as on a write failure, and en.json moldRepair.summaryLine renders both as "skipped".
- There are TEN non-success return sites in `repair_one`, not the session register's nine: mold_repair.rs:496, :503, :507, :511, :517, :523, :527, :532, :537, :541. The register omitted :507 `Err(e) => return refuse(format!("Could not read it: {e}"))`.
- mold_repair.rs:517 ("Skipped: the change could not be prepared safely.") is UNREACHABLE. `strip_stamp_and_mark_template` (mold_repair.rs:194-196) fails only at `mold_evidence(content)?` or `split_frontmatter(content)?`, and `repair_one` already proved `mold_evidence(&before)` is `Some` six lines earlier at :510 — which itself required `split_frontmatter` (mold_repair.rs:169).
- THE HOUSE ALREADY SHIPS THE EXACT RECEIPT THIS JOB NEEDS. src-tauri/src/phantom_prune.rs:585-604 `PruneReceipt { removed, skipped, failed, unknown, stopped_early, refused }`, whose own doc comments draw the line this job is trying to invent: :590-592 skipped = "the file reappeared, or the row no longer classified as a phantom. **Never an error.**", :593-594 failed = "Candidates whose delete returned `Err`. Their rows are still there.", :598-600 stopped_early = "The counts above are what actually happened up to that point, not a projection."
- That receipt is RENDERED, two doors from the mold door's own entry point, at SettingsModal.svelte:2753-2790 — a neutral title ("Last removal"), one sentence per non-zero class, a refusal replacing the lot, every count through `$tn('plurals.entries', n)`. Its in-code comment states the rule: "`removed` alone would flatter a partial run, so every non-zero outcome gets its own sentence and a refusal replaces the lot."
- The house's halt precedent for a consented, batched file operation is HALT-ON-FIRST-FAILURE: src-tauri/src/mig108.rs:1044-1046 "Halts with Err on the first failure (open handle, permissions) — the journal names exactly where; a re-run resumes from the first unmoved entry."
- "A backup of every file was saved first" is provably false. mold_repair.rs:435 creates the folder before anything is touched and :468 returns `backup_dir` unconditionally, while the per-file backup at :522 sits downstream of five earlier returns (:496, :503, :507, :511, :517).
- A write failure leaves the note byte-identical, provably: write_gate.rs:233-288 `atomic_write` writes only a same-dir `.cnstmp`, fsyncs, then replaces with five retries, and on exhaustion `let _ = fs::remove_file(&tmp); return Err(...)`. The original is opened for writing nowhere. So "Your file is exactly as it was" is sayable for mold_repair.rs:532.
- The verify-failure rollback is UNCHECKED and its message asserts an outcome the code discards: mold_repair.rs:540 `let _ = std::fs::write(path, before.as_bytes()); // put it back` then :541 reports "The change did not verify, so it was undone: {why}" regardless.
- One outcome class describes a file that WAS changed on disk: mold_repair.rs:531 `gate_write` succeeds, then :536-537 refuses with "Wrote the file but could not read it back to verify it." — `ok:false`, counted in `failed`, rendered by summaryLine as "skipped". No restore is attempted.
- PJ-469 is not cosmetic — it causes a real, avoidable write. A preview returning `will_apply:false` via mold_repair.rs:397 ("The prepared change did not verify cleanly") is still sent to the engine by MoldRepairDialog.svelte:158-159, where the same content takes the same path: strip succeeds, the backup is written, `gate_write` WRITES the file (:531), `verify_change` fails (:539) and the file is restored by the unchecked write at :540.
- `relinked_sources` over-counts (PJ-470), and it renders on this very card. mold_repair.rs:595-623 collects every source found and re-indexes with `let _ = crate::search::reindex_single_note(...)`, returning `sources` regardless; MoldRepairDialog.svelte:334 prints it as "{count} notes ... were refreshed."
- The engine's outcome text is English in all 15 languages: the literals at mold_repair.rs:496-563 are rendered raw at MoldRepairDialog.svelte:340 `<div class="mr-harm">{o.detail}</div>`.
- The user is not stranded after a halt, and the app already provides the way back: +layout.svelte:10683 `onDone={() => { showMoldRepairDialog = false; void refreshMoldRepairCount(); }}`, :620-625 re-scans, :8296 renders the banner while `moldRepairCount > 0 && !moldRepairDismissed`, and `moldRepairDismissed` is set only by the banner's own ✕ (:8302).
- A FIFTH honesty exit exists that no lens's scope covered until the attack round: MoldRepairDialog.svelte:99-106 — a thrown `scan_stamped_molds` sets `candidates = []` and calls `onDone?.()` without ever showing the dialog, and +layout.svelte:624 `catch { moldRepairCount = 0; }` then retracts the banner. A user-initiated click on "Review templates to fix" can therefore make the notice disappear with the molds still on disk and nothing said.
- The i18n gate refuses an English-first landing: scripts/i18n-parity.mjs:154 builds the reference from the UNION of all locales. Baseline verified green today: `node scripts/i18n-parity.mjs` → "✓ All 15 locales in parity".
- The plural machinery and the noun this job needs are house-standard: en.json `plurals` carries 33 nouns including `files`, `entries`, `notes`; ar.json carries the full six CLDR categories for each. The model receipt (SettingsModal.svelte:2767, :2776, :2781, :2786) uses `$tn('plurals.entries', n)` on every row.
- `detectDir` exists for the RTL case the receipt lacks: src/lib/utils.ts:310 `export function detectDir(text: string): 'rtl' | 'ltr'`. MoldRepairDialog.svelte has exactly one `dir` attribute, on the card at :267.

## Overturned

- THE LEDGER'S SCOPE — "three exits". There are four (MoldRepairDialog.svelte:173, :182, :194, :198). The omitted normal-completion exit is where the Boss has ALREADY seen a genuine write failure worded "skipped" (the rehearsal's Recovery 1, lab/reports/SESSION-LOG-2026-09-12.md). A fix scoped to the ledger's three would leave a confirmed-wrong screen shipping.
- THE SESSION'S OWN REGISTER, ITEM 3 — "`report` can be null ... e.g. no delicate files + a between-batches blocker". REFUTED by reading. With `delicate.length === 0` there is NO await between the blocker check at MoldRepairDialog.svelte:155-156 and the one at :179 (lines :158-162 and the `try {` at :163 are synchronous; the only await is inside `if (delicate.length > 0)`), so single-threaded JS cannot let a tab open in between and :179 cannot see a blocker :155 did not. Exit 2 therefore ALWAYS carries `report = r1` (:170). The real Exit-2 screen is worse than the empty one the register imagined: "Templates fixed / N fixed, 0 skipped / A backup of every file was saved first" over a run whose second half never happened. `report === null` reaches the summary by exactly one path — the catch at :198 when a batch's invoke throws with no delicate batch having run.
- THE SESSION'S OWN REGISTER, ITEM 4 — the four-refusals/five-failures enumeration. Ten sites, not nine; :507 ("Could not read it") was missed entirely, and :517 is unreachable. Any classification built on the nine leaves a filesystem read error unassigned.
- THE LEDGER'S QUESTION FOR THE BOSS — "should a stale-file refusal halt the cascade at all?" It is not an open question, and putting it to him would violate the 2026-09-14 LAW. phantom_prune.rs:590-592 already rules that a second-look disagreement is "Never an error"; mig108.rs:1044-1046 already rules that a consented batch halts on a genuine failure; and the mold door's own header (MoldRepairDialog.svelte:25-26) says the halt exists "so a systematic problem never reaches the ordinary files" — which a file that is already fixed is not.
- EVERY LENS'S PROPOSED NEW TAXONOMY — the exits lens's `fixed|left|failed`, the engine lens's `Repaired|NotNeeded|Failed`, the voice lens's three headings. All were invented beside a shipped one. `PruneReceipt` (phantom_prune.rs:585-604) already carries the distinction, already has its strings, already has its render, already uses `$tn`, and sits in the same Settings page as the mold door's own entry point. Under the Whole-Ecosystem Fix Law, shipping a thirteenth vocabulary is the drift the law exists to prevent.
- THE EXITS LENS'S "no sibling surface has a precedent phrase for a partial stop" — false on the file it did not open: en.json `settings.index.phantoms.stoppedEarly` = "The removal stopped early ... run it again to finish." Its boss_call built on that claim is void.
- THE EXITS LENS'S EXIT-2 REROUTE (`mode = 'blocked'`). Killed twice over: the blocked branch (MoldRepairDialog.svelte:314-323) renders neither `report` nor `outcomes`, so routing there DELETES the receipt for a batch that has already written to disk; and its retry at :322 re-enters `startRepairInner`, which resets `outcomes = []` (:162) but never `report`, so batch 1's totals are merged twice at :191 while the already-repaired files come back as ✕ refusals.
- THE ENGINE LENS'S CLASSIFICATION OF mold_repair.rs:503 AS BENIGN. A file refused by the write-sovereignty guard still carries the false birth date, and a guard whose own comment calls it "defense in depth — the scan already returns own-only paths" firing at all means the scan and the guard disagree about which universe owns the file. That is the most systematic failure in the set; it must halt.
- PJ-469'S LOW/GROUP-3 RANKING. It is not cosmetic: a `will_apply:false`-by-verify candidate is actually WRITTEN by `gate_write` and then restored by the unchecked `let _ = std::fs::write` at mold_repair.rs:540. Pre-filtering it is a two-clause change that removes a needless write from a write path.
- THE VOICE LENS'S `summaryNothingAttempted` ("No file was changed and none was written to. Every template is exactly as it was.") as proposed — it would have been FALSE on the read-back class (mold_repair.rs:536-537), which writes first. It becomes sayable only after the classification makes "nothing outside `repaired` and `failed` reaches a write" a provable invariant — which is why the classification is the fix and the wording is its consequence.
- THE ECOSYSTEM LENS'S WIDENING TO MigrationProgressStrip / search.rs AND ImporterModal. Real defects, different engines, different concerns; they do not consume the mold outcome vocabulary. Filed, not folded (see boss_ruling_needed #2). Its "eleven backfill modules with zero `.emit(`" aside was also factually wrong (review_backfill.rs emits twice, to no consumer) and is dropped entirely as irrelevant to this job.
- THE VOICE LENS'S FRAMING OF THE COST AS THE REASON TO SPLIT. Its ~419-vs-567 string arithmetic was sound, but adopting the shipped receipt shape does not shrink the string count much — it removes the invention risk. The honest number is ~435 written, and the job is a small /migration either way.

## Scope — IN

- All FIVE honesty exits of the Mold Repair Door — the four `mode = 'summary'` writers (MoldRepairDialog.svelte:173, :182, :194, :198) and the silent pre-run dismissal on a thrown scan (:99-106, with +layout.svelte:624's count retraction).
- The engine's outcome classification, modelled on the shipped `PruneReceipt`: `MoldRepairOutcome.ok: bool` becomes `kind: repaired | kept | failed` plus a stable `reason` code; `MoldRepairReport` gains `kept` and `backed_up`; `failed` narrows to "not repaired and the defect is still owed". The `refuse` closure at mold_repair.rs:491-493 splits into `keep(code, detail)` / `fail(code, detail)` so no future return can join a class by default.
- Assignment of all ten sites, explicitly: KEPT = :496 (file gone), :511 (no longer a stamped template). FAILED = :503 (not this universe's library — the defect stands), :507 (unreadable), :517 (unreachable, kept as a defensive `fail`), :523, :527, :532, :537, :541.
- The halt at MoldRepairDialog.svelte:171 keeps its exact expression `if (r1.failed > 0)` and becomes correct for free — matching mig108.rs:1044-1046 and the door's own stated purpose.
- The checked rollback: mold_repair.rs:540 stops discarding its Result, with its own reason code when the restore fails.
- PJ-469 — exclude `will_apply === false` from BOTH batches and synthesise their receipt rows from the preview's reason, so a known-refusing file is neither written nor silently dropped from the account.
- PJ-470 — count only successful re-indexes in `relinked_sources` (mold_repair.rs:617-620), because that number renders on this card.
- PJ-471 in full for this namespace — the engine's English `detail` literals become localised `reason` keys, and the four `{count}`+fixed-noun keys become `$tn`. PJ-471 closes with this job.
- The receipt's RTL handling: `detectDir()` on the file name, `dir="ltr"` on the technical tail.
- i18n ×15 (ar de en es fa fr he hi ja ko pt ru tr ur zh), one new plural noun `plurals.templates` with each language's own CLDR categories.
- SO#2 docs: docs/help.uConstellation.World/Templates/Templates.md, docs/User Manual.md, and the 8 translated manuals that carry the mold-door passage (ar de es fa ja ko ru zh) — none of them mentions the halt today.

## Scope — OUT

- PJ-468 (write_gate.rs journals nothing when `atomic_write` returns Err at :441, before `journal_ext`). Verified true, but it is the gate every writer in the app passes through — a write-path change, /migration-shaped on its own. The mold-local half (the ungated rollback) is IN.
- PJ-467 (the scan on every watcher flush). Performance; shares no code with the receipt.
- ImporterModal's success tick over a zero-import, N-error run, and MigrationProgressStrip's missing `error` phase. Real, same family, different engines — filed as new PJ entries (see boss_ruling_needed #2).
- Mig108UnifyDialog. Checked under the Whole-Ecosystem Fix Law and ruled out: its summary is reached only after the engine returns Ok (:215) and every failure routes to resume (:216+). Its unrendered `summary.skipped` is an omission, not a false claim — file it LOW.
- A "try again" / "fix the rest anyway" control on the summary. Refused: the banner already returns with a fresh count (+layout.svelte:10683, :620-625, :8296), and a retry on this screen re-sends repaired files and double-counts `report` (:162 resets `outcomes`, never `report`; :191 then merges batch 1 twice).
- The six translated manuals with no mold-door passage at all (fr he hi pt tr ur). A missing passage pre-dates this job; file it.

**Reason:** The concern is ONE: "this screen's account of its own run is untrue." It cannot be fixed in the locale files, because `MoldRepairOutcome` carries only `ok: bool` and free text (mold_repair.rs:56-62) — there is no sentence in any language that separates "left alone, nothing owed" from "still broken". The same missing field is why a benign refusal halts the run, why PJ-469's refusal is damaging, and why the receipt speaks English in fifteen languages. One field, one commit, four ledger entries closed. Everything excluded above either belongs to a different engine or to the shared write gate.


## Build plan


### Step 1
Classify the engine's outcomes on the shipped `PruneReceipt` shape. Split the `refuse` closure into `keep(code, detail)` / `fail(code, detail)`; add `kind` + `reason` to `MoldRepairOutcome`, `kept` + `backed_up` to `MoldRepairReport`; assign all ten sites per the scope ruling; check the rollback write at :540 and give its failure its own reason; count only successful re-indexes in `relinked_sources`. The Svelte interface is updated to the new shape and the dialog keeps rendering exactly today's screen (`ok` derived as `kind === 'repaired'`), so nothing user-visible changes yet.

- **Files:** `src-tauri/src/mold_repair.rs`, `src/lib/components/MoldRepairDialog.svelte`
- **Verification:** `cargo test` green including the 14 existing mold_repair tests, plus a NEW table test asserting (a) each of the ten non-success sites returns its declared `kind` and `reason`, and (b) the invariant that every `kind` other than `repaired` and `failed` returns BEFORE the first `std::fs::write` — the invariant every "nothing was changed" sentence later rests on. `npm run check` clean. The door still shows the old screen; the only behaviour change is that a stale-file refusal no longer sets `failed`, provable by the new test.

### Step 2
Stop sending files the app already knows will refuse (PJ-469). Filter both batch lists at MoldRepairDialog.svelte:158-159 on `previews.get(p)?.will_apply !== false`, and synthesise a `kept`/`notAttemptedStale` outcome row for each excluded file from its preview `reason`, so it stays on the receipt rather than vanishing.

- **Files:** `src/lib/components/MoldRepairDialog.svelte`
- **Verification:** With a candidate whose preview returns `will_apply:false` via mold_repair.rs:397, the write journal shows NO `pj454_mold_repair` write for that path (today it shows one, immediately followed by the unchecked restore), and the file still appears on the receipt with a reason. The delicate batch no longer contains it, so it cannot halt the run.

### Step 3
Land the copy and the render: the four title variants, the per-class sentences (one per non-zero class, PruneReceipt-style), the stopped/blocked/error ledes, the not-attempted sentence, the corrected backup sentence gated on `backed_up > 0`, the 13 localised reason lines with the raw engine text as a dimmed `dir="ltr"` technical tail, `detectDir()` on the file name, the new `plurals.templates` noun, and `$tn` on the four existing count keys. `summaryLine` is deleted; the word "skipped" leaves this door in every language.

- **Files:** `src/lib/components/MoldRepairDialog.svelte`, `src/lib/i18n/en.json`, `src/lib/i18n/ar.json`, `src/lib/i18n/de.json`, `src/lib/i18n/es.json`, `src/lib/i18n/fa.json`, `src/lib/i18n/fr.json`, `src/lib/i18n/he.json`, `src/lib/i18n/hi.json`, `src/lib/i18n/ja.json`, `src/lib/i18n/ko.json`, `src/lib/i18n/pt.json`, `src/lib/i18n/ru.json`, `src/lib/i18n/tr.json`, `src/lib/i18n/ur.json`, `src/lib/i18n/zh.json`
- **Verification:** `node scripts/i18n-parity.mjs` reports "✓ All 15 locales in parity" (it reports that today, so a regression is visible) and `npx vitest run tests/i18n` green — including the CLDR-category suite for the new `plurals.templates`. A written exit table in the commit message maps each of the four `mode = 'summary'` writers to the title, the class sentences and the receipt it now produces, with the guard condition for each. Any new number-initial Hebrew string carries U+200F, matching the four existing he.json moldRepair strings.

### Step 4
Close the fifth exit: a thrown `scan_stamped_molds` must not dismiss the door in silence. Keep the dialog visible and render an honest refusal in the register of `mig108.blockedBody`; and stop +layout.svelte:624's `catch { moldRepairCount = 0; }` retracting the banner on a failed re-scan — a scan that could not run is not a universe with zero molds.

- **Files:** `src/lib/components/MoldRepairDialog.svelte`, `src/routes/+layout.svelte`
- **Verification:** With `scan_stamped_molds` forced to return Err behind a dev-only flag, clicking "Review templates to fix" shows the refusal card instead of nothing, and the banner is still present after closing it. Today both disappear.

### Step 5
Run the standing safety inspection diff-scoped over the changed files, fix every confirmed finding before the commit, then take the Boss test material through the pipeline.

- **Files:** `src-tauri/src/mold_repair.rs`, `src/lib/components/MoldRepairDialog.svelte`, `src/routes/+layout.svelte`
- **Verification:** `Workflow({ name: 'safety-inspection', args: { files: [...] } })` returns with zero unfixed confirmed findings; `/simplify` on the final diff; then `tutorial-auditor` → `ui-inspector` (APPROVED, not merely un-rejected) → panel, and only then the staged Boss test. Release binary built BEFORE the tutorial is sent (`npm run build` then `cargo build --release`, with the new UI string grepped out of `build/`).

### Step 6
PCS: help topic + English User Manual + the 8 translated manuals gain the halt paragraph; the ledger closes PJ-466 / PJ-469 / PJ-470 / PJ-471 with evidence and files the four new entries (ImporterModal, MigrationProgressStrip, Mig108 unrendered `skipped`, the six manuals missing the passage); session log; MoCh; orientation v-bump — all in the same commit as the work.

- **Files:** `docs/help.uConstellation.World/Templates/Templates.md`, `docs/User Manual.md`, `docs/Constellation Pending Jobs v2.15.md`, `lab/reports/SESSION-LOG-2026-09-14.md`, `docs/MoCh/`, `docs/`
- **Verification:** SO#9 reconciliation written before the session-log entry; every closed PJ carries its commit hash and section; the new PJ numbers are allocated and ranked; the orientation version bumps as a NEW file alongside the existing ones.


## The copy, as ruled

```
TITLES (choose in this order; the first that matches wins)
  stopped before finishing  → moldRepair.summaryTitleStopped = "Stopped before finishing"
  else failed > 0           → moldRepair.summaryTitlePartial = "Some templates were fixed"
  else repaired > 0         → moldRepair.summaryTitle        = "Templates fixed"   (unchanged, already ×15)
  else                      → moldRepair.summaryTitleNone    = "Nothing needed fixing"

WHY IT STOPPED (one, only on a stopped run)
  moldRepair.stoppedCare  = "A file that needs care could not be fixed, so Constellation stopped there rather than let the same thing reach the rest."
  moldRepair.stoppedOpen  = "A template was opened while the fix was running. Constellation does not write to a file you have open, so it stopped there."
  moldRepair.stoppedError = "The repair could not finish: {reason}"
  moldRepair.resume       = "Nothing is lost. You can run this again from the notice at the top of the window."

WHAT HAPPENED (one sentence per non-zero class — the phantom-receipt pattern)
  moldRepair.sumFixed        = "Fixed {noun}."
  moldRepair.sumKept         = "Kept {noun} as they were: already fixed, edited since the scan, or no longer on disk."
  moldRepair.sumFailed       = "Could not fix {noun}. They still carry the birth date — each one's reason is below."
  moldRepair.sumNotAttempted = "{noun} were not attempted. Nothing about them was changed."
  (every {noun} is $tn('plurals.templates', n); plurals.templates en = one "{count} template" / other "{count} templates")

THE BACKUP (rendered only when backed_up > 0 — it is now a counted fact, not an assumption)
  moldRepair.backupAt = "Each file that was changed was backed up first, at: {dir}"   (replaces "A backup of every file was saved first, at: {dir}")

DELETED
  moldRepair.summaryLine — "{repaired} fixed, {failed} skipped." The word "skipped" leaves this door.

THE RECEIPT, ONE LINE PER FILE (localised; the raw engine text follows as a dimmed ltr tail)
  moldRepair.reason.repaired           = "The birth date was removed and the file is marked as a template."
  moldRepair.reason.repairedIndexLater = "Fixed. The search index could not be updated just now — it will catch up on the next launch."
  moldRepair.reason.gone               = "The file is no longer there."
  moldRepair.reason.notStamped         = "No longer a stamped template — it may have been edited, or already fixed."
  moldRepair.reason.notAttemptedStale  = "Not attempted — the check before the run showed this file no longer needs fixing."
  moldRepair.reason.notOurs            = "This file is not in this universe's own libraries, so nothing was written to it. It still carries the birth date — open the universe it belongs to and fix it there."
  moldRepair.reason.unreadable         = "The file could not be read, so nothing was changed."
  moldRepair.reason.notPreparable      = "The change could not be prepared safely, so nothing was written."
  moldRepair.reason.backupFailed       = "The backup could not be saved, so nothing was changed."
  moldRepair.reason.backupMismatch     = "The backup did not read back identical, so nothing was changed."
  moldRepair.reason.writeFailed        = "The change could not be written. Your file is exactly as it was."
  moldRepair.reason.readBackFailed     = "The file was changed, but it could not be read back to check it. Its backup is in the folder above — please look at this one."
  moldRepair.reason.verifyFailed       = "The change did not come out right, so it was put back."
  moldRepair.reason.verifyNotRestored  = "The change did not come out right and could not be put back. Use its backup in the folder above."
  technical tail, house pattern (cf. settings.index.repair.familyFailed "FAILED — {error}"):  — {detail}

THE SCAN THAT COULD NOT RUN (the fifth exit)
  moldRepair.scanFailedTitle = "Constellation could not check your templates"
  moldRepair.scanFailedBody  = "It could not read your libraries just now, so it has stopped and changed nothing. This is usually temporary — try again in a moment."

REWORDED FOR PLURALS (PJ-471, same dialog, same pass)
  moldRepair.count    = "{noun} in this universe carry a birth date they should not."      ($tn plurals.templates)
  moldRepair.repair   = "Fix these {noun}"                                                  ($tn plurals.templates)
  moldRepair.inbound  = "{noun} point to this one."                                         ($tn plurals.notes)
  moldRepair.relinked = "{noun} that linked to a fixed template were refreshed."            ($tn plurals.notes)

NOTES FOR THE 14 TRANSLATIONS
  Register is set by the existing siblings — settings.index.phantoms.skipped "Kept {noun}: …", .unknown "…were left alone", .stoppedEarly "…run it again to finish", mig108.blockedBody "Rather than guess, it has stopped and changed nothing."
  Short, declarative, one idea per sentence, no idiom. de keeps Sie-form; ja keeps です/ます; ar stays formal MSA; any number-initial he string carries U+200F, matching he.json's four existing moldRepair strings.

DOCS (help topic + User Manual + the 8 translated manuals)
  "Files that need care are fixed first, on their own. If one of them cannot be fixed, Constellation stops there and does not touch the rest — those files are left exactly as they were, and the notice returns so you can run the fix again when you are ready."
```


## Already decided by the app — NOT put to the Boss

- "Should a stale-file refusal halt the cascade at all?" — NO. phantom_prune.rs:590-592 already defines that class: "the file reappeared, or the row no longer classified as a phantom. **Never an error.**" The door's own header (MoldRepairDialog.svelte:25-26) says the halt exists "so a systematic problem never reaches the ordinary files", and a file that is already fixed is not one. This was the ledger's question for the Boss; it is a coding defect, not a design choice, and asking it would violate the 2026-09-14 LAW.
- "Should a genuine write failure in the delicate batch still halt the run?" — YES. mig108.rs:1044-1046: a consented, journaled, batched file operation "Halts with Err on the first failure … a re-run resumes from the first unmoved entry." Same shape, same answer; the narrowed `failed` restores the door's stated rule rather than reopening it.
- "May the end-of-run title change with the outcome?" — YES, twice over: SettingsModal.svelte:2796-2802 hangs the state off a neutral "Last repair", and JobProgressStrip.svelte:99-104 selects among `.done` / `.cancelled` / `.error`.
- "What does a partial-run receipt look like?" — ALREADY BUILT. SettingsModal.svelte:2753-2790, whose own comment is the rule: "`removed` alone would flatter a partial run, so every non-zero outcome gets its own sentence and a refusal replaces the lot."
- "Is 'A backup of every file was saved first' actually wrong?" — measured, YES. mold_repair.rs:435 creates the folder before anything is touched, :468 returns it unconditionally, and the per-file backup at :522 sits behind five earlier returns. WA#6 requires correcting a false statement; nothing to rule.
- "Should the raw technical error stay visible to the Boss?" — YES, inside a translated frame. The house pattern is shipped twice: `settings.index.repair.familyFailed` = "FAILED — {error}" and `settings.index.phantoms.refused` = "The removal did not run: {reason}".
- "Should the receipt's per-file reasons be translated?" — YES. CLAUDE.md's i18n convention: every user-facing string through `$t()` ×15. They are Rust literals rendered raw at MoldRepairDialog.svelte:340 today — a defect against a standing rule.
- "Can we land English now and translate later?" — NO. scripts/i18n-parity.mjs:154 builds its reference from the UNION of all locales, so an en-only key makes the other 14 report it missing and the CLI exits 1.
- "Should the counted sentences be plural-aware?" — YES. MIG-087, and the receipt I am copying already uses `$tn('plurals.entries', n)` on every row (SettingsModal.svelte:2767, :2776, :2781, :2786).
- "Should the halted summary offer a Try again button?" — NO. The banner already returns with a fresh count (+layout.svelte:10683 → :620-625 → :8296), and a retry on this screen would re-send the just-repaired files and double-count `report` (:162 resets `outcomes`, never `report`; :191 then merges batch 1 twice). Constraint as Design: the app already has the path.
- "Is this a /migration?" — YES, by CLAUDE.md's own boundary test: it crosses Rust ↔ Svelte (the IPC report shape) and touches a write-path invariant. Told to the Boss as a fact, with only the priority left for him.
- "What should the between-batches blocker screen look like?" — it does NOT get routed to the existing 'blocked' screen, and that is settled by reading it: MoldRepairDialog.svelte:314-323 renders neither `report` nor `outcomes`, so it would delete the receipt for files already written.

## For the Boss to rule


**1. Here are the exact sentences the screen will say after a run that stopped part-way. Do you want to change any of the words — in particular "Kept 3 templates as they were" for the files that needed nothing, and "Could not fix 1 template" for a file that is still wrong? (The full set is in the copy above.)**

- *Why it is not already decided:* The SHAPE is settled by the app — the phantom-removal receipt two doors away in Settings already does neutral-title-plus-one-sentence-per-outcome, and I am copying it rather than inventing one. But no existing string says these sentences for THIS door, and Constellation's user-facing vocabulary is yours by standing ruling (the Linked Universe ruling; "مصادر, not مَسَادِر"). A wording job whose wording I choose alone is the shape that once recommended a retired name back to you.
- *Panel recommendation:* Ship as drafted; they are modelled word-for-word on your own existing strings ("Kept {noun}: …", "…were left alone", "…run it again to finish"). Amend any word you dislike and I will carry it through all 15 languages.

**2. This grew. It is no longer a wording patch: it changes the engine's report shape (Rust ↔ Svelte), so by your own Migration Rule it runs as a small /migration — and it now closes four ledger entries at once (PJ-466, PJ-469, PJ-470, PJ-471) for about 24 new strings across 15 languages. Do you want it run now, ahead of PJ-456's 107-file wave, or after? And two defects I found on OTHER doors — the importer showing a green tick over a failed import, and one progress strip that can never say it failed — I am filing rather than fixing here, because they belong to different engines. Confirm that filing, or tell me to fold them in.**

- *Why it is not already decided:* Priority order in the ledger is yours (the "► Next action" line), and the /migration boundary test is what makes this a bigger commitment than the ledger's MED filing implies. The second half is a Working-Agreement-#6 obligation: a discovered defect is fixed in the same pass or explicitly ruled on by you — never quietly parked.
- *Panel recommendation:* Run it now — it is the door you have already had to look at twice, and doing it before PJ-456 means the 107-file wave lands on an engine that reports honestly. File the importer and the progress strip as their own entries; they share the disease but not the code, and folding them in would double the surface under one test.


## Risks

- Changing `MoldRepairOutcome.ok` is an IPC contract change. Verified there is exactly one frontend consumer (MoldRepairDialog) plus the command registration in lib.rs, and no test pins the serde shape — but this is the step that makes the job a /migration, and it must land with `npm run check` and the Rust suite in the same commit, never split across two.
- The unchecked rollback at mold_repair.rs:540 becomes checked. That changes behaviour on a write path under a failure condition nobody has reproduced. Mitigation: it only adds a branch on an error that is currently discarded, and the diff-scoped safety inspection is mandatory before the commit.
- Four of the ten reason codes are not reachable on demand (:517 is provably unreachable; :507, :527, :537 need a filesystem that misbehaves). Their strings will ship untested against a live screen. Mitigation: unit tests pin site→code, and the Boss test covers the three that PJ-454's rehearsal already produced (write failure, stale file, clean run).
- One new plural noun costs each language its own CLDR category set (ar six, ru four, he/es/fr/pt three, zh/ja/ko one). The parity test enforces exactly-its-own-categories, so a copied-across noun fails the gate rather than shipping wrong — the risk is rework, not a silent defect.
- The classification rests on one invariant: nothing outside `repaired` and `failed` reaches a write. That is true today because :496 and :511 both return before the backup at :522 — but it is an invariant a future return site could break silently. It is pinned by a test in step 1 precisely because every "nothing was changed" sentence depends on it.
- PJ-469's pre-filter makes the review screen's "Fix these {n} templates" a slightly larger promise than the run attempts, when previews exist. Accepted deliberately: the excluded files still appear on the receipt as not-attempted-with-a-reason, so the account stays complete, and the alternative (a count that shifts under the button between showing the sample and clicking) is worse.
- The word "skipped" disappears from a screen the Boss has already seen and passed. He should be told that plainly in the test material rather than discovering it.

## Dissent, and how the chair ruled

"Four lenses, eight attacks, and the single most important finding came from an attacker, not a lens: the house ALREADY ships this receipt. `PruneReceipt` (phantom_prune.rs:585-604) and its render (SettingsModal.svelte:2753-2790) sit in the same Settings page as the mold door's own entry point, carry the refusal/failure distinction in their doc comments, and already use `$tn`. Three of the four lenses independently proposed inventing a taxonomy while telling the panel no such precedent existed. I ruled for the shipped shape and against all three inventions — under the Whole-Ecosystem Fix Law, a thirteenth vocabulary beside a working twelfth is the drift the law exists to prevent.\n\nEXITS vs ENGINE on rerouting the between-batches blocker to the 'blocked' screen. The exits lens proposed it; its attacker showed the blocked branch renders no receipt and its retry re-sends repaired files while `report` is never reset. RULED AGAINST the reroute. The blocker stays on the summary, with the receipt intact.\n\nENGINE vs ITSELF on mold_repair.rs:503. The engine lens classed the write-sovereignty refusal as benign (\"the file is fine as it stands\") and its attacker proved it is not — the file still carries the false birth date, and the guard firing at all means the scan and the guard disagree about ownership. RULED for the attacker: :503 is a failure and halts. The same test disposed of :507 (unreadable): not benign, halts.\n\nEXITS and VOICE both wrote copy asserting \"your files are exactly as they were\". Their attackers proved it false on mold_repair.rs:536-537, which writes first, and on :540, whose restore is discarded. RULED: no positive on-disk guarantee may be written that the classification does not prove. The invariant is then pinned by a test, which converts the forbidden sentence into a sayable one — that is the whole argument for putting the fix in the engine rather than the locale files.\n\nENGINE vs ITS ATTACKER on whether the halt threshold is the Boss's. The lens asked him and admitted in the same breath it had not read the sibling engines; the attacker found mig108.rs:1044-1046. RULED: the house answers it, the question is withdrawn, and the Boss's time is spent on the two things only he owns.\n\nECOSYSTEM wanted ImporterModal and MigrationProgressStrip folded in. Its own attacker found a false stated fact in its enumeration (the backfill-module count) — a reminder that a wide sweep is the easiest place to be confidently wrong. RULED OUT of the build and INTO the ledger, with the WA#6 obligation surfaced to the Boss as one line inside an existing question rather than as a third.\n\nVOICE was right about the bill and wrong about why. Its ~419-string measurement survived re-verification; its inference that splitting the job would shrink it did not. Adopting the shipped shape saves the invention risk, not the strings. The honest number is roughly 24 new keys and one new plural noun across 15 locales — about 435 strings — plus a Rust IPC change.\n\nAnd the session's own register lost two of its five items. \"`report` can be null via the between-batches blocker\" is refuted by reading: with no delicate files there is no await between the two blocker checks, so that exit is unreachable and `report` is always batch 1's. The real Exit-2 screen is worse than the imagined empty one — a success headline with a green count over a run whose second half never happened. And the register's nine `ok:false` sites are ten; :507 was never in anyone's taxonomy until an attacker went and counted."
