---
title: MIG-055 — Constellation Base, Clean Slate (Plan)
version: 1.0
date: 2026-05-26
status: Plan doc. Awaiting Eisa's approval before the Build cascade fires.
direction_holder: Eisa
drafter: Claude (Opus 4.7)
predecessor: docs/MIG-055-constellation-base-clean-slate-ARCHITECT.md (v1.1, all 7 open questions locked 2026-05-26)
concept_paper: docs/Constellation-Base-Concept-Paper-v1.4.md (design north star)
---

# MIG-055 — Constellation Base, Clean Slate (Plan)

## §1. Premise

The Architect v1.1 locked the lens-definition YAML schema, the `execute_lens` Tauri command surface, the host-note assemblage entry pattern, the Five Acts sidebar label, the dimension naming convention (`note.X` / `link.X` / `note.cns.X` / `note.cece.X`), and 6 other foundational decisions. This Plan doc phase-decomposes the Architect's §8 step outline into **10 landable commits, each with an explicit verification clause**.

**Cascade discipline:** Per "Plan Approval = Build Approval" (CLAUDE.md top principal), once Eisa approves this Plan doc, §A → §J runs as one cascade — no per-step approval needed. Stops only at:

1. User-testable verification clauses (Boss-test at §I).
2. Genuine architectural surprise.
3. §F automated test failures.
4. §J PCS.

Verification clauses are testable artifacts — not "looks good to me."

