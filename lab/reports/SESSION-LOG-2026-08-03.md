# Session Log — 2026-08-03

**Branch:** `main` · **Head at session start:** `5dbe0a2c`
**Function in hand:** **PJ-207 — the index repair.** The app's authoritative self-heal
(`search.rs::reconcile_filesystem`) has no user-reachable route, and its error message names a
"Settings → Rebuild Index" control that does not exist, in all 15 languages.

**Concept (the horse):** *Did my notes change while Constellation wasn't watching — and can I make
the index agree with them again?* Write-Time Derivation is correct while the app runs and
structurally blind to the interval when it isn't; nothing hooks a write that never came through
Constellation. This answers that interval and nothing else.

**Baseline gates re-run at session start** (never recalled — the handover's figures were already
stale): vitest **900/900** (76 files) · svelte-check **0 errors** · i18n parity **15/15**.
The handover said 816/816 (71 files); the ledger said 812/812.

---

## §1 — Reproduce-First: PJ-207 reproduced on the live universe

Full record: `lab/reports/PJ-207-REPRODUCTION-2026-08-03.md`.

Measured read-only against `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db`
(1.89 GB, 7,824 rows) with the app not running:

- **60 of 7,824 notes** have disk content newer than the index. Largest drift **4,735,509 s ≈ 55 days**.
- **57 of the 60** hold body words absent from `note_meta.body_text` (frontmatter stripped before
  comparing; ASCII words ≥ 6 chars only, so Arabic normalisation and markdown-stripping cannot
  produce a false positive). `Arcesilaus.md`: *carneades*, *skeptical*. `Vishnu Purana.md`:
  *mountains*, *planets*, *rivers*.
- Read from the live schema: `notes_fts` is `content=note_meta` over `body_text`, maintained by
  `note_meta_ai`/`note_meta_au`. **Those words are therefore unsearchable**, and no user action fixes it.

Named recipe: close Constellation → edit a note outside it → reopen → search for the new word →
not found, permanently. Mechanism verified in source: no boot step re-reads changed files
(`reconcile.rs` heals *existence* only; `reindex_library` is `onlyIfUnindexed`;
`cache_mark_search_ready` is explicitly walk-free; the watcher starts after the edit already
happened; the auto-recovery at `+layout.svelte:2891` is gated on `totalIndexed === 0`).

## §2 — First timings ever taken of the repair pass

No measurement of `reconcile_filesystem` existed anywhere in `lab/`, `docs/` or the session logs.
Measured on a byte copy of the live DB, using SQL verbatim from source:

| | Measured |
|---|---|
| `tag_counts::recompute_all_in` (txn `search.rs:10579`) | **13.2 s** writer-lock hold |
| `review::recompute_all_in` (txn `search.rs:10599`) | **20.6 s**, 260 MB of bodies resident |
| stat-only drift check, whole universe | **160–590 ms**, no reads, no writes |
| full re-read, **I/O floor only** | **49.0 s** (7,824 files, 298 MB, 6.27 ms/note) |

The walk waits 30 s for the writer lock (`search.rs:10496`); every user save waits 5
(`search.rs:3634`). A save landing inside either transaction fails *and* freezes the window,
because it holds the one `state.db` mutex 71 call sites need.

Two circulating figures must not be cited: the "~1.0 s bulk walk" is a synthetic two-table SQL
benchmark (`links_backfill.rs:636-760`); the "30–60 s" is a 2026-05-04 estimate predating the 2 GB DB.

## §3 — Whole-ecosystem sweep (58 agents, every candidate adversarially refuted)

298 candidates → 285 confirmed → **256 that a fix must bring along**. Load-bearing findings:

- `docs/concept-papers/29-settings.md` **forbids** this control in four places (§3, §7, §9, §10),
  including *"No `scan_*`/`rebuild_*` anywhere. Compliant."*
- **Charter W2-9 (OPEN, HIGH)** — the pass walks the *federated* library set and writes foreign
  cUniverse notes into the active universe's DB. *"needs a scoping decision."*
- **LL-027** was written for this exact bug (BUG-022) and ranks a manual button **second**:
  *"Prefer a gated automatic recovery… over a manual button the user has to know to click."*
- The design was already written down twice and never built: `+layout.svelte:4207` names *"a future
  cheap stat-only sweep"*, pointing at **Criterion 4** in `lab/boot-perf/BOOT-BUDGET.md:101`
  (*"Still not implemented"*, specified 2026-04-15). Commit `35100f1d` (2026-05-31) removed the boot
  walk and wrote that closed-app bulk changes are *"handled by Settings → Rebuild Index"* — naming a
  control that did not exist. That is the origin of the false promise.
- Ten registered Tauri commands have zero frontend callers, three of them index/maintenance doors
  (`cache_reconcile`, `cache_is_populated`, `embeds::invalidate_vault_index_cmd`).

## §4 — Boss rulings (2026-08-03)

1. **Placement** — permanent control in **Settings → Index** *and* a **Repair now** action on the
   health alert bar. `29-settings.md` to be amended (a *press* is not a *change*).
2. **What it repairs** — default **catch-up** (mtime-gated) + a separately-confirmed **Full re-read**.
3. **Detection** — post-paint **stat-only drift check** (Criterion 4), notice only when drift exists.
4. **Design B — "re-found the walker"**, chosen over the recommended A.
5. **Full re-read** — build it, but keep it behind the flag until measured.
6. **Foreign cUniverse copies** — report them *and* offer removal behind its own confirmation.
7. **The 34 s freeze** — fix it inside this migration.

