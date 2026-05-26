# MIG-056 — Cross-Universe Federation (Architect v1.0)

**Status:** Architect phase. Pending Boss approval for Plan phase.
**Author:** Claude (in collaboration with Boss, 2026-05-26)
**Predecessor finding:** MIG-055 §I Stage 5 surfaced that `scope.federation: auto` in the lens YAML flows through the parser/validator/SQL-builder correctly, but the runtime layer only sees the active universe's `search.db`. Same architectural gap shows up in the sidebar's library `star_count` badges (0 for all cUniverses) and the status-bar's total-notes figure ("1101 notes" reflects only the active universe).

---

## §1 Context

### §1.1 The Constellation universe model

A Constellation **universe** is a top-level knowledge container — a directory containing:

- `{universe_root}/` — notes, folders, libraries (Obsidian-style flat layout)
- `{universe_root}/.constellation/universe.json` — federation + meta manifest (name, children, etc.)
- `{universe_root}/.constellation/libraries.json` — registered libraries (own libraries + the auto-`universe_notes` pseudo-library pointing at the universe root)
- `{universe_root}/.constellation/search.db` — **SQLite + FTS5 index of THAT universe's notes only**

A **cUniverse** (child universe) is another universe linked to a parent via `universe.json::children`. Each cUniverse is itself a full universe with its OWN `universe.json`, `libraries.json`, and `search.db`.

### §1.2 The current federation surface (what already works)

`crate::universe::resolve_universe_libraries(app)` recursively walks the active universe's children, returning a flattened `Vec<LibraryInfo>` of own + cUniverse libraries. This is the **file-level** federation:
- File tree displays cUniverse libraries
- File operations (read/write/move) work across cUniverse files
- `child.library_count` correctly counts libraries inside cUniverses

### §1.3 The federation surface that DOESN'T work (the gap)

The `search.db` is **per-universe** (`src-tauri/src/search.rs:396-399`):

```rust
pub(crate) fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    Ok(cdir.join("search.db"))
}
```

When the active universe is X and X has cUniverse Y linked:
- X's `search.db` contains `note_meta` rows for X's own libraries only
- Y's notes live in Y's `search.db`, invisible to X's queries

**Consequence:** any code path that queries the active `search.db` to learn about notes (lens results, library `star_count`, status bar total, global search FTS, etc.) silently excludes cUniverse content.

### §1.4 Observable symptoms (the bug surface)

1. **MIG-055 §I Stage 5 failure** — The Observation — Recent Captures lens in Eisa Universe (with 2 cUniverses linked) returned only 1 row from Eisa Universe's own libraries. Expected: rows from Eisa Universe + كون عيسى + Eisa Cognitive Knowledge.
2. **Status bar total notes** — Shows ~1101 for Eisa Universe; should sum across own + cUniverses.
3. **Sidebar `star_count` badges** — `libraryStats.star_count` for cUniverse libraries always renders 0 because their `note_meta` rows aren't in the active `search.db`.
4. **Global search (Ctrl+O / search bar)** — Same root cause; searches only see active universe's notes.

---

## §2 First-Party Authority Chain

Per Boss's standing rule from the MIG-055 §H debug: architectural decisions must rest on **first-party authority** (spec docs, library authors' own guidance, production-scale engineering content) — **not** community plugins.

This Architect's design rests on:

