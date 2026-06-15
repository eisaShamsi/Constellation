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
- Findings: `lab/reports/IMPORTER-TABLE-EXPLOSION-FINDINGS-2026-06-15.md`. The bug is in the **trial-universe generator** (`lab/trial-universe/generator/html-to-md.mjs::renderTable`), NOT the shipped app (which has no Wikipedia importer). Combinatorial `<tr>`-descendant + nested-table re-render on `{{clade}}` trees. **19 notes > 300 KB = 146 MB** (Spirochete 122.9 MB = 84%). Boss chose: fix it + clean the 19 notes BEFORE §BL.2 (so the eventual VACUUM reclaims the maximum).
- **Stage 1 — converter fixed + proven (commit `2a210309`).** `renderTable`: select only this table's own rows (`closest('table') === self`), flatten nested `<table>` in a cell to text (`renderCell`), `MAX_TABLE_ROWS=2000` cap; `build-note.mjs` `MAX_BODY_BYTES=600k` backstop. Live re-fetch: **Spirochete 122.9 MB → 25.6 KB** (seps 1,078,846 → 113), Archaea 6.7 MB → 151 KB, Brown algae 4.7 MB → 52 KB; prose intact.
- **Stage 2 — the 19 bloated notes cleaned on disk (backup-first).** Byte-based detection (a `|`-block > 80 KB = explosion — catches BOTH shapes: many rows AND the few-enormous-lines shape, e.g. Brown algae line 92 = 2.4 MB with 21,349 `| --- |` in one row); excise the garbage block (→ `> [!warning] Table omitted` marker), keep frontmatter + prose + legit small tables. **19 notes, 146.8 MB → 1.68 MB (~145 MB reclaimed).** Legit-large spared (Fourier transform, Permian-Triassic, Philippines, Reading, Women in Islam). Originals backed up to `lab/backups/importer-cleanup-2026-06-15/` (gitignored; Spirochete.md.bak = 128 MB). Cleaned-note content verified valid (frontmatter/cid_cn/TL;DR/prose/typed-links/footer intact).
- **DB reflects on next §BL.1-binary launch:** boot `cache_reconcile → reconcile_filesystem` reindexes the 19 (fresh mtime, `force:false` mtime cache) → note_meta.body_text shrinks 404 → ~258 MB + note_body dual-write gets clean bodies. The freed pages reclaim at §BL.3 VACUUM. **To verify: Eisa launches the §BL.1 binary; Claude re-checks the DB.**

### §BL.1 live boot verification — backfill works, but a BOOT-CONTENTION BLOCKER surfaced
Eisa launched the §BL.1 binary (mtime 14:25, note_body code grep-confirmed). Verified the live DB:
- **§BL.1 backfill works:** `note_body` = 7,653 rows, 0 missing, 0 divergent, sentinel stamped. ✓
- **BUT the cleaned files were NOT re-indexed:** `note_meta` still 404 MB; Spirochete still 122.9 MB with `modified=2026-05-30` (disk = 19.2 KB, mtime 15:48 today). So note_body copied the *pre-cleanup* bodies. The boot's `reconcile_filesystem` (which would reindex changed files by mtime) **did not run / left no log trace**.
- **Root blocker — the thundering-herd `init_db` race (the deferred §B1 item), now confirmed live in diagnostics.log:** `init_db` ran 8+ times concurrently (16:26:00–16:26:42); `mig003_step3_soft_rebackfill` ran **twice (26 s + 41 s)**; `[note_body_backfill] FAILED: database is locked` then retried. The contention is why the backfill locked and (likely) why reconcile didn't complete. NOTE: a prior attempt to fix this (hold `state.db` lock across `init_db`) was REVERTED as too invasive — the proper fix is `std::sync::Once` / a dedicated init mutex (careful design, = §B1).
- **Secondary finding:** the 26–41 s mig003 sweep fires every boot because **one note has a persistently-empty `cid_cn`** (`repaired note_meta=1` each pass). Worth isolating — it's a recurring boot cost and likely a contributor to the original "0 notes" flicker.
- The FTS search-identity check can't be run from plain Python (the `constellation` custom FTS5 tokenizer is Rust-only) — defer search verification to the in-app test at §BL.2.

