# MIG-019 — Sight v3 Density + Time + Search (Architect)

**Migration**: MIG-019 (within PJ-038)
**Phase position**: 2 of 3 in the v3 build trajectory (MIG-018 ✅ → **MIG-019** → MIG-020).
**Effort class**: Single MIG, multi-phase build (6 phases). Estimated 1–2 sessions.
**Reference design**: `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` §2 (visual grammar) + §5 (PJ-035 Milky Way absorption) + §6 (universe-health at-a-glance) + §7 (calendar rim) + §9.2 (phase scope).
**Predecessor**: `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-AUDIT.md` (closed Done 2026-05-07).

---

## 1 · Goal

Layer the **density / time / search / health** dimensions onto the v3 projection foundation. After MIG-019:

1. **Milky Way density wash** (PJ-035 absorbed) — diffuse band of high-similarity texture between/under stars, computed from TF-IDF cosine similarity. The InfraNodus-defining mechanic that v2 never shipped.
2. **Calendar rim** — outer ring of month markers around the dome. Gregorian default; users add other calendars via Settings. Click a month to filter stars by creation/last-traversed date.
3. **Universe-health card** in the side panel — Modularity + Dominance + Entropy + Connectivity metrics readable at a glance with green/yellow/red badges.
4. **Full search integration** — Search Hub query → matched stars flare (size-pulse + brightness boost), territories halo, connector lines between matches brighten. Esc + click-background clears.
5. **Always-on constellation labels toggle** — defaults stay hover-only; Settings → Sight → "Show constellation labels at rest" flips the behaviour.

End-state of MIG-019: v3 reaches feature parity with the v1.1 paper's §3.3 third edge type AND §6 universe-health-at-a-glance. The visual grammar is complete; what remains for MIG-020 is layer peeling (PJ-036 magnitude slider) and v2 retirement.

End-state of MIG-019 NOT-yet:
- Magnitude slider / layer peeling (PJ-036) — MIG-020.
- v2 retirement — MIG-020 (after Boss confirms v3 stable across multiple sessions).
- PJ-037 (Map↔Sight integration) — REJECTED, not in any v3 MIG.

---

## 2 · Surfaces this MIG creates / touches

### 2.1 New Rust modules / additions

| File | Purpose | Approx LOC |
|---|---|---|
| `src-tauri/src/sight_layout.rs` (existing, +400) | Adds `compute_similarity_field` function + new IPC `constellation_sight_v3_similarity_field`. Reads note bodies via `note_meta.path`, tokenizes, builds sparse TF-IDF, computes pairwise cosine for candidates above threshold via inverted-index lookup, returns list of high-similarity `(path_a, path_b, similarity)` triples. | ~400 added |

The compute lives in `sight_layout.rs` (not a new module) because it shares the same library-set-hash + graph-version machinery and operates on the same note set. Co-location keeps the Sight v3 compute surface in one Rust module.

### 2.2 Schema additions

In `src-tauri/src/search.rs::init_db`:

```sql
-- MIG-019 §2A: TF-IDF similarity edges cache.
-- One row per high-similarity pair above the similarity threshold,
-- per (library_set_hash, graph_version). Mirrors sight_v3_layout's
-- caching pattern.
CREATE TABLE IF NOT EXISTS sight_v3_similarity_edges (
    note_path_a       TEXT NOT NULL,
    note_path_b       TEXT NOT NULL,
    library_set_hash  TEXT NOT NULL,
    graph_version     INTEGER NOT NULL,
    similarity        REAL NOT NULL,
    PRIMARY KEY (note_path_a, note_path_b, library_set_hash, graph_version)
);
CREATE INDEX IF NOT EXISTS idx_sight_v3_similarity_libset_ver
    ON sight_v3_similarity_edges(library_set_hash, graph_version);
```

Bumps `SIGHT_V3_SCHEMA_VERSION` from 1 to 2 (the existing dead-code constant becomes live: any pre-MIG-019 cache rows get invalidated by version comparison; new compute populates the new schema).

### 2.3 New frontend modules

