

==========================================================================================
## MAPS :: db-model
==========================================================================================

**Per-Universe Database Model — Factual Map (evidence: file:line, all under `E:\مشاريع كلاود\Constellation\src-tauri\src\`)**

---

## (1) How the active universe's search.db is located and opened — and who opens a SECOND universe's DB

**Location is ambient, always.** `db_path()` = `active_constellation_dir(app).join("search.db")` (`search.rs:1465-1468`). `active_constellation_dir` → `active_universe_dir` reads `UniverseState.active_path: Mutex<Option<PathBuf>>` — Tauri managed state, the single process-wide "which universe" pointer (`universe.rs:69-72`, `universe.rs:340-344`, struct at `universe.rs:41-50`). So the DB is `<universe_root>/.constellation/search.db`, and its schema-version marker is `<...>/.constellation/search.version` (`path.with_extension("version")`, `search.rs:11541`, current version string `"7"` at `search.rs:11542`).

**Open sequence** — `ensure_search_db_ready` (`search.rs:11476`): lock-free `db_ready` fast path (11483) → `db.is_some()` check (11491-11496) → `init_lock` serialization (11508-11511, MIG-078 §B1 thundering-herd fix, poison-tolerant) → double-check (11515-11520) → capture `federation_generation` BEFORE `db_path()` (11537-11540, the Batch-2 §B2-5 epoch fix) → schema gate (`schema_gate`, `search.rs:1457-1463`: absent-marker = adopt+stamp, never delete; mismatched marker = rename-aside `search.pre-v7-<stamp>.db` + delete WAL/SHM siblings, 11573-11601) → `link_types::load_active(app)` (11606 — loads the ACTIVE universe's link vocabulary into a process global, see §2) → `init_db(&path)` (11607) → open read-only `read_db` (11630) → publish `db` + `read_db` + `db_ready=true` together under ONE generation check (11633-11660) → ~14 backfill `maybe_schedule` calls + reconcile + derived_heal (11661-11745), all taking `app` and re-resolving the ambient active universe.

**`init_db`** (`search.rs:4592-4594` → `init_db_scoped` 4601): `Connection::open` (read-write, 4603), `PRAGMA journal_mode=WAL` (4606), `synchronous=NORMAL; busy_timeout=5000; mmap_size=256MB` (4619-4622), `recursive_triggers=ON` (4641), per-connection FTS5 tokenizer registration (4649; tokenizer registration is connection-local — `search.rs:1558-1581`), then full schema DDL + `PRAGMA user_version` FTS gate (`FTS_SCHEMA_VERSION=1`, `search.rs:32`) + `schema_versions` module ledger (4678-4698, e.g. `SKY_SCHEMA_VERSION=10` at `search.rs:73`).

**Code that opens a SECOND universe's DB today — four sites, all in the federation subsystem:**

1. **Read-only ATTACH** — `federation::attach::attach_all` (`attach.rs:125-206`): for each cUniverse root (found by walking parents to `.constellation/universe.json`, `attach.rs:105-118`), `ATTACH DATABASE 'file:<path>?mode=ro' AS cu<i>` onto a **dedicated parent connection** (`attach.rs:228-233`), schema-column verification via `PRAGMA cu<i>.table_info(note_meta)` (`attach.rs:266-295`), cap 25 (`attach.rs:45`). The host connection is opened fresh in a background thread on the ACTIVE universe's `db_path` (`search.rs:11791-11804`), tokenizer registered (11834), stored as `SearchState.federated_conn` after a generation check (11894-11915).
2. **Write, schema-only migrate** — `federation::migrate::run_migrations_on` (`migrate.rs:68-180`): `BEGIN EXCLUSIVE` lock probe (`is_cuniverse_open_elsewhere`, `migrate.rs:191-200`), `fs::copy` backup, then `init_db_schema_only(cu_db_path)` (`migrate.rs:133`) = `init_db_scoped(path, InitScope::ForeignSchemaOnly)` (`search.rs:4597-4599`). `InitScope` (`search.rs:4571-4589`) exists because full `init_db` on a foreign DB persisted the PARENT's link-type registry into the child's trigger DDL and wrote frontmatter into the child's `.md` files (PJ-230/PJ-232 history, `migrate.rs:86-131`). ForeignSchemaOnly skips registry-generated DDL and every one-shot MIG-003 pass; keeps plain schema DDL + FTS rebuild.
3. **Write, FTS optimize** — `federation_prewarm` (`search.rs:11358-11474`): opens each cUniverse's search.db **read-write** (`Connection::open`, 11381), `busy_timeout=30000` (11397), and executes `INSERT INTO notes_fts(notes_fts) VALUES('optimize')` — an acknowledged "optimize-write" into the child's DB (11393-11436).
4. **Tests only** — `cache.rs:1672+` attaches synthetic DBs.

**Nothing else.** Every routine note operation (save/reindex/delete/rename) goes through `SearchState.db` — the active universe's connection — via `reindex_single_note(state, ...)` (`search.rs:12682-12688`, `let db = state.db.lock()`), called from 15+ sites (`libraries.rs:1440,1880,1957,2641,2782,7042`, `universe.rs:2436`, `bases.rs:412`, `tasks.rs:540`, `shape.rs:214`, `index_repair.rs:853`, `reconcile.rs:469,565`). There is **no** path that routes a write to a note's OWN (non-active) universe DB; instead the PJ-235 write-scope guard `require_own_library` (`libraries.rs:295-345`) **refuses** foreign-path writes ("reads but never writes"), and `owning_own_library_name` is pinned to the own, non-recursive registry (`libraries.rs:266-272`). The bulk walk is likewise own-scoped and skips `foreign_library_roots` (`search.rs:12004-12017`).

---

## (2) Process-global vs per-DB state

**Per-DB/per-universe, held in process-global singletons keyed to the ACTIVE universe (invalidated on switch):**
- `SearchState` (managed once, `lib.rs:359`): `db: Mutex<Option<Connection>>` — THE writer lock (`search.rs:1342`); `read_db: Mutex<Option<Connection>>` — read-only WAL reader, PJ-066 §C3 (`1353`); `db_ready: AtomicBool` (`1363`); `init_lock: Mutex<()>` (`1374`); `federation: Mutex<FederationContext>` (`1379`); `federated_conn: Mutex<Option<Connection>>` (`1400`); `federation_generation: AtomicU64` — the epoch counter every background thread checks (`1415`).
- Universe switch: `set_active_universe` flips `UniverseState.active_path` (`universe.rs:1040-1043`), `invalidate_libraries_cache()` (`universe.rs:1047`), `invalidate_search_state` (`universe.rs:1057` → `search.rs:11228-11284`: bump generation FIRST, clear `db_ready` twice — once before and once under the `db` lock, PJ-207 §15 — null `db`, `read_db`, `federated_conn`, reset `FederationContext`).
- `LIBRARIES_CACHE: static Mutex<Option<(PathBuf, Vec<LibraryInfo>)>>` — keyed by active universe path (`libraries.rs:181-182`).

**Truly process-global (ONE per process, no per-universe copy — the migration hazards):**
- **`link_types::REGISTRY: static OnceLock<RwLock<LinkTypeRegistry>>`** (`link_types.rs:351`) — loaded ONLY from the active universe (`load_active`, `link_types.rs:520`, called at `search.rs:11606`). This is the reason `InitScope::ForeignSchemaOnly` exists: trigger DDL is generated from `link_types::snapshot()` (`migrate.rs:110-116`), so full-initing a child DB from the parent's process stamps parent vocabulary into the child. **Any side-by-side full open of a child DB requires per-universe (or per-connection-snapshot) link vocabulary.**
- **Arabic overrides store** — process-wide, swapped on switch, with child layers stacked under parent (`universe.rs:1059-1093`).
- **`watcher_suppress` map** — `static OnceLock<Mutex<HashMap<PathBuf, Instant>>>` (`watcher_suppress.rs:35-36`), path-keyed so it is universe-agnostic in practice, but per-process (see §4).
- `WatcherState.watchers: Mutex<HashMap<String /*library_id*/, RecommendedWatcher>>` (`watcher.rs:9-11`) — keyed by library id, not by universe.
- FTS5 tokenizer: **connection-local**, never global (`search.rs:1558-1581`) — every new connection on any DB must re-register.

**Other connections on the SAME active DB** (WAL multi-reader model): `cache.rs::open_reader` — own READ_ONLY connection per boot-snapshot call, deliberately bypassing `SearchState.db` (`cache.rs:50-69`); `reconcile_filesystem`'s dedicated `walk_conn` writer (`search.rs:11968-11984`); each backfill opens its own connection; `libraries.rs` read-only opens for Index panel. WAL + `busy_timeout` is the concurrency contract throughout.

---

## (3) What opening a child universe's search.db side-by-side would take

Facts as they stand:

- **Path resolution**: mechanical — `<cu_root>/.constellation/search.db` (`attach.rs:156`); cu roots come from `resolve_universe_libraries` (`universe.rs:1513-1517`, recursive via `resolve_libraries_recursive` at `universe.rs:602`) + parent-walk to `universe.json` (`attach.rs:105-118`).
- **Schema-version gate**: the `search.version` marker gate (rename-aside, stamp-absent) runs ONLY inside `ensure_search_db_ready` for the active DB (`search.rs:11541-11625`). The federation path never evaluates the child's marker; it substitutes column-verification (`attach.rs:266-295`) + `init_db_schema_only`. A side-by-side **writable** open would need the marker gate replayed per-child (the gate logic itself is already isolated and unit-testable: `schema_gate`, `search.rs:1448-1463`) — and the rename-aside must anticipate the child being open in another process (the `Err` arm at `search.rs:11588-11596` already names "another instance" as the common Windows failure).
- **Init**: `init_db_scoped` exists with exactly two scopes (`search.rs:4571-4589`). Full `Active` init on a child is **forbidden today** for two documented reasons: registry-generated trigger DDL (parent vocabulary) and the one-shot MIG-003 passes that write frontmatter / rename the child's `.md` files (`search.rs:4576-4581`, `migrate.rs:106-125`). Side-by-side writable opens therefore need either (a) a per-universe registry load so DDL is generated from the CHILD's `.constellation/link-types.json`, or (b) continued ForeignSchemaOnly + owner-does-the-rest semantics.
- **WAL**: a file property, inherited by every subsequent open (`search.rs:1473-1474`); set once by `init_db` (`search.rs:4606`). No work needed.
- **Tokenizer**: per-connection registration is mandatory or FTS MATCH silently returns zero rows (proved live: MIG-056 §K.1 hotfix, `search.rs:11823-11842`).
- **Backfills/reconcile/heal**: all 14 `maybe_schedule(app)` calls resolve the ambient active `db_path` (`search.rs:11661-11742`); none can run against a child DB as written. The current model defers them to the child's own next launch (`migrate.rs:102-104`). A side-by-side model must decide per-surface whether that stays true.
- **Watcher**: already covers child directories — the frontend fans out `watch_library` over `$libraries`, which is the **federation-recursive** list (`store.ts:3925-3932` invokes `resolve_universe_libraries`; `+layout.svelte:2944-2947`). Events flow into `reindex_changed_paths`, which since PJ-207 §8 loads the OWN registry only, so a foreign path resolves to no owning library and is **skipped** (`search.rs:12956-12970`) — the child's changes are watched but deliberately not indexed anywhere. Routing them to the child's OWN DB would slot exactly there (`library_name_for_path` at `search.rs:13009` / `12860`).
- **State shape**: `SearchState` holds exactly one writer + one reader + one federated conn; there is no per-universe connection map. A side-by-side model needs a keyed pool (universe root → {writer, reader}) plus per-entry `db_ready`/init serialization, or reuse of the existing single-slot pattern per child. The generation-counter discipline (capture-before-init, check-before-publish: `search.rs:11537-11539`, `11635-11648`; thread variant `11889-11906`) is the proven template for keeping any new pool switch-safe.
- **Existing precedent for concurrent access to the same child file**: `federated_conn` holds the child ATTACHed read-only while `federation_prewarm` opens it read-write and takes the writer lock — coordinated only by `busy_timeout=30000` (`search.rs:11393-11397`). WAL makes this legal.

---

## (4) Risks of two Constellation instances holding the same DB open

- **No single-instance guard exists.** Cargo.toml plugins: opener, updater, process only (`Cargo.toml:35,68-69`); no `tauri-plugin-single-instance`, no lock file. Two instances (or parent + child-as-active-elsewhere) is a live scenario the code already half-anticipates.
- **SQLite level — mostly safe**: WAL permits cross-process many-readers/one-writer; all writers set `busy_timeout` (5s main at `search.rs:4620`, 30s walk at `11996`, 30s prewarm at `11397`). Beyond the timeout, writes fail with "database is locked" — noted as swallowed in at least one place (`search.rs:12421` comment).
- **Schema rebuild vs open handle**: the version-gate's rename-aside FAILS with a Windows sharing violation while another instance holds the file; handled — abort, don't stamp, retry next boot (`search.rs:11588-11596`). But instance B keeps serving the OLD schema, and after A's successful rebuild B is left holding a renamed-aside file (its `db_path` re-resolve on next ensure gets the NEW file, mid-session queries keep the old inode).
- **`is_cuniverse_open_elsewhere` is best-effort and admits false results both ways** — `BEGIN EXCLUSIVE` probe, documented as "sufficient for single-user-single-process pattern" with WAL false-positives (`migrate.rs:182-200`). Two instances federating each other's universes can race migrate/backup/restore: the restore path (`fs::copy(&backup_path, cu_db_path)`, `migrate.rs:155`) copies OVER a live-open WAL database — a corruption vector SQLite explicitly warns about (file replaced under an open connection).
- **Per-process state divergence, no cross-process invalidation**: `db_ready`, `federation_generation`, `LIBRARIES_CACHE`, `link_types` REGISTRY, `watcher_suppress` are all process-local. Concretely: instance A's rename-cascade writes are suppressed from A's watcher (`watcher.rs:115-119` → `watcher_suppress.rs:35-36`) but **not** from B's — B sees them as external edits and re-indexes/reloads (echo loops the suppress map exists to prevent, now cross-process).
- **Duplicate maintenance**: both instances run the same backfills/reconcile on boot. The one-shot stamps (`schema_versions`) are read then acted on — two processes can both pass the gate and run the same backfill concurrently; SQLite serializes the writes but the work duplicates and long writer holds starve the other instance's UI-facing writes for `busy_timeout` then error.
- **Reconcile/repair vs live edits**: A's boot reconcile snapshots rows then deletes/re-adopts on its own thread guarded only by A's OWN generation counter (`reconcile.rs:270-288`) — B's concurrent writes are invisible to that guard; the invariant "no write lands in a universe other than the one active when the run started" is enforced per-process only.
- **Non-SQLite stores**: `earned.jsonl` ledger appends, `session.json`, `libraries.json`, `universes.json` registry (`load_registry_for_update` guards corrupt-read but not concurrent write-write; `atomic_write` at `universe.rs:334` protects torn files, not lost updates) — two instances can interleave read-modify-write and silently drop each other's changes.
- **Design assumption stated in-code**: search.db is NOT a disposable index — it is the only home of earned link data (`search.rs:11543-11566`), which is why every one of these cross-process hazards is app-killer-class rather than "just rebuild it."

==========================================================================================
## MAPS :: write-surface
==========================================================================================

All verification complete. Here is the deliverable.

---

# PJ-235/PJ-276 Architect input — RE-VERIFICATION + operation-class inventory
**Verified against the current tree at `7921e593` (+28 uncommitted session-log lines only). Every claim below was read from source this session.**

## 0. CRITICAL FINDING ABOUT THE BANKED REGISTER

`docs/migrations/PJ-235-federation-boundary/ARCHITECT-INPUT-federated-write-sites.md` is **EMPTY — 0 bytes on disk AND committed empty in 7921e593** (`git show --stat` shows `| 0` insertions; `git show 7921e593:<path>` returns nothing). The "verified register of 22 federated write sites" was never written into the file; only the session log's one-line summary of it survives (`lab/reports/SESSION-LOG-2026-08-11.md:315-317`: "22 federated write sites (21 live, 1 latent), including `constellation_search_reindex` trusting a frontend-supplied library name at ~20 call sites"). There is nothing to re-verify item-by-item, so the register below is a **fresh, from-source enumeration** of the current tree. It lands on 22 write sites (21 live + 1 latent) plus the reindex-trust issue — consistent with the banked claim's shape, but the original list's exact contents are unrecoverable.

## 1. Verified architectural ground facts (the model everything below sits on)

| Fact | Evidence |
|---|---|
| One `search.db` per universe, at `<universe>/.constellation/search.db`; `db_path` always resolves the ACTIVE universe | `src-tauri/src/search.rs:1465-1468` |
| The earned-data ledger (`earned.jsonl`, `note-history.jsonl`) lives in the same `.constellation` dir, keyed by **cid_cn** (portable — cid travels in frontmatter) | `src-tauri/src/link_life.rs:55-60, 79-85, 680`; `review.rs:682-691` |
| Every cUniverse's `search.db` is already ATTACHed to the active connection **READ-ONLY** (`?mode=ro`, aliases `cu0..cu24`, cap 25) for federated queries | `src-tauri/src/federation/attach.rs:1-37, 125-160` |
| `find_universe_root` (walk up to `.constellation/universe.json`) is the existing path→owning-universe resolver primitive | `attach.rs:105-118` |
| Write-scope guard = own-prefix (longest-root) PLUS foreign-root set; foreign set is **best-effort and library-rooted** (a linked universe with no root-registered library is invisible to it) | `src-tauri/src/libraries.rs:295-345` (limits stated at :320-323) |
| Federated notes are NOT indexed into the active DB: boot repair refuses foreign libraries (`foreign` outcome), watcher flush skips foreign paths | `src/lib/libraries/store.ts:4081-4088`; `search.rs:12959-12970` |
| Boot reconcile skips rows under no owned root, counts them `foreign_rows` — misfiled rows are unrecoverable by it | `libraries.rs:286-287` |
| The watcher DOES watch federated libraries (started over the federated `libraries` store) | `+layout.svelte:2945` over the store filled by `resolve_universe_libraries` (`store.ts:3925-3935`) |
| `constellation_search_reindex` → `reindex_single_note` → `index_note` stamps a **caller-supplied, unverified** `library_name` into the ACTIVE DB | `search.rs:12251-12287, 12682-12718` |
| `reindexNote` has **22 frontend call sites** in 5 files, feeding `tab.libraryName` or a federated-store resolution | grep count; files: `store.ts`, `+layout.svelte`, `NoteEditor.svelte`, `ExpressionForge.svelte`, `SenseMakingCanvas.svelte` |
| DBs run WAL (WAL "is a file property — inherited") → SQLite transactions across two ATTACHed WAL databases are NOT atomic as a set — a cross-universe both-sides commit needs a journaled two-phase (the `mig108.rs` engine is the in-repo precedent) | `search.rs:1470-1484`; `mig108.rs` (journaled snapshot-first engine) |

## 2. The federated write-site register, re-derived from the CURRENT tree

**Class A — disk writers authorized by the FEDERATED resolver (`validate_path_in_any_library`, libraries.rs:730 = `load_all_libraries`):**

| # | Command | Evidence |
|---|---|---|
| 1 | `write_note` (edit/save) | libraries.rs:981 |
| 2 | `create_note` (folder auth) | libraries.rs:1359 |
| 3 | `create_folder` | libraries.rs:1470 |
| 4 | `rename_item` (both ends) | libraries.rs:1674, 1687, 1737 |
| 5 | `resolve_structural_conflict` | libraries.rs:2613 |
| 6 | `move_item` — SOURCE only (dest own-guarded :2673) | libraries.rs:2660-2673 |
| 7 | `get_daily_note_path` (creates the note) | libraries.rs:6617, 6666 |
| 8 | `quick_capture` | libraries.rs:6675, 6704 |
| 9 | `update_links_on_rename` (cascade RMWs referrer bodies; walk is foreign-bounded :6827, but boundary collapses to empty on unreadable registry :6829-6832) | libraries.rs:6777 |
| 10 | `save_clipboard_image` | libraries.rs:8119 |
| 11 | `delete_path` (guard refuses only a foreign LIBRARY under the path, :8482) | libraries.rs:8467 |
| 12 | `toggle_task` | tasks.rs:458 |
| 13 | `write_canvas` | canvas.rs:92 |
| 14 | `create_canvas` | canvas.rs:107 |
| 15 | `import_execute` | importers.rs:228 |
| 16 | `import_with_canonical` | importers.rs:818 |

**Class B — disk writers with a DIFFERENT (or NO) authorization boundary:**

| # | Command | Boundary | Evidence |
|---|---|---|---|
| 17 | `create_base` | `load_libraries_pub` = `load_all_libraries` = federated | bases.rs:325-336; libraries.rs:223-225 |
| 18 | `update_base_columns` | `validate_base_path` = active-universe-**prefix only** → a cUniverse nested under the active root PASSES (the exact hole `require_own_library_in` closes with the foreign set) | lens/query.rs:368; bases.rs:18-33 |
| 19 | `update_base_order` | same | lens/query.rs:410 |
| 20 | `ensure_cid_cn_cmd` | **NO library validation at all**; fires on every first open of any note in the main window → opening a federated note WRITES `cid_cn:` into the linked universe's file | canonical.rs:1478-1483; store.ts:3327-3329 |
| 21 | `sources_set_manual` (and its mirror `content_type_set_manual`, not individually read) | no path-scope check found in the command body; `rewrite_note_sources_on_disk` writes the note's frontmatter | sources/mod.rs:772-817 |
| — | `write_conflict_sidecar` | no validation (writes a `.txt` sidecar beside any path) — inert to indexing by design | libraries.rs:1025 |

**Class C — the latent site:** `move_to_trash` — no longer a `#[tauri::command]`; sole caller is Rust-side Template-Studio undo; still authorizes federated (libraries.rs:8385-8388). **That is 21 live + 1 latent = 22.**

**Class D — DB-attribution trust (the ~20-call-site issue, confirmed at 22):** `constellation_search_reindex` trusts the frontend `library_name` (search.rs:12251); `index_note` stamps it into `note_meta` unverified (search.rs:12718). Callers pass `tab.libraryName` (save tail, store.ts:~1871) or longest-prefix resolutions over the **federated** `libraryStats`/`libraries` stores (`addLinkToNote` store.ts:1505-1508; `addTagToNote` closed branch `+layout.svelte:7090-7091`).

**Class E — DB/JSON-only earned-data writers with no path scope (bookkeeping-attribution class):** `constellation_link_traverse` (search.rs:9735 — no guard; path-keyed UPDATE in the ACTIVE DB), `constellation_link_set_confidence` / `_archive` / `_unarchive` / `_dormant` / `_decay` (search.rs, same family); `mark_reviewed` / `snooze_note` / `dismiss_note` write `review-pulse.json` in the active (or caller-pinned) universe with **no existence check on the note** (review.rs:741-793 — a foreign path's review mark lands in the ACTIVE universe's pulse file); `set_review_priority` is the one that now REFUSES when no `note_meta` row exists (review.rs:703-715, the 2026-08-08 §11 fix — its error message itself names "a linked-universe note (permanently unindexed since §8)").

**Own-guarded already (for contrast):** `move_item` DEST (`require_own_library`, libraries.rs:2673); `update_note_property` (bases.rs:390); `apply_shape` → `set_note_shape`/`clear`/`revert` (shape.rs:168); all 8 write/reindex tails moved in 7921e593 (create_note:1436-1451, move tail:2740-2741, cascade tail:2637-2645, canonical.rs startup/canonicalize via `load_libraries` + foreign boundary:1532-1556, `reindex_changed_paths`:12959-12970); the Move/template destination pickers (`list_universe_folders` libraries.rs:2848-2879, `walk_exclusions` boundary intact).

## 3. Operation-class inventory for FULL cross-universe operations

Per operation: **command(s) → disk writes → DB writes → what must change to operate on a cUniverse note with bookkeeping in THAT universe's DB.** "Route the tail" below always means: resolve the owning UNIVERSE (via a `find_universe_root`-class resolver, attach.rs:105), open/obtain a WRITE handle on `<that universe>/.constellation/search.db`, and run the existing tail against it with THAT universe's library registry for attribution.

### 3.1 Edit / save
- **Commands:** `write_note` (libraries.rs:966-1013) via frontend `writeNote` (store.ts:2921-2926); save tail fires `reindexNote(savedPath, tab.libraryName)` + `reembedNote` (store.ts:~1871-1875) → `constellation_search_reindex` (search.rs:12251) + `constellation_embed_notes`.
- **Disk:** the `.md`, atomic + journaled via `gate_write` (write_gate).
- **DB:** `note_meta`/`notes_fts`/`note_links` rebuild, incoming/sky/term maintenance (`index_note`, search.rs:12718+); `note_embeddings`.
- **Cross-universe delta:** disk half already crosses (federated auth). The tail is the whole gap: `constellation_search_reindex` must stop trusting the frontend name and resolve universe+library in Rust (fixes PJ-275 in the same stroke), then write into the OWNING universe's DB — which is currently attached `mode=ro` (attach.rs:13). Embeddings likewise (`note_embeddings` lives per-universe). The `foreign_rows` skip in boot reconcile (libraries.rs:287) stops being the correct posture once foreign rows are never created.

### 3.2 Create note (incl. from template)
- **Commands:** `create_note` (libraries.rs:1358-1455); destination pickers are OWN-only (`list_universe_folders` libraries.rs:2858; Ctrl+N picker per 2026-08-10 ruling).
- **Disk:** `gate_create_exclusive` with minted identity (`cid_cn`, `created`, `title`) :1378-1412.
- **DB:** synchronous `reindex_single_note` under `owning_own_library_name` — **foreign → reindex SKIPPED, logged** (:1436-1451).
- **Cross-universe delta:** offer cUniverse destinations in the pickers behind an explicit affordance (the planet icon exists); route the reindex to the owning universe's DB. Identity minting is universe-neutral (cid is self-contained). MIG-099 collision check (:1414-1435) reads the ACTIVE index — must consult the owning universe's index instead.

### 3.3 Create folder
- **Command:** `create_folder` (libraries.rs:1467-1485). **Disk:** `fs::create_dir`. **DB: none.**
- **Cross-universe delta:** none beyond authorization policy — it already passes federated auth and has no bookkeeping. Only the pickers gate it.

### 3.4 Rename
- **Commands:** `rename_item` (libraries.rs:1672-…; .md branch `gate_rmw_rename` rewrites frontmatter `title:` + aliases, then renames); detached DB tail `migrate_note_db_paths` (:1518-1670) + reindex + 'rename' alias row; frontend `update_links_on_rename` (libraries.rs:6763+) rewrites `[[OldTitle]]` in referrer bodies; folder branch: `retarget_registered_libraries` fixes the OWN registry, `guard_no_foreign_library_under` refuses foreign libraries under the path (:1696).
- **Disk:** the renamed file (+title rewrite); every referrer `.md` the cascade touches.
- **DB:** path migration across `note_meta`/`note_links`/`note_aliases`/`note_embeddings`/`review_schedule`(+sky via trigger), alias row, reindex — all in the ACTIVE DB.
- **Cross-universe delta (renaming a note that lives in a cUniverse):** the fs rename already succeeds; ALL bookkeeping goes stale in the foreign DB (its rows sit at the dead path; the active DB has no rows to migrate). Must: run `migrate_note_db_paths` + alias + reindex against the OWNING universe's DB; run the wikilink cascade in **both** universes (referrers on both sides may cite the note — today's walk is deliberately foreign-bounded, libraries.rs:6817-6833, and the index-seek reads only the active DB); `retarget_registered_libraries` must be able to write the OTHER universe's `libraries.json` (PJ-207 §15's refusal exists precisely because it can't today).

