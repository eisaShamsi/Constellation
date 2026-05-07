# MIG-018 — Sight v3 Projection Foundation (Architect)

**Migration**: MIG-018 (within PJ-038)
**Phase position**: 1 of 3 in the v3 build trajectory (MIG-018 → MIG-019 → MIG-020).
**Effort class**: Single MIG, multi-phase build (4–6 phases). Estimated 1–2 sessions of focused build + Boss-test.
**Reference design**: `docs/Constellation-Sight-v3-Concept-Paper-v1.1.md` §3 (projection method) + §4 (interactivity, Phase-1 subset) + §8 (performance) + §9.1 (phase scope).

---

## 1 · Goal

Ship the **projection foundation** of v3 Sight: stars at their correct positions on a star-chart-style 2D dome, with constellation-territory polygons, faint-at-rest connector lines, basic hover/click/double-click interactivity, and a working Lambert ↔ stereographic projection toggle. Behind a `SIGHT_V3_ENABLED` feature flag. v2 stays available (MIG-017 left it as known-good fallback).

End-state of MIG-018:
- A user toggles the new v3 dock button. Sight v3 opens.
- Stars render at deterministic positions (graph-distance Landmark-MDS embedding → Lambert projection by default).
- Constellation territories drawn as soft-fill polygons in cycled-pastel colors.
- Connector lines render faint at rest (per Eisa's design call); brighten on hover/select.
- Hover a star → tooltip + incident-edge brighten.
- Click a star → its constellation lines all brighten + side panel slides in.
- Double-click a star → opens the note in editor (mirrors v2 behaviour).
- Settings → Sight panel exposes the projection toggle (Lambert / Stereographic).
- SQLite `sight_v3_layout` cache + write-time triggers persist the embedding across sessions.

End-state of MIG-018 NOT-yet:
- Milky Way density wash (PJ-035, deferred to MIG-019).
- Calendar rim (deferred to MIG-019).
- Full search integration with flares (deferred to MIG-019; basic search match works).
- Magnitude slider / layer peeling (PJ-036, deferred to MIG-020).
- Universe-health card in side panel (deferred to MIG-019).
- Always-on labels Settings toggle (Phase-1 default = hover-only; Settings toggle deferred to MIG-019).
- v2 retirement (deferred to MIG-020).

This phasing is straight from Concept Paper v1.1 §9.1.

---

## 2 · Surfaces this MIG creates / touches

### 2.1 New Rust modules

| File | Purpose | Approx LOC |
|---|---|---|
| `src-tauri/src/sight_layout.rs` (NEW) | Landmark-MDS embedding compute + helpers. Exports `compute_layout_embedding` IPC + per-library back-fill scheduler (`maybe_schedule_layout_backfill`). Mirrors `sky_backfill.rs` patterns. | ~400 |
| `src-tauri/src/sight.rs` (existing, +50) | Add a thin `constellation_sight_v3_layout` IPC that reads from cache or triggers compute. Existing centrality / community IPCs untouched. | ~50 added |

### 2.2 Schema additions

In `src-tauri/src/init_db.rs` (or wherever the `CREATE TABLE` block lives — to be confirmed during Plan phase):

```sql
-- MIG-018: Sight v3 layout cache (write-time-derived, mirrors sky_nodes pattern).
-- One row per (note path, library_set_hash, graph_version).
-- Read at frontend Sight-toggle in milliseconds; recompute only when invalidated.
CREATE TABLE IF NOT EXISTS sight_v3_layout (
    note_path TEXT NOT NULL,
    library_set_hash TEXT NOT NULL,
    graph_version INTEGER NOT NULL,
    embed_x REAL NOT NULL,           -- 2D embedding x in unit-disk coords
    embed_y REAL NOT NULL,           -- 2D embedding y in unit-disk coords
    community_id INTEGER NOT NULL,    -- Louvain community assignment
    centrality_norm REAL NOT NULL,    -- normalized [0,1] centrality
    PRIMARY KEY (note_path, library_set_hash, graph_version)
);
CREATE INDEX IF NOT EXISTS idx_sight_v3_layout_libset_ver
    ON sight_v3_layout(library_set_hash, graph_version);

-- Cursor for resumable back-fill (mirrors sky_backfill_cursor).
CREATE TABLE IF NOT EXISTS sight_v3_layout_cursor (
    library_set_hash TEXT PRIMARY KEY,
    graph_version INTEGER NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT 0,
    started_at INTEGER NOT NULL,
    completed_at INTEGER
);
```

**Triggers**: invalidate cached layout when `note_meta` or `note_links` writes happen. Mirrors `sky_nodes`/`sky_links` triggers if any exist; otherwise add fresh triggers that bump a `graph_version` table.

```sql
-- Single-row meta table tracking the current graph version per library set.
CREATE TABLE IF NOT EXISTS sight_v3_graph_version (
    library_set_hash TEXT PRIMARY KEY,
    version INTEGER NOT NULL DEFAULT 0,
    bumped_at INTEGER NOT NULL
);
```

Then the trigger logic on `note_meta` insert/update/delete + `note_links` insert/delete bumps the `version` column for the relevant library set, which in turn invalidates the cache (frontend reads version, compares to cached, recomputes if stale).

### 2.3 New frontend modules

| File | Purpose | Approx LOC |
|---|---|---|
| `src/lib/sight/projection.ts` (NEW) | Pure TypeScript: `embedToScreen(embed, mode, viewport)` — applies Lambert or stereographic transform to a `(x, y)` embedding tuple. Tested in isolation. | ~80 |
| `src/lib/sight/layout-cache.ts` (NEW) | Frontend wrapper over the `constellation_sight_v3_layout` IPC. Returns the cached layout or triggers compute. | ~100 |
| `src/lib/sight/community-territory.ts` (NEW) | Computes the alpha-shape (or convex hull) polygon for each Louvain community given the projected coords. Used by SightV3.svelte to render territory fills. | ~150 |
| `src/lib/sight/v3/SightV3.svelte` (NEW) | The v3 component. Two Pixi layers (base + focus overlay) + DOM layer for tooltips/side panel. | ~600 |
| `src/lib/sight/v3/SightV3SidePanel.svelte` (NEW) | The side panel that slides in on click — note metadata, top-5 incoming/outgoing, structural-gap suggestions. | ~250 |
| `src/lib/sight/engine.ts` (existing, +1 line) | Add `export const SIGHT_V3_ENABLED = false` (initially false; flipped to `true` once Boss-tests pass at end of MIG-018). | +5 |

### 2.4 +layout.svelte additions

| Edit | Description |
|---|---|
| Add `import { SIGHT_V3_ENABLED }` | Pair with existing `SIGHT_V2_ENABLED` import. |
| Add v3 dock button | Gated by `SIGHT_V3_ENABLED && enabledFeatures.constellationSight !== false` (yes — re-uses the same `enabledFeatures` field, since v3 is the *successor* to v2's user-visible "Constellation Sight" plugin). Mirrors the v2 dock-button block that MIG-017 hid. |
| Add v3 modal mount | `{#if sightV3Active && SIGHT_V3_ENABLED} <SightV3 ... /> {/if}`. New `let sightV3Active = $state(false);` declaration alongside the existing `lensActive`. Toggle handler sets `sightV3Active = true` + clears all other full-page flags. |
| Add v3 escape handler | `if (sightV3Active) { sightV3Active = false; return; }` in the existing keyboard-shortcut handler. |
| Add `sightV3Active` to `fullPageActive` $derived | So sidebars retract when v3 is open. |

### 2.5 Settings additions

| Edit | Description |
|---|---|
| `SettingsModal.svelte` — re-add `constellationSight` plugin entry | Now gated `...(SIGHT_V3_ENABLED ? [{ id: 'constellationSight', ... }] : [])`. The same id as v2 (forward-compat — when MIG-020 retires v2, the field name lives on). |
| Settings → Sight panel (NEW section) | Two controls in MIG-018: (a) projection toggle (Lambert / Stereographic). (b) — placeholder for MIG-019 calendar systems + MIG-019 always-on labels toggle. Storage in `appSettings.sight` object. |
| `DEFAULT_SETTINGS.sight` (NEW) | `{ projection: 'lambert', alwaysOnLabels: false, calendarSystems: ['gregorian'] }`. |
| `appSettings` TypeScript interface | Add `sight: SightSettings` field. |

### 2.6 i18n

15 locale keys to add (en + ar this MIG; 13 others queue under PJ-014 backfill):
- `settings.sight.title` — "Sight"
- `settings.sight.projection.label` — "Projection"
- `settings.sight.projection.lambert` — "Lambert (equal-area)"
- `settings.sight.projection.stereographic` — "Stereographic (equal-angle)"
- `sightV3.tooltip.centralityRank` — "Centrality rank: {n}"
- `sightV3.tooltip.community` — "Community: {name}"
- `sightV3.tooltip.lifecycle` — "Lifecycle: {stage}"
- `sightV3.sidePanel.incomingLinks` — "Incoming links"
- `sightV3.sidePanel.outgoingLinks` — "Outgoing links"
- `sightV3.sidePanel.structuralGaps` — "Structural gap suggestions"
- `sightV3.title` — "Constellation Sight"
- `commands.openSight` — "Open Constellation Sight"

Per Standing Order #6, all 15 locales get the new keys (Eisa's MIG-014/MIG-015 i18n discipline). Body translations for the 13 non-en/ar locales are placeholder values matching English; PJ-014 owns the localization backfill.

### 2.7 Help docs

| File | Edit |
|---|---|
| `docs/help.uConstellation.World/Constellation Sight/Constellation Sight.md` | The 🚧 banner from MIG-017 stays but its top line revises: "v3 (projection foundation) is live; v2 is retired here, see release notes." |
| `docs/User Manual.md` chapter for Sight | New section describing v3's controls. Defer detailed translations to PJ-014. |

---

## 3 · Invariants — what must not break

| # | Invariant | Why |
|---|---|---|
| 1 | **v2 stays disabled but on disk.** `SIGHT_V2_ENABLED` stays `false` through MIG-018. v3 ships alongside, not replacing. | Eisa's "secure-don't-muddle" rule. v3 must prove itself before v2 retires (deferred to MIG-020). |
| 2 | **Determinism.** Same `(library_set_hash, graph_version, k_landmarks)` → same `(x, y)` for every note. | Spatial-memory grammar requires it. |
| 3 | **Boot performance.** No new IPC on the boot critical path. `compute_layout_embedding` runs only on first v3 toggle (or in idle-prewarm post-paint). | CLAUDE.md Performance Rule 8. Boot ≤ 6 sec stays ≤ 6 sec. |
| 4 | **Write-time derivation (Rule 8).** The `sight_v3_layout` cache is invalidated by triggers on `note_meta` / `note_links` writes. Reads are cheap SELECTs. | CLAUDE.md Rule 8 + the v3 paper §3.2. |
| 5 | **No `$effect` loops.** `sightV3Active` reactive boolean follows v2's pattern (set on toggle, cleared on Escape / navigate-away). | CLAUDE.md Performance Rule 2. |
| 6 | **No memory leaks.** Pixi layers destroy on unmount; SightV3 component cleans up its `EditorView`-equivalent (Pixi `Application.destroy()`). | CLAUDE.md Performance Rule 4. |
| 7 | **i18n integrity.** All 15 locales receive the new keys (en + ar with full strings; 13 others with placeholder English values). | Eisa's MIG-014/MIG-015 i18n discipline. |
| 8 | **RTL parity.** Side panel + tooltip + Settings entry render RTL when interface locale is RTL. | CLAUDE.md Architecture Principle: Language-First by Design. |
| 9 | **No regression in other surfaces.** Sky View, OrgChart, Map, Index, SearchHub, Inspector360 unaffected. | Adjacent dock buttons / Settings entries; pre-existing gates retained. |
| 10 | **M11 zero-diff.** `git diff src-tauri/src/lexicon/` empty for this MIG. | The Arabic lexicon engine is orthogonal. |
| 11 | **PJ-038 §8 trajectory honored.** PJ-035 / PJ-036 absorbed in MIG-019/-020, NOT MIG-018. PJ-037 NOT touched (rejected). | Concept Paper v1.1 §9.1 phase scope. |
| 12 | **Pixi resource budget.** SightV3 base layer sprite count ≈ N stars + N+M edge segments + ~30 territory polygons + Milky Way (Phase 2). On 30k notes: ~30k stars + ~60k edges. Pixi v8 batched-rendering handles this; verified during build. | Concept Paper v1.1 §8.1 budgets. |
| 13 | **Settings round-trip.** New `appSettings.sight.projection` field round-trips on settings.json save+load. | Standard appSettings discipline. |

---

## 4 · Drift map

The v3 build introduces several new files + new schema. Drift checklist (LL-023):

| New thing | Consumers known at MIG-018 commit | Future consumers expected | Risk |
|---|---|---|---|
| `SIGHT_V3_ENABLED` const | +layout.svelte (3 sites: dock button, modal mount, escape handler), SettingsModal.svelte (1 site: plugin entry conditional spread) | MIG-019 + MIG-020 will gate new v3 features behind same const | Low — same idiom as MIG-017's `SIGHT_V2_ENABLED` |
| `sight_v3_layout` table | `compute_layout_embedding` IPC (read+write), frontend `layout-cache.ts` (read), invalidation triggers (write of `graph_version`) | MIG-019 will read same table; MIG-020 might add `peeled_centrality` column for layer-peeling cache | Low — schema is stable across the three MIGs |
| `appSettings.sight` object | SettingsModal.svelte (Sight section), SightV3.svelte (reads `projection`), new layout-cache.ts (no read) | MIG-019: `alwaysOnLabels`, `calendarSystems`. MIG-020: `magnitudeThreshold` (slider state) | Low — namespace pre-allocated |
| `constellation_sight_v3_layout` IPC | layout-cache.ts (frontend), SightV3.svelte indirectly | MIG-019 / MIG-020 read same IPC | Low |
| `sight_layout.rs` module | sight.rs (calls into it), lib.rs (registers IPC) | Static; no expected drift | Negligible |

Particularly watching for: a stray `sky_*` consumer that grep-misses but reads sight-layout via a generic SQL query. Audit phase will scan for this. Pattern check: `SELECT.*FROM sight_v3_layout` should be `LIMIT N` everywhere it's called from the frontend; full table scans only happen during MDS recompute in Rust.

---

## 5 · Migration-path map

Five scenarios for first-boot / mid-state / rollback:

| # | Scenario | Behavior |
|---|---|---|
| 1 | **Fresh install** | DEFAULT_SETTINGS gives `sight: { projection: 'lambert', ... }`. `SIGHT_V3_ENABLED = true` (post-Boss-test) shows the v3 dock button. First click triggers cold-cache compute. Status: clean. |
| 2 | **Existing user, no `sight` in saved settings** | Settings merge fills `sight` from DEFAULT_SETTINGS. Same behavior as #1. |
| 3 | **Existing user, has stale `enabledFeatures.constellationSight: true` from v2-era** | Honored: dock button shows. Same end-state. |
| 4 | **Existing user, has stale `enabledFeatures.constellationSight: false`** | Setting wins; no dock button. User can re-enable via Settings → Plugins. |
| 5 | **Mid-compute interruption** | Backfill cursor table tracks progress (mirrors sky_backfill). Resume on next Sight toggle. No corruption. |
| 6 | **Rollback to pre-MIG-018** | Flip `SIGHT_V3_ENABLED = false`. v3 dock button hidden. v2 still hidden (MIG-017 stayed in effect). User has no Sight surface — same as post-MIG-017 / pre-MIG-018 state. |
| 7 | **Forward to MIG-019** | New columns added by ALTER TABLE (or replaced via `CREATE TABLE`-with-version-bump pattern). MIG-018's `sight_v3_layout` rows survive. |

---

## 6 · Acceptance criteria — Boss-test gate

At the end of MIG-018 §1F (audit) and before flipping `SIGHT_V3_ENABLED = true` in production:

1. **Stars render.** Open Sight v3 from the dock button. The dome populates with stars; positions are reproducible across consecutive opens.
2. **Territories visible.** Communities show as colored regions with crisp borders.
3. **Faint connector lines visible at rest.** The structural pattern is readable without hover.
4. **Hover a star.** Star bumps in size + brightness; its incident edges brighten; tooltip shows note title + centrality rank + community + lifecycle stage.
5. **Click a star.** Side panel slides in; the clicked star's constellation lines all brighten; other constellations dim.
6. **Double-click a star.** The note opens in the editor (same behaviour as v2).
7. **Settings → Sight → Projection toggle.** Switching from Lambert → Stereographic redraws the dome; star positions update; behaviour parity (hover/click/double-click) preserved.
8. **Boot performance.** Cold boot to first paint ≤ 870 ms (Criterion 1 unchanged); `boot:hydrated` ≤ 6 sec (Criterion 2 unchanged). v3 prewarm runs idle-time only.
9. **First-toggle latency.** Cold-cache compute finishes in ≤ 500 ms on Boss's 7,600-note universe; warm-cache toggle ≤ 50 ms. Per Concept Paper v1.1 §8.1.
10. **No regression.** Sky View, OrgChart, Map, Index, SearchHub, Inspector360 all open normally. No console errors during a basic exercise of those surfaces.
11. **Type-check clean.** `npm run check` produces 1 pre-existing PJ-012 error and 0 new errors.
12. **Three-agent audit clean.** 0 P0 / 0 P1.

Boss-test failure on any item → fix and retest before flipping the const.

---

## 7 · Phase scope (build sequence)

Six phases inside MIG-018. Each lands as one commit. Each ends with a verification clause.

| Phase | Scope | Verification |
|---|---|---|
| **§1A** | Schema + Rust skeleton | Add `sight_v3_layout` + `sight_v3_layout_cursor` + `sight_v3_graph_version` tables in init_db. Add `sight_layout.rs` module skeleton with stubbed `compute_layout_embedding` returning empty. Wire IPC into lib.rs registry. **Verify**: `npm run tauri build` succeeds; SQLite tables present after first boot. |
| **§1B** | Landmark MDS embedding compute | Implement Landmark-MDS in `sight_layout.rs`. Returns deterministic `Vec<(NoteId, f32, f32, community_id, centrality)>`. Persists to `sight_v3_layout` table. **Verify**: cargo test passes; manually invoke IPC via Tauri devtools; rows present in SQLite. |
| **§1C** | Frontend skeleton + dock button | Add `src/lib/sight/{engine,projection,layout-cache}.ts` + `SightV3.svelte` skeleton (renders empty Pixi canvas + a placeholder text). Add dock button + modal mount + escape handler in `+layout.svelte`. Both gated behind `SIGHT_V3_ENABLED = false` (unchanged). **Verify**: dock button does not render; type-check clean; no regression. |
| **§1D** | Star rendering + projection toggle | SightV3 reads layout cache via IPC; renders stars with Lambert default. Add Settings → Sight panel with projection toggle. **Verify**: flip `SIGHT_V3_ENABLED = true` locally; stars render; toggle works; flip back to `false` for commit. |
| **§1E** | Territories + connector lines + hover/click | Compute community alpha-shapes in `community-territory.ts`. Render territory polygons + faint connector lines (base layer). Add focus overlay layer. Wire hover (incident edges brighten + tooltip) + click (constellation brightens + side panel) + double-click (open note). **Verify**: full Phase-1 behaviour observable with `SIGHT_V3_ENABLED = true`. **BOSS-TEST GATE**: Eisa runs the install, verifies all 12 acceptance criteria. |
| **§1F** | Three-agent audit + flip enable + close-out | Run invariants / drift / migration-path agents. Fix any P0/P1. Flip `SIGHT_V3_ENABLED = true` in source. Bump orientation v1.57 → v1.58 inline. Bump Pending Jobs v1.6 → v1.7 (MIG-018 closes; MIG-019 opens as next-up). Final commit. |

Phases §1A–§1D are implementation-only; no Boss interaction required. Phase §1E is the Boss-test gate. Phase §1F closes the MIG.

---

## 8 · Risks / mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| Landmark-MDS picks bad landmarks (low-betweenness nodes) on a small universe → poor embedding | Medium | Choose landmarks by max-betweenness rank; fall back to random if betweenness ties. Test on 50-note synthetic universe before Boss universe. |
| Pixi v2-era code patterns conflict with v3 component (v3 uses Pixi v8 via existing dependency) | Low | Sky View's `graphEngine.ts` already uses Pixi v8; v3 mirrors its idioms. |
| Alpha-shape boundary computation is brittle on degenerate cases (1-node communities, collinear points) | Medium | Fall back to convex hull when alpha-shape fails. 1-node communities render as a small circle, not a polygon. |
| SQLite cache invalidation triggers fire too aggressively (every note save → graph_version bump → v3 recompute on every toggle) | Medium | Bump only on `note_links` insert/delete, not on `note_meta` body update. Tag changes do bump (since shared-tag edges depend on tags). Tested in §1B. |
| Two Pixi layer architecture leaks textures on every focus change | Medium | Reuse texture atlases across overlay redraws; destroy only on component unmount. Unit-test the destroy path. |
| Settings `sight` namespace collides with future v4 / other features | Low | Reserved by this MIG. Future namespaces use sub-keys (`sight.v3`, `sight.timeline`, etc.). |
| Boss-test surfaces a fundamental visual-grammar issue (e.g., the dome looks too dense / too sparse) | Medium | The Concept Paper §3.3 edge cases describe the mitigations (per-universe layout tuning); §1E's Boss test is the moment to flag this. If the universe is too sparse, fall back to the v1.0 §3.3 "tiny universes" mode. |

---

## 9 · Out of scope

Strictly Phase-1. Defer:
- Milky Way density (PJ-035) → MIG-019.
- Calendar rim → MIG-019.
- Universe-health card → MIG-019.
- Search flares → MIG-019 (basic match-highlight may sneak in §1E if cheap).
- Always-on labels Settings toggle → MIG-019.
- User Style-Settings color override → MIG-019 (default cycled-pastels enough for §1E).
- Magnitude slider / layer peeling (PJ-036) → MIG-020.
- v2 retirement → MIG-020.
- PJ-037 (Map↔Sight) — REJECTED, not in any v3 MIG.

---

## 10 · Cross-references

- **`docs/Constellation-Sight-v3-Concept-Paper-v1.1.md`** §3 / §4 / §8 / §9.1 — design source of truth.
- **`docs/Constellation-Sight-Concept-Paper-v1.1.md`** — analytical foundation v3 inherits.
- **`lab/reports/MIG-016-SIGHT-INSTANT-TOGGLE-AUDIT.md`** — what v2 shipped, what was deferred (especially §1E SQLite cache, now absorbed here).
- **`lab/reports/MIG-017-DISABLE-V2-SIGHT-{ARCHITECT,PLAN,AUDIT}.md`** — gating mechanism (`SIGHT_V2_ENABLED` const), pattern v3 mirrors with `SIGHT_V3_ENABLED`.
- **`src-tauri/src/sky_backfill.rs`** — model for `sight_layout.rs` resumable-backfill pattern.
- **`src-tauri/src/sight.rs`** — existing centrality + community-detection IPCs (untouched, reused).
- **`src/lib/sight/engine.ts`** — current home of `SIGHT_V2_ENABLED`; gains `SIGHT_V3_ENABLED`.
- **`src/lib/components/ConstellationSight2.svelte`** — v2 component (untouched; preserved on disk).
- **CLAUDE.md** Performance Rules 1–8, Architecture Principles, Working Agreements (#4 architecture validation per change).

---

**End of Architect.** Next document: **MIG-018 Plan** (six phases §1A→§1F with verification clauses). After Plan lands, **stop for Eisa's explicit Plan approval before §1A Build begins.**
