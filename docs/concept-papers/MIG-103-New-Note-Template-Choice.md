# Should "New Note" ask the user to pick a template?

**MIG-103 — Template Studio — research synthesis**
**Question asked (Boss, 2026-07-21):** *"Shouldn't when a user creates a New Note give them the choice whether to select from the Templates library, or create one from blank? What is the standard in this case?"*

**Method:** three parallel evidence tracks (office/document applications; note & PKM applications; friction research and middle-ground patterns), each finding graded **[sourced]** (read verbatim in the vendor's own documentation or a peer-reviewed paper this pass) or **[recalled]** (believed true, but resting on search summaries or memory — *not* proof). A separate adversarial verification pass then attacked the load-bearing claims; §6 records everything it corrected or refuted.

**Grade discipline:** nothing in this paper is asserted beyond its grade. Where the evidence is thin or contested, it says so.

---

## 1. The direct answer — what IS the standard?

**There is no single industry standard. There are two camps, and the dividing line is not "office vs notes" — it is *where the template lives relative to the moment of creation*.**

- **Document applications put the template chooser BEFORE creation** — but only on the slow, deliberate path, and every single one ships a way to turn it off.
- **Note & PKM applications put the template AFTER creation** — inside the already-open blank note, or applied silently by the note's context (folder, type, section).

### The count — document camp (8 products examined)

| Product | Fast path (keyboard / primary button) | Chooser mandatory on every new document? |
|---|---|---|
| Microsoft Word | Ctrl+N → new document immediately **[sourced]** | **No** |
| Microsoft Excel | Ctrl+N → blank workbook **[sourced]** | **No** |
| Microsoft PowerPoint | "Blank Presentation" is the first listed option **[sourced]** | **No** |
| LibreOffice (Writer/Calc/Impress) | Start Centre buttons "each open a new document of the specified type" **[sourced]** | **No** |
| Google Docs | "In the top left, under 'Start a new document,' click **Blank**" **[sourced]** | **No** |
| Apple Pages | Cmd-N **is** "open the template chooser" **[sourced]** | **Yes** |
| Apple Numbers | same, with a "Use template" off-switch **[sourced]** | **Yes** |
| Apple Keynote | same, "Use theme" off-switch **[recalled]** | **Yes** |

**Score: 5 blank-first, 3 chooser-first — and all 3 chooser-first products are from one vendor (Apple).**

Two facts sharpen this further:

1. **Every one of the 8 lets the user set a DEFAULT template** so new documents arrive pre-shaped with *no chooser at all* — Word's `Normal.dotm`, Excel's `Book.xltx`, LibreOffice's "Set as Default", Apple's "Use template" / "Use theme" **[sourced]**. This is the industry's real answer to "how do I get structure without being interrogated": **a default, not a picker.** Google Docs is the lone exception with no per-user default-template setting **[sourced]**.
2. **Apple — the only vendor with a mandatory chooser — is also the only vendor that shipped an explicit off-switch for it, and pre-selected "Blank" as that switch's initial value [sourced].** That is the strongest single piece of evidence in the whole corpus that a mandatory chooser is felt as friction by the people who built it.

### The count — note / PKM camp (15 products examined)

Obsidian, Notion, Evernote, Bear, Craft, Apple Notes, Google Keep, OneNote, Logseq, Anytype, Capacities, Tana, UpNote, Joplin, Drafts.

**Zero of the 15 put a template chooser in front of every new note by default.** **[sourced]** — composite of the per-product findings in §3.

The nearest approaches, and why none of them is a gate:

- **Notion** shows a *page-type* menu (table / board / list / "simply keep it as an empty page") on a new page — that is a question about the page's **shape**, not its content, and it sits *inside* the already-created page **[sourced]**.
- **Anytype** is type-first, yet Ctrl/Cmd+N "immediately create[s] a new object with the type that is set as your default object type"; the type menu is a **second, different shortcut** (Cmd/Ctrl+Opt/Alt+N) **[sourced]**.
- **Capacities** is type-first, yet ships the Page type ("It's all set up for you, all you need to do is start writing!") and the daily note as ungated scratch surfaces, with promotion to a type afterwards **[sourced]**.
- **Tana** nodes exist untagged; the supertag — and therefore its template — is applied by the user *after* the node exists, by typing `#tag` **[sourced]**.
- **Notion databases** put a chooser on a dropdown beside the New button — but only inside a database, a context that already declares the rows' shape **[sourced]**.

### Why the split exists — **[our inference, NOT sourced]**

**No vendor anywhere publishes a design rationale for fronting or not fronting a template chooser.** Microsoft, Apple, Google, LibreOffice, Obsidian, Notion — all of them document *what* happens; none documents *why*. This was checked in all three tracks and came back empty. Any "why" below is reasoning, and must be presented to the Boss as such.

The inference, plainly: a **document** is a formatted artefact whose shape is decided up front and which the author will live inside for hours — a résumé, a deck, an invoice. Paying a chooser once at the start is cheap relative to the artefact's life. A **note** is capture — frequent, fast, often seconds long, and its shape is frequently unknown at the instant the thought arrives. Charging a decision at that instant taxes the one moment that must be free.

I flag this honestly: it is the most intuitive explanation of the observed behaviours, and it is **not** a cited principle. The observed behaviours themselves are sourced; the explanation is ours.

---

## 2. Where a personal-knowledge app belongs

> ## ⛔ BOSS CORRECTION (2026-07-21) — the "camp" framing is REJECTED
>
> **What I wrote:** *"A document is deliberate, infrequent, long-lived… a note is fast and frequent…
> Constellation belongs in the note camp."*
>
> **Boss:** *"Wrong assumption! Constellation is a PKM/PKF system. It is based on the Cognitive
> Knowledge making/formulation, and it could use long documents or short notes, to formulate one's
> knowledge."*
>
> **He is right, and the error is categorical, not cosmetic.** I imported an outside taxonomy
> (note-app vs document-app) and then reasoned from that camp's habits, instead of reading
> Constellation's own concept. Constellation is not a note app that happens to hold long things:
> **its declared purpose is Knowledge Formulation, and formulation spans the whole range.** The
> evidence was in front of me — the four compositional forms (atomic · serial · structured ·
> collected), note **shape** (scrap → page), and the **manuscript builder** we are building as the
> fourth kind, which is long-document territory by definition.
>
> ### What survives, and what does not
>
> - **The product counts survive.** 15 of 15 note apps do not gate; 5 of 8 document apps are
>   blank-first on the fast path. That is observed behaviour and is unaffected.
> - **The INFERENCE does not survive.** "Constellation is in the note camp, 15–0" is withdrawn.
>   Constellation is in **both** — or rather, the camp question is the wrong question for this app.
>
> ### The corrected frame: design PER COGNITIVE ACT, not per app
>
> The app contains **both kinds of act**, so the default belongs to the *gesture*, not to the product:
>
> | The act | What it needs | Constellation's surface |
> |---|---|---|
> | **Capture** — get it down before it evaporates | blank, instant, never gated | **New Note** |
> | **Compose** — deliberately build something structured | structure offered up front | **New from template · the manuscript builder** |
> | **Formulate** — a scrap turns out to be something bigger | move between the two, late | **apply-a-template-later · shape graduation** |
>
> **So "New Note stays blank" is still right — but the reason changes**, and the new reason is
> stronger. Not *"because we are a note app"* (an imported claim, now false), but *"because that
> particular gesture IS the capture gesture."* A different gesture may legitimately front structure —
> which is exactly what the manuscript builder does, and why it is not a contradiction.
>
> ### The deeper consequence — this is why gating is wrong HERE
>
> The strongest argument against a chooser at creation is no longer borrowed from note apps. It comes
> from Constellation's own concept: **in knowledge formulation you frequently do not yet know what the
> thing is.** A scrap becomes a chapter; a daily entry becomes an essay. Forcing *"is this a quick note
> or a book?"* at the instant of capture demands a decision the user has not made yet — and the process
> of finding out **is the work**. That is the same insight as MIG-101's **graduation** (a note outgrows
> its shape) and as Shipman & Marshall's incremental formalization, arrived at from our own doctrine
> rather than from someone else's product habits.
>
> **It also elevates one recommendation:** *apply a template to a note you already started* (§4.3-C,
> Decision 4) is no longer a nice-to-have third item. In a formulation system it is **the bridge
> between the two acts** — the mechanism by which a captured scrap becomes a composed thing. It should
> be weighted accordingly.

