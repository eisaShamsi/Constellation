# MIG-061 — Federate `cache_boot_snapshot_sky` (P1 of the Federation Audit)

**Date:** 2026-05-28
**Phase:** Architect (Phase 1 of /migration workflow)
**Status:** Awaiting Boss approval before drafting Plan

---

## Goal

Federate the Sky View / CNS / Backlinks / Outgoing data source so that, in a Universe that contains cUniverses, the gravity well and the backlinks/outgoing panels show **all 8 751 federated notes** instead of just the **987** from the active universe.

One backend fix → four user-visible surfaces unblocked.

---

## §1 — Map the territory

### §1.1 Current data path (parent-only)

```
┌─────────────────────────────────────────────────────────────────┐
│  src-tauri/src/cache.rs:382 — cache_boot_snapshot_sky           │
│    open_reader(&app)  →  Connection on parent's search.db       │
│    read_sky_nodes_raw(&conn)  →  SELECT … FROM sky_nodes        │
│    read_sky_links_raw(&conn)  →  SELECT … FROM sky_links        │
│    Result: { nodes: [..parent only..], links: [..parent only..] }│
└─────────────────────────────────────────────────────────────────┘
                            ↓
        +layout.svelte: skyNodes / skyLinks ($state.raw)
                            ↓
   ┌──────────┬─────────────┬──────────────┬───────────────┐
   ↓          ↓             ↓              ↓               ↓
  CNS      Sky View    Backlinks    Outgoing Links    (others read derived)
```

The bare `Connection` from `open_reader` only sees the parent universe's `search.db`. It never ATTACHes cUniverse databases. So `sky_nodes` / `sky_links` read returns parent-only data, and every downstream consumer inherits the gap silently.

### §1.2 Tables involved

`sky_nodes` (per-universe, populated at boot back-fill):
```sql
CREATE TABLE sky_nodes (
  id TEXT PRIMARY KEY,         -- lower(name)
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  library_name TEXT NOT NULL,
  stratum INTEGER,
  maturity TEXT,
  origin_type TEXT,
  created_at INTEGER
);
```

`sky_links` (per-universe):
```sql
CREATE TABLE sky_links (
  source_path TEXT NOT NULL,
  target_name TEXT NOT NULL,
  link_type TEXT
);
```

Both tables exist in **each cUniverse's `search.db`** as well (every Constellation universe has its own copy). The federation gap is that `cache_boot_snapshot_sky` doesn't union across them.

### §1.3 Existing federation precedent (MIG-056)

The federation primitives that already exist and work:
- **`federated_conn`** (`src-tauri/src/search.rs:411`): a warm `Connection` with each cUniverse's `search.db` already `ATTACH`ed as `cu0`, `cu1`, …, `cu24`. Built by `attach::attach_all`.
- **`federation::query::per_schema_select`** (`src-tauri/src/federation/query.rs:73`): helper that builds `SELECT … FROM <schema>.note_meta` clauses.
- **`federation::query::union_all_compose`** (line 127): joins per-schema parts with `UNION ALL` + optional outer ORDER BY/LIMIT.
- **`schema_alias(i)`** (`src-tauri/src/federation/attach.rs:61`): returns `cu0`, `cu1`, … — validated alphanumeric.

Consumed by libraryStats, search, lens — known to work on Eisa Universe with 24 cUniverses + 8 751 notes.

### §1.4 What the audit confirmed (re-stated)

- CNS gravity well: 987 of 8 751 nodes in Eisa Universe (verified by Eisa via diagnostic log + screenshot).
- Sky View: same data, same gap.
- Backlinks / Outgoing Links: derived from same `skyNodes`/`skyLinks` data → same gap.

---

## §2 — Design options

### Option 1 — SQL UNION ALL via `federated_conn`

Switch `cache_boot_snapshot_sky` to use `state.federated_conn` instead of `open_reader`. Build the SQL via the existing federation helpers:

```rust
// pseudocode
let conn = state.federated_conn.lock().unwrap();
let schemas = std::iter::once("main").chain(cu_schemas.iter()).collect::<Vec<_>>();
let parts: Vec<String> = schemas.iter().map(|schema| {
    format!(
        "SELECT id, name, path, library_name, stratum, maturity, origin_type, created_at \
         FROM {}.sky_nodes",
        schema
    )
}).collect();
let sql = union_all_compose(&parts, None, None);
// run as one prepared statement
```

**Pros:**
- Matches MIG-056 / lens / libraryStats / search precedent — the team has the pattern in muscle memory.
- One SQL roundtrip per scan (not N roundtrips). Same connection-state benefits as libraryStats.
- `union_all_compose` already exists; just need a sky-specific variant of `per_schema_select`.
- Connection is already warm (ATTACHes pre-applied at federation setup).

