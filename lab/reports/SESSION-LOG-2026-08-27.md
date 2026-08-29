# Session Log — 2026-08-27

**Branch `main`. Continues SESSION-LOG-2026-08-26.md (MIG-112 built, Boss-tested, committed
`890aae25`).**

**Why the section numbers start at §16:** these two sections were first drafted into the 26 August
log while the work ran past midnight. Nothing had been committed, so they were moved here before the
commit rather than left mis-filed — a future session looking for "what happened on 27 August" would
otherwise have found an empty file. The numbering is continuous with the 26 August log on purpose:
§13–§15 there and §16–§17 here are one unbroken arc of work.

---

## §16 — PJ-407 investigated and BUILT. The filed severity was wrong; the real defect was next door.

**Boss: "Proceed."** then **"How Obsidian approach similar issue? Lets find the right solution."** then
**"Approved."** Built, not committed — he tests every build first.

### §16.1 — PJ-407 as filed was WRONG about its own blast radius

The mechanism is real: `importers.rs` contains **zero** index calls, and `onImportComplete`
(`+layout.svelte:10329`) refreshes only the file TREE (a disk walk) and a read-only stats snapshot —
which is why an imported note appears in the tree and cannot be searched. Its own comment says
"surface the new notes", and it does surface them, in the one place that does not need the index.

**But measured on his machine, live exposure is ZERO notes attributable to it.** And there is a safety
net the filing missed: boot reconcile re-adopts orphans under a cap of `max(200, 10%)` ≈ **803** on his
daily universe, so an import smaller than that self-heals at the next launch. Only an import above the
cap could strand anything. **HIGH was wrong; it is a real gap with no current victim.**

### §16.2 — The real defect, found by measuring instead of accepting the filing

Orphan scan (`.md` on disk under registered libraries, minus `note_meta`):

| universe | on disk | indexed | orphaned |
|---|---:|---:|---:|
| `Eisa Cognitive Knowledge` | 7,976 | 8,031 | **2** |
| `Eisa Universe` | 2,112 | 2,112 | 18 → **4** genuine (14 are the MIG-112 nested universes, correctly excluded) |

Every one is **dot-prefixed**:
- `Computer Science\Algorithms & Data Structures\.NET.md` — **23,227 B**, real note, frontmatter + tags
- `…\.NET Framework.md` — **35,578 B**, likewise
- plus five 1-byte files literally named `.md` (junk, in `الكون المعرفي\Atlas\…` and a test universe)

**Cause, verified rather than inferred:** `search.rs:9041` (indexer) and `reconcile.rs` `collect_md`
both skip any entry whose name starts with `.` — applied to FILES as well as directories. So such a
note is invisible **twice**: never indexed, and never reportable as an orphan. **Invisible to the very
pass built to find invisible notes.** The panel had earlier refused this explanation as unproven; it is
now proven — the files are dot-prefixed and both guards skip them.

### §16.3 — WA#5 research: Obsidian, and why the instinct was wrong

My instinct was to narrow the guard to directories only. **The research says the opposite.** Obsidian,
on a thread now in its Bug graveyard: *"This is not a bug but a design decision. Files and folders
beginning with a `.` are intended to be hidden."* Asked what the fix would be: *"most likely preventing
users from creating .dot files."*

Narrowing would have been a ~30-surface change, a divergence from Obsidian (an explicit project value),
and would have started indexing things the user expects hidden. **The convention is right.**

And Constellation **already implements the fix Obsidian only intends to**: `note_display_filename`
(`libraries.rs:2776`) does `.trim_matches('.')`. Typing `.NET` as a title yields `NET`.

**So the entire defect was ONE line in ONE function where two doors disagreed** — the importer's
`sanitize_filename` handled `/ \ : * ? " < > |` and not the leading dot. The cheapest and most
conservative fix available, and only research found it.

**What Constellation adds beyond Obsidian:** their users' actual complaint was the SILENCE — files
vanished after a restart with no warning. That is the failure class this app exists to refuse.

### §16.4 — Built (three parts, as he approved)

1. **Convention stays.** No walker changed.
2. **`importers.rs::sanitize_filename`** now `.trim().trim_matches('.').trim()`, matching creation.
   Same function: **PJ-409 fixed in-pass** — `s.truncate(200)` panicked off a UTF-8 char boundary, and
   an import is exactly where Arabic titles arrive; a panic there aborts mid-loop with notes already
   written. Now walks back to a boundary.
