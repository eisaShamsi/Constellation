# MIG-022 §N Close-out Audit — Invariants

**Agent:** Invariants (1 of 3 — parallel to Drift, Migration Path)
**Date:** 2026-05-12
**Scope:** §0 + §A + §D + §E + §B.1-§B.4 (§B.5/§B.6 deferred-by-design)
**Source of invariants:** `lab/reports/MIG-022-ARCHITECT.md` §3 (10 invariants)

---

## Summary: 10 HOLDS · 0 AT RISK · 0 VIOLATED

All ten cross-cutting invariants identified in MIG-022 Architect §3 hold against the shipped cascade. Evidence below.

---

### Invariant 1 — File over app (`.md` is source of truth)

**Status:** HOLDS

**Evidence:** `src-tauri/src/cece/history.rs:5-10` explicit module docstring asserts "the on-disk `.md` file remains the source of truth (CLAUDE.md "File over app"); this table is the temporal index". `note_state_history` is an SQLite-only derived view; no `.md` write paths added in §B (verified by reviewing the §B.1-§B.4 commits — no `notes.rs` / file-write code touched). §A.1 frontmatter additions (`held_by`, `domain`, `function`, `ikhtilāf`, etc.) parse from disk, never write back silently — §A.1 commit body confirms parser-only changes to `src/lib/libraries/store.ts::parseFrontmatter`. §A.2 `supersedes` typed-link is a wikilink suffix on disk; same `.md`-as-truth pattern as the existing 7 link types.

---

### Invariant 2 — Performance Rule 1 (zero perceptible keystroke lag)

**Status:** HOLDS

**Evidence:** No new `invoke()` calls were added to keystroke hot paths. Audited Svelte components touched by MIG-022:
- `src/lib/components/PropertyEditor.svelte`: 0 `invoke()` calls (Grep verified)
- `src/lib/sources/TaxonomyTreePicker.svelte`: 0 `invoke()` calls (Grep verified)
- `src/lib/components/SourceReviewPanel.svelte`: 7 `invoke()` calls — all behind button event handlers (lines 450, 595, 599, 611, 632, 769, 791), none on input events
- `src/lib/components/BacklinksPanel.svelte` / `OutgoingLinksPanel.svelte`: panel-render only, not editor surfaces

§B trigger fires on `note_meta` UPDATE (which already happens on save, debounced — not keystroke). Trigger SQL uses `WHEN OLD.field IS NOT NEW.field` guard at `history.rs:110-112`, so typo-fix saves on body content (which only changes `body_text`/`modified`) do NOT fire the trigger.

---

### Invariant 3 — Performance Rule 8 (write-time derivation)

**Status:** HOLDS

**Evidence:** `note_state_history` is maintained at write-time via the `note_state_history_au` SQLite trigger at `src-tauri/src/cece/history.rs:108-130` (AFTER UPDATE on `note_meta`). No boot-time recompute, no read-time scan. Module docstring (history.rs:7-10) explicitly cites Rule 8. §B.4 query IPCs (`cece_get_note_history`, `cece_query_history` at history.rs:316-420) are pure SELECT against the covering index `idx_note_state_history_note_time` (history.rs:71-72), no derivation at read time. §B.3 backfill (history.rs:185-275) is one-shot, idempotent via `schema_versions.note_state_history_backfill` sentinel — runs once, never recomputes.

---

### Invariant 4 — Editor Parity Rule (NotePane = other editor views)

**Status:** HOLDS

**Evidence:** §A.2 `supersedes` typed-link added to the **shared** CM6 extension at `src/lib/editor/livePreview.ts::TYPED_LINK_TYPES` (per a03460e commit body). Per CLAUDE.md "the shared extension set lives in `$lib/editor/` and is imported by every editor instance" — therefore every editor view inherits supersedes recognition + the `.cm-link-supersedes` decoration class. No view-specific code paths added. No new editor-side feature added that bypasses livePreview. (FocusPane exception still applies per CLAUDE.md.)

---

### Invariant 5 — Living Link Architecture (typed-link integrity)

**Status:** HOLDS

**Evidence:** §A.2 chose D-A1.β (typed-link) over D-A1.α (YAML scalar) for `supersedes`. Verified across all 5 KNOWN_LINK_TYPES sites (per a03460e commit body):
- `src-tauri/src/libraries.rs:1958` — `"supersedes",  // MIG-022 §A.2 (D-A1.β)` (Bash grep verified)
- `src-tauri/src/strata.rs:72` — `"supersedes",  // MIG-022 §A.2 (D-A1.β)`
- `src-tauri/src/tension.rs:53` — `"supersedes",`
- `src/lib/libraries/store.ts:3663` — added in `LINK_TYPE_NAMES` constant
- `src/lib/libraries/store.ts:3604` — pill color added; line 3615 text color added

Lockstep across Rust (3 sites) + frontend (2 sites). No parallel YAML representation introduced. CLAUDE.md cognitive-vocabulary updated in same commit (per commit body).

---

### Invariant 6 — CECE 6-cataloger contract (no restructuring)

**Status:** HOLDS

**Evidence:** §D refactored `synthesize()` in `src-tauri/src/cece/synthesis.rs` to per-axis dispatch (lines 104-198). The 6-cataloger ensemble shape is preserved: `synthesis.rs:109-114` still partitions all incoming trails into voiced/silent regardless of which catalogers exist; `vote_on_axis` (line 237) iterates ALL voiced trails, not a fixed subset. §E.2/E.3 added optional `reasoning_template: Option<ReasoningTemplate>` to `ReasoningTrail` at `cece/cataloger.rs:122-123` with `#[serde(default, skip_serializing_if = "Option::is_none")]` — additive only. No cataloger added or removed. PJ-040 fix narrowed UA short-circuit semantics (lines 124-139) without altering the cataloger contract.

