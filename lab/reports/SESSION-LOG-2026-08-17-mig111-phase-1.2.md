# Session log — 2026-08-17 (afternoon) — MIG-111 Phase 1.2 opens

Continues `SESSION-LOG-2026-08-17.md` (which covered Phase 1.1 + the H1 harness).
Function in hand: **MIG-111 Phase 1.2 — the routed context pool.**

---

## 1. Architect step — done, Boss-approved

**Deliverables**
- `docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT-1.2.md`
- `docs/migrations/PJ-235-federation-boundary/MIG-111-ARCHITECT-1.2-EVIDENCE.md` (1,808 lines — raw
  six-slice call-site map, four design options, four adversarial passes; 14 agents, 13 returned.
  The maintenance-computation map slice failed its schema retries and was mapped by direct reading
  instead.)
- `docs/migrations/PJ-235-federation-boundary/MIG-111-PLAN-1.2.md` — 16 steps.

**The count correction.** The harness header (`vocab_harness.rs:6`) says "26 call sites across 11
files". The real figure is **29**, and `sight.rs:113` is a false positive — `is_null_type` is a
constant `matches!` that never touches the global. Amend the header when 1.2 lands.

**The finding that reshaped the phase — the SQLite trigger question.** Rust-side vocabulary
threading is necessary and NOT sufficient. `init_db_scoped` DROPs the `note_meta` sky trigger family
unconditionally and recreates it only under `if owns`, so the one production caller with
`owns == false` (`federation::migrate::run_migrations_on`) strips a linked universe's bookkeeping
and does not restore it. Safe today only because of the sentence at `search.rs:5966-5968` —
*"nobody writes through them, because the parent attaches a cUniverse read-only"* — **which Phase
1.2 is precisely the change that falsifies.** Hence a new refusal precondition (probe the child's
trigger set, refuse by name, never repair) and a separate PJ.

**Boss rulings taken:**
1. Approach approved as recommended (path-taking constructor + the three deletions + explicit
   vocabulary below the connection layer + three refusal preconditions).
