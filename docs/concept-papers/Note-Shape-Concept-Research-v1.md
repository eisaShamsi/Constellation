# Note Shape — Concept Research v1

**Status:** Research synthesis. Not a plan, not an Architect doc, not an approval to build.
**Date:** 2026-07-20
**Concept under research:** A note SHAPE as a synthetic container — the claim that a knowledge system should deliberately author the constraints a physical container got from physics for free, across five candidate dimensions (extent · persistence · attachment · signal · addressability).
**Dependent features:** shape graduation (a scrap that outgrows itself becomes a page) and Qusasah (annotation attached to a note without editing the note).

---

## 0. What this document rests on, and what it deliberately excludes

This synthesises six parallel research tracks. **Every load-bearing claim in every track was then put through an adversarial verification pass**, and the verification results — not the original research claims — are what this document reports. That distinction matters: of the ~40 load-bearing claims checked, **zero came back fully CONFIRMED-as-stated in tracks A, B, C and F**; the overwhelming verdict was PARTLY_CONFIRMED, meaning the citation was real and honestly quoted but the inference drawn from it was wider than the source supports. Two claims were effectively refuted. Section 9 records every one.

**No historical, codicological, manuscript, scribal, or pre-20th-century evidence is used anywhere in this document.** A previous research pass grounded this concept in manuscript history — marginalia, glossed pages, commonplace books. That pass was adversarially verified and largely collapsed: one central claim was refuted outright, others inverted, and the evidence base turned out not to support the design recommendations built on it. Modern product history (a product shipped, users complained, the vendor reversed) is in scope and is used heavily; it is product evidence, not historical framing.

**Grading convention, applied to every claim below:**
- **[sourced]** — a real, named, re-findable source was read and says this.
- **[recalled]** — believed from general knowledge; could not be verified in these passes.
- **[unknown]** — nobody in this chain established it. Say so; do not guess.

Where a claim is [sourced] but the *design inference* from it is the researcher's own reasoning rather than the source's, it is marked **[sourced — inference is ours]**. That distinction is the single most common failure this verification pass caught, and it is where the previous pass died.

---

## 1. Headline

**The concept survives, but not in the shape it was proposed, and its most intuitive dimension is its worst-evidenced one.** The premise that a container's constraints change what gets written is genuinely established — causally, at scale, with a control arm. But the five dimensions are not peers: **signal is structurally prior and well-supported; extent is the most intuitive and the most hostile to Constellation's actual purpose; persistence, attachment and addressability are largely unexamined by the evidence.** The two dependent features diverge sharply in how well they are grounded. **Qusasah is well-supported** — every product examined converges on the same architecture, and the one product that shipped the alternative withdrew it for data loss. **Automatic shape graduation, as specified verbatim by the owner, is contra-indicated by every field that speaks to it** — the type-systems literature calls the inference unreliable in principle, the only large-scale real-world instance of exactly this idea (Excel inferring type from cell structure) corrupted a fifth of a scientific literature and was fixed by adding an off switch, no shipped product does it, and the field observation of the behaviour it imitates found it was always user-initiated. The recommendation is not to refuse the feature; it is to split it — the container may graduate automatically, the *kind* must be proposed.

---

## 2. Does the premise hold? — does the shape of the capture surface change what gets written?

**Yes for the artefact. No for the mind. And the split is the finding.**

### It changes the artefact — [sourced], causal, controlled

Gligorić, Anderson & West, *"How Constraints Affect Content: The Case of Twitter's Switch from 140 to 280 Characters"*, ICWSM 2018 (arXiv:1804.02318). 4M tweets before / 1.9M after the limit change; matched-pair design (12K pairs matched for users, 4K for topics); Kolmogorov–Smirnov tests with Dunn–Šidák correction at α=0.00365. Under the constraint, writers used more abbreviations, more contracted auxiliaries, more `&`, fewer `and`, fewer definite articles.

**The control arm is what makes this credible and it is what most citations of this work omit.** A parallel study on tweets at 91–100 characters — far below the limit, so never constrained — found only **one** significant difference in 14 features, ruling out community-wide norm drift. A container's limits demonstrably change what gets written.

**Two corrections the verification pass forced, both of which narrow the claim:**

1. **"Fewer hashtags" is not significant.** Table 1 gives 0.53→0.54, unbolded in every column. The paper's own prose overstates its own table; the research inherited that error. Drop it.
2. **The one significant control difference is contracted auxiliary verbs** — 0.39→0.29 in the *unconstrained* band, a larger relative move than the −17% inside the constrained band. That feature is contaminated by the very drift the control existed to rule out. The authors hedge precisely ("hinting at the length constraint as the cause for the **other** features"); the word "other" is load-bearing and was dropped.

**What survives clean:** `and` and `&`, significant under both matchings and clean in the control. Two features. Effect size ~0.04–0.05 occurrences per tweet — roughly one extra ampersand per twenty tweets. Statistically real at 12K pairs; substantively minute.

**And the prose-level features were all null.** Words per sentence 17.81→17.86. Lexical variation 0.9089→0.9081. Lexical sophistication unchanged. Fraction of lexical words 56.45%→56.39%. **The constraint changed token-level compression, not sentence structure, readability, or lexical richness.** [sourced]

> **Consequence:** Constellation may say "a binding extent limit measurably changes how people compress at the boundary." It may **not** say "a shape that constrains extent will demonstrably alter the prose people put in it" — the study measured prose form directly and found nothing.

A second finding corroborates the container's power at corpus level: after the limit doubled, tweets *at* 280 post-switch were syntactically and semantically similar to tweets at 140 pre-switch, with topic prevalence correlating 0.91 — the same "squeezing" register. (Gligorić et al., arXiv:2009.07661, 2020.) [sourced — **preprint, never peer-reviewed**, no independent replication]

But peer-reviewed work by an independent group cuts the other way at corpus level: Boot, Tjong Kim Sang, Dijkstra & Zwaan, *Humanities and Social Sciences Communications* 2019 (s41599-019-0280-3), ~1.5M Dutch tweets, found the **whole corpus** changed register after the switch — more articles, conjunctions and prepositions, more formal language, fewer textisms. Writers stopped compressing. [sourced]

> **Reconciliation:** the container shapes the **tail**, not the mode. Constellation should expect a shape to do its work on the minority of notes that reach for its edge and to leave the typical note untouched. (Twitter's modal tweet length was 34 chars before and 33 after.)

### It does NOT change the writer — [sourced], and this plank has largely failed replication

The "form of the capture surface changes encoding and retention" claim is the weakest empirical element of the whole concept.

- Morehead, Dunlosky & Rawson, *Educational Psychology Review* 31(3), 2019 — direct replication + extension of Mueller & Oppenheimer (2014): "performance did not consistently differ between any groups, including a group who did not take notes."
- Urry et al., *Psychological Science* 2021 (10.1177/0956797620965541), n=74/68 — replicated the *process* finding (laptop users write more, more verbatim) but NOT the outcome: "results do not support the idea that longhand note taking improves immediate learning via better encoding."
- Voyer, Ronis et al., *Contemporary Educational Psychology* 2021 (10.1016/j.cedpsych.2021.102025) — multilevel meta-analysis, 77 effect sizes, 39 samples: mean effect **not significantly different from zero**.
- **Counterpoint, reported honestly:** Flanigan, Wheeler, Colliot & Kiewra, *Educational Psychology Review* 2024 (10.1007/s10648-024-09914-w), 24 studies: Hedges g=0.248 favouring handwriting for achievement. Small, and in tension with the direct replications. **We could not determine whether the difference comes from inclusion criteria, take-vs-review design, or publication-bias handling.** [unknown — conflict reported, not resolved]

> **Consequence, non-negotiable:** **No Constellation shape feature may be pitched to the user as "this helps you remember" or "this helps you think better."** That warrant is not available. If such a line appears in help text or a tutorial, it is unsupported.

### The theoretical home should be dropped

Affordance theory is contested in its own literature. Oliver, *E-Learning* 2(4), 2005: the concept "has drifted so far from its origins that it is now too ambiguous to be analytically valuable." Kaptelinin & Nardi, CHI '12: Gibsonian affordance is "of limited relevance to HCI research" and most HCI usage is "essentially incompatible with Gibson." Norman coined "signifier" in 2013 *because* designers were misusing "affordance." [sourced]

> **Consequence:** drop affordance framing from the concept paper. It buys contested vocabulary, not support. The concept stands on its own product-level terms.

**Norman's anti-affordance survives** and it is what makes signal structural rather than cosmetic: an anti-affordance must be **perceivable** to function. [sourced — concept confirmed via multiple concordant secondaries; **Norman's verbatim wording could not be verified**, both the DOET PDF and jnd.org failed]

> A shape whose denial is invisible until the user hits it is a broken anti-affordance — experienced as the app breaking, not as a container having a form. **Signal is a dependency of extent/persistence/attachment, not a peer dimension.**

### Verdict on the premise

| Dimension | Evidence status |
|---|---|
| **Signal** | **Strongest.** Norman's perceivability requirement; the bookmarks study (§4f); the visible-orphan reversal (§4g). Structurally prior to the others. |
| **Extent** | **Best-measured and partly hostile.** Real but tail-only and token-level effects; a hard cap is the mechanism users reject; and the field's most on-point PIM study lists "unconstrained content" as an *unmet need*. |
| **Persistence** | **Weakly supported, real.** Users already expect scraps to decay (TOIS 2008, §4). No constraint literature bears on it. |
| **Attachment** | **Real but categorically more expensive than the others** — see §5. Not a free dimension. |
| **Addressability** | **Standardised, not validated.** W3C blesses the data model; nobody measured whether users want it. |

