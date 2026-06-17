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

## ⚑ Boss Stage-1 result + MEASURED pivot (SO #5 state-of-standing) — 2026-06-17 ~10:45

**Boss findings (10:45 boot of the 10:30 binary):** boot ~20 s to populate (not the hoped ~2 s floor); backlink rows appear then a **~7 s freeze** before the list scrolls (Boss: pre-existing, "since we created it"); Sky View **PASS**; step-5 **lag/non-responsiveness/thrashing** within a note + switching notes (pre-existing).

**Measured (boot-perf history `[15]` = the 10:45 boot — times are UTC `Z`, so `06:45:57Z` = 10:45 local; file mtime confirms):**
- `read_links = None`, `cache_snapshot_graph_server_timings = [ensure_db=0, open_reader=0, read_tags=164, read_aliases=0]` → **§C.2b deferral WORKS** — the 234k link read is off the boot graph IPC; its body is **164 ms** (was ~11,500 ms).
- `[link_boot_index] idx_link_boot built + stamped` in `diagnostics.log`; `idx_link_boot` confirmed present on the live DB → **§C.3 WORKS.**
- **BUT `graph_ready = 16,935 ms`, of which `cache_snapshot_graph_queue_ms = 14,991 ms`** — the graph IPC sat ~15 s in the dispatcher QUEUE before its (now-trivial) body ran. The `ipc_arrival_log` shows `cache_boot_snapshot_sky` invoked at +2094 ms then a ~14.3 s gap with no other IPC → the graph waits behind **`cache_boot_snapshot_sky`**.
- **`sky_links` = 233,995 rows** (measured) — the IDENTICAL twin of `note_links`. So §C.2b deferred ONE 234k cold read and the boot now waits ~15 s on its untouched twin (the Sky read).

**Honest conclusion:** the Plan/handover mis-attributed the boot cost ENTIRELY to `read_links` (11 s). The data shows it was ~7 s of a ~24 s boot; the **Sky read (~15 s) is the co-equal/bigger half**, and the **234k-edge → JS main-thread load is the root of the panel/editor freezes** (deferring moved its timing, not its cost). §C.2b is a correct, committed ~7 s improvement — NOT the headline win.

**Boss ruling (pivot):** **Kill the 234k in-memory edge array.** Switch Backlinks/Outgoing to **per-note SQLite queries** (like the write-time count badges) so the app NEVER holds all 234k links in JS. Cures the scroll-freeze + thrashing at the root; likely removes the Sky boot cost as a side effect. → opened as **MIG-079 §C.2c** (read-path change crossing Rust↔Svelte → `/migration`: Architect → Plan → approval → Build → Audit). WA#5 note: per-note backlink queries are the STANDARD PKM pattern (Obsidian/Logseq), not invention.

**State protected on `main` (`6a4d5e20`):** §C.2b (edge deferral + `cache_full_links` + guards + 2 P1 fixes) + §C.3 (`idx_link_boot` covering index, built on the live DB). Functionally correct + never-empty verified by Boss (rows appear, Sky passes). Editor-Surface Gate (Stage 2) NOT yet run — deferred (the content path is structurally untouched, so low risk, but run it before final close).

## §C.2c — Architect + Plan + WA#5 cross-check (Boss-approved with conditions)
- Architect+Plan: `docs/MIG-079-C2c-Architect-Per-Note-Link-Queries.md` (committed `49127c2f`). End-state: per-note SQLite row queries replace the in-memory 234k array; retires §C.2b's `cache_full_links`/`ensureFullLinks`/`linksReady`/`idx_link_boot`. Sky boot read = separate §C.2d.
- **WA#5 cross-check (Boss-required before approval; `0f80abc6`):** CONFIRMED the design as the textbook split — COUNT = write-time materialized (§C.2a, an aggregate), ROWS = read-time indexed per-note query (a bounded access path). Backlinks == the inverted-index posting list for a note's name (`idx_nl_tnl`). Materializing rows would tax the save path (wrong trade for a keystroke-latency-sacred app). Corroborated: materialized-view-vs-index guidance, IR/Lucene posting-list practice, SQLite-as-graph (per-node query > load-all), Obsidian Backlink Cache + Logseq→SQLite.
- **Boss ruling:** approved, **stop after Step 1 for proof**; build now.

## §C.2c-1 — Rust per-note row queries: BUILT + rehearsal PROVEN (the Step-1 gate)
- `cache.rs`: `get_backlink_rows(note_name, aliases)` (the §C.2a alias-aware `target_name_lower IN (...)` set on `idx_nl_tnl`, federated) + `get_outgoing_rows(note_path)` (`idx_link_source`, federated) → the exact `NoteLink` shape `read_links_in_schema` returns (context lazy). Registered in lib.rs. **Additive — no frontend wired, no behavior change yet.**
- **In-memory unit test** (name+alias match, archived excluded, empty-type→None) PASS. **Full lib suite 936/0** (+6/7 ignored), no regression.
- **Live-DB rehearsal (copy of the 1.97 GB DB), 80 notes (40 hubs + 40 mid/aliased):** backlinks `set_mismatch=0` (query == brute-force oracle) AND `count_mismatch=0` (deduped sources == `incoming_count`, which §C.2a already tied to `getBacklinks`); outgoing `edge-count mismatch=0` (== oracle == `outgoing_count`). **Proven: the per-note query equals today's getBacklinks/getOutgoingLinks input byte-for-byte.** (A rehearsal-assertion bug — comparing distinct-targets to `outgoing_count`=COUNT(*) edges — was caught + fixed; the QUERY was always correct.)
- **PAUSED per Boss instruction** for go/no-go before §C.2c-2 (the flagged frontend swap). No binary rebuild needed (Step 1 is backend-only, proven by data not GUI).
