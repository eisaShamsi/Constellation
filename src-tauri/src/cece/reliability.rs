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
}
