# Handover — next session: the startup-race (thundering-herd `init_db`) fix

**Date prepared:** 2026-06-15 · **Branch:** main (all pushed) · **Active universe:** "Eisa Cognitive Knowledge" (`E:\Constellation Universes\Eisa Cognitive Knowledge`)

Read first: the highest orientation version (`docs/Constellation Orientation & Onboarding v2.82.md`), then this file, then `lab/reports/SESSION-LOG-2026-06-15.md`.

---

## Where we are
MIG-078 (persist the OrgChart/Map tree, scope A′→B→D + body_text de-bloat) is mid-flight. Shipped + committed this session:
- **§A′.1** (`a46278d0`) — OrgChart/Map open **2 min → <2 s** (in-memory tree from note_meta, no disk walk). Boss-validated.
- **§A′.2** (`6daac55b`) — note_meta↔disk reconcile (`reconcile.rs`); removed 14 stale phantom rows. Verified.
- **§BL.1** (`e0a3cf20`) — `note_body` table + dual-write + `note_body_backfill.rs`. Rehearsed on live DB.
- **Importer fix** (`2a210309`) + **19 notes cleaned** (`25c8c9e7`) — trial-generator table-explosion fixed; ~145 MB of garbage excised from the 19 affected notes on disk (backed up in `lab/backups/importer-cleanup-2026-06-15/`, gitignored).
- Docs: `MIG-078-Architect-OrgChart-Map-Tree.md`, `MIG-078-Plan.md`, `MIG-078-Phase-BL-Design.md`, `IMPORTER-TABLE-EXPLOSION-FINDINGS-2026-06-15.md`.

**The binary to run / rebuild from:** `src-tauri/target/release/constellation.exe` (the §BL.1 build was mtime 14:25; rebuild after any change — `npm run build` is NOT needed for Rust-only changes, but verify the binary contains your new code by grepping it).

---

## THE TASK: fix the thundering-herd `init_db` race (the migration's §B1, brought forward)

### The defect (reproduced live, 2026-06-15 — see `diagnostics.log`)
On the §BL.1-binary boot of the big universe, `ensure_search_db_ready` (`src-tauri/src/search.rs` ~6619) let **8+ threads run `init_db` concurrently** (timestamps 16:26:00–16:26:42). Consequences observed:
- `mig003_step3_soft_rebackfill` ran **twice** (26 s + 41 s).
- `[note_body_backfill] FAILED (non-fatal): database is locked` (then a retry completed).
- `reconcile_filesystem` left **no trace** → the 19 cleaned files were **never reindexed** → `note_meta` still 404 MB (Spirochete still 122.9 MB, `modified=2026-05-30`), and `note_body` copied the pre-cleanup bodies.
- This contention is almost certainly the original **"0 notes for ~18 s" boot flicker**.

### Why it happens (the code)
`ensure_search_db_ready` (`search.rs:6619`): checks `state.db.lock().is_some()`, **drops the lock**, then runs `db_path` + `init_db` + reacquires to store the conn. Between the check and the store, N boot callers (`cache_boot_snapshot_core/graph/sky`, federation attach, `cache_reconcile`) all pass the `is_none()` check and each run `init_db`.

### The fix (design — do NOT just hold the lock)
A prior attempt that **held `state.db` across `init_db` was REVERTED as too invasive** (init_db is slow; holding the only query lock across it starves/deadlocks the boot). The proper fix:
- Use a **dedicated init mutex** (separate from `state.db`) or `std::sync::Once`/`OnceCell`, so `init_db` runs **exactly once** while `state.db` stays free for readers. Pattern: take the init lock → re-check `state.db.is_some()` (double-checked) → if still none, run `init_db` → store into `state.db` → release init lock. Concurrent callers block on the init lock, then see the populated `state.db` and return.
- Must survive **universe switch** (the init state resets when `state.db` is cleared — don't make `Once` global-for-process if it must re-run per universe; a per-state mutex/flag is safer than a process-global `Once`).
- **Boot-perf invariant:** the fix must not add to the boot critical path or block first paint. Measure boot before/after on the big universe.

### Secondary (same area, cheap win)
**One note has a persistently-empty `cid_cn`** → the `EXISTS` pre-check in `mig003_step3_soft_rebackfill` (added §A′.2 era) returns true → the full 26–41 s sweep runs **every boot**. Find that note (`SELECT path FROM note_meta WHERE cid_cn IS NULL OR cid_cn=''`), understand why its cid_cn won't persist, fix it so the sweep stops firing.

### Done = 
- Clean boot: `init_db` runs once; no doubled mig003 sweep; no "database is locked"; `reconcile_filesystem` runs.
- The 19 cleaned files get reindexed (mtime-newer) → `note_meta` 404 → **~258 MB**; `note_body` re-syncs to the clean bodies (0 divergent).
- The empty-`cid_cn` sweep no longer fires every boot.
- Boss test: boot shows no/short "0 notes" flicker; OrgChart still <2 s.

### Verification harness (Claude-side, read-only on the live DB)
After the Boss launches the fixed binary: query `note_meta` total body (~258 MB), Spirochete body (~19 KB, `modified`=today), `note_body` 0-divergent, and grep `diagnostics.log` for a single `init_db` + a `reconcile` line + no lock errors. (FTS MATCH can't be run from plain Python — the `constellation` tokenizer is Rust-only; verify search in-app.)

---

## After the startup-race fix — remaining MIG-078 roadmap
- **§BL.2** — flip reads + FTS triggers to `note_body` (keep note_meta.body_text as hot standby); the **search-identity Boss-test gate** (prove FTS/vocab/Index byte-identical on a DB copy, then 5 in-app searches pre/post). Design + the corrected trigger set are in `MIG-078-Phase-BL-Design.md` (note: the `content_hash` gate there is INVALID — use the `note_body_au` trigger).
- **§BL.3** — `DROP COLUMN note_meta.body_text` + `VACUUM` (after a DB-copy harness pass + a tagged/ZIP backup of the ~1.7 GB DB). Now reclaims the freelist + the ~145 MB freed by the cleanup.
- **Phase B** — persisted `tree_node` + `folder_stats` maintained by write-time triggers (the durable end-state; OrgChart open <0.1 s). Schema in `MIG-078-Plan.md`.
- **Phase D** — lazy + virtualized rendering, incl. **§D1b** stable-skeleton expansion (siblings drift = 0).

## Don't forget
- The 19 cleaned notes are on disk + backed up; their DB reflection just awaits the clean reindex (which the startup-race fix enables).
- Editor-Surface Gate + Reproduce-First apply to §BL.2 (it touches the save/FTS path).
