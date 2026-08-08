# Session Log — 2026-08-07

**PJ-207 §8 — the index stops adopting notes that belong to a linked universe.**
Branch `main`. Working tree at session start: clean, head `7886d4c0`.

---

## 1 · Reproduce-First — the defect, before a line of code

**Charter W2-9 is real, and it is in the Boss's own data — no fixture required.**

Read from disk 2026-08-07 (`.constellation/libraries.json`, `.constellation/universe.json`,
`search.db` byte-copies queried read-only):

| Universe | own libs | live cUniverse children | rows | **rows owned by a LINKED universe** |
|---|---|---|---|---|
| `كون عيسى` (**active**) | 1 | 0 — its one declared child directory **does not exist** | 5 | **0** |
| `Eisa Universe` | 6 | **2** (`كون عيسى`, `Eisa Cognitive Knowledge`) | 1,890 | **13** (all 13 files still on disk) |
| `Eisa Cognitive Knowledge` | 27 | 0 | 8,021 | 0 |

`Eisa Universe` also holds **612 rows under no current library root** — a *different* defect
(libraries de-registered while their rows stayed). Explicitly NOT touched here:
`reconcile.rs`'s `if !roots_norm.iter().any(|r| under(&pn, r)) { continue; }` skips them, so
they are never stale candidates. This is the plan's Landability-F11 refutation, confirmed live.

**Then fired on demand**, in Rust, driving the *production* walk (`index_library_recursive`
via the `WalkCtx { app: None }` its own doc reserves for this) over roots from the *real*
recursive resolver: the linked universe's note lands in this universe's index. Same walk over
the own set: it does not.

### An answer the plan asked for and could not have
Plan §15 lists as an open item: *"whether any cUniverse root is nested under the active
Universe root."* **Answer: no.** Both federated parents on this machine declare children that
are **siblings**, not nested. §8's exclusion set is therefore defensive today — and is kept,
because it is the difference between a fix and a fix-shaped thing the moment that changes.

### A GUI reproduction was attempted and abandoned — stated, not hidden
Built a parent+child fixture universe pair, backed up the Boss's registry, pointed it at the
fixture and launched the release binary twice. The app ran but never created a `search.db` in
the fixture; the window could not be seen (masked). **Abandoned in favour of the Rust
reproduction**, which is strictly better for this defect: the mechanism is a Rust-side root
choice, and the Rust harness exercises the real production functions on demand.
Registry restored byte-identical (diffed); fixture universes deleted.

---

## 2 · The Whole-Ecosystem sweep — four surfaces named, six fixed, twenty-four found

The plan's §8 names four surfaces. Per the Whole-Ecosystem Fix Law ("do not enumerate from
memory"), a 7-angle / 64-agent adversarial sweep enumerated every index-write path scoped by
the federation-recursive library list. **57 candidates → 24 unique CONFIRMED**, each verified
by an independent refuter whose default verdict was REFUTED.

**Six are the automatic-adoption class §8 owns. All six are now scoped** — via
`libraries::try_load_libraries`, the strict loader that already existed for exactly this
("absent is a FACT; unreadable is an unknown"), so §8 introduced **no new loader**:

| # | Surface | Was | Note |
|---|---|---|---|
| 1 | `search.rs` `reconcile_filesystem` | recursive roots | + foreign-root skip set; loaded **before** the trigger window opens |
| 2 | `reconcile.rs` `run` | recursive roots **and** the orphan re-adopt walk | the half that made removal oscillate |
| 3 | `index_repair.rs` `submit` / `run_cold_start` | recursive membership check | refused **at the door** with a new typed `Foreign` outcome |
| 4 | `search.rs` `reindex_changed_paths` | recursive | the watcher — re-adopted **immediately**, no boot needed |
| 5 | the boot cold-start fan-out (`+layout.svelte`) | iterates `$libraries` (recursive) | closed server-side by #3 |
| 6 | `library_attribution_backfill.rs` `run` | recursive | automatic boot pass that UPDATEs `note_meta.library_name` |

