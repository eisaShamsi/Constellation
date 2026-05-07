# MIG-019 — Sight v3 Density + Time + Search (Plan)

**Companion to**: `MIG-019-V3-DENSITY-TIME-SEARCH-ARCHITECT.md`
**Phases**: 6 (§2A → §2F). Single Boss-test gate at §2E; audit at §2F.
**Approval mode**: per "Plan Approval = Build Approval" — after Eisa approves THIS plan, §2A → §2D cascade autonomously. Stop for Boss test at §2E. §2F closes after Boss-test pass + audit.

---

## §2A — TF-IDF compute + similarity IPC + schema (single commit)

### Steps

1. **Schema bump** (`src-tauri/src/search.rs`):
   - Bump `SIGHT_V3_SCHEMA_VERSION: i64 = 1` → `2`.
   - Remove `#[allow(dead_code)]` from the const (now read by §2A's invalidation logic).
   - In `init_db`, add `CREATE TABLE IF NOT EXISTS sight_v3_similarity_edges` block per Architect §2.2.
   - Add schema-version check: if stored sight_v3 version < `SIGHT_V3_SCHEMA_VERSION`, DELETE all rows from `sight_v3_layout` + `sight_v3_similarity_edges` + `sight_v3_layout_cursor` (force cold recompute on next toggle). Stamp the new version.

2. **TF-IDF compute** (`src-tauri/src/sight_layout.rs` — extend, ~400 LOC added):
   - `tokenize(body: &str) -> Vec<String>` — uses Constellation's FTS5 tokenizer if exposed; falls back to whitespace + lowercase + Arabic-normalization (mirroring `arabic/mod.rs`'s normalization).
   - `tf_vector(tokens: &[String]) -> HashMap<String, f64>` — term frequency normalized by doc length.
   - `idf_vector(corpus: &[Vec<String>]) -> HashMap<String, f64>` — `log(N / df_t)` per term across the corpus.
   - `compute_similarity_field(library_paths, k_top_terms, threshold) -> Vec<SimilarityEdge>`:
     - Read `note_meta.path + body` for each note in library set.
     - Tokenize each body.
     - Build IDF.
     - For each note, compute its top-k terms by TF×IDF.
     - Build inverted index: term → list of note indices.
     - For each note, find candidate notes via inverted index lookup of its top-k terms.
     - Compute cosine similarity for each candidate pair.
     - Keep similarities ≥ threshold (default 0.3).
     - Return Vec of `(path_a, path_b, similarity)` sorted by similarity descending.

3. **Cache** (`sight_layout.rs`):
   - On compute: persist results to `sight_v3_similarity_edges` keyed by `(library_set_hash, graph_version)`.
   - On read: SELECT cached rows for `(library_set_hash, current_version)`.

4. **IPC** registration (`lib.rs`):
   - Register `constellation_sight_v3_similarity_field` in `generate_handler!`.

5. **Unit tests** (3):
   - `tf_idf_basic_corpus` — synthetic 3-document corpus; verify IDF formula correct.
   - `cosine_similarity_known_vectors` — two synthetic TF-IDF vectors with known similarity.
   - `similarity_field_cache_roundtrip` — compute, persist, re-read, verify equality.

### Verification

- `cargo check` clean.
- `cargo test --lib sight_layout` passes 8/8 (5 from §1B + 3 new).
- Manual IPC call: `constellation_sight_v3_similarity_field(libraries, 50, 0.3)` returns ≤ 5000 edges in ≤ 500ms on Boss universe.

### Commit

`MIG-019 §2A — TF-IDF compute + similarity IPC + schema v2`

---

## §2B — Milky Way frontend render + Settings toggle (single commit)

### Steps

1. `src/lib/sight/similarity-cache.ts` (NEW): mirror of `layout-cache.ts`. Exports `fetchSimilarity(libPaths, kTerms, threshold)` + `invalidateSimilarity()`.

2. `src/lib/sight/v3/SightV3.svelte`:
   - Import `similarity-cache.ts`.
   - In `onMount` after `fetchLayout`, call `fetchSimilarity` with same `libraryPaths`.
   - Add `milkyWayContainer` Pixi Container; add to `app.stage` between background and `territoryContainer`.
   - New `drawMilkyWay()`:
     - For each high-similarity edge above threshold, draw a soft alpha-blended line from `pathToScreen[a]` to `pathToScreen[b]` with alpha proportional to similarity.
     - Apply `BlurFilter` to the container so lines merge into a band.
     - Cap at top-2000 edges for perf.
   - Wire `drawMilkyWay()` into `fullRedraw()` after `recomputeScreenPositions()` and before `drawTerritories()`.
   - Add `$effect` on `$appSettings.sight?.showMilkyWay` to toggle `milkyWayContainer.visible`.

3. `src/lib/components/SettingsModal.svelte`:
   - Add to Settings → Sight section: "Milky Way density wash" checkbox bound to `appSettings.sight.showMilkyWay`.

4. `src/lib/libraries/store.ts`:
   - Add `showMilkyWay?: boolean` to `sight` interface.
   - DEFAULT_SETTINGS.sight.showMilkyWay = `true`.

