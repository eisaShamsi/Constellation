# PJ-065 — Architect (Phase 1 of /migration) — The Structural (Parent / TOC) Link Type

**Date:** 2026-06-27 · **Phase:** 1 (Architect) · **Status:** territory mapped; options on the table; **awaiting Boss rulings** (D1/D3/D5 + confirm D2/D9). Build deferred to Phase 2 (Plan).
**Concept (the horse):** the structural link is the *compositional spine* of a work — it answers *"what is the ordered shape of the work I am composing from these notes?"*, NOT the cognitive *"how do these ideas relate?"*. A distinct **kind** from the 8 cognitive acts: ordered, tree/DAG (acyclic), settled by authorship, exempt from the living-link apparatus (no weight/confidence/decay) and from cognitive topology. (Ratified concept paper: `docs/concept-papers/PJ-065-Parent-TOC-Link-Type-Concept-Paper.md`.)
**Method:** 5 parallel territory-mapping agents (note_links schema · link-type registry + consumers · MIG-086 fold + rename cascade · OrgChart/Backlinks read surfaces · closure/acyclicity + exclusion points) → 1 synthesis. Run `wf_dc93fc64-478`. All facts cited to file:line; the registry color/label facts re-verified by hand (one agent claim — that part-of is labeled "Compositional hierarchy" — was **wrong**; its real label is "Part Of").

---

## 1. The territory (what exists today)

- **`note_links` schema** (`search.rs:3237-3252`): `id` PK, `source_path`, `source_name`, `target_path`, `target_name`, `link_type`, `annotation`, `confidence`, `weight`, `created`, `last_traversed`, `traversal_count`, `library_name`, `status`; `UNIQUE(source_path, target_name, link_type)`. **No order/position/seq column** — this is the one primitive PJ-065 adds.
- **Idempotent column-add pattern proven** (`search.rs:2375-2383` `ensure_note_links_mig003_columns` + `column_exists()` probe): adding a nullable `seq` is backward-safe; existing rows stay `seq=NULL`, no backfill.
- **Write-time derived state via triggers**: `note_links_outgoing_ai/ad/au` (→ `note_meta.outgoing_count`/`outgoing_link_types`), the incoming-aggregate subquery (→ `incoming_count`/`incoming_link_types`, `search.rs:1179-1260`), and `note_links_sky_ai/ad/au` (→ `sky_links`, `search.rs:3480-3512`). **These count ALL edges** — they must learn to exclude structural edges or every cognitive signal inflates (the central risk, see §3).
- **Writer**: `index_note` (`search.rs:5126-5568`) parses frontmatter typed-links (MIG-086 fold, `extract_frontmatter_typed_links` `search.rs:4725-4762`) + body links, diff-computes the edge set, and INSERTs new edges with hardcoded `confidence='hypothesis'`, `weight=1.0`, `traversal_count=0` (`search.rs:5556-5565`) — **the structural write path must skip/override this**.
- **Link-Type Registry** (`linkTypeRegistry.ts` mirror; Rust `link_types.rs`): `LinkTypeDef { id, label, parent, color, order, builtin, emoji, desc }`. 8 seeds, colors verified: supports `#4A9EFF`, contradicts `#FF4A4A`, causes `#FF8C42`, exemplifies `#4AFF88`, generalizes `#A44AFF`, derives-from `#FFD700`, **part-of `#AAAAAA` (label "Part Of")**, supersedes `#5B7A8A`. Custom types supported. **No `structural` flag yet.**
- **Rename cascade** (`libraries.rs:1009/1013`): updates `note_links.source_path`/`target_path` on file rename; recognizes a frontmatter key only if it is a registered link type.
- **OrgChart** reads the **filesystem** (`read_library_tree`, `OrgChart.svelte:103 → libraries.rs:281`), single-parent by file path, no link/seq awareness.
- **Backlinks/Outgoing panels** already give `VirtualList` virtualization (Rule 3), `LinkTypePill` from the registry, per-row snippet — a TOC panel can copy this template.
- **No recursive-CTE precedent** in the codebase; `provenance.rs:185-210` does an in-Rust visited-set ancestor walk (the proven cycle-check pattern). **BUG-011**: chained AFTER triggers silently skip even with `PRAGMA recursive_triggers=ON` — so the acyclicity guard must be an explicit pre-write check, **not** a chained trigger.

