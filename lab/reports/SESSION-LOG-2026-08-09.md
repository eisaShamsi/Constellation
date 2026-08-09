# SESSION LOG — 2026-08-09

**PJ-230 · PJ-231 · PJ-232.** Boss: *"Fix PJ-230 & 231 first"*, then *"Fix PJ-232 too, then commit all three."*
Preceding work: `SESSION-LOG-2026-08-08.md` (PJ-207 §9–§11, PJ-228, PJ-229).

## PJ-231 — a discarded result on a pass that writes to the user's `.md` files

`search.rs`: `let _ = mig003_step3_soft_rebackfill(&mut conn, path);`

Investigating it changed the fix. The function already `diag_log`s a full tally — repaired,
injected, per-file errors, elapsed — on every path that reaches its end. What `let _ =` discarded
was the **early exit**: an `Err` from the opening probe (a locked database, a schema the query does
not expect) returns before any of that, so the repair silently did not happen and nothing recorded
it. Narrow, but it matters here more than for its siblings: this pass is ungated, runs on EVERY
boot, and repairs rows by re-indexing from the file — writing an identity key into the user's
frontmatter when the file has none. **A repair that touches note files must not fail invisibly.**

## PJ-230 — not a bug to fix; a bug already fixed, and unrecorded

Investigated with a 4-agent workflow before touching anything. **The gap as filed is not
reachable.** `run_migrations_on` fires only on `schema_incomplete` — a `note_meta` missing one of
five columns — and any database old enough for that predates the crash markers entirely. *A
database old enough to drift cannot be armed; one new enough to be armed cannot drift.* PJ-228
removed zero coverage there. Option (b) — "heal the child when it is opened" — is already true for
free (`set_active_universe` → `invalidate_search_state` → `ensure_search_db_ready` →
`derived_heal::maybe_schedule` against ITS db_path, verified end to end).

**What the investigation actually found:** the pre-PJ-228 in-`init_db` heal, running in the
PARENT's process, rebuilt a CHILD's aggregates with the parent's vocabulary **and then cleared the
child's markers** — and the clearing is what made it permanent, because the child's own next boot
then saw nothing left to heal. PJ-228 ended that, but the commit credited boot latency, so the
repair went unrecorded. **The deliverable was the record**: two comments that still claimed
`init_db` self-heals, the `#[allow(dead_code)]` pointing at a dead reader, the call site itself,
and a regression test that fails if a self-heal is ever put back into `init_db` — because the next
person to have that idea will be reading about latency, not federation.

Deliberately NOT done: teaching the parent to load the child's vocabulary first. That means
swapping a process-global on a background thread while every other subsystem reads it — real risk
bought to close an empty gap.

## PJ-232 — the inspection refuted the comment I had just written

The diff-scoped inspection came back with one CONFIRMED HIGH, and its target was **my own new
comment**, which asserted that after PJ-228 `init_db` no longer applies the parent's vocabulary to
a foreign database. **False.** Verified myself before acting:

- `init_db` unconditionally recreates the outgoing-aggregate triggers (`create_outgoing_link_triggers`)
  and the three Sky stratum/maturity trigger blocks; every body is generated from
  `link_types::snapshot()` (`search.rs:2007`, `188-189`, `266-267`).
- That global holds ONLY the active universe's registry — `link_types::load_active` runs at
  `search.rs:10939`, immediately before `init_db`, for the ACTIVE universe.
- So `federation::migrate` persisted **parent-flavoured DDL into the child's `sqlite_master`**.
- And then — the sharper edge, which I had not seen — `mig003_step3_soft_rebackfill` (the very
  function PJ-231 had just touched) runs ungated ~290 lines later, re-indexing every row with an
  empty `cid_cn`. On a schema-drifted child that set is **non-empty by construction**, so it fires
  the parent-flavoured triggers on the child's rows and, for a file with no identity key, writes
  frontmatter into the child universe's `.md` files from the parent's process.

Qualified honestly: it diverges only when the two universes' link vocabularies actually differ (a
user-defined type on either side); with seeds only the generated SQL is byte-identical, which is
why nobody had seen it.

**The fix — a named door, not a boolean buried in a signature.** `init_db(path)` is unchanged and
keeps every existing caller (verified: converge, index_repair, link_life_*, mig108 — mig108's is
the ACTIVE universe, resolved via `db_path(app)`). New `init_db_schema_only(path)` runs
`init_db_scoped(path, InitScope::ForeignSchemaOnly)`, which migrates the schema and **nothing
else**: no vocabulary-dependent trigger DDL (4 blocks), no dependent-table back-fill, no soft
re-backfill, and **no Step-4 rename pass — that one walks libraries and RENAMES `.md` files**, on
another universe's notes, from this process. The owner does all of it correctly on its own next
launch, and until then nothing writes through those triggers because a cUniverse attaches
read-only.

