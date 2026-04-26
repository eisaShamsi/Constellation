# MIG-005 — Alias-Aware In-Memory Inbound Consumers

**Status**: Phase 1 complete (Architect). Phase 2 plan pending user approval.
**Scope**: Make six in-memory inbound-link consumers (5 Rust commands + 1 Svelte component) consult `note_aliases` so they stop mis-counting / mis-classifying / silently dropping wikilinks whose target was renamed. Closes MIG-004 audit deferrals 4B-1 and 4B-2.

---

## Phase 1 — Architect

### Why this exists

MIG-004 made the SQL-side stratum / maturity / Sky View boot snapshot / Backlinks panel alias-aware. **It did NOT update five Rust commands that compute the same kind of inbound information in-memory** (Sight, Inspector 360, Tension, Map, plus a stratum overlap), nor the LinkDashboard Svelte component which classifies wikilinks as "broken targets" without consulting the alias table. Result: after a rename, those surfaces under-count, mis-classify, or visibly omit aliased wikilinks even though the canonical SQL-driven surfaces (Sky View, Backlinks) show them correctly. The user sees inconsistent inbound counts across panels and false "broken" badges in the Link Dashboard.

The reference pattern already exists: `cache.rs::read_sky_links_raw` (MIG-004 §8) does a **3-tier resolution** — `name_to_idx → alias_to_path → unresolved`. MIG-005 reuses that pattern in each of the affected surfaces.

### Surface inventory (file:line citations from parallel surveys)

| File | Lines | Semantic | User-visible feature | Alias-aware today? |
|---|---|---|---|---|
| `strata.rs` | 73–81 | inbound-count | GraphMind View / stratum assignment | **No** — name-only equality |
| `strata.rs` | 141–157 | outgoing-scan (regex) | Source of `outgoing` Vec | N/A (regex parses raw) |
| `strata.rs` | 174–209 | stratum / orphan | Inherits via pre-computed fields | No (inherits bug) |
| `maturity.rs` | 44–49 + 88–144 | outgoing-scan (regex) | Source of `outgoing_targets` | N/A |
| `maturity.rs` | 54–61 | inbound-count | SkyView maturity coloring | **No** — name-only |
| `tension.rs` | 82–92 | inbound-count | Orphan + SPOF detection | **No** |
| `tension.rs` | 113–125 | contradiction-detection | TensionPanel "Contradictions" | **No** ⚠️ HIGH IMPACT (silently drops contradictions to aliased targets) |
| `tension.rs` | 130 | orphan-detection | TensionPanel "Orphans" | **No** |
| `tension.rs` | 164–171 | structural-gap | TensionPanel "Structural Gaps" | **No** |
| `tension.rs` | 194–210 | single-point-of-failure | TensionPanel "SPOF" | **No** |
| `tension.rs` | 246–252 | outgoing-scan | Source of `outgoing` | N/A |
| `inspector360.rs` | 107–122 | outgoing-list | Inspector360.svelte typed_links / untyped_links | **No** |
| `inspector360.rs` | 125–142 | **inbound-LIST** (not just count) | Inspector360.svelte ⚠️ HIGH IMPACT (named lists are visibly incomplete) | **No** |
| `inspector360.rs` | 145–163 | 2nd-order expansion | Inspector360.svelte depth-2 nodes | **No** |
| `inspector360.rs` | 334–349 | stratum-compute | Inspector360.svelte stratum header | **No** |
| `inspector360.rs` | 360–392 | provenance-walk | Inspector360.svelte origin / trust chain | **No** |
| `inspector360.rs` | 411–444 | trails (basename match) | Inspector360.svelte trails panel | **No** |
| `map.rs` | 71–83 | inbound-count (bubble sizing) | ConstellationMap.svelte | **No** ⚠️ DOC DRIFT |
| `map.rs` | 128–133 | inbound-count (universe view) | ConstellationMap.svelte | **No** |
| `map.rs` | 329–333 | outgoing-scan | Source of `outgoing_links` | N/A |
| `LinkDashboard.svelte` | 51 | "Broken" predicate | LinkDashboard "Broken" tab | **No** ⚠️ PRIMARY 4B-2 |
| `LinkDashboard.svelte` | 43 | Cross-Library target lookup | "Cross-Library" tab | No |
| `LinkDashboard.svelte` | 60 | Orphan computation | "Orphans" tab | No |
| `LinkDashboard.svelte` | 129 | Top-Connected counts | "Top Connected" tab | No |

### Documentation drift discovered during survey

`MIG-004-ALIAS-AWARE-RESOLUTION.md` line 130 claims `map.rs:81` was made alias-aware in MIG-004 §9. **The agent verified via code inspection: zero `note_aliases` references in `map.rs`.** The §9 fix was scoped, planned, and audit-deferred (4B-1) but never actually committed — the resolution summary lied. This needs a one-line correction in MIG-004's audit closure as part of MIG-005, since otherwise readers will assume map.rs is fine.

