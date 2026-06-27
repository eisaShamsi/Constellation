# PJ-065 — The Structural (Parent / Table-of-Contents) Link Type — Concept Paper

**Version 1.0 (draft for ratification) | 2026-06-27**
**Author of the concept:** Eisa ALSHAMSI (project owner / designer / IT Boss) — conceived 2026-06-24
**Maintainer / synthesis:** Claude (consultant / engineer / SME)
**Status:** **CONCEPT RATIFIED — Eisa 2026-06-27.** The *horse* (§1–§2: the structural link is the compositional spine, a distinct **kind** from the 8 cognitive acts, exempt from the inquiry/decay apparatus) and the **design direction** (the §3 prior-art pattern + the §4 recommendations — allow multi-parent with one primary for the spine; MVP = TOC panel + breadcrumb, deferring compile/export + roles/rollups) are approved. **The precise NAME (D3) and the remaining load-bearing decisions are finalized at the `/migration` Architect phase, where the Boss rules the vocabulary.** Building has NOT started — it follows only when the Boss greenlights the `/migration` (this is `/migration`-scale). Concept-Before-Function satisfied: the horse is stated and ratified before any carriage.

> This paper answers **why** a structural parent/TOC relationship should exist, and **how it differs in kind** from the eight cognitive Living Links — *before* it touches storage, schema, or UI. It is the input the Boss ratifies (the cognitive/structural vocabulary is Boss-defined). Grounded in: the Living Link Concept Paper v1.0 (RATIFIED), the Relationship Typology Research (2026-06-24), the MIG-086 frontmatter fold, and a 4-front prior-art dig (Constellation codebase · PKM tools · formal KR standards · long-form authoring tools).

---

## 1. The horse — the concept, in one sound sentence

> **The structural link is the *compositional spine* of a work. Where the eight cognitive Living Links answer *"how does this idea relate to that idea?"* (the vocabulary of inquiry), the structural link answers a categorically different question: *"What is the ordered shape of the work I am composing from these notes?"* It is the table of contents, the outline, the binder — the deliberate, *ordered* hierarchy by which a constellation of formulated ideas becomes a navigable, composable, exportable *work* (a book, a screenplay, a course, a Map of Content).**

It is **structural** (the skeleton of an artifact), not **epistemic** (a claim about meaning). That single distinction — proven independently by all four prior-art fronts — is what earns it its own *kind*, and it drives every design decision below.

The headline use-case the Boss named: **authors and screenwriters** laying chapters / scenes / acts under a work; and more broadly, **Map-of-Content / outline hierarchies**. "The sky is the limit."

---

## 2. Why it is NOT a ninth cognitive act — and NOT the existing `part-of`

