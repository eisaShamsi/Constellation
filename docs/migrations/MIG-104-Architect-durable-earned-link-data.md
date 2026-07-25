All load-bearing claims verified independently. Writing the Architect document.

# MIG-104 — The Durable Home for Earned Link Data

**Phase 1 — Architect.** Status: for Boss ruling. No code written.
Date: 2026-07-24. Author: Claude. All measurements taken this session against `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db` (2,026,405,888 bytes = 1.89 GB), opened read-only/immutable.

---

## The concept (the horse)

> **A link's earned life — how many times you walked it, when you last walked it, how confident you became in it, and your decision to retire it — is knowledge *you* created by using your Library. It must live in a plain-text file you own, so that the index can be thrown away and the knowledge cannot.**

The carriage (the function) is a small durable store plus a restore pass. The horse (the concept) is that `search.db` is a **conclusion**, and conclusions may be recomputed — but the **evidence** of your own intellectual traffic may not be.

---

## What is at stake today

Everything a link *is* — that it exists, its type, what it points to, its annotation — is already safe. It is re-parsed from the `[[type::target|annotation]]` text inside your own note (`parse_link_body`, `search.rs:5345`). Delete the index and all of that comes back.

**Four things do not come back.** They exist only inside `search.db`:

| What it is | Where it lives | Why it can't be recomputed |
|---|---|---|
| How many times you walked a link | `note_links.traversal_count` | It's a record of your clicks. Nothing on disk knows. |
| When you last walked it | `note_links.last_traversed` | Same. |
| How confident you became | `note_links.confidence` | Auto-promotes with use; `contested` is a deliberate judgment you made. |
| Your decision to retire a link | `note_links.status = 'archived'` | **Worse than lost — silently reversed.** |

