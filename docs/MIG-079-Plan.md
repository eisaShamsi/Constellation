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
- **C.1 Tags:** ✅ SHIPPED + Boss-validated (`e372381a`). `tag_counts(tag, n)` summary maintained by a Rust ±delta in `index_note`/`reindex_delete_note` (gated `schema_versions.tag_counts`) + atomic `json_each` backfill + read-flip with live-fallback + reconcile self-heal. Measured: boot `read_tags` 5,658 ms → 4/36 ms.
- **C.2 Links:** **defer** the 234k-row `read_links` off boot. *(Impact analysis `wf_d9a26cf7`; Boss decisions 2026-06-16.)* Split into:
  - **§C.2a — write-time INCOMING aggregate (the enabler).** No incoming aggregate exists today; the count badges invert the legacy `outgoing_links_json` (type/weight/status-blind → drifts). Add `note_meta.incoming_count` **+ `incoming_link_types` + `incoming_top_rank`** (FULL mirror of MIG-066 outgoing — Boss call) maintained by `note_links_incoming_ai/ad/au` triggers, **keyed exactly like the Backlinks panel** (`getBacklinks`: lowercased `target_name` + `note_aliases` 3-tier expansion — the P0 correctness pin), + a resumable backfill mirroring `links_backfill.rs`. **Re-point `constellation_search_link_counts` onto the typed aggregate NOW** (Boss call; flag the badge-number shift as user-visible).
  - **§C.2b — defer the edge load.** Drop the 234k-edge array from the boot graph payload (keep tags + aliases). Sky View renders from the existing write-time `sky_links`/`sky_nodes` aggregate (already pre-shaped). New on-demand edge IPC behind `ensureFullLinks()` (memoized) + a `linksReady` flag; **idle pre-fetch right after `boot:graph-ready`** (Boss call) + lazy on first Backlinks/Outgoing/Graph open. **Guard every `allLibraryLinks` consumer on `linksReady`** (P0: never render empty). Move `clearLinkTraversalBumps` into the lazy-load completion (no double-count / stale window).
  - **§C.3 — covering index** `idx_link_boot` on `note_links(status, source_path, target_name, link_type, library_name, weight, traversal_count, last_traversed, confidence)` (annotation excluded — fetched in the lazy row query). The lazy scan reads index leaf pages only.
- **Verify:** `incoming_count` == `getBacklinks` row count on hub notes (live-DB rehearsal, §C.1 discipline); cold `graph_ready` drops (boot-history tool); Backlinks/Outgoing/Sky show correct, **never-empty** rows after open; `EXPLAIN QUERY PLAN` = `USING COVERING INDEX`; per-edge save latency unchanged (incoming trigger is O(edges-on-note)); Editor-Surface Gate; federation fallback awaits edges. Phase-4 audit (invariants/drift/migration-paths) on the landed diff.

## §D — Phase-by-phase bring-up (applies the cure to every Rule-8 offender)
Per the bring-up sequence (concept-papers §5): each function is re-enabled only after its concept-paper §10 checklist passes — the write-time cure for its Rule-8 violation, the shared right-click menu, the i18n fixes, its feature gate. Order: core → search → backlinks/graph/tags → Sky/Map/Sight/OrgChart → curation → federation → second screen. The confirmed defects (Calendar day-click, Tasks gate-bypass, Review-Pulse dead code, Command-Palette stubs) are fixed in their function's phase.

## Phase 4 — Audit
Three-agent audit (invariants / drift / migration-paths) on the landed diff + the Editor-Surface Gate where the write path is touched (§C). The §B1 init-mutex + the §A guard interaction is re-verified (no stale-conn, no missed init).
