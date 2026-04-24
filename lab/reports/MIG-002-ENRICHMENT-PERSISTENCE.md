# MIG-002 — Enrichment Persistence (Option D: Hybrid)

**Status**: Phase 1 complete (Architect). Phase 2 plan drafted — awaiting Build.
**Scope**: Persist the three enrichment columns reserved as NULL placeholders by MIG-001 (`sky_nodes.stratum`, `sky_nodes.maturity`, `sky_nodes.origin_type`). Remove the frontend `enrichNodesBackground()` boot call once parity is proven.
**Chosen option**: **D — Hybrid.** Stratum + Maturity computed by SQL-native triggers against denormalized `note_meta` signals. Origin type computed by a background worker gated on a `sky_nodes.enrichment_dirty` flag (recursive derives-from walk is not a reasonable SQL expression).

## Phase 1 — Architect (summary)

Placed against MIG-001's precedent (sky_nodes/sky_links landed with NULL enrichment columns by design — see MIG-001 §74, Step 7 decision). Phase 1 surveyed the three enrichment sources:

- **`strata.rs:51` (`compute_stratum`)** — inputs: `word_count`, `outgoing.len()`, `inbound_count`, `outgoing_types` (semantic link subset), `inbound_sources.len()` (unique). All five signals are derivable from DB state: word count from `note_meta.body_text`, outgoing from `note_links`, inbound count/sources from the reverse direction of `note_links`, link types from `note_links.link_type`. No filesystem reads required once word count is denormalized onto `note_meta`.
- **`maturity.rs:31` (`compute_state`)** — inputs: `inbound_count`, `days_since_created`, `days_since_modified`. Currently pulled via `fs::metadata` per note. `note_meta.modified` already exists; a new `note_meta.created_at` column is needed. Both quantities become SQL `strftime` arithmetic.
- **`provenance.rs:97` (`compute_note_origins`)** — inputs: recursive derives-from walk (depth 10) + frontmatter external-source keys (`url`, `author`, `source`, `doi`, `isbn`). Graph walk is not expressible as a bounded-cost SQL expression — worker-based recomputation keyed on a dirty flag is the correct shape.

**Frontend coupling**: `+layout.svelte:2638` `enrichNodesBackground()` + `GraphMindView.svelte:1118` `onRequestEnrichment` trigger three IPC calls that compute in-memory and mutate `skyNodes[]` locally — values lost on relaunch, not visible to SecondScreenPage. MIG-002 replaces with persisted columns served through the existing `cache_boot_snapshot_sky` IPC.

**Option D rationale** — pure-SQL for strata/maturity keeps the write path zero-IPC (Invariant 2 / Rule 3) and gives each structural edge change instant propagation to color mode (Invariant 4). Worker-based origin type keeps recursive graph traversal out of triggers (Invariant 8 WHEN guard stays restrictive).

**Invariants** (from Phase 1):
1. Color-mode parity vs pre-MIG-002
2. Zero `invoke()` on keystroke hot path (Rule 3 / IPC-CONTRACT)
3. No per-keystroke full-library cascade
4. Bounded freshness with "computing…" signal on origin-type lag
5. Graceful mid-backfill NULL render
6. Rename preserves enrichment
7. SecondScreenPage reads same persisted columns
8. Restrictive AU WHEN guard
9. SKY_SCHEMA_VERSION bump back-fills new columns only
10. Rollback still callable on old IPCs

---

## Rename-trigger decision

**Approach: hybrid per signal type. UPDATE-preserving for strata/maturity, dirty-flag for origin_type.**

Current `note_meta_sky_au` (search.rs:618–657) DELETE+INSERTs the `sky_nodes` row on path/name/library change, which would wipe MIG-002's enrichment columns every rename. The two candidate fixes:

1. **UPDATE-preserving**: rewrite AU to `UPDATE sky_nodes SET path=NEW.path, name=NEW.name, library_name=NEW.library_name WHERE path=OLD.path`, leaving enrichment columns intact. A pure rename (no content change) does not change any of the stratum/maturity signals (word count, inbound count, outgoing count, age) — derived values are provably unchanged, so preservation is correct.
2. **Dirty-flag re-enrich**: stamp `enrichment_dirty=1` on rename and let workers/triggers recompute.

