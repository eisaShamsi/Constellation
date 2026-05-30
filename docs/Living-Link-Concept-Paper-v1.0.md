# Constellation — The Living Link — Concept Paper

**Version 1.0 | 2026-05-30**
**Author of facts:** Eisa ALSHAMSI (project owner, designer, IT Boss)
**Maintainer:** Claude (consultant / engineer / SME)
**Status:** Foundation agreed in the 2026-05-30 dialogue; this is the written form for ratification. On sign-off it becomes the canonical source for link-type *semantics and order* — the scattered orderings in code and docs reconcile to it.

> This paper answers **why** the Living Link exists before it touches **how** it's ordered or built. The practical mechanics live in [`Living-Links-Guide-v1.0.md`](Living-Links-Guide-v1.0.md); the broader philosophy in [`CONSTELLATION-KNOWLEDGE-FORMULATION.md`](CONSTELLATION-KNOWLEDGE-FORMULATION.md). This sits between them: the *concept*.

---

## 1. Thesis — knowledge is formed at the connections, and the connection must be alive

Every other tool begins from a hidden assumption: **knowledge is stuff you store, and a link is a string that points from one piece of stuff to another.** Constellation rejects this at the root. Its claim:

> **Knowledge is not stored — it is *formed* — and formation happens at the connections, not inside the notes.**

A note, alone, is an observation: inert. The moment you assert *this supports that*, *this contradicts that*, *this is caused by that*, a **cognitive act has occurred in the link**. The link therefore cannot be a dumb pointer. It is the place where the thinking is recorded — so it must carry the **state of an understanding**: what *kind* of move it is, how *settled* it is, how *load-bearing* it has become, and where it sits in its *life*.

The spec's ladder is the whole continuum in four lines:

> A note without links is an **observation**.
> A note with links is **knowledge**.
> A network of *typed* links is **understanding**.
> Understanding that survives contradiction is **wisdom**.

That is not four boxes. It is a single arc a piece of understanding travels — and the Living Link is the **vehicle** that carries it, with four state-properties as the **odometer**: **Type** (which cognitive act), **Confidence** (how settled), **Weight** (how load-bearing, earned through use), **Lifecycle** (where in its life).

---

## 2. The lineage — Constellation is an heir, not an inventor

The framework is not improvised; it is the oldest and truest account of how understanding grows. Naming the lineage is intellectual honesty, and it makes the design defensible against any critic.

