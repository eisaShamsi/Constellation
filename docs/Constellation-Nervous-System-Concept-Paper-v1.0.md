# Constellation Nervous System (CNS / الجهاز العصبي) — Concept Paper v1.0

**Status: DRAFT — produced 2026-06-11 for Eisa's ratification (MIG-075 docs-first step). No code rides on
this paper until it is ratified.**
**Author of record: Eisa ALSHAMSI** — this paper consolidates HIS April-2026 founding concept and HIS
recorded rulings (2026-05-07 → 2026-06-10) into the document the surface never had. Drafted by Claude;
every historical claim is cited to the original paper, a session log, or code read first-hand.

**FACT** = what the record/code says, cited. **PROPOSAL** = design language offered for ratification.
**Open questions** gate at §13.

---

## §0 · What this paper IS (and is NOT)

This is the **identity document of the CNS** — the only core plug-in that never had one. CCS, Sight, Map,
the Base, the Living Link, and 360.3D each carry a ratified concept paper; CNS's "why" lived in a help
topic and scattered passages. The 2026-05-19 *Sight Subsystem* paper — written five days after CNS was
named — does not even list CNS among Constellation's diagnostic subsystems (§3.3 names Sight, Sky View,
Map, 360.3D, Org Chart, Knowledge Health, Index — no CNS). This paper closes that gap.

It follows the truth-status discipline Eisa's own v1.1 Sight paper established: the concept is stated in
full, and §11 is the honest scoreboard of what's actually running today. It is **not** a build plan —
MIG-075 (`lab/reports/MIG-075-CNS-AUDIT.md`) carries the engineering; this paper carries the *meaning* the
engineering must serve.

---

## §1 · What CNS is

**CNS is the topology instrument of the Connection question — the wiring diagram of the universe.** It
applies network science to the whole link graph and answers one question:

> **"What is the SHAPE of my thinking — its regions, its bridges, its silences?"**

It renders every note as a body in a **gravity well** whose geometry IS the answer: distance from the
center = structural centrality (the load the note carries in the wiring); angular sector = the user's own
organization (libraries); the four analytical registers (§5) read the same graph as ranked, plain-language
diagnostics. CNS reads **structure only** — who connects to whom. It never reads flow (weight, decay,
traversal, confidence-as-evidence-state): that is its sibling CCS's blood chemistry.

---

## §2 · Origin — the lineage (FACT, dated)

This section exists on Eisa's explicit direction: *"go all the way back to when I first created the
Constellation Sight concept… You have to UNDERSTAND the whole idea."* The understanding, from the record:

**April 2026 — the founding paper.** Eisa writes *"The Constellation Lens — A Text Network Analysis
Engine for Knowledge Discovery"* (`Constellation_Lens_Concept_Paper_Eisa.pdf`, 14 pages; ported as
`Constellation-Sight-Concept-Paper-v1.1.md`). The proposition: transform the note graph "from a passive
relationship visualization into an **active analytical engine**" that identifies topical communities,
ranks notes by bridging importance, detects structural blind spots, and measures cognitive diversity —
answering ***"What patterns and gaps exist in my thinking?"*** The method is peer-reviewed network science
(InfraNodus lineage, WWW'19): Brandes betweenness centrality, Louvain communities, structural-gap
detection on Burt's structural-holes theory (1992), and a composite diversity metric
(modularity + dominance + entropy + connectivity). Local-first, in Rust, language-agnostic by design.

**The v2 build delivers the core.** The surface ships as "Constellation Lens" → "Constellation Sight"
(`ConstellationSight2.svelte`): centrality, communities, gaps, the health composite, the insights panel —
the v1.1 paper's own truth-status (2026-05-07) scores it **"~70–80% of the analytical promise delivered."**
Never built: content-similarity edges (PJ-035), layer peeling (PJ-036), Map↔Sight integration (PJ-037 —
later **rejected** outright by Eisa: *"There won't be Map-Sight integration"*).

**2026-05-07 — the fork.** Eisa rules the force-directed look insufficient for Sight's promise (*"the user
should identify what Sight claims to deliver with one look"*) and sets the star-chart north star. The
ruling is **secure-don't-muddle**: *"To secure what has been achieved so far with the current Sight, we
will disable it for now (the whole package), and will create the new Sight (v3) based on the current wins.
If it proves its worth, we will use it. If not, we already have the current one."* v2 is shelved as the
known-good fallback (MIG-017, `SIGHT_V2_ENABLED = false`).

**2026-05-07 → 05-13 — the Sight odyssey.** v3 (star-chart projection, MIG-018) ships and dies on an
architectural fault (the overlay's z-index wars; 13 failed close-button iterations; *"abandon v3, start
from scratch as v4"*). v4 is a brief clean slate. v5 (the seven-mode dome, Concept Papers v2.0 → v3.1)
**revokes the InfraNodus analytical framing entirely** — *"The analytical-network-science framing (Brandes
betweenness, Louvain, structural gaps, universe-health metrics) is out"* — and re-founds Sight on the
epistemic-content taxonomy. v5 passes the capability test and fails the outcome test; Eisa: *"Looking at
the modes, it is confusing! Again, what is Sight telling? What is unique about Sight? NOTHING!"* The
redesign conversation (v0.1–v0.3, six SMEs) produces v6 — the coordinated domes with 24 scholarly
traditions (Concept Papers v4.0/v4.1, MIGs 025/026/027).

**2026-05-14 — the survival.** With v6 live, Eisa runs the two surfaces side-by-side (commit `db8326a`
re-enables v2 for the comparison). The verdict, verbatim: ***"Sight v2 = Working. I decided to keep
it."*** He then rejects the SME naming proposals (Atlas/Threads) and rules the anatomical pair himself:
v6 = **Constellation Sight** (sensory), v2 = **Constellation Nervous System (CNS)** (neural) — *"Then, it
is going to be: Constellation Nervous System (CNS)"* — with the grammar fixed ("Nervous System", the
canonical anatomical term) and the name translating cleanly across all 15 locales (الجهاز العصبي ·
دستگاه عصبی · 神经系统 · מערכת העצבים …). (MIG-025 §A.15; SESSION-LOG-2026-05-14:298–359.)

**2026-05-19 — the divorce.** After the universal audit, Eisa externalizes the Sight line: *"No more Sight
as a core plug-in. But… we are going to make it an external plug-in."* MIG-038 disables Sight v6 + Map
into the future **Constellation Wings**; the same ruling defines the plugin taxonomy — **"Core Plug-in" =
a main-left-dock feature that stays in the app (Sky View / CNS / Index / CECE)**. CNS stays in the core.

**2026-06-09/10 — the anatomy completes.** Eisa ratifies and ships **CCS** (MIG-074), the Circulatory
sibling the founding spec (`CONSTELLATION-KNOWLEDGE-FORMULATION.md` §1.1) had always paired with the
Nervous register. The CCS paper's §4 boundary defines both organs at once. The MIG-074 Architect pins
CNS's identity in writing for the first time — and notes there is no CNS concept paper.

**2026-06-11 — this paper.** The MIG-075 audit finds CNS computing nine analytical results and rendering
five; the help topic documents registers ("Communities", "Blind Spots") that do not exist on screen; and
no document says what CNS is *for*. Eisa orders the full audit and this paper.

