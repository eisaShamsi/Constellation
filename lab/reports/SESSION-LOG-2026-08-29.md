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

---

## §23 — PJ-435 Phases 0–1 built (2026-08-30). The earned layer handled first, on the Boss's order.

**Boss: "Handle the 'earned.jsonl' first, then proceed with the build."** Done in that order.

### §23.1 — Phase 0: the earned layer proven through the SHIPPING function

`store_dir` derives the store from the DB connection's own path (`link_life.rs:79`), so the restore
is path-independent **by construction** — and now by test:
`the_earned_layer_survives_the_universe_moving` physically renames a whole universe directory
between seed and restore, models the true post-rebuild state, and restores through the shipping
`link_life_restore::restore`. It carries a CONTROL (a restore aimed at the OLD location must find
zero records) and pins the two measured casualties as executable assertions: a link's `created` does
NOT come back, and the `review_schedule` row stays orphaned at the old absolute path — with a message
telling whoever closes those gaps to update the CLAUDE.md storage section in the same commit.

**The test's first run failed usefully** (`records=1, restored=0`): my fixture left `note_meta` at
old paths while the fresh link used new ones — an inconsistent state matching no real scenario. The
failure demonstrated the matching is genuinely identity-first (cid → CURRENT path), and is kept in
the test's comment.

Also: the CLEAN fixture reproduces on demand (`E:/pj435fx` — a real 30-note universe, moved; all 30
rows stale, all 30 files present; 96 link birth dates and 30 path-keyed review rows at stake).

### §23.2 — Phase 1: one detector, the pair persisted, the duplicate killed

`universe::heal_paths_after_move` replaces the two copy-pasted healing blocks
(`set_active_universe`, `open_existing_universe`) that computed the old→new pair on every activation
and threw it away. Disciplines, each pinned by a behavioural test against the shipping helper:

1. **A move arms**: pair returned AND persisted to `.constellation/relocation.json`, written
   **before** healing (healing destroys the only evidence) and verified by read-back — persist
   failure leaves `libraries.json` unhealed so the condition stays detectable next boot.
2. **A copy never arms**: old root still on disk → heal only. A "repair" offer on a copy would
   rewrite a healthy index toward the copy.
3. **A second move chains**: `old_root` stays the ORIGINAL (the index still carries that prefix);
   only `new_root` advances.
4. **Moving back home disarms**: degenerate pair → record deleted.
5. Member libraries still re-pointed by folder name (the old behaviour, preserved verbatim).

**Two of the five tests caught a real bug in my first version**: the persisted record chained
correctly while the RETURNED pair did not, and a move-back still returned armed. A caller acting on
that return value would have rewritten a prefix the index never held. Fixed in the helper, not the
tests.

**The duplicate-entry bug is dead**: `open_existing_universe` now re-points a registry entry whose
(name, created) match the manifest and whose recorded path is gone — mandatory repoint, per the
review chair's warning that identity-match without repoint converts a duplicate-entry bug into
cannot-open-at-all. The Boss's own list carried two such ghosts on 2026-08-29.

### §23.3 — State

Suite **1,606 / 0** (was 1,600; +6 = exactly the six tests added). Phases 2–4 next: the honest
notice (suppressing the misleading "8,033 notes are not in the search index" row while a move is
armed), then the repair engine off the live app. Nothing committed; the Boss tests first.

---

## §24 — PJ-435 Phases 2–3 built (2026-08-30): the honest notice, and the safe repair on one click

### §24.1 — Phase 2: the notice that REPLACES the alarm

`DriftReport.moved: bool` (a bool deliberately — the struct is `Copy` and stays so; the old→new pair
travels via a new `get_relocation_record` command, invoked only when armed). While `moved` is armed
the frontend **suppresses the drift row and the phantom row**, dismissed or not: their numbers are
true (8,033 rows look stale; after a move the old-path rows even classify as phantoms), their
impression is false, and each carries a button that is destructive on a moved universe. The panel's
ruling is quoted at the suppression site. A fifth `.drift-note` row shows the honest sentence with
both folders named — `indexDrift.universeMoved` ×15, fallback verbatim.

