# Architect — Persist the Universe Map / OrgChart Tree (Write-Time Derivation)

> MIG-078 · `/migration` Phase 1 (Architect) · 2026-06-15 · No code written.
>
> Produced from an 8-agent investigation: 4 code-examination agents (dataflow, write path,
> boot path, consumers) + 4 state-of-the-art research agents (PKM caching, SQLite trees,
> incremental view maintenance, editor file-trees), fused into this recommendation. All
> file:line claims are sourced from the examination reports; all external patterns carry URLs.

**Function in hand:** the **OrgChart / Constellation-Map universe tree** — the hierarchical Universe → Library → Folder → Note structure served by `constellation_map_universe` (fullscreen OrgChart + D3 sunburst) and by `read_library_tree` (sidebar / list / navigator trees). Goal: stop rebuilding it by walking 7,600+ files on every open; persist the derived tree (and its per-folder aggregates) so opening is instant, per CLAUDE.md Rule 8 (Write-Time Derivation), mirroring the FTS5-trigger exemplar.

---

## 1. CURRENT STATE — How the tree is built today, and where the time goes

There are **two** distinct IPC surfaces that produce the tree (`examine:consumers`):

- **`read_library_tree`** — a structural filesystem dump (`name`, `path`, `is_dir`, `children`). Six consumers: OrgChart sidebar mode (`OrgChart.svelte:103`), NotebookNavigator (`NotebookNavigator.svelte:111`), the Libraries page (`libraries/+page.svelte:100`), two `+layout.svelte` panels (`:4259`, `:5108`), and the shared navigator data layer (`navigator/data.svelte.ts:145`).
- **`constellation_map_universe`** — a *computed semantic view* carrying all 12 `MapNode` fields (`weight`, `note_count`, `word_count`, `link_count`, `maturity`, `stratum`, `modified`, plus structure). Three consumers: OrgChart fullscreen (`OrgChart.svelte:449`), its reload-on-edit path (`:797`), and the D3 sunburst `ConstellationMap.svelte:378`.

**The fullscreen open path** (`examine:dataflow`):

1. `OrgChart.svelte` `$effect` at line 787 fires `loadFullscreenData()` (line 443), which makes **one IPC call** to `constellation_map_universe` (`map.rs:395`), `max_depth: 20`.
2. Rust loads the alias map (`map.rs:406`, ~15 ms), then **all note records once** via `load_note_records` → `SELECT path, name, word_count, outgoing_links_json, modified, created_at FROM note_meta` (`map.rs:108-143`). Measured ~634 ms warm, ~29 s cold (the 1.7 GB DB read through AV).
3. Child-universe records are appended from each cUniverse's own `.constellation/search.db` (`load_note_records_for_child_universe`, `map.rs:90-106`).
4. Per library, `build_library_node` (`map.rs:280-362`, confirmed in source) filters `db_records` by path prefix (lines 295-299) and **falls back to `collect_notes_recursive` when the filtered set is empty** (lines 300-302, confirmed). The fallback reads **every `.md` file's full content** (`fs::read_to_string`, `map.rs:536`) plus `fs::metadata` per file (`map.rs:556`).
5. **Even on the indexed (non-fallback) path, `build_tree` (`map.rs:588-676`) calls `fs::read_dir` unconditionally on every directory** (line 606) and `is_dir()` on every entry (line 615). This is the directory-walk cost that survives even when file *content* is not read.

**Where the time goes** (`examine:dataflow` summary table):

| Path | Cost |
|---|---|
| Indexed, warm FS cache | ~1.4–1.5 s (`load_note_records` ~634 ms + `build_tree` readdir ~761 ms) |
| Indexed, cold FS cache | dominated by readdir/stat (~600 stat calls) under AV |
| **Fallback (`collect_notes_recursive`)**, cold | **40–60 s** (7,600 × `read_to_string` + `metadata`) |