5. i18n keys: `settings.sight.showMilkyWay.label` + `.hint` in 15 locales.

### Verification

- `npm run check` clean.
- Locally toggle `SIGHT_V3_ENABLED = true`; click Sight v3 dock button; observe diffuse band of texture (not just lines) connecting high-similarity regions.
- Settings → Sight → "Milky Way density wash" toggle: ON → band visible; OFF → band hidden.

### Commit

`MIG-019 §2B — Milky Way density wash + Settings toggle`

---

## §2C — Calendar rim + month filter (single commit)

### Steps

1. `src/lib/sight/calendar-rim.ts` (NEW):
   - `monthArcSegments(viewport, ringIndex) -> { startAngle, endAngle, midAngle, monthIndex }[]` for 12 months around the dome.
   - `gregorianMonthLabels(locale) -> string[]` (Intl.DateTimeFormat).
   - `hijriMonthLabels(locale) -> string[]` (existing Hijri helpers in `lexicon/`).
   - `pickMonth(viewport, px, py) -> { calendar, monthIndex } | null` for hit-testing.

2. `src/lib/sight/v3/SightV3.svelte`:
   - Add `calendarRimContainer` Pixi Container above all base layers.
   - New `drawCalendarRim()`:
     - For each enabled calendar in `$appSettings.sight.calendarSystems`:
       - Compute monthArcSegments for that ring (innermost = first system).
       - Draw arc lines + month labels (using `Text` with appropriate i18n).
   - Wire into `fullRedraw()`.
   - Add pointer hit-testing for rim hover → preview filter (matched-month stars flare; others dim).
   - Add click hit-testing → persistent filter; click again or background → clear.

3. `src/lib/components/SettingsModal.svelte`:
   - Add to Settings → Sight section: "Calendar systems" multi-checkbox group (Gregorian, Hijri, Solar Hijri [placeholder], Hebrew [placeholder]).
   - Wire to `appSettings.sight.calendarSystems` array.

4. `store.ts`: `calendarSystems?: string[]` in interface; DEFAULT `['gregorian']`.

5. i18n: `settings.sight.calendarSystems.label` + per-system labels + `sightV3.calendar.months.gregorian.{0..11}` + `sightV3.calendar.months.hijri.{0..11}` (Hijri pulls from existing lexicon helpers).

### Verification

- `npm run check` clean.
- Locally: open Sight v3 → outer ring of 12 month markers visible; current month highlighted.
- Settings → Sight → enable Hijri → second concentric ring with Arabic Hijri month names.
- Hover a month → that month's notes flare; click → persistent filter; click background → clears.

### Commit

`MIG-019 §2C — calendar rim + Hijri toggle + month filter`

---

## §2D — Universe-health card in side panel (single commit)

### Steps

1. `src/lib/sight/universe-health.ts` (NEW):
   - `computeHealth(clusters, links, totalNotes) -> { modularity, dominance, entropy, connectivity, healthBadges: { ... } }`.
   - Use the existing `clusterEngine.ts::computeUniverseHealth` formulas to ensure parity with v2.
   - `healthBadges` returns "healthy" / "caution" / "imbalanced" per Concept Paper v1.1 §3.4 thresholds.

2. `src/lib/sight/v3/SightV3SidePanel.svelte`:
   - Add a new "Universe health" section above the note-detail section.
   - Always visible when no star is selected.
   - Tucks below note details when a star is selected.
   - Shows 4 metrics + colored badges.

3. `src/lib/sight/v3/SightV3.svelte`:
   - Compute health in `buildIndices()` from clusters + resolvedEdges.
   - Pass health data to `SightV3SidePanel` via props.

4. i18n: `sightV3.sidePanel.universeHealth` + `.modularity`, `.dominance`, `.entropy`, `.connectivity`, `.healthy`, `.caution`, `.imbalanced` in 15 locales.

### Verification

- `npm run check` clean.
- Locally: open Sight v3 → side panel shows "Universe health" with 4 metrics + badges.
- Click a star → side panel shows note details; universe-health card moves below.
- Cross-check (optional): if a developer toggles SIGHT_V2_ENABLED + SIGHT_V3_ENABLED both true, v2's health metrics match v3's exactly (same formulas).

### Commit

`MIG-019 §2D — universe-health card in side panel`

---

## §2E — Full search integration + always-on labels toggle (single commit; **BOSS-TEST GATE**)

### Steps

