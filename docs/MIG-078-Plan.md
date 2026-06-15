# MIG-078 — Plan (Phase 2): Persist the Universe Map / OrgChart Tree + de-bloat note_meta

> `/migration` Phase 2 · 2026-06-15 · Architect doc: [MIG-078-Architect-OrgChart-Map-Tree.md](MIG-078-Architect-OrgChart-Map-Tree.md)
>
> **Boss decisions (2026-06-15):** Scope = **A′ → B → D (everything)**. `body_text` bloat = **folded into MIG-078**.
>
> Each `§` is one landable commit with a verification clause. User-testable steps pause for a Boss test (Testing Instructions Rule). Plan approval = build approval: once approved, the cascade runs autonomously, stopping only at user-testable gates and genuine architectural surprise.

---

## Sub-decisions settled with recommended defaults (override any before approval)

- **D-1 Federation strategy** → **(b) per-cUniverse, read-federated.** Each cUniverse maintains its *own* `tree_node`/`folder_stats` in its *own* `search.db` (the same write hooks run whenever that universe is the active one). When federated as a child, the active read reads the child's tables directly; if a child's tables are absent/stale (never opened under the new code), the read falls back to assembling that child's subtree in memory from its `note_meta` (the A′ method) and a background back-fill populates them. Mirrors the existing `cache_boot_snapshot_graph` federation-by-concatenation pattern. Most File-Over-App-faithful; no cross-DB writes.
- **D-2 Toggle lifetime** → default OFF during build, flip ON when a step's flag-ON output is proven byte-identical to flag-OFF, delete the old path one validated release later (§B7/§B8).
- **D-3 Reconcile cadence** → cheap delta-vs-disk check on boot **after first paint**, plus on detected external change (the existing file-watch / refreshKey path). No timer.
- **D-4 Step-0 form** → **subsumed by A′.** A′ removes the disk walk entirely, so the incidental FS-cache-warm the regression deleted becomes irrelevant. The (correct) fullscreen `onMount` gate stays; the good uncommitted Rust bits (child-universe DB load, step3 early-exit) are folded into A′'s commit rather than blanket-reverted.
- **D-5 body_text target** → new `note_body(path PRIMARY KEY, body_text TEXT)` table; FTS5 external-content triggers + `index_note` write + `read_notes` repoint to it; `note_meta.body_text` dropped + `VACUUM` only after the repoint is proven.

---

## PHASE A′ — Kill the defect now (lightweight in-memory tree)

### §A′.1 — Assemble the map tree from `note_meta`, delete the disk walk
- **Files:** `src-tauri/src/map.rs`.
- **Do:** Rewrite `build_library_node` + `build_tree` so the tree is assembled purely from the `db_records` already loaded from `note_meta` (active) + each cUniverse's `note_meta` (already wired via `load_note_records_for_child_universe`). The folder hierarchy is derived from the note **paths** (materialized-path); subtree counts roll up in one CPU pass. **Remove `fs::read_dir`, `fs::read_to_string`, `fs::metadata`, and the `collect_notes_recursive` fallback from this path.** Keep the old disk-walk implementation behind a hidden `map_tree_engine` flag (`"diskwalk"` | `"in_memory"`, default `"in_memory"`) for one rollback cycle. Verify the covering index `idx_note_meta_map` (commit `9e0eec76`) is actually chosen by `EXPLAIN QUERY PLAN`; if not, adjust so the `note_meta` read is index-only (no `body_text` page reads).
- **Edge case to decide in-build:** folders with **zero** indexed notes won't appear (no path entry). Confirm against the old output; if empty folders must show, add them from a cheap per-library `read_dir` of *directories only* (no file reads) — decide during build and log the choice.
- **Verify (engineer):** flag-`in_memory` vs flag-`diskwalk` produce identical `MapNode` trees (structure + every count) on the 7,600-note universe; `cargo build --release` clean; no `fs::read_*` on the in-memory path (grep).
- **Verify (Boss test — GATE):** fullscreen OrgChart + the sunburst Map open in **under ~1 second**, cold, on your universe; counts and folders look right.

---

## PHASE BL — De-bloat `note_meta` (move `body_text` out)

### §BL.1 — Add `note_body` table + resumable back-fill
- **Files:** `src-tauri/src/search.rs` (`init_db`, new back-fill).
- **Do:** `CREATE TABLE IF NOT EXISTS note_body(path TEXT PRIMARY KEY, body_text TEXT)`. Background, post-first-paint, resumable back-fill copying `note_meta.body_text` → `note_body`, with a cursor and status-bar progress. No reads/writes repointed yet.
- **Verify:** after back-fill, `note_body` row count == `note_meta` row count; bodies match for a sample; boot time unchanged (back-fill is off the critical path); kill mid-fill → resumes and converges.

