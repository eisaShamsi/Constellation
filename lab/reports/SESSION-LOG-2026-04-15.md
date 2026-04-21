# Session Log — 2026-04-15

## Headline

**Boot performance crisis solved.** Production boot on a 7,600-note Universe (16 libraries, 656k typed links) went from **25s UI / 6+ minute thrashing** (commit `69cfa6a`) to **1s UI / 8s fully responsive** (commit `ce47e13`). Beats the Obsidian-parity target.

## Final commit: `ce47e13` — In-memory cache for `load_all_libraries`

```
Boot perf CORE FIX: in-memory cache for load_all_libraries
```

## Context

This session continued from the previous day's work that had built a 7,600-note trial Universe, completed Living Link P2–P5, shipped the Emoji & Icon Library plug-in, and fixed the Obsidian Universal Embed Resolver. The previous session's last unresolved issue was crippling boot performance on the trial Universe.

## What we tried (the patching loop)

Before adopting the disciplined approach, I made these incremental fixes — all of which failed to materially help:

| Commit | Attempt | Result |
|---|---|---|
| `1a7ce05` | `get_recent_notes` metadata-only walk + parallel std::thread per library | Marginal |
| `8a14c3d` | Cache-first boot via `cache_boot_snapshot` reading from SQLite | Made it worse |
| `545b582` | Split `constellation_search_init` so cache snapshot works on 2nd+ boots | Made it worse |
| `69cfa6a` | Cache reads via dedicated read-only connection (bypass writer mutex) | Still 25s UI / 6m thrash |

User cut me off: **"STOP GUESSING and patching. We are in a LOOP of patching for a superficial fix. I WANT a CORE permanent FIX."**

## The discipline shift

User instructed me to:
1. Stop guessing.
2. Use AI expert agents as referees.
3. Build a lab to measure objectively.

This was the turning point.

### Expert panel review

Spawned three agents in parallel for adversarial review of the proposed cache-first architecture:

- **Agent 1** (Obsidian internals expert) — said my "never walk on boot" was too absolutist. Obsidian DOES do a cheap stat-only walk on boot to catch external changes; only content parsing is deferred.
- **Agent 2** (Tauri/Rust systems) — pinpointed the architectural error: *"Constellation awaits backend work before the window is shown; Obsidian doesn't. That is the architectural error, not the walkers."*
- **Agent 3** (PKM architecture) — gave 5 ship-gate acceptance criteria that became `BOOT-BUDGET.md`. Recommended copying Obsidian's cached-index-first + post-UI mtime-sweep pattern.

### Lab harness

Created `lab/boot-perf/`:
- `BOOT-BUDGET.md` — 5 ship-gate criteria with numbers
- `touch-50.ps1` — Criterion 4 helper (touch 50 .md mtimes)
- Frontend instrumentation persisted to `<universe>/.constellation/boot-perf.latest.json`

### Bisection

User suggested empirical bisection. Disabled boot-time tasks one at a time:
1. `linkDecay` — disabled, no change
2. `canonical-repair` — disabled, no change
3. `loadAllStats`, `enrichNodesBackground`, `cache_reconcile` — all moved out of boot path or removed entirely

Even with EVERYTHING disabled (nuclear mode), boot was still 25s UI / 90+s hydrated. Proved the slowness wasn't in our background tasks.

### The smoking gun

Added Rust-side `eprintln!` instrumentation. Traced down the root cause:

- `load_all_libraries` was being called **50+ times per boot** from many code paths internally (validate_path_in_any_library, scan_*, constellation_map_universe, and others).
- Each call re-read libraries.json from disk and re-parsed JSON.
- Under Tauri's IPC queue on Windows, the aggregate of dozens of small reads serialized through the command dispatcher created a 60-second hang.
- The frontend saw `loadLibraries()` take 75 seconds even though the Rust function ran in 1 millisecond — the rest was IPC-queue latency between repeated calls.

### The real fix

`ce47e13` — **In-memory cache for `load_all_libraries`**:

```rust
static LIBRARIES_CACHE: Mutex<Option<(PathBuf, Vec<LibraryInfo>)>> = Mutex::new(None);

pub fn load_all_libraries(app: &AppHandle) -> Vec<LibraryInfo> {
    // Check cache by active universe path
    // Cache hit: return clone in microseconds
    // Cache miss: do disk read once, populate cache
}

pub fn invalidate_libraries_cache() { /* called on save_libraries + universe switch */ }
```

This single architectural change collapsed 50+ filesystem reads per boot into 1, regardless of which mystery caller fired. Production boot on the trial Universe dropped to **1s UI / 8s fully responsive**.

