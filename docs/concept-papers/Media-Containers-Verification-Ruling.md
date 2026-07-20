## Verdict table

| Claim | Verdict | May it drive design? | One-line reason |
|---|---|---|---|
| **C1 — Accursian legal page as a secular fixed frame** | PARTLY_CONFIRMED | **YES-WITH-CAVEAT** (re-described) | The frame page is real, but three of the claim's four specifics (centred column, interlinear zone, fixed geometry) are wrong, and its "secular invention" framing collapses — the named category's flagship illustration is a biblical commentary. |
| **C2 — lemma/quote anchoring beats positional anchoring** | PARTLY_CONFIRMED | **NO as written** (YES for the layered model it actually supports) | Hypothes.is tries quote *last*, not first; every historical success of lemma anchoring occurred over a frozen canonical text, which is the opposite of Constellation's regime. |
| **C3 — heading-at-capture is the one discriminator, and is what makes a note-heap a work** | PARTLY_CONFIRMED | **NO for the causal claim** (YES-WITH-CAVEAT for retrievability) | Heading is the genre's definitional criterion, but no historian claims it causes publishability, and the claim's own headline example (Lichtenberg) refutes it. |
| **C4 — annotation block written first, base text fitted around it** | REFUTED | **NO** | No source attests the ordering; the one secular manuscript with decisive internal evidence shows the host complete and the annotation abandoned — host first. |

---

## What must change in the synthesis document

### Scope-ruling violations — flag these first

**All four claims rest, in whole or in part, on a religious evidence base while being presented as secular design evidence.** This is not four accuracy quibbles; it is one systemic failure of the exclusion the owner set.

- **C1**: *textus inclusus* is defined genre-neutrally and its standard public exposition (Kwakkel) illustrates it with **Peter Lombard's *Glossa in epistolas Pauli*** — a biblical commentary — where the base text is a single centred column. The claim's "centred base text + interlinear zone" is the **glossed Bible's** architecture wearing a Roman-law label. Independence of the legal page from the glossed Bible is unestablished, and the chronology mildly favours the religious book arriving first as a published format.
- **C2**: the comparative evidence base is seventeen fragmentary Iliad commentaries plus the **Qumran pesharim** — scripture commentary outright, plus a functionally canonical Homer. Later links in the alleged chain (patristic catenae, biblical apparatus criticus, *matn-wa-sharḥ*) are the same shape.
- **C3**: florilegia are patristic/monastic; the flagship *Loci communes* is **Melanchthon's Lutheran dogmatics**; the manual tradition rests on the Jesuits Sacchini and Drexel; Locke's own worked example head is *Confessio*.
- **C4**: the entire mature-glossed-codex evidence base is the **Glossa ordinaria** — the Bible. The one secular manuscript checked *inverted* the finding.

Required document text:

> **Scope note.** The manuscript-glossing evidence assembled here is religious-corpus-dominant. Where a secular check was possible it either weakened or reversed the conclusion. No design recommendation in this document should be read as carrying secular-tradition warrant.

### C1 — exact corrections

Delete: *"a centred base text with interlinear glosses between widely-spaced lines, in a fixed frame, invented independently in a secular tradition c. 1230s."*

Replace with:

> The 13th-century Bolognese law book used *textus inclusus*: two dense central text columns in large script, enclosed on all four sides by a smaller-script commentary, with deliberate blank corridors between the columns and between text and gloss. The apparatus was pre-installed by the scribe and sold that way. The **geometry was not fixed** — text-block length is described as irregular, and the recognised achievement of the Venetian printer Baptista de Tortis was solving the problem of making a page's text and its gloss end together, a problem that exists only because both masses reflow. Interlinear glossing occurs but is a reader-added layer, not a ruled zone. Compiled at Bologna between the 1220s and the mid-13th century; completion contested (1228 to c. 1258). Roughly 100,000 glosses; around 1,200 surviving manuscripts. **Whether this page originated independently of the glossed Bible is unresolved; the architecture's own name is shared with religious books.**

Anything resting on "give the annotation a stable geometric frame" has **no** historical warrant. Anything resting on "an authored apparatus surrounds an untouchable core, separated by whitespace gutters, at ~2:1 annotation-to-text line density" is supported.