**The honest formulation, narrower than the concept as proposed: a note shape changes the note, not the mind.** That is still enough to make shape a first-class design object, because Constellation's value is the corpus it accumulates. It forecloses every cognitive claim.

---

## 3. Where the evidence contradicts the owner's stated requirement — stated plainly, up front

The requirement, verbatim: *"I could start with Post-It, form and shape, but if I pass its constructed limitation, it should transfer itself to the next kind, automatically."*

**Four independent fields converge against the word "automatically," and none support it.** This is the single most important finding in the six tracks.

1. **Type theory.** Inferring kind from structure ("it accreted dated entries, therefore it is a journal") is *accidental conformance* — the named, documented failure mode structural type systems are known for. Two things with the same shape and different meanings become interchangeable. A note with dated entries may be a journal, a changelog, meeting notes, or a note that happens to contain dates. [sourced] *(Verification narrowed this: what makes accidental conformance harmful is **silent substitution at a boundary**. The literature supports "a structural match must never silently confer identity," not the broader "structure may never classify." The narrower proposition is the one to cite — and it is sufficient.)*

2. **The one real-world instance at scale was a disaster.** Excel inferring type from the structure of user cell contents converted gene names to dates. Ziemann, Eren & El-Osta, *Genome Biology* 2016 (10.1186/s13059-016-1044-7): ~20% of papers with supplementary gene lists corrupted. Abeysooriya et al., *PLOS Computational Biology* 2021 (10.1371/journal.pcbi.1008984): **>30% by 2020** across ~10,000 papers — rising, not falling. HGNC ultimately **renamed the genes** (MARCH1→MARCHF1, SEPT1→SEPTIN1). The world changed its data to escape the inference. [sourced] The inference was (a) automatic, (b) silent, (c) destructive of the original value, (d) not reversible after save. Constellation's graduation must fail all four.

3. **The vendor's fix was not a smarter classifier.** Microsoft shipped per-conversion off switches (July 2022 Windows, October 2023 Mac + more formats) and a warning when conversions are enabled. [sourced via secondary reporting quoting an Excel PM — **first-party Microsoft release notes were not located**]

4. **The behaviour it imitates was always user-initiated.** Bernstein, Van Kleek, Karger & schraefel, ACM TOIS 26(4), 2008 — 27 knowledge workers, 533 information scraps analysed. "Transfer" from scrap to fuller representation is a real observed behaviour, but it was **rare, importance-triggered, and always something the person chose to do**, often as an act of *re-interpretation* (filling gaps to make the note "sixty-day proof"). An automatic transfer fires on volume, which is not the trigger people use, and performs none of the re-interpretation that makes transfer valuable. [sourced]

5. **No shipped precedent exists.** Every type-change feature located across Notion, Linear, Capacities, Anytype and Tana is user-invoked from a menu. [sourced — **bounded negative**: Slack threads, X/Twitter consecutive-post threading, and GitHub comment→issue were not investigated and could qualify.]

6. **The closest production analogue to a system computing a state change on user content and acting on it is GitHub's automatic "outdated" verdict — and it is one of the most persistently complained-about behaviours in its product**, with users reporting merged code containing unresolved feedback they never saw. [sourced: GitHub Community Discussions #130618, #23138, #30638] Users tolerate the system **observing and marking**; they resent it **deciding and acting**.

**The one serious counter-argument, reported because it cuts against this conclusion:** Aza Raskin, *"Never Use a Warning When You Mean Undo"* (A List Apart, 2007), and the current SaaS consensus: confirmation prompts suffer habituation. A prompt on every threshold crossing becomes a reflex dismissal within days, at which point the "proposal" is theatre. [sourced — thesis and title verified; full article read only at summary level]

**The resolution, and it inverts the usual build order: reversibility determines whether automatic is permissible at all.** If graduation is provably lossless and one gesture from being undone, act-then-offer-undo is defensible. If it is lossy — which the Linear precedent says is what happens the moment relational structure is involved (§4e) — it must ask. **Therefore: build the byte-exact revert first. Only then is automatic graduation even on the table.**

**Recommended split, which honours the requirement rather than refusing it:**

- **Container graduation → automatic, silent, reversible.** A scrap that outgrows Post-It extent gains page affordances. No judgement is made, nothing is lost, nothing is reclassified. This is exactly what the owner's sentence describes — *the container must not block him.*
- **Kind graduation → proposed, evidence shown, never automatic.** "This note has 14 dated headings — has it become a journal?" is a judgement with consequences for how the note is read, indexed and signalled. Automating it is the Excel failure.

That split is also directly supported by the closest analogue to graduation in practitioner method — see §4d.

---

## 4. The seven answers

### a. Does classification-at-capture pay for itself?

**No. Confidence: HIGH — this is the clearest answer the empirical literature gives.**

- Whittaker, Matthews, Cerruti, Badenes & Tang, *"Am I wasting my time organizing email?"*, CHI 2011 — 345 long-term users, 85,000+ refinding actions on an instrumented client. **"High filers were no more successful at finding messages than low filers"**, and were *less efficient* because folder-access took longer than scrolling. Tagging was **1% of all accesses** despite a dedicated tagging pane. [sourced — verified verbatim from full text]
- Bergman, Whittaker & Schooler, *JLIS* 2021 (10.1177/0961000620949652) — 50 participants, 250 bookmarked targets: only **16%** were retrieved via the Bookmark facility, and of those, only 9 via the menu hierarchy versus 32 from the always-visible toolbar. Bookmarked pages were retrieved **no better** than unbookmarked ones. [sourced]
- Bernstein et al., TOIS 2008 — direct interview evidence that a type step at capture doesn't slow capture, it **relocates capture out of your app**: *"Starting in Outlook forces me to make a type assignment, assign a category, set a deadline, and more; that takes too much work!"* and *"If it takes three clicks to get it down, it's easier to e-mail."* [sourced]

**Three corrections the verification pass forced — all narrowing, none reversing:**

1. **The email study measured EMAIL, and the file literature from the same author points the opposite way.** Bergman, Beyth-Marom, Nachmias, Gradovitch & **Whittaker**, ACM TOIS 2008: navigation is **56–68%** of file retrieval events versus search at 4–15%. Bergman, Whittaker et al., JASIST 2010: 296 participants, 5,035 retrieval steps, **94% navigation success**. Benn, Bergman … Whittaker, *Scientific Reports* 2015: folder navigation recruits spatial-navigation brain structures. **For files — Constellation's medium — location-like structure is the dominant retrieval route.** [sourced]
2. **The email study's "costs time" is retrieval-side, not capture-side.** It never measured the time cost of filing at capture. No study located measures that. [unknown]
3. **The tagging figure measures retrieval-by-tag (clicking a tag), not tag application.** The paper reports no count of tags created. It cannot distinguish "nobody tagged" from "people tagged but never navigated by tag." [sourced — measure mismatch]

> **The operative rule is about WHICH KIND of organiser, not organising per se.** Location-like organisers (Library, Folder — one home, stable, spatially navigable) are first-class retrieval infrastructure and Constellation should keep investing in them. Facet-like organisers (tags, and any "shape" behaving as a many-valued label) are weak retrieval routes in **both** media. **Never gate capture on either.** Shape must be assignable later or inferred, never demanded.

**Before this drives a decision, one question the evidence cannot answer for you: is a Constellation "shape" a PLACE (one home per note) or a LABEL (many per note)?** The literature gives opposite answers for the two. This must be ruled on.

### b. Do people sustain multi-kind schemes, or collapse them?

**They collapse imposed schemes and sustain self-invented ones. Confidence: MODERATE-HIGH on the first half, LOW-MODERATE on the second.**

Against imposed schemes:
- Tagging at 1% of retrieval operations with a dedicated UI pane. [sourced]
- Van Kleek, Bernstein, Panovich, Vargas, Karger & schraefel, *"Note to self"*, CHI 2009: *"traditional PIM divisions between applications and data types are **intentionally broken** when participants are given the opportunity."* [sourced]
- Evernote publishes a support article on recovering from "tagging chaos." The existence of the article is the evidence. [sourced]
- Capacities' documentation contains a three-question test before creating an object type. [sourced — **but see §9: this was substantially misread**]

For self-invented markers — the most useful positive finding in this area:
- Same CHI 2009 paper: *"several participants adopted syntactic conventions… Several users prefixed words with '@', while another surrounded words with asterisks '\*\*'… '!! means really important!'"* — and **those exact strings then reappeared as search queries.** [sourced]

> **Consequence, and it is The Constellation Way applied literally:** don't ship a fixed kind vocabulary as a menu — the tagging evidence says a menu will not be used. Ship an inline convention space, **observe what the user invents**, and propose promoting their own marker to a first-class kind with the evidence shown ("you've written `!!` on 14 notes"). That is the only route to graded kinds this evidence supports.

**Unverified corroborating numbers — do NOT quote these:** Windows-file tagging at 23% of participants, Gmail folder/tag split 64/36 (Bergman et al., JASIST 2013 — closed access, figures from search summaries only); "failed folders" 39%→16% (Whittaker & Sidner CHI 1996 / Fisher et al. CSCW 2006 — summaries only). [recalled]

