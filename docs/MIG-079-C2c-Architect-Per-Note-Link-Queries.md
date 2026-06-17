# MIG-079 §C.2c — Architect: per-note link queries (kill the 234k in-memory edge array)

> Phase 1 of `/migration`. Opened 2026-06-17 after the §C.2b measurement pivot (Boss ruling: "Kill the 234k in-memory array"). Read with: `lab/reports/SESSION-LOG-2026-06-17.md` (the measured pivot), `docs/MIG-079-Plan.md` (§C), orientation v2.87.

## 1. The problem (measured, not assumed)

§C.2b deferred the 234k-row `note_links` read off the boot graph IPC (proven: `read_links=None`, graph body 164 ms). But the Boss test + boot-perf history showed:
- **Boot still ~17–20 s** — the graph IPC waits `queue=14,991 ms` behind **`cache_boot_snapshot_sky`**, which reads the **233,995-row `sky_links`** table (the twin of `note_links`). *(Separate issue — see §6.)*
- **A ~7 s freeze when scrolling a note's backlinks, plus lag/thrashing within and between notes** (Boss: pre-existing). **Root cause:** the app loads all **233,995** edges into a JS array (`allLibraryLinks`) and the reactive derivations (`effectiveLibraryLinks`, `linkTraversalMap`) iterate them on the main thread. Deferring *when* that array loads (§C.2b) moved the cost in time; it did not remove it.

**The structural defect:** the Backlinks/Outgoing panels only ever need **one note's** links, but the app holds **every** note's links in memory and filters the whole array per note (`getBacklinks`/`getOutgoingLinks` are `allLinks.filter(...)`). That is the read-time-over-everything shape Rule 8 forbids — the counts were already fixed write-time (§C.2a `incoming_count`/MIG-066 `outgoing_count`); the **rows** were not.

## 2. End-state (the cure)

**Never hold the full edge list in JS.** The panels query **per-note** from SQLite, exactly as the count badges already do:
- **Backlinks rows** — a Rust IPC `get_backlink_rows(note_name, aliases)` that returns the rows using the SAME alias-aware UNION the §C.2a count uses (`incoming_aggregate_assignments`, search.rs:909): active edges whose `target_name_lower` = the note's name OR any alias (two index-seeking branches on `idx_nl_tnl`), SELECTing the row columns instead of `COUNT(DISTINCT)`. Dedupe-by-source + type accumulation done in Rust (mirror `dedupeBySource`).
- **Outgoing rows** — `get_outgoing_rows(note_path)`: `SELECT … FROM note_links WHERE source_path=? AND status!='archived'` (seeks `idx_link_source`). Trivial.
- Both return the EXACT shape `getBacklinks`/`getOutgoingLinks` produce today (the panels are unchanged below the data source).