3. **The notice.** `DriftReport.hidden_dotfiles` + `has_hidden()`, mirrored in TS; a THIRD
   `drift-note` row with its own dismissal, **no button** — a repair walks and re-reads and would skip
   these on the rule that hid them, so a control there would sit under a claim it cannot act on. Same
   precedent as the phantom row. Key `indexDrift.hiddenDotfiles` ×15.

**Collision path checked before it was raised:** every import site tests `target.exists()` and
**skips rather than overwrites** (`importers.rs:281`, `:325`, and siblings), so `.NET.md` and `NET.md`
colliding on one target cannot lose data — it counts as `skipped`. Pre-existing behaviour for any name
collision, not introduced here.

### §16.5 — THE WORST THING I DID TODAY: I silently disabled an existing test

My test insertion landed **between a neighbouring test's doc comment and its `#[test]` attribute**, so
the attribute bound to MY function. `collect_md_finds_orphans_skips_indexed` became an ordinary
function nobody calls — **the test covering the exact walk I was modifying.**

**The suite reported MORE tests passing than before** (1,592 → 1,593). Green throughout. Nothing looked
wrong.

**It was found only because I chased an intermittent failure instead of re-running until green.** My
fixture did `remove_dir_all` then `create_dir_all` on a fixed process-id path — a Windows race, failing
1 run in 3. Had that fixture been correct, the disabled test would have shipped and the loss would have
been permanent, silent, and invisible in the diff.

Fixed: attribute restored to its owner; my test moved after a complete test; fixture switched to
`TempDir`. **Verified by naming each test individually** (`1 passed` each) rather than trusting the
total — a total is exactly the check that could not disagree here.

### §16.6 — Verification

`cargo test --lib` **1,593 / 0 failed across FOUR consecutive runs** (no flake) · both neighbouring
tests confirmed running individually · `svelte-check` **0** · `i18n-parity` **3,693 ×15** · frontend
14:12 · binary **14:18**, chain verified against every source mtime · **safety inspection: 0 confirmed
findings** — the first clean pass over my own work today.

### §16.7 — Standing note for the next session

Three of today's defects came from **editing**, not designing: a test inserted into the wrong place, a
heredoc that mangled string literals, a fixture that raced. The reasoning about *what* to change held
up; the scripted text-replacement of Rust source is the weak link. Prefer targeted edits over generated
patches in this codebase, and after any test insertion **name the neighbouring test and run it by name.**

---

## §17 — The PJ-407 panel: six findings after a CLEAN inspection, and two false statements to the Boss

**Boss: "Proceed." / "Update" / "Proceed."** Still uncommitted.

### §17.1 — The inspection passed clean; the panel then found six things

Worth recording as calibration: `safety-inspection` returned **0 confirmed findings** on this diff, and I
reported that to the Boss as "the first clean pass over my own work today." The panel then found six
items **in the same code**. A clean pass from one gate is evidence that *one gate found nothing* — not
that the work is clean. I let it carry more weight than it earned.

### §17.2 — TWO false statements to the Boss, both mine

**(a) "Your 2 notes would have been reported as 4, and I caught it."** FALSE. `run` de-overlaps roots
before walking (`reconcile.rs:566-574`: skip any root nested under another), so `Computer Science` is
never walked separately and `.NET.md` is visited **once** regardless. The dedupe I added is DEFENSIVE,
not a production fix. **I presented it to him as an error I had caught, and it was an uncaught error of
the same species.** Corrected in the arm comment, the test doc, and to him directly.

**(b) "Renaming breaks nothing — no note links to `[[.NET]]`."** FALSE, and the reason is the session's
signature failure: I searched for `[[.NET`, the PLAIN wikilink form. **Constellation writes
`[[type::target]]`.** My search structurally could not find its target — it returned 0; the correct
search returns **27 notes**.

**Third instance of that exact class today**: a Latin-only key regex on a majority-Arabic corpus (missed
7 of 12 files); a dotfile check with `ls` (which hides dotfiles) offered as proof about dotfiles; and now
a plain-wikilink grep in a typed-wikilink app.

### §17.3 — But the inspector's CONCLUSION was also wrong, and the truth is better

It concluded a rename "will orphan these typed links in 26 notes." Verified against the index:

```
note_links  lower(target_name)='.net'            -> 15 rows
note_links  lower(target_name)='.net framework'  -> 10 rows
note_meta   lower(name)='.net'                   ->  0 rows
note_meta   lower(name)='.net framework'         ->  0 rows
```