### c. Has any product made note-typing mandatory at capture and made it stick?

**No product examined has. Confidence: MODERATE (bounded by what was surveyed).**

The strictest model found is **Capacities** — "Every object in Capacities has a type" — but the escape hatch is built in: `page` is a generic built-in type, and the Daily Note functions as an untyped-feeling inbox. **Tana**'s supertags are optional and added at any time by typing `#`. **Anytype** has types but a default. **Obsidian**'s frontmatter typing is entirely voluntary. [sourced]

**Drafts is the polar case and the strongest single data point.** Origin story verified at the publisher's own episode page (App Story Ep. 12): Greg Pierce began an email that needed to be a text message and built an app to *start with text and decide later*. The app opens to an empty keyboard-ready page. **The strongest evidence is what it still refuses to do 14 years and one full rewrite later:** workspaces do **not** auto-tag new drafts, and "Default tag for new Drafts in a workspace" remains an **open feature request** on its own forum. It added tags, flags and workspaces and still declines to classify anything at creation. [sourced]

**Two corrections:** "always opens to an empty textbox" is false — there is a configurable "New Draft After" timeout, plus pinning and Focus Mode, all of which return you to an in-progress draft. And opt-in pre-classification *does* exist (long-press `+` to tag; templates that assign tags).

> **The precise invariant, which is narrower and more useful than "always blank":** **no path into the capture surface may require a classification decision.** Constellation may — and probably should — return the user to an in-progress note rather than a fresh Post-It. That does not violate the principle. Classification at capture may exist as an opt-in reach; it must never be on the default path and must never block.

### d. Should graduation be automatic, proposed, or manual?

**Proposed, with manual always available. Automatic permissible only after byte-exact reversal is proven. Confidence: HIGH on the direction, and it converges from four unrelated fields.**

Full argument in §3. One addition here, from the closest analogue in practitioner method:

**Bullet Journal migration.** Verified verbatim at bulletjournal.com: *"If an entry isn't worth the effort to rewrite, then it's probably not that important. Get rid of it,"* and *"It may seem like a lot of effort to rewrite items over and over, but that's intentional. This process makes you pause and consider each item."* [sourced]

**But the verification pass corrected the mechanism, and the correction changes what to build.** The claim was that the *friction* is the value. It is not — the method's own causal chain is effort → pause → **consider each item** → decide. The mechanism is a **forced decision point at a boundary**; friction is one medium-specific way to create one, chosen because paper offered no other. **Decisive evidence: Carroll's own Bullet Journal Companion app does NOT auto-migrate and does NOT reproduce the rewriting friction — it substitutes a different forcing function, 72-hour log-item expiry.** [sourced] When the medium changed, the method's author kept the decision and dropped the effort.

> **Consequence:** do NOT make graduation deliberately tedious — nothing supports manufactured friction, and the method's own author declined to build it. The rule is **cheap action, unavoidable decision.** A proposal with the evidence shown that the user must actually answer — accept, decline, or defer — is faithful. A decay/expiry signal is the second faithful implementation, with direct precedent. **A silent auto-graduation with an undo is not faithful, because there is no moment where the user decides.**

**Caveat, and it limits transfer:** BuJo's decision is about *discarding*. Constellation's graduation is about *promoting*. The source says nothing about promotion.

**One more empirical steer on the trigger.** The intuition that users press against their container is not what the largest natural experiment shows: after Twitter's ceiling doubled, tweets near the limit became *far less* common than before. [sourced] **But this does not transfer to a desktop composition surface.** The peer-reviewed follow-up (Gligorić, Czestochowska, Anderson & West, CSCW 2022, doi:10.1145/3555659) disaggregates by client: on **Web**, estimated pre-switch cramming was 18.35%, actual post-switch usage above 140 chars was **24.81%**, cramming re-emerged at 6.88%, and the ceiling needed to fit 95% of tweets **rose by 67 characters**. Mobile behaved oppositely. **On a desktop writing surface, a fill-percentage trigger would fire often, not rarely.** [sourced]

> So prefer structural triggers (recurring dated entries, accreting headings, inbound links) — but **not** on the stated warrant that fill triggers are too rare. That warrant is empirically wrong for this form factor. The honest reason is that **fill measures volume while structure measures kind**: a 4,000-word single-argument essay is not graduating into anything; a 600-word note with six dated entries and three inbound links already has.

### e. Is there a safe pattern for in-place type change, and is it reversible?

**Yes for the pattern. Reversibility is NOT inherited — it must be designed, and it is where the field's own attempts break first. Confidence: HIGH on the pattern, HIGH on the warning.**

**The pattern (Elasticsearch).** A field's mapped type is immutable; changing it requires a new index and an alias swap: build the new representation alongside, reindex with an explicit coercion script, swap read traffic atomically via alias. [sourced — Elastic's own "Changing Mapping with Zero Downtime" describes the alias switch as "a single atomic step"]

**Three corrections, one of which surfaces a better option than the pattern itself:**

1. **"Refuses in-place type change" is overstated, and the overstatement hid the best idea.** Since 7.11, Elasticsearch ships **runtime fields** precisely to reinterpret a field's type at query time without reindexing — a runtime field shadows a mapped field of the same name, so the returned type changes with the stored representation untouched. This is instantly reversible, zero-rebuild type change. Cost is documented: evaluated per query, "considered expensive." [sourced]
   > **The Constellation analogue is strong: render a note AS IF graduated — a display/interpretation layer over an unchanged file — before any write occurs.** That gives reversibility *by construction* rather than by retained-copy bookkeeping. This was the single most valuable thing the verification pass surfaced on graduation mechanics.
2. **"Keep the old index for rollback until validated" is NOT sourced.** Elastic's canonical procedure ends *"delete the old index."* Elasticsearch exposes no revert operation. The retain-for-rollback step is practitioner convention presented as documented practice. If Constellation retains a pre-graduation state, that is **Constellation's own design decision** and must be justified on Constellation's terms. [correction — was graded sourced, is not]
3. **The clean four-step story hides the catch-up problem.** While the reindex runs, writes arriving via the alias land on the *old* index. That reconciliation is where the practical difficulty of this pattern actually lives — and a note editor with a live buffer and a debounced save has exactly the same race.

**The disanalogy that breaks reversibility, and it must be ruled on.** After an alias swap, the old index is frozen and nobody edits it, so "point back" is unambiguous. A retained pre-graduation note has no such guarantee: **the moment the user edits the graduated note, the retained prior state diverges, and "revert to Post-It" must answer a question Elasticsearch never has to answer** — discard post-graduation edits? merge? refuse? Either specify this, or scope revert to a short pre-edit window, which is an *undo* and should be named one.

**Schema coexistence (MongoDB).** Verified: the Schema Versioning Pattern "lets you have different versions of your schema in the same collection, which avoids large-scale schema migrations." Multiple versions coexist; application code is version-aware. [sourced]

**Correction that touches the build:** in every source, lazy migration fires on **access — primarily read**. The research restated this as "ride an existing user write, never a background scan." **That stricter rule is correct for Constellation, but it is a Constellation-specific tightening derived from File Over App ("never modify file content silently"), not something inherited from MongoDB.** Read-triggered write-back would mean *opening* a note rewrites it on disk. State it as ours, not as inherited.

**Cost to budget, and it is now permanent:** under a write-only trigger, a note never edited after a shape change keeps its old shape indefinitely. **The read path carries every shape ever written, for the life of the app.** Two guardrails: normalise-on-read into a single in-memory form regardless of on-disk shape; and make every new shape readable by an unchanged reader wherever possible (additive frontmatter keys, never renamed or restructured ones).

**Product precedent, and it is the warning:**
- **Notion** "Turn into page" — manual, menu-invoked, content-preserving, documented reversible. Reversible **because the underlying atom (a block) is unchanged — only its container/addressing changes.** [sourced]
  > Strongly implies Constellation's shapes should be a **property of** the note, not a different storage kind. If graduation changes declared constraints but not the file's substance, reversal is a property flip and stays trivially safe. If it rewrites the body, reversibility becomes a diff problem.
- **Linear** issue→project — verbatim from the docs: *"the original issue and its sub-issues are added to the project as standalone issues. The original issue is renamed to indicate the conversion, and **sub-issue relationships are removed**."* No documented reverse. [sourced]
  > A well-resourced product doing exactly this operation chose a **one-way, relationship-destroying** conversion. **Constellation's Qusasah attachments and typed living links ARE relational structure. What happens to a Post-It's attachments on graduation must be explicitly ruled on — Linear's precedent says that is exactly where reversibility breaks first.**
- **Anytype** — the research cited a user reporting delete-and-recreate as the workaround, losing creation dates and links. **The verification pass found this was the poster's mistaken premise, retracted by the poster himself 46 minutes later in the same thread.** Anytype had already solved graduation correctly — in-place type mutation on a stable object id. What it had not solved was **discoverability**: the working path lived only in a Set right-click and an unlabelled canvas click, and the relations panel showed a **lock icon** on a field that was mutable elsewhere. That inconsistency convinced an experienced user the feature did not exist, so he wrote up a destructive procedure as the recommended alternative. [correction — see §9]
  > **The real lesson, and it is sharper than the one claimed: a correct write path that users cannot find gets routed around by hand, and the hand-routing is what loses the data.** Graduation must be reachable from the note itself — context menu AND property surface — and a mutable property must never render as locked.
  > **New risk this surfaced:** Anytype bug reports show relation data emptied on type change and title/body desync. **Preserving identity does not preserve shape-specific properties.** Constellation must explicitly rule on what happens to properties that exist in the old shape and not the new one.

