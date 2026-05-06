# PJ-034 / MIG-016 Architect — Sight instant-toggle perf

**Status**: Draft for Eisa's approval · 2026-05-06
**Predecessor lookup**: completed (per Law 3.2)
**Migration ID**: this is **MIG-016** (next sequential after MIG-015 which closed today)
**Pending Job ID**: **PJ-034** (next unused after PJ-033; allocated per the Stable-Reference-Numbers rule for net-new perf work)

---

## §1 — Function in hand

The Constellation Sight panel takes multi-second wall-clock to populate on first-toggle of a fresh session. Eisa's UX target: **first-toggle should feel instant** (sub-200ms perceived latency) and **edges should be hidden until the user hovers a node or runs a search**, so the panel paints clean nodes-only initially and reveals link structure on demand.

This is a **perf + UX combined MIG**. Three independent gains stack:

1. **Edges-on-hover gate** removes per-frame edge-draw cost from initial paint (analogous to Sky View's "nervous system" pattern at `graphEngine.ts:1880-1894`).
2. **Worker offload** of the JS analysis pipeline (Louvain + structural gaps + community profiles + bridge suggestions) keeps the main thread responsive during compute.
3. **SQLite-persisted cache** + **post-paint prewarm** make first-toggle instant when the cache is warm (most subsequent sessions).

---

## §2 — Predecessor lookup (per Law 3.2)

Five distinct predecessors. None forbid the planned design; three are templates we extend.

### Predecessor 1 — In-memory `lensDataStale` cache (2026-04-22 §55, commit `a19bc05`)

Where it lives: `+layout.svelte:753` (state) + `+layout.svelte:1260-1266` ($effect-driven invalidation) + `+layout.svelte:3343-3346` (cache-fast path on toggle).

What it does: keeps Sight compute results in JS memory after first toggle. Re-toggling without graph mutation hits cached state instantly.

What's cut: nothing — this layer stays. **MIG-016 supersedes it** by adding two stronger layers above (worker + SQLite). The in-memory cache becomes the L1 cache; SQLite becomes the L2; worker is the recompute path on cache miss.

### Predecessor 2 — Sight2 "all links solid" UX decision (2026-04-13, [SESSION-LOG-2026-04-13.md:9](lab/reports/SESSION-LOG-2026-04-13.md:9))

What it decided: Sight2's redesign chose links-on-by-default, with confidence shown by line thickness and direction by multi-arrow markers.

**This MIG reverses that decision.** Eisa-confirmed 2026-05-06: the new default is edges-hidden, revealed by hover or search match. Why reverse: the 2026-04-13 decision was made before the 2026-04-21 Sky View edges-on-hover work proved how much render headroom that pattern unlocks; the new default brings Sight to parity with Sky View's "nervous system" UX.

**What stays from 2026-04-13**: the 4-ring gravity-well layout, library-color sectors, multi-arrow direction markers, neighborhood-highlight on click, structured search. All of these are orthogonal to edge visibility.

### Predecessor 3 — Sky View edges-on-hover pattern (`graphEngine.ts:1880-1894`)

Template for the Sight edge-draw gate. Mechanism: `needsEdgeDraw` boolean computed from `(hovered || searchActive || focusMode || highlights)`; when false, the entire edge-iteration loop is skipped. When hovered, only the hovered node's `neighborMap.get(hoveredId)` is iterated.

Sight will mirror this pattern verbatim, in `ConstellationSight2.svelte:566-641` (the current edge-draw site). Pre-built `neighborMap` constructed once in `buildSimData()` (`ConstellationSight2.svelte:942`).

Important: Sight's layout is **deterministic radial gravity well** (`ConstellationSight2.svelte:274-357`), NOT force-spring. Removing edges from initial paint costs nothing in layout correctness — node positions are computed from `(centrality_ring, library_sector)` purely.

### Predecessor 4 — SQLite-persisted derived data (`sky_backfill.rs`)

Template for `sight_cache`. The `sky_backfill::maybe_schedule` pattern (background thread, schema-version stamp, resumable cursor, idempotent INSERT OR IGNORE, inter-batch sleep) is the playbook.

`sight_cache` table will follow the shape:
```sql
CREATE TABLE sight_cache (
    universe_id TEXT PRIMARY KEY,
    sky_version INTEGER NOT NULL,
    payload_json TEXT NOT NULL,      -- centrality, communities, gaps, profiles, bridges
    computed_at INTEGER NOT NULL     -- unix seconds
);
```

Storage cost on a 7,600-note Universe: ~250-500 KB serialized payload. Trivial vs. the ~5-6 MB `notes_fts` already holds.

### Predecessor 5 — Web Worker pattern (`src/lib/graph/forceWorker.ts`)

Template for `sightWorker.ts`. Mechanism: single-purpose worker, `postMessage` init + `terminate()` on destroy, transferable buffers on the output path (zero-copy).

Sight's worker will be different scope: it runs Louvain + structural gaps + community profiles + bridge suggestions + stratum-weighted centrality + universe-health scoring. Rust centrality (already an IPC) stays where it is — the worker's input includes the centrality result.

---

## §3 — Invariants that must not break

1. **Boot critical path stays untouched.** Sight is not invoked synchronously at boot. Prewarm is fire-and-forget after `boot:graph-ready` mark, gated by `requestIdleCallback` (existing `schedule()` helper at `+layout.svelte:2751-2760`).
2. **Sight toggle hits cache-fast when warm.** SQLite cache hit hydrates state directly into the existing `lensCentrality` / `lensCommunities` / `lensGaps` / etc. writables — no recompute, no IPC roundtrip.
3. **Worker recompute is off-main-thread.** When cache is stale, the JS analysis pipeline runs in `sightWorker.ts`. Main thread stays responsive throughout.
4. **Edges hidden by default.** First paint shows nodes only. Edges drawn only when `needsEdgeDraw` is true: hovered, search-active, neighborhood-selected, or annotation-hover.
5. **Search-driven highlight preserved.** When search is active, matched nodes' edges show. When user hovers a matched node, only its edges show. The existing search-match logic at `ConstellationSight2.svelte:582-600` is preserved.
6. **Per-Universe cache scope.** `sight_cache` is keyed by `universe_id`. Switching Universes invalidates implicitly (different key); switching back hits the prior Universe's cached row if still fresh.
7. **Crash-recoverable.** SQLite writes are atomic per WAL transaction. A crash mid-compute leaves no row written; next session sees a miss and recomputes. Worker `terminate()` on Svelte `onDestroy` prevents zombie state.
8. **M11 zero-diff.** `git diff src-tauri/src/lexicon/` returns empty.
9. **No new IPC on the keystroke hot path.** The `read_sight_cache` IPC fires once per Universe activation in idle time; never during typing.
10. **Cache invalidation tied to `skyVersion`.** Existing `+layout.svelte:1260-1266` $effect already flips `lensDataStale = true` on `skyVersion` bump. Cache write/read includes the `skyVersion` value at compute time; mismatch triggers recompute.
11. **`ConstellationSight.svelte` (the v1) is dead code.** Confirmed no live import. **Cleanup is out of scope for this MIG** — separate housekeeping commit (logged as PJ-NNN at next bump if Eisa wants).

---

## §4 — Design options (recap; B-4 already chosen)

| Path | Description | Rejected because |
|---|---|---|
| (A only) | Edges-on-hover gate, no compute changes | Doesn't address compute pipeline; first-toggle still has Rust+JS wait |
| (B-1) | Worker offload of JS analysis | Removes main-thread freeze but first-toggle still costs full pipeline |
| (B-2) | Background prewarm only | Cold-session first toggle still hits compute if prewarm hasn't finished; needs L2 cache |
| (B-3) | SQLite cache only | Solves first-toggle on warm cache; doesn't help mid-session post-edit |
| **(B-4 + A)** | **All four layers stacked** | **Recommended.** First-toggle instant when cache warm; non-blocking when cold; edges-on-hover removes paint cost |

---

## §5 — Recommendation: B-4 + Path A combined

The four layers stack cleanly. Each addresses a different scenario:

| Scenario | Layer that handles it | User-facing latency |
|---|---|---|
| First toggle, very-first session ever, cache empty + prewarm hasn't finished | Worker compute (off main thread) | Loading-state visible; main thread interactive |
| First toggle, fresh session, cache warm (most-common path post-first-session) | SQLite read + state hydration | Instant (sub-200ms) |
| First toggle, fresh session, cache warm, prewarm already finished | In-memory cache hit (already happens) | Instant |
| Re-toggle within session, no graph change | In-memory cache | Already instant |
| Toggle after editing notes (graph changed) | Worker recompute, then cache write | Sub-second on typical Universe; main thread interactive |
| First paint with all edges drawn | **Skipped entirely** (Path A) | Zero paint cost for edges |
| User hovers a node | Edge draw on `neighborMap.get(id)` | `O(degree)` per frame |
| User searches | Edge draw on matched nodes' neighborhoods | `O(matches × degree)` per frame |

---

## §6 — Cross-check (per Working Agreement #5)

- **Web Worker for analysis pipelines** — proven pattern. d3, vis.js, sigma.js all use workers for force layout. Constellation already uses `forceWorker.ts` for Sky View's d3 force.
- **SQLite-persisted derived data** — proven pattern. Constellation's own `sky_backfill.rs` (sky_nodes/sky_links table), `notes_fts` (FTS5), `term_vocab` (CTSE) all follow it.
- **Hover-to-show-edges** — proven pattern. Sky View ships it as the "nervous system" design (`graphEngine.ts:1880`); Eisa-validated on the 7,294-node / 217k-edge measurement (~0.3 fps → 60 fps).
- **No reinvention.**

---

## §7 — Plan summary (full plan in §2 doc)

Six phases. Per Eisa's call (2026-05-06): Phase 1 is instrumentation BEFORE architectural changes, so subsequent phases are grounded in measurement.

| Phase | Scope | Visible? | Boss test? |
|---|---|---|---|
| §1A | Instrumentation: `performance.mark`s in `toggleLens()` + a small "Show Sight perf trace" button in DevTools/Settings. No behaviour change. | No | No (data read by Claude from Boss test build) |
| §1B | Path A — edges-on-hover gate in `ConstellationSight2.svelte`; pre-built `neighborMap`; search-driven edge reveal preserved. | **Yes** | **Yes** |
| §1C | `sightWorker.ts` extraction — move Louvain + community profiles + structural gaps + bridge suggestions + stratum-weighted-centrality off the main thread. Cancellation token + transferable buffers. | No (perf only; no UX change) | Boss-feel verification |
| §1D | Post-paint prewarm — fire-and-forget after `boot:graph-ready`, gated by `requestIdleCallback`. Sight result ready by the time user toggles. | **Yes** (first-toggle becomes instant) | **Yes** |
| §1E | SQLite `sight_cache` table — persist results keyed by `(universe_id, sky_version)`. Hydrate on prewarm; recompute on miss/stale. | **Yes** (fresh-session first-toggle becomes instant) | **Yes** |
| §1F | Three-agent audit (invariants / drift / migration-path) | No | No |

Each phase commits independently with verification clause.

---

## §8 — PJ-025 reframe paragraph (per Eisa's option (a) + (b))

PJ-025 was retired as OBSOLETE in Pending Jobs v1.2 (2026-05-06) on the grounds that Sight is on-demand and not boot-rebuilt. That framing is correct for the **boot-impact** axis — Sight does not block app boot, and the Rule 8 audit's boot-time concern is satisfied. PJ-025's retirement stands.

PJ-034 (this MIG) addresses a **different perf axis**: the user-perceived latency on the first Sight toggle of a session. The 2026-04-22 §55 in-memory cache makes re-toggles instant within a session, but it doesn't survive a session boundary. On a 7,600-note Universe, that means a multi-second wait every time the user opens the app and toggles Sight for the first time. That latency is what Eisa wants to eliminate. PJ-034 is net-new perf work atop the now-stable architecture; the Stable-Reference-Numbers rule keeps PJ-025 retired with its OBSOLETE entry preserved.

---

## §9 — Open questions for Eisa

Three small ones; the rest of the design is locked.

1. **Cache key**: I propose `(universe_id, sky_version)`. Alternative: `(universe_id, content_hash_of_skynodes_skylinks)` — content hash is more robust against `skyVersion` drift, but slightly more compute. **Default to `sky_version`** unless you have a preference.
2. **Worker fail-safe**: if `new Worker(...)` fails (some WebKit / second-screen edge cases), fall back to inline computation on the main thread (same as `forceWorker.ts` precedent at `graphEngine.ts:1349-1353`)? **Default yes.**
3. **Phase 1A (instrumentation) Boss-test gate**: technically Phase 1A is just adding `performance.mark`s — but the test cycle is "Eisa toggles Sight on the build, I read the trace, calibrate phase budgets." That's a Boss-action gate even though no UX change. Worth flagging in the Plan as a "data collection" gate vs. a regular Boss test. **Default yes — call it Phase 1A's micro-test.**

---

**Awaiting Eisa's "Architect approved" before writing the Plan.** If yes, the Plan will spec Phase 1A's exact mark points so we can move directly to instrument + measure.
