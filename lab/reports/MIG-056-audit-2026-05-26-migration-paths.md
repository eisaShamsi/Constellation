# MIG-056 §J Audit — Migration Paths

**Date:** 2026-05-26
**Auditor:** Migration-Path Audit Agent
**Scope:** 12 scenarios covering not-fresh-install federation state-transitions.
**Verdict:** **PASS-WITH-NOTES**

## Headline findings

- **11 of 12 scenarios PASS** at the code-trace level (some need live verification at §K Boss-test).
- **1 P1 finding (Scenario 6):** unmitigated race when the user switches universes during background-attach. Old thread's connection (opened against the OLD universe's `search.db`) can land in the NEW universe's `state.federated_conn` slot. Worth a focused fix before §K Stage 5, OR documenting + accepting + verifying at Boss-test.
- **3 P2 findings (Scenarios 2, 3, 11):** documented v1 limitations — federation is boot-time only; no mid-session refresh on add/remove cUniverse. Already in the Architect; Boss should know to restart Constellation after editing children.
- All `init_db` idempotency, backup/restore, audit-log, lock-probe, and dedupe paths verified safe by reading code + existing tests.

---

## Per-scenario findings

### Scenario 1 — Fresh universe (no cUniverses)
- **Status:** PASS
- **Code path traced:**
  - `ensure_search_db_ready` spawns background thread → `attach_all` runs.
  - `unique_cuniverse_roots(libs, active_root)` walks `libs`, excludes the active root, dedupes.
  - For a universe with no `children` in `universe.json`, `resolve_universe_libraries` returns only the active universe's own libs → `unique_cuniverse_roots` returns empty vec.
  - `attach_all` loop doesn't iterate → `ctx` stays empty → `ctx.set_ready(true)` → write `(ctx, conn)` to state.
- **Resulting state:** `state.federation.attached = []`, `state.federation.warnings = []`, `state.federation.ready = true`, `state.federated_conn = Some(conn_with_no_attaches)`.
- **Consumer behavior:** Lens / status bar / search check `is_ready()` AND iterate `attached()` — empty list = no UNION branches. All consumers fall through to `lexical_search(conn, ...)` against `main` schema only (the §G fast path at search.rs:3668 returns `lexical_search(conn, ...)` when no aliases). MIG-055 active-only behavior preserved.
- **Edge case:** None observed. The empty-vec path is exercised by `attach_all`'s `for` loop trivially. No FederationWarnings emitted.
- **Live-verification needed:** No — covered by integration tests.

### Scenario 2 — Existing universe gains a cUniverse mid-session
- **Status:** PASS-WITH-NOTES (P2 documented limitation)
- **Code path traced:**
  - `universe::add_child_universe(child_path)` at `universe.rs:1047`:
    1. Reads `{active}/.constellation/universe.json`.
    2. Validates child path has `.constellation/universe.json`.
    3. Self-collision check (canonicalize both, reject if equal).
    4. Appends to `meta.children` if not already present.
    5. Writes the JSON back to disk.
  - **No call to `invalidate_search_state`. No re-trigger of `attach_all`.**
  - The next federated query reads the existing `state.federation` / `state.federated_conn` — which were built before the cUniverse was added, so the cUniverse doesn't participate.
- **Resulting state:** `universe.json` on disk includes the new child, but `FederationContext.attached` does NOT include it until Constellation is restarted.
- **Edge case:** None corrupts data. The newly-added cUniverse just isn't federated until next boot. `libraries::invalidate_libraries_cache` is NOT called either, so the sidebar may not show the new cUniverse's libraries until other code paths refresh it.
- **User-visible behavior:** Status bar count + lens results don't update. Architect §3.3 implicitly accepts this (federation runs in background thread once per boot).
- **Recommendation:** Either (a) wire `add_child_universe` to re-trigger background-attach + emit `federation_refreshed` event, OR (b) document in help docs ("Constellation must be restarted after linking a new cUniverse for federation to take effect"). Boss-decision.
- **Live-verification needed:** No — pure code trace. But Boss should be told of the restart-required behavior in §K Stage 5+ tutorial.