### f. What is the production pattern for anchoring over an editable host?

**Redundant multi-representation anchoring, resolved cheapest-first at read time, with the original retained forever and a context snapshot captured at attach time. Confidence: HIGH — four independent systems converge.**

- **Hypothes.is** stores three selectors per annotation (RangeSelector, TextPositionSelector, TextQuoteSelector) and resolves through a fallback chain. **Verified against the live shipping source, not just the 2013 blog post:** `src/annotator/anchoring/types.ts` line 199 is `const contextLen = 32;` — the 32-character prefix/suffix window is a **current shipping constant**, not an archaeological figure. [sourced]
- **GitHub** stores current AND original coordinates for every dimension (`line`/`original_line`, `commit_id`/`original_commit_id`) plus a `diff_hunk` context snapshot. [sourced]
- **W3C Web Annotation Data Model** blesses multiple selectors per target: *"There MAY be 0 or more selector relationships… Consuming user agents MUST pick one of the described segments, if they are different."* It prescribes **no algorithm** for choosing, and has **no vocabulary at all for a selector that fails to resolve** — zero occurrences of "fail", "unable", "not found". [sourced]
- **URL Text Fragments** (`:~:text=[prefix-,]textStart[,textEnd][,-suffix]`) use the identical quote+context shape — independent convergent evidence that this is the portable representation. [sourced] Its failure mode is the one to avoid copying: browsers fail **silently** to top-of-page.
- **Zotero** stores PDF annotations in its database, not the file, for "fast, conflict-free syncing" — and **shipped the embed-in-host alternative and withdrew it** because it "could result in file conflicts and lost data." [sourced]

**Four corrections that change the implementation:**

1. **The Hypothes.is chain is THREE tiers, not four, and the 2013 blog post's architecture was abandoned.** Blog strategies 3 (context-first fuzzy) and 4 (last-ditch exact-only) were merged. In `match-quote.ts`, candidates are generated from the **exact quote alone** by edit distance (`maxErrors = Math.min(256, quote.length / 2)`); **prefix, suffix and position never generate candidates — they only rank them**, as weights: quote 50, prefix 20, suffix 20, position 2 ("used as a tie-breaker"). Building the blog's version would ship an architecture its author retired.
2. **Steal the cross-check the research omitted, and it may be the most valuable detail here.** Tiers 1 and 2 re-validate their result against the stored quote (`maybeAssertQuote`) and **fall through on mismatch**. Without it, a stale-but-valid range silently anchors to the wrong text — a content-integrity failure of exactly the class LL-014's three-strike law is already spent on.
3. **Budget the fuzzy tier explicitly.** hypothesis/client issue #3919 documents fuzzy quote anchoring blocking execution for **over 10 seconds** on long documents, ~60% of that in imperfect-match resolution, resolved serially. This collides with Performance Rules 1 and 3.
4. **The orphan rate is high, and the citation must be labelled correctly.** The **peer-reviewed** version (Aturban, Nelson & Weigle, TPDL 2015, doi:10.1007/978-3-319-24592-8_2) measured 6,281 annotations and reported **27% orphaned, 3.5% recoverable from archives, 61% at risk**. The 20,953 / 22% / 12% / 53% figures come from the **extended arXiv preprint** (1512.06195), not from a peer-reviewed paper. The research inverted these and dismissed the correct peer-reviewed numbers as search-tool noise. [correction — see §9]
   Two further method facts materially change how to read that number: attachment was tested by **exact string match**, not fuzzy anchoring, so 22–27% is an **upper bound**; and **"53% in danger" measures missing web-archive coverage, not anchor fragility** — citing it as evidence that anchors are precarious inverts what was measured.
   > **The paper's own conclusion is the one to act on and the research omitted it: "the need for archiving the target of an annotation at the time the annotation is created."** Prevention beats recovery. At capture time, store the quote, a generous context window, and a content hash. **A recovery affordance with no capture-time snapshot treats the symptom the study says to prevent.**

### g. How should a system show the user that an anchor became uncertain?

**Visible is necessary and demonstrably not sufficient. The bar is: visible AND reachable AND recoverable AND detected completely. Confidence: HIGH.**

- **Hypothes.is reversed a hide-it decision after user harm.** Verbatim: *"Before now orphans were not shown in the Hypothesis sidebar, we simply hid them from view… when you returned to the page, it would seem like annotations you had made before were gone, even though they were still discoverable from your profile."* Shipped an Orphans tab in client v1.2.0. [sourced]
- **But "visible" is not the bar.** Jon Udell's staff feedback page (Aug 2016, pre-release) describes the *earliest* state differently: *"Formerly they showed up in the sidebar mixed in with Annotations and Page Notes, but **nothing happened when you clicked them**."* Visible but **inert**. A dead orphan entry teaches the user their work is broken, not that it is safe. [sourced — **this contradicts the launch post; the two official sources disagree and the discrepancy is unresolved**]
- **GitHub shows its uncertainty verdict AND is still one of the most complained-about behaviours in its product**, because the label is coupled to *collapsing the comment out of the files view*. [sourced]
  > **The rule: an automatic system verdict about the user's content may change how something is PRESENTED, but must never reduce its REACHABILITY.** Badge it, sort it, group it, give it a tab. Never collapse it out of the primary view.
- **Detection must be complete, and this is the requirement the research missed.** hypothesis/product-backlog issue #954: annotations fail to anchor **and are not reported as orphans** — *"It's just sort of hanging around in limbo"*; one PDF where testing found four orphans while the client reported one. [sourced]
  > **An orphan surface whose detector under-reports is worse than no surface, because the user now trusts a channel that lies by omission.** Enforce as an invariant: every Qusasah is in exactly one of {anchored, orphaned}, no third reachable state, and `anchored + orphaned == total stored` at every read. Orphaned **by construction**, not by a heuristic that can fail to fire. This is the same shape as Rule 8 write-time derivation with a conservation check.
- **Weak corroboration, do not rely on it:** Microsoft Word's behaviour when anchor text is deleted is contested even within the single thread found — the poster says the comment is deleted, a community contributor says it persists but is hidden. No authoritative Microsoft documentation located. [unknown]

---

## 5. The Qusasah architecture

**Concept (the horse, per Concept Before Function):** *Qusasah lets you say something about a note without changing the note — so a past self's text stays intact while a present self argues with it.* The host is editable and also changes underneath us via file watcher, Syncthing, Git, or an external editor.

### 5.0 The dimension cost that must be acknowledged first

**Attachment is not a peer dimension.** Extent and persistence are enforceable locally and forever. Attachment creates an object whose validity depends continuously on another object's mutation history, with a measured failure rate of 22–27% in a system whose entire engineering effort is aimed at this problem. **And both available architectures have shipped and failed:** sidecar storage is acknowledged by Zotero to break down under external file modification; embedded storage was withdrawn by Zotero for "file conflicts and lost data." Constellation's local-first, file-over-app, sync-agnostic architecture puts it in the *hardest* version of the sidecar case — **the host file WILL change outside the app as a matter of design, not as an edge case.**

### 5.1 Two change sources, two regimes

**⚠️ This architecture is graded [recalled], not [sourced].** Every primitive is sourced. **No production system was found that handles both an owned edit stream and an unowned external-rewrite path for the same annotation set.** The assembly is our reasoning. It deserves scrutiny before it is built.

**Source A — edits inside Constellation.** CodeMirror 6 supplies exact position mapping through its own transaction stream. `ChangeSet.mapPos` and `RangeSet.map` exist as described and are battle-tested. Use a `StateField` over a `RangeSet`. **Do not invent a custom position-tracking scheme.**

**But the research's specific prescription — `MapMode.TrackDel`, treat null as the orphan signal — was empirically falsified against the shipped library (`@codemirror/state` 6.5.4, in Constellation's own `node_modules`):**

- `RangeValue.mapMode`'s own doc carries a qualifier that was dropped: it applies *"when its `from` and `to` are the same."* `Chunk.map`'s non-empty branch calls the **two-argument** `mapPos`, defaulting to `MapMode.Simple`. **For a spanning Qusasah range, `TrackDel` is never consulted. The instruction is inert.**
- `RangeSet.map` returns a `RangeSet`, **never null**. Dropped ranges are silently omitted — no null, no callback, no effect. **The "authoritative null" trigger can never fire.**
- Executed against the installed library: anchor at 5–10 in a 20-char doc, delete exactly 5..10 (the user selecting precisely the quoted passage) → **range SURVIVES as 5–5** and `mapPos(5,-1,TrackDel)` returns 5, not null. `TrackDel` requires *strict interior* containment, so boundary-aligned deletion is invisible to it. Replace 5..10 with "XYZ" → **range 5–8, silently re-anchored over unrelated text, zero signal.** Delete 5..9 → 5–6, a gutted one-character excerpt, zero signal.

