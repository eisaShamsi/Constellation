# MIG-001 — Sky View Write-Time Derivation

**Status**: Phase 1 complete. Awaiting Phase 2 plan.
**Scope**: Persist Sky View nodes + links in SQLite, maintained by triggers on `note_meta` / `note_links`. Replace JS `buildSkyData()` with direct table reads. Extend pattern to Sight (centrality cache) and sidebar counts in follow-ups.
**Chosen option**: C (full WTD) — calculated risk accepted by user.

## Phase 1 — Architect

### Territory

- **Frontend consumers**: `skyNodes`/`skyLinks` in `+layout.svelte` (L691–692); `localSkyNodes`/`localSkyLinks` ego filter (L1126–1127); second-screen duplicate in `SecondScreenPage.svelte` (L431, L497).
- **Types**: `SkyNode` and `SkyLink` in `src/lib/libraries/store.ts` L1924–1941.
- **Current assembly**: `buildSkyData(allLinks, allNotes)` at store.ts L1943–1982. O(N+E) JS pass.
- **Boot IPC**: `invoke('cache_boot_snapshot_graph')` (cache.rs L211–286) + `cache_boot_snapshot_core`.
- **Rust build**: `cache.rs` reads `note_links` (WHERE status='active') + `note_meta` projection.
- **Enrichment**: `strata.rs` populates `stratum`/`maturity`/`originType` in a second pass.
- **Secondary consumers**: `clusterEngine.ts` (Louvain — JS), `lens.rs` (centrality — Rust, on-demand), Sight/Map/Backlinks panels.

### Invariants (MUST NOT BREAK)

1. **Orphan notes remain in SV**: notes with zero incoming AND zero outgoing links still render as nodes. `buildSkyData` preserves this; `sky_nodes` must too.
2. **Link dedup by `source→target:type`**: duplicate wikilinks in a single note collapse to one edge with accumulated counts.
3. **`id = name.toLowerCase()` cross-library collision**: frontend relies on this shape. Changing to path-based ids = larger refactor. Keep string-name ids, but store path too.
4. **Archived links excluded**: `WHERE status = 'active'` must hold in the sky_links surface.
5. **Second-screen parity**: whatever the main window sees, the second screen sees (display-not-domain rule).
6. **Rename preserves edges**: `update_links_on_rename` must cascade into `sky_nodes` (path change) and `sky_links` (target_name change).
7. **Enrichment fields present**: `stratum`/`maturity`/`originType` must still populate the SkyNode shape.
8. **No regression in typing latency**: every note save already updates `note_links`; adding triggers must not push per-save latency above current baseline on a 100-edge note.

### Speed/cost estimate

| Option | Boot time | Effort | Risk | Incremental writes |
|--------|-----------|--------|------|-------------------|
| A: Rust-side shape | ~500ms (3–5× faster) | 0.5 day | Low | No |
| B: Snapshot blob | ~200ms (10×) | 1 day | Medium | Debounced |
| **C: Full WTD** | **~200–400ms (3–5×)** | **2–3 days** | **Higher** | **Yes, per write** |

C's boot win over B is modest (IPC floor ≈ 150ms for 217k edges). **C's real value is write-time incrementality** — new wikilink in a note updates `sky_links` within the same write transaction, so every downstream consumer (Sight, Map, counts) reads fresh data without rebuilding.

### Risks (calculated)

