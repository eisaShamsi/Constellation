# Constellation Sight v5

**Status:** Layer 1 ships in MIG-024. Layers 2–4 (diagnostic, recommendation, coaching) ship in MIG-025 → MIG-027.

Constellation Sight v5 is a single full-screen surface that shows the **shape and organization of your epistemic content** as a stable star chart. Each note is a star. The dome is divided into eight concentric strata bands — **L1 Datum** at the rim, **L8 Worldview** at the pole. Seven mode toggles re-cut the rim wedges to answer different cognitive questions; **strata radius, star size, brightness, and link colors never change with mode**.

This help topic covers Layer 1 — the visual foundation. The full feature set (diagnostic / recommendation / coaching) lands progressively.

---

## How to open Sight v5

Click the **star icon** in the left dock. Sight v5 takes the full content area; press **Esc** or click the **×** in the header to close.

## What you see

| Visual element | What it means |
|---|---|
| **Concentric rings** | The 8 strata bands. L1 Datum (rim) → L8 Worldview (pole). |
| **Calendar rim** | 12 months wrapping the outside of the dome. Current month subtly highlighted. |
| **Soft wash** | Content-similarity density — related themes without explicit links. |
| **Stars** | Your notes. **Size** = maturity (seed → sapling → evergreen → canonical → wilting). **Brightness** = confidence (hypothesis → evidence → established). **Color** = ink black for normal, red for contested. |
| **Faint lines** | Typed links between notes. Brighten on hover. |

## The 7 modes

The toggle bar at the top of the dome lets you re-cut the rim wedges:

- **R Regions** — wedge by Library (sized by note count).
- **L Link Types** — wedge by dominant outgoing link kind.
- **T Time** — wedge by creation month (12 wedges).
- **C Confidence** — wedge by per-note confidence (4 wedges).
- **S Stages** — wedge by lifecycle stage (6 wedges).
- **A Acts** — wedge by which Act produced the note (5 + Unacted).
- **P Provenance** — wedge by primary source family (11 + Unsourced).

**Spatial memory survives mode toggles.** A star at L7 Perspective in Regions stays at L7 Perspective in Confidence — only its angular position changes as the wedges re-cut.

## The 3 scopes

A second toggle row below the mode bar lets you narrow the view:

- **U Universe** — every note. Default.
- **L Library** — only notes in the currently-focused Library.
- **F Folder** — only notes in the currently-focused Folder.

## Interactions

| Action | Result |
|---|---|
| Hover a star | Tooltip shows its title; incident links brighten. |
| Click a star | Right-side panel opens with note detail (strata, maturity, stage, source, confidence, link count). |
| Click "Open in editor →" | Note opens in the main editor; Sight v5 closes. |
| Click background | Clears selection. |
| Press Esc | Clears selection. Press again to close Sight v5. |
| Change mode | Stars stay at the same strata band; angular positions update. |
| Change scope | Visible note set narrows; modes still work. |

## When mode P (Provenance) is dimmed

Mode P needs your notes to be classified by source (perception / inference / testimony / etc.). Constellation's CECE engine proposes classifications; you approve them via the **Source Review** panel in the right sidebar.

If most of your universe is unsourced, mode P shows a CTA card pointing you to Source Review. Once you've classified enough notes, mode P unlocks and shows the source-distribution wedge layout.

## What Sight v5 is NOT

Sight v5 does not duplicate other Constellation surfaces:

- It is **not Sky View** (live force-directed nervous-system graph).
- It is **not the Map** (library/folder hierarchy sunburst).
- It is **not Search** (point-query against your corpus).
- It is **not the Index** (term-level vocabulary browser).
- It is **not 360.3D** (per-note diagnostic surface).

Sight v5 = whole-universe shape. 360.3D = single-note depth. Selecting a star in Sight hands off to the editor / 360.3D; it does not deepen into a per-note view.

## What's coming next

- **Layer 2 — Diagnostic.** Computes health signals (strata distribution, source diversity, confidence balance, growth trajectory, contested resolution, acts coverage) and surfaces plain-language findings.
- **Layer 3 — Recommendation.** Converts findings into specific named actions via local LLM (Qwen3-1.7B + GBNF grammar).
- **Layer 4 — Coaching.** Walks you through executing recommendations conversationally — like having your own local AI tutor.

All inference is **local**. Zero cloud dependency. Your coaching is private.
