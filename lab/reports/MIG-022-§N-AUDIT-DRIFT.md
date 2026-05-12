# MIG-022 §N Close-Out Audit — Drift Agent

**Date:** 2026-05-12
**Scope:** §0 + §D + §E (1+2+3) + §A (1-4) + §B (1-4)
**Files audited:** synthesis.rs, history.rs, horizontal_taxonomy.rs, vertical_taxonomy.rs, SourceReviewPanel.svelte, PropertyEditor.svelte, store.ts, livePreview.ts, search.rs, lib.rs, classifier/mod.rs, all 15 i18n JSON files

**Summary: 0 P0 · 1 P1 · 4 P2 · 3 P3.**

The MIG-022 cascade is exceptionally clean for its scope (~7,400 i18n insertions + 4 cece commits + new history subsystem). No fabricated guards, no orphaned schema, no TODO/FIXME residue, no dead code introduced, no broken IPC registrations. The single P1 finding is by design (§B.5/§B.6 deferred). The main observable risks are documentation/comment-style accumulation, not functional drift.

---

## F1 — P1 — §B.4 IPCs have ZERO frontend consumers (deferred-by-design)

**Location:** `src-tauri/src/lib.rs:346-347`; `src-tauri/src/cece/history.rs:316-345, 359-420`

**Evidence:** Recursive grep across `src/**/*.{svelte,ts,js}` for `cece_get_note_history`, `cece_query_history`, `cece-get-note-history`, `cece-query-history` returns ZERO matches. The IPCs are registered in `lib.rs` and exported from `cece::history` but no frontend code calls `invoke('cece_get_note_history', …)` or `invoke('cece_query_history', …)`.

**Why this is documented as deferred-by-design:** §B.5 (Sight v3 overlay UI) and §B.6 are explicitly deferred per the cascade brief. `cece/mod.rs:34` and `cece/history.rs:37` document the deferral. Per LL-023 this is **planned drift**, not unplanned drift — but it remains drift in the sense that an IPC surface exists without a consumer.

**Recommended fix:** None; track in PJ for §B.5 ship. If §B.5 slips beyond two more migrations, downgrade to `#[allow(dead_code)]` on the IPC bodies and stop registering them in `lib.rs` until needed (avoids surfacing a non-functional command in `tauri-plugin-shell` introspection).

---

## F2 — P2 — `_suppress_unused` shim in synthesis.rs

**Location:** `src-tauri/src/cece/synthesis.rs:455-458`

**Evidence:**
```rust
// Stub for the unused import warning suppression; AxisAssignment is part
// of the public schema but not directly referenced in this file's logic.
#[allow(dead_code)]
fn _suppress_unused(_a: AxisAssignment) {}
```

`AxisAssignment` IS actually used in synthesis.rs at lines 475 and 485 (within the `tests` module), so the `_suppress_unused` shim is itself dead code at this point. It was pre-existing before MIG-022 (pre-cascade) but was preserved through the §D refactor without cleanup.

