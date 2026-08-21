# PJ-334 — THE PANEL'S RULING

**2026-08-21 · four lenses (data-safety, performance, orthodoxy, refuter), each attacked by an
independent skeptic, then synthesis.** Read-only throughout — nothing was written to any of the
Boss's files; live databases were queried `mode=ro`, the MIG-108 snapshot was byte-copied to a
scratchpad before being read, and the trigger harness ran in `:memory:`.

**Convened because the author proposed a write-path trigger change and doubted his own proposal.**
The panel confirmed the doubt, found two further reasons the proposal was inert, established the
ORIGIN of the defect, and corrected four of its own members.

---

## THE PANEL'S RULING — PJ-334

**Function in hand:** the `sky_nodes` row that gives an indexed note its node in Sky View — written once by `note_meta_sky_ai` at first index, and, if lost, never written again.

**Read-only discipline:** nothing was written to any of the Boss's files. Live databases were queried `mode=ro&immutable=1`; the pre-MIG-108 snapshot was byte-copied to the scratchpad and queried there; the trigger harness ran in `:memory:`.

---

## IN PLAIN LANGUAGE, FIRST

Some notes are indexed and searchable but have **no dot in Sky View**, and nothing in the app has ever been able to put them back. The proposal on the table would not have fixed them — the author suspected that himself, and he was right.

Three things the panel found that the proposal did not know:

1. **It is not eight notes. It is 770 across five of your universes** — including **758 in `Eisa Universe`, 28% of that universe missing from its own Sky View**, along with 1,853 links drawn from nodes that aren't there.
2. **It is not only Sky View.** Those same notes are written into the Reviewer with a rank of zero, which parks every one of them permanently at the bottom of your review queue. In every universe we checked, the count of zero-ranked notes equals the count of missing dots **exactly**.
3. **A repair for this already exists in your app and already runs on every launch.** It was written three weeks ago for a narrower version of the same bug. It is one clause too narrow to see any of these 770 notes.

**What the panel rules: widen that existing repair. Do not add anything to the save path.** Cost measured on your largest universe: about **1/20th of a second, once, at launch, and only when something is actually missing**. Everything comes back on the next launch — no button, no wait, no risk to a single file on disk.

And the cause is no longer a mystery. **The panel found it, reproduced its signature in a five-note universe, and confirmed the leak was sealed on 10 August.** Details below.

---

## 1. THE MECHANISM OF PERMANENCE — CONFIRMED, but the author is wrong that "nothing restores it"

| Step | Verdict | Evidence |
|---|---|---|
| 1. `index_note` upserts, so a re-index takes the UPDATE branch | **CONFIRMED** | `search.rs:8246-8265` — `const NOTE_META_UPSERT` = `INSERT INTO note_meta (…) ON CONFLICT(path) DO UPDATE SET …` |
| 2. `note_meta_sky_ai` is the only trigger that INSERTs, AFTER INSERT only | **CONFIRMED** | `search.rs:5679-5686`; its own comment says so verbatim at `search.rs:5704-5709` |
| 3. `note_meta_sky_au` is UPDATE-only, gated on path / name / library_name / cid_cn | **CONFIRMED against the live trigger SQL**, dumped from ECK, not read from source alone | `search.rs:5746-5769` |

**But step 2's conclusion — "nothing restores it" — is REFUTED.** A full grep of every `INTO sky_nodes` in the tree returns three production writers, not one:

- `search.rs:5684` — the AI trigger (`INSERT OR REPLACE`)
- `sky_backfill.rs:297` — `INSERT OR IGNORE`, one-shot behind the version stamp (PJ-332)
- **`search.rs:6329` — `INSERT OR IGNORE INTO sky_nodes … FROM note_meta m WHERE m.cid_cn = '' AND NOT EXISTS (…)`**

The rest (`cache.rs:1646`, `sight_v6.rs:1414`/`:1675`, `mig108.rs:1670`, `search.rs:1066`, `sky_backfill.rs:627`) are inside `#[cfg(test)]` — verified by locating the enclosing attribute (`cache.rs:1581`, `sight_v6.rs:1120`).

**`search.rs:6329` is PJ-207 §15's restore for this exact defect.** It is top-level in `init_db`, not version-gated, not `owns`-gated, and it runs on every boot of every universe. **It works** — and we proved it works rather than inferring it:

- ECK holds 25 blank-cid notes; **all 25 have sky rows**.
- `Eisa Universe` holds 231 sky rows with `cid_cn = ''`, and **all 231 share one `updated_at` second: 2026-08-10 05:06:31Z** — the largest cluster in that table by a factor of five. That is one boot, one statement, 231 rows restored.

