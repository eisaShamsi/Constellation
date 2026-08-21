# SESSION LOG — 2026-08-20 · MIG-111 Phase 1.2, Stage A (A1–A4)

**Function in hand:** the routed context pool — an operation on a note in a linked universe must do its
bookkeeping in THAT universe's database, using THAT universe's link vocabulary.

**Definition of done for the phase** (unchanged): `federation/vocab_harness.rs::routed_write_must_match_the_owners_vocabulary`
passes with `#[ignore]` removed. **Not reached today.** A1–A4 of eight are built and verified.

**Gates:** Rust **1516 passed / 0 failed / 20 ignored**, **five consecutive clean runs** (13.6s–17.7s).
Baseline was 1502; the delta is exactly the 14 tests added below. **Nothing committed** — the standing
order is that the Boss tests and passes every build before commit.

---

## 0 — the nested-universe question, closed

The Boss's own account closed it: he removed the `موسوعة عيسى` registry entry, used **+ Create New
Universe** pointed at `E:\موسوعة عيسى` (which produced the empty nested `E:\موسوعة عيسى\موسوعة عيسى\`),
removed that through the Universe Manager, then re-added via **Open Existing Universe**, which restored
all his notes. Verified afterwards:

- The nested folder survived the Manager's "Remove" (which **unregisters only** — its own text says files
  are not deleted). The Boss then deleted it in Explorer; confirmed gone.
- His real index is clean: **832 rows / 832 files**, zero rows pointing inside the deleted folder, zero
  orphans (every indexed row's file exists). The `833 / 832` drift line is resolved.
- **Not** caused by the door PJ-310 fixed: `open_existing_universe` reads `universe.json` and errors when
  absent, so it cannot create a universe.

**PJ-321 gained a controlled experiment and a second retraction** (commit `2333fb52`). Four registry
mutations in ~5 minutes, every one ending in an error-propagating `save_registry`, and the file
`list_universes` reads was unchanged throughout (277 bytes, one entry, mtime 08-07). An AppData sweep for
anything written in the following 30 minutes containing a universe id returned one Windows Recent-files
`.lnk`. The entry now carries an explicit **STOP THEORISING** — it has produced two confident wrong
explanations from me. Reproduce under instrumentation or leave it alone.

---

## A1 — `link_types::registry_for_root` (the door that did not exist)

Reads a universe's vocabulary **without making it active**. Strict via `universe::read_persisted_json`:
absent ⇒ the 8 seeds (a universe that never customized genuinely has them); unreadable, zero-length, or
corrupt ⇒ **Err naming the universe**.

The strictness is the point. `read_deltas` falls back to the seeds so a broken file can never break the
link grammar at boot — right when the alternative is an app that will not start. It is wrong for a routed
write, which would then classify one universe's links under another's vocabulary, silently, **with every
row count still correct**.

Also extracted **`write_link_types_at`** — the body of `save_universe_link_types` — so a routed write can
reuse the real writer without also making the vocabulary active, and so tests exercise the format the app
actually stores rather than one they invented (LL-048). The command now delegates to it.

**5 tests over real directories. MUTATION-TESTED:** replacing the strict read with a lenient one turns all
three refusals red.

## A2 — the merge-invariance pin

`structural_ids() == {contains, parent}` regardless of what deltas claim — a custom type claiming
`structural`, a cognitive seed claiming `structural`, a structural seed claiming cognitive. This matters
here specifically: a linked universe's `link-types.json` is a file the parent did not write. If a delta
could flip the flag, a child could hide a **cognitive** type from every maturity / strata / tension / sky
query the parent runs over it. **MUTATION-TESTED:** removing `d.structural = false` from the custom branch
turns it red.

## A3 — `federation/write_scope.rs`

`WriteScope` = the resolved pair of answers ("whose database, whose vocabulary"), carried explicitly.
`for_note` resolves the owner itself so a caller cannot pass the active universe for a note that lives
elsewhere; `routed_at` is path-taking so **production and the tests call the same function**.

Active arm: `SearchState.db` + `active_universe_vocabulary()` — byte-identical to today.
Routed arm: the `reconcile_filesystem` open shape (WAL, `synchronous=NORMAL`, **`recursive_triggers=ON`**,
`register_fts5_tokenizer`, 30s busy timeout), asserted by reading `PRAGMA recursive_triggers` back.

## A4 — four refusals, and the one that found a live bug

Each refusal names the universe, because the user has several and the one in front of him is not the one
at fault. Refusal 2 exists because `Connection::open` **creates** the file it cannot find — an unchecked
open founds an empty second index rather than failing. Refusal 3's trigger probe **self-calibrates against
the ACTIVE universe's own `sqlite_master`** rather than a hardcoded list, so it cannot go stale when a 15th
trigger is added.

### The federation reader was degrading silently

`universe::resolve_child_universe_roots` (universe.rs:674-696) answers `Vec::new()` for **three different
situations**: no children, the manifest could not be READ (:685-687), the manifest could not be PARSED
(:688-690) — plus it drops any declared child failing `fs::canonicalize` or `is_dir` (:693-694).

Under MIG-108, linked universes live **under** the active root, so losing a child from the candidate set
does not make `resolve_owner` refuse — **it makes it answer PARENT, `is_active: true`**, and the routed
write lands in the wrong database.

`resolve_owner`'s own doc comment (owner.rs:143-146) says it reads roots fresh rather than from
`load_all_libraries`' cache *precisely* to avoid a degraded federation (PJ-300). It then called a reader
that degraded silently one layer down. The comment described the intent; the code did not implement it.

**Fixed** with `resolve_child_universe_roots_strict` / `..._recursive_strict`. Two deliberate differences,
neither inventing a policy: an existing-but-unreadable manifest is an **error** (`read_persisted_json`'s
standing doctrine — only "not found" is trustworthy emptiness); a declared child that cannot be
canonicalized is **kept as its declared path**, because canonicalization here is de-duplication, not a
membership test, and dropping it is exactly what removes a universe from the candidate set.

The test **demonstrates the misroute first** (asserts `is_active` is true against the lenient reader),
then asserts the refusal. 4 tests, real directories, real manifests.

**Second surface, NOT fixed → PJ-322.** `mig108::assemble_foreign_roots` (mig108.rs:2084-2097) builds the
**foreign** set for the unification engine from the same lenient reader. Severity **UNVERIFIED** — I read
the enumerator and its caller, I did not trace the consumer, and I will not guess. **Panel ruling owed.**

---

## The build failure that cost ~8 rebuilds, and what it taught

A test that constructed a `WriteScope` made the **entire lib test binary fail to start** —
`STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139), before a single test ran — while the identical production code
compiled and linked fine.

