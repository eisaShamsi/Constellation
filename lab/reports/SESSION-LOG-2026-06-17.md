# Session Log — 2026-06-17 (MIG-079 §C.2b + §C.3)

> **Function in hand:** MIG-079 §C.2b — defer the 234k-row `read_links` off the boot critical path (the actual ~11 s boot win), then §C.3 covering index. Continues the bring-up program. Branch `main`. Active universe "Eisa Cognitive Knowledge" (7,653 notes, 1.97 GB DB).
> Picks up from `lab/reports/HANDOVER-2026-06-17-mig079-c2b.md`. §C.1 / §C.2a / save-path link-skip already SHIPPED + Boss-validated.

## Mapping (WA#4 impact, complete before any edit)

### Rust (`cache.rs`)
- `cache_boot_snapshot_graph` returns `{ links, tags, aliases }`. The `read_links_in_schema` scan (`SELECT source_path, source_name, target_name, link_type, library_name, weight, traversal_count, annotation, last_traversed, confidence FROM note_links WHERE status='active'`) is the 234k-row / ~11.3 s-cold cost.
- After §C.1, `read_tags_in_schema` already reads the `tag_counts` summary (~ms) when stamped; `read_aliases_in_schema` is ~ms (~1.4k rows). So dropping `read_links` leaves `cache_boot_snapshot_graph` genuinely fast (ensure_db + open_reader + count + tags + aliases).
- `cache_boot_snapshot` (back-compat shim) merges core+graph; used by ambient callers. Must keep returning FULL links (route through the extracted helper, not the now-empty graph).
- Index-build pattern: `idx_nl_tnl` is created in `incoming_links_backfill::run()` (off-boot, background, own conn, `IF NOT EXISTS`) — but that run is gated on the `incoming_links` stamp, already set on Eisa's DB, so it won't re-run. → a **dedicated** background builder for `idx_link_boot` with its own stamp (`link_boot_index`).

### Frontend consumers of the edge list (`allLibraryLinks` $state in `+layout.svelte`)
- `effectiveLibraryLinks` (derived wrap) → `getBacklinks`/`getOutgoingLinks` (sidebar effect 1328/1331) — **need full rows**.
- `linkTraversalMap` derived (759) — livePreview `×N` chips — needs full list (minor; fills when edges land).
- `buildSkyData` FALLBACK only (3239) — Sky View normally renders from the write-time `sky_*` payload (`cache_boot_snapshot_sky`); the edge-list fallback fires only when sky isn't ready → must `await ensureFullLinks()`.
- `applyConfidenceLocally`/`applyArchiveLocally` — local mutations of the loaded list (fine).
- **Four sites set `allLibraryLinks = graph.links`:** boot `loadGraph` (3190); federation:ready handler (2591) + post-init re-invoke (2644 — actually a *redundant second full-graph load at boot*); rename-cascade post-fix (5098). All re-route through `ensureFullLinks()`.
- **Sky View / GraphMindView / ConstellationSight / LocalSkyView** read `skyNodes`/`skyLinks` (own write-time source) — NOT the edge array. Untouched.
- **SecondScreenPage** fetches its OWN links via `scanLibraryLinks` (separate command) — untouched (display-not-domain holds).

## Predecessor → Replacement (Predecessor Lookup Rule)
- **Boot link load.** Now: `cache_boot_snapshot_graph.links` consumed at `+layout.svelte` boot `loadGraph` (3190) + 3 other sites; `read_links_in_schema` in `cache.rs`. Replacement: **same place / same module** — the edge read moves to a NEW `cache_full_links` command (same `cache.rs`, same `read_links_in_schema` helper), invoked lazily via a memoized `ensureFullLinks()` + `linksReady` flag in the SAME `+layout.svelte`. No feature relocated across components; no Settings/IPC surface dropped (the boot graph IPC stays, just stops shipping the edge array). The shim keeps full links.

## Design decision logged — §C.3 covering-index columns (deviation from the literal locked list, measurement-justified)
- Locked-design column list was `note_links(status, source_path, target_name, link_type, library_name, weight, traversal_count, last_traversed, confidence)` — it omitted `source_name` (REQUIRED by `getBacklinks`/`buildSkyData`) and `annotation`.
- **Measured (read-only, live DB):** 233,995 rows, ALL `status='active'`; 142,331 carry an annotation but total annotation text is only **1.33 MB** (~6 bytes avg — mostly the type word, often display-suppressed).
- For SQLite to report **USING COVERING INDEX**, the index MUST contain every column the scan SELECTs (a non-covering index on a full-table-returning query is ignored in favor of a table scan). So the index includes `source_name` + `annotation` → the bulk scan stays index-only AND every consumer stays byte-identical (NO new per-note annotation IPC, NO panel refactor). Lower-risk than the literal "drop annotation + fetch per row" reading; the 1.33 MB measurement makes inclusion negligible. `context` stays excluded (always `''` at boot).
- Final index: `idx_link_boot ON note_links(status, source_path, source_name, target_name, link_type, library_name, weight, traversal_count, last_traversed, confidence, annotation)`.

