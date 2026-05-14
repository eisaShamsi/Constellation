# Sight Redesign — Design Concept v0.1

> **Status**: Proposal, not approved. Awaiting Boss review + SME agent panel.
> **Date**: 2026-05-13
> **Author**: Claude
> **Supersedes (if approved)**: `docs/Constellation-Sight-Concept-Paper-v3.1.md` and the seven-mode v5 architecture currently on `main`.
> **Function in hand**: redesign Sight so the dashboard tells the whole story at one glance — Suwaidi-style — instead of forcing the user to toggle seven modes to assemble the cognitive portrait piece by piece.

---

## §1 — Why we're considering a redesign

Three things came together yesterday and today that pushed me to re-open the design:

1. **The purpose-achievement audit** (`docs/sight-v5-purpose-achievement-audit.md`) verified that v5 passes a capability test — every mode can technically compute and render its diagnostic. The audit did **not** verify that v5 passes the *outcome* test: can the user, on opening Sight, see whether his universe is healthy and where he needs to act?
2. **The Suwaidi star chart**. Eisa surfaced this image as the design north star: one circular field, calendar rim, strata as concentric guides, stars with brightness/size variation, constellation patterns visible. **One look. Whole story.** No toggle. No modes.
3. **Eisa's two-line verdict on v5 after the fix-7/8/9/10 + D-V10 ship**:
   - *"Looking at the modes, it is confusing! Again, what is Sight telling?"*
   - *"What is unique about Sight? Based on the modes. NOTHING!"*

The honest read: the v5 seven-mode toggle (R · L · T · C · S · A · P) gives the user 1/7th of the cognitive portrait at any moment. The whole portrait only assembles if the user clicks through every mode in his head and stitches the impressions together. That is not story-at-a-glance — that is *story-at-a-workflow*. And every other PKM with a graph view has variants of the same pattern: a toolbar of toggles over a single view. Whatever uniqueness Constellation Sight had as a concept got diluted into a familiar dashboard shape.

The redesign target: a Sight surface that, **at first glance** (under 30 seconds of looking, no docs read, no controls touched), tells the user:

- where the cognitive density is (which strata are full, which are empty);
- how confident the universe is (mostly bright or mostly dim);
- where the orphans are (notes with no links visible as outliers);
- when the last surge of thinking happened (recent month wedge densely populated, distant past sparse);
- which library dominates the dome (color clusters);
- whether the universe is in steady growth, dormancy, or imbalance.

That is the test. If the dome can answer those six things without a mode click, the redesign succeeds.

---

## §2 — The one-mode hypothesis (Eisa's framing)

Eisa's exact ask, in his words:

> *"Simplicity resides in Clarity. Let's assume that the Sight is a one-mode construction. How can we achieve simplicity, yet the sight tells the whole story at a glance? I want your honest input."*

The hypothesis I proposed and he accepted: **one mode, multiple visual channels overlaid simultaneously**. Spatial position is the anchor (strata × time, just like the Suwaidi chart). Library color is always on. The five remaining lens dimensions (L · C · S · A · P) are encoded as **distinct visual properties** of each star — opacity, core hue, ring color, glow, etc. — so they all read at once without overplotting.

This is the inverse of v5's design. v5 said: *one channel at a time, switched via toggle*. The redesign says: *all channels at once, distinguished by orthogonal visual properties*. Same data, opposite reading mode.

The "R Regions" mode of v5 is gone in the redesign — Region (library) becomes the always-on tint, not a toggleable view. The "T Time" mode is gone — Time becomes the always-on angular position. The toggle bar is gone.

---

## §3 — What is locked across all three options

The three mock-ups differ on **presentation philosophy**. They agree on the **spatial grammar** and **lens semantics**. This is non-negotiable in the design concept and the SME panel should not question it (it is the part Eisa already approved):

### 3.1 Spatial anchor

| Axis | Encoding | Why |
|------|----------|-----|
| Radial position | Stratum band (Foundation → Working → Connection → Synthesis → Edge of Knowing) | Cognitive maturity is the dominant epistemic dimension; radial position is the most pre-attentive spatial channel |
| Angular position | Month-of-creation, January at top, clockwise | Matches the Suwaidi calendar rim; gives the user a wall-clock sense of "when did I think about this?" |
| Tint | Library color | Eisa's libraries are the user's macro-organization; tint always on means library identity is never hidden |

### 3.2 Lens channels (five, all-on)

