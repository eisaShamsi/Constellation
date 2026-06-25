# PJ-066 — Build Plan: kill the link-dense reindex freeze (Sky-trigger perf)

**Migration:** PJ-066, `/migration` Phase 2. **Branch/root:** `main` @ `E:\مشاريع كلاود\Constellation`.
**Stack:** Rust (src-tauri) + SvelteKit. **Decision:** Boss approved **Option A + Option B together**.
**Architect source:** `docs/PJ-066-Architect-Sky-Trigger-Perf.md` (invariants carried below).

## Problem (measured, not re-derived)
A connect/save of a link-dense note freezes the app ~1–2 min. `index_note` rebuilds a note's edges via
DELETE-all + INSERT-all `note_links`. Every edge row fires the per-edge sky triggers
`note_links_sky_stratum_ai/ad/au` (`search.rs:3545–3578`) and `note_links_sky_maturity_ai/ad/au`
(`search.rs:3621–3647`), each running `UPDATE sky_nodes SET stratum/maturity = (EXPR)` whose
`COUNT(DISTINCT)` contains the `(target_name = sky_nodes.id OR target_name IN (alias-subquery))`
disjunction. Measured: that disjunction makes the COUNT **5,572 ms / 10,234 ms**; the
`IN (SELECT id UNION SELECT alias…)` rewrite is **26 ms / 429 ms** (20–200×), **result proven identical**
(0 mismatches across the 18 ≥500-inbound targets). For N edges, ~4×N fire under the write lock → the freeze.

## Two approved changes
- **Option A — fast COUNT:** rewrite the `(= OR IN)` disjunction → `IN (SELECT sky_nodes.id UNION SELECT
  alias_lower FROM note_aliases WHERE path = sky_nodes.path)` inside the shared constants `STRATUM_SQL_EXPR`
  (`search.rs:183–225`) and `MATURITY_SQL_EXPR` (`search.rs:256–317`). Shared by the init_db triggers,
  `sky_backfill.rs`, `name_fold_backfill.rs` — single source of truth. Identical results ⇒ **no backfill,
  no schema-version bump for correctness**; triggers are DROP+CREATE'd every `init_db` so the new bodies
  take effect next launch.
- **Option B — batch the recompute:** drop the per-edge `note_links_sky_stratum_*` and `note_links_sky_maturity_*`
  triggers; recompute stratum+maturity ONCE for the **affected set** after the edge churn, mirroring MIG-079
  §C.2a's `incoming_signature` / `maintain_incoming_after_save`. Affected = resolve(symmetric_difference(old,
  new targets)) ∪ {the source note itself}. Connect case → 1 target changes → ~2 nodes recompute → near-instant.

## Open question — RESOLVED (from the code)
`sky_backfill.rs` runs ONLY on a `SKY_SCHEMA_VERSION` bump (not as part of reconcile). `reconcile_filesystem`
currently maintains sky purely via the per-edge triggers firing inside its bulk `index_note` walk. So once
Option B removes those triggers, **reconcile needs an explicit `recompute_all_sky` pass** (the kept
`note_meta_sky_ai` inline value is stale — edges are inserted after note_meta — and does not suffice). The
source must be recomputed after its edges (handled by including self in the affected set). Add a
`drop_sky_aggregate_triggers` helper, call it in the reconcile window, and make init_db STOP creating the
per-edge sky triggers.

## Invariants (must hold across every step)
- **I1 Results identical:** sky_nodes.stratum/maturity equal today's values (A provably; B recomputes with the same shared EXPR).
- **I2 Single-writer:** note_links written only by index_note (DELETE+INSERT).
- **I3 Shared SQL single-source:** STRATUM_SQL_EXPR / MATURITY_SQL_EXPR stay the sole definition, shared with sky_backfill.rs + name_fold_backfill.rs; never inline a divergent copy.
- **I4 Maturity parity:** maturity == compute_state(incoming_count) (MIG-085 §B.1); the maturity EXPR's inbound stays COUNT(DISTINCT source_path).
- **I5 No boot regression:** no new boot-blocking work; reconcile recompute is the existing background self-heal, batched + busy-tolerant.

---

## Ordered steps (each lands as ONE commit)

### §A1 — Option A: rewrite the disjunction in the two shared constants
**Files:** `search.rs` — `STRATUM_SQL_EXPR` (`(= OR IN)` at ~200–202, 220–222) and `MATURITY_SQL_EXPR`
(~261–263, 272–274, 289–291, 303–305). Replace each
`(target_name = sky_nodes.id OR target_name IN (SELECT alias_lower FROM note_aliases WHERE path = sky_nodes.path))`
with `target_name IN (SELECT sky_nodes.id UNION SELECT alias_lower FROM note_aliases WHERE path = sky_nodes.path)`.
Preserve the surrounding `status` filter exactly (`= 'active'` stratum, `!= 'archived'` maturity). Don't touch
the ≥5-inbound `COUNT(*)` signal (204–207) or DISTINCT choices.
**Verify:** `cargo build` + stratum/maturity tests green; a test asserting the rewritten EXPR == old form for
a node matched by name AND by alias; live-DB EXPLAIN sub-30 ms vs 5,572 ms. Connect freeze already drops to a
few seconds. **Independently shippable.**

### §B1 — Add `recompute_all_sky` + wire into reconcile (the safety net, BEFORE removing triggers)
**File:** `links_backfill.rs` (sibling of `recompute_all_incoming:317`) using the shared EXPR (I3). Add a
500-row windowed, busy-retry `recompute_all_sky(conn)` doing `UPDATE sky_nodes SET stratum=(EXPR)` then
maturity. Call it in `reconcile_filesystem` (`search.rs:8227–8240`) after `recompute_all_incoming`.
**Verify:** triggers still present → no-op; reconcile snapshot of sky_nodes(stratum,maturity) before/after =
**0 changed rows**. `cargo test` green. Safe to ship alone.

