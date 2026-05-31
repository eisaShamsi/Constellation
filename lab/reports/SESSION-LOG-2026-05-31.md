# Session log — 2026-05-31

Continuation of the 2026-05-30 marathon (crossed midnight). MIG-066 §B closed; pivot to **MIG-067 — User-Definable Link Types ("The Living Vocabulary")**.

## MIG-066 §B — closed (early hours)
Per-type counts shipped + Boss-confirmed (`supports (358), contradicts (1), …` EN + AR). The overnight-blank chain resolved (3 bugs: link-type *syntax* mismatch → converter; missing *auto-reindex* → deferred self-heal reconcile; fragile *recompute* → batched + lock-tolerant). Detail in `SESSION-LOG-2026-05-30.md`. Commits `2481fb92` (syntax correction), `9b5a491d` (auto-reconcile), `28360a27` (counts + recompute hardening), all pushed.

## MIG-067 — User-Definable Link Types

Eisa's request: let users add their own link types — **top-level** (peers of the 8) OR **nested under one of the 8** — with **per-type sortable Base columns**. Rationale: thinking is ongoing; the vocabulary must grow with the thinker. Treated as its own `/migration` folding in MIG-066 §C/§D/§E. (Memory: [[project_user_definable_link_types]].)

- **Architect** (`docs/USER-DEFINABLE-LINK-TYPES-ARCHITECT.md`, commit `28360a27`-era): grounded in 3 Explore sweeps. Spine = ONE shared Link-Type Registry every surface reads (also delivers §E + fixes existing drift — Inspector360 missing `supersedes`). `resolve_dim`'s `prop.<key>` pattern is the template for dynamic `note.link.<id>` columns; `custom_stages`/`property-types.json` is the storage precedent. Corrected: **Sight is out of core (→ 360.3D)** — its surfaces excluded from v1, like `ConstellationEditor/`.
- **Plan** (`docs/USER-DEFINABLE-LINK-TYPES-PLAN.md`): 9 phases (A registry+parser, B materialization/JSON/change-flow, C frontend store, D reconcile surfaces=§E, E editor inline colors, F dynamic per-type columns, G Settings vocabulary editor, H i18n/docs/Concept-Paper v1.1, I audit). Boss-test pauses at D/E/F/G. Eisa: **all 7 decisions yes**; approved; MIG number = Claude's call → **MIG-067** (Epistemics → 068). **Invariant: a no-custom universe is byte-identical to today.**

### §A — registry + parser (commit pending)
- **`link_types.rs`** (new): `LinkTypeDef`, the 8 built-in seeds (ids/semantics/order immutable; canonical colors), `LinkTypeRegistry::merge` (seeds + deltas; seed-id deltas override presentation only; custom = flat or child-of-8; flattened canonical order), global `OnceLock<RwLock<…>>` defaulting to seeds, per-universe persistence (`link-types.json`, the `property-types.json` pattern), commands `read/save_universe_link_types` + `list_link_types`. Deltas-only file (created on first save, like `custom_stages`).
- **`search.rs`**: `extract_typed_links`/`parse_link_body` recognize a type via `link_types::is_known_type` (8 seeds + custom), not the removed `PARSER_LINK_TYPES`; `load_active` wired into `ensure_search_db_ready` (boot + universe-switch).
- **`lib.rs`**: `mod link_types` + 3 commands registered.
- **Verify:** 6 registry tests (seeds order/rank, custom top-level appended, child nested, seed-override protects grammar, invalid-parent→top-level, blank-id dropped) + 6 parser tests still green; **890/890 lib tests** pass. No-custom behavior identical (default registry = the 8).

### §B — registry-driven materialization + JSON + change-flow (commit `6987813a`)
The outgoing-link aggregates now come from the active registry, not a hardcoded 8:
- **`search.rs`**: `outgoing_aggregate_assignments` builds the IN-list + rank `CASE` + empty sentinel from `link_types::snapshot()` (removed the `LINK_TYPE_IN_LIST`/`LINK_TYPE_RANK_CASE` consts). New `note_meta.outgoing_link_types_json` column (`{"type":count}` via `json_group_object`) materialized write-time alongside the display string — feeds §F's per-type sortable columns. `create_outgoing_link_triggers` drops-first (a boot always reflects the current registry); `load_active` moved **before** `init_db` so the triggers it creates carry the right rank. New `on_link_vocabulary_changed` (recreate triggers + schedule re-materialize) called from `save_universe_link_types`.
- **`link_types.rs`**: `sql_in_list` / `sql_rank_case` / `sentinel_rank` / `fingerprint` (FNV-1a over ordered ids) / `sanitize_id` (slug floor for generated SQL + column names, applied to ids **and** parents at merge).
- **`links_backfill.rs`** — the **fingerprint gate**: `fingerprint` stamped at each completed back-fill as `schema_versions('links_vocab')`; `is_needed` re-triggers when stored ≠ active. This doubles as the §A→§B migration: an existing universe (no `links_vocab` stamp, fp 0 ≠ seed fp) gets a one-time background pass that fills the JSON column. `run()` resets the cursor on a vocab-change re-run (version already current) so every row re-materializes, not just the tail; `finalize` stamps the run's start-fingerprint (mid-run vocab edits re-trigger next boot — eventual consistency).
- **Invariant held**: a no-custom universe yields byte-identical `outgoing_count`/`outgoing_link_types`/`outgoing_top_rank` (default registry = the 8); only the JSON column is added.
- **Verify**: new `vocab_fingerprint_gate_triggers_rematerialize` (absent/matching/changed) + JSON-group-object assertion + drop/recreate cycle. **891/891 lib tests** pass.
- **Boss-visible on next launch**: a one-time, non-blocking JSON back-fill (~7,600 rows, batched 500/txn) — no UI change until §F reads the column.

## Open / next
- §C frontend registry store (`linkTypeRegistry.ts`, `boot_bundle` seed) — foundation, no Boss test. Then §D reconcile surfaces (Boss test), §E editor colors (Boss test), §F dynamic per-type columns (Boss test), §G Settings editor (Boss test), §H i18n/docs/Concept-Paper v1.1, §I audit + PCS.