Plus `note_meta.review_priority` (a note's review priority, 0 set today).

### The three ways this bites, in order of severity

**1. Archive is silently reversed by the next save of the note — no index loss required.** This one is live, today, and it is not a durability problem at all. When you archive a link, the wikilink stays in your note (archival, not deletion — correct). The next time that note is saved, `index_note` compares the note's links against the stored ones. The "unchanged, skip it" fast path requires the stored row to be `status = 'active'` (`search.rs:6256-6264`) — an archived row never qualifies. So the row is deleted and re-inserted with `status` **hardcoded to `'active'`** (`search.rs:6290` and `:6296`). One edit to the note body un-retires the link, with no error and no notice.

I measured 0 archived links in your Universe, so nothing is currently damaged. But it means the Retire feature does not currently hold. This is app-killer-class (silent reversal of an explicit user decision) and per Working Agreement #6 it must be fixed in this pass, not logged.

**2. Renaming or moving a note wipes earned data — again with no index loss.**
- Rename a link's **target**: the preservation key is `"{link_type}::{target_name}"` (`search.rs:6208`, looked up at `:6286`). The rename cascade rewrites `[[Foo]]` → `[[Foo v2]]` in every note that links to it (`libraries.rs:5675`), so the key no longer matches. Every inbound link resets to `hypothesis / weight 1.0 / 0 traversals`.
- **Move** a note: `reindex_delete_note` deletes every one of its outgoing link rows (`search.rs:9437`), then re-indexes at the new path where the preservation map is empty.
- **External** folder rename (Explorer, Syncthing): the code's own comment concedes it — *"per-note aux (review schedule, own outgoing-link weights) resets — acceptable for a rare external rename"* (`search.rs:9673-9675`).

**3. Losing `search.db` is total amnesia for all four fields.** The 2026-07-24 safety inspection already hardened this: a schema-version bump now **renames the database aside** instead of deleting it (`search.rs:8883-8911`), and the comment there names this migration as the cure — *"The durable cure — giving earned link data a home outside this derived store — is the LINK-file layer; see docs/migrations/MIG-104-*."* That directory does not exist yet; this document creates it.

### The honest scale — measured, not estimated

| Measurement | Result |
|---|---|
| Notes | **7,812** |
| Links | **234,192** |
| Links with any earned state at all | **35** (0.015%) |
| — of which archived | **0** |
| — of which non-default confidence | **2** |
| Total traversals ever recorded | **41** |
| Source notes holding earned links | **25** (0.3%) |
| Notes with `review_priority` set | **0** |
| `cid_cn` present on notes | 7,795 / 7,812 (99.8%) |
| `source_cid_cn` on links | 233,956 (99.9%) |
| `target_cid_cn` on links | 233,625 (99.75%) |
| Most links in one note | **535** |

**Read this number honestly in both directions.** 35 earned links is *not* a crisis — nothing large is at risk right now. But it is also the reason to build now rather than later: the durable store starts at 35 records, the back-fill is instant, and the mechanism is proven long before it is carrying years of your reading. The numerator is bounded by human clicks, not by corpus size — it will never be a 234k-row problem.

---

## The territory

### Producers — every write path that touches earned state

**Link earned state (5 writers, all DB-only, none touch disk):**

| Site | What it writes | Frequency |
|---|---|---|
| `search.rs:7326` `constellation_link_traverse` (UPDATE at `:7371-7385`) | `traversal_count+1`, `last_traversed`, `weight`, dormant→active, auto-`confidence` | Highest — once per wikilink follow, throttled |
| `search.rs:8321` `constellation_link_set_confidence` (`:8336`) | `confidence` (incl. user `contested`) | Rare |
| `search.rs:8385` `constellation_link_archive` (`:8396`) | `status='archived'`, `weight=0.0` | Rare |
| `search.rs:8409` `unarchive_link_rows` (`:8428`) | `status='active'`, weight recomputed from `traversal_count` | Rare |
| `search.rs:8349` `constellation_link_backfill_confidence` | bulk confidence promotion | One-shot |

**Review state:** `review.rs:634` `set_review_priority` (DB-only) · `review.rs:667/697/721` `mark_reviewed`/`snooze_note`/`dismiss_note` (these already write `review-pulse.json` — see below).

**Destroyers:** `search.rs:6244`/`:6270` (index_note DELETE) · `search.rs:9437` `reindex_delete_note` · `search.rs:9676` `delete_rows_under_prefix`.

**Identity migrators:** `libraries.rs:1149` (rename) · `reconcile.rs:275` (relocate) · `mig003_step4.rs:203`.

**A verified inconsistency the design must resolve.** All four earned-state writers match rows on `source_path AND LOWER(target_name)` — **with no `link_type` predicate at all** (`search.rs:7343`, `:8336`, `:8395`, `:8419`). A single traversal today bumps *every* type-variant of that pair identically. But `index_note`'s preservation map is keyed `"{link_type}::{target}"` (`search.rs:6208`). The writers are type-blind; the preserver is type-aware. Any durable store must pick one, and the choice changes what "this link's history" means (see Open Questions).

### Consumers — 16 surfaces read earned data, and **not one touches the filesystem**

All sixteen go through SQLite: 6 via live SQL, 2 via the `link_stats_cache` JSON snapshot, the rest via frontend stores derived from those IPCs. Backlinks panel (`cache.rs:504`), Outgoing panel (`cache.rs:541`), the editor's `×N` traversal chips (store-derived, `livePreview.ts:1511`), Knowledge Health, CCS View (6 registers + Retired), formulation analysis (8 diagnostics), Sight "Hubs", dormant links, the Reviewer's weight-ordered staleness probe (`review.rs:131`), the second-screen Cockpit lenses, Sight v6, ConfidencePicker, and the shared tier/decay derivations in `store.ts:3931-3985`.

**This is the single most important structural fact in the territory.** It means a durable store can be added as a *write target and restore source* without touching a single reader, without adding a byte to any boot payload, and without ever creating a moment where two code paths disagree about where `traversal_count` comes from.

### Weight and decay — nothing time-varying needs storing

`weight` is `1.0 + ln(1 + traversal_count)` (`earned_link_weight`, `search.rs:7316`), and `unarchive_link_rows` (`search.rs:8428`) **already regenerates it from `traversal_count` alone**. It is a materialised derivative.

Decay is display-only in TypeScript (`effectiveLinkWeight`, `store.ts:3965`), computed at sort time against a live user half-life slider. The write-time decay job was deleted 2026-06-10 (`aa9941ee`) for compounding corruption; `constellation_link_decay` is now read-only with zero frontend callers.

**Consequence: no file is ever touched because a day passed.** The durable payload is four event-shaped fields — a monotonic counter, a timestamp, a 4-value enum, and a boolean. (Note: CLAUDE.md's "5% monthly decay" does not match the code — the implementation is a 60-day half-life ≈ 29.5%/month. Already logged at `docs/Living-Links-Guide-v1.0.md:397`.)

### Boot surface

Default config is `perNoteLinkQueries: true` (`store.ts:5477`), so **the 234k-edge array is not loaded at boot at all today**. Earned data hydrates per-note via `get_backlink_rows` + `get_outgoing_rows` on a tab-keyed effect (`+layout.svelte:1731-1788`). Boot Phase 1 (`cache_boot_snapshot_core`) is awaited; Phase 2 (tags + aliases) and Sky run on idle callbacks. Proven background-work precedents to clone: `link_boot_index.rs:56-76` (stamped in `schema_versions`, spawned after paint, dedicated connection, failure non-fatal), `review_backfill.rs`, `links_backfill.rs`.

### Rename / move surface

`cid_cn` is the rename-proof identity and it already exists in the user's own frontmatter (`mig003_backfill_cid_cn`, `search.rs:2218`; stamped at creation, `libraries.rs:876`), is UNIQUE-indexed (`idx_note_meta_cid_cn`), and is already joined on by the review staleness probe (`review.rs:136`). Coverage measured above: 99.8% of notes, 99.9%/99.75% of link endpoints.

### Sync + watcher surface

- `.constellation` is in `EXCLUDED_DIRS` (`file_kinds.rs:18-20`) and skipped by the sidebar walk (`libraries.rs:1958`).
- The file-tree walker matches only `md`/`base` (`libraries.rs:2766`); the library index walk only `md` (`search.rs:6323`); `index_note` bails on non-`.md` (`search.rs:5811`).
- The watcher filter passes gone-paths, directories, and `.md` files (`watcher.rs:77-80`).

**Verified landmine.** The watcher's filter is `Err(_) => true` (`watcher.rs:78`) — a path whose `metadata()` fails passes. A temp file that has just been renamed away is exactly such a path. It flows to `reindex_changed_paths` Pass 2, takes the non-`.md` branch, and calls `delete_rows_under_prefix` (`search.rs:9793-9797`), which **acquires the DB writer lock and runs `SELECT path FROM note_meta`** — collecting and lowercase-normalizing all 7,812 path strings before finding zero victims (`search.rs:9676-9700`). Per flush. Forever. Any temp+rename inside a watched tree must call `watcher_suppress::mark()` on **both** paths, as `write_gate.rs:249-250` does.

- **No `.gitignore` is written by the app** for `.constellation/`. The 1.89 GB `search.db` currently sits in the same directory as any new sidecar from a sync tool's point of view. The Boss's Universe is **not a git repo today** (verified: no git artifacts at the root); how it is synced, I could not determine.
- **`universe::atomic_write` (`universe.rs:116-139`) is temp + `sync_all` + rename — but builds a FIXED `<name>.tmp`** (`:117-120`), so two concurrent saves of one file collide. That is open **PJ-087**. `write_gate.rs:284-328` (`ReplaceFileW`) and `cece/reliability.rs:155-177` (`tempfile::persist`) both use unique names; a new store must too.
- **The app-close flush hook EXISTS** — `lib.rs:691` `WindowEvent::CloseRequested` → the PJ-103 handshake at `lib.rs:286` (emits `session:final-flush`, awaits ack, 5 s cap, instant when clean). A durable store can join it. *(One of the option papers stated no such hook was found; that is incorrect.)*

### The in-repo precedent that already does what this migration wants

`review.rs` **already implements file-as-truth**: `mark_reviewed` writes `review-pulse.json` first (`:687`), *then* mirrors to the DB row (`:689`); `recompute_all_in` (`:1231`) rebuilds the whole `review_schedule` cache from `note_meta` + the JSON. That is the exact architecture. It also carries three defects inside this migration's territory:
- `save_pulse_data` is a bare `fs::write` (`review.rs:762`) — not atomic, no fsync, not gated (**PJ-075**).
- `load_pulse_data` silently returns `default()` on read *or* parse failure (`review.rs:747-757`) — a corrupt file loses all review history with no error.
- It is keyed by **absolute note path with no rename migration**, so every rename orphans the ✓ history and the next reconcile rebuilds the note as `never_reviewed`.
- Separately, `ensure_note_meta_review_columns` (`search.rs:995-999`) contains an `ALTER TABLE note_meta DROP COLUMN review_priority` that wipes every override if it meets the legacy shape.

---

## Invariants that must not break

**Durability & correctness**
1. **Read-time answers always come from SQLite.** The durable store is write-only in normal operation and read-only during restore. There must never be a moment when two code paths disagree about the source of `traversal_count`. (Guarantees zero reader changes across all 16 surfaces.)
2. **The note decides a link's existence; the durable store decides what it earned.** Stated as a rule, because a second store means the two can disagree.
3. **Absent ≠ empty for earned data.** The config-loader contract (silently default to empty — `universe.rs:101`, `review.rs:747`, `link_types.rs:506`) must **not** be copied. A corrupt store is renamed aside, never overwritten; the DB is the fallback; a fresh store is not written until acknowledged. This is `search.rs:735-746`'s own rule, already fought for.
4. **Never delete, always rename aside.** `search.rs:8883-8911` precedent.
5. **A failed write is surfaced and retried — never `.catch(()=>{})`, never a fire-and-forget task as the mechanism of record.** LL-033; the `propertyTypeRegistry.ts:44-48` finding.

**Reversibility**
6. **Every link operation reversible — archival, not deletion.** Removing a wikilink from a note must not destroy its earned record; it goes dormant and returns if the link is re-added. This is CLAUDE.md's Dormancy → Renewal lifecycle, currently unimplemented.
7. **The archive decision must survive a save of the source note.** Non-negotiable; it is broken today.

**Performance (hard constraint — measured on 7,812 notes / 234,192 links / 1.89 GB, before and after)**
8. **Zero `invoke()` and zero file I/O on the keystroke path and on the link-click path.** Traversal fires inside `openNoteTab` (`store.ts:2326-2346`) concurrent with `{#key}` editor teardown/mount — the heaviest interaction in the app. Only an O(1) in-memory operation is acceptable there.
9. **`index_note` may ENQUEUE but must never WRITE A FILE.** It runs inside `BEGIN IMMEDIATE`/`COMMIT` (`search.rs:6306`) holding the writer lock, on the 1500 ms save debounce, on notes with up to 535 links. File I/O there is the PJ-066 canonical freeze shape.
10. **Zero bytes added to any boot payload; nothing new read on the awaited boot path.** Any back-fill or restore runs on a background thread after paint, batched, resumable, stamped in `schema_versions`.
11. **Any bulk `note_links` UPDATE must be batched and background** — each one fires `note_links_sky_au` (DELETE + INSERT on 234k-row `sky_links`, `search.rs:3858-3874`) plus the outgoing AI/AD/AU pair's two `note_meta` UPDATEs.

**Watcher / adopt**
12. **No temp+rename inside a watched tree without `watcher_suppress::mark()` on both paths** — else a writer-lock-held full table scan per flush (verified above).
13. **An externally-changed store (Git pull, Syncthing) has no adopt handler today** — `review-pulse.json` has the identical gap. Either accept parity explicitly or add a reconcile-time re-read. Do not claim it is solved.

**Cross-platform (macOS mandated later)**
14. **Windows rename-over-existing needs `ReplaceFileW` or `tempfile::persist`** (`write_gate.rs:284-328`, `cece/reliability.rs:123-131`).
15. **Key on `cid_cn` (ASCII timestamp + hex), never on a filename** — this sidesteps macOS NFC/NFD and case-sensitivity entirely.
16. **Unique temp names** (PJ-087).

**Sync**
17. **Deterministic serialization** — sort before writing, fixed field order, never HashMap iteration order (`arabic/overrides.rs:392-396`).
18. **Nothing rewritten because time passed.** An idle Universe produces zero diffs.
19. **Merge must be commutative and idempotent** — a duplicated region, a restored older copy, a "keep both" conflict resolution must all fold to the same answer.

**Doctrine**
20. **Write-Time Derivation (Rule 8)** — no `scan_*` / `rebuild_*` that re-walks the Universe on boot to produce a derived view.
21. **Form-Aligns-To-Purpose** — store the earned subset, not 234k rows of zeros.

---

## Options considered

Four designs were developed in full and judged by a four-member panel. Aggregate Borda ranking: **journal 9, link-files 6, sidecar 6, frontmatter 3.**

### Option A — Earned state in the source note's frontmatter

A reserved `link_state:` key in the note's own YAML, as a sequence of flow-maps (`- { to: …, cid: …, type: …, n: 3, at: …, conf: evidence, retired: true }`). Rows exist only for earned links. Written at the note's existing save seams (debounced save, `flushIfDirty` on departure, close), plus an idle flush for a note that was traversed but not edited.

- **Speed:** slowest to land (touches the frontmatter writer). **Effort:** ~9 files, 5 steps. **Risk: HIGH.**
- **The strongest thing about it:** recovery is not a new subsystem. `index_note` already does `fs::read_to_string` + `parse_frontmatter` (`search.rs:5849-5851`) and already builds the preservation map (`:6172-6212`); the change is substituting the file for the DB as that map's source. Delete `search.db`, boot, and the cold walk restores everything on the way in. No restore module, no cursor, no progress bar. It is also the only design where the earned life ends up in a file the user genuinely authors and opens daily — the purest reading of File Over App.
- **What it gives up — and why it ranked last with three of four judges:**
  1. **The merge hazard is a correctness cliff, not churn.** Two devices traversing the same note produce Git conflict markers *inside the frontmatter fence*. `yamlDoc.ts:300` — verified — returns the original frontmatter bytes verbatim on **any** YAML parse error and drops every property edit while the save still reports success (PJ-085). Trading "lose 35 traversal counts" for "silently disable property editing on that note" is a bad trade.
  2. **Reading becomes writing *and* reindexing.** Traversal fires inside `openNoteTab`, so the departing note becomes dirty; the departure flush then rewrites the whole `.md` through `write_gate` (temp + fsync + `ReplaceFileW` + retry) **and** triggers a full `constellation_search_reindex` — FTS delete+insert, the link diff on up to 535 links, `review_schedule` upsert, `note_meta` upsert with triggers — inside `BEGIN IMMEDIATE`. Write amplification to persist a 3-byte counter is orders of magnitude worse than any alternative.
  3. **It adds a fourth writer branch to `composeFrontmatter`** — the function PJ-137 counts six app-killers against, whose immutability guard (`yamlDoc.ts:186`, verified) is **one day old**.
  4. It deletes earned history on re-type and on link removal, with no archive — violating invariant #6.
  5. `git status` shows modified notes after an evening of *reading*.

### Option B — A per-library sidecar ledger (`link-state.jsonl`, current state)

One line per earned link, deterministically sorted, in `<library>/.constellation/link-state.jsonl`. An in-memory `LinkStateLedger` per library behind an `RwLock`; a flusher thread on a Condvar (10 s idle / 5 min cap / 200 dirty records / app close / before any destructive DB op). Flush is read-merge-write, never blind. Sibling `note-state.jsonl` for review priority + a path→cid re-key of the review pulse.

- **Speed:** medium. **Effort:** 4-6 phases. **Risk: MEDIUM.**
- **Strengths:** it has the right asymptote (bounded by earned link count, never by history — no compaction problem ever exists). It has the most complete review coverage of the four. It found the watcher landmine itself. Its corrupt-file contract is the best-written in the field.
- **What it gives up:**
  1. **Archive — the one datum whose reversal this migration exists to stop — rides the same 5-minute snapshot flush as a click counter.** The value is not proportional to the guarantee.
  2. Whole-file rewrite: bytes-per-flush is O(all earned records), not O(changed). 7 KB today; ~10 MB every 5 minutes at 50k records, plus a whole-file re-send on iCloud/Dropbox.
  3. It must pay off the watcher landmine and PJ-087 with **discipline** (two lines a refactor can silently drop) rather than with structure.
  4. Its author never located the live 1.89 GB database — every size and cadence figure in that paper is a formula, not a measurement.

### Option C — Real LINK files (`<library>/Links/YYYY-MM/…_LINK_XXXX.link`)

One file per *earned* link, with `kind: link`, `from:`/`to:` cid pair, the four earned fields, and a **body** where the user writes prose about *why these two ideas belong together*. Rare decisions write file-first-then-mirror; traversals are DB-first with a 30 s debounced flush.

- **Speed:** slowest of the non-frontmatter options (it needs a UI to be worth having). **Effort:** highest. **Risk: MEDIUM-HIGH.**
- **Strengths, and they are real:** it did the best measurement work of the four papers. Its `.link` extension trick is verified and clever — all four `.md` gates (`libraries.rs:2766`, `search.rs:6323`, `search.rs:5811`, `watcher.rs:79`) key on `md`, so a `.link` file adds no `note_meta` row, no FTS row, no tree entry, no watcher event; it reuses the exact pattern `write_conflict_sidecar` already ships (`libraries.rs:535-543`). Its placement insight is the sharpest doctrinal argument in the set: **the durable store must not sit beside its own failure mode** — whatever a user excludes to skip a 1.89 GB `search.db` will exclude all of `.constellation/`. It has the best sync shape (two machines traversing different links = zero conflicts) and the smallest corruption blast radius. And it is the only design that delivers what CLAUDE.md actually claims — links as first-class objects the user can open.
- **What it gives up:**
  1. **`Links/` is NOT in `EXCLUDED_DIRS`** (unlike `.constellation`), so the recursive index walk and the sidebar walk descend into it and every month shard. At 35 files that is noise; if the promotion predicate ever widens — the paper names this as its own risk #2 — it is a recursive walk over 234k inodes on a boot path that is a hard constraint.
  2. It declares the files write-only in steady state, so the collateral wipers (target rename, note move, archive-reversed-by-save) are repaired **only by a restore pass** — either a recurring UPDATE storm firing the sky triggers, or stale data between passes. The other three fix those for free with an in-memory lookup.
  3. Most surface added per unit of durability: a new extension, a visible directory needing a Boss ruling plus a tree-skip, three file lifecycle states, an unverified extension-collision assumption, and an inspector UI without which "the files are an invisible backup, durable, but not the concept."

### Option D — An append-only earned-life journal (`earned.jsonl`)

One NDJSON line per *event* (`walk` / `trust` / `retire` / `restore` / `priority`), per library, keyed by cid pair, with the traversal count written **absolute** (not a delta). Walks are coalesced into a 5 s buffer; decisions flush immediately; everything flushes at app close. A boot fold rebuilds the DB columns; the folded map stays resident and is consulted by `index_note` when re-inserting a link.

- **Speed:** fastest to a working durability guarantee. **Effort:** ~350 new lines + 6 one-line hooks + ~20 lines in `index_note`. **Risk: LOW-MEDIUM, concentrated in one place.**
- **Strengths:**
  1. **An append cannot clobber.** No temp file, no rename, no vanished path — therefore **structurally immune to the watcher landmine and to PJ-087**, rather than mitigating them. A torn tail costs one line; every earlier line is immutable.
  2. **Absolute `n` + max-fold makes merge idempotent by construction.** A duplicated region, a "keep both" Git resolution, a re-appended restored copy — all fold to the same answer. This is not a rule that can have a bug; it is arithmetic.
  3. **Minimum bytes written per event** (~200 B, existing data never rewritten) and the smallest possible Syncthing block delta.
  4. **It fixes all four collateral wipers live**, without needing an index loss to matter — via the resident folded map consulted at `search.rs:6286-6297` plus killing the hardcoded `'active'` at `:6290`/`:6296`. That turns this migration from insurance into a bug fix.
  5. **Best reversibility:** revert the commits, delete one `.jsonl` per library, and the Universe is byte-identical. Zero residue in notes, no new visible folders, no schema mutation.
  6. Cleanest slicing — full recovery-from-DB-loss lands in slices that never touch the save path.
- **What it gives up:**
  1. **Wrong asymptote.** Cost scales with total events ever, not with earned links. Compaction is deferred to v2 — shipping the unbounded shape and promising the bound later, which WA#6 disfavours.
  2. **An internal inconsistency that must be resolved before build:** the boot fold is described as a `schema_versions`-stamped one-shot, but the `index_note` overlay needs the folded map resident **every process** — so either the growing log is folded at every launch (an unbounded boot cost) or the overlay is empty after the first boot and the wiper fixes stop working.
  3. Multi-device concurrent traversal **undercounts** (max, not sum).
  4. Ships no user-facing surface, and creates no LINK file — CLAUDE.md's "links are first-class knowledge objects" remains unimplemented.

---

## Recommendation

**Build Option D (the append-only journal), corrected with a bounded snapshot, and grafted with the location insight from Option C and the failure discipline from Option B.** Call it the **Earned-Life Ledger: snapshot + tail**.

### Why

1. **It is the only design whose write mechanism cannot destroy what it is protecting.** Three of the four designs have a rewrite path, and a rewriter with a stale or empty in-memory map writes an empty store. An append has no such surface. For a migration whose entire purpose is durability, "the mechanism is structurally incapable of the failure" beats "the mechanism has a rule that prevents the failure."

2. **It is immune to the two verified landmines rather than disciplined against them.** No temp+rename means no vanished-path watcher event means no writer-lock-held full table scan (`search.rs:9676`), and no exposure to PJ-087's fixed temp name. Options B and C both pay these off with two lines a refactor can drop.

3. **It fixes the live bugs, not just the amnesia.** The ~20-line `index_note` overlay closes all four collateral wipers — including the archive-reversed-by-save defect — with an in-memory hashmap lookup and **no disk read on the save path**. Option C's restore-only model leaves them live between passes.

4. **Its risk is isolated to one 20-line slice that lands last and is optional for the durability payload.** Slices 1-4 deliver full recovery-from-DB-loss and none of them touch the save path. If schedule pressure appears, cut the last slice, not its verification.

5. **Zero reader changes, zero boot payload, zero interaction-path I/O** — the same as B and C, but with the lowest total surface added.

### Grafts

**From Option B (the sidecar) — a bounded snapshot, which fixes D's only serious defect.** Ship **snapshot + tail**, the standard WAL/LSM shape both papers half-invented:
- `earned.jsonl` = the append-only tail. Every mutation is an append.
- `earned.snapshot.jsonl` = one line per earned link, current state — bounded by earned count, never by history.
- Load = snapshot + tail, both bounded. This removes the unbounded fold, resolves D's internal inconsistency (the resident map is rebuilt from a bounded snapshot every launch, not from a growing log), and turns compaction from a deferred promise into a rare background "rewrite the snapshot, rename the tail aside" — never the mechanism of record. Trigger on a byte threshold (2 MB), never on a timer.

**From Option B — the corrupt-store contract, verbatim.** File missing + DB has earned rows → **the DB is the survivor; write the store from it immediately** (this doubles as both the one-time seed and the self-heal for a deleted store). One unparseable line → skip it, count it, **surface the count**. Structurally unusable → rename to `earned.corrupt-<UTC>.jsonl`, fall back to the DB, and **refuse to write a fresh store until acknowledged** — a blind overwrite destroys the backup that was about to save the user.

**From Option B — the `.gitignore`.** Write `.gitignore` containing `search.db*`, `boot-perf.*`, `diagnostics.log`. Cheap, independent of the winner, and it is what makes the File-Over-App claim operationally true: what syncs is the user's files, not a 1.89 GB derived binary.

**From Option B — Syncthing conflict-copy adoption.** Glob `earned*.jsonl` so `.sync-conflict-*` copies are folded automatically, then delete the copy — mirroring `write_conflict_sidecar`'s handling of the same class for notes. Nearly free, because the fold is already commutative.

**From Option C — the location, which is D's one real flaw.** Do **not** put the durable store in `.constellation/` beside the 1.89 GB `search.db`. A folder-level exclusion — the normal way a user keeps a huge binary out of Syncthing/iCloud/a backup set — would lose the store in the same event it exists to survive. A `.jsonl` at the library root is invisible to the file tree (`libraries.rs:2766` matches only `md`/`base`) and to the watcher (`watcher.rs:79`), so the cost is only visual. *(This is a Boss decision — see Open Questions.)*

**From Option C — the two-path write order, stated as a governing rule.** Rare deliberate decisions (archive, unarchive, set-confidence, review priority) write **file-first with an fsync, then mirror to the DB** — the proven `review.rs:687 → :689` order. High-frequency idempotent counters (walk) are DB-first with a coalesced flush. Writing this down is what stops a future contributor from "helpfully" making archive lazy.

**From Option C — the measurement discipline.** Put the "is this link earned?" predicate in **one** function with a comment naming the ratio: **35 earned of 234,192 links (0.015%), 25 of 7,812 notes.** Every size, cadence, and sync figure in this design depends on it.

**From Option A — two things worth keeping.** (a) Don't record a confidence tier when it is merely the auto-tier derivable from the count (`search.rs:7364`: ≥10 established, ≥3 evidence) — a traversal that only crosses a threshold then writes no trust event at all. (b) **Ship the conflict-marker guard regardless of which design wins:** detect Git conflict markers on read and refuse to save until resolved. PJ-085's H1 passthrough (`yamlDoc.ts:300`, verified) silently drops every property edit while reporting success — that is a live hazard *today*, and Option A's analysis should not die with the losing option.

**Also join the existing close-flush.** `lib.rs:691` `CloseRequested` → the PJ-103 handshake at `lib.rs:286` already exists and is Boss-validated.

---

## Rollout

Each slice is independently landable, independently revertible, and (where marked) Boss-testable. **Slices 1-4 deliver the full recovery-from-`search.db`-loss guarantee and none of them touch the save path.**

**Slice 0 — the measurement + the migration doc.** Create `docs/migrations/` (it does not exist, and `search.rs:8876` already cites it). Record the baseline: boot-to-`graph-ready`, note-open latency on a wikilink click, save latency on the 535-link note. *No user-visible change.*

**Slice 1 — `link_life.rs`: the appender + the union reader.** Purely additive — writes a file nothing reads yet, so it cannot regress anything. Clone of `classifier/correction_log.rs:70-86` (create+append NDJSON, 178 shipped lines). Includes the corrupt-store contract and the `.gitignore` write. **Reversible:** delete the file, revert the commit. *Not Boss-testable — internal.*

**Slice 2 — the 6 write hooks.** `search.rs:7385` (traverse), `:8338` (confidence), `:8362`+`:8370` (backfill), `:8398` (archive), `:8430` (unarchive), `review.rs:645` (priority). Each is an O(1) push into a buffer **after** the DB lock is released. Decisions flush immediately file-first; walks coalesce on 5 s. Joins the `CloseRequested` flush. Routed through **one** choke-point function so a future seventh writer physically cannot skip the store. **Toggle:** a single `EARNED_LEDGER_WRITE` flag; off = today's behaviour exactly. **Boss test:** follow a few links, archive one, then open the `.jsonl` in a text editor and read your own activity in plain English.

**Slice 3 — the back-fill (seed the store from the existing DB).** 35 records today. Idempotent by construction (absolute `n`, max-fold), background after paint, stamped. **This is the moment the currently-at-risk data becomes safe.** *Boss-testable: a status-bar line, then the file exists.*

**Slice 4 — `link_life_restore.rs`: the boot fold → DB.** Clone of `link_boot_index.rs:56-76` — stamped in `schema_versions`, background thread after paint, dedicated connection, resumable cursor, failure non-fatal and logged. Batched, because each `note_links` UPDATE fires the sky trigger. **Boss test — the headline one:** with the app closed, rename `search.db` aside, restart. The app rebuilds from your notes as it already does; the status bar reports the earned layer being restored; your traversal counts, confidence tiers and retired links come back. Nothing you created is lost.

**Slice 5 — the snapshot + compactor.** `earned.snapshot.jsonl` + a 2 MB byte-threshold compaction that renames the tail aside and never deletes. Bounds the fold permanently. *No user-visible change.*

**Slice 6 (LAST, GATED) — the `index_note` overlay.** ~20 lines at `search.rs:6286-6297`: consult the resident folded map when the DB-to-DB `preserved` map misses, and write back the recorded `status` instead of the hardcoded `'active'` at `:6290`/`:6296`. **This is the only slice that changes what gets written on a save.** Requirements, non-negotiable:
- Behind its own flag, with the old path intact for one build.
- In-memory snapshot lookup only — no file I/O, no lock contention with the flusher.
- A Reproduce-First harness proving all four collateral-wiper recipes red→green: (i) rename a link's target, (ii) re-type a link, (iii) move the source note, (iv) archive a link then edit the note body.
- The full 8-point Editor-Surface Gate Checklist, **Focus mode included**.
- A diff-scoped `safety-inspection` before the commit.
- Before/after measurement on the 535-link note and on boot.
**Boss test:** archive a link, then edit the note it lives in and save. The link stays retired. (Today it silently comes back.)

**Slice 7 — the adjacent defects (WA#6).** Fix `ensure_note_meta_review_columns`'s `ALTER TABLE ... DROP COLUMN review_priority` (`search.rs:995-999`); make `save_pulse_data` atomic and gated (`review.rs:762`); make `load_pulse_data` refuse to silently default on a corrupt file (`review.rs:747`); re-key `review-pulse.json` from absolute path to `cid_cn` so a rename stops orphaning the ✓ history. *Scope subject to a Boss ruling — see below.*

**Slice 8 — docs.** CLAUDE.md's Living Link section (currently corrected to say the LINK-file layer is unimplemented) gets its accurate replacement; help files + User Manual + 14 translations; the orientation doc bump in the same commit.

**Reversible toggle:** two flags — `EARNED_LEDGER_WRITE` (slices 2-5) and `EARNED_LEDGER_READBACK` (slice 6). Both off = today's behaviour byte-for-byte. Full revert = revert the commits and delete one `.jsonl` per library; no residue in any note, no schema change, no new visible folder.

---

## Open questions for the Boss

**1. Where does the store live?**
- **(a) `<library>/.constellation/earned.jsonl`** — tidy, invisible, matches every other app file. **Risk:** whatever you exclude from sync/backup to skip the 1.89 GB `search.db` will exclude this too, in the same event it exists to survive.
- **(b) `<library>/earned.jsonl`** — at the library root, beside your notes, so whatever backs up your notes backs this up. Invisible to the file tree and the watcher (verified). **Cost:** a file you'll see in Explorer/Finder next to your knowledge.
- *My recommendation: (b).* The whole point is surviving the loss of the thing in (a)'s folder. But it is your folder.

**2. Does re-typing a link keep its history?** Today `[[supports::X]]` → `[[contradicts::X]]` wipes the earned data. Verified: all four writers ignore `link_type` entirely and match on source + target name — so a type-free key is *faithful to what the code already does*, and it would preserve the history across a re-type.
- **(a) Keep it type-free** (faithful, preserves history). **Consequence:** 14 traversals recorded while it said "supports" carry into "contradicts."
- **(b) Make the key type-aware** and make the writers match. **Consequence:** a re-typed link starts fresh — a different claim deserves a fresh record — but this is a behaviour change to the four writers.
- *I lean (a) for fidelity, but "inheriting confidence across a reversed stance" is a knowledge-formulation judgment, not an engineering one.*

**3. How much of the adjacent review-state damage do we fix in this pass?** Four verified defects sit inside this territory (slice 7). WA#6 says discovered defects get fixed, not logged. **(a)** Fix all four here. **(b)** Fix the two silent-loss ones (the `DROP COLUMN` and the non-atomic write) here, and file the path→cid re-key as its own PJ. **(c)** File all four. *I recommend (a) or (b); (c) means shipping four known silent-failure bugs.*

**4. Does this migration also build the LINK-file surface — links as objects you can open and write about?** Option C is the only design that delivers CLAUDE.md's "links are first-class knowledge objects." The durability layer recommended here does **not**, and no durability design needs it. **(a)** Keep MIG-104 as pure durability and file the LINK surface as its own PJ. **(b)** Build both. *I recommend (a) — the durability layer is ~7 slices; adding an authoring surface roughly doubles it and mixes an insurance job with a feature job. But CLAUDE.md has claimed the LINK file kind for a long time, and it should stop being a claim.*

**5. Multi-device: are you syncing this Universe, and how?** I verified it is not a git repo today and could not determine your sync method. The answer changes the merge story materially:
- **Git or Syncthing:** append-only merges cleanly; conflicts fold correctly; nothing is lost.
- **iCloud / Dropbox / OneDrive:** whole-file last-writer-wins can lose the losing device's tail. The max-fold makes it converge over time rather than ratchet down, but bytes already overwritten are gone. This is a stated limitation that belongs in the help file, not a solved problem.

**6. Are you comfortable that traversal counts are *coalesced*, not exact?** A crash without a clean close can lose up to 5 seconds of *walk counts* (never a decision — those flush immediately). And with two devices, concurrent walks fold to **max**, not sum, so the count under-reports rather than double-counts. I believe under-reporting is the honest direction for a number feeding a logarithm, but "how many times did I walk this?" is your instrument, not mine.

---

### Not verified / explicitly unknown

- **Nothing here is measured for performance.** Every boot/latency claim above is an argument from the code path. Rule 8's hard constraint requires before/after numbers on the real 7,812-note / 234,192-link / 1.89 GB Universe; that is slice 0.
- **Whether a write inside `.constellation/` can surface as a parent-directory event on Windows `ReadDirectoryChangesW`** — the watcher passes directories unconditionally (`watcher.rs:79`) and the Universe root is itself a watched library. Must be checked live in Build, not assumed.
- **Whether `.link` is claimed by any OS or common tool** (relevant only if Option C is revived).
- **17 of 7,812 notes have no `cid_cn`**, and a small number of link targets are unresolved. Those fall back to a fragile name key. The Plan should decide whether to force-stamp the missing cids first (`canonical.rs:1224 ensure_cid_cn` already exists) so the fragile set goes to zero before recording begins.
- **`sky_links.weight`** is a third copy of the number — written by the AU trigger (`search.rs:3846`), read by nobody (`cache.rs:1103-1112` projects only source/target/type). Do not leave it standing after this migration: wire it or drop it in the same pass.