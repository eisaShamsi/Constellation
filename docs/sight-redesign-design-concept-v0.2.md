# Sight Redesign — Design Concept v0.2

> **Status**: Proposal, not approved. Awaiting second SME panel review + Boss decision.
> **Date**: 2026-05-13
> **Author**: Claude
> **Supersedes**: `docs/sight-redesign-design-concept-v0.1.md` (kept on disk as historical record)
> **Function in hand**: a five-option redesign of Sight that addresses every structural critique the v0.1 SME panel surfaced.

---

## §1 — Why v0.2 exists

The v0.1 design concept proposed three mock-ups (A · Suwaidi Pure, B · Lens Stack, C · Atmospheric Bands) and a five-channel lens stack (L · C · S · A · P). Six SMEs reviewed it through methodologically distinct lenses. The panel **converged independently on three structural problems** — and these were not fixable by picking a different A/B/C winner. They were problems with the proposal itself.

The three findings:

1. **The five lens channels were not perceptually orthogonal.** Three confound pairs: opacity ↔ library-tint-saturation (both modulate luminance); outer-ring ↔ library-aura (both occupy star periphery); core-pip ↔ adjacent-dot (both small colored marks within the star radius). Three SMEs flagged this independently — Info Design, Cognitive Psychology, DataViz. Mackinlay's ranking suggested only ~2.5 of the five channels were actually decodable.

2. **The "one mode" framing carried cultural payload.** The Cross-Civilizational Epistemology SME's sharpest critique: the radial=stratum spatial grammar smuggles in Aristotelian degrees-of-certainty AND Suhrawardi's hierarchy of presence-knowledge — both *vertical* schemas. Nyāya pramāṇas are *kinds, not levels*. Mīmāṃsā's intrinsic-validity doctrine cannot be radially graded. Polanyi's tacit knowing has no spatial home. "The one-mode frame is itself the smuggled cultural payload Eisa is trying to avoid; honor the goal by inverting the frame."

3. **Sight cannot be only a diagnostic — it has to support navigation.** Both the LIS SME and the UX SME independently said this. Ranganathan's PMEST exists *because* no single ordering serves all queries. Real PKM users live in drill-down after the first 30 seconds. "If Sight ships as pure-glance-only it will be a beautiful screensaver Eisa opens twice a week instead of the diagnostic instrument the concept paper promises."

v0.2 addresses each of these by name. Three fixes, five options, and a new interaction grammar that is not a toggle bar.

---

## §2 — Three confound pairs and how v0.2 fixes them

| v0.1 confound pair | Why it broke perceptually | v0.2 fix |
|---|---|---|
| Opacity (confidence) ↔ tint saturation (library) | Both modulate luminance — Cleveland & McGill, Ware: same retinal feature, neither pops pre-attentively | Library moves to **shape** (circle, square, diamond, triangle, hexagon). Opacity stays for confidence with no competitor. |
| Outer ring color (provenance) ↔ library aura | Both occupy star periphery — eye cannot separate ring from rim | **Provenance is removed from the always-on star encoding.** It moves to a legend filter chip (filter the dome by source bucket) AND becomes a first-class small-multiple in Option D / a register selector in Option E. The provenance signal is no longer on every star; it is summoned. |
| Core pip color (stage) ↔ adjacent dot color (acts) | Both are small colored marks within the star radius — perceptually identical operations on the same glyph region | **Acts moves to size or motion.** Top-decile-act notes are rendered 30% larger, OR (in production) with a slow pulse animation. Stage stays as the inner pip hue, with no competitor. |

### Net result: the v0.2 channel taxonomy

