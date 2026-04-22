# Source trace — what I see at HEAD `ef45c17`

## The data-flow chain (end-to-end, file:line)

1. **`initializeApp()`** `+layout.svelte:1404`
   - `invoke('constellation_boot_bundle')` awaited (line 1445)
   - `libraries.set(bundle.libraries)` (line 1455) — populates `$libraries` store synchronously
   - Fire-and-forget: `refreshLibraryCaches().catch(() => {})` (line 1549)

2. **`refreshLibraryCaches()`** `+layout.svelte:1898`
   - `const libraryList = $libraries` (line 1908) — reads current libraries
   - `await invoke('cache_boot_snapshot')` (line 1916)
   - If `!snapshot.is_cold`:
     - `allLibraryLinks = snapshot.links` (line 1923) — `$state.raw`? NO — currently `$state<NoteLink[]>([])` (line 535)
     - `allNotes = snapshot.notes.map(...)` (line 1925) — $state
     - If `libraryList.length > 0`:
       - `const { nodes, links: gLinks } = buildSkyData(snapshot.links, allNotes)` (line 1932)
       - `skyNodes = nodes` (line 1936) — **PLAIN let, not $state**
       - `skyLinks = gLinks` (line 1937) — **PLAIN let, not $state**
       - `starVersion++` (line 1938) — $state, intended signal
   - `performance.mark('boot:hydrated')` (line 1943)
   - `recordBootPerf()` writes scorecard with `note_count: allNotes.length`

3. **Template `{#if showSkyView} <GraphMindView nodes={skyNodes} ... />` at `+layout.svelte:3856-3908`**

4. **`GraphMindView.onMount`** `GraphMindView.svelte:606`
   - `engine = new GraphEngine(containerEl, engineConfig, callbacks)`
   - `await engine.init()` — PIXI setup (line 627)
   - `if (nodes.length > 0) { engine.setData(nodes, links, activeColorMap) }` (line 629)

5. **`GraphMindView $effect` at `GraphMindView.svelte:556`** — the "data changes → engine" effect
   ```ts
   $effect(() => {
     const len = nodes.length;  // tracked read of props.nodes
     ...
     if ((len !== prevNodeLen || ...) && len > 0 && engine) {
       engine.setData(dataNodes, dataLinks, cmap);
     }
   });
   ```

6. **`engine.onStatsReady(nc, ec, mc)` → `nodeCount = nc; edgeCount = ec`** — status bar reads these.

## Known-good facts from the user's scorecard

- `note_count: 7595` → `allNotes.length === 7595` at recordBootPerf time.
- `criterion_2_hydrated: FAIL` → hydrated_ms > 6000 — but this only proves the path RAN, not its correctness.
- Sky View UI shows `0 nodes · 0 edges` → `nodeCount === 0` and `edgeCount === 0` → `onStatsReady` never received positive stats → `engine.setData()` was never called with populated data, OR `setData` was called with empty data.

## Where the chain *could* break

Since `allNotes.length === 7595` (confirmed by scorecard) and `buildSkyData` iterates every note into `nodeMap` unconditionally, `buildSkyData` would return `{ nodes: Array(7595), ... }`. So `skyNodes = nodes` at line 1936 DID execute with 7595 items in the array.

The break must be **between `skyNodes` being assigned and `setData` being called on the engine**. Candidates:

### Candidate A — `libraryList.length === 0` at line 1931
`const libraryList = $libraries` at line 1908 snapshots the store. If `$libraries.length === 0` at that moment, buildSkyData never runs.

But: the user's screenshot shows "17 libraries · 7597 notes" in the status bar — so `$libraries` has entries. Could it have been empty the moment refreshLibraryCaches entered? `libraries.set(bundle.libraries)` runs synchronously at line 1455, BEFORE line 1549 kicks off refreshLibraryCaches. By the time `const libraryList = $libraries` reads, the store is populated. UNLESS the bundle is empty and fallbacks later populate.

**Refutation of A:** line 1920 awaits invoke, then reads `$libraries` after. It's synchronous JS; store.set has already propagated. Ruling out Candidate A.

### Candidate B — `skyNodes` reassignment not seen by the child template

`skyNodes` is plain `let`. `starVersion++` is meant to signal. But:

- `<GraphMindView nodes={skyNodes} />` — the template expression `skyNodes` is a plain-`let` read.
- Svelte 5 runes mode tracks reactive deps: `$state`, `$derived`, `$props`, stores. Plain `let` is NOT tracked.
- So the template effect's re-evaluation does not subscribe to `skyNodes` changes.
- `starVersion` is $state, but **nothing in the GraphMindView prop-binding subtree reads `starVersion`**. I grepped every read of starVersion:
  - line 570, 581 — inside `$derived.by` blocks for WiW overlay (wiwFilteredNodes, wiwFilteredLinks)
  - line 908 — inside `$effect` for right-sidebar local star graph
  - **None of them touch the main Sky View `nodes={skyNodes}` binding.**

So when `skyNodes = nodes; starVersion++` runs:
- No reactive scope that affects `<GraphMindView nodes={skyNodes} />` re-evaluates.
- The child's `nodes` prop is whatever value the parent's template last passed.

**How is Sky View displayed in the first place?** `{#if showSkyView}` — `showSkyView` IS $state. When the user clicks, it flips true, the `{#if}` effect re-runs, the template body re-evaluates, and `nodes={skyNodes}` is read at THAT moment.

So the question becomes: **at the moment the user opens Sky View, has `refreshLibraryCaches` already assigned `skyNodes`?**

- Prod: `hydrated_ms: 8669`. From paint at 419 ms to hydrated at 8669 ms, there is an **~8.25-second window** during which skyNodes is still `[]`.
- Dev: much slower overall boot, but the user typically waits for the app to "settle" before clicking. In dev mode the window may also be much wider, but the click usually comes after it closes.
- Prod: paint is instant (~420 ms). The app looks ready. User clicks Sky View within 1–2 seconds of paint. skyNodes is still empty.

GraphMindView mounts with `nodes = []`. `onMount`'s `if (nodes.length > 0) setData(...)` is skipped. `prevNodeLen = 0`. When `skyNodes` is assigned 5+ seconds later, the parent's template does not re-evaluate (no tracked dep). The child's `nodes` prop stays `[]`. The `$effect` at GraphMindView.svelte:556 never re-fires with `len > 0`.

**This is the bug.** Dev works only because the user's click happens after the slow dev boot completes. Prod fails because the fast paint invites an immediate click.

### Candidate C — worker URL fails in bundled app
`new Worker(new URL('./forceWorker.ts', import.meta.url), { type: 'module' })` — Vite handles this pattern. Tauri v2 bundles all chunks. Should work. But the worker is only started inside `setData`, which is never called in the broken scenario, so this axis is downstream of Candidate B. Ruling out as primary cause.

## Provisional verdict

**Root cause: Candidate B — Svelte 5 runes plain-`let` + timing race.**

The "store as plain let to avoid proxy overhead" optimization (comment at line 539-540) is broken: `starVersion++` is meant to signal changes, but **no reactive scope on the main Sky View chain ever reads `starVersion`**. The optimization only works for paths that explicitly `const _ver = starVersion` (WiW overlay, right-sidebar local star). The main Sky View was missed.

In dev, slower boot hides the race. In prod, fast paint exposes it.

Awaiting Agent 1 / Agent 2 / Agent 3 reports to confirm / refute.
