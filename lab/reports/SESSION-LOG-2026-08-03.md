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

## §10 — Boss ruling 2026-08-03: surface a link's age

Asked at the §1 pass: *"I want the link's age to be surfaced."* Filed as **PJ-213** rather than
built inline — it is a new user-facing feature, not part of the approved 15-step plan, and doing it
properly is its own small job: the Outgoing Links **and** Backlinks panels (Whole-Ecosystem — they
are the two hosts of `ConfidencePicker`), a date-format decision, i18n ×15, and RTL. It also lands
in a file with a confirmed open inspection finding (`ConfidencePicker.svelte:61`).

Context that makes it worth doing: `created` is one of the eight Living-Link properties and the
basis of weight decay, it is **not** in the earned ledger, and until §1 shipped today it was being
silently reset by ordinary edits. Surfacing it makes that class of loss visible to the user instead
of only to a test.
