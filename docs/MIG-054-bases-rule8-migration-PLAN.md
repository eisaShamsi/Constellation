---
title: MIG-054 — Bases Phase 1, Rule 8 Migration (Plan)
version: 1.0
date: 2026-05-25
status: Plan doc. Awaiting Eisa's approval before the Build cascade fires.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
predecessor: docs/MIG-054-bases-rule8-migration-ARCHITECT.md (v1.1, approved 2026-05-25)
concept_paper: docs/Constellation-Base-Concept-Paper-v1.4.md (Phase 1 of §12 roadmap)
---

# MIG-054 — Bases Phase 1, Rule 8 Migration (Plan)

## §1. Premise

The Architect doc (v1.1) chose **Option B — query `note_meta` directly** (no new cache table; `note_meta.properties_json` is already write-time-maintained). Eisa locked all 5 Q1-Q5 decisions and approved the Architect doc. This Plan doc phase-decomposes the Architect's §7 step outline into **landable commits, each with an explicit verification clause**.

**Cascade discipline:** Per "Plan Approval = Build Approval" (CLAUDE.md top principal), once Eisa approves this Plan doc, the Build steps §A → §J run as one cascade — no per-step approval needed. Stops only at:
1. User-testable verification clauses (Boss-test at §I).
2. Genuine architectural surprise during build.
3. Plan completion (§J PCS).

Verification clauses are testable artifacts (build passes, test fixtures diff clean, perf gate met) — not "looks good to me."

---

## §2. Step Sequence — Landable Commits

Ten steps. Each lands as one commit with the format `MIG-054 §X — <description>`. §A is the largest; §B-§E are smaller polish commits; §F-§I are verification milestones; §J is PCS.

### §A — `query_base` SQL skeleton (folder + all source types, all 8 filter operators, sort)

**What it does:** Replace `bases.rs::query_base` (line 386) with a SQL implementation against `note_meta`. Build WHERE clause for `folder` and `all` source types (tag deferred to §B). Translate all 8 filter operators to `json_extract` expressions per Architect §5.2. Translate sorts per Architect §5.5. Use the existing `library_paths` parameter to build the library-name → library-path HashMap. Materialize rows into `BaseRow` shape with `name` → `file_name` (trim `.md`) and `library_path` from the map.

**Files touched:**
- `src-tauri/src/bases.rs` — replace `query_base` body; add `build_where_clause()`, `build_order_clause()`, `parse_properties_json()` helpers; keep `scan_folder`/`scan_by_tag` as dead code (deleted in §B once tag source ships SQL-first).

**Verification clause:**
- `cargo build --release` clean (no new warnings beyond pre-existing).
- `cargo test --lib bases::` passes (existing tests, if any).
- Manual exercise: open one folder-source `.base` on Eisa's universe, confirm rows return.
- `query_time_ms` < 50 for that one Base on the 7,600-note universe (intermediate gate; the full perf pass is §G).
- The behavioral-equivalence fixture snapshot is captured **before** §A's swap: with §A on a branch, the OLD `query_base` is run against all 75 fixtures and outputs are saved to `tests/bases/fixtures/snapshots/`. (The post-§A diff happens in §F.)

**Rollback:** revert §A; `query_base` walks the filesystem again. No data loss; only the speed reverts.

### §B — Tag source type via FTS5 MATCH