Strata and maturity depend *only* on structural and temporal signals that rename does not touch. Preserving them is cheaper and more correct (no transient NULL during the AU→recompute window). **Strata and maturity triggers are UPDATE-preserving on AU.**

Origin type is different: a rename can't change the renamed note's own frontmatter (which fires the separate content trigger), but the cascading effect on descendants — "this node's children must re-walk their chain because an ancestor renamed" — cannot be proven null without a walk. **Rename of a note that participates in any derives-from edge flips `enrichment_dirty=1` on affected rows** (source and descendants of OLD.name). The worker reconciles.

For renames of a note that does NOT appear in any derives-from edge (checked via EXISTS subquery), origin_type is also preserved — no descendants affected, no recompute needed.

---

## Schema changes (summary)

**New columns on `note_meta`**:
- `word_count INTEGER NOT NULL DEFAULT 0` — stamped by the Rust writer in `index_note` alongside `body_text` assignment.
- `created_at INTEGER` — populated on INSERT from `fs::metadata(...).created()` epoch seconds. Nullable for rows created before the migration; back-fill stamps from `fs::metadata` on first-touch.

**Bumped constant**: `SKY_SCHEMA_VERSION: 1 → 2`. Gate at `cache.rs:359` automatically forces back-fill on upgrade.

**New column on `sky_nodes`**:
- `enrichment_dirty INTEGER NOT NULL DEFAULT 1` — flag toggled by note_links triggers on any derives-from edge change. Worker drains dirty rows, writes `origin_type`, clears flag.

**Trigger diff** (vs MIG-001):
- `note_meta_sky_ai` — now also computes `stratum`, `maturity`, leaves `origin_type = NULL`, `enrichment_dirty = 1`.
- `note_meta_sky_au` — split: (a) UPDATE-preserving rename AU (path/name/library change only — no enrichment touched except conditional origin_type dirty); (b) content-change AU (word_count/modified/created_at changed — recomputes stratum + maturity in place; origin_type untouched).
- New `note_links_sky_stratum_*`, `note_links_sky_maturity_*`, `note_links_sky_enrichment_*` triggers covering count changes and derives-from edge changes respectively. WHEN guards keep them narrow.

**New Rust module**: `src-tauri/src/enrichment_worker.rs` (sibling of `sky_backfill.rs`). Owns origin_type reconciliation loop.

---

## Phase 2 — Plan (10 steps)

| # | Step | /simplify? | Verify |
|---|------|-----------|--------|
| 1 | Add `word_count` + `created_at` to `note_meta`; bump `SKY_SCHEMA_VERSION` 1→2 | — | App boots; existing rows get NULL until back-fill |
| 2 | Writer-side stamping of word_count / created_at in `index_note` | — | New saves populate columns; old rows pending back-fill |
| 3 | Add `enrichment_dirty` column to `sky_nodes`; back-fill stamps all=1 | ✔ | Fresh rows default dirty; existing rows flipped |
| 4 | SQL-native stratum trigger on note_meta + note_links | — | Insert note, save edit, add link → stratum populates/updates without IPC |
| 5 | SQL-native maturity trigger on note_meta + note_links | — | Age transitions visible; modified bump flips sapling→evergreen at 7-day boundary |
| 6 | Rewrite rename AU: UPDATE-preserving for stratum/maturity; conditional dirty for origin | ✔ | Rename unchanged note → stratum/maturity unchanged, visible within one frame |
| 7 | `enrichment_worker.rs` — dirty-row drain loop for origin_type | — | Dirty rows processed in batches; flag clears on write |
| 8 | note_links derives-from triggers flip `enrichment_dirty` | — | Add/remove a derives-from edge → worker picks up within batch interval |
| 9 | Extend `cache_boot_snapshot_sky` to populate enrichment columns; guard frontend `enrichNodesBackground` on isReady+non-null | ✔ | Color modes paint from persisted values; no IPC on hot path |
| 10 | Phase-4 audit + cleanup (byte-diff parity vs pre-MIG-002) | — | Write-amp <2× baseline; parity passes |

### `/simplify` checkpoints
Steps **3, 6, 9** — mirrors MIG-001's placement (after schema landing, after trigger rewrite, after frontend swap).

---

### §1 — Schema landing: word_count + created_at + SKY_SCHEMA_VERSION bump

