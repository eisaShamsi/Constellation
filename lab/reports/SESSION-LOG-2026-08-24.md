# Session Log — 2026-08-24

Function in hand: **PJ-369 Step 2 — the write-free phantom count in the boot drift notice.**
(The sidebar/top-of-window notice band that PJ-207 §9 built; the sentence gated by `hasPhantoms`.)

---

## §1 — PJ-369 Step 2 built (NOT yet Boss-tested, NOT yet committed)

**Concept (the horse).** *A search result must correspond to a real, openable note.* Step 2
does not remove anything; it makes an invisible defect countable and says the number out loud,
so the removal in Step 4 is offered against a number the user has already seen.

**Landed in the working tree:**

- `src-tauri/src/reconcile.rs` — `stale_phantoms` field + `has_phantoms()`; classification runs
  inside the row loop the pass already walks (no second scan); its own diag line.
- `src-tauri/src/phantom_prune.rs` — provenance + cost documentation (see §2, §3).
- `src/lib/index/driftReport.ts` — `stalePhantoms` + `hasPhantoms()`, deliberately NOT folded
  into `hasFindings()`.
- `src/routes/+layout.svelte` — `indexPhantomMessage` derived + its own `.drift-note` row with
  **no "Repair now" button** (a repair walks libraries and re-reads files; a phantom has neither,
  so the button would be a door that does nothing — the "false door" the design attack named).
- All 15 locales — `plurals.entries` (each language's OWN CLDR categories) and
  `indexDrift.stalePhantoms`.

**Verification:** Rust `cargo test --lib` **1559 passed**; frontend `vitest` **1006 passed /
87 files**; `svelte-check` **0 errors**; `i18n-parity` **all 15 locales in parity**; frontend
built and the new string confirmed present in `build/`.

### Two defects in my own draft, caught before they shipped

1. **An invented Settings route.** The sentence said *"…remove them in Settings → Universe &
   Libraries → Index."* The Settings nav is a **flat** list (`SettingsModal.svelte:265`) and
   "Index" is a top-level section — that parent does not exist. Caught by verifying the route
   instead of recalling it (the "Never Describe the App Without Looking At It" law).
2. **A pointer to a control that does not exist yet.** The removal control lands in **Step 4**.
   The approved plan's sentence promised it at Step 2, which would have sent the Boss to an
   empty page — the same false-door failure the design forbids one paragraph earlier. Step 2
   now ships the honest half ("…they can show up as search results that open nothing, and as
   connections to notes that aren't there"); Step 4 appends the route when the door is real.
   **Deliberate, recorded deviation from the approved plan text.**

Also fixed in-pass: `tests/pj344/…` type error (`ReturnType<typeof get<…>>[number]` → the real
exported `OpenTab`), and the phantom read now uses `search::open_read_only_search_conn` instead
of a raw `Connection::open` — `SQLITE_OPEN_READ_ONLY` + `query_only=ON` make "this pass cannot
write" an invariant SQLite **enforces**, and its 5 s `busy_timeout` stops the count drifting
between boots under writer contention.

---

## §2 — A provenance error of mine, found by measuring, corrected in three files

Step 1's ground-truth audit ("Prune 603, 19,472 edges") was recorded in source as *"the Boss's
own **daily** universe."* **That is false**, and I would have sent his test to the wrong place.

Measured today, read-only, against both live databases:

| | note_meta | libraries | rows outside all live roots | file-gone | phantoms |
|---|---|---|---|---|---|
| **Eisa Cognitive Knowledge** (his DAILY universe) | 8,031 | 19 (all present) | **0** | 0 | **0** |
| **Eisa Universe** | 2,731 | 5 (all present) | 621 | 603 | **603** |

The daily universe is **structurally** zero: its own root is itself a registered library
(`universe_notes`, path == universe root), so no row beneath it can ever be "outside a library."
Eisa Universe's 621 decompose exactly as the Step-1 audit reported — 9 linked + 7 earned +
603 phantoms + 2 still-alive — which is why that audit is trusted; only its *location* was wrong.

Corrected in `phantom_prune.rs`, `reconcile.rs`, `driftReport.ts`. Baseline for the Boss test
(both universes, plus the exact 603 phantom paths and their sha256) captured at
`lab/reports/PJ-369-step2-baseline.txt`.

**How it was caught:** not by re-reading the comment, but by measuring the boot cost — the
replay reported "0 rows outside own roots", which could not be reconciled with a claimed 603
until both databases were queried. The measurement was for a different purpose and caught a
different bug.

---

## §3 — Boot cost, measured rather than assumed

The project's hard constraint is that no feature may regress boot time, and this pass adds
per-row work at boot. Measured on the live 312 MB `Eisa Universe` database, 621 candidates:

| approach | cold | warm |
|---|---|---|
| per-row indexed point lookups (**what ships**) | 11.6 s | **0.042 s** |
| three batched `DISTINCT` / `IS NOT NULL` scans (the "obvious" optimisation) | 38.8 s | — |

The cost is **cold random page reads, not query shape** — every query already rides an index
(`idx_link_boot`; two primary keys). Batching is ~3× *worse* because three full scans touch far
more pages than 621 lookups. On the daily universe the added cost is **0.000 s** (nothing to
classify), and the pass runs on a background thread (`reconcile::maybe_schedule` spawns and
returns), so it delays no paint. The table is now a comment in `has_earned_data` so the next
reader does not "optimise" it into the slower form.

---

## §4 — Whole-app safety sweep: 59 confirmed findings, 1 APP-KILLER **fixed**

Register: `lab/reports/safety-sweep-2026-08-24-whole-app.md`.

**Process failure, recorded:** I passed `args.files` as a string again, so the per-build
diff-scoped inspection fell back to the whole-app sweep — **the same mistake as 2026-08-23**,
which that day's own register already documents.

> **⚠️ The PJ-377 account in this section is SUPERSEDED by §9.** The panel proved this fix
> incomplete: it asked about the open TAB, while the net it protects is keyed to the PATH and
> outlives the tab. Read §9 and the register's CORRECTION section, not the paragraphs below,
> which are kept only to show what was believed at the time.

**APP-KILLER, fixed before commit → PJ-377.** `preserveWorkBeforeVacating`
(`store.ts:5291`) asked `isNoteDirty` alone and ignored `hasUnsavedRecovery`, so a model that is
clean yet holds write-ahead-recovered content read as "already durable" — and Delete, an
**ancestor-folder** delete, or an Overwrite then wiped the net, its localStorage backup, the
banner and the model. The trashed file was the pre-edit version; the paragraph existed nowhere;
nothing was surfaced.

The naive fix (adding the predicate to the flush list) was **rejected after checking**:
`flushIfDirty` returns `ok` *without writing* for a clean model, so it would have reported
durability having written nothing. The shipped fix keeps the net and returns `false`, which both
callers already honour. **Reproduced first**: `tests/pj-377/vacateKeepsRecoveryNet.test.ts` —
all three triggers go red pre-fix (`expected undefined to be 'the paragraph that never reached
disk'`), control stays green.

**58 remaining → PJ-378** for triage. Not ruled on yet: SO#10 requires the PCS and orientation
current first. Several should be fixed as *families* (Whole-Ecosystem Fix Law): the missing
`ensure_search_db_ready` group, the swallowed-write-error group, the YAML quote/escape group.

---

## §5 — Open, and the next decision point

- **PJ-369 Step 2 awaits the Boss's test** (tutorial through `tutorial-auditor` →
  `ui-inspector` → panel → Boss). Commit is gated on his pass — the standing order is that he
  tests every build **before** commit, so nothing here is committed yet.
- **~~One unresolved fact, stated as unresolved:~~ RESOLVED 2026-08-25 — see §21. The paragraph
  below is WRONG and is kept only because it was on the record.** It reads:
  *"the live registry (`%APPDATA%\world.uconstellation.app\universes.json`, identifier confirmed
  at `tauri.conf.json:5`) lists one universe — `كون عيسى` — and was last written 2026-08-07, yet
  both other universes' databases were written today and today's write journal names all three.
  The file and the observed behaviour disagree and I could not reconcile them."*
  **It is not the live registry.** That path, read from this sandbox, resolves to a frozen copy
  inside the Claude Desktop MSIX container (`…\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\…`).
  The app's own registry was never observed. There was no disagreement to reconcile — the file and
  the behaviour describe two different files. The secondary claim in the same paragraph is also
  overstated: `write-journal.jsonl` names three universes because the app has written notes in
  three universes since 2026-06-08, not because three were active that day.
