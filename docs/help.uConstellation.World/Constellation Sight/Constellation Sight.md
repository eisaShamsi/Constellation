---
aliases:
  - Constellation Sight
  - Knowledge Lens
  - Network Analysis
  - Graph Analytics
  - Bridge Notes
  - Universe Health
description: The Constellation Sight visualizes your entire knowledge system as a gravity-well graph, revealing how ideas connect, where knowledge is strong, and where it needs growth.
---

> **🚧 Constellation Sight is being rebuilt.**
>
> The current "gravity-well" visualization (v2) has been disabled in this build. A new Sight (v3) is in design — based on a star-chart aesthetic that will let you see your entire knowledge universe at a glance, with bridge-note "stars," knowledge-cluster "constellations," and density "Milky Way" bands. The reference page below describes what v2 *did* and is preserved while v3 ships.
>
> Why the rebuild? To deliver Sight's promise — *"What patterns and gaps exist in my thinking?"* — the visualization needs to be readable in one look, with stable spatial memory across sessions. Force-directed layouts (v2's choice) re-run their physics simulation each session and don't give the eye that anchor. Star charts do.
>
> Read [`Constellation-Sight-Concept-Paper-v1.1.md`](../../Constellation-Sight-Concept-Paper-v1.1.md) for what Sight is for, §13 for the star-chart vision, and §14 for the v3 redesign.

# Constellation Sight

## What Is It?

Imagine looking at a city from above at night. Some buildings are in the center, surrounded by roads leading everywhere --- they're hubs. Other buildings sit on the outskirts with one road in. Between them, highways show how traffic flows.

The Constellation Sight does this for your knowledge. Every note is a point. Every link between notes is a line. The most connected, important notes sit at the center. The peripheral ones sit at the edges. Links show how ideas flow between them --- their type, direction, and strength.

It answers the question: **"What does my knowledge system look like, and how healthy is it?"**

---

## Why Does It Matter?

Most note-taking apps show you what you wrote. The Constellation Sight shows you the *shape* of what you know --- where your thinking is deep, where it's shallow, and how ideas connect across libraries.

---

## How to Open It

1. Click the **Sight button** (eye icon) in the left ribbon
2. Wait a few seconds while the graph builds
3. The Sight view appears: a gravity-well graph filling the screen

To close: click the **x** button in the top-right corner.

---

## What You See

### The Gravity-Well Graph

Notes are arranged in concentric rings based on their importance (centrality):

| Ring | Who's Here | Why |
|------|-----------|-----|
| **Center** | Top 5% most connected notes | These are the hubs of your knowledge --- bridges between different areas |
| **Inner ring** | 5-15% | Major concepts with strong connections |
| **Middle ring** | 15-35% | Supporting knowledge |
| **Outer ring** | 35-100% | Peripheral notes, orphans, seeds |

Within each ring, notes are grouped by **library** --- your own organization. Notes from the same library share the same color and cluster in the same angular sector.

### Visual Vocabulary

| Element | What It Means |
|---------|---------------|
| **Large node** | High centrality --- this note bridges different knowledge areas |
| **Small node** | Low centrality --- peripheral or within a single topic |
| **Node color** | Which library the note belongs to |
| **Solid line** | A link between two notes |
| **Direction arrows** | Small arrows along each link showing which way the connection flows |
| **Line thickness** | Thicker = higher confidence (established), thinner = lower (hypothesis) |

---

## Interacting with the Graph

### Pan and Zoom

- **Scroll wheel**: Zoom in/out
- **Click and drag**: Pan the view
- **Fit to Screen button**: Resets zoom to show all nodes (expand icon in toolbar)

### Neighborhood Highlight

**Single-click** any node to see its connections:

- The clicked node gets an amber ring
- All directly connected notes stay fully visible
- Everything else dims to ~12% opacity
- Links to/from the selected note stay visible; all others nearly disappear

This is the nervous system metaphor: touch a nerve, and you see what it connects to.

**Click empty space** to clear the highlight.

**Double-click** a node to open the note in the editor.

### Hover

Move your mouse over any node to see its name in a tooltip.

---

## Search

Click the **magnifying glass** in the toolbar to open the search bar. The Sight search supports all the same operators as the main Search Hub:

### Free Text
Type any word or phrase. Results are highlighted on the graph with blue rings and category badges:
- **T** = title match
- **C** = content match
- **#** = tag match
- **P** = property match
- **S** = semantic match

### Link Operators

| Operator | Example | What It Finds |
|----------|---------|---------------|
| `links to [[X]]` | `links to [[Ideaverse]]` | Notes that link TO this note |
| `links from [[X]]` | `links from [[Ideaverse]]` | Notes that this note links TO |
| `mutual [[X]]` | `mutual [[Ideaverse]]` | Notes with bidirectional links |
| `links all [[X]]` | `links all [[Ideaverse]]` | All linked notes (union of to + from) |
| `orphans` | `orphans` | Notes with no links at all |

