# Sight Redesign — Design Concept v0.3 (Option D Locked)

> **Status**: Architecture decision made. Option D ("Coordinated Views") adopted per the round-2 SME panel verdict + Boss approval (2026-05-13). v0.3 specifies it.
> **Date**: 2026-05-13
> **Author**: Claude
> **Supersedes**: `docs/sight-redesign-design-concept-v0.2.md` (kept on disk as historical record)
> **Function in hand**: lock the v0.3 architecture (Option D + 5 round-2 adjustments + 7-register chip + shape-only library + honest channel-tier labeling), then produce the formal Sight Concept Paper v4.0 contract.

---

## §1 — Why v0.3 exists

The round-2 SME panel converged on Option D as the winner across five of six methodological lenses, with a composite score of 72 vs. 63 (C) / 59 (B) / 56 (A) / 54 (E). The Data Visualization SME gave D a perfect 15/15; every other SME placed it in either first or second position even when their *outright* pick was a different option. Pure D won once, D-flavored hybrids won the other five.

The Boss approved this direction. v0.3 is the formal spec.

The panel also surfaced **five specific adjustments** for v0.3, plus **three structural issues** v0.2 introduced. v0.3 commits on all eight.

---

## §2 — The v0.3 architecture

A single screen, four zones, eight visual channels:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  Constellation Sight                                                          │
│  [pramāṇa] [masādir] [Polanyi] [Aristotelian ●] [Dignāga] [Ishrāqī] [Mohist] │  ← register chip
├────────────┬──────────────────────────────────┬──────────────────────────────┤
│            │                                  │  ┌──────────┐  ┌──────────┐  │
│   FACET    │                                  │  │ CONFID.  │  │  STAGE   │  │
│  SIDEBAR   │           ANCHOR DOME            │  │ opacity  │  │ hue only │  │
│            │     stratum × time × shape       │  └──────────┘  └──────────┘  │
│  Folder    │     + typed-link lines           │  ┌──────────┐  ┌──────────┐  │
│  Library   │                                  │  │  ACTS    │  │ PROVEN.  │  │
│  Stratum   │                                  │  │ size only│  │ 5 sectors│  │
│  Confidence│                                  │  └──────────┘  └──────────┘  │
│  Stage     │                                  │                              │
│  Provenance│                                  │                              │
│            │                                  │                              │
├────────────┴──────────────────────────────────┴──────────────────────────────┤
│  right-click stratum · click facet · click mini-dome category to cross-filter│
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Anchor dome (left-center, ~640×640 px)

