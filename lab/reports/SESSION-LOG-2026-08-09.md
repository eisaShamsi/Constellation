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

---

## §15 — the whole-app cycle sweep: all 32 findings FIXED

The per-cycle inspection returned **32 confirmed findings** (3 APP-KILLER, 7 HIGH, 20 MED, 2 LOW).
The Boss ruled twice: "Fix all three", then "Fix the remaining". None were deferred.

### The three app-killers
1. **Frontmatter edits silently discarded on malformed YAML** (`yamlDoc.ts` / `noteModel.ts`) —
   compose returned the original bytes on any parse error while the model accepted and displayed
   the edit, and `FmDoc.hasErrors` had no consumer anywhere. Now REFUSED at the model (all five
   mutators), so PJ-187's existing banner fires. 6 vitest pins.
2. **Workspaces carried across a universe switch** (`store.ts`) — `loadWorkspaces` adopted an empty
   successful read only `if (data.length > 0)`, contradicting its own comment, while still latching
   writes. `resetWorkspacesForUniverse()` + wired into `handleUniverseSwitch`. 3 vitest pins.
3. **The `sources:` block-strip ate a comment or continuation** (`sources/mod.rs`) — the block ended
   at the first non-sequence line, orphaning the remaining `- ` items under a deleted key, i.e.
   unparseable YAML → app-killer #1's precondition. Routed through `ends_dropped_block`.

### The seven HIGH
4. **Folder rename froze the app** (`libraries.rs`) — the descendant DB cascade held the global
   writer mutex across every `migrate_note_db_paths` and then reindexed each note, INLINE on the
   awaited IPC. Detached to `rename_folder_db_tail` via `spawn_blocking`, matching the `.md` branch
   and `move_item` which already did this.
5. **`bases.rs::format_yaml_value` under-quoted vs its TS twin** — missed a leading `- `, `*`, `&`,
   `!`, `@`, backtick, `|`, `>`, `%`, `?`, `,`, `}`, `]`, bare `true/false/null/yes/no`, whitespace.
   A Bases cell of `- pending` or `@home` emitted frontmatter that no longer parses. Parity + tests.
6. **Eight hand-rolled `inner.split(',')` sites in Rust** — a quoted alias with a comma
   (`"Ibn Khaldūn, ʿAbd al-Raḥmān"`) was torn in two on every rename. ONE `split_flow_seq_items`
   scanner in `yaml_lines.rs`, mirroring the TS twin; all eight routed through it.
7. **A quoted wikilink read as a list** (`store.ts`) — `parent: "[[Architecture]]"` was unquoted,
   then matched `[`…`]`, split, and came back as `['[Architecture]']` with a bracket pair gone.
   Constellation writes that shape itself. Fixed by remembering that quotes were stripped, and by
   testing the wikilink rule BEFORE the generic flow-sequence rule in `detectPropertyType`.
8. **`linkTypeRegistry` had no per-universe reset** — the sibling `propertyTypeRegistry` took this
   exact finding on 2026-08-01; this one was never swept. Reset at the two real switch sites rather
   than on every load, so an ordinary failed reload still keeps the last-good vocabulary.
9. **The axis writers deleted values they did not recognise** (`sources/mod.rs`) — both writers
   rebuilt the block from the taxonomy-filtered set, so a hand-written or Obsidian-imported
   `sources:` / `content_type:` value was erased from the user's file the first time anything
   touched that note. Unknown values now ride through, quoted by the shared quoter.
