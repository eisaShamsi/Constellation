# Session Log — 2026-07-08

Continues from `SESSION-LOG-2026-07-07.md` (MIG-099 create-latency fix + G4 frontmatter round-trip Phases 0–4, all Boss-validated, pushed to `origin/main` @ `293e77ca`).

---

## Function in hand: Runtime index freshness on EXTERNAL file changes (the "Quick Switcher stale" report)

**Boss report / task:** an `.md` note added on disk *externally* while the app runs (Obsidian sync, git pull, manual drop) is **not** Ctrl+O-jumpable until relaunch. Boss framed it as "the `allNotes` boot cache doesn't refresh," suggested fix = upsert into `allNotes` on the watcher event.

### Reproduction / mechanism — CONFIRMED from code (Reproduce-First)

The root cause is **deeper and broader** than the Quick Switcher. It is a single **Rule-8 (write-time-derivation) gap on the file-watcher hook**:

- `src-tauri/src/watcher.rs` (the `notify` callback) — on an external `.md` change it **only** `emit`s `library-changed {libraryId, paths}`. It **never reindexes** the changed file into the derived index (`note_meta` / `note_links` / `notes_fts`). App-created writes are `watcher_suppress`-filtered, so `library-changed` fires *only* for genuinely external changes.
- `src/routes/+layout.svelte` `scheduleWatcherFlush` (~L3094) — on `library-changed` it refreshes the **filesystem** tree (`read_library_tree` → fresh ✅), reloads open tabs, and 5 s later calls `refreshLibraryCaches()` which reads `cache_boot_snapshot_core` (= `note_meta`) into `allNotes`. But `note_meta` was never updated → the refresh reads a stale index.
- The **only** runtime paths that reindex are the app's *own* writes (`create_note`, rename cascade, add-tag/add-link → `reindex_single_note`). External changes are picked up **only at boot** — `reconcile::run` orphan-walk (step 4) + re-adopt (step 9); `reconcile::maybe_schedule` runs **boot-only** (`search.rs:8806`).

**Therefore every `note_meta`-derived surface is stale until reboot, not just Ctrl+O:**

| Surface | Reads | External note (never runtime-reindexed) |
|---|---|---|
| File tree | filesystem (`read_library_tree`) | ✅ fresh |
| Quick Switcher (Ctrl+O) | `allNotes` boot cache (`+layout.svelte:870`) | ❌ stale |
| SearchHub wikilink autocomplete (`wikiAuto`) | `allNotes` boot cache | ❌ stale |
| SearchHub full-text | `constellation_search` → live `note_meta`/FTS (`store.ts:1919`) | ❌ **also stale** |
| Index panel / backlinks | `note_meta` / `note_links` | ❌ stale |

### Instance vs class

