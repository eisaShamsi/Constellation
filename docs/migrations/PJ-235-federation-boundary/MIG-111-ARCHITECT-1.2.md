# MIG-111 Phase 1.2 — ARCHITECT: the routed context pool

**2026-08-17** · Binds R3 R4 R10 · Attack H1 · Evidence: `MIG-111-ARCHITECT-1.2-EVIDENCE.md`
(six-slice call-site map, four design options, four adversarial passes — 13 agents).

**Verification status.** Every claim marked ✅ below was read in source by me personally this
session, at `main` `857530f5`. Claims carried from the agent map without my own re-read are marked
⚠️ and named as such. The maintenance-computation map slice failed its schema retries; that group
was mapped by direct reading instead, so it is ✅ throughout.

---

## 0. The function in hand

> **MIG-111 Phase 1.2 — the routed context pool.** An operation on a note owned by a *linked*
> universe must do its bookkeeping in **that** universe's `search.db`, computed with **that**
> universe's link vocabulary — never the active universe's process-global `link_types::REGISTRY`.

## 1. The concept (the horse)

**A routed write must carry the vocabulary of the universe it writes into — as a value it holds,
not as a global it consults — so that *which universe happens to be active* can never change *what
is stored in another universe's rows*.**

The pool, the bundle, the threaded parameter are carriages. The horse is that **the target's
identity must travel with the write.** Today it does not travel at all: it is re-inferred from
ambient process state at each of 29 separate call sites, several call frames below the last
function that still knew which database was involved.

---

## 2. The territory — how a vocabulary reaches a call site today

One global, one door, no key:

```
static REGISTRY: OnceLock<RwLock<LinkTypeRegistry>>    link_types.rs:351
  written by  set_active(deltas)                        link_types.rs:481
     ← load_active(app)           link_types.rs:522   (boot / universe switch)
     ← save_universe_link_types   link_types.rs:544   (the Links editor SAVES)
     ← list_link_types            link_types.rs:588   (the Links editor OPENS — a read-shaped
                                                       IPC command that MUTATES the global)
  read by     snapshot()                                link_types.rs:498  (owned clone)
              is_known_type / is_structural_type        link_types.rs:359 / :369
              structural_frontmatter_targets            link_types.rs:385  (self-snapshots at :390)
```

✅ `snapshot()` takes no arguments. There is no seam in it for a universe, a path, or a connection.
✅ **There is no root-parameterised vocabulary reader anywhere in the codebase.** `read_deltas(app)`
(link_types.rs:514) resolves via `link_types_path(app)` (:507) → `active_constellation_dir`. A
child's vocabulary cannot currently be read *without making the child active* — the design LL-047
rules out. **Phase 1.2 must build that door before anything else.**

### The 29 production readers, grouped by what they DECIDE

| # | Group | Count | On 1.2's routed-write path? |
|---|---|---|---|
| **G1** | **Trigger DDL** — persisted into `sqlite_master`, fires forever after | 5 blocks | **YES — and no Rust threading reaches it. §3.** |
| **G2** | **Parse chain** — what a wikilink MEANS | 6 | **YES — the core of 1.2** |
| **G3** | **Maintenance computation** — call-time SQL writing row VALUES | 4 generators, ~27 sites | **YES** for the save-path pair; backfills are Stage B |
| **G4** | **Fingerprint gates** — a value in the DB vs the process-global | 4 | **NO** (R4 forbids a routed stamp) — but §6 |
| **G5** | **Read-side analytics** | 12 | **NO** — though three are arguably already wrong over federated reads today |
| **G6** | **The one reader whose answer reaches a `.md` FILE** | 1 (libraries.rs:7490) | Only if 1.2 routes renames — Boss question 3 |

✅ **Correction to the harness header.** `vocab_harness.rs:6` says "26 call sites across 11 files".
The real count is **29**, and one commonly-counted site is a false positive: `sight.rs:113` reads
`is_null_type`, which is `matches!(id, "associative" | "relates" | "")` (link_types.rs:493) — a
constant match that never touches the global. Amend the header when 1.2 lands.

### Two facts that reshape the problem

✅ **(a) Most of G1/G2 is vocabulary-INVARIANT today — and leaning on that would be a trap.**
Four of five G1 blocks and three of six G2 reads consult only `structural_not_in_clause`, which
filters on `t.structural`. Reading `merge` (link_types.rs:115-168) in full: it always seeds from
`seeds()` (:121); forces `structural=false` for a cognitive-seed delta (:131); sets `true` only for
the fixed `STRUCTURAL_SEED_IDS = &["parent","contains"]` (:61, :148); and forces `false` for **every**
custom type (:159). So `structural_ids()` is invariantly `{contains, parent}` in every universe.

