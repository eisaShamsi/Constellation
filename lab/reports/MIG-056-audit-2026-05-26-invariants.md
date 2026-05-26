# MIG-056 §J Audit — Invariants

**Date:** 2026-05-26
**Commits audited:** 6660e00e (§A) through 5c8c06ba (§I)
**Auditor:** Invariants Audit Agent
**Verdict:** PASS

---

## Verification commands run

| Command | Expected | Actual |
|---|---|---|
| `cargo test --lib federation::` | 42 passing | 42 passed; 0 failed |
| `cargo test --lib lens::` | 84 passing (MIG-055 regression check) | 84 passed; 0 failed |
| `cargo test --lib libraries::tests::build_aggregate` | 3 passing | 3 passed; 0 failed |
| `cargo test --lib mig056_federated_search` | 4 passing | 4 passed; 0 failed (verified against committed §I HEAD; uncommitted §J.1 hotfix stashed during verification) |

Total: **133 passing, 0 failing** across the four targeted suites.

---

## Per-invariant findings

### 1. Write-Time Derivation (Architect §3.1)

- **Status:** PASS
- **Evidence:**
  - Federation reads only. All `cu*.note_meta` / `cu*.notes_fts` references across `src-tauri/src/federation/`, `src-tauri/src/search.rs`, `src-tauri/src/libraries.rs`, and `src-tauri/src/lens/sql_builder.rs` are exclusively `SELECT` statements. Grep for `cu\d+\.note_meta|cu\d+\.notes_fts` shows zero `INSERT`/`UPDATE`/`DELETE` against attached schemas in production code (tests only create local fixtures on their own connections).
  - `attach.rs:218` uses `ATTACH DATABASE 'file:{}?mode=ro' AS {}` — URI mode=ro enforces read-only at SQLite level.
  - All `conn.execute*` calls in production code paths under `src-tauri/src/federation/`:
    - `attach.rs:232` — ATTACH (read-only URI)
    - `attach.rs:240` — DETACH (cleanup; no schema write)
    - `attach.rs:246` — `PRAGMA {alias}.cache_size = -512` (parent connection's per-attachment cache tune; does not write to the attached file)
    - `migrate.rs:155, 157` — lock-probe via `BEGIN EXCLUSIVE; ROLLBACK` (probe only; no commit)
    - `migrate.rs:84-93` — `crate::search::init_db(cu_db_path)` against the cUniverse's OWN search.db — the §5.3-authorized auto-migrate path on a separate connection
  - The per-universe `note_meta_ai` / `note_meta_au` / `note_meta_ad` triggers in `init_db` are untouched by federation. No trigger definitions exist under `src-tauri/src/federation/` (grep `CREATE TRIGGER|note_meta_ai|note_meta_ad|note_meta_au` returns no matches).

### 2. Local-First (Architect §3.2)

- **Status:** PASS
- **Evidence:**
  - cUniverses remain self-contained portable indexes. Federation never copies data between DBs — every federated query is a `UNION ALL` across schemas on a single connection (`SearchState.federated_conn`).
  - Removing a cUniverse from `universe.json::children` simply changes what `crate::universe::resolve_universe_libraries` returns (`attach.rs:132`). The cUniverse's `.constellation/search.db` is not modified by removal — no destructive writes are issued by the federation code when a cUniverse is unlinked.
  - `unique_cuniverse_roots` (attach.rs:71-100) operates purely on the libraries list returned by `resolve_universe_libraries`; it does not mutate manifests on disk.
  - Auto-migrate (§5.3) writes only to the cUniverse's OWN `search.db` (via `init_db(cu_db_path)`), with the four safeguards from `migrate.rs`: lock check, pre-migration backup to `search.db.pre-mig-056.bak`, atomic-via-backup-restore, audit log to parent's `.constellation/federation-audit.log`. None of these cross the cUniverse's own boundary.
  - Tests `attach_with_safety_succeeds_on_healthy_cuniverse` and `unique_cuniverse_roots_excludes_active_and_dedupes` lock the boundary behavior.

### 3. Boot perf — no regression (Architect §3.3)

- **Status:** PASS
- **Evidence:**
  - `search.rs:5055-5143` (`ensure_search_db_ready`): the federation work is spawned into `std::thread::spawn(move || { ... })`. The function returns `Ok(())` at line 5145 immediately after spawning — does not block on attach completion.
  - The background thread opens a fresh `Connection` (`Connection::open(&path)` at line 5067) rather than contending with `SearchState.db`, so the main thread's existing search queries are unblocked.
  - All federated query consumers fall back to active-only behavior while attach is in progress:
    - `lens::query::execute_lens` (lens/query.rs:98-99) checks `federation_has_attached(&app)` which gates on `FederationContext::is_ready()` — false during boot → single-schema path
    - `search.rs::federated_lexical_search_or_fallback:3659-3669` — same gate; falls to `lexical_search` if not ready or no attached schemas
    - `libraries.rs::aggregate_library_counts:427-432` — same gate; reads `state.db` single-schema if not ready
  - `attach.rs:246-258` tunes each attached schema's cache_size to -512 (512 KB) per Architect §7.1 to keep per-attachment memory bounded.

### 4. Failure isolation — skip_unavailable (Architect §3.4 + §5.2)

- **Status:** PASS
- **Evidence:**
  - `FederationError` (failure.rs:64-81) has exactly three parent-side variants: `ResolveFailed` (parent's universe.json), `LockPoisoned` (parent's Mutex), `SqlError` (main connection). **No cUniverse-specific variants exist** — comment at failure.rs:62-63 explicitly documents this contract.
  - Per-cUniverse failures funnel through `FederationWarning` (failure.rs:28-44) and `FederationContext::warn` (mod.rs:161-163). Examples:
    - `attach.rs:158-160` — missing `search.db` → `ctx.warn(..., "search.db missing")`, continue
    - `attach.rs:167-201` — schema-incomplete → auto-migrate attempt; failure surfaces as warning, continue
    - `attach.rs:198-200` — any other ATTACH failure → warning, continue
    - `attach.rs:140-152` — ATTACH cap exceeded → warning per over-cap cUniverse, continue
  - The federation continues with the cUniverses that DID attach (`for` loop in `attach_all` never `break`s on per-cUniverse failure — `attach.rs:153` uses `continue`).
  - Test `attach_with_safety_fails_on_missing_required_columns` (attach.rs:387-409) confirms DETACH cleanup on schema check failure — the connection is left in a clean state after a per-cUniverse failure.
  - Integration test `missing_cuniverse_attach_fails_gracefully` (integration_tests.rs) confirms the live skip_unavailable flow.

### 5. Read-only first (Architect §3.5)

- **Status:** PASS
- **Evidence:**
  - ATTACH uses `?mode=ro` URI parameter exclusively (`attach.rs:229`, `integration_tests.rs:75, 469`). No write-mode ATTACH anywhere.
  - All federated query builders produce only `SELECT ... UNION ALL ... [ORDER BY] [LIMIT]` shapes:
    - `federation::query::per_schema_select` (query.rs:73-107) builds `SELECT cols FROM {schema}.note_meta [joins] [WHERE]`
    - `federation::query::union_all_compose` (query.rs:127-148) composes `UNION ALL` over per-schema SELECTs
    - `lens::sql_builder::build_federated_sql` (sql_builder.rs:133-204) — UNION ALL of `build_per_schema_body` parts; outer ORDER BY only
    - `libraries::build_aggregate_counts_sql` (libraries.rs:397-410) — `SELECT library_name, path FROM {schema}.note_meta UNION ALL ...`
    - `search::build_federated_lexical_sql` (search.rs:3779-3796) — `SELECT ... FROM {schema}.notes_fts JOIN {schema}.note_meta ... WHERE MATCH ? UNION ALL ... ORDER BY score LIMIT ?`
  - No `INSERT`/`UPDATE`/`DELETE`/`CREATE`/`ALTER`/`DROP` against attached schemas. Cross-universe writes are out of scope per Architect §8.1.

### 6. Multilingual native (Architect §3.6)

- **Status:** PASS
- **Evidence:**
  - Integration test `multilingual_note_names_round_trip_through_federation` (`src-tauri/src/federation/integration_tests.rs:233-272`) verifies Arabic (`الالتقاطات الأخيرة`), Hebrew (`לכידות אחרונות`), Persian (`گرفت‌های اخیر`), and mixed-script (`Mixed عربي + English`) note names survive the UNION ALL pipeline unchanged. All four asserted via `rows.contains(...)` after `query_map` → `String` deserialization.
  - The federation builders do no string transformation — they only inline schema prefixes (`qualify_expr` in sql_builder.rs:308-311 is a pure `.replace("note_meta", "{schema}.note_meta")` on SQL templates, not on user data).
  - i18n keys verified: all 15 locale files (`src/lib/i18n/*.json`) contain a `federation` block with 4 keys (`warningBadge`, `popupTitle`, `cuniverseLabel`, `reasonLabel`). Frontend `+layout.svelte:6646, 6664, 6671, 6674` uses these via `$t('federation.*')` with English fallbacks.
  - `FederationWarning.cuniverse_path: String` (failure.rs:33) is UTF-8 throughout the IPC chain — `serde::Serialize` derive preserves the bytes; the TypeScript `FederationWarning` interface in `src/lib/federation/store.ts:14-22` is a verbatim mirror.

---

## Notes / observations

1. **Uncommitted §J.1 hotfix in working tree**: `src-tauri/src/search.rs` (and a related `src-tauri/src/lens/query.rs` graceful-fallback change) has uncommitted modifications that add a `federation_generation: AtomicU64` field to `SearchState` for a universe-switch-during-background-attach race (Scenario 6 from the §J migration-paths agent). This is OUT OF SCOPE for the §A-§I commit audit but worth flagging — once committed, it strengthens §3.3 (boot perf) by closing a race where a stale background-attach could write to `federated_conn` after a universe switch. To run the audit cleanly against §I HEAD I stashed those changes during the `mig056_federated_search` test verification.

2. **`schema_alias_is_alphanumeric_safe` test (attach.rs:353-369)**: provides a defensive lock that schema aliases generated by `schema_alias(i)` for 0..25 contain only ASCII alphanumeric chars — combined with `is_safe_schema_alias` (query.rs:155-160) this closes a SQL-injection vector on the schema position even though aliases never come from user input.

3. **Predicate pushdown enforced structurally**: `build_per_schema_body` (sql_builder.rs:210-297) embeds WHERE clauses INSIDE each branch by passing them through `qualify_expr`, not at the outer level. Test `union_all_compose_with_outer_order_and_limit` (query.rs:242-263) asserts WHERE positions are strictly before ORDER BY — locks the contract that prevents the Citus FDW anti-pattern (Architect §7.2).

4. **Audit log integrity**: `migrate.rs:178-208` writes append-only structured lines (tab-separated, RFC3339 timestamp) to `{parent}/.constellation/federation-audit.log` for every auto-migrate attempt. Test `audit_log_creates_dir_and_appends_line` locks the append-not-overwrite behavior. The audit log lives in the PARENT universe (per §5.3 safeguard 4), preserving the principle that the parent records its own cross-universe actions.

---

## Verdict

**PASS** — all six Architect v1.0 §3 invariants hold across commits 6660e00e (§A) through 5c8c06ba (§I). No invariant violations found; no regressions introduced; 133/133 verification tests pass.
