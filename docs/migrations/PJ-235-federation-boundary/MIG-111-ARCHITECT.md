# MIG-111 — Full Cross-Universe Operations · ARCHITECT (Phase 1)

**2026-08-12** · Subsumes PJ-235 / PJ-254 / PJ-270…PJ-276 · Evidence:
`MIG-111-ARCHITECT-EVIDENCE.md` (six subsystem maps, three design options, three adversarial
verdicts — every claim carries file:line, verified in source) and
`ARCHITECT-INPUT-federated-write-sites.md` (the 22-site write register).

---

## 0. The concept (the horse)

> "I want to be able to conduct full functions/operations between universes. You have to ask
> yourself, why did I design Constellation to have a cUniverse(s) if I wasn't planning to have
> full access and/or operations among them? If it is kept as-is today, just to (read) and not
> able to (write), then why bother to include other universes (as cUniverses) in the first
> place? That's why Constellation are unique." — **Eisa, 2026-08-12**

Federation is **one knowledge space with full agency across it** — not a read-only viewport.
This completes, rather than contradicts, the Boss's own 2026-07-05 ruling *"It is ONE
universe"*: resolution already spans the federation; operations now must too. The read-only
contract was the implementation's assumption (MIG-056/MIG-065 §J), never the design.

**What made the old crossings app-killers was never the crossing — it was the SILENT crossing
with broken bookkeeping.** The end-state: every operation a user commands on a cUniverse note
works as naturally as on an own note, **with the bookkeeping done in the note's own universe's
database.** Automatic/background writes still never cross on their own.

---

## 1. The territory (what the code actually is today)

1. **One `search.db` per universe**, located ambiently: `db_path()` = the ACTIVE universe's
   `.constellation/search.db` (`search.rs:1465`), selected by the single process-wide
   `UniverseState.active_path` pointer. Every routine write goes through `SearchState.db` —
   the active universe's connection. **No path routes a write to a non-active universe's DB.**
2. **Four sites already open a second universe's DB** — the seeds of Option A:
   read-only `ATTACH ... mode=ro` (`federation/attach.rs:125-233`, cap 25); schema-only
   migration with `InitScope::ForeignSchemaOnly` (`federation/migrate.rs:68-180`); the FTS
   `optimize` prewarm which opens child DBs **read-write** (`search.rs:11358-11474` — the
   precedent that "cUniverse files are writable by design" at the OS level); tests.
3. **The process-global hazard:** `link_types::REGISTRY` is a `OnceLock` loaded from the
   ACTIVE universe only (`link_types.rs:351`, `search.rs:11606`). Trigger DDL and the backfill
   fingerprint gates are generated from it. Full-initing or writing a child DB from the
   parent's process with the parent's vocabulary is the PJ-230/PJ-232 corruption class —
   `ForeignSchemaOnly` exists precisely because of it. **Any write-capable federation must
   thread per-universe link vocabulary**, including through the `is_built`/fingerprint gates
   (`incoming_links_backfill.rs:49`, `links_backfill.rs:99`) — the adversarial pass proved
   maintenance is otherwise silently skipped on routed writes.
4. **The earned-data payload** (census of all 30 tables in the evidence file, §earned-data):
   - `note_links`: 5 earned scalars per edge (`traversal_count`, `last_traversed`,
     `confidence`, `created`, `status`); `weight` is derived (`1+ln(1+n)`) — transfer the
     count, recompute the weight. `created` exists ONLY in the DB row — no ledger carries it.
   - `note_meta`: 3 earned columns (incl. `review_priority`).
   - `review_schedule`: 4 earned columns (mirrored in `review-pulse.json`).
   - `note_state_history`, `shape_history`, `index_search_history`: earned outright.
   - `note_aliases`: `'rename'`/`'import'` rows earned; `'frontmatter'` rows recomputable.
   - Everything else: recomputable or cache. **This table IS the cargo manifest for every
     cross-universe operation.**
