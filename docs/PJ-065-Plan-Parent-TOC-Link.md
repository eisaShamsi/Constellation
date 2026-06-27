# PJ-065 — Plan (Phase 2 of /migration) — The Structural (Parent / TOC) Link Type

**Date:** 2026-06-27 · **Phase:** 2 (Plan) · **Status:** drafted + adversarially stress-tested (2 critics, both "needs-changes" → all blockers/majors folded in) · **awaiting Boss approval.** Architect: `docs/PJ-065-Architect-Parent-TOC-Link.md`. Run: draft `wf_c1920818-855`.
**Locked rulings:** D1 distinct non-cognitive lane · D2 extend `note_links`+nullable `seq` · D3 support both `parent:`/`contains:` (quoted `[[ ]]`), reverse auto-derived, teal `#14B8A6` · D4 integer `seq` · D5 single-parent tree · D6 write-time Rust cycle-guard · D7 closure on read · D8 dual-layer exclusion · D9 new TOC panel + breadcrumb.

---

## A. Storage model (the per-note-diff knot, resolved) — **Model A: two stored faces, union on read**

`index_note(N)` computes the edge set **for `source=N`** and reconciles only rows `WHERE source_path=N` (`search.rs:5511-5568`). Any edge written with `source ≠ N` (a physical reverse owned by the target) is **clobbered** when that target is later re-indexed. So **no cross-note edge ownership** — Model B (write-time reverse materialization) is unsafe here. Therefore:

- A child with `parent: "[[P]]"` → one row `source=child, target=P, link_type='parent', seq=NULL`.
- A parent with `contains: ["[[C1]]","[[C2]]"]` → rows `source=parent, target=Ci, link_type='contains', seq=i` (declaration order).
- Each note owns only its own-source rows → survives its own diff; **no clobber**.
- **`seq` lives on the `contains` (parent→child) face** (the only face that expresses an ordered child *list*). The `parent` face has `seq=NULL` (a child has one parent — order is meaningless there).
- **Union on read** (the "reverse" is derived, not stored): children-of-P = (`contains` rows `source=P`) ∪ (`parent` rows `target=P`); parent-of-C = (`parent` row `source=C`) ∪ (`contains` rows `target=C`). Dedupe by resolved `(parent_path, child_path)`, preferring the seq-bearing `contains` row.
- **No LINK file.** Structural edges are frontmatter-derived `note_links` rows only — they are *not* first-class link objects (consistent with how MIG-086 frontmatter typed-links already work; the LINK-file model was never built). Intentional, stated here for the record.
- **Frontmatter-only authoring.** Structural edges come **only** from the frontmatter face. A body `[[parent::X]]` is **not** a structural edge (down-classified to `associative`) — so `seq`/guards (which live on the frontmatter path) are never bypassed.

### Deterministic single-parent rule (D5) — index-order-independent
A note has **at most one** structural parent across both faces. Conflicts are resolved by a **fixed precedence, not by which note was indexed last**:
1. The child's own **`parent:`** declaration is **authoritative** (the child names its one parent).
2. If the child has no `parent:` but ≥1 `contains:` claims target it, the parent with the **lexicographically smallest path** wins (deterministic).
3. A losing/conflicting claim is **never written silently and never modifies disk** (Editor-Surface Gate). The conflict is **surfaced at read time** (breadcrumb/panel shows a "multiple declared parents" indicator) so the user can fix the frontmatter themselves.
*Verification authors the conflict in **both** index orders and asserts the **same** surviving parent.*

### Acyclicity (D6) — over the **unioned** graph, pre-commit
A Rust ancestor walk (visited-set on resolved **paths**, the `provenance.rs:185-210` pattern) that (a) resolves each hop via the **same union** as the read API (both faces), (b) **injects N's pending edges** (the ones about to be written, not yet committed) so the closing edge is visible, (c) runs after old-edge DELETE is computed and **before COMMIT**. Rejects any edge making N its own ancestor. *Tested with a 2-node `parent`-face cycle, a `contains↔contains` cycle, AND a 3-node mixed-face cycle.*

---

## B. The safe commit order (each commit independently green + behavior-preserving)

> **Principle:** every exclusion filter references `structural_sql_in_list()` / `cognitiveLinkTypes()`, which are **empty until §5 registers the types**. So §1–§4 install the schema + flag + every filter as **provable no-ops** (no structural type exists → zero behavior change). §5 registers the types + emits edges — by which point all filters are already live. **No drift window.** (`NOT IN <empty>` is made safe by emitting `1=1`, never the SQL-error `NOT IN ()`.)