| File | Purpose | Approx LOC |
|---|---|---|
| `src/lib/sight/similarity-cache.ts` (NEW) | Frontend wrapper over `constellation_sight_v3_similarity_field` IPC. Module-level Map cache. Mirrors layout-cache.ts. | ~80 |
| `src/lib/sight/calendar-rim.ts` (NEW) | Pure-TS calendar-rim geometry: month-arc segments around a circle, Gregorian + Hijri label generators, hit-testing for month hover/click. | ~200 |
| `src/lib/sight/universe-health.ts` (NEW) | Pure-TS computation of M+D+E+C metrics from cluster + edge data. Returns numeric values + threshold-based status (green/yellow/red). | ~120 |

### 2.4 SightV3 component additions

`src/lib/sight/v3/SightV3.svelte` extended:

| New container layer | Purpose | Insertion point |
|---|---|---|
| `milkyWayContainer` | Soft alpha-blended lines between high-similarity pairs. Pixi `BlurFilter` smooths them into a band. Toggleable via Settings (default ON). | Between `app.stage` background and `territoryContainer` (z-order: deepest layer above sky). |
| `calendarRimContainer` | Month-arc segments around the perimeter + month labels. Multiple concentric rings if user enabled multiple calendars. | Above all base layers, below DOM tooltips. |

| New interaction | Purpose |
|---|---|
| Month hover/click on calendar rim | Filter stars by creation date / last-traversed within that month range. |
| Search match flare | Matched stars `Graphics.tween` size pulse + brightness boost. Territories with matches get halo glow. |
| "Show labels at rest" Settings toggle | When ON, render `Text` for each community territory at its centroid. When OFF (default), hover/click reveal pattern. |

### 2.5 SightV3SidePanel additions

| New section | Purpose |
|---|---|
| Universe-health card | M / D / E / C numeric values + green/yellow/red badge per Concept Paper v1.1 §3.4 thresholds. Always shown when no star is selected; tucks below note details when a star is selected. |

### 2.6 Settings additions

| Key | Default | Purpose |
|---|---|---|
| `appSettings.sight.alwaysOnLabels` | `false` | When true, constellation labels render at rest instead of hover-only. |
| `appSettings.sight.calendarSystems` | `['gregorian']` | Array of enabled calendar systems. Multiple systems → concentric rim layers. |
| `appSettings.sight.showMilkyWay` | `true` | Right-click toggle to hide the density wash for cleaner views. |

Settings → Sight section UI extends with three new controls (always-on labels toggle, calendar systems multi-checkbox, Milky Way visibility toggle).

### 2.7 i18n

Per-locale keys to add (en + ar full strings; 13 placeholder English values per PJ-014 backfill convention):

- `settings.sight.alwaysOnLabels.label`, `.hint`
- `settings.sight.calendarSystems.label` + per-system labels (`gregorian`, `hijri`, `solarHijri`, `hebrew`)
- `settings.sight.showMilkyWay.label`, `.hint`
- `sightV3.sidePanel.universeHealth` + sub-labels (`modularity`, `dominance`, `entropy`, `connectivity`, `healthy`, `caution`, `imbalanced`)
- `sightV3.calendar.{months}` (12 month names per calendar system; only Gregorian + Hijri shipped this MIG, others queue under PJ-014)
- `sightV3.search.flareLabel` (accessibility label for the flare animation)

Inject script: `lab/sight_v3_i18n_inject_f.py` (similar pattern to §1C/§1D/§1E injects).

### 2.8 Help docs

| File | Edit |
|---|---|
| `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` | The 🚧 banner from MIG-017/MIG-018 stays but its body extends with the new features: Milky Way + calendar rim + universe-health card + search behavior + always-on labels toggle. |
| `docs/User Manual.md` (en) | New paragraphs in the Sight v3 chapter describing the four new features. |

---

## 3 · Invariants — what must not break

