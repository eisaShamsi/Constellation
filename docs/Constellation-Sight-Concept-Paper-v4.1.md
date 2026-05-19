# Constellation Sight — Concept Paper

**Version 4.1 | 2026-05-18**

> **What changed in v4.1** (MIG-026 SHIP — Sight v6.3 24-tradition expansion + 9 shape renderers + user-definable plugin layer + full 15-locale localization)
>
> v4.0 (2026-05-13) specified the Sight v6 architecture as a coordinated-views design built around an Aristotelian anchor dome plus a tradition chip carrying seven cultural framings (Aristotelian + pramāṇa + masādir + Polanyi at production polish, plus three "v1 preview" traditions — Dignāga, Suhrawardi Ishrāqī, Mohist sān biǎo). The build cascade that followed (MIG-025 §A–§C in 2026-05-15 → 2026-05-17, then MIG-026 in 2026-05-17 → 2026-05-18) did three things v4.0 did not anticipate: it **expanded the curated baseline to 24 traditions** (under the religious-lineage rule formalized in orientation v2.09 — Dignāga and Ishrāqī were excluded entirely; nineteen new traditions across seven families were added in their place); it **generalized the rendering pipeline into nine TraditionShape renderers** (sectoral, concentric rings, grid, ladder/spiral, relational hub-and-spoke, cyclic flow, binary flow with three layouts, gradient fog, horizontal bands) so that traditions whose hero metaphor refuses sectoral form can render faithfully; and it **shipped a user-definable plugin layer** in two tiers (declarative JSON for shape-bound custom traditions, full-trust JavaScript plugins with arbitrary remap logic and an Obsidian-style consent banner for shapes outside the four declarative ones).
>
> v4.1 replaces v4.0's §4.1 (four production-polish traditions) with §4.1.1 → §4.1.24 (the canonical 24), and trims v4.0's §4.2 (three v1-preview traditions) to just §4.2.1 (Mohist sān biǎo as the surviving v1-preview chip that has since matured to full ship). A new §3.5 documents the nine shape renderers; a new §3.6 documents the user-definable plugin layer. Invariants 12 and 13 are added to §11 covering i18n labelize and plugin-label passthrough. Two doc-drift items from v4.0 are corrected inline: §4.1.2 (pramāṇa) and §4.1.3 (masādir) sector positions are now E/S/W/N rather than NE/SE/SW/NW, after the §δ.2-fix-1 and §θ-fix-1 rotations cleared the vertical axis from stratum-label collision.

> **Status**: Architectural contract. Ratified-by-shipping on 2026-05-18 (Phase μ ship gate closed; milestone tag `milestone/sight-v6.3-traditions-ship` cut at commit `99e4ed37`).
> **Date**: 2026-05-18
> **Supersedes**: `docs/Constellation-Sight-Concept-Paper-v4.0.md` (kept on disk as historical record). v3.1 (and earlier) likewise remain on disk.
> **Specifies**: Sight v6.3 as currently shipped on `main`. Two version axes:
> - **Concept Paper version**: v4.0 → **v4.1** (this document).
> - **Implementation version**: v6 (shipping at v6.3 as of MIG-026 Phase μ).

---

## §1 — What Sight is and what Sight is not

### 1.1 The canonical question Sight answers

> **"Is my universe healthy? If not, where does it need to be handled?"**

This is the user-facing function of Sight. Every architectural commitment in this paper either supports that question being answered at a glance, or supports the user acting on the answer.

### 1.2 The Suwaidi criterion (acceptance test)

A user, on opening Sight for the first time, must be able to read — without consulting documentation, without clicking any control — within roughly 30 seconds:

1. Where the cognitive density is (which strata are populated, which are empty).
2. How confident the universe is overall (mostly bright = consolidated; mostly dim = uncertain).
3. Where the orphans are (notes with no link affordance, visible as outliers).
4. When the last surge of thinking happened (recent-month wedge populated, distant past sparse).
5. Which library dominates (shape clusters visible).
6. Whether the universe is in steady growth, dormancy, or imbalance (gestalt).

The reference image — the Al-Suwaidi northern-hemisphere star chart Eisa surfaced as the design north star — does this kind of work. Sight v6 must do equivalent work for the user's knowledge universe.

### 1.3 What Sight is not

- **Not a file finder.** That is the editor sidebar, search, and the wiki-link surface.
- **Not a graph view.** Obsidian's graph view is force-directed link soup; Sight's spatial grammar carries semantic meaning (radial = stratum, angular = time, or — under a non-Aristotelian tradition — whatever the tradition's geometry encodes).
- **Not a settings dashboard.** Diagnostic only. No knobs that change behavior; the user acts through standard editor operations.
- **Not a coaching tool in v6.** The four-layer ambition (visual / diagnostic / recommendation / coaching) from v3.1 is preserved as direction but only layers 1–2 ship in v6. Layers 3–4 are post-v6 work.
- **Not single-tradition.** Sight is built on the Universal Epistemic Content Taxonomy and the 24-tradition curated baseline. The cultural frame is *switchable*, not assumed, and v6.3 also lets users **author their own traditions** (see §3.6).

### 1.4 What changed from v4.0 to v4.1

| Aspect | v4.0 / Sight v6.0–v6.1 | v4.1 / Sight v6.3 |
|---|---|---|
| Curated traditions | 7 (4 production-polish + 3 v1-preview, including Dignāga + Ishrāqī) | **24** (under religious-lineage rule; Dignāga + Ishrāqī excluded) |
| Tradition shapes | 1 (sectoral, with Polanyi as anchor-overlay) | **9** (`TraditionShape` union; see §3.5) |
| User-definable traditions | Not in scope | **Two-tier loader** (declarative JSON + full-trust JS plugin); see §3.6 |
| Tradition chip UI | One inline row of N chips | Family-categorized dropdown with ⓘ disclosure modal opening each manifest |
| Tradition manifests | Specified in §4.3 of v4.0, files not yet authored | Shipped at `docs/traditions/<id>.md` (24 files); see §4.1 |
| Theme awareness | Sight painted dark regardless of app theme | MIG-027 wired Sight to the app theme system (chrome + canvas) |
| Localization | English-only chrome + canvas | **Full 15-locale chrome + canvas** localization via `labelize`/`$t` (Phase λ-fix-3/4/5) |

The honest read of v4.0 → v4.1: v4.0 specified the right architecture but underestimated the scholarly debt incurred by the seven-tradition starting set. Three things drove the expansion: (a) the **religious-lineage rule** (orientation v2.09) excluded Dignāga and Ishrāqī as either non-Abrahamic-religious-source or non-Sunni-Islamic, requiring a substitution menu; (b) Eisa's choice from the candidate research (orientation v2.10) locked nineteen replacements across seven additional families, raising the count to 24; (c) several of those nineteen had hero metaphors that simply could not render as sectoral wedges (Maimonidean prophecy's eleven-step ladder, Dussel's totality/exteriority concentric flow, Mignolo's hub-and-spoke), forcing the rendering pipeline to grow nine shapes. The two-tier user-definable loader (Phase κ.1 + κ.2) was the final move: it lets users add their own traditions without code commits and admits arbitrary remap logic via the JS plugin layer, so the curated 24 are a starting set rather than an enclosure.

---

## §2 — Architecture: Coordinated Views (Option D, locked)

### 2.1 Four zones on one screen

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  Constellation Sight                                                          │
│  [Aristotelian ●]  (collapsed tradition chip — click to expand)                │  ← title bar
├────────────┬──────────────────────────────────┬──────────────────────────────┤
│            │                                  │  ┌──────────┐  ┌──────────┐  │
│  Filters   │                                  │  │ CONFID.  │  │  STAGE   │  │
│      ▶     │           ANCHOR DOME            │  │ opacity  │  │ hue only │  │
│            │     stratum × time × shape       │  └──────────┘  └──────────┘  │
│ (collapsed │     + typed-link lines           │  ┌──────────┐  ┌──────────┐  │
│  sidebar — │                                  │  │  ACTS    │  │ PROVEN.  │  │
│  click to  │                                  │  │ size only│  │ 5 sectors│  │
│  expand)   │                                  │  └──────────┘  └──────────┘  │
│            │                                  │ (mini-domes hidden by default│
│            │                                  │  — Cmd-D / "Show diagnostics │
│            │                                  │  to reveal)                  │
├────────────┴──────────────────────────────────┴──────────────────────────────┤
│  status strip                                                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Default state on first open: only the anchor dome is visible.** Sidebar collapsed to a tab. Tradition chip collapsed to a single label. Mini-domes hidden. This is the Suwaidi-grade view that satisfies §1.2.

**Engaged state**: any of three gestures expand chrome:
- Click sidebar tab → facets unfold (full sidebar visible)
- Click tradition chip label → tradition dropdown expands (family-categorized panel)
- Cmd-D or "Show diagnostics" button → four mini-domes slide in from the right

### 2.2 The anchor dome (left-center, ~640×640 px target render)

The anchor dome is the Suwaidi-grade view. It is non-negotiable in v6 — every other zone supports it.

**Spatial grammar (Aristotelian default; remappable by active tradition):**
- **Radial position** = stratum (Foundation → Working → Connection → Synthesis → Edge of Knowing). Five bands, Foundation innermost.
- **Angular position** = month of creation, January at top, clockwise.
- **Shape** = library identity (circle / square / diamond / triangle / hexagon at 1–5 libraries; outline-style rotation at 6–25; degraded fallback above).

When a non-Aristotelian tradition is active, the renderer overlays the tradition's own geometric grammar — sector dividers, concentric rings, ladder steps, hub-and-spoke clusters, flow arrows, fog gradient, or horizontal bands — and remaps each star's position accordingly (§3.5).

**Channel encoding on each star:**
- **Fill color**: neutral (`#cdd5e0`). No library hue. No exception.
- **Opacity**: confidence (0.4 hypothesis → 1.0 established; saturated for evidence; slightly desaturated for contested).
- **Size**: top-decile acts flag (binary: baseline 5 px, top-decile 7 px). Some shape renderers apply a per-shape size boost (sectoral +2 px, grid +2 px) to maintain pre-attentive size discrimination against the renderer's chrome.
- **Inner pip hue**: stage (green established / cyan fresh / violet growing / yellow at-risk / gray dormant). Pip diameter ≥1.8 px at default zoom; explicitly **focal-on-foveation**, not pre-attentive on the anchor (pre-attentive in the Stage mini-dome).

**Line encoding:**
- **Typed-link connections** as curves between linked stars. Line color = link type:
  - supports = green, contradicts = red (dashed), causes = orange, exemplifies = blue, generalizes = purple, derives-from = cyan, part-of = pink, associative = gray, supersedes = pale yellow.
- Lines auto-fade above 800 visible (prevent overplotting); user can re-enable via Settings.

**Stratum reference circles**: five concentric guides at 0.6 px stroke `#1a1f2e` (very subtle, Suwaidi-style).

**Calendar rim**: 12 month labels at `r ≈ 340 px` in mid-gray. Month labels localize via `labelize`/`$locale` (no longer leaks `navigator.language`).

**Stratum labels**: faint italic text along the vertical axis (FOUNDATION at center, EDGE OF KNOWING at top). Labels resolve through `STRATUM_LABEL_KEYS` at `sight.v6.stratum.<band>`.

### 2.3 The four mini-domes (2×2 grid, hidden by default, ≥320×320 px each in production)

Each mini-dome shows the same notes in the same radial position as the anchor, but **isolates one channel** with its optimal visual property. Stratum bands preserved at 0.04 opacity (radial anchor metaphor never disappears). **Mini-domes stay tradition-agnostic** — they encode Confidence, Stage, Acts, and Provenance under their Western-analytic stipulation (§7) regardless of which tradition is active on the anchor.

| Mini-dome | Isolated channel | Mark rendering | Pre-attentive in this view? |
|---|---|---|---|
| Confidence | opacity (0.4 → 1.0) | Uniform 2.8 px discs, opacity varies | Yes (opacity is pre-attentive) |
| Stage | hue (5 categorical) | Full-disk 2.8 px hue (no pip — the mark IS the stage color) | Yes (full-disk hue pops; this is where Stage is pre-attentive) |
| Acts | size (binary) | Top-decile = 6 px filled; rest = 1.5 px dot | Yes (size is a Treisman primitive at >30% delta) |
| Provenance | 5 angular sectors (Self / Read / Heard / Reasoned / Tradition) | Stars positioned in their source sector + radial=stratum | Yes (position pop-out within sector) |

