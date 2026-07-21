# MIG-103 — The Template Studio
## `/migration` Phase 2 — the Plan

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

## R — Boss rulings needed before the affected step (not before §1a/§1b)

- **R1 (before §1 ships):** "Save as template" keeps what? **Recommendation: keep property KEYS but
  blank their values, keep headings, drop body prose** — the mold is the structure, the cast is the
  content. Alternative: keep everything except identity (a true copy). Could also offer both as two
  menu items if neither feels right alone.
- **R2 (before §3):** where does the Studio live? **Recommendation: a full-page surface like the
  Style Setter** (it is a workshop, not a sidebar glance), reachable from the sidebar toolbar and
  the template picker's "manage" link.
- **R3 (during §2b):** the Arabic default set — I draft from your taxonomy; you validate before it
  ships.

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
