# PJ-034 / MIG-016 Plan v1 — Sight instant-toggle perf

**Architect**: `lab/reports/PJ-034-SIGHT-INSTANT-TOGGLE-ARCHITECT.md`
**Status**: Pending Eisa's "Plan approved." Cascade starts at §1A on go.
**MIG ID**: MIG-016 (closes PJ-034).

---

## Open questions resolved (per Eisa, 2026-05-06)

| # | Question | Resolution |
|---|---|---|
| 1 | Cache key | `(universe_id, sky_version)` (Claude's call; simple, leverages existing `skyVersion` invariant) |
| 2 | Worker fail-safe | **Yes** — try-catch around `new Worker(...)`; fall back to inline computation on main thread |
| 3 | Phase 1A Boss-test gate | **Yes** — data-collection gate (Boss toggles Sight on instrumented build; trace read by Claude) |

---

## Phase rollout

| Phase | Scope | Visible? | Boss test? |
|---|---|---|---|
| §1A | Instrumentation: `performance.mark`s in `toggleLens()` + a "Copy Sight perf trace" affordance | No (data only) | **Yes** (data-collection) |
| §1B | Path A — edges-on-hover gate + pre-built `neighborMap` + search-driven highlight | **Yes** | **Yes** |
| §1C | `sightWorker.ts` extraction — JS analysis pipeline off main thread | No (perf only) | Boss-feel verification |
| §1D | Post-paint prewarm — Sight result populated in idle time after `boot:graph-ready` | **Yes** (instant first-toggle) | **Yes** |
| §1E | SQLite `sight_cache` — cross-session persistence | **Yes** (instant fresh-session first-toggle) | **Yes** |
| §1F | Three-agent audit (invariants / drift / migration-path) | No | No |

---

## §1A — Instrumentation

### Goal
Add `performance.mark`s and `performance.measure`s around every step of `toggleLens()` so we have a measured time budget before architectural changes. Phases §1B–§1E will be calibrated against the trace from this build.

### Files touched
- `src/routes/+layout.svelte` — instrument `toggleLens()` at lines 3332-3427.

### Algorithm

```typescript
async function toggleLens() {
    if (lensActive) { lensActive = false; return; }
    if (!lensDataStale && lensHealth !== null) { lensActive = true; return; }

    performance.mark('sight:toggle:start');
    lensLoading = true;
    const computeVersion = skyVersion;

    try {
        performance.mark('sight:rust-centrality:start');
        const result = await invoke<...>('constellation_sight_centrality', { libraryPaths });
        performance.mark('sight:rust-centrality:end');
        performance.measure('sight:rust-centrality', 'sight:rust-centrality:start', 'sight:rust-centrality:end');

        performance.mark('sight:louvain:start');
        const clusterResult = detectClusters(...);
        performance.mark('sight:louvain:end');
        performance.measure('sight:louvain', 'sight:louvain:start', 'sight:louvain:end');

        // … same shape for: structural-gaps, universe-health, stratum-weighted-centrality,
        // top-bridges, community-profiles, bridge-suggestions, contradiction-detection.

        performance.mark('sight:state-flip:start');
        lensCentrality = ...;
        lensCommunities = ...;
        // … etc.
        lensActive = true;
        performance.mark('sight:state-flip:end');
        performance.measure('sight:state-flip', 'sight:state-flip:start', 'sight:state-flip:end');

        performance.mark('sight:toggle:end');
        performance.measure('sight:toggle:total', 'sight:toggle:start', 'sight:toggle:end');

        // Dump all measures to console for the Boss-test gate.
        const measures = performance.getEntriesByType('measure').filter(m => m.name.startsWith('sight:'));
        console.table(measures.map(m => ({ phase: m.name, duration_ms: Math.round(m.duration) })));
    } finally {
        lensLoading = false;
    }
}
```

The Sight2 mount is reactive (`{#if lensActive}` → component mounts), so the `onMount` cost (`buildSimData` + `computeGravityWellLayout` + first paint) gets its own measure inside `ConstellationSight2.svelte`:

```svelte
<script>
    onMount(() => {
        performance.mark('sight:mount:start');
        buildSimData();
        performance.mark('sight:mount:buildsim');
        computeGravityWellLayout();
        performance.mark('sight:mount:layout');
        fitToScreen();
        requestDraw();
        performance.mark('sight:mount:end');
        performance.measure('sight:mount:buildSimData', 'sight:mount:start', 'sight:mount:buildsim');
        performance.measure('sight:mount:layout', 'sight:mount:buildsim', 'sight:mount:layout');
        performance.measure('sight:mount:total', 'sight:mount:start', 'sight:mount:end');
    });
</script>
```

A small "Copy Sight perf trace" button appears when Sight is open (only when DevTools is open OR a `__SIGHT_TRACE__` URL flag is set — keeps the UI clean for production users).

### Verification

1. `npm run check` clean.
2. `cargo build --release --lib` clean (no Rust changes; routine).
3. **M11 zero-diff** check.
4. Boss-test (data collection): build, install, open Constellation, toggle Sight, copy console table to me. I'll read it and finalize per-phase budgets for §1B–§1E.

### Boss-test (Stage 1)

> **What this is**: Phase 1A is just measurement, no UX change. Sight will behave exactly as it does today, but every step of the cold-toggle path gets timed and printed to the developer console. The numbers tell us where the multi-second wait actually goes — Rust centrality? JS Louvain? Frontend mount? — so phases §1B–§1E target the right layers.
>
> **Stage 0 — install**: close Constellation. Run the new MSI (build timestamp will be in the Boss-test message). Open Constellation.
>
> **Step 1 — open DevTools**: press Ctrl+Shift+I to open the Constellation developer tools window. Click the "Console" tab.
>
> **Step 2 — toggle Sight**: click the Sight button in the dock (or use whatever shortcut you usually do). Wait for it to fully populate.
>
> **Step 3 — copy the trace**: a `console.table` will have printed in the console showing each phase's duration in milliseconds. Right-click the table → "Copy table" (or screenshot it). Send it to me.
>
> **Expected**: a tabular printout like:
>
> ```
> phase                              duration_ms
> sight:rust-centrality              ...
> sight:louvain                      ...
> sight:structural-gaps              ...
> sight:universe-health              ...
> sight:stratum-weighted-centrality  ...
> sight:top-bridges-sort             ...
> sight:community-profiles           ...
> sight:bridge-suggestions           ...
> sight:state-flip                   ...
> sight:mount:buildSimData           ...
> sight:mount:layout                 ...
> sight:mount:total                  ...
> sight:toggle:total                 ...
> ```
>
> Once I have the trace I'll calibrate §1B–§1E and produce the next Boss-test gate.
>
> **If you see this instead**:
> - **No console.table appears** → instrumentation didn't fire; check for JS errors in console.
> - **Some phases show `0` or negative durations** → mark-name typo; tell me which phase.

### Commit message skeleton
```
MIG-016 §1A — instrument toggleLens() with performance marks

Adds performance.mark/measure around every step of the cold-toggle
path in src/routes/+layout.svelte::toggleLens() and the cold-mount
path in ConstellationSight2.svelte::onMount(). On Sight toggle,
all measures are dumped to console.table for read-out.

No behaviour change. Data-collection only. The trace from this
build calibrates §1B-§1E budgets.
```

---

## §1B — Path A: edges-on-hover gate

### Goal
Render nodes-only on initial Sight paint. Edges drawn only when needed: hovered node, active search, neighborhood-selected, or annotation-hover. Mirrors Sky View's "nervous system" pattern at `graphEngine.ts:1880-1894`.

### Files touched
- `src/lib/components/ConstellationSight2.svelte`

### Algorithm

1. **Build `neighborMap` once in `buildSimData()`** (~line 942):
   ```typescript
   const neighborMap = new Map<string, Set<string>>();
   for (const link of simLinks) {
       const sId = link.source.id;
       const tId = link.target.id;
       if (!neighborMap.has(sId)) neighborMap.set(sId, new Set());
       if (!neighborMap.has(tId)) neighborMap.set(tId, new Set());
       neighborMap.get(sId)!.add(tId);
       neighborMap.get(tId)!.add(sId);
   }
   ```

2. **Edge-draw gate** in `draw()` (line 528) modeled on `graphEngine.ts:1891-1894`:
   ```typescript
   const needsEdgeDraw =
       hoveredNode !== null
       || searchActive
       || selectedNode !== null
       || annotationHover !== null;

   if (needsEdgeDraw) {
       drawLinks();   // existing function at line 566-641
   }
   ```

3. **Inside `drawLinks()`**: when `hoveredNode` is set, iterate only `neighborMap.get(hoveredNode.id)` and draw edges where the other endpoint is the hovered node OR a matched search node. When `searchActive`, iterate matched-node neighborhoods.

4. **`selectedNode` neighborhood-highlight at line 887-913**: replace the `simLinks` walk with `neighborMap.get(selectedNode.id)` lookup. `O(1)` instead of `O(E)`.

### Boss-test (Stage 1)

> **What this is**: Sight now opens with **nodes only** — no edges visible by default. Hovering a node reveals its connections. Searching highlights matched nodes and their connections. The change makes Sight populate visibly faster (no edges to draw on first paint) and keeps the visual cleaner until you ask for connection info.
>
> **Stage 0 — install**: standard MSI install over the current Constellation.
>
> **Step 1 — toggle Sight**: open Sight as you normally do.
>
> **Expected**: nodes appear in the gravity-well layout (rings + library sectors), but no lines between them. The panel should feel noticeably faster than before.
>
> **Step 2 — hover a node**: move your cursor over any node and pause.
>
> **Expected**: lines appear connecting that node to its neighbors. Move the cursor off → lines disappear.
>
> **Step 3 — search**: type a term in the Sight search box (e.g. an Arabic word from one of your notes).
>
> **Expected**: matched nodes glow / get badges (existing behavior). The connections between matched nodes appear as lines.
>
> **Step 4 — click a node** (existing neighborhood-select behavior): click any node.
>
> **Expected**: the clicked node + its neighbors stay highlighted; their connecting lines appear; click outside or click the same node again to clear.
>
> **If you see this instead**:
> - **All edges still visible by default** → §1B's `needsEdgeDraw` gate didn't take. Check if you installed the right build.
> - **Hovering doesn't reveal edges** → the gate fires but `neighborMap` lookup is wrong. Tell me which node + screenshot.
> - **Search no longer highlights matched edges** → search-active branch broken. Tell me the search term.
> - **Sight performance feels worse, not better** → the `O(E)` selectedNode walk wasn't replaced; tell me and I'll trace.

### Verification
1. `npm run check` clean (typing).
2. `cargo build --release --lib` clean (no Rust changes).
3. **M11 zero-diff**.
4. Boss-test passed.

### Commit message skeleton
```
MIG-016 §1B — edges-on-hover gate in Sight (Path A)

Mirrors Sky View's "nervous system" pattern from graphEngine.ts:1880.
Edges are no longer drawn by default in Sight — only when:
- A node is hovered (its neighborhood)
- Search is active (matched nodes' edges)
- A node is selected (neighborhood-highlight)
- Annotation hover

Pre-built neighborMap in buildSimData() makes hover-edge lookup
O(degree) instead of O(E). The selectedNode-highlight loop also
becomes O(1) lookup instead of O(E) walk.

Reverses the 2026-04-13 Sight2 redesign decision "all links solid
by default" — Eisa-confirmed 2026-05-06 after the 3-pass
cross-check identified Sky View's edges-on-hover as the
applicable template.

Boss test passed Stage 1.
```

---

## §1C — `sightWorker.ts` extraction

### Goal
Move JS analysis pipeline (Louvain + structural gaps + community profiles + bridge suggestions + stratum-weighted-centrality + universe-health) off the main thread into a Web Worker. Mirrors `forceWorker.ts` pattern.

### Files touched
- New: `src/lib/graph/sightWorker.ts` (~200 lines, modeled on `forceWorker.ts`).
- `src/routes/+layout.svelte` — `toggleLens()` rewires to spawn worker, postMessage init, listen for result.
- `src/lib/graph/clusterEngine.ts` etc. — verify the analysis functions are pure (no DOM access) so they're worker-safe.

### Algorithm

**Worker contract**:
```typescript
// init message
{ type: 'compute',
  computeId: number,
  centrality: Record<string, number>,  // from Rust IPC
  nodes: { id, name, libraryName, stratum }[],
  links: { source, target, link_type, weight, confidence }[],
  libraries: { name, color }[]
}

// result message
{ type: 'result',
  computeId: number,
  communities: Map<string, number>,
  gaps: StructuralGap[],
  health: UniverseHealth,
  profiles: CommunityProfile[],
  bridges: Bridge[],
  stratumWeightedCentrality: Record<string, number>,
  topBridges: Bridge[]
}
```

**Main-thread side** in `toggleLens()`:
```typescript
let sightWorker: Worker | null = null;
let currentComputeId = 0;

async function toggleLens() {
    // ... cache-fast path ...

    const myId = ++currentComputeId;

    // Step 1 — Rust centrality (unchanged)
    const result = await invoke('constellation_sight_centrality', { libraryPaths });
    if (myId !== currentComputeId) return;  // user toggled off / re-toggled

    // Step 2 — spawn worker (with try-catch fail-safe per Eisa Q2)
    let analysis;
    try {
        if (!sightWorker) {
            sightWorker = new Worker(new URL('$lib/graph/sightWorker.ts', import.meta.url), { type: 'module' });
        }
        analysis = await new Promise((resolve, reject) => {
            const onMessage = (ev: MessageEvent) => {
                if (ev.data.computeId !== myId) return;  // stale
                sightWorker!.removeEventListener('message', onMessage);
                resolve(ev.data);
            };
            sightWorker!.addEventListener('message', onMessage);
            sightWorker!.postMessage({ type: 'compute', computeId: myId, centrality: result.centrality, nodes: skyNodes, links: skyLinks, libraries: $libraries });
        });
    } catch (e) {
        console.warn('[sight] worker failed; falling back to inline compute:', e);
        analysis = computeInline(result.centrality, skyNodes, skyLinks, $libraries);
    }

    if (myId !== currentComputeId) return;

    // Step 3 — populate state from analysis (existing logic, just sourced from worker)
    lensCentrality = ...;
    lensCommunities = ...;
    // ... etc.
    lensActive = true;
}
```

`computeInline` is the current `toggleLens()` body, kept as fail-safe fallback (per Eisa Q2 = Yes).

### Verification
1. `npm run check` clean.
2. `cargo build --release --lib` clean.
3. **M11 zero-diff**.
4. Worker spawn / terminate correctly on Sight close.
5. CPU-throttle test: open DevTools → Performance → 6× CPU throttle. Toggle Sight. Type in another text input during compute. **Expected**: typing remains smooth (worker offload working).

### Commit message skeleton
```
MIG-016 §1C — sightWorker.ts: JS analysis pipeline off main thread

New src/lib/graph/sightWorker.ts handles Louvain (detectClusters),
structural-gap analysis, universe-health scoring, stratum-weighted-
centrality, top-bridges sort, community-profile build, and
bridge-suggestion compute. Mirrors forceWorker.ts pattern: single-
purpose worker, terminate on engine destroy, transferable buffers
on output.

Main thread becomes:
  Rust centrality (IPC, unchanged)
  → postMessage to worker with centrality + nodes + links
  → await result
  → state hydration

Worker fail-safe per Eisa-approved design: try-catch around
`new Worker(...)`; on failure, falls back to inline computation
on the main thread. Same UX, no worker benefit, but no break.

Cancellation: each compute carries a computeId. Stale results
(superseded by a newer toggle) are dropped on receipt.

No UX change. Boss-feel verification: typing during Sight compute
remains smooth.
```

---

## §1D — Post-paint prewarm

### Goal
Schedule Sight compute in idle time after `boot:graph-ready` mark fires, so by the time the user toggles Sight for the first time, results are already in memory.

### Files touched
- `src/routes/+layout.svelte` — add `prewarmSight()` function; schedule it after the existing `enrichNodesBackground()` resolves.

### Algorithm

```typescript
async function prewarmSight() {
    if (!lensDataStale) return;  // already warm (e.g. user toggled before prewarm fired)
    if (skyNodes.length === 0) return;  // empty universe; nothing to compute

    try {
        const result = await invoke('constellation_sight_centrality', { libraryPaths });
        // Same worker pipeline as toggleLens(), but DON'T set lensActive = true.
        const analysis = await runSightWorker(result.centrality, skyNodes, skyLinks, $libraries);
        // Hydrate lens* state writables but leave lensActive = false.
        lensCentrality = analysis.centrality;
        lensCommunities = analysis.communities;
        // ... etc.
        lensDataStale = false;
    } catch (e) {
        // Prewarm is best-effort; failure just means user pays the cost on first toggle.
        console.warn('[sight] prewarm failed; first toggle will recompute:', e);
    }
}

// Schedule after enrichNodesBackground (existing post-paint chain).
$effect(() => {
    if (graphReady && skyVersion > 0 && lensDataStale) {
        if (typeof requestIdleCallback !== 'undefined') {
            requestIdleCallback(() => prewarmSight(), { timeout: 5000 });
        } else {
            setTimeout(() => prewarmSight(), 100);
        }
    }
});
```

### Boss-test (Stage 1)

> **What this is**: Sight starts computing in the background as soon as Constellation has finished loading the graph (about 5-10 seconds after the app paints). By the time you click the Sight button, the math is done — the panel should appear instantly.
>
> **Stage 0 — install**: standard MSI install.
>
> **Step 1 — open Constellation**: launch the app. Wait until you see the sidebar populated with all your libraries (about 5-10 seconds — same as today).
>
> **Step 2 — wait an additional ~5 seconds**: this is the prewarm window. You don't have to do anything; Sight is computing in the background. The app should remain fully interactive (you can edit notes, open tabs, search) — the worker keeps the main thread free.
>
> **Step 3 — toggle Sight**: click the Sight button.
>
> **Expected**: the panel appears instantly with nodes-only (per §1B). No multi-second wait. **This is the goal.**
>
> **If you see this instead**:
> - **Sight still takes multi-seconds to populate** → prewarm hasn't run yet, OR has failed silently. Check console for `[sight] prewarm failed`. Or wait longer before toggling.
> - **App becomes unresponsive during the prewarm window** → worker isn't actually offloading. Check for `[sight] worker failed; falling back to inline compute` in console.
> - **Toggling immediately after launch is still slow** → that's expected during the first 5-10 seconds (prewarm hasn't started). Wait for the post-paint chain to fire.

