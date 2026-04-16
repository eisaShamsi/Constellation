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

*Next session pickup: run task B (boot_bundle cold-start) or the Index panel investigation, per user priority.*
