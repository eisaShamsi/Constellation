# Sight v3 — Visual Specification

**Version:** 1.1 (per-mode (X, Y, Z) grammar; supersedes v1.0)
**Date:** 2026-05-07
**Author:** Eisa Al-Shamsi (design); Claude (codification)
**Status:** Approved — locks the design contract for MIG-019 §2G implementation
**Reference mockup:** `docs/Constellation-Sight-v3-mockup-A2-toggle.svg`
**Reference generator:** `lab/sight-v3-mockup-generator.py` (`build_option_a_toggle`)

> **What changed in v1.1** (2026-05-07 evening, mid-§2G.3): Eisa
> elevated the design from "switchable rim axis" to "**per-mode
> (X, Y, Z) grammar**" — each mode declares its own azimuth, radius,
> and magnitude rules. Color stays invariant (community Louvain).
> §1.1 (mode wedge defs) and §2 (polar grammar) are restructured below.
> §7 invariants list updated: only **Color** is mode-invariant; X/Y/Z
> are mode-specific.

---

## Preamble

This document codifies the approved Sight v3 polar layout. It is the design contract for `src/lib/sight/v3/SightV3.svelte`. Any code change that touches the visual must reconcile against this spec. Any change to this spec must be approved by the Boss (Eisa) before implementation.

The design philosophy:

> The chart is a **multi-lens diagnostic instrument**. The same Universe can be read from six different cognitive angles by switching the rim axis. Radius, color, and magnitude are invariant across modes — only azimuth (rim wedge) changes. Stars migrate around the rim in a 600 ms ease when the user switches modes; the migration trajectory is itself a diagnostic signal (a star that sits at the center under Regions but flies to the edge under Confidence is telling you something).

The metaphor anchor is the Suwaidi northern-hemisphere star chart: a cream-parchment dome with near-black stars, blue ink labels, gold/cyan reference rings, and a soft Milky Way cloud. Eisa's design north star.

---

## §1. The Six Modes (per-mode X / Y / Z grammar)

Each mode is a **cognitive lens** that picks its own three input variables and maps them to azimuth (X), radius (Y), and magnitude (Z). **Only color is invariant** across modes (community membership via Louvain).

When the user toggles modes, every star's color is preserved while it migrates in (X, Y, Z) — the migration trajectory is itself a diagnostic signal (a star at the center under Regions but at the rim under Confidence is telling you something).

