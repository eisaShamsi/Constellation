# MIG-079 — Architect: Write-Time Derivation for the Boot Graph Snapshot + Single-Owner Activation

> Phase 1 (Architect) of the `/migration` workflow. Maps the territory, enumerates design options with speed/effort/risk, lists the invariants that must not break. **No code is written in this phase.** Produced 2026-06-15 by an 8-agent examine+research workflow (`wf_86695058-176`) + synthesis, then fact-checked against the live DB.
>
> **Function in hand:** the boot graph snapshot — `cache_boot_snapshot_graph` (`src-tauri/src/cache.rs:228`), the read-path IPC that feeds Sky View, Backlinks/Outgoing panels, the Tags browser, and Sight — and the active-universe activation path (`set_active_universe`, `universe.rs:601`) that the main window and second screen both invoke.

---

## 0. Ground truth (measured, not assumed)

From `boot-perf.latest.json` on the live 1.83 GB "Eisa Cognitive Knowledge" universe (7,653 notes):

| Metric | Value | Verdict |
|---|---|---|
| `paint_ms` | 941 | PASS — first paint is fast |
| `hydrated_ms` | 1,671 | PASS — hydration is fast |
| **`graph_ready_ms`** | **32,519** | **THE bottleneck** |
| `cache_boot_snapshot_graph` wall | 30,838 | — |
| ↳ `read_links` | 11,609 | full SCAN of every active `note_links` row, all 10 columns |
| ↳ `read_tags` | 5,577 | `SELECT tags_json FROM note_meta`, JSON-parse per row |
| ↳ `read_aliases` | 0 | already fast — leave alone |
| ↳ `count_notes` | 106 | — |
| `cache_snapshot_graph_queue_ms` | 11,250 | IPC round-robin contention (~34 concurrent boot IPCs) |
| init_db fires | **twice, 34 s apart** (17:50) | with the second screen open |

> **Measured correction (Claude-side, read-only on the live DB, 2026-06-15):** `note_links` is **233,995 rows, ALL `status='active'`** — not the "656k" an older boot-perf note assumed. `EXPLAIN QUERY PLAN` confirms both boot reads are full table scans: `SELECT … FROM note_links WHERE status='active'` → **`SCAN note_links`** (the `idx_link_status` index is useless because every row is active); `SELECT tags_json FROM note_meta` → **`SCAN note_meta`** (7,653 rows). So `read_links` = a 234k-row × 10-column heap walk; `read_tags` = a 7,653-row full scan + per-row JSON parse. The conclusion is unchanged; the number is corrected for the record.

First paint and hydration already PASS. **Everything painful lives after hydration, in the deferred graph snapshot — and it is doubled by a redundant second-window activation.**

---

## 1. Root cause — one disease, two instances

Both defects are the SAME violation of **CLAUDE.md Rule 8 (Write-Time Derivation)**: *"Every computed view is maintained at write time, not read time. The app does not recompute on boot. It reads what's already stored."*

### 1.1 The 32 s — read-time graph/links/tags recompute
`cache_boot_snapshot_graph` (`cache.rs:228–352`) rebuilds the entire link graph + tag-count map **from raw source tables on every boot**:

- `read_links_in_schema` (`cache.rs:924`) — `SELECT source_path, source_name, target_name, link_type, library_name, weight, traversal_count, annotation, last_traversed, confidence FROM {schema}.note_links WHERE status='active'`. No covering index → heap walk of all 234k rows, serialize 10 columns each → **11,609 ms**.
- `read_tags_in_schema` (`cache.rs:1027`) — `SELECT tags_json FROM {schema}.note_meta`, then `serde_json::from_str` per row and aggregate counts in Rust → full scan of 7,653 rows → **5,577 ms**.

