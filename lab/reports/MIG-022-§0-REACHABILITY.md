# MIG-022 §0.1 — Legacy classifier reachability analysis

**Date:** 2026-05-11
**Phase:** Build §0.1 (Cluster F, audit F1)
**Method:** Grep every `pub` symbol in `src-tauri/src/classifier/*.rs` + check callers outside `classifier/` + check `#[tauri::command]` registration in `lib.rs` + check frontend `invoke()` callers
**Verdict:** **988 LoC of dead code confirmed.** Three files deletable in their entirety. No production callers; only own internal tests.

---

## 1 · Reachability table

### 1.1 — REACHABLE (production callers exist) — KEEP

| Symbol | File | Reachable from |
|---|---|---|
| `classifier_suggest_for_note` | `mod.rs:36` | `lib.rs:333` (Tauri command), `SourceReviewPanel.svelte:456,604` (frontend invoke) |
| `cece_record_correction_for_card` | `mod.rs:230` | `lib.rs:341` (Tauri command), V3-§9.C.2 frontend Accept flow |
| `cece_resolve_disambiguation` | `mod.rs:288` | `lib.rs:339` (Tauri command), Sibling Disambig chip pick |
| `scan_job::ScanState` | `scan_job.rs:25` | `lib.rs:260` (`.manage(...)` registers as Tauri-managed state) |
| `scan_job::ScanStatus` | `scan_job.rs:43` | Returned by `classifier_scan_status`; consumed by frontend `ScanStatus` type |
| `scan_job::classifier_scan_start` | `scan_job.rs:95` | `lib.rs:335`, `+layout.svelte:2090` (V3-§10.A on-startup), `SettingsModal.svelte:227` (manual button) |
| `scan_job::classifier_scan_cancel` | `scan_job.rs:85` | `lib.rs:336`, `ClassifierScanProgressStrip.svelte:50` |
| `scan_job::classifier_scan_status` | `scan_job.rs:66` | `lib.rs:337`, `ClassifierScanProgressStrip.svelte:86` (polling), `SettingsModal.svelte:219` |
| `correction_log::CorrectionEntry` | `correction_log.rs:26` | Used internally by `log_correction`; carried in correction-log JSON |
| `correction_log::log_correction` | `correction_log.rs:43` | `sources/mod.rs:650`, `sources/mod.rs:1096` (Accept flow logs each correction) |
| `correction_log::library_root_for_note` | `correction_log.rs:102` | `sources/mod.rs:648`, `sources/mod.rs:1094`, `cece/reliability.rs:249,281`, `classifier/mod.rs:207` (4 separate call sites in 3 modules — heavily reachable) |

**Verdict for KEEP set:** all symbols above are tied to active IPC surfaces or to multi-call-site internal helpers. No deletion candidate.

### 1.2 — DEAD (no production callers; only internal tests) — DELETE

| Symbol | File | Reachability evidence |
|---|---|---|
| `source_definitions::ClassifierCandidate` | `source_definitions.rs:29` | Only consumer is `tier1_embedding.rs` (line 27 `use super::source_definitions::{...}` + body uses) |
| `source_definitions::ClassifierAxis` | `source_definitions.rs:36` | Same as above |
| `source_definitions::SOURCE_DEFINITIONS` | `source_definitions.rs:55` | Only consumer is `source_definitions.rs` itself + `tier1_embedding.rs` (transitively via build_classifier_candidates) |
| `source_definitions::HORIZONTAL_LEAF_HINTS` | `source_definitions.rs:109` | Same as above |
| `source_definitions::build_classifier_candidates` | `source_definitions.rs:207` | Only consumer is `tier1_embedding.rs:189` |
| `tier1_embedding::classify` | `tier1_embedding.rs:52` | NO production callers anywhere in `src-tauri/src/`. Comment in `mod.rs:64` claims Linguistic/Structural/Semantic catalogers consume it via the wiring layer; **verified false** by grep — `cece/wiring.rs` and the catalogers have ZERO references to `tier1_embedding` or `tier1_rules` |
| `tier1_embedding::pool_counts_for_test` | `tier1_embedding.rs:288` | Pub-but-unused; no callers anywhere (grep returned only the declaration) |
| `tier1_rules::Tier1Result` | `tier1_rules.rs:140` | Only consumer is `tier1_rules.rs` itself (test functions at lines 282-317) |
| `tier1_rules::classify_tier1` | `tier1_rules.rs:161` | Only callers are own tests in `tier1_rules.rs` (lines 282, 297, 304, 310, 317) |

**Verdict for DELETE set:** the v2 three-tier classifier (rules + embedding + LLM) was replaced wholesale by the V3-§8 CECE 6-cataloger ensemble. The bodies live on, the tests pass, but no production caller exists. The misleading comment at `mod.rs:64` led the V3-§11 audit drift agent to under-investigate.

---

## 2 · The misleading mod.rs comment

`classifier/mod.rs:60-67`:

```rust
// 3. MIG-021v3 V3-§8 — CECE Cataloger Ensemble.
//    Build the production-wired orchestrator (six catalogers,
//    cost-ordered, with embed/lookup/inference functions wired
//    to the real backends). Run the ensemble against this note.
//
//    The v2 three-tier classifier (tier1_rules + tier1_embedding)
//    is preserved — its outputs are now consumed by the Linguistic,
//    Structural, and Semantic catalogers via the wiring layer.
//    See lab/reports/MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md §6.
```

**This is wrong.** The wiring layer (`cece/wiring.rs`) does NOT consume `tier1_rules` or `tier1_embedding` — verified by `Grep "tier1_embedding|tier1_rules" src-tauri/src/cece/` returning zero hits.

The comment was likely written in the V3-§8 plan phase as the intended design but never updated when the actual implementation took a different path (the catalogers each implement their own logic from scratch, without piggybacking on the v2 tier1 modules).

§0.2 removes this comment along with the dead modules.

---

## 3 · Deletion plan (§0.2)

### 3.1 Files to delete entirely

- `src-tauri/src/classifier/source_definitions.rs` (323 LoC)
- `src-tauri/src/classifier/tier1_embedding.rs` (345 LoC)
- `src-tauri/src/classifier/tier1_rules.rs` (320 LoC)

**Total: 988 LoC removed.**

### 3.2 `mod.rs` edits

In `src-tauri/src/classifier/mod.rs`:

- **Remove** lines 16-17:
  ```rust
  mod source_definitions;
  mod tier1_embedding;
  ```
- **Remove** line 20:
  ```rust
  pub mod tier1_rules;
  ```
- **Update** the comment block at lines 59-67 to remove the misleading `tier1_*` claim. The new comment will note CECE replaced the v2 three-tier classifier wholesale.
- **Update** the comment at line 556 (`// the children directly via super::source_definitions / super::tier1_embedding.`) to remove the dead reference.

### 3.3 Files to KEEP (per §1.1)

- `src-tauri/src/classifier/mod.rs` (with edits per §3.2)
- `src-tauri/src/classifier/scan_job.rs` (3 active IPCs + ScanState)
- `src-tauri/src/classifier/correction_log.rs` (used by sources/ + cece/)

### 3.4 Documentation updates

- `src-tauri/src/sources/vertical_taxonomy.rs:18` — module-level comment references `classifier::source_definitions`. Update to remove the dead reference (the description text it points to no longer exists).
- `src-tauri/src/sources/mod.rs:179` — comment references `classifier::write_suggestions_with_composite`. Verify whether `write_suggestions_with_composite` still exists (Plan-phase verification — quick grep before §0.2 commit).

### 3.5 Verification clauses

After §0.2 deletion:

1. `cargo build --package constellation-tauri` succeeds.
2. `cargo test --package constellation-tauri --lib` succeeds. Test count unchanged minus the dead-module's own tests:
   - `tier1_rules.rs` had 5 tests (lines 282, 297, 304, 310, 317) — these go away; expected -5 tests.
   - `tier1_embedding.rs` had no tests (per the `pool_counts_for_test` helper grep — no `#[test]` callers).
   - `source_definitions.rs` had 2 tests (`assert_eq!(SOURCE_DEFINITIONS.len(), 11)`, `assert_eq!(HORIZONTAL_LEAF_HINTS.len(), 41)`, plus the `build_classifier_candidates` smoke test at line 298) — these go away; expected -3 tests.
   - **Net expected:** total tests minus 8 (5+3). Cece tests stay at 92.
3. `cargo clippy` — verify no new warnings.
4. NSIS rebuild succeeds (deferred to §N close-out commit; not strictly needed at §0).

---

## 4 · Risk assessment

### 4.1 Risks

- **Risk 1:** A subsystem somewhere consumes a deleted symbol via `crate::classifier::*` and grep missed it. **Mitigation:** the grep in §1 was over the entire `src-tauri/src/` tree — exhaustive for Rust call sites. Plus `cargo build` will fail loudly if any caller exists that grep missed.
- **Risk 2:** Frontend invokes a `#[tauri::command]` from one of the deleted files. **Mitigation:** verified — none of the 3 deleted files declare any `#[tauri::command]` (only `mod.rs`, `scan_job.rs` do, and both are KEEP).
- **Risk 3:** A test depends on the deleted modules. **Mitigation:** §3.5 verification expects -8 tests (the deleted modules' own tests); any unexpected test failure is a real bug to investigate before commit.
- **Risk 4:** External tooling (a separate scratch script, a Lab notebook, a doc example) references the dead code. **Mitigation:** scope of MIG-022 §0 is the Rust crate only; if external scripts break, they were already silently broken (the dead code wasn't producing useful output — the production path goes through CECE).

### 4.2 Boss-test impact: NONE

Internal cleanup. No user-visible behavior change. Source Review classification continues to work exactly as it does today (via the CECE ensemble, unchanged).

---

## 5 · Recommendation

**Proceed with §0.2 deletion as scoped above.** Delete `tier1_embedding.rs`, `tier1_rules.rs`, `source_definitions.rs` in full; edit `mod.rs` per §3.2; update the two doc references per §3.4; verify per §3.5; commit.

---

*Reachability analysis filed at MIG-022 Phase 3 §0.1. Awaiting Eisa OK to proceed to §0.2 deletion + commit.*