**Three tests**, deliberately paired so one cannot be satisfied by breaking the other: a foreign
init writes none of the five vocabulary-carrying triggers but DOES bring the schema up to current;
the active door still creates all five; and the marker test from PJ-230.

**Diff-shape note:** `search.rs` shows ~457 changed lines, mostly the re-indentation of three large
DDL statements now inside `if owns { … }`. The added whitespace falls inside multi-line SQL string
literals; SQL ignores it, and nothing in the tree fingerprints trigger DDL text (the only
`sqlite_master.sql` read is an INDEX check for the substring "where" — `search.rs:3581`, `3772`).
Verified rather than assumed.

**Gates:** Rust **1384/0** (15 ignored; +3 new). No frontend change.

**Boss-testability, stated plainly:** none of these three is reachable from the GUI in the Boss's
current setup — PJ-231's is an error path, PJ-230 is comments plus a test, and PJ-232 needs a
linked universe whose database is old enough to be missing schema columns. The tests are the
verification. The Boss ruled "commit all three."

## PJ-232, round 2 — the inspection caught a pass I MISSED, and a test that could not fail

Ran the diff inspection on the PJ-232 fix. **One CONFIRMED finding, and it was mine.**

**The miss: MIG-003 Step 1.** I guarded Steps 2, 3, 4 and the four vocabulary-DDL blocks and
left **Step 1 (`mig003_backfill_cid_cn`) unguarded** — the FIRST data pass a foreign database
reaches, sitting before all my guards. It was missed for a boring, instructive reason: it is not
named `mig003_step1_*`, so the grep that found its three siblings did not find it. **The
Whole-Ecosystem Fix Law failing in exactly the way it warns about — I fixed the sites my pattern
matched.** It is also the worst of the four: it writes `cid_cn:` frontmatter into the universe's
`.md` files, rewrites files in its dedup branch, and DELETEs `note_meta` rows whose path no longer
stats — cascading into `note_state_history`, which is NOT recomputable from the files — and then
stamps `schema_versions.note_meta`, so the owner's own next launch reports "already done".
`run_migrations_on`'s backup covers `search.db` only, so a restore would not undo the file writes.

**The test that could not fail.** The inspection also noted my new test ran against an EMPTY
database, so the file-writing pass never had a row to act on: it passed while the bug was live.
Two further attempts still could not go red — investigated rather than assumed each time:
1. the file-write path goes through the write gate, which cannot fire in a unit test with no
   universe registered, so **an assertion on file bytes can never go red here**. Kept, but only as
   a statement of intent;
2. the row-DELETE path needs no gate — that is the assertion with teeth;
3. and the fixture's own setup call was stamping MIG-003, so the real call skipped Step 1 anyway.

Fixed by seeding a row whose file has **moved away** (what a re-linked universe looks like) and
clearing the stamp after setup. **RED→GREEN proven by hand:** guard removed → `left: 1, right: 2`
(the row was deleted); guard restored → green.

**Also corrected:** with `owns &&` added, three else-branches were logging *"skipped (already
done)"* for a foreign database where it is emphatically not done — a false statement in the log,
the same class of error as the comment this whole thread started with. They now say which it was.
And `InitScope::ForeignSchemaOnly`'s doc no longer says the loose "no data repair": it names
exactly what is skipped (the four MIG-003 one-shots + vocabulary DDL) and what is deliberately
NOT (plain schema DDL, the FTS rebuild, the initial-history back-fill — all read only the child's
own rows, touch no file, use no process-global, and the parent READS a cUniverse's derived tables
through the read-only attach, so withholding them would degrade federated reads while protecting
nothing).

**Gates:** Rust **1385/0** (15 ignored; +4 tests over the session).

## PJ-207 §12 — the documentation

Boss: *"Start §12"*, then *"Continue"*.

**The plan's §12 was written to REMOVE a promise; §11 built the thing, so it also had to
DESCRIBE it.** Both halves done.

**Corrections** (all four plan targets verified still present): `Index.md` "there's nothing to
'rebuild'" scoped to *while Constellation is running*; the User Manual's semantic-search sentence
rescoped to the M11 layer it was always about, with a pointer to the real repair; "caches rebuild
in the background" reworded in `Universe.md` and `README.md`. The two the plan said NOT to sweep
(the Arabic-override reindex sentences) left alone — verified still accurate.