### §BL.2 — Repoint FTS5 + writes + reads to `note_body`  *(highest-risk step — search-corruption surface)*
- **Files:** `src-tauri/src/search.rs` (`index_note` UPSERT, `note_meta_ai/ad/au` triggers, `read_notes`), `src-tauri/src/cache.rs`.
- **Do:** Convert `notes_fts` to read body from `note_body` (rewrite the `note_meta_ai/ad/au` triggers to fire on `note_body` writes instead of `note_meta.body_text`, or move the FTS sync to the `note_body` write site). Make `index_note` write the body to `note_body` (not `note_meta.body_text`). Repoint `read_notes` (`cache.rs:893`) and any `SELECT body_text` to `note_body`. Stop writing `note_meta.body_text`.
- **Verify (engineer):** full-text search returns identical results before/after on a fixed query set; Index panel (`notes_vocab`) unaffected; `cargo test` (Rust suite) green. **Reproduce-First:** because static checks have shipped editor/index regressions before (LL/BUG-023), exercise on the running app, not just `cargo test`.
- **Verify (Boss test — GATE):** search for several known terms; results identical to before; nothing missing.

### §BL.3 — Drop `note_meta.body_text` + `VACUUM`
- **Do:** Once §BL.2 is validated, `ALTER TABLE note_meta DROP COLUMN body_text` (or table-rebuild if needed for SQLite version), then `VACUUM` (background, post-paint, with progress — the DB shrinks from ~1.7 GB).
- **Verify:** DB file size drops substantially; boot "0 notes" window shrinks; search + map + Index all still correct.

---

## PHASE B — Persisted, write-time-maintained tree (the durable end-state)

### §B1 — Schema: `tree_node` + `folder_stats` (+ fix the thundering-herd race)
- **Files:** `src-tauri/src/search.rs` (`init_db`, `ensure_search_db_ready`).
- **Do:** Create the two tables + indexes from the Architect doc §4-B (idempotent, alongside the FTS5 DDL). Fix the `ensure_search_db_ready` race (`search.rs:6619-6646`) so `init_db` runs once under the lock (or via `std::sync::Once`) — adjacent boot risk, cheap to close here.
- **Verify:** tables exist on fresh + existing DB; boot time unchanged; concurrent boot callers no longer double-run `init_db` (add a one-shot guard + log).

### §B2 — Write hooks: create / edit
- **Files:** `src-tauri/src/search.rs` (`index_note`, `:4079`).
- **Do:** On the existing `note_meta` UPSERT, also UPSERT the note's `tree_node` row (and any missing ancestor folder rows) and apply the count/word/link **delta** up the ancestor chain in `folder_stats`. O(depth), never O(universe).
- **Verify:** create/edit a note → `tree_node`/`folder_stats` match the live in-memory build (§A′) exactly; type-burst latency in NotePane + FocusPane unchanged (Rule 7).

