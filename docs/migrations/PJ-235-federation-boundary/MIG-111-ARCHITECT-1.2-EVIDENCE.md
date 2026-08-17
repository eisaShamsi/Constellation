# MIG-111 Phase 1.2 — ARCHITECT EVIDENCE

Produced 2026-08-17 by a 14-agent workflow (13 returned; the maintenance-computation map slice
failed its schema retries and was mapped by direct reading instead). Raw agent output, unedited.
The edited, personally-verified deliverable is `MIG-111-ARCHITECT-1.2.md` — where the two
disagree, the deliverable wins.

---

## A. The call-site map (six slices, 142 sites reported)

### SLICE: MAP SLICE 1 — trigger DDL and schema creation in `src-tauri/src/search.rs` (plus its direct callers in `federation/migrate.rs`, `index_repair.rs`, `mig108.rs`, and the two backfill fingerprint gates). Every claim below was read from source in this session; file:line are from the files as they exist on `main` at 857530f5.

=== THE TWO ANSWERS ASKED FOR DIRECTLY ===

**1. `init_db` takes a PATH, not a connection.**
  `pub(crate) fn init_db(path: &Path) -> Result<Connection, String>` — search.rs:4592, body is one line: `init_db_scoped(path, InitScope::Active)` (search.rs:4593).
  `pub(crate) fn init_db_schema_only(path: &Path) -> Result<Connection, String>` — search.rs:4597 → `init_db_scoped(path, InitScope::ForeignSchemaOnly)` (search.rs:4598).
  `pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>` — search.rs:4601. It **constructs** the connection itself at search.rs:4603 (`Connection::open(path)`) and returns it owned at search.rs:6407 (`Ok(conn)`), 1,807 lines later.
  Consequence for Phase 1.2: there is no seam to hand `init_db` a pre-bound routed connection carrying a vocabulary. A per-connection vocabulary binding cannot be installed at this entry point without changing the signature — the vocabulary must be a parameter of `init_db_scoped` (or of the `InitScope` value), because the connection does not exist until inside it.

**2. `InitScope::ForeignSchemaOnly` — what it skips, verbatim from the comment at search.rs:4574-4588.**
  The comment (written after an earlier vaguer one produced a false claim — search.rs:4576) states two skip classes:
    (a) "every DDL whose body is generated from the link-type registry (that registry is the ACTIVE universe's)";
    (b) "every one-shot MIG-003 pass — Step 1 (writes `cid_cn:` frontmatter into `.md` files, deletes rows whose path no longer stats), Step 2, Step 3 (re-indexes rows, writes frontmatter) and Step 4 (RENAMES `.md` files)".
  NOT skipped (search.rs:4583-4587): "plain schema DDL, and the derived-table population that makes that schema usable (the FTS rebuild, the initial-history back-fill). Those read only the child's OWN rows, involve no process-global state and touch no file — and the parent reads a cUniverse's derived tables through the read-only attach, so withholding them would degrade federated reads without protecting anything."
  Mechanism: `let owns = scope == InitScope::Active;` (search.rs:4602). `owns` is then consulted at exactly 13 places: search.rs:4838, 4863, 5640, 5891, 5933, 5969, 6219, 6242, 6284, 6312, 6335, 6364 (verified by grep for `owns` over search.rs).

**FINDING — claim (a) is NOT true of the code as written.** The sky_link mirror triggers at search.rs:5531-5575 ARE generated from the registry (`sx_new` at search.rs:5540 = `crate::link_types::snapshot().structural_not_in_clause("NEW.link_type")`, interpolated into the `note_links_sky_ai` WHEN guard at :5544 and the `note_links_sky_au` INSERT guard at :5573) and are **not** inside any `if owns` block — the nearest `owns` uses are at :4863 (before) and :5640 (after). So `init_db_schema_only` on a child universe does DROP and re-CREATE three triggers in the child's `sqlite_master` whose bodies were generated from the parent's registry. See `notes` for why the current blast radius of this specific one is zero, and why that is a property of `merge()` rather than of this code.

=== THE VOCABULARY-DEPENDENCE SPLIT (the load-bearing fact for 1.2) ===

The registry exposes two families of SQL generators, and they are NOT equally dangerous:

**Family A — genuinely varies with a universe's vocabulary.** `cognitive_ids()` (link_types.rs:234) = every type NOT flagged structural = the 8 seeds + EVERY user-defined type. So `sql_in_list_cognitive()` (link_types.rs:241), `sql_rank_case_cognitive()` (link_types.rs:253) and `cognitive_sentinel_rank()` (link_types.rs:292) differ between any two universes with different `link-types.json`. These are read at exactly two places: `outgoing_aggregate_assignments` (search.rs:2248-2250) and `incoming_aggregate_assignments` (search.rs:2493-2495). **This is the corruption vector.**

**Family B — provably invariant across all universes.** `structural_not_in_clause()` (link_types.rs:268) filters on `t.structural`. `LinkTypeRegistry::merge` (link_types.rs:115) forces `structural = false` for any delta whose id is a cognitive seed (link_types.rs:131), forces `structural = false` for EVERY custom id (link_types.rs:159 — `d.structural = false; // custom types are cognitive`), and forces `structural = true` only for ids in `STRUCTURAL_SEED_IDS` (link_types.rs:148), which is the fixed `&["parent", "contains"]` (link_types.rs:61). `merge` always seeds from `seeds()` (link_types.rs:121) and deltas can only override or add, never delete. Therefore `structural_ids()` is **invariantly `{contains, parent}` in every universe**, and `structural_not_in_clause(col)` always emits ` AND {col} NOT IN ('contains','parent')` — the exact string asserted at link_types.rs:650. The only variance a delta can introduce is the ORDER of the two ids inside the parenthesis (merge does not pin `order` for structural seeds), which is semantically identical SQL.
  Family B is what `stratum_sql_expr()` (search.rs:188-189) and `maturity_sql_expr()` (search.rs:266-267) read, and what `sx_new` at search.rs:5540 reads. **Those three carry no cross-universe value divergence.**

Also verified: `sql_in_list()` (link_types.rs:197), `sql_rank_case()` (link_types.rs:208) and `sentinel_rank()` (link_types.rs:283) — the non-cognitive variants — have **zero production call sites**; a repo-wide grep finds them only at their definitions and in link_types.rs's own tests.

=== WHERE THE VOCABULARY IS PERSISTED, PRECISELY ===

Into `sqlite_master` (trigger bodies):
  • `note_links_outgoing_ai` / `_ad` / `_au` — created at search.rs:2292/2298/2306 inside `create_outgoing_link_triggers`, bodies = `outgoing_aggregate_assignments("NEW.source_path")` / `("OLD.source_path")` (search.rs:2327-2328). **Family A. This is the one that bakes a universe's custom link types into another universe's schema.**
  • `note_links_sky_ai` / `_au` — search.rs:5542/5559, guard = `sx_new` (Family B).
  • `note_meta_sky_ai` / `_ad` / `_au` — search.rs:5650/5688/5717, bodies interpolate `stratum_sql_expr()` + `maturity_sql_expr()` (search.rs:5840). Family B, `owns`-gated at search.rs:5640.
  • `note_meta_sky_stratum_au` — search.rs:5897, `expr = stratum_sql_expr()` (search.rs:5910). Family B, `owns`-gated at search.rs:5891.
  • `note_meta_sky_maturity_au` — search.rs:5944, `expr = maturity_sql_expr()` (search.rs:5956). Family B, `owns`-gated at search.rs:5933.
  There is NO incoming-aggregate trigger: `init_db_scoped` unconditionally calls `drop_incoming_link_triggers(&conn)` at search.rs:5977 (defined search.rs:2730). The incoming aggregates are maintained by Rust save-path code, so `incoming_aggregate_assignments` never reaches `sqlite_master` — it reaches `note_meta` row VALUES instead.

Into `schema_versions` rows (vocabulary-derived state, not DDL):
  • `links_vocab` — stamped with `LinkTypeRegistry::fingerprint()` (link_types.rs:300) at links_backfill.rs:467, read back at links_backfill.rs:119-126, compared at links_backfill.rs:99.
  • the incoming twin — read at incoming_links_backfill.rs:86, compared at incoming_links_backfill.rs:49.

Registry-FREE DDL confirmed by reading the bodies (so they can be ruled out of 1.2): the FTS triggers `note_meta_ai`/`_ad`/`_au` (search.rs:5147/5150/5153 — static string, no `format!`), `ensure_note_state_history_trigger` (cece/history.rs:112-139), `ensure_sight_v6_invalidation_trigger` (sight_v6.rs:203-217). A scan of every `execute_batch(&format!` / `execute(&format!` inside search.rs:4601-6408 yields exactly six: 5541, 5641, 5892, 5934, 6177 (`PRAGMA user_version` — registry-free) and 6285.
- src-tauri/src/search.rs:2242 `outgoing_aggregate_assignments` via snapshot() | computes: The SQL IN-list (sql_in_list_cognitive, :2248), the rank CASE (sql_rank_case_cognitive, :2249), the empty-sentinel (cognitive_sentinel_rank, :2250) and the structural NOT-IN (:2251) for note_meta.outgoing_count / outgoing_link_types / outgoing_link_types_json / outgoing_top_rank. Family A — genuinely varies per universe. | reached on: init_db (via create_outgoing_link_triggers, search.rs:5970, owns-gated); on_link_vocabulary_changed (search.rs:2780); index_repair TriggerWindow::close (index_repair.rs:461) and its unwind Drop (index_repair.rs:473); mig108 db rewrite (mig108.rs:1203); links_backfill::recompute_all_outgoing (links_backfill.rs:248) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn outgoing_aggregate_assignments(src: &str) -> String
- src-tauri/src/search.rs:2286 `create_outgoing_link_triggers` via other (calls outgoing_aggregate_assignments at :2327 and :2328) | computes: The trigger DDL bodies of note_links_outgoing_ai/_ad/_au (:2292/:2298/:2306), persisted into sqlite_master. Drops first (:2290) so the CURRENT registry's rank CASE + IN-list is always what is stored. | reached on: init_db owns-gated (search.rs:5969-5971); live vocabulary edit (search.rs:2780); reconcile/repair window close + unwind (index_repair.rs:461, :473); MIG-108 unification rewrite (mig108.rs:1203) | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn create_outgoing_link_triggers(conn: &Connection) -> Result<(), String>
- src-tauri/src/search.rs:2488 `incoming_aggregate_assignments` via snapshot() | computes: The IN-list (:2493), rank CASE (:2494), sentinel (:2495) and structural NOT-IN on nl.link_type (:2496) for note_meta.incoming_count / incoming_link_types / incoming_link_types_json / incoming_top_rank. Family A. Never becomes DDL — there is no incoming trigger (init_db drops them at search.rs:5977) — it lands directly in note_meta row VALUES. | reached on: save path via maintain_incoming_after_save (search.rs:2661); search.rs:11040; search.rs:12548; links_backfill.rs:307; incoming_links_backfill (incoming_links_backfill.rs:363 rehearsal); name_fold_backfill.rs:157 | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn incoming_aggregate_assignments(np: &str) -> String
- src-tauri/src/search.rs:188 `stratum_sql_expr` via structural_frontmatter_targets | computes: The sky_nodes.stratum correlated SQL expression; the registry decides ONLY the /*SX*/ replacement (:189, :228) = structural_not_in_clause("link_type"). Family B — provably invariant (link_types.rs:148/159). Baked into note_meta_sky_ai (search.rs:5657), note_meta_sky_stratum_au (search.rs:5901) and the restore UPDATE (search.rs:6286). | reached on: init_db owns-gated (search.rs:5640, :5891, :6284); save path maintain_sky_after_save (search.rs:2719); search.rs:11054; search.rs:12560; sky_backfill.rs:388; links_backfill.rs:359; name_fold_backfill.rs:173 | impact: NO_IMPACT | conn in scope: false
    sig: pub(crate) fn stratum_sql_expr() -> String
- src-tauri/src/search.rs:266 `maturity_sql_expr` via structural_frontmatter_targets | computes: The sky_nodes.maturity CASE chain; the registry decides ONLY the /*SX*/ replacement (:267, :321) = structural_not_in_clause("link_type"). Family B — invariant. Baked into note_meta_sky_ai (search.rs:5658), note_meta_sky_maturity_au (search.rs:5949) and the restore UPDATE (search.rs:6286). | reached on: init_db owns-gated (search.rs:5640, :5933, :6284); maintain_sky_after_save (search.rs:2720); search.rs:11055; search.rs:12561; sky_backfill.rs:399; links_backfill.rs:360; name_fold_backfill.rs:178 | impact: NO_IMPACT | conn in scope: false
    sig: pub(crate) fn maturity_sql_expr() -> String
- src-tauri/src/search.rs:5540 `init_db_scoped` via snapshot() | computes: sx_new = structural_not_in_clause("NEW.link_type"), interpolated into the note_links_sky_ai WHEN guard (:5544) and the note_links_sky_au re-INSERT guard (:5573). These three triggers are DROPped (:5531-5535) and re-CREATEd (:5541-5575) with NO `if owns` gate — the nearest owns uses are :4863 before and :5640 after. So ForeignSchemaOnly DOES persist registry-generated DDL into a foreign sqlite_master, contradicting the InitScope doc at :4577-4581. | reached on: EVERY init_db_scoped call, both scopes: active-universe boot/universe-switch (ensure_search_db_ready, search.rs:11607) AND federation::migrate::run_migrations_on on a linked universe (federation/migrate.rs:169) | impact: NO_IMPACT | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:5640 `init_db_scoped` via snapshot() | computes: `if owns {` guarding the note_meta_sky_ai / _ad / _au trigger DDL (:5650/:5688/:5717). Bodies interpolate stratum_sql_expr() + maturity_sql_expr() at :5840. The comment at :5636-5639 names the reason: these read the ACTIVE universe's registry. Family B expressions, so the gate protects against a hazard that is currently nil in value terms. | reached on: init_db (Active) only — skipped for federation/migrate.rs:169 | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:5891 `init_db_scoped` via snapshot() | computes: `if owns {` guarding CREATE TRIGGER note_meta_sky_stratum_au (:5897), body `UPDATE sky_nodes SET stratum = ({expr})` with expr = stratum_sql_expr() (:5910). The per-edge note_links_sky_stratum_* triggers are deliberately NOT recreated (PJ-066 §B4, :5904-5909); the unconditional DROPs at :5868-5874 shed them from existing DBs on every boot, both scopes. | reached on: init_db (Active) only | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:5933 `init_db_scoped` via snapshot() | computes: `if owns {` guarding CREATE TRIGGER note_meta_sky_maturity_au (:5944), body `UPDATE sky_nodes SET maturity = ({expr})` with expr = maturity_sql_expr() (:5956). Maturity AI is inlined into note_meta_sky_ai (:5935-5938) and intentionally not recreated separately. | reached on: init_db (Active) only | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:5969 `init_db_scoped` via snapshot() | computes: `if owns { create_outgoing_link_triggers(&conn)?; }` (:5969-5971) — THE Family-A DDL site. The comment at :5964-5968 states it exactly: 'the trigger BODIES are generated from link_types::snapshot(), the ACTIVE universe's registry. Creating them in a foreign database would persist our link vocabulary into someone else's sqlite_master.' Today a foreign DB is left with whatever bodies its own owner last wrote; safe only because 'a cUniverse is attached read-only' (:5968). | reached on: init_db (Active) only — skipped for federation/migrate.rs:169 | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:6284 `init_db_scoped` via snapshot() | computes: `if owns {` guarding an UPDATE (not DDL) that stamps stratum/maturity on sky_nodes rows restored by the registry-free INSERT OR IGNORE immediately above (:6276-6283). Uses stratum_sql_expr() + maturity_sql_expr() (:6289-6290). The comment at :6272-6275 is explicit that the INSERT half is deliberately left ungated so a foreign DB still gets its missing rows back. | reached on: init_db (Active) only | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:6312 `init_db_scoped` via other (index_note parse chain → is_known_type / structural_frontmatter_targets) | computes: `if owns {` guarding mig003_step3_soft_rebackfill (:6313). That pass re-indexes every note_meta row with cid_cn='' by calling index_note (search.rs:4280), which is the full parse chain — extract_wikilinks → structural_frontmatter_targets (search.rs:7079), is_known_type (search.rs:7244, :7371). So this is the path by which init_db reaches the PARSE-time vocabulary, not just the SQL-generation vocabulary. The gate comment (:6308-6311) notes the candidate set is non-empty BY CONSTRUCTION on a schema-drifted child. | reached on: init_db (Active) only; ungated on every Active boot (no version stamp) | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:4211 `mig003_step3_soft_rebackfill` via other (calls index_note, search.rs:4280) | computes: Nothing from the registry directly; it REACHES the parse-time vocabulary through index_note, and its writes fire whatever triggers the database currently carries. Takes &mut Connection — so unlike init_db this one already has a connection to bind a vocabulary to. | reached on: init_db owns-gated (search.rs:6313) | impact: UNVERIFIED | conn in scope: true
    sig: pub(crate) fn mig003_step3_soft_rebackfill(
    conn: &mut Connection,
    db_dir: &Path,
) -> rusqlite::Result<()>
- src-tauri/src/search.rs:2771 `on_link_vocabulary_changed` via other (create_outgoing_link_triggers at :2780, which reads snapshot()) | computes: The live-edit refresh: recreates the outgoing triggers so the new rank CASE + IN-list is persisted, then schedules both re-materializes (links_backfill::maybe_schedule :2786, incoming_links_backfill::maybe_schedule :2792). It reaches the connection ONLY through `app.state::<SearchState>()` → `state.db.lock()` (:2773-2779) — i.e. the ACTIVE universe's connection and nothing else. Called from link_types.rs:563, gated on a fingerprint change (link_types.rs:562). | reached on: save_universe_link_types IPC command (link_types.rs:544) | impact: SKIPS_MAINTENANCE | conn in scope: false
    sig: pub fn on_link_vocabulary_changed(app: &tauri::AppHandle)
- src-tauri/src/search.rs:11476 `ensure_search_db_ready` via load_active | computes: The ordering that makes the whole scheme work for the active universe: `crate::link_types::load_active(app);` at :11606 immediately before `let conn = init_db(&path)?;` at :11607. The comment (:11602-11605) states the intent — load the vocabulary BEFORE init_db so the triggers init_db creates carry the right rank CASE + IN-list. This is the ONLY place the global is loaded on the boot path, and it loads from the ACTIVE universe's .constellation/link-types.json (link_types.rs:507-509). | reached on: boot, universe switch (invalidate_search_state, search.rs:11228 resets state.db so this re-runs) | impact: NO_IMPACT | conn in scope: false
    sig: pub fn ensure_search_db_ready(app: &tauri::AppHandle) -> Result<(), String>
- src-tauri/src/federation/migrate.rs:169 `run_migrations_on (the closure at :168)` via other (calls search::init_db_schema_only) | computes: The ONE production call site that hands a FOREIGN universe's search.db to the init path. The 40-line comment above it (:122-167) is the PJ-230/PJ-232 record: it names create_outgoing_link_triggers and the Sky stratum/maturity triggers as persisting 'parent-flavoured DDL into the child's sqlite_master' (:146-152), and states the divergence 'only when the two universes' link vocabularies actually differed (a user-defined type on either side); with seeds only the generated SQL is identical, which is why nobody had seen it' (:159-161). It ALSO explicitly forbids the ruled-out design: 'do not "fix" it by loading the child's vocabulary into the global first — that means swapping a process-global on a background thread while every other subsystem reads it' (:134-136). | reached on: cUniverse attach with a schema too old — the federation auto-migrate path | impact: PERSISTS_WRONG_DDL | conn in scope: false
    sig: let conn = crate::search::init_db_schema_only(cu_db_path)
            .map_err(|e| format!("init_db failed: {}", e))?;
- src-tauri/src/index_repair.rs:461 `TriggerWindow::close` via other (create_outgoing_link_triggers) | computes: Recreates the outgoing-aggregate triggers after a repair's trigger-free bulk window, regenerating their bodies from whatever the global registry holds AT CLOSE TIME — not at open time. The window is opened at :428-439 (drop_outgoing_link_triggers :432, drop_incoming :435, drop_sky_aggregate :438). Drop (:465-477) does the same on the unwind path. | reached on: index repair / cold-start walk | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn close(mut self) -> Result<(), String>
- src-tauri/src/mig108.rs:1203 `the db-rewrite closure (opened at mig108.rs:1046)` via other (create_outgoing_link_triggers) | computes: Restores the outgoing-aggregate triggers inside the MIG-108 unification transaction (dropped at :1051 for the O(N^2) reason), regenerating bodies from the global registry, then runs converge::after_mig108 (:1207). | reached on: MIG-108 'One Universe, One Location' unification rewrite | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: crate::search::create_outgoing_link_triggers(conn)?;
- src-tauri/src/links_backfill.rs:87 `is_needed` via snapshot() | computes: The fingerprint gate for the OUTGOING re-materialize: `stored_vocab_fingerprint(conn) != crate::link_types::snapshot().fingerprint()` (:99). stored side reads schema_versions module='links_vocab' (:119-126); the stamp is written at :467 from a run_fp captured up-front at :160. Compares a value PERSISTED IN A DATABASE against a value read from a PROCESS-GLOBAL. | reached on: boot backfill scheduling; on_link_vocabulary_changed (search.rs:2786) | impact: WRONG_READ_ONLY_ANSWER | conn in scope: true
    sig: fn is_needed(conn: &Connection) -> bool
- src-tauri/src/incoming_links_backfill.rs:48 `is_stamped` via snapshot() | computes: The INCOMING twin of the gate: `is_built(conn) && stored_vocab_fingerprint(conn) == crate::link_types::snapshot().fingerprint()` (:49). Its answer also flips readers between the materialized aggregates and the live getBacklinks path (:43-47), so a wrong answer here is a wrong READ, not only a skipped write. | reached on: backlink reads; save-path maintenance gate; reconcile recompute gate (:36) | impact: WRONG_READ_ONLY_ANSWER | conn in scope: true
    sig: pub(crate) fn is_stamped(conn: &Connection) -> bool
- src-tauri/src/link_types.rs:498 `snapshot` via snapshot() | computes: The read primitive every site above goes through: clones the process-global REGISTRY (link_types.rs:351, `static REGISTRY: OnceLock<RwLock<LinkTypeRegistry>>`) under a read lock, falling back to seeds_only() on a poisoned lock. Takes no arguments — there is no seam here to pass a universe, a path, or a connection. | reached on: every site in this table | impact: NO_IMPACT | conn in scope: false
    sig: pub fn snapshot() -> LinkTypeRegistry
- src-tauri/src/link_types.rs:522 `load_active` via load_active | computes: set_active(read_deltas(app)) — replaces the global from the ACTIVE universe's .constellation/link-types.json (path built by link_types_path at :507-509 via universe::active_constellation_dir). Lenient by design (:511-513): an unreadable file yields the 8 seeds rather than an error. | reached on: boot / universe switch via ensure_search_db_ready (search.rs:11606) | impact: NO_IMPACT | conn in scope: false
    sig: pub fn load_active(app: &tauri::AppHandle)
- src-tauri/src/link_types.rs:481 `set_active` via set_active | computes: Takes the write lock on the process-global and replaces it with merge(deltas). Three production callers: load_active (:523), save_universe_link_types (:554), and — notably — list_link_types (:588), an ordinary read-shaped IPC command the Links editor calls, which MUTATES the global as a side effect. That third one widens the H1b window beyond 'boot and vocabulary save'. | reached on: boot; vocabulary save; the list_link_types IPC command (:585) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub fn set_active(deltas: Vec<LinkTypeDef>)
NOTES: CALL-GRAPH SHAPE — how deep the threading has to go.

**Up from the DDL.** There are exactly two production entries into the init path: `ensure_search_db_ready` → `init_db` (search.rs:11607) for the active universe, and `federation::migrate::run_migrations_on` → `init_db_schema_only` (federation/migrate.rs:169) for a linked one. Everything else calling `init_db` is test code (verified: all 40-odd other hits are inside `#[cfg(test)]` modules — search.rs cfg(test) boundaries at 491/598/884/975/1026/1109/1158/1971/2031/6676/6765/… — plus the fixture openers in index_repair.rs:992, converge.rs:541, review_rehearse.rs:241/334, libraries.rs:7833/7884/7945, link_life_*.rs, target_base_backfill.rs:506, mig108.rs:2191, and federation/vocab_harness.rs:142/234).

**Down from the DDL.** Once the trigger text is in `sqlite_master`, the vocabulary is no longer read at write time — it is FROZEN in the trigger body. That splits the threading problem in two, and they need different mechanisms:

  (i) **DDL-time.** `create_outgoing_link_triggers(conn)` already takes a `&Connection`; `init_db_scoped(path, scope)` does not — it opens the connection itself at search.rs:4603. So the vocabulary can be bound at the connection for (i) only if it is passed INTO `init_db_scoped` and carried down to :5970. The cheapest true fix in this slice is to make the vocabulary part of what `InitScope` carries (e.g. `Active` vs `Owned(LinkTypeRegistry)`), since `owns` at :4602 is already derived from it and the six generation sites are all inside that one function. That would also let `ForeignSchemaOnly` become what its own comment already claims it is.

  (ii) **Write-time.** `outgoing_aggregate_assignments` / `incoming_aggregate_assignments` / `stratum_sql_expr` / `maturity_sql_expr` are called freshly on every save (search.rs:2661, :2719-2720, :11040, :11054-11055, :12548, :12560-12561) and by every backfill (links_backfill.rs:248/307/359-360, sky_backfill.rs:388/399, name_fold_backfill.rs:157/173/178). All four take either `&str` or nothing — none has a connection in scope, and none can be reached from a connection. Threading here means adding a `&LinkTypeRegistry` parameter to four functions and then to every one of their ~15 callers, and those callers are themselves reached from the save path, the watcher, and background threads.

**The genuinely load-bearing narrowing.** Of the five registry generators reaching DDL, only ONE varies with a universe's vocabulary: `outgoing_aggregate_assignments` (and its non-DDL twin `incoming_aggregate_assignments`). The other three (`stratum_sql_expr`, `maturity_sql_expr`, the `sx_new` guard at search.rs:5540) read `structural_not_in_clause`, and `merge()` pins the structural set to exactly `{contains, parent}` in every universe — link_types.rs:148 sets `structural = true` only for `STRUCTURAL_SEED_IDS`, link_types.rs:131 forces `false` for cognitive seed deltas, link_types.rs:159 forces `false` for every custom type, and `merge` always starts from `seeds()` (link_types.rs:121) so the two can never be removed. If Phase 1.2 wants a smaller blast radius, this is where it is: three of the six DDL sites do not need a vocabulary threaded at all, and the one ungated site (search.rs:5540, which the InitScope doc wrongly claims is gated) is one of those three. **But do not let that become the fix.** The invariance is a property of `merge()`, not a contract anyone declared; the moment a user-defined structural type becomes possible, three more DDL sites silently join Family A. If 1.2 leans on it, that reliance needs a test that fails when `merge` stops forcing `structural = false`.

**H1b's window is wider than the harness comment says.** `federation/vocab_harness.rs:38-46` frames the swap hazard around `set_active`. Verified third caller: `list_link_types` (link_types.rs:585) — a read-shaped IPC command the Links editor calls to populate itself — calls `set_active(deltas)` at :588 before returning. So merely OPENING the link-types editor mutates the process-global mid-flight. Any routed-write design that reasons about "when could the vocabulary change under us" must count that one, not just boot and save.

**What `init_db` does that a routed pool must decide about, beyond DDL.** Inside `init_db_scoped` the `owns` flag also gates: the MIG-003 Step 2 dependent-table backfill (:4838), the sky_nodes stratum/maturity restore stamp (:6284), `mig003_step3_soft_rebackfill` (:6312 — which re-indexes rows through `index_note` and can write `cid_cn:` frontmatter into `.md` files), and the MIG-003 Step 4 file-rename pass (:6335). Step 3 is the one that reaches the PARSE-time vocabulary (`is_known_type`, `structural_frontmatter_targets`), which is a different global read than the SQL generators — so a routed context that only fixes SQL generation would still mis-parse here.

**Notable absence.** There is no incoming-aggregate trigger anywhere. `init_db_scoped` unconditionally calls `drop_incoming_link_triggers(&conn)` at search.rs:5977 (MIG-079 §C.2a), so `incoming_aggregate_assignments` never reaches `sqlite_master` — it is executed as UPDATE statements from Rust on the save path. That means the incoming half of the corruption cannot be fixed by fixing DDL generation; it is purely a write-time threading problem.

**Two persisted fingerprints are compared against a process-global.** links_backfill.rs:99 and incoming_links_backfill.rs:49 both read a fingerprint stored IN a database and compare it to `snapshot().fingerprint()` from the global. For a child database, a stamp written under the child's vocabulary will never equal the parent's fingerprint — so a routed pool that touches a child's backfills will see "needed / not stamped" perpetually, and incoming_links_backfill.rs:43-47 says an unstamped answer also flips readers to the live getBacklinks path. That is a read-side consequence of the same coupling, and it does not go away by fixing the write side.
UNVERIFIED: Whether the sky_link trigger DDL at search.rs:5531-5575 being un-`owns`-gated was a deliberate exception or an oversight. The InitScope doc (search.rs:4577-4581) says every registry-generated DDL body is skipped; this one is not. I found no comment anywhere in search.rs or federation/migrate.rs acknowledging the exception. To settle: `git log -S "sx_new" -- src-tauri/src/search.rs` and the PJ-232 commit message. | Whether `LinkTypeRegistry::merge` forcing `structural = false` for custom types (link_types.rs:159) is a permanent contract or a v1 limitation. The comment reads `// custom types are cognitive` with no rationale. The whole Family-A/Family-B narrowing above rests on it. To settle: read the PJ-065 design doc and the frontend mirror `src/lib/.../linkTypeRegistry.ts` (referenced at link_types.rs:368 and :492) to see whether the editor can even offer `structural` on a custom type. | What `crate::converge::after_mig108` (called at mig108.rs:1207) and `derived_heal::maybe_schedule` (referenced at search.rs:6001-6004 as the relocated home of the five-family heal) read from the registry. Both run the recompute tail that PJ-230 identified as the original foreign-write vector; I did not open converge.rs or derived_heal.rs this session. To settle: read both files for `link_types::` and for calls to the four SQL generators. | Whether `federation::migrate::run_migrations_on` is the ONLY path by which the parent process opens a child's search.db for WRITE. I verified it is the only production `init_db*` call on a foreign path, and the comments at search.rs:5968 and federation/migrate.rs:167 both assert cUniverses are attached read-only — but I did not read the attach code. To settle: read `federation/attach.rs` (or wherever `attach_with_safety`, named at federation/migrate.rs:173, lives) and confirm the ATTACH is `mode=ro`. | The read-path sites outside this slice that also call `snapshot()`: sight.rs:77, tension.rs:277, cache.rs:516/548/1288, sky_backfill.rs:283, and the `is_structural_type` consumers at search.rs:8018/8485/8561/9550. I did not open their bodies — they belong to the parse-chain and read-path slices, and I am not asserting anything about them. | Whether the six `format!`-built SQL statements I enumerated inside init_db_scoped (search.rs:5541, 5641, 5892, 5934, 6177, 6285) are exhaustive for DDL specifically. I derived that set by scanning lines 4601-6600 for `execute_batch(&format!` and `execute(&format!`. A DDL string assembled into a `let` binding earlier and passed in without a literal `format!` at the call site would not appear in that scan. I did not find one, but I did not prove none exists.

### SLICE: MAP SLICE 2 — the `index_note` parse chain (src-tauri/src/search.rs), MIG-111 Phase 1.2. Every point below `index_note` where `link_types` decides an outcome, plus the two vocabulary-reading siblings that run in the same save transaction's caller frame (`reindex_single_note`), the DDL sites whose stored bodies the routed write EXECUTES, and the complete caller enumeration of `index_note` / `index_note_bulk`.
- src-tauri/src/search.rs:7079 `extract_wikilinks` via structural_frontmatter_targets | computes: [frame 2 below index_note] The set of lowercased wikilink targets declared under a STRUCTURAL frontmatter key (parent:/contains:). Those targets are then skipped by the byte-offset guard at 7087-7092, so this decides the membership of `note_meta.outgoing_links_json`. Internally: link_types.rs:390 snapshot(), :391 early-return when structural_ids() is empty, :406 reg.is_structural(key), :424 resolve_wikilink_type(&reg, …). | reached on: index_note (save via reindex_single_note; watcher flush; boot/repair bulk walk via index_note_bulk; init_db cid soft-rebackfill) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn extract_wikilinks(content: &str) -> Vec<String>
- src-tauri/src/search.rs:7222 `extract_typed_links` via other (pass-through frame — reads no link_types itself; calls parse_link_body at :7232) | computes: [frame 2 below index_note] Regex-scans the frontmatter-STRIPPED body for `[[…]]` bodies and delegates every typing decision to parse_link_body. Called from index_note_impl:8016. A threaded vocabulary must pass through this frame untouched. | reached on: index_note (all paths) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn extract_typed_links(content: &str) -> Vec<TypedLink>
- src-tauri/src/search.rs:7244 `parse_link_body` via is_known_type | computes: [frame 3 below index_note via extract_typed_links; frame 4 via emit_frontmatter_links] THE typed-vs-untyped parse decision, twice: (a) :7249 whether a `head::` prefix is a real type (else the whole `a::b` string is treated as the target name), and (b) :7268 whether the trailing `|segment` is a legacy predicate-last type (else it is a display alias and the link collapses to `associative`). This is the single line that turns `[[refutes::Target]]` into either link_type='refutes', target='target' or link_type='associative', target='refutes::target'. | reached on: index_note (all paths) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn parse_link_body(body: &str) -> Option<(String, String, String)>
- src-tauri/src/search.rs:8018 `index_note_impl` via is_structural_type | computes: [inline in index_note_impl's own body, frame 1] The PJ-065 filter that DROPS any body-authored structural link from `typed_links` (structural edges are frontmatter-only). A type the child calls structural but the parent does not would survive here as a cognitive body edge in the child's note_links; and vice-versa. | reached on: index_note (all paths) | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String>
- src-tauri/src/search.rs:7306 `extract_frontmatter_typed_links` via other (pass-through frame — reads no link_types itself; calls emit_frontmatter_links at :7341 and :7350) | computes: [frame 2 below index_note] Walks the frontmatter block, tracks the current top-level key (crate::yaml_lines::is_top_level_key_line at :7333) and hands each (key, value) chunk to emit_frontmatter_links. Called from index_note_impl:8025. Pass-through frame for a threaded vocabulary. | reached on: index_note (all paths) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn extract_frontmatter_typed_links(content: &str) -> Vec<TypedLink>
- src-tauri/src/search.rs:7371 `emit_frontmatter_links` via is_known_type | computes: [frame 3 below index_note] Property-name-as-type (MIG-086 §F1 D1): whether the frontmatter KEY is a link type, giving link_type = key, else `associative`. This frame ALSO calls parse_link_body at :7378 (which does its own is_known_type read at 7244) — so this branch is the deepest in the whole chain. | reached on: index_note (all paths) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn emit_frontmatter_links(
    wl: &regex::Regex,
    key: &str,
    value: &str,
    out: &mut Vec<TypedLink>,
    seen: &mut std::collections::HashSet<String>,
)
- src-tauri/src/search.rs:8485 `index_note_impl` via is_structural_type | computes: [inline in index_note_impl, frame 1] The `structural: bool` argument to link_row_is_preserved (search.rs:477) — i.e. whether an EXISTING note_links row's earned weight / confidence / traversal_count / archived status is preserved across the rebuild, or discarded. A wrong answer here silently destroys earned Living-Link data in the child universe (the half of the architecture CLAUDE.md documents as living ONLY in search.db). | reached on: index_note (all paths, whenever the edge diff at :8505 says `!unchanged`) | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String>
- src-tauri/src/search.rs:8561 `index_note_impl` via is_structural_type | computes: [inline in index_note_impl, frame 1] Which INSERT the edge gets: the structural row (:8567 — confidence='structural', weight 1.0, traversal 0, `seq` carried) vs the cognitive row (:8579 preserved-restore, or :8601 fresh confidence='hypothesis'). This is the write that lands in the child's note_links. | reached on: index_note (all paths, inside the `!unchanged` rebuild) | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String>
- src-tauri/src/search.rs:2489 `incoming_aggregate_assignments` via snapshot() | computes: The incoming-aggregate SQL: sql_in_list_cognitive() (:2493), sql_rank_case_cognitive() (:2494), cognitive_sentinel_rank() (:2495), structural_not_in_clause("nl.link_type") (:2496). Produces `incoming_count`, `incoming_link_types`, `incoming_link_types_json`, `incoming_top_rank`. NOT below index_note — it is a SIBLING in the caller frame: reindex_single_note:12754 -> maintain_incoming_after_save (search.rs:2637) -> :2661. This is exactly the value the H1 harness diffs (vocab_harness.rs:156, :99-:103). | reached on: save path (reindex_single_note), gated on incoming_links_backfill::is_built at search.rs:12712 | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn incoming_aggregate_assignments(np: &str) -> String
- src-tauri/src/search.rs:189 `stratum_sql_expr` via snapshot() | computes: `structural_not_in_clause("link_type")` substituted into the `/*SX*/` markers of the sky stratum expression (the inbound/outbound edge counts that set sky_nodes.stratum 1-8). Sibling of index_note, not below it: reindex_single_note:12765 -> maintain_sky_after_save (search.rs:2706) -> :2719. | reached on: save path (reindex_single_note); also the sky trigger DDL at search.rs:5891/5933 (owns-gated) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn stratum_sql_expr() -> String
- src-tauri/src/search.rs:267 `maturity_sql_expr` via snapshot() | computes: `structural_not_in_clause("link_type")` substituted into the `/*SX*/` markers of the maturity expression (DISTINCT-source inbound counts driving seed/sapling/evergreen/wilting/canonical). Same sibling path: reindex_single_note:12765 -> maintain_sky_after_save:2720. | reached on: save path (reindex_single_note); also the sky trigger DDL at search.rs:5956 (owns-gated) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn maturity_sql_expr() -> String
- src-tauri/src/search.rs:2243 `outgoing_aggregate_assignments` via snapshot() | computes: The outgoing-aggregate SQL (sql_in_list_cognitive :2248, sql_rank_case_cognitive :2249, cognitive_sentinel_rank :2250, structural_not_in_clause :2251). It is not called at index time — it is baked into the PERSISTED trigger bodies note_links_outgoing_ai/ad/au (create_outgoing_link_triggers, search.rs:2286-2328), which index_note's note_links INSERT/DELETE at :8531/:8557/:8567/:8579/:8601 then FIRE. So the vocabulary that decides the child's outgoing_* columns is whatever was in the global when that child's DDL was last created. | reached on: init_db_scoped -> create_outgoing_link_triggers (search.rs:5969-5971, `if owns` — SKIPPED for a foreign DB per PJ-232); also links_backfill.rs:248/739/740. Executed on every routed index_note write. | impact: PERSISTS_WRONG_DDL | conn in scope: false
    sig: pub(crate) fn outgoing_aggregate_assignments(src: &str) -> String
- src-tauri/src/search.rs:5540 `init_db_scoped` via snapshot() | computes: `sx_new = snapshot().structural_not_in_clause("NEW.link_type")`, interpolated into the note_links_sky_ai WHEN guard (:5544) and the note_links_sky_au re-INSERT WHERE (:5573). VERIFIED NOT `owns`-GATED: the DROP at :5531-5535 and the CREATE at :5541-5575 sit at 4-space indent, i.e. in init_db_scoped's top-level body, between the note_aliases DDL that ends at :5480 and the first `if owns {` at :5640. `federation::migrate` reaches this via init_db_schema_only (search.rs:4597-4598 -> InitScope::ForeignSchemaOnly) at federation/migrate.rs:169 — so a schema-drifted CHILD database receives the ACTIVE universe's structural exclusion baked into its sky_links triggers. The doc comment at federation/migrate.rs:163-164 claims the fix means "no vocabulary-dependent trigger DDL"; these three triggers are vocabulary-dependent and are exempt from the gate. | reached on: init_db (Active) AND federation::migrate::run_migrations_on -> init_db_schema_only (foreign child DB) | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:9550 `structured_search` via is_structural_type | computes: OFF the index_note chain — a READ path. A typed-link filter whose type is structural is forced to match nothing (`1 = 0`). Listed because it has a `&Connection` already in scope and is the same concern: a federated/attached read answers with the ACTIVE universe's notion of "structural". | reached on: read-only search query (SearchFilters.typed_links) | impact: WRONG_READ_ONLY_ANSWER | conn in scope: true
    sig: fn structured_search(conn: &Connection, filters: &SearchFilters, limit: u32) -> Vec<SearchResult>
NOTES: THE CALL CHAIN, EXACT (index_note = frame 0). Deepest vocabulary read sits 5 frames below index_note.

index_note (search.rs:7873)
└─ index_note_impl (search.rs:7884)                                                  [1]
   ├─ extract_wikilinks (search.rs:7068)  — called at :7927                          [2]
   │    ├─ frontmatter_byte_len (search.rs:7051) — called at :7077  (no link_types)  [3]
   │    └─ link_types::structural_frontmatter_targets (search.rs:7079 -> link_types.rs:385)   [3]
   │         ├─ link_types::snapshot()             (link_types.rs:390 -> :498)
   │         ├─ reg.is_structural(...)             (link_types.rs:406 -> :229)
   │         └─ link_types::resolve_wikilink_type  (link_types.rs:424 -> :451)
   ├─ extract_typed_links (search.rs:7222) — called at :8016                          [2]
   │    └─ parse_link_body (search.rs:7243) — called at :7232                         [3]
   │         └─ link_types::is_known_type (search.rs:7244 -> link_types.rs:359)       [4]
   ├─ link_types::is_structural_type — INLINE at search.rs:8018                       [1]
   ├─ extract_frontmatter_typed_links (search.rs:7306) — called at :8025              [2]
   │    └─ emit_frontmatter_links (search.rs:7361) — called at :7341 and :7350        [3]
   │         ├─ link_types::is_known_type (search.rs:7371)                            [4]
   │         └─ parse_link_body (search.rs:7243) — called at :7378                    [4]
   │              └─ link_types::is_known_type (search.rs:7244)                       [5]  ← deepest
   ├─ link_types::is_structural_type — INLINE at search.rs:8485                       [1]
   │    (the `structural` arg to link_row_is_preserved, search.rs:477-483)
   └─ link_types::is_structural_type — INLINE at search.rs:8561                       [1]

THE COST OF THREADING, stated concretely. Six pure-parse frames must gain the parameter, none of which has a Connection today: extract_wikilinks (7068), extract_typed_links (7222), parse_link_body (7243), extract_frontmatter_typed_links (7306), emit_frontmatter_links (7361), plus link_types::structural_frontmatter_targets (link_types.rs:385) which currently calls snapshot() itself at :390 — it already takes `&LinkTypeRegistry` internally at :424 via resolve_wikilink_type, so it is one parameter away from being pure. parse_link_body is reached from TWO parents (7232 and 7378), so it is the one shared leaf. The three inline sites (8018, 8485, 8561) are in index_note_impl's own body and already hold `conn`, so a connection-bound vocabulary would cost them nothing.

THE SIBLINGS THAT ARE NOT BELOW index_note BUT ARE IN THE SAME ROUTED WRITE. The H1 harness (federation/vocab_harness.rs:149-158) already documents this: index_note alone does not reach them.
  reindex_single_note (search.rs:12682)
  ├─ index_note (search.rs:12718)                                       ← the chain above
  ├─ crate::ctse::hooks::on_note_indexed (search.rs:12738)
  ├─ maintain_incoming_after_save (search.rs:2637) — called at :12754
  │    └─ incoming_aggregate_assignments (search.rs:2488) at :2661 -> snapshot() at :2489
  └─ maintain_sky_after_save (search.rs:2706) — called at :12765
       ├─ stratum_sql_expr  (search.rs:188) at :2719 -> snapshot() at :189
       └─ maturity_sql_expr (search.rs:266) at :2720 -> snapshot() at :267
All three of these DO have `&Connection` in the calling frame (reindex_single_note holds `conn` from state.db.lock() at :12688-12689), so a connection-bound vocabulary reaches them without new parameters; an explicitly-threaded one needs three more signatures changed.

WHO CALLS index_note / index_note_bulk — EVERY CALLER IN THE REPO.
Production (4):
  • search.rs:4280 — mig003_step3_soft_rebackfill (search.rs:4211); reached from init_db_scoped at search.rs:6312, inside `if owns {` (Active only).
  • search.rs:4339 — same function, the cid-heal retry after canonical::ensure_cid_cn.
  • search.rs:8725 — index_library_recursive (search.rs:8682), via index_note_bulk. Its only production caller is reconcile_filesystem (search.rs:11968) at search.rs:12039.
  • search.rs:12718 — reindex_single_note (search.rs:12682). This is the single-note funnel.
Tests / harness (all others):
  • federation/vocab_harness.rs:146 (index_under_vocabulary) and :245 (a_vocabulary_swap_reaches_back_into_an_already_open_database).
  • index_repair.rs:1018 — index_note_bulk inside `#[test] #[ignore] fn m1_full_reread_cost` (index_repair.rs:987).
  • libraries.rs:7845, :7887.
  • search.rs tests: 654, 669, 779, 866, 871, 921, 944, 2202, 2215, 13911, 13937, 13948, 13962, 15681, 16096, 16102, 16131, 16147, 16173, 16175, 16193, 16198, 16253, 16284, 16316, 16355, 16373, 16602, 16608, 16625, 16630.
  • index_library_recursive test callers: search.rs:722, 732, 800, 841, 16934.

WHO CALLS reindex_single_note (the real routed-write entry, since it carries index_note plus the three maintenance passes) — 15 production sites:
  search.rs:12275 (constellation_search_reindex, the IPC save hook, search.rs:12251), search.rs:12861 (reindex_md_descendants, search.rs:12815), search.rs:13010 (reindex_changed_paths, the watcher-flush IPC, search.rs:12948), bases.rs:437, shape.rs:214, libraries.rs:1450, libraries.rs:1890, libraries.rs:1967, libraries.rs:2662, libraries.rs:2811, libraries.rs:7071, universe.rs:2488, tasks.rs:540, index_repair.rs:853 (run_cold_start, index_repair.rs:785), reconcile.rs:469, reconcile.rs:565.

TWO THINGS THE PARENT SHOULD KNOW THAT ARE NOT OBVIOUS FROM THE GREP.

(1) The vocabulary is read at index time but the AGGREGATE half of the answer is decided by DDL that was written at init_db time. index_note's writes to note_links (search.rs:8531, 8557, 8567, 8579, 8601) fire the persisted triggers note_links_outgoing_ai/ad/au and note_links_sky_ai/ad/au. Threading a vocabulary through the parse chain fixes note_links.link_type and outgoing_links_json; it does NOT fix note_meta.outgoing_count / outgoing_link_types / outgoing_top_rank, because those come out of SQL text already stored in the child's sqlite_master. PJ-232 gates the outgoing family (`if owns` at search.rs:5969) and the sky stratum/maturity family (`if owns` at :5640, :5891, :5933) — but see (2).

(2) A verified gap in the PJ-232 gate: the note_links_sky_ai/ad/au triggers at search.rs:5531-5575 interpolate `snapshot().structural_not_in_clause("NEW.link_type")` (read at :5540) and are NOT inside any `if owns` block — the first one in init_db_scoped begins at :5640. federation/migrate.rs:169 calls init_db_schema_only on a linked universe's database, so this path writes the parent's structural exclusion into a child's sky-link trigger bodies. The comment at federation/migrate.rs:163-164 asserts the opposite ("no vocabulary-dependent trigger DDL"). With seeds-only vocabularies on both sides the generated SQL is byte-identical, which is why it has not surfaced — the same reason PJ-232 gives for its own bug at federation/migrate.rs:159-161.

WHAT THE HARNESS ALREADY PINS (federation/vocab_harness.rs): `a_vocabulary_swap_reaches_back_into_an_already_open_database` (:227) proves the coupling is call-time, not connection-time, by asserting at :249-255 that a set_active between init_db and index_note changes the result. `routed_write_must_match_the_owners_vocabulary` (:276) is the red→green for 1.2 and currently `#[ignore]` + `panic!`. `index_under_vocabulary` (:135) deliberately calls maintain_incoming_after_save (:156) after index_note because index_note alone does not reach the incoming aggregates — the same asymmetry described above.
UNVERIFIED: Whether any real linked-universe search.db on disk actually carries parent-flavoured `note_links_sky_ai/au` trigger bodies. I verified the CODE PATH that writes them (search.rs:5531-5575 ungated + federation/migrate.rs:169) but did not query a live child database. To settle: `sqlite3 <child>/search.db "SELECT sql FROM sqlite_master WHERE name LIKE 'note_links_sky_%'"` and compare its NOT IN list against that universe's own .constellation/link-types.json. | Whether `crate::review::upsert_schedule_row` / `crate::review::content_hash` / `crate::tag_counts::is_stamped` / `crate::tag_counts::apply_delta` / `crate::sources::extract_sources` / `crate::sources::extract_content_type` / `crate::ctse::hooks::on_note_indexed` read link_types. Established NEGATIVELY by a repo-wide grep for `link_types::` over src-tauri/src (review.rs, tag_counts.rs, sources.rs and ctse/* produce zero hits); I did not open those function bodies. To settle: read each body. | `crate::libraries::migrate_note_db_paths` (libraries.rs:1542, called from index_note_impl:8239 in the cid self-heal arm). I read :1542-1596 (the guard, the deferred-FK setup and the `run` closure) and bounded the function at 1542 to <1683 (next item is `rename_item` at libraries.rs:1683); the repo-wide grep puts libraries.rs's only link_types reads at :4040, :4065 (scan_links_recursive) and :7490 (rewrite_wikilinks_in_text), all outside that range. I did not read lines 1597-1682 line by line. | Exact call-frame depth of `link_types::structural_frontmatter_targets`'s internal `resolve_wikilink_type` (link_types.rs:424 -> :451) as a threading cost — I read both bodies, but resolve_wikilink_type already takes `&LinkTypeRegistry` explicitly, so it is not itself a global reader; only its caller's `snapshot()` at link_types.rs:390 is. | Whether search.rs:9550 (structured_search) is reachable on a routed WRITE. I verified it is a read-only query builder; I did not trace whether the Router's read path routes through it against a child schema.

### SLICE: MAP SLICE 3 — the maintenance computation functions in src-tauri/src/search.rs (+ the four backfill modules and converge.rs that share the same generated SQL). Every claim below was read from source in this session.

THE SHAPE OF THE PROBLEM, AS THE CODE ACTUALLY IS:

There are only FIVE places in this slice that touch `link_types` directly. Everything else is downstream of them:

  1. `stratum_sql_expr()`              search.rs:188  → read at :189  (`snapshot().structural_not_in_clause("link_type")`)
  2. `maturity_sql_expr()`             search.rs:266  → read at :267  (same)
  3. `outgoing_aggregate_assignments`  search.rs:2242 → read at :2243 (`snapshot()`, then sql_in_list_cognitive / sql_rank_case_cognitive / cognitive_sentinel_rank / structural_not_in_clause)
  4. `incoming_aggregate_assignments`  search.rs:2488 → read at :2489 (same four)
  5. the parse-chain predicates        `is_known_type` (search.rs:7244, :7371), `is_structural_type` (search.rs:8018, :8485, :8561, :9550, and via `structural_not_in_clause` above), `structural_frontmatter_targets` (search.rs:7079)

All five are `&self`-free free functions that take NO connection. That is the whole difficulty: `maintain_incoming_after_save` HAS a `&Connection` in scope and still computes with a process-global, because the vocabulary enters two frames below it, in a `-> String` SQL generator that has no way to be told which database the string is about to run against.

THE SAVE-PATH CALL CHAIN, TOP TO BOTTOM (all file:line read this session):

  constellation_search_reindex (search.rs:12251, #[tauri::command(async)])
    └─ ensure_search_db_ready (search.rs:11476) → link_types::load_active(app) at :11606 → init_db at :11607
    └─ reindex_single_note (search.rs:12682)                       ← the funnel; conn = state.db.lock() at :12688-12689
         ├─ incoming_signature (search.rs:2555)   :12711           ← NO vocabulary read (pure SQL over note_links)
         ├─ incoming_links_backfill::is_built     :12712           ← WRITE-side gate, version only, no fingerprint
         ├─ index_note (search.rs:7873) → index_note_impl (7884)   :12718
         │    ├─ extract_wikilinks (7068)   → structural_frontmatter_targets  :7079
         │    ├─ extract_typed_links (7222) → parse_link_body (7243) → is_known_type :7244
         │    ├─ extract_frontmatter_typed_links (7306) → emit_frontmatter_links (7361) → is_known_type :7371
         │    ├─ is_structural_type  :8018   (drop body-authored structural edges)
         │    ├─ is_structural_type  :8485   (link_row_is_preserved's `structural` arg, search.rs:477)
         │    ├─ is_structural_type  :8561   (which INSERT shape the edge gets)
         │    └─ note_meta UPSERT :8167-8189 → fires note_meta_sky_ai (5650) whose BODY carries
         │         stratum_sql_expr() :5657 + maturity_sql_expr() :5658 baked in at init_db time
         │      … and the note_links INSERT/DELETEs fire note_links_outgoing_ai/ad/au (2292-2325)
         │         whose bodies carry outgoing_aggregate_assignments() baked in at :2327-2328
         ├─ maintain_incoming_after_save (search.rs:2637)          :12754  [gated on is_built]
         │    └─ incoming_aggregate_assignments("note_meta")       :2661
         └─ maintain_sky_after_save (search.rs:2706)               :12765  [UNgated — PJ-187]
              ├─ sky_affected_paths (search.rs:2674)               :2713   ← NO vocabulary read
              └─ stratum_sql_expr() :2719 + maturity_sql_expr()    :2720

TWO SEPARATE INJECTION MOMENTS, AND THEY ARE NOT THE SAME:

  (a) CALL TIME — `maintain_incoming_after_save` :2661, `maintain_sky_after_save` :2719-2720,
      `recompute_after_link_status_change` :11040/:11054/:11055, `reindex_delete_note` :12548/:12560/:12561,
      and all six `links_backfill` recompute fns. These build the SQL string on every call. A routed
      write here uses whatever vocabulary is installed at that instant — the LL-047 window.

  (b) INIT TIME, PERSISTED — `create_outgoing_link_triggers` (2286) and the four sky trigger blocks in
      `init_db_scoped` (5540, 5657/5658, 5910, 5956) write the generated SQL into the child's
      `sqlite_master`. Threading a vocabulary through the CALL does not fix these: the wrong vocabulary
      is already frozen into the trigger BODY and fires on every subsequent write by anyone.
      Any 1.2 design that only threads call-time vocabulary leaves (b) uncovered.

A FINDING, READ NOT INFERRED — ONE OF THE (b) BLOCKS IS NOT `owns`-GATED:

  PJ-232 added `let owns = scope == InitScope::Active;` at search.rs:4602 and guards the sky/outgoing
  trigger creation with it at :5640, :5891, :5933, :5969, :6284. But the **sky_links** trigger family at
  search.rs:5531-5575 is NOT inside any `if owns` block — `let sx_new = crate::link_types::snapshot()
  .structural_not_in_clause("NEW.link_type");` sits at :5540 at the function's top-level indentation, and
  the CREATE at :5541-5575 interpolates it unconditionally. So `init_db_schema_only`
  (search.rs:4597, the only production caller being federation/migrate.rs:169) already persists the
  PARENT's structural-exclusion into a CHILD universe's `note_links_sky_ai` / `note_links_sky_au`
  trigger bodies today, before any Router exists.
- src-tauri/src/search.rs:189 `stratum_sql_expr` via snapshot() | computes: the `/*SX*/` structural-exclusion fragment spliced into the sky stratum SQL expression at three of its six signals (outgoing active-edge COUNT >=3 at :201-204, inbound active-edge COUNT >=5 at :205-208, DISTINCT inbound-source COUNT >=3 at :223-226). The 'generalizes' and 'causes'/'supports' signals at :213-222 are HARDCODED literals and are NOT vocabulary-derived. | reached on: index_note (via the note_meta_sky_ai trigger body baked at :5657), maintain_sky_after_save (:2719), reindex_delete_note (:12560), recompute_after_link_status_change (:11054), links_backfill::recompute_sky_range (:359), name_fold_backfill::run (:173), sky_backfill::process_batch (:388), init_db_scoped DDL (:5657, :5910, :6289) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn stratum_sql_expr() -> String {
- src-tauri/src/search.rs:267 `maturity_sql_expr` via snapshot() | computes: the `/*SX*/` structural-exclusion fragment spliced into all four DISTINCT-inbound-source counts of the maturity CASE chain (:271-273 >=10 canonical, :280-282 >=4 wilting, :295-297 >=4 evergreen, :307-309 >=1 sapling). The vocabulary decides which edges are counted as inbound, hence which of seed/sapling/evergreen/wilting/canonical is selected. | reached on: same set as stratum_sql_expr: note_meta_sky_ai (:5658), note_meta_sky_maturity_au (:5956), maintain_sky_after_save (:2720), reindex_delete_note (:12561), recompute_after_link_status_change (:11055), links_backfill::recompute_sky_range (:360), name_fold_backfill::run (:178), sky_backfill::process_batch (:399), init_db_scoped (:6290) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn maturity_sql_expr() -> String {
- src-tauri/src/search.rs:2243 `outgoing_aggregate_assignments` via snapshot() | computes: four things from one snapshot: `sql_in_list_cognitive()` (:2248, the SQL IN-list of type ids), `sql_rank_case_cognitive()` (:2249, the ORDER BY rank CASE), `cognitive_sentinel_rank()` (:2250, the empty-sentinel for outgoing_top_rank), `structural_not_in_clause("link_type")` (:2251). These generate the UPDATE assignments for note_meta.outgoing_count, outgoing_link_types, outgoing_link_types_json, outgoing_top_rank. | reached on: baked into the note_links_outgoing_ai/ad/au trigger bodies at :2327-2328 by create_outgoing_link_triggers; called live by links_backfill::recompute_range (:248) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn outgoing_aggregate_assignments(src: &str) -> String {
- src-tauri/src/search.rs:2489 `incoming_aggregate_assignments` via snapshot() | computes: the same four (`sql_in_list_cognitive` :2493, `sql_rank_case_cognitive` :2494, `cognitive_sentinel_rank` :2495, `structural_not_in_clause("nl.link_type")` :2496), generating the UPDATE assignments for note_meta.incoming_count (:2515, COUNT DISTINCT source_path with the structural NOT-IN applied inside BOTH arms of the matched UNION at :2507 and :2511), incoming_link_types (:2516-2518), incoming_link_types_json (:2519-2521), incoming_top_rank (:2522). | reached on: maintain_incoming_after_save (:2661), reindex_delete_note (:12548), recompute_after_link_status_change (:11040), links_backfill::recompute_incoming_range (:307), name_fold_backfill::run (:157), and the rehearsal print at incoming_links_backfill.rs:363 | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn incoming_aggregate_assignments(np: &str) -> String {
- src-tauri/src/search.rs:2661 `maintain_incoming_after_save` via snapshot() | computes: THE PINNED FUNCTION. Body read in full (:2644-2666): it computes the affected set with NO vocabulary read — incoming_signature (:2644) is plain SQL, symmetric_difference (:2648) is set logic, resolve_incoming_target_paths (:2649) is name/alias resolution — and then at :2659-2662 formats `UPDATE note_meta SET {incoming_aggregate_assignments("note_meta")} WHERE path = ?1`, executed per affected path at :2664. The vocabulary therefore decides ONLY the VALUES written, never WHICH rows are written. That is exactly the H1 shape: same rows, same count, different content. | reached on: the save path — constellation_search_reindex (:12251) -> reindex_single_note (:12682) -> :12754, gated on incoming_links_backfill::is_built at :12712; also the 1500 ms debounced save, the watcher flush (reindex_changed_paths :12948 -> :13010), rename/move/create/task-toggle/base-edit (see notes for the 16 reindex_single_note call sites) | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn maintain_incoming_after_save(
    conn: &Connection,
    note_path: &str,
    old_targets: &std::collections::HashSet<String>,
    old_name: &str,
    old_aliases: &std::collections::HashSet<String>,
) -> rusqlite::Result<()> {
- src-tauri/src/search.rs:2674 `sky_affected_paths` via other (NO registry read — verified by reading the whole body, :2680-2695) | computes: the sky recompute set only: symmetric_difference of the old/new incoming signatures (:2685-2687) plus the source itself when its own targets/name/aliases moved (:2691-2693). Pure set logic. Mapped here because it is half of maintain_sky_after_save and a reader could reasonably assume it reads the vocabulary — it does not. | reached on: maintain_sky_after_save (:2713) only | impact: NO_IMPACT | conn in scope: true
    sig: fn sky_affected_paths(
    conn: &Connection,
    note_path: &str,
    old_targets: &std::collections::HashSet<String>,
    old_name: &str,
    old_aliases: &std::collections::HashSet<String>,
) -> rusqlite::Result<std::collections::HashSet<String>> {
- src-tauri/src/search.rs:2719 `maintain_sky_after_save` via snapshot() | computes: THE SKY WRITE-TIME MAINTENANCE. Body :2712-2726: affected set from sky_affected_paths (:2713), then at :2717-2721 formats `UPDATE sky_nodes SET stratum = ({stratum_sql_expr()}), maturity = ({maturity_sql_expr()}) WHERE path = ?1`, executed per affected path at :2723. Wrong vocabulary changes sky_nodes.stratum (an integer 1-8) and sky_nodes.maturity (one of seed/sapling/evergreen/wilting/canonical) for the source note AND every target whose inbound moved. | reached on: reindex_single_note :12765 — UNGATED (PJ-187 comment at :12762-12763 says sky is independent of the incoming stamp), so it runs on EVERY save even on a fresh universe where the incoming maintenance above is skipped | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn maintain_sky_after_save(
    conn: &Connection,
    note_path: &str,
    old_targets: &std::collections::HashSet<String>,
    old_name: &str,
    old_aliases: &std::collections::HashSet<String>,
) -> rusqlite::Result<()> {
- src-tauri/src/search.rs:2327 `create_outgoing_link_triggers` via snapshot() | computes: interpolates outgoing_aggregate_assignments("NEW.source_path") and ("OLD.source_path") at :2327-2328 into the PERSISTED bodies of note_links_outgoing_ai / _ad / _au. Drops first at :2290 so the bodies always carry the CURRENT registry. This is INIT-TIME injection: the vocabulary is frozen into sqlite_master, and every later edge write in that database recomputes with it regardless of who is writing. | reached on: init_db_scoped :5970 (inside `if owns` at :5969), on_link_vocabulary_changed :2780, and reconcile's drop+recreate around the bulk walk (per the doc comment at :2280-2285) | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn create_outgoing_link_triggers(conn: &Connection) -> Result<(), String> {
- src-tauri/src/search.rs:5540 `init_db_scoped` via snapshot() | computes: `let sx_new = crate::link_types::snapshot().structural_not_in_clause("NEW.link_type");` — spliced into the WHEN guard of note_links_sky_ai (:5544) and the INSERT guard of note_links_sky_au (:5573), deciding which edges enter sky_links at all. ** NOT `owns`-GATED ** — the DROP at :5531-5535 and the CREATE at :5541-5575 are at the function's top-level indentation, outside every `if owns` block (which begin at :5640, :5891, :5933, :5969, :6284). init_db_schema_only therefore already writes the parent's vocabulary into a foreign database's triggers. | reached on: init_db (:4592, active universe) AND init_db_schema_only (:4597) — the latter called from federation/migrate.rs:169 for a schema-drifted cUniverse | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String> {
- src-tauri/src/search.rs:5657 `init_db_scoped` via snapshot() | computes: the note_meta_sky_ai trigger body (:5650-5686) interpolates stratum_sql_expr() at :5657 and maturity_sql_expr() at :5658 as `UPDATE sky_nodes SET stratum = ({stratum_expr})` / `SET maturity = ({maturity_expr})`. This trigger fires on every note_meta INSERT, i.e. on the FIRST index of any note. Guarded by `if owns` at :5640 with the PJ-232 comment at :5636-5639 naming this exact hazard. | reached on: init_db (Active scope only, today) | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String> {
- src-tauri/src/search.rs:5910 `init_db_scoped` via snapshot() | computes: the note_meta_sky_stratum_au trigger body — `UPDATE sky_nodes SET stratum = ({expr}) WHERE path = NEW.path`, fired WHEN NEW.word_count IS NOT OLD.word_count (:5899). Guarded by `if owns` at :5891. | reached on: init_db (Active scope only) | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String> {
- src-tauri/src/search.rs:5956 `init_db_scoped` via snapshot() | computes: the note_meta_sky_maturity_au trigger body — `UPDATE sky_nodes SET maturity = ({expr})`, fired WHEN NEW.modified or NEW.created_at changed (:5946-5947), i.e. on EVERY save by definition (comment at :5941-5942). Guarded by `if owns` at :5933. | reached on: init_db (Active scope only) | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String> {
- src-tauri/src/search.rs:6289 `init_db_scoped` via snapshot() | computes: a one-shot repair UPDATE (:6285-6291) that stamps stratum + maturity onto sky_nodes rows restored for blank-cid notes, using the same two exprs. Guarded by `if owns` at :6284; the registry-free restore INSERT above it at :6276-6283 is deliberately ungated (comment :6272-6275). | reached on: init_db, every boot of the active universe | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String> {
- src-tauri/src/search.rs:7079 `extract_wikilinks` via structural_frontmatter_targets | computes: the set of lowercased wikilink targets declared under a STRUCTURAL frontmatter key, used at :7087-7092 to skip exactly those frontmatter occurrences. Decides membership of note_meta.outgoing_links_json (serialized at :7988, written at :8189). A parent whose registry marks `contains` structural against a child whose registry does not (or vice versa) writes a different outgoing_links_json for the same file — and structured_search reads that column at :9390, :9404, :9434, :9474, :9495, :9508, :9525 and getBacklinks-adjacent code at :13114, :13133, :13660-13661. | reached on: index_note_impl :7927 — every index of every note | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn extract_wikilinks(content: &str) -> Vec<String> {
- src-tauri/src/search.rs:7244 `parse_link_body` via is_known_type | computes: THE PARSE DECISION, TYPED-VS-UNTYPED — the single most load-bearing vocabulary read in the whole slice. `let is_type = |s: &str| crate::link_types::is_known_type(s);` gates both forms: predicate-first `type::target` at :7249, predicate-last `target|type` at :7268. When the type id is NOT in the active registry, the function falls through to :7278/:7281 and returns `"associative"`. That is precisely the harness's `[[refutes::Target]]` case: with the child's vocabulary it is a typed cognitive edge; with the parent's it collapses to the null type — SAME row, same source, same target, different note_links.link_type. | reached on: index_note_impl -> extract_typed_links (:7222) -> :7232, and emit_frontmatter_links -> :7378 | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn parse_link_body(body: &str) -> Option<(String, String, String)> {
- src-tauri/src/search.rs:7371 `emit_frontmatter_links` via is_known_type | computes: `let link_type = if crate::link_types::is_known_type(key) { key.to_string() } else { "associative".to_string() };` — the property-name-as-type rule. A frontmatter block `refutes:\n  - "[[X]]"` yields link_type=`refutes` under one vocabulary and `associative` under another. Same collapse as parse_link_body, on the frontmatter face. | reached on: index_note_impl -> extract_frontmatter_typed_links (:7306) -> :7341 and :7350 | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn emit_frontmatter_links(
    wl: &regex::Regex,
    key: &str,
    value: &str,
    out: &mut Vec<TypedLink>,
    seen: &mut std::collections::HashSet<String>,
) {
- src-tauri/src/search.rs:8018 `index_note_impl` via is_structural_type | computes: `.filter(|l| !crate::link_types::is_structural_type(&l.link_type))` — DROPS body-authored structural edges entirely (PJ-065: structural edges are frontmatter-only). This one DOES change the row COUNT: under a vocabulary where `parent` is structural, a body `[[parent::X]]` produces NO note_links row; under one where it is not, it produces one. This is the only vocabulary read in the slice that the harness's `link_rows` counter could catch. | reached on: index_note_impl, every index | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String> {
- src-tauri/src/search.rs:8485 `index_note_impl` via is_structural_type | computes: the `structural: bool` argument to link_row_is_preserved (search.rs:477-489), whose body is `(traversal_count > 0 || weight != 1.0 || status != "active" || confidence != CONFIDENCE_UNJUDGED) && !structural`. The vocabulary therefore decides whether an existing edge's EARNED Living-Link data (weight, confidence, traversal_count, archived status, created date) is carried across a re-index or discarded. Under the wrong vocabulary an earned edge can be re-inserted with default weight 1.0, confidence reset, traversal 0 — the CLAUDE.md 'living only in search.db' data, silently reset. | reached on: index_note_impl, on any save where `unchanged` (:8505-8516) is false | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String> {
- src-tauri/src/search.rs:8561 `index_note_impl` via is_structural_type | computes: which INSERT shape the edge gets. TRUE -> the structural row at :8567-8571 (confidence='structural', weight 1.0, traversal_count 0, `seq` carried, `continue` — skipping the preserved path entirely). FALSE -> the living-link rows at :8579-8583 (preserved) or the fresh-defaults branch below. Same (source_path, target_name, link_type) key either way, so row COUNT and the harness's `edges` tuple are both identical; the confidence/weight/seq columns differ. | reached on: index_note_impl, per changed/added edge | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String> {
- src-tauri/src/search.rs:9550 `structured_search` via is_structural_type | computes: `if crate::link_types::is_structural_type(&link_type_lower) { conditions.push("1 = 0"); continue; }` — a typed-link search filter naming a structural type is forced to match nothing. Read-only: writes no rows. Under the wrong vocabulary a federated query for a CHILD's type either returns the empty set when it should return results, or returns results the child considers non-cognitive. | reached on: constellation_search (:13047) -> execute_search (:13079) -> :13108 and :13187 — a read-only IPC query path | impact: WRONG_READ_ONLY_ANSWER | conn in scope: true
    sig: fn structured_search(conn: &Connection, filters: &SearchFilters, limit: u32) -> Vec<SearchResult> {
- src-tauri/src/search.rs:11040 `recompute_after_link_status_change` via snapshot() | computes: the archive/unarchive repair path. Reads the vocabulary three times: incoming_aggregate_assignments at :11040 (gated on incoming_links_backfill::is_built at :11037), stratum_sql_expr at :11054 and maturity_sql_expr at :11055 (UNgated). Recomputes note_meta.incoming_* on the resolved target paths and sky_nodes.stratum/maturity on target(s) + source. The doc comment (:11010-11022) states no note SAVE ever heals this, so a wrong value written here is permanent until a full reconcile. | reached on: constellation_link_unarchive (:11069) and the archive command — a DB-only decision with no .md write, so the watcher never re-fires | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn recompute_after_link_status_change(
    conn: &Connection,
    source_path: &str,
    target_lower: &str,
) -> Result<(), String> {
- src-tauri/src/search.rs:12548 `reindex_delete_note` via snapshot() | computes: the DELETE-side twin of the save maintenance. `del_targets` is captured unconditionally at :12418; then incoming_aggregate_assignments at :12548 (gated on `inc_on` = is_built, captured :12419) and stratum_sql_expr/maturity_sql_expr at :12560-12561 (ungated) recompute the deleted note's FORMER targets. The connection is not a parameter — it is taken from `state.db.lock()` at :12375 in Phase 1 and re-acquired for Phase 3. | reached on: every delete funnel (trash / system trash / outright), plus the watcher's vanished-.md branch | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub fn reindex_delete_note(
    state: &SearchState,
    note_path: &str,
    ctx: DeleteCtx,
) -> Result<(), String> {
- src-tauri/src/search.rs:12718 `reindex_single_note` via other (indirect — index_note :12718, maintain_incoming_after_save :12754, maintain_sky_after_save :12765) | computes: THE SAVE-PATH FUNNEL. Reads no registry itself; it is where a routed vocabulary would have to be injected to cover the call-time half. Its connection comes from `state.db.lock()` at :12688 / `db.as_ref()` at :12689 — i.e. it is hard-bound to the ACTIVE universe's SearchState, which is the second structural obstacle for 1.2: there is no parameter through which a child's connection could enter, only the process-wide state. Returns MaintenanceOutcome (search.rs:346-361) recording which of the three best-effort steps failed. | reached on: 16 production call sites — see notes | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub fn reindex_single_note(
    state: &SearchState,
    note_path: &str,
    library_name: &str,
) -> Result<MaintenanceOutcome, String> {
- src-tauri/src/search.rs:2780 `on_link_vocabulary_changed` via other (calls create_outgoing_link_triggers :2780, which reads snapshot()) | computes: the in-session vocabulary-edit reaction: recreate the outgoing triggers so their persisted bodies carry the new rank CASE + IN-list, then schedule BOTH re-materializes (links_backfill::maybe_schedule :2786, incoming_links_backfill::maybe_schedule :2792). It takes the connection from `app.state::<SearchState>()` at :2773-2779 — the ACTIVE universe's DB only. There is no per-child equivalent: a linked universe's own vocabulary edit has no path to refresh that child's triggers or aggregates from this process. | reached on: save_universe_link_types (link_types.rs:544) | impact: NO_IMPACT | conn in scope: true
    sig: pub fn on_link_vocabulary_changed(app: &tauri::AppHandle) {
- src-tauri/src/links_backfill.rs:248 `recompute_range` via snapshot() | computes: `UPDATE note_meta SET {outgoing_aggregate_assignments("note_meta.path")} WHERE path > ?1 AND path <= ?2` — the windowed outgoing re-materialize. Also interpolated into the test-only trigger family at :739-740. | reached on: process_batch (:233) and recompute_all_outgoing (:285) | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_range(conn: &Connection, after_path: &str, last_path: &str) -> rusqlite::Result<usize> {
- src-tauri/src/links_backfill.rs:307 `recompute_incoming_range` via snapshot() | computes: `UPDATE note_meta SET {incoming_aggregate_assignments("note_meta")} WHERE path > ?1 AND path <= ?2` — the windowed incoming re-materialize. Same SQL string the save-path maintenance builds, so a mismatch between the two would be undetectable by comparing them. | reached on: recompute_all_incoming (:337) <- converge_derived_views (converge.rs:286) <- after_incoming_backfill / after_repair_run / heal_interrupted_walk / after_vocabulary_change | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_incoming_range(conn: &Connection, after: &str, last: &str) -> rusqlite::Result<usize> {
- src-tauri/src/links_backfill.rs:359 `recompute_sky_range` via snapshot() | computes: `UPDATE sky_nodes SET stratum = ({stratum_sql_expr()}), maturity = ({maturity_sql_expr()}) WHERE path > ?1 AND path <= ?2` (:357-361) — the windowed sky re-materialize, the authoritative self-heal maintain_sky_after_save's doc comment defers to. | reached on: recompute_all_sky (:391) <- converge_derived_views (converge.rs:297) <- after_repair_run / heal_interrupted_walk / after_vocabulary_change | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_sky_range(conn: &Connection, after: &str, last: &str) -> rusqlite::Result<usize> {
- src-tauri/src/links_backfill.rs:99 `is_needed` via snapshot() | computes: THE OUTGOING FINGERPRINT GATE — `stored_vocab_fingerprint(conn) != crate::link_types::snapshot().fingerprint()`. Compares a fingerprint STORED IN THE DATABASE (schema_versions.links_vocab, :119) against the PROCESS-GLOBAL registry's. Opening a child universe's DB with the parent's registry installed makes this permanently true, so the backfill re-runs forever, re-materializing every row with the parent's vocabulary on each pass. | reached on: maybe_schedule / boot / on_link_vocabulary_changed (search.rs:2786) | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: fn is_needed(conn: &Connection) -> bool {
- src-tauri/src/incoming_links_backfill.rs:49 `is_stamped` via snapshot() | computes: THE INCOMING FINGERPRINT GATE — `is_built(conn) && stored_vocab_fingerprint(conn) == crate::link_types::snapshot().fingerprint()`. Same cross-universe comparison as above: the stored value belongs to the database, the compared value to the process. Against a child DB it reads false permanently, which per the doc comment (:63-65) flips every gated READER back to the live getBacklinks path — so a routed read would silently take a different code path than the same read against the active universe. | reached on: maybe_schedule (:107), and the read-flip in constellation_search_link_counts (per the doc comment at :35-36) | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: pub(crate) fn is_stamped(conn: &Connection) -> bool {
- src-tauri/src/incoming_links_backfill.rs:149 `run` via snapshot() | computes: `let run_fp = crate::link_types::snapshot().fingerprint();` captured up-front, then written into the CHILD's schema_versions row `incoming_links_vocab` at :174-177 inside the same transaction as the `incoming_links` version stamp. A routed run stamps the PARENT's fingerprint into the child's database, which then reads as 'stamped and trustworthy' to that child's own owner process on its next launch. | reached on: maybe_schedule's background thread (:113); the recompute itself goes through converge::after_incoming_backfill (:153) | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<usize, String> {
- src-tauri/src/name_fold_backfill.rs:157 `run` via snapshot() | computes: three vocabulary-derived UPDATEs per affected note: incoming_aggregate_assignments hoisted at :157 and executed at :166-170, stratum_sql_expr at :173, maturity_sql_expr at :178. Affected set = note_meta rows whose Unicode fold differs from ASCII LOWER(name) (:145-155) — no vocabulary involvement in selecting them, only in the values written. | reached on: maybe_schedule (:45) — a one-shot MIG-085 §B.0 backfill on a dedicated connection | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<(usize, usize), String> {
- src-tauri/src/sky_backfill.rs:388 `process_batch` via snapshot() | computes: the first-population sky pass: `UPDATE sky_nodes SET stratum = ({stratum_sql_expr()}) WHERE stratum IS NULL AND path > ?1 AND path <= ?2` (:385-395) and the maturity twin at :396-406. `WHERE stratum IS NULL` means these values are written ONCE and never revisited by this pass — a wrong vocabulary here is not self-healing from this module. | reached on: maybe_schedule (:54) -> run (:102), the sky_nodes first-population backfill | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn process_batch(
    db: &Mutex<Option<Connection>>,
    after_path: &str,
) -> Result<(usize, String), String> {
- src-tauri/src/converge.rs:229 `converge_derived_views` via other (indirect — recompute_all_outgoing :259, recompute_all_incoming :286, recompute_all_sky :297) | computes: the single whole-corpus assembly for the three link families. Reads no registry itself, but is the one place all three vocabulary-derived re-materializes are driven from, so it is the natural chokepoint for a routed vocabulary on the bulk side — the mirror of reindex_single_note on the per-note side. Note the incoming family is gated on incoming_links_backfill::is_built at :279 except for Families::IncomingOnly (:278). | reached on: after_repair_run (:395), heal_interrupted_walk (:423), after_mig108 (:437), after_vocabulary_change (:448), after_incoming_backfill (:463) | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub fn converge_derived_views(
    conn: &Connection,
    _key: &ConvergeKey,
    families: Families,
    ctx: &Ctx<'_>,
) -> ConvergeReport {
NOTES: EVERY CALLER OF `maintain_incoming_after_save` — exhaustive, from `grep -rn maintain_incoming_after_save --include=*.rs` over all of src-tauri:

  PRODUCTION (exactly ONE):
    src-tauri/src/search.rs:12754 — inside reindex_single_note (fn def :12682), guarded by
      `if let Some((old_t, old_n, old_a)) = inc_old` where `inc_old` is Some only when
      incoming_links_backfill::is_built(conn) at :12712.

  TEST / HARNESS:
    src-tauri/src/search.rs:2089, :2099 — tests_.. maintain_incoming_touches_only_changed_targets (fn :2062)
    src-tauri/src/search.rs:2182       — a second unit test in the same module
    src-tauri/src/federation/vocab_harness.rs:156 — index_under_vocabulary (fn :135)

  DOC-COMMENT MENTIONS ONLY (not calls): search.rs:359, :2673, :2698, :5975, :11013;
    vocab_harness.rs:7, :150.

So the routed-write surface for the incoming aggregate is ONE production call site. That is the good news. The bad news is what feeds it: `reindex_single_note` itself has 16 production callers, all of which reach the SAME `state: &SearchState` (the active universe's connection), so routing has to happen either above all 16 or inside the funnel.

THE 16 PRODUCTION CALLERS OF `reindex_single_note` (file:line -> enclosing fn -> which app path):
  search.rs:12275      -> constellation_search_reindex (:12251)   — THE SAVE PATH (1500 ms debounced save IPC)
  search.rs:12861      -> reindex_md_descendants (:12815)         — watcher: external folder rename/move/bulk add
  search.rs:13010      -> reindex_changed_paths (:12948)          — WATCHER FLUSH (external .md change, 300 ms debounce)
  libraries.rs:1450    -> create_note (:1368)                     — new note
  libraries.rs:1890    -> rename_item_db_tail (:1845)             — rename cascade
  libraries.rs:1967    -> rename_folder_db_tail (:1912)           — folder rename cascade
  libraries.rs:2662    -> resolve_structural_conflict (:2617)     — PJ-065 structural Keep/Move-here
  libraries.rs:2811    -> move_item_db_tail (:2766)               — move
  libraries.rs:7071    -> update_links_on_rename (:6793)          — wikilink cascade rewrite, per affected source note
  reconcile.rs:469     -> run (:256)                              — reconcile re-adopt (moved file)
  reconcile.rs:565     -> run (:256)                              — reconcile bulk re-index
  index_repair.rs:853  -> run_cold_start (:785)                   — PJ-207 index repair / Full re-read
  bases.rs:437         -> update_note_property (:398)             — Base cell edit
  shape.rs:214         -> apply_shape (:178)                      — shape/stage promote
  tasks.rs:540         -> toggle_task (:453)                      — checkbox toggle
  universe.rs:2488     -> reindex_written_template (:2478)        — template write

WHAT THE H1 HARNESS ACTUALLY DRIVES, AND WHAT IT ACTUALLY OBSERVES.

`index_under_vocabulary` (vocab_harness.rs:135-160) performs exactly three production calls:
  :141 link_types::set_active(vocabulary)
  :142 search::init_db(dir/search.db)      -> init_db_scoped(path, InitScope::Active), so owns == true
  :146 search::index_note(&conn, path, "harness", true)   [per note]
  :156 search::maintain_incoming_after_save(&conn, path, &empty, "", &empty)   [per note]

`aggregates_for` (:73-106) observes exactly FOUR queries:
  :81  COUNT(*) FROM note_links                                        -> link_rows
  :83-86  source_path, target_name, COALESCE(link_type,'') FROM note_links -> edges
  :91-93  path, COALESCE(incoming_count, 0) FROM note_meta             -> incoming_counts
  :98-100 path, COALESCE(incoming_link_types,'') FROM note_meta        -> incoming_types

ON THE OBSERVED PATH (a wrong vocabulary here WILL fail the acceptance test):
  * search.rs:7244  parse_link_body / is_known_type      -> note_links.link_type            -> `edges`
  * search.rs:7371  emit_frontmatter_links / is_known_type -> note_links.link_type          -> `edges`
  * search.rs:8018  is_structural_type (drop body structural) -> row presence               -> `edges` AND `link_rows`
  * search.rs:2489  incoming_aggregate_assignments, reached via maintain_incoming_after_save :2661
                    -> note_meta.incoming_count + incoming_link_types                       -> `incoming_counts` + `incoming_types`

ON A DRIVEN PATH BUT NOT OBSERVED (a wrong vocabulary here passes the test silently):
  * search.rs:7079  structural_frontmatter_targets -> note_meta.outgoing_links_json  — aggregates_for reads no outgoing column
  * search.rs:8485  is_structural_type -> link_row_is_preserved -> weight/confidence/traversal_count/created — the edges query
                    selects only (source_path, target_name, link_type), so every earned Living-Link column is invisible to it
  * search.rs:8561  is_structural_type -> which INSERT shape (confidence='structural', weight 1.0, seq) — same blind spot
  * search.rs:2243  outgoing_aggregate_assignments, baked into note_links_outgoing_ai/ad/au at :2327-2328 by
                    create_outgoing_link_triggers (called from init_db at :5970). These triggers DO fire during the harness's
                    index_note and DO write note_meta.outgoing_count / outgoing_link_types / _json / _top_rank — none observed.
  * search.rs:5657/:5658  stratum_sql_expr/maturity_sql_expr baked into note_meta_sky_ai, which fires on the note_meta INSERT
                    during index_note and writes sky_nodes.stratum + sky_nodes.maturity — aggregates_for never reads sky_nodes.
  * search.rs:5540  sx_new baked into note_links_sky_ai/_au, deciding sky_links membership — not read either.
  * incoming_links_backfill.rs:49 is_stamped / :174 the vocab stamp — the harness never runs the backfill, so schema_versions
                    is never compared or written.

NOT DRIVEN AT ALL BY THE HARNESS (zero coverage today):
  * maintain_sky_after_save (search.rs:2706) — THE SKY WRITE-TIME MAINTENANCE the prompt asked about. The harness calls
    index_note + maintain_incoming_after_save; it never calls maintain_sky_after_save, and its aggregate struct has no
    sky field. The doc-comment at vocab_harness.rs:7 lists "the sky write-time maintenance" among the 26 call sites, but
    the harness neither drives nor observes it.
  * reindex_single_note (12682) — bypassed. The harness calls index_note and maintain_incoming_after_save DIRECTLY, which
    means it also bypasses the `incoming_links_backfill::is_built` gate at :12712. On a genuinely fresh database that gate
    is FALSE, so production would SKIP the incoming maintenance entirely — the harness gets values production would not
    have written. That is a real divergence between the acceptance test and the save path it stands for.
  * reindex_delete_note (12368), recompute_after_link_status_change (11023) — no delete, no archive/unarchive.
  * links_backfill (all six recompute fns), incoming_links_backfill::run, name_fold_backfill::run, sky_backfill::process_batch,
    converge::converge_derived_views — none reached.
  * structured_search (9347) — read side, not reached.

WHAT THIS MEANS FOR 1.2's DEFINITION OF DONE. Removing the `#[ignore]` at vocab_harness.rs:275 proves the routed
vocabulary reaches the parse chain (7244/7371/8018) and the incoming aggregate (2489). It proves NOTHING about the sky
values, the outgoing aggregate, the earned-link preservation decision, or any persisted trigger body — all of which are
equally vocabulary-derived and all of which a routed write touches. If the acceptance condition is to mean "the routed
write used the owner's vocabulary", `Aggregates` needs sky_nodes.stratum/maturity and note_meta.outgoing_* added, and
`index_under_vocabulary` needs to route through maintain_sky_after_save.

TWO STRUCTURAL OBSTACLES THAT THREADING A PARAMETER DOES NOT SOLVE:

1. THE GENERATORS TAKE NO CONNECTION AND RETURN A STRING. stratum_sql_expr(), maturity_sql_expr(),
   outgoing_aggregate_assignments(&str), incoming_aggregate_assignments(&str) are all `-> String` with no `conn`
   parameter, and the three predicates (is_known_type / is_structural_type / structural_frontmatter_targets) take only
   a `&str`. Binding the vocabulary "to the connection" therefore requires either a new parameter on all seven (and on
   every caller — 20+ sites across search.rs, links_backfill.rs, name_fold_backfill.rs, sky_backfill.rs), or a
   connection-keyed lookup the generators can consult. Neither exists today.

2. HALF THE VOCABULARY IS ALREADY FROZEN IN THE CHILD'S sqlite_master. create_outgoing_link_triggers (2286) and the
   four init_db trigger blocks (5540, 5657/5658, 5910, 5956) persist the generated SQL. A routed write that carries
   the right vocabulary through the call still fires triggers whose bodies carry whatever vocabulary last ran init_db
   on that file. For the outgoing aggregate this is the dominant path — outgoing_aggregate_assignments is NOT called
   at save time at all; it is only in the trigger bodies and in links_backfill::recompute_range.

AND THE ONE THAT IS ALREADY BROKEN, INDEPENDENT OF 1.2: search.rs:5540. The sky_links trigger family is created
outside every `if owns` guard, so `init_db_schema_only` (search.rs:4597 <- federation/migrate.rs:169) writes the
ACTIVE universe's structural-exclusion clause into a linked universe's `note_links_sky_ai` and `note_links_sky_au`
trigger bodies today. Its four siblings at :5657, :5910, :5956, :5970 are all correctly gated with the PJ-232 comment
at :5636-5639 naming exactly this hazard; this one was missed.
UNVERIFIED: The claim at search.rs:12266 that constellation_search_reindex has '21 frontend callers' — I did not grep the Svelte side; that number is a source comment, not something I verified. | Which of the 16 reindex_single_note call sites a Router would actually route. I read each enclosing fn NAME and the surrounding line, but I did not read the full bodies of bases.rs::update_note_property, shape.rs::apply_shape, tasks.rs::toggle_task, universe.rs::reindex_written_template, index_repair.rs::run_cold_start, or reconcile.rs::run — so my 'which app path' labels for those six come from the fn name plus the immediate call line, not from tracing their own callers. To settle: read each fn body and its callers. | Whether `sky_backfill::process_batch`'s `db: &Mutex<Option<Connection>>` should count as 'a &Connection in scope'. I marked it true because it obtains `guard.as_mut()` at :224 and :380, but it is a Mutex over an Option, not a borrowed connection — a routing design has to decide whether that is the same thing. | Whether the harness's `deltas(&["refutes"])` vs `deltas(&[])` difference actually reaches search.rs:8018 / :8485 / :8561. LinkTypeDef is constructed with `structural: false` at vocab_harness.rs:121, so no custom type in the harness is structural — meaning today the harness exercises the is_known_type sites but probably not the is_structural_type sites. I did not run the test to confirm. To settle: run `cargo test federation::vocab_harness` and inspect, or add a `structural: true` delta. | The exact set of readers gated on incoming_links_backfill::is_stamped. The doc comment at incoming_links_backfill.rs:35-36 names `constellation_search_link_counts` and 'the save-path maintenance gate' and 'the reconcile recompute gate'; I verified only the save-path one (search.rs:12712) and converge.rs:279 by reading them. I did not open constellation_search_link_counts. | Whether `federation::migrate::run_migrations_on` is the ONLY production path that reaches init_db_schema_only. I grepped for the symbol and found federation/migrate.rs:169 plus four test call sites (:618, :655, :733, :752), but I did not read run_migrations_on's own callers to confirm when it fires.

### SLICE: MAP SLICE 4 — THE BACKFILLS AND THEIR FINGERPRINT GATES (links_backfill.rs, incoming_links_backfill.rs, sky_backfill.rs). All three files read in full this session; every transitive vocabulary read followed into search.rs/link_types.rs and read in the function body, not the grep hit.
- src-tauri/src/links_backfill.rs:57 `maybe_schedule` via other (calls is_needed at :69, which reads snapshot() at :99) | computes: the spawn decision for the outgoing back-fill. Takes state.db (SearchState) — the ACTIVE universe's connection, published at search.rs:11649 — reads is_needed(conn), spawns a thread at :76 that calls run(&app_bg) | reached on: boot: ensure_search_db_ready (search.rs:11476) at search.rs:11672; vocabulary edit: on_link_vocabulary_changed (search.rs:2771) at search.rs:2786 | impact: NO_IMPACT | conn in scope: true
    sig: pub fn maybe_schedule(app: tauri::AppHandle)
- src-tauri/src/links_backfill.rs:99 `is_needed` via snapshot() | computes: THE OUTGOING FINGERPRINT GATE. `stored_vocab_fingerprint(conn) != crate::link_types::snapshot().fingerprint()`. Left side reads schema_versions WHERE module='links_vocab' (:120-125, default 0 when absent). Right side is the PROCESS-GLOBAL registry's FNV-1a over ordered type ids (link_types.rs:300-311) | reached on: boot schedule (search.rs:11672) and vocabulary edit (search.rs:2786), both via maybe_schedule:69 | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn is_needed(conn: &Connection) -> bool
- src-tauri/src/links_backfill.rs:106 `version_current` via other (no vocabulary read — schema_versions WHERE module='links_outgoing' only, compared to LINKS_OUTGOING_SCHEMA_VERSION = 1, search.rs:1645) | computes: whether a completed pass exists. Used twice: as the first clause of is_needed (:88) and as the CURSOR-RESET decision in run (:170) | reached on: same as is_needed; plus run:170 | impact: NO_IMPACT | conn in scope: true
    sig: fn version_current(conn: &Connection) -> bool
- src-tauri/src/links_backfill.rs:119 `stored_vocab_fingerprint` via other (reads the DB side of the gate: SELECT version FROM schema_versions WHERE module = 'links_vocab', .unwrap_or(0)) | computes: the fingerprint the aggregates in THIS database were last materialized under. This is the value a child universe stamped with its OWN vocabulary | reached on: is_needed:99 | impact: NO_IMPACT | conn in scope: true
    sig: fn stored_vocab_fingerprint(conn: &Connection) -> i64
- src-tauri/src/links_backfill.rs:160 `run` via snapshot() | computes: `run_fp` — the fingerprint captured up-front and later STAMPED into the database at finalize(&state.db, run_fp) (:184). On a routed open this is the PARENT's fingerprint being prepared for the CHILD's schema_versions | reached on: the thread spawned by maybe_schedule:76 | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<u64, String>
- src-tauri/src/links_backfill.rs:171 `run` via other (no vocabulary read — branch selected by version_current at :170, which is true whenever the run was triggered purely by the :99 fingerprint mismatch) | computes: DESTRUCTIVE: `DELETE FROM links_outgoing_backfill_cursor`. On a routed open against a child whose own back-fill completed, version_current is TRUE, so this fires and discards the child's resume cursor before re-materializing every row | reached on: run, immediately after the ANALYZE block | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<u64, String>
- src-tauri/src/links_backfill.rs:152 `run` via other (no vocabulary read) | computes: `conn.execute_batch("ANALYZE")` — rewrites sqlite_stat1 in the target database. Not vocabulary, but it is a WRITE the routed path would perform against the child before any recompute | reached on: run, before the batch loop | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<u64, String>
- src-tauri/src/links_backfill.rs:248 `recompute_range` via other (crate::search::outgoing_aggregate_assignments("note_meta.path") → search.rs:2243 `let reg = crate::link_types::snapshot();`) | computes: the SQL IN-list (sql_in_list_cognitive, link_types.rs:241), the rank CASE (sql_rank_case_cognitive, link_types.rs:253), the sentinel (cognitive_sentinel_rank, link_types.rs:292) and the structural NOT-IN fragment (structural_not_in_clause, link_types.rs:268) for `UPDATE note_meta SET outgoing_count/outgoing_link_types/outgoing_link_types_json/outgoing_top_rank WHERE path > ?1 AND path <= ?2` | reached on: process_batch:233 (every back-fill batch) and recompute_all_outgoing:285 | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_range(conn: &Connection, after_path: &str, last_path: &str) -> rusqlite::Result<usize>
- src-tauri/src/links_backfill.rs:264 `recompute_all_outgoing` via other (recompute_range:248 → outgoing_aggregate_assignments → search.rs:2243 snapshot()) | computes: walks EVERY note_meta row in 500-path windows and re-materializes all four outgoing columns. Takes a &Connection directly — it is already routable by argument today | reached on: converge::converge_derived_views:259 — i.e. after_repair_run (converge.rs:395, called from search.rs:12111), heal_interrupted_walk (converge.rs:423, called from derived_heal.rs:215), after_mig108 (converge.rs:437, called from mig108.rs:1207), after_incoming_backfill (converge.rs:463) is IncomingOnly so does not reach it | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_all_outgoing(conn: &Connection, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize>
- src-tauri/src/links_backfill.rs:306 `recompute_incoming_range` via other (crate::search::incoming_aggregate_assignments("note_meta") → search.rs:2489 `let reg = crate::link_types::snapshot();`) | computes: the cognitive IN-list, rank CASE, sentinel and the structural NOT-IN on `nl.link_type` for `UPDATE note_meta SET incoming_count/incoming_link_types/incoming_link_types_json/incoming_top_rank` | reached on: recompute_all_incoming:337 | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_incoming_range(conn: &Connection, after: &str, last: &str) -> rusqlite::Result<usize>
- src-tauri/src/links_backfill.rs:317 `recompute_all_incoming` via other (recompute_incoming_range:306 → incoming_aggregate_assignments → search.rs:2489 snapshot()) | computes: walks EVERY note_meta row and re-materializes all four incoming columns. Routable by argument today | reached on: converge::converge_derived_views:285 (All / LinksOnly / IncomingOnly) — reached from incoming_links_backfill.rs:154 via after_incoming_backfill, and from derived_heal / repair / mig108 paths | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_all_incoming(conn: &Connection, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize>
- src-tauri/src/links_backfill.rs:359 `recompute_sky_range` via other (crate::search::stratum_sql_expr() → search.rs:189 snapshot(); crate::search::maturity_sql_expr() → search.rs:267 snapshot()) | computes: the structural exclusion substituted into the /*SX*/ markers of the stratum and maturity expressions, for `UPDATE sky_nodes SET stratum = (...), maturity = (...) WHERE path > ?1 AND path <= ?2` | reached on: recompute_all_sky:391 | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_sky_range(conn: &Connection, after: &str, last: &str) -> rusqlite::Result<usize>
- src-tauri/src/links_backfill.rs:371 `recompute_all_sky` via other (recompute_sky_range:359 → stratum_sql_expr search.rs:189 / maturity_sql_expr search.rs:267, both snapshot()) | computes: walks EVERY sky_nodes row and rewrites stratum + maturity. Unconditional and ungated (converge.rs:296 comment: 'Ungated and idempotent'). Routable by argument today | reached on: converge::converge_derived_views:297 (All / LinksOnly) — repair tail, derived_heal, vocabulary-change path | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn recompute_all_sky(conn: &Connection, _key: &crate::converge::ConvergeKey) -> rusqlite::Result<usize>
- src-tauri/src/links_backfill.rs:454 `finalize` via other (no vocabulary read at this statement — it writes LINKS_OUTGOING_SCHEMA_VERSION) | computes: STAMP WRITE #1: `INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('links_outgoing', ?1, strftime('%s','now'))`, params LINKS_OUTGOING_SCHEMA_VERSION | reached on: run:184 when process_batch drains | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn finalize(db: &Mutex<Option<Connection>>, vocab_fingerprint: i64) -> Result<(), String>
- src-tauri/src/links_backfill.rs:464 `finalize` via other (writes the value captured by snapshot() at :160) | computes: STAMP WRITE #2 — THE FINGERPRINT STAMP R4 FORBIDS ON A ROUTED OPEN. `INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('links_vocab', ?1, strftime('%s','now'))` (SQL literal on :466), params![vocab_fingerprint] = run_fp from :160. Same transaction as :454 and the cursor DELETE at :470 | reached on: run:184 | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn finalize(db: &Mutex<Option<Connection>>, vocab_fingerprint: i64) -> Result<(), String>
- src-tauri/src/incoming_links_backfill.rs:49 `is_stamped` via snapshot() | computes: THE INCOMING FINGERPRINT GATE (READ side). `is_built(conn) && stored_vocab_fingerprint(conn) == crate::link_types::snapshot().fingerprint()`. This is the predicate every gated READER sits on | reached on: maybe_schedule:107 (boot search.rs:11699, vocabulary edit search.rs:2792); constellation_search_link_counts (search.rs:13779) at search.rs:13795 | impact: WRONG_READ_ONLY_ANSWER | conn in scope: true
    sig: pub(crate) fn is_stamped(conn: &Connection) -> bool
- src-tauri/src/incoming_links_backfill.rs:73 `is_built` via other (DELIBERATELY reads NO vocabulary — `SELECT version FROM schema_versions WHERE module = 'incoming_links'` >= SCHEMA_VERSION (=1, :32) only) | computes: THE WRITE-SIDE GATE. Documented at :53-72 as the deliberate split: readers ask is_stamped (version AND fingerprint), WRITERS ask is_built (version only) so save-path maintenance keeps running across a vocabulary re-materialize. CONSEQUENCE FOR ROUTING: because it excludes the fingerprint by design, it returns TRUE for a child database no matter whose vocabulary the writing process holds — it cannot refuse a foreign-vocabulary write | reached on: converge.rs:279 (the IncomingOnly gate bypass); search.rs:11037 recompute_after_link_status_change (search.rs:11023); search.rs:12419 reindex_delete_note (search.rs:12368); search.rs:12712 reindex_single_note (search.rs:12682) | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn is_built(conn: &Connection) -> bool
- src-tauri/src/incoming_links_backfill.rs:86 `stored_vocab_fingerprint` via other (reads the DB side: SELECT version FROM schema_versions WHERE module = 'incoming_links_vocab', .unwrap_or(0)) | computes: the fingerprint the incoming aggregates in THIS database were last materialized under | reached on: is_stamped:49 | impact: NO_IMPACT | conn in scope: true
    sig: fn stored_vocab_fingerprint(conn: &Connection) -> i64
- src-tauri/src/incoming_links_backfill.rs:97 `maybe_schedule` via other (!is_stamped at :107 → snapshot() at :49) | computes: the spawn decision. Pre-checks on state.db (ACTIVE universe's connection), then spawns run(&app_bg) at :113 | reached on: boot: search.rs:11699 in ensure_search_db_ready; vocabulary edit: search.rs:2792 in on_link_vocabulary_changed | impact: NO_IMPACT | conn in scope: true
    sig: pub fn maybe_schedule(app: tauri::AppHandle)
- src-tauri/src/incoming_links_backfill.rs:124 `run` via other (no vocabulary read at this statement — it fixes the DATABASE PATH) | computes: `Connection::open(&crate::search::db_path(app))` on its OWN dedicated connection. db_path (search.rs:1465-1468) = crate::universe::active_constellation_dir(app)?.join("search.db"), and active_constellation_dir (universe.rs:69) resolves the ACTIVE universe root. THIS IS THE ONLY THING KEEPING THIS BACK-FILL OFF A CHILD DATABASE TODAY — it is not a guard, it is an incidental path binding | reached on: the thread spawned at :113 | impact: NO_IMPACT | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<usize, String>
- src-tauri/src/incoming_links_backfill.rs:142 `run` via other (no vocabulary read) | computes: `CREATE INDEX IF NOT EXISTS idx_nl_tnl ON note_links(target_name_lower, status)` — DDL written into the opened database. Vocabulary-independent, but a routed open would create it in the child | reached on: run, before the recompute | impact: NO_IMPACT | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<usize, String>
- src-tauri/src/incoming_links_backfill.rs:149 `run` via snapshot() | computes: `run_fp` — the fingerprint captured up-front and stamped at :173. On a routed open this is the PARENT's fingerprint destined for the CHILD's schema_versions | reached on: run, before after_incoming_backfill | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<usize, String>
- src-tauri/src/incoming_links_backfill.rs:154 `run` via other (converge::after_incoming_backfill converge.rs:463 → converge_derived_views:285 → links_backfill::recompute_all_incoming:317 → incoming_aggregate_assignments search.rs:2489 snapshot()) | computes: the actual whole-database rewrite of every note's four incoming columns. Deliberately UNGATED (converge.rs:278 `let gate_on_stamp = !matches!(families, Families::IncomingOnly);`) because this caller is the builder, not a healer — so nothing in the stamp machinery can stop it | reached on: run, on every scheduled pass | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<usize, String>
- src-tauri/src/incoming_links_backfill.rs:167 `run` via other (writes SCHEMA_VERSION, :32) | computes: STAMP WRITE #3: `INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('incoming_links', ?1, strftime('%s','now'))` (SQL literal :169) | reached on: run, after the recompute succeeds | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<usize, String>
- src-tauri/src/incoming_links_backfill.rs:173 `run` via other (writes the value captured by snapshot() at :149) | computes: STAMP WRITE #4 — THE SECOND FINGERPRINT STAMP R4 FORBIDS. `INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('incoming_links_vocab', ?1, strftime('%s','now'))` (SQL literal :175), params![run_fp]. Same transaction as :167 | reached on: run, after the recompute succeeds | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<usize, String>
- src-tauri/src/sky_backfill.rs:54 `maybe_schedule` via other (is_needed at :66 — which reads NO vocabulary) | computes: the spawn decision for the sky back-fill. Pre-checks on state.db (ACTIVE universe), spawns run(&app_bg) at :74 | reached on: boot only: search.rs:11664 in ensure_search_db_ready. NOT scheduled by on_link_vocabulary_changed (search.rs:2771 schedules links at :2786 and incoming at :2792 — sky is absent) | impact: NO_IMPACT | conn in scope: true
    sig: pub fn maybe_schedule(app: tauri::AppHandle)
- src-tauri/src/sky_backfill.rs:89 `is_needed` via other (NO vocabulary read at all — `SELECT version FROM schema_versions WHERE module = 'sky'` < SKY_SCHEMA_VERSION (=10, search.rs:73)) | computes: THE FINDING: sky has NO vocabulary fingerprint gate. There is no 'sky_vocab' module row anywhere in src-tauri/src (grepped). Yet the values this back-fill writes ARE vocabulary-derived — the structural exclusion at :283, :388, :399. So once schema_versions.sky is at target, a vocabulary change — or a foreign-vocabulary write — can never re-trigger it | reached on: maybe_schedule:66 | impact: SKIPS_MAINTENANCE | conn in scope: true
    sig: fn is_needed(conn: &Connection) -> bool
- src-tauri/src/sky_backfill.rs:140 `run` via other (no vocabulary read) | computes: DESTRUCTIVE: `UPDATE sky_nodes SET stratum = NULL, maturity = NULL WHERE path > ?1` (SQL literal :141), scoped by the cursor. On a routed open against a child that has never stamped 'sky' (last_path = ""), this NULLs every sky_nodes row in the child before recomputing them with the parent's vocabulary | reached on: run, before the batch loop | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn run(app: &tauri::AppHandle) -> Result<u64, String>
- src-tauri/src/sky_backfill.rs:283 `process_batch` via snapshot() | computes: `crate::link_types::snapshot().structural_not_in_clause("link_type")` — the structural NOT-IN fragment appended to `INSERT OR IGNORE INTO sky_links (source_path, target_name, link_type, weight) SELECT ... FROM note_links WHERE status = 'active'{sx} AND source_path > ?1 AND source_path <= ?2` (:284-295). Decides WHICH EDGES EXIST in the child's sky_links | reached on: Phase A of every back-fill batch | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn process_batch(db: &Mutex<Option<Connection>>, after_path: &str) -> Result<(usize, String), String>
- src-tauri/src/sky_backfill.rs:388 `process_batch` via other (crate::search::stratum_sql_expr() → search.rs:189 `let sx = crate::link_types::snapshot().structural_not_in_clause("link_type");`) | computes: Phase D — the stratum expression substituted into `UPDATE sky_nodes SET stratum = (...) WHERE stratum IS NULL AND path > ?1 AND path <= ?2`. Vocabulary enters through the six /*SX*/ substitutions in the expression | reached on: Phase D of every back-fill batch | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn process_batch(db: &Mutex<Option<Connection>>, after_path: &str) -> Result<(usize, String), String>
- src-tauri/src/sky_backfill.rs:399 `process_batch` via other (crate::search::maturity_sql_expr() → search.rs:267 `let sx = crate::link_types::snapshot().structural_not_in_clause("link_type");`) | computes: Phase D — the maturity expression substituted into `UPDATE sky_nodes SET maturity = (...) WHERE maturity IS NULL AND path > ?1 AND path <= ?2` | reached on: Phase D of every back-fill batch | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn process_batch(db: &Mutex<Option<Connection>>, after_path: &str) -> Result<(usize, String), String>
- src-tauri/src/sky_backfill.rs:305 `process_batch` via other (no vocabulary read — crate::search::body_after_frontmatter search.rs:6419 and crate::search::extract_aliases search.rs:7124, both read in this session, neither touches link_types) | computes: Phase B reads each note's .md file from disk (read_note_signals, :435) and Phase C/E then UPDATE note_meta.word_count/created_at (:322-334) and INSERT INTO note_aliases (:353-362). Not vocabulary — but it is filesystem reading and index writing of a foreign universe's content from the parent's process | reached on: Phases B/C/E of every back-fill batch | impact: NO_IMPACT | conn in scope: true
    sig: fn process_batch(db: &Mutex<Option<Connection>>, after_path: &str) -> Result<(usize, String), String>
- src-tauri/src/sky_backfill.rs:463 `finalize` via other (no vocabulary read — writes SKY_SCHEMA_VERSION, search.rs:73) | computes: STAMP WRITE #5: `INSERT OR REPLACE INTO schema_versions (module, version) VALUES ('sky', ?1)` (SQL literal :464) plus `DELETE FROM sky_backfill_cursor` (:468), one transaction. NO fingerprint is stamped — and that is precisely why this stamp is the dangerous one: it says 'sky is current' with no record of WHOSE vocabulary produced it | reached on: run:154 when process_batch drains | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn finalize(db: &Mutex<Option<Connection>>) -> Result<(), String>
NOTES: ## 1. What `stored_vocab_fingerprint` actually reads, per family

There are TWO fingerprint stamps, both stored as rows in the ordinary `schema_versions` table (`module` TEXT PK, `version` INTEGER — the fingerprint is squatting in the `version` column as an opaque i64; links_backfill.rs:460-463 says so explicitly). There is NO third.

| family | stamp row read | reader | writer |
|---|---|---|---|
| outgoing | `schema_versions WHERE module='links_vocab'` | links_backfill.rs:119-126 | links_backfill.rs:464 (literal on :466) |
| incoming | `schema_versions WHERE module='incoming_links_vocab'` | incoming_links_backfill.rs:86-93 | incoming_links_backfill.rs:173 (literal on :175) |
| **sky** | **NONE — there is no sky vocabulary stamp** | — | — |

Both readers `.unwrap_or(0)`, so an absent row reads as fingerprint 0, which never equals a real fingerprint (`fingerprint()` is FNV-1a >> 1, link_types.rs:300-311; the seed registry's is asserted non-zero at links_backfill.rs:586).

## 2. EVERY place a fingerprint stamp is WRITTEN (R4's enumeration)

**Production — exactly two:**
- `src-tauri/src/links_backfill.rs:464` — `('links_vocab', run_fp)`, inside `finalize`, in the same transaction as the `links_outgoing` version stamp (:454) and the cursor clear (:470).
- `src-tauri/src/incoming_links_backfill.rs:173` — `('incoming_links_vocab', run_fp)`, inside `run`, in the same transaction as the `incoming_links` version stamp (:167).

**Non-fingerprint version stamps in the same three files (they gate the same machinery, so R4 has to rule on them too):**
- `links_backfill.rs:454` — `('links_outgoing', LINKS_OUTGOING_SCHEMA_VERSION)`
- `incoming_links_backfill.rs:167` — `('incoming_links', SCHEMA_VERSION)`
- `sky_backfill.rs:463` — `('sky', SKY_SCHEMA_VERSION)`

**Test-only writes (not production paths):** links_backfill.rs:588, :596; incoming_links_backfill.rs:271, :276, :286, :326, :334.

## 3. THE BRANCH WALK — parent's process, child's stamp, parent's snapshot()

### 3a. OUTGOING (`links_backfill`) — spurious full re-materialize, then a mislabelled stamp
`is_needed(child_conn)` (:87):
1. `version_current(child_conn)` (:106) reads the CHILD's `links_outgoing` row = 1 = target → **true**, so the first clause does NOT return early.
2. Falls to :99: child's `links_vocab` (child fingerprint) `!=` parent's `snapshot().fingerprint()` → **is_needed = TRUE**.

So the gate does **not** skip. It selects a *full re-materialize*, and then:
- `run` :160 captures `run_fp` = **the PARENT's** fingerprint.
- `run` :170 — `version_current` is true → :171 **`DELETE FROM links_outgoing_backfill_cursor`** — the child's resume progress is discarded (the code's own comment says this is for "a vocabulary change", which is exactly what a foreign vocabulary looks like from the inside).
- the loop rewrites **every** child `note_meta` row's `outgoing_count / outgoing_link_types / outgoing_link_types_json / outgoing_top_rank` through `recompute_range` :248 → `outgoing_aggregate_assignments` (search.rs:2242-2270), whose IN-list / rank CASE / sentinel / structural NOT-IN all come from the **parent's** `snapshot()` at search.rs:2243.
- `finalize` :447 stamps `links_vocab = PARENT fingerprint` into the CHILD.

**Answer to "skipped, re-run, or wrong stamp persisted": ALL THREE OF THE LAST TWO — re-run with the wrong vocabulary, and a wrong stamp persisted.** Nothing is skipped.

Nuance, stated because it changes severity and I verified it: for outgoing/incoming the wrong stamp is **self-correcting on the child's own next boot** — the child's `is_needed` will compare ITS fingerprint against the stored parent fingerprint, mismatch, and re-materialize. The stamp does not make the corruption permanent; it makes the child pay a full re-materialize and, in the window before that boot, serve parent-flavoured values.

### 3b. The genuinely *hidden* outgoing case (the inverse, and the worse one)
If a routed write corrupts child rows **without** running the back-fill — i.e. through the per-note trigger/save path, which is the ordinary Router case — then the child's `links_vocab` stamp is **still the child's own fingerprint**. On the child's next boot `is_needed` evaluates: version current AND fingerprint matches → **FALSE**. The back-fill never runs, and the parent-flavoured rows are never healed. This is the shape where "every row COUNT is still correct and nothing surfaces it" is literally true, and it is the case R4's "never write fingerprint stamps" does *not* by itself cover — the gate needs to know the write happened at all.

### 3c. INCOMING (`incoming_links_backfill`) — the read/write split leaves the write side wide open
- `is_stamped(child_conn)` (:49) = `is_built && child_fp == parent_fp` → **false**. Every gated **reader** falls back to the live `getBacklinks` path — conservative, so a routed read is degraded, not wrong.
- `is_built(child_conn)` (:73) reads **version only, by design** (the documented 2026-08-01 split at :53-72). It returns **TRUE** for any child whose own back-fill ever completed. So the **write** gates at search.rs:11037 (`recompute_after_link_status_change`), search.rs:12419 (`reindex_delete_note`), search.rs:12712 (`reindex_single_note`) all say "maintain", and the maintenance runs `incoming_aggregate_assignments` built from the **parent's** snapshot (search.rs:2489). **`is_built` cannot refuse a foreign-vocabulary write — it does not look at the vocabulary at all.** This is the single most load-bearing finding in this slice for Phase 1.2.
- If the parent additionally ran `incoming_links_backfill::run` against a child path, `converge::after_incoming_backfill` is **explicitly ungated** (converge.rs:278, "A builder is not a healer") — nothing would stop a whole-child rewrite — and :173 would stamp `incoming_links_vocab = PARENT fingerprint`.

### 3d. SKY (`sky_backfill`) — no fingerprint gate, and the stamp makes corruption PERMANENT
`is_needed` (:89-98) reads only `schema_versions.sky`. Two routed cases:
- **Child already stamped `sky` at 10** → is_needed FALSE → the whole back-fill is **skipped**. No corruption from *this* module — but equally, no self-heal is ever possible from it, because a vocabulary change cannot re-arm it (and `on_link_vocabulary_changed`, search.rs:2771, does not schedule sky at all — only links :2786 and incoming :2792).
- **Child not stamped, or stamped below 10** → the parent's process runs the full back-fill on the child: NULLs every `sky_nodes.stratum/maturity` (:140-144), inserts `sky_links` filtered by the **parent's** structural exclusion (:283), computes stratum (:388) and maturity (:399) with the **parent's** exclusion, then stamps `('sky', 10)` (:463). Because there is **no sky vocabulary fingerprint anywhere**, the child's own next boot reads `is_needed = false` and **never re-runs**. The parent-flavoured Sky values are permanent.

`converge::recompute_all_sky` (links_backfill.rs:371) is the only other path that would fix them, and it is described in converge.rs:296 as "Ungated and idempotent" — reachable only from a repair tail (search.rs:12111), `derived_heal` (derived_heal.rs:215) or mig108 (mig108.rs:1207), none of which is triggered by a vocabulary difference.

## 4. Scheduling — thread, trigger, and WHICH database

All three are `std::thread::spawn`'d background jobs, never blocking boot, all scheduled from **`ensure_search_db_ready`** (search.rs:11476), after `state.db` / `read_db` / `db_ready` are published at search.rs:11649-11659, and after `crate::link_types::load_active(app)` at search.rs:11606 (which is what installs the ACTIVE universe's vocabulary into the process-global immediately before `init_db`).

| module | scheduled at | thread | database it operates on |
|---|---|---|---|
| `sky_backfill` | search.rs:11664 (boot only) | sky_backfill.rs:74 | **`state.db`** — the ACTIVE universe's live connection (re-locked per batch/phase) |
| `links_backfill` | search.rs:11672 (boot) **and search.rs:2786** (`on_link_vocabulary_changed`) | links_backfill.rs:76 | **`state.db`** — ACTIVE universe's live connection |
| `incoming_links_backfill` | search.rs:11699 (boot) **and search.rs:2792** (`on_link_vocabulary_changed`) | incoming_links_backfill.rs:113 | **its OWN `Connection::open`** at `crate::search::db_path(app)` (incoming_links_backfill.rs:123-124) |

`db_path` (search.rs:1465-1468) = `crate::universe::active_constellation_dir(app)?.join("search.db")`; `active_constellation_dir` (universe.rs:69-72) resolves the **ambient active universe root**. So today none of the three can reach a child universe's database — but the binding is incidental (a path helper), not a guard. Making `db_path` routable, or handing any of these a routed `&Connection`, converts every site above into a live corruption path in one line.

`on_link_vocabulary_changed` is only called from `link_types.rs:563`, inside `save_universe_link_types`, and only when `snapshot().fingerprint() != before_fp` (link_types.rs:562) — a colour/label-only edit is correctly free.

## 5. Already-routable-by-argument surfaces (they take `&Connection`, no path binding at all)

These are the ones Phase 1.2 can reach *without changing any scheduling code*, and every one of them reads the parent's global vocabulary:
`links_backfill::recompute_range` :245, `recompute_all_outgoing` :264, `recompute_incoming_range` :304, `recompute_all_incoming` :317, `recompute_sky_range` :356, `recompute_all_sky` :371, `incoming_links_backfill::is_stamped` :48, `is_built` :73, and `links_backfill::is_needed` :87.

## 6. Corroborating precedent already in the tree

`federation/migrate.rs:120-167` and `search.rs:2363-2370` both document this exact failure having ALREADY happened once: `init_db` run against a linked universe's database rebuilt the child's aggregates with the parent's process-global vocabulary and then cleared the child's crash markers, making the wrong values permanent. The fix was `init_db_schema_only` (migrate.rs:169) — schema migration and nothing else. migrate.rs:134-136 explicitly forbids the rejected design: *"do not 'fix' it by loading the child's vocabulary into the global first — that means swapping a process-global on a background thread while every other subsystem reads it."* That is the same ruling as LL-047, written down at a different site.

## 7. Two dead/asymmetric things worth a ruling

- `converge::after_vocabulary_change` (converge.rs:448) — the named entry point for exactly this concern (Families::LinksOnly, which includes sky) — has **zero production callers**. Grepped across `src-tauri/src`: only its own definition. `on_link_vocabulary_changed` instead schedules the two back-fills directly and never converges sky.
- Sky is the only one of the three link-derived families with no vocabulary fingerprint, no vocabulary-change schedule, and a stamp that permanently suppresses re-runs.
UNVERIFIED: Whether MIG-111 Phase 1.2's Router is intended to hand these functions a routed `&Connection` directly, or to route at the `db_path`/`SearchState` level. I read the harness (federation/vocab_harness.rs) and migrate.rs but found no Router implementation in the tree — `routed_write_must_match_the_owners_vocabulary` (vocab_harness.rs:276) still panics with 'Phase 1.2 not implemented'. To settle it I would read the MIG-111 Phase 1.2 plan doc under docs/migrations/ and grep for a `resolve_owner` consumer (Phase 1.1, commit 660cfda8). | The exact wall-clock/row cost of a spurious full re-materialize on a large child universe. links_backfill is pure SQL and batched at 500 (:47), incoming is batched at 500 (links_backfill.rs:322-330), sky at 1000 (:44) with per-note file reads — but I did not run the benchmarks. `links_backfill::bench_reindex_trigger_overhead` (:638, #[ignore]) and `converge::tests::converge_boot_heal_cost` (converge.rs:532, #[ignore]) would give the numbers; converge.rs:10 cites 3,143 ms for all five families on a 2,721-note universe, measured by someone else, not by me this session. | Whether `search.rs::init_db` itself writes any of these five stamps (I read `finalize` in all three back-fills and `on_link_vocabulary_changed`, but I did not read `init_db`'s ~thousands of lines end to end). search.rs:2287-2290 shows init_db drop+recreates the outgoing triggers from `snapshot()` on every boot, and federation/migrate.rs:146-152 confirms that as a foreign-DDL hazard — but that is trigger DDL, which is MAP SLICE 4's neighbour, not a fingerprint stamp. To settle: read `init_db` in full and grep it for `schema_versions` writes. | Whether any watcher / debounced-save path calls `links_backfill::is_needed` or `sky_backfill::is_needed` with a non-`state.db` connection. Grep for those symbols returned only the call sites listed above (links_backfill.rs:69, :170; sky_backfill.rs:66), all on `state.db`. I did not read watcher.rs this session.

### SLICE: MAP SLICE 5 — the read-side / analytics surfaces (17 sites across cache.rs, sight.rs, tension.rs, strata.rs, inspector360.rs, libraries.rs).

BOUNDARY VERDICT: **16 of the 17 sites are reads. Exactly ONE is a writer: libraries.rs:7490.**

None of the 16 reads writes a row to any database. Verified by grep for `execute|INSERT|UPDATE|DELETE|fs::write` over sight.rs (zero hits), strata.rs (zero hits), inspector360.rs (zero hits), and tension.rs (hits at :532,:547,:548,:560,:561,:574,:575,:680,:681,:684,:685 — ALL past the `#[cfg(test)]` boundary at tension.rs:522). cache.rs:1654 is likewise test-only: the `#[cfg(test)] mod tests` boundary is cache.rs:1581 and the site sits in the fixture builder `make_synthetic_sky_db` (cache.rs:1589).

THE ONE WRITER — libraries.rs:7490. `is_known_type` inside `rewrite_wikilinks_in_text` (libraries.rs:7478) decides whether `[[Foo::Old]]` is a typed link and therefore whether its tail gets rewritten. Its result is written to DISK: `rewrite_candidates` (libraries.rs:7326) calls it inside `crate::write_gate::gate_rmw(path, "cascade", …)` at libraries.rs:7346, and `gate_rmw` (write_gate.rs:652) calls `atomic_write(path, &updated)` at write_gate.rs:667. This is the rename cascade, entered from `update_links_on_rename` (libraries.rs:6792, `#[tauri::command(async)]`).

TODAY that writer is federation-FENCED, on both of its branches, and I verified both:
  - walk branch: `update_links_recursive(Path::new(&library_path), …)` is passed `foreign` (libraries.rs:6982-6986), the set built at libraries.rs:6856-6864 from `foreign_library_roots` (libraries.rs:406) → `foreign_roots_of` (libraries.rs:420), = every library in the federated `load_all_libraries` that is NOT in `try_load_libraries` (the universe's OWN libs).
  - seek branch: libraries.rs:6963, `if path_is_under_any(&path…, &foreign) { continue; }` — with the comment naming exactly this hazard ("a seek that followed such a row would rewrite a note INSIDE a linked universe").
So the cascade cannot cross into a child universe today, and libraries.rs:7490 therefore computes with the right vocabulary today. It becomes an H1 routed-write site the moment Phase 1.2 routes a rename — at which point the `foreign` fence is the thing being deliberately removed, and the owner's vocabulary must arrive with the route.

THE REAL SLICE-5 STORY — a SECOND, pre-existing vocabulary hazard, on the FEDERATED READ side, out of 1.2's scope but a live correctness question:

Three of these surfaces ALREADY loop over child-universe data with the PARENT's vocabulary, in shipped code:
  (a) cache.rs:516/:548/:1288 — `backlink_rows_in_schema`, `outgoing_rows_in_schema`, `read_links_in_schema` each take a `schema: &str` and are called ONCE PER ATTACHED SCHEMA (cache.rs:629-631, :668-670 over `get_federated_schemas`, cache.rs:785, which returns `main` plus every alias in `state.federation.attached()`). The `structural_not_in_clause` string interpolated into `{schema}.note_links` is built from the ACTIVE registry — so the parent's structural ids are subtracted from the CHILD's rows, and the child's own structural ids are not. No routing needed; this is today's behaviour on the federated conn (`state.federated_conn`, cache.rs:626).
  (b) strata.rs:168/199/208, inspector360.rs:284/343/368/375, libraries.rs:4040/4065 — all filesystem re-walkers reached from `compute_note_strata` (strata.rs:84), `get_360_view` (inspector360.rs:111) and `scan_library_links` (libraries.rs:4019). Each gates on `validate_path_in_any_library` / `load_all_libraries`, and BOTH are federation-spanning: `load_all_libraries` (libraries.rs:186) → `resolve_universe_libraries` (universe.rs:1520) → `resolve_libraries_recursive` (universe.rs:602), which at universe.rs:640-649 recurses into `universe.json`'s children and extends the list with their libraries. `validate_path_in_library` (libraries.rs:709) admits a library ROOT passed as `file_path`, since `file_canon.starts_with(&library_canon)` is true for the path itself. The doc comment at libraries.rs:727-728 states this outright: "within any registered library (including child universe libraries)". So a cUniverse library path passes the access check and gets its .md files parsed with the parent's `snapshot()`.

CORRECTION OF AN IN-SOURCE CLAIM: tension.rs:88-92 comments that "cUniverse library paths are not registered own-libraries and are refused" by `validate_path_in_any_library`. That is FALSE as written — per libraries.rs:186/709/727-728 above, they are accepted. The reason `detect_tensions` still returns nothing useful for a child is different and downstream: `load_notes_from_db` (tension.rs:230) queries the ACTIVE universe's `state.db` (tension.rs:97-101) filtered by `library_name`, and a child's notes have no rows there. Empty report, not a wrong-vocabulary report.

FALSE POSITIVE, NAMED: sight.rs:113 is NOT a registry read. `is_null_type` (link_types.rs:493-495) is `matches!(id, "associative" | "relates" | "")` — a pure constant match that never touches `cell()`/`REGISTRY`. It is vocabulary-independent and cannot participate in H1. It should come off the H1 site count.
- src-tauri/src/cache.rs:516 `backlink_rows_in_schema` via snapshot() | computes: the ` AND link_type NOT IN (...)` structural-exclusion fragment interpolated into the SQL that reads `{schema}.note_links` — i.e. which link types are hidden from the Backlinks panel, per schema | reached on: read-only analytics query — IPC `get_backlink_rows` (cache.rs:602), federated: called once per schema from `get_federated_schemas` (cache.rs:785) at cache.rs:629-631 over `state.federated_conn`, or via `with_read_conn` at cache.rs:634-641 when single-schema | impact: WRONG_READ_ONLY_ANSWER | conn in scope: true
    sig: fn backlink_rows_in_schema(
    conn: &Connection,
    schema: &str,
    targets_lower: &[String],
) -> Result<Vec<NoteLink>, String>
- src-tauri/src/cache.rs:548 `outgoing_rows_in_schema` via snapshot() | computes: same structural NOT-IN fragment, for the Outgoing-links panel's per-note query against `{schema}.note_links` | reached on: read-only analytics query — IPC `get_outgoing_rows` (cache.rs:651), federated: once per schema at cache.rs:668-670 (federated_conn) or cache.rs:673-679 (with_read_conn) | impact: WRONG_READ_ONLY_ANSWER | conn in scope: true
    sig: fn outgoing_rows_in_schema(
    conn: &Connection,
    schema: &str,
    source_path: &str,
) -> Result<Vec<NoteLink>, String>
- src-tauri/src/cache.rs:1288 `read_links_in_schema` via snapshot() | computes: the structural NOT-IN fragment for the BOOT link bundle / full federated links payload — which edges the frontend cognitive graph consumers ever see | reached on: read-only boot bundle — schema-parameterized, the federated concatenation path described in its own doc comment (cache.rs:1275-1281) | impact: WRONG_READ_ONLY_ANSWER | conn in scope: true
    sig: fn read_links_in_schema(conn: &Connection, schema: &str) -> Result<Vec<NoteLink>, String>
- src-tauri/src/cache.rs:1654 `make_synthetic_sky_db` via is_structural_type | computes: whether a fixture edge is skipped before INSERT into a synthetic test sky_links table | reached on: TEST ONLY — inside `#[cfg(test)] mod tests` whose boundary is cache.rs:1581; the fn is declared at cache.rs:1589 and opens its own `Connection::open(path)` at cache.rs:1598 | impact: NO_IMPACT | conn in scope: true
    sig: fn make_synthetic_sky_db(
        path: &std::path::Path,
        nodes: &[(&str, &str, &str, &str)], // (id, name, path, library_name)
        links: &[(&str, &str, &str)],       // (source_path, target_name, link_type)
        aliases: &[(&str, &str)],           // (alias_lower, target_path)
        stamp_ready: bool,
    )
- src-tauri/src/sight.rs:77 `constellation_sight_centrality` via snapshot() | computes: the structural NOT-IN fragment for the Brandes betweenness-centrality input query `SELECT source_name, target_name, link_type FROM note_links WHERE status = 'active'{sx}` (sight.rs:79-80) | reached on: read-only analytics query (CNS centrality) — IPC only, no internal Rust caller; connection is `state.db.lock()` (sight.rs:71-73), the ACTIVE universe's writer connection, table unqualified so schema = main. NOT federated. | impact: NO_IMPACT | conn in scope: true
    sig: pub fn constellation_sight_centrality(
    app: tauri::AppHandle,
) -> Result<LensCentralityData, String>
- src-tauri/src/sight.rs:113 `compute_centrality_from_links` via is_null_type | computes: whether an edge's link_type collapses to None (default edge weight). NOT A REGISTRY READ: `is_null_type` at link_types.rs:493-495 is `matches!(id, "associative" | "relates" | "")` — a pure constant match that never touches `cell()`/REGISTRY. Vocabulary-independent. | reached on: pure function, no AppHandle, no connection — called from constellation_sight_centrality (sight.rs:93) | impact: NO_IMPACT | conn in scope: false
    sig: pub(crate) fn compute_centrality_from_links(
    rows: Vec<(String, String, Option<String>)>,
) -> LensCentralityData
- src-tauri/src/tension.rs:277 `load_notes_from_db` via snapshot() | computes: the structural NOT-IN fragment for the per-library outgoing-edge query `... FROM note_links WHERE library_name = ?1 AND status = 'active'{sx}` (tension.rs:280-283) — which edges feed the orphan / SPOF / contradiction verdicts | reached on: read-only analytics query — IPC `detect_tensions` (tension.rs:83); connection is `state.db.lock()` (tension.rs:98-100), the ACTIVE universe's DB, tables unqualified. Scope is narrowed by `library_name`, not by schema. | impact: NO_IMPACT | conn in scope: true
    sig: fn load_notes_from_db(
    conn: &rusqlite::Connection,
    library_name: &str,
) -> Result<HashMap<String, NoteInfo>, String>
- src-tauri/src/strata.rs:168 `scan_notes_recursive` via snapshot() | computes: `reg` — the registry clone taken once per directory and handed to `resolve_wikilink_type` at strata.rs:208; it decides which `[[x::y]]` heads count as types, i.e. the parse decision typed-vs-untyped for every note in the walk | reached on: read-only analytics — filesystem walk from IPC `compute_note_strata` (strata.rs:84), rooted at the caller-supplied `library_path` after `validate_path_in_any_library` (strata.rs:90). That gate is FEDERATION-SPANNING (libraries.rs:186→universe.rs:1520→universe.rs:602), so the root can be a cUniverse library while `reg` is the parent's. | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: fn scan_notes_recursive(
    dir: &Path,
    re: &regex::Regex,
    notes: &mut HashMap<String, NoteRecord>,
)
- src-tauri/src/strata.rs:199 `scan_notes_recursive` via structural_frontmatter_targets | computes: the lowercased wikilink targets declared under a STRUCTURAL frontmatter key, used at strata.rs:210-212 to skip them as non-cognitive outgoing links. Internally calls `snapshot()` (link_types.rs:385) and returns an EMPTY set if no structural type is registered (link_types.rs:386-388) — so a parent with no structural lane fails OPEN over a child that has one. | reached on: read-only analytics — same filesystem walk as strata.rs:168 | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: fn scan_notes_recursive(
    dir: &Path,
    re: &regex::Regex,
    notes: &mut HashMap<String, NoteRecord>,
)
- src-tauri/src/strata.rs:208 `scan_notes_recursive` via other (resolve_wikilink_type, driven by the `reg` snapshot taken at strata.rs:168) | computes: (target, Option<link_type>) per wikilink — the parse decision typed-vs-untyped, with include_associative=true; the resulting `outgoing_types` set drives the stratum bonuses at strata.rs:258-265 | reached on: read-only analytics — same filesystem walk as strata.rs:168 | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: fn scan_notes_recursive(
    dir: &Path,
    re: &regex::Regex,
    notes: &mut HashMap<String, NoteRecord>,
)
- src-tauri/src/inspector360.rs:284 `get_360_view` via snapshot() | computes: `snapshot().ids()` — the full ordered type-id list, differenced against `used_types` to produce `missing_link_types`, the 'blind spots / gaps' list rendered in the 360 view | reached on: read-only analytics query — IPC `get_360_view` (inspector360.rs:111), gated by `validate_path_in_any_library` (inspector360.rs:115) which admits cUniverse library paths; fn body spans inspector360.rs:111-316 | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: pub fn get_360_view(
    app: tauri::AppHandle,
    library_path: String,
    note_path: String,
) -> Result<Note360View, String>
- src-tauri/src/inspector360.rs:343 `scan_all_notes` via snapshot() | computes: `reg` — the per-directory registry clone handed to `resolve_wikilink_type` at inspector360.rs:375 with include_associative=FALSE (the 360.3D matrix treats `associative` as untyped) | reached on: read-only analytics — filesystem re-walk of the whole library from get_360_view; fn spans inspector360.rs:332-415 | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: fn scan_all_notes(
    dir: &Path,
    link_re: &regex::Regex,
    tag_re: &regex::Regex,
    notes: &mut HashMap<String, NoteInfo>,
)
- src-tauri/src/inspector360.rs:368 `scan_all_notes` via structural_frontmatter_targets | computes: structural-keyed frontmatter targets, used at inspector360.rs:377-379 to skip them; same fail-open-on-empty behaviour as strata.rs:199 (link_types.rs:386-388) | reached on: read-only analytics — same filesystem re-walk as inspector360.rs:343 | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: fn scan_all_notes(
    dir: &Path,
    link_re: &regex::Regex,
    tag_re: &regex::Regex,
    notes: &mut HashMap<String, NoteInfo>,
)
- src-tauri/src/inspector360.rs:375 `scan_all_notes` via other (resolve_wikilink_type, driven by the `reg` snapshot taken at inspector360.rs:343) | computes: the parse decision typed-vs-untyped for every edge in the 360 matrix (include_associative=false) | reached on: read-only analytics — same filesystem re-walk as inspector360.rs:343 | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: fn scan_all_notes(
    dir: &Path,
    link_re: &regex::Regex,
    tag_re: &regex::Regex,
    notes: &mut HashMap<String, NoteInfo>,
)
- src-tauri/src/libraries.rs:4040 `scan_links_recursive` via snapshot() | computes: `reg` — the per-directory registry clone handed to `resolve_wikilink_type` at libraries.rs:4065 (include_associative=true) | reached on: read-only analytics — filesystem walk from IPC `scan_library_links` (libraries.rs:4019), whose access check is `load_all_libraries(&app).iter().any(|v| v.path == library_path)` (libraries.rs:4020-4023) — federation-spanning, so a cUniverse library path passes. Returns Vec<NoteLink> to the frontend; writes nothing. | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: fn scan_links_recursive(dir: &Path, re: &regex::Regex, links: &mut Vec<NoteLink>, library_name: &str, exclude: &std::collections::HashSet<String>)
- src-tauri/src/libraries.rs:4065 `scan_links_recursive` via other (resolve_wikilink_type, driven by the `reg` snapshot taken at libraries.rs:4040) | computes: the parse decision typed-vs-untyped for each scanned wikilink; the `link_type` lands in the returned NoteLink (libraries.rs:4079-4092) — an in-memory result, never persisted by this path | reached on: read-only analytics — same filesystem walk as libraries.rs:4040 | impact: WRONG_READ_ONLY_ANSWER | conn in scope: false
    sig: fn scan_links_recursive(dir: &Path, re: &regex::Regex, links: &mut Vec<NoteLink>, library_name: &str, exclude: &std::collections::HashSet<String>)
- src-tauri/src/libraries.rs:7490 `rewrite_wikilinks_in_text` via is_known_type | computes: whether the `head` in `[[head::Old]]` is a REGISTERED type. TRUE ⇒ rewrite the tail to the new name (libraries.rs:7492-7493); FALSE ⇒ leave the whole match untouched (libraries.rs:7495). This decides what is WRITTEN TO THE .md FILE. | reached on: RENAME CASCADE — the only writer in this slice. IPC `update_links_on_rename` (libraries.rs:6792) → `rewrite_candidates` (libraries.rs:7326) → `crate::write_gate::gate_rmw(path, "cascade", …)` at libraries.rs:7346 → `atomic_write(path, &updated)` at write_gate.rs:667. TODAY fenced from child universes on BOTH branches by the `foreign` set (built libraries.rs:6856-6864 via foreign_library_roots libraries.rs:406 / foreign_roots_of libraries.rs:420): seek branch skips at libraries.rs:6963, walk branch is passed `foreign` at libraries.rs:6982-6986. | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn rewrite_wikilinks_in_text(content: &str, re: &regex::Regex, new_name: &str) -> String
NOTES: CALL GRAPH — HOW DEEP THREADING WOULD HAVE TO GO

Two distinct shapes in this slice, and they need different remedies.

**Shape A — the schema-parameterized DB readers (cache.rs:516, :548, :1288).**
These are the cheapest to fix and the only ones already structurally ready: each ALREADY takes `schema: &str` alongside `conn: &Connection`, and each is called in a `for schema in &schemas` loop (cache.rs:629-631, :636-639, :668-670, :675-678). The vocabulary is a sibling of the schema the caller already threads. A `&LinkTypeRegistry` (or a `Vocab` handle) added next to `schema` is a 3-signature change plus the two loop bodies; no new plumbing. Callers `get_backlink_rows` (cache.rs:602) and `get_outgoing_rows` (cache.rs:651) are `#[tauri::command(async)]` leaves with no internal Rust callers (verified by grep: only lib.rs:522 / lib.rs:523 registration hits), so the change stops there. Note that ONE connection serves all schemas — `state.federated_conn` (cache.rs:626) — which is exactly why connection-bound vocabulary does NOT work for this family. Per-schema vocabulary must be a PARAMETER here, not a property of the connection. That is a real constraint on 1.2's design choice between "threaded through the call" and "bound to the connection": the federated read path forces the former for at least these three.

**Shape B — the filesystem re-walkers (strata.rs, inspector360.rs, libraries.rs:4040/4065).**
Deeper and uglier. Each takes its registry via a `snapshot()` call INSIDE a recursive walker (`scan_notes_recursive` strata.rs:156, `scan_all_notes` inspector360.rs:332, `scan_links_recursive` libraries.rs:4033), re-snapshotting once per directory, and none of them has a connection or an AppHandle in scope — they have only `dir`, the regexes, and the accumulator. Threading a vocabulary means adding a parameter to each recursive walker and to its entry command (`compute_note_strata` strata.rs:84, `get_360_view` inspector360.rs:111, `scan_library_links` libraries.rs:4019). Depth is 2 (command → walker → recursive self). All three entry points are IPC leaves — verified no internal Rust caller for `compute_note_strata`, `get_360_view`, `detect_tensions`, `scan_library_links`, `constellation_sight_centrality` (the only hits outside lib.rs registration are prose in doc comments). So the blast radius is small; it is just three separate hand-plumbed chains, which is the drift risk the whole Whole-Ecosystem law is about.

**The connection-in-scope tally**, since it bears on "threaded vs connection-bound": 6 of 17 sites have a `&Connection` in scope (cache.rs:516, :548, :1288, :1654-test, sight.rs:77 via `state.db.lock()`, tension.rs:277). The other 11 have none — they are filesystem walkers or pure functions. **A purely connection-bound vocabulary would not reach 11 of the 17 sites in this slice at all.** That is the single most load-bearing fact this slice contributes to the 1.2 design decision.

**SCOPE DISCIPLINE — what 1.2 should and should not take on.**
In 1.2's scope: libraries.rs:7490 only, and only if 1.2 routes renames. It is the one site whose wrong answer reaches durable storage (a .md file, and thereafter the child's index when the watcher reindexes it). The other 16 cannot corrupt anything — they compute answers that are displayed and discarded.

Out of 1.2's scope but a genuine open correctness question: Shape A and Shape B both already give wrong answers over federated data TODAY, with no Router involved. cache.rs:516/:548/:1288 do it unconditionally on every federated boot (the loop over attached schemas is unavoidable once a cUniverse is linked). Shape B does it only when a cUniverse library path is actually supplied by the caller — the Rust gate admits it (verified), but whether the UI ever supplies one is a frontend question I did not verify (see unverified).

**A FALSE CLAIM FOUND IN SOURCE, worth fixing wherever this lands.**
tension.rs:88-92 asserts in a comment that `validate_path_in_any_library` REFUSES cUniverse library paths ("cUniverse library paths are not registered own-libraries and are refused"). It does not. `validate_path_in_any_library` (libraries.rs:730) iterates `load_all_libraries` (libraries.rs:186), which resolves through `resolve_universe_libraries` (universe.rs:1520) → `resolve_libraries_recursive` (universe.rs:602), which extends the list with every child universe's libraries at universe.rs:640-649; and `validate_path_in_library` (libraries.rs:709) accepts a library root passed as `file_path` because `file_canon.starts_with(&library_canon)` holds for the path itself. The function's OWN doc comment (libraries.rs:727-728) says "including child universe libraries" — the two comments contradict each other, and libraries.rs is the correct one. Anyone reading tension.rs:88-92 as the federation contract for the whole analytics family (which is how it reads) will conclude these surfaces are fenced when they are not.

**ONE SITE SHOULD COME OFF THE H1 LIST.** sight.rs:113 reads `is_null_type`, which at link_types.rs:493-495 is `matches!(id, "associative" | "relates" | "")` — a constant match that never touches `cell()` or REGISTRY. It cannot vary with vocabulary and is not an H1 site. The vocab_harness header (federation/vocab_harness.rs:5) counts "26 call sites across 11 files"; at least this one is a false positive, and the same `is_null_type` false-positive shape may inflate the count elsewhere.
UNVERIFIED: Whether MIG-111 Phase 1.2 will route RENAMES at all. libraries.rs:7490's routed_write_impact is CORRUPTS_CHILD_ROWS *conditional on that*; today the `foreign` fence (libraries.rs:6963, :6982-6986) makes it NO_IMPACT. I read the fence and the write path but the Router's intended operation set is a design question not readable from current source. To settle: the MIG-111 Phase 1.2 Architect/Plan doc, and whether `update_links_on_rename` appears in its routed-operation list. | Whether the FRONTEND ever passes a cUniverse library path to `compute_note_strata`, `get_360_view`, `scan_library_links`, or `detect_tensions`. I verified the Rust gate ADMITS such a path (libraries.rs:186 / :709 / :727-728 / universe.rs:602,640-649); I did NOT read the Svelte call sites to see which `library_path` value is actually supplied. This decides whether Shape B's WRONG_READ_ONLY_ANSWER is live-today or latent. To settle: grep the Svelte source for `invoke('compute_note_strata'`, `invoke('get_360_view'`, `invoke('scan_library_links'`, `invoke('detect_tensions'` and read where each gets its library_path. | Whether anything PERSISTS the output of `scan_library_links` (libraries.rs:4019). I verified there is no internal Rust caller (grep found only the lib.rs:422 registration and a prose mention at sight.rs:15), so from Rust it is a pure read. If a Svelte consumer writes the returned link_type values back through another IPC command, libraries.rs:4040/:4065 would become an indirect writer. To settle: grep the Svelte source for `scan_library_links` and trace what its result feeds. | inspector360.rs:441 `precompute_all_strata`, :490 `compute_stratum_for_note`, :507 `compute_maturity_for_note`, :516 `compute_provenance_for_note` — I did NOT read these bodies. The grep for `link_types::` in inspector360.rs returned only :284, :343, :368, :375, so they hold no registry call; but if any of them tests HARDCODED type-id string literals (the way strata.rs:258-265 tests "generalizes"/"causes"/"supports"), that is a separate custom-type-blindness defect this slice did not scope. To settle: read inspector360.rs:441-548. | `state.federated_conn` — I read its use at cache.rs:626 and :664 and the field's doc comment at search.rs:1380-1381, but I did NOT read `federation::attach::attach_all` to confirm the attached child databases are opened read-only. It matters for whether Shape A could ever become a writer. To settle: read src-tauri/src/federation/attach.rs.

### SLICE: MAP SLICE 6 — how a Connection is obtained today, and what "per-connection" could mean.
All line numbers read from source in this session at E:/مشاريع كلاود/Constellation/src-tauri/src.

════════ A. THE FIVE WAYS A CONNECTION COMES INTO EXISTENCE ════════

A1. `SearchState` — the process's whole connection inventory (search.rs:1341-1416), verbatim fields:
    pub db: Mutex<Option<Connection>>                       (1342)  ← THE WRITER
    pub read_db: Mutex<Option<Connection>>                  (1353)  ← PJ-066 §C3 read-only twin
    pub db_ready: std::sync::atomic::AtomicBool             (1363)
    pub init_lock: Mutex<()>                                (1374)
    pub federation: Mutex<crate::federation::FederationContext> (1379)
    pub federated_conn: Mutex<Option<Connection>>           (1400)
    pub federation_generation: std::sync::atomic::AtomicU64 (1415)
Constructed once by `SearchState::new()` (1418-1430) and registered as Tauri managed state at
lib.rs:360 `.manage(search::SearchState::new())`. **There is no map, no key, no per-universe slot
anywhere in it.** Universe identity is represented ONLY by the `federation_generation` epoch
counter (1415), bumped in `invalidate_search_state` (11228, fetch_add at 11236). A pool keyed by
`Owner` would be the first per-universe-keyed member this struct has ever had.

A2. `db_path` (search.rs:1465-1468) — the ONLY path resolver:
      `crate::universe::active_constellation_dir(app)?.join("search.db")`
    and `active_constellation_dir` (universe.rs:69-72) → `active_universe_dir` (universe.rs:340-344)
    which reads `UniverseState.active_path: Mutex<Option<PathBuf>>` (universe.rs:42). So db_path is
    AMBIENT — it answers "the active universe's db", never "this owner's db". A routed write cannot
    use it.

A3. The writer's lifecycle — `ensure_search_db_ready` (search.rs:11476-…):
    - lock-free fast path on `db_ready` (11483-11485);
    - `init_lock` serialises the slow path (11508-11511);
    - epoch captured BEFORE db_path (11537-11540);
    - schema-version gate: rename-aside, never delete (11567-11625);
    - **`crate::link_types::load_active(app);` at 11606 — the ONE place the process's vocabulary
      is installed, immediately before `let conn = init_db(&path)?;` at 11607**;
    - `open_read_only_search_conn(&path)` at 11630;
    - publish under one generation check: `*db = Some(conn)` (11649), read_db (11653-11656),
      `db_ready.store(true)` (11659).
    Teardown: `invalidate_search_state` (11228) sets db=None (11248), read_db=None (11270),
    federated_conn=None (11277), federation.reset() (11282).

A4. Readers: `with_read_conn<T>(state: &SearchState, f: impl FnOnce(&Connection) -> Result<T,String>)`
    (search.rs:1492-1507) — tries `read_db`, falls back to `db`. `open_read_only_search_conn`
    (1475-1485) opens SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX, busy_timeout 5000,
    `PRAGMA query_only=ON; mmap_size=…`, then `register_fts5_tokenizer`.

A5. Writers: everything else takes the mutex directly — `state.db.lock()` appears **146 times across
    37 files** (measured by grep this session). The canonical shape is `reindex_single_note`
    (search.rs:12682-12686 signature; 12688 `let db = state.db.lock()…`; 12689 `if let Some(conn) =
    db.as_ref()`; 12718 `index_note(conn, note_path, library_name, true)?`).
    Two long-job variants open their OWN handle on the SAME active db rather than hold the mutex:
      • `reconcile_filesystem` (11968) → `Connection::open(&path)` at 11976, PRAGMA batch 11978,
        `register_fts5_tokenizer(&mut walk_conn)` 11984, busy_timeout 30s 11996 — never stored;
        it is the closest existing precedent for a routed per-write connection.
      • the federation thread (11787-11804) → `Connection::open(&path)` at 11798.

A6. `init_db` family — the only manufacturer of a schema-ready Connection:
      `pub(crate) fn init_db(path: &Path) -> Result<Connection, String>`            (4592) → Active
      `pub(crate) fn init_db_schema_only(path: &Path) -> Result<Connection, String>`(4597) → ForeignSchemaOnly
      `pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>` (4601)
    Body: `let owns = scope == InitScope::Active;` (4602); `Connection::open(path)` (4603) — READ-WRITE;
    WAL (4606); synchronous=NORMAL/busy_timeout=5000/mmap_size (4619-4622); recursive_triggers=ON (4641);
    `register_fts5_tokenizer(&mut conn)` (4649). It RETURNS an owned `Connection`, so it is already the
    natural constructor for a pooled entry. `InitScope` is defined at 4570-4589.

════════ B. THE FOUR SITES THAT ALREADY OPEN A SECOND UNIVERSE'S DATABASE ════════

B1. READ-ONLY ATTACH — federation/attach.rs:225-233:
      `let path_uri = db_path.to_string_lossy().replace('\\', "/");`
      `format!("ATTACH DATABASE 'file:{}?mode=ro' AS {}", path_uri, alias)` then `conn.execute(...)`.
    Mode: **URI mode=ro, attached onto an existing read-write Connection.** The host connection is the
    federation thread's own `Connection::open(&path)` (search.rs:11798) which ends up in
    `SearchState.federated_conn` (search.rs:11912). Entry point `attach_all(conn: &mut Connection,
    app: &tauri::AppHandle) -> Result<FederationContext, FederationError>` (attach.rs:125-128);
    per-child wrapper `attach_with_safety(conn: &mut Connection, db_path: &Path, alias: &str)`
    (attach.rs:212-216); schema probe `verify_schema` (attach.rs:266) via
    `PRAGMA {alias}.table_info(note_meta)`; cache tune `PRAGMA {alias}.cache_size = -512` (246-249).
    Cap 25 (attach.rs:45, enforced 140-153). Aliases `cu0…cu24` (attach.rs:60-62).

B2. InitScope::ForeignSchemaOnly — federation/migrate.rs:169:
      `let conn = crate::search::init_db_schema_only(cu_db_path)` → search.rs:4598 →
      init_db_scoped(path, ForeignSchemaOnly) → `Connection::open(path)` at 4603.
    Mode: **READ-WRITE, own handle on the child's file, dropped immediately (migrate.rs:175
    `drop(conn)` "to release the file lock before we return").** Called only from
    `run_migrations_on(cu_db_path: &Path, parent_universe_root: &Path) -> Result<(), MigrationError>`
    (migrate.rs:96-99), which is called from attach.rs:172 on a `schema_incomplete` attach failure.
    `owns == false` gates out: NOTE_META schema pass (4838), the note_meta_sky_ai DDL that
    interpolates stratum/maturity exprs (5640), 5891, 5933, `create_outgoing_link_triggers` (5969),
    dependent-tables MIG-003 (6219), 6284, 6312, mig003_step4 (6335).
    *(Adjacent, same file: `backup_database(src,dst)` (migrate.rs:71-84) opens src with
    `OpenFlags::SQLITE_OPEN_READ_ONLY` (72-75) and dst with `Connection::open` (77-78) and runs
    `rusqlite::backup::Backup` — the MIG-111 Phase 0.1 R11 replacement for fs::copy.)*

B3. FTS optimize prewarm — search.rs:11358-11474, `fn federation_prewarm(app: tauri::AppHandle,
    cu_roots: Vec<PathBuf>, start_generation: u64)`:
      `let cu_db_path = cu_root.join(".constellation").join("search.db");` (11380)
      `Connection::open(&cu_db_path)` (11381) — **READ-WRITE, no flags**;
      `PRAGMA busy_timeout=30000` (11397); `register_fts5_tokenizer(&mut warm_conn)` (11403);
      `INSERT INTO notes_fts(notes_fts) VALUES('optimize');` (11434-11436) — a genuine WRITE into a
      child universe's file, justified in the doc-comment at 11352-11357.
    Spawned at search.rs:11943-11945 after federation state is published. Handle dropped at 11468.

B4. TESTS — federation/attach.rs:306-330 and :332-351 build child DBs with `Connection::open(&db_path)`
    (318, 341); federation/integration_tests.rs opens synthetic mains at :36, :111, :163, :206, :252,
    :295, :332, :362, :411, :464, :524 and ATTACHes read-only with the same
    `ATTACH DATABASE 'file:{}?mode=ro' AS {}` form at :75 and :469;
    federation/migrate.rs tests at :338, :361, :373, :393, :425, :436, :510, :577, :608 plus
    `init_db_schema_only` at :618, :655, :733, :752 and `init_db` at :778.
    universe_lock.rs:338 opens a real search.db in the two-process proof.

════════ C. WHAT A "CONNECTION + ITS VOCABULARY" BUNDLE HAS TO BE ════════

C1. The target signature today (search.rs:7873):
      `pub(crate) fn index_note(conn: &Connection, note_path: &str, library_name: &str, force: bool)
         -> Result<IndexOutcome, String>`
    and its sibling `index_note_bulk` (7880), both one-line wrappers over
      `fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool,
         bulk: bool) -> Result<IndexOutcome, String>` (7884, body runs to 8627).
    **The vocabulary is not a parameter at any depth.** It is read from the process-global
    `REGISTRY: OnceLock<RwLock<LinkTypeRegistry>>` (link_types.rs:351) through `cell()`
    (link_types.rs:353-355) by free functions, at six points on the index_note chain — see `sites`.

C2. The ONLY vocabulary type that exists is
      `#[derive(Debug, Clone)] pub struct LinkTypeRegistry { types: Vec<LinkTypeDef> }`
      (link_types.rs:100-103) — private field, all access through its methods
      (is_known 171, is_link_type_value 181, ordered 184, rank 188, ids 191, sql_in_list 197,
       sql_rank_case 208, structural_ids 224, is_structural 229, cognitive_ids 234,
       sql_in_list_cognitive 241, sql_rank_case_cognitive 253, structural_not_in_clause 268,
       sentinel_rank 283, cognitive_sentinel_rank 292, fingerprint 300).
    `snapshot()` (link_types.rs:498-503) ALREADY hands out an owned clone, and
    `resolve_wikilink_type(reg: &LinkTypeRegistry, before_pipe: &str, after_pipe: Option<&str>,
     include_associative: bool) -> (String, Option<String>)` (link_types.rs:451-456) is the existing,
    shipped precedent for **registry-as-parameter**. So the bundle's vocabulary half is
    `&LinkTypeRegistry` (borrow) or `Arc<LinkTypeRegistry>` (if the pool owns it); no new type is
    needed for the vocabulary itself.

C3. **"Bound to the connection" cannot be literal.** Four of the six vocabulary readers on the
    index_note chain have NO `&Connection` in scope at all — they are pure text parsers:
      `fn extract_wikilinks(content: &str) -> Vec<String>`                        (search.rs:7068)
      `fn extract_typed_links(content: &str) -> Vec<TypedLink>`                   (search.rs:7222)
      `fn parse_link_body(body: &str) -> Option<(String, String, String)>`        (search.rs:7243)
      `fn emit_frontmatter_links(wl,key,value,out,seen)`                          (search.rs:7361)
      plus `pub fn structural_frontmatter_targets(frontmatter: &str) -> HashSet<String>`
                                                                                  (link_types.rs:385)
    The only per-connection state this codebase attaches to a rusqlite handle is the FTS5 tokenizer
    (`register_fts5_tokenizer(conn: &mut Connection)` search.rs:1572-1581), and it lives inside
    SQLite's own FTS5 registry — it is not readable back from Rust. rusqlite is pinned at
    `version = "0.31", features = ["bundled","backup"]` (src-tauri/Cargo.toml:46) — no serialize/
    user-data feature. **Therefore the bundle is a THREADED PARAMETER, not a property SQLite holds.**

C4. Concretely, the minimum bundle is a pair — e.g.
      `struct RoutedCtx<'a> { conn: &'a Connection, vocab: &'a LinkTypeRegistry }`
    — that must replace `&Connection` at index_note / index_note_bulk / index_note_impl, and must ALSO
    be threaded (as `&LinkTypeRegistry` alone) into the five conn-less parsers above, plus the
    post-index maintenance pass the harness already exercises:
      `pub(crate) fn maintain_incoming_after_save(conn: &Connection, note_path: &str,
         old_targets: &HashSet<String>, old_name: &str, old_aliases: &HashSet<String>)
         -> rusqlite::Result<()>`                                                 (search.rs:2637-2643)
        → `incoming_aggregate_assignments("note_meta")` at 2661
      `fn maintain_sky_after_save(conn, note_path, old_targets, old_name, old_aliases)
         -> rusqlite::Result<()>`                                                 (search.rs:2706-2712)
        → `stratum_sql_expr()` / `maturity_sql_expr()` at 2719-2720
    and the DDL constructor, which bakes vocabulary into the CHILD's `sqlite_master`:
      `pub(crate) fn create_outgoing_link_triggers(conn: &Connection) -> Result<(), String>`
                                                                                  (search.rs:2286)
      called from init_db_scoped at 5970 under `if owns`.

C5. **Not every read is a hazard — three of the six are vocabulary-INVARIANT, provably.**
    `structural` is `true` for exactly `contains` + `parent` and can never be anything else:
    seeds() sets `structural: true` only on those two (link_types.rs:93-94, all eight cognitive seeds
    get `structural: false` at line 76); `merge()` forces `structural = false` for a SEED_IDS delta
    (link_types.rs:131), `structural = true` for a STRUCTURAL_SEED_IDS delta (148), and
    `structural = false` for every custom delta (159). So `is_structural_type`, `structural_ids` and
    `structural_not_in_clause` return the same answer under any two universes' link-types.json.
    That makes search.rs:8018, 8485, 8561 (all three `is_structural_type` calls inside
    index_note_impl), search.rs:189 (`stratum_sql_expr`), search.rs:267 (`maturity_sql_expr`) and
    search.rs:5540 (the sky_link trigger DDL, which is NOT under `if owns`) NO_IMPACT for the routed-
    write question. The genuinely divergent reads are the `is_known_type` ones (7244, 7371) and the
    `snapshot()`-derived cognitive lists (2243, 2489) — plus 7079 indirectly, via
    `resolve_wikilink_type`'s `is_link_type_value` (link_types.rs:424, 457-459).

════════ D. WHERE A POOL WOULD LIVE, AND WHAT IS ALREADY THERE ════════

D1. `SearchState` (search.rs:1341, managed at lib.rs:360). Contents listed in A1. Reached only via
    `app.state::<SearchState>()` — which means **any pool user needs an `AppHandle`, and neither
    `index_note` (7873) nor `init_db` (4592) has one.** Its slots are deliberately WIPED on universe
    switch (invalidate_search_state 11246-11283), so a pool placed here dies with every switch —
    correct for the active universe, arguably wrong for a child that is not switching.
D2. A new process-global static. Direct precedents, all read this session:
      `static REGISTRY: OnceLock<RwLock<LinkTypeRegistry>>`          (link_types.rs:351)
      `static ACTIVE_OWNER: OnceLock<Mutex<Option<OwnerLock>>>`      (universe_lock.rs:218, accessor 220-222)
      `static MIGRATION_ACTIVE: AtomicBool`                          (search.rs:2893)
      `static KH_RECOMPUTE_IN_FLIGHT: AtomicBool`                    (search.rs:10281)
      `static WAL_DAEMON_GENERATION: AtomicU64`                      (search.rs:11181)
      `static FAILED: OnceLock<Mutex<HashSet<String>>>`              (search.rs:4153, fn-local)
      `static HARNESS_LOCK: std::sync::Mutex<()>`                    (federation/vocab_harness.rs:133)
    A static needs no AppHandle and survives a switch — which is exactly the property that makes
    `link_types::REGISTRY` the bug in the first place, so a static pool must be keyed, never ambient.
D3. Another `.manage()` line beside the existing ones (lib.rs:358-371: WatcherState, UniverseState,
    SearchState, EmbeddingState, ScanState, RepairState, HealState, NscBackfillState, BulkAcceptState).
    Same AppHandle constraint as D1.

D4. **The key.** `pub struct Owner { pub root: PathBuf, pub is_active: bool }` (federation/owner.rs:58-73),
    produced by `resolve_owner_in(path: &str, active: &Path, federation: &[PathBuf]) -> Result<Owner,String>`
    (owner.rs:111) and `resolve_owner(app: &tauri::AppHandle, path: &str) -> Result<Owner,String>` (owner.rs:149).
    `root` is the **stripped** form (owner.rs:80-89 `strip_verbatim`, applied at 123). Its doc-comment
    (owner.rs:59-68) states the rule verbatim: *"Deriving a lock or pool key from this: go through
    `universe_lock::canon`, never through string equality… Two keys for one universe would mean two
    locks, which is no lock at all."* `pub(crate) fn canon(root: &Path) -> PathBuf` is
    universe_lock.rs:87-89 — `fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf())`, i.e. it
    KEEPS the `\\?\` verbatim form. The two forms differ; only `canon` reconciles them.
    `resolve_owner` has **no production caller yet** (grep this session: `resolve_owner` appears only
    inside owner.rs).

D5. Ownership lifetime, if the pool holds child connections open. `pub struct OwnerLock { file: File,
    root: PathBuf }` (universe_lock.rs:69-72) releases on `Drop` (177-185). `ACTIVE_OWNER`
    (universe_lock.rs:218) holds **exactly one** lock — the active universe's, swapped in `activate`
    (231-255, `*slot = None` first at 236). So `held_by_us(root)` (259-265) is FALSE for every child
    universe today, and `is_cuniverse_open_elsewhere` (migrate.rs:234-256) consequently runs the
    owner-lock probe against children rather than short-circuiting. A pool that keeps a child's
    database open needs either a second ownership notion or a multi-entry ACTIVE_OWNER.
- src-tauri/src/search.rs:7873 `index_note` via other (no direct read — it is the entry point whose signature has no vocabulary parameter) | computes: the whole per-note index write; delegates to index_note_impl | reached on: save path (constellation_search_reindex → reindex_single_note), rename cascade, Base cell edit, watcher reindex_changed_paths, cid heal inside init_db | impact: UNVERIFIED | conn in scope: true
    sig: pub(crate) fn index_note(conn: &Connection, note_path: &str, library_name: &str, force: bool) -> Result<IndexOutcome, String>
- src-tauri/src/search.rs:7884 `index_note_impl` via is_structural_type | computes: the body of the per-note index write; contains three direct is_structural_type reads (8018, 8485, 8561) and reaches three more readers through the parse chain | reached on: every index_note / index_note_bulk call — save path, rename cascade, bulk repair walk, Full re-read | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String>
- src-tauri/src/search.rs:8018 `index_note_impl` via is_structural_type | computes: drops body-authored structural-typed links from typed_links (PJ-065: structural edges are frontmatter-only) | reached on: every index_note call | impact: NO_IMPACT | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String>
- src-tauri/src/search.rs:8485 `index_note_impl` via is_structural_type | computes: the `structural` argument to link_row_is_preserved — whether an existing edge's earned weight/confidence/traversal survives the rebuild | reached on: every index_note call that finds existing note_links rows | impact: NO_IMPACT | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String>
- src-tauri/src/search.rs:8561 `index_note_impl` via is_structural_type | computes: which INSERT branch an edge takes — the structural branch writes confidence 'structural', weight 1.0, traversal 0, seq; the cognitive branch restores preserved earned data | reached on: every changed/added edge on every index_note call | impact: NO_IMPACT | conn in scope: true
    sig: fn index_note_impl(conn: &Connection, note_path: &str, library_name: &str, force: bool, bulk: bool) -> Result<IndexOutcome, String>
- src-tauri/src/search.rs:7244 `parse_link_body` via is_known_type | computes: THE parse decision typed-vs-untyped: whether `[[refutes::Target]]` yields (type=refutes, target=Target) or falls through to an untyped link whose target is the literal string 'refutes::target' | reached on: every wikilink on every index_note call, via extract_typed_links (search.rs:7232) and emit_frontmatter_links (search.rs:7378) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn parse_link_body(body: &str) -> Option<(String, String, String)>
- src-tauri/src/search.rs:7371 `emit_frontmatter_links` via is_known_type | computes: the property-name-as-type decision: a frontmatter key that is a known type becomes the link_type, otherwise the link is stamped 'associative' | reached on: every frontmatter wikilink on every index_note call, via extract_frontmatter_typed_links (called at search.rs:8025) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn emit_frontmatter_links(
    wl: &regex::Regex,
    key: &str,
    value: &str,
    out: &mut Vec<TypedLink>,
    seen: &mut std::collections::HashSet<String>,
)
- src-tauri/src/search.rs:7079 `extract_wikilinks` via structural_frontmatter_targets | computes: the set of structural-frontmatter targets to EXCLUDE from note_meta.outgoing_links_json; the set's membership depends on the vocabulary because structural_frontmatter_targets resolves each wikilink through resolve_wikilink_type → is_link_type_value (link_types.rs:424, 457-459) | reached on: every index_note call (invoked at search.rs:7927) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: fn extract_wikilinks(content: &str) -> Vec<String>
- src-tauri/src/link_types.rs:390 `structural_frontmatter_targets` via snapshot | computes: the lowercased wikilink targets declared under a structural frontmatter key; early-returns empty when structural_ids() is empty (line 391) | reached on: index_note via extract_wikilinks; also strata.rs / inspector360.rs content scanners | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub fn structural_frontmatter_targets(frontmatter: &str) -> std::collections::HashSet<String>
- src-tauri/src/search.rs:2489 `incoming_aggregate_assignments` via snapshot | computes: the SQL IN-list (sql_in_list_cognitive), the rank CASE (sql_rank_case_cognitive), the sentinel (cognitive_sentinel_rank) and the structural NOT-IN for note_meta.incoming_count / incoming_link_types / incoming_top_rank | reached on: save path — maintain_incoming_after_save builds `UPDATE note_meta SET {assign} WHERE path = ?1` at search.rs:2659-2661 and executes it per affected note | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub(crate) fn incoming_aggregate_assignments(np: &str) -> String
- src-tauri/src/search.rs:2243 `outgoing_aggregate_assignments` via snapshot | computes: the IN-list, rank CASE and sentinel for note_meta.outgoing_count / outgoing_link_types / outgoing_link_types_json / outgoing_top_rank | reached on: init_db (trigger DDL bodies, via create_outgoing_link_triggers) and the links_backfill recompute path | impact: PERSISTS_WRONG_DDL | conn in scope: false
    sig: pub(crate) fn outgoing_aggregate_assignments(src: &str) -> String
- src-tauri/src/search.rs:2286 `create_outgoing_link_triggers` via other (indirect — interpolates outgoing_aggregate_assignments twice, at the `ins =` / `del =` format args) | computes: the persisted bodies of note_links_outgoing_ai / _ad / _au in sqlite_master; DROPs first so they always carry the CURRENT registry | reached on: init_db_scoped at search.rs:5970, gated `if owns` (5969) — so today it never fires on a foreign database; also the reconcile TriggerWindow recreate | impact: PERSISTS_WRONG_DDL | conn in scope: true
    sig: pub(crate) fn create_outgoing_link_triggers(conn: &Connection) -> Result<(), String>
- src-tauri/src/search.rs:189 `stratum_sql_expr` via snapshot | computes: the /*SX*/ structural-exclusion fragment inside the sky stratum expression — vocabulary-INVARIANT (structural_not_in_clause only) | reached on: init_db note_meta_sky_ai DDL (search.rs:5640 `if owns`), maintain_sky_after_save (search.rs:2719), sky_backfill | impact: NO_IMPACT | conn in scope: false
    sig: pub(crate) fn stratum_sql_expr() -> String
- src-tauri/src/search.rs:267 `maturity_sql_expr` via snapshot | computes: the /*SX*/ structural-exclusion fragment inside the sky maturity CASE — vocabulary-INVARIANT (structural_not_in_clause only) | reached on: init_db maturity trigger DDL (search.rs:5956), maintain_sky_after_save (search.rs:2720), sky_backfill | impact: NO_IMPACT | conn in scope: false
    sig: pub(crate) fn maturity_sql_expr() -> String
- src-tauri/src/search.rs:5540 `init_db_scoped` via snapshot | computes: `sx_new` — the structural NOT-IN clause baked into the note_links_sky_ai / _au trigger bodies. NOT under an `if owns` guard, so it DOES run on a foreign database; harmless only because the structural set is vocabulary-invariant | reached on: init_db (Active) AND init_db_schema_only (ForeignSchemaOnly, via federation::migrate::run_migrations_on) | impact: NO_IMPACT | conn in scope: true
    sig: pub(crate) fn init_db_scoped(path: &Path, scope: InitScope) -> Result<Connection, String>
- src-tauri/src/search.rs:11606 `ensure_search_db_ready` via load_active | computes: installs the ACTIVE universe's vocabulary into the process-global registry, one line before `init_db(&path)` at 11607 — the single place the vocabulary is ever set in production | reached on: boot and every universe switch (it re-runs because invalidate_search_state resets state.db to None) | impact: UNVERIFIED | conn in scope: false
    sig: pub fn ensure_search_db_ready(app: &tauri::AppHandle) -> Result<(), String>
- src-tauri/src/link_types.rs:481 `set_active` via set_active | computes: replaces the whole process-global registry under a write lock — the mutation the LL-047 ruling forbids inside a routed write window | reached on: load_active (link_types.rs:523) from ensure_search_db_ready; save_universe_link_types; the vocab harness (federation/vocab_harness.rs:141, 233, 241) | impact: CORRUPTS_CHILD_ROWS | conn in scope: false
    sig: pub fn set_active(deltas: Vec<LinkTypeDef>)
- src-tauri/src/search.rs:2637 `maintain_incoming_after_save` via other (indirect — builds its UPDATE from incoming_aggregate_assignments at line 2661) | computes: recomputes note_meta.incoming_* for every affected target after a save; the harness calls this explicitly (federation/vocab_harness.rs:156) because index_note alone does not reach it | reached on: save path, from reindex_single_note (search.rs:12754), gated on incoming_links_backfill::is_built | impact: CORRUPTS_CHILD_ROWS | conn in scope: true
    sig: pub(crate) fn maintain_incoming_after_save(
    conn: &Connection,
    note_path: &str,
    old_targets: &std::collections::HashSet<String>,
    old_name: &str,
    old_aliases: &std::collections::HashSet<String>,
) -> rusqlite::Result<()>
- src-tauri/src/search.rs:2706 `maintain_sky_after_save` via other (indirect — interpolates stratum_sql_expr() / maturity_sql_expr() at lines 2719-2720) | computes: UPDATE sky_nodes SET stratum, maturity for every affected path after a save | reached on: save path, from reindex_single_note (search.rs:12765) | impact: NO_IMPACT | conn in scope: true
    sig: fn maintain_sky_after_save(
    conn: &Connection,
    note_path: &str,
    old_targets: &std::collections::HashSet<String>,
    old_name: &str,
    old_aliases: &std::collections::HashSet<String>,
) -> rusqlite::Result<()>
- src-tauri/src/search.rs:12682 `reindex_single_note` via other (no vocabulary read of its own — it is the connection-acquisition site: state.db.lock() at 12688, conn handed to index_note at 12718) | computes: the whole save-path sequence: old-body capture, incoming signature capture, index_note, ctse hook, maintain_incoming_after_save, maintain_sky_after_save | reached on: the save path IPC (constellation_search_reindex, search.rs:12275) and 12 other call sites across bases.rs, libraries.rs, reconcile.rs, shape.rs, tasks.rs, universe.rs, index_repair.rs | impact: UNVERIFIED | conn in scope: true
    sig: pub fn reindex_single_note(
    state: &SearchState,
    note_path: &str,
    library_name: &str,
) -> Result<MaintenanceOutcome, String>
- src-tauri/src/search.rs:11976 `reconcile_filesystem` via other (connection-acquisition site: Connection::open(&path) — its own dedicated read-write handle on the ACTIVE universe's db) | computes: a dedicated walk connection with PRAGMA batch (11978), FTS5 tokenizer (11984) and 30s busy_timeout (11996), never stored in SearchState — the closest existing precedent for a routed per-write connection | reached on: the index-repair runner, via reconcile_filesystem_guarded | impact: UNVERIFIED | conn in scope: true
    sig: fn reconcile_filesystem(app: &tauri::AppHandle, run_id: u64, force: bool) -> Result<SearchIndexStats, String>
- src-tauri/src/federation/owner.rs:111 `resolve_owner_in` via other (no vocabulary read — this is the pool KEY producer) | computes: Owner { root: PathBuf (stripped form), is_active: bool } by longest-match over {active} ∪ {federation descendants}; fail-closed on no match | reached on: nothing in production yet — grep this session found resolve_owner / resolve_owner_in referenced only inside owner.rs | impact: NO_IMPACT | conn in scope: false
    sig: pub fn resolve_owner_in(path: &str, active: &Path, federation: &[PathBuf]) -> Result<Owner, String>
NOTES: CALL-GRAPH OBSERVATIONS — how deep the threading has to go.

1. THE DEPTH IS FIVE FRAMES, AND FOUR OF THEM HAVE NO CONNECTION.
   reindex_single_note (search.rs:12682) → index_note (7873) → index_note_impl (7884) →
   extract_typed_links (7222) → parse_link_body (7243). Only the first three carry a
   `&Connection`; extract_typed_links and parse_link_body are pure `&str → value` functions.
   The frontmatter arm is the same depth: index_note_impl (8025) → extract_frontmatter_typed_links →
   emit_frontmatter_links (7361) → parse_link_body (7378). The outgoing-links arm is
   index_note_impl (7927) → extract_wikilinks (7068) → link_types::structural_frontmatter_targets
   (link_types.rs:385) → resolve_wikilink_type (link_types.rs:451). That last one ALREADY takes
   `reg: &LinkTypeRegistry` as its first parameter — the threading pattern exists, it just stops
   one frame short: `structural_frontmatter_targets` calls `snapshot()` itself (line 390) instead
   of accepting a registry. Threading `&LinkTypeRegistry` down these three arms is the whole of the
   parse-side work, and it is mechanical: no trait objects, no lifetimes beyond the call.

2. THE MAINTENANCE PASS IS A SEPARATE ARM AND THE HARNESS ALREADY KNOWS IT.
   `index_note` does NOT call maintain_incoming_after_save; `reindex_single_note` does, at
   search.rs:12754, and maintain_sky_after_save at 12765. federation/vocab_harness.rs:149-158 calls
   maintain_incoming_after_save explicitly with the comment "the maintenance pass, which index_note
   alone does not reach — and which is precisely what H1 is about". So a routed write that only
   re-keys index_note leaves the incoming aggregates computed with the wrong vocabulary. Any bundle
   must reach reindex_single_note's whole sequence, not just its middle.

3. THE ROUTED CALLER HAS TO SUPPLY THREE THINGS TODAY'S CALLERS GET AMBIENTLY.
   reindex_single_note takes `state: &SearchState` and derives everything else from the active
   universe: the connection (state.db), the vocabulary (the process-global), and the schema
   (already migrated by ensure_search_db_ready). A routed write to a child has none of those:
   db_path (1465) is ambient, link_types is ambient, and the child's schema is only guaranteed by
   the ForeignSchemaOnly path (migrate.rs:169) which deliberately does NOT create the vocabulary-
   dependent triggers (`if owns` at 5969, 5640, 5891, 5933). So a pooled child entry needs, at
   minimum: (a) the Owner root, (b) a Connection built by init_db against
   `<owner.root>/.constellation/search.db`, (c) that owner's LinkTypeRegistry read from
   `<owner.root>/.constellation/link-types.json`, and (d) a decision about the DDL — because the
   child's persisted triggers were created under whichever vocabulary last ran init_db as Active
   there, and PJ-232's whole point (documented at search.rs:4556-4589 and migrate.rs:142-167) is
   that the parent must not rewrite them.

4. THE VOCABULARY LOADER IS ALSO AMBIENT AND HAS NO PER-ROOT DOOR.
   `load_active(app)` (link_types.rs:522-524) = `set_active(read_deltas(app))`, and `read_deltas`
   (514-518) resolves its path through `link_types_path(app)` (507-509) →
   `universe::active_constellation_dir(app)`. There is no `read_deltas_at(root: &Path)`. Phase 1.2
   needs one — a pure `fn registry_for(root: &Path) -> LinkTypeRegistry` — and it is a three-line
   function, but it does not exist today, so nothing can construct a child's registry without
   temporarily making that child active. Precisely the design LL-047 rules out.

5. THE READ SIDE IS ALREADY ROUTED-ISH AND SHOULD NOT BE CONFUSED WITH THE WRITE SIDE.
   Federated READS go through `SearchState.federated_conn` with children ATTACHed `mode=ro`
   (attach.rs:229) — one connection, many schemas, no per-child vocabulary needed because the
   parent only SELECTs. Nothing in the ATTACH path can be reused for writes: a `mode=ro` schema
   rejects the write, and the aggregate columns the parent reads out of `cuN.note_meta` were
   computed by the CHILD's own process under the child's own vocabulary. The write pool is
   necessarily a second, separate mechanism from `federated_conn`.

6. TWO EXISTING WRITE PATHS ALREADY REACH A CHILD'S FILE AND WOULD BE THE POOL'S FIRST CUSTOMERS
   OR ITS FIRST CONFLICTS. federation_prewarm (search.rs:11381-11436) opens the child read-write
   and issues an FTS5 optimize; run_migrations_on (migrate.rs:169) opens it read-write for schema
   migration and drops the handle at 175 specifically so attach.rs:175's re-attach is not blocked.
   A pool that HOLDS a child connection open changes the second one's premise — migrate.rs:171-174
   says in as many words that the handle must be released before returning. Any pool needs to
   answer what happens when run_migrations_on wants the file the pool is holding.

7. THE GENERATION EPOCH IS THE ONLY EXISTING STALENESS MECHANISM AND IT IS ACTIVE-UNIVERSE-SHAPED.
   `federation_generation` (search.rs:1415) is bumped once per universe switch (11236) and every
   background worker compares it (11372, 11537, 11894, 12026). It answers "did the ACTIVE universe
   change", not "is THIS owner still linked". A child universe can be unlinked from
   `universe.json` without any switch, and nothing bumps the counter. A pool keyed by Owner needs
   its own invalidation, or an entry outlives its federation membership.

8. MEASURED SURFACE OF THE CHANGE, FOR SCOPING: `state.db.lock()` appears 146 times across 37
   files; `with_read_conn(` 13 times; `reindex_single_note(` 20 call sites (7 of them tests).
   Only the write half — reindex_single_note's 13 production callers plus the two bulk walkers
   (index_library_recursive at search.rs:12039, index_note_bulk at 8725) — is in scope for routing.
UNVERIFIED: Whether rusqlite 0.31's `Connection` is `Send` in this build. Cargo.toml:46 pins `rusqlite = { version = "0.31", features = ["bundled", "backup"] }`; I did not read rusqlite's own trait impls or the SQLITE_THREADSAFE compile setting. What I DID verify is that every Connection held in SearchState is wrapped in a Mutex (search.rs:1342, 1353, 1400) and that the read-only opener passes SQLITE_OPEN_NO_MUTEX (search.rs:1477). To settle: read the rusqlite 0.31 source under ~/.cargo/registry for `unsafe impl Send for Connection`. | Whether SearchState is the ONLY Tauri-managed state holding a Connection. I read lib.rs:358-371 (the nine `.manage()` lines) but did not open EmbeddingState, ScanState, RepairState, HealState, NscBackfillState or BulkAcceptState to check their fields. To settle: read each struct definition. | Whether `index_note_impl`'s span really ends at 8627. `awk` reported the first column-0 `}` after 7884 at line 8627; I read 7884-8030 and 8460-8580 but not 8030-8460 or 8580-8627 in full, so there could be additional vocabulary reads in those two gaps. The grep for `link_types::` over the whole file returned no hits inside them, which is evidence but not the same as having read the bodies. | How the index-repair runner obtains its Connection for the production bulk walk. I verified reconcile_filesystem's `walk_conn` (search.rs:11976) and that index_library_recursive is called with it at 12039, and that index_repair.rs:1018 is the `#[ignore]`d m1_full_reread_cost harness. I did NOT read index_repair.rs's `submit` / runner to confirm reconcile_filesystem_guarded is its only door. To settle: read index_repair.rs's runner and TriggerWindow. | Whether anything else in the app writes into a linked universe's `.md` files or its search.db beyond the three sites named (attach.rs read-only ATTACH, migrate.rs ForeignSchemaOnly + backup_database, search.rs federation_prewarm). I grepped `Connection::open` in federation/* and mig108.rs only. mig108.rs's opens (497-522, and 722/754/1486/1693/2526 in tests) are against the ACTIVE universe's own db_path in the unification backup, not a second universe's — verified for 497-522 only. To settle: grep `Connection::open` across all of src and classify each by whose path it resolves. | The exact set of `if owns` blocks and therefore exactly which DDL a ForeignSchemaOnly init does and does not write. I confirmed the guard lines (4838, 5640, 5891, 5933, 5969, 6219, 6284, 6312, 6335) and read the bodies at 5640 and 5969, and confirmed that the sky_link trigger block at 5531-5575 is NOT guarded. I did not read 5891, 5933, 6284 or 6312. To settle: read those four blocks.
---

## B. The four design options

## OPTION 1: Angle A — the vocabulary as a threaded parameter (`&LinkTypeRegistry`), constructed once at the frame that chooses the database
Every function that today asks `link_types` a question loses its ambient door and gains a `&LinkTypeRegistry` parameter; the value is built once, from the OWNER's `link-types.json`, at the exact frame that already decides which database is being written.

**Mechanism**: NO NEW TYPE IS STRICTLY NEEDED. `pub struct LinkTypeRegistry` already exists (link_types.rs:100-103), already derives `Clone`, and `pub fn resolve_wikilink_type(reg: &LinkTypeRegistry, …)` (link_types.rs:451-456) is the SHIPPED precedent for registry-as-parameter — the pattern stops exactly one frame short, because its caller `structural_frontmatter_targets` calls `snapshot()` itself at link_types.rs:390 instead of accepting a registry.

BORROWED, never owned, at every frame: `vocab: &LinkTypeRegistry`. Owned exactly once, in a `let` at the construction point, whose lifetime spans the routed operation. No `Arc`, no `Clone`-per-call, no `Send` question (the value never crosses a thread boundary that a `&Connection` does not already cross), no pool, no invalidation, no lifetime beyond the call.

This is strictly CHEAPER than today at runtime: `is_known_type` (link_types.rs:359-364) takes an `RwLock` read on EVERY call, and `parse_link_body` (search.rs:7244) calls it up to twice per wikilink; `structural_frontmatter_targets` clones the entire registry once per note (link_types.rs:390). Threading a borrow removes both.

ONE NEW FUNCTION, in link_types.rs, and I verified every piece it needs is already public:

    /// The vocabulary of the universe rooted at `root` — read from ITS
    /// `.constellation/link-types.json`, never from the process-global.
    pub fn registry_for_root(root: &Path) -> LinkTypeRegistry {
        let path = crate::universe::constellation_dir(root).join("link-types.json");
        let deltas = std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str::<Vec<LinkTypeDef>>(&s).ok())
            .unwrap_or_default();
        LinkTypeRegistry::merge(deltas)
    }

Verified possible: `universe::constellation_dir(universe_root: &Path) -> PathBuf` is `pub` at universe.rs:64 (root-parameterized already — `active_constellation_dir` at :69 is the ambient wrapper over it); `LinkTypeRegistry::merge` is `pub` at link_types.rs:115; `LinkTypeDef` derives `Deserialize` at link_types.rs:17. Today NO root-parameterized reader exists — `read_deltas(app)` (link_types.rs:514) goes through `link_types_path(app)` (:507-509) → `active_constellation_dir(app)`, which is ambient. This function is the missing door, and it is four lines.

WHERE IT IS CONSTRUCTED — the load-bearing rule: **the vocabulary is introduced at exactly the frame that chooses the database, and nowhere else.** Today those frames are enumerable and few:
  • `ensure_search_db_ready` — search.rs:11606-11607 already does `load_active(app)` immediately before `init_db(&path)`; that pairing becomes `let vocab = registry_for_root(&active_root); init_db(&path, &vocab)`.
  • `federation::migrate::run_migrations_on` — migrate.rs:169, which has `cu_db_path` and therefore the child's root two levels up.
  • `reindex_single_note` — search.rs:12682; it holds `note_path`, so it takes an `&Owner` (federation/owner.rs:58-73, Phase 1.1, currently ZERO production callers) and does `registry_for_root(&owner.root)` ONCE per save.
  • the four backfill runners, each of which already fixes a database path (incoming_links_backfill.rs:124 opens `db_path(app)` on its own connection).

HOW IT REACHES `index_note`: the parameter travels FIVE frames, which is the honest depth read off the chain —
  index_note (7873) → index_note_impl (7884) → extract_frontmatter_typed_links (7306) → emit_frontmatter_links (7361) → parse_link_body (7243) → `reg.is_known(id)`.
Two shallower arms: index_note_impl → extract_typed_links (7222) → parse_link_body (7243); and index_note_impl → extract_wikilinks (7068) → link_types::structural_frontmatter_targets (385) → resolve_wikilink_type (451, ALREADY takes `&LinkTypeRegistry`). Three reads are INLINE in index_note_impl's own body (8018, 8485, 8561) and become `vocab.is_structural(…)` — free, since that frame gains the parameter anyway.

I deliberately do NOT bundle `conn` and `vocab` into a context struct. `conn` and `&LinkTypeRegistry` are different types, so no call site can transpose them; and four of the six parse frames have NO connection in scope at all (extract_wikilinks, extract_typed_links, parse_link_body, emit_frontmatter_links, structural_frontmatter_targets are pure `&str → value`). Bundling would force a fake connection through pure text parsers. A bundle is a different angle; keeping them separate is what makes this one a clean comparison.

THE CLOSING MOVE, and it is what turns this from a convention into a structure: once the frames are converted, DELETE the free functions `link_types::is_known_type` (359), `is_structural_type` (369), and make `structural_frontmatter_targets` take the registry. Their bodies are already `LinkTypeRegistry` methods (`is_known` :171, `is_structural` :229) — the free functions exist only to read the global. With them gone, `snapshot()` (498) is the ONLY ambient door left, and it gets renamed to `active_universe_vocabulary()` so its ambience is in its name and a routed frame calling it is visible in review and greppable in CI.

**Signature changes**: Counts below are from greps run this session over `src-tauri/src`; production vs test separated by reading the `#[cfg(test)]` boundaries (search.rs boundaries at 491/598/884/975/1026/1109/1158/1971/2031/6676/6765/6902/7395/7506/…; libraries.rs at 7513; links_backfill.rs:485; incoming_links_backfill.rs:189; index_repair.rs:949).

TIER 1 — what the acceptance assertion actually requires (≈25 production, ≈50 test):

  link_types::structural_frontmatter_targets   link_types.rs:385
    (frontmatter: &str) -> HashSet<String>
    → (reg: &LinkTypeRegistry, frontmatter: &str) -> HashSet<String>
    Delete the `let reg = snapshot();` at :390. Prod sites: search.rs:7079, strata.rs:199, inspector360.rs:368 (3).

  extract_wikilinks         search.rs:7068  (content:&str)→(reg,content)          prod 1 (7927), test 2
  extract_typed_links       search.rs:7222  (content:&str)→(reg,content)          prod 1 (8016), test 1
  parse_link_body           search.rs:7243  (body:&str)→(reg,body)                prod 2 (7232, 7378), test 4
  extract_frontmatter_typed_links search.rs:7306 (content)→(reg,content)          prod 1 (8025), test 3
  emit_frontmatter_links    search.rs:7361  (wl,key,value,out,seen)→(reg,wl,…)    prod 2 (7341, 7350)

  index_note        search.rs:7873  (conn,note_path,library_name,force)
                                  → (conn, vocab:&LinkTypeRegistry, note_path, library_name, force)
                                    prod 3 (4280, 4339, 12718), test ~30
  index_note_bulk   search.rs:7880  same shape                                    prod 1 (8725), test 1
  index_note_impl   search.rs:7884  (conn,…,bulk) → (conn, vocab, …, bulk)
                                    + 3 inline rewrites at 8018 / 8485 / 8561

  incoming_aggregate_assignments  search.rs:2488
    (np:&str) -> String → (reg:&LinkTypeRegistry, np:&str) -> String
    Delete `let reg = snapshot();` at :2489. Prod 5: search.rs:2661, 11040, 12548; links_backfill.rs:307; name_fold_backfill.rs:157.

  maintain_incoming_after_save    search.rs:2637
    (conn, note_path, old_targets, old_name, old_aliases)
    → (conn, vocab, note_path, old_targets, old_name, old_aliases)      prod 1 (12754), test 4

  init_db / init_db_schema_only / init_db_scoped   search.rs:4592 / 4597 / 4601
    (path:&Path) / (path,scope:InitScope) → (path, vocab:&LinkTypeRegistry) / (path, scope, vocab)
    `init_db_scoped` CONSTRUCTS its own connection at :4603 and returns it owned at :6407, so there is no
    seam to hand it a pre-bound handle — the vocabulary must be a parameter here or it cannot arrive at all.
    Prod entries 2 (search.rs:11607; federation/migrate.rs:169). `init_db(` appears 44× repo-wide,
    `init_db_schema_only(` 6×, `init_db_scoped(` 3× — the remainder are test/harness fixtures.

  NEW  link_types::registry_for_root(root: &Path) -> LinkTypeRegistry

TIER 2 — required by the Whole-Ecosystem Fix Law in the same pass, not by the assertion (≈92 production):

  outgoing_aggregate_assignments  search.rs:2242  (src)→(reg,src)                prod 3 (2327, 2328; links_backfill.rs:248)
  create_outgoing_link_triggers   search.rs:2286  (conn)→(conn,vocab)            prod 5 (search.rs:2780, 5970; index_repair.rs:461, 473; mig108.rs:1203)
  stratum_sql_expr                search.rs:188   ()→(reg)                       prod 9 (search.rs:2719, 5840, 5910, 6289, 11054, 12560; links_backfill.rs:359; name_fold_backfill.rs:173; sky_backfill.rs:388)
  maturity_sql_expr               search.rs:266   ()→(reg)                       prod 9 (search.rs:2720, 5840, 5956, 6290, 11055, 12561; links_backfill.rs:360; name_fold_backfill.rs:178; sky_backfill.rs:399)
  maintain_sky_after_save         search.rs:2706  +vocab                         prod 1 (12765)
  reindex_delete_note             search.rs:12368 +vocab                         prod 6 (libraries.rs:8452, 8576, 8586; reconcile.rs:531; search.rs:12936, 13029)
  recompute_after_link_status_change search.rs:11023 +vocab                      prod 2 (10968, 11105)
  structured_search               search.rs:9550 read-side is_structural_type    prod 1
  links_backfill                  is_needed:99, recompute_range:248, recompute_incoming_range:307,
                                  recompute_sky_range:359, recompute_all_{outgoing,incoming,sky}   ≈8
  incoming_links_backfill         is_stamped:49, is_built:73, run:149/173                          ≈7
  sky_backfill                    process_batch:283                                                 1
  name_fold_backfill              run:157/173/178                                                   1
  converge::converge_derived_views converge.rs:229                                                 5 callers
  READ SIDE (cannot be skipped — see below)  cache.rs:516/548/1288, sight.rs:77, tension.rs:277,
                                  strata.rs:168/199/208, inspector360.rs:284/343/368/375,
                                  libraries.rs:4040/4065                                          ≈14
  libraries.rs::rewrite_wikilinks_in_text  libraries.rs:7478 (is_known_type at :7490) — the ONE
                                  vocabulary read in the tree whose answer reaches a `.md` FILE, via
                                  `gate_rmw` → `atomic_write` (write_gate.rs:667)                   ≈2

  reindex_single_note   search.rs:12682
    (state:&SearchState, note_path, library_name)
    → (state:&SearchState, owner:&federation::owner::Owner, note_path, library_name)
    prod 16: bases.rs:437; index_repair.rs:853; libraries.rs:1450, 1890, 1967, 2662, 2811, 7071;
    reconcile.rs:469, 565; search.rs:12275, 12861, 13010; shape.rs:214; tasks.rs:540; universe.rs:2488.
    It takes `&SearchState` and has NO `AppHandle`, so it cannot call `resolve_owner(app, path)` itself —
    the Owner has to arrive as a parameter, which is what finally wires Phase 1.1 into production.

  DELETIONS that close the ambient doors: link_types::is_known_type (359), is_structural_type (369);
  rename snapshot() (498) → active_universe_vocabulary().

A CONSEQUENCE I HAVE TO NAME BECAUSE IT IS NOT OPTIONAL: this design BREAKS
`a_vocabulary_swap_reaches_back_into_an_already_open_database` (vocab_harness.rs:227-256) BY DESIGN.
That test drives `index_note(&conn, &p, "harness", true)` directly and asserts the mid-flight
`set_active` DID reach the write. With the vocabulary as a parameter it no longer can, so the
assertion at :249-255 fails. Its own doc-comment says "If this ever fails, the coupling changed and
1.2's design premise must be re-checked" — that is exactly what happens, and the test must be
re-authored in the SAME commit into its positive form: *a swap must NOT reach a routed write*.
`index_under_vocabulary` (:135-160) likewise stops calling `set_active` and threads
`LinkTypeRegistry::merge(vocabulary)` instead.
**Call sites touched**: 215
**Speed**: Tier 1 alone — half a day to a day of wall clock. It is 25 production edits and ~50 test edits confined to four files (search.rs, link_types.rs, strata.rs, inspector360.rs), and the compiler drives every one of them: change the leaf signature, compile, fix the errors it names, repeat. There is no design left to discover — the chain is mapped and I read every frame in it.

Tier 2 — two to three days honestly. Not because any single edit is hard, but because ~92 more production sites span 12 files, `reindex_single_note`'s 16 callers each have to acquire an `&Owner` (which means `resolve_owner` gets its first production callers and its own failure mode has to be handled at 16 sites — it is fail-closed and returns `Err`), and the four backfill schedulers each need a decision about which root they are running against. Add the full Rust suite between tiers.

The honest total for "Phase 1.2's vocabulary half, whole-ecosystem": 3 days. The honest total for "the ignore attribute comes off": 1 day. Those are different deliverables and the gap between them is the entire risk of this option.
**Effort**: Mechanically low, judgementally moderate, and the work is almost entirely compile-error-driven rather than design-driven.

Cheap because: no new type to keep in sync; no pool; no lifetime that outlives a call; no `Send`/`Sync` question (rusqlite is pinned at 0.31 with `bundled` + `backup` and no serialize feature, and nothing here crosses a thread the `&Connection` does not already cross); no invalidation story; nothing persisted; instantly reversible commit-by-commit. `resolve_wikilink_type` already proves the pattern compiles in this codebase.

Expensive because: the parameter is a NEW ARGUMENT on nine hot functions, so every touch is a merge-conflict surface against any other work in search.rs, and search.rs is 16,900 lines that everything else in the migration also edits. And because Tier 1 is genuinely tempting to ship alone — it is one day, it turns the test green, and it leaves ~92 unconverted sites reading the global exactly as today.

One real discipline cost with a measurable failure mode: `registry_for_root` reads a file. It MUST be hoisted to the frame that chooses the database and never called inside a loop. `index_library_recursive` (search.rs:8682) walks 7,820 notes on the Boss's universe through `index_note_bulk` at :8725 — a per-note construction there is 7,820 file reads added to a repair, and it would not fail, it would just be slow, which is the kind of regression this project has a standing rule against.
**Risk**: LOUD where converted; SILENT where missed. That asymmetry IS the risk, and it is the whole story.

Loud half: every converted frame becomes a compile error until a vocabulary is supplied. A missing argument cannot ship. A transposed argument cannot ship (`&Connection` and `&LinkTypeRegistry` are different types). This is the strongest property Angle A has.

Silent half: a frame NOT converted keeps calling `snapshot()` / `is_known_type` and behaves exactly as today — same values, same row counts, no error, nothing on screen. A partial conversion reads green in `cargo test` AND green in the acceptance harness, because the harness observes only four things (vocab_harness.rs:81-103: `COUNT(*) FROM note_links`, the `(source, target, link_type)` tuples, `note_meta.incoming_count`, `note_meta.incoming_link_types`) and never looks at `sky_nodes`, `note_meta.outgoing_*`, or any earned Living-Link column. Convert `parse_link_body` and `incoming_aggregate_assignments` and the test passes with 90 sites still ambient. That is precisely the Phase 0.4 failure the owner.rs header names — "proving a property over the part you happened to look at."

The cure exists and is inside this option: DELETE `is_known_type` and `is_structural_type` and rename `snapshot()`. Then every un-converted reader is a compile error, and the silent half collapses to zero. Angle A is only as safe as that deletion — if the deletion is deferred "for now," the option ships its own worst failure mode.

Second risk, distinct: **the wrong registry compiles.** `index_note(&conn, &link_types::snapshot(), path, lib, true)` type-checks and reproduces today's bug byte for byte. The parameter makes the choice VISIBLE and greppable; it does not make it CORRECT. See the last field.

Third risk, verified, and it would surface mid-build if not planned for: `a_vocabulary_swap_reaches_back_into_an_already_open_database` goes red on purpose. That is a red test in the tree unless it is re-authored in the same commit.
**Invariants at risk**: ONE PARSER, ONE ANSWER (link_types.rs:380-384, the MIG-067 'ONE shared implementation … so they can never drift from the indexed note_links again' rule). The parse chain is shared by index_note AND by three filesystem re-walkers that never touch index_note — strata.rs:168/199/208, inspector360.rs:343/368/375, libraries.rs:4040/4065. Converting only search.rs's chain makes those three disagree with the index about the same note, which is the exact drift MIG-067 was written to end.; PJ-065 EARNED-LINK PRESERVATION (search.rs:8485 → link_row_is_preserved, search.rs:477-489). The `structural` argument decides whether an existing edge's weight / confidence / traversal_count / archived status survives a rebuild. CLAUDE.md states this data lives ONLY in search.db — there is no disk layer. A wrong vocabulary here silently resets earned Living-Link data, and the harness cannot see it: `aggregates_for` selects only (source_path, target_name, link_type) at vocab_harness.rs:83-86.; ROW-COUNT vs VALUE symmetry. search.rs:8018 is the one vocabulary read on the index chain that changes the row COUNT (a body-authored structural edge produces no note_links row at all). Everything else changes only values. So a mistake at 8018 is the ONE thing `link_rows` would catch, and a mistake anywhere else is exactly the invisible class this harness exists for.; TRIGGER-BODY ↔ LIVE-SQL AGREEMENT. `outgoing_aggregate_assignments` (search.rs:2242) generates BOTH the persisted trigger bodies (interpolated at 2327-2328 by create_outgoing_link_triggers) and a live windowed UPDATE (links_backfill.rs:248). Hand those two different registries and the trigger and the backfill each rewrite the same four columns to different values, permanently, with every row count correct.; RANK ↔ SENTINEL COHERENCE. search.rs:2248-2251 and 2493-2496 each read sql_in_list_cognitive / sql_rank_case_cognitive / cognitive_sentinel_rank / structural_not_in_clause from ONE `reg` binding. `cognitive_sentinel_rank` is `cognitive_ids().len() + 1` (link_types.rs:292). If the threaded version derives rank from one registry and sentinel from another, `outgoing_top_rank` can exceed its own no-links sentinel and every rank-ordered surface mis-sorts.; FINGERPRINT ↔ ROWS AGREEMENT. links_backfill.rs:99 and incoming_links_backfill.rs:49 compare a fingerprint STORED IN A DATABASE against one read from the PROCESS-GLOBAL; the stamps are written at links_backfill.rs:464 and incoming_links_backfill.rs:173. Threading changes which vocabulary computed the rows — the stamp writer must be handed the SAME registry, or the gate lies in both directions and, per incoming_links_backfill.rs's own doc, an unstamped answer also flips READERS onto a different code path.; BOOT AND WALK COST. `registry_for_root` is a file read. Hoisted, it is one read per routed operation; dropped into index_library_recursive's per-note loop (search.rs:8725) it is 7,820 reads on the Boss's universe. Nothing fails — it just gets slower, which is the regression class CLAUDE.md's hard constraint forbids.; THE ONE WRITER THAT REACHES DISK. libraries.rs:7490 (`is_known_type` inside rewrite_wikilinks_in_text) decides what text is written into a `.md` file through gate_rmw → atomic_write (write_gate.rs:667). It is fenced from child universes today by the `foreign` set (libraries.rs:6963 and 6982-6986). If Phase 1.2 routes renames, that fence is the thing being removed, and this site's vocabulary must arrive with the route or the cascade rewrites a child's notes under the parent's grammar.
**Makes the test pass by**: `routed_write_must_match_the_owners_vocabulary` (vocab_harness.rs:276-289) would replace its `panic!` with a routed write that names its vocabulary at every call:

    // The child's OWN vocabulary, on disk where the child keeps it.
    let child_root = tmp_dir("routed_child");
    std::fs::create_dir_all(child_root.join(".constellation")).unwrap();
    std::fs::write(
        child_root.join(".constellation").join("link-types.json"),
        serde_json::to_string(&deltas(&["refutes"])).unwrap(),
    ).unwrap();

    // The PARENT is active, and STAYS active for the whole routed write.
    crate::link_types::set_active(deltas(&["exemplifies"]));

    // The Router's two answers, both derived from the OWNER — never from the global.
    let owner = crate::federation::owner::resolve_owner_in(
        &child_root.join("Source.md").to_string_lossy(),
        &parent_root, &[child_root.clone()]).unwrap();
    let vocab = crate::link_types::registry_for_root(&owner.root);
    let conn  = crate::search::init_db(&child_root.join("search.db"), &vocab).unwrap();

    for (name, body) in NOTES {
        let p = child_root.join(name);
        std::fs::write(&p, body).unwrap();
        crate::search::index_note(&conn, &vocab, &p.to_string_lossy(), "harness", true).unwrap();
    }
    let empty = std::collections::HashSet::new();
    for (name, _) in NOTES {
        crate::search::maintain_incoming_after_save(
            &conn, &vocab, &child_root.join(name).to_string_lossy(), &empty, "", &empty).unwrap();
    }
    let routed = aggregates_for(&conn, &child_root).unwrap();
    assert_eq!(routed, expected, "a routed write must use the OWNER's vocabulary");
    assert_ne!(routed, wrong,     "and never the active universe's");

WHY IT PASSES, read off the two observed quantities:

`edges` — the harness note is `[[refutes::Target|because of X]]` (vocab_harness.rs:181), body text with no frontmatter. It reaches `extract_typed_links` (8016) → `parse_link_body` (7232). Today `parse_link_body` asks the global at 7244; with the parent active, `refutes` is unknown, the predicate-first branch at 7247-7249 falls through, and the link collapses at 7278 to `("associative", "refutes::target", …)`. Threaded, the same line asks `vocab.is_known("refutes")` against the CHILD's registry, returns true, and stores `link_type='refutes', target='target'` — byte-identical to `expected`.

`incoming_counts` / `incoming_types` — `maintain_incoming_after_save` builds `UPDATE note_meta SET {incoming_aggregate_assignments("note_meta")}` at search.rs:2659-2661. Today :2489 snapshots the global, so `sql_in_list_cognitive` omits `refutes` and `incoming_link_types` for Target comes back empty. Threaded, the IN-list contains `refutes` and the column reads `refutes (1)` — again identical to `expected`.

AND CRUCIALLY: `set_active(parent)` may be called at ANY point inside that block — including between `init_db` and `index_note`, the exact swap `a_vocabulary_swap_reaches_back_into_an_already_open_database` pins — and `routed` is unchanged, because no frame on the converted path reads the global at all. That is the property LL-047 asks for, obtained without a window because there is no window: nothing shared is mutated for a duration.

Two honest caveats about this pass. First, the harness note has no frontmatter, so `extract_frontmatter_typed_links`, `emit_frontmatter_links` and `structural_frontmatter_targets` are not exercised by it — threading only `parse_link_body` and `incoming_aggregate_assignments` would ALSO turn it green, and that would be gaming the test. Second, `init_db` here still bakes `note_links_outgoing_*` and `note_meta_sky_*` trigger bodies, which fire during `index_note` and write `note_meta.outgoing_*` and `sky_nodes.stratum/maturity` — none of which `aggregates_for` reads. Passing this test does not prove those are right.
**Cannot do**: 1. IT CANNOT FIX SQL THAT IS ALREADY FROZEN IN THE CHILD'S `sqlite_master`. `note_links_outgoing_ai/_ad/_au` carry `outgoing_aggregate_assignments` baked in at search.rs:2327-2328, and `note_meta_sky_ai` carries `stratum_sql_expr`/`maturity_sql_expr` baked in at :5840. `index_note`'s writes to note_links (8531/8557/8567/8579/8601) FIRE those triggers. Threading a vocabulary through the call changes `note_links.link_type`; it changes nothing about `note_meta.outgoing_count / outgoing_link_types / outgoing_top_rank`, which come out of text already stored in that file. Angle A can only ensure the DDL is GENERATED from the right registry at the moment of creation — it does not answer WHEN a parent may create a child's triggers, and PJ-232's current answer is "never" (`if owns` at search.rs:5969). That ruling is a separate decision this option does not make.

2. IT CANNOT REACH A FRAME THAT WAS NOT CONVERTED. Coverage equals exactly the set of signatures changed. `list_link_types` (link_types.rs:585-589) still calls `set_active` on an ordinary editor open — merely OPENING the Links editor mutates the global mid-flight — and every un-threaded reader still sees it.

3. IT CANNOT BE RECOVERED FROM A CONNECTION. A future function handed a `&Connection` and no vocabulary has no way to ask which universe it belongs to. Every new code path is a fresh opportunity to forget, forever, and the only guard is review.

4. IT CANNOT ROUTE THE CONNECTION. This is the vocabulary half only. If `reindex_single_note` keeps taking its connection from `state.db` (search.rs:12688-12689 — the ACTIVE universe's), a routed write computes the CHILD's values and writes them into the PARENT's database. The two halves must land together or the result is worse than today.

5. IT CANNOT REPAIR ALREADY-CORRUPTED ROWS, and the fingerprint gate will not notice them. If a routed write lands parent-flavoured values in a child through the per-note path without running that child's backfill, the child's `links_vocab` stamp is still its OWN fingerprint, so `is_needed` (links_backfill.rs:99) evaluates FALSE on the child's next boot and the rows are never healed. Sky is worse: there is no `sky_vocab` stamp anywhere in the tree, and `sky_backfill::is_needed` (sky_backfill.rs:89) reads only `schema_versions.sky`, so once stamped it can never re-run.

6. IT DOES NOT PAY FOR ITSELF ON THREE OF THE SIX INDEX-CHAIN READS. `structural_ids()` is provably `{contains, parent}` in every universe — `merge` seeds from `seeds()` (link_types.rs:121), forces `structural=false` for cognitive-seed deltas (:131), `structural=true` only for `STRUCTURAL_SEED_IDS` (:148), and `structural=false` for every custom type (:159). So `is_structural_type` at 8018/8485/8561, `stratum_sql_expr` (189) and `maturity_sql_expr` (267) currently return the same answer under any two vocabularies. Threading them buys nothing TODAY. It must be done anyway, because that invariance is a property of `merge`'s body and not a contract anyone declared — and if it is leaned on, it needs a test that goes red the day `merge` stops forcing `structural = false`.
**Cannot-forget check**: IN ITS BARE FORM, IT DOES NOT SATISFY LL-048 RULE 2. I should say that plainly rather than dress it up.

`index_note(&conn, &link_types::snapshot(), path, lib, true)` type-checks, compiles, and reproduces today's bug byte for byte. The compiler forces the caller to SUPPLY a vocabulary; it does not force them to supply the OWNER's. That is a promise a caller must keep, expressed as a parameter — which is better than a promise nobody can see, but it is the same category of thing. Sixteen `reindex_single_note` call sites each writing `&link_types::snapshot()` would be sixteen places a future router-aware change must remember to revisit, and this migration has already been burned twice by exactly that shape: the §0.4 base guard that could never fire, and `resolve_owner`'s own wrapper, where nine green tests sat over a caller that returned the inverted answer.

What Angle A DOES buy, honestly stated: it converts an INVISIBLE ambient read, five frames deep inside a text parser with no connection in scope, into a VISIBLE argument at the call site, greppable in one command and reviewable in a diff. And it makes the omission loud: a converted frame will not compile without one.

THE COMPLETION THAT DOES SATISFY RULE 2, and it costs one tuple struct:

    /// A vocabulary that can only have come from a resolved owner.
    pub struct OwnerVocabulary(LinkTypeRegistry);
    impl OwnerVocabulary {
        /// The ONLY constructor. No `From<LinkTypeRegistry>`, no `from_active()`,
        /// no path in from `snapshot()`.
        pub fn of(owner: &crate::federation::owner::Owner) -> Self {
            Self(registry_for_root(&owner.root))
        }
        pub fn reg(&self) -> &LinkTypeRegistry { &self.0 }
    }

Thread `&OwnerVocabulary` instead of `&LinkTypeRegistry` at every frame that writes. Now `&link_types::snapshot()` does not type-check anywhere on the write path, and the ONLY way to obtain one is to have already resolved an `Owner` — which, per federation/owner.rs:111-141, is fail-closed and returns `Err` for any path not under `{active} ∪ {federation}`. The type carries the proof. `Owner` is already Phase 1.1's shipped type and already normalised to one identity per universe (owner.rs:59-68, :123), so there is no second key and no second lock.

Two holes remain and I will not pretend otherwise. Tests need a construction door, so a `#[cfg(test)] pub fn for_test(reg: LinkTypeRegistry)` exists — cfg-gated, but a door. And the deletion of `is_known_type` / `is_structural_type` (link_types.rs:359, 369) is what stops an un-converted frame from quietly reading the global forever; without that deletion, the newtype guards the frames you converted and nothing else. Angle A is exactly as safe as that deletion, and the deletion is the cheapest part of the whole option.

---

## OPTION 2: ANGLE B — `Db<'a>`: the connection and its vocabulary as ONE value, with no `Deref`
Introduce a two-field borrowed value `Db<'a> { conn: &'a Connection, vocab: &'a LinkTypeRegistry }`, make every vocabulary-reading write-path function take it instead of `&Connection`, delete the free-function global readers so the compiler names every remaining site, and give the value exactly two producers — one that reads the process-global (active universe) and one that reads the owner's `link-types.json` (routed) — so no caller ever supplies a vocabulary by hand.

**Mechanism**: THE TYPE (new module, e.g. `src-tauri/src/db_ctx.rs`; name `Db` is free — a repo-wide grep for `struct Db`/`enum Db` returns nothing, and `Ctx` is taken by `converge.rs:193`):

    #[derive(Clone, Copy)]
    pub struct Db<'a> { conn: &'a Connection, vocab: &'a LinkTypeRegistry }
    impl<'a> Db<'a> {
        pub(crate) fn conn(&self)  -> &'a Connection        { self.conn }
        pub(crate) fn vocab(&self) -> &'a LinkTypeRegistry  { self.vocab }
    }
    // Deliberately NO `impl Deref for Db`. See `why_a_caller_cannot_get_it_wrong`.

`Copy` matters: `index_note(db, ...)` forwards `db` to `index_note_impl(db, ...)` with no borrow gymnastics, and inside a body `let conn = db.conn();` on line 1 leaves the remaining ~700 lines of `conn.query_row/prepare/execute` byte-untouched. That one-line prologue is the whole per-function migration cost.

TWO PRODUCERS, NEITHER TAKING A CALLER-SUPPLIED VOCABULARY.

(1) Active universe — a scoped builder, so the vocabulary is never a parameter anyone writes:

    pub(crate) fn with_active_db<T>(conn: &Connection, f: impl FnOnce(Db<'_>) -> T) -> T {
        let vocab = crate::link_types::active_universe_snapshot(); // today's `snapshot()`, renamed
        f(Db { conn, vocab: &vocab })
    }

This is honest because the process-global genuinely IS the active universe's vocabulary: `link_types::REGISTRY` (link_types.rs:351) is written only by `set_active` (link_types.rs:481), whose production callers are `load_active` (link_types.rs:522-524, path via `link_types_path` → `active_constellation_dir`, link_types.rs:507-509), `save_universe_link_types` (link_types.rs:554) and `list_link_types` (link_types.rs:588) — all three the ACTIVE universe's file. The bug was never that the global holds the wrong thing; it is that non-active writes read it.

(2) Routed owner — an owned handle whose only input is an `Owner`:

    pub struct UniverseDb { root: PathBuf, vocab: LinkTypeRegistry, conn: Connection }
    impl UniverseDb {
        pub fn open(owner: &crate::federation::owner::Owner) -> Result<Self, String>;
        pub fn db(&self) -> Db<'_> { Db { conn: &self.conn, vocab: &self.vocab } }
    }

`open` does three things and takes no vocabulary argument:
  a. refuses `owner.is_active` (that path is `with_active_db` over `SearchState.db`, so one universe never gets two writer connections);
  b. `vocab = LinkTypeRegistry::merge(read the owner's deltas)` via a new 4-line `registry_for_root(root)` = `universe::read_persisted_json::<Vec<LinkTypeDef>>(universe::constellation_dir(root).join("link-types.json"))`. `constellation_dir(universe_root)` already exists as a pure root→path helper (universe.rs:64-66). **STRICT, not lenient** — `read_persisted_json` (universe.rs:260-289) returns `Ok(None)` only for NotFound, and `Err` for unreadable/empty/corrupt. That is the required polarity: `load_active`'s lenient fallback to the 8 seeds is right at boot (link_types.rs:511-513) and is exactly the wrong-vocabulary write when routed. `list_link_types` already made this same strict/lenient split for the same reason (link_types.rs:568-589);
  c. `conn = init_db_scoped(db_path, scope, &vocab)`.

HOW IT REACHES `index_note`. The vocabulary rides `Db` down to the connection layer and then travels ALONE below it, because four of the readers are pure text parsers with no connection:

  reindex_single_note (search.rs:12682; `conn` from `state.db.lock()` at :12688-12689)
    → with_active_db(conn, |db| ...)
      → index_note(db, path, lib, true)                 [search.rs:12718]
        → index_note_impl(db, ...)                       [search.rs:7884]
           let conn = db.conn(); let v = db.vocab();
           ├ extract_wikilinks(v, &content)              [call at :7927]
           │   └ link_types::structural_frontmatter_targets(v, fm)   [:7079 → link_types.rs:385, whose own `snapshot()` at :390 is deleted]
           ├ extract_typed_links(v, &body)               [:8016]
           │   └ parse_link_body(v, body) → v.is_known(s)  [:7232 → :7244; `LinkTypeRegistry::is_known` already exists, link_types.rs:171]
           ├ v.is_structural(&l.link_type)               [:8018 — drops body-authored structural edges]
           ├ extract_frontmatter_typed_links(v, &content)[:8025]
           │   └ emit_frontmatter_links(v, wl, key, ...) [:7341, :7350 → v.is_known(key) at :7371]
           │        └ parse_link_body(v, body)           [:7378]
           ├ v.is_structural(&ltype) → link_row_is_preserved(..)  [:8485 — decides whether earned weight/confidence/traversal survive]
           └ v.is_structural(link_type) → which INSERT shape       [:8561]
      → maintain_incoming_after_save(db, ...)            [:12754]
           → incoming_aggregate_assignments(db.vocab(), "note_meta")  [:2661 → the generator at :2488, whose `snapshot()` at :2489 is deleted]
      → maintain_sky_after_save(db, ...)                 [:12765]
           → stratum_sql_expr(db.vocab()) / maturity_sql_expr(db.vocab())  [:2719-2720 → generators at :188/:266, `snapshot()` at :189/:267 deleted]

DDL. `init_db_scoped(path: &Path, scope: InitScope)` (search.rs:4601) opens its own connection at :4603, so the bundle cannot be handed IN — the vocabulary must be a third parameter: `init_db_scoped(path, scope, vocab: &LinkTypeRegistry)`. `init_db(path)` (search.rs:4592) keeps its signature by passing `&active_universe_snapshot()`, which leaves ~35 test callers untouched. That single parameter fixes all five registry-generated DDL sites at their source: `create_outgoing_link_triggers` (search.rs:2286, bodies at :2327-2328), `note_meta_sky_ai` (:5840), `note_meta_sky_stratum_au` (:5910), `note_meta_sky_maturity_au` (:5956), and the sky_link family at :5540 — which I confirmed by reading search.rs:5531-5575 is at the function's top-level indentation, inside no `if owns` block, so `init_db_schema_only` (search.rs:4597) called from federation/migrate.rs:169 writes the PARENT's `structural_not_in_clause("NEW.link_type")` into a CHILD's `note_links_sky_ai`/`_au` bodies today. Once the vocabulary is a parameter, that site is correct by construction rather than by a gate someone remembered.

PERFORMANCE — this is a net WIN on the save path, not a cost. Today `parse_link_body` takes an RwLock read per wikilink (link_types.rs:360-363 via :7244), `structural_frontmatter_targets` clones the whole registry per note (link_types.rs:390), and both aggregate generators clone per call (search.rs:2243, :2489). Threading `&LinkTypeRegistry` collapses all of that to ONE lock+clone per `with_active_db`. Relevant because CLAUDE.md forbids any change that regresses typing latency or IPC responsiveness.

**Signature changes**: Counts below are from greps run this session over `src-tauri/src`, excluding definition lines and comment-only lines. **~72 production + ~65 test ≈ 137 compile errors.**

A. `&Connection` → `Db<'_>` (the bundle):
  • `index_note` / `index_note_bulk` / `index_note_impl` — search.rs:7873 / :7880 / :7884.
    before `fn index_note(conn: &Connection, note_path: &str, library_name: &str, force: bool)`
    after  `fn index_note(db: Db<'_>, note_path: &str, library_name: &str, force: bool)`
    Call sites: 4 production (search.rs:4280, :4339, :8725, :12718) + ~35 test in search.rs + libraries.rs:7845, :7887 + index_repair.rs:1018 + vocab_harness.rs:146, :245.
  • `maintain_incoming_after_save` — search.rs:2637. 1 production (search.rs:12754) + search.rs:2089, :2099, :2182 + vocab_harness.rs:156.
  • `maintain_sky_after_save` — search.rs:2706. 1 production (search.rs:12765).
  • `create_outgoing_link_triggers` — search.rs:2286. 5 production (search.rs:2780, :5970; index_repair.rs:461, :473; mig108.rs:1203) + 2 test (search.rs:6874, :6892).

B. `-> String` generators gain `vocab: &LinkTypeRegistry` as the FIRST parameter (they have no connection and never will):
  • `stratum_sql_expr()` → `(vocab)` — search.rs:188. 9 production: search.rs:2719, :5840, :5910, :6289, :11054, :12560; links_backfill.rs:359; sky_backfill.rs:388; name_fold_backfill.rs:173.
  • `maturity_sql_expr()` → `(vocab)` — search.rs:266. 9 production (search.rs:2720, :5840, :5956, :6290, :11055, :12561; links_backfill.rs:360; sky_backfill.rs:399; name_fold_backfill.rs:178) + 1 test (search.rs:1090).
  • `outgoing_aggregate_assignments(src)` → `(vocab, src)` — search.rs:2242. 3 production (search.rs:2327, :2328; links_backfill.rs:248) + 4 test (search.rs:6796, :6797 — inside the `#[cfg(test)]` module beginning at search.rs:6765; links_backfill.rs:739, :740).
  • `incoming_aggregate_assignments(np)` → `(vocab, np)` — search.rs:2488. 6: search.rs:2661, :11040, :12548; links_backfill.rs:307; name_fold_backfill.rs:157; incoming_links_backfill.rs:363.

C. Pure parsers gain `vocab: &LinkTypeRegistry`:
  • `extract_wikilinks(content)` — search.rs:7068. 1 prod (:7927) + 2 test (:15652, :15659).
  • `extract_typed_links(content)` — search.rs:7222. 1 prod (:8016) + 1 test (:7572).
  • `parse_link_body(body)` — search.rs:7243. 2 prod (:7232, :7378) + 4 test (:7515, :7564, :7565, :7566).
  • `extract_frontmatter_typed_links(content)` — search.rs:7306. 1 prod (:8025) + 3 test (:7403, :7480, :7486).
  • `emit_frontmatter_links(wl, key, value, out, seen)` — search.rs:7361. 2 prod (:7341, :7350).
  • `link_types::structural_frontmatter_targets(fm)` → `(vocab, fm)` — link_types.rs:385, deleting its internal `snapshot()` at :390. 3 prod: search.rs:7079, strata.rs:199, inspector360.rs:368.

D. `init_db_scoped(path, scope)` → `(path, scope, vocab)` — search.rs:4601. 3 sites: search.rs:4593, :4598, federation/migrate.rs:169 (+4 test callers of `init_db_schema_only` in migrate.rs:618, :655, :733, :752). `init_db(path)` keeps its shape, so its ~35 remaining callers are untouched.

E. **The compiler-forcing half — delete/rename the free-function global readers.** `LinkTypeRegistry` already has the method form of each, so every site is a mechanical `X(s)` → `v.method(s)`:
  • delete `link_types::is_known_type` (:359) → `LinkTypeRegistry::is_known` (:171). 3 prod: search.rs:7244, :7371; libraries.rs:7490.
  • delete `link_types::is_structural_type` (:369) → `LinkTypeRegistry::is_structural` (:229). 4 prod: search.rs:8018, :8485, :8561, :9550.
  • rename `link_types::snapshot()` (:498) → `active_universe_snapshot()`, documented as "the ACTIVE universe's vocabulary — correct only when the target of this computation is the active universe; a routed write takes its vocabulary from its `Db`." 14 remaining production sites must each answer *whose vocabulary is this?*: cache.rs:516, :548, :1288; incoming_links_backfill.rs:49, :149; inspector360.rs:284, :343; libraries.rs:4040; links_backfill.rs:99, :160; sight.rs:77; sky_backfill.rs:283; strata.rs:168; tension.rs:277.
  • `link_types::is_null_type` (:493) is NOT touched — I read it: `matches!(id, "associative" | "relates" | "")`, a constant match that never reads `REGISTRY`. sight.rs:113 is a false positive in the "26 call sites" count in vocab_harness.rs:6.

NOT changed: `SearchState.db: Mutex<Option<Connection>>` stays a bare `Connection`. Wrapping it would turn all 146 `.db.lock()` sites (grep count this session) into compile errors for no gain, since the active universe's vocabulary is available from the global at the ~15 boundaries that actually need it.
**Call sites touched**: 137
**Speed**: Design: done — it is written above and every hop is verified against source. Build: the edit is wide but almost entirely mechanical (add a parameter, add `let conn = db.conn();`, `X(s)` → `v.method(s)`), and the compiler drives it — you fix errors until it stops. Realistically **one focused working day, 6–9 hours**, of which the design and the harness rewrite are ~1.5 h and the rest is compile-fix cycles across search.rs (16k+ lines), links_backfill.rs, incoming_links_backfill.rs, sky_backfill.rs, name_fold_backfill.rs, cache.rs, strata.rs, inspector360.rs, libraries.rs, tension.rs, sight.rs, index_repair.rs, mig108.rs, federation/migrate.rs. **UNVERIFIED: this crate's actual `cargo build` wall-clock — I did not run a build this session.** `rusqlite` is pinned with `features = ["bundled"]`, so a full rebuild compiles SQLite from C; if the incremental cycle on search.rs is minutes rather than seconds, the honest figure moves toward the top of that range or past it. To settle: time one `cargo check` after touching search.rs.
**Effort**: Wide, shallow, and compiler-guided — the opposite shape from the risk profile that usually accompanies a number like 137. There is no new algorithm, no schema change, no lifetime puzzle beyond `Db<'a>` being `Copy`. The two genuinely thoughtful pieces are small: `registry_for_root` (4 lines, strict-read polarity) and the `InitScope`/vocabulary decision for a routed open. Everything else is typing. The cost is concentration and build cycles, not difficulty — but it is NOT a change that can be half-landed and left overnight: between the first signature change and the last compile error the tree does not build, so it is one sitting or a branch.
**Risk**: **The migration itself breaks LOUDLY.** Every one of the ~137 sites is a type error or an unresolved-name error. Nothing about this change can compile-and-be-wrong in the direction of "I forgot to migrate a call site" — that is the entire point of choosing a type change over a convention.

**Four things stay SILENT, and they are what the risk actually is:**

1. **A function you leave taking `&Connection`, reached via `db.conn()`.** It compiles, and it reads the global. The `.conn()` token is the audit list — greppable, countable, and each occurrence is a place someone decided the vocabulary does not matter. Silent, but marked. (This is the exact hazard `Deref` would erase, which is why the design refuses `Deref`.)

2. **Trigger bodies already sitting in an existing child's `sqlite_master`.** Threading fixes what the NEXT `init_db_scoped` writes; it audits nothing already there. Verified: `create_outgoing_link_triggers` is skipped for a foreign database (`if owns`, search.rs:5969) so a child keeps whatever its own owner last wrote — but the sky_link family at search.rs:5531-5575 is ungated, so any child ever schema-migrated through federation/migrate.rs:169 is carrying the parent's exclusion clause right now. Silent, permanent until re-created, and invisible to row counts. Angle B is what makes the repair *possible* (init now has the child's vocabulary) but does not perform it.

3. **A deliberate mispairing** — `with_active_db(routed.db().conn(), ...)`. Expressible in one line. No type short of newtyping `Connection` across the whole crate prevents it.

4. **The acceptance test going green while the routed write is still wrong elsewhere.** `aggregates_for` (vocab_harness.rs:73-106) reads exactly four things: `COUNT(*) FROM note_links`, `(source_path, target_name, link_type)`, `note_meta.incoming_count`, `note_meta.incoming_link_types`. It never reads `outgoing_*`, never reads `sky_nodes.stratum/maturity`, and its `edges` tuple omits `weight`/`confidence`/`traversal_count` — the columns the vocabulary decides at search.rs:8485 (the `structural` argument to `link_row_is_preserved`) and search.rs:8561 (which INSERT shape an edge gets). Removing the `#[ignore]` is therefore necessary and **not sufficient**, and declaring 1.2 done on it alone is the most expensive silent risk in this whole option.
**Invariants at risk**: ONE UNIVERSE, ONE IDENTITY. owner.rs:59-68 states it verbatim: a lock or pool key derived from `Owner.root` must go through `universe_lock::canon`, never string equality, because two keys for one universe means two locks, which is no lock at all. If `UniverseDb` is ever cached in a map, that map's key is this invariant. (I read owner.rs:59-73 and 111-141; I did NOT open universe_lock.rs this session — the `canon` claim is owner.rs's, quoted, not re-verified by me.); ONE WRITER PER DATABASE FILE. `UniverseDb::open` must refuse `owner.is_active` (owner.rs:72 exposes the flag), or the active universe gets a second read-write handle alongside `SearchState.db` (search.rs:12688-12689 is where the existing one is taken). Fail-closed, not clever.; THE CHILD'S FILE LOCK MUST BE RELEASED BEFORE `run_migrations_on` RETURNS. Read this session at federation/migrate.rs:168-177: the connection is explicitly `drop`ped at :175 with the comment 'otherwise the file stays held when run_migrations_on returns, blocking attach_with_safety's re-open in the caller.' A long-lived `UniverseDb` on that same file collides with this. Open-per-write (the `reconcile_filesystem` precedent) has no such conflict; a pool does.; THE ACTIVE-UNIVERSE PATH MUST STAY BYTE-IDENTICAL. `with_active_db` feeds `active_universe_snapshot()`, which is today's `snapshot()` (link_types.rs:498-503) unrenamed in behaviour — so every generated SQL string for the active universe is the same string as before. Any deviation here is a regression in the 99% path to fix the 1% path.; THE `ForeignSchemaOnly` SKIP LIST MUST NOT SILENTLY WIDEN. search.rs:4576-4588 (read this session) splits its skips into two classes: registry-generated DDL, and the MIG-003 one-shots that write `cid_cn:` frontmatter into `.md` files and RENAME them. Giving init the right vocabulary retires the FIRST reason only. If a routed scope is introduced, the file-mutating passes must stay off — a non-owner process rewriting a child's `.md` files is a different and worse hazard than a wrong SQL clause.; EARNED LIVING-LINK DATA. search.rs:8485 (read this session) passes `is_structural_type(&ltype)` as the `structural` argument to `link_row_is_preserved`; that boolean decides whether an existing edge's weight, confidence, traversal_count and created date survive a re-index. CLAUDE.md records that this data lives ONLY in search.db. A wrong vocabulary here destroys it — and `aggregates_for` cannot see it.; THE FINGERPRINT GATES. links_backfill.rs:99 and incoming_links_backfill.rs:49 compare a fingerprint STORED IN a database against one read from the process-global. Threading makes them compare against `db.vocab()`, which is correct — but it does not settle who is allowed to WRITE a child's `schema_versions` stamp. A parent stamping a child's `links_vocab` row is a policy question the type system does not answer.
**Makes the test pass by**: `routed_write_must_match_the_owners_vocabulary` (vocab_harness.rs:274-289) becomes, in place of the `panic!` at :288 — and note there is **no `set_active` anywhere in the routed section**, which is the whole assertion:

    let child_vocab  = deltas(&["refutes"]);
    let parent_vocab = deltas(&["exemplifies"]);
    let expected = index_under_vocabulary(&tmp_dir("child"),  child_vocab.clone(),  NOTES).unwrap();
    let wrong    = index_under_vocabulary(&tmp_dir("parent"), parent_vocab.clone(), NOTES).unwrap();
    assert_ne!(expected, wrong);

    let _serial = HARNESS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    crate::link_types::set_active(parent_vocab);        // the PARENT is active, and STAYS active

    // A real mini-universe on disk: the child's vocabulary is a FILE, not an argument.
    let dir = tmp_dir("routed");
    let cdir = crate::universe::constellation_dir(&dir);          // universe.rs:64-66
    std::fs::create_dir_all(&cdir).unwrap();
    std::fs::write(cdir.join("link-types.json"),
                   serde_json::to_string(&child_vocab).unwrap()).unwrap();

    // The ONE door. `Owner`'s fields are pub (owner.rs:69, :72), so the test builds one directly;
    // production reaches the same value through `resolve_owner` (owner.rs:149).
    let owner = crate::federation::owner::Owner { root: dir.clone(), is_active: false };
    let udb = crate::search::UniverseDb::open(&owner).expect("routed open");

    let empty = std::collections::HashSet::new();
    for (name, body) in NOTES {
        let p = dir.join(name);
        std::fs::write(&p, body).unwrap();
        crate::search::index_note(udb.db(), &p.to_string_lossy(), "harness", true).unwrap();
    }
    for (name, _) in NOTES {
        let p = dir.join(name);
        crate::search::maintain_incoming_after_save(udb.db(), &p.to_string_lossy(), &empty, "", &empty).unwrap();
    }
    let routed = aggregates_for(udb.db().conn(), &dir).unwrap();

    assert_eq!(routed, expected, "a routed write must use the OWNER's vocabulary");
    assert_ne!(routed, wrong,    "and never the active universe's");

WHY IT PASSES, VALUE BY VALUE. `NOTES` (vocab_harness.rs:180-183) is `[[refutes::Target|because of X]]`. `parse_link_body` (search.rs:7243) splits on `::`, and its `is_type` check at :7244 becomes `v.is_known("refutes")`. With `v` = the child's registry it is TRUE → `link_type = "refutes"`, `target = "target"`. With the parent's (`exemplifies` is a SEED id, so `deltas(&["exemplifies"])` merges to the same id set as seeds-only) it is FALSE, falls through to the predicate-last branch at :7263-7278, `"because of x"` is not a type either, and the edge collapses to `("associative", "refutes::target")`. Different `edges`. `maintain_incoming_after_save` (search.rs:2637) then builds its UPDATE at :2659-2661 from `incoming_aggregate_assignments(db.vocab(), "note_meta")` — with the child's registry `refutes` is in `sql_in_list_cognitive()` so Target's `incoming_link_types` is `"refutes (1)"`; with the parent's it is `""`. Different `incoming_types`. Both differences are exactly what `aggregates_for` observes, and `assert_ne!(expected, wrong)` at :282 already proves the two vocabularies diverge on this note.

AND `a_vocabulary_swap_reaches_back_into_an_already_open_database` (vocab_harness.rs:226-256) STAYS GREEN, which is the check that the fix is real rather than a re-arrangement. That test calls `index_note` on a connection built by the ACTIVE path, so under the new signature it becomes `with_active_db(&conn, |db| index_note(db, ...))` — which re-reads the global at call time and therefore still observes the swap at :241. The pinned hazard is unchanged for the active path; what changed is that a routed write no longer goes through the global at all. If that test ever flips, the coupling moved and the premise needs re-checking, exactly as its comment at :251-254 demands.

RECOMMENDED, NOT SMUGGLED IN: `Aggregates` (vocab_harness.rs:58-69) should also carry `note_meta.outgoing_count`/`outgoing_link_types` and `sky_nodes.stratum`/`maturity`, and `index_under_vocabulary` should call `maintain_sky_after_save` alongside `maintain_incoming_after_save` (it already calls the latter at :156 with the comment at :149-152 explaining why `index_note` alone does not reach it — the identical argument applies to sky). Without that, the test cannot see three of the vocabulary-derived surfaces a routed write touches.
**Cannot do**: 1. **It cannot repair vocabulary already frozen in a child's `sqlite_master`.** Trigger bodies are written once and fire forever after. Verified: `create_outgoing_link_triggers` (search.rs:2286) drops-then-creates from the current registry at :2327-2328, and is `owns`-gated at search.rs:5969 so a foreign DB keeps whatever its owner last wrote — but search.rs:5531-5575 (read this session) is at the function's top-level indentation, inside no `if owns` block, so `init_db_schema_only` → federation/migrate.rs:169 has been writing the parent's `structural_not_in_clause` into children's `note_links_sky_ai`/`_au` bodies. Angle B makes the correct re-creation *expressible* for the first time; performing it is a separate one-shot with its own decision about scope.

2. **It cannot reach a vocabulary reader that has no connection to bundle with.** By the supplied map, 11 of 17 read-side sites are filesystem walkers with neither a `&Connection` nor an `AppHandle`: strata.rs:168/199/208, inspector360.rs:284/343/368/375, libraries.rs:4040/4065. They still need `&LinkTypeRegistry` threaded — the SAME parameter type, sourced from an `Owner` rather than from a `Db`. The design composes with them, but the bundle is not the vehicle there. Nor does it help cache.rs:516/:548/:1288, which loop over MANY schemas on ONE `federated_conn` — per-schema vocabulary is necessarily a parameter there, not a property of the connection, which is a standing argument against ever making `Db` the only shape.

3. **It routes nothing.** `reindex_single_note` (search.rs:12682) still takes `&SearchState` and still uses `state.db`; its 16 production callers are untouched. Which of them route, and on what signal, is a separate decision. Angle B builds the vehicle and proves it carries the right cargo — it does not choose the destinations.

4. **It does not make the acceptance test sufficient.** See `risk` #4: `aggregates_for` observes four values; the routed write touches at least eight vocabulary-derived surfaces.

5. **It provides no pool.** Each routed write would run the full `init_db_scoped` (search.rs:4601-6407, ~1,800 lines of `IF NOT EXISTS` DDL) on open. Correct but not cheap. `UniverseDb` is the natural pool entry and `rusqlite::Connection` is `Send` (verified: `unsafe impl Send for Connection {}` at rusqlite-0.31.0/src/lib.rs:382, and Cargo.toml pins `rusqlite = { version = "0.31", features = ["bundled", "backup"] }`), so a `Mutex<HashMap<..>>` static is mechanically possible — but the moment it holds a child's connection open it collides with federation/migrate.rs:171-175 and with `federation_prewarm`'s own read-write handle on the same file. Out of scope here, and deliberately so.

6. **It does not settle who may write a child's `schema_versions` fingerprint stamps** (links_backfill.rs:464, incoming_links_backfill.rs:173). Threading makes the comparison correct; the policy is a different question.
**Cannot-forget check**: **Mostly yes — and the one place it is only a convention, I will name rather than paper over.**

What the structure genuinely forbids:

* **You cannot call the write path without a vocabulary.** `index_note(db: Db<'_>, ...)` will not accept a `&Connection`. Every one of ~72 production and ~65 test call sites is a compile error until it supplies one. Forgetting is not expressible.
* **You cannot invent a vocabulary.** There are exactly two producers of a `Db`, and NEITHER takes a vocabulary argument a caller writes. `with_active_db(conn, f)` reads the process-global itself; `UniverseDb::open(owner)` derives it from `owner.root`'s `link-types.json`. There is no `Db::new(conn, vocab)` in production. The pairing is made by the constructor, never by the call site — which is the LL-048 shape: the structure holds the invariant, the caller holds nothing.
* **You cannot silently leave a reader on the global.** This is the part `Deref` would have destroyed, and why the design refuses it. Deleting `link_types::is_known_type` (link_types.rs:359) and `is_structural_type` (:369) and renaming `snapshot()` (:498) to `active_universe_snapshot()` turns all 29 remaining production reader lines into unresolved names. The compiler does not merely check what you migrated — it *enumerates what you did not*. Each of those 29 becomes a forced, one-line answer to "whose vocabulary is this?", which is precisely the question nobody was being asked.
* **You cannot get a wrong-vocabulary answer from a missing file.** `read_persisted_json` (universe.rs:260-289) errors on unreadable/empty/corrupt and returns `Ok(None)` only for genuine absence, so a routed open of an unreadable `link-types.json` FAILS instead of falling back to the 8 seeds. `load_active`'s leniency (link_types.rs:511-513) is the right default at boot and would be a silent corruption here; `list_link_types` (link_types.rs:568-589) already documents this exact split for the exact same reason.

**Where it does NOT satisfy the rule, stated plainly:**

* `Db::conn()` is an escape hatch. A function you leave on `&Connection` keeps compiling when handed `db.conn()`, and it reads the global. The compiler will not find it. What the design buys is that the hatch is a distinct, greppable token rather than an invisible ambient read — the migration's residue is *countable*, not *hidden*. That is weaker than "cannot forget"; it is "cannot forget without leaving a fingerprint."
* `with_active_db(routed.db().conn(), |db| ...)` re-pairs a routed connection with the active vocabulary in a single expression. Nothing but reading the line stops it. Closing that would require newtyping `Connection` across all 257 `&Connection` signatures in the crate, which is a different and much larger option than this one.
* And the honest boundary: this makes the vocabulary impossible to *forget*. It does not make a routed write *safe* — points 1 and 2 of `what_it_cannot_do` are hazards the type cannot reach, because the wrong vocabulary is already on disk in one case and there is no connection to bundle with in the other.

---

## OPTION 3: The Owner Scope — one constructor, and no ambient door left to walk through
Bundle connection + vocabulary into a WriteScope that only a path-in, owner-resolved constructor can build, then DELETE the three process-global reader functions so the compiler refuses to let any parse or SQL-generation site compute without a registry in hand — a scope, not a pool.

**Mechanism**: THREE new items; the load-bearing half is three DELETIONS.

NEW 1 — `link_types::registry_for_root(root: &Path) -> Result<LinkTypeRegistry, String>`. Pure, no AppHandle. Path = `universe::constellation_dir(root).join("link-types.json")`; `constellation_dir` (universe.rs:64) is already pure per-root — only `link_types_path` (link_types.rs:507-509) is ambient and is not used. Read via `universe::read_persisted_json::<Vec<LinkTypeDef>>` (universe.rs:260-289), then `LinkTypeRegistry::merge` (link_types.rs:115). STRICT, not lenient: `read_deltas` (link_types.rs:514-518) falls back to the 8 seeds on an unreadable file; `read_persisted_json` returns Ok(None) only for NotFound (universe.rs:267) and Err for permission-denied / truncated / unparseable (universe.rs:268-289). Same split link_types.rs:526-531 and :571-583 already draw — leniency is right for a boot read, wrong for a read a WRITE derives from. Unreadable child vocabulary ⇒ refuse, matching resolve_owner's fail-closed rule (owner.rs:134-140).

NEW 2 — `search::WriteScope<'a> { conn, vocab: LinkTypeRegistry, owner: Owner }`, where `conn` is an enum arm: a borrow of the `state.db` guard (search.rs:1342) or an owned Connection (routed).

NEW 3 — ONE public constructor: `WriteScope::for_note(app, note_path) -> Result<WriteScope,String>`. It calls `federation::owner::resolve_owner(app, note_path)` (owner.rs:149) ITSELF and branches on `owner.is_active`: active ⇒ borrow state.db, vocab = `snapshot()` (byte-identical to today); routed ⇒ `Connection::open(owner.root/.constellation/search.db)` + the PRAGMA batch + `register_fts5_tokenizer(&mut c)` (search.rs:1572) — exactly the shipped shape of `reconcile_filesystem`'s dedicated walk connection (search.rs:11976-11997) — and vocab = `registry_for_root(&owner.root)`. The caller supplies a PATH, never a universe and never a vocabulary, so conn and vocab are derived from ONE key and cannot be mismatched: there is no constructor that accepts them separately.

NOT A POOL. Nothing is cached or held open. Four verified reasons: (a) `run_migrations_on` explicitly `drop(conn)`s to release the child's file lock before returning, else `attach_with_safety`'s re-open blocks (federation/migrate.rs:171-175) — a held-open child entry breaks that premise; (b) `federation_generation` (search.rs:1415) bumps only on universe switch (search.rs:11236), so unlinking a child from universe.json leaves a stale entry; (c) `ACTIVE_OWNER` (universe_lock.rs:218) holds exactly ONE lock, so a held child connection has no ownership story; (d) routed writes are rare. If open cost ever measures, an LRU keyed by `universe_lock::canon(owner.root)` (universe_lock.rs:87-89 — the identity function owner.rs:59-68 mandates) drops in BEHIND the same constructor and changes no call site.

THE DELETIONS — this is what makes it a structure rather than a promise:
1. delete `link_types::is_known_type` (link_types.rs:359);
2. delete `link_types::is_structural_type` (link_types.rs:369);
3. turn `link_types::structural_frontmatter_targets` (link_types.rs:385, which self-snapshots at :390) into `LinkTypeRegistry::structural_frontmatter_targets(&self, fm)`.
After that there is NO way anywhere in the codebase to answer a link-type question without holding a `&LinkTypeRegistry`. Every one of the 10 production readers becomes a compile error until it is given one.

HOW IT REACHES index_note: `index_note`/`index_note_bulk`/`index_note_impl` (search.rs:7873/7880/7884) take `&WriteScope` in place of `&Connection`. The three inline `is_structural_type` reads (search.rs:8018, 8485, 8561) become `scope.vocab.is_structural(..)`. The five conn-less parsers take `reg: &LinkTypeRegistry`: `extract_wikilinks` (7068), `extract_typed_links` (7222), `parse_link_body` (7243), `extract_frontmatter_typed_links` (7306), `emit_frontmatter_links` (7361). `resolve_wikilink_type` (link_types.rs:451) already takes `&LinkTypeRegistry` — the pattern exists and stops one frame short.

The four SQL generators gain the registry as first parameter: `outgoing_aggregate_assignments` (2242), `incoming_aggregate_assignments` (2488), `stratum_sql_expr` (188), `maturity_sql_expr` (266). At the ~12 ambient (non-routed) sites the caller hoists `let reg = link_types::active_universe_snapshot();` — `snapshot()` renamed so every remaining ambient read SAYS "active universe" in the diff. `maintain_incoming_after_save` (2637) and `maintain_sky_after_save` (2706) gain the same parameter.

The scope is constructed at exactly TWO places: `reindex_single_note` (search.rs:12682) and `reindex_delete_note` (search.rs:12368) — the two per-note funnels — not at their 16 + N callers.

**Signature changes**: ~75 production edits (~120 more in test modules). Every one verified by grep this session.

CONNECTION → SCOPE (3 defs + 4 production callers = 7):
- `index_note(conn: &Connection, ..) ` → `index_note(scope: &WriteScope, ..)` (search.rs:7873); same for `index_note_bulk` (7880) and `index_note_impl` (7884). Production callers: search.rs:4280, 4339 (mig003_step3_soft_rebackfill), 8725 (index_library_recursive), 12718 (reindex_single_note).

PARSE CHAIN, +`reg: &LinkTypeRegistry` (5 defs + 6 production callers = 11):
- `extract_wikilinks(content)` (7068) ← 7927
- `extract_typed_links(content)` (7222) ← 8016
- `parse_link_body(body)` (7243) ← 7232, 7378
- `extract_frontmatter_typed_links(content)` (7306) ← 8025
- `emit_frontmatter_links(wl,key,value,out,seen)` (7361) ← 7341, 7350

DELETIONS + forced rewrites (3 defs + 10 production readers = 13):
- `is_known_type` (link_types.rs:359) ← search.rs:7244, search.rs:7371, libraries.rs:7490
- `is_structural_type` (link_types.rs:369) ← search.rs:8018, 8485, 8561, 9550
- `structural_frontmatter_targets` free fn → `&self` method (link_types.rs:385) ← search.rs:7079, strata.rs:199, inspector360.rs:368. strata.rs and inspector360.rs already hold a `reg` (strata.rs:168, inspector360.rs:343) — verified by reading strata.rs:160-214 — so they pass it and LOSE a redundant per-directory `snapshot()` at link_types.rs:390.

SQL GENERATORS, +`reg: &LinkTypeRegistry` first param (4 defs + ~27 production callers ≈ 31):
- `outgoing_aggregate_assignments(src)` (2242) ← 2327, 2328, links_backfill.rs:248. (links_backfill.rs:739/740 and search.rs:6796/6797 are inside test modules — verified.)
- `incoming_aggregate_assignments(np)` (2488) ← 2661, 11040, 12548, links_backfill.rs:307, name_fold_backfill.rs:157, incoming_links_backfill.rs:363
- `stratum_sql_expr()` (188) ← 2719, 5840, 5910, 6289, 11054, 12560, links_backfill.rs:359, name_fold_backfill.rs:173, sky_backfill.rs:388
- `maturity_sql_expr()` (266) ← 2720, 5840, 5956, 6290, 11055, 12561, links_backfill.rs:360, name_fold_backfill.rs:178, sky_backfill.rs:399

MAINTENANCE, +reg (2 defs + 2 production callers + 1 harness = 5):
- `maintain_incoming_after_save` (2637) ← 12754, vocab_harness.rs:156
- `maintain_sky_after_save` (2706) ← 12765

READ PATH pulled in by deleting `is_structural_type` (≈3):
- `structured_search(conn, filters, limit)` (9347) gains a registry; callers at 13108, 13187.

RENAME CASCADE pulled in by deleting `is_known_type` (≈3):
- `rewrite_wikilinks_in_text(content, re, new_name)` (libraries.rs:7478) gains a registry; threaded from `rewrite_candidates` (libraries.rs:7326) and `update_links_on_rename` (libraries.rs:6792). Today that command is fenced off child universes on BOTH branches (libraries.rs:6963, 6982-6986), so it passes the active registry and behaviour is unchanged — but the parameter now EXISTS, so whoever removes the fence in a later phase must decide what to put in it instead of silently inheriting the global.

Test churn: `index_note` alone has ~30 test call sites in search.rs plus libraries.rs:7845/7887 and index_repair.rs:1018. Mitigate with a `#[cfg(test)] WriteScope::for_tests(conn, vocab)` — test-only, so production cannot reach it.
**Call sites touched**: 75
**Speed**: Stage A (the part that turns the acceptance test green): 1.5–2 days of build. The compiler drives it — after the three deletions, `cargo check` enumerates every site that must be fixed, so nothing can be missed by inattention. Breakdown, honestly: the scope + registry_for_root + parse chain is half a day; the ~31 SQL-generator call sites are mechanical (hoist one `let reg = ...` per function) but touch 6 files; the test-module churn (~120 sites) is the real sink and is why the `#[cfg(test)]` scope constructor matters. Stage B (the DDL blind half, below) is a further half day plus a Boss ruling. Stage C is out of 1.2.
**Effort**: High-mechanical, low-conceptual. There is no new concurrency, no lifetime cleverness beyond one borrow-or-own enum, no trait objects, no async. The conceptual work is already done: `resolve_owner` (owner.rs:111/149) exists and is tested, `resolve_wikilink_type` (link_types.rs:451) is the shipped precedent for registry-as-parameter, `merge` (link_types.rs:115) is pure, `read_persisted_json` (universe.rs:260) is the strict reader, and `reconcile_filesystem` (search.rs:11976-11997) is the shipped precedent for a dedicated per-operation connection. The effort is the breadth of the diff, not its difficulty — which is exactly the kind of cost the Whole-Ecosystem Fix Law says to pay once rather than in six places later.
**Risk**: WHAT BREAKS LOUDLY (the design intent): every deletion is a hard compile error at all 10 production readers. There is no path where a site silently keeps reading the global — that is the whole point of deleting rather than deprecating.

SILENT RESIDUAL 1 — THE PERSISTED TRIGGER BODIES, which no call-time threading can reach. A routed write fires the CHILD's own `note_links_outgoing_ai/_ad/_au` (bodies baked by `create_outgoing_link_triggers`, search.rs:2286-2330) and the sky trigger family. That is CORRECT when the child ran `init_db` Active for itself. It is a silent GAP when it did not: `init_db_scoped` gates `create_outgoing_link_triggers` behind `if owns` (search.rs:5969), so a universe that has only ever been federated may have no outgoing triggers, and `note_meta.outgoing_*` simply never moves under a routed write. Wrong-by-omission, not wrong-by-value; unobserved by the acceptance test (`aggregates_for`, vocab_harness.rs:73-106, reads no outgoing column). Detectable with one `sqlite_master` probe — that is Stage B.

SILENT RESIDUAL 2 — the owner's self-heal will not notice. After a routed write the child's `links_vocab` stamp still equals the child's own fingerprint, so `links_backfill::is_needed` (links_backfill.rs:99) reads FALSE on the child's next boot and the gap from Residual 1 is never healed. Stage B's answer is a distinct `schema_versions` marker row written by the routed write and consumed by the owner's boot — NOT a fingerprint stamp, so it does not violate R4's prohibition, but it is a write into a child's database and needs an explicit ruling before it ships.

SILENT RESIDUAL 3 — the ~12 remaining ambient `snapshot()` reads (the backfills, the init_db DDL, the two `recompute_*` repair paths). Behaviour-identical today because none of them is routed in 1.2. If a later phase routes one and nobody changes the hoisted call, it is silently wrong again. Mitigation is legibility, not typing: renaming `snapshot()` to `active_universe_snapshot()` makes every one of those lines say what it assumes, at the call site, in the diff.

CROSS-PLATFORM: `registry_for_root` and the routed `Connection::open` take `owner.root`, which `resolve_owner_in` returns in the STRIPPED form (owner.rs:80-89, 123). Any lock or cache key derived from it must go through `universe_lock::canon` (universe_lock.rs:87-89), per owner.rs:59-68 — one universe, one identity. Stage A introduces no key, so it cannot break this; a later LRU must.

NOT AT RISK: nothing in Stage A mutates shared state for a duration. There is no window. That is the hard constraint met by construction, not by care.
**Invariants at risk**: LL-047 / H1b — no shared-mutable-state window. MET BY CONSTRUCTION: nothing calls `set_active` (link_types.rs:481); the vocabulary is a value carried by the WriteScope. Pinned by `a_vocabulary_swap_reaches_back_into_an_already_open_database` (vocab_harness.rs:227), which must KEEP passing for the un-routed path and whose premise the routed path no longer shares.; `link_types::list_link_types` (link_types.rs:585) calls `set_active` at :588 from an ordinary read-shaped IPC command the Links editor invokes — so merely OPENING the editor mutates the global mid-flight. This design is immune (routed writes never read the global), but the ACTIVE universe's writes still read `snapshot()` and are still exposed to it. Out of 1.2's scope; must be filed, because it means the H1b window is wider than boot-and-save.; PJ-232's `if owns` gates (search.rs:4602, 5640, 5891, 5933, 5969, 6284, 6312, 6335) must not be weakened. This design does not touch `init_db_scoped` at all in Stage A — the routed write never calls it, which is stronger than gating it.; The un-`owns`-gated sky_link trigger DDL at search.rs:5531-5575 (`sx_new` read at :5540) contradicts InitScope's own doc (search.rs:4577-4581). It survives Stage A unchanged. It is harmless ONLY because `merge` pins the structural set to exactly {contains, parent} — link_types.rs:93-94, 131, 148, 159, with `seeds()` always the base at :121. That invariance is a property of `merge`, not a declared contract. Stage A must add a test that fails the moment `merge` stops forcing `structural = false` for custom types (link_types.rs:159), or three more DDL sites silently join the divergent family.; Earned Living-Link data. `is_structural_type` at search.rs:8485 feeds `link_row_is_preserved` (search.rs:477-489), which decides whether an edge's weight / confidence / traversal_count / archived status survives a re-index. CLAUDE.md records those columns as living ONLY in search.db. A wrong answer here destroys them silently and the acceptance test cannot see it — `aggregates_for` selects only (source_path, target_name, link_type). Stage A routes this read correctly; the harness still does not observe it.; `incoming_links_backfill::is_built` (incoming_links_backfill.rs:73) deliberately excludes the fingerprint, so it CANNOT refuse a foreign-vocabulary write. Stage A makes that safe by making the write carry the right vocabulary, but the gate itself stays unable to detect a wrong one — it must not be mistaken for a guard later.; Boot latency and typing latency. Stage A adds ONE `resolve_owner` call plus a `snapshot()` clone per save on the active path — the same clone the code already performs three frames down. Must be measured on the 7,824-note universe before commit.
**Makes the test pass by**: `routed_write_must_match_the_owners_vocabulary` (vocab_harness.rs:276) becomes, replacing the `panic!` at :288 and dropping the `#[ignore]` at :275:

```rust
let child_dir = tmp_dir("routed");
std::fs::create_dir_all(child_dir.join(".constellation")).unwrap();
std::fs::write(child_dir.join(".constellation/link-types.json"),
    serde_json::to_string(&deltas(&["refutes"])).unwrap()).unwrap();

// THE PARENT IS ACTIVE — the process-global holds the WRONG vocabulary throughout.
crate::link_types::set_active(deltas(&["exemplifies"]));

let vocab = crate::link_types::registry_for_root(&child_dir).unwrap();  // NEW 1
let conn  = crate::search::init_db(&child_dir.join("search.db")).unwrap();
let scope = crate::search::WriteScope::for_tests(&conn, vocab);          // NEW 2/3
for (name, body) in NOTES {
    let p = child_dir.join(name);
    std::fs::write(&p, body).unwrap();
    crate::search::index_note(&scope, &p.to_string_lossy(), "harness", true).unwrap();
}
let empty = std::collections::HashSet::new();
for (name, _) in NOTES {
    let p = child_dir.join(name);
    crate::search::maintain_incoming_after_save(&scope, &p.to_string_lossy(), &empty, "", &empty).unwrap();
}
let routed = aggregates_for(&conn, &child_dir).unwrap();
assert_eq!(routed, expected, "a routed write must use the OWNER's vocabulary");
assert_ne!(routed, wrong,     "and never the active universe's");
```

WHY IT PASSES, read off `aggregates_for` (vocab_harness.rs:73-106), which observes exactly four things:
- `edges` = (source_path, target_name, link_type). `[[refutes::Target|because of X]]` is decided at `parse_link_body`'s `is_type` (search.rs:7244) — now `scope.vocab.is_known("refutes")` = TRUE, yielding link_type `refutes`, target `target`. Under the parent's vocabulary that same body falls through search.rs:7247-7260 to the predicate-last arm and returns `("associative", "refutes::target", "because of X")` (search.rs:7278). Different values, identical row count — the exact trap the header at vocab_harness.rs:16-20 names.
- `link_rows` — unchanged either way, which is what makes the row-count assertion at :197 keep passing.
- `incoming_counts` + `incoming_types` — produced by `maintain_incoming_after_save` (search.rs:2637) building `UPDATE note_meta SET {incoming_aggregate_assignments("note_meta")}` at :2659-2661. With the registry threaded, `sql_in_list_cognitive` (:2493), `sql_rank_case_cognitive` (:2494) and `cognitive_sentinel_rank` (:2495) come from the CHILD's registry, so Target's `incoming_link_types` reads `refutes (1)` instead of collapsing.

`expected` is built by `index_under_vocabulary` (vocab_harness.rs:135) under `set_active(child_vocab)` with `init_db` Active-scoped, so its DDL carries the child's vocabulary too — and paths are stripped to the harness root at :76-80, so the two temp directories compare equal.

DEADLOCK CHECK: `HARNESS_LOCK` (vocab_harness.rs:133) is a non-reentrant `std::sync::Mutex`. The two `index_under_vocabulary` calls at :280-281 take and release it before the routed block, so the routed block may take it once for its own `set_active` — it must NOT be held across those two calls.

WHAT REMOVING THE `#[ignore]` DOES *NOT* PROVE, stated so nobody reads it as more: it proves the parse chain (7244, 7371, 8018) and the incoming aggregate (2489) are routed. It proves nothing about sky_nodes.stratum/maturity, note_meta.outgoing_*, or the earned-link preservation decision at 8485/8561 — the harness observes none of them. If the acceptance condition is to mean "the routed write used the owner's vocabulary" rather than "four columns did", `Aggregates` needs `sky_nodes.stratum/maturity` and `note_meta.outgoing_*` added and `index_under_vocabulary` needs to route through `maintain_sky_after_save`. I recommend adding both in Stage A; it is ~20 lines of harness.
**Cannot do**: 1. It cannot fix a wrong vocabulary already FROZEN in a child's `sqlite_master`. `outgoing_aggregate_assignments` is not called at save time at all — it lives in the trigger bodies (search.rs:2327-2328) and in `links_backfill::recompute_range` (links_backfill.rs:248). Threading the call reaches neither. Stage A's answer is that the routed write must never run `init_db` on a foreign database, so the child's own bodies are what fire; that is right when they exist and a silent gap when they do not (search.rs:5969's `if owns`).

2. It does not make the child's derived views self-heal after a routed write. `links_backfill::is_needed` (links_backfill.rs:99) compares a stamp in the child's DB against a process-global fingerprint; after a correct routed write both still match the child's own, so nothing is re-armed. Needs the Stage B marker and a ruling.

3. It does not touch the FEDERATED READ side, which is already wrong today with no Router involved: `backlink_rows_in_schema` (cache.rs:516), `outgoing_rows_in_schema` (cache.rs:548) and `read_links_in_schema` (cache.rs:1288) each take `schema: &str` and are called once per ATTACHED schema over ONE connection (`state.federated_conn`, cache.rs:626) while building their structural-exclusion from the active registry. Note the design consequence: because one connection serves many schemas there, per-schema vocabulary must be a PARAMETER, never a property of the connection — which is why this proposal threads a value and does not try to bind anything to a rusqlite handle. Filed, not fixed.

4. It does not route renames. `rewrite_wikilinks_in_text` (libraries.rs:7490) is the only vocabulary reader in the repo whose answer reaches DISK (`gate_rmw` at libraries.rs:7346 → `atomic_write` at write_gate.rs:667). Stage A gives it the parameter and leaves the federation fence (libraries.rs:6963, 6982-6986) in place; removing that fence is a separate phase.

5. It does not route the backfills. `incoming_links_backfill::run` opens `db_path(app)` (incoming_links_backfill.rs:123-124 → search.rs:1465-1468 → universe.rs:69), which resolves the AMBIENT active universe. That path binding is what keeps them off child databases today — it is incidental, not a guard, and this proposal does not convert it into one.

6. It cannot prove the un-gated sky_link DDL (search.rs:5540) is harmless. It relies on `merge`'s structural invariance, which is undeclared. Stage A adds the pinning test; it does not remove the reliance.

7. It has no answer for two concurrent routed writes to the SAME child from this process. SQLite's single-writer + `busy_timeout` handle the database, but there is no `ACTIVE_OWNER`-equivalent for a child (universe_lock.rs:218 holds exactly one lock, the active universe's) — so nothing prevents the child's OWN process, running elsewhere, writing at the same time. UNVERIFIED whether that is already covered; `is_cuniverse_open_elsewhere` (federation/migrate.rs:234-256) suggests a probe exists. I did not read it this session.
**Cannot-forget check**: MOSTLY YES, AND HERE IS EXACTLY WHERE IT IS NOT.

WHERE THE STRUCTURE GENUINELY CANNOT FORGET:
- After deleting `is_known_type` (link_types.rs:359) and `is_structural_type` (link_types.rs:369) and making `structural_frontmatter_targets` (link_types.rs:385) a `&self` method, there is no expression in the language that answers a link-type question without a `&LinkTypeRegistry` in scope. A new call site cannot be written wrong; it cannot be written at all until someone decides whose vocabulary it uses. That is the difference between this and every "thread a parameter and remember to pass the right one" plan: the ambient alternative is gone, not discouraged.
- `WriteScope::for_note(app, note_path)` takes a PATH and resolves the owner itself. A caller cannot select a universe, so it cannot select the wrong one; conn and vocab are derived from one key inside the constructor, so they cannot disagree.
- The routed vocabulary reader is strict (universe.rs:267-289), so an unreadable child `link-types.json` cannot silently degrade to the 8 seeds and write seeds-flavoured rows into a child that has custom types. It errors.

WHERE IT IS STILL A PROMISE — three places, named honestly:
1. `WriteScope::for_this_process_active_universe(conn)` — needed at `init_db_scoped`'s two `mig003_step3_soft_rebackfill` calls (search.rs:4280, 4339) and `reconcile_filesystem`'s walk connection (search.rs:11976), which have a connection but no `AppHandle` and are active-universe-only by construction. It reads `snapshot()` itself (no registry parameter, so at least a wrong registry cannot be handed to it), but nothing in the type system stops someone pointing it at a foreign connection. The defence is the name plus a test asserting its production call-site count. That is legibility, not enforcement, and I am not going to dress it up as more.
2. The ~12 ambient generator call sites hoisting `active_universe_snapshot()`. Correct today; a promise the day one of them is routed. The rename is the whole mitigation.
3. `search.rs:5540`'s un-gated sky_link DDL relies on `merge`'s undeclared structural invariance. The pinning test converts a silent future break into a loud one; it does not remove the reliance.

WHY I STILL THINK THIS IS THE RIGHT SHAPE: LL-048 rule 2 asks for a structure that cannot forget. The three deletions deliver that for the entire class the migration is about — 10 production readers, all forced open by the compiler, none able to close again. The three residuals are all in the OPPOSITE direction from the failure mode this migration exists to end: each of them keeps the ACTIVE universe reading the active vocabulary, which is correct today and becomes a visible line in a future diff rather than a hidden global read three frames down. I would rather ship a guarantee with three named, greppable seams than a promise with none.
---

## C. The four adversarial passes

## ATTACK ON: Angle A — the vocabulary as a threaded parameter (`&LinkTypeRegistry`), constructed once at the frame that chooses the database — verdict VIABLE_WITH_CONDITIONS
- [survives] A1 — the 1500ms debounced save fires for an ACTIVE note while a routed write to a child is in flight: Eisa types in an active-universe note; the 1500ms debounce fires `constellation_search_reindex` (search.rs:12251) -> `reindex_single_note` (search.rs:12682), which takes `state.db.lock()` at search.rs:12688. Concurrently a routed write holds the CHILD's connection. Under Angle A the routed frame holds `&LinkTypeRegistry` on its own stack; the active save holds its own. Neither reads `cell()` (link_types.rs:353). I attacked this harder: `list_link_types` (link_types.rs:585) is a READ-shaped IPC command the Links editor calls to populate itself, and it calls `set_active(deltas)` at link_types.rs:588 — so merely OPENING the link-types editor mutates the process-global mid-flight. Under Angle A that mutation lands on a global no converted frame reads. The LL-047 window is genuinely closed AT EVERY CONVERTED FRAME — this is the option's real strength and it survives cleanly. The residual: any Tier-2 frame left unconverted still reads the global, and because `set_active` is always fed from the ACTIVE universe's file (link_types.rs:507-509, :588), the stale read is correct for active writes and wrong ONLY for routed ones — i.e. exactly the surface nobody observes.
    evidence: link_types.rs:585-590 (list_link_types calls set_active on a read command); link_types.rs:353-355 (cell()); search.rs:12688 (state.db.lock()); search.rs:12251
- [survives] A2 — the watcher's adopt path fires on a child universe's file: VERIFIED REACHABLE, and it is the Router's actual target. The frontend calls `watch_library` per library (src/lib/libraries/store.ts:5457) and the library set comes from the federation-spanning resolver, so the app DOES watch cUniverse paths — search.rs:12961-12966 says so verbatim: 'the app watches every library in the recursive set, so any path a federated library owned was indexed straight into the active universe's index.' PJ-207 §8 fences Pass 1 today by scoping `libs` to `try_load_libraries` (search.rs:12970) so a foreign path resolves to no owning library and is SKIPPED. Phase 1.2 removes that fence. Angle A survives Pass 1: `reindex_single_note` gains an `&Owner` parameter, so all 16 production call sites (bases.rs:437, index_repair.rs:853, libraries.rs:1450/1890/1967/2662/2811/7071, reconcile.rs:469/565, search.rs:12275/12861/13010, shape.rs:214, tasks.rs:540, universe.rs:2488) become compile errors and every one must state whose universe it is writing. Pass 2 (deletes) is the harder half — search.rs:12969-12971 says it 'deliberately keeps its own shape and consults no library set', and calls `reindex_delete_note(&state, ...)` (search.rs:13029) on the ACTIVE db for ANY vanished .md, including a child's. Angle A's Tier-2 signature change on `reindex_delete_note` (search.rs:12368) turns its 6 production call sites into compile errors too, forcing that routing decision to be made rather than inherited. Loud, not silent.
    evidence: src/lib/libraries/store.ts:5457; src-tauri/src/search.rs:12961-12971; search.rs:13029; search.rs:12548 + :12560-12561 (reindex_delete_note's three vocabulary reads)
- [survives] A3 — a backfill tick runs on a background thread mid-routed-write: `links_backfill::maybe_schedule` spawns at links_backfill.rs:76 and operates on `state.db`; `incoming_links_backfill::run` opens its OWN handle at incoming_links_backfill.rs:124 via `crate::search::db_path(app)` = `active_constellation_dir(app).join("search.db")` (search.rs:1465-1468, universe.rs:69-72). Both are bound to the ACTIVE universe today, so neither can touch a routed child connection. Angle A survives, and does something stronger that no connection-bound design does: the two fingerprint gates compare a value STORED IN A DATABASE against a value read from the process-global — links_backfill.rs:99 (`stored_vocab_fingerprint(conn) != snapshot().fingerprint()`) and incoming_links_backfill.rs:49. Because Angle A hands the VALUE, a routed gate compares the child's stored fingerprint against the child's registry — child-vs-child, correct. A pool that bound only a connection would still have to answer where `fingerprint()` comes from. Angle A answers it for free.
    evidence: links_backfill.rs:76, :99; incoming_links_backfill.rs:49, :124; search.rs:1465-1468; universe.rs:69-72
- [survives] A4 — a universe SWITCH happens while a routed handle is held: `invalidate_search_state` (search.rs:11228) sets `state.db = None` (:11248), `read_db = None` (:11270), `federated_conn = None` (:11277) and bumps `federation_generation` (:11236). Angle A's vocabulary is a stack-local `let` derived from `owner.root`, never from the ambient active pointer — `registry_for_root(&owner.root)` cannot be re-pointed by a switch, and there is no pool entry to invalidate. That is a genuine advantage over a pooled design. Two residual seams, both narrow: (a) if the switch lands BETWEEN the path arriving and `resolve_owner(app, path)` running, `resolve_owner` reads the NEW active universe (owner.rs:150-152) and `resolve_owner_in` fails CLOSED with an error (owner.rs:137-141) rather than guessing — loud, correct; (b) Angle A specifies the vocabulary's lifetime exactly and says NOTHING about the routed connection's. If the routed connection is opened per-write in the `reconcile_filesystem` shape (search.rs:11976, its own `Connection::open` never stored in SearchState) it is immune; if it is parked in SearchState it dies mid-write. The option must state which.
    evidence: search.rs:11228-11283; search.rs:11976; federation/owner.rs:137-141, :149-153
- [BREAKS] A5 — a new call site is added next year by someone who has not read LL-047: THE BARE FORM FAILS. `index_note(&conn, &crate::link_types::snapshot(), path, lib, true)` type-checks and reproduces the bug byte for byte; the option concedes this. I attacked the NEWTYPE completion too, and it also fails, for a reason the option did not name: `Owner` is a plain struct with PUBLIC fields — `pub root: PathBuf`, `pub is_active: bool` (federation/owner.rs:71, :73), no private member, no `#[non_exhaustive]`. So `OwnerVocabulary::of(&Owner { root: active_root, is_active: true })` compiles anywhere in the crate, and a developer who wants the active universe's vocabulary in a hurry will write exactly that. The newtype's proof is only as strong as `Owner`'s constructibility, and today `Owner` can be built by literal. Its own doc-comment (owner.rs:59-68) warns that two identities for one universe means 'two locks, which is no lock at all' — the same reasoning applies to two ways of minting an Owner. What Angle A DOES buy is real and I will not understate it: an omitted argument is a compile error, and `&Connection` / `&LinkTypeRegistry` are different types so a transposition cannot ship. Omission is loud. Wrongness is not.
    evidence: federation/owner.rs:56-73 (pub struct Owner with both fields pub); link_types.rs:498-503 (snapshot() takes no arguments and is callable from anywhere)
- [BREAKS] A6 — the child's search.db has NEVER been opened by the child's own process: REACHABLE AND IT KILLS THE ACCEPTANCE ARGUMENT, not just the write. attach.rs:157-160 skips a child whose `search.db` is missing ('search.db missing' warning); attach.rs:172 routes a schema_incomplete child through `run_migrations_on` -> `init_db_schema_only` (federation/migrate.rs:169), where `owns == false` (search.rs:4602) skips `create_outgoing_link_triggers` (search.rs:5969-5971) and the whole MIG-003 chain. Such a child has NO `links_outgoing` / `incoming_links` stamps. Now route a save into it: `reindex_single_note` reaches search.rs:12712, `incoming_links_backfill::is_built(conn)` reads `schema_versions WHERE module='incoming_links'` (incoming_links_backfill.rs:73) and returns FALSE, so `inc_old` is None and the `if let Some(...)` at search.rs:12754 SKIPS `maintain_incoming_after_save` ENTIRELY. The routed write produces correct `note_links` rows and writes NO incoming aggregates at all. And Angle A's own red-green cannot see it, because `index_under_vocabulary` bypasses `reindex_single_note` and calls `maintain_incoming_after_save` DIRECTLY at vocab_harness.rs:156 — the harness never crosses the `is_built` gate that production must cross. The acceptance test proves a property the production path does not have.
    evidence: federation/attach.rs:157-160, :172; federation/migrate.rs:169; search.rs:4602, :5969-5971; incoming_links_backfill.rs:73; search.rs:12712, :12754; federation/vocab_harness.rs:156
- [BREAKS] A7 — the child's link-types.json is unreadable / locked / corrupt at routed-open time: THE OPTION INHERITS A LENIENCY THIS CODEBASE HAS ALREADY BEEN BURNED BY, IN THIS EXACT FILE. `read_deltas` (link_types.rs:517-521) is `let Ok(data) = fs::read_to_string(&path) else { return Vec::new(); }` plus `unwrap_or_default()` on the parse — an unreadable or corrupt file yields the 8 seeds. The `registry_for_root` the option proposes copies that shape verbatim (`.ok()` / `.unwrap_or_default()`). Walk it: OneDrive/Syncthing/Defender holds the child's `link-types.json` for 300ms; a routed write lands in that window; the child's `refutes` vanishes; `parse_link_body` (search.rs:7243) evaluates `is_type("refutes")` FALSE at search.rs:7249, falls through to predicate-last, and returns `("associative", "refutes::target", "")` at search.rs:7281. One note_links row before, one after. No error anywhere. The codebase already ruled on precisely this split — the 2026-08-02 triage doc-comment above `list_link_types` (link_types.rs, immediately preceding :585) says `read_deltas` staying lenient 'is right for boot… That reasoning holds for a read; it does not survive a read the user will write back from.' A routed write IS a read the user will be written back from. The strict primitive already exists one line away: `universe::read_persisted_json` (universe.rs:260-286) refuses NotFound-vs-everything-else, refuses zero-length as 'no data', and its comment names the case by name — 'Permission-denied, sharing violations (the Windows AV/sync case)'.
    evidence: link_types.rs:514-521 (read_deltas leniency); link_types.rs doc-comment above :585 (the 2026-08-02 ruling); universe.rs:260-286 (read_persisted_json, strict); search.rs:7243-7281 (the collapse)
- [BREAKS] A8 — SQLite TRIGGERS: the frozen DDL in the child's sqlite_master, outside the parameter's reach: THE ATTACK THAT DOES THE MOST DAMAGE, AND IT LANDS. `outgoing_aggregate_assignments` (search.rs:2242) is interpolated into the BODIES of `note_links_outgoing_ai/_ad/_au` at search.rs:2327-2328 and stored in `sqlite_master`. The generated text carries `link_type IN {list}` where `{list}` is `sql_in_list_cognitive()` (link_types.rs:241-249) — a FROZEN literal list of type ids. Angle A's threaded `&LinkTypeRegistry` decides what VALUE `index_note` INSERTs into `note_links`; the frozen trigger decides whether that value is COUNTED. Two different vocabularies, one row. In the steady state Angle A survives — a child whose own process ran `init_db` (Active) has child-flavoured triggers, and `save_universe_link_types` -> `on_link_vocabulary_changed` (search.rs:2771-2780) refreshes them on every vocabulary edit, so a routed write with the child's vocabulary and the child's triggers agrees. It does NOT survive the schema-drifted child of A6: `init_db_schema_only` skips `create_outgoing_link_triggers` (search.rs:5969), so the child keeps whatever an older build wrote, or nothing. A routed INSERT then fires stale trigger text or no trigger at all, and `note_meta.outgoing_count / outgoing_link_types / outgoing_link_types_json / outgoing_top_rank` are silently stale or never maintained. Decisively: PJ-232's own safety argument is a comment at search.rs:5964-5968 — 'The owner creates them correctly on its own next launch; until then nobody writes through them, because the parent attaches a cUniverse read-only.' Phase 1.2 exists to falsify that last clause. Angle A does not touch the comment, the gate, or the frozen text. And `aggregates_for` (vocab_harness.rs:72-106) never reads a single outgoing column, so the acceptance condition is structurally blind to all of it.
    evidence: search.rs:2242-2270 (the generator), :2327-2328 (interpolated into DDL), :5964-5971 (the `if owns` gate and its now-false premise); link_types.rs:241-249; federation/vocab_harness.rs:72-106 (four observed columns, none outgoing)
- [survives] A9 — does the option make `a_vocabulary_swap_reaches_back_into_an_already_open_database` fail?: It does not make it FAIL — it makes it NOT COMPILE, which is the better outcome, and it is a SIGNAL not a defect. The test calls `crate::search::index_note(&conn, &p, "harness", true)` at vocab_harness.rs:245; Angle A changes that arity, so the file stops building and a human must open it. Its own doc-comment (vocab_harness.rs:229-243) pre-authorises exactly this: 'If this ever fails, the coupling changed and 1.2's design premise must be re-checked.' Breaking the call-time coupling IS the LL-047 ruling, so the test has done its job and must be re-authored into its positive form — a swap must NOT reach a routed write. THE LIVE RISK, and it is not hypothetical: the smallest diff that makes the file compile again is `index_note(&conn, &crate::link_types::snapshot(), &p, "harness", true)`, which restores the exact anti-pattern, turns the assertion at vocab_harness.rs:249-255 GREEN again, and lets the author conclude nothing changed. A mechanical compile-error repair can silently re-pin the coupling the migration exists to break. Nothing in the type system stops it; only a human reading the doc-comment does.
    evidence: federation/vocab_harness.rs:227-256 (the test), :243 ('the coupling changed and 1.2's design premise must be re-checked'), :245 (the index_note call whose arity changes)
- [BREAKS] A10 — the acceptance condition cannot see 92 of the 117 sites Angle A must convert: Not one of the nine assigned, but it is the attack that decides whether any of the others get fixed. `aggregates_for` (vocab_harness.rs:72-106) observes exactly four things: `COUNT(*) FROM note_links` (:81), the `(source_path, target_name, link_type)` tuples (:83-86), `note_meta.incoming_count` (:91-93) and `note_meta.incoming_link_types` (:98-100). It never reads `note_meta.outgoing_*`, never `sky_nodes.stratum/maturity`, and never `note_links.weight / confidence / traversal_count`. Angle A's own scoping puts ~25 production sites in Tier 1 (what the assertion requires) and ~92 in Tier 2 (what the Whole-Ecosystem Law requires). Convert `parse_link_body` and `incoming_aggregate_assignments` alone and `routed_write_must_match_the_owners_vocabulary` goes green with 92 sites still ambient — the precise failure the harness header names at vocab_harness.rs:19-20: 'proving a property over the part you happened to look at.' The one Tier-2 site that matters most is invisible: `is_structural_type` at search.rs:8485 feeds `link_row_is_preserved` (search.rs:477-489), which decides whether an existing edge's earned weight / confidence / traversal_count survives the rebuild — and per CLAUDE.md that data lives ONLY in search.db with no disk layer to restore it from.
    evidence: federation/vocab_harness.rs:72-106, :19-20; search.rs:477-489; search.rs:8485
WORST SILENT FAILURE: THE COMPOUND OF A7 AND A8, AND IT DESTROYS DATA THAT HAS NO DISK LAYER TO RESTORE IT FROM.

A child universe defines a custom type `refutes`; a note there reads `[[refutes::Target|because of X]]`. A routed write lands while Defender or a sync client holds the child's `.constellation/link-types.json` for a few hundred milliseconds. The constructor the option proposes — `fs::read_to_string(..).ok().and_then(|s| serde_json::from_str(..).ok()).unwrap_or_default()` — is exactly `read_deltas`' shape (link_types.rs:514-521) and yields the 8 SEEDS. `parse_link_body` (search.rs:7243) evaluates `is_type("refutes")` false at :7249, falls to predicate-last, and returns `("associative", "refutes::target", "")` at :7281.

What lands: still exactly one `note_links` row. Still one incoming link on the target. `link_rows` unchanged. No error, no log, no surfaced anything — the write reports success.

What is actually gone:
1. The edge's `target_name` is now the literal string `refutes::target`, which resolves to no note. `Target.md` silently loses a backlink; `maintain_incoming_after_save` (search.rs:2637-2666) recomputes `incoming_link_types` for a target set that no longer contains it.
2. `link_row_is_preserved` (search.rs:477-489) is keyed on the edge's identity. The target_name changed, so the OLD row is deleted and a NEW row inserted with fresh defaults — weight 1.0, `confidence` reset to unjudged, `traversal_count` 0, `created` re-stamped. **The user's earned Living-Link data on that edge is destroyed**, and CLAUDE.md states plainly that `traversal_count`, `weight`, `last_traversed`, confidence promotions and archival state live ONLY in search.db — "there is no elsewhere yet."
3. Nothing heals it. The `.md` file did not change, so no watcher event, no re-index, no reconcile. It is permanent until a full re-read.
4. If Tier 2 also lets `create_outgoing_link_triggers(conn, vocab)` run against that child with the seeds-only registry, the seeds-only `IN` list is FROZEN into the child's `sqlite_master` (search.rs:2327-2328) and every subsequent write through it — including some of the child's own — miscounts until the child's next boot recreates the triggers.

Angle A's compile-time discipline is blind to all of it: the argument was supplied, the types matched, the frame was converted. It was simply the wrong registry, obtained leniently. That is the same failure shape the 2026-08-02 triage already ruled on one screen above (`list_link_types`, link_types.rs preceding :585) — a lenient read feeding a write is how the user's vocabulary gets erased — and Angle A reintroduces it at a new site.
CONDITIONS: STRICT CONSTRUCTOR, NOT LENIENT. `registry_for_root` must NOT be the `.ok()/.unwrap_or_default()` shape the option proposes. It must go through `universe::read_persisted_json` (universe.rs:260-286) and return `Result<LinkTypeRegistry, _>`: NotFound is the only trustworthy emptiness; permission-denied, sharing violation, zero-length and parse failure all ABORT the routed write with a surfaced error. Nothing is written. This is not a new rule — link_types.rs' own 2026-08-02 doc-comment (preceding :585) already drew this exact line for reads that feed writes; the routed write is on the write side of it. Without this condition the option ships the silent failure above.; THE DELETIONS ARE NOT OPTIONAL AND MUST LAND IN THE SAME COMMIT AS THE FIRST CONVERSION. Delete `link_types::is_known_type` (link_types.rs:359) and `is_structural_type` (:369) — their bodies are already `LinkTypeRegistry` methods (`is_known` :171, `is_structural` :229) and exist only to read the global — and rename `snapshot()` (:498) to `active_universe_vocabulary()`. Add a CI grep that fails on `active_universe_vocabulary()` anywhere under the write path. Until this lands, every un-converted frame reads the global silently and the acceptance test cannot see it (A10). The option's own risk statement concedes this; it must be a gate, not an intention.; CLOSE THE `Owner` LITERAL. `OwnerVocabulary::of(&Owner { root, is_active })` compiles today because both fields are `pub` (federation/owner.rs:71, :73). Make them private with accessors, or add a private zero-size witness field so only `resolve_owner_in` (owner.rs:111) can mint one. Without this the newtype guards nothing — it is a promise wearing a type's clothes, and A5 walks straight through it.; RULE ON THE TRIGGERS EXPLICITLY — THIS IS THE ONE ANGLE A DOES NOT COVER. `search.rs:5964-5968` justifies the `if owns` gate with 'nobody writes through them, because the parent attaches a cUniverse read-only.' Phase 1.2 makes that false. Three sub-conditions: (a) a routed write must REFUSE, loudly, when the target child lacks `note_links_outgoing_ai/_ad/_au` in `sqlite_master` — a missing trigger means the outgoing aggregates are silently unmaintained (A6/A8); (b) if 1.2 instead lets the parent CREATE those triggers from the child's own vocabulary, PJ-232 must be explicitly amended, and condition #1 becomes load-bearing because a lenient registry would freeze seeds-only DDL into the child; (c) the ungated sky_links trigger block at search.rs:5531-5575 (`sx_new` read at :5540, outside every `if owns`) must be brought under the gate or its exemption documented — it currently contradicts the InitScope doc at search.rs:4577-4581.; DECIDE THE `is_built` GATE FOR A ROUTED WRITE. On a child with no `incoming_links` stamp, `incoming_links_backfill::is_built` (incoming_links_backfill.rs:73) returns false and search.rs:12754 skips `maintain_incoming_after_save` entirely — the routed write produces zero incoming aggregates and reports success. Either the routed path stamps/backfills the child first, or it refuses. Silently skipping is not an option.; FIX THE ACCEPTANCE CONDITION BEFORE TRUSTING IT. `aggregates_for` (vocab_harness.rs:72-106) must additionally observe `note_meta.outgoing_count / outgoing_link_types / outgoing_link_types_json / outgoing_top_rank`, `sky_nodes.stratum / maturity`, and `note_links.weight / confidence / traversal_count`; and `index_under_vocabulary` must route through `reindex_single_note` rather than calling `index_note` + `maintain_incoming_after_save` directly (vocab_harness.rs:142-158), so it crosses the same `is_built` gate production crosses. As written, removing the `#[ignore]` at vocab_harness.rs:275 can go green with 92 of ~117 sites still ambient.; RE-AUTHOR THE PINNED TEST DELIBERATELY, IN THE SAME COMMIT. `a_vocabulary_swap_reaches_back_into_an_already_open_database` (vocab_harness.rs:227) will stop compiling. The minimal repair — passing `&link_types::snapshot()` — turns it green while restoring the exact coupling LL-047 forbids. It must be rewritten into its positive form (a swap must NOT reach a routed write), and the commit must say so, or the pin silently inverts.; STATE THE ROUTED CONNECTION'S LIFETIME. Angle A specifies the vocabulary's lifetime exactly and is silent on the connection's. Follow the `reconcile_filesystem` precedent (search.rs:11976 — its own `Connection::open`, never stored in SearchState) so a universe switch wiping `state.db` (search.rs:11248) cannot invalidate a write in flight, and so `run_migrations_on`'s requirement to release the child's file lock before returning (federation/migrate.rs:171-175) is not broken by a held handle.; CONVERT TIER 2 IN THE SAME PASS, NOT 'NEXT'. The Whole-Ecosystem Fix Law names this exact shape. In particular `reindex_delete_note` (search.rs:12368, 6 call sites), `maintain_sky_after_save` (:2706), `recompute_after_link_status_change` (:11023 — whose doc at :11010-11022 states no note save ever heals it), and `libraries.rs:7490` (`rewrite_wikilinks_in_text` — the ONE vocabulary read whose answer reaches a `.md` FILE via `gate_rmw` -> `atomic_write`, write_gate.rs:667, currently fenced from children at libraries.rs:6963 and :6982-6986 and un-fenced the moment 1.2 routes a rename).

## ATTACK ON: ANGLE B — `Db<'a>`: the connection and its vocabulary as ONE value, with no `Deref` — verdict VIABLE_WITH_CONDITIONS
- [survives] 1. The 1500 ms debounced save fires on an ACTIVE-universe note while a routed write to a child is in flight: Boss types in a note in the active universe; the debounced save calls `constellation_search_reindex` → `reindex_single_note` (search.rs:12682), which takes `conn` from `state.db.lock()` (search.rs:12688-12689). Concurrently the Router holds a `UniverseDb` on a child. ATTACK ATTEMPTED: make one of them read the other's vocabulary. It does not land. Under Angle B the active path enters `with_active_db(conn, …)`, which snapshots the process-global ONCE at closure entry and hands `&LinkTypeRegistry` down; the routed path carries its own `vocab` field. The two are different files (different `Connection`s), so there is no writer-lock contention either — WAL is on and `busy_timeout=5000` is set for both (search.rs:4606, :4620). Even the nastiest concurrent mutator cannot reach the in-flight save: `list_link_types` (link_types.rs:585) is a READ-shaped IPC command the Links editor calls that nonetheless calls `set_active` at link_types.rs:588, and `ensure_search_db_ready`'s slow path calls `load_active` at search.rs:11606 — but both mutate the global AFTER the save's snapshot was taken, so the save keeps computing with one consistent vocabulary. This is a genuine improvement over the ruled-out set_active/restore design. RESIDUAL, not a kill: the option pins no DURATION. If `with_active_db` is placed inside `index_note` rather than around a whole operation, the bulk walk `index_library_recursive` (called at search.rs:12039 from `reconcile_filesystem`) re-snapshots per note, so a `set_active` from `list_link_types` mid-walk splits one walk across two vocabularies. That is today's behaviour too, so it is a not-fixed, not a regression.
    evidence: search.rs:12688-12689 (conn from state.db.lock), link_types.rs:588 (list_link_types calls set_active), search.rs:11606 (load_active on the ensure slow path), search.rs:4606 and :4620 (WAL + busy_timeout=5000), search.rs:12039 (bulk walk call site)
- [BREAKS] 2. The file watcher fires on a CHILD universe's file — and every routed open runs DDL in the child's database: REACHABILITY VERIFIED END TO END, not assumed. `loadLibraries()` (src/lib/libraries/store.ts:3949-3958) invokes `resolve_universe_libraries`, which resolves through `resolve_libraries_recursive` (universe.rs:602) — that function recurses into `universe.json`'s children and extends the list with THEIR libraries (universe.rs:640-649). The boot fan-out then installs a RECURSIVE watcher on every entry: `$libraries.map(lib => startWatchingLibrary(lib.id, lib.path))` (src/routes/+layout.svelte:2965-2966), and `handleChildUniverseChanged` re-runs the same loop on link (+layout.svelte:6295-6296). So Constellation watches child universes' directories TODAY. Today the event is fenced downstream: `reindex_changed_paths` loads the OWN set via `try_load_libraries` (search.rs:12970) and a foreign path resolves to no owning library, so it is silently skipped (search.rs:13009). Phase 1.2 removes exactly that fence. ATTACK: with the fence gone, each watcher batch needs a `Db` for a foreign root, and Angle B's only routed producer is `UniverseDb::open(owner)` → `init_db_scoped(db_path, scope, &vocab)`. `init_db_scoped` opens its own connection (search.rs:4603) and, on EVERY call in BOTH scopes, executes `DROP TRIGGER IF EXISTS note_links_sky_ai/_ad/_au` (search.rs:5531-5535) followed by `CREATE TRIGGER` for all three (search.rs:5541-5575) — that block sits at the function's top-level indentation, outside every `if owns` guard (the first is at search.rs:5640). It also unconditionally runs `drop_incoming_link_triggers` (search.rs:5977) and the ungated sky_nodes restore INSERT (search.rs:6276-6283). So a routed open is a SCHEMA WRITE against the child, not a cheap handle. Angle B specifies a bare constructor with no pool, no cache, no handle lifetime — the word 'pool' in 'routed context pool' has no counterpart in this option. A git-pull touching 300 child files, or a Syncthing burst, becomes 300 DROP/CREATE TRIGGER transactions in a universe another process may own (`universe_lock::activate` logs 'NOT ENFORCED YET (MIG-111 Phase 1.4)' and proceeds unlocked — universe_lock.rs:246).
    evidence: src/lib/libraries/store.ts:3949-3958; universe.rs:602 and :640-649; src/routes/+layout.svelte:2965-2966 and :6295-6296; search.rs:12970 and :13009 (today's fence); search.rs:4603, :5531-5535, :5541-5575, :5640, :5977, :6276-6283; universe_lock.rs:246
- [BREAKS] 3. A backfill tick runs on a background thread mid-routed-write — and Angle B hands it `active_universe_snapshot()` with the compiler's blessing: The backfills are spawned from `ensure_search_db_ready` (`sky_backfill::maybe_schedule` search.rs:11664, `links_backfill::maybe_schedule` search.rs:11672, `incoming_links_backfill::maybe_schedule` search.rs:11699) and again from `on_link_vocabulary_changed` (search.rs:2786, :2792). Against the ACTIVE database they are correct under Angle B, because they keep calling the renamed `active_universe_snapshot()`. THE ATTACK IS THE OPPOSITE DIRECTION, and it is structural rather than concurrent. Angle B's signature list adds `vocab` to the four `-> String` GENERATORS but NOT to the functions that call them. I read those three bodies: `recompute_range(conn: &Connection, after_path: &str, last_path: &str)` (links_backfill.rs:245-251) interpolates `outgoing_aggregate_assignments` at :248; `recompute_incoming_range(conn, after, last)` (links_backfill.rs:304-310) interpolates `incoming_aggregate_assignments` at :307; `recompute_sky_range(conn, after, last)` (links_backfill.rs:356-363) interpolates `stratum_sql_expr()` / `maturity_sql_expr()` at :359-360. None has a vocabulary in scope and Angle B does not give one. The ONLY expression that compiles is `active_universe_snapshot()`. So the migration's own mechanics WRITE the wrong-vocabulary read into six backfill functions across four files (add `sky_backfill.rs:283/388/399` and `name_fold_backfill.rs:157/173/178`), converting a wrong-by-omission ambient read into a wrong-by-DECLARATION one that reads as deliberately migrated. That is worse to find later, and it is on the exact surfaces whose only tie to the active universe is an incidental path helper: `incoming_links_backfill::run` opens its own handle at `crate::search::db_path(app)`, and the write-side gate `is_built` (incoming_links_backfill.rs:73-81) reads the schema version ONLY — by design (documented at :52-72) — so it can never refuse a foreign-vocabulary write.
    evidence: links_backfill.rs:245-251, :304-310, :356-363 (all three take conn only, no vocabulary); incoming_links_backfill.rs:73-81 (is_built reads version only); incoming_links_backfill.rs:49 and links_backfill.rs:99 (the two fingerprint gates on the global); search.rs:11664, :11672, :11699, :2786, :2792
- [BREAKS] 4. A universe SWITCH happens while a routed `UniverseDb` handle is held — and Angle B creates a SECOND, independently-timed vocabulary source: `UniverseDb::open` refuses `owner.is_active` — but that check happens once, at open. I read the teardown: `invalidate_search_state` (search.rs:11228-11284) clears ONLY `state.db` (:11248), `state.read_db` (:11270), `state.federated_conn` (:11277) and `state.federation` (:11282). A `UniverseDb` is an owned value on the Router's stack; nothing in that function can see it, and `federation_generation` (bumped at :11236) is the app's only staleness signal — a `Db<'a>` carries no generation. Boss then switches TO the child universe. `ensure_search_db_ready` runs `load_active` (search.rs:11606) and `init_db(&path)` (search.rs:11607) on the SAME file, so that universe now has two read-write connections; nothing prevents it (`universe_lock::activate` is explicitly NOT ENFORCED — universe_lock.rs:246). Now the kill: Boss edits the link types of what is now the active universe. `save_universe_link_types` writes the file, calls `set_active` (link_types.rs:554) and, on a fingerprint change (link_types.rs:562), `on_link_vocabulary_changed` (search.rs:2771), which calls `create_outgoing_link_triggers(conn)` at search.rs:2780 — and that function DROPs and re-CREATEs the outgoing trigger bodies from the CURRENT registry (search.rs:2290, :2327-2328). The database's SQL-side vocabulary is now V2. The still-held `UniverseDb.vocab` is V1, read from disk at open, with no invalidation path anywhere in the option. The next routed write parses `note_links.link_type` under V1 while the triggers it fires compute `outgoing_count / outgoing_link_types / outgoing_link_types_json / outgoing_top_rank` under V2. Two vocabularies inside one write, one database, row counts identical, no error. THIS IS NOT A PRE-EXISTING BUG — today both halves come from the one global and CANNOT disagree. Angle B introduces the divergence axis by making the Rust half read live from disk while leaving the SQL half frozen in `sqlite_master`. The same divergence is reachable without any switch, by a hand-edit of `.constellation/link-types.json` (a plain JSON file, under 'file over app') or a sync of that file from another device while the child is not active.
    evidence: search.rs:11228-11284 (invalidate_search_state touches only SearchState fields), search.rs:11236 (generation bump), search.rs:11606-11607, universe_lock.rs:218 and :231-255 and :246, link_types.rs:554 and :562, search.rs:2771-2793, search.rs:2290 and :2327-2328
- [BREAKS] 5. A new call site is added next year by someone who has not read LL-047: MIXED, and the mix is the finding. For anything reached through the bundle it IS a compile error: `index_note(db: Db<'_>, …)` will not accept a `&Connection`, and deleting `link_types::is_known_type` (link_types.rs:359) and `is_structural_type` (link_types.rs:369) makes every stale reader an unresolved name. That half of the claim holds. But the option leaves TWO idioms that compile and are silently wrong on a routed path, and both are pre-seeded in-tree so the newcomer copy-pastes them. (a) `active_universe_snapshot()` — the option itself leaves 14 production sites on it, and attack 3 shows the migration ADDS six more inside the backfills. A name is a speed bump; `snapshot()` was already named after the thing it returns and 26 sites read it wrongly anyway (vocab_harness.rs:5-8). (b) `db.conn()` — any new helper written as `fn foo(conn: &Connection)` compiles when handed `db.conn()`, and if it needs the vocabulary the only in-scope answer is `active_universe_snapshot()`. There is precedent for exactly this drift: `link_types::structural_frontmatter_targets` (link_types.rs:385) ALREADY threads `&LinkTypeRegistry` internally into `resolve_wikilink_type` at :424 — the parameter pattern exists and is one frame from complete — yet the function still calls `snapshot()` itself at :390. Someone already had the right shape in hand and stopped one frame short. Verdict for the newcomer: compile error if they touch the write path, SILENT WRONG VALUE if they add a new derived-view recompute, which is the class this migration is about.
    evidence: link_types.rs:359, :369 (the two deletions that DO force errors), link_types.rs:385 and :390 and :424 (the existing half-threaded precedent), links_backfill.rs:245/304/356 (the six sites the migration itself seeds with active_universe_snapshot), federation/vocab_harness.rs:5-8
- [BREAKS] 6. The child's search.db has NEVER been opened by the child's own process — no scope Angle B can pass is correct: Reachable by the shipped federation path: `federation::migrate::run_migrations_on` calls `init_db_schema_only` (federation/migrate.rs:169) on a linked universe whose schema is too old to attach, i.e. a child that this process has schema-migrated but that has never run its own `init_db` as Active. `UniverseDb::open` must choose a scope, and I checked both. (A) `InitScope::ForeignSchemaOnly` → `owns = false` (search.rs:4602) → `if owns { create_outgoing_link_triggers(&conn)?; }` at search.rs:5969-5971 is SKIPPED, so `note_links_outgoing_ai/_ad/_au` DO NOT EXIST in that database. Angle B's routed write then does everything right in Rust — `index_note` parses with the child's vocabulary, `note_links` rows are correct, `maintain_incoming_after_save` (search.rs:2661) computes correct incoming values — and `note_meta.outgoing_count / outgoing_link_types / outgoing_link_types_json / outgoing_top_rank` are NEVER WRITTEN, because the only thing that maintains them at save time is a trigger that is not there (`outgoing_aggregate_assignments` is not called on the save path at all; it exists only in trigger bodies at search.rs:2327-2328 and in `links_backfill::recompute_range`). Correct row counts, correct values everywhere the harness looks, no error. (B) `InitScope::Active` → `owns = true` → the routed open ALSO runs `mig003_step3_soft_rebackfill` (search.rs:6312-6319), which the code's own comment says re-indexes rows and 'for a file carrying no identity key at all, write[s] frontmatter into the child universe's `.md`' (search.rs:6308-6311), and MIG-003 Step 4, which 'RENAMES `.md` files' (search.rs:6334-6335). The parent's process mutating a child universe's note files. Angle B defines exactly two producers and neither can pick a correct scope, because the option changes only the DDL GENERATION and never touches the `owns` gate that decides whether that DDL runs at all.
    evidence: federation/migrate.rs:169; search.rs:4602 (owns), :5969-5971 (outgoing triggers gated off), :2327-2328 (the only place outgoing_aggregate_assignments reaches sqlite_master), :6308-6319 (Step 3 writes .md frontmatter, owns-gated), :6334-6335 (Step 4 renames .md files, owns-gated), :2661 (incoming is Rust-side, so it DOES land — which is what makes the outgoing hole invisible)
- [survives] 7. The child's link-types.json is unreadable / locked / corrupt at routed-open time: Angle B's strictness claim is TRUE — I verified `read_persisted_json` (universe.rs:260-290): NotFound alone returns `Ok(None)` (:267); a permission/sharing error returns `Err(Unreadable)` (:268-274); a zero-length file returns `Err(Corrupt)` (:277-282); a parse failure returns `Err(Corrupt)` (:283-289). So the option does NOT inherit `read_deltas`'s leniency, and the `Ok(None)` case is correct rather than dangerous: a child that never customised its vocabulary has no file, and the 8 seeds genuinely ARE its vocabulary (`merge` always starts from `seeds()`, link_types.rs:121). THE ATTACK THAT DOES LAND IS THE ASYMMETRY IT CREATES. `read_deltas` (link_types.rs:514-518) is lenient by construction — `std::fs::read_to_string` error → `Vec::new()`, and `serde_json::from_str(...).unwrap_or_default()` → `Vec::new()` on corrupt — so the CHILD's own process, reading the same file in the same second, indexes happily with the 8 seeds while the parent's routed write refuses. And the triggering condition is not hypothetical: the codebase records it as observed, at link_types.rs:575-578 — 'link-types.json is held for a second by a sync tool or antivirus'. Consequence to rule on: a routed write that fails leaves the child's index un-updated. Whether that is loud depends on the caller, and `reindex_single_note`'s three maintenance steps are already best-effort (`maint.incoming_failed` / `sky_failed` at search.rs:12756, :12767 are counted, not raised). A transient AV lock that turns into 'this child universe silently stopped indexing' is the failure shape to design against.
    evidence: universe.rs:263-289 (strict: Ok(None) only for NotFound; Err for unreadable/empty/parse), link_types.rs:514-518 (read_deltas lenient), link_types.rs:121 (merge always seeds), link_types.rs:575-578 (the AV/sync lock recorded as a real observed condition), search.rs:12756 and :12767 (best-effort maintenance outcomes)
- [BREAKS] 8. THE TRIGGERS — Angle B's `init_db_scoped(path, scope, vocab)` parameter changes no observable value for any child universe: This is the attack the option is most confident about ('that single parameter fixes all five registry-generated DDL sites at their source') and it does not survive. I checked each of the five. FOUR OF THEM ARE VOCABULARY-INVARIANT, proven from source rather than assumed: `note_links_sky_ai/_au` (search.rs:5540 → `sx_new`), `note_meta_sky_ai` (search.rs:5657-5658), `note_meta_sky_stratum_au` (search.rs:5910) and `note_meta_sky_maturity_au` (search.rs:5956) all interpolate ONLY `structural_not_in_clause`, which filters on `t.structural` (link_types.rs:268-280). And the structural set cannot vary between universes: `seeds()` marks `structural: true` on exactly `contains` and `parent` (link_types.rs:93-94) and `false` on all eight cognitive seeds (link_types.rs:76); `merge` starts from `seeds()` (link_types.rs:121) and can only override or insert, never delete; it forces `structural = false` for a cognitive-seed delta (:131), `structural = true` for a `STRUCTURAL_SEED_IDS` delta (:148 — and that constant is the fixed `&["parent", "contains"]` at :61), and `structural = false` for EVERY custom delta (:159, `// custom types are cognitive`). So `structural_not_in_clause` emits the same clause in every universe; only the order of two literals inside the parens can differ, which is semantically identical SQL. THE FIFTH — `create_outgoing_link_triggers` (search.rs:2286), the only generator whose output genuinely varies (it reads `cognitive_ids`-derived lists: `sql_in_list_cognitive` link_types.rs:241, `sql_rank_case_cognitive` :253, `cognitive_sentinel_rank` :292, all of which include every user-defined type) — IS GATED OFF for a foreign database at search.rs:5969-5971, and Angle B does not change that gate. Net: for a routed open, the new `vocab` parameter is a no-op on four sites and unreachable on the fifth. Worse, the gate's justification is a premise Phase 1.2 deletes: search.rs:5966-5968 reads 'nobody writes through them, because the parent attaches a cUniverse read-only.' And the deeper structural point stands regardless of the gate: trigger bodies live in `sqlite_master` and fire on INSERT no matter what the Rust holds, so Angle B controls the Rust half and the SQL half only at the moment of a re-CREATE it does not perform. The option's own `what_it_cannot_do` #2 concedes existing bodies are not audited — but it does not name the case where the bodies are ABSENT (attack 6), which is the one that produces silently missing aggregates.
    evidence: search.rs:5540, :5657-5658, :5910, :5956 (all four DDL sites use only structural_not_in_clause); link_types.rs:268-280, :61, :76, :93-94, :121, :131, :148, :159 (the structural set is invariantly {contains, parent}); link_types.rs:241, :253, :292 (the only genuinely varying generators); search.rs:2286, :2327-2328, :5969-5971 (the varying one is owns-gated off), search.rs:5966-5968 (the read-only premise 1.2 deletes)
- [survives] 9. Does the option make `a_vocabulary_swap_reaches_back_into_an_already_open_database` FAIL?: SIGNAL, NOT DEFECT — and the signal is that the pin covers half the system after this change. I read the test (federation/vocab_harness.rs:227-256). It sets `set_active(deltas(&["refutes"]))` (:233), opens `init_db` (:234), then `set_active(deltas(&[]))` (:241), then calls `index_note(&conn, …)` (:245), and asserts the swap REACHED the write: `t.contains("refutes::") || ty == "associative"` (:250). Under Angle B `index_note` takes `Db<'_>`, so line 245 no longer compiles and the test MUST be rewritten. Rewritten the natural way — `with_active_db(&conn, |db| index_note(db, …))` — the snapshot is taken after the swap, `refutes` is unknown, the target keeps its `refutes::` prefix, and the assertion PASSES. So Angle B keeps it green, correctly: the ACTIVE path should follow the global. What changes is scope. The assertion at :249-255 is precisely what a connection-bound vocabulary would break, and Angle B's routed producer binds the vocabulary at `UniverseDb::open` — i.e. exactly the property this test says does not hold. That is explicitly permitted: the harness doc says a routed write must carry its vocabulary 'threaded through the call, or held per-connection' (vocab_harness.rs:224, echoing :45-46). So the test remains true of the active half and becomes false-by-design of the routed half, and nothing re-pins the routed half's own timing rule. Second-order risk worth naming: the rewrite is discretionary. A rewriter who chooses to hoist `with_active_db` ABOVE line 241 turns this test green for the wrong reason, and the only thing stopping that is the same attention the whole design is trying to replace.
    evidence: federation/vocab_harness.rs:227-256 (the test), :233, :234, :241, :245, :249-255 (the assertion), :224 and :45-46 (the doc that blesses 'held per-connection'), :275-276 (the acceptance test still panicking)
WORST SILENT FAILURE: THE WORST SILENT FAILURE ANGLE B STILL PRODUCES — a routed write into a child universe whose search.db was schema-migrated by the parent and never opened by its own process, which writes CORRECT values everywhere the acceptance test looks and leaves four columns permanently unwritten.

Path, every step verified this session:
1. A cUniverse is linked whose schema is too old to attach. `federation::migrate::run_migrations_on` calls `init_db_schema_only(cu_db_path)` (federation/migrate.rs:169) → `init_db_scoped(path, ForeignSchemaOnly)` (search.rs:4597-4598) → `owns = false` (search.rs:4602).
2. `if owns { create_outgoing_link_triggers(&conn)?; }` (search.rs:5969-5971) is SKIPPED. The child's `sqlite_master` therefore contains NO `note_links_outgoing_ai / _ad / _au`. Angle B does not change this gate — it changes only how the DDL is GENERATED.
3. The Router lands a routed write. Angle B works exactly as advertised for the Rust half: `index_note(db, …)` parses with the child's vocabulary, so `note_links.link_type` is right, the `edges` are right, `maintain_incoming_after_save` (search.rs:2637, generator at :2661) writes the right `incoming_count` and `incoming_link_types`.
4. `note_meta.outgoing_count`, `outgoing_link_types`, `outgoing_link_types_json` and `outgoing_top_rank` are never touched. `outgoing_aggregate_assignments` (search.rs:2242) is NOT called on the save path at all — it reaches a database only through the trigger bodies (search.rs:2327-2328) and through `links_backfill::recompute_range` (links_backfill.rs:248). Neither runs here.

Why nothing surfaces it:
- Row counts are correct. `note_links` has exactly the right rows with the right values.
- No error is raised anywhere; the missing triggers are an absence, not a failure.
- The acceptance harness cannot see it. `aggregates_for` (federation/vocab_harness.rs:73-106) reads exactly four things — `COUNT(*) FROM note_links` (:81), `(source_path, target_name, link_type)` (:83-86), `note_meta.incoming_count` (:91-93), `note_meta.incoming_link_types` (:98-100). It reads NO outgoing column. So `routed_write_must_match_the_owners_vocabulary` goes green over a child whose outgoing aggregates are stale or NULL.
- The self-heal cannot re-arm. `links_backfill::is_needed` (links_backfill.rs:87-100) returns false when the version is current AND the stored `links_vocab` fingerprint matches — and if the routed write never ran the backfill, the child's stamp is still the CHILD's own fingerprint, so on the child's next boot the gate says "nothing needed" and the wrong rows are never recomputed.

TWO MORE SILENT FAILURES ANGLE B ADDS RATHER THAN REMOVES:
(a) The V1/V2 split of attack 4 — `UniverseDb.vocab` is read live from disk at open with no invalidation, while the same database's trigger bodies are frozen from whenever `create_outgoing_link_triggers` last ran (search.rs:2290, :2780). One routed write then computes `note_links` under one vocabulary and `note_meta.outgoing_*` under another, in one transaction, with correct counts. TODAY THIS CANNOT HAPPEN — both halves come from the single process-global. Angle B creates the divergence axis.
(b) Six backfill recompute functions gain `active_universe_snapshot()` by construction (links_backfill.rs:245, :304, :356; sky_backfill.rs:283/388/399; name_fold_backfill.rs:157/173/178), because Angle B parameterises the generators but not their callers, and those callers hold only a `&Connection`. A wrong-by-omission read becomes a wrong-by-declaration one that reads as migrated.

Also invisible to the harness, and equally vocabulary-decided: `link_row_is_preserved`'s `structural` argument (search.rs:8485) and the INSERT-shape choice (search.rs:8561) govern whether an existing edge's earned `weight` / `confidence` / `traversal_count` / archived status survives a re-index — the CLAUDE.md data that lives ONLY in search.db — and the harness's `edges` tuple selects just `(source_path, target_name, link_type)`.
CONDITIONS: 1. RULE ON `owns` EXPLICITLY, OR THE WHOLE DDL PARAMETER IS DEAD WEIGHT. Split the single `owns` flag (search.rs:4602) into two independent decisions: (a) may this init WRITE registry-generated DDL — which becomes YES once the vocabulary is a parameter, so `create_outgoing_link_triggers` (search.rs:5969-5971) runs for a routed open with the CHILD's vocabulary; and (b) may this init TOUCH THE USER'S FILES — which stays NO for any non-active universe, keeping MIG-003 Step 3 (search.rs:6312-6319, writes `cid_cn:` frontmatter) and Step 4 (search.rs:6334-6335, RENAMES `.md` files) gated off. Without this split there is no scope `UniverseDb::open` can pass that is correct, and the new `vocab` parameter changes no observable value for any child (attacks 6 and 8).; 2. THREAD THE VOCABULARY INTO THE SIX BACKFILL RECOMPUTE FUNCTIONS, NOT JUST THE GENERATORS. `recompute_range` (links_backfill.rs:245), `recompute_incoming_range` (:304), `recompute_sky_range` (:356), `sky_backfill::process_batch` (sky_backfill.rs:283/388/399) and `name_fold_backfill::run` (:157/173/178) must take the vocabulary too, or the migration writes `active_universe_snapshot()` into all six and calls it done (attack 3).; 3. GIVE THE HANDLE AN INVALIDATION RULE, AND STATE IT. `UniverseDb.vocab` is a snapshot with no expiry; `invalidate_search_state` (search.rs:11228-11284) cannot see it and `federation_generation` (search.rs:11236) is active-universe-shaped. At minimum: re-check `resolve_owner` / `is_active` per write rather than per open, and forbid a handle from outliving a `federation_generation` bump. Otherwise a universe switch, or `save_universe_link_types` → `on_link_vocabulary_changed` (link_types.rs:562 → search.rs:2780), leaves the Rust half on V1 and the SQL half on V2 (attack 4).; 4. RECONCILE THE TWO VOCABULARY SOURCES INSTEAD OF ASSUMING THEY AGREE. `UniverseDb::open` should compare the vocabulary it just read from disk against the one baked into the child's `sqlite_master` — the `fingerprint()` machinery already exists (link_types.rs:300-311) and both stamps are already persisted (`links_vocab` links_backfill.rs:121; `incoming_links_vocab` incoming_links_backfill.rs:88). On mismatch, re-create the DDL under condition 1 before writing. Angle B currently makes the two halves independently timed and never checks them against each other.; 5. DECIDE THE HANDLE'S LIFETIME AND CACHING BEFORE ANY WATCHER ROUTING. Every `UniverseDb::open` executes `DROP TRIGGER` + `CREATE TRIGGER` on the child (search.rs:5531-5575, ungated in both scopes) plus `drop_incoming_link_triggers` (search.rs:5977) and the sky_nodes restore INSERT (search.rs:6276-6283). A per-watcher-event open turns a git-pull into hundreds of schema-write transactions against a universe another process may hold (`universe_lock::activate` is NOT ENFORCED — universe_lock.rs:246). Angle B specifies a constructor, not a pool; the pool is the part still missing (attack 2).; 6. WIDEN `Aggregates` BEFORE REMOVING THE `#[ignore]`, OR THE ACCEPTANCE TEST CERTIFIES A CORRUPT CHILD. Add `note_meta.outgoing_count / outgoing_link_types / outgoing_link_types_json / outgoing_top_rank`, `sky_nodes.stratum / maturity`, and `weight / confidence / traversal_count` to the `edges` tuple (federation/vocab_harness.rs:59-106), and route `index_under_vocabulary` (:135) through `maintain_sky_after_save` (search.rs:2706), which it does not currently call. As written, the four columns of the primary silent failure above are outside the test's field of view.; 7. RE-PIN THE ROUTED HALF'S TIMING RULE. `a_vocabulary_swap_reaches_back_into_an_already_open_database` (federation/vocab_harness.rs:227) must be rewritten to compile, and after the rewrite it pins only the ACTIVE path. Add a sibling test asserting the routed handle does NOT follow a mid-flight `set_active` — the property Angle B actually introduces — so the routed half has a pin of its own rather than inheriting a blessing from a doc comment (vocab_harness.rs:224).; 8. ADD A REGRESSION TEST FOR THE STRUCTURAL INVARIANT THE DESIGN LEANS ON. Four of the five DDL sites are safe only because `merge` forces `structural = false` for every custom type (link_types.rs:159) and `true` only for `STRUCTURAL_SEED_IDS` (link_types.rs:148, :61). That is a property of one function, not a declared contract; the moment a user-definable structural type ships, four more DDL sites silently join the varying family. A test that fails when `merge` stops forcing it is the cheapest thing on this list.; 9. FIX THE UNGATED SKY-LINK DDL REGARDLESS OF WHICH ANGLE WINS. `search.rs:5531-5575` sits outside every `if owns` guard (the first is at :5640), so `init_db_schema_only` (federation/migrate.rs:169) already writes the parent's `structural_not_in_clause` into a child's `note_links_sky_ai/_au` bodies today. Harmless only because the structural set is invariant — i.e. by accident, not by the gate. Either gate it or document why it is deliberately exempt; the `InitScope` doc at search.rs:4577-4581 currently claims it is skipped, and it is not.

## ATTACK ON: The Owner Scope — one constructor, and no ambient door left to walk through (MIG-111 Phase 1.2) — verdict VIABLE_WITH_CONDITIONS
- [survives] A1 — the 1500 ms debounced save fires for an ACTIVE-universe note while a routed child write is in flight: User edits a note in the active universe; the 1500 ms debounce fires `constellation_search_reindex` (search.rs:12251) -> `reindex_single_note` (search.rs:12682) on the IPC thread. Concurrently a routed write to a child is mid-`index_note`. ATTACK: does either read the other's vocabulary? NO, and there is no window to hit. Under the option each write holds an OWNED `LinkTypeRegistry` — `snapshot()` already returns a clone, not a guard (link_types.rs:498-503) — and the routed arm never calls `set_active` (link_types.rs:481). The two writes share no mutable state: the active arm borrows `state.db` (search.rs:12688), the routed arm holds its own `Connection`. I also tried the sharper version: `list_link_types` (link_types.rs:585-591) calls `set_active` at :588 as a side effect of merely opening the Links editor, so TODAY a `set_active` can land BETWEEN `parse_link_body`'s read (search.rs:7244) and `emit_frontmatter_links`' read (search.rs:7371) inside ONE `index_note` — a torn vocabulary within a single note. The option's single per-write snapshot removes that too. This attack fails against the option and lands against the status quo. RESIDUAL, non-vocabulary: `reindex_single_note` takes `state.db.lock()` as its FIRST statement (search.rs:12688). If the routed arm is added after that line rather than before it, a routed write to a child holds the ACTIVE universe's writer lock across a foreign-file open and write — a stall, not a wrong value.
    evidence: src-tauri/src/link_types.rs:498-503 (snapshot returns an owned clone); src-tauri/src/link_types.rs:481-485 (set_active is the only mutator); src-tauri/src/link_types.rs:588 (list_link_types mutates the global on an editor open); src-tauri/src/search.rs:12688 (state.db.lock() is the first statement of reindex_single_note); src-tauri/src/search.rs:7244 and 7371 (two separate call-time global reads inside one index_note)
- [survives] A2 — the file watcher's adopt path fires on a CHILD universe's file: Verified reachable and verified currently fenced — but the fence is what the option leaves unguarded. The watcher watches recursively (watcher.rs:138 `.watch(&path, RecursiveMode::Recursive)`) and the app watches EVERY library in the federation-recursive set — stated verbatim in source: 'the app watches every library in the recursive set, so any path a federated library owned was indexed straight into the active universe's index' (search.rs:12961-12963). So a Syncthing/Git update inside a linked universe DOES raise `library-changed` today. The PJ-207 §8 fence stops it downstream: `reindex_changed_paths` loads `try_load_libraries(&app)` — the OWN set (search.rs:12971) — so `library_name_for_path(&libs, p)` (search.rs:13008) returns None for a child path and `reindex_single_note` is never called. THE FINDING: the option constructs the WriteScope INSIDE `reindex_single_note`, i.e. downstream of that fence. So the watcher path cannot exercise routing at all — and, worse, whoever later swaps `try_load_libraries` for `load_all_libraries` at search.rs:12971 to enable child adoption gets routed child writes everywhere with ZERO compile errors and zero prompts, because `index_note` already resolves the owner itself. The option makes the vocabulary unforgettable and makes the FENCE REMOVAL invisible. It is explicit about this for the rename cascade ('the parameter now EXISTS, so whoever removes the fence must decide') and silent about it here, where the fence is one identifier.
    evidence: src-tauri/src/watcher.rs:138 (recursive watch); src-tauri/src/search.rs:12960-12968 (the PJ-207 §8 comment stating the app watches the recursive set); src-tauri/src/search.rs:12971 (`let libs = crate::libraries::try_load_libraries(&app)?` — the one-identifier fence); src-tauri/src/search.rs:13008-13010 (library_name_for_path gate before reindex_single_note)
- [survives] A3 — a backfill tick runs on a background thread mid-routed-write: All three backfills are hard-bound to the ACTIVE database and read the ambient registry, so they cannot contaminate a routed write and a routed write cannot contaminate them: `links_backfill::maybe_schedule` and `sky_backfill::maybe_schedule` operate on `state.db`; `incoming_links_backfill::run` opens its own handle at `crate::search::db_path(app)`, which resolves through `universe::active_constellation_dir` — ambient-active by construction. Under the option they hoist `active_universe_snapshot()`, which is the correct vocabulary for the database they are on. No shared mutable state with a routed scope, no lock overlap (different files). This attack does not break the option. WHAT IT DOES EXPOSE (the option's admitted Residual 2, verified): `links_backfill::is_needed` compares a fingerprint STORED IN THE DATABASE against `crate::link_types::snapshot().fingerprint()` (links_backfill.rs:99). After a routed write into a child, the child's `links_vocab` stamp is still the CHILD's own fingerprint, so on the child's own next boot `is_needed` reads FALSE and nothing re-materializes. Any gap the routed write left is never healed by the owner.
    evidence: src-tauri/src/links_backfill.rs:99 (`stored_vocab_fingerprint(conn) != crate::link_types::snapshot().fingerprint()`); src-tauri/src/links_backfill.rs:106-114 (version_current, the first clause that makes is_needed short-circuit FALSE); src-tauri/src/search.rs:1465-1468 (db_path is ambient-active); src-tauri/src/universe.rs:64-72 (constellation_dir / active_constellation_dir)
- [survives] A4 — a universe SWITCH happens while a routed handle is held: The routed scope is immune on the vocabulary axis: it holds an OWNED `LinkTypeRegistry`, so `ensure_search_db_ready`'s `link_types::load_active(app)` (search.rs:11606) replacing the global mid-write reaches nothing. The scope also holds an OWNED `Connection`, so `invalidate_search_state` setting `state.db = None` cannot pull the handle out from under it. THE REAL BITE IS A PER-CONNECTION PRAGMA, NOT THE VOCABULARY. If the switch makes the CHILD the active universe, that file now has two writers — the routed owned connection and the new `state.db` — and WAL + busy_timeout serialise them. But `recursive_triggers` is a CONNECTION-level setting, and search.rs:4620-4641 documents in detail what happens when it is OFF: `note_meta_sky_ai` writes to sky_nodes and SQLite then SILENTLY SKIPS the chained triggers, leaving stratum and maturity NULL on the new row. If the routed connection's PRAGMA batch omits `PRAGMA recursive_triggers=ON`, the same INSERT produces different derived state depending on which connection made it — correct row counts, NULL derived columns, no error. The option's spec says only 'the PRAGMA batch', and the template it names (reconcile_filesystem's walk connection) DOES set it — so a faithful implementation is safe and a sloppy one is silently wrong, invisibly to the type system the option is built on.
    evidence: src-tauri/src/search.rs:4620-4646 (the recursive_triggers rationale: 'edit-save on a note leaves stratum + maturity NULL on the new row'); src-tauri/src/search.rs:11976-11985 (reconcile_filesystem's walk_conn sets `PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA recursive_triggers=ON;` then register_fts5_tokenizer); src-tauri/src/search.rs:11606-11607 (load_active immediately before init_db)
- [BREAKS] A5 — a new call site is added next year by someone who has not read LL-047: THE OPTION'S CENTRAL CLAIM DOES NOT HOLD. Deleting `is_known_type` (link_types.rs:359) and `is_structural_type` (link_types.rs:369) closes three named doors. It does not close the door the option must keep open for its own ~12 ambient sites: `snapshot()` stays `pub` (link_types.rs:498), `LinkTypeRegistry` is a `pub struct` (link_types.rs:100-103) and every predicate on it is `pub` — `is_known` (link_types.rs:171), `is_structural` (link_types.rs:229), `is_link_type_value`, `structural_not_in_clause` (link_types.rs:268). So next year's developer writes `link_types::active_universe_snapshot().is_known(&head)` — ONE line, compiles clean, no warning, no lint — and if that site is ever reached by a routed write it produces the H1 failure exactly as before. The compiler cannot tell the ambient registry from the owner's registry: they are the same type. The option's own cannot-forget section concedes residual 3 as 'the ~12 remaining ambient reads'; the honest statement is stronger — nothing prevents an unbounded number of NEW ambient reads, and the rename to `active_universe_snapshot()` is legibility in the diff, not enforcement. VERDICT ON THIS ATTACK: the mistake is a SILENT WRONG VALUE, not a compile error. That is a materially weaker guarantee than the option advertises, and it is the one claim the whole 'structure, not a promise' framing rests on.
    evidence: src-tauri/src/link_types.rs:498-503 (`pub fn snapshot() -> LinkTypeRegistry`); src-tauri/src/link_types.rs:100-103 (`pub struct LinkTypeRegistry`); src-tauri/src/link_types.rs:171 (`pub fn is_known`); src-tauri/src/link_types.rs:229 (`pub fn is_structural`); src-tauri/src/link_types.rs:268 (`pub fn structural_not_in_clause`)
- [BREAKS] A6 — the child's search.db has NEVER been opened by the child's own process: 'search.db missing' is a REAL, handled state, not a hypothetical: attach.rs:156-160 checks `if !cu_db_path.exists()` and warns 'search.db missing', skipping that cUniverse. Now route a write to such a child. `WriteScope::for_note` does `Connection::open(owner.root/.constellation/search.db)` — and `Connection::open` CREATES the file (this is exactly how `init_db_scoped` at search.rs:4603 manufactures a new database). Result: a ZERO-TABLE search.db now exists at the child root. `index_note`'s first statement against `note_meta` fails with 'no such table' — loud for that one write, which is fine. THE FILE PERSISTS. On the next boot `cu_db_path.exists()` is now TRUE, so attach.rs:161 proceeds: ATTACH succeeds (a valid empty DB), `verify_schema` fails on `PRAGMA {alias}.table_info(note_meta)`, the caller matches `schema_incomplete` (attach.rs:167) and calls `run_migrations_on` (attach.rs:172) -> `init_db_schema_only` (migrate.rs:169) with `owns = false` (search.rs:4602). The child's ENTIRE schema is now built by the PARENT's process, permanently missing the five `if owns`-gated sky triggers and the outgoing-aggregate triggers, until the child is next opened as its own active universe. A single failed routed write has converted a clean 'skipped, warned' state into a silently degraded child schema. The option's constructor is the sole place this can be prevented, and its spec does not mention existence-checking the target.
    evidence: src-tauri/src/federation/attach.rs:156-160 (`if !cu_db_path.exists() { ctx.warn(cu_root.clone(), "search.db missing"); continue; }`); src-tauri/src/search.rs:4603 (`Connection::open(path)` is what creates a new search.db); src-tauri/src/federation/attach.rs:167-172 (schema_incomplete -> run_migrations_on); src-tauri/src/search.rs:4601-4602 (init_db_scoped, `let owns = scope == InitScope::Active`)
- [survives] A7 — the child's link-types.json is unreadable / locked / corrupt at routed-open time: I attacked the inherited-leniency claim and it holds. `read_deltas` (link_types.rs:514-518) IS lenient — `let Ok(data) = std::fs::read_to_string(&path) else { return Vec::new(); }` and `serde_json::from_str(...).unwrap_or_default()` — so a locked file yields the 8 seeds and a routed write under it would collapse every custom-typed edge in the child to `associative` with correct row counts. That is the H1 failure with a different cause. The option does NOT go through `read_deltas`; it goes through `universe::read_persisted_json`, which I read in full and which is strict on every branch that matters: only `ErrorKind::NotFound` returns `Ok(None)`; permission-denied / sharing-violation / IO returns `Err(Unreadable)` with the comment 'Refusing to treat it as empty'; a zero-length file returns `Err(Corrupt)`; a parse failure returns `Err(Corrupt)`. A missing file correctly yields the 8 seeds via `merge(vec![])`, which is the right answer for a child with no custom types. I also checked the merge path for a silent-drop hole — `merge` sanitizes ids and drops empties (link_types.rs:120-125) — but the child's own process runs the identical `merge`, so both sides agree and there is no divergence. This is the option's strongest result: it converts today's silent seeds-flavoured degradation into a refusal, matching `resolve_owner`'s fail-closed rule.
    evidence: src-tauri/src/universe.rs:260-289 (read_persisted_json: NotFound -> Ok(None); Err -> Unreadable 'Refusing to treat it as empty'; empty -> Corrupt; parse fail -> Corrupt); src-tauri/src/link_types.rs:514-518 (read_deltas, the lenient path the option does NOT use); src-tauri/src/link_types.rs:115-168 (merge — seeds base, deltas can only override or add); src-tauri/src/federation/owner.rs:134-140 (the fail-closed precedent)
- [BREAKS] A8 — SQLITE TRIGGERS: the option controls what the RUST computes, and nothing about what the child's sqlite_master contains. THE KILL SHOT.: Fully verified chain, reachable on an ordinary boot. (1) `init_db_scoped` DROPS the child's sky triggers UNCONDITIONALLY — `DROP TRIGGER IF EXISTS note_meta_sky_au; DROP TRIGGER IF EXISTS note_meta_sky_ai;` sits at the function's top-level indentation (search.rs:5631-5635), and the stratum family (search.rs:5867-5874, 5884-5888) and maturity family (search.rs:5924-5930) are dropped the same way. (2) All three are RECREATED only inside `if owns` (search.rs:5640, 5891, 5933). (3) `init_db_schema_only` sets `owns = false` (search.rs:4597-4598, 4602). (4) It is called on a LINKED universe from federation/migrate.rs:169, reached from attach.rs:172 whenever a cUniverse's schema is stale — the ordinary case for a universe the user has not opened since a Constellation update. NET: after one parent boot, a linked child's database has NO note_meta sky triggers at all. (5) `note_meta_sky_ai` (search.rs:5650) is the ONLY thing that INSERTs a sky_nodes row for a new note, AND it carries the PJ-207 §15 `target_cid_cn` back-resolution (search.rs:5672-5684). (6) `maintain_sky_after_save` only UPDATEs — `UPDATE sky_nodes SET stratum = (...), maturity = (...) WHERE path = ?1` (search.rs:2717-2724) — so with no row it affects 0 rows and returns `Ok(())`, leaving `maint.sky_failed` FALSE. NOW ROUTE A WRITE THERE. The Owner Scope delivers the child's vocabulary perfectly: note_meta and note_links carry the right values, the right link_type, the right incoming aggregates. And the note has NO sky_nodes row (invisible in Sky View), NO stratum, NO maturity, and its inbound links never get their identity key — with every row count correct and `MaintenanceOutcome` reporting success. This is the H1 shape one layer below where the option operates. Decisively: migrate.rs:167-171 states the safety premise for `init_db_schema_only` IN WORDS — 'The owner does all of that on its own next launch ... and until then nothing writes through those triggers, because a cUniverse is attached read-only.' Phase 1.2's Router IS the thing that makes that sentence false, and the Owner Scope does not address it, because the defect is ABSENT DDL, not wrong DDL. No vocabulary-threading design can fix it.
    evidence: src-tauri/src/search.rs:5631-5635 (unconditional DROP of note_meta_sky_ai/_au); src-tauri/src/search.rs:5636-5640 (the PJ-232 comment and `if owns {`); src-tauri/src/search.rs:5867-5888 and 5891 (stratum drops unconditional, recreate owns-gated); src-tauri/src/search.rs:5924-5933 (maturity drops unconditional, recreate owns-gated); src-tauri/src/search.rs:5650-5686 (note_meta_sky_ai — the sole sky_nodes INSERT plus the target_cid_cn resolution at 5672-5684); src-tauri/src/search.rs:2706-2725 (maintain_sky_after_save is UPDATE-only, returns Ok on 0 rows); src-tauri/src/federation/migrate.rs:169 and src-tauri/src/federation/attach.rs:167-172 (the reachable auto-migrate path); src-tauri/src/federation/migrate.rs:167-171 (the read-only premise this migration falsifies)
- [survives] A9 — does the option make `a_vocabulary_swap_reaches_back_into_an_already_open_database` FAIL?: YES, and it is a SIGNAL, not a defect — but only because I verified the fixture is genuinely discriminating rather than vacuously satisfiable. The test installs the full vocabulary, opens the DB, then `set_active(deltas(&[]))`, then indexes, and asserts `got.edges.iter().any(|(_, t, ty)| t.contains("refutes::") || ty == "associative")` (vocab_harness.rs:246-254). I checked whether an unrelated untyped link could satisfy that disjunction regardless of the swap: NOTES is exactly two notes with exactly ONE link — `[[refutes::Target|because of X]]` in Source.md, and Target.md has none (vocab_harness.rs:180-183). Under the full vocabulary the single edge is (Source, "target", "refutes") — the target contains no `refutes::` and the type is not `associative`, so BOTH disjuncts are false and the assertion genuinely fails. Under the option, `index_note` takes a `&WriteScope` whose vocabulary was captured at construction, so the swap no longer reaches and the test correctly goes red. The test's own message names this outcome: 'If this ever fails, the coupling changed and 1.2's design premise must be re-checked.' THE DEFECT IS WHAT HAPPENS NEXT. The test as written is a detector for the RULED-OUT design; it cannot distinguish 'the coupling was severed correctly' from 'someone reintroduced global swapping and it happened not to fire this run'. Landing 1.2 with a deliberately-red test, or deleting it, both lose the pin. It must be REWRITTEN to assert the inverse — that a `set_active` between scope construction and the write does NOT reach the result — and the rewrite must construct the scope BEFORE the swap, or it re-pins nothing.
    evidence: src-tauri/src/federation/vocab_harness.rs:226-255 (the pinned test); src-tauri/src/federation/vocab_harness.rs:180-183 (NOTES — two notes, one link, no second untyped edge); src-tauri/src/search.rs:7243-7281 (parse_link_body: unknown head -> whole body is the target, unknown alias -> type collapses to associative); src-tauri/src/federation/vocab_harness.rs:275-277 (routed_write_must_match_the_owners_vocabulary, still #[ignore] + panic!)
WORST SILENT FAILURE: THE WORST SILENT FAILURE THE OPTION STILL PRODUCES — a routed write into a child whose sky triggers the PARENT has already dropped. Sequence, every step verified: a linked cUniverse's search.db is at a stale schema (the ordinary state for a universe not opened since a Constellation update). The parent boots, federation attaches, `verify_schema` fails, attach.rs:172 calls `run_migrations_on`, which calls `init_db_schema_only` (federation/migrate.rs:169). That runs `init_db_scoped` with `owns = false` (search.rs:4602), which DROPS `note_meta_sky_ai` / `_au` unconditionally (search.rs:5631-5635), DROPS the stratum family (search.rs:5867-5888) and the maturity family (search.rs:5924-5930) unconditionally, and recreates NONE of them because every recreate sits behind `if owns` (search.rs:5640, 5891, 5933). The child's database is now permanently missing its note_meta sky triggers until it is next opened as its own active universe.

Phase 1.2 then routes a write there. The Owner Scope does its job perfectly: the child's `link-types.json` is read strictly, the parse chain gets the child's registry, `note_links.link_type` is right, `incoming_count` and `incoming_link_types` are right, every row count is right, and `routed_write_must_match_the_owners_vocabulary` would pass — because `aggregates_for` (vocab_harness.rs:73-106) reads only note_links and the two incoming columns.

What actually happened on disk: `note_meta_sky_ai` is the ONLY statement that INSERTs a sky_nodes row for a new note (search.rs:5650-5657), so the note gets NO sky_nodes row and is invisible in the child's Sky View. That same trigger body carries the PJ-207 §15 back-resolution (search.rs:5672-5684), so every existing `note_links` row pointing at this note keeps `target_cid_cn` NULL and every identity-keyed reader — collection membership, identity link resolution — silently misses the edge. And `maintain_sky_after_save` cannot repair it, because it is UPDATE-only (`UPDATE sky_nodes SET stratum = (...), maturity = (...) WHERE path = ?1`, search.rs:2717-2724): with no row it affects zero rows, returns `Ok(())`, and `maint.sky_failed` stays FALSE. The write reports success.

Correct row counts. Correct values in every column the acceptance test reads. Missing derived state, unresolved link identities, no error anywhere — and the child's own self-heal will not fire, because `links_backfill::is_needed` compares the child's stored fingerprint against the process global (links_backfill.rs:99) and the routed write left that stamp untouched, so on the child's next boot it reads FALSE.

The structural point: the Owner Scope's guarantee is scoped to what the RUST computes. The trigger layer is a second computing authority living in the child's `sqlite_master`, and `federation/migrate.rs:167-171` states the premise that made its absence safe — "until then nothing writes through those triggers, because a cUniverse is attached read-only." Phase 1.2 is precisely the change that falsifies that sentence, and no amount of vocabulary threading touches it.
CONDITIONS: C1 (BLOCKER — kills A8). The routed arm of `WriteScope::for_note` must VERIFY the owner's trigger set before writing, and REFUSE if it is incomplete. Probe the child's `sqlite_master` for `note_meta_sky_ai`, `note_meta_sky_stratum_au`, `note_meta_sky_maturity_au` and `note_links_outgoing_ai/_ad/_au`; if any is absent, return an error naming the universe and telling the user to open it once. This is the only place the check can live, because `init_db_schema_only` (search.rs:4597) DROPS those triggers (search.rs:5631-5635, 5867-5888, 5924-5930) and recreates none of them under `owns = false`. Do NOT 'fix' it by having the parent recreate them — that is exactly the parent-flavoured DDL PJ-232 closed (federation/migrate.rs:142-167). Refuse, don't repair.; C2 (BLOCKER — kills A6). The routed constructor must check `search.db` EXISTS before `Connection::open`, and error if it does not. `Connection::open` creates the file (that is how search.rs:4603 manufactures a new database), and attach.rs:156-160 proves 'search.db missing' is a real handled state whose only symptom today is a warning. One failed routed write otherwise leaves a zero-table search.db that flips `cu_db_path.exists()` to true and drags the child through `run_migrations_on` on the next boot, permanently degrading its schema.; C3 (BLOCKER — repairs the A5 hole in the cannot-forget claim). Deleting three free functions does not close the ambient door while `snapshot()` is `pub` (link_types.rs:498) over a `pub struct LinkTypeRegistry` (link_types.rs:100) with `pub fn is_known` (link_types.rs:171) and `pub fn is_structural` (link_types.rs:229) — `link_types::active_universe_snapshot().is_known(x)` compiles in one line forever. Either (a) make `active_universe_snapshot()` `pub(crate)` and add a test asserting its production call-site count and file list, so a new one is a red test rather than a clean build; or (b) return a distinct newtype (`ActiveVocabulary`) that the routed APIs will not accept, forcing a named unwrap at each of the ~12 sites. Without one of these, the option's central claim is legibility, and it should be described that way in the plan.; C4 (BLOCKER — A9). Rewrite `a_vocabulary_swap_reaches_back_into_an_already_open_database` (vocab_harness.rs:226-255) to assert the INVERSE: construct the WriteScope BEFORE the `set_active` swap and assert the result is UNCHANGED by it. Landing 1.2 with that test red, or deleting it, both lose the pin on the ruled-out design. I verified the fixture is genuinely discriminating (NOTES has one link and no second untyped edge, vocab_harness.rs:180-183), so the rewrite is a real test, not a tautology.; C5. Pin the routed connection's PRAGMA set in the constructor and test it. `PRAGMA recursive_triggers=ON` is per-CONNECTION and search.rs:4620-4646 documents the exact silent failure when it is off — chained triggers skipped, stratum and maturity NULL on the new row, no error. Copy reconcile_filesystem's shape verbatim (search.rs:11978-11985: WAL, synchronous=NORMAL, recursive_triggers=ON, then `register_fts5_tokenizer`) and add a test asserting `PRAGMA recursive_triggers` reads 1 on a routed scope, so a future edit to the batch fails loudly.; C6. Construct the routed arm BEFORE taking `state.db.lock()`. Today that lock is the first statement of `reindex_single_note` (search.rs:12688); if the routed branch is added after it, a routed write holds the ACTIVE universe's writer lock across a foreign-file open and write, stalling the 1500 ms debounced save. Only the active arm should ever touch `state.db`.; C7. Name the fences the option leaves one identifier away from silently enabling routed child writes, and put a test on each. `reindex_changed_paths` uses `try_load_libraries` at search.rs:12971 (PJ-207 §8) and `reconcile_filesystem` walks the OWN library set; the option's WriteScope resolves the owner INSIDE `reindex_single_note`, so swapping `try_load_libraries` for `load_all_libraries` would enable routing with zero compile errors and zero prompts. The option already makes this argument for the rename cascade's `foreign` set (libraries.rs:6963, 6982-6986); it must make it for the watcher too.; C8. Add an explicit Boss ruling on Residual 2 BEFORE it is implemented. The owner's self-heal cannot notice a routed write — `links_backfill::is_needed` (links_backfill.rs:99) short-circuits FALSE on the child's next boot because `version_current` is true (links_backfill.rs:106) and the routed write left the fingerprint stamp untouched. The proposed remedy (a distinct `schema_versions` marker row written by the routed write) is still a write into another universe's database and needs a ruling, not a note.; C9. Extend the acceptance test's `Aggregates` (vocab_harness.rs:60-71) before removing the `#[ignore]`. It currently reads only `note_links` and two incoming columns, so it is BLIND to every failure in C1: sky_nodes.stratum, sky_nodes.maturity, note_meta.outgoing_*, and note_links.target_cid_cn. As written, 'green' would certify a write that lost all four. Add those columns and route `index_under_vocabulary` through `maintain_sky_after_save` (search.rs:2706), which it does not call today.