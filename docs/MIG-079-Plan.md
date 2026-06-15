# MIG-079 — Plan (Phase 2): Boot WTD + Single-Owner Activation

> Phase 2 of `/migration`. Each step lands as one commit with a verification clause. Architect: [MIG-079-Architect-Boot-WTD-Graph-Snapshot.md](MIG-079-Architect-Boot-WTD-Graph-Snapshot.md). Boss decisions (2026-06-15): links **defer off boot**; tags maintained **in the indexer (Rust)**; **activation fix first**. This MIG is the Phase-3 satellite fix inside the bring-up program ([concept-papers/00-MASTER](concept-papers/00-MASTER-Bring-Up-Charter-and-Checklist.md)).

## §A — Single-owner activation (FIRST — kills the double-init)  [in build]
- **A.1** Idempotency guard at the top of `set_active_universe` (`universe.rs`): if the requested universe's path == the active path, return `Ok(())` — no `invalidate_search_state`, no second `init_db`. *(SHIPPED.)*
- **A.2** Second screen is display-only: remove `setActiveUniverse` from `SecondScreenPage.svelte:923` (it only needs the active name, already first in `listUniverses()`); drop the now-unused import. *(SHIPPED.)*
- **Verify (Boss):** relaunch with the second screen open → `diagnostics.log` shows `init_db` **once** (no second block ~34 s later); boot perceptibly faster; OrgChart still <2 s; universe **switch** still works (different path → still re-inits).

## §B — `safeBootMode` + missing gates (the minimal-mode switch)
- **B.1** `appSettings.safeBootMode` (default false), read once at boot; gate the 4 unconditional boot IPCs (`cache_boot_snapshot_graph`, the `federation:ready` listener, `listFiveActsNotes`, `getFederationWarnings`).
- **B.2** Add the missing `enabledFeatures` gates surfaced by the audit (Search Hub, Quick Switcher, Knowledge Health, Tension/Provenance, Forge/Canvas, Second Screen, Style Setter).
- **Verify:** `safeBootMode` on → editor-only boot; capture the clean editor baseline (paint/hydrated/graph_ready) for the regression harness.

## §C — Boot graph Write-Time Derivation (the 30 s)
- **C.1 Tags:** maintain a `tag_counts(tag, n)` summary **in `index_note` (Rust ±delta)** + resumable backfill (gate `schema_versions.tag_counts`) + flip `read_tags` to the table. `read_tags` 5.6 s → <1 ms.
- **C.2 Links:** **defer** the 234k-row `read_links` off the `graphReady` critical path — boot reads the persisted `sky_links` + `note_meta.outgoing_*` aggregates; the full edge list loads lazily on first graph/panel open.
- **C.3** Covering index on `note_links` for any residual in-order read.
- **Verify:** `graph_ready_ms` 32.5 s → <5 s; search-identity preserved; per-keystroke save latency unchanged (the trigger/indexer delta is O(tags-on-note)).

## §D — Phase-by-phase bring-up (applies the cure to every Rule-8 offender)
Per the bring-up sequence (concept-papers §5): each function is re-enabled only after its concept-paper §10 checklist passes — the write-time cure for its Rule-8 violation, the shared right-click menu, the i18n fixes, its feature gate. Order: core → search → backlinks/graph/tags → Sky/Map/Sight/OrgChart → curation → federation → second screen. The confirmed defects (Calendar day-click, Tasks gate-bypass, Review-Pulse dead code, Command-Palette stubs) are fixed in their function's phase.

## Phase 4 — Audit
Three-agent audit (invariants / drift / migration-paths) on the landed diff + the Editor-Surface Gate where the write path is touched (§C). The §B1 init-mutex + the §A guard interaction is re-verified (no stale-conn, no missed init).