5. **Identity:** `cid_cn` is unique **per-universe only** (partial UNIQUE per DB;
   generation collision-checks only the target directory). A note moved between universes
   carrying a colliding cid makes the second note **invisible to the index**
   (SQLITE_CONSTRAINT_UNIQUE on upsert), and the repair path re-mints — which **severs every
   earned row keyed to the old cid**. A cross-universe move needs a cid collision protocol.
6. **Cross-universe links today:** the edge lives in the AUTHOR's DB; `target_cid_cn` stays
   NULL for a foreign target; the target's universe **never learns of the edge** — open it
   standalone and the backlink does not exist. The rename cascade refuses foreign referrers,
   so **cross-universe inbound links break silently on rename** (the PJ-253 family, now in
   scope as a correctness case of full operations).
7. **Cross-process reality:** `link_life`'s ledger lock is a process-local mutex
   (`link_life.rs:191`); `is_cuniverse_open_elsewhere` is a `BEGIN EXCLUSIVE` probe that in
   WAL mode **does not detect a mere reader/idle instance — false negative in the routine
   case**; `federation/migrate.rs` backs up a child DB with `fs::copy` (no `-wal` sibling)
   and restores over the live file on failure. All three are pre-existing hazards that become
   critical the moment writes cross universes. **Any option must fix them.**

---

## 2. The options, with verdicts (full text + attacks in the evidence file)

### Option A — ROUTE TO THE OWNER · **VIABLE-WITH-CONDITIONS · RECOMMENDED**

Keep one `search.db` per universe exactly as-is. Build a **routing layer**: an operation on a
note whose home is universe U opens U's DB (side-by-side handle, schema-gated, per-connection
tokenizer, U's link vocabulary) and does its bookkeeping THERE. The active universe's DB never
holds foreign rows. A cross-universe move = fs move + the §1.4 earned payload migrated between
the two DBs under a journaled two-phase protocol.

*Why it wins:* the only shape that keeps every universe folder **self-contained and portable**
(File-Over-App at universe scale); matches the repo's own hard-won lessons (the PJ-230 comment
forbids the registry-swap alternative; `ForeignSchemaOnly` and the ro-ATTACH are its seeds);
no new schema concept; each universe remains authoritative for its own earned data — B's one
sound principle, absorbed. *The attack found:* the vocabulary threading must reach the
fingerprint gates (condition 1); the two-instance probe is a false-negative (condition 2);
two-DB transfers need crash-consistency (condition 3). All fixable, none structural.

### Option B — FEDERATED INDEX WITH PROVENANCE · viable-with-conditions, NOT recommended

Active DB indexes federated rows with a provenance column; child DBs stay authoritative;
writes flow home under a sync contract. *The attack found two app-killer-class holes* (the
cross-process ledger append-vs-compaction race; attach-time `fs::copy` migration rollback
clobbering committed home-writes), plus a ghost-row identity model. It buys nothing A doesn't,
at the price of a permanent two-copy sync contract — the class of design this codebase has
repeatedly paid to remove (one truth, many readers).

### Option C — ONE KNOWLEDGE STORE · **NOT VIABLE AS WRITTEN**

A single machine-wide store fails structurally on FTS5: `notes_vocab` has no per-universe
form (the Index panel breaks by construction); **BM25 statistics become machine-global**
(registering universe B changes ranking inside universe A — a cross-universe leak no WHERE
clause can reach); top-K MATCH queries lose their index-only path. And there is **no durable
universe identity** to key rows by (registry ids are machine-local and destroyed on remove;
paths break on move). Its honest form is a hybrid that forfeits the simplification it was for.
**PJ-262 note:** if the Living Link disk layer someday moves earned data into files, the DB
becomes disposable and this option's calculus changes — that is PJ-262's migration, not a
reason to couple the two.

---

## 3. Conditions of Option A (blocking, from the adversarial pass — Plan phase must schedule each)

1. **Per-universe link vocabulary threading**, end to end: connection open → trigger DDL →
   `index_note` parse chain → the `is_built`/`stored_vocab_fingerprint` gates. A routed write
   that silently skips incoming-aggregate maintenance is the divergence class this migration
   exists to kill.
