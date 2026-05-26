# MIG-056 §J Audit — Drift

**Verdict:** PASS-WITH-NOTES
**Counts:** P1=1 / P2=5 / P3=4
**Date:** 2026-05-26
**Auditor:** Drift Audit Agent
**Test baseline:** 42/42 federation tests passing (`cargo test --lib federation::`)

---

## Scope verified

- All 7 files under `src-tauri/src/federation/` read in full.
- `src-tauri/src/lens/sql_builder.rs` — `build_federated_sql`, `build_per_schema_body`, `qualify_expr` read.
- `src-tauri/src/lens/query.rs` — `execute_lens`, `federation_has_attached`, `federation_attached_aliases`, `execute_federated_query` read.
- `src-tauri/src/libraries.rs` — `aggregate_library_counts`, `build_aggregate_counts_sql` read.
- `src-tauri/src/search.rs` — `federated_lexical_search_or_fallback`, `federated_lexical_search`, `build_federated_lexical_sql`, `ensure_search_db_ready` federation hook, `invalidate_search_state` federation cleanup read.
- `src-tauri/src/lib.rs` — `federation::federation_get_warnings` command registration confirmed (line 509).
- All 15 locale files have identical `federation` key sets (`cuniverseLabel`, `popupTitle`, `reasonLabel`, `warningBadge`).
- Multilingual integration test exists (`multilingual_note_names_round_trip_through_federation` — Arabic, Hebrew, Persian).

## Audit method outputs

- `unwrap()` / `expect()` / `panic!()` scan: every occurrence in `federation/` is under `#[cfg(test)]` OR is a documented programmer-error guard in `federation/query.rs::per_schema_select` (caller-bug detector). Same for `lens/sql_builder.rs` and `lens/query.rs`.
- `is_safe_schema_alias` is `pub(crate)` and validates `cu0..cu24` shape.
- All four §5.3 safeguards present in `federation/migrate.rs::run_migrations_on` (verified line-by-line: lock check 73-77, backup 80-82, restore 96-132, audit log 99 & 117).
- Predicate-pushdown holds in every federated builder.

---

## Findings (severity-ordered)

### P1 (must fix before §K Boss-test)

**P1-1 — `execute_lens` propagates federation race error instead of falling back.**
File: `src-tauri/src/lens/query.rs` lines 98-118.

The lens path checks `federation_has_attached(&app)` (which acquires `federation` Mutex briefly and returns `true` if ready+attached). Then it calls `execute_federated_query(&app, &built, ...)?` — the `?` operator. If during that call `federated_conn` is `None` (because `invalidate_search_state` ran between the check and the query, or because the background thread crashed after marking `is_ready` true), `execute_federated_query` returns:

```rust
Err("federation: federated_conn is None (background-attach not complete)".to_string())
```

That `Err` propagates out of `execute_lens`, surfacing in the frontend as a lens rendering FAILURE rather than a graceful degradation to single-schema results.

Compare `aggregate_library_counts` (libraries.rs lines 481-507) and `federated_lexical_search_or_fallback` (search.rs lines 3679-3685) — BOTH have explicit fallback to single-schema. `execute_lens` is the odd one out.

**Repro window:** user-initiated universe switch while a lens is being rendered on screen. With `{#key}` re-mounting, the active lens code path could land on the race.

**Fix:** in `execute_lens` line 101-118, wrap the federated branch in a fallback when `execute_federated_query` returns Err:

```rust
let rows = if federation_ready_and_auto {
    let attached_aliases = federation_attached_aliases(&app);
    let mut schemas: Vec<&str> = vec!["main"];
    for alias in &attached_aliases { schemas.push(alias.as_str()); }
    let built: BuiltQuery = build_federated_sql(&def, &allowed_libs, &schemas)?;
    match execute_federated_query(&app, &built, &def, &lib_path_map) {
        Ok(rows) => rows,
        Err(_fed_err) => {
            // Race: federated_conn became None between check and query.
            // Fall back to single-schema (matches §5.2 skip_unavailable).
            let built_fallback = build_sql(&def, &allowed_libs)?;
            let db_path = crate::search::db_path(&app)?;
            let conn = Connection::open(&db_path)?;
            execute_query(&conn, &built_fallback, &def, &lib_path_map)?
        }
    }
} else { /* single-schema path as today */ };
```

---

### P2 (should fix before §L PCS)

**P2-1 — Filesystem-path SQL injection via apostrophe in cUniverse path.**
File: `src-tauri/src/federation/attach.rs` lines 225-231.

```rust
let path_uri = db_path.to_string_lossy().replace('\\', "/");
let attach_sql = format!("ATTACH DATABASE 'file:{}?mode=ro' AS {}", path_uri, alias);
```

`path_uri` is interpolated INSIDE a single-quoted SQL literal. On macOS/Linux (and even Windows), a filesystem path containing an apostrophe (`'`) breaks out of the literal:

- Path: `/Users/x/has'apostrophe/.constellation/search.db`
- Resulting SQL: `ATTACH DATABASE 'file:/Users/x/has'apostrophe/.constellation/search.db?mode=ro' AS cu0`
- The first `'` ends the literal; the rest is parsed as SQL identifiers.

