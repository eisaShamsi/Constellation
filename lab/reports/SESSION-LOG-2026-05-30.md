# Session log — 2026-05-30

MIG-066 (Living-Links columns in the Constellation Base). Continuation session resuming at **§A.2** with the Plan already approved (cascade authorized). Model `claude-opus-4-8`.

Context carried in from the handover (`docs/HANDOVER-MIG-066-continuation.md`): §A.1 (commit `10d3caf9`) shipped the write-time engine — `note_meta.outgoing_count / outgoing_link_types / outgoing_top_rank`, the shared recompute SQL (`outgoing_aggregate_assignments`), and the `note_links_outgoing_ai/ad/au` triggers. Existing notes still showed defaults pending the §A.2 back-fill.

## §A.2 — resumable background back-fill + re-index trigger-pause

**Goal:** one-time populate the three columns for pre-existing notes from `note_links`, batched + background, never boot-blocking (the MIG-013 lesson); gated on a new `schema_versions.links_outgoing`.

**The back-fill** — new module `src-tauri/src/links_backfill.rs`, modeled on `sky_backfill.rs` but pure-SQL (no per-note file reads):
- `maybe_schedule` → quick main-thread version-gate check → background thread (returns immediately; wired right after `sky_backfill::maybe_schedule` in `ensure_search_db_ready`).
- Resumable `links_outgoing_backfill_cursor`; 500-note batches with per-batch lock release; one `ANALYZE` so the correlated subqueries hit `idx_link_source` and not the non-selective `idx_link_status` (the exact trap `sky_backfill` documents).
- `recompute_range` reuses the §A.1 `outgoing_aggregate_assignments` (correlated on `note_meta.path`), so the back-fill and the triggers can never drift.
- `finalize` stamps `links_outgoing=1` + clears the cursor atomically.

