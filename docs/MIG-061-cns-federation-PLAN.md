# MIG-061 — Plan: Federate `cache_boot_snapshot_sky`

**Date:** 2026-05-28
**Architect:** `docs/MIG-061-cns-federation-ARCHITECT.md` (v1.0, locked §8 Q1-Q4)
**Phase:** Plan (Phase 2 of /migration workflow)

---

## Goal (one sentence)

Make `cache_boot_snapshot_sky` aggregate `sky_nodes` + `sky_links` from `main` plus every attached `cu*` schema, so CNS / Sky View / Backlinks / Outgoing Links all see federated data.

## Locked decisions (from Architect §8)

| Q | Pick |
|---|---|
| Q1 | Option 2 — Rust per-schema loop + merge |
| Q2 | Option C — Keep `id = lower(name)`; `path` for disambiguation |
| Q3 | Option B — Federated link resolution across the merged set |
| Q4 | Option A — All-or-nothing readiness gate |

---

## Steps

Each step is one commit with a self-contained verification clause. Build cascades autonomously per Plan-Approval-Equals-Build-Approval; stops only at user-testable verification (§H Boss-test) and at completion (§I PCS).

### §A — Add `get_federated_schemas` helper (1 commit, ~20 lines)

In `src-tauri/src/cache.rs`, add a private helper:

```rust
/// Returns the list of schema aliases to query for federated sky data.
/// Always includes `"main"` first; appends each attached cUniverse alias
/// (`"cu0"`, `"cu1"`, …) in attach order. Empty cUniverse list → returns
/// just `["main"]` (single-universe behavior — identical to pre-MIG-061).
fn get_federated_schemas(app: &tauri::AppHandle) -> Vec<String> {
    let state = app.state::<crate::search::SearchState>();
    let fed = state.federation.lock().unwrap();
    let mut schemas = vec!["main".to_string()];
    if fed.is_ready() {
        for (alias, _path) in fed.attached() {
            schemas.push(alias.clone());
        }
    }
    schemas
}
```

**Verification clause:** `cargo test --lib cache::tests` continues to pass (no behavior change yet — helper not called).

---

### §B — Generalize `read_sky_nodes_raw` to accept a schema (1 commit, ~30 lines)

Rename to `read_sky_nodes_raw_in_schema(conn, schema)` and update the SQL:

```rust
fn read_sky_nodes_raw_in_schema(
    conn: &Connection,
    schema: &str,
) -> Result<Vec<SkyNodeOut>, String> {
    let sql = format!(
        "SELECT id, name, path, library_name, stratum, maturity, origin_type, created_at
         FROM {}.sky_nodes",
        schema
    );
    let mut stmt = conn.prepare(&sql).map_err(/* … */)?;
    // … existing row mapping unchanged …
}
```

Schema name is interpolated AS-IS; safe because §A's helper only yields validated alphanumeric aliases (from `schema_alias(i)` in `federation/attach.rs:61`) or the literal `"main"`.

