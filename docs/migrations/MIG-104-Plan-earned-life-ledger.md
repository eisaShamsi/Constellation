# MIG-104 — Phase 2: THE PLAN

**The Earned-Life Ledger — one append-only mechanism, two streams.**

Status: **APPROVED by the Boss 2026-07-27 — BUILD IN PROGRESS.**

> **APPROVAL RECORD (Boss, 2026-07-27).** All eight checklist decisions settled:
> **#1** accept (walks append immediately, no coalescing) · **#2** accept (Q3 = fix all four + PJ-163)
> · **#3** accept (one store per Universe in `.constellation/`, per Q1) · **#4 "Ok"** → **BUILD the
> continuous note-history mirror** (Slice 9 — the 19,481 existing rows stop being single-copy) ·
> **#5** accept (archive former names) · **#6 "Yes"** → **ARCHIVE THE NOTE BODY TOO** (new sub-slice
> 8b: the time machine must survive an emptied Recycle Bin, ~35 KB per deleted note) · **#7 "Ok"** →
> the LINK authoring surface ships as **sibling MIG-106**, opened immediately after Slice 6 validates
> (PJ-169); Q4's "build both" is honored by sequencing, not reduced · **#8** accept (the corrected
> `.gitignore` list — takes `.constellation/` from 2,836 MB to 38 KB).
>
> Plan approval = build approval: the slices cascade without per-step sign-off, stopping only at
> Boss-testable verification clauses and at genuine architectural surprise.

Original status line: for Boss approval. No code written at authoring time. Date 2026-07-27. Repo `main` @ `042802c5` (MIG-105 Stage 0).
All measurements in this document were taken **this session** against
`E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db` opened read-only
(`mode=ro`), or read out of the tree at that commit. Every mechanism claim carries a `file:line`.
Anything not verified is marked **[UNVERIFIED]**.

---

## 1. Function in hand + the concept (the horse)

**Function in hand:** the **Earned-Life Ledger** — a plain-text, append-only store in
`<universe>/.constellation/` that holds (a) what each link *earned* from your use of it, and
(b) the change history of a note that is being deleted, so it can be brought back later.

**The concept (the horse):**

> **The traffic of your own mind — how often you walked a link, when you last walked it, how
> confident you became, your decision to retire it, and how a note's thinking changed over
> time — is knowledge YOU created by using your Library. It must live in files you own, so
> that the index can be thrown away, a note can be deleted, and the knowledge cannot.**

The carriage is a ~200-byte NDJSON append. The horse is that `search.db` is a **conclusion**, and
conclusions may be recomputed — but the **evidence of your own intellectual traffic may not be.**
Under the Boss's 2026-07-26 ruling the same sentence now covers deletion: a deleted note's history
is **archived first**, never destroyed, because *"I am planning to enable Constellation… to restore
deleted notes/files… like a time machine."*

---

## 2. What changed since the Architect doc

### 2.1 The Boss's archive-first / time-machine ruling (2026-07-26) — the scope widener

On a genuine delete the note's change history must be **archived first**, not destroyed. Agreed
consequences, all already reflected below:

1. The archive lives **on disk** (not in `search.db` — the thing it insures against), is keyed by
   `cid_cn` (paths change), is **append-only**, and is readable **without Constellation**.
   That is the ledger's mechanism exactly → **ONE mechanism, two streams.** No second bespoke
   format (Whole-Ecosystem Fix Law).
2. `docs/concept-papers/Backup-System-Concept-Paper.md:15` carries a now-**false** hard constraint —
   *"the `note_links` / `notes_fts` / `sky_*` SQLite tables are the EPHEMERAL index — NOT a backup
   target; they are rebuilt from the files on restore."* Verified false for earned link data and for
   `note_state_history` (19,481 rows that exist nowhere else). Amended in Slice 14.
3. MIG-105 Stage 0 (`042802c5`) added purges of `note_summaries` / `note_state_history` /
   `sight_v3_layout` / `shape_history` / `sources_suggestions` to `reindex_delete_note`
   (`search.rs:9874-9878`). Under the ruling that path must **archive before purging** — a
   correction to already-shipped code, and its own slice (Slice 8).

### 2.2 The single most important correction in this Plan — the archive hook cannot go where the purge went

`PRAGMA foreign_keys = 1` on **every** production connection (rusqlite default; pinned by
`tests_pj150_fk_enforcement_reality`, `search.rs:13576-13631`, which asserts `PRAGMA foreign_keys == 1`
on an `init_db` connection). `note_state_history` declares
`FOREIGN KEY (note_path) REFERENCES note_meta(path) ON DELETE CASCADE` (live `sqlite_master` dump;
`cece/history.rs:61-76`). `note_summaries` and `sources_suggestions` declare the same.

Therefore **`DELETE FROM note_meta` at `search.rs:9845` already destroys the history**, thirty lines
**before** the explicit `DELETE FROM note_state_history` at `search.rs:9875`. That explicit purge is
a **0-row no-op in production** (reproduced in an isolated in-memory DB of the live schema with
`PRAGMA foreign_keys=ON`: 5 history rows → 0 after the parent DELETE; the 9875-equivalent statement
reported `rowcount = 0`; the FK-less `sight_v3_layout` row survived).

⇒ **An archive hook placed "before the Stage-0 purges" archives NOTHING. It must sit before
`search.rs:9845`.**

