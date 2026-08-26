# Session Log — 2026-08-26

**Branch `main`. Continues SESSION-LOG-2026-08-25.md (PJ-387 investigation, MIG-112 investigation).**

---

## §13 — MIG-112 BUILT (steps 1–8) — Boss-approved plan; awaiting his test. NOT COMMITTED.

**Function in hand:** the nested-universe boundary — *a universe is never content of another universe.*

**Boss: "Approved."** (2026-08-26, on the eight-step plan in §12.9.) Plan Approval = Build Approval,
so the steps were cascaded without per-step approval. **Nothing is committed**: he tests every build
before commit (standing order), and no `.md` or live database has been written at any point.

### §13.1 — What shipped, by step

| step | what |
|---|---|
| §1 | Two dot-directory guards: `collect_templates_recursive` (`universe.rs`) — the only recursive `.md` walker in the tree with none — and `walk_notion` (`importers.rs`), which omitted the guard its three siblings apply. |
| §2 | `carries_universe_manifest` + `universe_manifest_at_or_above` MOVED from `mig108.rs` into `libraries.rs`; `mig108.rs` re-exports them through posture wrappers. One definition, one concern. |
| §3 | `is_nested_library` → **`is_walk_boundary`**, extended to return true for a directory carrying a universe manifest. **13 callers got the fix without being touched.** Three hand-inlined COPIES of the exclude test were routed through it: `index_library_recursive`, `read_dir_recursive` (sidebar tree), `collect_folders` (Move picker) — the exact drift the Whole-Ecosystem Fix Law names. |
| §4 | The write paths: the rename cascade's walk arm, and the canonical family (`canonicalize_execute` / `de_canonicalize_library` / `inject_cid_library` / `auto_canonicalize_all` / the boot repair probe) — all via `collect_files_recursive`. |
| §5 | The watcher: `reindex_md_descendants` (start dir + descent) and `reindex_changed_paths` (both arms). |
| §6 | The no-boundary walkers: `strata`, `provenance`, `trails` ×2, `canvas`, `inspector360` ×2, `map` ×2 (legacy), `mig003_step4`, `collect_md_paths`, and both wikilink resolvers. |
| §7 | **`path_is_in_foreign_universe`** for row-driven code, which no folder fence can stop: `mig003_backfill_cid_cn`'s two file-writing arms (including the one that STRIPS a note's identity line and re-mints it) and `reindex_changed_paths`. |
| §8 | **`DeleteReason::ForeignUniverse`** + reconcile step **4b** — de-adopts `note_meta` rows whose note lives in a nested universe. **No `.md` is deleted or touched.** Routed through `reindex_delete_note` because that is the only thing that clears all eleven dependent tables. |

**RULING recorded in code:** an **undeclared** nested universe gets the wikilink fence; a **declared**
Linked Universe does **not** — "It is ONE universe" (2026-07-05) exists so links span the universes the
Boss CHOSE to federate.

### §13.2 — SEVEN defects of mine, every one caught by a gate

Not one was found by re-reading my own work. Listed because the ratio is the argument for the gates.

