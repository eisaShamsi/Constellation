# Session Log — 2026-07-21

**Arc:** the Boss made the Template Studio priority one. A research→rule→build loop ran five times;
**four of my own framings were corrected by him or by the evidence**, and each correction produced a
better design than the one it replaced.

---

## 1. MIG-103 §1 — Save as Template (the three kinds)

**R1 cross-check first** (`wf_e5aee265-899`) — the Boss asked *"What are the standards? Is there a
case study we can follow?"* before I locked the keep-vs-strip semantics. **It overturned my own
recommendation.**

- **Two international standards exist** (OOXML `.dotx`, ODF `.ott`) and agree: a template is a **full
  document whose type label changes**. Nothing is stripped. Converting in Word is a one-line re-label.
- **8 of 11 shipped products keep everything**; exactly **one** offers structure-only, and only as an
  opt-in toggle. **My earlier "structure-only by default" would have made Constellation the only one.**
- **The case study: Evernote's Save-as-Template** — the same gesture — plus its one documented
  failure: instantiated notes inherit the template's creation date. **Identity must be re-stamped at
  instantiation**, which is exactly the Boss's caveat, landing where the standards put it.

**Built:** `create_template` + `template_content_from_note`, the `TMPL` re-type (the kind already
existed in `file_kinds.rs`), collision-resolving create-exclusive write.

**Also fixed, found while writing the plan:** `create_note`'s frontmatter merge **trimmed every
line**, destroying nested-YAML indentation on EVERY template instantiation, and its identity filter
matched trimmed lines so a NESTED `title:` was falsely dropped. Both red-first.

---

## 2. The Boss's taxonomy — FOUR kinds, not two

I had a snippet-vs-scaffold binary. **Boss:** *"three kinds: a whole note, just the frontmatter, a
snippet. Did I miss something?"*

**His cut is better, and it is the anatomy of a note** (frontmatter + body → either part, or both).
And it surfaced something my binary missed entirely: **frontmatter-only is a distinct ACTION** —
*apply* properties to the note you're in — which is neither "create" nor "insert at cursor."

Then he added the **fourth**: a manuscript/project builder. Answer to "did I miss something": within
a single note, no — but a multi-note structure is genuinely beyond the three, and he took it.

**Built:** `TemplateKind` (whole | frontmatter | snippet), each stamping `template_kind:` on the file
so the use-side knows the action. Boss-validated.

**Then:** *"I want to be able to choose whether to select a word/sentence/paragraph or the whole
note."* → snippet extent is now the user's choice; the dialog offers *My selection* vs *Whole note*
**only when a selection exists** (nothing to choose between otherwise). Boss-validated.

---

## 3. The wizard study — my framing was right in spirit, wrong in shape

Boss described a step-by-step manuscript wizard — against his own standing "a wizard is a rail"
ruling. Rather than resolve it myself: *"Let's study and research the wizard, and we will decide upon
the results."*

`wf_e169f193-8a5`, five tracks. **Verdict: do NOT build a linear wizard.** Of **8 leading manuscript
tools** (Scrivener, Longform, Ulysses, yWriter, Manuskript, Notion, Word, LaTeX) — **ZERO** use a
step-by-step wizard. All use *pick a type → whole structure appears → edit directly*. NN/g names
expert-creative work as precisely where wizards fail.

**The reconciliation:** not wizard-good-or-bad but **rail** (decides order before meeting you) vs
**proposal** (offers a complete editable structure derived from *you*). A 9-item checklist gates it.

**Size finding:** MEDIUM, not huge — the PJ-065 structural lane, the Structure panel, and
`create_note` with a template body all already exist. **The builder orchestrates reality; it does not
invent the spine.**

Boss ruled: propose-editable-structure · manual slotting first with the smart proposal layered on ·
"structured composition" as a named mode.

---

## 4. ⛔ THE BOSS CORRECTION — the "note camp" framing was a category error

I researched whether New Note should offer a template chooser (`wf_ada130cf-4ae`) and concluded
*"Constellation is in the note camp, 15–0."*

> **Boss:** *"Wrong assumption! Constellation is a PKM/PKF system… it could use long documents or
> short notes, to formulate one's knowledge."*

**He is right, and it is categorical.** I imported an outside taxonomy (note-app vs document-app),
assigned Constellation to a camp, and reasoned from that camp's habits — instead of reading our own
concept. The evidence was already in the repo: the four compositional forms, note **shape**
(scrap→page), and the **manuscript builder** we are building.

