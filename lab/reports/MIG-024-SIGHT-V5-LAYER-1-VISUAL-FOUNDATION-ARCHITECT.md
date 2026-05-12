# MIG-024 — Sight v5 Layer 1 Visual Foundation — Architect

**Phase:** 1 of 4 (/migration discipline) · **Date:** 2026-05-12
**Status:** Draft for Eisa to lock decisions before Plan opens.
**Reference contract:** `docs/Constellation-Sight-Concept-Paper-v3.1.md` §12.1.
**Reference visual:** `docs/Sight-vNext-MockB1-Toggle.svg` (7-button revision shipped 2026-05-12).
**Authorized scope:** Eisa, 2026-05-12 — *"MIG-024 The Concept Paper says MIG-024 lands the dome, the eight strata bands, the calendar rim, the 7-button toggle bar with R + T modes initially active, and wire L+C+S+A+P wedges in Layer 1, deferring only the diagnostic computations to MIG-025."*

---

## §0 · What this Architect doc IS

The territory map for MIG-024. It enumerates work clusters, surfaces design tradeoffs, lists the invariants any change must not break, and surfaces Eisa-decisions needed before Phase 2 (Plan) opens.

**This doc is NOT** the Plan. The Plan lays out commits + verification clauses + Boss-test gates after Eisa locks the decisions below.

**This doc is NOT** the Concept Paper. Concept Paper v3.1 is the design contract; this doc maps the implementation territory the contract covers.

---

## §1 · Background — what's already in flight

### §1.1 What Sight v5 inherits (Concept Paper §3 lineage)

Four prior Sight implementations ship code that this MIG either reuses, retires, or supersedes:

| Identity | Module home | MIG-024 disposition |
|---|---|---|
| **Sight v2** (`ConstellationSight2.svelte` + `sight.rs` + `constellation_sight_*` IPCs) | `src/lib/components/`, `src-tauri/src/sight.rs` | Already disabled (`SIGHT_V2_ENABLED = false`); cleanup MIG retires the on-disk code after v5 stable. |
| **Sight v3** (`src/lib/sight/v3/SightV3.svelte`) | `src/lib/sight/v3/` | Already retired (`SIGHT_V3_ENABLED = false`); 13 close-button iterations failed due to position:fixed overlay. Cleanup MIG retires. |
| **Sight v4** (`src/lib/sight/v4/SightV4.svelte`) | `src/lib/sight/v4/` | Currently active (`SIGHT_V4_ENABLED = true`). MIG-024 keeps v4 reachable as rollback target until Boss-test PASS, then cleanup MIG retires. |
| **Helper modules** in `src/lib/sight/`: `calendar-rim.ts`, `community-territory.ts`, `density-cache.ts`, `engine.ts`, `layout-cache.ts`, `palette.ts`, `projection.ts`, `universe-health.ts` | `src/lib/sight/` | **Audit each for reusability** — `palette.ts` (Suwaidi tokens) and `calendar-rim.ts` (12-month rim geometry) are likely directly reusable. `projection.ts` may be obsolete (it implemented the v3 per-mode XYZ grammar v5 revoked). `universe-health.ts` is InfraNodus-era; deferred but not needed for MIG-024 (Layer 2 may revive it). See §8 Plan-phase verification queue. |

### §1.2 What Sight v5 inherits from MIG-021v3 (CECE)

| Asset | Use |
|---|---|
| `note_meta.sources` column + frontmatter contract | Mode P data source |
| `sources_suggestions` table + Source Review panel | Sight v5 "Unsourced wedge" diagnostic prompt links to Source Review |
| 6-cataloger ensemble (`src-tauri/src/cece/`) | Indirect — Sight v5 doesn't call CECE; it reads what CECE has populated |
| `multilingual-e5-small` ONNX (Tier 1) + Qwen3-1.7B path (Tier 2, deferred to MIG-026) | Tier 2 inference is the substrate for Layers 3+4; not used in MIG-024 |
| 15-locale i18n infrastructure (`src/lib/i18n/*.json`) | Sight v5 chrome inherits the existing patterns |

### §1.3 What Sight v5 inherits from the rest of Constellation