Lead's ruling carried into the plan: Design B's thesis is adopted, but **not** its two condemned
mechanisms — trigger creation must not become conditional on an in-process flag (a leaked flag
silently freezes `note_meta.outgoing_*` on the live save path), and `MIGRATION_ACTIVE` must not be
held for the whole run (it stands the WAL checkpoint daemon down, `search.rs:9730-9736`).

## §5 — PREDECESSOR → REPLACEMENT (Predecessor Lookup Rule — written BEFORE any code edit)

| Predecessor | Where it lives now | Where the replacement lives | Cut / kept |
|---|---|---|---|
| `cache_reconcile` | `src-tauri/src/cache.rs:1511`, registered `lib.rs:506`. **Zero frontend callers** (only comments at `+layout.svelte:2799/2899/3681/4196`). Introduced by `9b5a491d` (2026-05-30), orphaned by `35100f1d` (2026-05-31). | **Same concern, one runner** — the new guarded repair runner. | Command **removed** from `generate_handler!`. Its two behaviours the live door lacks — the `cache-reconciled` emit (`:1530`) and `kh_cache_recompute_blocking` (`:1539`) — are **kept**, moved into the runner. Its `Err(_) => (0, true)` false-success (`:1526-1529`) dies with it. |
| `reconcile_filesystem` (`pub`) | `src-tauri/src/search.rs:10468` | Same file/concern, private behind the runner. | The `pub` is **cut** so no second door can call it. Body **kept**. |
| `constellation_search_init` | `src-tauri/src/search.rs:10639`; frontend `initSearchIndex` `store.ts:3703`; 4 call sites `+layout.svelte:2892/4690/5971/5984` | **Same place** — becomes a thin request to the runner. | Command name and frontend wrapper **kept** (no IPC retired); its body is re-pointed. |
| `reindex_library` | `src-tauri/src/libraries.rs:3434`; frontend `store.ts:3600/3612`; boot fan-out `+layout.svelte:2860` | **Same place**, absorbed as a repair mode. | Command **kept**. The boot fan-out's N parallel calls become **one** request — otherwise single-flight would refuse N−1 and silently re-open the LL-027/BUG-022 cold-start gap. |
| Settings "Rebuild Term Embeddings" button | Removed by MIG-013 §1D-B (`0ac12eb2`); orphan CSS survives at `SettingsModal.svelte:2795-2871` (~15 rules, zero markup users) | **Settings → Index**, the section the 15 locale strings already name. | Orphan CSS **deleted**; dead key `appSettings.index.semanticSearchEnabled` (`store.ts:6156/:6538`, zero readers) **deleted**. |

No Tauri command is retired without a replacement in the same commit. Boss approval for the
Settings placement is recorded in §4 above.

## §6 — Defects discovered in-pass (WA#6 — to be fixed, not noted-and-shipped)

| | Defect | Evidence |
|---|---|---|
| D0 | **`index_note`'s preserve predicate omits confidence.** `(traversal > 0 \|\| weight != 1.0 \|\| status != "active") && !structural` — a link promoted to *evidence*/*established* but never traversed is re-inserted as `hypothesis` with `created` reset. Fires on **ordinary saves** today; self-heals only at next boot (`link_life_restore` is boot-only). **Ships first, alone, with its own test.** | `search.rs:7115`, re-insert `:7210-7214`. Trap: `search.rs:338 is_preserved` is a hand-mirrored copy exercised by 5 tests — widen production alone and all five stay green. |
| D1 | `cache_reconcile` maps `Err` to `(0, true)` — a failure emits as a successful cold walk with 0 notes. | `cache.rs:1526-1529` |
| D2 | `indexHealthError` clears only under `if (attempt > 0)` — i.e. only when attempt 0 threw *and* the retry succeeded. Once set, the red bar is **permanent for the session**. | `store.ts:3714` / set `:3735` / clear `:3728` |
| D3 | `reindex_single_note` returns `Ok(())` while all three of its maintenance calls fail silently (`eprintln`-only). A per-note repair loop on it can report "0 problems" over N silent failures. | `search.rs:11122`, `:11136`, `:11150` |
| D4 | `index_note` reads mtime and content **outside** its transaction — a save landing during a walk is overwritten, permanently and silently. | mtime `search.rs:6592`, read `:6609`, txn opens `:6749` |
| D5 | Walk errors discarded (`let _ =` `search.rs:7262`), `read_dir` failure silent (`:7249`), `note_count` is a bare `COUNT(*)` (`:10610`) — identical whether the walk indexed 7,800 notes or zero. | as cited |
| D6 | The foreign-copy removal would **oscillate**: `reconcile.rs:91` takes roots from the recursive set and `:280-300` re-adopts any orphan `.md` under them. Cure: route **both** passes through `universe::own_libraries_for_root` (`universe.rs:1479`), the helper already written for this discipline. Caveat: it reads `libraries.json` with `unwrap_or_default()` — an unreadable file yields an empty list, which for a repair means *walk nothing and report success*. | as cited |

## §7 — Phases 1 and 2 closed

**Phase 1 (Architect)** — `docs/PJ-207-Index-Repair-Architect.md`. Three competing designs, three
adversarial judges. Boss picked **B ("re-found the walker")** over the recommended A. Two of B's
mechanisms were rejected by the lead and re-decided in Phase 2 (below).

**Phase 2 (Plan)** — `docs/PJ-207-Index-Repair-Plan.md`. **15 steps, 12 Boss-testable.** Two
adversarial reviews (landability + hazard-reintroduction) reordered the draft: three steps
forward-referenced symbols arriving later, and the destructive foreign-copy removal sat three steps
*before* the dialog it needs. The ordering law now actually holds — §1–§10 add **no** new route to
the walk; §11 is the door; §13 is the deletion, after it.