**What survives:** the product counts (15/15 note apps don't gate; 5 of 8 document apps blank-first).
**What is withdrawn:** the inference that they settle Constellation's identity.

**The corrected frame — design per COGNITIVE ACT, not per app:**

| Act | Needs | Surface |
|---|---|---|
| **Capture** | blank, instant, never gated | New Note |
| **Compose** | structure offered up front | New-from-template · the manuscript builder |
| **Formulate** | move between them, late | apply-a-template-later · shape graduation |

**"New Note stays blank" survives with a STRONGER reason:** not *because we're a note app* (imported,
false) but *because that gesture IS the capture gesture* — and in formulation **you frequently don't
yet know what the thing is**. A scrap becomes a chapter. Finding out **is the work**.

**This promoted D4** (apply a template to a note already started) from third-priority nicety to **the
bridge between capture and composition**.

*(Also corrected in the same pass: my pre-research claim that "Word, Pages and Google Docs all open a
template gallery." Wrong for Word and Google Docs — Ctrl+N gives a blank document. Only Apple's three
gate, and Apple is the only vendor that shipped an off-switch for its own chooser.)*

---

## 5. D1–D4 ruled; D2 built

Boss: **D1 blank · D2 yes · D3 yes · D4 yes.**

- **D1** needed no work — New Note already opens blank and instant.
- **D2 — the template door.** A quiet "Start from a template…" inside an empty note, gone on the first
  keystroke. **Boss-validated (Steps 1–6).**

### The D2 miss, and why it is a lesson about assumptions

First build showed **nothing**. Diagnosed by reading the Boss's actual note bytes AND CodeMirror's
source — not guessed:

- a new note's body is **`"\n"`** — the blank line after the closing `---`;
- CM6's placeholder condition is literally `doc.length ? Decoration.none : this.placeholder`.

**Length 1 → never rendered.** My assumption was that "looks empty" means `length === 0`.

**Fixed** with our own blank-predicate (empty **or** whitespace-only), O(1) on real notes, rebuilding
only when blankness flips. **Deliberately NOT fixed by trimming what `create_note` writes** — content
handling stays byte-exact; a UI affordance is never a reason to change what lands on disk.
`tests/mig-103/blankBody.test.ts` pins the exact failing case.

---

## Standing-order notes

- **Reproduce-First earned its keep again**: the D2 diagnosis came from the artifact + the library
  source, exactly as the earlier `search.db` and disk-bytes diagnoses did.
- **The Boss's corrections outperformed my framings four times** (three kinds > my binary; the fourth
  kind; the wizard reframe; the camp category error). Recorded because the pattern is the lesson:
  when I import an outside taxonomy, I should check it against Constellation's own doctrine first.
- **Suite flake noted honestly:** 3 Sight v6 wall-clock benchmarks (32 ms / 16 ms thresholds) slip
  2–4 ms under parallel load. **Verified pre-existing** — they fail on clean `main` with my changes
  stashed. Unrelated to this work; worth a separate look since a flaky suite trains us to ignore it.

---

## 6. Pre-commit safety inspection — and an APP-KILLER in my own D2 code

`wf_1850299a-c4e`. Ran **whole-app** despite `args.files` (PJ-124 re-confirmed, third time).
**60 confirmed: 4 APP-KILLER · 24 HIGH · 20 MED · 12 LOW.**

### The one that was MINE — fixed before commit

**Split-view wrong-note wipe.** The template door dispatched `detail.path`, and
`handleApplyTemplateHere` **discarded it**; `applyTemplateToCurrentNote` then targeted
`get(focusedTab)` and replaced the **whole document**.

The mechanism is what makes it vicious: the door fires on **mousedown**, and the picker mounts as a
`position:fixed; inset:0` overlay **before mouseup** — so the pane's click-to-focus never runs and
focus stays on the *other* pane. Pane A (a long note, focused) gets its entire body replaced and the
template's properties merged in; pane B — the blank note whose door was clicked — is untouched. No
error, and the user is looking at B.

**My guards passed while doing the wrong thing:** they checked focus *consistency*
(`still.id === tab.id`), not that the target was the door's note.

**Fixed in two layers:** (1) the target is resolved from the door's own path, carried through
explicitly; (2) a whole-document replacement may only run on a **still-blank** document — the door
cannot appear otherwise, so a non-blank target means something moved underneath us and the right
answer is to refuse. `tests/mig-103/applyTemplateTarget.test.ts` pins both, **proven red** against the
defective shape (3 failed → restored → 16 green).

