# MIG-111 Phase 1.2 — PLAN (the routed context pool)

**2026-08-17** · Architect: `MIG-111-ARCHITECT-1.2.md` · Evidence: `MIG-111-ARCHITECT-1.2-EVIDENCE.md`

## Boss rulings taken 2026-08-17 (these shape the plan)

| # | Ruling |
|---|---|
| 1 | **Approach approved as recommended** — one path-taking constructor that resolves the owner itself; delete the three global-reading shortcuts; explicit vocabulary below the connection layer; the three refusal preconditions. |
| 2 | **Missing triggers ⇒ REFUSE, naming the universe.** The parent never writes schema into a universe it does not own. PJ-232 stays closed. |
| 3 | **The missing-trigger defect is a SEPARATE PJ, fixed FIRST** — its own reproduction, its own commit, before 1.2 builds on it. → **PJ-302**. |
| 4 | **Renames ARE in scope** (against the Architect's recommendation; ruled in). |

**Scoping assumption on ruling 4, stated for correction:** "renames in scope" = *a rename commanded
on a note that lives in a linked universe works end-to-end, doing its bookkeeping and its wikilink
rewriting in that universe with that universe's vocabulary.* It does **not** include the
cross-universe cascade (renaming in universe A healing referrers in universe B), which the migration
plan schedules as Phase 3 / R23 and calls the acceptance test of the whole migration.

---

## The ordering rule renames impose

✅ Verified: `rewrite_wikilinks_in_text` (libraries.rs:7490) decides "is `[[Foo::Old]]` a typed link?"
via the **global** `is_known_type`. The cascade's federation fences are the SEEK-branch
`path_is_under_any(&…, &foreign)` (libraries.rs:6969) and the `&foreign` set passed into
`update_links_recursive` (libraries.rs:6990).

> **Drop a fence before the vocabulary reaches the rewriter and a rename silently corrupts a linked
> universe's files on disk.** `[[refutes::Old]]` in a child, where `refutes` is a child-only type:
> the parent does not know `refutes`, so the whole `refutes::Old` reads as the target name, the
> rewrite does not fire, and the link is left broken — no error, no count wrong.

**Therefore, in every step below: the vocabulary reaches the rewriter FIRST; the fence comes down
SECOND. Never the reverse, and never in the same commit.**

---

## Step 0 — PJ-302: the dropped triggers (separate, first, its own commit)

**Concept (the horse):** *a universe's bookkeeping machinery must not be removed by a process that
will not put it back.*

**The defect.** `init_db_scoped` DROPs the sky trigger families unconditionally (search.rs:5868-5874,
:5884-5888, :5924-5930, and the `note_meta_sky_ai`/`_au` drop preceding :5640) and recreates them only
under `if owns` (:5640, :5891, :5933, :5969). The only production caller with `owns == false` is
`federation::migrate::run_migrations_on` (federation/migrate.rs:169) via `federation/attach.rs:172`.
**So any linked universe whose schema this app has migrated is on disk right now with its
bookkeeping triggers deleted and not restored.** Latent only because nothing writes to a cUniverse
— which Phase 1.2 changes.

| | |
|---|---|
| **Reproduce first** | A test that builds a child universe with a stale `user_version`, runs `run_migrations_on` against it, then asserts `sqlite_master` still holds `note_meta_sky_ai`, `note_meta_sky_stratum_au`, `note_meta_sky_maturity_au`. **Red before any fix.** |
| **Fix** | Make the DROPs conditional on `owns`, matching the CREATEs. A foreign schema migration must be schema-only in both directions — it may not *remove* vocabulary-dependent DDL any more than it may *write* it. |
| **Also in this commit** | Correct `InitScope`'s false doc comment (search.rs:4576), which claims every registry-generated DDL is skipped for a foreign database while `note_links_sky_ai/_ad/_au` (search.rs:5531-5575) sits outside every gate and interpolates `snapshot()` at :5540. Gate that block too. |
| **Verify** | The red test goes green; `cargo test` green; a child DB round-tripped through `run_migrations_on` is byte-comparable in `sqlite_master` to one that was not. |
| **Boss test** | Yes — user-visible as "a linked universe still works after an update". Goes `tutorial-auditor` → `ui-inspector` → Boss. |

---

## Stage A — the routed context (this is what removes the `#[ignore]`)

### Build status — 2026-08-20

| step | state | evidence |
|---|---|---|
| **A1** | **DONE** | `link_types::registry_for_root` + `link_types_file_in` + `write_link_types_at` (the save command's body, extracted so tests exercise the real writer). 5 tests over real directories. **Mutation-tested**: making the read lenient turns all three refusals red. |
| **A2** | **DONE** | `merge_decides_the_structural_lane_no_matter_what_the_deltas_claim`. **Mutation-tested**: removing `d.structural = false` from the custom branch turns it red. |
| **A3** | **DONE** | `federation::write_scope::WriteScope` — `for_note` (resolves the owner itself) and `routed_at` (path-taking, so production and the tests call the same function). Active arm = `SearchState.db` + `active_universe_vocabulary()`, unchanged. Routed arm = the `reconcile_filesystem` open shape, asserted `PRAGMA recursive_triggers == 1`. |
| **A4** | **DONE, and it found a second surface** | Four refusals, each naming the universe. The trigger probe **self-calibrates against the active universe's own `sqlite_master`** rather than a hardcoded list, so it cannot go stale when a 15th trigger is added. |
| **A5** | **PART-LANDED — the plan said "not started" and that was false** | `active_universe_vocabulary()` exists (link_types.rs) and `snapshot()` survives as a transitional alias delegating to it. The RENAME landed; the DELETION of the three ambient readers and the ~26 call-site threading did not. Corrected after the panel caught the doc contradicting the code. |
| A6–A8 | not started | |

### What A4 turned up — the fourth refusal was not a formality

`universe::resolve_child_universe_roots` (universe.rs:674-696) returns an **empty list** for an
unreadable manifest, an unparseable manifest, *and* genuinely having no children. Under MIG-108
nesting a linked universe lives UNDER the active root, so losing it from the candidate set does not
make `resolve_owner` refuse — **it makes it answer PARENT, `is_active: true`.** The routed write
then lands in the wrong database with every row count correct.

`resolve_owner`'s own doc comment says it reads roots fresh rather than from `load_all_libraries`'
cache *specifically* to avoid a degraded federation (PJ-300). It then called a reader that degraded
silently one layer down. The comment described the intent; the code did not implement it.

Fixed with `resolve_child_universe_roots_strict` / `..._recursive_strict` (fail-closed on an
unreadable manifest; a declared child that cannot be canonicalized is **kept as its declared path**
rather than dropped, since dropping is what removes it from the candidate set). The test
`a_manifest_that_cannot_be_parsed_refuses_instead_of_reporting_no_children` **demonstrates the
misroute first**, then the refusal.

**Second surface, NOT fixed — `mig108::assemble_foreign_roots` (mig108.rs:2084-2097)** builds the
set of roots the unification engine treats as foreign from the same lenient reader. That is a write
path in a different migration whose failure mode is moving directories. Filed **PJ-322**, severity
UNVERIFIED (I read the enumerator and its caller; I did not trace the consumer). **Panel ruling owed
before 1.2 closes.** The two display surfaces (`bases.rs:729`, `lens/system_notes.rs:186`) stay
lenient on purpose — the same strict/lenient split the codebase already runs for `read_deltas` vs
`read_persisted_json`.


| step | what | verification |
|---|---|---|
| **A1** | **`link_types::registry_for_root(root) -> Result<LinkTypeRegistry, String>`** — the missing door. Reads `<root>/.constellation/link-types.json` through the STRICT `universe::read_persisted_json` (universe.rs:260), never the lenient `read_deltas`. | Unit tests over REAL directories: absent file ⇒ the 8 seeds; unreadable ⇒ `Err`; zero-length ⇒ `Err`; corrupt ⇒ `Err`. Written by `save_universe_link_types`, read back here — the production writer's format, not a literal. |
| **A2** | **The `merge` invariance pin.** A test asserting `structural_ids() == {contains, parent}` for a registry merged with custom types, custom *structural-flagged* deltas, and seed overrides — so §2(a) becomes a declared contract instead of a property of a function body. | Red the day `merge` stops forcing `structural = false`. |
| **A3** | **`WriteScope::for_note(app, note_path)`** — resolves the Owner itself via `resolve_owner` (owner.rs:149); active ⇒ borrows `state.db` + `snapshot()`, byte-identical to today; routed ⇒ the three preconditions, then a dedicated open in the `reconcile_filesystem` shape (WAL, `synchronous=NORMAL`, **`recursive_triggers=ON`**, `register_fts5_tokenizer`). **No pool.** | `PRAGMA recursive_triggers == 1` on a routed scope; the active arm proven to produce the same four generated SQL strings as before the change. |
| **A4** | **The three preconditions, each a refusal** — (1) trigger-capability probe over `sqlite_master`, refusing by name (ruling 2); (2) `search.db` must already exist, because `Connection::open` *creates* it (search.rs:4603); (3) strict vocabulary read (A1). | One test per refusal, each asserting the message names the universe and that **nothing was written** (filesystem hash + row counts unchanged). |
| **A5** | **Delete the three ambient readers** — `is_known_type` (link_types.rs:359), `is_structural_type` (:369); make `structural_frontmatter_targets` (:385) a `&self` method. Rename `snapshot()` → `active_universe_vocabulary()`, `pub(crate)`. Every remaining reader becomes an unresolved name and must answer "whose vocabulary is this?". | Compiler enumerates the sites. Plus a call-site-pinning test on `active_universe_vocabulary` (file + count) so an eleventh ambient read fails a test rather than compiling clean. |
| **A6** | **Thread G2's parse chain** — `extract_wikilinks`, `extract_typed_links`, `parse_link_body`, `emit_frontmatter_links`, `extract_frontmatter_typed_links`, and `link_types::structural_frontmatter_targets`, plus the two `is_structural_type` decisions at search.rs:8018 and :8485 (the latter feeds `link_row_is_preserved` — earned Living-Link data). | The acceptance test (A8). |
| **A7** | **Thread G3's save-path pair** — `incoming_aggregate_assignments` (search.rs:2489) and `outgoing_aggregate_assignments` (search.rs:2243) take `&LinkTypeRegistry`; `maintain_incoming_after_save` and `maintain_sky_after_save` carry the scope. | Invariant 3's byte-identical check. |
| **A8** | **The harness rewrite + `#[ignore]` removed.** Per Architect §7: drive `reindex_single_note` (the production funnel, crossing the `is_built` gate at search.rs:12712), build both universes with `create_universe` / `add_child_universe` / `save_universe_link_types`, and **resolve** the Owner rather than constructing it. Extend `Aggregates` with the four `outgoing_*` columns, `sky_nodes.stratum/maturity`, and `weight`/`confidence`/`traversal_count`. Re-author `a_vocabulary_swap_reaches_back_into_an_already_open_database` into its positive form (construct the scope BEFORE the swap; assert the result is UNCHANGED) — **and not by passing `&active_universe_vocabulary()`, which would restore the anti-pattern and turn it green.** | **`routed_write_must_match_the_owners_vocabulary` passes with the attribute removed. This is the definition of done.** |

---

## A5–A7 — the measured surface, replacing the plan's estimate

**Mapped 2026-08-20 by grep, not by memory.** The Architect estimated "~26 call sites across 11
files". The *ambient-reader* surface is smaller and more concentrated than that, and knowing exactly
where it is changes A5 from a sprawl into one coherent change.

### The three ambient readers — every PRODUCTION site

| reader | production sites | note |
|---|---|---|
| `is_known_type` | `libraries.rs:7490` (the rename rewriter), `search.rs:7297` (`parse_link_body`), `search.rs:7424` (`emit_frontmatter_links`) | 3 |
| `is_structural_type` | `search.rs:8071`, `:8538`, `:8614` (all inside `index_note_impl`), `search.rs:9603` (`structured_search`) | 4 |
| `structural_frontmatter_targets` | `search.rs:7132` (`extract_wikilinks`), `strata.rs:199`, `inspector360.rs:368` | 3 |

**10 production sites, not 26.** `cache.rs:1654` is inside `#[cfg(test)] mod tests` (module opens at
`cache.rs:1581`) and is **not** a production reader — the Architect's count included it.

### The shape this reveals

Seven of the ten sit in ONE call graph:

```
index_note / index_note_bulk  →  index_note_impl(search.rs:7937)
                                   ├─ extract_wikilinks(:7121)        → structural_frontmatter_targets
                                   ├─ parse_link_body(:7296)          → is_known_type
                                   ├─ emit_frontmatter_links(:7414)   → is_known_type
                                   └─ is_structural_type ×3 (:8071, :8538, :8614)
```

`index_note_impl` is the **single chokepoint**. Give it the vocabulary and the whole parse chain is
threaded. Its production callers are only three: `index_repair.rs:1018` (the bulk walk) and
`search.rs:4280` / `:4339` (the reindex paths). Everything else calling `index_note` is a test.

### The decision this forces, stated so it is not made by accident

`index_note` keeps its current signature and calls **`active_universe_vocabulary()` explicitly, at
that one line, with a comment saying why** — a note indexed through the active universe's connection
genuinely should use the active universe's vocabulary. The routed path gets
**`index_note_with(conn, path, lib, force, vocab)`**, which is what `WriteScope` callers and the
acceptance harness drive.

That is not a loophole. **A5's purpose is to delete the reads that are HIDDEN inside the parser,
where no caller can see which vocabulary is being used.** One named, commented, greppable call at a
funnel is the honest form of the same fact — and it keeps ~40 existing test call sites from churning
for no gain in truth. The A5 call-site-pinning test (file + count on `active_universe_vocabulary`)
is what stops an eleventh one appearing quietly.

### Order of execution

1. `structural_frontmatter_targets` becomes `&self` on `LinkTypeRegistry`; the three callers take a registry.
2. `is_known_type` / `is_structural_type` deleted; the 7 sites become `reg.is_known(..)` / `reg.is_structural(..)`.
3. `index_note_impl` + the three parse helpers take `&LinkTypeRegistry`; `index_note_with` added.
4. `maintain_incoming_after_save` (search.rs:2637) takes it — A7's half; this is the function the
   harness calls after `index_note`, and it is where the incoming aggregates get their answer.
5. `snapshot()` alias deleted; the pinning test lands.
6. **Only then** the harness rewrite (A8) and the `#[ignore]` removal.

`strata.rs:199`, `inspector360.rs:368`, `libraries.rs:7490` and `search.rs:9603` are Stage B's B3/B4/B5
and take the registry from their own walk context — but they must be converted in the SAME pass as the
deletion, because deleting the free functions is what makes them stop compiling. **B5's ordering rule
still holds absolutely: the vocabulary reaches `rewrite_wikilinks_in_text` first; the rename fence comes
down in a LATER commit.**

## Stage B — the whole ecosystem, same pass (Whole-Ecosystem Fix Law)

| step | what | verification |
|---|---|---|
| **B1** | **The backfills' `recompute_*` FUNCTIONS**, not merely the generators — `links_backfill::recompute_range` (links_backfill.rs:248), the incoming twin, `sky_backfill` (:283, :388, :399), `name_fold_backfill` (:157, :173, :178). Parameterising only the generators forces `active_universe_vocabulary()` into six call sites by construction. | R4: assert no `schema_versions` row changes across a routed write; the two stamp writers (links_backfill.rs:464, incoming_links_backfill.rs:173) proven unreachable from `WriteScope`. |
| **B2** | **G1's DDL generation** takes the vocabulary explicitly, so `init_db_scoped` can no longer bake an ambient answer into anyone's `sqlite_master`. | With PJ-302 landed, a foreign init writes no vocabulary-derived DDL in either direction. |
| **B3** | **The three filesystem re-walkers** that share the parse chain — `strata.rs` (:168, :199, :208), `inspector360.rs` (:284, :343, :368, :375), `libraries.rs` (:4040, :4065). Converting only search.rs's chain makes them disagree with the index about the same note, breaking MIG-067's **ONE PARSER, ONE ANSWER** (link_types.rs:380). | Index a note, re-walk it, assert identical typed-link output. |
| **B4** | **G5 read-side analytics** — `cache.rs` (:516, :548, :1288), `sight.rs` (:77), `tension.rs` (:277), `search.rs` (:9550). Per-schema vocabulary on a federated read: each takes `schema: &str` over ONE connection, so this is a threaded registry per schema, not a bundle. **Also correct `tension.rs:88-92`**, which falsely claims `validate_path_in_any_library` refuses cUniverse paths. | A federated read over two universes with differing vocabularies reports each universe's own types. |
| **B5** | **RENAMES (ruling 4) — vocabulary first.** `rewrite_wikilinks_in_text` (libraries.rs:7490) and `update_links_recursive` take the OWNER's registry. **The fences stay up in this commit.** | A rename inside a linked universe, driven with the child's vocabulary but still fenced, proves the rewriter's decisions are the child's — before anything can act on them. |
| **B6** | **RENAMES — the fence comes down.** Remove the SEEK-branch refusal (libraries.rs:6969) and the `&foreign` exclusion (:6990) for the OWNER's own universe only; a rename still refuses to cross into a *third* universe (that is Phase 3 / R23). Route the rename's DB tail through `WriteScope`. | Red→green on a child-only-type link: `[[refutes::Old]]` in a linked universe renames correctly, where before B5 it would have been left broken. **Editor-Surface Gate checklist on a FEDERATED note**, including the linked-probe-pair shape (item 6). |
| **B7** | **The watcher fence, named and tested.** `reindex_changed_paths` is fenced by a single identifier at search.rs:12971 (`try_load_libraries`, the OWN set). Swapping it for `load_all_libraries` would enable routed child writes with **zero compile errors**. It stays as-is in 1.2; a test pins it so the swap cannot happen silently. | Test asserts the own-scope resolver is the one in use. |

---

## Gates on every step

- **Per-build diff-scoped inspection** before each commit: `Workflow({ name: 'safety-inspection', args: { files: [<changed>] } })`. Every confirmed finding fixed **before** the commit.
- **`/simplify`** on the final diff of each stage.
- **The Boss tests and passes every build before it is committed.** Tutorials go `tutorial-auditor` → `ui-inspector` → Boss, never direct.
- **SO#9** — the Pending Jobs ledger reconciled at the close, in the same commit (v1.92: close 1.2, file PJ-302, re-rank).
- **SO#6** — orientation doc v-bump in the same commit as the change that triggers it.
- **Measurement gate (R33)** on the 7,820-note universe: boot, typing latency, routed-write latency. Any regression blocks.

## Known non-determinism that weakens every red→green claim here

`arabic::fst_bake::tests::persist_then_try_load_cached_roundtrip` fails ~1 run in 6 (measured: six
full runs, 1500/0 five times, 1499/1 once). It writes to the **real user cache path** that production
code also writes to (fst_bake.rs:969-973 vs fst_index.rs:119/:134). **Filed as PJ-303**; until it is
fixed, every "green" in this plan is asserted over a suite that is not deterministic, and any single
green run is not proof. Re-run on failure and record which test failed — never assume it was this one.