**The uncommitted regression** (confirmed at `OrgChart.svelte:365-372`): `onMount` was gated to `if (!fullscreen) loadData()`. Previously `loadData()` ran on every mount and incidentally warmed the OS dentry/inode cache via `read_library_tree`. Removing it for fullscreen mode left the FS cache cold, so any path that touches disk (the unconditional `build_tree` readdir, and especially the `collect_notes_recursive` fallback for cUniverse libs whose paths don't prefix-match `db_records`) pays full AV-scanned I/O — turning ~26 s into 2+ minutes (`examine:dataflow`, "Why the regression happened").

Note also that even the *cheap* `note_meta` read is taxed by table bloat: `body_text` is stored inline in `note_meta` (`examine:bootpath` §4), so the `SELECT path, name, word_count…` in `load_note_records` pays row-store full-page reads against a ~1.7 GB table even though it never selects `body_text`. (A covering index `idx_note_meta_map` was committed in `9e0eec76` to make this read index-only; whether the planner uses it must be verified empirically.)

---

## 2. ROOT CAUSE

The tree is **recomputed at read time from an eager whole-Universe disk walk**, instead of being read from an already-current persisted structure. Every open of the OrgChart/Map invokes `build_tree`, which `fs::read_dir`s every directory in the Universe (and, on the federation/cold-index fallback, `read_to_string`s every file) before it can hand back a single node — there is no persisted tree to read, so the cost is paid in full on every open and scales with Universe size and antivirus per-file latency, not with what the user is actually looking at. This is the exact shape CLAUDE.md Rule 8 forbids ("Don't write a `scan_*` / `rebuild_*` command that re-walks the Universe to produce a derived view"). The symptom patches attempted so far — Round 7 async-command offloading, MIG-077's "prefer `db_records` over file content," and the FS-cache-warming side effect of `loadData()` — all reduce the *constant factor* of a read-time recomputation; none of them remove the read-time recomputation itself. The uncommitted regression is precisely what happens when one of those constant-factor crutches (the incidental cache warm-up) is pulled: the underlying O(Universe) read-time walk is exposed again.

---

## 3. INVARIANTS THAT MUST NOT BREAK

1. **Aggregate correctness.** Per-folder/per-library/universe-root `note_count`, `word_count`, `link_count`, plus `weight`, `maturity`, `stratum` must equal what the current `build_tree`/`build_library_node` computes today (`map.rs:340-356`). The sidebar badge (`OrgChart.svelte:1189`) and the sunburst center label depend on exact counts.
2. **Child-universe federation.** The tree must still include libraries federated from cUniverses, whose notes live in *separate* `.constellation/search.db` files (`examine:bootpath` §3; `map.rs:90-106`). A persisted tree in the active DB cannot silently drop them.
3. **Rename / move / delete reflected.** Structural mutations must appear in the tree without a full rebuild. The single production write-site for path changes is `rename_item` in `libraries.rs:1004-1024` (`examine:writepath` §4/§5); create/edit flow through `index_note` (`search.rs:3945`, UPSERT at `:4079-4096`).
4. **Both consumers served.** The structure-only consumers (`read_library_tree`, 6 call sites) and the full-metadata consumers (`constellation_map_universe`, 3 call sites) must both keep working. `examine:consumers` concludes they cannot collapse into one IPC shape without major change — the persisted source can be shared, but the two response shapes stay.
5. **No boot-time regression.** Boot is already fragile (`examine:bootpath`: ~18 s cold, a thundering-herd race in `ensure_search_db_ready` at `search.rs:6619-6646`, `body_text` bloat). The change must not add work to the boot critical path; the first back-fill must run *after* first paint (CLAUDE.md Rule 8 "first-time population").
6. **No typing-latency regression.** Write-path hooks (triggers / `index_note` additions) must be O(delta), never O(Universe). Zero `invoke()` on the keystroke hot path; saves stay debounced ≥1500 ms.
7. **File-Over-App.** `.md` files on disk remain the source of truth. The persisted tree is an ephemeral derived index — rebuildable from disk, never authoritative, never silently mutating file content. Must self-heal when files change outside the app (Git/Syncthing/iCloud).
8. **Resumable first back-fill.** Populating the tree for an existing 7,600-note Universe must be checkpointed and resume-from-cursor on interrupt (`research:ivm` §5, GitLab batched-migration pattern), with status-bar progress.

---

## 4. DESIGN OPTIONS

### Option A — Lazy `readdir`-on-expand + cheap persisted aggregates

**How it works.** Stop computing the whole tree on open. The tree panel renders only the root level, then calls a per-folder `loadChildren(dir)` (one `readdir` + stat) when a folder is expanded; results cache in an in-memory model keyed by path; expansion state persists. Rows are virtualized so a wide folder renders only the viewport. Per-folder counts come from a cheap persisted aggregate lookup rather than a walk.

**Pattern mirrored.** VS Code's `IFileService.resolve()`-on-expand + `ExplorerItem` cache + watcher invalidation; IntelliJ's lazy-populated VFS snapshot; the lazy-tree-view consensus (`research:editor-trees`).

- **Open latency:** O(root children) — effectively instant regardless of Universe size.
- **Effort:** Medium-high on the *frontend* (the OrgChart/sidebar must become lazy + virtualized; today fullscreen requests `max_depth: 20` eagerly). Low on the Rust side for structure.
- **Risk:** Medium. The sunburst (`ConstellationMap.svelte`) fundamentally needs the *whole* weighted subtree to draw a radial partition — lazy-on-expand does not serve a full-Universe visualization well. So Option A fully solves the **tree** consumers but only partially serves the **sunburst** consumer (Invariant 4).

### Option A′ — Assemble the tree in memory from `note_meta`, zero disk walk *(lightest; SME addition)*

**How it works.** The note paths in `note_meta` already encode the full folder hierarchy (a materialized-path column). Build the entire `MapNode` tree in memory from the flat list of paths + their already-stored aggregates (`word_count`, `outgoing_links`, `modified`, `created_at`), rolling counts up the tree in a single CPU pass. **No `fs::read_dir`, no `fs::read_to_string` — the disk walk is deleted outright.** For child universes, read their `note_meta` (already implemented in `load_note_records_for_child_universe`). The covering index `idx_note_meta_map` (committed `9e0eec76`) makes the `note_meta` read index-only.

**Pattern mirrored.** Querying the index and shaping the tree client-/in-memory-side is the common PKM read pattern; the path-as-materialized-path is `research:sqlite-trees` Part 2.

- **Open latency:** O(query + in-memory build) — index read (tens of ms with the covering index) + CPU tree-build over 7,600 rows (tens of ms). Sub-second, cold or warm.
- **Effort:** **Low.** A single-file rewrite of `build_library_node`/`build_tree` in `map.rs`; no new schema, no triggers, no back-fill, no boot changes.
- **Risk:** Low. Self-contained to one Rust file behind a toggle. Caveat: folders containing **zero** indexed notes won't appear (the path set has no entry for an empty folder). Must confirm empty folders are not required in the Map view (they carry no knowledge; likely acceptable — to be verified).
- **Rule 8 standing:** The *source* (`note_meta`) is write-time-maintained; the tree *shape + rollups* are computed on read, but in trivial CPU. This is "Rule 8-adjacent" — reads what's stored, recomputes only cheap aggregates. It removes the root cause without persisting the assembled tree.

### Option B — Persisted normalized tree (adjacency list) + per-folder aggregate rollups, maintained by triggers / write-path hooks *(recommended end-state — see §5)*

**How it works.** Add a normalized tree to the existing SQLite DB and maintain it at write time, exactly as the FTS5 `notes_fts` table is maintained by the `note_meta_ai/ad/au` triggers (`examine:writepath` §1; `search.rs:2257-2272`), and as `sky_nodes` is maintained by `note_meta_sky_ai/ad/au` (`search.rs:2660+`). Two derived tables:

```sql
-- Structure: adjacency list. One row per folder AND per note node.
CREATE TABLE IF NOT EXISTS tree_node (
    path           TEXT PRIMARY KEY,      -- canonical path, same key as note_meta.path
    parent_path    TEXT,                  -- NULL for a library root; indexed
    name           TEXT NOT NULL,         -- display name (frontmatter title w/ stem fallback)
    is_dir         INTEGER NOT NULL,      -- 1 = folder/library, 0 = note
    node_type      TEXT NOT NULL,         -- 'universe' | 'library' | 'folder' | 'note'
    library_name   TEXT NOT NULL,
    -- per-NOTE leaf metrics (NULL for folders), copied from note_meta at write time:
    word_count     INTEGER,
    link_count     INTEGER,               -- outgoing_count from note_meta
    maturity       TEXT,
    stratum        INTEGER,
    modified       INTEGER
);
CREATE INDEX IF NOT EXISTS tree_node_parent ON tree_node(parent_path);

-- Rolled-up subtree aggregates, one row per folder/library/root:
CREATE TABLE IF NOT EXISTS folder_stats (
    path             TEXT PRIMARY KEY,    -- folder/library/root path
    note_count       INTEGER NOT NULL DEFAULT 0,  -- notes anywhere in subtree
    word_count       INTEGER NOT NULL DEFAULT 0,
    link_count       INTEGER NOT NULL DEFAULT 0,
    weight           REAL    NOT NULL DEFAULT 0
);
```

Tree reads become a single recursive CTE over `tree_node` (`research:sqlite-trees` Part 2), or a flat `parent_path` lookup hydrated client-side — no `fs::read_dir`, no `read_to_string`. Aggregates are O(1) lookups in `folder_stats`.

**Maintenance (write-time):** create/edit → `index_note` (`search.rs:4079`) already UPSERTs `note_meta`; add a parallel UPSERT into `tree_node` for the note row, and propagate the count/word/link delta up the ancestor chain in `folder_stats`. Rename/move → `rename_item` (`libraries.rs:1004`) already UPDATEs `note_meta.path`; add `tree_node` parent/path update (adjacency-list move is a single-row `UPDATE parent_path`, the model's signature strength — `research:sqlite-trees` Part 1) plus a delta-shift between old-ancestor and new-ancestor chains in `folder_stats`. Delete → mirror with the **`__ivm_count__` multiplicity discipline** so a folder's `folder_stats` row vanishes only when its last contributing note is gone, never leaving a zombie count (`research:ivm` §3). Folder create/delete (no note involved) flows through `libraries.rs` folder ops and must get an explicit hook there.

**Pattern mirrored.** The FTS5 external-content + trigger pattern Constellation already runs (`research:ivm` §1, https://sqlite.org/fts5.html; the in-repo exemplar at `examine:writepath` §1). Adjacency list + recursive CTE + trigger-maintained aggregates is the recommended SQLite tree pattern (`research:sqlite-trees`). Logseq/Dendron's persisted-index-as-read-source confirms boot becomes a cheap restore, not a recompute (`research:pkm-caching`).

- **Open latency:** O(visible tree) from a single DB query — sub-100 ms; no filesystem touch.
- **Effort:** High. New schema, triggers/hooks on three write paths (create/edit, rename/move, folder ops), a resumable back-fill, and a read-path swap for both IPC commands.
- **Risk:** Medium. Incremental-aggregate drift is the classic trap (`research:ivm` §3-4) — mitigated by the `__ivm_count__` counter, recompute-in-trigger for direct-child counts, and a periodic reconcile pass. Folder-only operations are the easy-to-miss hook.

### Option C — Persisted serialized-JSON tree blob *(disqualified)*

Store the whole `MapNode` tree as one JSON blob, rebuilt on save. `research:sqlite-trees` Part 4 verdict: for a 7,600-note tree that mutates on nearly every action and must be queried into, the JSON blob is the **wrong fit** — every partial change rewrites the whole document, it's unindexable, and "rebuilt incrementally" usually means re-walking the Universe, reintroducing the very O(Universe) walk we're removing. Listed for completeness; not viable.

### Option D — Hybrid: persisted normalized tree (B) + lazy/virtualized rendering (A) on top *(best, biggest)*

Persist structure + aggregates as in Option B (DB is the read source, maintained at write time). Render the sidebar/OrgChart tree lazily and virtualized as in Option A, but children come from a `parent_path` DB lookup instead of `readdir`. `readdir` is demoted to the reconcile/verification path. **Effort: highest** (B's backend + A's frontend). The true best-in-class shape (`research:editor-trees` "canonical recipe" + `research:sqlite-trees` + Rule 8).

---

## 5. RECOMMENDATION

**End-state: Option B** — persisted normalized adjacency-list tree (`tree_node`) + per-folder rollups (`folder_stats`), maintained by write-path hooks/triggers on `index_note`, `rename_item`, and the folder operations in `libraries.rs`.

**Justification against Constellation's constraints:**

- **It is the FTS5 exemplar applied to the tree.** Rule 8 names FTS5 triggers as the canonical write-time-derivation shape, and the repo already runs exactly this pattern for `notes_fts`, `sky_nodes`, stratum, maturity, and outgoing-link aggregates (`examine:writepath` §1-2). Option B is the *same proven mechanism* extended to one more surface — lowest-novelty, highest-precedent. The external research independently lands on the same recommendation (`research:sqlite-trees`; `research:ivm`).
- **Instant open on large Universes.** Reads become a single recursive CTE / flat lookup — no `fs::read_dir`, no `read_to_string`, no AV tax. Removes the O(Universe) read-time walk that is the root cause.
- **Local-first / File-Over-App preserved.** The tables are a derived, rebuildable index in the same local SQLite DB; `.md` files stay authoritative. A reconcile pass handles out-of-app edits.
- **Adjacency list is correct for a constantly-mutating tree.** Moves/renames are single-row `UPDATE parent_path` (cheapest of the four models); the research rejects nested-set (O(tree) writes) and JSON-blob for this profile.
- **Boot and typing protected.** Hooks are O(delta); the historical back-fill runs post-paint and resumable; reads move *off* the boot critical path.

**Pragmatic sequencing — the key SME note for Eisa.** Option **A′ removes the actual defect with a fraction of the effort and risk** (one Rust file, no schema, no triggers, no back-fill): it deletes the disk walk and assembles the tree from the index Constellation *already* maintains at write time. It does not persist the assembled tree, so it is not the full Rule-8 end-state — but it takes open from 2 min / 26 s to sub-second and is independently shippable. The clean path is: **ship A′ to kill the defect now → then build B as the durable Write-Time-Derivation end-state → optionally layer A's lazy+virtualized rendering to reach D.** Each step is landable and verifiable on its own.

**Runner-up (full):** Option D. Strictly the best shape (also bounds wide-folder render cost), but it bundles a frontend lazy/virtualized rework with the backend migration — larger, riskier as one migration. Better reached by composition (A′ → B → A-rendering) than attempted whole.

**Honest industry-pattern comparison.** Mature *code editors* (VS Code, IntelliJ) use **lazy readdir-on-expand + watcher invalidation + virtualized rows** (Option A) — they have no per-folder semantic rollups to serve. Mature *PKM* tools (Logseq, Dendron, Obsidian) use **a persisted index as the single read source, maintained incrementally** (Option B's shape). Constellation is a PKM, already owns the trigger machinery, and the sunburst consumer *requires* a whole-Universe weighted tree that lazy-on-expand cannot serve well — so the PKM pattern (B) is the right end-state, with A's *rendering* half as the correct follow-on.

---

## 6. ROUGH PHASE BREAKDOWN (provisional — for the eventual Plan)

Each phase lands as one commit with a verification clause.

- **§0 — Revert the uncommitted regression.** Restore the fullscreen FS-cache warm (or gate the fullscreen overlay behind `{#if}` so the walk runs only on explicit open) in `OrgChart.svelte:365-372`. *Verify:* fullscreen OrgChart opens at the pre-regression time (not 2+ min) on the 7,600-note Universe. Unblocks Eisa immediately; independent of the migration.
- **§A′ — In-memory tree from `note_meta` (if chosen as the fast win).** Rewrite `build_library_node`/`build_tree` to assemble the tree from `note_meta` paths + aggregates, behind a toggle; delete the disk walk on that path. *Verify:* fullscreen OrgChart and sunburst render identically to the disk-walk path; open < 1 s cold; counts match.
- **§1 — Schema.** Add `tree_node` + `folder_stats` tables/indexes in `init_db` (idempotent, alongside the FTS5 DDL). No reads/writes wired yet. *Verify:* tables exist on fresh + existing DB; boot time unchanged. Address or ticket the `ensure_search_db_ready` thundering-herd race (`search.rs:6619-6646`).
- **§2 — Write-path hooks (create/edit).** Extend `index_note` (`search.rs:4079`) to UPSERT the note's `tree_node` row + apply the `folder_stats` ancestor delta. *Verify:* create/edit a note; counts match live `build_tree`; type-burst latency unchanged.
- **§3 — Write-path hooks (rename/move/delete + folder ops).** Extend `rename_item` (`libraries.rs:1004`) + folder create/delete with `tree_node` parent/path updates and `folder_stats` delta-shift, using `__ivm_count__` discipline. *Verify:* BUG-023-shape linked-probe rename; move a folder; delete a folder's last note → its `folder_stats` row disappears (no zombie).
- **§4 — Resumable back-fill.** Background, post-first-paint job with a cursor table, idempotent per-batch upserts (200–500 notes/batch), status-bar progress, resume-on-interrupt. *Verify:* kill mid-backfill, restart, it resumes and converges; final tables equal a full recompute.
- **§5 — Reconcile / self-heal pass.** Periodic O(delta-vs-disk) integrity check to repair drift from out-of-app changes. *Verify:* change a file outside the app; reconcile corrects the tree.
- **§6 — Federation.** Implement cUniverse handling (Open Question 1). *Verify:* a Universe with ≥1 cUniverse shows all federated libraries with correct counts.
- **§7 — Read-path swap behind a toggle.** Reimplement `constellation_map_universe` to read `tree_node` + `folder_stats` (recursive CTE), behind a flag defaulting OFF. *Verify:* flag ON vs OFF produce identical `MapNode` trees; open < 100 ms with flag ON. Measure before/after (Rule 8 hard constraint).
- **§8 — Consumer cutover.** Point `read_library_tree`'s 6 consumers at the `tree_node` skeleton; flip default ON; remove the disk-walk path after a validation period. *Verify:* all 9 consumers render correctly; Editor-Surface Gate + full Boss test.
- **§9 — (Follow-on, separate migration) Lazy + virtualized rendering (Option A half → reach D).** Independent; can ship later.

---

## 7. OPEN QUESTIONS FOR THE BOSS (Eisa)

1. **Child-universe federation strategy.** Should cUniverse subtrees be (a) back-filled into the *active* Universe's `tree_node` and refreshed on attach, or (b) maintained per-cUniverse and UNION-federated at read (matching how `cache_boot_snapshot_graph` already concatenates federated schemas)? (b) is more File-Over-App-faithful but more read-path work.
2. **Scope: how far this migration goes.** A′ only (kill the defect, low risk) · A′ → B (defect now + durable end-state) · A′ → B → D (also lazy+virtualized rendering, biggest).
3. **`body_text` bloat — in scope or separate?** The `note_meta` row-store bloat (`examine:bootpath` §4) taxes even the cheap `load_note_records` read and inflates boot. Moving `body_text` to its own table is a related but distinct schema change.
4. **Toggle lifetime.** How long should the dual-path toggle (§7) stay before the disk-walk fallback is deleted?
5. **Reconcile cadence.** On every boot (post-paint), on a timer, or only on explicit user action / detected external change?
6. **Step 0 form.** Restore the `loadData()` cache-warm, or gate the fullscreen overlay behind `{#if}`?

**One honest gap:** none of the 8 reports measured the *persisted-tree* read latency or the trigger-write overhead on Constellation's actual DB — those are projections from the FTS5/sky_nodes precedent and the external research. The Rule 8 hard constraint (measure before/after on the 7,600-note Universe) must be satisfied empirically in §A′/§2/§7, not assumed.