The genuinely divergent reads are exactly: `is_known_type` (search.rs:7244, :7371, libraries.rs:7490)
and the `cognitive_ids`-derived lists inside `outgoing_aggregate_assignments` (search.rs:2243) and
`incoming_aggregate_assignments` (search.rs:2489).

**Do not turn this into a scope reduction.** The invariance is a property of `merge`'s *body*, not a
declared contract — precisely the class of undeclared truth this migration has twice been burned by
(LL-046, LL-048). Anything relying on it needs a test that goes red the day `merge` stops forcing
`structural = false`. That test is the cheapest item in this whole document.

✅ **(b) A purely connection-bound vocabulary is not expressible.** Four of the six G2 readers are
pure `&str → value` with no connection in scope and no prospect of one — `extract_wikilinks`,
`extract_typed_links`, `parse_link_body`, `emit_frontmatter_links`, plus
`link_types::structural_frontmatter_targets`. The deepest sits **five frames** below `index_note`:

```
index_note → index_note_impl → extract_frontmatter_typed_links → emit_frontmatter_links
           → parse_link_body → is_known_type
```

**Therefore, whatever bundle exists at the top, the mechanism below the connection layer is a
threaded `&LinkTypeRegistry`.** That is forced by the call graph, not chosen. The precedent already
exists and stops one frame short: `resolve_wikilink_type(reg: &LinkTypeRegistry, …)`
(link_types.rs:451) already takes the registry as a parameter — while its own caller
`structural_frontmatter_targets` calls `snapshot()` itself at :390.

---

## 3. The finding that changes this phase — Rust-side threading is necessary and NOT sufficient

The plan assumed the trigger hazard is *wrong* DDL in a child's `sqlite_master`. **It is ABSENT DDL.**

✅ `init_db_scoped` **drops** the sky trigger families *unconditionally*, at top-level indentation,
and **recreates them only under `if owns`** (`owns = scope == InitScope::Active`, search.rs:4602):

| DROP (unconditional) | CREATE (gated) |
|---|---|
| search.rs:5868-5874, :5884-5888 (stratum family) | `if owns` — search.rs:5891 |
| search.rs:5924-5930 (maturity family) | `if owns` — search.rs:5933 |
| the `note_meta_sky_ai` / `_au` drop preceding :5640 | `if owns` — search.rs:5640 |
| — | `if owns` → `create_outgoing_link_triggers` — search.rs:5969 |
| search.rs:5977 `drop_incoming_link_triggers` (unconditional; by design — incoming is Rust-side) | — |

✅ The only production caller of `init_db_schema_only` is `federation::migrate::run_migrations_on`
(federation/migrate.rs:169), reached from `federation/attach.rs:172` whenever a linked universe's
schema is stale — **the ordinary state of a cUniverse not opened since a Constellation update.**

> ### ✅ Consequence, true on `main` today, before any Router exists
> **A linked child universe that the parent has schema-migrated has had its three `note_meta` sky
> triggers stripped and not restored.**

**CORRECTED 2026-08-17, by running the test rather than trusting the reading.** The first version of
this section said such a child also has no outgoing-aggregate triggers. **That is wrong.**
`drop_outgoing_link_triggers` is called on the FIRST LINE of `create_outgoing_link_triggers`
(search.rs:2290), which is itself inside the `if owns` gate at search.rs:5969 — so for a foreign
database the outgoing family is neither dropped nor created, and it **survives**. The PJ-302 red test
named the true casualty list on its first run:

```
Stripped: ["note_meta_sky_ai", "note_meta_sky_stratum_au", "note_meta_sky_maturity_au"]
```

That is the whole finding, and the error is worth recording: I asserted a blast radius from reading
the gates instead of executing them, in a document whose own §7 warns against exactly that. A
first-mismatch assertion would have hidden it a second time; the test now reports every casualty.

**Two distinct states, and the probe must handle both:**
- **A child opened by its own process, then parent-migrated** — outgoing triggers present, the three
  `note_meta` sky triggers gone.
- **A child never opened by its own process** — nothing was ever created, so it has neither family.

### What a routed write into such a child would do

Rust-side threading works perfectly — `note_links.link_type` correct, `maintain_incoming_after_save`
correct — and then:

1. ✅ **`note_meta.outgoing_*` go stale only in the never-opened case.** Where the owner created the
   triggers they survive and maintain correctly. Where it did not, nothing writes those columns at
   all: I verified every call site of `outgoing_aggregate_assignments` — the only production ones are
   the trigger DDL (search.rs:2327-2328) and `links_backfill::recompute_range` (links_backfill.rs:248);
   the other three (search.rs:6796, links_backfill.rs:739, cache.rs:1646) are `#[cfg(test)]`.
   **It is not on the save path at all.**
2. ✅ **The note gets no `sky_nodes` row — in BOTH states.** The only production `INSERT … INTO
   sky_nodes` is search.rs:5655, inside `note_meta_sky_ai`, which is stripped in the first state and
   never created in the second. No row ⇒ no `stratum`, no `maturity` — absent rather than wrong.
3. ⚠️ Every existing `note_links` row pointing at the note keeps `target_cid_cn` NULL — the PJ-207 §15
   back-resolution lives in that same absent trigger body (search.rs:5672-5684, agent-reported).
4. ✅ **`maintain_sky_after_save` cannot repair it.** Its body (search.rs:2706-2726) is
   `UPDATE sky_nodes SET … WHERE path = ?1`. With no row it affects zero rows and returns `Ok(())`,
   so `maint.sky_failed` stays FALSE (search.rs:12765). **The write reports success.**

✅ And the safety premise that made the `owns` gate correct is stated verbatim at search.rs:5966-5968:

> *"The owner creates them correctly on its own next launch; until then nobody writes through them,
> **because the parent attaches a cUniverse read-only.**"*

**Phase 1.2 is precisely the change that falsifies that sentence.** No vocabulary-threading design
touches it. Row counts stay correct; success is reported; nothing surfaces it — the exact silent
class this migration exists to end.

**The ruling this forces: a routed open must PROBE the child's trigger set and REFUSE if incomplete.
It must not repair it** — the parent recreating a child's trigger DDL is exactly the parent-flavoured
DDL hazard PJ-232 closed (federation/migrate.rs:142-167).

✅ **A second, smaller contradiction found in passing.** `InitScope`'s doc comment (search.rs:4576)
claims *"every DDL whose body is generated from the link-type registry"* is skipped for a foreign
database. The `note_links_sky_ai/_ad/_au` block (search.rs:5531-5575) sits **outside** every `if owns`
and interpolates `snapshot().structural_not_in_clause(…)` read at :5540. So `init_db_schema_only`
**does** write parent-registry-generated SQL into a child's `sqlite_master` today — harmless only
because of the structural invariance in §2(a), i.e. **by accident, not by the gate.** The comment is
false as written and should be corrected in the same commit.

---

## 4. The options

| | Mechanism | Prod sites touched | Effort | Cannot-forget? |
|---|---|---|---|---|
| **A** | Bare `&LinkTypeRegistry` parameter, hoisted where the DB is chosen | ~25 → ~117 | 1–3 d | **No.** `&snapshot()` type-checks fine |
| **B** | `Db<'a> { conn, vocab }`, `Copy`, **no `Deref`** | ~72 prod + ~65 test | ~1 d | Partly — `db.conn()` is a greppable escape hatch |
| **C** | `WriteScope` with **one** constructor taking a PATH, **plus deleting the three free-function global readers** | ~75 prod | 1.5–2 d | **Closest** |

**Ranking: C > B > A.**

- **A** is the honest baseline and its own author concedes the failure: the parameter makes the choice
  *visible*, not *correct*. It survives the concurrency attacks (debounced save, backfill tick,
  universe switch) and fails the ones that matter — a new call site next year is a silent wrong value,
  not a compile error.
- **B** contributes the single best constraint in the analysis — **refuse `Deref`**, because `Deref`
  would make every un-migrated `&Connection` function silently reachable — then over-claims that its
  parameter fixes the five DDL sites. It does not: four are vocabulary-invariant and the fifth is
  `owns`-gated off for a foreign database. B also *introduces* a hazard that does not exist today: a
  vocabulary read live from disk at open, paired with trigger bodies frozen from whenever
  `create_outgoing_link_triggers` last ran — one write computing `note_links` under V1 and
  `note_meta.outgoing_*` under V2. Today both halves come from one global and **cannot** disagree.