Two of these (#5, #6) are **not in the plan**. #5 matters most: the boot fan-out submits one
`ColdStart` per entry of the frontend's library list, and that list is the recursive set — so
a federated universe re-offered every linked library for indexing on **every boot**.

`Foreign` is deliberately distinct from `Blocked`: `Blocked` means "not now" and the runner's
drain re-offers it, which would loop forever on a refusal that is "not ever, by this universe".
It is also checked **before** the single-flight flag is taken and before the worker starts —
the old registration check lived inside the worker, where a refusal would have landed in
`last_error` and emitted `ok: false`, making an ordinary federated boot look like a failed repair.

### The other 18 — the user-action class, ruled on, not parked
`validate_path_in_any_library`'s first branch is the **recursive** set, and its doc says so:
*"including child universe libraries."* Editing a linked universe's note from the parent is
**deliberate, designed behaviour** — so the save path (`constellation_search_reindex` →
`reindex_single_note`), `create_note`, `rename_item`, `move_item_db_tail`, `delete_path`,
`update_links_on_rename` and the rest still write a row here when the user acts on a linked note.

§8 closes the **automatic** doors. It does not close the you-edited-it-yourself door, and it
cannot without deciding where that write should go instead — routing it nowhere would leave an
edit findable in no index at all until that other universe is opened.

**Boss ruling, 2026-08-07: ship §8 as scoped; file the rest.** → **PJ-219**.

---

## 3 · What the tests actually pin, and what they do not

**The reproduction test** (`search.rs::tests_pj207_s8_index_write_scope`) drives the real walk
over both root sets in one test: recursive → the foreign row appears; own + exclusion → it does
not. Plus the nesting case, the zero-children case (own set **==** recursive set, byte-for-byte
— the live case that must not regress), the strict-loader contract, and separator-bounding.

**Two mirrors were removed rather than written.** `walk_exclusions` and `foreign_roots_of` are
shared by production and test, so the test cannot keep passing after production changes them —
the `search.rs:338` mirror trap §1 deleted, not re-introduced in a test. What remains mirrored
is two lines of `for lib in libs { index_library_recursive(...) }`, and that is stated in the
test's own module doc rather than left for a reviewer to discover.

**The gap, closed structurally.** Those tests pin the *mechanism*, not the *wiring* — every
entry point takes an `AppHandle` and this crate has no Tauri test harness (`Cargo.toml` carries
no `tauri`/`test` feature). Reverting `try_load_libraries` → `load_all_libraries` at any call
site would have left the whole suite green. §6's `ConvergeKey` token does not fit (the parameter
is a plain `&[LibraryInfo]` shared with a dozen legitimate read paths), so the invariant is
asserted against the source: `tests_pj207_s8_write_scope_guard`. Whole-module for the three
write-only modules (after §8 the count is **zero**, and zero cannot rot the way a line number
does); function-scoped for `search.rs`, which uses the recursive set legitimately elsewhere.

**RED proven twice, both observed:**
- Revert `foreign_roots_of` to an empty set → `a_linked_universe_nested_inside_the_active_root_is_skipped_not_merely_unlisted` fails: `left: 1, right: 0`.
- Revert `reconcile_filesystem` to `load_all_libraries` → the wiring guard fails, naming the line: `Offending line(s): ["    let libraries = crate::libraries::load_all_libraries(app);"]`.