### Pre-existing app-killers — FILED, not buried
- **PJ-133** — *"Discard my changes" silently un-discards* (`store.ts:506`). The discarded text stays
  in the dirty model and is written back on the next flush, after the banner and the recovery net are
  gone. The only `reloadTabsFromDisk` caller lacking both cascade guards.
- **PJ-134** — `ensure_universe_notes_folder` turns any libraries.json read/parse failure into an
  EMPTY library list (`universe.rs:384`).

Neither is from this job. **PJ-130's scope grew** (60 vs the earlier 37) and needs re-triage against
HEAD before its batches are worked.

## Close

**Rust 1093/0 · MIG-101/103/G4 suites 55/0 · MIG-103 16/16 · svelte-check 0 errors.**
Orientation → **v3.62**. PJ ledger → **v1.41** (PJ-132/133/134 filed).
**Boss-validated:** §1 save-side (three kinds, title prompt, snippet extent) and D2 (Steps 1–6).
**NOT Boss-tested:** the split-view fix above (found after his pass), and Batch 1 from PJ-130.

---

## §4 — Shape discovery hardened against prior art (Boss-directed audit) — `18f41bea`

Boss: *"I need you to conduct a thorough audit of other systems, such as Obsidian,
to design a robust algorithm."* Five research tracks, load-bearing claims
adversarially verified. Full audit: `docs/concept-papers/MIG-103-Shape-Discovery-Algorithm-Audit.md`.

**The concept is unclaimed; the algorithm is not.** 11 of 11 type mechanisms
audited across Obsidian (Dataview/Bases/Templater/Metadata Menu), Notion, Tana and
Logseq require the user to DECLARE a type — none proposes one from the corpus. But
JSON/NoSQL schema inference and Wikipedia infoboxes both solved the algorithm at
scale, and they agree on the same answer.

**ADOPTED — fill rates, not rigid sets.** Our exact-signature grouping is Baazizi's
*label equivalence*; the fragmentation is the documented cost of the
maximum-precision end of a known spectrum, and no threshold tuning fixes it. The
universal treatment is ONE kind carrying a superset of keys with a fill rate
against each (`Infobox person`: 142 parameters, ZERO required, `name` at 91%;
Compass renders "present in 87%"; quicktype merges and marks optional). A kind is
now `{core keys}` + `{key → count, fill}`. Real Universe: **106 rigid signatures →
21 coherent kinds**; the philosopher went from 32 fragments to one 161-note kind
with an honest tail (born 90% · died 84% · main_interests 83% · school 73%).

**Four designs discarded with evidence, three of them mine:** closed frequent
itemsets (surfaced bare fields as types) · maximal itemsets (91 patterns, largest
covers 18, every core deleted) · Jaccard clustering (mean 1.94 keys/note = its
documented failure condition) · signature-only cores (a unit test exposed that a
family's core need not exist as a signature; our corpus hid it by happening to
contain 146 plain notes).

**One filter the audit did not name but the data demanded:** intersections traded
one over-generation for another — `{born,institutions}`, `{alma_mater,born}`,
`{born,field}`, `{awards,born}` are one family sliced four ways, and minimality
cannot see it because none contains another. Comparing MEMBERSHIP instead of keys
collapses 30 → 21, while `{born,institutions}` survives on merit (an academic is
not just a person).

**VERIFIED, NOT ADOPTED** — the audit's two "mechanical fixes" do not apply here.
`tags` lives in a SEPARATE column (97.6% non-empty) this algorithm never reads — as
a frontmatter key it is on ZERO notes; `kind` sits in properties on 185 of 7,802
notes, 176 of them `'note'`. Checked against the live DB, not implemented on faith.

**12 `template_discovery` tests · Rust suite 1105/0.** Backend only — no user
surface yet, so nothing for the Boss to click. **The surface is the next build and
IS Boss-gated before commit.**

---

## §4B — a kind's name, read off its own members — `35263d3d`

Boss ruling: *"Propose a name from the members, let me edit it."*

**Concept:** the name of a kind is already written in the user's own notes — naming
is reading, not inventing. `TemplatePrompt` already prefills-and-selects, so the
proposal arrives as accept-or-overtype with no new UI paradigm.