**The reading of this history (the paper's thesis):** *CNS is not a failed Sight project — it is the
original project.* Eisa's April-2026 analytical engine is exactly what CNS is: the network-science
instrument over the universe graph. The name "Sight" wandered through five redesigns chasing a different
question — "How is my epistemic content shaped/organized?" → "Is my universe healthy?" — re-founded
itself on the epistemic taxonomy, and was eventually externalized to Wings. The original instrument
survived every cull for one recorded reason: **it worked.** What it never received was its own paper, its
finished surface, and its own vocabulary. This document supplies the first; MIG-075 supplies the rest.

---

## §3 · Why "Nervous" — the defense

The frame is inherited, not decorative — the same §3 defense CCS's paper makes from the other side:

**3.1 — The founding design is explicitly dual.** `CONSTELLATION-KNOWLEDGE-FORMULATION.md` §1.1 models
the link on two biological systems: Nervous (structural) and Circulatory ("vessels strengthen under heavy
flow, weaken without use"). CCS shipped the Circulatory register; CNS **is** the Nervous one. An anatomy
with blood and no wiring diagram is as incomplete as the reverse.

**3.2 — The nervous system is the wiring, not the blood.** It is the fixed structural network along which
signals travel: what connects to what, where the trunks are, where a severed pathway leaves a region
isolated. That is precisely what CNS reads — topology. It ignores how often a pathway is walked (CCS's
traversal), how strong it has grown (CCS's weight), how settled the claim is (CCS's confidence). A
neurologist and a cardiologist examine the same body and write different reports.

**3.3 — The name is Eisa's own ruling** (2026-05-14), chosen against SME alternatives for the anatomical
pairing and its clean translation across all 15 languages — and it acquired its full meaning when CCS
shipped: the two organs now define each other (§6).

---

## §4 · The gravity well — the layout IS the answer

The canvas is not a picture; it is the diagnosis (Form-Aligns-To-Purpose):

- **Distance from center = structural centrality.** The notes your wiring routes through sit deep in the
  well; peripheral thinking sits at the rim. (Stratum-weighted Brandes betweenness; rings at the 5% / 15%
  / 35% percentiles.)
- **Angular sector = your own organization.** Libraries divide the circle. The code states the design
  intention verbatim: *"Position = centrality (distance from center) × library (angular sector). No
  community detection. Libraries are the user's own organization."* Position belongs to the user's
  declared order; **emergent structure (Regions, §5) is a color lens over it, never a relocation of it.**
- **The well stays circular** — stretching it to an ellipse would distort the centrality-equals-radius
  encoding (Eisa-verified ruling, PJ-11, 2026-05-29).
- **Stability is a feature.** The layout is deterministic (centrality × sector, plus a 15-tick collision
  relaxation for overlap only — no free physics). The v3-era critique of v2 — "the user can never build a
  stable spatial mental map" — was answered *inside* v2 by this design: same universe, same well.
- **Resting state is calm** (the founding Principle 6, Eisa 2026-05-07: *"When Sight opens, it shall
  display only the nodes; if we hover over one of these nodes, it will display its links."*) Edges render
  on hover / selection / search. The hidden state is the resting state.
- **Interaction grammar (locked, Eisa 2026-05-16):** single-click = preview (select the node, light its
  neighborhood, open the side panel); double-click = open the note in the editor. CNS never modifies
  anything — and never silently feeds the metrics it displays (no `_link_traverse`; the I2b
  no-observer-effect rule holds for CNS as it does for CCS).

---

## §5 · The registers — the four readings (cognition-named)

The medical metaphor stays in the engine room; the labels speak cognition (the founding paper's own rule:
*"Plain-language labels: 'Bridge notes' not 'Betweenness centrality.' 'Knowledge clusters' not
'Communities.' 'Blind spots' not 'Structural gaps.'"*). Four registers, each a question a thinker already
asks:

| Register | The question you bring to it | The signal underneath |
|---|---|---|
| **Regions** *(of thought)* | *"What neighborhoods has my thinking formed?"* | Louvain communities over the link graph — emergent regions, found not declared; rendered as a color lens on the well + a ranked list (size, character) |
| **Bridges** | *"Which notes hold my thinking together?"* | stratum-weighted betweenness centrality — the notes whose removal would cut regions apart; the deep-well bodies, listed top-10 |
| **Blind Spots** | *"Which regions should touch and don't?"* | structural gaps (Burt) — region pairs with dense interiors and no connecting tissue, each with **suggested bridge notes** (the candidates that could connect them). The founding paper's signature register: *"the dark sky between constellations is where the next discovery lives"* |
| **Structural Cohesion** *(name pending Q3)* | *"How well-formed is the whole?"* | the founding composite — 25·modularity + 25·(1−dominance) + 25·entropy + 15·connectivity + 10·(1−gap-penalty), 0–100 — with its four sub-metrics readable |

Contradiction **edges** render in the well like every typed link (registry red); the contradiction *list*
is TensionPanel's, the contradiction *count* is CCS's Acts register, the per-note flag is 360.3D's. CNS
adds no fourth contradiction surface (parsimony; audit ruling).

---

## §6 · CNS vs CCS — the dual-organ boundary (ratified, CCS Concept Paper §4)

| | **CNS** (Nervous) | **CCS** (Circulatory) |
|---|---|---|
| **Reads** | structure — *who connects to whom* | flow over time — *weight · decay · traversal · lifecycle* |
| **Ignores** | age, weight, traversal recency | topology (regions, bridges, centrality) |
| **Question** | "what is the *shape* of my thinking?" | "how is my thinking *circulating*?" |
| **Picture** | the wiring map (the well + four registers) | the pulse (ranked, curatable registers) |
| **Owns** | regions · bridges · blind spots · cohesion | living/cooling/load-bearing · conviction-flow · lifecycle · retired |

> *CNS never tells you a bridge is going stale; CCS never tells you a worn link is a bridge.* Same
> metaphor, opposite system — together they answer the Connection question whole.

**Vocabulary ruling this paper makes (PROPOSAL):** the word **"load-bearing" belongs to CCS** (earned
weight — Load-Bearing Reasoning). CNS's structural counterpart is always **"Bridges."** The older
Cognitive-Engine line that listed "load-bearing" among CNS's measures reads, going forward, as the
Bridges register. One word, one organ — the G4 collision dissolves.

---

## §7 · What CNS is NOT (the complementarity, precisely)

- **NOT Sky View** — the spatial *picture* (all notes, force-directed beauty, rings/glows/maturity, the
  full Style-Setter vocabulary). Sky View shows you the sky; CNS tells you what the sky *means*
  structurally. If a CNS feature does not derive from centrality, regions, gaps, or cohesion, it belongs
  to Sky View.
- **NOT CCS** — flow (its sibling, §6).
- **NOT Knowledge Health** — the at-a-glance count cards. KH gives you numbers; CNS gives you the wiring
  those numbers live in.
- **NOT 360.3D** — one note's connection signature. CNS is the universe; the ratified scope split
  (Sight-era, inherited) stands: universe ↔ note, mutually exclusive cardinality.
- **NOT the TensionPanel** — the per-library tension *lists* (contradiction pairs, orphans, tag-cluster
  gaps, single points). Two gap definitions coexist honestly: the TensionPanel's are vocabulary-clusters
  (tags) under-linked; CNS's Blind Spots are link-topology regions un-bridged. Lists there; geometry here.
- **NOT Sight** — the externalized Wings lineage (epistemic-taxonomy domes). CNS is Sight's *ancestor*,
  not its module; the 2026-05-19 Core-Plug-in ruling keeps CNS in the dock regardless of Wings' fate.
- **NOT the Map / Org Chart / Index / Cataloger / Search** — hierarchy density, part-of trees, the
  lexicon, Origin classification, finding. Orthogonal questions.
- **NOT an editor or a manager** — CNS writes nothing, ever (§9).

---

## §8 · The inherited principles (the founding paper's, restated as CNS's)

1. **Reveal, don't prescribe.** A blind spot is information, not an instruction (founding Principle 1).
   Layer 3/4-style recommendation/coaching was retired from this whole family by Eisa (2026-05-19); CNS
   surfaces structure; interpretation is the user's act.
2. **Compute locally, present honestly.** All analytics on-device (founding Principle 2) — and honest at
   the data layer: analytics cover the active universe; if the well displays federated notes, the scope is
   said out loud, never implied (the audit's federation-honesty rule).
3. **Emergent structure over imposed structure — in its place.** Regions are found, not declared (founding
   Principle 4) — but position belongs to the user's own order (§4). Emergence colors; it does not relocate.
4. **Language-agnostic by design.** The analytics are structural (founding Principle 5); every label flows
   through `$t()` ×15, RTL-correct; the registers speak the user's language (and the note's, where pills
   are involved, per the ratified MIG-067 §H rule).
5. **Reveal-on-demand.** Eisa's Principle 6 verbatim (§4). The resting state is calm.
6. **One vocabulary, one source.** Typed-link colors and names come from the Link-Type Registry —
   everywhere, including the well's edges and the legend (MIG-067/072 single-source invariant).
7. **Rule 8 — write-time derivation / indexed reads.** CNS opens from what is already stored (`note_links`,
   the boot graph, cached snapshots) — never by re-reading the corpus (the MIG-075 modernization contract).
8. **No observer effect.** Looking at the wiring never changes the wiring's numbers (I2b).

---

## §9 · What CNS reads & writes (contracts)

- **Reads (FACT):** the boot sky graph (`sky_nodes`/`sky_links` — alias-resolved, federated for display);
  `note_links` (the typed-link record layer — the modernization re-sources centrality/contradictions from
  here); the Link-Type Registry; cached snapshot keys where a register's number already lives (KH/CCS
  layer). All write-time-maintained.
- **Writes: nothing.** No file, no row, no link mutation, no traversal increments. The only persisted
  state is UI preference (panel visibility etc.) via the standard settings path.
- **Counts honesty (FACT):** CNS's header counts the **graph layer** — resolved edges between existing
  notes, self-links excluded (233,538 today); CCS/KH count the **record layer** — every recorded link row
  (234,062). Both true, different layers; the surface carries a one-line caption saying which it shows.

---

## §10 · Home & scope

A **Core Plug-in** in the main left dock (Eisa's taxonomy ruling, 2026-05-19) — peer of Sky View, Index,
the Cataloger, and CCS; opened as the full-page well; never a note-context side panel. Scope = the active
universe's wiring (display may include federation; analytics scope is labeled — true federation is the
reserved MIG-063 family).

---

## §11 · Truth-status matrix (the honest scoreboard, 2026-06-11)

The founding paper's §12 discipline, applied to CNS today — *the concept above vs the code as audited*:

| Founding mechanic | CNS today | Status |
|---|---|---|
| Betweenness centrality (Brandes, stratum-weighted) | the well's radial encoding + Top Bridges list | ✅ live — but fs-walk-sourced + sync (the MIG-075 S1 re-source fixes the engine, not the meaning) |
| Louvain communities | computed every open; **rendered nowhere** (props dead) | ❌ dark — Q2 decides the rendering form |
| Structural gaps + suggested bridges | computed every open; **rendered nowhere** (the founding signature register) | ❌ dark — restore per Q4/audit D2 |
| Cohesion composite (M+D+E+C+gaps) | the "Universe Health" card + 2 sub-metrics | ✅ live — rename pending (Q3); show all four sub-metrics (Plan detail) |
| Non-linear navigation | single-click preview / double-click open | ✅ live (locked grammar) |
| Reveal-on-demand (Principle 6) | edges on hover/select/search | ✅ live |
| Multi-edge graph (wikilinks + tags + similarity) | **wikilink edges only**; the tag-edges command is dead code (zero callers); similarity never built | ⚠ unrealized — Q5 rules the inheritance |
| Layer peeling (PJ-036) | never built | ⚠ inherited open question (Q5) |
| Content-similarity edges (PJ-035) | never built in CNS (embeddings now exist app-wide; Sky View renders semantic links) | ⚠ inherited open question (Q5) |
| Map↔Sight integration (PJ-037) | — | ❌ rejected by Eisa 2026-05-07; closed forever |
| Plain-language register labels ×15 | partially — raw keys leak; the title is English ×15; one header shows the wrong key | ❌ defects DF-05/07/08 — fixed in MIG-075 |
| One-registry colors | three hardcoded color maps | ❌ defect DF-06 — fixed in MIG-075 |
| Circulatory data in the panel (BY TYPE / BY CONFIDENCE / dormant / 4 insight tabs) | present | ❌ boundary violation — the Boss-approved shed removes it (→ CCS deep-link) |

---

## §12 · Current state → target

**Today (FACT):** the engine computes the founding instrument; the surface shows roughly half of it,
slowly, with three live IPCs and a corpus walk on the open path, circulatory blocks it doesn't own, and
no document saying what it is.

**Target (this paper + MIG-075):** the well opens instantly from indexed reads; the four registers render
complete in plain language ×15; the circulatory material lives in CCS behind one deep-link; every color
and name flows from the registry; the score carries its own name; and this paper is the contract every
future change is audited against — exactly the role CCS's paper plays for the circulatory side.

---

## §13 · Open questions for Eisa (the ratification gate)

| Q | Decision | Recommendation |
|---|---|---|
| **Q1** | Ratify the identity: §1 the question · §2 the lineage reading ("CNS is the original project") · §4 the layout philosophy · §5 the four registers · §6 the boundary + the "load-bearing belongs to CCS / CNS says Bridges" vocabulary ruling | **Ratify** |
| **Q2** | Regions rendering form: **(a)** a color *lens* toggle — node fill switches library-colors ↔ region-colors, legend follows, position untouched ★ · (b) region hulls/territories drawn around groups · (c) drop Louvain entirely (no Regions register) | **(a)** — honors "libraries are the user's own organization" while finally rendering the emergent read; (b) adds geometry the well's encoding doesn't need; (c) amputates a founding register |
| **Q3** | The composite's name (the "Universe Health" ↔ "Knowledge Health" collision): **(a)** Structural Cohesion / التماسك البنيوي ★ · (b) Cohesion · (c) Shape Score · (d) keep as-is | **(a)** — says what it measures; ends the two-healths confusion |
| **Q4** | Blind Spots register: ranked gap pairs + suggested bridge notes in the panel (the founding promise, restored) | **Yes** |
| **Q5** | The two unbuilt founding mechanics: **layer peeling** (hide the brightest, see beneath) and **content-similarity edges** | Layer peeling: **keep on the CNS roadmap** (a future toggle; cheap once analytics are DB-sourced). Similarity edges: **defer indefinitely** (Constraint as Design; Sky View's semantic links already serve the latent-connection read) |
| **Q6** | Hubs (most-connected): does CNS carry a small **Hubs** row (in-degree, topology) — making CNS its canonical home and retiring KH's card (reverses MIG-073's "keep"; only you can) — or does KH keep it and CNS shows none? | **CNS carries Hubs; KH's card retires** — topology lives in the topology organ; KH keeps its seven other cards + the CCS/CNS deep-links |
| **Q7** | The native title ×15: Arabic **الجهاز العصبي للكوكبة** (sibling of الجهاز الدوري للكوكبة)? The other 13 follow the CCS native-title pattern | Confirm the Arabic (or give your term) |

---

## §14 · Cross-references

`Constellation_Lens_Concept_Paper_Eisa.pdf` (the origin, April 2026) · `Constellation-Sight-Concept-Paper-v1.1.md`
(the founding analytical text + truth-status discipline) · `Constellation-Circulatory-System-Concept-Paper-v1.1.md`
(the sibling organ + the §4 boundary) · `CONSTELLATION-KNOWLEDGE-FORMULATION.md` §1.1 (the dual-system design) ·
`Cognitive-Engine-One-Picture-Concept-Paper-v1.0.md` (the four questions; Q4 Connection) ·
`lab/reports/MIG-075-CNS-AUDIT.md` (the engineering audit this paper governs) · the CNS help topic ×15
(rewritten to match this paper once ratified) · `lab/reports/SESSION-LOG-2026-05-14.md` (the keep + naming rulings).

---

*End of v1.0 (draft). New file per revision; nothing is deleted. The founding mission is
`CONSTELLATION-KNOWLEDGE-FORMULATION.md`; the founding analytics are Eisa's April-2026 Lens paper; CNS's
sibling is CCS. The wiring map and the pulse, together, answer the Connection question whole.*
