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

### Phase 4 — burst hardening (`+layout.svelte` flush) — type-clean

- **Reordered tree-first:** `refreshLibraryTree` now runs BEFORE the reindex, so new/renamed files appear in the sidebar immediately regardless of batch size (a git pull / bulk import no longer waits on indexing for visual feedback).
- **Burst branch:** `reindexPaths.length ≤ 250` → **await** (note_meta readers see rows within ~1 s). `> 250` → **background** (`invoke(...).then(() => { loadAllStats(); refreshLibraryCaches(); })`) so the flush never stalls on minutes of indexing; the tree is already fresh and allNotes/counts refresh when the reindex settles. No chunking needed — `reindex_single_note` takes/releases the writer lock per note (WAL reads interleave), so there is no long lock hold.
- No reconcile fallback (per-file scales and is non-blocking; avoids reconcile's "modified content not reindexed" caveat). **Deferred (honest):** a status-bar "indexing N external changes" progress indicator — files are visible/openable immediately and search catches up silently, so this is a nice-to-have, not shipped.
- `npm run check` → **0 errors**.

### Phase 5 gate — self-review hardening (before the safety-inspection landed) — GREEN

Diff self-review found one latent app-killer + two robustness gaps in `delete_rows_under_prefix`; fixed proactively (WA#6):

- **Offline-drive mass-deletion guard.** A watched folder (or library root) reported *vanished* during a **transient unmount** (network-drive blip, sync mid-move) would have silently purged every `note_meta` row under it — the "offline drive" mass-index-wipe `reconcile` explicitly caps against. Added: refuse if the prefix would purge **>50% of the whole index** (defer to boot reconcile).
- **Per-path re-stat.** Only purge a row whose file is **actually gone** (a spurious "folder removed" while the notes are still on disk keeps its rows) — reconcile's re-stat-before-remove discipline.
- **Add-before-delete ordering** in `reindex_changed_paths` (two passes): a folder rename's NEW rows are indexed before the OLD-side prefix-purge runs its >50% guard, so a legit rename reads as ≤50% and is never refused.
- `watcher.rs` filter → **single `metadata()` stat** per path (was `!exists() || is_dir()` = two stats) — cheaper on the notify watch thread.
- 2 new tests (`delete_rows_under_prefix_refuses_mass_deletion`, `..._keeps_rows_whose_files_still_exist`). Full suite: `cargo test --lib` = **1039 passed** (pre-hardening); watcher-freshness module now **10 passed; 0 failed**.

### Phase 5 gate — safety-inspection (whole-app) — migration diff CLEAN

`safety-inspection` `wf_8a41970f-36d` ran **whole-app** (the `args` reached the workflow JSON-stringified, so `args.files` was undefined → whole-app mode — which is the per-cycle sweep this migration's close triggers anyway). 43 agents, **22 confirmed** (2 APP-KILLER · 7 HIGH · 11 MED · 2 LOW).

- **The migration diff is CLEAN — ZERO confirmed findings in the new code.** Verified by scanning every confirmed finding: none reference `reindex_changed_paths` / `reindex_md_descendants` / `delete_rows_under_prefix` / `library_name_for_path` / the watcher filter / the flush wiring.
- **One finding touched the new command** (`search.rs:9285` — `reindex_single_note` returns `Ok` when `state.db` is None): a pre-existing behavior inherited by `reindex_changed_paths`. **Fixed this build** — added `ensure_search_db_ready(&app)?` at the top of the command (guards the mid-universe-switch edge). Tests unaffected (they call the helpers directly).
- The other 21 are the **standing G2–G8 Safety-Audit backlog, mostly re-confirms of the 2026-07-07 register** (nothing between the sweeps touched those paths). 2 APP-KILLERs are pre-existing G2 content-loss (`NoteEditor.svelte:233` save-before-write; `store.ts:1693` same-tab-nav model clobber). NEW: `yamlDoc.ts:150/254` nested-object-list flatten (a G4 gap), `link_types.rs`/`universe.rs` non-atomic JSON (G6), `bulk_ops.rs:305` TOCTOU, `FocusPane` no-net. Full register appended to `docs/Constellation-Safety-Audit-CHARTER.md`. **Surfaced to Eisa for sequencing — NOT fixed in this build** (pre-existing; would muddle the clean watcher-freshness change; each needs its own Reproduce-First remediation).

Full lib suite: **1039 passed** (pre-guard). Watcher-freshness module: **10 passed** (pre-guard); db-ready guard is a 1-line command-only addition (compiled; doesn't touch the helper tests).

### Boss test — release binary `constellation.exe` (15:06, verified fresh)

Release build clean (`cargo build --release`, 2m04s); frontend re-embedded (`npm run build` first; `reindex_changed_paths` confirmed in `build/`). All 7 phase commits pushed to `origin/main` (`293e77ca..ac87d62d`).

- **Stage 1 (single external note add) — PASS (Boss-validated 2026-07-08).** An `.md` created in Explorer while the app runs became Ctrl+O-jumpable + searchable within ~1s, no restart. The reported bug is fixed.
- **Stage 2 (external folder rename) — PASS (Boss-validated 2026-07-08).** Renamed `OldName`→`NewName` in Explorer; notes findable at the new location, exactly one Search result (old-side ghost rows purged). The Windows one-event-per-folder gap (Phase 3) is closed.
- **Stage 3 (bulk external sync) — PASS (Boss-validated 2026-07-08).** Dropped a batch of `.md` files while typing; typing stayed instant, files appeared immediately, search caught up. Burst handling (Phase 4) validated.

### Watcher-Index-Freshness migration — CLOSED (all 3 Boss stages pass)

The `/migration` is complete and Boss-validated end-to-end. Closeout:
- **Orientation v-bump (SO #6):** `docs/Constellation Orientation & Onboarding v3.32.md` (NEW file; v3.31 retained) — v3.32 changelog block + a migration-index table row. Landed same-day as the migration close.
- **User Manual (SO #2):** added a "Syncing and External Changes" subsection to `docs/User Manual.md` (§2 Universe and Libraries) — plain-language description of the auto-freshness behavior + the external-folder-rename aux-reset caveat.
- **Follow-up (flagged, not silently dropped):** propagate the manual note to the 14 `docs/help.{lang}/` translations. No UI strings changed (no i18n locale-file edits needed) — this is a prose-help propagation only. Spawned as a background task.
- **Audit (migration Phase 4):** satisfied by the per-cycle whole-app safety sweep (diff CLEAN, `wf_8a41970f-36d`) + the 8 invariants + 10 tests + the 3 Boss stages. No schema change / no backfill / no migration-state → trivial migration-path (rollback = revert the commits; the command + wiring carry no persisted state).

**NEXT (recommended, awaiting Boss go):** the 2 pre-existing APP-KILLERs from the sweep — `NoteEditor.svelte:233` (save-before-write) and `store.ts:1693` (same-tab-nav model clobber) — Reproduce-First, one at a time. Both are top of the active Safety Audit.

---

## Docs: sync-note translated to 14 languages (Boss-requested, done)

Workflow `wf_67cb0d21-01f` (14 agents) propagated the "Syncing and External Changes" User Manual subsection into all 14 `docs/help.{lang}/User Manual.md` — inserted in the English slot (§2, after "Managing Libraries"), native technical vocabulary reusing each file's existing feature-name translations, RTL-natural for ar/fa/he/ur. All 14 = exactly +1 `###` heading. Committed `b48bceba`, pushed. *(Prose-only; no UI strings / i18n locale changes. NOTE: a redundant spawned chip-session `task_f0f1a01b` may also be doing this in a separate worktree — this session's commit on `main` is authoritative; the spawned branch is a duplicate to ignore. The AR/EN `.docx` manual exports were not touched — regenerate from the `.md` when convenient.)*

## APP-KILLER #1 — save-before-write (Safety Audit G2) — STARTED (Reproduce-First)

Boss go: "start with the first one." Function in hand: the NotePane **debounced-save path** (`NoteEditor.svelte` `handleSave` → `markSaved` → `writeNote`).

**Mechanism confirmed from code:** `handleSave` (219) calls `markSaved(tab.id, r.version)` at :233 — marking the model CLEAN — BEFORE the awaited `writeNote` at :238, whose rejection is swallowed by `.catch(()=>{})` at :280, and (unlike `handleFlush`:308) with NO `setWriteAhead` net. A transient `.md` write failure (sync/AV lock) → model falsely clean, edit silently lost; `flushAllTabsInLibrary` (`store.ts:695` `if (!isNoteDirty) continue`) then SKIPS the falsely-clean tab and the rename cascade rewrites stale disk (the F2 loss the code claims to prevent). **It's a CLASS:** `handleFlush`:297 + `handlePromote`:194 + `saveTabContent` (`store.ts:765`) + focus-save (`+layout.svelte:1490`) all `markSaved`-before-durable-write.

**Key lever (Solve-the-Class):** the CORRECT shared primitive already exists — `noteSession.ts:84 save()` does `compose → await write → markSaved` (clean only if the await didn't throw). The component sites BYPASS it with an inlined wrong order + no net. Fix direction: consolidate every save onto one correct primitive (net-before-write → await → mark-clean-only-on-success + clear-net; on failure keep dirty + retain net + SURFACE the error, never silent).

Adversarial Architect workflow `wf_16260085-719` (4 agents) COMPLETE → `docs/APP-KILLER-Save-Durability-Architect.md`. Census confirmed 5 wrong sites (handleSave/handleFlush/handlePromote/saveTabContent/commitFocusSave) vs the CORRECT `noteSession.save()`; WA#5 = VS Code dirty-until-resolved + hot-exit journal is the universal rule. Verified myself (WA#1): `write_note` → `write_gate::atomic_write` is **already atomic + fsync-durable** (Architect decision #3 resolved).

### Plan APPROVED (Boss, 2026-07-08) — full 8-step `/migration`

- Scope: **all 8 steps** (class fix — durability everywhere + recovery net + visible notice).
- Save-fail notice: **top banner + Retry** (Obsidian-style, path-coalesced, i18n ×15).
- Auto-retry: **~10 s timer** while a save stays dirty.

### Step 1 — the hardened primitive, proven headless — GREEN

- `noteSession.save()` rewritten into the durability contract: `compose → setNet(before write) → try await write → catch: onError + return {write_failed} (NO markSaved; net RETAINED) → markSaved → clearNetIf(compare-and-clear) → onSuccess`. **Backward-compatible** — the 3rd arg accepts `DiskWriter | SaveEnv`, so the existing harness + minimal callers are unchanged; production sites pass the full env. Never throws on a write failure (returns `write_failed`).
- `store.ts`: `clearWriteAheadIf` (compare-and-clear, INV-3), `saveHealth` writable + `reportSaveFailure`/`clearSaveFailure` (path-keyed, coalesced), `standardSaveEnv({origin, name, onSaved})` factory (write + net + surface + optional on-durable-write hook).
- Migrated the 5 already-correct flush sites (`store.ts` task_toggle/flush_all/rename/structural + `lens/store.ts` base_edit) onto `standardSaveEnv` — they gain the net + surface, keep the correct order.
- Harness `tests/mig-076/runtimeHarness.test.ts` +5 cases: GREEN (failed write → dirty + net retained + surfaced once + nothing written), SUCCESS (clean + compare-and-clear with the written content), **RED baseline** (inlined markSaved-before-write → falsely-clean), type-during-await (newer edit stays dirty), compare-and-clear (newer net survives). **16/16 pass.** *(App behavior for the wrong component sites is unchanged yet — Phase 2 reroutes them.)* Commit `81d5873c`.

### Phase 2 (Steps 2–6) — reroute all 5 wrong save sites — GREEN

All five now go through `saveNoteSession` + `standardSaveEnv` (mark clean only on a durable write; net + surface; the inlined markSaved-before-write + swallowed `.catch(()=>{})` deleted at each): **handleSave** (the debounced APP-KILLER) · **handleFlush** (markSaved→success, net via the gate, no-write path stashes the net) · **handlePromote** (+ADDS the reindex it never had, INV-7) · **saveTabContent** (post-write side effects → `onSaved`, run only on durable write) · **commitFocusSave** (+fixed the false "error is surfaced" comment — now genuinely via save-health). Dropped now-unused markSaved imports. svelte-check 0 errors; harness 16/16. Commit `5519fa4a`.

### Phase 3 Step 7 — save-health banner + Retry + ~10s auto-retry + i18n ×15 — GREEN

`SaveHealthBanner.svelte` (non-blocking top banner, one row/failed note, auto-dismiss on success) mounted in main + SS `+layout`; `store.ts::retrySaveFailure(path)` (re-drives an open+dirty note); a ~10s auto-retry `$effect` in main `+layout` (runs only while failures remain, teardown-cleaned); `saveHealth.couldNotSave/{note}`+`retry` in all 15 locales (JSON-validated). svelte-check 0 errors; harness 16/16. Commit `ed4c9d4d`.

### Step 8 — gate (in progress)

`safety-inspection` `wf_5f9b257d-a99` (42 agents, ran whole-app again — the workflow's `args` arrive stringified, so it never enters diff-mode; still covers the diff files). **23 confirmed — the migration's own diff is CLEAN (zero NEW app-killers from the 5-site reroute).** Every finding cross-checked against the new code (primitive / reroutes / banner / compare-and-clear).

**WA#6 fix folded in:** `store.ts:824` — `saveTabContent`'s single-flight `saveLocks` guard silently dropped a concurrent property edit (early-returned BEFORE `editNoteProps` reached the model). Same silent-loss class; fixed by moving `editNoteProps` (the model update) BEFORE the guard → a concurrent edit always lands in the model (dirty), persisted on the next save/flush; the guard now serializes only the WRITE. svelte-check 0 errors; harness 16/16.

The other 22 are the standing G2–G8 backlog (appended to the Charter). **Highest-priority remaining = the 2nd APP-KILLER, still open:** `store.ts:1787` (`openNoteTab` in-place reuse discards the outgoing dirty model, no flush → ~30 s nav-loss) + `+layout.svelte:3320` (two-tab same-note clobber) → the **next migration** (notemodel-ownership: flush-before-replace). Surfaced to Eisa.

**`write_note` verified already atomic + fsync-durable** (`write_gate::atomic_write`), and `replace_file` (Windows `ReplaceFileW`) fails on a read-only target → a Boss-reproducible way to force a save failure for the Editor-Surface Gate test. Rebuilding the binary with all fixes → Boss test.
