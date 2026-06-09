# Constellation Circulatory System (CCS / الجهاز الدوري) — Concept Paper v1.1

**Status: RATIFIED by Eisa, 2026-06-09.** Supersedes v1.0 (draft) — which itself superseded the
Link-Dashboard draft. v1.1 locks: the **placement** (the circulatory complement to CNS), the **name** (CCS),
the **"Why Circulatory" defense** (§3), the **section set** in cognition terms (§6), and the four owner
rulings (§15). The CCS `/migration` Architect doc follows from this version.

**FACT** = current code/docs. **PROPOSAL** = design argued here (now ratified where §15 says so).

---

## 1. What CCS is

**CCS is the diagnostic instrument for the *circulation* of the Living-Link graph at universe scale — the
pulse of your thinking.** It answers the Cognitive Engine's **Connection** question (Q4) from the
**circulatory** side: *how is my thinking flowing?* — which connections are load-bearing through use, which
are cooling toward dormancy, which have gone stale, what lifecycle stages dominate, how settled (conviction
vs doubt) the graph has become.

It is the peer of **CNS (Constellation Nervous System / الجهاز العصبي)**: the founding architecture
(`CONSTELLATION-KNOWLEDGE-FORMULATION.md §1.1`) is an explicit **dual** biological system — **Nervous** and
**Circulatory**. CNS shipped the Nervous register; **CCS is the Circulatory register.**

---

## 2. Why CCS exists — completing the dual system

CNS surfaces *who connects to whom* (topology). But every link also carries a **life**: Weight earned by
traversal, Last-Traversed, Traversal-Count, Confidence, and a lifecycle (`fresh → emerging → established →
load-bearing → stale`, + archived). That is the blood chemistry of the graph — **built, computed, and
decaying in the dark, with no window onto it.** The Knowledge-Formulation spec named that window as **P5 —
"Knowledge health dashboard / visualize circulatory health."** It was never built. CCS is that window.

---

## 3. Why "Circulatory" — the defense

This frame is not a metaphor we chose for symmetry with CNS. It is one we **inherited**.

**3.1 — We didn't apply circulation to the data; the data was *built* as circulation.** A link's living
properties aren't *like* a bloodstream — in the code they *are* one:
- **Weight is earned by flow** — `weight = 1 + ln(1 + traversals)`. A vessel strengthens under flow; a link
  you keep walking becomes an artery. The same curve.
- **Weight is lost without use** — `effectiveWeight = weight × e^(−ln2 · days/halfLife)`. An unused vessel
  narrows; a link untraversed 90 days goes stale. *That decay already runs in the app.*
- **The lifecycle** (fresh → … → stale → archival) *is* the life of a vessel (angiogenesis → strengthening →
  artery → regression → apoptosis).

CCS invents nothing. It makes *visible* what the engine has computed since links became living objects.

**3.2 — The founding design is explicitly dual.** §1.1 models the link on **two** systems — Nervous
(structural) and Circulatory ("vessels strengthen under heavy flow, weaken without use"). Shipping CNS and
not CCS is shipping an anatomy with a nervous system and no blood.

**3.3 — Topology cannot see death; only circulation can.** A note can be **topologically central and
intellectually dead** — a hub you haven't *thought through* in a year. CNS calls it load-bearing; CCS calls
it flat-lining. A thinker needs both — the anatomy chart **and** the vital signs; one without the other is a
map of a body that may already be dead.

**3.4 — It is formulation, not management.** "Link Health / Analytics / Dashboard" are *management* words
(inventory, status). **"Circulatory" is a formulation word** — it asserts your knowledge is a *living body*,
and this is its pulse. It makes executable the founding principle *"without ongoing thought, I will not find
truth through knowing."* Decay isn't a feature; it's that sentence, running.

---

## 4. CCS vs CNS — the boundary that defines both

| | **CNS** (Nervous) | **CCS** (Circulatory) |
|---|---|---|
| **Reads** | structure — *who connects to whom* | flow over time — *weight · decay · traversal · lifecycle* |
| **Ignores** | age, weight, traversal recency | topology (clusters, bridges, centrality) |
| **Question** | "what is the *shape* of my thinking?" | "how is my thinking *circulating*?" |
| **Picture** | the wiring map (a graph) | the pulse (ranked, curatable registers) |
| **Owns** | communities · centrality · bridges · structural blind-spots | living/cooling/load-bearing · conviction-flow · lifecycle · retired |