### §1 — Schema: nullable `seq` column
- `search.rs`: `ensure_note_links_pj065_columns(conn)` mirroring `ensure_note_links_mig003_columns` (`2375-2383`): `if !column_exists("note_links","seq")` → `ALTER TABLE note_links ADD COLUMN seq INTEGER` (nullable, no default, **no backfill**). Call it **unconditionally** in `init_db` right after the mig003 call (stamp `schema_versions` `module='pj065_structural'` is **observability only**, never a guard that can skip the ALTER).
- **No new index** for MVP (per-parent child counts are tiny; `idx_link_source` covers the seek, sort in Rust). Adding `idx_link_seq` later is pure-additive.
- **Verify:** `cargo test`; fresh DB + existing DB both show nullable `seq`; `init_db` twice = no error. *(Not Boss-testable.)*

### §2 — Registry plumbing (the `structural` flag + helpers) — **no types registered yet**
- `link_types.rs`: add `#[serde(default)] pub structural: bool` to `LinkTypeDef`; all constructors set `false`. New **separate** constant `STRUCTURAL_SEED_IDS` (do **NOT** append to `SEED_IDS` — that is the 8-only lock read by many iterators). `merge()` (`90-122`) locks **both** sets `builtin=true`, but structural ids get `structural=true` + own group/parent rules. Helpers: `cognitive_ids()`, `structural_ids()`, `is_structural_type(id)`, `sql_in_list_cognitive()`, `sql_rank_case_cognitive()`, `structural_sql_in_list()` — **all empty-set-safe** (emit `1=1`, never `NOT IN ()`). Keep `sql_in_list()`/`sql_rank_case()` for the few callers that legitimately want all types.
- `linkTypeRegistry.ts`: add `structural: boolean` to the interface; `isStructuralLinkType()`, `cognitiveLinkTypes()`, `structuralLinkTypes()`.
- **Verify:** full `cargo test` + `svelte-check` green; helper unit tests incl. **empty-set SQL shape**; a user delta `{id:'parent',builtin:false,structural:false}` is coerced back to locked `builtin/structural:true` (test); **behavior identical** (no structural types exist). Audit + list every `SEED_IDS` reader to confirm none broke. *(Not Boss-testable — internal.)*

### §3 — Install Rust cognitive exclusions (no-op while empty)
- **Aggregate triggers:** `outgoing_aggregate_assignments` (`1135-1157`) raw `COUNT(*)` (`1141`) → add `AND link_type NOT IN <structural>`; breakdown cols → cognitive-only helpers. `incoming_aggregate_assignments` (`1231-1264`) `matched` subquery both UNION branches → `AND nl.link_type NOT IN <structural>`; breakdown → cognitive-only. **The edit lives in the shared assignment helpers** so `links_backfill`/`incoming_links_backfill` recompute paths inherit it.
- **`MATURITY_SQL_EXPR`** (`252-305`): all four inbound `COUNT(DISTINCT source_path)` → `AND link_type NOT IN <structural>`.
- **Sky:** `note_links_sky_ai/au` WHEN guards (`3480-3512`) **and** `sky_backfill.rs` populator → exclude structural (so neither live triggers nor the one-off backfill put structural in `sky_links`).
- **Read path (the missing gap):** `backlink_rows_in_schema` (`cache.rs:464-491`) + `outgoing_rows_in_schema` (`496-509`) → `AND link_type NOT IN <structural>`; **update the `getBacklinks==incoming_count` parity test (`incoming_links_backfill.rs:232`) so BOTH sides exclude structural** (keeps the drift canary green).
- **Boot bundle:** `cache_full_links`/`BootLinks` (`cache.rs:358`) → **exclude structural** (the TOC panel uses the dedicated lazy APIs, not the boot bundle → zero boot-bundle size delta, and frontend cognitive consumers of BootLinks never even see structural).
- **Raw-edge consumers:** `sight.rs` centrality SQL (`76`); `tension.rs` `load_notes_from_db` (`204-207`); `graph.rs` `link_type_weight` (`188`) → lock `parent`/`contains` at `0.0` by test + comment.
- **`strata.rs` + `inspector360.rs` (DRY):** these whole-content regex scanners lack frontmatter-key awareness. Route their frontmatter region through the **shared** block-aware `extract_frontmatter_typed_links` (or one shared `is_wikilink_under_structural_key` helper) and drop structural before it enters `outgoing`/`used_types` — **no divergent re-implementation.**
- **Verify:** all filters no-op (no structural ids) → **full suite green, every count identical to pre-§3**. Add the **through-real-`init_db` trigger test scaffold** (triggers built from the registry snapshot exactly as boot does) — asserted live in §5. *(Not Boss-testable — internal, behavior-preserving.)*