| Lens | Encoded as | Diagnostic Reading |
|------|------------|---------------------|
| **L · Links** | Constellation lines between linked stars; line color = link type (supports green, contradicts red, causes orange, exemplifies blue, generalizes purple, derives-from cyan, part-of pink, associative gray, supersedes pale yellow) | Cluster shapes reveal cognitive topology; absence of lines flags orphans |
| **C · Confidence** | Star brightness (opacity 0.4 hypothesis → 1.0 established; saturated for evidence, slightly desaturated for contested) | Dim universe = uncertain knowledge state; bright universe = consolidated knowledge |
| **S · Stage** | Tiny core pip color in each star (green established · blue fresh · violet growing · yellow at-risk · gray dormant) | Reveals the lifecycle distribution: too many gray pips = stagnation, too many yellow = neglect, healthy mix = active universe |
| **A · Acts** | Small adjacent dot or aura spike near each star, present only when act-density on that note is in the top decile (Observation / Connection / Tension / Synthesis / Conviction) | Surfaces *where* the user is actively engaged in knowledge formulation right now |
| **P · Provenance** | Thin outer ring color (one of the 11 sources from the Universal Epistemic Content Taxonomy collapsed to 5 visible buckets: Self · Read · Heard · Reasoned · Tradition) | Reveals source diversity — single dominant ring color = over-reliance on one source |

### 3.3 What is intentionally absent

- **No mode toggle**. The toggle is the v5 concession the redesign is rejecting.
- **No 7-button bar**. R and T are no longer modes — they are always-on axes.
- **No "scope" U/L/F toggle in the mockup top bar** (it can return as a contextual control if needed, but it is not on the at-a-glance critical path).
- **No layer-2 panel slide-in for this design phase**. The redesign is layer 1 + layer 2 fused into one read. Layer 3 (recommendations) and Layer 4 (coaching) are downstream; not part of this concept.

---

## §4 — Three design options

The three mock-ups (linked below) explore three different ways to express the same lens stack. Each has a name that captures its philosophy.

### Option A · Suwaidi Pure

**File**: `docs/sight-redesign-mockA-suwaidi-pure.svg`

**Philosophy**: trust the data. Minimum chrome. All five lens channels always on at *low-key opacity*. No toggle bar. No isolation. The dome speaks the way the Suwaidi star chart speaks — through the gestalt, not through controls.

**Visual language**:
- Deep black background (`#080c16`).
- Strata circles drawn as the thinnest possible reference rings — visible only as faint guides, like the celestial reference circles in the Suwaidi chart.
- Stars small (radius 2–4 px), library tint subdued, channel encoding low-saturation.
- Calendar rim labels in mid-gray, not competing with the data.
- Constellation lines very faint (`#2c3450` at 60% opacity).
- Tiny corner legend explaining the encoding once.
- Top-right diagnostic indicator (healthy/at-risk/dormant percentages) in lo-key gray.

**Pros**:
- Closest visual fidelity to the Suwaidi reference. If Eisa's north star is *that image*, this is the closest match.
- Data-density is unfiltered — the user sees everything simultaneously.
- No controls to learn; the dome is the entire interface.
- Pre-attentive patterns (cluster density, empty strata, color dominance) read instantly.

**Cons**:
- Steep visual learning curve. The user must look at the legend at least once to know what brightness means.
- Subtle channels (provenance ring, acts adjacent dot) can be lost at small star size.
- No way to "isolate" a single dimension if the user wants to inspect one channel alone.
- If a universe is in a degenerate state (e.g., all dormant), the dim/desaturated read could be confused with under-rendering or a bug.

### Option B · Lens Stack

**File**: `docs/sight-redesign-mockB-lens-stack.svg`

**Philosophy**: trust the user. Five lens channels stack on the dome by default, but a bottom pill bar lets the user **isolate** any one channel — turn the others to grayscale — to drill into a single dimension. The dome is the same one-mode anchor; the chrome supports progressive disclosure.

**Visual language**:
- Slightly lighter background (`#0d1322`).
- Strata circles more visible (mid-gray, 0.9 px stroke).
- Stratum labels on the left side of the dome with light dashed connector lines.
- Stars medium (radius 4–6 px), library tint vivid, each channel expressed boldly.
- Constellation lines colored by link type, full opacity.
- **Bottom pill bar**: five toggles `L · C · S · A · P` — clicking one isolates it (others go grayscale).
- **Right-side detail panel**: when a lens is isolated, the panel shows the per-category breakdown for that lens (e.g., L isolated → counts of supports / contradicts / causes / ... + a ratio diagnostic + a recommendation CTA).
- Top-right diagnostic indicator.

**Pros**:
- Best of both worlds: gestalt read + drill-down on demand.
- The right-side panel is where layer-2 diagnostic computations land naturally without adding a slide-over.
- Power-user friendly; matches PKM/dashboard conventions users already know.
- The isolated-lens view is itself a strong visual story.