**Those links are already dead** — no indexed note carries either name, because the notes are hidden.
(**Three figures, all measured, all true of different things** — and worth writing down, because
each was produced by an instrument answering a slightly different question:
**25** rows in `note_links`, because its `UNIQUE(source_path, target_name, link_type)` collapses a
repeat within one note (`Algorithmic synthesis` ×3 → 1, `Pascal (programming language)` ×2 → 1);
**28** occurrences in the text of those 25 notes — the inspector's figure, which counted the FILES
while I had counted the INDEX, and which I then reproduced; **38** occurrences universe-wide across
**27** files, because the two hidden notes carry **10** such links of their own (6 + 4) — and those
are in no index at all, since an unindexed note's outgoing links were never parsed. 38 − 10 = 28.
The test's sentence is scoped to notes pointing AT the pair, so it states 25 notes / 28 links.
The third figure was found only because a whole-universe grep was left running after I had already
"answered" the question from the database.)
Wikilinks resolve by **title** (orientation §6.7), and both files carry `title: .NET` /
`title: .NET Framework` in frontmatter, untouched by a FILE rename. So renaming makes the notes visible
while the link target is unchanged: **the rename REPAIRS every one of them, broken since those
notes were written.**

The cost of this defect was never "two notes are unsearchable". It was **every link into them silently
failing**, across a quarter of his Computer Science library.

### §17.4 — The other four panel findings, fixed

- **A user-visible string that disagreed with its own number and over-promised.** "{noun} … Rename **the
  file** … and **it** will appear at the next start" reads "2 notes … the file … it"; and "will appear at
  the next start" is untraced — re-adoption is capped and this app has logged skipping it. Rewritten ×15
  with **no pronoun reaching back to the injected noun**, checked at n=1 AND n=2, promising only what the
  code does. **Four gates had passed this string** — `ui-inspector` verifies text against source, and the
  source text was wrong; there was no gate for *is the sentence true of the number injected into it*.
- **The counter could state a falsehood.** It incremented without consulting `known`, and
  `reindex_md_descendants`'s `.md` branch has no dot guard, so a dot-named note CAN reach the index by
  the watcher path. Now gated on `!known.contains_key(&pn)` — same `norm()` derivation as `known`'s own
  population, verified. True by construction rather than by the state of his database.
- **A SECOND displaced doc block.** The PJ-207 §8 block had been transplanted onto my test, leaving
  `collect_md_skips_a_linked_universes_root_without_marking_the_walk_incomplete` undocumented — the same
  insertion error as the disabled test, **in the very pass that claimed the class was handled**.
  Re-attached; all three tests verified to carry their own doc AND their own `#[test]`.
- **The parity claim was over-stated.** `sanitize_filename` and `note_display_filename` diverge on
  reserved-char handling, the Windows reserved-name guard, and 200 vs 240-byte truncation. Only the dot
  rule is pinned. Test renamed `the_two_doors_agree_on_the_dot_rule` and the divergences documented.
- **A stale fallback literal.** `+layout.svelte`'s `tOr` fallback kept the OLD over-promising wording
  after all 15 locales were fixed — a second, quieter source of truth, reachable whenever the key fails
  to resolve. Now verbatim-equal to `en.json`, verified by substring match, with a comment recording that
  it survived an entire correction round.

### §17.5 — Verification

`cargo test --lib` **1,593 / 0**, all four affected tests confirmed **individually by name** ·
`svelte-check` **0** · `i18n-parity` **15/15** · sources 15:44 → bundle 16:48 → binary **16:52**, chain
verified · corrected sentence confirmed present in the built bundle.

### §17.6 — The pattern, stated for the next session

Every one of today's worst errors was **a check that could not disagree with me** — and three of them
were searches that structurally could not find their target. The remedy is not more care; it is asking,
before believing any negative result: *what would this search fail to see?*

Also: two of my three worst errors today were **statements to the Boss**, not code. The gates inspect
code. Nothing inspects the register except the next person to check a claim in it.

---

## §18 — The PJ-407 panel: 36 findings, 32 survived. The test would have passed while leaving the job undone.

**Five lenses, each finding then attacked by an independent refuter (42 agents).** The
`ui-inspector` had APPROVED the test. The panel found four things that would each have cost the
Boss a wasted round, one contamination of my own working tree, and three doors I had claimed in a
durable code comment were already closed.