The existing `read_sky_nodes_raw(conn)` call site in `cache_boot_snapshot_sky` is updated to pass `"main"` so behavior is byte-identical with the pre-MIG-061 path. (Done in this step or §E — whichever's cleaner.)

**Verification clause:** `cargo test --lib cache::tests` passes — 5 existing tests still green; the helper is exercised with `schema="main"` only.

---

### §C — Same generalization for `read_sky_links_raw` (1 commit, ~30 lines)

```rust
fn read_sky_links_raw_in_schema(
    conn: &Connection,
    schema: &str,
    path_to_idx: &HashMap<String, usize>,
    name_to_idx: &HashMap<String, usize>,
    alias_to_path: &HashMap<String, String>,
    nodes_mut: &mut [SkyNodeOut],
) -> Result<Vec<SkyLinkOut>, String> {
    let sql = format!(
        "SELECT source_path, target_name, link_type FROM {}.sky_links",
        schema
    );
    // … same row processing as before …
}
```

Crucially the **same** `path_to_idx`, `name_to_idx`, `alias_to_path` maps are passed in — these are now built across ALL schemas (§E). So a link from `cu0.sky_links` with `target_name="FooBar"` correctly resolves to the FooBar node from whichever schema has it (Q3 lock = Option B federated resolution).

**Verification clause:** `cargo test --lib cache::tests` passes.

---

### §D — Federated readiness gate (1 commit, ~25 lines)

Add a helper that returns `true` only if every schema in the merged set has stamped `sky_schema_version ≥ SKY_SCHEMA_VERSION`:

```rust
fn is_federated_sky_ready(conn: &Connection, schemas: &[String]) -> bool {
    for schema in schemas {
        let sql = format!(
            "SELECT version FROM {}.schema_versions WHERE module = 'sky'",
            schema
        );
        let v: i64 = conn.query_row(&sql, [], |r| r.get(0)).unwrap_or(0);
        if v < crate::search::SKY_SCHEMA_VERSION as i64 {
            return false;
        }
    }
    true
}
```

Used to replace the existing readiness check in `cache_boot_snapshot_sky`. Per Q4 lock (Option A): if ANY schema isn't ready, return `is_ready=false` and frontend falls back to `buildSkyData` legacy path — no partial data.

**Verification clause:** `cargo test --lib cache::tests` passes — new tests added in §G.

---

### §E — Wire it together in `cache_boot_snapshot_sky` (1 commit, ~50 lines)

Replace the single-schema flow:

```rust
// Before:
let conn = open_reader(&app)?;
// readiness check on bare conn
// read_sky_nodes_raw(&conn)
// read_sky_links_raw(&conn, &path_to_idx, ...)
```

with the federated flow:

```rust
// After:
let state = app.state::<crate::search::SearchState>();
let schemas = get_federated_schemas(&app);

// Pick connection: federated_conn if attached cUniverses exist,
// otherwise the bare reader (single-universe optimization).
let conn_guard;
let conn: &Connection;
if schemas.len() > 1 {
    // federated_conn has the ATTACHes already applied
    conn_guard = state.federated_conn.lock().unwrap();
    let Some(c) = conn_guard.as_ref() else {
        // Federation context not ready yet — fall back to single-universe
        // path. is_ready=false → frontend uses buildSkyData legacy.
        return Ok(BootSnapshotSky {
            nodes: vec![],
            links: vec![],
            is_ready: false,
            timings_ms: timings,
        });
    };
    conn = c;
} else {
    let bare = open_reader(&app)?;
    // … existing single-universe path unchanged …
}

// All-or-nothing readiness check (Q4 Option A)
if !is_federated_sky_ready(conn, &schemas) {
    return Ok(BootSnapshotSky { ..is_ready: false, .. });
}

// Loop: collect nodes across all schemas
let mut nodes = Vec::new();
for schema in &schemas {
    nodes.extend(read_sky_nodes_raw_in_schema(conn, schema)?);
}

// Build the path/name/alias maps across the MERGED node set
// (Q3 Option B: federated link resolution)
let mut path_to_idx = HashMap::with_capacity(nodes.len());
let mut name_to_idx = HashMap::with_capacity(nodes.len());
for (i, n) in nodes.iter().enumerate() {
    path_to_idx.insert(n.path.clone(), i);
    name_to_idx.insert(n.name.clone(), i);
}

// Build alias_to_path across all schemas too
let mut alias_to_path = HashMap::new();
for schema in &schemas {
    let sql = format!(
        "SELECT alias_lower, path FROM {}.note_aliases ORDER BY path",
        schema
    );
    // … same logic as before, but loops per schema …
}

// Loop: collect links across all schemas, resolving against the
// MERGED maps
let mut links = Vec::new();
for schema in &schemas {
    links.extend(read_sky_links_raw_in_schema(
        conn, schema, &path_to_idx, &name_to_idx, &alias_to_path, &mut nodes,
    )?);
}

Ok(BootSnapshotSky { nodes, links, is_ready: true, timings_ms })
```

**Determinism for Q3 tiebreak:** since `path_to_idx` and `name_to_idx` are built in schema order ("main" first, then cu0, cu1, …), the **first** node with a given name wins on cross-universe name collision. This is deterministic + matches the federation ordering used by libraryStats / lens / search.

**Verification clause:**
1. `cargo test --lib cache::tests` passes (all old tests, plus new ones from §G).
2. On a single-universe Constellation (no cUniverses), boot timings_ms is within ±10ms of pre-MIG-061 (INV-1).
3. On Eisa Universe (24 cUniverses + 8 751 notes), `cache_boot_snapshot_sky` returns `nodes.len() = 8 751` (or whatever the actual total is — to be verified at Boss-test).

---

### §F — Cross-universe alias resolution edge case (1 commit, ~15 lines)

`note_aliases` is per-schema. After §E, `alias_to_path` is merged across schemas — but a renamed cUniverse note's alias still resolves to its old path (preserving rename history). One subtle case: if cu0 has alias `FooBar → /path/in/cu0/old.md` and cu1 also has alias `FooBar → /path/in/cu1/different.md`, the first-insert-wins rule applies — cu0's alias wins. Document this in the inline comment.

If Boss-test surfaces unexpected behavior here, this step becomes a guard / disambiguation patch.

**Verification clause:** new test `test_federated_alias_collision_deterministic_winner` in `cache::tests` — set up two schemas with conflicting aliases, verify schema-order winner.

---

### §G — Unit tests (1 commit, ~150 lines test code)

Add to `src-tauri/src/cache.rs::tests`:

1. `test_sky_nodes_raw_in_schema_main_only` — schema="main", verifies same result as pre-MIG-061.
2. `test_sky_nodes_raw_in_schema_attached_cu` — set up an ATTACHed in-memory cu0; verify reads from it.
3. `test_federated_sky_ready_single_universe` — only `main`, sky_schema stamped → ready=true.
4. `test_federated_sky_ready_partial_unstamped` — main stamped, cu0 unstamped → ready=false (Q4 Option A).
5. `test_federated_link_resolution_cross_universe` — link in cu0 targets a name that exists in cu1 → resolves correctly (Q3 Option B).
6. `test_federated_link_resolution_within_universe_wins_on_collision` — same name in both cu0 and main; cu0 link to that name resolves to cu0's node (its own universe wins via the path → idx map; not the cross-universe tiebreak).
7. `test_federated_alias_collision_deterministic_winner` (from §F).
8. `test_node_id_uniqueness_lower_name` — Q2 Option C — two notes named "FooBar" in cu0 and cu1; merged result has both rows; consumers must use `path` to disambiguate.

**Verification clause:** all 8 new tests pass; existing 5 cache tests continue to pass; total `cache::tests` = 13 passing.

---

### §H — Boss-test (1 commit, doc only)

Write `docs/MIG-061-BOSS-TEST.md` with stages:

1. **Stage 1 — Single-universe regression.** Open the Eisa Cognitive Knowledge universe (no cUniverses). CNS opens, gravity-well node count matches the universe's note count. Boot time visible in timings strip if any UI surfaces it.

2. **Stage 2 — Federated count.** Switch to Eisa Universe (24 cUniverses). Open CNS. **Header shows `~8 751 nodes`** (or whatever the actual federated total is) instead of 987.

3. **Stage 3 — Cross-universe link resolution.** Find a note in cu0 with a `[[link]]` to a note in cu1 (or use a deliberate test setup). Click the link in CNS or check Backlinks; verify resolution.

4. **Stage 4 — Backlinks panel for a cUniverse note.** Open a note that lives in a cUniverse and is linked from notes across multiple universes; verify the Backlinks panel lists ALL linking notes.

5. **Stage 5 — Outgoing Links panel.** Same note; Outgoing Links shows all wikilinks, including those pointing into other universes.

6. **Stage 6 — Sky View parity.** Switch to Sky View dock button; verify same merged data, same node count.

**Verification clause:** Eisa replies "All pass" or surfaces specific failure modes.

---

### §I — PCS (1 commit set)

After Boss-test passes:
1. **Orientation v2.41** — captures MIG-061 close, updates the audit findings doc to mark P1 surfaces as ✓, advances the four-MIG cascade pointer.
2. **MoCh entry** for today's conversational arc (MIG-060 + audit + MIG-061).
3. **15-locale help-doc updates** — CNS help-doc gets a "shows your full federation" line; Backlinks/Outgoing same.
4. **Milestone tag** `milestone/mig-061-cns-federation-shipped`.
5. **ZIP backup**.

**Verification clause:** new tags pushed, help docs in all 15 locales updated, orientation file dated and version-bumped.

---

## Risks (catalogued for the Audit phase)

| # | Risk | Mitigation |
|---|---|---|
| R1 | `federated_conn` mid-switch race — universe changes while query runs | Existing `federation_generation` AtomicU64 check (MIG-056 §H) — re-use, don't reinvent |
| R2 | Memory blow-up — merged nodes could be huge | Eisa Universe = 8 751 nodes × ~150 bytes each = ~1.3 MB. Acceptable. |
| R3 | `note_aliases` table absent in a cUniverse (older schema) | Skip gracefully — `query_row` with `.unwrap_or(0)` pattern already handles missing table; document the case |
| R4 | Cross-universe link resolution surprises user — wikilink in cu0 silently resolves to cu1's note when both have the same name | Acceptable per Q3 lock; first-insert-wins is deterministic + matches QuickSwitcher behavior |
| R5 | Boot time regression on a 25-cUniverse Universe | Measure timings_ms in §H Stage 2; if >2× single-universe time, escalate. Likely OK because federation read is one statement per schema on a warm conn. |
| R6 | `sky_links` doesn't carry library_name → can't tell which universe a link came from after merge | Not required by current consumers; if a future feature needs it, add a column then. Out of scope. |

---

## Out of scope (deferred to later MIGs)

- **MIG-062 (P3)** — Tag Browser / Five Acts sidebar / Workspace Bases filesystem walks.
- **MIG-063 (P2 read-paths)** — Index, Knowledge Health, Unlinked Mentions, right-sidebar previews.
- **MIG-064 (P2+P4 write-paths)** — Cataloger / Classifier / NSC FK decisions.
- **Org Chart's alias-map federation** — tracked under MIG-063.
- **Performance benchmarking beyond Eisa Universe's scale** — separate effort if/when a 50+ cUniverse setup appears.

---

## Approval gate

Eisa reviews this Plan. On "proceed", Build cascades through §A → §I autonomously. Verification stops only at:
- §H (user-testable Boss-test stages 1-6).
- Genuine architectural surprises uncovered during Build.
- §I completion (close-out summary).

The Standing Order session-log discipline applies between every commit; the Audit phase (3 parallel agents) fires at the end of Build before §I.