| Authority | What it informs |
|---|---|
| **SQLite official docs** — [ATTACH DATABASE](https://www.sqlite.org/lang_attach.html), [WAL](https://www.sqlite.org/wal.html), [FTS5](https://sqlite.org/fts5.html), [locking](https://www.sqlite.org/lockingv3.html), [limits](https://www.sqlite.org/limits.html), [forum thread on read-only attach](https://sqlite.org/forum/info/e38e9f5032d05736) | ATTACH semantics, FTS5 cross-DB legality, `BEGIN DEFERRED` requirement, `SQLITE_MAX_ATTACHED` limit |
| **Lucene MultiReader / Solr distributed search / Elasticsearch CCS** docs | Scatter-gather coordinator pattern, `skip_unavailable=true` failure model, ranking strategies (BM25 concat → RRF) |
| **PostgreSQL FDW** docs + Citus's "what we learned from FDW" engineering blog | Predicate-pushdown mandate, "don't build a planner" anti-pattern, failure-quarantine guidance |
| **DEVONthink** (official) + **Apple Notes** (Apple Support docs) | Production desktop multi-DB federation pattern; "AI/semantic operations stay per-DB" caveat |
| **Constellation's own codebase** — `note_meta_ai/au/ad` triggers in `init_db`, the existing `ensure_search_db_ready` boot path, `resolve_universe_libraries` | Write-time derivation invariant (CLAUDE.md Rule 8), the existing federation file-layer it builds on |

Notably **excluded** as authority (per Boss correction during MIG-055 §H):
- Community plugins (Dataview, etc.) — too fragile / known issues / not representative of mature architecture

---

## §3 Invariants That Must Not Break

These constraints MUST be preserved across MIG-056. Any design that violates one is rejected.

### §3.1 Write-Time Derivation (CLAUDE.md Rule 8)

> Every computed view in Constellation is maintained at write time, not read time.

Each universe's `search.db` is maintained by triggers (`note_meta_ai`, `note_meta_au`, `note_meta_ad`) on writes within THAT universe. MIG-056 MUST NOT:
- Re-walk universes at boot to rebuild indexes
- Introduce a "first-fill" pass that scans all cUniverses
- Move write-side responsibility (each universe still owns its own writes)

### §3.2 Local-First (CLAUDE.md File Over App)

> `.md` files on disk are the source of truth. The app is just a window. Every universe is a self-contained directory the user can move/back up/sync independently.

MIG-056 MUST preserve:
- Each universe's `search.db` is a self-contained, portable index
- Removing a cUniverse from a parent's `children` list does not corrupt the cUniverse itself
- The parent universe is operable when cUniverses are missing / unmounted / offline

### §3.3 No regression on Boot Time / Typing Latency

> Hard constraint: no new feature may regress boot time, typing latency, or IPC responsiveness.

Cold-attach cost per cUniverse `search.db` is ~tens-to-low-hundreds ms (SQLite mailing list evidence, Agent 1). With up to 25 attached DBs the boot regression could exceed 1-2s if synchronous. MIG-056 MUST:
- Background-attach cUniverses AFTER the main paint
- Active universe's existing search continues working during the background-attach window
- Federated queries gracefully fall back to "active universe only" until the federation layer is ready

### §3.4 Failure Isolation

A cUniverse that is missing / locked / corrupt / schema-drifted MUST NOT break the parent universe's search or rendering. (Boss locked this as `skip_unavailable=true` model — see §5.2.)

### §3.5 Read-Only First (Citus Anti-Pattern Lesson)

Per Citus's published cautionary tale (Agent 3): they tried to make FDW into a generic write-path query layer and ABANDONED it. MIG-056 v1 is **read-only**: federated queries read across multiple universes' `search.db` files, but each universe still owns its own writes.

Cross-universe writes (e.g., a typed link from a note in Universe A to a note in Universe B) are explicitly out of scope for v1. The Living Link architecture would need its own federation MIG if/when cross-universe links become a thing.

### §3.6 The Multilingual / RTL invariant

Constellation supports all languages simultaneously. The federation layer MUST:
- Preserve UTF-8 throughout the SQL → IPC → frontend chain
- Pass Arabic/Persian/Hebrew note names + headlines unchanged (already verified working in MIG-055 §G test 10)
- Not assume LTR row layout in federated results

---

## §4 Design Options

Four options surveyed. Tradeoffs scored. Recommended: Option A.

### Option A — SQLite `ATTACH DATABASE` + `UNION ALL` (RECOMMENDED)

**Shape:** The active universe's SQLite connection ATTACHes each cUniverse's `search.db` at boot time (background). Federated queries are `UNION ALL` across `main` + each attached schema.

**Example:**
```sql
SELECT * FROM main.note_meta WHERE created_at >= ?
UNION ALL
SELECT * FROM cuniverse1.note_meta WHERE created_at >= ?
UNION ALL
SELECT * FROM cuniverse2.note_meta WHERE created_at >= ?
ORDER BY created_at DESC
LIMIT 50;
```

**Pros:**
- Single connection, single query — SQLite's planner handles cross-schema `UNION ALL` well (Agent 1)
- FTS5 `MATCH` works cross-schema for self-contained FTS tables (verified in SQLite docs — content=note_meta in same DB qualifies; Constellation's `notes_fts` qualifies; Agent 1)
- Matches the proven Lucene/Elastic/DEVONthink scatter-gather pattern (Agents 2, 4)
- Each `search.db` keeps its own WAL (per-DB atomicity preserved; Agent 1)
- Failure isolation is natural — missing/locked DBs skip with `SQLITE_CANTOPEN` (Agent 1)

**Cons:**
- `SQLITE_MAX_ATTACHED` default 10 (raisable to 125) — compile-time bump needed (locked at 25 per Boss §5.4)
- Schema-cache churn on DDL — must avoid DDL on the hot path (existing `init_db` is one-shot at boot, so manageable)
- Per-attached-DB page cache — tune `PRAGMA cache_size` down (500KB-1MB each) to avoid bloat
- `BEGIN IMMEDIATE` fails if any attached DB is read-only — MUST use `BEGIN DEFERRED` exclusively (Agent 1, SQLite forum)

**Effort:** Medium. ~2 weeks of focused work including audit + Boss tests.

**Risk:** Medium-low. Each gotcha has a known mitigation. The pattern is broadly proven in production (Simon Willison's Datasette uses ATTACH-into-`:memory:` for cross-DB queries; same primitive).

### Option B — Multi-connection merge in Rust (FALLBACK)

**Shape:** Open each cUniverse's `search.db` in a separate `rusqlite::Connection`. Query each individually. Merge results in Rust.

**Pros:**
- No `SQLITE_MAX_ATTACHED` limit
- Cleaner failure isolation (each connection is independent)
- Sidesteps the `BEGIN IMMEDIATE`-fails-on-read-only-attach issue entirely

**Cons:**
- ~3x more code than ATTACH (each query has parse → execute → merge sub-results plumbing)
- No SQL-level `UNION ALL` / `ORDER BY` / `LIMIT` — Rust has to do the merge + sort + limit (more error-prone, slower for large result sets)
- Loss of single-SQL atomicity (multiple connections see independent points-in-time)

**Effort:** Medium-high. ~3 weeks (more code than A).

**Risk:** Low (each piece is straightforward Rust), but the merge code is where bugs hide.

**Use case:** Keep in pocket as a v2 fallback if Option A's gotchas (cache contention, schema-cache churn) prove worse than expected at scale.

### Option C — Unified federated index (REJECTED)

**Shape:** Migrate from per-universe `search.db` to a single index covering all universes. Each note row has a `universe_id` column.

**Pros:**
- No federation layer needed; queries are normal SQL with `WHERE universe_id IN (...)`
- Globally accurate BM25/IDF scoring

**Cons:**
- **Violates §3.2 (Local-First)** — cUniverses are no longer self-contained portable indexes; removing a cUniverse from the universe.json means orphaning its notes in the central index
- Massive write-path complexity — every cUniverse write has to coordinate with the central index
- Index would grow without bound as cUniverses are added
- One-time migration to merge all existing universes' indexes is itself a multi-week project
- The single index becomes a single point of failure

**Rejected.** Conflicts with the project's foundational architectural principles.

### Option D — Periodic sync (REJECTED)

**Shape:** A background task periodically reads each cUniverse's `search.db` and copies summary rows into the active universe's `search.db` for fast querying.

**Pros:**
- No new federation layer at query time — uses normal SQL
- Failure-isolated (stale data is "ok" temporarily)

**Cons:**
- **Violates §3.1 (Write-Time Derivation)** — introduces a read-time-rebuild pathway exactly like the patterns CLAUDE.md Rule 8 forbids
- Stale data UX: lens shows yesterday's cUniverse content; user adds note in cUniverse, has to wait for sync to see it in parent
- Storage doubling (cUniverse data replicated in each parent that links to it)

**Rejected.** Conflicts with Rule 8.

---

## §5 Boss-Locked Decisions

Locked during Architect phase (2026-05-26):

### §5.1 v1 Federation Scope — ALL FOUR consumers

The federation layer serves these 4 consumers in v1:

| # | Consumer | What it does today | What it must do post-MIG-056 |
|---|---|---|---|
| 1 | **Lens (`execute_lens`)** | Queries active `search.db` only; `scope.federation: auto` is a no-op at runtime | Query active + all cUniverse `search.db`s; YAML's `federation: auto` activates the federated query path |
| 2 | **Status bar total notes** (`$totalStars`) | Sums `star_count` across active universe's libraries | Sum across active + cUniverses (true total notes the user can see) |
| 3 | **libraryStats `star_count` badges** | Per-library note counts; zero for cUniverse libs | Per-library counts include cUniverse libs |
| 4 | **Global search** (Ctrl+O / search bar / `constellation_search_*`) | FTS5 across active universe only | FTS5 across active + cUniverses, with `skip_unavailable` failure handling |

This is broad scope. It means MIG-056's federation layer is a CROSS-CUTTING capability that touches the lens, the status bar, the sidebar, and the global search surfaces. Each consumer will need to adopt the federated query path.

### §5.2 Failure Handling — Skip with surfaced warning (Elastic model)

When a cUniverse is missing / locked / corrupt / schema-drifted at federated query time:
- The federation layer **skips** that cUniverse
- A non-blocking warning bubbles up to the UI (e.g., a small badge in the status bar: "1 cUniverse unavailable" with hover-tooltip details)
- The federated query continues with the cUniverses that ARE available
- Result is marked as "partial" so the caller can decide whether to show a banner

Implementation hook: a `FederationWarning { cuniverse_path, reason }` struct piggybacks on the `LensResult` (and equivalent return types for the other 3 consumers).

### §5.3 Schema-drift handling — Auto-migrate cUniverse on first federated attach

When a cUniverse is below the supported `user_version` floor:
- Constellation runs the missing migrations against the cUniverse's `search.db` BEFORE attaching it for federation
- This is invasive: Constellation writes to a DB belonging to a different universe
- Required safeguards:
  - **Lock check**: ensure no other Constellation process has the cUniverse open (file lock check on `search.db`)
  - **Backup**: copy `search.db` → `search.db.pre-mig-056.bak` before migration
  - **Atomic**: migration runs in a single transaction; failure rolls back
  - **Log**: record the migration in the parent universe's audit log (`.constellation/federation-audit.log`)
  - **Bail-out**: if the user runs Constellation in the cUniverse as the active universe later, the cUniverse already has the migrated schema (no double-migrate)

This is the most architecturally invasive of the 4 decisions. Risks documented in §9.3.

### §5.4 ATTACH cap — Bump to 25 at compile time

`rusqlite`'s `Connection::open_with_flags` uses SQLite's compile-time `SQLITE_MAX_ATTACHED`. Default 10. We bump to 25 via `rusqlite`'s feature flag or via `bundled` SQLite source patch.

Rationale: 10 is too tight for power users with deep federation trees. 125 is overkill and locks in worst-case design considerations. 25 covers any realistic universe network without over-engineering.

Documentation: when a 26th cUniverse is added, the universe.json save succeeds but the federation layer refuses to attach beyond 25 — a warning surfaces and the user is told to either bump the cap (would require a custom build) or restructure their federation.

---

## §6 Architecture — The Federated Search Layer

### §6.1 Module layout

New Rust module: `src-tauri/src/federation/`

```
src-tauri/src/federation/
├── mod.rs            — public API: `federated_connection()`, `FederationContext`
├── attach.rs         — boot-time attach logic; user_version check; auto-migrate hook
├── query.rs          — federated query helpers (UNION ALL builders, schema name resolver)
├── failure.rs        — FederationWarning type; skip_unavailable model
├── migrate.rs        — auto-migration of schema-drifted cUniverses (per §5.3)
└── tests.rs          — in-memory + temp-dir integration tests
```

### §6.2 Boot path — `federation::attach_all`

Called from `ensure_search_db_ready` AFTER the main `init_db(&path)?` returns and the connection is in state.

```rust
pub fn attach_all(conn: &mut Connection, app: &AppHandle) -> Result<FederationContext, FederationError> {
    let libs = crate::universe::resolve_universe_libraries(app.clone())?;
    let cuniverse_paths = unique_cuniverse_roots(&libs);  // dedupe by physical path
    
    let mut ctx = FederationContext::new();
    for (idx, cu_root) in cuniverse_paths.iter().take(25).enumerate() {
        let cu_db = constellation_dir(cu_root).join("search.db");
        if !cu_db.exists() {
            ctx.warn(cu_root, "search.db missing");
            continue;
        }
        let schema_alias = format!("cu{}", idx);  // safe SQL identifier
        match attach_with_safety(conn, &cu_db, &schema_alias) {
            Ok(()) => ctx.add_attached(cu_root, schema_alias),
            Err(e) => ctx.warn(cu_root, &e.to_string()),
        }
    }
    Ok(ctx)
}

fn attach_with_safety(conn: &mut Connection, db_path: &Path, alias: &str) -> Result<()> {
    // 1. ATTACH read-only via URI mode=ro
    conn.execute(
        &format!("ATTACH DATABASE 'file:{}?mode=ro' AS {} KEY ''", 
            db_path.display(), alias),
        []
    )?;
    
    // 2. Read user_version
    let cu_version: u32 = conn.query_row(
        &format!("PRAGMA {}.user_version", alias),
        [], |r| r.get(0)
    )?;
    
    if cu_version < SUPPORTED_FLOOR {
        // Per §5.3 — auto-migrate
        conn.execute(&format!("DETACH DATABASE {}", alias), [])?;
        migrate::run_migrations_on(db_path, cu_version, SUPPORTED_FLOOR)?;
        // Re-attach read-only after migration
        conn.execute(
            &format!("ATTACH DATABASE 'file:{}?mode=ro' AS {} KEY ''",
                db_path.display(), alias),
            []
        )?;
    }
    
    // 3. Tune cache to avoid bloat per Agent 1
    conn.execute(&format!("PRAGMA {}.cache_size = -512", alias), [])?;  // 512KB
    
    Ok(())
}
```

(Sketched, not final code — Plan phase locks the exact signatures.)

### §6.3 Boot timing

```
Existing boot:
  ensure_search_db_ready
    └─ init_db(&path) (main)
    └─ register the connection in SearchState

After MIG-056:
  ensure_search_db_ready
    └─ init_db(&path) (main)
    └─ register the connection in SearchState
    └─ MIG-055 §E init_five_acts_system_notes (unchanged)
    └─ NEW: federation::attach_all (BACKGROUND, after paint)
        └─ for each cUniverse:
            └─ check exists
            └─ ATTACH read-only
            └─ check user_version → auto-migrate if drifted (§5.3)
            └─ tune cache_size
            └─ on any failure: skip + warn (§5.2)
        └─ store FederationContext in SearchState
```

Federated query consumers (lens, status bar, etc.) check `SearchState.federation_ready` flag:
- If ready → run federated `UNION ALL` query
- If not ready (boot still in progress) → fall back to active-universe-only query
- Either way: NO blocking wait

### §6.4 Query shapes per consumer

#### §6.4.1 Lens — `execute_lens`

The `build_sql` function in `src-tauri/src/lens/sql_builder.rs` is updated to consult `FederationContext` when `scope.federation: auto`:

```rust
fn build_federated_sql(def: &LensDefinition, ctx: &FederationContext, ...) -> Result<BuiltQuery> {
    let mut parts = vec![build_per_schema_select(def, "main", ...)];
    if def.scope.federation == FederationMode::Auto {
        for (alias, _) in ctx.attached() {
            parts.push(build_per_schema_select(def, alias, ...));
        }
    }
    // UNION ALL the parts; apply ORDER BY + LIMIT at the outer level
    let inner = parts.join(" UNION ALL ");
    let outer = format!("SELECT * FROM ({}) {}", inner, build_order_clause(...));
    // ...
}
```

Each `build_per_schema_select` call substitutes the schema alias into the joins (`{schema}.note_meta`, `{schema}.note_summaries`, etc.).

#### §6.4.2 Status bar `$totalStars`

`get_all_library_stats(app)` is updated to query `UNION ALL` across schemas, aggregating by `library_name`:

```sql
SELECT library_name, SUM(c) FROM (
    SELECT library_name, COUNT(*) c FROM main.note_meta GROUP BY library_name
    UNION ALL
    SELECT library_name, COUNT(*) c FROM cu0.note_meta GROUP BY library_name
    UNION ALL ...
) GROUP BY library_name
```

Each `LibraryStats` row then gets its true star_count.

#### §6.4.3 libraryStats `star_count` badges

Direct beneficiary of §6.4.2. Once `get_all_library_stats` returns proper cross-universe counts, the badges populate.

#### §6.4.4 Global search — `constellation_search_*` family

The FTS5 path. Each consumer (e.g., `constellation_search`) is updated to `UNION ALL` MATCH queries across schemas:

```sql
SELECT score, path, snippet FROM (
    SELECT rank score, path, snippet(notes_fts, ...) FROM main.notes_fts WHERE notes_fts MATCH ?
    UNION ALL
    SELECT rank score, path, snippet(cu0.notes_fts, ...) FROM cu0.notes_fts WHERE cu0.notes_fts MATCH ?
    UNION ALL ...
) ORDER BY score LIMIT ?
```

V1 uses **naive score concatenation** (Agent 2 recommendation — fine for small N). RRF ranking is a future enhancement.

### §6.5 Frontend integration

Per-consumer frontend changes:

| Consumer | Change |
|---|---|
| `LensBlockWidget` | None — backend `execute_lens` transparently federates |
| Status bar | None — `$totalStars` derives from `libraryStats`; auto-updates when stats federate |
| Library badges | None — derived from `libraryStats` |
| Search bar | None — `constellation_search` IPC transparently federates |

The federation is **invisible to the frontend** — the backend's federated layer changes the rows returned, not the IPC contract.

**One new UI surface**: the `FederationWarning` rollup. A small status-bar element or popup showing "N cUniverses unavailable" when `FederationContext.warnings.len() > 0`. Click → details (which cUniverse, why unavailable, last-tried-at timestamp).

---

## §7 Cross-Cutting Concerns

### §7.1 Boot perf budget

Cold ATTACH ≈ 50-200ms per cUniverse (SQLite mailing list evidence). With 25 max, worst case ~5s synchronous. MUST be background-attached after main paint. Active universe's existing search continues working during the attach window.

### §7.2 Predicate pushdown (Agent 3 lesson)

Federated queries MUST push `WHERE` clauses into each `UNION ALL` branch. Anti-pattern (must avoid):

```sql
-- BAD: pulls all rows from all DBs, filters in outer query
SELECT * FROM (
    SELECT * FROM main.note_meta UNION ALL
    SELECT * FROM cu0.note_meta UNION ALL ...
) WHERE created_at >= ?
```

Correct pattern (each branch filters):

```sql
-- GOOD: each schema filters; only matching rows are unioned
SELECT * FROM main.note_meta WHERE created_at >= ?
UNION ALL
SELECT * FROM cu0.note_meta WHERE created_at >= ?
UNION ALL ...
```

The federated query builder must enforce this at compile time (Rust code that generates SQL).

### §7.3 Multilingual / RTL

All current MIG-055 multilingual fixtures (test 10 — Arabic/Persian/Hebrew note names) must continue to pass after MIG-056. The federated layer adds no string transformations.

### §7.4 Telemetry / observability

`SearchState.federation_audit` — an in-memory ring buffer of the last 100 federation events (attach success, attach skip, migrate run, query partial). Surfaced via a debug Tauri command for diagnostic purposes.

### §7.5 Testability

The federation layer is testable WITHOUT requiring multiple physical universes:
- Create 3 in-memory `:memory:` DBs with the `note_meta` schema
- ATTACH them as cu0, cu1, cu2
- Run UNION ALL queries and assert merged results
- Lifecycle tests: cu0 missing, cu1 schema-drifted, cu2 healthy → verify warnings + partial results

Plan §G locks the exact test set.

---

## §8 Out of Scope for v1

These are explicit non-goals — recorded so future MIG drift doesn't accidentally absorb them:

1. **Cross-universe writes** — Per §3.5. Each universe owns its own writes. Cross-universe typed links wait for a separate MIG.
2. **Cross-universe JOINs** — The federation layer is `UNION ALL` only. Cross-universe JOINs would need ATTACH per-query overhead and complex query rewriting (Agent 3's Citus warning).
3. **AI / semantic operations across universes** — DEVONthink's published lesson (Agent 4): the model degrades on a "search everything" megabase. Aligns with the existing direction in `project_sight_classifier_local_llm.md`. Semantic operations stay scoped to active universe.
4. **RRF (Reciprocal Rank Fusion) ranking** — V1 uses naive BM25 score concatenation per Agent 2's recommendation. RRF is a future enhancement if observable ranking skew appears.
5. **Cross-cluster / network federation** — Federation is local-disk only. No remote cUniverses over network.
6. **Permission model** — All cUniverses linked are fully readable. No per-cUniverse ACLs in v1.
7. **`UNION` (with dedup)** — Always `UNION ALL`. Dedup overhead breaks plans (Agent 1).

---

## §9 Risks Per Locked Decision

### §9.1 Risk: ALL FOUR consumers federate (§5.1)

**Risk:** Wide scope = more places where a federation bug can surface. Each consumer needs careful adoption + testing.

**Mitigation:** Plan §B-§E phase the consumer rollouts. Each consumer ships as its own commit with its own test. Stage 1-4 of §I tests each consumer.

### §9.2 Risk: skip_unavailable warning (§5.2)

**Risk:** Users may not notice the warning bubble → think their data is wrong when really a cUniverse is offline.

**Mitigation:** Status bar warning element is persistent (not a transient toast). Plan §F adds the UI surface with explicit visibility tuning.

### §9.3 Risk: Auto-migrate on attach (§5.3) — HIGHEST RISK DECISION

**Risk surface:**

1. **Concurrent access** — User has the cUniverse open in another Constellation process. Parent's auto-migrate writes while child process is reading. SQLite's WAL allows readers during writes BUT a schema-changing migration may conflict.
   
   *Mitigation:* Lock-file check before migrate. If `{cuniverse}/.constellation/.lock` exists or `search.db-shm` shows active connections, skip migration with a clear warning ("cUniverse X is open in another window — close it to enable federation").

2. **Migration corruption** — Migration fails mid-way; cUniverse's `search.db` is now in a partial state.
   
   *Mitigation:* Pre-migration backup to `search.db.pre-mig-056.bak`. Migration runs in transaction. If anything fails, roll back + restore from backup + skip cUniverse + warn user.

3. **Privilege violation** — Conceptually, writing to a DB owned by another universe is an architectural smell.
   
   *Mitigation:* Audit log entry in PARENT universe records exactly what was done. User can audit. Migration is opt-in (the cUniverse must be in the parent's `children` list — user explicitly federated it).

4. **Recurring drift** — User opens cUniverse standalone, makes changes, re-opens parent. Schema needs re-checking.
   
   *Mitigation:* `attach_with_safety` runs on EVERY attach (every boot or universe-switch), not just first time. The check is cheap.

This decision needs careful Plan-phase design. Worth a dedicated §H.3 audit-step in the Plan.

### §9.4 Risk: ATTACH cap 25 (§5.4)

**Risk:** Power user with 26+ cUniverses hits the wall.

**Mitigation:** Documented limit + clear error message. Compile-time bump path is documented. V2 can raise to 125 if needed.

---

## §10 Migration Path — How MIG-055 Integrates

MIG-055's lens system was correctly designed for the *intent* of federation (`scope.federation: auto`); the runtime layer just couldn't deliver it because the per-universe `search.db` architecture wasn't federated.

When MIG-056 ships:

1. MIG-055's `execute_lens` is updated to consult `FederationContext` (§6.4.1). The YAML's `federation: auto` finally activates a real federation path.
2. MIG-055 §I Stage 5 is re-run as an MIG-056 §I test → expected to PASS with cUniverse rows showing up.
3. The MIG-055 documented limitation ("federation: auto is parser/validator-accepted but runtime is active-universe-only") is removed from the orientation doc.
4. MIG-056 § J PCS subsumes the deferred MIG-055 §J PCS — both ship together as a combined release.

---

## §11 Sources

### First-party docs

- [SQLite — ATTACH DATABASE](https://www.sqlite.org/lang_attach.html)
- [SQLite — Implementation Limits (SQLITE_MAX_ATTACHED)](https://www.sqlite.org/limits.html)
- [SQLite — Write-Ahead Logging](https://www.sqlite.org/wal.html)
- [SQLite — FTS5](https://sqlite.org/fts5.html)
- [SQLite — Locking and Concurrency](https://www.sqlite.org/lockingv3.html)
- [SQLite forum — Read-only attach + BEGIN IMMEDIATE](https://sqlite.org/forum/info/e38e9f5032d05736)
- [PostgreSQL — postgres_fdw](https://www.postgresql.org/docs/current/postgres-fdw.html)
- [Lucene — MultiReader](https://lucene.apache.org/core/8_0_0/core/org/apache/lucene/index/MultiReader.html)
- [Elasticsearch — Cross-Cluster Search](https://www.elastic.co/docs/explore-analyze/cross-cluster-search)
- [Solr — User-Managed Distributed Search](https://solr.apache.org/guide/solr/latest/deployment-guide/user-managed-distributed-search.html)

### Production engineering

- [GitHub — The technology behind GitHub's new code search (Blackbird)](https://github.blog/engineering/architecture-optimization/the-technology-behind-githubs-new-code-search/)
- [Citus — pg_shard and what we learned from our failures](https://www.citusdata.com/blog/2015/09/09/pgshard-learn-from-failure/)
- [Svix — Why Postgres FDW Made My Queries Slow](https://www.svix.com/blog/fdw-pitfalls/)
- [ClickHouse — Postgres FDW: Pushdown is a negotiation](https://clickhouse.com/blog/postgres-fdw-pushdown-negotiation)
- [Notion — Two years of vector search at Notion](https://www.notion.com/blog/two-years-of-vector-search-at-notion)
- [Simon Willison — Cross-database queries in SQLite](https://simonwillison.net/2021/Feb/21/cross-database-queries/)

### Production desktop apps (federation references)

- [DEVONthink — Use Multiple Databases](https://www.devontechnologies.com/blog/tipusemultipledatabases)
- [DEVONthink Community — Search Across Multiple Databases](https://discourse.devontechnologies.com/t/search-across-multiple-databases/13498)
- [Apple Support — Search your notes (per-account toggle)](https://support.apple.com/guide/notes/search-your-notes-not18ab658ed/mac)
- [Obsidian Help — Manage vaults (isolation-first)](https://obsidian.md/help/manage-vaults)

### Constellation internal references

- `src-tauri/src/search.rs:396-399` — current `db_path` (per-universe)
- `src-tauri/src/universe.rs::resolve_universe_libraries` — file-level federation (already works)
- `src-tauri/src/lens/sql_builder.rs` — MIG-055's `build_sql` (will consume `FederationContext`)
- `docs/MIG-055-constellation-base-clean-slate-ARCHITECT.md` §11 #5 — federation auto-default lock (still correct intent)
- `lab/reports/MIG-055-audit-2026-05-26-migration-paths.md` Scenario 7 — flagged as needing live verification (now closed by MIG-056)
- `CLAUDE.md` Rule 8 — Write-Time Derivation invariant (§3.1)

---

## §12 Open Questions for Plan Phase

None at Architect level. All architectural decisions locked. Plan phase will:

1. Decompose §6 into 10–12 buildable commits (one per concern: module skeleton, attach helper, query builders per consumer, failure path, migration helper, tests, UI for warning bubble, etc.)
2. Define verification clauses per commit
3. Schedule the §I Boss-test stages (re-running MIG-055 §I Stage 5 + 4 new federation tests)
4. Schedule the 3-agent audit (invariants / drift / migration-paths)

---

*End of MIG-056 Architect v1.0. Pending Boss approval to proceed to Plan phase.*
