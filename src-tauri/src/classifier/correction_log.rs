//! MIG-021v2 §1G2' — Append-only correction log.
//!
//! Every time a user overrides a classifier suggestion (via the Source
//! Review panel's Reject / Edit-and-Save flows, or via PropertyEditor
//! manual setting), we append one NDJSON line to
//! `<library>/.constellation/classifier_corrections.jsonl` so the data
//! is never lost.
//!
//! This is the substrate for two future capabilities:
//!   1. Active learning — periodic retraining of the embedding head on
//!      accumulated overrides.
//!   2. Telemetry-style honesty — letting the user audit what the
//!      classifier was wrong about.
//!
//! The file is per-library and lives next to other Constellation state
//! (libraries.json, search.db). Append-only — no deletion API; the
//! file IS the audit trail.

use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
pub struct CorrectionEntry {
    /// Unix epoch seconds.
    pub ts: i64,
    pub note_path: String,
    /// "horizontal" | "vertical".
    pub axis: String,
    /// What the classifier suggested (sorted-as-stored taxonomy IDs).
    pub predicted: Vec<String>,
    /// What the user committed.
    pub corrected: Vec<String>,
    /// 1 = rules, 2 = embedding, 3 = LLM, 0 = unknown / not classified.
    pub tier_used: i64,
}

/// Append a correction event. Best-effort: failures are logged but
/// never abort the user's save flow. The log is a write-once
/// observation, not a load-bearing primary.
pub fn log_correction(
    library_path: &str,
    note_path: &str,
    axis: &str,
    predicted: &[String],
    corrected: &[String],
    tier_used: i64,
) {
    let entry = CorrectionEntry {
        ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        note_path: note_path.to_string(),
        axis: axis.to_string(),
        predicted: predicted.to_vec(),
        corrected: corrected.to_vec(),
        tier_used,
    };
    let line = match serde_json::to_string(&entry) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[correction_log] serialize failed: {}", e);
            return;
        }
    };

    let mut path = PathBuf::from(library_path);
    path.push(".constellation");
    if let Err(e) = std::fs::create_dir_all(&path) {
        eprintln!("[correction_log] mkdir {:?} failed: {}", path, e);
        return;
    }
    path.push("classifier_corrections.jsonl");

    let res = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| writeln!(f, "{}", line));
    if let Err(e) = res {
        eprintln!("[correction_log] write {:?} failed: {}", path, e);
    }
}

/// Find the library path that contains a given note. Used by the
/// IPC handlers to know where to drop the correction-log file.
/// Returns None if the note isn't under any registered library
/// (e.g. orphan path / path resolution failure).
pub fn library_root_for_note(
    libraries: &[(String, String)], // (library_id, library_path)
    note_path: &str,
) -> Option<String> {
    let normalized = note_path.replace('\\', "/");
    libraries
        .iter()
        .filter(|(_, p)| {
            let lp = p.replace('\\', "/");
            normalized.starts_with(&lp)
        })
        // Pick the longest-prefix match so nested libraries resolve to
        // the most-specific one.
        .max_by_key(|(_, p)| p.len())
        .map(|(_, p)| p.clone())
}
