# HANDOVER — MIG-066 (Living-Links Columns) — resume at §A.2

**Written:** 2026-05-30 · **Resume point:** MIG-066 **§A.2** (the back-fill) · **Plan already approved** (cascade is authorized).

This is the resume map for a fresh session. Read the **Read-first** docs below, then `git pull`, `git log --oneline -8`, and continue at §A.2.

---

## 0. Read-first (in order)
1. **`CLAUDE.md`** — the working rules. Non-negotiables that bite on this task: **BASIC RULE** (no fabrication — say "I don't know"), **Working Agreement #4** (prove a write-path change safe + *measure* perf before shipping), **Plan-Approval = Build-Approval** (cascade autonomously, pause only at Boss-testable steps), **staged tests** (Stage 1 first), **verify binary mtime before any Boss test** (Stage 0).
2. **`docs/Constellation Orientation & Onboarding v2.46.md`** — architectural fluency (read only the highest version).
3. **THIS FILE** — the §A.2 task + §B–§G + the §E directive + file anchors.
4. **`docs/Living-Link-Concept-Paper-v1.0.md`** — **RATIFIED 2026-05-30.** The single source of truth for link-type semantics + the canonical order. Governs §D and §E.
5. **`docs/MIG-066-living-links-columns-ARCHITECT.md`** + **`...-PLAN.md`** — the design + the 7-phase plan (the Plan reflects the approved scope incl. Eisa's directive).

## 1. The ratified canonical link-type order (front-and-centre)
> **supports · contradicts · causes · exemplifies · generalizes · derives-from · part-of · supersedes**

Derived from the inquiry arc (stance → explanation → abstraction → lineage → composition → succession; Concept Paper §7). `associative` = the null/untyped synonym (NOT in the semantic order). This **confirms the spec + backend** order; the **Living-Links Guide §2 and the 360.3D matrix are drift to reconcile** (that's §E).

## 2. What MIG-066 is
Add two opt-in **Living-Links columns** to the Constellation Base (the "Connection" question as columns): **Outgoing count** + **Link types** (the typed relations a note participates in as a *source*, listed in canonical order), write-time materialized (Rule-8-clean). **Backlinks are deferred to v2** (they cross library/cUniverse boundaries — no cross-schema trigger; see ARCHITECT §4). Plus a reusable **rank-aware sort** (§D) and a full **reconciliation of every Living-Link surface** (§E, Eisa's directive). Governing principle: **Strong yet Simple, by default.**

## 3. What §A.1 already shipped (commit `10d3caf9`, pushed)
The **write-time engine** — new/changed links self-maintain the aggregates. In `src-tauri/src/search.rs`:
- **`ensure_note_meta_mig066_columns`** (near the other `ensure_note_meta_mig00X` helpers) — idempotent ALTER adding 3 columns to `note_meta`:
  - `outgoing_count INTEGER` — active outgoing links (incl. untyped).
  - `outgoing_link_types TEXT` — distinct outgoing TYPED types, **stored in canonical order** (`, `-joined; empty when none).
  - `outgoing_top_rank INTEGER` — min canonical index (1–8) or **9** when no typed links — the **§D sort key**.
  Called from `init_db` right after `ensure_note_meta_mig003_column`.
- **`outgoing_aggregate_assignments(src)`** + consts **`LINK_TYPE_IN_LIST`** / **`LINK_TYPE_RANK_CASE`** — ONE definition of the recompute SQL (encodes Concept Paper §7). Reuse this in §A.2.
- **Triggers `note_links_outgoing_ai/ad/au`** (right after the maturity triggers) — recompute the SOURCE note's 3 aggregates on any edge change (same-DB, source-side, Rule-8-clean; no WHEN guard). Type read from the **`link_type` column** (consistent with the existing stratum SQL at ~search.rs:204-213).
- **Test `tests_mig066_outgoing::outgoing_aggregates_maintained_by_triggers`** — pins the maths, the **canonical-order GROUP_CONCAT** (confirmed working on the bundled SQLite), archive/delete, and the empty-sentinel. `cargo test --lib outgoing_aggregates` is green.

**Caveat:** existing notes still show defaults (0/''/9) until §A.2 back-fills them.

## 4. §A.2 — THE NEXT TASK (do this first)
**Goal:** one-time populate `outgoing_count / outgoing_link_types / outgoing_top_rank` for all existing notes from `note_links`. **Must be batched + background — never a boot-blocking bulk UPDATE** (the MIG-013 lesson: a big bulk UPDATE froze boot for tens of seconds; see memory `project_mig013_v2_migration_blocking_boot.md`).

- **Model it on `src-tauri/src/sky_backfill.rs`**: `maybe_schedule` (54, quick main-thread check then `thread::spawn`), `is_needed` (89, version-stamp gate), `run` (102, re-locks the DB mutex per batch, resumable cursor, `ensure_cursor_table`). Completion stamps `schema_versions`.
- **The gate:** a NEW `schema_versions` module, e.g. **`links_outgoing` = 1**. `is_needed` = stored < target.
- **The per-note recompute SQL** is already written: `outgoing_aggregate_assignments("note_meta.path")` → `UPDATE note_meta SET <assignments> WHERE path IN (<this batch of paths>)`. Batch the paths (e.g. 500/txn), release the lock between batches.
- **Wire-in:** call the new `maybe_schedule` in the boot sequence where `sky_backfill::maybe_schedule` is called (grep `sky_backfill::maybe_schedule` — likely `lib.rs` setup / after `init_db`).
- **Bump** whatever schema-version constant forces a re-run on existing DBs (parallel to `SKY_SCHEMA_VERSION`).
- **Verify (§A.2):**
  1. A Rust test — seed `note_meta` + `note_links`, run the back-fill, assert the columns populate (mirror the §A.1 test setup).
  2. **PERF MEASUREMENT (Rule 8 hard constraint — REQUIRED before §B):** `npm run tauri build`, then on the **7,600-note Universe** measure (a) boot time and (b) a full library **re-index** time — *before vs after*. The 4th trigger family (`note_links_outgoing_*`) fires per-edge during re-index; confirm no meaningful regression. If it regresses, consider a `MIGRATION_ACTIVE`-style trigger-pause during bulk re-index (the `run_bigram_purge` pattern, search.rs:675/770).

## 5. §B–§G (the rest — see MIG-066-PLAN.md)
- **§B (Boss-testable)** — register **`note.outgoing_count`** (Number) + **`note.link_types`** (Text) in `src-tauri/src/lens/dimensions.rs` `resolve_dim` registry (sql_expression reads the materialized columns; sortable) and add to `ADDABLE_REGISTERED_DIMS` (frontend `tableModel.ts` / picker). They appear in **+ Add column → Constellation**.
- **§C (Boss-testable)** — render: count = number; `link_types` = the stored canonical-ordered string rendered as **localized** type names (each via `$t`). Empty = blank. The string is *already* canonical-ordered (§A.1), so no re-sort needed at render.
- **§D (Boss-testable)** — rank-aware sort: a general mechanism (a dimension may declare a canonical order). **For `link_types` the sort key is already materialized — `outgoing_top_rank`** — so sorting that column = `ORDER BY outgoing_top_rank`. Make the mechanism reusable for maturity/stage/stratum (the MIG-068 need).
- **§E (Boss-testable) — RECONCILE EVERY Living-Link surface (Eisa's explicit directive — see §6).**
- **§F** — i18n (column labels + the 8 type names) × **all 15 locales**; help **Bases** topic (+ 14 translations), **User Manual §15**, **orientation** v-bump.
- **§G** — `/migration` Phase-4 **3-agent audit** (invariants / drift / migration-path) + **staged Boss test** + **PCS** (push, milestone tag, ZIP).

## 6. §E surface list (the audit — reconcile ALL to the canonical order; build ONE shared source so drift can't recur)
- `src/lib/components/Inspector360.svelte:43` **`TYPE_ORDER`** — Camp-B order, no `supersedes` → canonical order **+ add a `supersedes` column** (+ `TYPE_COLORS` / `TYPE_LABEL_KEYS`).
- `src-tauri/src/inspector360.rs:70` **`ALL_LINK_TYPES`** — add `supersedes` (drives the matrix gap analysis).
- `src/lib/components/CodeMirrorEditor.svelte:735` **`LINK_TYPES`** — ⚠ **LEGACY/WRONG** (`related-to`/`prerequisite`/`see-also`/`extends` — *not Constellation types*) → the canonical 8.
- `src/lib/editor/completions.ts:83` **`LINK_TYPES`**, `src/lib/editor/livePreview.ts:175` **`TYPED_LINK_TYPES`**, `src/lib/libraries/store.ts:2383` **`KNOWN_LINK_TYPES`** → canonical 8 (+ `supersedes`).
- Color/label maps: `KnowledgeHealthDashboard.svelte`, `SightPanel.svelte` — cover all 8 + `supersedes`.
- **`docs/Living-Links-Guide-v1.0.md` §2** → canonical order; **bump Guide to v1.1** (SO #6, same commit).
- The Rust parsing consts (`libraries.rs` / `strata.rs` / `tension.rs` `KNOWN_LINK_TYPES`) already match Camp A + include `supersedes` — verify, don't churn.

## 7. File:line anchors (the map — verify with grep, lines drift)
- `note_meta` CREATE: `search.rs:1723`. The `ensure_note_meta_mig066_columns` + `outgoing_aggregate_assignments` + consts: `search.rs` ~575–620. Triggers `note_links_outgoing_*`: `search.rs` after the maturity triggers (~2770). Test: `search.rs` `mod tests_mig066_outgoing`.
- Back-fill model: `sky_backfill.rs` (`maybe_schedule`:54, `is_needed`:89, `run`:102). `schema_versions` reads: `search.rs:721+`. `MIGRATION_ACTIVE` / `run_bigram_purge`: `search.rs:675/770`.
- Dimensions/registry: `lens/dimensions.rs:170` (`resolve_dim`), `ADDABLE_REGISTERED_DIMS` (frontend `src/lib/lens/tableModel.ts`). Engine: `lens/query.rs` (`execute_lens`). Base UI: `src/lib/lens/BaseTab.svelte`.
- `discover_keys` (the picker's "Your fields", with the list-item + reserved filters): `lens/query.rs:441`.

## 8. Non-negotiables for this task
- **One location:** `E:\مشاريع كلاود\Constellation`, branch `main`. Commit per `§`, push at boundaries. Commit messages end with `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`. Never use "vault" in new code.
- **WA#4:** §A.2 + §B touch the write/boot path — prove safe + **measure perf on 7,600 notes** before shipping. A slower proven change beats a fast regression.
- **Verify cargo's real output** ("Finished" / error count) — never a piped exit code (the §J convert lesson).
- **Boss tests:** Stage-0 verify binary mtime is fresh; Stage-1 first, wait, then more; tutorial-style (define the feature, then click-by-click).
- **SO:** log each `§` to `lab/reports/SESSION-LOG-2026-05-30.md`; update help/manual/orientation on user-facing change; bump the Concept Paper / Guide when their facts change.

## 9. Deferred / open (don't lose these)
- MIG-066 **v2 backlinks** (federated design — ARCHITECT §5 option C-ii/iii).
- The §A.2 **perf measurement** outcome may add a trigger-pause-during-reindex follow-up.
- Pre-existing deferred PJs (from MIG-065): faithful list/nested `properties_json` (re-index); physical dead-Rust sweep (KEEP dataview-shared helpers); engine-side LIMIT/COUNT split; broaden the legacy-base notice to Obsidian-YAML.
- Concept Paper §9: deeper verification that no shipping tool has the *living/decay* dimension + fuller argumentation-theory grounding before any external claim.
