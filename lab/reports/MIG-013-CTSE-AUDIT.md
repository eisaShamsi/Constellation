# MIG-013 — CTSE Bridge Adapter Audit

**Date**: 2026-05-05
**Auditors**: three parallel general-purpose agents per Migration Rule §4 (`/migration` Phase 4).
**Scope**: cumulative diff `e87adeb..HEAD` (32 files, +11,268 / −974 lines).
**Outcome**: **CLOSE WITH MINOR P1 CLEANUP**. Zero P0 findings. Three P1 findings — two cleaned in this commit, one deferred with documentation.

---

## §1. Invariant Audit — PASS (0 / 0 / 0)

| # | Invariant | Result | Notes |
|---|---|---|---|
| 1 | CLAUDE.md Rule 1 — keystroke instant | PASS | `IndexPanel.svelte:134-163` debounce 300 ms + fetchToken cancellation |
| 2 | CLAUDE.md Rule 2 — no `$effect` loops | PASS | Reads/writes disjoint; writes inside `untrack`; cleanup bubbles correctly |
| 3 | CLAUDE.md Rule 3 — no main-thread heavy work | PASS | `ctseSearchTermsByConcept` only called from debounced effect |
| 4 | CLAUDE.md Rule 4 — no memory leaks | PASS | `setTimeout`/`clearTimeout` paired; no orphan `listen()` |
| 5 | CLAUDE.md Rule 8 — write-time derivation | PASS | `term_vocab` maintained via `ctse::hooks`; no `rebuild_*` added |
| 6 | M11 zero-diff invariant | PASS | `git diff e87adeb..HEAD -- src-tauri/src/lexicon/` returns 0 lines |
| 7 | IPC contract — no keystroke IPC | PASS | All `ctse_search_terms_by_concept` calls debounced ≥ 300 ms |
| 8 | Tokenizer symmetry write↔read | PASS | Shared `tokenize_to_vec` + `OnceLock`-cached stopwords. Read-side has no body cap (lemmas are short — documented intentional asymmetry) |
| 9 | Test coverage | PASS | 5/5 ctse tests green: 4 hooks + 1 `concept_lemmas` cross-script sanity |

**Summary.** All nine invariants hold cleanly. The query-time expansion pivot honors every CLAUDE.md performance rule, the M11 zero-diff hard constraint, and the IPC contract. No P0 or P1 findings. Audit recommends MIG-013 close.

---

## §2. Drift Audit — PASS WITH P1 (0 / 2 / 3)

### Findings

**P1-D1 — Dead `settings.index.semanticSearch.*` i18n block in all 15 locale files.**

The MIG-012 "Semantic search" toggle UI was removed from `SettingsModal.svelte` in §1D-B but its i18n keys were left behind. Each locale file carries ~17 keys (label, description, progress phases, indexed/notBuilt strings, rebuild/build/buildConfirm strings, error strings, loadingModel / scanningVocab / optimizingFts5 / tokenizing strings). That's ~255 dead keys total across `en.json` / `ar.json` / 13 partial-coverage locales. Verified zero readers in `src/`.

Action: cleanup commit deletes the entire `settings.index.semanticSearch` block from every locale.

**P1-D2 — Four stale Rust doc comments reference the retired `ctse_run_backfill` / "the backfill" as live writers.**

Locations:
- `src-tauri/src/search.rs:88` — schema-version comment for `bridge_concept_id` v1 says "Populated by `ctse::hooks::on_note_indexed` (fast path) and the `ctse_run_backfill` Tauri command (slow path)".
- `src-tauri/src/search.rs:91-97` — v2 schema comment describes "the backfill" as the consumer of bigram-sentinelled rows.
- `src-tauri/src/search.rs:455-456` — `ensure_term_vocab_bridge_column` doc says "population happens lazily via the write-time hook and the explicit `ctse_run_backfill` command".
- `src-tauri/src/search.rs:483-498` — `sentinel_bigram_rows` doc says "post-fill backfill which now targets only the ~50K real stems".

All four describe code paths that no longer exist (Option B retired the entire backfill / first-fill subsystem). They mislead a future reader about the column lifecycle.

Action: cleanup commit rewrites all four to reflect dead-schema status.

**Notes (forward-compat, sanctioned).**

