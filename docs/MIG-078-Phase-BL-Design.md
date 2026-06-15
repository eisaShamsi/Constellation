# MIG-078 Phase BL — De-bloat `note_meta` (move `body_text` → `note_body`)

> Detailed design for Phase BL of [MIG-078-Plan.md](MIG-078-Plan.md). Produced by the BL impact-review workflow (4 dependency maps + Architect synthesis), then **hardened by Claude with live experiments that retired 4 of 5 risks**. Full workflow output: session task `wffa0had8`.

## Goal
Relocate the inline `note_meta.body_text` column (404 MB) into a dedicated `note_body(path, body_text)` table, drop the column, and `VACUUM` — reclaiming ~783 MB (404 MB body + 379 MB freelist) from the 1.69 GB DB, shrinking boot scans and the covering-index reads.

## The one overriding constraint
Full-text search results, the `notes_vocab` term dictionary, and the Index-panel output must be **byte-identical at every intermediate commit** — never a window where a search returns stale/missing rows.

---

## Load-bearing decision — DO NOT touch the FTS5 `content=` config
`notes_fts` is external-content FTS5: `content=note_meta, content_rowid=rowid` (`search.rs:2237-2238`). That clause is baked in at CREATE and cannot be ALTERed; changing it forces a **full FTS rebuild** (re-tokenize all 7,653 notes incl. the 123 MB one → the multi-minute hang the WHEN-gate exists to prevent) and would churn internal docids → **not** byte-identical.