2. Missing triggers ⇒ **REFUSE**, naming the universe. PJ-232 stays closed.
3. The trigger defect is a **separate PJ, fixed first** → PJ-302.
4. **Renames ARE in scope** (against the Architect's recommendation; ruled in). Scoping assumption
   stated for correction: renames *within* the owning universe, not the Phase-3 cross-universe
   cascade. Ordering rule this forces: **the vocabulary reaches `rewrite_wikilinks_in_text` FIRST
   (B5), the federation fence comes down SECOND (B6)** — never the reverse, never the same commit,
   because `[[refutes::Old]]` under the wrong vocabulary silently fails to rewrite and breaks a link
   on disk.
5. PJ-303 fixed before Step 0.

---

## 2. Baseline — the suite was NOT deterministic

Measured before touching anything: **six full runs → 1500/0 five times, 1499/1 once.**
The handover's claimed "1500 / 0" was true per-run and not true as a property.

Flake: `arabic::fst_bake::tests::persist_then_try_load_cached_roundtrip`. **→ PJ-303.**

---

## 3. PJ-303 — the flaky Arabic cache test (Reproduce-First honoured)

**Mechanism, read from source, not theorised.** The test wrote a hand-built bundle to the REAL
per-machine cache path (`cache_file_path()`, `fst_bake.rs:969-973`) and read it back. The production
initialiser `GenerativeFst::get()` (`fst_index.rs:113-140`) calls the same `try_load_cached` /
`persist_best_effort` against the same file, and on a cache miss rebuilds and re-persists the real
bundle. `sample_bundle()`'s FST bytes are `[0xAA,0xBB,0xCC]` — not a valid `fst::Map` — so a miss is
**guaranteed**. When `get()` landed in the window, the test read the real blob.

**Deterministic red** by forcing `get()` into the window (temporary `repro_pj303_real_cache_collision`,
run isolated). Failed with the identical real-FST blob the intermittent failure printed.

**Fix.** Path-taking cores `try_load_cached_at(path)` / `persist_best_effort_at(path, bundle)`; the
no-arg forms resolve `cache_file_path()` and delegate. The test now uses `tmp_path` — whose own doc
already said *"Avoids stomping on the real user cache during tests"*; it simply was not using it.
The test also stopped **deleting the developer's real Arabic cache on every run**.

Added coverage the old test lacked: a missing cache file must read as `None`, never an Err/panic.

**Whole-Ecosystem sweep.** After the fix the ONLY callers of the no-arg forms are the two production
initialisers (`fst_index.rs:119`/`:134`, `lexicon/graph.rs:273`/`:294`). No test reaches them. The
lexicon twin never had the defect — its cache test inspects the filename and does no I/O.

**Verification.** 11 consecutive green suite runs. Stated honestly: at the observed 1-in-6 rate, 11
clean runs occur by luck ~13% of the time, so the runs are corroboration — the proof is the
deterministic red on the exact mechanism plus the shared resource being removed outright.

**Diff-scoped safety inspection: 0 confirmed findings.**

**Boss tutorial:** `tutorial-auditor` → `ui-inspector` **REJECTED** (2 findings) → corrected →
re-inspection. Findings were (a) label "Arabic Engine Overrides" → the real label is **"Arabic
Overrides"**; (b) the draft said the cache may rebuild at app-open — it cannot; `GenerativeFst::get`
is a lazy `OnceLock` fired by note indexing or search-query tokenisation, and `lib.rs`'s `.setup()`
makes no call into the Arabic subsystem. The aside was moved to the step where it can actually occur.

---

## 4. PJ-302 — the foreign door stripped a child universe's triggers

**Red first.** New test
`federation::migrate::tests::schema_only_init_does_not_drop_a_foreign_universes_own_triggers`:
seed a child through a real owner-side `init_db`, hand it through `init_db_schema_only`, require the
triggers to survive.

**Why the existing sibling test could not see it.** `schema_only_init_writes_no_vocabulary_triggers_
into_a_foreign_db` starts from an EMPTY database, so `count == 0` passes *for the wrong reason* —
with no trigger present, an unconditional DROP has nothing to remove. The very next test's own doc
comment records this exact lesson for MIG-003 Step 1: **a test whose subject cannot fire is not a
test.**

### The Architect doc was WRONG, and the test corrected it

I wrote that a parent-migrated child also loses its outgoing-aggregate triggers. **It does not.**
`drop_outgoing_link_triggers` is the first line of `create_outgoing_link_triggers`
(`search.rs:2290`), which is itself inside the `if owns` gate — so for a foreign database that family
is neither dropped nor created, and survives. The red test named the true casualty list on its first
run:

```
Stripped: ["note_meta_sky_ai", "note_meta_sky_stratum_au", "note_meta_sky_maturity_au"]
```

I had asserted a blast radius from **reading** the gates instead of **executing** them — in a
document whose own §7 warns against precisely that. A first-mismatch assertion would have hidden it
a second time; the test now reports every casualty. Architect §3 corrected in place, with the error
recorded rather than quietly overwritten.

**Two distinct states the probe must handle:** a child opened by its owner then parent-migrated
(outgoing present, the three sky triggers gone); and a child never opened by its owner (neither
family ever created).

### The fix — stated as a construction

**The foreign door migrates SCHEMA and mutates NO trigger, in either direction.** Every trigger
DROP/CREATE in `init_db_scoped` is now `owns`-gated:
- `note_meta_sky_au` / `_ai` drop
- the stratum family drop + the legacy stratum drop
- the maturity family drop
- the `note_links_sky_ai/_ad/_au` **drop AND create** — this block was the counter-example to
  `InitScope`'s own doc comment: it sat outside every gate while interpolating `snapshot()`, so the
  schema-only door *did* write registry-generated SQL into a foreign `sqlite_master`. Harmless only
  because `structural_not_in_clause` happens to be vocabulary-invariant (`merge` forces
  `structural = false` on every non-seed delta) — an accident, not the gate.
- `drop_incoming_link_triggers`

`InitScope`'s doc comment corrected: it was false in **both** directions and a test starting from an
empty database could see neither.

**Result:** red → green; both sibling PJ-232 tests still pass (the fix does not reintroduce writing
parent DDL into a foreign database).

---

## 4b. PJ-304 — the H1 harness contaminated the suite with the hazard it documents

**Found only because PJ-303's fix stopped masking it.** After PJ-302 landed, a 3-run sweep failed on
run 2 with a pair I had not seen in 17 prior runs:

```
links_backfill::tests::backfill_populates_existing_rows
search::tests_mig066_outgoing::outgoing_aggregates_maintained_by_triggers
```

**Mechanism.** Both assert the empty-sentinel rank `9` — `cognitive_ids().len() + 1`, correct only
for a seeds-only registry. `vocab_harness::index_under_vocabulary` calls `set_active` with a custom
type and **never restores it**, so the first harness test to run left 9 cognitive types installed for
**every subsequent test in the process**. Not a race window — permanent contamination, with test
scheduling order deciding whether it bit. The 1/2/4 rank assertions survive because a custom type
sorts *after* the seeds: **only the sentinel moves**, which is why the failure looked arbitrary.

**Pre-existence PROVEN, not argued.** Every Phase 1.2 change stashed; 12 runs against pristine `main`
at `857530f5`; failed on run 3 with exactly that pair. It also retro-explains the day's very first
baseline run (2 failures, names not captured) — there were two independent flakes from the start.
**I did not assert "this looks unrelated to my change"; I removed my change and measured.**

**The irony is the lesson.** That harness was committed *specifically* to constrain Phase 1.2 against
LL-047 — *never install context into shared state for a duration* — and did precisely that to the
suite. **→ LL-049.**

**Fix (interim, and labelled as such).** An RAII `RestoreVocabulary` guard restoring seeds-only on
`Drop`, incl. on panic, applied to `index_under_vocabulary` and to the swap test which calls
`set_active` directly. It shrinks exposure from "the rest of the process" to "the duration of the
call" but does **not** close it — a concurrent non-harness test can still read the mutated global.
That residue is what LL-047 says is unclosable while the vocabulary is ambient. **Stage A removes it
structurally** (the harness stops calling `set_active`); the guard is deleted then.

**Note on the PJ-302 inspection.** Its first run spanned the moment the tree was stashed for the
pristine test, so its "0 findings" cannot be trusted for that diff — **re-run after restore** before
the commit. Recorded because a clean verdict over the wrong bytes is exactly the kind of false green
this session has been cataloguing.

---

## 4c. PJ-305 · PJ-307 · PJ-308 — what the inspection and my own carelessness turned up

- **PJ-305 (fixed).** Six Arabic-overrides persistence tests used FIXED temp folder names and each
  opened with `remove_dir_all`, so two `cargo test` processes delete each other's fixture. The
  unique-path idiom already existed *lower in the same file*. **Honest provenance: I triggered this
  by running two suites at once — it was not a baseline flake.**
- **PJ-307 (fixed, no reproduction).** `set_active` maintained the `ACTIVE_STORE_EMPTY` fast-path bit
  OUTSIDE its write guard while `set_sovereign_layer` maintained it INSIDE. Two writers, one
  invariant, two disciplines ⇒ no serialisation; either interleaving leaves the bit `true` over a
  non-empty store, so `active_if_non_empty` returns `None` from the atomic alone and the FTS5
  tokenizer stems every Arabic token as though no override existed — silently, with `active()` still
  correct so all diagnostics look healthy. Fixed by extracting ONE `publish()` both writers call.
- **PJ-308 (fixed).** `reindex_arabic_overrides` lacked `ensure_search_db_ready`; on a `None`
  connection `reindex_notes_matching_text` returns `Ok(0)`, identical to "no matches", which the
  panel paints green. Every sibling command already had the line.

### The mistake worth more than the fixes

I wrote a concurrency test for PJ-307. It **failed twice over**:

1. **It did not reproduce the defect.** Reverted against the pre-fix code, it PASSED — the
   interleaving needs a preemption inside a few-instruction window and 40 × 120 writes never hit it.
2. **It broke its neighbours.** Hammering the process-global store from two threads for the test's
   duration took down `set_active_replaces_prior_store_entirely`,
   `set_active_then_active_roundtrips`, `set_sovereign_layer_on_empty_active_creates_single_layer`
   and `set_sovereign_layer_preserves_child_layers` in **6 of 8** suite runs.

**I wrote a test that mutates shared state for a duration — in the same file as a fix for a bug whose
entire nature is mutating shared state for a duration, minutes after writing LL-049 about it.**
Deleted, with the reasoning recorded at the call site so the absence is a stated decision. LL-049
gains rules 6 and 7. **PJ-307 ships WITHOUT reproduction backing — flagged to the Boss as a
Reproduce-First exception, not assumed.**

### Measurement discipline — my own failure, recorded

I twice ran two `cargo test` loops concurrently and once edited source mid-sweep, then read the
resulting failures as evidence. They were artefacts of my own concurrency. **Rule for the rest of
this migration: one suite at a time, no source edits mid-sweep.** The contaminated runs were
discarded and re-measured clean.

---

## 4d. Boss validation — ALL PASSED 2026-08-17

| test | result |
|---|---|
| **PJ-303** Test 1 (Arabic cache) | **PASS**, then re-run after PJ-309 and **PASS** again |
| **PJ-309** snippet rendering | **PASS** — screenshot: no literal `<mark>` text, single clean highlight |
| **PJ-302** Stage 1 | **PASS** — 8023 → **8024** nodes, edges/MOCs unmoved, across a relaunch |
| **PJ-302** Stages 2 + 3 | **PASS** — all steps |

**The Boss improved the test twice.** He asked to run it on **Eisa Cognitive Knowledge** rather than the
parent — correctly, because that is one of his LINKED universes and therefore one that has actually
been through the foreign door this fix repairs, on ~7,500 notes instead of 253. And he renamed the
probe note when he found the original name already existed, which is exactly right: a duplicate would
have made the +1 ambiguous.

**PJ-309 was found by his eyes, not by any gate.** 1,501 Rust tests, 997 vitest, svelte-check and four
safety inspections were green over literal `<mark>` tags appearing in every Contents snippet, because
none of them look at rendered output. Worth remembering the next time the Boss-test step looks like a
formality: it is the only part of the pipeline with eyes.

---

## 5. Open / next

**Shipped and Boss-validated this session:** PJ-302, PJ-303, PJ-304, PJ-305, PJ-307, PJ-308, PJ-309.

**Awaiting a Boss ruling — none blocking, all recorded rather than assumed:**
- **PJ-311** *(HIGH · freeze)* — a Search Hub query can fan out to ~1,200 serial ONNX embeddings.
  Pre-existing; found by an inspection that the standing order arguably let me skip. The fix is a
  design choice (cache-read-only vs cap/slice vs cancellable-with-progress), so it is asked, not
  assumed. Recommendation: **cache-read-only**.
- **PJ-312** *(MED)* — a failed search renders identically to "no matches".
- **PJ-307** — shipped on a structural argument, **not** a reproduction. A Reproduce-First exception,
  flagged deliberately.
- **PJ-310** — `Open Existing Universe` may not refresh the frontend the way `Switch` does.
  UNVERIFIED; filed rather than guessed.
- **The rename scoping assumption** — Stage B assumes renames *within* the owning universe; the
  cross-universe cascade stays Phase 3. Still uncorrected by the Boss.

**Next:** Stage A (A1…A8), ending with `#[ignore]` removed from
`routed_write_must_match_the_owners_vocabulary` — the definition of done. It starts with
`registry_for_root`, the door that lets a universe's vocabulary be read *without* making that
universe active, which does not exist today and which every later step depends on.
- **Rename scoping assumption still uncorrected** by the Boss; B6 changes substantially if he meant
  the cross-universe cascade.
- Filed in passing, not yet numbered: `tension.rs:88-92` contains a false claim that
  `validate_path_in_any_library` refuses cUniverse paths — `libraries.rs:727-728`'s own doc says
  "including child universe libraries".
- Build cost noted for planning: a cold `cargo test` compile is **~8–18 minutes** on this machine;
  incremental suite runs are ~15–25 s.