**Cons:**
- The existing `per_schema_select` is `note_meta`-hardcoded (line 91: `format!("SELECT {} FROM {}.note_meta", ...)`). Need to either generalize the helper or inline the SQL.
- `sky_links` doesn't have a `library_name` column → can't distinguish cross-universe links from same-universe links downstream without joining `sky_nodes` somehow.

**Estimated speed / effort / risk:**
- **Speed:** likely as fast as the current parent-only path on the warm federated_conn (single prepared statement). MIG-058 §K.3 proved per-schema UNION on the warm conn is competitive.
- **Effort:** small. Maybe ~80 lines of new Rust + helper generalization.
- **Risk:** low. Same pattern as libraryStats. Same readiness-gate model.

### Option 2 — Per-schema Rust-side loop + merge

Iterate over schemas in `cache_boot_snapshot_sky`; call `read_sky_nodes_raw` / `read_sky_links_raw` once per schema; concatenate in Rust:

```rust
// pseudocode
let conn = state.federated_conn.lock().unwrap();
let mut nodes = Vec::new();
let mut links = Vec::new();
for schema in std::iter::once("main").chain(cu_schemas.iter()) {
    nodes.extend(read_sky_nodes_raw_in_schema(&conn, schema)?);
    links.extend(read_sky_links_raw_in_schema(&conn, schema, &maps)?);
}
```

**Pros:**
- Minimal change to existing read functions — they already take `&Connection`; just add a `schema` parameter and substitute it into the SQL.
- No need to touch federation/query.rs helpers.
- Each per-schema query is a separate `prepare`, but SQLite caches plans; second-run cost is negligible.

**Cons:**
- N prepare calls instead of 1 — slightly more overhead on first boot (probably <50ms total on a 25-cUniverse setup; negligible after first run).
- Less of a "match the precedent" feel — MIG-056's pattern was the SQL UNION approach.

**Estimated speed / effort / risk:**
- **Speed:** very similar to Option 1. The cost is dominated by row materialization, not SQL prepare.
- **Effort:** smallest. Maybe ~40 lines of new Rust.
- **Risk:** lowest — no new SQL construction patterns, fewer moving parts.

### Option 3 — Materialize a federated `sky_nodes` view in the parent universe

Back-fill copies all cUniverse `sky_nodes` rows into a `federated_sky_nodes` table in the parent's `search.db`. `cache_boot_snapshot_sky` reads only from `federated_sky_nodes` (no ATTACH needed).

**Verdict — REJECTED.** Contradicts MIG-056's chosen pattern (ATTACH, not materialization). Doubles the derivation cost. Adds maintenance burden (parent must be notified when cUniverse changes). Not pursued further.

### Option 4 — Frontend-side N-call federation

Frontend calls `cache_boot_snapshot_sky` once per universe (parent + each cUniverse); merges client-side.

**Verdict — REJECTED.** N IPC roundtrips instead of 1. JS-side merge cost. Boot time scales with cUniverse count. Demonstrably worse than Option 1 or Option 2.

---

## §3 — Invariants that must not break

| # | Invariant | How verified |
|---|---|---|
| INV-1 | Single-universe Universes (no cUniverses) see ≤ +10ms boot time on `cache_boot_snapshot_sky` | Boss-test Stage 2 on Eisa Cognitive Knowledge universe (no cUniverses) — compare timings_ms before/after |
| INV-2 | Federated Universes see ≤ 2× single-universe boot time at the worst case (25 cUniverses) | Boss-test Stage 3 on Eisa Universe — measure timings_ms |
| INV-3 | The frontend `BootSnapshotSky` shape stays unchanged | Type contract — no consumer change |
| INV-4 | CNS, Sky View, Backlinks, Outgoing all render correctly with merged data | Boss-test Stage 4 — verify each surface in Eisa Universe |
| INV-5 | Schema readiness gate still works per-cUniverse | If any cUniverse hasn't stamped sky schema, federation degrades gracefully (see Q4 decision below) |
| INV-6 | Single-universe Sky View regression test passes (5 tests in `cache.rs::tests`) | `cargo test --lib cache::tests` |
| INV-7 | Federation race-condition guard (universe-switch mid-query) still fires | Existing `federation_generation` AtomicU64 check from MIG-056 §H |

---

## §4 — Decision questions for the Boss

### Q1 — Option 1 (SQL UNION ALL) or Option 2 (Rust-side per-schema loop)?

Both are viable. Recommendation: **Option 2** — it's the smallest change, lowest risk, doesn't touch federation/query.rs helpers, and matches the existing `read_sky_nodes_raw` shape (just add a `schema` parameter). Option 1 has a slight aesthetic match with MIG-056's UNION pattern but introduces more moving parts for no measurable speed benefit.

### Q2 — ID uniqueness across cUniverses

Current schema: `sky_nodes.id = lower(name)`. Cross-library name collisions are real ("index.md" exists in multiple libraries; "Eisa ALSHAMSI" might exist in personal-notes and Encyclopedia).