- **C** is right for one reason the others lack: **its central move is a deletion.** Removing
  `is_known_type` (link_types.rs:359) and `is_structural_type` (:369) — whose bodies are already
  `LinkTypeRegistry::is_known` (:171) / `is_structural` (:229), existing *only* to read the global —
  removes the ambient alternative instead of discouraging it. And its constructor takes a **path**,
  resolving the Owner itself, so a caller cannot select a universe and therefore cannot select the
  wrong one. Its honest residue: `snapshot()` stays `pub` over a `pub` struct with `pub` methods, so
  a new ambient read remains one clean compiling line (→ Invariant 9, Boss question 4).

---

## 5. Recommendation

**Adopt C's shape — one owner-resolving constructor plus the three deletions — with B's honest
boundary (the bundle stops at the connection layer; a threaded `&LinkTypeRegistry` continues below
it), staged in two, and gated by three preconditions no option contained.**

```rust
// link_types.rs — the missing door. Pure, root-parameterised, STRICT.
pub fn registry_for_root(root: &Path) -> Result<LinkTypeRegistry, String>;

// search.rs — ONE constructor. The caller supplies a PATH, never a universe, never a vocabulary.
pub struct WriteScope<'a> { conn: ConnRef<'a>, vocab: LinkTypeRegistry, owner: Owner }
impl<'a> WriteScope<'a> {
    pub fn for_note(app: &tauri::AppHandle, note_path: &str) -> Result<Self, String>;
    pub fn conn(&self)  -> &Connection;        // greppable escape hatch, deliberately named
    pub fn vocab(&self) -> &LinkTypeRegistry;  // what travels below the connection layer
}
```

`for_note` calls `federation::owner::resolve_owner(app, path)` (owner.rs:149) **itself** and branches
on `owner.is_active`: active ⇒ borrow `state.db`, `vocab = snapshot()`, **byte-identical to today**;
routed ⇒ the three preconditions, then a dedicated open in the shipped `reconcile_filesystem` shape
(WAL, `synchronous=NORMAL`, **`recursive_triggers=ON`**, `register_fts5_tokenizer`).

**Not a pool — and that is a decision, not an omission.** Nothing cached, nothing held open, for four
reasons: `run_migrations_on` explicitly `drop(conn)`s to release the child's file lock or
`attach_with_safety`'s re-open blocks (federation/migrate.rs:171-175); `federation_generation` bumps
only on universe switch, so unlinking a child leaves a stale entry undetected; `universe_lock` logs
*"NOT ENFORCED YET (MIG-111 Phase 1.4)"*, so a held child connection has no ownership story; and
routed writes are rare. An LRU keyed by `universe_lock::canon(owner.root)` drops in behind the same
constructor later and changes no call site. **Read "the routed context pool" as *the routed context*;
the pooling is deferred.**

### The three preconditions — each a REFUSAL, never a repair

1. **Trigger-capability probe (§3).** Query the child's `sqlite_master` for `note_meta_sky_ai`,
   `note_meta_sky_stratum_au`, `note_meta_sky_maturity_au`, `note_links_outgoing_ai/_ad/_au`. Any
   absent ⇒ refuse, naming the universe.
2. **Existence check before `Connection::open`.** `search.db` must already exist — `Connection::open`
   *creates* it (search.rs:4603), and one failed routed write would otherwise convert a clean
   "skipped, warned" federation state into a permanently parent-migrated child schema.
3. **Strict vocabulary read.** ✅ `registry_for_root` goes through `universe::read_persisted_json`
   (universe.rs:260-290 — verified strict on every branch: only `NotFound` returns `Ok(None)`;
   permission-denied returns `Unreadable` with *"Refusing to treat it as empty"*; zero-length and
   parse-failure return `Corrupt`), never through `read_deltas`'s lenient
   `let Ok(data) = … else { return Vec::new() }` (link_types.rs:517). The codebase already ruled on
   this exact split one screen away at link_types.rs:526-531.

### Staging

- **Stage A** (removes the `#[ignore]`) — `registry_for_root`; `WriteScope`; the three preconditions;
  the three deletions; G2's parse chain (6 sites); G3's save-path pair; the harness rewrite (§7).
- **Stage B** (same pass — Whole-Ecosystem Fix Law, not "next") — the rest of G3 including the
  backfills' `recompute_*` **functions** (not just the generators, or `active_universe_snapshot()`
  gets written into six call sites by construction); G1's DDL generation; G5; G6; and the three
  filesystem re-walkers that share the parse chain (strata.rs, inspector360.rs, libraries.rs) —
  converting only search.rs's chain would make them disagree with the index about the same note,
  breaking MIG-067's `ONE PARSER, ONE ANSWER` (link_types.rs:380).

### What I would NOT do

