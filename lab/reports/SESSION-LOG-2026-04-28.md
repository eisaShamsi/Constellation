# Session Log — 2026-04-28

---

## §91 — MIG-008 Canonical Naming Cleanup — closed

**Lab plan**: `lab/reports/MIG-008-CANONICAL-NAMING-CLEANUP.md`.

**What shipped**:

- Shared helper `note_display_name(path, content_opt)` in libraries.rs.
- Patched ~14 user-visible label sites across map.rs, inspector360.rs,
  strata.rs, maturity.rs, provenance.rs, review.rs, lenses.rs (dead-
  code-on-arrival per the wiring investigation), tasks.rs, tension.rs,
  libraries.rs::scan_index_words_recursive, trails.rs::find_note_
  recursive, universe.rs::collect_templates_recursive.
- Two correctness fixes (not just label fixes): inspector360.rs:88 now
  matches incoming wikilinks for canonical notes; trails.rs::find_note_
  recursive can now resolve canonical notes by display name (was
  deterministically broken before).

**User testing** (Stages 1, 3, 4a, 4b, 5): all PASS.

- Stage 1 (Constellation Map): tooltips show human titles.
- Stage 3 (Strata + Maturity + Provenance): all three correct.
- Stage 4a (Tasks panel): "Lunch Plan" displayed correctly.
- Stage 4b (Review Pulse): clean.
- Stage 5 (Tension via Health panel): clean.

**Stages skipped**:

- Stage 2 (360° Inspector): component deliberately disabled in
  +layout.svelte:84 — no UI surface to test.