**Option A:** Keep `id = lower(name)` and accept that downstream consumers (CNS's `simNodes.find(n => n.id === ...)`) may match the wrong node. (Current behavior — works correctly within a universe, ambiguous across universes.)

**Option B:** Qualify the merged ID with library_name or universe schema: `id = "cu1:eisa alshamsi"`. Cleaner identity but every consumer would need to update (CNS, Sky View, Backlinks).

**Option C (recommended):** Keep `id = lower(name)` for backward compat; rely on `path` (which IS globally unique — filesystem path) as the disambiguator for lookups that need it. CNS's `selectedNode` setter already uses `path` (we just fixed that in §C-fix). Backlinks/Outgoing use names but accept the collision risk (matches current within-universe behavior).

### Q3 — Cross-universe link resolution

A link in cu0 has `target_name = "FooBar"`. A note named "FooBar" exists in cu1. Should the link resolve **cross-universe** to cu1's FooBar, OR stay **universe-local** (treat as unresolved if not in cu0)?

**Option A:** Stay universe-local. Each cUniverse's links resolve only to nodes in that same cUniverse. Cross-universe wikilinks become "unresolved." (Simpler. Matches the current intra-universe behavior. cUniverses are conceptually self-contained.)

**Option B (recommended):** Federated resolution. After merging all nodes, run the link-resolution pass once across the merged set. A wikilink in cu0 pointing at "FooBar" resolves to whichever universe has it (with a deterministic tiebreak when multiple universes have a "FooBar" — e.g., first-by-universe-index).

**Option B is the user-visible recommendation** — Eisa's whole point of federation is that the universe is intellectually one thing; cross-universe links work in QuickSwitcher search, lens results, etc., so they should also work in CNS.

### Q4 — Readiness gate for partial cUniverse readiness

If parent universe has stamped sky schema (`schema_versions.module = 'sky' >= SKY_SCHEMA_VERSION`) but cu3 hasn't (e.g., a newly-added cUniverse that hasn't run the back-fill), what do we return?

**Option A:** `is_ready = false` until ALL universes (main + cu*) have stamped. Frontend falls back to legacy `buildSkyData` for everyone. (Conservative — never shows partial data.)

**Option B (recommended):** `is_ready = true` if PARENT is ready; cUniverses that haven't stamped yet contribute zero nodes/links. As each cUniverse's back-fill completes, the next `cache_boot_snapshot_sky` call (typically on universe-switch or refresh) picks them up. Partial federation is honest — Eisa sees `8 751 - <unstamped count>` instead of 0 or 987.

---

## §5 — Recommended path

| Question | Recommended answer |
|---|---|
| Q1 — Option | **Option 2** (Rust-side per-schema loop + merge) |
| Q2 — ID | **Option C** (keep `id = lower(name)`; rely on `path` for disambiguation in lookups) |
| Q3 — Link resolution | **Option B** (federated resolution across the merged set) |
| Q4 — Readiness | **Option B** (partial federation — parent-ready is enough) |

If Eisa locks these four, the Plan writes itself: 4-5 commits totaling ~80 lines of Rust + a few test cases.

---

## §6 — Out of scope (explicit non-goals)

- **MIG-062 (P3 — filesystem walks).** Separate MIG.
- **MIG-063 (P2 — other read-path commands).** Separate MIG.
- **MIG-064 (P4 — Cataloger FK + write paths).** Separate MIG. Will need its own Architect to resolve the schema-design question.
- **Org Chart's `load_alias_map` parent-only behavior.** Tracked separately under MIG-063 (P2 family).
- **Performance optimization for >25 cUniverses.** Current `federated_conn` is sized for ≤25 ATTACHes (SQLite default `SQLITE_MAX_ATTACHED = 10` but we bump to 125 at build). If a Universe has 26+ cUniverses, that's a separate problem; out of scope here.

---

## §7 — Approval

Once Eisa picks A/B/C answers for Q1-Q4 (or confirms the §5 recommendation set), the Plan drafts and Build cascades per Plan-Approval-Equals-Build-Approval.

Audit phase: three parallel agents at the end (invariant-checker, drift-detector, migration-path-validator).

---

## §8 — Locks (Boss decisions — 2026-05-28)

Eisa's picks via AskUserQuestion:

| Question | Lock |
|---|---|
| Q1 — Approach | **Option 2** — Rust per-schema loop + merge |
| Q2 — Node IDs | **Option C** — Keep `id = lower(name)`; rely on `path` for disambiguation |
| Q3 — Link resolution | **Option B** — Federated resolution across the merged node set with deterministic tiebreak |
| Q4 — Readiness | **Option A (departed from §5 recommendation)** — All-or-nothing readiness: `is_ready=false` until every schema (`main` + every `cu*`) has stamped sky_schema_version ≥ `SKY_SCHEMA_VERSION`. Conservative; falls back to existing `buildSkyData` legacy path |

Plan drafts next.