| Source | What Sight v5 reads |
|---|---|
| `note_meta.stratum` (MIG-014) | Radial position (the load-bearing constant) |
| `note_meta.maturity` (MIG-014) | Star size |
| `note_meta.stage` (MIG-014) | Mode S wedge |
| `note_meta.created` | Mode T wedge |
| `note_links.link_type` | Mode L wedge + connector line color |
| `note_links.confidence` | Star brightness + Mode C wedge + contested detection (red dot) |
| Library membership (`libraries.json` + `note_meta.library_path`) | Mode R wedge |
| Acts data (per-note act tag — CE Layer 2; partial coverage) | Mode A wedge (with Unacted slice for missing data) |
| `note_state_history` (MIG-022 §B.1–§B.4 just shipped) | Available for future Layer 2 "growth trajectory" — not used in MIG-024 |
| SkyView component (`src/lib/components/SkyView.svelte`) | **Mount pattern reference** — flex child inside `.content-area`, close button in `+layout.svelte` header row. v3's overlay failure ⇒ v4 fix ⇒ v5 inherits the v4 pattern. |

---

## §2 · Scope

### §2.1 IN MIG-024

Per Eisa's authorization 2026-05-12 + Concept Paper v3.1 §12.1, expanded to wire all 7 modes:

- **The dome** — circular Suwaidi cream-parchment field (`#faf6e8`); 8 concentric strata band rings (L1 rim → L8 pole); faint sand grid lines (`#b8a98a` @ ~0.5–0.55 alpha); soft Milky Way wash (two `#e6dec0` radial-gradient ellipses).
- **The calendar rim** — 12-month Gregorian outer ring; current month subtly highlighted; locale-aware month labels via `dir="auto"` HTML overlay (NOT canvas-drawn text).
- **The 7-button toggle bar** — R · L · T · C · S · A · P. All seven modes wired and functional from Day 1; default first-launch mode = R; last-used mode persists per Universe via `appSettings.sight.lastMode`.
- **The four constants** — radial position (strata), size (maturity), brightness (confidence), color (state). NEVER change with mode toggle. Encoded per Concept Paper §5.3 / §7.
- **Per-mode wedge re-slicing** — `azimuthForMode(mode, note, context) → angle` dispatch; 600 ms ease animation interpolating only angular position; stars stay in their stratum band. All 7 modes:
  - **R** Regions — wedges = Library set, sized by note count (largest first)
  - **L** Link Types — wedges = 9 typed-link kinds + Untyped
  - **T** Time — wedges = 12 months (current highlighted)
  - **C** Confidence — wedges = hypothesis · evidence · established · contested
  - **S** Stages — wedges = 6 lifecycle stages (Spark → Birth → Growth → Maturity → Dormancy → Archival)
  - **A** Acts — wedges = 5 Acts (Observation → Connection → Tension → Synthesis → Conviction) + Unacted
  - **P** Provenance — wedges = top-level horizontal-axis families from CECE's live taxonomy + Unsourced
