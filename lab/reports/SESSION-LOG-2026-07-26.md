# Session Log — 2026-07-26

**Branch:** `main` @ `73d28bed` (clean at start; docs-only changes this session, uncommitted pending
Boss approval of the Architect doc). **Function in hand:** MIG-105 Architect — the root library /
Core Organizer.

---

## Phase: MIG-105 Architect (PJ-145) — run, re-founded, LOCKED by Boss rulings

### Arc

1. **SO#8 cross-check** — confirmed MIG-105 not partly shipped (orientation v3.68 + 07-24/07-25
   logs all mark the shipped work symptom-level).
2. **Cycle 1** (`wf_25a129dc-853`, 29 agents): 11 territory surveys (720 file:line surfaces) →
   4 option architects (A/B/C/D) → 5 judges (D won 38/50) → 8 adversarial attacks → completeness
   critic. All load-bearing claims re-verified by hand against the live universe + 2.0 GB search.db.
3. **Boss concept intervention** — the *Brain / Core Organizer* concept (verbatim in the Architect
   doc): the entity is the Universe's head; it remains; scope amended. Killed options B/C.
4. **Cycle 2** (`wf_8ade7478-139`, 34 agents): concept passed to the **Art Director & Team**
   (4 researchers → locked spec → 4 designs → 4 judges → lead merge) and the **Inspectors**
   (saved safety-inspection + 3 concept inspections → Chief verdict). Two hand-offs truncated.
5. **Re-run** (`wf_e46ff8f5-60c`, 6 agents, file-based I/O): all 4 designs judged (hybrid 29 ·
   apparatus 28 · minimal 25 · identity 24); AD verdict v2 (4 reversals, incl. NO-mark restoring
   the Boss's 07-25 steer); Chief verdict v2 (federation inspection included — the Brain entry is
   the load-bearing **federation anchor**; verdict SAFE-AFTER-PREREQUISITES, amendment must be
   TOTAL + FEDERATION-STAGED).
6. **Boss rulings 2026-07-26** — name "Core Organizer" (Root/System Library + Core MOC rejected
   with accepted reasoning); NO icon; Universe MOC = one front-page note inside `_Core`; closed
   kinds (MOC + Templates + Five Acts); TOTAL root rule (no user notes at root; home-Library asks);
   **all loose root content → "Eisa Test"** (58 indexed + 1 unindexed + canvas; Templates/Five Acts
   → `_Core`; `.trash` stays).

### Documents produced (docs/migrations/, uncommitted)

- `MIG-105-Architect-root-library-vs-flat-universe.md` — **v2, the locked design + rulings**
  (supersedes the v1 options analysis; awaiting Boss approval → Stage 0 → Plan).
- `MIG-105-ArtDirector-verdict-Core-Organizer.md` (v2) · `MIG-105-Inspectors-verdict-Core-Organizer.md` (v2).
- `MIG-105-root-library-vs-flat-universe.md` — status stamped SUPERSEDED (pointer to canonical docs).

### Defects discovered this session (all verified; to file at next PJ bump per SO#9)

- **P1** path cascade 5-of-11 tables (every rename/move orphans note_body / note_summaries /
  note_state_history / sight_v3_layout / shape_history / sources_suggestions rows) · **P2** FK
  pragma enabled only in a test · **P3** reconcile self-heal: 1,577 silent "relocate deferred",
  error discarded at reconcile.rs:192 · **P4** UniverseMeta eats unknown keys — Boss's
  `custom_stages` destroyed by rename/attach/detach; zero readers/writers exist · **P5** 17 root
  notes lack cid_cn · **P6** `Testing opened note.md` on disk, no index row · **P7** second-screen
  fixed-name walker mislabels nested libraries' notes · **P8** three first-match resolvers
  (bases.rs:382, shape.rs:161, tasks.rs:529) + callerless get_library_mode · **P9** vitest
  `include` allow-list silently skips unlisted test files.
