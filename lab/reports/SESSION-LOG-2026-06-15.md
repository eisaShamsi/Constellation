# Session Log — 2026-06-15

## MIG-078 — Persist the Universe Map / OrgChart Tree (Write-Time Derivation)

### Context / correction
Carrying over from 2026-06-14's perf work: the OrgChart fullscreen open had regressed from ~26 s to **2+ minutes** after a string of symptom patches. Eisa called **"Stop patching."** Applied the Stop-On-Correction Rule: stopped all edits, listed the uncommitted changes since last approval, and reframed. Eisa directed: **"Fix it properly. Examine the structure, cross-check it against the state-of-the-art design for similar functions, then provide recommendations."**

### Phase 1 — Architect (workflow)
Ran an 8-agent background workflow (`wf_67609267-f61`): 4 code-examination agents (dataflow, write-path, boot-path, consumers) + 4 state-of-the-art research agents (PKM caching, SQLite trees, incremental view maintenance, editor file-trees) → 1 synthesis. (Research+synthesis 529'd on first run; resumed from journal, examine results cached.)
- Architect doc: `docs/MIG-078-Architect-OrgChart-Map-Tree.md`.
- **Root cause:** the Map/OrgChart tree is recomputed at read time from an eager whole-Universe **disk walk** (`build_tree` → `fs::read_dir` on every folder; `collect_notes_recursive` content-reads on the fallback), instead of read from an already-current persisted structure. The symptom patches only reduced the constant factor.
- **In-house exemplar found:** `sky_nodes` is already maintained at write time by `note_meta_sky_ai/ad/au` triggers — Sky View already does the Write-Time Derivation the Map needs.

### Phase 2 — Plan (approved)
Plan doc: `docs/MIG-078-Plan.md`. Eisa decisions:
- **Scope = A′ → B → D (everything).** A′ in-memory tree (kill the defect) → B persisted `tree_node` + `folder_stats` maintained at write time → D lazy + virtualized rendering.
- **body_text bloat = folded in** (Phase BL).
- Sub-decisions defaulted: federation per-cUniverse read-federated; toggle OFF→prove→ON→delete; reconcile post-paint + on external change; native-path output.

### Ground-truth investigation (Reproduce-First + Walk-Through-Writes)
The "7,600 notes / 1.7 GB" figures were inherited assumptions — verified against the **actual** DB before writing any code:
- Active universe per `universes.json` = "كون عيسى" (5 notes, 413 KB) — NOT the one that hangs.
- The hanging universe is **"Eisa Cognitive Knowledge"**: **7,667** `note_meta` rows across **19 libraries** whose notes live under `E:\Cognitive Knowledge\…` (external paths). DB = **1.69 GB**.
- `note_meta.path` = **native Windows backslash absolute paths** → A′ can emit them verbatim (matches the disk-walk's output format).
- **Covering index `idx_note_meta_map` IS used** (`SCAN … USING COVERING INDEX`) → the map query already avoids `body_text`.
- **DB bloat breakdown:** `body_text` 404 MB (one note is **122 MB**), **freelist 379 MB** (22 % dead space → VACUUM reclaims), plus large FTS/vocab/links/embeddings tables.
- ⇒ The 2-min hang is the **cold directory walk** under AV (exposed when the regression removed the incidental FS-cache warm-up), **not** the 7,600-file content read the reports assumed.

### §A′.1 — In-memory tree from `note_meta` (no disk walk)  [in build]
- `src-tauri/src/map.rs`: new `build_tree_from_records()` assembles each library's tree purely from indexed note paths (materialized-path), rolling up aggregates in one pass; depth cutoff, child sort, and empty-folder exclusion mirror `build_tree`. `build_library_node` switched to it behind `const MAP_TREE_FROM_INDEX = true` (legacy `build_tree`/`collect_notes_recursive` retained for `constellation_map_data`). No `fs::read_dir`/`read_to_string`/`metadata` on the new path.
- **Engineer-verify (equivalence harness on the real DB, then removed):** `build_tree_from_records` == legacy `build_tree` **byte-identical** for the 4 real libraries tested (Physics 550, Philosophy 548, Architecture 450, جغرافيا 400). ✔
- **Finding surfaced:** the universe-root scratch library mismatched by **+14 phantom notes** — these are **stale `note_meta` rows** (rename/delete test notes whose old rows lingered; files confirmed missing on disk). Total drift = **14 / 7,667 (0.18 %)**, all in the two test-scratch libraries; the 15 real knowledge libraries have **zero** drift. The disk-walk masked this; pure-index exposes it. → Reconcile (plan §B5) brought forward as the immediate next step (§A′.2).
- i18n fix: OrgChart loading text used a non-existent key `layout.loading` (rendered raw — what Eisa saw) → switched both occurrences to `common.loading` (present in all 15 locales).

### §A′.2 — Reconcile note_meta ↔ disk (stale-row self-heal)  [SHIPPED + verified]
- New module `src-tauri/src/reconcile.rs` (`mod reconcile;` in lib.rs; `crate::reconcile::maybe_schedule` called from `ensure_search_db_ready` alongside the other background tasks). Background, after first paint; lock-free existence checks; reuses the canonical `reindex_delete_note` (drops note_links + note_meta, triggers cascade FTS/sky, CTSE term cleanup).
- **Safety (WA #4):** a row is a candidate only if it sits under a *currently-accessible* library root (offline drive → skipped, never purged); hard cap aborts the pass if > max(200, 10 %) of rows look stale (transient sync/mount glitch protection).
- **Verified on the live DB (Claude-side, WA #1):** note_meta 7,667 → **7,653** (exactly 14 removed); all 14 stale paths gone; **0** rows now point at a missing file (no phantoms left, no real note wrongly deleted); real libs intact (Physics 550 / Philosophy 548 / History 549 / Biology 550). Diagnostics log: `[reconcile] removed 14 stale note_meta rows`. Safety brake did not trip.

### Graph-tech assessment (Eisa-requested) — graph-playground.aisloppy.com
- The site is a **layout-stability lab** (not a library): it shows that expanding a node in a DAG/tree should use **stable skeleton + in-slot expansion** so siblings don't move ("sibling drift" = 0), vs Dagre full re-layout (the jump). Cross-checks to the field's *constrained incremental layout* (Cytoscape fcose, ELK incremental, d3-force `.fx/.fy`). No public repo/license — adopt the **principle**, not code.
- **Decision (Boss-approved):** fold "siblings stay put on expand (drift = 0)" into **MIG-078 Phase D as §D1b** (acceptance criterion + a tiny internal drift check). Grounded in our actual code: `OrgChart.toggleFsExpand` (`OrgChart.svelte:463`) currently re-fits + re-centers the whole chart on every expand — the exact anti-pattern. GraphMind + Sky View (pin via `.fx/.fy` once incremental) logged as follow-ons; sunburst excluded (Form-Aligns-To-Purpose).

### Phase BL impact review + de-risking
- 8-agent-equivalent impact workflow (4 dependency maps + Architect) → `docs/MIG-078-Phase-BL-Design.md`. **Load-bearing decision:** keep `notes_fts` as `content=note_meta` (NEVER rebuild — that would churn docids → not byte-identical); keep FTS triggers on note_meta; only change where they READ body. Bridge note_meta↔note_body on `path` (never rowid).
- **Risks retired by live experiment (Python sqlite 3.50):** R1 — after repointing triggers off body_text, `DROP COLUMN` succeeds and MATCH is **byte-identical** before/after; `DROP COLUMN` is blocked while a trigger references the column (so repoint must precede drop — proven); a new note inserts searchable via the note_body-sourced trigger with note_body-written-first ordering. R6 — libsqlite3-sys 0.28 → SQLite 3.45 (DROP COLUMN ✓). R8 — E: 7.9 TB free.
- **Design correction:** the workflow's proposed `content_hash` WHEN-gate is INVALID — `note_meta.content_hash` is never written (only `note_summaries`). Replaced with a split: note_meta triggers gate on name/existence; a new **note_body_au** trigger gates on `OLD.body_text IS NOT NEW.body_text` (uses note_meta.rowid by path; doesn't touch content=).

### §BL.1 — note_body table + dual-write + resumable backfill  [SHIPPED]
- `note_body(path PRIMARY KEY, body_text)` in init_db. `index_note` dual-writes note_body FIRST (establishing §BL.2 trigger ordering now), then the unchanged note_meta UPSERT. `reindex_delete_note` deletes note_body after note_meta. New `note_body_backfill.rs` (mirrors links_backfill): post-paint, resumable cursor, `INSERT…SELECT` (body stays in SQLite — the 128 MB note never enters Rust), completeness-gated sentinel. **No reads/triggers changed → search provably unchanged.**
- **Rehearsed on a consistent backup of the live 1.69 GB DB (WA #1):** backfill copied all **7,653 bodies in 7.5 s**; note_body == note_meta; **0 missing, 0 divergent**; Spirochete.md (128,845,855 bytes) copied byte-for-byte. Live backfill runs on next launch.

### Importer table-explosion — investigated (separate workstream, Boss-approved)
- Findings: `lab/reports/IMPORTER-TABLE-EXPLOSION-FINDINGS-2026-06-15.md`. The bug is in the **trial-universe generator** (`lab/trial-universe/generator/html-to-md.mjs::renderTable`), NOT the shipped app (which has no Wikipedia importer). Combinatorial `<tr>`-descendant + nested-table re-render on `{{clade}}` trees. **19 notes > 300 KB = 146 MB** (Spirochete 122.9 MB = 84%). Recommendation (c): fix renderTable (child-row selector + nested-table policy) + output cap, then re-import the 19 from source_url. **Separate follow-up task; independent of MIG-078.**

### Open / next
- §BL.2 (flip reads + FTS triggers to note_body; the search-identity Boss-test gate) → §BL.3 (drop body_text + VACUUM, after a DB-copy harness pass + ZIP backup). Then Phase B (persisted tree_node + folder_stats), Phase D (lazy/virtualized + §D1b stable-skeleton expand).
- Follow-up: fix the trial-generator renderTable + re-import the 19 bloated notes (Boss to schedule).