---

## 2. The load-bearing decisions (option tables)

> `★` = Boss must rule before schema design. The rest are dictated by codebase rules (Rule 8, BUG-011, LL-XXX) and are engineering calls.

### ★ D1 — How is the structural kind made distinct from the 8 cognitive types?
| Option | Speed | Effort | Risk |
|---|---|---|---|
| **Add optional `structural: bool` to `LinkTypeDef`** (`#[serde(default)]`), register the structural type as a registry-owned entry → one clean predicate `def.structural` every consumer reads | medium | M | low/med — single source; risk is forgetting an exclusion guard (that's D8) |
| Repurpose `builtin=false` + a reserved id (no schema change) | fast | S | **high** — overloads `builtin`, forces id-matching across ~25 surfaces (the drift MIG-067 killed) |
| Keep structural edges entirely out of the registry | medium | M | med — strongest isolation but loses the registry's color/label/15-locale/pill for free |
**Recommendation:** Option 1. Reuses MIG-067 plumbing; one predicate replaces id-matching; `#[serde(default)]` keeps existing `link-types.json` valid. **Boss rules:** is it a *9th locked seed* (permanent grammar like part-of) or a *distinct non-cognitive lane* beside the 8 acts? And how does it relate to the existing cognitive **`part-of`** (which already means epistemic meronymy)?

### ★ D2 — Storage: extend `note_links` + nullable `seq`, or a dedicated `note_toc` table?
| Option | Speed | Effort | Risk |
|---|---|---|---|
| **Extend `note_links` + nullable `seq`** (proven idempotent ALTER) | fast | S | low/med — reuses indices, rename cascade, federation reads, the fold writer; **cost: trigger aggregates need a structural exclusion filter (= the D8 work anyway)** |
| Dedicated `note_toc(source_path, child_path, seq, …)` table | slow | L | med/high — doubles schema; new rename-cascade + federation wiring; loses the fold writer for free |
**Recommendation:** Option 1 — extend `note_links`. Backward-safe, no backfill, inherits the whole write/rename/federation machinery. *(Technical call; Boss confirms, doesn't design.)*

### ★ D3 — Name(s), declaration direction, implied reverse, color
| Option | Note | Risk |
|---|---|---|
| Assert **`parent`** on the child (`parent: [[Chapter]]`), materialize reverse **`contains`** | aligns with the MIG-086 fold (property-key → link_type, zero new parser code) | low — but "parent" may blur with the filesystem folder parent in the user's mind |
| Assert **`contains`** on the parent (`contains: [[Sec1]], [[Sec2]]`), materialize reverse `parent` | **list order = `seq` naturally** (interacts with D4) | low/med — a long TOC bloats one note's frontmatter; single-child edits touch the parent |
| Domain names (`chapter-of`/`scene-of`/`toc`) | — | med — too book-bound for a general spine |
**Color:** must be distinct from all 8; free families include teal (`#14B8A6` / `#66B2B2`) and indigo (`#6366F1` / `#4F46E5`). **Boss rules the vocabulary + color** (pure design — this is *your* cognitive grammar).

### ★ D5 — Cardinality: single-parent tree, or multi-parent DAG with a primary spine?
| Option | Speed | Effort | Risk |
|---|---|---|---|
| **Single-parent tree** (≤1 structural parent per note) | fast | S | low — mirrors filesystem ownership, unambiguous breadcrumb, simplest acyclicity; needs an explicit "at most one structural parent" write guard (the UNIQUE constraint does NOT give this) |
| **Multi-parent DAG + primary-spine flag** | slow | L | med/high — needs a primary-spine marker, breadcrumb disambiguation, full closure cycle-check; more UI |
**Recommendation:** single-parent for MVP (does not preclude a later DAG upgrade — only the write guard relaxes). **Note for Boss:** the *concept ratification* leaned "multi-parent with one primary spine." The Architect evidence says single-parent is materially simpler and unambiguous for the MVP. **This is the one place the Architect recommendation differs from the ratified direction — your call.**

### D4 — Ordering mechanism *(codebase-forced)*
Integer `seq` (1,2,3…), resequence-on-insert. Fractional rank is a premature optimization for a bulk-reorder gesture not in the MVP; sibling fan-out is small. **→ integer `seq`.**

### D6 — Acyclicity *(codebase-forced)*
Write-time, structural-edges-only; **Rust ancestor walk (the `provenance.rs` pattern) as an explicit pre-write guard**, NOT a chained AFTER trigger (BUG-011). Measure cost on the 7,600-note universe before commit.

### D7 — Direct vs closure *(codebase-forced by Rule 8 / LL-XXX)*
Store **direct edges only**; compute ancestors (breadcrumb) + descendants (outline) **on read**, lazily, on user gesture, via a separate isolated read API (`get_structural_ancestors`/`get_structural_descendants`). **A stored closure is forbidden** (the 3 GB WAL OOM). If a read is slow, cache in-component, never in DB.

### D8 — Exclusion from the cognitive apparatus *(codebase-forced; the build's hardest work)*
**Both layers:** (a) put the structural exclusion in the write-time aggregate triggers + `recompute_all_incoming` + `MATURITY_SQL_EXPR` so every count-reading surface is clean at the source; (b) add an explicit guard at each raw-edge-walking consumer (`graph.rs` `link_type_weight`, `sight.rs` centrality, `tension.rs`). This is the **LL-023 regression surface** — see §3. The Audit phase MUST checklist every site.

### ★ D9 — MVP UI surface
| Option | Speed | Effort | Risk |
|---|---|---|---|
| **New dedicated TOC/Outline panel** reusing Backlinks/Outgoing infra + breadcrumb trail | fast | M | low — proven VirtualList/pill/snippet; only new code is the read API + breadcrumb; decoupled from OrgChart |
| Extend OrgChart with a "links-derived" mode | medium | M | med — procedural state branching; conflates the filesystem spine with the structural spine |
**Recommendation:** Option 1 — new TOC panel + breadcrumb (matches the ratified MVP). OrgChart links-mode = Phase-2 follow-up. **Boss confirms MVP scope.**

---

## 3. Invariants that must not break

1. **LL-023 cognitive-exclusion surface (the central risk).** Structural edges must be invisible to every cognitive subsystem. Concretely they must NOT enter: `note_meta.incoming_count`/`outgoing_count` aggregates (triggers `search.rs:1179-1260`); `MATURITY_SQL_EXPR` inbound count (`search.rs:252-305`); maturity `compute_state` (`maturity.rs:34-122`); strata inbound (`strata.rs:68-150`); inspector360 connection-counts + SPOF + `missing_link_types` (`inspector360.rs:21-227`, `Inspector360.svelte:19,62-76`); sight Brandes centrality (`sight.rs:68-182`); tension orphan/contradiction/SPOF (`tension.rs:163-276`); cece `link_type_weight` (`graph.rs:188`); `recompute_all_incoming` (`links_backfill.rs:254`). **Missing any one site silently corrupts a signal.**
2. **Living-link apparatus exclusion (D8):** no weight/confidence/decay/traversal on structural edges; the fold's hardcoded `confidence/weight/traversal` (`search.rs:5556-5565`) must be skipped. Frontend enumerators (`LinkTypePicker`, CNS legend `ConstellationSight2`, Base `note.link.*` columns `tableModel.ts`, `GraphMindView`, `CCSView`, `KnowledgeHealthDashboard`, `completions.ts`) must filter or distinctly render the structural type.
3. **Filesystem single-parent ownership is immutable.** Structural links add a *second, independent* spine; they never change which folder/library owns a `.md` file.
4. **Search grammar** (`parseSearchQuery` typed-link regex, `store.ts:2299-2308`) must not treat the structural type as a cognitive typed-link (guard by the `structural` predicate, not list membership).
5. **Closure ON READ only** (Rule 8 / LL-XXX) — no stored closure, no trigger-maintained child/ancestor column.
6. **`UNIQUE(source_path, target_name, link_type)` is preserved** — but it does NOT enforce single-parent; D5 single-parent needs a separate write guard.
7. **Editor-Surface Gate (BUG-015):** structural derivation stays on the post-save `index_note` path; never writes note body; TOC panel reads on gesture only.
8. **Rename cascade (BUG-023 shape):** the structural frontmatter key must be a registered type; a linked-probe pair test (A structurally-parents B; rename B; both identities + the edge intact) must pass.
9. **Acyclicity at write-time only** — explicit pre-write guard, not a chained trigger (BUG-011).
10. **Boot/typing/IPC non-regression on 7,600 notes** — nullable `seq` is a no-op on existing rows; no boot scan, no closure precompute; structural reads are lazy + isolated; a pre-PJ-065 universe with zero structural edges behaves identically.

---

## 4. Migration / back-fill / rollback concerns

- **First boot (existing universe):** add `seq` idempotently via a new `ensure_note_links_mig065_columns()` (`column_exists()` probe). **No data backfill** — cognitive edges legitimately have `seq=NULL`. Register the structural type; existing `link-types.json` deserializes unchanged (`#[serde(default)]`). No universe walk, no closure precompute.
- **Schema-version mismatch:** stamp `schema_versions` so the column-add + registry reconciliation run once, idempotent on re-boot. Verify the registry merge does not reparent/reorder the 8 immutable seeds.
- **Mid-backfill interrupt:** there is NO mandatory backfill. The only backfill-shaped work is the D8 aggregate-exclusion `recompute_all_incoming` — at migration time there are zero structural edges, so it is a no-op on first deploy; write-time triggers keep counts current as structural edges are later authored. Uses the resumable cursor + BATCH_SIZE 500 + `schema_versions` stamp (interrupt-safe).
- **Rollback:** additive + reversible. (a) the `seq` column can stay ignored; (b) removing the registry entry makes structural rows inert; (c) **the aggregate-trigger exclusion filter and the registry entry are ONE atomic unit** — reverting one without the other drifts counts. Audit must run a linked-probe test + a full Editor-Surface Gate pass (all 8 checks, Focus included) before declaring no-drift.
- **Federation:** extending `note_links` inherits the existing per-universe federation read path; a dedicated table would need new wiring. A structural parent in a *different* library than its child (filesystem forbids it, structural links could express it) is an open D5 sub-question — until ruled, keep MVP structural edges within one library.

---

## 5. The minimal set the Boss rules before Phase 2 (Plan)

1. **D1** — distinct non-cognitive lane vs 9th locked seed; relationship to the existing cognitive `part-of`.
2. **D3** — the name(s) + declaration direction + reverse + a distinct color. *(Don't schema-design before the name is ruled.)*
3. **D5** — single-parent tree (Architect rec) vs multi-parent + primary spine (ratified direction).
4. **D2** — confirm extend-`note_links`-+-`seq` (recommended) — technical, Boss confirms.
5. **D9** — confirm MVP = new TOC/Outline panel + breadcrumb (recommended; matches ratification).

D4/D6/D7/D8 are dictated by codebase rules and are engineering calls (documented above), not Boss decisions.

*End of PJ-065 Architect (Phase 1).*
