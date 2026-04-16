# Session Log — 2026-04-16

## Headline

**Sky View rendering bug fixed** (PIXI v8 + Tauri CSP collision — one-line import). Boot pipeline split `cache_boot_snapshot` into `_core` (awaited) + `_graph` (deferred via `requestIdleCallback`) per plan `optimized-humming-trinket.md`. Cold-start revealed SQLite first-read is the remaining bottleneck; warm-start is now ~100× faster.

## Work in order

### 1. Executed the boot-snapshot split (plan `optimized-humming-trinket.md`)

Followed yesterday's landed plan to close Boot Criterion 2 (hydrated_ms ≤ 6s):

**Rust (`src-tauri/src/cache.rs`)**
- Added `BootSnapshotCore { notes, is_cold }` and `BootSnapshotGraph { links, tags }` types.
- Added `cache_boot_snapshot_core(app)` — runs only `SELECT name, path, library_name FROM note_meta` + cold-check.
- Added `cache_boot_snapshot_graph(app)` — runs `read_links` + `read_tags` paths, factored helpers.
- Kept `cache_boot_snapshot(app)` as a back-compat shim calling both (second screen, tests, ambient callers).
- Registered both new commands in `src-tauri/src/lib.rs`.

**Frontend (`src/routes/+layout.svelte`)**
- Rewrote `refreshLibraryCaches()` as a two-phase flow:
  1. Phase 1 (awaited): `cache_boot_snapshot_core` → populate `allNotes`, mark `boot:hydrated`, call `recordBootPerf()`.
  2. Phase 2 (deferred via `requestIdleCallback`, fallback `setTimeout 0`): `cache_boot_snapshot_graph` → assign `allLibraryLinks` / `allLibraryTags`, run `buildSkyData`, bump `starVersion`, call `recordBootPerfGraphPhase()`.
- Added `graphReady = $state(false)` flag.
- Split `recordBootPerf()` into core + graph phases, both feeding a shared `buildBootPerfReport(includeGraphPhase)` — report now exposes `graph_ready_ms`.
- Added a `void allLibraryLinks.length` read at the top of the sidebar effect so backlinks / outgoing panels re-read when Phase 2 lands.

### 2. Measured — the split works, but exposed a deeper bottleneck