One svelte-check catch worth keeping: negating the `hasMoved()` **type-guard** narrowed `r` to
`null` for the rest of each suppressed body — nine `never` accesses. Suppression tests the raw
flag; the guard stays for positive use only.

### §24.2 — Phase 3: the engine, borrowed whole, isolated at three points

`relocate.rs::repair_moved_universe` drives `mig108`'s proven rewrite with a ONE-ENTRY journal —
verified backup first, one transaction, conservation proved inside it, the eight JSON stores after.
Three deliberate isolations, each pinned by a test:
- **its own journal file** (`relocation-journal.json`) — new optional `Journal.journal_file`,
  serde-defaulted so every existing journal loads unchanged. Landing in `mig108-journal.json` would
  make the boot resume machinery present a crashed relocation as a half-finished unification.
- **its own backup dir** (`relocation-backup`) — `take_snapshot` now takes the dir name (11 call
  sites updated). Unmodified it would have rotated the Boss's 1.9 GB `mig108-backup` aside.
- **no move phase** — the OS moved the folder; the journal enters at `Phase::Moved`.

Post-run it recreates the dropped sky triggers via idempotent `init_db` — the same post-commit step
`mig108_execute` performs, which the engine's own comments require of every command layer.

**The engine change the review panel made a condition of use: the destination purge is now
CONDITIONAL.** A dest row whose old-path counterpart exists in `note_meta` is crash-window junk —
purged; a dest row with NO old counterpart is the only copy of something real — spared, and counted
in the in-transaction baseline so conservation covers it. This also fixes a latent mig108 defect: a
note genuinely created at the destination during a crash window was previously deleted. Test
`the_destination_purge_spares_rows_with_no_old_counterpart`, proven RED against the unconditional
purge (the "only copy" assertion fails exactly as predicted).

**Crash story = re-click.** A crash after the DB commit leaves the record armed; the second click
finds nothing stale, and the conditional purge spares everything — idempotent by construction.

### §24.3 — The self-matching trap, caught BEFORE first run this time

My relocate isolation test asserted `!src.contains("run_move_phase")` — and the assertion's own
string literal (plus the module doc) contains that token, so the test was green forever. Third
instance of this exact shape in this codebase; the first two were found only by reverting a fix.
This one was caught before the test ever ran: truncate at `#[cfg(test)]`, build the forbidden token
with `concat!`.

### §24.4 — State and the honest gap

Suite **1,608 / 0** (1,600 → +8: move-survival, 5 detection, conditional purge, relocate isolation).
`svelte-check` 0. Parity 15/15 ×3 new keys. **Gap stated rather than glossed:** the DIRTY scenario
(self-healed rows at the new root surviving the repair) is proven at the unit level by the purge
test; a full end-to-end through `repair_moved_universe` needs an AppHandle harness and is NOT
covered. The Boss's test exercises the CLEAN end-to-end on a real moved universe.

### §24.5 — A law deviation recorded against myself, and the gate restored

**PJ-428's Boss test skipped the panel wrap.** The Panel-Speaks-First law says test material runs
auditor → inspector → **panel** → Boss; PJ-428's went to him after the inspector rounds alone. He
passed it and nothing in it was wrong — but that is outcome-luck, not compliance, and the one test
that DID get the wrap (PJ-407) had four findings surface there that two inspector rounds missed.
Recorded here because a deviation that works is the most dangerous kind: it teaches the shortcut.

PJ-435's Stage-1 test is going through the full gate: auditor (self-verified the binary) →
inspector ×2 (rejected once on a single character; surfaced the active-universe hand-off; traced the
1-note self-heal race and ruled the banner honest, caveat deferred to Stage 2) → panel wrap (three
lenses: false-pass/false-fail routes, the Boss's round minute-by-minute, every factual claim
default-refuted) → him.

### §24.6 — The panel wrap earned its convening: DO-NOT-SEND, one CODE defect, eight edits