Bisected, every step measured:

| configuration | exe bytes | starts? |
|---|---|---|
| module absent | 86,437,376 | ✅ 1532 tests |
| module present, no tests | 86,438,400 | ✅ 1532 |
| trivial no-call test in the module | 86,438,400 | ✅ |
| trivial test in a **different** module | 86,438,400 | ✅ 1533 — **rules out a test-count threshold** |
| test calling `search::init_db` from the module | 86,441,472 | ✅ — **rules out "any heavy call"** |
| test calling `WriteScope::routed_at` | 86,816,256 | ❌ |
| …with `AppHandle` removed from the type | 86,456,832 | ✅ |

`dumpbin /dependents` showed the working and broken import tables were **identical**, and every imported
function was satisfiable. The trigger was `WriteScope` **holding a `tauri::AppHandle` as a field**:
returning one from a test instantiates the Wry runtime's type graph into the test binary.

**The fix is a better design anyway.** `Target::Active` now carries nothing and `with_conn` takes
`&AppHandle` as a parameter — a scope is a **value** (root + vocabulary + at most an owned connection),
constructible and assertable without a running Tauri app. Recorded at the type.

**Two process lessons, both mine:**

1. **A "successful build" was not.** One link failed with `LNK1104` (the exe was locked); cargo's
   fingerprint then reported the **corrupt** binary as up-to-date for several subsequent runs. I told the
   Boss "it isn't a stale artifact" — wrong; it was exactly that, a different artifact than the one I
   cleaned. Deleting the exe and forcing a relink is the check.
2. **`cargo clean -p constellation` removed 36,940 files / 26.4 GB**, not just the crate — an ~18-minute
   cold rebuild. Not a cheap diagnostic. Also noted: `onnxruntime.dll` is absent from `target/debug/deps`
   **and** from the ort cache (`%LOCALAPPDATA%/ort.pyke.io/...` holds only `DirectML.dll` and
   `onnxruntime.lib`). Whether that is latent or a red herring is **unresolved** — put to the panel.

## My own test was bitten by the thing this migration exists to remove

`a_properly_built_universe_yields_its_own_vocabulary` passed in isolation and **failed in the full suite**:
its guard reads the process-global vocabulary, and `vocab_harness` mutates it via `set_active`. Run
concurrently, the harness had `refutes` installed at the moment my guard read it. Exactly the residue
`RestoreVocabulary`'s own doc comment predicted ("a non-harness test running concurrently can still read
the mutated global"). Serialized on `HARNESS_LOCK` as the interim answer; **A5 deletes the coupling with
the readers**.

---

## State at close of this stretch

- **Uncommitted**, working tree: `link_types.rs`, `universe.rs`, `federation/{mod,owner,write_scope}.rs`,
  the plan doc, the PJ ledger.
- **Committed today:** `2333fb52` (PJ-321 controlled experiment + retraction).
- **Open, blocking nothing yet:** PJ-322 panel ruling; the plan's A5–A8.
- **Next:** A5 — delete `is_known_type` / `is_structural_type`, make `structural_frontmatter_targets` a
  method, rename `snapshot()` → `active_universe_vocabulary()` (the alias exists; the removal is A5), and
  let the compiler enumerate the ~26 remaining ambient reads across 11 files.

