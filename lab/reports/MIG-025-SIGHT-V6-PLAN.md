# MIG-025 — Sight v6 Build — Plan

**Phase:** 2 of 4 (/migration discipline)
**Date:** 2026-05-14
**Status:** Plan ready for Boss review.
**Locked option pack:** A1, B2, C3, D1, E1, F3, G1, H1, I1; Phase-2 chip hidden until Phase 3.
**Reference contract:** `docs/Constellation-Sight-Concept-Paper-v4.0.md` §13 ship-gates; `docs/sight-redesign-v0.3-full-layout.svg` + `docs/sight-redesign-v0.3-tradition-chip-detail.svg` visual contract.
**Architect:** `lab/reports/MIG-025-SIGHT-V6-ARCHITECT.md` (territory + invariants).

---

## §0 — Scope summary

MIG-025 builds Sight v6 as a four-phase rollout (§A=v6.0, §B=v6.1, §C=v6.2, §D=v6.3) inside one MIG with one Architect (done), one Plan (this), one Audit (after §D). v5 stays mounted via `SIGHT_V6_ENABLED` dev flag through Phases 1–3, then v5 is deleted in Phase 4. SQLite cache rebuilds progressively in the background via Tauri events with status-bar progress; Sight v6 is render-ready when the first stratum tier completes (not all rows). The seven tradition manifests land together in Phase 3 alongside the geometry. The first-boot tour fires once on the first-ever Sight v6 open via a `tourSeen` flag. Continuous CI perf gating (vitest + playwright) is plumbed in Phase 1 so Phases 2–4 inherit it. v5 settings are read once on first v6.0 boot: `lastScope` carries forward, `lastMode` is dropped silently. v0.3 mock SVGs are the binding visual contract.

---

## §A — Phase 1 (Sight v6.0) — Anchor + Sidebar + Default-simple + Tour

### §A.1 — Add `SIGHT_V6_ENABLED` flag (dev-flag-only, default false)
**Files**: `src/lib/sight/engine.ts`
**Action**: Add `export const SIGHT_V6_ENABLED = false;` below the v5 line. Update the doc-block comment to name MIG-025. No callers touched yet.
**Verification**: `npm run check` passes; `npm run dev` boots; Sight v5 still works; the flag is importable but unreferenced.
**Risk-mitigation**: Architect §4.3 rollback path requires this flag to exist before v6 mounts.

### §A.2 — Backend cache schema for v6 (`sight_v6_layout` table + triggers, idempotent)
**Files**: new `src-tauri/src/sight_v6.rs` (skeleton: types + `ensure_sight_v6_layout_table` + `ensure_sight_v6_invalidation_trigger` + `compute_universe_snapshot_hash` clone); `src-tauri/src/search.rs` lines ~1537–1545 (call `ensure_*` for v6 alongside v5).
**Action**: Create `sight_v6_layout` (PK `note_path` + 12 v5-equivalent columns + 4 new: `link_in_count`, `link_out_count`, `frontmatter_key_count`, `body_chars`) + 2 covering indexes + 2 triggers (`sight_v6_layout_invalidate_au`/`_ad` on `note_meta`). Both v5 and v6 schemas coexist — Architect §4.3 dual-trigger window.
**Verification**: New cold install creates both tables; existing v5 install adds v6 table without touching v5 rows; `SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'sight_%'` returns both.
**Risk-mitigation**: Architect §4.3 dual-trigger throughput; mitigated by indexed DELETE only (cheap).

### §A.3 — Backfill skeleton + sentinel `mig025_sight_v6_layout_backfill_v1`
**Files**: `src-tauri/src/sight_v6.rs` (`backfill_sight_v6_layout` function); reuse v5 sentinel/wipe pattern from `sight_v5.rs:183–220`.
**Action**: Implement bulk INSERT OR REPLACE backfill mirroring v5's pattern but populating the 4 new columns via subqueries (`COUNT(*) FROM note_links WHERE source_path/target_path = nm.path`; `length(content)` on `note_content`; `json_each` count on `properties_json`). Sentinel chain isolated from v5.
**Verification**: Run a one-shot Rust unit test (`#[test] fn backfill_writes_all_rows()` modeled on v5 test pattern) — backfill on a 50-note fixture yields 50 rows with non-NULL `link_in_count`. Sentinel sets to 1.
**Risk-mitigation**: Architect §4.4 sentinel race — v6 sentinel uses `mig025_*` prefix, no collision with `mig024_*`.