1. **Trigger coverage** — 9 writer sites for `note_links` (search.rs L623, 1389, 1743, 1812, 1833, 1841, 1869, 1891, 2033). Triggers on the table catch them all by design, but only if the triggers fire on all relevant ops (INSERT/UPDATE/DELETE). Mitigation: write triggers against the base table, test each writer site.
2. **Back-fill on 7,294 notes / 217k edges** — must be resumable, background, progress-reported. LL-XXX prior OOM is the warning. Mitigation: cursor-based populator in a separate transaction per 1,000 rows, `sky_backfill_progress(id, last_path)` state row.
3. **Schema version bump** — `FTS_SCHEMA_VERSION` pattern is the template. New `SKY_SCHEMA_VERSION` gates back-fill on upgrade.
4. **Rollback** — if user rolls back to pre-MIG-001 build, old code bypasses `sky_*` triggers → drift. Mitigation: tables are read-only to the old code (it still uses buildSkyData); on next upgrade, re-run back-fill if SKY_SCHEMA_VERSION mismatches. Triggers are idempotent.
5. **Enrichment** — `stratum`/`maturity`/`originType` come from `strata.rs` which writes to `note_meta.properties_json` (or similar). Triggers on `note_meta` UPDATE catch them; add enrichment columns to `sky_nodes` table.
6. **Second-screen** — swap its `buildSkyData` call to the new read path in the same migration, else parity breaks.
7. **Dense-graph write amplification** — per-save trigger work scales with note's outgoing count. Mitigation: trigger uses the same DELETE + INSERT pattern `index_note` already uses, so write cost is roughly 2× current. Measure before calling acceptable.
8. **`id` collision** — two notes named "Introduction" in different libraries share an id today. Carry forward as-is. `sky_nodes` PK is `path` (unique), `id` is a derived column.

### Decision

Proceed with C. Accept risks 1–8 with the listed mitigations.

## Phase 2 — Plan (11 steps)

| # | Step | /simplify? | Verify |
|---|------|-----------|--------|
| 1 | Introduce `SKY_SCHEMA_VERSION` constant + schema gate | — | App boots, FTS unaffected |
| 2 | Create `sky_nodes` + `sky_links` tables (empty, no triggers) | — | Fresh/existing DBs upgrade without data loss |
| 3 | Triggers on `note_links` → `sky_links` | ✔ | Insert/archive via any of 9 writers produces correct deltas |
| 4 | Triggers on `note_meta` → `sky_nodes` (incl. rename path) | — | Create/rename/delete; orphans preserved |
| 5 | Resumable back-fill populator (`sky_backfill.rs`) | — | 7,294-note cold boot <1s to paint; kill+resume works |
| 6 | Rename cascade coverage (`update_links_on_rename`) | ✔ | Rename X → all source/target rows updated |
| 7 | Strata enrichment → triggers populate `stratum`/`maturity`/`origin_type` | — | No stale null enrichment columns |
| 8 | New IPC `cache_boot_snapshot_sky` (direct table read) | — | Byte-diff identical to current buildSkyData output |
| 9 | Frontend swap: main window (`store.ts`, `+layout.svelte`) | ✔ | SV renders identical; ego filter unchanged; typing latency unchanged |
| 10 | Frontend swap: second-screen parity | — | Second-screen matches main |
| 11 | Phase-4 audit + cleanup (delete dormant `buildSkyData`) | — | Write-amp <2× baseline; zero drift vs. fresh back-fill |

Full plan: see commit message of §64 and the Plan agent return.

## Step 7 — Enrichment scope decision

Audited strata.rs / maturity.rs / provenance.rs: all three are pure
filesystem scanners that compute on demand and return to the frontend.
No existing persistence path; the Phase-1 assumption of an
`UPDATE note_meta SET properties_json=...` write-side hook was wrong.

Decision: keep the enrichment columns (stratum/maturity/origin_type)
as forward-compat placeholders in sky_nodes, leave them NULL for
MIG-001 v1. Frontend continues calling the compute commands
separately after Step 8's new IPC returns the base graph. Enrichment
WTD migration deferred to future MIG-002 as its own focused task.

Rationale: per LL-023, enrichment is orthogonal to Sky View's primary
perf pain (node + edge serialization across IPC). Expanding MIG-001
to include enrichment would triple the trigger surface and invite
the scope-creep failure mode. The shape of sky_nodes already matches
the SkyNode TypeScript interface, so MIG-002 can flip NULL → populated
without a schema change.

**Next**: Phase 3 — Build, starting with Step 1.