**Recommended fix:** Delete the `_suppress_unused` function and the corresponding line 20 import re-validation. Verify with `cargo build` that no unused-import warning fires (it shouldn't — `AxisAssignment` is used in tests).

---

## F3 — P2 — Magic threshold `0.80` for "secondary" candidate cutoff lacks explanatory link to source

**Location:** `src-tauri/src/cece/synthesis.rs:340-345`

**Evidence:**
```rust
let secondary: Vec<String> = sorted
    .iter()
    .skip(1)
    .filter(|(_, w)| *w / normalizer >= 0.80)
    .map(|(id, _)| id.clone())
    .collect();
```

The `0.80` threshold is well-commented (cites LC SCM H 180), but the threshold value itself is a magic number not extracted to a `const`. If LIS practice changes or if A/B testing finds 0.75 or 0.85 fits CECE better, callers must hunt for the literal in the function body.

**Recommended fix:** Extract to module-level `const SECONDARY_WEIGHT_THRESHOLD: f32 = 0.80;` with the LC SCM H 180 citation in the doc comment. Same treatment for the `2/3` ratio in `compute_regime` (lines 388-393): the comment explains why, but the literal `2` and `3` are scattered.

---

## F4 — P2 — `confidence_multiplier` magic numbers (1.0/0.7/0.4/0.0) lack provenance

**Location:** `src-tauri/src/cece/synthesis.rs:400-407`

**Evidence:**
```rust
fn confidence_multiplier(c: Confidence) -> f32 {
    match c {
        Confidence::High => 1.0,
        Confidence::Medium => 0.7,
        Confidence::Low => 0.4,
        Confidence::Abstain => 0.0,
    }
}
```

Four bare floats with no comment explaining the calibration choice (why 0.7 not 0.75? why 0.4 not 0.5?). Pre-MIG-022 baseline carried over without comment refresh during the §D refactor. Future tuning attempts will lack guidance on what "Medium" cataloger weight should mean.

**Recommended fix:** Add a one-paragraph doc comment explaining the calibration and link to the Reliability Profile spec (V3-§9). If these were arbitrary "first-cut" values, say so explicitly so future tuning is empowered.

---

## F5 — P2 — `compose_reasoning` builds an English fallback but it is never validated against `ReasoningTemplate`

**Location:** `src-tauri/src/cece/synthesis.rs:422-453`

**Evidence:** The function emits `(english_text, template)` together, but the English text and the template params are *not* automatically kept in sync. The English text uses `format!("Horizontal: {} ({}); Vertical: {} ({}). ", ...)` with full prose; the template is `compose.weighted_vote` with only `{ count }` param. If a future edit changes the English to add/remove fields, nothing will catch the drift between fallback and template.

**Recommended fix:** Either (a) generate the English fallback by interpolating the en.json value of the template (single source of truth) OR (b) add a unit test that, when the en.json template is loaded, rendering it yields the same string as the English fallback. The latter is faster to ship.

---

## F6 — P3 — Comment in vertical_taxonomy.rs references retired `classifier::source_definitions` module — accurate but indicates retired-code reference accumulation

**Location:** `src-tauri/src/sources/vertical_taxonomy.rs:18-21`

**Evidence:**
```rust
//! ... were originally derived mechanically from parent context in
//! `classifier::source_definitions`; that module was retired in MIG-022 §0
//! (the v2 three-tier classifier was replaced wholesale by the V3-§8 CECE
//! 6-cataloger ensemble, ...).
```

The comment is accurate and useful (explains why no rich descriptions ship). However it's a referent to a deleted symbol — readers may grep `source_definitions` and find only this comment + classifier/mod.rs. Acceptable as historical record; flag-only.

**Recommended fix:** None; leave as-is. If `classifier/mod.rs:16-17` is later cleaned up, update this comment to point at `MIG-022 §0` cleanup commit `d626ae7` for traceability.

---

## F7 — P3 — Dual maintenance of trigger SQL: backfill function inlines the same trigger DDL as `ensure_note_state_history_trigger`

**Location:** `src-tauri/src/cece/history.rs:107-133` (canonical) and `:238-264` (duplicated inside `backfill_initial_history`)

**Evidence:** The `CREATE TRIGGER ... note_state_history_au` SQL is written verbatim twice. The doc comment at line 237 acknowledges this ("inlined here so we stay inside the transaction"). Per CLAUDE.md "Don't" rule: **never duplicate working code by copy-pasting and adapting**. If the watched-field set evolves (e.g. §B.5 adds a `warrant_chain` column), both copies must be updated in lockstep.

**Recommended fix:** Extract trigger DDL into a `const TRIGGER_SQL: &str = "..."` at module level; call sites both use it via `tx.execute_batch(TRIGGER_SQL)`. Keeps single-source-of-truth without breaking the in-transaction execution.

---

## F8 — P3 — i18n: 137 keys present in en+ar but missing from 13 other locales (PRE-EXISTING — NOT MIG-022 drift)

**Location:** `src/lib/i18n/{de,es,fa,fr,he,hi,ja,ko,pt,ru,tr,ur,zh}.json` — all 13 missing the same set

**Evidence:** Key-by-key parity audit shows 137 keys missing in every non-en/non-ar locale, breaking down by prefix:
- `sources.review.*` (22 keys) — Source Review panel labels
- `sources.label.*` (12), `sources.description.*` (12) — taxonomy labels
- `inspector360.*` (~80 keys) — 360.3D matrix UI
- `classifierScan.*` (7), `taxonomyTreePicker.*` (7), `propertyEditor.stagePlaceholder` (1), etc.

**Verified MIG-022's own delta is clean:** all 7 `cece.reasoning.compose.*` and per-cataloger `cece.reasoning.*.both` keys ship in all 15 locales. The §A.4.a + §A.4.b + §E.2.b + §E.3.d cascades hit every locale (verified via `git diff --stat 547e558 -- src/lib/i18n/`: all 15 files +483 to +558 lines each).

**Conclusion:** This is **pre-existing drift** from MIG-021 (Inspector360, classifierScan) and MIG-021v2 (sources.review). MIG-022 inherited it but did not introduce it. Boss-test agents will see English fallbacks (the calls use `$t('key') || 'English fallback'` defensively in most spots, e.g. `SourceReviewPanel.svelte:884`).

**Recommended fix:** Spawn a separate cleanup task: backfill the 137 keys into the 13 locales. NOT a MIG-022 blocker.

---

**Cross-cluster verification performed:**
- All §B.x init_db migration calls present in `search.rs:1484-1520` in correct order
- §B.4 IPC registrations present in `lib.rs:345-347`
- `supersedes` typed-link added consistently across livePreview.ts (3 sites) + store.ts (4 sites)
- All `pe-ikhtilaf-*` CSS classes (5) referenced by templates (5 instance hits)
- Zero `tier1_*`/`tier2_*`/`tier3_*` references in production code (only `classifier/mod.rs:16-17` historical note + tests)
- Zero TODO/FIXME/XXX/HACK in any audited file
- Synthesis.rs `_suppress_unused` shim is the only minor residue

The cascade lands clean.