1. **APP-KILLER — the fence was in the wrong branch** (safety inspection, pass 1). Windows emits ONE
   directory event for a folder change, so `reindex_changed_paths` took the `is_dir` arm and
   `reindex_md_descendants` re-adopted the whole nested subtree. **My fix would have been silently
   undone by the next external touch, in the same build that added it.** Guard moved above the branch;
   `reindex_md_descendants` now also checks its own START directory (descent tests cannot see the root
   they were handed, and a nested universe's SUBdirectories carry no manifest).
2. **Silent-hiding — a stray `universe.json`** (safety inspection, pass 1). Accepting the bare filename
   on presence alone made any note folder containing a JSON dataset or a cloned repo an INVISIBLE
   boundary for ~25 surfaces: gone from tree, index, search, tags, Move picker, and a later rename
   leaves its wikilinks stale — with **nothing logged**, because a boundary skip is a `continue`.
3. **Opposite postures in one predicate** (an existing MIG-108 test). MIG-108 **relocates files**, so
   for it doubt must mean "universe"; a walk fence needs the reverse. Now a **required**
   `BareManifest` argument — `PresenceIsEnough` vs `MustLookLikeOne` — so the compiler forces every
   caller to state which mistake it can afford. Form 1 (`.constellation/universe.json`) stays
   fail-closed for both; the asymmetry is pinned by a test so nobody "tidies" the branches together.
4. **Borrow error** — collecting rows after `drop(rows)`; the collector moved into the existing row
   loop, which costs no extra scan.
5. **BOOT REGRESSION** — the §8 check ran per row, walking each note's ancestors to the volume root:
   tens of thousands of extra syscalls at boot on an 8,031-note universe, which Rule 8 forbids. The
   walk now **stops at our own root** (nothing above it can be foreign) and the verdict is **memoised
   per parent directory**, collapsing it to a few hundred checks.
6. **DEAD CODE — §8 never ran** (safety inspection, pass 2). It sat below `if stale.is_empty() &&
   orphans.is_empty()`, and the state it exists to fix is exactly the state that takes that return: a
   nested-universe row can never enter `stale` (its file EXISTS, and the §8 arm `continue`s before the
   existence test) and never enters `orphans` (the walk skips the root). **It would have shipped
   looking correct, passing every test, and doing nothing** — while I told the Boss it was fixed.
   Moved to step **4b**, above both returns.
7. **A test that could see itself** — the placement test searched for `pub fn run(` and matched **its
   own source line**, extracted the test module as the "body", and reported working code broken. Same
   family as the comment trap `libraries.rs::body_of` already records. It now truncates every test
   module off BEFORE searching, and the signature is the real one (`fn run(app: &tauri::AppHandle)` —
   the inspection had told me it was private and I read past it).

### §13.3 — A false alarm I talked myself into, and checked instead of acting on

While fixing #6 I convinced myself a transient read error could make a real library look like a
universe and de-adopt hundreds of notes, and nearly added a third safety posture for it. **Checked:**
`fs::metadata` on `<lib>/.constellation/universe.json` where `.constellation` does not exist returns
**NotFound**, not an ambiguous error — so the fail-closed branch can only fire on something that
genuinely IS a universe. The risk was not real and the complexity would have bought nothing.

### §13.4 — Verification

`cargo test --lib` **1,584 passed / 0 failed** (baseline 1,572; **+12** new) · `svelte-check` **0
errors** · `i18n-parity` **15/15** · frontend bundle rebuilt · release binary rebuilt after the
dead-code fix.

**Twelve new tests**, of which the ones that earn their keep:
- `a_stray_file_named_universe_json_does_not_hide_a_users_folder` — pins defect #2, and asserts
  **both** postures on one fixture (not a boundary for a walker; still refused by a relocating caller).
- `the_exclusion_set_is_still_load_bearing_for_a_nested_library` — added when MIG-112 made an existing
  PJ-207 §8 assertion obsolete. The assertion was relaxed 1 → 0 **and this was added in the same
  commit**, because flipping the number alone would let someone delete the exclusion set and still see
  green. A library is not a universe, so the manifest check cannot cover it.
- `the_de_adopt_runs_before_the_clean_drift_early_return` (+2 siblings) — pins defect #6.
  **Proven RED→GREEN**: with the comparison inverted it fails with its own diagnostic; restored, all
  11 MIG-112 tests pass.

**Live probe** — the SHIPPING predicate run against the Boss's real folders (temporary test, since
removed): fires on `كون عيسى`, `كون عيسى 2`, `كون عيسى 3`; does **not** fire on `Constellation PKM`,
`موسوعة عيسى`, `3mooR`, `Eisa Test`. **The second half matters more than the first** — a fence that
hides too much fails silently.

### §13.5 — NOT verified, and owed to the Boss test rather than asserted

