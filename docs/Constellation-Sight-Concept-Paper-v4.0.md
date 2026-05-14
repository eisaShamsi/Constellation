# Constellation Sight — Concept Paper v4.0

> **Status**: Architectural contract. Approved by Boss (Eisa) on 2026-05-13 after a three-round SME panel review (rounds 1, 2, 3 — all six SMEs returned GO WITH CONDITIONS on v0.3; conditions integrated below).
> **Date**: 2026-05-13
> **Supersedes**: `docs/Constellation-Sight-Concept-Paper-v3.1.md` (v3.1 of the Concept Paper) and the implemented Sight v5 architecture currently on `main`.
> **Specifies**: the next Sight implementation — referred to throughout as **Sight v6**.
> **Function in hand**: the binding contract that the next Sight MIG (number to be allocated when the build cascade starts) implements against.
>
> Two version axes kept deliberately distinct:
> - **Concept Paper version**: v3.1 → **v4.0** (this document)
> - **Implementation version**: v5 (current) → **v6** (specified here)

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
- **Not a graph view.** Obsidian's graph view is force-directed link soup; Sight's spatial grammar carries semantic meaning (radial = stratum, angular = time).
- **Not a settings dashboard.** Diagnostic only. No knobs that change behavior; the user acts through standard editor operations.
- **Not a coaching tool in v6.** The four-layer ambition (visual / diagnostic / recommendation / coaching) from v3.1 is preserved as direction but only layers 1–2 ship in v6. Layers 3–4 are post-v6 work.
- **Not single-tradition.** Sight is built on the Universal Epistemic Content Taxonomy (5 branches × 11 sources, cross-civilizational). The cultural frame is *switchable*, not assumed.

### 1.4 What changed from v3.1 to v4.0

| Aspect | v3.1 / Sight v5 | v4.0 / Sight v6 |
|---|---|---|
| Architecture | Seven-mode toggle bar (R · L · T · C · S · A · P) showing 1/7 of the cognitive portrait at any moment | Coordinated views: anchor dome + 4 mini-domes + facet sidebar + register chip |
| First-touch story | User had to mentally compose seven mode-switches into a portrait | Whole story visible at one glance via the anchor dome; diagnostics one gesture away |
| Cultural framing | Implicit Western-classical (Aristotelian stratification) | Seven explicit epistemic registers (Aristotelian default + 6 others); register chip on title bar |
| Channel encoding | Five lens channels stacked on one star with three confound pairs | Six pre-attentive channels + one focal + two deferred, no confounds, Bertin-clean |
| Library identity | Color tint (saturation gradient) | Shape only with neutral fill (`#cdd5e0`); color reserved for stage hue and link-type hue |
| Toggle/control bar | Persistent 7-mode bar | No persistent control bar; gestures replace toggles |
| Provenance | Encoded on every star (outer ring color) | Off the always-on star; available via facet sidebar + dedicated provenance mini-dome |
| Default chrome | All-on | **Default-simple**: anchor dome + collapsed sidebar + collapsed register chip + "Show diagnostics" affordance. Pro-mode opt-in |