The committed tree is green (§1's discipline: demonstrate red locally, never commit a red tree).

---

## 4 · A HIGH the safety inspection found, fixed in this commit

`reconcile::run` had **no guard of any kind** — verified by grep, not assumed. It runs on a
spawned thread, computes its roots / stale set / orphan list from the universe active at start,
then writes through `state.db`, which a universe **switch** replaces underneath it. Its re-adopt
tail would index the **departed** universe's `.md` files into the **newly-active** universe's
index: Charter W2-9 through a second door, and a direct breach of Architect Invariant 10.

§7 built `federation_generation_now` and `walk_may_proceed` for exactly this and wired only the
bulk walk to them. The boot reconcile now takes the **same shared decision** (not a second copy)
at start, before the write phases, and inside both the removal and re-adopt loops — per-iteration,
because a capped sweep is up to 200 deletes. Guarded by
`the_boot_reconcile_checks_the_universe_generation_before_it_writes`.

---

## 5 · Gates

Rust **1364 / 0** (13 ignored; baseline 1355 — +9 new) · svelte-check **0 errors** ·
i18n parity **15/15 ✓** · vitest **895/900**.

The 5 vitest failures are all `expected 19.11 to be less than 16`-shaped Sight-v6 timing
assertions (PJ-172). The failing SET differed on each run (1, then 3, then 5, different
traditions) and they were competing with 14 audit agents for CPU; **re-run in isolation:
31/31 pass**. §8's diff is Rust plus one TypeScript type — it cannot reach Sight.

Release binary rebuilt **13:48** from source last touched **13:40**.

---

## 6 · The whole-app safety inspection — 38 confirmed, §8 introduced NONE

Ran (interrupted by a session restart, resumed from cache; 69 agents, 0 errors).
**It went whole-app again despite a real `args.files` array — see PJ-220.**

**Every one of the 38 was cross-referenced against this commit's actual diff hunks.** None
falls inside them — e.g. `store.ts:7221` vs my only store.ts hunk at 3724-3730;
`index_repair.rs:452` vs my 551-559; `reconcile.rs:287` vs my 91-107. All 38 are **pre-existing**,
surfaced because the sweep ignored its diff scope. The one that lands inside §8's own concern
*and* inside a function this commit edits was fixed here (§4 above); the register goes to the
Charter and the ledger.

Two APP-KILLERs, neither new, both already known or now filed:
- `bases.rs:796` `format_yaml_value` — a weaker quoter than the TS twin it shadows; a Bases cell
  beginning `- `, `|`, `>`, `%`, `@` or a backtick writes frontmatter that no longer parses, after
  which **every later property edit on that note is silently discarded**. → **PJ-221**.
- `store.ts:7221` `loadWorkspaces` — the already-escalated triage item #11 (universe A's
  workspaces latched into universe B). Unchanged, still open.

---

## 7 · Filed this job (SO#9)

- **PJ-219** — the user-action write class (Boss-ruled 2026-08-07 to file, not fix). Needs a
  design decision: when the user edits a **linked** universe's note from the parent, where does
  the index write go? Today: into the parent. 18 confirmed surfaces.
- **PJ-220** — **PJ-166, twelfth strike, cause narrowed.** My first diagnosis (the skill template
  passes `args` as a JSON string) was **wrong** — I passed a real array via `scriptPath` and it
  still ran whole-app (14 agents = exactly the 14 `WHOLE_APP` groups). Verified: the script's own
  gate `Array.isArray(args.files)` is **correct code**, so `args` are not reaching the workflow at
  all. Separately: the `{name: 'safety-inspection'}` form is **blocked outright** by a permission
  handler ("script contains control characters") — only `{scriptPath}` launches, and the file
  itself contains **no** control characters.
- **PJ-221** — `bases.rs:796` frontmatter quoter (APP-KILLER, above).
- **PJ-222** — `collect_md_paths` has no library-boundary notion at all; §8 filters its output in
  `run_cold_start` instead of fixing the collector, because its four other callers are folder-move
  paths with different semantics. Worth unifying.

**Charter:** W2-9 moves to *partially closed* — the automatic-adoption half is closed by §8
(evidence: this commit + the six surfaces above); the user-action half is PJ-219. It is **not**
marked closed, because §13's removal can still be undone one note at a time through PJ-219's door.

---

## 8 · Boss test — PASSED, on `Eisa Universe`

The Boss corrected the target mid-session: **`Eisa Universe`, not `كون عيسى`** — the right call, because it is the one universe here with **live** linked children (`كون عيسى` + `Eisa Cognitive Knowledge`), so §8's behaviour is real there rather than merely non-regressive.

**A correction to my own reading, worth recording.** I twice told him things about his setup that the registry contradicted. The registry at `world.uconstellation.app/universes.json` still lists only `كون عيسى` and has not changed since 09:56 — yet the 18:33 session demonstrably booted into `Eisa Universe`: its `boot-perf.latest.json` carries `note_count 1890` at 18:33:53 and the diagnostics log shows `[federation-prewarm]` attaching **both** children. **How the active universe reached `Eisa Universe` without that file changing is unexplained** — I searched every `universes.json` on the machine (two exist; neither was written at 18:33). It is not blocking and it is not §8's, but it is unresolved and should not be quietly forgotten.

**Result — both steps passed, and the load-bearing measurement is mine, not his:**
- **Linked-universe rows: 9 before his session → 9 after.** A full boot of a federated universe adopted **zero** new foreign notes. Before §8 that boot submitted a cold-start for every federated library.
- **`vandrasil`, typed into Notepad while the app ran, was indexed under the correct OWN library** — `[الكون المعرفي] …\وظيفة مجلد الذرات.md` — through `reindex_changed_paths`, the surface §8 re-scoped. An in-app save could not have proven this: `write_gate::atomic_write` marks the path in `watcher_suppress` before writing and `watcher.rs` filters it, so an in-app save is invisible to the watcher **by design**.
- Total rows unchanged at 1,890. `Atlas` returned 103 matches across 98 notes.