- Stage 4c (Multi-Lens): apply_lens has zero frontend callers (dead
  code, see §92's investigation).

**Phase 4 audit** (3 parallel agents): all PASS.

- 4A Invariants: wikilink resolution unchanged, sky/backlinks/search
  unaffected, snapshot IPCs untouched, helper logic correct, command
  set intact, helpers properly scoped.
- 4B Drift: zero remaining `path.file_stem()` Category-B leaks; zero
  remaining old IPC names; zero remaining `lens.rs` source references.
- 4C Migration path: zero-note Universe safe; schema unchanged; no
  partial-state risk; IPC rename graceful; perf impact negligible.

---

## §92 — MIG-009 Lens-to-Sight Naming Cleanup — closed

**Lab plan**: `lab/reports/MIG-009-LENS-TO-SIGHT-NAMING.md`.

**What shipped**:

- `git mv src-tauri/src/lens.rs src-tauri/src/sight.rs` (file rename
  preserves git history).
- Function rename: `constellation_lens_centrality` →
  `constellation_sight_centrality`; `constellation_lens_tag_edges` →
  `constellation_sight_tag_edges`.
- `lib.rs:16` `mod lens;` → `mod sight;` and `lib.rs:310-311` handler
  list updated atomically.
- `+layout.svelte:3235` invoke renamed atomically with the backend.
- Frontend JS variables (`lensActive`, `toggleLens`, etc., ~60
  occurrences) intentionally NOT renamed — internal naming lag
  accepted.
- `lenses.rs` (plural — CE Phase 9 Multi-Lens) NOT renamed; separate
  concern.

**User testing**: Constellation Sight loads and renders centrality +
community clusters + structural gaps + universe-health header
correctly after rebuild. PASS.

**Audit**: same Phase 4 pass as MIG-008.

**Side-finding catalogued** (memory: `project_lenses_apply_lens_dead_
code.md`): `lenses.rs::apply_lens` is dead code (zero frontend
callers). Settings can still create + save lens definitions but
they're never applied. Decision deferred — delete or re-wire CE
Phase 9 Multi-Lens.

---

## Doc bump

`docs/Constellation Orientation & Onboarding v1.6.md` written
alongside v1.0..v1.5 per SO #6 versions-stack rule. Changelog notes
MIG-008 + MIG-009 closure, dead-code findings, and new backlog items
(CE Phase 9 fate, CE Phase 12 360° Inspector fate, IPC-CONTRACT.md
drift now also missing constellation_sight_* rename).

**Backlog status after this PCS**:

- MIG-007 — Links Settings tab consolidation (queued).
- CE Phase 9 Multi-Lens dead-code: delete or re-wire (decide).
- CE Phase 12 360° Inspector: re-enable or withdraw (decide).
- SecondScreenPage.svelte buildSkyData alias-blind (cosmetic).
- Architectural mystery: cache_boot_snapshot_sky bypass at boot.
- MIG-005 Steps 4-8 (alias-aware tension/inspector360/LinkDashboard).
- Constellation Map perf / search-highlight (separate from canonical-
  naming label which IS now fixed).
- IPC-CONTRACT.md ~5 weeks stale.



---

## §85–§89 — MIG-003 Human-name Filenames (CLOSED 2026-04-28)

**Plan doc**: `lab/reports/MIG-003-HUMAN-NAME-FILENAMES.md`  
**Approach**: Per-library mode (E) with γ Strict cid_cn-as-internal-PK.

**Boss directive that drove the architecture**: *"internally Constellation will use the canonical naming structure cid_cn to ensure practicality and uniqueness... but either the constellation file system/file tree or any OS file system will use the canonical naming system as file name. No, it should handle it as a human naming style."*

### What landed (commits + PCS)

| § | Commit | Step | What it does |
|---|---|---|---|
| §85 | `4f8e3c9` | Step 1 | `cid_cn` column on `note_meta` + UNIQUE index `idx_note_meta_cid_cn` + 3-phase backfill (snapshot/dedup/commit) on 7,610 rows. 38 + 4 collisions auto-resolved. FTS `note_meta_au` trigger gated with WHEN clause to prevent retokenization on cid_cn-only updates. Schema-versions `note_meta` v=1. |
| §86 | `003833a` | Step 2 | `cid_cn` columns on `note_links` (source + target) / `sky_nodes` / `note_aliases` / `note_embeddings` + per-table backfill via JOIN on existing path columns. Schema-versions `dependent_tables_mig003` v=1. |
| §87 | `e54766d` | Step 3 | All 7 INSERT writers stamp cid_cn. `note_meta_sky_ai` trigger updated to copy cid_cn into sky_nodes. Boot-time soft re-backfill (cheap path-keyed cases only). |
| §88 | `a3c365a` | Step 4 | New module `mig003_step4.rs`. Walked 17 libraries; found 19 canonical-named .md files (only the user inbox / Universe Notes folder; the 16 declared libraries already had human filenames from before MIG-003). Per-library transaction; audit log `.constellation/mig003-step4-renames.tsv`. Schema-versions `mig003_step4` v=1. |
| §89 | `18ab164` | Step 5 | Unified `create_note` + `rename_item` flows. Removed canonical-detection special case (dead code post-Step-4). rename_item .md branch: fs::rename + DB cascade + rename-alias stamp + reindex. |

### Steps deliberately skipped

- **Step 6** (promote cid_cn to PRIMARY KEY, drop redundant path columns) — Boss decision; dual-keyed schema is not a defect; rebuild risk not worth cleanliness gain.
- **Frontmatter alias-append for 19 renamed test notes** — Boss confirmed throwaway test notes; saved as wanted-feature memory if future external integration needs it.
- **User Manual + 14 i18n updates** — small user-visible change; separate doc-only commit when convenient.

### Bugs caught + fixed mid-flight (Working Agreement #4 in action)

1. §85 cid_cn corruption from `ensure_cid_cn` return-value misuse (returns full content, not id). Fixed by parsing cid_cn from returned content.
2. §85 schema_versions stamp not persisting — UNIQUE index creation failed due to 38 cid_cn duplicates from a partial prior run. Fixed by scanning ALL plans (Skip + SetCidCn) for collisions.
3. §85 Phase C transaction hung 8+ minutes — note_meta_au trigger had no WHEN clause; fired full-body FTS retokenization on every UPDATE. Fixed with WHEN clause on name/body_text change.
4. §87 boot hang — soft rebackfill `LOWER(name)=LOWER(target_name)` query on 232k x 7610 rows had no supporting index. Caught when user reported app unresponsive after 8 minutes. Fixed by omitting that specific UPDATE; bulk target_cid_cn rebackfill deferred.
5. §88 path bug — db_path is search.db file, treated as directory; libraries.json read failed; falsely stamped as success. Fixed with db_path.parent() + return Err on parse failure.

### What got verified end-to-end

- 7,611 note_meta rows, 0 empty cid_cn, UNIQUE index present.
- 7,611 sky_nodes (matches), 0 empty cid_cn — trigger update working.
- File Explorer across all libraries: zero canonical-format filenames remain. Mixed-script titles (Arabic, English, special chars) all rendered correctly.
- Stage 5 Pass: create_note produces human filenames; rename actually renames file on disk; double-rename preserves wikilinks via cascade walker + alias chain.
- Sky View showed 7,309 displayed vs 7,617 notes — pre-existing filter, not an MIG-003 regression.

### Backups taken before Step 4

- Git tag `milestone/mig-003-step3` pushed.
- Codebase ZIP `E:/Backups/Constellation/Constellation-mig-003-step3-20260428.zip` (96 MB).
- Universe config snapshot `E:/Backups/Constellation/universe-config-mig-003-step3-20260428/`.
- Did NOT zip the 2.6 GB universe per Boss directive.

### Doc updates

- `docs/CANONICAL-FILENAME-ARCHITECTURE.md` — § 0 banner + Post-MIG-003 section explaining inversion.
- `docs/Constellation Orientation & Onboarding v1.7.md` — written as new file alongside v1.0..v1.6 per SO #6. § 6 fully rewritten; § 8 migration table updated.
- Memory: `project_rename_collision_popup_wanted.md` — wanted-feature record for the Stage 5 5.2 UX gap.

### Backlog status after MIG-003 closure

- MIG-005 Steps 4-8 — pending.
- MIG-006 §3 redo + §4-§11 — pending.
- MIG-007 Links Settings tab consolidation — queued.
- CE Phase 9 Multi-Lens dead-code: delete or re-wire — decide.
- CE Phase 12 360° Inspector: re-enable or withdraw — decide.
- Constellation Map perf / search-highlight — pending.
- IPC-CONTRACT.md ~5 weeks stale.
- Rename-collision popup UX (wanted; Stage 5 finding).


---

## Handover to next session — state-of-standing 2026-04-28 end of day

**This is the snapshot a fresh Claude session reads to pick up exactly where this one left off** (per CLAUDE.md Standing Order #5).

### What is verified, shipped, and protected

- MIG-003 fully closed. §85–§90 pushed; tagged `milestone/mig-003-closed`. The Boss-directed architecture inversion (cid_cn = immutable internal id in frontmatter; filename = human-readable, mutable) is live. 7,611 note_meta rows verified, zero canonical filenames remaining on disk.
- Backups intact: `E:/Backups/Constellation/Constellation-mig-003-step3-20260428.zip` (codebase), `E:/Backups/Constellation/universe-config-mig-003-step3-20260428/` (config snapshot), git tag `milestone/mig-003-step3` (pre-Step-4 codebase state).
- Orientation v1.7 written, in main `docs/`. § 6 fully rewritten; § 8 migration table marks MIG-003 closed.

### What is at-risk / in-flight / uncommitted

Nothing. Working tree is clean (the binary at `src-tauri/target/release/constellation.exe` is the live MIG-003-closed build dated 2026-04-28 18:19; nothing in the staging area; everything pushed).

### What is known-broken or pending decision

- **CE Phase 9 Multi-Lens** — `lenses.rs::apply_lens` is dead code, zero frontend callers. Settings can still create + save lens definitions but they are never applied. Boss decision needed: delete or re-wire.
- **CE Phase 12 360° Inspector** — status unclear. Boss decision needed: re-enable or withdraw.
- **Constellation Map** — perf/memory leak, search doesn't highlight matched arcs. Tooltip-shows-canonical-filename was fixed by MIG-003 (notes now have human filenames; the bug it was a symptom of is gone).
- **MIG-006 §3 redo** — Wikilink Rename Cascade open-editor coherence piece was reverted at §116 after BUG-015 corrupted target body content. Plan exists at `lab/reports/MIG-006-WIKILINK-CASCADE.md`; needs careful redesign per lessons from §115/§116.
- **MIG-005 Steps 4–8** — alias-aware tension/inspector360/LinkDashboard. Steps 1–3 shipped (§121–§123). Tutorial paused mid-flight after the fabrication incident.
- **MIG-002 §7–§10** — enrichment persistence remaining steps.
- **Sky View** — displayed 7,309 of 7,617 notes during Stage 2 verification. Pre-existing filter (likely archived/hidden notes). Worth investigating but not a regression.

### Documentation drift acknowledged

- **User Manual + 14 i18n manual translations** — not yet updated for MIG-003. Small visible change ("rename now actually renames the file"; "filenames look like titles"); separate doc-only commit when convenient.
- **IPC-CONTRACT.md** — ~5 weeks stale; missing recent additions (cache_boot_snapshot_sky, sight rename, MIG-003 cascade hooks).
- **Rename-collision popup UX** — wanted feature recorded in `project_rename_collision_popup_wanted.md`. Backend already returns the right error string on collision; frontend needs a modal dialog with Override / Rename / Cancel options.

### Recommended start for next session

Before any code, the Boss should make the two pending decisions:
1. CE Phase 9 Multi-Lens — delete the dead code path or re-wire it?
2. CE Phase 12 360° Inspector — re-enable or withdraw?

Once those are resolved, the highest-impact remaining work is **MIG-006 §3 redo** — the open-editor coherence problem is a real user-visible gap (rename a note in another tab while it is open elsewhere, the open tab does not update). The §115 attempt corrupted body content because a value-prop → CM6 doc sync `$effect` raced with `{#key}` onDestroy; the redo MUST validate against NotePane spec §2.6 (which forbade exactly that pattern) and run Working Agreement #4 architectural-impact review before shipping.

### Lessons recorded today

Five regressions caught + fixed mid-flight before user-visible damage. The pattern across all of them: **a write path was added without walking through what it would do row-by-row on the actual data shape**. Saved as `feedback_walk_through_writes.md` for future sessions. The cost of the walk-through is 5 minutes; the cost of skipping it ranges from an 8-minute hang to silent false-stamping of a migration as complete when it never ran.
