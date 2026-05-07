# Constellation Sight v3 — Concept Paper

**Version 1.0 (initial draft) | 2026-05-07**
**Origin**: PJ-038 (multi-MIG). Companion paper to `Constellation-Sight-Concept-Paper-v1.1.md` (the *analytical foundation* — what Sight is for). This v3 paper is the *visual + interaction specification* — what v3 looks like and how it is built.
**Author**: Eisa Alshamsi · eisa@uconstellation.world · drafted with Claude.
**Status**: Awaiting Eisa's design approval before any code is written.

> **Read this paper alongside the v1.1 paper.** The v1.1 paper covers the analytical core (centrality, communities, structural gaps, universe health, three edge types, six core mechanics, five design principles, Principle 6 reveal-on-demand). All of that is **inherited** by v3 unchanged. This paper covers what's new: the visual grammar, the projection math, the interaction model, the absorption of PJ-035 / PJ-036 / PJ-037, and the phased rollout.

---

## §0 · Why a v3 (one paragraph)

v2 Sight delivered ~70-80% of the v1.1 paper's analytical promise (centrality, communities, structural gaps, universe health all real and running on the Boss's universe). What v2 *did not* deliver was the user-facing payoff Eisa articulated 2026-05-07:

> *"To deliver the Sight promise, the UI should be 2D to begin with. The user should identify what Sight claims to deliver with one look."*