### Invariants (must hold throughout MIG-005)

1. **Sky View / Backlinks already-alias-aware behavior cannot regress.** Those reads come through `cache_boot_snapshot_sky` / `cache_boot_snapshot_graph` and are governed by cache.rs's existing 3-tier resolution. MIG-005 doesn't touch cache.rs.
2. **Outgoing-scan regex paths are invariant.** Extracting `[[wikilinks]]` from note bodies needs no alias resolution — that's parse-side. Aliases enter at *resolution* time (after parse, when matching target name to a real note).
3. **3-tier resolution semantics must match cache.rs's**: `name_to_path hit → alias_to_path hit → unresolved`. The first matching tier wins; aliases never override an exact name hit. (Avoids the "alias collides with a different note's name" ambiguity.)
4. **Per-command inbound count must equal `sky_nodes.link_count`** for any note. Two surfaces showing different counts for the same note is the visible bug we're fixing — MIG-005 must not introduce new variants of it.
5. **No write paths.** All MIG-005 changes are READ-side. Zero data-corruption risk class.
6. **No reactivity / lifecycle / IPC contract changes.** No CM6, no `$effect`, no `{#key}` interaction. (BUG-015's vector cannot be reintroduced through MIG-005's surface.)
7. **Performance**: each Rust command runs `SELECT alias_lower, path FROM note_aliases` once at the top. On a 7,600-note universe that's ~1 ms (small index, single column). Per-link resolution becomes one extra `HashMap::get`. Acceptable; no per-command profile regression expected.
8. **Tension's contradiction-detection** must continue to surface contradicting links. Today, a `[[OldTitle|contradicts]]` to a renamed note silently disappears from the report; post-MIG-005 it must appear with the canonical resolved target.
9. **Inspector360's named lists** must be complete: every note that links to the inspected note (by current name OR any historical alias) appears in the inbound list with the correct source label.
10. **LinkDashboard's "Broken" tab** must classify a wikilink as broken only if it resolves to neither a current note name nor any alias. Aliased-but-resolvable wikilinks must drop out of "Broken" and (where appropriate) appear in "Top Connected" / "Cross-Library" counts.

### Options

**Option A — Per-surface alias_to_path map (RECOMMENDED).**
Each Rust command, at the top of its body, runs:
```rust
let alias_to_path: HashMap<String, String> =
    db.prepare("SELECT alias_lower, path FROM note_aliases ORDER BY path")?
      .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
      .filter_map(Result::ok)
      .collect();
```
Then during link resolution uses the cache.rs 3-tier pattern. For `LinkDashboard.svelte`, `+layout.svelte` already loads `graph.aliases` from `cache_boot_snapshot_graph`; the change is to thread that data through as a prop and extend the 4 affected derived stores.

- **Pros**: Self-contained per command. Zero shared-state risk. Mirrors cache.rs §8 exactly. Smallest surgical change. No schema changes.
- **Cons**: Five small SELECTs instead of one shared cache. ~5 × 1 ms extra at command time. Negligible at user-visible scale.
- **Risk**: low. Each surface is read-side, additive (new lookup path beside existing one). Worst-case bug is "alias resolution returns wrong path" → wrong inbound-list entry. Same blast radius as MIG-004's cache.rs change.

**Option B — Shared `resolve_target_to_path(target, alias_map, name_map) -> Option<&str>` helper.**
Extract the resolution logic into a new module (e.g. `src-tauri/src/link_resolve.rs`) and call it from all 5 Rust surfaces. Update `cache.rs` to use it too.

- **Pros**: One place to maintain. Future MIG-006 §3 redo could reuse it. Less duplication.
- **Cons**: Touches `cache.rs` (which is currently closed and verified post-MIG-004 audit) — that's a Working Agreement #4 amber flag. Refactor cost. Marginal benefit since the resolution logic is ~6 lines.
- **Risk**: medium. Disturbing `cache.rs` to share code is the kind of "muddle with what's already working" the new top principal rule warns against.

**Option C — Write-time-derived `inbound_count` column on `note_meta` (or `sky_nodes`).**
Persist an alias-aware inbound count on the note row, maintained by the same triggers that maintain stratum/maturity in MIG-002. All five Rust surfaces just read the column.