**Two findings surfaced on his live data during the test, neither caused by this build** — see §7 (PJ-223, PJ-224). PJ-224 is the one with teeth: it invalidates a premise **§13** is built on.

## 9 · Worktree prune — and the unsaved work it nearly took with it

Boss-authorised a prune of the stale worktrees under `.claude/worktrees/`. `git log main..<branch>`
reported **zero unique commits for all seven**, which is exactly the reading that would have made a
blind `--force` removal look safe. It was not: **two of them hold real work that exists nowhere else.**

- `eager-turing-493c70` — **23 modified Sight v6 tradition files** (`+139 / −111`) **plus two
  untracked concept papers**: `Constellation-Sight-Concept-Paper-v4.1.md` and
  `Constellation-Sight-Subsystem-Concept-Paper-v1.0.md`.
- `sweet-jackson-2fbff3` — two untracked mockups, `Sight-vNext-MockA-Dashboard.svg` and
  `Sight-vNext-MockB-Metaphor.svg`.

Both **kept**, and flagged in the handover for a Boss ruling (land or discard deliberately). Removed
only the four whose entire diff was an untracked `dev/null` artifact: `angry-knuth-ed8072`,
`crazy-pascal-8ce11c`, `frosty-stonebraker-75c9bf`, `suspicious-wright-ebc3fa`. The session's own
worktree was left in place (the running shell resolves to it), as was the clean detached
`Constellation-wtSC`.

**The lesson, and it is the same one as three times earlier today:** the commit graph is one
artefact, and it said "empty". The working tree said otherwise. *Look at the target before deleting
it* is not a formality — here it was the difference between a tidy-up and losing a subsystem concept
paper.

## 10 · Not done in this job, deliberately

- The Boss tested and passed before commit (standing order).
- The test tutorial went `tutorial-auditor` → `ui-inspector` and came back **REJECTED**, correctly:
  the draft's pass/fail step exercised **nothing this build changed**. An in-app save reaches
  `reindex_single_note`, which is not in the diff — and it cannot reach the watcher either,
  because `write_gate::atomic_write` calls `watcher_suppress::mark` *before* writing and
  `watcher.rs` filters suppressed paths, so an in-app save is invisible to the watcher **by
  design**. The revision routes the test through an **external** edit, which is not suppressed
  and does travel `reindex_changed_paths` — a surface §8 actually re-scoped.
  Two further findings: "three passes" (wrong count) and "six changed files" (seven).
- Library *watching* still covers federated directories (`+layout.svelte:2851` iterates the
  recursive `$libraries`). Left alone on purpose: watching is not writing, the write it feeds is
  now scoped, and narrowing it risks federated display. Named here rather than silently left.

---

# PJ-207 §9 — Constellation notices, after it opens, that notes changed while it was closed

**Status at write time:** built, all gates green, release binary building, Boss test in the
`tutorial-auditor` → `ui-inspector` pipeline. **Not committed** — the Boss tests and passes first.

## 1 · The step, and the one place the plan was wrong

§9's concept: after the app opens, it should notice that notes changed on disk while it was shut,
and say so. "Criterion 4", specified 2026-04-15 in `lab/boot-perf/BOOT-BUDGET.md:101`, never built.

The plan (`docs/PJ-207-Index-Repair-Plan.md:234-248`) specified a **new command in
`index_repair.rs` that stats every `.md` under the own roots**. Verified against the code before
writing anything, and that is a second walker: **`reconcile::run` (`reconcile.rs`) already walks
exactly those roots on every launch**, scheduled from `ensure_search_db_ready`
(`search.rs:11033`) — which is the first statement of the very `cache_mark_search_ready` call the
plan wanted to attach to. It already computes **two of §9's four counters** (`stale` = rows with no
file, `orphans` = files with no row) and reports them to `diagnostics.log` and nowhere else.

So §9 was built as: **the pass that already walks learns to compare timestamps**, and
`index_repair.rs` keeps the command the plan asked for — as a pure read of the published report.

## 2 · Reproduce-First — measured on the Boss's live `Eisa Universe`