### Scenario 3 — cUniverse removed from children mid-session
- **Status:** PASS-WITH-NOTES (P2 documented limitation, no crash risk)
- **Code path traced:**
  - `universe::remove_child_universe(child_path)` at `universe.rs:1080` — symmetric to add. Reads `universe.json`, removes from `meta.children`, writes back. No call to `invalidate_search_state` or `attach_all`.
- **Resulting state:** The cUniverse stays ATTACHed in `state.federated_conn` until app restart. The cUniverse's `.constellation/search.db` remains held by the connection (read-only mode, but file handle live).
- **Edge case to verify:** Does the cUniverse's owner being able to write to it (since we hold a `mode=ro` attach, not a write lock) survive without lock contention? `mode=ro` ATTACH means the federation reader doesn't hold a WRITE lock. The cUniverse's owner (a different Constellation process) can still write under WAL. **No corruption risk.** But the cUniverse's WAL may grow as long as our read-only attach holds an old snapshot — this is an existing WAL-mode quirk, not specific to MIG-056. Acceptable for v1.
- **User-visible behavior:** Lens block keeps showing the departing cUniverse's rows; status bar count keeps including its notes. Until restart, the user sees a "lingering" cUniverse that's no longer in their `universe.json`.
- **Recommendation:** Same as Scenario 2 — either wire remove to re-trigger, or document.
- **Live-verification needed:** No.

### Scenario 4 — Auto-migrate runs on a cUniverse, user later opens it standalone
- **Status:** PASS
- **Code path traced:**
  - Parent boot: `attach_all` → `attach_with_safety` returns `Err("schema_incomplete:...")` → `migrate::run_migrations_on(cu_db_path, parent_root)` → calls `crate::search::init_db(cu_db_path)`.
  - `init_db` (search.rs:1569) is described as idempotent: reads `PRAGMA user_version`, runs only the steps needed to reach `FTS_SCHEMA_VERSION`, stamps `schema_versions` rows (`term_vocab_bridge`, `term_vocab_vacuum`, `term_vocab_dropcol`, etc.) with `INSERT OR REPLACE`.
  - Later: user opens the cUniverse as the active universe. `ensure_search_db_ready` → `init_db(cu_db_path)` again. The stamps mean the migration steps short-circuit (queries like `SELECT version FROM schema_versions WHERE module = 'term_vocab_bridge'` see the value and skip).
- **Resulting state:** No double-migrate. No data loss. The Architect's §5.3 "bail-out" requirement is satisfied by the existing `init_db` idempotency design — no MIG-056-specific code needed for this.
- **Edge case:** The `notes_fts` table's tokenizer-rebuild path (search.rs:1632 `let needs_fts_rebuild = stored_version < FTS_SCHEMA_VERSION`) also only fires on version mismatch — verified idempotent.
- **Live-verification needed:** No.

### Scenario 5 — Two parent universes both linking the SAME cUniverse
- **Status:** PASS
- **Code path traced:**
  - Each parent is a separate boot/active session. Parent A active → `attach_all` opens `?mode=ro` against the cUniverse → cUniverse is attached.
  - User switches to parent B → `invalidate_search_state` drops `state.db`, `state.federated_conn` (sets both to `None`), `state.federation.reset()`.
  - Next federated query → `ensure_search_db_ready` re-spawns background-attach → parent B's `attach_all` opens a fresh `?mode=ro` against the SAME cUniverse → also attached.
  - SQLite supports multiple read-only ATTACHes via WAL — no lock contention. The cUniverse's `search.db-shm` / `search.db-wal` files coordinate read snapshots safely.
- **Resulting state:** No lock contention. No data loss. Each parent has its own `FederationContext` and its own connection pool.
- **Edge case:** If the user has BOTH parent windows open simultaneously (Constellation supports a single active window per process, but a multi-process scenario is theoretically possible), the cUniverse is attached read-only in both → both readers see consistent WAL snapshots. Safe.
- **Live-verification needed:** No (single-process Constellation; the documented v1 architecture).