- **Pros**: Zero per-command compute cost. Aligns with WTD principle (CLAUDE.md Rule 8). Solves the "two surfaces disagree" problem structurally.
- **Cons**: **Doesn't solve Inspector360 or Tension's needs.** Both surfaces require *the actual list* of source notes (Inspector360 displays them; Tension's contradiction detection iterates them). They need a HashMap anyway. So Option C only helps map.rs / strata.rs / maturity.rs counts — and even then, those surfaces also use the link list for other purposes (orphan detection, type counts). Schema change. Cross-cuts with MIG-002 §7-§10.
- **Risk**: medium-high. Schema change + trigger work + back-fill + interaction with MIG-002. Scope exceeds MIG-005's frame.

### Recommendation: **Option A**

- Smallest blast radius.
- Zero schema, zero IPC, zero reactivity.
- Each surface change is a discrete commit with a discrete verification clause.
- Pattern is already battle-tested in `cache.rs` §8.
- Doesn't preempt Option C — if profiling later shows the per-command SELECT is hot, MIG-005 commits don't need to be reverted; we just add the column on top.

### Out-of-scope for MIG-005

- Add a `'human_name'` source to `note_aliases` (that's MIG-003).
- Replace `cache.rs`'s in-line resolution with a shared helper (Option B refactor).
- Persist inbound counts (Option C).
- Fix the title-heading rename → cascade gap (separate item; no relation to alias resolution).

---

## Phase 2 — Plan (draft, awaiting user approval to commit)

| # | Step | /simplify? | Verify (user-testable) |
|---|------|-----------|--------|
| 1 | `map.rs::constellation_map_data` + `constellation_map_universe`: load `alias_to_path`, apply 3-tier resolution at `inbound_map` populator (lines 71–83 + 128–133) | — | Rename a note with N inbound. Map view bubble size for the renamed note remains unchanged (counts the inbound that still target the old title). |
| 2 | `strata.rs::compute_note_strata`: load `alias_to_path`, apply 3-tier at lines 73–81 | — | Same fixture rename. Stratum of the renamed note doesn't drop. |
| 3 | `maturity.rs::compute_note_maturity`: same treatment at lines 54–61 | ✔ /simplify checkpoint after §3 | Same fixture rename. Maturity tier (Seed/Sapling/Evergreen/Canonical) doesn't regress. |
| 4 | `tension.rs::detect_tensions`: load `alias_to_path`, apply at all 6 read paths (82–92, 113–125, 130, 164–171, 194–210, 246–252) | — | Create a contradicting wikilink to a target by its OLD title. Rename target. Confirm contradiction still appears in TensionPanel post-rename. |
| 5 | `inspector360.rs::get_360_view`: 3-tier at all 6 read paths (107–122, 125–142, 145–163, 334–349, 360–392, 411–444) | ✔ /simplify checkpoint after §5 | Open Inspector360 on a note that has 5 inbound links, where 2 target it via aliases. Confirm all 5 sources appear in the inbound list (typed + untyped) and `total_inbound = 5`. |
| 6 | `LinkDashboard.svelte`: thread `notePathToAliases` map (already loaded in `+layout.svelte`) as prop; build `allAliasesLower: Set<string>`; extend predicates at lines 51, 43, 60, 129 | — | Open LinkDashboard. Aliased wikilinks no longer appear in "Broken" tab. Aliased targets count toward "Top Connected" and don't mark canonical notes as orphans. |
| 7 | Documentation drift fix: correct `MIG-004-ALIAS-AWARE-RESOLUTION.md` line 130 / 289 to clarify §9 was DEFERRED to MIG-005 not shipped in MIG-004 | — | n/a |
| 8 | Phase 4 audit: 3 parallel agents (Invariants 1–10 hold / Drift / Migration path) | — | Audit report appended to this file. |

### Phase 4 audit triggers

After §1–§7 land. Audit agents check:
- Invariant verification (10 items above).
- Drift: any new alias-blind surface introduced elsewhere by recent commits.
- Migration path: first-boot, mid-rename, alias-table-empty, alias-collision-with-name (where alias of A equals name of B — should resolve to B's name not A's alias per Invariant 3).

### Rollback

Each step is a single-file commit (mostly). Revert any one of §1–§6 without affecting the others. If the whole approach turns out wrong, revert all six in reverse order; surfaces return to their pre-MIG-005 alias-blind state with no schema impact.

### Test plan (per Testing Instructions Rule)

For each step, the user-testable verification clause above is written in plain language ("Rename a note with N inbound. Map bubble unchanged"). I will define each feature first (what it does, why it matters, where in the UI to look), then walk through the click-by-click validation before asking the user to test.

---

## Notes for the next session reading this

- **Working Agreement #4 applied**: Phase 1 used 6 parallel Explore agents to map all 6 surfaces before any code design. The Architect doc cites file:line for every claim; no paraphrase.
- **Standing Order #5 applied**: a state-of-standing record was written before pivoting to MIG-005 (see `SESSION-LOG-2026-04-25.md §STATE-OF-STANDING`).
- **Reference pattern**: `cache.rs::read_sky_links_raw` lines 540–600 is the canonical 3-tier resolution. New code should match its shape.
- **`note_aliases` schema**: `(path TEXT, alias_lower TEXT, source TEXT, added_at TEXT, PRIMARY KEY(path, alias_lower))`. `source` ∈ `{frontmatter, rename, …}` per MIG-004 §3.
