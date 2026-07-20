# Note Shape · The Post-it · Template Studio — brainstorm record

*2026-07-19 · Boss ↔ Claude brainstorm, captured mid-flight at the Boss's pace ("Don't rush
things out, we are still brainstorm the idea, the concept"). This is a RECORD of a live concept
conversation, not an approved design. Nothing here is scheduled or approved. Its companion is
`MIG-TPL-Templates-v2-Architect.md`, which covers the templates ENGINE and is awaiting the Boss's
ruling on options A/B/C.*

---

## How this started

The Boss reported "My Templates engine is not working." The audit found the engine largely intact
but the plumbing severed (the visible `templateFolder` setting is read by nothing; the real lookup
targets a hidden `.constellation/templates/` the app never reveals; no create-template flow exists
anywhere). That is documented in the Architect doc.

The Boss then rejected the framing of my response twice — correctly:

1. **"You started with Option B. Where are the other options?"** — I led with a recommendation
   instead of presenting the option space.
2. **"You forgot to develop the Concept, first."** — Concept-Before-Function is temporally first;
   I had written a concept paragraph but had not *developed* it before proposing mechanism.

Then he opened the real subject: **a template is also the SHAPE of a note.** *"Like, 3M Post-it,
can we consider it a note? If yes, what shape will it take?"* And: *"the note in Constellation is
the gate to one's knowledge, any note, shape, or type, even a tissue, or a scribe on a hand palm."*

---

## The concept as developed so far

**A template is the captured shape of a recurring cognitive move.** The question it answers:
*"What shape does this kind of thinking take?"* — asked once, answered permanently.

**The load-bearing distinction: a template is a MOLD; a note is a CAST.**

### What "shape" decomposes into

Chasing "what shape is a Post-it?" showed the yellow square is the least of it. Its cognition
lives in constraints:

| Dimension | What it carries |
|---|---|
| **Extent** | How much room the thought gets. A Post-it's size is a promise about scope. |
| **Ceremony** | How much structure must be filled before it's valid. A Post-it has none. |
| **Persistence** | Permanent vs expiring vs meant-to-be-processed. Post-its fall off *on purpose*. |
| **Attachment** | Standalone vs stuck to a host. Often forgotten; it is the Post-it's defining trait. |
| **Visual signal** | How the kind reads at a glance. *(Initially the weakest — see Template Studio.)* |

**A tissue** — worthless medium, disproportionate idea: *a knowledge system must never judge a
thought by the dignity of its container.* **A palm scribble** — nearest surface, urgent, will be
washed off: near-zero capture friction, near-zero persistence.

### The synthesis with the Boss's taxonomy message

The Boss separately laid out a five-dimension note taxonomy (function in the knowledge cycle ·
content kind · provenance · actionability · maturity), warning that most systems conflate them.
Reading dimension 1 through the Post-it: **function-role and shape are the same thing seen from
two sides.** A *fleeting* note IS a Post-it (small, ceremony-free, processed away). A *permanent*
note IS a codex page. A *MOC* IS a map. And **maturity is shape changing over time** — seed →
sapling → evergreen → canonical.

### What already exists in the app (verified, not assumed)

- `MATS = seed · sapling · evergreen · canonical` — the maturity dimension, **already shipped**.
- `CONF = hypothesis · evidence · established · contested` — an epistemic dimension the Boss's
  taxonomy doesn't name.
- `file_kinds.rs` — a **container taxonomy** already first-class: NOTE · BASE · TMPL · LINK ·
  MARK · CLIP + media. **MARK** = bookmark (url field + short body); **CLIP** = clipping
  (source field + blockquotes). These are *shape signatures detected from content*.
- **NotePane vs FocusPane** — the app already has two note FORMS, and FocusPane's charter (no
  parser, no highlighting, "capture ideas fast") is a Post-it sitting next to a codex page.
- `content_kind` / `function_role` / `actionability` — **do not exist**; genuinely new.
- Search is **SQLite FTS5**, not Tantivy — but `docs/IPC-CONTRACT.md` records a planned
  `full_text_search` "tantivy-backed FTS", so a facet design would not be stranded by that swap.

**The gap: Constellation already INFERS shape but never lets you DECLARE one.** A template is
precisely the act of declaring one.

---

## Boss rulings from this brainstorm

**R1 — Graduation ("Constellation should be Smart").** *"I could start with Post-It, form and
shape, but if I pass its constructed limitation, it should transfer itself to the next kind,
automatically."* Shape constraints are **thresholds, not walls** — this dissolves the
bind-vs-decorate dichotomy. Notes outgrow their shape and metamorphose.

Consequences identified:
- Each shape's limit should be **native to its nature**, not a universal word count: a Post-it
  overflows by *extent*; a bookmark when your commentary exceeds the link; a clipping when your
  words outnumber the quoted ones.
- **The classifier exists; the trigger does not.** `classify_file` knows the signatures but runs
  at canonicalization/import — never on content change. *(Correction of my own earlier claim that
  the machinery "half-exists" — the smaller half exists.)*
- **Must never be silent** (an app changing a note's identity invisibly is the scariest possible
  behaviour) and **must never touch the keystroke path** (Rule 1) — evaluate at the debounced
  save boundary.
- **Do not conflate with maturity.** Graduation is about *extent*; maturity is about *epistemic
  development*. A 2,000-word note can still be a seed. Fusing them loses "long but unripe."

**R2 — The Post-it is NOT a link; it is a second surface.** *"I wrote a note a long time ago, and
I have revisited it, but I don't want to edit it. I can use Post-It on top of the note, with my
remarks, to remind myself that I need to do something about it."*
- It is **marginalia**. The value is that the original stays untouched — the record isn't quietly
  revised by hindsight.
- Needs its own lane: visible on the note, **absent from backlinks, Sky, and every cognitive link
  surface** (precedent: PJ-065 structural links sit outside the cognitive vocabulary).
- **Unlocks annotating a note you cannot edit** — read-only notes, and notes in someone else's
  federated cUniverse. Today there is no way to have a thought *about* such a note without
  altering it or manufacturing a link.
- File-Over-App: its own `.md`, host named in frontmatter, so it survives outside Constellation.

**R3 — Anchor granularity.** *"Post-It should address whether the whole note, a paragraph,
sentence, context, idea, or a term, within a note."* Reduces to **one mechanism**: whole-note is
"no anchor"; paragraph/sentence/context/idea/term are all ranges of different size. Several
selection conveniences, one data model.

**R4 — Shape belongs to BOTH the note and the template.** Template seeds the initial shape; the
note carries the live one. Graduation rewrites the note's, never the template's — the same
relationship `created:` already has.

**R5 — Reference UX.** Microsoft Word anchored comments on an **Arabic (RTL)** document: anchor
highlight, marker icon, margin card with author + text + timestamp + reply box, card in the
**LEFT** margin (mirrored from English Word).

**R6 — Template Studio + a plain path.** *"I want to create a 'Template Studio' where users can
independently manage and control template styles… different styles in the same universe. Unique
style, fonts, size, page colors, etc. On the other hand, we can provide the users a simple plain
way to use a templates."*

**R7 — THE NAME: قصاصة (quṣāṣah)** — *"a clip, snip, or scrap."* Boss ruling 2026-07-19, taken
after the team review proposed *marginalia / الحاشية*. Plural **قصاصات**.

Why it beats both candidates: it is not a trademark (unlike *Post-it*), and unlike *ḥāshiya* — which
is bound to the **margin of a manuscript** — it names **the scrap itself**, which sits far closer to
the Boss's own founding image (*"even a tissue, or a scribe on a hand palm"*). A قصاصة is precisely
a tissue-tier medium: humble, cut off, provisional.

