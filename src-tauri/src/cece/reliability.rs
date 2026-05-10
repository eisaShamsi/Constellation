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
pub fn load_or_default(library_path: &str) -> ReliabilityProfile {
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

/// Atomic write via temp-file rename. Best-effort: write failures are
/// logged but never abort the user's save flow (the next correction
/// will overwrite stale data). Per Architect §13 risk mitigation.
pub fn save(library_path: &str, profile: &ReliabilityProfile) {
    let path = reliability_path(library_path);
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("[reliability] mkdir {:?} failed: {}", parent, e);
            return;
        }
    }
    let serialized = match serde_json::to_string_pretty(profile) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[reliability] serialize failed: {}", e);
            return;
        }
    };
    let tmp_path = path.with_extension("json.tmp");
    if let Err(e) = fs::write(&tmp_path, serialized) {
        eprintln!("[reliability] tmp write {:?} failed: {}", tmp_path, e);
        return;
    }
    if let Err(e) = fs::rename(&tmp_path, &path) {
        eprintln!("[reliability] rename failed: {}", e);
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