**What**
- `src-tauri/src/search.rs` — add `word_count INTEGER NOT NULL DEFAULT 0` and `created_at INTEGER` to `note_meta` CREATE TABLE. Idempotent `ALTER TABLE note_meta ADD COLUMN` path for existing DBs (gate on `PRAGMA table_info`).
- `src-tauri/src/search.rs:50` — `SKY_SCHEMA_VERSION: 1 → 2`. Docstring table extended with row `| 2 | note_meta.word_count + created_at; sky_nodes enrichment populated |`.

**Why** — Invariants 9, 10. ALTER TABLE columns are nullable/defaulted so pre-MIG-002 code continues reading the table unchanged.

**Verification** — Existing DBs still boot; `SELECT word_count FROM note_meta LIMIT 1` returns 0; `stored_sky_version < 2` triggers `sky_backfill::maybe_schedule`.

**Risk mitigation** — ALTER TABLE idempotent via `PRAGMA table_info`. SKY_SCHEMA_VERSION single-source per MIG-001 §86.

**Rollback** — Pre-MIG-002 binary reads the wider schema without error. No DB revert needed.

---

### §2 — Writer-side stamping of word_count + created_at

**What**
- `src-tauri/src/search.rs` — extend the INSERT column list at every `INSERT INTO note_meta` site (L987, L1130, L3130, L3315, L3848) to include `word_count` and `created_at`. Word count = whitespace split of `body_text` after frontmatter strip (factor helper). Created_at = `fs::metadata(&path).created()` epoch seconds, falling back to `modified`.
- Document in module comment: writer always recomputes word_count; triggers rely on the writer, not SQL expressions.

**Why** — Invariant 2. Putting word-count in the writer keeps trigger WHEN guards cheap (`word_count IS NOT OLD.word_count`).

**Verification** — Saving a new note populates word_count > 0 and created_at > 0; existing rows still NULL/0 until back-fill in §3.

**Risk mitigation** — Fallback to `modified` for `created_at` avoids panics on ReFS/FAT32. Word-count helper pure; no divergence with strata.rs logic.

**Rollback** — Column-level: leave the columns. Binary-level: standard revert.

---

### §3 — enrichment_dirty on sky_nodes + back-fill stamping `/simplify`

**What**
- `src-tauri/src/search.rs` — add `enrichment_dirty INTEGER NOT NULL DEFAULT 1` to `sky_nodes`. Idempotent ALTER.
- `src-tauri/src/sky_backfill.rs` — extend to back-fill `word_count`/`created_at` on existing `note_meta` rows (read file, compute, UPDATE). Keep 1000-row batches.
- Populate `stratum`, `maturity` for existing sky_nodes rows via the same SQL expressions used in §§4–5. Leave `origin_type = NULL`, `enrichment_dirty = 1`.
- `/simplify`: prune duplication between back-fill and trigger expressions (shared stratum-from-signals helper).

**Why** — Invariants 5, 9.

**Verification** — Cold universe: back-fill completes with word_count/created_at populated; stratum/maturity persisted; origin_type NULL + enrichment_dirty=1.

**Risk mitigation** — Cursor-resumable (MIG-001 §86 finalize-transaction carries forward). 1000-row batches bounded. UPDATE-by-PK idempotent.

**Rollback** — Older binary tolerates extra column. Frontend enrichNodesBackground overwrites values in memory (no persistence collision).

---

### §4 — SQL-native stratum trigger

**What**
- `src-tauri/src/search.rs` — add `note_meta_sky_stratum_ai` (AFTER INSERT) and `note_meta_sky_stratum_au` (AFTER UPDATE, WHEN `NEW.word_count IS NOT OLD.word_count`). Body: `UPDATE sky_nodes SET stratum = (<SQL expr>) WHERE path = NEW.path`.
- SQL expression mirrors `strata.rs::compute_stratum`: CASE on word_count for base stratum; bonus via subqueries on note_links for outgoing/inbound/semantic types; sources-cardinality via `COUNT(DISTINCT source_path)`.
- `note_links_sky_stratum_au/ai/ad` triggers on `note_links` recompute stratum for affected source_path's sky_nodes row AND any row whose `name = NEW.target_name`. WHEN-guarded to inserts/deletes/status transitions.

