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