### §18.1 — A review agent wrote a probe INTO my source file, and I nearly committed it

`importers.rs` was 1,159 lines; the authored diff ended at 1,111. Lines 1113–1159 held
`mod tests_pj407_refute_l3_05` — three `#[test]` functions full of `println!` probes, written by a
refuter agent during an earlier round and left behind. Earlier variants of it carried `assert!(false)`
and a syntax error.

It compiled, so nothing failed. `cargo test` counted it as passing tests. **The suite was green
with a foreign agent's scratch code inside a source file staged for commit.** This is the second
time in two days that a test-count went the wrong way without failing: on 26 August my own insertion
stole an existing test's `#[test]` attribute and the suite reported MORE tests passing.

The lesson is the same both times and it is not "be careful": **a green suite is not evidence about
what is IN the file.** Removed; restored to 1,111 lines.

### §18.2 — "This one line is the whole defect" was false, in a durable comment

The first pass fixed `sanitize_filename` and wrote, in the code: *"Constellation already prevents it
at creation. **The importer did not**"* and *"This one line is the whole defect."* Three lenses
independently extracted `copy_full_tree`, compiled it, and ran it: **a folder containing `.NET.md`
lands `.NET.md`.** That function routes `markdown | folder | bear | obsidian` — and `obsidian` is
the default selection in the first-run Universe Setup wizard, so it is the likeliest import a new
user ever performs.

A fourth door was open too: `universe::sanitize_template_stem` trimmed **trailing** dots only, so
"Save as template" under `.Draft` wrote `.Draft.md`, the picker's walker skipped it, and the command
returned `Ok` — a silent false success on a write path.

Fixed in this pass: `unhide_md_leaf` (new, narrow, tested) and the template stem. Filed as
PJ-420…PJ-425: note rename, folder rename, New Folder, New Library, the daily-note / quick-capture
folder fields, and the counter's dot-directory gap — each needs a *strip-or-refuse* decision plus
fifteen locale strings, which is the WA#6 exemption, and it is only honest because the false
comments came out in the same pass.

**Why the comment mattered more than the code.** The code was incomplete; the comment made the
incompleteness *invisible to the next reader*, including me. That is the 2026-08-25 law's exact
shape, one day after it was written.

### §18.3 — The test would have PASSED while leaving two wrongly-named files

`HideFileExt = 1` in his registry (verified). Explorer therefore renders `.NET.md` as `.NET`. The
instruction said *rename to `NET.md`* — typing that produces **`NET.md.md`** on disk. And because a
note's name comes from frontmatter `title:`, `NET.md.md` still indexes as `.NET`, the links still
resolve, and the bar still clears: **all three stated post-state signals pass.** Rewritten to
"press Home, delete only the leading dot, do not retype".

Also caught: the test told him 25 links were dead and never told him not to click one.
`handleLinkClick` (`NoteEditor.svelte:765`) creates a missing note **in the source note's folder**,
and `note_display_filename` strips the dot — so one curious click creates `NET.md` and blocks the
rename. **22 of the 25 source notes live in that very folder** (verified by query).

### §18.4 — Two panel claims I refuted before they reached him

The panel is not an oracle either. Two of its instructions were wrong and I checked rather than
relayed:

1. **"It will open كون عيسى — the only universe in the list."** I declined to assert it. My
   `%APPDATA%` may be redirected into the Claude package's `LocalCache`, and the instrument I first
   reached for — comparing the two paths' file IDs — **cannot disagree with itself** when both go
   through one redirect. Contradicting evidence: the file was last written 7 August, yet the app
   demonstrably ran against `Eisa Cognitive Knowledge` on 26 August, and both registration doors call
   `save_registry`.

   **The question WAS answerable, and the inspector answered it with the right tool.**
   `fsutil hardlink list` enumerates the hardlinks of a single NTFS file record: it returned the
   `LocalCache` path FOR the `%APPDATA%` path, proving one record, not two copies. So the registry
   genuinely holds one entry (`كون عيسى`), and **`Eisa Cognitive Knowledge` is not registered at
   all** — the app reaches it by another route. My hedge was right to refuse the claim and wrong to
   treat it as unanswerable: the lesson is not "cannot be known", it is *that comparison could not
   have told me either way, so find one that could.* The test's route works regardless, so it stands
   unchanged.