**Verified, no user-facing collision:** the file kinds (`NOTE`/`BASE`/`TMPL`/`LINK`/`MARK`/`CLIP`)
are **internal Rust classification codes with no i18n entries** — checked across `en.json`/`ar.json`
and the frontend; `CLIP` appears nowhere user-visible. So the name is free.

**But there is a CONCEPTUAL overlap to write down**, because `CLIP` already means "clipping" and
قصاصة *is* a clipping. The distinction is clean and opposite in direction:
- **CLIP** — something clipped **from** the world **into** your universe (external source; the
  existing heuristic is `source:` field + blockquotes).
- **قصاصة** — something **you wrote**, stuck **onto** a note (the source is you).

Consequences: the new kind needs its **own internal code, NOT `CLIP`**, and that contrast belongs in
a comment at `file_kinds.rs` where the next reader will look. Plural handling goes through the CLDR
`plurals.*` namespace (Arabic has five forms) — see the `plurals.walks` precedent.

**R7b — NAMING POLICY (Boss, 2026-07-19):** *"It will be Qusasah in the other languages, and in
Arabic قصاصة."*

So: **Arabic → قصاصة · all other locales → "Qusasah"** (transliterated). This is consistent with the
app's existing practice — verified: `app.name` is **"Constellation" in Latin script in all 15
locales**, Arabic/Japanese/Hebrew/Chinese included — but it is *sharper* than that precedent.
"Constellation" is a coined brand; **قصاصة is a real Arabic word being loaned out.** Hence Arabic
gets the genuine word and every other language gets the loanword. Linguistically correct, not merely
consistent.

**IT IS A NAME, NOT A TRANSLATION — Boss correction, same session.** *"No. The others gets to use
'Qusasah'. Not a translation."*

**Exactly two strings exist, total:** `قصاصة` for `ar`, and `Qusasah` for **all fourteen other
locales**. It is NOT re-transliterated per language (no クサーサ, no Кусаса, no קוסאסה) and NOT
translated. Persian, Urdu and Hebrew use `Qusasah` like everyone else.

*My error, recorded so it is not repeated:* I raised the Arabic-script question for fa/ur and an
LTR-island concern for the RTL locales, and proposed routing it to per-locale translator agents.
That was over-thinking a decision the Boss had already made completely — a NAME does not get a
localization pass. There is no per-language rendering to resolve, so there is no question to ask.
The i18n work here is two entries, not fifteen.

**Plural:** follows the name, not a per-language grammar — no `plurals.*` category map is needed
for it (contrast `plurals.walks`, which IS a counted common noun).

**Open — inherited from the guardian's objection:** is قصاصة the name of the **SHAPE** (the smallest,
ceremony-free cast, with the *concept* still needing a name such as التعليق) or of the **WHOLE
FEATURE**? Both readings are defensible. If the whole feature: the name "expiring" on graduation is
not a bug but an accurate description — the scrap genuinely *became a page*, and its life as a
قصاصة ended, which is exactly what the team's graduation ruling already models (live anchor freezes
to provenance + a `derives-from` link is minted).

---

## THE HARD PROBLEM (open, under team review)

**Anchors must survive editing of the host, while the host is NEVER modified.** That rules out
both common answers — Obsidian `^block-ids` and CriticMarkup **write markers into the annotated
file**. Anchoring must live entirely outside the `.md` and still re-find its span after the text
moves, is reworded, or is deleted. This is a studied problem (W3C Web Annotation Data Model,
Hypothes.is) — WA#5 says take the field's answer rather than invent one. Team review in flight
(`wf_635b637f-ca9`): four research tracks + engineer, auditor, Art Director, concept guardian.

---

## Template Studio — what it settles, and its discipline

**It legitimises the visual-signal dimension.** Alone, "the Post-it is yellow" is decoration and
fails Form-Aligns-To-Purpose. As a *kind* signal it is an instrument: the look tells you what kind
of thinking is inside before you read a word — the reason a Post-it, a legal pad and a leather
journal don't look alike on a real desk.

**It makes graduation self-announcing.** R1 required graduation to be visible but not intrusive.
With per-template style, no notification is needed: the yellow scrap *becomes* a page under your
hands. The promotion is **perceived rather than announced** — better than a toast.

**The discipline that prevents a scrapbook:** *style belongs to the mold, and every cast inherits
it.* All Post-its look alike — that is what makes them recognisable as Post-its. The moment an
individual note can be styled arbitrarily, the look stops meaning anything. Style is a property
of the **kind**, not the instance.

**It lands in an existing cascade.** Constellation already styles at three scopes: app-global
Styles → per-Universe `styleOverride` → per-Library appearances. Template Studio is a **fourth**.
Precedence must be deliberate — initial instinct: templates win for the note's *surface* (most
specific), chrome stays with the Universe.

**Open: where does a template's style live?** In the template's own frontmatter, the mold is
self-describing and portable — share the template and its look travels, File-Over-App all the way
down. In settings, it is centralised but the template alone is incomplete. Leaning strongly to
frontmatter.

**Reuse, don't rebuild.** The Style Setter engine is mature (categories, elements, live preview,
CSS-variable application). Template Studio should be a new **scope** within it, not a parallel
styling system ("secure the winning — one source of truth").

**The plain path is guarded hardest.** Writing a `.md` with `{{date}}` and using it must remain a
complete, respectable way to use templates — **never opening the Studio**. Styling is an upgrade,
not a gate; otherwise ceremony becomes compulsory, contradicting the tissue and the palm.

---

## Open questions (not for me to decide)

1. **What is the ladder?** Graduation implies an order. Post-it → ? → Note. Is there a rung
   between? Do MARK and CLIP sit *on* that ladder, or are they parallel shapes with their own
   paths that all terminate in a full Note? — decides whether shape is one progression or a family.
