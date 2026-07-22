# MIG-103 — The Template Studio
## `/migration` Phase 2 — the Plan

---

## ★★ THE CONCEPT (the horse) — articulated from the origin, Boss-approved 2026-07-22 ★★

*Written after the Boss asked: "Go back to the origin of the Templates Studio to understand the
reason we created it in the first place. And, out of it, articulate the concept." The origin is the
2026-07-19 directive ("My Templates engine is not working… create a state-of-the-art template
engine"), the two framing corrections that followed, and the question that actually created the
Studio — "Like, 3M Post-it, can we consider it a note? If yes, what shape will it take?"*

> **A template is the captured shape of a recurring cognitive move** — the question *"what shape
> does this kind of thinking take?"*, asked once and answered permanently. **A template is a MOLD;
> a note is a CAST.**
>
> **The Template Studio is the workshop where molds are made, seen, chosen and tended.**
>
> **A mold reaches the workshop by one of TWO ROADS, and neither is secondary.**
>
> **ROAD ONE — you already made the shape.** You wrote the note, or twenty of them, and the form is
> sitting there in your own material. The Studio takes the impression: from one note you point at,
> or from a pattern it finds across many and hands back for you to name. Nothing is invented; the
> cast came first.
>
> **ROAD TWO — you know the shape you need.** No note exists yet; the form is in your head. You say
> what it is, plainly — these fields, these sections, this look — and the mold exists. **This road
> must be the simple one:** a handful of ordinary choices, no ceremony, no wizard, and no
> requirement to have written anything first.
>
> Both roads end in the same object: an ordinary `.md` you own, that shapes every note cast from
> it, and whose look belongs to the KIND rather than the instance.

**Why both roads, and not one.** Formulation runs in both directions. Sometimes you *discover* the
shape by writing — you do not know it is a "contradiction workup" until you have made four. Sometimes
you *decide* the shape and then write into it. A workshop that only took impressions would force the
user to write badly-shaped notes just to earn a good shape; a workshop that only took declarations
would make them specify everything up front, which is the rail the Boss has ruled against twice.

**The load-bearing inheritance from the origin** (do not drop these again):
- **The look is not decoration.** *"The look tells you what kind of thinking is inside before you
  read a word — the reason a Post-it, a legal pad and a leather journal don't look alike on a real
  desk."* This is why it is a *studio*, and why R4 (its own style setter) and R5 (a visual library)
  exist at all.
- **Style belongs to the mold; every cast inherits it.** The moment one note can be styled freely,
  the look stops meaning anything. Style is a property of the KIND, not the instance.
- **The plain path is guarded hardest.** Writing a `.md` containing `{{date}}` and using it must
  remain a complete, respectable way to work — **never opening the Studio**. Styling is an upgrade,
  not a gate.
- **The mold has no identity and is inert.** No `cid_cn`, no `created`, no scripts, no side effects.

**Recorded failure this concept prevents (2026-07-22):** I briefed the Art Director to design the
Studio's dock icon from a concept I invented for one panel ("recognition of recurrence"), instead of
from the Studio's own concept. The brief therefore contained no trace of the visual-signal dimension
— the very reason the Studio exists. **Any new requirement starts with the concept (the horse);
a component never gets a concept of its own invented for it.** — Boss, 2026-07-22.

---

## ★★ LOCKED DESIGN (2026-07-21) — all Boss rulings consolidated ★★

*This block supersedes the scattered R-notes below. Every decision here is Boss-confirmed via the
research→decide loop. Three research passes fed it, all committed:
`MIG-103-R1-Standards-and-Case-Studies.md`, `MIG-103-Template-Use-Interaction-Model.md`,
`MIG-103-Manuscript-Builder-Wizard-Research.md`.*

### The FOUR kinds of template (Boss taxonomy, 2026-07-21)

A Constellation note is **frontmatter + body**, so a template can carry either part, both, or a
*structure*. The four kinds each pair with one action:

| # | Kind | Payload | Action |
|---|---|---|---|
| 1 | **Whole note** | frontmatter + body | **create** a new note (or fill an empty one) |
| 2 | **Frontmatter only** | properties, no body | **apply** — merge properties into the note you're in |
| 3 | **Snippet** | a body fragment | **insert** at the cursor |
| 4 | **Manuscript / project** | a *structure* | **build** many linked notes + a derived TOC |

Kind 2's "apply" action is the one my earlier snippet/scaffold binary missed — the Boss's separation
of frontmatter as its own kind unlocked it.

### The interaction model (all Boss-confirmed)

- **Save side** — "Save as template" lets you choose which of kinds 1–3 you're making (replaces the
  earlier full / structure-only split, which was a hybrid).