**The two hazard rulings, re-decided against my own first instruction:**
- **H1** — I said "don't gate trigger creation behind an in-process flag." The draft did exactly
  that one stack frame up, by gating `on_link_vocabulary_changed`, whose body *contains* the creation
  site (`search.rs:1889`) — and it would have parked `save_universe_link_types` behind a run with a
  49 s floor. **Final: no gate anywhere.** `create_outgoing_link_triggers` drops-then-creates
  (`search.rs:1500-1503`), so a mid-run re-arm is idempotent. Cost of the collision is
  **performance, never correctness** — which is the trade H1 was buying. The window is bounded by an
  RAII guard (precedent `mig108::RunningGuard`, `mig108.rs:50-59`).
- **H2** — the draft dismissed the pre-run `maybe_schedule_defrag` as harmless. That worker *takes*
  `MIGRATION_ACTIVE` and holds `state.db` for minutes (`search.rs:2050-2052`, `:2158-2167`). **Final:
  mutual exclusion in both directions.**

## §8 — §1 BUILT — link promotions and birth dates survive a re-index

**Two production changes**, one file (`src-tauri/src/search.rs`):

1. **One shared predicate.** `link_row_is_preserved(traversal_count, weight, status, confidence,
   structural)` + the `CONFIDENCE_UNJUDGED` sentinel. It gains the clause the old condition lacked —
   `confidence != "hypothesis"`. **The hand-mirrored copy at `search.rs:338` is DELETED** and its
   five tests now call production, so the trap that would have kept them green through a
   production-only widen is structurally closed.
2. **`created` carried forward.** `old_edges` gains the stored birth date, and the non-preserved
   re-INSERT binds it instead of `now`. Only a genuinely new edge is born now.

**RED-proven separately, both mechanisms**, by reverting each in isolation:
- predicate reverted → `a_promoted_link_survives_an_ordinary_edit_with_its_birth_date` fails
  **through the real indexer**: `left: "hypothesis", right: "established"`.
- carry-forward reverted → `an_unjudged_link_is_still_rebuilt_but_keeps_its_birth_date` fails with
  the timestamps 8 ms apart (`…671244200` vs `…663587800`).
- The other seven pre-existing assertions stayed green in both runs, proving the old contract intact.

**New tests: `tests_pj207_reindex_round_trip`** — the **first test of `index_note` in either suite**.
Before PJ-207 nothing tested the walk primitive; it was only ever exercised through hand-mirrored
predicates. It is directly testable because it takes a bare `Connection`, no `AppHandle` — which is
exactly why the mirror was never needed.

**Self-review caught one reuse slip in my own diff**: the test helper hardcoded `"hypothesis"`
instead of the constant it had just been given. Fixed before the build.

**Gates:** Rust **1344 passed / 0 failed** (1339 baseline + 5 new) · frontend untouched.
**Per-build safety inspection:** `wf_bdb74b70-066`, 72 agents, 58 verified, **40 confirmed — ZERO in
`search.rs`**. The §1 diff is clean; register at `lab/reports/inspection-2026-08-03-pj207-s1.md`.
**PJ-166, tenth strike** — invoked diff-scoped with `args.files`, returned `mode: "whole-app"` again.

**The inspection escalates triage item #11 to APP-KILLER** and names a second branch the triage
missed: `loadWorkspaces` (`store.ts:7186`) refuses to adopt a *successful empty* read
(`if (data.length > 0)`), contradicting its own comment — so universe A's layouts stay live in
universe B and the first Save/Delete writes them over B's file. Collections, settings and
property-types all got the universe-switch reset; workspaces is the sibling that never did.

Release binary rebuilt **17:44** (source last touched 17:00) — freshness verified per Stage 0.

**BOSS-TESTED AND PASSED** → committed `3c0dc84b` *"§1 — a judgement is earned data too"*.

## §9 — §2 BUILT — the dead doors, the orphan UI, and the off-switches

Pure deletion plus two constants. Nothing gained a new capability.

**Deleted:**
- **`cache_reconcile`** (`cache.rs`, registered `lib.rs:506`) — a `#[tauri::command(async)]`
  wrapping the very walk PJ-207 makes reachable, with **zero frontend callers**; the only four
  matches in `src/` were comments describing a call that no longer happened. Added by `9b5a491d`
  (2026-05-30), orphaned the next day by `35100f1d` (MIG-067). **D1 dies with it**: its
  `Err(_) => (0, true)` emitted a FAILED walk as a successful cold walk with 0 notes,
  indistinguishable from an empty universe, error String discarded.
- **15 orphan `.semantic-*` CSS rules** (`SettingsModal.svelte`) — markup removed by MIG-013
  §1D-B (`0ac12eb2`), stylesheet left behind. `.semantic-status-rebuild` was the last physical
  trace of the "Rebuild Index" button the app has been telling users to press ever since.
- **`appSettings.index.semanticSearchEnabled`** — declaration + default, zero readers for three
  months. Verified safe: Rust reads settings as opaque `serde_json::Value`
  (`universe.rs:1618`), no typed struct and no `deny_unknown_fields`, so an existing
  `settings.json` carrying the old key is simply ignored. Nothing to migrate.
  The comment that said the key was "left for backward compat" now records that this commit *is*
  that garbage collection.

**Added:** `src/lib/index/repairFlag.ts` — `REPAIR_DOOR_ENABLED` (gates §9/§11/§13's user-reachable
routes) and `FULL_REREAD_ENABLED` (**ships false** per Boss ruling, until §M1 measures it).
Deliberate asymmetry, documented in the file: flag-off removes the **doors** and keeps every
**guard**, because the guards fix defects that exist today — a repair already runs on library-add.