- **Not** let the parent create or repair a child's trigger DDL. PJ-232 closed that; §3 is an argument
  to *refuse*, not to reopen it.
- **Not** lean on §2(a)'s structural invariance as a scope reduction. Pin it with a test instead.
- **Not** ship Stage A on the acceptance test as written — see §7.
- **Not** build the pool now.
- **Not** claim the deletions deliver "cannot forget." They deliver *cannot forget at the ten sites
  that exist today.*

---

## 6. Invariants that must not break

| # | Invariant | Verified how |
|---|---|---|
| 1 | **Zero keystroke-latency regression.** Threading is a net *win*: today `parse_link_body` takes an `RwLock` read **per wikilink**, and `structural_frontmatter_targets` + both aggregate generators clone the whole registry **per call**. One snapshot per `WriteScope` collapses all of it. **Hard rule:** `registry_for_root` reads a FILE — hoist to the constructor, never inside a per-note loop. | 10-char burst in NotePane + FocusPane on the 7,820-note universe, before/after |
| 2 | **Boot time unchanged.** No `WriteScope` at boot. | boot measured on the same universe |
| 3 | **The active-universe path byte-identical.** | snapshot-test the four generated SQL strings under the seed registry vs the pre-change strings |
| 4 | **R4 — no fingerprint stamp on a routed open.** Exactly two production writers: links_backfill.rs:464, incoming_links_backfill.rs:173. | assert no `schema_versions` row changes across a routed write |
| 5 | **R10 — child DB opened lazily, never on boot.** ⚠️ Name the pre-existing exception in the plan: `federation_prewarm` already opens child DBs read-write at boot — out of 1.2's scope, but nobody should read R10 as already true. | call-graph test + boot trace |
| 6 | **No process-global mutation for a duration (LL-047).** ⚠️ The window is *wider* than the harness comment says: `list_link_types` calls `set_active` (link_types.rs:588), so merely OPENING the Links editor mutates the global mid-flight. | rewritten swap test (§7) + CI grep on `set_active` |
| 7 | **A routed write never writes DDL or `.md` files into a child.** | filesystem-hash assertion over the child root across a routed write |
| 8 | **Earned Living-Link data survives.** `is_structural_type` (search.rs:8485) feeds `link_row_is_preserved`, which decides whether `weight` / `confidence` / `traversal_count` survive a re-index — and CLAUDE.md records that this data lives ONLY in `search.db`. | extend `Aggregates` (§7a) |
| 9 | **The ambient door stays closed.** | rename `snapshot()` → `active_universe_vocabulary()`, `pub(crate)`, + a call-site-pinning test. Stronger option → Boss question 4 |
| 10 | **`recursive_triggers=ON` on the routed connection** — per-CONNECTION; search.rs:4620-4646 documents the exact silent failure when off. | `PRAGMA recursive_triggers == 1` |
| 11 | **ONE PARSER, ONE ANSWER** (link_types.rs:380). | index a note, re-walk it, assert identical typed-link output |

---

## 7. How the acceptance test gets driven — and why it cannot repeat Phase 1.1

**The Phase 1.1 failure, named precisely:** nine tests were green over an inverted entry point because
every one drove the *pure function* with hand-built values. `resolve_owner_in` was tested
exhaustively; `resolve_owner` — the wrapper that assembles the real arguments — was tested by nobody.
**The bug lived in the assembly, and the assembly is exactly what the tests replaced with literals.**

✅ **The current harness has the identical shape and would repeat it.** `index_under_vocabulary`
(vocab_harness.rs:135-160) calls `set_active`, `init_db`, `index_note` and
`maintain_incoming_after_save` **directly**, bypassing `reindex_single_note` — and therefore bypassing
the `incoming_links_backfill::is_built(conn)` gate at search.rs:12712, which on a genuinely fresh
database is **FALSE**, so production sets `inc_old = None` and **skips
`maintain_incoming_after_save` altogether** (search.rs:12753). *The harness observes values production
would not have written.* Same class of divergence, same migration, one phase later.

### Three rules that make it immune

1. **Drive the production funnel, not its parts.** Call `reindex_single_note` (the routed funnel), so
   the test crosses every gate production crosses.