### 3.5 Move (the PJ-276 door — the migration's headline)
- **Commands:** `move_item` (libraries.rs:2658-2727) + `move_item_db_tail` (:2737-2795). Dest refused foreign (`require_own_library` :2673); source guarded only by `guard_no_foreign_library_under` (:2698) — plain-note source from a linked universe still passes (stated at :2668-2672, PJ-270).
- **Disk:** `gate_rename`; `libraries.json` retarget for moved library dirs (:2709-2711).
- **DB:** `migrate_note_db_paths` per descendant + reindex under OWN libs (:2740-2789).
- **Cross-universe delta (the full door):** (a) replace the dest refusal with an explicit-intent path (the guard stays for SILENT crossings); (b) the tail becomes an **export/import**: read the source universe's rows (`note_meta` earned columns, `note_links` weight/confidence/traversal/last_traversed/status, `review_schedule`, `note_aliases`, `note_embeddings`) and the cid-keyed `earned.jsonl`/`note-history.jsonl` entries (link_life.rs — cid makes them portable), insert into the destination universe's DB/ledger, purge the source; (c) WAL forbids one atomic cross-DB transaction → journaled two-phase with resume (mig108.rs engine precedent); (d) watcher suppression must cover both roots; (e) wikilink cascade both sides (source universe's referrers now point at a note that left).

### 3.6 Delete
- **Command:** `delete_path` (libraries.rs:8460-8568), three modes; descendants snapshotted pre-delete; `reindex_delete_note` per path (+ archive with `DeleteReason`, search.rs:12293+); `retarget_registered_libraries(.., None)` on dirs; foreign-library-under refusal :8482.
- **Disk:** the file/dir (or move into `<trash_root>/.trash` — **ONE trash at the active root**, MIG-108).
- **DB:** row purge + delete-archive rows in the ACTIVE DB.
- **Cross-universe delta:** a foreign plain note passes today: file deleted, foreign DB keeps its rows forever (active-DB purge matches nothing). Must: route the purge + delete-archive to the OWNING universe's DB, and route `trash` mode to **that universe's** `.trash` (today's frontend-supplied `trash_root` would drag the note's remains into the active universe — the exact "unlinking takes the note with it" failure inverted).

### 3.7 Tag add
- **Commands:** open note → `saveTabContent` → `write_note` (props into the model, compose, gate); closed note → `readNote` + `writeNote(..., 'add_tag')` + `reindexNote(path, lib.name)` with `lib` resolved from the federated store (`+layout.svelte:7053-7102`, closed branch :7084-7091). Batch tag = the same per note (:10386).
- **Disk:** frontmatter `tags:` rewrite. **DB:** reindex (tags land in `note_meta`/FTS).
- **Cross-universe delta:** disk write crosses already; the closed-branch `reindexNote` would file the foreign note into the ACTIVE DB under a foreign library name (Class-D trust) — route it. Open-branch tail same as 3.1.

### 3.8 Typed-link connect
- **Commands:** `addLinkToNote` (store.ts:1495-1560) — link is born as frontmatter `<type>: ["[[target]]"]` text ONLY; `index_note` derives the `note_links` row (hypothesis confidence, weight 1.0) — the helper never writes `note_links` (stated :1476-1493). Earned-side commands are Class E (traverse/confidence/archive, search.rs:9735+).
- **Disk:** source note's frontmatter. **DB:** derived `note_links` row + target's `incoming_count` at reindex; earned mutations DB-only.
- **Cross-universe delta:** three sub-cases. (a) Foreign SOURCE: disk write passes; the `note_links` row must be born in the SOURCE's universe DB (route the reindex). (b) Cross-universe TARGET (own note → foreign note): the row lives in the source universe, but `target_name` resolution and the target's `incoming_count` live in the TARGET's universe — needs a cross-universe link representation (the attach layer already gives read access for resolution; the incoming-count bump needs a write on the target's DB or a query-time federated count). (c) Earned data (`constellation_link_traverse` etc.): today matches 0 rows for foreign paths (foreign unindexed) — earned life silently dies; must route to the universe that owns the `note_links` row, and the durable ledger append (`link_life::append`, cid-keyed) must go to THAT universe's `.constellation`.

### 3.9 Property edit
- **Commands:** Properties panel → `saveTabContent`/`commitAndSave` (PropertyEditor.svelte:636, 1004-1052) → `write_note`; Base cell edit → `update_note_property` (bases.rs:372-422) — **own-guarded, refuses foreign outright** (:390-392); lens store.ts:145.
- **Disk:** frontmatter. **DB:** reindex (`note_meta.properties_json` etc.).
- **Cross-universe delta:** panel path = 3.1's delta. `update_note_property` needs its own-guard widened to universe-routing (replace `owning_own_library_name` with owning-universe resolution + that-DB reindex).

### 3.10 Task toggle
- **Command:** `toggle_task` (tasks.rs:452-552) — **federated-authorized** (:458), gate_rmw checkbox rewrite; reindex own-only (:533) → foreign toggle writes the file, index of BOTH universes silently stale.
- **Cross-universe delta:** route the reindex; nothing else — the operation is otherwise self-contained.

### 3.11 Canvas
- **Commands:** `create_canvas` (canvas.rs:101-130), `write_canvas` (:73-98), both federated-authorized, `atomic_write`; `list_canvases` walks disk (:47-57). **DB: none — canvases are unindexed.**
- **Cross-universe delta:** essentially none in Rust — it already crosses with correct (nonexistent) bookkeeping. Only picker/policy surfaces decide whether that is a *deliberate* door.

### 3.12 Import
- **Commands:** `import_execute` (importers.rs:219-248), `import_with_canonical` (:810+) — federated-authorized targets; write via `fs::copy`/`gate_write`; **no reindex in the command** (frontend follows up; boot repair covers own libraries).
- **Cross-universe delta:** importing into a foreign library passes today and the files are never indexed anywhere (foreign refused by the repair runner). Must route post-import indexing to the target's universe DB; `file_kinds.json` registry is read from the ACTIVE `.constellation` (:828-830) — should read the target universe's.

### 3.13 Base create
- **Commands:** `create_base` (bases.rs:318-366) — federated-authorized (Class B), `fs::write` (not atomic, noted in passing); `update_base_columns`/`update_base_order` (lens/query.rs:363-417) — active-root-prefix auth. **DB: none** (`.base` is a YAML file; execution reads the attach layer, lens/query.rs:88-98).
- **Cross-universe delta:** lens EXECUTION is already federation-aware read-side. A base living in a cUniverse needs consistent authorization (universe-routed instead of the current prefix/federated mix) — and note the results it shows for foreign notes are empty today because foreign rows aren't indexed (the PJ-224 adjacency).

### 3.14 Quick capture
- **Command:** `quick_capture` (libraries.rs:6673-6711) — federated-authorized, `gate_create_exclusive`, **no reindex in the command**.
- **Cross-universe delta:** if settings ever point the inbox at a cUniverse library, creation succeeds and the note is indexed NOWHERE. Route: index into the owning universe's DB at create (mirror 3.2).