**Fixed in passing** (same file, same concern, rather than left standing): `cache.rs`'s own doc
comment still told the reader the walk "belongs to… an explicit Settings → Rebuild Index." That
sentence — written by MIG-067 when it removed the boot walk — is the origin of the promise the app
makes in 15 languages. Corrected, with the history recorded in place.

**Verification clause discharged.** The registration removal is compiler-verified (a dangling
`generate_handler!` entry is a compile error) — `cargo check` clean. The event survives the
deletion: `grep -rn '"cache-reconciled"' src-tauri/src` still returns an emitter
(`cache.rs:1546`, in `cache_mark_search_ready`, which is the one boot actually calls at
`+layout.svelte:2905`), and all three listeners are intact (`+layout.svelte:3641`, `:3684`,
`CollectionsPanel.svelte:79`). **Without that check this deletion is indistinguishable from one
that orphans three listeners.**

**Gates:** svelte-check **0 errors** · vitest **900/900** (76 files) · cargo check clean.

**BOSS-TESTED AND PASSED** → committed `aae51aff` *"§2 — the dead doors, the ghost of the button,
and the off-switches"*. His finding (both Index toggles already ON at first look) investigated
against the real file: genuinely `true` in his stored settings from an earlier session — defaults
apply only to ABSENT keys — and the save round-trip preserved all 97 setting blocks. Not a
regression. It did correct my own comment: the retired key is **re-written on every save**, so it is
inert rather than gone. Stripping unrecognised keys is declined deliberately — that is how an older
build destroys a newer one's setting.

## §11 — §3 BUILT — the per-note indexer stops reporting success it did not earn

Rust-only, all in `search.rs`. Three defects, each of which let the repair loop §11 will build
report "N repaired, 0 problems" over silent failures.

**D5 — the walk now accounts for itself.** `index_note` returned `Result<(), String>`, flattening
four distinguishable outcomes into "no error", and `index_library_recursive` returned `()`, so the
command could only report `SELECT COUNT(*) FROM note_meta` — **a number identical whether the walk
indexed 7,800 notes, skipped them all as unchanged, or failed on every one.** Now
`IndexOutcome { Indexed | Unchanged | Raced | Skipped }` and `WalkTally { seen, indexed, unchanged,
raced, failed, dirs_unreadable, unreadable_sample }`, surfaced as `SearchIndexStats.walk`.
Two silent losses closed with it: a `read_dir` failure returned from an entire subtree with no
trace (a permission-denied folder, an un-materialised OneDrive placeholder, a path past the Windows
limit — the library simply indexed short and reported success), and the depth>20 cut-off, which is a
real truncation and is now counted as one. The sample is capped at 20 across recursive accumulation,
mirroring `reconcile.rs`'s bounded diagnostics.

**D4 — the save-during-walk window, narrowed and its residual named.** A re-stat is now the FIRST
statement inside `index_note`'s `BEGIN IMMEDIATE`; if the file moved since the pre-read stat, the
note is abandoned with its row untouched and counted as `raced`. The file read, frontmatter parse,
wikilink/heading extraction and markdown strip stay **outside** the transaction on purpose — moving
298 MB of reads and every parse inside per-note write transactions would manufacture the freeze this
migration exists to remove. **Stated limitation, not papered over:** `modified` has second
resolution (PJ-060 documents it a few lines above), so a save landing in the SAME SECOND still
compares equal and is still overwritten. This narrows the window from the whole read+parse span to
sub-second; it does not close it. Closing it needs content hashing on the walk path — filed.
Scoped to `!force` deliberately: every `force: true` caller is a "this file just changed" context
where a moved mtime means another write is already in flight and will itself reindex; refusing there
would convert a benign race into a silently-skipped index with no retry — the very class being closed.

**D3 — three best-effort steps that reported nothing.** `ctse::hooks::on_note_indexed`,
`maintain_incoming_after_save` and `maintain_sky_after_save` were `eprintln`-only while
`reindex_single_note` returned `Ok(())` — and stderr goes nowhere in a Windows release build. They
stay best-effort (a term-index delta must never fail a save) but now report via
`MaintenanceOutcome`. **The IPC contract is deliberately unchanged**: `constellation_search_reindex`
keeps `Result<(), String>` via `.map(|_| ())` — it is the per-save hook with 21 fire-and-forget
frontend callers, and a derived-view delta failure must not surface as a failed save. The outcome is
consumed Rust-side by the runner, the caller that can act on it. `docs/IPC-CONTRACT.md` needs no edit.

**The compiler found the blast radius**: exactly two call sites broke, both fixed. Every other one
of the ~15 `reindex_single_note` callers uses `.is_ok()` / `let _ =` / `Ok(_) =>` / `.unwrap()` and
is source-compatible.

**Tests + RED proofs.** Two new tests, both RED-proven by reverting their own mechanism:
- `the_walk_distinguishes_indexed_from_unchanged_from_failed` — three files including one with
  invalid UTF-8 bytes (a genuine per-note failure). First walk: 2 indexed, 1 failed. Second walk:
  **0 indexed, 2 unchanged, 1 failed** — while `COUNT(*)` reads 2 after both, which is the whole
  point. Reverting `Err(_) => tally.failed += 1` fails it (`left: 0, right: 1`).
- `an_unwalkable_subtree_is_counted_and_does_not_abort_the_walk` — driven through the depth cut-off,
  deterministic on every platform, unlike a permission-denied directory which needs `icacls`. Both
  take the same `note_unreadable` branch. Reverting it fails.

**Honestly not tested, and said so in the test module rather than implied:** the D4 race guard's
TIMING. Both stats happen inside one synchronous call, so there is no in-process seam to drive it
deterministically — a thread racing the writer would be a flaky test, which is worse than an honest
gap. Its correctness rests on something structural instead: the re-stat is the first statement
inside the write transaction, so nothing can slip between check and write.