## Build sequence
1. **§C.2b** — Rust: drop `read_links` from `cache_boot_snapshot_graph`, add `cache_full_links`, shim via helper. Frontend: `ensureFullLinks()`/`linksReady`, panel `loading` prop, re-route the 4 link-load sites, universe-switch reset. Verify: cold `graph_ready` drops; Backlinks/Outgoing/Sky never-empty after open; Editor-Surface Gate.
2. **§C.3** — `link_boot_index.rs` background builder + schedule; assert `EXPLAIN QUERY PLAN = USING COVERING INDEX`; lazy scan faster.

## BUILT + Claude-verified (one combined commit — lib.rs interleaves both steps so they land together)
- **Files:** `cache.rs` (drop boot link read + `cache_full_links` + `BootLinks` + shim), `lib.rs` (register `cache_full_links` + `mod link_boot_index`), `search.rs` (schedule the index builder), `link_boot_index.rs` (NEW), `+layout.svelte` (`ensureFullLinks`/`linksReady`/guards/re-routes), `BacklinksPanel.svelte` + `OutgoingLinksPanel.svelte` (`loading` prop, reuse `common.loading`).
- **Verification done (Claude-side, pending Boss runtime validation):**
  - `svelte-check`: **0 errors** (315 pre-existing CSS warnings).
  - `cargo build --release`: clean (52 pre-existing dead-code warnings).
  - `cargo test --lib`: **935 passed / 0 failed / 6 ignored** — incl. cache:: per-schema link/alias isolation.
  - New `link_boot_index` tests: covering-index plan assertion (`USING COVERING INDEX idx_link_boot`) + stamp gate — **both pass**.
  - **Live-DB rehearsal (copy of the 1.97 GB DB):** before = `SCAN note_links`; after `CREATE INDEX idx_link_boot` (built in **2.0 s**) = `SEARCH note_links USING COVERING INDEX idx_link_boot (status=?)` — proven on the REAL 234k-row schema. (Warm scan ~450 ms both ways — materialization-bound; the covering win is on COLD page reads, which Eisa's boot-history will quantify.)
  - Frontend re-embed confirmed: `cache_full_links` + `_linksEpoch` present in `build/`.
- **Adversarial review (general-purpose agent on the diff):** no P0; all 7 audit questions PASS; Editor-Surface Gate clean (zero matches for `composeNoteModel|write_note|save_note|onDestroy|#key|.content =`). Two **P1 races found + FIXED before commit:**
  - **P1-1 universe-switch stale-edge race** — an in-flight `cache_full_links` from the OLD universe could resolve after the switch reset and write stale edges (empty-overwrite guard can't catch non-empty stale). **Fix:** `_linksEpoch` captured at call entry, bumped on switch, re-checked before every assignment → stale resolution discarded.
  - **P1-2 per-keystroke retry on failure** — the sidebar `$effect` re-runs per keystroke; a persistent `cache_full_links` failure would fire one IPC/char (keystroke-hot-path IPC ban). **Fix:** `_linksRetryAfter` 3 s cooldown gates the keystroke-driven retry; idle pre-fetch / force paths still retry.
  - Nits logged (defer to `/simplify`): orphaned non-schema wrappers `read_tags`/`read_links`/`read_untyped_links_fallback` now unused (pre-existing-style dead code).
- **Editor-Surface Gate:** §C.2b/§C.3 change only the link-edge READ path (a derived view of `note_links`); they touch NO note content / save / lifecycle code → the content-integrity class is structurally untouched (same class as §C.1/§C.2a derived views). Boss test still exercises the gate (Focus round-trip + tab switch + body-intact) as belt-and-suspenders.

## Pending Boss runtime validation (the gate before "validated")
Launch the rebuilt binary (mtime 2026-06-17 10:30). Stage 1: boot is faster (boot-history `graph_ready` drops from ~33 s) + Backlinks/Outgoing/Sky never-empty after open + the one-time `[link_boot_index]` diag line. Stage 2: Editor-Surface Gate (Focus round-trip, tab switch, body intact).
