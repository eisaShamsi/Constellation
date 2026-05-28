# MIG-061 — Federation Audit Findings

**Date:** 2026-05-28
**Trigger:** MIG-060 Boss-test surfaced CNS + Cataloger federation gaps. Eisa requested full audit of how core surfaces handle Universes that contain cUniverses.
**Method:** Four parallel exploration agents surveyed the codebase (graph surfaces, cataloger/classifier, sidebar panels, search/index + other dock surfaces). Output below is the consolidated table + root-cause groupings + recommended scope options.

---

## Federation status — one table, all surfaces

Legend:
- ✓ **federated** — uses `federated_conn` (ATTACH'd) or correctly aggregates across cUniverses.
- ✗ **broken** — reads only parent universe; cUniverse data silently absent.
- ◑ **partial** — federates some data, misses others.
- N/A — single-note / universe-scoped by design / pure client-side.

### Working today (✓ federated — 4 surfaces)

| Surface | Tauri command | File:line |
|---|---|---|
| **libraryStats** (sidebar 8 751-note count) | `get_all_library_stats` | `src-tauri/src/libraries.rs:529` (UNION ALL with race-condition handler at 481–507) |
| **Search Hub / QuickSwitcher** | `constellation_search` → `federated_lexical_search_or_fallback` | `src-tauri/src/search.rs:5859` + 3931 |
| **Lens execution** (`base` blocks) | `execute_lens` | `src-tauri/src/lens/query.rs:58` (uses `build_federated_sql` + UNION ALL) |
| **Federation Warnings popup** | `federation_get_warnings` | `src-tauri/src/federation/mod.rs:62` |

### N/A by design (5 surfaces)

| Surface | Why N/A |
|---|---|
| **360.3D Inspector** | Per-note scope; reads the focused tab only |
| **Bookmarks** | Universe-scoped by design — `bookmarks.json` lives per-universe |
| **Global Tasks** | Filesystem walk of *active universe's* libraries (federating across cUniverses might be wanted as a separate decision) |
| **Expression Forge / Sense-Making Canvas / Dashboard** | Pure client-side; no DB read path |
| **Constellation Map** (sunburst arcs only — see Org Chart row in ◑ for the tree-builder side of `map.rs`) | Per Agent 1 — sunburst reads filesystem; check Org Chart row for the tree side |

### Partial — federates some data (1 surface)

| Surface | Tauri command | What works | What's broken |
|---|---|---|---|
| **Org Chart** | `constellation_map_universe` (`map.rs:303`) | cUniverses appear in the tree structure | `load_alias_map` (line 310) reads alias_to_path from active universe only → cross-universe note renames don't resolve via child-universe aliases |

### Broken — silent parent-only behavior (14 surfaces)

#### Group A — `cache_boot_snapshot_sky` choke point

This single Tauri command (`src-tauri/src/cache.rs:382`) feeds **four** consumer surfaces. Fixing it federates all four in one stroke.

| Surface | How it consumes the broken data |
|---|---|
| **CNS (Constellation Nervous System)** | Reads `skyNodes` / `skyLinks` — populated by `cache_boot_snapshot_sky`. Verified: 987 of 8 751 nodes shown in Eisa Universe. |
| **Sky View** | Same `skyNodes` data, filtered client-side |
| **Backlinks panel** | `getBacklinks` (`store.ts:2518`) filters `allLibraryLinks` — derived from same source |
| **Outgoing Links panel** | `getOutgoingLinks` (`store.ts:2569`) — same source |

Root cause: `read_sky_nodes_raw` (`cache.rs:485`) and the link scans use the bare `Connection` from `open_reader()`. No UNION across `cu*.sky_nodes`, `cu*.sky_links`.

#### Group B — Cataloger / Classifier / NSC backend (4 surfaces)

| Surface | Tauri command | Failure mode |
|---|---|---|
| **The Cataloger (full-page)** | `classifier_scan_start` → enumerates `note_meta` (`classifier/scan_job.rs:236–244`) | Scan stops at parent universe — cUniverse notes never classified |
| **Classifier single-note** (gesture, "Classify a note…", right-sidebar) | `classifier_suggest_for_note` (`classifier/mod.rs:40`) | INSERT into `sources_suggestions` (line 622) fails FK constraint `sources_suggestions(note_path) → note_meta(path)` (`sources/mod.rs:173`) when note lives in cUniverse |
| **NSC Backfill** ("Build all summaries") | `nsc_backfill_start` (`nsc/backfill.rs:97`) | Enumerates bare `note_meta`; cUniverse notes never summarised |
| **Source Review queue (list)** | `sources_list_pending_suggestions` (`sources/mod.rs:687`) | Works as-is *only because* the table itself is parent-only by current design — but that's the symptom of the upstream gap, not a real fix |

Root cause: backend uses `state.db` (bare parent conn), never `state.federated_conn`. The FK constraint design is parent-coupled.

#### Group C — Filesystem-walk surfaces (3 surfaces)

| Surface | Tauri command | Failure mode |
|---|---|---|
| **Tag Browser** | `scan_library_tags` (`libraries.rs:2236`) | Filesystem walk of one library path; no per-cUniverse enumeration |
| **Five Acts sidebar** | `list_five_acts_notes` (`lens/system_notes.rs:152`) | Reads `{active_universe}/Five Acts/` — cUniverse Five Acts dirs invisible |
| **Workspace Bases** | `list_workspace_bases` (`bases.rs:717`) | Reads parent's `workspace_bases_dir()` — cUniverse `.base` files invisible |

Root cause: hardcoded `universe_root.join(...)` style paths; no cUniverse directory enumeration loop.

#### Group D — FTS / read-path queries that bypass federated_conn (3 surfaces)

| Surface | Tauri command | Failure mode |
|---|---|---|
| **Unlinked Mentions** | `scan_unlinked_mentions` (`libraries.rs:2097`) | Queries `notes_fts` without `UNION cu*.notes_fts` |
| **Index panel** (vocab + term mentions) | `read_index_entries`, `read_term_mentions` (`libraries.rs:3425`, 3550) | Opens its own read-only Connection with `SQLITE_OPEN_READ_ONLY` (lines 3429–3433); no ATTACH applied. Cross-language bridge terms (M11) invisible across cUniverses |
| **Knowledge Health** | `constellation_link_stats` (`search.rs:4547`) + `constellation_formulation_analysis` (4789) | All `note_links` queries hit parent only — bias checks, weak-foundation finds, most-connected ranking all skip cUniverse data |

Root cause: each command does its own `open_reader()` or pulls `state.db` without re-using the warm `federated_conn`.

#### Group E — Right-sidebar previews / trail

| Surface | Failure mode |
|---|---|
| **Right-sidebar trail / preview** | Various detail queries (note preview, link counts) use `state.db` — federated notes get blank previews or wrong counts |

---

## Shared root-cause summary

Three patterns cover all 14 broken surfaces:

| Pattern | Broken surfaces |
|---|---|
| **P1 — `cache_boot_snapshot_sky` not federated** (one Tauri command, 4 dependent surfaces) | CNS, Sky View, Backlinks, Outgoing |
| **P2 — Backend command uses bare `state.db` instead of `federated_conn`** | Cataloger, Classifier (single-note + scan), NSC, Index (entries + mentions), Knowledge Health, Right-sidebar previews |
| **P3 — Hardcoded `{active_universe}` filesystem paths** (no cUniverse enumeration loop) | Tag Browser, Five Acts sidebar, Workspace Bases |
| **P4 — FK constraints to parent's `note_meta`** (compounds P2 for write paths) | Cataloger/Classifier INSERT path |

The **MIG-056 federation work** (libraryStats / search / lens) established the pattern correctly. It just wasn't applied beyond those three surfaces. P1, P2, P3 are all the same shape of fix: switch the read site to use `federated_conn` (P1, P2, D) or add a cUniverse enumeration loop (P3). P4 is separate — it's a schema design choice that needs its own decision (replicate note_meta entries, drop the FK, or move suggestions to per-cUniverse tables).

---

## Scope options for the fix MIG

### Option A — One mega-MIG fixes all 14 surfaces

- One Architect → one Plan → 14-step cascade.
- Pros: pattern fixed wholesale; no half-federated state.
- Cons: large plan, large risk, large audit. Probably 2–3 sessions.

### Option B — Pattern-grouped MIGs (recommended)

Four small MIGs, ordered by user-visible impact:

| MIG | Scope | Surfaces fixed | Effort |
|---|---|---|---|
| **MIG-061** | Fix `cache_boot_snapshot_sky` (P1) | CNS, Sky View, Backlinks, Outgoing | **Highest impact / lowest risk** — one command, 4 surfaces |
| **MIG-062** | Filesystem-walk federation (P3) | Tag Browser, Five Acts, Workspace Bases | Small — add a cUniverse enumeration loop to 3 commands |
| **MIG-063** | Read-path federated_conn switch (P2, read-only) | Index entries, Index mentions, Unlinked Mentions, Knowledge Health, right-sidebar previews | Medium — 5 commands, but no write-path FK concerns |
| **MIG-064** | Cataloger/Classifier/NSC (P2 + P4) | Cataloger, Classifier (scan + single-note), NSC | Largest — write-path FK decision needed first (Architect-locks) |

Pros: ships value incrementally; each MIG is testable on its own; clean rollback boundary if one MIG regresses.
Cons: requires Architect+Plan+Build+Audit per MIG (4×).

### Option C — Just fix CNS (MIG-061 scope only)

- One MIG, smallest. Fixes the surface that triggered this audit.
- Pros: fastest path to "CNS shows all 8 751 nodes."
- Cons: leaves 10+ surfaces still broken; the next user discovery of another gap reopens the conversation.

### Recommendation

**Option B** — patterns are genuinely different (P1 is one command, P3 is filesystem, P4 is schema-design). Each pattern gets its own Architect with its own invariants. MIG-061 (the P1 fix) ships first because it unblocks the most surfaces per unit work.

---

## Boss decision points

1. Which Option (A / B / C)?
2. If B: confirm MIG-061 scope = "Fix `cache_boot_snapshot_sky` to federate across cUniverses, unblocking CNS / Sky View / Backlinks / Outgoing."
3. P4 (FK design) Architect question to surface in MIG-064: replicate note_meta entries for cUniverse paths into parent? Drop the FK? Move `sources_suggestions` to per-cUniverse DBs?

---

## Appendix — agent transcript locations

- Agent 1 (graph surfaces): `tasks/a84c95acd4226eb64.output`
- Agent 2 (cataloger/classifier): `tasks/a1995145d5c73c48a.output`
- Agent 3 (sidebar panels): `tasks/a5d1e3053dcb8887c.output`
- Agent 4 (search/index + dock): `tasks/a9036dd2be2658738.output`

Raw transcripts retained for the next-session reader who wants to verify any specific file:line claim above.