The honest read of why v3.1 needed to be superseded: v5 passed every capability test but failed the outcome test (Eisa's 2026-05-13 verdict: *"Looking at the modes, it is confusing! Again, what is Sight telling? What is unique about Sight? NOTHING!"*). The redesign exists because the seven-mode toggle architecture, however well executed, does not satisfy the Suwaidi criterion.

---

## §2 — Architecture: Coordinated Views (Option D, locked)

### 2.1 Four zones on one screen

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  Constellation Sight                                                          │
│  [Aristotelian ●]  (collapsed register chip — click to expand)                │  ← title bar
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

**Default state on first open: only the anchor dome is visible.** Sidebar collapsed to a tab. Register chip collapsed to a single label. Mini-domes hidden. This is the Suwaidi-grade view that satisfies §1.2.

**Engaged state**: any of three gestures expand chrome:
- Click sidebar tab → facets unfold (full sidebar visible)
- Click register chip label → register options expand inline
- Cmd-D or "Show diagnostics" button → four mini-domes slide in from the right

### 2.2 The anchor dome (left-center, ~640×640 px target render)

The anchor dome is the Suwaidi-grade view. It is non-negotiable in v6 — every other zone supports it.

**Spatial grammar (locked):**
- **Radial position** = stratum (Foundation → Working → Connection → Synthesis → Edge of Knowing). Five bands, Foundation innermost.
- **Angular position** = month of creation, January at top, clockwise.
- **Shape** = library identity (circle / square / diamond / triangle / hexagon at 1–5 libraries; outline-style rotation at 6–25; degraded fallback above).

**Channel encoding on each star:**
- **Fill color**: neutral (`#cdd5e0`). No library hue. No exception.
- **Opacity**: confidence (0.4 hypothesis → 1.0 established; saturated for evidence; slightly desaturated for contested).
- **Size**: top-decile acts flag (binary: baseline 5 px, top-decile 7 px). The size delta is +40% to ensure pre-attentive size discrimination.
- **Inner pip hue**: stage (green established / cyan fresh / violet growing / yellow at-risk / gray dormant). Pip diameter ≥1.8 px at default zoom; explicitly **focal-on-foveation**, not pre-attentive on the anchor (pre-attentive in the Stage mini-dome).

**Line encoding:**
- **Typed-link connections** as curves between linked stars. Line color = link type:
  - supports = green, contradicts = red (dashed), causes = orange, exemplifies = blue, generalizes = purple, derives-from = cyan, part-of = pink, associative = gray, supersedes = pale yellow.
- Lines auto-fade above 800 visible (prevent overplotting); user can re-enable via Settings.

**Stratum reference circles**: five concentric guides at 0.6 px stroke `#1a1f2e` (very subtle, Suwaidi-style).

**Calendar rim**: 12 month labels at `r ≈ 340 px` in mid-gray.

**Stratum labels**: faint italic text along the vertical axis (FOUNDATION at center, EDGE OF KNOWING at top).

### 2.3 The four mini-domes (2×2 grid, hidden by default, ≥320×320 px each in production)

Each mini-dome shows the same notes in the same radial position as the anchor, but **isolates one channel** with its optimal visual property. Stratum bands preserved at 0.04 opacity (radial anchor metaphor never disappears).

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

The faceted-browse surface — Hearst's Flamenco pattern. Six facets with live counts; clicking any category cross-filters all views.

```
▼ Folder            (top facet — addresses the Universe/Folder hierarchy gap)
    Research                    1,247
    Projects                      892
    Reading-notes                 563
    + 12 more
▼ Library
    ● Research                  3,124
    ■ Projects                  2,447
    ◆ Personal                  1,205
    ▲ Reading                     560
    ⬡ Reference                   300
▼ Stratum
    Foundation                  1,856
    Working                     2,103
    Connection                  1,247
    Synthesis                     924
    Edge of Knowing               506
▼ Confidence
    hypothesis                    812
    evidence                    2,567
    established                 3,894
    contested                     363
▼ Stage
    ● established               3,141
    ● fresh                     1,247
    ● growing                   1,038
    ● at-risk                   1,254
    ● dormant                     956
▼ Provenance
    Self                        1,247
    Read                        3,894
    Heard                         563
    Reasoned                    1,532
    Tradition                     400
```

Each facet category is clickable. Click → all views filter, counts in OTHER facets rebalance to show *what's available given the current filter set* (Hearst Flamenco preview pattern). Multi-facet filters compose (AND across facets, OR within a facet).

**Folder is the top facet** because round-2 and round-3 LIS critique flagged it as missing from v0.2; v6 surfaces it.

**Universe** is implicit (the canvas = current Universe). For federated cUniverse view, a Universe selector chip will appear next to the register chip in v4.1.

### 2.5 The register chip (title bar)

The cross-civilizational lens. Seven epistemic registers, each remaps the anchor dome's semantic axes. **Applies to anchor dome only**; mini-domes stay culturally neutral (see §7).

**Default state**: collapsed, shows current active register only (e.g., `Aristotelian ●`).
**Engaged state**: click to expand the full chip row showing all 7 options.
**Active register highlight**: blue stroke + dot indicator.
**Hover any chip**: English secondary label appears as tooltip (e.g., *"pramāṇa — Nyāya fourfold valid means of knowing"*) — this is mandatory in v6 (round-3 UX SME Condition #9).

The seven registers and their geometries are specified in §4.

---

## §3 — Channel taxonomy

### 3.1 The honest pre-attentive ledger

v6 commits to honest channel-tier labeling. No "N channels at one glance" overclaim.

| Channel | Tier | Encoding | Notes |
|---|---|---|---|
| Stratum | **pre-attentive** | radial position | strongest channel per Mackinlay (1986) |
| Time | **pre-attentive** | angular position | second-strongest channel |
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
| Position (x,y) | Stratum + Time | Spatial anchor |
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

**Shape-weight normalization** (Bertin-aware, round-3 Info Design SME Condition #4): each shape rendered at equal *perceived* area, not equal bounding-box area. Diamond shrunk ~15%, triangle upscaled ~20%, hexagon shrunk ~10%, square baseline, circle baseline.

### 3.4 Color palette (v6 commit, CIE Delta-E ≥30 verified)

**Stage hue (5 categorical, anchor pip + mini-dome full-disk):**
- established = `#4ade80` (green)
- fresh = `#22d3ee` (cyan)
- growing = `#a78bfa` (violet)
- at-risk = `#facc15` (yellow)
- dormant = `#94a3b8` (gray)

**Typed-link line colors (9 kinds):**
- supports = `#4ade80` · contradicts = `#f87171` (dashed) · causes = `#fb923c` · exemplifies = `#60a5fa` · generalizes = `#a78bfa` · derives-from = `#22d3ee` · part-of = `#f472b6` · associative = `#94a3b8` · supersedes = `#fde68a`

**Hue overlap risk** (Stage green + Supports green; Stage violet + Generalizes violet; Stage cyan + Derives-from cyan): mitigated by spatial separation (lines are between stars, pips are inside stars) and confirmed by CIE Delta-E check at build time. If overlap fails Delta-E, link palette gets a hue rotation.

---

## §4 — The seven registers (v1 scope, polish-tiered)

All 7 registers ship in Sight v6.0. Four at **production polish**, three at **v1 preview** with explicit "polish in v4.1" labeling.

### 4.1 Production polish (4 registers, v1 ship-ready)

#### 4.1.1 Aristotelian (default)

**Geometry**: radial = stratum (Foundation → Edge of Knowing), angular = time. Same as default Sight grammar.
**Cultural framing**: Western-classical; knowledge as maturity gradient.
**Citation**: Aristotle, *Posterior Analytics*; Lloyd, *The Ambitions of Curiosity*.
**Why default**: makes the implicit Western frame explicit rather than smuggled.

#### 4.1.2 pramāṇa

**Geometry**: 4 quadrants for the Nyāya valid means of knowing: pratyakṣa (NE, perception), anumāna (SE, inference), upamāna (SW, analogy/comparison), śabda (NW, testimony). Quadrant dividers visible. Radial within quadrant = stratum (neutral, same encoding as Aristotelian); angular within quadrant = time.
**Cultural framing**: Indian Nyāya epistemology; honors pramāṇas as **kinds, not levels**.
**Citation**: Nyāya-Sūtra 1.1.3; Mohanty, *Classical Indian Philosophy* (2000), pp. 17–34; Matilal, *Perception* (1986), ch. 1.
**v4.1 polish target**: per-quadrant radial-internal structure (e.g., pratyakṣa: indriya-artha-sannikarṣa loci; anumāna: pakṣa/sādhya/hetu loci per the 5-membered syllogism).

#### 4.1.3 masādir

**Geometry**: 4 categorical sectors (NOT concentric ladder): Qur'an (NE), sunnah (SE), ijmāʿ (SW), qiyās (NW). Each sector annotated with kind distinction (naṣṣ vs. ijtihādī; qaṭʿī vs. ẓannī). Four extension chips below the dome: istiḥsān, istiṣḥāb, maṣlaḥa mursalah, ʿurf.
**Cultural framing**: Sunni *uṣūl al-fiqh*; sources as different kinds of proof, not degrees-of-one-thing.
**Citation**: al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl* (vol. 1, ed. Hafnawi), pp. 81–94; Rosenthal, *Knowledge Triumphant* (1970).
**Note**: ijmāʿ-as-ijtihādī is contested by Ash'arī/Māturīdī kalām (which treats it as transmitted/binding). v6 ships the Mustaṣfā-aligned reading; alternative kalām reading is a v4.1 variant.

#### 4.1.4 Polanyi

**Geometry**: single dome with tacit/explicit fog gradient — fog **dense at center** (tacit core, proximal pole), **clear at edges** (explicit periphery). Inner tacit stars at 0.14–0.18 opacity (acknowledged but visually inarticulable); outer explicit stars at 0.85–0.95 opacity (clearly readable).
**Cultural framing**: modern Western pluralism; tacit knowing as the proximal pole of all knowing.
**Citation**: Polanyi, *Personal Knowledge* (1958), ch. 4; *The Tacit Dimension* (1966), p. 10 ("we can know more than we can tell").
**Why inversion from v0.2 matters**: v0.2 had fog dense at edges, which inverts Polanyi's actual argument. v6 fixes this.

### 4.2 v1 preview, v4.1 polish target (3 registers)

These ship fully functional in v6.0 but with explicit "v1 preview" labeling on the chip + tooltip. Geometry implemented; deeper internal structure deferred.

#### 4.2.1 Dignāga

**Geometry**: vertical hemisphere split. Left hemisphere = pratyakṣa (perception only). Right hemisphere = anumāna (inference only). Center marked with explicit "rejected" labels for śabda and upamāna.
**Cultural framing**: Buddhist epistemological critique; only two valid pramāṇas. The absence is a *feature*, not a gap.
**Citation**: Dignāga, *Pramāṇasamuccaya* I.2; Hattori, *Dignāga, On Perception* (1968).
**v4.1 polish**: render pratyakṣa as svalakṣaṇa-instances and anumāna as sāmānya-lakṣaṇa categorization explicitly.

#### 4.2.2 Suhrawardi Ishrāqī

**Geometry**: luminous central core (`ʿilm ḥuḍūrī` — knowledge by presence) rendered as a small gold disc surrounded by an emanation glow. Three concentric emanation rings outward (dashed). Peripheral stars represent discursive/acquired knowledge (`ʿilm ḥuṣūlī`).
**Cultural framing**: Persian-Islamic Ishrāqī tradition; presence-knowledge as irreducible epistemic foundation.
**Citation**: Suhrawardi, *Ḥikmat al-Ishrāq* §I.5; Walbridge, *The Wisdom of the Mystic East* (2001).
**v4.1 polish**: render the emanation rings as labeled `anwār al-anwār` hierarchy (qāhir / mudabbir / etc.) per Suhrawardi's metaphysics.

#### 4.2.3 Mohist sān biǎo 三表

**Geometry**: three horizontal zones (not radial): historical precedent (top, 本 root), observational evidence (middle, 原 origin), social benefit (bottom, 用 use). Angular = time *across* zones (chronology orthogonal to standards).
**Cultural framing**: early Chinese pragmatist register; the three standards are *tests* of doctrines, not a time-ordering.
**Citation**: *Mozi* "Fei Ming" 非命 ch. 35; Lloyd, *Disciplines in the Making* (2009), ch. 4.
**v4.1 polish**: cross-reference each zone with named application examples; differentiate sān biǎo's role from sān fa 三法 (three methods).

### 4.3 Register chip behavior

**Switching a register**: only the anchor dome's spatial semantics change. Mini-domes, facet sidebar, and gesture grammar are unchanged. The user's current filters (from sidebar or mini-dome clicks) remain applied.

**Highlighted-star behavior during register switch** (Info Design SME Condition, round 3): when a star is highlighted via linked brushing and the user switches register, the star's *anchor-dome position* changes (it now sits in a different quadrant or zone) but its *mini-dome positions* remain. v6 animates the anchor-dome position transition over 400ms so the user can track where the star moved (v4.1 polish; v6 v6.0 ships with instant snap + a brief flash to maintain identity).

**Register definitions** as version-controlled manifest (LIS SME Condition, round 3): each register defined in `docs/registers/<id>.md` with schema `{id, name, citation, geometry_spec, sectors, exclusions, extensions, version, changelog}`. Citation surfaced via "ⓘ" affordance on the chip.

---

## §5 — Gesture grammar

Sight has no persistent toggle controls. All interaction is through gestures.

| Gesture | Effect | Default state vs Engaged state |
|---|---|---|
| Click sidebar tab "Filters ▶" | Expand facet sidebar | Triggers Engaged |
| Click "Aristotelian ●" label in title | Expand register chip row | Triggers Engaged |
| Cmd-D or click "Show diagnostics" button | Reveal mini-domes (slide in from right) | Triggers Engaged |
| Click any facet category | Cross-filter all views; facet counts rebalance | — |
| Click a mini-dome category | Cross-filter all views | — |
| Click a library shape in sidebar | Other libraries fade to 0.3 opacity | — |
| Right-click a stratum band | Other strata dim; selected stays bright | — |
| Hover a stratum band | Diagnostic popover (count, confidence avg, stage breakdown, acts %, link density) | — |
| Hover a star | Side popover with full lens breakdown | — |
| Click a star | Open note in editor | — |
| Click a register chip | Re-frame anchor dome's semantic axes | — |
| Hover a register chip | English tooltip ("pramāṇa — Nyāya fourfold valid means of knowing") | — |
| Esc | Reset filters, close popovers, collapse expanded chrome back to default | Returns to default |
| Cmd-F | Search overlay highlights matching stars, dims non-matches | — |
| Cmd-Shift-D | Toggle Pro mode permanently in Settings (persistent default) | — |

**First-boot orientation tour** (UX SME Condition, round 3): on first ever open of Sight, a skippable 4-step overlay appears:
1. "This is your knowledge universe. Each star is a note; position shows where it lives in your thinking."
2. "Hover any star for detail. Click to open."
3. "These tabs let you filter, switch perspective, or open the full diagnostic view."
4. "Esc resets. Cmd-F searches. Try clicking a stratum."

Auto-skipped on subsequent opens. Always available via Help → "Sight tour."

---

## §6 — Default-simple, Pro-opt-in (first-touch behavior)

This is the most important interaction model commitment in v6. It is what makes the Suwaidi criterion satisfiable.

### 6.1 Default state (first open, every time)

- Anchor dome visible (full canvas devoted to it minus title strip + collapsed sidebar tab + collapsed register label).
- Sidebar collapsed to a tab on the left edge ("Filters ▶" label, 20 px wide).
- Register chip collapsed to a single label in the title bar (current register name + dot).
- Mini-domes hidden. A subtle "Show diagnostics" link in the top-right corner.
- Status strip at bottom shows: universe note count + healthy/at-risk/dormant percentages.

Total visible interactive surfaces in default state: **4** (anchor dome + sidebar tab + register label + diagnostics link). This is well below v0.3's full-engaged count of 10.

### 6.2 The Suwaidi-fidelity guarantee

In default state, the anchor dome occupies **≥80% of the visible canvas (excluding title strip and status strip)**. This is the architectural guarantee that protects §1.2. Any layout work in v6 that violates this guarantee fails the visual contract.

### 6.3 Engagement gestures

Any single click expands one zone:
- Click sidebar tab → sidebar slides out to 180 px width, anchor dome compresses to remaining space (anchor still readable).
- Click register label → register row expands inline at the top.
- Click "Show diagnostics" or press Cmd-D → mini-domes slide in from right, anchor dome compresses to ~60% width.

Each expansion is **independent**. The user can engage any subset.

### 6.4 Persistent Pro mode

If the user prefers always-expanded chrome, **Cmd-Shift-D toggles Pro mode** as a persistent setting. Pro mode default state = all chrome expanded on every open. This is opt-in only — never the system default.

### 6.5 The fidelity vs. engineering trade

v6 ships every engineered surface (mini-domes, register chip, facet sidebar) — none of this work is wasted. The only thing that changes is **default chrome visibility**. The Suwaidi-grade story-at-glance reads on first open; the user discovers deeper surfaces as they engage. This honors both Eisa's stated outcome criterion AND the panel's verdict that the engineered surface is valuable.

---

## §7 — Mini-dome channel ontology (Western-analytic by stipulation)

The four mini-domes encode Confidence, Stage, Acts, and Provenance. These channel names are derived from the analytic-Western tradition (Bayesian confidence; lifecycle staging; activity quantiles; sourcing categories).

**v6 explicitly stipulates**: the mini-dome channel names are **by-stipulation labels for the underlying note metadata**, not claims about the universal structure of knowledge. The metadata fields in the SQLite store (`confidence`, `lifecycle_stage`, `act_density`, `provenance_source`) are what they are regardless of register.

**The register chip remaps the anchor dome's spatial semantics only**; mini-dome channel labels stay constant across registers in v6.0. This is the architectural commitment that prevents rhetorical pluralism — the cultural framing is honest about what it does and doesn't do.

**v4.1 enhancement** (deferred): register-aware mini-dome relabeling. When the masādir register is active, the Confidence mini-dome relabels its axis as qaṭʿī/ẓannī; the Provenance mini-dome relabels sectors as the four uṣūl sources. When Dignāga is active, the Provenance mini-dome collapses to two sectors (pratyakṣa, anumāna). This is purely relabeling — the underlying metadata is unchanged. Estimated cost: ~3 weeks engineering.

---

## §8 — Visual contract (what v6 locks in)

### 8.1 Channel orthogonality invariant

No two channels share a Bertin visual variable. Specified in §3.2. Verified at code-review time on every Sight PR. If a future change would put two channels on the same variable, the PR is blocked.

### 8.2 Default-state Suwaidi-fidelity guarantee

In default state, the anchor dome occupies ≥80% of visible canvas (excluding title and status strips). Verified by an automated layout test in the test suite.

### 8.3 Performance budget

- **Cross-filter response**: ≤16 ms on a 7,636-note universe with 5 coordinated views (one anchor + 4 mini-domes). Verified via render-budget test in CI.
- **Default-state render**: ≤100 ms from store-ready to first paint on a 10,000-note universe.
- **Pro-mode render**: ≤180 ms with all 5 views.
- **Hex-bin aggregation kicks in**: above 5,000 visible notes per view. d3-hexbin library required.

### 8.4 CIE Delta-E ≥30 between co-rendered hues

Stage palette (5 hues) and Link palette (9 hues) audited at build time. Co-rendering checks: stage-pip vs link-line within ≤5 px region. If any pair fails Delta-E ≥30, build fails.

### 8.5 Pip foveation threshold

Anchor pip diameter ≥1.8 px at default zoom. At zoom levels where pip would render <1.5 px, pip is suppressed entirely (Stage falls back to mini-dome only). This prevents the "claimed but invisible" channel failure mode.

### 8.6 Register manifest contract

Each register defined in `docs/registers/<id>.md` with the schema in §4.3. Cross-Civ corrections (pramāṇa, masādir, Polanyi) are checked into version control. Any register geometry change requires a PR with citation update.

---

## §9 — Implementation contract (Sight v6 build)

### 9.1 Module organization

New directory: `src/lib/sight/v6/`.

| File | Purpose | Predecessor in v5 |
|---|---|---|
| `SightV6.svelte` | Main component, layout, default-vs-Pro state machine | `src/lib/sight/v5/SightV5.svelte` (rebuilt; v5 deleted after v6 ships) |
| `anchor.ts` | Anchor dome Canvas-2D renderer | `src/lib/sight/v5/render.ts` (substantial rebuild — multi-channel, no mode toggle) |
| `miniDome.ts` | Single mini-dome renderer (instantiated 4×, parameterized by channel) | New |
| `facetSidebar.svelte` | Facet sidebar with cross-filter logic | New |
| `registerChip.svelte` | Register chip with collapse/expand + tooltip | New |
| `registers/<id>.ts` | Per-register geometry remap (7 files, one per register) | New |
| `gestures.ts` | Gesture dispatch (right-click, click-filter, hover, Esc, Cmd-F, Cmd-D) | Partial in `src/lib/sight/v5/SightV5.svelte` (extracted + extended) |
| `tour.svelte` | First-boot orientation overlay | New |
| `types.ts` | TypeScript contracts | `src/lib/sight/v5/types.ts` (rebuilt) |

Backend: new `src-tauri/src/sight_v6.rs` with new cache schema. `src-tauri/src/sight_v5.rs` deleted when v6 ships.

`src/lib/sight/engine.ts` feature flag: `SIGHT_V6_ENABLED = true`; v2/v3/v4/v5 all `false`.

### 9.2 Phased build (4 phases)

The Concept Paper supports staged shipping. Each phase delivers user-visible value.

#### Phase 1 — Anchor dome + facet sidebar + Default-simple (Sight v6.0)

- Anchor dome with full channel encoding (shape, opacity, pip, size, lines).
- Facet sidebar with 6 facets, live counts, cross-filter logic.
- Default-simple layout: anchor + collapsed sidebar tab + collapsed register label + Show-diagnostics link.
- First-boot orientation tour.
- All gestures except mini-dome cross-filter (no mini-domes yet).
- **Ships as Sight v6.0.** Replaces Sight v5 on `main`.
- **Effort**: ~6 weeks.

#### Phase 2 — Mini-domes + Pro mode (Sight v6.1)

- Four mini-domes implementation (Confidence opacity, Stage hue, Acts size, Provenance sectors).
- Cross-filter from mini-domes; linked brushing across all 5 views.
- Hex-bin aggregation above 5,000 visible.
- Cmd-D / Show-diagnostics gesture wired.
- Pro mode toggle (Cmd-Shift-D persistent).
- **Ships as Sight v6.1.**
- **Effort**: ~4 weeks.

#### Phase 3 — Register chip + 4 production-polish registers (Sight v6.2)

- Register chip with collapse/expand, English hover labels.
- Aristotelian (default), pramāṇa, masādir, Polanyi at production polish.
- Per-register `registers/<id>.ts` geometry remap modules.
- Register manifest at `docs/registers/`.
- Register switch animation (anchor-dome position transition, 400ms).
- **Ships as Sight v6.2.**
- **Effort**: ~5 weeks.

#### Phase 4 — 3 v1-preview registers + polish + final tests (Sight v6.3)

- Dignāga, Suhrawardi Ishrāqī, Mohist sān biǎo at v1-preview polish with explicit labels.
- Channel orthogonality automated test in CI.
- Performance budget tests (16ms cross-filter, 100ms default render).
- CIE Delta-E palette validation.
- v5 module deletion.
- **Ships as Sight v6.3 (final v6).**
- **Effort**: ~3 weeks.

**Total**: ~18 weeks of focused engineering, phased.

### 9.3 Migration from Sight v5 → Sight v6

- v5 deletion: after v6.0 ships and runs cleanly for one release cycle (likely ~2 weeks of usage). Phase 4 removes the v5 module set.
- Settings migration: `sight_v5_*` keys read once at v6.0 first boot, mapped where possible, then deleted. Specifically: `pinnedMode` is dropped (no modes in v6); `wedgeFilter` is dropped (no wedges in v6); `scope` carries forward to v6's universal scope.
- SQLite cache migration: `sight_v5_layout` table is dropped; new `sight_v6_layout` table created with sentinel version field. Backfill runs on first boot in background with progress in status bar (resumable per Standing Order).
- User-facing: a Settings note explaining the upgrade appears once at first v6.0 launch. No mid-edit interruption.

### 9.4 Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Phase-1-only ship feels incomplete | Medium | Default-simple state means most users won't notice missing mini-domes for a while; "Show diagnostics" gesture exists but renders a "coming in v6.1" placeholder. |
| Register implementation cost overruns | High | Polish tiering: 4 production + 3 v1-preview. Preview labeling sets user expectation. |
| Performance fails at >10k notes | Medium | Hex-bin aggregation fallback at ≥5,000 visible. WebGL upgrade path reserved for v6.4 if needed. |
| Cross-Civ register critique resurfaces | Low | Citation manifests in `docs/registers/` + v4.1 polish targets named explicitly. |
| User misses removed v5 modes (R/L/T/C/S/A/P) | Low | First-boot tour explains the new model. Settings → Help has a "v5 → v6 migration guide." |
| Library color loss feels austere | Medium | v4.1 escape hatch: low-saturation tint as recognition aid (optional). |
| Folder-on-dome request resurfaces | Low | Sidebar surfaces Folder. If users push for dome encoding, revisit in v4.1. |
| Register-switch animation creates motion sickness | Low | Animation respects `prefers-reduced-motion`; falls back to instant snap. |

### 9.5 Tech stack

- **Frontend**: Svelte 5 + Canvas 2D rendering. SVG only for static legend icons. No WebGL in v6 (reserved for future).
- **Geometry / layout math**: TypeScript pure functions in `anchor.ts`, `miniDome.ts`, per-register modules.
- **Aggregation**: `d3-hexbin` (proven library; do not roll your own).
- **Backend**: Rust (`sight_v6.rs`); SQLite cache; Tauri events for live updates.
- **Test framework**: existing vitest + playwright stack. New invariant tests for channel orthogonality and Suwaidi-fidelity guarantee.

---

## §10 — v4.1 polish targets (post-v6.0)

Deferred items, prioritized:

1. **Pramāṇa internal-structure rendering** (per-quadrant: indriya-artha-sannikarṣa loci, etc.).
2. **Dignāga, Ishrāqī, Mohist polish** (v1-preview → production).
3. **Register-aware mini-dome relabeling** (§7 enhancement).
4. **Color-accessibility variant** (high-contrast / colorblind-safe palette).
5. **Animated transitions** (register switch refinements, mini-dome reveal eased animation).
6. **Universe selector chip** for cUniverse federation view.
7. **Library color recognition aid** (low-saturation tint, opt-in setting).
8. **Layer 3 (Recommendations) and Layer 4 (Coaching)** — the post-diagnostic guidance the v3.1 paper named but v6 only foundations.

These are v4.x work, not v6.x work. The Concept Paper bump (v4.0 → v4.1 → ...) tracks them; the implementation version stays v6 until a structural redesign happens.

---

## §11 — Open invariants (the contract v6 guarantees)

1. **Channel orthogonality**: no two channels share a Bertin variable.
2. **Default Suwaidi-fidelity**: anchor dome ≥80% of visible canvas in default state.
3. **Cross-filter performance**: ≤16 ms on 7,636 notes × 5 views.
4. **CIE Delta-E ≥30**: between any two co-rendered hues at build time.
5. **Pip foveation threshold**: anchor pip ≥1.8 px at default zoom, suppressed below 1.5 px.
6. **Register isolation**: register chip remaps anchor dome only; mini-domes stay culturally neutral.
7. **Register manifest**: each register's geometry is documented + citation-tracked in version control.
8. **Folder visibility**: Folder is a first-class facet in the sidebar.
9. **Gesture chrome**: no persistent toggle bars. All interaction via gestures + sidebar/chip/mini-dome clicks.
10. **First-boot tour**: 4 steps, skippable, always re-available in Help.

A future change that violates any of these is, by definition, no longer Sight v6 — it is v7 or a regression. The invariants are the contract.

---

## §12 — Supersession

- `docs/Constellation-Sight-Concept-Paper-v3.1.md` is **superseded** by this document. v3.1 remains on disk as historical record.
- The implemented Sight v5 (current `main`) is **superseded** by Sight v6 (specified here). v5 sunset per §9.3.
- All v0.x design concept docs (`sight-redesign-design-concept-v0.1.md`, `v0.2.md`, `v0.3.md`) are **archived as design conversation history**. v4.0 is the binding contract; v0.x are the artifacts that led to it.
- All v0.x mock SVGs are **historical record**. The v0.3 full-layout + register-chip detail SVGs remain the visual reference for Sight v6 implementation.

---

## §13 — Verification clauses

At each phase ship, the following must hold:

### After Phase 1 (Sight v6.0)
- [ ] Anchor dome renders with all 6 pre-attentive channels per §3.1.
- [ ] Default-simple layout satisfies §6.2 (≥80% anchor).
- [ ] Facet sidebar cross-filters correctly across stratum, library, confidence, stage, provenance.
- [ ] Folder facet shows accurate counts and filters.
- [ ] First-boot tour fires on first open, skippable.
- [ ] All gestures from §5 work except mini-dome cross-filter.
- [ ] v5 module set still present (deleted in Phase 4).

### After Phase 2 (Sight v6.1)
- [ ] Four mini-domes render with their isolated channel encoding.
- [ ] Stratum bands at 0.04 opacity visible in each mini.
- [ ] Linked brushing (gold ring) propagates across all 5 views.
- [ ] Click in mini-dome filters all 5 views; counts rebalance.
- [ ] Hex-bin aggregation kicks in above 5,000 visible; per-star below.
- [ ] Cmd-D toggles diagnostics visibility.
- [ ] Pro mode persists across sessions.

### After Phase 3 (Sight v6.2)
- [ ] All 4 production-polish registers (Aristotelian, pramāṇa, masādir, Polanyi) render correctly per §4.1.
- [ ] Hover tooltip on each register chip shows English secondary label.
- [ ] Register switch animation runs at 400ms with motion-reduce respect.
- [ ] Register manifests in `docs/registers/` are present with citations.
- [ ] Mini-dome channels unchanged across register switches (§7 stipulation honored).

### After Phase 4 (Sight v6.3)
- [ ] Dignāga, Ishrāqī, Mohist sān biǎo render with "v1 preview" labels.
- [ ] Channel orthogonality automated test in CI.
- [ ] Performance budget tests pass (≤16 ms cross-filter, ≤100 ms default render).
- [ ] CIE Delta-E ≥30 verified for stage + link palettes.
- [ ] v5 module set deleted; v5 settings keys migrated/dropped.
- [ ] v5 SQLite cache table dropped; v6 cache present.

The Phase-4 verification is the v6.0 ship gate. Until every box is checked, Sight v6 is not done.

---

## Appendix A — Files

| File | Purpose |
|---|---|
| `docs/Constellation-Sight-Concept-Paper-v4.0.md` | This document (current contract) |
| `docs/Constellation-Sight-Concept-Paper-v3.1.md` | v3.1 (superseded, historical) |
| `docs/sight-redesign-design-concept-v0.3.md` | The design conversation that converged to v4.0 |
| `docs/sight-redesign-v0.3-full-layout.svg` | Visual reference for Sight v6 layout |
| `docs/sight-redesign-v0.3-register-chip-detail.svg` | Visual reference for the 7 registers |
| `docs/registers/aristotelian.md` | (TBD — to be created in Phase 3) |
| `docs/registers/pramana.md` | (TBD) |
| `docs/registers/masadir.md` | (TBD) |
| `docs/registers/polanyi.md` | (TBD) |
| `docs/registers/dignaga.md` | (TBD) |
| `docs/registers/ishraqi.md` | (TBD) |
| `docs/registers/mohist-san-biao.md` | (TBD) |

## Appendix B — Cross-references

- `docs/CONSTELLATION-KNOWLEDGE-FORMULATION.md` — the Five Acts, the 7+2 link types, the Living Link Architecture.
- `docs/sight-v5-mode-concepts.md` — semantic source for channel encodings (carries forward into v6 channel taxonomy).
- `docs/sight-v5-purpose-achievement-audit.md` — the audit that opened the redesign.
- `lab/reports/SESSION-LOG-2026-05-13.md` — the session log capturing the redesign trajectory.

## Appendix C — Round-3 SME panel verdicts (the GO-with-conditions evidence)

All six SMEs returned GO WITH CONDITIONS on v0.3, with composite scoreboard:

| SME | Score | Δ vs round-2 D | Verdict |
|---|---|---|---|
| Information Design | 13 | ↑ +2 | GO |
| Cognitive Psychology | 12 | same | GO |
| Library & Info Science | 14 | ↑ +2 | GO |
| Data Visualization | 14 | ↓ −1 | GO |
| Cross-Civ Epistemology | 12 | ↑ +1 | GO (one fix: pramāṇa prose, settled before v4.0) |
| End-User UX | 13 | ↑ +2 | GO |

The 14 spec-level conditions from those verdicts are integrated into this contract:
- §3.1 honest channel tier ledger (cluster gestalt as emergent, not pre-attentive)
- §3.3 shape-weight normalization
- §3.4 CIE Delta-E ≥30 invariant
- §4 register manifest contract + citation tracking
- §4.1.4 Polanyi fog inversion
- §4.2 polish tiering (3 registers labeled "v1 preview")
- §4.3 register-switch animation behavior
- §5 first-boot tour
- §5 English hover tooltips on register chips
- §6 Default-simple / Pro-opt-in interaction model
- §7 Western-analytic stipulation for mini-dome channels
- §8.3 measured performance budget (16ms cross-filter)
- §8.5 pip foveation threshold spec
- §9.3 migration plan with progress reporting

---

## Appendix D — The redesign journey, in one paragraph

Sight was originally Lens, renamed Sight, then iterated through v2/v3/v4/v5 implementations against Concept Paper v3.0 → v3.1. v5 shipped a seven-mode toggle bar (R · L · T · C · S · A · P) that passed every capability test but failed Eisa's outcome test ("the modes are confusing... what is unique about Sight? NOTHING!"). Eisa surfaced the Al-Suwaidi northern-hemisphere star chart as the design north star and asked for a one-mode redesign that "tells the whole story at one glance." The redesign worked through three SME-panel rounds: v0.1 (3 mock-ups, 6 SMEs surfaced 3 structural problems), v0.2 (5 options responding to round-1 critique, panel converged on Option D with composite 72 vs. 63/59/56/54), v0.3 (Option D locked, 5 panel adjustments + 7 registers + shape-only library + honest channel-tier labels). Round 3 returned unanimous GO WITH CONDITIONS. Eisa made the final calls on register count (ship 7 in v1, polish-tiered), default chrome density (Default-simple / Pro-opt-in), and mini-dome ontology (Western-analytic by stipulation), then delegated three remaining tensions back to design judgment — those calls are now baked into this contract. The pramāṇa prose ↔ SVG mismatch (Cross-Civ SME's one hard condition) is settled before v4.0 ratification. The contract specifies the build for Sight v6, replacing v5 on `main`, in four phases over ~18 weeks.

---

*End of Concept Paper v4.0. Awaiting Boss ratification.*

*On ratification: a new MIG number is allocated for the Sight v6 build cascade; Phase 1 begins per §9.2.*