| Channel | Visual property | Orthogonality claim |
|---|---|---|
| **Stratum** (5 levels: Foundation → Edge of Knowing) | Radial position | Strongest channel for ordinal data (Mackinlay 1986); locked. |
| **Time** (12 months) | Angular position | Locked — Suwaidi reference. |
| **Library** (5 identities visible; more clustered by color family) | **Shape** (circle, square, diamond, triangle, hexagon) | Shape is pre-attentive (Treisman primitive) and orthogonal to color and luminance. |
| **Confidence** (4 levels: hypothesis → evidence → established → contested) | Opacity 0.4 → 1.0; saturated for evidence, slightly desaturated for contested | Opacity has no other claimant in v0.2. |
| **Stage** (5 lifecycle: established / fresh / growing / at-risk / dormant) | Inner pip hue (5 well-separated colors: green / cyan / violet / yellow / gray) | Small colored center; foveal when reading. |
| **Acts** (binary: top-decile act-density flag) | Size +30% relative to baseline (or slow pulse in production) | Size/motion is pre-attentive and orthogonal to the other channels. |
| **Typed links** (9 kinds) | Line color between linked stars | Lines are spatially distinct from stars; cannot be confused with star properties. |
| **Provenance** (5 buckets) | **NOT** on the always-on dome. Surfaced via: (a) legend filter chip, (b) Option D's mini-dome, (c) Option E's masādir register. | Removed from the perceptual budget of the main view. |

**Total channels visible at one glance: 6** (stratum + time + library + confidence + stage + acts; plus typed-link lines when present). Provenance is one click away. This is what the Cognitive Psychology SME meant by "2–3 pre-attentive + the rest on focused inspection" — but raised to 6 by re-distributing the load across truly orthogonal channels.

---

## §3 — The progressive-disclosure gesture grammar (replacing all toggle bars)

v0.1 oscillated between "no chrome" (A, C) and "toggle bar" (B). The UX SME and the Cognitive Psychology SME both surfaced the same fix independently: **a single progressive-disclosure gesture, not a control bar.**

v0.2 commits to this universally. **No toggle bar exists in any v0.2 option.** The interaction grammar is:

| Gesture | Effect |
|---|---|
| **Right-click a stratum band** | Other strata dim to 0.3 opacity; selected stratum stars stay at full encoding. Esc to release. |
| **Click a library shape in the legend** | Other library shapes fade to 0.3 opacity; selected library stays full. Click again to release. |
| **Hover a stratum band (no click)** | Small popover appears showing that stratum's diagnostic: note count, confidence average, stage distribution, acts %, link density. Disappears when cursor leaves. |
| **Hover a star** | Side popover with full lens breakdown for that note: stratum, library, confidence, stage, acts, provenance, link list. |
| **Click a star** | Open that note (navigate out of Sight to the editor). |
| **Esc** | Reset to default all-on view, close any popover. |
| **Cmd/Ctrl-F** | Open a search overlay that highlights matching stars and dims non-matches. |

That is the entire chrome. No buttons. No mode switches. No persistent panels. The dome answers "is my universe healthy?" at a glance; gestures answer "where exactly is the problem?" on demand.

---

## §4 — Five design options

Each option implements the channel taxonomy of §2 and the gesture grammar of §3. They differ on **presentation philosophy** and **whether they admit a second view** beyond the central dome.

### Option A · Suwaidi Pure (revised from v0.1)

**File**: `docs/sight-redesign-v0.2-mockA-suwaidi-pure.svg`

**Philosophy**: trust the data. Minimum chrome. All six channels on the dome at low-key opacity. No companion view. Gestures from §3 are the entire interaction.

**Visual language** (v0.2 changes from v0.1):
- Library encoded as **shape** instead of tint gradient. The same five hues survive (so libraries are still visually distinct by color), but the *primary* discrimination is now shape — a colorblind user still reads library identity.
- Stars with top-decile acts are visibly larger (radius 5–6 px vs baseline 3–4 px).
- No outer provenance ring on individual stars. The legend strip at bottom-left lists the five provenance buckets as filter chips.
- Same Suwaidi minimal-chrome aesthetic, same calendar rim, same stratum reference circles.

**SME concerns addressed**: ✓ Channel orthogonality fixed (shape, opacity, hue, size — Bertin variables, non-conflicting). ✓ No toggle bar. ✓ Pre-attentive gestalt now reliably reads stratum (position) + library (shape) + acts (size) + cluster density (gestalt) — four channels in <250ms, per Treisman.

**Remaining concern**: still a single view. LIS SME's "diagnostic dome + faceted-browse companion" is not satisfied. UX SME's "screensaver" risk persists if user wants navigation beyond gestures.