**Linked brushing (level 2 — cross-filter, not hover-only):**
- Hover a star in any view → gold ring highlight propagates across all 5 views.
- Click a category in a mini-dome → all 5 views filter to matching stars; non-matches dim to 0.15 opacity; facet sidebar counts rebalance.
- Click the same category again → release filter.

**Aggregation at scale (≥5,000 visible notes):**
- Mini-domes switch to **hex-bin** rendering (d3-hexbin library). Each hex shows the dominant channel value + a count badge.
- Per-star rendering returns automatically when filters reduce visible to <5,000.
- Threshold is **tunable** in Settings (default 5,000).

### 2.4 The facet sidebar (left edge, 180 px wide, collapsed by default)

The faceted-browse surface — Hearst's Flamenco pattern. Six facets with live counts; clicking any category cross-filters all views. Group labels and static category labels (Foundation, Hypothesis, Self, Established, etc.) localize via `$t`; user-domain values (folder paths, library names, custom stage names) fall through the i18n fallback chain unchanged so user content stays literal.

### 2.5 The tradition chip (title bar)

The cross-civilizational lens. **24 curated traditions** plus any user-defined traditions present in the active Universe; each remaps the anchor dome's geometric grammar according to its `TraditionShape` (§3.5). **Applies to anchor dome only**; mini-domes stay culturally neutral (see §7).

**Default state**: collapsed, shows current active tradition only (e.g., `Aristotelian ●`).
**Engaged state**: click to expand the dropdown panel. The panel is **family-categorized** — 10 family headers (Western classical, Indian Nyāya, Sunni Islamic uṣūl, Arabic / Islamic beyond uṣūl, Modern Western, Jewish (Abrahamic), East Asian Confucian, Chinese pragmatist, Latin American decolonial, African philosophical) and an optional **User-defined** section at the bottom if any user traditions are loaded.
**Active tradition highlight**: blue stroke + dot indicator; chip background tinted via `hsla(var(--accent-h)...)`.
**Hover any chip**: tooltip with the tradition's one-line scope (e.g., *"pramāṇa — Nyāya fourfold valid means of knowing"*).
**ⓘ disclosure**: each chip exposes an info button that opens the tradition's manifest at `docs/traditions/<id>.md` (curated) or a synthesized manifest (user-defined). The 24 manifests ship in 15 locales — the modal reads the active-locale variant when present.

The 24 traditions and their geometries are specified in §4.

---

## §3 — Channel taxonomy

### 3.1 The honest pre-attentive ledger

v6 commits to honest channel-tier labeling. No "N channels at one glance" overclaim.

| Channel | Tier | Encoding | Notes |
|---|---|---|---|
| Stratum | **pre-attentive** | radial position (Aristotelian; remappable) | strongest channel per Mackinlay (1986) |
| Time | **pre-attentive** | angular position (Aristotelian; remappable) | second-strongest channel |
| Library | **pre-attentive** | shape | Treisman primitive; 5 distinct shapes pre-attentive at ≥4 px |
| Confidence | **pre-attentive** | opacity 0.4–1.0 | value/opacity in pre-attentive set |
| Acts | **pre-attentive** | size +40% binary | size pre-attentive at ≥30% delta |
| Cluster density | **emergent gestalt** | spatial pattern of positions | Not an independent channel; emerges from positions |
| Stage (anchor) | **focal-on-foveation** | inner pip hue ≥1.8 px | Hue on small marks requires foveation; honest label |
| Stage (mini-dome) | **pre-attentive** | full-disk hue 2.8 px | Same encoding, larger area = pops |
| Typed-link kind | **pre-attentive when sparse** | line color | Above 800 visible lines auto-fades |
| Provenance | **deferred** | not on anchor star; sidebar filter + dedicated mini-dome | Removed from per-star encoding |

**Net pre-attentive on the anchor dome at first glance: 5 independent features + 1 emergent gestalt** (stratum, time, shape, opacity, size + cluster gestalt). That is the truthful claim — well within Treisman's pre-attentive set (≤7 primitives).

The remaining channels (stage, link-color, provenance) are accessible via foveation, mini-dome, or sidebar.

### 3.2 Channel orthogonality invariant (locked)

**No two channels of Sight may share a Bertin visual variable.** This is the design invariant that fixes the v0.1 channel-confound problem at the architectural level.

| Bertin variable | Claimant | What it carries |
|---|---|---|
| Position (x,y) | Stratum + Time (Aristotelian); tradition geometry (otherwise) | Spatial anchor |
| Size | Acts | +40% binary |
| Value (opacity) | Confidence | Continuous |
| Texture | (reserved for >5 library shapes) | Outline-style fallback |
| Color hue | Stage (anchor pip + mini-dome) + Link line | Two non-co-rendered uses |
| Orientation | (unused) | Reserved |
| Shape | Library | Five pre-attentive shapes |

Two uses of hue (Stage + Link line) co-render on the anchor dome but in spatially distinct regions (pip = star interior; line = between stars), so they don't perceptually collide. CIE Delta-E ≥30 between any two simultaneously-rendered hues is a v6 hard requirement (verified before build).

### 3.3 Library shape vocabulary

| Library count | Shape strategy |
|---|---|
| 1 | circle |
| 2 | circle + square |
| 3 | circle + square + diamond |
| 4 | + triangle |
| 5 | + hexagon |
| 6–10 | five shapes + outline-style: solid → outlined |
| 11–15 | + double-outlined |
| 16–20 | + striped |
| 21–25 | + dotted |
| >25 | degraded fallback: same shape family, library name surfaced in tooltip only |

**Shape-weight normalization** (Bertin-aware): each shape rendered at equal *perceived* area, not equal bounding-box area. Diamond shrunk ~15%, triangle upscaled ~20%, hexagon shrunk ~10%, square baseline, circle baseline.

### 3.4 Color palette (v6 commit, CIE Delta-E ≥30 verified)

**Stage hue (5 categorical, anchor pip + mini-dome full-disk):**
- established = `#4ade80` (green)
- fresh = `#22d3ee` (cyan)
- growing = `#a78bfa` (violet)
- at-risk = `#facc15` (yellow)
- dormant = `#94a3b8` (gray)

**Typed-link line colors (9 kinds):**
- supports = `#4ade80` · contradicts = `#f87171` (dashed) · causes = `#fb923c` · exemplifies = `#60a5fa` · generalizes = `#a78bfa` · derives-from = `#22d3ee` · part-of = `#f472b6` · associative = `#94a3b8` · supersedes = `#fde68a`

**Theme awareness** (added MIG-027): chrome colors (axis labels, sector dividers, ring boundaries, ladder steps, hover ring) read from a `ChromePalette` driven by the active app theme (`appSettings.activeThemeId` + `colorScheme`). A `--sight-highlight` family (foreground / bg-soft / bg-strong / border-soft) provides theme-conditional emphasis: bright amber `#fbbf24` for dark themes, deep amber `#b45309` (WCAG AA on cream) under the light-theme body class. Semantic colors (Stage palette, Link palette) are theme-invariant.

**Hue overlap risk** (Stage green + Supports green; Stage violet + Generalizes violet; Stage cyan + Derives-from cyan): mitigated by spatial separation (lines are between stars, pips are inside stars) and confirmed by CIE Delta-E check at build time. If overlap fails Delta-E, link palette gets a hue rotation.

### 3.5 The nine shape renderers

v4.0 assumed one geometric grammar (sectoral) with Polanyi as a per-star opacity overlay on top of it. v4.1 ships **nine** `TraditionShape` values, each backed by a dedicated renderer in `anchor.ts`. They were added incrementally across MIG-026 Phases γ through θ as the 24-tradition curated set required them; the rendering pipeline accepts a tradition's declared `shape` and dispatches to the matching draw helper. The shapes are not interchangeable — each one is the geometric metaphor a tradition's hero metaphor demands, and the manifests in §4 use them as faithful visual translations of the philosophical claim.

**1. `sectoral` (angular slices).** The default geometry. The dome is divided into N wedges (typically 3, 4, or 5) by radial divider lines; stars within a wedge keep the Aristotelian radial=stratum encoding. Used by Aristotelian (identity remap — no dividers drawn; the five stratum bands ARE the structure), pramāṇa (4 wedges), masādir (4 wedges), Peirce (3), Habermas (3), Longino (4), Mencian sprouts (4 + central xìn ring), Korean Sŏngnihak (4, encoding a 2×2 conceptual grid), and Akan Wiredu (3). Sector dividers are drawn at the angles each module declares; rotation offsets (typically +π/4 for 4-wedge and +π/6 for 3-wedge variants) clear the vertical axis from stratum-label collision.

**2. `rings` (concentric depth tiers).** The dome is divided into N concentric annuli, with the innermost ring labeled at center and the outermost at the rim. Used by Husserl (4 regional ontologies — formal innermost, spirit outermost), Ibn Rushd burhān (4 demonstrative arts — burhān innermost, shiʿr outermost), PaRDeS (4 levels of Jewish exegesis — pəshaṭ outermost, sod innermost), and Maldonado-Torres (3 tiers of coloniality — power outermost, being innermost). The depth metaphor is the *point*: these traditions read knowledge as layered-in-depth, not categorical.

**3. `grid` (ring × sector composition).** A two-dimensional categorical grid drawn by crossing N concentric tiers with M radial sectors, yielding N×M cells. The only curated tradition using this shape is Shāṭibī maqāṣid (3 tiers × 5 essentials = 15 cells), where the geometry is the cleanest available rendering of any Islamic register in the curated set.

**4. `ladder` (N-step spiral).** A spiral rising from rim to center in N labeled steps. Added in Phase ζ.2 specifically to honor hierarchical-N-step traditions that don't fit any sectoral or ring geometry. Used by Maimonidean prophecy (11-step spiral, per Guide of the Perplexed II.45) and Talmudic 13 middot (13-step spiral, per the *Baraita de-Rabbi Ishmael*). The ladder makes the *graduated-attainment* claim visible: each step is qualitatively distinct, the order is non-arbitrary, and the cardinality is high enough that the structure is the geometry.

**5. `relational` (hub-and-spoke network).** A central disc surrounded by N satellite clusters, with spoke lines linking them. Added in Phase θ for traditions whose hero metaphor is *relational rather than spatial* — where being is constituted by complementarity or where the center is something to be displaced. Used by Mignolo pluriversal (central modernity hub + 5 decolonial satellite clusters; the hub is *critiqued*) and Ibuanyidanda (central "missing link" unity + 5 complementary clusters; the hub is *supportive*). The same renderer, different ethical valence — and Constellation acknowledges that relational-not-spatial traditions sit imperfectly inside any spatial visualization.

**6. `cyclic-flow` (ring with directional arrows).** The dome is rendered as a five-segment ring with chevron arrows indicating temporal direction. Used by Dewey's pattern of inquiry (5 segments — indeterminate, problem, hypothesis, reasoning, testing — with clockwise chevrons making the cycle's temporality visible). The chevrons are the *point*: this is not a static typology but a process that re-fires.

**7. `binary-flow` (two-pole with directional flow, three layouts).** The dome divides into two opposed regions connected by flow arrows. Three layout variants in v6.3 — though structurally one `TraditionShape` union value:
   - **horizontal**: two stacked bands (top/bottom), with rotational (left-up + right-down) flow arrows encoding *cyclic* civilizational transition. Used by Ibn Khaldūn ʿumrān (ḥaḍarī top, badawī bottom; the badawī conquers ḥaḍarī, settles, decays, is conquered by a new badawī wave).
   - **vertical**: two side-by-side hemispheres (left/right), with bidirectional flow arrows and an optional central axis label. Used by Wang Yangming (zhī left, xíng right, liángzhī as the central reservoir; the doctrine zhī-xíng héyī is rendered as a binary with flow, not as two disjoint cells).
   - **concentric**: an inner disc + an outer ring, with radial flow arrows pointing inward from ring to disc — *asymmetric* flow, not symmetric both-sides. Used by Dussel transmodernity (inner = totality, outer = exteriority; the analectic movement is *from exteriority back into totality*).

**8. `gradient` (continuous opacity overlay).** No re-positioning, no dividers, no rings — instead a continuous opacity field is drawn over the Aristotelian layout. Polanyi is the only curated tradition using this shape: fog dense at center (tacit, acknowledged but inarticulable, 0.14–0.18 opacity), clear at the rim (explicit, 0.85–0.95 opacity). Polanyi is a *modulation* rather than a *redistribution*: notes do not move, their visibility shifts. The shape exists precisely because Polanyi's claim is about *articulability*, not category.

