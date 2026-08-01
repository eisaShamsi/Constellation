# 33 — Search as Aerial View (Concept Paper)

> The missing horse. `06-search-hub.md` describes the Search Hub's *mechanics* — six result
> groups, two query modes, the operator vocabulary. It never states the concept those mechanics
> serve, and the **Semantic** group was built with no concept paper at all (a
> Concept-Before-Function breach, `00-MASTER` line 3). This paper states the concept, in the
> Boss's words, and derives what follows from it.
>
> **Status:** Boss-stated 2026-08-01. Supersedes nothing; it *governs* `06-search-hub.md` §2
> and is the authority for any future change to the Semantic group, the lexical bridge, or the
> cross-language path.

---

## 1. The concept (the horse)

Boss, 2026-08-01, verbatim:

> "It is all about Constellation, as a unique PKF. If a user searches for something, it will
> help them find **every note that matches their search query, regardless of the language of
> the related notes**. So, in a way, it will help the user get an **aerial view of their
> knowledge (universe)**."

Stated as one sentence:

> **Search shows you everything you know about a thing — every note that matches, whatever
> language you happened to write it in — so that you see your own knowledge from above,
> whole.**

## 2. What the concept actually claims

Three claims, each load-bearing. A change that breaks any one of them breaks the concept.

**2.1 — Completeness is the goal, not precision.** An aerial view that shows two rooftops is
not an aerial view. The user is not asking "give me the single best match"; they are asking
"show me my terrain." Where completeness and tidiness conflict, **completeness wins**, and the
display cap — not a quality bar — is what bounds the list.

**2.2 — Language is an accident of writing, not a boundary of knowing.** The user thought about
knowledge in Arabic on Tuesday and in English on Thursday. That is **one** body of knowledge
that happens to be recorded in two scripts. A search that returns only the Tuesday notes has
not answered the question; it has reported an artefact of how the notes were typed. This is
what `lexicon/mod.rs` already calls *"a defining feature"* that "Obsidian, Roam, Logseq do not
offer."

**2.3 — The view must be legible.** "From above" implies the user can *see the shape* of what
came back — where it lives, what language it is in, and **why it was included**. A result the
system cannot explain is a dot on a map with no legend. This is the Constellation Way applied
to search: propose from the user's own evidence, and **show the evidence**.

## 3. The three mechanisms, and the one question each answers

The concept is served by three mechanisms that have historically been confused with one
another. They are not competitors; they are three ways a note can *belong* to a query.

| Mechanism | The question it answers | Cross-language | Can it explain itself? |
|---|---|---|---|
| **Literal** (Titles / Contents / Tags / Properties / Wikilinks) | "The word is here." | no — by definition | yes: the word is highlighted |
| **Lexical bridge** (`via {lemma}`) | "The *concept* is here, said in another word or another language." | **yes, by construction** | **yes**: `via معرفة → knowledge` |
| **Semantic** (vectors) | "This note *belongs* here, though it never names the thing." | **no** — measured at +0.004…+0.019 on the live universe | **no**: only a similarity number |

**The division that matters:** the **bridge** is what delivers claim 2.2. The **Semantic** group
delivers the residue — notes that belong to the topic without using any word the dictionary
knows. Semantic is *not* the cross-language mechanism and must never again be asked to be one;
the measurement says it cannot, at any threshold, including none.

## 4. What follows — the consequences the concept forces

These are derived from §2, not chosen. Each names the code that contradicts it today.

**4.1 — A quality bar set by the best result contradicts 2.1.**
`search.rs:11097` keeps only results within `0.03` of the top score, so the *better* the best
match, the *fewer* results the user sees. On the Boss's universe, searching `المعرفة` returned
**2 of 7,750** because he owns a note with that title. That is a keyhole, and the concept
demands an aerial view. Whatever replaces it must be **absolute** — a floor the winner cannot
move — with the display cap doing the bounding. *(The number itself is still a Boss ruling; the
shape is not.)*

**4.2 — Cross-language must be on the path people actually use.**
The bridge is wired only to the operator path and the Index panel. A user typing a bare word —
the ordinary case — never reaches it. A defining feature reachable only by operator syntax is
not delivered. Claim 2.2 is unmet for the default route.

**4.3 — The index and the dictionary must agree on what a word IS.**
*(Corrected 2026-08-01 after cross-checking the Search Hub against the Index panel at the
Boss's direction. The first draft of this clause said "strip the definite article." That is
true but far too small — it names one wall and there are two.)*

The two surfaces feed the dictionary **different kinds of string, and neither is the kind the
dictionary holds**:

| Surface | What it hands the dictionary | Outcome |
|---|---|---|
| Search Hub | the RAW text typed — `المعرفة` | **miss** — nothing strips `ال` |
| Index panel | a term from the FTS vocabulary — `معرف` | **miss** — that is a *stem*, not a dictionary word |

Measured on the live universe: the vocabulary holds **`معرف`** (448 docs) and contains neither
`معرفة` nor `المعرفة`. The dictionary holds **`معرفة`** (row 9737) and does not contain `معرف`.
**One side stems Arabic, the other stores dictionary-forms, and for Arabic they can never
meet** — not for this word, not for any word whose stem differs from its lemma. The Index panel
does not rescue the Hub; it fails at the same junction from the opposite side.

**This is why the feature tested clean and still failed.** English stems to itself, so
`knowledge` is in the vocabulary (1,937 docs) *and* in the dictionary, and the M12 example
`tree → شجرة` works. The capability is therefore **half-built along a language axis**:

- **English → Arabic bridges** (`knowledge` → `علم`, 778 docs in this universe).
- **Arabic → English cannot**, because the typed form never matches a dictionary key.

