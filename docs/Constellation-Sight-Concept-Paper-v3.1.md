# Constellation Sight — Concept Paper v3.1

**Version:** 3.1 (Eisa-feedback iteration on v3.0 — analytical instrument + diagnostic question reframed)
**Date:** 2026-05-12
**Author:** Eisa Alshamsi · eisa@uconstellation.world · drafted with Claude.
**Status:** Draft for Eisa's validation and approval.
**Supersedes:** `Constellation-Sight-Concept-Paper-v3.0.md` (2026-05-12 — first synthesis pass; this v3.1 folds in Eisa's structural corrections), and through that v3.0's chain of supersession (v2.0 → v1.x analytical paper, v3 visual paper, v3 visual spec). All older papers stay on disk per SO #6.
**Reference visual:** `docs/Sight-vNext-MockB1-Toggle.svg` (the Mock B1 Eisa approved on 2026-05-09 — note: §6 specifies a 7-button toggle for production, so the SVG needs a follow-up edit to add the P button before MIG-NN visual-foundation Architect lands).

> **What changed in v3.1** (Eisa-directed, 2026-05-12 feedback pass):
>
> 1. **§1 Executive summary REFRAMED — Sight v5 IS analytical.** v3.0 had said *"It does not analyze, score, recommend, or coach."* Eisa's correction: *"What I want is to be able to analyze, score, recommend, and/or coach. I want Sight to be an analytical instrument that, after identifying the shape of the user's Cognitive Knowledge and Epistemic Content, will help the user enhance their Cognitive and Epistemic Knowledge. It is like having your own local AI."* v3.1 promotes Sight from "visualization-only" to a **four-layer instrument**: visual foundation → diagnostic → recommendation → coaching, all running on local inference. The "shape and organization" reading from v3.0 becomes Layer 1; the analytical / coaching ambition becomes Layers 2–4.
> 2. **§2 Canonical question REFRAMED.** v3.0 had *"How is my Epistemic Content shaped and/or organized?"* Eisa's correction: *"Sight v5 should answer 'is my universe healthy? If not, where does it need to be handled?'"* The new canonical question is diagnostic and actionable. The old shape-and-organization phrasing survives as the *visual* question Layer 1 answers en route to the new canonical question.
> 3. **§3 Lineage trimmed.** Eisa: *"Do you want the v0 PDF Lens framing kept in or trimmed? trimmed."* Lens v0 row removed; lineage starts at Lens v1 / Sight v1.
> 4. **§4.1 + §4.2 Taxonomy framing TIES TO CECE'S LIVE STATE.** Eisa: *"True, but also, you have to consider how CECE becomes."* / *"check also the progress of our CECE, to be considered."* The taxonomy is no longer described as a static skeleton; v3.1 acknowledges that CECE has built out a richer live classification taxonomy (`horizontal_taxonomy.rs` + `vertical_taxonomy.rs`, ~280 nodes) and that Sight v5's mode P visualizes whatever the live taxonomy carries — not just the 5-branch × 11-source skeleton.
> 5. **§5.4 Connector lines — `supersedes` ADDED.** Eisa: *"Don't forget the recent one; supersedes."* The 9th typed link (slate blue-gray `#5B7A8A`, shipped MIG-022 §A.2) now appears in the connector-line color table. All 9 link types named.
> 6. **§6 Seven modes — production toggle has 7 buttons from the start.** Eisa: *"It should have 7 not 6."* v3.0 had said the toggle bar shows 6 (Mock B1) and adds P later. v3.1 commits production to 7 from Day 1: the Mock B1 SVG needs a follow-up edit to add the P button at the end of the row (with the dimmed dashed-border style for users whose Sources data is sparse).
> 7. **§10 boundary table — KHD vs Sight v5 health distinction sharpened.** With Sight v5 now answering "is my universe healthy?", the line against the Knowledge Health Dashboard (which also asks a health question) needs to be cleaner. KHD = link-graph health (modularity, decay, hub patterns); Sight v5 = epistemic-posture health (strata distribution, source diversity, confidence balance, growth trajectory). Two health framings, two surfaces.
> 8. **§12 phased rollout LAYERED.** v3.0 had two MIGs (visual + mode completion). v3.1 has four layered MIGs corresponding to the four instrument layers. Layer 1 (visual foundation) ships first as the standalone deliverable; Layers 2–4 build progressively on top.
> 9. **MIG number-collision RESOLVED.** Eisa: *"Your call."* v3.1 commits: the gap-analysis-response MIG-022 (already shipped through §A; §B in progress) keeps its number; Sight v5's visual foundation gets **MIG-024** (since MIG-023 is already reserved for the Warrant Research workstream).

---

## §0 · What this paper IS (and is NOT)

**This paper IS the design contract for Constellation Sight v5.** Every implementation phase, every Architect doc, every code commit that touches Sight reconciles against this paper. Disagreements between this paper and the running app are resolved by amending this paper first, code second.

**This paper IS NOT a status report.** As of writing, **no Sight v5 visual code has been written.** Sight v4 (`SIGHT_V4_ENABLED = true` in `src/lib/sight/engine.ts`) is the user-visible Sight on `main`. v5 is gated on this paper's approval, the visual-foundation Architect doc (MIG-024), and Eisa's per-MIG sign-off under the /migration discipline.

**This paper IS NOT the Universal Epistemic Content Taxonomy.** The taxonomy is the scholarly foundation — see `docs/epistemic-content-taxonomy.md`, `docs/epistemic-content-EN.md`, `docs/epistemic-content-AR.md`. The *live* taxonomy (the classification IDs CECE actually fires against) lives in `src-tauri/src/sources/horizontal_taxonomy.rs` and `vertical_taxonomy.rs` — and is richer than the published taxonomy skeleton.

**This paper IS NOT a v4-to-v5 patch plan.** v5 is built fresh on a stable spatial grammar with a layered analytical instrument on top. v4 stays as the rollback target until Eisa confirms v5 across multiple sessions; v4 retires in a cleanup MIG.

---

## §1 · Executive summary

**Constellation Sight v5** is a **four-layer analytical instrument** built around a single full-screen surface — a stable star chart on cream parchment that visualizes a user's entire knowledge universe. Each note is a star. The dome is divided into eight concentric strata bands (L1 Datum at the rim, L8 Worldview at the pole). Seven mode toggles re-cut the rim wedges to answer different cognitive questions; the strata radius, star size, brightness, and link colors stay constant across every mode.

The four layers, in build order:

| Layer | Job | Phase |
|---|---|---|
| **1 — Visual foundation** | Show the user the shape and organization of their Epistemic Content as a stable star chart they can learn and remember. | Ships first (MIG-024). |
| **2 — Diagnostic** | Assess the health of the user's epistemic posture: stratum distribution, source diversity, confidence balance, growth trajectory, dormancy patterns. Surface findings as plain-language signals on the dome. | MIG-025. |
| **3 — Recommendation** | When Layer 2 finds something worth handling, name it specifically. *"Your L4 hypotheses have stalled — 47 of them have not gained an `evidence` link in 90 days."* *"Your universe is 71 % testimony — consider where you'd benefit from independent inference."* | MIG-026. |
| **4 — Coaching** | Walk the user through specific enhancement actions, using local-LLM inference (CECE's Tier 2 Qwen3-1.7B, no cloud). *"Want to promote one of those stalled hypotheses to L5? I'll suggest the three with the strongest existing supporting links — pick one and I'll guide you through what `evidence` to gather."* | MIG-027. |

**Sight v5 is not a passive viewer.** It is an instrument that, after showing the user the shape of their thinking, **helps them improve it** — diagnosing weaknesses, recommending actions, coaching through enhancements. Eisa's framing: *"like having your own local AI."*

The local-AI framing is load-bearing. All inference for Layers 2–4 runs on the user's device (CECE's existing infrastructure: e5-small ONNX for embedding, Qwen3-1.7B GGUF via llama.cpp for reasoning). No data leaves the device. No cloud dependency. The coaching is private the way a private tutor is private — only the user and the instrument know what was discussed.

The target experience for Layer 1 (visual foundation): a first-time user with no Constellation training opens Sight v5 and within ~5 seconds can articulate what they're looking at. That ~5-second comprehension threshold is the load-bearing acceptance criterion for Layer 1; Layers 2–4 add depth without breaking it.

---

## §2 · The canonical question

> *Is my universe healthy? If not, where does it need to be handled?*

This question (Eisa, 2026-05-12) replaces v3.0's *"How is my Epistemic Content shaped and/or organized?"* — which itself replaced the InfraNodus-era *"What patterns and gaps exist in my thinking?"*. The evolution traces the maturation of Sight's ambition: from *"show me the analytics"* → *"show me the shape"* → *"tell me if I'm healthy and what to do about it."*

The question is **diagnostic and actionable**:

- **Diagnostic** — Sight has criteria for what an epistemically healthy universe looks like. It computes against those criteria continuously and reports findings in plain language. This is Layer 2's job.
- **Actionable** — when the diagnostic finds something off, Sight names what to handle and walks the user through handling it. This is Layers 3 + 4's job.

What does *healthy* mean for an epistemic universe? The criteria are pluralistic (no single "right" shape) but the dimensions are concrete:

| Health dimension | Unhealthy pattern | What "handling" looks like |
|---|---|---|
| **Strata distribution** | All notes stuck at L1-L2 (data without theory) or all at L7-L8 (worldview without ground) | Recommend stratum promotions where the data supports it; surface synthesis prompts |
| **Source diversity** (mode P / CECE) | One source dominates >70 % (e.g., all testimony, no inference; all perception, no postulation) | Highlight under-represented sources; recommend notes ripe for cross-source examination |
| **Confidence balance** | Universe is 90 % `hypothesis` confidence (everything tentative, nothing locked) or 90 % `established` (nothing being questioned) | Recommend specific hypotheses to test; surface established notes that haven't been re-examined recently |
| **Growth trajectory** | Most notes haven't gained a typed link in 60+ days (universe is calcifying) | Surface the dormant cluster; recommend bridge candidates |
| **Contested resolution** | Many notes carry contested links that have been unresolved for months | Recommend a tension-resolution session for the oldest contested cluster |
| **Coverage of Acts** | All Observation, no Tension/Synthesis/Conviction (collecting without reasoning) | Recommend Tension or Synthesis notes that would naturally bridge existing Observation clusters |

Layer 1 (the visual foundation) STILL answers the v3.0 question of *shape and organization* — that is the perceptual surface the diagnostic reads against. The user sees the shape; the instrument reads the shape; the diagnostic surfaces what's worth handling.

What the question excludes: Sight v5 does not answer *"where is X?"* (Search Hub), *"what's connected to what?"* (Sky View / Backlinks / Outgoing), or *"what is the structure of THIS note?"* (360.3D / Inspector). Sight is the universe-shape-and-health surface.

---

## §3 · Lineage — four Sight identities, one through-line

Sight has carried four implementation identities (after trimming the Lens v0 PDF concept that never shipped). Naming the lineage is the audit trail that explains why v5 looks the way it does.

| Identity | Period | Visual grammar | What it taught |
|---|---|---|---|
| **Lens v1** → **Sight v1 rename** | early-mid 2026 (`apply_lens` shipped) | Group-by-tag/property collapse | Group-by is a *query*, not a perceptual surface. Withdrawn 2026-05-09. |
| **Sight v2** | spring 2026 (`ConstellationSight2.svelte`, force-directed Pixi.js) | Force-directed graph with InfraNodus-style overlays (Brandes centrality, Louvain communities, structural gaps, universe health) | The math was right; the layout re-ran every session, so the user could never build a spatial mental map. |
| **Sight v3** | 2026-05-07 (Concept Paper + Visual Spec; partial implementation) | Star chart with **per-mode (X, Y, Z) grammar** — each mode declared its own azimuth, radius, magnitude; stars flew across the dome between modes | Star-chart aesthetic was right. Per-mode XYZ destroyed spatial memory the moment the user toggled a mode (the v2 problem in slow motion). |
| **Sight v4** | 2026-05-08 (Canvas 2D + D3-zoom) | Same v3 grammar; rendering tech swap to fix close-button regressions on a `position: fixed` overlay | Implementation lesson, not a design lesson. v4 is "v3 with a different render path." |

The through-line across all four: **the user wants a single image that shows the shape of their own thinking, anchored stably enough to learn — and then helps them improve it.**

v2's force-directed instability and v3/v4's per-mode-XYZ instability both broke the same invariant: spatial memory. The v5 reframe makes spatial memory the load-bearing constraint:

- **Strata is the constant radius across all modes.** Eisa, 2026-05-09: *"strata as the constant radius across all modes (only azimuth changes per mode)."* This revokes v3's per-mode XYZ grammar entirely.
- **Color stops carrying community membership.** v2/v3 used Louvain palette. v5 uses ink black for ordinary stars, red for contested. Color is a state cue, not a topology cue.
- **InfraNodus heritage is dropped from the visual.** Brandes betweenness, Louvain communities, modularity-dominance-entropy-connectivity scoring — all out of Sight v5's visual layer. The diagnostic instinct InfraNodus had (universe-health scoring) returns in Layer 2, but reframed around epistemic posture, not link-graph topology.
- **Sight v5 is the canonical, supported Sight.** The numbering ladder ends at v5; future evolution happens inside v5's grammar.

The four other foundational decisions Eisa ratified 2026-05-09, all binding on this paper:

1. **`lenses.rs::apply_lens` is deleted.** The withdrawn group-by mechanic is not part of v5; the cleanup MIG retires the dead Rust code.
2. **Sight is the whole universe; 360.3D is the single note.** Mutually exclusive scopes. Selecting a star in Sight hands off to the editor / 360.3D — Sight does not deepen into a per-note view.
3. **If a first-time user can't understand v5, v5 doesn't exist.** Eisa: *"If future Constellation users don't understand it or think it is difficult, then its existence is unnecessary."* This is the ~5-second comprehension threshold formalized as an existence condition for Layer 1.
4. **Sources are tracked Day 1, not deferred.** This made Sources a real subsystem (which became CECE), not a future PJ — and made the classifier infrastructure part of Sight v5's foundation.

---

## §4 · The scholarly foundation — Universal Epistemic Content Taxonomy + CECE's live state

The InfraNodus heritage Sight v5 leaves behind from the visual layer was a *single-tradition analytical scaffold* (network science, betweenness centrality, modularity scoring). v5 replaces it with a *cross-civilizational scholarly scaffold* — the **Universal Epistemic Content Taxonomy** Eisa drafted 2026-05-09 — which is then operationalized by CECE.

The taxonomy has two orthogonal axes, distilled from convergent structure across five major epistemological traditions plus four supplementary ones:

| Tradition | Key contribution |
|---|---|
| Greek + Western analytic (Plato, Aristotle, Stoics, Kant, Frege, Russell, Polanyi) | The JTB analysis of knowledge; *lekton* / propositional content; the Data-Information-Knowledge-Wisdom hierarchy |
| Sunni Islamic — *kalām* / *uṣūl al-fiqh* / *falsafa* (Al-Jurjānī, Al-Āmidī, Al-Ghazālī, Ibn Sīnā, Ibn Rushd) | The *taṣawwur* / *taṣdīq* binary; the graded epistemic-states scale (*shakk* → *ẓann* → *ʿilm* → *yaqīn*); the *masādir al-maʿrifah* including the distinctively Sunni *al-tawātur* |
| Indian *pramāṇa-vāda* (Nyāya, Mīmāṃsā, Vedānta, Buddhist Dignāga / Dharmakīrti, Jaina) | The six *pramāṇa*; the *prama* / *aprama* distinction |
| Classical Chinese (Mohist, Confucian, Daoist, Neo-Confucian) | The *míng-shí* (name-reality) correspondence; *zhī xíng hé yī* (unity of knowing and acting) |
| Persian-Islamic *Ishrāqī* (Suhrawardī) | *Al-ʿilm al-ḥuṣūlī* vs *al-ʿilm al-ḥuḍūrī* |

Plus supplementary input from Jewish (Maimonides), Tibetan Buddhist (Sa-paṇ), African (Wiredu, Oruka), and Mesoamerican (León-Portilla, Maffie) traditions.

### §4.1 The two axes — and what CECE has built on top

The taxonomy doc declares two axes:

- **Vertical axis** — five primary branches of epistemic content: (1) Sensory inputs · (2) Symbolic entities · (3) Semantic contents · (4) Epistemic states · (5) Higher-order constructs.
- **Horizontal axis** — eleven sources / means of knowledge: perception · inference · testimony · mass-transmission · comparison · postulation · non-apprehension · memory · innate disposition · inspiration · revelation.

A single epistemic item is located by both axes: *what kind of content is it* (vertical) × *what source produced it* (horizontal). "The cat is on the mat" is a **proposition** (vertical: §3.3) produced by **perception** (horizontal: §S1).

**What CECE made of this skeleton (MIG-021v3 cascade, 2026-05-10 → 2026-05-11):** the published 5-branch × 11-source skeleton became a **live classification taxonomy of ~280 nodes** carrying structured IDs like `testimony/authoritative`, `inference/qiyas`, `epistemic-states/doubt`, `higher-order-constructs/worldview`, etc. The live taxonomy lives in `src-tauri/src/sources/horizontal_taxonomy.rs` (~53 nodes) and `vertical_taxonomy.rs` (~224 nodes). All nodes carry bilingual EN+AR labels; 13 other locales got translated through V3-§10 / MIG-022 §E.3.d.

Sight v5's mode P visualizes whatever the live taxonomy carries — **not** the published 11-source skeleton. So the wedge count in mode P is not "11 fixed wedges" but "however many primary horizontal-axis IDs CECE is using as parents at the time the dome renders." Likely ~11 at the top level plus an "Unsourced" wedge, with the leaf IDs (`testimony/authoritative`, `inference/qiyas`, etc.) surfaced on hover or via a "drill into wedge" interaction.

### §4.2 Strata IS the Constellation projection of the vertical axis — anchored by what CECE actually classifies

Constellation already shipped an 8-level strata field (MIG-014, populated by the user across all 7,636 trial-universe notes). The 8 strata levels map cleanly onto the 5-branch vertical axis condensed by epistemic elevation:

| Stratum | Universal Taxonomy mapping (skeleton) | CECE live-taxonomy parent IDs (representative) |
|---|---|---|
| L1 Datum | Branch 1 (sensory inputs) + Branch 2.3 (data) | `sensory-inputs/*`, `symbolic-entities/data` |
| L2 Fact | Branch 3.5 (fact / *wāqiʿah*) | `semantic-contents/fact` |
| L3 Opinion | Branch 4.5 (epistemic state — *ẓann*) | `epistemic-states/opinion`, `epistemic-states/doubt` |
| L4 Hypothesis | Branch 5.1 (construct — *faraḍiyyah*) | `higher-order-constructs/hypothesis` |
| L5 Theory | Branch 5.2 (*naẓariyyah*) | `higher-order-constructs/theory` |
| L6 Framework | Branches 5.3 – 5.5 (model / law / doctrine) | `higher-order-constructs/{model,law,doctrine}` |
| L7 Perspective | Branches 5.6 – 5.7 (insight / wisdom — *baṣīrah / ḥikmah*) | `higher-order-constructs/{insight,wisdom}` |
| L8 Worldview | Branch 5.8 (*ruʾyah kawniyyah*) | `higher-order-constructs/worldview` |

This is doubly-justified design: strata-as-radius is justified by Constellation's native taxonomy (the user already labels every note with a stratum), by the cross-civilizational scholarly tradition the taxonomy synthesizes, **and by what CECE has actually proven classifiable on the trial Universe** (Boss-Test Gate 2 PASS, 2026-05-11 — 5 distinct vertical primaries fired correctly across 5 test notes, including `higher-order-constructs/worldview` for Arabic *رؤية كونية*).

The diagnostic layer (Layer 2) reads against this strata-mapped distribution: a universe that's 70 % Branch 1 and 5 % Branch 5 has a different epistemic posture than one that's 20 % Branch 1 and 40 % Branch 5. Both can be healthy depending on the user's intent; Sight surfaces the pattern, the user interprets.

### §4.3 What Sight v5 inherits from CECE

CECE's MIG-021v3 cascade closed 2026-05-11 with:

- The `note_meta.sources` column + frontmatter contract.
- The `sources_suggestions` table + Source Review panel (where the user accepts/edits/rejects).
- A 6-cataloger ensemble that proposes both axes for any note (User-Authority, Structural, Linguistic, Graph, Semantic, Reasoning — the last currently abstaining pending llama.cpp wiring).
- Three confidence regimes (Unanimous / Strong-Majority / Split with Sibling Disambiguation).
- Per-Library reliability tracking that gets smarter as the user accepts/rejects.
- 15-locale i18n for every source label, every cataloger label, every confidence regime, every reasoning template — the Source Review card is fully localized.
- The bundled e5-small embedding model (Tier 1 classifier, ~113 MB, Day 1).
- The Qwen3-1.7B Q4_K_M GGUF download path (Tier 2 classifier, ~1.1 GB, opt-in via Settings → AI). Wiring deferred to V3-§7.b.

So the Sources data Sight v5 visualizes (mode P) is **already real and accumulating** — Eisa is using the Source Review workflow on the trial Universe. By the time MIG-024 (Sight v5 visual foundation) ships, mode P will have a meaningful Sources distribution to render.

Sight v5's Layer 4 (coaching) leans on the same llama.cpp + Qwen3-1.7B inference path CECE pioneered. No new model bundle needed — Sight v5 reuses CECE's Tier 2 stack for its own analytical inference.

---

## §5 · The visual grammar (Layer 1 foundation)

Sight v5's visual layer is a circular star chart on Suwaidi cream parchment. The Mock B1 reference (approved by Eisa 2026-05-09) is the binding visual contract for everything below the toggle bar; the toggle bar itself bumps from 6 to 7 buttons in production (see §6).

### §5.1 The Mock B1 visual contract

![Sight v5 Mock B1 — the approved visual reference](Sight-vNext-MockB1-Toggle.svg)

*Mock B1 — file: `docs/Sight-vNext-MockB1-Toggle.svg` (open in any modern browser to view at full resolution). Active mode shown is Time; the toggle bar above the dome shows R · L · T · C · S · A; the right-hand legend names the perceptual encodings in plain language. The legend's punchline is the load-bearing rule: "**THE ONLY THING A MODE SWITCH CHANGES is what the rim wedges divide on. Strata, sizes, brightness, links, wash — all stay the same.**"*

**Pending edit to the Mock B1 SVG (per §6):** add a 7th button "P" at the end of the toggle row, styled with the dimmed dashed-border treatment used for C/S/A in the current SVG. This edit lands as a §0 housekeeping step within MIG-024 (visual-foundation Architect doc).

This mockup is the visual spec for Layer 1. Production code reconciles against it pixel-for-pixel within the Suwaidi palette tokens defined in the SVG `<style>` block.

### §5.2 The dome

A circular field on Suwaidi cream parchment (`#faf6e8` background). Eight concentric strata bands divide the dome from the central pole (L8 Worldview) to the rim (L1 Datum). Faint sand grid lines (`#b8a98a`) mark each band boundary at low opacity (~0.5–0.55).

A 12-month calendar rim wraps the outside of the dome. Always present; serves the Time mode and provides a stable temporal reference in every other mode.

A soft Milky Way wash drifts across the chart in two diffuse ellipses (`#e6dec0` radial gradient, ~0.55 alpha at center fading to 0 at edges) — content-similarity density, the visual texture of *related themes that aren't explicitly linked*.

### §5.3 The stars

Each note is one circular dot — a star. Position, size, brightness, and color encode four orthogonal properties that **never change with mode**:

| Encoding | Property | Levels |
|---|---|---|
| **Radial position** (center → rim) | Strata | L8 pole → L1 rim. 8 bands. Never changes across modes. |
| **Angular position** (around the rim) | Active mode's wedge basis | Mode-dependent (see §6). The only thing a mode switch changes. |
| **Size** | Maturity | seed (1.5 px) → sapling (2.5) → evergreen (3.5) → canonical (5) → wilting (2, greyed) |
| **Brightness** (alpha) | Confidence | hypothesis (0.45) → evidence (0.7) → established (1.0) |
| **Color** | State | Ink (`#1a1a1a`) by default. Red (`#a83232`) for notes whose primary link confidence is *contested*. |

Library color is **not** used for stars. That's the Constellation Map's vocabulary. Sight uses strata-band rings as its grouping signal, not per-star color.

### §5.4 The connector lines

Faint at rest (~0.10–0.15 alpha), color-coded by the **9 typed-link kinds** (post-MIG-022 §A.2 — `supersedes` is the most recent addition):

| Color | Hex | Link kinds |
|---|---|---|
| Green | `#3a8a4a` | *supports* / *derives-from* |
| Red | `#a83232` | *contradicts* |
| Gold | `#c9a227` | *exemplifies* / *generalizes* |
| Blue ink | `#2a4a8c` | *causes* / *part-of* |
| **Slate blue-gray** | `#5B7A8A` | ***supersedes*** (MIG-022 §A.2, 2026-05-12 — the "this replaces an older claim" link) |
| Cool grey | (default associative) | *associative* (untyped wikilinks fall here) |

User-overrideable per Universe via Settings → Link pills color picker (MIG-006 / MIG-007 territory).

On hover or select, the focused star's incident edges brighten to ~0.85 alpha; other edges stay faint. **Principle 6 (reveal-on-demand) reframed:** *reveal* now means *brighten*, not *render-from-zero*. The structural pattern of the universe is always visible at rest; focus simply highlights what the user is looking at.

### §5.5 The toggle bar (production = 7 buttons)

A row of buttons at the top of the dome, naming the seven modes from Day 1: **R · L · T · C · S · A · P**. Letter-buttons, ~50 × 44 px, 10 px gap, centered above the dome.

- **Active mode**: gold-filled background (`#c9a227`), parchment letter (`#faf6e8`), 600 weight.
- **Ready but inactive**: cream background (`#fbf8ec`), near-black letter (`#1a1a1a`), 1 px solid border.
- **Available later** (data not yet populated): cream background, faded letter (45 % opacity), dashed sand border (`#b8a98a`, dasharray `3 2`). Hover-tooltip explains what unlocks the mode (e.g., for P: *"Available once you've classified some notes — see Source Review in the right sidebar"*).

A small caption under the bar names the active mode in the user's interface locale.

### §5.6 What is NEVER shown in the chrome

The visual chrome is plain language only. Sight v5 never surfaces:

- **Civilizational labels** ("Branch 2 Symbolic Entities", "*pramāṇa*", "*kalām*"). The taxonomy is the scholarly foundation; the UI uses Constellation-native vocabulary.
- **Network-science terminology** ("betweenness centrality", "Louvain modularity"). These are the InfraNodus heritage Sight v5 is leaving behind.
- **Numerical scores out of 100.** Layer 2 surfaces health findings as plain-language signals; if the user wants a number, it appears in a hover-tooltip badge, never as a chrome element.

---

## §6 · The seven modes

Each mode declares its own **azimuth** (rim wedge slicing). Strata stays the radius. Star size, brightness, color, and link colors stay constant. The transition animation between modes (~600 ms ease) interpolates only the angular position of each star — stars slide tangentially around their stratum ring as the wedges re-cut.

| ID | Mode | Wedge basis | The cognitive question | Data source | Status |
|---|---|---|---|---|---|
| **R** | Regions | Library (sized by note count, biggest first) | "Where in my cosmos does this idea live?" | Library membership | Ready (existing) |
| **L** | Link Types | Dominant outgoing link type (9 typed-link kinds + Untyped) | "What kind of reasoning, and how versatile?" | `note_links.link_type` | Ready (existing — supersedes added MIG-022) |
| **T** | Time | Creation month (12 wedges; current month subtly highlighted) | "When did it emerge, and is it still alive?" | `note_meta.created` | Ready (existing) |
| **C** | Confidence | Dominant per-note link confidence (4 wedges: hypothesis · evidence · established · contested) | "How certain, and how consistent?" | `note_links.confidence` | Ready (existing) |
| **S** | Stages | Dominant lifecycle stage (6 wedges: Spark → Birth → Growth → Maturity → Dormancy → Archival) | "How alive, and how worn the path?" | `note_meta.stage` | Ready (MIG-014) |
| **A** | Acts | Which Act produced the note (5 wedges: Observation → Connection → Tension → Synthesis → Conviction) | "Where in the formulation arc?" | per-note act tag | Partial (CE Layer 2) |
| **P** | Provenance | Primary horizontal-axis ID from CECE's live taxonomy (~11 top-level wedges + Unsourced) | "What kind of knowing produced this?" | `note_meta.sources` | Ready (MIG-021v3 — populating incrementally via Source Review) |

When a mode's data is partially populated, Sight v5 renders what's available and shows an "Unsourced" / "Unstaged" / "Unacted" wedge for the missing slice — the visible wedge becomes a to-do list. **This is also the input Layer 2 reads against** to surface the diagnostic finding "your Unsourced wedge is the largest in mode P; consider running a classification pass."

A universe whose Unsourced wedge dominates is a universe whose epistemic provenance has not been examined yet; the visual itself is the prompt, and the diagnostic seconds it.

### §6.1 Mode persistence

Last-used mode persists per Universe via `appSettings.sight.lastMode`. Default for first-time use: **R** (Regions) — the lowest-cognitive-load mode, since "which library carries which strata" is the most familiar cut for a new user. If the saved mode is unavailable (P before any sources are assigned, A before any acts are tagged), fall back to R.

### §6.2 Why no per-mode (X, Y, Z) grammar

The v3 visual spec had each mode declaring its own (X = azimuth, Y = radius, Z = magnitude). v5 revokes this. Four reasons:

1. **Spatial memory survives mode switches.** The user learns the dome once; mode toggles re-aim the lens through the same sky. If the same star sits at L7 in Regions and L7 in Confidence, the user remembers where it lives.
2. **The cognitive question maps cleanly to the visual axes.** *Shape* lives on the radius (constant strata); *organization* lives on the angular axis (mode-dependent wedges). One image, two readings.
3. **Cross-surface coherence with 360.3D.** 360.3D's Stratification Matrix anchors the per-note view to strata. Sight anchoring the per-universe view to strata makes the two surfaces echo — same axis, different scope.
4. **The InfraNodus-derived (X = library, Y = centrality rank) Regions-mode anchor doesn't apply** with InfraNodus dropped. Centrality is no longer the radius. The natural successor is constant-strata-radius.

The cost of revoking per-mode XYZ: stars don't fly across the sky between modes (the v3 "diagnostic migration trajectory" is gone). A star at L7 in May stays at L7 in Research. That cost is acceptable — diagnostic patterns appear in the *wedge weights* and in **Layer 2's plain-language signals**, not in star migration.

---

## §7 · The four constants

These are the load-bearing visual invariants. Every mode honors them. Breaking any of them is a P0 regression that fails the ~5-second comprehension threshold for Layer 1.

| Constant | Encoded property | Source data |
|---|---|---|
| **Radial position** | Strata (L8 pole → L1 rim) | `note_meta.stratum` |
| **Size** | Maturity (5 sizes from seed to canonical) | `note_meta.maturity` |
| **Brightness** | Confidence (3 alpha levels from hypothesis to established) | derived from `note_links.confidence` (per-note primary) |
| **Color** | State (ink for ordinary; red for contested) | `note_links.link_type` + `confidence` (any inbound `contradicts` link with non-archived status → contested) |

The Mock B1 legend names all four constants with their semantic meaning. The user reads the legend once and never has to learn it again because the encoding never changes.

---

## §8 · The Sources subsystem (CECE foundation for mode P + Layers 2–4)

The Sources axis is the dimension Sight v5 introduces beyond what was tracked pre-MIG-021. It is the horizontal axis of the Universal Epistemic Content Taxonomy lifted into Constellation as a per-note property and operationalized by CECE.

### §8.1 The 11 source families (top-level horizontal-axis branches)

| # | English | Arabic | Sanskrit | Definition |
|---|---|---|---|---|
| 1 | Perception | الحِسّ | *pratyakṣa* | Direct sensory contact with an object |
| 2 | Inference | العَقل | *anumāna* | Derivation of conclusion from premises |
| 3 | Testimony | الخَبَر | *śabda* | Reliable verbal report from another knower |
| 4 | Mass-transmission | التَّواتُر | — | Convergent reports too numerous to collude on falsehood |
| 5 | Comparison | القياس | *upamāna* | Knowledge of an object via similarity to a known object |
| 6 | Postulation / IBE | الاستنباط الافتراضي | *arthāpatti* | Inference of an unobserved fact required to explain an observed one |
| 7 | Non-apprehension | عَدَم الإدراك | *anupalabdhi* | Knowledge of absence via the absence of perception |
| 8 | Memory | الذاكرة | *smṛti* | Recall of previously cognized content |
| 9 | Innate disposition | الفِطرة | — | Pre-experiential cognitive endowment |
| 10 | Inspiration | الإلهام | — | Non-discursive apprehension in mystical traditions |
| 11 | Revelation | الوحي | — | Communication from a divine source |

Each top-level family expands into leaf IDs in CECE's live taxonomy (`testimony/authoritative`, `inference/qiyas`, `inference/arthapatti`, `revelation/quranic`, etc.). The leaf set evolves as CECE matures; Sight v5's mode P top-level wedges count = top-level family count, with leaves visible on hover/drill-in.

All 11 ship Day 1. Per the taxonomy's own pluralism on contested categories, Constellation does not editorialize: a journalist logging a quoted source picks *testimony*; a *Hadith* scholar picks *mass-transmission*; a poet logging a vision picks *inspiration*; a philosopher logging a deduction picks *inference*. Constellation provides the vocabulary and lets the user choose.

### §8.2 Per-note storage

**Frontmatter** (canonical, source of truth):

```yaml
sources:
  - testimony
  - mass-transmission
```

The list is **ordered**: first = primary source, subsequent = secondary. A note may have any number of sources from 0 to 11. An empty/missing field means *unsourced*.

**SQLite mirror** (`note_meta.sources`, JSON-encoded list, fast-read index). Updated by the existing write-time `scan_note_*` pipeline that already maintains `stratum`, `maturity`, `stage`. Mirror policy matches MIG-014: frontmatter wins on disagreement; SQLite is the read cache. Write-time derivation per CLAUDE.md Rule 8.

### §8.3 Setting sources — three paths

Per Eisa's six Sources sub-decisions (2026-05-09):

1. **PropertyEditor combobox** — multi-select dropdown listing all 11 sources in the user's interface locale. Matches the existing Strata / Maturity / Stage controls exactly.
2. **Source Review sidebar panel** — queue surface where CECE presents its suggestions. User can Accept, Edit, or Reject. Mirrors the Review Pulse panel pattern.
3. **Right-click → "Suggest sources for this note"** — context menu on any note (file tree, Sky View, Sight, anywhere a note is selectable). Triggers an on-demand single-note classification.

### §8.4 The "Unsourced" wedge in mode P

Notes whose `sources:` field is empty render in a dedicated **Unsourced** wedge in mode P. This wedge is the visible to-do list — its size shrinks as the user (with CECE's help) classifies more of the universe. **Layer 2's diagnostic** reads against this wedge to surface "Your epistemic provenance is mostly unexamined — N notes are unsourced" as a plain-language signal.

---

## §9 · CECE — the engine Sight v5's Layers 2–4 build on

The Constellation Epistemic Content Engine (CECE) is the subsystem Sight v5 inherits from MIG-021v3 (closed 2026-05-11). CECE classifies notes; Sight v5 visualizes the classifications (Layer 1) and reasons about them (Layers 2–4).

### §9.1 Six-cataloger ensemble (already shipped)

CECE classifies a note through six methodologically distinct lenses, each producing its own reasoning trail:

| Cataloger | Lens | Cost tier |
|---|---|---|
| User-Authority | Frontmatter the user already wrote | Cheap (500 ms) |
| Structural | Citations + structural patterns (DOI/ISBN, blockquote+attribution, code blocks) | Cheap (500 ms) |
| Linguistic | CAE morphology + lexicon match + Bridge slow-path embedding for Arabic | Medium (2 s) |
| Graph | Living Links typed-neighbor consensus | Medium (2 s) |
| Semantic | Per-Library kNN-blend embedding similarity | Medium (2 s) |
| Reasoning | Local LLM (Qwen3-1.7B) with GBNF grammar — abstains today; wires up in V3-§7.b | Expensive (5 s) |

A **synthesis layer** combines the six per-cataloger trails into one of three confidence regimes per axis (horizontal = Source, vertical = Content Type):

- **Unanimous** — all voiced catalogers agree. Single primary, no disambiguation needed.
- **Strong-majority** — clear winner with named dissenter. Single primary; trail surfaces who disagreed and why.
- **Split** — no clear winner. Engine refuses to assign and asks the user via Sibling Disambiguation chips.

### §9.2 LLM stack (the local-AI substrate for Layers 2–4)

Per Eisa's 2026-05-09 picks (research summary `lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md`):

- **Bundled "starter" classifier**: reuse the existing `multilingual-e5-small` ONNX model (~113 MB, already shipping for semantic search). Embedding-similarity classification — Tier 1.
- **Optional larger classifier**: **Qwen3-1.7B Q4_K_M GGUF** (~1.1 GB), downloadable via Settings → AI. Apache 2.0, first-class Arabic, 25–45 tok/s on CPU. Tier 2.
- **Inference engine**: **llama.cpp** via the `llama-cpp-2` Rust crate. The killer feature is GBNF grammar-constrained decoding, which guarantees valid JSON output for the 11-source classification AND for the structured recommendations Layer 3 will emit.
- **Bundling strategy**: e5-small bundled in the `.exe` so Sight v5 mode P + Layer 2's basic diagnostic work Day 1 with no network. Qwen3-1.7B is opt-in; required for Layer 4 (coaching).

Per Eisa's amendment 2026-05-10: **all CECE inference is local-only.** No cloud inference path. Sight v5's "local AI" framing inherits this guarantee directly: **the coaching is private the way a private tutor is private — only the user and the instrument know what was discussed.**

### §9.3 What's already running on Eisa's universe

As of 2026-05-12: CECE has classified ~270 cards on the trial Universe; the Source Review panel surfaces them with per-cataloger trails, queue composition filters, Sibling Disambiguation, Approve All / Reject All, and full 15-locale i18n. The data Sight v5 will eventually visualize in mode P is already accumulating. Layers 2–4 will read against this same data plus the strata + maturity + stage + confidence + acts fields the rest of Constellation already populates.

### §9.4 What Layers 2–4 add to CECE

CECE is *per-note* classification. Sight v5's analytical layers are *whole-universe reasoning* on top of CECE's outputs:

- **Layer 2** runs aggregations over CECE's outputs (e.g., "what fraction of the universe is `testimony/*`?") plus aggregations over the existing strata / maturity / stage / confidence fields. Outputs plain-language signals.
- **Layer 3** uses Qwen3-1.7B with a different system prompt + GBNF grammar to convert Layer 2's signals into specific named recommendations ("47 stalled hypotheses; here's the 3 most actionable").
- **Layer 4** uses Qwen3-1.7B in conversational mode to walk the user through executing a recommendation. State machine on top of llama.cpp.

---

## §10 · What Sight v5 IS NOT

The 360.3D Concept Paper enumerates "what 360.3D is NOT vs Sky View, Map, Sight, Index, OrgChart." That section is load-bearing — the boundary that prevents accidental duplication. Sight's prior papers never wrote the reciprocal section. v5 writes it.

| Adjacent surface | What it answers | Why Sight v5 is not it |
|---|---|---|
| **Sky View** | "What does my note network *feel* like, alive?" | Sky View is a force-directed PIXI bubble graph showing the live nervous-system topology. Sight v5 is a stable star chart anchored by strata. Different visual grammar, different question. Sky View has bubbles; Sight has stars-by-strata. |
| **Constellation Map** | "What is the shape and density of my libraries?" | The Map is a D3 sunburst tracking the file-tree hierarchy. Sight v5 is a sky chart tracking epistemic content distribution. Different organizing principle (hierarchy vs strata). The Map has sunburst arcs; Sight has stars. |
| **OrgChart** | "What is my Universe's organizational hierarchy?" | OrgChart is the connector-box tree of structural containment. Sight doesn't show containment; it shows distribution. |
| **Search Hub** | "Where in my system does this term/link/concept exist?" | Search is point-query against a corpus. Sight is whole-universe distribution + diagnostic + coaching. Zero use-case overlap. |
| **Index Panel** | "Which terms appear in my notes and where?" | Index is term-level vocabulary browsing (built on FTS5). Sight is note-level epistemic distribution. |
| **360.3D / Inspector 360** | "Where does THIS note stand? (Position / Profile / Absence)" | **Eisa's load-bearing line, 2026-05-09**: 360.3D = single note; Sight = whole universe. Mutually exclusive scopes. Selecting a note in Sight should hand off to 360.3D / the editor — not deepen Sight's own per-note view. |
| **Knowledge Health Dashboard (KHD)** | "What is the health of my **link-graph** — modularity, decay, hub patterns, weak foundations, bias alerts?" | KHD = link-graph health diagnostics (the InfraNodus-style metrics). Sight v5 = **epistemic-posture** health diagnostics (strata distribution, source diversity, confidence balance, growth trajectory). Two different "health" framings. KHD reads the graph topology; Sight reads the user's *thinking shape*. They complement; they don't overlap. |
| **Source Review panel** (CECE) | "Which source-classification suggestions need my approval?" | Source Review is the queue surface for accepting/editing/rejecting one note's sources. Sight visualizes the *aggregate result* across the universe and reasons about it. |
| **Multi-Lens** (`lenses.rs::apply_lens`) | (was: "group notes by tag / property") | WITHDRAWN 2026-05-09. `apply_lens` queued for deletion. The "group-by" job is reframed as Sight's multi-mode wedges. |

The five-core-functions invariant from 2026-04-13 still holds: **Search Hub · OrgChart · Sky View · Map · Sight** are the five non-overlapping cognitive surfaces.

---

## §11 · Performance budgets

Per CLAUDE.md Performance Rules + the 2026-04-15 boot-perf discipline. Each layer carries its own budget; lower layers must hold even when upper layers are running.

### §11.1 Layer 1 — visual foundation (per-frame budgets)

| Metric | Budget | Notes |
|---|---|---|
| First-toggle latency on 7,636-note universe (cold) | ≤ 500 ms | Layout cache miss; rebuild + draw |
| First-toggle latency (warm SQLite cache) | ≤ 50 ms | Layout cached; draw only |
| Mode-switch animation (R ↔ L ↔ T ↔ ...) | 600 ms ease | Pure JS re-projection, no IPC |
| Hover-star highlight | ≤ 16 ms (single frame) | Decoration on focus overlay only |
| Idle-Sight per-frame cost (no hover, no select) | ≤ 1 ms | Static base layer; no redraw |
| Memory footprint (Sight open, 7,636 notes, all 4 layers loaded) | ≤ 60 MB above app baseline | Two Canvas 2D layers + DOM overlays + Layer 2/3 cached aggregates |
| Boot impact | **Zero** | Sight is lazy-mounted on dock-button click; layout cache warmed via `requestIdleCallback` after `boot:hydrated` |

### §11.2 Layer 2 — diagnostic (background recompute)

| Metric | Budget | Notes |
|---|---|---|
| Diagnostic recompute on universe change | ≤ 2 s | Background thread; results cached; doesn't block UI |
| Diagnostic-card render | ≤ 100 ms | Read-from-cache; plain DOM |

### §11.3 Layer 3 — recommendation (on-demand LLM)

| Metric | Budget | Notes |
|---|---|---|
| Recommendation generation (Qwen3-1.7B) | ≤ 10 s | User-initiated; loading state shown |
| Recommendation cache hit | ≤ 50 ms | Cache key = (universe-snapshot-hash, diagnostic-finding-id) |

### §11.4 Layer 4 — coaching (interactive LLM)

| Metric | Budget | Notes |
|---|---|---|
| First coaching-turn response (Qwen3-1.7B, ~200 token output) | ≤ 8 s on 4-core CPU | Streaming; user sees tokens land |
| Subsequent turns | ≤ 4 s | Context already loaded into KV cache |

CECE has its own budgets per the MIG-021v3 close-out. Sight v5's Layer 4 reuses CECE's llama.cpp infrastructure — no separate model load.

---

## §12 · Phased rollout

Sight v5 ships across four layered MIGs (one per layer), plus a cleanup MIG that retires v4 once v5 is Eisa-confirmed-stable. Each MIG ends with an Eisa-test gate.

> **MIG number-collision resolution** (Eisa: *"Your call."*):
> - The gap-analysis-response cascade (PJ-040 / 041 / 042 / 043 + history-axis Rust foundation) **keeps the MIG-022 number** it already shipped under (§0 + §D + §E + §A complete; §B in progress).
> - The Warrant Research workstream **keeps the MIG-023 reservation** (Eisa-committed 2026-05-11).
> - Sight v5 visual foundation **takes MIG-024** as the first free number.
> - Subsequent Sight v5 layers continue numerically.

| MIG | Layer | Deliverable | Eisa-test gate |
|---|---|---|---|
| **MIG-024** | Layer 1 — visual foundation | The dome, the eight strata bands, the calendar rim, the 7-button toggle bar with R + T modes initially active (the two whose data is fully populated and least controversial), the four constant encodings, the connector-line layer with hover/select brightening, the side panel for selected-star detail. Mock B1 SVG updated to show 7 buttons. | Open Sight v5; dome renders correctly on the trial Universe; mode toggle + animation work; hover / select / Esc; close button works (mount inside `.content-area` per SkyView pattern). |
| **MIG-025** | Layer 2 — diagnostic | Background diagnostic engine: stratum-distribution, source-diversity, confidence-balance, growth-trajectory, contested-resolution, acts-coverage health computations. Plain-language signals surfaced in a "Findings" side panel. No LLM yet — pure aggregation + threshold rules. | Open Sight v5 on the trial Universe; verify findings card surfaces realistic signals; verify findings update after a note edit. |
| **MIG-026** | Layer 3 — recommendation | Qwen3-1.7B (Tier 2) wired through llama.cpp (V3-§7.b lands here as a sub-phase). For each Layer 2 finding, a "Recommend" button generates a structured recommendation via GBNF grammar (named notes / actions / rationale). | Open Sight; click Recommend on a finding; verify the generated recommendation names real notes and is actionable. |
| **MIG-027** | Layer 4 — coaching | Conversational coaching mode on top of Qwen3-1.7B. A "Help me handle this" button on a recommendation opens a side-panel chat that walks the user through executing the recommendation, with Constellation-aware actions (open note, create link, propose stratum promotion). | Open Sight; pick a recommendation; coach me through it; verify actions land correctly in the Library. |
| **Cleanup MIG** (TBD number) | — | Delete `lenses.rs::apply_lens` (CE Phase 9 withdrawn), the orphaned `constellation_sight_*` IPCs in old `sight.rs`, the v2/v3/v4 Sight Svelte components (after Eisa confirms v5 stable across multiple sessions), move obsolete v1.x / v2.0 / v3-paper files to `docs/historical/`. | Verify v4 retired; verify no Sight surface other than v5 reachable from the dock. |

---

## §13 · Acceptance criteria

For Sight v5 to close as Done, all four layers must satisfy their criteria:

### Layer 1 — visual foundation
1. The five-core-functions invariant holds: Sight v5 does not duplicate Search Hub, OrgChart, Sky View, Map, or any adjacent surface (per §10).
2. **A first-time user with no Constellation training opens Sight v5 and can articulate within ~5 seconds what they're looking at.** Load-bearing existence-condition from Eisa's 2026-05-09 directive.
3. All seven modes render correctly on the 7,636-note trial Universe within Layer 1 performance budgets (§11.1).
4. Mode toggle preserves spatial memory: the same star sits at the same strata band in every mode.
5. The four constants (radial position / size / brightness / color) hold across every mode without exception.

### Layer 2 — diagnostic
6. Findings card surfaces realistic signals on the trial Universe (Eisa-validated against his sense of his own universe).
7. Diagnostic budgets met (§11.2).
8. Findings update incrementally on note edits without UI stutter.

### Layer 3 — recommendation
9. Recommendations name real notes and are actionable.
10. GBNF-constrained output is always valid (zero parse failures).
11. Recommendation budgets met (§11.3).

### Layer 4 — coaching
12. Coaching session can guide the user through at least one full enhancement workflow end-to-end (e.g., promote a hypothesis from L4 to L5 by gathering evidence links).
13. Coaching output is grounded in the user's actual notes (no hallucinated note titles).
14. Coaching budgets met (§11.4).
15. **Privacy guarantee verified**: zero outbound network calls during any Layer 4 session.

### Cross-layer
16. Sources field populated for ≥ 80 % of the trial Universe via the CECE-and-approval workflow (already underway).
17. Both Tier 1 (bundled e5-small) and Tier 2 (optional Qwen3-1.7B) inference paths work end-to-end.
18. Help docs + User Manual (EN + AR canonical, 13 other locales queued) describe Sight v5 as it ships.
19. Three-agent integration audit clean across all four Sight-v5 MIGs.
20. Eisa confirms across multiple sessions that Sight v5 delivers the at-a-glance promise (Layer 1) AND the local-AI promise (Layers 2–4).

---

## §14 · Glossary

| Term | Definition |
|---|---|
| **Sight v5** | The fifth Sight implementation generation. Star-chart aesthetic, taxonomy-spined, seven modes, strata-as-radius, four-layer analytical instrument. The canonical Sight target. |
| **The four layers** | (1) Visual foundation; (2) Diagnostic; (3) Recommendation; (4) Coaching. Built in order; each holds its own budget while lower layers continue to honor theirs. |
| **Local AI** | Eisa's framing for Sight v5's analytical / coaching ambition. All inference runs on the user's device via CECE's existing infrastructure (e5-small ONNX + Qwen3-1.7B GGUF via llama.cpp). Zero cloud dependency. |
| **Star** | A note rendered as a circular dot on the dome. |
| **Strata band** | A concentric ring on the dome corresponding to one of the eight stratum levels (L1 Datum at rim, L8 Worldview at pole). |
| **Mode** | A wedge-slicing scheme that reorganizes the rim. Seven modes: R / L / T / C / S / A / P. |
| **Wedge** | A radial sector of the dome corresponding to one bucket of the active mode. |
| **The four constants** | Radial position (strata), size (maturity), brightness (confidence), color (state). The encodings that never change with mode. |
| **The ~5-second rule** | A first-time user articulates what they're looking at within ~5 seconds. Layer 1 acceptance criterion. |
| **Universal Epistemic Content Taxonomy** | The cross-civilizational scholarly framework Sight v5 uses as backend vocabulary. Five branches × eleven sources skeleton. |
| **CECE** | Constellation Epistemic Content Engine. The 6-cataloger ensemble (~280-node live taxonomy) that proposes source assignments. Shipped MIG-021v3 (2026-05-11). The substrate Sight v5's Layers 2–4 build on. |
| **Tier 1 / Tier 2 classifier** | Bundled (e5-small embedding similarity) vs optional-download (Qwen3-1.7B with GBNF grammar) inference paths. Tier 2 is required for Layer 4 coaching. |
| **Source Review panel** | The sidebar surface where the user approves CECE suggestions. Lives in the right sidebar; complements Sight v5 but is not part of it. |
| **Findings card** | The Layer 2 surface — plain-language signals about the user's epistemic posture. Lives in Sight v5's side panel. |
| **Mock B1** | The visual reference Eisa approved 2026-05-09. File: `docs/Sight-vNext-MockB1-Toggle.svg`. The binding visual contract for Layer 1 (with §6's 7-button toggle update pending). |
| **Health (Sight v5)** | Epistemic-posture health: stratum distribution, source diversity, confidence balance, growth trajectory, contested resolution, acts coverage. NOT link-graph health (KHD's job). |

---

## §15 · Cross-references

- `docs/Sight-vNext-MockB1-Toggle.svg` — **the approved visual reference** (Mock B1). Pending §6 edit: add 7th button "P" with dimmed dashed-border style.
- `docs/Sight-vNext-MockB2-Compare.svg` — two-dome compare view (help-doc teaching diagram only — NOT production UX).
- `docs/Sight-vNext-MockA-Dashboard.svg` — alternative dashboard mock (rejected; kept as historical record).
- `docs/epistemic-content-taxonomy.md` — formal two-axis taxonomy skeleton, bilingual EN/AR.
- `docs/epistemic-content-EN.md` — comparative civilizational essay, English (the intellectual case).
- `docs/epistemic-content-AR.md` — Arabic version.
- `docs/epistemic-content-taxonomy-chart.html` — interactive 5-level chart, self-contained, bilingual.
- `src-tauri/src/sources/horizontal_taxonomy.rs` — CECE's live horizontal taxonomy (~53 nodes).
- `src-tauri/src/sources/vertical_taxonomy.rs` — CECE's live vertical taxonomy (~224 nodes).
- `docs/360.3D-Concept-Paper-v1.0.md` — the per-note diagnostic surface; explicit "what is NOT" boundary partner.
- `lab/reports/MIG-021-LOCAL-LLM-RESEARCH.md` — the LLM research that informed §9.
- **Obsoleted by this paper, preserved as historical record:**
  - `docs/Constellation-Sight-Concept-Paper-v3.0.md` — the prior synthesis pass (Eisa's review surfaced the structural corrections in v3.1).
  - `docs/Constellation-Sight-Concept-Paper-v2.0.md` — the first Sight v5 spec (drafted same session as the v.5 naming lock).
  - `docs/Constellation-Sight-Concept-Paper-v1.1.md` — InfraNodus-spined analytical foundation.
  - `docs/Constellation-Sight-v3-Concept-Paper-v1.0.md` and `v1.1.md` — per-mode (X, Y, Z) grammar.
  - `docs/SIGHT-V3-VISUAL-SPEC.md` — codification of the per-mode grammar.

---

**End of v3.1.** This paper is the design contract for Sight v5, awaiting Eisa's validation and approval. Once approved, the next document in the chain is the **MIG-024 visual-foundation Architect doc** — the first of four MIG cycles (Layers 1 → 2 → 3 → 4) that build Sight v5 progressively into a full local-AI analytical instrument.