10. **`reloadTabsFromDisk` was not focus-aware** — it bumps `tab.reloadVersion` (NotePane's `{#key}`)
    but FocusPane is not under that key, so Focus kept the pre-adopt body and its next flush wrote
    it back over the adopted change. Two of nine callers hand-wired the reseed. Now a
    `registerFocusSurface` registration the callers cannot forget; both hand-wirings retired.

### The twenty MED + two LOW (grouped by concern)
- **Silent write failures**: swallowed reindex `Result` in both rename/move tails; `libraries.json`
  rewrite failure reported only by `eprintln!` (invisible in a release build) → diagnostics log;
  discarded `MaintenanceOutcome` on the save-path reindex → logged, and `is_clean()`'s "no consumer
  yet" annotation is now false and removed; `saveAppPrefs` swallowing a failed language write →
  returns a result, and Settings says so; `ConfidencePicker` closing before ignoring both its IPC
  failures → closes only on success, with an inline reason; `toggleTaskReconciled` resolving after
  SKIPPING the write → throws, which is what makes its three callers' existing failure paths run.
- **Partial operations reported as none**: `gate_rmw_rename` rewrote the frontmatter and then, on a
  terminal rename failure, returned an error having left the note carrying its new title under its
  old filename → the rewrite is rolled back; the legacy-data migration deleted the localStorage
  source keys unconditionally after ignoring all four write failures → deletes only what arrived.
- **Wrong-universe / wrong-note state**: `applyParsedSettings` returned early on `{}` leaving the
  previous universe's settings live AND latched; `openNoteTab` reindexed with the caller's
  untrustworthy `libraryName`, wrote `cid_cn` into the user's file from the READ-ONLY second screen,
  and could open a model for a tab closed during its two awaits; the second screen never followed a
  rename/move/delete, holding tabs on dead paths.
- **Freezes**: `list_trails`, `read_trail`, `discover_base_properties`, `list_canvases` were all
  SYNC commands doing recursive walks on the main thread → `(async)`, as their own file-sibling
  `execute_lens` already was.
- **Derived-data staleness**: `note_links.target_cid_cn` was stamped only while indexing the SOURCE
  note, so a link written before its target existed stayed unresolved until that source happened to
  be edited again → resolved at write time by the `note_meta` insert trigger (index seek on
  `idx_link_target`), per Rule 8.
- **Ordering**: `invalidate_search_state` cleared `db_ready` BEFORE taking the lock a publisher
  holds, so the final state could be `db = None` with `db_ready = true` — every later call taking
  the lock-free fast path against no database, for the session. Cleared again under the lock.
- **PropertyEditor**: the seed baseline was cloned from the RAW props while the displayed rows carry
  type overrides, and `samePropRow` compares `type` and `listItems` — so an overridden row counted
  as user-edited on seed and the 800 ms commit rewrote it on disk unasked (a scalar `parent:`
  becoming a list by merely opening the note). Baseline now cloned from the rows shown. And the
  debounced commit wrote `tab.content` BEFORE the PJ-187 identity guard, putting note A's properties
  into note B's tab content one line above the refusal that exists to prevent exactly that.
- **LOW**: `update_frontmatter_title` kept the newline after the opening `---`, so `lines()` emitted
  an empty first line and every rename added one more blank line to the note's frontmatter (its
  sibling `set_frontmatter_parent` had always stripped it) — fixed with a test that renames three
  times and asserts no accumulation.

**Gates:** Rust **1400/0** · vitest **926/926** · svelte-check **0 errors** · i18n **15/15 in parity**.
New tests this pass: 4 (quoted-wikilink scalar, TS) + 5 (axis preservation, quoting, blank-line
accumulation, Rust).

### §15 second pass — the cycle sweep audited the fixes, and found five of them wanting

The per-cycle inspection ran **whole-app** (the diff-scoped `files` argument did not take, which
turned out to be the more useful outcome) against the code as fixed. 72 agents, 14 hunt groups,
**32 confirmed findings** after adversarial refutation — a different 32. No app-killer survived
refutation; the one candidate (`noteModel.ts:513`, a missing model epoch) was downgraded to HIGH
when two of its three claims were refuted.

**Five of the 32 were defects in fixes made hours earlier today. All five are now fixed:**