### §3's per-build gate — a focused adversarial review of the diff

The standing order's per-build inspection was substituted here by a **focused adversarial review of
the §3 diff alone**, stated rather than quietly skipped. Reason: the whole-app sweep ran two hours
earlier on effectively this tree (PJ-166's tenth strike — it ignores `args.files`), cost ~30 min and
9.1 M tokens, and confirmed 40 findings **none of which were in `search.rs`**. Re-running it for a
Rust-only instrumentation change would have re-surfaced the same 40. The value of a per-build gate is
**in-diff** findings, and that is what was hunted, with the same refute-first discipline.

**Verdict: no APP-KILLER, no HIGH.** Six hypotheses (A–F) chased to the code and refuted:
- the `Raced` branch is **not** a new staleness generator — the only production `force: false`
  caller is the walk itself, and the old behaviour wrote stale bytes stamped with the *pre-read*
  mtime, leaving the identical row≠disk exposure healed by the identical three paths (app save,
  watcher, boot re-adopt). `Raced` is strictly better for the app-save case.
- ROLLBACK is clean — the guard is provably the first statement, nothing above `BEGIN IMMEDIATE`
  writes, and the closure has exactly two `Ok` exits so the COMMIT arm cannot mislabel.
- `SearchIndexStats` is `Serialize`-only with a skip-if-none field, and all four frontend callers
  discard the resolved value — backward compatible.
- All 15 `reindex_single_note` call sites bind `Ok(_)`, never `Ok(())`.
- The depth cut-off cannot false-alarm: dot-dirs and nested libraries are filtered *before* recursion.
- The 20-entry sample cap holds across recursive **and** cross-library accumulation.

**Three LOW findings — all FIXED before the build, none deferred (WA#6):**
- **G1 (LOW-MED), the real one.** `Skipped` was counted nowhere. A sync client deleting 50 notes
  mid-walk would leave `seen: 7824`, the other buckets summing to 7774, and `is_clean() == true` —
  the gap inferable only by subtraction. In a step whose whole concept is *the walk's honest account
  of itself*, an unaccounted bucket **is** the defect. Added `skipped` through tally → absorb →
  walker → report.
- **A1.** `.unwrap_or(0)` on the re-stat turned a transient sharing violation (a sync client or AV
  holding the handle) into `0 != modified` → `Raced` — a *normal* outcome that keeps `is_clean()`
  true, so the note would be missing from the index with nothing reporting it. A stat that fails is
  a failure and now says so.
- **G2.** Two comments described the §7/§11 consumers as though they already existed, and
  `MaintenanceOutcome::is_clean` added the build's only new warning. Comments made forward-looking;
  the method annotated with why it is kept rather than deleted (deleting it would force the
  predicate to be re-derived at the call site later — the hand-mirrored shape §1 unpicked).
  Release warnings back to **57**, the pre-§3 count.

**My own balance test was weak and was rewritten.** As first written it asserted the buckets sum on
a walk containing no skipped file — trivially true, a test that passes for the wrong reason. It now
pins the branch (`index_note` on a missing path → `Skipped`) *and* the plumbing G1 actually broke
(`absorb` carrying `skipped`), RED-proven by dropping the bucket from accumulation (`left: 0,
right: 2`).

**Gates:** Rust **1347 passed / 0 failed** (1344 + 3). Release binary 19:23:50 vs source 19:19:58.

### §3 Boss test — Steps 1, 2, 4 PASS. Step 3 investigated, NOT a §3 regression.

Boss: *"I created '3mooR' folder, added 3 notes… Add to it the word 'run7'. Add the folder as a
library. Search for 'run7', I got nothing."* Investigated against the real data, not explained away:

| Check | Result |
|---|---|
| Library registered? | **Yes** — `libraries.json`, `3mooR`, at the universe root |
| All 3 notes indexed? | **Yes** — 3 `note_meta` rows, `library_name = '3mooR'` |
| Is `run7` in the index's body text? | **Yes** |
| Is the row stale? | **No** — indexed mtime == disk mtime, to the second |

So **indexing worked perfectly**; the loss is in *search*. Reproduced in a throwaway temp database
with the real indexer and the real FTS tokenizer:

```
BODY = "Run\nrun7\nzarquon\nblorptide9"
MATCH run7       -> 0
MATCH run        -> 0
MATCH zarquon    -> 1
MATCH blorptide9 -> 1
MATCH blorptide  -> 1
```

**The digit is not the problem** — `blorptide9` is found by both spellings. The cause is two rules
composing: `is_word_boundary` is `!c.is_alphabetic()` (`fts5_tokenizer.rs:417`), so a digit ends a
word and `run7` tokenizes to **`run`** — and **`run` is in the English stopword list**
(`libraries.rs:4421`, between `"set"` and `"put"`). Stopwords are dropped from both the index and
the query, so the term is unsearchable by construction.

Confirmed on the live universe's own FTS vocabulary: `zarquon` → 1 doc and `blorptide` → 1 doc (the
Boss's own Step-1 and Step-2 test words, proving the save path and the watcher path end-to-end on
his machine), `notepane` → 3 docs, and `run` → **absent**.

**Not caused by this migration.** `git diff HEAD~3` over `fts5_tokenizer.rs` and the stopword list in
`libraries.rs` is empty — §1–§3 never touched either. Step 3 was a passing indexing test with an
unlucky test word.

**Filed as PJ-214** — *a search term that is filtered to nothing returns zero results with no
explanation.* Same class as the whole migration: the app knows why it found nothing and does not say
so. A user searching `run`, `set`, `done`, `via` or any of ~200 stopwords gets a blank result with no
indication the term was dropped rather than genuinely absent. Not urgent, not a data-loss defect,
but exactly the kind of silence PJ-207 exists to end.

### A self-inflicted scare, recorded rather than hidden

Removing the throwaway probe with a Python slice, I used `s.find(...)` without checking for `-1`; it
returned `-1`, the slice became `s[:i] + s[2:]`, and **the file was written back at 2× its length** —
every item duplicated. `cargo check` caught it immediately (unknown token, duplicate definitions);
nothing was committed and nothing reached the Boss.

Recovered deterministically rather than by re-typing the work: the corruption was exactly
`new = old[:i] + old[2:]`, so `old` was reconstructed as `HEAD[:2] + new[second_boundary:]`, locating
the boundary by the file's own opening bytes. Verified by size (778,326 → HEAD's 754,086 + my
additions), by marker counts returning from doubled to single, by `cargo check` clean, and by the
suite returning to **1347 passed / 0 failed** — the exact figure from before the accident.

