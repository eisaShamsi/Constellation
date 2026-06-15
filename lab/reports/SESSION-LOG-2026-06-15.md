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

### Open / next
- §A′.2 — reconcile: remove the 14 stale `note_meta` rows (delete cascades via triggers to FTS/sky), guarded so an offline drive can't trigger mass-deletion. Boss-test: phantoms gone.
- Then Phase BL (body_text de-bloat + VACUUM), Phase B (persisted tree), Phase D (lazy/virtualized).
