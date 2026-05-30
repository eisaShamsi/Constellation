# MIG-066 — Living-Links Columns in the Base — Architect

**Phase 1 of the `/migration` workflow (Architect). Design map only — no code.**
**Date:** 2026-05-30 · **Predecessor:** MIG-065 (Unified Constellation Base, shipped & validated) · **Sequence:** 065 Simple → **066 Links** → 067 Epistemics.

> **Function in hand:** add **Living-Links columns** to the Base — let a note's table row show what the note is *connected to* (how many notes reference it, what typed relationships it participates in, how load-bearing its strongest link is), using the existing `note_links` index. This is the "Connection" question of the four-question Cognitive lens, surfaced as columns.

---

## 1. Why this, why now

MIG-065 shipped the Base as a table of a note's **own** properties (name, frontmatter, created). The Cognitive Engine's four questions are Development / Altitude / Origin / **Connection**. MIG-066 brings **Connection** into the table: a note is not an island, and the Base should be able to show its place in the web. The Living Link Architecture already stores everything needed — MIG-066 is about *surfacing* it as columns, not building new link machinery.

**Strong yet Simple, by default** still governs: link columns are opt-in via **+ Add column** (the "Constellation" group), never crowding the default view.

---

## 2. Territory map (verified against the code)

- **The link index — `note_links`** (`search.rs`): write-time, **trigger-maintained** (kept in lock-step with `sky_links` via `note_links_sky_ai`/etc.; `note_meta_au` triggers fire on edge changes). Columns: `source_path, source_name, target_name, link_type, annotation, confidence, weight, created, last_traversed, traversal_count, library_name, status, source_cid_cn, target_cid_cn`. So every Living-Link property (type, weight, confidence, traversal count, lifecycle status) is already indexed.
- **Existing precedent:** `search.rs:194–293` already runs `COUNT(*) FROM note_links WHERE …` per note (for knowledge-health / stratum scoring). Counting links per note in SQL is a solved, in-use pattern here.
- **How a Base column resolves — `lens::dimensions::resolve_dim`**: a column name either (a) looks up the **registered dimension table** (4 today: `note.name/path/created_at/headline`), each carrying a `sql_expression`, an optional `requires_join`, and `sortable/filterable/filter_ops`; or (b) is `prop.<key>` → `json_extract`. **Adding a Living-Links column = adding a registry entry whose `sql_expression` (and/or `requires_join`) reaches `note_links`.** The mechanism already supports joins (`note.headline` joins `note_summaries`).
- **The query — `execute_lens`**: builds one SQL `SELECT` over `note_meta` (federated across `main` + each cUniverse schema, then UNION), returns all matching rows; the frontend virtualizes. **Implication:** anything per-row-expensive runs ×(all matching rows), not ×(visible rows).
- **Rule 8 (write-time derivation):** every derived surface is maintained at write time, read cheaply. Link aggregates on the Base read-path must obey this — no live graph walk on open.

---

## 3. Candidate columns (the menu)

Grouped by cost-to-compute, all sourced from `note_links`:

| Column | Meaning | Source |
|---|---|---|
| **Outgoing links** | count of links where this note is the `source` | `note_links` same-library as the note |
| **Backlinks** | count of links where this note is the `target` | `note_links` **possibly other libraries** ⚠ |
| **Link types** | which typed relations it participates in (supports, contradicts, causes…) | `DISTINCT link_type` |
| **Strongest link** | max `weight` (load-bearing-ness) | `MAX(weight)` |
| **Most-traversed** | max `traversal_count` (the path the mind actually walks) | `MAX(traversal_count)` |
| **Confidence mix** | spread across hypothesis→evidence→established→contested | `confidence` |
| **Last traversed** | recency of use | `MAX(last_traversed)` |

v1 need not ship all of these — see §6 scope question.

---

## 4. The one real architectural tension: **outgoing is cheap, backlinks cross libraries**

A note's **outgoing** links live in the **same** library DB as the note → trivially materialized/queried in that schema.

A note's **backlinks** are recorded in the **source's** `note_links` — which may be a **different library / cUniverse**. SQLite triggers are per-database; there is **no cross-schema trigger** that could update a target note's materialized backlink count when a link is created in another library. This is the crux MIG-066 must answer, and it mirrors the wrinkle the MIG-065 Concept-Paper flagged for CNS metrics ("graph-global, not per-note-cheap").

This splits the menu into **cheap (outgoing-side, same-DB)** vs **federation-sensitive (incoming/backlink-side)**.

---

## 5. Design options (speed / effort / risk)

