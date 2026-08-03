Verified against HEAD `5dbe0a2c`. Every `file:line` below I read this session. Baseline re-measured: `node scripts/i18n-parity.mjs` → **✓ All 15 locales in parity**.

# PJ-207 — The Index Repair · `/migration` Phase 2 (PLAN — FINAL)

**Design B — "re-found the walker"** (Boss-chosen, R4). **15 steps, each one commit.**

The draft had 16 steps in a conceptual order (guards → plumbing → door). Two reviews proved that order does not compile-and-render: three steps forward-referenced symbols that arrive later, one shipped a destructive user action three steps before its host UI, one could not land i18n-green, and one merged two passes whose atomicity contracts differ. **This revision reorders, merges three steps into one, splits two, and closes six silently-dropped dangers.** Every change is marked `[revised: …]`.

**Ordering law, restated and now actually honoured:** §1 ships alone. §2–§10 are guards, hardening and plumbing — **none adds a user-reachable route to the walk**. §11 is the door. §13 (deletion) is after the door, not before it.

---

## The two hazard rulings — both re-decided, because the draft failed them

**H1 — no in-process pause flag. `[revised: the draft's own mechanism was the rejected one, renamed.]`**
The draft said "the three creation sites stay byte-unconditional," then three sentences later said `on_link_vocabulary_changed` "takes the same run mutex." Those contradict: the creation site at `search.rs:1889` **is inside** `on_link_vocabulary_changed` (`pub fn` at `:1880`, `create_outgoing_link_triggers(conn)` at `:1889`). Gating the enclosing function is the `if` moved up one stack frame — H1 verbatim, and it would also park `save_universe_link_types` (`link_types.rs:544`) behind a run whose measured I/O floor is 49 s.

**Final ruling: there is NO gate at `search.rs:1889`, none at `search.rs:4831`, none at `mig108.rs:1199`.** The vocabulary path runs unconditionally, including its two `maybe_schedule` calls (`search.rs:1901`, `:1902`). The runner absorbs a mid-run re-arm instead of preventing it, and it can, because `create_outgoing_link_triggers` **drops first, then creates** (`search.rs:1500-1503`: *"drop first so the triggers always carry the CURRENT registry's rank CASE"*) — so a re-arm is idempotent, and the run's own `converge` tail recomputes every aggregate after. **Cost of a mid-run re-arm: the O(N²) trigger fire the walk was avoiding (~+17 s on a 216k-link universe, `search.rs:1875-1878`). Cost is performance, never correctness.** That is the trade H1 was ruling for.

The window itself is bounded by an **RAII guard** — `TriggerWindow`, precedent `mig108::RunningGuard` (`mig108.rs:50-59`, *"RAII so the flag clears on EVERY exit path — the `?` early-returns inside the engine loop and a panic included"*). **Stated failure mode:** a `SIGKILL` or power loss does not unwind, so `Drop` never runs and the triggers stay dropped — which is exactly what the on-disk marker at `search.rs:1558/1568/1578` is for, and is today's behaviour too.

**H2 — `MIGRATION_ACTIVE` is not taken by the runner. `[revised: the draft accepted a transitive door that takes it anyway.]`**
Direct H2 was correctly discharged (`search.rs:9730-9732`: the daemon sleeps 15 s and re-polls while set; `:9741`: a 300 s cadence, far too slow to bound a full re-read). The runner takes only `PRAGMA wal_checkpoint(PASSIVE)` on its own connection every 500 notes.

But the draft then **accepted** the pre-run `maybe_schedule_defrag` as "harmless." Read the worker it schedules: `search.rs:2158` sets `MIGRATION_ACTIVE`, `:2164-2165` holds `state.db.lock()` across `wal_checkpoint(TRUNCATE)` then `VACUUM`, `:2167` clears. Its own doc (`search.rs:2050-2052`) says *"holds the DB lock for its duration — minutes on a multi-GB file, and unchunkable by nature."* The dismissal reason was backwards: a large freelist is precisely the **post-mass-rewrite** state a repair produces. **Final ruling: §7 adds mutual exclusion with the defrag worker in both directions.** Verified the schedule site is init-only — `ensure_search_db_ready` early-returns on `db_ready` (`search.rs:9999`) before reaching `:10257` — so it fires on the cold path only; exclusion is cheap and mandatory anyway.

---

## §1 — Link promotions and birth dates survive a re-index

**SHIPS ALONE.**

**Files** `search.rs:7115-7117` (the live predicate `(tc > 0 || w != 1.0 || status != "active") && !is_structural_type`) · `search.rs:7095-7097` (the row already reads `confidence` at index 11 and `created` at 12) · `search.rs:7121-7124` (`preserved` stores both; `old_edges` stores neither) · `search.rs:7210-7214` (the non-preserved INSERT hardcodes `'hypothesis'`, `1.0`, and binds `?6 = now` into **both** `created` and `last_traversed`) · `search.rs:338` (`is_preserved`, the hand-mirrored copy) with tests at `:345 / :355 / :361 / :369 / :376`.

**Change** One shared `link_row_is_preserved(traversal_count, weight, status, confidence, structural)`; the mirror at `:338` is **deleted**, and the test module calls the production function. The predicate gains `confidence != "hypothesis"`. Separately: `created` is added to the `old_edges` tuple (`search.rs:7124`) so a non-preserved edge whose `(target, link_type)` key already existed carries its stored birth date forward instead of `now`. `[revised: the draft never named the `old_edges` tuple change — it is a second data-shape edit in the same commit and must be stated.]`

Structural edges (`search.rs:7186-7197`) untouched — they carry no living-link apparatus by design.

**One scope note the test recipe depends on:** the `unchanged` fast path (`search.rs:7126-7133`) requires `o_status == "active"` and short-circuits when the edge set is identical, so the loss fires **only when the note's edges actually changed**. The test must change an annotation, not just re-save.

**VERIFICATION CLAUSE** After this step, all five existing behaviours hold unchanged (archived preserved on status alone; untouched active hypothesis **not** preserved; earned history preserved; structural never preserved; stored status restored verbatim). New `#[cfg(test)] mod tests_pj207_preserve` using the `init_db(&tmp)` idiom (`search.rs:12495`) — `index_note` takes a bare `Connection` (`search.rs:6569`), no `AppHandle` needed, which is why this is the first-ever test of the walk primitive. Assert `confidence == 'established'` and `created` unchanged after an annotation edit + re-index.

**RED-proof `[revised: the draft prescribed committing a failing test — a red tree.]`** Demonstrate red **locally** by reverting only the production predicate; paste the observed failure output into the commit message and the session log. **The committed tree is green.** Because both call sites now share one function, the `search.rs:338` trap is structurally closed regardless.

Diff-scoped `safety-inspection` before commit.

**BOSS-TESTABLE — yes.** Open a note that links to another. Set that link's confidence to "established". Edit the note's text and save. Reopen: the link must still say "established" and still show its original creation date.

**Risk discharged:** Invariant 2; the lead's mandatory-first ruling; the `search.rs:338` mirror trap.

---

## §2 — Delete the two dead doors, the orphan UI, and add the two off-switches

`[revised: now also defines both rollback flags (the draft had §11 gated on a constant §14 defined), and proves the event survives the deletion.]`

**Files** `cache.rs:1511-1542` (`cache_reconcile`; verified zero `invoke('cache_reconcile')` anywhere in `src/`) · `lib.rs:506` (its registration) · `SettingsModal.svelte:2795-2871` (13 `.semantic-*` rule blocks, zero markup users) · `store.ts:6156` + `:6538` (`index.semanticSearchEnabled`, zero readers) · `SettingsModal.svelte:110-116` (the comment recording *"left in the settings shape for backward compat but is no longer read anywhere"* — this commit is that GC).