First test report (cold start, user's trial Universe, 7,595 notes):

```
paint_ms=409   libraries_loaded_ms=13167   hydrated_ms=21117   graph_ready_ms=39592
criterion_1_paint=PASS   criterion_2_hydrated=FAIL
```

Second run on the same Universe (warm disk cache) — core=27ms, graph=226ms. **~100× speedup with warm cache.**

**Diagnosis:** The IPC split is correct and measurably helps, but hydrated_ms is now gated by `constellation_boot_bundle` taking ~13s on first cold read — Rust-side file I/O against hundreds of small JSON files (universe, libraries, settings, bookmarks, workspaces, property_types, bases, child universes, child-universe libraries) before SQLite is touched. On a cold NTFS page cache this dominates.

**Implication:** Criterion 2 cold-start fix requires either parallelizing `constellation_boot_bundle`'s inner reads or a priming pass at OS-level. Deferred as open item B.

### 3. Sky View rendering bug — 13-component disassembly

After the split, user reported Sky View showed `0 nodes · 0 edges` despite correct counts in the status bar. Data was reaching the engine; nothing rendered.

**Method:** broke the render pipeline into 13 components (data push → PIXI Application init → container setup → layout → draw tick → texture cache → etc.) and instrumented each boundary with `[SV#<N>]` probes via a temporary `sv_log` facility (Rust command `sv_log_frontend` + frontend `_svLog` helper) so the logs survive the WebView2 console closing.

**Probes A–I inside `init()`** captured the exact throw:

```
Current environment does not allow unsafe-eval,
please use pixi.js/unsafe-eval module to enable support.
```

PIXI v8 generates WebGL shader programs at runtime via `new Function(...)`. Tauri's default CSP forbids `unsafe-eval`. PIXI internally catches the throw and leaves the renderer half-constructed — no crash, no red border, silent empty canvas.

**Fix (one line, `src/lib/graph/graphEngine.ts`):**

```ts
import 'pixi.js/unsafe-eval';   // MUST be first PIXI-related import
import { Application, Container, Graphics, Text } from 'pixi.js';
```

`pixi.js/unsafe-eval` ships a pre-compiled shader generator that does not use `new Function()`. Relaxing app-wide CSP was rejected — weakens security across the whole frontend.

**User verified:** *"It is Working. I also opened a note from the SV, checked its backlinks and tags, all fine."*

### 4. Cleanup

- Removed all `[SV#*]` probes from `graphEngine.ts` (class fields, probes, progressive `init()` probes A–I, `setData` probe, worker `[SV#11]` probes, `draw` `[SV#13a]` counters).
- Removed all `[SV#*]` probes from `+layout.svelte` (`[SV#6]`, `[SV#6a]`, `[SV#6b]`, `[SV#6c]` effects; `_svLog` / `_svLog2` helpers).
- Removed Rust-side `sv_log` / `sv_log_frontend` facility entirely — function, Tauri command, registration in `lib.rs`, dangling callers in `cache.rs`, and orphan imports (`tauri::Manager`, `crate::search::SearchState`).
- Added **LL-019** to `docs/LESSONS-LEARNED.md` (PIXI v8 + Tauri CSP — import `pixi.js/unsafe-eval` as a side-effect).

## Commits expected

1. **Boot snapshot split + Sky View PIXI/CSP fix + cleanup + LL-019** — squashed, one commit covering plan implementation, the one-line PIXI fix, probe cleanup in all files, and the lesson entry.

## Open items (priority order)

1. **B: `constellation_boot_bundle` 13s cold-start** — parallelize its inner reads (`rayon`-style `par_iter` across child universes + JSON reads) or fire it pre-warm. Dominant hydrated_ms cost on cold disk.
2. **Index panel broken in dev AND `.exe`** — user reports the Index panel (different from Sky View) doesn't render. Not yet investigated. Similar diagnostic pattern likely applies.
3. **`starVersion` → `skyVersion` rename** — naming drifted (it's a Sky View signal, not a "star" signal). Cosmetic but worth doing before more call sites accumulate.
4. **BOOT-BUDGET.md Criterion 2** still open on cold start (~21s vs 6s target). Split landed; needs B to close.
5. **Criterion 3 / 4 / 5** of `BOOT-BUDGET.md` — RSS budget, stat-only sweep, kill-mid-index recovery — untouched this session.

## Files touched

- `src-tauri/src/cache.rs` — new split commands, shim, helper factoring, `sv_log` removed.
- `src-tauri/src/lib.rs` — registered new commands, removed `sv_log_frontend`.
- `src/routes/+layout.svelte` — two-phase flow, `graphReady`, extended boot-perf report, probes removed.
- `src/lib/graph/graphEngine.ts` — `import 'pixi.js/unsafe-eval'`, probes + `svLog` helper + `invoke` import removed.
- `docs/LESSONS-LEARNED.md` — LL-019 appended.
- `lab/reports/SESSION-LOG-2026-04-16.md` — this file.

## Lessons applied

- **LL-014** (three strikes): Sky View problem was solved at the root — one-line fix — after progressive probes captured the exact throw. No patching loop.
- **LL-017** (adversarial review): yesterday's expert-panel pattern was not needed today; the diagnostic methodology (13-component disassembly) carried this one.
- **LL-018** (paint-first): confirmed — `boot:paint` still fires in 409ms on cold start despite everything else being slow. The shell-first principle held.
- **LL-019** (new): PIXI v8 + Tauri CSP requires the `unsafe-eval` pre-compiled variant imported as a side-effect.

---

## Addendum — Fix 1 + Fix 2 landed (same day, evening)

After the split-snapshot landing exposed cold-start as the bottleneck, a third round of deep attribution proved the cause was **not** `constellation_boot_bundle` (~31 ms total) but two different things:

1. **Tauri IPC queue contention at the OS I/O layer.** All 34 boot IPCs (16 watchers + 16 appearances + 1 stats + 1 snapshot) raced simultaneously. Wall-clock measurements showed every IPC converging on the same endpoint (~27.7 s) — textbook shared-resource serialization. The ship-gate IPC `cache_boot_snapshot_core` had only 8,094 ms of actual Rust work but a 19,616 ms wall-clock wait.
2. **Row-store full-scan cost.** `SELECT name, path, library_name FROM note_meta` forced SQLite to read every page of the wide row-store table (body_text + JSON blobs) to project three narrow text columns.

### Fix 1 — reorder fan-out (frontend)

`src/routes/+layout.svelte`: moved `refreshLibraryCaches()` from fire-and-forget to **awaited, before** the watcher/appearance/stats fan-out. The core snapshot gets exclusive I/O until `boot:hydrated` fires; every other boot IPC runs after.

### Fix 2 — covering index (Rust / SQLite)

`src-tauri/src/search.rs`, inside `init_db()`:

```sql
CREATE INDEX IF NOT EXISTS idx_note_boot_snapshot
    ON note_meta(name, path, library_name);
```

Planner switches from full table scan to index-only scan — three narrow TEXT columns instead of full-page reads of wide rows. `IF NOT EXISTS` + no schema-version bump means existing DBs pick it up on next launch without reindex.

### Instrumentation retained

`cache.rs` `BootSnapshotCore` and `BootSnapshotGraph` now carry `timings_ms: Vec<(String, u64)>` with per-phase Rust wall-clock (ensure_db, open_reader, read_notes / count_notes, read_links, read_tags). Frontend wall-clock around each awaited invoke is written to `boot-perf.latest.json` (`cache_snapshot_core_wall_ms`, `cache_snapshot_core_server_timings`, …). This delta (wall − server_sum) is the IPC queue / contention time. Standing instrumentation — useful for any future boot-perf work.

### Measured result (same binary, clean universe, second boot)

```
Before fixes:         hydrated_ms = 28,425    cache_snapshot_core_wall = 27,710
After Fix 1 + Fix 2:  hydrated_ms = 10,759    cache_snapshot_core_wall = 10,244
                      read_notes (Rust) = 5 ms (was 8,021 ms)
                      ensure_db (Rust) = 110 ms (after one-time 7,717 ms CREATE INDEX on first launch)
```

**2.65× improvement.** `read_notes` dropped 1,600×. Criterion 2 still misses: **10.7 s vs 6 s target**.

### Residual — unattributed 10.1 s dispatch wait

With `cache_snapshot_core_wall = 10,244` and server phases summing to only 121 ms (ensure_db:110 + open_reader:6 + read_notes:5), there is a **10,123 ms gap** between frontend invoke dispatch and the Rust function actually running. The Rust work is done — but *something* is delaying the handler from being picked up by Tauri's dispatcher or the `spawn_blocking` pool.

After three rounds of instrumentation (LL-014), the disciplined move is to stop drilling and file the residual. It would require another Rust-side `eprintln!` at the very top of the handler to measure pre-execution queue time, and we do not yet know whether the cause is (a) Tauri dispatcher thread, (b) blocking thread pool saturation, or (c) some module-level Svelte IPC firing during onMount before initializeApp resolves.

**Criterion 2 status: NEAR (not PASS).** 10,759 ms vs 6,000 ms target. Shipping what works; residual filed as LL-020 and listed below as open item B.1.

### Files touched (addendum)

- `src-tauri/src/cache.rs` — added `timings_ms` field and per-phase Instant probes to `BootSnapshotCore` / `BootSnapshotGraph`.
- `src-tauri/src/search.rs` — `CREATE INDEX IF NOT EXISTS idx_note_boot_snapshot ON note_meta(name, path, library_name)` in `init_db()`.
- `src/routes/+layout.svelte` — `await refreshLibraryCaches()` before fan-out; wall-clock probes around every awaited boot invoke; extended `buildBootPerfReport` with `*_wall_ms` and `*_server_timings` fields.
- `lab/boot-perf/boot-bundle-cold-start.md` — investigation writeup (smoking-gun analysis, warm-run paradox, fix rationale, retained for history).
- `docs/LESSONS-LEARNED.md` — LL-020 (wall-vs-server measurement distinguishes IPC queue from Rust work; covering indexes on row-stores for narrow projections).

### Commits expected (addendum)

2. **Boot perf: reorder fan-out + covering index for note_meta snapshot** — Fix 1 + Fix 2, instrumentation, investigation writeup, LL-020.

### Open items updated

- **B.1 — Cold-start IPC dispatch delay (~10 s).** Unknown whether cause is Tauri dispatcher or blocking-thread pool or pre-onMount IPC. Would need `eprintln!` instrumentation at the entry of the handler to attribute. Deferred.
- **Criterion 2 — NEAR, not PASS.** 10.7 s vs 6.0 s. Two wins shipped; third round deferred per LL-014.

---

*Next session pickup: run the Index panel investigation (task D).*