**~~Constellation is in the note camp, and the evidence for that is not mixed — it is 15–0.~~**
*(Withdrawn — see the correction above. The 15–0 count stands as an observation about note apps; the
claim that it settles Constellation's identity does not.)*

### Reconciling with the project's own verified finding

Constellation's prior commonplace-book research established: **do not gate capture on a head** — historical compilers who had to decide a heading before writing abandoned the classification and wrote anyway. This synthesis independently arrives at the same place from three directions:

1. **The 15–0 product count.** No note app gates.
2. **The type-first products specifically.** The three products in the entire market that most strongly believe knowledge should be typed — Anytype, Capacities, Tana — each ship an explicit *un-typed* capture path and let classification happen afterwards **[sourced]**. Even the true believers refuse to gate.
3. **Peer-reviewed HCI.** Shipman & Marshall, *Formality Considered Harmful* (CSCW 1999) **[sourced, author-hosted full text read]**: *"There are many cognitive costs associated with adding formalized information to a computer system"*; *"the negative effects of prematurely or unnecessarily imposing a structure"*; *"users are often justified in rejecting formalisms."* Their named remedy is **incremental formalization** — *"defer formalization of that information until later in the task… information in such systems can be kept without structure until the user wants to add structure."* **A template is a formalism.** A chooser at creation is premature formalization by definition.

And the named failure mode when capture is over-structured: Bernstein, Van Kleek, Karger & schraefel, *Information scraps* (ACM TOIS 26(4), 2008) **[sourced — abstract only; full PDF would not parse]**: *"existing rich graphical application approaches interfere with user input in many ways, forcing complex interactions to enter simple information and requiring complex cognition to decide where data should be stored."* The cost of a creation-time chooser is not mild annoyance — it is **the note gets written somewhere else**: a text file, a phone, paper. For a system whose value compounds with capture volume, that is the actual risk.

### Where the evidence is honestly thin

**I found NO empirical study measuring whether a chooser step reduces how much people capture.** No A/B test, no field study, no capture-rate comparison — searched in all three tracks. The case against gating is **inferential**: Shipman & Marshall on cognitive overhead, Bernstein et al. on scraps, the observed 15–0 behaviour of the market, and Constellation's own commonplace-book precedent. Anyone stating "a chooser reduces capture by X%" would be fabricating. Present it as a design inference from converging evidence — which is strong — not as a measured finding, which it is not.

**Also thin:** the "defaults are powerful" argument. Johnson & Goldstein's organ-donation default effect is real in the lab (*"twice as many participants affirmed their willingness… in the opt-out condition even though a simple click of the mouse was the only effort required in all three conditions"* **[sourced via PMC3458339]**), but its real-world policy translation was **contested in 2024**: Dallacker et al., *Public Health* 236 (Nov 2024): *"Switching from an opt-in to an opt-out default did not result in an increase in donation rates"* across five countries **[sourced]**. Use the lab-strength form only — *within one UI where every option costs the same click, the default is what almost everyone gets* — which is exactly Constellation's situation. Do **not** cite the organ-donation story; it invites the exact "confident filler" failure this project is under standing orders to avoid.

### The one honest counter-argument

A totally bare empty note is itself a documented UX defect. Nielsen Norman Group, *Designing Empty States in Complex Applications* **[sourced]**: *"Do not default to totally empty states. This approach creates confusion for users, who may be left wondering if the system is still loading information or if errors have occurred"*; guideline 3: *"Provide direct pathways (i.e., links) to getting started with key tasks."*

So the Boss's instinct is **half right, and the right half matters.** The answer is not "keep it blank." It is **"keep it blank, and put the template door inside the note."**

---

## 3. The middle-ground patterns that are ACTUALLY SHIPPED

Ranked by strength of evidence, best-attested first. Products named. Shipped patterns are separated from plausible-but-unattested ideas at the end.

### Rank 1 — Scoped default template, applied silently *(strongest — 6 independent shipped instances)*

The note's **context** carries a template; it lands automatically at creation with **no dialog, no step, no question**.

| Product | Scope | Evidence |
|---|---|---|
| Obsidian core — Daily notes | per-surface | *"In Template file location, select the 'Daily template' note"*; *"Obsidian uses the template the next time you create a new daily note."* **[sourced]** |
| Obsidian core — Unique note creator | per-surface | *"In Template file location, enter the file you want to use as template."* **[sourced]** |
| Templater (Obsidian community plugin) | **per folder** | *"a template that will automatically be used on a selected folder and its children. The most specific (deepest) matching folder wins."* Also a regex-path mode, and an "Excluded folders" carve-out. **[sourced]** |
| Capacities | per object type | Star icon in *Object type settings → Define Templates*; *"It will be applied automatically going forward."* **[sourced]** |
| Anytype | per type / per list view | *"either the default template will be applied automatically, or you will be asked to choose a template yourself"* — depending on route; default is user-chosen. **[sourced]** |
| Joplin (plugin-templates) | global + per notebook | Per-notebook defaults *"take precedence over global defaults"*; fires from a dedicated command (Alt+Shift+N), **not** from the plain New Note button. **[sourced]** |
| OneNote | per section | "Always use a specific template" — **[recalled]**, not found on Microsoft's own pages |
| Notion | per database | "Set as default" — **[recalled]**, the word "default" does not appear on Notion's own database-templates help page |

**The proven contract, worth copying verbatim** (from Templater, the most mature implementation): **off by default** (`trigger_on_file_creation = false`, mode = `none` — gated twice); one template per context; **deepest/most-specific context wins**; applied silently; plus an **excluded-folders** escape hatch and an explicit documented hazard warning. Note also Templater's split of a **per-device** enable toggle against **vault-level** rules — a deliberate choice to copy or reject, not to inherit by accident.

**One constraint to adopt (from OneNote, [recalled]):** a template applies cleanly only to a **still-empty** note. Never apply a template over a note with body content without an explicit merge decision.

### Rank 2 — Affordance inside the already-created empty note *(3 shipped instances, 1 verbatim)*

The note is created instantly and *the blank note itself* offers the template. Nothing is blocked; typing dismisses it.

- **Capacities** — verbatim: *"If you already have an object created, you can simply choose your template from the template section at the bottom of your empty object."* **[sourced]**
- **UpNote** — *"When you create a new note, you can choose to write from the saved templates"*; the user taps **"select from Templates"** in the new-note prompt. Plus a mid-note `/temp` command. **[sourced]**
- **Notion** — *"Create a new page in your database and choose any of the templates from the gray menu it contains"*, and on a plain new page *"A menu of page types will appear… or simply keep it as an empty page."* **[sourced]**
- **Evernote** — "Start from a template" / "My Templates" inside a fresh note — **[recalled]**, help.evernote.com returned HTTP 403 to every fetch attempt in two separate tracks. **Do not cite Evernote as sourced precedent.**

**This is the pattern with the strongest design-literature backing** (it is precisely NN/g guideline 3 applied to a blank note) **and the lowest implementation risk** — the note already exists, the cursor is already live, the prompt costs nothing to ignore.

### Rank 3 — Separate command / gallery *(3 shipped instances — this is what Constellation has today)*

- **Craft** — *"Go to Settings → Templates. Browse or search for the template you want to use. Open the template to create a new document based on it."* Plus `/template` to insert into an open document. **[sourced]**
- **Notion** — Templates in the sidebar (Marketplace), separate from page creation. **[sourced]**
- **Obsidian core Templates plugin** — *"insert pre-defined snippets of text into your active note"*, at the cursor, via the `Templates: Insert template` command. **[sourced]**

Legitimate and mainstream. **Its weakness is discoverability, not correctness.**

### Rank 4 — Split button (primary click = create, caret = choose) *(exactly 1 shipped instance in the note camp)*

- **Notion database** — *"click the dropdown menu on the right of the blue New button at the top right of your database. Choose any template you've created."* **[sourced]** — and note it is **scoped to a database**, a context that already implies a shape.
- **Google Drive** — New → Google Docs → "From a template" submenu — **[recalled]**, search summaries only.
- **Craft** — a "+ → From Template" menu was claimed by third-party blogs; the official Craft help article does **not** document it. **[unconfirmed — do not cite]**
- **Obsidian Bases** — a template dropdown on the New button is **requested but not shipped** (forum feature requests + a community plugin filling the gap). **[sourced as a request]**

### Rank 5 — User setting: "blank vs chooser" *(attested only in the DOCUMENT camp)*

- **Pages / Numbers / Keynote** — Settings → General → "Use template" / "Use theme". **[sourced]**
- **Word / Excel / PowerPoint** — File → Options → General → Start up options → uncheck the show-start-screen box. **[sourced, per-app label]**
- **Google Docs** — Settings → "Display recent templates on home screens" on/off. **[sourced]**

**In the note camp, the equivalent is always SCOPED** — per type, per database, per section, per notebook, per folder. **A global blank-vs-chooser switch in a note app is not attested in anything either track read.** That is a real finding: if Constellation shipped one it would be inventing, not following.

### Rank 6 — Last-used template / recency ordering *(essentially unattested)*

**One** officially-attested instance in the entire corpus, and it is a document app: Google Docs' "Display recent templates on home screens" toggle — and even that is attested only *by the existence of the toggle's label*, never described directly by Google **[sourced-by-inference]**. **No note or PKM app documents last-used-template memory or recency ordering.** Recommending this would be recommending a plausible idea, not a shipped one.

### Plausible but NOT attested anywhere — do not present as prior art

- A global "always ask me / never ask me" preference in a note app.
- Last-used-template memory in a note app.
- Recency-ranked template pickers in a note app.
- Applying a template to an **existing** note with content already in it (Tana's `#tag` and Obsidian's insert-at-cursor are the closest, and neither reshapes an existing note). **Note: Shipman & Marshall's "incremental formalization" says this is exactly the right move — so it is a genuine gap in the market, and a defensible place for Constellation to be first, but it must be presented as our design move, not as an industry pattern.**

---

## 4. The recommendation for Constellation

**Concept first (the horse): "New Note" exists to get a thought out of the user's head and onto disk before it evaporates. Its one job is to cost nothing. Every design decision below serves that, and nothing else.**

### 4.1 What "New Note" should do

**Keep it blank and instant. Do not change it.** The keyboard path, the sidebar button, the command palette — all produce an empty note with a live cursor, immediately.

This is not conservatism. It is the position of Word's Ctrl+N, Excel's Ctrl+N, LibreOffice's Start Centre, Google Docs' Blank tile, Obsidian's Ctrl+N, Apple Notes, Google Keep, Drafts (*"Drafts opens to a new page with the keyboard ready so you can type immediately"* **[sourced]**), Anytype's Ctrl+N, and Capacities' Page type. **The 15–0 count in the note camp and the 5–3 count in the document camp both point the same way on the fast path.**

### 4.2 Is the existing "New note from template" command sufficient?

**It is correct, and it is not sufficient.**

Correct: it is Rank 3, the same shape shipped by Craft, Notion and Obsidian core. Constellation is **on** the standard here, not off it — this is worth stating plainly to the Boss, because the question was framed as though the current design might be an oversight. It is not; it is the incumbent norm in Constellation's own camp (files-on-disk Markdown PKM).

Not sufficient: a separate command solves *capability* but not *discoverability*. A user who has never opened the command palette does not know templates exist. That is the legitimate core of the Boss's instinct, and NN/g's empty-state guideline says a bare blank surface with no pathway is itself a defect.

### 4.3 The three additions, in build order

**(A) Put the template door inside the blank note — Rank 2. Build this first.**
When a new note is created and is still empty, show a quiet, ignorable affordance — *"Start from a template…"* — near the cursor or at the foot of the empty body. It disappears the instant the user types a character. Zero cost to capture, zero risk to typing latency, directly answers the discoverability gap, and is shipped verbatim by Capacities and UpNote.

This is the **highest-value, lowest-risk** change in this paper.

**(B) Per-folder / per-kind default template — Rank 1. Build second.**
Let a context carry a template that lands silently. Copy the proven contract wholesale: **off by default**, one template per context, **deepest context wins**, applied with no dialog, **only onto a still-empty note**, with an excluded-folders carve-out. Because Constellation's Template Studio is already building a note-**kind** taxonomy, bind each template to exactly one kind — Capacities' explicit rule (*"Templates belong to one object type only"*) is the attested precedent.

Note this would also be a genuine **differentiator against Obsidian**, whose community has asked for a default-template-on-new-note since **2020-12-22** and whose core has never shipped it **[sourced — forum thread read, 21 replies, no team commitment in-thread]**. Critically, that five-year community ask is for a **default**, not for a picker — independent confirmation of the direction.

**(C) Apply a template to an EXISTING note. Build third.**
This is Shipman & Marshall's incremental formalization made concrete: write first, shape later. **No surveyed product does this well.** Guard it with the OneNote constraint — over a note that already has body content, this requires an explicit merge decision, never a silent overwrite.

### 4.4 What NOT to build

- **A modal template gallery in front of every new note.** Zero of 15 note apps do it. The one vendor family that does it in documents shipped the off-switch.
- **A global "always ask me" preference.** Not attested in any note app; scoped defaults are the note-camp idiom.
- **Recency ordering / last-used-template memory** as a headline feature — one weak instance in the whole corpus.
- **A split button on the global New Note.** Its only shipped instance (Notion) is scoped to a database. If Constellation wants a split button, put it on a **Base / collection / typed view**, never on the universal New Note.

---

## 5. Owner decision points

**Decision 1 — Does "New Note" stay blank and instant?**
*Options:* (a) stays blank — capture is never interrupted; (b) opens a template chooser every time.
**Recommendation: (a).** 15 of 15 note apps, 5 of 8 document apps on the fast path, the project's own commonplace-book finding, and peer-reviewed HCI all agree. **Confidence: high.**

**Decision 2 — Does the blank note show a "Start from a template…" prompt that vanishes when you type?**
*Options:* (a) yes; (b) no, leave the note completely bare and keep templates on the separate command only.
**Recommendation: (a).** This is the Boss's instinct captured without its cost. Shipped verbatim by Capacities and UpNote; backed by NN/g's empty-state guideline. Cheapest change with the largest discoverability payoff. **Confidence: high.**

**Decision 3 — Do folders (or note kinds) get to carry their own default template that lands silently?**
*Options:* (a) yes, off by default, deepest-context-wins, opt-in per context; (b) no, templates only ever chosen explicitly.
**Recommendation: (a)** — but ship it **after** Decision 2, and ship it **off by default**. Six independent products attest it; Templater's contract is mature enough to copy line-for-line; and it fills a five-year gap Obsidian's core has left open. **Confidence: high on the pattern, medium on priority** — it is more design surface than Decision 2 for less immediate payoff.

**Decision 4 — Do we build "apply a template to a note I already started"?**
*Options:* (a) yes, as a Template Studio feature with an explicit merge step when the note has content; (b) no.
**Recommendation: (a), third in order.** It is the direct implementation of the strongest academic finding in this corpus, and it is a genuine market gap — but flag it honestly to the Boss as **our design move, not an industry pattern**, and gate it behind the still-empty / explicit-merge constraint. **Confidence: medium** — the principle is sourced, the product precedent is not.

---

## 6. Verification record — what was refuted or corrected

An adversarial pass attacked the load-bearing claims. Conflicts are surfaced, not averaged.

### REFUTED

**"Word's template gallery is an artefact of the app-LAUNCH screen; it does not appear on every new document."** — **REFUTED in its load-bearing half.** Microsoft's own *Create a document* article documents the in-app path: *File → New*, then *"Select Blank document, or double-click a template image…"* — a template gallery on the new-document action, inside a running Word, independent of launch. So the "gallery when you make a new document" impression is **not** merely a launch-screen artefact; Word ships **two** gallery surfaces. **The corrected framing, which does hold:** gallery on the **menu** path every time; **no** gallery on the **keyboard** path (Ctrl+N); recent files only on the launch Start screen. *This correction does not change any recommendation — Constellation's keyboard-and-button New Note maps to the fast path, which is blank in Word either way.*

**"Apple ships a keyboard bypass in both directions."** — **REFUTED.** Apple documents exactly **one** direction (once a fixed template is pinned, hold Option → *File → New from Template Chooser* restores the chooser). There is **no** documented reverse path. And the mechanism is a **modifier-revealed menu item**, not a keyboard shortcut — Apple's sentence names the File menu explicitly. Do not cite "bypass in both directions" as prior art.

### CORRECTED

**"Microsoft ships the same off-switch labelled 'Show the Start screen when this application starts'."** — **Substance holds; citation and label do not.** The article originally cited applies to **Office 2013 only**. The setting is confirmed current via three other Microsoft pages, but the **label is app-parameterised**: Word = *"Show the start screen when Word starts"*; the shared Office page = *"...when [Program] starts"*; the exact quoted string is currently attested **only for Excel**. Also: Microsoft's current Excel page **does** confirm the template-bearing Start screen is the **default**, which slightly strengthens the document-camp reading.

**"Google Docs shows the template gallery on the home screen."** — **Overstated.** What Google documents is a **Template Gallery control at the top right** of the home screen that the user clicks. Google never states what the home screen displays by default; the string "home screen" appears exactly once on the whole help page, inside the setting's label. Corrected wording: *"the home screen carries a Template Gallery entry point at top right."* Separately, Google's *File → New → From a template* is an **insert-into-the-current-document** flow, not a create-new-from-template flow.

**"Obsidian core DOES apply templates at creation."** — **Overstated headline; correct in detail.** Obsidian's own docs state the opposite default: *"By default, new unique notes are empty."* The Template-file-location field ships **blank**; template application is **opt-in configuration** in both surfaces. This actually **strengthens** the recommendation — the precedent is "mechanism present but blank and off-the-path until configured," which is exactly the shape recommended in §4.3(B). Also corrected: "core, not plugin" is not Obsidian's own vocabulary — core plugins are "officially built… and included within the application," and *"Some core plugins are disabled by default."* I could not establish from the docs whether Daily notes or Unique note creator is on out of the box, and I do not assert it.

**"Templater's folder templates and regex rules co-exist."** — **Corrected.** They are **mutually exclusive modes** of a single "Template matching mode" setting (None / Folder / File regex). There is no folder-vs-regex precedence rule because the two never run together. They also use **different conflict rules**: folder = deepest wins (implicit, structural); regex = first match top-to-bottom (explicit, user-ordered). "Deepest wins" is settled for the folder mode only. Two omitted details worth copying: an **Excluded folders** carve-out, and an explicit documented hazard warning.

### CONFIRMED VERBATIM (load-bearing, survived attack)

- Word Ctrl+N creates a new blank document; the gallery lives on the deliberate menu path. **(Windows desktop only — the Mac/web/mobile equivalents are NOT substantiated. Relevant under Constellation's cross-platform standing rule: do not assume the macOS keyboard path.)**
- Word's File → New lists "Blank document" as the first named action — first **in Microsoft's step list**; no vendor doc states tile *position*. (The position claim IS verbatim-sourced for Google Docs: *"In the top left… click Blank."*)
- Apple Pages: *"Every time you create a new document, you choose a template from the template chooser"*, plus the Settings → General → "Use template" off-switch. **(Mac guide only; not substantiated for iPadOS/iOS/iCloud.)**
- Obsidian's generic new note is blank; the core Templates plugin *"lets you insert pre-defined snippets of text into your active note"* at the cursor — a post-creation act, not a creation step. **Qualifier:** Obsidian's own CLI documents `obsidian create name="…" template=Travel`, so "core never templates at creation" would be false. The safe and still-decisive formulation: **no documented Obsidian creation path presents a template chooser** — capture is never gated, either by blankness or by silent pre-configuration.
- Templater's automatic folder templates are real, opt-in, gated twice, chooser-free, deepest-wins.

### COULD NOT ESTABLISH — stated so the Boss is not misled

1. **No vendor publishes a design rationale** for fronting or not fronting a chooser. The artefact-vs-capture explanation in §1 is **ours**.
2. **No empirical study** measures whether a chooser reduces capture. §2's argument is inferential.
3. **No documented user-friction evidence** for the type-first products (Anytype / Capacities / Tana) — searches returned nothing indexed. We can say the type step is not mandatory; we cannot say what users report about it.
4. **Notion's "Set as default template"** — not found in Notion's own help; the word "default" does not appear on their database-templates page. Third-party only.
5. **OneNote's per-section default** — not found on Microsoft's own template pages. Third-party and Microsoft Q&A only.
6. **Evernote's in-empty-note affordance** — help.evernote.com returned HTTP 403 to every fetch, in two independent tracks. Lean on Capacities and UpNote instead.
7. **Bear's absence of templates** — search summaries of Bear's own blog only; no first-party doc read.
8. **Logseq** — official docs not reached; deliberately excluded from the graded findings rather than presenting community text as documentation.
9. **Craft's "+ → From Template" split-button menu** — third-party blogs only; Craft's own help does not document it.
10. **Johnson & Goldstein's 42% / 82% figures** — not verifiable from a primary source this pass. Only the qualitative "twice as many" is sourced.
11. **Not examined:** Roam, Amplenote, Reflect, Heptabase; Word for the web / Word for Mac; Google Docs mobile; per-folder default templates in Word/Excel (their `Normal.dotm` / `Book.xltx` are global-per-user as far as was read).

---

*Prepared for MIG-103 (Template Studio). Three research tracks + one adversarial verification pass, 2026-07-21.*
