# Constellation Circulatory System (CCS / الجهاز الدوري) — Concept Paper v1.0

> **⚠️ SUPERSEDED 2026-06-09 by `Constellation-Circulatory-System-Concept-Paper-v1.1.md` (RATIFIED).** v1.1
> adds the "Why Circulatory" defense, the cognition-named section set, the user pitch, and the four ratified
> owner rulings. Read v1.1 for the canonical concept; this v1.0 is preserved as the draft of record.

**Drafted 2026-06-09.** Status: **first concept of record** for CCS. It **supersedes**
`Constellation-Link-Dashboard-Concept-Paper-v1.0.md` — that draft was the first attempt; cross-checking it
against the other core-plugin concept papers (the Cognitive-Engine One-Picture, CNS, the Base, the Cataloger,
360.3D, Sight Subsystem) revealed it had (a) conflated the authoring-side *"Link Dashboard"* archive panel
with the diagnostic instrument, and (b) tried to own *topology* that belongs to CNS. The instrument the
project actually lacks is the **circulatory** complement to CNS. This paper defines it.

**FACT** = current code/docs. **PROPOSAL** = what this paper argues, for the Owner to ratify (the placement
and the name **CCS** were ratified by Eisa 2026-06-09; the section set + open questions in §13 are not yet).

---

## 1. What CCS is

**CCS is the diagnostic instrument for the *circulation* of the Living-Link graph at universe scale — the
bloodstream monitor of your thinking.** It answers the Cognitive Engine's **Connection** question (Q4) from
the **circulatory** side: *how is my thinking flowing?* — which connections are load-bearing through use,
which are decaying toward dormancy, which have gone stale, what lifecycle stages dominate, how settled
(confidence) the graph has become.

It is the peer of **CNS (Constellation Nervous System / الجهاز العصبي)**. The founding architecture
(`CONSTELLATION-KNOWLEDGE-FORMULATION.md` §1.1) is an explicit **dual** biological system — **Nervous** *and*
**Circulatory**. CNS shipped the Nervous register; **CCS is the Circulatory register**. الجهاز الدوري.

---

## 2. Why CCS exists — completing the dual system

The Nervous System is fast, typed, structural: *who signals whom* (typed synapses). The Circulatory System is
continuous, sustaining, temporal: *vessels strengthen under flow and weaken without use.* Constellation's
whole link architecture is built on both — yet only the Nervous half has a universe-scale home:

- **FACT — CNS exists** (`Constellation Nervous System` help + MIG-061): a federated gravity-well graph that
  computes **topology** — Universe-Health (modularity / dominance / entropy / connectivity), **communities**,
  **top bridges** (load-bearing *connectors*), **structural blind-spots**, **centrality**. It reads *who
  connects to whom* and **ignores each link's age, weight, and traversal**.
- **FACT — the circulatory layer is built but unsurfaced.** Every link carries Weight (earned by traversal),
  Last-Traversed, Traversal-Count, Confidence, and a lifecycle (`fresh → emerging → established →
  load-bearing → stale`, + archived) — `store.ts` `linkLifecycle()` / `effectiveLinkWeight()`. This is the
  blood chemistry of the graph. **No surface renders it at universe scale.** The Knowledge-Formulation spec
  named it as **P5 — "Knowledge health dashboard — visualize circulatory health."** It was never built.

CCS fills that genuinely open slot. It is **not** an overlap with anything; it is the missing organ.

---

## 3. CCS vs CNS — the boundary that defines both

This is the one distinction that matters; everything else follows from it.

| | **CNS** (Nervous) | **CCS** (Circulatory) |
|---|---|---|
| **Reads** | graph *structure* — who connects to whom | *flow over time* — weight, decay, traversal, lifecycle |
| **Ignores** | age, weight, traversal recency | topology (clusters, bridges, centrality) |
| **Question** | "what is the *shape* of my thinking?" | "how is my thinking *circulating*?" |
| **Picture** | the wiring map (a graph) | the bloodstream monitor (ranked, curatable registers) |
| **Owns** | communities · centrality · bridges · structural blind-spots | weight · decay · dormancy · lifecycle · confidence-flow · archive |

