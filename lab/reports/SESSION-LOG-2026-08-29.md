# Session Log — 2026-08-29

**Branch `main`. Continues SESSION-LOG-2026-08-27.md (PJ-407 built and panel-cleared, awaiting the
Boss's test; the per-build safety inspection confirmed PJ-428).**

**Why this file starts at §19:** the numbering is continuous with 27 August on purpose — §16–§18
there and §19 here are one unbroken arc. Nothing was committed when the day turned, so the section
was filed under the date it actually happened rather than the date the work began.

---

## §19 — PJ-428 FIXED on the Boss's order, ahead of the test he was waiting on

**Boss: "Fix the High App-Killer first."** He overrode my file-don't-fold ruling. Fixed in three
parts; nothing else in the pass changed.

**Function in hand:** the walk/watcher fence contract — *is a library the user declared content of
this universe everywhere, or only in the one place that currently says so?*

### §19.1 — The gates could not run

The design panel I convened returned nothing: **all eight agents failed on the account's weekly
limit** (resets 04:00 Asia/Dubai). So this fix was designed and verified by me alone — no panel, no
second lens. That is a real gap in the record and it is stated here rather than left to be inferred.
What replaced it: every claim below was read in the shipping source, and every test was **proven RED
against the pre-fix code before being accepted as green.**

### §19.2 — Reading the code reframed the fix

Pass 2's missing fence turned out to be **documented as deliberate**: *"Pass 2 (deletes) deliberately
keeps its own shape and consults no library set: purging the row of a file that no longer exists is
correct in every scope, and it can only ever remove rows, never create them."*

That sentence is true of the ROW and false of the USER'S DATA — and the asymmetry with Pass 1 is the
entire defect. Pass 1 refuses to index a path inside a foreign universe; Pass 2 purges one. An
external rename there therefore destroys the row (and the earned link data `search.db` alone holds)
while Pass 1 declines to re-create it.

So the fix is not "add a fence" but an invariant: **what Pass 1 will not index, Pass 2 must not
purge.** And it is self-correcting for the case that SHOULD purge — if the nested universe folder is
itself deleted, its manifest goes with it, the check reads false, and the prefix-purge proceeds
exactly as before.

The same reading gave the second half for free: `walk_complete`'s own doc already says *"a sweep that
could not look must never report 'nothing changed'."* Reporting through that existing contract needs
**no new string in fifteen languages** and disables dead-row removal for the pass — the conservative
side to fail on.

### §19.3 — Three changes, three tests, all proven RED first

| # | Change | Test |
|---|---|---|
| 1 | `search.rs` Pass 2 carries Pass 1's fence | `both_passes_of_the_watcher_agree_on_what_is_ours` (RED: "found 1") |
| 2 | `reconcile.rs` `run` clears `walk_complete` when a declared library was fenced out | `a_declared_library_the_walk_could_not_reach_makes_the_sweep_incomplete` |
| 3 | `universe.rs` `link_library_as_universe` refuses to create the contradiction | `a_folder_inside_the_open_universe_cannot_be_made_into_one` |

Suite **1,598 / 0** (was 1,595; +3 is exactly these three — the same arithmetic check that catches a
stolen `#[test]`). The MIG-112 pin
`collect_md_skips_a_linked_universes_root_without_marking_the_walk_incomplete` still passes, which is
why change 2 sits in `run` and not in `collect_md`: an ordinary linked-universe root being skipped is
not a failure to look at OUR content.

### §19.4 — THREE bugs in my own verification, none of which would have failed

This is the part worth keeping.

1. **A test that could see itself.** `code_of` ran to EOF for a function with no `#[tauri::command]`
   after it, so the extracted "body" swallowed the test module and the assertion **matched its own
   literal.** Green forever. Second time this exact shape has appeared in this codebase.
2. **A fix for that which broke the search.** Truncating the FILE at the first `#[cfg(test)]` cut
   `search.rs` at line 495 while the function under test sits at 13573 — the test then panicked
   "not found". The body is now bounded at whichever marker comes first.
3. **A RED proof that edited the thing it was proving.** My revert used a blanket string-replace,
   which rewrote the test's own assertion to match the reverted code — so the test "passed" against
   code with no fix, and I nearly recorded that as evidence the test was sound. **A revert must
   touch production lines only.**

Every one of the three was caught by the same act: reverting the fix and re-running. None would have
been caught by reading, and none by a green suite. That is now three sessions running in which the
decisive check was *"could this method have disagreed with me?"*

### §19.5 — What was deliberately not done

The declared-library exemption still lives at ONE of ~23 fence sites. Content under such a library
still goes stale silently between repairs. Filed as **PJ-429**, because choosing between
"declaration wins everywhere" and "fence wins everywhere" reverses or extends a Boss-ruled MIG-112
contract — a ruling, not a bug fix. Still measured latent: 52 library entries across 8 universes,
zero meet the precondition, and change 3 now closes the door that could create it from inside the app.

---

## §20 — The PJ-428 review that could not run earlier: seven required changes, and my fix did not do what my comment said

The agent limit reset at 04:00, so the panel I had recorded as impossible was convened after the
fact — three lenses, every finding attacked by an independent refuter, 22 agents. **Verdict:
SHIP-WITH-CHANGES.** All seven applied.

### §20.1 — The central finding: a false causal claim in durable source, one layer deeper than before

My change 2 cleared `walk_complete` and its comment said that fixed *"the boot notice renders a clean
launch for content nobody looked at."* **I verified the panel's refutation myself rather than take
it:** `DriftReport::has_findings` (`reconcile.rs:191`) keys on five counters and deliberately
EXCLUDES `walk_complete`; the frontend's `indexDriftMessage` (`+layout.svelte:633`) returns `''` on
`!hasFindings(r)` **before** it ever reaches the `!r.walkComplete` branch. A fenced library
increments no counter. So the flag went false, dead-row removal was correctly disabled — and **the
user still saw a clean launch.**

The fix did not do the thing its own comment claimed, and PJ-428 would have been recorded as closed
on that basis. That is the 2026-08-25 law's exact shape — a sentence that matches its source
perfectly and does not match reality — and it is worse than the sentence that prompted the law,
because here the sentence described *my own change*.

Fixed properly: `fenced_libraries` is now a `DriftReport` field with its own `has_fenced()`, mirrored
in `driftReport.ts`, wired through six sites in `+layout.svelte` as a fourth notice row, with
`indexDrift.fencedLibraries` in all 15 locales (parity 3,694 ×15). Deliberately NOT in
`has_findings`: that band offers "Repair now" and the repair walker carries this same fence — the
"false door" this file already forbids twice.

### §20.2 — The fourth self-satisfying check, found where I was told to expect one

I briefed the panel to assume a fourth existed. It did, in my own test:

```rust
assert!(body.contains("!l.is_universe_notes"), "only a library the user DECLARED counts …")
```

`code_of` bounds the body at the first `#[cfg(test)]` (line 1224) while `fn run` ends around 955 — a
window over-reaching ~270 lines. `!l.is_universe_notes` occurs at line **461** too, in **pre-existing
MIG-112 code present at HEAD**. So the assertion was satisfied by code PJ-428 did not write and would
have stayed green with my filter deleted outright. Now anchored to `body[calc..used]`, and **proven**:
delete the filter, it fails.

The panel's sharper point stands and is recorded: **all three PJ-428 tests were source-text greps**,
so none could observe behaviour. A behavioural test now calls the shipping predicate with a CONTROL
that must answer the other way — without it, a fence that answered "foreign" for everything would
pass.

One assertion I added turned out not to be load-bearing and I am saying so rather than counting it:
`fenced_libraries: fenced_libraries.len()` cannot be removed without a compile error, because
`DriftReport` requires the field.

### §20.3 — A panel number I checked, and it was UNDERSTATED

It reported 3 of 13 registries carrying a drifted `universe_notes.path`. My first scan found **zero**
— because it walked only top-level directories, and the drifted ones are nested. The exact shape I
keep hitting: **a search that could not find its target.** Recursively: **7 of 17**, including three
`كون عيسى` copies under `Eisa Universe`, three backup snapshots — one a copy of the daily universe
carrying **18 libraries** whose recorded root names the LIVE directory — and one with doubled
separators. `own_root` now comes from `active_universe_dir` at both consuming sites; a drifted value
would make our own notes read as foreign and turn every guard built on it into a silent no-op.

### §20.4 — The other four

- **`mig108::norm_under`** replaces my hand-rolled path comparison, which omitted the NFC step that
  `ensure_under_active_root` applies three files away for a stated reason: NFC is live on Arabic
  names, and every universe here has one.
- **Both refusal strings rewritten.** Mine carried 18–22 literal spaces mid-sentence (a botched line
  continuation) and named two controls that do not exist — "New Library" and "Unregister"; the app
  says "Bring In a Library" and "Remove".
- **A liveness check** before the refusal: `active_path` is written at two sites and never cleared,
  so it can outlive its universe, and a refusal must never be issued in the name of a universe that
  is no longer registered. Fails open.
- **`create_universe` now carries the same refusal** (Whole-Ecosystem). It is reachable from the
  Universe Manager **while a universe is open**, whereas the door I had guarded is only reachable
  from the first-run setup screen. One concern, two doors — and I had guarded the less likely one.

### §20.5 — State

Suite **1,599 / 0**. `svelte-check` 0 errors. Parity 15/15 at 3,694 keys. Fallback literal
byte-identical to `en.json` (it was 316 characters against the canonical 377 — the stale-copy defect
again, caught programmatically rather than by eye). Sources 06:59–07:09 → bundle 07:14 → binary
**07:17**, nothing newer than the binary, the new string confirmed in three built chunks.

`ui-inspector` is reviewing the three new user-facing strings. Nothing is committed.

---

## §21 — The PJ-428 string: three inspection rounds, and each fix created the next defect

The new notice row went to `ui-inspector` three times. Recording the arc, because the *pattern* is
more useful than any of the individual fixes.

### §21.1 — Round 1, four findings, all correct

1. **The remedy was BACKWARDS.** The string told the user that removing the library from this
   universe would let Constellation read those notes again. I verified the refutation myself:
   `remove_library` (`libraries.rs:1202-1212`) only rewrites `libraries.json` and never touches
   `note_meta`. After removal the rows stay, the library drops out of `fenced_libraries` (so **the
   notice disappears**), and it stops being a walk root, so nothing revisits it. `phantom_prune`
   then returns `Keep("file still exists on disk")`, so it is not even offered for cleanup. The
   notes go from flagged-and-frozen to **silently frozen with no notice at all** — the string
   promised the opposite of what it does. A user-facing sentence that converts a visible problem
   into an invisible one is worse than the silence PJ-428 exists to end.
2. **The plural framing did not do what its own comment claimed.** I wrote that it copied PJ-407's
   technique. It did not: PJ-407 introduces its generic singular INDEFINITELY ("a file name that
   begins with a dot marks **the** file as hidden"), giving the definite article a local antecedent.
   Mine went straight to "a folder above **the** library" whose only antecedent was `{noun}` — plural
   at n≥2.
3. **"Removing that mark" was unfollowable.** No in-app control removes it (searched:
   `remove_universe_manifest`, `unlink_universe`, `strip_universe_manifest`, `demote_universe` — none
   exist), and the string never said what or where it was. PJ-407's sibling string gives a concrete,
   executable instruction; this one gave none.
4. **The guard I had just added was DEAD CODE.** `link_library_as_universe` is reachable only from
   `UniverseSetup`, shown only when no universe is registered or none could be activated
   (`+layout.svelte` 3618/3624/3631/3650/10765). In every such state the liveness check computed
   false and skipped the guard. **The review panel's own recommendation (R4) defeated the change it
   was protecting.** Both doors now ask the FILESYSTEM via `universe_manifest_at_or_above`, which has
   no registration dependency — the same question `create_universe` asks.

### §21.2 — Round 2: the fix for #3 created finding #5

Making the remedy concrete made it SINGULAR — "that universe.json", "the library". But
`fenced_libraries` is a `Vec`, and two fenced libraries can sit under two DIFFERENT ancestor
manifests. At n=2 the instruction clears one fence and leaves the other; the message returns saying
"1 library". A singular action promised against a plural state.

**The pattern worth keeping: each correction made the sentence more specific, and specificity is
exactly what broke the plural case.** Rewritten distributively — "In each case" makes every later
definite reference distribute over the count, and "Removing **a** library" is indefinite for the
same reason.

### §21.3 — What the inspector declined to claim, and why that mattered

It said plainly that it had NOT checked the 14 translations for the same scaling defect, suspected it
was structural, and refused to assert either way. It was right. The fix was therefore applied to all
14 as a REWRITE with the same distributive framing, not as a mechanical translation of the corrected
English — and the third round asks it to check Arabic, Japanese, Chinese and German at n=1 and n=2
specifically, those spanning dual/plural agreement, absent number marking, and article inflection.

An inspector that names the limit of its own check is worth more than one that passes everything.

### §21.4 — State

Suite **1,599 / 0**. `svelte-check` 0 errors. Parity 15/15. Fallback byte-identical at 567 chars —
and it had to become a DOUBLE-quoted literal, because the new text contains an apostrophe and the
escaped single-quote version broke the parse at column 544. Binary rebuilding against the corrected
string; the one on disk was built against the previous version and must not be used.

---

## §22 — PJ-407 BOSS-TESTED AND PASSED. PJ-431 found in the passing.

**Boss ran both steps and passed them.** Screenshots: the notice reading exactly "2 notes"; after the
rename the bar gone and the status bar showing **7494 → 7496 notes**; the sidebar listing **NET** and
**NET Framework**; and `.NET` open — a 3,521-word, 14-property note the app could not see that
morning.

**Verified afterwards what he could not see.** Running the app's own resolution join against his
index: `.NET` carries **16** incoming links and `.NET Framework` **11**. All 27 were dead before the
rename. (27, not 25 — the two notes link to each other, and their outgoing links only entered the
index once the notes became visible.) Files renamed cleanly: `NET.md` 23,262 B and
`NET Framework.md` 35,613 B. **No `NET.md.md`** — the trap the panel caught in the instruction did
not fire.

### §22.1 — PJ-431: the index and the file disagree about a note's identity, and nothing can see it

Checking the one claim he had not exercised turned up a second defect. Both notes carry
`cid_cn: 20260414T092241Z_NOTE_39F9` / `_45E6` in their frontmatter, while `note_meta.cid_cn` is
**empty** — and all 27 links hold `target_cid_cn = ''`. Confirmed against the live database **with
the write-ahead log replayed**, not an `immutable` snapshot, because the first reading could have
been a stale artefact and saying so mattered more than the finding.

The chain, read off the code:
1. Indexed while the file had no identity → `note_meta.cid_cn` is `''`.
2. `note_meta_sky_ai`'s propagation is guarded by `NEW.cid_cn <> ''` — correctly declines.
3. First open: `ensure_cid_cn` injects the identity **into the file**, through `gate_write`, which
   suppresses the watcher precisely so the app's own writes cannot re-trigger indexing.
4. Nothing updates `note_meta`, so `note_meta_sky_au` — whose own comment says it exists for "the
   lazy `cid_cn` injection on first open" — never fires either.

**Both triggers are correct. The write path never announced itself.** Fixed in
`canonical.rs::ensure_cid_cn_cmd`: re-index when the content actually changed, guarded so the
ordinary open path pays nothing (`ensure_cid_cn` returns its input unmodified for a note that already
has an identity). Test `stamping_a_notes_identity_also_tells_the_index` proven RED.

**The part that makes it worse, and it is measured:** both rows are **in step by mtime**, so the
drift check — the mechanism whose whole job is spotting index/disk disagreement — **cannot see this**,
because it compares timestamps, not content. It will not self-heal and would never have been
reported. The remedy that does work already exists and is named for exactly this case:
**Settings → Index → Full re-read**, whose own description reads *"Use this only if you suspect a
note's contents changed without its file being marked as changed."*

### §22.2 — The PJ-428 test: three inspection rounds, and a fifth wrong count

- **A third unguarded door.** I wrote "the only two ways the app can create that state" into his
  test. False: `migrate_legacy_data` writes a universe manifest at a caller-chosen path and had
  **neither** guard. Unreachable on his machine only because `check_migration_needed` requires the
  registry to be absent — a fact about a machine, not a property of the code. **Guarded**, and
  `neither_door_can_make_a_universe_inside_another_universe` now iterates all THREE names.
- **My fifth wrong count.** "8 universes, 52 library entries" did not reproduce. My script
  *approximated* the production predicate instead of mirroring it: it walked only top-level folders
  (missing three nested `كون عيسى` copies and `CE Test Universe\CE Test`) and counted the
  `is_universe_notes` roots that `reconcile.rs:642` explicitly excludes. Re-derived properly:
  **12 universe folders, 45 declared libraries, 0 trapped** — matching the inspector exactly,
  per-folder. The conclusion never moved; only the evidence for it was wrong.
- **An instruction that would have manufactured a false bug report.** My parenthetical said any
  folder inside the universe works identically. It does not: pick a subfolder and the refusal names
  the **root**, not the folder clicked — colliding with the test's own failure mode "*a folder named
  that is not the one you chose → that is a real finding*."
- **PJ-432 filed** — the first-run screen IS reachable in-app by removing the last universe, but that
  path skips the flush an ordinary switch performs. The test now gives that as the real reason for
  not walking him there, instead of the untrue "would need a separate first-run scenario".

### §22.3 — A finding I REFUTED, and why that direction matters too

The inspector reported the test's quoted refusal as misquoted, citing `universe.rs:1542`. **That is
`link_library_as_universe`'s message.** The test exercises `create_universe`, whose string at
`universe.rs:923` is character-for-character what the draft quotes — verified by reading both
functions. Acting on that report would have rewritten a correct quote into a wrong one and shipped a
false expectation **with an inspector's approval on it.**

Five of its findings today were right and cost me real defects. This one was not. The rule is the
same in both directions: verify the finding, not the wording — and a gate is not an oracle.

### §22.4 — State

Suite **1,600 / 0**. Binary **16:00:40**, bundle 15:53, newest source 15:38, nothing newer than the
binary, both notice strings confirmed in the built chunks. PJ-407 passed and ready; PJ-428 awaiting
its test. **Nothing committed.**