2. **Build the child universe with the app's own constructors** — `universe::create_universe`,
   `universe::add_child_universe` (this is what makes `resolve_owner` *real* rather than hand-fed),
   and `link_types::save_universe_link_types` (which, because it resolves through the *active*
   universe, means the test writes the child's vocabulary **while the child is active**, then switches
   back — literally how a child's vocabulary comes to exist on disk). No literal in the fixture that a
   future signature change could leave stale.
3. **The Owner is RESOLVED, not constructed.** The test never writes `Owner { root, is_active }` —
   which compiles today, since both fields are `pub` (owner.rs:69, :72). It calls
   `WriteScope::for_note(&app, &child_note_path)`. **If `resolve_owner` ever inverts again, this test
   goes red** — making 1.2's acceptance test the first production-shaped consumer of 1.1's entry
   point, which is the direct structural repair of the Phase 1.1 miss.

### Two harness changes that must land in the same commit

**(a) `Aggregates` must observe what §3 can lose.** It reads exactly four things today
(vocab_harness.rs:73-106). **As written, "green" would certify a routed write that produced no
`sky_nodes` row, no stratum, no maturity, no `target_cid_cn`, and stale `outgoing_*`.** Add the four
`outgoing_*` columns, `sky_nodes.stratum/maturity`, and `weight`/`confidence`/`traversal_count` on the
edges tuple.

**(b) `a_vocabulary_swap_reaches_back_into_an_already_open_database` must be deliberately
re-authored, in the same commit.** It will stop compiling when `index_note`'s signature changes; its
doc comment pre-authorises that outcome, and severing the coupling **is** the LL-047 ruling. **The
danger is that the smallest repair that makes the file compile is passing `&link_types::snapshot()`,
which restores the exact anti-pattern and turns the assertion green again.** Rewrite it into its
positive form: construct the scope BEFORE the swap, assert the result is UNCHANGED by it.

---

## 8. Open questions for the Boss

1. **When a routed write meets a child whose triggers the parent already dropped (§3), what should the
   user see?** Recommendation: **refuse**, naming the universe. The alternative — the parent
   recreating the child's triggers — is now technically expressible but reopens what PJ-232 closed.
   User-visible, so it needs a ruling.
2. **§3 is a defect on `main` today, independent of 1.2.** Latent only because nothing writes to a
   cUniverse. **Separate PJ fixed first, or absorbed into Stage A?** Recommendation: **separate** — it
   has its own reproduction, and mixing it in muddies 1.2's diff.
3. **Does 1.2 route renames?** libraries.rs:7490 is the only vocabulary read whose answer reaches a
   `.md` FILE. It is fenced from child universes today; if renames are in scope, that fence is what
   is being removed. Same question for the watcher, fenced by a single identifier at search.rs:12971
   (`try_load_libraries`) — swapping it for `load_all_libraries` would enable routed child writes with
   **zero compile errors**. Both fences need naming in the plan, with a test on each.
4. **Invariant 9's strength** — call-site-pinning test (cheap) vs a newtype return forcing a named
   unwrap at each remaining site (more churn, makes every ambient read a written justification in a
   diff). Recommendation: **the newtype**, because this migration has twice been burned by a property
   that was true but undeclared.

---

## 9. Filed in passing — not in 1.2's path

- ✅ **A flaky test makes every red→green claim in this migration probabilistic.**
  `arabic::fst_bake::tests::persist_then_try_load_cached_roundtrip` failed 1 run in 6 (measured over
  6 full runs: 1500/0 five times, 1499/1 once). Mechanism, read from source: the test writes a
  hand-built bundle to the **real user cache path** (`cache_file_path()`, fst_bake.rs:969-973) and
  reads it back, while the production Arabic-index init path (fst_index.rs:119, :134) calls the same
  `try_load_cached` / `persist_best_effort` against the same file. A concurrent test triggering that
  init atomically renames the real bundle over the test's — and the assertion prints a large genuine
  FST blob, which is exactly what it printed. **Same disease as LL-047, one layer out:** a shared
  mutable resource with a window. Fix shape: make the cache path injectable so the test cannot
  collide — a structure that cannot forget, not a promise.
- ✅ **`tension.rs:88-92` contains a false claim** — it asserts `validate_path_in_any_library` refuses
  cUniverse library paths. It does not; libraries.rs:727-728's own doc says "including child universe
  libraries". Anyone reading it as the federation contract for the analytics family will conclude
  those surfaces are fenced when they are not.

**UNVERIFIED, and what settles each:** whether `attach_with_safety`'s ATTACH is genuinely `mode=ro`
(read attach.rs:212-250); whether the frontend ever hands a cUniverse library path to the G5 analytics
commands (grep the Svelte `invoke(` sites); whether `converge.rs` / `derived_heal.rs` read the registry
(the repo-wide grep shows no hits, which is evidence, not a reading).