- **Use, no note open (empty state)** → **offer to create a new note** (route to kind 1). No more silent no-op.
- **Use, open note WITH content** → **insert body at cursor** (kind 3), and **show a heads-up before
  mixing** *(Boss chose the heads-up — more cautious than any product; his call).*
- **Use, new note — destination** → **propose the active-context library/folder and SHOW it, one click
  to change.** Never the silent "first library" guess (no product precedent; non-deterministic).
- **Use, new note, nothing open** → **show a library picker** *(Boss chose the picker over a named home).*
- **Frontmatter merge (kind 2 & the FM carried by kind 1)** → **never dump YAML as body text**; merge
  keys, never duplicate `title:`, never clobber identity keys; one undoable transaction. *(The #1
  hazard — the unsolved bug in Templater #1387; our Insert already strips template FM, so we're safe
  on that path today.)*

### The fourth kind — the manuscript builder (Boss-confirmed, 2026-07-21)

**NOT a wizard. A proposer + editable canvas.** *(8 of 8 leading manuscript tools use this; zero use a
step-by-step wizard. NN/g: wizards are the wrong pattern for expert-creative work.)*

- **Model** → **propose a complete editable draft structure** (from the chosen type + your own notes),
  show it whole, edit it directly. One guided moment (pick the type) that expands into everything;
  after that, pure direct manipulation.