> *CNS would never tell you a load-bearing **bridge** is going **stale** (high centrality, zero recent
> traversal). CCS would never tell you a frequently-traversed link is topologically a **bridge**.* Same
> founding metaphor, opposite biological system. Together they answer the Connection question whole.

---

## 4. The questions CCS answers (the circulatory diagnostics)

Grounded in the diagnostic-instrument canon (`CONSTELLATION-KNOWLEDGE-FORMULATION.md` §V.3) + the Living-Link
properties. Each is a *circulatory* reading — never a topological one:

- **ECG — the worn arteries.** Most-traversed links: the paths you actually walk (Traversal-Count).
- **Blood pressure — load-bearing flow.** High *earned* weight, recently active (`weight = 1+ln(1+tc)`).
- **Capillary regression — decay & dormancy.** Links untraversed 90+ days, effective weight halving; the
  `stale` tier; what is quietly dying.
- **Blood test — settledness.** Confidence distribution across the graph: hypothesis / evidence / established
  / **contested** — how much of your thinking is bedrock vs hunch vs actively-disputed.
- **Lifecycle census.** The `fresh → emerging → established → load-bearing → stale` distribution — the age
  structure of your connective tissue.
- **Autopsy — the retired.** Archived links: what intellectual paths you let go, restorable.
- **Typed-act balance** (registry-driven). The distribution of the cognitive acts (the 8 + the user's custom
  types, in canonical order). `contradicts` is **the engine**: a `supports`-only universe reads as
  one-sided, not healthy.
- **Cross-library circulation.** Federated flow across cUniverses.

---

## 5. The outputs — what you leave CCS holding