### §4 — Install frontend cognitive exclusions + search-grammar pin (no-op while empty)
- `LinkTypePicker.svelte` (`34-38`) → `cognitiveLinkTypes()`. `ConstellationSight2.svelte` CNS legend (`196-300`) → `cognitiveLinkTypes()`. `tableModel.ts` Base column labels (`29-39,114-117`) **and** `BaseColumnPicker.svelte` (`68-69`, the surface that *offers* `note.link.<id>` columns) → exclude structural. `Inspector360.svelte` `missing_link_types` (`62-76`) → compare against `cognitiveLinkTypes()`. `GraphMindView`/`CCSView`/`KnowledgeHealthDashboard` → filter via `isStructuralLinkType`. `store.ts:4259` `linkTypeNames()` → cognitive-only variant for Settings. **`LinkTypesEditor.svelte` (§G):** add structural ids to the lock set **and hide** them (they are registry-owned, not user-recolourable/deletable). `completions.ts` `[[` autocomplete → **exclude** structural (frontmatter-only authoring). **Search grammar** `parseSearchQuery` (`store.ts:2299-2308`) → pin structural OUT (comment + test) so `parent [[X]]` is plain text, not a typed operator.
- **Verify:** `svelte-check` green; with no structural types, every surface unchanged. *(Not Boss-testable — internal.)*

### §5 — **CORE:** register types + emit edges + guards *(data-affecting; all exclusions already live)*
- `link_types.rs`: register `parent` (label "Parent", child→parent face) + `contains` (label "Contains", parent→child face), both `color #14B8A6`, `structural:true`, locked via `STRUCTURAL_SEED_IDS`, ordered into their **own group after the 8**. i18n×15 pill labels (`linkTypes` block, RTL-correct ar/fa/he/ur).
- `search.rs` `index_note`: partition merged `typed_links` (`5229-5240`) into cognitive vs structural via `is_structural_type`; **structural only from the frontmatter face** (drop body `[[parent::X]]`). Carry **declaration-order `seq`** on the `TypedLink` (set in `emit_frontmatter_links` while iterating the ordered list — the **only** order-preserving point; not derivable from the `new_edges` HashMap). Thread `seq` through `new_edges`' value + **both INSERT branches** (`5556-5565`) + **the "unchanged"/"identical" comparisons** so a **reorder forces re-INSERT**. Branch on `is_structural_type` at INSERT → write **neutral confidence/weight** (skip the `'hypothesis'/1.0/0` living-link defaults). Run the **deterministic single-parent guard** (§A) + the **union acyclicity guard** (§A) in the pre-commit window.
- **Verify (through real `init_db`):** `parent:`/`contains:` produce correct source-owned rows with `seq=1,2`; **reordering `contains:` updates stored `seq`**; conflict resolves to the **same** parent in both index orders; 2-node, `contains`-face, and 3-node mixed-face cycles all rejected; re-indexing the target does **not** clobber the other face; a structural edge adds **zero** to outgoing/incoming counts, maturity, Sky, centrality, tension (exclusions live); structural rows carry no living-link defaults. **Boss GATE A.**

