# MIG-018 — Sight v3 Projection Foundation (Plan)

**Companion to**: `MIG-018-V3-PROJECTION-FOUNDATION-ARCHITECT.md`
**Phases**: 6 (§1A → §1F). Each lands as one commit. Phase §1E is the Boss-test gate; phase §1F is the audit + close-out.
**Approval mode**: per the Migration Rule + "Plan Approval = Build Approval" — once Eisa approves THIS plan, §1A → §1E cascade autonomously, with Boss-test pause at §1E. §1F's three-agent audit gates the close-out commit.

---

## §1A — Schema + Rust skeleton (single commit)

### Steps

1. Add three tables in `src-tauri/src/init_db.rs` (or current schema bootstrap location — confirm during build):
   - `sight_v3_layout` (the layout cache).
   - `sight_v3_layout_cursor` (the resumable backfill cursor).
   - `sight_v3_graph_version` (the invalidation pivot).
   - All `CREATE TABLE IF NOT EXISTS` (idempotent).
2. Add invalidation triggers on `note_links` insert/delete and on the relevant `note_meta` columns that affect the graph (frontmatter tags). Each trigger does `INSERT OR REPLACE INTO sight_v3_graph_version (library_set_hash, version, bumped_at) VALUES (..., COALESCE((SELECT version FROM sight_v3_graph_version WHERE library_set_hash = ...), 0) + 1, unixepoch())`.
3. Create `src-tauri/src/sight_layout.rs` with:
   - Module-level docstring referencing this Plan + the Concept Paper v1.1 §3.
   - Stub `compute_layout_embedding(library_paths: Vec<String>, k_landmarks: usize) -> Result<Vec<LayoutPoint>, String>` returning empty Vec.
   - `LayoutPoint` struct with the columns of `sight_v3_layout`.
4. Register the IPC in `src-tauri/src/lib.rs` `generate_handler!` macro. Name: `constellation_sight_v3_layout`.
5. Add a brief unit test that the stub IPC compiles, returns Ok([]) when called with empty `library_paths`.

### Verification

- `cargo check` clean.
- `npm run tauri build` succeeds.
- After first boot, `sqlite3 constellation.db ".schema sight_v3_layout"` shows the new table.
- IPC callable from frontend devtools (returns []).

### Commit

`MIG-018 §1A — schema + sight_layout.rs skeleton + IPC registered`

---

## §1B — Landmark MDS embedding compute (single commit)

### Steps

1. In `sight_layout.rs`:
   - Implement helper `compute_pairwise_distances` (BFS from each landmark; O(k·V·E)).
   - Implement `pick_landmarks` (top-k by betweenness, ties broken lexicographically by note id).
   - Implement classical MDS on the V×k landmark distance matrix → 2D coords for landmarks.
   - Implement triangulation embedding for non-landmarks (bilinear interpolation via 3 nearest landmarks; standard Landmark-MDS).
   - Normalize to unit-disk: scale by max distance from origin so max radius = 1.0.