### Verification
1. `npm run check` clean.
2. Boss-test passed.

### Commit message skeleton
```
MIG-016 §1D — post-paint prewarm: Sight ready by toggle time

Adds prewarmSight() called via $effect after boot:graph-ready
fires + skyVersion > 0 + lensDataStale. Gated by
requestIdleCallback so it doesn't compete with user IPC during
hydration.

prewarmSight() runs the same worker pipeline as toggleLens()
but doesn't set lensActive = true. State writables are hydrated;
the toggle action becomes a state flip, not a compute.

Best-effort: if prewarm fails (worker down, IPC error), first
toggle pays the cost as before. No regression.

Boss test passed Stage 1.
```

---

## §1E — SQLite `sight_cache`

### Goal
Persist Sight outputs across sessions so the very-first-toggle of a fresh session reads from disk. Cache key per Eisa Q1: `(universe_id, sky_version)`.

### Files touched
- New: `src-tauri/src/sight_cache.rs` — table schema, two IPC commands.
- `src-tauri/src/lib.rs` — register `read_sight_cache`, `write_sight_cache`.
- `src-tauri/src/search.rs::init_db` — add `sight_cache` table create + `schema_versions.sight_cache = 1` stamp.
- `src/lib/libraries/store.ts` — TS wrappers + types.
- `src/routes/+layout.svelte` — `prewarmSight()` reads cache first; on hit hydrates without recomputing; on miss/stale runs worker pipeline + writes the result.