Force-directed graph layouts (v2's choice) re-run a physics simulation each session — the user can never build a stable spatial mental map. Star charts are different: once the projection is computed, the same notes always sit in the same regions of the dome. **Spatial memory becomes a feature.** v3 trades the force-directed grammar for an astronomy-inspired one. The math underneath is the same; the perception is fundamentally changed.

---

## §1 · The vision in one image

The reference image is a 19th-century-style printed star chart of the northern hemisphere — a circular field of stars on deep navy ground, constellations outlined with faint connector lines, the Milky Way drawn as a softer band of density across the chart, mythological figures sketched over the major constellations, and a calendar rim around the perimeter showing months and right-ascension hours.

The intuition the user reaches with one look:

- "Here's the *shape* of my entire knowledge base."
- "These big bright stars are my most important notes — the bridges."
- "These regions are different topics — knowledge constellations."
- "This Milky Way wash is the related-but-not-explicitly-linked density — shared themes."
- "These dark patches between constellations are my structural gaps — missing connections."
- "The rim shows me time — what was I writing about last spring."
- "When I focus on a constellation, its connector lines appear — Principle 6 in visual form."

This paper specifies how to translate that intuition into running code.

---

## §2 · Visual grammar — element-by-element semantic

| Star-chart element | Sight semantic | Source data | Visual treatment |
|---|---|---|---|
| **Star (point)** | Note | One per note in the universe | Filled circle in white/cream; size = centrality; brightness = lifecycle freshness (recently-traversed = brighter, dormant = dimmer) |
| **Star magnitude** (size + brightness) | Betweenness centrality | `constellation_sight_centrality` IPC | Logarithmic size scale, ~6 magnitudes (brightest = top 1% bridge notes; faintest = degree-1 leaves) |
| **Constellation territory** (bordered region) | Louvain community | `constellation_sight_communities` IPC | Soft-filled polygonal region in community color (low alpha); border drawn at the convex hull or alpha-shape boundary |
| **Constellation lines** (connector lines between stars) | Wikilinks + shared-tag edges *within* a community | Sight graph build (existing) | Faint white lines, **rendered only when the user focuses a constellation** — Principle 6 made visual |
| **Milky Way band** (diffuse density wash) | Content-similarity density (PJ-035) | TF-IDF cosine similarity field | Continuous density gradient; high-similarity regions form the band texture |
| **Constellation label** (mythological name) | Top-3 representative terms per community | Existing `community-profiles` machinery | RTL-aware text placed at territory centroid, sized by community population |
| **Calendar rim** (months / RA-hour markers) | Time dimension — note creation date, last-traversed, lifecycle stage band | Existing note metadata | Outer ring; rotatable; click a month to filter to notes from that period |
| **Empty patches between constellations** | Structural gaps | Existing `structural-gaps` IPC | Visually obvious by their darkness; gap-suggestion arrow drawn between cluster pairs the user might consider bridging |
| **Dome of the sky as a whole** | Universe health (Modularity + Dominance + Entropy + Connectivity) | Existing `universe-health` IPC | Visible at a glance — a balanced sky reads healthy; one constellation taking 60% of the dome reads imbalanced |

Each row maps a familiar perceptual idiom (something the user already knows from looking at a star chart) to a graph-analytic concept. The mapping is exact, not metaphorical; that's what makes the at-a-glance read work.

---

## §3 · Projection method — math and trade-offs

The technical core: how to translate a graph embedding into stable 2D star positions.

### §3.1 Two-stage pipeline

1. **Stage A — Graph embedding into 2D plane.** Compute a 2D coordinate per note using a deterministic graph embedding algorithm. Two candidates:
   - **Spectral embedding** (eigenvectors of the graph Laplacian). Deterministic. Produces a "natural" 2D layout where graph-distance approximates Euclidean distance. Standard in network visualization. Trade-off: computationally heavier (eigendecomposition).
   - **Graph-distance MDS** (multidimensional scaling on shortest-path distance matrix). Deterministic given a tie-break rule. Better neighbor-preservation than spectral for highly modular graphs. Trade-off: O(V²) memory; on a 30k-note universe that's 900M float entries — heavy. Mitigated by sparse approximations (Landmark MDS).

   **Recommendation**: spectral embedding. Sub-second on 30k nodes with sparse eigensolvers (ARPACK / LOBPCG); deterministic per `(graph_hash, k)` tuple where `k` is the number of eigenvectors retained (typically 2 or 3).

2. **Stage B — Lambert azimuthal equal-area projection** of the 2D embedding onto a unit disk. The Lambert projection maps a point at distance `r` from the embedding origin to disk-radius `r' = 2·sin(arctan(r) / 2)`. Result: dense clusters near the embedding origin land near the dome center; sparse outliers land near the rim. **Equal-area**: the visual area of a constellation territory is proportional to its node count — community sizes are perceptually accurate.

   *Why Lambert and not stereographic?* Stereographic preserves angles (constellation shapes look "right") but distorts area; community sizes mislead. Lambert distorts shapes near the edges but preserves area. For Sight, area accuracy matters more than shape accuracy because community size is the at-a-glance cue for "how dominant is this knowledge cluster."

### §3.2 Determinism & cache

The projection must be **deterministic per universe snapshot**. Two notes added or removed → recompute. Same input graph → same coordinates always. This makes the layout cacheable in SQLite (the deferred PJ-034 §1E `sight_cache` design absorbed here):

- **Cache key**: `(library_set_hash, graph_version, eigensolver_seed)`.
- **Cached payload**: per-note `(x, y)` in unit-disk coordinates + community id + centrality magnitude.
- **Invalidation**: on `note_links` write (graph version bumps, mirroring v2's `lensDataStale` pattern).
- **Persistence**: SQLite table `sight_v3_layout` with one row per `(note_id, library_set_hash, graph_version)`. Boot reads the cached layout in milliseconds; recompute only if the hash mismatches.

This is **write-time derivation** (CLAUDE.md Rule 8): the cache is maintained at write time, not rebuilt at read time. The first computation on a new graph is the only expensive moment; everything after is a SQLite SELECT.

### §3.3 Edge cases

- **Disconnected components** (notes / sub-graphs with no path to the main mass). Spectral embedding produces NaN coordinates for disconnected components. Mitigation: detect via connected-components pre-pass; place disconnected components in dedicated "outlier rings" near the rim, in their own mini-territories, labeled "Isolated."
- **Tiny universes** (fewer than ~20 notes). The star-chart aesthetic doesn't scale down well — too few stars looks empty. Mitigation: render a different layout below the threshold (e.g., a simple radial fan). Threshold tunable.
- **Universe with one giant community** (Dominance > 80%). The chart looks like a single big constellation taking the whole dome. *That's a feature, not a bug* — universe-health's Dominance metric is precisely what this visualization makes obvious.

---

## §4 · Interactivity — what the user does

### §4.1 Resting state (no interaction)

- Stars rendered.
- Constellation territories outlined with their soft fill.
- Constellation labels placed at territory centroids.
- Milky Way band rendered (PJ-035, when shipped).
- Calendar rim rendered with current month highlighted.
- **No connector lines.** This is Principle 6 from v1.1 §4.

### §4.2 Hover a star

- The hovered star highlights (size bump + brighter fill).
- Connector lines from the hovered star to its direct neighbors *appear* (only the hovered star's incident edges).
- Tooltip near the cursor shows: note title (RTL-aware), centrality rank, community name, lifecycle stage badge, last-traversed date.
- Other stars dim slightly (focus effect).

### §4.3 Click (select) a star

- The clicked star becomes the **focus star**: its territory's connector lines all render (the full constellation pattern around the focus, not just the focus's direct edges).
- A side panel slides in showing: note metadata, top-5 incoming wikilinks, top-5 outgoing wikilinks, structural-gap suggestions involving this note.
- The focus persists until the user clicks elsewhere (background = clear focus; another star = reassign focus).

### §4.4 Double-click a star

- Opens the note in the editor (current Sight v2 behavior preserved).

### §4.5 Hover a constellation territory (not a star)

- The territory highlights (border brightens).
- Tooltip shows: community name (top-3 representative terms), node count, top-3 bridge notes within, intra-community modularity contribution.

### §4.6 Click a constellation territory

- Filters the chart to that community alone (other territories dim heavily). The selected community's full constellation lines render.
- Side panel: community details, bridge notes, structural-gap suggestions involving this community.

### §4.7 Search

The Search Hub continues to work as it does for v2 Sight, but the visual response is different:
- Matched notes' stars **flare** (size pulse + brightness boost).
- Non-matching stars dim heavily.
- Constellation territories containing matched notes get a halo glow.
- Connector lines between matched notes render automatically.

### §4.8 Calendar-rim interaction

- Hover a month on the rim → preview filter (stars from that month flare).
- Click a month → persistent filter (only that month's notes render at full brightness).
- Drag the rim to "rotate" through years.

### §4.9 Right-click (context menu)

- On a star: "Open note", "Open in side pane", "Show backlinks", "Show structural-gap suggestions for this note", "Hide from chart" (temporarily exclude from Sight; resets on app restart).
- On a constellation territory: "Filter to this community", "Show bridge candidates between this and another community" (then click a second territory).
- On the background: "Reset filters", "Toggle Milky Way", "Toggle calendar rim", "Show universe-health card".

---

## §5 · Absorbing the deferred PJs

The three v1.1 paper §12 truth-status gaps absorb cleanly into v3.

### §5.1 PJ-035 — Content-similarity edges → Milky Way band

In v2, content-similarity edges would have been *more lines* on a graph already saturated with explicit-wikilink lines. Confusing.

In v3, content-similarity becomes a **density field**, not a set of edges. Compute pairwise TF-IDF cosine similarity (above a threshold, say 0.3) and accumulate per-pixel density across the projected disk. Render as a soft gradient — the Milky Way band. High-similarity regions form the brightest band texture.

Implementation: a 2D heatmap layer underneath the stars, with low alpha. Rendered once per cache-warm cycle (it's deterministic). Toggleable via right-click menu (some users will prefer cleaner stars).

### §5.2 PJ-036 — Layer peeling → "hide brightest stars" toggle

In v2, layer peeling would have been a Settings checkbox somewhere in the sidebar. Buried.

In v3, layer peeling is a **visual idiom** — there's a magnitude slider on the side panel: "show stars with magnitude ≥ N." Drag the slider to the right and the brightest (most central) stars vanish, exposing the secondary structure beneath. Drag back to restore. The recompute happens client-side because the analytics are already cached.

This is the same UX pattern as exoplanet-discovery and meteor-shower atlases — astronomers iterate magnitude thresholds as a routine workflow.

### §5.3 PJ-037 — Map ↔ Sight integration → two-up panel

In v2, Map and Sight were two independent overlays. In v3, they become a **two-up panel** when both are activated:

- Left half: Constellation Map (radial sunburst — the "shape" view).
- Right half: Sight v3 (star chart — the "patterns" view).
- Shared selection cursor: clicking a Map segment highlights the corresponding stars/territory on Sight; clicking a Sight territory highlights the corresponding Map arcs.
- Shared search bar: query highlights matching notes in both views simultaneously.

The two-up panel is opt-in (a toggle in the dock). Single-view usage (Map alone or Sight alone) remains the default for users who want focus.

---

## §6 · Universe-health at a glance

The four health metrics from v1.1 §3.4 (Modularity, Dominance, Entropy, Connectivity) become *visually obvious* in v3:

- **High modularity** (well-defined communities) → the constellation territories have crisp borders with empty space between them.
- **Low modularity** → territories blend into each other; the chart reads as a smear.
- **High dominance** (one community owns >50% of notes) → one giant constellation takes up most of the dome.
- **Healthy dominance** (<35%) → multiple constellations of comparable size populate the dome.
- **High entropy** (knowledge evenly distributed) → the dome looks "balanced" — territories of similar size in similar visual prominence.
- **Low entropy** → a few dominant constellations + many tiny ones in the periphery.
- **High connectivity** (low avg path length) → bridge notes (high-magnitude stars) populate the regions between constellations.
- **Low connectivity** → bridge notes are sparse; constellations are visually isolated.

A small universe-health card lives in the side panel showing the four numerical metrics with green/yellow/red badges. The visual reading is the *primary* interpretation; the numbers are the *confirmation*.

---

## §7 · Internationalization

Constellation is RTL-first by design (CLAUDE.md "Language-First by Design"). v3 must work natively for Arabic, English, mixed-script, and 13 other locales.

### §7.1 Constellation labels

Top-3 representative terms per community are language-of-the-content. An Arabic-content community labels in Arabic; an English-content community in English; mixed in mixed-script. The label text uses `dir="auto"` and `unicode-bidi: plaintext` to render correctly without per-locale switching.

### §7.2 Calendar rim

Month names render in the user's interface locale. Hijri-aware option: the rim toggles between Gregorian and Hijri month markers (Constellation already has the Hijri date helpers in the lexicon).

### §7.3 RTL UI

The dock button + side panel + tooltip behave RTL when the interface locale is RTL (CSS `dir="rtl"` cascades down from `<html>`). The chart itself is direction-agnostic (a circle is a circle), but the rim's month labels and the side panel's text follow direction.

### §7.4 Search

Search query string + results follow the same multi-language path the v2 Search Hub already uses (CTSE bridge adapter for cross-language ≈ similar matches; Arabic root-aware search via the existing Arabic Engine). v3 doesn't change search; it changes how results visualize.

---

## §8 · Performance

### §8.1 Targets

| Metric | Budget |
|---|---|
| First-toggle latency on 7,600-note universe (cold cache) | ≤ 500 ms |
| First-toggle latency on 7,600-note universe (warm SQLite cache) | ≤ 50 ms |
| First-toggle latency on 30k-edge universe (warm cache) | ≤ 100 ms |
| Hover-star highlight | ≤ 16 ms (single-frame) |
| Search-flare animation start | ≤ 50 ms after query |
| Idle-Sight per-frame cost (no hover, no search, no select) | ≤ 1 ms |
| Memory footprint (Sight overlay open, 7,600 notes) | ≤ 40 MB above app baseline |

### §8.2 Strategy

- **Spectral embedding** runs in Rust (existing analytics module; new function `compute_layout_embedding`). Sub-second on Brandes-scale graphs.
- **SQLite cache layer** (mirrors the deferred PJ-034 §1E design). Boot reads the cached layout in milliseconds; full recompute only on graph version bump.
- **Three-layer rendering**:
  - **SVG layer 1** — territories + Milky Way + calendar rim + labels + selection cursor (~50-100 elements; manageable).
  - **Canvas layer 2** — stars (up to 30k+ elements; Pixi.js or vanilla 2D Canvas).
  - **DOM layer 3** — hover tooltips, side panel, right-click menu (sparse; ergonomic).
- **Reveal-on-demand** (Principle 6) keeps the per-frame edge-iteration cost zero in the resting state — the same insight that landed in v2 §1B.
- **Web Worker offload** for the magnitude-slider recompute (Layer Peeling — PJ-036). The slider fires `requestAnimationFrame`-throttled events; each event triggers a worker to recompute the visible-star set; result returned in milliseconds.
- **Idle-prewarm**: on app boot, after first paint, schedule a `requestIdleCallback` to compute the layout and warm the cache so the first toggle is instant. (Mirrors the deferred PJ-034 §1D design.)

### §8.3 Boot-perf invariant

Per CLAUDE.md "Performance Rule 8 (write-time derivation)" + "no new feature may regress boot": v3's prewarm runs **only** during idle time (after `boot:hydrated`). The boot critical path stays ≤ 6 seconds (already ≤ 1 second on Boss's machine). v3 never blocks first paint.

---

## §9 · Phased rollout

v3 ships in three phases, each its own MIG, each landable independently.

### §9.1 Phase 1 — Projection foundation (MIG-018)

- Rust: `compute_layout_embedding` IPC (spectral embedding + Lambert projection).
- SQLite: `sight_v3_layout` table + write-time triggers on `note_meta` / `note_links`.
- Frontend: new `src/lib/sight/projection.ts` (cache reader + invalidation), `src/lib/sight/v3/SightV3.svelte` (the chart component, SVG + Canvas layers).
- v3 dock button + Settings plugin entry behind `SIGHT_V3_ENABLED` const (mirrors v2's gating).
- **Deliverable**: stars render at correct positions; constellation territories drawn as soft polygons; basic hover/click works; no Milky Way, no layer peeling, no Map↔Sight yet.

### §9.2 Phase 2 — Density + time + search (MIG-019)

- PJ-035 (content-similarity edges) implemented as Milky Way band. New Rust IPC `constellation_sight_v3_similarity_field` returning a 2D density grid.
- Calendar rim: month labels, year drag, Gregorian ↔ Hijri toggle.
- Search integration: matched stars flare, territories halo, connector lines render between matches.
- Universe-health card in side panel.
- **Deliverable**: full visual grammar present; v3 reaches feature parity with v2's analytics + the v1.1 paper's §3.3 third edge type.

### §9.3 Phase 3 — Layer peeling + Map↔Sight (MIG-020)

- PJ-036 (layer peeling) implemented as magnitude slider in side panel. Worker offload for recompute.
- PJ-037 (Map ↔ Sight integration) implemented as two-up panel. Shared selection cursor across both views.
- v2 retired: `SIGHT_V2_ENABLED` const removed (or the `lens.rs` module deleted if no other consumers remain). v3 becomes the only Sight engine.
- **Deliverable**: v3 supersedes v2 in production; v2 fallback removed from the codebase only after Boss confirms v3 is stable across multiple test sessions.

### §9.4 Sequencing

Each MIG follows the four-phase migration cycle (Architect → Plan → Build → Audit). Each MIG ends with a Boss test gate where Eisa runs the new phase and confirms before the next opens. v3 doesn't replace v2 silently — every step is opt-in via `SIGHT_V3_ENABLED`, and v2 stays available as the rollback target through Phase 2.

---

## §10 · Out of scope (this paper)

- **AI integration.** v3 doesn't add LLM features. Future work (CE Layer 2 phases) can layer AI on top of the v3 analytics, but that's separate.
- **Real-time collaborative Sight.** Multi-user shared cursor. Not on the roadmap.
- **3D mode.** v3 is 2D by Eisa's directive. A 3D flyaround is a separate concept, possibly compelling, but not what's being decided here.
- **Mobile / web Sight.** Constellation is a desktop-first PKM; v3 inherits that scope.
- **Custom user constellations.** Letting users draw their own constellation lines (manual annotation of "these notes belong together"). Cool idea; defer to a future PJ if Boss requests after using v3.

---

## §11 · Open questions for Eisa

These are design choices I'm flagging for your call before code begins.

1. **Spectral vs. graph-distance MDS** for the embedding (§3.1). Spectral is faster and standard. MDS preserves neighbor-distance better. I lean spectral; do you have a preference, or should I run a comparison on your universe and show both?

2. **Lambert vs. stereographic** projection (§3.1). Lambert preserves area (community sizes accurate); stereographic preserves shape (constellations look "right"). I lean Lambert; flag if you want shape accuracy.

3. **Calendar rim — Gregorian only, Hijri only, or both with toggle?** Default suggestion: both, with toggle defaulting to user's locale (Hijri for Arabic locale, Gregorian for others).

4. **Magnitude slider direction.** Drag right = hide bright stars (peel layer); drag left = show all. Or inverse (drag right = brighten; drag left = peel). Astronomy convention is the former; UX intuition might be the latter. I lean astronomy convention, but worth asking.

5. **Two-up panel default**. Map alone, Sight alone, or both side-by-side as the default Sight experience? I lean **Sight alone as default**, with a "open Map alongside" toggle. Two-up is power-user mode.

6. **Tooltip vs. always-on labels for constellations.** Should constellation labels always be visible (cluttery on a dense chart) or only on hover/select (cleaner)? I lean hover-only, with a Settings toggle to flip.

7. **Color scheme for territories.** Eight pastel hues cycled via Louvain community id, or a perceptually-uniform scale, or user-pickable per-community? I lean cycled pastels; respects existing Sky View / Map color conventions.

8. **Search query persistence.** When a search filters Sight, is the filter cleared on Esc, on next click, or persistent across toggle? I lean Esc + click-background; persistent across toggle is power-user.

9. **Render layer choice — Pixi.js, vanilla Canvas, or WebGL via deck.gl/regl?** Pixi.js is what v2 + Sky View use. Vanilla Canvas is simpler. WebGL libraries scale higher. I lean Pixi.js for consistency with the rest of the codebase, unless 30k+ stars start showing strain.

10. **Accessibility.** Should I bake in a high-contrast mode (sky black → light gray; stars white → black) and a tab-navigable star list for keyboard users in Phase 1, or defer to a separate accessibility pass? I lean **bake into Phase 1** — accessibility is cheaper to design in than to retrofit.

---

## §12 · Acceptance criteria (high level — refined per-MIG)

For PJ-038 to close as Done:

1. v3 ships in production behind `SIGHT_V3_ENABLED = true`; users can use v3 as their primary Sight.
2. Star-chart visual grammar matches §2 — territories, magnitudes, connector-lines-on-hover, calendar rim all present.
3. Spatial layout is **stable across sessions** (Eisa can build muscle memory of where notes live on the dome).
4. First-toggle latency on Boss's universe meets §8.1 budgets.
5. PJ-035 (Milky Way), PJ-036 (layer peeling), PJ-037 (Map↔Sight) all absorbed and live.
6. Universe-health is readable at a glance — Eisa can look at the dome and tell whether their universe is balanced or dominated, without reading numbers.
7. Three-agent audit clean across all three MIGs (MIG-018 / 019 / 020).
8. v2 retired (Phase 3 deliverable) — `SIGHT_V2_ENABLED` removed or `lens.rs` deleted, depending on whether other consumers remain.
9. Help docs rewritten (en + ar) with new screenshots + interaction walkthroughs.
10. Boss confirms — across multiple sessions — that v3 delivers the at-a-glance promise of v1.1 §13.

---

## §13 · Glossary

| Term | Definition |
|---|---|
| **Star** | A note rendered as a point on the dome. |
| **Magnitude** | The size + brightness of a star, mapped to its betweenness centrality. |
| **Constellation** | A Louvain community. The bordered territory + the connector lines + the label collectively. |
| **Territory** | The polygonal region of the dome assigned to a constellation. |
| **Connector line** | A wikilink or shared-tag edge within a constellation. Rendered only on hover/select. |
| **Milky Way band** | The diffuse density wash representing content-similarity. |
| **Calendar rim** | The outer ring showing months / years. |
| **Dome** | The full circular field of the chart — the user's universe at a glance. |
| **Magnitude slider** | The Layer Peeling control. Hide stars dimmer than threshold N. |
| **Two-up panel** | The Map + Sight side-by-side mode (power user). |

---

## §14 · Cross-references

This paper is read alongside:

- **`docs/Constellation-Sight-Concept-Paper-v1.1.md`** — the analytical foundation (six core mechanics, three edge types, universe health, six design principles incl. Principle 6 reveal-on-demand). v3 inherits all of it.
- **`docs/Constellation_Map_Concept_Paper_Eisa.pdf`** — companion paper for the Map (radial sunburst). PJ-037 stitches the two views together.
- **`docs/Constellation Pending Jobs v1.5.md`** — PJ-038 (this paper's home), PJ-035 / PJ-036 / PJ-037 (absorbed deliverables).
- **`lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`** — what v2 shipped, what was deferred, why §1E inheritance matters.
- **`lab/reports/MIG-017-DISABLE-V2-SIGHT-{ARCHITECT,PLAN,AUDIT}.md`** — how v2 was disabled cleanly, how `SIGHT_V2_ENABLED` will pair with `SIGHT_V3_ENABLED`.
- **CLAUDE.md** — Performance Rules (esp. Rule 8 write-time derivation), Architecture Principles (esp. Language-First by Design), Working Agreements (esp. #4 validate-against-architecture).

---

**End of v1.0.** This paper is the design contract for PJ-038. After Eisa approves, the next document is the **MIG-018 Architect** — the first of three migration cycles that build v3 in phases.
