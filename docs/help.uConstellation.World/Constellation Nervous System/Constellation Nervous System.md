---
aliases:
  - Constellation Nervous System
  - CNS
  - Universe Health
  - Top Bridges
  - Communities
  - Blind Spots
  - Sight v2
description: Constellation Nervous System (CNS) is the connection-traversal view of your universe. It analyzes the link graph between your notes and surfaces Universe Health metrics, communities, top bridges between clusters, and structural-gap "Blind Spots". CNS is the complementary view to Constellation Sight — if Sight is the sensory shape of your universe, CNS is its neural connections.
---

# Constellation Nervous System (CNS)

## What Is It?

Constellation Nervous System is the **connection-traversal** view of your universe. While Constellation Sight shows the *shape* of your notes (stratum × time × channel encoding), CNS shows the *wiring* — the typed-link graph that connects them and the structural patterns hidden in that graph.

It answers: **"How are the ideas in my universe connected, and where are the gaps?"**

The view is built around four analytical surfaces:
- **Universe Health** — overall and per-metric scores for how connected, balanced, and modular your knowledge is.
- **Communities** — groups of densely-interlinked notes (your "ideological clusters").
- **Top Bridges** — the few notes that link otherwise-separate communities (your "load-bearing connectors").
- **Blind Spots** — structural gaps where you'd expect connections but don't have them yet.

The name "Nervous System" is anatomical: nerves are connection pathways carrying signals between distant parts of an organism. The CNS visualization treats your typed-link graph the same way — the lines between notes are the "neural" carriers of meaning across your universe.

## Why Does It Matter?

Most note-taking apps treat links as plumbing (jump from here to there). Constellation treats them as **knowledge architecture**:

- A note with many incoming links is **load-bearing** — many ideas depend on it.
- A note that bridges two communities is a **synthesis point** — it ties domains together.
- A community with weak internal linking is **fragile** — risks losing coherence over time.
- A "Blind Spot" is a place where the structure SHOULD have a connection but doesn't — a hypothesis worth exploring.

CNS makes these structural features explicit. You don't have to read every note to understand the shape of your thinking — the network reveals it.

## How to Open It

1. Click the **neuron icon** (small branching nerve-cell shape — cell body in the middle with three dendrite branches and small synaptic terminals) in the dock at the left edge of Constellation. It sits near the Sight eye icon.
2. CNS opens in a full-window overlay, gravity-well style — a force-directed graph where each note is a node and each typed link is an edge.
3. To close: click the **(×)** at the top, or press **Esc**.

## What You See

### The Universe Health Card

A summary panel showing your universe's overall connectivity health, with the gold roundel showing a composite score (e.g., **91 / 100**) and four metrics:

- **Modularity** — how cleanly your notes cluster into distinct communities. High = well-organized into themes; low = single tangled mass.
- **Dominance** — whether one community dominates the universe. High = one cluster has most notes (specialist); balanced = healthier mix.
- **Entropy** — variety of community sizes. Balanced communities = healthy diversity; one giant + many tiny = unbalanced.
- **Connectivity** — average links per note. Higher = more interconnected ideas.

Each metric has a colored status pill: **HEALTHY** (green) / **CAUTION** (yellow) / **IMBALANCED** (red).

### The Gravity Well

The main visualization: notes float as nodes, links pull them together, repulsion pushes them apart. Communities self-organize into clusters. The layout settles in a few seconds.

- **Node size** = link count (highly-linked notes are bigger).
- **Node color** = community membership (notes in the same cluster share a color).
- **Edge** = typed link between two notes.

### Top Bridges

A list of the notes that link the most distinct communities — these are your synthesis points, the connectors between separate domains. Click any to focus on it.

### Communities

A list of detected note clusters. Click any community to highlight its notes in the gravity well.

### Blind Spots (Structural Gaps)

Suggested missing connections — pairs of notes the graph algorithm thinks SHOULD be linked based on shared neighbors or topical clustering. These are hypotheses for you to evaluate: a Blind Spot might be a genuine missing link to add, or it might be a coincidence to dismiss.

## Interaction

CNS uses a **single-click preview / double-click open** pattern (different from Sight's plain-click-opens):

| Gesture | Effect |
|---|---|
| **Single click a node** | Selects it. A right-side panel slides in showing the note's title, community, centrality rank, incoming links, and outgoing links. The note IS NOT opened in the editor. |
| **Double click a node** | Opens the note in the editor. A **"Return to CNS"** button appears in the note's tab bar so you can jump back. |
| **Hover a node** | Tooltip shows the note's title. |
| **Click empty space** | Clears the selection; side panel slides out. |
| **Mouse wheel** | Zoom in / out. |
| **Click + drag** | Pan the view. |
| **Click a community in the list** | Highlights that community's notes in the gravity well. |
| **Click a Top Bridge entry** | Focuses on that bridge note + highlights the communities it connects. |
| **Esc** | Closes CNS. |

The single-click-preview is deliberate: it lets you scan many notes' details (and their connections) without committing to opening each one in the editor. Double-click is the "I want to read/edit this one" gesture.

## When CNS Is Most Useful

- **Audit your connection density** — Universe Health gives a one-glance read on whether your universe is well-wired.
- **Find your synthesis points** — Top Bridges shows you the notes doing the most architectural work.
- **Discover communities you didn't know existed** — clusters that emerge from the graph might surface implicit themes in your thinking.
- **Patch Blind Spots** — when the graph suggests two notes SHOULD link but don't, evaluate the gap.
- **Plan re-organization** — communities map naturally to folder structure; weak modularity might tell you a re-org would help.

## CNS vs Sight — When to Use Which

- **Sight** = "How is my universe SHAPED?" Spatial / categorical analysis. Stratum × time × channels.
- **CNS** = "How is my universe CONNECTED?" Network / topological analysis. Communities × bridges × gaps.

They're complementary: Sight reads the surface; CNS reads the wiring underneath. Most analysis benefits from looking at both.

## Related Surfaces

- **Constellation Sight** — the sister visualization (eye icon in the dock). Sight shows stratum × time × channel encoding; CNS shows the link graph.
- **Sky View** — also a graph view, but built differently — Sky View is for free-form link exploration; CNS is structured around metrics and communities.
- **Backlinks / Outgoing Links panels** — per-note connection lists. CNS is the universe-wide view of the same data.