**Change** Pure deletion, plus two module-level constants: `REPAIR_DOOR_ENABLED` (gates §9's drift notice, §11's Settings control and health-bar action) and `FULL_REREAD_ENABLED` (gates §14 alone). `constellation_search_init` (`search.rs:10639`) is untouched here; §7 re-routes it.

**D1 is moot by deletion.** `cache.rs:1526-1529` maps `Err` → `(0, true)`, emitting a failure as a successful cold walk with 0 notes. The command is deleted; the surviving path (§7) carries a typed outcome that cannot collapse a failure into a zero. No separate fix is filed and none is needed.

**VERIFICATION CLAUSE** `cargo build` clean (a dangling `generate_handler!` entry is a compile error, so the registration removal is compiler-verified). **`grep -rn "cache-reconciled" src-tauri/src` still returns an emitter — `cache.rs:1561` in `cache_mark_search_ready`, which is the one boot actually calls (`+layout.svelte:2905`)**, so the three listeners (`+layout.svelte:3641`, `:3684`, `CollectionsPanel.svelte:79`) keep working. `[revised: without this line a reviewer cannot distinguish this deletion from one that orphans three listeners.]` Settings → Index renders and both toggles persist across restart. `npm run test` green; `svelte-check` 0.

**BOSS-TESTABLE — yes.** Open Settings → Index, flip both toggles, restart, confirm both stayed where you put them.

**Risk discharged:** D1 (moot), D6; R4's "delete both dead doors" (first half).

---

## §3 — The per-note indexer stops reporting success it did not earn

`[revised: the draft moved file reads inside the writer lock without saying so, and claimed a race was closed that second-resolution mtime leaves open.]`

**Files** `search.rs:6592` (stat), `:6609` (`read_to_string`), `:6611-6614` (parse/extract/strip), `:6749` (`BEGIN IMMEDIATE`) · `search.rs:7249` (`read_dir` failure → bare return), `:7262` (`let _ = index_note(...)`) · `search.rs:10610` (bare `COUNT(*)`) · `search.rs:11122 / :11136 / :11150` (three `eprintln`-only maintenance calls inside `reindex_single_note`, which still returns `Ok(())`).

**Change**

1. **D4 — precisely scoped.** Only a **re-stat** moves inside the transaction. The file read, the frontmatter parse, the wikilink/heading extraction and the markdown strip **stay outside** — moving 298 MB of reads and every parse inside per-note write transactions would manufacture the freeze this migration is fixing. Inside `BEGIN IMMEDIATE`, re-stat; if `modified` moved since the pre-read stat, **abort this note leaving its row untouched** and count it as `raced`. **Stated limitation, not papered over:** `modified` has second resolution and `search.rs:6585-6591` (PJ-060) documents it — *"a write landing in the same second as the cached one is invisible to it."* A save landing in the **same second** as the walk's stat still passes the comparison and is still overwritten. This step narrows the window from "the whole read+parse span" to "sub-second"; it does not close it. Closing it needs content hashing, which is its own job (filed in §15).
2. **D5.** `index_library_recursive` returns `WalkTally { seen, indexed, skipped, raced, failed, dirs_unreadable }`; `:7262`'s `let _ =` accumulates; `:7249` records the unreadable path (bounded at 20 — the `reconcile.rs:293` precedent). The tally replaces the `COUNT(*)` at `:10610`, which is identical whether the walk indexed 7,800 notes or zero.
3. **D3.** The three best-effort calls accumulate into a `MaintenanceOutcome`. They stay best-effort — a term-index delta failure must never fail a save — but failures are counted.

**IPC decision, stated `[revised: the draft left it ambiguous while deferring the doc to §14.]`** `constellation_search_reindex` (`search.rs:10676-10680`) keeps its `Result<(), String>` shape via `.map(|_| ())`; the count is consumed Rust-side by the runner. **The `invoke` contract does not change, so `docs/IPC-CONTRACT.md` needs no edit in this commit.** Verified all 15 Rust call sites of `reindex_single_note` (`bases.rs:411`; `libraries.rs:1140/1434/1586/2249/2363/3477/6356`; `reconcile.rs:209/287`; `search.rs:11240/11377`; tests `search.rs:14131/14150/14168`) are `.is_ok()`, `let _ =`, `Ok(_) =>` or `.unwrap()` — all source-compatible.

**VERIFICATION CLAUSE** A normal note save still reindexes and search finds the new text within a second. `reconcile::maybe_schedule` still relocates/re-adopts/removes and still logs its triple. `reindex_library` still returns a file count. No `reindex_single_note` caller fails a save on a maintenance error. New Rust tests: a note deleted between gate and transaction leaves its row untouched and increments `failed`; an unreadable subdirectory increments `dirs_unreadable` and does not abort the walk. Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** Add a library containing one folder Windows won't let Constellation read. The rest of the library still indexes, and the app tells you one folder was skipped — rather than silently indexing less.

**Risk discharged:** D3, D4 (narrowed, with the residual named), D5; Architect §8.2; Invariant 8.

---

## §4 — The tag-count rebuild stops freezing the app

`[revised: split out of the draft's single §4, and the mechanism changed — the draft's scratch-and-swap silently discarded live saves' tag deltas.]`

**Why the draft's mechanism was wrong.** `tag_counts` is maintained write-time by a **± delta applied inside `index_note`'s own transaction** (`search.rs:6931-6935`, gated on `crate::tag_counts::is_stamped(conn)` captured at `search.rs:6756`). Today's `DELETE` + one `INSERT … json_each … GROUP BY` (`tag_counts.rs:58-67`) is atomic under the writer lock, so **no delta can interleave**. A windowed scratch-build + swap opens exactly that window: a delta landing on the live table after its rows were scratched is discarded at the swap. The draft's own verification clause ("byte-identical") and its own Boss test ("keep typing the whole time") could not both hold.

**Change — attack the cost, keep the atomicity.** The 13,040 ms is a full scan of `note_meta` dragging body_text off a 2 GB file for a query that needs only `tags_json`. Add a **covering index** `idx_note_meta_tags ON note_meta(path, tags_json)` (verified absent: the existing indexes are `search.rs:4180-4183`, `:4220`, `:4229`, none covering `tags_json`; the payload is ~50 bytes/note ≈ 400 KB) so the `GROUP BY` is satisfied from the index without touching the table. **One transaction, unchanged semantics, no delta window at all.**

**Gate, stated up front:** the planner must actually choose it. `EXPLAIN QUERY PLAN` on the measurement copy must show the covering index. **If it does not**, the fallback is the windowed scratch-and-swap **with the delta window closed explicitly**: build the scratch in 500-row `path >` windows, then in ONE short swap transaction re-apply the notes whose `modified >= run_start` — cheap and index-backed, because `idx_note_modified ON note_meta(modified)` already exists (`search.rs:4181`). The plan does not guess which; §M4 decides.

**VERIFICATION CLAUSE** `SELECT tag, n FROM tag_counts ORDER BY tag` is **byte-identical** before and after on an idle DB. Under the concurrent Boss test, every tag's `n` equals a fresh recount after quiescence. Longest single writer-lock hold **< 1 s**, down from 13.2 s (§M4). The tag browser shows the same counts. Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** Add a library, and keep typing in an open note the whole time it indexes. The window must never freeze and the note must save.

**Risk discharged:** R7 (half); the reproduction record's 13.2 s hold.

---

## §5 — The review rebuild stops holding 260 MB and 20 seconds

`[revised: split out; and the draft silently dropped the fact that an interrupted review rebuild has no heal anywhere.]`