| Finding | What my fix missed |
|---|---|
| `yaml_lines.rs` / `sources/mod.rs:559` (HIGH) | The app-killer fix covered only the **indented** comment. `is_top_level_key_line("# note")` is TRUE, so a **flush-left** comment still ended the block and orphaned every item below it — and the regression test I shipped used `  # a note to self`, pinning the half that worked. Fixed in `ends_dropped_block` (one rule, three call sites) + a column-0 test. The two sibling writers KEEP the user's comment, so these now do too — which means the earlier test's assertion that the comment is deleted was itself wrong, and is corrected with a note saying so. |
| `noteModel.ts:247` (HIGH) | I guarded five mutators and missed the **sixth**: `setProps`, the whole-array path from `saveTabContent`. The app-killer was reachable through it unchanged. |
| `store.ts:2014` (HIGH) | I added the tab-closed-mid-navigation guard to `openNoteTab` and **not to its sibling** `loadTabHistoryEntry` — the Whole-Ecosystem Fix Law failing in the same session that quotes it. Worse here: the net was already consumed, so it is restored on the refusal. |
| `search.rs:5370` (MED) | My new back-resolution trigger fires `AFTER INSERT`, but `index_note` upserts (`ON CONFLICT(path) DO UPDATE`) — so it only ever fired for brand-new notes and **never for the lazy cid injection it was written for**. I trusted a neighbouring comment claiming "index_note uses DELETE+INSERT"; that comment is false and is now corrected in place. Twin statement added to the `AU` trigger. |
| `+layout.svelte:9745` (MED) | Making `toggleTaskReconciled` throw fixed two of its three callers. The **sidebar** one only logged, leaving the checkbox ticked over an unchanged file. Now re-scans from disk on failure. |

The lesson is the one this codebase keeps paying for and is worth stating plainly: **a rule applied
to the half of the shape you happened to test is not a fix**, and **a comment is a claim with a
shelf life** — two of the five came from believing a neighbouring comment instead of reading the
statement it described.

**Gates after the second pass:** Rust **1401/0** · vitest **926/926** · svelte-check **0** · i18n 15/15.

**Still open: 27 findings** (10 HIGH, 11 MED, 6 LOW) — awaiting a Boss ruling on sequencing. Full
list preserved at `scratchpad/sweep2.json`; the notable ones are the missing model **epoch**
(`noteModel.ts:513`), the rename cascade skipping `[[type::Old]]` and heading/block anchors
(`libraries.rs:6406`), a universe switch **never notifying the second screen** (`notifyUniverseSwitch`
is imported and called from nowhere), `sky_links` rename re-pointing edges with no identity
qualification, a **non-partial UNIQUE index on `sky_nodes.cid_cn`** colliding on the `''` default,
and the `.canvas` writer using a plain `fs::write` with no app-close flush.

### §15 third pass — 23 of the 27 fixed; 5 held for a Boss ruling

Boss ruling: *"Fix all 27, then one test"*, accepting the caveat that anything touching a schema or
a source-of-truth write path gets flagged rather than slipped in.

**Method.** Nine agents verified each finding against current source and returned exact anchored
patches — read-only; I applied them serially so nothing collided, then read the result. All 27 were
re-confirmed still true; none had been fixed by the earlier passes. 105 edits proposed.

Two mechanical problems, both mine: the plan schema had no per-edit `file` field, and several plans
legitimately span files (a `NoteEditor` plan edits `store.ts` and `en.json` too) — 24 edits were
resolved to their real file by unique-anchor search. And my JSON writer used 2-space indent where
the repo uses tabs, turning the 15 locale files into ~8,500-line phantom diffs; rewritten in the
repo's own format, they collapse to the 4 real keys.

**Two applied edits were WRONG and were caught by the gates, not by review:**
- The cascade-regex widening broke two existing walker tests (0 rewrites where 2 expected): the
  tests build an inline 2-group pattern that the new 4-group replacement function cannot read. That
  is the drift the agent's own comment warned about — and it is also the change I had promised to
  flag, so it is **reverted and held**.
- `registerActiveEditor`'s read-only refusal used `view.state.readOnly`, which throws on the test's
  view stub (6 failures). Rewritten to PROVE read-only (`view?.state?.readOnly === true`), so the
  guard can never itself be why an editor stops receiving inserts.

