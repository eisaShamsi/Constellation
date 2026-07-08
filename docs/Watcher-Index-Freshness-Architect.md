# Watcher Index-Freshness — Runtime reindex on external `.md` change (Architect)

**Status:** Architect complete → awaiting Plan approval
**Opened:** 2026-07-08 · Owner: Claude · Boss: Eisa
**Analysis:** workflow `wf_9192113e-92f` (4 agents): stale-surface census · reindex-primitive/watcher census · WA#5 research (Obsidian/VS Code/Logseq/FTS5) · adversarial design (all 9 refutation points addressed).

---

## Concept (the horse)

> **When a note changes on disk from *outside* the app, every derived surface that answers a question about that note must become current in the same debounced beat — because Constellation is file-over-app, and a file the user can see in the tree but cannot search, jump to, or backlink is a broken promise.**

The function (the carriage): a scoped, watcher-driven, per-path **incremental reindex** of only the changed rows.

## 1. The gap (confirmed from code)

The file watcher (`watcher.rs:82-90`) is **emit-only** — on an external `.md` change it emits `library-changed {libraryId, paths}` and *never* reindexes. This is deliberate: taking the DB lock + `BEGIN IMMEDIATE` on notify's watch thread risks `ReadDirectoryChangesW` buffer overflow → dropped events. The JS flush (`scheduleWatcherFlush`, `+layout.svelte:3094`) refreshes the **filesystem tree** (`read_library_tree` → fresh) but calls **no reindex**. The only runtime reindex paths are the app's *own* writes (`store.ts:778` save, `+layout.svelte:1501` focus save), which are `watcher_suppress`-filtered. External changes are picked up **only at boot** (`reconcile::run` orphan-walk + re-adopt; `maybe_schedule` runs boot-only, `search.rs:8806`).

**Result: the file tree is fresh, but every `note_meta`/`notes_fts`/`note_links`-derived surface is stale until reboot.** Census (verified):

| Surface | Reads | External note (never runtime-reindexed) |
|---|---|---|
| File tree | filesystem (`read_library_tree`) | ✅ fresh |
| Wikilink *resolution* (click `[[New]]`) | filesystem walk (`resolve_wikilink`) | ✅ fresh → **phantom state** |
| Quick Switcher (Ctrl+O) | `allNotes` ← `note_meta` | ❌ stale |
| SearchHub full-text | `constellation_search` ← `notes_fts` | ❌ stale |
| SearchHub wikilink autocomplete / semantic | `allNotes` / `note_embeddings` | ❌ stale |
| Index panel / term mentions | `notes_vocab` / `notes_fts` | ❌ stale |
| Backlinks / Outgoing / graph fallback | `note_links` | ❌ stale |
| Sidebar + Dashboard counts | `note_meta` (`get_all_library_stats`) | ❌ stale |
| Tag browser / aliases map | `note_meta` (`cache_boot_snapshot_graph`) | ❌ stale |
| Sky View / Map / Sight / Review / Bases | `note_meta` / `sky_*` | ❌ stale (some flag-disabled) |

**Phantom-note hazard:** `resolve_wikilink` walks the filesystem (fresh), so an existing note's `[[New Note]]` opens the new file — yet that same note is unsearchable, has zero backlinks, and is missing from counts. Confusing, inconsistent, until reboot.

## 2. Instance vs class

- **Instance (Boss's first suggestion):** upsert `allNotes` in JS on the watcher event. Fixes *only* Quick Switcher + wikiAuto (the two `allNotes` consumers). Leaves full-text/Index/backlinks/counts/tags stale. **Worse:** forces the frontend to re-derive name/aliases from raw YAML in JS — duplicating the Rust indexer just hardened in **G4 Phase 4** → guaranteed drift (violates Don't-Duplicate + Solve-the-Class).
- **Class (recommended):** on the watcher hook, reindex the changed `.md` paths into `note_meta` via the **same** `reindex_single_note`/`reindex_delete_note` the app already uses. Then *every* derived surface becomes current in one write — no per-surface patching, no duplicated decode. This is Rule 8 placed exactly where CLAUDE.md prescribes ("wire a trigger/hook on the source-of-truth write path").