**Cons**:
- Brings back a toggle bar, which is the very mechanism the redesign was trying to escape. If a user lives in "isolate L" mode 99% of the time, they're back in v5's situation.
- Chrome competes with content for attention (the right panel and pill bar are heavy).
- The "all-on" default state can feel less curated than Option A or C — like an unfiltered dashboard.
- Visual decisions doubled: the dome AND the panel need to coordinate, which is more design surface to keep coherent.

### Option C · Atmospheric Bands

**File**: `docs/sight-redesign-mockC-atmospheric-bands.svg`

**Philosophy**: trust the metaphor. Strata bands get **translucent atmospheric color** (warm orange-red at the Foundation core → cool violet-white at the Edge of Knowing periphery). Stars carry **aurora glow** — outer halo rings, core temperature, glow intensity — instead of simple opacity. No toggles. The dome is read like a cosmic photograph.

**Visual language**:
- Radial gradient background (deep navy core → black edges) so the dome already *feels* like a sky.
- Strata as translucent colored zones (`Foundation` = warm orange, `Working` = amber, `Connection` = blue-cyan, `Synthesis` = purple, `Edge of Knowing` = violet-white).
- Stars with multi-layer aurora: outer fade halo + middle library-tinted glow + inner colored core + tiny stage pip.
- Constellation lines as glowing arcs (Gaussian-blur filter for soft luminescence).
- Calendar rim in atmospheric light gray.
- Right-edge vertical legend in "atmospheric" style — colored squares for each stratum, then a stack of channel keys.

**Pros**:
- Most distinctive — no other PKM looks like this. Strong product identity moment.
- Strata as colored bands make stratum-position pre-attentively obvious, even before the user reads any label.
- Star aurora gives more visual real estate per star, so subtle channels (provenance, acts) can be encoded without overplotting.
- Story-at-glance is reinforced by atmospheric gestalt — a "cold and quiet" universe and a "warm and active" universe look visibly different at the macro level.

**Cons**:
- Color-encoding bandwidth is highest. If a colorblind user can't see the strata band differences, the whole anchor weakens.
- Risk of decorative drift — atmospheric color can read as "pretty" rather than informative.
- Performance: gradient fills + blur filters + per-star aurora is more expensive than Option A's flat circles. At 7,000+ stars this needs canvas-2D care.
- Strongest visual style commitment — if Eisa wants a more clinical / scholarly aesthetic, Option C is the wrong direction.

---

## §5 — Comparison matrix

| Criterion | Option A · Suwaidi Pure | Option B · Lens Stack | Option C · Atmospheric Bands |
|-----------|------------------------|----------------------|------------------------------|
| **Story-at-glance** | Strong (gestalt read) | Medium (default view) / Strong (isolated) | Strong (atmospheric gestalt) |
| **Fidelity to Suwaidi reference** | Highest | Lower (chrome shifts dominant) | Medium (different metaphor but star-chart spirit) |
| **First-touch comprehension (no docs)** | Medium — legend needed once | Medium — needs to discover toggle behavior | High — atmosphere does the teaching |
| **Drill-down support** | None | Excellent | None |
| **Channel orthogonality** | Excellent (opacity, hue, ring) | Excellent | Excellent + glow adds bandwidth |
| **Visual learning curve** | Medium | Low (familiar dashboard pattern) | Medium |
| **Performance @ 10k notes** | Excellent (flat fills) | Excellent (flat fills) | Demanding (gradients + blur) |
| **Distinctiveness vs other PKMs** | High (Suwaidi look) | Medium (looks like a dashboard) | Highest (atmospheric, unique) |
| **Risk of decorative drift** | Low | Low | Medium |
| **Aligns with "constraint as design" principle** | Strongest | Weakest (chrome added back) | Medium |
| **Bandwidth for future Layer-3 recs / Layer-4 coaching** | Needs separate surface | Right-panel ready | Needs separate surface |

---

## §6 — Open questions for Boss

Before approving a winner (or a hybrid), there are a handful of decisions that change which option is correct. I am *not* asking you to answer these one at a time — list them so the SME panel can weigh them and so you can answer them in one pass when you reply.

