# Investigation — `constellation_boot_bundle` 13s cold-start

**Date:** 2026-04-16
**Baseline report:** `paint_ms=409  libraries_loaded_ms=13167  hydrated_ms=21117  graph_ready_ms=39592` (`criterion_2_hydrated=FAIL`)

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