The wrap ruled DO-NOT-SEND on a draft two inspector rounds had approved — and the decisive finding
was CODE, not prose: **the mount-read fallback set `indexDrift` without fetching the relocation
pair**, so a cold boot into an armed moved universe that missed the event rendered NOTHING — no
honest banner (gated on a null `movedInfo`), no old alarm (correctly suppressed on `moved`). A
silent armed state, on exactly the relaunch the test's own hand-off paragraph invites; the Boss
would have reported "detection did not fire" against working detection. Fixed with one shared
`syncMovedInfo` helper called from both the event handler and the fallback; rebuilt (binary
2026-08-30 07:25:48, chain verified).

The wrap also **overturned the inspector in the stricter direction** on the self-heal race: the
report is netted and emitted only AFTER the re-adopt step, so on a 1-note fixture the index has
ALWAYS healed by the time the banner renders. The draft's "not covered" section was inverted and is
rewritten: Stage 1 proves detection; Stage 2 proves the rails; the mass rewrite is automated-only,
with a bigger-fixture decision deferred to the Boss at Stage 2. And it caught the oldest trap on
file: the INSTALLED exe is from June — Step 0 now mandates launching the fresh build by path
(`feedback_verify_binary_before_testing`, the 3-lost-hours memory, nearly relived).

Chair adjudications kept for the record: the "~10 s on your daily universe" clause SURVIVED (its
refuter sourced the 37-boot measurement); the "8,033" figure was cut from the intro (he never lived
that alarm, and his status bar reads ~7,496). **Stage-2 obligation carried forward: decide the
instrument — a >200-note fixture above the re-adopt cap so the rewrite engine runs live, or
rails-only — and pin which self-heal branch fires on the running app, before Stage 2 is drafted.**

---

## §25 — PJ-435 Stage 1 BOSS-PASSED (2026-08-30), and the fixture reproduced the panel's predicted defect class LIVE

**All eight steps passed on his screen.** The banner rendered verbatim with both paths, the safe
button in place, no false alarm, no double-banner. He stopped exactly at the line, did not click
repair, did not dismiss — and closed the app on my recommendation (frees the binary for any
gate-forced rebuild; freezes the fixture; turns his relaunch into the first live observation of the
cold-boot `syncMovedInfo` fix).

### §25.1 — The number that did not add up, and what it turned out to be

His Step-8 screenshot read **3 notes** where 2 were expected. Read directly from the frozen DB
(immutable, WAL-safe):

| row | cid | state |
|---|---|---|
| `PJ435 Note` @ NEW path | stamped | correctly RELOCATED by the cid self-heal |
| `Observation — Recent Captures` @ NEW path | **empty** | a fresh DUPLICATE — the self-heal is cid-gated |
| `Observation — Recent Captures` @ OLD path | empty | the stale original, still present |

The Five Acts scaffold note is created WITHOUT a cid — so the very first live move of a real
universe reproduced **exactly** the stamp-less-duplicate class the review panel predicted from the
27 cid-less rows on the daily universe. Predicted on paper 2026-08-29; reproduced on screen
2026-08-30, on a two-note fixture.

**And the conditional purge is precisely its cure**: the new-path duplicate has a surviving old
counterpart → junk by the rule → purged, the original remapped in. Stage 2 therefore carries a
falsifiable, user-visible prediction sharper than anything designed in advance: **the repair click
must take the status bar from 3 notes to 2.** 3 unchanged or 1 = the purge misbehaved.

This also UPGRADES the honest-gap statement from §24.4: the DIRTY scenario (mixed self-healed +
duplicated rows through the full command) is no longer unit-proven only — his Stage-2 click IS the
live DIRTY end-to-end, on a state measured before the click.

### §25.2 — Frozen baseline (stage2_pre_small.json) and the Stage-2 instrument

Small: 3 note_meta rows (1 old-path, 2 new-path), 3 review rows, relocation
{E:\PJ435 Test → E:\PJ435 Moved\PJ435 Test}. Big: `E:\PJ435 Big` built — 500 notes, Alpha/Beta,
~1,000 typed links with created dates, unique token `blorvath` in `Alpha\Big Note 250.md`
(my generator's own log line said Beta; the code's even/odd split put it in Alpha — caught by
grep before it reached any test). 500 > max(200, 10%) so every self-heal valve stays shut and the
mass rewrite is the only path. No search.db yet — his baseline open indexes it at the original
location, which is what makes the later move real. Post-repair verification runs against the
engine's OWN backup (relocation-backup), so "keeps everything" is proven, not asserted.