---

# Afternoon — the four rulings, two panels, and a naming failure

## Boss rulings taken 2026-08-20

| # | Question | Ruling | Status |
|---|---|---|---|
| 1 | May the directory-moving commands hard-fail on an unreadable registry? | **Yes** | **Built** |
| 2 | Does A5 wait for the rest of PJ-322? | **No** | A5 mapped, not started |
| 3 | Refuse entirely on a dangling Linked Universe? | **No — provide options** | Concept v1.1 |
| 4 | Spend 18 min proving the Tauri mechanism? | **Defer (panel rec.)** | Deferred |
| — | Priority for PJ-326..331 | **Accept panel ordering** | Scheduled after Stage A |

## PJ-322 — decision 1 built

- `universe::registered_universe_roots_strict` — reuses `load_registry_for_update`'s split verbatim:
  **`Unreadable` refuses** (transient), **`Corrupt` sets the file aside** and proceeds from an empty
  list that is now genuinely true. That split is a recorded scar; the first version of *that* fix
  refused on both and locked the user out of the app.
- `mig108::assemble_foreign_roots` is `Result`-returning, using both strict readers. Safe because
  preflight runs **before** `Journal::new` — verified by the panel, and the opposite of the reason I
  had given for parking it.
- `Mig108UnifyDialog.svelte` gained a **`blocked`** state: heading, plain-language body, the verbatim
  reason, **Not now / Try again**. Never a gate. 3 i18n keys × 15 locales, parity verified.

## PJ-325 — the defect the concept panel found in the morning's own code

`universe.rs` preserved `PersistedError::{Unreadable, Corrupt}` under a comment saying `Corrupt` is
permanent and retrying is pointless — and `mig108.rs` **flattened both one call later**. The card
told the user *"usually temporary… Try again"* about a file that will never repair itself.

**Fixed:** the kind crosses IPC as a machine-readable marker (stripped before display — the frontend
must never pattern-match a translated sentence to choose buttons); a distinct body for `damaged`; and
**the Try-again button is not rendered at all** when retrying cannot work. Also fixed: a malformed
user-facing sentence carrying 22 literal spaces — **the only prose Constellation currently produces
about a broken Linked Universe link**.

## The naming failure — PJ-331

Boss: *"We have decided to change the naming from 'cUniverse/Child Universe' to 'Linked Universe'.
Have you forgotten?"*

Searched before answering. **"linked universe": 431 occurrences across 75 files** — long established
in practice. **The ruling itself: written nowhere.** `CLAUDE.md` still *defined* the level as
"cUniverse (Child Universe)"; orientation v3.98 listed `cUniverse` among labels that "intentionally
stay English."

**Cost, concretely: a nine-agent panel read those documents and formally recommended the retired name
back to the Boss.** A ruling that lives only in conversation does not fade quietly — it is
contradicted by the project's own records and then re-proposed as advice.

Recorded in `CLAUDE.md`, orientation **v3.99**, memory, and PJ-331 (10 translated values + 2
hardcoded labels; keys and Rust identifiers explicitly out of scope; historical records never
rewritten).

**Boss's diagnosis of the root cause — _"That's why you have to conduct the PCS and orientations more
often"_ — is correct and was acted on immediately** rather than deferred to session close.

## Panels convened (both corrected me on substance)

1. **Stage A panel** — 4 lenses + 4 skeptics + synthesis. Found my reason for parking PJ-322 factually
   wrong; refuted an app-killer escalation raised by its own data-safety lens; found the trigger floor
   could be read while a background repair had emptied it; found my test proved nothing.
2. **Concept + test panel** — concept ACCEPTED, **diagnosis REJECTED ON FACT**; measured that my
   Gone/Unreachable rule fails on Windows; found PJ-325; found **you cannot unlink a Linked Universe**
   outside the first-run wizard. `ui-inspector` **REJECTED** the Scratch test — the draft's safety
   promise was false. **Not sent.**

## Filed this afternoon

PJ-325 (fixed) · PJ-326 cannot unlink · PJ-327 badge can never appear · PJ-328 reason string is an
IPC-contract change · PJ-329 dead child's libraries leak into the active list *(severity not
established)* · PJ-330 legacy-layout child may never warn *(not adjudicated)* · PJ-331 the rename.

## Gates

**Rust 1522 / 0** (20 ignored) · **svelte-check 0 errors** · **15/15 locales in parity** ·
frontend perf flake proven **pre-existing** by stashing to pristine `main` (2 fail there too).

## State at close of this block

- **Documentation committed.** Rulings, concept papers, ledger, LL-050, orientation v3.99, MoCh.
- **Code NOT committed** — the Boss-test gate is still owed on the `blocked` card and the PJ-322/325
  backend. Splitting the commit secures the reasoning without shipping untested code to `main`.
- **Next:** A5 — measured surface is **10 production sites, not 26**; seven sit behind one chokepoint
  (`index_note_impl`). See the plan's "A5–A7 — the measured surface" section.
