# PJ-065 Phase-2 — Visualizing the Structural Spine on Sky View & OrgChart (Concept Stub)

**Status:** STUB / parked for later (Boss-requested 2026-06-28, "to consider, not necessarily now"). Not built. Pursue only after PJ-065 Phase-1 (the structural link + Structure panel) is closed, and only behind a full concept paper → `/migration`.
**Depends on:** PJ-065 §6 read APIs (`get_structural_children` / `get_structural_ancestors` / `get_structural_descendants`) — already shipped.

---

## The question
How should the structural (parent / TOC) link be *seen* on the two graph/tree surfaces — **Sky View (SV)** (PIXI bubble force-graph) and **OrgChart (OC)** (hierarchical tree)?

## Where it stands today (by design, not by gap)
- **SV** — structural edges are **deliberately excluded** from `sky_links` (PJ-065 §3): a TOC edge is not an idea-relation, and must never feed centrality/clustering. So SV shows nothing structural today, correctly.
- **OC** — reads the **filesystem** tree (`read_library_tree`, single-parent by folder), with no link-awareness. So it shows folder shape, not the authored spine.

## OC — the natural home (strong fit) ✅
**Horse:** *OrgChart answers "what is the shape of this?" — and there are two shapes: the folders, and the* work. *A "Structure" mode lets OC draw the authored compositional spine (Book → Parts → Chapters → Scenes) instead of the directory tree.*

The spine **is** a tree, and OC is the tree renderer — but a *different* tree from folders (a note's folder ≠ its structural parent). The data is **already there**: §6's `get_structural_descendants` gives OC the ordered parent→children tree directly.
- **Shape:** a mode toggle on OC — *Folders* (today) ↔ *Structure* (the spine). Teal edges, `seq`-ordered, breadcrumb root selectable.
- **Cost:** moderate — OC's `loadData` branches on a second data source (the §6 API). Keep the filesystem path untouched; add the links-derived path beside it. The concept paper §D9 earmarked exactly this.
- **Risk:** low — read-only, reuses §6, no cognitive-topology entanglement (structural is its own tree).

## SV — the debatable one (lean cautious) ⚠️
**Tension with Form-Aligns-To-Purpose:** SV's purpose is the *cognitive* topology (how ideas relate). The compositional spine is a different kind of thing; folding it into the idea-graph muddies that purpose, and it must never feed the graph math (centrality/clustering) — the very thing §3 protects.

The only defensible form is a **display-only overlay layer**: distinct teal edges, **toggled off by default**, that render *over* the bubble graph but never participate in layout or centrality. It would answer a narrow question — *"where do my chapters sit relative to my idea clusters?"* (composition vs. cognition).
- **Verdict:** weaker fit than OC. Build **only** if a concrete need surfaces; gate it behind its own concept that justifies the overlay. Do not add it speculatively.

## Recommendation / sequence
1. **OC "Structure" mode** — the clean Phase-2 build (sound concept, data ready). Own concept paper → `/migration`.
2. **SV structural overlay** — a *maybe*, display-only + toggled-off, pending a concept that earns it.
3. Both strictly **after** PJ-065 Phase-1 closes.

*Vocabulary note (per the standing correction): SV = Sky View (PIXI bubbles); OC = OrgChart (hierarchical tree). Neither is the Constellation Map (D3 sunburst arcs), which is a separate surface (and currently a disabled Wing).*
