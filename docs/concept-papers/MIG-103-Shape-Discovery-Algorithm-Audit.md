# MIG-103 — Shape Discovery Algorithm Audit

**Function in hand:** Template Studio / Shape Discovery — the routine in `src-tauri/src/template_discovery.rs` that reads the Universe's frontmatter and proposes recurring note shapes ("kinds") the user can adopt as templates.

**Concept (the horse):** *Recognition, not prediction.* The app tells the user what they have demonstrably already written, and shows the notes that prove it. The user names it and decides whether it becomes a template.

**Date:** 2026-07-21 · **Status:** research synthesis, pre-design · **Audience:** the project owner (Part I) and the engineer who will harden the algorithm (Parts II–VIII).

---

# PART I — THE VERDICT IN PLAIN LANGUAGE

## Has anyone solved this before?

**No PKM tool has. But two other fields have, and they agree with each other.**

Nothing in the note-taking world does what Template Studio does. Across every tool we audited — Obsidian core, Obsidian Bases, Metadata Menu, Note Type, Obsidian Schema, the database plugin, Notion, Tana, Logseq — **eleven out of eleven ways of giving a note a "type" require the user to declare it**, either directly or by writing a rule (a folder rule, a tag rule, a query). Zero of them read the corpus and propose a type the user hadn't already thought of. The one shipped "suggest fields" feature in the whole category (Tana) suggests from an AI model's general knowledge of what a "Person" usually has — not from your own notes. [sourced]

*(One correction to that headline, from verification: Obsidian is not inert. It does guess the **value type** of each property key across the whole vault, and it autocompletes property **names** you've used before. What it never does — what nobody does — is look at which keys travel **together** and propose that combination as a kind. Obsidian's unit of analysis is the key. Ours is the key set. That is the honest differentiator, and it survives an Obsidian user checking it in ten seconds.)*

Two adjacent fields **have** solved it, at enormous scale:

1. **JSON/NoSQL schema inference** — the problem of "here are ten million records with no declared schema; what shapes are in here?" Published, peer-reviewed, with shipping implementations.
2. **Wikipedia infoboxes / Wikidata / DBpedia** — the largest real instance in the world of "articles of a kind share a schema," with ~376,000 person articles and ~165,000 film articles to measure against.

Both arrive at the same answer, and it is not the answer we implemented.

## The single most important change

**Stop treating a discovered kind as a *set of property keys*. Treat it as a *list of keys with a count against each one*.**

That one change is the whole audit. Today the algorithm asks "which notes have exactly these keys?" and the answer fragments: `{born, died}`, `{born, died, occupation}`, `{born, died, predecessor, successor}` become three anonymous kinds. The rest of the world asks "for notes of this kind, how often does each key appear?" and the answer is one kind:

```
Person — 679 notes
  born        100%   (679)
  died         87%   (591)
  occupation   16%   (107)
  successor     3%    (18)
```

This is exactly what Wikipedia does. `Infobox person` is **one** template carrying a superset of parameters; `name` fills 91% of articles, `birth_date` 86%, `occupation` 82%, `alma_mater` 17.5%, and thirty parameter names are used exactly once. **Not one parameter is declared required.** Same for `Infobox film`. [sourced, verified] MongoDB Compass shows a field as "present in 87% of documents." Wikidata's Recoin shows "entities like this usually also carry X, Y, Z." quicktype merges samples into one type and marks the varying fields optional. Four independent systems, one representation.

Everything else in this report follows from that change: it dissolves the optional-field problem, it gives the user interface something honest to display, it makes naming easier, and it makes the whole thing incrementally updatable so we never rescan the Universe.

## Two concrete bugs found, both worth fixing before anything else

1. **`tags` is contaminating almost every proposal.** It sits on 99.7% of notes (7,603 of 7,625) and rides inside **111 of the 113** shapes the algorithm produces. The 90%-ceiling guard cannot catch it, because that guard filters candidate *groups* by size, never *keys* by frequency. The code comment claiming the ratio "catches whatever universal property a future Universe carries" is demonstrably false on the very corpus it was validated against. [sourced, independently reproduced twice] **Consequence that matters: after removing `tags`, only 3,040 of 7,625 notes carry any semantic property at all. Our effective corpus is 3,040 notes, not 7,802.**

2. **We throw away the single best signal we have.** Step 1 strips `kind` before grouping. Two independent literatures say this is precisely backwards: the JSON schema-inference field treats a `kind`-like property as *the* canonical attribute to split a collection **by** before inferring anything, and Wikidata's SchemaTree measured a **75% absolute** improvement when type information is fed in as a first-class signal — with the improvement largest exactly where our notes live, on records with few properties. And `kind`'s value is also the best available **name** for the group. [sourced]

## What is already right

Quite a lot — see Part VI. Notably: min-support 3 is in line with real practice; abandoning closed frequent itemsets was correct and the literature says why; refusing to invent a confidence score is correct; scanning the *whole* corpus rather than sampling is a genuine advantage over MongoDB Compass and we should say so on the panel.

## What is *not* true

The algorithm is **not currently running.** `discover_shapes` has zero callers repo-wide — `lib.rs` declares the module and nothing invokes it. There is no production extractor feeding it either. So these are latent defects in committed, doc-published, real-data-validated code, not user-visible ones. This is design work before wiring, which is the cheap moment to do it.

---

# PART II — THE OPTIONAL-FIELD QUESTION, ANSWERED

**The question:** are `{born, died}` and `{born, died, occupation}` one kind with an optional field, or two kinds?

## The answer has a name, and the name is "it's a parameter"

Baazizi, Colazzo, Ghelli & Sartiani's **parametric schema inference** makes the merge decision an explicit, chosen **equivalence relation**: [sourced, read in full — EDBT 2020, quotes verified verbatim]

- **Kind equivalence (K):** every record type is equivalent to every other; everything collapses into one type with fields marked optional (`?`).
- **Label equivalence (L):** record types are equivalent only if they share the same top-level field labels; differing key sets stay distinct.

The authors state flatly that **"there exists no 'best' precision level that can be fixed a priori."** JSONoid ships this as a `--equivalence-relation` flag with four settings. [sourced]

**Our step 2 is label equivalence.** Not "like" it — it *is* it, over a noise-stripped, non-recursive, untyped key set. So the fragmentation is not a bug in our implementation; it is the published, expected cost of sitting at the maximum-precision end of a known spectrum. No amount of threshold tuning at that end will fix it.

### Conflict surfaced — do not average these