**Corrected Source A design:**
1. Keep the `StateField` + `RangeSet` chassis.
2. **Drop `MapMode.TrackDel`.** Set `startSide: 1, endSide: -1` on the Qusasah `RangeValue` — verified empirically, this is what makes an exactly-deleted anchor collapse and drop. The drop comes from **side configuration**, not mapMode.
3. **Replace the null trigger with an explicit membership diff.** After mapping, compare surviving ids against the pre-transaction set; vanished ids are the orphaning event. This layer must be written; there is no built-in signal.
4. **Add content verification, which position mapping fundamentally cannot provide.** Persist the anchored text (or a hash) and re-verify against the mapped range after each transaction. Without it, the replace-in-place case silently re-anchors a Qusasah onto text it never quoted, and **no side/mapMode configuration prevents it.** Treat content-mismatch and shrink-below-threshold as first-class degraded states alongside dropped.
5. **Scope honestly: the edit stream exists only while a note is open in a live CM instance.** Notes touched by the rename cascade, watcher-adopt (PJ-070), federated writes, or simply not currently open have **no ChangeSet at all**. **Position mapping is an optimisation for the in-session fast path, not the durable mechanism.** The durable mechanism is content-based.

**Source B — edits from outside.** No edit stream exists. The mapped ranges are not stale, they are **void** — the offsets refer to a document that no longer exists, and using them will silently point at wrong text, which is worse than orphaning. Fall back to content-based re-anchoring over the new file text.

### 5.2 The persistence record