Under claim 2.2 this is the concept failing at the first Arabic word, invisibly, while passing
every English test. The remedy is not a query-side patch — it is a **normalisation contract**
between the indexer, the query parser and the corpus: one agreed answer to "what is the
dictionary form of this word," honoured by all three.

*Corpus note:* 2,656 of 21,236 Arabic dictionary entries (12.5%) are themselves stored **with**
`ال` (`الغفران`, `التجريد`, `الخلافة العباسية`) while the rest are bare. So even the dictionary
has no single policy, and today it is arbitrary per word whether a definite-form query could
ever hit. The contract must cover the corpus, not only the code.

*The answer is present and unreachable:* every `c:knowledge` bridge term is live in this
universe — knowledge 1,937 · علم 778 · cognition 19 · Wissen 16 · connaissance 13 · savoir 12.
The aerial view exists in the data. Only the lookup fails.

**4.4 — Silent under-delivery is the worst failure mode.**
`FTS-Health-Forensics-2026-06-23` §A.2 found the Index panel's bridge filtering against a table
that is ~92% empty: *"valid suggestions are silently dropped."* Under 2.1 this is not a
performance note — it is the concept failing quietly, which is worse than failing loudly,
because the user cannot tell an empty terrain from an unmapped one.

**4.4b — The Arabic vocabulary itself contains fused tokens.**
Found while cross-checking 4.3, not investigated further: the FTS vocabulary holds run-together
entries — `معرفغرض` (36 docs), `معرفعلم` (25), `علممعرف` (18), `دارمعرف` (16), `معرفقراء` (14).
Each is two Arabic words fused into one token. This is upstream of everything in this paper: it
corrupts the Index panel's vocabulary view *and* silently costs content-search recall, because
text indexed under a fused token cannot be found by either of its real words. It is a
**tokenisation** defect, not a bridge defect, and it is filed separately (PJ-197) rather than
folded in — but no measurement of "completeness" under claim 2.1 is trustworthy until it is
understood.

**4.5 — Every included result owes a reason (2.3).**
The `via {lemma}` badge is **not decoration** — it is how the bridge discharges 2.3, and it is
currently painted in a view where it can never appear (all six universal constructors set it to
`None`). Conversely, the Semantic group's inability to explain itself is a genuine tension with
2.3 and must be resolved deliberately: either it earns a legible reason, or it is presented
honestly as "related, unexplained."

## 5. What this concept does NOT license

- **Not a dump.** "Every note that matches" is bounded by *matching*, not by lowering the bar
  until everything matches. A floor that admits the entire library is as useless as one that
  admits two — it just fails in the opposite direction.
- **Not displacing the literal.** MIG-057's standing ruling binds: expansion is **additive**.
  The Boss: *"By getting the right term I am searching for, not only the semantic terms."* The
  word the user typed always ranks and always shows; bridged and semantic results are added
  **around** it, never in place of it.
- **Not silent cleverness.** Under 2.3, an expansion the user cannot see the reason for is a
  regression even if its results are good.

## 6. Acceptance — how we will know the concept is served

1. A bare query in **any** of the 15 languages returns matching notes written in the **other**
   languages, on the **default** route, with a visible reason on each bridged row.
2. `المعرفة` — the design's own worked example — returns the `c:knowledge` family: Arabic
   `معرفة/علم`, English `knowledge/cognition`, French `connaissance/savoir`, German
   `Wissen/Erkenntnis`.
3. A strong exact match **increases** what the user sees; it never reduces it.
4. Nothing is dropped silently: where a mechanism cannot answer, the surface says so.
5. The literal hit for the typed word is always present and never outranked out of view.

## 7. Open — Boss rulings still owed

1. **The Semantic group's standing.** Peer group (always shown), or fallback (speaks only when
   the bridge is silent)? §3 argues it answers a real, distinct question; its standing is still
   a decision.
2. **The bound.** With the relative bar gone, what bounds the Semantic list — the display cap
   alone, or an absolute floor as well, and at what value?
3. **Default-on cross-language in the Hub.** The 2026-05-04 ruling made the *Index panel's*
   bridge off-by-default to protect literal browsing. Does the same caution apply to the Hub,
   or does 2.2 make it default-on there?
4. **The Arabic article policy** for the corpus — normalise the 12.5% that carry `ال`, or
   handle both forms at lookup, or both.

## 8. Status

Concept: **stated by the Boss, 2026-08-01.** Implementation: **not started** — this paper
precedes the Architect phase, per Concept-Before-Function. The defects in §4 are not to be
patched individually; they are one concern (`4.1`–`4.3` especially) and belong in one
`/migration` whose Architect doc opens with §1 of this paper.

**Allocated: `MIG-109` — "Search as Aerial View".** Boss-directed 2026-08-01: *"we will allocate
an MIG for it, to be dealt with in the right time."* The number is reserved; the migration is
**not** scheduled and must not be started ahead of the Boss's word. Its Architect phase opens
with §1 of this paper and must answer the four rulings in §7 before any design is chosen.

Filed alongside it (see the Pending Jobs ledger):

| PJ | What |
|---|---|
| **PJ-196** | The Semantic group's relative cutoff (§4.1) — the collapse the Boss saw. In MIG-109's scope. |
| **PJ-197** | Arabic fused tokens in the FTS vocabulary (§4.4b) — upstream of MIG-109, own investigation. |
| **PJ-198** | `notes_vocab` under-delivery: the `≈ similar` lookup table is ~92% empty (§4.4). Discovered 2026-06-23, recommended, never built, **never filed until now**. |
| **PJ-199** | The `via {lemma}` badge is painted in a view where it can never appear (§4.5). Cosmetic today; evidence the default route was *intended* to bridge. |
