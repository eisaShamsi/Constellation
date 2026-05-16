---
aliases:
  - Constellation Sight
  - Sight
  - Coordinated Views
  - Anchor Dome
  - Mini-Domes
  - Sight v6
description: Constellation Sight visualizes your entire knowledge universe as a stratified anchor dome with four coordinated mini-domes that re-encode the same notes through different channels (Confidence, Stage, Acts, Provenance). Hover anywhere to see everywhere; click sidebar chips or Shift+click stars to filter; promote any mini to inspect at full size with zoom. The dome and the facet sidebar form a closed bidirectional loop.
---

# Constellation Sight

## What Is It?

Constellation Sight is the **diagnostic instrument** for your knowledge universe. A central **anchor dome** shows every note positioned by **stratum** (how foundational the thinking is) and **time** (when it was created), with four **mini-domes** alongside that re-encode the same universe through different channels: **Confidence**, **Stage**, **Acts**, and **Provenance**.

It answers one question, with five complementary lenses: **"How is my Epistemic Content shaped and organized?"**

Hover any star in any dome and the same note lights up in all five surfaces — gold ring on the star, gold tint on the matching sidebar chips. Click a sidebar chip and all five views narrow. Shift+click a star in the Stage mini and the universe filters to that lifecycle stage instantly. Click empty space in any mini and that mini "promotes" into the big primary slot at full size, while the previous primary demotes into the vacated mini slot. Dome and sidebar form a closed bidirectional loop — read your universe from any angle, narrow it with any gesture.

## Why Does It Matter?

Most note-taking apps show you what you wrote. Constellation Sight shows you the **shape** of what you know.