**Perf gate (Rule 8 / WA#4) — a regression was found and fixed.**
A committed `#[ignore]`d benchmark (`bench_reindex_trigger_overhead`, 7,600 notes / ~216k links, release) isolated the one thing §A.2 can regress: the §A.1 triggers firing per-edge during a full re-index (`reconcile_filesystem` → `index_library_recursive` → `index_note` rebuilds every note's links via per-source DELETE+re-INSERT — **no content-hash skip, so it runs the full walk on every boot reconcile + on demand**).

| Full re-index rebuild (7,600 notes / 216k links) | Time |
|---|---|
| Trigger-free baseline (bulk walk) | ~1.0 s |
| **UNFIXED** — `note_links_outgoing_*` firing per-edge | **~39 s** (≈ +3600 %, O(N²): each edge rescans the source's links) |
| **FIXED** — triggers paused for the walk + one `recompute_all` | **~1.3 s** (bulk ~1.0 s + recompute ~0.25 s) |

**The fix** (handover-named "MIGRATION_ACTIVE-style trigger-pause"), at the clean `reconcile_filesystem` chokepoint (dedicated `walk_conn`, single library loop):
- Extracted the trigger creation into `create_outgoing_link_triggers` (init_db now calls it) + added `drop_outgoing_link_triggers` (both `pub(crate)`, one source of truth).
- `reconcile_filesystem`: `busy_timeout(30s)` → **drop** the 3 outgoing triggers → trigger-free bulk walk → **recreate** them (before the recompute, so a concurrent live save is trigger-covered) → one `links_backfill::recompute_all_outgoing` pass.
- Live single-edge edits keep maintaining the columns write-time (a single save touches ~28 edges → low-ms, on a 1500 ms-debounced save — not on any hot path). SQLite single-writer + `busy_timeout` keep the recompute conflict-free. A crash mid-reconcile self-heals: next boot's `init_db` recreates the triggers (CREATE IF NOT EXISTS) and the next reconcile repopulates.

**Boot / typing / IPC:** unchanged. The back-fill and reconcile are both background threads; `maybe_schedule` returns immediately. Eisa's upgrade does not bump `current_version`, so no forced full re-index on upgrade; the back-fill (direct UPDATEs, ~0.25 s-class) populates the columns shortly after first paint.

**Verification — 5 tests green + benchmark:**
- `links_backfill::tests` — `backfill_populates_existing_rows`, `backfill_is_range_scoped`, `backfill_is_idempotent` (recompute populates pre-existing rows, canonical order + rank sentinel, excludes archived edges, range-scoped, idempotent).
- `search::tests_mig066_outgoing` — `outgoing_aggregates_maintained_by_triggers` (§A.1, still green after the trigger extraction) + new `create_drop_recompute_pause_cycle` (the pause mechanism: triggers maintain → dropped stops maintenance → recreate + recompute_all restores).
- `bench_reindex_trigger_overhead` — the table above.
- `npm run tauri build` — `constellation.exe` + NSIS/MSI built (exit-1 is only the updater `TAURI_SIGNING_PRIVATE_KEY` step, irrelevant to the binary). Re-verify binary mtime before any Boss test.

**Files:** `links_backfill.rs` (new), `search.rs` (gate const + `pub(crate)` SQL + extracted create/drop triggers + reconcile pause + pause-cycle test), `lib.rs` (`mod links_backfill`).

**Note for §G audit (drift/migration-path):** reconcile now drops+recreates the outgoing triggers (schema DDL during a background reconcile); recompute_all is a single un-batched UPDATE over all note_meta (background, post-walk — batch it if very large universes warrant).

**§E scoping note:** a legacy nested `ConstellationEditor/` subtree (and a doubly-nested copy) holds the OLD wrong link types (`related-to`/`prerequisite`/`see-also`/`extends`). Per WA#2 (one location), §E edits ONLY the main `src/` + `src-tauri/src/` trees — never the `ConstellationEditor/` copies.

## §B — register the two Living-Links dimensions (Boss-testable)

Registered `note.outgoing_count` (Number → `note_meta.outgoing_count`) and
`note.link_types` (Text → `note_meta.outgoing_link_types`) in the `dimensions.rs`
`REGISTRY` (both `sortable`, not filterable, no JOIN — plain materialized-column
reads, Rule 8), and added them to the frontend picker (`tableModel.ts`
`ADDABLE_REGISTERED_DIMS` + `REGISTERED_LABELS` with keys `lensBlock.colOutgoingCount`
/ `lensBlock.colLinkTypes`, English fallback "Outgoing links" / "Link types" until
§F localizes all 15 locales). Both sortable (not in `NON_SORTABLE_REGISTERED`);
`link_types`' sort becomes rank-aware in §D.

`discover_keys` reads only `properties_json`, so the new table columns can't leak
into the picker's "Your fields" — they appear only in the Constellation tier.

**Verify:** 15 `lens::dimensions` tests pass (updated `registry_includes_v1_plus_links`
to 6, `registry_iteration_is_stable` to include the two, + new
`link_dimensions_read_materialized_columns_and_sort`). No frontend test pins the
picker list. Boss test pending (columns appear in **+ Add column → Constellation**
and populate). §B cell render is the raw materialized value (count as number,
link_types as the stored canonical string); §C localizes link_types + right-aligns
the count.

## Link-Type Syntax Correction (MIG-066 §B prerequisite)

The §B Boss test on the real universe (7,651 notes) showed **Outgoing links** populating + sorting correctly, but **Link types blank everywhere**. Diagnosed on the live index (`diag_mig066.py`): **every** `note_links.link_type` was `'relates'` (232,668 rows, one value). Root cause — a syntax mismatch, NOT a MIG-066 bug: the backend `extract_typed_links` (search.rs:3614) parsed a predicate-FIRST `[[type::target]]` convention (SMW-style) and defaulted to `'relates'`, while the data + the live-preview editor use predicate-LAST `[[target|display|type]]` (type = last segment). So the index stored `'relates'` for all; the panels worked via an annotation-field workaround. (Memory: [[project_note_links_link_type_relates_bug]].)

**Decision (Eisa): "NEVER accept wrong approaches"** — adopt the field-standard **predicate-first `[[type::target|display]]`** (cross-checked: SMW `[[property::value]]`, Dataview `key:: [[value]]` — both predicate-first; Roam discourse-graph optimizes for natural writing) and **convert the real data**, not patch the parser to read the wrong form.

**Shipped (coordinated so nothing breaks mid-cutover):**
- **Backend** (`search.rs`): rewrote `extract_typed_links` + `parse_link_body` to accept BOTH forms (predicate-first `::`, legacy predicate-last last-segment-as-type) and default untyped to **`associative`** (the canonical null), never `relates`. 6 new `tests_link_parser` (both forms, Arabic, .NET, dedup across forms, empty-target). Stale `relates` comment in `libraries.rs` corrected.
- **Editor** (`livePreview.ts`): added the predicate-first decoration branch (render `[[type::target|display]]` — show target/display in the type color, hide the `type::` plumbing) + fixed the traversal-chip target to strip the `type::` prefix. (`completions.ts`): added **`supersedes`** (was missing — §E vocab drift) in canonical order + the apply now emits `[[type::target|display]]` from the familiar `[[target|type` muscle-memory flow.
- **Converter** (`lab/reports/convert_links.py`, one-time, backed-up by Eisa): scoped to the 17 registered library paths; dry-run-first; idempotent (skips `::`); fenced-code-safe; UTF-8/RTL-safe; display preserved verbatim. **Applied: 644,524 links across 7,512 files** → `type::` form. Residual ~6 type-last links are all inside code fences (`find_remaining.py` confirmed `fence=True`) — correctly left as examples.

**Verify:** 884 lib tests pass (+6 parser); svelte-check = 3 pre-existing errors only (zero new); post-apply re-scan shows 644,526 links now `type::X`. `CodeMirrorEditor.svelte:735` wrong list is **dead code** (only a stale comment references it; not mounted) → §E retire.

## Re-index trigger gap (B2 failure → automatic self-heal)

Boss test after the conversion: **A2 (editor renders converted links) PASS; B2 (Link types column) FAIL** — still blank after 25 min. Diagnosed on the live index (`diag_mig066.py` + `check_mtime.py` + diagnostics.log): `note_links.link_type` was **unchanged** — still 232,668 `'relates'` rows, identical to pre-conversion. Even the OLD parser would extract `supports` from `[[supports::X]]`, so this proved **the app never re-read the converted files**.

Root cause: the **"ZERO BOOT-TIME WALKS"** rule (2026-04-15 perf panel, `+layout.svelte:2067`) removed the on-launch filesystem reconcile. A full re-index now fires ONLY from (a) the live file-watcher (only catches changes while running) or (b) a "Rebuild Index" action *referenced in comments but never built*. The conversion happened while the app was closed → the watcher missed it → no boot re-scan → stale index. The `index_note` mtime gate (search.rs:3887) would have re-indexed (cached `1776186181` ≠ file `1780164917`) — it was just never called.

**Eisa's steer: "Shouldn't the indexing happen automatically?"** — correct. The right fix is auto-detection, not a manual button. **`+layout.svelte`:** restored a **deferred (5s) background** `cache_reconcile` in `initializeApp` (runs on boot AND universe-switch). It fires after the critical path has hydrated — boot-to-interactive is structurally unaffected (the original perf concern) — on its own thread, and re-reads ONLY files whose mtime changed (cheap when nothing did; emits `cache-reconciled` to refresh the snapshot). So Constellation now **self-heals after any external bulk change** (sync, restore, the conversion) with zero user action. `invoke('cache_reconcile')` was called 0× before, so this is a clean re-enable.

**Trade-off (honest):** this re-opens the aggressive zero-boot-walks optimization — every boot now does a cheap background stat-scan (~1–3s, deferred, off the critical path). Justified by the auto-detect requirement. First post-conversion boot does the heavy re-index (7,512 changed files) in the background; subsequent boots find nothing changed → cheap. Perf to confirm on Eisa's relaunch (boot stays instant).

## §B CONFIRMED + per-type counts + recompute hardened

After the auto-reconcile build + overnight: B2 was STILL blank — but `diag_mig066.py` showed `note_links` was now **fully real-typed** (supports 87,922 · derives-from 82,967 · … · 196,009 canonical links). The auto-reconcile worked; the failure was **downstream**: `note_meta.outgoing_link_types` (the materialized column the Base reads) was stale — only 1/7,655 notes populated, Ancient history still `count=525, types='', rank=9` (the OLD back-fill's values from the `'relates'` era). Root cause: my §A.2 `recompute_all_outgoing` was a **single whole-table UPDATE** that silently failed under boot DB lock-contention (the error was `eprintln!`'d, not in diagnostics.log).

**Repair + hardening:**
- **Stopgap** (`recompute_notemeta.py`, batched + lock-tolerant): recomputed `note_meta` from the now-correct `note_links` on the live DB → column populated immediately. Eisa confirmed **§B works** (Ancient history → real types, English + Arabic notes).
- **Per-type counts (Eisa request):** `outgoing_link_types` now stores each type WITH its active count — `"supports (358), contradicts (1), causes (29), exemplifies (36), derives-from (106), part-of (1)"`, canonical order. `outgoing_aggregate_assignments` changed from `GROUP_CONCAT(DISTINCT lt)` to `GROUP_CONCAT(lt||' ('||cnt||')')` over `GROUP BY link_type`. Eisa confirmed perfect.
- **Durable recompute fix:** `recompute_all_outgoing` rewritten **batched (500-row windows) + retry-on-SQLITE_BUSY** — the bug behind the overnight blank. 5 §A tests updated to the count format + green. Built 05:29.

**Lesson:** a swallowed `eprintln!` error on a write-path recompute hid the failure for an overnight. Write-path failures must be loud (diag_log) + the operation lock-tolerant. The chain was THREE bugs — link-type *syntax*, missing *auto-reindex*, fragile *recompute* — each masking the next.

## Open / next
- **Eisa to relaunch** the 05:29 counts build (durable: app produces the count format itself + the recompute self-heals via the batched retry). Current view already correct from the stopgap.
- **§C (localization):** render `outgoing_link_types` localizing each English type id → locale term while KEEPING the ` (count)` (e.g. `يدعم (358)`). Parse on ` (`; the 8 ids are a fixed set.
- **NEW (awaiting Eisa's steer):** "sub-type + sort each" request — clarifying between per-type sortable columns vs. custom link sub-types vs. sort-one-column-by-type. (Eisa's AskUserQuestion click was a misclick; re-asking.)
- Editor A2 caveat: Eisa's screenshot showed raw `[[supports::…]]` (likely active-line/source view; he passed A2) — re-confirm clean badge rendering in live-preview after the re-index.
- Possible follow-up: a manual "Rebuild Index" button (Settings → Index) as a belt-and-suspenders backstop; smarter dirty-check to skip the stat-scan when nothing changed.
- §C/§D/§E/§F/§G per the approved Plan. §D groundwork mapped: rank-aware sort hooks via an optional `sort_expression` on `ResolvedDim`/`DimensionDef` (link_types sorts on the materialized `outgoing_top_rank`); `sql_builder.rs` resolves sort cols at `build_per_schema_body` + outer ORDER BY by ordinal.
- **PJ candidate** (from the §A.2 perf dig): `reconcile_filesystem` / `index_library_recursive` re-index EVERY file on every boot reconcile (no content-hash skip) — all note_links trigger families fire per-edge each time. A "skip unchanged" guard would cut every boot's reconcile cost across the board. Flagged for Eisa.