### §B3 — Write hooks: rename / move / delete + folder ops
- **Files:** `src-tauri/src/libraries.rs` (`rename_item` `:1004`, folder create/delete ops), `src-tauri/src/search.rs`.
- **Do:** On path change, `UPDATE tree_node.parent_path/path` (single-row move) + delta-shift `folder_stats` between old- and new-ancestor chains. Delete uses the `__ivm_count__` multiplicity discipline so a folder's row vanishes only when its last contributing note is gone (no zombie counts). Add explicit hooks for folder create/delete (the one event not on a `note_meta`/`note_links` write path).
- **Verify:** BUG-023-shape **linked-probe rename pair** (A links B, rename B, both files' identities + the tree intact); move a folder (counts follow); delete a folder's last note → its `folder_stats` row disappears.

### §B4 — Resumable back-fill for `tree_node`/`folder_stats`
- **Do:** Background, post-first-paint, cursor-checkpointed back-fill (200–500 notes/batch), status-bar progress, resume-on-interrupt.
- **Verify:** kill mid-fill → resumes → final tables == a full recompute; boot critical path untouched.

### §B5 — Reconcile / self-heal pass
- **Do:** Post-first-paint delta-vs-disk integrity check that repairs drift from out-of-app edits (Git/Syncthing); + on detected external change.
- **Verify:** edit/add/delete a `.md` outside the app → reconcile corrects the tree on next boot.

### §B6 — Federation (cUniverse) read path
- **Do:** Implement D-1: read each cUniverse's own `tree_node`/`folder_stats`; in-memory `note_meta` fallback + background back-fill when absent.
- **Verify:** a universe with ≥1 cUniverse shows all federated libraries with correct counts, whether or not the child has the new tables.

### §B7 — Read-path swap: `constellation_map_universe` reads the tables
- **Do:** Reimplement `constellation_map_universe` to read `tree_node` + `folder_stats` (recursive CTE), behind the `map_tree_engine` flag (`"persisted"`). Default OFF until proven.
- **Verify (engineer):** `"persisted"` vs `"in_memory"` produce byte-identical `MapNode` trees on the 7,600-note universe; open **< 100 ms** with `"persisted"`. Measure before/after (Rule 8 hard constraint). Flip default to `"persisted"`.
- **Verify (Boss test — GATE):** OrgChart + Map open effectively instantly; everything correct.

### §B8 — Consumer cutover + remove the disk walk
- **Files:** the 6 `read_library_tree` consumers (`examine:consumers` matrix), `map.rs`.
- **Do:** Serve the structure-only consumers from the `tree_node` skeleton (a thin command or shared path). After a validation period, delete the old disk-walk implementation + the `map_tree_engine` flag.
- **Verify:** all 9 consumers render correctly; Editor-Surface Gate checklist + full Boss test.

---

## PHASE D — Lazy + virtualized rendering (follow-on within MIG-078)

### §D1 — Lazy children from `parent_path` lookups
- **Do:** OrgChart sidebar + fullscreen render the root level, then load each folder's children from a `parent_path` DB query on expand (no eager `max_depth: 20`). Expansion state persists.
- **Verify:** expanding a deep folder is instant; nothing renders that isn't visible/expanded.

### §D1b — Stable-skeleton expansion (sibling-drift = 0)  *(added 2026-06-15, Boss-approved)*
- **Why:** the OrgChart full-screen `toggleFsExpand` (`OrgChart.svelte:463`) currently re-runs `fitToWidth()` + re-centers the WHOLE chart on every expand — the "Dagre full re-layout" anti-pattern (ref: graph-playground.aisloppy.com layout-stability lab). Expanding a folder makes the chart rescale/jump.
- **Do:** adopt **stable skeleton + in-slot expansion** (the lab's "Custom lanes" / the field's *constrained incremental layout*): reserve a lane for each node's subtree and grow its children **inside that reserved slot**, WITHOUT a global `fitToWidth`/re-center. Anchor pan/zoom on the expanded node only. (Industry equivalents: Cytoscape `fcose` incremental, ELK incremental, d3-force `.fx/.fy` pinning.)
- **Verify (acceptance criterion — hard):** **expanding a folder MUST NOT move its siblings** — measure max sibling displacement (a tiny internal "drift meter" dev assertion, the lab's metric); target **0 px**. Expanding deep in a large library feels solid, no whole-chart jump.
- **Out of scope for MIG-078 (logged follow-ons):** the same principle for **GraphMind** (pin on-screen nodes, lay out only new neighbors) and **Sky View** (pin settled bubbles via `.fx/.fy` once Write-Time Derivation makes it incremental, instead of "reheat"). Constellation Map (sunburst) is NOT a fit (radial re-partition is its nature — Form-Aligns-To-Purpose).

### §D2 — Virtualize tree rows
- **Do:** Virtualize the tree row list (Rule 3: "virtualize every list that can exceed 50 items") so a folder with thousands of children renders only the viewport.
- **Verify:** a 5,000-child folder scrolls smoothly; no frame stutter.

---

## Cross-cutting verification (every phase)
- **Editor-Surface Gate checklist** (NotePane + FocusPane round-trips, tab switch in/out incl. while in Focus, PropertyEditor, rename-with-linked-probe, second screen, restart) on any step touching the write path (§BL.2, §B2, §B3).
- **Boot-perf criterion:** measure boot + open before/after on the 7,600-note universe; no regression (Rule 8 hard constraint).
- **SO logging:** each `§` commit logged to `lab/reports/SESSION-LOG-2026-06-15.md`; orientation + help/User-Manual updated in the same commit when a Boss-visible behavior or a §6 trigger changes.
- **Phase 4 (Audit):** after Build, three parallel agents check invariants / drift / migration-path (first-boot, schema mismatch, mid-backfill interrupt, rollback).

## Risk ranking (where to be most careful)
1. **§BL.2** — repointing FTS5/body to `note_body`: search-corruption surface. Reproduce-first on the running app.
2. **§B3** — rename/move/delete aggregate maintenance: the classic IVM drift trap + the BUG-023 content-integrity class. `__ivm_count__` + reconcile safety net + linked-probe test.
3. **§B7** — read swap: must be byte-identical; gated behind the flag with an equality harness.
4. **§A′.1 empty-folder edge case** — decide and log during build.