**Notable among the 23 fixed** — the model gained a **lineage stamp** (`gen`, minted per model
object): `openModel` re-seeds an id in place for the SAME path, resetting `version` to 0 while
`savedVersion` only rises, so a save resolving across that re-seed stamped `savedVersion` far above
`version` and `isDirty` reported CLEAN for the next N real edits — which the departure flush, the
app-close flush and the write-ahead net are ALL gated on. Paired with `netUnsaved`, which marks a
model whose content came from the crash-recovery net and is not on disk; the restore seeds such a
model clean by design, so a genuine external write adopted over the recovered work and the adopt
cleared the net behind it. Both guards refuse rather than act, and `hasUnsavedRecovery` is wired at
all three arbitration sites — including the conflict branch, so a real external change on a recovery
model now raises the `.conflict` sidecar instead of vanishing.

Also fixed: the universe switch now **notifies the second screen** (`notifyUniverseSwitch` was
imported and called from nowhere, so the SS kept the outgoing universe's tabs and models for the
life of the window); a read-only pane can no longer become the emoji/template insert target; the
`.canvas` writer and `arabic-overrides.json` moved onto `atomic_write` (fsync + unique temp);
`de_canonicalize_library` no longer treats an unreadable sidecar as "no original filename" and
rename an attachment to a mangled third name; batch delete and single rename/delete surface their
failures (2 new keys × 15 locales).

**Held for a ruling (5)** — all real, none shipped:
1. `+layout.svelte:7342` — the rename cascade walks ONLY the renamed note's own library, so a
   referrer in a sibling library keeps a dangling link. Widening the walk alone would CREATE a
   defect: the flush, freeze and save-gate are all scoped to the same one library.
2. `libraries.rs:6406` — the cascade rewrites `[[Old]]` and `[[Old|…]]` but skips the app's own
   `[[type::Old]]` and every `#heading` / `^block` anchor. Widening it changes what a rename
   rewrites in the user's files.
3. `search.rs:5350` — `sky_nodes` has a NON-partial UNIQUE index on `cid_cn` while the documented
   default is `''`, so notes without an identity collide and `INSERT OR REPLACE` evicts each other.
   Schema change on an existing database.
4. `search.rs:5433` — the rename trigger re-points `sky_links.target_name` for EVERY edge naming the
   renamed note, with no identity qualification.
5. `UniverseSetup.svelte:251` — `save_universe_bookmarks` does not exist in `src-tauri/src`, so that
   leg of the first-run migration always fails. (Harmless today only because this session's earlier
   fix stopped the migration deleting the localStorage copy of what it failed to move.)

**Gates:** Rust **1404/0** · vitest **926/926** · svelte-check **0** · i18n **15/15 in parity**.

### §15 fourth pass — the 5 held (Boss: "fix all 5") + a third sweep's two app-killers

**The 5 held are fixed.** Boss ruled to take them despite the migration-sized flag:
1. The rename cascade now walks every own library in the Universe, with the flush, freeze and
   save-gate widened to match. **CORRECTION, same day:** when this was first written the entry was
   FALSE. The applied plan widened the freeze/flush/gate to every library root and left the rewrite
   call itself passing one library — and the code comment claimed the widening had happened, which
   is what I repeated here without checking the call below it. The **tutorial-auditor caught it**
   while deciding whether the behaviour was real enough to show the Boss, and refused to write a
   test for something the code did not do. Completed properly afterwards: the walk is rooted at the
   Universe root (MIG-108 puts every own library under it, so one pass covers all of them with no
   duplicate walks) and `update_links_recursive` — the one tree walker in the file with no boundary
   notion — now stops at a linked universe via the same `foreign_library_roots` its siblings use.
   The widened freeze/flush/gate is not trimming: without it a dirty tab in another library would be
   rewritten from its stale disk and the rewrite then dropped by that tab's next save.
   The lesson is this session's third instance of one thing: **a comment is a claim, and I read the
   comment instead of the call it sat above.**
