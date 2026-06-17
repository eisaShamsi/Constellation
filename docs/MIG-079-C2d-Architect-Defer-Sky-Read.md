# MIG-079 §C.2d — Architect: Defer the Sky read off the boot critical path

**Status:** Architect (Phase 1 of the /migration). Awaiting Boss approval of the Plan before any code.
**Date:** 2026-06-17. **Branch:** `main`. **Active universe:** "Eisa Cognitive Knowledge" (7,651–7,653 notes, 1.97 GB DB, 233,995 `sky_links` rows).
**Function in hand:** the **boot Sky read** — `cache_boot_snapshot_sky` (`cache.rs:788`), invoked at boot by `loadGraph` in `+layout.svelte`, which reads the whole 233,995-row `sky_links` table to populate `skyNodes`/`skyLinks` for the Sky View / CNS / Lens / Sight surfaces.

---

## 1. Problem (measured, not guessed)

After §C.2b/§C.3/§C.2c shipped, the app is **responsive at ~1.1 s** (`hydrated_ms`), but `graph_ready` is still **~11 s — and it is almost entirely *queue* wait**, not work.

Boot-perf history (`lab/boot-perf/read-boot-history.py`, active universe), the post-§C.2c boots:

| boot | hydrated_ms | graph_ready_ms | graph queue_ms | read_links | read_tags |
|---|---|---|---|---|---|
| [16] 07:59 | 1188 | 10,951 | 9,612 | None | 81 |
| [17] 08:23 | 967 | 11,454 | 10,185 | None | 70 |
| [18] 09:01 | 1120 | 11,684 | 10,117 | None | 80 |
| [21] 13:47 | 1123 | 11,286 | 9,494 | None | 90 |

The graph IPC **body** is trivial (`read_links=None` — §C.2b deferred it; `read_tags≈80 ms`). So `graph_ready` is **~10 s of pure queue wait**.

**Root cause — read directly off the IPC arrival trace** (`ipc_arrival_log`, cold boot `mqfizrd9`):

```
+   879 ms   cache_boot_snapshot_sky      ← sky IPC starts
+ 12253 ms   cache_boot_snapshot_sky      ← 11,374 ms gap, NO other IPC dispatched in between
+ 12631 ms   federation_get_warnings
```

A **single 11,374 ms gap with no other IPC dispatched** = the synchronous sky command monopolised the one IPC dispatch thread for 11.4 s. Everything else (the graph IPC, federation, perf-trace) queued behind it.

**It is I/O-bound on cold pages, not compute-bound.** On a *warm* boot (`mqfj19sm`) the identical sky read is **250 ms** (`+224 → +474`). The 11.4 s is entirely cold disk reads of 233,995 `sky_links` rows from the 1.97 GB DB — the untouched twin of the `note_links` read that §C.2b deferred.

> **Correction to one agent finding:** the Rust mapping agent estimated the sky read at "200–400 ms / 2–3 % of the critical path." That is the **warm** body time. The authoritative **cold** figure, measured off the IPC trace, is **~11.4 s** and it blocks the entire IPC thread. The cold/warm gap is the whole story.

---

## 2. Territory map (verified, with citations)