### Option B · Quiet Detail (revised from v0.1 Lens Stack — toggle bar removed)

**File**: `docs/sight-redesign-v0.2-mockB-quiet-detail.svg`

**Philosophy**: same dome as A, but with a **summoned-on-demand diagnostic panel** that slides in from the right. The panel appears only when the user hovers a stratum or focuses a star; it slides away on Esc or pointer-leave. No persistent right rail.

**Visual language**:
- Identical dome to A.
- When a stratum is hovered, a thin right-side card appears: stratum name, note count, confidence avg, stage breakdown bar, acts %, top-3 typed links.
- When a star is clicked, the card swaps to that star's lens detail.
- The card has no toggle controls — it is read-only.

**SME concerns addressed**: ✓ All channel-orthogonality fixes from A. ✓ No toggle bar. ✓ Diagnostic readout is **at the moment of need**, not always-on.

**Remaining concern**: still a single view of the universe. Linked drill-down on demand, not facet navigation.

### Option C · Subtle Bands (revised from v0.1 Atmospheric Bands)

**File**: `docs/sight-redesign-v0.2-mockC-subtle-bands.svg`

**Philosophy**: stratum gets gentle visual emphasis through a **luminance-monotonic** band tint (not the warm/cool atmospheric ramp v0.1 used). Reading is gestalt-first; the band tint helps pre-attentive stratum identification without imposing temperature semantics.

**Visual language** (v0.2 changes from v0.1):
- Stratum tint reduced from 0.18 to **0.06 opacity** (panel recommendation).
- Color ramp replaced: **dark-gray → light-gray, monotonic luminance**. No warm-cool, no cultural-payload temperature gradient. The Cross-Civ SME's specific critique of v0.1 C is fixed.
- Aurora glow removed from all stars (panel said it caused performance + decorative drift).
- Same channel encoding as A (shape, opacity, hue, size).
- Right-edge minimal legend.

**SME concerns addressed**: ✓ Channel orthogonality. ✓ Cultural-neutrality on the stratum ramp (luminance, not temperature). ✓ DataViz performance concerns (no glow filter). ✓ Constraint-as-design preserved.