2. The cascade pattern now matches `[[type::Old]]` and `#heading` / `^block` anchors. Both walker
   tests were rebuilt to construct the PRODUCTION pattern via `cascade_pattern` — each had held its
   own inline copy, which is why they went green against a regex the app had stopped using and then
   failed the moment the replacement function was taught the new groups.
3. `sky_nodes.cid_cn` UNIQUE index made partial, so notes without an identity stop colliding.
4. The `sky_links` rename is identity-qualified — it used to re-point EVERY edge naming a note with
   the same title (measured: retitling one of seven same-titled notes rewrote 4,359 rows, ~4,300 of
   them belonging to the other six, with nothing to heal them).
5. `save_universe_bookmarks` exists again, so that leg of the first-run migration can succeed.

**The third whole-app sweep (73 agents, 37 confirmed) found TWO APP-KILLERS, both fixed:**

- **`sources/mod.rs` — a note whose bytes begin before its `---` fence got a SECOND fence.** The
  writers detected frontmatter with a raw `starts_with("---")` while the indexer's canonical reader
  trims first; on such a note the writer concluded there was none and PREPENDED one, pushing the
  note's real YAML down into its body. **A scan of the live universes found 28 notes with exactly
  this shape**, several of them Arabic notes in the Boss's own libraries, and the trigger is an
  ordinary Accept in the Source Review panel. One `fence_offset` helper now serves the reader and
  both writers, and the bytes before the fence are preserved rather than trimmed. Proven RED→GREEN:
  with the old check restored, the reader returns `[]` for a note it should read.
- **`NoteEditor.svelte` — the crash-recovery net erased itself.** A model restored from the net is
  CLEAN by construction while holding content disk never had, so the teardown flush stamped the
  note's only recovery copy as an already-durable snapshot; the next open called it stale and
  deleted it. Merely LOOKING at a recovered note and switching tab was enough. `hasUnsavedRecovery`
  existed and was not consulted at that one line.

Three more of the 37 were consequences of this session's own fixes and are fixed: the `AU` trigger's
`WHEN` clause excluded `cid_cn`, making yesterday's link back-resolution unreachable for the very
case its comment named; `setProps`'s new refusal reached `addTagToNote`/`addLinkToNote` as a clean
success (it now returns a boolean and surfaces through the save-health banner); and the second
screen's peek pane did not follow a rename.

**Gates:** Rust **1410/0** · vitest **926/926** · svelte-check **0** · i18n **15/15**.

**Open: ~29 findings from the third sweep**, overwhelmingly long-standing rather than new — e.g.
`move_item` authorising its destination with the FEDERATED resolver (a note can be moved into a
linked universe), the write-ahead net's localStorage blob never pruned or capped, `bases.rs` and
`set_frontmatter_parent` ending a dropped block at a BLANK line, `note_embeddings` carrying the same
UNIQUE-collision shape as `sky_nodes` did, the sky/links back-fills driving in-memory cursors across
a universe switch with no generation guard, and three `universe.rs` readers returning an empty
success on a metadata failure that the frontend latches as a real read.

---

## §15 close — the performance arc, measured end to end

The Boss's Stage-1 test passed 4/4 (plain link, typed link with annotation, folder-rename contrast,
cross-library rename). He then reported, unprompted: **"Every time I create a new note, it takes 54
seconds, and about 50 seconds to rename one. We have solved this issue in the past."**

He was right that it had been solved before — and right that it had come back, in a way nobody had
looked for. Four causes, found by measurement, fixed in order:

| # | Cause | Measured | After |
|---|---|---|---|
| 1 | **The federated title-collision walk.** MIG-099 made the create/rename title check an index seek for OWN libraries ("13.6 s → sub-10 ms", its own comment) and left FEDERATED libraries on a filesystem walk, reasoning *"federated trees are small"*. His linked universe holds **7,964** notes against his active **2,104** — so every create and rename read all 7,964 files to inspect their `title:`. | ~50 s | seek via the ATTACHed cUniverse schemas |
| 2 | **A dead field read from every file on every tree walk.** `extract_frontmatter_status` opened and read EVERY `.md` to produce `FileEntry.status` — which reaches exactly one consumer, which copies it into an object whose `status` appears in no markup. Measured on his 803-note library: **3.62 s with, 0.017 s without**. The walk runs TWICE per create and again inside the rename chain. | 3.62 s × 2 | field deleted |
| 3 | **Redundant syscalls in the cascade walker** — a `path.is_dir()` re-stat of an entry `read_dir` had already classified, plus an `exists()` that could not close the race it looked like it closed. | 752.7 ms | 2.3 ms |
| 4 | **The cascade reading 140.8 MB one file at a time.** | 8.3-8.5 s live | collect-then-rewrite in PARALLEL (rayon; `gate_rmw` locks per PATH, so nothing is shared) |

**Result, Boss-measured on the live app: note create 54 s → near-instant; rename 50 s → 14 s → 7-9 s
→ under 1 second.**

Two more fixed in the same pass, both mine from earlier today: `rename_folder_db_tail` held the
GLOBAL writer mutex across every descendant in ONE acquisition (19.79 ms × N — ~15.9 s of app-wide
DB blocking on an 800-note folder; now batches of 50 with the lock released between), and
`note_links_outgoing_au` had no WHEN clause so a `target_cid_cn`-only write re-ran four aggregate
subqueries per row (69.9 ms → 4.61 ms).

### What was deliberately NOT done, and why

**The index-driven cascade.** `SELECT DISTINCT source_path FROM note_links WHERE target_name = ?` is
an index seek — **1.8 ms against a measured 8.5 s walk**, and the median rename would open ONE file
instead of 2,105 (referrer distribution across 8,328 distinct target names: p50=1, p90=7, p99=31).
It is NOT safe yet: **290 rows store the anchor (`#`, 75) or the predicate head (`::`, 215) inside
`target_name`**, so a seek on the bare title would silently skip those referrers — the exact
invisible-miss class this migration's regex fix was written to close. Normalising that column is a
stored-derived-view + write-path change, i.e. its own `/migration`. Filed, not smuggled in behind a
speedup.

**The full-universe refresh after every create** (`cache_boot_snapshot_core`, ~0.7 s, 10,752 notes
over IPC): a boot-shaped refresh used as an incremental update. Shared with the universe-switch and
index-repair paths, so scoping it is a design decision, not a mechanical fix. Filed.

### The lesson this arc paid for — LL-037

Four defects in one session shared one cause: **I read a comment instead of the code beneath it.**
`"index_note uses DELETE+INSERT"`, `"this AU path is rare"`, `"the cascade now walks every library"`,
`"federated trees are small"`. Each was true when written. The last is the sharpest, because it
encodes a fact about the USER'S DATA that changes without anyone touching the code — and it turned
into a 54-second freeze the day a bigger universe was linked to a smaller one. The counter-example
is MIG-099's own comment, which carried the NUMBER ("13.6 s → sub-10 ms") and therefore pointed
straight at the mechanism the moment the symptom returned.

**Gates at close:** Rust **1410/0** · vitest **926/926** · svelte-check **0** · i18n **15/15**.

### Close-out note — one flaky gate, recorded rather than ignored

`tests/sight-v6/perf.test.ts` ("Hearst facet-count rebalancing … ≤32 ms") fails intermittently in a
full-suite run under machine load (observed 33.3 / 35.9 ms against its 32 ms ceiling) and passes in
isolation every time. It is a render-BUDGET assertion, so it measures the machine as much as the
code — and this machine spent the session compiling Rust at 93% load. Not a regression, and not
touched: raising a budget to silence a test is how a real regression gets through later. Recorded
here so the next person seeing it red does not go looking for a bug that is not there. Worth a
decision — either give it headroom with a stated reason, or make it measure work rather than
wall-clock.