- **C.S. Peirce, *The Fixation of Belief* (1877).** Inquiry is the struggle to replace **doubt** with settled belief. Peirce distinguishes **genuine doubt** — which "arises from surprise, conflict of ideas, or recalcitrant experience" — from **"paper doubt,"** the artificial, self-imposed kind (his charge against Descartes's universal skepticism: a *sham* inquiry). He names four ways people fix belief — **Tenacity** (ignore contrary evidence), **Authority**, the **A Priori** (elegance), and **Science** (self-correcting) — and only the last, which *welcomes* refutation, holds up over time. *(Most PKM tools quietly enable Tenacity: pile up confirming links.)* ([source](https://philarchive.org/rec/PEITFO))
- **Karl Popper, *Conjectures and Refutations*.** Knowledge advances not by *confirming* a hypothesis but by *trying to refute* it: "the very refutation of a theory is always a step forward that takes us nearer the truth." Fallibilism — all knowledge provisional. ([source](https://plato.stanford.edu/entries/popper/))
- **The Socratic method.** Knowledge through questioning until false certainty collapses.
- **Stephen Toulmin (1958)** typed the *parts* of an argument — claim, data, warrant, backing, qualifier, **rebuttal**. **Horst Rittel's IBIS (1970s)** organized knowledge around **Issues — questions — as the primary unit**, with positions and pro/con arguments. ([source](https://en.wikipedia.org/wiki/Issue-based_information_system))

Constellation is heir to all four. What it adds is in §3.

---

## 3. What is genuinely new — not the first to *type* a connection, the first to keep it *alive*

**Honest about prior art.** Typed relations are not unprecedented. Toulmin typed argument parts; IBIS typed pro/con; and **Roam Research's "Discourse Graph" extension** types relations (*supports / opposes / informs*) between first-class **Question / Claim / Evidence** nodes, queryable. So Constellation must **never** claim "no one types links." ([source](https://oasis-lab.gitbook.io/roamresearch-discourse-graph-extension/fundamentals/what-is-a-discourse-graph))

**The moat — concept vs. application.**

- *At the concept level*, what is Constellation's own: **none of them make the link *living*.** Every prior system is **static** — an argument map or discourse graph drawn today reads identically in three years. Not one has **weight earned through traversal**, **decay without use**, or a **lifecycle**. That temporal, self-maintaining layer is the operationalization of the author's own principle: *"Without ongoing thought, I will not find truth through knowing"* — which is not a slogan but `effectiveLinkWeight()` decaying a connection you have stopped walking. A discourse graph is **a better filing cabinet for arguments**; the Living Link is **a circulatory system that goes stale without thought**.
- *At the application level*, what stands alone is **Constellation itself**: the living link is **native, ambient, and simple by default** — not a specialist plugin you opt into for a literature review, but the connective tissue of a complete, **local-first, file-over-app, multilingual** knowledge instrument (Sky View, 360.3D, Sight, the diagnostic search). Discourse Graph asks you to formally cast everything as Question/Claim/Evidence; Constellation lets **facts rest** (§5) and invites structure only where thinking is actually happening.

> **The defensible claim:** Constellation is not the first to *type* a connection — it is the first to keep it *alive*; and as an *application*, the first to make that aliveness the ambient, simple-by-default fabric of a whole local-first knowledge instrument.

---

## 4. The Question is the Key — and the untyped link *is* the question

The engine of the continuum is not accumulation; it is **inquiry**. Knowledge does not flow on its own — it is **driven by questioning**, and the sharpest question is the one you aim at your *own* formulation: *"wait — is this even right?"* Doubting and contradicting what you have formulated is the **natural way of thinking**, and it is how you reach territory you had not imagined.

This forces a reframe of the **untyped link**. The eight typed names are all **declarations** — *X supports Y*, *X contradicts Y*. They record the **answers**. But the generative moment — the connection you *sense* but cannot yet name — is a **question**, not an assertion. We have a home for it we have been mislabeling: the untyped `[[wikilink]]`. Calling it "incomplete — upgrade it" is backwards.

> **The untyped link is the question itself** — *"I feel these belong together, but I'm still asking how."* It is not backlog to clear; it is the **live edge of thinking** — the open inquiry. The most *alive* link in a library may be the one not yet answered.

(This is IBIS's "Issue" arrived at independently: the question as the primary unit.) Typed links are therefore **how an inquiry resolves** — or honestly refuses to.

---

## 5. Facts rest, formulations inquire — the guardrail against a tyranny of doubt

Doubt is the engine, but it does not run on everything. **A simple fact is the substrate, not a claim under argument.** *"The meeting is at 3"; "water boils at 100 °C at sea level"* — these have no position to challenge, and manufacturing doubt about them is precisely Peirce's **paper doubt**: empty motion. The spec's first rung already says it — *"a note without links is an observation"* — and an observation resting unlinked and unchallenged is **complete, not deficient.** The ladder is **available, never mandatory.**

The architecture already encodes this: the **stratum** axis (L1 **Datum** → L8 **Worldview**) *is* the line along which "needs challenge" rises. A Datum is a fact at rest; a Worldview is your most formulated, most contestable stance.

> **The rule, stated plainly:** *"A library of only `supports` has stopped thinking"* applies to **claims, never to facts.** The system **invites** challenge into formulations; it must **never nag** a fact-note for having no `contradicts` and no links. Challenge is an **invitation**, never a **guilt**.

**Design consequence:** blind-spot / missing-challenge signals stay scoped to the *formulation* layer (higher strata, claim-notes). Facts are never flagged as lacking.

---

## 6. The eight cognitive acts

Each typed link is a **distinct cognitive act** — a different way one idea can stand in relation to another. They are not a ranked hierarchy; they are a **vocabulary of inquiry**.

| Act | What it asserts | Role in inquiry |
|---|---|---|
| **supports** | this note *strengthens* another's claim | building / corroboration |
| **contradicts** | this note *disputes* another's claim | **the engine** — doubt, tension, intellectual honesty |
| **causes** | A → B, a directional cause-and-effect | explanation / mechanism |
| **exemplifies** | this note is an *instance* of a broader idea | abstraction ↓ (concrete) |
| **generalizes** | this note *abstracts upward* from another | abstraction ↑ (synthesis) — inverse of exemplifies |
| **derives-from** | this note's reasoning is *based on* another | provenance / lineage (trust-depth) |
| **part-of** | this note is a *component* of a larger whole | structural composition |
| **supersedes** | this note *replaces an earlier stance* | succession — the outcome of completed inquiry |

`associative` is the null/untyped synonym — not a semantic act; it round-trips without surfacing. Untyped (the open question, §4) is rendered in its own right, never as a deficiency.

---

## 7. The canonical order — *derived*, not chosen

The order must fall out of the **inquiry arc**, not from which file happened to list them which way. Clustering the eight acts by their function:

1. **Stance** — the two ways to position a claim: it strengthens or it disputes → **supports · contradicts**
2. **Explanation** — the mechanism that makes it hold → **causes**
3. **Abstraction** — movement across levels of generality (the inverse pair, kept adjacent, in the synthesis direction concrete→abstract) → **exemplifies · generalizes**
4. **Lineage** — where the reasoning descends from → **derives-from**
5. **Composition** — how it sits in a structure → **part-of**
6. **Succession** — the new understanding that retires the old → **supersedes**

This yields the canonical order:

> ### supports · contradicts · causes · exemplifies · generalizes · derives-from · part-of · supersedes

The arc moves from the **immediate epistemic stance** (do I affirm or deny?) outward through explanation, abstraction, lineage, and composition, ending at **succession** — the terminus where inquiry concludes in a new settled stance. `supersedes` is therefore **last** on principle, not merely because it was added last.

**This resolves the existing drift.** The derived order matches the **Knowledge-Formulation spec (Part II.4)** and the **backend `KNOWN_LINK_TYPES`** — so those are confirmed canonical. The **Living-Links Guide §2** and the **360.3D matrix `TYPE_ORDER`** use a different middle (`derives-from · generalizes · exemplifies`); that camp's only rationale is grouping the two "origin-tracing" relations (`causes`, `derives-from`) together — coherent, but it breaks the abstraction pair's natural place in the synthesis direction and diverges from the foundational spec. **They are drift, to be reconciled to the order above.**

---

## 8. Design implications

- **The Base (MIG-066).** A note's **Link types** column lists its outgoing typed acts in the canonical order above; **rank-aware sort** orders by the top-ranked (lowest-index) act present. Untyped is surfaced as an **open-question** state, not a deficiency. Facts (low stratum, no claims) are never flagged for missing types.
- **Reconciliation (follow-up).** Update `Living-Links-Guide-v1.0.md` §2, the `Inspector360.svelte` `TYPE_ORDER`, and `inspector360.rs` `ALL_LINK_TYPES` to the canonical order, and add `supersedes` to the 360.3D matrix (it is parsed but not yet displayed there). Bump the Guide to v1.1 in the same commit.
- **Blind-spot signals** stay scoped to the formulation layer (§5).

---

## 9. What this governs + open items

On ratification, this paper is the **single source of truth** for link-type semantics and order; CLAUDE.md, the Guide, the spec, and the code defer to it. Open items: (a) Eisa's sign-off on the written form and the derived order; (b) the reconciliation follow-up (§8); (c) for full rigor, a deeper verification that no shipping tool carries the *living/decay* dimension, and a fuller argumentation-theory grounding (RST, Walton's argumentation schemes) before any external-facing claim.

**Sources:** [Peirce — Fixation of Belief](https://philarchive.org/rec/PEITFO) · [Popper (Stanford Encyclopedia)](https://plato.stanford.edu/entries/popper/) · [IBIS — Rittel](https://en.wikipedia.org/wiki/Issue-based_information_system) · [Roam Discourse Graph](https://oasis-lab.gitbook.io/roamresearch-discourse-graph-extension/fundamentals/what-is-a-discourse-graph)