**Correction to the author, plainly:** the PJ-334 entry was written against a system it described as having no repair path, quoting §15's post-mortem while missing §15's remediation twenty lines below it. The correct framing is *"the existing restore's predicate is one clause too narrow"* — not *"nothing restores it."* That single reframing changes the whole answer.

**The other maintainers are all silent no-ops, and there are more than the entry lists** — `note_meta_sky_stratum_au` (`search.rs:5932-5938`), `note_meta_sky_maturity_au` (`search.rs:5982-5992`), `maintain_sky_after_save` (`search.rs:2705-2727`, `UPDATE … WHERE path = ?1`), and `recompute_all_sky` (`links_backfill.rs:371-380`, iterates `SELECT path FROM sky_nodes`, so it cannot create what is absent). **Five silent no-ops, one version-stamped one-shot, and one heal whose predicate excludes every affected note.**

---

## 2. THE AUTHOR'S DOUBT — CORRECT, and the proposal is inert twice over

**Reproduced**, not reasoned. A harness built from ECK's *live* trigger bodies (`:memory:`, `recursive_triggers=ON`):

```
TEST 1  hole note + ordinary edit (modified, word_count, body_text)   -> sky row: []
```

The guard names `path`, `name`, `library_name`, `cid_cn`. An ordinary edit touches none of them; `content_hash` is not even in the upsert's `DO UPDATE SET` list (`search.rs:8249-8265`). **"Any re-index of an affected note self-heals it" is false.**

**And it is worse than the author's own doubt.** Giving the *body* upsert semantics fails a second time regardless of the guard, because the body is UPDATE-only — widening the `WHEN` to include `modified` still produces no row. **Two independent reasons the proposal is inert.**

**Third, and decisive: it cannot reach these notes at all.** All 770 stranded notes carry a **non-empty `cid_cn`** (measured, every database). So the one non-rename arm of that guard — `OR OLD.cid_cn IS NOT NEW.cid_cn`, which PJ-207 §15 added for lazy cid injection — can never fire for any of them either. Under the proposal as written, **only a rename, a move, or a library rename would heal a stranded note.** Three of ECK's seven are in `.trash` and will never be renamed.

---

## 3. THE MEASURED SCOPE — 770 live, not 8

Twelve `search.db` files exist on this machine. The PJ-334 entry counted four; the panel positions counted five, eight and twelve. **The panel enumerated all twelve.**

| database | notes | sky rows | **missing** | blank-cid among missing | `sky` stamp | cid index |
|---|---|---|---|---|---|---|
| Eisa Cognitive Knowledge | 8,031 | 8,024 | **7** | 0 | 10 | partial |
| **Eisa Universe** | 2,731 | 1,973 | **758 (27.8%)** | 0 | 10 | partial |
| Scratch | 30 | 29 | **1** | 0 | 10 | partial |
| جوامع عيسى الشامسي | 6 | 5 | **1** | 0 | 10 | partial |
| Eisa Universe\\كون عيسى | 5 | 4 | **1** | **1** | 10 | **FULL (legacy)** |
| Eisa Universe\\كون عيسى 2 | 5 | 4 | **1** | **1** | 10 | **FULL (legacy)** |
| Eisa Universe\\كون عيسى 3 | 5 | 4 | **1** | **1** | 10 | **FULL (legacy)** |
| موسوعة عيسى *(the real PKF)* | 832 | 832 | **0** | — | 10 | partial |
| كون عيسى | 6 | 6 | 0 | — | 10 | partial |
| Review Demo | 3 | 3 | 0 | — | 10 | FULL |
| MIG108 Rehearsal *(a rehearsal copy)* | 7,827 | 7,809 | 18 | 14 | 10 | FULL |
| Constellation Test | 5,161 | *no `sky_nodes` table; `schema_versions` empty* | n/a | — | — | — |

**770 stranded rows in live universes; 788 including the rehearsal copy.** Orphan sky rows: **0 in every database.**

**The blast radius is not cosmetic and it is not confined to Sky View.** `index_note` reads `SELECT CAST(stratum AS INTEGER) FROM sky_nodes WHERE path = ?1` with `.unwrap_or(0)` (`search.rs:8402-8406`); `review.rs:1408-1412` does the same. Measured:

| `review_schedule.stratum = 0` | equals missing-sky count? |
|---|---|
| ECK: 7 | **7 / 7 exact** |
| Eisa Universe: 758 | **758 / 758 exact** |
| Scratch: 1, Jawami: 1 | **exact** |
| موسوعة عيسى: 0 | **exact** |