2. **Does an anchored Post-it graduate?** If it grows into a full Note, does it keep its anchor
   and host? Is an "anchored Note" still marginalia — or has it become something that should be a
   link after all? *(Flagged to the concept guardian as the concept's weakest seam.)*
3. **Is a Post-it findable?** It is not a link and is excluded from cognitive surfaces — but
   "find all notes I flagged for review" is a real need. Outside the graph, inside search?
4. **Anchor to a place or the whole note** — R3 says both; the mechanism must serve the range case
   without ceremony for the whole-note case.
5. **The Reviewer boundary.** Reviewer = system-initiated resurfacing; Post-it = user-authored
   marginalia. Where exactly is the hand-off? (Boss ruling: complement, zero duplication.)
6. **Which taxonomy dimensions become real**, and in what order? (`content_kind` first is my
   instinct — the one users mean by "type", the one templates most naturally carry.)

---

## Process notes for whoever picks this up

- The Boss is **brainstorming**; do not convert this into a plan or code until he says so.
- Two framing corrections already earned: **present the option space, don't lead with a
  recommendation**; and **develop the concept before proposing mechanism**.
- The identity-clean ruling from the templates thread applies here too: **a template never
  contains `cid_cn` or creation date/time** — and note that opening any `.md` currently *injects*
  `cid_cn` (`canonical.rs:1224` via `store.ts:2168/2468`), so the templates folder needs an
  exemption or editing a mold would stamp it.

---

# TEAM REVIEW — engineer · auditor · Art Director · concept guardian (`wf_635b637f-ca9`)

*Boss-requested 2026-07-19. Four research tracks (W3C/Hypothes.is anchoring · editor-native
position maintenance · sidecar storage · anchored-comment UX), four role reviews, one synthesis.*

**Verdict: SOUND WITH CHANGES — unanimous.** No reviewer said "sound"; none said "needs-rethink";
they converged on the same reasons independently, which the synthesizer called the strongest signal
in the packet. *Process note: my script truncated the synthesizer's input, so it received 3 of 4
reviews; the concept guardian's was recovered from the journal afterwards — it had reached the same
headline conclusion independently.*

## 1. BLOCKER — this is three concepts under one name

Guardian and synthesizer reached this separately:

- **(A) Marginalia** — a remark anchored over a note the user will not edit.
- **(B) Mold/Cast + graduation** — a general law over ALL notes, not a Post-it feature. A daily log
  that outgrows its form should graduate too. Building it "inside Post-it" puts a universal law
  inside one feature.
- **(C) Template Studio** — per-template visual style; a fourth style scope.

As briefed they are welded: **anchored marginalia cannot ship until Template Studio is designed.**
Recommendation: three migrations, (A) first, consuming only a minimal template hook. (A) is complete
without the other two.

## 2. The name — "Post-it" names the shape, not the concept

Lesser objection: **a 3M trademark** — a real distribution consideration. Larger objection: it names
*a yellow square of paper*, an instance of a SHAPE, and says nothing about the cognitive act. And
because shape graduates, **the feature routinely stops being a Post-it while remaining the same
thing** — a name that expires at first use.

The act is ancient and already named in the Boss's own tradition: **الحاشية (hashiya) / التعليق
(taliq)** — scholarly commentary anchored to a passage. Recommendation: name the CONCEPT
*marginalia / al-hashiya*; keep **"Post-it" as the name of its default SHAPE**. Then "my Post-it
graduated into a page" stays true instead of self-contradictory, and the x15 i18n pass has a real
word to translate rather than a brand to transliterate.

## 3. BLOCKER — the graduation seam breaks, in a specific place

If a graduated Post-it keeps a **live anchor**, we have invented an *"anchored Note"*: a graph node
with a coordinate, which no other node has — and it quietly becomes the link the Boss ruled it must
not be. Marginalia is *defined by subordination*; a Note is the opposite.

**Resolution:** *only a Post-it may hold a live anchor.* At graduation the anchor **freezes into
provenance** (source id + the quote as it stood + timestamp — W3C's `State`/`cached` idea, whose
omission the research identifies as the root cause of unrecoverable orphans) and a typed link is
minted, defaulting to `derives-from`. Reversible by archive.

Further: **SHAPE graduates automatically and visibly** (the Boss's self-announcing insight holds);
**KIND is a one-click, undoable proposal.** "Constellation is still smart; it just does not rename
your file while you type."

## 4. The granularity ruling and orphaning are the SAME mechanism

The most elegant finding. The six granularities are not a picker — they are a **ladder of decreasing
precision**: term → sentence → paragraph → whole note. Orphaning is what happens when precision is
lost. An anchor that can no longer find its sentence **has not failed — it has fallen one rung**, to
its paragraph, then to the note. The bottom rung ("a remark about this note") is a perfectly good
Post-it.

Research backing: **~22% of Hypothes.is annotations orphan; 88% of those are unrecoverable**
(arXiv:1512.06195). Degradation converts the field's ugliest failure into graceful decay.

Requirement: the resolution outcome must be a **total, asserted function** over
{exact, degraded-to-N, host-lost} — **never inferred** from the absence of a highlight, which is the
shape of Hypothes.is's "limbo" bug (product-backlog#954) and of the app-killer class the Safety
Charter exists to prevent.

## 5. Anchoring — we are better positioned than the prior art

- **We own CodeMirror 6.** Within a live session anchors map **exactly** through every edit
  (`tr.changes.mapPos`, already used in `calloutPlugin.ts` / `paragraphDir.ts`). Hypothes.is
  fuzzy-matches *because it cannot observe the document.* The hard part is only the **COLD seam** —
  note reopened, external edit, sync, merge.
- **The quote is a CHECKSUM, not a fallback.** A cheap selector (stored offset) is accepted only if
  the text there still equals the stored quote — making "silently drifted onto neighbouring text"
  impossible on the fast path.
- **Do NOT copy Hypothes.is's tuning.** Verified from source: `maxErrors = quote.length / 2` (50%
  edit distance), a minimum-score threshold the docstring claims but the code does not implement,
  and an unconditional top-candidate return — deliberate recall-over-precision for the public web.
  **"An orphan is honest; a mis-anchor lies."**
- **Arabic**: `offset_unit` and `norm_form` are **schema, not convention** — mixing Rust bytes /
  Rust chars / JS UTF-16 across the boundary is a silent corruption source in exactly this corpus.

## 6. Not a link is not the same as not in the system — PJ-065 settles it

The Boss's ruling holds (weight/confidence/traversal are meaningless on a remark; forcing it in
would debase the vocabulary). But the repo already contains the pattern for *"a lane outside the
cognitive vocabulary that is still fully real"*: the structural lane is excluded from cognitive
surfaces **by explicit predicate**, not by omission. Adopt that shape — own file kind, **body
indexed in FTS5 and fully findable**, anchor never indexed, excluded from Sky edges / link weight /
traversal / cognitive backlinks by a written not-in clause.

## 7. Word's card is a COLLABORATION UI; Constellation is a SOLITUDE UI

The RTL *layout* is right and worth copying (`linkTip.ts` already provides the mirroring +
measure-then-place machinery — a reuse, not a build). The *contents* are not: the **author avatar**
answers "who said this?", a question with one possible answer in a single-user PKM; the **name**
likewise; the **reply thread** answers "how do we converge?", which a solitary reader does not need.
Specify **six elements** — highlighted span · marker · RTL-correct margin card · the remark ·
created-time · one action — **and forbid the rest by name**, so "just add replies" must argue
against a written ruling.

## 8. Template Studio — sound, but decouple, and style cannot be raw CSS

Style-as-signal is judged well-reasoned, and *"style belongs to the mold, every cast inherits it"* is
endorsed as exactly right. Two cautions: **coupling makes a small shippable concept hostage to a
large unresolved one** — ship the Post-it's default look as a built-in shape, let the Studio adopt it
later; and in a **federated** Universe a template arrives from someone else, so arbitrary CSS
travelling with it can phone home via `url()`, overlay app chrome, or render text at zero contrast.
**A template's style must be a declarative TOKEN SET** from the existing registry, validated for
contrast and clamped at apply time, identically for local and foreign templates: *"it can style the
PAPER; it can never style the DESK or the INSTRUMENTS."* Frontmatter over settings. The plain path
stays an invariant.

## 9. The fourth thing hiding in the Boss's own sentence

*"…to remind myself that I need to do something about it"* is an **OBLIGATION**, and obligations
already have two owners (Tasks, Review Pulse). **Reviewer is DERIVED and system-initiated; Post-it
is ASSERTED and user-authored** — opposite in origin. The hand-off is one-directional and narrow:
**Post-it owns the remark and its location; the Reviewer owns all scheduling, ranking and
resurfacing**, reading *one bit* — open vs resolved. **One attention inbox, and it belongs to the
Reviewer.** "Show me everything I flagged" is a Reviewer lens fed by the Post-it signal, not a
Post-it panel. If a remark also carries an obligation, the user writes a task line and the existing
Tasks surface picks it up unchanged.

## 10. Remaining blockers before design proceeds