### Algorithm

**Schema** (`sight_cache.rs`):
```rust
fn init_sight_cache_table(conn: &Connection) -> Result<(), String> {
    conn.execute("
        CREATE TABLE IF NOT EXISTS sight_cache (
            universe_id TEXT PRIMARY KEY,
            sky_version INTEGER NOT NULL,
            payload_json TEXT NOT NULL,
            computed_at INTEGER NOT NULL
        )
    ", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn read_sight_cache(app: tauri::AppHandle, expected_sky_version: i64) -> Result<Option<String>, String> {
    let universe_id = current_universe_id(&app)?;
    let state = app.state::<SearchState>();
    let conn = state.db.lock()...;
    let row: Option<(i64, String)> = conn.query_row(
        "SELECT sky_version, payload_json FROM sight_cache WHERE universe_id = ?1",
        rusqlite::params![universe_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).ok();
    match row {
        Some((cached_version, payload)) if cached_version == expected_sky_version => Ok(Some(payload)),
        _ => Ok(None)  // miss or stale
    }
}

#[tauri::command]
pub fn write_sight_cache(app: tauri::AppHandle, sky_version: i64, payload_json: String) -> Result<(), String> {
    let universe_id = current_universe_id(&app)?;
    let state = app.state::<SearchState>();
    let conn = state.db.lock()...;
    conn.execute(
        "INSERT OR REPLACE INTO sight_cache (universe_id, sky_version, payload_json, computed_at) VALUES (?1, ?2, ?3, strftime('%s','now'))",
        rusqlite::params![universe_id, sky_version, payload_json]
    ).map_err(|e| e.to_string())?;
    Ok(())
}
```