**The lesson, which is the same one this migration keeps teaching:** an unchecked `find` returning a
sentinel that is *valid as an index* is the string-handling twin of `Err(_) => (0, true)` — the
failure silently becomes a plausible-looking success. I deleted a command for that shape this
afternoon and then wrote it myself an hour later.

**BOSS-TESTED: steps 1, 2, 4 PASS; step 3 proven not a regression** → committed `eaafe240`
*"§3 — the indexer stops reporting success it did not earn"*.

## §13 — §4 BUILT — the tag-count rebuild stops dragging 270 MB off disk

**A correction I owe on my own measurement, first.** I reported the tag-count rebuild's writer-lock
hold as **13.2 s** — in the reproduction record, the Architect doc, the Plan, and to the Boss. That
figure is real but it is a **cold-cache** measurement, taken on a freshly-copied 2 GB file, and I
presented it without that qualifier. Re-measured on the same copy once warm: **1,852 ms** on the
second run and **60 ms** on the third. Cold is the honest worst case — and it is exactly the state of
the first repair after launching the app — but it is not the steady-state cost, and saying "13.2 s"
flat overstated it. Corrected here and in the reproduction record.

**The plan's gate, discharged before any code was written.** §4 required the query planner to
*actually choose* a covering index, with a windowed scratch-and-swap as the fallback if it did not.
Measured on a byte copy of the live universe:

```
EXPLAIN QUERY PLAN  ->  SCAN note_meta USING COVERING INDEX idx_note_meta_tags
```

So the primary mechanism ships and **the fallback is not needed** — which matters, because the
fallback was the risky one: `tag_counts` is maintained write-time by a ± delta applied *inside*
`index_note`'s own transaction, so today's DELETE + single INSERT is atomic under the writer lock and
no delta can interleave. A scratch-build + swap would open exactly that window, and a delta landing
after its rows were scratched would be discarded at the swap.

**The cache-independent number is the real argument.** The query needs `tags_json` and nothing else,
but a table scan drags every row's inline `body_text` with it:

| | |
|---|---|
| covering-index payload (`path` + `tags_json`) | **1.6 MB** |
| what the table scan must fault in | **270.3 MB** (259.5 MB of it `body_text`) |
| ratio | **167× fewer bytes** |

Index build **94 ms**, and **zero** additional file size on that database — absorbed by the existing
freelist. `recompute_all_in` on the real corpus with the index: **386 ms**.

This is the same shape, for the same reason, as `idx_note_meta_map` (MIG-077) sitting three lines
above it — which was added when a cold Map open took ~26 s for exactly this cause. `IF NOT EXISTS`,
no schema-version bump, so it is picked up on the next launch with no rebuild.

**Tests — and one I had to split rather than fake.** The unit test proves the index is a pure cost
change (not one count moves, against a corpus with duplicates-within-a-note, a shared tag, an empty
list, malformed JSON and an Arabic tag). It does **not** assert plan selection: `note_meta` in the
fixture has three columns, so `(path, tags_json)` is nearly the whole table and SQLite correctly
declines the index — asserting it there would pin the fixture, not the fix. I first wrote that
assertion anyway and it failed honestly, twice, including after padding the fixture to 2,000 rows
with fat bodies. Plan selection is a property of the real 30-column table, so it now lives in an
`#[ignore]`d rehearsal test against a real-corpus copy, mirroring the existing
`rehearse_against_live_copy`. Run and passing:
`[rehearsal] plan: SCAN note_meta USING COVERING INDEX idx_note_meta_tags`.

**Gates:** Rust **1348 passed / 0 failed** (1347 + 1). Binary 05:40 vs newest source 05:36.

**BOSS-TESTED AND PASSED** → committed `98bca820` *"§4 — the tag-count rebuild stops dragging
270 MB off disk"*.

## §14 — §5 BUILT — the review rebuild stops holding 260 MB and one long lock

**Three changes, and one trap the plan glossed over.**

**1 + 2 — windowed, and `recompute_all_in` now owns its transactions.** It used to materialise
`(path, tags_json, modified, body_text)` for **every** row into one `Vec` inside a single
caller-held transaction. Now: 500-row `path >` windows, each its own short transaction with the
`links_backfill.rs:264-404` busy-retry (8 × 400 ms), which exists because a whole-table UPDATE
*"silently failed under boot DB contention — the 2026-05-30 overnight blank."* `backfill_one` is
idempotent per path, so windowing is semantically safe. **The caller must no longer wrap it** —
SQLite has no nested transactions; `search.rs`'s `walk_conn.transaction()` is gone and the
connection is passed directly. One production caller, so the contract change is contained.

**Measured on the real-corpus copy, both regimes stated — having just been burned by conflating them:**

