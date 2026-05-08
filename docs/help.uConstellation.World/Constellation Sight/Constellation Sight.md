---
aliases:
  - Constellation Sight
  - Sight
  - Star Chart
  - Knowledge Lens
  - Cognitive Lens
  - Universe Health
description: The Constellation Sight visualizes your entire knowledge universe as a celestial-hemisphere star chart. Each note is a star; libraries are wedges around the rim; the most-central notes sit near the pole. Multiple modes let you read the same universe through different cognitive lenses.
---

# Constellation Sight

## What Is It?

Imagine looking at the night sky from the northern hemisphere on a clear evening. The brightest stars cluster near the celestial pole. Constellations spread across the sky in regions you can navigate by. The Milky Way arcs faintly across the dome.

The Constellation Sight does this for your knowledge. Every note is a star on a parchment-cream sky. The most-connected notes sit near the center of the dome. The peripheral ones spread to the rim. Each library gets its own colored wedge. A small panel on the side lists every library by number and color — so you can read the chart at a glance.

It answers: **"What does my knowledge system look like, and how healthy is it?"**

---

## Why Does It Matter?

Most note-taking apps show you what you wrote. The Constellation Sight shows you the *shape* of what you know — where your thinking is concentrated, where it's thin, and how ideas connect across libraries.

The chart is also a **multi-instrument**. The same universe can be viewed through six different cognitive lenses (Regions, Link Types, Time, Confidence, Stages, Acts). Stars don't move between lenses — only how the chart organizes them changes. A note that sits at the center under Regions but flies to the rim under Confidence is telling you something diagnostic.

---

## How to Open It

1. Click the **star icon** (Sight) in the left ribbon.
2. The dome renders silently — no progress bar in normal use; the chart appears when ready (typically 2–5 seconds on large universes).
3. To close: click the **(×)** button in the header bar at the top, or press **Esc**.

---

## What You See

### The Dome

A circular chart of stars on a cream parchment background — Suwaidi northern-hemisphere chart aesthetic. Each star is a note in your universe. The polar layout means:

- **Center → rim (radius) = how central the note is.** Most-connected hubs sit near the pole; peripheral leaves spread to the rim.
- **Around the rim (azimuth) = which library the note lives in.** Libraries are arranged in wedges, sized proportional to note count.

### The Library Legend

A panel on the left side of the screen (or the right side if your Universe name reads right-to-left, like Arabic / Hebrew / Persian) lists every library:

- **UNIVERSE caption** at the top — your Universe's name in italic blue serif.
- **Numbered list of libraries.** Each row has a colored circular badge (the library's color), the library name, and the note count.
- The same numbers appear **around the rim of the dome** in the matching color, so you can navigate by glancing between the legend and the chart.

### The Universe Health Card

Anchored above the dome:

- **UNIVERSE HEALTH** caption.
- A gold roundel with the overall score (e.g., **91 / 100**).
- Four metrics flanking the roundel: **Modularity**, **Dominance**, **Entropy**, **Connectivity**.
- Each metric has a colored status pill (**HEALTHY** / **CAUTION** / **IMBALANCED**).

### The Universe Name

A blue-serif italic header above the dome (and below the Universe Health card) shows your Universe's name. Renders right-to-left for RTL languages automatically.

### The Stars

Each star is a small colored dot:

- **Color = library.** Every star in the same library shares one color (deterministic per library).
- **Size = total link count.** Most-connected notes are biggest; sizes are capped so no star dwarfs the others.
- **Thin black outline** on every star for contrast against the cream background.
- **Stars don't touch each other** — a repulsion algorithm keeps a 9 px minimum gap so the chart stays readable.

---

## Interaction

| Gesture | Effect |
|---------|--------|
| **Hover a star** | A tooltip appears near the cursor: the note's title (bold), community, centrality rank. |
| **Click a star** | A gold ring appears around the star; its links radiate out as dark-amber lines; connected neighbour stars get thin gold rings. The right-side panel slides in with note details. |
| **Double-click a star** | Opens the note in the editor. |
| **Click empty space** | Clears the selection; lines and rings disappear; side panel slides out. |
| **Mouse wheel over the chart** | Zoom the entire chart in/out. Everything scales together — the dome, the library legend, the Universe Health card, the Universe-name header. Like a magnifying glass over a fixed page. Range: 0.4× to 5×. |
| **Click + drag empty space** | Pan the chart. Drag threshold is 4 px so short clicks still hit stars. |
| **Reset View** *(bottom-left button)* | Always visible. Muted when at default zoom/pan; prominent when not. One click returns the chart to canonical view. |
| **Esc** | Cascading. First press clears the selected star. Second press resets zoom and pan. Third press closes the Sight. |

The (×) close button at the top-right and the right-side note panel (slides in when a star is selected) stay anchored to the window — they don't scale with the lens zoom.

### The Note Side Panel

When you click a star, the right-side panel slides in showing:

- **The note's title** at the top.
- **Community** — the Louvain cluster the note belongs to (notes that frequently link to each other share a community, regardless of library).
- **Centrality rank** — where this note ranks among all notes in the universe by centrality (e.g., #12 of 247).
- **Incoming links** — how many notes link TO this note.
- **Outgoing links** — how many notes this note links to.
- **Connected notes (N)** — a clickable list of every note linked to this one (up to 50). Each row shows a small colored dot (the linked note's library color), the note's title, and the library name. **Click any row** to recentre the panel on that connected note — Sight stays open, you don't switch to the editor. This lets you walk the graph one hop at a time, exploring the neighbourhood of an idea without leaving the dome.
- **Open in editor** — a button at the bottom that switches to the standard Editor pane on the selected note.

The connected-notes list makes the side panel a *navigation surface*, not just an information display. You can chase a thought from one note to its supporters, contrasts, exemplars, and back — all by clicking dots in a list.

---

## The Six Modes (cognitive lenses)

Sight isn't just one chart — it's six chart **modes**, each with its own way of organizing the dome. Color (library) stays the same across all modes; only the *meaning* of position and size changes.

| Mode | What the rim shows | What the radius means | What the size means | Asks |
|------|--------------------|------------------------|----------------------|------|
| **R · Regions** *(default, ready)* | Library | Centrality rank | Total degree | "Where in my cosmos does this idea live?" |
| **L · Link Types** *(ready)* | Dominant outgoing link type | Type diversity | Outgoing links | "What kind of reasoning?" |
| **T · Time** *(ready)* | Creation date wedge (year, month) | Recency | Age | "When did it emerge, and is it still alive?" |
| **C · Confidence** *(coming soon)* | Dominant confidence | Certainty homogeneity | Link count | "How certain?" |
| **S · Stages** *(coming soon)* | Dominant lifecycle stage | Avg link weight | Traversal count | "How alive?" |
| **A · Acts** *(coming soon)* | Which Act produced the note | Synthesis depth | Connections | "Where in the formulation arc?" |

**Today's build:** Regions is the default; the toggle UI to switch to other modes is shipping in the next phase. Stars will migrate between (X, Y, Z) positions with a 600 ms eased animation when modes switch — same patient, different scan.

---

## Tips

- **Read the chart by region first.** Glance at the colored wedges and rim numbers, then check the legend for what each wedge means.
- **Look for outliers.** A star sitting alone near the center of an empty wedge is a hub note in a small library — likely an important bridge.
- **Use Esc liberally.** Three presses always returns you to the main app, regardless of what state the chart is in.
- **Zoom in for detail.** Wheel-zoom in to inspect a dense wedge; wheel-zoom out to see the whole knowledge cosmos at once. Reset View snaps back instantly.
- **The Universe Health card is your compass.** Modularity below 0.3 ("CAUTION") suggests your knowledge is too tangled; Dominance above 50 % ("CAUTION") suggests one library is swallowing your thinking; Entropy below 1 ("IMBALANCED") suggests low diversity. Watch these metrics over time as you grow your knowledge graph.

---

## Concept paper

For the design philosophy and architecture: see `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` (the design paper, updated for v4) and `docs/SIGHT-V3-VISUAL-SPEC.md` (the per-mode (X, Y, Z) grammar specification).
