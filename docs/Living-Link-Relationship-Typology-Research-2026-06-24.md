# Living Link — Relationship Typology Research (2026-06-24)

**What this is.** An open-minded, sourced survey of *the territory of typed relationships* — commissioned by
Eisa after we discovered the **directional vs symmetric** split. It maps the fields that have studied "how
things relate" (relation algebra / OWL, knowledge-organization standards, argumentation theory, PKM typed
links, cognitive linguistics) so we can see what Constellation's 8 link types already cover, what we just
found, and what is genuinely uncharted. Workflow: `wf_f97e9d18-518` (5 parallel digs + adversarial
fact-check). Every claim traces to a named source; honesty flags are preserved.

> **Status:** research only. No vocabulary decision is made here. This is the input to a future Concept Paper
> (the cognitive vocabulary is Boss-defined — Concept-Before-Function). MIG-086 §D proceeds on the current
> vocabulary + the directional/symmetric model.

---

## The headline: a link varies on TWO axes

1. **DIMENSIONS** — the *structural properties* a relation has, independent of meaning (from relation
   algebra; OWL 2 makes each one a declarable characteristic). "Direction" is only one of these.
2. **FAMILIES** — the *recurring meanings* of relations (from ontology, argumentation, cognitive science).

We had been thinking only about one value of one dimension (direction = asymmetric). The map is far larger.

---

## Axis 1 — DIMENSIONS (structure)

| Dimension | What it is | Constellation status |
|---|---|---|
| **Symmetry** (symmetric / asymmetric / **non-symmetric** / antisymmetric) | does the relation hold both ways? OWL `SymmetricProperty` / `AsymmetricProperty` | **JUST FOUND.** Nuance: `supports` is *non-symmetric* (mutual support is coherent), not strictly asymmetric like `causes`. A binary directional/symmetric flag can't say "mutual support." |
| **Transitivity** (A→B, B→C ⟹ A→C) | OWL `TransitiveProperty`; turns flat edges into a traversable inference structure | **UNCHARTED.** `generalizes`, `part-of`, `supersedes` are mathematically transitive (chains/DAGs); we store flat edges and compute no closure — inference left on the floor. **Caveat:** `part-of` is 6 subtypes (Winston/Chaffin/Herrmann 1987) and is transitive *only within a subtype*. |
| **Inverse / converse** (every directional rel has a partner) | OWL `inverseOf`; SEP: a relation and its converse "cannot exist independently" | **UNCHARTED.** `causes`↔caused-by, `part-of`↔has-part, `generalizes`↔specializes. We store one direction; the backlink reads generic ("X links here") instead of typed ("X has-part this"). The names don't exist yet. |
| **Arity** (binary vs **n-ary**) | W3C: "a property is a *binary* relation." n-ary needs reification into a node | **UNCHARTED — the biggest frontier.** "These notes TOGETHER form an idea" is genuinely n-ary; our `note_links` table is purely pairwise. (See the `complements` reframe below.) |
| **Cardinality / functional** | OWL `FunctionalProperty` (≤1 target) | `supersedes` is the natural functional/one-to-one relation → clean version *lineage* vs a tangle. Unenforced today. |
| **Taxonomic vs Thematic** (the cognitive axis) | taxonomic = shared features (dog/bear); thematic = co-occur in a scenario (dog/leash). Neuro-dissociated (PMC5393928; PNAS) | **PARTIAL, lopsided.** We are almost entirely taxonomic + logical. The whole **thematic** half (temporal, spatial, functional) is collapsed into the untyped `associative` default. |
| **Polarity / Target-of-attack / Illocutionary** (argumentation) | support vs attack; attack the claim vs premise vs **inference**; assertion vs **question** | **UNCHARTED.** Our `contradicts` collapses the three attack targets; we have no question relation at all. |

---

## Axis 2 — FAMILIES (meaning), and where we stand

| Family (source) | Constellation | Status |
|---|---|---|
| Class inclusion / hyponymy (Chaffin; WordNet) | `generalizes` / `exemplifies` | **COVERED** |
| Contrast / opposition (Cruse; RST Contrast) | `contradicts` | **COVERED** (symmetric — RST confirms) |
| Causal (ConceptNet; RST Cause) | `causes` | **COVERED** |
| Evidential / argumentative (RST Evidence/Justify) | `supports` | **COVERED** |
| Part-whole / meronymy (Winston/Chaffin/Herrmann — **6 subtypes**) | `part-of` (1 of 6) | **COVERED but coarse** |
| Provenance / derivation (ConceptNet DerivedFrom) | `derives-from` | **COVERED** |
| Supersession / replacement | `supersedes` | **COVERED** (Constellation-specific) |
| Similarity / synonymy (Chaffin Similars; ConceptNet SimilarTo) | `associative` (≈ RelatedTo) | **PARTIAL** — no explicit symmetric `similar-to` |
| **Thematic / functional** (used-for, prerequisite-of, precedes, near) | — | **NOT CHARTED** (half of human relating) |
| **Analogy / structure-mapping** (Gentner — analogous-to / maps-to) | — | **NOT CHARTED** (the synthesis engine) |
| **Argument attack/question** (Pollock; IBIS; Toulmin) | — | **NOT CHARTED** (the Tension act) |
| **N-ary synthesis** (W3C n-ary) | the proposed `complements` | **JUST FOUND** (mis-named — see below) |

---

## The big reframe — "complements" is the n-ary SYNTHESIS frontier, and a naming trap

Both the cognitive and argumentation digs flagged this, load-bearing:

- **"Complementarity" in linguistics means the OPPOSITE of what we mean.** Lexically it is *mutually-exclusive
  opposition* — dead/alive, true/false, odd/even (Cruse 1986): two meanings that "exhaustively divide a
  domain into two mutually exclusive parts." Our "complements" = two notes that **complete each other into a
  synthesis** (jigsaw pieces). Same word, opposite spirit. **Do not name it "complementarity."** A better
  name: **co-completes / jointly-constitutes.**
- Worse (or better): "together they form an idea" is **not a pairwise symmetric link at all — it's n-ary.**
  The honest model is a **synthesis node** that N constituent notes point into (reification), itself carrying
  confidence/weight — which the Living Link architecture *already* supports (the W3C n-ary note motivates
  n-ary precisely by the need to attach certainty/strength to the relation, which our links already do). So
  the right home for "complements/synthesis" is the **arity** frontier, not the contrast family.

---

## The genuinely uncharted territories — ranked by (value × source-strength)

1. **Thematic / functional family** — `used-for`, `prerequisite-of` ("you need this idea before that one"),
   `precedes` (temporal), `near` (spatial). Backing: taxonomic/thematic dissociation (PMC5393928, PNAS);
   ConceptNet `UsedFor`/`HasPrerequisite`; CARIN (Gagné & Shoben). *Half of human conceptual relating; we
   have ~none.* Highest value, rock-solid sources.
2. **Analogy / structure-mapping** — `analogous-to` / `maps-to`. Backing: Gentner Structure-Mapping Theory.
   *Relational* similarity (deeper than surface `similar-to`); the engine of insight/synthesis; squarely
   on-mission ("connecting, challenging, synthesizing"). Nothing covers it.
3. **N-ary synthesis** — "these notes TOGETHER form an idea" (the `complements` reframe). Backing: W3C n-ary.
   The biggest architectural leap (breaks the binary-link assumption) and the most distinctive.
4. **Inverse-pair converses** — name the reverse of each directional type (causes↔caused-by, part-of↔has-part,
   generalizes↔specializes). Backing: OWL `inverseOf`; lexical converseness (buy/sell). Low-cost, high
   coherence — makes every directional link readable, and typed, from both ends.
5. **Transitive closure of hierarchical links** — inference over `generalizes` / `supersedes` / `part-of`
   (within one meronymy subtype). Backing: OWL `Transitive`; OBO Relation Ontology. **Caveat:** never chain
   `part-of` across its 6 subtypes (finger→musician→orchestra ≠ finger→orchestra).
6. **Argument relations beyond support/contradict** — the reasoning vocabulary of the **Tension** act:
   - **undercuts / undermines** — attack the *inference* or the *premise*, not the claim (Pollock's three-way
     attack; Argdown/ASPIC+/Walton). *The #1 missing reasoning relation* — our `contradicts` collapses all three.
   - **problematizes / raises-question** + inverse **answers / responds-to** — questions as first-class
     (IBIS; QUD). Serves Tension → Synthesis directly.
   - **qualifies / limits** — Toulmin's qualifier; "true, but only when…" (a gradable weakener ≠ contradict).
   - **refutes vs rebuts** — the *decisive* defeat (Walton); possibly expressible via existing **confidence**
     on `contradicts` rather than a new type.
   - **elaborates / concedes / provides-background-for / is-condition-for** — RST subject-matter relations.

---

## Mapping to the Five Acts of Knowledge Creation

- **Connection** → thematic/functional + similarity + inverse-pairs (richer ways to relate).
- **Tension** → the attack family (undercuts/undermines/refutes) + contradicts + problematizes.
- **Synthesis** → analogy (maps-to) + n-ary synthesis (co-completes) — the act we're *thinnest* on.
- (Observation, Conviction) → served by confidence/weight already on the Living Link.

The thinnest acts — **Tension** (no inference/premise attacks, no questions) and **Synthesis** (no analogy,
no n-ary) — are exactly the uncharted territory. For a *formulation* (not management) tool, that's the gap
that matters most.

---

## Honesty flags (carry these)
- **`complements` naming** — "complementarity" (lexical) = mutually-exclusive opposition; ours = co-completion.
  Rename; treat as n-ary synthesis.
- **part-of** is 6 subtypes; transitive only within a subtype (Winston/Chaffin/Herrmann).
- **gIBIS extended link names** (generalizes/specializes/questions/replaces…) could not be verified against a
  primary source — not canonical.
- **refute vs rebut** is a real theoretical distinction but "not firm in everyday usage" — design accordingly.
- **UMLS** has *five* associative top categories (physically/spatially/temporally/functionally/conceptually
  related), not four.
- Murphy's exact "Relation by Contrast" wording + Chaffin's full 31-relation list were not retrieved verbatim
  (publisher 403s); the five families + three distinguishing properties ARE confirmed.

## Primary sources (selected)
OWL 2 Primer & Reference (W3C); "Defining N-ary Relations on the Semantic Web" (W3C); SEP *Relations* &
*Defeasible Reasoning*; Mann & Thompson 1987/88 + SFU RST definitions; Pollock via Prakken & Horty 2012;
ASPIC+ (Prakken); Argdown syntax; Toulmin (Hitchcock); IBIS (Kunz & Rittel; Conklin); QUD (Roberts 1996);
SDRT (Asher & Lascarides); Gentner 1983 Structure-Mapping (+ Northwestern QRG SME); Chaffin & Herrmann 1984;
Winston/Chaffin/Herrmann 1987; Cruse 1986 *Lexical Semantics*; Murphy 2003 *Semantic Relations and the
Lexicon*; ConceptNet relations; WordNet; UMLS Semantic Network; taxonomic/thematic (PMC5393928, PNAS).
