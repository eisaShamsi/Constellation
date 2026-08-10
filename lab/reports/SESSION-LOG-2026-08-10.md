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

Plan agent briefed with the census, the writers — **CORRECTED by the 4B audit:** the plan and
this log said four writer FILES must stamp the column; 4B enumerated every production statement
and proved `index_note_impl` is the SOLE production row-creator (the other three files' INSERTs
are test fixtures; their production lanes UPDATE columns that cannot affect `target_base`), so
§3's three stamps plus the §4 backfill are complete coverage — the invariants, the rollback property to preserve (no NOT NULL, no old-build
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

## Phase 4 — Audit (three agents, aggregated)

**4A Invariants: 8/8 STILL HOLD**, each with file:line evidence. Two gaps beyond the list, both
fixed in §6c: (1) **the seek had no federation boundary** — the index can hold residual
linked-universe rows (13 live; the §13 purge is blocked on PJ-224), and a matching row would have
had the cascade rewrite a note inside a LINKED universe, a direction the equivalence pin cannot
see (it proves seek ⊇ walk, not seek ⊆ boundary); the shared `foreign` set now filters seek
candidates. (2) `needs_run`'s errored NULL-probe read as "clean" (`unwrap_or(false)`) — flipped
to dirty: the one state we cannot verify walks and heals.

**4B Drift: ZERO.** Every production `note_links` writer enumerated and classified CONFORMS. The
stale-non-NULL hole does not exist for a structural reason: `target_name` is part of the edge's
diff key, so a changed target is DELETE+reINSERTed with a fresh stamp — no path mutates one
without the other. The freshness net's `modified` premise fail-safes both ways (stat failure
stores 0 → permanently suspect → read). Correction absorbed above: `index_note_impl` is the sole
production row-creator.

**4C Migration path: all six scenarios SAFE** — first boot (mid-backfill renames walk; concurrent
saves convergent), schema mismatch (adopt-and-stamp, never aside), mid-backfill kill (stamp is
last; pre-stamp always walks), rollback + return (self-heals via Rearm; the previous build's
init_db verified non-destructive against `git show`), linked universes (backfill and seek both
open the ACTIVE universe's db only; the federated conn is read-only and unused by the cascade).
**Real-data numbers (perf.db):** all 31,367 rows filled; **933 change semantically** (636 folder +
75 `#` + 218 `^` + 11 `.md`, 7 overlapping; the 215 unknown-`::` kept whole by design); **246
folder-qualified targets gain rename coverage for the first time**.

Suite after §6c: **1,434 passed / 0 failed**.

## §6d — the Boss's timing, and what the instrumentation refuted

**Boss's Stage-1 result:** Step 0 exact (`31367 rows updated`); Steps 3 and 4 CORRECT — the log
shows `[cascade] path=SEEK candidates=1` on both renames, i.e. ONE file opened instead of 2,105 —
but **7 s and 8 s**, worse than the ~1 s of the previous build.

The journal attributes it to the millisecond: `rename_chain_resume → cascade_dispatch` = **54 ms**
(so yesterday's tree-walk fix DID close the frontend gap, and my correction to the inspector on
that point was right), then **~6.4 s inside `update_links_on_rename`**, before its own SEEK line.

**The instrumentation refuted my suspicion.** I believed the freshness net's tree walk was the
cost — it is **34 ms** (0 suspects). The 3.2 s sat in the block beside it:
`SELECT path, modified FROM note_meta`. That query IS covered — by `idx_note_meta_map`, which also
carries `outgoing_links_json` at ~300 bytes/row: **798 KB of index pages to read two columns worth
310 KB**. Warm it costs 35 ms and hides; cold on the Boss's USB mechanical disk it costs seconds.
Added `idx_note_meta_path_modified (path, modified)` — SQLite prefers it — and split the timing so
the next measurement separates the candidate seek from the freshness map instead of leaving me to
choose between them by reasoning.

**The lesson, which is LL-037's sibling:** *covered is not the same as cheap.* An EXPLAIN that says
`USING COVERING INDEX` closes the "is it a full scan?" question and says nothing about how WIDE the
cover is. This is the PJ-066 note_meta family wearing a different hat — and I walked into it while
holding the rule.

Had I fixed the freshness net on suspicion, the rename would still be slow and the net — which
genuinely closes two audit-confirmed HIGH findings — would have been damaged for nothing.

## Boss finding (unrelated) — the New Note picker offered another universe's libraries

Ctrl+N listed **25 libraries**, including `Architecture`, `Film`, `Literature`, `Philosophy` and
the Arabic set that belong to the LINKED universe *Eisa Cognitive Knowledge*. Choosing one would
have created a note inside that universe.

Cause: `LibraryPicker` read the federated `$libraries` store directly, while the sidebar has always
filtered linked-universe libraries out (`ownLibraries` → `isChildUniverseLib`). Only the picker had
drifted. Fixed by making the list a REQUIRED prop — the caller decides — with `ownUniverseLibraries`
(own-universe, root kept) passed from `+layout`. Reading a universe-wide list is right for
RESOLVING a name and wrong for CHOOSING where to write; the component no longer gets to guess.

Third member of one family now: PJ-235 (`move_item` authorises its destination through the
federated resolver), the §6c seek boundary, and this. Worth a ledger entry as a class, not three
unrelated bugs — filed at the next bump.