**Why** — Invariants 1, 2, 3.

**Verification** — Saving a note with new wikilinks updates its stratum within the same transaction; target notes' strata update on the same write. Zero `compute_note_strata` IPC calls for display.

**Risk mitigation** — WHEN guard skips body edits without word-count change. Single-row indexed UPDATEs.

**Rollback** — Drop triggers; stratum column goes stale but non-null; frontend IPC fallback still works.

---

### §5 — SQL-native maturity trigger

**What**
- `note_meta_sky_maturity_ai/au` keyed on modified/created_at changes. CASE expression from `maturity.rs::compute_state`:
  - `'seed'` — inbound=0 AND (modified - created_at) ≤ 86400
  - `'sapling'` — inbound 1–3 OR (modified - created_at) > 172800
  - `'evergreen'` — inbound ≥ 4 AND (modified - created_at) ≥ 604800
  - `'canonical'` — inbound ≥ 10 AND (now - modified) ≥ 2592000
  - `'wilting'` — evergreen AND (now - modified) ≥ 7776000
- Inbound count via subquery. `'now'` is `strftime('%s','now')` at trigger fire time.
- `note_links` triggers recompute maturity for target_name changes.

**Why** — Invariants 1, 4. Wilting/canonical advance only on writes that cross the threshold — matches current on-demand behavior; users who never save never see transitions but also don't render SV with fresh data otherwise.

**Verification** — Saving an old note advances it through maturity buckets; newly-inbound-linked note transitions seed→sapling on the save that added the inbound.

**Risk mitigation** — Wilting-on-time-only staleness accepted; documented in trigger comment.

**Rollback** — Drop triggers; values freeze at last computed state.

---

### §6 — Rename AU rewrite: UPDATE-preserving + conditional origin dirty `/simplify`

**What**
- `src-tauri/src/search.rs:638–657` — replace `note_meta_sky_au` body:
  ```sql
  -- Path change: in-place UPDATE of sky_nodes by PK (preserving enrichment cols).
  UPDATE sky_nodes
     SET path = NEW.path, name = NEW.name, library_name = NEW.library_name,
         updated_at = strftime('%s','now')
   WHERE path = OLD.path;
  -- sky_links source_path / target_name fix.
  UPDATE sky_links SET source_path = NEW.path
   WHERE source_path = OLD.path AND OLD.path IS NOT NEW.path;
  UPDATE sky_links SET target_name = NEW.name
   WHERE target_name = OLD.name AND OLD.name IS NOT NEW.name;
  -- Conditional origin-type dirty cascade.
  UPDATE sky_nodes SET enrichment_dirty = 1
   WHERE path IN (SELECT source_path FROM note_links
                   WHERE link_type='derives-from' AND target_name=OLD.name)
      OR path = NEW.path;
  ```
- Drop DELETE+INSERT entirely for rename. Docstring explains preservation guarantee.
- `/simplify`: merge overlap with note_meta_sky_ai; extract enrichment-dirty stamp helper.

**Why** — Invariant 6 (the central correctness fix).

**Verification** — Rename a note → `sky_nodes.stratum` unchanged. Rename a derives-from ancestor → descendants marked dirty; worker recomputes.

**Risk mitigation** — `OLD.path IS NOT NEW.path` guard on PK update. EXISTS subquery indexed via note_links target. Hub rename dirties many descendants — acceptable, correct.

**Rollback** — Revert to DELETE+INSERT. Enrichment goes NULL on next rename; frontend IPC fallback recomputes.

---

### §7 — `enrichment_worker.rs` — dirty-row drain loop

**What**
- New `src-tauri/src/enrichment_worker.rs`, sibling of `sky_backfill.rs`:
  - `pub fn maybe_schedule(app)` — background thread from `ensure_search_db_ready` after `sky_backfill`.
  - Loop: `SELECT path, name FROM sky_nodes WHERE enrichment_dirty=1 LIMIT 500`. For each batch: scoped derives-from walk (depth ≤ 10, cycle-guarded) via note_links, compute origin_type via `provenance.rs::classify_origin` (factor pub). `UPDATE sky_nodes SET origin_type=?, enrichment_dirty=0 WHERE path=?`.
  - Inter-batch 100ms; empty-batch idle 5s re-poll.
  - External-source frontmatter via `note_meta.properties_json` (no file re-open).
