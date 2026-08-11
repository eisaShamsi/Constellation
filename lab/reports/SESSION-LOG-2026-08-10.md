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

**The lesson, which is LL-043's sibling:** *covered is not the same as cheap.* An EXPLAIN that says
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

---

## §6f — the index the migration built was never used

**Boss test of §6e: "It took 4 seconds to rename."** Worse than the 2,693 ms it was meant to fix.
The §6e diagnosis (cold connection, warm it at boot) was **wrong**, and the warm proved it:

```
[target_base_backfill] seek path warmed in 674 ms (ok=true)
[target_base_backfill] seek path warmed in 815 ms (ok=true)
   ... 205 seconds later ...
[cascade-timing] seek-query 2579 ms | freshness-map 2730 rows 2 ms
```

The warm ran, succeeded, finished three minutes early, and bought nothing.

### The actual cause, read off the live DB (a copy — never the live file)

```
EXPLAIN QUERY PLAN  SELECT DISTINCT source_path FROM note_links WHERE target_base = ?
  ->  SCAN note_links USING INDEX idx_link_source            <-- a FULL SCAN of 31,368 rows

sqlite_stat1:  idx_link_target_base -> 31367 31367           <-- "one distinct value, all rows"
real cardinality: 3.8 rows per value
```

**`idx_link_target_base` has never been used since the day it was created.** The statistic was
collected while `target_base` was entirely NULL, and §4's back-fill — whose whole purpose is to
turn that column from uniform into diverse — never re-collected it. The planner was reasoning
correctly from a number the back-fill itself had made false. Verified as the ONLY stale stat on
the table; the other eight were checked and are accurate.

So PJ-249 did not replace an 8.5 s file walk with a 44 ms index seek. It replaced it with a
**2.6 s full table scan**, and I read the improvement as success. The 44 ms second rename was the
same full scan served from the OS page cache.

### Measured, on a copy of the live 327 MB DB, same key, same data

| | plan | time |
|---|---|---|
| as shipped | `SCAN note_links USING idx_link_source` | 16.946 ms |
| + `ANALYZE` only | `SEARCH idx_link_target_base` + temp b-tree | 0.023 ms |
| + index widened to `(target_base, source_path)` | `SEARCH … COVERING INDEX` | 0.006 ms |
| + widened, **sqlite_stat1 deleted** | `SEARCH … COVERING INDEX` | 0.006 ms |