**The MIG-054 audit lessons carry over.** LL-014 (don't patch the same bug more than three times), LL-022 (lazy mount everywhere), LL-023 (drift catches), LL-025 (test on a copy of the real DB). Applied to MIG-055 with the same discipline.

## §2. Step Sequence

Ten steps. Each lands as one commit with the format `MIG-055 §X — <description>`.

### §A — Dimension registry foundation

**What it ships.** A new `src-tauri/src/lens/` module. Files:
- `lens/mod.rs` — module entry; declares submodules; re-exports.
- `lens/dimensions.rs` — the `DimensionDef` struct + a const array of dimension definitions. v1 ships exactly 4 dimensions: `note.name`, `note.path`, `note.created_at`, `note.headline`. Each entry declares its kind (Text / Timestamp), SQL expression, sortable flag, filterable flag, and supported filter ops.

**Files touched.**
- `src-tauri/src/lens/mod.rs` (new)
- `src-tauri/src/lens/dimensions.rs` (new)
- `src-tauri/src/lib.rs` — add `pub mod lens;` declaration.

**Verification clause.**
- `cargo check --lib` clean (no new warnings beyond pre-existing 42).
- `cargo test --lib lens::dimensions::tests` passes ≥ 6 unit tests:
  - `dimension_registry_includes_4_v1_dimensions`
  - `dimension_registry_lookup_by_name`
  - `dimension_registry_unknown_returns_none`
  - `note_created_at_filter_ops_includes_after_before_between_within`
  - `note_name_is_sortable`
  - `note_headline_is_not_sortable_not_filterable_in_v1`

**Rollback.** Revert §A; the new module disappears. No external consumers yet.

### §B — Lens YAML parser + schema validator

**What it ships.** The `LensDefinition` Rust struct + a serde-yaml parser + schema-validation logic that catches: unknown dimensions (in `where` / `order` / `columns`), unsupported filter ops for the named dimension, unsupported `view` shapes, missing required fields, schema version mismatches.

**Files touched.**
- `src-tauri/src/lens/definition.rs` (new) — `LensDefinition`, `LensScope`, `LensFilter`, `LensSort`, `LensColumn`, `LensView` types.
- `src-tauri/src/lens/parser.rs` (new) — `parse_lens_yaml(yaml: &str) -> Result<LensDefinition, LensError>`.
- `src-tauri/src/lens/validator.rs` (new) — `validate(def: &LensDefinition) -> Result<(), LensError>` using the §A dimension registry.
- `src-tauri/Cargo.toml` — confirm `serde_yaml` already present (audit'd in MIG-054 §B Arabic-normalizer pass — already a dep).

**Verification clause.**
- `cargo test --lib lens::parser::tests` + `lens::validator::tests` ≥ 15 unit tests covering:
  - Valid schema v1 parse round-trip (the Recent Captures fixture)
  - Unknown dimension in `where` → error with the unknown name surfaced
  - Unknown filter op for known dimension → error
  - Schema version other than 1 → error
  - Missing `lens` name → error
  - Empty `columns` → error (a lens must show at least one column)
  - Multilingual lens names (Arabic / Persian) parse correctly
  - The `template` field is optional; presence parses correctly
  - `scope.federation: "off"` parses correctly (foundation for future opt-out)
  - `where` empty list (no filter) is valid → returns all rows in scope
  - `order` empty (no sort) → arbitrary order acceptable
  - `view: list` parses; `view: table` rejects with "supported v1 views: [list]"
  - Time-relative filter values (`"now - 14 days"`, `"now"`, ISO timestamps) parse correctly

**Rollback.** Revert §B; parser disappears. §A's dimension registry stays.

### §C — `execute_lens` Tauri command

**What it ships.** The single Tauri command that takes a YAML lens definition + host context, runs the SQL query against the federated library set, returns `LensResult` with rows + dimensions populated.

**Files touched.**
- `src-tauri/src/lens/query.rs` (new) — `execute_lens` Tauri command + supporting types (`LensResult`, `LensRow`, `DimensionValue`).
- `src-tauri/src/lens/sql_builder.rs` (new) — builds the SQL string + parameter list from a validated `LensDefinition`.
- `src-tauri/src/lib.rs` — register `execute_lens` in `invoke_handler!`.

**SQL shape for v1** (Recent Captures):

```sql
SELECT
    note_meta.path,
    note_meta.name,
    note_meta.library_name,
    note_meta.created_at,
    note_summaries.headline
FROM note_meta
LEFT JOIN note_summaries ON note_summaries.path = note_meta.path
WHERE note_meta.library_name IN (?, ?, ...)   -- federated library set
  AND note_meta.created_at >= ?                -- "now - 14 days" → unixepoch - 14*86400
ORDER BY note_meta.created_at DESC
```

**Verification clause.**
- `cargo test --lib lens::query::tests` ≥ 8 integration tests using in-memory SQLite + seeded `note_meta` + `note_summaries`:
  - Recent Captures fixture returns rows from last 14 days only
  - Federated library scoping respects `scope.libraries: all`
  - Federated library scoping respects `scope.libraries: [Lib1]` (specific subset)
  - `scope.federation: off` excludes cUniverse children (when present)
  - Empty result returns `rows: []`, `total_count: 0`
  - NSC headline JOIN works (rows with summaries get headline; rows without get null/empty)
  - `query_time_ms` populated and < 200ms for 1000-note seeded universe
  - Multilingual note names + library names round-trip cleanly

**Rollback.** Revert §C; the Tauri command vanishes; frontend can't query lenses (degrades to nothing — no fallback).

### §D — Markdown ` ```base ` block extractor + LensBlock renderer

**What it ships.** A new Svelte component `src/lib/components/LensBlock.svelte` that mounts when the markdown renderer encounters a fenced code block with language `base`. The component reads the block text, calls `execute_lens`, renders `LensResult` rows.

**Files touched.**
- `src/lib/lens/store.ts` (new) — TS bridge to `execute_lens` Tauri command + types (`LensDefinition`, `LensResult`, `LensRow`, `DimensionValue`).
- `src/lib/components/LensBlock.svelte` (new) — the renderer.
- `src/lib/components/NotePane.svelte` (modified) — the CM6 / markdown plugin pipeline already has fenced-code-block handling; add the `base` language hook to mount `LensBlock` for matching blocks. Specifics locked at build time (CM6 extension shape).

**LensBlock UX (v1).**
- On mount: call `execute_lens(blockText, hostContext)`.
- Loading state: small "Loading lens..." text.
- Error state: red text with the validator's error message (so user sees `"Unknown dimension: note.frobnitz"` directly).
- Success state: render rows as a list view. Each row is `<note.name> — <note.headline>`. Click on name opens the note.

**Verification clause.**
- svelte-check: 0 new errors (3 pre-existing baseline errors remain unchanged).
- Manual exercise (recorded for §I tutorial):
  - Create a test note `temp-test.md` with body containing a ` ```base ` block holding the Recent Captures YAML.
  - Open the note in Constellation.
  - The lens block renders below the prose with a list of recent notes + their NSC headlines.
  - Clicking a row name opens that note in a new tab.

**Rollback.** Revert §D; LensBlock disappears; the ` ```base ` blocks render as plain code (the default CM6 behavior).

### §E — System note: "Observation — Recent Captures"

**What it ships.** A new function `init_five_acts_system_notes(app)` called from `init_db` (or the universe-init path). On first boot of a universe (idempotent), it creates the file `{universe}/Five Acts/Observation — Recent Captures.md` with:

```markdown
---
template: five-acts.observation
description: "The intake queue — last 14 days of notes. Browse what you've recently captured."
---

# Observation — Recent Captures

The Observation Act of knowledge formulation is **noticing**. Before connecting, tensing, synthesizing, or committing, you must first SEE what you've recently captured. This page shows you the last 14 days of notes across your universe — your intake queue.

Scan, read, mark as processed, or develop further. The list is your raw material.

\`\`\`base
schema: 1
lens: "Recent Captures"
template: five-acts.observation
scope:
  libraries: all
  federation: auto
where:
  - dimension: note.created_at
    op: after
    value: "now - 14 days"
order:
  - dimension: note.created_at
    direction: desc
columns:
  - dimension: note.name
  - dimension: note.headline
view: list
\`\`\`
```

**Files touched.**
- `src-tauri/src/lens/system_notes.rs` (new) — `init_five_acts_system_notes(app)` function + helpers.
- `src-tauri/src/search.rs` (modified, small) — call `init_five_acts_system_notes` once during init_db / universe-init.

**Edit-policy invariant.** Per Architect §11 #3 lock — the function checks: if the file exists AND its content differs from the canonical template, **do nothing** (user took ownership). If absent, create. If present and content matches the canonical, leave (no-op). The `template: five-acts.observation` frontmatter marker is the lineage record.

**Verification clause.**
- `cargo test --lib lens::system_notes::tests` ≥ 5 tests:
  - Fresh universe → file is created with canonical content
  - Existing file with canonical content → unchanged (no-op)
  - Existing file with user edits → unchanged (transfer-on-edit honored)
  - Existing file with empty `Five Acts/` directory → directory is created
  - Two consecutive init_db calls → idempotent (no duplicates, no overwrites)

**Rollback.** Revert §E; the system note is no longer auto-created. Existing system notes on disk remain (the function never deletes).

### §F — Sidebar "Five Acts" section

**What it ships.** A new section in the left sidebar listing Five Acts host notes (just one in v1: "Observation — Recent Captures"). Replaces the old "Bases" section. Click opens the host note in a tab — which routes to the normal NoteEditor (since the host note IS a `.md` file), and the `` ```base `` block inside gets rendered by §D's LensBlock.

**Files touched.**
- `src/routes/+layout.svelte` — replace the old "Workspace Bases" section (lines 4720–4754 of the pre-revert version, currently absent on `main` post-revert) with a new "Five Acts" section. The section enumerates files in `{universe}/Five Acts/` and renders them as clickable entries.
- `src/lib/lens/store.ts` — add `listFiveActsNotes()` helper that calls a new Tauri command `list_five_acts_notes`.
- `src-tauri/src/lens/system_notes.rs` — add `list_five_acts_notes` Tauri command that enumerates `*.md` files in `{universe}/Five Acts/` and returns their paths + display names.
- `src-tauri/src/lib.rs` — register `list_five_acts_notes`.
- `src/lib/i18n/*.json` (15 files) — add `sidebar.fiveActs` translation key. EN value: "Five Acts". AR value: "الأفعال الخمسة" (or Eisa's preferred Arabic — locked at build).

**Verification clause.**
- Sidebar shows "Five Acts" section header (collapsible).
- Expanding shows "Observation — Recent Captures" as a single entry.
- Clicking opens the host note in a new tab.
- The note renders with prose + the embedded lens block showing the rows.
- 15-locale i18n key resolves (verified by svelte-check + a quick locale-flip in dev).

**Rollback.** Revert §F; the sidebar entry disappears. Users can still open the Five Acts notes via the file tree (since they live in a visible folder per Architect §11 #2 lock).

### §G — Behavioral tests on synthetic universe

**What it ships.** An integration test module `src-tauri/src/lens/tests.rs` (or `tests/lens_equivalence.rs`) with ~10 end-to-end test cases that seed a synthetic universe (in-memory SQLite + temp dir) and exercise the lens pipeline (parse → validate → execute → result).

**Test cases (Plan §G locks the fixture set):**
1. `recent_captures_returns_last_14_days_only`
2. `recent_captures_excludes_older_notes`
3. `recent_captures_orders_desc_by_created_at`
4. `recent_captures_with_nsc_headlines_populated`
5. `recent_captures_with_missing_headlines_returns_empty_string`
6. `recent_captures_respects_library_filter`
7. `recent_captures_federation_auto_includes_cuniverse_children`
8. `recent_captures_federation_off_excludes_cuniverse_children`
9. `recent_captures_empty_universe_returns_no_rows`
10. `recent_captures_multilingual_note_names_round_trip`

**Files touched.**
- `src-tauri/src/lens/tests.rs` (new) OR `src-tauri/tests/lens_equivalence.rs` (new) — location locked at build time based on whether the items are `pub(crate)` or `pub`.

**Verification clause.** All 10 tests pass. `cargo test --lib lens::` total ≥ 35 tests across §A + §B + §C + §G (registry + parser + query + integration). `cargo check --lib` clean.

**Rollback.** Revert §G; the test module disappears. §A–§F continue to work; just less coverage.

### §H — 3-agent audit

**What it ships.** Three parallel audit agents per `/migration` Phase 4 discipline:
- **Invariants agent** — verifies the Architect §3 invariants hold (clean break from old MVP; no BaseView reuse; no auto-detected columns; federation auto-default; etc.).
- **Drift agent** — checks for undocumented behaviors (SQL injection / panic paths / error-message hygiene / dimension-registry leaks).
- **Migration-path agent** — fresh universe / existing universe with old `.base` files / existing universe with no `Five Acts/` folder / system-note edit / re-init / rollback scenarios.

Consolidated report at `lab/reports/MIG-055-audit-YYYY-MM-DD.md`.

**Verification clause.** All invariants PASS. Zero P1 drift findings (P2/P3 documented). All migration paths PASS or have documented graceful-failure paths.

**Rollback.** If audit surfaces a P1, fix it before §I; if P2/P3, document as PJ.

### §I — Boss-test gate

**What it ships.** Eisa runs the new build on his Cognitive Knowledge universe. Five tutorial tests (per the Testing Instructions Rule):

**Test 1 — Sidebar Five Acts entry visible.** On installer launch + universe open, the left sidebar shows a "Five Acts" section. Expanding reveals "Observation — Recent Captures".

**Test 2 — Open the note.** Click "Observation — Recent Captures". A note tab opens. The note shows prose (the Observation Act explanation) above a rendered lens (NOT raw YAML) — a list of recent notes with names + NSC headlines.

**Test 3 — Click a row.** Click any note name in the rendered list. That note opens in a new tab.

**Test 4 — Empty / sparse case.** If the universe has no notes created in the last 14 days, the lens shows "No notes match this lens." (graceful empty state).

**Test 5 — Multilingual.** If any of the last 14 days' notes have Arabic / Persian / Hebrew names or headlines, they render correctly (RTL where appropriate; no mojibake).

**Verification clause.** Eisa reports "Pass" for all 5. Any fail → LL-014 discipline (max 3 attempts before root-cause); fix commits land before §J.

**Rollback.** If Boss-test reveals unfixable issues, revert the whole MIG cascade; Architect needs amendment.

### §J — PCS + Orientation v2.37 + help-doc note

**What it ships.**
- **Help docs.** New topic `docs/help.uConstellation.World/Five Acts/Five Acts.md` (and the 14 locale mirrors) — explains the Five Acts cognitive model + the Observation Act + how the host-note lens works. ~200 lines EN; 14-locale translation via background sub-agent.
- **Orientation v2.37** — bump v2.36 → v2.37. Preamble captures: the MIG-054 revert + the MIG-055 fresh-start arrival of "Constellation Base" as the new shape. The §4 subsystem map gains a "Lens" entry (replacing the old Bases entry). §8 Migrations table adds MIG-055 row.
- **MoCh** — fresh `docs/MoCh/MoCh-YYYY-MM-DD-HHMM.md` for the build cascade conversation.
- **Session log** — `lab/reports/SESSION-LOG-YYYY-MM-DD.md` with the §A–§J commit map.
- **3-agent audit report** — `lab/reports/MIG-055-audit-YYYY-MM-DD.md`.
- **Single commit** — `MIG-055 §J — Phase 1 SHIPPED + orientation v2.37 + PCS`, push.

**Verification clause.** Push lands clean; Eisa confirms PCS.

**Rollback.** If PCS reveals doc inconsistencies, fix in follow-up commit.

## §3. Step Dependencies

```
§A (dimension registry)
 ├──> §B (parser uses registry)
 │     └──> §C (query uses parser + registry)
 │           ├──> §D (LensBlock calls execute_lens)
 │           │     ├──> §E (system note depends on LensBlock to render)
 │           │     └──> §F (sidebar opens note → LensBlock renders)
 │           └──> §G (integration tests exercise full pipeline)
 │
 v
§H (audit — requires §A-§G complete)
 v
§I (Boss test — requires §H clean)
 v
§J (PCS — requires §I pass)
```

§A is the hard prerequisite. §B/§C/§D/§E/§F can interleave once §A is in place, but the natural order keeps the diff cleanest.

## §4. Cross-Cutting Verifications

1. **LL-014 — Don't patch the same bug more than three times.** If any §A-§G test fails on 3 fix attempts, STOP and root-cause investigate.
2. **LL-022 — Lazy mount.** The LensBlock component (§D) must attach its IPC subscriptions only when visible; detach on unmount.
3. **LL-023 — Drift catches.** Any new guard/constraint introduced mid-build must be documented in the Architect doc as an amendment OR caught by §H's drift agent.
4. **LL-025 — Test on a copy of the real DB.** §I Boss-test runs on Eisa's live universe; but if any pre-§I integration test exercises real-world data shapes, it uses a snapshot copy (not live).
5. **File-Over-App per Concept Paper v1.4 §10.2.** The lens-definition YAML lives in plain `.md` files. The host note IS the durable artifact; the rendered lens is ephemeral. Delete the markdown → lens is gone. No proprietary container.
6. **Multilingual native per §10.4.** All test cases include at least one Arabic / Persian / Hebrew fixture. All UI strings (sidebar label, empty-state message, error messages) route through i18n (`$t()`).

## §5. Risks per Step (cross-reference Architect §9)

| Risk (Architect §9) | Mitigation step | Detection step |
|---|---|---|
| #1 Dimension registry surface lock-in | §A (4 dimensions only in v1, prefix convention frozen) | §H drift audit + future-MIG review of additions |
| #2 NSC headline fetch latency | §C (use cached `note_summaries.headline` via JOIN) | §G test 4 + §I Test 2 |
| #3 System-note creation idempotency | §E (canonical-content diff check; transfer-on-edit lock) | §G tests 3-5 |
| #4 Pre-existing search.rs parse_frontmatter bug | §A (v1 dimensions don't read `properties_json`; bug irrelevant) | §H drift audit confirms no `properties_json` dependency |
| #5 Old `.base` files on disk | §F (sidebar lists ONLY `Five Acts/*.md`; no `.base` enumeration anywhere) | §I Test 1 + §H migration-path audit |
| #6 Lens YAML schema versioning | §B (`schema: 1` required; mismatch = error) | §G test (schema 2 input → validator error) |

## §6. Build Cascade — What Eisa Approves When

**With approval of this Plan doc**, Eisa authorizes the §A → §J cascade. Per Plan-Approval-Equals-Build-Approval (CLAUDE.md top principal), no per-step approval needed. Stops happen only at:

1. **§I Boss-test gate** — user-testable verification. I deliver Tests 1-5 as the Testing-Instructions-Rule tutorial above; Eisa runs them; pass/fail returns to me.
2. **Architect-amendment-worthy surprise** during build — if §A reveals an unmapped invariant or contract change not in Architect v1.1, I stop and surface it.
3. **§G automated test failures** — diagnose root cause, fix, re-run. LL-014 caps fix attempts at 3 before root-cause investigation.
4. **§J PCS** — final state.

Between stops, session log gets updated per the Standing Order. Each step lands as its own commit with format `MIG-055 §X — <description>`.

## §7. Rollback Strategy

**Whole-MIG rollback (worst case):**
- Revert §A → §J as a sequence (or one collapsing commit per the MIG-046 precedent).
- `Five Acts/` system-shipped files on disk remain — they're user-readable markdown; the user can delete if undesired.
- `note_meta` and `note_summaries` untouched — MIG-055 reads from them but never writes.
- No data loss. Only the Lens UI vanishes.

**Per-step rollback (granular):**
- §A: revert; downstream §B-§G break (depend on it); whole MIG comes down with it.
- §B-§G: revert individual; upstream steps survive; downstream of revert breaks.
- §E (system note creation): revert; the function no longer runs at init_db. Existing on-disk system notes are untouched (function never deletes).
- §F (sidebar): revert; sidebar entry disappears. Users still open Five Acts notes via file tree.
- §H/§I/§J: revert cleanly without affecting code state.

**The MIG is gated on §I (Boss-test).** If §I can't reach Pass after 3 LL-014 attempts at root-cause fixes, revert the whole MIG and revisit Architect.

## §8. Closing — Ready for Approval

This Plan doc decomposes Architect v1.1 into 10 landable commits with verification clauses. Each step has a rollback path. Cross-cutting verifications are documented. Risks per step map to Architect §9.

**With Eisa's approval, the Build cascade fires — §A through §J — autonomously per Plan-Approval-Equals-Build-Approval.**

The only stop along the way is §I (Boss-test). After §I Pass → §J PCS → MIG-055 closes; the new Constellation Base ships its first user-visible vertical slice.

Architect doc approval gate: ✓ (Eisa delegated locks 2026-05-26).
Plan doc approval gate: **pending Eisa's "Approved"**.

After approval: §A starts.

---

*End of MIG-055 Plan v1.0. Updated only on substantive change of build sequence.*