---

### Invariant 7 — i18n parity (en + ar minimum, 13-locale backfill)

**Status:** HOLDS

**Evidence:** All four MIG-022 i18n surfaces are present in all 15 locale files (Grep verified across `src/lib/i18n/*.json`):
- `cece.confidence.*` (PJ-042, §E.1) — 15/15 files match `"confidence"`
- `cece.reasoning.*` (PJ-041, §E.2) — 15/15 files match `"reasoning":`
- `cece.taxonomy.*` (PJ-043, §E.3) — 15/15 files match `"taxonomy":`
- `linkTypes.supersedes` (§A.2 + §A.4.d) — 15/15 files match `supersedes`

§A.4.b ships User Manual chapter + Help topic in 14 locales (per 738a35a commit). Boss corrections for ar (9a6a938) and ur (5e9b5ed) landed for the supersedes term ("يحلّ محلّ" / "کی جگہ لیتا ہے").

---

### Invariant 8 — Reliability data continuity (composite_json shape)

**Status:** HOLDS

**Evidence:** §E.2 added `reasoning_template: Option<ReasoningTemplate>` as a new field on `ReasoningTrail` (cataloger.rs:122-123) and `composite_reasoning_template` on `CompositeAssignment` (synthesis.rs:78-79) — both wrapped in `#[serde(default, skip_serializing_if = "Option::is_none")]`. Existing `reasoning: String` (cataloger.rs:115) and `composite_reasoning: String` (synthesis.rs:71) fields **preserved** as English fallback. Legacy `composite_json` blobs without these fields deserialize cleanly via `serde(default)` to `None`. The doc comment at synthesis.rs:69-71 explicitly describes the legacy fallback path. `cece_record_correction_for_card` IPC consumers see no schema break (existing fields untouched).

---

### Invariant 9 — Migration from CECE v1 universes (additive landings)

**Status:** HOLDS

**Evidence:**
- `note_state_history` table created via `CREATE TABLE IF NOT EXISTS` at `history.rs:64`; trigger via `CREATE TRIGGER IF NOT EXISTS` at `history.rs:108`. Both idempotent.
- §B.3 backfill (`backfill_initial_history` at history.rs:185) checks `schema_versions.note_state_history_backfill` sentinel before running (history.rs:187-197) — skips on subsequent boots.
- §B.3 backfill is **resumable**: wraps in `BEGIN IMMEDIATE` transaction (history.rs:200) and rolls back on any error (test `mig022_b3_backfill_idempotent` at history.rs:699-714 verifies).
- §A.1 frontmatter parser changes are additive — old notes without `held_by`/`ikhtilāf` keep working (existing parser branches unchanged for unknown keys).
- §A.2 supersedes is a new typed-link name that old notes simply won't carry; no backward-incompat schema.
- §0 cleanup deleted only DEAD code (per `lab/reports/MIG-022-§0-REACHABILITY.md` reachability analysis confirmed in commit d626ae7) — no live IPCs removed; KEEP set was `classifier/mod.rs` (3 active commands), `scan_job.rs` (3 IPCs), `correction_log.rs` (4 call sites).

---

### Invariant 10 — No regression on boot time / typing latency (7,600+ note Universe)

**Status:** HOLDS

**Evidence:**
- §B.1 covering index `idx_note_state_history_note_time` (history.rs:71-72) ensures lookup is O(log N + k) per note.
- §B.3 backfill uses single bulk INSERT with DROP-TRIGGER protocol (history.rs:207, 219-234) — avoids 7,600 sequential trigger fires (the canonical SQLite footgun called out in commit 6ecf8ec body).
- §B.2 trigger WHEN guard at history.rs:110-112 means typo saves (which don't change `sources`/`content_type`/`properties_json`) skip the INSERT — no per-keystroke history bloat.
- §0 deleted -982 LoC; net cost negative.
- §A.1 frontmatter parser remains in JS but parses on-demand (not on every keystroke) — same access pattern as pre-§A.
- No new `invoke()` calls on keystroke hot path (Invariant 2 evidence). Boot time test plan: §B.3 backfill on 7,600 notes is one-shot under transaction; commit body for 6ecf8ec describes resumable design with progress events. Mid-cascade Boss-Test gates (Gate 1 after §0+§D+§E.1; Gate 2 after §E full; Gate 3 after §A) all PASSED per Pending Jobs / orientation v1.99.

---

## Notes for the close-out

- **§B.5 (Sight v3 overlay) and §B.6 are deferred-by-design** — Sight v3 is retired by the Sight v5 Concept Paper v3.1 (commit 61cd085). The §B.4 query IPCs (`cece_get_note_history`, `cece_query_history`) remain available for whatever Sight v5 surface lands in MIG-NNN; no live consumer today, but no orphan either — they are the public read API for the temporal-axis derived view.
- All 11 §B history tests pass per c3c5c66 commit body (4 §B.1 + 4 §B.2 + 3 §B.3); §D adds 2 tests (synthesis.rs partial-UA cases, 92→94 cece total per c072700 commit body). End-to-end IPC tests for §B.4 deferred (require Tauri AppHandle setup); compiled-and-types-resolve verified via `cargo check --lib`.