The Living Link Concept Paper (RATIFIED §6) is explicit: the eight typed links are **cognitive acts** — "a vocabulary of inquiry," each "a distinct way one idea can stand in relation to another." `part-of` (seed #7) is the **meronymy / composition** act: *"this **idea** is a component of a larger **idea/whole**"* (premise part-of an argument; finger part-of a hand). It is a **claim about conceptual structure**, held under inquiry (it can carry `confidence: hypothesis→established`, earn `weight`, decay without use).

The structural link is different on **three axes that the cognitive `part-of` does not have** (codebase + authoring + KR studies all converge here):

| | The 8 cognitive acts (incl. `part-of`) | The structural link (PJ-065) |
|---|---|---|
| **Question answered** | *How do these ideas relate epistemically?* | *In what order, under what whole, does my work read?* |
| **Order** | None — links are unordered among themselves | **Ordered** — a TOC *is* an ordering (Chapter 1, 2, 3) |
| **Shape** | Many-to-many graph, cycles allowed | A **tree / DAG** — the spine of a work, acyclic |
| **Settled by** | **Inquiry** — asserted, doubted, refined; *facts rest, formulations inquire* (§5) | **Authorship** — the author *decides* the structure; it is not a claim to be challenged |
| **Living apparatus** | Weight (earned), Confidence (4 levels), Decay (without use), Lifecycle | **None of these apply** — order + role, not epistemic weight; structure does not "go stale without thought" |
| **Cognitive topology** | Feeds stratum, maturity, 360 gap-analysis, Knowledge-Health | **Should be excluded** — a TOC edge must not inflate a note's perceived *cognitive* connectedness |

> **The decisive line (authoring study, verbatim):** *"That concept is orthogonal to the 8 cognitive link types ('how do these ideas relate epistemically?'), which is exactly why it earns its own structural link type rather than reusing `part-of`."*

This is **Form-Aligns-To-Purpose**: the structural relation has degrees of freedom (order, tree-shape) the cognitive primitive does not afford, and the cognitive relation carries an apparatus (decay, doubt) the structural one must not. Forcing one into the other would be exactly the kind of dimension-mismatch that principle forbids.

**Where it sits in the philosophy.** Knowledge Formulation is *"what can I BUILD from what I know?"*. The eight cognitive links are how understanding is **formed**; the structural link is how formed understanding is **composed into a work** — the **Externalization / Expression** face of the Cognitive Engine (CE element #6), the last mile from *constellation of ideas* → *finished artifact*. It is downstream of Conviction, not a new act of inquiry.

### 2.1 Hierarchies in Constellation — where this one sits (four kinds, one machinery)

"Hierarchy" is not one relation. Constellation deliberately keeps **four** separate — *unlike SKOS, which collapses is-a + part-of into a single `broader`* (good thesaurus practice splits them; Constellation does too):

1. **Taxonomy (is-a / class-inclusion)** — `generalizes` / `exemplifies` — *cognitive* (PJ-067 §2).
2. **Meronymy (part-whole)** — `part-of` + its 6 Winston/Chaffin/Herrmann subtypes — *cognitive* (PJ-067 §2).
3. **Lineage (derivation / version)** — `derives-from` / `supersedes` — *cognitive* (PJ-067 §2).
4. **Compositional (the work's ordered outline)** — **this paper's structural parent/TOC link.**

Kinds 1–3 are **epistemic** (claims about how *ideas* relate; held under inquiry; carry the living-link apparatus — weight/confidence/decay). Kind 4 is **structural** (the authored *shape of a work*; settled by authorship; exempt from that apparatus — §2 above). They differ in **meaning** but share the **same mechanics**: **transitive closure computed on read** (recursive CTE, never stored — the LL-XXX rule), a **write-time acyclicity guard**, **inverse/converse** (parent↔child / broader↔narrower), and **ordering**. **Build that hierarchy primitive ONCE; the four kinds are its instances** — the closure / acyclicity / inverse machinery this paper specifies (§3–§4) is the same PJ-067 §2 needs for kinds 1–3.

---

## 3. Prior art (WA#5) — the battle-tested pattern, and where Constellation already half-built it

Every mature system that models hierarchy converges on the **same four-part pattern**. Constellation should adopt it rather than invent.

1. **Assert ONE direction; derive the reverse.** SKOS `skos:broader` `owl:inverseOf` `skos:narrower`; Dublin Core `isPartOf`/`hasPart`; schema.org `isPartOf`/`hasPart`; Obsidian **Breadcrumbs** (declare `up:`, it auto-implies `down`/child + siblings). The reverse is **never** a second hand-maintained fact — one source of truth, no drift. **Cautionary tales:** Tana's #1 community request is literally "Bidirectional Fields" because it *didn't* auto-imply the reverse; Logseq issue #10250 — a one-sided declaration the parent didn't reflect produced *empty parent hierarchies*. **If you derive direction from one side, you MUST materialize the reverse write-time, or the TOC silently breaks.**

2. **Order is separate from the containment edge.** SKOS uses a separate ordered `skos:memberList`; schema.org puts a `position` integer on each child; Dublin Core has `tableOfContents`; Breadcrumbs uses a manual `order` field (beats alphabetical); Dendron `nav_order`. **The containment predicate itself carries no order — a per-child sort-key does.** Long-form tools (Scrivener, Workflowy) use **fractional / rebalanceable ranks** so "insert between" and drag-reorder never renumber all siblings.

3. **Store DIRECT edges; compute closure separately.** SKOS splits asserted `broader` (not transitive) from inferred `broaderTransitive` (`owl:TransitiveProperty`). Store only the immediate parent/child; compute *ancestors/descendants* with a recursive query on demand. **Two read APIs, never conflated:** `get_direct_children` vs `get_all_descendants`. (Tana warns its recursive `COMPONENTS REC` "runs slower and slower as connections grow" — validating Constellation's **Rule 8 / Write-Time Derivation**: the child list is a persisted, trigger-maintained view, never graph-walked on panel open.)

4. **Acyclic by construction.** A containment hierarchy is a **strict partial order** — **asymmetric** (A parent-of B ⇒ not B parent-of A) and **irreflexive** (nothing is its own parent). *OWL-DL cannot co-assert transitive + asymmetric* (decidability) — which is *why* SKOS splits the properties. **Constellation is not an OWL reasoner, so it sidesteps the limit entirely:** enforce acyclicity with a **write-time recursive-CTE cycle check** (reject an edge whose target is already a descendant of the source) *and* compute transitive closure on read — getting *both* guarantees OWL-DL can't.

**Long-form authoring adds three things the spine wants (likely a later layer):** children carry a **role** (Part/Chapter/Scene/Beat — defaultable by depth, overridable; Scrivener "Section Types", Save-the-Cat's 15 named beats); parents show **rollups** (descendant word-count / draft-status, aggregated write-time up the ancestor chain); and every read surface (printed TOC, **Compile/export**, Outliner, Corkboard) is a **projection of the one ordered tree** — *store the structure once, render many views.*

**What Constellation already has (codebase study):**
- The **dual-source frontmatter fold** (MIG-086 §F) already lets a *type-as-property* (`parent:\n  - "[[X]]"`) be derived into `note_links` write-time — the exact authoring path a structural type would ride.
- The **Link-Type Registry** (MIG-067) already gives any type a color, label, 15-locale i18n, pill rendering, and the parser hook — and supports custom types. **MIG-086 decision D5 already earmarked a Breadcrumbs-style auto-implied reverse for PJ-065.**
- **What's missing, and it's the whole game: ORDER.** `note_links` has **no order/position column**; the only order that exists today is the frontmatter YAML list's insertion order, and it is **lost** crossing into the index (no `seq`, no `ORDER BY`). **OrgChart** is the only surface that renders an ordered parent→children tree — but it reads **filesystem containment**, not `note_links`. There is **zero precedent** in the codebase for persisted sibling-ordering of links. This is the central thing PJ-065 must add.

---

## 4. The load-bearing design decisions (for Boss ratification — NOT decided here)

These are the questions the `/migration` Architect will answer *after* the Boss rules the concept. I surface each with the prior-art recommendation and the trade-off, but the cognitive/structural vocabulary is the Boss's to define.

- **D1 — Distinct *kind*, not a 9th cognitive act.** *Recommendation:* yes — a `structural: true` (non-cognitive) flag on the registry so it gets color/label/i18n/pill/parser/rename-cascade for free, but is **excluded** from the Five-Acts gap analysis (360 `missing_link_types`), strata/maturity, Knowledge-Health, and the living-link apparatus. *Boss rules:* is it one structural type, or a small family (e.g. `parent` + an optional `precedes` sequence — see D9)?

- **D2 — Storage: ride `note_links` + a `seq` column, vs a dedicated `note_toc`/`note_structure` table.** The PKM + KR studies lean *ride `note_links`* (cheapest; reuses the whole index + cascade + rename-safety; add a nullable `seq`). The authoring study leans *own table* (the single-parent + ordered + role constraints don't fit the free many-to-many `note_links` shape cleanly). *Trade-off:* reuse-and-extend vs purpose-built. *Recommendation:* start by extending `note_links` with `seq`; only split to a table if cardinality/role enforcement demands it.

- **D3 — Direction + declaration ergonomics + the implied reverse.** Two authoring shapes, both valid (Breadcrumbs supports both, compiling to one edge):
  - **(a) on the child:** `parent: "[[Work]]"` — the child knows its one parent (SKOS `broader` convention). *But assembling the parent's **ordered** children from backlinks loses order* (each child names its parent independently).
  - **(b) on the parent:** `contains:\n  - "[[Ch1]]"\n  - "[[Ch2]]"` — the YAML **list order = TOC order** (the MOC / list-note pattern). *This is where order lives naturally.*
  - *Recommendation:* support both, but treat the **parent's ordered list as the canonical home of order**; **materialize the reverse write-time** (hard requirement — the Tana/Logseq cautionary tales). *Boss rules:* the **name(s)** — candidates: `parent`/`contains`, `chapter-of`, `under`, `toc`. **Must not collide with the cognitive `part-of`** and should not reuse its neutral-gray `#AAAAAA`.

- **D4 — Ordering mechanism:** integer `seq` (simple) vs **fractional/rebalenceable rank** (O(1) insert-between, no sibling renumber — the Scrivener/Workflowy idiom; aligns with the ranked-dimensions memo). *Recommendation:* fractional rank for a smooth reorder UX.

- **D5 — Cardinality:** strict **single-parent tree** (authoring: a scene is in one chapter) vs **multi-parent DAG** (MOC: a note in several maps). *Recommendation:* allow multi-parent (MOC-friendly), but the spine/TOC view uses one **primary** parent; never let it fight or move the Universe→Library→Folder→Note **file** tree (which stays single-parent ownership — the structural link is a *cognitive overlay*, not a second folder system).

- **D6 — Acyclicity:** a **write-time recursive-CTE guard** (reject self-parent + reject an edge whose target is already a descendant). *Recommendation:* yes — in Rust, before the row lands, same transaction as `seq` assignment.

- **D7 — Direct vs closure:** store **direct edges only**; expose `get_direct_children/parent` *and* `get_all_descendants/ancestors` (recursive CTE) as **separate** read APIs. *Recommendation:* yes (SKOS pattern); materialize closure only if a 7,600-note measurement demands it.

- **D8 — No living-link apparatus on structural edges.** No decay, no confidence ladder, no weight-earned-through-traversal (those are for claims under inquiry; *facts rest, structure is authored*). *Recommendation:* yes — exempt; keep `order` + (later) `role`, drop epistemic weight.

- **D9 — Rendering surfaces (MVP scope):** the natural projections — **(1) a TOC/Outline panel** (ordered, collapsible, *zoom-to-subtree* à la Workflowy, virtualized per Rule 3); **(2) a breadcrumb/ancestors trail**; **(3)** later, a **Compile/Export** that depth-first concatenates the subtree (the author's payoff: "build the work"); **(4)** optionally a **next/prev reader** (a *separate* horizontal sequence axis — don't conflate with vertical containment). OrgChart today is filesystem-only; *Boss rules:* extend OrgChart with a links-derived-hierarchy mode, or a **new dedicated TOC panel**. *Recommendation:* new panel reusing Backlinks/Outgoing infra; MVP = panel (1) + trail (2).

- **D10 — Richer authoring layers (defer behind their own concept):** child **role** (Part/Chapter/Scene/Beat templates), **rollups** (descendant word-count/status, write-time up the chain), per-node **draft status/label**. *Recommendation:* MVP is the *ordered containment edge + its two views*; everything in D10 is "the sky is the limit" follow-on, each stated as a concept before it's built.

---

## 5. The Five Acts mapping (Form-Aligns-To-Purpose)

- **Observation → Connection → Tension → Synthesis → Conviction** are served by the eight **cognitive** links (and the untyped question).
- The **structural** link serves the **act after Conviction**: **Externalization / Expression** — composing settled understanding into an ordered, navigable, exportable *work*. It is the **carriage the formed ideas ride into the world**, not another act of forming them.

This is why it must stay *out of* the inquiry instruments (no blind-spot nagging, no decay, no maturity contribution): a chapter being "third under the book" is not a claim that can be *wrong* or *go stale* — it is a structural decision the author owns.

---

## 6. Scope, risk, and the ask

- **Scale:** `/migration` (schema: new type + `seq` + acyclicity guard; write path: frontmatter fold + reverse materialization + rename cascade; read paths: TOC panel, breadcrumb, search-by-type; possibly OrgChart). It crosses Rust↔Svelte↔write/read boundaries → the **four-phase workflow** (Architect → Plan → Build → Audit), with the acyclicity check and ordering **proven on the 7,600-note universe** before commit (no boot/typing/IPC regression — Rule 8 hard constraint).
- **Reuses the proven, safe paths:** the MIG-086 frontmatter props save path (Editor-Surface Gate, avoids the BUG-015 body-write landmine), the registry, the rename cascade (must include the new key — BUG-023-shape linked-probe test), and Write-Time Derivation for the reverse/child-list.
- **The one genuinely new primitive** the codebase has never had: **persisted, queryable sibling order** on a relationship.

**The ask (Boss ratification):**
1. **Ratify the concept** (§1–§2): the structural link is the *compositional spine*, a distinct **kind** from the 8 cognitive acts, exempt from the living-link/inquiry apparatus.
2. **Rule the load-bearing decisions** in §4 — at minimum **D1** (distinct kind + structural flag), **D3** (the **name** + declaration direction), **D5** (single- vs multi-parent), and the **D9 MVP surface**.
3. On ratification, I open the `/migration` **Architect** doc against these rulings. **Until then: build nothing.**

---

*End of PJ-065 Concept Paper v1.0 (draft). Sources: Living-Link-Concept-Paper-v1.0.md (RATIFIED §5/§6); Living-Link-Relationship-Typology-Research-2026-06-24.md; MIG-086-Architect-Frontmatter-Typed-Links.md; grounding workflow wf_240289a1-cdd (Constellation codebase · Obsidian Breadcrumbs/Tana/Dendron/Logseq/MOC · SKOS/Dublin Core/schema.org/OWL · Scrivener/Workflowy/screenwriting structure).*
