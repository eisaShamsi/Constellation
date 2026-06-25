# PJ-066 — Architect: the sky-trigger reindex storm (the ~1-min connect freeze)

**Migration:** PJ-066 (perf). **Branch/root:** `main` @ `E:\مشاريع كلاود\Constellation`.
**Concept (the horse):** *Reindexing one note must stay cheap and must not hold the database hostage.
A note's save should never block the app while it recomputes universe-wide sky aggregates.*

Phase 1 of `/migration`. **Grounded in measurement on the live universe** ("Eisa Cognitive Knowledge":
7,659 notes, 234,010 `note_links`/`sky_links`, 2 GB) per the Reproduce-First rule.

---

## 1. The territory

`index_note` re-derives a note's edges via **DELETE-all `note_links` for the note + INSERT-all** (the
load-bearing traversal-snapshot pattern at `search.rs:976`/`3323`). Every such row fires the per-edge sky
triggers in `search.rs`:
- `note_links_sky_stratum_ai/ad/au` → `UPDATE sky_nodes SET stratum = (STRATUM_SQL_EXPR) WHERE id = NEW.target_name`
- `note_links_sky_maturity_ai/ad/au` → `UPDATE sky_nodes SET maturity = (MATURITY_SQL_EXPR) WHERE id = NEW.target_name`

Both `STRATUM_SQL_EXPR` (search.rs:~218) and `MATURITY_SQL_EXPR` (search.rs:~256) contain
`COUNT(DISTINCT source_path) FROM note_links WHERE status … AND (target_name = sky_nodes.id OR target_name
IN (SELECT alias_lower FROM note_aliases WHERE path = sky_nodes.path))`.

So for a note with **N** edges, `index_note` fires ~**4×N** of these aggregate UPDATEs (DELETE side +
INSERT side, stratum + maturity), each evaluating that COUNT for the edge's target.

## 2. Measured root cause (the smoking gun)

The `(target_name = ? OR target_name IN (alias-subquery))` **disjunction** makes the `COUNT(DISTINCT)`
plan degenerate (`MULTI-INDEX OR` + bloom filter) even though `idx_link_target` exists:

| Query form (heavy target "isbn (identifier)", 5,358 inbound) | Time |
|---|---|
| CURRENT `(= OR IN)` — bound param | **5,572 ms** |
| CURRENT `(= OR IN)` — correlated trigger form, 3 heaviest nodes | **10,234 ms** |
| REWRITE `target_name IN (SELECT ? UNION SELECT alias…)` — bound | **26 ms** (215×) |
| REWRITE `IN(UNION)` — correlated trigger form, 3 heaviest nodes | **429 ms** (24×) |
| `target_name = ?` (no alias) | **26 ms** |

- Link-dense notes have up to **531 edges**; **18 targets** have ≥500 inbound (the identifiers isbn/doi/
  issn/s2cid…). A note citing a handful of these fires the 5–10 s COUNT per edge × ~4 → the ~1–2 min freeze.
- The freeze is felt app-wide because the reindex holds the SQLite **write lock** for that whole time, so
  every other IPC/query blocks behind it.
- **The handover's guessed "composite index" fix would NOT work** — `EXPLAIN` shows the index is already
  used; the cost is the OR-plan, not a missing index. (Stated so the audit knows it was considered.)

## 3. Verified fix correctness

Rewriting `(target_name = sky_nodes.id OR target_name IN (alias-subquery))` →
`target_name IN (SELECT sky_nodes.id UNION SELECT alias_lower FROM note_aliases WHERE path = sky_nodes.path)`:
- **0 mismatches across all 18 heavy targets** (identical `COUNT(DISTINCT)` result — e.g. 5358==5358).
- `note_meta.incoming_count` was also confirmed to EXACTLY equal the distinct-source count (5358/4359/2999)
  — relevant to Option B below.

## 4. Options

| # | Option | What | Speed gain | Effort | Risk |
|---|--------|------|-----------|--------|------|
| **A** | **Rewrite OR→IN(UNION)** in `STRATUM_SQL_EXPR` + `MATURITY_SQL_EXPR` (shared by the triggers AND `sky_backfill.rs`) | Surgical SQL change in 2 constants | ~20–200× per firing → ~2 min ⇒ a few s | **Low** | **Low** — results provably identical; triggers re-create on next boot via the existing unconditional DROP+CREATE; **no schema bump, no backfill** (values unchanged) |
| B | Batch the recompute | Drop the per-edge stratum/maturity triggers; have `index_note` recompute stratum/maturity ONCE for the affected nodes (source + distinct targets) after the edge churn | Removes the 4×N fan-out → near-instant | Med-High | Med — changes the write path + trigger architecture; needs the full 4C audit |
| C | Composite index | (the handover's guess) | **none** | Low | — REJECTED by measurement (index already used) |

## 5. Invariants that must not break
- **Results identical:** sky_nodes.stratum/maturity must equal today's values (A is provably so; verified).
- **Single-writer:** `note_links` still written only by `index_note` (DELETE+INSERT); A touches no write path.
- **Shared SQL single-source:** `STRATUM_SQL_EXPR`/`MATURITY_SQL_EXPR` are shared with `sky_backfill.rs` —
  the rewrite must land in the constants so triggers AND back-fill stay identical (cannot drift).
- **Reviewer/360/maturity parity:** maturity must still equal `compute_state(incoming_count)` (MIG-085 §B.1).
- **No boot regression:** A adds no backfill; B's first run must be background-after-paint + resumable.

## 6. Migration / rollback
- **A:** triggers are DROP+CREATE'd unconditionally every `init_db`, so the new bodies take effect on next
  launch automatically; existing sky_nodes values stay correct (identical math). Rollback = revert the 2
  constants. No schema-version change, no data migration.
- **B:** would need a schema-version bump + a one-shot background backfill (resumable, status-bar progress).

## 7. Recommendation
**Ship Option A now** (low-risk, verified, ~20–200× per firing — turns the ~1-min freeze into a few seconds
of brief background contention). **Then re-measure on the live universe**; if still not acceptable, do
Option B as a fast-follow for near-instant. This is the proportionate, measured path — don't take the
invasive write-path change (B) until A's measured result proves it's needed.