The better-sourced contrast to keep is not secular-vs-religious but **pre-installed apparatus vs. prepared-blank scaffolding** (law books ship with the gloss; glossed-Aristotle books ship with empty ruled annotation columns the reader fills). Note that this contrast traces to a single scholar (Kwakkel) and is uncorroborated.

### C2 — exact corrections

Both sentences are false as written and must go:

- ✗ *"The W3C spec designates TextQuoteSelector as the durable selector."* The spec ranks nothing — though it **does** say TextPositionSelector is "very brittle with regards to changes to the resource… unlike the Text Quote Selector," so it leans that way on this one axis. State that precisely, not as a ranking.
- ✗ *"Hypothes.is anchors on quoted text rather than character offsets."* The client tries Range → Position → Quote → MediaTime, advancing only on failure. Quote is the **validator** (`maybeAssertQuote`) and the recovery path.
- ✗ The 32-character prefix/suffix window is **not in the spec**. It is a Hypothes.is house constant.
- ✗ *"in the absence of line or paragraph numeration, as was the case in antiquity."* Antiquity had stichometry — Homeric papyri, the Bankes Homer among them, are numbered by hundreds in the margin. What was missing was a *standardised citation system*, not positional marking.

Replace the design rationale with:

> Every durable tradition ran a **positional address alongside a quoted copy** — critical sign plus lemma in the two-roll Alexandrian system; line number plus lemma in the apparatus criticus; Range plus Position plus Quote in Hypothes.is. Keep the pairing argument; drop the supremacy argument. Note further that every historical success of lemma anchoring occurred over a text that was not permitted to change. Constellation's notes are mutable and the annotated passage is the one most likely to be edited next.

### C3 — exact corrections

Delete the generalisation entirely: *"structure applied at write time is what converts a jotting-heap into a work."* No historian argues it, and the Sudelbücher — unheaded, untransferred, canonical — refute it on the claim's own turf. Lichtenberg's leading editor (Joost) holds that the Sudelbücher *became* his ledger and that his waste-book vocabulary was ironic understatement.

Also cut *"waste book = not the record of record."* Archival sources call the waste book the book of original entry, interchangeable with daybook and journal; the "discarded, hence the name" etymology traces only to Wikipedia and content-marketing blogs.

Replace with:

> Attaching a head or keyword to an entry is what makes a heap **retrievable**. Locke's genuine innovation was the **index** — a retrieval device — not the filing scheme; his heads were emergent, chosen during note-taking. Second-order commonplacing (capture loose, apply heads at transfer) was a real and used option. And the surviving manuscripts show the heading discipline being abandoned by most compilers, including Locke and including the authors of the manuals: "sooner or later he gave up, leaving many if not most pages almost or indeed entirely empty."

Design consequence, quote-ready: **do not gate capture on a head.** A mandatory write-time classification UI reproduces the documented historical failure mode — enthusiastic start, mostly empty book. Cheap capture plus strong automatic retrieval is the pattern the evidence supports, and it is already Constellation's FTS5-trigger shape.

### C4 — exact corrections

Strike the ordering claim and the "small fraction" figure **entirely**. Replace with:

> Base text and gloss were copied together in one planned operation from a combined exemplar, on a **single shared ruling grid** laid down before writing. The base text occupied roughly 30–40% of the written area in the measured examples, varying page to page and sometimes inverting. Where evidence of ordering survives in a secular manuscript, it runs the other way: a Justinian *Institutiones* with a complete text and a gloss that stops abruptly at f. 14.

The rule "shrink the host, never the annotation" loses its precedent and must stand on UX merit. Two substantive design points:

1. Parchment was zero-sum; a scrolling viewport is not. The medieval remedy for an overgrown gloss was to **enlarge the surface** — the move the page could not make and we can. Defaulting to "shrink the host" imports a constraint we don't have.
2. The historically accurate elastic rule is **reflow, not suppress**: the host's column width narrowed under annotation pressure, but no host content was ever truncated, collapsed, or scrolled out of reach.

---

## What is now solid enough to build on