| # | Invariant | Why it matters |
|---|---|---|
| 1 | **MIG-018 stays intact.** All §1A–§1F deliverables (star rendering, territories, faint connector lines, Lambert/stereographic toggle, hover/click/double-click, side panel skeleton, Suwaidi palette) keep working. | MIG-019 is purely additive; no v3 regression allowed. |
| 2 | **Determinism.** TF-IDF compute deterministic given the same `(library_paths, graph_version, note_bodies)`. Universe-health metrics deterministic given the same Louvain output. | Spatial-memory grammar requires it. |
| 3 | **Boot performance.** No new IPC on the boot critical path. TF-IDF compute fires only when v3 is opened (lazy), not at boot. | Boot ≤ 6 sec stays ≤ 6 sec. |
| 4 | **Write-time derivation (Rule 8).** `sight_v3_similarity_edges` cache invalidated by triggers on `note_meta` body changes (or version bump on save). Reads are cheap SELECTs. | CLAUDE.md Rule 8. |
| 5 | **No `$effect` loops.** New `$effect`s tracking `$appSettings.sight.{alwaysOnLabels, calendarSystems, showMilkyWay}` follow the same pattern as MIG-018's projection-toggle effect — read-only inside, no reactive writes. | Performance Rule 2. |
| 6 | **No memory leaks.** New Pixi containers (`milkyWayContainer`, `calendarRimContainer`) destroy with the parent on `onDestroy`. Pixi `BlurFilter` instances must be released. | Performance Rule 4. |
| 7 | **i18n integrity.** All 15 locales receive the new keys. | SO #6. |
| 8 | **RTL parity.** Calendar rim labels render RTL when locale is RTL. Hijri month names render correctly under RTL. Universe-health card uses `dir="auto"`. | Language-First by Design. |
| 9 | **No regression in other surfaces.** Sky View / OrgChart / Map / Index / SearchHub / Inspector360 untouched. v2 stays disabled (SIGHT_V2_ENABLED unchanged). | MIG-017 + MIG-018 contracts. |
| 10 | **M11 zero-diff.** `git diff src-tauri/src/lexicon/` empty. | Lexicon engine orthogonal. |
| 11 | **TF-IDF tokenizer consistency.** Use Constellation's existing FTS5 tokenizer (Arabic-aware, multi-script). Don't roll a custom tokenizer that gives different similarities than search would expect. | Cross-cutting — search and similarity should agree on what "same word" means. |
| 12 | **Performance budget per Concept Paper v1.1 §8.1.** First-toggle latency ≤ 500ms cold cache; warm-cache toggle ≤ 50ms; idle-Sight per-frame ≤ 1ms; memory ≤ 40MB above app baseline. | Boss-test verifies. |
| 13 | **PJ-038 trajectory honored.** PJ-036 (layer peeling) NOT shipped — MIG-020. PJ-037 (Map↔Sight) NOT touched. | Concept Paper v1.1 §9. |
| 14 | **Settings round-trip.** New `sight.alwaysOnLabels`, `sight.calendarSystems`, `sight.showMilkyWay` round-trip cleanly. Existing `sight.projection` preserved. | Standard appSettings discipline. |

---

## 4 · Drift map