**9. `horizontal-bands` (N stacked horizontal zones).** The dome divides into N horizontal bands, top to bottom, with no angular encoding within each band (positioning is by deterministic per-note jitter). Used by Mohist sān biǎo (3 bands — 本 běn top, 原 yuán middle, 用 yòng bottom — because Mohist's three standards are *categorical*, not ordinal, and the horizontal axis carries no meaningful encoding).

The nine-shape pipeline is what makes the 24-tradition baseline tractable — and it sets the ceiling for the user-definable plugin layer in §3.6, which exposes the same nine shapes as the surface area against which user authors can build custom traditions.

### 3.6 The user-definable plugin layer

v4.0 said nothing about user-defined traditions; the curated set was assumed sufficient. Phase κ of MIG-026 shipped a two-tier plugin layer so that the 24 curated traditions are a starting set rather than an enclosure. User traditions appear in the chip dropdown alongside the curated set, under a "User-defined" section at the bottom.

**Tier 1 — declarative JSON (Phase κ.1).** For shape-bound custom traditions whose geometry fits one of four declarative shapes — `sectoral`, `rings`, `horizontal-bands`, `gradient` — users drop a `.json` file into `<Universe>/.constellation/traditions/`. On Sight mount, `userDefinedLoader.ts` invokes a Rust IPC that lists the folder, validates each file against `docs/traditions/schema/tradition.v1.schema.json`, and registers valid modules into the side-map. Required fields: `schema_version: 1`, `id` (matching `^user-[a-z0-9][a-z0-9-]{2,40}$`, so user ids cannot collide with curated ids), `name`, `shape`, and the per-shape spec (`sectorDividers` for sectoral, `rings` for rings, `horizontalBands` for horizontal-bands, `gradient` for gradient). Optional `family` defaults to `user-defined`; optional `tooltip` / `scope` / `citation` populate the chip tooltip and synthesized manifest modal. Invalid files are skipped with a console warning naming the file and specific violation; one bad file does not block the others.

**Tier 2 — full-trust JavaScript plugins (Phase κ.2).** For traditions needing arbitrary `remapStarPosition` logic or shapes outside the four declarative ones (`grid`, `ladder`, `relational`, `cyclic-flow`, `binary-flow`), users drop a `.js` file into the same `<Universe>/.constellation/traditions/` folder. Constellation's CSP forbids `unsafe-eval` (LL-019), so runtime TypeScript transpilation is not an option — authors compile their TS to JS with `tsc` and ship the `.js` (the same constraint Obsidian plugins live under). The plugin file's `export default` must satisfy the `PluginModule` contract: `id`, `name`, `shape`, and a deterministic `remapStarPosition(row, defaultPos, layout)` function; optional per-shape callbacks (`sectorDividers`, `ringBoundaries`, `horizontalBandsSpec`, `gradientSpec`). Self-contained files only — plugin `import` statements are not resolvable because Vite's bundler does not see the files at build time; helpers must be inlined.

**Obsidian-trust consent model.** Tier-2 plugins run with full Tauri-IPC privileges. On first detection of a `.js` file, `pluginLoader.ts` displays a consent banner showing the absolute path; clicking "Enable plugin" persists the filename to `appSettings.sight.enabledTraditionPlugins` and the plugin auto-loads on every subsequent Sight mount. Disabling is symmetric (remove from the array, or delete the file). Plugins that crash during `remapStarPosition` fall back to the default Aristotelian position for the affected note; other notes and other traditions are unaffected.

**Loading mechanism.** Tier-1 JSON files are read as text via the Rust IPC + parsed in the frontend, so no special CSP allowance is needed. Tier-2 JS plugins load via native dynamic `import()` of a Tauri `asset://` URL, which required adding `asset:` to the CSP's `script-src` list — a deliberate widening matched by the consent gate so users opt into the broader trust envelope explicitly. Both loaders register into the `USER_REGISTRY` side-map inside `traditions/index.ts`, which `getTraditionById` and `allTraditions` consult after the curated `REGISTRY`. The curated TraditionId union stays closed and type-safe; the user side-map accepts any schema-valid string id, with the renderer-safe cast applied at the registry boundary.

The user-definable layer is the answer to a question v4.0 left implicit: *who gets to add traditions?* v4.1 says: anyone, on their own machine, without a code commit — at two trust tiers, with the consent gate matching the trust ask.

---

## §4 — The 24 curated traditions (and the lone v1-preview survivor)

All 24 curated traditions ship as production modules in Sight v6.3. Each has (a) a tradition module at `src/lib/sight/v6/traditions/<id>.ts` exporting a `TraditionModule` const, (b) a manifest at `docs/traditions/<id>.md` carrying the scholarly briefing the ⓘ disclosure modal surfaces, (c) per-tradition canvas labels in 15-locale i18n at `sight.v6.tradition.canvas.<id>.*`, and (d) chip dropdown strings at `sight.v6.tradition.list.<id>.{name,tooltip,scope}`.

The §4.1.x subsections below give the scholarly contract: one-line hero, cultural framing, geometry, scope, critique, primary citation. They are the Concept Paper's binding statement of what each tradition means as a Sight visualization. The full manifests (with applicability, lineage, modern scholarship, and per-note frontmatter) live at `docs/traditions/<id>.md` and are the source of truth a scholar should consult.

### 4.1 The curated 24 (production-shipped in v6.3)

#### §4.1.1 — Aristotelian (Western classical, default)

**Hero.** The dome shows your universe as a maturity gradient through time, with five concentric stratum bands reading inward to outward and angular position encoding the month of creation. Notes "rise" through the strata as your understanding of them deepens. This is the default Sight grammar — Constellation's choice — made explicit as one frame among many rather than smuggled in as the unnamed baseline.

**Cultural framing.** Western classical, in the tradition of Aristotle's *Posterior Analytics* and the long taxonomic Western philosophy that followed. The maturity-stratum mapping itself is Constellation's: Aristotle did not assign geometry, but his hierarchical model of knowing-by-demonstration grounds the depth-levels assumption.

**Geometry.** Sectoral with identity remap — no dividers, no per-tradition reorganization. The renderer takes the Aristotelian default positions (radial = stratum, angular = time) and renders them as-is; the five stratum bands ARE the structure.

**Scope.** When you want to see your universe's overall maturity profile — where work sits, where dense clusters are, what is at the edge of knowing. Not for categorical questions (*what kind* of evidence supports a claim) — pramāṇa, masādir, Peirce give kind-discrimination Aristotelian flattens away.

**Critique.** The most Western-default of all the traditions. Making it explicit as a choice (rather than smuggling it as the unnamed baseline) is the *point* — but the maturity-gradient assumption encodes a particular epistemology where knowledge progresses, deepens, matures. Pluralist and pluriversal traditions in the set explicitly resist that assumption.

**Citation.** Aristotle, *Posterior Analytics*, trans. Jonathan Barnes, 2nd ed. (Oxford: Clarendon Press, 1994).

#### §4.1.2 — pramāṇa (Indian Nyāya)

**Hero.** The dome divides into **four quadrants of valid knowing**, each housing notes whose epistemic ground is of one kind: direct perception (pratyakṣa), inference (anumāna), analogy from a known case (upamāna), trusted testimony (śabda). Knowledge is sorted not by *how mature* but by *how it came to be known*. Within each quadrant, the radial stratum encoding from Aristotelian is preserved, so depth-of-understanding remains legible inside its warrant-kind.

**Cultural framing.** Classical Indian Nyāya — the school of formal Indian epistemology that analyzed cognition by enumerating the valid means by which it arises. The four-pramāṇa Nyāya canon is the version Constellation ships; other Indian schools count differently (Sāṃkhya three, Mīmāṃsā six).

**Geometry.** Sectoral, 4 quadrants. After §δ.2-fix-1 (2026-05-17) the quadrants sit at **E/S/W/N** rather than the originally documented NE/SE/SW/NW, rotated to clear the vertical axis from stratum-label collision.

**Scope.** When you want to see how your knowledge is *grounded* — what proportion of your work rests on direct observation vs. inferred conclusion vs. comparison vs. authority. Useful for epistemic self-audit. Poor fit when warrant doesn't vary across notes, or for content that doesn't admit clean source-of-knowing classification (creative work, speculation).

**Critique.** Choosing the four-pramāṇa Nyāya variant is itself a scholarly stake — the Mīmāṃsā six-pramāṇa view was excluded (under the religious-lineage rule) because Vedic-authority-based; the Buddhist Pramāṇavāda traditions (Dignāga, Dharmakīrti) likewise. Users from other Indian-philosophical lineages may find the rendering reductive.

**Citation.** *Nyāya-Sūtra* 1.1.3, in Gautama, *The Nyāya Sūtras of Gautama*, trans. Satisa Chandra Vidyābhūṣana, rev. ed. Nandalal Sinha (Delhi: Motilal Banarsidass, 1990).

#### §4.1.3 — masādir (Sunni uṣūl al-fiqh)

**Hero.** The dome divides into **four sources of authoritative proof** in Sunni *uṣūl al-fiqh*: Qur'an, sunnah, ijmāʿ (scholarly consensus), and qiyās (analogical reasoning). Each is a different *kind* of proof — not a different degree of one proof — and so the layout is sectoral, not concentric. Below the dome, four supplementary sources sit as chips: *istiḥsān*, *istiṣḥāb*, *maṣlaḥa mursalah*, *ʿurf*.

**Cultural framing.** Classical Sunni uṣūl al-fiqh — the science of the sources and methods of Islamic legal reasoning. The four-source canon is conventional across the four Sunni madhāhib (Hanafi, Maliki, Shafiʿi, Hanbali), with internal variation; the Constellation rendering follows the al-Ghazālī *Mustaṣfā* line.

**Geometry.** Sectoral, 4 quadrants + 4 extension chips. After §θ-fix-1 (2026-05-18) the quadrants sit at **E/S/W/N** rather than the originally documented NE/SE/SW/NW (the same rotation pramāṇa received), to clear the vertical axis from stratum labels.

**Scope.** For Sunni Islamic legal-scholarly content where the question is about kinds of proof. Useful for fiqh derivation, uṣūl coursework, fatwa analysis, cross-source balance audit. For non-Islamic content the labels make no sense; Shīʿī uṣūl is deliberately not included per the religious-lineage rule.

**Critique.** The placement of ijmāʿ in the *ijtihādī* (reasoning-derived) cluster rather than the *naṣṣ* (textually-transmitted) cluster is contested by Ashʿarī/Māturīdī kalām, which treats ijmāʿ as binding-transmitted. Constellation ships the Mustaṣfā-aligned reading. The four-source canon also flattens doctrinal differences across the four madhāhib.

**Citation.** Abū Ḥāmid al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl*, ed. Ḥamza ibn Zuhayr Ḥāfiẓ (Medina: al-Jāmiʿa al-Islāmiyya, 1413/1993).

#### §4.1.4 — Polanyi (Modern Western, gradient)

**Hero.** The dome stays in its Aristotelian layout — but a **fog** is drawn over it. Stars near the center are dim ("tacit", acknowledged but inarticulable); stars near the rim are clear ("explicit", what you can fully say). The metaphor inverts the usual Aristotelian reading: at the center is not foundation but the **tacit dimension** — the knowledge-in-skill, knowledge-in-judgment, knowledge-in-recognition that we possess but cannot reduce to propositions.

**Cultural framing.** Modern Western pluralist epistemology. Michael Polanyi (1891–1976) introduced the tacit/explicit polarity as a continuous spectrum. The key thesis: *we know more than we can tell* — and what we cannot tell nevertheless does epistemic work.

**Geometry.** Gradient overlay (the only tradition in the curated baseline that overlays rather than re-positions). Notes do not move; their visibility shifts. Fog opacity: 0.14–0.18 at center, 0.85–0.95 at rim.

**Scope.** For knowledge with a tacit-vs-explicit dimension; what you know vs. what you can articulate. Useful for craft, performance, medical-clinical, scientific-experimental, and pedagogical content where the gap between what one can do and what one can say is the interesting variable. Poor fit for content that is entirely explicit propositions, or where you want a categorical (this-vs-that) lens.

**Critique.** The tacit/explicit polarity has been criticized as too neat — Harry Collins's typology argues Polanyi conflates several distinct phenomena. Polanyi's broader emergence metaphysics is not invoked in the Sight rendering — only the polarity is.

**Citation.** Michael Polanyi, *The Tacit Dimension* (Garden City, NY: Doubleday, 1966), ch. 1; *Personal Knowledge: Towards a Post-Critical Philosophy* (Chicago: University of Chicago Press, 1958), Part III.

#### §4.1.5 — Mohist sān biǎo (Chinese pragmatist, horizontal bands)

**Hero.** The dome divides into **three horizontal zones stacked top to bottom**, one per Mohist standard for evaluating a doctrine: 本 běn (root — historical precedent of the sage-kings) at top; 原 yuán (origin — direct observational evidence) in the middle; 用 yòng (use — practical social benefit) at the bottom. A doctrine is worth holding only if it passes all three tests.

**Cultural framing.** Classical Chinese pragmatist epistemology. Mòzǐ 墨子 (~5th c. BCE) founded the Mohist school, which presented itself as a critical alternative to Confucianism. The sān biǎo appear in the "Fēi Mìng" (Anti-Fatalism) chapter as the test the Mohists applied to the inherited fatalist doctrine.

**Geometry.** Horizontal-bands, 3 zones. The horizontal axis within each band carries no encoding — Mohist's three standards are *categorical*, not ordinal; positioning is by deterministic per-note jitter.

**Scope.** For content where the test is *whether a doctrine is worth holding*. Useful for policy, ethics, applied-empirical, and practical-decision content. Poor fit for descriptive content with no doctrinal or evaluative dimension.

**Critique.** Sān biǎo are sometimes critiqued as an early form of pragmatism that conflates evidential warrant with utility — the "benefit to the people" criterion in particular is hard to formalize. Grandfathered into the curated baseline under the religious-lineage rule despite the Mòzǐ's Heaven-theology context, because the methodological core is secular.

**Citation.** *Mòzǐ* 墨子, Book IX, "Fēi Mìng Shàng" 非命上. Critical edition: Sūn Yíràng, ed., *Mòzǐ jiāngǔ* (Beijing: Zhonghua Shuju, 1986). English: Ian Johnston, trans., *The Mozi: A Complete Translation* (New York: Columbia University Press, 2010).

#### §4.1.6 — Peirce (Modern Western, sectoral)

**Hero.** The dome divides into **three phenomenological categories** that Peirce argued underlie all experience and all reasoning: Firstness (quality, feeling, possibility — "what it is to be red" before anything red exists), Secondness (action, reaction, brute fact — the actual collision, the resistance of the world), Thirdness (mediation, law, habit, sign — the pattern that connects).

**Cultural framing.** American pragmatism. Charles Sanders Peirce (1839–1914), founder of pragmaticism and modern formal logic, articulated the three categories across his entire career; they appear early ("On a New List of Categories," 1867) and become refined in his semiotic and phenomenological work. The categories are *universal* — Peirce argued they constitute the irreducible vocabulary of all phenomena.

**Geometry.** Sectoral, 3 wedges at 120° each, rotated +π/6 from cardinal axes (§δ.1-fix-1) so no divider runs through stratum labels at the top.

**Scope.** When the question is *what category of experience* this content describes. Excellent for semiotic work, for distinguishing felt quality from acted fact from explanatory law, for argument-type analysis (abductive Firstness, deductive Secondness, inductive Thirdness in one Peircean reading). Poor fit when content has no phenomenological cut.

**Critique.** The categories are notoriously hard to apply without training; Peirce himself revised his presentations many times. Critics from the analytic side question whether the three-way partition is exhaustive; from the phenomenological side, that the categories are too formal to capture lived experience.

**Citation.** Charles S. Peirce, "On a New List of Categories" (1867), in *Writings of Charles S. Peirce*, vol. 2, ed. Edward C. Moore et al. (Bloomington: Indiana University Press, 1984).

#### §4.1.7 — Habermas (Modern Western, sectoral)

**Hero.** The dome divides into **three knowledge-constitutive interests** — Habermas's claim that knowledge production is always already shaped by one of three irreducible human interests: **technical** (instrumental control of objectified processes; the empirical-analytic sciences), **practical** (intersubjective understanding through interpretation; the historical-hermeneutic sciences), **emancipatory** (self-reflection toward autonomy / *Mündigkeit*; the critical sciences).

**Cultural framing.** German critical theory, second-generation Frankfurt School. Jürgen Habermas (b. 1929) articulated the three interests in *Erkenntnis und Interesse* (1968) as the explicit framework for his early epistemology. The interests were later subsumed under the *Theory of Communicative Action* (1981) — but as a typology of inquiry-stance the 1968 formulation remains pedagogically useful and is what Constellation renders.

**Geometry.** Sectoral, 3 wedges at 120° each, rotated +π/6 (§δ.1-fix-1) to keep dividers off stratum labels.

**Scope.** When the question is *what is this knowledge for?* — what interest underlies its production. Useful for methodology of social science, critical-theory work, distinguishing applied / interpretive / critical research strands. Poor fit if your notes are sorted by topic rather than by the work they do.

**Critique.** Habermas himself moved away from the strict three-interest typology in his mature work; critics (Rorty, Foucault from a different angle) argued the typology essentialized interests that are in practice more fluid. The emancipatory interest has been the most contested.

**Citation.** Jürgen Habermas, *Erkenntnis und Interesse* (Frankfurt am Main: Suhrkamp, 1968); English: *Knowledge and Human Interests*, trans. Jeremy J. Shapiro (Boston: Beacon Press, 1971).

#### §4.1.8 — Dewey (Modern Western, cyclic flow)

**Hero.** The dome is rendered as a **cyclic ring with five segments** — Dewey's pattern of inquiry — with chevron arrows indicating clockwise temporal flow. The five stages, in order: indeterminate situation (the unsettled, problematic, doubtful condition that initiates inquiry); problem formulation; hypothesis (idea); reasoning; testing / judgment. The cycle then re-fires from a new indeterminate situation.

**Cultural framing.** American pragmatism, mature phase. John Dewey (1859–1952) articulated the pattern as the systematic core of his *Logic: The Theory of Inquiry* (1938). The pattern is intended as a universal description of how settled belief is recovered from unsettled doubt — applicable equally in everyday problem-solving, scientific investigation, ethical deliberation, aesthetic judgment.

**Geometry.** Cyclic-flow, 5 segments + chevron arrows. The chevrons make the *temporality* visible: this is not a static typology but a process.

**Scope.** When your universe is itself a record of *inquiry* — research notes, experimental notebooks, debugging logs, investigative journalism, casework. Surfaces where you are stuck (problem formulation? hypothesis-poor?), where work piles up. Poor fit for static, encyclopedic, or contemplative content.

**Critique.** The pattern has been criticized as too tidy — actual inquiry rarely proceeds through five discrete stages in clean sequence. Dewey himself qualified the pattern as an analytical reconstruction, not a descriptive sequence.

**Citation.** John Dewey, *Logic: The Theory of Inquiry* (New York: Henry Holt, 1938), esp. ch. 6 ("The Pattern of Inquiry").

#### §4.1.9 — Husserl (Modern Western, concentric)

**Hero.** The dome divides into **concentric rings of regional ontologies** — Husserl's claim that each *region of being* has its own *a priori* essential structures, distinct from formal ontology (which studies the essence of objectivity in general). The Constellation rendering uses four rings: formal (innermost; essences of objectivity-as-such), material nature (physical-causal regularity), animal nature (psychic-vital structures), spirit / Geist (outermost; cultural, intersubjective, historical formations). Each ring is a domain with its own essential laws — you cannot reduce spirit to animal nature, animal nature to material nature, or any of them to formal ontology.

**Cultural framing.** German phenomenology, founding generation. Edmund Husserl (1859–1938) articulated the formal-vs-regional distinction in *Ideas I* (1913) as part of his project to provide phenomenology as the foundational discipline for the sciences. The four-region rendering approximates the *Ideas II* presentation.

**Geometry.** Rings, 4 concentric zones.

**Scope.** When your content spans several ontological regions and you want to see which one a given note inhabits. Useful for philosophy of mind / biology / social science and any project explicitly working across natural / vital / cultural domains. Poor fit when all content sits in one region.

**Critique.** Husserl's regional ontologies presuppose his broader transcendental phenomenology, which is itself contested — Heidegger's critique in *Being and Time* (1927) rejected the formal-vs-regional split in favor of a fundamental ontology of Dasein.

**Citation.** Edmund Husserl, *Ideen zu einer reinen Phänomenologie und phänomenologischen Philosophie. Erstes Buch* (Halle: Niemeyer, 1913); English: *Ideas Pertaining to a Pure Phenomenology and to a Phenomenological Philosophy: First Book*, trans. F. Kersten (The Hague: Nijhoff, 1982).

#### §4.1.10 — Longino (Modern Western, sectoral)

**Hero.** The dome divides into **four norms for objective knowledge production** — Helen Longino's Critical Contextual Empiricism (CCE), which argues that objectivity is a social-procedural achievement, not an individual cognitive virtue: *venues for criticism* (recognized public forums where claims can be challenged), *uptake of criticism* (actual responsiveness of communities to challenges that survive scrutiny), *public standards* (shared evaluative criteria, however provisional), *tempered equality of intellectual authority* (qualified participation across the community, against authority-by-rank alone).

**Cultural framing.** Feminist social epistemology and contextualist philosophy of science. Helen Longino (b. 1944) developed CCE across *Science as Social Knowledge* (1990) and *The Fate of Knowledge* (2002) as an alternative to both naïve realism and strong-program social constructivism.

**Geometry.** Sectoral, 4 wedges at 90° each, rotated +π/4 from cardinal axes so dividers land at NE/SE/SW/NW — the "Longino offset" — clearing the vertical axis from stratum labels.

**Scope.** When you are auditing the *social process* by which a body of claims comes to count as objective — when the question is not "is this true?" but "by what procedure did this become defensible?". Excellent for science-studies, peer-review analysis. Not a content register — if your notes are about subject-matter rather than vetting procedure, the four-norm structure surfaces little.

**Critique.** Critics from the science-studies side (Steve Fuller) argue the norms are too thin to constrain inquiry; from the rationalist side (Susan Haack), that they over-socialize the epistemic. Longino has defended CCE as a procedural floor without claiming it exhausts the conditions of good knowledge.

**Citation.** Helen E. Longino, *Science as Social Knowledge: Values and Objectivity in Scientific Inquiry* (Princeton: Princeton University Press, 1990); *The Fate of Knowledge* (Princeton: Princeton University Press, 2002).

#### §4.1.11 — Ibn Rushd burhān (Arabic-Islamic-beyond, concentric)

**Hero.** The dome divides into **four concentric rings** of discourse, in descending order of demonstrative force from center to rim: *burhān* (apodictic demonstration; the philosophers' rigor), *jadal* (dialectic; the theologians' contention from probable premises), *khaṭāba* (rhetoric; the preachers' and jurists' persuasion), *shiʿr* (poetic image; the discourse of imaginative association for the general public). This is Ibn Rushd's adaptation of Aristotle's *Organon* mapped to four audiences. Each is the *correct* discourse for its hearer; the visualization is not a hierarchy of better vs. worse but of *fitness to audience*.

**Cultural framing.** 12th-century Andalusian falsafa. Ibn Rushd (Averroes, 1126–1198) of Córdoba, in the *Faṣl al-Maqāl* and his commentaries on the *Organon*, adapted the Greek discourse hierarchy to the Islamic context — arguing that *burhān* is the philosophers' privileged mode but does not delegitimize *jadal*, *khaṭāba*, or *shiʿr* for their proper audiences. The treatise was a direct intervention in the post-Ghazālī polemics about whether philosophy was permissible in Islam.

**Geometry.** Rings, 4 concentric zones.

**Scope.** For philosophical, theological, rhetorical, or pedagogical content where the same idea takes different discourse-forms for different audiences. Useful for rhetoric analysis in religious-philosophical texts and for writing self-audit (am I in *burhān* mode or *khaṭāba* mode?).

**Critique.** Ibn Rushd's hierarchy has been criticized as elitist — placing philosophers' demonstration above the masses' poetic-imaginative understanding can read as condescension. The concentric-ring shape makes the hierarchical reading visually salient; users who reject the hierarchy may prefer to read the rings as discourse *kinds* rather than as a quality ladder.

**Citation.** Ibn Rushd, *Faṣl al-Maqāl fī mā bayn al-sharīʿa wa-l-ḥikma min al-ittiṣāl*. Modern English: Charles E. Butterworth, trans., *Averroës: Decisive Treatise and Epistle Dedicatory* (Provo: Brigham Young University Press, 2008).

#### §4.1.12 — Shāṭibī maqāṣid (Sunni uṣūl, grid)

**Hero.** The dome divides into a **3 × 5 grid** — three concentric tiers crossed by five radial sectors — yielding 15 cells, one per (tier, necessity) combination. The three tiers (innermost to outermost): *ḍarūriyyāt* (essentials; without these the human order collapses), *ḥājiyyāt* (needs; ease and improve life but not strictly required for survival), *taḥsīniyyāt* (embellishments; refinements of conduct and culture). The five sectors (*al-kulliyyāt al-khams*): *dīn* (religion), *nafs* (life), *ʿaql* (mind), *nasl* (lineage), *māl* (property).

**Cultural framing.** 14th-century Maliki Andalusian *uṣūl al-fiqh*. Abū Isḥāq al-Shāṭibī (d. 790/1388) of Granada, in *al-Muwāfaqāt fī uṣūl al-sharīʿa*, gave the *maqāṣid* their most systematic medieval treatment, building on al-Juwaynī and al-Ghazālī. Major contemporary revival through Jasser Auda and others.

**Geometry.** Grid, 3 tiers × 5 sectors = 15 cells. Sectors rotated +π/4 (Longino offset) so dividers land at NE/SE/SW/NW. The cleanest geometric representation of any Islamic register in the curated set.

**Scope.** For legal, ethical, or policy content that maps onto the *maqāṣid* framework — questions about which goods are at stake and at what tier of necessity. Useful for contemporary applied ethics in Islamic finance, environmental policy, bioethics. Non-legal-ethical content doesn't sort meaningfully into the 15 cells.

**Critique.** The neat 3 × 5 = 15 cells can mislead — al-Shāṭibī's actual treatment is more porous than the grid suggests. The contemporary maqāṣid revival has been criticized (from within) for risking the reduction of *uṣūl al-fiqh* to a utilitarian calculus.

**Citation.** Abū Isḥāq al-Shāṭibī, *al-Muwāfaqāt fī uṣūl al-sharīʿa*, ed. ʿAbd Allāh Darrāz, 4 vols. (Cairo: al-Maktaba al-Tijāriyya al-Kubrā, n.d.).

#### §4.1.13 — Ibn Khaldūn ʿumrān (Arabic-Islamic-beyond, binary-flow cyclic)

**Hero.** The dome divides into **two horizontal bands** with cyclic flow arrows connecting them — Ibn Khaldūn's foundational distinction in the *ʿilm al-ʿumrān* (science of civilization): *ʿumrān ḥaḍarī* (top — urban, sedentary life; refined, dependent on division of labor, prone to luxury and the decay of *ʿaṣabiyya*) and *ʿumrān badawī* (bottom — rural, nomadic, tribal life; hardy, cohesive, the source of *ʿaṣabiyya*, the dynastic-energy reservoir that produces the next ruling house). The cyclic flow arrows (left-up + right-down) make the *civilizational temporality* visible: badawī conquers ḥaḍarī, settles, becomes ḥaḍarī, decays, is conquered by a new badawī wave.

**Cultural framing.** 14th-century Maghribi historiography and proto-sociology. Ibn Khaldūn (1332–1406), Tunisian-born Maliki jurist, polymath, and historian, wrote the *Muqaddima* (1377) as the methodological introduction to a projected universal history. The *ʿumrān* framework is the methodological centerpiece — Ibn Khaldūn argued he was founding a new science distinct from history-as-narrative and political philosophy-as-prescription.

**Geometry.** Binary-flow, horizontal layout (cyclic). The only horizontal binary-flow in the curated set.

**Scope.** For content with a civilizational, macro-sociological, or longue-durée historical dimension. Useful for political-theoretical work, philosophy of history, anthropology, comparative sociology. Awkward fit for legal, ritual, or mystical content.

**Critique.** The badawī/ḥaḍarī binary has been criticized as essentializing. The *ʿaṣabiyya* concept, while illuminating, has been used to justify a wide range of incompatible political programs.

**Citation.** Ibn Khaldūn, *Muqaddimat Ibn Khaldūn*, ed. ʿAlī ʿAbd al-Wāḥid Wāfī, 4 vols. (Cairo: Lajnat al-Bayān al-ʿArabī, 1957). English: Franz Rosenthal, trans., *The Muqaddimah: An Introduction to History*, 3 vols. (Princeton: Princeton University Press, 1958).

#### §4.1.14 — PaRDeS (Jewish Abrahamic, concentric)

**Hero.** The dome divides into **four concentric rings**, one per level of interpretation in the classical Jewish exegetical scheme. The acronym PaRDeS spells the Hebrew word for "orchard" — and the four levels are the gates one enters in succession: *Pəshaṭ* (outermost; the plain or literal sense of the text), *Remez* (the hint or allegorical sense), *Drash* (the interpretive or midrashic sense, homiletical and ethical), *Sod* (innermost; the hidden or mystical sense). The geometry encodes *depth of interpretation*: surface at the rim, mystery at the core.

**Cultural framing.** Medieval Jewish exegesis. The four levels are present in Talmud and Midrash; their canonical four-fold systematization is attributed to Moses de León (13th c.) in the orbit of the *Zohar*. The framework became dominant in both rabbinic and Kabbalistic exegesis and remains the standard heuristic in contemporary Jewish hermeneutics.

**Geometry.** Rings, 4 concentric depth zones.

**Scope.** When working with text-interpretive content where the same passage admits multiple layered readings. Useful for Bible / Torah commentary, literary hermeneutics inspired by the rabbinic model. Poor fit for content with no text-interpretive dimension.

**Critique.** PaRDeS has been criticized for risking a "deeper-is-truer" reading in which *sod* dominates *pəshaṭ*. Modern scholars (Scholem, Idel) stress the levels are best read as complementary modes rather than as a ladder. The concentric-ring rendering visually privileges the depth metaphor; users who prefer a flat-categorical reading may find it tendentious.

**Citation.** *Zohar*, multiple loci. English: Daniel C. Matt, trans., *The Zohar: Pritzker Edition*, 12 vols. (Stanford: Stanford University Press, 2004–2017).

#### §4.1.15 — Maimonidean prophecy (Jewish Abrahamic, spiral ladder)

**Hero.** The dome shows **eleven graduated steps** of prophetic knowledge, rendered as a spiral ladder rising from the rim inward. The lower steps are accessible to the philosopher who has perfected reason and imagination but has not received divine emanation; the higher steps involve increasing direct contact with the Active Intellect. The specific identity of each step is intentionally not shown in the chip — the *Guide of the Perplexed* II.45 enumerates them as the textual authority. Above the eleventh step sits a (visually unrepresented) twelfth: the prophecy of Moses, which Maimonides treats as a special kind beyond the hierarchy.

**Cultural framing.** 12th-century Andalusian-Egyptian Jewish philosophy. Moses Maimonides (Rambam, 1138–1204), in the *Moreh Nevukhim* II.32–48, articulated the eleven levels as a synthesis of biblical-prophetic material with Avicennan and Aristotelian philosophy. The hierarchy was controversial in its own time and remains a touchstone in medieval-Jewish-philosophy scholarship.

**Geometry.** Ladder, 11-step spiral. The spiral renderer was added in Phase ζ.2 specifically to honor hierarchical-N-step traditions that don't fit a clean sectoral geometry.

**Scope.** When your content has a genuinely graduated structure (whether literally about prophecy or by methodological analogy — degrees of insight, stages of mastery, levels of contemplative attainment). Eleven is a high cardinality — most user content will cluster at one or two steps.

**Critique.** The eleven-step hierarchy presupposes the medieval Aristotelian philosophy-of-mind that grounded it (Active Intellect, perfection of imagination as a precondition for prophecy). Modern readers without that framework can find the steps hard to motivate. The hierarchy is also philosophical-elite-oriented.

**Citation.** Moses Maimonides, *Moreh Nevukhim* (Guide of the Perplexed), part II, chs. 32–48. English: Shlomo Pines, trans., *The Guide of the Perplexed*, 2 vols. (Chicago: University of Chicago Press, 1963).

#### §4.1.16 — Talmudic 13 middot (Jewish Abrahamic, ladder)

**Hero.** The dome shows **thirteen hermeneutic rules** — Rabbi Ishmael's *sheloshah ʿaśar middot*, the canonical Tannaitic toolkit for deriving law from the Torah text — rendered as a 13-step spiral ladder. The middot are not a *content hierarchy* but a *toolkit*: each rule is an inference-pattern (the *qal vaḥomer* a-fortiori, the *gezerah shavah* verbal analogy, the *binyan av* paradigm-from-one-source, etc.). The 13-step rendering is chosen because Rabbi Ishmael's enumeration became the canonical liturgical and didactic version; Hillel's earlier seven-rule version is the alternative often cited as cleaner.

**Cultural framing.** Tannaitic / early rabbinic Judaism, 2nd c. CE. Rabbi Ishmael ben Elisha codified the thirteen rules as an expansion of Hillel's earlier seven. The list is preserved in the *Sifra* on Leviticus and recited daily in traditional liturgy.

**Geometry.** Ladder, 13-step spiral.

**Scope.** When working with legal-hermeneutic content where the question is *which inference pattern produced this ruling*. Useful for Talmud and halakhic study, philosophy of legal interpretation, comparative analysis with Islamic legal hermeneutics (*qiyās* vs. *qal vaḥomer*). Non-legal-hermeneutic content doesn't sort meaningfully into the 13.

**Critique.** Thirteen is an awkward cardinality for clean visual sorting — most user content will not exercise more than three or four rules. Some scholars argue the rules are *post-hoc* rationalizations of legal decisions rather than generative procedures. The choice of Ishmael's 13 over Hillel's 7 is also a contested editorial decision.

**Citation.** *Baraita de-Rabbi Ishmael*, preserved in the *Sifra* on Leviticus, introduction. Available in the *Sifra*, ed. Isaac Hirsch Weiss (Vienna: Schlossberg, 1862).

#### §4.1.17 — Mencian sprouts (East Asian Confucian, sectoral + central ring)

**Hero.** The dome divides into **four quadrants**, one per sprout (*duān*) of the moral heart-mind that Mencius identified as innate in every human: *cèyǐn* 惻隱 (compassion → *rén* humaneness), *xiūwù* 羞惡 (shame → *yì* rightness), *círàng* 辭讓 (deference → *lǐ* ritual propriety), *shìfēi* 是非 (sense of right and wrong → *zhì* wisdom). A small central ring carries the Han-dynasty addition: *xìn* 信 (trustworthiness), the fifth constant virtue (*wǔcháng* 五常). The "sprout" metaphor is the heart of the framework: these are not finished virtues but seeds that must be cultivated.

**Cultural framing.** Classical Confucianism. Mencius (Mèngzǐ 孟子, c. 372–289 BCE) articulates the four sprouts in *Mengzi* 2A.6 — the famous "child falling into a well" passage where the universal human response of horror is offered as evidence that compassion is innate, not learned. The Han-dynasty addition of *xìn* produced the *wǔcháng* canon that became standard in subsequent Confucian education.

**Geometry.** Sectoral, 4 quadrants at 90° each (Longino offset, +π/4) + an optional central xìn ring at the dome center.

**Scope.** When working with moral-psychological content, ethical-character development, or applied Confucian ethics. Useful for sorting notes by which virtue they explore or exemplify; for self-cultivation journaling. Purely descriptive empirical content does not have a moral-sprout dimension.

**Critique.** The innate-sprout thesis has been a Confucian-internal controversy since Xunzi (3rd c. BCE), who argued the opposite — that human nature is initially bad and virtues are cultural achievements. The Han addition of *xìn* is also a doctrinal move; Constellation includes it as an optional center ring to keep both the four-fold and five-fold readings available.

**Citation.** *Mèngzǐ* 孟子, 2A.6 and 6A.6. English: Bryan W. Van Norden, trans., *Mengzi: With Selections from Traditional Commentaries* (Indianapolis: Hackett, 2008).

#### §4.1.18 — Wang Yangming (East Asian Confucian, binary-flow vertical)

**Hero.** The dome divides into **two hemispheres**, left and right, with bidirectional flow arrows connecting them and a central labeled axis: *zhī* 知 (knowing) on one hemisphere, *xíng* 行 (acting) on the other, *liángzhī* 良知 (innate moral knowing) labeled as the central reservoir feeding both. Wang Yangming's distinctive Ming-dynasty doctrine — *zhī-xíng héyī* (知行合一, the unity of knowledge and action) — is rendered as a *binary with flow* rather than as two separate cells: the two are analytically distinct but in practice one. The bidirectional arrows say: every knowing is already an acting, and every acting is already a knowing.

**Cultural framing.** Ming-dynasty Neo-Confucianism. Wáng Yángmíng (1472–1529) founded the *Xīnxué* (心學, School of the Heart-Mind) as a critical alternative to the Zhu Xi orthodoxy that dominated official Confucianism. Wang's claim challenged the Zhu Xi position that one first investigates the principles of things and then acts on what one knows.

**Geometry.** Binary-flow, vertical layout — distinct from Ibn Khaldūn's horizontal binary-flow and Dussel's concentric binary-flow. The vertical-layout binary-flow renderer was added in Phase η.2 specifically for this tradition.

**Scope.** When your content explicitly connects insight to practice — when the question is "what did this knowing *do*?" or "what knowing is implicit in this act?". Useful for applied ethics, contemplative-practice journaling. Poor fit for purely theoretical or purely practical content where one dimension is taken for granted.

**Critique.** The unity thesis has been criticized as philosophically obscure — critics ask how one accounts for genuine *akrasia* (acting against what one knows is right). Wang's defenders argue the doctrine *redescribes* akrasia: what looks like knowing-without-acting is actually incomplete knowing.

**Citation.** Wáng Yángmíng, *Chuánxí lù* (傳習錄, Instructions for Practical Living). English: Wing-tsit Chan, trans., *Instructions for Practical Living and Other Neo-Confucian Writings by Wang Yang-ming* (New York: Columbia University Press, 1963).

#### §4.1.19 — Korean Sŏngnihak (East Asian Confucian, 2×2 sectoral)

**Hero.** The dome divides into **four wedges** encoding a 2 × 2 conceptual grid from the Korean *Four-Seven Debate* (사칠논변, *sa-chil nonbyŏn*). Vertical axis: *lǐ* 理 / *qì* 氣 (*i / gi*) — *principle* vs. *material force*, the classical Neo-Confucian metaphysical pair. Horizontal axis: **four moral sprouts** (四端 *sa-dan*) vs. **seven feelings** (七情 *chil-chŏng*), the Mencian moral sprouts vs. the broader *Liji* catalogue of human emotional response. The four resulting cells let you sort by both axes at once.

**Cultural framing.** Chosŏn-dynasty Korean Neo-Confucianism, 16th c. The *Four-Seven Debate* arose from an exchange of letters between Yi Hwang (T'oegye, 1501–1570) and Ki Daesŭng (Kobong, 1527–1572), later extended by Yi I (Yulgok, 1536–1584). It became the longest-running and most-cited philosophical controversy in Korean intellectual history.

**Geometry.** Sectoral, 4 wedges at 90° each (Longino offset, +π/4), encoding a 2×2 grid concept.

**Scope.** When working with Korean Neo-Confucian philosophy or with moral-psychological content that benefits from the two-axis sort. Useful for distinguishing principle-discourse from material-force discourse. Non-Confucian content can be force-fit but loses meaning.

**Critique.** The Four-Seven Debate is famously dense and was contested in its own time as much for what was at stake politically as philosophically. The 2 × 2 grid is a clean visualization but flattens the way the original debate moved fluidly between metaphysics, ethics, and political theory.

**Citation.** The T'oegye / Kobong / Yulgok correspondence. English: Michael C. Kalton et al., trans., *The Four-Seven Debate: An Annotated Translation of the Most Famous Controversy in Korean Neo-Confucian Thought* (Albany: SUNY Press, 1994).

#### §4.1.20 — Mignolo pluriversal (Latin decolonial, relational)

**Hero.** The dome shows a **central modernity disc surrounded by multiple satellite clusters**, with spoke lines linking them — Mignolo's *pluriversality* / *pensamiento fronterizo* (border thinking) rendered as a hub-and-spoke network. The center is the modern/colonial European center; the satellites are the *border epistemes*, the standpoints from which the center can be seen *as* a particular position rather than as the unmarked universal. Constellation's curated rendering uses Mignolo's own decolonial vocabulary for the satellite cluster labels (*colonial difference*, *epistemic disobedience*, *border gnosis*, *pluri-versal*, *de-westernizing*) rather than invoking specific indigenous traditions.

**Cultural framing.** Latin American decolonial theory. Walter D. Mignolo (b. 1941, Argentina; later Duke), influenced by Aníbal Quijano's *coloniality of power*, developed *border thinking* and *pluriversality* across *Local Histories / Global Designs* (2000) and *The Darker Side of Western Modernity* (2011).

**Geometry.** Relational, hub-and-spoke. The chip is *about* irreducible plurality — and putting Mignolo "inside" a register is *performative tension*: his work explicitly critiques the gesture by which the European center catalogues subaltern knowledges.

**Scope.** When you want a *meta*-register for your work — a frame that reminds you any one of the curated traditions is itself situated. Useful as a discipline, inappropriate as a daily-driver lens. The chip is best read as a *prompt to suspicion*.

**Critique.** Constellation's hub-and-spoke geometry can read as enacting the very gesture Mignolo critiques. From within decolonial theory, Catherine Walsh and others have argued the pluriversal framework is insufficiently grounded in specific subaltern struggles.

**Citation.** Walter D. Mignolo, *Local Histories / Global Designs: Coloniality, Subaltern Knowledges, and Border Thinking* (Princeton: Princeton University Press, 2000; 2nd ed. 2012).

#### §4.1.21 — Dussel transmodernity (Latin decolonial, binary-flow concentric)

**Hero.** The dome divides into **an inner disc + an outer ring**, with radial flow arrows pointing inward from the ring to the disc — Dussel's *analectic method* rendered as a concentric binary. **Inner disc**: *totality* — Eurocentric modernity's closed system; the totalizing horizon that takes itself as universal. **Outer ring**: *exteriority* — the standpoint of the *excluded*: the colonized, the poor, the marginalized — those whose existence the totality cannot account for on its own terms. The *analectic* movement is *from exteriority back into totality*, to critique it from the standpoint of the excluded. The inward arrows make this directionality visible; the geometry refuses the symmetric both-sides framing.

**Cultural framing.** Argentine-Mexican Philosophy of Liberation. Enrique Dussel (1934–2023) developed the analectic method as an alternative to Levinasian phenomenology of the Other — insisting the *Other* is not abstract but concrete: the colonized, the poor, the women, the oppressed. *Filosofía de la liberación* (1977) is the systematic statement.

**Geometry.** Binary-flow, concentric layout (the only tradition in the curated set using this layout). The asymmetric flow is the *point*: this is not a both-sides register.

**Scope.** When your content engages totality / exteriority as a critical lens — when the question is "what does this totality exclude, and what does the excluded reveal about the totality?". Useful for liberation theology, political-ethical, and post-colonial work. Explicitly *non-neutral*; users seeking a descriptive rather than critical-normative sort will find it tendentious by design.

**Critique.** Dussel's totality/exteriority binary has been criticized for risking the same totalizing gesture it critiques — by naming "the exteriority" as a definite something, the framework can flatten the plural and conflicting standpoints of actual excluded groups.

**Citation.** Enrique Dussel, *Filosofía de la liberación* (Mexico City: Edicol, 1977); English: *Philosophy of Liberation*, trans. Aquilina Martinez and Christine Morkovsky (Maryknoll, NY: Orbis, 1985).

#### §4.1.22 — Maldonado-Torres (Latin decolonial, concentric tiers)

**Hero.** The dome divides into **three concentric rings**, each a tier of the *coloniality* concept as Nelson Maldonado-Torres extended it from Quijano and Mignolo. **Coloniality of power** (outermost): the structural matrix of global racial-capitalist organization (Quijano's foundational contribution). **Coloniality of knowledge** (middle): the eurocentric hegemony of categories, methodologies, and disciplinary forms (Mignolo's extension). **Coloniality of being** (innermost): the ontological dimension — the way coloniality reaches into selfhood and the experience of personhood (Maldonado-Torres's own contribution, in dialogue with Heideggerian fundamental ontology). The Russian-doll geometry encodes a nesting argument: power is the outer condition; knowledge is the medium through which power reproduces itself; being is the ontological depth where the wound lives.

**Cultural framing.** Caribbean-American decolonial philosophy. Nelson Maldonado-Torres (Rutgers University), building explicitly on Aníbal Quijano and Walter Mignolo, articulated the *coloniality of being* in his 2007 essay of that name as a third tier complementing the two earlier formulations.

**Geometry.** Rings, 3 concentric tiers.

**Scope.** When working with content that specifically engages the *coloniality* literature, or with critical work where the question is *at what depth* coloniality is operating — structural, epistemological, or ontological. Useful for critical race theory, post-colonial philosophy, decolonial-feminist work.

**Critique.** The three-tier ring rendering, like Mignolo's hub-and-spoke, can itself be critiqued for performing the totalizing gesture decolonial theory resists. From within decolonial scholarship, the proliferation of "coloniality of X" formulations suggests the three-tier framing may already be insufficient.

**Citation.** Nelson Maldonado-Torres, "On the Coloniality of Being: Contributions to the Development of a Concept," *Cultural Studies* 21, nos. 2–3 (2007): 240–70.

#### §4.1.23 — Akan Wiredu (African philosophical, sectoral)

**Hero.** The dome divides into **three sectors** drawn from Kwasi Wiredu's analytic-philosophical work on the Akan conceptual scheme: *nokware* (truthfulness — in Wiredu's reading, the Akan term conventionally translated as "truth" is fundamentally a property of *utterances and persons*, not of propositions; this is Wiredu's most-cited claim — the Western T/F binary does not translate cleanly into Akan), *ahonyam* (opinion / conjecture — the standing of a claim short of warranted assertion), *adwene* (thought / cognitive faculty — the mind itself, the capacity that produces both truthful assertion and mere opinion). The framework is *deflationary*: not an alternative metaphysics of truth so much as a critique of the assumption that Western metaphysics of truth is universal.

**Cultural framing.** Contemporary academic African philosophy. Kwasi Wiredu (1931–2022, Ghana), in *Philosophy and an African Culture* (1980) and "The Concept of Truth in the Akan Language" (1985), argued for *conceptual decolonization* — working through the Akan conceptual scheme to identify what is Akan and what is colonial residue.

**Geometry.** Sectoral, 3 wedges at 120° each, rotated +π/6 (the Peirce / Habermas 3-sector pattern) to clear stratum labels.

**Scope.** When your content engages African philosophy, comparative epistemology, or conceptual-decolonization work. Useful as a prompt to ask whether the Western propositional-truth lens is doing distorting work on your material. Wiredu himself would caution against applying the framework as a sorting mechanism for content unrelated to its motivating questions.

**Critique.** Wiredu's deflationary reading has been criticized from two sides: some African philosophers (notably Paulin Hountondji) worry it risks *ethnophilosophy* — packaging a particular African culture's linguistic features as "African philosophy" simpliciter. From the analytic side, the truth-as-truthfulness claim is contested even within Akan philosophy (Gyekye disagrees on the textual reading).

**Citation.** Kwasi Wiredu, "The Concept of Truth in the Akan Language," in *Philosophy in Africa: Trends and Perspectives*, ed. Peter O. Bodunrin (Ile-Ife: University of Ife Press, 1985); *Cultural Universals and Particulars: An African Perspective* (Bloomington: Indiana University Press, 1996).

#### §4.1.24 — Ibuanyidanda (African philosophical, relational)

**Hero.** The dome shows a **central "missing link" hub surrounded by complementary clusters**, with spoke lines connecting them — Innocent Asouzu's *Ibuanyidanda* (complementary reflection) rendered as a hub-and-spoke network. The Igbo word *Ibuanyidanda* roughly translates as "no single load is heavy for a brigade of cooperating ants" — the philosophical thesis is that *every entity is a missing link of reality*, partial without the others, and that being itself is complementary. The five outer clusters carry Asouzu's own complementary-existence vocabulary; the central hub is the *missing-link unity* that the clusters together constitute. Same hub-and-spoke renderer as Mignolo, but with opposite ethical valence: Mignolo's geometry encodes a *critical center* (modernity to be displaced); Ibuanyidanda's encodes a *supportive center* (the unity that complementarity discloses).

**Cultural framing.** Contemporary Igbo (Nigerian) philosophy. Innocent I. Asouzu (formerly University of Calabar) articulated *Ibuanyidanda* as a "complementary ontology" in a sequence of works culminating in *Ibuanyidanda: New Complementary Ontology* (2007). Ikechukwu Anthony Kanu's *Igwebuike* philosophy is a related contemporary Igbo framework with strong overlap.

**Geometry.** Relational, hub-and-spoke (shared renderer with Mignolo).

**Scope.** When your content engages complementarity, relational ontology, or African systematic philosophy. Useful as a deliberate alternative to substance-ontology traditions: in Ibuanyidanda, no entity is what it is independently of the others. Content built on substance-ontology assumptions can be force-fit but loses its claim to independent reality.

**Critique.** Ibuanyidanda has been critiqued from within African philosophy for risking abstraction — the move from concrete Igbo lifeworld to systematic ontology of complementarity may import a Western philosophical idiom (systematic-ontology-building) that reshapes the Igbo material. Hountondji-style critics would raise the *ethnophilosophy* concern. The hub-and-spoke rendering is the most geometrically faithful Constellation can offer; relational-not-spatial traditions sit imperfectly inside *any* spatial visualization.

**Citation.** Innocent I. Asouzu, *Ibuanyidanda: New Complementary Ontology Beyond World-Immanentism, Ethnocentric Reduction and Impositions* (Münster: LIT Verlag, 2007).

### 4.2 v1-preview survivor (the lone holdover)

v4.0 listed three "v1 preview" traditions in §4.2 — Dignāga, Suhrawardi Ishrāqī, Mohist sān biǎo — to ship with explicit "preview" labeling while their deeper internal structure was deferred. The MIG-026 build cascade overtook that staging: the religious-lineage rule (orientation v2.09) excluded Dignāga (the Buddhist Pramāṇavāda traditions fail the Abrahamic-only restriction for religious-source traditions) and Suhrawardi Ishrāqī (overwhelmingly absorbed into Twelver Shīʿī ḥikma, failing the Sunni-only restriction; also fundamentally religious-mystical theology rather than philosophical-epistemological scholarship). Both were removed from the chip dropdown, the TraditionId union, and the manifests directory entirely; settings migrations rewrite any persisted `activeTradition: 'dignaga' | 'ishraqi'` back to `'aristotelian'`.

#### §4.2.1 — Mohist sān biǎo (originally v1-preview, since matured to full ship)

The lone survivor of the v4.0 §4.2 trio, Mohist sān biǎo — originally slated as a v1-preview with "polish in v4.1" labeling — was upgraded to a full production module during MIG-026 Phase γ alongside Polanyi, with the **horizontal-bands** renderer purpose-built to honor the three-zone categorical layout that no other shape in the pipeline could capture. The current scholarly contract, geometry, scope, critique, and citation are in §4.1.5 above; this §4.2 entry exists to record the v4.0 → v4.1 staging change rather than to specify the tradition. Mohist sān biǎo is no longer "v1 preview" — it ships at the same production polish as every other tradition in §4.1.

---

## §5 — Gesture grammar

Sight has no persistent toggle controls. All interaction is through gestures.

| Gesture | Effect | Default state vs Engaged state |
|---|---|---|
| Click sidebar tab "Filters ▶" | Expand facet sidebar | Triggers Engaged |
| Click "Aristotelian ●" label in title | Expand tradition dropdown (family-categorized panel) | Triggers Engaged |
| Cmd-D or click "Show diagnostics" button | Reveal mini-domes (slide in from right) | Triggers Engaged |
| Click any facet category | Cross-filter all views; facet counts rebalance | — |
| Click a mini-dome category | Cross-filter all views | — |
| Click a library shape in sidebar | Other libraries fade to 0.3 opacity | — |
| Right-click a stratum band | Other strata dim; selected stays bright | — |
| Hover a stratum band | Diagnostic popover (count, confidence avg, stage breakdown, acts %, link density) | — |
| Hover a star | Side popover with full lens breakdown | — |
| Click a star | Open note in editor | — |
| Click a tradition row in the dropdown | Re-frame anchor dome under the chosen tradition's geometry | — |
| Click ⓘ on a tradition row | Open the tradition's manifest modal | — |
| Hover a tradition row | Tooltip with one-line scope | — |
| Esc | Reset filters, close popovers, collapse expanded chrome back to default | Returns to default |
| Cmd-F | Search overlay highlights matching stars, dims non-matches | — |
| Cmd-Shift-D | Toggle Pro mode permanently in Settings (persistent default) | — |

**First-boot orientation tour**: on first ever open of Sight, a skippable 4-step overlay appears (auto-skipped on subsequent opens; always available via Help → "Sight tour").

**RTL handling**: in right-to-left locales, the sidebar tab opens from the right edge; chevron icons flip; facet counts use `padding-inline-end` rather than physical-direction padding so the count stays separated from its label.

---

## §6 — Default-simple, Pro-opt-in (first-touch behavior)

This is the most important interaction model commitment in v6. It is what makes the Suwaidi criterion satisfiable.

### 6.1 Default state (first open, every time)

- Anchor dome visible (full canvas devoted to it minus title strip + collapsed sidebar tab + collapsed tradition label).
- Sidebar collapsed to a tab on the left edge ("Filters ▶" label, 20 px wide).
- Tradition chip collapsed to a single label in the title bar (current tradition name + dot).
- Mini-domes hidden. A subtle "Show diagnostics" link in the top-right corner.
- Status strip at bottom shows: universe note count + healthy/at-risk/dormant percentages.

Total visible interactive surfaces in default state: **4** (anchor dome + sidebar tab + tradition label + diagnostics link).

### 6.2 The Suwaidi-fidelity guarantee

In default state, the anchor dome occupies **≥80% of the visible canvas (excluding title strip and status strip)**. This is the architectural guarantee that protects §1.2.

### 6.3 Engagement gestures

Any single click expands one zone:
- Click sidebar tab → sidebar slides out to 180 px width, anchor dome compresses to remaining space.
- Click tradition label → tradition dropdown panel expands (family-categorized).
- Click "Show diagnostics" or press Cmd-D → mini-domes slide in from right, anchor dome compresses to ~60% width.

Each expansion is **independent**. The user can engage any subset.

### 6.4 Persistent Pro mode

If the user prefers always-expanded chrome, **Cmd-Shift-D toggles Pro mode** as a persistent setting. Pro mode default state = all chrome expanded on every open. This is opt-in only — never the system default.

### 6.5 The fidelity vs. engineering trade

v6 ships every engineered surface (mini-domes, tradition dropdown, facet sidebar) — none of this work is wasted. The only thing that changes is **default chrome visibility**. The Suwaidi-grade story-at-glance reads on first open; the user discovers deeper surfaces as they engage.

---

## §7 — Mini-dome channel ontology (Western-analytic by stipulation)

The four mini-domes encode Confidence, Stage, Acts, and Provenance. These channel names are derived from the analytic-Western tradition (Bayesian confidence; lifecycle staging; activity quantiles; sourcing categories).

**v6 explicitly stipulates**: the mini-dome channel names are **by-stipulation labels for the underlying note metadata**, not claims about the universal structure of knowledge. The metadata fields in the SQLite store (`confidence`, `lifecycle_stage`, `act_density`, `provenance_source`) are what they are regardless of tradition.

**The tradition chip remaps the anchor dome's spatial semantics only**; mini-dome channel labels stay constant across traditions in v6.3. This is the architectural commitment that prevents rhetorical pluralism — the cultural framing is honest about what it does and doesn't do, and is verified by the Phase μ.1 channel-isolation invariant (mini-dome contents are identical pre- and post- tradition switch).

**v4.1+ enhancement** (deferred): tradition-aware mini-dome relabeling. When the masādir tradition is active, the Confidence mini-dome could relabel its axis as qaṭʿī/ẓannī; the Provenance mini-dome relabels sectors as the four uṣūl sources. This is purely relabeling — the underlying metadata is unchanged.

---

## §8 — Visual contract (what v6 locks in)

### 8.1 Channel orthogonality invariant

No two channels share a Bertin visual variable. Specified in §3.2. Verified at code-review time on every Sight PR.

### 8.2 Default-state Suwaidi-fidelity guarantee

In default state, the anchor dome occupies ≥80% of visible canvas (excluding title and status strips). Verified by an automated layout test in the test suite (test runner pending — see PJ-054).

### 8.3 Performance budget

- **Cross-filter response**: ≤16 ms on a 7,636-note universe with 5 coordinated views.
- **Default-state render**: ≤100 ms from store-ready to first paint on a 10,000-note universe.
- **Pro-mode render**: ≤180 ms with all 5 views.
- **Hex-bin aggregation kicks in**: above 5,000 visible notes per view.

### 8.4 CIE Delta-E ≥30 between co-rendered hues

Stage palette (5 hues) and Link palette (9 hues) audited at build time. Co-rendering checks: stage-pip vs link-line within ≤5 px region.

### 8.5 Pip foveation threshold

Anchor pip diameter ≥1.8 px at default zoom. At zoom levels where pip would render <1.5 px, pip is suppressed entirely (Stage falls back to mini-dome only).

### 8.6 Tradition manifest contract

Each tradition defined in `docs/traditions/<id>.md` with the schema documented in `docs/traditions/README.md`. The 24 curated manifests ship in 15 locales under `docs/traditions/<lang>/<id>.md`. User-defined traditions follow `docs/traditions/schema/tradition.v1.schema.json`.

### 8.7 Channel-isolation invariant (added v6.3)

For every tradition `T` in the registry, the mini-dome contents must be identical to those rendered under Aristotelian for the same Universe. This is the formal commitment that protects §7 — tradition switching cannot leak into mini-domes. Verification via the Phase μ.1 channel-isolation test (test runner pending, PJ-054).

---

## §9 — Implementation contract (Sight v6 build)

### 9.1 Module organization (post-MIG-026)

Directory: `src/lib/sight/v6/`.

| File | Purpose |
|---|---|
| `SightV6.svelte` | Main component, layout, default-vs-Pro state machine, theme + locale subscriptions |
| `anchor.ts` | Anchor dome Canvas-2D renderer + 9 shape draw helpers |
| `miniDome.ts` | Single mini-dome renderer (instantiated 4×) |
| `MiniDome.svelte` | Mini-dome component shell + theme/locale repaint effects |
| `dome.ts` | Shared chrome palette + semantic colors + `STRATUM_LABEL_KEYS` |
| `facets.ts` | Facet definitions with i18n-key labels |
| `facetSidebar.svelte` | Facet sidebar with cross-filter logic |
| `traditionChip.svelte` | Tradition chip + family-categorized dropdown + ⓘ disclosure |
| `traditions/<id>.ts` | Per-tradition geometry remap (24 files) |
| `traditions/index.ts` | Tradition registry + family groupings + user side-map |
| `traditions/userDefinedLoader.ts` | Tier-1 JSON loader + schema validation |
| `traditions/pluginLoader.ts` | Tier-2 JS plugin loader + consent banner + asset:// import |
| `traditions/_manifests.generated.ts` | Prebuild-regenerated bundle of all per-tradition `docs/traditions/**/<id>.md` manifests across all 15 locales. **Never edit this file directly** — `scripts/build-tradition-manifests.mjs` regenerates it on every `npm install` / `npm run build`. The .md files are the source of truth; the .ts is the bundled artifact the chip's ⓘ disclosure modal reads at runtime. |
| `gestures.ts` | Gesture dispatch (right-click, click-filter, hover, Esc, Cmd-F, Cmd-D) |
| `tour.svelte` | First-boot orientation overlay |
| `types.ts` | TypeScript contracts (`TraditionId`, `TraditionShape`, `TraditionModule`, etc.) |

Backend: `src-tauri/src/sight_v6.rs` with cache schema + IPC for tradition folder listing.

### 9.2 Shipped phases (MIG-025 + MIG-026 cascade)

The Concept Paper v4.0 staged the v6 build as Phase 1 → 4 over ~18 weeks. The actual cascade landed in two MIGs:

- **MIG-025 §A / §B / §C** (2026-05-15 → 2026-05-17) — Sight v6.0 (anchor dome + facets + default-simple) → v6.1 (mini-domes + Pro mode) → v6.2 (tradition chip + initial trio + Polanyi placeholder). Three production traditions shipped (Aristotelian, pramāṇa, masādir).
- **MIG-026 Phases 0 / α / β / γ → θ / ι / κ / λ / μ** (2026-05-17 → 2026-05-18) — Sight v6.3. K1 rename (`register` → `tradition`) under Phase 0; multi-shape architecture under α; chip dropdown UI under β; 21 new tradition modules + 9 shape renderers under γ → θ; 24 manifests + ⓘ disclosure under ι; user-definable JSON + JS plugin layer under κ; full 15-locale localization under λ; ship gate + audit under μ. Final tally: 28 phases, ~36 hours focused work, ~50 commits.

The Concept Paper version (v4.0 → v4.1) and the implementation version (v6.0 → v6.3) diverged during this cascade — v4.0 specified through v6.3 in principle but the curated tradition list and shape pipeline outgrew v4.0's specifications. v4.1 reconciles them.

### 9.3 Migration from Sight v5 → Sight v6 (completed)

v5 was made unreachable at MIG-017 (flag-off only) and fully **retired in MIG-028 (2026-05-18)** — the `src/lib/sight/v5/` module set, `src-tauri/src/sight_v5.rs`, the 4 `sight_v5_*` IPCs, and the `SIGHT_V5_ENABLED` flag are all removed from the build. The `sight_v5_layout` table + invalidation trigger are dropped from existing databases on the first MIG-028-build boot via an idempotent `DROP TABLE IF EXISTS` / `DROP TRIGGER IF EXISTS` in `init_db`. The MIG-024 Plan/Architect docs at `lab/reports/MIG-024-SIGHT-V5-*.md` stay on disk as historical record. Settings migration for excluded traditions: `activeTradition: 'dignaga' | 'ishraqi'` → `'aristotelian'`.

### 9.4 Tech stack

- **Frontend**: Svelte 5 + Canvas 2D. SVG only for static legend icons. No WebGL in v6.
- **Geometry / layout math**: TypeScript pure functions in `anchor.ts` draw helpers + per-tradition modules.
- **Aggregation**: `d3-hexbin`.
- **Backend**: Rust (`sight_v6.rs`); SQLite cache; Tauri events for live updates; folder-listing IPC for user-defined traditions.
- **i18n**: full key-based labelize through `$t`; 15-locale `sight.v6.*` subtree.

---

## §10 — v4.2 polish targets (post-v4.1)

Deferred items carrying forward + new from MIG-026:

1. **Per-tradition frontmatter integration** — the Rust-side `LayoutCacheRow` extension so per-note frontmatter fields (`pramana_kind`, `masadir_source`, `peirce_category`, …, `mencian_sprout`, etc.) override the default placement. Until shipped, defaults populate visually but do not reflect user intent.
2. **Tradition-aware mini-dome relabeling** (§7 enhancement).
3. **Color-accessibility variant** (high-contrast / colorblind-safe palette).
4. **Animated transitions** (tradition switch refinements, mini-dome reveal eased animation).
5. **Universe selector chip** for cUniverse federation view.
6. **Library color recognition aid** (low-saturation tint, opt-in setting).
7. **Sight v6 vitest test runner** (PJ-054) — unblocks the Phase μ.1 channel-isolation and Phase μ.2 perf invariant tests.
8. **User-plugin schema warning** (PJ-055) for dotted-path label collision.
9. **Translation native-quality re-audit** (PJ-053) for the ~95 polish items found in λ-fix-6.
10. **MIG-026 drift cleanup** (PJ-056) — `dome.ts` stale comment + 24 dead `name:` literals + 10 dead `FAMILIES[*].label` literals + 2 stale doc comments.

These are v4.x work, not v6.x work.

---

## §11 — Open invariants (the contract v6.3 guarantees)

1. **Channel orthogonality**: no two channels share a Bertin variable.
2. **Default Suwaidi-fidelity**: anchor dome ≥80% of visible canvas in default state.
3. **Cross-filter performance**: ≤16 ms on 7,636 notes × 5 views.
4. **CIE Delta-E ≥30**: between any two co-rendered hues at build time.
5. **Pip foveation threshold**: anchor pip ≥1.8 px at default zoom, suppressed below 1.5 px.
6. **Tradition isolation**: tradition chip remaps anchor dome only; mini-domes stay culturally neutral and tradition-agnostic.
7. **Tradition manifest**: each tradition's geometry is documented + citation-tracked in version control under `docs/traditions/<id>.md`.
8. **Folder visibility**: Folder is a first-class facet in the sidebar.
9. **Gesture chrome**: no persistent toggle bars. All interaction via gestures + sidebar/chip/mini-dome clicks.
10. **First-boot tour**: 4 steps, skippable, always re-available in Help.
11. **Religious-lineage rule**: no non-Abrahamic religious-source tradition; for Islamic, Sunni only (orientation v2.09). Curated baseline conforms; the rule is enforced at curation time, not at runtime.
12. **i18n labelize**: every on-canvas label flows through `$t(key)` so the dome's vocabulary follows the active interface language without a per-renderer special case. The `labelize` option on `renderAnchorDome` / `renderMiniDome` defaults to identity for tests but is always `$t` in production paint; the `_labelize` module-level state mirrors the chrome-palette pattern so every `fillText` callsite translates uniformly. Stratum labels resolve through `STRATUM_LABEL_KEYS` at `sight.v6.stratum.<band>`; mini-dome titles + provenance sectors through `sight.v6.miniDome.{title,provenance}.<key>`; per-tradition canvas labels through `sight.v6.tradition.canvas.<id>.*`.
13. **Plugin label passthrough**: user-defined plugin labels pass through `$t` unchanged via the i18n fallback chain (active-locale → en → raw-key returned literal). Plugin authors should avoid labels shaped like dotted i18n key paths (e.g. `sight.v6.foo.bar`) because the fallback chain would attempt to resolve them against the global key namespace before falling through to the literal; the schema documentation should carry an explicit warning (tracked as PJ-055).

A future change that violates any of these is, by definition, no longer Sight v6 — it is v7 or a regression. The invariants are the contract.

---

## §12 — Supersession

- `docs/Constellation-Sight-Concept-Paper-v4.0.md` is **superseded** by this document. v4.0 remains on disk as historical record.
- `docs/Constellation-Sight-Concept-Paper-v3.1.md` and all earlier v3.x / v2.x / v1.x documents remain on disk as historical record (already superseded by v4.0).
- The shipped Sight v6.3 (on `main` since the Phase μ ship gate close, milestone tag `milestone/sight-v6.3-traditions-ship`) is the implementation this Concept Paper specifies.
- All v0.x design concept docs remain archived as design conversation history.

---

## §13 — Verification clauses

At each Sight version ship, the following must hold:

### Sight v6.0 (MIG-025 §A — shipped 2026-05-15)
- [x] Anchor dome renders with all 6 pre-attentive channels per §3.1.
- [x] Default-simple layout satisfies §6.2 (≥80% anchor).
- [x] Facet sidebar cross-filters across stratum, library, confidence, stage, provenance, folder.
- [x] First-boot tour fires on first open, skippable.
- [x] Aristotelian default renders cleanly.

### Sight v6.1 (MIG-025 §B — shipped 2026-05-16)
- [x] Four mini-domes render with their isolated channel encoding.
- [x] Stratum bands at 0.04 opacity visible in each mini.
- [x] Linked brushing (gold ring) propagates across all 5 views.
- [x] Click in mini-dome filters all 5 views; counts rebalance.
- [x] Hex-bin aggregation kicks in above 5,000 visible.
- [x] Cmd-D toggles diagnostics visibility.
- [x] Pro mode persists across sessions.

### Sight v6.2 (MIG-025 §C — shipped 2026-05-17)
- [x] Tradition chip + initial 3 traditions (Aristotelian, pramāṇa, masādir) production-polish.
- [x] Polanyi gradient overlay implemented (initially as a chip placeholder; full module shipped under MIG-026 Phase γ).
- [x] Hover tooltip on each tradition chip shows scope line.
- [x] Religious-lineage rule applied: Dignāga + Ishrāqī removed entirely.

### Sight v6.3 (MIG-026 — shipped 2026-05-18)
- [x] 24 curated tradition modules production-shipped.
- [x] 9 shape renderers implemented (`TraditionShape` union complete).
- [x] Tradition chip dropdown family-categorized; ⓘ disclosure opens manifests.
- [x] 24 manifests at `docs/traditions/<id>.md` with the README-documented schema.
- [x] User-definable tradition loader (Tier 1 JSON + Tier 2 JS plugin, Obsidian-trust consent).
- [x] CSP `script-src` updated to allow `asset:` for dynamic plugin import.
- [x] Full 15-locale chrome + canvas localization (`sight.v6.*` subtree complete in en/ar/de/es/fa/fr/he/hi/ja/ko/pt/ru/tr/ur/zh).
- [x] RTL-aware chevron flip + `padding-inline-end` count spacing.
- [x] Theme awareness (MIG-027) — chrome reads `ChromePalette`; semantic colors theme-invariant.
- [x] Phase μ 3-agent migration audit: all 10 architectural invariants PASS; migration path 9/9 PASS with 2 advisories.
- [ ] Channel-orthogonality automated test in CI (blocked on vitest runner — PJ-054).
- [ ] Performance budget tests pass (blocked on vitest runner — PJ-054).
- [ ] Channel-isolation invariant test (blocked on vitest runner — PJ-054).

---

## Appendix A — Files

| File | Purpose |
|---|---|
| `docs/Constellation-Sight-Concept-Paper-v4.1.md` | This document (current contract) |
| `docs/Constellation-Sight-Concept-Paper-v4.0.md` | v4.0 (superseded, historical) |
| `docs/Constellation-Sight-Concept-Paper-v3.1.md` | v3.1 (superseded, historical) |
| `docs/sight-redesign-design-concept-v0.3.md` | The design conversation that converged to v4.0 |
| `docs/sight-redesign-v0.3-full-layout.svg` | Visual reference for Sight v6 layout |
| `docs/sight-redesign-v0.3-tradition-chip-detail.svg` | Visual reference (pre-expansion 7-tradition variant) |
| `docs/traditions/README.md` | Manifest index + format key |
| `docs/traditions/aristotelian.md` … `docs/traditions/ibuanyidanda.md` | 24 curated tradition manifests |
| `docs/traditions/<lang>/<id>.md` | 14 locale mirrors (ar/de/es/fa/fr/he/hi/ja/ko/pt/ru/tr/ur/zh) |
| `docs/traditions/schema/tradition.v1.schema.json` | User-definable JSON schema |
| `docs/traditions/schema/EXAMPLE.json` | Tier-1 template |
| `docs/traditions/schema/SAMPLE-PLUGIN.js` | Tier-2 template |

## Appendix B — Cross-references

- `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` — the Five Acts, the 7+2 link types, the Living Link Architecture.
- `docs/Constellation Orientation & Onboarding v2.15.md` — current orientation; §3 Architecture, §4 Cognitive Engine, the v2.09 religious-lineage rule, the v2.10 curated picks.
- `lab/reports/SESSION-LOG-2026-05-17.md` + `lab/reports/SESSION-LOG-2026-05-18.md` — the MIG-026 build cascade trace.
- `docs/research/MIG-026-candidate-registers.md` — full survey of ~25 candidates with citations + geometric implications.

## Appendix C — The cascade tally, in one paragraph

The MIG-026 cascade — γ (Polanyi + Mohist) · δ.1 (Peirce + Habermas + rotation fix) · δ.2 (Dewey + Husserl + Longino + star size / chevron / pramāṇa rotation fix) · ε.1 (Ibn Rushd burhān) · ε.2 (Shāṭibī maqāṣid + grid star size fix) · ε.3 (Ibn Khaldūn ʿumrān) · ζ.1 (PaRDeS) · ζ.2 (Maimonidean spiral + new ladder renderer) · ζ.3 (Talmudic 13 middot) · η (Mencian + Wang Yangming + Sŏngnihak + binary-flow vertical) · θ (Mignolo + Dussel + Maldonado-Torres + Akan Wiredu + Ibuanyidanda + relational renderer + binary-flow concentric) · ι (manifests + ⓘ disclosure) · κ (user-definable JSON + JS plugin loader) · λ (full 15-locale chrome + canvas localization with fix-3/4/5/6) · μ (ship gate + 3-agent audit) — ran across 2026-05-17 evening and 2026-05-18, ~36 hours of focused work, ~50 commits, closing the 24-tradition + 9-shape + user-definable + full-localization expansion that this Concept Paper now documents. The Phase μ migration-rule audit returned zero blockers; the 2 advisories surfaced are tracked as PJ-055 and PJ-052 (this document).

---

*End of Concept Paper v4.1.*

*Cut at milestone tag `milestone/sight-v6.3-traditions-ship` (commit `99e4ed37`, 2026-05-18). Future revisions either ship as v4.2 alongside this file or as v5.0 if the architecture itself changes.*
