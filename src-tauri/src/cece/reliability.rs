//! MIG-021v3 V3-§1 — Per-Library cataloger reliability tracking.
//!
//! Each cataloger's accuracy is tracked per axis per Library at
//! `<library>/.constellation/cataloger_reliability.json`. Empty file
//! (or missing file) = uniform weights. Updates atomically on every
//! correction-log entry.
//!
//! Per Architect §3.3: per-Library calibration captures vault-specific
//! cataloger strengths (Graph Cataloger excels on densely-linked
//! Libraries; Linguistic Cataloger excels on Arabic-heavy Libraries).
//!
//! Per Architect §10 invariant 9: per-Library calibration is
//! per-Library — no cross-Library data leakage.

use crate::cece::cataloger::Axis;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// One cataloger's accuracy histogram on one axis in one Library.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccuracyHistogram {
    pub correct: u64,
    pub wrong: u64,
}

impl AccuracyHistogram {
    pub fn total(&self) -> u64 {
        self.correct + self.wrong
    }

    /// Accuracy ratio. Returns `None` when the cataloger has fewer than
    /// `min_samples` observations on this Library/axis (uniform weight
    /// in synthesis until enough data accumulates).
    pub fn ratio(&self, min_samples: u64) -> Option<f32> {
        let total = self.total();
        if total < min_samples {
            None
        } else {
            Some(self.correct as f32 / total as f32)
        }
    }
}

/// Whole-Library reliability profile. Loaded once per classification
/// pass; written atomically after each correction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliabilityProfile {
    /// (cataloger_name, axis) → histogram. Stored as nested HashMap
    /// for JSON friendliness (axis as string key inside cataloger map).
    pub stats: HashMap<String, HashMap<String, AccuracyHistogram>>,
}

impl ReliabilityProfile {
    /// Lookup the histogram for a (cataloger, axis) pair. Returns a
    /// fresh empty histogram if not present.
    pub fn get(&self, cataloger: &str, axis: Axis) -> AccuracyHistogram {
        self.stats
            .get(cataloger)
            .and_then(|m| m.get(axis.as_str()))
            .cloned()
            .unwrap_or_default()
    }

    /// Increment the (cataloger, axis) counter.
    pub fn record(&mut self, cataloger: &str, axis: Axis, was_correct: bool) {
        let by_axis = self
            .stats
            .entry(cataloger.to_string())
            .or_default();
        let h = by_axis.entry(axis.as_str().to_string()).or_default();
        if was_correct {
            h.correct += 1;
        } else {
            h.wrong += 1;
        }
    }
}

fn reliability_path(library_path: &str) -> PathBuf {
    let mut p = PathBuf::from(library_path);
    p.push(".constellation");
    p.push("cataloger_reliability.json");
    p
}

/// Load a Library's reliability profile, or return a default (empty)
/// profile if the file doesn't exist or fails to parse. Failures are
/// logged but never propagated — uniform weights are always a valid
/// fallback.
///
/// V3-§8.r4.1 also sweeps any orphaned .tmp files left over from a
/// previous kill-mid-write (audit P1 .tmp accumulation finding).
pub fn load_or_default(library_path: &str) -> ReliabilityProfile {
    sweep_tmp_orphans(library_path);
    let path = reliability_path(library_path);
    if !path.exists() {
        return ReliabilityProfile::default();
    }
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            eprintln!(
                "[reliability] {:?} failed to parse: {} — falling back to default",
                path, e
            );
            ReliabilityProfile::default()
        }),
        Err(e) => {
            eprintln!(
                "[reliability] {:?} failed to read: {} — falling back to default",
                path, e
            );
            ReliabilityProfile::default()
        }
    }
}

