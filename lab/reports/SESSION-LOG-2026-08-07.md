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

## 9 · Not done in this job, deliberately

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