### 2.1 Rust side (`cache.rs`, `search.rs`, `lib.rs`)
- `cache_boot_snapshot_sky` is **`#[tauri::command]` — synchronous** (`cache.rs:788`), so it runs on and blocks the IPC dispatch thread. The §9.1 precedent `constellation_map_universe` is **`#[tauri::command(async)]`** (`map.rs:565`) — async-ifying it closed Boot Criterion 2 (`core_queue_ms` 19.9 s → 4 ms, a 5,100× reduction).
- **Only the frontend boot + tests** consume `cache_boot_snapshot_sky` / `read_sky_*_in_schema`. Zero other Rust dependents (`lib.rs:446` is the only registration). Deferring the read is fully isolated server-side.
- **`sky_nodes`/`sky_links` are fully write-time maintained** by triggers: `note_meta_sky_ai/ad/au` (`search.rs:3108/3117/3146`), `note_links_sky_ai/ad/au` (`search.rs:3005/3013/3022`), plus stratum/maturity triggers (`search.rs:3276–3384`). **Rule 8 is already satisfied** — §C.2d defers the *read*, never the maintenance. The on-open read always finds a fresh snapshot.
- The read is `SELECT source_path, target_name, link_type FROM {schema}.sky_links` — a **full-table scan, no WHERE**. A covering index would **not** help the cold read (an unconstrained scan reads every leaf page regardless; the cost is cold page faults). Query optimisation is exhausted; **off-the-critical-path is the only remaining win.**
- Readiness/fallback is conservative: `is_federated_sky_ready` (`cache.rs:754`) gates on every schema's `sky` version ≥ `SKY_SCHEMA_VERSION`; if any lags, the command returns `{nodes:[], links:[], is_ready:false}` — never partial/stale data.

### 2.2 Frontend consumers (`+layout.svelte`, `livePreview.ts`)
- **`skyNodes`/`skyLinks`** (`$state.raw`, `:870/871`) feed: GraphMindView main Sky View (`:6422`, inside `{#if showSkyView}`), the WiW overlay via `wiwFiltered*` (`:6497`), ConstellationSight/Lens (`:6348`, `{#if lensActive}`), SightV3 (`:6322`), SightV4 (`:6554`), ExpressionForge (`:6537`, `notes={skyNodes}`), LocalSkyView right-sidebar star tab (`localSky*`, `:7185`). **Every visual consumer is already behind a visibility/lazy condition.**
- **`skyNodePathSet`** (`:880`, mirrored from `skyNodes` via `$effect`) is read by `livePreview.ts:1043` to gate the CNS gesture button on Lens rows — the one consumer touched **at boot, before any Sky open**. It already **degrades permissively**: `inGraphOrBooting = skySet.size === 0 || skySet.has(path)` (`livePreview.ts:1043`), and CNS open falls back to fit-to-screen when the node isn't found (`ConstellationSight2.svelte:1134`). So an empty set at boot = all Lens-row CNS icons show; opening CNS just fits-to-screen. Acceptable, documented degradation.
- **Lazy-mount precedent (LL-022):** `mapEverOpened`/`orgChartEverOpened`/`catalogerEverOpened` (`:651–656`), reset in `handleUniverseSwitch` (`:2457–2459`), two-tier `{#if *EverOpened}` (mount) + `class:*-visible` (CSS). **No `skyEverOpened` exists** — Sky View's markup is always present.
- **`loadGraph`** (`:3317`) currently fires `skyPromise` (`:3341`) **and awaits it** (`:3405`) **before** setting `graphReady` (`:3448`) — so even the "concurrent" sky kick-off gates `graph_ready`. The "max(graph, sky)" comment at `:3332` is wrong for a synchronous, awaited command.
- **`safeBootMode` already skips** the sky IPC (`:3342`) — a precedent for "boot without sky."
- **The `buildSkyData` fallback** (`:3429–3435`) calls `await ensureFullLinks()` then guards on `allLibraryLinks.length > 0`. **But `ensureFullLinks` is a no-op when `perNoteLinkQueries` is on** (`:820`, the default) — so under today's defaults the fallback **cannot build** (it never populates `allLibraryLinks`). The only live sky path post-§C.2c is the `sky.isReady` branch. The Plan must handle the `is_ready=false` (mid-backfill) case explicitly rather than relying on the dead fallback.