2. Compose with existing centrality + community IPCs:
   - Call `brandes_betweenness` (or its approximate variant) for centrality scores.
   - Call community detection (frontend's Louvain is the source of truth — but for cache write-time, mirror in Rust if needed; otherwise persist the embedding only and let frontend overlay community-id at read time. Decision pending §1B build.).
3. Persist `LayoutPoint` rows to `sight_v3_layout` table inside a transaction.
4. Bump `sight_v3_graph_version.version` for the `library_set_hash`.
5. Backfill: if `sight_v3_layout_cursor.completed = 0`, resume from where it left off.

### Verification

- Unit test: a 10-node ring graph yields a circular embedding (visually verifiable).
- Unit test: re-running on same input produces identical output (determinism).
- Manual test on a synthetic 50-note universe: embedding looks sensible (cluster notes group together).
- `sight_v3_layout` populated after one IPC call on Boss's 7,600-note universe; ≤ 500ms.

### Commit

`MIG-018 §1B — Landmark-MDS embedding compute + persistence + invalidation`

---

## §1C — Frontend skeleton + dock button (single commit)

### Steps

1. Add `src/lib/sight/projection.ts`:
   - Pure function `embedToScreen({x, y}, mode: 'lambert' | 'stereographic', viewport): {x, y}`.
   - Lambert: `r' = 2 * sin(arctan(r) / 2)`; angle preserved.
   - Stereographic: `r' = 2 * tan(arctan(r) / 2)`; alternative formula.
   - Both return screen-pixel coordinates given viewport center + radius.
2. Add `src/lib/sight/layout-cache.ts`:
   - `async fetchLayout(libraryPaths: string[], k: number): Promise<LayoutPoint[]>`.
   - Calls `constellation_sight_v3_layout` IPC.
   - Caches in module-level Map keyed by `libraryPaths.join('|')`.
3. Add `src/lib/sight/community-territory.ts`:
   - `computeAlphaShape(points: {x, y, communityId}[], alpha: number): Map<communityId, Polygon>`.
   - Falls back to convex hull on degenerate communities.
4. Add `src/lib/sight/v3/SightV3.svelte`:
   - Empty Pixi `Application` mounted on a `<canvas ref>`.
   - Renders placeholder text: "Sight v3 — projection foundation (MIG-018 §1C)."
   - On mount: calls `fetchLayout()` and logs the count.
   - On unmount: `Application.destroy()`.
5. Add `src/lib/sight/v3/SightV3SidePanel.svelte` skeleton (empty for now).
6. Edit `src/lib/sight/engine.ts`:
   - Add `export const SIGHT_V3_ENABLED = false;` (kept false until §1E Boss-test passes).
7. Edit `src/routes/+layout.svelte`:
   - Import `{ SIGHT_V3_ENABLED }`, `SightV3` component.
   - Add `let sightV3Active = $state(false);` near `lensActive`.
   - Add `sightV3Active` to the `fullPageActive` $derived.
   - Add `if (sightV3Active) { sightV3Active = false; return; }` to the Escape handler.
   - Add v3 dock button gated by `{#if SIGHT_V3_ENABLED && $appSettings.enabledFeatures?.constellationSight !== false}` — adjacent to where the v2 button used to be (now also gated by SIGHT_V2_ENABLED). When clicked: sets `sightV3Active = true`, clears all other full-page flags.
   - Add v3 modal mount: `<div class="sight-v3-overlay" class:visible={sightV3Active && SIGHT_V3_ENABLED}> {#if sightV3Active && SIGHT_V3_ENABLED} <SightV3 onClose={() => sightV3Active = false} /> {/if} </div>`.

### Verification

- `npm run check` clean (1 pre-existing PJ-012 error, 0 new).
- `npm run tauri build` succeeds.
- Dock button does NOT render in production (because `SIGHT_V3_ENABLED = false`).
- Locally toggle `SIGHT_V3_ENABLED = true`, rebuild — dock button shows; clicking opens an empty Pixi canvas with the placeholder text. Console logs the LayoutPoint count.

### Commit

`MIG-018 §1C — frontend skeleton + dock button gated behind SIGHT_V3_ENABLED`

---

## §1D — Star rendering + projection toggle (single commit)

### Steps

1. In `SightV3.svelte`:
   - Replace placeholder text with star rendering. For each LayoutPoint:
     - Compute screen position via `embedToScreen(layoutPoint, currentProjection, viewport)`.
     - Draw a star sprite sized by `centrality_norm` (logarithmic scale: 6 visual magnitudes).
     - Color star by lifecycle freshness (recently traversed = brighter; defer to `lifecycle` field; if absent, use a default warm-white).
   - On viewport resize: recompute screen positions; redraw.
   - Add a star-coordinate index (KD-tree or simple grid) for hover hit-testing in O(log N).
2. In Settings → Sight (NEW section in `SettingsModal.svelte`):
   - Add radio group: Projection: ◉ Lambert (equal-area)  ◯ Stereographic (equal-angle)
   - Bind to `appSettings.sight.projection` (with merge into DEFAULT_SETTINGS).
3. In `SightV3.svelte`: subscribe to `$appSettings.sight.projection`; on change, redraw stars (no recompute, just re-project).
4. Add the `appSettings.sight` shape to the TypeScript interface in `store.ts`. Default: `{ projection: 'lambert', alwaysOnLabels: false, calendarSystems: ['gregorian'] }`.
5. Add 6 i18n keys per locale: `settings.sight.title`, `.projection.label`, `.projection.lambert`, `.projection.stereographic`, `sightV3.title`, `commands.openSight`.

### Verification

- `npm run check` clean.
- Locally `SIGHT_V3_ENABLED = true`; clicking the v3 dock button opens a dome of stars at correct positions.
- Settings → Sight → Projection toggle: switching from Lambert → Stereographic visibly redraws the dome (stars shift to new positions); same notes still occupy the dome.
- Star size + brightness varies by centrality (visual sanity check on Boss's universe).

### Commit

`MIG-018 §1D — star rendering + Lambert/stereographic projection toggle`

---

## §1E — Territories + connector lines + hover/click (single commit; **BOSS-TEST GATE**)

### Steps

1. In `SightV3.svelte`:
   - Compute community territories via `computeAlphaShape(layoutPoints, alpha=0.05)`.
   - Render territory polygons on the base Pixi layer with cycled-pastel fill (id mod 8 → palette index) at low alpha (~0.15).
   - Render constellation labels at territory centroids — only on hover/select per Eisa's call (Concept Paper v1.1 §11 Q6). Hidden at rest.
   - Render faint connector lines on the base layer for all wikilinks + shared-tag edges (alpha ~0.12).
2. Add focus overlay Pixi layer on top of base:
   - On hover: brighten incident edges from base alpha to focus alpha (~0.85). Clear on hover-leave.
   - On click: assign focus star; brighten ALL edges within that star's community to focus alpha; show side panel.
   - On double-click: open the note in editor (use existing `openNoteTab` flow).
3. Add tooltip DOM overlay:
   - On star hover: show note title + centrality rank + community name + lifecycle stage.
   - RTL-aware (uses `dir="auto"` on the title element).
4. Add side panel `SightV3SidePanel.svelte`:
   - On focus-star: show note metadata + top-5 incoming wikilinks + top-5 outgoing wikilinks + structural-gap suggestions involving this note.
   - "Open in editor" button (mirrors double-click behavior).
   - "Clear focus" button.
5. Search integration (basic, full version in MIG-019):
   - When `searchHubMatchIds` is non-empty: matched stars flare (size pulse via Pixi tween); non-matching stars dim.
   - Esc / click-background clears (existing search-hub clear path).
6. Add 6 more i18n keys: `sightV3.tooltip.centralityRank`, `.community`, `.lifecycle`, `sightV3.sidePanel.incomingLinks`, `.outgoingLinks`, `.structuralGaps`.

### Verification

- `npm run check` clean.
- Locally `SIGHT_V3_ENABLED = true`. Manual exercise of all 12 acceptance criteria from Architect §6.
- Performance check: on Boss's 7,600-note universe, first-toggle ≤ 500ms cold cache; ≤ 50ms warm.

### **BOSS-TEST GATE — pause for Eisa**

Eisa runs the install per the Boss-test tutorial that I'll write inline in the §1E commit message + as a session-log entry. The tutorial walks through:

1. **Install the build.** `src-tauri/target/release/constellation.exe` (Eisa's preference per `feedback_prefer_exe_over_msi.md`). Confirm build mtime is post-§1E commit.
2. **Open Sight v3.** Click the new v3 dock button (a star icon, distinct from the now-hidden v2 eye icon).
3. **Observe at-rest.** Confirm: dome of stars; soft territory regions in pastel colors; faint white connector lines visible (not invisible — this is the v3 reframe of Principle 6); current month highlighted on rim (rim itself deferred to MIG-019, may not appear); no labels at rest; no console errors.
4. **Hover a star.** Cursor over any star → tooltip pops up with note title, centrality rank, community, lifecycle. Edges from that star to its neighbors brighten visibly.
5. **Click a star.** That star's whole constellation's edges brighten; side panel slides in from right; shows note metadata + linked notes; other constellations dim.
6. **Double-click a star.** Sight v3 closes; the clicked note opens in the editor in a new tab.
7. **Open Settings → Sight.** Confirm: "Projection" row with Lambert (selected) / Stereographic radio. Click Stereographic. Close Settings.
8. **Re-open Sight v3.** Stars are at new positions (stereographic projection). Same notes, same constellations, redistributed across the dome.
9. **Switch back to Lambert.** Stars shift back. Stable spatial memory: a note that was at "9 o'clock on the dome" in Lambert returns to that exact position.
10. **Search.** Open Search Hub, query a note title. Matching stars flare; rest dim. Press Escape. Filter clears.
11. **No regression.** Open Sky View, Map, OrgChart, Inspector360, Index. All still work normally.

**Pass criteria**: all 11 steps observable; no console errors; performance feels instant on warm cache. **Fail at any step → stop, fix, retest.**

### Commit

`MIG-018 §1E — territories + connector lines + hover/click + side panel + Boss-test tutorial`

(With `SIGHT_V3_ENABLED` still `false` in committed source; Eisa toggles it locally for the Boss test.)

---

## §1F — Three-agent audit + flip enable + close-out (single commit)

### Steps

1. **Three-agent audit (parallel)**:
   - **Invariants**: verify Architect §3's 13 invariants hold against the diff.
   - **Drift**: verify Architect §4's drift map is honored — no implicit consumers of `SIGHT_V3_ENABLED`, no schema-table drift.
   - **Migration-path**: verify Architect §5's 7 scenarios still hold after the full §1A–§1E build.
2. Fix any P0 / P1 findings.
3. Flip `SIGHT_V3_ENABLED = true` in `src/lib/sight/engine.ts`.
4. Bump orientation v1.57 → v1.58 inline:
   - v1.58 preamble: MIG-018 closes; v3 projection foundation live in production; MIG-019 next-up.
   - §8 Migrations table: add MIG-018 row ✅ Closed.
5. Bump Pending Jobs v1.6 → v1.7:
   - PJ-038 status: "Confirmed · next-up" → "**In-Progress · MIG-018 closes; MIG-019 next-up**".
   - §8 trajectory updated with MIG-018 closure timestamp + commit hash.
6. Append to today's session log with the close-out summary.
7. Write the audit report at `lab/reports/MIG-018-V3-PROJECTION-FOUNDATION-AUDIT.md` consolidating the three-agent findings.

### Verification

- All Architect §3 invariants verified by audit.
- 0 P0 / 0 P1 / acceptable P2 / P3.
- `npm run check`: 1 pre-existing PJ-012 error; 0 new.
- Production build with `SIGHT_V3_ENABLED = true` shows the v3 dock button to Eisa.

### Commit

`MIG-018 closes — v3 projection foundation live + audit + Pending Jobs v1.7 + orientation v1.58`

---

## Phase verification checklist (recap)

| Phase | Lands | Verification | User-testable? |
|---|---|---|---|
| §1A | Schema + Rust skeleton | cargo check + IPC stub callable | No |
| §1B | Landmark-MDS compute | unit tests + manual IPC + DB inspect | No |
| §1C | Frontend skeleton + dock button | npm check + tauri build | No |
| §1D | Star rendering + projection toggle | local toggle ON; visual sanity | No (still gated off in committed source) |
| §1E | Territories + lines + hover/click | local toggle ON; **BOSS TEST** | **YES — pause for Eisa** |
| §1F | Audit + flip + close-out | three-agent audit clean; flip enable; bump docs | No (audit is the gate) |

The cascade pauses at §1E for Boss test. §1A–§1D commit one after another without pause; §1F runs after §1E passes.

---

## Time estimate

Rough order-of-magnitude:
- §1A: 30 min (schema + IPC scaffold).
- §1B: 2-3 hours (Landmark-MDS implementation + tests + persistence + invalidation triggers).
- §1C: 1 hour (frontend scaffold).
- §1D: 1-2 hours (Pixi rendering + projection toggle + Settings panel).
- §1E: 3-4 hours (territories + alpha-shape + faint lines + focus overlay + tooltip + side panel + search basics).
- §1F: 1-2 hours (audit + flip + doc bumps).

Total: ~8-12 hours of focused build, plus Boss-test cycles. Could be one long session or two shorter ones.

---

**End of Plan.** Awaiting Eisa's explicit approval before §1A begins.