## 3. WA#5 research verdict

The industry standard is **decisively (a): debounced, per-path incremental reindex on the watcher event** — reindex only the changed file's rows; full-walk reserved for cold boot / root switch / desync recovery. Obsidian (`metadataCache.on('changed', file)` fires *after* that one file re-indexes), VS Code (dirty-queue + debounce, "only reparses what changed"), Logseq file-graph (parse the changed file → patch DB), SQLite FTS5 (per-row maintenance by design). Self-writes de-duped by a **suppress-by-path set** *plus* an **idempotent hash/mtime guard** so a slipped-through self-event is a no-op. Debounce anchors: chokidar `awaitWriteFinish` 2000 ms / 100 ms poll; Watchman `settle` 20 ms. Constellation's existing 300 ms JS debounce + `watcher_suppress` (2.5 s TTL) + `force=true` idempotency already realizes this pattern — the fix just adds the reindex call.

## 4. Final hardened design

**Rust — new `#[tauri::command(async)]` `reindex_changed_paths(app, paths: Vec<String>)`** (in `search.rs` after :9180). Per path, decide by **`exists()` at flush time** (last-writer-by-reality), swallow per-path errors (never abort the batch):

- existing `.md` → `reindex_single_note` (G4 decoder, no JS re-derivation); lib via shared `library_name_for_path`.
- existing **dir** → `reindex_md_descendants` (bounded `read_dir_recursive`) — closes the Windows dir-rename **new-side** gap (`ReadDirectoryChangesW` emits one dir event, not per-child).
- vanished `.md` → `reindex_delete_note` (idempotent no-op if no row).
- vanished **dir** → `delete_rows_under_prefix` (`path LIKE norm(prefix)+sep+'%'`, separator-guarded so `Notes/` never matches `NotesArchive/`) — closes the dir-rename **old-side** gap.

A dir rename emits **both** old+new dir paths in one batch → new subtree reindexed **and** old-prefix rows deleted.