### Option A — Query-time correlated subqueries
Each Living-Links column is a registry entry whose `sql_expression` is a correlated subquery (`(SELECT COUNT(*) FROM note_links WHERE target_cid_cn = note_meta.cid_cn)` etc.).
- **Speed:** ✗ poor at scale. SQLite **evaluates a correlated subquery once per outer row** — over 7,651 rows × several link columns that is thousands of subquery executions per Base open. The research is explicit that this is the slow path and that a materialized count beats it. ([SQLite optimizer overview](https://sqlite.org/optoverview.html), [DuckDB on correlated subqueries](https://duckdb.org/2023/05/26/correlated-subqueries-in-sql))
- **Effort:** ◐ low (registry entries only, no schema change).
- **Risk:** ✗ violates Rule 8 (compute-on-read); regresses Base open-time on large universes — the hard constraint MIG-065 set.

### Option B — Materialize aggregates into `note_meta`, write-time via triggers
Add columns to `note_meta` (e.g. `outgoing_count`, `backlink_count`, `link_types`, `max_weight`), maintained by triggers on `note_links` (extending the existing `note_links_*` trigger family). Columns become trivial registry entries reading the materialized field.
- **Speed:** ✓ excellent — read is a plain column; matches the FTS/`notes_fts` precedent and the research recommendation (materialized degree + triggers > correlated subquery).
- **Effort:** ◐◐ moderate (schema add + triggers + a resumable back-fill for existing links).
- **Risk:** ⚠ the **backlink** field can't be maintained by a same-DB trigger when the source is another library (§4). Outgoing-side materializes cleanly; backlink-side does **not** in a federated universe.

### Option C — Hybrid (recommended)
**Materialize the same-DB / outgoing-side aggregates** (Option B, Rule-8-clean, cheap) for v1, and treat backlink/incoming aggregates as a **scoped second step** with an explicit federation design:
- v1: `outgoing_count`, `link_types` (outgoing), `max_weight`, `most_traversed` — all same-DB, write-time materialized, instant to read.
- v2 (backlinks): choose between (i) a **federated backlink-aggregate pass** that writes each target's count after a federated link resolve (like `resolve_libraries_recursive`), maintained on link change + on federation attach; or (ii) a single **global link-aggregate table** keyed by `cid_cn` that every library's trigger writes into; or (iii) query-time backlink count **only when the backlinks column is added** (bounded cost, opt-in).
- **Speed:** ✓ v1 instant; v2 bounded by design.
- **Effort:** ◐◐ v1 moderate; v2 scoped separately so v1 isn't blocked.
- **Risk:** ✓ lowest — ships the cheap, high-value, Rule-8-clean columns first; defers the genuinely-hard federated-backlink question to its own focused step instead of forcing it now.

---

## 6. Recommendation & open questions for the Plan phase

**Recommend Option C (hybrid).** Ship the same-DB outgoing-side Living-Links columns first (write-time materialized, Rule-8-clean, instant), and design backlinks as a deliberate second step with a real federation answer — rather than paying correlated-subquery cost or hand-waving the cross-library trigger.

This also matches the field: Dataview surfaces both `file.outlinks` and `file.inlinks` + `length()` as the canonical link-column vocabulary ([Dataview backlinks-as-column](https://forum.obsidian.md/t/dataview-table-backlinks-as-column/20576), [counting linked mentions](https://github.com/blacksmithgu/obsidian-dataview/discussions/1829)) — we ship the cheap half first, then the federation-correct backlink half.

**Decisions the Boss owns before Phase 2 (Plan):**
1. **v1 column set** — which of {outgoing count, link types, strongest link, most-traversed, last-traversed} ship first? (My lean: outgoing count + link types — the two most legible.)
2. **Backlinks in v1 or v2?** — accept the federation deferral (v2), or require backlinks now (then we pick C-iii query-time-when-added as the bridge)?
3. **How a multi-type cell renders** — a note that both `supports` and `contradicts` others: comma-list of types? count badge? (Form-Aligns-To-Purpose: the cell should answer "what kinds of connection," not decorate.)
4. **Rank-aware sort** — link types and confidence are ranked vocabularies; their sort order ties into the deferred rank-aware-sort work (MIG-068). Co-design or defer?

---

## 7. Invariants that must not break (the audit floor)

- **Rule 8** — no live graph walk on Base open; link aggregates maintained write-time.
- **Boot time / typing latency / IPC** — unchanged on a 7,600+ note universe (measure before/after; the MIG-065 hard constraint carries forward).
- **Federation correctness** — a federated Base must not show wrong/partial link counts; if v1 is outgoing-only, the columns must be *honestly* outgoing-only (no silently-partial backlink number).
- **Living Link semantics preserved** — weight (earned, decaying), confidence levels, lifecycle `status` (archival ≠ deletion). A column reads these; it never mutates them.
- **The 8 typed-link vocabulary** stays the cognitive vocabulary; a "link types" column reflects it, doesn't flatten it.
- **Existing Base behavior** — `prop.*` columns, registered dims, sort/edit/reorder/virtualization all keep working; the picker's "Constellation" group simply gains entries.
- **Reversibility** — schema additions are additive (`ALTER TABLE … ADD COLUMN`, idempotent), back-fill resumable in the background with status-bar progress (per Rule 8 first-time-population).

---

## 8. Decision point

This is the end of the Architect phase. **Next:** the Boss picks the v1 column set + the backlinks-now-or-v2 call (§6), then I write the **Phase 2 Plan** — phase-by-phase, each step one commit with a verification clause — for approval before any code.

**Sources (cross-check, WA#5):** [SQLite optimizer overview](https://sqlite.org/optoverview.html) · [DuckDB — correlated subqueries](https://duckdb.org/2023/05/26/correlated-subqueries-in-sql) · [Dataview — backlinks as column](https://forum.obsidian.md/t/dataview-table-backlinks-as-column/20576) · [Dataview — counting linked mentions](https://github.com/blacksmithgu/obsidian-dataview/discussions/1829)
