# PJ-067 — The Living Link Relationship Model v2 — Concept Paper

**Version 1.0 (draft for ratification) | 2026-06-27**
**Author of the vocabulary:** Eisa ALSHAMSI (the cognitive vocabulary is Boss-defined — Concept-Before-Function)
**Maintainer / synthesis:** Claude (consultant / engineer / SME)
**Status:** **CONCEPT RATIFIED — Eisa 2026-06-27** (the model's shape + the load-bearing **R4 grammar ruling = FAMILY**). The remaining rulings (R1 dimension-model silent-vs-surfaced · R2 inverse labels · R3 thematic family · R5 the synthesis node + who-authors · R6 phasing) are finalized at the relevant `/migration` Architect (sequenced **Tension-first**, per R6). **No schema or code is touched yet** — building follows only when the Boss greenlights each `/migration`. Concept-Before-Function satisfied: the vocabulary's shape is stated and ratified before any carriage. *(R4 was decided via a three-defence wargame — `docs/concept-papers/PJ-067-R4-Wargame-Question-Relation.md`.)*

> **What this answers.** The eight cognitive Living Links are a real vocabulary of inquiry — but the Relationship Typology Research (2026-06-24) showed they cover only part of the territory of *how things relate*. This paper turns that **survey** into a ratifiable **model**: it (a) makes the structural *behaviour* of a link **declarable** (the inference engine), and (b) charts the uncharted **families**, each anchored to the Five Acts — with the *thinnest* acts, **Tension** and **Synthesis**, as the priority for a *formulation* (not management) tool. Grounded in: Living-Link-Concept-Paper-v1.0 (RATIFIED), the Typology Research, and a 4-frontier design dig (n-ary synthesis · the Tension vocabulary · analogy/structure-mapping · per-type characteristics — workflows `wf_e6c2dade-bed` / `wf_bae71cca-fd1`, with primary sources).

---

## 1. Thesis — a link has STRUCTURE *and* MEANING; v2 makes both first-class

The Typology Research's headline: a typed relation varies on **two axes**.

- **DIMENSIONS** — the *structural algebra* a relation has, independent of meaning (relation algebra; OWL 2 makes each a declarable characteristic): symmetry, transitivity, inverse/converse, arity, cardinality, taxonomic-vs-thematic.
- **FAMILIES** — the *recurring meanings* (ontology, argumentation, cognitive science): the covered 8, plus the uncharted (thematic, analogy, n-ary synthesis, argument-attack, questions).

Today Constellation stores only the **meaning** (the `link_type` string) and leaves the **behaviour** implicit — in code and in the user's head. **The v2 move is two-fold:**

1. **Lift the algebra into the type definition** (the *dimension model*, §2) — so the engine can *reason*: render typed backlinks from both ends, compute trustworthy transitive reachability, and warn on structural violations. *(Concept: a link doesn't only say something — it behaves a certain way; make the behaviour declarable.)*
2. **Chart the uncharted families** (§3–§4), each with its **horse** (the one question it answers) and its place in the **Five Acts** — so the thin acts get the vocabulary a formulation tool needs.

This is **Form-Aligns-To-Purpose** at the level of the whole vocabulary: every dimension and every family earns its place by answering a question the existing eight cannot, never by decoration.

---

## 2. Layer A — DIMENSIONS as declarable per-type characteristics (the engine)

The single highest-leverage change. OWL 2 already separates a relation's *name* from its *characteristics*; Constellation should adopt the same set as a **per-type declaration** in the Link-Type Registry (seeds and custom deltas alike).

**The declarable set (OWL 2):** `Symmetric / Asymmetric / Non-symmetric` · `Transitive (true | guarded | false)` · `inverseOf` (a converse label, + an `inverse_type` when the converse is itself a forward type) · `Functional / InverseFunctional` (cardinality) · `Reflexive / Irreflexive`.

**The characteristics of the current 8** (`+`=holds, `–`=forbidden, `~`=qualified):

| Type | Symmetry | Transitive | Converse (inbound reading) | Functional | Reflexive |
|---|---|---|---|---|---|
| supports | **non-sym** (mutual support is coherent) | – | supported-by | – | irreflexive |
| contradicts | **symmetric** | – | contradicts (self-converse) | – | irreflexive |
| causes | asymmetric | ~ weak (attenuates; **off by default**) | caused-by | – | irreflexive |
| exemplifies | asymmetric | ~ within one abstraction lattice | generalizes (**inverse type**) | – | irreflexive |
| generalizes | asymmetric | ~ within one abstraction lattice | exemplifies (**inverse type**) | – | irreflexive |
| derives-from | asymmetric | **+ (lineage)** | basis-for | – | irreflexive |
| part-of | asymmetric | **~ only within one meronymy subtype** | has-part | – | irreflexive |
| supersedes | asymmetric | **+ (lineage)** | superseded-by | **+** (one predecessor) | irreflexive |
| *associative* (untyped) | symmetric | – | associative (self-converse) | – | reflexive-OK |

**The horse for each dimension** (what it's FOR):
- **Symmetry** — so the engine knows when a backlink is the *same* relation (contradicts) vs a *different* one (caused-by), and can warn on an accidental mutual-causation pair. The research's nuance lives here: `supports` is *non-symmetric* (mutual support is fine), not strictly asymmetric — a binary direction flag couldn't say that.
- **Inverse / converse** — so **backlinks read TYPED from both ends.** Today B's backlink panel shows a bare "← A"; with a declared converse, B reads "**caused-by** A". *Store one physical row* (owned by A's markdown — File-Over-App), *derive the converse on read* (SKOS `broader`/`narrower`: assert one direction, infer the inverse). No dual-write, no consistency race — the reverse query already exists for backlinks. For exemplifies↔generalizes (already two forward types), `inverseOf` makes "A generalizes B" and "B exemplifies A" the *same edge*.
- **Transitivity** — so chains become *inference*. Enable closure **on read** (recursive CTE) for `derives-from` / `supersedes` (true lineages), the abstraction pair (domain-guarded), and the thematic transitives (§3). **Hard caveat:** `part-of` is transitive *only within one* of the six Winston/Chaffin/Herrmann meronymy subtypes (component / member / portion / stuff / feature / place); a blanket `part-of` is **non-transitive** (the finger→musician→orchestra trap). The six subtypes are the canonical use for the registry's one-level custom nesting under the `part-of` seed.
- **Cardinality (functional)** — `supersedes` is the natural functional/one-to-one relation → a clean version *lineage* (v3→v2→v1) instead of a tangle. Enforced as a *soft warning* ("two notes both supersede X — which is canonical?"), never a hard block.
- **Reflexive** — nearly everything is **irreflexive** (a note does not support/contradict/supersede *itself*); the engine rejects `A part-of A`. Only the untyped link tolerates reflexivity (the question can point anywhere).

> **Closure is the one named exception to Rule 8.** Base edges are maintained write-time (already true). Transitive *closure* is **too volatile to persist** — materialising graph reachability is exactly the shape that OOMed and needed a 3 GB WAL vacuum (the LL-XXX incident). So closure is a **thin bounded recursive query over the always-current base** (the FTS5-vocabulary pattern), never a stored, trigger-maintained surface.

> **Hierarchies in Constellation — four kinds, one machinery (cross-ref PJ-065).** The transitivity above is the *engine of hierarchy*, and Constellation keeps **four** hierarchy kinds deliberately separate (*unlike SKOS, which collapses is-a + part-of into one `broader`*): **(1) taxonomy / is-a** — `generalizes` / `exemplifies`; **(2) meronymy / part-whole** — `part-of` (+ its 6 subtypes); **(3) lineage** — `derives-from` / `supersedes` — all three **cognitive/epistemic**, handled here in §2 — and **(4) the compositional / structural outline** — the parent/TOC link, which is **structural** (the authored shape of a work, exempt from the living-link apparatus) and lives in **PJ-065**. Kinds 1–3 differ from kind 4 in *meaning*, but **all four share the same mechanics**: closure-on-read (recursive CTE), the write-time acyclicity guard, inverse/converse, and ordering. **Build the hierarchy primitive ONCE** — the §2 closure engine here and PJ-065's structural tree are two consumers of it, not two implementations.

**Open ruling D-A — silent engine, or surfaced vocabulary?** The dimension model's *concept* is sound either way, but its *form* changes what it's for: **(A) silent** — characteristics drive inference invisibly (typed backlinks just appear correct; "the app is just a window"); **(B) surfaced** — the characteristics become a *teaching surface* (creating a type shows "causes: directional, doesn't chain, can't point at itself"; custom types declare their own). This is a Concept-Before-Function fork the Boss should rule before the Architect.

---

## 3. Layer B — FAMILIES, charted by the Five Acts

The eight cover **Observation** (the note itself) and most of **Connection** and **Conviction**; the gaps cluster — tellingly — on **Connection's thematic half, Tension, and Synthesis.**

### 3.1 The CONNECTION gap — the Thematic / functional family

Constellation is almost entirely **taxonomic + logical** (exemplifies/generalizes/part-of = kind/level/composition; supports/contradicts/causes/derives-from/supersedes = proof/tension/provenance). The whole **thematic** half of human relating — relations of *participation in a process/scenario*, neuro-dissociated from the taxonomic — collapses into the untyped `associative`. The thesaurus tradition long ago named these the Related-Term family. **Proposed additions (a second seed-family, each born declaring its §2 characteristics):**

| Proposed type | Horse (what it answers) | Characteristics |
|---|---|---|
| **used-for / uses** | "X is the instrument/method for Y" | asymmetric, non-transitive |
| **prerequisite-of** | "you must grasp X before Y" | asymmetric, **transitive** (dependency chains compose) |
| **precedes / follows** | "X comes before Y in time/sequence" | asymmetric, **transitive** (temporal order) |
| **near / co-occurs** | "X and Y share a scene/context" (pure RT) | **symmetric**, non-transitive |

This gives the **Connection act** a typed home *without emptying the untyped link*: `associative`-as-RT is "I've decided they're thematically related, full stop"; untyped-as-**question** (Concept Paper §4) is "I sense a link I can't yet name." The thematic types land *between* them — an *answered* functional connection. (`prerequisite-of`/`precedes` are the transitive members → they exercise the §2 closure engine, proving the dimension model pays off across families.)

### 3.2 The TENSION gap — argument-attack + questions (the thinnest act, priority #1)

Today the entire Tension vocabulary is one seed, `contradicts`, doing four jobs and missing a fifth. The argumentation literature (Pollock; ASPIC+/Walton; Toulmin; IBIS/QUD) resolves it precisely — and Constellation's **registry (children-under-a-seed) + confidence ladder** are the exact two mechanisms it needs.

**Add as CHILDREN of `contradicts`** (one-level nesting; a bare `contradicts` stays valid = the user who just feels "these clash" isn't forced to specify — same generosity as the untyped link):
- **undermines** — *horse:* "the attack lands on a **premise** the target rests on, not its thesis — so the conclusion may still stand once the bad leg is removed." (ASPIC+ undermining.)
- **undercuts** — *horse:* "the attack severs the **inference/warrant** — neither premises nor conclusion are denied, only the bridge between them — Pollock's most surgical, most-missed defeater." (Asymmetric/directional, unlike the symmetric parent.)
- *(rebut = the default meaning of a bare `contradicts` — a clash of claims — so it needs no separate child; document that bare contradicts = rebut.)*

**Add as a new top-level WEAKENER:**
- **qualifies / limits** — *horse:* "say 'true, but **only when…**' — a gradable scope-bound that **preserves** the target inside a boundary; the most common real move that is neither agreement nor contradiction." (Toulmin's qualifier — a *distinct part* from rebuttal in his own model.)

**Model via CONFIDENCE, NOT a new type:**
- **refutes** — a decisive defeat (Walton) is a `contradicts`/undermine/undercut that *won*. That is **degree**, and Constellation already has the degree axis: `confidence=established` on a contradicts *is* a refutation; `confidence=contested` is "a rebuttal that hasn't won." A `refutes` type would duplicate confidence and force the author to name an **outcome** (who won) at write time — which they cannot know.

**Add as the new INTERROGATIVE FAMILY (R4 = FAMILY) — the Question relations (the Tension→Synthesis loop):**
- **problematizes / raises-question** ⇄ inverse **answers / responds-to** — *horse:* "let a note pose a **question about** another (not just oppose it), and let another note **answer** it — turning Tension from a dead-end clash into the loop that drives Synthesis." (IBIS Issue-as-unit; QUD: each unit either answers a question or raises one that helps.) Distinct from Concept-Paper §4's *whole-note* untyped question: this is a note-to-note **interrogative edge**.

> **✅ R4 RULED — FAMILY (Eisa 2026-06-27, via the three-defence wargame).** `problematizes / answers` enters as its **own first-class *interrogative* family** — NOT among the canonical 8 (which stay the *answer/declaration* acts, §7 order **frozen**) and NOT discharged by the untyped link. *Why:* the question is **interrogative in mood** — categorically different from the 8 *declarations* (§4 defines the 8 as "all declarations / answers"), so it earns its own home rather than falsifying that definition (the NINTH option) or being left untyped (UNTYPED). It carves a real joint the untyped link and `contradicts` leave open: **`problematizes`** = *directed, verdict-free doubt* (between the unnamed untyped pull and the `contradicts` verdict); **`answers`** = the *resolution of a question* (which the stance-8 cannot express for a question-target). The three `contradicts` children, `qualifies`, and refutes-via-confidence remain registry-/confidence-native. **Guardrail (carried to the Architect):** scope `answers` to *resolving a question* — never a synonym for `supports`/`contradicts` on a claim; and the untyped link stays the unchanged **default** open question (typing is opt-in — UNTYPED's "don't force filing" concern is met). The interrogative family is the **engine of Tension** and the bridge to **Synthesis**. (Wargame: `PJ-067-R4-Wargame-Question-Relation.md`.)

### 3.3 The SYNTHESIS gap — analogy + n-ary co-completion (thinnest act, priority #2)

**analogy → `maps-to`** — *horse:* "assert that a **system of relations** holding in one note (the base) holds also in another (the target) despite their objects differing — and thereby **generate a new conjecture** about the target." It is the one link that connects by *deep structure* where surface similarity finds nothing — **Synthesis made into an edge** (Gentner's Structure-Mapping: analogy maps *relations*, not attributes). It is the natural *resolution* of an `associative` open-question when the shared relational structure becomes nameable, and the **best-fit case for the confidence ladder** (a candidate inference is *born* `hypothesis` and matures as the target-side prediction is confirmed) and for **weight/decay** (an analogy walked earns weight; one abandoned decays — the canonical "living link"). *Ships pairwise now* (a directional, non-symmetric `maps-to` row; the correspondence rides in the existing Annotation) — but its **mature form is the synthesis node** (§4), which it *shares* with n-ary co-completion.

**n-ary co-completion → the synthesis node** — see §4 (its own section; it is the architectural leap).

---

## 4. The N-ary Synthesis Node — the architectural leap (the `complements` reframe)

**The problem the binary edge cannot solve.** "Notes A, B, C — *taken together, and only together* — constitute idea X." Four independent edges A→X, B→X, C→X **lose the conjunction**: they assert separate part-of-ish relations, none carrying the truth that the idea exists *in their joint configuration* and would collapse if one were removed. And the synthesis is an emergent object with **its own confidence** ("I'm at *evidence* on this synthesis, though constituent A is *established*") and **its own weight** (it gets traversed, earns weight, decays independently of its parts). A fan-in of binary edges has nowhere to put any of that.

**The settled pattern (four traditions, one conclusion):** when a relation has >2 participants *or* properties of its own, give the relation an **identity** (a node) and attach participants + properties to it.
- **W3C "Defining N-ary Relations" Pattern 1** — *"create a new class/individual for the relation … attributes attach to the relation individual, not to either participant."* Its motivating case is a diagnosis carrying a *probability* — the exact shape of a synthesis carrying *confidence*. (Pattern 2 — an ordered list — is the opt-in refinement for a *sequential* synthesis, e.g. a derivation chain.)
- **Neo-Davidsonian event semantics** — "Brutus stabbed Caesar with a knife in the forum" is not a 4-place predicate but a reified **event** `e` with thematic-role participants. **A synthesis is a cognitive *event* — the act of seeing N things cohere — so its logically honest form *is* a reified node with role-edges in.** (Classic RDF reification is the weaker cousin: it annotates *one* binary triple, the wrong cardinality.)

**The Constellation model (recommended).** Constellation already reifies a link as a *first-class file*; the v2 move is to let that object have **N participants instead of 2** — a new **`SYNTH` note-kind** (`YYYYMMDDTHHMMSSZ_SYNTH_XXXX.md`, a sibling of the existing file kinds). The N constituents point into it via a typed link; **confidence + weight live on the SYNTH node** (per W3C's motivation). Reuses `note_links` unchanged + one new type; no schema upheaval. **This is shared infrastructure with analogy's Tier 2** (the reified relation-node that carries `maps-to`'s correspondence-set + candidate inferences) — *build the node once; analogy and co-completion are two instances.*

**The name — `complements` → `co-completes` / `jointly-constitutes`.** Two load-bearing reasons (the Typology Research's flag, sourced to Cruse 1986):
1. **"Complementarity" means the OPPOSITE of what we mean.** Lexically it is *mutually-exclusive opposition* — dead/alive, true/false, odd/even (two meanings that exhaustively divide a domain). Ours is two (or N) notes that **complete each other into a synthesis** (jigsaw pieces). Same word, opposite spirit. **Do not name it "complementarity."**
2. It is **not pairwise-symmetric at all — it is n-ary.** "Together they form an idea" is the synthesis-node relation, not an edge. So the name *and* the structure move together: **`co-completes` / `jointly-constitutes`, realized as a synthesis node.**

**Open ruling N — who authors the constituency?** **(A) from the constituent** (while reading note A, mark "A `jointly-constitutes` →" and pick/create the synthesis — formulation-flow-native, matches how links are born today, but the synthesis *body* is written later) vs **(B) from the synthesis** (create a SYNTH note, write the idea, then name constituents in frontmatter — idea-first, but a "go-make-the-container" interruption). *Recommendation: (A) primary, (B) available* — synthesis is usually *discovered while inside a constituent*. This shapes how the Synthesis act *feels*; the Boss rules it before a Plan.

---

## 5. The Five Acts mapping (the spine — where v2 spends its effort)

| Act | Covered today | v2 adds | Thinness |
|---|---|---|---|
| **Observation** | the note itself; facts rest (§5) | — | full |
| **Connection** | taxonomic/logical 8; untyped = the question | **Thematic family** (used-for, prerequisite-of, precedes, near); **inverse labels** (typed backlinks both ends) | partial |
| **Tension** | `contradicts` (one type, 4 jobs) | **undermines/undercuts** (children), **qualifies**, **problematizes/answers** (the loop), refutes-via-confidence | **THINNEST → priority #1** |
| **Synthesis** | — (essentially none) | **analogy `maps-to`**, **n-ary `co-completes`** (the synthesis node) | **THINNEST → priority #2** |
| **Conviction** | confidence + weight already carry it | (refutes-via-confidence sharpens it) | full |

> The thin acts — **Tension** (no premise/inference attacks, no questions) and **Synthesis** (no analogy, no n-ary) — are *exactly* the uncharted territory. For a **formulation** (not management) tool, that is the gap that matters most. **v2 should be sequenced Tension-first, then Synthesis** — the dimension engine (§2) underpinning both.

---

## 6. The two load-bearing flags (called out, per the Boss's brief)

1. **`complements` → `co-completes` / `jointly-constitutes`** — "complementarity" is the lexical *opposite* (mutually-exclusive opposition); rename, and realize it as **n-ary**, not a pairwise symmetric link (§4).
2. **The n-ary synthesis node** — the biggest architectural leap (a `SYNTH` note-kind; confidence/weight on the node; W3C Pattern 1 + Davidsonian); **shared with analogy's mature form** — build the primitive once (§4).

---

## 7. The decisions to ratify (the vocabulary is Boss-defined)

- **R1 — The dimension model (§2):** adopt a per-type `characteristics` declaration in the Registry (symmetry/transitive/inverse/functional/reflexive). *Sub-ruling D-A:* **silent engine** or **surfaced (teaching) vocabulary**?
- **R2 — Inverse/converse labels (§2):** adopt the converse-label set; **store one direction, derive the reverse on read** (no dual-write). → typed backlinks both ends.
- **R3 — The Thematic family (§3.1):** add `used-for` / `prerequisite-of` / `precedes` / `near` (the Connection act) — *yes/no/which*.
- **R4 — The Tension vocabulary (§3.2): ✅ RULED (Eisa 2026-06-27, via the wargame).** Add `undermines`/`undercuts` (children of contradicts) + `qualifies`; model `refutes` via confidence. **THE GRAMMAR RULING: `problematizes / answers` = its own first-class INTERROGATIVE FAMILY** — NOT a 9th cognitive act, NOT left untyped. The canonical 8 (the answer acts) + the §7 order stay **frozen**; questions get a typed home as a distinct *mood*; the untyped link stays the default open question. (See `PJ-067-R4-Wargame-Question-Relation.md`.)
- **R5 — The Synthesis frontier (§3.3 + §4):** add analogy **`maps-to`** (pairwise now); adopt the **n-ary synthesis node** (`SYNTH` kind) for **`co-completes`** *and* analogy's mature form — *who authors* (ruling N).
- **R6 — Phasing:** this is **`/migration`-scale, and multiple** — recommend sequencing **dimension engine (§2) → Tension (§3.2) → Synthesis (§3.3/§4) → Thematic (§3.1)**, each its own four-phase `/migration`, each proven on the 7,600-note universe (no boot/typing/IPC regression).

---

## 8. Scope, risk, and the ask

- **Scale:** several `/migration`s (registry schema + characteristics; new types + the children-nesting; the `SYNTH` note-kind + the n-ary write/read; the inverse-rendering of backlinks; closure-on-read). It crosses Rust↔Svelte↔schema↔write/read → the four-phase workflow, each phase Boss-approved.
- **Reuses the proven, safe paths:** the existing `note_links` + dual-source frontmatter fold (MIG-086), the registry's children-nesting (MIG-067), the confidence/weight columns, the backlinks reverse query, the file-kind grammar. The genuinely new primitives: a **per-type characteristics declaration** and a **reified synthesis node (`SYNTH` kind)**.
- **Honesty flags carried** (from the research): `part-of` is 6 meronymy subtypes (transitive only within one); refute-vs-rebut is real but "not firm in everyday usage" (→ model via confidence); the gIBIS extended link-names couldn't be primary-sourced (not canonical).

**The ask (Boss ratification):**
1. **Ratify the model's shape** (§1): a link has declarable *structure* + charted *meaning*, prioritized by the Five Acts (Tension + Synthesis first).
2. **Rule R1–R6** — especially **R4's grammar ruling** (does the canonical "8" grow?), **R5's synthesis-node** + who-authors, and the **`co-completes` rename**.
3. On ratification, I open the **first `/migration` Architect** (the dimension engine, or Tension — Boss's call on R6). **Until then: build nothing.**

---

*End of PJ-067 Concept Paper v1.0 (draft). Sources: Living-Link-Concept-Paper-v1.0.md (RATIFIED §4–§7); Living-Link-Relationship-Typology-Research-2026-06-24.md; frontier workflows wf_e6c2dade-bed / wf_bae71cca-fd1 — primary sources: W3C "Defining N-ary Relations" (Pattern 1; diagnosis-probability); Davidson / neo-Davidsonian event semantics (Parsons, Landman); OWL 2 property characteristics (W3C; Protégé); SKOS broader/narrower inverse + assert-one-infer-reverse (W3C); Pollock 1987/1995 + ASPIC+ (Modgil & Prakken) rebut/undermine/undercut; Toulmin qualifier; IBIS (Rittel) + QUD (Roberts 1996); Walton refute-vs-rebut; Gentner 1983 Structure-Mapping (+ Northwestern QRG SME); Winston/Chaffin/Herrmann 1987 (6 meronymy subtypes); Cruse 1986 (complementarity = mutually-exclusive opposition); taxonomic/thematic dissociation.*