Read-only, against a byte copy of `search.db` plus a disk sweep mirroring `collect_md` exactly:

| | |
|---|---|
| `drifted` | **19** (largest 4,407,256 s ≈ 51 days) |
| `missing_from_index` | **825** — 798 of them in `Constellation PKM`, a registered own library |
| `missing_on_disk` | 0 |
| `foreign_rows` | **9** |
| books close | 2,094 files = 1,250 unchanged + 19 drifted + 825 missing ✓ |

825 matches his own `diagnostics.log` verbatim — *"825 orphan files (> cap 200) — skipping
re-adopt"*, logged **four times on 2026-08-07**. **PJ-223 is not an undetected defect; it is an
unreported one.** The recovery correctly refuses on its safety cap and tells only a log file.

**`foreign_rows` had to be counted by PATH, not by `library_name NOT IN (own)`.** Those two differ
by 69x here: 621 rows sit outside the own roots, but only **9** belong to a linked universe — **603
point at `E:\Cognitive Knowledge\...`, the pre-MIG-108 location, where no file exists**, and 9 more
point into the repo. Reporting 621 as "duplicated from linked universes" would have been a
fabrication in fifteen languages.

## 3 · The plan's return shape could not report PJ-223

`{ drifted, missing_on_disk, foreign_rows }` has **no field for a file on disk the index has never
seen** — which is exactly what the 798 are. `missing_on_disk` is the opposite direction (the
codebase's own vocabulary: `reconcile.rs` "phantom" rows vs "orphan files", two concepts, two
remedies). Construct the state PJ-223 actually is with nothing else wrong and all three read **0**:
the app says nothing while 40% of the universe is invisible to search. Added
**`missing_from_index`**, plus `unchanged` / `files_seen` / `rows_seen` so the closing sum is
checkable — the discipline §3 built `WalkTally` for, one step earlier in this same migration.

## 4 · §M6 — the measurement that gates this step, taken in the shipped code

`pj207_s9_drift_cost` (`#[ignore]`d, `PJ207_S9_TREE=<root> cargo test --release ... -- --ignored`).
Warm, on `E:` — which is a **LaCie d2, a USB mechanical HDD**, not the SSD `BOOT-BUDGET.md`
assumed. Nobody had written that down.

| tree | before (`is_dir`, no drift check) | after (`file_type` + drift check) |
|---|---|---|
| 7,964 `.md` | 252–260 ms | **17–19 ms** |
| 2,094 `.md` | 207–219 ms | **17–18 ms** |

**The step that added a per-file comparison made the boot walk ~14x faster.** Two findings drove it:
adding the timestamp comparison costs **+4 to +10 ms on 7,964 files** (proved empirically, not
assumed: one syscall costs ~31 us here, so 7,964 extra stats would have shown as ~250 ms — the
timestamps genuinely arrive with the directory listing); and `Path::is_dir()`, which was ~95% of the
walk, is `fs::metadata` — a handle open per entry. `entry.file_type()` with a symlink fallback
preserves junction traversal bit-for-bit.

The plan's *"160–590 ms"* figure has one un-methodised source and is a **warm-only** number. Cold
first-touch of the same trees measured **3.5–8.7 s**. §M6's threshold was ~600 ms; the honest figure
for §9's cost is **negative — about 200–240 ms cheaper per launch than the walk it replaces.**

## 5 · RED-proofs, all three observed (committed tree green)

1. Revert the comparison → `the externally-edited note is drift` — *left: 0, right: 1*.
2. Count a no-row file as drift (**the plan's own shape** — `existing_mod == Some(m)` is false for a
   missing row) → `and it did NOT change — reporting it as changed is a false sentence`
   — *left: 1, right: 0*. That branch would have told the Boss **825 notes changed** when none had.
3. Remove the publish call → the source guard fires by name.

The wiring guard is source-level for §8's reason: every entry point takes an `AppHandle` and the
crate has no Tauri harness, so a second walker would leave the whole suite green.

## 6 · The `/simplify` pass and the diff-scoped inspection — 9 findings, all fixed before commit

**PJ-220 diagnosed on the way in.** `Workflow({scriptPath})` was rejected with *"script contains
control characters"* — the file had **130 CR bytes** (Python's `write_text` translates newlines on
Windows); the repo's own copy has zero. Written LF-only it launched immediately and ran a genuine
**diff sweep over 6 files** (2 hunt groups), not the whole app. The blocker is CRLF.