Split into three migrations · rule the graduation seam as a **schema fact** · close the pre-existing
**silent-reclassification path** (`file_kinds.rs` infers kind from content TODAY — Post-it
frontmatter must carry an explicit `kind:` honoured at priority 1; worth fixing even if Post-it is
never built) · decide **host identity = `cid_cn`** (strong rec) vs name · **one host-content
accessor, no fallback** (model if open, else disk — never both; never re-anchor a DIRTY host) · rule
**federation scope for v1** (if in, the Rust parse-hardening lands in the SAME migration) · draw the
**Reviewer boundary** · a **performance acceptance gate** measured on the 7,600-note Universe with a
synthetic 5,000-Post-it load *before* commit.

---

# RULINGS — Boss: "You should know me by now. Choose what will work with Constellation philosophy."

*2026-07-19. The Boss delegated the remaining open questions. Decided by me against the app's stated
principles, each with its grounding named so a future session can re-open it on the merits.*

**D1 — Qusasah is ONE name, for the whole feature.** The guardian's "the name expires on graduation"
objection is REJECTED: Constellation already names stages meant to be outgrown (`seed → sapling →
evergreen`). A seed that grows is no longer a seed — that is the point, not a defect. A Qusasah that
graduates genuinely stops being one; the name retiring IS the transformation. A second abstract name
for "the act" would be the unnecessary abstraction the Don't-list forbids. *(Grounding: Constraint as
Design; no unnecessary abstractions.)*

**D2 — There is NO ladder; there are ON-RAMPS.** No intermediate rung between Qusasah and NOTE (a
scrap too long to be a scrap is just a short note — inventing a rung fills a degree of freedom the
geometry affords but the cognition does not require). MARK and CLIP are NOT rungs but PARALLEL molds,
each with its own native overflow — Qusasah by *extent*, MARK when commentary outgrows the link, CLIP
when your words outnumber the quoted ones — **all terminating in NOTE**, the gate to knowledge.
Graduation is convergent, not a chain. *(Grounding: Form-Aligns-To-Purpose.)*

**D3 — SPLIT INTO THREE; Qusasah ships FIRST.** Qusasah · the mold/cast graduation law · Template
Studio. The decisive reason is not scheduling: **the graduation law governs every note, so it cannot
live inside one feature** — that is a universal law smuggled into a carriage. Qusasah is complete on
its own and needs only a minimal template hook. *(Grounding: Concept-Before-Function; the Migration
Rule.)*

**D4 — Templates engine: OPTION B, with A as its first phase.** A alone leaves the mold English-only
in a 15-language app — a standing Language-First violation, not a rough edge. C fails the CONCEPT
before the security review: a mold that executes code is no longer inert shape, and under federation
it is someone else's code in your Universe. *(Grounding: Language-First; Constraint as Design;
federation safety.)*

**D5 — The held-open rulings.**
- **Host identity = `cid_cn`**, never path. The app already has an identity field; a name would
  re-derive it worse.
- **Graduation: SHAPE changes automatically and visibly; KIND is a one-click undoable proposal.**
  Smart, but it does not rename your file while you type.
- **The card is SIX elements** — highlighted span · marker · RTL-correct margin card · the remark ·
  created-time · one action. No avatar, no author name, no replies: they answer questions a solitary
  reader does not have. Forbid the rest by name.
- **Template style is a declarative TOKEN SET, never raw CSS.** It styles the paper, never the desk.
- **Reviewer owns ALL scheduling; Qusasah owns the remark and its location**, exposing exactly one
  bit (open | resolved). One attention inbox, and it is the Reviewer's.
- **Federation is IN v1**, with its Rust parse-hardening in the SAME migration. It is the distinctive
  value, and Templater's history proves a trust boundary cannot be bolted on afterwards.

**D6 — ORDER OF WORK.** The app-killers-first ruling predates the Boss reporting the templates
breakage. Weighed honestly: the two remaining app-killers are RACES — real, but rare and requiring an
instrumentation build first; the templates breakage is CERTAIN AND CONSTANT (the picker is empty
every time). Therefore:

1. **"Templates You Can See"** (migration §1 — days)
2. **The two app-killer races** — instrumentation BEFORE fixes, per Reproduce-First
3. **Qusasah**
4. **Mold/cast law + Template Studio**
5. **PJ-126** (the content-bearing tooltip sweep)

*Any of these is reversible on the Boss's word; they are recorded as decisions, not as facts.*

---

# ★ THE CONSTELLATION WAY — Boss-dictated 2026-07-19, confirmed verbatim

> *"Old-fashioned apps are one way to interact with users. The users don't have a choice but to
> follow what the app wants them to do. So, if the app is STUPID, then imagine the outcomes.
> The Constellation way, on the other hand, is a two-way interaction between the user and the app.
> It shouldn't assume that it understands the user's needs and push them toward an undesired
> outcome. Instead, it should start the 'Smart' interaction from step 0 and walk them through
> (interact) to meet their needs. All that means that the Template Engine (Studio) has to be
> smart."* — Eisa
>
> Boss confirmation of the formulation below: **"Exactly, well said. This is the Constellation way."**

**THE LAW:** *The app observes, proposes, and adapts. The user decides. It never assumes, and never
railroads. **Visible reasoning is what keeps "smart" from becoming "presumptuous."***

## Why it matters

A one-way app traps the user inside whatever the app assumed. If the assumption is wrong — or the
app is stupid — there is no exit. Two-way means the app begins at **step 0**, before it knows
anything, and learns the user's actual need through interaction rather than declaring it.

## What it invalidates (recorded so it is not re-proposed)

**A wizard is NOT automatically the Constellation way.** A wizard is a RAIL: the app decided the
steps, their order, and what matters, before meeting the user. Prettier than a text file; the same
one-way street. *(Canonical violation, same session: after the Boss rejected the comment-stuffed
`.md` scaffold as "old fashion," I proposed a "Template Wizard" — and he corrected that it was
still one-way. Two mechanism-proposals in a row, each smuggling an assumption.)*

## Smart ≠ guessing

| Presumptuous | Smart |
|---|---|
| "Here's a Book Note template" — from a generic list of what apps think people write | "43 of your notes open with a Source, a Claim and a Verdict. Shall I make that a shape?" |
| The app decides what you need | The app proposes from **evidence in your own Universe**, and **shows the evidence** |
| Refusing costs effort | Trivially refusable — a proposal you cannot easily reject is a rail |

**The qualifier is the Boss's own:** *"It shouldn't assume that it understands the user's needs."*
So "smart" cannot mean the app deciding confidently on the user's behalf — that is merely a stupid
app with better manners.

## "From step 0" — what it rules out

Step 0 is before the app knows anything about what the user wants. **A form cannot be step 0,
because a form IS the app's assumption rendered as UI.** The opening move must cost the user
nothing and teach the app something — e.g. *"more like this"* pointed at a note the user already
wrote, with the app showing what it thinks the shape is (what it kept, what it stripped as
note-specific, what it noticed is always filled in) and the user correcting it. **The mold derived
from casts the user already made** — the opposite of a blank form plus a syntax card.

It does not stop at creation: if the user keeps adding a field the template lacks, the app should
notice and offer it. **The mold keeps learning from the casts** — the same observe→propose→decide
loop, running for the life of the template.

## The app ALREADY holds the evidence (verified, not assumed)

`note_meta` (`search.rs:2949`) carries, for **every note in the Universe**: `properties_json`,
`tags_json`, `headings_json`, `outgoing_links_json`, `word_count`, `body_text`, folder, and dates.
**Constellation already knows the shape of everything the user has ever written** — it has simply
never used that to help them build a mold. Smart here requires no guessing and no AI: it requires
reading what is already indexed.