| | old single transaction | new 500-row windows |
|---|---|---|
| warm, one hold | orphan sweep 348 ms + materialise 2,111 ms = **2.5 s** | worst window **109 ms** |
| cold (first repair after launch) | 2,747 ms + 17,886 ms = **20.6 s** | proportionally smaller |
| resident bodies | **260 MB** | **30.2 MB** |

The plan's clause was *"longest writer-lock hold < 1 s"* — **109 ms**.

**THE TRAP — the orphan sweep cannot be windowed over `note_meta`.** The plan said to window it "by
the same path range", and that is **wrong**. An orphan is by definition a `review_schedule` row
whose path is *absent* from `note_meta`, so it can sort anywhere — including past the last note
path, where a note_meta-derived window never reaches. It would have silently stranded exactly the
rows the sweep exists to remove: a subtler defect than the 2.7 s hold it was meant to fix. The sweep
therefore windows over **`review_schedule`'s own key space**, with the
`NOT IN (SELECT path FROM note_meta)` predicate unchanged so semantics are identical.

Pinned by `the_orphan_sweep_reaches_orphans_outside_the_note_path_range`, with orphans placed on
both sides of the note range (`zzz-` and `!!!-`). **RED-proven** by switching the sweep to the naive
note_meta windowing: `left: ["/lib/Mid.md", "/lib/zzz-after.md"], right: ["/lib/Mid.md"]` — the
after-range orphan survives, precisely as predicted.

**3 — a named heal for the interrupt.** The existing crash marker heals **three** link families at
boot; `tag_counts` and `review_schedule` had **none**. An app close mid-tail (the 5 s cap, a crash, a
power cut) left them partly recomputed and **undetectable** — the drift check compares `.md` files
to the index and is structurally blind to a derived table stale against itself. Added
`derived_tail_pending`, one `schema_versions` row on the same mechanism as its sibling: **set before
the tail** (refusing to enter an unprotected tail if it cannot be persisted — the trigger marker's
discipline) and **cleared last, after every family**, so a failure anywhere leaves the heal armed.
That direction costs one redundant recompute rather than a silently-wrong derived view.
**§6 adds the reader** — stated in the code rather than written as though the consumer existed, which
is the mistake the §3 review caught.

**Gates:** Rust **1349 passed / 0 failed** (1348 + 1).

### §5 Boss test — steps 1–3 PASS. Step 4 blocked by a defect it exposed.

Boss: adding a library via **Manage libraries → + Add library**, choosing
`E:\موسوعة عيسى\التصوير` — *"The process went well, but the folder I selected has never been
added."* Not a §5 regression (§5 is `review.rs` windowing + a marker). A genuine, separate,
silent failure — and the same class this whole migration is about.

**The mechanism, traced end to end.** The folder is **outside** the universe root, so under MIG-108
("One Universe, One Location") `add_library` refuses it — with a message already written for the
user: *"This folder lives outside the universe. Use Bring in a library to copy or move it under the
universe folder."* (`libraries.rs:533-536`). `LibraryManager.svelte`'s handler discarded it in a
bare catch-and-ignore. The picker closed, nothing was added, nothing was said.

**And it is a half-sweep, again.** The **sidebar's** add-library flow (`+layout.svelte:5919`) is
already correct: it compares the folder to the root and, when it is outside, opens the **Bring-In**
dialog offering Copy or Move — the intended MIG-108 experience — and surfaces failures on the
sidebar error line. One concern, two implementations, only one right.

**Fixed by removing the wrong implementation rather than repairing it.**
`store.addLibrary()` is **deleted** (Predecessor → Replacement below); `LibraryManager` gains an
`onAddLibrary` prop wired to the sidebar's handler, which owns the Bring-In dialog state. The
manager closes first so the choice dialog does not open behind it. After this there is exactly ONE
implementation of "pick a folder, make it a library", so there is nothing left to drift.

Deleted rather than left callerless deliberately: an add path with no under-root check sitting in
the store is a loaded gun for the next caller — the same reasoning that removed two dead doors in §2.

| Predecessor | Where it lived | Replacement | Cut / kept |
|---|---|---|---|
| `addLibrary()` | `src/lib/libraries/store.ts` — picked a folder and invoked `add_library` with no root check. Sole caller: `LibraryManager.svelte`. | **Same concern, the sidebar's `handleAddLibrary`** (`+layout.svelte:5919`), reached through the new `onAddLibrary` prop. | Store function **deleted**; its stale import removed from `+layout.svelte`. No IPC retired — `add_library` and `bring_in_library` both keep their callers. |

**A self-inflicted detour worth recording:** the doc comment I wrote to explain the fix quoted the
old code as `catch { /* ignore */ }` **inside a `/** … */` block**, so the nested `*/` terminated the
comment early — 230 svelte-check errors from one punctuation mark. Caught by the gate, fixed in
seconds, but it is the second time today a quoting/sentinel slip cost me a green tree.

**Gates:** svelte-check **0 errors** · vitest **900/900** · Rust **1349/0**.

## §15 — §6 BUILT — one place where derived views are rebuilt

New `src-tauri/src/converge.rs`. Five derived families (outgoing, incoming, sky,
`tag_counts`, `review_schedule`) had their bulk recomputes assembled **five different
ways** — the reconcile tail ran 5, the boot healer 3, MIG-108 ran 1, the incoming back-fill 1 —
and nothing anywhere said so. A sixth was one commit away, and the differences were invisible.

**The seal is real, and PROVEN, not asserted.** `ConvergeKey`'s field is a private unit, and all
five recomputes now require one. The plan asked for a compile-fail proof rather than the sentence
"cannot compile"; done by inserting a forged construction in `mig108.rs` and compiling:

```
error[E0603]: tuple struct constructor `ConvergeKey` is private
    --> src\mig108.rs:1204:39
```