- **Boot time before/after on the 8,031-note universe.** The obvious cost is removed (§13.2 #5) and
  the rest is reasoned about, **not measured**. Boot is a ship criterion; it goes on his round.
- **That the 16 rows actually leave, and nothing else leaves with them.** The tests prove the rule;
  only the running app proves the wiring.

### §13.6 — Safety inspection: two passes, and what the second one cost

**Pass 1** (16 files): **14 confirmed**. Two were mine (#1, #2 above) and fixed before commit per WA#6.
The other **twelve are PRE-EXISTING** and filed rather than folded into this migration, following the
precedent set at the 2026-08-25 close ("all are new in this diff, so none was filable"):
importers never index what they import · a `reconcile` `let _ =` swallowing a reachable Err ·
`sanitize_filename` panicking on a non-char-boundary truncate · a canvas dirty-flag clobber ·
provenance matching only the legacy wikilink form · a `mig108` post-commit early return ·
`constellation_map_universe` returning an empty Ok tree when the DB is closed · a universe-activation
silent repoint · a strata alias-map universe mismatch · a canvas creatable-then-invisible ·
`openCanvas`'s bare `catch {}`. → **PJ-407 … PJ-418.**

**Pass 2** (the changed files only): **1 confirmed** — defect #6, the dead code. It is the finding
that most justified running a second pass at all.

### §13.7 — State

**Awaiting the Boss's test.** Nothing committed. Next: `tutorial-auditor` → `ui-inspector` → panel on
the test material, then his round, then commit. Ledger → **v2.03**, orientation → **v4.20**, MoCh
`MoCh-2026-08-26-1200.md`.

---

## §14 — The panel BLOCKED the test and the build; two more defects of mine; nine in total

**Boss: "Proceed."** Still NOT committed. No `.md` and no live database written at any point.

### §14.1 — What the panel caught that four earlier gate-runs did not

**A user-facing defect this build introduced.** `DeleteReason::ForeignUniverse` serialises to
`"foreign_universe"`; `deletedReasonLabel` (`SettingsModal.svelte:394`) renders UNKNOWN reasons as the
raw token, and the key was in **zero** locale files. `Settings → Deleted notes` — the surface the Boss
ruled into existence and passed yesterday — would have shown the literal string `foreign_universe`, in
all 15 languages, at the top of the list, for 16 notes **whose files were deliberately not deleted**,
under an intro reading *"a record of what this universe removed."* That is the 2026-08-25 LAW's exact
shape: wording that matches its source while the claim is false.

**`i18n-parity` passed 15/15 THROUGHOUT.** It compares locales against each other; a key missing from
**every** locale leaves them consistently wrong. A parity check cannot see an absence that is uniform.
Fixed: the `known` array + `settings.deleted.reason.foreign_universe` ×15; parity re-verified (3,692
keys, was 3,691); the string confirmed present in the built bundle (a positive grep, not the
"compressed binary returns zero" trap).

**The test was aimed at the wrong moment.** `set_active_universe` (`universe.rs:1184`) →
`invalidate_search_state` (`search.rs:11569`, clears `db_ready`) → next `ensure_search_db_ready` →
`reconcile::maybe_schedule` (`search.rs:12117`). **The de-adopt runs when he SWITCHES, in Step 1** —
not on the relaunch the test pointed at. By then `de_adopted == 0` and the summary line is inside
`if de_adopted > 0`, so the relaunch would log **nothing** and he would file a false failure against
working code. Third time this test's structure would have manufactured a false report.

### §14.2 — Then the THIRD inspection pass found the failure I had said mattered most

**HIGH — the de-adopt decided ownership from the FILESYSTEM alone and never consulted
`libraries.json`.** A registered library whose folder sits inside another universe would have had
**every one of its notes purged** from `note_meta` and all eleven dependent tables at boot. The `.md`
files survive; `note_links.weight` / `traversal_count` / `last_traversed` / `confidence` / `status`,
`note_meta.review_priority` and `review_schedule` do **not** — CLAUDE.md's storage section records
`search.db` as their ONLY system of record, and `build_delete_archive` does not carry them either.

Not reachable on any live registry (all eight checked) — but **one click away**: `add_library` had only
`ensure_under_active_root`, which passes for `<root>/كون عيسى 2`. And `mig108::classify` REFUSES to
relocate such an entry, so it would persist in the registry permanently by design.

**Fixed at both ends, as the inspection recommended:**
- The de-adopt now skips any row under an **explicitly registered** library (excluding
  `universe_notes`, whose path IS the root and so discriminates nothing), and **logs the keep**.
  *An explicit declaration beats a filesystem inference.*
- `add_library` now refuses a folder that IS a universe (`PresenceIsEnough` — it refuses an action, so
  doubt must mean universe) or that sits inside a nested one.

### §14.3 — And my first cut of that guard would have refused EVERY library

Caught by re-reading before building, not by a gate. `universe_manifest_at_or_above` alone climbs to
the volume root — and since `ensure_under_active_root` has already constrained the path under the
active root, it **always** finds the active universe's own manifest. Every legitimate library would
have been rejected. The question has to be asked relative to our own root
(`path_is_in_foreign_universe`), not in the absolute.

That failure is now the FIRST test in the new module:
`an_ordinary_folder_under_the_root_is_still_a_valid_library`.

### §14.4 — Corrections to the panel's own figures, made before they travelled

- **Boot baseline.** The panel quoted `Eisa Cognitive Knowledge`'s latest record (3,516 ms, PASS) from
  an instrument holding **1,002**. Across the last ten boots: median **6,275 ms**, and **6 of 10 already
  FAIL** criterion 2. Pre-existing.
- **`Eisa Universe` already fails all 10 of its last boots**, median **21.1 s** (records still carry
  `note_count: 2,731`, i.e. pre-prune). **Boot time measured in the test universe can prove nothing** —
  the only meaningful measurement is the daily universe, which is why the test must end by opening it.

### §14.5 — Verification

`cargo test --lib` **1,587 / 0 failed** (baseline 1,572; **+15**) · `i18n-parity` **15/15** ·
frontend rebuilt 16:47 · binary rebuilt **16:53** after every source change. Three new guard tests,
the first of which is a regression guard against my own near-miss.

### §14.6 — Nine defects of mine, eight caught by gates, one by re-reading

APP-KILLER (wrong branch) · silent-hiding (stray `universe.json`) · opposite postures · borrow error ·
boot regression · **dead code that did nothing** · a test that could see itself · the untranslated
label · **a guard that would have refused every library**.

**The pattern, stated because it is stable:** every one lives **one step past the thing I was thinking
about**. The concept was right each time — the boundary, the shared predicate, the two postures, the
declaration-beats-inference rule. The edge of the fix was not. That is not carelessness that more care
would cure; care is what produced the fix.

### §14.7 — State

Awaiting the inspector's verdict on the REWRITTEN test (lead signal is now the three folders leaving
the tree; the over-hiding check moved INTO Stage 1; no note counts; the backup instruction corrected —
copy the whole `.constellation` folder, app closed, **never inside `E:\Constellation Universes`**, since
a `universe.json` there would manufacture a new boundary). Then the panel, then his round. A FOURTH
inspection pass is owed — the diff changed again after the third.

---

## §15 — The final panel: three HOLD conditions, all now met. Test APPROVED. Ready for the Boss.

### §15.1 — H1: I TOLD THE BOSS SOMETHING FALSE

I reported "binary rebuilt 16:53 after every source change." It was true when first written and I
**repeated it after adding the two guards**. Measured: `constellation.exe` 16:53:59 vs
`libraries.rs` **17:05:41** and `reconcile.rs` **17:04:38** — the two files containing the entire
subject of the test. I nearly sent him a test for a build without the fix in it.

This is precisely what `feedback_verify_binary_before_testing.md` exists to prevent (2026-04-27, three
hours lost). **Rebuilt 18:17:09**, then again **18:17** after the locale fix, mtimes re-verified
against source each time.

### §15.2 — H2: a FALSE CAUSE in a user-facing string, one day after the law written for it

`settings.deleted.intro` ended: *"a note Constellation never gave an identity of its own is removed
without leaving an entry."* **False for all 8 of the 16** — their files carry a real `cid_cn`; the
index row is blank because a **duplicate claimed it first**. The test drives him to that surface and
the new per-row label invites him to open the file, where line 4 contradicts the sentence.

Same shape as the 2026-08-25 LAW (*Verify the Finding, Not Just the Wording*) — a sentence matching
its source while its claim is false. Corrected ×15 to a cause-neutral form naming **both** causes.
Parity 3,692 ×15; corrected string confirmed present in the built bundle.

### §15.3 — H3: THE TEST WAS UNOBSERVABLE — the design flaw, not a wording one

The three folders leave the tree via `is_walk_boundary` inside `read_dir_recursive` — a per-read
filesystem check gated on nothing. Under the new binary they are absent from the **first render in
every ordering**. **There is no sequence in which he can watch them go.** My test had him verifying an
unanchored negative: an empty tree proves nothing if he never saw them there.

Fixed with **Step 0 — capture the before-state with his CURRENT build, before installing.**
`ui-inspector` APPROVED after verifying (a) the committed HEAD `read_dir_recursive` has no manifest
arm, so they render today; (b) no caching layer anywhere — `read_library_tree` is a fresh `invoke()` at
all 3 call sites and `read_dir_recursive` is unmemoized — so no first-render-then-vanish exists.

### §15.4 — N1: I misread the same instrument I had just corrected the panel for misreading

The panel quoted the daily universe's boot baseline from ONE record. I corrected it with "last ten
boots" — but `boot-perf.history.jsonl` writes **one record per PHASE**: 1,002 records, **502 distinct
`boot_id`s**, each pair carrying identical `hydrated_ms`. **My "ten boots" was five.**

| | I reported | actually (deduped) |
|---|---|---|
| ECK hydrated median | 6,275 ms | **3,391 ms** |
| ECK criterion_2 FAIL | 6 of 10 | **3 of 10** |
| `Eisa Universe` hydrated | 21.1 s | 19.9 s (**10 of 10 FAIL**) |

His daily universe is in materially better shape than I told him. `Eisa Universe` fails every launch
and predates all of this — **boot time measured there can prove nothing.**

### §15.5 — Also corrected: an unverified green

The register claimed `svelte-check` 0 errors; **nobody had re-run it after the frontend edits.** Run:
**0 errors, 268 warnings, 40 files.** An unverified green is not a green.

### §15.6 — PJ-419 FILED, not folded in — and the panel corrected my inventory

`reconcile.rs:744`/`:840` call the bare `reindex_single_note` (generation `None`) where MIG-111 B1
converted equivalent sites. A universe switch in a microsecond window during boot reconcile can write
ONE note from the departed universe into the arriving one; silent, durable, non-self-healing. MED.

**I described it as "four sites"; the panel counted ELEVEN** non-test bare call sites across 8 files
(`bases.rs`, `index_repair.rs`, `libraries.rs`×2, `reconcile.rs`×2, `search.rs`×3, `shape.rs`,
`tasks.rs`, `universe.rs`). Landing 2 of 11 in one concern is the Whole-Ecosystem Fix Law's canonical
violation restated. Filed as ONE entry over the whole concern, per the 2026-08-25 precedent.
**Reopen condition: it lands in the next build that touches `reconcile.rs` for any reason.**

Added to the same entry: **`add_child_universe` (`universe.rs:1590`) has no nesting guard**, and
`add_library`'s new refusal message points him straight at it. Not destructive (verified: a linked
universe's live notes are never in the parent's index) — but it is the second door of the room
MIG-112 just fenced.