### §B2 — Add `maintain_sky_after_save` (mirror MIG-079 §C.2a)
**File:** `search.rs`. Add `maintain_sky_after_save(conn, note_path, old_targets, old_name, old_aliases)`
modeled on `maintain_incoming_after_save:1214`. REUSE existing `incoming_signature:1146` (same inputs).
affected = for each target in old.symmetric_difference(new): `resolve_incoming_target_paths:1181`; THEN insert
`note_path` (source) unconditionally. Per affected path: `UPDATE sky_nodes SET stratum=(EXPR) WHERE path=?1`
+ maturity. Best-effort (eprintln); reconcile is the self-heal.
**Verify:** unit test — add-one-link recomputes exactly {target, self}; text-only edit recomputes {} (zero
rows); other nodes byte-identical. `cargo test` green.

### §B3 — Call it from save + delete paths (reuse the captured signature)
**File:** `search.rs`. Save path `reindex_single_note` (8456–8460): reuse the `inc_old` tuple already captured
at 8416; after `maintain_incoming_after_save`, call `maintain_sky_after_save(conn, note_path, &old_t, &old_n,
&old_a)`. Delete path `reindex_delete_note` (8369–8381): resolve `inc_targets` → affected → recompute sky for
former targets ONLY (the deleted note's own sky_nodes row is removed by `note_meta_sky_ad:3372` — do NOT add
self on delete).
**Verify:** triggers still present → connect + delete snapshot diff = 0 changed rows (new path == trigger
output before we rely on it). `cargo test` green.

### §B4 — Drop the per-edge sky triggers; stop creating them; cover the reconcile window (the payoff)
**File:** `search.rs`. In init_db delete the `CREATE TRIGGER … note_links_sky_stratum_ai/ad/au` (3545–3578)
and `note_links_sky_maturity_ai/ad/au` (3621–3647); KEEP `note_meta_sky_stratum_au` (3532),
`note_meta_sky_maturity_au` (3610), `note_meta_sky_ai` (3361). KEEP the unconditional DROP lines (3508–3510 /
3595–3597) so old DBs shed the triggers on next boot. Add `drop_sky_aggregate_triggers(conn)` (mirror
`drop_incoming_link_triggers:1247`) and call it in the reconcile bulk window beside 8213.
**Verify (payoff):** live-DB Boss connect on the heaviest note → write-lock time ~1–2 min ⇒ **sub-second**;
full sky_nodes snapshot vs a pre-PJ-066 reconcile = **0 mismatches** (I1 end-to-end); `cargo test` +
`npm run check` green.

### §B5 (optional / belt-and-suspenders) — bump `SKY_SCHEMA_VERSION` (search.rs:73) 10→11
Forces one background, resumable `sky_backfill` recompute on the live universe's first boot. NOT needed for
correctness (values identical; reconcile self-heals) — decide during build. **Verify:** progress strip shows;
`schema_versions.sky = 11`; values identical.

## Measurement verification (on a COPY of the live universe)
1. Baseline connect on heaviest note (~1–2 min). 2. After §A1: few seconds; EXPLAIN ~26 ms. 3. After §B4:
sub-second; Boss connect test; app stays responsive during the save. 4. Correctness: full sky_nodes snapshot
diff before/after reconcile = 0 mismatches; spot-check the 18 ≥500-inbound targets. 5. `cargo test` +
`npm run check` green.

## Phase-4 Audit focus
- **Invariants** I1–I5 (esp. I3: grep `UPDATE sky_nodes SET (stratum|maturity)` — every writer uses the
  shared constant: triggers, sky_backfill.rs, name_fold_backfill.rs, new maintain_sky_after_save +
  recompute_all_sky).
- **Drift:** no writer bypasses the shared EXPR / new maintenance path; no inline copy introduced in §B2/§B3.
- **Migration path:** existing universe first boot after the trigger change — reconcile's recompute_all_sky
  (and §B5 if taken) restores all values; no node left stale.
- **Reconcile gap (highest risk):** `drop_sky_aggregate_triggers` called in the bulk window AND
  `recompute_all_sky` runs after.
- **Deletion path:** former targets recomputed; absent deleted node not recomputed.

## Risks & mitigations
- **R1 (highest) bulk/reconcile gap:** §B1 lands recompute+wiring BEFORE §B4 removes triggers; §B4 verifies
  via full snapshot diff after reconcile.
- **R2 deletion path:** §B3 mirrors the incoming deletion block; test asserts targets recompute, self excluded.
- **R3 source staleness from note_meta_sky_ai inline value:** §B2 always includes self; §B3 runs after index_note.
- **R4 accented/alias targets:** §A1 test covers an alias-matched node (the 0-mismatch result is on
  alias-heavy identifiers).
- **R5 racing live save during a trigger-free reconcile window:** mirrors the existing incoming design
  (single-writer + busy_timeout serialize; reconcile recompute closes any gap).

## Rollback
- **A:** revert the two constants; triggers re-create old bodies next boot. No data migration.
- **B:** restore the `CREATE TRIGGER … note_links_sky_*` in init_db; remove the drop_sky/recompute_all_sky
  calls. Next boot re-creates the per-edge triggers. §B5 version bump is harmless to leave or revert.
- **Safety net:** a reconcile always re-derives sky from note_meta + note_links → self-heals.

**Commit boundaries:** §A1 · §B1 · §B2 · §B3 · §B4 · (§B5). Plan-approval = build-approval: cascade,
pausing at the §B4 Boss connect re-test.