**Reuse, not invention (WA#5).** In-house: §C.2a already computes the alias-aware backlink set in SQL on `idx_nl_tnl`. External: Obsidian's *Backlink Cache* maintains a per-note backlink index queried on demand — the proven 10k-vault pattern; the field does NOT hold all edges in memory for the panel. ([obsidianstats backlink plugins](https://www.obsidianstats.com/tags/backlink), [Obsidian SQLite persistent index](https://glama.ai/mcp/servers/@suhailnajeeb/obsidian-mcp/blob/1e324b1c977d0ae1aa2a6f4ef09f46bf67ef3f17/obsidian_mcp/utils/persistent_index.py))

## 3. What this supersedes / retires

§C.2c is the END-STATE that §C.2b's lazy-array was a stepping-stone toward. Once the panels query per-note, the in-memory array machinery is DEAD and gets removed:
- `allLibraryLinks` $state + `effectiveLibraryLinks` + the array-wide `linkTraversalMap` derivation.
- `ensureFullLinks()` / `linksReady` / `cache_full_links` (Rust) / the panel `loading` prop.
- `idx_link_boot` (§C.3) — built to make the *bulk* scan covering; with no bulk scan it is orphaned → DROP it (the per-note queries ride `idx_nl_tnl` + `idx_link_source`). **Honest note:** §C.3's covering index becomes moot under §C.2c. §C.2b's lasting value was the MEASUREMENT that redirected us (sky, not links) + confirming the never-empty UX path.
- The 4 `allLibraryLinks = graph.links` reroutes collapse (no array to refresh).

## 4. Invariants that must not break (the per-note query == getBacklinks exactly)

1. **Alias-aware** — name + every alias (frontmatter + rename-stamped), via the `idx_nl_tnl` UNION.
2. **Dedupe-by-source** — one row per source note; both `[[X]]` and `[[type::X]]` collapse, type badges accumulate (`linkTypes[]`).
3. **Status filter** — exclude `archived`.
4. **Sort** — Living-Link weight desc (decay-aware when enabled), tie-break by source name (backlinks) / target (outgoing).
5. **Per-row fields** — source_name/path, library_name, link_type(s), traversal_count, last_traversed, lifecycle `tier`, confidence, annotation. `context` stays empty (lazy, as today).
6. **Count == rows** — the `incoming_count`/`outgoing_count` badge must equal the row count the per-note query returns (same `matched` set → guaranteed by construction; assert in a rehearsal).
7. **Live mutations** — confidence change / archive must reflect instantly (re-query the active note, or patch the small per-note result; NOT a global array).
8. **The `×N` traversal chips** in the open note's body derive from the OPEN note's outgoing rows (which carry `traversal_count`) — not a global map.
9. **Editor-Surface Gate** — read-path only; no note content/save/lifecycle touched.

## 5. Options

- **Option A (recommend) — full per-note queries + virtualized panels.** Convert Backlinks/Outgoing/×N to per-note IPCs; remove the array machinery; **virtualize** the Backlinks/Outgoing lists (CLAUDE.md: virtualize any list > 50 — hub `ISBN` has 5,358 backlinks; the query is fast/indexed, rendering must virtualize). Behind a `perNoteLinkQueries` flag until Boss-validated, then delete the old path.
- **Option B — hybrid.** Per-note for panels, keep a bulk array for Sky/other. Rejected: leaves the 234k array (and its freeze) in memory; defeats the purpose.
- **Option C — per-note without virtualization.** Rejected: a 5,358-row render on a hub note re-introduces a main-thread stall.

## 6. Out of scope (separate follow-ups, named honestly)

- **The boot ~15 s** is the **Sky read** (`cache_boot_snapshot_sky` over 234k `sky_links`), NOT the panel array. The per-note fix does **not** remove it (corrects the option-card optimism). The SAME structural model applies to Sky next — defer/lazy the sky read off boot (§C.2d candidate). Flagged, not bundled.
- The right-sidebar relocation `/migration` (already designed, post-§C).

## 7. Migration path / safety

- **No schema change** — `note_links`, `idx_nl_tnl`, `idx_link_source` already exist. The per-note queries work on first boot.
- **Flag-gated swap** — `perNoteLinkQueries` (default off → on after rehearsal). Old array path stays until the swap is Boss-validated, then removed in a cleanup commit.
- **Rehearsal gate (the §C.1/§C.2a discipline)** — on a copy of the live DB, assert `get_backlink_rows` == today's `getBacklinks` (alias-aware, deduped, sorted) for a sample of hub + leaf + aliased + renamed notes, and `count == rows`. Red→green before the swap.
- **Editor-Surface Gate** — content path untouched; the gate still runs (Focus round-trip, tab switch, body intact).

## 8. Phase-4 audit

Invariants (the 9 above) / drift (new guards) / migration-path (first-boot, hub note, aliased/renamed target, archived edge, mid-backfill). Plus the live-DB rehearsal equivalence.

---

# Plan (Phase 2) — each step one commit with a verify clause

**§C.2c-1 — Rust per-note query IPCs (no frontend change yet).**
Add `get_backlink_rows(note_name, aliases)` (reuse the §C.2a `matched` UNION, SELECT row columns, dedupe-by-source + type accumulation in Rust) and `get_outgoing_rows(note_path)` (`idx_link_source`). Unit tests mirror the §C.2a row semantics (alias, dedupe, archived-excluded, sort).
*Verify:* `cargo test` green; a **live-DB rehearsal** asserts `get_backlink_rows` == today's `getBacklinks` and `get_outgoing_rows` == `getOutgoingLinks` (alias-aware, deduped, sorted, byte-identical rows) for hub/leaf/aliased/renamed sample notes, and `count == rows`.

**§C.2c-2 — Frontend swap behind `perNoteLinkQueries` flag.**
The sidebar effect (the existing 500 ms-debounced, off-keystroke block) calls the two IPCs for the active note instead of filtering `effectiveLibraryLinks`. `applyConfidenceLocally`/`applyArchiveLocally` patch the small per-note result (or re-query). The `×N` chips derive from the active note's `get_outgoing_rows`. Flag default OFF (old array path intact).
*Verify (Boss):* flag ON → open a hub note (e.g. ISBN) + a leaf note + an aliased/renamed note → Backlinks/Outgoing rows, type badges, ×N chips, sort order, and the count badge ALL match the old path exactly; **no scroll freeze**; confidence/archive right-click updates instantly.

**§C.2c-3 — Virtualize the Backlinks + Outgoing lists.**
Render only visible rows (the file-tree/search virtualization pattern) so a 5,358-row hub note never stalls the main thread.
*Verify (Boss):* open ISBN (5,358 backlinks) → instant, smooth scroll; switching notes is responsive (the step-5 thrashing gone).

**§C.2c-4 — Remove the dead array machinery (the cleanup swap).**
Flip `perNoteLinkQueries` ON by default, then delete `allLibraryLinks`/`effectiveLibraryLinks`/the global `linkTraversalMap`/`ensureFullLinks`/`linksReady`/`cache_full_links`/the panel `loading` prop/the 4 reroutes, and DROP `idx_link_boot`. `/simplify` the diff.
*Verify:* boot has no edge-array load at all; `svelte-check` 0; full Editor-Surface Gate; cold boot + panel open both smooth.

**Phase 4 — audit** (§8) on the landed diff.

*(Sky boot read — the remaining ~15 s — is a SEPARATE follow-up §C.2d, same model applied to `cache_boot_snapshot_sky`/`sky_links`. Not in §C.2c.)*