- Register in `src-tauri/src/lib.rs`.

**Why** — Invariants 2, 4.

**Verification** — Boot worker drains dirty rows within ~(count / 500 × 100ms) + 5s idle; `SELECT COUNT(*) FROM sky_nodes WHERE enrichment_dirty=1` → 0 at steady state.

**Risk mitigation** — 500-row batches (smaller than sky_backfill — each row walks). Polling idle-sleep. Cycle guard from provenance.rs.

**Rollback** — Unregister module; origin_type stale; frontend fallback.

---

### §8 — note_links triggers flip enrichment_dirty on derives-from edges

**What**
- `src-tauri/src/search.rs` — add three triggers on `note_links`, WHEN-guarded on `link_type='derives-from'`:
  - AI: stamp source_path + any sky_nodes with `name = NEW.target_name`.
  - AU: stamp if OLD or NEW link_type is derives-from.
  - AD: stamp OLD source_path and any sky_nodes with `name = OLD.target_name`.
- Narrow stamp to source+target immediately; worker does transitive cascade on walk.

**Why** — Invariants 4, 8.

**Verification** — Adding `[[Foo|derives-from]]` flips dirty on source's sky_nodes row; origin_type updates within one worker batch.

**Risk mitigation** — WHEN guards skip non-derives writes (majority).

**Rollback** — Drop triggers; origin_type staleness grows on edge changes; frontend IPC fallback.

---

### §9 — Frontend swap: remove enrichNodesBackground on ready-path `/simplify`

**What**
- `src-tauri/src/cache.rs:380` (`read_sky_nodes_raw`) — include `stratum`, `maturity`, `origin_type` in SELECT. Map to `SkyNodeOut` DTO (already has optional fields).
- `src/lib/libraries/store.ts` — confirm `SkyNode` interface has optional stratum/maturity/originType (it does — MIG-001 reserved them).
- `src/routes/+layout.svelte:2633` — guard `enrichNodesBackground` invocation: skip when `sky.isReady === true` AND returned nodes already have non-null stratum/maturity/origin_type. Leave callable for manual refresh / fallback.
- `src/routes/+layout.svelte` onRequestEnrichment — no-op when persisted values present; falls through to `enrichNodesBackground` otherwise.
- `src/lib/components/SecondScreenPage.svelte` — verified to read from shared libraries store; no code change.
- `/simplify`: confirm remaining `compute_note_*` IPC calls are rollback-only.

**Why** — Invariants 2, 7, 10.

**Verification** — Boot-perf trace: zero `compute_note_strata/maturity/origins` IPC on ready universe; color modes correct; SecondScreenPage matches.

**Risk mitigation** — `isReady=false` path unchanged — mid-backfill, old DB, corrupted stamp all fall through to existing code.

**Rollback** — Remove `isReady`-gated skip; old flow resumes. Zero data loss.

---

### §10 — Phase 4 audit + cleanup

**What**
- Run audit agents in parallel per `/migration` skill (invariant / drift / migration-path).
- Byte-diff parity: dump `{path, stratum, maturity, origin_type}` from sky_nodes; compare to pre-MIG-002 by running compute_note_* IPCs against same DB.
- Write-amp: per-save trigger cost on 100-edge note < 2× pre-MIG-002 baseline.
- Consider (don't auto-execute) relabeling "Compute for this session" button since values now persist — decide based on whether button triggers `enrichment_dirty=1` full sweep or stays session-only override.

**Verification** — ≤3 HIGH findings; all triaged; parity + write-amp budgets met.

**Rollback** — N/A (read-only).

---

## Critical Files for Implementation

- `src-tauri/src/search.rs` — schema, triggers, init_db
- `src-tauri/src/sky_backfill.rs` — extended back-fill for word_count/created_at/stratum/maturity
- `src-tauri/src/cache.rs` — SELECT enrichment cols, boot snapshot payload
- `src-tauri/src/enrichment_worker.rs` — **new** — dirty-row drain loop
- `src-tauri/src/strata.rs` / `maturity.rs` / `provenance.rs` — factor pub helpers; IPCs kept for fallback
- `src/routes/+layout.svelte` — guard enrichNodesBackground; retain as fallback

**Next**: Phase 3 — Build, starting with §1.
