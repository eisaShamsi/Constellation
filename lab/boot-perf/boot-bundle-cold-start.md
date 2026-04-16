# Investigation — cold-start bottleneck (updated 2026-04-16)

**Date:** 2026-04-16
**Baseline report (pre-fix):** `paint_ms=409  libraries_loaded_ms=13167  hydrated_ms=21117  graph_ready_ms=39592` (`criterion_2_hydrated=FAIL`)
**Second measurement (post-boot_bundle instrumentation):** `paint_ms=483  libraries_loaded_ms=550  hydrated_ms=28214  graph_ready_ms=46354` with `boot_bundle_timings` all ≤ 31ms.
**Third measurement (warm relaunch ~3 min later):** `paint_ms=611  libraries_loaded_ms=686  hydrated_ms=26605  graph_ready_ms=45477`.
**Fourth measurement (deep attribution):** `hydrated_ms=28425  cache_snapshot_core_wall_ms=27710  server_timings=[ensure_db:73, open_reader:0, read_notes:8021]  load_all_stats_wall=27671  start_watching_all_wall=27676  load_all_appearances_wall=27673` — the smoking gun.

## The smoking gun (round 4)

All 34 boot IPCs finished at essentially the same wall-clock endpoint (~27.7 s). The ship-gate IPC `cache_boot_snapshot_core` executed in 8094 ms of Rust time but waited 27710 ms wall — a **19,616 ms pure queue/contention delta**. Every fire-and-forget IPC converged on the same endpoint, which is textbook shared-resource serialization: Windows' NTFS I/O scheduler round-robins concurrent reads, so every simultaneous request gets roughly `1/N` of the disk bandwidth and every one takes `N×` as long to complete.

This chose the fix path definitively:
- **Primary cause:** Tauri IPC queue contention at the OS I/O layer.
- **Secondary cause:** 8 s `read_notes` on a 7,595-row SELECT of three narrow TEXT columns — a row-store full-scan reading wide rows (`body_text` + JSON blobs) just to project three columns.

## The fix (two changes, both shipped 2026-04-16)

### Fix 1 — reorder the boot path
Move `refreshLibraryCaches()` from fire-and-forget to awaited, before the watcher/appearance/stats fan-out. The core snapshot gets exclusive I/O until `boot:hydrated` fires; everything else runs after.

```typescript
// BEFORE: 4 things race, everyone loses
Promise.all(watchers).catch(() => {});
forEach(appearances).catch(() => {});
loadAllStats().catch(() => {});
refreshLibraryCaches().catch(() => {});

// AFTER: hydrate first, fan-out after
await refreshLibraryCaches().catch(() => {});
Promise.all(watchers).catch(() => {});
forEach(appearances).catch(() => {});
loadAllStats().catch(() => {});
```

### Fix 2 — covering index for the boot-snapshot projection
Add to `init_db()` in `src-tauri/src/search.rs`:

```sql
CREATE INDEX IF NOT EXISTS idx_note_boot_snapshot
    ON note_meta(name, path, library_name);
```

SQLite planner switches from full table scan (wide rows → ~80 MB of page reads) to index-only scan (three narrow TEXT columns → ~200 KB of index pages). `IF NOT EXISTS` + no schema version bump means existing DBs pick it up on next launch with no reindex.

## The warm-run paradox

A warm relaunch 3 minutes later — with the OS page cache essentially still hot — produced nearly identical timings (`hydrated_ms=26605` vs cold 28214, `graph_ready_ms=45477` vs cold 46354).

**This destroys the "cold NTFS page cache" explanation as the sole cause.** Something is taking ~26 s regardless of whether the disk has been pre-read. That cannot be a cold-cache cost; it has to be either:

1. **Rust-side CPU/SQLite work that is O(notes) even with cached pages** (e.g., row-decode overhead on a wide row-store), OR
2. **Tauri IPC queue contention** — `cache_boot_snapshot_core` is awaited by the frontend, but 33 fire-and-forget IPCs are issued in the same tick (16× `watch_library`, 16× `read_library_appearance`, 1× `get_all_library_stats`) and may be starving the blocking thread pool, OR
3. **Lock contention on `SearchState.db` mutex** — `ensure_search_db_ready` briefly grabs it; something else in the tree may hold it longer.

## What `constellation_boot_bundle` actually costs

Instrumentation proved it runs in ~31 ms total (`load_all_libraries=0, read_universe_settings=31, rest=0`). It is not where the cost lives. The 13,167 ms attributed to it on the prior day was either cumulative boot work on a much colder universe or measurement noise across a different run.

## The 26-second gap — what it covers

Between `boot:libraries-loaded` (line 1550 of `+layout.svelte`) and `boot:hydrated` (marked inside `refreshLibraryCaches` after `cache_boot_snapshot_core` resolves, line 1989), only four things run:

1. `Promise.all($libraries.map(lib => startWatchingLibrary(lib.id, lib.path)))` — 16 parallel `watch_library` IPCs (fire-and-forget).
2. `$libraries.forEach(lib => loadLibraryAppearance(lib.path, lib.id))` — 16 parallel `read_library_appearance` IPCs (fire-and-forget).
3. `loadAllStats()` — one `get_all_library_stats` IPC that spawns 16 threads to walk each library's filesystem twice (count + recent) + 160 file-content reads for previews (fire-and-forget).
4. `refreshLibraryCaches()` → `await invoke('cache_boot_snapshot_core')` — this is what `boot:hydrated` waits for.

All four race into Tauri's command queue at the same tick. If `get_all_library_stats` saturates the blocking thread pool with 16 parallel filesystem walks of 7,595 files, `cache_boot_snapshot_core` may sit in the queue for most of that time.

## Candidate fixes (ranked, cannot yet choose — measuring first)

1. **If per-phase Rust timings show `read_notes` ~25 s:** SQLite row-store row-decode is the bottleneck. Fix: covering index `idx_note_boot_snapshot(name, path, library_name)` so the planner does an index-only scan — three narrow TEXT columns instead of full-page reads of wide rows.

2. **If per-phase Rust timings sum to a small number but `cache_snapshot_core_wall_ms` is ~26 s:** IPC queue contention. Fix: reorder the fire-and-forget chain so `refreshLibraryCaches()` fires FIRST, then the stats/appearances/watchers follow (they don't gate anything). Or move `get_all_library_stats` onto a background thread with its own emit.

3. **If `get_all_library_stats` alone is ~26 s:** it's the filesystem walk. Fix: kill it from the boot path entirely (read star counts from the SQLite index, which already has the notes) and compute fresh counts lazily when the library switcher opens.

## Current instrumentation (3rd build)

Per-phase Rust timings inside `cache_boot_snapshot_core`:
- `ensure_db` — how long to grab the state mutex and ensure schema.
- `open_reader` — how long to open the dedicated read-only SQLite connection.
- `read_notes` — the actual `SELECT name, path, library_name FROM note_meta` scan.

Per-phase Rust timings inside `cache_boot_snapshot_graph`:
- `ensure_db`, `open_reader`, `count_notes`, `read_links`, `read_tags`.

Frontend wall-clock fields (ms) in `boot-perf.latest.json`:
- `cache_snapshot_core_wall_ms`, `cache_snapshot_core_server_timings`
- `cache_snapshot_graph_wall_ms`, `cache_snapshot_graph_server_timings`
- `load_all_stats_wall_ms`, `start_watching_all_wall_ms`, `load_all_appearances_wall_ms`

If `cache_snapshot_core_wall_ms >> sum(cache_snapshot_core_server_timings)`, the delta is queue/contention time, not Rust execution. That alone chooses between fix 1 and fix 2.

## Superseded early hypothesis (kept for history)

### Cold-disk row-scan theory (partially valid, insufficient alone)

## Why a small-looking query takes 27 seconds cold

`note_meta` schema (from `src-tauri/src/search.rs:107-118`):

```sql
CREATE TABLE note_meta (
    path TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    library_name TEXT NOT NULL,
    modified INTEGER NOT NULL,
    content_hash TEXT,
    properties_json TEXT DEFAULT '{}',
    tags_json TEXT DEFAULT '[]',
    outgoing_links_json TEXT DEFAULT '[]',
    headings_json TEXT DEFAULT '[]',
    body_text TEXT DEFAULT ''      -- ← full indexed note content
);
```

SQLite is a **row store**. `SELECT name, path, library_name FROM note_meta` still has to read every row's page to find those columns, which means every page is pulled off cold disk — including the `body_text`, `outgoing_links_json`, and other heavy columns we never use. For 7,595 notes this is a 100–300 MB scan of scattered pages.

The cold I/O pattern kills us: SQLite reads pages in B-tree order, which is nearly random w.r.t. on-disk layout after months of insertions and updates. That's 100k+ cold 4 KB reads at whatever the drive's random-read latency is.

## Proposed fix — covering index on `note_meta`

Add a **covering index** containing just the three projected columns:

```sql
CREATE INDEX IF NOT EXISTS idx_note_boot_snapshot
    ON note_meta(name, path, library_name);
```

SQLite's planner will pick this index for `SELECT name, path, library_name FROM note_meta` because an index-only scan is cheaper than a table scan when the index covers all projected columns.

**Expected sizes:**
- Main table cold scan: **~200 MB** (includes body_text + all JSON columns)
- Covering index cold scan: **~1.5 MB** (three small text columns × 7,595 rows)

**Expected cold-read time reduction:** 100–150×. Should take `cache_boot_snapshot_core` from 27.7 s to sub-second even on a cold NTFS page cache.

Cost: ~1.5 MB extra on-disk storage. No migration risk — `CREATE INDEX IF NOT EXISTS` is idempotent; existing indexes/tables are untouched.

## The graph phase (18 s) — out of scope for this fix

`cache_boot_snapshot_graph` has three queries:
1. `SELECT COUNT(*) FROM note_meta` — trivial
2. `SELECT source_path, source_name, target_name, link_type, library_name FROM note_links WHERE status='active'` — 656k rows, different table
3. `SELECT tags_json FROM note_meta` — still needs a full scan of `note_meta` because `tags_json` is not in any index

Graph phase is deferred via `requestIdleCallback` (not on the ship-gate path for Criterion 2). 18 s is ugly but doesn't block the user from typing or navigating. Address after core phase is fixed.

## Next step

Ship the covering index + a one-line adjustment to `read_notes` that explicitly uses it (belt-and-suspenders — `INDEXED BY idx_note_boot_snapshot`). Rebuild. Re-measure. If the core phase drops below 6 s, Criterion 2 passes on cold boot.

## Superseded sections below (original investigation, kept for history)

## What `constellation_boot_bundle` actually does

Reading `src-tauri/src/boot_bundle.rs` against `src-tauri/src/universe.rs` and `src-tauri/src/bases.rs`, the command performs **~8 sequential file-I/O operations** on the main thread. All synchronous `fs::read_to_string` / `fs::canonicalize`.

On a Universe with **N child universes**, the actual work is:

| Step | Call | I/O on cold disk |
|------|------|------------------|
| 1 | `load_all_libraries` → `resolve_libraries_recursive` | parent `libraries.json` + parent `universe.json` + for each child: `fs::canonicalize(path)` + child `libraries.json` + child `universe.json` |
| 2 | `read_universe_settings` | `settings.json` |
| 3 | `read_universe_bookmarks` | `bookmarks.json` |
| 4 | `read_universe_workspaces` | `workspaces.json` |
| 5 | `read_universe_property_types` | `property-types.json` |
| 6 | `list_workspace_bases` | `bases/` dir enum + read each `.base` (name only) |
| 7 | `get_child_universes` | parent `universe.json` **(REPEAT of step 1)** + for each child: child `universe.json` **(REPEAT)** |
| 8 | **loop over children** → `read_child_universe_libraries` | each child's `libraries.json` **(REPEAT of step 1)** |

## Root-cause candidates (ranked)

1. **Triple-read of child-universe metadata.** Steps 1, 7, 8 each re-open the same `universe.json` / `libraries.json` files per child. With N children, that's ~3N redundant file-opens. On cold NTFS this is the single biggest cost.
2. **`fs::canonicalize` on each child path.** Windows canonicalize stats every path component and resolves symlinks. Cold cost is 100–500 ms per deep child path. With N=5 children, that's 0.5–2.5 s just from canonicalize.
3. **Serial execution.** All 8 steps run on the main thread, one after another, despite all top-level reads being independent (settings/bookmarks/workspaces/property_types/bases/child_universes/libraries never touch each other).
4. **Per-base name read in `list_workspace_bases`.** Each `.base` definition is opened a second time for its display name.

## Two clean wins, stackable

### Win 1 — De-duplicate child reads (highest impact, no new threads)

`constellation_boot_bundle` should call `resolve_libraries_recursive` **once**, capture the enumerated children with their names/paths/library lists, and pass that structure into the same function body where `get_child_universes` + the child-libraries loop currently stand. Removes 2N redundant file-opens + N redundant canonicalize calls.

Risk: `resolve_libraries_recursive` currently throws away the per-child grouping — it flattens everything into one `Vec<LibraryInfo>`. Either refactor it to also return a `Vec<ChildGroup { path, name, libs }>`, or write a new helper that walks once and returns both shapes. Both are localized changes.

### Win 2 — Parallelize the independent top-level reads

After Win 1, the remaining reads (`settings`, `bookmarks`, `workspaces`, `property_types`, `workspace_bases`, `load_all_libraries`) are fully independent. Wrap in `std::thread::scope` spawning 5–6 threads. On cold disk with Windows' I/O queue, this gives roughly a 3–4× speedup for the parallel portion.

Risk: none — all the sub-functions are pure reads with no shared mutable state.

### Optional Win 3 — Drop `fs::canonicalize` for cycle detection

If we need cycle detection only (not symlink resolution), comparing normalized path strings (lowercase + forward-slash) is ~100× cheaper than canonicalize. Defer until Wins 1 & 2 data shows whether it still matters.

## Recommended next step

**Measure before optimize, per LL-014 + LL-017.**

Add per-step timing to `constellation_boot_bundle` — a `timings_ms: Vec<(&'static str, u64)>` field on `BootBundle`. Frontend writes it into `boot-perf.latest.json` alongside existing metrics. User runs one cold-start trial (reboot → launch), we get the exact breakdown, and we know which of the three wins to ship first.

Cost: ~15 lines of Rust + 3 lines of frontend + one build + one trial. Then the fix is mechanical and confident.

**Alternative (if user wants to skip the measurement step):** ship Win 1 + Win 2 together as a single commit. The code change is small enough (~50 lines of Rust) and the pattern is well-understood — parallelizing independent file reads on cold disk is not speculative. But we'd be guessing at the breakdown.

## Out of scope

- Universe-switch cold-start (rare; users boot far more often than they switch).
- `constellation_search_init` SQLite cold-read (separate concern; was already split yesterday).
- Warm-cache boot — this investigation is only about the first cold boot.