### §6 — Lazy read APIs (closure on read)
- `structural.rs` (new): `get_structural_children(parent)` (union both faces, dedupe by resolved pair, order by `seq` then stable fallback); `get_structural_ancestors(note)` (breadcrumb — walk the single-parent chain via the union, visited-set on **paths**, name→path resolution per the writer's `LOWER(name)` lookup `5452`, bounded depth); `get_structural_descendants(note)` (outline — recursive children walk, visited guard, bounded). Register in `lib.rs` invoke_handler.
- **Verify:** `cargo test` — union+dedupe across faces; ancestors = single-parent chain; descendants = subtree; cycle cannot infinite-loop; name-collision safe (paths, not names). *(Boss-tested via §7.)*

### §7 — TOC/Outline panel + breadcrumb
- `StructuralOutlinePanel.svelte` (new), mirroring `BacklinksPanel.svelte`: `VirtualList` (≥50 rows), `LinkTypePill` teal (localized via §5), per-row snippet; `get_structural_descendants` outline (indented tree) + `get_structural_ancestors` breadcrumb (pattern from `ConstellationMap.svelte:68,232-238`). **Reads on user gesture only** (Editor-Surface Gate — never on note open, never writes body). Wire into the panel host beside Backlinks/Outgoing. OrgChart links-mode **deferred** (D9). i18n×15 chrome (title, empty-state, "no parent"/"no children"), RTL-correct.
- **Verify:** `svelte-check` (compile). Runtime invariants (gesture-only, **zero body write on open**) via the running app + the write-journal harness. **Boss GATE B.**

### §8 — Rename linked-probe (both faces) + cold-start + docs + i18n audit
- `libraries.rs`: confirm the path cascade (`1009-1015`, link_type-agnostic) + the frontmatter wikilink rewrite (`5068`, regex, link_type-agnostic) cover structural. **BUG-023 linked-probe test, both faces:** (1) A has `parent: "[[B]]"` → rename B→B2 → A's frontmatter rewritten + edge `target_name` updated + `get_structural_ancestors(A)` resolves to B2; (2) C listed in P's `contains: ["[[Old]]",…]` (array shape) → rename → array entry rewritten + outline resolves with correct `seq`.
- **Cold-start:** pre-existing `parent:`/`contains:` frontmatter (File-Over-App; authored in Obsidian) is populated by the **normal per-note guarded re-index** (resumable via the existing reconcile path; guards are cycle/conflict-safe and never modify disk). No separate backfill needed; documented.
- **Docs:** link-type/living-link design doc + orientation v-bump (SO #6, same commit as the feature close). 15-locale audit (all new strings present + RTL).
- **Verify:** rename both faces (above); i18n lint 0 missing; full `cargo test` + `svelte-check` green; manual rename end-to-end. **Boss GATE C.**

---

## C. Consolidated exclusion checklist (the LL-023 surface — Audit Phase 4 walks every row)
**Write-time aggregates (§3):** outgoing `COUNT(*)` `1141` · outgoing breakdown `1142-1151` · incoming `matched` subquery `1244-1252` · incoming breakdown `1257-1261` · `MATURITY_SQL_EXPR` ×4 `255-293` · sky triggers `3480-3512` · `sky_backfill.rs` · `recompute_*` (via shared helpers) · parity test `incoming_links_backfill.rs:232`.
**Read path (§3):** `backlink_rows_in_schema` `cache.rs:464-491` · `outgoing_rows_in_schema` `496-509` · `cache_full_links`/BootLinks `358`.
**Raw-edge consumers (§3):** `sight.rs:76` · `tension.rs:204-207` · `graph.rs:188` · `strata.rs:153-218` (shared FM parser) · `inspector360.rs` scan (shared FM parser).
**Frontend (§4):** `LinkTypePicker:34-38` · CNS legend `ConstellationSight2:196-300` · `tableModel.ts:29-39` · `BaseColumnPicker:68-69` · `Inspector360.svelte:62-76` · `GraphMindView` · `CCSView` · `KnowledgeHealthDashboard` · `linkTypeNames store.ts:4259` · `LinkTypesEditor.svelte` (lock+hide) · `completions.ts` · search grammar `store.ts:2299-2308`.

## D. Rollback
**Atomic unit = §5 (registration+emission) + §3/§4 (exclusions).** §1–§4 are no-ops while no structural type is registered, so they are safe to land/keep alone; §5 is the only data-affecting commit and it lands **after** all filters exist. If drift is found in production: revert §7/§8 (read-only) → revert §5 (registration; structural rows become inert unknown-type rows) → §3/§4 filters can stay (no-ops again) → run `reconcile_filesystem`/`on_link_vocabulary_changed` to rebuild `note_meta` aggregates from surviving cognitive edges. The `seq` column (§1) stays (nullable, inert). **Drift canary:** the `getBacklinks==incoming_count` parity assertion.

## E. Staged Boss test gates (sent one at a time per the staged-tests rule)
- **GATE A (after §5):** the new teal Parent/Contains pills render with the correct localized label in their own group beside the 8 (and in an RTL locale); authoring `parent:`/`contains:` builds the right tree with correct order; **adding a structural link does NOT move the note's maturity badge, connection counts, or Sky presence.**
- **GATE B (after §7):** the TOC/Outline panel shows the descendant outline + ancestor breadcrumb in teal, opens only on click (no spurious save), correct in RTL.
- **GATE C (after §8):** rename a parent note → the child's frontmatter updates and the breadcrumb still resolves; full end-to-end.

## F. Decisions I made inside the plan (flag if you disagree — otherwise I proceed on approval)
1. **Frontmatter-only authoring** — a body `[[parent::X]]` is not a structural edge (keeps guards/seq from being bypassed).
2. **Single-parent conflict** — deterministic precedence (child's `parent:` wins; ties → smallest path); conflicts are **surfaced read-side, never silently dropped, never modify your files**.
3. **No LINK file** for structural edges (matches the existing frontmatter typed-link model).
4. **Boot bundle excludes structural** (zero boot-perf delta; panel uses lazy APIs).
5. **No bare-name autowrap** in the MVP (you didn't request it; `[[ ]]` quoted is the convention) — easy to add later.

*End of PJ-065 Plan (Phase 2). On approval I cascade §1→§8, pausing only at GATE A/B/C for your test.*