## Scope — this is a GENERAL law, not a templates rule

It governs **graduation** (notice a note outgrowing its shape → propose visibly, never force),
**Qusasah** (the Reviewer surfaces unresolved remarks; it never nags), and every future "smart"
surface. It pairs with **Concept-Before-Function** (state the horse first) and with MIG-084's
self-explanatory law (*"just by looking, the user UNDERSTANDS"*) — visible reasoning is the same
demand applied to the app's own proposals.

**Likely belongs in `CLAUDE.md` as a standing top-principal** alongside Concept-Before-Function and
Form-Aligns-To-Purpose — flagged for the Boss's ruling rather than added unilaterally.

---

# THE SMART TEMPLATE STUDIO — concept (2026-07-19, Boss-directed "Start with the concept")

*Developed under [[THE CONSTELLATION WAY]] above. Concept only — no mechanism, no UI. Nothing here
is approved or scheduled.*

## The concept

**The Template Studio exists to recognise the shapes you are already writing in, and let you name
them.**

## The inversion

Every template system researched assumes: *you know what template you want and lack a place to
write it* — so it gives you a file and a syntax card (Obsidian, Templater). **The assumption is
wrong**, and the user's own Universe disproves it: he has been writing in shapes for years. Notes
about sources look alike; daily reviews look alike. Those ARE templates — unnamed, undeclared,
rebuilt by hand every time. The mold was never missing; it was never *carved*.

So the Studio's work is not invention. It is **taking an impression from the casts already made.**
This is also why it can be smart WITHOUT guessing: it does not predict what the user might want, it
reports what he demonstrably already does — from `note_meta`, which already holds properties, tags,
headings, links and length for every note (`search.rs:2949`).

## Two movements

**Recognition** — surfacing the shapes latent in the practice, with evidence attached.
**Tending** — a mold is not finished when named. If the user keeps adding a field the mold lacks,
the app notices and offers it. The mold stays true to how he *actually* writes, not to how he wrote
the day he named it. Most systems have only creation; tending is what makes a mold a relationship
rather than a snapshot.

## ★ BOSS RULING — a stated need is not an invitation to interrogate

> *"If I ask for a book template, then Constellation should provide one, not something else. On the
> other hand, the 'smart' templates library should contain what the universe has."*

My proposed design had the request path routing THROUGH recognition ("your 12 book notes split into
two shapes — which did you mean?"). **Rejected, and rightly**: that is the app being clever instead
of useful — presumption wearing a helpful face, making the user do work he did not ask for in answer
to a question he already answered.

**The refinement to the law: TWO-WAY DOES NOT MEAN INTERROGATIVE.** Observing and proposing belongs
where the user has NOT stated a need. The moment he states one, meeting it plainly IS the respectful
move.

### Two surfaces, two jobs

| | Job | Behaviour |
|---|---|---|
| **The request path** | You name a type → you receive that type | Curated defaults. No questions, no cleverness. |
| **The smart library** | What your Universe actually has | Recognition lives here — shapes you already write in, with evidence. Nobody asked it a question, so it is free to observe and offer. |

Natural consequence: asking for *Book* when you already have your own Book mold gives you **yours**.
Your practice outranks the default.

## ★ BOSS RULING — cultural contrast is STRUCTURAL, not verbal

> *"Since Constellation is a multinational app, it should consider the cultural contrast. For
> example, the structure of an Arabic book differs from that of Japanese books."*

I had been treating localization as *wording*. The Boss's point is that the **structure** differs:

- An **Arabic** book carries a substantial **مقدمة** doing methodological work a Western preface does
  not, moves through **أبواب** and **فصول**, closes with a **خاتمة** — and in the scholarly tradition
  may carry **إسناد** / **تخريج** apparatus with no Western counterpart at all.
- **Japanese** composition has **起承転結**, whose **転** is a *turn* rather than a conflict resolved —
  structurally not the thesis→argument→conclusion spine.
- A note *about* a book inherits this, because what is worth recording depends on how the thing is
  built and what its tradition treats as significant.

**Therefore: a translated English book template is the wrong template in the right language.** The
default templates are **per-language artifacts, authored natively — NOT translated.** This is a
deeper localization than Constellation has attempted: today it localizes strings; this localizes
*shape*.

**Two constraints on doing it:**
1. **Do not author these from general knowledge.** Sketching Arabic or Japanese book structure from
   a general impression is exactly the confident filler the BASIC RULE forbids. Arabic can start
   from the Boss's own genres/containers taxonomy (his 2026-07-19 message: الأجناس الكتابية / الأوعية
   النصية); every other tradition needs native authorship + real research — the i18n-agent method,
   but for STRUCTURE rather than wording.
2. **Culture ≠ locale.** A Japanese speaker writing Western-academic papers should get the Western
   structure. The culturally-native default is a DEFAULT, never a constraint; the offered list stays
   open and cross-tradition picking must be trivial.

**Staging, honestly:** Arabic first (it is the Boss's Universe and his taxonomy already maps the
territory); other languages get natively authored defaults as they are written — rather than
shipping fifteen translations of an English shape and calling it multinational.

## Step 0 — three bands, in descending authority

**Your molds** (already named) → **Shapes noticed** (unnamed, with evidence — where the smart lives)
→ **Types on offer** (the vocabulary, for when neither has what you want).

The order matters: the user's own practice outranks anything the app suggests.

**The offered list must be OPEN** — the Boss's examples (daily, summary, extraction, project, book,
essay) are a starting vocabulary, not an enum. Typing your own type (*شرح*, *fatwa*, *maqāmah*) is
first-class and queries the Universe identically.

**The offered types are HUMAN HANDLES.** The Boss's six deliberately span different taxonomy
dimensions — *daily* is an activity, *extraction* a function, *project* actionability, *book*
provenance, *essay* a genre. That is correct: nobody thinks *"a literature-note with project
actionability"*; they think *"a book note."* Each handle quietly sets several dimensions underneath;
the structured taxonomy stays internal, visible on request, never a form to fill.

## What the concept refuses

**Many weak proposals** — 47 patterns listed is noise wearing the costume of intelligence. Few,
strong, well-evidenced; and *"I haven't seen a strong recurring shape yet"* is an honest, valuable
answer. Silence beats a list of guesses.
**Mystery** — every proposal carries reachable evidence. Proposing without showing why is assuming.
**Capture** — the `.md` stays the artifact; the Studio is a lens onto files the user owns. Hand-
editing becomes the escape hatch, not the entrance.
**Cost** — pattern-finding never runs at boot, never touches typing (Rule 8; derived at write time
or on demand in Rust).

## One emergent consequence

If shapes are *recognised from the corpus*, **the taxonomy emerges from practice instead of being
imposed.** `content_kind` stops being an enum someone else chose and becomes a description of what
this user actually does — which is precisely what the Boss's taxonomy message warned flat enums
cannot do.

## Still open

- **How assertive is recognition** — does the library open with what it found, or wait to be asked?
- **Does tending ever act alone**, or always propose and wait? (Leaning: always wait — but graduation
  ruled the other way, so it deserves a deliberate answer.)
- **Is a mold allowed to be wrong** — if a named shape goes unused for a year, does the Studio say so?

---

# COMPOSITIONAL FORMS — "if a book, a journal and a note are types of something, what?"

*Boss question, 2026-07-19. Recorded at his instruction.*

## The answer

