# MIG-103 — How "Use a Template" Should Behave

**A concept paper for the Template Studio interaction model**
Audience: the project owner. Plain language. Every claim below is graded — `sourced` (found in a product's own documentation), `recalled` (general knowledge, not re-verified this pass), or a `Constellation-code` fact read directly from our repo. Where sources disagreed, the conflict is shown, not averaged.

---

## The horse before the carriage — the concept

A template is a **head start**. Its whole purpose is to save the user from re-typing the same scaffolding — a meeting note's structure, a book-note's fields, a daily log's headings. The cognitive job it does is: *"give me the shape I always use, so I can start filling in the thought, not the shape."*

That single purpose splits into two very different physical acts, and the confusion the owner is sensing comes from those two acts being blurred together. This paper's job is to un-blur them and then decide, for each of three situations the user can be in, exactly what should happen — and to make the destination of a new note something the app **proposes and shows**, never something it silently decides.

---

## 1. The core distinction — Snippet vs Scaffold

There are two, and only two, things "use a template" can mean, and mature software treats them as **two separate acts**:

- **SNIPPET (insert-at-cursor):** the template's *body text* is dropped into the note you're already writing, at the cursor. It mixes into existing content. Nothing new is created. (Obsidian core Templates, Templater's "insert" command, Word's Quick Parts, Logseq's `/template`, TextExpander — all `sourced`.)
- **SCAFFOLD (create-a-new-document):** the template *becomes a whole new note* — a fresh file with the template's structure and properties. (Word "New from template", Notion pages, Evernote, Google Docs, Craft, Anytype — all `sourced`.)

**Do shipped products keep these as two commands, or one clever command that adapts?** Two commands — universally. `sourced`

> **The count (Boss Q1):** Across every multi-paradigm product examined, snippet and scaffold are exposed as **two separate gestures**. **Zero** products were found that expose a *single* "use template" command which inspects whether the current note is empty vs full and silently switches between inserting and creating. The nearest cases — NotePlan and Coda — differentiate by the *template's own type/shape* (NotePlan has an "empty-note" template type; Coda single-page templates insert, multi-page templates are new-doc-only), **not** by interrogating the live note. `sourced`

**What this means for us:** building one "smart" button that guesses the paradigm from note-emptiness would be **inventing a pattern no mature tool uses** — exactly the presumptuous-rail failure The Constellation Way rejects. The proven pattern is: keep two clearly-named gestures, and let *which situation the user is in* decide **which gesture is offered or emphasised** — never hide a branch inside one gesture.

**Constellation already has both** (`Constellation-code`, Track D): an **Insert** gesture and a **New note from template** gesture, held in `templatePickerMode = 'insert' | 'newNote'`. So the division the field validates is already ours. The gaps are narrower than "redesign the feature": they are (a) the silent dead-end when Insert is invoked with no note open, and (b) the presumptuous destination guess on New-note. Everything below targets those two, plus the frontmatter safety rule.

---

## 2. The recommended model — the three states

The owner asked how it should behave across three situations. Here is the recommendation, each row grounded in the research **and** in what our code does today (the delta is what we'd change).

| Situation | What should happen | Evidence | What the code does **today** (the delta) |
|---|---|---|---|
| **(a) No note open** — user invokes a template | **Offer to create a new note from it** (route to the Scaffold path), rather than doing nothing. The empty state is where Scaffold naturally belongs. | NotePlan surfaces the *Insert Template* button **only in empty notes** and treats it as a create-here act; Notion/Evernote/Craft all make the empty/new state a create-from-template act. `sourced` | **Insert with no note open is a *silent no-op*** — `if (!tab) return;` fires before anything happens: no note, no message, no effect (`+layout.svelte:4912-4913`, `Constellation-code`). This is the empty-state dead-end the owner is probing. **Delta: convert the dead no-op into "create a new note from this template here."** |
| **(b) Open note *with content*** — user invokes Insert | **Insert the template *body* at the cursor** (additive mix). This is the whole point of a snippet gesture and is non-destructive of surrounding text. | Insert-at-cursor is the near-universal norm: Obsidian core ("inserted at your current cursor position"), Templater, NotePlan existing-note flow ("where your cursor is located"), TextExpander. **No product examined warns before mixing** — mixing at the cursor *is* the understood behaviour. `sourced` | **Already correct** — Insert dispatches the processed body into the CM6 editor at the selection (`+layout.svelte:4929-4966`), through the normal write path, never a raw disk write; it even refuses if the focused tab changed mid-prompt. **No delta for body behaviour.** The one real hazard is the *frontmatter* carried alongside — see §4. |
| **(c) New note** — user explicitly asks for a new note from a template | **Create it — and *propose* the destination, showing where and why, letting the user confirm or change it** (§3). | Word/Notion/Evernote: choosing a template creates a new document. Destination should be inherited-from-context or a neutral home, surfaced, not silently guessed (§3). `sourced` | **Creates correctly, but the destination is a *silent guess*:** focused note's folder, else `libraryStats[0]` — the *first* library — with no picker and no confirmation, only a title prompt (`+layout.svelte:4862-4873`, `Constellation-code`). **Delta: replace the silent guess with a proposed-and-shown destination.** |

### Boss Q3 — the crisp answer on mixing

**The question:** when a snippet lands in a note that already has content, is the safe rule *"fill only if empty, never inject into content"* or *"insert-at-cursor and trust the user"*?

**The answer depends on which of the two acts you mean — and this is the crux the owner should hold onto:**

- **For SNIPPET / Insert (mixing *body text* into a note you're editing): "insert-at-cursor and trust the user" is correct and safe.** `sourced` It is the near-universal norm, it is additive (it does not replace the note), and **no surveyed product warns before it** — a warning would be surprising, not protective, because mixing *is* the point of pressing Insert. Constellation already matches this exactly.
- **For SCAFFOLD applied to a *non-empty* target (turning a note that already has content *into* a template — whole-note apply): the safe rule is "fill only if empty."** `sourced` The dominant safety pattern across products is that whole-note templating only ever touches an empty target (Notion default templates, daily-note plugins apply *only at creation*). Where a product *does* apply over existing content (Notion's API), the **default is APPEND**, and whole-content **REPLACE is a separate, explicitly destructive, irreversible, opt-in flag** (`erase_content:true`, "a destructive operation that cannot be reversed"). `sourced` And the battle-tested UX when the target is non-empty is a **dirty-check** (GitLab issue #16188): if the user hasn't typed anything, apply plainly; the moment they have real content, **warn — apply-and-discard vs cancel**, never a silent whole-note overwrite. `sourced`

> **Bottom line:** Constellation should **not** build a "silently replace this note's content with a template" act at all. Keep Insert as trust-the-user body-mixing (no warning), keep New-note as create-fresh, and if we ever add "apply a template over a note that already has content," it must **default to append and gate any replace behind an explicit, confirmed choice** — never silent.

---

## 3. Destination selection — where the new note goes

**The dominant pattern, ranked** (what shipped products actually do for "which container does the new note land in"):

1. **The container you're currently in** — the active file's folder, the current database, the current notebook. Inherited automatically, never asked. Notion (a page from a database template lands in *that* database), Evernote (the notebook/space you're in), Obsidian's "Same folder as current file". `sourced` **This is the default the field converges on.**
2. **Degrade to the library/vault ROOT** when nothing is open. Obsidian's "Same folder as current file" falls back to the vault root when no file is open; Templater's "Create new note from template" defaults to the global new-note location (root unless the user changed it). `sourced` (Note: the *no-open-file → root* behaviour is documented via community reports for Obsidian, not its own help center — graded accordingly in the verification record.)
3. **A configured fixed folder** — attested, but **specifically for periodic/daily notes** (Obsidian Daily Notes has a "New file location"), *not* generalised to arbitrary templates. `sourced`
4. **A per-template destination binding** ("this template always saves to folder X") — this is a **requested-but-not-core** feature even in the most powerful tool (Templater issues #80/#857, both closed-as-workaround, never built natively). The attested mechanism runs the *other* direction: **folder→template** (the folder you create a note in decides which template fires), which is context-driven and fits The Constellation Way. `sourced`

**What NO product does:** fall back to "the *first* container." `sourced` None of the surveyed multi-container products documents any fallback beyond *current container, else root*. Constellation's `libraryStats[0]` — "the first library" — has **no precedent in any product found**, and its ordering isn't even guaranteed deterministic across boots (`could_not_establish`, Track D). This is precisely the presumptuous default the owner is right to question.

### Recommendation for Constellation's multi-library reality

Constellation's several **Libraries** map to Evernote notebooks / OneNote sections — so the multi-container question is real for us. The recommendation, in Constellation-Way terms (observe, propose, *show why*, user decides):

- **If launched from a folder or note** (a note is focused, or the user right-clicked a folder in the file tree): **propose that note's / that folder's library and folder as the destination, and *show it* on the create dialog** ("New note in **Library › Folder**"), with a one-click way to change it. Inheriting the active context is the proven pattern (rank 1). The launch context supplies the answer; don't ask redundantly, but **do show** what was inferred. The building blocks already exist — `libIdForPath` resolves any path to its owning library, and `libraryStats` enumerates every library — so a "propose-and-show, click to change" picker is buildable from existing frontend state with no new backend (`Constellation-code`, Track D).
- **If nothing is open (the ambiguous case): this is exactly where an explicit choice belongs.** Do **not** silently pick the first library. Either (a) fall back to a single, **named, predictable home** (the Universe's default `universe_notes` library / root — the "one named fallback" pattern Apple Notes and Obsidian use), shown on the dialog, or (b) surface a short library picker. Both are honest; "first library" is not.

> **Answer to the owner's destination question:** the new note should go to **the folder of the note/place you launched from, shown and confirmable** (active context, proposed not imposed); with nothing open, it should go to **one named default home (or a small picker) — never silently to "the first library."**

---

## 4. The frontmatter-collision hazard — what must NEVER happen

This is the sharpest concrete danger, and it is **guaranteed, not edge-case**, for Constellation because our notes *always* carry YAML frontmatter and our templates carry YAML too.

**What must never happen** (`sourced`, Templater issue #1387 — an explicitly *unsolved, open* problem in the leading tool): inserting a template that carries its own YAML into a note that already has YAML does **not** merge. The template's frontmatter gets dropped as **raw text at the cursor**, producing:

- **two frontmatter blocks / duplicate keys** (e.g. two `title:` keys, two `tags:` keys), and
- **broken template variables**, because the dumped YAML was never parsed as properties.

**The minimum safe behaviour for Constellation:**

1. **Never paste a template's frontmatter into a note's body as text.** Body and frontmatter are **two separate merge targets**, never one text blob.
2. **On Insert (snippet into existing note):** strip the template's YAML *before* injecting, and inject **body only**. If we ever want the template's *properties* to reach the note, **merge them into the note's existing frontmatter** — add new keys, **never clobber** the user's identity keys (`title`, `id`, `created`), **never duplicate**.
3. **On New note:** carry the template's frontmatter through the create path, but **filter the template-identity keys** (`kind: template`, the template's own title) and stamp fresh ones.
4. **Make the whole application ONE undoable transaction** (body + any frontmatter merge in a single editor/document transaction) so a mistaken apply is instantly reversible with Ctrl+Z — a real safety improvement over several surveyed tools, which offer *no* undo on their replace paths. `sourced` / `recalled`

**Where Constellation stands today** (`Constellation-code`, Track D — and this is good news):

- **Insert already strips the template's frontmatter before processing** (`extractTemplateBody`, `engine.ts:242-246`) and injects body only — so **there is no body-text YAML-dump hazard on insert today.** The trade-off: the template's properties are simply *discarded* on insert. A properties-merge capability does **not** exist yet (build it *only* as a careful merge, per rule 2 — never as a raw dump).
- **New note carries the template FM through `create_note` with identity-key filtering** — but that filtering is asserted by a **code comment**, not re-verified in Rust this pass (`could_not_establish`). **Before shipping, confirm the Rust `create_note` actually filters `kind`/`title` and stamps fresh identity** — a live test against a note-with-existing-YAML is required (this is a runtime check outside the research pass).

---

## 5. The owner's decision points — rule on these

Five either/or choices. Each has a recommendation. These become the R-questions for the §2 build.

**R1 — Two gestures or one adaptive button?**
☐ One "smart" button that guesses from note-emptiness ☐ **Keep two clearly-named gestures (Insert / New note), context decides which is offered** ✅
*Recommend: two gestures.* Zero mature products use one adaptive button; it's the presumptuous-rail pattern. `sourced` We already have two.

**R2 — What happens when Insert is invoked with no note open?**
☐ Keep the silent no-op ☐ **Offer to create a new note from the template here** ✅
*Recommend: offer to create.* A silent nothing is the empty-state dead-end; the empty state is where Scaffold belongs (NotePlan precedent). `sourced` + `Constellation-code`

**R3 — Where does a New-note-from-template land?**
☐ Silent guess (today's "focused folder, else first library") ☐ **Propose the active-context destination, show it, let the user confirm/change; with nothing open, one named home or a picker — never "first library"** ✅
*Recommend: propose-and-show.* "First library" has no precedent in any product found and isn't even deterministic. `sourced` + `Constellation-code`

**R4 — When Insert mixes into a note that already has content, do we warn?**
☐ Show a "you're about to mix into existing content" confirmation ☐ **No warning — insert at cursor and trust the user** ✅
*Recommend: no warning.* No surveyed product warns; mixing at the cursor *is* the understood point of Insert. (A warning belongs only to a *whole-note replace* act, which we should not build.) `sourced`

**R5 — How do a template's properties (YAML) reach a note?**
☐ Dump the template's YAML wherever (risks two-title-keys corruption) ☐ **Never dump YAML as text; on New-note carry+filter identity keys; on Insert, body-only today, and any future properties-reach is a careful MERGE (add keys, never clobber/duplicate), all in one undoable transaction** ✅
*Recommend: the merge-or-strip rule.* Raw-dump is the exact unsolved corruption in the leading tool; our notes always have YAML so the collision is guaranteed. `sourced`

---

## 6. Verification record — what was refuted or corrected

The research was adversarially verified. The material corrections that change how much weight a claim carries:

- **REFUTED — "Notion database templates apply only to empty pages (fill-only-if-empty)."** Notion's *own* developer docs contradict the mechanism: templates **can** apply to existing pages, and the **default is APPEND** ("the template's content is appended to any existing page content"), with `erase_content:true` as an opt-in destructive replace. The "must be empty / select-all-and-delete first" instruction came from **third-party** paid-template docs, not Notion. **Net effect on our design:** *strengthens* §2/§4 — the authoritative pattern for applying over existing content is **append-by-default, replace-only-as-explicit-destructive-flag**, which is exactly what we recommend. The "empty-only" framing was overstated; the append/replace distinction is the real, sourced lesson.

- **CONFIRMED — Notion's API default = append; `erase_content` = opt-in, irreversible, no built-in warning.** Verbatim from Notion's own developer guide. This is the load-bearing basis for §2's Boss-Q3 answer and §4's undo rule.

- **CONFIRMED — Daily/periodic-note templates apply *only at note creation*, never re-applied to an existing note** (Obsidian's own help + Actions-for-Obsidian docs, covering both core Daily Notes and the Periodic Notes plugin). Basis for §2(a)/(c) and §3 rank 2.

- **CONFIRMED — Obsidian core Templates is pure snippet (insert-at-cursor, never creates a note); Templater keeps Insert and Create-new-note as two distinct commands; Word runs New-from-template and Quick Parts as two gestures.** These three underpin the §1 "two commands, not one" finding and the count.

- **PARTLY CONFIRMED — "Same folder as current file is a commonly-chosen default, and no-open-file degrades to vault root."** Obsidian *documents* "Same folder as current file" as one of three options, but its **actual out-of-box default is "Vault folder" (root)**, so calling the same-folder option "the default" is unsupported. And the **no-open-file → root fallback is real per community reports but is NOT in Obsidian's own help center.** **Net effect:** §3's ranking stands (current-container, else root), but treat the empty-state→root as *community-attested, not first-party-documented*. It remains far better grounded than "first library," which has **no** precedent at all.

- **PARTLY CONFIRMED — "Templater new-from-template defaults to vault root; per-template folder is an open request."** Correct that per-template destination is **not core** (achievable only via scripting/QuickAdd). Two corrections: the feature requests (#80, #857) are **closed-as-workaround, not open**, and the default is **Obsidian's global new-note location** (root *only if the user hasn't changed it*), not a hardwired root. Design conclusion (root/global default is the safe assumption; per-template binding is not table stakes) is unchanged.

**Open items the research could not close (must be checked live before build):**
1. Whether the Rust `create_note` actually filters template-identity keys (asserted by a code comment only — Track D). **Live test required.**
2. Whether Constellation's own Insert / New-note paths, run against a note that already has YAML, merge or duplicate frontmatter — a runtime check outside the web-research pass.
3. Whether `libraryStats[0]` ("first library") ordering is deterministic across boots — it is used as a fallback but its sort was not traced. (Reinforces: don't rely on it.)

---

*Grades preserved throughout. Conflicts surfaced, not averaged. No claim invented — where a fact was not established, it is marked `could_not_establish` and flagged for a live check.*
