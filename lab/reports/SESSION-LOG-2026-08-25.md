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