- Queue after Step 2: Steps 3–5, then PJ-375 probe repair, then PJ-367 → PJ-366 → PJ-360
  (federation), then MIG-111 B2/B7.

---

## §6 — PJ-377: the Whole-Ecosystem check, done rather than assumed

The Whole-Ecosystem Fix Law says a fix is not done until every surface sharing the concern is
consistent. The concern here is **"destroying a note's recovery state"**, so I enumerated every
site that does it rather than trusting that the sweep found them all:

- `clearPathKeyedAuxStateOnDelete` — two callers (`store.ts:5206` deleteWithSetting, `:5419`
  moveToTrash). Both gated on `preserveWorkBeforeVacating`, which is what was fixed.
- `clearWriteAhead` — ten call sites. The two that could plausibly destroy recovered work are
  both already correct: `:3736` clears only when `adopted` is true, and `externalChangeNoteModel`
  explicitly refuses on a *"dirty, echo, or recovered-work baseline"*; `:5047` clears only when
  `adoptedFreshDisk`, under a comment stating that a tab which kept the user's model keeps its
  net. `:3217` is `if (!opts.preserveNet)`, the restore path's own switch.
- The neighbouring `isNoteDirty` sites (`:1558`, `:1785`, `:1942`) are *should-I-write* /
  *should-I-refresh* decisions, not *is-it-safe-to-destroy* decisions, and their reload paths
  run through `:1169` / `:1374`, both of which already ask both questions.

**Conclusion: `preserveWorkBeforeVacating` was the only site asking the half-question.** The fix
is complete for its concern — established by enumeration, not by assuming the sweep was exhaustive.

## §7 — Test material: REJECTED twice, and why that was worth it

The Boss-facing tutorial has not reached him. It has been rejected twice, both times for a defect
that would have wasted a full round-trip:

1. **Round 1** assumed the 603 phantoms were in his daily universe. They are not (§2). He would
   have seen nothing and reported a working feature as broken.
2. **Round 2** told him to land in his daily universe and switch via the Universe Manager. The
   `ui-inspector` traced the list to `list_universes` → the registry file, which holds ONE entry
   (`كون عيسى`) — and found the **same registry drives boot-time selection**
   (`+layout.svelte:3576-3593`), so even "you will land in Eisa Cognitive Knowledge" is
   unsupportable from source. It also noted the Switch button is not rendered for the active
   entry, so a one-entry list offers no Switch at all.

Source and observation genuinely disagree here, and that disagreement **is PJ-321** (five
corroborations, STOP order: observe, do not diagnose). Rather than diagnose it, round 3 is built
to be *independent* of it: I measured the expected phantom count for **every** universe on disk,
so the Boss's observation is self-checking whichever one opens.

| universe | notes | libraries | phantoms |
|---|---|---|---|
| Constellation Test | 5,161 | 3/3 | **1,200** |
| Eisa Universe | 2,731 | 5/5 | **603** |
| Eisa Cognitive Knowledge | 8,031 | 19/19 | 0 |
| MIG108 Rehearsal | 7,827 | 20/20 | 0 |
| Review Demo · Scratch · جوامع عيسى الشامسي · كون عيسى | 3 · 30 · 6 · 6 | — | 0 |

Note that `كون عيسى` — the one universe the registry actually lists as active — has **zero**
phantoms, so if the app opens there, silence is correct and proves nothing about the positive
case. Two universes carry real loads (1,200 and 603), which is the defect PJ-369 exists to expose.

---

## §8 — Final verification on the shipping tree, and one flaky test worth naming

Run against the exact tree that will be committed, after the release binary was built:

- Rust `cargo test --lib` — **1559 passed**, 0 failed.
- Frontend `vitest` — **1006 passed / 87 files**, 0 failed (clean run).
- `i18n-parity` — **all 15 locales in parity**.
- `svelte-check` — **0 errors**.
- Release binary rebuilt **13:22:22**, postdating `build/` (**12:59:00**), which contains both the
  phantom sentence and the PJ-377 fix. The Rust diag line is greppable in the exe and the
  superseded "offered for removal in Settings → Index" wording is provably **absent** from it.
  (The frontend strings are not greppable in the binary because Tauri compresses embedded assets —
  verified via `build/` + build ordering instead of asserting a grep that cannot work.)