From the four `/simplify` agents:

- **`--background-modifier-notice` / `--text-notice` are defined nowhere** — my own bug: the notice
  would have rendered a hardcoded light-mode band in dark theme forever. Worse, the same read found
  `--background-modifier-error` and `--text-error` are **both** `var(--color-red)`: the two shipped
  bars have been **red text on a red background**. All three fixed together with the codebase's own
  tinted-surface idiom (Whole-Ecosystem: one concern, three surfaces).
- `Walk` hand-writes `Default` (its safe `complete` is `true`, so the derive was wrong and ten sites
  had to remember it); `ReconcileOutcome` derives it; `let mut report` → `let`; `drop(rows)`.
- A row whose file turned out to be present is no longer missing — counted (`resurrected`), so the
  notice cannot over-report it.

From the safety inspection (5 confirmed, 4 fixed here):

- **`has_findings` ignored the incomplete-walk case** — a library with one unlistable folder yields
  all three drift counts at zero (its notes were never *seen*), so the report was suppressed and
  **silence is this feature's encoding of "all clear"**. It contradicted the sentence written on
  `walk_complete` two fields above it, and my own test pinned the defect. Fixed on both sides of the
  wire; the test now asserts the opposite.
- **The drift notice was never cleared on a universe switch** — universe A's 825 asserted about
  universe B, permanently in the three cases where B's pass returns no report.
- **A pre-existing unbounded busy-spin in `submit`'s drain** (§7): a re-submit returning `Queued`
  has already been pushed back onto `pending`, but only `Blocked` was collected as deferred — so the
  loop re-drained the same scope and re-parsed `libraries.json` for the whole duration of the next
  run. Silent, because it burns a background worker rather than the UI thread.
- **LOW, filed not fixed:** §8's narrowing means a `note_meta` row pointing at a *deleted* linked-
  universe file is now permanently exempt from dead-row removal (9 such rows here). That is a §13 /
  PJ-219 concern, not §9's.

## 7 · Gates

Rust **1370 / 0** (14 ignored — one is the new §M6 harness) · vitest **900/900** ·
svelte-check **0 errors** · i18n parity **15/15 OK** (four new `indexDrift.*` keys x 15 in the same
commit; placeholder parity is covered by the existing test).

## 8 · Honest open item carried into the Boss test

**I could not verify where the notice physically appears.** `<div class="app">` is a CSS grid whose
four columns are exactly saturated by dock + sidebar + content + right sidebar, and all three notice
rows declare no grid placement. No browser was available to render it (the Chrome extension is not
connected; access to a desktop browser was declined), and I will not guess. The two existing bars
have the identical property, so whatever they do, this does. The test asks the Boss to **report**
where it appears rather than asserting a position.

## 9 - The test pipeline: THREE rounds, four findings, all mine

`tutorial-auditor` -> `ui-inspector` rejected the §9 test **twice** before it reached the Boss.

1. **The marker word was contaminated.** The draft used `zarquon` — which the 2026-08-03 log records
   as already indexed in `Eisa Cognitive Knowledge`, a **federated child of the universe under test**.
   A hit could have come from a pre-existing note rather than from anything this build did, and the
   draft's "you should expect not to find it" would have been false. `vandrasil` is contaminated too
   (indexed under Eisa Universe's own `الكون المعرفي`). Fixed by *proving* a clean marker: a script
   queried the `search.db` of all three universes and read every `.md` on all three disks; six
   candidates came back clean and `plarnwick` was chosen. That turned Step 4 from an unreliable
   assurance into a real assertion — a hit can now only mean Constellation indexed the Boss's edit.
2. **A number from the wrong universe.** The motivating "60 of 7,824" was measured on
   `Eisa Cognitive Knowledge` but presented as "your own data" immediately before Eisa Universe's
   19/825 — inviting him to read 60 as a prediction for the universe he was about to open.
3. **A trailing period inside a quoted button label.**
4. **The wrong icon shape.** The draft said "⋯" — a horizontal ellipsis. The button's SVG is three
   circles at `cx=12, cy=5/12/19` with no transform: a **vertical** kebab. Verified in the markup
   before accepting. He would have been hunting his own screen for a shape that does not render.

**APPROVED on round three**, 18 claims verified. Every finding was a place where he would have been
sent looking for something that was not there — which is the whole reason the gate exists.