/// Atomic write via cross-platform NamedTempFile::persist. Best-effort:
/// write failures are logged but never abort the user's save flow
/// (the next correction will overwrite stale data).
///
/// V3-§8.r4.1 fix (audit P1.2): the original implementation used
/// `std::fs::write` to a `.json.tmp` then `std::fs::rename` to the
/// final path. On Windows pre-NTFS-fixup, `std::fs::rename` to an
/// EXISTING destination fails outright — meaning the per-Library
/// reliability JSON would never update after the first write, and
/// every subsequent `record_correction` would silently fail. The
/// `tempfile::NamedTempFile::persist` method handles the platform
/// difference correctly (uses `MoveFileExW` with
/// `MOVEFILE_REPLACE_EXISTING` on Windows).
pub fn save(library_path: &str, profile: &ReliabilityProfile) {
    let path = reliability_path(library_path);
    let dir = match path.parent() {
        Some(p) => p,
        None => {
            eprintln!("[reliability] no parent dir for {:?}", path);
            return;
        }
    };
    if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("[reliability] mkdir {:?} failed: {}", dir, e);
        return;
    }
    let serialized = match serde_json::to_string_pretty(profile) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[reliability] serialize failed: {}", e);
            return;
        }
    };
    // tempfile::NamedTempFile creates a uniquely-named temp file in
    // the target directory, so the rename is same-filesystem and
    // atomic on POSIX + correctly REPLACES on Windows.
    let mut tmp = match tempfile::Builder::new()
        .prefix(".cataloger_reliability.")
        .suffix(".tmp")
        .tempfile_in(dir)
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[reliability] tempfile create in {:?} failed: {}", dir, e);
            return;
        }
    };
    use std::io::Write;
    if let Err(e) = tmp.write_all(serialized.as_bytes()) {
        eprintln!("[reliability] tmp write failed: {}", e);
        return;
    }
    if let Err(e) = tmp.as_file_mut().sync_all() {
        // Not fatal — durability is best-effort.
        eprintln!("[reliability] tmp sync failed (non-fatal): {}", e);
    }
    if let Err(e) = tmp.persist(&path) {
        eprintln!("[reliability] persist failed: {}", e);
    }
}