**One test is load-sensitive, not broken.** `tests/sight-v6/perf.test.ts` — *"Hearst facet-count
rebalancing on 7,636 notes completes in ≤32 ms"* — failed once while a `cargo build --release` was
saturating the CPU, then passed in isolation (35 ms of test time) and passed again on a clean full
run. It is a wall-clock budget assertion with no relation to anything changed today (the diff
touches reconcile, the phantom classifier, the drift band, the vacate path and i18n). Recorded
rather than waved away: a suite that can go red because another process is busy will eventually
cost someone an afternoon chasing a regression that never existed. Filed as **PJ-380** — budget
tests should measure work, not wall clock, or be quarantined from the default run.

---

## §9 — The panel overturned my "done", and it was right

The adversarial panel (Panel-Speaks-First law) reviewed the build before any of it reached the
Boss. It did not rubber-stamp: it found a **blocker in Step 2** and proved my **PJ-377 fix
incomplete**. Both are now fixed. What it caught, in order of severity:

### Step 2 — BLOCKER: the software was about to make my own wrong-universe error

`reconcile.rs` returned `report: Some(report)` at **two** early exits with no `still_ours()`
check, while the write-phase gate below them refuses to publish a departed universe's numbers in
terms — *"the same cross-universe contamination §8 exists to prevent, in the notice instead of
the index."* Worse, the clean-drift exit is the path a universe with no stale/orphan rows takes on
**every boot**. Concretely: the pass starts in a universe with phantoms, the user switches while
it is still running, and the **clean** universe displays the departed one's count.

That is precisely the mistake I made on paper this week (recording the 603 against his daily
universe), about to be made by the software, in front of the person who caught it. Both exits now
carry the gate, and a third check was added immediately before the emit — because a switch can
also land between `run()` returning and the event firing, and the emit is what the user sees.

### Step 2 — MAJOR: the fail-closed guard failed OPEN

`ClassifierCtx::build` read `.ok()` … `.unwrap_or(0)`, collapsing **"I could not read the
federation"** into **"there is no federation"** — disarming the Attack-1 guard in the exact case
it exists for. Harmless while the step only counts; at Step 3/4 it deletes another universe's
index rows. Now refuses on either failure. *An unknown federation is not an absent one.*

Also: a refused run reported `0` and rendered identically to a clean universe. Undecided rows are
now counted and logged (`phantom classification INCOMPLETE … the count is a floor, not a total`),
so "we could not tell" is distinguishable from "there is nothing to tell". The user-facing form is
owed with Step 4's control rather than invented now.

### Step 2 — MINOR: one ✕ dismissed both sentences