The Reviewer sorts its due queue `b.stratum.cmp(&a.stratum)` (`review.rs:496`). **Every one of these 770 notes is ranked last in the review queue, permanently, and every save rewrites that zero.**

Also measured: **`sky_links` rows sourced from a path with no node — ECK 236, Eisa Universe 1,853.** The repair is not "node-only, one table"; it makes those edges drawable again.

`.trash`: ECK 57 `note_meta` rows, **54 with sky rows**; Eisa Universe 14/14. **`.trash` is not the discriminator and must not be made one.**

---

## 4. THE ORIGIN — **ESTABLISHED**, and the leak is sealed

The panel did not need to take this on faith, and it did not stop at correlation.

**MIG-108 is EXONERATED, by a stronger test than any position ran.** `Eisa Universe`'s own pre-run snapshot (`mig108-backup/search.db.pre-mig108`, journal `"phase":"done"`) was compared to the live database **joined on `cid_cn`, not on `path`** — because MIG-108 rewrites paths and a path join proves nothing:

```
SNAPSHOT: notes 1891  sky 1729  missing 162  orphans 0
live stranded = 758 | existed pre-MIG108 = 162 | HAD a sky row pre-MIG108 = 0
```

**MIG-108 destroyed zero sky rows.** The remaining 596 did not exist in `note_meta` at all before the run — they arrived with the libraries it unified.

**The snapshot then hands over the mechanism.** Its `sqlite_master` shows all 11 triggers present — so trigger absence is refuted — and:

```
CREATE UNIQUE INDEX idx_sky_nodes_cid_cn ON sky_nodes(cid_cn)     -- FULL. No WHERE.
missing by library: الكون المعرفي 161 ... of 162 notes
```

**One library, 162 notes, exactly one survivor.** That is the `INSERT OR REPLACE` chain signature, and it is arithmetic, not inference.

**Isolated cleanly in a five-note universe.** `Eisa Universe\\كون عيسى`, still carrying the legacy FULL index, caught mid-mechanism before any cid injection:

```
Observation — Recent Captures.md   cid ''                              sky 0   <- deleted
إختبار المرحلة 2.md                 cid ''                              sky 1   <- survivor
Image Preview Test.md              cid '20260329T000000Z_NOTE_4CC5'    sky 1
تجربة 1.md / مثال1.md               real cids                           sky 1
```

**Exactly two cid-less notes; exactly one sky row between them.** The same file — `Five Acts\\Observation — Recent Captures.md`, the system note written cid-less at universe init — **is the sole stranded note in five separate universes**, still `cid_cn = ''` in the three that have not booted since the fix, already cid-injected (and therefore invisible to the §15 restore) in Scratch and جوامع.

**The chain, every link verified:**

1. Notes are indexed while `cid_cn` is still `''` (a bulk library import; the Five Acts system note at init).
2. `note_meta_sky_ai`'s `INSERT OR REPLACE` collides on the then-**FULL** `UNIQUE(sky_nodes.cid_cn)`. REPLACE = DELETE + INSERT: each cid-less note deletes the previous one's row. **One survivor per cohort.**
3. The cid is injected later. That takes the **UPDATE** branch — `note_meta_sky_ai` never fires again; `note_meta_sky_au` fires but only UPDATEs a row that is not there. **Permanent.**
4. `2edc97d7` (**2026-08-10 11:30:58 +0400**) made the index partial — sealing the leak — and added the §15 restore scoped `WHERE m.cid_cn = ''`. It healed everyone still blank (ECK 25, EU 231) and **permanently skipped everyone who had already acquired a cid (ECK 7, EU 758)**.

`Constellation PKM` closes the arithmetic: **808 notes = 234 still-blank at restore time (223 of them healed into that 05:06:31Z cluster) + 574 already cid-injected and stranded.**

**Is the producer dead? Yes, for any database that boots.** `ensure_dependent_tables_mig003_indexes` runs unconditionally at `search.rs:6304`, ahead of the restore, and converts a legacy FULL index to partial. And across all four active universes, **every note created or modified after 2026-08-10 07:30 UTC has a sky row — zero exceptions.** *(Labelled: that post-fix sample is only 19 notes. The date boundary is strong; the sample is small. The design below does not depend on it — see §5.)*

**Ruling on question 5: the origin is established, and it did not need to be, but knowing it changes the design.** Because the producer is closed, a **one-shot repair is sufficient and a save-path trigger is not needed**. Had the origin stayed unknown, the panel would have been forced onto the write path.

---

## 5. THE RULING — DO THIS

**Widen the repair that already exists, in `init_db`. Change no trigger.**