> *CNS never tells you a load-bearing bridge is going **stale**; CCS never tells you a worn link is a
> **bridge**.* Same metaphor, opposite system — together they answer the Connection question whole.

---

## 5. For the user (the pitch)

> Most tools hand you a filing cabinet. Constellation hands you a **living body of thought** — and CCS is its
> pulse. Open it and you won't see a list of links. You'll see which ideas are **warm** with recent thinking
> and which have gone **cold**; where your **convictions** are hardening into bedrock and where your **doubts**
> are still alive; what you keep returning to, and what you connected once and quietly abandoned. Because a
> connection you made and never walked again isn't understanding yet — it's a note you wrote to yourself and
> forgot. CCS tells you, at a glance, whether your thinking is **still circulating** — or whether, without
> noticing, you stopped. Your knowledge has a nervous system (**CNS** — how it's wired) and a circulatory
> system (**CCS** — whether it's alive).

*(Seeds the CCS help article + onboarding copy.)*

---

## 6. The section set — the circulatory diagnostics (cognition-named)

The medical metaphor stays in the engine room (§3); the labels speak cognition. Seven registers, each a
question a thinker already asks:

| Section | The question you bring to it | The living signal underneath |
|---|---|---|
| **Living Connections** | *"What am I actively thinking through?"* | most-traversed links — warm, in circulation |
| **Load-Bearing Reasoning** | *"What does my understanding rest on?"* | high earned-weight, recently walked — the arteries |
| **Cooling Inquiries** | *"What have I stopped returning to?"* | decaying toward dormancy (90+ days untouched) |
| **Conviction & Doubt** | *"How settled is my thinking?"* | the spread: hypothesis → evidence → established → **contested** |
| **The Life of a Connection** | *"Where are my links in their lifecycle?"* | the fresh → emerging → established → load-bearing → stale census |
| **Retired Reasoning** | *"What did I set aside — and can revive?"* | archived links, restorable |
| **The Acts of Inquiry** | *"What kinds of thinking am I doing?"* | distribution of the cognitive acts (supports · contradicts · … + custom) |

---

## 7. The guardrails CCS must honor (the Living-Link canon)

From the ratified `Living-Link-Concept-Paper-v1.0.md`:

1. **Untyped is the *question*, not "broken."** If CCS surfaces untyped at all, it is as **open inquiry**
   (the live edge of thinking), never a defect. (Untyped ≠ broken: broken = unresolved target.)
2. **Facts rest; formulations inquire.** **Cooling Inquiries** and **Conviction & Doubt** are *invitations*,
   shown only over the formulation layer (higher strata / claim-notes). CCS **never** tells a resting fact
   it's "cold" or under-challenged.
3. **`contradicts` is the engine.** **The Acts of Inquiry** *values* tension; a `supports`-only universe
   reads as one-sided, not tidy.
4. **Registry-driven.** The act vocabulary comes from the Link-Type Registry (the 8 + custom), canonical
   order.
5. **Reversible.** Archive is soft-delete; every action undoable (apoptosis, not necrosis).

---

## 8. What CCS reads & writes (contracts)

- **Reads (FACT):** `note_links` (weight, confidence, traversal_count, last_traversed, lifecycle, status) +
  the Link-Type Registry. All write-time-maintained — no graph walk on open (Perf Rule 8).
- **Writes (FACT):** only the existing lifecycle commands (`_link_traverse`, `_link_set_confidence`,
  `_link_archive`/`_unarchive`). CCS is a view + action layer — **no new write path.**
- **Consequence:** building CCS is a frontend relocation + composition, **not** a schema/data change.

---

## 9. What CCS is NOT (the complementarity, precisely)

- **NOT CNS** — topology/wiring (its sibling).
- **NOT Sky View** — the spatial force-directed *picture*. CCS is the ranked, curatable *pulse*, not a graph.
- **NOT the Base** — the cross-cutting survey/compare table. CCS is a deep-read instrument the Base *threads
  into* (alongside 360.3D / CNS / the Cataloger).
- **NOT 360.3D** — one note's connection signature. CCS is the universe population.
- **NOT the Cataloger (CECE)** — the **Origin** question (source × content-type). Orthogonal.
- **NOT Knowledge Health** — the count *cards*. **Ruling (§15): coordinate, not subsume** — CCS is the deep
  circulatory register and links *to* Knowledge Health; it does not absorb its note-quality counts.
- **NOT the authoring "Link Dashboard" panel** — **Ruling (§15): that panel is fully retired.** The universe
  view becomes CCS; the per-note view is Backlinks / Outgoing.

CCS is the only surface whose subject is **the link as a circulating population with a lifecycle.**

---

## 10. Home & scope

A **first-class universe-scale surface — a left-dock Core Plug-in, peer of CNS and the Cataloger.** Federated
across cUniverses. **Never** a note-context side panel.

---

## 11. Architectural invariants

- **I1 Write-time derivation** (Perf Rule 8) · **I2 Reversibility** · **I3 No new write path** ·
  **I4 Complement-not-overlap** (circulatory only; topology→CNS, spatial→Sky View, table→Base) ·
  **I5 Registry-driven** acts · **I6 Facts rest** (no signal flags a fact) · **I7 Federation-transparent** ·
  **I8 Locale + theme aware** (15 locales · Style-Setter).

---

## 12. Current state → target

**Today (FACT):** the right-sidebar **"Link Dashboard"** panel (`LinkDashboard.svelte`, 7 sections) is a
partial, mis-homed seed that **mixes registers** — *Most-Connected* is topology (CNS's), while
*Most-Traveled / Stale / Archived* are circulatory.

**Target:** CCS as a first-class left-dock surface owning **only the circulatory register** (§6) — topology
handed to CNS — and the **authoring panel fully retired** (§15). Per-note links remain in Backlinks/Outgoing.

---

## 13. Migration shape (frontend-mostly)

A `/migration` (Architect → Plan → Build → Audit): stand up the CCS left-dock surface; build the seven §6
registers from `note_links` + the registry; hand topology to CNS; **retire** the right-sidebar Link-Dashboard
panel; **coordinate** with Knowledge Health (link out, don't absorb); **re-point** the MIG-007 hub button to
CCS. No schema/data-flow change. Architect doc opens next.

---

## 14. The cognitive frame CCS sits in (complementarity recap)

The Cognitive Engine mirrors four questions back about each note — **Development** (Stages/maturity),
**Altitude** (stratum; Sky View size / 360.3D vertical), **Origin** (the Cataloger), **Connection** — toward
one destination, **Conviction**. CCS and CNS are the two universe-scale instruments of **Connection**
(circulatory + nervous). Cross-cutting: the **Base** surveys all four as columns and threads into the
deep-read instruments; **NSC** supplies aboutness; **Index** browses terms; **Sight/Map** are Wings.
*Learning one surface teaches all — they share the four questions.*

---

## 15. Ratified rulings (Eisa, 2026-06-09)

1. **Placement** — ✅ circulatory complement to CNS.
2. **Name** — ✅ **CCS (Constellation Circulatory System / الجهاز الدوري)**.
3. **"Why Circulatory" defense** — ✅ approved (§3).
4. **Section set / terms** — ✅ approved (§6).
5. **Knowledge Health** — ✅ **coordinate** alongside (not subsume).
6. **Authoring "Link Dashboard" panel** — ✅ **fully retire** into CCS + per-note Backlinks/Outgoing.
7. **MIG-007 hub button** — ✅ **re-point to CCS** when CCS ships.

**Open** (for the Architect/Plan, not blocking ratification): exact left-dock placement + dock icon; whether
CCS shows a small per-section trend over time; the Knowledge-Health coordination surface (a link vs an
embedded card).

---

*End of v1.1 (ratified). New file per revision; v1.0 preserved. The founding mission is
`CONSTELLATION-KNOWLEDGE-FORMULATION.md`; the link semantics are `Living-Link-Concept-Paper-v1.0.md`; CCS's
sibling is CNS.*