A single `indexDriftDismissed` gated both notice rows and both ✕ buttons, so clearing the
repairable-drift notice silently erased the phantom count — this step's entire deliverable —
without the user ever reading it. Separate `indexPhantomDismissed` added, cleared at both reset
sites. The sentence also gained a disposition ("Nothing has been changed — they have only been
counted"), across 15 locales: naming no button is right, leaving a bare fact is not.

### PJ-377 — my fix was incomplete; see the register for the full correction

Summary: it asked about the **tab** when the net is keyed to the **path** and outlives the tab
(`closeTab` deliberately keeps both net and banner). The predicate now reads the net directly.
Returning a bare `false` also leaked every sibling's aux state under a deleted folder — fixed with
a `keep` action so the cleanup is per-key. `netUnsaved` turned out to be set at **two** sites, the
second in a live session with no restart, so the exposure was wider than I recorded. One test
named a trigger it did not run. And **Overwrite-on-collision cannot be fixed by any predicate** —
the vacated path is re-occupied in the same click — so it goes to the Boss as a question.

### PJ-380 — fixed rather than tolerated

The flaky perf budget failed **3 of 5** full-suite runs. Timing one cold call measures JIT warm-up
and whatever GC lands in it. All four budgets in that file now take the **best of five** runs — the
standard estimator, since noise can only add time while a real regression slows even the fastest
run. Three consecutive clean full-suite runs afterwards (1008 passing each).

### The guard against the inspection mistake recurring

The per-build inspection had **never actually run** for this build: passing `args.files` as a
string silently degraded to a whole-app sweep, and I did it twice, the second time after the first
was written up in that day's own report. That silent degrade is itself a false success — the class
this workflow hunts. `.claude/workflows/safety-inspection.js` now **throws** on a malformed `args`
and logs its resolved mode. Verified behaviourally: the string form is refused with a message
naming the mistake; no-args (whole-app) and a proper array both still work. The properly-scoped
diff inspection was then launched for real.

---

## §10 — The diff-scoped inspection ran properly and found a bug in the fix from §9

With the argument guard in place the per-build inspection finally ran as intended (`mode: "diff"`,
five files). One confirmed finding, **in code I had written and called fixed two hours earlier**:

`phantom_prune.rs` — the Attack-1 federation guard refused only when the linked-universe set was
**completely** empty, while its own field doc promises *"If ANY linked universe fails to resolve …
`refused` is set."* The gap is reachable because the guard's two inputs disagree about a missing
child: the strict resolver **keeps** a `NotFound` child, `resolve_libraries_recursive` **skips** it.
Two Linked Universes, one renamed in Explorer between sessions → non-empty set → guard silent →
every parent-index row under the renamed child's old path classifies as a phantom, though those
notes exist.

This is not hypothetical for the Boss: **`Eisa Universe` declares two children** — `كون عيسى` and
`Eisa Cognitive Knowledge`. Both resolve today, so the 603 stands; but rename either folder and the
count would have started counting real notes as phantoms.

Fixed in-pass (WA#6) rather than left for Step 3/4, where the same verdict feeds
`reindex_delete_note` and the wrong answer becomes a deletion of a Linked Universe's index rows —
a write-sovereignty violation. The decision is now the pure `federation_is_complete(declared,
linked)`, extracted free of `AppHandle` on the `foreign_roots_of` precedent so the test exercises
the real function. Four new tests; two go red against the old semantics.

**Why no existing test caught it:** `attack1b` passed throughout. It proves a *refused* context
yields `Unknown` — the **consumption** of a refusal, never the **decision** to refuse. A guard
needs a test on the predicate that arms it, not only on what it triggers.

Two doc drifts in the same module fixed while there: condition 4 still described "any
`review_schedule` row" (the exact policy whose correction this morning took the audit from
`Prune: 0` to `Prune: 603`), and an earlier insertion of mine had orphaned a sentence.

**Verification:** Rust **1563 passed** (+4), frontend **1008 passed**, `svelte-check` **0 errors**,
15 locales in parity.

---

## §11 — Standing at the end of this block

**Not committed.** The Boss tests every build before commit; nothing here has been committed.

**Approved and ready to send:** the Step-2 test tutorial — `tutorial-auditor` → `ui-inspector`
**APPROVED** after **27 cumulative claims checked across six rounds and eight findings**, including
two invented UI claims, a thousands separator the app never prints, an unfollowable universe-switch
step, a promise scoped too broadly, and a failure threshold I set *below* the legitimate worst case.

**Owed to the Boss as questions, not decisions:**
- **PJ-381** — Overwrite-on-collision can still discard unsaved work and cannot be fixed by a
  path-keyed predicate. Two real remedies, both product calls.
- **PJ-378** — the 58 remaining sweep findings, for ranking.

**Fixed today, pending his test:** PJ-377 (the app-killer, twice-reworked), PJ-369 Step 2 (with the
panel's blocker and two majors), PJ-380 (flaky perf budget), and the inspection-argument guard.

---

## §12 — The Boss's test FAILED Step 2, and the cause was my own fix from 30 minutes earlier

**His result:** Step 1 (daily universe) — no strip, correct. Step 2 (`Eisa Universe`) — **no strip
at all**, expected "603 entries…". Step 3 — no strip, correct.

**Diagnosed from the app's own log in one read** (`Eisa Universe/.constellation/diagnostics.log`):

```
[reconcile] phantom classification INCOMPLETE — 612 row(s) undecided; run refused:
the federation resolved only partially … The phantom count below is a floor, not a total.
[reconcile] drift check: … 9 from a linked universe (2110 files / 2731 rows seen, walk complete)
```

The 612 + 9 decomposition confirms the model exactly: 621 outside-root rows = 9 linked (counted
separately) + 612 offered to the classifier = 603 phantoms + 7 earned + 2 still-alive.

**Root cause — mine, introduced by the PJ-382 fix ~30 minutes before he tested.** The Attack-1
guard compares declared child roots against resolved linked-library roots.
`resolve_child_universe_roots_recursive_strict` returns **canonicalised** paths, and on Windows
`fs::canonicalize` yields the VERBATIM form. Probed on this machine:

```
canonicalize("E:\Constellation Universes\Eisa Universe")
  = "\?\E:\Constellation Universes\Eisa Universe"
norm(that)                = "//?/e:/constellation universes/eisa universe"
norm(plain library path)  =    "e:/constellation universes/eisa universe"
```

Those can never compare equal, so **every** declared child looked unresolved, the guard refused
every run, and a working feature became permanent silence. Before PJ-382 the old `is_empty()`
test happened to pass here (the set was non-empty), so the fix for a MED finding broke the feature
outright.

**Fix:** strip the verbatim prefix inside `norm` — a no-op for the plain paths that dominate, so
it cannot disturb them, and it inoculates every future comparison rather than one call site.

**Verified against his real data before asking him to retest** (not just unit tests): replaying
both sides from `universe.json` + each child's `libraries.json` gives
`federation_is_complete -> True`, both children contributing, 20 linked roots resolved.

**Why the tests missed it, and the rule that follows.** `attack1c`–`attack1f` were correct and
passed throughout: they fed `federation_is_complete` **pre-normalised literals**, exercising the
LOGIC while never exercising the INPUT FORM the caller actually supplies. This is the same shape as
`attack1b` testing the *consumption* of a refusal rather than the *decision* to refuse — twice in
one day, in one module.

> **A pure function extracted for testability inherits none of its caller's input forms. Test it
> with a value produced the way the caller produces it, or the test only proves the arithmetic.**

`attack1g_a_canonicalised_declared_root_still_matches_a_plain_linked_root` now builds the declared
side with a real `fs::canonicalize` on a real temp directory. It goes red without the strip.

**Standing correction to my own account:** §10 said PJ-382 was "fixed in-pass". It was fixed and
simultaneously broke the feature. Both statements are now on the record.

---

## §13 — The second failure: my diagnosis of §12 was itself wrong

The Boss re-tested on the new binary (his run at 17:13:15, built 17:06:45 — verified, he ran the
right one). **Still no strip.** Same log line: `run refused: the federation resolved only partially`.

So §12's verbatim-prefix diagnosis was real but **not the cause**. I had verified the fix against
"his real data" by replaying `universe.json` in Python — and that replay read only `Eisa Universe`'s
OWN two declared children. **The Rust resolver is RECURSIVE.** Probing the actual function found a
THIRD declared child:

```
Contributed       <- e:/constellation universes/eisa cognitive knowledge
Contributed       <- e:/constellation universes/كون عيسى
NOT canonicalised <- e:/constellation universes/two universe universe/two universe universe
```

`كون عيسى` still declares a grandchild universe whose folder was deleted long ago. `Eisa Universe`
federates `كون عيسى`, so it inherits that dead grandchild — and my guard refused every run because
of it. **The guard was working exactly as written; the rule it implemented was wrong.**

I had even recorded this fact myself: `lab/reports/PJ-369-step2-baseline.txt` notes that
`كون عيسى` "declares 1 child that DOES NOT EXIST → REFUSAL FIRES". I wrote it down and did not
connect it to the universe that federates it.

**The design fix.** Refusing exists to stop a linked universe's notes being called phantoms. A
folder that does not exist can hide no note, so it cannot make that check vacuous — refusing on its
account disables the feature permanently for anyone who ever deleted a linked universe's folder.
The rule is now three-way (`declared_child_status`):

| child | meaning | action |
|---|---|---|
| **Contributed** | resolved ≥1 library root | proceed |
| **AbsentButTrusted** | folder gone, nearest existing ancestor readable | proceed — **and its declared path joins the protected set**, so any row still pointing under it is never pruned |
| **Unresolved** | present but yielded nothing, or absence untrustworthy (possible unmounted drive) | refuse the whole run |

Attack-1 is preserved exactly: rows under a dead child are still protected. The refusal is now
reserved for genuine ambiguity.

**Verified against his real federation by calling the REAL resolver** — not a Python re-implementation
of it: the three children come back Contributed / Contributed / AbsentButTrusted, `unresolved=0`,
so the run proceeds. That is the check §12 should have been.

**The lesson, and it is the sharper one:** §12's "verified against his real data" was a
**re-implementation** of the production logic in another language. It agreed with my hypothesis
because it shared my misunderstanding — it read one manifest where the real code walks a tree. A
verification that re-implements the thing it verifies can only confirm what you already believe.
*Call the real function.*

Two tests added, both from this failure: `attack1h` (a dead grandchild must not refuse) and
`attack1i` (a child that EXISTS but contributed nothing still refuses — the ambiguity that must
survive). Two older tests were renamed: `attack1c`'s name claimed it proved a missing child refuses
the run, which is no longer true and was never what it tested — the same "a test named something it
does not do" defect I flagged in someone else's work this morning.

**Rust suite: 1566 passed.**

---

## §14 — A build that reported success and produced nothing

After §13's fix, `cargo build --release` was run in the background and the task reported
**exit code 0**. The binary on disk was unchanged: still 17:06:45, while the fixed source was
17:27:17. Caught by checking the timestamps rather than trusting the exit code.

The real output:

```
Access is denied. (os error 5)
warning: build failed, waiting for other jobs to finish...
```

Constellation was still running (`constellation.exe`, PID 109632) and holding the executable, so
the link could not write it. **The command was `cargo build --release 2>&1 | tail -2`, and a pipe
makes the shell report the exit status of `tail`, not of `cargo`.** So a hard build failure was
reported as success.

That is the same defect class this session has now hit three times in different clothes: a signal
that degrades silently into a false success — the malformed inspection argument that became a
whole-app sweep, the refusal that rendered identically to a clean universe, and now a failed build
that exits 0. Each time the failure was invisible precisely because the surface looked normal.

Two guards, since "remember to check" is not one:
1. Never pipe a build command whose success matters — capture the output and test cargo's own
   status, or run it unpiped and read the tail afterwards.
2. Before ANY Boss test, assert the binary's mtime is NEWER than every source file in the diff.
   That check is what caught this, and it is the standing "verify the binary before testing" rule
   ([[feedback-verify-binary-before-testing]]) doing its job — this time on my own build rather
   than on a stale install.

Nothing was sent to the Boss on the strength of that build. He was told to close the app, and the
rebuild is pending.

---

## §15 — BOSS-VALIDATED, and the write-free promise proven

**His fourth run passed.** `Eisa Universe` shows the sentence with **603 entries** and a ✕ as its
only control. Step 1 (daily universe silent) and Step 3 (count follows the universe) passed
earlier. He observed it on a plain boot rather than a switch — the count runs on any universe
activation, so that is the same test.

**The write-free promise, proven rather than asserted** (re-queried after his run):

| | before | after | |
|---|---|---|---|
| `note_meta` | 2,731 | 2,731 | identical |
| `note_links` | 31,368 | 31,368 | identical |
| `review_schedule` | 2,731 | 2,731 | identical |

All **603** captured phantom paths still present, sha256 `fa1cac7ca107499f5ac8f66b1ad9526d`
unchanged. Row totals alone would not have proven this — the pre-existing reconcile legitimately
heals and can remove rows — which is why the exact path set was captured before the test.

**Committed** (split so the app-killer has its own revert handle, per the panel):
- `66922086` PJ-377 — the delete/recovery-net app-killer
- `8bde0169` PJ-380 + the inspection-argument guard
- `234a53a7` PJ-369 Step 2 — BOSS-VALIDATED

**Four attempts, three failures, every one caught by his test:**
1. Refused because a MED-finding fix compared canonicalised against plain paths.
2. Refused again because the "verification" of fix 1 was a Python re-implementation that shared my
   misunderstanding — it read one manifest where the real code walks a tree.
3. No new binary at all: `cargo build --release | tail -2` reported the exit code of `tail`, so a
   link failure (the app held the .exe) came back as success.
4. Passed.

The through-line of all three is one defect class — **a signal degrading silently into a false
success** — which is also two of the three things committed here. The lesson that generalises
beyond this feature: *verify by calling the real function on the real data; a re-implementation
can only confirm what you already believe.*

---

## §16 — PJ-369 Step 3: the prune executor, proven on a copy of the live database

**Function in hand:** the backend command that removes confirmed phantom rows through the single
delete funnel, archive-first, with no UI caller until Step 4.

`phantom_prune_run` → `prune_stale_phantoms` (classify) → `prune_confirmed` (act). The loop was
deliberately split out **free of `AppHandle`** so the harness runs the SHIPPING function rather
than a copy of it — the direct lesson of §12/§13, where two "verifications" that re-implemented
production logic agreed with a wrong hypothesis because they shared its misunderstanding.

Guards, and why each is where it is:
- **Universe check before EVERY delete**, not once per run — a sweep is hundreds of deletes, and
  one landing after a switch destroys a row that was never ours (`reconcile.rs`'s precedent).
- **Re-stat immediately before each delete** — a drive can come back between classify and act; a
  reappeared file is a real note again.
- **No safety cap.** Every other bulk path aborts above a threshold; here the human confirm IS the
  ceiling, so a silent partial abort would be a different operation than the one approved.
- **Archive-first inherited, not re-implemented** — the funnel refuses and purges nothing if the
  archive cannot be written (invariant 9: one funnel, no hand-rolled bulk DELETE).

### The verification clause, met in full — against a COPY of the live `Eisa Universe`

| check | result |
|---|---|
| candidates classified | **603** |
| removed / failed | **603 / 0** |
| path-bearing tables purged | **12 of 12**, incl. `sky_nodes` + `sky_links` |
| archive lines written | **603** ("archive all", the Boss's ruling) |
| rows after | 2,731 → **2,128** — exactly 603, no more |
| second run | **0 candidates, 0 removed** (idempotent) |
| switch mid-run | stopped after 3, `stopped_early` set, counts honest |

The 2,128 survivors include the 9 linked-universe rows and the 7 carrying earned work.

**The harness caught two defects, both mine, both in the harness:** it opened a raw connection
without the app's custom FTS5 tokenizer (all 603 deletes failed with "no such tokenizer"), and its
idempotency case replayed a hard-coded list where the command re-derives from `note_meta`. The
second *looked* like a product defect — a second run reporting "removed: 603" — and was a test
defect. It surfaced only because the harness goes through the real funnel.

### The diff inspection found one thing, and the two agents disagreed by reading different universes

`build_delete_archive` returns an empty archive when `cid_cn` is empty; Phase 2 is gated on
`!archive.is_empty()`, so the archive-first contract is skipped **silently** and Phase 3 purges
anyway, returning Ok. The hunter measured 234/2,731 empty-cid rows; the verifier called that
"wrong" and cited 25/8,031 — **both are right, for different universes**, and the verifier checked
the daily one rather than the one under test. That is the identical which-universe error I made on
paper this morning, made this time by a verifying agent.

Measured directly: **0 of the 603 candidates have an empty cid**, so the phantom path is latent.
Guarded anyway, because the overlap is one lost file away and this module's law is FAIL CLOSED —
the executor now refuses such a row with a reason a receipt can carry, and the skip notice moved
from `eprintln!` (invisible in a release Windows GUI build) to `diagnostics.log`, which helps the
two live funnels that reach the same line. The root fix — keying the archive on the
universe-relative path — changes SHARED delete semantics and is filed as **PJ-384** rather than
smuggled into this step.

**Rust suite: 1566 passed** (3 live harness tests `#[ignore]`d by default; run with
`cargo test --lib phantom_prune::live -- --ignored`).

**Not Boss-tested and not shippable to him yet by design:** Step 3 has no UI. Step 4 adds the
Settings control, the danger-confirm and the receipt, and that is the next Boss test gate.

---

## §17 — PJ-369 Step 4: the door, and three defects the gates caught in it

**Function in hand:** the Settings → Index control that removes the phantoms Step 2 counts.

**Built:** "Remove stale index entries", rendered only at `phantomCount > 0` (an always-visible
control for a problem the user does not have invites him to go looking for one, and "Remove 0
entries" is a button that cannot act). A `danger: true` confirm quoting the count — consent to a
number, not to a verb. A receipt with a separate honest line per outcome, because `removed` alone
would flatter a partial run. Strings in all 15 locales, flat with a `{noun}` param so the plural
agreement lives in `plurals.entries` alone rather than being duplicated across six categories in
fifteen files.

**Step 2's deferred promise kept in the same commit that earns it:** the notice sentence's tail
changed from "they have only been counted" to "you can remove them in Settings → Index" — never
before the door existed.

### Three defects, none of them found by me

1. **Safety inspection (MED).** `RepairState.drift` was written by the boot pass and cleared
   NOWHERE, so after a universe switch `index_drift_report` kept answering with the DEPARTED
   universe's numbers — and Settings reads that command directly, re-importing the value the
   layout had just discarded on switch. Result: "Remove stale index entries — 603 entries" in a
   universe with none, and a danger dialog asking consent for that number. No wrong deletion was
   possible (the executor re-derives from the current universe and the classifier fails closed),
   so the harm was a false statement of state inside a destructive consent. Fixed with
   `forget_drift_report`, called from `invalidate_search_state` — the ONE fence every switch
   passes through, so no future switch path can miss it.

2. **Tutorial-auditor (product defect, not a tutorial problem).** It traced the refresh path
   instead of assuming, and found that after a successful removal BOTH surfaces still said "603"
   until a restart — the count comes from a cached report only `reconcile::maybe_schedule` writes.
   Telling the user "603 entries" immediately after removing 603 is the false-success shape at the
   worst possible moment. It had honestly built the test around the quirk; the right answer was to
   fix the quirk. `update_drift_phantom_count` now patches ONLY `stale_phantoms` and re-emits the
   drift event; `SettingsModal` subscribes to it so an open modal is no longer frozen at mount.
   The residue is **derived** (`candidates - removed`), so a run that stopped early or skipped rows
   reports its real remainder instead of claiming zero.

3. **ui-inspector (interaction hazard).** `Enter` instantly confirmed the dialog — a held key or a
   return pressed at an unread dialog would have removed 603 entries. **My first fix was too
   broad:** I keyed it to `danger`, which would also have taken Enter from the everyday
   note-delete confirm — an unasked-for change to a daily flow, and a note delete is recoverable
   from the trash. Narrowed to an explicit `enterConfirms` opt-out used only where there is no way
   back. Escape still cancels everywhere; only the unsafe direction now costs a deliberate click.

### What the inspector established that I had only hedged

The tutorial said "I have not verified there is a screen that shows you the archived copy". The
inspector checked and can state it as fact: `read_history_for` — the only reader of
`note-history.jsonl` — has **zero callers outside its own tests**, and the two commands that expose
note history query the live `note_state_history` table, which the removal deletes. **No shipped code
path can read that archive back.** The archive is a durable record, not a restore.

It also re-ran the live harness independently against a copy of the Boss's database and reproduced
603 removed / 0 failed / 603 archived / idempotent — a second pair of hands on the same claim.

**Verification:** Rust 1566, frontend 1008, `svelte-check` 0 errors, 15 locales in parity, binary
20:30:52 newer than all 22 changed sources, both build exit codes captured unpiped.

**Not committed.** The Boss tests before commit. The panel is the last gate.

---

## §18 — PJ-385: the delete-archive reader (Boss-ruled, 2026-08-25)

**His ruling, verbatim:** *"First, build a way to read that archive back."* Asked whether to
proceed with the 603-row removal or build the reader first, he chose the reader. The 603 are
untouched.

**Concept (the horse).** *When Constellation destroys something permanently, the person must be
able to see what it destroyed.* Every delete already wrote an envelope before anything was purged
and refused to purge if that write failed — a guarantee that was true and useless in the same
breath, because the only reader took a content id the caller had to already know and returned only
the change-events. The app could say "its history was kept" while offering no way to look.

**Built:** `Settings → Universe → Deleted notes`. Every removal this universe has recorded —
trash, permanent, a file vanished outside the app, a startup cleanup, an index prune — newest
first, with what/when/where/why, how much text was kept, and how many changes came with it. Click
a row for the archived text. Reads a file, opens no database, writes nothing. **Read-only: it is a
record, not an undo.**

Placed under **Universe**, not Index, because the archive covers every deletion in the universe
while Index is about vocabulary and search — filing a record of destroyed notes under a heading
about words would have been the wrong shelf.

### What it immediately showed about his real data

> **⚠️ WITHDRAWN — see §19.** The framing below ("what an unreadable archive was hiding") was a
> manufactured alarm and is false. Those entries are FRONTMATTER-ONLY notes; nothing was hidden.
> Kept unedited to show what was believed at the time.

5 deletions in `Eisa Universe`, 8 in `Eisa Cognitive Knowledge`, zero unreadable lines — and
**several entries that kept 0 characters of text**. The record exists; the body does not. That is
what an unreadable archive was hiding, and it is the argument for having built this before the
removal rather than after.

### Nine inspection findings, all in code written the same hour, all mine

Two were serious, and both told the user something FALSE about the last surviving copy of a
destroyed note:

- **A stale-result race (HIGH).** Clicking row A then row B, with A resolving last, painted A's
  text under B's heading — as a settled answer, no error, no cue. On an archive that is the last
  copy, that is presenting one destroyed note's content as another's. Now the resolved cid is
  compared against the still-open row and a late answer is discarded.
- **Wrong-envelope addressing (MED).** One note can have several deletion envelopes (a sync agent
  removing and re-adding a file archives a `vanished` envelope; the note may be deleted again
  later). Addressed by cid alone, every row for that note expanded together and all showed the
  NEWEST envelope's text — a row could advertise "12,000 characters kept" and then display none,
  or display a different deletion's text as what was destroyed. Now addressed by `(cid, at)` end
  to end, and change-events are attributed by FILE ORDER to the deletion they were appended with.

The other seven are the same shape in smaller places: a failed body read rendering as "no text was
kept"; a failed list load rendering as "not loaded yet"; a classifier refusal rendering as
`Ok(0)` — which made the removal control vanish and read as all-clear; the archive read skipping
the ledger lock every other reader in that module takes; `limit` bounding the payload but not the
work (documented rather than hidden); and the modal's close guard covering one of three paths.

### The one worth naming, because it is mine and it repeated

**My test was a copy of the parser.** When the real one was fixed, the copy kept asserting the old
behaviour and the test failed against correct code. That is the identical defect this session has
now hit four times — a verification that does not enter through the shipping entry point. There is
now ONE `parse_archive`, called by the command and by every test.

**Verification:** Rust **1571 passed** (5 new), frontend **1008**, `svelte-check` **0 errors**,
15 locales in parity, and the live test reads both real archives cleanly.

**Also fixed:** `tests/sight-v6/tradition-perf.test.ts` — the sibling of the perf file fixed
yesterday, left behind then and red today on a change that touched nothing near it. Same
best-of-five estimator. Three consecutive clean full-suite runs.

---

## §19 — The panel overturned the premise the reader was announced on

**DO_NOT_SEND as a viewer test.** Five changes, two of them blocking, plus a correction to my own
framing that would have put a manufactured alarm in front of the Boss.

### The framing I got wrong

I told him: *"several entries kept 0 characters — the record exists, the body doesn't; that's what
the unreadable archive was hiding."* Measured, that is not a finding at all. Those notes are
**frontmatter-only** — 101 of 2,731 rows hold no `body_text` while their files sit on disk with
real content, and the one I opened is four lines of properties and nothing else. Nothing hidden.

Worse, I had generalised it into a claim about the 603, in code comments, the TS mirror and the
user-facing explanation: *a phantom has no text "because its file was already gone when it was
indexed"*. **Backwards.** Measured: **601 of the 603 carry body text — 20,484,230 characters,
median 18,944.** The prune would archive ~20 MB of real text.

I had already caught and fixed the user-facing half of this myself, an hour before the panel
reported, by asking *why* the text was missing and reading `search.rs:12836` — the body comes from
`note_meta.body_text`, the INDEX, not the file. The panel caught the rest: the same false premise
still sitting in three code comments and the TS doc.

### The blocker that matters most

**The prune's consent sentence still said "Constellation cannot read it back" — in all 15 locales
— shipping in the same tree as the reader that refutes it.** That is the exact falsehood that made
him order the reader. Two documents said it too: `User Manual.md:317` ("There is no screen that
shows it to you") and the Index help topic. All corrected; the consent sentence now points at
Settings → Universe & Libraries → Deleted notes.

Also corrected: `settings.deleted.intro` claimed a record of **every** note removed, and a blanket
"removes nothing if it cannot be written". Neither is true — 234 rows have no content id and leave
no entry at all, and a permanent delete destroys the file before the archive write.

### What the panel established about the 603, independently

- All 603 paths are under `E:\Cognitive Knowledge\…` — the OLD address of a collection that now
  lives in the Linked Universe `Eisa Cognitive Knowledge`. Leftovers from a move, not deletions.
- **597 have a live twin** matched by permanent id; 590 identical length, **7 where the surviving
  copy is LONGER, none shorter.**
- **6 orphans, 134 characters** (I had said 5 — the missed one contributes 0 characters, so my
  total matched anyway; an error invisible in its own arithmetic).
- All 603 carry a content id, so all 603 will archive.

### PJ-384 escalated

**234 real notes, all files on disk, would be purged with no archive entry at all** — invisible in
the very panel just built. None of the 603 are affected. Boss ruling owed: close it before the
prune, or file it separately.

### The lesson

Two of three panel lenses made a BLOCKER of a string that was **not in the tree** — they quoted the
`noTextStored` text I had already corrected mid-run. Their headline finding was against stale text;
only the lens that grepped caught it. Adversarial review is not immune to the defect it hunts, and
the synthesis catching its own lenses is what made the output usable.

---

## §20 — The orphan count: the panel was right, I was wrong, and the error was invisible

I reported **5 true orphans, 134 characters**. The panel said **6**. Re-measured: **the panel is
right.**

There are TWO `Collision Test.md` paths among the 603 — one in `Eisa Test\.trash\` with **no twin**
(a genuine orphan, 0 characters) and one live at `Eisa Test\Collision Test.md` (38 characters, twin
present). My script resolved each phantom by content id and then **fell back to matching on
filename**. For the `.trash` orphan that fallback found the LIVE note of the same basename in the
daily universe and scored it "alive".

**The total still came to 134 characters**, because the missed orphan contributes zero — so the
number I checked against agreed perfectly with the number I had wrong. An error cannot hide better
than that: a flawed method, a plausible result, and an internal cross-check that confirms it.

Corrected count: **6 orphans, 134 characters.** The conclusion is unchanged — the stakes are still
134 characters of test scraps — but the count I gave the Boss was wrong and is now right.

This is the third distinct failure of the same shape today (a Python re-implementation that shared
my misunderstanding; a test that was a copy of its parser; a filename fallback that silently matched
the wrong note), and it is what motivated the Boss's instruction to build a standing verifier.

---

## §21 — The registry was never stale. I was reading a shadow copy, and so was PJ-321.

*(Written 2026-08-25. Resolves the "one unresolved fact" above, and closes a Group-1 ledger entry
that had accumulated five corroborations.)*

**The finding.** `%APPDATA%\world.uconstellation.app\universes.json`, read from inside this
session's sandbox, is **not the file Constellation reads and writes**. It is a stale copy held by
the Claude Desktop MSIX container:

```
fsutil hardlink list "C:\Users\ealsh\AppData\Roaming\world.uconstellation.app\universes.json"
  \Users\ealsh\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\world.uconstellation.app\universes.json
```

Two methods, either of which could have disagreed:

1. `fsutil hardlink list` on all four files in that directory. Only `universes.json` resolves into
   the container's `LocalCache`. The three siblings (`write-journal.jsonl`, `app-prefs.json`,
   `style-presets.json`) return `Error 50: The request is not supported` — the signature of a
   handle served by the virtualization filter, i.e. **pass-through to the real location**. A
   wholly container-local directory would have answered the same way for all four. It did not.
2. Directory divergence. The container path holds **exactly one file**; the merged view shows five
   entries including a 1.1 MB `write-journal.jsonl` written 2026-08-24 17:12. The container
   shadows one file and lets the rest fall through.

**Every observation PJ-321 rested on is explained by this, and no other explanation is needed.**
The file never changed under registry writes because a snapshot does not change. The app listed
nine universes while "the file" held one because the app was reading the real file. The Boss's
controlled experiment — creating two universes through the Universe Manager and watching the file
stay byte-identical — measured the snapshot, not the registry.

**The app is proven correct, from its own trace.** `Eisa Universe`'s `boot-perf.latest.json` holds
the process-lifetime IPC log for the 2026-08-24 boot: `load_app_prefs` at process start
(13:42:35.013Z), then `list_universes`, then **one** `set_active_universe` at +16 ms. That report
is written to the *active* universe's `.constellation`, and it landed in `Eisa Universe`
(`note_count 2731`). `set_active_universe` hard-fails at `universe.rs:1026` when the id is not in
the registry, and writes `active_id` + `save_registry` at `:1225-1226`. **Therefore the real
registry contained an `Eisa Universe` entry and was written that day.** `owner.info.json`
corroborates independently: `universe_lock::write_info` is called from exactly one place —
a *successful* `OwnerLock::acquire` — and `refresh_heartbeat` has **zero callers**, so its mtime is
an acquisition time, not a touch.

**The contamination is three records deep, and the third is the one that looks most authoritative:**

- `lab/reports/SESSION-LOG-2026-08-24.md:135-142` — corrected above.
- `docs/Constellation Pending Jobs v1.98.md:362-369` and `:923-941` — the fifth and fourth PJ-321
  corroborations.
- **`lab/reports/pj321-evidence-snapshot-2026-08-22/universes.json`** — committed to the repo as
  PJ-321's durable evidence by `bbb6ba9e` / `5ae1036d`. It is a copy of the shadow. Three hashes
  on three objects that could have differed — the committed blob, the working tree, and the live
  container file — are all `c20f9694c5b3d21c9dce964700250c6c7e3f614f3115db0c6c9d04aa17946afd`. The
  ledger entry at `:926` records that invariance **as the finding**. It is a finding: that a
  snapshot does not change. The bundle's two siblings came from `E:\` and are genuine.

**The shape, named.** This is the fourth failure mode from the law written yesterday — *a
cross-check that could not disagree* — in its purest form. Five corroborations were gathered, each
re-reading the same frozen 277 bytes through the same redirected path, and their agreement was
recorded as mounting evidence. No number of repetitions could have produced a different answer.
The discriminating measurement took **one command** and was never run, because the observation
looked self-confirming. The ledger entry's own instruction — *"the next person to touch this
reproduces it under instrumentation or leaves it alone"* — was right, and the instrument was
`fsutil hardlink list`.

**Standing method rule, added to the verifier's brief:** any Constellation file read under
`%APPDATA%` from this environment must be `fsutil hardlink list`-checked **before** its contents are
treated as fact. If the answer names `…\Packages\Claude_…\LocalCache\…`, the bytes are a snapshot.

**Two real defects surfaced while settling this** (neither is the premise, both are genuine):

- **`set_active_universe` saves the durable intent LAST, with no rollback** (`universe.rs`): it
  flips `active_path` at `:1166` and takes the OS owner lock at `:1170`, but `save_registry` is the
  final statement at `:1226`. If that write fails, the command returns `Err` *after* the process
  has already switched — `UniverseManager.handleSwitch` then does not call `onSwitch()`, so the
  window keeps rendering universe A while every Rust command targets universe B. That is the
  "half a switch" state PJ-310 closed at the function's *entry* and left open at its *tail*. At
  boot, `+layout.svelte:3596` swallows the throw and `continue`s, calling the command again for the
  next entry — moving the pointer and the lock a second time in a loop that believes the first
  attempt did nothing. **Filed.**
- **`remove_universe_from_registry` never clears `active_path` or releases the owner lock** — a
  universe can be active but unregistered in-session. **Already filed under PJ-322**; this is an
  independent second observation of it, not a new entry.