**What it does:** Implement tag source via SQL — frontmatter tags via `json_extract(tags_json, ...)`, body tags via `notes_fts MATCH '#<tag>'`. Verify FTS5 hashtag tokenization works (Architect §8 risk #3); fall back to `body_text LIKE '%#<tag>%'` if MATCH doesn't tokenize hashtags as expected. Delete the dead `scan_by_tag` function from §A.

**Files touched:**
- `src-tauri/src/bases.rs` — extend `build_where_clause()` for tag source.

**Verification clause:**
- Test database with three notes: one with `tags: [test-tag]` in frontmatter only, one with `#test-tag` in body only, one with both. Tag-source Base for `test-tag` returns all three.
- Multilingual: same test with Arabic tag `#الإمارات` — all three returned.
- Verify against Eisa's universe: a known tagged Base returns the same notes as it did before §A (via the §F harness — the tag fixtures must diff clean here).

**Rollback:** revert §B; tag source returns an error (no fallback to old `scan_by_tag` because §B deleted it — explicit failure mode is acceptable for the short rollback window).

### §C — `columns_detected` via `json_each`

**What it does:** Replace the in-memory dedup of property keys with a SQL query: `SELECT DISTINCT key FROM note_meta, json_each(properties_json) WHERE <same WHERE clause as the main query>`. Returns sorted distinct property keys.

**Files touched:**
- `src-tauri/src/bases.rs` — new `detect_columns_sql()` helper.

**Verification clause:**
- Same `columns_detected` output as before for every §F fixture (behavioral equivalence covers this).
- Single SQL query; no in-memory dedup of property HashMaps.

**Rollback:** revert §C; keep the in-memory dedup from §A.

### §D — Field-name alignment + legacy `"vault"` retirement (Q1 lock)

**What it does:**
1. Rename Rust `BaseSource.selected_vaults` → `BaseSource.selected_libraries`. Serde: `#[serde(rename = "selectedLibraries", alias = "selectedVaults", default)]` for backward-compat read.
2. Update all internal references to the new field name.
3. Legacy `"vault"` source type: `parse_base_file` translates `source.type = "vault"` → `source.type = "all"`; the named vault value flows into `selected_libraries = [the value]`. Save paths never write `"vault"`.
4. New `.base` files written via `create_base`, `create_workspace_base`, `save_base_file`, `save_workspace_base` always serialize with `selectedLibraries` and source types ∈ {`folder`, `tag`, `all`}.

**Files touched:**
- `src-tauri/src/bases.rs` — field rename + alias + legacy translation.
- `src/lib/bases/types.ts` — already uses `selectedLibraries` (verified earlier); no change needed.
- Frontend Svelte components: verify none read `selectedVaults` directly. If any do (grep `selectedVaults` in `src/`), update.

**Verification clause:**
- Round-trip test 1: create a fresh `.base` file via UI → file on disk has `selectedLibraries` key, not `selectedVaults`.
- Round-trip test 2: load an existing `.base` file with `selectedVaults` key → parses correctly; if user re-saves, the file is rewritten with `selectedLibraries`.
- Round-trip test 3: load a legacy `"vault"` source type → behaves as `"all"` + selected_libraries. Saving rewrites as `"all"`.
- §F fixture set includes legacy-shape `.base` files; they diff clean.
- Behavioral equivalence: an old file and a new file with the same filter logic return identical row sets.

**Rollback:** revert §D; both field names remain readable indefinitely via the alias. New saves go back to `selectedVaults`.

### §E — Cell-edit refresh event (Q2 lock)

**What it does:**
1. In `bases.rs::update_note_property`, after the file write, call `update_note_meta_property_immediate(&app, &file_path, &key, &value)` — a new helper that opens the search DB and updates `note_meta.properties_json` directly via `json_set(...)`.
2. Emit Tauri event `bases:note_updated` with payload `{ path: String, changed_keys: Vec<String> }`.
3. Frontend: add a listener in `BaseTableView.svelte`, `BaseCardView.svelte`, `BaseListView.svelte` (and any future view shape). On event for a path in the current Base's row set, re-query the Base OR patch the row in-place using `changed_keys`.
4. Graceful degradation: if `update_note_meta_property_immediate` fails (DB locked, file not yet indexed, etc.), the file write still succeeded; the file-watcher's later re-parse handles the eventual consistency.

**Files touched:**
- `src-tauri/src/bases.rs` — `update_note_property` + new helper.
- `src/lib/components/BaseTableView.svelte`, `BaseCardView.svelte`, `BaseListView.svelte` — add listener.
- `src/lib/bases/types.ts` — new `BasesNoteUpdatedPayload` type.

**Verification clause:**
- Open a folder-source Base view on Eisa's universe. Edit a frontmatter property cell. **The row updates within 100ms** (visible cue: cell value updates without UI flicker; new value persists across tab switch).
- The file on disk has the new value (re-open in editor; confirm).
- Re-query the Base after 5 seconds (post file-watcher debounce). Row still shows the new value (no overwrite).
- Force DB-write failure (simulate via injected error in dev build) — file write still succeeds; row updates after the file-watcher debounce. **No data loss path.**

**Rollback:** revert §E; cell edits go back to the 1.5s window. The file write itself is unchanged.

### §F — Behavioral-equivalence test pass on copy of universe DB (Q3 + Q4 locks)

**What it does:**
1. Copy Eisa's Cognitive Knowledge universe DB to a snapshot path (per LL-025).
2. Build the 75-fixture test suite from Architect §4.E:
   - 15 source-type permutations (folder / tag / all variants)
   - 16 filter-operator × property-type combinations
   - 10 sort fidelity tests (text / numeric / date / multilingual / mixed-script)
   - 10 cross-library `selectedLibraries` queries
   - 10 multilingual frontmatter tests (Arabic / Persian / Hebrew property keys + values + mixed-script in same view)
   - 5 edge cases (empty results, all-empty properties, sparse properties, single-row results, large-result-set perf-stress)
   - 5-10 **representative `.base` files Eisa nominates** from his Cognitive Knowledge universe (the gold-master tier — see §6).
3. Run the new `query_base` against each fixture.
4. Diff against the pre-§A snapshots from §A's verification.
5. Fail loud on any mismatch.

**Files touched:**
- `tests/bases/fixtures/` — 75 `.base` files + 75 snapshot files.
- `tests/bases/equivalence_harness.rs` — the diff harness (or a separate binary if cleaner).
- Per-fixture trace files for any mismatch under `tests/bases/diffs/<fixture-name>.diff`.

**Verification clause:**
- 100% diff-clean across all 75 fixtures.
- For each diff (during iteration before clean): identify root cause (parse difference, type coercion, etc.), fix in §A-§E, re-run.
- LL-014: if any single fixture mismatch requires more than 3 fix attempts, STOP and root-cause-investigate before continuing.

**Rollback:** if behavioral mismatches can't be resolved cleanly across the fixture set, revert §A-§E entirely. **The MIG is gated on §F passing.** No code ships if §F doesn't diff clean.

### §G — Performance verification (< 50ms on 7,600 notes)

**What it does:**
1. On the same DB snapshot from §F, time each of the 75 fixtures.
2. Verify all 75 return in under 50ms (the §10.3 mandate).
3. For outliers (any fixture exceeding 50ms): identify the bottleneck. Likely candidates:
   - Tag source FTS5 MATCH (verify the query plan; may need a different index).
   - Sort by user-defined property without an index (consider expression index on `json_extract(properties_json, '$.<prop>')` for hot properties).
   - Large-result-set fixtures with multiple json_extract evaluations.

**Files touched:**
- May add SQLite indexes if needed: `CREATE INDEX IF NOT EXISTS idx_note_meta_library ON note_meta(library_name);` (probably already exists for search.rs).
- Performance report: `lab/reports/MIG-054-perf-2026-NN-NN.md` — per-fixture timings, slowest queries, any added indexes.

**Verification clause:**
- All 75 fixtures < 50ms. Median expected well below.
- Document max query_time_ms and 95th percentile in the perf report.

**Rollback:** if performance can't hit 50ms cleanly, revisit §A SQL design. Possible: lazy column materialization, query-result streaming, or LIMIT-with-paging on large result sets (but this changes the API surface — would require an Architect-doc amendment).

### §H — 3-agent audit (`/migration` Phase 4)

**What it does:** Spawn three parallel agents:

1. **Invariants agent** — verifies all 11 invariants from Architect §3 hold post-implementation. For each invariant, file:line evidence + a PASS/FAIL/UNCLEAR verdict. Special attention to:
   - Inv #5 (behavioral equivalence) — confirms §F harness passed.
   - Inv #6 (no regression on update_note_property) — checks §E's graceful degradation works.
   - Inv #11 (backward compat with legacy `.base` files) — checks §D's alias handling.
2. **Drift agent** — checks for new guards or undocumented constraints introduced during the build. Per LL-023: any new write-path that the system doesn't know about is a drift. Particular focus: did §E introduce any race condition between the immediate write and the file-watcher re-parse?
3. **Migration path agent** — first-boot scenarios (fresh universe with no `note_meta` rows), schema mismatch (old DB with no `properties_json` column), mid-build interrupt (partial §A applied), rollback (reverting individual steps).

**Files touched:** `lab/reports/MIG-054-audit-2026-NN-NN.md` — consolidated audit report with each agent's findings.

**Verification clause:**
- All 11 invariants PASS.
- Zero new undocumented drift items (any drift caught → add to LESSONS-LEARNED or open a PJ).
- All migration scenarios PASS or have a documented graceful-failure path.

**Rollback:** if the audit surfaces a P1 issue, fix it before §I; if a P2/P3, document as a follow-up PJ.

### §I — Boss-test gate (Stage 1)

**What it does:** Eisa runs the new build on his Cognitive Knowledge universe (now live, after §F's copy-test passed).

**Tutorial format (per Testing Instructions Rule):**

> **What's being tested:** Constellation Bases now reads from the search index instead of walking your folders, so every Base opens instantly — even on universes with thousands of notes. Cell edits in a Base view also become instant (no more 1.5-second wait for the row to update).
>
> **Test 1 — Open a Base, time the response.**
> 1. Open Constellation. Navigate to a Bases view you've built on your Cognitive Knowledge universe (any existing one).
> 2. The view should appear **without any visible loading delay** — under one second from click to rows visible.
> 3. **If you see a spinner or empty rows for more than 1 second**, that means the query is slower than the 50ms gate. Report which Base + the universe note count.
>
> **Test 2 — Edit a cell in place.**
> 1. In the same Bases view, click a cell in any row that holds a frontmatter property (e.g., a `status:` or `tag:` column).
> 2. Type a new value. Hit Enter (or Tab away).
> 3. **The row should update with the new value immediately** — within a fraction of a second, no visible delay.
> 4. **If the row stays at the old value for 1+ seconds**, that's the Q2 fix not working. Report which Base + which property.
>
> **Test 3 — Confirm the file on disk is updated.**
> 1. After Test 2, navigate to the note you just edited.
> 2. Open it in the editor.
> 3. The frontmatter `status:` (or whichever property you edited) should show the new value.
> 4. **If it shows the old value**, the cell edit didn't persist. Critical bug — report immediately.
>
> **Test 4 — Old `.base` files still work.**
> 1. Open any older `.base` file that you created before this build. (If you can't remember which, the most recently-modified one is fine.)
> 2. The Base should load and show its rows just like any other Base.
> 3. **If the Base shows an error or empty rows**, the Q1 backward-compat is broken. Report the file name.
>
> **Test 5 — Same rows as before.**
> 1. Open 3-5 different Bases you use regularly.
> 2. For each, the rows shown should be **exactly the same notes** as before this build. No missing notes, no extra notes, no shuffled order (unless you've added/changed notes since the last open).
> 3. **If you spot a row that's missing or extra**, report which Base + which note.

**Verification clause:** Eisa reports "Pass" for all 5 tests. Any "Fail" → identify root cause, fix per LL-014 (max 3 attempts before root-cause investigation), re-test.

**Rollback:** if Boss-test reveals an unfixable issue, revert the MIG; the Architect doc may need amendment before re-attempt.

### §J — PCS + Orientation v2.37 + help-doc note

**What it does:**

1. **Help doc updates:**
   - `docs/help.uConstellation.World/Bases/Bases.md` — add a brief "Bases now load instantly" + "Cell edits are now instant" note. ~2 paragraphs.
   - 14-locale translation via background sub-agent.
2. **Orientation v2.37** — bump v2.36 → v2.37 with a "What changed in v2.37" preamble:
   - Phase 1 of Bases shipped (MIG-054 §A through §I closed).
   - The 5 differentiator promise becomes real: queries are instant; cell edits are instant.
   - §4582 corrected inline: 5 commands → 10 commands.
   - §8 Migrations table adds MIG-054 row (Architect approved, Plan approved, Build §A-§J landed).
   - Boss-test §I results.
3. **MoCh** — a fresh MoCh-YYYY-MM-DD-HHMM.md for the build cascade conversation.
4. **Session log** — `lab/reports/SESSION-LOG-YYYY-MM-DD.md` (whatever day §J lands) — the §A-§J commit map + Boss-test outcomes + LL entries if any.
5. **3-agent audit report** committed at `lab/reports/MIG-054-audit-YYYY-MM-DD.md`.
6. **Performance report** committed at `lab/reports/MIG-054-perf-YYYY-MM-DD.md`.
7. **PCS:** stage all files, commit with message `MIG-054 §J — Phase 1 SHIPPED + orientation v2.37 + PCS`, push.

**Verification clause:** push lands clean; Eisa confirms PCS.

**Rollback:** if PCS reveals a doc inconsistency, fix in a follow-up commit.

---

## §3. Step Dependencies

```
§A (SQL skeleton, folder+all)
 ├──> §B (tag via FTS5)
 ├──> §C (json_each columns_detected)
 ├──> §D (field rename + vault retirement)
 └──> §E (cell-edit refresh event)
        │
        v
§F (behavioral-equivalence pass — requires §A-§E)
 │
 v
§G (perf — requires §F passing)
 │
 v
§H (3-agent audit — requires §A-§G)
 │
 v
§I (Boss-test — requires §A-§H)
 │
 v
§J (PCS — requires §I pass)
```

§B / §C / §D / §E can land in any order after §A; they don't strictly depend on each other. The Plan above orders them §B → §C → §D → §E for natural reading; the actual commit order can flex based on what's cleanest at the moment.

---

## §4. Cross-Cutting Verifications

1. **Behavioral equivalence (Q3 + Q4 + Architect §3 inv #5).** The §F harness is the central correctness gate. No code ships if §F doesn't diff clean.
2. **Performance (Architect §10.3, §G).** All 75 fixtures under 50ms on the 7,600-note universe.
3. **LL-014 (don't patch the same bug more than three times).** If any single step requires more than 3 fix attempts, STOP and root-cause-investigate. This is the project's hard-learned rule.
4. **LL-022 (lazy-mount everywhere).** §E's frontend listener must follow the existing lazy-mount pattern — only attach the listener when the Bases view is visible; detach on unmount.
5. **LL-023 (drift catches).** Any new guard introduced during build (e.g., a new "this property is treated specially" branch) must be documented in the Architect doc as an amendment OR caught by §H's drift agent.
6. **LL-025 (test on a copy of the real DB).** §F always runs against a snapshot of Eisa's universe, never directly on live. **This is non-negotiable.**

---

## §5. Risks per Step (cross-reference Architect §8)

| Risk (Architect §8) | Mitigation step | Detection step |
|---|---|---|
| #1 Frontmatter edge cases (inline lists, multi-line lists, quoted scalars) | §A (filter translation + JSON parsing) | §F (behavioral equivalence catches) |
| #2 `note_meta.properties_json` empty for old notes | §A (verify pre-flight) | §F (gold-master fixtures would expose this) |
| #3 FTS5 hashtag tokenization | §B (verify MATCH syntax; fall back to LIKE) | §B verification clause |
| #4 `update_note_property` immediate-write conflict with file-watcher | §E (idempotent content_hash check) | §H (drift agent + race condition check) |
| #5 `library_path` join misses for cUniverse children | §A (test cross-library fixtures) | §F (10 cross-library fixtures) |
| #6 Backward-compat read of `selectedVaults` | §D (precedence logic) | §F (legacy-shape fixtures) |

---

## §6. Open Ask Still Outstanding

**5-10 representative `.base` files from Eisa's Cognitive Knowledge universe** to seed the §F fixture gold-master tier. These are the "if these still return correct rows, we're shippable" fixtures.

Concretely: copies (or paths to) `.base` files Eisa actually uses regularly. The §F harness snapshots their output on the existing (pre-§A) implementation and compares against the new SQL-backed implementation.

**Not blocking the Build start** — the §F harness can run on the 65 synthetic fixtures while Eisa nominates the 10 real Bases at his pace. Final §F clean-pass gate requires all 75.

---

## §7. Build Cascade — What Eisa Approves When

**With approval of this Plan doc**, Eisa authorizes the §A → §J cascade. The cascade proceeds autonomously per Plan-Approval-Equals-Build-Approval (CLAUDE.md top principal). I do not seek per-step approval.

Stops happen only at:

1. **§I Boss-test gate** — user-testable verification. I deliver Test 1-5 as the Testing-Instructions-Rule tutorial above; Eisa runs them; pass/fail returns to me.
2. **Architect-amendment-worthy surprise** during build — if §A reveals an unmapped invariant or contract change not in the Architect doc, I stop and surface it.
3. **§F harness fails to diff clean** — automated stop. I diagnose root cause, fix, re-run. If 3 attempts fail (LL-014), I stop and surface the case to Eisa.
4. **§G perf gate fails** — same shape as §F.
5. **§J PCS** — final state.

Between stops, I log progress to `lab/reports/SESSION-LOG-YYYY-MM-DD.md` per the Standing Order. Each step lands as its own commit with the format `MIG-054 §X — <description>`.

---

## §8. Rollback Strategy — Whole-MIG and Per-Step

**Whole-MIG rollback (worst case):**
- Revert §A through §J as a sequence of git reverts (or a single revert commit collapsing all of MIG-054).
- `.base` files on disk remain unchanged.
- `note_meta.properties_json` remains as-is (no destructive operation in this MIG).
- No data loss. Only the new fast `query_base` reverts to filesystem walk.

**Per-step rollback (granular):**
- Each step's "Rollback" subsection above describes the local revert path.
- §A is the biggest cliff: reverting §A means reverting §B-§E too (they depend on §A's SQL builder structure).
- §F / §G / §H / §I / §J revert cleanly without affecting earlier steps.

**The MIG is gated on §F passing.** If §F can't diff clean, the recommendation is to revert §A-§E and revisit the Architect doc.

---

## §9. Closing — Ready for Approval

This Plan doc decomposes Architect v1.1 into 10 landable commits with verification clauses. Each step has a rollback path. Cross-cutting verifications are documented. Risks per step map to Architect §8.

**With Eisa's approval, the Build cascade fires — §A through §J — autonomously per Plan-Approval-Equals-Build-Approval.**

The only stop along the way is the §I Boss-test gate (and any architectural-surprise stop or §F/§G automated failures).

Architect doc approval gate: ✓ (Eisa, 2026-05-25).
Plan doc approval gate: **pending Eisa's "Approved"**.

After approval: §A starts.

---

*End of MIG-054 Plan v1.0. Updated only on substantive change of build sequence; the Build commits log progress in the session log.*