/// Sweep up any orphaned `.tmp` reliability files in a Library's
/// `.constellation/` directory left over from a previous kill-mid-write.
/// V3-§8.r4.1 fix (audit P1, .tmp orphan accumulation). Best-effort:
/// errors logged but never propagated. Called from `load_or_default`
/// so the cleanup runs lazily on first read.
fn sweep_tmp_orphans(library_path: &str) {
    let mut dir = std::path::PathBuf::from(library_path);
    dir.push(".constellation");
    if !dir.exists() {
        return;
    }
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(".cataloger_reliability.") && name_str.ends_with(".tmp") {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Convenience: increment + save in one call. The atomic-rename pattern
/// inside `save` makes this safe to call from a hot path.
pub fn record_correction(
    library_path: &str,
    cataloger: &str,
    axis: Axis,
    was_correct: bool,
) {
    let mut p = load_or_default(library_path);
    p.record(cataloger, axis, was_correct);
    save(library_path, &p);
}

/// V3-§10.A — Read the active Library's reliability profile for UI
/// display. Returns the entire ReliabilityProfile struct (which
/// serializes to the same JSON shape as the on-disk file). The Source
/// Review Settings panel converts the result to per-cataloger
/// per-axis accuracy display rows.
///
/// "Active Library" is resolved as: if a note is currently open in
/// NotePane, the Library that contains that note path. Otherwise,
/// fall back to the first Library returned by `list_libraries`.
/// Returns an empty default profile (no panic, no error) when no
/// reliability JSON exists for the resolved Library yet — the UI
/// renders an empty-state message in that case.
///
/// `note_path` is optional; the frontend passes the active note path
/// when it has one, or null/empty otherwise.
#[tauri::command]
pub fn cece_get_reliability_for_active_library(
    app: tauri::AppHandle,
    note_path: Option<String>,
) -> Result<ReliabilityProfile, String> {
    let libs = crate::libraries::list_libraries(app.clone());
    if libs.is_empty() {
        return Ok(ReliabilityProfile::default());
    }
    // First try resolving via the active note path (most precise).
    let lib_root = match note_path.as_deref().filter(|p| !p.is_empty()) {
        Some(np) => {
            let pairs: Vec<(String, String)> = libs
                .iter()
                .cloned()
                .map(|l| (l.id, l.path))
                .collect();
            crate::classifier::correction_log::library_root_for_note(&pairs, np)
                // Fall back to first Library when the note isn't under any
                // registered Library (e.g. orphaned notes).
                .or_else(|| libs.first().map(|l| l.path.clone()))
        }
        None => libs.first().map(|l| l.path.clone()),
    };
    match lib_root {
        Some(root) => Ok(load_or_default(&root)),
        None => Ok(ReliabilityProfile::default()),
    }
}

/// V3-§10.A companion — return the resolved Library path so the UI can
/// label the calibration view ("Active Library: arab-literature"). Same
/// resolution logic as the reliability getter above.
#[tauri::command]
pub fn cece_get_active_library_root(
    app: tauri::AppHandle,
    note_path: Option<String>,
) -> Result<Option<String>, String> {
    let libs = crate::libraries::list_libraries(app.clone());
    if libs.is_empty() {
        return Ok(None);
    }
    let lib_root = match note_path.as_deref().filter(|p| !p.is_empty()) {
        Some(np) => {
            let pairs: Vec<(String, String)> = libs
                .iter()
                .cloned()
                .map(|l| (l.id, l.path))
                .collect();
            crate::classifier::correction_log::library_root_for_note(&pairs, np)
                .or_else(|| libs.first().map(|l| l.path.clone()))
        }
        None => libs.first().map(|l| l.path.clone()),
    };
    Ok(lib_root)
}

/// V3-§9.C — Update a Library's per-cataloger reliability based on
/// the user's final correction.
///
/// Reads the composite_json blob (the per-cataloger trails the engine
/// produced at classification time), iterates each cataloger that
/// voiced on this axis, and marks "correct" if its primary matched
/// the user's final pick — "wrong" otherwise. One file write per call
/// (atomic via `save`). Catalogers that abstained (didn't voice on
/// this axis) don't get a counter bump in either direction — silence
/// is neither right nor wrong.
///
/// Best-effort: malformed JSON / missing fields → no-op (corrections
/// still log via correction_log; reliability just doesn't update).
///
/// `axis_str` is "horizontal" or "vertical" (the string form used by
/// the existing IPC layer; converted internally to the Axis enum).
pub fn update_reliability_from_correction(
    library_path: &str,
    composite_json: &str,
    axis_str: &str,
    user_pick: &[String],
) {
    let axis = match axis_str {
        "horizontal" => Axis::Horizontal,
        "vertical" => Axis::Vertical,
        _ => return,
    };
    let composite: serde_json::Value = match serde_json::from_str(composite_json) {
        Ok(v) => v,
        Err(_) => return,
    };
    let trails = match composite.get("per_cataloger_trails").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return,
    };
    let mut profile = load_or_default(library_path);
    let mut any_update = false;
    for trail in trails {
        let cataloger_name = match trail.get("cataloger").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => continue,
        };
        let voiced = trail
            .get("voiced_opinion")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !voiced {
            continue; // silence is neither correct nor wrong
        }
        let assignments = trail
            .get(axis_str)
            .and_then(|v| v.as_array());
        let assignments = match assignments {
            Some(a) => a,
            None => continue,
        };
        // Find this cataloger's primary on this axis.
        let primary_id = assignments.iter().find_map(|a| {
            if a.get("primary").and_then(|p| p.as_bool()).unwrap_or(false) {
                a.get("id").and_then(|i| i.as_str())
            } else {
                None
            }
        });
        let primary_id = match primary_id {
            Some(id) => id,
            None => continue, // no primary on this axis
        };
        let was_correct = user_pick.iter().any(|p| p == primary_id);
        profile.record(cataloger_name, axis, was_correct);
        any_update = true;
    }
    if any_update {
        save(library_path, &profile);
    }
}