- **Instance fix (Boss's suggestion):** upsert `allNotes` in JS on the watcher event. Fixes Ctrl+O only; leaves SearchHub full-text / Index / backlinks stale. **Worse:** it forces the frontend to re-derive a note's name/aliases from raw frontmatter *in JS*, duplicating the Rust indexer just hardened in **G4 Phase 4** → guaranteed to drift (violates Don't-Duplicate + Solve-the-Class).
- **Class fix (recommended):** on the watcher hook, **reindex the changed `.md` paths into `note_meta`** (scoped, per-path — `reindex_single_note` for create/modify, `reindex_delete_note` for delete). Then the *existing* frontend refresh makes `allNotes` **and** SearchHub full-text / Index / backlinks all current. This is Rule 8 placed exactly where CLAUDE.md says: "wire a trigger/hook on the source-of-truth write path." The watcher IS that hook for external changes.

Architect workflow `wf_9192113e-92f` (4 agents) COMPLETE → `docs/Watcher-Index-Freshness-Architect.md`. Census expanded the stale-surface list (semantic, tags, aliases, counts, Sky/Map/Sight/Review/Bases). Adversarial pass caught the **Windows dir-rename gap** (`ReadDirectoryChangesW` emits ONE event for a renamed folder, not per-child — naïve per-file reindex would lose every note inside). WA#5: the fix shape (debounced per-path incremental reindex) is the Obsidian/VS Code/Logseq/FTS5 standard. WAL `open_reader` isolation verified — reindex writer never blocks/tears the snapshot read.

### Plan APPROVED (Boss, 2026-07-08)

- **Scope:** full class fix, all 6 phases (incl. Phase 3 Windows dir-rename hardening).
- **Burst thresholds:** soft 500 (background + status bar), hard ~2000 (reconcile fallback) — defaults accepted.
- **Open-note external MODIFY:** Boss chose **focus-reconcile (Obsidian-style)** over today's live-reload. → **SEPARATE follow-up** after the freshness fix (touches the open editor buffer = Editor-Surface Gate / BUG-015/023 territory; its own Reproduce-First harness + review). Queued, NOT bundled into this migration.

**Migration-class** (Rust watcher/index ↔ Svelte cache ↔ derived-index write path). Plan-Approval = Build-Approval → cascading Phases 0→5, stopping at the Phase 2 + Phase 3 Boss tests.

### Phase 0 — RED harness (Reproduce-First) — GREEN

`search.rs::tests_watcher_freshness` (4 tests, `init_db` real-schema temp DB + `SearchState`, real `.md` files on disk):
- `external_add_absent_until_reindex_then_present` — RED precondition (written to disk, absent from `note_meta`) → GREEN after `reindex_single_note` (present, title decoded as name). The core reproduction.
- `block_scalar_title_decodes_via_g4_reader` — a `title: |` block scalar decodes to the words, NOT the literal `|` — proves the fix reuses the G4-hardened Rust decoder (no JS re-derivation).
- `external_delete_purges_note_meta_and_links` — vanished `.md` → `reindex_delete_note` purges `note_meta` + `note_links`.
- `nonexistent_path_delete_is_a_clean_noop` — create-then-delete race / never-indexed path → clean `Ok`, no abort.

`cargo test --lib tests_watcher_freshness` → **4 passed; 0 failed** (0.20s). Harness proven; safe to build the fix.

### Phase 1 — Rust command, file-level — GREEN

- `libraries.rs::library_name_for_path(libs, path)` — shared longest-root-wins resolver (separator-bounded, case/backslash-insensitive), batch-friendly (`load_all_libraries` once per flush). Mirrors `reconcile::lib_for`; the single resolver the reindex path uses (Don't-Duplicate). `reconcile.rs` left untouched (risk containment — its `lib_for` is proven).
- `search.rs::reindex_changed_paths(app, paths)` — `#[tauri::command(async)]`. Per path, decide by `exists()` at flush time: existing `.md` → `reindex_single_note` (G4 decoder, no JS re-derivation); vanished `.md` → `reindex_delete_note` (idempotent); non-`.md`/dirs skipped (Phase 3). Per-path errors swallowed (batch never aborts). Loop-free (reindex writes only SQLite, outside the watched root). Registered in `lib.rs` handler.
- New test `library_name_for_path_picks_most_specific_root` (nested-wins, separator bound, Windows path). `cargo test --lib tests_watcher_freshness` → **5 passed; 0 failed**. Command not yet wired to the flush (Phase 2).

### Phase 2 — wire the watcher flush (`+layout.svelte`) — type-clean

- `pendingReindex: Set<string>` populated in the `library-changed` listener behind the same `wasRecentlyWritten` guard as `pendingTabReloads` (external changes only).
- `scheduleWatcherFlush` now **awaits `reindex_changed_paths(reindexPaths)` FIRST**, before `refreshLibraryTree`/`loadAllStats`/`refreshLibraryCaches` — so every `note_meta` reader sees the committed rows. The `(async)` Promise resolves only on completion → ordering guaranteed.
- `cacheRefreshDebounce` re-pointed 5000 ms → **800 ms**: `note_meta` is already current after the awaited reindex, so `allNotes`/tags/aliases (the Quick Switcher source) repopulate within ~1 s of the change instead of 5 s. Short debounce still coalesces a burst; `refreshLibraryCaches` self-guards re-entry.
- Fixed two stale comments the census flagged (`cache_reconcile` "triggered by the file watcher per-file" → false; `loadAllStats` "metadata-only walk" → reads `note_meta`, not a walk).
- `npm run check` → **0 errors** (317 pre-existing unused-CSS warnings). Boot/typing untouched (no hot-path change). Awaiting the Boss single-file-add test after the binary build (bundled with Phases 3–4).

### Phase 3 — directory / folder-rename completeness — GREEN

**Subtlety the Architect spec missed (found during build):** a renamed-away folder's OLD path is *filtered out* by the watcher (`is_dir()` false — it's gone — and not `.md`), so the old-side cleanup signal never reached the frontend. Two-part close:

- `watcher.rs` — relaxed the path filter to `!p.exists() || p.is_dir() || is_md` (still **emit-only**; the reindex runs off-thread). Passes vanished files/dirs so the old-side signal arrives; existing non-`.md` files stay ignored.
- `search.rs` — `reindex_changed_paths` now dispatches by kind: existing dir → `reindex_md_descendants` (NEW side: index not-yet-known `.md` under a renamed/moved/bulk-added folder; skips already-known → spurious dir-modify is cheap); vanished non-`.md` → `delete_rows_under_prefix` (OLD side: purge rows under the gone folder). Prefix match is a **Rust `starts_with`, NOT SQL `LIKE`** — Constellation paths contain `_` which `LIKE` treats as a wildcard (a `LIKE prefix||'%'` would over-match and delete unrelated notes = app-killer). Separator-bounded (`…/Old` ≠ `…/OldArchive`); each victim via `reindex_delete_note` (former-target counts/sky maintained).
- **Known limitation (documented):** an external folder RENAME lands as delete-old + index-new (NOT a `cid_cn` relocate), so per-note aux (review schedule, own outgoing-link weights) resets. Incoming links survive (name-keyed). Acceptable for a rare external rename; boot reconcile relocate-by-cid is the gentler heal.
- 3 new tests (folder rename new+old, spurious-event skip, separator+underscore guards). `cargo test --lib tests_watcher_freshness` → **8 passed; 0 failed**.