**Measured before researched.** Probing the real Universe killed the §4 audit's guess
that the modal tag/folder names a kind: plain frequency proposes `source=Wikipedia`
(100% of EVERY kind) and `the` (44% of film titles); the film kind's top folder is
**History** at 79%/16x, so container-nearest naming (quicktype's rule) would answer
"History" with confidence. Differential scoring fixes the first completely, with no
stopword list — the only approach that survives fifteen languages. It does NOT fix
the second: lift over-rewards rarity and the top-lift token is the imported slug
`template-film-date-with-1-release-date` (69x). No scalar statistic separates it from
`film`, because the same importer emitted both it and the frontmatter keys the kind
was discovered from. Only CORROBORATION does — and the slug is then *mined*, donating
`film` to the tag family.

**Independence had to be engineered:** `folder:` is also a property and the library is
also a path segment; naive splitting double-counts one fact as two and inflated 8
honest results into 18 mostly-wrong container names (ARTS, HUMANITIES, SCIENCE).

**55-agent prior-art round**, every load-bearing claim adversarially refuted; almost
all died. Six survivors changed the code — Wilson lower bound *on the gates* replacing
my ad-hoc `n/(n+5)` (Monroe/Colaresi/Quinn §3.2.6: a count floor "simply removes the
most problematic features without resolving the issue"), word floor 3→2 chars (a
3-char rule silently drops 映画 from `カサブランカ (映画)`), `is_value_noise` so
`kind: film` can name a kind, ranked alternates + collision resolution by the user's
own rarest core key (`person · institutions`, never `Person 2`), and raw COUNTS in the
evidence because we have no ground truth for a confidence figure.

**Derived, not tuned:** a 3-note kind can never be named (max attainable bound 0.438 <
0.50); a 9-note kind needs 8/9; a 679-note kind needs 53%.

**Two avenues tested and rejected rather than assumed:** retuning z (at 1.645 not one
lost name returns — corroboration binds, not confidence) and outgoing links as a
seventh family (only JSTOR/ISSN identifiers clear both gates).

**Real Universe: 3 of 21 named** (`film`, `cathedral`, `import`); 18 say so. Abstention
is a real answer — `born · died` is the largest kind of all (679) and has no name
anywhere in the corpus, sitting 20% Philosophy / 15% Film / 13% Literature.

**23 tests + an `--ignored` real-Universe harness** (four designs passed their unit
tests and were still wrong, so the real-data check is now a test, not a script).
**Rust 1116/0 · svelte-check 0 errors.**

**FOUND, NOT INTRODUCED → PJ-135.** Validating across the Boss's other Universes
exposed that `is_noise` is an English-only list: `أنشئ · حُدث` ("created · modified")
is the #1 shape in Eisa Universe (241 notes) and #3 in Constellation Test (538). Real
shapes, no meaning, outranking every genuine kind. Needs a Boss ruling — see the PJ.

**PJ ledger → v1.42.** Per-build safety inspection launched diff-scoped.

---

## The standing inspection — three APP-KILLERs, all fixed and Boss-validated

Per-build inspection launched diff-scoped over `template_discovery.rs`; it **ignored
`args.files` and swept the whole app** (PJ-124, confirmed a 5th time). 90 agents,
57 confirmed: **3 APP-KILLER · 9 HIGH · 34 MED · 11 LOW** — and **zero findings in the
new §4/§4B code**.

### 1. Rename destroyed a nested frontmatter value — `6cb55169`
`update_frontmatter_title` matched `title:` on the TRIMMED line, discarding the only
thing that distinguishes a nested YAML key from a root key. A note carrying
`source:` → `title: Muqaddimah` lost that value PERMANENTLY on rename; `source:` was
left empty and `author:` orphaned under a scalar. **The second-order effect was worse:**
the result is invalid YAML, and `composeFrontmatter`'s invalid-YAML branch passes
frontmatter through verbatim by design — so from that rename onward every property,
tag and typed-link edit on the note was silently discarded. The note quietly stopped
accepting changes while looking completely normal.

**THIRD STRIKE on trim-the-line parsing (LL-014)** — the same class was fixed hours
earlier in `merge_initial_frontmatter`. Both sites now key off column 0. *Indentation
is data, not whitespace.* A **second, independent bug** in the same function surfaced
while fixing the first: with `aliases:` above `title:`, the title branch `continue`d
with the list still open and appended the alias AFTER the title line — a stray `- "A"`
under a root key. Reproduce-first: 4 tests, 3 red against the old code.

### 2. A locked `libraries.json` could delete every library — `5144c610`
`ensure_universe_notes_folder` runs on EVERY universe activation and read the registry
with `.ok()…unwrap_or_default()`, which answers "there are no libraries" for a file it
merely failed to read — then atomically replaced it with a single entry. No error, no
backup (this site bypasses `load_libraries`' quarantine). One transient lock was enough.
**Absent is a fact; unreadable is an unknown.**

### 3. An external file change could land in the wrong note — `5144c610`
`adoptExternalChangeIntoTabs` iterated a tab snapshot captured BEFORE its awaited disk
reads. Fixed at source, **and structurally**: `adoptDisk` was the only model mutator
without an identity guard and now takes the same `expectPath` every other write path
proves. Solve-the-class — protection no longer depends on each caller being careful.

**Boss test:** Stage 1 (rename × nested property) PASS — disk confirmed Muqaddimah,
Ibn Khaldun and 1377 intact through rename + a tag edit, exactly one root `title:`.
Stage 2A (libraries intact across restart + two universe switches) PASS. Stage 2B
(Notepad edit to an open note arrives, right note, nothing disturbed) PASS.

## PJ-136 — the "Empty" row the Boss spotted — `8de291a2`
His Step-1 screenshot showed `source` rendering as **Empty**. Ran it down: two parsers,
one rule. `yamlDoc.projectProps` skips nested maps by design ("preserved in the CST,
not editable here — Boss decision"); `store.parseFrontmatter`, the hand-rolled line
parser feeding the note MODEL and therefore the visible panel, has no such rule.

Untouched, the data is safe — proven on disk. **Typing into the row composes
`source: <typed>` over the whole block**, proven by test: Muqaddimah, Ibn Khaldun, 1377
gone, no error. A field that reads Empty is an invitation to fill it in, so the wrong
label IS the defect. Three characterization tests pinned; the destructive one asserts
the bug and MUST go red when fixed.

**BOSS RULED: render it read-only with a summary of its children.** Next build.

## SO checklist
- **SO#1** session log — this file. **SO#6** orientation → **v3.63**. **SO#7** MoCh →
  `docs/MoCh/MoCh-2026-07-21-1400.md`. **SO#9** PJ ledger → **v1.43** (PJ-136 filed +
  Boss-ruled; PJ-134 closed — it was app-killer #2).
- **SO#2 help files / User Manual — reviewed, no change required.** All three fixes are
  behaviour corrections with no new or altered user-facing surface, and the recognition
  engine has no UI yet. PJ-136's fix WILL be user-facing and takes its help/manual pass
  in the same commit.
- **Rust 1120/0 · svelte-check 0 errors · vitest 602/602.**

---

## PJ-136 — the "Empty" row, closed — Boss-validated in two rounds

**Boss ruling:** *"Show it read-only with a summary."* Then, crucially: *"showing it
read-only is a temporary procedure, until you research for a solution and fix it for
good. Isn't it?"* — Yes. It is containment, not a decision, and the code and ledger now
say so in his words.

**What shipped.** A nested map (`source:` with `title`/`author`/`year` under it) is now a
first-class property type. The row shows its child field names as chips plus a faint
*read-only* label with a tooltip explaining why.

**The protection is in the WRITE PATH, not the widget.** `composeFrontmatter` refuses to
write or splice a `nested-map`, so the block survives however the panel behaves — the
same reasoning that gave `adoptDisk` an identity guard instead of trusting its callers. A
read-only widget protects data only while every caller keeps it read-only.

**Design detail that kept the blast radius at zero:** `value` stays EMPTY, exactly as
before; the summary rides in a new `nestedKeys` field. The legacy `reconstructFrontmatter`
(still live behind `buildFullContent`, which caches `tab.content`) therefore serializes
this key byte-identically to before. Nothing about existing write behaviour moved.

**Boss round-1 catch — the label was still lying, smaller.** *"The chips are read-only,
while the source is writable."* The key NAME was an editable input with autocomplete, and
renaming it would have looked like it worked then done nothing, because the write path
refuses the whole row. **A control that silently no-ops is the same silent-failure class
this whole day was spent removing.** Closed at the row level: the key renders as plain
text (reusing the existing `tags`/`aliases` span, no new UI), the × delete is gone, and
right-click drops *Remove property* while *Copy value* copies the field names shown.

**MY ERROR, logged.** The round-1 tutorial told the Boss to set `stage` to `sapling`. No
such stage exists — the vocabulary is Spark/Birth/Growth/Maturity/Dormancy/Archival ×
Seed. I had seen `maturity: sapling` as a property VALUE in his imported Wikipedia notes
and carried it across to a different field without checking. That is a BASIC-RULE
violation (invented factual detail in a tutorial). Grep before asserting a vocabulary.

**Verification.** 7 tests, incl. *a write against the nested map is refused, not applied*
and *dropping the row from the props does not splice the block*, plus guards that a real
list is still a list and an ordinary property still deletes. i18n ×15 (4 keys).
**Rust 1120/0 · svelte-check 0 errors · vitest 606/606.** Boss re-test 1–5 PASS. Disk
confirmed: `Muqaddimah` / `Ibn Khaldun` / `1377` byte-identical with indentation intact,
while `stage` moved to `spark-seed` beside them.

**SO#2 help/manual — DONE, and it surfaced a gap.** EN help topic *Properties* gained a
"Properties that contain other fields" section (incl. an explicit "this is temporary"
note and how to edit them in a text editor meanwhile); the User Manual §10 type list
gained the same entry. **The translated help sets have no Properties topic at all** — EN
carries 42 topics, ar/de/zh carry 19–21 — so this could only be documented in English.
Filed as **PJ-138**; not invented as 14 new half-topics.

**PJ ledger → v1.44** (PJ-136 closed; PJ-137 + PJ-138 filed).

---

## The regression I shipped — caught by the per-build inspection, fixed, Boss-validated

`98b71440` (PJ-136) introduced an APP-KILLER. The per-build inspection run immediately
after it — 108 agents — found it before any user did.

**Mechanism.** The nested-map guard was consulted in `composeFrontmatter`'s REMOVE loop
but **not in SET/ADD**, and the immutable set came from the **props array** rather than
the file. Safe only while the props report the type honestly — and the app manufactures
a dishonest array by itself: PropertyEditor caches `tab.content` via `buildFullContent`,
`reconstructFrontmatter` dropped the block's children from that cache, and re-parsing it
re-typed the key as ordinary empty text. The next edit of ANY property then reached
compose with the key absent from the old side and present on the new → the CST block-map
item was spliced and `source: ""` appended. Durable, silent, unrecoverable.

**Fixed twice over, either alone sufficient:** `nestedMapKeys(rawYaml)` reads the
immutable set from the FILE (and now guards SET/ADD too), and `nestedRaw` carries the
block's lines verbatim so the cache is lossless and the dishonest state never arises.

**Recorded reasoning error.** The commit message claimed *"zero blast radius by design."*
I verified that keeping `value` empty made `reconstructFrontmatter` byte-identical and
stopped there — I never traced the re-parse of the lossy cache. **I checked the step I
thought of and called it proof.** The deeper mistake: I made the file's own structure
something the CALLER DECLARES rather than something the FILE STATES. A props array can
lie; the file cannot. That is the generalizable lesson, not "add another guard".

## The same class, still live in the shared library loader

`load_libraries` collapses any failure into `vec![]`; `add_library` pushes its one new
entry onto that empty Vec and atomically renames a ONE-ENTRY `libraries.json` over a
registry holding 19. **This is the anti-pattern I fixed the same morning at
`universe.rs:380` — at the one call site instead of the shared loader, so the class
survived.** Solve the class, not the instance. `try_load_libraries` refuses on an
unreadable/unparsable registry, backs up before refusing; reads still degrade.

**Boss test 1–5 PASS** on the release binary, including the deliberate
edit → switch-away → edit sequence. Disk verified byte-identical afterwards.
**Rust 1123/0 · svelte-check 0 errors · vitest 609/609.**

## PJ-139 — and a misread I should record

Boss found that a deleted library folder leaves its registration behind and the sidebar
shows it as completely normal, and that **the right-click menu offers no way to remove a
library** (only *Manage libraries*, at the sidebar footer, does).

**My error in handling it:** I read his message as "it can't be removed anywhere", saw
RegressionCheck absent from his Manage-libraries screenshot, and told him the manager
could not see it — a wrong conclusion from a screenshot taken AFTER he had removed it.
Re-reading the registry settled it in one command: the entry was gone; the manager had
worked. **Check the state before narrating a diagnosis from a screenshot.**

**PJ ledger → v1.45** (both app-killers closed; PJ-139 filed).