| ID | Mode | X (azimuth) | Y (radius: center → rim) | Z (magnitude / size) | Cognitive question | Data status |
|----|------|-------------|--------------------------|----------------------|--------------------|-------------|
| **R** | Regions | Library | Centrality rank | Total degree (link count) | "Where in my cosmos does this idea live, and how central?" | **Ready** |
| **L** | Link Types | Dominant outgoing link type | Type diversity (# distinct types used) | Total outgoing links | "What kind of reasoning, and how versatile?" | **Partial** — Z ready; X needs `note_links.link_type` piped through |
| **T** | Time | Creation date wedge (year, with month sub-wedges on the most recent year) | Recency (last edit; center = recently edited, rim = dormant) | Age (oldest = brightest, like ancient stars) | "When did it emerge, and is it still alive?" | **Ready** |
| **C** | Confidence | Dominant confidence (hypothesis → evidence → established → contested) | Certainty homogeneity (center = consistent, rim = mixed) | Total link count | "How certain, and how consistent?" | **Available later** — Concept Paper §6.3 P2 |
| **S** | Stages | Dominant lifecycle stage (Spark → Birth → Growth → Maturity → Dormancy → Archival) | Average link weight (center = high-weight, rim = low-weight) | Total traversal count | "How alive, and how worn the path?" | **Available later** — P3 |
| **A** | Acts | Which Act produced the note (Observation → Connection → Tension → Synthesis → Conviction) | Synthesis depth (center = fully synthesized, rim = raw) | Total connections | "Where in the formulation arc?" | **Available later** — P4 |

When a mode's X data isn't yet piped, the implementation falls back to Regions positioning so the chart always renders. The toggle UI dims that mode with an "available later" tooltip.

### §1.1 Wedge labels per mode

| Mode | Wedge labels |
|------|-------------|
| **R** Regions | Library name (e.g., "Research", "Reading", "Daily"). Order: by note count, largest first. Empty libraries (zero notes) compressed out. |
| **L** Link Types | `supports` · `contradicts` · `causes` · `exemplifies` · `generalizes` · `derives-from` · `part-of` |
| **T** Time | Year wedges (sized by note count per year), with month sub-divisions on the most recent year. Empty years compressed out. |
| **C** Confidence | `hypothesis` · `evidence` · `established` · `contested` |
| **S** Stages | `Spark` · `Birth` · `Growth` · `Maturity` · `Dormancy` · `Renewal/Archival` |
| **A** Acts | `Observation` · `Connection` · `Tension` · `Synthesis` · `Conviction` |

### §1.2 Mode-switch animation

- 600 ms eased transition (CSS easing: `cubic-bezier(0.4, 0, 0.2, 1)`)
- Each star's **angular** position interpolates from old → new wedge along the shorter arc
- Radius and color do NOT change — those are mode-invariant
- During transition: edges (if a node is selected) follow the migrating endpoints

### §1.3 Mode persistence

- Last-used mode persists per Universe via `appSettings.sight.lastMode` (debounced save, 300 ms)
- Default for first-time use: **R** (Regions)
- If saved mode is "Available later" (C/S/A), fall back to **R**

---

## §2. The Polar Grammar

### §2.1 Star position

```
radius = (1 − centrality) × DOME_R + edge_padding
azimuth = mode-dependent (see §1)
```

- **`centrality`** = normalized betweenness centrality `[0, 1]`. High centrality → near pole. Periphery → near rim.
- **`DOME_R`** = visible dome radius. ~540 px in the mockup; final value scales to viewport.
- Inner padding ~8 px so the highest-centrality star doesn't sit exactly on the pole.

### §2.2 Star magnitude (size)

Six log-distributed magnitudes, mapped from `degree`:

| Centrality | Radius (px) | Alpha |
|-----------|------------|-------|
| > 0.85 | 6.0 | 1.00 |
| > 0.65 | 4.0 | 0.95 |
| > 0.40 | 2.5 | 0.83 |
| > 0.20 | 1.5 | 0.71 |
| > 0.08 | 0.9 | 0.59 |
| ≤ 0.08 | 0.5 | ≥ 0.55 |

### §2.3 Star color

Color encodes **community membership** (Louvain). 8 pastel hues cycled across communities:

```
[#7c8a9e, #9e8a7c, #7c9e8a, #8a7c9e, #9e9e7c, #7c9e9e, #9e7c8a, #8a9e7c]
```

Colors are unaffected by mode toggle — they ride along with the star as it migrates.

### §2.4 Polar grid

- **4 declination rings** at r/R = 0.25, 0.50, 0.75, 1.00 (faint sand `#b8a98a` @ 18 % opacity; outermost @ 40 %)
- **Reference rings**:
  - Gold ecliptic-equivalent at r/R = 0.40 (`#c9a227`, 0.55 opacity, dashed `4 3`)
  - Cyan equator-equivalent at r/R = 0.70 (`#2b8fa8`, 0.50 opacity, dashed `2 4`)
- **Wedge spokes** (very faint) only at active-mode wedge boundaries

### §2.5 Milky Way

Two soft elliptical clouds with radial gradients in `#e6dec0` (parchment beige):
- Upper-left ellipse, rotated 35°
- Lower-right ellipse, rotated −35°

Visualizes TF-IDF content-similarity density (the existing v3 density field IPC). Toggleable via `appSettings.sight.showMilkyWay`.

---

## §3. Edges (Resting State + Active State)

Per Concept Paper §4.1:

> Resting state... we will show it as faint lines until the user hovers over or the connected nodes linking them.

### §3.1 Resting state

**Edges are HIDDEN in the resting state.** No constellation lines drawn by default. The dome shows stars only — clean, uncluttered.

### §3.2 Active state (hover or click)

When a star is hovered or clicked:
- **Selected star** gets a gold ring (`#c9a227`, 14 px radius, 1.6 px stroke)
- **All outgoing + incoming links** of that star are drawn as gold lines (`#c9a227`, 0.7 opacity, 0.9 px stroke)
- **Connected stars** (one-hop neighbors) get a thinner gold ring around them (size + 1.5 px, 0.7 px stroke)
- **Hint label** appears beside the selected star: `"selected · {N} links shown / edges appear on hover or click"` (italic, 11 px, INK color, 0.8 opacity)

### §3.3 Click vs hover

- **Hover**: temporary reveal; clears when pointer leaves the star
- **Click**: persistent reveal; clears when user clicks empty space or presses `Esc`
- Click promotes to "selection" — the star can then be opened (double-click → open note)

### §3.4 Edge filtering scope

- For dense universes (>50 outgoing links from one star), cap at top 50 by link weight
- Hint label updates: `"selected · 50 of 312 links shown (top by weight)"`

---

## §4. The Universe Health Anchor

### §4.1 Position

**Top-center, above the dome.** The roundel sits at the chart's vertical pole, with the four metrics flanking it left/right at the same vertical center.

### §4.2 Layout

```
                      UNIVERSE HEALTH                                    ← caps caption, y ≈ 97
                                                                          
   MODULARITY  DOMINANCE     ┌───────┐    ENTROPY  CONNECTIVITY          ← metrics + roundel
      0.63       18%         │  91   │     3.62      28.43               ← all centered y ≈ 165
   [CAUTION]  [HEALTHY]      │ /100  │   [HEALTHY]  [HEALTHY]            ← status pills
                             └───────┘                                    
                                                                          
                       ╭──── dome top edge ────╮                          ← y = (CY − DOME_R)
```

- **Roundel** at chart center-x, fixed y (≈ 165 px). Radius ≈ 50 px. Score in gold serif (`#c9a227`, 38 px, 600 weight). "/ 100" caption inside, below score (10 px, INK 60 %).
- **Caption "UNIVERSE HEALTH"** above roundel, small caps, letter-spacing 3, INK 60 %.
- **Metrics flanking** at 160 px stride from roundel center (e.g., x = 480, 640, 800-roundel, 960, 1120 in 1600 px canvas).
  - Each metric block: label (top, 10 px caps), value (middle, 22 px serif), status pill (bottom, 16 px tall, colored stroke + 12 % fill).
- **Status pill colors**:
  - `healthy` = `#3a8a4a` (green ink)
  - `caution` = `#c9831f` (amber ink)
  - (Other states TBD — define before adding)

### §4.3 Metrics displayed

In order (left to right, flanking the roundel):
1. **MODULARITY** — Louvain community quality score
2. **DOMINANCE** — largest community size as % of total
3. **ENTROPY** — Shannon entropy of community distribution
4. **CONNECTIVITY** — average degree (`2 × edges / nodes`)

(Backend already computes these in `compute_universe_health`.)

---

## §5. The Toggle UI

### §5.1 Design (mockup version)

The full preview strip from the mockup is a **design exposition** — it shows what each mode looks like. It is NOT the production toggle UI.

### §5.2 Production toggle UI

A compact letter-button bar anchored to the **top-right** of the chart panel (above the dome, opposite the title):

```
        [ R ]   L   T   C   S   A
          ─────────────────────────
           Regions          (caption shows active mode)
```

- Six small letter-buttons (R · L · T · C · S · A), 36 × 36 px each, 8 px gap
- **Active** mode: gold-filled background (`#c9a227`), parchment letter (`#faf6e8`), 600 weight
- **Ready** but inactive: cream background, near-black letter, 1.0 px outline (`#1a1a1a` 70 %)
- **Available later** (C/S/A): cream background, faded letter (45 % opacity), dashed outline. Tooltip on hover: `"Available with the Confidence Pack"` / `"Stages Pack"` / `"Acts Pack"`.
- Below the bar: a small caption showing the active mode's full name (e.g., "Regions"), serif, 13 px.

### §5.3 Keyboard shortcuts

- `R` / `L` / `T` / `C` / `S` / `A` keys = direct mode switch (when chart panel has focus)
- `Esc` = clear current selection (deselect star, hide edges)

### §5.4 Hover preview (optional, defer)

If/when we want full-fidelity previews: long-press or `Shift+hover` on a button surfaces a small thumbnail tooltip showing the mode's wedge layout. Defer until §2G.4 ships.

---

## §6. Color Palette (Suwaidi Reference)

| Token | Hex | Use |
|-------|-----|-----|
| `BG` | `#faf6e8` | Cream parchment background |
| `INK` | `#1a1a1a` | Near-black for stars + body text |
| `INK_SOFT` | `#3a3a3a` | Softer near-black for connector lines |
| `RULE_FAINT` | `#b8a98a` | Faded sand for grid + borders |
| `GOLD` | `#c9a227` | Ecliptic, score, current-year, active mode, selection |
| `CYAN` | `#2b8fa8` | Equator/reference ring |
| `RED_INK` | `#a83232` | (reserved — was time rim, now unused at top-level) |
| `BLUE_INK` | `#2a4a8c` | Title, region wedge labels |
| `MILKY` | `#e6dec0` | Milky Way clouds |

Status pill colors:
| State | Hex |
|-------|-----|
| `healthy` | `#3a8a4a` |
| `caution` | `#c9831f` |
| `critical` (TBD) | `#a83232` |

---

## §7. Layout Invariants (must hold under all modes)

These are the load-bearing rules. Breaking any of them is a P0 regression.

1. **Color = community** (Louvain) — the *only* mode-invariant axis. A star's color is preserved when the user toggles modes.
2. **X / Y / Z are mode-specific** — each mode declares its own (azimuth, radius, magnitude) rules. Their formulas live in `src/lib/sight/v3/modes.ts::positionForMode`. Changes to the cognitive grammar must update both this spec and that dispatcher in the same commit.
3. **Edges hidden by default** — never render constellation chains in the resting state.
4. **No edge spaghetti during animation** — even if a node is selected, edges follow the endpoints during the 600 ms migration; no blinking or instant snap.
5. **Empty wedges compress out** — the rim never shows a wedge with zero notes, regardless of mode.
6. **Universe Health stays top-center** — roundel + metrics never overlap the dome edge or the toggle bar.
7. **Universe-name header sits between Universe Health and the dome** — top-center, blue ink serif italic, `dir="auto"` so RTL universe names render correctly.
8. **OOM-safe Canvas 2D rendering** — single `<canvas>` element with D3-zoom for pan/zoom; stars and territories drawn via `CanvasRenderingContext2D` path batching. Never allocate per-star objects; clear and redraw the visible viewport on each frame.
9. **No per-frame allocations in hover handlers** — recompute decoration sets only on selection change, not on every mousemove.
10. **Mode-switch is a state change, not an IPC call** — `positionForMode(mode, ctx)` re-projects in JS. No re-fetch from Rust. The frontend has all data needed (SkyNode + layout + region wedges + ModeStats) to re-position every star in O(N).
11. **Color is preserved across mode switches** — the migration animation only interpolates X/Y/Z; the star's fill color stays constant. This is what makes the toggle a *diagnostic* tool: same patient, different scan.
12. **Rim labels are HTML overlay**, not canvas-drawn text — `dir="auto"` for native bidi shaping (Arabic / Hebrew / mixed-script library names render correctly).

---

## §8. What this spec does NOT cover (deferred)

- **Sight ↔ Map integration**: out of scope (per Eisa, §5.3 of mode discussion).
- **Two-up panel default**: out of scope (Sight only, by default).
- **Accessibility (keyboard nav, screen reader)**: deferred to a later migration.
- **Library-level color encoding**: rim labels use uniform `BLUE_INK`. If Eisa later requests per-library wedge colors, that requires backend work (LibraryInfo extension) and a follow-up migration.
- **Time-mode wedge granularity** (years vs months vs days): default to year-wedges with month sub-divisions on the most recent year. Granularity toggle is a follow-up.
- **Confidence/Stages/Acts modes**: rendered as "Available later" dimmed buttons. Implementation lands when Concept Paper P2/P3/P4 features ship.

---

## §9. Implementation phase mapping (MIG-019 §2G)

| Phase | Codifies which spec section |
|-------|----------------------------|
| §2G.1 | This document (you are reading it) |
| §2G.2 | §2 (polar grammar), §1.1 (mode wedge defs), §6 (palette tokens) → pure helpers |
| §2G.3 | §2 (positioning), §3 (edges), §4 (Health anchor), §7 (invariants) → SightV3.svelte rewrite |
| §2G.4 | §1.2 (animation), §5 (toggle UI), §5.3 (keys) → toggle + migration |
| §2G.5 | §1.3 (persistence) → settings field |
| §2G.6 | §7 (invariant audit) → close |

---

## §10. Acceptance criteria

The implementation passes when:
1. Default mode = Regions; rim shows library wedges sized by note count.
2. Switching to Link Types or Time animates star migration in 600 ms.
3. Confidence / Stages / Acts buttons are dimmed with a tooltip; clicking them is a no-op (or shows a "coming with Pack X" toast).
4. Edges are hidden by default; clicking a star reveals its links in gold.
5. Universe Health roundel + 4 metrics sit top-center, flanking the roundel.
6. Boss-test on the 7,636-note universe renders without OOM in under 5 s.
7. Mode toggle persists across Sight panel close/reopen.
8. v2 (`ConstellationSight2.svelte`) is unaffected (verifiable by toggling between v2/v3).

---

*This document is the authoritative visual specification for Sight v3. All MIG-019 §2G implementation must reconcile against it. Last updated: 2026-05-07.*