**Shared resolver:** extract `reconcile.rs::lib_for` + `load_all_libraries` into `libraries::library_name_for_path(app, path)` (longest-root-wins), used by both reconcile and the new command (Don't-Duplicate).

**JS — wire into the existing flush** (`+layout.svelte scheduleWatcherFlush`), watcher.rs stays **unchanged** (emit-only):
- add `pendingReindex: Set<string>`; populate it in the `library-changed` handler behind the same `wasRecentlyWritten` guard.
- at flush top: `await invoke('reindex_changed_paths', { paths })` **FIRST**, *then* `refreshLibraryTree`/`loadAllStats`/`refreshLibraryCaches` (they read the now-current `note_meta`).
- re-point the 5 s `cacheRefreshDebounce` to fire **after** the awaited reindex (prompt, not a 5 s hedge).
- fix the stale comments at `+layout.svelte:2499-2500` / `:2505`.

## 5. Adversarial refutation (all 9 — see workflow output for full text)

1. **Loop/echo:** ✅ reindex reads the `.md`, writes only SQLite (in app-data, outside the watched root) → no notify re-fire.
2. **Double-reindex:** ✅ app writes `watcher_suppress`-marked pre-write; slipped-through = idempotent `force=true` (wasted work, never corruption).
3. **Delete discrimination + create-then-delete race:** ✅ decide by `exists()` at flush; TOCTOU → per-path error swallowed, batch continues.
4. **Directory rename (the real Windows gap):** ⚠️→✅ hardened via `reindex_md_descendants` (new) + `delete_rows_under_prefix` (old).
5. **Burst (git pull = 500 files):** ⚠️→ chunk txns ~50-100; soft cap ~500 (background + status bar); hard cap ~2000 → `reconcile_filesystem` fallback (honest limit: reconcile won't reindex *modified content* above the cap). **Boss decision.**
6. **Thread/lock vs snapshot reads:** ✅ `cache_boot_snapshot_core` reads via `open_reader` (dedicated WAL read-only conn, no mutex contention) — never blocks/tears against the writer.
7. **Ordering:** ✅ `(async)` Promise resolves on completion; `await` before readers guarantees `note_meta` committed first.
8. **JS command vs Rust-callback debounce:** ✅ JS command lower-risk — keep watch thread emit-only (no `ReadDirectoryChangesW` overflow); reuse the working 300 ms debounce (Don't-Duplicate); `(async)` routes off UI+watch threads. *(Future: Rust mpsc coalescing task survives a closed WebView — noted, not Phase 1.)*
9. **Reproduce-First harness:** RED baseline (write `.md` to disk directly → 0 search hits, no `note_meta` row) · GREEN add · frontmatter torture (block-scalar name) · delete · Windows dir-rename (old+new) · race (nonexistent path → clean no-op).

## 6. Invariants (audit checklist)

1. No `.md` write in the reindex path → loop-free.
2. App writes stay watcher-suppressed → never double-reindexed; slipped-through idempotent.
3. Zero `invoke()` on the keystroke hot path — reindex only in the 300 ms flush.
4. No full-Universe walk on a single external change (Rule 8 / Rule 3) — scoped + bounded subtree.
5. Reindex off the UI thread AND the notify watch thread — watch thread stays emit-only.
6. `note_meta` committed before any `note_meta`-reader (`await` ordering).
7. WAL reader isolation — snapshot reads never block/tear vs the writer.
8. Editor-Surface Gate: reindex touches index tables only, never an open editor buffer.

## 7. Phased plan (each = one commit + verification clause, Reproduce-First)

- **Phase 0 — RED harness.** Rust `#[cfg(test)]`: external-add staleness (RED on today's tree) + primitive add/delete/frontmatter (GREEN). Commit `reproduce-first: external-change staleness harness`.
- **Phase 1 — Rust command, file-level.** `reindex_changed_paths` (`.md` exists→reindex, gone→delete, skip non-md, per-path error-swallow) + extract `libraries::library_name_for_path`. *Verify:* Phase-0 add/delete/frontmatter green; not wired.
- **Phase 2 — wire the flush (Boss-testable).** `pendingReindex`; `await` reindex before readers; re-point `cacheRefreshDebounce`; fix stale comments. *Verify (Boss tutorial):* add a `.md` in Explorer/Obsidian → within ~1 s jumpable in Ctrl+O, in Search Hub full-text, sidebar count increments — no reboot.
- **Phase 3 — directory completeness.** `reindex_md_descendants` + `delete_rows_under_prefix`. *Verify:* Windows dir-rename test green; Boss test — rename a folder externally, notes stay searchable at new location, vanish from old.
- **Phase 4 — burst hardening.** Chunked txns + soft/hard thresholds + status-bar progress + reconcile fallback. *Verify:* 500-file git-pull sim; typing instant, boot/IPC unregressed, all searchable when status bar clears.
- **Phase 5 — gate.** `safety-inspection` (diff-scoped) + `/simplify` + help/manual/orientation v-bump. *Verify:* zero confirmed app-killers; docs same commit.

## 8. Decisions for Eisa (defaults in **bold**)

1. **Burst thresholds (Phase 4):** soft cap **500** (background + status bar), hard cap **~2000** (reconcile fallback). Confirm.
2. **Directory-rename old-side (Phase 3):** **include the prefix-scoped delete now** (WA#6 — closes a discovered Windows gap) vs defer bare-dir moves to next-boot reconcile.
3. **Open-note external MODIFY (pre-existing, related):** today `+layout.svelte:3118` live-reloads an open tab from disk on external change; Obsidian instead reconciles on focus (protects unsaved edits — Editor-Surface Gate). **Keep live-reload for now**, flag as separate future item, or switch?