### §25.3 — The Boss's PC crashed mid-rebuild, and the design passed a test nobody scheduled

Hard power loss + reboot while `cargo build --release` was compiling. Full damage sweep, most
valuable first: **his daily universe — integrity ok, 8,033 rows** (he had closed the app before the
crash, so the DB shut cleanly); the frozen fixture — ok, exactly 3 rows, relocation record intact;
the 500-note fixture — all present; sources with both new fixes — intact; git tree — coherent.
**Sole casualty: one compile.** Suite re-ran clean post-crash (1,608/0 — cargo's incremental state
survived), rebuilt: binary **17:39:51**, chain verified.

**The accidental result that matters:** the armed relocation record survived a genuine crash. The
durability I had recorded as automated-only ("crash-resume is covered by tests, not by anything you
can click") just happened live, unscheduled — Stage 2A's Step 0 banner is now persistence proven
through a real power loss, not a polite quit. Written into the test itself.

### §25.4 — The auditor found a defect while DRAFTING, and the fix sharpened the test

Building Stage 2, the `tutorial-auditor` traced the status-bar note count and found **the repair
never refreshes it** — the figure is fed by `loadAllStats()`, wired to boot / add-library /
universe-switch only. Post-repair the bar would keep saying 3 until a relaunch, and the draft had
grown a relaunch step explaining the stale number away — a Boss test carrying a workaround for a
defect discovered an hour earlier, which is exactly what WA#6 forbids. Fixed (one line in
`startMovedRepair`'s success path, after the awaited invoke — DB committed by then), rebuilt.
Step 2's prediction is now sharper: **click → count 3 → 2 in place**, and Step 3's relaunch proves
the repair STAYS done instead of explaining a lag.

The failure ladder given to the Boss maps outcomes to meanings honestly: still 3 = the purge did not
do its second job; 1 = data loss, the serious case; 2 = correct. The inspector has been asked whether
a FOURTH outcome exists that the ladder omits.

---

## §26 — PJ-435 Stage 2 BOSS-PASSED at both scales (2026-08-30). "Keeps everything" measured, not asserted. CLOSED.

### §26.1 — Stage 2A (the small universe): four steps, four passes, the 3→2 prediction held

Cold launch → the banner already waiting (persistence through a real quit AND, courtesy of his PC
crash, through a hard power loss). Repair click → the status count folded **3 → 2 without a
relaunch** — the frozen fixture's falsifiable prediction, exact: the stamp-less duplicate of the
built-in Five Acts page was recognised by identity and merged; his own note untouched. Relaunch →
no banner, count stays 2. `zarquon` → one result, opens.

Offline verification (all through findings-verifier, CONFIRMED): `relocation.json` deleted,
`relocation-backup\` + `relocation-journal.json` present, note_meta exactly 2 rows both at the new
root, review rows re-addressed, receipt in `diagnostics.log`:
`[relocate] PJ-435: index repaired after a universe move: E:\PJ435 Test → E:\PJ435 Moved\PJ435 Test (2 notes; backup in relocation-backup)`.

### §26.2 — Stage 2B (501 notes): the scale where the repair is the ONLY path — passed

The panel-predicted **501** count held at every checkpoint (500 fixture notes + the app's own
`Five Acts\Observation — Recent Captures.md`, planted by `system_notes::init_at` on first open —
the same built-in page that was 2A's duplicate; one mechanism, both fixtures). `blorvath` unique at
baseline and after. The banner at scale with **all 501 rows intact** — the `max(200, 10%)` self-heal
caps refusing exactly as designed (`[reconcile] 501 orphan files (> cap 200) — skipping re-adopt`).
Repair passed; negative control silent on Eisa Cognitive Knowledge; registry cleanup done on-screen.

**The core promise, verified against the repair's own backup (findings-verifier, 7/7 CONFIRMED):**
- **1,000 of 1,000 link `created` dates byte-identical** to `relocation-backup\search.db.pre-mig108`,
  keyed per link (`UNIQUE(source_path, target_name, link_type)` guarantees the keying). Precision
  note carried verbatim from the verifier: 998 of 1,000 row ids differ (re-inserted by the
  post-repair re-index); ids 1001–1002 belong to the one pre-stamped note (`Alpha\Big Note 250.md`)
  and survived in place.
- **501 of 501 review_schedule rows re-addressed** (backup: all 501 at the old root; live: all 501
  at the new, zero at the old).
- Record deleted, backup + journal present, receipt logged.

### §26.3 — The Boss reported "about 95 seconds," and the log answered it exactly

He reported the one number outside the tutorial's bound, precisely as asked. Forensics from
`diagnostics.log` (verifier-confirmed): the post-repair `init_db` block spanned **92 s
([1788105714] → [1788105806])**, of which **91.6 s** was
`mig003_step3_soft_rebackfill: stale=500 … injected=500 … elapsed=91.6147082s` — the pre-existing
MIG-003/PJ-153 identity-injection pass, finally able to reach the 500 generator-made notes that
carried no `cid_cn` (backup db: 500 of 501 rows empty; live: 0). The PJ-435 path rewrite itself fits
in the remaining ~3 s. Corroboration that could have disagreed: the pre-repair boot ran the same
pass against the stale paths in **623 ms with injected=0** — the rewrite *enabled* the 91.6 s, it
did not *cost* it.

**Scaling, measured not assumed:** Eisa Cognitive Knowledge has **27 of 8,033** rows with empty
`cid_cn` — 14 templates (exempt before any file read) + 13 candidates (1 in `.trash`, 10 in
`3mooR`, 2 in Algorithms & Data Structures). Worst case ≈ seconds, paid at an ordinary boot anyway
(the pass runs in every `init_db`; steady state is an indexed probe + early return, search.rs:4317).

### §26.4 — Small findings filed, not shipped-silent

- **PJ-439 (cosmetic):** `take_snapshot`'s backup filename is hardcoded `search.db.pre-mig108`
  (mig108.rs:564) even inside `relocation-backup\` — misleading name in the relocation flow.
  Renaming touches restore instructions → separate small job, not a reopen of a passed build.
- **PJ-438 datum not captured:** the Boss did not report whether Step 5 showed a progress strip;
  the which-route-wins question stays open with no new evidence.
- The one pre-stamped note being exactly the note he opened in Step 6 is *consistent with* the
  PJ-431 on-open identity write doing its job — recorded as an observation, not a verified cause.

### §26.5 — State

All three stages Boss-passed. Verifier 7/7. Diff-scoped safety inspection run before commit (result
recorded below). CLAUDE.md storage section amended in the same commit, as its own text ordered —
the MOVE exposure of `created` + `review_schedule` closed; the REBUILD exposure remains, pointed at
PJ-437. Fixture folders deleted after verification. **PJ-435 CLOSED.**

---

## §27 — The pre-commit inspection earned its standing order: 8 findings AFTER the Boss's pass, all in the new code, all fixed RED→GREEN (2026-08-30)

§26 was written as a close; the mandatory diff-scoped safety inspection then returned **8 confirmed
findings (2 HIGH, 3 MED, 3 LOW — six distinct defects after dedup), every one in the new PJ-435
code, every one on an edge path the live test never exercised.** The close was premature by exactly
the amount this workflow exists to catch. Per WA#6 nothing ships logged-and-unfixed; the fix pass
ran before the commit.

### §27.1 — The register, and what each fix is

1. **HIGH — foreign record aims the rewrite at another universe** (relocate.rs + universe.rs):
   a COPY of a moved-but-unrepaired universe inherits `relocation.json` whose `new_root` is the
   SOURCE folder; the repair consumed it verbatim → full rewrite toward another universe's living
   root, reported as success. **Fixed twice over:** activation now deletes any record that is not
   OURS (parsed `new_root` ≠ this root), and the command refuses + removes a record describing a
   different folder.
2. **HIGH — unreadable record = total notice blackout** (reconcile.rs + universe.rs): `moved`
   armed on bare `.exists()` suppressed the drift and phantom rows while the moved row — needing
   the parsed pair — rendered nothing; the safe repair sat unreachable inside the unrendered row.
   **Fixed:** `relocation_armed()` parses, never exists-checks; activation deletes an unreadable
   record so the honest drift report resumes.
3. **MED — stale in-memory stores clobber the repaired JSON** (+layout.svelte): the repair rewrote
   collections/bookmarks/workspaces/session on disk while the window's stores still held old-path
   lists; the next whole-list save would silently undo the repair for that store. **Fixed with the
   proven pattern:** post-repair `window.location.href = '/'` — the same reload-through-boot the
   mig108 unification flow uses. (One visible change for the Boss: the repair now ends in a brief
   app reload instead of an in-place banner fade.)
4. **MED — non-atomic record write** (universe.rs): the one manifest in the file written with bare
   `fs::write`; a torn write is what creates finding 2. **Fixed:** `atomic_write` + the existing
   read-back proof. The chain-read's silent `.ok()` fallback now logs when an existing record
   cannot be parsed.
5. **LOW×2 — swallowed disarm + backup rotation on re-click** (relocate.rs): `let _ =
   remove_file` left the notice armed forever with false text; and each full re-run rotated the
   backup generations — a third click would have deleted the only genuine pre-repair backup.
   **Fixed:** `disarm_relocation` clears a read-only attribute and reports; the **already-repaired
   fast path** (journal at `JsonRewritten`/`Done` + zero rows under the old prefix) disarms
   without re-running the engine, so a re-click can never rotate the backup.
6. **LOW — post-repair init_db failure only eprintln'd** (relocate.rs): triggers stayed dropped
   for the live session with no record anywhere. **Fixed:** retry once, then a loud diagnostics
   receipt naming the consequence. The repair receipt now also carries the remapped-row count
   ("N rows remapped" / "0 — nothing needed remapping"), so a wrong-pair no-op can never read as
   a mass repair.

### §27.2 — RED→GREEN, the discipline held

The two reproducible defect shapes were pinned as failing tests BEFORE the fixes: 
`a_foreign_record_inherited_by_a_copy_is_removed` and `an_unreadable_record_is_removed_at_activation`
both RED on the unfixed detector (run recorded), GREEN after — alongside the regression guard the
fix must not break (`an_armed_record_for_this_folder_survives_the_next_boot`, green throughout —
the legitimate armed-and-declined record between boots survives) and
`disarm_clears_a_readonly_attribute_and_removes_the_record`. The relocate isolation test gained two
structural pins: the record-must-describe-this-folder guard and the fast-path condition. Suite:
**1,612 / 0** (+4 = exactly the four tests added).

### §27.3 — What this changes about the close

PJ-435's Boss-pass covered the happy path and stands. The fixed build differs in one observable
(the post-repair reload) — per the Boss-tests-every-build order, a SHORT smoke re-test goes to him
through the full pipeline before anything commits. The ledger v2.07 / orientation v4.24 close
entries are trued to this state in the same pass.

### §27.4 — The re-inspection, its one finding, and a premise refuted by probe

The fresh diff-scoped re-inspection over the fixed files confirmed **all six original defects
dead and returned exactly ONE new finding (LOW)** — a sharp one: both foreign-record removal
sites used bare `let _ = remove_file` instead of the `disarm_relocation` helper *this same build
added*, and the log said "removed" unconditionally — a lie whenever removal failed. Fixed: all
three removal sites (foreign-cleanup at activation, the command's not-ours refusal, moved-back-
home) now route through the reporting helper, and every message states what actually happened.

**The finding's read-only scenario was then REFUTED by direct probe** (recorded, not shipped as
fact): Rust 1.94's `std::fs::remove_file` deletes read-only files natively — a standalone probe
binary proved it, after the RED test unexpectedly passed and Python's `os.remove` (which IS
blocked) showed the OS attribute is real. The structural half of the finding (the unconditional
"removed" log) was true and is what got fixed; the helper's attribute-clearing retry stays as a
belt for other toolchains, with its comment corrected to say so. The two read-only tests now pin
the *guarantee*, with comments honest about the mechanism.

The `ui-inspector` round on the smoke re-test: **one finding in 30 claims** (a missing trailing
period in a quoted Rust string) — and its report documented a real render artifact in passing:
reasons ending in "." produced "repair.. Nothing was changed" through the template concatenation.
Both fixed (draft quote corrected; the frontend now strips a trailing period from the reason
before interpolation — inspector-caught, in the bundle by byte-grep).

Final state: suite **1,613 / 0** (+1 read-only pin), binary **2026-08-30 21:23:37**, bundle
byte-verified to carry the reload and the period-strip. The smoke re-test draft goes to the
panel, then to the Boss; his pass gates the commit.

### §27.5 — The panel wrap on the re-test: ten edits taken, one REFUTED by the ghost it warned about

Three lenses (false-failure, coverage, Boss's-time) + chair: **SEND-WITH-EDITS, eleven edits.**
The keepers sharpened real traps: the reload transient must not be the sole build discriminator
(a missed sub-second blank would have fired a false OLD-BUILD verdict on the correct build — the
durable checks now decide, and I verify the running binary from here if needed); banner timing
allowances before any failure conclusion; a transient status-bar `0` explained; the draft's own
contradiction fixed (7 edge cases + 1 happy-path finding, not "8 edge cases, none on your paths");
the "proven by automated tests" claim made honest (RED-first behavioural vs. source-pinning vs.
the reload that only the Boss can see); the double-click gloss no longer pre-declared harmless;
and — the chair's own addition, per the Verify-the-Finding law — "a read-only file" removed from
the fixed-list, since that premise was probe-refuted.

**Edit 5 REFUSED, with evidence.** The Boss's-time lens read
`%APPDATA%\world.uconstellation.app\universes.json`, found only `كون عيسى`, and the chair ordered
Step 8's "Eisa Cognitive Knowledge" replaced. That file's mtime is **2026-08-07 — three weeks
stale, untouched by tonight's many registry-writing actions on this very binary** — it is the
MSIX-virtualized ghost this session already documented this morning ("post-round verification
must rely on on-screen evidence"). The authoritative evidence is the Boss's own screen: Stage 2B
Step 12, passed tonight, switched to **Eisa Cognitive Knowledge** in this binary's Universe
Manager. The panel fell into the trap the pipeline note exists for; the refutation is the second
time today a gate's finding was itself refuted with primary evidence (the first: the inspector's
line-number citation during Stage 2 prep). A gate is not an oracle — in either direction.

---

## §28 — The smoke re-test BOSS-PASSED 8/8 (2026-08-31, just past midnight). PJ-435 CLOSED for real this time.

Four screenshots, all eight steps passed on the 21:23 build. The evidence read exactly as the
panel-hardened tutorial predicted:

- **Pre-repair count 3, post-repair count 2** — the false-failure lens had traced that the
  pre-repair shape is legitimately variable (2, 3, or 4 rows depending on cid collisions) while
  the ENDING is invariant at 2; his screenshots showed 3 → 2, the second live confirmation of the
  duplicate fold, on a fixture built entirely through the app.
- Banner verbatim at the new address (screenshot); post-repair state settled with **no banner,
  `2 notes`**, and `flimwick` → exactly one result that opens.
- The durable checks decided the verdict (the panel's edit) — no reliance on catching the
  sub-second reload transient.

**The commit gate is cleared: two Boss passes on this feature** — the full three-stage test on
the 17:39 build, and this smoke re-test on the 21:23 build carrying the eight inspection fixes.

Close-out actions in this pass: the English manual's repair procedure gained the reload ending;
a 14-agent workflow corrected the translated manuals' **stale false promise** ("a moved universe
is repaired automatically" — the pre-PJ-435 auto-repair claim English had already dropped) into
the honest banner-and-button paragraph, each anchored to its locale's own `movedRepairNow`
string; retest folders deleted; commit + push of everything since `4aee6ea2`.