This is the **last uncured instance** of a pattern the codebase already cures three times:
- **FTS5 `notes_fts`** — maintained by `note_meta_ai/ad/au` triggers (`search.rs:~2272`). Search is instant; nothing rebuilt on boot. (Rule 8's canonical example.)
- **`sky_nodes` / `sky_links`** — maintained by `note_meta_sky_ai/ad/au` + `note_links_sky_ai/ad/au` triggers (MIG-001/002).
- **`note_meta.outgoing_count` / `outgoing_link_types` / `outgoing_top_rank`** — maintained by `note_links_outgoing_ai/ad/au` triggers (MIG-066), with the aggregate SQL factored into `outgoing_aggregate_assignments("NEW.source_path")` (`search.rs:665`). This is the closest exemplar: an aggregate over `note_links` scoped to one source, recomputed per affected edge, never O(table).

Read-path consumers (Sky View, Backlinks/Outgoing panels, Tags browser, Sight) all wait on `graphReady`.

### 1.2 The multiplier — redundant per-window activation
No single owner for activation. The main window activates at `+layout.svelte:2515` (and `:2177`); the second screen redundantly activates the **same** universe id at `SecondScreenPage.svelte:923` — `await setActiveUniverse(universes[0].id)`.

`set_active_universe` (`universe.rs:601`) was built for universe **switching** (changing ids), not idempotent re-activation. It unconditionally runs:
- `invalidate_libraries_cache()` (`universe.rs:732`)
- `invalidate_search_state()` (`universe.rs:742`, the MIG-055 §H DB-connection reset)

There is **no idempotency guard before these side effects**. The second call forces a second `ensure_search_db_ready` → a second `init_db` (observed: **init_db ran twice, 34 s apart**) → a full second `read_links`(11.6 s) + `read_tags`(5.6 s) recompute. The second screen also has **no read-only path** to learn the active id without activating.

### 1.3 Why these are not alternatives
- Idempotent activation removes the **duplicate** 30 s.
- Write-time derivation removes the 30 s **itself**.

Shipping only the guard would make `graph_ready_ms=32519` still stand for the primary window while looking "fixed." **Both are required; neither substitutes for the other.**

---

## 2. WA#5 — industry pattern vs Constellation proposal (honest comparison)

The research is unanimous, and it points at the patterns the codebase **already ships** — so the honest answer is *"don't invent; extend in-house."*

| Concern | Dominant industry pattern | Constellation status | Verdict |
|---|---|---|---|
| Derived view maintenance | **Immediate (eager) incremental view maintenance** — apply the delta at write time, read pre-computed state (PostgreSQL pg_ivm; SQLite's vehicle = AFTER-triggers; Materialize/Noria are server-only references) | Already live: `sky_nodes`, `outgoing_*`, `notes_fts` triggers | **Extend the in-house trigger pattern.** No new dependency. |
| Backlink/graph at cold start | **Persist the forward-link index, load it directly, re-parse only changed files** (Obsidian IndexedDB MetadataCache; Dendron content-hash cache; Logseq SQLite datoms). Foam recomputes-on-launch and is the named anti-pattern that doesn't scale. | `note_links` (234k rows) **IS** the persisted forward index — but `cache_boot_snapshot_graph` ignores it and recomputes (the Foam mistake) | **Read the persisted table; stop recomputing.** |
| Facets/counts (tags) | **Precompute at write time, store the column** (Lucene doc-values moved field-data from query-time to index-time; inverted index term→postings) | Tags scanned + JSON-parsed at read time | **`tag_counts(tag, n)` summary table, trigger-maintained.** |
| Adjacency read shape | **Edge table + covering indexes both directions** (SQLite covering-index ~2× win; native graph DBs use index-free adjacency, which does NOT map to SQLite) | `note_links` has no reverse covering index for the boot projection | **Add a covering index** so any residual read streams in index order. |
| Multi-window shared resource | **Backend-owns-state + idempotent activation + thin-display windows** (Tauri `app.manage` app-global state; VS Code refuses to re-open an open workspace; pull-on-mount + revision gate) | Rust holds app-global `UniverseState` (correct shape) but activation is non-idempotent and per-window | **Add idempotency guard + make second screen display-only.** |

**Conclusion:** there is no inventive Constellation-specific proposal to weigh against the field — the field's answer *is* the in-house pattern, applied to the one surface that still recomputes. The only genuinely novel bit is parsing the `tags_json` array delta inside the maintenance path (open decision #3).

---

## 3. Options (speed / effort / risk / boot impact)

### Option A — `tag_counts` summary table (read_tags only)
- **Approach:** `tag_counts(tag TEXT PRIMARY KEY, n INTEGER)` per schema, trigger-maintained on `note_meta` tags_json writes (parse old vs new JSON delta, ±1 per tag). Boot `read_tags` → `SELECT tag, n FROM tag_counts`. Resumable background backfill gated by `schema_versions.tag_counts` (mirrors `sky_backfill`/`links_backfill`); fall back to the live scan until the sentinel flips.
- **Speed:** `read_tags` 5,577 ms → <1 ms. `read_links` unchanged.
- **Effort:** Low — one table, one trigger trio, one backfill module, one gate; all copy-the-pattern.
- **Risk:** Low. Trigger O(tags-on-changed-note). Only novel bit: JSON-delta parse.
- **Boot impact:** 32.5 s → ~26.9 s. Proves the pattern; does **not** hit target alone.

### Option B — covering index on `note_links` (links read)
- **Approach:** Covering index `note_links(status, source_path, target_name, link_type, library_name, weight, traversal_count, last_traversed, confidence)` — **exclude `annotation`** (not read at boot; `cache.rs:949` already sets `context` empty). Boot read becomes an index-only range scan streaming in order.
- **Speed:** `read_links` 11.6 s → ~6–8 s. The 234k-row IPC payload + 11.25 s queue contention remain.
- **Effort:** Low-medium — one CREATE INDEX, background-built, schema_versions-gated.
- **Risk:** Low-medium. Index footprint; the 234k-row transfer is the residual this can't remove.
- **Boot impact:** with A, ~17–21 s. Insufficient alone.

### Option C — RECOMMENDED — persisted derived snapshot + idempotent single-owner activation
Two coordinated workstreams, landed as separate validated commits, each schema_versions-gated.

**Workstream 1 — the 32 s (write-time derivation):**
- (a) Tags: Option A's `tag_counts`.
- (b) Links: stop recomputing the 234k payload on the `graphReady` critical path. Two sub-choices (**open decision #1**): **(b1)** defer the full read off the critical path — Sky View/Backlinks read the already-persisted `sky_links` + `note_meta.outgoing_*` at boot, the full edge list loads lazily on first graph/panel open (materialized → a read, not a scan); or **(b2)** materialize a persisted boot-graph edge-projection table maintained by `note_links` triggers, read in index order. Add Option B's covering index regardless.
- (c) Compose with **MIG-078 Phase B**: the persisted tree already proved the `schema_versions`-gated resumable-backfill machinery on this exact universe; the graph snapshot becomes the second persisted-derived surface read the same way, reusing `sky_backfill`/`links_backfill`/`note_body` as templates.

**Workstream 2 — the multiplier (single-owner idempotent activation):**
- (d) Idempotency guard at the **top** of `set_active_universe` (after resolving `final_path`, **before** `invalidate_libraries_cache`/`invalidate_search_state`): `if *active_path.lock() == Some(final_path) && db.is_some() { return Ok(()) }`, holding the lock across check-and-set. Compares **resolved/healed path + db presence**, never the raw input id (path-healing runs every activation, `universe.rs:611–723`).
- (e) Remove `SecondScreenPage.svelte:923`'s `setActiveUniverse`; add a read-only `get_active_universe` IPC on mount + a `federation:ready`/active-changed subscription (pull-on-mount + revision gate) — *"additional screens are displays, not domains."*
- (f) Revision/length gate on the 3× `cache_boot_snapshot_sky` re-fires so a late/empty federation re-fire can't clobber good data (MIG-061 §P empty-overwrite-guard precedent).

- **Speed:** fastest end-state — `read_tags` <1 ms; duplicate activation gone (no second init_db, no second 30 s); residual links read deferred or index-streamed.
- **Effort:** medium-high, but every piece reuses an in-house exemplar; ~5 commits, each with a verification clause.
- **Risk:** medium, fully characterizable (see §4). Two invariants dominate: single write-path ownership (triggers fire on every mutation) and a correctly-scoped activation guard (resolved path + db, not raw id).
- **Boot impact:** `graph_ready_ms` 32,519 → **target sub-5,000** for the primary window; the second-screen double init_db disappears.

**Recommendation: Option C.** A and B are not discarded — they are the first two landable commits *inside* C. Workstream 2 is independent and should land **first** (MIG-079 §A) so the Boss can validate the double-init_db disappearance before the larger derivation change.

---

## 4. Invariants that must not break

1. **Edge completeness** — the link set to Sky View/wikilink resolution/Backlinks stays COMPLETE (every active link). No WHERE-filter may drop edges; `status='active'` (cache.rs:929) is the only legitimate filter.
2. **Tag-count accuracy** — `tag_counts` stays in sync with the aggregate of `note_meta.tags_json`; crash/bypass → stale with no error → mitigated by resumable backfill + `schema_versions` gate + live-scan fallback until the sentinel flips.
3. **Single write-path ownership** — every `note_links` / `tags_json` mutation flows through the trigger-covered path (`index_note` the single funnel). Audit must enumerate: rename cascade, link archival (status toggle), second-screen edits, federation merge, bulk re-index (drop+recreate triggers then recompute). Out-of-band UPDATE = the LL-XXX hand-rolled-index OOM class.
4. **No write-path / typing-latency regression** — trigger bodies O(affected rows), never O(table) (MIG-066 caveat). Per-keystroke save → debounced `index_note` → trigger must not regress. Hard constraint: no regression to typing latency / boot / IPC.
5. **Activation idempotency correctness** — early-return compares resolved/healed `final_path` AND `db.is_some()`, never the raw id, so a genuine re-activation after move/heal/consolidate or intentional DB close still reopens. Guard holds the lock across check-and-set.
6. **Five Acts idempotence** — `init_five_acts_system_notes` must not fire twice (the MIG-055 §H reason the reset exists); the guard preserves the single-seed guarantee.
7. **Second-screen display-only contract** — removing the activation call must not leave the second screen without a universe id if it mounts first; it needs `get_active_universe` + an active-changed subscription.
8. **Federation coherence** — per-Universe persisted snapshot composed cheaply at `federation:ready` (union of persisted tables), never a re-scan of every federated note; revision gate prevents a late/empty re-fire clobbering good data.
9. **Alias resolution** — `read_aliases` stays co-resolved with the link read (already 0 ms; leave alone).
10. **On-screen === on-disk after every transition** — harness asserts derived === live for the editor-surface-gate cases; devtools is dev-only → observable UI + harness assertions, never console checks.

---

## 5. Scope & composition with MIG-078

- **New migration: MIG-079**, composing with (not extending) MIG-078. Different subsystem boundaries — MIG-079 touches the **cache read path** + **multi-window activation/ownership**; MIG-078 touched **de-bloat / `note_body` / init serialization**. (Open decision #2 confirms the number.)
- **Reuses MIG-078's machinery directly:** the `schema_versions` per-module gate, the resumable-after-paint background backfill with status-bar progress, the MIG-078 §B1 init-serialization lock (already shipped — do not re-litigate).
- **Crosses subsystem boundaries (Rust cache ↔ Svelte boot order ↔ schema ↔ write path),** so the full four-phase `/migration` protocol applies.

---

## 6. Open decisions for the Boss (before Phase 2 Plan)

1. **Links strategy (b1 defer vs b2 materialize)** — the single biggest design fork.
2. **Scope / MIG number** — confirm MIG-079 vs MIG-078 §C.
3. **Tag-delta maintenance** — pure-SQL trigger (`json_each` on OLD/NEW) vs explicit ±1 in `index_note` Rust (more debuggable).
4. **Activation-guard sequencing** — land Workstream 2 first (recommended) vs bundle.
5. **Backfill UX** — confirm after-paint, status-bar progress, resumable, non-blocking (UX confirmation; gates the test tutorial).
6. **Second-screen snapshot re-fire** — keep all three sky re-fires (cheap once persisted) vs collapse to one + revision gate.

---

## 7. First measurement step (before any code)

On the live universe, capture the baseline so before/after is unambiguous:
1. Boot-perf harness 3× cold — confirm `read_links≈11.6s`, `read_tags≈5.6s`, `queue≈11.25s` reproduce.
2. `ipc_arrival_log` — confirm init_db fires **once** (second screen closed) vs **twice** (open) — isolates Workstream 2.
3. `EXPLAIN QUERY PLAN` on both boot reads — **DONE 2026-06-15:** `note_links WHERE status='active'` → `SCAN note_links` (234k rows, all active); `SELECT tags_json FROM note_meta` → `SCAN note_meta` (7,653 rows). Both confirmed full scans.
4. Per-keystroke save latency baseline — every new trigger amplifies this path; this is the regression guard for invariant #4.