**Frontend** in `prewarmSight()`:
```typescript
async function prewarmSight() {
    if (!lensDataStale) return;
    const ver = skyVersion;

    // L2 cache check (SQLite).
    const cached = await invoke<string | null>('read_sight_cache', { expectedSkyVersion: ver });
    if (cached) {
        const payload = JSON.parse(cached);
        // Hydrate lens* state directly. No compute.
        lensCentrality = new Map(Object.entries(payload.centrality));
        lensCommunities = ...;
        // ... etc.
        lensDataStale = false;
        return;  // Sub-100ms path.
    }

    // L2 miss — run worker pipeline (existing §1D logic).
    const analysis = await runSightWorker(...);
    // Hydrate state.
    lensCentrality = analysis.centrality;
    // ... etc.
    lensDataStale = false;

    // Write back to L2 for next session.
    const payload = JSON.stringify({
        centrality: Object.fromEntries(analysis.centrality),
        communities: ...,
        gaps: analysis.gaps,
        // ... etc.
    });
    invoke('write_sight_cache', { skyVersion: ver, payloadJson: payload }).catch(() => {});
}
```

### Boss-test (Stage 1)

> **What this is**: Sight outputs are now saved to disk per Universe. The very first time you open the app and toggle Sight, the result was computed on a previous session and is loaded from disk — no worker, no compute. Faster than even the prewarm path.
>
> **Stage 0 — install**: standard MSI install.
>
> **Step 1 — first session**: open Constellation. Toggle Sight. (Behind the scenes: the prewarm + worker compute runs because the cache is empty for this Universe. The result is saved to disk.) Observe the result, then close Constellation.
>
> **Step 2 — second session**: open Constellation again. Toggle Sight as soon as the app paints — don't wait for the prewarm window.
>
> **Expected**: Sight appears almost instantly. No worker compute, no Rust centrality call. The result is hydrated from disk in sub-100ms.
>
> **Step 3 — graph mutation invalidation**: in the second session (with cache populated), edit any note (add a tag, add a link, change frontmatter). This bumps `skyVersion` and invalidates the cache. Toggle Sight off and on.
>
> **Expected**: Sight recomputes (worker runs, ~hundreds of ms). The new result is written back to disk. Next session will hit the new cache.
>
> **If you see this instead**:
> - **Second session still takes multi-seconds** → cache write didn't happen on first session, OR cache read fails. Check console for `read_sight_cache` errors.
> - **Cache hits but data is stale (old node positions, wrong centrality)** → `sky_version` invalidation broken. Tell me what graph change triggered it.
> - **Universe switch breaks** → cache key may not be including universe correctly. Tell me which Universes you switched between.