| New thing | Consumers known at MIG-019 commit | Future consumers expected | Risk |
|---|---|---|---|
| `sight_v3_similarity_edges` table | `compute_similarity_field` IPC (read+write); new frontend `similarity-cache.ts` (read) | None expected | Low |
| `constellation_sight_v3_similarity_field` IPC | `similarity-cache.ts` (frontend wrapper); `SightV3.svelte` `onMount` after layout fetch | None expected | Low |
| `appSettings.sight.alwaysOnLabels` | SettingsModal Sight section (read+write); SightV3.svelte `$effect` for label visibility | None expected | Low |
| `appSettings.sight.calendarSystems` | SettingsModal Sight section (read+write); SightV3.svelte `$effect` for rim layers | MIG-020 might add per-system settings (e.g., week start day) — namespace pre-allocated | Low |
| `appSettings.sight.showMilkyWay` | SettingsModal Sight section (read+write); SightV3.svelte `$effect` for visibility; right-click context menu | None expected | Low |
| `calendar-rim.ts` module | SightV3.svelte (rim render + hit-test); new helper functions for `monthArcSegments`, `gregorianLabels`, `hijriLabels` | MIG-020 might add solar-Hijri / Hebrew labels — pre-allocated | Low |
| `universe-health.ts` module | SightV3SidePanel.svelte (universe-health card); pure-TS, no Pixi dep | None expected | Low |
| `similarity-cache.ts` module | SightV3.svelte `onMount` (one consumer); export `invalidateSimilarity()` for future graph-mutation invalidation (parallel to `invalidateLayout()`) | None expected (invalidation wiring deferred to MIG-020 along with PJ-036's full graph-aware invalidation) | Low |
| `SIGHT_V3_SCHEMA_VERSION = 2` | `init_db` schema-version comparison logic (becomes live); old version 1 rows in `sight_v3_layout` invalidated by version bump | None expected | Medium — bumping schema invalidates all existing v3 caches; first toggle after upgrade does cold compute |

LL-023 risk class: **medium**. The schema-version bump is the only non-trivial drift; mitigated by §1B's "always recompute on cache miss" semantics — first-toggle is cold and slow once, then warm forever. Acceptable because:
- v3 was just shipped in MIG-018 (small install base; barely any cached data).
- Cold compute is sub-second on Boss-scale graphs.

---

## 5 · Migration-path map

| # | Scenario | Behavior |
|---|---|---|
| 1 | Fresh install (no settings.json, no DB) | DEFAULT_SETTINGS provides `sight.{projection:'lambert', alwaysOnLabels:false, calendarSystems:['gregorian'], showMilkyWay:true}`. `init_db` creates `sight_v3_similarity_edges` table empty. First v3 toggle: cold MDS (from MIG-018) + cold TF-IDF compute, both persist. |
| 2 | Existing user, has MIG-018 layout cached but no MIG-019 data | Settings merge picks up new keys from DEFAULT_SETTINGS. Schema-version bump invalidates `sight_v3_layout` (because version went from 1 to 2); cold recompute MDS + TF-IDF on first v3 toggle. ≤500ms hit acceptable. |
| 3 | Existing user, has stale `sight.alwaysOnLabels: false` saved | Honored: hover-only labels (default behavior). Same as MIG-018 §1E. |
| 4 | Existing user, has `sight.calendarSystems: ['gregorian','hijri']` saved | Both rims render concentrically; Gregorian innermost. |
| 5 | Mid-compute interruption (TF-IDF on 30k notes) | `sight_v3_layout_cursor`-equivalent cursor for similarity (or just always-recompute semantics from MIG-018 §1B). Resume on next toggle. |
| 6 | Rollback to MIG-018 (revert MIG-019 commits) | `sight_v3_similarity_edges` table stays (idempotent CREATE) but is unused. Schema-version drops back to 1 — cached MIG-018 rows for version 2 become stale and the next compute regenerates at version 1. Cosmetic cache pollution; no data loss. |
| 7 | Forward to MIG-020 | Magnitude slider state persisted in `appSettings.sight.magnitudeThreshold` (new key). Layer-peeling client-side recompute uses the cached MDS coords (no Rust trip). |
| 8 | Note body change → similarity invalidation | When user saves a note, `note_meta` row updates. Triggers on `note_meta` (body or content-hash column) bump `sight_v3_graph_version`. Next v3 toggle recomputes both MDS and similarity. (Or: lazy-invalidate-on-toggle if real-time accuracy isn't worth the per-save cost.) |

---

## 6 · Acceptance criteria — Boss-test gate (§2E)

1. **Milky Way visible.** Open Sight v3. A faint diffuse band of soft texture connects regions with high content similarity. Density visibly varies (sparser in isolated communities, denser in cross-cutting themes). Right-click → "Hide Milky Way" → band disappears; right-click again → "Show Milky Way" → band returns.
2. **Calendar rim visible.** Outer ring of 12 Gregorian month markers (English by default; localized to interface locale). Current month visibly highlighted.
3. **Calendar rim — Hijri toggle.** Settings → Sight → Calendar systems → enable Hijri. Outer ring gains a second concentric layer with Hijri month names. Disable Hijri → second ring disappears.
4. **Calendar rim — month hover/click.** Hover a Gregorian month → preview filter (notes from that month flare; others dim). Click a month → persistent filter. Click again or click chart background → clear filter.
5. **Universe-health card.** Open Sight v3 with no star selected. Side panel (or inline indicator) shows M / D / E / C with numeric values + green/yellow/red badge per Concept Paper §3.4 thresholds. Numbers match v2's `lensHealth` computation (cross-validate by toggling SIGHT_V2_ENABLED + SIGHT_V3_ENABLED both true and comparing).
6. **Search flare.** Open Search Hub, type a note title. Matched stars flare (size pulse + brightness boost); non-matching stars dim heavily; territories containing matches get halo glow; connector lines between matches brighten. Press Esc → flares stop, full chart restores.
7. **Always-on labels toggle.** Settings → Sight → "Show constellation labels at rest" → flip ON. Constellation labels appear at territory centroids without hover. Flip OFF → labels disappear, hover-only behavior restored.
8. **No regression.** All MIG-018 features still work: hover star, click star, double-click star, projection toggle, side panel, Esc.
9. **Performance.** First-toggle latency ≤ 500ms cold (Milky Way + MDS recompute combined); ≤ 50ms warm; idle-Sight per-frame ≤ 1ms.
10. **`npm run check`** clean (1 pre-existing PJ-012 error; 0 new errors).
11. **Three-agent audit clean.** 0 P0 / 0 P1.

Boss-test failure on any item → fix and retest before §2F audit + close-out.

---

## 7 · Phase scope (build sequence)

Six phases. §2A–§2D cascade autonomously; §2E is the Boss-test gate; §2F closes after audit.

| Phase | Scope | Verification | Boss-testable? |
|---|---|---|---|
| **§2A** | TF-IDF compute (Rust) + similarity IPC + schema additions + cache write | cargo check + 3 unit tests (tokenizer, sparse vector cosine, sentinel similarity result) | No |
| **§2B** | Milky Way frontend render + Settings toggle | npm check + visual sanity locally | No |
| **§2C** | Calendar rim frontend render + Hijri toggle + month filter interaction | npm check + visual sanity | No |
| **§2D** | Universe-health card in side panel + computation module | npm check + cross-validate against v2 metrics if both enabled | No |
| **§2E** | Full search integration (flares + halos) + always-on labels Settings toggle | npm check + **BOSS TEST GATE** (11 acceptance criteria above) | **YES** |
| **§2F** | Three-agent audit + orientation v1.58 → v1.59 + Pending Jobs v1.7 → v1.8 + close-out commit | All Architect §3 invariants verified by audit | No |

---

## 8 · Risks / mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| TF-IDF compute > 500ms on 30k notes | Medium | Sparse vectors (top-50 terms per note) + inverted-index candidate filtering. Bench during §2A on Boss universe. If too slow, fall back to LSH approximation. |
| Tokenizer disagreement (FTS5 stems differently than expected) | Medium | Reuse the existing FTS5 tokenizer. If unavailable as a callable function, document the discrepancy and align in a future MIG. |
| Pixi BlurFilter perf on 30k Milky Way line segments | Medium | Limit to top-2000 highest-similarity edges for rendering. Frontend cap; full edge list still in the cache for future use. |
| Calendar rim labels collide on tight viewports | Low | Render only every-other-month label below a viewport-radius threshold. |
| Universe-health metrics drift from v2's | Medium | Use the same formulas as `clusterEngine.ts::computeUniverseHealth` (already in frontend). Pure-TS module shares the math; cross-check in §2D unit test. |
| User has many notes with no body content (notes used as link nodes) | Low | TF-IDF gracefully degrades: empty bodies → zero similarity → no Milky Way contribution. Documented in §2A. |
| Schema-version bump invalidates user's existing MIG-018 cache | Low (cosmetic) | First-toggle hits cold compute once; sub-second on Boss universe. Ship a one-line note in the v1.59 orientation. |

---

## 9 · Out of scope

Strictly Phase 2. Defer:
- Magnitude slider / layer peeling (PJ-036) → MIG-020.
- v2 retirement → MIG-020.
- PJ-037 (Map↔Sight) — REJECTED.
- LSH approximation for similarity → future PJ if §2A timing exceeds 500ms.
- Solar-Hijri / Hebrew calendars → User can enable them in Settings but rendered with placeholder English labels until PJ-014 backfill.
- Custom user constellations (manual annotation) → future PJ.

---

## 10 · Cross-references

- `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` §5 (PJ-035 absorption), §6 (universe-health), §7 (calendar rim), §9.2 (phase scope).
- `docs/Constellation-Sight-Concept-Paper-v1.1.md` §3.3 (third edge type — content similarity).
- `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-{ARCHITECT,PLAN,AUDIT}.md` — predecessor MIG.
- `src-tauri/src/sight_layout.rs` — extended in §2A.
- `src-tauri/src/search.rs` — `init_db` adds `sight_v3_similarity_edges` table; `SIGHT_V3_SCHEMA_VERSION` bumps to 2.
- `src/lib/graph/clusterEngine.ts` — already has `computeUniverseHealth`; `universe-health.ts` mirrors the formula.
- CLAUDE.md Performance Rules + Architecture Principles (esp. Language-First by Design for Hijri rim).

---

**End of Architect.** Next document: **MIG-019 Plan** (six phases §2A→§2F with verification clauses). After Plan lands, **stop for Eisa's explicit Plan approval before §2A Build begins.**