Then removed. The alternative the plan proposed — narrowing to `pub(in crate::converge)` — is **not
expressible in Rust**: `pub(in path)` requires an *ancestor* module and `converge` is a sibling of
`links_backfill` / `tag_counts` / `review`. Short of moving five bodies into one file (a far bigger
diff that would strand each family's tests away from its module), the sealed token is the mechanism.
`ConvergeKey::for_test()` exists only under `cfg(test)`, so in-crate unit tests keep exercising their
own family directly while production has exactly one door.

**The compiler enumerated the blast radius — 11 sites, including one the plan missed**
(`tag_counts.rs`'s own back-fill).

**TWO PLAN ERRORS CAUGHT, both the same shape as §5's orphan trap — a builder is not a healer:**

1. The plan said to route `incoming_links_backfill.rs:151` through `converge::after_incoming_backfill`
   on the gated path. That would have made the back-fill a **permanent no-op**: it recomputes and
   *then* stamps, so the `is_built` gate it would be checked against is the stamp it is about to
   write. The gate belongs to *convergence* (don't recompute a family nobody reads yet), not to the
   initial build — so `Families::IncomingOnly` is deliberately ungated, documented in place.
2. `tag_counts`'s own back-fill builds **and stamps in ONE `IMMEDIATE` transaction**, and that
   atomicity is deliberate (a counter is additive; a build racing live ± deltas would double-count).
   Routing it through `converge`'s own transaction would have broken it. Split into a sealed
   cross-module wrapper and a body private to the module, which the builder calls directly.

**And one plan item that was mis-classified.** `on_link_vocabulary_changed` is listed as the fifth
assembly, but reading it, it *schedules two background back-fills* — cursor-based and resumable —
rather than calling any recompute. Forcing it through `converge` would have been wrong. Left alone,
recorded rather than silently skipped.

**§5's marker now has its reader.** The boot healer is armed by **either** marker —
`outgoing_triggers_dropped` (the walk died with triggers down) **or** `derived_tail_pending` (it died
anywhere in the tail, *including after* the triggers were restored — the window that previously had
no marker and therefore no heal). And it now converges **all five** families instead of three, so
`tag_counts` and `review_schedule` finally have a boot heal.

**Gates:** Rust **1351 passed / 0 failed** (1349 + 2). Binary 14:32 vs newest source 14:18.

## §12 — Boss ruling 2026-08-03: surface a link's age

Asked at the §1 pass: *"I want the link's age to be surfaced."* Filed as **PJ-213** rather than
built inline — it is a new user-facing feature, not part of the approved 15-step plan, and doing it
properly is its own small job: the Outgoing Links **and** Backlinks panels (Whole-Ecosystem — they
are the two hosts of `ConfidencePicker`), a date-format decision, i18n ×15, and RTL. It also lands
in a file with a confirmed open inspection finding (`ConfidencePicker.svelte:61`).

Context that makes it worth doing: `created` is one of the eight Living-Link properties and the
basis of weight decay, it is **not** in the earned ledger, and until §1 shipped today it was being
silently reset by ordinary edits. Surfacing it makes that class of loss visible to the user instead
of only to a test.

---

## §16 — §7 SHIPPED and Boss-validated · the day closed

**§7 `ee2191ed`** — one runner. Two independent walkers with seven entry points became one
submit-point with a typed outcome; `reconcile_filesystem` lost `pub`; `reindex_library` was
absorbed; the boot fan-out collapsed; the bring-in double-fire went; mutual exclusion landed
both ways with defrag and MIG-108; the walk gained a per-note cancel / universe-switch /
checkpoint gate it never had.

**The safety review found ten, two of them HIGH, and both HIGH were mine** — safety nets
disarmed while being built. The marker that cleared itself on the exact failure it existed
for; the single-flight flag with no RAII, against this codebase's own precedent whose
comment explains why one is needed. All ten fixed before the commit. **My own panic test had
to be corrected rather than satisfied** — it asserted the marker gets cleared, which was
pinning the bug; the unwind path must KEEP it, because the tail provably did not run.

**Boss test:** passed, on the pipeline's second pass. The inspector rejected pass 1 for,
among others, an instruction to watch for an error that cannot occur (the guard is invisible
in that flow) and an expected result that would have produced a false failure (the note
appears under BOTH Titles and Contents, because its name is its filename).

### The close (PCS)
- **Commits:** 11 this session — §1–§7, the Boss-found library fix, two LAWS, two agents.
- **Pending Jobs `v1.67`** — written, closing §1–§7 and filing PJ-213…218.
- **Orientation `v3.83`** — written as a NEW file (SO#6; v3.82 untouched).
- **MoCh** `docs/MoCh/MoCh-2026-08-03-1200.md`.
- **Handover** `docs/HANDOVER-2026-08-03-pj207.md`.
- **Memory** — `project_pj207_index_repair.md` and
  `feedback_verify_ui_surface_before_naming_it.md` added and indexed.
- **Help / User Manual: deliberately NOT updated.** Nothing user-facing shipped yet — the
  repair still has no door. The docs that say "there is no Rebuild button" are still TRUE
  and become false only when §11 lands the control; correcting them now would be the
  fabrication this session wrote two laws about. §12 owns that pass. Recorded here so the
  omission is a decision, not a miss (SO#2).

### Gates at close
Rust **1355 / 0** · vitest **900/900** (76 files) · svelte-check **0 errors** · i18n **15/15 ✓**.

### What the next session should know in one line
§8 is next, and the Architect doc's §8 correction is load-bearing: scoping only the walk does
**not** close Charter W2-9, because the boot reconcile re-adopts from the recursive set —
both passes must go through `own_libraries_for_root` or foreign-copy removal oscillates forever.