- **Radial position** = stratum (Foundation → Edge of Knowing).
- **Angular position** = month of creation.
- **Shape** = library identity (circle / square / diamond / triangle / hexagon).
- **Opacity** = confidence (0.4 hypothesis → 1.0 established).
- **Inner pip hue** = stage (foveal channel; bumped from 1.1 px to ~1.8 px to address the Cog Psych SME's threshold concern, but explicitly labeled "focal-on-foveation," not pre-attentive).
- **Size** = top-decile acts flag (binary: baseline 5 px, top-acts 7 px; size delta increased from v0.2 +30% to +40% so pre-attentive discrimination holds at distance).
- **Lines** = typed-link connections (line color = link type).

### 2.2 Four mini-domes (right, 2×2 grid, ≥320 px each in production)

Each mini-dome shows the **same notes, in the same radial position** as the anchor dome, but **isolates one channel**. The mini-dome's stratum bands are preserved (rendered at 0.04 opacity, monochromatic gray) so radial position never loses meaning when the user looks at a mini.

- **Confidence mini-dome** — opacity only. All marks neutral fill, no shape distinction, no pip.
- **Stage mini-dome** — hue only. Stars rendered as 4 px circles with full-disk stage hue (not just inner pip), making stage pre-attentive in this view (resolves the Cog Psych pip-too-small critique by giving stage its own dedicated mini).
- **Acts mini-dome** — size only. Binary on/off: top-decile = 6 px filled disc; rest = 1.5 px dot.
- **Provenance mini-dome** — divided into 5 angular sectors (Self · Read · Heard · Reasoned · Tradition); stars positioned in their source sector + radial=stratum. Sector boundaries shown at 0.2 opacity.

Mini-domes are **clickable as filters** (Hearst Flamenco pattern — see §3.1). They are not display-only.

### 2.3 Facet sidebar (left edge, 180 px wide)

A collapsible list of all the corpus's facet categories, each with live counts:

```
▼ Folder
    Research          1,247
    Projects            892
    Reading-notes       563
    …
▼ Library
    Research          ●  3,124
    Projects          ■  2,447
    Personal          ◆  1,205
    …
▼ Stratum
    Foundation        1,856
    Working           2,103
    Connection        1,247
    Synthesis           924
    Edge of Knowing     506
▼ Confidence
    hypothesis          812
    evidence          2,567
    established       3,894
    contested           363
▼ Stage
    established  ●    3,141
    fresh        ●    1,247
    growing      ●    1,038
    at-risk      ●    1,254
    dormant      ●      956
▼ Provenance
    Self              1,247
    Read              3,894
    Heard               563
    Reasoned          1,532
    Tradition           400
```

Clicking any facet category filters all five views (anchor + 4 minis) and updates the counts in the other facets — Hearst's "preview" pattern. This is the navigation surface the LIS SME has been asking for since round 1. It also makes **Folder** visible (round-2 LIS critique: "Universe/Folder still invisible in v0.2").

### 2.4 Register chip (title bar)

Seven epistemic registers, each remaps the anchor dome's semantic axes:

| Register | Geometry | What the dome means |
|---|---|---|
| **Aristotelian** *(default)* | Radial = stratum (Foundation → Edge), angular = time | The default Western-classical reading: knowledge as a maturity gradient. |
| **pramāṇa** *(rewritten)* | 4 quadrants for pratyakṣa/anumāna/upamāna/śabda; radial = stratum (neutral, same as Aristotelian default); angular = time within quadrant | **No radial-certainty rings** (per Cross-Civ critique). Quadrant = pramāṇa-kind does the cultural work; radial stays stratum-neutral. Honors Nyāya pramāṇas as **kinds, not levels**. *Per-quadrant radial-internal structure (e.g. pratyakṣa: indriya-artha-sannikarṣa loci; anumāna: pakṣa/sādhya/hetu loci) is a v4.1 polish target.* |
| **masādir** *(rewritten)* | 4 sectors as **categorical kinds** (not nested ladder) for Qur'an / sunnah / ijmāʿ / qiyās; extension chips for istiḥsān, istiṣḥāb, maṣlaḥa mursalah, ʿurf | Sources rendered as different **kinds of proof** (naṣṣ vs. ijtihādī, qaṭʿī vs. ẓannī), not degrees-of-one-thing. Per al-Ghazālī's *Mustaṣfā*. |
| **Polanyi** *(rewritten — fog inverted)* | Tacit/explicit gradient, but **denser at center, clear at edges**. | Tacit is the **proximal pole** of all knowing (Polanyi 1958, ch. 4). Explicit notes appear clearly at the periphery; the foundational tacit core is acknowledged but visually inarticulable. |
| **Dignāga** *(new)* | Half-dome split: pratyakṣa hemisphere | anumāna hemisphere only. Śabda and upamāna explicitly **absent**. | Honors the Buddhist epistemological critique: only two valid pramāṇas. The absence is a feature, not a gap. |
| **Ishrāqī (Suhrawardi)** *(new)* | Luminous central core (ʿilm ḥuḍūrī — knowledge by presence) with progressive emanation outward to acquired/discursive knowledge | Per Suhrawardi's *Ḥikmat al-Ishrāq*. Presence-knowledge is irreducible to anything else; emanation logic supplants stratification. |
| **Mohist sān biǎo** *(new)* | Three arcs: historical precedent (top) · observational evidence (middle) · social benefit (bottom). Angular = time. | The unique premodern Chinese pragmatist register. Per Lloyd's *Disciplines in the Making*, ch. 4. |

The register chip lives on the title bar only and applies to the **anchor dome only**. The four mini-domes stay culturally neutral (channel-isolated views with no register semantics). This is "pluralism made architectural" — the cross-civ commitment is structural, not theatrical.

---

## §3 — The five round-2 adjustments now committed

### 3.1 Clickable mini-domes as filters (Hearst Flamenco pattern)

Each mini-dome is interactive. Hovering brushes (gold ring highlight propagates across all five views). **Clicking** a category in a mini-dome cross-filters:

- Click an opacity band in the Confidence mini-dome → all 5 views dim non-matching stars to 0.15 opacity; sidebar counts update.
- Click a hue cluster in the Stage mini-dome → same.
- Click a sector in the Provenance mini-dome → same.

This converts D from *coordinated views* (one-directional brushing) to *true faceted browse* (cross-filter with count-rebalancing). The LIS SME explicitly asked for this in both rounds.

### 3.2 Facet sidebar with live counts

Specified in §2.3. Folder is now visible.

### 3.3 Stratum bands preserved in mini-domes

Mini-domes render stratum-ring guides at 0.04 opacity (monochromatic gray). The user keeps the radial-anchor metaphor even while a mini isolates one channel.

### 3.4 Mini-dome sizing and rendering at scale

Mocks render mini-domes at the visual ratio the screen permits (~220 px in the design mock; production target ≥320 px). At >5,000 visible notes, mini-domes switch to **hex-bin aggregation** (per DataViz SME recommendation): each hex shows the dominant channel value + a count badge. Below 5,000 visible (post-filter), mini-domes revert to per-star rendering. This avoids the Vega-Lite/Plot density failure at 7,636 points × 240 px.

### 3.5 E's register-chip shipped alongside D

The seven registers in §2.4 are the v0.3 register set, all corrected against the Cross-Civ SME's round-2 critiques. The chip is structural — it exists in every layout regardless of which mini-dome the user is browsing.

---

## §4 — Channel taxonomy: hue residual decision

**v0.3 commits to shape-only library encoding with neutral fill.**

The DataViz SME called the v0.2 "shape primary, color identity hint" position a hedge. v0.3 takes the cleaner side: stars are rendered with a single neutral fill (`#cdd5e0`); shape is the sole library discriminator. Library color is gone from the always-on encoding.

### Why this commit

- **Mackinlay/Cleveland-McGill cleanliness.** Hue used three ways (library + stage + link line) was the residual problem. v0.3 reduces hue usage to two: stage (in the mini-dome) and link type (on the lines). Within accepted limits.
- **Closer to the Suwaidi reference.** The actual Suwaidi star chart did not differentiate stars by color either. Monochromatic stars is *more* Suwaidi-fidelity, not less.
- **Shape genuinely carries library identity.** Five distinct shapes (circle, square, diamond, triangle, hexagon) are pre-attentively decodable per Treisman/Healey.

### What this costs

- Library identity loses chromatic recognition. Users who associate "Research = blue" lose that mental peg.
- Above ~5 libraries the shape vocabulary exhausts. v0.3 specifies a **shape rotation within color family** fallback: libraries 6–10 reuse the same five shapes with an outline-style differentiator (solid → outlined → double-outlined → striped → dotted). This is a v0.4 implementation detail; v0.3 specs the contract.

### Honest pre-attentive channel ledger (per Cog Psych SME)

v0.2 claimed "6 channels at glance." v0.3 commits to honest labeling:

| Channel | Tier | Notes |
|---|---|---|
| Stratum (radial) | **pre-attentive** | Position is the strongest channel |
| Time (angular) | **pre-attentive** | Position |
| Library (shape) | **pre-attentive** | Treisman primitive |
| Confidence (opacity) | **pre-attentive** | Value/opacity is in the pre-attentive set |
| Acts (size +40%) | **pre-attentive** | Size is a Treisman primitive at ≥30% delta |
| Cluster gestalt (density) | **pre-attentive** | Emerges from positions |
| Stage (inner pip hue) | **focal-on-foveation** | Pip ~1.8 px; cannot pop without foveation; pre-attentive in the Stage mini-dome (full-disk hue) |
| Provenance | **filter or mini-dome only** | Not encoded on the anchor dome star |
| Typed links (line color) | **pre-attentive when sparse** | At link-density thresholds becomes visual noise; auto-fades when >800 links visible |

**Net: 6 pre-attentive channels + 1 focal + 2 deferred.** This is the truthful claim; v0.3 documents it instead of papering over it.

---

## §5 — Gesture grammar (carried from v0.2 + extended)

| Gesture | Effect | Where it applies |
|---|---|---|
| Right-click a stratum band | Other strata dim to 0.3 opacity; selected stratum stars stay bright | Anchor dome + propagates to minis |
| Click a facet category in the sidebar | All 5 views filter to matching stars; other facet counts rebalance | Cross-view |
| Click a category in a mini-dome | Same — cross-filter | Cross-view |
| Click a library shape in the sidebar | Other libraries fade; selected shape stays | Cross-view |
| Hover a stratum band | Diagnostic popover (count, confidence avg, stage breakdown, acts %, link density) | Anchor only |
| Hover a star | Side popover with full lens breakdown | Cross-view |
| Click a star | Open the note in the editor | Cross-view |
| Esc | Reset all filters, close all popovers | Cross-view |
| Cmd-F | Search overlay highlights matching stars, dims non-matches | Cross-view |
| Click a register chip | Re-frame the anchor dome's semantic axes; minis stay neutral | Anchor only |

No persistent toggle controls anywhere. No mode bar. Chrome consists of: title bar (register chips), sidebar (facets), and the four mini-domes themselves. Each chrome element is also a data display.

---

## §6 — Handling the remaining structural issues

### 6.1 Universe / Folder hierarchy

Round-1 and round-2 LIS critique: "Universe/Folder still invisible." v0.3 fixes this in the facet sidebar (§2.3, §3.2). Folder appears as the top facet (folder names with counts). Universe is implicit (the full canvas = current Universe; for federated cUniverse view, a small Universe selector chip will sit next to the register chip in v0.4).

### 6.2 Stage pip below pre-attentive threshold

Round-2 Cog Psych critique: 1.1 px pip cannot pop pre-attentively. v0.3 response:
- Pip enlarged from 1.1 → ~1.8 px at default zoom.
- Stage **labeled as focal-on-foveation** on the anchor dome (honest).
- Stage rendered as **full-disk hue** in its dedicated mini-dome (pre-attentive there).
- Net: user gets pre-attentive stage reading in the mini-dome; focal stage reading on the anchor.

### 6.3 Shape vocabulary ceiling at >5 libraries

v0.3 specifies the rotation pattern (§4: solid → outlined → double-outlined → striped → dotted within each shape family). Production cap: 25 libraries (5 shapes × 5 outline styles) before this hits a wall. Beyond 25, the spec defers to v0.4.

### 6.4 Library color loss

The shape-only commit is real loss. v0.3 mitigates two ways:
- Library names appear in the facet sidebar with shape glyphs next to them.
- Hovering a star surfaces the library name in the side popover.

If post-implementation testing shows users miss the chromatic recognition badly, v0.4 can reintroduce a low-saturation tint as a *recognition aid* (not a discriminator). v0.3 ships shape-only.

### 6.5 Hue-channel CIE Delta-E validation

DataViz SME flagged that "Stroop-like cross-channel interference" was a v0.2 risk because three independent hue dimensions co-rendered. v0.3 has two (stage + link). Before implementation, the palettes will be checked for CIE Delta-E ≥30 between any two co-rendered hues. This is an engineering checklist item, not a design question.

---

## §7 — The Suwaidi-fidelity reading

The honest reading of v0.3 against your original criterion:

> *"the image tells a whole story with one look"*

v0.3 takes the position that **"one look" = "the universe's state is comprehensible without serial workflow."** Under that reading, v0.3 succeeds:

- The **anchor dome** is still the Suwaidi-grade gestalt — radial strata, calendar rim, library-shape stars, cluster patterns. A user reading only the anchor dome already has cognitive density, confidence distribution, library dominance, and link topology at a glance.
- The **four mini-domes** are subordinate diagnostic supports that the eye samples on saccades. The Cog Psych SME's saccade-economics point: a user looking at a 5-panel layout fixates 2–3 panels per glance anyway. The minis are there for the second and third fixation, not the first.
- The **facet sidebar** is for the moment after the glance — the *"what do I do about it?"* surface.

If your Suwaidi-fidelity criterion is stricter — *"one literal circle, no companions"* — v0.3 fails on principle and you should pick Option A or B with the E register-chip grafted instead. That is the alternative I'd want named explicitly so it doesn't haunt the v4.0 contract.

---

## §8 — Implementation cost estimate (rough)

| Surface | Effort |
|---|---|
| Canvas 2D anchor dome (replaces v5 SightV5.svelte render) | ~2 weeks |
| Four mini-domes with hex-bin aggregation at scale | ~2 weeks |
| Facet sidebar with cross-filter + count rebalancing | ~1.5 weeks |
| Register chip + 7 registers (geometry remaps) | ~3 weeks (the registers are the heaviest single item — each needs its own visual grammar) |
| Gesture grammar wiring (right-click, click-to-filter, Esc, Cmd-F) | ~1 week |
| Side popover (stratum diagnostic + star detail) | ~1 week |
| Performance: KDE/hex-bin at >5,000 visible notes | ~1.5 weeks |
| Test/polish | ~1.5 weeks |
| **Total** | **~13.5 weeks** of focused engineering |

This is ~3× the v5 (MIG-024) build cost. If we're going to spend it, v4.0 has to be right.

---

## §9 — Open questions for Boss

1. **Seven registers is a lot.** Aristotelian (default), pramāṇa, masādir, Polanyi, Dignāga, Ishrāqī, Mohist sān biǎo. Do you want all seven for v1, or should v1 ship with the four pre-existing (Aristotelian, pramāṇa, masādir, Polanyi) and add the three new ones in v4.1?
2. **Mini-dome aggregation mode.** Above 5k visible notes, mini-domes switch to hex-bin. Per-star rendering returns when the user filters to <5k. Is this acceptable, or do you want per-star always?
3. **Library color loss.** §4 commits to shape-only neutral fill. Confirm — or push back if you want a subtle tint retained.
4. **Folder visibility.** §6.1 puts Folder in the facet sidebar. Do you want it ALSO encoded on the dome itself (e.g., border style for folder), or is the sidebar enough?
5. **v4.0 vs v4.x naming.** When the design crystallizes into a Concept Paper, do you want it numbered v4.0 (clean break from v5/v3.1) or v4.x (acknowledging the redesign as a major track)?
6. **Round-3 SME panel.** Should I spawn one more SME panel to verify the v0.3 adjustments resolved their round-2 concerns before drafting v4.0? Or proceed directly to the Concept Paper draft?

---

## §10 — What v0.3 still doesn't commit to

- Color accessibility variant (high-contrast / colorblind-safe). Deferred to v0.4.
- Animated transitions when register switches or filters apply. Deferred to v0.4.
- Whether v4.0 supersedes v5 entirely or coexists during a transition window. Deferred until Concept Paper drafting.
- Universe-selector chip for cUniverse federation view. Deferred to v0.4.
- The exact CIE Delta-E palettes (§6.5 engineering checklist).

---

## §11 — Round-2 panel responses, settled

| Round-2 SME concern | v0.3 disposition |
|---|---|
| Hue residual (library color + stage + link type co-rendering) — *DataViz* | Settled: shape-only neutral fill (§4) |
| Inner pip too small for pre-attentive — *Cog Psych* | Settled: pip enlarged + honest tier labeling + full-disk hue in mini-dome (§6.2) |
| Mini-dome density at 7k+ notes — *DataViz* | Settled: hex-bin at >5k visible (§3.4) |
| Mini-domes lose stratum context — *UX + Cog Psych* | Settled: stratum bands preserved at 0.04 opacity (§3.3) |
| Coordinated views ≠ faceted browse — *LIS* | Settled: clickable mini-domes + facet sidebar = Hearst Flamenco pattern (§3.1, §3.2) |
| Folder/Universe invisibility — *LIS* | Settled: Folder in facet sidebar (§6.1) |
| Pramāṇa radial-certainty rings wrong — *Cross-Civ* | Settled: rewritten to pramāṇa-internal structure, not certainty levels (§2.4) |
| Masādir-as-ladder wrong — *Cross-Civ* | Settled: rewritten as categorical kinds + extension chips for additional uṣūl sources (§2.4) |
| Polanyi fog inverted from his own argument — *Cross-Civ* | Settled: fog denser at center, clear at edges (§2.4) |
| Three registers curatorially thin — *Cross-Civ* | Settled: seven registers (Aristotelian + Dignāga + Ishrāqī + Mohist sān biǎo added) (§2.4) |
| Shape vocabulary ceiling — *Info Design* | Settled: rotation pattern specified (§6.3); deferred to v0.4 implementation |
| Universal-vs-tradition-specific channels — *Cross-Civ* | Settled: minis stay culturally neutral; only anchor dome remaps per register (§2.4) |
| E mis-renders its own traditions — *Cross-Civ* | Settled: registers rewritten (§2.4) |

Items deliberately deferred: color-accessibility variant, animation transitions, cUniverse selector — all v0.4 concerns.

---

## §12 — From v0.3 to Sight Concept Paper v4.0

If v0.3 is approved, the next move is the formal Concept Paper. v4.0 is what becomes the build contract for whichever MIG implements it. v0.3 is the design conversation; v4.0 is the architectural commitment.

Pre-v4.0 work I'd recommend:
1. Optional: one more SME panel pass on v0.3 to verify the adjustments held (cheaper than discovering breakage during v4.0 drafting).
2. Then draft `docs/Constellation-Sight-Concept-Paper-v4.0.md` as the formal supersession of v3.1.
3. The v4.0 paper then becomes the spec for `MIG-026` (or whatever number gets allocated) — Sight v6 architecture build.

---

## Appendix A — Files

| File | Purpose | Version |
|---|---|---|
| `docs/sight-redesign-design-concept-v0.3.md` | This document | v0.3 (current) |
| `docs/sight-redesign-v0.3-full-layout.svg` | The committed Option D layout | v0.3 |
| `docs/sight-redesign-v0.3-register-chip-detail.svg` | All 7 registers visualized | v0.3 |
| `docs/sight-redesign-design-concept-v0.2.md` | v0.2 (kept for reference) | v0.2 |
| `docs/sight-redesign-design-concept-v0.1.md` | v0.1 (kept for reference) | v0.1 |
| All v0.1 and v0.2 mock SVGs | Historical reference | — |

## Appendix B — Cross-references

- `docs/Constellation-Sight-Concept-Paper-v3.1.md` — current contract; will be superseded by v4.0 if v0.3 is approved.
- `docs/sight-v5-mode-concepts.md` — semantic source for channel encodings.
- `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` — the Five Acts, the 7+2 link types.

---

*End of v0.3. Awaiting Boss approval to draft Sight Concept Paper v4.0 — or to spawn one more SME round on v0.3 first.*
