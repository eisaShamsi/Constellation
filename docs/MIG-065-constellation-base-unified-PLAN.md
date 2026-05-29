---
title: MIG-065 — The Unified Progressive Base (Dual-World) — Plan
version: 1.0
date: 2026-05-29
status: Plan. Awaiting Eisa's single go-ahead (the one Migration gate before code).
architect: docs/MIG-065-constellation-base-unified-ARCHITECT.md
governing_principle: "Strong yet Simple, by default."
---

# MIG-065 — Plan: The Simple Foundation

**Scope of this MIG:** the *Simple* half of the Dual-World Base — a familiar, editable table on the clean SQL engine. Living Links (MIG-066) and Epistemics (MIG-067) follow immediately after. Each § below lands as one commit with its own verification clause; Boss-testable steps pause for a tutorial-style test (Testing Instructions Rule).

---

## §A — Concept Paper v2.0 (doc-only, no code)
Write `docs/Constellation-Base-Concept-Paper-v2.0.md` per Architect §9: add the **Strong-yet-Simple-by-default** principle (§5.0); reframe v1.4 §3's refusal (we refuse empty aspirational structure, not familiar surfaces); fold in the unified design + MIG-065+ roadmap. Preserve v1.4 (new file, never overwrite).
**Verify:** v2.0 exists; v1.4 untouched; the reconciliation principle is stated. (No build; the guiding light updates before code, per decision #6.)

## §B — `properties_json` reliability audit + fix  *(BLOCKER — Architect R1)*
Audit the `note_meta.properties_json` write path against on-disk frontmatter on the Eisa Universe (row-by-row spot check per `feedback_walk_through_writes`). If the flagged `search.rs::parse_frontmatter` bug corrupts/omits keys, fix it so `properties_json` faithfully mirrors each note's YAML frontmatter.
**Verify:** for ≥20 sampled notes (incl. RTL keys, nested/empty values), `json_extract(properties_json,...)` equals the on-disk frontmatter. Backend test added. *Until this passes, §C–§H do not start — the familiar table is only as trustworthy as this column.*

## §C — Schema extension (additive)
Extend `lens/definition.rs`: `LensView` gains `Table`; `LensColumn` accepts **either** `property: <key>` (frontmatter) **or** `dimension: <registered>`; `filters`/`order` likewise. Update `parser.rs` + `validator.rs` (a `property:` column is always valid; a `dimension:` column validates against the registry). Keep `display:` (displayName/width/dir) optional.
**Verify:** parser/validator unit tests — a YAML mixing `property:` and `dimension:` columns parses; an unknown `dimension:` errors; an unknown `property:` does not. All existing lens tests still green.

## §D — SQL builder: dynamic property columns + table view
Extend `sql_builder.rs`: emit `json_extract(properties_json,'$.<key>') AS <alias>` for `property:` columns; table view selects every declared column; `property:` filters/sorts compile to `json_extract(...)` comparisons. Mirror it in `build_federated_sql` (symmetric across the `UNION ALL`).
**Verify:** in-memory SQLite tests — a table-view lens with 2 frontmatter columns + 1 registered dimension returns correct rows single-schema AND federated; filter/sort on a `property:` column works. Federation parity confirmed.

## §E — Frontmatter-key discovery command
Add a cheap Tauri command (e.g. `discover_base_properties(scope)`) enumerating distinct frontmatter keys across the scope via `json_each(properties_json)` (federated-aware). Feeds the picker's "Your fields" tier.
**Verify:** on the Eisa Universe, returns the real frontmatter keys; runs in well under the §10.3 budget; federated keys included.

## §F — Standalone `.base` → table tab + renderer re-point  *(Boss-testable)*
Route a `.base` file open to a full-tab **table** mounting the reused `BaseTableView.svelte` family, fed by `execute_lens` rows (not `query_base`). Inline ` ```base view: table ` blocks render the same table in `LensBlockWidget`.
**Verify (Boss tutorial):** open/create a `.base` → see a familiar table of your notes' fields; columns resize/reorder; RTL correct. **Stop for Boss test.**

## §G — The tiered add-column picker + filter/sort builders
`+ Add column` opens the tiered menu: **Your fields** (from §E) + file intrinsics; **Constellation** power section present but shown as the (initially small) registered set, clearly marked read-only — the Strong-yet-Simple surface. Adapt `BaseFilterBuilder`/`BaseSortBuilder` to the new column model.
**Verify (Boss tutorial):** add/remove a frontmatter column; add a filter and a sort; the power section is visible-but-secondary. **Stop for Boss test.**

## §H — Edit-in-place (frontmatter only)  *(Boss-testable)*
Cell edit on a `property:` column writes back via `update_note_property` (debounced single-writer, reload-after-write — Architect R5). `dimension:` columns are non-editable and visually marked.
**Verify (Boss tutorial):** edit a cell → the note's frontmatter on disk updates; a cognitive column can't be edited. **Stop for Boss test.**

## §I — Retire the old live-scan engine
Re-point `.base` routing + the MIG-062 `list_workspace_bases` federation to the new engine; remove/deprecate `query_base`'s filesystem walk and the dead legacy path. Keep reused UI components.
**Verify:** no Base read performs a filesystem walk (grep + a perf check on 7,600 notes — Rule 8 holds); `list_workspace_bases` still lists, now via the unified path; svelte-check + lib tests green.

## §J — Tests + 3-agent audit (Migration Phase 4)
Full test pass (parser/validator/sql/dynamic-columns/edit/federation). Then 3 parallel audit agents: **invariants** (§5), **drift** (new guards), **migration path** (first-boot, old JSON `.base` present, mid-edit interrupt, federated + single-universe).
**Verify:** all tests green; audit findings triaged (blockers fixed, non-blockers logged as PJ-NNN).

## §K — Boss-test gate (staged tutorial)
Staged per `feedback_staged_tests`: Stage 1 = open a `.base`, see the familiar editable table; Stage 2 = add/filter/sort columns; Stage 3 = edit-in-place writes frontmatter; Stage 4 = federated universe shows cUniverse rows. Each stage defined feature-first, then click-by-click (Testing Instructions Rule). Send Stage 1 first.
**Verify:** Eisa confirms each stage.

## §L — PCS
Orientation vX.Y (Dual-World Base foundation shipped; the two-Bases problem closed; `query_base` retired — update the stale §4.x body that still calls `bases.rs` "the" Bases). Session log + MoCh. Help-doc `Bases.md` rewritten for the unified Base (currently documents only the old MVP). Milestone tag `milestone/mig-065-unified-base-foundation`. Push + ZIP.

---

## Landing order & gates
A → B *(blocker gate)* → C → D → E → F *(Boss)* → G *(Boss)* → H *(Boss)* → I → J → K *(Boss, staged)* → L.

After Eisa approves this Plan, the cascade runs autonomously (Plan-Approval-Equals-Build-Approval), pausing only at the Boss-test clauses (F/G/H/K) and any genuine architectural surprise. **MIG-066 (Living Links power columns) is the immediate next MIG** — the first installment of "Strong."