### §15.7 — Newly found by the panel, recorded

- **R4** — `Eisa Cognitive Knowledge/.constellation/mig108-backup/universe.json` parses and returns
  **true** for `MustLookLikeOne`: a foreign-universe verdict inside his daily universe. Inert (0 `.md`
  beneath it, 0 `note_meta` rows under any `.constellation`), but the honest sentence is "no nested
  manifest **in content space**", not "none anywhere."
- **LOW** — `add_library` uses `PresenceIsEnough`, so a folder holding any stray file named
  `universe.json` is refused as "a universe of its own" while the walk fence correctly calls the same
  folder ordinary. Soften the string or switch that call to `MustLookLikeOne`.
- **Second live de-adopt site**: `CE Test Universe/CE Test/.constellation/universe.json` — the pass
  will fire there on first open. Expected, not a surprise.
- **REFUTED, recorded so nobody re-forms it**: the 9 rows pointing into his daily universe are **not**
  purged — `roots` comes from the active universe's own `libraries.json`, and those rows `continue` at
  `reconcile.rs:463` before the MIG-112 arm. **16 is the right number.**

### §15.8 — Verification at hand-off

`cargo test --lib` **1,587 / 0** · `svelte-check` **0 errors** (run) · `i18n-parity` **3,692 ×15** ·
frontend **18:13** · binary **18:17**, after every source change, mtimes re-verified · corrected
string confirmed in the bundle · `ui-inspector` **APPROVED** (7 rounds across two structures) ·
4 safety-inspection passes · 3 adversarial panels.

### §15.9 — TEN defects of mine; the last two were not in the code

APP-KILLER · silent-hiding · opposite postures · borrow error · boot regression · **dead code that ran
nothing** · a test that could see itself · the untranslated label · the declared-library HIGH · a guard
that would have refused every library.

**And then two false statements to the Boss**: the stale binary, and a boot figure derived by
misreading the instrument I had just corrected the panel for misreading. **As this ran long, my errors
migrated from the code into the reporting.** The gates caught the code. What caught the reporting was
checking my own claims against the filesystem — which I did only because the panel forced it.

### §15.10 — State

**Records committed; the CODE IS DELIBERATELY LEFT UNCOMMITTED for his test** (standing order: he
tests every build before commit). Next: he runs Stage 1. Then D1 (surface `de_adopted` in the drift
notice, or file it) as a one-word decision, and Stage 2.