### §A.4 — Progressive backfill via Tauri events (Option C3)
**Files**: `src-tauri/src/sight_v6.rs` (add `backfill_sight_v6_layout_progressive` returning per-stratum-tier progress events); `src/lib/sight/v6/backfillProgress.ts` (new — store wrapping the event stream).
**Action**: Replace single bulk INSERT with stratum-tiered loop (5 passes, one per stratum band, lowest first). After each tier, emit `sight-v6-backfill-progress` Tauri event with `{tier, doneRows, totalRows, firstTierComplete}`. Sentinel only stamps after final tier. Frontend store subscribes; gates render-ready on `firstTierComplete=true`.
**Verification**: Manual: open a fresh universe of 7,636 notes; status-bar widget appears within 2 s showing "Sight v6 cache: tier 1/5"; Sight render unblocks at tier 1; tiers 2–5 stream in background with no UI freeze.
**Risk-mitigation**: Architect §4.1 first-boot block; Standing Order resumability honored (sentinel only on final tier — interrupted runs resume by re-running tiers).

### §A.5 — Tauri IPC commands for v6 (mirror v5 surface, drop `scope_kind` parameter)
**Files**: `src-tauri/src/sight_v6.rs` (add `sight_v6_warm_cache`, `sight_v6_get_layout`, `sight_v6_get_link_set_for_notes`); `src-tauri/src/lib.rs:352–355` (tradition handlers in `tauri::generate_handler!` alongside v5).
**Action**: `sight_v6_get_layout(app)` returns the full layout (no scope param — facet sidebar filters frontend-side per Architect §1.4). `sight_v6_get_link_set_for_notes` caps at 800 lines per Concept Paper §2.2 (vs. v5's 2000). `sight_v6_warm_cache` triggers progressive backfill.
**Verification**: Rust unit tests for each IPC; smoke test from Svelte console (`invoke('sight_v6_get_layout')`) returns rows.
**Risk-mitigation**: Architect §1.4 — drop the unused `sight_v5_get_universe_snapshot_hash` pattern; v6 will reuse `compute_universe_snapshot_hash` for live-update polling later.

### §A.6 — Frontend types + module skeleton (`src/lib/sight/v6/`)
**Files**: new `src/lib/sight/v6/types.ts` (`LayoutCacheRow`, `LinkEdge`, `Facet`, `FacetCategory`, `TraditionId`, `MiniDomeChannel`, `GestureEvent`); new `src/lib/sight/v6/SightV6.svelte` (placeholder mount with header strip + canvas div); new `src/lib/sight/v6/anchor.ts` (stub exports for `renderAnchorDome`, `computeStarPositions`, `starHitTest`).
**Action**: Skeletons only — no rendering yet. Mode/scope unions deleted; `LayoutCacheRow` extended with the 4 new columns + `topDecileActs: boolean`, `provenanceSector: 'Self'|'Read'|'Heard'|'Reasoned'|'Tradition'`, `libraryShapeIndex: number`.
**Verification**: `npm run check` clean; importable from `+layout.svelte` even though not mounted yet.
**Risk-mitigation**: Architect §1.1 disposition — locks the v6 module surface.

### §A.7 — Mount v6 alongside v5 in `+layout.svelte` (B2 dual-mount)
**Files**: `src/routes/+layout.svelte` (lines 67/68 imports; line 752 `let sightV6Active = $state(false)` adjacent to `sightV5Active`; line 1047 add v6 to `fullPageActive`; lines 1706, 2940, 3619, 4358, 4454, 4852 add `sightV6Active = false` to mutual-exclusivity blocks; new dock button gated on `SIGHT_V6_ENABLED && $appSettings.enabledFeatures?.constellationSightV6 !== false` directly below the v5 dock at lines 4465–4475; new mount block `{:else if sightV6Active && SIGHT_V6_ENABLED}` after the v5 mount at 5343–5362).
**Action**: Wire v6 dock button + mount block. Add `constellationSightV6` to `enabledFeatures` defaults in `store.ts`. v5 dock stays unchanged. Both engines stay mutually exclusive.
**Verification**: With `SIGHT_V6_ENABLED=false`, dock shows v5 only (today's behavior). Manually flip to `true` in dev — v6 dock appears alongside v5; clicking either closes the other.
**Risk-mitigation**: Architect §4.3 — single-line flip restores v5-only.

### §A.8 — Anchor dome render: chrome (5 strata bands, calendar rim, stratum labels)
**Files**: `src/lib/sight/v6/anchor.ts` (port `stratumBandBoundaries` from `v5/dome.ts` reparameterized 8→5 bands; reuse `calendarRimSpokes`); `src/lib/sight/v6/dome.ts` (new — 5-band geometry + Suwaidi palette tokens).
**Action**: Render 5 concentric guides at 0.6 px stroke `#1a1f2e`, 12 month labels at `r ≈ 340 px` mid-gray, vertical-axis stratum labels (FOUNDATION center → EDGE OF KNOWING outer). No stars yet.
**Verification**: Mount v6 → see empty dome with 5 bands + calendar rim. Visual diff against `docs/sight-redesign-v0.3-full-layout.svg` chrome layer.
**Risk-mitigation**: Concept Paper §11 invariant 2 (Suwaidi-fidelity ≥80%) — measure via DOM-rect computation in §A.10 test.

### §A.9 — Anchor dome render: stars + lines (channel encoding)
**Files**: `src/lib/sight/v6/anchor.ts` (`renderStars`: shape from `libraryShapeIndex` per §3.3 vocab + shape-weight normalization; opacity from `confidenceAlpha`; binary size from `topDecileActs`; pip hue from `stage` ≥1.8 px else suppressed); `renderLinks` with 9 typed-link colors per §3.4, 800-visible auto-fade.
**Action**: Wire `sight_v6_get_layout` → `renderStars` → `renderLinks`. Hit-test ports `wedgeKeyAtPoint` pattern from `v5/render.ts` adapted to per-star `starHitDistanceSq`.
**Verification**: Dev fixture: load 50 notes → see 5 distinct shapes (when 5 libraries), opacity gradient on confidence, 7 px stars on top-decile, pips visible on stage. Hover star → tooltip.
**Risk-mitigation**: Concept Paper §11 invariants 1, 4, 5 (orthogonality, Delta-E, pip foveation) — automated tests deferred to §D, but channel mapping is locked here.

### §A.10 — Facet sidebar (collapsed-by-default tab + Folder, Library, Stratum, Confidence, Stage, Provenance)
**Files**: new `src/lib/sight/v6/facetSidebar.svelte`; `SightV6.svelte` mounts it as a 20 px collapsed tab, expanding to 180 px on click.
**Action**: 6 facets per §2.4. Folder facet TOP per Concept Paper §11 invariant 8. Live counts derived from `LayoutCacheRow[]`; click-category dispatches `filter:apply` event; multi-facet AND across, OR within. Hearst Flamenco preview pattern: count rebalancing on filter set change.
**Verification**: 50-note fixture → all 6 facets render counts; clicking "Library: Research" filters anchor stars; counts in Stratum facet rebalance to filtered subset; clicking again releases filter.
**Risk-mitigation**: Concept Paper §11 invariants 8 (Folder visibility), 9 (no persistent toggle bars).

### §A.11 — First-boot tour overlay (4 steps, skippable, `tourSeen` flag)
**Files**: new `src/lib/sight/v6/tour.svelte`; `SightV6.svelte` (mount tour conditionally on first open); `src/lib/libraries/store.ts` (add `appSettings.sight.tourSeen?: boolean` near lines 3416–3446).
**Action**: 4-step overlay per Concept Paper §5; "Skip" + "Next" buttons; on completion or skip, set `tourSeen=true` via `saveSettings`. "Help → Sight tour" affordance re-fires the tour by clearing `tourSeen` via Help menu (added in §C.10).
**Verification**: Fresh Universe (no `tourSeen`): tour fires on first v6 mount. Click Skip → tour disappears, `tourSeen` saved. Reload → no tour. Manual `tourSeen=false` in DevTools → tour fires again.
**Risk-mitigation**: Concept Paper §11 invariant 10 (first-boot tour).

### §A.12 — Settings migration: read v5 once, map `lastScope` forward, drop `lastMode`
**Files**: `src/lib/libraries/store.ts` (one-shot migration block in store init); add `appSettings.sight.proMode?: boolean`, `tourSeen?: boolean`, `activeTradition?: TraditionId`, `hexBinThreshold?: number`, `linkFadeThreshold?: number` to schema.
**Action**: On first v6.0 boot per Universe, read `appSettings.sight.lastScope` if present → store as `appSettings.sight.activeFolderFacet` (or equivalent); delete `appSettings.sight.lastMode`; stamp `sight.v6MigrationDone=true` to avoid re-migration. Quiet — no user dialog.
**Verification**: Universe with v5 settings (`lastMode='C'`, `lastScope='library'`) opened in v6 build: settings file post-launch shows `lastMode` removed, `activeFolderFacet` populated from `lastScope`, `v6MigrationDone=true`. Re-launch: no second migration.
**Risk-mitigation**: G1 lock — Architect §4.3 rollback survives (v5 still reads `lastScope` if it stays in the file; we drop only `lastMode`).

### §A.13 — CI perf harness (vitest+playwright, F3 plumbing)
**Files**: new `tests/sight-v6/perf.test.ts` (vitest harness for render-budget); new `tests/sight-v6/layout-fidelity.test.ts` (playwright: ≥80% anchor in default state); `package.json` scripts (`test:sight-v6:perf`, `test:sight-v6:layout`); `.github/workflows/sight-v6-ci.yml` (runs both on PRs touching `src/lib/sight/v6/**` or `src-tauri/src/sight_v6.rs`).
**Action**: Skeletons that run today and exercise §A's deliverables (default render ≤100 ms on 1k-note fixture; anchor occupies ≥80% canvas). Phases B/C/D extend with cross-filter, channel orthogonality, Delta-E.
**Verification**: PR with §A.9 changes runs CI; both tests pass on the v6 mount.
**Risk-mitigation**: Concept Paper §8.3 explicit; Architect Option F3 lock.

### §A.14 — Phase 1 ship gate (Concept Paper §13.1 verification clauses)
- [ ] Anchor dome renders all 6 pre-attentive channels per §3.1.
- [ ] Default-simple layout satisfies §6.2 (≥80% anchor, automated test green).
- [ ] Facet sidebar cross-filters across all 6 facets including Folder.
- [ ] First-boot tour fires once, skippable, persisted via `tourSeen`.
- [ ] All §5 gestures work except mini-dome cross-filter (mini-domes don't exist yet).
- [ ] v5 module set still present + reachable via dock toggle (B2).
- [ ] CI perf gate green.
- [ ] Settings migration runs cleanly on a v5-state Universe.

**Action on gate pass**: flip `SIGHT_V6_ENABLED=true` in `engine.ts`, ship as Sight v6.0. v5 dock stays.

---

## §B — Phase 2 (Sight v6.1) — Mini-domes + Cross-filter + Pro mode

### §B.1 — Mini-dome renderer skeleton + 2×2 grid layout
**Files**: new `src/lib/sight/v6/miniDome.ts` (single renderer parameterized by `MiniDomeChannel`); `SightV6.svelte` (right-side hidden grid; `Cmd-D` / "Show diagnostics" toggles visibility; anchor compresses to ~60% width).
**Action**: 2×2 grid, ≥320×320 px each. Stratum bands at 0.04 opacity in each (radial anchor metaphor preserved). Default hidden.
**Verification**: Cmd-D reveals empty 2×2 grid; anchor compresses; pressing Cmd-D again hides.

### §B.2 — Confidence mini-dome (opacity, 2.8 px discs)
**Files**: `src/lib/sight/v6/miniDome.ts` (`renderConfidenceChannel`).
**Action**: Uniform 2.8 px discs, opacity = `confidenceAlpha` (0.4–1.0), no pip, no shape variation.
**Verification**: 50-note fixture: visible opacity gradient across stars; visually distinct from anchor's shape-and-pip rendering.

### §B.3 — Stage mini-dome (full-disk hue, 5 categorical, no pip)
**Files**: `src/lib/sight/v6/miniDome.ts` (`renderStageChannel`).
**Action**: Full-disk 2.8 px hue per Concept Paper §3.4 stage palette (`#4ade80`/`#22d3ee`/`#a78bfa`/`#facc15`/`#94a3b8`).
**Verification**: 5 distinct hue clusters visible; pre-attentive pop confirmed by visual inspection.

### §B.4 — Acts mini-dome (binary size, 6 px filled vs 1.5 px dot)
**Files**: `src/lib/sight/v6/miniDome.ts` (`renderActsChannel`).
**Action**: Top-decile = 6 px filled disc; rest = 1.5 px dot. Size ratio >30% Treisman threshold honored.
**Verification**: Approximately 10% of stars render as the larger disc; visually pre-attentive.

### §B.5 — Provenance mini-dome (5 angular sectors)
**Files**: `src/lib/sight/v6/miniDome.ts` (`renderProvenanceChannel`).
**Action**: Stars positioned by `provenanceSector` (Self/Read/Heard/Reasoned/Tradition) angular sector + radial=stratum. Sector dividers visible.
**Verification**: 5 visible angular wedges, each populated by stars of matching provenance.

### §B.6 — Linked brushing across all 5 views (gold ring on hover)
**Files**: new `src/lib/sight/v6/gestures.ts` (`linkedBrush` store); `SightV6.svelte` + each mini-dome subscribes.
**Action**: Hover any star → `hoveredStarPath` store updates → all 5 views render a gold ring on the matching star. Pointer leaves → ring clears.
**Verification**: Hover anchor star → 4 corresponding mini-dome stars get gold ring. Hover mini-dome star → anchor + 3 other minis ring.

### §B.7 — Cross-filter from mini-dome category click
**Files**: `gestures.ts` (`miniDomeFilter` store); each mini-dome adds click-region detection.
**Action**: Click a mini-dome category (e.g., a Stage hue cluster region) → all 5 views filter to matching stars; non-matches dim to 0.15 opacity; facet sidebar counts rebalance. Click same again → release.
**Verification**: Click Stage mini's "established" cluster → only established stars stay full opacity in all 5 views; sidebar's Stage facet shows highlight on "established" with rebalanced co-facet counts.
**Risk-mitigation**: Concept Paper §11 invariant 3 (≤16 ms cross-filter on 7,636 × 5 views) — perf test added in §B.8.

### §B.8 — Cross-filter performance test in CI
**Files**: `tests/sight-v6/perf.test.ts` (extend with cross-filter scenario).
**Action**: Synthetic 7,636-note fixture; programmatic filter dispatch; assert ≤16 ms render time (Performance.now()).
**Verification**: CI gate fires on PR; passes on the mini-dome implementation.
**Risk-mitigation**: Concept Paper §11 invariant 3 + §8.3 budget.

### §B.9 — Hex-bin aggregation above 5,000 visible (d3-hexbin)
**Files**: `src/lib/sight/v6/miniDome.ts` (conditional renderer); `package.json` (add `d3-hexbin` dep); `appSettings.sight.hexBinThreshold` consumed (default 5000).
**Action**: Above threshold, mini-domes render as hex-bin with dominant-channel-value cell + count badge. Below, per-star. Auto-switches on filter change.
**Verification**: 7,636-fixture: mini-domes render as hex-bins. Filter to <5,000 visible → switches to per-star.

### §B.10 — Pro mode persistence (Cmd-Shift-D, default-state override)
**Files**: `gestures.ts` (Cmd-Shift-D handler); `SightV6.svelte` (read `appSettings.sight.proMode` at mount; if true, expand all chrome on first paint).
**Action**: Cmd-Shift-D toggles `proMode` in settings; persisted across sessions. Default false. Pro mode = sidebar expanded + tradition chip expanded + mini-domes shown on every Sight open.
**Verification**: Cmd-Shift-D → all chrome expands; quit + relaunch → still expanded. Cmd-Shift-D again → returns to default-simple; persists.

### §B.11 — Phase 2 ship gate (Concept Paper §13.2 verification clauses)
- [ ] Four mini-domes render with their isolated channel encoding.
- [ ] Stratum bands at 0.04 opacity visible in each mini.
- [ ] Linked brushing (gold ring) propagates across all 5 views.
- [ ] Click in mini-dome filters all 5 views; counts rebalance.
- [ ] Hex-bin aggregation kicks in above 5,000 visible; per-star below.
- [ ] Cmd-D toggles diagnostics visibility.
- [ ] Pro mode persists across sessions.
- [ ] Cross-filter perf test ≤16 ms on 7,636 × 5 views.
- [ ] Tradition chip area HIDDEN entirely (per locked Phase-2-chip decision).

**Ship as Sight v6.1.**

---

## §C — Phase 3 (Sight v6.2) — Tradition chip + 4 production traditions + manifests

### §C.1 — Tradition chip component (collapsed-by-default, click-to-expand)
**Files**: new `src/lib/sight/v6/traditionChip.svelte`; `SightV6.svelte` (mount in title bar).
**Action**: Default state: single label `Aristotelian ●`. Click → expand row showing all 7 chips. Active tradition has blue stroke + dot. Hover any chip → English secondary label tooltip per Concept Paper §2.5 + §11 invariant.
**Verification**: Title bar shows collapsed chip; click → 7-chip row appears; hover pramāṇa → "pramāṇa — Nyāya fourfold valid means of knowing" tooltip.

### §C.2 — Tradition module pattern + Aristotelian (default geometry)
**Files**: new `src/lib/sight/v6/traditions/index.ts` (registry); new `src/lib/sight/v6/traditions/aristotelian.ts` (geometry: radial=stratum, angular=time — same as default).
**Action**: Each tradition exports `{id, name, remapStarPosition(row, defaultPos): {x,y}, sectorDividers?: SectorSpec[]}`. Aristotelian is the identity remap.
**Verification**: Active tradition Aristotelian → anchor renders identical to v6.1 dome.

### §C.3 — pramāṇa tradition (4 quadrants)
**Files**: new `src/lib/sight/v6/traditions/pramana.ts`.
**Action**: NE pratyakṣa, SE anumāna, SW upamāna, NW śabda. Quadrant divider strokes visible. Radial=stratum within quadrant, angular=time within quadrant. Star quadrant assignment from a frontmatter `pramana_kind` field (default: pratyakṣa if absent, with sidebar facet hint).
**Verification**: Switch tradition chip → stars redistribute across 4 quadrants; dividers visible; mini-domes unchanged.
**Risk-mitigation**: Concept Paper §11 invariant 6 (tradition isolation: anchor only).

### §C.4 — masādir tradition (4 categorical sectors + 4 extension chips)
**Files**: new `src/lib/sight/v6/traditions/masadir.ts`.
**Action**: NE Qur'an, SE sunnah, SW ijmāʿ, NW qiyās; sector annotations (naṣṣ vs ijtihādī; qaṭʿī vs ẓannī). 4 extension chips below the dome (istiḥsān, istiṣḥāb, maṣlaḥa mursalah, ʿurf). Star sector assignment from `masadir_source` frontmatter field.
**Verification**: Active masādir → 4 sectors + 4 extension chips render; per Mustaṣfā citation (§4.1.3).

### §C.5 — Polanyi tradition (tacit/explicit fog gradient)
**Files**: new `src/lib/sight/v6/traditions/polanyi.ts`.
**Action**: Single dome, fog **dense at center** (tacit core 0.14–0.18 opacity), **clear at edges** (explicit periphery 0.85–0.95). Inverted from v0.2 per §4.1.4.
**Verification**: Active Polanyi → visible fog gradient inside-out; opacity inversion visible.

### §C.6 — Tradition switch transition (instant snap + brief flash, motion-reduce respect)
**Files**: `SightV6.svelte` (transition handler); `gestures.ts` (`prefers-reduced-motion` check).
**Action**: v6.0 ships with instant snap + 200 ms identity flash on highlighted star. Animated 400 ms transition is v4.1 polish target.
**Verification**: Switch tradition with a hovered star → flash visible; with `prefers-reduced-motion: reduce` → no flash, just snap.

### §C.7 — Tradition manifests (`docs/traditions/*.md` × 7, all created here per H1)
**Files**: new `docs/traditions/aristotelian.md`, `docs/traditions/pramana.md`, `docs/traditions/masadir.md`, `docs/traditions/polanyi.md`, `docs/traditions/dignaga.md`, `docs/traditions/ishraqi.md`, `docs/traditions/mohist-san-biao.md`.
**Action**: Each follows schema `{id, name, citation, geometry_spec, sectors, exclusions, extensions, version, changelog}` per Concept Paper §4.3 + §11 invariant 7. Citations from §4.1/§4.2 of Concept Paper.
**Verification**: All 7 files exist; citation field populated; ⓘ chip affordance opens the file in editor.
**Risk-mitigation**: Concept Paper §11 invariant 7.

### §C.8 — `activeTradition` persistence + frontend store wiring
**Files**: `appSettings.sight.activeTradition: TraditionId` consumed; `SightV6.svelte` initial state; `traditionChip.svelte` writes on selection.
**Action**: Default `aristotelian`. Saves on click. Restores on reopen.
**Verification**: Click pramāṇa, quit, relaunch → opens with pramāṇa active.

### §C.9 — Mini-dome stipulation honored (channels stay constant across traditions)
**Files**: assertion test in `tests/sight-v6/tradition-isolation.test.ts`.
**Action**: Programmatically switch through all 7 traditions; assert mini-dome channel labels + spatial encoding unchanged. Per Concept Paper §7 + §11 invariant 6.
**Verification**: Test green for all 7 traditions.
**Risk-mitigation**: Concept Paper §11 invariant 6 — automated.

### §C.10 — "Help → Sight tour" affordance (re-fires tour from §A.11)
**Files**: existing Help menu in `+layout.svelte`; new menu item "Sight tour" (clears `tourSeen` and re-mounts v6 with tour=true).
**Action**: Per Concept Paper §11 invariant 10 ("always re-available in Help").
**Verification**: Click Help → Sight tour → tour fires.
**Risk-mitigation**: Concept Paper §11 invariant 10.

### §C.11 — Phase 3 ship gate (Concept Paper §13.3 verification clauses)
- [ ] All 4 production-polish traditions render correctly per §4.1.
- [ ] Hover tooltip on each chip shows English secondary label.
- [ ] Tradition switch animation runs (instant + flash for v6.0; 400 ms eased = v4.1 polish target).
- [ ] All 7 tradition manifests in `docs/traditions/` with citations.
- [ ] Mini-dome channels unchanged across tradition switches (test green).
- [ ] Help → Sight tour re-fires the orientation overlay.

**Ship as Sight v6.2.**

---

## §D — Phase 4 (Sight v6.3) — 3 v1-preview traditions + CI hardening + v5 deletion

### §D.1 — Dignāga tradition — **SUPERSEDED / EXCLUDED** (Eisa 2026-05-16, §C.1-fix-1)

> **Status**: EXCLUDED from Constellation entirely. Per Eisa's direction during §C.1 Stage 2 Boss-test review: "don't include the 'Dignāga' at all in any of Constellation functions." The Dignāga tradition is permanently out — no chip option, no tradition module, no manifest, no Phase 4 build step. The 'dignaga' literal is removed from `TraditionId` (types.ts) and from the `activeTradition` union (store.ts); a migration block rewrites any persisted `'dignaga'` value back to `'aristotelian'`. Concept Paper §4.2.1 carries a matching EXCLUDED note. The tradition set shrinks from 7 to 6 (4 production + 2 v1-preview). §D's remaining v1-preview steps are §D.2 (Suhrawardi Ishrāqī) and §D.3 (Mohist sān biǎo).

~~**Files**: new `src/lib/sight/v6/traditions/dignaga.ts`; chip tooltip shows "v1 preview — polish in v4.1".~~
~~**Action**: Left hemisphere = pratyakṣa, right hemisphere = anumāna. Center labeled "rejected: śabda, upamāna" per Concept Paper §4.2.1 — the absence is the feature.~~
~~**Verification**: Active Dignāga → 2-hemisphere split; center labels visible; chip badge shows "preview".~~

### §D.2 — Suhrawardi Ishrāqī tradition — **SUPERSEDED / EXCLUDED** (Eisa 2026-05-16, §C.4-religious-rule)

> **Status**: EXCLUDED from Constellation entirely per the new top-principal religious-lineage rule (orientation v2.09): "when dealing with religious references, no non-Abrahamic; for Islamic, Sunni only." The Ishrāqī tradition (Suhrawardi, 1154–1191) was overwhelmingly absorbed into Twelver Shīʿī ḥikma (Mulla Sadra, Sabzavari, modern Qom curriculum) — failing the Sunni-only restriction — and is fundamentally religious-mystical rather than philosophical-epistemological. No chip option, no tradition module, no manifest, no Phase 4 build step. The 'ishraqi' literal is removed from `TraditionId` (types.ts) and from the `activeTradition` union (store.ts); a migration block rewrites any persisted `'ishraqi'` value back to `'aristotelian'`. Concept Paper §4.2.2 carries a matching EXCLUDED note. §D's remaining v1-preview step is §D.3 (Mohist sān biǎo).

~~**Files**: new `src/lib/sight/v6/traditions/ishraqi.ts`.~~
~~**Action**: Small gold disc center + emanation glow + 3 dashed concentric rings outward; peripheral stars = ʿilm ḥuṣūlī. Per §4.2.2.~~
~~**Verification**: Active Ishrāqī → gold core + emanation rings visible.~~

### §D.3 — Mohist sān biǎo tradition (3 horizontal zones, "v1 preview" label)
**Files**: new `src/lib/sight/v6/traditions/mohist-san-biao.ts`.
**Action**: 3 horizontal zones — top 本, middle 原, bottom 用. Angular = time across zones. Per §4.2.3.
**Verification**: Active sān biǎo → 3 horizontal zones visible; chip badge shows "preview".

### §D.4 — Channel orthogonality automated test in CI
**Files**: new `tests/sight-v6/orthogonality.test.ts`; `tests/sight-v6/delta-e.test.ts`.
**Action**: Static-analysis test enumerating channel-to-Bertin-variable mappings; assert no two channels share a variable per §3.2 table. Delta-E test loads stage + link palettes, computes pairwise CIE Delta-E, asserts ≥30 for any co-rendered pair within 5 px.
**Verification**: Both tests green; PR-blocking.
**Risk-mitigation**: Concept Paper §11 invariants 1, 4 — automated.

### §D.5 — Pip foveation threshold test
**Files**: `tests/sight-v6/pip-foveation.test.ts`.
**Action**: Render anchor at min/default/max zoom; assert pip diameter ≥1.8 px at default; pip suppressed when computed <1.5 px.
**Verification**: Test green.
**Risk-mitigation**: Concept Paper §11 invariant 5.

### §D.6 — v5 deletion (the cleanup commit)
**Files**: delete `src/lib/sight/v5/` entire directory; delete `src-tauri/src/sight_v5.rs`; remove `mod sight_v5;` from `src-tauri/src/lib.rs`; remove v5 IPC registrations from `lib.rs:352–355`; remove `ensure_sight_v5_*` calls from `search.rs:1537–1545`; drop v5 dock button + mount block + 7 mutual-exclusivity blocks from `+layout.svelte`; remove `SIGHT_V5_ENABLED` from `engine.ts`; remove `lastMode`, `lastScope` keys from `store.ts` schema (they were already migrated in §A.12 and never read).
**Action**: Migration: also `DROP TABLE sight_v5_layout` + `DROP TRIGGER sight_v5_layout_invalidate_au`/`_ad` in a one-shot migration sentinel `mig025_sight_v5_cleanup_v1` in `search.rs::init_db`. Per Concept Paper §9.3.
**Verification**: `npm run check` clean; cold install creates only `sight_v6_layout`; existing v5-installed Universe drops the v5 table on first v6.3 boot; v5 dock gone; no broken imports.
**Risk-mitigation**: Architect §4.3 dual-trigger window closes here. Concept Paper §13.4 ship gate items.

### §D.7 — Phase 4 ship gate + Audit prerequisites (Concept Paper §13.4)
- [ ] Dignāga, Ishrāqī, sān biǎo render with "v1 preview" labels.
- [ ] Channel orthogonality automated test in CI.
- [ ] Performance budget tests pass (≤16 ms cross-filter, ≤100 ms default render).
- [ ] CIE Delta-E ≥30 verified for stage + link palettes.
- [ ] Pip foveation threshold test green.
- [ ] v5 module set deleted; v5 settings keys migrated/dropped.
- [ ] v5 SQLite cache table dropped; v6 cache present.
- [ ] All 10 Concept Paper §11 invariants have at least one passing test or verified manual check.

**Ship as Sight v6.3 (final v6.0 release).** Then Phase 4 Audit (3 agents in parallel per /migration §Phase 4).

---

## §E — Risk tradition cross-reference

| Architect §4 risk | Severity | Mitigation steps |
|---|---|---|
| First-boot block on 7,636-note backfill | Medium | §A.4 progressive backfill + status-bar progress; render-ready on first stratum tier. |
| Mid-Phase-2-shipped behavior of tradition chip | — | Locked: hidden in v6.1 (§B.11), appears in v6.2 (§C.1). |
| Rollback v6.x → v5 regression | Low (B2) | §A.1 + §A.7 + §D.6 — flip `SIGHT_V6_ENABLED=false`, v5 dock returns; cache rows survive. |
| Dual-trigger throughput during Phases 1–3 | Low | §A.2 indexed DELETE only; §D.6 closes the window. |
| Schema-version sentinel race | Low | §A.3 `mig025_*` prefix prevents collision with `mig024_*`. |
| Phase-1-only ship feels incomplete | Medium | §A.7 dock toggle; default-simple state (§A.8–§A.10) + tour (§A.11) hides minimum-viable feel. |
| Tradition implementation cost overruns | High | §D.1–§D.3 explicit "v1 preview" labels set expectations per §4.2. |
| Performance fails at >10k notes | Medium | §B.9 hex-bin aggregation; §B.8 + §D.4 CI gates catch regressions early. |
| Library color loss feels austere | Medium | Out of scope (§G item — v4.1 escape hatch). |
| Tradition-switch animation creates motion sickness | Low | §C.6 motion-reduce respect; v6.0 ships instant snap. |

---

## §F — Rollback plan (B2-aligned)

**Trigger condition**: regression discovered in v6.0 production.

**Steps**:
1. Hotfix PR: flip `SIGHT_V6_ENABLED=false` + `SIGHT_V5_ENABLED=true` in `src/lib/sight/engine.ts` (single commit).
2. Ship hotfix release.
3. v5 dock button reappears; v5 component mounts; v5's `sight_v5_warm_cache` and `sight_v5_layout` rows are intact (we never dropped v5 cache pre-Phase-4).
4. v6 `sight_v6_layout` rows stay in DB — they don't conflict with anything.
5. `tourSeen=true` survives — re-enabling v6 later won't re-fire tour.
6. Settings: `lastScope` was preserved in §A.12 (only `lastMode` was deleted) — v5 reads `lastScope` correctly; no data loss.
7. `activeTradition`, `proMode`, `hexBinThreshold` settings stay in file but are unread by v5 — harmless dead keys.

**Post-rollback**: file a follow-up MIG to root-cause and re-attempt v6 cutover.

**Hardest-rollback step**: §D.6 (v5 deletion). Once v5 is deleted, rollback requires `git revert` + DB-level recreation of `sight_v5_layout`. Mitigation: §D.6 lands ONLY after v6.3 ships and runs ≥2 weeks clean per Concept Paper §9.3.

---

## §G — Out-of-scope (deferred to v4.1 / future MIGs per Concept Paper §10)

1. Pramāṇa internal-structure rendering (per-quadrant indriya-artha-sannikarṣa loci, etc.).
2. Dignāga / Ishrāqī / Mohist polish (v1-preview → production).
3. Tradition-aware mini-dome relabeling (§7 enhancement).
4. Color-accessibility variant (high-contrast / colorblind-safe palette).
5. 400 ms eased tradition switch animation.
6. Universe selector chip for cUniverse federation.
7. Library color recognition aid (low-saturation tint, opt-in setting).
8. Layer 3 (Recommendations) and Layer 4 (Coaching).
9. WebGL upgrade path if hex-bin aggregation insufficient at >10k notes.
10. Deep-link / router-hash mounting of Sight (no router state today).

---

## §H — Three Plan inferences (Boss-confirmed 2026-05-14)

All three locked per Eisa's answers:

1. **Frontmatter source for tradition sector assignment** (§C.3, §C.4): **LOCKED → frontmatter convention.** New frontmatter fields `pramana_kind` (values: `pratyaksa | anumana | upamana | sabda`) and `masadir_source` (values: `quran | sunnah | ijma | qiyas | istihsan | istishab | maslaha | urf`) populated by the user. Notes without the field render in a default bucket (pramāṇa default = pratyaksa quadrant; masādir default = qiyās quadrant) with a sidebar facet hint inviting the user to fill in. Cross-Civ SMEs reviewed this pattern in round-3.

2. **Help menu integration for "Sight tour" re-fire** (§C.10): **LOCKED → existing Help menu in `+layout.svelte`.** Add menu item "Sight tour" alongside existing Help entries. Standard PKM-tool affordance pattern.

3. **`enabledFeatures.constellationSightV6` settings flag name** (§A.7): **LOCKED → fresh `constellationSightV6` key.** v5's `constellationSightV3` legacy quirk does not propagate to v6. v6 ships with a clean settings-key namespace.

---

**End of MIG-025 Plan. All inferences locked. Awaiting Boss "approved" before §A.1 Build opens.**
