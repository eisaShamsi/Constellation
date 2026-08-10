# SESSION LOG — 2026-08-10

**PJ-249 `/migration` — normalise the link index so the rename cascade is driven from it.**
Boss: *"Fix PJ-249 first."* Predecessor work: `SESSION-LOG-2026-08-09.md` (PJ-207 §15 close,
commit `2edc97d7` — the performance arc that ended with rename at ~1 s via the parallel walk).

Function in hand: **the rename cascade's referrer lookup** — replace "read 2,105 files / 140.8 MB
to find who links to the old title" with an index seek (measured 8.5 s → 1.8 ms; median rename
opens ONE file).

## Phase 1 — Architect (done)

Mapped writers, readers, and the dirty-row census on the Boss's real data. The headline: **the
blocker is 4× bigger than the PJ entry recorded.** The ledger said 290 malformed
`note_links.target_name` rows (75 `#` + 215 `::`); the full census is **1,148 dirty rows across
314 distinct targets**, and the biggest class was one nobody had counted: **637 folder-qualified
links** (`[[folder/Note]]`) storing the folder path inside the target name — a form the cascade
regex has NEVER matched, walk or not, so those links have never followed a rename at all.

Facts that shaped the options:

- **Every reader uses exact folded equality**, so all 1,148 dirty rows are ALREADY invisible to
  backlinks, incoming aggregates, Sky View, the back-resolution triggers, and the frontend.
  Normalising only ever ADDS correct results — no reader regresses.
- **Zero dirty rows carry `target_cid_cn` or earned link data** (7 earned rows exist DB-wide,
  none dirty) — nothing precious is at stake in a merge.
- **In-place normalisation (option a) has teeth**: 60 rows collide on
  `UNIQUE(source_path, target_name, link_type)` and must be merged by rule; and rollback is lossy —
  an older build's parser re-dirties rows on every save, silently re-splitting the universe.
- **A new column (option b/d) is rollback-safe by construction**: an older build never reads it.
- Invariant with a scar attached: `sky_links` stays byte-identical to active non-structural
  `note_links` VIA THE TRIGGERS — never write `sky_links` directly (the PJ-207 §15 4,359-row
  rename corruption was exactly that, one layer over).
- The named most-likely-failure: **the index hands the cascade a `[[folder/Old]]` referrer the
  regex cannot rewrite**, and the rename reports success while skipping it — fast-but-quietly-
  incomplete, the outcome PJ-249 was filed to avoid.

## Boss rulings (Phase 1 gate)

1. **Option (d): new clean column.** `target_base` (bare folded title) added alongside
   `target_name`; back-filled in the background on the `name_fold_backfill` template; the cascade
   seeks on it only once the per-universe stamp lands, and stays walk-based below the stamp.
   `target_name` and every existing reader untouched; retiring the raw form is a later migration.
2. **Folder-qualified links are IN SCOPE.** `cascade_pattern` learns `[[folder/Old]]` in the same
   pass — otherwise the fast cascade would knowingly skip 637 referrers. Same widening class as
   the typed/anchor forms the Boss approved and Stage-1-tested on 2026-08-10.

## Phase 2 — Plan (in flight)

Plan agent briefed with the census, the four writers (`index_note_impl` + `links_backfill` +
`link_life_restore` + `link_life_backfill` — ALL must populate the new column or it drifts,
LL-023 class), the invariants, the rollback property to preserve (no NOT NULL, no old-build
schema trip), the unregistered-`::`-head question to settle, and the Reproduce-First requirements
(the folder-form miss and the mixed-universe gate need failing tests first).

## Phase 3 — Build (§1–§6 landed, one commit each)

| § | Commit | What |
|---|---|---|
| §1 | 413c83d5 | `note_links.target_base` — idempotent xinfo ensure + index; NO `.version` bump (that gate sets the DB aside); old-build INSERTs leave NULL by design |
| §2 | 2192c941 | `target_base_of` — anchor cut → last segment → `.md` strip → shared fold. **The Arabic test caught a real panic before it shipped**: the `.md` check byte-sliced `len()-3`, which lands inside a multi-byte char on an Arabic title. Boundary-safe via `str::get` |
| §3 | bea10433 | The three production INSERTs stamp `target_base`. Deviation, reasoned: the five fixture INSERTs stay UNSTAMPED — a column-omitting INSERT is the pre-backfill shape §4's tests need |
| §3b | 3ece24b9 | Two hand-mirrored test schemas gained the column they drifted from (the full suite caught it; the pj249-filtered run could not). Also recorded: §3's commit chained past the failure — gated on grep, not the suite. Fixed forward |
| §4 | 9bbe4f50 | The backfill + THE DRIFT GUARD: re-arms when a NULL row exists behind the stamp (an older build's session). Unstamps first, restamps only on completion. Core factored to `run_on(&mut Connection)` — four tests on real `init_db` fixtures |
| §5 | 071674a5 | Folder-qualified links follow a rename, red→green: four positives observed FAILING against the pre-widening pattern; negatives held on both sides |
| §6 | d043490e | The flip: `cascade_candidates_via_index` = gate + seek, the gate being `needs_run == No` — the same predicate as the backfill's re-arm, so they cannot disagree. Seek candidates get the walk's PJ-092 exclusion + a stat-guard; the 'unhit exclude' warning scoped to the walk path. Equivalence pin (seek ⊇ walk) + mixed-universe refusal test |

Suite at §6: **1,432 passed / 0 failed**.

## Phase 3 close-out (in flight)

`/simplify` (4 lenses) + the per-build diff-scoped safety inspection running over
`2edc97d7..d043490e`. Then Phase 4 (three-agent audit), the fresh binary, and the Boss test via
tutorial-auditor → ui-inspector.