**Remaining concern**: visual distinctiveness is reduced relative to v0.1 C (which was C's strongest virtue). C now sits between A and the others; its identity is less marked.

### Option D · Coordinated Views (NEW — DataViz SME's proposal)

**File**: `docs/sight-redesign-v0.2-mockD-coordinated-views.svg`

**Philosophy**: instead of cramming everything onto one dome, **render the universe as a small-multiples coordinated view** — one anchor dome on the left + four mini-domes on the right, each isolating one lens with its optimal channel. Vega-Lite / Observable Plot idiom. Linked brushing: hovering or clicking a star in any view highlights it in all.

**Visual language**:
- Left 60% of canvas: anchor dome with stratum × time × library shape + typed-link lines. Same encoding as A.
- Right 40%: 2×2 grid of mini-domes, each ~240×240 px:
  - **Confidence mini-dome** — opacity-only encoding, no other channels rendered.
  - **Stage mini-dome** — hue-only, larger pips, no other channels.
  - **Acts mini-dome** — size-only, binary on/off (top decile = filled large; rest = small dots).
  - **Provenance mini-dome** — 5 colored sectors corresponding to the 5 source buckets; stars positioned in their sector.
- Linked brushing visible: one star highlighted in the anchor dome is also highlighted in all four mini-domes.

**SME concerns addressed**: ✓ Every channel gets its own optimal visual property (no confounds). ✓ DataViz SME's exact proposal. ✓ Cognitive psychology load reduced (each mini-dome is a single-feature search, O(1) regardless of set size). ✓ Provenance is given first-class real estate without overloading the anchor dome.

**Remaining concern**: visual fragmentation. UX SME's first-touch comprehension might dip — the user has more places to look. Suwaidi fidelity drops (the Suwaidi chart was one circle, not five). Constraint-as-design tension: more surface, more chrome.

### Option E · Tradition Registers (NEW — Cross-Civ SME's proposal)

**File**: `docs/sight-redesign-v0.2-mockE-tradition-registers.svg`

**Philosophy**: the cultural-neutrality problem (v0.1 finding #2) is solved not by removing cultural framing but by **making the cultural frame explicit and switchable**. The user picks the epistemic tradition; the dome remaps accordingly. Three registers in v0.2:
- **pramāṇa register** — Indian Nyāya epistemology. The dome divides into 4 quadrants by source-kind (pratyakṣa NE, anumāna SE, upamāna SW, śabda NW); radial position within each quadrant = certainty in *that* source's terms; angular within quadrant = time.
- **masādir register** — Sunni *uṣūl al-fiqh* epistemology. The dome's strata become concentric authority levels (Qur'an inner → sunnah → ijmāʿ → qiyās outer); angular = time.
- **Polanyi register** — modern Western pluralism. The dome shows a tacit/explicit gradient as a fog overlay (tacit knowledge fades as fog; explicit knowledge stands clear). Stratum bands remain but are visually muted.

The register selector is **one small chip** on the title bar. Not a toggle bar. Picking a register remaps the dome's semantic axes; the underlying note set is unchanged.

**Visual language**:
- Three register chips at top center: [pramāṇa] [masādir] [Polanyi] — current highlighted.
- Main canvas: the full-size active register (pramāṇa shown in mock).
- Two small thumbnails at top showing the alternative registers (so the user can see at a glance what's available).
- Channel encoding (shape, opacity, etc.) carries over; only the spatial semantics shift per register.

**SME concerns addressed**: ✓ Cultural neutrality (Cross-Civ SME's direct ask). ✓ Pluralism made operational. ✓ Conceptual rigor — each tradition gets its own geometry instead of being flattened. ✓ "Switchable epistemic registers" — the panel's exact language.

**Remaining concerns**: UX SME may flag this as academic / scholarly. First-touch comprehension is the lowest of the five options because the user must understand what a "register" is. Distinctiveness is the highest of all options — no other PKM does anything remotely like this. Implementation cost is the highest.

---

## §5 — Comparison matrix (five options)

| Criterion | A · Suwaidi Pure | B · Quiet Detail | C · Subtle Bands | D · Coordinated Views | E · Tradition Registers |
|---|---|---|---|---|---|
| **Story-at-glance** | High (gestalt) | High (gestalt + summoned detail) | High (gestalt + redundant stratum) | Medium (multi-view) | Variable per register |
| **Suwaidi fidelity** | Highest | High | Medium-High | Low | Medium (one register) |
| **First-touch (no docs)** | Medium | Medium-High | High | Medium | Low |
| **Drill-down support** | Gesture | Gesture + summoned panel | Gesture | Linked-brushing across 5 views | Register-switch |
| **Channel orthogonality** | High (fixed in v0.2) | High | High | Highest (each in own view) | High |
| **Visual learning curve** | Medium | Low | Low | Medium-High | High |
| **Performance @ 10k notes** | Excellent | Excellent | Excellent | Good (5 views) | Excellent |
| **Distinctiveness vs other PKMs** | High | Medium | High | Medium | Highest |
| **Cultural neutrality** | Medium | Medium | High | Medium | Highest |
| **Risk of decorative drift** | Low | Low | Low | Low | Medium |
| **Constraint-as-design** | Strongest | Strong | Strong | Medium | Strong (cultural frame is explicit, not implicit) |
| **Implementation cost** | Low | Medium | Low | Medium-High | Highest |

---

## §6 — Open questions for Boss

1. **Single view vs. coordinated views.** Is the small-multiples idiom (Option D) a serious contender, or does it fracture the "one look, whole story" goal you set?
2. **Tradition-explicit pluralism.** Is Option E's switchable epistemic registers a feature (deepest expression of Constellation's cross-civ commitment) or a liability (too academic for daily use)?
3. **Diagnostic-only vs. diagnostic + navigation.** Should Sight evolve into the navigation surface too (Option D's mini-domes can each filter the main view), or should it stay as a pure diagnostic instrument with editor-jump being the only navigation gesture?
4. **Hybridization.** Many hybrids exist: A + Option E's register selector (the most narrowly-scoped hybrid); B + Option D's mini-domes for provenance only; C + register selector. Is your preference a clean option or a hybrid?
5. **Library scale.** Five shapes for five libraries works. Above 8–10 libraries, shape vocabulary exhausts. v0.3 may need a clustering or letter-tag fallback. Is this a v0.2 blocker or a v0.3 concern?
6. **Color accessibility.** Stage hue + library shape + provenance filter chip + typed-link colors = three color channels active. A high-contrast / colorblind-safe variant is doable in v0.3 if the answer is "yes, plan for it."

---

## §7 — Second SME panel review protocol

The same six SMEs from v0.1 round 1 are being re-engaged. Each receives:
- v0.2 design concept doc (this file)
- v0.1 design concept doc (historical context)
- All five v0.2 mocks (revised A/B/C + new D/E)
- Their own v0.1 verdict (pasted into the brief) so they can specifically address whether v0.2 resolved their previous critiques

Each round-2 SME returns:
- A re-scored matrix across all five options on their three dimensions
- A direct yes/no on whether their v0.1 critiques are resolved
- A revised recommendation (winner / hybrid / further-iteration call)
- One paragraph of any new concerns introduced by v0.2's changes

I will synthesize the round-2 panel into a single ranked verdict for you to weigh against your own read.

---

## §8 — What v0.2 still does not commit to

- Which option (A / B / C / D / E / hybrid) wins. That is your call after the round-2 panel.
- Implementation surface (canvas-2D / SVG / WebGL). Performance modelling at 100,000 notes hasn't been done; v0.2 picks Canvas-2D as default but reserves WebGL for D + animated pulses.
- Color accessibility variant. Deferred to v0.3 unless the panel surfaces it as a blocker.
- Whether v4.0 architecture supersedes v5 entirely or coexists during a transition. Deferred until a winner is picked.
- The Sight Concept Paper v4.0 itself. v0.2 is design discussion; the formal contract comes after convergence.

---

## §9 — What's locked in v0.2 (will not be re-litigated)

The following are committed in v0.2 regardless of which option wins, because the round-1 SME panel agreed on them across lenses:

- **Channel orthogonality discipline**: no two channels share a Bertin visual variable. The §2 taxonomy is the floor.
- **No toggle bar**: gestures replace persistent mode controls.
- **Stratum × time as the spatial anchor**: Suwaidi reference holds (modulated per register in Option E).
- **The one-mode framing is open for question**: Options D and E offer alternatives. v0.2 does not insist on one mode.

---

## Appendix A — Files

| File | Purpose | Version |
|---|---|---|
| `docs/sight-redesign-design-concept-v0.2.md` | This document | v0.2 |
| `docs/sight-redesign-design-concept-v0.1.md` | Previous version (historical) | v0.1 |
| `docs/sight-redesign-v0.2-mockA-suwaidi-pure.svg` | Mock A revised | v0.2 |
| `docs/sight-redesign-v0.2-mockB-quiet-detail.svg` | Mock B revised | v0.2 |
| `docs/sight-redesign-v0.2-mockC-subtle-bands.svg` | Mock C revised | v0.2 |
| `docs/sight-redesign-v0.2-mockD-coordinated-views.svg` | Mock D NEW (DataViz SME proposal) | v0.2 |
| `docs/sight-redesign-v0.2-mockE-tradition-registers.svg` | Mock E NEW (Cross-Civ SME proposal) | v0.2 |
| `docs/sight-redesign-mockA-suwaidi-pure.svg` | v0.1 (kept for reference) | v0.1 |
| `docs/sight-redesign-mockB-lens-stack.svg` | v0.1 (kept for reference) | v0.1 |
| `docs/sight-redesign-mockC-atmospheric-bands.svg` | v0.1 (kept for reference) | v0.1 |

## Appendix B — Cross-references

- `docs/Constellation-Sight-Concept-Paper-v3.1.md` — current contract (potentially superseded)
- `docs/sight-v5-mode-concepts.md` — semantic source for the channel encodings
- `docs/sight-v5-purpose-achievement-audit.md` — origin of the redesign moment
- `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` — the Five Acts, the 7+2 link types

---

*End of v0.2. Awaiting round-2 SME panel review + Boss direction.*