(16.9 ms is a warm SSD copy; the same scan on the Boss's USB disk is his 2,579 ms.)

The last row is why the fix is the **widened index**, not `ANALYZE` alone: a covering index that
satisfies the whole query is chosen structurally, so a stale statistic cannot resurrect the scan.
`ANALYZE` alone fixes today and leaves the trap armed for tomorrow.

### The fix — three parts, all off the boot path

1. **`idx_link_target_base` carries `source_path`.** Fresh universes get the shape from
   `ensure_note_links_target_base`. Existing ones get it from `widen_seek_index` in the back-fill,
   because **`CREATE INDEX IF NOT EXISTS` is a silent no-op against a name that already exists** —
   every universe that booted the §1 build would have kept the narrow index forever. The shape is
   detected via `pragma_index_info` and the index rebuilt (measured 56–118 ms).
2. **`analyze_note_links`** — the back-fill re-collects the statistic it invalidated (78–89 ms),
   scoped to the one table. Not needed by the cascade any more; needed because leaving a knowingly
   false statistic in the DB is a trap set for the next query written against the column.
3. **`SCHEMA_VERSION` 1 → 2**, using the mechanism its own doc comment already described. An
   already-stamped universe re-arms once, finds zero dirty rows, repairs index + stats, re-stamps.
   Re-arm pass measured end-to-end at **289 ms** on a copy. During that window the gate reports
   not-stamped and the cascade walks the disk — slow and correct, the intended fail-safe direction.

`warm_seek_path` now calls **`cascade_candidates_via_index` itself** (`pub(crate)`) instead of a
hand-written lookalike. That was §6e's failure in miniature: `COUNT(*)` planned as a covering-index
SEARCH while the real seek planned as a full SCAN — two spellings of one question, warming disjoint
parts of the file. Same shape as §6b's two key functions. One caller, one function, no drift.

### The test that was missing

Five new tests in `tests_pj249_6f_seek_plan` pin **the plan**, not the data — including
`narrow_index_loses_to_a_stale_statistic`, which poisons `sqlite_stat1` and asserts the OLD index
is rejected, so the fix is provably a fix and not a coincidence.

Every §1–§4 test asserted the right `target_base` values. Every one passed. All of them passed on
a build that full-scanned 31,368 rows on every rename. **The data was never wrong; the plan was —
and a correctness test cannot see a plan.**

Suite 1,439 passing (+5). Binary 20:02.

### The class, not the instance

Any back-fill that fills a previously-uniform column leaves `sqlite_stat1` describing the old
cardinality. `note_links.target_base` is the instance that happened to be measured. A whole-app
audit of every back-fill for the same trap is running.

## §6g — the same trap on a second index, found by auditing the class

The §6f class audit (34 backfills, adversarially refuted, against a copy of the live DB)
confirmed **one** stale-statistics instance — the one already fixed. But it surfaced the
**mirror-image** of the shape trap, live and unticketed, and I verified every claim by hand
before acting on it.

### `idx_link_boot` stopped covering in June and its test could not see it

MIG-079 §C.3 built `idx_link_boot` so the boot edge load reads index pages instead of the
wide `note_links` row-store. Commit `6c810836` (*§3a — read-widening: a link's `created` now
reaches the UI*) added `created` to the projection in `cache.rs` and not to the index.

```
production (12 cols, incl. `created`)   ->  SCAN note_links
the test's own 10-col string            ->  SEARCH ... USING COVERING INDEX idx_link_boot
```

**The test spelled the projection out a THIRD time** — ten columns, missing `created` and
`status` — so it asserted a covering plan for a query the app does not run, and stayed green
from June to 2026-08-10 while production scanned. Same shape as §6f's warm: *two spellings of
one question drift apart in silence, and the copy is the one that gets tested.*

**And its documented repair path could not work.** `link_boot_index.rs:40` said "Bump to
force a rebuild (e.g. if the covering column set changes)" above a `CREATE INDEX IF NOT
EXISTS` with no `DROP`. `IF NOT EXISTS` keys on the NAME: bumping re-ran a no-op and
re-stamped. No existing universe could ever have received a new column set.

### Measured before widening, because "covered" is not "cheap"

| | plan | time |
|---|---|---|
| as shipped | `SCAN note_links` | 71.8 ms |
| `created` added | `SEARCH … USING COVERING INDEX idx_link_boot` | 70.2 ms |

Rebuild 232 ms once; database file +0 KB. **No warm gain** — that benchmark is CPU-bound on
materialising all 31,368 rows and cannot see the I/O difference that shows on a mechanical
disk — but no regression, and it restores the invariant the module exists for. Shipped on
"restores a documented invariant at no measured cost", not on a performance claim.

### The fix

- **`search::ensure_index_shape`** — reads the current shape via `pragma_index_info`, drops
  when it differs, then creates. **Shared** by `widen_seek_index` (§6f) and `link_boot_index`,
  because the same trap appeared on two indexes within an hour and a second hand-rolled copy
  is how it becomes a third. Documented as background-only: rebuilding a 31k-row index is
  118–232 ms and belongs nowhere near boot.
- **`BOOT_LINK_COLUMNS`** — one constant. `cache.rs` projects from it, the index is built from
  it, the tests derive from it. The three cannot be edited apart.
- **`SCHEMA_VERSION` 1 → 2** in `link_boot_index`, and now the bump actually rebuilds.

### The risk the fix introduced, closed in the same pass

`cache::read_links_in_schema` reads its results **positionally** (`row.get(0)`…`row.get(11)`).
Extracting one shared string removed the drift and put a new hazard in its place: re-ordering
the constant compiles, passes every other test, and silently feeds each link's annotation into
its confidence. `boot_projection_order_is_pinned_to_the_positional_reads_in_cache_rs` pins the
exact order with the `row.get` index beside each column.

### And a hole in §6f that the safety sweep caught, in code written the same evening

`with_read_conn` holds `read_db` for the whole closure. Right for a covering-index seek;
wrong for a full scan — which is what the seek degrades to if `widen_seek_index` failed (it is
non-fatal and the pass stamps regardless). The boot warm would then block **every other read**
for seconds. It now verifies the index is covering before seeking, and logs
`seek warm SKIPPED` instead of stalling.

Suite 1,442 passing (+8 over the session's start).

### Not fixed — needs a Boss ruling

The sweep confirmed a **HIGH** in the cascade itself: the seek folds case
(`target_base_of` → `fold_match_key`) so `[[meeting notes]]` IS returned when renaming
"Meeting Notes", but `cascade_pattern` matches literally via `regex::escape`, so the link is
read and left pointing at a title nothing owns — and the rename reports success. **Not a
PJ-249 regression**: the old walk used the same regex and missed it identically. Fixing it
changes which links get rewritten on disk, so it is surfaced rather than slipped into a build
under test. Same for the 25 pre-existing whole-app findings, including the rename/move/create
tails resolving their library through the FEDERATED list (the Ctrl+N family).

## §6h — instrument the freshness net (Boss-directed)

### The §6f/§6g Boss test: the fix worked, and the bottleneck moved

| | before | after |
|---|---|---|
| `seek-query` | 2,579 ms | **62 ms** |
| boot warm | 674 / 815 ms | 17 / 102 ms |
| **total rename** | 2,878 ms | **1,673 ms** |

`[target_base_backfill] completed: 0 rows updated` — the re-arm found nothing dirty, exactly
as designed: the DATA was always right, only the index shape and the statistic were wrong.
The Boss waited 2¾ minutes (rename at 1786385872, boot repair at 1786385666), so none of
this is the fallback window.

**1,673 ms is not a pass by the criterion I gave him**, and it is recorded as a fail.
Of it, **1,442 ms is the freshness net** — `index-seek-and-freshness-map-done at 159 ms` →
`freshness-net-done (1 suspects) at 1601 ms`.

### The measurement I cannot explain, which is why this section is instrumentation and not a fix

The same stat-walk, over the same library (`Constellation PKM` — verified from the
`[rename-tail] START` lines of every run, all six are in it):

| session | suspects | net |
|---|---|---|
| 1786375983 (first rename of that session) | 0 | **49 ms** |
| 1786385872 (first rename of this session) | 1 | **1,442 ms** |

29× apart. `t_phase` fires immediately after `collect_fresh_suspects` returns, so nothing
downstream is inside the mark; and a suspect costs a `PathBuf` push. **The only logged
difference cannot be the cause, and I do not know what is.**

I blamed this same net on suspicion earlier tonight and it was 34 ms. So: no fix, no
theory — the split that will name it.

### What was added

`NetStats` counts what the walk did and times the only two syscall families it spends in:

```
freshness-net-done (N suspects) [D dirs, M .md (u unknown, d drifted) | read_dir X ms | metadata Y ms]
```

That separates *walked more* (counts) from *walked slower*, and a cold directory cache
(`read_dir`) from slow per-file stats (`metadata`) — the three candidate explanations.

Incidental, and an improvement rather than a cost: timing the enumeration required draining
each directory's entries before recursing, so the directory handle now **closes before
descending** instead of being held open down the whole depth of the tree.

Suite 1,442 passing.

## Session close — PCS

**Commits:** `904bccbc` (§6f–§6h) + this docs commit.
**Gates:** Rust **1,442/0** · vitest **927 passed + 3 expected-fail** (82 files) ·
svelte-check **0 errors** · release binary **22:35**, Boss-validated at **216 ms**.

**SO #9 — Pending Jobs reconciled** → `docs/Constellation Pending Jobs v1.79.md`.
PJ-249 **closed** (and recorded as having closed in a *different shape* than v1.78 described —
Boss-ruled `target_base` column, not the `target_name` normalisation). Filed **PJ-252** (the
tag-destroying APP-KILLER, reproduced + pinned + exposure measured at 1/10,077), **PJ-253**
(the case-fold miss, ruling required), **PJ-254** (federated resolver in every write-path tail),
**PJ-255** (six ungenerationed detached tails), **PJ-256** (no back-fill re-collects its
statistics), **PJ-257** (`props_reparse` fails every boot forever). Group 1 re-ranked;
► Next action is now PJ-252.

**SO #6 — Orientation** → `docs/Constellation Orientation & Onboarding v3.95.md` (v3.94 kept).

**SO #2 — User Manual, all 15 languages.** The "every form of link is updated" paragraph
gained the **folder-qualified** link (PJ-249 §5 — 637 live rows spell their target that way
and had never followed a rename) **and an honest caveat**: a link whose capitalisation differs
from the note's title still resolves but is NOT rewritten (PJ-253). The manual was
*overclaiming*; a manual that promises what the app does not do is worse than one that admits
the gap.

**MoCh** → `docs/MoCh/MoCh-2026-08-10-1900.md`. **Handover** →
`lab/reports/HANDOVER-2026-08-11-pj249-close.md`, with the ready-to-paste next-session prompt.

**Durable registers** → `lab/reports/sweeps/SWEEP-2026-08-10-fourth-whole-app.json` (25
confirmed) and `SWEEP-2026-08-11-fifth-whole-app.json` (29), plus
`AUDIT-2026-08-10-stale-statistics-class.json`. Most of those 54 are **not yet individually
numbered** — stated plainly in v1.79 rather than implied closed.