### Verification
1. `npm run check` clean.
2. `cargo build --release --lib` clean.
3. **M11 zero-diff**.
4. Boss-test passed.

### Commit message skeleton
```
MIG-016 §1E — SQLite sight_cache: cross-session persistence

New src-tauri/src/sight_cache.rs with read_sight_cache /
write_sight_cache IPCs. Table:

    sight_cache (
        universe_id TEXT PRIMARY KEY,
        sky_version INTEGER NOT NULL,
        payload_json TEXT NOT NULL,
        computed_at INTEGER NOT NULL
    )

prewarmSight() now checks sight_cache first. On hit (matching
universe_id + sky_version): hydrate state directly from JSON
payload, skip worker entirely. Sub-100ms path. On miss/stale:
run worker pipeline (§1D), write result to cache.

Cache key (universe_id, sky_version) per Eisa Q1. Schema version
stamp at schema_versions.sight_cache = 1. Forward-compatible.

Boss test passed Stage 1.
```

---

## §1F — Three-agent audit

Three parallel agents:

1. **Invariants agent** — verifies: (a) cache one-shot per `(universe_id, sky_version)`; (b) crash-recoverable (cache write atomic per WAL); (c) M11 zero-diff; (d) no new boot IPCs (prewarm uses an existing IPC + a new one called once in idle); (e) worker `terminate()` on Svelte `onDestroy`; (f) no `$state` mirrors of canonical state per Law 2.7.

