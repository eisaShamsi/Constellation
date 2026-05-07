# Constellation Sight v3 — Concept Paper

**Version 1.1 (Eisa's design calls baked in) | 2026-05-07**
**Origin**: PJ-038 (multi-MIG). Companion paper to `Constellation-Sight-Concept-Paper-v1.1.md` (the *analytical foundation* — what Sight is for). This v3 paper is the *visual + interaction specification* — what v3 looks like and how it is built.
**Author**: Eisa Alshamsi · eisa@uconstellation.world · drafted with Claude.
**Status**: ✅ Design ratified by Eisa 2026-05-07. Ready for MIG-018 Architect.

> **What changed from v1.0** (the same-day draft):
>
> - **§2 / §4 (resting state)**: connector lines are now **faint at rest** instead of fully hidden. Hover/select brightens them. Eisa's directive: *"with v3, we will show it as faint lines until the user hovers over it or the connected nodes linking them."* This is the v3 reframe of v1.1 paper's Principle 6 — *reveal* now means *brighten*, not *render-from-zero*.
> - **§5.3 (Map↔Sight integration) — removed.** Eisa: *"There won't be Map-Sight integration."* PJ-037 is **Rejected** in Pending Jobs v1.6. Sight v3 stays single-view; Map and Sight remain independent surfaces. (Two-up panel concept dropped from §9 / §10 / §13 accordingly.)
> - **§3 projection math**: **graph-distance MDS** chosen over spectral. **Both Lambert and stereographic projections** ship; user-toggle in Settings.
> - **§7 calendar rim**: **Gregorian by default**; users add other calendars (Hijri, Solar Hijri, Hebrew, etc.) via Settings → "Calendar systems."
> - **§4 magnitude slider**: astronomy convention — drag right hides bright stars (peel layer).
> - **§4 / §6 territory labels**: hover/select only by default; Settings toggle for always-on.
> - **§6 territory colors**: cycled pastel by Louvain id; user-overridable per community via existing **Style Settings** system.
> - **§4 search filter persistence**: Esc + click-background to clear.
> - **§8 render layer**: **Pixi.js** (consistent with Sky View / v2).
> - **§10 accessibility**: **deferred** to a separate post-v3 PJ; not baked into Phase 1.
>
> All ten §11 questions resolved. §11 is now **§11 — Design decisions (resolved 2026-05-07)** below.

> **Read this paper alongside the v1.1 analytical paper.** The v1.1 analytical paper (`Constellation-Sight-Concept-Paper-v1.1.md`) covers the analytical core (centrality, communities, structural gaps, universe health, three edge types, six core mechanics, five design principles, Principle 6). All of that is **inherited** by v3 unchanged. This v3 paper covers what's new: the visual grammar, the projection math, the interaction model, PJ-035 (Milky Way) + PJ-036 (layer peeling) absorption, and the phased rollout.

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
| **Constellation territory** (bordered region) | Louvain community | `constellation_sight_communities` IPC | Soft-filled polygonal region (low alpha); border drawn at the convex hull or alpha-shape boundary. **Default color**: cycled from a fixed 8-pastel palette by Louvain community id (consistent with Sky View / Map idioms). **User override**: per-community color via existing Style Settings (Eisa's call, §11 Q7). |
| **Constellation lines** (connector lines between stars) | Wikilinks + shared-tag edges *within* a community | Sight graph build (existing) | **Faint white lines at rest** (very low alpha, ~0.10–0.15) — the structure is always visible, just unobtrusive. **Brighten on hover/select** to high alpha (~0.8) for the focused star or constellation; other lines stay faint. Principle 6 reframed: *reveal* means *brighten*, not *render-from-zero*. |
| **Milky Way band** (diffuse density wash) | Content-similarity density (PJ-035) | TF-IDF cosine similarity field | Continuous density gradient; high-similarity regions form the band texture |
| **Constellation label** (mythological name) | Top-3 representative terms per community | Existing `community-profiles` machinery | RTL-aware text placed at territory centroid, sized by community population |
| **Calendar rim** (months / RA-hour markers) | Time dimension — note creation date, last-traversed, lifecycle stage band | Existing note metadata | Outer ring; rotatable; click a month to filter to notes from that period |
| **Empty patches between constellations** | Structural gaps | Existing `structural-gaps` IPC | Visually obvious by their darkness; gap-suggestion arrow drawn between cluster pairs the user might consider bridging |
| **Dome of the sky as a whole** | Universe health (Modularity + Dominance + Entropy + Connectivity) | Existing `universe-health` IPC | Visible at a glance — a balanced sky reads healthy; one constellation taking 60% of the dome reads imbalanced |

Each row maps a familiar perceptual idiom (something the user already knows from looking at a star chart) to a graph-analytic concept. The mapping is exact, not metaphorical; that's what makes the at-a-glance read work.

---

## §3 · Projection method — math and trade-offs

The technical core: how to translate a graph embedding into stable 2D star positions.

### §3.1 Two-stage pipeline (decisions baked in)

1. **Stage A — Graph-distance MDS embedding into 2D plane** (Eisa's call, §11 Q1).
   - Compute pairwise shortest-path distance matrix on the multi-edge graph (wikilinks, weight 1.0; shared tags, weight 0.6; content similarity once PJ-035 lands, weight 0.3).
   - Apply classical MDS (multidimensional scaling) to embed the distance matrix into a 2D plane.
   - **Why MDS over spectral**: MDS preserves *neighbor distances* better on highly modular graphs — the graphs Constellation actually has. Densely-connected notes cluster correctly; the "feel" of the layout matches the graph's topology more faithfully than spectral's eigenvector projection. Spectral was the *speed* candidate; MDS was the *fidelity* candidate. Eisa picked fidelity.
   - **Determinism**: classical MDS is deterministic given a tie-break rule on equal-distance pairs (lexicographic on note id). Same input graph → same coordinates always.
   - **Memory cost**: O(V²) distance matrix is heavy at 30k+ notes (900M float entries ≈ 3.6 GB). **Mitigated by Landmark MDS**: pick k landmarks (typically k = 50-100 by max betweenness or random), compute a V×k distance matrix, embed via classical MDS on landmarks, place non-landmarks via triangulation. O(V·k) memory; sub-second on 30k notes.
   - **Implementation home**: new Rust function `compute_layout_embedding` in the existing `lens.rs` analytics module (or a new `sight_layout.rs` if it grows large). Returns `Vec<(NoteId, f32, f32)>` per `(graph_hash, k_landmarks)` cache key.

2. **Stage B — Projection to the unit disk** (Eisa's call, §11 Q2: ship both, user-toggle).
   - **Lambert azimuthal equal-area** (default). Maps embedding distance `r` to disk radius `r' = 2·sin(arctan(r) / 2)`. Equal-area: community territory sizes are visually proportional to node count. Best for *"which community dominates?"* reads.
   - **Stereographic** (alternative, user-pickable). Preserves angles → constellation shapes look "right" near the edges. Best for users who prioritize *shape recognition* over area accuracy.
   - **User toggle**: Settings → Sight → "Projection: Lambert (equal-area) / Stereographic (equal-angle)". Default = Lambert. Switching is a frontend-only operation (the embedding is the same; the projection is a per-point math step in the renderer). No recompute, no cache invalidation.
   - Both projections share the same MDS embedding output; the projection step is a transformation applied at render time. This is why both can ship cheaply in Phase 1 — the cost is one extra branch in the projection function, not a duplicate pipeline.

### §3.2 Determinism & cache

The MDS embedding must be **deterministic per universe snapshot**. Two notes added or removed → recompute. Same input graph → same coordinates always. This makes the embedding cacheable in SQLite (the deferred PJ-034 §1E `sight_cache` design absorbed here):

- **Cache key**: `(library_set_hash, graph_version, k_landmarks)`.
- **Cached payload**: per-note `(x, y)` in unit-2D-embedding coordinates + community id + centrality magnitude. The projection (Lambert vs. stereographic) is applied at render time, *not* cached — switching projections is a free frontend operation.
- **Invalidation**: on `note_links` write (graph version bumps, mirroring v2's `lensDataStale` pattern).
- **Persistence**: SQLite table `sight_v3_layout` with one row per `(note_id, library_set_hash, graph_version)`. Boot reads the cached layout in milliseconds; recompute only if the hash mismatches.

This is **write-time derivation** (CLAUDE.md Rule 8): the cache is maintained at write time, not rebuilt at read time. The first computation on a new graph is the only expensive moment; everything after is a SQLite SELECT plus a projection transform.

### §3.3 Edge cases

- **Disconnected components** (notes / sub-graphs with no path to the main mass). Spectral embedding produces NaN coordinates for disconnected components. Mitigation: detect via connected-components pre-pass; place disconnected components in dedicated "outlier rings" near the rim, in their own mini-territories, labeled "Isolated."
- **Tiny universes** (fewer than ~20 notes). The star-chart aesthetic doesn't scale down well — too few stars looks empty. Mitigation: render a different layout below the threshold (e.g., a simple radial fan). Threshold tunable.
- **Universe with one giant community** (Dominance > 80%). The chart looks like a single big constellation taking the whole dome. *That's a feature, not a bug* — universe-health's Dominance metric is precisely what this visualization makes obvious.

---

## §4 · Interactivity — what the user does

### §4.1 Resting state (no interaction)

- Stars rendered.
- **Connector lines rendered as faint base layer** (very low alpha, ~0.10–0.15). The structural pattern of the universe is always visible at rest — the user can see *that* there are connections without those lines screaming for attention. This is the v3 reframe of v1.1 paper's Principle 6: *reveal* now means *brighten*, not *render-from-zero*. Rationale (Eisa 2026-05-07): seeing the structure is part of the at-a-glance read; v2's "all lines hidden" went too far for v3's spatial-memory grammar.
- Constellation territories outlined with their soft fill.
- Constellation labels: **hidden by default; on-hover/select; or always-on** per Settings → Sight → "Show constellation labels at rest" toggle.
- Milky Way band rendered (PJ-035, when Phase 2 ships).
- Calendar rim rendered with current month highlighted (Gregorian by default; users may add other calendars via Settings → "Calendar systems").

### §4.2 Hover a star

- The hovered star highlights (size bump + brighter fill).
- **Incident edges brighten** — connector lines from the hovered star to its direct neighbors transition from faint base alpha (~0.12) to focus alpha (~0.85). Other lines stay at base alpha. The transition is animated at ~150 ms for visual softness.
- Tooltip near the cursor shows: note title (RTL-aware), centrality rank, community name, lifecycle stage badge, last-traversed date.
- Other stars dim slightly (focus effect, ~0.6 alpha).

### §4.3 Click (select) a star

- The clicked star becomes the **focus star**.
- **Its constellation's edges all brighten** to focus alpha (the full constellation pattern around the focus, not just the focus's direct edges). Edges in other constellations stay at base faint alpha.
- A side panel slides in showing: note metadata, top-5 incoming wikilinks, top-5 outgoing wikilinks, structural-gap suggestions involving this note.
- The focus persists until the user clicks elsewhere (background = clear focus; another star = reassign focus).

### §4.4 Double-click a star

- Opens the note in the editor (current Sight v2 behavior preserved).

### §4.5 Hover a constellation territory (not a star)

- The territory highlights (border brightens; fill alpha bumps slightly).
- **All edges within that territory brighten** to focus alpha. Edges in other constellations stay faint.
- Tooltip shows: community name (top-3 representative terms — labels appear here on hover even if "always-on labels" is off), node count, top-3 bridge notes within, intra-community modularity contribution.

### §4.6 Click a constellation territory

- Filters the chart to that community alone (other territories dim heavily; their edges drop to ~0.05 alpha).
- The selected community's full constellation lines render at focus alpha; constellation label appears at the territory centroid.
- Side panel: community details, bridge notes, structural-gap suggestions involving this community.

### §4.7 Search

The Search Hub continues to work as it does for v2 Sight, but the visual response is different:
- Matched notes' stars **flare** (size pulse + brightness boost).
- Non-matching stars dim heavily.
- Constellation territories containing matched notes get a halo glow.
- Connector lines between matched notes brighten to focus alpha; all other lines drop to a deeper-than-base dim (~0.05 alpha) to push the matches forward.
- **Filter persistence**: ESC clears the search filter; clicking the chart background clears it. Filter does NOT persist across Sight close/reopen — fresh open = no filter. (Resolves §11 Q8.)

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

Two of the three v1.1 paper §12 truth-status gaps absorb cleanly into v3. The third (PJ-037 Map↔Sight integration) was **rejected** by Eisa 2026-05-07 — Sight v3 is single-view; Map and Sight remain independent surfaces.

### §5.1 PJ-035 — Content-similarity edges → Milky Way band

In v2, content-similarity edges would have been *more lines* on a graph already saturated with explicit-wikilink lines. Confusing.

In v3, content-similarity becomes a **density field**, not a set of edges. Compute pairwise TF-IDF cosine similarity (above a threshold, say 0.3) and accumulate per-pixel density across the projected disk. Render as a soft gradient — the Milky Way band. High-similarity regions form the brightest band texture.

Implementation: a 2D heatmap layer underneath the stars, with low alpha. Rendered once per cache-warm cycle (it's deterministic). Toggleable via right-click menu (some users will prefer cleaner stars).

### §5.2 PJ-036 — Layer peeling → "hide brightest stars" toggle

In v2, layer peeling would have been a Settings checkbox somewhere in the sidebar. Buried.

In v3, layer peeling is a **visual idiom** — there's a magnitude slider on the side panel: "show stars with magnitude ≥ N." Drag the slider **to the right** and the brightest (most central) stars vanish, exposing the secondary structure beneath. Drag back to restore. The recompute happens client-side because the analytics are already cached.

This is the same UX pattern as exoplanet-discovery and meteor-shower atlases — astronomers iterate magnitude thresholds as a routine workflow. Direction follows astronomy convention (Eisa's call, §11 Q4): right = peel.

### §5.3 PJ-037 — Map ↔ Sight integration: REJECTED

Eisa's directive 2026-05-07: *"There won't be Map-Sight integration."*

The two surfaces remain **independent**. Map is the "shape" view (radial sunburst, organizational hierarchy). Sight is the "patterns" view (star chart, conceptual relationships). Each lives in its own dock and is opened separately. PJ-037 is marked **Rejected** in Pending Jobs v1.6; the number is retired with the entry preserved per the stable-reference-numbers rule.

Rationale (inferred): two-up panels split the user's attention; Eisa's vision for v3 is *one image, one read*. Map and Sight answer different questions cleanly when used separately. The "Map diagnoses, Sight prescribes" loop framed in the v1.1 analytical paper §7 happens in the user's head, not in a shared cursor.

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

Eisa's call (§11 Q3): **Gregorian by default**; users add other calendars (Hijri, Solar Hijri, Hebrew, etc.) via **Settings → Sight → "Calendar systems."** Multiple calendars can be enabled simultaneously — concentric rim layers, one per active calendar (innermost = primary; outer rings = secondary calendars). Constellation already has Hijri date helpers in the lexicon and the existing locale-aware date-formatting infrastructure; v3 reuses both.

Month-name rendering follows the user's interface locale + the per-calendar locale (e.g., a user with English UI but Hijri secondary calendar sees English Gregorian month names + Arabic Hijri month names on separate rim layers).

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

- **Landmark-MDS embedding** runs in Rust (existing analytics module; new function `compute_layout_embedding`). Sub-second on Brandes-scale graphs (~30k notes) using k = 50-100 landmarks. The full V×V distance matrix is *never* materialized.
- **SQLite cache layer** (mirrors the deferred PJ-034 §1E design). Boot reads the cached layout in milliseconds; full recompute only on graph version bump.
- **Three-layer rendering** (Pixi.js for Layers 1+2 per Eisa's §11 Q9, consistent with Sky View / v2):
  - **Pixi.js Layer 1 — base** — stars + faint connector lines (always-on, ~0.10–0.15 alpha) + territory fills + Milky Way density + calendar rim. Drawn ONCE per cache-warm cycle. Static; not redrawn per frame.
  - **Pixi.js Layer 2 — focus overlay** — the brightened constellation lines + flared stars + halo glows + selection cursor for the currently-hovered/selected element. Redrawn on hover/select state change only (not per frame). Sparse element count even when focused (one constellation's worth of edges + a few stars).
  - **DOM Layer 3 — UI** — tooltips, side panel, right-click menu, magnitude slider, calendar-rim drag handle (sparse; ergonomic).
- **Why two Pixi layers, not one**: separating *base* from *focus overlay* keeps the per-frame draw cost near zero at rest. The base layer is computed-once-stays-still; only the overlay redraws on interaction. This is the v3 reframe of v2's Principle 6 — the perf benefit (zero per-frame edge cost at rest) is preserved even though we *show* the faint structure now.
- **Web Worker offload** for the magnitude-slider recompute (Layer Peeling — PJ-036). The slider fires `requestAnimationFrame`-throttled events; each event triggers a worker to recompute the visible-star set; result returned in milliseconds, base layer redraws.
- **Idle-prewarm**: on app boot, after first paint, schedule a `requestIdleCallback` to compute the layout and warm the cache so the first toggle is instant. (Mirrors the deferred PJ-034 §1D design.)
- **Projection switch** (Lambert ↔ stereographic): re-runs the projection transform on the same cached embedding; redraws Layer 1; Layer 2 redraws on next interaction. ~1-frame transition.

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

### §9.3 Phase 3 — Layer peeling + v2 retire (MIG-020)

- PJ-036 (layer peeling) implemented as magnitude slider in side panel. Worker offload for recompute. Astronomy-convention direction (drag right = peel bright stars).
- v2 retired: `SIGHT_V2_ENABLED` const removed (or the `lens.rs` module deleted if no other consumers remain). v3 becomes the only Sight engine.
- **Deliverable**: v3 supersedes v2 in production; v2 fallback removed from the codebase only after Boss confirms v3 is stable across multiple test sessions.

*PJ-037 (Map ↔ Sight integration) is NOT in MIG-020.* Eisa rejected the integration concept 2026-05-07; PJ-037 is marked Rejected in Pending Jobs v1.6, number retired.

### §9.4 Sequencing

Each MIG follows the four-phase migration cycle (Architect → Plan → Build → Audit). Each MIG ends with a Boss test gate where Eisa runs the new phase and confirms before the next opens. v3 doesn't replace v2 silently — every step is opt-in via `SIGHT_V3_ENABLED`, and v2 stays available as the rollback target through Phase 2.

---

## §10 · Out of scope (this paper)

- **AI integration.** v3 doesn't add LLM features. Future work (CE Layer 2 phases) can layer AI on top of the v3 analytics, but that's separate.
- **Real-time collaborative Sight.** Multi-user shared cursor. Not on the roadmap.
- **3D mode.** v3 is 2D by Eisa's directive. A 3D flyaround is a separate concept, possibly compelling, but not what's being decided here.
- **Mobile / web Sight.** Constellation is a desktop-first PKM; v3 inherits that scope.
- **Custom user constellations.** Letting users draw their own constellation lines (manual annotation of "these notes belong together"). Cool idea; defer to a future PJ if Boss requests after using v3.
- **Map ↔ Sight integration (PJ-037).** Eisa rejected 2026-05-07. The two surfaces remain independent; users open Map and Sight separately, and the analytical loop ("Map diagnoses, Sight prescribes") happens in the user's head, not in a shared cursor.
- **Accessibility — high-contrast mode + keyboard star-list navigation.** Eisa's call (§11 Q10): defer to a separate post-v3 PJ. Phase 1 ships without these; a follow-up PJ will retrofit. (Note: this paper still bakes RTL + per-locale calendars + multi-script labels into Phase 1 — those are part of Constellation's Language-First architecture, not "accessibility extras.")

---

## §11 · Design decisions (resolved 2026-05-07)

All ten v1.0 open questions resolved by Eisa's design review. Two structural revisions made beyond §11 scope (§4.1 faint-lines-at-rest; §5.3 PJ-037 rejected).

| # | Question | Decision | Implementation effect |
|---|---|---|---|
| 1 | Embedding algorithm | **Graph-distance MDS** (Landmark variant for memory) | §3.1 Stage A; Rust `compute_layout_embedding` uses MDS, not spectral. Better neighbor-preservation on highly modular graphs. |
| 2 | Projection — Lambert or stereographic | **Both ship; user toggle** | §3.1 Stage B; Settings → Sight → "Projection" with two options. Lambert default. Switching is frontend-only (free). |
| 3 | Calendar rim | **Gregorian default; user adds others via Settings** | §7.2; Settings → Sight → "Calendar systems." Multiple calendars render as concentric rim layers. |
| 4 | Magnitude slider direction | **Astronomy convention** — drag right = peel bright stars | §5.2; Phase 3 implementation per §9.3. |
| 5 | Two-up panel default | **N/A — PJ-037 rejected** | §5.3 reframed; Map and Sight stay independent. Two-up panel concept dropped from §9 / §10 / §13. |
| 6 | Constellation labels at rest | **Hover/select only by default; Settings toggle for always-on** | §4.1 / §4.5 / §4.6; Settings → Sight → "Show constellation labels at rest." |
| 7 | Color scheme | **Cycled pastels by Louvain id default; user-overridable via Style Settings** | §2 territory row; integrates with existing Style Settings system. |
| 8 | Search filter persistence | **Esc + click-background to clear** | §4.7; no persistence across Sight close/reopen. |
| 9 | Render layer | **Pixi.js** (consistent with Sky View / v2) | §8.2; two Pixi layers (base + focus overlay) + DOM layer for UI chrome. |
| 10 | Accessibility (high-contrast / keyboard nav) | **Defer to separate post-v3 PJ** | §10 out-of-scope. RTL + per-locale calendars + multi-script labels still in Phase 1 (Language-First architecture, not "accessibility extras"). |

### Beyond §11 — two structural revisions

| Topic | Decision | Where it landed |
|---|---|---|
| **Resting-state connector lines** (was: "no lines at rest" in v1.0) | **Faint at rest, brighten on hover/select.** Eisa: *"with v3, we will show it as faint lines until the user hovers over it or the connected nodes linking them."* Reframes v1.1 paper Principle 6 — *reveal* now means *brighten*, not *render-from-zero*. | §2 territory-line row; §4.1–§4.6 interactivity revised; §8.2 rendering split into base + focus-overlay layers. |
| **Map ↔ Sight integration** (was: §5.3 absorbed into v3) | **Rejected.** Eisa: *"There won't be Map-Sight integration."* PJ-037 marked Rejected in Pending Jobs v1.6; number retired per stable-reference-numbers rule. | §5.3 reframed; §9.3 MIG-020 phase reduced to PJ-036 + v2 retire only; §10 lists explicitly; §13 glossary drops "Two-up panel." |

---

## §12 · Acceptance criteria (high level — refined per-MIG)

For PJ-038 to close as Done:

1. v3 ships in production behind `SIGHT_V3_ENABLED = true`; users can use v3 as their primary Sight.
2. Star-chart visual grammar matches §2 — territories, magnitudes, connector-lines-on-hover, calendar rim all present.
3. Spatial layout is **stable across sessions** (Eisa can build muscle memory of where notes live on the dome).
4. First-toggle latency on Boss's universe meets §8.1 budgets.
5. PJ-035 (Milky Way) and PJ-036 (layer peeling) absorbed and live. PJ-037 explicitly *not* part of v3 (Eisa rejected the integration concept; remains independent in Pending Jobs).
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
| **Magnitude slider** | The Layer Peeling control. Drag right = hide stars *brighter* than threshold N (peels the dominant layer; astronomy convention). |
| **Base layer / focus overlay** | Two Pixi.js render layers. Base is static (stars + faint connector lines + territories + Milky Way + rim) drawn once per cache cycle. Focus overlay redraws on hover/select to brighten the focused star or constellation. |

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

**End of v1.1.** This paper is the design contract for PJ-038, ratified by Eisa 2026-05-07. The next document is the **MIG-018 Architect** — the first of three migration cycles that build v3 in phases (MIG-018 projection foundation → MIG-019 density + time + search → MIG-020 layer peeling + v2 retire).

v1.0 stays in `docs/` as historical record (the "drafted, awaiting review" state). v1.1 supersedes it for active reference.