**Track B recommended:** compute both a K-view (merged headline) and an L-view (drill-down).
**Verification refuted half of it:** K is not per-family. Applied to a 7,802-note Universe it yields **exactly one** record type carrying the union of every key in the corpus — useless. Getting "a Person note" requires first *partitioning* into a person family and only then merging within it. **That partition step is neither K nor L, and the paper does not supply it.** The hard part is precisely where the citation runs out.
**And our own code disagrees with Track B's framing:** `template_discovery.rs:130-139` records an empirical result on this same corpus — a closed-frequent-itemset pass was tried and was *measurably worse*, and the author concluded "a philosopher note and a plain person note ARE different templates, so reporting the variants separately is correct, not fragmentation." Track B's design consequence contradicts a measured result in the file it is about, without engaging it.

**Ruling for this document:** both can be true. The variants *are* real and worth showing; presenting them as 113 flat anonymous rows is what fails. The resolution is not "merge or don't" — it is **a family with per-key fill rates, plus its variants shown underneath**.

## The three techniques that actually implement this

**(1) Superset-plus-fill-rate — Wikipedia's answer.** [sourced, verified] One template, all parameters, nothing required, empty rows vanish on render so a superset costs nothing. Measured: `Infobox person` = 376,588 transclusions, 142 distinct parameters observed, **zero required, 3 suggested**; `Infobox film` = 165,230 transclusions, 43 parameters, **zero required, 17 suggested**.
*Correction from verification:* the earlier parameter counts in the research pass (130/68, 42/16) were mis-parsed — alias rows were dropped. Use 142/43. And "nothing on Wikipedia is required" is false in general (`Infobox album` requires name+artist); zero-required is the norm among *large* infoboxes, which is the relevant case. Also note Wikipedia splits **across** types (person vs officeholder vs musical artist) and composes via embedded modules — so it is not evidence against ever splitting kinds.

**(2) δ-tolerance closed itemsets — the tunable middle.** Cheng, Ke & Ng (ICDM 2006): X survives if no single-item extension retains ≥(1−δ) of its support. Their Lemma 1: closed = 0-tolerance. Lemma 2: maximal = 1-tolerance. **One knob, closed at one end, maximal at the other.** [sourced, definition read directly]

**⚠ Correction that will cost an engineer a day if missed:** Track C's prose defines δ by *retention* and then states the limits from the *tolerance* convention — the two are inverted. Track C's "δ ≈ 0.6–0.8" equals the literature's δ_TCFI ≈ 0.2–0.4. Anyone implementing from Track C's sentence, or importing a library following Cheng et al., **will tune the knob backwards.** Also: Track C's "a.k.a. margin-closed / Δ-closure" conflation is refuted by the very sentence it quotes — Buzmakov's Δ-closure is an *absolute* object-count drop, Δ=1 is closed, and large Δ collapses the lattice rather than approaching maximal. Three related, non-interchangeable notions; only δ-TCFI has the endpoints the argument needs.