Per Qusasah, store:
- **(a)** the mapped character range (fast path, valid only within a known document state);
- **(b)** the **exact quoted text** — the durable anchor, and the verification key for every other tier;
- **(c)** a **32-character prefix/suffix context window** (Hypothes.is' live shipping constant);
- **(d)** a **content stamp** (hash) of the note as of the last in-app transaction — *this is the join between the two regimes*;
- **(e)** the **original anchor, retained forever** (GitHub's `original_*` pattern) — what lets an orphan render "here is what this remark was originally about";
- **(f)** a **context snapshot captured at attach time** (GitHub's `diff_hunk`, and the TPDL 2015 paper's own recommendation to archive the target at creation). Store it explicitly and immutably; do not recompute.

Serialize in a W3C-compatible form for portability. The standard blesses the multi-selector shape and hands back the hard part.

### 5.3 The resolution rule — write this down, do not let it emerge from code

W3C obliges the consuming agent to "pick one" and prescribes no algorithm. **Constellation's rule:**

> **On any file-watcher event: hash the file. If the hash matches the anchors' stamp, the mapped ranges are authoritative — do nothing. If it diverges, the mapped ranges are VOID; discard them and re-anchor every Qusasah through the quote+context cascade. Anything that fails becomes a visible orphan. Prefer the mapped position IFF the stamp matches; otherwise prefer the quote.**

**Cascade order (matching what Hypothes.is actually ships, not what its blog describes):**
1. Mapped range — **then re-validate against the stored quote; fall through on mismatch.**
2. Exact quote match at the stored position hint — **same validation.**
3. Fuzzy quote match: generate candidates from the exact quote by edit distance; **rank** with quote 50 / prefix 20 / suffix 20 / position 2.
4. Fail → orphaned, by construction.

**Performance envelope, non-negotiable:** the fuzzy tier runs **off the keystroke path** (Rule 1), **Rust-side or batched** (Rule 3), capped, cancellable, and never serially per-item across a large Universe. Hypothes.is shipped this exact code and froze pages for 10+ seconds. Constellation's advantage over Hypothes.is is **not** that its position tier is faster — it is that owning the edit stream and the file makes tier 3 a **rare recovery path** (external edits, Syncthing/Git merges, conflict sidecars) rather than the normal case.

### 5.4 Storage and rendering

- **Sidecar, never in the host body.** Zotero validates the model and, decisively, withdrew the embed alternative for data loss. This also satisfies the owner's literal requirement.
- **Sidecar-JSON-per-note is the shipped convention in this exact ecosystem** — the Obsidian "Side Comments" plugin stores annotation data under `.obsidian-side-comments/` with files/cache/backups subdirectories, keeping source files unmodified. [sourced — **its actual relocation algorithm was not read**] Worth matching for portability.
- **Anchor against raw Markdown source offsets, not rendered output.** The same plugin admits "reading-mode source mapping can still have edge cases for complex Markdown." Raw source is also what CodeMirror maps.
- **Visible on the host by default.** The bookmarks study is unambiguous: a curation artefact the user must open a layer to see is created and not read (32 of 41 retrievals came from the always-visible bar; 9 from the menu hierarchy).
- **Behavioural fit:** 75% of notes in the CHI 2009 study were never edited after capture. **"Revisit without editing" is the normal relationship people have with their own old notes, not an exotic one.** [sourced — but note the ≤10-day window; see §9]

### 5.5 What is explicitly NOT solved

The Qusasah anchoring problem over an editable, externally-mutating local file **has no product precedent that was located.** Hypothes.is anchors to immutable published pages; PDF and Google Docs comments anchor to hosts whose edits flow through the same application. **Design defensively: expect anchors to break, and design the broken state as a first-class visible outcome rather than assuming re-anchoring will succeed.**

---

## 6. What the evidence says NOT to build

1. **Do NOT gate capture on a shape/kind decision.** A type picker at capture doesn't slow capture — it relocates capture out of your app. [sourced]
2. **Do NOT ship a user-authorable shape system, and do NOT ship a shape editor.** Every user-maintained classification vocabulary in this product space decays (Evernote tag chaos; Obsidian inconsistent frontmatter). *(Nuance: Capacities is NOT a precedent for a closed catalogue — see §9. If Constellation's shape set is closed, that must be justified on Constellation's own grounds.)*
3. **Do NOT sell shapes as improving memory, comprehension, or thinking.** The encoding claim failed two direct replications and a 77-effect-size meta-analysis. [sourced]
4. **Do NOT claim "the blank page paralyses people."** Targeted searching found only SEO listicles and vendor marketing with untraceable figures ("84% abandon", "3x higher abandonment" attributed vaguely to "Hotjar UX research" — no primary source located). The one academic source found points the **other** way: Mike Rose, *CCC* 31(4), 1980, attributes blocking to *rigid internalised rules and inflexible plans*. [sourced negative + [recalled] on Rose's method — his full text could not be obtained]
5. **Do NOT scaffold CONTENT.** The expertise reversal effect is real and well-replicated (Tetzlaff et al., *Learning and Instruction* 98, 2025: 60 studies, 176 effect sizes, N=5,924; novices d=+0.505 under high assistance, experts d=−0.428 under low assistance — a genuine crossover). *(But see §9: the form/content dichotomy is NOT what this literature licenses. What it licenses is **avoidable-and-fading vs mandatory-and-permanent** — and it says over-support is the **cheaper** error.)*
6. **Do NOT impose one shape system-wide.** Logseq's block-as-atom is the most consistently cited reason long-form writers reject it. [sourced — practitioner writeups, weighted low]
7. **Do NOT enforce a hard extent cap that refuses input.** The Miro community dissent names the failure mode: *"I've used other tools where I reach a hard-limit and I find that frustrating."* **Never refuse the keystroke; change the container.** [sourced — one request, publicly disagreed with; **do not present as a groundswell**]
8. **Do NOT justify shapes on atomicity.** Zettelkasten.de — the most serious practitioner community on atomicity — has explicitly concluded the card size was an incidental implementation detail, "not a rigid law, but a guiding compass," with digital practitioners running to ~500 words. [sourced]
9. **Do NOT assume structure is self-justifying.** *Frontiers in Psychology* 2025 (10.3389/fpsyg.2025.1697151), n=134, 4-arm randomised: only Cornell beat plain sentence notes on retention (15.0 vs 12.4). Parallel and Digital bought nothing. The time × group interaction was **not** significant. And **cognitive load had no predictive value for retention** — motivation did. **Do not sell shapes as friction-reduction; that is the dimension that empirically did nothing.** [sourced]
10. **Do NOT let an automatic verdict remove anything from view.** GitHub's most-complained-about review behaviour. [sourced]
11. **Do NOT expose the five dimensions as nested configuration.** NN/g's progressive-disclosure article makes essentially no quantitative claims but states one concrete rule: *"designs that go beyond 2 disclosure levels typically have low usability."* Named shapes at level 1, dimensions at level 2, no deeper. [sourced]
12. **Do NOT quote "reduces task completion time 20–40%" for progressive disclosure.** The figure was returned by a search summary attributed to NN/g. **It is not in the article.** This is the exact failure mode this pass exists to prevent, occurring live during the research.
13. **Do NOT cite the physical-vs-digital sticky-note study (Jensen, Thiel, Hoggan & Bødker, CSCW 2018) on this question at all, in either direction.** The digital condition was stylus-drawn on note-sized cards — **the size constraint was held constant across conditions.** It varies materiality, not boundedness. It also has n=14 in 7 pairs, and its physical condition was deliberately handicapped. *(It does contain a usable finding, but a different one — see §7.)*
14. **Do NOT use any circulating journaling-abandonment statistic** ("87% quit in 7 days", "3 in 4 abandon", "8% keep a journal"). Every one traced only to marketing blogs with no cited methodology. [unknown]
15. **Do NOT make graduation deliberately tedious.** Manufactured friction is unsupported, and the Bullet Journal's own author declined to build it into his own app.
16. **Do NOT build any mechanism that assumes a knowable optimum constraint level.** Acar, Tarakci & van Knippenberg, *Journal of Management* 45(1), 2019, is the source for the inverted-U — and it presents that curve as **its own proposal**, repeatedly flagged as untested ("We are unaware of any direct empirical study on this linkage"). Nothing that estimates "the optimum," scores a note's constraint level, or nudges toward a computed midpoint. [correction — see §9]

---

## 7. Where the concept is now grounded

Claims that survived verification and may carry design weight, with honest confidence.

| # | Claim | Grade | Confidence |
|---|---|---|---|
| 1 | A binding extent limit measurably changes how people compress at the boundary (`&`/`and` robust across matchings and clean in the control). Token-level, tail-only, small. | [sourced] | HIGH for the narrow form |
| 2 | A container shapes the **tail** of a corpus, not the mode. Design a shape to work on notes that reach its edge. | [sourced] | HIGH |
| 3 | On a **desktop** composition surface, content presses against a limit far more than the aggregate/mobile picture suggests (Web: 24.81% above the old ceiling post-switch; 95% coverage required +67 chars). | [sourced, CSCW 2022] | MODERATE-HIGH |
| 4 | Signal is structurally prior: an anti-affordance must be perceivable to function. | [sourced — concept; verbatim wording unverified] | HIGH |
| 5 | Preparatory classification does not buy retrieval success (email, n=345, 85k actions); tagging is ~1% of retrieval operations. | [sourced] | HIGH within email |
| 6 | For **files** — Constellation's medium — location-like navigation is the dominant retrieval route (56–68% vs 4–15% search; 94% success). | [sourced] | HIGH |
| 7 | A required type step at capture relocates capture out of the app. | [sourced, qualitative] | MODERATE-HIGH |
| 8 | Users invent their own lightweight markers when a tool declines to impose a schema, and reuse them as search terms. | [sourced, qualitative] | MODERATE |
| 9 | Users expect temporary scraps to decay — persistence as a dimension is real. | [sourced, qualitative] | MODERATE |
| 10 | Scrap→richer-representation transfer is a real behaviour, but rare, importance-triggered, and always user-initiated. | [sourced, qualitative] | MODERATE-HIGH |
| 11 | Every classification precedent examined ships an untyped-feeling default capture surface; classification is applied afterwards. | [sourced] | HIGH |
| 12 | Redundant multi-representation anchoring + retained original + attach-time context snapshot is the converged production pattern. | [sourced ×4 systems] | HIGH |
| 13 | Sidecar storage for annotations; embedding in the host was shipped and withdrawn for data loss. | [sourced] | HIGH |
| 14 | Anchor orphaning is high-frequency (27% peer-reviewed / 22% preprint, exact-match upper bound), not an edge case. | [sourced] | HIGH for order of magnitude |
| 15 | Visible orphan state is required but insufficient; reachability must not be reduced, and detection must be conservation-checked. | [sourced] | HIGH |
| 16 | Reversibility comes from the underlying atom being unchanged (Notion), not from a retained copy; and it breaks first at relational structure (Linear). | [sourced] | HIGH |
| 17 | Read-time reinterpretation (Elasticsearch runtime fields) is a real, shipped, zero-rebuild alternative to destructive type change. | [sourced] | HIGH |
| 18 | Constraint framed as **controlling** harms; framed as the user's own **task** it does not. The best-evidenced leg is **expected evaluation** (Amabile, Goldfarb & Brackfield 1990: "consistently strong"). | [sourced] | MODERATE-HIGH for evaluation only |
| 19 | Shapes must be dismissible without friction — the expertise-reversal harm arises "especially if these learners can not ignore or otherwise avoid processing" the redundant material. | [sourced — verbatim from Kalyuga 2007] | HIGH |
| 20 | Where structured formats helped, the mechanism was **motivation**, not reduced cognitive load. | [sourced] | MODERATE |
| 21 | Capture friction and manipulation friction are separate problems with opposite medium preferences: digital pays a tax at externalising a thought and earns it back at rearranging what exists. | [sourced — Jensen et al. 2018, its actual usable finding] | MODERATE |
| 22 | Structure derived from what the user already does beats structure the user hand-files (threading 91% vs 85% success). | [sourced — **correlational, single study, 6pp, p just under .05**] | LOW-MODERATE |

**Item 18 carries a correction worth stating separately, because it inverts a prohibition.** Byron & Khazanchi, *Psychological Bulletin* 138(4), 2012 — 60 studies, 69 samples — found that **creativity-contingent rewards INCREASE creative performance**; only completion- and performance-contingent rewards have a slight negative effect. So: **Constellation may surface and celebrate that a user connected, contradicted, or synthesized. It may NOT rate how good a note is.** Recognition of creative activity and evaluation of artefact quality are different things, and the original claim collapsed them.

**And item 18's design consequence should be re-sourced.** "Shape assignment must be user-confirmable" is better warranted by **participant choice** as a moderator (Byron & Khazanchi 2012) and the autonomy-support literature (Slemp, Kern, Patrick & Ryan, *Motivation and Emotion* 2018) than by the social-vs-task dichotomy, two legs of which are weak or refuted.

---

## 8. What remains unknown

### (a) Checked and genuinely contested — surface the conflict, do not average it

1. **Does loosening a constraint improve or degrade discourse quality?** Gligorić et al. (ICWSM 2018) found constraint-era tweets slightly more successful at matched length. Jaidka, Zhou & Lelkes (*Journal of Communication* 69(4), 2019, n=358,242) found doubling the limit produced **more polite, less informal, more constructive** political discussion. Same natural experiment, opposite emphases — and the constructs differ (engagement vs civility), so it is not a clean contradiction. **Neither measures a private author's own thinking.**
2. **Does handwriting beat typing for learning?** Voyer et al. 2021 (77 effect sizes, null) vs Flanigan et al. 2024 (g=0.248 favouring handwriting). Both meta-analyses of overlapping literatures. **The source of the difference could not be determined.**
3. **Do 280-char post-switch tweets really reproduce squeezing?** The authors state in their own results section: *"there is no counterfactual observation."* Squeezing at 140 is causally demonstrated; squeezing at 280 is **pattern-similarity only**.
4. **Hypothes.is' pre-tab orphan state.** The 2017 launch post says orphans were hidden. Udell's Aug-2016 staff page says they were visible but inert. Two official sources, unreconciled.
5. **Is a Constellation "shape" a PLACE or a LABEL?** The retrieval literature gives opposite answers for the two. **This is a design ruling, not a research question — but it must be made before anything is built.**

### (b) Checked and no sources found

1. **Any quantified capture-friction study** — an experiment measuring how a required field changes whether capture happens. Searched specifically. Not found. If the owner wants a number, it does not appear to exist.
2. **Any note-app abandonment research.** Health/lifestyle app attrition exists; it is a different population and behaviour and should not be transferred.
3. **Any longitudinal measurement of whether note-app users sustain the organisational distinctions they set up.**
4. **Any product that automatically changes an object's type from its structure.** Bounded negative — Slack threads, X/Twitter threading, GitHub comment→issue were not checked.
5. **Any product implementing sub-range annotation over an editable, externally-mutating local file.**
6. **Any positive experimental result for authored constraint in personal capture** — better retention, better retrieval, more capture, or better thinking. None found in any track. Absence of evidence, not evidence of absence, but the concept currently has **no empirical proponent in the PIM field** — only a design intuition and one supportive dimension.
7. **Any measurement of when users accept vs reject an inference** against confidence, visibility, or reversibility. The "propose, not automatic" recommendation rests on convergent argument, not measurement.
8. **The prompt-fatigue threshold** — how many proposals per week turn a meaningful prompt into a reflex dismissal. That is exactly the number the graduation design wants.
9. **First-party Microsoft documentation of the Excel conversion-settings fix.** Secondary reporting only.
10. **Whether Notion's "turn into page" is reversible in the strong sense**, and what happens to links pointing at the created page.
11. **Whether Linear's convert-to-project has an undocumented reverse.** Silence is not proof of absence.
12. **Replication records.** Gligorić 2019 CSCW, Haught-Tromp 2017 (ηp²=.53 from one lab — an implausibly large effect, unreplicated as far as could be checked), Whittaker 2011, Jensen 2018, Aturban 2015, Troppmann 2024 — **none has a verified replication.** Several were graded [sourced] without this flag.
13. **Primary texts not obtained:** Kirsh 1995 (scanned image PDF, no text layer); Rose 1980 (paywalled, mirror refused); Norman's DOET verbatim; Malone 1983; Bergman's MIT Press book; Klettke et al. SCDM 2016 (unparseable); Haught-Tromp 2017 lit-review quote (closed access); Kalyuga et al. 2003 (author order was also mis-cited).

### (c) Can only be answered by building a prototype and testing it with the owner

**These are the questions where more desk research will not help. Each one is cheap to answer inside Constellation's own 7,600-note Universe, and answering them from the owner's own evidence is more Constellation-Way than citing anyone's sources.**

1. **Does the blank-page premise hold in this Universe?** Instrument note-creation abandonment: created-then-empty notes, time-to-first-keystroke. If created-then-abandoned-empty notes are rare, the blank-page argument is gone. Cheap. Decisive.
2. **Do shaped notes differ compositionally from unshaped notes here?** If they do not, the corpus argument is gone too, and shape is cosmetics. **These first two together would falsify the concept, and both are measurable.**
3. **Will the owner apply a kind label at all?** The tagging evidence measures retrieval-by-tag, not application — it cannot settle this. A trial on a real library can.
4. **What does the owner's own convention space produce?** Ship an inline convention space, observe what markers he invents, and see whether they recur as search terms. That is the only evidence-backed route to a kind vocabulary.
5. **What is the prompt-fatigue threshold for graduation proposals in his actual working rhythm?** Only observable.
6. **What actually happens to a Qusasah under each real edit shape?** Six recipes, each needing to go red→green in a reproduction harness before anything ships: (i) exact-boundary deletion of the quoted span; (ii) replace-in-place; (iii) partial deletion leaving a gutted excerpt; (iv) enclosing deletion; (v) tab-switch teardown with the note closed and reopened; (vi) external rewrite via watcher/Syncthing/Git. Per Solve-the-Class and the Editor-Surface Gate, this is a content-integrity-class design and must be built whole and proven in isolation, not evolved live.
7. **Is byte-exact revert of a graduation achievable at all** once Qusasah attachments and typed living links are attached to the graduating note? **This is the gating question for whether automatic graduation is even permissible.** Linear's precedent says it is where reversibility breaks. Answer it with a prototype before designing the interaction.
8. **Does read-time reinterpretation (render-as-if-graduated) feel right?** The Elasticsearch runtime-field analogue is the most promising mechanism found and has no PKM precedent. It is a prototype question, not a research question.

---

## 9. Verification record — claims REFUTED, inverted, or materially downgraded

Durable trail. Every entry is something a track asserted and verification corrected.

| # | Original claim | What verification found |
|---|---|---|
| 1 | Length constraints "preferentially drop cognitive/reasoning words." | **REFUTED.** The paper's own text: cognitive-process words are "preserved less (**no significant differences compared to the baseline**)." The parenthetical was omitted. Reasoning words sat at baseline, not below it. |
| 2 | That LIWC result is causal / "the strongest single result in the track." | **REFUTED as causal.** Authors: "we do not study the linguistic aspects in a controlled setup." The causal experiment was about success ratings. |
| 3 | Gligorić 2019 shows constraint is hostile to reasoning, arguing against extent limits. | **ONE-SIDED READ.** The same paper's headline causal result is that shortening **improved** perceived quality up to 30–40%. Omitted. |
| 4 | "Constrained tweets have fewer hashtags." | **REFUTED.** 0.53→0.54, unbolded in every column. Not significant. |
| 5 | The control study "rules out norm drift as the cause." | **DOWNGRADED.** The one significant control difference is **contracted auxiliary verbs** — one of the six headline features, moving *more* in the unconstrained band. The authors' word "**other**" was dropped. |
| 6 | The inverted-U is "the settled synthesis position." | **REFUTED.** Acar et al. present it as **their own proposal**: "we *propose*", "*suggests*", "We are unaware of any direct empirical study on this linkage." Also: Ohly et al. 2006 reports no curvilinear time-pressure effect; Baer & Oldham 2006's inverted-U held only under a three-way interaction; Bendoly & Chao 2016 measures market performance. And Simonsohn 2018 shows quadratic-regression U-tests can hit a 100% false-positive rate. |
| 7 | "Optimally trim 10–20%; cutting capacity by half is past the harm threshold." | **DOWNGRADED and partly inverted.** The outcome measure was a **prediction of virality**, not quality ("Which one do you think will get more retweets?"). Most of the lift came from a copy-editing floor — a baseline 1–5 chars shorter beat the original in 65% of per-tweet majorities. And the **same paper's comprehension task** found information survives to ~**80% reduction**. Half is comfortably inside the comprehension range. |
| 8 | The 2020 adoption paper is "ICWSM." | **REFUTED.** Semantic Scholar venue: arXiv.org. Never published at ICWSM. The ICWSM paper is the 2018 sibling. The peer-reviewed successor is CSCW 2022. |
| 9 | "People do not expand to fill the new space" → fill-based graduation triggers fire rarely. | **INVERTED for desktop.** CSCW 2022 by client: Web cramming estimate 18.35% → actual post-switch 24.81%; re-emergent cramming 6.88%; 95%-coverage ceiling rose 275→342 chars. Mobile behaved oppositely. **The stated warrant is empirically wrong for a desktop writing surface.** |
| 10 | Surveillance, evaluation and contracted reward "reliably reduce" creativity. | **TWO OF THREE LEGS FALL.** The cited Amabile 1990 paper itself: "Coaction had no effect, and **surveillance had a weak negative effect**," attributed to experienced evaluation. Byron & Khazanchi 2012 (60 studies): **creativity-contingent rewards INCREASE creative performance.** Only expected evaluation survives intact. |
| 11 | Acar et al. corroborates a social-vs-task dichotomy. | **SELF-DEFEATING CITATION.** The quoted SDT line — "*any* external constraint reduces the perception of control" — includes task constraints, collapsing the dichotomy it was recruited to support. |
| 12 | Expertise reversal ⇒ constrain FORM, never scaffold CONTENT. | **NOT LICENSED.** Kalyuga's own qualifier was dropped: harm arises "**especially if these learners can not ignore or otherwise avoid processing**" the material. The distinction the literature licenses is **avoidable-and-fading vs mandatory-and-permanent**, not form vs content. Kalyuga's actual prescription is *adaptive, dynamically faded guidance keyed to measured expertise* — citing him for a permanent assumed-expert stance cites him against himself. The meta-analysis also reports the effect as **asymmetric**: over-support (+0.505 for novices) is the **cheaper error** than under-support (−0.428). Also: author order mis-cited (Kalyuga, Ayres, Chandler & Sweller — not Sweller first). |
| 13 | Preparatory organisation is a small minority of how people retrieve **their own information**. | **MEDIUM OVER-GENERALISED.** True for email. The **file** literature — same lead author — reports the opposite: navigation 56–68% vs search 4–15%; 94% navigation success; an fMRI account of why. Constellation's medium is files. Also "opportunistic" ≠ "scanning": the 87% includes ~22% **search**. |
| 14 | Tagging is near-vestigial "in real use." | **MEASURE MISMATCH.** The logged variable is "whenever a user clicked on a tag." The paper reports **no count of tags created**. It cannot distinguish "nobody tagged" from "nobody navigated by tag." Also confounded: users arrived with legacy folder trees and **zero legacy tags** over a 64-day mean window. |
| 15 | Effortful organisation at capture "costs time." | **WRONG CLOCK.** t(357)=6.71 measures **retrieval-side** folder-access time. The study never measured capture-side filing cost. Also: high filers used **fewer** operations (3.69 vs 4.16, p<.05) — organisation produced a real efficiency gain, cancelled by slow folder navigation in that client's UI. Both groups succeeded 88% of the time. And the paper's own explanation was dropped: people file for **inbox rationalisation and task management**, not retrieval. |
| 16 | Threading was "the one preparatory structure that improved retrieval." | **DOUBLY WRONG.** The paper's own taxonomy calls threading **opportunistic**, i.e. explicitly NOT preparatory. And "the one" is false — the same abstract sentence says "**both search and threading** promote more effective finding," with search also improving efficiency. Effect is 6pp, p just under .05, correlational, and threading proportion is a property of one's incoming mail, not a chosen behaviour. Its use as "the evidential basis for shape graduation" has **zero** connection to the paper. |
| 17 | Curation artefacts "don't improve retrieval." | **CONTRADICTED by the same lead author's later work.** Bergman & Shnaper-Reinberg, *Journal of Documentation* 2025: actively stored recipes (explicitly including bookmarks) had **3% retrieval failures vs 15%** passive, and were **36% faster**. "The very activity of organizing the information improves retrieval." The measured harm was **fragmentation** (errors correlated with number of storage locations, r=0.34), not classification. |
| 18 | Real notes are already tiny "without any imposed extent constraint." | **FALSE AS STATED.** list.it imposed extent affordances by design: a single input box with **Enter-to-commit**, making a line break a non-default gesture. "80% contained no line breaks" is partly a UI artifact. Participants were also **paid per note**, biasing toward volume/brevity. |
| 19 | 75% of notes never edited ⇒ users don't revise. | **INVERTED.** ≤10-day window with censoring. The same group's in-the-wild TOIS 2008 study: "Annotation and revision were also **quite common**," with "Work-In-Progress" a top-tier scrap role — revision occurs via **transfer to a richer tool**, which list.it structurally could not observe (no promotion target). Also: appending was markedly more common than revising; 28% were deleted. |
| 20 | Capacities' docs "warn AGAINST creating types eagerly." | **REFUTED as vendor posture.** The page is neutral disambiguation and resolves **positively** ("a new object type is likely the right choice"). The sibling page opens "Creating your own object types is a **great way** to adapt Capacities." The product ships a "+ new type" button, a template gallery, and a Discord showcase channel, with **no documented limit** on type count. Capacities is a precedent for **deferred authoring**, not for a closed catalogue. The failure mode the page addresses is over-**splitting one entity**, not type proliferation. |
| 21 | Capacities' conversion dialog is a precedent for **proposing** graduation. | **NOT LICENSED.** Capacities has no automatic or proposed type change at all. Absence of silent migration in a product that never migrates is not evidence about designing an automatic promotion. It licenses "when the user converts, make the data landing explicit" — nothing more. Also: the **bulk** path has **no** mapping surface, and basic types (PDF/image/audio) cannot be converted. A better pattern exists in the same docs: property conversion **keeps the original alongside** until the user deletes it. |
| 22 | Anytype users delete-and-recreate, losing creation dates and links. | **REFUTED.** That was the poster's **initial mistaken premise**, retracted by him 46 minutes later in the same thread ("OK, I found the menu… Good news 👍") and marked superseded by his own edit. Nobody lost a creation date. The real failure is **discoverability** — including a **lock icon** on a field mutable elsewhere. Anytype had already solved graduation the right way. |
| 23 | Tana's docs "do not say what happens to field data." | **TOO ABSOLUTE.** Tana publishes a non-destruction principle — "will never delete data that is indirectly associated with another element that you delete" — for **definition deletion**, and is silent only on the **untag** case. A stronger, missed finding exists: field values can only be bulk-deleted when "**not part of a supertag template**" — a documented, asymmetric tag↔data coupling. Also, "users get burned by this" is **unevidenced**; searches found no such reports. |
| 24 | Tana's reception is "bimodal." | **UNEVIDENCED.** Traces to one unnamed, undated aggregator sentence, unreplicated. The only actual distributions found are unimodal-positive (Product Hunt 4.8/5 across 55; App Store 4.0/5 across 48), both survivor-selected. **The steep learning curve IS confirmed** on better sources than the one cited. And "unintuitive" is a **phase, not a property** — Mark McElroy found supertags "remarkably straightforward and much easier for non-programmers" than Obsidian's Dataview. |
| 25 | Drafts "always opens to an empty textbox." | **FALSE.** Configurable "New Draft After" timeout (30s–1hr, or Never), pinning, and Focus Mode all return to the last-edited draft. Opt-in tag-at-creation also exists. The correct invariant is narrower: **no path into capture may require a classification decision.** |
| 26 | `MapMode.TrackDel` returning null is the authoritative orphan signal for Qusasah. | **EMPIRICALLY FALSIFIED against the shipped library.** `mapMode` is only consulted when `from == to`; `RangeSet.map` never returns null; deleting exactly the quoted span leaves a surviving 5–5 range with `TrackDel` returning 5. Replace-in-place silently re-anchors onto unrelated text with zero signal. The prescribed trigger **can never fire**. |
| 27 | Hypothes.is tries four strategies in order, context-first then exact-only. | **STALE ARCHITECTURE.** Shipping code has **three** tiers; blog strategies 3 and 4 were merged, and the role of context **inverted** — candidates come from the exact quote by edit distance, and prefix/suffix/position only **rank** them (50/20/20/2). Building the blog's version would ship an architecture Hypothesis retired. Also missed: `maybeAssertQuote` cross-validation, and issue #3919's 10+ second blocking. |
| 28 | "Peer-reviewed study of 20,953 annotations found 22% orphaned, 12% recoverable." | **PROVENANCE WRONG, AND THE RESEARCHER'S OWN 'CORRECTION' WAS THE ERROR.** The peer-reviewed paper (TPDL 2015) measured **6,281** annotations and reported **27% / 3.5% / 61%**. The 22%/12%/53% figures are from the **unrefereed extended arXiv preprint**. A search summary reporting 27%/3.5% was dismissed as noise; it was correct. Also: attachment was tested by **exact string match**, so the rate is an **upper bound**; and "**53% in danger**" measures **missing archival coverage**, not anchor fragility. The paper's own conclusion — archive the target at creation time — was omitted. |
| 29 | Visible orphan state "IS the industry norm." | **UNSUPPORTED.** One blog post from one niche product. Counter-evidence points the other way: Google Docs' `anchor` field is stored but ignored by the editors ("Original content deleted"); Zotero has open reports of annotations vanishing with no orphan surface. Also, the "after user harm" causal story is the **vendor's own launch narrative**, uncorroborated. And two official Hypothesis sources **contradict each other** on the pre-tab state. |
| 30 | GitHub's `position`/`original_position` exemplifies "keep both coordinates forever." | **INVERTED EXAMPLE.** That is the one pair GitHub is **deleting** — "We are phasing out diff-relative positioning for PR comments." The surviving pairs are file-absolute or immutable (`line`, `commit_id`). The real lesson is narrower: keep both **in a coordinate system that is not itself derived**. Also, "`diff_hunk` is frozen at attach time" is **inference, not documentation**. |
| 31 | Elasticsearch "refuses" in-place type change; the pattern includes "keep the old index for rollback." | **BOTH OVERSTATED.** **Runtime fields** (7.11+) exist precisely for zero-rebuild, instantly-reversible read-time type reinterpretation. And Elastic's canonical procedure ends "**delete the old index**" — ES exposes no revert. Retain-for-rollback is practitioner convention presented as sourced. Also, the catch-up problem (writes landing on the old index during reindex) was omitted. Also: "The Great Mapping Refactoring" is a 2015 post about conflicting mappings **across document types**, not the migration remedy. |
| 32 | Mapping explosion argues for a small hand-written closed shape set. | **DIRECTION REVERSED, AND SCALE INAPPLICABLE.** Elastic's field limit counts fields "created **manually or dynamically**" — explicit mappings explode identically. The field-tested pattern is **bounded dynamism**: open inference over inputs with a constrained output type space via dynamic templates, plus a cardinality cap and graceful degradation. And the pathology is a cluster-state/JVM-heap problem engaging at thousands of fields; Constellation's five dimensions sit 3–4 orders of magnitude below it, so the citation cannot discriminate between design options. Elastic's own issue #89911 argues the default limit is both too restrictive and ineffective. |
| 33 | "schemaVersion **plus** lazy migration" is the standard document-DB pattern. | **CONJUNCTION NOT SOURCED.** The official docs describe versioning with **no migration at all** — coexistence is the point. MongoDB's blog presents migration as a free choice of three. Also, lazy migration fires on **access — primarily read**; the "ride an existing user write, never a read" rule is a **Constellation-specific tightening from File Over App**, not inherited. And "the standard" is one vendor's pattern catalogue; the academic literature treats eager migration as the mainstream default. |
| 34 | Gradual-typing bugs concentrate at the boundary "because developers assume the annotation is enforced." | **THE MECHANISM IS THE PAPER'S UNCONFIRMED HYPOTHESIS.** Its own RQ2: "we do **not** observe a clear trend indicating that type annotations affect a parameter's likelihood of being type-checked." Annotated, unannotated, and untyped-baseline params are all checked ~2.5%. In Python the direction **flips** (annotated 2.28% vs unannotated 1.82%). The authors defer the hypothesis to "future work." Also: the study counts **type checks**, not bugs — it has no bug corpus. Also: author mis-cited as "Chen, Staicu et al."; the authors are **Troppmann, Fass & Staicu**. |
| 35 | Accidental conformance ⇒ structural evidence may not classify. | **NARROWED.** What makes it harmful is **silent substitutability at a boundary**. The literature supports "a structural match must never silently confer identity-based permissions" — a shown, confirmable inference is not the thing it warns about. Also, the nominal side's own failure was omitted: Malayeri & Aldrich (ESOP 2009) found 98.6% of inferable parameters were declared with an **overly specific nominal type** — pure declaration systematically **under-classifies**. |
| 36 | Add a "nominal marker" (branded type) — the standard mitigation. | **WRONG MECHANISM, AND NOT STANDARD.** *Learning TypeScript*: "Branded types aren't used in most TypeScript projects"; the brand "is just in the type system" — "a useful lie." It has **no runtime existence** and cannot support a persisted YAML field. The correct citation is the **discriminated (tagged) union**, which is real at runtime and is officially documented mainstream practice. Also: the quoted sentence is **not in the Wikipedia article it implies** — Wikipedia's only named mitigation is "one algebraic data type for each use." Critically, a plain-text frontmatter field is **forgeable and can go stale** — it is a hint with provenance, not a nominal marker. |
| 37 | "Reduces task completion time 20–40%" (progressive disclosure, attributed to NN/g). | **NOT IN THE SOURCE.** The NN/g article makes essentially no quantitative claims. Caught live during the research pass. Its one concrete rule: no more than **2 disclosure levels**. |
| 38 | The CSCW 2018 sticky-note study shows digital loses nothing / is "most-cited." | **BOTH WRONG.** The paper documents an author-asserted loss at capture ("stylus and posting issues"; digital creation "requires additional attention"), the physical condition was deliberately handicapped by design, the digital deficit is partly a **hardware artifact** (incompatible passive stylus, cross-device tap-to-post), the "more interaction" advantage includes moves the authors coded as "**redundant**," n=7 pairs with no power analysis, and OpenAlex gives 43 citations with an **influentialCitationCount of zero** (Harboe & Huang 2015 has 167). |
| 39 | Bullet Journal migration's "**entire** cognitive value" is the friction, and the page gives no guidance on automation. | **BOTH CORRECTED.** The method's chain is effort → pause → **consider** → decide; friction is the forcing function, not the end. And guidance exists: Carroll's own Companion app **does not auto-migrate and does not reproduce the rewriting friction** — it substitutes 72-hour expiry. The mechanism is a **forced decision point**, and manufactured tedium is not supported. |
| 40 | Haught-Tromp 2017 supports extent limits. | **WRONG MANIPULATION.** Its constraint is **additive** (include this given noun) — a generative seed, not a subtractive extent cap. The author's own limits: "the carryover effect… is specific to the rhyming task"; "The present two experiments used a **binary** design." And ηp²=.53 from one lab with no located replication should be treated with suspicion. |

---

*Compiled 2026-07-20. Every load-bearing claim above carries its grade. Where a track's inference exceeded its source, the source won. Where two tracks disagreed, the disagreement is recorded rather than averaged. Where nothing was established, the entry reads [unknown] and nothing was invented to fill it.*