Order matters and is preserved: this sits **after** `ensure_dependent_tables_mig003_indexes` (`search.rs:6304`), for §15's own stated reason — against a FULL index an `INSERT OR IGNORE` swallows the collision it exists to repair.

**(a) Leave the shipped narrow arm at `search.rs:6327-6344` exactly as it is.** No shipped line is overturned; foreign databases keep receiving their blank-cid rows back.

**(b) Add, inside the existing `if owns` block, a count-gated two-phase repair.**

```
GATE:     SELECT (SELECT count(*) FROM note_meta), (SELECT count(*) FROM sky_nodes)
          -> proceed only when they differ
PHASE 1:  SELECT m.path FROM note_meta m
           WHERE NOT EXISTS (SELECT 1 FROM sky_nodes s WHERE s.path = m.path)
PHASE 2:  for those paths only —
           INSERT OR IGNORE INTO sky_nodes (path, id, name, library_name, cid_cn, updated_at)
             SELECT … FROM note_meta WHERE path = ?      -- PK seek
           UPDATE sky_nodes SET stratum = ({stratum_expr}), maturity = ({maturity_expr})
            WHERE stratum IS NULL AND path = ?
```

**Measured by the panel, on the Boss's live databases, read-only:**

| | query plan | cold | warm |
|---|---|---|---|
| **Gate** (two counts, ECK 8,031 notes / 1.6 GB) | two covering-index scans | **54.6 ms** | **0.0 ms** |
| **Phase 1** (ECK) | `SCAN m USING COVERING INDEX idx_note_meta_path_modified` + `SEARCH s USING COVERING INDEX sqlite_autoindex_sky_nodes_1` | **806 ms** | **2.2 ms** → 7 rows |
| **Phase 1** (Eisa Universe, 2,731 notes) | same | **2.2 ms** | **1.0 ms** → 758 rows |
| **the trap — one statement with the full column list** | **`SCAN m`** — the table b-tree, dragging 273 MB of `body_text` | — | **25 ms** |