- `term_vocab.bridge_concept_id` column + index + v1 / v2 schema migrations: kept as dead schema. The v2 sentinel migration runs once on existing DBs (~5.7M rows on Boss's library) but writes a value nothing reads. This is the documented Option B trade-off — the column survives in case a future "deep concept tagging" feature wants to populate it.
- `index.semanticSearchEnabled` setting flag in `DEFAULT_SETTINGS`: kept as forward-compat. Zero readers. Documented in `SettingsModal.svelte:44-52`.
- `searchHub.concept` / `searchBadges.concept` i18n keys: kept after the SearchHub `concept` category was reverted in §1D-D. Zero callers. Per the §1D-D commit's "kept for now" note. Decision deferred — confirm intent on next clean-pass.
- Orphaned `term_embeddings` table on existing DBs: tens of MB of dead disk on pre-MIG-013 universes. CLEAR-on-next-cleanup-migration candidate.

### Drift summary

The MIG-013 retirement landed cleanly across the IPC boundary, frontend store, and Rust backend. Zero P0 drift: no broken imports, no orphaned `invoke()` strings, no readers of dead schema, no doc surfaces describing retired UI. The two P1s above are mechanical 5-minute cleanups applied in the close-out commit.

---

## §3. Migration-Path Audit — PASS WITH P1 (0 / 1 / 2)

### Scenario walkthroughs

**Scenario 1 — Fresh install on empty Universe → PASS.**
`init_db` creates `term_vocab` cold (3-column schema). `ensure_term_vocab_bridge_column` adds the column + index. `sentinel_bigram_rows` UPDATE matches zero rows (no-op). Stamp lands at v2. `ctse_search_terms_by_concept` on empty `term_vocab` returns empty Vec without throwing.

**Scenario 2 — Pre-MIG-013 DB (Boss-equivalent ~5.7M rows) → PARTIAL with P1-M1.**

The bigram-sentinel UPDATE has to scan ~5.7M rows, evaluate `term LIKE '%' || CHAR(31) || '%'` (no index — leading wildcard), and write `'-'` to ~5.68M of them inside one implicit transaction. Wall-clock: tens of seconds on commodity SSD, multiple minutes on slower disk. The boot path is fully blocking — no progress UI, no status-bar message. The user sees a frozen splash for the duration.

The migration is correct and one-shot; subsequent boots short-circuit on the stamped `schema_versions.term_vocab_bridge = 2`. After the migration completes, IndexPanel filter, `≈ similar` cross-language, and all read paths work because none of them read `bridge_concept_id`.

**P1-M1 — boot-time freeze on pre-MIG-013 DBs is unreported.** Mitigations to consider:
1. Chunk the UPDATE into N batched UPDATEs with `LIMIT … OFFSET …`, emitting a Tauri event between chunks.
2. Wrap in `BEGIN IMMEDIATE` + emit a "migration in progress" event before the UPDATE.
3. At minimum: a pre-UPDATE `diag_log` line emitting the matching-row count so the freeze is diagnosable from logs.

**Decision: deferred to follow-up MIG.** Boss's library has already completed the migration on the §1D-D binary; this is a one-time pain. New users with pre-MIG-013 backups would hit it. The fix is non-trivial but bounded; ship in a focused mini-MIG before any v1.0 release. Memory entry queued: `project_mig013_v2_migration_blocking_boot.md`.

**Scenario 3 — Mid-migration interrupt → PASS.**
rusqlite + SQLite WAL atomicity: the `conn.execute("UPDATE …")` runs as an implicit autocommit transaction. Power loss / `kill -9` mid-write leaves either fully-committed or fully-rolled-back state. The schema-version stamp lands strictly *after* the UPDATE returns. Re-entering on a partial state finds no stamp, re-runs `ensure_term_vocab_bridge_column` (idempotent — probe-then-ALTER), re-runs `sentinel_bigram_rows` (idempotent — `WHERE bridge_concept_id IS NULL` filter skips already-sentinelled rows). Converges cleanly.

**Scenario 4 — Rollback to pre-MIG-013 binary → PASS with Note.**
Old binary opens DB at `term_vocab_bridge = 2` with the wider schema. SQLite tolerates the extra column transparently. The old `init_db` doesn't reference `term_vocab_bridge`, so no errors. Old `init_term_embeddings` re-runs against `term_embeddings` (still on disk on existing DBs); UX regresses to MIG-012 cross-language behavior. Newer notes added since §1C are missing from the stale `term_embeddings` index — search degrades but doesn't fail. No DB corruption. Acceptable.

### Migration-path summary

The MIG-013 schema migration is mechanically sound across all four scenarios. Fresh installs converge to the wide schema with zero matched sentinel rows; pre-MIG-013 DBs converge via idempotent column-add and one-shot bulk UPDATE; mid-migration interrupts recover cleanly via stamp-after-UPDATE ordering + SQLite WAL atomicity; rollback is non-destructive but degrades cross-language search to MIG-012 quality. **The single deferred P1 is the unreported boot freeze on Boss-equivalent libraries** — fix queued in a focused follow-up MIG.

---

## §4. Closing actions

### Applied in close-out commit

1. **P1-D1 fixed**: deleted the `settings.index.semanticSearch` block from all 15 locale i18n files. Net ~255 dead keys removed.
2. **P1-D2 fixed**: rewrote the four stale Rust doc comments in `src-tauri/src/search.rs` (lines 88, 91-97, 455-456, 483-498) to reflect post-§1D dead-schema status.

### Deferred with documentation

3. **P1-M1**: boot-time freeze on pre-MIG-013 DBs during the v2 bigram-sentinel UPDATE. Memory entry created. Will ship as a focused mini-MIG before any v1.0 release. Boss's library has already completed this migration; new users with pre-MIG-013 backups remain at risk for ~30-90 sec freeze.

### Forward-compat artifacts (intentional, no action)

- `term_vocab.bridge_concept_id` column + index + v1/v2 migrations.
- `index.semanticSearchEnabled` settings flag.
- `searchHub.concept` / `searchBadges.concept` i18n keys.
- Orphaned `term_embeddings` table on existing DBs (cleanup migration candidate).

---

## §5. MIG-013 close

All P0s clear. P1s addressed (2 in this commit) or deferred (1 with documentation). The CTSE Bridge Adapter ships:

- `bridge_vectors` — 30 MB baked asset (20K M11 concepts × 384 f32 vectors).
- `ctse::hooks` — write-time `term_vocab` ledger maintenance via `on_note_indexed` / `on_note_deleted`.
- `ctse::search::ctse_search_terms_by_concept` — query-time concept expansion for the IndexPanel filter `≈ similar` row.
- `concept_lemmas` in-memory `concept_id → [lemmas]` map, lazily built once at boot from `LexiconGraph`.
- M11 zero-diff invariant intact across the entire migration.

Architecture aligns with industry best practice (Lucene `SynonymGraphFilter`, SQLite FTS5 Method 2, CLIR query-translation, Primo controlled-vocabulary expansion). No backfill, no first-fill, no per-library setup cost. Reacts automatically to new M11 releases.

Boss-test passed Stage 1 + Stage 2. MIG-013 closes.