**(3) Pattern profiles — the closest match to what we actually want.** Yan, Cheng, Han & Xin (KDD'05): a profile is a *master pattern* (union of a cluster of similar itemsets) **plus a per-item probability vector** plus a support. That is literally "mandatory fields at p≈1.0, optional fields at p<1.0." Their diagnosis of our problem, verbatim: *"for any pattern α, as long as there is a small disturbance on the transactions containing α, it may generate hundreds of subpatterns with different supports."* They also supply a principled way to choose how many profiles to emit (restoration error J; pick K where J jumps). Reported: 688 closed patterns on Mushroom → ~30 profiles at <10% error. [sourced, read in full] *Caveat: 2005 paper; we did not establish whether it is still in active use or whether the MDL line superseded it. Treat as re-implement-from-paper.*

## Recommendation

**Adopt representation (1) as the data model, and get the grouping from a partition + fill-rate roll-up rather than from a similarity dial.**

A discovered kind is stored as:

```
kind_id · member_note_count · { property_key → count } · { heading → count } · [variant signatures]
```

**Tradeoff, stated plainly:** merging loses the exact co-occurrence facts — that `predecessor` and `successor` always appear *together*, and never with `occupation`. That information is real and a user would want it. Baazizi's paper names this loss explicitly. **Mitigation:** keep the exact signatures as the substrate (we compute them anyway) and expose them as "the 3 shapes this kind actually takes." Cost: one extra table, no extra scan.

## What NOT to do

- **Do not switch to maximal itemsets.** Refuted on our own corpus, twice, independently. At min-support 3: 13,115 frequent → 849 closed → **91 maximal**, and the largest maximal itemset covers **18 notes**. `{born,died,tags}` at 679 notes is *not* maximal because it has frequent supersets. Maximal deletes exactly the type cores we want. A min-support sweep (3 / 25 / 100) confirms this is structural, not an artifact — the core only becomes maximal at min-support ≥163, where the whole feature collapses to 10 patterns. [sourced, reproduced with an independent Eclat implementation]
- **Do not retry closed frequent itemsets.** Our negative result is corroborated by the literature: Baazizi on Wang et al.'s "skeleton" — *"the skeleton may totally miss information about paths that can be traversed in some of the JSON objects."* SchemaTree names the three pathologies of association-rule mining for schema work: *"spuriousness of the underlying itemset generation, the inability to find negative association rules, and variations in property densities."* Record it as refuted so it is not retried a third time. [sourced]
- **Do not reach for Jaccard + hierarchical clustering as "the standard move."** It isn't standard, and the closest paper to our problem (Klettke et al., BTW 2015) considered Jaccard and rejected it by name: *"One JSON document with divergent structure can cause that the Jaccard measure delivers the value 0."* An empirical comparison of 13 clustering approaches on binary presence/absence data found hierarchical methods **worst** — single linkage chains, complete linkage cannot merge zero-overlap pairs. With 56 distinct keys and a mean of 1.94 semantic keys per note, a large fraction of our note pairs share **zero** properties, so the failure condition is in force. [sourced; caveat: that comparison is on species-by-geography data, transfer is by mechanism not by result]
- **Do not import quicktype's 3/4 constant.** ⚠ **Conflict between tracks.** Track E cites quicktype as "the industry answer: merge into one type and mark fields optional." Track B read quicktype's actual shipped source and computed that its merge criterion **would not merge our example**: `{born,died}` vs `{born,died,occupation}` fails the size guard (2 < 2.25); `{born,died,occupation}` vs `{born,died,predecessor,successor}` passes the guard but allows 0 faults and `occupation` is a fault. Both are right about different layers — quicktype's *output shape* (union + `?`) is the right target; quicktype's *merge test* is calibrated for generated API classes with 10–40 properties and degenerates to near-exact key-set identity below 5 keys, which is where all our notes live. Take the shape, drop the constant. [sourced, source read in full]

---

# PART III — THE ALGORITHM, HARDENED

## Pipeline specification

### Step 0 — Key-level noise filter *(NEW — fixes the `tags` defect)*

Before grouping, drop any property key whose corpus frequency exceeds ~90%, and any key seen only once or twice in the entire Universe.

- **Prior art for the high end:** Wikipedia's own tooling flags parameters absent from a template's declared vocabulary; Klettke's outlier detection is two-sided (`ratio < θ` = additional property, `ratio > 100−θ` = missing property). [sourced]
- **Prior art for the low end:** the tagged-unions paper's always-applied default heuristics — ignore single-valued-domain attributes and unique-valued attributes, because they *"cause overfitting and schema bloat."* This is a **data-driven replacement for our hardcoded `IDENTITY_KEYS` / `PROVENANCE_KEYS` lists**, and it generalises to user-invented keys that no list can anticipate. `cid_cn` and `created` would be *discovered* as identifiers rather than named. [sourced]
- **Our judgment:** keep the hardcoded lists as a belt-and-braces layer; add the cardinality test as the general mechanism.
- **Measured consequence:** removing `tags` leaves 3,040 notes with any semantic property. All supports, ratios and ceilings must be computed against that denominator, not 7,625.
- **Test debt:** `MAX_SUPPORT_RATIO` has **no covering test** today. The test named `a_property_on_almost_every_note_is_not_a_shape` passes for an unrelated reason (the lone-property filter), not via the ratio. [sourced, verified]

### Step 1 — Partition by discriminator, do NOT strip it *(NEW — reverses current step 1)*

Where a note carries a `kind` (or `type`, `category`) value, partition on it first, then group within each partition.

- **Klettke et al. §4.1 "Document Selection"**, verbatim: *"If a split attribute exists that can be applied for distinguishing the JSON documents of a collection, then we can build schema for a subset of documents… we preselect a NoSQL subset if different kinds of JSON documents are organized in the same collection."* Their own motivating example document literally carries `"kind": "BlogPost"`. [sourced]
- **SchemaTree (ESWC 2020):** feeding type information in as pseudo-properties boosted the score by **75% absolute** for entities with limited input properties. [sourced]
- **This also supplies the name** (Part IV). One change, three benefits.
- Notes with no discriminator fall through to the global path unchanged.

### Step 2 — Exact signature grouping *(KEEP — this is label equivalence, and it is the substrate)*

Do not replace it. Baazizi's own system computes the **precise (L) schema first and in full**, then derives the succinct view from it client-side; refinement is a pure presentation operation over already-computed data. [sourced, verified verbatim] Our 113 exact signatures are the substrate we must compute regardless. The roll-up is a *rendering*, not an alternative.

### Step 3 — Roll-up to families with fill rates *(NEW — the core change)*

Within a partition, produce `{key → count}` over the member notes, plus the list of exact signatures that composed it.

- **How to draw the family boundary** — three options, ranked:
  - **(a) Discriminator partition alone** (Step 1). Cheapest, most honest, no new parameter. *Recommended first build.*
  - **(b) δ-TCFI over signatures**, one tunable knob, endpoints closed↔maximal. Second build if (a) leaves too much fragmentation. **Remember the inverted-δ correction.** Note two unaddressed properties: the constraint checks only *immediate* single-item extensions, so a field *pair* that is jointly cheap is never caught; and the family is not an antichain, so both `{born,died}` (679) and `{born,died,institutions}` (131) survive — "one type per pattern" needs an extra rule the papers do not give.
  - **(c) Pattern profiles** (Yan et al.). Closest to the ideal output; heaviest build; requires re-implementation from a 2005 paper with no verified maintained implementation.
- **Do NOT do a naive union across the whole corpus.** That is Baazizi's K applied globally — one type, everything optional, useless.

### Step 4 — Alias detection by mutual exclusivity *(NEW — cheapest high-value item in the audit)*

Within a candidate family, for each key pair (a,b): if support(a) and support(b) are both meaningful and support(a∧b) ≈ 0, propose them as **two names for one field**.

- **KOG (WWW 2008), verbatim:** *"if two attributes of class c never have values filled simultaneously… this edit pattern also indicates duplication."* KOG found 5,365 duplicate attributes, ~4 per class, at 87% precision / 79% recall. Its alias bag for person→birth place: *birthplace, place of birth, place birth, location, origin, cityofbirth, born.* [sourced]
- Add a **canonicalisation pass** over keys before grouping — lowercase, strip underscores/hyphens/digits, singularise — exactly DBpedia's stated motivating problem: *"Different templates use different names for the same attribute (e.g. birthplace and placeofbirth)."* Keep an alias bag so the user's original spellings survive on disk. **File-Over-App holds: canonicalise for grouping, never rewrite the note.** [sourced]
- This is O(n), needs no new data, and is falsifiable against our 7,802-note Universe tonight.

### Step 5 — Variant lattice, not a flat list *(NEW)*

`{born,died}` is the parent; `{born,died,occupation}` and `{born,died,predecessor,successor}` are children. Present the parent as the offered template with children as "variants used by N notes."

- **KOG** builds an ISA lattice (MLN inference, 98.8% precision / 92.5% recall at confidence 0.5) and then uses **shrinkage** — a sparse child borrows evidence from its parent — reporting recall improvements of 55% and 457% on the two sparsest classes. [sourced]
- This preserves the user's real distinctions (a monarch note *is* different from a plain biography) while removing the anonymous three-way split that reads as noise. It also honours the `template_discovery.rs` author's empirical conclusion rather than overruling it.

## Thresholds — is there a principled method?

**Honest answer: no, and the field has not found one either. Our thresholds are in line with real practice; the defect is that they are invisible, not that they are guessed.**

| Threshold | Ours | Prior art | Verdict |
|---|---|---|---|
| min-support | 3 | KOG: 5 instances/class [sourced]. Tagged-unions: tested 50/35/15%, concluded 15% worked, *"our approach to setting this threshold is rather coarse"*, auto-adjustment = future work [sourced] | Defensible. **Do not claim a "published band" — verification refuted that; KOG's 5 is one data point and KOG itself calls its thresholds "simple statistics" pending better methods, with no ablation** |
| heading quorum | 40% | KOG kept attributes used by >15% of instances, discarding 54% of all attributes [sourced, verified verbatim] | **Test at 15% before defending 40%.** But do not claim "we know it's discarding real headings" — KOG never evaluated its own threshold's quality, and its noise was stranger-authored crowd data, not one person's notes |
| universal-key ceiling | 90% on groups | Klettke's two-sided θ on keys [sourced]; AWS Glue reportedly 70% partition threshold + 5-cluster cap [sourced, but from AWS re:Post support answers, **not** primary docs] | **Broken as implemented** — must move to key level |
| top-k shown | — | Recoin chose k=5 by inspection and said so in print; RecSys 2010 found 20 vs 5 recommendations gave **no** satisfaction gain [sourced] | See Part V |

**Three principled alternatives exist, and one is mature:** statistical significance testing with multiple-testing correction (Hämäläinen & Webb tutorial; Tarone's correction makes it tractable), null-model calibration (find s* where the count deviates from a random dataset with the same item frequencies), and MDL/top-k which sidestep the threshold entirely. [sourced] The Constellation-shaped version is a **swap-randomisation null over our own corpus**: *"this combination occurs 679 times; under a null preserving per-field frequencies you would expect 4."* That is both a threshold **and** a user-facing justification, which is the only kind of threshold this product should ship.

**Also note the rare-item dilemma:** our kind sizes span 3 to 679. A single min-support cannot serve both ends — this is the documented "rare item problem" (Liu et al., MSApriori, per-item minimum supports). [sourced]

**And there is a theoretical result that closes off the tuning branch entirely:** Han, Cheng, Xin & Yan (DMKD 2007) — *"in the presence of even low levels of noise, large frequent itemsets are broken into fragments of logarithmic size; thus the itemsets cannot be recovered by a routine application of frequent itemset mining."* An optional field **is** that noise. Stop looking for the right exact-match threshold. [sourced, section number and title verified]

## Redundancy control

The field's taxonomy is **lossless condensation vs lossy summarisation**, and the consensus is that condensation alone does not solve pattern explosion. Our own corpus is a clean instance: closed cut 13,115 → 849, still 7.5× more than our 113 exact groups. [sourced]

So we are choosing a **lossy summariser**, and the honest question is *lossy in what way*:
- δ-closedness loses low-margin subsets.
- Profiles lose exact per-variant supports (recoverable to <10% error).
- MDL loses everything that doesn't pay for itself in bits — and its output is a compression code table, not a taxonomy of types. **Do not reach for KRIMP as the primary fix**; the MDL survey's own discussion section reports that many published encodings in this space are unsound. [sourced]

Pick the loss a user would forgive. For Recognition, that is: lose the exact variant supports (recoverable on drill-down), never lose a type core.

## Incrementality — the full-rescan problem is solved

**Make the merge operator commutative and associative. Incremental maintenance falls out.**

Baazizi et al. (EDBT 2017), verbatim: *"Associativity is also important to enable incremental evolution of the inferred schema under updates… in the case of insertion of a new record in an existing record collection, thanks to associativity, we simply need to fuse the existing schema with the schema of the new record."* Their fusion rules: **(R1)** matching keys collapse and fuse recursively; **(R2)** *"keys without a match are deemed optional in the resulting type and decorated with a question mark ?"* JSONoid generalises every collected statistic to a monoid with a commutative merge, supporting both streaming and distributed tree-merge. [sourced, read in full]

**This lands exactly on CLAUDE.md Performance Rule 8 (Write-Time Derivation):**

```
kind_key_counts(kind_id, key, count)      -- maintained at note-save time
signature_support(signature_hash, count)  -- ditto
```

On save: decrement the old signature's counters, increment the new one's. That is a monoid over counters — **no re-walk of the Universe, no `scan_*` command** (which Rule 8 explicitly forbids), and it survives the resumable-backfill requirement because partial sums fuse. Both the family view and the variant view are computable from the same counters, so both stay incremental.

Recoin independently reaches the same conclusion from the other direction: *"a live computation of property frequencies is not realistic. By weighting property frequencies of types, it is possible to have these precomputed, and the difference is generally minor."* [sourced]

**Honest gap:** incrementality is proven for the *fusion/counter* route. We did **not** establish that δ-closed or profile-based mining can be maintained incrementally — both depend on global support ratios, and we suspect not straightforwardly. This is an argument for the counter representation over the mining representation.

## Implementation notes

- **Mining cost is a non-issue.** 56 distinct semantic keys, 38 frequent at support ≥3, mean 1.94 keys per note, max 14. A naive Python full-subset enumeration mined all 13,115 frequent itemsets plus closedness, maximality and five δ levels in well under a minute over 7,625 notes. **Do not import a mining library.** Spend the budget on the summarisation and naming layers, which is where all the difficulty is. [sourced, measured]
- **Determinism is not free.** quicktype's clustering is a greedy single-pass leader algorithm with move-to-front reordering and multiple prototypes per clique, allowing transitive chaining (A joins via B even where A-vs-C fails). Input order would be file-walk order for us — the same Universe could yield different kinds on different runs, which is fatal for a feature whose promise is "report what demonstrably recurs." If any agglomeration is built: process signatures in **descending support order**, test against a fixed clique representative, forbid transitive chaining. quicktype also found an opt-out (`--no-combine-classes`) necessary in practice; plan our equivalent from the start. [sourced, source read]
- **Doc drift to fix:** `template_discovery.rs` lines 22-29 still present closed frequent property sets as the adopted unit, contradicting the code at 127-148. And the published example "314 :: content_type, sources" no longer reproduces (302 today) — corpus drift.
- **Corpus discrepancy to reconcile:** our docs say 7,802 notes / 106 kinds; two independent measurements this pass found 7,667–7,672 `.md` files, 7,625 with semantic frontmatter, **113** exact-signature groups. The named examples reproduce exactly (film 64, philosopher 32, city 37), so the algorithm matches — the corpus has changed. Re-run through the real Rust pipeline before quoting any number.

---

# PART IV — NAMING

## Can a property set be matched to Wikidata / schema.org / DBpedia to propose "Person" or "Film"?

**Honest answer: not from the property set alone. No published method does this, and we could not find one.** [could-not-establish, both Track B and Track D]

Every naming method located requires something *other* than the key set:

| Method | What it keys on | Measured | Offline? |
|---|---|---|---|
| **KOG category heuristic** | The **member articles' own category tags** | precision 86.7 / recall 61.3 / **F1 71.8**; all four heuristics combined **86.1** [sourced] | **Yes — pure corpus read** |
| KOG CaseCheck / GoogleSpell / WikiQuery | Splitting on case; spellcheck; external lookup | F1 18.2 / 12.9 / 10.7 — all weak alone | Mixed |
| **Tagged unions (ucCFD)** | A **discriminator property's value** (`[type = "Point"] → …`) | Their justification: *"value-based conditions have a greater influence on the differentiation of schema variants than structural constraints, and are therefore preferred"* [sourced, read in full] | **Yes** |
| SDType | Statistical distribution of properties → an **existing ontology class** | ~F1 88.5%, added 3.4M type statements to DBpedia [sourced from abstracts only — **not verified against the paper body**] | Needs a shipped prior |
| quicktype | The **containing property key**; synthesises `IntOrString` for unnamed unions [sourced; second-hand] | — | Yes |
| DBpedia ontology | **Built by hand**, then crowd-sourced for ~18 years | 285 of 7,225 English templates mapped (3.94%), covering 80.7% of template occurrences but only **49.2%** of property occurrences [sourced; page vintage uncertain] | N/A |

## Recommendation — name from the members, not from the keys

**Route 1 (build this): the discriminator value.** Where notes carry `kind: film`, the name is *film*, read off the user's own data. Evidence-backed, zero guessing, no LLM, fully local. This is the same change as Step 1 — stop stripping `kind`.

**Route 2 (build this too): mine the member notes.** Most frequent shared tag → most frequent folder/library segment → most frequent title token → most frequent H1/H2. Rank candidates by how many members carry them. KOG's measured analogue scores **F1 71.8 from category tags alone, 86.1 combined** — so expect roughly 85% usable proposals and design the accept/edit affordance for the other 15%. Our corpus already has libraries called Film, Architecture, Physics; **measure the correspondence before building anything cleverer.**

**Route 3 (do NOT build first): vocabulary matching.** SDType-style scoring against a shipped schema.org/Wikidata prior is real and works — but it can only assign an *existing* class. For a private Universe full of one person's idiosyncratic kinds, it cannot invent a name for a set no vocabulary knows. It is also a shipped-prior dependency in a local-first app. Park it.

**Route 4: the user.** OpenRefine, quicktype, Compass, Glue and Kibana all converge here — **not one of them names an inferred type semantically.** quicktype lifts a name from the data's own vocabulary and expects a rename. OpenRefine pre-fills the new value from an observed value and requires approval. So: **pre-fill from the corpus's own tokens, show it as an editable field with a one-click "use this value" affordance from any observed candidate, and let the user's rename be the naming act.** [sourced]

**Stop treating "no naming" as a defect to be solved by cleverness.** It is the state of the art, and the user typing the name *is* The Constellation Way.

**Design consequence:** the UI needs a first-class **"recurs, unnamed"** state. DBpedia after 18 years maps only ~49% of property occurrences; roughly half of any real corpus's keys will never get a clean name. Do not force a label.

---

# PART V — PRESENTATION

**There is a converged, proven UX for "here is what we found — accept, edit, ignore," and OpenRefine's Cluster & Edit dialog is the closest working instance.** [sourced from OpenRefine core source, verified]

## The OpenRefine row — copy this shape, with the verified column order

Actual left-to-right order in `clustering-dialog.js` is **not** what the research pass reported. Verified: `Merge? | Values in cluster | New cell value | Cluster size | Row count`. The decision checkbox comes **first**, evidence second, the editable name **immediately adjacent to the evidence it is drawn from**, and both counts are demoted to the far right.

**Our row:**

```
[☐ Adopt] | [property set + each variant signature, each with its note count] | [editable Name ▾] | [# variants] | [# notes]
```

Verified UI strings: `Cluster size`, `Row count`, `Values in cluster`, `Merge?`, `New cell value`, `Use this value`, `($1 rows)`. [sourced verbatim]

**"Use this value"** — clicking any member value writes it into the name box **and auto-ticks the checkbox**. That is the naming affordance from Part IV, already solved and shipping.

## Five mechanisms worth copying exactly

**1. Default = do nothing.** No proposal applies unless ticked; OpenRefine blocks with *"You must check some 'Merge?' checkboxes for your edits to be applied."* Verified in source: every cluster is constructed with `edit: false`, and `_apply()` posts nothing when the edit list is empty. **Template Studio ships zero templates by default.**
*Correction from verification:* OpenRefine also ships a **"Select all"** bulk-adopt button, and it holds **no standing backlog** — clusters are computed on demand for one column, shown in a modal, discarded on close. So OpenRefine substantiates "nothing applies without a user act"; it does **not** substantiate "a large persistent proposal list is harmless because it is inert." That inference is ours, and should be stated as ours.

**2. Link straight to the evidence.** Each row carries a link that builds a real facet on the underlying data and opens the **actual rows** in OpenRefine's own grid — not a summary. *Correction: `Browse this cluster` and `Browse only these values` are **one** anchor (link text + tooltip), not two links, and it is hover-revealed (`visibility: hidden` until mouseenter).* For us: a one-click "show me the 64 notes" opening our existing note-list surface. **Cheapest trust mechanism in the entire audit, and it costs nothing — the corpus is local and the list already exists.** Unlike OpenRefine, make it **persistently visible**; the hover-gating is the one part not worth copying.

**3. Facet the proposal list, don't paginate it.** OpenRefine gives the *cluster list* its own histograms: `# Choices in cluster`, `# Rows in cluster`, `Average length of choices`, `Length variance of choices`. This is the direct answer to "113 is a wall." Give our list facets on support, property-count, heading-count, and library/folder concentration. "Show me kinds with ≥20 notes and ≥3 properties" turns the wall into eight rows, reusing our existing facet vocabulary with no ranking cleverness.

**4. Truncate honestly.** `"<b>$1</b> clusters included from <b>$2</b> total"`. Our header must read **"12 of 113 kinds shown"** with the residual reachable — never "12 kinds found." Hiding the tail silently is what converts a recognition tool into an authority claim, which the concept forbids.

**5. Merge-then-recompute, user-driven.** OpenRefine fixes over-splitting with a **"Merge selected & re-cluster"** loop, and its manual explicitly recommends working strictest-to-laxest *"starting with the most strict rules and moving to the laxest, which require more human supervision."* **This is the answer to fragmentation that does not require a better algorithm:** ship the strict pass (high precision), plus a user action "these are one kind" that re-runs discovery treating them as merged and recomputes support, the union property set, and per-property presence. Our closed-frequent-itemset failure *is* the "lax method without supervision" outcome the manual warns about.

## Per-property presence — MongoDB Compass

Compass renders a field as **percentage of sampled documents containing it**, with a per-type percentage breakdown when a field holds multiple types (`phone_no: 20% int32, 80% string`), plus value distributions. [sourced]

Our kind renders as `property — present in 87% (28 of 32)`. **That single number is what makes "one type with optional fields" legible instead of mushy** — and it makes the merge obviously *correct* to a human eye: born 100%, died 100%, occupation 34%, successor 12%.

**And we have a real advantage to state on the panel:** Compass samples **1,000 documents** and its docs concede rare fields can be missed. We read every note. **"64 of 7,802 notes — all notes scanned"** is a stronger trust statement than any confidence score, and it costs one line of copy. It is also the argument for keeping the full read honest rather than quietly sampling for speed.

## Ranking and how many to show

**Wrangler (CHI 2011)** ranks by (1) frequency of equivalent transforms in a usage corpus, then (2) **ascending complexity** — simpler first, because users evaluate simpler descriptions faster — then (3) a diversity filter capping any one transform type at roughly 1/3 of the list. [sourced; the exact wording of the diversity rule is *not* settled — two extractions of the paper disagreed]

**Our current rank (support, then specificity) matches criterion 1 but inverts criterion 2** — specificity favours the biggest, most complex property sets, which are the slowest to read. Recommended: **support first, then FEWER properties first as tie-break, plus a diversity cap** so the top tier isn't eight variants of the person family. Diversity is also a cheap partial mitigation for fragmentation — it stops near-duplicates crowding the visible tier before the user merges anything.

**Wrangler also previews on keyboard navigation** — arrowing the list live-previews the transform's effect in the data table before commit. Ours should live-preview the resulting **template** (frontmatter block + heading skeleton) in a side pane. Preview-on-hover is what lets a user evaluate ten proposals in seconds.

**How many:** **5–12 in the visible tier.** RecSys 2010 (Bollen, Knijnenburg, Willemsen, Graus) tested 5 vs 20 recommendations with quality held high and found **no** gain in choice satisfaction — the greater attractiveness of the large set was cancelled by increased choice difficulty, with behavioural evidence of longer search. [sourced; **caveat: media recommendation, transfers by analogy only**] 113 flat is out of the question. Short ranked tier + facets + honest truncation.

## The cautionary tale — Great Expectations

GX's Onboarding Data Assistant did the evidence part **right**: it emitted the underlying metrics alongside each generated rule and shipped `plot_metrics()` / `plot_expectations_and_metrics()` so users could see *why* each rule was chosen. [sourced]

But it also *"will create as many applicable Expectations as it can for the permitted columns. This provides a solid base... but may exceed your needs."* — its own docs, warning about its own output. **The auto-profiling path is gone from Great Expectations 1.0**; the recommended path is now manual authoring. [sourced — removal evidenced via issue #9969; **the reason for removal is inference, not sourced**]

**113 exhaustive proposals is the GX shape.** This is the strongest warning in the audit and it argues directly against improving recall as an end in itself.

## Do NOT invent a confidence score

Zhang, Liao & Bellamy (FAT\* 2020): showing confidence helps trust calibration **only when the confidence is itself well calibrated**; when it is not, displaying it **harms** human-AI team performance. Local per-prediction explanations produced no perceivable calibration effect. [sourced]

We have no calibrated basis for a 0–100 "template confidence," and a miscalibrated number is worse than none. **Show the raw verifiable quantities instead** — note count, share of corpus, per-property presence %, per-heading frequency — because those are facts the user can check by clicking through. That is the form of evidence that survives this literature, and it is the same conclusion GX reached by shipping metrics rather than scores.

## After acceptance

Land the adopted kind in a **conventional editable property list** — Notion/Airtable shape: one row per property, drag handle for order, inline-editable name, type control, duplicate/delete, "+ Add a property." [sourced] Keeping the post-acceptance editor conventional means **the only novel UI we must design is the proposal row itself.**

## Adopt the softness dial, not a schema

Three independent systems converge: **the type never declares requirements; the consumer does.**
- Wikipedia TemplateData: `required` / `suggested` / everything-else-optional — but the declaration model is really two independent booleans defaulting to false, plus `deprecated`. The trichotomy is a *consumer rendering* (VisualEditor grouping), not what the schema encodes. Build it as flags-with-a-default, not an enum. [verified correction]
- Wikidata: `mandatory` vs `suggestion` constraint status; and formal hand-authored EntitySchemas essentially failed to take hold — on the order of 500 schemas against millions of items. **Do not ask users to author schemas; the largest collaborative KG in the world could not make that stick.** [sourced]
- schema.org: *"we take a pragmatic view of conformance"*; validators *"are not obliged to treat unexpected structures as errors"*; *"some data is better than none."* Required/recommended is a **Google consumer-side layer**, not part of the type system. Google's own advice: prefer fewer well-filled properties over a long aspirational list — which argues against offering all 20 tail keys when a user picks a kind. [sourced]

**Ship discovered kinds with zero required keys**, and let the user promote a key to suggested/required if they choose — warn, never block.

## One free feature that falls out

Once a kind is `{key → fill rate}`, we get Recoin's read-side for nothing: opening a note, *"notes like this usually also carry X, Y, Z"* — pure recognition from the user's own corpus, no prediction. And once a kind is adopted, near-misses become a signal: *"7 notes look like Film but are missing `country`."* That turns adoption into ongoing value rather than a one-shot template emit. Segment Protocols does exactly this (accepted plan → deviations surface as "Violations" rather than silently changing the plan), and also lets the user scope the evidence window (last 24h / 7d / 30d) — a cheap, honest way to shrink 113. [sourced]

---

# PART VI — WHAT OUR CURRENT ALGORITHM GETS RIGHT

Do not rewrite these.

1. **Exact-signature grouping is a real, named technique — label equivalence — and it is the correct substrate.** Baazizi's own system computes the precise view first and derives the succinct one from it. Keep it. [sourced, verified]
2. **Abandoning closed frequent itemsets was correct, and the literature says why.** Our empirical negative result (bare `born`, `country` surfacing as if they were types) is corroborated independently by Baazizi's critique of Wang et al.'s "skeleton" and by SchemaTree's three named pathologies of association-rule mining. Not an implementation mistake. [sourced]
3. **The recorded judgment that "a philosopher note and a plain person note ARE different templates" is defensible** and is backed by KOG's variant lattice. The problem is the flat anonymous presentation, not the distinction.
4. **min-support 3 is in line with real practice**, not an outlier — and the field has not cured threshold-guessing either. Recoin picked k=5 by inspection and said so in print; the tagged-unions team called their own approach "rather coarse" and deferred auto-adjustment to future work. [sourced]
5. **Reading the whole corpus rather than sampling is a genuine competitive advantage.** MongoDB Compass samples 1,000 documents and concedes rare fields can be missed. Our supports are exact. Say so on the panel. [sourced]
6. **Refusing to invent a confidence score is correct** and is backed by FAT\* 2020. [sourced]
7. **The stripping instinct is right even though the current list is wrong.** Identity and provenance keys genuinely must go — the tagged-unions paper applies exactly these as always-on default heuristics. Our error is *how* (hardcoded list) and *what* (`kind` should not be in it), not *whether*.
8. **Attaching headings to a kind is empirically sound.** Wikipedia infobox-type prediction from Table of Contents alone: 65% (Random Forest) / 76.5% (CNN) micro-F1; abstract alone 86%/95.1%; ToC+abstract 88%/96.1%. Headings work as a **confirming** signal; body text is the stronger lever if kind-detection ever needs strengthening. [sourced]
9. **Mining cheaply and locally is the right architecture** — no library needed, sub-second in Rust at our data shape.
10. **The concept is unclaimed territory and worth building.** 11/11 audited mechanisms require a declared type. Nobody proposes a kind from the corpus. [sourced]

---

# PART VII — DECISION POINTS FOR THE OWNER

**1. Does a discovered kind mean "notes with exactly these fields" or "notes of this sort, with these fields at these rates"?**
→ **Recommendation: the second.** It is what Wikipedia, Wikidata, MongoDB and quicktype all converged on independently; it dissolves the fragmentation argument; and it makes the panel legible ("occupation: 34%"). **Cost:** we lose the exact co-occurrence facts unless we also keep the variants — so keep both, family on top, variants underneath.

**2. Do we stop stripping `kind` from the analysis?**
→ **Recommendation: yes, strongly.** Two independent literatures say a `kind`-like property is the canonical thing to split **by**, and SchemaTree measured a 75% absolute improvement from feeding type information in — largest exactly where our notes live. It also solves naming. This is one small change to step 1 with three payoffs. **Risk:** notes with no `kind` need the old global path as a fallback; that must be built, not assumed.

**3. Do we ship one strict pass plus a user "merge these" action, or try to get the grouping automatically right?**
→ **Recommendation: strict + user merge.** OpenRefine's manual says laxer methods "require more human supervision"; AWS Glue makes schema-combining an explicit user policy rather than an algorithmic guess; our own closed-itemset attempt is the documented failure mode of unsupervised laxness. **Trade:** the user does more work up front; the app never presumes. That is The Constellation Way and it is also the shipped industry default.

**4. How many proposals do we show — a short ranked tier, or everything?**
→ **Recommendation: 5–12 visible, faceted, with "12 of 113 shown" stated in the header and the tail one click away.** Great Expectations generated everything, warned in its own docs that it "may exceed your needs," and the feature is gone from 1.0. RecSys evidence says 20 beats 5 on nothing. **Trade:** a user with an unusual small kind has to filter to find it — which is what the facets are for.

**5. Do we name kinds automatically?**
→ **Recommendation: propose, never assert.** Pre-fill from the user's own vocabulary (`kind` value → most common tag → folder/library → title token), show it as an editable field with one-click "use this value," and add a first-class **"recurs, unnamed"** state. Expect ~85% usable proposals (KOG's measured analogue). **Do not** build vocabulary matching against schema.org/Wikidata first — it can only assign names a shipped vocabulary already knows, which is the wrong shape for a private Universe.

**Two things to fix regardless of the above rulings, before any design is locked:** the key-level `tags` filter, and a covering test for the universal-key guard.

---

# PART VIII — VERIFICATION RECORD

## Refuted

| Claim | Status |
|---|---|
| Switch to **maximal itemsets** to fix bare-single-field noise | **REFUTED on our own corpus**, twice, independently (a second verifier wrote a fresh Eclat implementation). 91 maximal itemsets, largest covers 18 notes; every type core deleted. A min-support sweep (3/25/100) shows it is structural, not an artifact |
| **Jaccard + hierarchical clustering** as the standard move | **REFUTED.** Klettke et al. considered and rejected Jaccard by name; a 13-method comparison found hierarchical worst (chaining / zero-overlap saturation). Our corpus has the failure condition in force (mean 1.94 keys/note) |
| **Closed frequent itemsets** | **REFUTED empirically (ours) and corroborated by literature.** Record it so it is not retried |
| **KRIMP/MDL** as the primary fix | Refuted as *primary*. Classic KRIMP still needs a pre-mined list (hence a min-support); output is a compression code table, not a taxonomy; the MDL survey's own discussion reports many published encodings are unsound |
| A vault-scanning **type-inference plugin exists somewhere in Obsidian** | Not found — but this is negative evidence from targeted search, not an exhaustive registry enumeration. Do not state it as certain |

## Corrected — errors that would have cost build time

| What was claimed | The correction |
|---|---|
| **δ: "δ→0 gives closed, δ→1 approaches maximal"** | **Inverted.** Track C defined δ by *retention* and stated limits from the *tolerance* convention. Track C's δ≈0.6–0.8 = literature's δ_TCFI≈0.2–0.4. **An engineer implementing from that prose, or importing a Cheng-et-al.-conformant library, will tune the knob backwards.** Also: δ-TCFI, margin-closed, and Δ-closure are **three different things**, not aliases — only δ-TCFI has the closed↔maximal endpoints the argument needs |
| **"Compute a K-view: one merged kind = a person note"** | **That is not K.** K merges *every* record type; on 7,802 notes it yields exactly one type with the union of all keys. Getting "a Person note" needs a partition step first, which the paper does not supply. The hard part is where the citation runs out |
| **"Our grouping is label equivalence, unmodified"** | Overstated. It is L over a *noise-stripped, non-recursive, untyped* key set. The filtering is load-bearing — without it the corpus gave 7,380 signatures for 7,802 notes |
| **"quicktype = the industry answer, merge into one type"** vs **"quicktype's rule would NOT merge our example"** | **Both true, different layers.** Its output shape (union + `?`) is the right target; its 3/4 merge test degenerates to near-exact key-set identity below 5 keys. Take the shape, drop the constant |
| **Obsidian "has no inference / no suggestions of any kind"** | **Refutable in ten seconds by any Obsidian user.** Obsidian *does* infer value types per property key vault-wide (`.obsidian/types.json` holds only manual selections; "the rest is guessed" — per an Obsidian team member) and *does* autocomplete property names from the vault. Correct framing: **"Obsidian's unit of analysis is the key; it never proposes a key set."** Also: Bases has been **core** since 1.9.0 (May 2025), so "Obsidian core cannot group notes by properties" would be false — the differentiator is *inferred/proposed* grouping |
| **Obsidian Bases "infers nothing"** | Confirmed for schema/kind, but trim the absolute: Bases *does* auto-derive file metadata (`file.name`, `file.tags`, `file.backlinks`, `file.mtime`…) and auto-recognises frontmatter links as Link objects. Say "performs no schema or kind inference" |
| **Metadata Menu is "the most-cited" types plugin** | Unsourceable editorialising; no ranking exists. Only hard figure: ~274,434 downloads. Also the docs' priority list has **seven** tiers (the seventh is settings presets, not a fileClass binding), and the plugin *can* prompt on file creation — it asks *when*, never proposes *what*. And "no feature that scans" rests on documentation, not a source read: say **"no documented feature"** |
| **Wikipedia parameter counts (130/68/3/1 and 42/13/16/13)** | **Mis-parsed** — alias rows were dropped. Correct: person = **142** parameters (0 required, 3 suggested), film = **43** (0 required, **17** suggested), verified against the live `action=templatedata` API. Every fill-rate percentage quoted *is* correct |
| **"In practice nothing on Wikipedia is required"** | False as a general claim — `Infobox album` requires name+artist; `Cite web` requires url+title. **Zero-required is the norm among large infoboxes** (person, film, settlement, officeholder, television, book, musical artist all 0), which is the relevant case. Say the narrow version |
| **Wikipedia's answer is "one superset template"** | Half of it. Wikipedia maintains **hundreds of type-specific infoboxes** and composes them via embedded modules. Superset *within* a type; template family *across* types. Do not cite Wikipedia as evidence against ever splitting kinds |
| **KOG gives us a "published band" for thresholds** | **Overstated.** KOG's 5 and 15% appear once, with no ablation and no sensitivity analysis. KOG's own text calls them *"simple statistics"* pending *"more sophisticated methods."* We can say our numbers are *anchored* to a precedent; we cannot say they are *calibrated*. And min-support 3 is our own guess — one precedent is not a band. Also drop "the only published analogue" |
| **"Expect 40% to be discarding real headings"** | Over-transfer. KOG never evaluated its threshold's quality, and its noise was stranger-authored crowd data, not one person's notes. Test 15%; don't predict the outcome |
| **OpenRefine column order** | Wrong. Actual: `Merge? | Values | New cell value | Cluster size | Row count` — **decision first, counts last.** The research pass had counts first, and the design consequence inherited the error |
| **"Browse this cluster" + "Browse only these values" = two links** | **One anchor** — the second string is its tooltip. It is also hover-revealed (`visibility: hidden`), which is the one part not worth copying |
| **"Nothing applies unless you tick a box"** | Confirmed — but OpenRefine also ships **Select all**, and clicking a candidate *value* auto-ticks the row. And OpenRefine holds **no standing backlog** (modal, computed on demand, discarded on close), so it does **not** substantiate "113 unadopted proposals are harmless because inert." That inference is ours |
| **`tags` is a "live" defect** | **Latent, not live.** `discover_shapes` has zero callers; `lib.rs:70` declares the module only. Committed, doc-published, real-data-validated — but unreachable. Also: the docstring claim that the ratio "catches whatever universal property a future Universe carries" is **demonstrably false** (removing `stage`/`maturity` from the hardcoded list leaves all 112 groups carrying them intact), and `MAX_SUPPORT_RATIO` has **no covering test** |

## Could not establish — do not fill these in

- **Baazizi et al. VLDB Journal 2019** (the primary parametric-schema-inference paper) is **paywalled**. K/L definitions come from the EDBT 2020 demo paper and JSONoid, which agree — but we do **not** have the full menu of equivalence relations or the experimental table showing how many types each equivalence yields on real data. That table would be the closest available calibration for "how much would K collapse our 113 kinds."
- **No published criterion calibrated for small property sets.** Every concrete rule found (quicktype 3/4, Glue 70%) targets 10–40-field API classes or table columns. Our frontmatter has 2–8 semantic keys, where ratio arithmetic behaves qualitatively differently.
- **No principled way to choose δ.** Every source treats it as user-supplied. The δ≈0.7 figure came from eyeballing output on our corpus — the same class of judgment call as min-support 3, just better placed.
- **No method names a type from its property set alone.** Every naming approach found requires a discriminator value, member labels, or a hand-built vocabulary. The LLM-labelling route (arXiv:2407.03286, CEUR Vol-3941) was **not read** — no claim either way.
- **Incrementality for δ-closed or profile-based mining is unestablished.** Proven only for the fusion/counter route. Both depend on global support ratios; we suspect not straightforwardly, but did not verify.
- **Whether SchemaTree was ever deployed** in production Wikidata (the live service still appears to be the association-rule PropertySuggester).
- **The reason Great Expectations removed auto-profiling** in 1.0 — the removal is evidenced; the motive is inference.
- **AWS Glue's 0.7 threshold and 5-cluster cap** appear in AWS re:Post *support answers*, **not** in the primary crawler documentation. Treat as support-channel, not spec.
- **Wrangler's exact diversity rule** — two extractions of the same paper disagreed ("no type exceeds 1/3" vs "1/3 emphasise less common alternatives"). Direction is sourced; formulation is not.
- **OpenRefine's numeric truncation default** — the strings are verified verbatim, the default value and preference key are not.
- **Read from abstracts/summaries only** (marked as such throughout): SDType (Paulheim & Bizer), Infoboxer, iPopulator, Wang et al.'s "skeleton," Moerchen's margin-closed (Springer paywalled), and the Wikipedia-infobox-suggestion papers.
- **Primary sources unreadable this pass:** Horvitz's mixed-initiative principles (CHI 1999) and Amershi et al.'s 18 Guidelines (CHI 2019) — both blocked; **do not cite specific guideline numbers** until read. Also *"Schema validation and evaluation framework for extracted schemas in JSON databases"* (Scientific Reports 2026), which is the most significant gap for "current standing" — our picture of the field rests on 2015–2023 sources plus this pass's reading of shipped source code.
- **Roam, Mem, Reflect, Amplenote, Zettlr, SilverBullet, Anytype, Capacities, Breadcrumbs, Linter, Templater, Projects** — **not audited to source depth.** No evidence either way. Do not report them as "no inference" on this document's authority.
- **Corpus discrepancy unresolved** — 7,802 notes / 106 kinds (our docs) vs 7,625 / 113 (two independent measurements). Named examples reproduce exactly, so the algorithm matches; the corpus drifted. **Re-run through the real Rust pipeline before quoting any number in a design doc.**
- **Nothing here was benchmarked against our corpus through the production extractor.** Every prediction about what a technique would do to our 113 kinds is a calculation or a proxy-parse measurement, not a result from the shipping code path.

---

*Prepared under WA#5 (cross-check against proven methods before building) and the BASIC RULE (no invented detail). Every claim carries its grade; conflicts between research tracks are surfaced rather than averaged.*