**New:** a User Manual subsection, *"If your notes changed while Constellation was closed"*, inside
§2's external-changes section — which until now covered only changes arriving while the app is
OPEN. Covers the band, Repair now, the Settings route, what the repair does and does not touch,
background progress + Cancel, the fresh re-derivation, the Last-repair report and its in-memory
lifetime, and PJ-228's catch-up strip.

**Two plan assumptions corrected by looking:** the semantic sentence exists in **Arabic only**, not
"the 13 other locales" — the rest are shorter partial translations that never carried it. And every
locale HAS the external-changes section, but the heading after it differs and several lack "Universe
Notes Folder" entirely, so each translator located its own insertion point.

**Code + contract:** four stale comments now name the control that exists (`+layout.svelte` ×2,
`library_attribution_backfill.rs`, `search.rs`); two others already record the false promise as
history and were left as the durable trail. `IPC-CONTRACT.md` gains the repair commands with their
real contracts, both progress events, and why `cache_reconcile` is deliberately absent. The plan's
verification grep returns only intentional history. ✓

### The gate caught me contradicting my own measurement
REJECTED on one claim of 29: I wrote that earlier versions did this *"before showing the window,
which made that one launch look frozen."* The window is created visible (`tauri.conf.json`) and was
already on screen; and **"frozen" is the word I had explicitly rejected for this defect the day
before**, after measuring it — `SESSION-LOG-2026-08-08` says *"a ~3 s silent pause, not a freeze."*
I did the measurement so severity would not be an adjective, then reached for the adjective anyway.
Corrected and re-gated: APPROVED. The inspector also flagged (not rejected) that "about three
seconds" is one universe on one machine; I shipped the approved wording rather than editing after
approval — post-gate "improvements" produced three of tonight's findings.

### Translation: 14 agents, then a verification pass — 4 real findings
Each agent pasted UI labels **verbatim from its own locale's `i18n` JSON** rather than translating
them (a manual naming a button the app does not show is worse than one left in English). The
verifier re-checked all 14 against those same values. Structure: 16 insertions, 0 deletions, one
subsection each, correctly placed — all 14.