⇒ The comment shipped in Stage 0 at `search.rs:9869-9872` — *"The declared ON DELETE CASCADE on
three of these tables is inert in production (FKs never enabled)"* — **is false and contradicts the
FK-reality test landed in the same commit.** Corrected in Slice 2 (WA#6).

### 2.3 A second history destroyer the Architect never saw

`migrate_note_db_paths` (`libraries.rs:1094-1230`) does a **destination pre-delete** on every
rename/move: `DELETE FROM note_meta WHERE path = <new_path>` (`libraries.rs:1157`, cascades the
phantom's history) and `DELETE FROM note_state_history WHERE note_path = <new_path>`
(`libraries.rs:1190`). The comment at `:1180-1186` says this is deliberate — clear a **dead** note's
trail at the destination so two timelines never merge. Under archive-first that trail must be
archived, then dropped. A slice that hooks only `reindex_delete_note` leaves this hole open on every
rename that lands on a previously-occupied path → **Slice 10.**

### 2.4 MIG-105 Stage-0 facts absorbed (not re-derived)

* `migrate_note_db_paths` now covers **all 11 path-bearing tables** and is the ONE shared cascade
  (`libraries.rs:1157-1200`); `reconcile::relocate_row` delegates to it (`reconcile.rs:335`).
  It runs under `PRAGMA defer_foreign_keys` inside a transaction (`libraries.rs:1124-1133`).
* **PJ-164 / C8** (rebuild `note_state_history` / `note_summaries` / `sources_suggestions` with
  `ON UPDATE CASCADE`) was gated on Boss ruling **R2** — *"on a genuine delete, does the history die
  with the note or get archived first?"* **R2 is now ANSWERED = archive-first**, so PJ-164 is
  **unblocked** and lands here as **Slice 12**, after the archive exists to protect the data during
  the rebuild.
* `vitest.config.ts` is **glob-based** since PJ-157 — `include: ['tests/**/*.{test,spec}.ts',
  'src/**/*.{test,spec}.ts']` with a 4-entry exclude list. **A new test file under `tests/` runs
  automatically**; no registration step in any slice below.
* cid gaps: **15** empty-`cid_cn` rows live = templates (exempt by design, MIG-TPL §1) + trashed
  duplicates whose identity the guard correctly refused to steal.

### 2.5 Stale figures corrected by re-measurement (my own queries, this session)

| Claim in the Architect doc | Re-measured 2026-07-27 |
|---|---|
| 7,812 notes | **7,817** |
| 234,192 links | **234,233** |
| "17 notes have no `cid_cn`" | **15** (0 NULL, 15 empty-string) |
| 35 earned links | **35 confirmed** by the strict predicate — but **2 of them are `structural`** (`id 568654 'contains' tc=2`, `id 568645 'parent' tc=1`) which `index_note` already refuses to preserve (`search.rs:6560-6561`) and re-inserts with `traversal_count 0` (`search.rs:6639`). **Recordable set = 33.** |
| Total traversals 41 | **41 rows-sum, but 39 real clicks over 33 distinct `(source, target)` pairs** — the writer is type-blind (`search.rs:7724-7727`, no `link_type` predicate) so 2 pairs hold two type-variants each. |
| `weight` is "a materialised derivative" of `traversal_count` | **FALSE for 236 rows.** Live histogram: `1.0 ×233,964 · 0.564 ×119 · 0.526 ×115 · 1.6931 ×29 · 2.3863 ×2 · 2.0986 ×2 · 1.081 ×1 · 0.952 ×1`. `earned_link_weight` = `1.0 + ln(1+n)` (`search.rs:7699-7701`) has minimum **1.0**, so **every sub-1.0 value is arithmetically impossible** — decay residue from the job deleted 2026-06-10. **234 rows have `weight<>1.0` with `traversal_count=0`.** |
| `last_traversed` marks earned state | **FALSE — non-empty on all 234,233 rows.** `index_note` stamps it at insert (`search.rs:6639`, `:6657`, both bind `?6 = now` to `created` AND `last_traversed`). |
| "Archive is silently reversed by the next save" (defect #1, Slice 6) | **ALREADY FIXED in the tree.** `search.rs:6560-6566` now includes `status != "active"` in the preserved predicate and carries `status` as the 6th tuple element; `search.rs:6645-6653` binds `?14 = status` with the comment *"`status` is RESTORED, not hardcoded to 'active'."* The Boss test the Architect proposed for Slice 6 would pass today and prove nothing → Slice 6 re-scoped (now **Slice 13**). |
| `note_state_history` "19,443 rows" | **19,481 rows / 7,785 distinct notes / 15,103,149 bytes of `changes_json`** (avg 775 B, max 2,352). `MAX(history_id)=20,174` vs `COUNT=19,481` ⇒ **693 rows already destroyed** in this DB's life. |
| Line anchors | `constellation_link_traverse` is at **`search.rs:7710`** (not ~7326); `earned_link_weight` at **`:7699`**; `ensure_cid_cn` at **`canonical.rs:1256`** (not 1224). Architect anchors have drifted ≈ +380 lines. |

### 2.6 Four NEW defects discovered while planning (WA#6 — all fixed in-pass)

| # | Defect | Evidence |
|---|---|---|
| D1 | **~28% of the note-history stream is machine-generated no-op churn.** `parse_frontmatter` returns `HashMap<String,String>` (`search.rs:4893`) and `props_json = serde_json::to_string(&properties)` (`search.rs:6116`) with **no sort** → non-deterministic key order → the trigger's `OLD.properties_json IS NOT NEW.properties_json` guard (`cece/history.rs:105-134`) fires on **semantically identical** content. Measured: of 10,299 `properties_json` events, **2,861 (27.8%) parse to the identical dict**. Two `.trash` twins accrued **179 rows each, 175 of them no-op**, at ≈ **one row per app boot**. | Slice 2 |
| D2 | **`dedupeBySource` SUMs traversal counts** — `existing.traversalCount = (existing.traversalCount ?? 0) + (row.traversalCount ?? 0)` (`store.ts:4203`). With the type-blind writer, 2 live pairs display **double** the real clicks (`Islam → the four books`; `Link type test v2 → banana`). | Slice 13 |
| D3 | **A live writer-lock full-table scan, today, from inside `.constellation/`.** `cece/reliability.rs:155-158` persists a `tempfile` into `<library>/.constellation/` on every save; the vanished temp path passes `watcher.rs:77-78` (`Err(_) => true`) → `reindex_changed_paths` Pass 2 → `delete_rows_under_prefix` takes the **writer lock** and collects + lowercases all **7,817** `note_meta` paths to find zero victims (`search.rs:10108-10128`). Nothing in `.constellation/` is suppressed: `watcher_suppress::mark` is called only from `write_gate.rs:249,250,598,599,725,726,763`. | Slice 1 |
| D4 | **`reindex_changed_paths` Pass 1 has no dot-segment guard** (`search.rs:10189-10201`), so `.trash` notes are still indexed. Live: **62 `note_meta` rows at `.trash` paths**; 543 history rows across 40 of them. The index walk (`search.rs:6698`) and reconcile's walk (`reconcile.rs:396`) both DO skip dot segments; `has_dot_segment` exists at `libraries.rs:2693-2699` for read paths only. | Slice 2 |

---

## 3. The design, locked

### 3.1 Files — ONE location, per Boss ruling Q1

Q1 reads: *"location = `.constellation/` — inside the **Universe's** config dir."* Taken literally
and locked:

```
<universe>/.constellation/earned.jsonl              ← link-life tail   (append-only)
<universe>/.constellation/earned.snapshot.jsonl     ← link-life snapshot (bounded, 1 line/link)
<universe>/.constellation/earned.corrupt-<UTC>.jsonl← rename-aside target
<universe>/.constellation/note-history.jsonl        ← note-history archive (append-only, NEVER folds)
<universe>/.constellation/.gitignore                ← the exclusion file
```

**How every writer finds it with zero plumbing:** `conn.path()` → parent directory *is*
`.constellation/`. This is the trick `diag_log` already uses from inside these very functions
(`libraries.rs:1200-1202`, inside `migrate_note_db_paths`'s `run_lazy`). It works identically from
`search.rs`, `libraries.rs`, `reconcile.rs` and the watcher path — no `AppHandle`, no
`load_all_libraries`, no library lookup.

**The cost of per-Universe, stated honestly (Boss checklist item 3):** 18 of 20 registered libraries
live **outside** the Universe root (live `libraries.json`: 17 under `E:\Cognitive Knowledge\…`, 1 at
`E:\مشاريع كلاود\Constellation\lab\PJ-065-test-book`). If a library is detached from this Universe
and attached to another, its earned records stay behind. The per-library alternative
(`<library>/.constellation/`, the shape `classifier/correction_log.rs:71-81` already ships) would
travel with the library — 27 of the 33 records would land in per-library dirs whose entire current
content is ~3 KB — but it needs a library-root at every write site. Filed as **PJ-167** if the Boss
wants it later; the ruling as written is per-Universe.

### 3.2 The `.gitignore` — corrected against the live folder, and this is what makes Q1 SAFE

The Architect's Option-C objection to `.constellation/` was: *whatever you exclude to skip a 1.89 GB
`search.db` will exclude the store too.* Measured byte inventory of the live folder (`os.scandir`):

| bytes | name |
|---|---|
| 2,026,405,888 | `search.db` |
| **939,413,504** | **`Constellation SV Test.db`** ← a SECOND huge DB (PJ-159's orphan) |
| 4,206,552 | `search.db-wal` |
| 2,457,321 | `boot-perf.history.jsonl` |
| 1,545,338 | `diagnostics.log` |
| 32,768 ×2 | `search.db-shm`, `Constellation SV Test.db-shm` |
| … | 26 further small JSON/TSV/log files |
| **2,974,144,901 (2,836 MB)** | **TOTAL — of which SQLite `.db`/`-wal`/`-shm` = 2,970,091,480 B = 99.86%** |

Residuals computed with `fnmatch` over the real filenames:

* **Architect's list as written** (`search.db*`, `boot-perf.*`, `diagnostics.log`) → residual
  **939,492,801 B (895.97 MB)**. It **misses `Constellation SV Test.db` entirely** — the glob is
  anchored on the name `search.db`. **The objection survives the graft unchanged.**
* **Corrected list** → residual **38,294 B (0.04 MB)**.

**Locked content:**

```gitignore
*.db
*.db-wal
*.db-shm
boot-perf.*
diagnostics.log
sv-trace.log
```

**Two prohibitions, written into the slice:** never `search.db*` (leaves 896 MB); **never `*.jsonl`
or `*.log`** — `boot-perf.history.jsonl` must be excluded **by its own name**, because the ledger
shares that extension and a `*.jsonl` line would silently un-protect the exact file this migration
exists to protect.

**What this does NOT do, stated in the help slice:** `.gitignore` binds **git only**. Syncthing reads
`.stignore` at the *folder root*; iCloud/Dropbox/OneDrive and backup tools use their own config. **No
file inside `.constellation/` can stop a backup tool from excluding the folder.** Since Q5 = NO SYNC,
the residual exposure is the user's **backup set** — documented, and covered by the corrupt-store
contract's "store missing + DB has earned rows → the DB is the survivor, reseed immediately."
Verified today: **no `.gitignore`, `.git`, `.stignore`, `.stfolder`, `.stversions` or
`*.sync-conflict-*` anywhere in the Universe** (depth-4 `find`), and the app writes none — the only
`gitignore` hit in the whole tree is a dictionary row, `lexicon/data/lexicon_v1.tsv:7311`.

### 3.3 Record shapes — real lines, deterministic field order, `\n`, NFC, forward slashes, no drive letters

**Stream A — link life (`earned.jsonl`). Folds.**

```json
{"v":1,"t":"walk","cid":"20260703T091101Z_NOTE_A1B2","to":"20260512T144233Z_NOTE_77C9","tn":"the four books","n":3,"at":"2026-07-03T09:11:05Z","epoch":0}
{"v":1,"t":"trust","cid":"20260703T091101Z_NOTE_A1B2","to":"20260512T144233Z_NOTE_77C9","tn":"the four books","conf":"contested","at":"2026-07-18T08:17:49Z","epoch":0}
{"v":1,"t":"retire","cid":"20260703T091101Z_NOTE_A1B2","to":"","tn":"banana","at":"2026-07-18T08:20:02Z","epoch":0}
{"v":1,"t":"priority","cid":"20260703T091101Z_NOTE_A1B2","p":2,"at":"2026-07-18T08:21:00Z","epoch":0}
```

* **Key = `(cid, to)`** where `cid` = source `note_meta.cid_cn`, `to` = target `cid_cn`.
  **Type-free** (Boss ruling Q2 = keep history across a re-type) — which is also *faithful to what
  the code already does*: the writer matches on `source_path AND LOWER(target_name)` with no
  `link_type` predicate (`search.rs:7724-7727`).
* `tn` = lowercased target name — the **documented fallback key** when `to` is empty, and a
  human-readable label. **567 rows lack `target_cid_cn`** today (494 resolvable by name to a note
  that already has a cid; 73 genuinely dangling; **48 whose name matches >1 note**). Name keys are
  ambiguous for 7.4% of the corpus: **261 distinct `name_lower` values are shared by ≥2 notes,
  covering 576 notes.** So `tn` is a fallback, never a primary.
* `n` is the **absolute** count, never a delta.
* `epoch` is a reserved monotonic integer, always `0` today. **It exists because max-fold makes a
  DECREASE unrepresentable** — and the time-machine ruling guarantees a "forget this link's history"
  request eventually arrives. Reserving it now avoids a format break.
* `weight` is **never stored and never restored** — see §3.5.

**Stream B — note history (`note-history.jsonl`). NEVER folds.**

```json
{"v":1,"t":"nh","cid":"20260424T054559Z_NOTE_C3A4","lib":"Religion & Comparative Traditions","rel":"Abrahamic/Islam.md","hid":20174,"at":1785131711000,"k":"chg","ev":{"sources":null,"content_type":null,"properties_json":{"old":"{…}","new":"{…}"}}}
{"v":1,"t":"nd","cid":"20260424T054559Z_NOTE_C3A4","lib":"Religion & Comparative Traditions","rel":"Abrahamic/Islam.md","at":1785131711000,"mode":"trash","to":".trash/Islam 1.md","n":7}
{"v":1,"t":"nr","cid":"20260424T054559Z_NOTE_C3A4","rel":".trash/Islam 1.md","at":1785140000000,"n":7}
```

* `hid` = the source `history_id`, preserved verbatim as the **ordinal**. Non-negotiable:
  **765 `(note_path, captured_at)` groups collide, involving 1,536 rows** (worst = 5 rows in one
  second — `captured_at` is `strftime('%s','now')*1000`, whole-second granularity,
  `cece/history.rs:105-134`), and there are **2,066 inversions** between `history_id` order and
  `captured_at` order (the `_seed` block uses `modified*1000`, `cece/history.rs:219-234`). **A
  time-only ordering silently scrambles 1,536 rows.** `hid` is an ordinal only — never a join key,
  never re-used on restore (`AUTOINCREMENT`).
* `ev` = **`changes_json` verbatim.** Do not re-encode, do not strip the JSON `null` keys. The doc
  comment at `cece/history.rs:53-56` ("fields that didn't change are absent") is **wrong** and
  contradicts `:95-97` in the same file — live rows carry unchanged fields as explicit `null`
  (`hid=19496`: `{"sources":{"old":null,"new":"[\"perception\"]"},"content_type":null,
  "properties_json":null}`). Any transform makes the archive lossy relative to the table it insures.
* `k` = `"seed"` | `"chg"` — derivable, but explicit so the file reads **without a JSON parser**
  (File-Over-App). Live split: **7,611 `_seed` + 11,870 trigger rows.**
* `rel` = path relative to the **owning LIBRARY root**, forward slashes, NFC — **not** the Universe
  root: 18 of 20 library roots are outside it, so "Universe-relative" is impossible for 90% of
  libraries. Verified that every one of the 6 delete call sites can supply a library root:
  `libraries.rs:6577` / `:6659` / `:6662` and `reconcile.rs:260` all hold `app`; `search.rs:10155`
  sits in `delete_rows_under_prefix(state, dir_path)` whose only caller `reindex_changed_paths(app,
  paths)` (`search.rs:10167`) holds `app`; `search.rs:10217` is inside that same function.
* `lib` = library **name** (`note_meta.library_name`) — for human readability only, **never a join
  key**: a library rename must not orphan the archive.
* `n` on the `nd` marker = the count of `nh` records just written — the completeness check a reader
  and the tests use to detect a torn batch.
* `mode` ∈ `trash` | `system` | `permanent` | `vanished` | `reconcile-gone` | `displaced-by-move`.

### 3.4 Fold semantics — per stream, and they are NOT the same

**Stream A folds.** `n = max` · `at`/`conf`/`status`/`p` = latest by `at` then file order · union of
keys. Commutative and idempotent by construction — a duplicated region, a restored older copy, a
re-appended tail all fold to the same answer. **Max-fold, not sum-fold**, and Q5's *never concurrent*
ruling deletes Option D's stated defect #3 (concurrent under-count) from the risk list entirely; sum
would double-count on a re-appended tail, which is the event that actually happens.

**Stream B NEVER folds, and can never be snapshotted — only segmented by time.** The record *is* the
payload. Folding two events destroys the intermediate state, which is the entire value. Measured
proof that consecutive events are individually meaningful: `hid` 8251 / 8252 / 8253 record one
property key evolving `ma` → `mas` → `masadir`.

**This is the decisive argument for two files, not one.** The compactor's mechanism is "rewrite the
snapshot, rename the tail aside." If Stream B shared that tail, every compaction would copy a
15-MB-and-growing **unfoldable** stream verbatim — reintroducing the whole-file-rewrite hazard
Option D was chosen to be structurally immune to, and putting the history stream inside the one
operation that can lose it.

**Share the appender, not the file.** One module `link_life.rs` owns: `open(create+append)` → **one
`write_all` of `line + "\n"`** → the fsync policy → the corrupt-store contract → the `.gitignore`
write → `earned*.jsonl` / `note-history*.jsonl` conflict-copy adoption → the `CloseRequested` flush
(`lib.rs:691` → the PJ-103 handshake at `lib.rs:286-295`). Clone
`classifier/correction_log.rs:70-86` for the shape, but fix its three gaps: (a) it stores an
absolute note path (Q5 violation), (b) no fsync, (c) `writeln!(f, "{}", line)` issues **two** write
syscalls (fragment + newline) — build the full line including `\n` so a torn tail can only ever lose
the **last** line. Also hoist its per-call `create_dir_all` out of the hot path.

### 3.5 Write ordering — two paths, stated as a governing rule

| class | order | flush |
|---|---|---|
| **Rare deliberate decisions** — retire, unretire, set-confidence, review priority | **file-first + fsync, THEN mirror to the DB** — the proven `review.rs:687 → :689` order | immediate |
| **High-frequency idempotent counter** — walk | **DB-first, then append off the lock** | **immediate append, no per-event fsync**; fsync on a 5 s timer and at `CloseRequested` (see Q6, §4.2) |
| **Note-history archive** | **file-first + fsync, THEN purge. If the archive write fails, REFUSE the purge and return `Err`.** | immediate (batched per delete operation) |

Writing this down is what stops a future contributor from "helpfully" making retire lazy.

### 3.6 The ONE earned predicate — locked, with two explicit prohibitions

```rust
/// Is this link row carrying anything the user EARNED?
/// 33 recordable of 234,233 links (0.014%) across 25 source notes — measured 2026-07-27.
///
/// FORBIDDEN in this predicate, both verified against the live DB:
///  * `weight != 1.0` — 236 rows carry values `1.0 + ln(1+n)` cannot produce (115 at 0.526,
///    119 at 0.564, both with traversal_count = 0; 2 at 0.952/1.081 with traversal_count = 1).
///    Decay residue. Using it would seed the durable store with 236 junk records.
///  * `last_traversed != ''` — non-empty on ALL 234,233 rows; index_note stamps it at
///    insert (search.rs:6639, :6657 both bind ?6 = now).
///
/// `confidence = 'structural'` is NOT earned (PJ-065: structural edges carry no living-link
/// apparatus). index_note already refuses to preserve them (search.rs:6560-6561) and
/// re-inserts with traversal_count 0 (search.rs:6639).
traversal_count > 0
  OR status <> 'active'
  OR (confidence IS NOT NULL AND confidence NOT IN ('hypothesis','structural'))
```

Live result: **35** rows match; excluding the two structural rows leaves the **33** recordable.
`weight` is **derived on restore** from `n` (`earned_link_weight`, `search.rs:7699-7701`) — which
is exactly what `unarchive_link_rows` already does (`search.rs:8816`), and which **heals all 236
corrupt weights for free**.

### 3.7 The corrupt-store contract (from Option B, verbatim — invariant #3)

* **Store missing + DB has earned rows** → **the DB is the survivor**; write the store from it
  immediately. Doubles as the one-time seed and the self-heal for a deleted store.
* **One unparseable line** → skip it, count it, **surface the count**.
* **Structurally unusable** → rename to `earned.corrupt-<UTC>.jsonl`, fall back to the DB, and
  **refuse to write a fresh store until acknowledged.** A blind overwrite destroys the backup that
  was about to save the user. Precedent: `search.rs:8883-8911` renames the DB aside, never deletes.
* **Never copy the config-loader contract** (`universe.rs:101`, `review.rs:747`,
  `link_types.rs:506` all silently `default()` on read *or* parse failure). Absent ≠ empty for
  earned data.
* **Conflict-copy adoption:** glob `earned*.jsonl` / `note-history*.jsonl` so any
  `.sync-conflict-*` copy is folded, then removed — nearly free, because Stream A's fold is
  commutative. **Stream B copies are appended, never folded** (see §3.4); dedupe on
  `(cid, hid, sha256(ev))`.

### 3.8 Q5 portability rules — applied to every stored key

1. **`cid_cn` first, always** (ASCII timestamp + hex — sidesteps macOS NFC/NFD and case-sensitivity
   entirely). Fallback `rel` only when a cid is genuinely absent.
2. **`rel` is relative to the owning library root**, forward slashes, NFC-normalized, no drive
   letter, no `\`.
3. **`\n` line endings**, written explicitly — never `writeln!`'s platform default.
4. **Deterministic field order** (a `struct` with `#[derive(Serialize)]`, never a `HashMap`) — the
   same discipline `arabic/overrides.rs:392-396` already applies, and the absence of which is
   defect D1.
5. **UTC only.** Stream A uses RFC3339 (matching `note_links.last_traversed`,
   `chrono::Utc::now().to_rfc3339()` at `search.rs:7720`); Stream B uses epoch-ms verbatim from
   `captured_at`.
6. **Unique temp names** for the snapshot/compaction rename (`tempfile::Builder…tempfile_in`, the
   shape `cece/reliability.rs:155-158` uses) — never `universe::atomic_write`'s fixed `<name>.tmp`
   (PJ-087).

---

## 4. Q3 and Q6 — ANSWERED

### 4.1 Q3 — how much of the adjacent review-state damage do we fix in this pass?

**ANSWER: (a) fix all four, and fold in PJ-163.** One line to overrule: *"Q3 = (b) — fix the two
silent-loss ones and file the rest."*

**Evidence.** The four sit inside this migration's territory and three are silent-loss class:
`ensure_note_meta_review_columns` contains an `ALTER TABLE note_meta DROP COLUMN review_priority`
(`search.rs:995-999`) that wipes every override if it meets the legacy shape; `save_pulse_data` is a
bare `fs::write` — not atomic, no fsync, not gated (`review.rs:762`, PJ-075); `load_pulse_data`
silently returns `default()` on read *or* parse failure (`review.rs:747-757`) — a corrupt file loses
all review history with **no error**; and `review-pulse.json` is keyed by **absolute note path with
no rename migration**, so every rename orphans the ✓ history.

**Why (a) and not (b):** the path→cid re-key is the one that looks big, and it is exactly the helper
this Plan builds anyway (§3.8 rule 1–2). Doing it here costs one slice; doing it later costs the
helper twice. **PJ-163 (review-pulse RMW wipe)** lives in the same file and the same function pair —
fixing `save_pulse_data`/`load_pulse_data` without fixing the read-modify-write window would be
touching the file twice for one concern (Whole-Ecosystem Fix Law).

**What Q3 must be re-scoped to include** — three findings the Architect never saw, all in the same
WA#6 bucket: **D1** (the properties_json key-order churn, §2.6), **D2** (`store.ts:4203` sums
traversal counts), and the **weight heal** (236 rows off the curve, §2.5). These are in Slices 2 and
13, not deferred.

### 4.2 Q6 — are coalesced traversal counts acceptable?

**ANSWER: NO — reject the 5-second coalescing for walks. Append immediately, off the DB lock, without
a per-event fsync. Keep max-fold, and reserve `epoch`.** One line to overrule: *"Q6 = accept
coalescing, 5 s is fine."*

**The measured click rate (from the 35 `last_traversed` timestamps).** Max distinct links inside
**any** 5-second window = **3** (2026-07-03 09:11:01.772 → 09:11:05.917). Inside any 60 s window =
**6**. Densest real reading burst = **14 traversals in ~8 minutes**. By month (rows / traversals):
04: 2/2 · 05: 6/8 · 06: 5/6 · 07: 22/25.

**Why the loss is not "a slightly smaller number".** Δweight per lost click:
`n 0→1 = 0.693` · `1→2 = 0.406` · `2→3 = 0.288` · `9→10 = 0.095` · `99→100 = 0.010`. The Architect's
"under-reporting is the honest direction for a number feeding a logarithm" is true asymptotically and
**false in the region 100% of the live data occupies** — the distribution is `n=1 → 31 rows,
n=2 → 2, n=3 → 2`. **Nothing is above 3.** And the loss flips *categories*, not just magnitudes:
losing the **first** walk makes `linkLifecycle` return `'fresh'` at `tc===0` (`store.ts:4031`) instead
of `'emerging'` and `getLinkStage` return `birth` instead of `growth` (`store.ts:3197`, `:3202`), and
`effectiveLinkWeight` short-circuits `tc === 0` (`store.ts:4063`, `:4073`) so the decay/ordering
surfaces treat it as ageless. **The link reads as never walked.** Losing the walk that crosses n=3
loses a confidence tier (`search.rs:7747-7753`: ≥10 → established, ≥3 → evidence).

Honest worst case in the Boss's own numbers: **a crash within 5 seconds of reading loses up to 3
links' entire recorded existence** (measured peak), typically 1.

**There is no performance reason to buy that risk.** `constellation_link_traverse` is already
`#[tauri::command(async)]` (`search.rs:7709`), already takes the global writer lock
(`search.rs:7716`), already runs a SELECT + N UPDATEs (`search.rs:7754-7768`) each of which fires
`note_links_sky_au` (DELETE + INSERT on the 234k-row `sky_links`), and **nothing awaits the result** —
the caller is `invoke('constellation_link_traverse', …).catch(() => {})` (`store.ts:2444`), throttled
2000 ms per pair (`TRAVERSAL_THROTTLE_MS`, `store.ts:2176`, gate at `:2424`), with the user-visible
`×N` chip updated optimistically in memory via `queueMicrotask(() => bumpLinkTraversal(...))`
(`store.ts:2443`). One ~200-byte append **after the guard is dropped** is orders of magnitude cheaper
than what the call already pays, and invariant #8 ("zero file I/O on the link-click path") is
satisfied in substance because the click path never blocks on it.

**fsync policy:** no per-event fsync. `classifier/correction_log.rs:74-86` is the in-repo precedent
and it does not fsync — bytes reach the OS page cache immediately, which survives an **app crash**
(the common case); only a power loss / OS crash can lose the tail. fsync on a 5 s timer and at the
`CloseRequested` handshake. **If the Boss prefers a buffer anyway, cap it at 2000 ms** (matching the
existing throttle), never 5 s, and say so in the help file.

**[UNVERIFIED]** unclean-exit frequency. 814 boots are recorded in `boot-perf.history.jsonl`
(2026-06-15 → 2026-07-27) but nothing distinguishes clean from unclean exits, so I cannot state a
probability for the 5 s window. **This recommendation deliberately does not rest on that number** —
it rests on the measured click rate and the fire-and-forget IPC.

---

## 5. THE SLICES

**Spine preserved.** The Architect's rollout (appender → hooks → back-fill → boot-fold →
snapshot/compactor → the GATED `index_note` overlay → adjacent defects → docs) is intact; the mapping
column names the Architect slice each one descends from. **Toggles:** `EARNED_LEDGER_WRITE`
(Slices 3–7), `NOTE_HISTORY_ARCHIVE` (Slices 8–11), `EARNED_LEDGER_READBACK` (Slice 13). All off =
today's behaviour byte-for-byte.

| # | Arch. | What lands | Files / functions | Lands alone? | Boss-testable? |
|---|---|---|---|---|---|
| **0** | S0 | **Performance baseline** on the real Universe (7,817 notes / 234,233 links / 2.03 GB) + the Reproduce-First harness skeleton | `tests/mig104/` (new), `lab/reports/` | yes | no (measurement) |
| **1** | — | **`.constellation` watcher predicate** — makes Q1's location structurally safe; fixes **D3** live | `watcher.rs:77-80` | yes | **yes** |
| **2** | — | **Determinism + honesty pass (WA#6)**: sort properties before serializing (kills **D1**, ~28% of the stream); dot-segment guard on Pass 1 (**D4**); correct the two false comments | `search.rs:6116`, `:10189-10201`, `:9869-9872`, `cece/history.rs:53-56` | yes | **yes** |
| **3** | S1 | **`link_life.rs`** — appender, union reader, corrupt-store contract, `.gitignore` writer, conflict-copy adoption | `link_life.rs` (new), `lib.rs` | yes | no |
| **4** | S2 | **The 6 link-life write hooks** + the `CloseRequested` flush, all off-lock, one choke point | `search.rs:7768`, `:8338`, `:8362`, `:8370`, `:8398`, `:8430`, `review.rs:645`, `lib.rs:286` | yes | **yes** |
| **5** | S3 | **Back-fill** — seed the 33 records from the DB | `link_life_backfill.rs` (new) | yes | **yes** |
| **6** | S4 | **Boot fold → DB restore** (`link_life_restore.rs`) + the weight heal | new module; `search.rs` | yes | **yes (headline)** |
| **7** | S5 | **Snapshot + compactor**, 2 MB byte threshold | `link_life.rs` | yes | no |
| **8** | — | **Archive-before-purge** in `reindex_delete_note` — the Boss ruling. `DeleteCtx`, one transaction, real `Err` on archive failure, batch handle | `search.rs:9808-9917` + the 6 call sites | yes | **yes** |
| **9** | — | **Continuous note-history mirror** (GATED, recommended — see detail) | `search.rs` `index_note`, `link_life.rs` | yes | **yes** |
| **10** | — | **Archive the destination pre-delete** in the shared cascade | `libraries.rs:1157`, `:1190` | yes | **yes** |
| **11** | — | **Restore rejoin** — a re-appearing note reclaims its archived history | `search.rs:6374` (`index_note` funnel) + drain worker | yes | **yes** |
| **12** | — | **PJ-164 / C8** — child tables to `ON UPDATE CASCADE`, shared pragma helper, `foreign_key_check` quarantine gate | `search.rs` `init_db` + a stamped migration | yes | **yes** |
| **13** | S6 | **GATED `index_note` overlay** (re-scoped) + **D2** + the earned-predicate function | `search.rs:6524-6566`, `:6645-6653`, `store.ts:4203` | yes | **yes** |
| **14** | S7 | **Adjacent defects (Q3 = a)** + PJ-163 | `search.rs:995-999`, `review.rs:747`, `:762`, `:634-645` | yes | **yes** |
| **15** | S8 | **Docs** — Backup paper §3, CLAUDE.md, help ×15, User Manual, orientation bump, PJ ledger | `docs/**` | yes | no |

---

### Slice 0 — the performance baseline (Rule 8 hard constraint)

**Change shape.** No product code. Record, on the real Universe, before anything lands:
boot → `graph-ready` (from `boot-perf.latest.json` + `boot-perf.history.jsonl`, which already carry
the phase timings); note-open latency on a wikilink click (the traverse path); save latency on the
**535-link** note; and the cost of one 200-byte append + one `fsync` on the Boss's drive
(**[UNVERIFIED] today — no in-repo precedent measures it:** `review.rs:762` is a bare `fs::write`
with no fsync, `classifier/correction_log.rs:78-82` an unsynced append).

**Also lands:** `tests/mig104/harness.ts` + `tests/mig104/README.md` — the Reproduce-First recipe
registry every later slice appends to. Glob-driven vitest picks new files up automatically
(`vitest.config.ts` `include`), so no registration step.

**Tests.** `tests/mig104/baseline.test.ts` — asserts the harness can construct a temp Universe, a
library with `.constellation/`, and read back an appended line byte-exactly (`\n`, NFC, forward
slashes). Rust: `mod tests_mig104_baseline` — asserts `conn.path()`'s parent is the
`.constellation` dir on a freshly `init_db`'d temp DB (the mechanism §3.1 depends on).

**Verification clause.** The four baseline numbers are written into
`lab/reports/SESSION-LOG-2026-07-27.md`. No slice after this one may commit without its own
before/after against these numbers.

**Revert.** Delete the test dir. Zero product surface.

---

### Slice 1 — the `.constellation` watcher predicate

**Concept (the horse):** *the app's own bookkeeping folder must never look like the user's knowledge
changing.*

**Why it must be FIRST.** Boss ruling Q1 puts the ledger in a folder that sits **inside a recursive
watch**: `+layout.svelte:2696` starts a watcher for **every** library, the Universe root **is** a
registered library (`libraries.json` entry `universe_notes_18a64dc05384fa4c7be0`,
`is_universe_notes: true`, `path` = the Universe root), and `watcher.rs:104` watches
`RecursiveMode::Recursive`. `EXCLUDED_DIRS` **does contain `".constellation"`** (`file_kinds.rs:18-20`)
but is referenced **only** at `importers.rs:270`, `:546`, `:1000` and `canonical.rs:1197` — never by
`watcher.rs`, never by `reindex_changed_paths`. The only gate is `watcher.rs:77-80`.

**Measured, with a ctypes probe replicating `notify`'s exact `CreateFileW` +
`ReadDirectoryChangesW` call** (same flags, `bWatchSubtree`, dot-prefixed subdir — `notify 7.0.0`,
`Cargo.lock:2681-2683`; event path built as `request.data.dir.join(FileName)` at
`notify-7.0.0/src/windows.rs:340-343`):

| action on `<root>/.constellation/earned.jsonl` | reported | bare `.constellation`? | vanished path? |
|---|---|---|---|
| append + fsync | `MODIFIED .constellation\earned.jsonl` | **YES in one run, NO in another — NON-DETERMINISTIC** | – |
| temp + `replace` (snapshot/compaction) | `ADDED`/`MODIFIED` `earned.tmp`, `REMOVED`, `RENAMED_OLD_NAME`, `RENAMED_NEW_NAME` | **YES, twice** | **`earned.tmp`** |
| rename-aside (corrupt-store contract) | `RENAMED_OLD_NAME` / `RENAMED_NEW_NAME` | **YES** | **`earned.jsonl`** |

Against `watcher.rs:77-80`: an **existing** non-`.md` file is filtered (verified suffixes:
`earned.jsonl`→`.jsonl`, `earned.tmp`→`.tmp`, `.constellation`→`""`) — so the tail's own path is
invisible **✓**. But the **bare directory** hits `m.is_dir()` → **passes**, and **any vanished path**
hits `Err(_) => true` → **passes**. Downstream there is no `.constellation` filter at any hop:
`+layout.svelte:3374-3384` adds every passed path to `pendingReindex` **and** `pendingTreeRefresh`
with no predicate → a full `refreshLibraryTree` re-walk of the Universe root + `loadAllStats()` +
debounced `refreshLibraryCaches()` (`+layout.svelte:3328-3371`); and a vanished non-`.md` path →
`delete_rows_under_prefix` → **writer lock + collect and lowercase all 7,817 `note_meta` paths**
(`search.rs:10108-10128`) to find zero victims.

**This is live today (D3), independent of this migration:** `cece/reliability.rs:155-158` persists a
`tempfile` into `<library>/.constellation/` on every reliability save, and nothing in that folder is
suppressed (`watcher_suppress::mark` only from `write_gate.rs:249,250,598,599,725,726,763`).

**Change shape.** One predicate in the `watcher.rs:77-80` filter chain: reject any path having a
`.constellation` **path component**. Scoped to that segment **only** — **not** all dot-dirs, because
`.trash` holds 62 `note_meta` rows and is a separate design question (Slice 2 handles it at the
indexer, not the watcher).

**Safe because, measured:** **zero `.md` files exist under any of the 11 `.constellation` dirs**
across `E:\Constellation Universes\Eisa Cognitive Knowledge` and `E:\Cognitive Knowledge`
(`find`), and **zero `note_meta` rows contain `.constellation`** (live query). Nothing the indexer or
the tree owns lives there.

**Belt-and-braces (a named rule, because the predicate could be refactored away):** every temp+rename
the ledger performs marks **three** `watcher_suppress` keys — the temp path, the final path, **and
the containing `.constellation` directory path**. `was_recent` is **exact-path keyed**
(`HashMap<PathBuf>`, `watcher_suppress.rs:35`, lookup `:71`, TTL 2.5 s at `:27`), so the directory is
a separate key; marking only the two file paths leaves the bare-dir event unsuppressed.

**Tests.** `mod tests_mig104_watcher_excludes_constellation` (Rust, in `watcher.rs`) — the filter
predicate, extracted as a pure function, rejects `<root>/.constellation`,
`<root>/.constellation/earned.jsonl`, `<root>/.constellation/x.tmp` (vanished),
`<lib>/.constellation/cataloger_reliability.json`, and **accepts** `<root>/Notes/a.md`,
`<root>/.trash/b.md`, and a vanished `<root>/Folder`.

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** Constellation keeps its own bookkeeping — the search index, your settings, the
> new earned-life files — in a hidden folder called `.constellation` inside your Universe. Those are
> the app's files, not your knowledge. Until now, when the app wrote one of them, it could look to
> Constellation like *you* had changed something in your Universe — which made it re-scan your whole
> file tree for nothing. This change teaches Constellation to ignore its own folder.
>
> **Why it matters.** It is the reason the earned-life files can safely live in that folder at all
> (your ruling), and it removes an invisible stall that happens today.
>
> **Step 1 — pre-state.** Open Constellation and let it finish loading. Open any note and leave the
> sidebar file tree visible with a folder expanded.
>
> **Step 2 — action.** Use the app normally for two minutes: open three or four notes, follow a
> wikilink, switch tabs.
>
> **Step 3 — expected post-state.** The sidebar file tree **never blinks, re-collapses, or reorders
> on its own.** Nothing you did not touch changes.
>
> **Step 4 — the direct check.** Close Constellation. In File Explorer open
> `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation`, right-click any small `.json`
> file → Open with → Notepad, add a space at the end, save. Re-open Constellation. **Expected:** the
> file tree does not refresh and no folder collapses.
>
> **Failure modes.** If the tree blinks or a folder collapses by itself → the predicate is not
> catching the directory event. If notes stop appearing in the tree when you create them in
> Explorer → the predicate is too broad (it is catching real paths) — that is a stop-and-revert.

**Revert.** Remove one predicate. No data touched.

---

### Slice 2 — determinism + honesty (WA#6, and it must precede any archiving)

**Concept:** *the history stream must record what YOU changed — not what a HashMap felt like
serializing.*

**2a — kill the churn (D1).** `search.rs:6116` is
`let props_json = serde_json::to_string(&properties).unwrap_or_default();` where `properties` is the
`HashMap<String,String>` returned by `parse_frontmatter` (`search.rs:4893`). No sort → key order
varies per process → `NOTE_META_UPSERT` (`search.rs:6243-6267`) writes a byte-different
`properties_json` for identical content → the trigger's
`OLD.properties_json IS NOT NEW.properties_json` guard (`cece/history.rs:105-134`) fires. MIG-024 §0
deliberately switched DELETE+INSERT → UPSERT *so that trigger would fire* (`search.rs:6184-6193`), so
this is not a dormant path — it is the dominant one.

**Measured:** of 10,299 `properties_json` events, **2,861 (27.8%) have `old` and `new` that parse to
the identical dict** = 14.7% of all 19,481 rows; 100 notes carry ≥3 such rows; the worst pair
(`.trash\20260704T162439Z_NOTE_1B16.md` / `_351C.md`) holds **179 rows each of which 175 are no-op**,
2026-07-08 → 2026-07-26, at ≈ **one row per app boot** (cross-referenced against
`boot-perf.history.jsonl`: 07-20 5 rows/10 boots · 07-22 7/8 · 07-24 10/12 · 07-26 4/6), and the twins
share **165 identical `captured_at`** values.

**Change:** serialize from a `BTreeMap` (or a sorted `serde_json::Map`). ~2 lines. Same discipline as
`arabic/overrides.rs:392-396`.

**2b — the dot-segment guard (D4).** Add `has_dot_segment` (`libraries.rs:2693-2699`) to
`reindex_changed_paths` **Pass 1** (`search.rs:10189-10201`), which today has no such guard while the
index walk (`search.rs:6698`, `if name.starts_with('.') { continue; }`) and reconcile's walk
(`reconcile.rs:396`) both do. Live consequence: **62 `note_meta` rows at `.trash` paths**, 543 history
rows across 40 of them, two notes still accruing history **while sitting in the trash**. Without this,
"deleted" notes keep getting indexed and keep generating the very history the archive is meant to
seal. **[UNVERIFIED]** that Pass 1 is the *actual* producer of those 62 rows — it is a sufficient
mechanism (`watcher.rs:77-85` has no dot filter either, so an existing `.md` under `.trash` reaches
it), and `move_item` → `migrate_note_db_paths` (`libraries.rs:2102`) is a second candidate. **One grep
pass in Build settles it before this sub-slice commits; I will not theorize between them.**

**2c — correct two false comments (no behaviour change, but they are load-bearing lies).**
`search.rs:9869-9872` ("the declared ON DELETE CASCADE … is inert in production (FKs never
enabled)") — contradicted by `search.rs:13598-13603` in the same commit. And
`cece/history.rs:53-56` ("fields that didn't change are absent … `json_object` natively skips
NULL") — contradicted by `:95-97` in the same file and by every live row.

**Tests.** `mod tests_mig104_props_json_is_deterministic` — parse the same frontmatter twice through
`parse_frontmatter` + the serializer, assert byte-identical output; then run the real trigger on an
`init_db` temp DB: upsert the same note twice with reordered-but-identical frontmatter and assert
`SELECT COUNT(*) FROM note_state_history = 1` (today: 2).
`mod tests_mig104_pass1_skips_dot_segments` — `reindex_changed_paths` with `<lib>/.trash/x.md` present
on disk creates **no** `note_meta` row.

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** Constellation quietly keeps a history of how each note's properties changed over
> time — the raw material for the time machine you asked for. Two problems: it was recording an entry
> **every time the app started** even when nothing had changed, and it was recording history for notes
> already sitting in the trash. This fixes both, so the history is *your* edits and nothing else.
>
> **Step 1 — pre-state.** Open Constellation, open any note, and note the current time.
>
> **Step 2 — action.** Close Constellation completely. Re-open it. Close it again. Re-open it. Do
> **not** edit anything.
>
> **Step 3 — expected post-state.** After Slice 9 lands (which makes this readable), the history file
> gains **zero** new lines for those four launches. Before Slice 9, the check is the test above.
>
> **Step 4 — the trash check.** In the sidebar, right-click a note you do not need → Delete. Restart
> Constellation twice. **Expected:** the deleted note does **not** reappear anywhere in the app, and
> the sidebar count does not change.
>
> **Failure modes.** If a deleted note reappears in search results after a restart → the dot-segment
> guard is not holding.

**Revert.** Three independent one-to-two-line reverts. **No data is rewritten**, so a revert cannot
lose anything — existing no-op rows stay, they just stop multiplying.

---

### Slice 3 — `link_life.rs`: the appender, the union reader, the contract  *(Architect S1)*

**Change shape.** One new module, **~250 lines**, purely additive — it writes files nothing reads yet,
so it cannot regress anything.

```rust
pub enum Stream { Earned, NoteHistory }          // one appender, two streams (§3.4)
pub fn store_dir(conn: &Connection) -> Option<PathBuf>   // conn.path() → parent (§3.1)
pub fn append(dir: &Path, s: Stream, lines: &[String]) -> Result<(), String>  // ONE write_all per line, incl. '\n'
pub fn fsync(dir: &Path, s: Stream) -> Result<(), String>
pub fn read_folded(dir: &Path) -> (FoldedMap, LoadReport)   // snapshot + tail, Stream A only
pub fn read_history_for(dir: &Path, cid: &str) -> Vec<HistRecord>  // Stream B, ordinal-ordered by hid
pub fn ensure_gitignore(dir: &Path)               // §3.2 content, write-once, never overwrite
pub fn adopt_conflict_copies(dir: &Path) -> usize  // earned*.jsonl fold+remove; note-history*.jsonl append-dedupe
```

Cloned from `classifier/correction_log.rs:70-86` with its three gaps fixed (§3.4). `LoadReport` carries
`skipped_lines`, `corrupt_renamed_to`, `refuse_write` — the corrupt-store contract of §3.7, surfaced,
never swallowed. `read_folded` is the ONLY fold implementation; `read_history_for` deliberately has
none.

**Tests** (`tests/mig104/` for the TS side is not needed — this is Rust; `mod tests_mig104_link_life`):
`append_writes_exactly_one_line_with_lf` · `torn_tail_loses_only_the_last_line` (truncate mid-line,
assert every earlier record loads) · `fold_is_commutative_and_idempotent` (shuffle + duplicate the tail,
assert identical `FoldedMap`) · `max_fold_never_decreases_n` · `history_never_folds`
(two `nh` records for one cid both survive; ordering follows `hid`, not `at` — seeded with the real
collision shape: two records, same `captured_at`, `hid` 8251 < 8252) · `unparseable_line_is_skipped_and_counted`
· `structurally_corrupt_store_is_renamed_aside_and_refuses_fresh_write` ·
`gitignore_content_excludes_every_db_in_the_live_folder` (asserts the six patterns match
`search.db`, `search.db-wal`, `search.db-shm`, `Constellation SV Test.db`, `boot-perf.history.jsonl`,
`diagnostics.log`, `sv-trace.log` and **do NOT** match `earned.jsonl`, `note-history.jsonl`,
`settings.json`) · `keys_are_os_portable` (no drive letter, no `\`, NFC, `\n`).

**Verification clause.** `cargo test tests_mig104_link_life` green; **no** product behaviour change
(the module is unreferenced except by its tests). Diff-scoped `safety-inspection` before commit.

**Revert.** Delete the module + its `mod` line. No files exist in any Universe yet.

---

### Slice 4 — the 6 link-life write hooks  *(Architect S2)*

**Change shape.** One choke-point function `link_life::record(dir, event)` so a future seventh writer
**physically cannot** skip the store, called from:

| site | event | order |
|---|---|---|
| `search.rs:7754-7768` (`constellation_link_traverse` UPDATE) | `walk` (absolute `n`) | DB-first, **append after the guard is dropped** |
| `search.rs:8338` (`constellation_link_set_confidence`) | `trust` | **file-first + fsync**, then DB |
| `search.rs:8398` (`constellation_link_archive`) | `retire` | **file-first + fsync**, then DB |
| `search.rs:8430` (`unarchive_link_rows`) | `restore` | **file-first + fsync**, then DB |
| `search.rs:8362` + `:8370` (`constellation_link_backfill_confidence`) | `trust` ×N, one batched append | file-first + fsync |
| `review.rs:645` (`set_review_priority`) | `priority` | **file-first + fsync**, then DB |

**The lock restructure is the load-bearing part.** `constellation_link_traverse` takes
`state.db.lock()` at `search.rs:7716` and the guard currently lives to end-of-body. Move the
SELECT + UPDATE into an inner scope so the guard **drops** before the append. **Never hold
`state.db.lock()` across file I/O** — that is the PJ-066 canonical freeze shape.

**Don't record a derivable tier.** Per the Architect's Option-A graft: if the new `confidence` is
merely the auto-tier derivable from `n` (`search.rs:7747-7753`: ≥10 established, ≥3 evidence), the
walk writes **no** `trust` event. Only a *user* judgment (`contested`, or a manual pick) does.

**Joins the existing close flush** — `lib.rs:691` `CloseRequested` → the PJ-103 handshake at
`lib.rs:286-295` (emits `session:final-flush`, awaits ack, 5 s cap, instant when clean). Already
Boss-validated; the ledger fsync rides it.

**Toggle:** `EARNED_LEDGER_WRITE`. Off = today's behaviour exactly.

**Tests.** `mod tests_mig104_hooks`: `traverse_appends_one_walk_line_with_absolute_n` ·
`traverse_does_not_hold_the_db_lock_across_the_append` (the guard is dropped — asserted by taking the
lock from a second thread during a stubbed slow append) · `archive_writes_the_file_before_the_db`
(inject a failing DB mirror; assert the file line exists and the command surfaces the error) ·
`auto_tier_promotion_writes_no_trust_event` · `type_variants_of_one_pair_produce_ONE_ledger_key`
(the type-free key of Q2 — seeded with the real `Islam → the four books` shape: `supports` +
`derives-from`, one click each). `tests/mig104/ledger-format.test.ts` — reads a fixture tail and
asserts the exact field order and `\n` of §3.3.

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** Every time you follow a link between two notes, Constellation counts it. That
> count, when you last followed it, how confident you have become in the link, and your decision to
> retire a link, are things **you** created by reading — and until now they existed **only** inside
> the app's database. From this build they are also written into a plain text file you own, one line
> per action, in your own Universe folder.
>
> **Why it matters.** If the database is ever lost, corrupted, or rebuilt, that record of your
> reading survives — and you can read it yourself in Notepad without Constellation.
>
> **Step 1 — pre-state.** Open a note that links to other notes. Open File Explorer at
> `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation` and note that there is **no**
> file called `earned.jsonl` yet.
>
> **Step 2 — action.** In the note, click three different wikilinks, one at a time, coming back each
> time. Then right-click one of those links and choose **Retire** (the link stays visible in your
> text — retiring is archival, not deletion).
>
> **Step 3 — expected post-state.** `earned.jsonl` now exists. Right-click → Open with → Notepad.
> You see **four lines** — three beginning `{"v":1,"t":"walk"` and one beginning `{"v":1,"t":"retire"`.
> Each carries the note's identity code, the target's, and the time. **Nothing else in the folder
> changed, and your note file on disk was not touched.**
>
> **Step 4 — the durability check.** Close Constellation from the window's X. Re-open Notepad on the
> file. **Expected:** the four lines are still there, plus nothing new.
>
> **Failure modes.** No file at all → the toggle is off or the folder could not be created. Lines
> appear but the app feels slower when clicking links → stop; the append is on the wrong side of the
> lock (Slice 0's baseline is the arbiter). A `retire` line missing while the link *is* retired in the
> app → the file-first order is inverted, which is the one ordering this Plan calls non-negotiable.

**Revert.** Flip `EARNED_LEDGER_WRITE` off (immediate, no rebuild of data), or revert the commit and
delete `earned.jsonl`. **No note file, no schema, and no DB row is touched by this slice.**

---

### Slice 5 — the back-fill: seed the store from the DB  *(Architect S3)*

**Change shape.** `link_life_backfill.rs`, cloned from `link_boot_index.rs:56-76` (stamped in
`schema_versions`, spawned after paint, dedicated connection, failure non-fatal and logged).
Selects with the ONE earned predicate of §3.6 and appends **33 records**. Idempotent by construction
(absolute `n` + max-fold), so re-running is a no-op.

**This is the moment the currently-at-risk data becomes safe.**

**Explicitly SKIP, count, and surface — no force-stamp.** The back-fill records nothing it cannot key
and reports how many it skipped. Expected skip set today, resolved row-by-row:

| slice | count |
|---|---|
| earned rows (strict predicate) | 35 |
| — structural, excluded by design | 2 (`568654 contains tc=2`, `568645 parent tc=1`) |
| — **orphans** (source path absent from `note_meta`) | **2** (`51677` under the retired root `E:\Cognitive Knowledge\…`; `285986`, source note deleted) |
| **truly unkeyable by any identity** | **1** (`285986` — source gone AND target name matches 0 notes) |
| recordable | **33** |

**Why no force-stamp:** `ensure_cid_cn` (`canonical.rs:1256`) **writes the note file** via
`write_gate::gate_write` (`:1267`, `:1288`). The 15 cid-less notes are **templates and `.trash`
copies** — stamping a template changes what every future note spawned from it emits. **Zero live
content notes lack a `cid_cn`.** There is nothing to gain and a real side effect to cause.

**Also lands: the 236 orphan link rows are NOT recorded.** All 236 rows missing `source_cid_cn` have a
`source_path` absent from `note_meta` (235 under the retired `E:\Cognitive Knowledge` root; 1 a
deleted note). **[UNVERIFIED]** why reconcile has never purged them — filed as **PJ-168**, not fixed
here.

**Tests.** `mod tests_mig104_backfill`: `predicate_matches_35_and_excludes_structural` (seeded with
the real distribution: `n=1 ×31, n=2 ×2, n=3 ×2`, 14 `structural` rows, 234 rows with `weight<>1.0`
and `tc=0`) · `weight_off_curve_rows_are_NOT_recorded` (the 236) ·
`orphan_source_rows_are_skipped_and_counted` · `rerun_is_a_no_op` · `stamp_prevents_a_second_run`.

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** Everything you have already earned — 33 links you have walked, and the confidence
> tiers those walks produced — is currently held only in the database. This step copies it out into the
> file, once, in the background after the app has finished loading.
>
> **Step 1 — pre-state.** With Constellation closed, rename `earned.jsonl` to `earned.old.jsonl` (so
> we can watch it be recreated).
>
> **Step 2 — action.** Open Constellation. Wait until the sidebar and your notes are fully loaded, then
> wait a further ten seconds. Watch the status bar at the bottom.
>
> **Step 3 — expected post-state.** The status bar briefly shows an earned-life line and then clears.
> `earned.jsonl` exists again. Opened in Notepad it holds **33 lines**, one per link you have ever
> walked. The app never froze, and boot took the same time as before (Slice 0's number).
>
> **Step 4 — the no-double-count check.** Close and re-open Constellation. **Expected:** still 33
> lines. Not 66.
>
> **Failure modes.** More than 33 lines on the second launch → the stamp is not holding, and the fold
> is the only thing preventing a wrong count. Boot visibly slower → the back-fill is on the awaited
> path instead of after paint; that is a stop-and-revert.

**Revert.** Revert the commit and delete `earned.jsonl`; the DB is untouched (this slice only reads it).

---

### Slice 6 — the boot fold → DB restore, and the weight heal  *(Architect S4)*

**Change shape.** `link_life_restore.rs`, same clone shape as Slice 5. Loads `snapshot + tail`
(both bounded), then for each folded record whose DB row disagrees, `UPDATE note_links SET
traversal_count, last_traversed, confidence, status` — and `weight = earned_link_weight(n)`
(`search.rs:7699-7701`), **never** a stored weight. **Batched**, because every `note_links` UPDATE
fires `note_links_sky_au` (DELETE + INSERT on the 234k-row `sky_links`) plus the outgoing AI/AD/AU
pair's two `note_meta` UPDATEs (invariant #11).

**The weight heal rides here, in the same batch loop:** one background pass setting
`weight = 1.0 + ln(1 + traversal_count)` for every row where it differs (**236 rows** today; compute
in **Rust**, not SQL — `constellation_link_traverse` already avoids SQLite math functions for exactly
this reason, `search.rs:7721-7723`). This is what stops `index_note`'s live `w != 1.0` clause
(`search.rs:6560`) from treating 236 corrupt rows as earned forever.

**Restores review priority too** (`note_meta.review_priority`; 0 set today).

**Tests.** `mod tests_mig104_restore`: `db_loss_round_trip` — build a temp DB with the 33-record shape,
export via Slice 5, **drop `note_links` entirely**, re-index from note text (so every row comes back
`hypothesis / 1.0 / 0`), run restore, assert **every one of the 33** matches the original
`(n, at, confidence, status)` and `weight == earned_link_weight(n)` · `restore_is_idempotent` ·
`weight_heal_fixes_the_236_shape_and_touches_nothing_else` (seeded with `0.526`/`0.564`/`0.952`/`1.081`
rows, asserts the 233,964 `weight=1.0` rows are **not** written) · `restore_is_batched`
(assert ≤ N statements per transaction) · `failure_is_non_fatal_and_logged`.

**Verification clause (Boss-testable — THE HEADLINE TUTORIAL).**

> **What this is.** This is the whole point of MIG-104. Constellation's search index is a big database
> file that can, in principle, be lost — a bad disk, a bad backup, a corrupted file. Everything
> *about* your notes comes back from the notes themselves. But the record of **your reading** —
> which links you walked, how often, what you came to trust, what you retired — has until now lived
> **only** in that database. After this step it comes back too.
>
> **Step 1 — pre-state.** In Constellation, open a note where you have followed links. Note the small
> `×N` numbers on the link chips (for example `×3`). Write down two or three of them, and the name of
> any link you have retired.
>
> **Step 2 — action.** **Close Constellation completely.** In File Explorer, go to
> `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation`. Rename `search.db` to
> `search.db.MOVED` (do **not** delete it — we are keeping it as your safety net). Leave
> `earned.jsonl` exactly where it is. Re-open Constellation.
>
> **Step 3 — expected post-state.** Constellation rebuilds its index from your notes, as it already
> knows how to do — the sidebar fills in, search works. Then the status bar reports the earned layer
> being restored. Re-open the note from Step 1. **The `×N` numbers are the same ones you wrote down.
> The link you retired is still retired.** Nothing you created is lost.
>
> **Step 4 — the safety net.** Close Constellation, delete the freshly built `search.db`, rename
> `search.db.MOVED` back to `search.db`, re-open. You are exactly where you started.
>
> **Failure modes.** `×N` numbers all show as nothing / links appear never-walked → the fold read the
> file but the restore did not write, or the file was empty (check `earned.jsonl` in Notepad first —
> if the lines are there, the bug is in the restore, not the ledger). A retired link comes back active
> → the `status` field is not being restored. The app freezes during restore → the batch size is
> wrong; the sky trigger is being fired 234,000 times.

**Revert.** Revert the commit; the ledger file stays (harmless). The weight heal is **not** reverted by
that — it corrected 236 arithmetically-impossible values, and re-introducing them has no user value.
Documented in the commit message.

---

### Slice 7 — the snapshot + compactor  *(Architect S5)*

**Change shape.** `earned.snapshot.jsonl` = one line per earned link, current folded state.
Compaction triggers on a **byte threshold (2 MB)**, never on a timer (invariant #18: an idle Universe
produces zero diffs): write the new snapshot to a **unique temp** (`tempfile::Builder…tempfile_in`,
the `cece/reliability.rs:155-158` shape — never `universe::atomic_write`'s fixed `<name>.tmp`,
PJ-087), persist, then **rename the tail aside** (`earned.tail-<UTC>.jsonl`) — **never delete**
(invariant #4). Load = snapshot + tail, both bounded. This resolves Option D's only serious defect
(the unbounded fold) and its internal inconsistency: the resident map is rebuilt from a **bounded
snapshot** every launch, not from a growing log.

**`note-history.jsonl` is NEVER compacted** (§3.4). If its size ever becomes a concern the only legal
operation is **time segmentation** (`note-history-2026.jsonl`), and that is not in this Plan.

**Watcher discipline applies here specifically** — this is the slice that performs a temp+rename inside
`.constellation/`. Slice 1's predicate is the structural guarantee; the three `watcher_suppress::mark`
keys (temp, final, **containing directory**) are the fallback.

**Tests.** `mod tests_mig104_compact`: `compaction_is_lossless` (append 5,000 synthetic events, compact,
assert the folded map is byte-identical before/after) · `compaction_renames_the_tail_never_deletes` ·
`temp_name_is_unique_under_concurrency` (two compactions in flight do not collide — the PJ-087
regression) · `note_history_is_never_compacted` (the compactor refuses the Stream-B path) ·
`threshold_is_bytes_not_time` (an idle store produces zero writes across 100 load cycles).

**Verification clause.** Not Boss-testable (no user-visible change). Verified by the tests plus a
before/after byte count on the live store, recorded against Slice 0.

**Revert.** Revert the commit; on next load the union reader ignores a `snapshot` file it does not
know about only if the reverted code cannot read it — so the revert path is: revert, then rename
`earned.snapshot.jsonl` and every `earned.tail-*.jsonl` back into a single `earned.jsonl` (documented
as a one-line `cat` in the commit message). **Tested by `compaction_revert_recipe` in the same module.**

---

### Slice 8 — ARCHIVE BEFORE PURGE in `reindex_delete_note`  *(NEW — the Boss's ruling)*

**Concept:** *a deleted note's thinking is archived first, so it can be brought back — the delete
removes the note, not its history.*

**The territory, verified.** `reindex_delete_note` (`search.rs:9808-9917`) is the **single funnel**
every delete reaches, from **6 call sites**: `libraries.rs:6577` (`move_to_trash`), `libraries.rs:6659`
(single file) and `:6662` (per folder descendant), `reconcile.rs:260` (gone from disk at boot),
`search.rs:10155` (`delete_rows_under_prefix` — the folder-vanish bulk path, so it needs **no separate
hook**), `search.rs:10217` (vanished `.md` from the watcher). Today the function holds
`state.db.lock()` for its whole body (`search.rs:9809`), runs **no transaction** (7+ auto-commit
statements at `:9844-9878`), makes every delete `let _ =` best-effort, and returns `Ok(())`
**unconditionally**.

**The insertion point is `BEFORE search.rs:9845`, not before the Stage-0 purges** — see §2.2. The
existing pre-capture block at `search.rs:9815-9843` (which already reads `body_text` and `tags_json`
before the deletes, for exactly this reason) is where the new reads go.

**Change shape.**

1. **Extend the pre-capture block** with, under the already-held guard:
   `SELECT cid_cn, library_name FROM note_meta WHERE path=?1` ·
   `SELECT history_id, captured_at, changes_json FROM note_state_history WHERE note_path=?1 ORDER BY captured_at, history_id`
   (rides `idx_note_state_history_note_time`, `cece/history.rs:61-76`) ·
   `SELECT from_shape, to_shape, changed_at, changed_by, undone FROM shape_history WHERE path=?1`
   (lazily-created table → treat `"no such table"` as a correct no-op, mirroring `run_lazy` at
   `libraries.rs:1196-1210`).
   **The cid must be resolved HERE** — `note_state_history` has no cid column (keyed on an absolute
   Windows path) and after `:9845` the mapping is gone.
2. **Change the signature** to
   `reindex_delete_note(state, note_path, ctx: DeleteCtx)` where
   `DeleteCtx { reason: DeleteReason, library_root: Option<String>, dest: Option<String> }` and
   `DeleteReason ∈ Trash | SystemTrash | Permanent | Vanished | ReconcileGone`.
   **No defaultable parameter** — the point is that a future 7th caller *cannot* skip it. `dest` is
   knowable only at the two de-collide sites (`libraries.rs:6541-6560` and `:6681-6696`), so the trash
   callers pass it down. `library_root` is available at **all six** sites (verified: the four in
   `libraries.rs`/`reconcile.rs` hold `app`; `delete_rows_under_prefix` receives it from
   `reindex_changed_paths(app, …)`, `search.rs:10167`).
3. **Lock discipline:** read + serialize under the guard → **drop the guard** → append + fsync →
   re-acquire → purge. Never hold `state.db.lock()` across file I/O. Invariant 9 is not in play (it
   forbids file I/O inside `index_note`'s `BEGIN IMMEDIATE`, not here), and both trash entry points are
   already `#[tauri::command(async)]` (`libraries.rs:6519`, `:6600`).
4. **Archive-first means REFUSE on failure.** If the archive write fails: log, return a real `Err`, and
   **do not purge**. The file has already left its old location, so leaving the rows is a one-note
   index↔disk divergence that reconcile's removal loop (`reconcile.rs:249-266`) retries next boot —
   strictly better than destroying history. This makes `reindex_delete_note` able to fail for the first
   time; the four `let _ =` / `.is_ok()` call sites (`libraries.rs:6659`, `:6662`, `search.rs:10155`,
   `:10217`) must surface it. **This is the LL-035 false-success shape and PJ-165's exact sweep target.**
5. **Wrap the purge (`search.rs:9844-9878`) in ONE transaction.** It is 7+ auto-commit statements today;
   a crash mid-purge leaves a half-deleted note, and with an archive it would leave archive/DB
   disagreement.
6. **Batch API.** `DeleteArchive::open(dir)` + `append(records)` + one `sync_all()` on close, used by the
   three loops (`libraries.rs:6661-6663`, `search.rs:10148-10158`, `reconcile.rs:249-266`). A 500-note
   folder delete must be **one** fsync, not 500 — sized from the measurement: ~2.5 rows / **~1,940 bytes
   per note** (19,481 rows / 7,785 notes / 15,103,149 bytes), so 500 notes ≈ 1 MB in one pass.

**What is archived, and what is explicitly NOT — judged by "recomputable from the `.md`?"**

| table | rows live | verdict | why |
|---|---|---|---|
| `note_state_history` | **19,481** / 7,785 notes / 15.1 MB | **ARCHIVE** | Trigger-captured old→new of past frontmatter. The `.md` holds only the **current** values — every past value exists nowhere else (`cece/history.rs:1-10` states this doctrine). **Zero frontend readers today** (`cece_get_note_history` / `cece_query_history` at `cece/history.rs:317`/`:360`, registered `lib.rs:433-434`, **no `invoke` anywhere in `src/`**; orientation v3.70:3671-3674 records the surface as *"CONTRADICTED — Sight v3 is retired … Deferred indefinitely"*). So nothing else is protecting these rows. |
| `shape_history` | **1** | **ARCHIVE** | An undo/redo **decision stack**: `record_change` truncates the redo branch (`DELETE … WHERE undone = 1`, `shape.rs:236-241`). The current shape IS in frontmatter (`SHAPE_KEY`, `shape.rs:35`), so the **value** is recomputable — the **trail** is not. Cost ≈ 0. |
| the note's **outgoing earned link state** | 33 recordable | **already covered** | `DELETE FROM note_links WHERE source_path` at `search.rs:9844` destroys it, but Slices 3–7 already hold it keyed on cid pairs. **This slice adds nothing for links.** Stated so nobody re-invents it. |
| `note_summaries` | 7,767 | **do not archive** | A `content_hash`-keyed cache, recomputed on miss/stale from `body_text` + `properties_json` (`nsc/mod.rs:641-660`) with a resumable backfill (`nsc/backfill.rs:1-20`). |
| `sight_v3_layout` | 7,635 | **do not archive** | A graph embedding keyed `(library_set_hash, graph_version)`, rebuilt wholesale per version with a cursor (`sight_layout.rs:334-370`) and an explicit invalidate (`:161-179`). |
| `sources_suggestions` | 0 | **do not archive** | Classifier output (`classifier/mod.rs:395`, `sources/mod.rs:408`). |
| `note_aliases` **'rename' rows** | — | **Boss ruling needed** | `search.rs:9852` deletes **ALL** alias rows for the path *"including any 'rename' aliases"* — the record of the note's **former names**, which is not recomputable. Cheap to add to the same stream. **Checklist item 5.** |
| `review_schedule` | 7,817 | **needs nothing** | `review-pulse.json` already survives the delete — `review.rs` removes a pulse entry only at `:685` inside `mark_reviewed`; nothing removes a deleted note's entry. |
| `note_body` | 7,823 / **272,146,004 bytes** (avg 34.8 KB) | **Boss ruling needed** | Purged at `search.rs:9864`. The **file** survives in `.trash` forever for a `'trash'` delete (live settings: `trashDestination=local`, `trashFolderScope=universe`), only until the bin is emptied for `'system'`, and not at all for `'permanent'`. **Checklist item 6.** |

**Tests.** `mod tests_mig104_archive_at_delete`:
`archive_hook_runs_BEFORE_the_note_meta_delete` — **the regression test for §2.2**: seed 5 history rows,
run the delete with FKs on, assert the archive holds **5** records (today a hook at the purge site would
capture 0) · `explicit_purge_is_a_zero_row_noop_in_production` (pins the FK reality alongside
`tests_pj150_fk_enforcement_reality`) · `failed_archive_refuses_the_purge_and_returns_Err` ·
`folder_delete_of_500_notes_is_one_fsync_and_500_streams_with_correct_n` ·
`nd_marker_records_the_de_collided_trash_destination` ·
`cid_less_note_archives_with_the_rel_fallback` (the 15-note shape) ·
`hid_ordinal_survives_a_captured_at_collision` (the 765-group shape) ·
`purge_is_one_transaction` (kill mid-purge; assert all-or-nothing).

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** When you delete a note, Constellation used to throw away the quiet record of how
> that note's thinking had changed over its life — the properties you set, the sources you attached,
> the stages it passed through. Your ruling: **archive it first.** From this build, deleting a note
> writes its whole change history into a plain text file in your Universe before removing anything.
> That file is the foundation of the time machine you described.
>
> **Why it matters.** It means a delete is never final in the way that matters. The note leaves; the
> record of your thinking about it does not.
>
> **Step 1 — pre-state.** Create a note called `Archive Test`. Give it a few properties (a source, a
> stage), save, then change them, save again, then change one more time. You have now made three
> changes. Open Notepad on
> `E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\note-history.jsonl` — it does not
> exist yet, or does not mention this note.
>
> **Step 2 — action.** In the sidebar, right-click `Archive Test` → **Delete**.
>
> **Step 3 — expected post-state.** The note is gone from the sidebar, and the file itself is sitting
> in `.trash` as it always was. Now open `note-history.jsonl` in Notepad. **You can read your own edit
> history in plain text** — several lines beginning `{"v":1,"t":"nh"` (one per change you made,
> each showing the old value and the new value), followed by **one** line beginning `{"v":1,"t":"nd"`
> that records where the note came from, where it went in the trash, and how many history lines belong
> to it. This is File-Over-App made literal: no Constellation required to read it.
>
> **Step 4 — the folder check.** Create a folder with three notes in it, edit each once, then delete the
> folder. **Expected:** the file gains all three notes' histories and three `"nd"` lines, and the delete
> feels instant.
>
> **Failure modes.** The file exists but has **no** `"nh"` lines for the note, only the `"nd"` line →
> **this is the §2.2 bug**: the hook is on the wrong side of the `note_meta` delete and the history was
> already gone. If the delete reports success but the file was not written → the refuse-on-failure rule
> is not wired, which is the app-killer class this Plan exists to prevent.

**Revert.** Flip `NOTE_HISTORY_ARCHIVE` off, or revert the commit. **The archive file is never deleted
by a revert** (invariant #4) — it stays readable.

---

### Slice 9 — the continuous note-history mirror  *(NEW, GATED — recommended; Boss checklist item 4)*

**The honest fork.** Slice 8 protects notes deleted **after** it ships. It does **not** make
`note_state_history` survive the loss of `search.db` — **19,481 rows / 15.1 MB across 7,785 live
notes** would still be lost, and `MAX(history_id)=20,174` vs `COUNT=19,481` shows **693 rows have
already been destroyed** in this database's life. If the time machine is meant to cover *live* notes'
past, this slice is required. **I recommend building it**, gated.

**Change shape, and why it does not violate invariant 9.** The producer is a **SQLite trigger**
(`cece/history.rs:105-134`), not Rust code, so Rust must read back what the trigger wrote:

1. Before the `NOTE_META_UPSERT` in `index_note` (`search.rs:6263-6267`), capture
   `SELECT MAX(history_id) FROM note_state_history` — **O(1) on the PK**.
2. After `COMMIT`, `SELECT … WHERE history_id > <that>` — an **indexed range scan**, normally 0–1 rows.
3. Push the rows into the **same in-memory buffer** the ledger flusher already drains.
   **No file I/O under the writer lock. None on the keystroke path.**

Reading back rather than recomputing the diff in Rust keeps the SQLite trigger as the **single
definition of "what an event is"** — no duplicate logic can drift.

**Cost, measured.** Monthly trigger volume today: 2026-05 = 8,458 rows / 9.13 MB · 2026-06 = 536 / 0.33 MB
· 2026-07 = 2,876 / 1.12 MB. **After Slice 2 removes the 27.8% churn**, ≈ **2 MB/month**. A one-off
back-fill of the existing 19,481 rows (≈15 MB) runs in the background after paint, resumable, stamped —
the Rule-8 first-time-population shape (`nsc/backfill.rs:1-20` precedent).

**Restore semantics.** A restore pass inserts with **fresh** `history_id`s (AUTOINCREMENT + the live
sequence) and treats the archived `hid` purely as the **ordinal** (§3.3).

**Toggle:** `NOTE_HISTORY_MIRROR`. **This is the only slice in the note-history family that adds cost to
the save path** — its verification is the Slice-0 save-latency number on the 535-link note, before and
after.

**Tests.** `mod tests_mig104_history_mirror`: `one_property_edit_appends_exactly_one_nh_line` ·
`no_edit_appends_nothing` (the D1 regression, from the other side) ·
`max_history_id_capture_is_a_pk_lookup_not_a_scan` · `mirror_never_writes_a_file_inside_the_transaction`
(the invariant-9 pin: assert zero fs syscalls between BEGIN and COMMIT via an injected fs shim) ·
`backfill_is_resumable_from_a_cursor` · `restore_assigns_fresh_history_ids_and_preserves_hid_as_ordinal`.

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** Slice 8 saves a note's history when you **delete** it. This step keeps a running
> copy for **every** note, all the time — so if the database itself is ever lost, the history of how
> your thinking changed is still on disk, in a file you can read.
>
> **Step 1 — pre-state.** Note the size of `note-history.jsonl` in File Explorer. Open a note.
>
> **Step 2 — action.** Change one property (for example, set its stage), and save. Wait three seconds.
>
> **Step 3 — expected post-state.** `note-history.jsonl` has grown by **one** line, and that line shows
> the old value and the new value of the property you just changed. **Typing in the note feels exactly
> as instant as before** — that is the whole constraint on this step.
>
> **Step 4 — the no-noise check.** Close Constellation and re-open it twice **without editing
> anything**. **Expected:** the file gains **zero** lines. (Before Slice 2, this test would have gained
> a line per launch.)
>
> **Failure modes.** Typing or saving feels even slightly slower → stop; measured against Slice 0 this
> is a revert, not a tune. Lines appear on launch with no edit → Slice 2's determinism fix regressed.

**Revert.** Flip `NOTE_HISTORY_MIRROR` off. The file stays; the DB is authoritative and unaffected.

---

### Slice 10 — archive the destination pre-delete in the shared cascade  *(NEW)*

**Change shape.** `migrate_note_db_paths` (`libraries.rs:1094-1230`) is the ONE shared cascade
(MIG-105 Stage 0) and it destroys history in two places: `DELETE FROM note_meta WHERE path = <new_path>`
(`libraries.rs:1157`, which **cascades** the phantom's history under live FKs) and
`DELETE FROM note_state_history WHERE note_path = <new_path>` (`libraries.rs:1190`). The comment at
`:1180-1186` states the intent — clear a **dead** note's trail at the destination *"so two notes'
timelines are never silently merged under one path."* That intent is correct **and** it is a
history-destroying event: under archive-first it must archive, then drop.

Archive **before `libraries.rs:1157`**, `reason: DisplacedByMove`, using the same `DeleteArchive`
handle and the same `conn.path()` location — the file already uses `conn.path()` two lines later at
`:1200-1202` for its diagnostics, so the mechanism is already present in this exact function.

**Its 3 production callers** are unchanged: folder-rename descendants (`libraries.rs:1270`), the rename
tail (`:1403`), and move (`:2102`), plus `reconcile::relocate_row` delegating at `reconcile.rs:335`.

**Why this cannot be skipped.** An archive slice that hooks only `reindex_delete_note` leaves this hole
open on **every** rename that lands on a previously-occupied path — the Whole-Ecosystem Fix Law's exact
failure shape (fix the sidebar walker, leave the Move picker).

**Tests.** `mod tests_mig104_displaced_by_move`:
`destination_phantom_history_is_archived_before_the_pre_delete` (seed a dead row + 3 history rows at the
destination, rename onto it, assert 3 `nh` records + one `nd` with `mode:"displaced-by-move"`) ·
`a_move_onto_a_free_path_archives_nothing` (no false positives) ·
`the_moving_note_own_history_is_MIGRATED_not_archived` (the `:1191` UPDATE still carries it — a rename
must not archive-and-lose the live note's trail).

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** Sometimes you rename or move a note onto a name where a *different, already-deleted*
> note used to live. Constellation clears that dead note's leftover history so the two notes' timelines
> never get mixed up — correct behaviour, but it was throwing that history away. Now it archives it
> first, exactly like a delete.
>
> **Step 1 — pre-state.** Create a note `Ghost`, edit its properties twice, then delete it. (Its history
> is now archived by Slice 8 — good.) Create a second note `Living`, edit its properties once.
>
> **Step 2 — action.** Right-click `Living` → Rename → type `Ghost`. Confirm.
>
> **Step 3 — expected post-state.** `Living` is now called `Ghost`, and **its own** single history entry
> is intact (visible after Slice 11's check, or by the test). `note-history.jsonl` shows no data loss for
> either note.
>
> **Failure modes.** If renaming onto a previously-used name fails outright → the archive is refusing and
> blocking the rename; that is a stop (the refuse rule belongs on **delete**, and a rename must not be
> blocked by an archive failure — it should log and proceed, since the rename itself loses nothing the
> ledger needs).

**Revert.** Revert the commit; behaviour returns to Stage-0's (destroy the phantom trail).

---

### Slice 11 — the restore rejoin  *(NEW)*

**Concept:** *bring the file back and its history comes with it.*

**Why a NEW hook is needed.** There is **no restore path in the app today** — an exhaustive grep for
`untrash|restore_from_trash|restore_note|restoreNote|recover_deleted|undelete` across `src-tauri/src`
and `src` returns **zero hits**, there is no Trash/History panel component, and there is **no
empty-trash command anywhere** (`.trash` grows forever — 57 items live). Restore is therefore a **manual
OS action** today: drag the file out of `.trash`, or restore from the Recycle Bin. And the existing cid
self-heal cannot cover it: `index_note`'s cid-collision heal (`search.rs:6280-6372`) fires only when a
**dead `note_meta` row still owns the cid** — after a delete that row was purged, so no UNIQUE conflict
occurs and `migrate_note_db_paths` is never called.

**Nothing new is written onto the file.** The `.trash` copies already carry their own `cid_cn` in
frontmatter (58 of the 62 live `.trash` `note_meta` rows have one) and `index_note` re-derives it on
adoption (`search.rs:6090-6097`). **The file is self-identifying.** Everything missing is on the archive
side, and the ledger already supplies it.

**Change shape.**

1. A resident `archived_cids: HashSet<String>` loaded once at boot from the archive's `nd` markers —
   bounded by **deleted-note count** (0 today ⇒ zero boot payload).
2. In `index_note`, immediately after the upsert succeeds (`search.rs:6374`): if the cid is non-empty and
   in the set, push `(cid, note_path)` onto a queue. **O(1), memory-only — no file I/O inside
   `BEGIN IMMEDIATE`** (invariant 9).
3. Drain on the ledger's flusher thread: read that cid's `nh` records, take the DB lock, INSERT into
   `note_state_history (note_path, captured_at, changes_json)` (and `shape_history`) for records not
   already present — **fresh `history_id`s**, `captured_at` and `ev` verbatim — then append
   `{"t":"nr"}` so the replay is idempotent and can never double. One transaction. The trigger is on
   `note_meta` UPDATE, so there is no trigger storm.
4. **Never delete from the archive** — append `nr`, keep the trail (invariant #4 + "archival, not
   deletion").

**One funnel covers every re-appearance route** — manual drag out of `.trash`, Recycle-Bin restore, a
`git checkout`, a Syncthing re-add, and reconcile's re-adopt — because they all end at `index_note`.
This is the same argument PJ-154 used for the cid self-heal.

**Explicitly NOT built: a `.trash` manifest.** That would be the second bespoke format the
Whole-Ecosystem Fix Law forbids, and it would live in the folder the user empties.

**Tests.** `mod tests_mig104_rejoin`: `restored_note_reclaims_its_archived_history` ·
`replay_is_idempotent_across_three_boots` (the `nr` guard) ·
`replay_assigns_fresh_history_ids` · `a_never_deleted_note_triggers_no_lookup`
(the O(1) claim: assert the queue stays empty over 1,000 indexes) ·
`rejoin_does_no_file_io_inside_the_transaction`.

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** The first half of the time machine. When you bring a deleted note back — by dragging
> it out of the trash folder, or restoring it from the Windows Recycle Bin — Constellation now recognises
> it and gives it its history back.
>
> **Step 1 — pre-state.** Use the note you deleted in Slice 8's test (`Archive Test`). It is sitting in
> `E:\Constellation Universes\Eisa Cognitive Knowledge\.trash`.
>
> **Step 2 — action.** With Constellation **open**, drag that file from `.trash` back into your Universe
> root (or any library folder). Wait a few seconds.
>
> **Step 3 — expected post-state.** The note reappears in the sidebar. `note-history.jsonl` gains **one**
> new line beginning `{"v":1,"t":"nr"` — the record that its history has been re-attached. The three
> changes you made before deleting it are back in the database.
>
> **Step 4 — the no-double check.** Close and re-open Constellation twice. **Expected:** no further
> `"nr"` lines, and the history has not been duplicated.
>
> **Failure modes.** No `"nr"` line and the note behaves as brand new → the resident set or the queue is
> not wired. Two `"nr"` lines, or duplicated history → the idempotence guard is not holding, which is
> the one thing this slice must never get wrong.

**Revert.** Revert the commit. Archived history simply stays archived (the pre-slice behaviour); nothing
is lost.

---

### Slice 12 — PJ-164 / C8: the child tables to `ON UPDATE CASCADE`  *(NEW — unblocked by R2 = archive-first)*

**Why it is here.** `tests_pj150_fk_enforcement_reality` (`search.rs:13576-13631`) states the case in its
own doc comment: the child FKs are **`ON UPDATE NO ACTION`**, so *any* note owning a
`note_summaries` / `note_state_history` / `sources_suggestions` row **cannot** have its
`note_meta.path` updated — *"that is the true root cause of the 1,591 'relocate deferred' lines
(PJ-151) … therefore the C8 child-table rebuild (ON UPDATE CASCADE) is not an optional hardening — it is
the fix."* Stage 0 made moves **work** via `PRAGMA defer_foreign_keys` inside a transaction
(`libraries.rs:1124-1133`); the **declaration is still wrong**, and every future path-writer inherits
the trap. PJ-164 was gated on Boss ruling **R2** — *does a deleted note's history die with the note or
get archived first?* **R2 is answered: archive-first.** So the rebuild can proceed knowing the archive
is the safety net.

**Ordering is deliberate: this lands AFTER Slices 8–11.** A table rebuild is the one operation in this
Plan that recreates 19,481 rows; it must not run before the archive exists to hold them.

**Change shape.** A stamped one-shot migration, inside one transaction, per child table:
`CREATE TABLE <t>_new (… FOREIGN KEY (<col>) REFERENCES note_meta(path) ON DELETE CASCADE ON UPDATE CASCADE)`
→ `INSERT INTO <t>_new SELECT … FROM <t>` → `DROP TABLE <t>` → `ALTER TABLE <t>_new RENAME TO <t>` →
recreate the indexes (`idx_note_state_history_note_time`, `cece/history.rs:61-76`) **and the trigger**
(`note_state_history_au`, `cece/history.rs:105-134` — a `DROP TABLE` does not carry it). Plus:

* **A shared connection-pragma helper** so every connection is opened identically and the FK reality is
  stated in **one** place instead of being a rusqlite default nobody knew about (this is what made
  PJ-150's diagnosis wrong).
* **A `foreign_key_check` gate that quarantines ROWS, never the DB file** — run it before and after; any
  violating row is moved to a `<t>_quarantine` table with its reason, logged, and surfaced. **Never**
  delete, never rename the DB aside for this (`search.rs:8883-8911`'s rename-aside is for a schema
  version mismatch, not for row-level violations).
* **`PRAGMA legacy_alter_table` must be OFF** and the rebuild must not run inside an outer
  `defer_foreign_keys` scope — **[UNVERIFIED]** interaction; Build must confirm on a copy of the live DB
  before it runs on the real one.

**Tests.** `mod tests_pj164_child_tables_cascade_on_update`:
`path_update_no_longer_needs_defer_foreign_keys` (the direct inverse of
`tests_pj150_fk_enforcement_reality`'s `blocked.is_err()` assertion — that test must be **amended, not
deleted**, with the new expectation and a comment naming this slice) ·
`rebuild_preserves_every_row_and_the_trigger_still_fires` (seed 100 history rows + the trigger, rebuild,
assert 100 rows and that a subsequent `note_meta` UPDATE emits row 101) ·
`indexes_are_recreated` (assert `idx_note_state_history_note_time` exists post-rebuild) ·
`foreign_key_check_violations_are_quarantined_not_dropped` · `migration_is_stamped_and_runs_once` ·
`defer_foreign_keys_path_still_works` (Stage 0's mechanism must not regress — belt and braces for one
release).

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** A plumbing correction. Constellation stores several kinds of side information about
> each note (its change history, its summary, its source suggestions) linked to the note's file path.
> Those links were declared in a way that **blocked renaming or moving** any note that had them — the
> app worked around it, but the declaration was wrong, and every future feature inherited the trap.
> This fixes the declaration itself. Because your ruling is "archive first," the note's history is
> already safe on disk while this runs.
>
> **Step 1 — pre-state.** Pick a note you have edited several times (so it has history). Note its name
> and its folder. **Before you start: copy `search.db` somewhere safe** — this step rebuilds internal
> tables, and a copy is a free insurance policy.
>
> **Step 2 — action.** Open Constellation and let it finish loading (the rebuild runs once, in the
> background, after the app is usable). Then rename that note. Then move it to a different folder. Then
> rename the folder it is in.
>
> **Step 3 — expected post-state.** All three operations succeed with no error. The note keeps its
> properties, its links, its review status. Search still finds it under the new name. Nothing in the
> sidebar is duplicated or missing. The app's overall speed is unchanged.
>
> **Step 4 — the count check.** Restart Constellation. **Expected:** the note count in the sidebar is
> exactly what it was before Step 2.
>
> **Failure modes.** Any note disappearing, or the note count dropping → stop immediately, close the
> app, and restore your `search.db` copy. A rename failing with an error → the rebuild left the old
> declaration in place; revert. Boot noticeably slower on the launch after the rebuild → the trigger
> or an index was not recreated.

**Revert.** The migration is stamped, so a code revert leaves the **rebuilt** schema in place — which is
strictly better than the old one (it is a superset of the old behaviour: `defer_foreign_keys` still
works, tested). If the rebuild itself fails it does so inside one transaction and rolls back whole. The
Boss's `search.db` copy is the outer net.

---

### Slice 13 — the GATED `index_note` overlay, RE-SCOPED  *(Architect S6)*

**Re-scoped, and this is a correction to the Architect.** Its stated headline defect —
*"archive is silently reversed by the next save"* — is **already fixed in the tree**:
`search.rs:6560-6566` includes `status != "active"` in the preserved predicate and carries `status` as
the 6th tuple element; `search.rs:6645-6653` binds `?14 = status` with the comment *"`status` is
RESTORED, not hardcoded to 'active'."* Live data agrees (`status = 'active'` on all 234,233 rows — nothing
is currently damaged). **Slice 6's headline Boss test would pass on today's build and prove nothing —
it is dropped.**

**What genuinely remains** are the three **cross-path** wipers, where the DB-to-DB `preserved` map is
legitimately **empty**:

| wiper | mechanism |
|---|---|
| **target rename** | the preservation key is `"{link_type}::{target_name}"` (`search.rs:6645`, built at `:6560-6566`); the rename cascade rewrites `[[Foo]]` → `[[Foo v2]]` in every inbound note (`libraries.rs:5675`), so the key no longer matches and every inbound link resets to `hypothesis / 1.0 / 0`. |
| **source note move** | `reindex_delete_note` deletes every outgoing row (`search.rs:9844`), then re-index at the new path finds an empty map. |
| **external folder rename** (Explorer / Syncthing) | the code's own comment concedes it. |

**Change shape (~20 lines).** At the `preserved.get(&pkey)` miss branch (`search.rs:6645`), consult the
**resident folded map** (Stream A, rebuilt from the bounded snapshot at launch — Slice 7 is what makes
this possible) keyed **type-free on `(source_cid, target_cid)`** per Q2, and re-insert with the recorded
`n / at / confidence / status` and `weight = earned_link_weight(n)`.

**Non-negotiable requirements (from the Architect, kept verbatim in spirit):**
* Behind `EARNED_LEDGER_READBACK`, with the old path intact for one build.
* **In-memory snapshot lookup only** — no file I/O, no lock contention with the flusher (invariant 9:
  this runs inside `BEGIN IMMEDIATE` on the 1500 ms save debounce, on notes with up to **535** links).
* A Reproduce-First harness proving each remaining wiper **red → green**: (i) rename a link's target,
  (ii) re-type a link (Q2: history is **kept**), (iii) move the source note, (iv) an external folder
  rename.
* **The full 8-point Editor-Surface Gate Checklist, Focus mode included.**
* A diff-scoped `safety-inspection` before the commit.
* Before/after measurement on the 535-link note and on boot, against Slice 0.

**Also lands here — D2, the display double-count (WA#6).** `dedupeBySource` **sums**:
`existing.traversalCount = (existing.traversalCount ?? 0) + (row.traversalCount ?? 0)`
(`store.ts:4203`). With the type-blind writer (`search.rs:7724-7727`), a pair with two type-variants
shows **double** the real clicks — live on exactly 2 pairs today (`Islam → the four books`:
`supports` + `derives-from`, one click each, displayed as ×2; `Link type test v2 → banana`:
`contradicts` + `supersedes`). Change the merge to **max**. It belongs in this slice because the
type-free ledger stores the correct value `1` for those pairs but the restore writes it back to **both**
rows — so without this fix the display double-count persists after a restore.

**Also lands here — the ONE earned predicate as a named function** (§3.6), replacing the inline
`(tc > 0 || w != 1.0 || status != "active")` at `search.rs:6560`. This is the change that stops
`index_note` treating the **236 off-curve weight rows** as earned. Safe to do here because Slice 6's
weight heal has already normalised them.

**Tests.** `mod tests_mig104_overlay`: `target_rename_preserves_earned_state` ·
`retype_preserves_earned_state_per_Q2` · `source_move_preserves_earned_state` ·
`external_folder_rename_preserves_earned_state` · `overlay_does_no_file_io_inside_BEGIN_IMMEDIATE` ·
`535_link_note_save_latency_within_baseline` · `earned_predicate_excludes_off_curve_weight_and_structural`.
`tests/mig104/dedupe-max-not-sum.test.ts` — the two live pair shapes, asserting `×1` not `×2`.

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** Until now, three ordinary actions quietly erased what a link had earned: renaming
> the note a link points **to**, moving the note the link lives **in**, and renaming a folder from
> outside Constellation. In each case the app could not match the link to its old record, so the count
> went back to zero. Now it matches on the notes' permanent identity codes instead of their names, so
> renaming and moving no longer costs you anything.
>
> **Step 1 — pre-state.** Pick a note `A` that links to a note `B`. Click that link three times, coming
> back each time, so the chip reads `×3`. Write that down.
>
> **Step 2 — action (target rename).** Right-click `B` in the sidebar → Rename → `B v2`. Confirm. Open
> `A` again.
>
> **Step 3 — expected post-state.** `A`'s text now reads `[[B v2]]` (the rename cascade, as always) and
> the chip **still reads `×3`**. Today it would read nothing.
>
> **Step 4 — action (source move).** Drag `A` into a different folder. Open it. **Expected:** still `×3`.
>
> **Step 5 — action (re-type).** Change the link's type from one kind to another (for example `supports`
> → `contradicts`). **Expected:** still `×3` — per your ruling, re-typing a link **keeps** its history.
>
> **Step 6 — action (the double-count fix).** Find a link where you have used **two different types** to
> the same target. **Expected:** the chip shows the real number of clicks, not twice it.
>
> **Step 7 — the speed gate.** Open your largest note (the one with hundreds of links), type ten
> characters quickly, and save. **Expected: no perceptible lag whatsoever.** This is the one step where a
> "feels slightly slower" answer means revert, not tune.
>
> **Failure modes.** `×3` becomes nothing after any of Steps 2/4/5 → the overlay is not being consulted
> at the miss branch. Typing lags in the big note → the lookup is not in-memory, which is the PJ-066
> freeze shape. Anything odd in Focus mode → stop; Focus is one of the two editor surfaces and was the
> site of the 2026-06-12 corruption.

**Revert.** Flip `EARNED_LEDGER_READBACK` off — the old code path is intact for one build by design.

---

### Slice 14 — the adjacent defects  *(Architect S7; Q3 = (a) fix all four + PJ-163)*

**Change shape, four fixes + one:**

1. **`search.rs:995-999`** — `ensure_note_meta_review_columns` contains an
   `ALTER TABLE note_meta DROP COLUMN review_priority` that wipes every override if it meets the legacy
   shape. Replace with a shape check that **never drops a data-bearing column**; if the legacy shape is
   genuinely present, migrate the values across, never discard them.
2. **`review.rs:762`** — `save_pulse_data` is a bare `fs::write`: not atomic, no fsync, not gated
   (PJ-075). Route through the same append/persist discipline as the ledger (unique temp +
   `sync_all` + persist, `cece/reliability.rs:155-158` shape) and the write gate.
3. **`review.rs:747-757`** — `load_pulse_data` silently returns `default()` on read **or** parse failure.
   Apply the corrupt-store contract of §3.7: rename aside, surface, refuse to overwrite. This is
   invariant #3 ("absent ≠ empty") applied to the file that already had the bug.
4. **`review-pulse.json` re-keyed from absolute path to `cid_cn`** (§3.8 rule 1), with a one-shot
   stamped migration that maps existing entries by joining `note_meta`. This is why Q3 = (a): the helper
   already exists in this Plan.
5. **PJ-163 (review-pulse RMW wipe)** — same file, same function pair. Fixing 2 and 3 without closing the
   read-modify-write window would be touching the file twice for one concern.

**Tests.** `mod tests_mig104_review_adjacent`:
`ensure_columns_never_drops_a_populated_column` · `save_pulse_is_atomic_and_fsynced`
(kill mid-write; assert the old file is intact, never truncated) ·
`corrupt_pulse_is_renamed_aside_and_surfaced_not_defaulted` ·
`rename_no_longer_orphans_the_review_history` (the cid re-key: rename a reviewed note, assert
`last_reviewed` survives) · `pulse_rmw_window_is_closed` (concurrent mark+snooze does not lose either).

**Verification clause (Boss-testable — TUTORIAL).**

> **What this is.** Constellation tracks which notes you have reviewed and when, in a small file called
> `review-pulse.json`. Four separate problems were found in how that file is written and read: it could
> be truncated by a crash, a damaged file silently wiped your whole review history with no warning,
> renaming a note lost its review record, and two review actions at once could overwrite each other.
> All four are fixed.
>
> **Step 1 — pre-state.** Open the Reviewer. Mark two notes as reviewed. Note their names and that they
> now show as recently reviewed.
>
> **Step 2 — action.** Rename one of those two notes.
>
> **Step 3 — expected post-state.** In the Reviewer, the renamed note **still shows as reviewed**, with
> the same date. Today it would come back as never reviewed.
>
> **Step 4 — the corruption check.** Close Constellation. Open
> `.constellation\review-pulse.json` in Notepad, delete a single `{` from the middle, save. Re-open
> Constellation and open the Reviewer. **Expected:** Constellation tells you the review file could not be
> read, and your original file is preserved beside it with `corrupt` in its name. It does **not** silently
> show every note as never reviewed, and it does **not** overwrite your file.
>
> **Step 5 — restore.** Close the app, delete the damaged file, rename the `corrupt` one back, remove the
> character you added, and re-open. Your review history is intact.
>
> **Failure modes.** Step 3 showing "never reviewed" → the cid re-key did not run. Step 4 showing an empty
> review state with no message → the silent-default is still in place, which is the exact defect.

**Revert.** Fixes 1–3 and 5 revert cleanly. Fix 4 is a **stamped data migration** of a user file: the
revert path is documented in the commit message and the pre-migration file is kept as
`review-pulse.pre-cid.json` (never deleted).

---

### Slice 15 — docs  *(Architect S8)*

1. **`docs/concept-papers/Backup-System-Concept-Paper.md:15`** — amend the false constraint. It currently
   reads *"the `note_links` / `notes_fts` / `sky_*` SQLite tables are the **EPHEMERAL** index — NOT a
   backup target; they are **rebuilt from the files on restore**."* Replacement must name: the four
   earned link fields, `note_meta.review_priority`, and **`note_state_history` (19,481 rows, 15.1 MB, no
   on-disk twin, zero readers)** as **system-of-record-in-SQLite until MIG-104 lands** — and state
   precisely what becomes rebuildable after which slice (Slice 8 → notes deleted from that point; Slice 9
   → the whole stream, including the existing 19,481 rows).
2. **CLAUDE.md's Living Link storage section** — currently corrected (2026-07-24) to say the disk layer
   does **not** exist. Replace with what actually shipped, and only what shipped. **Per its own closing
   instruction: "Amend this section again when it ships, and only then."**
3. **`.gitignore` / backup guidance** in the help files (`docs/help.uConstellation.World/`) and the
   **User Manual** + all 14 translations: what `.constellation/` is, that the earned-life files live
   there, what the `.gitignore` covers (git only — **the honest limits of §3.2**), and that the folder is
   38 KB once the databases are excluded.
4. **Q6's stated limitation** in the help file, per whichever way the Boss rules (§4.2).
5. **Orientation doc bump** — in the **same commit** as each behaviour-landing slice, not batched (SO#6
   is a top-principal; the Architect's single "docs slice" cannot be read as permission to defer it).
6. **PJ ledger** — a new `docs/Constellation Pending Jobs v1.53.md` per SO#9: close what shipped, file
   **PJ-167** (per-library store placement, if the Boss wants it), **PJ-168** (the 235 orphan link rows
   under the retired `E:\Cognitive Knowledge` root), **PJ-169** (the LINK authoring surface — §7), and
   re-rank.
7. **LL entry** for the §2.2 lesson: *a purge ordered after the parent DELETE is a no-op when the FK
   cascades — and a comment asserting otherwise is worse than no comment.*

**Verification clause.** Not Boss-testable. Verified by: every figure in the docs matching a query in
this Plan; the 15 locale files updated; and `docs/Constellation Orientation & Onboarding v3.7x.md`
existing as a **new file** (never an overwrite).

---

## 6. Risk register — invariant → guard, per slice

| # | Invariant | Slice(s) at risk | Guard |
|---|---|---|---|
| 1 | Read-time answers always come from SQLite; the store is write-only in normal operation | 13 | Overlay reads an **in-memory** map only, behind `EARNED_LEDGER_READBACK`; **zero reader changes** across all 16 consuming surfaces |
| 2 | The note decides a link's existence; the store decides what it earned | 13 | The overlay only fills the **miss** branch; it never creates a link row |
| 3 | Absent ≠ empty for earned data | 3, 14 | The corrupt-store contract (§3.7) with `LoadReport.refuse_write`; test `structurally_corrupt_store_is_renamed_aside_and_refuses_fresh_write` |
| 4 | Never delete, always rename aside | 7, 8, 11, 14 | Compactor renames the tail; the archive is never deleted by a revert; `review-pulse.pre-cid.json` kept |
| 5 | A failed write is surfaced, never `.catch(()=>{})` | 4, 8 | Slice 8 turns `reindex_delete_note`'s unconditional `Ok(())` into a real `Err` and fixes the four swallowing call sites (LL-035 / PJ-165 shape) |
| 6 | Every link operation reversible | 3, 13 | Type-free key (Q2) + `epoch` reserved for a future reset; removing a wikilink leaves the ledger record intact |
| 7 | The archive decision survives a save | — | **Already fixed** (`search.rs:6560-6566`, `:6645-6653`); pinned by `earned_predicate_excludes_off_curve_weight_and_structural` so the predicate change cannot un-fix it |
| 8 | Zero file I/O on the keystroke and link-click paths | 4, 9 | The append moves **off** the dropped guard; the traverse caller is fire-and-forget (`store.ts:2444`) and throttled 2000 ms (`store.ts:2176`); measured against Slice 0 |
| 9 | `index_note` may ENQUEUE but never WRITE A FILE | 9, 11, 13 | All three are memory-only inside `BEGIN IMMEDIATE`; pinned by `mirror_never_writes_a_file_inside_the_transaction` and `rejoin_does_no_file_io_inside_the_transaction` (injected fs shim) |
| 10 | Zero bytes added to any boot payload | 5, 6, 9, 11, 12 | Every back-fill/restore/rebuild is stamped in `schema_versions`, spawned after paint on a dedicated connection, resumable, failure non-fatal (`link_boot_index.rs:56-76` clone); the resident sets are bounded by **earned-link count (33)** and **deleted-note count (0)** |
| 11 | Any bulk `note_links` UPDATE must be batched + background | 6, 13 | Batched writes; `restore_is_batched` asserts the statement count. Each UPDATE fires `note_links_sky_au` (DELETE+INSERT on 234k `sky_links`) |
| 12 | No temp+rename inside a watched tree without `watcher_suppress::mark()` | 7 | **Slice 1's `.constellation` predicate is the structural guard** (measured: temp+rename emits a vanished path AND two bare-dir events); the three-key `mark()` rule is the fallback because `was_recent` is exact-path keyed (`watcher_suppress.rs:35`, `:71`) |
| 13 | An externally-changed store has no adopt handler | 3 | `adopt_conflict_copies` folds `earned*.jsonl`; **Stream B is append-deduped, never folded**. Q5 = NO SYNC, so this is parity-plus, and the limit is documented rather than claimed solved |
| 14 | Windows rename-over-existing needs `ReplaceFileW` / `tempfile::persist` | 7 | `tempfile::Builder…tempfile_in` + persist (`cece/reliability.rs:155-158`) |
| 15 | Key on `cid_cn`, never on a filename | 3, 5, 8, 11 | §3.8; `tn`/`rel` are documented fallbacks with counts (567 rows lack a target cid; **261 names are shared by 576 notes**) |
| 16 | Unique temp names (PJ-087) | 7 | `temp_name_is_unique_under_concurrency` |
| 17 | Deterministic serialization — never HashMap order | 2, 3 | `BTreeMap` at `search.rs:6116`; `#[derive(Serialize)]` structs for both record shapes; `props_json_is_deterministic` |
| 18 | Nothing rewritten because time passed | 7, 9 | Byte-threshold compaction only; `threshold_is_bytes_not_time` (100 idle load cycles → zero writes); D1's fix makes an idle boot produce zero history rows |
| 19 | Merge commutative + idempotent | 3 | `fold_is_commutative_and_idempotent`, `max_fold_never_decreases_n` — arithmetic, not a rule that can have a bug |
| 20 | Write-Time Derivation — no `scan_*` / `rebuild_*` on boot | 5, 6, 9 | All are **stamped one-shots**, not per-boot walks; steady state is trigger/hook-driven |
| 21 | Form-Aligns-To-Purpose — store the earned subset | 3, 5, 13 | The ONE earned predicate (§3.6): **33 of 234,233 rows (0.014%)**, not 234k rows of zeros |
| **NEW-A** | **The archive hook must precede the FK cascade** | 8 | `archive_hook_runs_BEFORE_the_note_meta_delete` + `explicit_purge_is_a_zero_row_noop_in_production`. **This is the single defect most likely to be shipped silently** — a hook that looks correct and captures nothing |
| **NEW-B** | **A delete must not silently succeed after a failed archive** | 8 | Real `Err` + the four call sites; `failed_archive_refuses_the_purge_and_returns_Err` |
| **NEW-C** | **Note history NEVER folds and is never compacted** | 3, 7 | `history_never_folds`, `note_history_is_never_compacted`; two separate files (§3.4) so the compactor **cannot reach** Stream B |
| **NEW-D** | **`hid` is the ordinal; `captured_at` is not sufficient for order** | 3, 8, 11 | `hid_ordinal_survives_a_captured_at_collision` (765 colliding groups / 1,536 rows / 2,066 inversions live) |
| **NEW-E** | **The child-table rebuild must not lose the trigger or the index** | 12 | `rebuild_preserves_every_row_and_the_trigger_still_fires`, `indexes_are_recreated`, plus the Boss's `search.db` copy and a one-transaction rollback |

---

## 7. What this Plan does NOT do

### 7.1 The LINK authoring surface — Q4 said BUILD BOTH. My honest recommendation: a SIBLING migration, started immediately after.

**Q4's ruling stands and I am not asking to overturn it — I am asking where it lands.** The recommendation
is **MIG-106, a sibling `/migration` opened the moment MIG-104's Slice 6 is Boss-validated**, not a set of
slices inside this Plan. Reasons, stated as engineering, not preference:

1. **They share no code.** The durability layer needs **zero** reader changes across all 16 consuming
   surfaces (the Architect's "single most important structural fact"). The authoring surface is *entirely*
   reader/UI: a way to open a link, write prose about why two ideas belong together, and search that prose.
   Bundling them means the insurance job cannot ship until the feature job's UI is designed.
2. **Q4's own words — "The full CLAUDE.md vision, one job"** — are satisfied by *two migrations in
   sequence with no gap*, and are actively harmed by one migration of 15 + N slices where a UI review can
   block a durability fix. This Plan is already **15 slices**; the Architect estimated an authoring surface
   *"roughly doubles it."*
3. **The authoring surface has an unresolved design question this Plan cannot answer**: Option C's real
   LINK files placed `Links/` **outside `EXCLUDED_DIRS`**, so the recursive index walk and the sidebar walk
   descend into it and every month shard — at 35 files that is noise, at scale it is a recursive walk on a
   boot path that is a hard constraint. That needs its own Architect pass, its own Boss ruling on a
   **visible** folder, and a tree-skip decision. It is not a slice; it is a territory.
4. **The durability layer makes the authoring surface cheaper**, not the reverse: once a link's earned life
   has a durable keyed home, "open this link as an object" has something to open.

Filed as **PJ-169** with this framing, so it cannot be lost. **One line to overrule:** *"No — fold the LINK
surface into MIG-104"*, and I will re-plan with the authoring slices appended after Slice 13.

### 7.2 The time machine / backup system itself

This Plan builds **only the durable substrate the time machine will read.** It does **not** build:
a restore UI, a version-history browser, a trash panel, an empty-trash command (there is none today —
`.trash` grows forever, 57 items live), a scheduled snapshot, or a Git integration. Those are the
`docs/concept-papers/Backup-System-Concept-Paper.md` feature, which needs its own `/migration`.

What the Boss gets from *this* Plan toward the time machine, stated precisely:
* **Slice 8** — every note deleted from that build onward has its full change history on disk, keyed by
  identity, readable in Notepad.
* **Slice 9 (if approved)** — every **live** note's change history is on disk too, so the substrate covers
  the past and not only the future.
* **Slice 11** — bringing a file back re-attaches its history automatically, through the one funnel every
  re-appearance route already passes.
* **What is still missing for a true time machine:** the note's **body** at each point in time. This Plan
  archives *property/source/type* history (that is all `note_state_history` holds — only 12 of 19,481 rows
  even contain the substring `body`). Bodies live in `note_body` (7,823 rows / **272 MB** / avg 34.8 KB) and
  in the `.md` file itself. **Checklist item 6** asks whether the archive should carry the body.

### 7.3 Deferred, each with a number

| item | why deferred | PJ |
|---|---|---|
| Per-**library** store placement (`<library>/.constellation/`) so earned data travels with a detached library | Boss ruling Q1 says the Universe's config dir; the trade-off is measured in §3.1 | **PJ-167** |
| The **235 orphan link rows** under the retired `E:\Cognitive Knowledge` root (+1 from a deleted note) | Skipped, counted and surfaced by Slice 5; **[UNVERIFIED]** which code path was supposed to purge them | **PJ-168** |
| The **LINK authoring surface** | §7.1 | **PJ-169** |
| `sky_links.weight` — a third copy of the number, written by the AU trigger (`search.rs:3846`), **read by nobody** (`cache.rs:1103-1112` projects only source/target/type) | The Architect said "wire it or drop it in the same pass"; it is not on any path this Plan touches, and dropping a trigger column is a schema change deserving its own slice | **PJ-170** |
| The **conflict-marker guard** (detect Git conflict markers on read and refuse to save) — Option A's surviving insight; `yamlDoc.ts:300` returns the original frontmatter verbatim on any YAML parse error and drops every property edit **while reporting success** (PJ-085) | A live hazard today, but in the frontmatter writer — a different subsystem from the ledger, and PJ-085 already owns it | **PJ-085** (existing) |
| `note_summaries` (7,767 rows) and `sight_v3_layout` (7,635 rows) archiving | Both judged recomputable in §8's table with their recomputation sites cited. **Note `note_summaries` shares the exact FK-CASCADE-before-the-purge trap** — Slice 8's test pins the class, so if the verdict is ever revisited the ordering is already correct | — |
| **[UNVERIFIED] items carried forward:** which boot-path loop calls `index_note` on `.trash` paths ≈once per boot (proven to happen; not traced); how the 62 `.trash` `note_meta` rows were created; whether a parent-directory fsync is needed for the archive file's first creation on Windows (no in-repo precedent does it); the macOS event shape for `.constellation` writes under FSEvents; `PRAGMA legacy_alter_table` interaction with the Slice-12 rebuild | Each is a **Build-phase grep or measurement**, named in its slice. **None is theorized in this Plan.** | — |

---

## 8. Boss approval checklist

Eight decisions. Every one can be accepted with "yes" or overruled in one line.

| # | Decision | My recommendation | Cost of the other way |
|---|---|---|---|
| **1** | **Q6 — traversal counts: append immediately, no 5-second coalescing.** | **Accept.** Measured peak is 3 links per 5 seconds; 100% of your earned data sits at 1–3 walks, where a lost click makes a link read as **never walked** (not "slightly lower"). The call is already fire-and-forget and already pays a database write — one 200-byte append is cheaper than what it does now. | Accepting coalescing risks losing up to 3 links' entire recorded existence on a crash. If you want a buffer anyway, cap it at 2 seconds, never 5. |
| **2** | **Q3 — fix all four adjacent review-state defects here, plus PJ-163.** | **Accept (a).** Three of the four are silent-loss class; the fourth (re-keying review history to identity so a rename stops orphaning it) reuses the exact helper this Plan already builds. | Deferring the re-key means building the same helper twice and shipping a known "rename loses your review history" bug. |
| **3** | **Store location: ONE folder per Universe — `<universe>/.constellation/` — as Q1 says.** | **Accept.** Zero plumbing (`conn.path()` reaches it from every writer), one `.gitignore`, one place to look. | Per-library would let earned data travel with a library that is detached and re-attached elsewhere (18 of your 20 libraries live outside the Universe root). Say the word and it becomes PJ-167. |
| **4** | **Slice 9 — mirror EVERY live note's change history to disk continuously, not only on delete.** | **Build it.** Without it, the time machine covers only notes deleted after this ships; the existing **19,481 history rows (15.1 MB) across 7,785 notes stay single-copy**, and 693 rows have already been lost. Cost after the churn fix is ~2 MB/month, and it is the one slice with save-path cost — gated and measured. | Skipping it is defensible and cheaper, but then "restore my deleted note's history" only works for future deletes. |
| **5** | **Archive a deleted note's former NAMES too** (`note_aliases` 'rename' rows, deleted at `search.rs:9852`). | **Yes — add it.** Not recomputable, and nearly free in the same stream. | Losing the trail of what a note used to be called. |
| **6** | **Should the archive also carry the note's BODY?** | **Not in this Plan — but you decide.** For a normal delete the file survives in `.trash` **forever**, so the body is already safe. It is NOT safe if you empty the Recycle Bin (`system` mode) — and body archiving costs ~35 KB per deleted note (avg measured). | If you want the time machine to survive an emptied Recycle Bin, say so and it becomes a sub-slice of Slice 8. |
| **7** | **Q4 — the LINK authoring surface as a SIBLING migration (MIG-106), opened immediately after Slice 6 validates.** | **Accept the sequencing, not a change to your ruling.** Both get built, back to back. They share no code, and bundling lets a UI review block a durability fix. This Plan is already 15 slices. | Folding it in roughly doubles the Plan and delays the insurance behind a feature. One line and I re-plan. |
| **8** | **The `.gitignore` content is `*.db` / `*.db-wal` / `*.db-shm` / `boot-perf.*` / `diagnostics.log` / `sv-trace.log`** — not the Architect's `search.db*`. | **Accept.** Measured: the Architect's version leaves **896 MB** in the sync set (it misses `Constellation SV Test.db`); the corrected version takes `.constellation/` from **2,836 MB to 38 KB**, which is what removes any reason to exclude the folder your earned data now lives in. | The Architect's list leaves the objection to your Q1 ruling standing. |

**Also, for the record — two facts the Boss should know before approving, because they change what a
"successful" delivery means:**

* **Nothing in Constellation reads `note_state_history` today.** `cece_get_note_history` and
  `cece_query_history` (`cece/history.rs:317`, `:360`) are registered (`lib.rs:433-434`) and invoked
  from **nowhere** in the frontend; the surface that was to read them (Sight v3) is retired
  (orientation v3.70:3671-3674). So Slices 8–11 are verified by **reading the archive file in Notepad**
  and by the harness — **not** by a screen. The screen is the time-machine feature (§7.2).
* **`search.db` is 2.03 GB and there is a second, orphaned 939 MB database beside it**
  (`Constellation SV Test.db` — PJ-159). Together they are **99.86%** of the `.constellation/` folder.
  The `.gitignore` slice makes that irrelevant to backup; PJ-159 makes it go away.

---

*End of Plan. Nothing is built until this is approved. On approval, Slice 0 runs first — no product code
until the baseline numbers exist, because Rule 8's constraint is measured, not argued.*