/// Compute a synthesis weight for a cataloger on an axis. Uniform
/// weight (1.0) until the cataloger has at least `MIN_SAMPLES`
/// observations on this Library/axis. After that, weight = ratio
/// (clamped to [0.1, 1.0] so even a poorly-performing cataloger still
/// has a tiny voice — per Architect §3.2 weighted-vote design).
pub const MIN_SAMPLES_FOR_WEIGHTING: u64 = 20;

pub fn weight_for(profile: &ReliabilityProfile, cataloger: &str, axis: Axis) -> f32 {
    let h = profile.get(cataloger, axis);
    match h.ratio(MIN_SAMPLES_FOR_WEIGHTING) {
        Some(r) => r.clamp(0.1, 1.0),
        None => 1.0, // uniform weight until enough samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_weight_until_enough_samples() {
        let mut p = ReliabilityProfile::default();
        for _ in 0..(MIN_SAMPLES_FOR_WEIGHTING - 1) {
            p.record("linguistic", Axis::Horizontal, true);
        }
        // Below threshold — uniform weight.
        assert_eq!(weight_for(&p, "linguistic", Axis::Horizontal), 1.0);

        p.record("linguistic", Axis::Horizontal, true);
        // Now at threshold — perfect accuracy → weight = 1.0 (clamp upper).
        assert_eq!(weight_for(&p, "linguistic", Axis::Horizontal), 1.0);
    }

    #[test]
    fn accuracy_below_threshold_clamps_to_floor() {
        let mut p = ReliabilityProfile::default();
        for _ in 0..MIN_SAMPLES_FOR_WEIGHTING {
            p.record("graph", Axis::Vertical, false);
        }
        // 0% accuracy → clamped to 0.1 floor (still has a voice).
        assert_eq!(weight_for(&p, "graph", Axis::Vertical), 0.1);
    }

    #[test]
    fn round_trip_serialization() {
        let mut p = ReliabilityProfile::default();
        p.record("linguistic", Axis::Horizontal, true);
        p.record("linguistic", Axis::Horizontal, false);
        p.record("graph", Axis::Vertical, true);
        let json = serde_json::to_string(&p).unwrap();
        let p2: ReliabilityProfile = serde_json::from_str(&json).unwrap();
        let h = p2.get("linguistic", Axis::Horizontal);
        assert_eq!(h.correct, 1);
        assert_eq!(h.wrong, 1);
    }

    // ─── V3-§9.C — update_reliability_from_correction wiring tests ───
    // The Architect doc framed Phase C as "schema migration" but
    // auditing the actual code revealed the per-axis schema was
    // already there; the real gap was that reliability updates were
    // never wired into the correction flows. These tests verify the
    // new helper function correctly bumps per-cataloger per-axis
    // counters from a composite_json blob + user pick.

    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    static TEST_LIB_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn unique_test_library() -> tempfile::TempDir {
        // Each test gets a fresh tempdir so writes don't cross-pollute.
        let n = TEST_LIB_COUNTER.fetch_add(1, AtomicOrdering::SeqCst);
        tempfile::Builder::new()
            .prefix(&format!("v3p9c-test-{}-", n))
            .tempdir()
            .expect("tempdir create")
    }

    fn sample_composite_json() -> &'static str {
        // Two voicing catalogers — one agrees with synthesis primary
        // (testimony/authoritative), one dissents (testimony/direct-witness).
        // One silent cataloger (graph) which should NOT get a counter bump.
        r#"{
          "horizontal": {"primary": "testimony/authoritative", "regime": "strong_majority"},
          "vertical": {"primary": "epistemic-states/doubt", "regime": "unanimous"},
          "per_cataloger_trails": [
            { "cataloger": "linguistic", "voiced_opinion": true,
              "horizontal": [{"id":"testimony/authoritative","primary":true,"weight":0.9}],
              "vertical":   [{"id":"epistemic-states/doubt","primary":true,"weight":0.85}] },
            { "cataloger": "structural", "voiced_opinion": true,
              "horizontal": [{"id":"testimony/direct-witness","primary":true,"weight":0.7}],
              "vertical":   [{"id":"epistemic-states/doubt","primary":true,"weight":0.85}] },
            { "cataloger": "graph", "voiced_opinion": false,
              "horizontal": [],
              "vertical":   [] }
          ]
        }"#
    }

    #[test]
    fn v3_p9c_correction_bumps_correct_cataloger_horizontal() {
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        update_reliability_from_correction(
            &lib,
            sample_composite_json(),
            "horizontal",
            &["testimony/authoritative".to_string()],
        );
        let p = load_or_default(&lib);
        // Linguistic voted testimony/authoritative → correct.
        let ling = p.get("linguistic", Axis::Horizontal);
        assert_eq!(ling.correct, 1);
        assert_eq!(ling.wrong, 0);
        // Structural voted testimony/direct-witness → wrong.
        let struc = p.get("structural", Axis::Horizontal);
        assert_eq!(struc.correct, 0);
        assert_eq!(struc.wrong, 1);
    }

    #[test]
    fn v3_p9c_silent_cataloger_does_not_get_counter_bump() {
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        update_reliability_from_correction(
            &lib,
            sample_composite_json(),
            "horizontal",
            &["testimony/authoritative".to_string()],
        );
        let p = load_or_default(&lib);
        // Graph was silent (voiced_opinion: false) — no counter bump
        // in either direction. Silence is neither right nor wrong.
        let graph = p.get("graph", Axis::Horizontal);
        assert_eq!(graph.correct, 0);
        assert_eq!(graph.wrong, 0);
    }

    #[test]
    fn v3_p9c_axis_specific_no_cross_pollution() {
        // Bump on horizontal-axis correction MUST NOT touch
        // vertical-axis counters and vice versa.
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        update_reliability_from_correction(
            &lib,
            sample_composite_json(),
            "horizontal",
            &["testimony/authoritative".to_string()],
        );
        let p = load_or_default(&lib);
        // Linguistic vertical counters must stay at 0 (only horizontal
        // axis was corrected).
        let ling_v = p.get("linguistic", Axis::Vertical);
        assert_eq!(ling_v.correct, 0);
        assert_eq!(ling_v.wrong, 0);
        let struc_v = p.get("structural", Axis::Vertical);
        assert_eq!(struc_v.correct, 0);
        assert_eq!(struc_v.wrong, 0);
    }

    #[test]
    fn v3_p9c_vertical_correction_bumps_vertical_only() {
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        update_reliability_from_correction(
            &lib,
            sample_composite_json(),
            "vertical",
            &["epistemic-states/doubt".to_string()],
        );
        let p = load_or_default(&lib);
        // Both linguistic + structural voted doubt on vertical → both correct.
        assert_eq!(p.get("linguistic", Axis::Vertical).correct, 1);
        assert_eq!(p.get("structural", Axis::Vertical).correct, 1);
        // Horizontal counters unchanged.
        assert_eq!(p.get("linguistic", Axis::Horizontal).correct, 0);
        assert_eq!(p.get("linguistic", Axis::Horizontal).wrong, 0);
    }

    #[test]
    fn v3_p9c_malformed_json_is_no_op() {
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        update_reliability_from_correction(
            &lib,
            "not valid json {{{",
            "horizontal",
            &["testimony/authoritative".to_string()],
        );
        let p = load_or_default(&lib);
        // No file written → empty default profile.
        assert_eq!(p.stats.len(), 0);
    }

    #[test]
    fn v3_p9c_unknown_axis_is_no_op() {
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        update_reliability_from_correction(
            &lib,
            sample_composite_json(),
            "diagonal", // not a real axis
            &["testimony/authoritative".to_string()],
        );
        let p = load_or_default(&lib);
        assert_eq!(p.stats.len(), 0);
    }

    #[test]
    fn v3_p9c_existing_profile_accumulates() {
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        // Apply two corrections — second one should accumulate, not replace.
        update_reliability_from_correction(
            &lib,
            sample_composite_json(),
            "horizontal",
            &["testimony/authoritative".to_string()],
        );
        update_reliability_from_correction(
            &lib,
            sample_composite_json(),
            "horizontal",
            &["testimony/authoritative".to_string()],
        );
        let p = load_or_default(&lib);
        let ling = p.get("linguistic", Axis::Horizontal);
        assert_eq!(ling.correct, 2);
        let struc = p.get("structural", Axis::Horizontal);
        assert_eq!(struc.wrong, 2);
    }

    // ─── V3-§9.C.2 — dual-axis correction tests ───
    // The IPC `cece_record_correction_for_card` calls
    // `update_reliability_from_correction` twice (once per axis) from
    // a single composite_json snapshot. These tests verify the dual-
    // axis behavior by mimicking what the IPC does without needing a
    // full Tauri AppHandle.

    fn dual_axis_record(
        library_path: &str,
        composite_json: &str,
        horizontal_pick: &[String],
        vertical_pick: &[String],
    ) {
        // Mirrors cece_record_correction_for_card's body (without
        // the Tauri AppHandle + library lookup wrapper).
        if !horizontal_pick.is_empty() {
            update_reliability_from_correction(
                library_path,
                composite_json,
                "horizontal",
                horizontal_pick,
            );
        }
        if !vertical_pick.is_empty() {
            update_reliability_from_correction(
                library_path,
                composite_json,
                "vertical",
                vertical_pick,
            );
        }
    }

    #[test]
    fn v3_p9c2_dual_axis_accept_updates_both_axes() {
        // Boss-test 2026-05-11 Stage 3 surfaced this gap: dual-axis
        // Accept only updated horizontal because the second per-axis
        // IPC found the suggestion row already cleared. This test
        // verifies the new dual-axis call updates BOTH axes from a
        // single snapshot.
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        dual_axis_record(
            &lib,
            sample_composite_json(),
            &["testimony/authoritative".to_string()],
            &["epistemic-states/doubt".to_string()],
        );
        let p = load_or_default(&lib);
        // Horizontal: linguistic voted authoritative (correct),
        // structural voted direct-witness (wrong).
        assert_eq!(p.get("linguistic", Axis::Horizontal).correct, 1);
        assert_eq!(p.get("structural", Axis::Horizontal).wrong, 1);
        // Vertical: both voicing catalogers voted doubt (both correct).
        assert_eq!(p.get("linguistic", Axis::Vertical).correct, 1);
        assert_eq!(p.get("structural", Axis::Vertical).correct, 1);
        // The PRE-V3-§9.C.2 bug would show vertical counters at 0.
        // After fix, they're correctly 1.
    }

    #[test]
    fn v3_p9c2_horizontal_only_pick_updates_horizontal_only() {
        // When a single-axis caller (or a Split-on-one-axis disambig
        // with no other-axis settled value) passes empty vertical_pick,
        // only horizontal counters should bump.
        let dir = unique_test_library();
        let lib = dir.path().to_string_lossy().to_string();
        dual_axis_record(
            &lib,
            sample_composite_json(),
            &["testimony/authoritative".to_string()],
            &[], // empty vertical pick
        );
        let p = load_or_default(&lib);
        assert_eq!(p.get("linguistic", Axis::Horizontal).correct, 1);
        assert_eq!(p.get("structural", Axis::Horizontal).wrong, 1);
        // Vertical counters MUST stay at 0 — empty pick = no update.
        assert_eq!(p.get("linguistic", Axis::Vertical).correct, 0);
        assert_eq!(p.get("linguistic", Axis::Vertical).wrong, 0);
        assert_eq!(p.get("structural", Axis::Vertical).correct, 0);
        assert_eq!(p.get("structural", Axis::Vertical).wrong, 0);
    }
}
