---
aliases:
  - Constellation Lens
  - Knowledge Lens
  - Network Analysis
  - Graph Analytics
  - Bridge Notes
  - Structural Gaps
  - Universe Health
description: The Constellation Lens applies network science algorithms to your knowledge graph, revealing bridge notes, topical communities, structural blind spots, and cognitive diversity.
---

# Constellation Lens

The Constellation Lens transforms your Sky View graph from a passive visualization into an active knowledge discovery tool. It applies network science algorithms to your note graph, answering the question: **"What patterns and blind spots exist in my thinking?"**

> [!important] CE Layer 3
> The Constellation Lens is part of the Cognitive Engine. It requires notes with wikilinks to function — the more connections between your notes, the richer the analysis.

---

## Activating the Lens

1. Open **Sky View** (the graph visualization)
2. Click the **Lens toggle button** in the Sky View toolbar (magnifier icon with a plus)
3. The graph transforms: nodes recolor by detected community and resize by bridging importance
4. The **Lens Panel** appears on the right with analytics

Click the toggle again to deactivate and return to the normal graph view.

---

## What the Lens Reveals

### Bridge Notes (Betweenness Centrality)

The Lens identifies notes that serve as **bridges** between different knowledge areas. These are not necessarily the most-linked notes — they are the most *structurally important* connectors. A note with only 3 links can be a critical bridge if it's the only connection between two large topic clusters.

Bridge notes appear **larger** in the graph when the Lens is active. The top 10 bridges are listed in the Lens Panel.

### Knowledge Communities (Louvain Detection)

The Lens automatically detects **topical clusters** that emerge from actual link patterns — not from your folder structure or tags. Each community gets a distinct color on the graph. This reveals the natural structure of your knowledge, which may differ from how you organized it in folders.

### Structural Gaps (Blind Spots)

The Lens identifies pairs of communities with **high internal density but low inter-community connections**. These are blind spots — areas where two relevant knowledge domains exist but lack bridges between them.

Structural gaps appear as **red dashed lines** between community centroids in the graph. They represent opportunities for new insight: "These two areas of your knowledge could meaningfully connect but don't yet."

### Universe Health

A composite score (0-100) measuring the cognitive diversity of your knowledge base:

| Component | What It Measures | Healthy Range |
|-----------|-----------------|---------------|
| **Modularity** | How distinct your topic clusters are | 0.3 - 0.6 |
| **Dominance** | % of notes in the largest community | < 35% |
| **Entropy** | How evenly knowledge is distributed | > 2.0 bits |
| **Connectivity** | Links per note ratio | > 1.0 |

A healthy universe has clear but well-bridged communities, with no single topic overwhelming the rest.

---

## Advanced Features

### Shared-Tag Edges

Toggle **Tag Edges** in the Advanced section to reveal implicit connections between notes that share tags but have no explicit wikilinks. These edges appear as additional links in the graph, often revealing connections you haven't made explicit yet.

### Layer Peeling

Use the **Layer Peeling** slider to temporarily hide the top-N most central notes (usually MOCs and index notes). This reveals the underlying conceptual structure beneath the obvious surface — smaller clusters and bridge notes that were invisible under the dominant layer.

---

## The Lens Panel

The panel shows four sections:

1. **Universe Health** — composite score with breakdown
2. **Top Bridges** — notes ranked by bridging importance (click to navigate)
3. **Communities** — detected topic clusters with colors and member counts
4. **Blind Spots** — structural gaps between communities

---

## Design Principles

1. **Reveal, don't prescribe** — the Lens surfaces patterns and gaps; it doesn't tell you what to do
2. **Compute locally** — all analytics run on your machine in Rust; no data leaves your device
3. **Language-agnostic** — graph algorithms work on structure, not language; Arabic and English are treated identically
4. **Emergent structure** — communities come from actual link patterns, not folder hierarchy

---

## Technical Notes

- **Betweenness centrality**: Brandes' algorithm (2001), O(VE) complexity, computed in Rust
- **Community detection**: Louvain algorithm, computed in JavaScript
- **Structural gaps**: Based on Ronald Burt's structural holes theory (1992)
- **Performance**: Sub-second for 5,000 notes with 20,000 edges