**Files** `review.rs:1356-1377` — orphan `DELETE … NOT IN` (measured 2,747 ms) then a `Vec` materialising `(path, tags_json, modified, body_text)` for **every** row (17,886 ms, 260 MB resident) before the `backfill_one` loop. Pattern to copy: `links_backfill.rs:264-404` (500-row `path >` windows, `is_busy_error` retry ×8 at 400 ms), which exists because a whole-table UPDATE *"silently failed under boot DB contention — the 2026-05-30 overnight blank"* (`links_backfill.rs:259-263`).

**Change**
1. Stream in 500-row `path >` windows; `backfill_one` per window inside that window's transaction. `backfill_one` is idempotent per path, so windowing is semantically safe. Resident memory drops from 260 MB to ~500 bodies (~17 MB).
2. The orphan sweep is windowed by the same path range with its `NOT IN (SELECT path FROM note_meta)` subquery **unchanged**, so semantics are preserved. `[revised: the draft proposed leaving it as one statement; at 2,747 ms that is still 2.7 s of writer-lock hold inside a step whose whole purpose is bounding holds.]`
3. **A named heal for the interrupt `[revised: silently dropped by the draft.]`** The crash marker at `search.rs:4859-4887` heals **three link families only** — `tag_counts` and `review` have no boot heal at all, so a 5 s close-cap expiry mid-stream leaves `review_schedule` partly recomputed, undetectable (§9's drift check stats `.md` files and is blind to it). This step adds a **run-owned tail marker** in `schema_versions` — the same table `defrag_last_attempt` uses (`search.rs:2131`) — recording the pending family set before the tail passes and clearing it after. §6's boot entry point reads it and extends the heal from three families to five, using the mechanism that already exists rather than inventing one.

**VERIFICATION CLAUSE** `review_schedule` dump identical before/after on an idle DB; the Review Pulse lists the same notes, due dates and snooze states. Longest writer-lock hold **< 1 s**. New Rust test: a run aborted mid-window leaves the tail marker set, and the boot entry point re-runs the family. A save during a running recompute **succeeds** rather than timing out on `state.db`'s 5 s (`search.rs:3634`) against the walk's 30 s (`search.rs:10496`). Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** With a repair running (reachable via add-library), close Constellation. Reopen: the Review panel must be complete and correct, not half-rebuilt.

**Risk discharged:** R7 (second half); the "app closed mid-walk" danger, now for all five families.

---

## §6 — One place where derived views are rebuilt

`[revised: the draft's compiler token was decoration — nothing stopped a sixth assembly calling the five underlying functions directly, which is what all five current assemblies do.]`

**The five assemblies being replaced**

| Site | Passes |
|---|---|
| `search.rs:10535-10606` (`reconcile_filesystem` tail) | 5 — outgoing, incoming, sky, tag_counts, review |
| `search.rs:4859-4887` (`init_db` crash-marker healer) | 3 — outgoing, incoming, sky |
| `mig108.rs:1199-1200` (create, then recompute) | 1 — outgoing |
| `search.rs:1889-1902` (`on_link_vocabulary_changed`) | 2 — schedules both backfills |
| `incoming_links_backfill.rs:151` | 1 — `recompute_all_incoming` (verified: the fifth production caller) |

**Change** New `src-tauri/src/converge.rs`:
- `pub struct ConvergeKey(());` — the unit field is private to the module, so no code outside can construct one.
- `converge_derived_views(conn, key, families, ctx) -> ConvergeReport` — the ONE body. `Ctx` carries the `.constellation` dir and today-string only some callers have (`search.rs:10596-10598`).
- Five named entry points constructing the key internally: `after_repair_run`, `after_interrupted_walk_at_boot` (now 5 families, per §5), `after_mig108`, `after_vocabulary_change`, `after_incoming_backfill`.
- **The visibility narrowing that makes the token real `[revised]`.** Today `tag_counts::recompute_all_in` is `pub(crate)` (`tag_counts.rs:57`), `review::recompute_all_in` is `pub` (`review.rs:1356`), and `links_backfill::recompute_all_{outgoing,incoming,sky}` are `pub(crate)` (`:264`, `:317`, `:371`). All five become reachable **only** from `converge` (module-scoped visibility), and `incoming_links_backfill.rs:151` — a genuine cross-module caller — re-points to `converge::after_incoming_backfill`.
- `ConvergeReport` is **generated** from outcomes: per family `Converged(n) | Skipped(reason) | Failed(msg)`. The three stamp gates (`:10554`, `:10578`, `:10595`) become `Skipped(NotStamped)` — visible, not invisible.

**VERIFICATION CLAUSE** All five previous call sites behave identically: boot after an interrupted walk heals and clears the marker only when every family succeeded (`search.rs:4890-4894`); MIG-108 still recreates triggers **before** recomputing (`mig108.rs:1199-1200`) and passes its in-transaction aggregate verify (`:1204-1210`); a vocabulary edit still refreshes both aggregates; the incoming backfill still stamps only after success. **The token is proven by a compile-fail fixture** (`trybuild`, or a documented "revert this one line and observe E0603") — not by the sentence "cannot compile." `[revised]` New test: a `Families` selection including an unstamped family yields `Skipped(NotStamped)`, never `Converged(0)`. Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** Add or rename a link type in the link-type settings; the Sky View and backlink type badges must still update exactly as they do today.

**Risk discharged:** R4's one-body + compiler-token requirement; "partial repair advertised as whole."

---

## §7 — Only one thing may walk the library, and only inside its own universe

`[revised: MERGES the draft's §6 + §7 + §8. The draft split them, and each split produced a half-migrated commit — the project's own dominant defect shape.]`

**Why the merge is mandatory.** (a) The draft's §6 delivered its guarantee by forward-referencing a "run mutex" §7 introduced and a "run id" §7 introduced — neither existed at §6. (b) The draft's §7 routed only `constellation_search_init` and deferred `reindex_library` to §8, leaving one commit in which the app has a single-flight-guarded walker **and an unguarded independent one** — `reindex_library` (`libraries.rs:3434`) is not a wrapper over `reconcile_filesystem`; it is `collect_md_paths` (`:3462`) + per-file `reindex_single_note` (`:3477`). §7's clause asserting "Invariant 3: one index job process-wide" was **false at §7**. This is the largest commit in the plan and it is the smallest diff that is not half-migrated.

**Files** New `src-tauri/src/index_repair.rs`, modelled on `classifier/scan_job.rs` (`ScanState` `:41-47`; start guard `if state.running.swap(true,..)` `:122`; `ensure_search_db_ready` **inside** the worker `:133-135`; `INTER_NOTE_SLEEP_MS = 30` `:38`) · `search.rs:10468` (`reconcile_filesystem` loses `pub`, becomes module-private to the runner) · `search.rs:10505-10538` (the marker/drop/recreate sequence becomes `TriggerWindow`) · `search.rs:10639` (`constellation_search_init` → thin submit) · `libraries.rs:3434-3485` (absorbed) · `lib.rs:711-774` (close handler) · `mig108.rs:39/43/50-59` (flag, checker, RAII precedent) · `search.rs:775` (`federation_generation`, bumped at `:9765-9766` inside `invalidate_search_state`, and **never read anywhere in `10468-10626` today**).

**Change**

1. **`TriggerWindow`** — RAII, on the runner's own connection. `new()` persists the marker **before** the drop (preserving the discipline at `search.rs:10499-10505`: *"If the marker cannot be persisted, do NOT enter the unprotected window"*) then drops the three families. `Drop` recreates the outgoing triggers **from the then-current `link_types::snapshot()`** and clears the marker on every exit path including `?` and panic — preserving today's "recreate BEFORE the recompute so any concurrent live save is trigger-covered" ordering (`search.rs:10529-10531`). **No creation site is gated** (H1 above). The marker is run-owned so a second run cannot clear a first run's.
   **The sky edge-mirror is an explicit accept, and it is the status quo, not a new choice `[revised: the draft framed it as a live decision]`.** `drop_sky_aggregate_triggers` (`search.rs:1856-1866`) drops only `note_links_sky_stratum_*` / `_maturity_*` — **not** `note_links_sky_ai/_ad/_au` (`search.rs:4483-4530`). The mirror already stays armed through today's walk. Dropping it would be worse: `recompute_all_sky` only `UPDATE`s `sky_nodes` columns (`links_backfill.rs:371`), so **nothing in the codebase rebuilds `sky_links`**. Bound: `index_note`'s diff-edges leaves unchanged edges untouched (`search.rs:7126-7133`), so the mirror fires only for edges that actually changed — ~60 notes' worth in catch-up. For full re-read this is **unmeasured**, which is why §14 stays flag-gated.

2. **Single-flight with a typed outcome**, not a bare `Err`: `Started(run_id) | Queued(run_id) | AlreadyRunning(run_id) | Blocked(reason)`. Every existing caller swallows errors — `store.ts:3600` `.catch(console.error)`, `store.ts:3612` `.catch(() => {})`, `+layout.svelte:2860` `.catch(() => 0)` — so the typed outcome is what makes §11's report truthful.

3. **Queued, never refused** (Invariant 4). A submit whose scope the running job does not cover records a pending scope; the runner merges it into its residual or auto-starts a follow-up.

4. **`reindex_library` absorbed as `Scope::ColdStart`**, preserving its `COUNT(*)` gate verbatim (`libraries.rs:3446-3457` — that gate is what honours ZERO-BOOT-WALKS / LL-022) and its per-file `library_name_for_path` attribution (`libraries.rs:3466-3472`, the 2026-07-25 Whole-Ecosystem fix). **ColdStart keeps `reindex_single_note` as its per-note primitive, not the walker's bare `index_note` `[revised: silently changed by the draft].`** `libraries.rs:3475-3477` records why in terms: *"reindex_single_note wraps index_note AND runs the MIG-079 §C.2a incoming-aggregate diff post-commit — so a cold-started library's TARGET notes get correct backlink (incoming_count) values, not just outgoing."* Substituting the bare `index_note` would lose that, and the replacement tail is stamp-gated — on an unbuilt aggregate §6 yields `Skipped(NotStamped)`, correctly reported and still absent.

5. **All THREE `reindex_library` call sites re-pointed `[revised: the draft named one].`** `store.ts:3600` (addLibrary), `store.ts:3612` (bringInLibrary), `+layout.svelte:2860` (boot fan-out). Plus the **double-fire**: `handleBringInChoice` (`+layout.svelte:5979-5986`) calls `bringInLibrary()` — which fires `reindex_library` at `store.ts:3612` — **and then** calls `initSearchIndex()`. One user action fires both walkers today; under single-flight the second would be refused into `store.ts:3612`'s bare `.catch(() => {})`. It becomes one submit.

6. **Boot fan-out collapsed to ONE request.** `+layout.svelte:2859-2863` is N parallel invokes with `.catch(() => 0)`; under single-flight that is 1 `Started` and N−1 refusals rendered as "0 notes" — silently re-opening the LL-027 / BUG-022 cold-start gap for every library but the first. One submit, one outcome, one `loadAllStats()` / `refreshLibraryCaches()` follow-up.

7. **Universe-switch safety.** Capture `db_path` (`:10472`) **and** `federation_generation` at start; re-read the generation before each window; a mismatch aborts cleanly as `Cancelled(UniverseSwitched)` writing nothing further. Precedent: the capture-first discipline at `search.rs:10053-10055`.

8. **MIG-108 mutual exclusion**, both directions (`mig108::engine_is_running()`, `mig108.rs:43`). Closes the collision where MIG-108's finish calls `init_db` (`mig108.rs:2181`) whose marker block (`search.rs:4859`) would recompute and clear a running walk's marker. **The same `init_db` reachability is why the marker is run-owned** — `ensure_search_db_ready` → `init_db` is reachable whenever `db_ready` is false (`search.rs:9998`), which `invalidate_search_state` sets on every switch (`search.rs:9770`).

9. **Defrag mutual exclusion, both directions `[revised: not discharged in the draft].`** The runner refuses to start while the defrag worker holds `MIGRATION_ACTIVE`, and `maybe_schedule_defrag` (`search.rs:2124-2157`) refuses to spawn while a repair is running — its existing 10-minute `defrag_last_attempt` cooldown (`search.rs:2112-2114`) is not a mutex. The runner calls `maybe_schedule_defrag` **explicitly after** the run per the standing rule (`search.rs:2032-2057`); §7.2's auto-started follow-up run consults the same exclusion so it cannot start into a VACUUM.

10. **Close handshake with a stated budget `[revised: the draft's wording implied 10 s].`** `lib.rs:735-744` records the Boss ruling — *"up to 5s, instant when clean."* The repair-cancel wait **shares** that single 5 s budget with the existing `session:final-flush` handshake; it does not add a second one. Cancel lands on a window boundary (§4/§5 make windows short enough for that to be cheap), and §5's tail marker covers an expiry mid-stream.

11. **H2** — `MIGRATION_ACTIVE` not taken; `wal_checkpoint(PASSIVE)` every 500 notes on the runner's own connection.

**VERIFICATION CLAUSE** All four existing doors still work and now route through the runner: add a library (`+layout.svelte:4690`), link an external library (`:5971`), bring a library in (`:5984`), and empty-index auto-recovery (`:2891`). **LL-027/BUG-022 regression test:** seed a universe with three unindexed libraries, boot, assert all three have `note_meta` rows — not just the first. An already-indexed universe still does **zero** filesystem walking at boot (walk tally 0; boot time unchanged, §M3). New Rust tests: the start guard admits one of two racing submits; a generation bump mid-run aborts without writing; `Blocked(Mig108Running)` and `Blocked(DefragRunning)` fire; a `TriggerWindow` dropped on a panic recreates the triggers (asserted via `sqlite_master`); a second run cannot clear a first run's marker. Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** With Constellation closed, copy new notes into two different libraries at once. Launch: both libraries' new notes must be searchable. Then, while a library is indexing, try adding a second — it must be accepted and processed, not silently ignored; and closing the app during indexing must close within about five seconds, neither hanging nor cutting the work dead.

**Risk discharged:** Invariants 3, 4, 10; concurrent runs; crash-marker clobber (including the `init_db` actor); universe switch; MIG-108 collision; **defrag collision**; closed-mid-walk; H1; H2; R4's runner + `pub` removal + boot fan-out fix; LL-027/BUG-022.

---

## §8 — The index stops adopting notes that belong to a linked universe

`[revised: the draft scoped two passes; four surfaces write foreign rows, and one of them is the very cold-start path the draft's own §8 built. It also dropped the Architect's explicitly-handed read-succeeded caveat.]`

**All four surfaces of one concern** (Whole-Ecosystem Fix Law):

| Surface | Today | Why it matters |
|---|---|---|
| `search.rs:10515` — the walk's roots | `load_all_libraries` (recursive) | writes foreign rows |
| `reconcile.rs:89` — boot reconcile roots | same recursive call; `reconcile.rs:287` re-adopts orphans | re-adopts next launch |
| `libraries.rs:3435-3437` — `reindex_library`'s validation + `library_name_for_path` attribution | recursive set | **after removal its `COUNT(*)` gate reads zero, so the very next cold start re-walks the foreign library** |
| `search.rs:11337` — `reindex_changed_paths` (the watcher) | recursive set; `:11375-11378` indexes any path a federated library owns | re-adopts **immediately**, no boot required |

**Change** All four take their roots from the **own** (non-recursive) set — `universe::own_libraries_for_root` (`universe.rs:1479`), documented at `:1471-1478` as *"NON-recursive — deliberately WITHOUT the federated cUniverse libraries."*

Plus **two things the draft dropped**:

- **Read-succeeded discipline `[revised: silently dropped].`** `universe.rs:1481-1484` is `match fs::read_to_string { Ok(data) => serde_json::from_str(&data).unwrap_or_default(), Err(_) => vec![] }` — an unreadable or corrupt `libraries.json` yields an **empty list**. The Architect handed this to Phase 2 in terms (`PJ-207-REPRODUCTION-2026-08-03.md:204-208`): for a repair it means *"walk nothing and report success"* — the "couldn't read it → you have none" class PJ-200 closed elsewhere. The runner uses a `Result`-returning variant; a failed read is `Failed(LibrariesUnreadable)`, **never** an empty walk reported as `Converged(0)`.
- **Nested-foreign-root exclusion `[revised: the draft's precondition assumed no federated root sits under an own root].`** `collect_md` (`reconcile.rs:378-421`) recurses through everything not starting with `.`, and `universe_notes` has `path == Universe root`. If any cUniverse directory lives **inside** the active Universe root, its `.md` files are orphans under an own root and get re-adopted at `reconcile.rs:287` — R6's oscillation, restored. The mechanism to prevent it already exists and is already used by the walk: `crate::libraries::nested_library_paths(&libraries, &lib.path)` (`search.rs:10517`) and the `exclude` set. Both passes take a foreign-root exclusion set. **I do not know whether any child universe is nested under the Boss's live root** — I would read `E:\Constellation Universes\Eisa Cognitive Knowledge\universe.json`'s `children` array during Build. The exclusion is correct either way.

**One thing the draft asserted but never proved, and it does hold `[false-positive rebuttal to the fear it invites]`:** narrowing `reconcile.rs`'s roots does **not** turn the capped sweep into a mass delete of foreign rows. The load-bearing line is `reconcile.rs:130-133` — `if !roots_norm.iter().any(|r| under(&pn, r)) { continue; }` — a row outside every root is **skipped**, never a stale candidate. Cite it, or every reviewer rediscovers it and one concludes this step silently deletes up to 200 rows per boot.

**Nothing is removed here.** The runner counts and reports: `SELECT COUNT(*) FROM note_meta WHERE library_name NOT IN (<own set>)`.

**Foreign notes stay searchable** via the MIG-056 federated scatter-gather path — `federated_lexical_search_or_fallback` (`search.rs:7583`, called at `:11461` and `:11537`) over the ATTACHed `state.federated_conn` (`search.rs:760`), independent of the local duplicate rows.

**VERIFICATION CLAUSE** A universe with **zero** cUniverses behaves byte-identically — assert `own set == recursive set` in a test, since that is the common case and the one that must not regress. With a cUniverse: search still returns foreign notes; the boot reconcile no longer re-adopts foreign orphans; the watcher no longer indexes them; a cold start no longer walks a foreign library. New test: **a `.md` under a federated root nested inside the active root is NOT re-adopted.** New test: an unreadable `libraries.json` yields `Failed`, not `Converged(0)`. Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** In a universe that links a child universe, search for a word that exists only in the child's notes — it must still be found.

**Risk discharged:** Charter W2-9 (first half); R6's stated precondition, now on all four paths; Architect §8.1; the reproduction record's `:204-208` caveat; Invariant 10.

---

## §9 — Constellation notices, after it opens, that notes changed while it was closed

`[revised: carries its own 15 locale strings — the draft put every string in §14, which cannot land i18n-green.]`

**Why:** `scripts/i18n-parity.mjs:20-22` states the contract — *"Ordinary keys — governed by the UNION of all locales. Every locale must carry every key any locale carries."* Exit 1 on drift, and `tests/i18n/locale-parity.test.ts` imports the same helpers *"so the script and the test can never disagree."* An `en`-only key at §9 → 14 locales short → parity exit 1 → vitest red. **Plan-wide rule, not a §11 line item: every step that renders a string carries its own keys ×15 in the same commit.**

**Files** New command in `index_repair.rs`; called post-paint from `+layout.svelte` alongside the existing `setTimeout(… cache_mark_search_ready …, 800)` at `:2905` · gated on `REPAIR_DOOR_ENABLED` (defined §2) · `lab/boot-perf/BOOT-BUDGET.md:101` (*"Criterion 4 — cheap stat-only post-UI sweep… Still not implemented"*, specified 2026-04-15) · new `settings.index.drift.*` keys ×15.

**Change** A read-only pass that stats every `.md` under the **own** roots (§8) and compares against `note_meta.modified`. **No file bytes read. No row written.** Measured 160–590 ms on the 7,824-note universe. Returns `{ drifted, missing_on_disk, foreign_rows }`; the frontend surfaces a notice **only when drift exists** — never a green "all clear" banner, which would be noise.

**VERIFICATION CLAUSE** Boot stays walk-free (Invariant 6): assert **zero** `read_to_string` calls and **zero** writes, and boot-to-interactive unchanged within noise (§M3). On the reproduction universe the check reports **60**. On a zero-drift universe no notice appears. `node scripts/i18n-parity.mjs` **15/15** and `npx vitest run tests/i18n/locale-parity.test.ts` green. Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** Close Constellation, edit a note in Notepad, relaunch — a notice must appear saying some notes changed while Constellation was closed.

**Risk discharged:** R3; Criterion 4; Invariant 6.

---

## §10 — One progress strip instead of three copies

`[revised: + two component tests; the extraction had zero automated coverage and three consumers.]`

**Files** `ClassifierScanProgressStrip.svelte` and `NscBackfillProgressStrip.svelte` — verified **both exactly 159 lines** and byte-equivalent modulo identifiers: same `Phase` union, `visible/phase/total/completed/cancelling` state, `handleEvent`, recover-on-mount `invoke(status)`, 4 s linger, `role="status" aria-live="polite"`, `onDestroy` cleanup, CSS block. Mounts: `+layout.svelte:10075-10077` (`.sb-center`) and `CatalogerView.svelte:304`. `MigrationProgressStrip.svelte` is 100 lines with a different event contract and stays.

**Change** Extract `JobProgressStrip.svelte` parameterised by `{ eventName, statusCommand, cancelCommand, labels }`. Re-point Classifier and Nsc; the repair uses it too. **Net strip components 3 → 2, three consumers, one implementation.** This is the one place Design B risked doing exactly what the half-sweep lens punishes — adding a fourth clone. It does not.

**VERIFICATION CLAUSE** Both existing strips still appear, count, cancel, linger 4 s, and recover on mount after navigating away and back, in **both** mount locations. RTL still correct (`margin-inline-start`, `ClassifierScanProgressStrip.svelte:155`). **Two new component tests** — recover-on-mount (the `invoke(statusCommand)` path) and the 4 s linger — because verified: no file under `tests/` mentions `ProgressStrip`, and after extraction one regression breaks all three consumers at once.

**BOSS-TESTABLE — yes.** Start a classification scan from Settings → Intelligence, watch the strip count, press Cancel — it must behave exactly as before.

**Risk discharged:** the half-sweep judge's single named objection to Design B; the "never copy-paste and adapt" standing rule.

---

## §11 — THE DOOR: Repair index in Settings, Repair now on the warning bar, and every surface refreshes

`[revised: MERGES the draft's §13 (freshness wiring) with §14a (the control). The draft's §13 verification clause and Boss test both said "after a repair" — there was no repair at §13; the only reachable trigger was library-add. §13 has no independent user value; its whole point is that the door's result is visible.]`

**Files — code**
- `SettingsModal.svelte:2030-2090` (the Index section body; the new controls follow the `.setting-item` + `.setting-btn` shape at `:2071-2089` and the classifier's `Start scan` shape at `:1766-1783`)
- `SettingsModal.svelte:101-107` + mount `:2604-2617` — the localised `confirmDialog`. **`window.confirm` is forbidden** (`:96-99`: *"Replaces browser-native confirm() which forces OS-locale OK/Cancel labels"*).
- `+layout.svelte:550-559` (`storeHealthError`) and `:7442-7446` (its render) — gains a **"Repair now"** action; `tOr()` (`:566-569`) is the explicit-English-fallback helper these notices already use.
- `+layout.svelte:1082-1096` (`ensureSky()`, memoised via `_skyPromise`, already has `force` and `_skyGeneration`), `:5867+` (`stageMap`/`maturityMap`, populated only on library expand), `:5838-5856` (the Index panel gate keyed on `` `${activeUniverseName}|${$libraries.length}` `` — blind to a repair, since neither changes).
- `SecondScreenPage.svelte` — gains the `index-repair:done` listener as a **display**, not a domain: it re-reads what the main window's core refreshed. No save/load logic added.
- `store.ts:3714-3738` — **D2.** `indexHealthError` is set at `:3735` and cleared **only** at `:3728` under `if (attempt > 0)`, so it clears only when attempt 0 threw *and* the retry succeeded. Once set, the red bar is permanent for the session. It now clears on any successful reindex and on a zero-failure repair — and **stays** on a repair reporting ≥1 failure (Invariant 8).

**Files — strings and the concept paper, in this same commit**

| Surface | Change |
|---|---|
| `en.json:4246` `storeHealth.index` + 14 locales | stops naming a control that did not exist; names the one that now does |
| new `settings.index.repair.*` ×15 | control, description, confirmation, report |
| `docs/concept-papers/29-settings.md` §3, §7, §9, §10 | the **verbatim** amendment from Architect §7 (`docs/PJ-207-Index-Repair-Architect.md:113-131`) — reused word-for-word, not reinvented. It is the *justification* for the control and belongs with it. |

**Change** Two controls in Settings → Index — **"Repair index"** (default; catch-up, mtime-gated; no confirmation needed — it writes nothing a note did not already say) and **"Full re-read"** (separately confirmed, **flag-off** until §M1 gives the dialog a real number, R5). Plus **"Repair now"** on the health alert bar. Both submit to the one runner and render its `ConvergeReport` verbatim — `converged(n)` / `skipped(reason)` / `failed(msg)` per family — so a stamp-gated skip can never render as a whole repair. On `index-repair:done`: `ensureSky(true)`, `indexLoadedKey` invalidated, `stageMap`/`maturityMap` refreshed for expanded libraries, `loadAllStats()` / `refreshLibraryCaches()`.

**VERIFICATION CLAUSE** `node scripts/i18n-parity.mjs` **15/15**; `npx vitest run tests/i18n/locale-parity.test.ts` green — verified today's contract: an `en` key missing anywhere renders as a raw dotted key in **all 15** languages (`i18n-parity.mjs:11-13`). Every locale's `storeHealth.index` names a control that exists (Invariant 9). Settings → Index renders correctly in RTL. The confirmation is the localised dialog, never `window.confirm`. The health-bar action pressed twice quickly produces **one** run. **D2:** force a failed reindex (make `search.db` read-only for one save) → red bar appears; a successful repair clears it; a repair reporting ≥1 failure leaves it. Sky View, Index panel, file-tree stage emoji, sidebar counts and the second screen all reflect repaired data **without a restart**. No `$effect` loop — temporary instrumentation shows the listener firing ≤2 times per repair. Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** Close Constellation. Add a distinctive word to a note in Notepad. Relaunch — you see the notice. Press **Repair now**, watch the progress strip, then search for that word and find it. Open Sky View and the Index panel **without restarting** and confirm the repaired notes are there. Then switch the app language to Arabic and confirm the whole control reads correctly right-to-left.

**Risk discharged:** R1, R2; D2; Invariants 8, 9; post-repair UI staleness; the second-screen gap; the reproduction record's named recipe (§2) end-to-end.

---

## §12 — The documentation stops promising a button that never existed

`[revised: split out of the draft's §14, which was ~50 files across code, i18n and docs AND the only Boss-testable commit in the plan. The split is for reviewability — both land the same day — so a doc typo cannot block the door.]`

**Files** (all verified this session)

| File:line | Currently says | Becomes |
|---|---|---|
| `docs/help.uConstellation.World/Index/Index.md:85` | *"There's nothing to 'rebuild' — the moment you save a note, its terms are reflected in the Index."* | true **while Constellation is running**; names reconciliation for the closed interval |
| `docs/User Manual.md:952` | *"no embedding-build phase, no per-library training, no 'Rebuild' button anywhere"* | **scoped to the M11 semantic layer it is actually about** — that sentence sits in the semantic-search section and stays true of embeddings; it must not read as a claim about the index |
| `docs/help.ar/User Manual.md:827` + the 13 other locales | the same sentence (*"لا زر «إعادة بناء» في أيّ مكان"*) | same scoping |
| `docs/help.uConstellation.World/Universe/Universe.md:104` | *"File watchers and index caches rebuild in the background"* | accurate |
| `README.md:91` | *"caches rebuild in the background"* | same |
| `docs/IPC-CONTRACT.md` "Search & Index" table | — | add the repair commands with their one-shot/debounce contract; remove `cache_reconcile` |
| `+layout.svelte:2800`, `:2903`, `:4203` | three **code comments** citing *"Settings → Rebuild Index"* | corrected — `[revised: not in either critique; found this session]` |

**DO NOT sweep** (verified correct and still true): `docs/User Manual.md:1818` ×15 and the Arabic `Engine.md:52` ×15 — both scoped to the targeted Arabic-override reindex, which genuinely is targeted and genuinely is not a full rebuild.

**VERIFICATION CLAUSE** `grep -rn "Rebuild Index" docs/ src/ src-tauri/src/` returns only intentional history. No doc still promises a control that does not exist. Docs-only commit — trivially green, reviewable alone. Exempt from the per-build inspection (no write/index/lifecycle path).

**BOSS-TESTABLE — no** (documentation only).

---

## §13 — Offer to remove duplicated copies of linked-universe notes

`[revised: MOVED from the draft's position §10 (before the door) to after it. The draft's §10 required "the report surface built in §13" and a confirmation dialog that did not exist until §14 — and it broke the plan's own ordering law by adding the most user-reachable route in the entire plan, a confirmation-gated deletion, in the "no user-reachable route" block.]`

**Files** `index_repair.rs` (`Scope::RemoveForeignCopies`) · the report surface from §11 · `reconcile.rs:57-60` — the existing capped, archive-first sweep whose caps (`MAX_STALE_FRACTION = 0.10`, `MAX_STALE_ABSOLUTE = 200`) this mode reuses rather than inventing new ones · its own `confirmDialog`, **separate** from the repair confirmation.

**Preconditions, all non-negotiable**
- Runs only after §8, so no pass re-adopts. **§10's oscillation test is now falsifiable-proof**: §8 scopes `reindex_library` and the watcher too, which the draft's §8 would have re-adopted through (its `COUNT(*)` gate reads zero after removal, so the next cold start walks the foreign library).
- **A federated-coverage precondition `[revised: new — neither critique caught it, and it is the difference between "still searchable" and data loss].`** "Foreign notes stay searchable" holds only if the child universe's **own** `search.db` actually contains them — `federated_lexical_search_or_fallback` merges results from the ATTACHed schemas (`search.rs:7583-7600`), and a child universe never opened as an active universe may have an empty or absent index. Removal is offered **only** when federation is attached (`state.federation.lock()` reports `is_ready() && !attached().is_empty()`, `search.rs:7590-7595`) **and** a per-alias `COUNT(*)` confirms the foreign paths are present there. Otherwise the report says so and offers nothing.
- Uses only the existing capped archive-first path — Invariant 7 forbids any other delete.
- **Does not touch the Earned-Life Ledger.** It lives in the active universe's `.constellation` dir, derived from `conn.path()`'s parent (`link_life.rs:78-84`), keyed by cid-pair (`:247`). Removing `note_links` rows leaves entries intact and orphaned; `link_life_restore` re-folds them if rows return. **Unknown, to be read during Build before this ships:** whether `link_life_restore`'s row `UPDATE` (`link_life_restore.rs:409`) treats a missing target row as a no-op or a failure. I would read `link_life_restore.rs:380-420`; if it counts a miss as a failure, this step adds a "row absent" outcome rather than changing the removal. It affects report accuracy only, not data safety.

**VERIFICATION CLAUSE** After a removal: the foreign notes are **still findable in search**; the child universe's files on disk are **byte-untouched** (Invariant 1 — hash the child tree before and after); **two consecutive restarts leave the count at zero** (the oscillation test, now unfalsifiable by any other surface); the ledger file's line count is unchanged. Diff-scoped `safety-inspection`.

**BOSS-TESTABLE — yes.** Run the repair in a universe with a child universe. Read the "N notes from linked universes are duplicated here" report. Confirm removal. Restart twice — they must not come back, and they must still be findable in search.

**Risk discharged:** R6 (second half); Charter W2-9 (closable); Invariants 1, 7.

---

## §14 — Full re-read: built, measured, still switched off

`[revised: + dead-code gating. Verified this session: `src-tauri/Cargo.toml` has no `[lints]` section and neither `lib.rs` nor `main.rs` carries a crate-level `#![deny(...)]` — so an unconstructed enum variant is a **warning**, not a build failure. The critique flagged this as unchecked; it is checked, and downgraded, but still gated.]`

**Files** `index_repair.rs` `Scope::FullReread` → `index_note(..., force = true)` · `FULL_REREAD_ENABLED` (defined §2) · `SettingsModal.svelte` confirmation copy that must state a **measured** number.

**Change** Built and exercised **only** by the measurement harness (§M1) against a **byte copy** of the real universe, per the reproduction record's own discipline (`PJ-207-REPRODUCTION-2026-08-03.md:141-144`). The flag stays off until the number exists. The runtime `match` references the variant behind the `const false` branch, so the variant is constructed-in-code and produces no `dead_code` warning.

**One fact that materially bounds the cost, to be confirmed by measurement rather than assumed:** `note_meta_au` is guarded `WHEN OLD.name IS NOT NEW.name OR OLD.body_text IS NOT NEW.body_text` (read from the live schema, reproduction record `:51-52`), so an unchanged note in a full re-read does **not** rewrite its FTS row. The `note_meta` UPSERT and the `note_body` dual-write still happen for all 7,824, and the sky edge-mirror (§7) stays armed. Measured floor for I/O alone: **49.0 s** (`:180-182`). Whether the whole thing is a minute or tens of minutes is **unknown** and is exactly what §M1 measures.

**VERIFICATION CLAUSE** With the flag off, the shipped binary is inert with respect to full re-read: no control renders, no command path reaches `force = true`; `cargo build --release` produces no new warning. Catch-up behaves exactly as after §11. The harness run on the copy produces a duration, a peak WAL size and a peak RSS.

**BOSS-TESTABLE — no** (flag off; nothing user-visible ships).

**Risk discharged:** R5.

---

## §15 — Migration close

**Files** `docs/Constellation Orientation & Onboarding v3.83.md` (**NEW file** — v3.82 is current; never overwrite, SO#6) · `docs/Constellation Pending Jobs v1.67.md` (**NEW file** — verified v1.66 is the highest present; SO#9: close PJ-207, file what surfaced, re-rank, "► Next action") · `docs/Constellation-Safety-Audit-CHARTER.md:93` (**W2-9** → closed, evidence §8/§13) · `docs/LESSONS-LEARNED.md` · `lab/reports/SESSION-LOG-2026-08-03.md` · `docs/MoCh/`.

**Newly filed PJs this migration must open** (each is a real defect this plan names but does not fix, so none is "noted-and-shipped" — they are filed with a ruling, per WA#6):
- **Same-second mtime residual** (§3): `search.rs:6585-6591` documents that a save landing in the same second as the walk's stat is invisible. Closing it needs content hashing on the walk path — a write-path change, its own job.
- **Charter W2-14** — the save-path incoming diff keys on names only (`search.rs:1365`). The repair now heals it; the write-path fix is its own job.
- Whether any cUniverse root is nested under the active Universe root (§8's exclusion is correct either way, but the live answer belongs in the record).

**The Lessons-Learned entry** — *a repair pass with no door is indistinguishable from no repair pass at all, and a localised string that names a nonexistent control is a promise the app breaks in 15 languages.* `reconcile_filesystem` shipped 2026-04-08 reachable only from an empty-index gate; `storeHealth.index` named "Settings → Rebuild Index" in every locale while the only trace of that button was 13 orphan CSS rules and three code comments.

**The one item no commit can reach.** The 2026-05-04 memo `project_index_rebuild_button_decision.md` lives **outside the repo** at `C:\Users\ealsh\.claude\projects\E---------------Constellation\memory\`. It set its own reopen condition — *"Reopen if Boss flags index desync"* — which has fired and is measured (60/7,824). Mark it **overturned** as a separate manual checklist item at close. Flagged here because it is structurally invisible to a git-based process.

**Per-cycle safety inspection:** `Workflow({ name: 'safety-inspection' })` — whole-app; a `/migration` close *is* a cycle boundary. Every confirmed finding fixed before the cycle is declared closed.

**VERIFICATION CLAUSE** The orientation doc's §17 "not read in detail" list is honest. The PJ ledger's "► Next action" names the current top item. `grep -rn "Rebuild Index" docs/ src/ src-tauri/src/` returns only intentional history. `[revised: the draft's "git log shows 16 §N commits" is not a check of anything and is dropped.]`

**BOSS-TESTABLE — no.**

---

# ROLLBACK PLAN

**Level 1 — the flags** (both defined in §2, so every later step can gate on them). `REPAIR_DOOR_ENABLED = false` removes every new user route in one line — §9's notice, §11's control and health-bar action, §13's removal offer — leaving every shipped guard in place. `FULL_REREAD_ENABLED` gates §14 alone. **No user data shape changes, so there is nothing to migrate back.**

**Level 2 — revert by step, in reverse order.** Dependency constraints that make some reverts unsafe out of order:
- **§13 must be reverted before §8** — removal without scoping restores the oscillation.
- **§6 (`converge.rs`) cannot be reverted while §7 stands** — the runner calls it, and §5's tail marker is read by its boot entry point.
- **§11 must be reverted before §7** — the door submits to the runner.
- **§1 should never be reverted.** A strict correctness widen with no dependents; reverting re-opens the confidence/`created` loss that made it the mandatory first step.
- **§10 and §12 are independent** and revertible at any time.

**Level 3 — drift discovered in production.** The failure mode to plan for is a repair that leaves a family stale. Recovery already exists and is strengthened here: the run-owned crash marker (§7) heals the three link families at next boot, §5's tail marker extends that to `tag_counts` and `review`, and §9's drift check reports what remains. **A partly-repaired index is a correct state** — the next check finds the residual (the resumability contract at `SettingsModal.svelte:2525-2543`, Criterion 5).

**Level 4 — the catastrophic case.** `search.db` is the system of record for the earned half of the Living Link Architecture (`CLAUDE.md`; the ledger at `link_life.rs:73-84` is the disk mirror). **No step in this plan deletes `search.db`, renames it, or gates on a schema version.** The only delete anywhere is §13, capped and archive-first through `reconcile.rs:57-60`, and it does not touch the ledger. `link_life_restore` remains the backstop.

---

# DELIBERATELY NOT IN SCOPE

| Not doing | Reason |
|---|---|
| Dropping the sky edge-mirror triggers (`search.rs:4483-4530`) for the walk | Nothing rebuilds `sky_links` — `recompute_all_sky` only `UPDATE`s `sky_nodes` columns (`links_backfill.rs:371`). Dropping them loses Sky View edges permanently. Explicit accept (§7), and it is today's behaviour: `drop_sky_aggregate_triggers` (`search.rs:1856-1866`) never touched this family. |
| A persisted `repair_queue` table (the Architect's Option-C graft) | Design B was chosen over A; the runner's residual is re-derivable by re-stat (§9 is the cursor). A new table is additive state we would then have to keep coherent. Revisit only if §M1 shows full re-read long enough that mid-run resumption matters. |
| Charter W2-7 / W2-8 (lossy frontmatter parser, `store.ts:1179/1196`) | The `.md` write path — a different concern. This migration writes **no** `.md` file (Invariant 1). |
| Charter W2-13 (`write_gate.rs:420` staleness inert) | Unrelated subsystem. |
| `MigrationProgressStrip` consolidation into §10's shared strip | Different event contract (it listens for `migration:term_vocab_v2`, which the defrag worker also emits at `search.rs:2153`); consolidating widens the diff without reducing duplication of the cloned pair. |
| Flipping `FULL_REREAD_ENABLED` on | R5: the dialog must state a measured number. The flip is a follow-up commit after §M1. |
| A "rebuild from scratch" mode | Forbidden by Rule 8 and by the Architect's framing: the honest name is **reconciliation**. No drop-and-replay path exists anywhere in this plan. |
| PJ-144 (classifier reloads the neighbour set per note, `scan_job.rs:22-24`) | Named in the chassis we reuse; its own migration. |

---

# MEASUREMENTS I RUN MYSELF BEFORE ANY BOSS-FACING NUMBER IS QUOTED

**Protocol for all:** Constellation **closed**; a **byte copy** moved outside the universes tree — never the live database. The discipline the reproduction record already used (`PJ-207-REPRODUCTION-2026-08-03.md:19-20, 141-144`).

```bash
SRC="/e/Constellation Universes/Eisa Cognitive Knowledge/.constellation"
DST="$HOME/AppData/Local/Temp/claude/E---------------Constellation/pj207"
mkdir -p "$DST" && cp "$SRC/search.db" "$DST/search.db"
cp "$SRC/search.db-wal" "$DST/" 2>/dev/null; cp "$SRC/search.db-shm" "$DST/" 2>/dev/null
ls -l "$DST"      # expect 2,026,405,888 bytes
```

**§M1 — Full re-read duration** (gates R5's dialog number and §14's flag flip). `Scope::FullReread` against the copy via `#[test] #[ignore]` (`cargo test --release pj207_full_reread_measure -- --ignored --nocapture`). Record wall clock, notes re-read, FTS rows actually changed, peak WAL, peak RSS. Floor is 49.0 s of pure I/O. **Until this produces a number, no duration is stated to the Boss anywhere.**

**§M2 — Catch-up duration on the real drift.** Same harness, `Scope::CatchUp`; should touch the measured **60** rows. Confirm `SELECT COUNT(*) FROM note_meta;` → 7824 first; the per-row mtime comparison runs in the harness, not SQL (SQLite cannot stat).

**§M3 — Boot stays walk-free** (Invariant 6; gates §7 and §9). Three cold launches before and after, comparing the app's own `read_boot_perf_report` / `lab/boot-perf/` output. Assert boot-to-interactive unchanged within noise and the walk tally **0** on an already-indexed universe.

**§M4 — The freeze is gone, and which §4 mechanism we ship.** `[revised: this measurement now decides a design branch, not just a claim.]`
```sql
.timer on
EXPLAIN QUERY PLAN
  SELECT je.value, COUNT(*) FROM note_meta,
    json_each(CASE WHEN json_valid(tags_json) THEN tags_json ELSE '[]' END) je
  WHERE je.type='text' AND je.value<>'' GROUP BY je.value;
-- then, after: CREATE INDEX idx_note_meta_tags ON note_meta(path, tags_json);
-- re-run EQP. If it shows the covering index → §4 ships one atomic transaction.
-- If it does not → §4 ships the windowed scratch+swap with modified>=run_start replay.
```
Baselines to reproduce verbatim from source: `tag_counts.rs:58-67` (expect ~13.0 s on the INSERT) and `review.rs:1361` (expect ~2.7 s on the orphan DELETE). After §4/§5, instrument the new versions and record the **longest single writer-lock hold**. The number quoted to the Boss is only ever that measured maximum.

**§M5 — Foreign-copy count** (gates §8's report and §13's confirmation number).
```sql
SELECT library_name, COUNT(*) FROM note_meta GROUP BY library_name ORDER BY 2 DESC;
```
Cross-check names against `.constellation/libraries.json` (own) versus `universe.json`'s `children` (federated). **Read both files** — do not infer which names are foreign. This is also where I learn whether any child universe is nested under the active root (§8's exclusion).

**§M6 — Drift-check cost after scoping** (gates §9's "post-paint, invisible" claim). Pre-scoping figure is 160–590 ms. Anything above ~600 ms goes behind a longer post-paint delay before §9 ships.

**§M7 — Locale parity, before every commit that touches strings.**
```bash
node scripts/i18n-parity.mjs && npx vitest run tests/i18n/locale-parity.test.ts
```
Baseline verified today: **✓ All 15 locales in parity**; svelte-check 0 errors. Any step leaving any of these non-green does not commit.

---

# CRITIQUES ANSWERED — where a finding was wrong, in one line each

- **Landability F13** (unconstructed variant may fail the build): **not a false positive, but downgraded.** Verified `src-tauri/Cargo.toml` has no `[lints]` and neither `lib.rs` nor `main.rs` carries a crate-level `#![deny(...)]` — it is a warning, not a build failure. §14 gates it anyway.
- **Landability F11** (§8 might turn into a mass delete): **refuted, and the plan now cites the refutation** — `reconcile.rs:130-133` `if !roots_norm.iter().any(|r| under(&pn, r)) { continue; }` skips any row outside every root, so foreign rows become invisible to the stale set, never stale. The critique itself flagged this as the good news the draft failed to bank; it is banked in §8.
- **Hazards F8** (citation drift `mig108.rs:1200` vs `1199`): confirmed — `:1199` is `create_outgoing_link_triggers`, `:1200` is `recompute_all_outgoing`. Both are now cited correctly, and the draft was right that `mig108.rs:2181` reaches creation only transitively through `init_db`.
- **Hazards F8** (the sky accept is status quo, not a live choice): confirmed — `drop_sky_aggregate_triggers` (`search.rs:1856-1866`) drops only the `stratum`/`maturity` family; the edge mirror already survives today's walk. §7 and the not-in-scope table now say so.
- Every other finding in both critiques is accepted and fixed above.

**Net: 16 steps → 15**, with one genuinely large merge (§7) that buys the elimination of the half-migrated commit this project calls its dominant defect, and one genuinely new hazard closed that neither critique caught (§13's federated-coverage precondition).