- Where is your thinking **concentrated**? (the density gradient in the anchor)
- What's still **early-stage** vs **stable foundation**? (the Stage mini's color gradient)
- Which notes are **load-bearing** vs which are **isolated**? (the Acts mini's size encoding)
- Where did each idea **come from** — your own thinking, reading, hearing, tradition? (the Provenance mini's sector layout)
- How **confident** are you in your conclusions? (the Confidence mini's opacity gradient)

A note sitting at the center of the anchor (high connectivity → load-bearing) but in cyan in the Stage mini (`spark` — just barely started) is telling you something diagnostic: a load-bearing idea that hasn't matured yet. Coordinated Views reveals these mismatches at a glance.

## How to Open It

1. Click the **eye icon** in the dock at the left edge of Constellation.
2. The anchor dome renders within 2–5 seconds on most universes.
3. To close: click the **(×)** in the top-right of the Sight header, or press **Esc**.

## What You See

### The Header Strip

Top of the Sight view, left to right:
- **"Constellation Sight"** — the title.
- **"v6.1 — Coordinated Views (Phase 2)"** — the version subtitle.
- **"X / Y notes"** badge in gold — visible only when a filter is active; shows how many notes match the current filter out of your universe total.
- **"EXTENDED"** badge in gold small-caps — visible only when Extended view is on (see Cmd-Shift-D below).
- **"Reset View"** button — visible only when the layout has been changed away from the default; one click restores anchor primary + zoom 1.0.
- **(×)** close button — top-right, always present.

### The Anchor Dome (Primary Slot)

The large cream-on-dark dome in the center of the body:
- **Stratum rings** — 5 concentric circles. The innermost ring is your most foundational notes; the outer rim is your latest sparks. Stratum encodes the **depth** of thinking as radial distance.
- **Calendar rim** — 12 month labels around the outside (Gregorian by default; configurable in Settings to add Hijri, Solar Hijri, Hebrew).
- **Stratum labels** — italic text in the upper portion, naming each stratum band.
- **Stars** — every note in your universe drawn as a small cream dot, positioned by stratum × time. Density-rendered: dense regions read as soft texture; sparse regions show individual dots.
- **Connection lines** — typed-link edges between notes, drawn at low opacity beneath the stars. Only renders between visible (filtered-in) notes.
- **Hover ring** — a gold circle appears around any star the cursor is over.

### The Four Mini-Domes

Right side of the Sight body, in a 2×2 grid. Hidden by default; revealed with **Ctrl+D** (session-only) or **Ctrl+Shift+D** (persistent — see Extended view).

Each mini renders the same universe through one encoding:

1. **CONFIDENCE — opacity.** Notes you're more confident in render brighter; tentative notes fade. Encoded as per-note `confidenceAlpha`: hypothesis = 0.45, evidence = 0.7, established = 1.0, contested = 0.85.
2. **STAGE — hue (full-disk).** Each note gets a categorical color based on its lifecycle stage:
   - **Cyan** = `spark` (newly sparked idea)
   - **Orange** = `birth` (taking form)
   - **Violet** = `growth` (actively in motion)
   - **Green** = `maturity` (fully formed)
   - **Yellow** = `renewal` (recently revisited)
   - **Gray** = `dormancy` / `archival` (inactive / closed)
3. **ACTS — size (top decile).** Top-10%-by-link-count notes render as bigger dots; the rest stay small. Hot-spots pop pre-attentively.
4. **PROVENANCE — 5 sectors.** Stars are re-positioned into 5 angular sectors: **Self / Read / Heard / Reasoned / Tradition**. Tells you where each idea came from.

### The Demoted Anchor (When a Mini Is Promoted)

If you've promoted any mini to the primary slot, the anchor demotes into the vacated mini slot. There it renders as **plain neutral cream dots** (no channel encoding — just the universe baseline at small scale) with the title **"UNIVERSE — primary view"**. Click empty space inside it to swap the anchor back into the primary slot.

### The Facet Sidebar (Left Edge)

A collapsible panel with **6 filter facet groups**, each showing categories with live counts:

- **Folder** — your folder hierarchy (TOP facet per design)
- **Library** — your library names
- **Stratum** — Foundation / Roots / Trunk / Branches / Twigs / Edge of Knowing
- **Confidence** — Hypothesis / Evidence / Established / Contested
- **Stage** — Spark / Birth / Growth / Maturity / Dormancy / Renewal / Archival (and any other stage values present in your data)
- **Provenance** — Self / Read / Heard / Reasoned / Tradition

Click the **▶** tab at the edge to expand the sidebar. Click any chip to toggle it as a filter. Counts in the OTHER facets rebalance to show what's available given the active filter set (Hearst Flamenco preview pattern).

## Interaction

| Gesture | Effect |
|---|---|
| **Hover a star** anywhere | Gold ring on the same star in all 5 surfaces (anchor + 4 minis). Matching chips in the sidebar tint gold. Tooltip near the cursor shows the note's title. |
| **Plain click a star** | Opens the note in the editor. A **"Return to Sight"** button appears in the note's tab bar so you can jump back. |
| **Shift+click a star** in Stage / Confidence / Provenance mini | Toggles a filter on that star's category. All 5 surfaces re-render with only matching notes. Multi-select within a facet: Shift+click another category to ADD it. Shift+click on the same category again to remove. |
| **Shift+click a star** in Acts mini or on the anchor | No-op. Acts and anchor have no channel-specific facet category. |
| **Click empty area of a mini-dome** | That mini swaps into the primary slot at full size; the previous primary demotes into the vacated mini slot. Click any other mini's empty area to shuffle. |
| **Wheel-zoom (primary slot)** | Zooms toward the cursor. Range: 0.5× to 24×. |
| **Click+drag empty space** (primary slot) | Pans the view. 4-px drag threshold so short clicks still hit stars. |
| **Ctrl+0 / Cmd+0** | Resets zoom + pan to default on the primary slot. |
| **Ctrl+D / Cmd+D** | Toggles the mini-domes visibility — **session only**. Doesn't persist. |
| **Ctrl+Shift+D / Cmd+Shift+D** | Toggles **Extended view** — persistent. When on, minis are visible on every Sight open. "EXTENDED" badge appears in the header. |
| **Click sidebar chip** | Toggle that facet category into the filter set. All 5 surfaces and the count badge re-render. |
| **Reset View button** | Returns to anchor primary at zoom 1.0. Visible only when the layout has been changed. |
| **Esc** | Closes Sight. |

## Ghost Mode — Multi-Select from the Dome

When any filter is active, non-matching stars stay visible but render at **low opacity (15%)** instead of disappearing. This means:

- You can still SEE where the non-matching stars are.
- You can hover them (gold ring still appears).
- You can **Shift+click them to ADD their category to the filter**.

So "I want both spark AND birth notes" works directly from the dome:
1. Shift+click a cyan (spark) star → filter becomes `{spark}`. Orange (birth) stars become ghosts.
2. Shift+click a faded orange ghost → filter becomes `{spark, birth}`. Both colors at full opacity again.

You never have to leave the dome to compose multi-category filters within a facet.

## Density Mode

When the visible (matched) star count exceeds the density threshold (default **5,000**), the mini-domes switch to a **perceptual density rendering** — overlapping stars additive-blend into a soft texture instead of saturating to solid blobs. Below 5,000, mini-domes render as discrete dots at full encoding.

The threshold is configurable in Settings (`appSettings.sight.hexBinThreshold`). The anchor dome already uses additive-blend density rendering as its baseline, so it looks the same in either mode.

## Extended View

Pressing **Ctrl+Shift+D** (or **Cmd+Shift+D** on Mac) toggles "Extended view" — when on, the mini-domes are visible by default every time you open Sight. The state persists across Sight closes, app restarts, and reboots (stored as `appSettings.sight.extended`). A small **"EXTENDED"** badge in the header shows when it's active.

The per-session Ctrl+D toggle still works inside Extended view to temporarily hide the minis without flipping the persistent setting.

## When Sight Is Most Useful

- **Audit your knowledge shape** — open Sight after a writing session to see how the new notes fit into your universe.
- **Find blind spots** — sectors of the dome with few notes might be areas to explore.
- **Spot load-bearing weakness** — a centrally-positioned note in early-stage color (cyan = spark) tells you you're depending on something not yet matured.
- **Filter and inspect** — Shift+click cross-filter narrows the universe; promote any mini to study a channel at full size with zoom.
- **Track epistemic provenance** — promote the Provenance mini to see how your knowledge is sourced across Self / Read / Heard / Reasoned / Tradition.

## Related Surfaces

- **Constellation Nervous System (CNS)** — the complementary visualization (neuron-shape icon next to the Sight eye icon in the dock). CNS shows the **connection-traversal** view: Universe Health metrics, top bridges between communities, structural-gap "Blind Spots". If Sight is the sensory shape of your universe, CNS is its neural connections.
- **Constellation Map** — sunburst visualization of knowledge hierarchy.
- **Sky View** — graph-based link visualization (literal connection diagrams).
- **Index panel** — term-browser sidebar; complementary to Sight's spatial view.