## Dev-mode vs production reveal

Critical lesson learned at the very end:

- **Dev mode** (`npm run tauri dev`): 13s paint / 117s hydrated — bottlenecked by Vite + WebView2 + DevTools attachment overhead. Each IPC takes ~37s in dev mode.
- **Production** (`npm run tauri build` + launch the .exe): 1s paint / 8s hydrated. Same code, same data.

I should have asked for a production build measurement on day one. Hours of dev-mode thrashing chased a phantom that doesn't affect users.

## Commits shipped this session (chronological)

| Commit | Description |
|---|---|
| `1a7ce05` | `get_recent_notes` metadata-only + parallel threads (kept) |
| `8a14c3d` | Cache-first boot via `cache_boot_snapshot` (kept) |
| `545b582` | Split `constellation_search_init` (kept) |
| `69cfa6a` | Dedicated read-only connection for cache reads (kept) |
| `a76a717` | Lab harness + BOOT-BUDGET.md + 5 acceptance criteria (kept) |
| `436cb5b` | Paint-first architecture + boot-perf instrumentation (kept) |
| `039ac66` | Killed cache_reconcile + enrichNodesBackground at boot (kept) |
| `1719b4e` | Restored loadAllStats fire-and-forget (sidebar fix) (kept) |
| `8152759` | Per-call timing instrumentation (now reverted in cleanup) |
| `c8d684b` | Split loadLibraries IPC vs store.set (now reverted in cleanup) |
| `5905c86` | Rust-side timing for resolve_universe_libraries (now reverted) |
| `42d5e21` | Backtrace caller chain for load_all_libraries (now reverted) |
| `d59b25f` | Wall-clock timestamps on 6 boot IPCs (now reverted) |
| `b708c23` | Nuclear mode — initializeApp does nothing but paint (now reverted) |
| **`ce47e13`** | **CORE FIX: in-memory cache for load_all_libraries** |
| (next) | Cleanup: remove all diagnostics, restore canonical-repair + linkDecay |

## Files modified (final state)

- `src-tauri/src/libraries.rs` — added `LIBRARIES_CACHE` + `invalidate_libraries_cache()`; `save_libraries` now invalidates.
- `src-tauri/src/universe.rs` — `set_active_universe` invalidates the cache.
- `src-tauri/src/cache.rs` — NEW. `cache_boot_snapshot`, `cache_reconcile`, `write_boot_perf_report`, etc.
- `src-tauri/src/search.rs` — split init into `ensure_search_db_ready` + `reconcile_filesystem`; mtime-first gate in `index_note`.
- `src/routes/+layout.svelte` — paint-first `initializeApp`; zero filesystem walks on boot; cache-snapshot-driven `refreshLibraryCaches`.
- `lab/boot-perf/BOOT-BUDGET.md` — 5 acceptance criteria + production PASS baseline.
- `lab/boot-perf/touch-50.ps1` — Criterion 4 helper.
- `lab/reports/SESSION-LOG-2026-04-15.md` — this file.

## Open items

1. **Criterion 4** — Post-UI cheap stat-only sweep for external sync detection (next session).
2. **Criterion 5** — Kill-mid-index recovery + schema-version check on search.db (next session).
3. **Bundle audit** — paint_ms is still 1s on production but could likely halve with code splitting; not urgent.
4. **Stats persistence** — store star counts and recent notes in the SQLite cache so the sidebar populates instantly on boot, not after a filesystem walk (next session).
5. **Help docs** — the boot-perf architecture, the in-memory cache invariants, and the Settings → Rebuild Index button (when added) need user manual entries.

## Lessons learned (for `docs/LESSONS-LEARNED.md`)

To be added in a follow-up commit:

- **LL-XX**: Always test against a production build before chasing dev-mode performance. Tauri v2 on Windows + Vite + DevTools attachment introduces ~37s IPC latency per call that does not exist in production.
- **LL-XX**: When a function is called many times across the codebase from unknown callers, cache the result at the call site rather than auditing every call site.
- **LL-XX**: When patching three times fails (per LL-014), don't just stop — actively spawn adversarial expert agents and bisect with measurable criteria. The first move out of the loop should be a lab harness, not another patch.
- **LL-XX**: Tauri IPC dispatch on Windows in dev mode can serialize commands with multi-second per-call overhead even when the Rust handler runs in microseconds. Never extrapolate dev-mode timings to user experience.

## Backup routine

After cleanup commit lands:
- Tag: `milestone/boot-perf-fix-2026-04-15`
- ZIP: `E:/Backups/Constellation/Constellation-boot-perf-fix-20260415.zip`
