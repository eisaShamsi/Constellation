# MIG-103 — The Manuscript Builder: Wizard, or Something Better?

**A research synthesis for the decision.** *2026-07-21.*

You asked us to "study and research the wizard, and we will decide upon the results." You used the word *wizard*, but you also rejected the "one-way rail." This paper answers whether those two things are the same, what the evidence says the manuscript builder should actually be, and how big a build it is given what Constellation already owns.

Five research tracks fed this synthesis:

- **Track A** — the wizard as an HCI/UX interaction pattern (when it works, when it becomes a rail).
- **Track B** — how 8 leading real manuscript/book tools actually scaffold a new long-form project.
- **Track C** — how the Table of Contents gets built, and how existing documents get slotted into a structure.
- **Track D** — how to keep a guided creative flow two-way instead of a linear rail.
- **Track E** — what Constellation *already has* that a builder would orchestrate (with file:line).

Every claim below keeps its research **grade**: **sourced** (read from an authoritative primary source this pass), **recalled** (corroborated but not read from the primary source), or an attested **gap** (a proven-absence, or an absence we could not fully rule out). Where sources conflict, the conflict is shown — not averaged.

---

## 1. The verdict, up front

**A step-by-step interrogative wizard is the wrong pattern for the manuscript builder. The right pattern is: pick a project type → a whole structure appears at once → you edit it directly.** The field calls this "template-that-expands + direct manipulation." Constellation already has the parts to build it.

### The product count — the single most decisive fact

**Across 8 leading long-form authoring tools studied, ZERO use a step-by-step interrogative wizard to build a book.** *(Track B, sourced — each tool independently.)* Every one of them uses "pick a project type, a whole structure appears, you edit it directly," or structure-by-declaration:

| Tool | How a new long-form project is scaffolded | Grade |
|---|---|---|
| **Scrivener** (the reference tool for this job) | Template chooser → name/locate → Create → the project opens **directly** with the Binder structure (folders + placeholder docs) already laid down; everything after is direct manipulation of the Binder. No wizard. | sourced |
| **Obsidian Longform** (closest Markdown-native analogue) | Structure is an **ordered scene list in the index note's YAML frontmatter**; you add scenes and drag-drop to reorder/nest, and every reorder writes straight back to the frontmatter array. No wizard. | sourced |
| **Ulysses** | Nestable Groups of Sheets, arranged by direct manipulation (drag/merge/split/glue); the manuscript is produced by selecting sheets and exporting. No wizard. | sourced |
| **yWriter** | Outline tree of chapters/scenes; the only "quantity" moment is "Create Multiple Chapters" — type a count, they appear, reorder later. One instant expansion, not a rail. | sourced |
| **Manuskript** | Outliner — draft a tree of folders/scenes, drag-drop any element to reorder. No wizard. | sourced |
| **Notion** | Duplicate a template that is itself a whole page-tree (parent + all children); edit the tree directly. No wizard. | sourced |
| **Word** | Structure *declares* the document (Heading 1/2/3 styles); the TOC is generated from it. No wizard. | sourced |
| **LaTeX** | `\chapter`/`\section` commands declare structure; `\tableofcontents` generates the TOC automatically. No wizard. | sourced |

This is not a close call. The mature pattern the whole field converged on is **stamp-a-structure-then-edit**, not **interview-the-user**.

### The HCI conditions — when a wizard IS appropriate, and why a book fails them

Nielsen Norman Group's canonical definition *(Track A, sourced — Budiu, NN/g "Wizards," 2017)*: a wizard is "a step-by-step process that allows users to input information in a prescribed order and in which subsequent steps may depend on information entered in previous ones." NN/g says wizards are **appropriate** for novice users and infrequent/complex setup/configuration, and **inappropriate** for repeated/frequent use, expert users with their own mental models, tasks needing comparison across fields, and tasks needing external information.

Here is the tension in the evidence, surfaced honestly:

- On the **appropriateness axes** (infrequent, complex, some real step dependency), a book-build *looks* wizard-appropriate. That is why the word "wizard" is a natural first instinct.
- But NN/g **explicitly names expert-creative work as where wizards are the wrong pattern** *(Track D, sourced — same NN/g article)*: wizards "limit the users' control and creativity," and it would be "too limiting for a professional graphic designer to use a wizard to improve an image." A book/thesis build is the archetypal expert-creative task, not a novice config task.

