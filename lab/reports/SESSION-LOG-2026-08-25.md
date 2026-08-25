# Session Log — 2026-08-25

Function in hand: **the delete archive and what it actually keeps** — `Settings → Universe &
Libraries → Deleted notes` (PJ-385), the stale-index removal that writes into it (PJ-369 Step 4),
and the verification method the Boss ordered after a false finding shipped in fifteen languages.

---

## §1 — The Boss's three instructions

1. *"Regarding the 234 notes, file them as their own job."* → **PJ-387**, filed in ledger v1.99.
2. *"Have you reverted or fixed your false findings (603)?"* → §2.
3. *"Develop a method to double-check your future judgments to avoid falsifying any findings.
   Maybe by using an SME for this purpose."* → §3.

---

## §2 — The false findings: corrected, and one more found in the worst place

Yesterday's sweep left **two** live survivors of the claim that a deleted note has no text
*"because its file was already gone"*:

- `deleted_notes.rs:155` — a code comment asserting it.
- `settings.deleted.noTextStored` — **user-facing, in all 15 locales**, softened to "…or when its
  file was already gone before the text could be read." The same backwards claim, hedged.

Both are gone. The replacement states only what was traced: `index_note_impl` returns `Skipped`
when the path does not exist and `search.rs:8473` is the only production INSERT into `note_meta`,
so a row cannot come into existence for an unreadable file; an empty body means the **index** held
no text. The corrected code comment now quotes the false version so it cannot be rewritten.

**Two wrong numbers were also still in the diff**, both found by the verifier, not by me:

- **The median.** 18,944 is the median of all **603**; of the **601** that carry text it is
  **18,984**. The comment documenting the last wrong figure carried the next one. Corrected in
  `deleted_notes.rs:43` and `deletedNotes.ts:46`.
- **"101 of 2,731 rows … every one of them a file still on disk"** — it is **99**. The other 2 are
  `…\Eisa Test\.trash\Collision Test.md` and `…\Eisa Test\Town Eisa v2.md`, which are exactly the
  2 of the 603 phantoms carrying no text — **which is why the error was invisible inside its own
  arithmetic.**

**The orphan count: the panel was right, I was wrong.** Six, not five. My script matched by
filename and silently hit a different note sharing a basename. Recorded in
`SESSION-LOG-2026-08-24.md` §20.

---

## §3 — The SME: `findings-verifier`, and the law that carries it

`.claude/agents/findings-verifier.md`. It verifies **claims about reality**, not claims about code
— the gap neither existing gate covered. `ui-inspector` checks that a quoted string matches its
source; it passed the false sentence **twice**, correctly, because the string matched perfectly and
*the source was false*.

Its discipline: **default verdict REFUTED** · primary evidence only · **enter where the system
enters** (never a re-implementation, which agrees with a wrong hypothesis because it shares the
misunderstanding) · and the standing question **"if my method were wrong, would this result look
different?"** — if no, the result is worthless.

Written into `CLAUDE.md` as a top-of-all-rules LAW: *Verify the Finding, Not Just the Wording*.

**It works. Its first run refuted four of the ten claims I was about to repeat to the Boss**, and
the second and third runs each overturned something larger. Every correction below came from a
verifier or the panel, none from me.

---

## §4 — What the 603 actually are — my framing was wrong, twice

**First framing (wrong):** "residue of a reorganisation." The relative-path test that suggested it
was **mis-specified** — it kept two container segments that guaranteed a miss for every deep path,
so its "disagreement" with the content-id method carried no information. Corrected: **594 of 603
sit at an identical library-relative path.** It was a lift-and-shift.

**Second framing (wrong, and told to the Boss):** "not pointers to your writing." I generalised
from the first 120 rows of an unsorted join and two printed samples. Measured over the whole set by
two methods that could disagree — frontmatter on the live files, and provenance read from the old
index rows — **575 are Wikipedia-derived demo notes, 27 are the Boss's own hand-typed notes** (all
from the `Eisa Test` library; 6 of the 27 he had already deleted), and 1 is app-generated `Welcome.md`.
Corrected to him in the same session.

