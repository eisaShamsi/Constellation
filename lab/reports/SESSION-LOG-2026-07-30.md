# SESSION LOG — 2026-07-30

Continues 2026-07-29 (PJ-187 S-sweep, uncommitted, Boss Stage-1 in progress).

## §1 — Pre-commit safety inspection: an APP-KILLER in the sweep's own fix, plus three more

Run `wf_860b6505-c41` (85 agents, ~10.1M tokens): **34 confirmed**. Triaged against the diff:
**6 in the sweep's own changes — all fixed + RED-proven before commit**; 27 pre-existing → the
triage feed `lab/reports/pj187-inspection-2026-07-30-remaining.md` (1 APP-KILLER, 10 HIGH,
13 MED, 3 LOW). 25 verify agents were lost to server 500s — those candidates died UNVERIFIED,
so the register is an under-count, not an all-clear.

### §1.1 The APP-KILLER (store.ts:624) — LL-040's shape, fourth time

The sweep's cascade gate in `flushOutgoing` refused to write a dirty model mid-cascade —
correct — but returned `{ok:true}`, which the FlushResult contract (noteSession.ts:266)
defines as "safe to proceed with the nav/replace". Every departure site then destroyed the
model: nav re-seeded it, closeTab/universe-sweep disposed it. The edit existed nowhere — not
on disk (refused), not in the net (NoteEditor's stash sites sit BELOW its own cascade gate),
no banner (nothing "failed"). Reachable by: rename a heavily-linked note, edit another note's
property in the right sidebar (not covered by the freeze overlay), close/switch mid-walk.

**Fix at the choke point:** stash the model's composed content into the write-ahead net
(unflagged — real work, so PJ-181's stale-snapshot check never discards it), THEN return
`{ok:false, reason:'cascading'}`. Nav sites abort (user stays on the note); close/sweep
proceed but the net now holds the content for reopen. Proof:
`tests/pj-187/cascadeDepartureLoss.test.ts` (4 cases incl. a no-cascade control), RED-proven:
under the broken version the edit vanishes from the net and the nav destroys the model.

### §1.2 Three more in-diff findings, fixed + proven

- **store.ts:1820** — `collectionsLoaded` was never reset, so a universe SWITCH whose read
  failed kept the latch TRUE and the store holding the PREVIOUS universe's collections: the
  next star wrote universe A's list over universe B's file. Fix: reset latch + list at the
  top of `loadCollections` (no subscriber auto-saves, verified). RED-proven.
- **store.ts:1833** — `saveCollections` snapshotted its payload at call time; two rapid
  toggles could land save1(old) → save2(new) → save1-RETRY(old), dropping the newer star.
  Fix: single-flight chain, payload read at WRITE time. Proven (blocked-first-write harness).
- **store.ts:3098** — `drainCidEnsure` writes the note's permanent cid through the gate
  (watcher-suppressed) and never reindexed — the exact gap the sweep closed in `openNoteTab`,
  missed in this sibling. The Whole-Ecosystem law violated INSIDE the sweep, second instance
  (first: move_to_trash). Fix: same fire-and-forget reindex.
- **store.ts:1737 / :2865** — both nav paths consume the INCOMING note's write-ahead entry
  (resolveNoteContent) BEFORE the flush-abort/supersede checks; an aborted nav destroyed the
  only copy of a failed save's recovery. Fix: re-stash the consumed entry (unflagged) on every
  abort after the read. RED-proven via the cascade-abort path.

### §1.3 Gates after the fixes

vitest **67 files / 717** (+ Sight **5/84** in the PJ-172 serial lane) · svelte-check **0** ·
Rust untouched since 1287/0. Binary rebuild running; Test 5 instructions held until the
chain (sources → build/ → exe) verifies per the standing rule.

### §1.4 PJ-166 — NINTH strike

Invoked diff-scoped with `args.files` (17 files); returned `mode: "whole-app"` again.
85 agents / ~36 min for what should be a 17-file gate. It earned its cost AGAIN (§1.1 is
real), but the per-build gate the standing order requires still does not exist.

## §2 — Stage 1 COMPLETE: Boss all-Pass; the sweep lands

Test 5, all seven steps: **Pass** (typing latency, save+tab round-trip, property round-trip,
linked rename incl. the mid-cascade typing recipe, move-with-unsaved-typing, Alt-nav, restart
restore). With Tests 1–4 that is the full Stage-1 matrix green. Per the Boss-test standing
order the commit gate is satisfied; landing as a thematic commit series (Rust safety · Rust
silent-failures · core store/layout · panels+i18n · docs/reports/ledger), then push.

Books written in this close: MoCh-2026-07-29-2100 · Orientation **v3.78** · PJ ledger **v1.61**
(files PJ-193 trash-browser UI; supersedes the "Consolidate trash" button into the migration
back-fill; records PJ-166's ninth strike). `tests/pj-187/zz-out.txt` (debug scratch) removed.

## §3 — MIG-108 Phase 1 (Architect) — "One Universe, One Location"

Boss: "Go." Territory mapped by a 7-slice workflow + completeness critic (run
`wf_1e0af182-c53`, 8/8 agents, ~1.24M tokens): sqlite (16 path-bearing tables, trigger
gating, FK hazards), json-state (17 stores ranked), frontend-state (localStorage/boot/
watcher/SS), move-machinery (gate_rename reuse; NAME-based links → zero body rewrites,
proven), import-linking (add_library the choke point; cUniverse out of scope by
construction), trash-settings (collapse mechanics §D-F), prior-art (Lightroom re-link
shape; index-only-data horror stories = our search.db exposure). Critic findings: the
reconcile hard-abort cap invalidates it as a crash net (journal required); WAL-blind
snapshot hazard; cross-registry entanglement; basename collisions; two walkers without
nested-library exclusion (canonical.rs, embeds.rs); SS outside every freeze channel;
earned.jsonl path-fallback claim REFUTED from code (cid/name only).

**Architect doc:** `docs/MIG-108-One-Universe-One-Location-Architect.md` — concept,
measured scope, 12 hazards → mitigations, run design (P/S/F/M/R/T/V/W), standing
constraints, 10 audit invariants, rehearsal protocol (full-copy dry run first), 4 Boss
decision points (layout D1, copy-vs-move D2, PJ-065-test-book D3, timing D4). Maps
archived in scratchpad; key excerpts inline in the doc.

Awaiting Boss rulings on D1–D4 → Phase 2 (Plan).

## §4 — MIG-108 Phase 2 (Plan) drafted; Boss rulings D1–D4 received

D1 layout=FLAT `<root>\<name>` · D2 bring-in=ASK (Copy default / Move) · D3
PJ-065-test-book=relocate BY COPY (repo originals stay as git-tracked fixtures; the other 17
are true moves) · D4 order=MIG-108 first, MIG-104 Slice 8 after.

Plan: `docs/MIG-108-One-Universe-One-Location-Plan.md` — 8 slices (0 pre-work walkers ·
1 journal/classifier/snapshot · 2 move+rewrite+verify · 3 trash+settings+PJ-192 ·
4 proposal UI ⏸ Stage-A on a scratch copy · 5 bring-in flows · 6 full-copy rehearsal ·
7 live run ⏸ Stage-B · 8 docs + Phase-4 audit). Awaiting Boss approval of the Plan → Build.

## §5 — MIG-108 Build: Slice 0 landed

Plan APPROVED by the Boss; cascade begun. Slice 0: `canonical.rs::collect_files_recursive`
(+ its 6 call sites, incl. the boot repair probe) and `embeds.rs::build_vault_index` (+ the
walk/lookup/resolve chain) gain the `nested_library_paths` exclusion — the last two walkers
outside the Library ≠ Folder discipline. New `invalidate_all_vault_indexes()` hooked into
`save_libraries`: any registry change clears the embed index cache, so a stale
pre-registration index can never keep serving (or shadowing behind) a now-foreign subtree.
resolve_embed reads the registry ONCE for both its own exclusion and the cross-library
fallback. Proofs: 3 new tests, each with a no-exclusion control proving the guard
load-bearing. Rust **1290 passed / 0 failed**.

## §6 — MIG-108 Build: Slice 1 landed

New `src-tauri/src/mig108.rs` — the engine's three foundations:
- **Journal** (`.constellation/mig108-journal.json`): phases Planned → Snapshotted → Moving →
  Moved → DbRewritten → JsonRewritten → Done (+ VerifyFailed), per-entry moved flags,
  per-store rewritten flags, the pre-move Baseline aggregates. Written via atomic_write
  BEFORE each mutating step; a corrupt journal is SURFACED, never silently discarded (it is
  the only record of a possibly half-moved universe — the boot reconcile hard-aborts at this
  scale, Architect H4).
- **Classifier** (pure, AppHandle-free): UnderRoot / Move / Copy (D3) / ForeignUniverse (H6)
  / Missing; flat destinations (D1) with basename de-collision against the fs AND the plan
  itself (H7); one normalization rule (NFC + separators + case, H3); same-volume detection
  per entry.
- **Snapshot** (H5): wal_checkpoint(TRUNCATE) → copy search.db (+ non-empty sidecars) →
  reopen read-only and assert the Baseline matches → copy the 8 path-bearing JSON stores.

Proofs: 7 tests — every classifier class incl. foreign-under-root and both de-collision
sources; journal round-trip/resume/corrupt-surfacing; snapshot verify incl.
"complete-without-its-WAL". Rust **1297 passed / 0 failed**.

## §7 — MIG-108 Build: Slices 2 + 3 code-complete (uncommitted, inspection in flight)

**Slice 2 — move + rewrite + verify** (mig108.rs +~700 lines):
- `remap_path` — component-wise, NFC-safe (an NFD-stored Arabic component has a different
  LENGTH than its NFC form, so byte-offset prefix slicing is unsound); suffix components
  carried VERBATIM so every equality-keyed consumer that found a row before finds it after.
- `run_move_phase` — journaled per entry; crash-window adoption (dest exists + source gone
  = moved); copy-class copies without deleting the source (D3); cross-volume fallback.
- `run_db_rewrite` — ONE transaction: defer FKs, drop the O(N²) outgoing triggers (sky
  triggers stay ACTIVE — the proven live-move cascade), loop the proven
  `migrate_note_db_paths` per enumerated note (it joins the outer tx via is_autocommit),
  straggler sweep over note_links/note_aliases/sky_nodes/sky_links/review_schedule (catches
  FK-orphans + the unstamped-review gate), cursor resets + sight_v3/link_stats wipes,
  recreate triggers + recompute_all_outgoing once, then IN-TX VERIFY (baseline aggregates
  byte-equal; zero rows under any old prefix; every moved dir present) — COMMIT only on
  green, else ROLLBACK + journal VerifyFailed.
- `run_json_rewrites` — ONE deep remapper over keys AND values (folderTemplates is keyed by
  absolute path) across 8 stores incl. session.prev.json; per-store journaled resume.
- `run_engine` — the resumable orchestrator.
- Proofs: end-to-end fixture (backslash-stored rows, NFD-vs-NFC Arabic, FK child, unstamped
  review row, copy-class, 5 JSON stores) · interrupt-after-moves resume · RED verify-failure
  rollback · remap unit proofs. Rust 1302/0 at slice close.

**Slice 3 — trash consolidation + settings collapse + PJ-192**:
- `consolidate_trash` — per-library `.trash` top-level entries → root `.trash` via the shared
  de-collide pair; folders move as units (attachments included); emptied sources removed;
  idempotent (proof incl. cross-library same-name + pre-existing suffixed destination).
- PJ-192 CLOSED: `move_to_trash` demoted from #[tauri::command] to pub(crate) (frontend
  stopped invoking it in PJ-187; the sole Rust caller passes the universe root, which is now
  the only meaning the collapsed setting has).
- Frontend: `trashFolderScope` retired — type, default, resolver arm, Settings row, and an
  explicit purge in applyParsedSettings (the spread-over-defaults load resurrects stale keys
  and saveSettings round-trips them forever); i18n −6 keys ×15 (incl. the two verified
  orphans: settings.files.permanentDelete, dialogs.batchDeleteTrash).
- `tests/pj-187/trashDestination.test.ts` REWRITTEN for the collapsed contract (6 tests,
  incl. legacy-key purge + Overwrite≡Delete at the backend boundary).

Gates: Rust **1303/0** · vitest **66/710** · svelte-check **0**. Commits deferred until the
batched diff-scoped safety inspection (run `wf_013fa5cc-4ba`) clears — the sweep precedent:
one inspection before the commit series, every confirmed finding fixed first.

## §8 — MIG-108 Slices 2+3 LANDED; the inspection was PARTIAL (limit-truncated)

Commits `1be3d098` (Slice 2), `4243dc78` (Slice 3 + PJ-192 closed), `13e9a96d` (the two
inspection APP-KILLERs). Gates: Rust **1303/0** · vitest **67 files / 715** · svelte-check **0**.

### ⚠ THE INSPECTION DID NOT FINISH — re-run required
Run `wf_013fa5cc-4ba`: **11 of 28 agents died on the weekly usage limit** (resets Aug 1). Only
**3 of 14 scopes** actually ran — cross-window-integrity, frontmatter-property-writes,
persisted-json-state. The eleven that never ran include every scope most relevant to this
build: rename-move-delete-gate, note-save-index, notemodel-ownership, editor-lifecycle,
rename-cascade-integrity, derived-index-triggers, boot-init-ordering, reactivity-concurrency,
freeze-and-leaks, frontend-write-callers, cece-sources-derived. **MIG-108 must not reach its
live run (Slice 7) until a complete inspection has covered the engine.** Recorded here rather
than in a passing sentence because a truncated inspection reads exactly like a clean one.

### What the 3 completed scopes found (12 confirmed; 2 APP-KILLERs fixed in `13e9a96d`)
Both APP-KILLERs are the PJ-187 collections bug living unfixed on sibling stores — a
Whole-Ecosystem gap, and both are files MIG-108's own rewrite phase replaces:
- **workspaces.json** ("the precious file", MIG-100's words; no `.prev` rotation): a failed
  read left `[]`, and the first Save workspace replaced every named snapshot with one entry.
- **property-types.json**: a failed read left `{}`, and one type assignment wiped every
  property-type assignment in the universe.
Both now latch on a SUCCESSFUL read, refuse to write otherwise, and surface it. **My first
property-types fix carried the bug its own test caught** — `{}` is truthy AND an object, so
the obvious guard latched on precisely the ambiguous empty-bundle case it existed to reject
(LL-040's shape again: the predicate must be "a read succeeded", not "no error was recorded").

**Filed, not fixed** (pre-existing, outside MIG-108's blast radius — for the next PJ triage):
SecondScreenPage never repaths/closes on rename/move/delete (HIGH) and never clears tabs on
universe switch (MED) · yamlDoc.ts:311 malformed-YAML passthrough + :199 a FOURTH unguarded
block shape (interior comment in a flat list) (HIGH ×2) · settings.json has the same missing
latch as its two siblings (HIGH) · rename_universe repoints libraries.json even when the
folder rename was skipped (MED) · set_review_priority ignores its affected-row count (MED) ·
two parser-fidelity LOWs (quote-blind list split, `\"` unescaping).

## §9 — MIG-108 Slice 4 landed: proposal · progress · resume-on-boot

Rust: `mig108_preflight` (read-only, feeds the dialog; re-runnable as the user flips entries)
· `mig108_journal_state` (the boot probe) · `mig108_execute(copy_paths)` · `mig108_resume` —
all thin over the tested engine; `run_engine_step` steps one phase per call so the wrapper
emits `mig108:progress` between phases (the honest granularity: moves are near-instant
renames, the DB rewrite is one indivisible tx). Trash consolidation joined the resumable flow
(JsonRewritten → Done, idempotent).

Frontend: `Mig108UnifyDialog.svelte` — self-probing (renders nothing unless there is
something to unify or resume), four states (proposal / running / summary / resume). The
proposal lists every entry old→new with a per-entry Move/Copy flip (Boss D2/D3 generalized:
the dialog IS the copy_paths selector, so no repo-detection hack for PJ-065-test-book — the
Boss flips it to Copy at Stage-B). Skips shown with plain-language reasons; the backup named
before the button. The run envelope lives in the dialog: flush dirty tabs → close the second
screen (H9) → unwatch every library (H10) → engine → summary → RELOAD (boot re-reads the
rewritten registry, rewatches at new paths, restores the session against rewritten tab paths
— the wake choreography through the one already-proven path). Resume-on-boot surfaces an
unfinished journal, never silently continues; VerifyFailed gets its own explanation.

i18n: the `mig108.*` group — 25 keys × 15 locales, all translated in-pass.

Gates: Rust **1303/0** (16 engine tests) · vitest **67/715** · svelte-check **0**.

## §10 — Stage-A ready: the rehearsal world + the binary

`lab/tools/mig108_make_rehearsal.py` — builds a COMPLETE, self-consistent scratch copy: root
(minus the stale SV-Test db trio) + all 18 external trees + libraries.json re-pointed + a
78,931-distinct-path DB remap (triggers dropped; init_db recreates on boot) + the 8 JSON
stores deep-remapped + the universe renamed "MIG108 Rehearsal". Without the DB remap the
scratch db would still point at the REAL trees and the rehearsal would "pass" without
exercising the rewrite at all — the maker asserts ZERO leftover references to the real
universe before declaring READY. Two defects found while building it: Windows MAX_PATH broke
shutil on a ~200-char attachment filename (fixed with the extended-length prefix; the Rust
engine is immune — std handles it internally), and my first prefix fix itself was mangled by
shell escaping (caught by asserting longpath's output before use).

Disk-verified (not log-trusted): 18 external dirs · 7,684 .md notes (the exact real count) ·
registry entries outside the scratch world: none. Real universe: read-only throughout.

Binary: bundle 11:30 → constellation.exe 11:35; mig108_execute in the JS chunk,
mig108_preflight in the exe. Stage-A tutorial delivered to the Boss.

## §11 — Three rehearsal cycles, measured to the honest number

Cycle 1 (Boss, Stage-A): ~25 min — sky triggers active through the bulk. Cycle 2 (headless,
post-fix): 555 s; instrumentation split it — in-tx verify alone 205 s (my remap_any answering
a BOOLEAN with ~68M per-component NFC normalizations) + ~357 s unaccounted. Cycle 3 (fast
path + full spans): **455 s total** — verify 205→12.7 s (NormPrefixes: normalize each prefix
and each row ONCE); the blind region resolved to sweep 20.3 s + hygiene 0.1 s + COMMIT 80.3 s.
Run-to-run variance on identical work (loop 64→261 s, recompute 1.4→52 s) carries the
AV/OS-cache signature, not a code signature. Structural floor: real I/O — the ~1 GB WAL
COMMIT, the 2 GB snapshot copy, 160k point-statements through the PROVEN per-note cascade.

**Ruling taken:** 5–10 minutes on a universe this size is the honest number; batching the
cascade into hand-rolled SQL to shave minutes off a ONCE-per-universe migration is the wrong
trade against proven-ness (Constraint as Design). The dialog now promises "several minutes"
×15 locales; the timing spans stay (stderr-only, they feed Stage-B's report). Wider-net stale
rows = 0 on every cycle since the copy-class fix.

Slice 6's deliverable exists and has run three times green: the maker + the env-driven
harness + the independent verifier ARE the mechanical rehearsal.

## §12 — MIG-108 Slice 5 landed: the One-Location law becomes standing behaviour

- `ensure_under_active_root` (pure, tested) enforced at `add_library` — the choke point
  every registration flow funnels through. External paths are REJECTED with a message that
  names the bring-in flow; nothing can reference content in place again.
- `bring_in_library(source, mode)` — the D2 backend: Copy (original untouched, unmanaged)
  or Move (same-volume rename, cross-volume fallback), destination de-collided at the root,
  then registered + background-reindexed. Refuses: paths already under the root, foreign
  universe roots/children (H6), and REGISTERED external libraries (those belong to the
  unification proposal, which relocates them WITH their index rows — bringing one in here
  would strand its rows at the old path).
- `create_new_library_at`: parent constrained under the root; legacy `create_new_library`
  (pick-AFTER-Create) retired command+wrapper; `handleNewLibrary` defaults the location to
  the universe root.
- `BringInDialog.svelte` — the ask-each-time UI (Copy default / Move / Cancel), wired into
  `handleAddLibrary`: an under-root pick registers directly, an external pick asks.
- Importer targets restricted to OWN libraries (a copy into a read-only federated cUniverse
  library violated the per-universe boundary).
- Doctrine strings rewritten ×15: "Link Existing Library" → "Bring In a Library" (+Desc,
  setup variants, libraryManager), and `app.tagline` "A Vault of Vaults" → "One Universe
  for all your knowledge" — which also retires the last forbidden "vault" in the UI.

Gates: Rust **1305/0** (2 new slice-5 tests) · vitest **67/715** · svelte-check **0**.
NOT done in this slice (recorded): the `link_library_as_universe` double-entry
registration fix — deferred to Slice 8's registry-normalization note; it conforms to
One-Location already (its own root) and the dedup-by-path keeps resolve correct.