1. **Drill-down vs. pure-glance.** Do you want the ability to isolate a single lens (Option B's strength) or is the gestalt always the answer (Options A and C)?
2. **Decorative tolerance.** Is the atmospheric color of Option C an asset (gives the universe a *feel*) or a distraction (color that doesn't carry data)?
3. **Performance trade.** Option C's gradients + glow at 7,000+ stars may need optimization; Options A and B are guaranteed fast. Is "looks distinctive" worth the engineering tax?
4. **Chrome budget.** Option B brings back a toggle bar and right-side panel — chrome the redesign was trying to escape. Is the drill-down power worth it, or does the chrome violate the simplicity-through-clarity principle that started this redesign?
5. **Color accessibility.** Library tint + stage hue + provenance ring + constellation line color = four simultaneous color channels. Should I plan a high-contrast / colorblind mode as part of v0.2 of this concept?
6. **Where do recommendations and coaching go?** Layer 3 (recommendations) and Layer 4 (coaching) from the v3.1 architecture are not in any of these mocks. Should they fold into Option B's right panel, or stay as a separate sidebar / Sight v.next surface?

---

## §7 — SME panel review protocol

In parallel with delivering this doc to you, I'm spawning six SME agents to give honest opinions about the three options. Each SME reviews **all three mock-ups + this doc** through a methodologically distinct lens and returns a scored, recommended option. The intent is to surface bias I'd miss working alone.

| SME | Lens | What they're checking |
|-----|------|----------------------|
| **Information Design** | Visual hierarchy, channel encoding orthogonality, Tufte-style data-ink ratio | Does the strongest data use the strongest channel? Are channels confusable? |
| **Cognitive Psychology** | Perceptual chunking, working memory load, pre-attentive processing | How many channels can a user track at once? What reads in <250ms? |
| **Library / Information Science** | Controlled vocabularies, browsable taxonomies, classification semantics | Does the visualization respect the Universal Epistemic Content Taxonomy? Can a user navigate the corpus? |
| **Data Visualization** | D3/Vega-Lite practice, Mackinlay encoding hierarchy, interactive grammar | Are the encoding choices defensible by accepted data-viz theory? Does it scale 100 → 10,000 notes? |
| **Cross-Civilizational Epistemology** | Pluralism of pramāṇa / masādir / Polanyi, civilizational neutrality | Does any option lean on a single tradition's metaphor? Does the visual respect the taxonomy's plural roots? |
| **End-User UX** | PKM workflows (Obsidian, Logseq, Roam, Tana, Capacities) | First-touch comprehension, competitive distinctiveness, workflow fit |

Each SME returns:
- 1–5 score per option on their own dimensions;
- Each option's strongest and weakest point;
- A recommended winner OR a recommended hybrid;
- One paragraph of honest critique of my proposal itself.

I will summarize the panel into a single ranked verdict for you to weigh against your own read.

---

## §8 — Decision points awaiting you

After reading this doc and the three mocks, the cleanest decision sequence is:

1. **Is the one-mode hypothesis itself right?** If a different architectural premise (e.g., two coexisting views, or a 360-style per-note focus mode promoted to universe scale, or something I haven't proposed) is closer to your vision, say so now. The mocks all assume the one-mode hypothesis.
2. **Which of A / B / C / hybrid?** Pick a winner or sketch a hybrid (e.g., "A's chrome restraint + B's right panel only when summoned by hotkey" or "C's atmosphere + A's minimal legend").
3. **What's still missing from the chosen design?** Anything in the open-questions list (§6) that needs resolving before I write the new Sight Concept Paper.

Once we converge, I draft the new **Constellation Sight Concept Paper v4.0** (or whatever version number you want it to carry) as the formal contract — and that becomes the spec the next MIG implements against.

---

## §9 — What this doc is not

- It is **not** a build plan. The current main carries the v5 seven-mode architecture and the post-§N fixes (fix-7/8/9/10 + D-V10). Nothing is being removed without your approval of a redesign direction.
- It is **not** a commitment to any of A/B/C. All three are sketches in service of a conversation.
- It is **not** the new Sight Concept Paper. That comes after you converge on a direction and approve.
- It is **not** an admission that v5 is "wrong." v5 passes the capability test. The redesign question is whether v5 meets your *outcome* test (story at one glance), and that is what we're stress-testing here.

---

## Appendix A — Files

| File | Purpose |
|------|---------|
| `docs/sight-redesign-design-concept-v0.1.md` | This document |
| `docs/sight-redesign-mockA-suwaidi-pure.svg` | Mock A — minimum chrome, all channels on, Suwaidi fidelity |
| `docs/sight-redesign-mockB-lens-stack.svg` | Mock B — lens toggles + right panel, drill-down |
| `docs/sight-redesign-mockC-atmospheric-bands.svg` | Mock C — atmospheric strata, aurora stars |

## Appendix B — Cross-references

- `docs/Constellation-Sight-Concept-Paper-v3.1.md` — current contract (potentially superseded)
- `docs/sight-v5-mode-concepts.md` — what each of the seven v5 modes was meant to encode (still the semantic source for lens channels)
- `docs/sight-v5-purpose-achievement-audit.md` — why we're at this redesign moment
- `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` — the Five Acts, the 7+2 link types, the Living Link Architecture
- (Suwaidi star chart) — the reference image Eisa cited; treated as design north star

---

*End of v0.1. Awaiting Boss + SME panel review.*