2. **"Constellation writes an identity line into each file by itself at a following launch, whether
   or not you open them."** REFUTED. `mig003_backfill_cid_cn` is gated on
   `stored_note_meta_version < NOTE_META_SCHEMA_VERSION`; his live `schema_versions.note_meta` is
   **1** and the target is **1**, so it short-circuits. Every other caller of `ensure_cid_cn` is the
   note-OPEN path. Corrected to "the first time you *open* either note" — a claim about writing into
   his own notes is not one to relay unchecked.

Also corrected: the panel's "37 notes under two `.trash` folders". A direct `find` says **94** (73 +
21). Recorded in PJ-425 with the reason the two numbers differ.

### §18.5 — Also fixed this round, found by looking rather than by the panel

`docs/User Manual.md:321` still carried the **refuted cause** corrected in the app ×15 the day
before: that a note leaves no delete record because Constellation *"never gave it an identity of its
own."* False for all 8 — a duplicate had claimed it first. Corrected, then verified against the code
rather than against the string it now matches (`search.rs:~13154` returns an empty record when
`cid_cn` is blank, and logs the skip). The 14 translated manuals do not contain this chapter at all
(~1,635 lines against English's 2,687) — that gap is PJ-394's, not this pass's.

**PJ-409 closes with PJ-407**: the old `sanitize_filename` called `String::truncate(200)`, which
panics off a UTF-8 char boundary — an import is exactly where non-ASCII titles arrive, and the panic
aborts the command after some notes are already written. Fixed with a 300-Arabic-character
regression test.

### §18.6 — The pattern, restated because it did not change

Yesterday: *every one of the worst errors was a check that could not disagree with me.* Today the
same shape, twice more — a green test suite that could not see foreign code in the file, and a
file-ID comparison that could not see a redirect. Both times the instrument was answering a
different question from the one I was asking it.

The standing question stays: **if my method were wrong, would this result look different?**

### §18.7 — The per-build Safety Inspection: one confirmed HIGH, and it is not in this diff

Run diff-scoped per the standing order over the five changed files. **One CONFIRMED finding, HIGH.**

Attribution first, because it decides the response: my `reconcile.rs` hunks are at 128 / 197 / 242 /
264 / 592 / 1051 / 1359. **The finding is at 539** — MIG-112 code, committed yesterday in `890aae25`.
The other half is in `search.rs`, which this pass does not touch at all. So it is pre-existing
relative to this diff, and the 2026-08-25 precedent applies: **file, don't fold in.** Fixing it would
also alter the very fence the Boss validated yesterday, un-testing it.

**The contradiction** (filed as PJ-428): MIG-112 exempts a REGISTERED library from de-adoption when it
sits behind a universe manifest — *"an explicit declaration beats a filesystem inference."* Nothing
else carries that exemption, so the rows are kept and never refreshed, and because the subtree is
never walked, `has_findings()` is false and **the boot notice reports a clean launch.** The signal
built to surface staleness structurally cannot see it. Pass 2 of `reindex_changed_paths` has no fence
at all, so an external rename purges the row along with `weight`, `traversal_count`,
`last_traversed`, `confidence`, `status`, `review_priority` and the review schedule — data
`search.db` alone holds, and no walk can regenerate.

**Measured latent before deciding anything:** 52 library entries across 8 universes, **zero** meet the
precondition. Including the one library that sits outside its root (`Constellation Test →
Ideaverse Pro 2.5`) — no manifest anywhere in its chain. Nothing is degrading while it waits.

**One claim of the finding was refuted during its own verification**, and the correction is kept so
nobody re-forms it: the subtree is NOT frozen indefinitely. `reconcile_filesystem` starts
`index_library_recursive` AT `lib.path`, and `is_walk_boundary` is only applied to child entries
during descent — the helper's own doc says never to call it on the walk's own start root. A
user-triggered Repair does reach and heal the content staleness. It does not dissolve the finding: the
repair is never offered, and the purged earned data is beyond any walk.

**Note on what this says about the inspection itself.** MIG-112 shipped yesterday with its own
per-build inspection, which did not surface this. It surfaced today only because `reconcile.rs` was in
a second diff for an unrelated reason. That is the per-cycle sweep's job, and it is an argument for
running it at the close of this cycle rather than trusting the per-build pass alone.

---

**PJ-428 continues in `SESSION-LOG-2026-08-29.md` (§19).** The Boss ordered it fixed ahead of the
PJ-407 test; the work ran past midnight into 29 August, and the section was moved before anything was
committed — the same correction made earlier in this session for §16–§17.