`LibraryInfo.path` is user-controlled (filesystem path of the library registered via libraries.json). Although the attack surface requires the user to create or register a cUniverse with such a path, this is a **trust-boundary** failure — paths are untrusted input.

**Fix:** SQLite's `ATTACH DATABASE` supports parameter binding for the path:

```rust
conn.execute("ATTACH DATABASE ? AS cu0_param_placeholder", params![format!("file:{}?mode=ro", path_uri)])
```

— but the alias position still requires interpolation. Combine parameter binding for the path with the existing `is_safe_schema_alias` check (currently only applied in `per_schema_select`; extend to attach.rs's `attach_with_safety`). Note: `ATTACH ... AS <alias>` doesn't accept a parameter for the alias, so the alphanumeric guard is the proven defense for the alias half.

**P2-2 — `is_safe_schema_alias` not invoked by the three production SQL builders that interpolate aliases.**
Files:
- `src-tauri/src/lens/sql_builder.rs::build_per_schema_body` (line 222-296)
- `src-tauri/src/libraries.rs::build_aggregate_counts_sql` (line 397-410)
- `src-tauri/src/search.rs::build_federated_lexical_sql` (line 3779-3796)

All three interpolate `schema` / `alias` directly into SQL via `format!`. They trust that callers only pass `"main"` + `schema_alias(i)` outputs from `attach.rs::schema_alias` (which is alphanumeric by construction — verified by the §B test `schema_alias_is_alphanumeric_safe`). But none of the three defensively call `federation::query::is_safe_schema_alias` (which is `pub(crate)` and available).

This is a hidden invariant: today's call graph is safe; tomorrow's refactor could pass a user-derived alias and silently inject. The function `is_safe_schema_alias` was deliberately written for this; it should be invoked.

**Fix:** add a defensive `is_safe_schema_alias(schema)` check at the top of each builder. Same panic-on-violation pattern as `per_schema_select` (line 79-84 of query.rs). Promote `is_safe_schema_alias` to fully `pub` or expose a thin wrapper.

**P2-3 — Auto-migrate trigger relies on string-prefix match (`reason.starts_with("schema_incomplete")`).**
File: `src-tauri/src/federation/attach.rs` line 167.

```rust
Err(reason) if reason.starts_with("schema_incomplete") => { ... }
```

The `"schema_incomplete"` prefix is produced by `verify_schema` (line 280-281, 287-291). There's no shared constant, enum, or marker — just a string contract spanning two files. If someone changes the error message format (e.g., to "incomplete_schema:..." for clarity), the auto-migrate path silently no-ops and every drifted cUniverse falls into the generic `Err(other)` warning arm.

**Fix:** introduce an explicit `enum AttachOutcome { Ok, SchemaIncomplete(String), OtherFailure(String) }` returned by `attach_with_safety`, or wrap the prefix in a `const SCHEMA_INCOMPLETE_PREFIX: &str` shared by both producer and consumer.

**P2-4 — `verify_schema` checks only `note_meta` columns; `notes_fts` schema not validated.**
File: `src-tauri/src/federation/attach.rs` lines 266-295.

`REQUIRED_NOTE_META_COLUMNS` (line 49-55) lists 5 columns but only for `note_meta`. The federated FTS5 path (`build_federated_lexical_sql`, search.rs line 3786-3791) queries `{s}.notes_fts` + joins `{s}.note_meta`. If a cUniverse has `note_meta` columns but `notes_fts` is missing or has a wrong schema (e.g., old cUniverse pre-FTS5), the ATTACH succeeds, `verify_schema` passes, and the cUniverse gets added to `attached`. Then `federated_lexical_search` line 3724 fails `prepare()` on the UNION ALL SQL — and the WHOLE federated search returns `Vec::new()` (line 3726).

**User impact:** one drifted cUniverse causes ALL search results to vanish until the cUniverse is fixed/detached. The skip_unavailable model is meant to prevent exactly this.

**Fix:** extend `verify_schema` to also check `notes_fts` exists with the expected shape. If `notes_fts` is missing, treat as `schema_incomplete` (auto-migrate will rebuild it via `init_db`).

**P2-5 — `setTimeout` in `loadFederationWarnings` is unstored and uncleared.**
File: `src/routes/+layout.svelte` lines 2109-2125.

```ts
async function loadFederationWarnings() {
    try { federationWarnings = await getFederationWarnings(); } catch { ... }
    setTimeout(async () => { ... federationWarnings = await getFederationWarnings(); ... }, 3000);
}
```

The `setTimeout` handle is not captured. If the layout component is destroyed (rare — only on full app teardown) OR if `loadFederationWarnings` is called multiple times in rapid succession (universe switches), each call schedules a timeout that survives. The CLAUDE.md Rule 4 (No Memory Leaks) explicitly forbids un-cleared `setTimeout`.

The risk is bounded — each timeout just sets `federationWarnings` once and self-disposes — but the rule is the rule. Calling `loadFederationWarnings()` 4× during rapid universe switching schedules 4 outstanding timeouts, all of which will eventually fire and update state.

**Fix:** store the handle:

```ts
let federationReloadTimer: number | undefined;
async function loadFederationWarnings() {
    try { federationWarnings = await getFederationWarnings(); } catch { federationWarnings = []; }
    if (federationReloadTimer) clearTimeout(federationReloadTimer);
    federationReloadTimer = setTimeout(async () => { ... }, 3000);
}
```

And add `clearTimeout(federationReloadTimer)` to onDestroy.

---

### P3 (nice-to-have for future MIG)

**P3-1 — `qualify_expr` does naive `String::replace` on `note_meta` substring.**
File: `src-tauri/src/lens/sql_builder.rs` lines 308-315.

If a future dimension's `sql_expression` contains the literal string `"note_meta"` inside a string literal (e.g., `WHERE category = 'note_meta_archive'`), the replace would mangle the literal. Doc comment at line 299-307 notes the assumption but doesn't enforce it. Adding a `dimensions.rs` test that pins the allowed table-name set would lock the contract.

**P3-2 — `aggregate_library_counts` reads ALL `(library_name, path)` rows across all schemas.**
File: `src-tauri/src/libraries.rs` lines 397-410 + 461-509.

The Rust-side aggregation walks every note's ancestor directories. For 5 cUniverses × 100K notes = 500K rows + ancestor walks. The status-bar update is "best-effort" (silent failures return empty), but a long aggregation could delay status-bar updates noticeably on huge federations. Architect §7.2 is OK with this (aggregation has no WHERE to push down); a future enhancement could maintain per-library counts at write time (Rule 8 — Write-Time Derivation).

**P3-3 — Audit log writes `cu_db_path.display()` which can contain platform-specific separators OR tab characters.**
File: `src-tauri/src/federation/migrate.rs` lines 193-199.

The audit log format is tab-separated. If a Windows path contains a literal tab character (rare but allowed), it breaks the format. Also the `\` separators on Windows make the log harder to grep across platforms.

**Fix:** normalize the path before logging (`replace('\\', '/')` + escape literal tabs), OR switch to JSONL audit log format.

**P3-4 — `is_cuniverse_open_elsewhere` silently CREATES the cUniverse's `search.db` if it doesn't exist.**
File: `src-tauri/src/federation/migrate.rs` lines 144-161.

`Connection::open(db_path)` creates the file if missing. The next step (backup via `fs::copy`) would then copy this empty file, and `init_db` would initialize it as a fresh DB. The `migration_on_completely_empty_db_runs_full_init` test (line 363-393) confirms this is the intended behavior for empty files. But the side effect is non-obvious — a "lock check" function shouldn't be creating files. If the path is genuinely invalid (e.g., parent directory doesn't exist), `Connection::open` fails and the check returns false (no lock); the backup then fails. End-state is correct, but the implicit file creation is surprising.

**Fix:** add `db_path.exists()` early-return at the top of `is_cuniverse_open_elsewhere` so the lock check is read-only.

---

## Non-findings (explicitly verified clean)

1. **All four §5.3 safeguards present.** Lock check, backup, atomic-via-backup-restore, audit log. Failure paths cross-checked.
2. **`federation::federation_get_warnings` registered in lib.rs invoke_handler.** Line 509.
3. **Predicate-pushdown enforced.** No federated builder accepts an outer WHERE parameter. `union_all_compose` takes only `outer_order` + `outer_limit`. `build_federated_sql`, `build_aggregate_counts_sql`, `build_federated_lexical_sql` all push WHERE inside each branch (or have no WHERE at all in the aggregation case).
4. **Mutex poison handling.** All four federation Mutex accesses (`state.federation`, `state.federated_conn`) match on `Err(_)` and fall back to a safe-empty state — no panics.
5. **Lifecycle hygiene on universe switch.** `invalidate_search_state` (search.rs 4972-4993) drops `federated_conn` AND resets `FederationContext`. The new universe's `ensure_search_db_ready` re-spawns the background-attach thread.
6. **Write-Time Derivation (Rule 8) holds.** Federation reads only, never writes — except `migrate.rs::run_migrations_on`, which is the documented exception under §5.3's safeguards.
7. **15-locale parity.** All locales have `federation.warningBadge`, `federation.popupTitle`, `federation.cuniverseLabel`, `federation.reasonLabel`.
8. **42/42 federation tests pass.** No regressions.
9. **No production `unwrap()`/`expect()`/`panic!`.** All occurrences in `federation/` are inside `#[cfg(test)]` modules or programmer-error guards in `per_schema_select` (caller-bug; not runtime input).

---

## Summary verdict

PASS-WITH-NOTES. The MIG-056 federation cascade is architecturally sound — safeguards intact, predicate-pushdown enforced, lifecycle hygiene correct. One P1 is an inconsistency with two sister fallback paths that already exist (`aggregate_library_counts`, `federated_lexical_search_or_fallback`) — straightforward to align. Five P2s are defensive-coding hardening that would protect against future drift. Four P3s are polish.

§K Boss-test should be deferred until P1-1 is fixed; P2s can land in §L's PCS pass.