- **Smart slotting** ("these notes fit Chapter 3") → **manual drag/link first as the floor; the
  evidence-driven proposal layered on top**, with visible reasoning and one-click reject. Never the
  only path. *(This proposal is genuine novelty — no manuscript/PKM tool does it; it IS the
  Constellation Way's differentiator.)*
- **Named mode** → **"structured composition" is a first-class named thing** the builder surfaces.
  `shape:` cannot carry it (closed to scrap/page) — this is the one new concept-layer piece.
- **TOC** → a **persisted, write-time-derived `.md` note**, re-derived from structural links on every
  structure edit (Waypoint-style + Rule 8 + File-Over-App).
- **Size: MEDIUM, not huge.** The PJ-065 structural lane (`structural.rs`; `contains:`/`parent:` →
  ordered edges, proven by test at `search.rs:12203`), the Structure panel, `create_note` with
  `initial_body`/`initial_frontmatter` (`libraries.rs:792`), and the "book = structured composition"
  concept all EXIST. New: a batch-scaffold wrapper, an enroll-existing-note-into-TOC command, the
  named mode surfaced in UI, and validated per-tradition skeleton files.
- **The 9-item rail-vs-proposal checklist** (research §2) governs the build — satisfy all 9 or it has
  regressed to the rejected rail.

### Re-scoped phase sequence

- **§1 — the three note-kinds round-trip** *(save + use)*: save-as (kinds 1–3), and use (create /
  apply / insert) with the confirmed interaction model — empty-state offer, destination propose+show,
  no-open picker, mixing heads-up, FM-merge safety. *(Save-as-full/structure + new-from-template are
  BUILT but need re-shaping to the three kinds + the interaction rulings before they're final.)*
- **§1B — ROAD TWO: make a mold from nothing** *(added 2026-07-22 on the Boss's ruling — "the other
  side should cover creating a template from scratch, to meet one's need, the simple way")*. The
  concept's second road had no phase; it was a throwaway clause in a sentence. It is a FOUNDING
  phase, not an add-on: §1 and §1B together are the two ways a mold comes into being. Simplicity is
  the requirement, not a nicety — a few plain choices, no ceremony, no wizard, nothing written first.
- **§2 — the request path** (type a type → get it; your molds outrank curated defaults; Arabic
  defaults from R3, Boss-validated). **Note the distinction from §1B:** §2 is *receiving someone
  else's mold* (a curated or Arabic default); §1B is *making your own*. They are not the same act.
- **§3 — the Studio surface as a core-plugin app-within-app, with the visual template gallery** (R2/R5).
- **§4 — recognition** (the smart library).
- **§5 — the manuscript builder** (the fourth kind — proposer + editable canvas; manual floor first).
- **§6 — the Studio's own style setter** (R4).
- **§7 — tending.**

**Honest limit recorded:** the "not-a-linear-wizard" verdict is strongly sourced (8/8 + NN/g); the
finer "propose-full-draft-then-prune beats chapter-by-chapter" is strongly-*implied*, not proven — no
controlled study exists. We build on the strong finding; the finer one is a reasonable bet, revisited
if a Boss test says otherwise.

---

**Boss ruling 2026-07-21: "Let's focus on the Template Studio/engine. For me, it is priority one now."**

**Concept (the horse):** *The Template Studio exists to recognise the shapes you are already writing
in, and let you name them.* Not invention — **taking an impression from the casts already made.**

**Function in hand:** the Template Studio — the surface where templates are created, tended and
chosen — and the engine that fills them in.

**Phase 1 (Architect) exists:** `docs/concept-papers/Note-Shape-and-Template-Studio-Brainstorm.md`
(the concept + five Boss rulings) and `docs/concept-papers/MIG-TPL-Templates-v2-Architect.md`.
MIG-TPL §1 (the plumbing fix — commit `f3133666`) already shipped and is Boss-validated: the
template folder is real, visible, honoured, and the placeholder engine's four corruption defects are
fixed. **This migration builds the Studio on top of that working engine.**

---

## 0. The Boss rulings this plan is bound by (all recorded in the concept paper)

1. **The Constellation Way** — two-way; the app observes and proposes from the user's own evidence,
   the user decides. A wizard is a rail.
2. **A stated need is not an invitation to interrogate** — *"If I ask for a book template, then
   Constellation should provide one, not something else."* The request path answers plainly;
   recognition lives only where no need was stated.
3. **Your practice outranks the default** — asking for *Book* when you have your own Book mold gives
   you yours.
4. **Cultural contrast is STRUCTURAL** — default templates are per-language artifacts authored
   natively, never translated shapes. Arabic first (the Boss's own taxonomy maps it); nothing
   authored from general knowledge.
5. **Cross-tradition is free** — *"an Arabic user can write a book using a Spanish structure."* The
   native default is a default, never a constraint.
6. **No creation stamps in a template** — *"creating a template, whether from a new file or based on
   an existing one, shouldn't include the creation time & date, or the cid_cn."*
7. **Don't gate capture** (verified research, and the one finding that survived the book-history
   collapse): classification is never a toll before writing.

## Codebase facts verified this session

| Fact | Where |
|---|---|
| Engine: sync `{{date/time/title/folder/library/cursor}}` + async `{{clipboard/frontmatter.KEY/file.*/yesterday/tomorrow/date±N/prompt:Q/suggester:…}}` — all four corruption defects fixed | `src/lib/templates/engine.ts` (246 lines) |
| `resolve_templates_dir` honours `appSettings.templateFolder`, default visible `Templates/`; `list_templates` | `src-tauri/src/universe.rs:1791/1830` |
| Picker + prompt + suggester UI exist | `TemplatePicker/TemplatePrompt/TemplateSuggester.svelte` |
| `create_note` stamps `title`/`cid_cn`/`kind`/`created` and strips those keys from passed frontmatter | `libraries.rs:784–801` |
| **⚠ DEFECT found writing this plan:** the `initial_frontmatter` merge pushes each line **`trim()`ed**, destroying nested-YAML indentation — a template carrying `source:\n  author: X` arrives corrupted | `libraries.rs:789–800` |
| `note_meta` already holds properties/tags/headings/links/word-count per note — recognition's data is already derived | `search.rs:2949` |

---

## The phases

### §1 — "Save as Template" — the impression-taking gesture *(the Studio's founding act)*

The Studio's concept says molds are taken from casts. So the first mechanism is: **any note → ⋯ menu
→ "Save as template."**

- **1a.** New Rust `create_template(note_path, template_name)`: reads the note, **strips identity**
  (`cid_cn`, `created`, `title` — ruling #6) and the `shape`/stage lineage values, keeps the
  structural skeleton, writes to the templates folder. Never overwrites (create-exclusive).
- **1b.** Fix the `initial_frontmatter` trim defect (`libraries.rs:789`) — preserve each line's own
  indentation so nested YAML survives the template→note journey. Red-first test.
- **1c.** What "skeleton" keeps vs drops is a **Boss ruling** (§R1 below) — the default build ships
  the recommended option, switchable later.

> **Boss test:** take a real note with properties + headings → Save as template → open the template
> file: no `cid_cn`, no `created`; structure intact including nested properties. Create a new note
> from it: filled correctly, identity freshly stamped.

### §1B — ROAD TWO: make a mold from nothing *(the concept's second founding act)*

*Added 2026-07-22. Boss: "the other side should cover creating a template from scratch, to meet
one's need, the simple way."*

§1 takes an impression from a cast that already exists. §1B is the other road: **the user knows the
shape they need and no note has been written yet.** Both are founding; the plan previously had only
one of them, which quietly made the Studio backward-looking.

**The governing requirement is SIMPLICITY**, stated by the Boss and inherited from the origin's
"tissue, or a scribe on a hand palm". Concretely that means:
- A handful of ordinary choices — name it, pick which of the four kinds it is, list the fields, list
  the sections. Nothing else is required to reach a usable mold.
- **No wizard and no rail** (standing ruling): the whole form is present and editable at once, not
  a sequence of gated steps.
- **Nothing must be written first.** An empty Universe can produce a mold on day one.
- It writes the same ordinary `.md` §1 writes — one object, two roads, no second format.

**BOSS RULING 2026-07-22 — §1B LIVES INSIDE THE STUDIO.** Making a mold from nothing is a
workshop act, and it belongs in the workshop.

*Reconciled with "the plain path is guarded hardest":* no conflict. That principle says a user must
be able to write a `.md` containing `{{date}}` by hand and use it as a template **without ever
opening the Studio** — it does not require the from-scratch BUILDER to live outside. The hand-written
path stays complete and unguarded; §1B is the assisted road for users who want one, inside the
workshop where the other molds already are.

**Still open for a Boss ruling before build:** whether §1B's field list offers the user's existing
property vocabulary as suggestions (their own words, per the Constellation Way — and the recognition
engine already knows every key in the Universe with its real spelling and how often it is used) or
starts blank.

> **Boss test:** from a Universe with nothing written, make a template called *Meeting* with two
> fields and two sections, in one screen, without being walked through steps. Then create a note
> from it and confirm the note is born with those fields and sections and a fresh identity.

### §2 — The request path — "I ask for a book, I get a book"

- **2a.** The **type-in-request**: an open vocabulary field (ruling: the six examples are handles,
  not an enum; *شرح*, *fatwa*, *maqāmah* are first-class). Resolution order = **your molds → curated
  defaults**. Your practice outranks the default (ruling #3). No questions asked (ruling #2).
- **2b.** **Curated Arabic defaults, authored from the Boss's own taxonomy** (his الأجناس الكتابية /
  الأوعية النصية message) — drafted, then **Boss-validated before shipping** (ruling #4's "do not
  author from general knowledge" applied to the one person who can validate Arabic structure).
  English defaults authored in parallel. Other languages ship **empty** rather than translated —
  honest scaffolding for future native authorship.
- **2c.** Defaults are plain `.md` files in a `defaults/` namespace — inspectable, copyable,
  editable; File Over App. Using one **copies it into your molds** so it becomes yours to tend.

> **Boss test:** type *كتاب* → receive the Arabic book skeleton (مقدمة/أبواب/فصول/خاتمة per your
> taxonomy). Type *Book* with your own Book mold present → receive **yours**. Type a type nobody
> ever heard of → honest empty-handed answer plus "start blank / save one first."

### §3 — The Studio surface — browse, tend, choose

One place listing the **three bands in descending authority** (Step 0 ruling): **Your molds** →
**Shapes noticed** *(empty until §4 — the band renders with an honest "nothing noticed yet")* →
**On offer** (the curated defaults). Rename / delete / open-as-file / set the template's declared
`shape`. Placement is a **Boss ruling** (§R2). i18n ×15, RTL native.

> **Boss test:** the full loop without touching the file system once — yet every artifact remains a
> plain `.md` you can open and hand-edit, and hand-edits appear in the Studio.

### §4 — Recognition — the smart library *(the differentiator, and deliberately after §1–§3)*

- Rust-side, **on demand** (open the Studio → compute; never at boot, never on the typing path —
  Rule 8 discipline; `note_meta` already carries the needed columns).
- Clusters notes by structural signature (property-key set + heading skeleton + tag family).
  **Few, strong, well-evidenced** proposals only; *"no strong recurring shape yet"* is a first-class
  answer (the concept's refusals). Every proposal carries its evidence: *"14 notes share these 5
  properties and this heading pattern — [see them]."*
- Naming a noticed shape **mints a mold** from the shared skeleton = the same §1 write path.

> **Boss test:** in your real Universe, the noticed band shows a handful of honest shapes with
> clickable evidence — or honestly none. Naming one produces a template identical in kind to §1's.

### §5 — Tending *(after everything above has lived for a while)*

The mold stays true to how you actually write: if your last N casts from a mold all add a field the
mold lacks, the Studio *holds* that observation and shows it **in the Studio only** — never
mid-writing (the Uninterrupted Stream ruling governs here too). Accepting updates the mold; the
observation is evidence-backed and dismissible. **Design point held open until §4's recognition is
proven in the Boss's Universe.**

### Explicitly OUT of this migration
- **Template visual styling** (the fourth style scope) — ruled "decouple; a declarative token set,
  never raw CSS." Its own migration, after the Studio exists.
- **Qusasah / anchored marginalia** — blocked on Studio per the concept weld, unblocked once §3 ships.
- **MIG-101 Phases B–F** — paused by this priority call, not cancelled.

---

## R — Boss rulings (updated 2026-07-21 — R2/R3/R4/R5 RULED; R1 in cross-check)

- **R1 — OPEN, Boss-directed cross-check in progress.** Boss: *"What are the standards? Is there a
  case study we can follow?"* — a WA#5 check against proven methods before locking the keep-vs-strip
  semantics. Research running (`wf_e5aee265-899`): template file-format standards (.dotx/.ott —
  what they keep, how identity restamps at new-from-template), shipped save-as-template flows
  (Evernote/Notion/Word/Canva/Figma…), and whether any product does structure-only. §1's default
  waits on this answer.
- **R2 — RULED: "Template Studio shall be treated as a core plugin. It will be like an app within
  the app."** The Studio is a self-contained surface: its own toggle, its own settings, its own
  views — shipped in-box but modular, on the Obsidian core-plugin model the app already parallels.
  §3 is re-scoped accordingly; the research includes a repo-grounded track on how Style Setter and
  the Wings flags mount today, so the Studio reuses those patterns rather than inventing a shell.
- **R3 — RULED: "The Arabic should be built based on the old and new manuscript."** The Arabic
  defaults derive from ATTESTED structures — the classical scholarly monograph (old) and modern
  Arabic academic/trade conventions (new) — never from general impressions. The standing exclusion
  of religious books as design evidence applies; the classical evidence base is the secular
  scholarly tradition (grammar, medicine, history, adab, philosophy). The Boss's own 2026-07-19
  taxonomy remains a named source. Draft skeletons come back **with per-element evidence grades**
  for his validation — he is the validator of record.
- **R4 — RULED: "The studio will have its own independent style setter."** This REVERSES the plan's
  earlier "styling is out of scope": template visual styling comes INTO this migration as a
  Studio-scoped style setter — its own instance, inside the Studio, not a category bolted onto the
  main Style Setter. Lands as **§6** after §3 exists to host it. The earlier concept rulings still
  govern its content: a declarative token set, never raw CSS; it styles the paper, never the desk.
- **R5 — RULED: "The studio will store its templates in a visual library."** The Studio's template
  list is a VISUAL gallery, not a filename list — §3 is re-scoped from "browse" to a card gallery
  with previews. The research decides the preview mechanism (stored thumbnail vs live render vs
  structure-skeleton preview); the concept leans structure-skeleton — showing the MOLD, not a cast.

### Phase list as amended by R2/R4/R5
§1 Save-as-Template (+ the frontmatter-trim defect fix) → §2 request path + Arabic/English defaults
→ **§3 the Studio as a core-plugin app-within-app, with the visual template gallery** → §4
recognition → §5 tending → **§6 the Studio's own style setter**.

## Discipline

Per-build: Boss tests **every** build before commit · diff-scoped safety inspection on write-path
steps (§1a/§1b/§2c) · `/simplify` per phase · session log + ledger reconciliation per SO#9 ·
Editor-Surface Gate not triggered (no editor-lifecycle changes) unless §1b's create-path fix expands.

**Ledger repair filed with this plan:** the Studio previously had **no PJ number** — the omission
the Boss caught on 2026-07-21. This migration IS the ledger entry now; PJ-130's remaining batches
(2–13) stay open and queued behind it, Batch 1 built-but-uncommitted awaiting its Boss test.

---

*Plan awaiting Boss approval. Approval = build approval; the cascade then runs §1a → §1b → the §1
Boss test.*