### 2.3 Render primitive — **Option C (nodes-now, edges-later) is NOT viable**
The PIXI/d3-force Sky View needs **all edges at init**: `GraphEngine.setData` iterates every link to build `neighborMap`/`outgoingMap`/`incomingMap` (`GraphMindView` engine), and `forceWorker.ts` attaches `forceLink()` to the full edge array and runs a z-smoothing pass over all resolved edges. There is **no windowing / LOD / streaming**. You cannot feed a force simulation nodes first and stream edges in. **Edges and nodes must load together when Sky opens.** (This directly answers the handover's open question.)

### 2.4 Cross-window — **no constraint**
The second screen builds its **own** sky data locally via `scanLibraryLinks` + `buildSkyData` (`SecondScreenPage.svelte:290/310`); the main window never emits `skyNodes`/`skyLinks` to it. Deferring the main window's sky read does **not** affect the second screen (display-not-domain holds). The second screen's Sky companion is already event-driven/lazy on its own context.

---

## 3. WA#5 cross-check — defer-to-open is the **standard**, not an invention
- **Obsidian v1.7.2 "Defer views"** (official dev docs): all views are `DeferredView` until their tab is visible, *explicitly to improve startup time and memory* — the proposed move almost verbatim, in the same product category (local-first MD PKM with a graph view).
- **Negative corroboration:** eager full-graph load is the documented failure mode — Logseq "freezes loading 18,500 pages", Obsidian's graph "blocks the editor" on large vaults. Constellation's 11 s boot stall is the same disease; the field's cure is "don't build the graph until asked."
- **Neo4j Bloom** is query-first / expand-on-demand — never "load the whole graph."
- **Refinement the field would add:** an *after-idle background warm-up* (prefetch) of the read so the first open is warm — gated behind a true idle signal and cancellable if the user opens mid-warm-up. This matches Constellation's own Write-Time Derivation first-time-population rule ("back-fill in background after paint, resumable"). **Caveat:** don't over-prefetch and starve I/O.
- **Keep the source-of-truth write-time-materialised** (already true) — defer the read/render, never the index.

**Verdict:** "defer the 234k-edge graph read off boot, load on graph-view-open" is the textbook pattern. The only inventive-risk is the *inverse* (eager-at-boot) — which is exactly what we're removing.

---

## 4. Design options (speed / effort / risk)

### Option A — async-ify `cache_boot_snapshot_sky` only (`#[tauri::command(async)]`)
- **Speed/effort:** trivial (one attribute).
- **Effect:** sky runs on a worker thread; other boot IPCs no longer queue behind it. **But** `loadGraph` still `await`s `skyPromise` before `graph_ready`, so `graph_ready` stays ~11 s unless also decoupled; and the 11.4 s cold disk read still happens at boot, thrashing I/O while the user starts working.
- **Risk:** low. **Insufficient alone** — moves the timing, not the cost. Necessary *companion*, not the answer.

### Option B — defer the sky read off boot (LL-022 lazy-mount; load on first Sky-surface open) ✅ recommended core
- **Speed/effort:** moderate — frontend lazy scaffolding + an epoch guard + a universe-switch reset; one Rust attribute.
- **Effect:** boot fires no sky IPC. `graph_ready` drops to ~graph body (paint + ~80 ms tags) ≈ **sub-second**. The 11.4 s cold read happens **once, on first Sky-surface open** (memoised, behind a loading state, on a worker thread). Boot disk is quiet — the user's first actions (type/search) aren't competing with a 234k-row scan.
- **Risk:** medium, fully mapped (see §5 invariants). The seams are: the `skyEverOpened` gate must cover **all** sky surfaces; a `_skyEpoch` stale-guard + universe-switch reset (closes a **pre-existing latent bug** — `skyNodes` is not reset today); the `is_ready=false` path; the federation:ready handlers.

### Option B+ — Option B plus an after-idle background warm-up (WA#5 refinement)
- **Speed/effort:** B plus a cancellable idle prefetch.
- **Effect:** boot fast **and** first Sky-open is warm (~250 ms) *if* the warm-up finished; if the user opens first, the warm-up is cancelled and becomes the on-open load.
- **Risk:** medium+. **Reintroduces the 11.4 s disk read at boot** (now async/non-blocking, but still I/O the user's early actions compete with). Trades "quiet boot disk" for "instant first open." **Recommend deferring** — ship pure-lazy first, measure how the cold first-open feels, add warm-up only if Boss finds it slow.

### Option C — nodes at boot, edges deferred/virtualised
- **RULED OUT** by the render primitive (§2.3): d3-force needs all edges at init; no streaming/LOD exists. Loading only nodes for `skyNodePathSet` isn't worth a boot read (it degrades permissively without it).

---

## 5. Invariants that must not break (Audit will verify each)

- **INV-1 — Data identity.** When any sky surface opens, it renders byte-identical nodes/links to today (just later). No data loss.
- **INV-2 — `skyNodePathSet` gate.** Degrades permissively at boot (Lens-row CNS icons show; CNS opens to fit-to-screen). Must become exact once sky loads.
- **INV-3 — Universe-switch isolation (NEW guard).** No stale sky from the previous universe. Add `_skyEpoch` (mirror `_linksEpoch:808/2439`) + reset `skyNodes/skyLinks/skyVersion/localSky*/skyEverOpened/skyReady` in `handleUniverseSwitch`. **This closes a pre-existing latent race** (`skyNodes` is not reset today — `frontend` agent finding).
- **INV-4 — Second screen independent.** Builds its own sky; unaffected. (Confirmed.)
- **INV-5 — Write-time maintenance untouched.** Triggers keep `sky_*` current; we defer only the read. Rule 8 preserved.
- **INV-6 — Boot responsiveness.** `hydrated_ms` not regressed; `graph_ready` improves; **no new IPC on the boot critical path.** Measure before/after on the boot-history tool, cold (PC restart).
- **INV-7 — Federation.** The per-schema federated read (MIG-061) still works on Sky-open; `federation:ready` handlers (`:2752/2816`) route through the new on-demand path and respect `_skyEpoch`.
- **INV-8 — On-open must not freeze.** The on-demand read runs **async** (worker thread) so the app stays responsive behind a loading state. **Editor-Surface Gate:** the sky path touches no note content/save/lifecycle → content-integrity class structurally untouched (read-path only, same class as §C.2b/§C.2c).

---

## 6. Migration-path cases (Audit phase)
- **First boot / mid-backfill (`is_ready=false`):** with lazy sky, this surfaces on Sky-open, not boot. The on-demand path must show "indexing…" and retry on `federation:ready` (NOT rely on the dead `buildSkyData` fallback under `perNoteLinkQueries`).
- **Schema mismatch / rollback:** the Rust command is unchanged except the `async` attribute; `sky_*` tables unchanged → forward/back-compatible. A pre-§C.2d binary simply reads sky at boot again.
- **`safeBootMode`:** already skips sky — consistent with the deferral.
- **Kill mid-warm-up (if B+ ever ships):** the warm-up is cancellable; a kill leaves `sky_*` intact (write-time maintained, resumable back-fill).

---

## 7. Recommendation
**Option B (defer sky to first open via LL-022 lazy-mount) + async-ify `cache_boot_snapshot_sky` [Option A as a required companion] + the `_skyEpoch` stale-guard.** Defer Option B+ (idle warm-up) as a documented, measure-first follow-up.

**The one user-visible trade to approve:** today the 11 s sky read happens at *every* boot (blocking `graph_ready`); after this change it happens **once, the first time you open Sky View / CNS / Lens / Sight in a session**, behind a loading spinner, while the rest of the app stays responsive. Boot itself drops from ~11 s-to-graph-ready toward the sub-second editor floor.

---

## 8. Open decision for Boss
1. **Pure-lazy now, warm-up later (recommended)** vs **include the idle warm-up now (B+)**. Recommended: pure-lazy first; it's the clean boot win and keeps the boot disk quiet. Add warm-up only if the first cold Sky-open feels slow.