- **The connector-line layer** — faint at rest (~0.10–0.15 alpha); color-coded by 9 typed-link kinds per Concept Paper §5.4; brighten to ~0.85 alpha on hover/select; other lines stay faint (Principle 6 reframed).
- **Hover / select / Esc interactivity** — hover star → tooltip + edge brighten; click → side panel + persistent edge brighten; click background or Esc → clear.
- **Side panel for selected-star detail** — slide-in right panel showing note title, strata, maturity, stage, sources (if classified), top incident links. "Open in editor" button for the handoff to 360.3D / NotePane (Eisa's load-bearing line: Sight = whole universe; 360.3D = single note).
- **SQLite layout cache** (`sight_v5_layout` table) — write-time derived per `(library_set_hash, mode, note_id)`; invalidated on `note_meta` / `note_links` writes via existing trigger pattern; warm read in <50 ms for warm cache.
- **Lazy mount + idle-prewarm** — Sight v5 component mounts only on dock-button click; layout cache warmed via `requestIdleCallback` after `boot:hydrated`. **Zero boot impact.**
- **Feature flag** — new `SIGHT_V5_ENABLED` const in `src/lib/sight/engine.ts`; mutually exclusive with `SIGHT_V4_ENABLED`. Initially gated to `false`; flips to `true` only after Boss-test PASS.
- **i18n stub** — every new chrome string goes through `$t()`; en + ar populated; 13 other locales backfilled in MIG-024 §6 (parallel agents per the V3-§10.D pattern).
- **Help topic stub** — new `docs/help.uConstellation.World/Sight v5/Sight v5.md` introducing Layer 1 (NOT the comprehensive help topic that lands with Layer 4).

### §2.2 NOT in MIG-024 (deferred to MIG-025+)

- **Layer 2 — diagnostic computations.** Health signals (strata distribution / source diversity / confidence balance / growth trajectory / contested resolution / acts coverage) and the "Findings" side-panel surface. MIG-025.
- **Layer 3 — recommendation engine.** Qwen3-1.7B + GBNF grammar wiring for converting findings into named actions. V3-§7.b llama.cpp work lands here. MIG-026.
- **Layer 4 — coaching mode.** Conversational chat panel with Constellation-aware actions. MIG-027.
- **Sight v4 retirement.** v4 stays reachable through MIG-024 as rollback target. Cleanup MIG retires after Eisa confirms v5 stable across multiple sessions.
- **Lenses.rs::apply_lens deletion.** Cleanup MIG. (Already on the queue.)
- **Mock B1 SVG further evolution** (PJ-051 housekeeping). Triggered by future v3.x Concept Paper revisions, not by MIG-024.
- **The full polished help topic + User Manual chapter for Sight v5** (lands with Layer 4 / MIG-027).

---

## §3 · Cross-cutting invariants — must NOT break

These invariants survive MIG-024 unchanged. Any code change that touches them requires explicit acknowledgment and remediation.

| # | Invariant | Source | How MIG-024 preserves it |
|---|---|---|---|
| **I-1** | **File-over-app**: `.md` files on disk are the source of truth | CLAUDE.md Architecture | MIG-024 reads only; no file writes. |
| **I-2** | **Performance Rule 1**: zero perceptible keystroke lag in editors | CLAUDE.md Rule 1 | Sight v5 lazy-mounts on dock click; doesn't touch the editor hot path. |
| **I-3** | **Performance Rule 8**: write-time derivation, not read-time recomputation | CLAUDE.md Rule 8 | `sight_v5_layout` cache maintained via SQLite triggers on `note_meta`/`note_links` writes; reads are SELECTs. |
| **I-4** | **Editor Parity Rule**: NotePane/FocusPane share rendering | CLAUDE.md | MIG-024 doesn't touch the editor. |
| **I-5** | **Living Link Architecture**: 9 typed-link kinds, 4 confidence levels | Concept Paper §5.4 | MIG-024 reads `note_links`; doesn't extend the schema. |
| **I-6** | **CECE 6-cataloger contract** | MIG-021v3 | MIG-024 reads `note_meta.sources`; doesn't call catalogers. |
| **I-7** | **i18n parity** across 15 locales for chrome strings | CLAUDE.md | Every new chrome string through `$t()`; en+ar populated in MIG-024 §1; 13-locale backfill in §6. |
| **I-8** | **Boot perf**: ≤ 6 s critical path on 7,636-note universe | CLAUDE.md + 2026-04-15 discipline | Sight v5 lazy-mounted; idle-prewarm after `boot:hydrated`. |
| **I-9** | **Five-core-functions invariant**: Sight does not duplicate Search Hub / OrgChart / Sky View / Map | Concept Paper §10 + 2026-04-13 rule | MIG-024 honors all 9 boundary entries in Concept Paper §10. |
| **I-10** | **The four constants stay constant across mode toggles** | Concept Paper §7 | `azimuthForMode` is the ONLY function that varies per mode; radial / size / brightness / color come from invariant per-note properties. |
| **I-11** | **Spatial memory across mode toggles** — same star at same stratum band in every mode | Concept Paper §6.2 | `radiusForNote(stratum)` is mode-agnostic. Mode toggle interpolates angular position only. |
| **I-12** | **~5-second comprehension threshold for first-time users** | Concept Paper §13 #2 | Validated via Boss-test Stage 7 with the legend visible per Mock B1. |
| **I-13** | **Sight = whole universe; 360.3D = single note** | Concept Paper §10 + Eisa 2026-05-09 | Side panel "Open in editor" hands off to 360.3D / NotePane; Sight does NOT deepen into per-note view. |
| **I-14** | **Mock B1 visual contract** (Suwaidi palette, 7 buttons, legend wording) | Concept Paper §5.1 | Production code reconciles pixel-for-pixel against the SVG within Suwaidi palette tokens. |

---

## §4 · Audit of existing infrastructure

### §4.1 Reusable as-is (high confidence)

| Asset | Path | Reuse |
|---|---|---|
| Suwaidi color palette | `src/lib/sight/palette.ts` (audit needed) | Sight v5 uses identical hex tokens |
| Calendar rim geometry | `src/lib/sight/calendar-rim.ts` (audit needed) | 12-month rim layout reusable; verify month label rendering uses HTML overlay (not canvas) per v3 spec invariant 12 |
| SkyView mount pattern | `src/lib/components/SkyView.svelte` | Sight v5 mirrors this pattern (flex child in `.content-area`, close button in layout header) |
| Universe data IPCs | existing `note_meta` / `note_links` queries | Read-only; no new IPCs needed for the per-note data |
| 15-locale i18n machinery | `src/lib/i18n/*.json` + `src/lib/i18n.ts` | New `sight.v5.*` keys go through the existing `$t()` machinery |
| Layout cache pattern | `src/lib/sight/layout-cache.ts` (audit needed) | If v3/v4's cache pattern is sound, v5 inherits the same shape with v5-specific keys |

### §4.2 Likely obsolete or needs significant refactor

| Asset | Path | Why obsolete |
|---|---|---|
| v3 per-mode XYZ projection | `src/lib/sight/projection.ts` (audit needed) | v5 revokes per-mode XYZ; needs to be either retired or reduced to the strata→radius constant projection |
| Universe health (InfraNodus-era) | `src/lib/sight/universe-health.ts` (audit needed) | InfraNodus heritage dropped; deferred for MIG-025 (Layer 2 may revive parts) |
| Community territory rendering | `src/lib/sight/community-territory.ts` (audit needed) | Louvain-community color rendering is dropped; v5 colors are state-only |
| Density-cache | `src/lib/sight/density-cache.ts` (audit needed) | TF-IDF density may stay (Milky Way wash is part of Mock B1); verify the cache shape is sound |

### §4.3 New modules MIG-024 needs to ship

| Module | Path | What |
|---|---|---|
| **Engine flag bump** | `src/lib/sight/engine.ts` | Add `SIGHT_V5_ENABLED = false` (initially); mutually exclusive with V4 |
| **Sight v5 component** | `src/lib/sight/v5/SightV5.svelte` | The mounted component. Reuses v4's mount pattern. |
| **Dome geometry** | `src/lib/sight/v5/dome.ts` | 8 strata band radii, calendar rim layout, Milky Way ellipse positions. Pure functions. |
| **Mode dispatcher** | `src/lib/sight/v5/modes.ts` | `azimuthForMode(mode, note, context) → angle` for all 7 modes. Pure functions; one file. |
| **Render pipeline** | `src/lib/sight/v5/render.ts` | Canvas 2D + D3-zoom; two-layer (base + focus overlay); per Mock B1 |
| **Side panel** | `src/lib/sight/v5/SightV5SidePanel.svelte` | Selected-star detail; "Open in editor" handoff |
| **Layout cache (Rust)** | `src-tauri/src/sight_v5.rs` (or extend existing) | `sight_v5_layout` table CRUD + write-time triggers + IPC for warm-cache reads |
| **i18n keys** | `src/lib/i18n/{en,ar}.json` (+ 13 locales backfilled in §6) | `sight.v5.*` block: chrome strings, mode names, side-panel labels, tooltip text |
| **Help topic stub** | `docs/help.uConstellation.World/Sight v5/Sight v5.md` (+ 14 locales) | Brief introduction; full help lands with MIG-027 |

---

## §5 · Six work clusters

The MIG-024 work decomposes into six clusters, each landable as one or more commits. Effort estimates are in agent-time; cumulative estimate ~2-3 weeks per Eisa's authorization.

| Cluster | What | Effort | Risk |
|---|---|---|---|
| **§1 — Engine flag + module skeleton** | `SIGHT_V5_ENABLED` const; `src/lib/sight/v5/` directory; SightV5.svelte stub mounted via dock button; Settings → Sight section gated on the flag. v4 stays active. | ½ day | Low |
| **§2 — Layout cache (SQLite + IPC)** | `sight_v5_layout` table in init_db migration; write-time triggers on `note_meta`/`note_links`; `cece_get_sight_v5_layout` IPC returning per-note `(stratum, mode, wedge_index)`; cache warming via `requestIdleCallback` after boot. Backfill for existing universes. | 2-3 days | Medium (schema migration on existing universes; write-trigger correctness) |
| **§3 — Dome geometry + Canvas 2D render layers** | Dome (8 bands + Milky Way + calendar rim); Canvas 2D base layer (drawn once per cache-warm cycle); Canvas 2D focus overlay (redrawn on hover/select); D3-zoom for pan/zoom; HTML overlays for month labels (not canvas-drawn); Mock B1 pixel-fidelity verification. | 4-5 days | Medium-high (perf budgets must hit; calendar-rim label geometry is fiddly with RTL) |
| **§4 — Seven mode toggles + per-mode wedge dispatch** | `azimuthForMode(mode, note, context)` for R / L / T / C / S / A / P; toggle bar UI with three states (active gold / ready / dimmed); 600 ms ease animation interpolating angular position only; mode persistence per `appSettings.sight.lastMode`. | 3-4 days | Medium (each of 7 modes has per-mode data shape; A and P need empty-data-state handling) |
| **§5 — Stars + connectors + side panel + interactivity** | Star rendering with maturity sizing + confidence brightness + state coloring; connector lines per 9 typed-link kinds; hover/select/Esc state machine; side panel slide-in with note detail + "Open in editor" handoff. | 3-4 days | Medium (perf at 7,636 stars; hover hit-testing precision) |
| **§6 — i18n stub + help stub + dock button + Settings** | All new chrome strings through `$t()`; en + ar populated; 13-locale backfill (parallel agents); dock button in left rail; Settings → Sight section; help topic stub in 15 locales. | 2-3 days | Low (well-trodden i18n + docs path) |

**Total: ~15-21 agent-days across 6 clusters.** Each cluster ends with at least one verification clause; clusters §3, §4, §5 each have a Boss-testable observable.

---

## §6 · Recommended scope = all six clusters in one MIG

Per Eisa's 2026-05-12 authorization (*"wire L+C+S+A+P wedges in Layer 1, deferring only the diagnostic computations to MIG-025"*), MIG-024 takes the full six-cluster scope. No clusters split out into a separate MIG. **Boss-test gate fires once after §6 lands** — not per-cluster, since the dome isn't fully usable until §3+§4+§5 are all in place.

---

## §7 · Decisions for Eisa to lock before Plan opens

Seven decisions surface from this Architect pass. Recommendations attached; Eisa locks per-line.

### D-V1 — Render technology

The Mock B1 shows ~7,600 stars + the dome chrome. Three options:

- (α) **Canvas 2D + D3-zoom** — v4's choice; proven; performant up to ~50k elements. Recommended.
- (β) **SVG** — DOM-friendly; cleaner for hover hit-testing; slow at >2k elements.
- (γ) **WebGL via PixiJS** — overkill for 7,600 stars; reintroduces Sky View's complexity into Sight.

**Rec: α.** The v4 pivot already proved Canvas 2D + D3-zoom works for this scale; reuse the proven path.

### D-V2 — Side panel placement

When a star is selected:

- (α) **Slide-in right** — Sight v4 pattern; matches Outgoing/Backlinks pane shape.
- (β) **Anchored bottom** — preserves more dome real estate for users who want both panel and dome visible.
- (γ) **Floating tooltip-sized card** — minimal footprint; no persistent panel.

**Rec: α.** Matches user's existing mental model from Sight v4 + the Backlinks/Outgoing panel pattern. Slide-in width ~300 px; collapsible.

### D-V3 — Mode persistence per-Universe vs per-Library

Concept Paper §6.1 says per-Universe (`appSettings.sight.lastMode`).

- (α) **Per-Universe** (Concept Paper default).
- (β) **Per-Library** — different libraries might benefit from different default modes (e.g., a "Quotes" library defaults to mode P; a "Daily Notes" library defaults to mode T).

**Rec: α.** Concept Paper default. Per-Library is a future PJ if Eisa requests after using v5.

### D-V4 — Layout cache strategy

For 7,636 notes × 7 modes:

- (α) **Single `(note_id, library_set_hash)` row + per-mode reprojection at render time** — smaller storage; mode-switch reprojection in JS.
- (β) **Per-mode rows: `(note_id, mode, library_set_hash)` × 7** — larger storage; mode-switch is a single SELECT.

**Rec: α.** The per-mode azimuth computation is cheap (it's just a lookup or a date-modulo); not worth 7× SQLite rows. Mode-switch reprojection runs in JS during the 600 ms animation anyway.

### D-V5 — v4 retire timing

- (α) **Keep v4 reachable through MIG-024** (rollback if MIG-024 Boss-test fails). Cleanup MIG retires v4 after Eisa confirms v5 stable.
- (β) **Hide v4 immediately when v5 ships** — single button, single component visible.

**Rec: α.** v3's retirement only after v4 was Eisa-confirmed-stable was the right pattern; honor it for v5. v4 hides via Settings toggle (developer-only) once v5 default ships.

### D-V6 — Mode P empty-state UX

If a Universe has classified < 5 % of notes:

- (α) **Render the dome with a giant Unsourced wedge + a CTA "Classify some notes via Source Review"** — the visual itself is the prompt.
- (β) **Disable mode P entirely** — show the dimmed dashed-border button with tooltip "Classify some notes first."
- (γ) **Render only the classified notes (sparse dome)** — honest but visually empty.

**Rec: α.** Per Concept Paper §6 + §8.4: "the visible wedge becomes a to-do list." This is exactly the visual-as-prompt the Concept Paper specifies.

### D-V7 — Settings toggle for "always-on labels"

The v3 Concept Paper §4.1 had a Settings toggle for always-on constellation labels (default: hover/select only). v5 doesn't have constellations (Louvain communities dropped) — but still has hover-to-show note titles.

- (α) **Always-on toggle in Settings → Sight** (defaults to off; on shows note titles for top-N stars by maturity).
- (β) **Hover-only, no toggle** — keep chrome minimal; user can use side panel for detail.

**Rec: β.** Concept Paper §5.6 explicitly forbids "numerical scores out of 100" + civilizational labels in chrome — keep Sight chrome quiet. If users want labels they can hover. Future PJ if Eisa wants the toggle.

---

## §8 · Plan-phase verification queue

Five Architect-phase claims that the Plan must verify in code before proceeding. Each is a "verify before drafting Phase X" gate:

1. **`palette.ts` audit** — confirm Suwaidi tokens are unchanged from Mock B1 SVG. If drift, reconcile.
2. **`calendar-rim.ts` audit** — confirm 12-month rim geometry uses HTML overlay for labels (not canvas-drawn text) per v3 invariant 12. Verify RTL behavior.
3. **`projection.ts` audit** — confirm what's reusable; what's the v3 per-mode XYZ that v5 revokes; what new strata→radius constant function is needed.
4. **`layout-cache.ts` audit** — confirm the cache key pattern is sound for v5's per-`(note_id, library_set_hash)` shape per D-V4.
5. **SkyView mount pattern audit** — confirm the exact flex-child + close-button-in-+layout.svelte pattern v5 must mirror. Document the contract between SightV5.svelte and `+layout.svelte`.

These audits are Plan-phase work, not Architect-phase. The Plan opens with them.

---

## §9 · Risk register

| ID | Risk | Severity | Mitigation |
|---|---|---|---|
| **R-1** | 7,636-star dome render exceeds 500 ms cold-cache budget | High | Per Concept Paper §11.1: two Canvas layers (base drawn once, focus overlay redrawn on interaction), `requestIdleCallback` prewarm, RangeSetBuilder-style sorted insert |
| **R-2** | Mode-switch animation janky on lower-end machines | Medium | Pure JS angular interpolation (no IPC during animation); 600 ms ease; if perf marginal, drop frames gracefully (better to skip frames than to stutter) |
| **R-3** | Calendar rim label rendering breaks in RTL locales | Medium | HTML overlay with `dir="auto"` per Concept Paper / v3 invariant 12; avoid canvas-drawn text for any rim chrome |
| **R-4** | `sight_v5_layout` table grows large and slows boot read | Low | Per-Universe row count = note count × 1 (per D-V4); 7,636 rows is trivial; cache invalidation via existing trigger pattern |
| **R-5** | Mode P empty-state on a fresh universe shows nothing usable | Medium | D-V6.α: giant Unsourced wedge with CTA → Source Review; visual-as-prompt |
| **R-6** | Existing v4 users' `appSettings.sight.lastMode` doesn't match v5's seven modes | Low | v5 reads existing `lastMode`; if value is unrecognized (e.g., a v4 mode name), fall back to R |
| **R-7** | Mock B1 SVG and production code drift over time | Medium | PJ-051 already filed (Pending Jobs v1.11); future Concept Paper revisions trigger parallel SVG edits |
| **R-8** | i18n backfill of 13 locales × ~30 sight.v5.* keys = ~390 translations introduces new keys missing from old locales | Low | Use the V3-§10.D parallel-agent pattern; verify JSON parse + key presence post-backfill |
| **R-9** | Side panel "Open in editor" handoff conflicts with the editor's own mount lifecycle | Medium | Mirror the existing Backlinks/Outgoing panel's "open note" handler exactly; don't invent a new IPC |
| **R-10** | v4 → v5 cutover fails Boss-test; need rollback | Low | D-V5.α keeps v4 reachable; flip the engine flag; no schema rollback needed |

---

## §10 · Test surface

### §10.1 Rust tests (cargo test --lib sight_v5)

- `sight_v5_layout_table_creation` — schema present after init_db
- `sight_v5_layout_write_trigger` — note_meta UPDATE invalidates layout cache
- `sight_v5_layout_query_returns_per_note_data` — the IPC payload shape matches frontend expectations
- `sight_v5_layout_handles_unsourced_notes` — empty `note_meta.sources` returns "unsourced" wedge for mode P

### §10.2 Frontend tests (svelte-check + manual)

- svelte-check: zero new errors on SightV5.svelte / SightV5SidePanel.svelte
- `azimuthForMode(R, note)` returns correct angle for known library / note pairs
- `azimuthForMode(T, note)` returns correct angle for known created-month / note pairs
- ... per mode
- `radiusForNote(stratum)` returns same value regardless of mode passed

### §10.3 Boss-Test Gate (after §6)

7 stages per the Testing Instructions Rule:

- **Stage 0** — Verify NSIS bundle mtime is post-MIG-024 ship.
- **Stage 1** — Open Sight v5 from the dock; dome renders correctly (8 bands, calendar rim, stars at correct strata); ~5-second comprehension threshold validated.
- **Stage 2** — Toggle modes R → L → T → C → S → A → P; verify wedges re-cut without breaking spatial memory (same star sits at same stratum band in every mode).
- **Stage 3** — Hover a star; verify connector lines brighten + tooltip shows.
- **Stage 4** — Click a star; verify side panel opens with note detail; click "Open in editor" → handoff to NotePane works.
- **Stage 5** — Esc clears selection; click background also clears.
- **Stage 6** — Mode P empty-state on a sparse-Sources universe shows the giant Unsourced wedge + CTA per D-V6.α.
- **Stage 7** — Close button works; reopen Sight v5 → restored to last mode + clean state.

---

## §11 · What's next

After Eisa locks D-V1 through D-V7: **Phase 2 Plan opens** — `lab/reports/MIG-024-SIGHT-V5-LAYER-1-VISUAL-FOUNDATION-PLAN.md` lays out the 6-cluster phase sequence with verification clauses, files-touched lists, and the Boss-Test Gate's 7 stages spelled out per the Testing Instructions Rule.

After the Plan: **Phase 3 Build** cascades through the clusters per Plan-Approval-Equals-Build-Approval, stopping only at the Boss-Test Gate and on genuine architectural surprise.

After Build PASS: **Phase 4 Audit** — three-agent integration check (mirroring the MIG-022 §N pattern that ran in parallel with this Architect doc). MIG-024 close-out commit + orientation v-bump + Pending Jobs update.

Then MIG-025 (Layer 2 diagnostic) opens.

---

**End of MIG-024 Architect.** Awaiting Eisa's lock on D-V1 through D-V7 before Phase 2 Plan opens.
