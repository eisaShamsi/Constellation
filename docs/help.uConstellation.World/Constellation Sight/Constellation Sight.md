---
aliases:
  - Constellation Sight
  - Knowledge Lens
  - Network Analysis
  - Graph Analytics
  - Bridge Notes
  - Structural Gaps
  - Universe Health
description: The Constellation Sight applies network science algorithms to your knowledge graph, revealing bridge notes, topical communities, structural blind spots, and cognitive diversity.
---

# Constellation Sight

## What Is It?

Imagine looking at a city from above at night. Some buildings have many roads connecting them — they're hubs. Some neighborhoods are dense with activity, while others sit isolated with few connections. And between some neighborhoods, there are no roads at all — blind spots that could benefit from a bridge.

The Constellation Sight does this for your knowledge. It takes your notes and the links between them and analyzes their structure using algorithms from network science. It answers the question: **"What patterns and gaps exist in my thinking?"**

It doesn't tell you what to think. It shows you the shape of what you already know — where your knowledge is deep, where it's shallow, and where two areas of expertise could connect but don't yet.

---

## Why Does It Matter?

Most note-taking apps show you what you wrote. The Constellation Sight shows you what you *haven't* written — the connections you haven't made, the bridges you haven't built, the areas you've neglected.

This is based on a principle from network science: **the most valuable insight often lies not in what's present, but in what's absent.** The gap between two well-developed knowledge areas is where new ideas are born.

---

## How to Open It

1. Click the **Lens button** in the left dock bar — it looks like a magnifier with a plus sign
2. Wait a few seconds while the analysis runs (the more notes and links you have, the richer the results)
3. The Lens view appears: a graph on the left, an analytics panel on the right

To close: click the **×** button in the top-right corner, or press **Escape**.

---

## What You See

### The Graph

When the Lens activates, your note graph transforms:

| Element | What It Means |
|---------|---------------|
| **Large node** | A bridge note — it connects different areas of your knowledge. The larger the node, the more important it is as a connector. |
| **Small node** | A note that lives within a single topic area. Not a bridge, but still part of the fabric. |
| **Node color** | Each color represents an auto-detected topic cluster (community). Notes of the same color are densely connected to each other. |
| **Solid line** | A wikilink connection between two notes. |
| **Red dashed line** | A structural gap — two topic areas that could meaningfully connect but currently don't. These are your blind spots. |
| **Colored region** | A translucent ellipse showing the boundary of a community — a group of notes that form a topic cluster. |

### The Analytics Panel

The panel on the right shows five sections:

#### 1. Legend

Click to expand. Explains every visual element in the graph — what large/small nodes mean, what colors represent, what the lines indicate.

#### 2. Universe Health (0-100)

A single number that captures the overall diversity and balance of your knowledge:

- **Green (70-100)**: Healthy — your knowledge is diverse, well-connected, and balanced across topics
- **Amber (40-69)**: Moderate — some imbalance or isolation. Consider exploring underconnected areas
- **Red (0-39)**: Low diversity — your knowledge may be concentrated in too few areas

The score is built from four components:

| Component | What It Measures | What's Healthy |
|-----------|-----------------|----------------|
| **Modularity** | How distinct your topic clusters are. Clear topics = good. | 0.3 – 0.6 |
| **Dominance** | What percentage of your notes belong to the largest topic. Lower is better — you don't want one topic to overwhelm everything. | Below 35% |
| **Entropy** | How evenly your knowledge is spread across topics. Higher = more diverse. | Above 2.0 |
| **Links/Note** | Average connections per note. More links = better integration. | Above 1.0 |

#### 3. Top Bridges

The 10 most structurally important notes in your universe. These aren't necessarily the notes with the most links — they're the notes that *connect different areas*. A note with only 3 links can be a critical bridge if it's the only connection between two large topic clusters.

Click any bridge note to open it in the editor.

#### 4. Communities

Auto-detected topic clusters. The Lens uses the Louvain algorithm to find groups of notes that are densely connected to each other. Each community gets:

- A **color dot** matching the graph
- An **auto-generated name** from the most common words in member note titles
- A **member count**

These communities emerge from your actual link patterns — not from your folder structure or tags. They may surprise you.

#### 5. Blind Spots

Pairs of communities with few or no connections between them. Each blind spot shows:

- The two community names
- How many links exist between them (often zero)

A blind spot is not an error — it's an opportunity. It tells you: "These two areas of your knowledge could meaningfully relate, but you haven't explored the connection yet."

---

## Advanced Features

Expand the **Advanced** section in the panel to access:

### Layer Peeling

Drag the slider to temporarily hide the top 1-20 most central notes. This is like removing the obvious landmarks to see what's underneath. When you peel away your MOC notes and index pages, a secondary structure emerges — smaller clusters and bridge notes that were invisible under the dominant layer.

### Tag Edges

Toggle this on to reveal implicit connections. When two notes share the same tag but have no wikilink between them, the Lens draws an additional edge. This often reveals connections you've been making unconsciously (through tags) but haven't formalized (through links).

---

## Tips for Using the Lens

> **Start with the health score.** If it's low, look at the dominance and entropy numbers to understand why. Is one topic overwhelming your universe? Are your notes evenly distributed?

> **Check the blind spots.** Each gap between two communities is a potential research question. Ask yourself: "Is there a meaningful connection between these two areas that I haven't explored?"

> **Watch the bridges.** Your top bridge notes deserve extra attention. They're the structural backbone of your knowledge. Consider developing them further — adding more detail, more links, more context.

> **Use layer peeling when the graph feels cluttered.** Hide your MOC and index notes to see the organic structure underneath.

> **The Lens works better with more links.** If your notes don't have many wikilinks, the analysis will be thin. The more you connect your notes, the richer the insights become.

---

## How It Works (For the Curious)

The Constellation Sight uses three algorithms:

1. **Betweenness Centrality** (Brandes' algorithm, 2001): For each note, it counts how many shortest paths between all other note pairs pass through it. Notes that sit on many shortest paths are bridges — structurally important connectors.

2. **Community Detection** (Louvain algorithm): Groups notes into clusters by maximizing "modularity" — finding groups where connections within the group are denser than connections between groups.

3. **Structural Gap Detection** (based on Ronald Burt's structural holes theory, 1992): Identifies pairs of communities with high internal density but low inter-community connections.

All computation runs locally on your machine. No data leaves your device. The centrality algorithm runs in Rust for performance; community detection runs in JavaScript.

---

## RTL Support

The Constellation Sight works identically for Arabic, Hebrew, English, and any language. The graph algorithms analyze structure (links between notes), not language. Community names auto-detect text direction. The panel respects your interface language setting.

---

## Relationship to Other Features

| Feature | What It Shows | How It Relates to the Lens |
|---------|---------------|---------------------------|
| **Sky View** | Your note graph as a force-directed visualization | The Lens uses the same graph but overlays analytics (centrality, communities, gaps) |
| **Constellation Map** | The shape and maturity of your knowledge as a sunburst | The Map shows *structure*; the Lens shows *patterns*. Use both for complete awareness. |
| **OrgChart** | Your folder/library hierarchy as a visual tree | The OrgChart shows how you *organized* your notes; the Lens shows how they *actually connect*. |
| **Index** | Terms and concepts extracted from your notes | The Index shows *what you wrote about*; the Lens shows *how those topics relate*. |

All Constellation functions complement each other — they don't compete. Each gives you a different perspective on the same knowledge.