1. `src/lib/sight/v3/SightV3.svelte`:
   - Add prop `searchHubMatchIds: Set<string>` (mirrors v2's prop pattern).
   - `$effect` on `searchHubMatchIds`: when non-empty, apply flare to matched stars + halo to territories with matches + brighten edges between matches.
   - Flare animation: Pixi `Ticker` callback that pulses size + brightness over 600ms cycle.
   - Halo: territory polygon stroke widens + brightens for territories containing any match.
   - Edge brightening: same as click-on-star but scoped to match set.
   - Esc clears search filter (existing v2 behaviour mirror).

2. `src/lib/sight/v3/SightV3.svelte`:
   - Add `$effect` on `$appSettings.sight.alwaysOnLabels`.
   - When ON: render `Text` for each community territory at its centroid (always visible).
   - When OFF: only on hover (existing §1E behaviour).
   - Labels use Suwaidi gold tone with low alpha (~0.7).

3. `src/lib/components/SettingsModal.svelte`:
   - Add to Settings → Sight section: "Show constellation labels at rest" checkbox.
   - Wire to `appSettings.sight.alwaysOnLabels`.

4. `store.ts`: `alwaysOnLabels?: boolean` in interface; DEFAULT `false`.

5. `src/routes/+layout.svelte`: pass `searchHubMatchIds={searchHubMatchIds}` prop to `<SightV3 ... />`.

6. i18n: `settings.sight.alwaysOnLabels.label` + `.hint` in 15 locales.

### Verification

- `npm run check` clean.

### **BOSS-TEST GATE — pause for Eisa**

Tutorial (11 steps, per Architect §6):

1. **Build install path**: `src-tauri/target/release/constellation.exe` (post §2E commit). Confirm mtime later than commit time.
2. **Open Sight v3**. Click star-icon dock button.
3. **Observe Milky Way at-rest**: faint diffuse band of texture under the stars, visibly denser in some regions.
4. **Right-click → "Hide Milky Way"** → band disappears. Right-click → "Show" → returns.
5. **Calendar rim** visible: 12 Gregorian month markers; current month highlighted.
6. **Settings → Sight → Calendar systems → enable Hijri** → second ring with Hijri Arabic months.
7. **Hover a Gregorian month** → that month's notes flare; click → persistent filter; click background → clears.
8. **Universe-health card** visible in side panel: M / D / E / C metrics + color badges. Numeric values reasonable for your universe.
9. **Search** a note title → matched stars flare; territories halo; edges between matches brighten. Esc clears.
10. **Settings → Sight → "Show constellation labels at rest"** → flip ON. Labels appear on territories. Flip OFF → labels disappear.
11. **No regression**: hover star + click star + double-click + projection toggle + Esc + open other surfaces (Sky View, Map, OrgChart, Inspector360).

Pass criteria: all 11 observable; no console errors; performance feels instant on warm cache.

### Commit

`MIG-019 §2E — full search integration + always-on labels + Boss-test tutorial`

(With `SIGHT_V3_ENABLED` already `true` in committed source from MIG-018 §1F.)

---

## §2F — Three-agent audit + close-out (single commit)

### Steps

1. **Three-agent audit** (parallel):
   - Invariants agent: verify all 14 invariants from Architect §3.
   - Drift agent: verify §4 drift map honored.
   - Migration-path agent: verify §5 scenarios (8 rows).

2. Fix any P0 / P1.

3. Bump `docs/Constellation Orientation & Onboarding v1.58.md` → `v1.59.md`:
   - v1.59 preamble: MIG-019 closes; phase 2 of 3 done; MIG-020 next-up.
   - §8 Migrations table: MIG-019 ✅ Closed; MIG-020 next-up.

4. Bump `docs/Constellation Pending Jobs v1.7.md` → `v1.8.md`:
   - PJ-038 status update: 2 of 3 MIGs done.
   - Top-of-queue rotation: MIG-020 next-up.

5. Append to today's session log with §2A→§2F summary.

6. Write `lab/reports/MIG-019-V3-DENSITY-TIME-SEARCH-AUDIT.md`.

### Verification

- All Architect §3 invariants verified by audit.
- 0 P0 / 0 P1 / acceptable P2 / P3.
- `npm run check`: 1 pre-existing PJ-012 error; 0 new.

### Commit

`MIG-019 closes — v3 density + time + search live + audit + Pending Jobs v1.8 + orientation v1.59`

---

## Phase verification checklist (recap)

| Phase | Lands | Verification | User-testable? |
|---|---|---|---|
| §2A | TF-IDF compute + IPC + schema v2 | cargo check + 3 unit tests | No |
| §2B | Milky Way render + Settings toggle | npm check + visual sanity | No |
| §2C | Calendar rim + Hijri toggle + month filter | npm check + visual sanity | No |
| §2D | Universe-health card | npm check + cross-validate | No |
| §2E | Full search integration + always-on labels | npm check + **BOSS TEST** | **YES — pause for Eisa** |
| §2F | Three-agent audit + bump + close-out | audit clean | No |

§2A → §2D cascade autonomously. Pause at §2E for Boss test. §2F runs after Boss-test pass.

---

## Time estimate

- §2A: 2-3 hours (TF-IDF compute + tests + IPC + cache + schema bump).
- §2B: 1-2 hours (Milky Way Pixi render + BlurFilter + Settings toggle).
- §2C: 2 hours (calendar rim + Hijri labels + month-filter interaction).
- §2D: 1 hour (universe-health card module + side panel section).
- §2E: 2-3 hours (search flares + halos + always-on labels).
- §2F: 1-2 hours (audit + flip-already-done + bump docs).

Total: ~9-13 hours. One long session or two shorter ones.

---

**End of Plan.** Awaiting Eisa's explicit approval before §2A begins.