So the two readings collide — and the tie-breaker is empirical, not theoretical: **NN/g's own reported failure case.** Eventbrite's event-creation wizard failed because "users' mental models of setting up a new event didn't include all the necessary steps, and they often dropped off and didn't complete the wizard" *(Track A, sourced — but see §6: this case lives in NN/g's "Best Application Designs" article, not the Wizards article the research first cited)*. That is the exact anchor for your "rail" objection: **a wizard becomes a rail — and users abandon it — when the APP decides the steps before meeting the user, and its steps are not this user's steps.**

A second real-world case reinforces it *(Track D, sourced — UXmatters, "Wizards Versus Forms," 2011)*: warehouse users rebelled against a wizard that replaced batch spreadsheet entry, and the deployment team had to write custom code to restore the direct/batch workflow. The recurring axis in the literature: "novices and infrequent users like wizards, but frequent and power users prefer forms."

**Verdict restated plainly:** you asked us to decide on the results. The results say do **not** build a linear step-by-step wizard. Build a **proposer + editable canvas** — the app proposes a complete draft structure from the chosen type and from your own notes, shows it whole, and hands it to you to rearrange directly. That is not a contradiction of "wizard"; it is the humane thing the word was reaching for, minus the rail.

---

## 2. The reconciliation with The Constellation Way — the checklist

The distinction is not "wizard good or bad." It is **rail-shaped flow vs. proposal-shaped flow.** A rail decides the steps, the order, and what-matters *before meeting you*, one-way and sequential. A proposal offers a complete editable structure *derived from you*, two-way and non-sequential.

Here is the gate. If the manuscript builder satisfies all nine, it is an assistant (The Constellation Way). If it enforces order or interrogates before proposing, it has regressed into the rejected rail. *(Track D synthesis — each item traces to a sourced finding.)*

| # | Property the builder must satisfy | Evidence it rests on | Grade |
|---|---|---|---|
| 1 | **Opt-in / user-invoked for a stated goal** ("start a book project") — starts from your consent and your scope, not the app's initiative. | Mixed-initiative interaction (Horvitz, CHI '99) + user-initiated vs system-initiated agency distinction. | sourced (paper exists/thesis); recalled (the 12 principles' exact wording) |
| 2 | **Proposes a full, editable artifact up front** — not ordered questions. Pre-fills a proposed title and a proposed structure rather than asking blank questions. | Smart defaults beat empty forms (Johnson & Goldstein 2003: opt-in 42%→82% by flipping the default); NN/g "reuse previous selections as defaults." | sourced |
| 3 | **The artifact is directly manipulable** — drag/reorder/rename the TOC with immediate visible feedback. | Direct manipulation (Shneiderman; NN/g "Direct Manipulation" — raises "the sense of control"). | sourced (via NN/g); the 3 Shneiderman principles recalled in Track A, sourced via NN/g in Track D |
| 4 | **Every proposed item is pre-filled from your OWN evidence and one-click removable.** | Smart defaults + The Constellation Way ("propose from the user's own evidence"). | sourced |
| 5 | **The reasoning for each proposal is shown** ("these 6 notes matched because they carry heading/tag X"). This is the guard that stops a smart default from becoming a silent rail — because defaults are *sticky* and obeyed uncritically. | The default effect (Johnson & Goldstein) + mixed-initiative uncertainty-scoping + The Constellation Way. | sourced |
| 6 | **Non-modal and resumable** — exit, reorder, skip, return; never modal lock-in. | NN/g complex-app guideline against rigid linear workflows; NN/g wizard drawbacks (modal lock-in). | sourced |
| 7 | **No enforced step order** — any part reachable any time; the ONE exception is a genuine data dependency (you cannot build the TOC before a structure is chosen). | NN/g: "avoid rigid, linear workflows... provide skipping ahead, looping back, moving fluidly from any step to any other." | sourced |
| 8 | **Every action reversible.** | Direct manipulation (reversible incremental operations). | sourced |
| 9 | **Escape hatches everywhere** — including an escape to plain direct editing of the notes. | NN/g complex-app guideline. | sourced |

**Two named mechanisms turn a forced sequence into a reviewable proposal** *(Track A, sourced — GOV.UK Design System)*:

- **"Don't ask what you already know."** GOV.UK: "only ask for a piece of information once within a single journey... do not ask a user to re-enter information they've already provided." This *is* your "don't interrogate / observe from the user's own evidence."
- **"Check answers + Change links."** GOV.UK ends a flow with a summary page listing every answer, each with a "Change" link. This *is* "propose, then let the user decide/edit" at the whole-flow level: the builder should end by **showing the assembled TOC + note plan as one reviewable summary where every element is a Change link**, not commit silently at the last "Next."

**One caution against over-building.** GOV.UK explicitly warns against rich progress indicators that "show all questions at once" and let users jump around — research showed they confuse *(Track A, sourced)*. This tempers the Material Design "jump-anywhere stepper" enthusiasm. The synthesis: **minimize the guided part, maximize the editable output** — a short proposal phase that resolves into a directly-editable artifact, not a many-node stepper the user navigates.

The repo already forbids the rail. Constellation's own rulings — the Uninterrupted Stream ("a mid-flow prompt is a wizard with one step"), "TWO-WAY DOES NOT MEAN INTERROGATIVE," and MIG-103's "a stated need is not an invitation to interrogate" — already draw this exact line *(Track E, sourced — Note-Shape-and-Template-Studio-Brainstorm.md:492-557, 636-655)*.

---

## 3. The recommended model, walked end to end

**The model: a template-that-expands, populated from your own notes, shown whole, edited directly.** One guided moment (choosing the project type) that expands into a complete proposed structure; everything after is direct manipulation of a real artifact. Below, each step names what the app **PROPOSES** vs what it **ASKS**, grounded in a Constellation primitive that already exists (Track E, all sourced with file:line unless noted).

**The book example, start to finish:**

**Step 0 — Invocation (opt-in).**
You explicitly start a book project. *Nothing is surfaced until you state the need.* — Satisfies checklist #1. Grounded in the repo's own "request path meets a stated need plainly" ruling.

**Step 1 — Choose the project type. (The ONE defensible guided moment.)**
The app **ASKS** exactly one thing: which structure — book / thesis / treatise / essay (Arabic: كتاب → باب → فصل, etc.). This single pick is what expands into the whole proposed structure — the same one instant expansion Scrivener and yWriter use.
- *Why this is legitimate and not a rail:* it is the only real data dependency in the whole flow — the TOC is *derived* from the structure choice, so it genuinely must come first (checklist #7 exception).
- *Grounded in:* the compositional-forms concept is **already written down and repo-verified** — a book is the "structured" mode realized over the PJ-065 structural lane *(Note-Shape-and-Template-Studio-Brainstorm.md:751-788)*. **Note:** today `shape:` models only `scrap`/`page` — "book/chapter" is **not** a shape *(shape.rs:45; test at :447 asserts `journal` is not a valid shape)*. So "book" is a *new named compositional mode* to surface, not an existing field to reuse. This is a concept-layer build, but the concept work is done.

**Step 2 — Title.**
The app **PROPOSES** a title (e.g. from the folder name or a seed note) as an editable, pre-filled field — it does not ASK with a blank box. — Checklist #2, smart defaults.

**Step 3 — The structure appears WHOLE.**
The app **PROPOSES** a complete draft skeleton at once: parts → chapters → sections, from the chosen type's template. You see it whole and rearrange by drag/reorder/rename. It does not march you chapter-by-chapter.
- *Grounded in:* the **PJ-065 structural lane is fully built** — a parent "TOC" note declares an ordered child list via `contains:` frontmatter (carrying `seq` order), and/or children declare `parent:`; the indexer folds these into ordered `note_links` rows *(structural.rs:1-18; proven by test `contains_frontmatter_becomes_ordered_structural_edges` in search.rs:12203 — `contains:` list → ordered edges seq=1, seq=2)*. The builder writes frontmatter; the existing indexer produces the spine. **No new link storage, ordering, cycle-guard, or single-parent logic is needed.** The **Structure panel already renders the outline** and a Keep/Move-here resolver already handles contested parents *(StructuralOutlinePanel.svelte; resolve_structural_conflict at libraries.rs:1682, on the rename-cascade gate)*. And structural edges are deliberately **non-cognitive** — a 40-chapter book will *not* flood Sky View or the link graph *(link_types.rs:229 `is_structural`, :268 `structural_not_in_clause`)*.

**Step 4 — Populate: use-existing vs create-new. (The two-way heart.)**
For each slot, the app **PROPOSES** — "these notes from your library appear to fit Chapter 3" — **and SHOWS the reasoning**; you accept, reject, or drag your own. For empty slots it creates fresh notes. — Checklist #4 + #5.
- *Grounded in:* `create_note` **already accepts** a template's processed body (`initial_body`) and arbitrary extra frontmatter (`initial_frontmatter`), merging everything except identity keys *(libraries.rs:792; merge at :767-789)*. Each chapter can be created in one existing call carrying its template body + a `parent: [[Book]]` line, or the TOC note created carrying the whole `contains:` list.
- *What is genuinely NEW here (see §3 sizing):* (a) a **batch/scaffold orchestration** that expands one intent into many linked notes + a TOC note in one resumable transaction — `create_note` is strictly single-note today, no scaffold command exists; and (b) a command to **enroll an EXISTING note into a parent's TOC** — today the only structural *write* command is the conflict resolver; there is no `append_contains`/`set_note_parent` "add child" action *(Track E, sourced via grep — only resolve_structural_conflict exists)*. Both can be modeled on the proven `resolve_structural_conflict` gate path.

**Step 5 — Review the whole thing (Check-answers).**
The app **PROPOSES** the assembled TOC + note plan as one reviewable summary; every element is a "Change" link. Nothing commits silently. — Checklist #5 + the GOV.UK Check-answers pattern.

**How big is this, really?** *(Track E headline — the fourth "kind" is MEDIUM, not huge.)* Storage, ordering, rename-safety, outline rendering, contested-parent resolution, per-note create-with-template, and the "book = structured composition" concept **all already exist**. What must be *built* is: (1) the multi-note batch-scaffold wrapper; (2) the enroll-existing-note-into-TOC command; (3) surfacing "book/thesis" as a named compositional mode (concept done, UI not); (4) validated per-tradition skeleton **template files** (Arabic drafts exist but are unvalidated research — 10 open items — *MIG-103-R1-Standards-and-Case-Studies.md:160-190*; Japanese/Western are named but undrafted). **The builder orchestrates reality; it does not invent the spine.**

---

## 4. The TOC, and populating from existing notes

### The TOC is a DERIVED artifact, not a hand-built index

Every attested tool treats the TOC as a *generated view of a structure*, refreshed on demand — never a hand-curated list the user keeps in sync *(Track C, all sourced)*:

- **Word** — the TOC is a generated field derived from Heading 1/2/3 styles, refreshed via "Update entire table." *(Note: the research's parenthetical "Markdown" generalization is loose — Markdown the syntax has no TOC feature; this is a property of specific tools, per §6.)*
- **LaTeX** — `\tableofcontents` generated automatically from sectioning commands.
- **Scrivener** — TOC derived from Binder hierarchy + each item's Section Type; either a one-shot "Copy Documents as ToC" snapshot or the `<$toc>` placeholder auto-filled on Compile.
- **Obsidian Waypoint** — the closest external analogue to what Constellation needs: a **generated-but-persisted** TOC saved "as real markdown text... still usable" in any `[[links]]`-supporting editor, auto-updated on file create/rename/move/delete.

**This directly endorses Constellation's own write-time-derivation rule (CLAUDE.md Rule 8):** the TOC note should be *generated from the chosen structure and re-derived when notes are added/reordered/renamed, in the same transaction* — and, per Waypoint and File-Over-App, still be a **real `.md` file**. The PJ-065 lane already computes ancestors/descendants on read and folds `contains:`/`parent:` into ordered edges at index time, so the derived-TOC shape is native to what Constellation already has. Cheap restructuring is only cheap *because* nothing is hand-maintained — reorder is a structural-link edit, and the TOC follows automatically.

### Slotting existing notes into a structure — the attested UX

In every attested writing tool, assigning an existing document to a chapter slot is a **manual, user-initiated gesture** *(Track C, sourced)* — in three shapes:

- **Scrivener** — drag the file in the Binder (drop-onto = nest as child, drop-between = reorder).
- **Longform** — write a note anywhere, then the plugin **detects it and prompts "add or ignore"**, or you insert frontmatter; then reorder.
- **Notion** — explicit "/link to page" for an existing page vs "/page" for a new one.

This validates the "user decides" half of The Constellation Way: **assignment is always an explicit user act, never silent.** Constellation has a proven vocabulary to copy (drag, detect-then-confirm, link-picker).

### Does anything PROPOSE which existing notes fit? — The genuine novelty

**No.** We could not find any dedicated manuscript or PKM tool that *evidence-drives* the assignment — that scans your own note corpus and proposes "this note fits Chapter 3." *(Track C, attested gap.)* AI note tools (Mindgrasp, Musely, Taskade, Notion AI) organize content you *feed into* them (content → outline), or suggest tags/relations at the property level — but none map an existing vault's notes into named chapter slots. In every attested writing tool the note→slot assignment is 100% manual.

**This is exactly the Constellation Way's differentiator, and it has no external template to copy:** observe + propose from the user's *own* evidence, show the reasoning, user decides. It is not a wizard rail, because the proposal is *derived from the user's material* rather than decided before meeting them, and each slotting is accepted or rejected. **Build it as a proposal *layer on top of* the proven manual drag/link fallback — never as the only path.** *(Grade: attested gap, not a proven universal negative — we cannot say for certain the market lacks it entirely, only that our searches surfaced none.)*

---

## 5. The Boss decision points

Five either/or choices. Each is a plain-language decision that becomes an R-question for the fourth-kind build.

**Decision 1 (the headline) — Linear wizard, or propose-editable-draft-structure?**
- **Option A:** a step-by-step wizard that asks you questions in a fixed order and assembles the book from your answers.
- **Option B:** the app proposes a complete editable draft structure (from the chosen type + your own notes), shows it whole, and you edit it directly.
- **Recommendation: Option B.** The evidence is one-sided — 8 of 8 leading tools use B; NN/g says a wizard is the wrong pattern for expert-creative work; and B *is* The Constellation Way. This is the rejection of the rail you already made, confirmed by the field.

**Decision 2 — One guided moment (project type), or a multi-step chrome?**
- **Option A:** minimize the guided part to a single pick (project type) that expands into everything.
- **Option B:** a multi-node stepper the user navigates.
- **Recommendation: Option A.** GOV.UK warns that jump-around progress chrome confuses; the safe synthesis is "minimize the wizard, maximize the editable output." The type-pick is the only true data dependency; everything else is direct editing.

**Decision 3 — Build the evidence-driven "propose which of your notes fit" step now, or ship manual-slotting first?**
- **Option A:** build the smart proposal (scan the library, propose notes-per-slot, show reasoning) as part of v1.
- **Option B:** ship the proven manual drag/link slotting first; add the proposal layer after.
- **Recommendation: Option B as the floor, Option A as the differentiator layered on top.** The manual path is proven prior art and is the fallback that must always exist. The proposal step is un-attested novelty (higher risk, higher payoff) — build it *on top of* the fallback, with visible reasoning and one-click reject, never as the only path.

**Decision 4 — Model "book/thesis" as a first-class named compositional mode, or leave it as "a plain note that carries `contains:`"?**
- **Option A:** introduce a named "structured composition" concept the builder surfaces (concept already written and repo-verified).
- **Option B:** don't name it — a book is just any note with a `contains:` list.
- **Recommendation: Option A.** The concept work is done and verified; surfacing it is what makes the fourth kind legible to the user. But note this is the one genuinely *new concept-layer* piece — worth a clear ruling before build. `shape:` cannot carry it (it's closed to scrap/page).

**Decision 5 — TOC as a persisted, write-time-derived `.md` file, or a live-rendered view?**
- **Option A:** persist the TOC as a real `.md` note, re-derived from structural links on every structure edit (Waypoint-style, File-Over-App).
- **Option B:** render it live at read time, nothing persisted.
- **Recommendation: Option A.** It matches Waypoint (the closest analogue), CLAUDE.md Rule 8 (write-time derivation), and File-Over-App (the TOC survives as a plain file). Option B would violate the write-time-derivation rule already standing in the codebase.

---

## 6. Verification record — what was refuted or corrected

The research ran an adversarial verification pass. The load-bearing findings held; the corrections below matter for accuracy when the manuscript cites them.

**Confirmed (load-bearing, no change):**
- NN/g's wizard definition, appropriate/inappropriate lists, five design recommendations, and five drawbacks — **CONFIRMED verbatim** against the primary source (Budiu, NN/g "Wizards," 2017).
- NN/g's expert-creative critique (wizards "limit control and creativity"; the graphic-designer counter-example) — **CONFIRMED verbatim.**
- NN/g's complex-app guideline against rigid linear workflows (skip/loop-back/any-step navigation) — **CONFIRMED verbatim.**
- Scrivener template-chooser flow, Longform frontmatter scene-array, Ulysses groups/sheets, Word heading-derived TOC, Scrivener Section-Type TOC — all **CONFIRMED** against product docs.
- UXmatters wizards-vs-forms guideline and the warehouse-rebellion case — **CONFIRMED** against the primary article.

**Corrections to carry (substance holds; cite accurately):**
1. **Eventbrite failure case — misattributed URL.** The case is genuine NN/g reporting and the paraphrase is near-verbatim, **but it appears in NN/g's "Best Application Designs" article (nngroup.com/articles/best-application-designs/), NOT the "Wizards" article** the research first cited. The Wizards article's named examples are TurboTax, Fidelity, Mint, etc. — not Eventbrite. *Correct the citation before publishing.*
2. **Scrivener "opens directly... no separate wizard dialog step"** — presented as a verbatim quote but is a **paraphrase**; the substance (no interrogation wizard) is confirmed. Also: after Create there is a standard OS Save-As dialog (a file-save step, not a wizard).
3. **Ulysses "compile"** — Ulysses' docs explicitly say "there is no need to compile" and use **"export"**; the "compiles... into one continuous document" quote is a paraphrase. Mechanism is correct.
4. **Obsidian TOC — Folder Index misattributed.** Only **Waypoint** persists the TOC as *real markdown links* (its explicit differentiator: "Unlike other plugins..."). **Folder Index renders its index dynamically via a code-block processor at read time** — not persisted real-markdown links. The design conclusion rests on Waypoint alone, so it stands; but do not claim Folder Index persists real markdown.
5. **UXmatters ≠ Tidwell.** The claim that the UXmatters article "references Jenifer Tidwell's *Designing Interfaces* wizard pattern" is **false** — no such reference appears. Drop that attribution.
6. **Word "Markdown" generalization is loose** — Markdown the syntax has no TOC feature; TOC-from-headings is a property of specific tools/generators. The Word-anchored core is confirmed.

**Grade cautions (not refutations, but not fully sourced):**
- **Shneiderman's three direct-manipulation principles** — in Track A the primary PDF failed to parse (graded *recalled*); in Track D they were sourced via NN/g's "Direct Manipulation" article. Treat as sourced-via-NN/g, recalled-from-Shneiderman-directly.
- **Horvitz's 12 mixed-initiative principles** — the paper's existence, venue (CHI '99), and thesis are sourced; the *exact wording* of individual principles is recalled.
- **Johnson & Goldstein defaults** — the **lab default effect** (42%→82%) is robustly sourced; the **real-world "saves lives" organ-donation translation is contested** in follow-up literature (opt-out registration ≠ actual donation rates). Cite the lab effect, not the life-saving claim.
- **"Draft-then-refine beats the blank canvas"** — graded **recalled**; the readily-available sources are vendor marketing, not a controlled study. **No rigorous head-to-head experiment was found** comparing "propose a full editable structure then prune" against "ask step-by-step chapter-by-chapter" for creative document structuring. The direction is *strongly implied* by the sourced findings (defaults beat empty forms; direct manipulation; wizards limit creativity) — treat it as strongly-implied best practice, not proven fact.

**Open conflict left un-averaged:** NN/g's *own* wizard guidance says "enforce a clear sequential order of the steps," while Material Design (non-linear/editable steppers) and GOV.UK step-by-step allow non-linear order. The literature gives no single universal rule. The synthesis — **gate order only where a real data dependency exists (structure → TOC), make everything else non-linear and editable** — is *our reconciliation of conflicting guidance*, not a single sourced ruling. It is also precisely why the builder must NOT be architected as a wizard (which would inherit "enforce order") but as a proposer + editable canvas.

---

*Prepared from five verified research tracks. Repo claims verified against source with file:line (Track E). External claims graded sourced / recalled / gap and never averaged where they conflict.*