**What actually happened, from the record rather than inference.** On **2026-08-01** a MIG-108
unification ran on **Eisa Cognitive Knowledge**. Seventeen of its libraries were physically outside
its root, under `E:\Cognitive Knowledge\` — the legal pre-MIG-108 arrangement. The migration
**moved** them under the ECK root and rewrote **ECK's own** manifests and index. `Eisa Universe`,
which federates ECK as a Linked Universe and had separately indexed those files at their then-real
external paths, was not rewritten — correctly, since a member does not rewrite its parent. Its 603
rows still point at the pre-move locations. Evidence: ECK's own `mig108-journal.json`
(`phase: "done"`, 17 entries `action=move, moved=true, copied=false`, `old_path` values matching
the 603's prefixes exactly), its pre-migration `mig108-backup/libraries.json` showing the same
library **IDs** at the old paths, and NTFS creation times on the destination directories that
predate the move — a copied directory cannot carry them.

**Not settled by any file I read:** why the parent indexed only 603 of the ~7,352 notes now in
those libraries. Stated rather than papered over.

---

## §5 — PJ-321 closed: five corroborations, one shadow file

Full account in `SESSION-LOG-2026-08-24.md` §21 and ledger v1.99's preamble. In brief:
`%APPDATA%\world.uconstellation.app\universes.json`, read from a Claude session, resolves via
`fsutil hardlink list` into the Claude Desktop MSIX container's `LocalCache` — a copy frozen
2026-08-07. Its three sibling files pass through to the real location; only this one is shadowed.

Every observation the entry rested on follows from that, including the Boss's own controlled
experiment. The app is proven correct by its boot trace. **The contamination reached the repo**:
the committed `pj321-evidence-snapshot-2026-08-22/universes.json` is a copy of the shadow, and the
entry recorded its byte-invariance *as the finding*. A `READ-THIS-FIRST.md` now sits beside it, and
the `%APPDATA%` check is in the verifier's standing brief.

Two genuine defects surfaced while settling it: `set_active_universe` saving the durable intent
last with no rollback (**PJ-393**), and `remove_universe_from_registry` not clearing `active_path`
(already **PJ-322**).

---

## §6 — The archive keeps a search rendering, not the note

`note_meta.body_text` = `parse_frontmatter` (drops the whole YAML block) → `strip_markdown` →
`normalize_arabic_for_search`. That column is the only text a delete archives.

**Proven on real shipping output, not a re-implementation.** The live envelope for
`…\Constellation PKM\Vindar.md`, against the file recovered from the recycle bin: the file reads
`[[supports::Quazzle Renamed|it explains why]]`; the archive holds `supports::Quazzle Renamed`.
**The annotation — one of the eight Living-Link properties — is gone.** Three sibling files of
148–166 bytes archived as `''`. Worst case measured: 8,037 characters → 713.

**My own measurement was biased and the panel said so:** 59 of my 60 samples were Constellation's
own session logs. On unbiased samples the conclusion survives and mostly strengthens (diacritics
are removed **to zero**, not "reduced"), with two honest limits — markdown **tables survive**, and
**7–11% of ordinary short notes** do round-trip byte-exactly.

Consequence: `search.rs:13049`'s "the time machine survives an emptied recycle bin" was false in
both halves and is corrected, along with the `phantom_prune.rs` comment claiming a phantom's index
row is the **last** copy (measured false for 597 of the 603 — the guard stays, because fail-closed
is correct precisely when the assumption does not always hold). → **PJ-388**, which **blocks
PJ-386**.

---

## §7 — Five safety-inspection findings, all fixed before commit (WA#6)

Diff-scoped over the 11 changed files. Every one is new in this uncommitted diff, so none was
filable.

| | class | where | fix |
|---|---|---|---|
| **HIGH ×2** | false-success | `link_life.rs:445` → `deletedNotes.ts` | `read_lines` mapped **every** `read_to_string` error to an empty Vec. An archive that exists and cannot be decoded reached the user as *"The record exists and is empty"* — asserted as fact about the last surviving record of destroyed notes, with no error and no cue. `unreadableLines` could not rescue it: that banner renders only when `total > 0`. Now `NotFound` is a fact and everything else sets `LoadReport::unreadable_file`; both archive commands refuse. **Mutation-tested.** |
| **MED** | false-success | `driftReport.ts:153` | `lastPruneReceipt` was the one per-universe surface `handleUniverseSwitch` did not clear — module-level by design, so it carried "removed 603 entries" into the next universe. |
| **LOW** | concurrency-race | `SettingsModal.svelte:1362` | Three buttons called `onClose?.()` directly, bypassing the `closeSettings()` gate whose `phantomBusy` guard is the only re-entrancy protection on a permanent bulk removal. |
| **LOW** | index-divergence | `SettingsModal.svelte:1397` | The Deleted-notes list had no refresh path, so it omitted the very deletions the button beside it had just made. |

---

## §8 — User-facing text: six strings replaced, one added, ×15 locales

Panel-ruled. Every replacement clause was individually verified.

- **`settings.index.phantoms.confirm`** — the consent sentence at the irreversible moment. It said
  a record "**including its text**" is saved first, which is what makes an irreversible action feel
  safe while pointing at a record that cannot restore anything. It now says the text is the search
  index's, that it cannot rebuild the note, and that **the connections are removed with the entries
  and are not recorded at all**.
- **`settings.deleted.noTextStored`** — the false mechanism, removed without substituting a second
  causal claim (this message also shows when no envelope is found).
- **`settings.deleted.intro`** — "not always identical to the file" implied occasional divergence;
  it is never identical for any note with formatting. Now enumerates what is lost, and states that
  a note without its own identity leaves **no entry at all**.
- **NEW `settings.deleted.textCaption`** — the strongest finding on the surface: the archived text
  rendered in a `<pre>` with **no label of any kind**. A caveat in the section intro is a scroll
  away and read once; a false impression created at the bottom of a screen is not repaired at the
  top of it.
- **`settings.deleted.empty` / `.noArchive`** — both inferred absence-of-event from
  absence-of-record.
- **`plurals.deleted_textKept` + `.noTextKept`** — changed together, because the count is of the
  indexed rendering and relabelling one without the other says they describe different things.

`i18n-parity`: **all 15 locales in parity**, CLDR categories preserved per language.

---

## §9 — Verification

`cargo test --lib` **1572 passed / 0 failed** (one new, mutation-proven) · `vitest` **1008 passed /
87 files** · `svelte-check` **0 errors** · `i18n-parity` **15/15**.

The Boss has **not** tested this build. Nothing is committed until he does.

---

## §9b — SO#2: done in English, and the gap named rather than skipped

`docs/User Manual.md` and the English help topic now describe what the record actually keeps — a
stripped rendering, not the file — and state that the links a removed entry carried are recorded
nowhere.

**The 14 translations could not be updated, and that is a finding, not an omission.** The section
this change belongs in exists in **3 of 14** translated manuals (de, fr, ar) and is absent from the
other eleven; the English manual is 2,687 lines against Arabic's 1,982. There is no target
paragraph to edit. Filed as **PJ-394** — distinct from PJ-336, which measured the help *topics*.

Stated plainly because the failure mode is quiet: when the target section does not exist, SO#2
cannot be satisfied by an edit, and every such build reports success while widening the gap.

---

## §9c — BOSS TEST: Stage 1 PASSED, and it confirmed PJ-388 on his own data

All five steps completed. Two questions, both answered from source, neither a defect:

1. **"Why is the amber strip still here?"** Correct behaviour. It reports a count, and nothing has
   been removed — the count is still 603. `indexPhantomDismissed` (`+layout.svelte:607`) is plain
   in-session `$state(false)`, reset at boot and on every fresh drift report (`:3962`), so the ✕
   hides it for the session and it returns next launch until the entries actually go.

2. **"Where are the square brackets on the link line?"** — the finding, observed by him
   unprompted. **This is the test passing**, and it is PJ-388 reproduced on his own note by the
   person who designed the app.

**His probe, and what the prediction bought.** I told him in advance the record held exactly five
entries and named them. His screenshot shows **"6 deletions recorded"** — those five plus his
`Archive Probe`. A prediction that could have been wrong and was not.

His `Archive Probe` was three lines: `# My headline`, `this is **important**`, and a wikilink. The
archive kept **54 characters**, rendered as:

```
  My headline

this is  important
test1-fiction-brief
```

The `#` gone (leaving its space), the `**` gone (leaving a double space), the `[[ ]]` gone with the
target surviving as bare words. Exactly `strip_markdown`'s documented behaviour, on live data,
witnessed by the Boss.

**Fixed in-pass (WA#6) — the caption underspecified.** He read the caption, then read the text, and
still had to ask about the brackets. "Strips markup" did not read as "the brackets around your
links" to the person who designed the app — the category word failed where the specific loss would
have landed. The caption now NAMES the three things that visibly change (`#`, `*`, `[[ ]]`) plus
Arabic diacritics, and states that what a link pointed at survives while the brackets do not.
Rewritten in all 15 locales; parity verified. **Requires a rebuild before he sees it** — it does not
affect the removal path.

**Still his to decide:** the 603 removal. Not pressed.

---

## §9d — THE REMOVAL RAN. Boss-authorised, executed, and verified against the snapshot.

He pressed **Remove**. Receipt: *"Removed 603 entries. A record of each was saved in your universe
folder."* The control then vanished (it renders only while `phantomCount > 0`) and the amber notice
went with it.

**The receipt is a claim. Here is the evidence**, measured read-only against the pre-prune snapshot
(`E:\Backups\Constellation\EisaUniverse-preprune-20260825`) and the live database:

| table | before | after | delta |
|---|---:|---:|---:|
| `note_meta` | 2,731 | 2,128 | **−603** |
| `note_links` | 31,368 | 11,896 | **−19,472** |
| `sky_links` | 31,361 | 11,889 | −19,472 |
| `sky_nodes` | 2,731 | 2,128 | −603 |
| `note_body` | 2,731 | 2,128 | −603 |
| `review_schedule` | 2,731 | 2,128 | −603 |
| `note_summaries` | 753 | 254 | −499 |
| `note_aliases` | 157 | 30 | −127 |
| `note_embeddings` | 5,161 | 5,161 | **0** |

**Every predicted figure landed exactly** — the −19,472 links and −127 aliases were both forecast
before the run. Nothing was removed that was not on the list: comparing the full path sets,
**603 removed, 0 added, 0 removed that were not among the 603.**

**The archive.** 5 del-envelopes before → 609 after. **604 new, of which 603 carry
`reason=phantom_prune`** and their paths match the 603 list **exactly**; the 604th is his own
`Archive Probe` from the test. **20,484,230 characters archived** — the exact figure measured
yesterday — with **2 envelopes carrying no text**, exactly the two frontmatter-only notes predicted.
Zero unparseable lines.

**Nothing else was touched.** `Eisa Cognitive Knowledge` still holds 8,031 notes. No `.md` file was
created, moved or deleted. `PRAGMA integrity_check` = `ok`. FTS is in step with `note_meta`
(2,128 = 2,128).

**The prune left NO orphans**: 0 rows remaining at the 603 paths in `note_embeddings`, and 0
orphaned rows in `note_body`, `note_links` (by source), `note_summaries`, `review_schedule` or
`note_aliases`. PJ-392's concern does not apply to this path — `note_body` was purged with the rest.

**One pre-existing defect surfaced and is NOT attributable to the prune → PJ-395.** 4,062 of the
5,161 embeddings are orphaned. The count is **identical in the snapshot and in the live database**,
which is the check that could have disagreed and did not. They predate the removal; the producer is
unknown and must be found before any cleanup.

---

## §9e — PCS complete

Committed **`35a9921d`**, pushed to `origin/main`. 38 files, +15,067/−97.

- Session logs: `SESSION-LOG-2026-08-24.md` §21 (the shadow-file resolution) and this file.
- Help + User Manual: English corrected. **The 14 translations could not be** — the section does not
  exist in 11 of them → **PJ-394**, filed rather than skipped (§9b).
- Ledger reconciled twice: **v1.99** at the build, **v2.00** at the close (PJ-385 and PJ-369 closed
  with evidence; PJ-395 filed; `► Next action` → PJ-387).
- MoCh: `MoCh-2026-08-25-0900.md` and `MoCh-2026-08-25-1300.md`.
- Orientation: **v4.16** at the build, **v4.17** at the close — v4.16 stated the build was
  uncommitted and untested, which the Boss's test and the removal made false within the hour.
- Handover: `lab/reports/HANDOVER-2026-08-25.md`.

---

## §10 — Open

- **Boss test of PJ-385 (Deleted notes)**, then the 603 ruling — carrying PJ-390's collateral
  figures (19,472 link rows, unrecorded) and PJ-388's finding.
- **Before that ruling he must confirm the active universe is `Eisa Universe`** — the status-bar
  universe name; Universe Manager → **Switch** if not. Every figure here is that universe's.
- PJ-387 (the 234) · PJ-378 (58 sweep findings) · PJ-388–393 as filed.

---

## §11 — PJ-387 INVESTIGATION (Boss-ordered) — the 234 with no identity; the 13 explained

**Function in hand:** the 234 `Eisa Universe` index rows carrying an empty `cid_cn`, and specifically
the 13 whose files carry a real one.

**Boss's order (2026-08-25):** *"Start with the 13, not the 221. Why that happened is not known.
Investigate it — don't theorise it… The other 221 have no frontmatter. Giving them identities means
writing into my notes, so bring me the options; don't act."*

**Nothing was written.** No `.md` file, no live database. Every database read was against a scratch
copy taken with its `-wal`/`-shm`. Constellation was verified not running before the copy.
`git status --porcelain` empty at `9d4cb7ec` throughout.

### §11.1 — The population, measured (all `Eisa Universe` unless named)

`note_meta` 2,128 rows · **234** with `cid_cn = ''` (empty string; **0 NULL**; a `typeof` sweep
returned `text` for all 2,128) · all 234 files exist on disk · all `.md`.

Split by a real frontmatter parse, not a substring search: **13** carry a `cid_cn` key in their file,
**221** have no frontmatter fence at all.

The 234 hold **32,625,753 characters** of `body_text` — **65% of the universe's 50,123,949** — and are
the source of **2,461** `note_links` rows. **0** rows target them.

### §11.2 — Cause 1: TEN of the 13 are a deliberate refusal, and the app logged it 845 times

`search.rs:8497-8536`. After the `note_meta` upsert, on a UNIQUE violation of `note_meta.cid_cn`
where the current owner's file **still exists on disk**, the indexer refuses to steal the identity
and calls `do_upsert("")` — the documented `''` sentinel — emitting:

```
[index_note] cid DUPLICATE (both files live): <cid> also claimed by <owner> — indexed <path> with '' sentinel, NOT stolen
```

`.constellation/diagnostics.log` holds **845 such lines naming exactly 10 distinct paths and 6
distinct cids**, most recently **2026-08-25 12:36:27Z** — the latest app run, not history. Those 10
are a subset of the 13. Verified per path by a second method independent of the log (read the file's
own `cid_cn`, query the owning row, `stat` the owner): **10/10**, every owner live, and the log's
named owner agreed in all ten.

**Re-indexing does not fix them** — the arm re-fires while all three conditions hold. Each path has
been re-attempted **81-85 times across 46-54 distinct log timestamps**. (Caveat: `index_note`'s mtime
gate at `:8217-8227` short-circuits an *unforced* re-index of an unchanged file, so this is the
forced-reindex path.)

**Where the duplicates came from, established:** `كون عيسى 2\` and `كون عيسى 3\` are folder copies of
a **whole universe** (`كون عيسى`) sitting inside `Eisa Universe`'s root — each carries its own
`.constellation\` with its own `search.db` and a `universe.json` naming itself `كون عيسى`. Their notes
carry the original's identities verbatim, so 4 notes per copy collide three ways.
**The check that could have disagreed and did not:** a 5th note in each copy
(`إختبار المرحلة 2.md`) has a *different* cid in each copy (`…F8CD` / `…4103` / `…F8A3`) and indexed
cleanly. The other two of the ten are `BUG-015-target-NOTE_531D-corrupted-snapshot.md` (cid owned by
`BUG-015-source-NOTE_EE1E-snapshot.md`, a lab artifact of BUG-015) and
`Constellation Working Docs\README.md` (cid owned by `MIG-090-Plan-Notes-Navigator.md`) — the latter
two being *differently-named* files sharing one cid, not copies.

### §11.3 — Cause 2: THREE of the 13 have two frontmatter blocks, and the app can never repair them

`موسوعة عيسى\الزراعة\الكيماويات السامة.md` · `موسوعة عيسى\العرب\الحروف العربية.md` ·
`موسوعة عيسى\…\دورة InDesign 2025 من Linkedin Learning\الواجهة.md`.

Each has a first block carrying a legacy `cid:` and no `cid_cn`, a blank line, then a second block
carrying `cid_cn` **with the same value**. `split_frontmatter` (`search.rs:3835-3845`) takes only up
to the FIRST closing fence; everything after is body. So `properties.get("cid_cn")` (`:8257`) yields
`""`.

Proven three ways that separate this cleanly from Cause 1: the rows' `properties_json` holds the
first block's keys (`cid`, `created`, `title`); their `body_text` **literally begins with the second
block**; and they appear **zero** times in `diagnostics.log`. File mtime `== note_meta.modified`, so
this is the parser's verdict on these exact bytes.

**`canonical::ensure_cid_cn` can never repair them.** `canonical.rs:1451` returns early when
`content.contains("\ncid_cn:")` — a **whole-file** scan — which is true because block 2 contains it.

**Origin of the double block:** the three files' mtimes are `2026-05-28 13:40:32/33/34Z`, one per
second, with no other file in the universe written in that window. Neither `migrate_cid_to_cid_cn`
nor `inject_cid_cn` can produce the shape, at HEAD or at `b19908c1` (and **no commit touched
`canonical.rs` between 2026-04-14 and 2026-06-11**). **The author is NOT established and is not
attributed.** What IS established is §11.6 N1 — a live path at HEAD that produces exactly this shape.

`note_state_history` rows 445/446/447 (`captured_at` **2026-08-08T12:46:45Z**) record these three
notes' properties changing from a set containing `cid_cn` to one containing the retired `cid`, with
`title` and `created` byte-identical on both sides. Row 902 (`README.md`) is `{}` → `{cid_cn:…}`.
**All four `note_state_history` rows on the 234 are the app's own identity churn — zero user content.**

### §11.4 — The consequence, and where it is NOT exposed

`build_delete_archive` (`search.rs:13078-13113`) returns an **empty Vec** when the resolved cid is
empty. Phase 2 — the entire archive-or-refuse contract — sits inside `if !archive.is_empty()`
(`:12881`), so an empty archive **skips it silently**. Phase 3 purges and returns `Ok` (`:12909`).

- **`PhantomPrune` is NOT exposed** — `phantom_prune.rs:836-843` refuses an empty-cid candidate and
  reports it as `skipped`, never `removed`.
- **`Trash` / `SystemTrash`** leave the `.md`, so the loss is bounded.
- **`Permanent`, `Vanished`, `ReconcileGone` are the unbounded cases**, and the latter two are
  **automatic** (watcher `search.rs:13488`/`:13581`; boot reconcile `reconcile.rs:666`). All three
  sites re-`stat` immediately before deleting and fire **only when the file is already absent** —
  verified, and it is why a naive "refuse to purge" fix is wrong.
- The pinning test `a_note_without_a_cid_is_purged_but_not_archived` (`search.rs:17467-17488`)
  asserts only the *not archived* half. **It never asserts the row was purged** — a regression that
  stopped purging would keep it green.

**No earned data is at risk on this population.** Across both live universes, of the empty-cid rows'
outgoing links: `weight <> 1.0` → **0**; `traversal_count > 0` → **0**; all `confidence =
'hypothesis'`; all `status = 'active'`. `review_priority` NULL for all. (`review_schedule` **does**
hold a row for each — because it holds one for *every* note; all are `never_reviewed`, and
`last_reviewed IS NOT NULL` returns **0 across both universes**.)

### §11.5 — The 221, and the fact that reverses the options

219 are `Constellation Orientation & Onboarding v*.md`; the other 2 are
`CANONICAL-FILENAME-ARCHITECTURE.md` and `CANONICAL-FILENAME-ARCHITECTURE 1.md`. All in
`Constellation PKM\Constellation Working Docs\`. Against the repo's `docs\`, by basename:

| | count |
|---|---:|
| byte-identical | 102 |
| identical after CRLF/LF normalisation (library LF, repo CRLF) | 113 |
| same name, content differs | 5 |
| no file of that basename in the repo | 1 |

The 5 differ by **120 changed lines (+29 / −91)** and it is **not** cosmetic: the library copy of
`CANONICAL-FILENAME-ARCHITECTURE.md` is missing the entire `## 0. Post-MIG-003 — Architecture
Inverted` section. The no-twin file, `CANONICAL-FILENAME-ARCHITECTURE 1.md`, is byte-identical to the
repo's `CANONICAL-FILENAME-ARCHITECTURE.md` and is the **newer** of the library pair.

**THE REVERSAL — `ensure_cid_cn` writes NONE of the 234 on open.** Proven by running the *shipping*
function against copies of all 234 files: **234/234 — no disk write, no content change.** 232 trip
the whole-file `\ncid_cn:` guard because the orientation documents contain a worked YAML example
showing a `cid_cn:` line; the 2 `CANONICAL` files take the legacy-`cid:` branch instead, where
`migrate_cid_to_cid_cn` (`canonical.rs:1382-1384`) returns early because neither file has a leading
fence. **A note is permanently denied an identity because it explains how identities work.**
The set is therefore **stable** — "leave them" is durable, not a slow drift.

### §11.6 — Discovered on the way (filed, not smuggled in)

- **N1 → PJ-396.** A **live frontend path at HEAD** prepends a second frontmatter block. Rust
  `split_frontmatter` uses `trim_start()`; the frontend's two parsers (`yamlDoc.ts:182-186`,
  `store.ts:2573-2575`) require line 0 to be exactly `---`. On a note whose bytes begin with a
  newline before the fence, the frontend concludes there is no frontmatter and
  `composeFrontmatter` (`yamlDoc.ts:449-459`) prepends one. **Reproduced through the shipping entry
  points** — `composeUpdatedContent` (`store.ts:2957`) and the `noteModel` compose path
  (`noteModel.ts:183-189 → 549`) — with a control on an ordinary note that merged correctly.
  The **Rust** half of this class was fixed by PJ-207 §15 (`fence_offset`, `sources/mod.rs:326-347`,
  which records "a scan of the live universes found 28 notes with exactly this shape"). **The
  frontend half was never swept.** Currently **0 of 10,159 indexed notes** are in the vulnerable
  state; 9 files have the shape, all under `.trash`.
- **N2 → PJ-397.** **34 files already carry two stacked YAML blocks** (26 `Eisa Universe`,
  8 `Eisa Cognitive Knowledge`), distinct from the 46 files with a benign `---` rule after
  frontmatter. Asking the index itself — *does `body_text` begin with the second block?* — **12** have
  the second block filed as body. **Nine of those keep a valid identity in block 1, so PJ-387's own
  `cid_cn = ''` lens could never reach them.**
- **N3 → PJ-398.** Block-list properties with no dedicated storage are **dropped**.
  `parse_frontmatter` returns `HashMap<String,String>`; a list key stores `""`. `tags`, `sources`,
  `content_type` and `aliases` DO survive (dedicated columns / `note_aliases`), so the honest
  residual is **1,204 notes** (394 EU + 810 ECK) — not the 3,244 an unfiltered count gives. Top keys:
  `المجموعة` 197 · `institutions` 191 · `main_interests` 170 · `school` 143 · `field` 133 ·
  `notable_ideas` 130 · `notable_works` 128 · `author` 100 · `collections` 99 · `up` 84.
- **N4 → PJ-397.** `Eisa Cognitive Knowledge\Eisa Test\تجربة الكتابة باللغة العربية.md` carries
  `tags: idea` in its second block. Index `tags_json` = `[]`; `tag_counts` has no `idea` row among
  20,462; a filesystem sweep of all 8,070 ECK `.md` finds **0** notes with `idea` as a first-block
  tag. **A tag the Boss wrote exists nowhere in the app.**
- **N5 → fix in-pass.** Five shipped comments cite **2,731** as `Eisa Universe`'s row count
  (`search.rs:13067`, `:13094`, `phantom_prune.rs:12`, `:832`, `deleted_notes.rs:37`). Live is
  **2,128** — 2,731 was the pre-prune count and `35a9921d` removed the 603. `search.rs:13094` is also
  the source of a true-but-misleading figure: of `Eisa Cognitive Knowledge`'s 25 empty-cid rows,
  **11 are `\Templates\` and cid-exempt BY DESIGN** (Boss ruling 2026-07-19 — a template is a MOLD),
  4 are in `.trash`, and **0 are ordinary live user notes.**
- **N6 → PJ-399.** `write_gate`'s self-attestation is **blind to this corruption**: with
  `expect: None` it reads `extract_frontmatter_cid_cn(content)`, and a stacked-block rewrite's *first*
  block has no `cid_cn`, so the verdict is `OkUnchecked` and no anomaly is journalled.
- **N7 → PJ-400.** cid minting has **8 sites** (not 5) and **no identity-uniqueness check**. The loop
  in `generate_canonical` (`canonical.rs:49-93`) only tests whether a *file of that canonical name*
  exists in one directory — dead since MIG-003 made filenames human, and **entirely dead at the 5 of 8
  sites that pass `target_dir: None`**, where it returns on the first iteration. The suffix is
  16 bits (`rand … {:04X}`). The only real enforcement is the partial UNIQUE index, whose remedy is
  §11.2 — silently blanking the note's identity. Orientation §6.4's "collision avoidance tries 10 hex
  suffixes" describes real code but implies a protection that does not exist for `cid_cn`.
- **N8 → PJ-401.** `ensure_cid_cn`'s write is **not undoable** — `atomic_write` → `ReplaceFileW` with
  `lpBackupFileName = std::ptr::null()`, the API's own backup facility explicitly declined; the
  journal stores an FNV-1a hash, not content; neither universe root is a git repo. And the behaviour
  is documented **nowhere** in `docs/User Manual.md` or any of the 15 `docs/help.*` trees (313 files
  searched, including the native identity vocabulary in each language).
- **N9 → PJ-402.** Every `note_links` row in both universes has `last_traversed == created`
  (2,461/2,461 and 41/41 on this population). A column that reads as evidence of traversal is stamped
  at creation — a cross-check that cannot disagree, for anyone who uses it as one.

### §11.7 — Process: three gates ran, and each one changed the answer

- **`findings-verifier`** on 24 claims: 22 CONFIRMED, **2 REFUTED**. It broke my ecosystem scan
  (a Latin-alphabet key regex on a majority-Arabic corpus — moved the count from 4 to 12) and it
  caught the `search.rs:13094` template-exemption framing before it reached the Boss.
- **A 9-agent adversarial panel** (4 lenses, each refuted, then a synthesis that ruled) **overturned
  the single most decision-relevant claim in my brief** — that opening these notes writes identities
  into them — and caught me ranking a change as "writes no user file" when it writes 221 of his.
- **`findings-verifier` on the panel's own 12 new claims**: it **refuted the panel** on three
  (the on-open count is 0 of 234 and my "2 exceptions" were also wrong; the 5 differing docs lose
  120 lines including real content, not "23 lines of headings"; "216 have an exact twin" is 215, and
  the 216 was arithmetic rather than a check) — and **confirmed N1 with a running reproduction**.

**Errors of mine, corrected in the open before any reached a durable record:** the Latin-only regex;
the on-open claim (twice — my original, then my own correction of it); "9 files lose properties"
(several are not losses); and a `note_state_history` query keyed on the wrong column that returned a
false zero.

### §11.8 — State

**Awaiting the Boss's ruling. Nothing built, nothing written to his data.** PJ ledger → **v2.01**,
orientation → **v4.18**, MoCh `MoCh-2026-08-25-1700.md`.