**Not medium** — a book can be papyrus, paper or a file. **Not physical container** either (in the
Boss's sense): a book can live in a scroll, a codex or a database; a journal can be a bound notebook
or a folder of files. Both survive being moved between vessels. **Not function** — a book may be
scholarly, literary, religious or technical.

What distinguishes them is **how units of writing are combined into a whole**:

> **Book, journal and note are COMPOSITIONAL FORMS. The axis is MODE OF COMPOSITION.**

In the Boss's own vocabulary they are **أوعية نصية / textual containers** — but *abstract* ones,
independent of both the medium and the physical vessel.

## The four modes, and the list

| Mode | Definition | Members |
|---|---|---|
| **Atomic** | one unit, whole in itself | note · **قصاصة** · card · fragment · entry · memo · reminder · definition · quote · observation · question · aphorism · **حاشية** (gloss) |
| **Serial** | accretive, time-ordered, open-ended, never "finished" | journal · diary · log · daybook · **دفتر** · ledger · chronicle · **تاريخ** · annals · minutes · notebook · waste book |
| **Structured** | designed architecture, closed, carries an arc | book · **كتاب** · treatise · **رسالة** · essay · **مقالة** · article · thesis · monograph · manual · textbook · commentary · **شرح** · report · study |
| **Collected** | gathered rather than composed; the arrangement IS the work | anthology · commonplace book · dossier · **ملف** · archive · **أرشيف** · encyclopedia · **موسوعة** · dictionary · **معجم** · **ديوان** · catalogue · bibliography |

**Unresolved:** the **letter** — atomic in extent but defined by having a RECIPIENT. That suggests
**audience is a separate dimension crossing all four modes**, not a fifth mode. The Boss's genres
table already treats correspondence as its own family, so he may have resolved this already.

## Two clarifications this produces

**1. The Boss's size scale conflates PARTS and WHOLES.** *sign → word → phrase → note → paragraph →
letter → document → book → archive.* But a paragraph is a **part** — nobody makes a paragraph as a
thing. A note is a **whole** — small, but complete. Sign/word/phrase/paragraph are units of
LANGUAGE; note/letter/book/archive are compositional FORMS. Two different scales laid end to end.
It matters because Constellation's atom is a *whole*, not a part.

**2. Composition is independent of medium and container** — the third axis the taxonomy message was
arguing for. What a thing is MADE of · what VESSEL holds it · how its units are COMBINED.

## All four modes already exist in Constellation — unnamed, and unrelated to each other

| Mode | Where it already lives |
|---|---|
| Atomic | the **note** — the app's declared atom |
| Serial | **daily notes**, implemented as its own feature |
| Structured | the **PJ-065 structural / parent–TOC link lane** |
| Collected | **Collections** (MIG-092) |

Four features, built at different times for different reasons, which turn out to be the four ways
writing has always been composed. **Same shape as the Studio insight: the thing was already there,
unnamed.**

---

# ★ THE FOUNDATIONS QUESTION — "Why are we stuck with the 'Note' as the gate to knowledge?"

> Boss, 2026-07-19: *"But you discover one important thing: The app's foundations. Constellation
> calls everything a 'note'! Why are we stuck with the 'Note' as the gate to knowledge? It is not
> complying with the Constellation Way. Isn't it?"*

**Partly yes — and the precision matters, because it decides whether this is a rewrite or a
vocabulary fix.**

### Where the violation is NOT

**The `.md` file is a NEUTRAL vessel.** It can hold a two-word scrap or an entire book; nothing in
the architecture constrains extent, structure or purpose. File-Over-App gives us a vessel with no
opinion. The atom is not the problem.

### Where the violation IS

**"Note" is a non-neutral NAME on a neutral vessel.** The word pre-decides what the user is making,
before the app has met them — which is precisely the one-way move the Constellation Way rejects.
And the exact fault is a conflation: **one word doing two jobs — the name of the ATOM and the name
of the WHOLE CATEGORY.** That is the same conflation the Boss's taxonomy message warned against,
sitting in the app's most basic vocabulary.

**What it actually forecloses:** Constellation can already HOLD all four compositional forms; it can
only NAME one. Serial exists as a mechanism (daily notes) but there is no *journal* the user can
point at and say "this is my 2026 journal" and have the app understand. The series is implicit; the
structure is implicit; only the atom is named. **In a knowledge system naming is not cosmetic — it
is how the user understands what they have.**

**Is it a Constellation Way violation?** It is an *inherited* one — a decision made before the app
met any user, and taken from the field (Obsidian: notes; Roam: blocks; Notion: pages) rather than
chosen. An inherited assumption is still an assumption.

### The proportionate response — NOT a rename

**Renaming the atom is the wrong fix** and would be enormous churn (every table, command, i18n key,
doc, and every user's mental model) to correct a naming fault the shape work already addresses.

The right move is narrower: **stop letting "note" be the name of everything.** Keep it as the atom's
name; let the compositional forms become first-class beside it, so the app can name what the user is
actually making.

**The Boss already began this himself.** *Qusasah* is the first thing in Constellation that is
deliberately NOT a note — he named it instinctively rather than calling it "an annotation note."
That is the precedent, and he set it.

**Caution (Constraint as Design):** this must not become "forty note types." Four principled
compositional modes, each earning its place — not a proliferation.

### What the Template Studio must therefore know

- A template declares its cast's **compositional mode**, not merely its content shape. "Daily note"
  is *serial*; "book note" is *atomic*; "book" is *structured*; "reading list" is *collected*.
- The Studio's offered types should be **organised by mode**, because mode is what determines how
  the thing behaves over time — whether it accretes, closes, or gathers.
- Mode interacts with **graduation**: outgrowing a shape may mean changing *extent* (scrap → page),
  but it may also mean changing *mode* (a note that keeps accreting dated entries has become a
  journal). Those are different transitions and should not be conflated.

---

# MEDIA & CONTAINERS ACROSS TRADITIONS — research findings (`wf_c054b0e2-6cf`)

*Boss-directed 2026-07-19, Arabic first. Four tradition studies (Arabic/Islamic at high effort,
East Asian, European, South Asian & Hebrew) + synthesis. Claims below are GRADED by the researchers:
"attested" = in classical sources or the codicological handbooks; "lexical only" = a real word, not
verified as a technical term; "UNKNOWN" where unconfirmed.*

## ⛔ SCOPE RULING — religious books are EXCLUDED (Boss, 2026-07-19)

> **"Exclude all religious books."**

Sacred, scriptural and liturgical texts are **out of scope as design evidence**: the Qur'an /
مُصحَف, the Torah scroll, the Talmud page, the glossed Bible (*glossa ordinaria*), masorah,
Buddhist sutra rolls, prayer and devotional books. **Two independent reasons, and both are good:**

1. **They are the wrong model.** A sacred text's layout is optimised for recitation, veneration,
   ritual use and transmission-fidelity. It answers questions Constellation never asks. Constellation
   is a tool for a person working out their *own* thinking — a purpose no canonical text shares.
2. **They had already produced one factual error** (the paragraph claim, below). Excluding the class
   removes the failure mode structurally instead of patching each instance. *That is the same move as
   Solve-the-Class-Not-the-Instance, applied to research rather than to code.*

**What this removes, named honestly** — so a future reader knows the survey is *deliberately*
secular, not accidentally thin:
- The **Talmud page** and the **biblical *glossa ordinaria***, which were the two exemplars of the
  **fixed-frame** layered page. The v2 synthesis was mid-run when the ruling landed and has been
  redirected to secular fixed-frame evidence instead — chiefly **Accursius's *Glossa Ordinaria* on
  the Corpus Juris Civilis** (the *legal* glossa: same designed geometry, no scripture), plus
  humanist commentary editions, variorum editions, and the modern apparatus criticus.
- The Qur'anic mise-en-page markers that caused the paragraph error.

**What this does NOT remove** — most of the layered-page evidence was never religious. The Arabic
scholarly apparatus (حاشية · تعليقة · لَحَق · عَطْفة · ضَرْب · بلغ) is attested overwhelmingly on
**grammar, logic, medicine, astronomy, mathematics, lexicography, adab and poetry** — Ibn Sīnā's
قانون carries glosses exactly the way a canonical text does. Same for East Asian 頭注 / 訓点 /
振り仮名 on secular classics, European scholia on classical authors, and commonplace-book marginalia.
**The tradition of the layered page is a scholarly tradition first; the scriptural instances are a
subset, not the source.**

**Standing rule where evidence is thin:** if a tradition's layered-page practice is attested *only*
on sacred texts, the finding is **"unknown for secular texts in this tradition"** — a stated gap.
Never substitute a religious example under a secular label.

## ★ THE HEADLINE — this justifies the whole shape concept

> **Every historical container got its shape from physics FOR FREE. Digital affords nothing and
> denies nothing — so Constellation must AUTHOR the constraints that stone, wax, papyrus and paper
> used to impose. That is not a metaphor for templates; it is the literal justification for them.**

A لِخاف (thin white stone) denies extent and grants permanence because of what it *is*. A washed
لَوح denies persistence because it wipes. A bound مُجَلَّد demands ceremony because binding is
irreversible and expensive. A bamboo 簡 rations extent to one column. **None of these had to be
designed.**

In Constellation a file is a file. Therefore **a note shape is a SYNTHETIC CONTAINER** — a
deliberate re-imposition of affordances the substrate no longer supplies.

**And the proof that shape is first-class rather than cosmetic is that languages give the container
its own name, distinct from the work it holds.** A *symphony* is the work; a *score* is the object —
you perform the first and shelve, annotate, inherit and fingerprint the second. Latin split the same
pair as *liber* (the work) and *volumen* (the physical roll). **Shape changes what the thing IS**,
and the vocabulary proves people always knew it. *(This point originally rested on the مُصحَف/قرآن
pair; replaced with secular equivalents under the exclusion ruling above — the argument is unchanged
because it was never a religious observation, only a naming one.)*

## The five-dimension hypothesis: HOLDS, but incomplete — and one is MIS-TYPED

**Holds cleanly** (falls out of physics without being forced):
- **Extent** ← the unit of surface. عَسِيب/لِخاف hold one short passage because that is all they are.
  A bamboo 簡 holds exactly one vertical column — *which is why the column became the citation unit
  of Chinese texts and outlived bamboo by two millennia.* صَحيفة = one sitting · جُزء = one session ·
  مُجَلَّد = a life's work.
- **Persistence** ← erasability, the cleanest mapping of all. The school لَوح is *expiring by
  construction*. The مُسوَّدة ("blackened" draft) is *meant-to-be-processed* — it exists to be
  superseded by the مُبيَّضة ("whitened" fair copy). Inscription and cast bronze are permanent
  because the medium refuses revision. **Every tradition that possessed an erasable surface used it
  for drafting, without exception.**
- **Attachment** ← whether the thing has its own text block. مَتْن stands alone; حاشية / تعليقة /
  لَحَق are meaningless without a host. Likewise 頭注, 訓点.
- **Signal** ← rubrication, script size, column doubling, framing. East Asian is the extreme:
  half-size double columns 雙行小字 mean "gloss", full size means "text", above the frame line means
  "head-note". Arabic: red vs black, overlined lemma, قوله / أقول as speaker markers in red.
  **In both, the KIND is read before a single word is.**

**MIS-TYPED — CEREMONY is not a container affordance.** It is a proxy for **irreversibility and
cost**. A commissioned bound volume demands a formal opening, a formal hand, a colophon and
completeness not because binding *affords* ceremony but because binding is expensive and irrevocable
— the ceremony is the psychological tariff on a commitment you cannot undo. *The tariff scales with
the bill, not with the subject matter: a bound medical or legal codex carries the same formality as
any other, which is precisely why the exclusion ruling costs this argument nothing.* A لَوح demands
nothing because wiping costs nothing. **In digital, cost and irreversibility are both ~zero, so ceremony has NO physical source.
If Constellation imposes ceremony without a real downstream commitment behind it, it is
manufacturing friction and calling it design.** Ceremony must be *earned* by the shape promising
something irreversible or expensive — or dropped.

**MISSING 6th — ADDRESSABILITY (the strongest finding).** The East Asian 巻 / 冊 split: 巻 divides
the *work*, 冊 divides the *object*, and they float free — the Yongle Encyclopedia is **22,937 巻
bound as 11,095 冊**. The logical address survived a total change of container (roll → stitched
book). Arabic has the same layer: folio recto/verso, **lemma-citation** (a شرح addresses its base
text by quoting its opening words after قوله — an address that survives *any* recopying because it
is made of the text itself, not of a position), and the **عَطْفة** reference
mark tying a marginal لَحَق to its exact gap in the line. *A container's fitness is partly: "can a
stranger point at a piece of this, later, and still hit it after the object is reorganised?"* A
scroll denies addressability; a codex grants it. **A قصاصة is impossible without it.**

> **★ A convergence worth pausing on, surfaced by the secular re-framing.** Lemma-citation anchors a
> gloss by *quoting the words it is about*. That is, exactly, the W3C Web Annotation Model's
> **TextQuoteSelector** — and the reason Hypothes.is anchors on quoted text with fuzzy matching
> rather than on character offsets is the same reason a شرح did: **a position-based address dies the
> moment the container re-flows, and a quote-based one does not.** Twelfth-century commentators and
> the 2017 W3C spec solved the same problem the same way, eight centuries apart, because it is the
> *only* shape of address that survives re-layout. → This is strong independent confirmation for the
> anchoring design already sketched at §"Anchor granularity": **store the quote, not the offset**;
> keep offsets only as a fast hint to be re-validated, never as the identity.

Not covered
by any of the five.

**MISSING 7th — PROVENANCE-CARRYING.**

## ★ QUSASAH — what the layered page teaches

1. **The governing law: annotations go OUTSIDE the text block, never inside it.** The مَتْن (lit.
   "the firm ground, the back") is inviolable; everything a later reader says goes around it. **This
   was not politeness — it is the mechanism by which the WORK stayed stable while the COPY
   accumulated history.** A reader five centuries later tells base from gloss *at a glance*, because
   the difference is spatial and absolute. → *A قصاصة never enters the host's body, under any
   circumstance, including "just this once for convenience."*

2. **The base stays authoritative through FIVE structural mechanisms — not through locking.** Nobody
   forbade writing in the matn; the architecture made it obviously wrong. (a) **Spatial
   segregation** — outside the ruled block, boundary often drawn literally in gold (جدول).
   (b) **Graphic subordination** — glosses smaller, faster, less formal; the eye ranks them before
   the mind reads them. (c) **Explicit voice marking** — قوله ("his statement:") / أقول ("I say:")
   in RED at every switch of speaker. (d) **Self-attribution** — a gloss signs itself, terminating
   in صح ("verified") or the glossator's mark. (e) **Additive-only editing** — errors struck by
   ضَرْب and **remain legible**; omissions appended as a لَحَق, never interpolated; disagreement
   written alongside, never substituted.
   → **Don't make the host read-only; make the قصاصة obviously not the host.**

3. **Anchor with an explicit mark; RENDER with spatial adjacency.** Arabic solved anchoring with the
   **عَطْفة** — a curved stroke at the gap AND at the head of the marginal supplement: explicit,
   portable, durable. East Asian used **no anchor syntax at all** — a 頭注 is bound to its column by
   pure vertical alignment; *position is the link*. Digital reflow kills the second. **So: store the
   Arabic way (a durable anchor surviving re-layout), present the East Asian way (the remark appears
   beside its passage — no footnote number, no superscript, no marker cluttering the host).** The
   user should never see the plumbing.

4. **Attachment scope is THREE-way, not binary** — both traditions independently distinguish:
   **interlinear** (one word — a synonym, a vowelling, an ʿajamī translation word; 振り仮名 furigana)
   · **marginal** (a passage — outer/lower margin in Arabic, upper band in East Asian) ·
   **title-page/flyleaf** (the whole object — ownership تملّك, endowment وقف, prices).
   *Interlinear = "this word means that word." Marginal = "about this passage." Title-page = "about
   this thing as an object."* → ship word-scope, passage-scope and note-scope as visually distinct
   classes. **A single flat "comment" would be a regression against 12th-century practice.**

5. **A remark's KIND matters and the kinds are not interchangeable** — gloss · objection · variant
   reading · correction · **collation mark (بلغ — "the comparison reached this point", a progress
   marker addressed to a future self)** · **audition record (سماع)** · ownership · endowment.

## Cautions — what to REFUSE (cargo-cult failure modes, named)

- **Refuse the scroll.** Its defining affordance is a DENIAL: no random access, no two-place
  reading, no margin, no stable page address. It survives digitally only as the accidental default
  of a scrolling viewport — *which is why we mistake it for neutral.* Importing "scroll mode" as a
  deliberate shape imports a defect and calls it heritage.
- **Refuse aged-paper texture, parchment backgrounds, torn-edge قصاصة graphics, faux-stitched
  袋綴じ borders, rotated "pinned" sticky notes.** They signal *oldness*, not *kind*. Colour, weight,
  size and zone carry the distinction (that is what rubrication and 雙行小字 actually did); a paper
  texture carries nothing and costs contrast across 15 languages.
- **Refuse ceremony with nothing behind it.** If a shape demands ceremony, name the irreversible
  thing it commits to; if you cannot name it, cut the ceremony.
- **Refuse the literal fixed-frame glossed page as UI** — the Accursian legal folio, the 注疏 block,
  any designed geometry of nested zones. That frame was a *print* solution for a **small, fixed,
  canonical set of layers known before the page was laid out**. قصاصات are unbounded in number,
  arbitrary in length, and unknown at layout time — the precondition the frame depends on is exactly
  the one we cannot supply. **Take the ARCHITECTURE (zones own layers, base untouched, size ranks
  the layer); refuse the PICTURE.** *(Originally written against the Talmud page; re-grounded on the
  secular legal glossa under the exclusion ruling. The caution is unchanged — which is itself
  evidence the point was structural, not scriptural.)*
- **★ Refuse claiming قصاصة as a heritage term.** It is genuine Arabic ("a cut-off scrap", from قصّ)
  but **NOT attested as a technical term of classical codicology** — the attested terms are
  **تعليقة · حاشية · لَحَق · رُقْعَة**. Ship it as a deliberate, well-motivated coinage in the
  spirit of the tradition. *Presenting it in 15 languages as "the historical Islamic term for
  marginalia" would be exactly the fabrication class the BASIC RULE forbids.*
- **Refuse importing dead containers because their names are beautiful** (دِيوان، سِجِلّ، بَياض،
  龍鱗裝 — the last not even reliably reconstructed). A container earns a slot only if its profile
  differs from every shape already shipped. **Naming is the last step, not the motivation.**
- **Refuse a narrow decorative margin.** The margin worked because it was GENEROUS. A 40px gutter
  with 8pt text is a margin in name only — قصاصات become unwritable and users will put remarks in
  the host body instead, *defeating the entire feature*. If the viewport cannot afford real margin,
  use a different primitive (overlay, facing pane, reveal) — not a token strip.

## Open questions the research could NOT close

- ~~**Two studies were truncated before reaching the synthesizer**~~ **→ SUPERSEDED TWICE.** The
  truncation was mine (a character slice in the workflow script) and cost the synthesis its two
  **fixed-frame** exemplars — the model that Arabic and East Asian **flowing-margin** practice does
  not cover. The v2 re-run was launched to recover them; **the exclusion ruling then removed both,
  because both were religious books.** The open question therefore survives its original evidence:

  > **Fixed-frame vs flowing-margin remains genuinely unanalysed, and it is the one architectural
  > question the قصاصة design cannot be frozen without.** v2 is answering it from **secular**
  > fixed-frame evidence — chiefly Accursius's *Glossa Ordinaria* on the Corpus Juris Civilis, plus
  > variorum editions and the apparatus criticus.

  The question to answer, stated precisely: *what does a fixed frame SOLVE that a flowing margin
  does not, and vice versa — and what did each tradition do when annotation **OVERFLOWED** its
  allotted zone?* That last clause matters most: a physical page was **forced** to answer it, and a
  digital one usually dodges it. Our قصاصات are unbounded, so overflow is not an edge case for us —
  it is the normal condition.
- **طُرَّة** — flagged unverified; real word, attested in West African ʿajamī scholarship, technical
  scope in the Eastern tradition UNKNOWN. Needs Gacek's *Glossary of Technical Terms*, not a search.
- **Does قصاصة have any technical usage a native codicologist would recognise?** None found, but
  "I searched and found nothing" is weaker than a specialist's confirmation — and this word may ship
  in 15 locales. Alternative with better lineage if ever needed: **تعليقة** ("a thing hung/attached"
  — *the etymology is the design spec*). قصاصة has better transparency; تعليقة better pedigree.
- ~~**Did the classical Arabic page have a PARAGRAPH unit at all?**~~ **✗ CLOSED — the research was
  WRONG, corrected by the Boss (2026-07-19): "No. It does have a paragraph. What you are referring
  to is the Quran's structure."**

  The study reported separation by rubricated heading, overlining and coloured circle rather than
  indentation, and graded فِقْرة as modern usage. Every marker it cited is **Qur'anic** mise-en-page
  — āya separators (rosettes/coloured circles), sūra headings, overlining — i.e. the layout of a
  specific **recitational/liturgical** text, wrongly generalised to all Arabic prose. **Ordinary
  Arabic prose manuscripts (scholarly, literary, administrative) paragraph normally.**

  The paragraph therefore stands as an anchor unit for قصاصة; nothing downstream needs rethinking.

  **The lesson, which generalises:** a web-search agent reading about a tradition will over-weight
  its most-documented *sacred* text and mistake that text's conventions for the culture's. The study
  had itself graded this claim "needs a codicologist" — and the Boss is one.

  **→ This is what the exclusion ruling above actually fixes.** My first response was to warn the v2
  agent to watch for the same trap in the Hebrew material — i.e. to patch the instance. The Boss's
  ruling removes the whole class instead: with sacred texts out of scope, *no* tradition can be
  characterised from its most-venerated book. **The stronger fix was the categorical one, and it was
  his, not mine** — a research-side instance of Solve-the-Class-Not-the-Instance.
- **Whether users sustain graded annotation KINDS** or collapse them to one. History is unambiguous
  the kinds were distinct and useful — but they were maintained by a professional scholarly culture
  with institutional incentives.