2. **Drift agent** — checks: (a) other UI surfaces that listen to `lensCentrality` / `lensCommunities` etc. still work; (b) `searchActive` and `selectedNode` paths still draw their edges correctly under §1B's gate; (c) no stale references to the old `_lens_*` IPC names anywhere; (d) Sight2's existing features (multi-arrow, library colors, gravity wells, search) all preserved; (e) no Rust `lens.rs` references remain (was renamed to `sight.rs` in MIG-009).

3. **Migration-path agent** — checks: (a) fresh universe (no `sight_cache` row) — prewarm writes; (b) universe with stale cache (sky_version mismatch) — prewarm recomputes + overwrites; (c) switch active Universe — cache row scoped per-Universe, no cross-talk; (d) kill mid-compute (worker terminated, no zombie state); (e) concurrent toggle-on-during-prewarm (toggle hits the cached state if hydrated, else falls through to worker which ignores the prewarm's stale promise via computeId); (f) Sight panel close mid-render — `onDestroy` terminates worker cleanly.

P0/P1 fixed before close. P2/P3 logged as memory follow-ups + allocated PJ-NNN at next Pending Jobs bump.

---

## Closing the cascade

After §1F:
- `Constellation Pending Jobs vX.Y.md` — PJ-034 → SHIPPED.
- `lab/reports/SESSION-LOG-YYYY-MM-DD.md` — phase commits + state-of-standing.
- Orientation bumped (last v1.51 → next).
- MoCh next-block file written if 3-hour boundary crossed.

---

**Awaiting Eisa's "Plan approved" before §1A.**