### 3.15 Daily note
- **Command:** `get_daily_note_path` (libraries.rs:6614-6670) — federated-authorized, creates via `gate_create_exclusive` (`daily_note` surface), **no cid_cn minted and no reindex** (identity arrives via `ensure_cid_cn_cmd` at first open — which, note, is itself unguarded, Class B #20).
- **Cross-universe delta:** same as 3.14; plus the daily-note settings are per-app pointing into the active universe — a cross-universe daily note needs the owning universe's settings honored.

### 3.16 Review actions (adjacent, required for "full operations")
- **Commands:** `mark_reviewed`/`snooze_note`/`dismiss_note` (review.rs:740-800+) — write `review-pulse.json` + schedule row in the active/pinned universe with no note-existence check; `set_review_priority` (:649-721) refuses unindexed paths and appends the cid-keyed earned ledger (:682-691).
- **Cross-universe delta:** the `universe_root: Option<String>` param (:744, added for the switch-race) is already the routing mechanism — extend it (or Rust-side resolution) so a foreign note's review state lands in ITS universe's `review-pulse.json`, schedule row, and earned ledger.

## 4. The shared infrastructure the door needs (all operations converge on these five changes)

1. **A universe-routing resolver** — path → owning universe root → that universe's `.constellation` (registry, `search.db`, ledger, `review-pulse.json`, `.trash`, templates, `file_kinds.json`). Primitive exists: `find_universe_root` (attach.rs:105-118). Must fix the guard's stated limit (libraries.rs:320-323): resolve by cUniverse ROOT, not by registered-library roots.
2. **A write handle per foreign universe DB** — today: one writer on the active DB (search.rs:1465), cUniverses attached `mode=ro` (attach.rs:13). Either a per-operation `Connection` on the foreign `search.db` (schema-version gate applies — the gate renames aside, never deletes) or a scoped rw re-attach. **WAL ⇒ no atomic cross-DB transaction ⇒ journaled two-phase with resume**, the mig108.rs pattern.
3. **Rust-side attribution** — kill Class D: `constellation_search_reindex` resolves universe+library itself instead of trusting the frontend name (also closes PJ-275); the 22 `reindexNote` call sites then need no per-site change.
4. **The earned-data transfer payload** (for move/delete-with-history): `note_meta` earned columns, `note_links` (weight/confidence/traversal_count/last_traversed/status), `review_schedule`, `note_aliases`, `note_embeddings`, delete-archive rows, plus the cid-keyed `earned.jsonl`/`note-history.jsonl` lines — cid_cn travels in frontmatter, so ledger entries are replayable by cid on the destination side.
5. **Guards become doors, not holes** — `require_own_library` (move dest), the picker filters, `walk_exclusions`, and the §8 foreign-skips stay as the SILENT-crossing seal; each gains one explicit-intent bypass parameter carried from a deliberate UI gesture. The unguarded writers (`ensure_cid_cn_cmd` canonical.rs:1478, `sources_set_manual` sources/mod.rs:817, `write_conflict_sidecar` libraries.rs:1025, `update_base_columns/order` prefix-auth lens/query.rs:368/410) must be brought onto the same boundary first — they are today's silent crossings that 7921e593 did not reach.

**Prior rulings this direction supersedes (for the Boss's explicit repeal list):** MIG-065 §J ("a write must never be authorized onto a read-only cUniverse" — bases.rs:380-382, libraries.rs:267-269 doc-comments cite it), the "reads but never writes" text inside `require_own_library_in`'s user-facing refusal strings (libraries.rs:330-343), the PJ-207 §8 permanent-foreign-skip posture (store.ts:4081-4088, search.rs:12959-12965), and `set_review_priority`'s "linked-universe note (permanently unindexed)" framing (review.rs:705-707). Interacts with the pending PJ-224 search-federation ruling (the attach layer already exists read-side, attach.rs:1-37).

==========================================================================================
## MAPS :: identity-links
==========================================================================================

# IDENTITY & LINKS Across Universes — Findings

## Architecture ground truth (evidence-first)

Each universe owns one `search.db` at `<universe root>/.constellation/search.db` (`src-tauri/src/federation/attach.rs:156`). Federation ATTACHes each cUniverse's `search.db` **read-only** (`mode=ro`) as `cu0..cu24`, cap 25, to the active universe's connection (`attach.rs:125-233`). Federated reads are **pure concatenation per schema — "no cross-universe merge or remapping"** (Boss principle Q3 Option A, `src-tauri/src/cache.rs:1276-1281`, `cache.rs:1992`).

---

## (1) cid_cn: unique per-universe, NOT globally

- **Generation** (`src-tauri/src/canonical.rs:33-93`): `YYYYMMDDTHHMMSSZ_KIND_XXXX` — timestamp-second + random 4-hex suffix (65,536 values). Collision check is **only against the target directory listing** (`canonical.rs:64-67`, 10 retries then +1s fallback). `inject_cid_library` passes `target_dir=None` — **no collision check at all** (`canonical.rs:946`).
- **Uniqueness enforcement is per-DB**: partial UNIQUE index `idx_note_meta_cid_cn ON note_meta(cid_cn) WHERE cid_cn != ''` (`src-tauri/src/search.rs:3841-3844`), in each universe's own `search.db`. There is **no global registry and no cross-universe check anywhere**. cid_cn travels with the file's frontmatter and is preserved, never re-minted, on merge (`canonical.rs:310-331`).
- **What collides if two universes hold the same cid**:
  - **Today, federated READS: nothing.** No federated query joins by cid — federated payloads are keyed by path/name (`cache.rs:1249-1252`, `1294-1299`). The cid-keyed joins (`note_links.target_cid_cn → note_meta.cid_cn`, `src-tauri/src/review.rs:149,288`) run on `main` only.
  - **The real collision surface is file movement between universes**: a note copied/moved from universe Y into active universe X carries its `cid_cn:`. If X already holds that cid, `index_note`'s upsert fails with SQLITE_CONSTRAINT_UNIQUE — `ON CONFLICT(path)` does not absorb a cid_cn violation (`search.rs:3821`, `3979`, `4276`) — **the second note becomes invisible to search/index** until repaired. The MIG-003 backfill's dup rule re-mints the later-modified file's cid (`search.rs:3495-3499`), which **severs every earned row keyed to the old cid** (links, review, history — `canonical.rs:311-315` states cid overwrite "severs every earned row").

## (2) Cross-universe link [[B]] (A in X, B in cUniverse Y)

- **READ resolution**: frontend passes the **federated** library list — `$libraries` = `resolve_universe_libraries` (`src/lib/libraries/store.ts:3925-3934`; `src-tauri/src/universe.rs:1513-1517` → `resolve_libraries_recursive` `universe.rs:602-659`: own `libraries.json` first, then children recursive, dedup by path). `resolve_wikilink_cross_library` (`src-tauri/src/libraries.rs:3040-3048`): current library first, then list order; OWN libraries = indexed seek on `main` (`libraries.rs:3135-3171`); FEDERATED libraries = `federated_title_candidates` querying `cuN.note_meta.name_lower` + `cuN.note_aliases` (`libraries.rs:3505-3556`), falling back to a filesystem walk of the linked tree for resolution when federation isn't ready (`libraries.rs:3173-3202`). So B **is** found (the 2026-07-05 ONE-universe ruling holds for READ).
- **The edge lives in the AUTHOR's universe's DB.** `note_links` row (schema `search.rs:5277-5293`) is written when the SOURCE note A is indexed — into **X's** `search.db`. `target_cid_cn` is stamped only from unqualified (`main`) `note_meta` (`search.rs:8419-8427`), so for a target in Y it stays **NULL**; the only tie to B is the string `target_name`. **Y's DB never learns of the edge** (cu ATTACH is read-only; the indexer indexes own libraries).
- **Backlinks**: the boot payload concatenates `note_links` across `main` + all `cuN` (`cache.rs:1272-1336` `read_links_in_schema`), and `getBacklinks` matches by folded target NAME + aliases (`store.ts:5641-5663`). So while X is active with Y attached, B's backlinks panel shows A — **by name-match across the concatenated payload**. But: Y's own write-time incoming aggregates (`note_meta.incoming_count`, `note_links_incoming_*` triggers `search.rs:1783-1784`) never count X's edge; and **open Y standalone (federation is one-directional — Y has no pointer back to X) and the backlink from A does not exist anywhere Y can see.**

## (3) Rename cascade with referrers in a LINKED universe

- **Their files: NOT rewritten.** Both cascade branches enforce the federation boundary: the index-seek path drops any candidate under a foreign root (`libraries.rs:6932-6942` — "a seek that followed such a row would rewrite a note INSIDE a linked universe"); the walk path refuses to cross foreign roots (`libraries.rs:7071-7096`, rationale at 7074-7078: "rewriting them on OUR rename would be editing a knowledge base this universe does not own"); the freshness net does the same (`libraries.rs:7199-7203`).
- **Their DB rows: NOT touched.** Post-rewrite reindex covers only own-universe referrers into the active DB (`libraries.rs:7020-7054`, own-only `load_libraries` at 7037); cu attachments are read-only anyway.
- **Net effect: cross-universe inbound links break silently on rename.** Y's referrer files still say `[[OldName]]`, Y's `note_links` rows still carry the old `target_name`, and nothing records the rename for them (no cid-keyed fallback — the edge's `target_cid_cn` is NULL per (2)). No warning is surfaced.
- Note: the seek source `cascade_candidates_via_index` reads `main.note_links.target_base` only (`libraries.rs:7392-7425`, PJ-249). The active DB still holds **13 residual linked-universe rows** (adoption stopped by PJ-207 §8; purge blocked on a PJ-224 ruling — `libraries.rs:6933-6935`); the 6940 guard is what keeps the seek from following them into foreign files.

## (4) Same-name collisions across universes

- **Silent shadowing, own-first**: resolution tries current library, then the federated list in order (`libraries.rs:3244-3288`); own libraries precede cUniverse libraries (`universe.rs:611-652`); within a library, byte-shortest path wins (`libraries.rs:3230,3250,3280`). An own-universe note **shadows** a same-named cUniverse note; among cUniverses, declaration order wins. No ambiguity is surfaced.
- **`library:note` prefix** disambiguates by library NAME only, and returns on the FIRST name match — a miss inside it returns `None` without trying a second same-named library (`libraries.rs:3223-3239`), so a duplicate library name across universes makes the later one unreachable by prefix.
- **Backlinks are name-keyed**, so a same-named note in a linked universe **attracts the other note's backlinks** in the federated payload (both match the same folded name — `store.ts:5655-5663`); `target_cid_cn` stamping always resolves to the own-universe note (`search.rs:8419-8427`). Sky edges are name-keyed too (`sky_links(source_path, target_name, link_type)`, `search.rs:5405-5412`; federated sky read is per-schema concatenation, MIG-061, `cache.rs:1563-1580`), so the federated sky graph has the same name-collision wiring.
- **Mitigations that exist**: the create/rename title-collision check spans federated universes via the index (`resolve_title_collision` `libraries.rs:3058-3066` + `federated_title_candidates` PJ-207 §15) — the user is warned at authoring time; and the Quick Switcher's `allNotes` deliberately spans all schemas (`cache.rs:1245-1249`, MIG-093 §A, ONE-universe ruling).

**Summary of the identity/link contract today**: identity (`cid_cn`) is universe-scoped with per-DB uniqueness and no global authority; cross-universe edges are name-strings held solely in the author's DB, readable only while that author's universe is active; renames do not propagate across the federation boundary (by design, and with silent breakage as the cost); and every cross-universe surface — resolution, backlinks, sky — degrades to name-matching where identity would be needed.

==========================================================================================
## MAPS :: earned-data
==========================================================================================

All evidence gathered. Composing the spec now.

# EARNED-DATA TRANSFER PAYLOAD — search.db census and cross-universe transfer spec

Method: exhaustive sweep of every production `CREATE TABLE` reachable from `search::init_db` (`src-tauri/src/search.rs:4645–6160`) plus module-owned `ensure_*` schema functions it calls. Every claim carries file:line. Test-fixture CREATEs (cache.rs:1598, collections.rs:191, review.rs:1777, etc.) excluded after verification.

## 1. Full table census (30 tables) — classification

| Table | Created at | Class |
|---|---|---|
| `note_meta` | search.rs:4718 | Recomputable-from-md/fs **except 3 earned columns** (§2.2) |
| `note_body` | search.rs:4743 | Recomputable (body shadow of the .md) |
| `tag_counts` | search.rs:4754 | Recomputable (backfill: tag_counts.rs:465) |
| `review_schedule` | search.rs:4767 | Derived due-list **except 4 earned columns** (§2.3) — earned cols mirrored in `review-pulse.json` |
| `note_embeddings` | search.rs:4988 | Cache, expensive (re-embed from body; path PK) |
| `term_vocab` | search.rs:5027 | Recomputable (rebuilt from body_text) |
| `index_search_history` | search.rs:5099 | **EARNED** (user search behavior) §2.7 |
| `notes_fts`, `notes_vocab` | search.rs:5123, 5174 | Recomputable (FTS rebuild, search.rs:6148-6155) |
| `note_links` | search.rs:5277 | Structure recomputable from wikilinks; **6 earned columns** (§2.1) |
| `sky_nodes`, `sky_links` | search.rs:5371, 5405 | Recomputable (triggers + sky_backfill.rs) |
| `link_stats_cache` | search.rs:5424 | Cache ("droppable + rebuildable", search.rs:5420-5423) |
| `note_aliases` | search.rs:5469 | Mixed: `source='frontmatter'` recomputable; **`'rename'`/`'import'` rows EARNED** (§2.6) |
| `sight_v3_layout` / `_cursor` / `_graph_version` / `_density_grid` | search.rs:6040-6105 | Cache (wiped wholesale on version bump, search.rs:6135-6140) |
| `sight_v6_layout` | sight_v6.rs:106 | Cache (invalidation triggers delete rows on any note_meta update, sight_v6.rs:203-217) |
| `note_summaries` | nsc/mod.rs:518 | Cache (content_hash-invalidated) |
| `sources_suggestions` | sources/mod.rs:168 | Cache (classifier output; regenerable) |
| `note_state_history` | cece/history.rs:71 | **EARNED** §2.4 — "NOT recomputable from the files" stated verbatim at search.rs:4832 |
| `shape_history` | shape.rs:68 | **EARNED** §2.5 |
| `review_backfill_cursor`, `note_body_backfill_cursor`, `sky_backfill_cursor`, `links_outgoing_backfill_cursor` | review_backfill.rs:152, note_body_backfill.rs:156, sky_backfill.rs:166, links_backfill.rs:414 | Progress state — never transfer (id=1 CHECK singleton rows) |
| `schema_versions` | search.rs:4679 | Infrastructure — never transfer |

Note: the prompt said "embeddings.rs" — no such file; `note_embeddings` is created in search.rs:4988 (writer path elsewhere; reconcile/test refs only).

## 2. The earned payload, table by table

### 2.1 `note_links` — earned columns (search.rs:5277-5293)
- **Earned**: `traversal_count`, `last_traversed`, `confidence`, `created`, `status` (+ `weight`, but see below). Codified by the single-source preserve rule `link_row_is_preserved` (search.rs:450-489): a row is earned iff `traversal_count > 0 || weight != 1.0 || status != 'active' || confidence != 'hypothesis'`, and structural (parent/TOC) edges are never earned.
- **Recomputable**: existence, `source_path`/`source_name`, `target_name`, `link_type`, `annotation`, `target_name_lower` (virtual), `target_base`, `library_name` — all re-parsed from `[[type::target|annotation]]`.
- **`weight` is a cache of a function**: `earned_link_weight(n) = 1 + ln(1+n)` — link_life_restore.rs:8-11 rules it "DERIVED, never restored". Transfer `traversal_count`, recompute weight at destination.
- **Writers**: traverse bump search.rs:9786-9800 (traversal_count/last_traversed/weight/status/confidence); archive `status='archived', weight=0.0` search.rs:10959; confidence promotion search.rs:10878, 10904, 10912.
- **PK shape**: `id INTEGER PK AUTOINCREMENT` (meaningless across DBs); logical key `UNIQUE(source_path, target_name, link_type)` (search.rs:5292) — source is path-keyed (absolute path), target is *name*-keyed. Nullable `target_path` (search.rs:5281), nullable `source_cid_cn`/`target_cid_cn` (search.rs:4091-4101).
- **Transfer must**: rewrite `source_path` (and `target_path` where set — note the rename cascade only updates `source_path`, libraries.rs:1598; `target_path` is re-stamped by triggers search.rs:5682/5785 and mig003_step4.rs:213); carry the 5 earned scalars per logical key; prefer cid keys (`source_cid_cn` + `target_cid_cn`), falling back to (source, target_name, link_type); on ambiguity apply the link_life_restore rule (link_life_restore.rs:24-31): a name-keyed record matching several rows is SKIPPED, never distributed.
- **`created` exists ONLY here** — "created is not in the ledger at all" (search.rs:470). Ledger replay cannot restore it; the DB row itself must be carried.

### 2.2 `note_meta` — 3 earned columns (table otherwise recomputable)
- `review_priority INTEGER` nullable — user override, NULL = use-computed (search.rs:1685-1703); written by `set_review_priority` (review.rs:649-721), deliberately omitted from index_note's ON CONFLICT column list (search.rs:8170-8184) so reindex preserves it. Mirrored to `earned.jsonl` as a cid-keyed `priority` record, `-1` = cleared (link_life.rs:120-129, review.rs:682-692).
- `created_at INTEGER` — stamped from fs::metadata with `modified` fallback (search.rs:4710-4716). Semi-earned: a file copy across universes/OSes can lose the true creation timestamp, so the transfer should carry it.
- `content_changed_at INTEGER` — observed last-real-body-change signal, no mtime fallback (search.rs:1654-1660). Not reconstructible; drives Mode-2 staleness.
- NOT earned despite appearances: `sources`/`content_type` are extracted from frontmatter on every index (search.rs:8033-8048) and the accept flow writes disk FIRST ("canonical store", sources/mod.rs:814-830); `cid_cn` is written into the .md frontmatter by MIG-003 (search.rs:4826-4837) — it is the portable identity, unique per universe via partial index `WHERE cid_cn != ''` (search.rs:4866-4871).
- **PK**: `path` (absolute). **Transfer must**: rewrite path; carry the 3 columns keyed by cid_cn (fallback path); insert parent rows before FK children (§3).

### 2.3 `review_schedule` — earned columns on a derived table (search.rs:4767-4787)
- Derived: `reason`, `due_days`, `is_checkpoint`, `stratum` (recomputed from tags/links/content). **Earned**: `last_reviewed`, `interval`, `snoozed_until`, and the `reason='dismissed'` terminal state (review.rs:741-820 — mark/snooze/dismiss actions; review.rs:1264, 1275).
- **PK**: `path`. **On-disk companion**: `.constellation/review-pulse.json` is the system of record (review.rs:11, 21-30: `last_reviewed`/`snoozed`/`intervals` maps + `dismissed` list, all **path-keyed**); DB rows are the §B-2 cache. review.rs:829-838 states these are "EARNED and live nowhere else on disk."
- **Transfer must**: move review-pulse.json AND rewrite every path key inside it; rewrite/carry the 4 earned columns per path (the rename cascade precedent: libraries.rs:1606-1609 with destination pre-delete).

### 2.4 `note_state_history` (cece/history.rs:71-83)
- Append-only epistemic timeline (`sources`/`content_type`/`properties_json` transitions), trigger-fed (cece/history.rs:112-141). Explicitly "NOT recomputable from the files" (search.rs:4832).
- **PK**: `history_id AUTOINCREMENT` (drop on transfer, reassign); key column `note_path` with **FK → note_meta(path) ON DELETE CASCADE** (cece/history.rs:76). Ordering key is the row ordinal (`hid`), never `captured_at` — timestamps collide massively (link_life.rs:34-35).
- **On-disk companion**: `note-history.jsonl` (Stream B — never folds, never compacts; link_life.rs:16-32), cid-keyed with `hid`.
- **Transfer must**: carry all rows per note in `history_id` order, rewrite `note_path`, re-key `history_id` monotonically preserving order; or replay from note-history.jsonl by cid. Presence-of-key in `changes_json` is meaningless — test values (cece/history.rs:55-63).

### 2.5 `shape_history` (shape.rs:66-84)
- User shape transitions + the `undone` undo-cursor. Current shape lives in frontmatter (recomputable); the trail and its undo state live only here. **No on-disk companion.**
- **PK**: `id AUTOINCREMENT`; key column `path`, no FK; table is lazily created (may not exist — the cascade tolerates "no such table", libraries.rs:1630-1650).
- **Transfer must**: carry rows in `id` order per path, rewrite `path`, preserve `undone` flags and relative order.

### 2.6 `note_aliases` — partially earned (search.rs:5437-5480)
- `source='frontmatter'` rows: recomputable (repopulated per save). `source='rename'` rows: the prior display name stamped at rename so old wikilinks still resolve — exists in no .md file. `source='import'` rows: Obsidian import provenance, likewise DB-only. index_note deliberately keeps non-frontmatter rows across reindex (search.rs:5806, 8366-8367).
- **PK**: composite `(path, alias_lower)`; **Transfer must**: rewrite `path`, carry `'rename'`/`'import'` rows verbatim (`alias_lower`, `added_at`, `source`). Losing them silently breaks inbound-link resolution for renamed notes. **Not covered by the earned ledger.**

### 2.7 `index_search_history` (search.rs:5099-5107)
- Per-universe user search behavior: `query` (UNIQUE), `last_used`, `use_count`, 200-row FIFO cap. Not note-keyed — no rewrite needed; carry rows verbatim. **Not covered by the earned ledger.**

## 3. Keys, refs, and the rewrite rules a transfer must obey

- **Paths in search.db are ABSOLUTE.** MIG-108's engine remaps prefixes Rust-side with normalized matching — "never SQL replace()/LIKE" (mig108.rs:1122-1136). The authoritative path-column census is mig108.rs:1061-1075 (13 pairs): note_meta.path, note_links.source_path, note_aliases.path, note_embeddings.path, note_body.path, review_schedule.path, note_summaries.path, sources_suggestions.note_path, sight_v3_layout.note_path, note_state_history.note_path, shape_history.path, sky_nodes.path, sky_links.source_path (+ SWEEP list mig108.rs:989-1003). Of these, only the §2 tables carry earned data — the rest can be rebuilt instead of rewritten.
- **The proven per-note cascade** is `libraries::migrate_note_db_paths` (libraries.rs:1532-1670): 11 tables, destination pre-delete per path-PK table (PK collision otherwise aborts the UPDATE silently), `PRAGMA defer_foreign_keys = ON` inside one transaction. Reuse it — do not reinvent.
- **FK enforcement is ON** on every rusqlite connection (libraries.rs:1547-1556): `note_summaries`, `note_state_history`, `sources_suggestions` reference `note_meta(path)`. Transfer order: note_meta parents first, or defer FKs to commit.
- **Merge collision rule (precedent mig108.rs:1049-1058)**: rows freshly re-adopted at the destination are "default weights, no history — recomputable junk by construction"; the earned rows win. On `UNIQUE(source_path, target_name, link_type)` conflict, keep the earned side; fold counts by MAX, decisions by latest (the ledger's fold algebra, link_life.rs:19).
- **cid-first identity**: `note_meta.cid_cn` travels inside the .md frontmatter, so it is the stable cross-universe key; use it to re-join earned rows to relocated files, with universe-relative forward-slashed NFC path as fallback (the ledger's own key contract, link_life.rs:37-40).

## 4. The on-disk companions the payload must also move (`.constellation/` beside search.db)

| File | Covers | Keyed by | Evidence |
|---|---|---|---|
| `earned.jsonl` + `earned.snapshot.jsonl` | link `n`/confidence/status decisions, note review-priority | `cid>TARGET_CID` or `cid>~target-name` | link_life.rs:63-74, 146-152; writes ON (`EARNED_LEDGER_WRITE = true`, link_life.rs:680) |
| `note-history.jsonl` | note_state_history mirror | cid + hid | link_life.rs:16-32, 154-162 |
| `review-pulse.json` | review_schedule earned cols | **absolute path** — must be rewritten | review.rs:11, 21-30 |

Ledger replay (`link_life_restore.rs`, runs every boot, unstamped by design — lines 73-80) restores link decisions + priorities into a rebuilt index — but it does NOT cover: `note_links.created` (search.rs:470), `note_aliases` rename/import rows, `shape_history`, `index_search_history`, `note_meta.created_at`/`content_changed_at`. **A cross-universe transfer therefore cannot be ledger-replay alone; it must carry the §2 DB rows too.**

## 5. Minimal payload summary

Per note (keyed cid_cn, path-fallback): note_meta {review_priority, created_at, content_changed_at}; review_schedule {last_reviewed, interval, snoozed_until, dismissed}; note_state_history rows (ordered); shape_history rows (ordered, with undone); note_aliases rows where source IN ('rename','import'). Per link (keyed source_cid+target_cid, name-fallback with skip-on-ambiguity): {traversal_count, last_traversed, confidence, created, status} — recompute weight. Per universe: index_search_history rows; the three ledger files + review-pulse.json (paths rewritten). Optional to avoid recompute cost: note_embeddings (path-rewritten). Everything else in search.db is rebuildable and should be rebuilt, not transferred.

Key files: `E:\مشاريع كلاود\Constellation\src-tauri\src\search.rs`, `src-tauri\src\review.rs`, `src-tauri\src\cece\history.rs`, `src-tauri\src\shape.rs`, `src-tauri\src\link_life.rs`, `src-tauri\src\link_life_restore.rs`, `src-tauri\src\libraries.rs` (migrate_note_db_paths), `src-tauri\src\mig108.rs` (path-rewrite precedent), `src-tauri\src\sources\mod.rs`, `src-tauri\src\nsc\mod.rs`, `src-tauri\src\sight_v6.rs`.

==========================================================================================
## MAPS :: frontend-model
==========================================================================================

All evidence gathered. Final findings:

# Frontend own-vs-federated model — Constellation

## Data spine (context for everything below)

`$libraries` and `$libraryStats` are **federated lists**: `loadLibraries()` invokes `resolve_universe_libraries` (own + cUniverse libs flattened, fallback `list_libraries`) — `src/lib/libraries/store.ts:3925-3935`; `loadAllStats()` → `get_all_library_stats` — `store.ts:3938-3941`. Own-vs-federated is **not a field on any library or note object**; it is derived only in the main window, in `src/routes/+layout.svelte`, from two pieces of state built at boot: `childUniverses = bundle.child_universes` and `childUniverseLibPaths` (Map cuPath → Set of normalized lib paths) — `+layout.svelte:2795-2800` (boot bundle) and `2820-2834` (fallback via `read_child_universe_libraries`). Everything else is predicate functions over that map. (`DigestPane.svelte:44-51` documents the reason: `LibraryInfo` has no `universe_id`, so identifying federated libs requires this side-map.)

## (1) How a federated note's tab differs from an own note's today: IT DOESN'T

- **`OpenTab` has no own/federated/readOnly field** — `store.ts:98-118` (`id, path, content, libraryName, libraryPath, name, libraryColor, highlightTerm, history, historyIndex, cursorPos, scrollTop, pinned, reloadVersion`). A cUniverse note's tab is structurally identical.
- `tab.libraryName` is resolved by `deriveLibraryForPath` — longest-prefix match over `get(libraries)`, the **federated** list (`store.ts:3209-3222`, used at `3300-3301`) — so a federated tab carries the cUniverse library's name (e.g. "Architecture") exactly like an own library's name.
- **No read-only enforcement in the editor.** `NoteEditor`'s `readOnly` prop defaults to `false` (`NoteEditor.svelte:82`); the only mounts that pass `readOnly={true}` are the Index preview (`+layout.svelte:8662`) and the second screen (`ssReadOnly = true`, `SecondScreenPage.svelte:208`, mounts `1253`, `1419`). The two main-window mounts — split pane `+layout.svelte:9208` and single pane `9393` — pass **no** `readOnly`. `FileTree.svelte` contains zero federated/readOnly logic (grep: no matches).
- **Typing in a cUniverse note already attempts (and completes) saves.** `handleSave` gates only on `readOnly`, path-match, and cascade/reseed (`NoteEditor.svelte:284-287`); it then runs the full pipeline: `editBody → saveNoteSession(standardSaveEnv({origin:'editor_save'}))` → durable `writeNote` → `onSaved: broadcastNoteSaved + reindexNote(savedPath, tab.libraryName) + reembedNote + CECE` (`NoteEditor.svelte:314-339`). Same for teardown flush (`handleFlush`, `360-390`), stage promote (`handlePromote` → `stage_promote` save, `207-274`), nav-flush (`navFlushEnv`, `store.ts:647-664`), retry (`store.ts:520-531`), session-close flush of dirty tabs (`store.ts:3593-3595`), and restore-reindex (`store.ts:3677`) — **none checks federated-ness**. So a federated note edited in the main window is written to the other universe's disk and reindexed under its federated library name via `constellation_search_reindex` (`store.ts:4111-4114`).

## (2) Every frontend site that branches on own-vs-federated

**Predicates (all in `+layout.svelte`; the ONLY module with the distinction):**
- `isChildUniverseLib(libPath)` — `1602-1608`
- `getChildUniverseLibs(cuPath)` — `1610-1614`
- `ownLibraries` ($derived, excludes cU libs + universe_notes) — `1616`
- `ownUniverseLibraries` ($derived over `$libraries`, Boss finding 2026-08-10) — `1617-1623`
- `isFederatedTreePath(p)` (path-prefix version) — `5104-5113`
- `writableSelection` / `writableNoteSelection` — `5115-5116`

**Write-boundary branch sites (exclusion):**
1. Batch bar (MIG-091 §C): `batchMove` `5126-5132`, `batchDelete` `5134-5137`, `batchTag` `5139-5142`; batch confirm loops `handleMoveConfirm` `6979`, `handleDeleteConfirm` `7208`; buttons disabled on `writableSelection.length === 0` — `8441`, `8443` (comment `8435-8436`: "federated members excluded").
2. Move picker + new-note-from-template destination picker: `buildUniverseFolderEntries` filters `$libraryStats.filter(l => !isChildUniverseLib(l.path))` — `6921` (PJ-235 comment `6911-6920`; Rust `list_universe_folders` narrowed the same way; the planet-icon case "gone from this builder by construction" `6925-6927`).
3. Ctrl+N-class create pickers: `LibraryPicker items={ownUniverseLibraries}` — `10272`; `LibraryPicker.svelte:7-17` documents that it used to read the federated store directly and created notes inside the linked universe.
4. Importer targets: `ImporterModal libraries={...}` filtered to paths under the universe root — `10197-10203` (MIG-108 Slice 5).
5. Workspace-bases sidebar: federated cU bases are open-only, **no context menu** — `8264-8287` (comment `8264-8266`: "READ-ONLY... would violate the read-only guarantee").

**Display/grouping branch sites (no write semantics):**
- Sidebar sections: cUniverse groups `8332-8387` (planet icon `8346`) vs `ownLibraries` `8390-8420`; Five Acts federated groups `fiveActsByCu` `8206-8227` (open-only via `constellation:open-note`).
- `revealInTree` expands the containing cUniverse first — `6832-6840`.
- `bookmarkLocation` breadcrumb prefixes the cUniverse name — `6528-6549`.
- `wiwTitle` names the cUniverse for multi-path selections — `1288-1298`; sidebar width `calcContentWidth` includes cU names — `341`.
- Federation warnings badge/popup — `10505-10541`; `src/lib/federation/store.ts`.
- Expand-all walks cU libs — `5033-5034`, `5068`.
- Other components (display-only): `OrgChart.svelte` `isCUniverse` nodes `62/159/1160-1217`; `DashboardView.svelte` `110/211`; `LibrarySwitcher.svelte` Child-Universes section `119-135` + `addChildUniverse` `29-35` (**note**: its "Own Libraries" list at `108` actually renders the full federated `$libraryStats` — a labeling drift); `ConstellationMap.svelte:271` (`child_universe` label); `graphEngine.ts:370, 570`; `StyleSetter.svelte:182-186, 1512` (cuniverse styling); `LibraryIcon.svelte` `kind='cuniverse'`; `bases/store.ts:24-27` and `lens/store.ts:181` (`universe_name` grouping field).

**Sites that DO NOT branch (writable UI offered on federated content today):**
- Tree context menu: federated library trees wire the same `handleLibraryHeaderContextMenu` (`8355`) and `handleContextMenu` (`8372`) as own trees; `getContextMenuItems` offers **New note/folder/base on the federated library root** (`6573-6580`), **Rename/Move/Delete/New-*** on federated folders (`6581-6592`) and notes (`6593-6612`) — no federated gate. `handleCreateNote` resolves from federated `$libraries` (`7160-7167`).
- List-surface menus: `buildNoteActions` `allowMutate` defaults `true` (`6644-6670`) — Rename/Move/Delete offered for federated notes on all ~26 list surfaces.
- `handleOrgNodeMenuAction` (also the second screen's forward target) — no gate (`6749-6803`).
- `openMoveDialog` accepts a federated **source** (`6946-6950`); only destinations are own-only.
- Inline tree rename (`renamingPath`, `6585/6599`) → `handleRenameComplete` (`7257+`) — no federated gate on source.
- Typed-link writes: `addLinkToNote` / `linkMentionInNote` longest-prefix over federated `libraryStats` and write to the source note — `store.ts:1495-1523`, `1618`.

So the frontend's enforced boundary is only: pickers-don't-list + batch-bar-excludes; everything single-item relies on Rust (`move_item` refuses federated destinations per the PJ-235 comment `6915-6917`) or on nothing at all (plain editor saves go through).

## (3) UI state that must change for full operations

- **The boundary state to parameterize** is the derived-filter layer, not object state: `ownUniverseLibraries` (`1623`), `buildUniverseFolderEntries`'s `!isChildUniverseLib` filter (`6921`) plus its Rust twin `list_universe_folders`, `LibraryPicker items` (`10272`), `ImporterModal` filter (`10197-10203`), `writableSelection` (`5115`) and the batch-bar disabled predicates (`8441/8443`), and the workspace-bases "no context menu" carve-out (`8264-8287`).
- **Marking machinery for re-listed linked universes already exists**: `MoveDialog.svelte:27` still types `iconKind?: 'root' | 'library' | 'cuniverse'`, and `LibraryIcon.svelte:6,30` renders `kind='cuniverse'` as the planet/orbit mark in `#6366f1` (as used in the sidebar header, `+layout.svelte:8346`). PJ-235 removed the *producer* of `iconKind:'cuniverse'` in `buildUniverseFolderEntries` (`6925-6927`), not the renderer — re-listing means restoring that branch and grouping entries under the cUniverse name (the `bookmarkLocation` breadcrumb `6542-6543` and `bases/store.ts:26 universe_name` show the existing grouping conventions).
- **No per-tab state change is needed for the editor itself** — there is nothing to flip: `OpenTab` carries no flag and the editor already writes. What's missing for a *deliberate* model is the inverse: if any surface is to stay read-only, a tab-level or path-level predicate would have to be introduced, because today only `+layout.svelte` can answer "is this path federated" (the predicate is module-local; `store.ts` and all components have no access to `childUniverseLibPaths`).
- **The second screen needs nothing**: it has zero own/federated state (no `childUniverseLibPaths` — grep confirms no match in `SecondScreenPage.svelte`) and is uniformly read-only.

## (4) Second screen and cross-window sync vs federated notes

- The second screen is **read-only for every note, own or federated** — the constant `ssReadOnly = true` (`SecondScreenPage.svelte:205-208`, Boss ruling 2026-07-09) feeds all NoteEditor mounts (`1253`, `1419`) and TasksPanel (`1210`); plus `setDisplayOnlyWindow()` at script-init (`213`) makes every `openNoteTab` there preserve the crash-recovery net (`store.ts:3283`) and skip the cid_cn injection write (`store.ts:3322-3327`). Federated-ness is irrelevant to this window: read-only comes from the window, not from the note.
- It loads the **federated** library list and note universe: `loadLibraries()` (→ `resolve_universe_libraries`) at `SecondScreenPage.svelte:479`, then builds `allNotes` across all `$libraries` including cUniverse libs (`483-489`). Federated notes display, peek, and sky-companion there like own notes.
- The sync protocol is federated-agnostic: `ScreenNote` is `{path, name, libraryName, libraryPath, libraryColor}` (`secondScreen.ts:13-19`); no event carries an own/federated marker. A federated note saved in the main window fires `broadcastNoteSaved` (`NoteEditor.svelte:320`, `secondScreen.ts:156-158`); the SS refreshes via `onNoteSaved` (`secondScreen.ts:198-200`) identically for federated paths. Rename/move/delete following on the SS is path-based (`onNoteMutation` handlers, `SecondScreenPage.svelte:139-165`).
- SS write verbs are **forwarded to main** (Display-not-Domain): `showSSNoteMenu` forwards rename/move/delete/addTag/etc. via `requestNoteActionOnMain` (`SecondScreenPage.svelte:105-130`, `secondScreen.ts:114-122`); main dispatches through `handleOrgNodeMenuAction` (`+layout.svelte:3968-3971` → `6749`), which has **no federated gate** — so a federated note right-clicked on the second screen opens main's rename/move/delete dialogs exactly like an own note; the Move dialog then offers own-only destinations (PJ-235), but rename and delete proceed unguarded on the frontend.

==========================================================================================
## MAPS :: rulings
==========================================================================================

# Prior Rulings & Standing Assumptions Touching Federation — reconciliation input for the PJ-235/PJ-254 Architect doc

**The new Boss ruling (verbatim source):** `lab/reports/SESSION-LOG-2026-08-11.md:339-363` — "BOSS RULING (2026-08-12) — FULL CROSS-UNIVERSE OPERATIONS. The federation contract is REFRAMED." Quote: *"I want to be able to conduct full functions/operations between universes… If it is kept as-is today, just to (read) and not able to (write), then why bother to include other universes (as cUniverses) in the first place?"* — Eisa, 2026-08-12. The log itself states: the interim walls (commit `7921e593`, now HEAD) remain correct until each operation is made safe; the defect was the SILENT crossing with broken bookkeeping; PJ-276 (the move door, filed at the Boss's Part-B pass, log :330-335) is subsumed into the larger goal.

## The reconciliation table

| # | Ruling / assumption | Where stated (evidence) | Dated | Disposition vs. full cross-universe ops |
|---|---|---|---|---|
| 1 | **MIG-056 v1 is read-only** — "federated queries read across multiple universes' search.db files, but each universe still owns its own writes" (Citus-FDW cautionary rationale) | `docs/MIG-056-cross-universe-federation-ARCHITECT.md:111`; mechanism: `src-tauri/src/federation/attach.rs:4,13,217-229` (`?mode=ro`) | MIG-056 era (pre-2026-07) | **SUPERSEDED as product contract** (the session log names the read-only assumption explicitly, :347-351). BUT "each universe owns its own writes" plausibly **survives as the implementation shape**: write through the child's own identity/DB, never through the parent's ro-attached handle. Technical fact that stays regardless: `BEGIN IMMEDIATE` fails if any attached DB is read-only → `BEGIN DEFERRED` only (ARCHITECT.md:154). Architect must decide: rw-ATTACH vs. open-child-directly. |
| 2 | **Concept paper: federation is NOT a write path** — "NOT a copy/import — read in place, read-only… never writes to a child"; "no 'edit a federated note through the parent.' Editing requires switching to that universe as active"; acceptance item "never writes a child (File-Over-App)" | `docs/concept-papers/25-federation.md:12-14, 48, 54` | draft (bring-up era) | **SUPERSEDED** — the paper's §3/§10 must be amended by the migration. Note: the File-Over-App linkage is spurious — File-Over-App forbids *silent* modification, not cross-universe operations the user explicitly commands. |
| 3 | **MIG-065 §J — the federated-write blocker**: "WRITE-path validation must scope to the active universe's own libraries so an edit never lands on a read-only cUniverse file"; "an attribution resolver that feeds a write or a reindex MUST use `load_libraries`… NEVER `load_all_libraries`" | `src-tauri/src/libraries.rs:62-69` and `:266-269` | MIG-065 (~2026-07-2x); reasserted 2026-08-11 | **SUPERSEDED as contract** — the session log names MIG-065 §J as the thing superseded (:347-348). **Mechanism RETAINED** for the *silent/automatic* class: automatic writes (reindex tails, canonicalize, startup repair) must still never cross; only a deliberate, user-commanded door may. |
| 4 | **The interim guards (PJ-235/PJ-254, commit `7921e593` = HEAD)** — `require_own_library` "the write-scope rule, in ONE place"; refusal text ×2: "a LINKED universe, which Constellation reads but never writes… Open that universe and do it there"; same phrasing in canonicalize | `src-tauri/src/libraries.rs:274-345` (refusals :329-344); `src-tauri/src/canonical.rs:718-728` (repeated :1153, :1534) | 2026-08-11/12 | **COEXISTS until each op ships its door** — Boss ruling explicitly keeps the walls (:353-356). The refusal **wording** is already superseded ("reads but never writes" is no longer the contract); when the door exists the message must route the user to it, not to "open that universe." Known limits filed in-code (:320-323): foreign set is best-effort; holds foreign LIBRARY roots, not cUniverse roots. |
| 5 | **"It is ONE universe" (Boss ruling)** — every name resolver spans the whole universe including federated cUniverses via `resolve_universe_libraries`; never single-library scope | `docs/Constellation Orientation & Onboarding v3.96.md:1454`; memory `feedback_one_universe_resolution.md` | 2026-07-05 | **COEXISTS — REINFORCED.** The session log (:349-351) already reads the new ruling as this one's completion: "the read-only contract was the implementation's assumption, never his design." The 2026-08-10 corollary survives intact: *a universe-wide list is right for RESOLVING a name and wrong for CHOOSING where to write* (libraries.rs:289-291) — resolution stays federated; write-targeting stays deliberate. |
| 6 | **MIG-100 — write-authorization is registry MEMBERSHIP, not the active pointer** — a note in a registered-but-not-active universe is writable to its OWN file; "never a federated cUniverse file, which is read-only from here and writable only through its own universe's identity" | code: `src-tauri/src/libraries.rs:749-777` (`validate_path_in_any_library` fallback); narrative: orientation v3.96:1287-1291 | 2026-07-12 | **COEXISTS — and provides the authorization shape for the door.** "Writable only through its own universe's identity" is exactly the both-sides-bookkeeping model the new ruling demands. Its embedded "read-only from here" clause is superseded. **Hazard for the Architect:** registry-membership auth depends on `universes.json`, which is *proven wrong on the live machine* (PJ-233 — lists only كون عيسى while the app runs Eisa Universe; ledger v1.82:265). |
| 7 | **PJ-224 (PENDING Boss ruling) — the ordinary search box does NOT federate.** A plain word routes `SearchHub.triggerSearch` → `universalSearch` → `execute_universal_search(conn,…)` on the active connection alone; the federated path is reachable only via `constellation_search` advanced syntax | filed: `docs/Constellation Pending Jobs v1.68.md:33`; still gating PJ-207 §13: v1.82:7, :269 | filed ~2026-08-07 | **FRESH BOSS CONFIRMATION REQUIRED.** The new ruling's "full access" spirit implies the box should federate, but the ruling itself directs the Architect to surface PJ-224's interaction explicitly (log :361-363). Do NOT treat it as auto-resolved. PJ-207 §13 (the duplicate-row removal offer) follows whatever is ruled. |
| 8 | **PJ-227 — a linked universe's phantom rows are permanently exempt from dead-row removal** (9 live rows, post-§8) | ledger v1.82:269 | ~2026-08-07 | **NEEDS FRESH CONFIRMATION** — its premise ("we cannot touch a foreign universe's rows") dissolves under both-sides bookkeeping; the migration can likely reconcile those rows properly instead of exempting them forever. |
| 9 | **PJ-219 — the user-action write class awaits its design ruling**, incl. the federated-drift asymmetry: an external edit to a linked note is invisible to the parent's drift check (own-scope) AND to the child's until the child is opened | ledger v1.69:25; carried v1.82:269 | ~2026-08-05 | **COEXISTS / FOLD IN** — still Boss-blocked; cross-universe ops sharpen the asymmetry (a cross-universe move touches an index the drift check of neither side may be watching). The Architect should fold PJ-219 into the operation-class map. |
| 10 | **MIG-108 — "One Universe, One Location"**: every additional library lives UNDER the universe root; `ensure_under_active_root` at every registration flow; "nothing may reference content outside the root" | `CLAUDE.md:242, 252, 255`; enforcement `src-tauri/src/libraries.rs:798-824` (+`:3774`), `mig108.rs` | Boss-ruled 2026-07-29 | **NO CONFLICT — COEXISTS.** Verified what it actually constrains: **library registration/layout of the ACTIVE universe's OWN content**, nothing about operations. Proof: `add_child_universe` (`src-tauri/src/universe.rs:1453-1489`) accepts a child universe at ANY path (only checks validity + not-self) — cUniverse locations were never under One-Location, so `CLAUDE.md:255`'s "nothing may reference content outside the root" is library-registration scope, not federation scope. **Constraint the door inherits:** a cross-universe move must land the note UNDER the destination universe's root inside one of ITS own libraries (each universe individually keeps One-Location). Subtlety in-code (libraries.rs:312-318): a linked cUniverse may be *nested under* the active root, so prefix-match alone lies — the foreign-root second boundary must survive into the door. |
| 11 | **CLAUDE.md hierarchy — cUniverse framing**: "Enables *viewing* notes from multiple independent Universes in one window"; federation opt-in; zero cUniverses = complete valid setup | `CLAUDE.md:231-260` (cUniverse bullet ~:256) | current | **COEXISTS, doc amendment needed** — "viewing" is read-only-era wording. The four-level structure + opt-in federation are untouched by the new ruling. |
| 12 | **Storage section — search.db is the system of record for earned link data** (weight, traversal, confidence promotions, archived status, review rows); no disk layer exists yet | `CLAUDE.md:143-186` | corrected 2026-07-24 | **COEXISTS as a HARD CONSTRAINT.** This is why "both-sides bookkeeping" is non-trivial: a cross-universe move must transfer earned rows between two universes' search.dbs or the data is stranded (the exact defect the ruling names, log :353-356). Interacts with **PJ-262** (Living Link disk layer, M1): if the disk layer ships first, the transfer payload shrinks to index rows. Sequencing question for the Architect. |
| 13 | **Prior authorized write-to-child precedent — MIG-056 §C auto-migration**: "Writes to a search.db belonging to a different universe than the active one — an explicit deviation from Constellation's normal per-universe ownership model", Boss-locked (Architect §5.3) with four safeguards (lock-check, pre-backup, atomic-via-backup restore, permanent backup) | `src-tauri/src/federation/migrate.rs:1-25, 67-82`; summarized `docs/concept-papers/25-federation.md:19` | MIG-056 era | **COEXISTS — and is the design template.** "Never writes" was never absolute: the system already writes a child's search.db under Boss-locked safeguards. The door should generalize this safeguard pattern (lock-check against the child being open elsewhere is directly reusable for cross-universe move). |
| 14 | **PJ-276 — deliberate cross-universe move with full both-sides bookkeeping** | session log :330-335; ledger v1.82 carries the ruling | 2026-08-12 | **IS the new direction's first concrete requirement** — subsumed into the migration's headline; PJ-270's defect framing is subsumed into PJ-276's correctness case. |

## Discrepancy the Architect must know about

**The banked Architect input file is EMPTY.** `docs/migrations/PJ-235-federation-boundary/ARCHITECT-INPUT-federated-write-sites.md` exists but contains 0 bytes — confirmed by direct read AND by `git show 7921e593 --stat` (committed as 0 lines). The ledger v1.82:7 and session log :313-317 both describe it as "a verified enumeration of 22 federated write sites (21 live, 1 latent), including `constellation_search_reindex` trusting a frontend-supplied library name at ~20 call sites" — that enumeration does NOT exist on disk and must be re-derived (or recovered from the prior session's scratchpad) before the Architect phase can read it.

## Correction to the task brief

The brief said to grep "MIG-065 read-only rulings" — confirmed correct but for precision: MIG-065 is the Unified Constellation Base migration; the federation-relevant piece is specifically **MIG-065 §J** (write-path validation scoping, libraries.rs:66), which is what the session log names as superseded. The federation system itself is **MIG-056** (engine) + MIG-061 (CNS/Sky) + MIG-062 (federated sidebar groups — read-side, unaffected structurally).

## One-line summary for the doc

Nothing structural conflicts with full cross-universe operations: MIG-108 constrains layout (per-universe), not operations; MIG-100 and the MIG-056 §C migrator already contain the authorization and safeguard patterns the door needs; "It is ONE universe" is reinforced. What is superseded is exactly the read-only *contract* (MIG-056 v1 stance, 25-federation.md §3, MIG-065 §J, the interim guards' "reads but never writes" wording — the guards' mechanisms stay until each door ships). Three items need fresh Boss confirmation rather than silent repeal: PJ-224 (does the ordinary search box federate), PJ-227 (phantom-row exemption), PJ-219 (user-action write class design).

==========================================================================================
## OPTIONS :: A-route-to-owner
==========================================================================================

All load-bearing code sites verified. Here is the design.

# DESIGN OPTION A — OPERATION ROUTING TO THE OWNING UNIVERSE

**Concept (the horse):** a note's bookkeeping lives where the note lives. Any operation, invoked from any universe, executes its disk half wherever the file is and its DB/ledger half against the note's OWN universe's `search.db` and `.constellation/` — opened side-by-side, never through the active universe's identity. The active DB never holds a foreign row; unlinking a cUniverse loses nothing because nothing of the child's ever lived in the parent.

---

## 1. The design in concrete terms

### 1.1 New components (all Rust-side; ~4 new modules + surgery on 3 existing)

**A. `federation/router.rs` — the ownership resolver (`resolve_owner(app, path) -> Owner`).**
Returns `Owner::Active`, `Owner::Foreign { universe_root: PathBuf }`, or `Owner::Unknown` (fail-closed → refuse). Implementation: longest-prefix match over the set of known universe roots (active root + cUniverse roots from `resolve_universe_libraries`, universe.rs:1513), with `find_universe_root` (the parent-walk to `.constellation/universe.json`, attach.rs:105-118) as the authoritative fallback. This **fixes the guard's filed known-limit** (libraries.rs:320-323: today's foreign set holds foreign LIBRARY roots, not cUniverse roots, so a linked universe with no root-registered library is invisible). Library attribution within the owning universe uses `try_load_libraries_at(<owner>/.constellation/libraries.json)` (libraries.rs:90 — the `-at` pattern already exists) with longest-root-wins, i.e. `owning_own_library_name_in` fed the OWNER's registry instead of the active one's.

**B. `ForeignDbPool` — side-by-side write handles, one per foreign universe.**
Added to `SearchState` (search.rs:1341): `foreign_dbs: Mutex<HashMap<PathBuf, Arc<ForeignDbEntry>>>` where `ForeignDbEntry { writer: Mutex<Option<Connection>>, ready: AtomicBool, init_lock: Mutex<()> }` — the proven single-slot pattern (`db`/`db_ready`/`init_lock`) replicated per child. Lazy: an entry opens on the FIRST routed write to that universe, never at boot. Open sequence per entry:
1. Replay `schema_gate` (search.rs:1457 — already isolated and unit-testable) against `<child>/.constellation/search.version`. **Divergence from the active gate:** on mismatch the parent must NEVER rename-aside a foreign DB (that is destructive surgery on a database we don't own and whose rebuild we cannot finish — the 14 backfills are owner-only). Instead: run the MIG-056 §C safeguard pattern that already ships (`migrate.rs:68-180`: `is_cuniverse_open_elsewhere` lock-probe → `fs::copy` backup → `init_db_schema_only`), or refuse the operation with a visible error naming the child's schema state.
2. Open rw `Connection`, apply the standard pragmas, **register the FTS5 tokenizer** (connection-local, mandatory — MIG-056 §K.1 proved silent zero-row MATCH without it, search.rs:1558-1581).
3. Verify trigger completeness (see D below).
WAL makes rw-open-while-ro-attached legal — `federation_prewarm` already does exactly this (search.rs:11358-11474). The read-side attach layer (`cu0..cu24`, `mode=ro`) is **unchanged**; federated queries keep reading through it. Writes never go through the attached alias.
Pool lifecycle: cleared wholesale by `invalidate_search_state` on universe switch (same as `federated_conn`), guarded by the existing `federation_generation` capture-before/check-before-publish discipline (search.rs:11537-11539, 11635-11648).

**C. The guard becomes the router — `require_own_library` → `route_write(app, path, intent)`.**
`require_own_library` (libraries.rs:295-345) stops being a refusal and becomes the dispatch point: `intent: WriteIntent::UserCommanded | WriteIntent::Automatic`. `Automatic` (reindex tails, canonicalize, startup repair, watcher-driven maintenance the user didn't ask for) keeps today's own-scope refusal for anything it can't attribute — the MIG-065 §J *mechanism* survives for the silent class, exactly as the ruling requires. `UserCommanded` routes: returns `WriteTarget { owner, library_name, db_handle }`. The two refusal strings ("reads but never writes… Open that universe and do it there") are rewritten — reserved for `Owner::Unknown` only.

**D. Per-universe link vocabulary — the deepest cut in Option A.**
`link_types::REGISTRY` is a process-global `OnceLock<RwLock<LinkTypeRegistry>>` loaded ONLY from the active universe (link_types.rs:351, `load_active` called at search.rs:11606). Verified by grep: the PARSE/index path reads it globally at 30+ sites — `is_known_type` decides what is a typed link at search.rs:7244/7371, structural exclusion at 8018/8485/8561/9550, trigger-DDL generation at 5540/5637/5964, backfill fingerprints in links_backfill.rs/incoming_links_backfill.rs. **Indexing a foreign note with the ACTIVE registry silently mis-derives its `note_links`** (a child's custom type parses as a plain frontmatter key → the link doesn't exist; the parent's custom type parses as a link the child never defined). Required change: `LinkTypeRegistry` gains per-universe loading (`load_at(<child>/.constellation/link-types.json)`), and `index_note`'s callee chain takes an explicit `&LinkTypeRegistry` (defaulting to the global snapshot for the active path so the 30+ non-indexing read sites don't churn). Additionally, `InitScope::ForeignSchemaOnly` (search.rs:4571-4589) deliberately SKIPS registry-generated trigger DDL — so a child whose own boot hasn't regenerated triggers for the current schema may be missing derived-maintenance triggers. The pool's open step verifies the child's registry-DDL fingerprint and, if stale, generates the DDL **from the child's own registry** (a third scope: `ForeignFullWithRegistry(&child_registry)`) — this is safe precisely because the DDL source is now the child's vocabulary, which was the entire reason ForeignSchemaOnly exists.

**E. Rust-side attribution — kill Class D.**
`constellation_search_reindex` (search.rs:12251) drops trust in the frontend-supplied `library_name`; it resolves owner + library via the router. All 22 `reindexNote` frontend call sites then need zero per-site changes, and PJ-275 closes in the same stroke. `reindex_single_note` (search.rs:12682) gains a routed variant; note the core is ALREADY connection-parameterized — `index_note(conn, path, library, force)` at search.rs:12718 — so routing means handing it the pool's conn plus the owner's registry/library, not rewriting the indexer.

**F. Ambient-dir consumers gain a root parameter.**
Everything that writes `.constellation/` siblings must accept the owner root instead of resolving the ambient active dir: `link_life::append` (earned.jsonl / note-history.jsonl), review-pulse.json (already parameterized — `universe_root: Option<String>` at review.rs:744/779/806 via `resolve_constellation_dir`; extend to Rust-side resolution from the note path), `.trash` resolution in `delete_path`, `retarget_registered_libraries` (the child's `libraries.json`), and importers' `file_kinds.json` read (importers.rs:828-830).

**G. `federation/transfer.rs` — the earned-data move engine** (§2 Move below).

### 1.2 What stays untouched
One `search.db` per universe; the whole schema; the ro attach layer and every federated read path (cache.rs concatenation, lens, federated search); `SearchState.db`/`read_db` for the active universe; the boot sequence and all 14 backfills (owner-only, unchanged); the watcher fan-out (already watches child libraries — +layout.svelte:2944); the frontend save pipeline and all 22 `reindexNote` call sites; `migrate_note_db_paths` (conn-parameterized, libraries.rs:1532 — reused as-is against the pool conn); the mig108 journaled-engine pattern (reused, not rewritten); the read-only second screen.

### 1.3 Frontend changes (small, by design)
The boundary today is a derived-filter layer in `+layout.svelte`, not object state — so the door is opened by re-parameterizing filters, not by re-plumbing tabs: restore the `iconKind:'cuniverse'` branch in `buildUniverseFolderEntries` (+layout.svelte:6921-6927 — the RENDERER still exists: MoveDialog.svelte:27, LibraryIcon.svelte planet mark), widen `LibraryPicker items` behind an explicit affordance, re-enable `writableSelection` for federated members, and label federated destinations with the planet icon + universe name so the crossing is always a visible, deliberate gesture (The Constellation Way: propose and show, user decides). `OpenTab` needs no new field — the editor already writes; correctness moves to the Rust tail.

---

## 2. Each operation class under Option A

**Edit/save.** Disk half unchanged (`write_note` already crosses). Tail: routed reindex — pool conn + child registry + child library attribution; sky/term/incoming maintenance inside `reindex_single_note` follows the conn (all per-DB, gated by the CHILD's own backfill stamps). `reembedNote` routes identically (`note_embeddings` is per-universe). The boot-reconcile `foreign_rows` skip (libraries.rs:287) becomes an assertion: foreign rows in the active DB are now a bug, and the 13 residual linked-universe rows (PJ-227) get a one-shot migration to their owner's DB — the exemption dissolves.

**Create (note / quick capture / daily note / import).** Pickers re-list cUniverse destinations behind the planet-marked affordance. `create_note`'s reindex routes to the owner (replacing the skip at libraries.rs:1436-1451); the MIG-099 collision check consults the OWNER's index (via pool conn or the ro attach — read suffices). Quick capture / daily note / import gain the same routed post-create indexing they currently lack for foreign targets (today: created and indexed NOWHERE); importer reads the TARGET universe's `file_kinds.json`.

**Rename cascade — the fan-out operation.** fs rename + title rewrite already cross. DB tail: `migrate_note_db_paths` + alias `'rename'` row + reindex run against the OWNER's conn (all conn-parameterized already). The referrer cascade becomes multi-universe: referrers may exist in the note's own universe, the active universe, and any other linked universe. The seek generalizes: query `target_base` in each universe's `note_links` (own via `main`, others via the ro attach — read-only is enough for the SEEK), then for each referrer: fs rewrite (federated auth already permits it) + reindex routed to THAT referrer's owner. Today's foreign-boundary refusals (libraries.rs:6932-6942, 7071-7096) become the routing dispatch instead of a drop. This closes the silent cross-universe link-breakage documented in the identity map. `retarget_registered_libraries` writes the child's `libraries.json` when a foreign LIBRARY dir renames (PJ-207 §15's refusal retires).

**Move (cross-universe — PJ-276, the headline).** Journaled two-phase, mig108-engine pattern, because WAL forbids one atomic transaction across two DBs:
1. **Journal intent** (coordinator journal in the active universe's `.constellation`, breadcrumb marker in both roots; resumable from either side's next boot).
2. **Extract payload from source DB** per the earned-data census: `note_meta {review_priority, created_at, content_changed_at}`; `note_links` earned scalars ({traversal_count, last_traversed, confidence, created, status} — `created` exists ONLY in the row, search.rs:470, so ledger replay alone is insufficient); `review_schedule` earned cols; `note_state_history` rows in `history_id` order; `shape_history` rows with `undone`; `note_aliases` `'rename'`/`'import'` rows; optionally `note_embeddings` to skip recompute. Plus the cid-keyed `earned.jsonl`/`note-history.jsonl` lines and the path-keyed review-pulse.json entries.
3. **fs move** (gate_rename; watcher suppression stamped for BOTH roots).
4. **Insert into destination DB** (paths rewritten Rust-side, never SQL replace — mig108.rs:1122 rule; destination pre-delete per path-PK table — the `migrate_note_db_paths` discipline; merge collisions resolved earned-side-wins per `link_row_is_preserved`, search.rs:450-489; weight recomputed from traversal_count, never copied — link_life_restore.rs:8-11), then routed reindex so structure re-derives from the .md.
5. **Purge source DB rows**, rewrite source review-pulse.json, append a move record to both ledgers.
6. **Cascade both sides**: source-universe referrers now link a departed note (name resolution still works federated at read time — links are name-keyed — but attribution/sky shift; the cascade updates `target_base` bookkeeping per side).
Crash at any step: journal replay, keyed by cid_cn (portable — travels in frontmatter). Destination must land UNDER the destination universe's root inside one of ITS libraries (each universe individually keeps MIG-108 One-Location).

**Delete.** Route the row purge + `DeleteReason` archive rows to the owner's DB (the undo record must live with the universe that owns the note); route `trash` mode to the OWNER's `<root>/.trash` — never drag remains into the active universe. The foreign-LIBRARY-under-path refusal stays (deleting someone's whole library is a different act than deleting a note).

**Link.** Three sub-cases. (a) Foreign SOURCE: frontmatter write crosses already; the `note_links` row is born in the source's universe via routed reindex parsed with the SOURCE's vocabulary. **Design decision to surface to the Boss:** when editing a foreign note, the link-type palette must offer the SOURCE universe's types (a type unknown to the source's registry parses there as a plain key — the link silently doesn't exist). (b) Cross-universe TARGET (own → foreign): the row stays in the source universe (edge lives with its author — invariant preserved); `target_cid_cn` stamping extends to query the ro-attached `cuN.note_meta` (read suffices), which gives cross-universe edges cid identity for the first time and shrinks the rename-breakage class; the target's `incoming_count` should be computed federated at READ time rather than writing the target's DB for a source-side gesture — keeps writes single-owner. (c) Earned life (traverse/confidence/archive/dormant): route by the row's owner (resolved from `source_path`), UPDATE in that DB, ledger append to THAT universe's `earned.jsonl`. Today these silently match 0 rows for foreign paths; earned life stops dying.

**Review.** The routing mechanism half-exists: `universe_root: Option<String>` + `resolve_constellation_dir` (review.rs:744-746). Complete it: resolve Rust-side from the note path via the router; schedule row → owner's DB (pool), pulse entry → owner's review-pulse.json, priority → owner's `note_meta` + owner's earned ledger. `set_review_priority`'s "permanently unindexed linked-universe note" refusal (review.rs:705-707) dissolves — the existence check queries the owner's DB.

**Embeddings.** Write-side: routed (per-universe `note_embeddings`). A foreign note edited via the parent gets its embedding refreshed by the routed tail; wholesale backfills remain owner-run on the child's own boot. Read-side semantic search across universes is a PJ-224-adjacent question — the table is readable through the attach, but that ruling stays with the Boss.

---

## 3. Speed / effort / risk — honestly

**Speed (runtime).** Steady-state foreign write = own write + one map lookup + one registry-cache hit. First routed write per child pays the lazy pool open: schema-gate read + pragmas + tokenizer (~tens of ms warm); if the child needs the §C schema-only migrate, that one operation stalls for the backup+FTS-rebuild (seconds to minutes on a large child — must show progress, must be the visible exception not the rule). **Boot: zero change** — the pool is lazy, the attach layer is unchanged. Reads: zero change.

**Effort.** LARGE — honest sizing: this is the biggest `/migration` since MIG-108. Tractable half (patterns already exist in-repo): router (+`find_universe_root` promotion), pool (single-slot pattern ×N + generation discipline), guard→door conversion, routed reindex/Class-D kill, review/ledger root parameters. Expensive half: (1) the link-registry threading — `index_note`'s callee chain plus trigger-DDL generation from a passed registry, with 30+ global read sites to audit for which are index-path vs active-UI-path; (2) the multi-universe rename cascade; (3) the transfer engine with two-root journal + resume; (4) the frontend affordance pass (small) + i18n ×15 + help/manual. Rough phase count: 8–10 landable steps, each Boss-testable.

**Risk.** Highest-risk items, ranked: (1) **wrong-vocabulary indexing** (app-killer class — silent `note_links` mis-derivation; mitigated only by completing the registry threading before any foreign index write ships — this must be Phase 1, gated, not incremental); (2) **cross-DB move crash windows** (mitigated by the journal; mig108 engine is proven but was single-root — the two-root variant needs its own harness); (3) **child-DB schema/trigger drift** (mitigated by the §C safeguard pattern + trigger-fingerprint verification at pool open); (4) **two-instance write contention** (below). The interim guards (7921e593) stay up per operation until that operation's routed tail ships and is Boss-tested — the walls come down door by door, never all at once.

---

## 4. What breaks or degrades

- **Universe-folder portability: PRESERVED — this is Option A's strongest property.** Everything about a note lives in its own universe's folder; copy the folder, get everything; unlink a cUniverse, lose nothing. No cross-universe rows exist anywhere.
- **Two-instance safety: DEGRADES from edge-case to routine.** A child universe open as ACTIVE in instance B while instance A routes writes into it stops being exotic. SQLite level is safe (WAL + busy_timeout; prewarm precedent). NOT safe: B's per-process caches (boot payload, LIBRARIES_CACHE, sky, tabs) see A's `.md` writes via B's watcher (adopt path exists — PJ-070) but A's **DB-only** writes (earned data, review rows) are invisible to B until reconcile/restart — no cross-process invalidation exists, and `watcher_suppress` is per-process so A's writes echo into B as external edits. Honest posture: ship with the `is_cuniverse_open_elsewhere` lock-probe refusing routed writes while the child is open elsewhere (extend the probe from migrate-time to write-time), and file the single-instance plugin / cross-process signal as the follow-up. Do not pretend WAL alone solves this.
- **Boot time: unchanged.** No new boot work; the child's own boot still runs its own backfills, now also blessing any parent-written rows (fingerprint gates re-run per-DB — acceptable, must be documented).
- **PJ-262 (Living Link disk layer): synergy, with a sequencing choice.** If PJ-262 ships first, the move payload shrinks (earned link data becomes recomputable from LINK files that travel with the .md) and the "search.db is sole system of record" hazard shrinks with it. Option A does not depend on it — but building the transfer engine BEFORE PJ-262 means building payload-extraction code PJ-262 partially obsoletes. Recommend surfacing the ordering to the Boss: PJ-262-first shrinks Option A's riskiest module.
- **PJ-224 / PJ-219 / PJ-227:** PJ-224 (search-box federation) is read-side — untouched, still needs its own ruling. PJ-219's drift asymmetry SHARPENS (a routed write updates a child index that neither side's drift check watches from the other's seat) — fold into the design's watcher-routing step. PJ-227's exemption dissolves (one-shot migration of the 13 residual rows to their owners).

---

## 5. Invariants

**Preserved (and in three cases strengthened):**
1. One `search.db` per universe; each universe a self-contained, portable unit (File-over-App at universe scale).
2. **"Each universe owns its own writes" survives as the implementation shape** — writes execute through the OWNER's identity (its registry, vocabulary, ledger, trash), merely driven by the parent's process. MIG-100's "writable only through its own universe's identity" is exactly this.
3. **Active DB never holds foreign rows** — strengthened from "mostly true, 13 residuals" to enforced.
4. MIG-108 One-Location, per universe individually.
5. Silent crossings stay refused — the `WriteIntent::Automatic` refusal is MIG-065 §J's mechanism retained; only user-commanded gestures cross.
6. Write-Time Derivation — derived surfaces update in the same transaction, per-universe, via the child's own triggers.
7. Earned data never silently lost — journaled transfer, cid-keyed, resume-safe; `search.db`-as-system-of-record respected on both sides.
8. Read-side federation contract (attach, concatenation-no-merge, ONE-universe resolution) — untouched.

**Endangered (each needs an explicit defense in the Plan):**
1. **Link-vocabulary correctness** — parsing a foreign note with the wrong registry silently changes what a link IS. Threading must be complete and gated before any foreign index write ships; this is Option A's internal app-killer.
2. **Child trigger/schema completeness** — ForeignSchemaOnly's deliberate registry-DDL skip means a routed write can land in a child whose derived-maintenance triggers are stale → index↔disk divergence inside the CHILD. Defense: fingerprint check + child-registry DDL generation at pool open.
3. **Cross-process coherence** — per-process caches, suppress maps, and generation counters have no cross-instance analogue. Defense: write-time lock-probe refusal now; single-instance/signal follow-up filed.
4. **Schema-gate ownership** — the parent must never rename-aside or rebuild a child's DB; mismatch handling is §C-migrate-or-refuse, never destroy.
5. **The reversibility promise** — cross-universe move's crash windows are the one place a note's earned half can exist in zero or two places; the journal + cid replay is the entire defense and must be harness-proven (Reproduce-First: every crash window red→green) before the door opens.

**Files the build will touch (core):** `src-tauri/src/federation/router.rs` (new), `federation/transfer.rs` (new), `search.rs` (SearchState pool, routed reindex, InitScope third variant, registry threading), `link_types.rs` (per-universe load + passed snapshots), `libraries.rs` (guard→door, cascade fan-out, `-at` attribution, trash/registry routing), `review.rs` (Rust-side root resolution), `link_life.rs` (rooted appends), `canonical.rs` (`ensure_cid_cn_cmd` brought onto the boundary — it is today's unguarded writer), `bases.rs`/`lens/query.rs` (auth unification), `+layout.svelte` (affordance pass), plus the unguarded Class-B writers (`sources_set_manual`, `update_base_columns/order`) brought onto the router first — they are the silent crossings 7921e593 never reached.

==========================================================================================
## OPTIONS :: B-provenance-index
==========================================================================================

# OPTION B — FEDERATED INDEX WITH PROVENANCE
## Architect-phase design input for PJ-235/PJ-276 (full cross-universe operations)

**One-line concept (the horse):** the active universe's `search.db` becomes the single *view* of the whole federation — every row stamped with its origin — while each universe's own `search.db` remains the sole *authority* for its notes' earned life; every operation writes authority **home** first and refreshes the cache second.

All file:line evidence below is from `E:\مشاريع كلاود\Constellation\src-tauri\src\` unless stated; items marked **[verified this session]** were read directly, the rest are carried from the six verified maps.

---

## 1. The design in concrete terms

### 1.1 Core schema change (active DB, additive, in-place)

- **New dimension table** `origin_universes(origin_id INTEGER PRIMARY KEY, root_path TEXT UNIQUE NOT NULL, name TEXT, last_synced INTEGER, child_data_version INTEGER)`. Row `origin_id = 0` is reserved for OWN. No stable universe UUID exists anywhere today — `ChildUniverseInfo` is `{name, path, library_count}` only (**[verified]** `universe.rs:1546-1552`) — so identity is the normalized root path; a relocated child invalidates its cache rows, which is acceptable *because they are cache*.
- **New column** `origin_id INTEGER NOT NULL DEFAULT 0` on: `note_meta`, `note_body`, `note_links`, `note_aliases`, `sky_nodes`, `sky_links`, `tag_counts`. NOT on the earned-only tables (`review_schedule`, `note_state_history`, `shape_history`, `index_search_history`) — under this option those tables never hold federated rows (§1.3).
- **Delivered as a `schema_versions` module migration** (the in-place ledger pattern, **[verified]** `search.rs:4679-4698`, module rows manipulated throughout e.g. `search.rs:2373-2447`) — **NOT** via the `search.version` marker bump. The marker gate renames the whole DB aside (`search.rs:11573-11601`), and ledger replay does not cover `note_links.created`, rename/import aliases, `shape_history`, `index_search_history`, `created_at`/`content_changed_at` (earned-data map §4) — a marker bump would silently lose earned data. Additive ALTER + stamped module row avoids that entirely.
- **The cid uniqueness index MUST change.** Today: `CREATE UNIQUE INDEX idx_note_meta_cid_cn ON note_meta(cid_cn) WHERE cid_cn != ''` (**[verified]** `search.rs:3841-3844`). cid_cn is unique *per universe only* (identity-links map §1). The moment federated rows enter `note_meta`, a legitimate cross-universe cid collision (copied note, or plain birthday-collision on the 4-hex suffix) hits SQLITE_CONSTRAINT_UNIQUE and the second note goes invisible — the exact failure the identity map documents for file moves, now fired by *attaching a universe*. Becomes `UNIQUE(origin_id, cid_cn) WHERE cid_cn != ''`, and the PJ-207 §15 dependent-table partial-unique indexes get the same treatment. **Every cid-keyed join** (`note_links.target_cid_cn → note_meta.cid_cn`, `review.rs:149,288`) **gains an origin-match term** or collisions cross-wire universes.
- **FTS federation falls out for free, structurally**: `notes_fts` is external-content on `note_meta` (`content=note_meta, content_rowid=rowid`, **[verified]** `search.rs:5123-5129`) and is trigger-fed by `note_meta_ai/ad/au` (**[verified]** `search.rs:5146-5161`). Federated `note_meta` inserts flow into FTS with zero new code. Consequence: **the ordinary search box becomes federated by default** — PJ-224 collapses from an engine question to a one-WHERE-clause filter question. It must still go to the Boss as a ruling (rulings map #7), but under Option B either answer is cheap.

### 1.2 What is CACHE and what is AUTHORITY (the demotion contract)

| Data | Authority | Active-DB federated row |
|---|---|---|
| Note existence, name, body_text, tags, properties, frontmatter aliases | the `.md` file (unchanged) | full cache row, `origin_id = N` |
| Link existence/type/annotation (`[[type::target\|ann]]`) | the `.md` file | cache row in `note_links`, `origin_id = N` |
| Link earned scalars (traversal_count, last_traversed, confidence, status, created; weight = derived) | **home universe's search.db + its cid-keyed `earned.jsonl`** (CLAUDE.md storage section; earned-data map §2.1) | **read-only mirror values** on the cache row, refreshed by the sync contract — never written directly |
| `review_schedule` earned cols, `review-pulse.json`, `note_state_history`, `shape_history`, rename/import aliases, `index_search_history`, `review_priority` | home universe's search.db / `.constellation` files | **no federated rows at all** (v1); surfaces that need them read via the existing ro-ATTACH (`attach.rs:125-233`), which **stays** |
| `note_embeddings` | home universe's DB (embed once, stored home) | **not mirrored** — vectors are large; semantic search reads `cuN.note_embeddings` via ATTACH |

This split is the load-bearing safety property: **for every federated row in the active DB, loss = re-derivable**, either from the child's files (structure) or from the child's DB (mirrored scalars). Nothing in the active DB is ever the only copy of another universe's earned life. The dual-write problem therefore collapses to *authority-then-cache* for every routine operation — no cross-DB atomicity needed except for MOVE (§2.2).

### 1.3 Components that change

1. **`federation/home_write.rs` (new)** — the write-home primitive: given a foreign path → `find_universe_root` (by cU root, `attach.rs:105-118`, fixing the guard's stated library-root limit at `libraries.rs:320-323`) → open a **short-lived per-operation `Connection`** on `<child>/.constellation/search.db` → **schema handshake**: read the child's `search.version` marker; on mismatch REFUSE the operation with an actionable error ("open that universe once to upgrade it") — never run the marker gate or full init on a DB we don't own (the PJ-230/PJ-232 lesson, `migrate.rs:86-131`) → register the tokenizer (mandatory per connection or MATCH silently returns zero — MIG-056 §K.1) → run the existing tail against it → close. WAL + `busy_timeout` makes the concurrent-open case legal (precedent: `federation_prewarm`, `search.rs:11358-11474`).
2. **`link_types` de-globalization** — the process-global `REGISTRY` (`link_types.rs:351`) becomes a *parameter*: `snapshot_for(universe_root)` loading the child's `.constellation/link-types.json`, passed into any tail that runs on a home connection. This is the exact hazard `InitScope::ForeignSchemaOnly` exists to dodge (`search.rs:4571-4589`); Option B needs it solved properly, not dodged, because `index_note` on the home connection classifies links against a vocabulary.
3. **`constellation_search_reindex` re-plumbed (kills Class D / PJ-275)** — stops trusting the frontend `library_name` (`search.rs:12251, 12718`); resolves owning universe + library Rust-side; `origin_id = 0` → active DB as today; `origin_id = N` → home-write (authoritative index) **then** cache upsert into the active DB. The 22 frontend `reindexNote` call sites need zero per-site changes.
4. **The foreign skips flip into the indexer** — watcher flush (`search.rs:12956-12970`, the slot the db-model map identifies), boot-repair `foreign` refusal (`store.ts:4081-4088`), reconcile `foreign_rows` skip (`libraries.rs:286-287`): each becomes "index with provenance" instead of "skip". Reconcile becomes **origin-aware and availability-aware**: rows of an *unreachable* origin are left alone (never purged as dead just because the drive is unplugged); rows of a *reachable* origin reconcile against that child's tree.
5. **First-population backfill** — on federation attach, a background, resumable, after-paint walk of each child indexes it into the active DB with `origin_id = N` + a mirror pull of earned scalars via the ATTACH (Write-Time Derivation's sanctioned first-time-population clause).
6. **The sync/refresh contract** (cache freshness):
   - *File changes*: the watcher already watches federated libraries (`+layout.svelte:2944-2947`, `store.ts:3925-3932`) — flows straight into the flipped indexer. Near-real-time.
   - *Earned changes made by the child instance elsewhere*: `PRAGMA data_version` on the ro-ATTACHed child connection changes when any other connection commits — poll it (timer + window focus + before earned-sensitive reads); on change, one bounded refresh query per origin (`SELECT` earned cols from `cuN.note_links` → `UPDATE` mirrors), stamp `origin_universes.last_synced`.
   - *Boot*: per-origin freshness pass (mtime/content_hash diff against cache rows), background + resumable.
7. **Guards become the router, not the wall** — `require_own_library` (`libraries.rs:295-345`) stops refusing and starts *routing*: own → today's path; foreign-with-explicit-intent → home-write + cache; foreign-without-intent (automatic/silent paths: canonicalize sweeps, startup repair, `ensure_cid_cn_cmd`) → still refused until that path's door ships. The interim guards' mechanism survives as the silent-crossing seal, exactly as the Boss ruling requires; the refusal strings' "reads but never writes" wording is retired.
8. **Read-layer collapse (later phase, optional)** — the per-schema concatenation loops (`cache.rs:1272-1336`, `read_links_in_schema`; sky per-schema MIG-061) can shrink to single-DB queries grouped by `origin_id`. The ATTACH layer stays regardless (earned reads, embeddings, refresh source), so this is a simplification that can land last with zero coupling.
9. **`mig108.rs` path-rewrite engine** — must skip `origin_id != 0` rows (its 13-pair path-column census, `mig108.rs:1061-1075`, would otherwise mangle foreign absolute paths when the *parent* relocates).

### 1.4 Components that stay untouched

The editor/save pipeline and `OpenTab` (no read-only flag needed — frontend-model map §1); the second screen (window-level read-only, federation-irrelevant); the write gate / journal; the schema marker gate per-DB; per-universe `earned.jsonl`/`note-history.jsonl` ledgers and `link_life_restore` (with one new rule: **replay and `link_row_is_preserved` skip `origin_id != 0` rows** — a universe's ledger must never fold onto mirror rows); `ensure_search_db_ready`'s generation discipline (template for keeping the per-op home connections switch-safe); MIG-108 One-Location per universe (rulings map #10 — a cross-universe move must land under the destination's root, unchanged).

---

## 2. Every operation class under Option B

Common preamble for all: resolve owning universe (Rust-side, by cU root). `origin_id = 0` → exactly today's code path. Foreign → the pattern **file write → home authority → cache refresh**, with watcher suppression covering both roots.

**2.1 Edit/save.** Disk write already crosses (federated auth, `libraries.rs:981`). Tail: home-connection `index_note` (child's registry snapshot; child's own triggers maintain its sky/FTS/incoming) → cache upsert into active DB (`origin_id = N`, mirrored earned scalars carried over unchanged — reindex preserves them exactly as `index_note`'s preserved-traversal snapshot does today). Embeddings: re-embed → write home `note_embeddings`. Crash between home write and cache upsert = stale cache row, healed by watcher/freshness — benign by construction.

**2.2 Move (cross-universe — PJ-276, the headline).** The ONLY operation where both sides are authoritative, so the only one needing the journaled two-phase (mig108 engine precedent): (1) journal intent; (2) export the earned payload from source-home DB + cid-keyed ledger lines (exactly the earned-data map §5 payload: note_meta earned cols, note_links earned scalars keyed cid-first with skip-on-ambiguity, review_schedule earned cols + `review-pulse.json` entry, state/shape history ordered rows, rename/import aliases; recompute weight from traversal at dest); (3) `fs::rename`/copy the `.md`; (4) import into dest-home DB via the proven `migrate_note_db_paths`-shaped cascade (destination pre-delete per path-PK table, `PRAGMA defer_foreign_keys`, `libraries.rs:1532-1670`); (5) purge source-home rows; (6) fix BOTH universes' cache/own rows in the active DB (delete old-origin row, insert new-origin or own row); (7) wikilink cascade both sides (§2.3 mechanics); (8) journal complete, resumable at every step. `require_own_library` at the dest (`libraries.rs:2673`) becomes the explicit-intent door; the silent-crossing refusal stays for undirected paths.

**2.3 Rename cascade.** This is where Option B pays for itself. Today cross-universe inbound links break silently (identity-links map §3: both cascade branches refuse foreign, `libraries.rs:6932-6942, 7071-7096`). Under B, the active DB indexes *all* referrers, so `cascade_candidates_via_index` (reads `main.note_links.target_base`, `libraries.rs:7392-7425`) finds cross-universe referrers natively; the 6940 guard becomes the door: rewrite the foreign referrer's `.md` (Boss-authorized operation), reindex it **home**, refresh its cache row. Renaming a *federated* note: fs rename + home-connection `migrate_note_db_paths` (all 11 tables incl. earned, on the home DB where the earned rows actually live) + home reindex + rename-alias row written home + cache path update + two-sided cascade. `retarget_registered_libraries` writing the other universe's `libraries.json` gains the same journaled treatment. The PJ-227 phantom-row exemption dissolves: the 13 residual rows are re-adopted as origin-tagged rows or purged by origin-aware reconcile (needs the fresh Boss confirmation the rulings map calls for).

**2.4 Create (note/folder/daily/quick-capture/import).** Pickers re-list cU destinations behind the explicit affordance (renderer machinery still exists — `LibraryIcon kind='cuniverse'`, `MoveDialog.iconKind`; PJ-235 removed only the producer). Create writes the file (already crosses), mints cid (universe-neutral), then home-connection `index_note` + cache insert. MIG-099 collision check now reads the *unified* active index — strictly better coverage than today — with the home DB as the authoritative recheck. Import: post-import indexing routes home; `file_kinds.json` read from the *target* universe's `.constellation` (fixes importers.rs:828-830). Folders: no bookkeeping, policy-only (already true).

**2.5 Delete.** Home-connection purge + delete-archive rows (`DeleteReason`) in the HOME DB; trash mode routes to the **home universe's** `<root>/.trash` (Rust-resolved, never the frontend-supplied active trash root); cache rows purged from active DB. Fixes today's forever-stale foreign rows (write-surface map 3.6).

**2.6 Typed-link connect + earned link life.** (a) Foreign source: frontmatter write crosses; the `note_links` row is *born in the source's home DB* at home reindex; cache mirror follows. (b) Cross-universe target: in the **active cache**, target resolution can now stamp `target_cid_cn` across origins (name→cid within one index — with the origin-scoped cid join); in the **home DB** the target stays name-only/NULL-cid as today — a standalone child still cannot see inbound foreign edges (unchanged from today; a "foreign-inbound stub row written home" is a possible v2 extension, deliberately out of v1 scope since it would put non-recomputable foreign-derived data in the child). Federated `incoming_count`/backlinks are computed over the unified cache — better than today's name-string concat. (c) Earned mutations (`constellation_link_traverse`, confidence, archive/unarchive/dormant — the Class-E family, `search.rs:9735+`): route by the row's origin — foreign → home-connection UPDATE + cid-keyed ledger append into the **home** `.constellation` → mirror refresh. Today these silently match zero rows for foreign paths; under B the earned life actually accrues, in the right universe.

**2.7 Review actions.** `mark_reviewed`/`snooze`/`dismiss`: the `universe_root` routing parameter already exists (`review.rs:744`) — extend with Rust-side resolution so a foreign note's review state lands in ITS `review-pulse.json`, ITS schedule row, ITS earned ledger. Reviewer read surfaces: v1 = own-only as today (review_schedule holds no federated rows), with a follow-on choice: read federated due-lists via ATTACH union, or extend the mirror to `review_schedule` as a second cache table. Do not silently widen the Reviewer in this migration — it's a surface ruling.

**2.8 Embeddings / semantic search.** Embed-once, home-stored; federated semantic search joins `cuN.note_embeddings` via ATTACH (paths absolute). No vector duplication. `constellation_embed_notes` gains the same origin routing as reindex.

---

## 3. Speed / effort / risk — honestly

**Speed (runtime).** Reads get *faster*: quick-switcher, backlinks, sky, search collapse from N-schema concatenation to one indexed DB (the 17s sky-read precedent is the class of cost this removes). Federated writes gain one short-lived home-DB open per operation — WAL open + tokenizer registration per connection; **unmeasured**, must be measured before commit (hard constraint), but it sits on the debounced save tail, never the keystroke path. Keystroke latency: untouched by construction.

**Boot.** Two new costs, both mandatory to measure on the 7,600+ corpus: (1) first-attach backfill of a child — indexing a child the size of the Boss's universe is a minutes-scale background job (FTS tokenization dominates); resumable + after-paint or it violates the boot constraint. (2) Ongoing per-origin freshness pass — must be incremental (mtime cursor), or boot regresses. The read side *improves* (fewer cross-file queries at snapshot time).

**Storage — the biggest honest cost of Option B.** `note_meta` carries inline `body_text` (measured 1.7 GB on the Boss's universe, **[verified]** `search.rs:4188-4192` doc-comment); external-content FTS adds the token index on top. Caching a child full-fidelity means the parent's DB grows by roughly *the child's note_meta + FTS size* — federating a Boss-scale universe adds gigabytes to the parent, and a child federated by N parents is indexed N times on disk. Mitigations exist (cache without body_text — but that guts federated full-text search; contentless FTS — real complexity) and each subtracts from the option's headline benefit. This must be put to the Boss as a stated cost, not buried.

**Effort.** `/migration`-sized, 6–7 phases, touching schema + write path + indexer + watcher + reconcile + federation module + link_types + ~10 command tails. The de-globalization of `link_types` and the origin-aware retrofit of every purge/reconcile/repair path are the two long poles. Rough shape: comparable to MIG-108 in blast radius, larger than PJ-207. The per-operation doors can ship *incrementally* (guards stay per-op until each door lands), which matches the Boss's interim-walls directive and de-risks delivery.

**Risk register (ranked).**
1. **Provenance mislabeling = app-killer, both directions.** The same tables now hold system-of-record rows (origin 0) and droppable cache (origin ≠ 0). A purge that misreads origin eats earned data; a replay that ignores origin corrupts mirrors. Every destructive path (`reconcile`, `index_repair`, dead-row removal, ledger replay, `link_row_is_preserved`, marker-gate rebuild) must be origin-aware, and this must be safety-inspected as its own class.
2. **cid collision migration** on existing DBs (index swap under data) + every cid join gaining origin scope — miss one and review joins cross-wire universes.
3. **Vocabulary divergence.** Active-DB triggers interpolate the ACTIVE registry (`structural_not_in_clause`, `stratum_sql_expr` — **[verified]** `search.rs:5540, 5636-5640`); federated cache rows carrying a child's custom link types get classified by the parent's vocabulary → sky/maturity/strata drift on cached rows. Mitigation: classification-set union across attached children's `link-types.json` (read-only), or accept as bounded cache-quality drift; must be decided, not discovered.
4. **Two-instance write contention** (§4 below).
5. **Schema-version skew**: parent at v7 writing a child at v6 — the handshake refuses, which is safe but means a stale child blocks its doors until opened once; UX must say so plainly.

---

## 4. What breaks or degrades

- **Universe-folder portability: PRESERVED for the child** — its `.constellation` stays complete and authoritative; copy the folder, everything travels. This is Option B's decisive advantage over any design that hoists earned data into the parent. **Degraded for the parent**: its DB accretes foreign absolute paths; relocating a *child* orphans that origin's cache (heal: purge-and-rebuild per origin — cheap by construction, but a visible re-index); relocating the *parent* requires mig108 to skip foreign rows (new code, new failure surface).
- **Two-instance safety: degrades from "rare writes into a child" to "routine writes into a child."** WAL keeps it file-safe (many-readers/one-writer, busy_timeout everywhere), and `data_version` polling gives both directions a freshness signal — but every §4 hazard in the db-model map (process-local `db_ready`/generation/caches, non-cross-process `watcher_suppress`, duplicate backfills, `fs::copy`-over-live-WAL in the migrate/restore path) now fires more often. Minimum bundled hardening: `data_version` polling both ways, the exclusive-probe before earned-mutation bursts, and retiring `fs::copy` restore in favor of the backup API or a closed-connection window. A true fix (single-instance plugin or cross-process lease) should be named as its own PJ.
- **Boot time**: at risk from backfill + freshness; mitigations stated in §3; the hard constraint stands — measure or don't ship.
- **PJ-262 (Living Link disk layer)**: fully compatible, order-independent, cheaper after — once earned link data has a file layer inside each universe, the mirrors become recomputable-from-child-files and staleness stakes drop; the MOVE payload shrinks to index rows. Design the mirror refresh keyed to "home authority" so PJ-262 changes only the refresh *source*. Recommendation: do not sequence-block on it.
- **PJ-219 (drift asymmetry)**: largely *cured* — external edits to linked notes are watched and indexed; home-DB earned drift is caught by the polling contract. PJ-224/PJ-227 become cheap but still require their fresh Boss rulings (rulings map #7, #8).

---

## 5. Invariants

**Preserved:** earned data has exactly ONE home (the child's DB + ledgers — mirrors are demoted, marked, read-only cache); "each universe owns its own writes" survives as *authoritative writes go through the home identity* (MIG-100's phrasing, now load-bearing); File-Over-App (files remain source of truth; no silent modification — doors are explicit-intent only); "It is ONE universe" (reinforced — resolution literally reads one index); Write-Time Derivation (cache maintained at write time, backfill only for first population); One-Location per universe; the interim guards as the silent-crossing seal; second-screen display-not-domain.

**Endangered (each needs an explicit defense in the Plan):** the "search.db is NOT disposable" rule bifurcates *within one file* — origin 0 is irreplaceable, origin ≠ 0 is droppable, and every tool that ever "just rebuilds" must know the difference (this is the design's one genuinely new app-killer class); per-DB cid uniqueness becomes per-origin uniqueness (every consumer audited); the reconcile invariant "no write lands in a non-active universe" is formally REPEALED and replaced by "no *authoritative* write lands outside the note's home universe — and no write of any kind lands anywhere without explicit user intent or a provenance tag"; boot-time and index↔disk-divergence guarantees now span N roots instead of one.

**Bottom line for the option comparison:** Option B buys the most capability per operation (two-sided rename cascade, federated backlinks/search/incoming with real identity, earned life that actually accrues for federated notes) and keeps the strongest portability story, at the price of the largest storage footprint, a genuinely new provenance-safety obligation across every destructive path, and a first-attach indexing cost — with the saving grace that its dual-write problem is authority-then-cache (self-healing) everywhere except MOVE, which was already irreducibly two-phase.

==========================================================================================
## OPTIONS :: C-single-store
==========================================================================================

# OPTION C — ONE KNOWLEDGE STORE: Design Assessment

**Fresh verification done this session (beyond the six maps):** per-machine app-data precedent exists (`universes.json` lives at `{app_data_dir}/universes.json` — `universe.rs:96-102`); `notes_fts` is an external-content FTS5 table over `note_meta` (`content=note_meta, content_rowid=rowid` — `search.rs:5123-5129`); the SQL read surface is **242 `FROM note_meta` sites across 40 files + 242 more reads of the other big tables (note_links/sky/fts/review_schedule/aliases/tag_counts/term_vocab/embeddings) across 29 files ≈ ~484 read sites** (grep counts, this session); **20 `CREATE TRIGGER` statements across 3 files**, at least one of which **embeds the active universe's link vocabulary into trigger DDL** (`structural_not_in_clause("NEW.link_type")` at `search.rs:5540`); boot snapshot reads are full-table scans per schema (`cache.rs:1249-1252`, `1294-1299`); and the "no cross-universe merge" Boss principle (Q3 Option A) is stated verbatim at `cache.rs:1276-1281`.

---

## 0. Scope decision inside Option C: per-machine, not per-federation-root

"Per federation root" is incoherent as a partition: federation is a **graph, not a tree** — `add_child_universe` (`universe.rs:1453-1489`) accepts any valid universe path with only validity + not-self checks, so nothing prevents one universe being a child of two parents, and each cUniverse recursively has its own children. Two overlapping "federation roots" would both claim the same universe's rows. The only coherent form of Option C is **one store per machine**, at `{app_data_dir}/knowledge.db` (same location class as `universes.json`), holding every universe registered on that machine.

## 0.5 The earned-data fork — Option C is only sound in one of its three forms

- **C1 — machine DB stays the system of record for earned data.** REJECT. Today a universe folder carries its own `.constellation/search.db` + ledgers: copy the folder to another machine and earned data travels. Under C1 it does not — all earned data of ALL universes concentrates in one unsynced, unportable machine file. That is *worse* than today on the exact axis (CLAUDE.md Storage section) that makes search.db app-killer-class. C1 contradicts File-Over-App harder than the status quo.
- **C2 — complete the per-universe cid-keyed ledger first; the machine DB becomes fully derived.** The earned-data map proves the current ledger does NOT cover: `note_links.created` (`search.rs:470`), `note_aliases` rename/import rows, `shape_history`, `index_search_history`, `note_meta.created_at`/`content_changed_at`. C2 = extend `earned.jsonl`/`note-history.jsonl`/`review-pulse.json` (all staying in each universe's `.constellation/`) to cover that full §2 payload, keyed by cid_cn. Then the machine DB is disposable by construction: rebuild = re-walk .md files + replay ledgers.
- **C3 — PJ-262 full form (earned data in .md/LINK files).** C2's superset; same consequence.

**Everything below assumes C2/C3. Option C without the ledger completion is not a viable design.**

---

## 1. The design in concrete terms

### What changes

| Component | Change |
|---|---|
| **DB file** | One `{app_data_dir}/knowledge.db`. `db_path()` (`search.rs:1465-1468`) stops resolving the ambient active universe; becomes a constant. The per-universe `<root>/.constellation/search.db` files are merged in (one-shot journaled migration) then retired. |
| **Schema** | `universe_id` (or `universe_root TEXT`) column + index on every note-keyed table — at minimum the 13 path-keyed tables (mig108.rs:1061-1075 census) plus `tag_counts`, `term_vocab`, `index_search_history`, sight caches, backfill cursors (per-universe progress), `schema_versions` (per-universe module stamps). `idx_note_meta_cid_cn` becomes `UNIQUE(universe_id, cid_cn)` — a **global** unique would reject the legitimate copied-note case (same cid in two universes, identity-links map §1). |
| **Link vocabulary** | The hardest structural change nobody sees coming: vocabulary must move **from DDL/process-global into data**. Today `link_types::REGISTRY` is process-global loaded from the active universe (`link_types.rs:351, 520`), trigger DDL embeds the vocabulary (`search.rs:5540`), backfill gates fingerprint against the process snapshot (`links_backfill.rs:99`), and ~10 query sites splice `structural_not_in_clause` into SQL. Under a shared store, one universe's vocabulary in a shared trigger is exactly the PJ-230/PJ-232 defect made permanent. Fix: a `link_types(universe_id, id, structural, …)` table; triggers and queries JOIN it instead of embedding IN-clauses; fingerprints become per-universe rows. This is a self-contained sub-migration (~25 sites, verified by grep this session). |
| **SearchState** | Stays one writer + one reader — but the invalidate-on-switch machinery (`invalidate_search_state`, generation counter for DB identity, `search.rs:11228-11284`) largely dissolves: switching universes no longer swaps connections, it changes a filter value. The federation modules **get deleted**: `attach.rs` (ro ATTACH, cap 25), `federation_prewarm`, `federated_conn`, `migrate.rs`'s foreign-schema init, `InitScope::ForeignSchemaOnly`. Negative code — a genuine simplification. |
| **Every SQL statement** | The ~484 read sites + all writes need a universe-scoping decision: filter to active+federated set, filter to one universe, or deliberately global. This is the migration's true bulk. |
| **Attribution** | `constellation_search_reindex` resolves `path → universe_root (find_universe_root) → library within that universe` Rust-side; Class D (frontend-supplied `library_name`, 22 call sites) dies as a side effect. |
| **Frontend** | The `childUniverseLibPaths` side-map and predicates survive as **visibility/policy** state, no longer as write-routing state. `OpenTab` still needs no new field. |

### What stays

`.md` files as source of truth; per-universe `.constellation/` folders (ledgers, `libraries.json`, `universe.json`, review-pulse, `.trash`, templates); `universes.json` registry; One-Universe-One-Location (a layout rule, untouched — rulings map row 10); the watcher (already watches federated libs); Write-Time Derivation (triggers still fire per write, now vocabulary-joined); the write gate / journaling; the whole editor/frontend save pipeline.

---

## 2. Each operation class under Option C

1. **Edit/save** — write the .md (already crosses via federated auth); reindex into the one DB with Rust-resolved `universe_id`. No routing question exists; the "which DB" problem vanishes. Embedding write same.
2. **Create** — mint cid (universe-neutral), write file, index. MIG-099 collision check (`libraries.rs:1414-1435`) becomes one query over the store scoped to the target universe. The cross-universe cid-collision hazard (invisible-note SQLITE_CONSTRAINT, identity-links map §1) becomes **detectable at index time** across all universes — a win.
3. **Move (cross-universe, the PJ-276 headline)** — the single biggest technical win: fs rename + **one SQLite transaction** updating `universe_id` + path columns on all earned rows. The WAL "no atomic cross-DB transaction → journaled two-phase" problem (write-surface map §4.2) **vanishes**. No export/import payload, no row transfer, no re-key. Remaining non-atomic part: moving the cid-keyed ledger lines between the two universes' `.constellation/` files — but ledger entries are cid-keyed and replay is idempotent-by-design (`link_life_restore.rs` runs every boot unstamped), so crash-mid-move is recoverable by replay, not by a resume journal. Trash routes to the owning universe's `.trash` by path.
4. **Rename cascade** — referrer seek becomes ONE indexed query over the whole store (`note_links.target_base`, all universes, filtered to the federation set); the foreign-bounded walk refusals (`libraries.rs:6932-6942, 7071-7096`) become policy filters instead of structural walls; the 13 residual phantom rows (PJ-227) dissolve into properly-attributed rows. Cross-universe inbound links stop breaking silently — their rows are in the same table and get migrated in the same transaction. `retarget_registered_libraries` still needs to write the other universe's `libraries.json` (disk-side, unchanged by C).
5. **Delete** — row purge + delete-archive scoped by universe_id in the one store; trash to the owning universe's `.trash`. The "foreign DB keeps its rows forever" defect (write-surface §3.6) cannot occur.
6. **Link** — the `note_links` row carries the source's universe_id. `target_cid_cn` stamping (`search.rs:8419-8427`) can now resolve across the whole store: **cross-universe edges become identity-keyed rows instead of name-strings**, backlinks stop being name-concatenation (`store.ts:5641-5663`), the same-name backlink-attraction bug (identity-links §4) becomes fixable, and the standalone-open asymmetry ("open Y alone and A's backlink doesn't exist") becomes a *visibility policy choice* rather than a structural impossibility — the row is in the store either way. Earned link life (traverse/confidence/archive) needs no routing: the row is findable by path regardless of which universe is active; ledger append goes to the source universe's `.constellation/`.
7. **Review** — `review_schedule` keyed (universe_id, path); `review-pulse.json` stays per-universe, path-keyed (rewritten on move as today, `libraries.rs:1606-1609` precedent); `mark_reviewed`'s existing `universe_root` param generalizes to Rust-side path resolution. `set_review_priority`'s "permanently unindexed linked-universe note" refusal becomes obsolete — foreign notes ARE indexed.
8. **Embeddings** — `note_embeddings(universe_id, path, …)`; semantic search across the federation becomes one query; re-embed cost unchanged.
9. **Search (PJ-224 adjacency)** — the ordinary box federates **by construction** (one FTS index; external-content over the unified note_meta). Whether it *should* remains the pending Boss ruling — under C the implementation of either answer is a WHERE clause.

---

## 3. Speed / effort / risk — honest

**Effort: the largest migration in the project's history, strictly larger than MIG-108.** Concrete basis: ~484 verified read sites across 40+ files each needing a scoping decision; 20 triggers regenerated (one vocabulary-embedding); the link-vocabulary-to-data sub-migration (~25 sites); 14 boot backfills + reconcile + heal re-scoped to per-universe cursors; the N-database merge engine (journaled, snapshot-first, resumable — mig108.rs precedent, but merging *live earned data* with real cid collisions, where mig108 only rewrote paths); the C2 ledger completion as a **prerequisite migration in its own right**; frontend visibility filters; and per the standing order every build Boss-tested. This is multiple `/migration` cycles, not one. I will not invent a week count; the honest statement is: MIG-108 was one journaled engine plus path rewrites and was itself a major migration — Option C contains a MIG-108-class merge engine as *one of its phases*.

**Speed (runtime):** per-note operations unchanged (indexed lookups). Cross-universe move becomes dramatically simpler AND faster (one transaction vs export/import). Federated queries lose the ATTACH/schema-loop overhead. **But boot is the hard constraint**: the boot bundle is full-table scans (`cache.rs:1249-1252`); the SKY read of ONE universe (233,995 sky_links) already measured ~17s (MIG-079 memo). Un-filtered scans over a machine-wide store regress boot for any user with multiple universes; every boot query needs a `universe_id IN (federation set)` filter + covering index (the PJ-249 covering-index drift saga shows how easily that un-covers silently). CLAUDE.md's hard constraint — no boot-time regression, measured on 7,600+ notes — applies to every one of these queries.

**Risk: the scoping inversion is the headline risk.** Today isolation is **structural** — separate files; a query physically cannot see another universe's rows unless it ATTACHes. Under C, isolation is a **WHERE clause repeated ~500 times**, and every missed filter is a silent cross-universe leak — data from a never-federated universe appearing in search results, tag counts, Sky, Sight, review queues. This is the exact silent-failure app-killer class the Safety Inspection exists for, relocated from the write path (where the current guards sit) into *every read in the app*. A write-path boundary is ~30 guarded sites; a read-path boundary is an order of magnitude more. Mitigable (views per federation set, or a query-builder that injects the filter) but the mitigation is itself architecture.

**Blast-radius concentration:** the schema-version gate becomes ONE marker for all universes — a schema bump renames aside and re-derives EVERY universe on the machine at once (recoverable under C2 by walk+replay, but the recovery window covers the user's entire knowledge, not one universe). Two-instance hazards (db-model map §4) now apply to every instance pair unconditionally; a single-instance guard (`tauri-plugin-single-instance`, absent today — Cargo.toml verified) becomes effectively mandatory, which itself conflicts with the second-window architecture unless scoped carefully.

---

## 4. What breaks or degrades

- **Universe-folder portability:** under C1, destroyed (rejected). Under C2, **improved over today** — the folder carries .md + ledgers only; arriving on a new machine, the store indexes it and replays. Bonus: the SQLite file leaves the user's sync path entirely (search.db over Syncthing is a live corruption vector today; append-only JSONL is sync-friendly).
- **"Detach a cUniverse and the remaining universes' link data is unaffected"** (Boss principle Q3 Option A, verbatim at `cache.rs:1276-1281`): the guarantee survives logically but its *enforcement* moves from file structure to code. Detach becomes a filter change; the rows remain interleaved in one file. Un-registering a universe poses a new policy question: purge its rows (loses derived state if re-registered — fine under C2) or retain (store bloat).
- **Two-instance safety:** degrades — see §3. Everything in db-model map §4 concentrates onto one file.
- **Boot time:** at risk everywhere a filter is missed; strictly more rows resident per table.
- **Privacy-by-structure:** a never-federated universe's content sits in the same physical file as everything else; one query bug exposes it. Today that exposure is impossible by construction.
- **PJ-262 interaction — the honest strategic point:** C2's prerequisite (complete cid-keyed ledger) is ~80% of PJ-262 itself. And **once that ledger exists, the per-universe-DB model ALSO becomes fully rebuildable** — cross-universe move under Options A/B collapses to "move the .md + move the ledger lines + reindex both sides," no earned-row export needed. So the ledger work, which Option C cannot skip, simultaneously shrinks Option C's unique advantage. What remains uniquely C after PJ-262: single-transaction moves, WHERE-clause federated search (PJ-224 trivial), identity-keyed cross-universe links/backlinks, deletion of the whole attach layer, and the death of Class D by construction. The Architect must weigh exactly those five wins against the largest-in-history migration plus the scoping-inversion risk class.

## 5. Invariants — preserved vs endangered

**Preserved (some strengthened):** File-Over-App (C2 strengthens it — earned data finally becomes files); Write-Time Derivation (triggers per write; reads stay lookups); "It is ONE universe" resolution ruling (reinforced — resolution is literally one table); MIG-100 registry-membership authorization (subsumed; attribution becomes automatic); One-Universe-One-Location (untouched); MIG-056 §C's safeguard pattern (obsoleted rather than violated — there is no foreign DB to write); earned-data durability (C2: machine-DB loss recoverable by replay — better than today, where losing search.db loses earned data permanently).

**Endangered:** per-universe isolation as a *structural* guarantee (becomes ~500 WHERE clauses — the new silent-failure class); the boot-perf hard constraint (every unfiltered scan); the schema-gate blast radius (one marker, all universes); two-instance safety (one shared file, always); the Q3-Option-A "pure concatenation, no merge" contract (its letter — separate per-schema reads — is repealed by construction; its spirit — no cross-universe remapping of one universe's data — must be re-proven in the merge engine, where cid collisions across universes are handled for the first time).

**Bottom line, stated honestly:** Option C is architecturally the cleanest end-state for the Boss's ruling — every cross-universe operation becomes an ordinary operation, atomically — but it is only sound with the PJ-262 ledger completion done FIRST, and that prerequisite independently removes most of the pain from the per-universe alternatives. Its irreducible costs are the ~500-site scoping inversion (a new app-killer class), a boot-perf proof obligation on every query, and a multi-cycle migration containing a MIG-108-class merge engine as one phase. If chosen, sequence: (1) C2 ledger completion as its own migration, (2) link-vocabulary-to-data, (3) schema + merge engine behind a toggle with the old per-universe DBs retained as rollback, (4) query-surface conversion subsystem-by-subsystem with per-subsystem boot measurements.

Key files: `src-tauri/src/search.rs` (db_path 1465, notes_fts 5123, vocab trigger 5540), `src-tauri/src/universe.rs` (registry_path 96-102, add_child_universe 1453), `src-tauri/src/cache.rs` (boot scans 1249-1336, Q3 principle 1276-1281), `src-tauri/src/link_types.rs` (REGISTRY 351, structural_not_in_clause 268), `src-tauri/src/federation/attach.rs` (deleted under C), `src-tauri/src/mig108.rs` (merge-engine precedent), `src-tauri/src/link_life.rs` + `link_life_restore.rs` (the C2 prerequisite's foundation).

==========================================================================================
## VERDICTS :: A-route-to-owner
==========================================================================================

All load-bearing citations have been verified in source. Composing the adversarial findings.

## VERDICT: VIABLE-WITH-CONDITIONS

Option A is the architecturally correct shape — it is the only routing design that keeps every universe folder self-contained (nothing of the child's ever lives in the parent), and it matches the repo's own hard-won lessons (the PJ-230 comment in `src-tauri/src/federation/migrate.rs:86-131` explicitly forbids the global-registry-swap alternative; Option A's threading is the documented right answer). But the design as written contains one materially false claim, one defense that factually does not work, and three under-specified crash/consistency hazards. Conditions below.

---

### A. Claim verification (all cited sites read in source, this session)

**Verified TRUE:** commit 7921e593 and its scope (incl. "~7 commands still authorise federated", PJ-270..276 filing); `require_own_library`/`require_own_library_in` at `libraries.rs:295-345` with the known-limit comment at 320-323 exactly as quoted; `SearchState` single-slot pattern at `search.rs:1341`; `find_universe_root` at `src-tauri/src/federation/attach.rs:105-118`; `InitScope::ForeignSchemaOnly` at `search.rs:4570-4589`; the §C safeguard at `migrate.rs:68-180`; `link_types::REGISTRY` OnceLock at `link_types.rs:351` with `load_active` at `search.rs:11606`; `review.rs` `universe_root: Option<String>` at 744/779/806 and the `set_review_priority` refusal at ~703-715; `link_row_is_preserved` at `search.rs:450-489` incl. "`created` is not in the ledger at all" (line 470); weight-is-derived at `link_life_restore.rs:8-11`; `constellation_search_reindex` trusting frontend `library_name` at `search.rs:12251-12255`; `index_note(conn, path, library, force)` conn-parameterized at 12718; the create_note reindex skip at `libraries.rs:1436-1451`; `migrate_note_db_paths` at `libraries.rs:1532`; cascade refusals at `libraries.rs:6932-6942`/`7071-7096`; `importers.rs:828-830`; the removed `'cuniverse'` branch at `+layout.svelte:6921-6928` with the renderer union type surviving at `MoveDialog.svelte:27`; the watcher fan-out at `+layout.svelte:2944` over `$libraries`, which IS the federated list (`store.ts:3929` → `resolve_universe_libraries`); `federation_prewarm` rw-open precedent at `search.rs:11358` incl. the "cUniverse files are writable by design" comment; the generation discipline at 11635-11648; `resolve_universe_libraries` at `universe.rs:1513`.

**FALSE claim #1 (app-killer class, inside the design's own risk #1):** §2 Edit/save says sky/term/incoming maintenance is "gated by the CHILD's own backfill stamps." Wrong. The stamps are per-DB but the gate compares them against the **ACTIVE** universe's registry fingerprint: `incoming_links_backfill.rs:49` — `is_built(conn) && stored_vocab_fingerprint(conn) == crate::link_types::snapshot().fingerprint()` — and `links_backfill.rs:99` likewise. `reindex_single_note` calls `is_built(conn)` at `search.rs:12712`. With differing parent/child vocabularies, `is_stamped(child_conn)` is false → incoming maintenance is silently skipped on every routed write while `is_built` stays true, so **stale child aggregates are served with no error** — the exact silent-divergence class the design promises to prevent, at a site the design did not name. The registry threading must reach the fingerprint gates, not just `index_note`'s parse chain. This proves the "30+ sites to audit" list is incomplete as enumerated.

**FALSE defense #2 (the two-instance posture does not work):** the design's honest posture is "extend the `is_cuniverse_open_elsewhere` lock-probe to write-time." The probe (`migrate.rs:191-208`) is `BEGIN EXCLUSIVE; ROLLBACK` with 100ms busy_timeout. In WAL mode — which every Constellation DB uses — `BEGIN EXCLUSIVE` acquires the **write** lock only; it does not exclude readers, and an instance B holding the child open as its active universe with no write in flight does **not** block it. The probe returns "not open elsewhere" in precisely the routine case Option A worries about — a **false negative**, and the in-code comment (documenting false *positives*) describes the opposite, rollback-mode failure. The write-time defense as designed will almost always admit the write. Condition 3 below replaces it.

---

### B. Findings the option missed (ranked)

1. **Pool-open trigger regeneration can resurrect the MIG-056 marker-clearing bug shape.** The vocab fingerprint stamp is what schedules the child's own boot-time rematerialization after a vocabulary change (`links_backfill::is_needed`). If `ForeignFullWithRegistry` generates DDL from the child's registry **and stamps the fingerprint**, the child's next boot sees fingerprint-current and skips the backfill that would have repaired aggregates built under the old triggers — stale data blessed permanently. This is exactly the shape `migrate.rs:88-95` documents ("The clearing is what made it permanent"). The pool may regenerate triggers but must NEVER write the vocab-fingerprint stamps; only the owner's completed backfill stamps.

2. **The transfer journal violates universe self-containment as specced.** The coordinator journal lives in the *active* universe's `.constellation` with only "breadcrumb markers" in both roots. Universes are portable and independently syncable (File-over-App); after a crash between fs-move and dest-insert, the destination's next boot may run on a machine/session where the parent's folder is unreachable — replay impossible, and the destination's cold-start auto-index (`+layout.svelte:2949`, PJ-065 §8) indexes the arrived note fresh at `'hypothesis'` defaults, i.e. the earned half exists in zero places the destination can see. The **full extracted payload must be durable in the destination root's own breadcrumb before the fs move**, and journal replay must be ordered before boot reconcile/de-index on both sides. Symmetrically, a source-side crash-boot's reconcile will see the .md gone and run `DeleteReason` cleanup on rows the journal still needs — extraction must be durably journaled before step 3, which the design implies but never states.

3. **cid_cn is not collision-safe across universes.** `generate_canonical` (`canonical.rs:49-79`) is timestamp + random 4-char suffix, collision-checked only against the target *directory*. A user who file-copies a note between universes duplicates the frontmatter `cid_cn` verbatim. Journal replay "keyed by cid_cn" can then attach earned data to the wrong note in the destination. Replay must key on (cid, exact destination path) and refuse-and-report on ambiguity — the same rule `link_life_restore.rs:18-30` already established for name-keyed records.

4. **Child-registry staleness re-opens the wrong-vocabulary hole through the correct path.** The pool caches the child's registry at open; the child's `link-types.json` can change underneath it (child booted elsewhere, Syncthing). A stale cached snapshot mis-parses exactly like the global registry did. Cheap fix: stat/fingerprint the child's `link-types.json` per routed write or short TTL.

5. **First-routed-write latency + backup-integrity cliff.** Pool open can trigger the §C migrate on the save tail: `fs::copy` of a possibly multi-GB child DB plus the FTS rebuild that `ForeignSchemaOnly` deliberately does NOT skip (`search.rs:4583-4587`). Two problems: (a) a minutes-long stall triggered by a debounced *save*, colliding with PJ-103's 5s app-close flush cap; (b) `fs::copy` of `search.db` alone while a `-wal` exists (live attaches, or unclean child close) produces a backup that is not a valid restore point, and restore-on-fail beside a stale `-wal`/`-shm` is a corruption recipe. Pre-existing at boot-migrate, but Option A multiplies exposure. Conditions: use the SQLite backup API (or checkpoint-TRUNCATE first), and run pool-open/migrate at link-time or first foreign-tab-open with progress UI — never on the save tail; saves refuse visibly until the child is ready.

6. **`WriteIntent` classification has unruled hard cases.** `ensure_cid_cn` runs on the note-OPEN pipeline and writes frontmatter (`canonical.rs:1449-1472`) — opening a foreign note is a user gesture, but the write is automatic identity-injection into another universe's file: exactly the "silent crossing" class the ruling keeps refused. Watcher-adopt reindex after an external child edit is `Automatic` → refused → the parent *displays* fresh content over a stale child index (display/index divergence inside the parent's own panels; PJ-219 sharpening, acknowledged but not resolved). The Plan needs an enumerated write-site → intent table with Boss rulings for the ambiguous rows.

7. **Trash restore surface (Whole-Ecosystem law).** Routing deletion to the owner's `<root>/.trash` is right, but whatever surface lists/restores trash must learn to look in the owner's trash, or a foreign note deleted from the parent seat becomes unrestorable from anywhere the user will look. Not in the design's surface enumeration.

8. **Honesty items:** the multi-universe rename cascade only reaches the *visible* federation set (directional; universes linking the child but not linked by the active parent stay silently broken — inherent to federation, must be stated, not marketed as "closed"). PJ-227's 13-row purge is documented in-code as blocked on a PJ-224 ruling (`libraries.rs:6932-6935`) — the dissolution goes back through that ruling, not around it. Embedding-model/dimension compatibility between parent process and child `note_embeddings` was not verified by the design or by this review — a Plan-phase verification item, not an assumption.

---

### C. What Option A gets right (that competing options likely miss)

- **Portability is preserved by construction, not by cleanup.** No cross-universe rows exist anywhere; unlink loses nothing. Any design that mirrors child rows into the parent DB (or a shared coordinator DB) breaks copy-the-folder semantics and inflates the PJ-227 residual class from 13 rows to the whole federation.
- It correctly refuses the global-registry-swap trap that `migrate.rs:98-100` explicitly forbids, and correctly identifies registry threading as risk #1 and Phase-1-gated — my finding #1 deepens that scope but confirms the direction.
- Schema-gate ownership (never rename-aside a child's DB) is the right generalization of the 2026-07-24 rename-aside lesson.
- The Class-D kill via Rust-side attribution closes PJ-275 with zero churn at 22 frontend call sites — strictly better than per-site fixes.
- Boot untouched, reads untouched, lazy pool with the proven generation discipline — respects the hard boot/typing constraint, and the rw-side-open-while-ro-attached pattern has a verified in-repo precedent (`federation_prewarm`).
- The transfer engine's earned-data rules (weight recomputed never copied; `link_row_is_preserved` as the merge criterion; Rust-side path rewrite) match MIG-104/MIG-108 source exactly.
- `WriteIntent::Automatic` retaining the refusal is the correct reading of the Boss ruling: full operations means user-commanded gestures cross, ambient maintenance still never does.

---

### D. Conditions for viability

1. **Registry threading must be provably complete**, including the fingerprint gates (`links_backfill.rs:99`, `incoming_links_backfill.rs:49`) and every registry read reachable from a routed write — enforced structurally (routed paths receive a snapshot type the global accessor cannot satisfy), with a red→green test that indexes one note under two differing vocabularies and diffs `note_links` and the maintenance outcomes. Ships before any foreign index write, as Phase 1, gated.
2. **Pool open never writes vocab-fingerprint stamps**; trigger regeneration leaves mismatched fingerprints in place so the child's own boot heals its aggregates.
3. **Replace the write-time lock probe** with a real cross-process presence signal (lockfile + heartbeat in the child's `.constellation`, or the single-instance guard) — the WAL `BEGIN EXCLUSIVE` probe is blind to an idle open-elsewhere child and cannot carry this defense.
4. **Transfer journal: payload durable in the destination root before the fs move; replay ordered before reconcile on both sides; replay keyed (cid, path) with refuse-and-report on duplicate cids.** Every crash window red→green in the harness before the door opens (Reproduce-First; the design already commits to this — hold it to it).
5. **Pool open (incl. §C migrate) off the save tail**, backup via SQLite backup API or after checkpoint-TRUNCATE, progress surfaced, saves refusing visibly until ready.
6. **Per-write staleness check on the cached child registry** (stat `link-types.json`).
7. **Write-site → intent table Boss-ruled in the Plan**, with `ensure_cid_cn`-on-foreign-open ruled explicitly.
8. **Trash listing/restore extended to owner roots** in the same pass (Whole-Ecosystem law).
9. **Sequencing**: surface PJ-262-first to the Boss — it shrinks the transfer engine's riskiest module and the design's own recommendation is sound.
10. **Measure** (7,600+ note universe): steady-state routed-write latency, first-write pool-open cost, child WAL growth under sustained parent writes — before commit, per the CLAUDE.md hard constraint.

With conditions 1-5 as blocking gates and 6-10 in the Plan, Option A is the strongest available shape for the Boss ruling: it is the only design where "full operations between universes" and "each universe remains a self-contained, portable unit" are the same property rather than a trade-off.

==========================================================================================
## VERDICTS :: B-provenance-index
==========================================================================================

All verification complete. Composing the adversarial report.

# ADVERSARIAL ATTACK REPORT — OPTION B (Federated Index with Provenance)

## VERDICT: **VIABLE-WITH-CONDITIONS** — the architecture is the right shape, but it ships with two app-killer-class cross-process holes it did not name, one load-bearing claim that is only conditionally true, and an origin-identity model that manufactures permanent ghost rows and a wrong-file-rewrite path. All are fixable; each must be a named blocking condition in the Plan.

---

## 1. FALSE-CLAIM AUDIT — the option is factually sound

Every citation I checked (~30) verified against source this session: `idx_note_meta_cid_cn` unique-on-cid_cn (`search.rs:3841-3844`), external-content FTS + `note_meta_ai/ad/au` triggers (`search.rs:5122-5161`), `schema_versions` ledger (`search.rs:4678-4684`), marker gate renames-aside (`search.rs:11573-11601`), `ChildUniverseInfo {name,path,library_count}` (`universe.rs:1546-1552`), global `REGISTRY` (`link_types.rs:351`), `InitScope::ForeignSchemaOnly` (`search.rs:4571-4589`), frontend-trusted `library_name` (`search.rs:12251, 12718`), watcher-flush foreign skip (`search.rs:12956-12970`), `write_note` federated validation (`libraries.rs:981`), cascade foreign refusals (`libraries.rs:6932-6942, 7071-7096`), `migrate_note_db_paths` + FK reality (`libraries.rs:1532-1575`), cid join (`review.rs:149`), `universe_root` param (`review.rs:744`), mig108 13-pair census (`mig108.rs:1061-1075`), trigger vocab interpolation (`search.rs:5540, 5636-5640`), 1.7 GB inline body_text (`search.rs:4188-4192` doc), `importers.rs:828-830`, `cache.rs:1272-1300`, boot-repair `foreign` outcome (`src/lib/libraries/store.ts:4079-4088`), watcher start over federated `$libraries` (`+layout.svelte:2944-2947`, `store.ts:3925-3934`). Shorthand note: `attach.rs`/`migrate.rs` are `federation/attach.rs`/`federation/migrate.rs`; `store.ts` is `libraries/store.ts`. `tag_counts`, `note_body`, `note_aliases`, `index_search_history` all exist. **No fabrications found.** One material omission in its schema census (finding B4).

## 2. SILENT-DATA-LOSS PATHS THE OPTION CREATES

**A1 — APP-KILLER: cross-process ledger append vs. compaction (the Slice-7 bug returns, verbatim, across the process boundary).** `link_life.rs:191` — the ledger's `FILE_LOCK` is a **process-local** `static Mutex`. §2.6(c) has the parent appending earned-decision lines into the CHILD's `.constellation` ledger. When the child universe is open in its own instance — routine under B — the child's `maybe_compact` renames the tail aside while the parent's append is in flight; on Windows the rename succeeds under an open append handle (`FILE_SHARE_DELETE`, documented at `link_life.rs:170-172`), so parent-appended lines land in `earned.tail-<stamp>.jsonl` **which nothing ever reads back**. The module's own doc states the damage (`link_life.rs:179-184`): the next fold carries the pre-decision value and **writes it back — silently reversing a retirement/priority while every step logs success**. The option's §4 hardening bundle (data_version polling, exclusive-probe, fs::copy retirement) does not cover the ledger. **Condition: OS-level file lock (or lock-file protocol) on the ledger directory, or refuse home earned-appends whenever the child is open elsewhere — the exclusive probe must extend to the LEDGER, not just the DB.**

**A2 — APP-KILLER: attach-time auto-migration can roll back another writer's committed home-writes into the authority DB.** `federation/migrate.rs:80-82` backs up the child DB with `fs::copy` (no `-wal` sibling); `:155` restores by `fs::copy` over the live file on failure; `is_cuniverse_open_elsewhere` (`:191-207`) is explicitly best-effort TOCTOU whose comment says it is sufficient "for v1's expected single-user-single-process pattern" — **the exact pattern Option B repeals**. Under B: (a) a torn backup of a live-WAL child (another parent or the child itself writing), then (b) restore-on-failure overwrites committed home-writes made after the backup — silent earned-data loss inside the AUTHORITY. The option names fs::copy retirement as "minimum bundled hardening"; it is a **blocking precondition**, because this code runs on every parent's boot path.

**A3 — Lost-update race on earned scalars.** `constellation_link_traverse` (`search.rs:9749-9769`) is read-modify-write: SELECT `traversal_count` → compute weight in Rust (deliberately, because `ln()` needs SQLITE_ENABLE_MATH_FUNCTIONS, `:9753-9754`) → per-id UPDATE, as separate autocommit statements serialized today only by the process-local `state.db` mutex. Parent home-write + child-instance traverse on the same link = a lost increment (or, wrapped in DEFERRED tx, SQLITE_BUSY_SNAPSHOT aborts). The option specifies no transaction discipline for home earned mutations. **Condition: every home earned mutation wrapped in `BEGIN IMMEDIATE` + bounded retry (or restructured single-statement).**

## 3. UNSOUND OR UNDER-SPECIFIED CLAIMS

**B1 — "Child's own triggers maintain its sky/FTS/incoming" (§2.1) is only conditionally true, and the proposed handshake cannot see the condition.** `init_db_schema_only` deliberately creates NO vocabulary-dependent triggers on a foreign DB, justified in-source by *"until then nothing writes through those triggers, because a cUniverse is attached read-only"* (`federation/migrate.rs:127-131`) — **the premise Option B repeals**. The `search.version` marker does not attest trigger presence or vocabulary currency. A home-write into a schema-only-migrated child inserts `note_links` rows that fire no sky/outgoing/incoming maintenance → the AUTHORITY's derived tables silently diverge from its own note_links; the child's next active boot recreates triggers (`search.rs:5531+` drop/create) but **backfills nothing**. **Condition: handshake must check an owner-stamped vocabulary-DDL row in the child's `schema_versions` (written only by an active boot), refusing otherwise — or the home tail must maintain derived rows explicitly and never rely on child triggers.**

**B2 — Path-keyed origin identity × "never purge unreachable origins" = permanent ghosts and a wrong-file rewrite path.** A relocated child (drive letter, folder move, NFC/8.3 spelling variance the codebase itself documents at `libraries.rs:7104`) becomes one forever-"unreachable" origin of stale rows PLUS a fresh origin of the same notes: duplicate search/backlink/switcher results with no reconcile authority to remove them. Sharper: the §2.3 cascade finds referrers in cache and rewrites files at recorded paths; a stale foreign row whose old path is later occupied by a DIFFERENT universe's file (path reuse) directs a Boss-authorized rewrite **into the wrong universe's note** — the `path.exists()` stat-guard (`libraries.rs:6929`) makes it fire, not prevent it. **Condition: mint a durable universe UUID into `universe.json` at first attach (the option notes none exists, then builds identity on the weaker key anyway); relocation-detection + explicit stale-origin purge UX; cascade must re-resolve a foreign referrer's owning root at rewrite time, never trust the cached origin.**

**B3 — "Watcher suppression covering both roots" (§2 preamble) is process-local.** `watcher_suppress.rs:35-38` is a static in-process map. The child instance's watcher sees every parent-side cascade/move/home rewrite as an external edit: clean → adopt; open-and-dirty → `.conflict` sidecar (PJ-070). A universe-wide rename cascade from the parent can spawn conflict sidecars across the child instance's open tabs in one stroke. Safe, not silent loss — but a designed-in cross-instance behavior the Boss must be shown and accept.

**B4 — The schema census omits the FK web.** FKs are enforced on every rusqlite connection; `note_summaries`, `sources_suggestions`, `note_state_history` reference `note_meta(path)` ON UPDATE NO ACTION (`libraries.rs:1547-1556` — the proven PJ-151 silent-failure class). The option's origin_id list covers 7 tables and never states a policy for these dependents: if any surface creates a dependent row for a federated note (NSC summary, sight layout, state history), origin purge/refresh delete+reinsert hits FK refusals — best-effort-silently, the exact 1,591-deferred-relocates shape. **Condition: either origin_id + policy on all FK dependents, or a hard enforced invariant "no dependent rows for origin≠0", stated per producer. Note the option's own mig108 citation (`mig108.rs:1061-1075`) lists these very tables — the refutation of §1.1's census is inside its own evidence.** Corollary: mig108's in-transaction aggregate verification must itself become origin-aware, or skipping foreign rows makes the verifier false-fail/false-pass.

**B5 — cid-collision handling is directionally right but incomplete.** (a) The join audit ("every cid-keyed join gains an origin term") must be a grep-audited enumeration in the Plan (Whole-Ecosystem Fix Law), with a two-universes-same-cid harness test — today `query_row` on a colliding join silently picks an arbitrary row. (b) **MOVE (§2.2) has no cid-collision policy at destination**: the note carries cid_cn in frontmatter; import pre-deletes by PATH, not cid; a colliding cid at a different dest path hits the per-origin unique index mid-two-phase → constraint abort inside the journaled sequence, or one note invisible. Needs explicit re-mint + inbound `target_cid_cn` rewrite policy — itself a cascade.

**B6 — A Boss-locked principle is contradicted without a ruling line-item.** `cache.rs:1276-1281` records MIG-061 Q3 Option A (Boss-locked): the federated graph is a pure concatenation, "no cross-universe merge or remapping." §2.6(b) stamps `target_cid_cn` across origins in the cache — a cross-universe merge. The 2026-08-12 full-ops ruling plausibly supersedes it, but the option's ruling list (PJ-224, PJ-227) omits it; it must be surfaced for an explicit ruling, not silently overwritten.

## 4. PERF/STORAGE — sharpening what it already admits
- Backfill writes gigabytes through ONE WAL; without per-batch checkpointing this is the 3 GB-WAL-vacuum incident class again. Checkpoint per batch, stated in the Plan.
- First-attach backfill and the already-running federated watcher (`+layout.svelte:2944-2947`) index the same child concurrently — needs a single-flight guard or the active DB writer lock contends on the boot path.
- `PRAGMA data_version` polling fires on the parent's own short-lived home connections too (any other connection counts) → self-triggered refresh churn; debounce.
- ATTACH cap is real and Boss-locked at 25 with the compile-time bump still pending (`federation/attach.rs:25-30, 45`) — the embeddings-via-ATTACH read path inherits that ceiling; fine, but say so.

## 5. WHAT OPTION B GETS RIGHT (that competitors likely miss)
1. **The authority/cache split with "loss = re-derivable" for every federated row** is the only shape that keeps a child's folder self-contained and movable (File Over App). Any design hoisting earned data into the parent destroys portability permanently; B provably does not.
2. **Refusing the `search.version` marker-bump delivery is correct and non-obvious** — verified: the gate renames the whole DB aside (`search.rs:11562-11601`) and replay does not cover several earned columns; a naive option would silently shed earned data on upgrade.
3. **It caught the cid_cn unique-index collision before it fires** — verified at `search.rs:3841-3844`; on any naive federated-index design the FIRST cross-universe cid collision at attach makes a note invisible. This is a first-attach app-killer pre-empted at design time.
4. **FTS-for-free is real** (`content=note_meta` + triggers, verified) — mirror-table designs pay a second tokenization pipeline and a drift surface.
5. **It kills Class D at the root** (frontend-supplied `library_name`, verified at 12251/12718) instead of guarding around it, with zero frontend call-site churn.
6. **`link_types` as a parameter, not a global swap**, is exactly the fix `federation/migrate.rs:98-100` warns is required — the option engaged with the hardest prior lesson (PJ-230/232) rather than dodging it.
7. **Correct decomposition of the dual-write problem**: authority-then-cache is self-healing everywhere except MOVE, which was already irreducibly two-phase (mig108 precedent). This is the honest minimum of cross-DB atomicity.
8. Origin-aware, availability-aware reconcile (unplugged drive ≠ dead rows) is the right default — it just needs B2's durable identity to be complete.

## 6. CONDITIONS FOR VIABILITY (all blocking, in Plan order)
1. Ledger cross-process lock or refuse-home-earned-appends-when-child-open (A1).
2. Retire `fs::copy` backup/restore in `federation/migrate.rs` (SQLite backup API or closed-connection window) BEFORE any door opens (A2).
3. `BEGIN IMMEDIATE` + retry discipline for every home earned mutation (A3).
4. Handshake verifies owner-stamped vocabulary-DDL currency, not just `search.version` (B1).
5. Durable universe UUID for origin identity + relocation detection + stale-origin purge UX + rewrite-time root re-resolution in the cascade (B2).
6. FK-dependent-table policy (origin_id or enforced no-federated-rows invariant) incl. origin-aware mig108 verification (B4).
7. Grep-audited cid-join enumeration + same-cid harness test + MOVE destination cid-collision policy (B5).
8. Boss rulings explicitly refreshed: MIG-061 Q3 pure-concatenation principle (B6), plus the already-listed PJ-224/PJ-227.
9. Cross-instance `.conflict` sidecar behavior presented to the Boss as designed behavior (B3).
10. Backfill WAL checkpointing + single-flight vs. watcher + measured boot/typing on the 7,600+ corpus (hard constraint, already stated).

With these ten conditions, Option B is the strongest candidate shape: it is the only design in this space where every federated byte in the active DB is droppable by construction, and where the Boss's "full operations" land in the note's own universe as first-class earned life rather than as a parent-side annotation.

==========================================================================================
## VERDICTS :: C-single-store
==========================================================================================

VERDICT: **NOT VIABLE AS WRITTEN — viable-with-conditions only in an amended (hybrid-FTS) form.** Two unaddressed structural defects (the FTS5 lane, universe identity) break the option's own headline claims; the rest of its risk accounting is honest but incomplete. Every claim below was verified in source this session.

---

## A. FALSE OR OVERSTATED CLAIMS IN THE OPTION (verified against source)

**A1. "Search federates by construction — the implementation of either answer is a WHERE clause" (§2.9) — FALSE. The FTS5 lane cannot be scoped by a WHERE clause. This is the option's biggest hole.**
Three independent breaks, all structural to SQLite FTS5, none fixable by scoping discipline:

1. **`notes_vocab` has no per-universe form.** The Index panel reads `SELECT term, cnt FROM notes_vocab` (`src-tauri/src/libraries.rs:5446-5521`), an `fts5vocab(notes_fts,'row')` aggregate over the ENTIRE index. Under one machine-wide `notes_fts`, the Index panel shows every universe's vocabulary with machine-global counts — including never-federated universes. `fts5vocab('instance')` can't rescue it: `libraries.rs:5761` documents it "has no index on doc, so a SQL-level filter" is a full scan. The Index panel — a canonical Write-Time-Derivation surface per CLAUDE.md — breaks by construction, and fixing it means building a new per-universe term table + triggers, i.e., abandoning the fts5vocab design the panel exists to exploit.
2. **BM25 statistics become machine-global.** `bm25(notes_fts, 10.0, 1.0)` drives search ranking (`search.rs:8841-8854` — deliberately an "index-only bm25 scan, no join") and relatedness scoring (`libraries.rs:6114`, `ORDER BY rank LIMIT 60`). BM25's IDF and average-length statistics come from the whole index: registering universe B **changes result ranking inside universe A** even with perfect row filtering. Today's federation computes bm25 **per schema** and concatenates (`search.rs:8759-8786`) — Option C silently repeals that ranking semantics. This is a cross-universe leak by *statistics* that no WHERE clause can reach.
3. **Top-K dies.** `... MATCH ?1 ORDER BY rank LIMIT 60` is index-only today. Adding a non-FTS `universe_id` predicate forces FTS5 to materialize the full machine-wide match set, then filter, then limit — search latency proportional to total machine corpus, on the hot search path. The only in-index fix (an indexed `universe` column inside the FTS table, prefiltered in MATCH) requires rewriting every MATCH builder AND still pollutes `notes_vocab` with universe tokens.

Consequence: the honest form of Option C is **"one relational store + per-universe FTS/vocab tables"** — a hybrid. That deletes §2.9's "federated search by construction" win and materially weakens the "ONE store" simplification story. The option as written never confronts this.

**A2. "universe_id (or universe_root TEXT)" — presented as cosmetic; it is a first-order gap with data-loss on both branches.**
Verified: `UniverseMeta` (the folder's own `universe.json`, `universe.rs:13-22`) has **no id field** — only name/created/version/children/notes_folder. The only universe identity in the system is the registry entry id, minted per-add as `universe_{uuid}` (`universe.rs:833-834, 1416-1417, 2032-2033`) and **machine-local, destroyed on remove**. So:
- Key by registry id → `remove_universe_from_registry` + re-add mints a new uuid and **orphans every row** of that universe; and `universes.json` becomes load-bearing for all attribution. Its own corruption-recovery code (`universe.rs:105-124`, the 2026-08-02 audit) deliberately falls back to an empty registry because re-adding is free today. Under C, registry loss orphans the entire machine store. A recovery path calibrated for a cheap world becomes a data-catastrophe path.
- Key by root path → moving/renaming a universe folder orphans every row; worse, cUniverse membership is stored as raw absolute path strings (`meta.children`, `universe.rs:1476-1477`), so a *different* universe later placed at the same path inherits the dead one's rows/filters.
- The obvious fix — mint a durable id INTO `universe.json` — recreates the copied-folder problem at universe level (backup-copy a universe, register both: two folders, one id, one store, no disambiguator), which is exactly the collision class the option itself flags for cid_cn. This needs a designed identity + collision protocol; the option doesn't contain one.

**A3. "Identity-keyed cross-universe links" (§2.6) — overstated.** `target_cid_cn` stamping is a **name→cid** lookup: `SELECT cid_cn FROM note_meta WHERE name_lower = ?1 ... LIMIT 1` (`search.rs:8419-8427`) — no universe precedence, no disambiguation. Across a machine store, same-name notes in different universes resolve **arbitrarily** (LIMIT 1), and the option's own schema decision (UNIQUE per `(universe_id, cid_cn)` because copied notes legitimately share cids) concedes cid is NOT a global identity. "Identity-keyed" requires `(universe_id, cid)` plus a resolution-precedence spec (active universe first? federation order? error on ambiguity?) that does not exist in the option. Without it, §2.6 upgrades the same-name backlink-attraction bug from within-universe to machine-wide. (`getBacklinks` name-matching confirmed at `src/lib/libraries/store.ts:5641-5663`.)

**A4. "20 CREATE TRIGGER statements across 3 files" — wrong; ~27 real trigger definitions across 4 files** (`search.rs` ×19, `links_backfill.rs` ×3, `cece/history.rs` ×2 defs, `sight_v6.rs` ×2 — grep this session). The miss under-scopes the trigger work: `sight_v6` layout-invalidation triggers and `note_state_history` triggers also need universe-scoping review.

**A5. "Single-instance guard conflicts with the second-window architecture" — unfounded.** The second screen is a Tauri **window in the same process** (CLAUDE.md: emit/listen cross-window sync); `tauri-plugin-single-instance` blocks a second *process*. No conflict. What the guard DOES foreclose: running two instances to view two universes side-by-side — free today (separate DB files, separate writers), permanently unsafe under C. The option flags the guard as mandatory but not this workflow cost.

**A6. "Crash-mid-move recoverable by replay, not a resume journal" (§2.3) — asserted, not designed.** Cross-universe move requires ledger lines to *leave* the source universe's `earned.jsonl` — but the ledgers are append-only by contract (`link_life.rs:56-58`; `link_life_backfill.rs:42` shows delete-before-reseed is already a special ceremony). "Remove from source" means a tombstone/fold protocol that doesn't exist. A crash after copying lines to the destination but before tombstoning the source leaves the earned data in BOTH ledgers; replay then resurrects earned rows in the source universe (a note that left), and a later move-back double-folds. This is precisely the shape that needs a small resume journal — the option waves it away.

---

## B. HAZARDS THE OPTION UNDERWEIGHTS (verified mechanics)

**B1. Data remanence — a new silent privacy/data-loss class.** Today, deleting a universe folder deletes its `search.db` with it: content copies die with the source. Under C, `note_meta.body_text` (inline, wide — per the PJ-066 comments), the FTS index text, and embeddings of EVERY universe live in `{app_data_dir}/knowledge.db` on C:. A user who deletes a sensitive universe folder in Explorer leaves a complete, searchable copy of its entire content in AppData — indefinitely, silently. The option mentions "store bloat" for unregistering; it never names the remanence problem or the required GC/purge contract for folder-deleted-outside-the-app.

**B2. WAL coupling across universes.** One WAL for the machine. The measured 17s SKY read (233,995 sky_links, ONE universe) is a long read snapshot; under C it pins checkpointing for the whole store while other universes' backfills append (14 backfill families exist; a newly added universe triggers full backfills over the shared writer while the user works elsewhere). Checkpoint starvation + WAL growth were previously a per-universe blast (the 3GB-WAL/vacuum incident); C makes them machine-wide and *coupled* — universe B's backfill degrades universe A's save-reindex latency behind one `busy_timeout=5000` writer (`search.rs:4606-4620`). Today separate files = separate writers; this contention class does not exist.

**B3. Schema-gate blast radius is worse than stated.** The option says recovery is "walk + replay." Verified reality: rebuild also means full FTS retokenization (the Arabic-heavy 7,600-note universe collapses ~452k surface forms — `search.rs:5110-5121` — this is the expensive pass the `note_meta_au` WHEN-gate exists to avoid, `search.rs:5140-5145`) **and full re-embedding**, for EVERY universe on the machine at once — hours of degradation across the user's entire knowledge, not one universe. Plus the two-process rename-aside race: instance A's gate renames `knowledge.db` aside while instance B holds handles (Windows FILE_SHARE_DELETE lets the rename succeed); B keeps writing into the set-aside file; those writes are silently discarded at next boot. C2 replay recovers only the ledger-covered payload.

**B4. The scoping inversion has no structural mitigation available for its hardest cases.** The option proposes "views per federation set" — but FTS5 MATCH cannot go through a view, so the mitigation fails exactly where the leak is worst (search). The realistic mitigation is a mandatory query-builder layer over ~484 read sites — which is itself a migration-scale artifact the effort estimate doesn't include as a named phase.

**B5. `index_search_history` into the universe folder (C2 list) is a privacy mis-allocation** — search queries are user-behavior data; ledgering them into a folder the user syncs/shares leaks them off-machine. Machine-local is the right home; the C2 enumeration didn't think per-item.

---

## C. WHAT THE OPTION GETS RIGHT (that the alternatives likely miss)

1. **The vocabulary-in-DDL discovery is real and important.** Verified: `structural_not_in_clause("NEW.link_type")` is spliced into trigger DDL at `search.rs:5540`, the registry is process-global (`link_types.rs:351`), the backfill gate fingerprints the process snapshot (`links_backfill.rs:88-99`), and 13 splice sites exist across 6 files. ANY multi-universe write design — including Options A/B writing into a foreign universe's DB — hits this (the foreign DB's triggers were generated under the foreign vocabulary, but a cross-universe writer's process registry holds the ACTIVE one). Naming link-vocabulary-to-data as a self-contained prerequisite is a genuine contribution.
2. **C1 rejection and the per-machine (not per-federation-root) scoping are both correct.** Verified: `add_child_universe` (`universe.rs:1453-1489`) checks only validity + not-self — federation is a graph; overlapping roots would double-claim rows. The reasoning holds.
3. **The single-transaction cross-universe move is a real, unique win** — the 13-table census (`mig108.rs:1061-1075`) confirms the earned-row surface, and one-file transactionality genuinely deletes the two-phase journal problem (for the DB half only — see A6 for the ledger half).
4. **Class D death by Rust-side attribution is real** — the PJ-254 comment at `libraries.rs:1436-1438` shows the codebase already fighting exactly the mis-filing class that Rust-side `path→universe→library` resolution kills.
5. **The §4 PJ-262 point is the strategic crux, honestly stated:** the C2 prerequisite it cannot skip simultaneously makes per-universe DBs rebuildable and cross-universe move cheap under Options A/B. After A1 deletes the "federated search by WHERE clause" win and A3 dilutes the identity-keyed-links win, the residual uniquely-C column is: single-transaction moves + attach-layer deletion + Class D death — against the largest migration in project history plus a new machine-wide silent-failure class. The option supplies the evidence for its own demotion and does not hide it. That honesty should be credited.

---

## D. VERDICT AND CONDITIONS

**Not viable as written.** The written form asserts one unified FTS index ("federates by construction") and treats universe identity as a column-naming detail; both are load-bearing and both fail against verified source (A1, A2). An Architect doc built on this text would ship the Index panel broken, search ranking cross-contaminated, and a store whose keys orphan on remove/re-add or folder-move.

**The amended form is viable-with-conditions** — and is honestly a *hybrid*, not "one knowledge store":

1. **C2 ledger completion first** (the option already requires this) — with per-item placement review (B5) and explicit treatment of embeddings/FTS as *recomputable-at-cost*, with the machine-wide recovery cost stated.
2. **A durable universe identity contract**: id minted into `universe.json`, copied-folder collision protocol, registry-loss recovery redesigned for a world where ids are load-bearing (A2).
3. **FTS/vocab stays per-universe** (per-universe `notes_fts`/`notes_vocab` over the shared relational store), OR an in-index universe column with a rewritten MATCH builder + a replacement per-universe term surface for the Index panel + an explicit Boss ruling on global-vs-per-universe BM25 semantics (A1).
4. **A mandatory scoped-query layer** (not "views") as a named migration phase, with an adversarial leak-hunting harness (plant a sentinel universe; assert zero sentinel rows in every surface) — the read-side analogue of the safety-inspection.
5. **Single-instance guard shipped in the same phase as the merge**, with the two-universes-two-windows workflow either explicitly killed by ruling or replaced (A5, B3).
6. **A specified ledger-move protocol** (tombstone/fold records + idempotence proof in the harness) for cross-universe move (A6).
7. **Remanence contract**: unregister/folder-deletion → purge + VACUUM policy, surfaced to the user (B1).
8. **Per-subsystem boot + search-latency measurement on a multi-universe fixture** (≥2 universes, one at 7,600+ notes) before each conversion phase lands — the existing single-universe 7,600-note benchmark no longer covers the constraint (B2).

Given that conditions 1 alone hands Options A/B most of their missing pieces, the Architect should weigh the residual uniquely-C wins (single-transaction move, attach-layer deletion, Class D death) against conditions 2–8 — the honest reading of this option's own §4 is that C is the cleanest *end-state* but the most expensive *path*, and the path's two hardest problems (FTS partitioning, universe identity) were not in its risk register until this attack.

Key evidence: `src-tauri/src/libraries.rs:5446-5521, 5761, 6114`; `src-tauri/src/search.rs:8419-8427, 8759-8854, 5540, 5110-5153, 4606-4620, 11228-11284`; `src-tauri/src/universe.rs:13-38, 96-102, 105-124, 833-834, 1453-1489`; `src-tauri/src/link_types.rs:351`; `src-tauri/src/links_backfill.rs:42, 88-99`; `src-tauri/src/mig108.rs:1061-1075`; `src-tauri/src/link_life.rs:56-58`; `src/lib/libraries/store.ts:5641-5663`; no `single-instance` in `src-tauri` (grep).