- **es** — wrote the Settings window as *"Configuración"*; the app says **"Ajustes"**
  (`es.json:104`, rendered `SettingsModal.svelte:772`). Two occurrences. Also misquoted the band
  (*podría* for the app's *puede*). Both fixed. The agent had reported these labels as "verbatim
  from es.json" — a false self-report, which is exactly why the verifier exists.
- **ja** — dropped the spaces the template puts around `{noun}`; the quote now matches. Fixed.
- **ko** — the manual was FAITHFUL, and in being faithful it exposed an **app bug**: `indexDrift.
  changed` / `missingFromIndex` used the generic `이(가)` particle placeholder after a noun that is
  ALWAYS `노트 {count}개` — always vowel-final — so Korean users read `개이(가)`, which is not
  Korean. Fixed in `ko.json` (→ `개가`); the manual re-quoted to match. **Found by writing the docs.**
- **ur** — glossed Sky View as *"(سٹار ویو)"* = **Star View**, a name the Boss corrected long ago
  and which appears nowhere in `ur.json`.

### And that gloss uncovered a real documentation defect
`fa` and `ur` call the panel **Star View** throughout — 22 and 20 occurrences in their manuals, plus
25 more across four `fa` topic pages — while their own app strings say **Sky View**
(`fa: نمای آسمان`, `ur: آسمانی منظر`). A reader would hunt for a panel that does not exist. My new
paragraphs had inherited the error, because I told the agents to match each file's conventions and
the convention was wrong. **All 67 corrected**, headings and TOC anchors together (anchor links
re-verified against their headings). Persian's remaining "star" references are legitimate — notes
ARE stars in Constellation's metaphor ("fly through the star field", "right-click any star").
Also fixed in passing: `ur` named `Ctrl+O` *"سٹار جمپ"* (Star Jump), a feature that does not
exist — the app calls it the Quick Switcher (`ur.json: فوری سوئچر`).

**Left as-is, deliberately:** `de` restates the five family names parenthetically and `fr` names the
Repair-index item as well as the button — both additions to the English, both accurate, all labels
verified. Trimming accurate text for symmetry is not worth a pass.

**Gates:** i18n parity **15/15 ✓** (ko.json changed) · docs-only otherwise. Per the plan, §12 is
exempt from the per-build inspection (no write/index/lifecycle path) and **NOT Boss-testable** —
except the Korean string fix, which is user-visible but only in Korean.

## PJ-207 §14 — Full re-read: built, MEASURED, still switched off

Boss: *"Start §14"*. Built exactly as the plan specifies, and the flag stays **off** — the flip
is its own commit, and by Boss ruling (2026-08-03) it must carry a confirmation dialog quoting a
real number. This step produces that number.

### Built
`WalkCtx.force` threaded end to end (`reconcile_filesystem` → `_guarded` → `run_full`), so the walk
calls `index_note(force)` instead of a hard-coded `false`. `Scope::FullReread` added.

**`covers()` needed a rule, not a default.** A `FullReread` subsumes an ordinary `Full` (strictly
more work, same scope) — but **`Full` does NOT cover `FullReread`**: it skips exactly the notes the
re-read exists to reach, so answering "already running" would refuse a request it will never
satisfy. Pinned both directions.

**Refused in RUST, not only in the UI.** `repairFlag.ts`'s own scope note demanded this — *"a
UI-only gate hides a feature, it does not make it unreachable, and PJ-207 exists precisely because
a reachability claim went unverified for months."* `submit` returns
`Blocked{ FullRereadDisabled }` while the new `index_repair::FULL_REREAD_ENABLED` is false, so a
devtools `invoke` or a future caller cannot reach it either. **Two constants must flip together.**

Verification clause met: `cargo build --release` mentions `FullReread` in **zero** warnings (the
variant is referenced by the dispatch arm and the refusal comparison). Rust **1387/0**.

### §M1 — the measurement, and the first run of it was WRONG
An `#[ignore]`d, env-gated harness re-indexes from a COPY of `search.db` while reading the REAL
`.md` files. Safe against live data because `index_note` never writes a note file — verified by
grep, and it is the invariant §11's Boss test asserts.

**First run produced plausible, invalid numbers** (204 s for 799 notes) and the tells were in the
output: `note_meta` moved **2,721 → 3,541**, and `unchanged` was **0 even with force=false**. Cause:
the app stores platform-native paths (`E:\…` backslashed, walked from `libraries.json`'s root) and
the harness passed forward slashes, so every note MISSED its row and was INSERTED. It was measuring
fresh indexing and calling it a re-read. Fixed by normalising to `MAIN_SEPARATOR_STR`, **and the
harness now asserts the row count did not move** — a mismatched run fails instead of reporting a
number.

**Valid measurement, Eisa Universe's four own libraries, on the Boss's USB drive:**

| Library | notes | ordinary (mtime-gated) | full re-read |
|---|---|---|---|
| تخطيط الدولة | 276 | 0.0 s | 3.3 s (83/s) |
| المساعد الذكي | 21 | 0.0 s | 0.1 s (216/s) |
| Constellation PKM | 799 | 0.1 s | 20.1 s (40/s) |
| الكون المعرفي | 162 | 0.0 s | 1.2 s (139/s) |
| **total** | **1,258** | **0.1 s** | **24.7 s** |

**≈ 51 notes/second overall**, varying 40–216/s with note size (PKM's are the largest). Run-to-run
variance is real: PKM measured 14.9 s and 20.1 s on two runs (40–54 notes/s), so any quoted figure
is a rough guide, not a promise. The mtime-gated pass is effectively free — **0.1 s for 1,258
notes**, ~10,000 notes/s, because it stats and does not open.

The harness also caught the Boss's `brimsloe` edit as the one genuinely-changed note in
المساعد الذكي (`indexed 1 · unchanged 20`) — an independent confirmation that the mtime gate works.

**Extrapolation, stated as such:** at ~51 notes/s the Boss's 7,824-note universe would be ≈ 2.5
minutes. NOT measured — the plan's 49 s I/O floor was for that universe; this run was 1,258 notes.

**BOSS-TESTABLE — no.** The flag is off; nothing user-visible ships. **The mid-run Cancel check
therefore cannot ride §14** — it belongs to the flip commit, where a minutes-long run finally makes
the gesture catchable. I told the Boss otherwise at §12's close; correcting it here.

## PJ-207 §14 FLIP — the Full re-read is ON, Boss-passed

Boss: *"Flip it on"*, then **"All pass"**.

### Built
`constellation_search_init` gains an optional `full_reread` — **one door, two scopes** — so every
pre-existing caller (boot cold-start, add-library, the band's Repair now, the ordinary Repair)
keeps its exact call shape AND meaning; only the new control passes `true`. Settings → Index gains
a second row with its own confirmation, and the confirm copy **quotes the §M1 numbers** rather than
warning vaguely, which is what the 2026-08-03 ruling demanded. `settings.index.repair.fullReread.*`
×15 locales. Both gates flipped — the frontend one hides the control, `index_repair::
FULL_REREAD_ENABLED` refuses the request; the Rust one is the load-bearing half.

**The OFF-pin was inverted, not deleted.** The test asserting the flag was off existed to fail
loudly on a careless flip. The property did not disappear, it reversed: it now asserts both gates
agree, with the reason stated — turning the feature off again means turning BOTH off, because
leaving the Rust gate open while hiding the button reproduces the exact "unreachable, we assumed"
shape PJ-207 exists because of.

### ☠️ The inspection found FIVE, four of them mine, one a live regression
1. **HIGH — a save during the re-read could be overwritten with stale bytes.** `index_note`'s
   save-during-read guard was scoped `if !force`, on the stated premise that *"every force:true
   caller reaches here from a 'this file just changed' context"* where another write is already in
   flight. **§14 falsified that premise**: the bulk walk became a force caller, nothing is coming
   for a walked note, and the guard silently switched off for a run lasting tens of seconds. A note
   saved mid-run could be written back with the pre-save bytes the walk had already read — counted
   `Indexed`, no error. Fixed by splitting the intent: `index_note` (event-driven) and
   `index_note_bulk` (walk, guard ON regardless of `force`). **Two call sites, not 34.**
2–4. **One root cause.** `matches!(scope, Scope::Full)` appeared THREE times meaning "is this a
   whole-universe run?" — the receipt gate, the post-run follow-ups, the progress gate — and the new
   variant slipped past **all three at once**: a full re-read would have rendered the PREVIOUS
   repair's receipt as its own, never re-derived the drift band, and shown a strip frozen at zero.
   Replaced by `Scope::is_whole_universe()`, written as a `match` so the next variant must answer.
5. **A dead §11 placeholder** gated on the same flag rendered a SECOND, broken "Full re-read" row —
   disabled button, i18n keys that never existed. The flip made it visible. Removed.

**Re-inspection after the fixes: 0 confirmed.**

### 🛡️ The test gate: three rounds, and a finding that would have wasted the round
REJECTED twice. The critical finding: **the test never said which Universe to be in.** Step 5 needs
the run to last; on `كون عيسى` (6 notes) a full re-read ends in well under a second and Cancel would
have outrun the Boss a THIRD time, for reasons unrelated to the fix. A Step 0 now checks the title
bar.

**The inspector reached the right action from the WRONG evidence** — it read `universes.json`, whose
`active_id` names `كون عيسى`. That registry is known-unreliable here. Challenged to verify
independently, it did better than the mtime argument I offered: it read the **app-generated**
`"timestamp":"2026-08-09T05:04:04.965Z"` inside Eisa Universe's boot record and its logged boot
sequence, then **counted the files** — 1,260 in the four libraries (draft: "about 1,250") and 2,102
in the whole own tree (draft: "roughly 2,100"), and 6 in `كون عيسى`. Then rejected again for ONE
stray "Library" left in the second clause of the very sentence whose first clause I had just fixed —
**the same partial-fix pattern as the MIG-003 Step 1 miss earlier today.**

Also corrected: the draft narrated the dialog as saying "Library" where it says **Universe**; and a
caveat was added for the progress-clears-cancelling quirk §10 preserved deliberately (a tick in
flight when Cancel is pressed briefly restores "Repairing the index…"), so a working Cancel is not
reported as broken.

### Boss result — "All pass"
Step 0 → Step 6, including the mid-run **Cancel finally exercised** after outrunning him twice, and
the next-launch "Finishing an interrupted index repair…" safety net behaving as described.

**Gates:** Rust **1389/0** · vitest 913/913 · svelte-check 0 · i18n 15/15 ✓ · inspection 5 → **0**.
Binary 14:02, newer than every fix.

### 🆕 PJ-233 — filed, not chased: the app is running a Universe absent from its own registry
The inspector could not reconstruct, from the source, HOW `set_active_universe` — which requires the
target id to exist in `registry.entries` — activated Eisa Universe when `universes.json` lists only
`كون عيسى` (registry mtime 2026-08-07). It said so plainly rather than inventing a mechanism. The
conclusion about which universe is active rests on independent evidence (app-generated timestamp,
file counts, the federation manifest), so it does not affect this test — but a registry that
disagrees with reality is exactly the class of "unverified reachability claim" PJ-207 exists for.