### Scenario 6 — Universe switch during background-attach
- **Status:** **FAIL — P1, unmitigated race**
- **Code path traced:**
  - **T0:** User has Universe X active. `ensure_search_db_ready` runs.
    - Line 5055: `let app_for_federation = app.clone();`
    - Line 5056: `std::thread::spawn(move || { ... });` — thread captures `app_for_federation`.
    - Inside thread: `let path = db_path(&app_for_federation)?;` reads Universe X's search.db path.
    - Line 5067: `Connection::open(&path)` — opens UNIVERSE X's search.db.
  - **T1 (concurrent):** Before the thread reaches `attach_all`, the user clicks "switch to Universe Y" in the sidebar.
    - `set_active_universe(Y)` runs (universe.rs:546):
      - Updates `UniverseState.active_path = Y`.
      - Calls `invalidate_search_state(app)` — resets `state.db`, `state.federated_conn`, `state.federation`.
    - `ensure_search_db_ready` is called for Universe Y (e.g., on next search), spawns a SECOND background thread for Universe Y.
  - **T2:** The X-thread resumes. Calls `attach_all(&mut conn, &app_for_federation)`.
    - Line 5074 → `attach.rs:128` `attach_all`:
      - Reads `universe::active_universe_dir(app)` — **returns Universe Y** (the new active path, because `app.clone()` shares the `UniverseState` Arc → the lock returns Y).
      - Reads `resolve_universe_libraries(app)` — **returns Y's libs** (including Y's cUniverses).
      - Builds `cu_roots` based on Y's children.
      - **For each of Y's cUniverses, calls `attach_with_safety(conn, ...)` — but `conn` is the connection opened against UNIVERSE X's search.db at line 5067.**
      - Y's cUniverses are ATTACHed to X's main DB.
    - Line 5086: writes `Some(conn)` into `state.federated_conn` — `conn` is X's main DB + Y's cUniverses.
    - Line 5092: writes `ctx` (which describes Y's cUniverses as `attached`) into `state.federation`.
  - **T3 (concurrent):** Y's own background-attach thread also runs and writes its results into the same Mutex slots. Whichever thread finishes LAST wins.
- **Resulting (worst-case) state:** `state.federated_conn` contains UNIVERSE X's main DB connection with UNIVERSE Y's cUniverses attached. Federated queries run UNION ALL over `main.notes_fts` (X's content) + `cu*.notes_fts` (Y's cUniverses' content). The lens block, status bar, and search show a MIX of X's notes + Y's cUniverse notes. User-visible: wrong data on screen, no error surfaced.
- **Why current code doesn't prevent it:**
  - The thread captures `app_for_federation` (an `AppHandle`, which is a clone of the Tauri runtime's `Arc<App>`). The `UniverseState.active_path` lives behind that handle and is read at execution time, NOT at thread-spawn time.
  - There's no generation token / epoch / cancellation channel between `invalidate_search_state` and in-flight federation threads.
  - The thread doesn't verify that the `path` it opened at line 5060 is still the active universe's path before writing into `state.federated_conn`.
- **Fix options (for §H discussion, before §K):**
  1. **Capture the path at thread-spawn time and re-check before write.** Inside the thread, after `attach_all` returns, re-query `active_universe_dir(app_for_federation)` and compare to the path captured at line 5060. If they differ, drop `conn` and bail.
  2. **Generation counter on SearchState.** Increment a `state.federation_generation` AtomicU64 in `invalidate_search_state`; the background thread snapshots the value at spawn and compares before writing. Only writes if generations match.
  3. **Pass the resolved `active_universe_dir` into `attach_all` as a parameter** (instead of `attach_all` re-resolving it from `app`). Then `attach_all` operates on a fixed snapshot, mismatch is impossible — but the wrong cUniverses would still be attached if Y is active by the time `resolve_universe_libraries(app)` is called inside `attach_all` (which still re-reads via `app`). So option 1 or 2 is needed in addition.
- **Probability assessment:** The race window is narrow — the boot-spawn happens once per `ensure_search_db_ready`, and `attach_all` is fast (SQLite ATTACH + 1-2 PRAGMAs per cUniverse). On a fast SSD, the window is ~10-100ms. A user who actively clicks "switch universe" during that window can hit it. In testing it's hard to reproduce; in real-world long sessions it CAN happen, especially if the user double-clicks a universe in the picker.
- **Live-verification needed:** **YES — recommend §K Stage 6 (or Stage 5 sub-step):** rapidly switch universes in the universe picker immediately after boot, then check status bar count. If count doesn't match the active universe's content, the race fired.
- **Recommendation:** Land a Scenario-6 hotfix in §H.X (similar to MIG-055 §H.1) before §K. Option 2 (generation counter) is the cleanest; ~30 lines of code.

### Scenario 7 — Power outage during auto-migrate
- **Status:** PASS-WITH-NOTES (recovery requires manual intervention; documented in code comments but not in user-facing docs yet)
- **Code path traced (migrate.rs:68):**
  - Step 1: lock check — no file changes.
  - Step 2: `fs::copy(cu_db_path, backup_path)` — writes `.pre-mig-056.bak`.
  - Step 3: `init_db(cu_db_path)` opens + migrates the source.
  - **Power-cut points and recovery:**
    - During Step 2 (backup copy in progress): partial backup file exists. Source untouched. Next boot: `attach_with_safety` returns "schema_incomplete" → `run_migrations_on` runs again → `fs::copy` overwrites the partial backup → migration retries from scratch. Safe.
    - During Step 3 (init_db running): source DB partially migrated. SQLite's WAL guarantees that committed transactions survive a power cut, but the migration spans MULTIPLE `execute_batch` calls across many distinct schema-version stamps. After power cut, the source is in an intermediate state. Next boot: `attach_with_safety` checks `note_meta` columns — if the partial migration already added them, attach succeeds and we proceed (init_db idempotency picks up where it left off on next active-universe open). If columns aren't there yet, `run_migrations_on` runs again with the partial source as input — init_db re-runs migration steps that may have already been stamped, but the stamps make them no-ops.
- **Resulting state:** Backup at `.pre-mig-056.bak` remains on disk indefinitely (no GC in v1, by design per migrate.rs:103 comment). User can manually restore via copy-back if anything looks wrong.
- **Recovery path:**
  - Automatic: next boot re-attempts migrate. init_db's idempotency handles partial state.
  - Manual: user copies `search.db.pre-mig-056.bak` back to `search.db`. There's NO documented user-facing recovery procedure for this in `docs/help.*/` — only the code comment.
- **Edge case:** If `init_db` itself crashed mid-migration AND `fs::copy` of the restore also failed (e.g., disk full caused both), we'd hit `MigrationError::BackupRestoreFailed` and the warning surfaces with "CATASTROPHIC: ..." prefix. The user gets the diagnostic in the federation popup. Manual recovery from `.pre-mig-056.bak` works.
- **Live-verification needed:** No — would require artificial power-cut simulation, out of scope for §K. Recovery procedure is documented inline in `migrate.rs` and the variant `MigrationError::BackupRestoreFailed` carries the explanation.
- **Recommendation:** Add a one-paragraph "If you see a CATASTROPHIC migration warning" entry to `docs/help.uConstellation.World/federation.md` (or wherever the federation help topic lands in §L).

### Scenario 8 — Rollback of MIG-056 mid-deploy
- **Status:** PASS
- **Code path traced:**
  - §A (failure types + `FederationContext` struct in mod.rs): public symbols `attach_all`, `FederationContext`, etc. If §A is reverted, the symbols disappear → §B-§L all break compile. Plan's §A rollback ("downstream §B-§L break") is accurate. A whole-MIG revert is needed.
  - §B (`attach::attach_all`): if reverted alone, the `ensure_search_db_ready` background-thread spawn at search.rs:5055-5105 won't compile (refers to `crate::federation::attach_all`). Need to also revert the search.rs hook. Per Plan §6 — "§B-§D: revert individual; upstream steps survive; downstream of revert breaks."
  - §C (`migrate::run_migrations_on`): if reverted, `attach.rs:172` (`super::migrate::run_migrations_on`) won't resolve. Need to also revert that call site. Per Plan §6 — same pattern; with §C reverted the schema-drift path becomes "skip + warn" instead of "migrate then attach," which the Architect §5.3 documents as acceptable degradation.
  - §E/§F/§G consumer adoptions: each touches a different file (lens runtime, status bar query, global search). Independently revertable per Plan §6.
  - §H frontend warning UI: separate file (`FederationWarningsBadge.svelte` or similar). Independently revertable.
- **Resulting state:** Each commit's revert is mechanically clean — git can `revert` each one with predictable conflicts only where adjacent steps share files. Plan §6's rollback claims hold.
- **Edge case:** `state.federated_conn` field in `SearchState` is a struct-shape change. If §B is reverted, the field becomes dead code but doesn't break compile (it's still defined in §A's mod.rs but unused). The integration tests (§I) would still build but skip the federation paths. Consumer code (§E/F/G) would have to be reverted to use single-schema queries. Clean.
- **Audit-log files (`federation-audit.log`) and backup files (`.pre-mig-056.bak`)** remain on disk after a revert. No code change wipes them. Per Plan §6: "Auto-migrated cUniverses' `search.db.pre-mig-056.bak` files remain on disk for manual recovery."
- **Live-verification needed:** No — git-level revert mechanics, not runtime behavior.

### Scenario 9 — cUniverse search.db is in WAL mode + another reader is active
- **Status:** PASS-WITH-NOTES (false-positive risk acknowledged in code)
- **Code path traced (migrate.rs:144 `is_cuniverse_open_elsewhere`):**
  - Opens a fresh `Connection::open(db_path)` — not held; will close on function exit.
  - Sets `PRAGMA busy_timeout = 100` (100ms).
  - Executes `BEGIN EXCLUSIVE; ROLLBACK;`.
    - `BEGIN EXCLUSIVE` requires the EXCLUSIVE lock, which is incompatible with even READ locks held by other connections.
    - If any other connection (in this process OR another process) has an active read lock under WAL, the BEGIN EXCLUSIVE blocks until `busy_timeout` expires, then errors.
  - On error → returns `true` ("locked elsewhere").
- **Why this is correct for the intended scenario:**
  - When another Constellation window has the cUniverse open as its active universe, that window's `state.db` connection is in WAL read mode → BEGIN EXCLUSIVE will fail → we correctly bail.
  - migrate.rs:148-150 acknowledges that WAL + multiple readers can produce false positives — a passive reader (e.g., a stale SHM mapping) MAY cause a false-positive "locked elsewhere" even when no active writer is in flight. The doc comment calls this out as a "best-effort check" acceptable for "v1's expected single-user-single-process pattern."
- **Edge case the comment hints at:** SQLite's `-shm` file persists even after connections close, marking the DB as last-touched-by-WAL. A fresh `Connection::open` on a never-checkpointed WAL DB can briefly need to handle the SHM-mapping handshake. In testing (`is_cuniverse_open_elsewhere_true_for_locked_db`), the test holds an `EXCLUSIVE` lock and confirms detection — this is the worst case the test exercises.
- **Resulting behavior on false positive:** Auto-migrate bails with `MigrationError::CUniverseLocked` → federation warning surfaces → user sees "cUniverse is open in another Constellation window — close it to enable federation." If it's a FALSE alarm, the user is mystified ("but I don't have it open"). They'd need to fully kill all Constellation processes and re-boot. Acceptable for v1 per Architect §5.3 + Boss-lock.
- **Live-verification needed:** **YES — recommend §K Stage 5 sub-step:** open the cUniverse in a second Constellation window, then trigger federation in the parent. Verify the warning appears.

### Scenario 10 — 30+ cUniverses linked
- **Status:** PASS
- **Code path traced (attach.rs:139-153):**
  - `for (i, cu_root) in cu_roots.iter().enumerate()` — iterates ALL cu_roots from `unique_cuniverse_roots`.
  - `if i >= ATTACH_CAP_V1` (= 25) — emits a warning for cu_root, `continue`s to the next iteration.
  - **Does NOT `break`** — keeps emitting warnings for cu_roots 25, 26, 27, ... so the user sees every skipped cUniverse, not just the first.
- **Warning message:** "ATTACH cap reached ({} cUniverses; v1 limit is 25). Federation skipped for this and any subsequent cUniverses." (attach.rs:144-147) — clear + non-fatal.
- **Resulting state:** `ctx.attached.len() == 25`, `ctx.warnings.len() == (cu_roots.len() - 25)`. `ctx.set_ready(true)`. Federated queries proceed with the 25 successfully-attached cUniverses + main. The user sees the warnings in the §H FederationWarningsBadge popup.
- **Edge case:** The SQLite compile-time `SQLITE_MAX_ATTACHED` is currently 10 in the bundled SQLite (per attach.rs:26-30 — the bump to 25 lands in §L). Until then, attempting cu#11 → cu#25 may fail at the SQLite ATTACH layer with "too many attached databases" before reaching the §B's soft cap check. The error becomes a per-cUniverse warning ("ATTACH failed: too many attached databases") — clear diagnostic. **The §B soft cap is design correctness, not yet enforced runtime correctness.** After §L's SQLite rebuild, the runtime cap matches.
- **Live-verification needed:** No (would require synthetic 25+ cUniverses; not realistic for Eisa Universe).

### Scenario 11 — First-time federation on an existing universe with notes
- **Status:** PASS-WITH-NOTES (P2 — Boss should see this for the first time at §K Stage 1-4)
- **Code path traced:**
  - This is the canonical Boss-test scenario. Eisa Universe has 2 cUniverses pre-linked from before MIG-056 shipped. Their `search.db` files were built standalone with the same schema (no drift expected, since Eisa keeps everything on `main`).
  - First boot after MIG-056 ships:
    - `ensure_search_db_ready` for Eisa Universe runs.
    - Background-attach: `unique_cuniverse_roots` returns the 2 cUniverses.
    - Each `attach_with_safety` should succeed (matching schema) → `FederationContext.attached.len() == 2`.
    - `state.federated_conn` gets populated.
  - First federated query (e.g., lens block render):
    - Detects `is_ready() == true` → uses `federated_conn` → UNION ALL across main + cu0 + cu1.
    - Status bar count goes from 1101 → 1101 + Σ(cu counts).
  - User observes the change.
- **Edge case:** If either cUniverse has a schema mismatch (e.g., was built with an older Constellation), `attach_with_safety` returns `schema_incomplete` → `run_migrations_on` runs → §C's safeguards activate. This is the FIRST live exercise of §C, which is the highest-risk surface per Architect §9.3.
- **Live-verification needed:** **YES — Stage 1-4 of §K Boss-test (already planned in Plan §K).** Specifically:
  - Verify status bar count crosses 1101 (Stage 2 in Plan).
  - Verify the lens block shows cUniverse rows (Stage 1).
  - Verify sidebar cUniverse badges show non-zero counts (Stage 3).
  - Verify cross-cUniverse search returns results (Stage 4).
  - If Eisa's cUniverses are schema-current → no auto-migrate path exercised, just attach happy path. **Recommend additionally testing §C on a manually-drift-injected synthetic cUniverse on Boss's machine** to actually exercise §C live before signing off.

### Scenario 12 — cUniverse's search.db is corrupted (not just missing)
- **Status:** PASS
- **Code path traced (attach.rs:163-201):**
  - File exists check (`cu_db_path.exists()`) — true.
  - `attach_with_safety` called → at line 232, `conn.execute(&attach_sql, [])` runs the ATTACH statement.
  - SQLite errors on a corrupt file with "file is not a database" or "malformed database schema" or similar.
  - The `.map_err(|e| format!("ATTACH failed: {}", e))` at line 233 turns the rusqlite error into a string.
  - Returns `Err("ATTACH failed: ...")` to `attach_all`.
  - The match arm at attach.rs:198 `Err(other)` → `ctx.warn(cu_root.clone(), other)`.
- **Resulting state:** `FederationContext.warnings` gets an entry: `cuniverse_path = <cu root>, reason = "ATTACH failed: file is not a database"` (or similar) + timestamp.
- **User-visible:** Status bar warning badge shows "1 cUniverse unavailable" → popup lists the path + reason verbatim. Federation continues with remaining cUniverses (skip_unavailable per §5.2).
- **Edge case:** A CORRUPT file might not match `schema_incomplete: ...` prefix → the `Err(reason) if reason.starts_with("schema_incomplete")` match arm doesn't fire → auto-migrate is NOT attempted → falls through to the generic `Err(other)` arm. This is the CORRECT behavior — auto-migrate would just compound the problem. The user's manual recovery: restore the cUniverse from its own backup, or open the cUniverse standalone in Constellation to trigger `init_db` (which may also fail, but at least is in the cUniverse-owner's domain not the parent's).
- **Live-verification needed:** YES — covered by §K Stage 5 (rename `search.db` to simulate missing). To cover corruption specifically, recommend an additional sub-step: replace one cUniverse's `search.db` with garbage bytes, restart, verify warning surfaces with "ATTACH failed: ..." reason.

---

## Open items for §K Boss-test (live verification)

The following scenarios need live verification beyond code trace:

1. **Scenario 6 (RACE) — P1.** Switch universes rapidly post-boot. Verify status bar count + lens results match the FINAL active universe (no cross-contamination from the in-flight thread). If this fails, ship the generation-counter hotfix before §L.

2. **Scenario 9 (LOCKED CUNIVERSE).** Open the cUniverse in a second window before the parent boots; verify the locked-warning surfaces with a clear message.

3. **Scenario 11 (FIRST-TIME ON EISA UNIVERSE).** Already covered by Stages 1-4. Specifically verify:
   - Status bar count crosses 1101.
   - Lens block tooltip shows cUniverse origin for some rows.
   - Sidebar cUniverse libraries show non-zero star counts.
   - Cross-cUniverse search returns cUniverse-only matches.

4. **Scenario 11 (AUTO-MIGRATE EXERCISE) — additional.** Drift-inject one cUniverse manually (e.g., copy in an older `search.db` from a backup) and verify §C executes the full migrate-then-attach path. Confirm `.pre-mig-056.bak` exists post-migrate. Confirm `federation-audit.log` entry exists with `AUTO_MIGRATE\tresult=OK`.

5. **Scenario 12 (CORRUPT).** Replace one cUniverse's `search.db` with `echo "garbage" > search.db`; verify warning surfaces with clear ATTACH-failed reason.

6. **Scenarios 2 & 3 (RESTART-REQUIRED).** Add and remove a cUniverse mid-session. Verify status bar doesn't update until restart (expected v1 behavior). Document in help docs before §L closes.

---

## Recommended actions before §K signs off

1. **Fix Scenario 6 race** (P1) — add a generation counter to `SearchState` and gate the federated-conn write on a match check. ~30 lines. Land as §H.X or §I.X.
2. **Document Scenarios 2 & 3** (P2) — add a paragraph to `docs/help.uConstellation.World/federation.md` (or equivalent §L help topic) explaining the restart-required behavior.
3. **Document Scenario 7 recovery** (P3) — one paragraph in same help topic for the `.pre-mig-056.bak` manual restore procedure.
4. **Expand §K Stage 5** to cover Scenarios 6, 9, and 12 alongside the existing "rename to simulate missing" test.

---

## Verdict justification

**PASS-WITH-NOTES** rather than PASS:
- 11/12 scenarios cleanly traced through code with documented behavior matching Architect §5.x decisions.
- 1 scenario (#6) has an unmitigated race condition that's narrow but real — should be fixed OR explicitly accepted before Boss-test signs off.
- 3 scenarios (#2, #3, #11) are P2 documented limitations that the user must be informed of, not bugs.

The MIG-056 federation layer is fundamentally sound — the skip_unavailable model, the 4 auto-migrate safeguards, the boot-time-only attach model, and the per-universe SearchState invalidation all hold under realistic conditions. The one race is the kind of thing the §J audit was designed to catch BEFORE §K rather than at §K Stage 5.