**The triggers are the only writer to the FTS index** (`INSERT INTO notes_fts(rowid,name,body_text) VALUES(...)`). FTS5 serves `MATCH` from its own stored postings and only dereferences the content table on `'rebuild'`/`'integrity-check'` — which we never call (Write-Time Derivation). So: **keep `notes_fts` exactly as-is; keep `notes_fts.rowid == note_meta.rowid`; only change where the triggers READ the body from.** Bridge `note_meta ↔ note_body` on **`path`** (shared PK), never on rowid (two tables' rowids are not guaranteed equal).

### ✅ VERIFIED by isolated experiment (Python sqlite 3.50, FTS5)
- After repointing the trigger off `body_text`, `ALTER TABLE note_meta DROP COLUMN body_text` **succeeds**, and `MATCH` results are **byte-identical** before vs after the real drop (`fox→[1], knowledge→[3], dog→[2]` unchanged). **Risk R1 retired.**
- `DROP COLUMN` is **blocked while any trigger references the column** ("no such column: new.body_text") → triggers MUST be repointed in §BL.2 *before* the §BL.3 drop. (Mandatory ordering, now proven.)
- A new note inserted with **note_body written before note_meta** is immediately searchable via the note_body-sourced trigger (`serendipity→[4]`). (The ordering fix works.)

---

## Corrected trigger design (the workflow's `content_hash` gate was INVALID)
**Finding:** `note_meta.content_hash` is **never written** — the only `content_hash` writes are to the `note_summaries` table (`nsc/mod.rs:656`). The `index_note` UPSERT (`search.rs:4079`) writes `body_text` but not `content_hash`. So the workflow's proposed `WHEN OLD.content_hash IS NOT NEW.content_hash` gate would never fire. **Rejected.**

**Replacement — split the FTS sync by where the signal lives:**
- **note existence + name** live on `note_meta` → keep INSERT/DELETE/UPDATE(name) triggers there:
  - `note_meta_ai` (INSERT) → FTS insert; body via `LEFT JOIN note_body nb ON nb.path=new.path`.
  - `note_meta_ad` (DELETE) → FTS delete; old body from `note_body` (fires before the `note_body` row is deleted — ordering: delete note_meta first, then note_body).
  - `note_meta_au` **`WHEN OLD.name IS NOT NEW.name`** → FTS delete+insert; body from `note_body`. (Preserves the perf gate: metadata-only `note_meta` updates don't churn FTS.)
- **body** lives on `note_body` → new trigger:
  - `note_body_au` **`WHEN OLD.body_text IS NOT NEW.body_text`** → FTS delete+insert, using `note_meta.rowid` looked up by `path`. Fires exactly when body changes (preserves the perf gate). Does NOT touch `content=`.

A save that changes both name and body fires both triggers (idempotent final state, one transaction) — harmless.

---

## Phasing (each = one commit; invariant after each)
**§BL.1 — add `note_body`, dual-write, backfill.** `CREATE TABLE note_body(path PRIMARY KEY, body_text NOT NULL DEFAULT '')`. `index_note`: keep the `note_meta` write, ADD a `note_body` upsert in the same txn (**write note_body FIRST** for the §BL.2 trigger ordering). `reindex_delete_note`: add `DELETE FROM note_body`. Resumable post-paint backfill (sentinel `note_body_backfill`), byte-capped batches (the 123 MB note alone). **No reads/triggers change.** *Invariant:* `note_meta.body_text` still feeds everything; `note_body` is a shadow. Search provably unchanged. DB grows transiently (~404 MB; 7.9 TB free — fine).

**§BL.2 — flip reads + triggers to `note_body`** (only after the backfill sentinel + divergence-check = 0). Repoint the 16 read sites (below) and the CTSE old/new-body reads; swap the triggers to the corrected design above. **Keep dual-writing `note_meta.body_text`** as a hot standby (rollback = pure code revert, zero data loss). *Invariant:* all reads/FTS flow through `note_body`, which equals `note_meta.body_text` row-for-row → byte-identical. **This is the §8 harness gate + the Boss search-identity test.**

**§BL.3 — drop + VACUUM** (only after §8 passes on a DB copy + a tagged/ZIP backup). Stop dual-writing; `ALTER TABLE note_meta DROP COLUMN body_text`; guarded post-paint `VACUUM` (atomic, retriable; ~1.7 GB temp, free space OK). *Invariant:* single body source = `note_body`; search still byte-identical (proven pre-drop); DB ~783 MB smaller.

---

## Read sites to repoint in §BL.2 (from the dependency maps)
`search.rs`: lexical snippet 4317-4319 · `synth_snippet_for_body` 4413-4459 (no SQL) · CTSE pre-delete SELECT 6970 · CTSE pre-update 7011 · CTSE post-update 7027 · mention filter 4932 · fulltext replace 7682 · test helper `body_of` 7838 · test fixtures 8052/8244/8851/9275/9297. `libraries.rs`: cooccurrence 3925-3927 (rowid call site → join note_body by path). `nsc/mod.rs`: 476-477. `sight_v6.rs`: 372, 595. **No change:** boot snapshot reader (`cache.rs:893-896`, already body-free); `notes_vocab` (untouched → Index panel unchanged).

## Backfill / rollback / harness
- **Backfill:** `INSERT INTO note_body SELECT path,body_text FROM note_meta WHERE path > :cursor ORDER BY path LIMIT :batch ON CONFLICT DO UPDATE` — cursor in a progress row, idempotent, byte-cap the 123 MB note alone. Gate §BL.2 on `COUNT(note_meta LEFT JOIN note_body WHERE nb IS NULL)=0` AND `COUNT(divergent bodies)=0`.
- **Rollback:** §BL.1/§BL.2 are pure code reverts (note_meta.body_text kept current). §BL.3 is the point of no return → tagged milestone + 1.69 GB DB ZIP before it ships; keep ≥1 week.
- **Search-identity harness (Boss-test gate):** on a DB *copy* — snapshot `notes_vocab` dictionary + a ~50-query battery (EN, Arabic stems, bigram, mention-filter, rare terms, a term only in the big note) rowid-lists + Index-panel JSON; apply §BL.1+§BL.2+backfill; re-snapshot → **empty diff required**. Then apply §BL.3 → re-run battery (proves Option A post-drop) + size drop. Boss runs the same 5 searches + Index panel pre/post → identical.

---

## Risk ledger (retired vs open)
| # | Risk | Status |
|---|---|---|
| R1 | `content=note_meta` MATCH survives DROP COLUMN | ✅ **RETIRED** — experiment: byte-identical |
| R3 | WHEN-gate proxy | ✅ **RESOLVED** — `content_hash` invalid → split FTS sync (note_meta name / note_body body) |
| R6 | SQLite ≥ 3.35 for DROP COLUMN | ✅ libsqlite3-sys 0.28 → SQLite 3.45 |
| R8 | Disk free space (dual-write + VACUUM temp) | ✅ E: 7.9 TB free |
| — | Trigger ordering (note_body before note_meta; repoint before drop) | ✅ proven in experiment |
| R4 | **The 123 MB `Spirochete.md` note** | ⚠ **NEEDS BOSS** — see below |
| R5 | VACUUM lock/time on 1.7 GB | open — guarded post-paint one-shot, retriable; measure on copy |

## ⚠ Spirochete.md — needs a Boss decision
`E:\Cognitive Knowledge\Science\libraries\Biology\Cell Biology\Spirochete.md` is **123 MB but only ~64 lines** (normal Wikipedia frontmatter on top of one/two multi-MB lines). It is **30% of all body_text**. A real article is ~80 KB. Almost certainly a runaway paste / embedded data-blob / import bug — likely corruption worth its own investigation (and a 123 MB note may be straining the editor too). Decision: investigate/repair/replace this note separately (and possibly the other large Biology/Cell Biology notes: Archaea 6.7 MB, Brown algae 4.7 MB)? BL proceeds regardless (dual-write byte-caps it), but cleaning it would reclaim 123 MB immediately and de-risk the editor.