- **Federation class (Inspectors v2):** PR-F1 anchor · PR-F2 five first-match `.find()` sites incl.
  the trash root · PR-F3 name-keyed joins forbid a shared stored Core name · PR-F4 per-universe
  shape version at attach · PR-F5 cross-universe writes vs read-only child DBs · PR-F6 620
  parent-held child rows.
- **In passing:** `.v-chev` never RTL-flips; "vault" shipped in 10 locales (`universe.manager.*`);
  `.section-label` uppercase breaks Arabic cursive; `RECENT_CAPTURES_CONTENT` English-only;
  939 MB orphan `Constellation SV Test.db` + uncapped logs; `.base` files are YAML parsed as JSON
  (name never read); review-pulse RMW can wipe history after a transient read failure; appearance/
  i18n/Style-Setter surface has never been inspected.
- **Blocker:** Roaming `universes.json` disagrees with the live universe (lists only كون عيسى) —
  instrument `save_registry` before the migration names its target.

### Open items

Boss rulings pending: R3 (.trash 55 rows) · R4 (Stage 0 now — recommended yes) · R5 (MIG-104
first — recommended yes) · R7 (sight cache purge) · R8 (orphan DB + log caps) · stored-name token
format (recommended `"_Core — <Universe name>"`, confirm at Plan). Then: Architect-doc approval →
Stage 0 → Phase 2 Plan.

**PJ ledger note:** reconciliation (SO#9) deferred to the Architect-phase close (Boss approval of
the doc) — the §9 ledger list in the Architect doc is the staging list for that bump.

---

## Phase: MIG-105 STAGE 0 — the live-defect remediation (BUILT + Boss-validated)

**Binary:** `src-tauri/target/release/constellation.exe` @ 21:42. **Gates:** Rust **1181/0** (10 ignored)
· svelte-check **0 errors** · vitest **53 files / 626 tests**. Per-build safety inspection
`wf_75a9d203-e96`: 41 confirmed whole-app (15 HIGH / 19 MED / 7 LOW), **ZERO in-diff** — verified
mechanically by intersecting every finding's file:line against `git diff -U0` hunk ranges (±3).

### Commits landed (C1-C7 + P0)

- **C1 (PJ-157)** — `vitest.config.ts` 52-entry allow-list → **globs** (`tests/**`, `src/**`) with
  explicit, reason-commented excludes; new `vitest.manual.config.mjs` + `npm test` /
  `npm run test:red:frontmatter`. A test file can never again silently not-run. All 5 clauses verified
  (52 collected, red-pin 4F/1P, exclusion holds, sight-v6 5 files, before==after).
- **C2 (PJ-151)** — reconcile: the discarded `rusqlite::Error` is captured and named; the fabricated
  "target busy/contended" message is gone; re-adopt `Err` surfaced; bounded ≤20 lines/boot + a forced
  boot summary whenever anything failed.
- **C3 (PJ-149)** — `migrate_note_db_paths` **5 → 11 tables** (+note_body, note_summaries,
  note_state_history, sight_v3_layout, shape_history, sources_suggestions), destination pre-deletes,
  lazy-table no-op for shape_history, every statement de-silenced. `reindex_delete_note` purges the
  same five. `mig003_step4` delegation comment. **The new idempotence test caught a latent landmine:**
  a repeat call with the same (old,new) DELETED the freshly-migrated destination rows → note indexed
  NOWHERE. Guarded by a source-row existence check.
- **C4 (PJ-154)** — cid-collision self-heal at the `index_note` funnel: owner-file gone → relocate the
  dead row via the 11-table cascade → retry; owner-file live → index under the `''` sentinel, NEVER
  steal. Covers all adoption surfaces at once.
- **C5** — `relocate_row` delegated to the single shared cascade (no more drifted duplicate).
- **C6 (PJ-153)** — `cid:` → `cid_cn:` at all three canonical emitters (value-preserving legacy
  migration); healer injects for non-templates via `ensure_cid_cn` + reindex; template exemption after
  index selection; `resolve_templates_dir_for_root` factored out. 19 tests.