### Open / next (priority order revised by the live finding)
1. **Fix the thundering-herd `init_db` race (§B1, brought forward — it's hurting every boot + blocked the cleanup reflection + explains the "0 notes" flicker).** `std::sync::Once`/init-mutex so `init_db` runs exactly once; ensure `reconcile_filesystem` reindexes the 19 cleaned files (mtime-newer) on a clean boot → note_meta 404→~258 MB + note_body re-synced clean.
2. Isolate the 1 note with empty `cid_cn` causing the recurring 26–41 s mig003 sweep.
3. §BL.2 (flip reads + FTS triggers to note_body; search-identity Boss-test gate) → §BL.3 (drop body_text + VACUUM — now reclaims the freelist + the ~145 MB freed by the cleanup, once reindexed).
4. Phase B (persisted tree_node + folder_stats) → Phase D (lazy/virtualized + §D1b stable-skeleton expand).
- The 19 notes are clean on disk + backed up regardless; the reflection just awaits a clean re-index.

---

## §B1 — The startup-race fix (thundering-herd `init_db`) — diagnosis + plan  [in progress, next-session pickup]

**Working on:** the **startup-race fix** — the thundering-herd `init_db` boot race (`ensure_search_db_ready`, `src-tauri/src/search.rs:6645`), brought forward as MIG-078 §B1.

### Ground-truth reproduction (Reproduce-First) — read off the LIVE DB + diagnostics.log
Active universe "Eisa Cognitive Knowledge" (`E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db`, 1.83 GB; app not running, WAL 0 B → read-only Python queries safe).

1. **Thundering herd confirmed in `diagnostics.log`:** many concurrent `init_db` blocks; `mig003_step3_soft_rebackfill` ran repeatedly (28s, 24s, 26s, **41s**) on the same boot; `[note_body_backfill] FAILED (non-fatal): copy range: database is locked` then later `completed: 253 bodies copied`. Each `init_db` re-runs the full mig003 sweep.

2. **The recurring 26–41 s sweep — ROOT CAUSE FOUND (a frontmatter-parser bug, NOT a one-note quirk).**
   - Exactly **1** `note_meta` row has empty `cid_cn`: **`أولي (كائن)`** at `E:\Cognitive Knowledge\العالم العربي\libraries\جغرافيا\شبه الجزيرة العربية\أولي (كائن).md`.
   - Its `properties_json` = `{"title":"أولي (كائن)"}` (only title), `tags_json` = `["\"cite-q"]` (a garbage fragment), and `body_text` **starts with** `cites-a-work-with-an-erratum"\n  - "مجموعات-...` — i.e. the rest of the YAML leaked into the body.
   - The file on disk DOES have `cid_cn: 20260414T092241Z_NOTE_B85A`. The note's **first tag is `"cite-q---cites-a-work-with-an-erratum"`** (importer-generated). The indexer's `parse_frontmatter` (search.rs:3323) + `body_after_frontmatter` (3309) detect the closing fence with the naive `content[3..].find("---")` — which matches the `---` **inside that quoted tag value**, truncating the frontmatter before the `cid_cn:` line. So `index_note` (4018) sets `cid_cn = properties.get("cid_cn") = ''`.
   - The mig003_step3 `EXISTS(empty cid_cn)` pre-check (added v2.80) therefore returns true **every boot** → the full sweep (5 UPDATEs scanning note_links 232k+ rows, sky_nodes, aliases, embeddings) runs = the 26–41 s. The `UPDATE … COALESCE(json_extract(properties_json,'$.cid_cn'), cid_cn)` can never fix it (props has no cid_cn → stays `''`, but `changes()`=1 → "repaired note_meta=1" forever).
   - The proven-correct reference already exists: `extract_frontmatter_cid_cn` (1538) uses line-anchored `find("\n---")`.

3. **Handover premise CORRECTED — boot is WALK-FREE (MIG-067).** `+layout.svelte:2167-2173` calls only `cache_mark_search_ready` (no walk); `cache_reconcile`/`reconcile_filesystem` fire ONLY via the file watcher, Settings → Rebuild Index, `add_library`, or the BUG-022 empty-index auto-recover. diagnostics.log shows **no `reconcile_filesystem` trace** on boot (only the §A′.2 `[reconcile] removed 14 stale rows`). ⇒ **Nothing reindexes the 19 cleaned files NOR the empty-cid_cn note on a normal boot.** The note's disk mtime == stored `modified` (both 2026-05-30 22:15:50), so even a walk's `force:false` mtime gate would skip it. The empty-cid_cn note must be **actively reindexed** by the fix; the 19 cleaned files reflect via an explicit Rebuild Index (or a controlled reconcile at the §BL.2/§BL.3 gate).

### Plan (two independent commits)
- **Commit 1 — init mutex (the race).** Add `init_lock: Mutex<()>` to `SearchState`; restructure `ensure_search_db_ready`: fast-path `db.is_some()` → take `init_lock` → **double-check** `db.is_some()` → run `init_db` (NOT holding `state.db`) → store → schedules → federation spawn, all under `init_lock`. Per-state Mutex (not process-global `Once`) so it re-runs on universe switch (`invalidate_search_state` clears `state.db`). NOT holding `state.db` across `init_db` — that 2026-06-14 "Fix 3" was reverted. Deadlock-checked: nothing under the lock calls `ensure_search_db_ready` synchronously (`init_five_acts_system_notes` = fs only; schedules spawn threads + run after `state.db` stored → fast-path).
- **Commit 2 — the parser root-cause + the empty-cid_cn reindex.** (a) `parse_frontmatter` + `body_after_frontmatter` switch the closing-fence search to line-anchored `\n---` (mirrors `extract_frontmatter_cid_cn`); +unit test for the `cite-q---cites` shape. (b) `mig003_step3_soft_rebackfill`: when empty-`cid_cn` rows exist, **force-reindex** them via `index_note(force:true)` (re-reads file with the fixed parser → correct cid_cn + body + props + tags; bounded to the stale set; tokenizer+note_body+FTS triggers all created before the call at 3209). Then EXISTS→false next boot → sweep gone.

### Predecessor → Replacement (Predecessor Lookup Rule)
- **`SearchState` (search.rs:384)** — add `init_lock` field + init in `new()` (436). Same place; additive.
- **`ensure_search_db_ready` (search.rs:6645)** — restructure in place; behavior preserved (init still runs once, schedules+federation still spawned once). No call-site changes (45+ callers unaffected — signature identical).
- **`parse_frontmatter` (3317) / `body_after_frontmatter` (3307)** — fix closing-fence detection in place; callers unchanged.
- **`mig003_step3_soft_rebackfill` (1657)** — repair logic now reindexes from file instead of the `json_extract` UPDATE that could never reach the unparsed cid_cn; same call site (init_db:3209).

### Done =
- One `init_db` per boot; no doubled mig003 sweep; no "database is locked"; `أولي (كائن)` reindexed → `cid_cn=20260414T092241Z_NOTE_B85A`, EXISTS→false, sweep stops firing; no "0 notes" flicker; OrgChart still <2 s. (The 19 cleaned files reflect via an explicit Rebuild Index — surfaced as a handover correction.)

### SHIPPED (3 commits, all on main; release binary rebuilt 2026-06-15)
- **`fe01e3db`** — init mutex. `SearchState.init_lock: Mutex<()>` + double-checked locking in `ensure_search_db_ready` (fast-path → init_lock → re-check → init body under lock). Poison-tolerant (`into_inner`). NOT holding `state.db` across `init_db`.
- **`44a230fd`** — parser root cause + index-friendly sweep. `parse_frontmatter`/`body_after_frontmatter` → line-anchored `\n---` (EOF bounds-guarded). `mig003_step3` probe `cid_cn=''` (index, not the `IS NULL OR` full SCAN) + force-reindex empty rows via `index_note` + path-scoped dependent updates. +2 parser tests.
- **`a7590837`** — Phase-4 audit hardening (see below). Shared `split_frontmatter` helper (one source of truth, fixes the `trim_start` divergence); universe-switch generation re-check before storing the conn; `index_note` Result captured + cleaner diag log; corrected the stale FTS-prewarm doc comment; fixed the pre-existing §BL.1 test gap (`tests_pj060_index_gate` lacked `note_body`). +2 tests.

### Phase-4 adversarial audit (4-lens workflow `wf_56f7f310-41d`) — VERDICT PASS_WITH_NOTES, no P0
Lenses: invariants / drift / migration-paths / adversary → synthesis (independently re-verified each P1/cross-cutting claim). No corruption / panic / boot-hang / sweep-never-stops. UTF-8 boundary safety confirmed (fences are ASCII; slices land on char boundaries; `.get()` guards EOF). Findings addressed in `a7590837` (P2-1/2-2/2-3 + P3 observability). **Remaining deferred (logged, NOT blocking):**
- P3 — a genuinely-cid_cn-less note (file lacks `cid_cn:`) stays `still_empty` and is re-reindexed each boot — bounded to ≤1 row by the UNIQUE index (can't be the full-scan sweep); not present in this universe (the one note HAS a cid_cn). Convergent fix (inject via `canonical::ensure_cid_cn`) deferred — avoids adding a file-write to boot init for a non-occurring case.
- P3 — `ensure_search_db_ready`'s two `state.db.lock()` are still `map_err` (not poison-tolerant) like the rest of the file; pre-existing, optional.
- Follow-up PJ — **active-index FTS5 optimize**: the active universe has NO routine FTS5 segment maintenance (the old "boot write merges segments" belief was always false — `note_meta_au` gates on name/body_text, not cid_cn). If active search ever fragments, add a background segid-gated `optimize` mirroring `federation_prewarm`. (Couldn't measure segid from Python — FTS5 shadow column; needs the Rust conn.)
- Follow-up PJ — the 4 naive `find("---")` copies in `libraries.rs` (2424/3515/4251/4272), preview/tag surfaces only (display, not corruption); when fixed, route them through `split_frontmatter` too.

### Verification status
- **Static:** `cargo check --lib` clean; `cargo test --lib search::` **65/65** (incl. 4 new parser regression tests). Query plans confirmed `cid_cn=''` rides `idx_note_meta_cid_cn` vs `IS NULL OR` → full `SCAN`. cid_cn `20260414T092241Z_NOTE_B85A` is unique (no reindex collision).
- **Pending (runtime, Boss launch + Claude DB re-check):** one `init_db`; `أولي (كائن)`.cid_cn populated; empty-cid_cn count 0; `diagnostics.log` shows `stale=1 reindexed=1 … still_empty=0`, no doubled sweep, no "database is locked"; body_text no longer leaks YAML; no/short "0 notes" flicker; OrgChart <2 s.

---

## The bring-up pivot + concept-paper governance (afternoon)

### From §B1 to the real boot bottleneck (Reproduce-First)
Boss-tested the §B1 binary: data fixes confirmed (empty-cid_cn 0; `أولي` self-healed; no thundering herd; no lock errors), BUT boot still ~70s/3-freezes. Read the trace, not inference: `init_db` ran TWICE (17:50:02 + 17:50:36). Cause: only `invalidate_search_state` resets `state.db`, only `set_active_universe` calls it, and the boot calls it redundantly (main window `+layout.svelte:2515` + second screen `SecondScreenPage.svelte:923`). Then `boot-perf.latest.json` gave the real number: paint 0.9s / hydrated 1.7s PASS, but **`graph_ready_ms`=32,519** — `cache_boot_snapshot_graph` recomputes the graph every boot (`read_links` 11.6s over **233,995** links [measured, all `status='active'`, full SCAN — NOT the "656k" an old doc assumed], `read_tags` 5.6s, +11.25s queue). A Rule-8 (Write-Time Derivation) violation.

### Boss correction → MIG-079 Architect (commit the rules)
Boss: *"a solid fix at the root, not a patch."* Then: *"commit to the rules for auditing and research."* Ran the 8-agent Architect workflow (`wf_86695058-176`) + WA#5 web research (incremental view maintenance, Obsidian/Logseq/Roam backlink persistence, VS Code/Tauri backend-owns-state). Verdict: extend the in-house write-time trigger pattern (notes_fts/sky_nodes/outgoing aggregates), don't invent. Doc: `docs/MIG-079-Architect-Boot-WTD-Graph-Snapshot.md`. Boss decisions: links **defer off boot**; tags **maintained in the indexer (Rust)**; **activation fix first**.

### Boss method: minimal-editor bring-up + concept-paper-per-function
Boss directive: disable all functions (via FLAGS not scissors), keep only the editor (the gate), measure the baseline, re-enable one at a time, fixing lag/wiring per step — and **every function gets a concept paper defining its purpose, acting as the checklist**. Plus: each paper covers the element's **right-click** and **multilingual** conformance; and a **core Constellation concept paper** sits above all.
- Foundation (commit `c4106927`): `docs/concept-papers/00-Constellation-Core-Concept-Paper.md` (core), `00-MASTER-Bring-Up-Charter-and-Checklist.md` (charter + 11-section template incl. Right-click §5 + Multilingual §6 + inventory + bring-up order + safeBootMode design), `01-Note-Editor.md` (the gate).
- 31 per-function papers (commit `e9f809bc`, workflow `wf_9d8697c5-2f6`) — each grounded in code (component + context menu + i18n). **The pass became an app-wide audit** → 00-MASTER §7 Debt Register: Rule-8 recompute is SYSTEMIC (~14 functions); MIG-077 right-click incomplete (4 hand-rolled + ~19 missing); hardcoded English ~17 surfaces; missing feature gates (incl. Search Hub has none). **Confirmed defects:** Calendar day-click always opens TODAY (`get_daily_note_path` ignores the date); Tasks `toggle_task` bypasses the Editor reindex gate; Second Screen `:923` re-activation; Review-Pulse `record_note_visit` dead; 6 no-op Command-Palette stubs.

### Boss rulings + MIG-079 §A shipped
- Boss: **start the bring-up build**; **reframe Tasks** (→ "open epistemic loops" serving Tension→Synthesis→Conviction; kept, not removed — paper updated).
- **MIG-079 §A SHIPPED (commit `256d7c5f`)** — single-owner activation: (A.1) idempotency guard at the top of `set_active_universe` (requested path == active → no-op, no invalidate/re-init); (A.2) second screen display-only (removed `setActiveUniverse:923` + unused import). Plan: `docs/MIG-079-Plan.md`. `cargo check --lib` clean. **Pending: Boss relaunch (2nd screen open) → diagnostics.log shows `init_db` ONCE.** Building (npm+cargo).
- Next: §B safeBootMode + missing gates → measure editor baseline → §C boot-graph WTD (tag_counts in indexer + defer links) → §D phase-by-phase bring-up (write-time cure + right-click + i18n per concept paper).

### MIG-079 §B.1 — safeBootMode switch SHIPPED + editor baseline measured
- `appSettings.safeBootMode` (top-level, default false) + 4 satellite-IPC gates in `+layout.svelte` (graph snapshot, sky snapshot, federation:ready re-invoke, listFiveActsNotes + loadFederationWarnings). svelte-check 0 errors. Commit shipped; binary rebuilt (21:33) after the app was closed (the running .exe had locked cargo's output — Access-denied; resolved by closing).
- **Editor baseline measured (minimal mode, live universe):** `paint 452ms · hydrated 588ms · graph_ready 603ms` vs full `941 / 1671 / 32,519`. graph/sky/federation/Five-Acts IPCs confirmed NOT fired (ipc_arrival_log). **The editor spine boots in ~0.6s; the whole ~32s is satellites.** Recorded as the bring-up regression reference (00-MASTER §6). Measurement done by flipping safeBootMode in settings.json (backed up + restored to full mode after).
- **Next: §C** — remove the 30s for real: `tag_counts` maintained in `index_note` (Rust) + defer the 234k-link `read_links` off the graphReady critical path, so FULL boot approaches the 0.6s baseline. Then §D phase-by-phase re-enable.

### MIG-079 §A + §B Boss-validated; the boot-perf HISTORY TOOL; cold measurements; a guessing correction
- **§A validated:** Boss relaunched (2nd screen) → diagnostics.log shows `init_db` ONCE (was twice at 17:50). "Way better, no freezes." (PCS done mid-session: orientation v2.83 + MoCh-1400 + handover, pushed `9117a3a3`.)
- **§B.1 validated** (above) — but Boss observed a **cold** launch ~13 s w/ a freeze vs warm ~instant. My boot-perf read had been the WARM boot (latest.json keeps only the last). **I then GUESSED the 13 s was "the cold 1.83 GB DB-file read" — and stated it as a mechanism. Eisa corrected me (standing order: don't state a cause without a measurement). The guess was WRONG.**
- **The boot-perf HISTORY TOOL (commit `c8f7ff9a`):** `write_boot_perf_report` (cache.rs) now ALSO appends each report to `.constellation/boot-perf.history.jsonl` (append-only, NEVER overwritten — captures every boot, cold+warm, many per session); `buildBootPerfReport` adds `boot_id`/`phase`/`safe_boot_mode`; reader `lab/boot-perf/read-boot-history.py`. Built to MEASURE instead of infer.
- **MEASURED (3 boots, cold via PC restart):** cold-minimal **1.7 s** (DB read only 0.65 s); warm-full 5.6 s; **cold-full 33 s** = graph reads (read_links **11.3 s** / read_tags **5.7 s** / queue **12 s**). **Conclusion: the editor spine is fast cold+warm; the ENTIRE boot cost is the graph reads → §C is the whole fix, cold+warm.** The "13 s" did not reproduce (pre-tool binary); not chased.
- **§C.1 VERIFIED + designed, NOT built** (checkpointed — write-path change, not rushed at session tail): `index_note`:4266 is the SOLE `tags_json` writer (grep-confirmed) → Rust ±delta is complete; delete decrement in `reindex_delete_note`:7217. Design = `tag_counts(tag,n)` + ±delta gated on `schema_versions.tag_counts` + non-blocking own-conn backfill + read-flip-with-live-fallback + reconcile recompute. MUST DO before save-path: live-DB rehearsal (counts == read_tags), adversarial audit of the backfill-coexistence race, delta unit test, Editor-Surface Gate. Pickup: HANDOVER addendum.

---

## MIG-079 §C.1 — write-time `tag_counts` BUILT + proven (next-session pickup, executed)

**Working on:** the **boot-graph tag aggregate** — `cache::read_tags` (the 5.6 s full-`tags_json` scan on every boot, CLAUDE.md Rule 8 violation), replaced by a persisted `tag_counts(tag, n)` summary maintained write-time.

### Ground-truth data shape FIRST (Reproduce-First / Walk-Through-Writes)
Read-only analysis of the live 7,653-note universe (`lab/tag-counts/analyze-live-tags.py`, dumps the serde target to `live-read-tags-target.json`):
- **All 7,653 `tags_json` rows are clean JSON string-arrays.** 0 NULL, 0 malformed, 0 non-array, 0 non-string-element, 0 empty-string, 0 duplicate-within-note. (Structural: `index_note` always writes `serde_json::to_string(&Vec<String>)`.)
- **`serde` (the `read_tags` path) and `json_each` aggregates are byte-identical** here: distinct **19,542**, occurrences **36,193**, ZERO diffs.
- **Raw `SELECT tags_json FROM note_meta` alone = 5.77 s** — the cost is SQLite's wide-row traversal (inline `body_text`), NOT JSON parsing. The summary table (~19.5k rows) reads in ms.

### What shipped (one proven unit)
- **`src-tauri/src/tag_counts.rs`** (new module; `mod tag_counts;` in lib.rs). Holds: `recompute_all_in` (the shared `json_each` aggregate SQL — single source of truth for backfill + reconcile), `tag_multiset` (the exact `read_tags` semantics: per-occurrence count, skip empty, malformed→empty), `apply_delta` (the ±delta with `old==new` fast-path + zero-prune), `is_stamped`/`is_stamped_in_schema`, and `maybe_schedule`/`run` (the atomic backfill).
- **Schema:** `CREATE TABLE tag_counts(tag PRIMARY KEY, n)` in `init_db` (inert until stamped).
- **Write path:** `index_note` captures old `tags_json` before its UPSERT and calls `apply_delta` after — INSIDE the existing `BEGIN IMMEDIATE` (atomic with the note_meta write), gated on the stamp. `reindex_delete_note` reads old tags before the row delete and applies `old → []` (best-effort).
- **Read flip:** `cache::read_tags_in_schema` reads `{schema}.tag_counts WHERE n>0` when that schema is stamped, **else falls back to the legacy live scan** (zero-risk; an un-upgraded attached cUniverse just scans). Any read error also falls through.
- **Backfill:** scheduled in `ensure_search_db_ready` (after `note_body_backfill`). Builds the whole table from `note_meta` in a SINGLE `json_each` aggregate inside ONE `IMMEDIATE` transaction on a **dedicated connection** (the proven reconcile `walk_conn` pattern), then stamps, then commits.
- **Self-heal:** `reconcile_filesystem` rebuilds `tag_counts` authoritatively (one txn, when stamped) after its walk + the outgoing-link recompute.

### Design refinement vs the "locked" handover (surfaced for Eisa)
Handover said "non-blocking own-connection backfill modeled on `note_body_backfill` (batched)". I made the backfill a **single atomic aggregate** instead of a batch loop. Reason: `tag_counts` is an *additive* aggregate, so a batched build racing live ±deltas is the exact double-count race the handover flagged as audit item (b). Atomicity **eliminates** the race instead of bounding it, and tags have no giant-row problem (unlike `note_body`'s 128 MB outlier) so one statement is cheap. Same module *shape* (own-connection background thread, gated on `schema_versions`, completeness implicit in the single aggregate, non-fatal, read-flip-with-fallback, reconcile recompute).

### Adversarial audit of the coexistence race (mandatory item b) — race ELIMINATED, proof by cases
SQLite single-writer + `busy_timeout` serialize a concurrent save **S** and the backfill **B**:
- **S commits before B:** when S ran, table not stamped → S's delta gated off → S writes only note_meta. B then aggregates the current note_meta (sees S) → correct → stamp. ✓
- **S commits after B:** B built+stamped first. S sees stamped → reads old tags (the value B counted) inside its own txn → applies `new−old` → final = B's count + (new−old) = correct. ✓
- **Contention:** IMMEDIATE + busy_timeout(30 s) force one of the two above; a writer can't observe a half-built B (B holds the write lock to COMMIT; WAL readers see only committed state). ✓
- **The one wart (~6 s write-lock hold):** one-time (stamp-gated), background (post-paint), readers unaffected (WAL), a save in the window waits ≤30 s then succeeds (no loss). Measured aggregate = **5.79 s**.
- **Also audited:** reconcile self-heal is DELETE+rebuild (not additive → can't double-count); first-boot-after-ship falls back to scan then stamps (no regression, matches note_body rollout); cold/empty stamps an empty table maintained by deltas; federation checks each schema's own stamp; delete-path decrement is best-effort (reconcile is the net); a delta error in `index_note` rolls back the whole index (fail-safe).

### Verification done (Claude-side)
- **Unit tests (mandatory item c):** 4 new in `tag_counts.rs` — multiset semantics, add/remove/rename+prune, no-op-on-unchanged, and **backfill==sum-of-deltas==live-aggregate + idempotent**. `cargo test --lib` **930 passed / 0 failed** (incl. the 65 search tests — save path intact).
- **Live-DB rehearsal (mandatory item a):** `tag_counts::tests::rehearse_against_live_copy` runs the REAL `recompute_all_in` against a copy of the 1.74 GB live DB and asserts equality to `live-read-tags-target.json`: **PASS byte-for-byte** (19,542 distinct / 36,193 occ, 0 diffs; aggregate 5.79 s).
- **Editor-Surface Gate (mandatory item d):** tags are a *derived* view — the delta never touches note content/body/display, so the content-integrity class is structurally untouched; covered by the save-path tests + the Boss test tutorial (NotePane save, Focus round-trip, tab switch, rename probe-pair — asserting tags display correct AND body intact).

### Runtime validation — Boss-tested this session (Steps 1 & 2 PASS)
- **Step 1 PASS — counts correct + backfill ran.** Boss opened right sidebar → Tags → "All tags": header total reads **19,542** (exact match to §C.1's computed/rehearsed distinct count); per-tag counts correct (e.g. `1423-births`=3). `diagnostics.log`: `[tag_counts_backfill] completed: 19542 distinct tags` — the atomic backfill ran AND stamped on the first launch (stamp is in the same txn as the build, so "completed" ⟹ stamped).
- **Step 2 PASS — measured on the REAL instrument** (boot-perf history tool, not a side-channel): `read_tags` **5,658 ms → 4 ms / 36 ms**.
  | entry (local, UTC+4) | which | read_tags | read_links | graph_ready |
  |---|---|---|---|---|
  | 22:07 | cold, **pre-§C.1** | **5,658 ms** | 11,336 | 33,100 |
  | 23:29 | Step 1 (§C.1) | **4 ms** | 11,399 | 28,553 |
  | 23:37 | Step 2 (§C.1) | **36 ms** | 9,494 | 20,264 |
  Step 1's 4 ms on a *first* launch (a fallback scan would be cold ≈ 5,658 ms) confirms the table-read path is live — the backfill stamped ~2 s into that boot (diagnostics timestamp matches entry [4]).
- **Correction logged (standing order: measure, don't guess).** I first read the history tool's UTC (`Z`) timestamps as "old 19:37 entries" and wrongly side-noted "the logger didn't capture your launches." The timestamps are UTC; the machine is UTC+4, so `19:37Z` = `23:37` local = the Step-2 launch. The logger captured both launches correctly. Boss caught the loose end and required reading the actual log instead of trusting my copy-measurement — exactly right.

### Remaining for §C.1 (optional; Boss-directed)
- Steps 3–4 (live add/remove a tag → count maintained) + Step 5 (content-integrity gate). Step 3 next.
- A guaranteed-COLD §C.1 boot would lock the cold before/after (entry [5] was likely warm); optional.
- Then **§C.2** (defer the 234k `read_links` — the big 11 s) + **§C.3** (covering index).