Four circulatory reads (the corrected version of the v1.0 draft's reads):

- **ARTERIES** — "these connections carry my thinking" (most-traveled, load-bearing). Protect them.
- **EROSION** — "these are decaying / going stale" (dormancy, decay). Re-walk them, or let them go.
- **SETTLEDNESS** — "this is how settled my thinking is" (confidence distribution; contested = alive).
- **AUTOPSY** — "this is what I retired, and can revive" (archived).

**Deliberately NOT here:** "orphans / broken / blind-spots as deficiencies." Structural gaps are **CNS's**
(topology). And per the Living-Link canon (§7), they must never be presented as a fact's failing.

---

## 6. What CCS reads & writes (contracts)

- **Reads (FACT):** `note_links` (weight, confidence, traversal_count, last_traversed, lifecycle, status) +
  the **Link-Type Registry** for the act vocabulary. All write-time-maintained — no graph walk on open
  (Perf Rule 8).
- **Writes (FACT):** only through the existing lifecycle commands (`_link_traverse`, `_link_set_confidence`,
  `_link_archive`/`_unarchive`). CCS is a view + action layer; it introduces **no new write path**.
- **Consequence:** building CCS is a **frontend relocation + composition**, not a schema/data change.

---

## 7. The guardrails CCS must honor (the Living-Link canon)

From the ratified `Living-Link-Concept-Paper-v1.0.md`:

1. **Untyped is the *question*, not "broken."** An untyped wikilink is the live edge of inquiry — *"I feel
   these belong, I'm still asking how."* If CCS surfaces untyped at all, it is as **open inquiries**, never a
   defect to clear. (And untyped ≠ broken: broken = unresolved target, a different thing.)
2. **Facts rest; formulations inquire.** CCS **never nags a fact** for low traversal or few links. Circulatory
   signals (decay, dormancy) are **invitations**, scoped to the **formulation layer** (higher strata / claim
   notes), never to a Datum. *"A library of only `supports` has stopped thinking"* applies to claims, not facts.
3. **`contradicts` is the engine.** CCS *values* tension as circulatory health, not noise.
4. **Registry-driven.** Typed distributions read the registry (the 8 + custom types), in canonical order
   (`supports · contradicts · causes · exemplifies · generalizes · derives-from · part-of · supersedes`).
5. **Reversible.** Archive is soft-delete; every action undoable (apoptosis, not necrosis).

---

## 8. What CCS is NOT — the complementarity, precisely

- **NOT CNS** — topology / wiring (communities · centrality · bridges · structural blind-spots). Its sibling.
- **NOT Sky View** — the spatial, force-directed *picture* of the graph. CCS is the tabular, ranked, curatable
  *health register*, not another graph.
- **NOT the Base** — the cross-cutting survey/compare *table* (it exposes link columns across notes). CCS is a
  **deep-read instrument the Base threads into**, alongside 360.3D / CNS / the Cataloger.
- **NOT 360.3D** — one note's typed-link signature + gaps. CCS is the universe-population view.
- **NOT the Cataloger (CECE)** — the **Origin** question (source × content-type). Orthogonal axis.
- **NOT Knowledge Health** — the count *cards* (tension / fragile / orphan counts). CCS is the deep circulatory
  register; it should **subsume or coordinate** those counts, not duplicate them (§13).
- **NOT the authoring "Link Dashboard"** — the right-sidebar archive/unarchive *editing* panel. That stays a
  per-note affordance; CCS is the universe-scale *diagnostic*. (Resolving this name clash is why CCS is named
  distinctly.)

CCS is the only surface whose subject is **the link as a circulating population with a lifecycle.**

---

## 9. Home & scope

**PROPOSAL.** A **first-class universe-scale surface — a left-dock Core Plug-in, peer of CNS and the
Cataloger.** Federated across cUniverses. **Never** a note-context side panel. (A note still gets its
lightweight links view via Backlinks / Outgoing.)

---

## 10. Architectural invariants

- **I1 — Write-time derivation** (Perf Rule 8): reads the maintained index; never recomputes on open.
- **I2 — Reversibility**: archive = soft-delete; all actions undoable.
- **I3 — No new write path**: composes existing lifecycle commands.
- **I4 — Complement, not overlap**: CCS owns *circulatory* readings only; topology stays CNS's; the spatial
  graph stays Sky View's; the survey table stays the Base's.
- **I5 — Registry-driven**: the act vocabulary comes from the Link-Type Registry (8 + custom).
- **I6 — Facts rest**: no circulatory signal flags a fact; signals scope to the formulation layer.
- **I7 — Federation-transparent**: honors the active universe's federated library set.
- **I8 — Locale + theme aware**: full i18n (15 locales) + Style-Setter theming, like every core surface.

---

## 11. Current state → target

**Today (FACT):** the right-sidebar **"Link Dashboard"** panel (`LinkDashboard.svelte`, 7 sections) is a
**partial, mis-homed seed** of CCS — and it **mixes registers**: *Most-Connected* is topology (CNS's job),
while *Most-Traveled / Stale / Archived* are genuinely circulatory. It lives in a ~300px note-context strip.

**Target (PROPOSAL):** CCS as a first-class surface that owns **only the circulatory register** — hands
*Most-Connected* / bridges / communities to **CNS**, keeps the circulatory sections, and adds the missing
ones (confidence distribution, lifecycle census, dormancy, registry-driven typed-act balance). The authoring
archive panel may remain as a note-scoped editing affordance (§13).

---

## 12. Migration shape (frontend-mostly)

A `/migration` (Architect → Plan → Build → Audit): stand up the CCS surface; relocate the circulatory
sections out of the right-sidebar panel; coordinate the boundary with CNS (topology) and Knowledge Health
(counts); make the typed views registry-driven; re-point the MIG-007 Links-Settings hub button to CCS. No
schema or data-flow change (it reads `note_links` + writes via existing lifecycle commands). Architect doc
to follow once §13 is ratified.

---

## 13. Questions for the Owner

1. **Knowledge Health.** Does CCS **subsume** the Knowledge-Health count cards (tension / fragile / orphan)
   into its dashboard, or **coordinate** alongside a still-separate Knowledge Health? *(Lean: subsume the
   link-health counts; leave note-quality counts to Knowledge Health.)*
2. **The authoring "Link Dashboard" panel.** Retire it into CCS + the per-note Backlinks/Outgoing — or keep a
   slim note-scoped archive/edit affordance in the sidebar? *(Lean: keep a slim per-note affordance; the
   universe view is CCS.)*
3. **Section set.** Confirm the §4 circulatory diagnostics (ECG / pressure / decay / blood-test /
   lifecycle-census / autopsy / typed-balance). Add or drop any?
4. **MIG-007 hub button.** Re-point "Open Link Dashboard" → CCS once it ships (until then it opens the current
   panel).

On your answers, this becomes v1.1 (ratified) and the CCS `/migration` Architect doc opens.