**High confidence:**
- Hypothes.is's real architecture — position/structure first as a fast path, quote as validator and recovery, orphans as a visible first-class state. The `maybeAssertQuote` validator pattern is the single most transferable artefact in the verification, and it maps directly onto Constellation's watcher-adopt path (PJ-070).
- Quote anchoring's measured failure profile on read-only hosts: ~27% orphaned, 61% at risk, ~3.5% archive-recoverable. That is the honest prior; we should beat it because we own the edit stream, but the orphan state must exist and be visible.
- Quote anchoring cannot survive edits to the quoted text itself. Position mapping through the transaction stream can. This is decisive for an editable host.
- W3C TextQuoteSelector's normative rules on Unicode code points and grapheme clusters — load-bearing for Arabic combining marks and CJK.

**Medium confidence:**
- *Textus inclusus* as a real, named, pre-installed enclosing-frame architecture, and the shared single ruling grid for host and annotation. Both are attested; both are single-source or dealer-catalogue-sourced. Good enough for a shared-layout-engine argument, not for a fixed-frame argument.
- Heading as the definitional criterion of the commonplace-book *label*, with strong prescription-vs-practice evidence that the discipline was widely abandoned in the actual manuscripts.

**Nothing in C4 is solid enough to drive design.**

---

## What remains genuinely unknown

**(a) Checked; the scholarship is genuinely divided.**
- The completion date of the Accursian Glossa: Treccani says probably 1228, Wikipedia c. 1250, another source c. 1258. There is no modern critical edition to settle it. "c. 1230s" is defensible; declaring c. 1250 consensus was itself an overstatement.
- The commonplace-book genre boundary: Arrighetti's broad definition vs. narrower ones is a live classification dispute, not settled fact.
- Whether the waste book was the evidentially privileged record of original entry or a throwaway. Archival glossaries and the popular etymology point opposite ways; the OED entry is paywalled and unverified.

**(b) Checked; could not find sources.**
- **Whether the legal glossed page developed independently of the glossed Bible.** No named scholar asserts either direction. What surfaced twice were automated search summaries confidently asserting *opposite* directions with no traceable authority — precisely the fabrication class this verification exists to catch. Treat this as a red flag, not a neutral null.
- Independent corroboration of the glossed-Aristotle "prepared-blank" contrast. One specialist, uncontested and unconfirmed.
- Whether the gloss block was ever laid down before the base text (C4's core). No source found in either direction; the only decisive internal evidence found points against.

**(c) Needs a specialist we do not have.**
- L'Engle & Gibbs, *Illuminating the Law* — the standard work on Bolognese legal mise-en-page. No online text.
- de Hamel, *Glossed Books of the Bible* (1984), ch. 2–3, and Smith, *The Glossa Ordinaria* (2009), ch. 3 "Layout" — the two measured studies. Paywalled and lending-restricted. **A reader of de Hamel could still overturn the C4 refutation.**
- The full text of *"A Medieval Puzzle: The 'Architecture' of the Page…"* (403) — would settle fixed-vs-variable geometry in detail.
- Soetermeer, *Utrumque ius in peciis*.

A codicologist with library access could close (b) and (c) in a day. Nothing in (a) is closeable at all.

---

## The single most important thing the owner should know

The verification did not merely correct details — it **removed the historical warrant from the design recommendations, and in two cases showed the precedent pointing the other way.** C4 is refuted outright. C2's recommendation is inverted: quote-first anchoring is what you do when you re-encounter a document you did not watch change, and every historical instance of it worked because the base text was forbidden to change — the opposite of a personal note. C3's "heading at capture makes it a work" is contradicted by its own headline example. And the systemic finding is that all four claims leaned on a religious evidence base while being presented as secular, which is a scope-ruling violation, not an accuracy quibble. The pattern across all four is identical: a plausible *shape* recalled from general knowledge, with specifics back-filled confidently and wrongly — the exact failure mode the BASIC RULE names. Qusasah's design should now be justified on its own engineering and UX merits; the manuscript history is a source of vocabulary and one genuinely useful pattern (positional address paired with a stored quoted verifier, with visible orphaning on mismatch), and nothing more.