- **C7 (PJ-155/156)** — `collect_library_notes` exclude-set (second-screen duplicates/mislabels fixed);
  new shared `owning_own_library_name` (own libraries only — MIG-065 §J) replacing the last three
  first-match resolvers (bases/shape/tasks); callerless `get_library_mode` deleted; scoped_paths
  boundary; store.ts `addLinkToNote` longest-root + sidecar-trash boundary guard. 11 Rust + 3 TS tests.
- **P0 (PJ-161)** — **no app defect.** The "registry disagreement" was an observer artifact: Claude
  sessions read a stale MSIX AppContainer shadow of `%APPDATA%`. Architect §5 corrected; standing
  protocol recorded (never trust an in-container AppData read; never launch Constellation from inside
  a Claude session; registry writes only from a Boss-launched process).

### ★ THE DISCOVERY — foreign keys ARE enforced (overturns the PJ-150 diagnosis)

The Boss's first Stage-1 boot logged `FOREIGN KEY constraint failed` from the cascade. A direct probe
on a real `init_db` connection settled it: **`PRAGMA foreign_keys = 1`** — rusqlite enables FKs on
every connection, so no PRAGMA appears in our source and a grep for the enabler could never find it.
The child tables are `ON UPDATE NO ACTION` ⇒ **SQLite refuses the parent `note_meta.path` UPDATE for
any note owning a summary / state-history / suggestions row.** That is the true root cause of the
**1,591** "relocate deferred" failures — invisible for ~3 weeks, and unreproducible in every replica
replay because the replicas ran FK-off. Fix: the cascade now runs under `PRAGMA defer_foreign_keys`
inside a transaction (owned when the caller has none), so parent and children move together and are
validated at COMMIT. **Red proven** by disabling the pragma — the harness reproduces the Boss's live
log line verbatim — then green with it. Pinned permanently as `tests_pj150_fk_enforcement_reality`.

### ★ TWO FALSE-SUCCESS BUGS I INTRODUCED, both fixed

1. `index_note`'s heal logged "relocated … and re-indexed" **before** its retry → claimed victory on a
   refused relocation while the note stayed invisible. Now reports the verified outcome
   (OK / PARTIAL / FAILED) after re-reading the dead row.
2. `relocate_row`, once delegated to the deliberately best-effort shared cascade, returned `Ok`
   unconditionally → reconcile announced **"healed index drift: 14 relocated"** on a boot where
   **nothing moved**. Now verifies the row actually moved before reporting success; the failure
   sentinel is named distinctly in the reconcile log.
Both are **LL-035** (added this session): a claim that something is inactive is a RUNTIME claim; and
never log a success you have not verified.

### Boss Stage-1 validation (2026-07-26, second binary) — PASS, verified in the DB not the log

`Testing opened note.md` present at its real path with identity `20260711T142152Z_NOTE_2FDC`, dead row
gone; **8/8** verifiable heal pairs healed (old=0 / new=1); **0** orphaned child rows in all three
tables; note_meta total unchanged 7817 (no duplicates); root library 126 → 128. cid healer:
`stale=17 templates=14 injected=3 still_empty=0` on the first healed boot — the single remaining
empty-cid row is `.trash\ابن فضلان - Copy.md`, whose identity the duplicate-guard **correctly refused
to steal** from the live original. (An earlier claim of mine that the healer "didn't fire" was wrong —
the baseline was taken post-heal; corrected here.)

### Open

- **R2 ruling still needed** (gates C8): on a genuine note delete, does `note_state_history` die with
  the note (CASCADE as declared) or get archived first? C8 (child-table rebuild to
  `ON UPDATE CASCADE`) is now upgraded from optional hardening to the proper structural fix — the
  deferred-FK cascade makes moves work, but the declaration is still wrong.
- PJ-124 struck a **third** time (inspection ignored `args.files`, ran whole-app). 23 verifier agents
  died on a session limit — their candidates are unverified, not cleared.