2. **A real cross-instance protocol.** The WAL `BEGIN EXCLUSIVE` probe cannot detect an idle
   second instance. Design: an OS-level lock file per universe (`.constellation/owner.lock`,
   acquired by the instance that has the universe ACTIVE; routed writers acquire it
   share/deny-write or refuse with a plain message). **Refuse, never last-writer-wins.**
3. **Journaled two-phase transfer** for move/copy: write-intent journal → fs move → source-DB
   row export → destination-DB import (cid collision checked FIRST) → source cleanup → journal
   clear; resumable at every seam (the mig108.rs engine is the in-house precedent).
4. **cid_cn collision protocol** on cross-universe entry: check destination's cid index before
   the move; on collision, re-mint AND re-key the travelling earned rows to the new cid in the
   same transaction — never sever.
5. **The pre-existing cross-process hazards fixed first**: `link_life` ledger gets an OS-level
   lock (or home-append refusal when the owner lock is held elsewhere);
   `federation/migrate.rs` backup/restore retired from `fs::copy` to the safe pattern.
6. **Watcher + suppress become universe-aware** for routed writes (the suppress map is
   path-keyed and survives; the watcher fan-out already spans `$libraries` — verify, don't
   assume, in Plan).
7. **Cross-universe rename cascade** (the PJ-253 family): with both DBs open, a rename may now
   heal foreign referrers — as a *deliberate* part of the operation, surfaced in the UI.

## 4. Invariants that must not break

- A universe folder remains **self-contained and portable** — copy it to another machine and
  nothing dangles.
- **Earned data is never lost silently** — every transfer is journaled and resumable.
- **Automatic writes never cross universes on their own** — only user-commanded operations do
  (MIG-065 §J survives as the *mechanism* for the background class).
- Boot time, typing latency, IPC responsiveness — unchanged (routing opens child DBs lazily,
  never on boot).
- Two instances never corrupt each other — refusal, not racing.
- The Editor-Surface Gate checklist passes on federated notes exactly as on own notes.

## 5. Prior rulings — the reconciliation the Boss must confirm (Phase 2, first item)

| Ruling | Disposition proposed |
|---|---|
| MIG-056 "federation is read-only" | **SUPERSEDED** as contract; "each universe owns its writes" survives as Option A's shape |
| Concept paper 25-federation §3/§10 ("never writes a child") | **SUPERSEDED** — amend in this migration; its File-Over-App justification was spurious (that law forbids *silent* writes, not commanded ones) |
| MIG-065 §J (write-validation own-scope) | **SUPERSEDED as contract, RETAINED as mechanism** for automatic/background writes |
| Interim guards' refusal text ("reads but never writes") | **Wording superseded** — when each door ships, the refusal routes to the door |
| 2026-07-05 "It is ONE universe" | **REINFORCED** — this migration is its completion |
| 2026-08-10 "universe-wide right for RESOLVING, wrong for CHOOSING" | **AMENDED**: automatic targeting stays own-scope; *deliberate user choice* may span universes through clearly-marked affordances |
| MIG-108 "One Universe, One Location" | **COEXISTS** — it constrains layout (libraries under their root), not operations |
| MIG-100 write-auth = registry membership | **COEXISTS** — and supplies the door's authorization shape ("writable through its own universe's identity") |
| PJ-224 (does the ordinary search box federate?) | **STILL THE BOSS'S GATED RULING** — this direction suggests yes, but it is asked, not assumed |

## 6. Phase-2 decision points for the Boss

1. **Confirm the repeals table** (§5) — explicit, not silent.
2. **The door's UI shape**: linked universes re-listed in the pickers under a clearly-marked
   section (planet icon + confirmation), vs a separate "Move to another Universe…" command.
3. **Wave order**: recommended Wave 1 = move + copy (the transfer engine); Wave 2 =
   edit-in-place/tag/property/task on federated notes (the routing layer); Wave 3 =
   create-in/rename-in/delete-in + the cross-universe cascade. Each wave lands
   Boss-tested behind the existing guards.
4. **Two-instance policy**: confirm *refuse with a message* over any merge semantics.
5. **PJ-224**, now in frame.