### Cognitive Link Types

| Operator | What It Finds |
|----------|---------------|
| `supports [[X]]` | Notes that support this note |
| `contradicts [[X]]` | Notes that contradict this note |
| `causes [[X]]` | Notes that cause this note |
| `exemplifies [[X]]` | Notes that exemplify this note |
| `generalizes [[X]]` | Notes that generalize this note |
| `derives from [[X]]` | Notes derived from this note |
| `part of [[X]]` | Notes that are part of this note |

### Directional Link Colors

When searching link operators, the connections are highlighted with direction:
- **Green lines** = inward (the result links TO the target)
- **Red lines** = outward (the target links TO the result)
- **Amber lines** = between two results

### Navigating Results

- Press **Enter** to jump to the next result (zooms to 3x)
- Press **Shift+Enter** for the previous result
- An **amber pointer arrow** marks the current result
- The result counter shows your position (e.g., "3/15")

### Syntax Chips

Click the **three-dot button** next to the search input to see all available operators as clickable chips. These are localized to your interface language.

### Search History

Click the search input when empty to see your recent searches (up to 8).

---

## Analytics Panel (SightPanel)

Click the **grid icon** in the toolbar to open the analytics sidebar.

### Universe Health (0-100)

A single number capturing the overall health of your knowledge system:

- **Green (70+)**: Healthy --- well-connected, diverse knowledge
- **Amber (40-69)**: Moderate --- some areas need attention
- **Red (below 40)**: Low --- knowledge may be too concentrated or disconnected

### Overview

Quick metrics at a glance:
- **Notes**: Total notes in your universe
- **Links**: Total connections between notes
- **Orphans**: Notes with no links (candidates for connection)
- **Links/Note**: Average connectivity ratio

Below these, a **library breakdown** shows how many notes belong to each library, with colored dots matching the graph.

### Link Health

Expandable section showing:
- **By Type**: Horizontal bars for each link type (supports, contradicts, causes, etc.) with counts
- **By Confidence**: Bars for hypothesis, evidence, established, contested
- **Dormant Links**: Count of links not traversed in 90+ days (a warning signal)

### Top Bridges

The 10 most structurally important notes. These are the notes that connect different areas of your knowledge. Click any bridge to highlight its neighborhood on the graph.

### Knowledge Insights

Six diagnostic views of your knowledge system:

| View | What It Shows |
|------|---------------|
| **Strongest Evidence** | Links with highest weight and established confidence |
| **Weak Foundations** | Hypothesis-level links with low weight --- ideas that need more support |
| **Tensions** | Contradiction pairs --- ideas in active disagreement |
| **Stagnating** | High-weight links not traversed recently --- knowledge going dormant |
| **Most Connected** | Notes with the highest number of connections |
| **Knowledge Gaps** | Areas where connections are missing |

Click any insight row to highlight the relevant nodes on the graph.

---

## Settings

Click the **gear icon** in the toolbar:

- **Legend**: Toggle the legend panel on/off
- **Stroke**: Adjust link line thickness (0.5x to 4x)
- **Opacity**: Adjust link visibility (10% to 100%)
- **Arrows**: Adjust direction arrow size (2px to 16px)

Settings persist when you navigate away and come back.

---

## Tips

> **Start with the health score.** A low score means your knowledge needs more connections or diversity.

> **Click nodes to explore neighborhoods.** The highlight reveals local structure that's invisible in the full graph.

> **Use link search to trace reasoning.** "links to [[X]]" shows everything pointing at an idea. "links from [[X]]" shows where an idea leads.

> **Watch the bridges.** Your top bridge notes are the backbone of your knowledge. Strengthen them.

> **The Sight works better with more links.** The more you connect your notes, the richer the visualization.

---

## How It Works

The Constellation Sight uses **betweenness centrality** (Brandes' algorithm) to determine which notes are most structurally important. Notes that sit on many shortest paths between other notes are bridges --- they connect different areas of knowledge.

The **gravity-well layout** positions nodes in concentric rings by centrality percentile, with angular sectors grouped by library. No random simulation --- positions are deterministic based on your knowledge metrics.

All computation runs locally on your machine. No data leaves your device.

---

## Relationship to Other Features

| Feature | What It Shows | How It Relates to Sight |
|---------|---------------|------------------------|
| **Sky View** | Your note graph as a force-directed visualization | Sight uses the same graph but with a structured gravity-well layout and analytics |
| **Constellation Map** | The shape and maturity of your knowledge as a sunburst | The Map shows *structure*; Sight shows *connections* |
| **OrgChart** | Your folder/library hierarchy as a visual tree | OrgChart shows how you *organized* notes; Sight shows how they *actually connect* |
| **Search Hub** | Full-text, tag, property, and semantic search | Sight search uses the same engine but highlights results on the graph |