**Read the first two rows together — that is the whole design.** The anti-join is 806 ms **cold**, and boot is cold by definition. The gate reduces the steady-state boot cost to **55 ms cold / free warm**, and the 806 ms is paid **once**, on the one boot that actually repairs. *(Phase 2's write cost was measured by a panel member on a scratch copy at ~30 ms insert + ~190 ms stamp for 758 rows; the panel did not re-run any write, and labels that figure second-hand.)*

**Non-negotiable clauses, each with its reason:**

1. **`INSERT OR IGNORE`. Never `OR REPLACE`.** REPLACE is DELETE + INSERT and deletes on **any** uniqueness conflict — it is the mechanism that caused this bug (`search.rs:4508-4517`). It would also reset `stratum`, `origin_type`, `created_at`, `link_count` to defaults. **Never a bare `INSERT`** either: a constraint violation inside a trigger or a batch aborts the enclosing statement.
2. **Widen the stamp, not just the insert.** The shipped stamp carries the **same narrow predicate** — `AND path IN (SELECT path FROM note_meta WHERE cid_cn = '')` (`search.rs:6338-6341`). Widening the INSERT alone would restore 770 rows with `stratum` NULL, into tables that today hold **zero** NULL strata, and `unwrap_or(0)` would write stratum 0 straight back into the Reviewer — **re-creating the exact harm being repaired, and reporting success.** A row is restored complete or it is not restored.
3. **Keep the whole new arm `owns`-gated.** `stratum_sql_expr()` / `maturity_sql_expr()` read the active universe's link-type registry (PJ-232), so a foreign database cannot be given a complete row. Boss ruling 2 (2026-08-17) holds unchanged; the owner repairs its own on its next launch.
4. **No `.trash` exclusion.** 54 of ECK's 57 trashed notes already have sky rows; excluding trash would make the derived view *less* consistent than it is now and would strand a note that is later restored from trash.
5. **Report `candidates` / `inserted` / `stamped` as three separate numbers to `diag_log`.** `OR IGNORE` can silently under-heal on a cid collision. *Verified it does not fire today* — 0 duplicate non-empty cids in any database, and **0 of the 770 stranded cids appear in any existing sky row** — but a count that assumes success is the false-success class.
6. **No `scan_*` / `rebuild_*` command, no walk, no file I/O, no back-fill cursor touched, `sky_backfill::is_needed` untouched.** The PJ-332 bar — "8,031 notes re-read on every boot, forever" — is cleared **by construction**: this is pure SQL over `note_meta`. Rule 8 is satisfied by its own back-fill clause; the count gate makes the repair standing and idempotent, so it re-fires if the defect ever recurs.

**Verification before it ships:** assert the widened repair restores exactly the 7 named ECK paths and 0 elsewhere; assert `SCAN m` does **not** appear in Phase 1's query plan (the 25 ms → 800 ms regression guard); assert restored rows carry non-NULL `stratum` and `maturity`; assert a duplicate-cid candidate is skipped rather than destroying the incumbent, and is counted in the receipt.

---

## 6. WHAT THE PROPOSAL BREAKS — including two hazards no position stated correctly

**(a) A heal trigger on the save path is REJECTED, and the panel reproduced why.** Harness, live trigger bodies, `recursive_triggers=ON`:

```
TEST 8  firing order of three AFTER UPDATE triggers  -> z_last, m_mid, a_first
        (SQLite fires them in REVERSE creation order — and guarantees no order by contract)

TEST 4  unguarded heal + rename of a healthy BLANK-CID note
        -> UNIQUE constraint failed: sky_nodes.path

TEST 3  same rename, non-blank cid
        -> passes; the partial UNIQUE index on cid_cn silently swallows the INSERT and MASKS the bug
```

`sky_nodes.path` is the PRIMARY KEY. A heal that fires before `note_meta_sky_au` inserts a bare row at `NEW.path`, and the rename's `UPDATE sky_nodes SET path = NEW.path WHERE path = OLD.path` then collides. `migrate_note_db_paths` runs every statement through a **best-effort runner that logs and continues** (`libraries.rs:1584-1596`) — so the file moves on disk, every other table migrates, and **`note_meta` is left stranded at a dead path.** ECK holds 31 blank-cid sky rows and Eisa Universe 231: **reachable by renaming any of them.** This turns a 770-row cosmetic gap into the silent index↔disk divergence class.

**(b) And the order-safe placement is not safe either.**

```
TEST 9  guarded heal firing LAST + ordinary edit  -> sky row: (stratum None, maturity None)
TEST 10 guarded heal, edit changing only `modified` -> (stratum None, maturity 'sapling')
```

A healed row is only complete if the heal fires *before* the stratum/maturity AU triggers **and** the edit happens to change `word_count`. Neither is guaranteed. A write-path heal would therefore have to carry its own stratum stamp — putting the registry-reading CASE chain on the save path, in a trigger that cannot be `owns`-aware.

**(c) Trigger recursion — REFUTED, verified not assumed.** `sky_nodes` carries **zero triggers** in all five databases checked. `pragma foreign_key_list(sky_nodes)` is empty. An INSERT into it cascades nowhere.

**(d) The save-path cost objection is REJECTED on its own terms — and it is not why we said no.** A `NOT EXISTS` probe is a single covering-index seek and measures in microseconds against a ≥1500 ms save debounce; `note_meta_sky_maturity_au` already fires on **every** save (`search.rs:5982-5990` — its own comment says so), and `sight_v6_layout_invalidate_au` has no `WHEN` clause at all. **The panel does not kill the trigger on latency. It kills it on (a), (b), and on being unnecessary: the producer is closed, so a standing write-path guard buys nothing a one-shot boot repair does not already give.**

---

## 7. WHERE THE PANEL FOUND ITS OWN MEMBERS WRONG

- **"7 notes, 15 microseconds, no door needed"** — derived from a census of four databases. At 770 the proportionality argument had to be re-taken (§8).
- **"The repair is node-only, 8 rows, one table"** — refuted: 236 + 1,853 `sky_links` rows are sourced from missing nodes.
- **"`INSERT OR REPLACE` would cause universe-wide erosion of earned enrichment; `created_at` unrecoverable"** — **not supported by the Boss's data.** `link_count`, `outgoing_count`, `created_at`, `origin_type` are 0/0/NULL/NULL on **every row** of every database; `enrichment_dirty` is 1 on 100%. The real cost of REPLACE here is transient blanking of stratum and maturity. The verdict against REPLACE stands on §6's grounds; that particular evidence was harness output presented as a finding about live data.
- **"MIG-108 is the origin"** and **"MIG-108's window is suspicious"** — both closed by the cid-joined snapshot comparison: 0 of 758 had a sky row before the run.
- **"The AI-dispatch-skip is the origin"** — dropped. It is deterministic; if it were live in the shipping schema, 8,024 sky rows could not exist. Two contradicting source comments (`search.rs:5906-5913` vs `search.rs:4645-4661`) are zero evidence, not one.

---

## 8. WHAT THE PANEL DECIDES vs WHAT ONLY THE BOSS DECIDES

**The panel decides (engineering, and it is decided):** reject the trigger proposal; widen the existing boot restore in the two-phase count-gated form; `INSERT OR IGNORE` only; widen the stamp with it; `owns`-gate the new arm; include `.trash`; three-number receipt to `diag_log`. **No `DriftReport`** — its fields close on the identity `files_seen == unchanged + drifted + missing_from_index + files_unreadable` (`reconcile.rs:83-118`), a filesystem-vs-index axis; this is an index-vs-derived axis and would break that identity and mis-name the finding in fifteen languages.

**Only the Boss decides — one question, and the panel recommends briefly:**

> **`Eisa Universe`'s Sky View will gain 758 nodes and 1,853 edges on its next launch, and three trashed notes will reappear in ECK's.** That is a visible change to a knowledge surface, unannounced. At 7 notes the panel would have shipped it silently. At 758 it will not decide that for you.
>
> **Recommendation: ship it silently but *loudly logged*** — automatic repair, one line in the status bar naming the number repaired, no button, no progress strip, no fifteen locale files. Nobody answers "leave 758 of my notes invisible," so a *permission* door is disproportionate; a *notification* is not. **PJ-207 got a full door because it re-reads files and rewrites your frontmatter. This reads nothing and writes one row per missing note.**

*(Secondary, and genuinely yours: whether trashed notes belong in Sky View at all. Today 54 of 57 are there. The repair makes the existing behaviour consistent; it must not be used to smuggle in a change of that policy.)*

---

## 9. FILED SEPARATELY — do NOT fold into PJ-334

1. **`sky_nodes.cid_cn` is never maintained after INSERT.** `note_meta_sky_au` fires on `OR OLD.cid_cn IS NOT NEW.cid_cn` (`search.rs:5751-5756`) and its body (`:5763-5769`) sets `path`, `id`, `name`, `library_name`, `updated_at` — **never `cid_cn`**. Measured divergence: **ECK 6, Eisa Universe 18, Scratch 1**, all of the form `sky.cid_cn = ''` while `note_meta` holds a real cid; `sky.id` / `name` / `library_name` mismatches are **0**, so this is one unmaintained column, not general rot. **This producer is still live** — ECK's six are all dated 2026-08-11, after the fix. Note also that **no reader of `sky_nodes.cid_cn` exists anywhere in the tree** — the UNIQUE index that destroyed 770 rows sits on a column nothing reads. Two cheap candidate fixes (add `cid_cn = NEW.cid_cn` to the AU body, or drop the column and its index). **It touches the very trigger this ruling declined to touch — separate job, Whole-Ecosystem Fix Law.**
2. **Enrichment worker has never drained.** ECK: `origin_type` NULL on 8,024/8,024 and `enrichment_dirty = 1` on 8,024/8,024. The §7 queue is 100% of the universe. Out of scope; needs its own entry.
3. **`Constellation Test`** (5,161 notes) has **no `sky_nodes` table and an empty `schema_versions`** — `init_db` has not run there in a long time. Panel did not establish whether it is a live universe; noted so it is not mistaken for a new defect.

---

## 10. WHAT THE PANEL DID **NOT** VERIFY

- **That any specific note's `cid_cn` was `''` at the instant of its collision.** The aggregate arithmetic closes (162 → 1 survivor; 808 = 234 + 574; two cid-less notes → one sky row), and the FULL index is proven present in the snapshot — but the per-note state at collision time is **not established**. It does not gate the repair.
- **Cold timings are single samples on this machine**, with another process holding `Eisa Universe`'s `-shm`. The **query plans**, not the milliseconds, are the durable claim.
- **Phase 2's write cost (~220 ms for 758 rows)** is second-hand from a panel member's scratch-copy benchmark; the panel ran no writes.
- **All harness reproductions ran on SQLite 3.50.4** (Python's). The app bundles **3.45.x** (`libsqlite3-sys 0.28.0`). The firing-order and PK-collision results are standard SQLite semantics, but were **not** re-run on 3.45.
- **Whether `Eisa Universe` is a universe the Boss still actively uses.** Measured only: `owner.info.json` heartbeat 2026-08-17T18:39Z, `boot-perf.latest.json` 2026-08-17, 2,731 notes across 8 libraries, last written 2026-08-12. The dev registry lists only `كون عيسى` and is not the answer. **Its stranded notes sit in `Constellation PKM` (574), `الكون المعرفي` (161), `Eisa Universe` (22), `تخطيط الدولة` (1) — none in its `موسوعة عيسى` library, which is clean.**
- **`sky_backfill.rs` is uncommitted working-tree code** (`git status: M`, +255/−48). PJ-332's `finalize` completeness report is **proposed, not shipped**; the receipt in §5 clause 5 should land alongside it.

---

## THE ONE-LINE RULING

**Do not touch a trigger. Widen `search.rs:6329`'s repair — both statements, one candidate set, two phases, behind a count gate, `owns`-gated, `INSERT OR IGNORE`, `.trash` included, three-number receipt — and 770 notes come back on the next launch for about a twentieth of a second, once.**"
  },
  "workflowProgress": [
    {
      "type": "workflow_phase",
      "index": 1,
      "title": "Positions"
    },
    {
      "type": "workflow_phase",
      "index": 2,
      "title": "Attack"
    },
    {
      "type": "workflow_phase",
      "index": 3,
      "title": "Ruling"
    },
    {
      "type": "workflow_agent",
      "index": 1,
      "label": "pos:data-safety",
      "phaseIndex": 1,
      "phaseTitle": "Positions",
      "agentId": "a7e2c4eda0dc8dd49",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787327168248,
      "queuedAt": 1787327168243,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "E:/مشاريع كلاود/Constellation" && sed -n '5914,5922p' s…",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787327745778,
      "tokens": 172105,
      "toolCalls": 40,
      "durationMs": 577525,
      "resultPreview": "# RULING — PJ-334

**Repair. But not the proposed repair, and not at the proposed trigger.** The author's doubt is correct, his conclusion from it is wrong, and the design he needs is already shipped in his own codebase 20 lines from where he was looking.

---

## 1. The mechanism of permanence — CONFIRMED, and stronger than stated

All three steps hold. Verified against source **and** against the…"
    },
    {
      "type": "workflow_agent",
      "index": 2,
      "label": "pos:performance",
      "phaseIndex": 1,
      "phaseTitle": "Positions",
      "agentId": "aa794d12044d79d24",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787327168250,
      "queuedAt": 1787327168243,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "C:/Users/ealsh/AppData/Local/Temp/claude/E-------------…",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787327671046,
      "tokens": 138375,
      "toolCalls": 25,
      "durationMs": 501285,
      "resultPreview": "## RULING — PJ-334, from the save-path lens

**Verdict in one line: the performance objection does not hold — I measured it and the heal costs 3 µs per save, 1/20th of what the same UPDATE statement already pays unconditionally. I do not kill it on latency. I kill the *form* the proposal implies, on a different ground: `INSERT OR REPLACE` would re-arm the app-killer PJ-207 §15 just removed. And th…"
    },
    {
      "type": "workflow_agent",
      "index": 3,
      "label": "pos:orthodoxy",
      "phaseIndex": 1,
      "phaseTitle": "Positions",
      "agentId": "a6b2dd93d64d1684e",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787327168250,
      "queuedAt": 1787327168243,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "E:/مشاريع كلاود/Constellation" && git log --format="%h …",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787327844898,
      "tokens": 186981,
      "toolCalls": 47,
      "durationMs": 675137,
      "resultPreview": "I have everything. Writing the ruling.

---

# RULING — PJ-334
## Lens: Rule 8 / Write-Time Derivation / the codebase's own precedents

**Function in hand:** the `sky_nodes` derived row for an indexed note — the row Sky View draws a node from, written at note-index time by `note_meta_sky_ai` and never again.

---

## HEADLINE RULING

**Reject the proposal. It is the wrong idiom, in the wrong place…"
    },
    {
      "type": "workflow_agent",
      "index": 4,
      "label": "pos:refuter",
      "phaseIndex": 1,
      "phaseTitle": "Positions",
      "agentId": "af9a00b277f2211cf",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787327168251,
      "queuedAt": 1787327168243,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "C:/Users/ealsh/AppData/Local/Temp/claude/E-------------…",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787327779261,
      "tokens": 145158,
      "toolCalls": 39,
      "durationMs": 609500,
      "resultPreview": "I have what I need. Writing the ruling.

---

# RULING — PJ-334

**Verdict: the diagnosis is materially right, the proposed fix is wrong, and the scope claim is wrong by two orders of magnitude.** I set out to argue "do nothing" and the live data refuted me. What survives is a *different* fix than the one proposed.

---

## 1. The mechanism of permanence — 2 steps confirmed, 1 **overstated**

| St…"
    },
    {
      "type": "workflow_agent",
      "index": 5,
      "label": "atk:performance",
      "phaseIndex": 2,
      "phaseTitle": "Attack",
      "agentId": "a9d2ef5c7bc4ff3f3",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787327671054,
      "queuedAt": 1787327671051,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "E:/مشاريع كلاود/Constellation" && sed -n '371,410p' src…",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787328256782,
      "tokens": 151841,
      "toolCalls": 39,
      "durationMs": 585728,
      "resultPreview": "# ADVERSARIAL REVIEW — the "performance" position on PJ-334

**Verdict: the position's factual core survives; its two load-bearing conclusions do not. It asserted as CONFIRMED a claim that is false, and that false claim is the reason it recommended a save-path trigger. A repair for this exact defect ALREADY SHIPS and ALREADY RUNS ON EVERY BOOT — `search.rs:6329` — and its candidate set is `WHERE m…"
    },
    {
      "type": "workflow_agent",
      "index": 6,
      "label": "atk:data-safety",
      "phaseIndex": 2,
      "phaseTitle": "Attack",
      "agentId": "abddc95622c716b08",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787327745780,
      "queuedAt": 1787327745779,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "E:/مشاريع كلاود/Constellation" && grep -n "" src-tauri/…",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787328702020,
      "tokens": 213076,
      "toolCalls": 63,
      "durationMs": 956240,
      "resultPreview": "## VERDICT — the position is **largely sound in mechanism and largely wrong in scale.** Its core engineering call survives; the two claims it advertises as its own corrections — the census and the cost — are both refuted, and the second failure follows from the first. Confirmed with amendments; its "no door" recommendation is **rejected**.

---

## 1. What verifies (I re-derived every one; nothing…"
    },
    {
      "type": "workflow_agent",
      "index": 7,
      "label": "atk:refuter",
      "phaseIndex": 2,
      "phaseTitle": "Attack",
      "agentId": "acd28b6fc6a4b530f",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787327779263,
      "queuedAt": 1787327779262,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "C:/Users/ealsh/AppData/Local/Temp/claude/E-------------…",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787328928978,
      "tokens": 193955,
      "toolCalls": 54,
      "durationMs": 1149715,
      "resultPreview": "# ATTACK ON THE REFUTER POSITION — PJ-334

**Verdict on the position: its diagnosis survives, its forensics are half-wrong, its arithmetic is wrong by three orders of magnitude, and its own recommendation (a) is an app-killer as written.** I reproduced the defect it found *and* a silent index↔disk divergence its fix introduces, on the Boss's live data. Where I could not verify on the shipping buil…"
    },
    {
      "type": "workflow_agent",
      "index": 8,
      "label": "atk:orthodoxy",
      "phaseIndex": 2,
      "phaseTitle": "Attack",
      "agentId": "ad51812fa5ca6998f",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787327844900,
      "queuedAt": 1787327844899,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "E:/مشاريع كلاود/Constellation" && sed -n '8396,8415p' s…",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787328891256,
      "tokens": 206567,
      "toolCalls": 52,
      "durationMs": 1046356,
      "resultPreview": "## RULING ON THE ORTHODOXY POSITION — PJ-334

**Verdict on the position: UPHELD IN ITS CONCLUSION, REFUTED IN THREE OF ITS SUPPORTS, AND ITS DESIGN AS WRITTEN IS DEFECTIVE.**

The position is right that the answer is the existing PJ-207 §15 boot restore with a wider predicate, and right to reject the trigger. But it ruled on a census it never widened, and the design it hands the Boss silently repr…"
    },
    {
      "type": "workflow_agent",
      "index": 9,
      "label": "ruling",
      "phaseIndex": 3,
      "phaseTitle": "Ruling",
      "agentId": "a4cc0e6e89fc2b15f",
      "model": "claude-opus-5",
      "state": "done",
      "startedAt": 1787328928982,
      "queuedAt": 1787328928980,
      "attempt": 1,
      "lastToolName": "Bash",
      "lastToolSummary": "cd "C:/Users/ealsh/AppData/Local/Temp/claude/E-------------…",
      "promptPreview": "Constellation (Tauri v2 + SvelteKit personal-knowledge app), repo E:/مشاريع كلاود/Constellation. **READ-ONLY**: do not edit
files, do not run cargo build/test. You MAY read, grep, and query SQLite **read-only**.

Laws in force: **Don't Make Things Up**; **No Guessing — Investigate** (every claim from something
read or queried, never inferred); **Reproduce-First** (no defect-targeting change is eve…",
      "lastProgressAt": 1787329763997,
      "tokens": 192749,
      "toolCalls": 36,
      "durationMs": 835015,
      "resultPreview": "## THE PANEL'S RULING — PJ-334

**Function in hand:** the `sky_nodes` row that gives an indexed note its node in Sky View — written once by `note_meta_sky_ai` at first index, and, if lost, never written again.

**Read-only discipline:** nothing was written to any of the Boss's files. Live databases were queried `mode=ro&immutable=1`; the pre-MIG-108 snapshot was byte-copied to the scratchpad and q…"
    }
  ],
  "totalTokens": 1600807,
  "totalToolCalls": 395
}
