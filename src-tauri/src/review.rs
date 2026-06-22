//! Review Pulse — Cognitive Engine Phase 7 (نبض المراجعة).
//!
//! Spaced resurfacing and staleness monitoring. Not flashcards — knowledge
//! revisit prompts: "Still relevant? Link it? Archive it?"
//!
//! 3 Modes:
//!   1. Spaced Resurfacing: expanding intervals (1→3→7→14→30 days), strata-weighted
//!   2. Staleness Scan: Evergreen/Canonical untouched while domain has new notes
//!   3. Mental Model Checkpoints: #assumption/#model tags resurface every 30 days
//!
//! Storage: .constellation/review-pulse.json (never inside .md files)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Persisted review schedule data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReviewPulseData {
    #[serde(default)]
    pub last_reviewed: HashMap<String, String>,  // path → ISO date (YYYY-MM-DD)
    #[serde(default)]
    pub snoozed: HashMap<String, String>,        // path → ISO date (snooze until)
    #[serde(default)]
    pub intervals: HashMap<String, u32>,         // path → current interval in days
    #[serde(default)]
    pub dismissed: Vec<String>,                  // paths permanently dismissed
}

/// A note that's due for review.
#[derive(Debug, Clone, Serialize)]
pub struct DueNote {
    pub note_path: String,
    pub note_name: String,
    pub reason: String,        // "never_reviewed" | "interval_due" | "stale" | "checkpoint"
    pub days_overdue: i64,
    pub stratum: u8,
    pub last_reviewed: Option<String>,
}

/// Get all notes due for review in a library.
#[tauri::command]
pub fn get_due_notes(
    app: tauri::AppHandle,
    library_path: String,
) -> Result<Vec<DueNote>, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let cdir = crate::universe::active_constellation_dir(&app)?;
    let pulse = load_pulse_data(&cdir);
    let today = today_str();
    let today_days = date_to_days(&today);

    // Scan library for note metadata
    let tag_re = regex::Regex::new(r"(?:^|\s)#(assumption|model)\b").ok();
    let mut due: Vec<DueNote> = Vec::new();

    scan_due_recursive(
        Path::new(&library_path),
        &pulse,
        &today,
        today_days,
        &tag_re,
        &mut due,
    );

    // Sort: higher stratum first, then more overdue first
    due.sort_by(|a, b| {
        b.stratum.cmp(&a.stratum)
            .then(b.days_overdue.cmp(&a.days_overdue))
    });

    Ok(due)
}

/// Mark a note as reviewed. Doubles the review interval.
#[tauri::command]
pub fn mark_reviewed(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let mut pulse = load_pulse_data(&cdir);
    let today = today_str();

    pulse.last_reviewed.insert(note_path.clone(), today);

    // Double the interval (start at 1 day, max 30)
    let current = pulse.intervals.get(&note_path).copied().unwrap_or(1);
    let next = (current * 2).min(30);
    pulse.intervals.insert(note_path.clone(), next);

    // Remove from snoozed if present
    pulse.snoozed.remove(&note_path);

    save_pulse_data(&cdir, &pulse)
}

/// Snooze a note for N days.
#[tauri::command]
pub fn snooze_note(
    app: tauri::AppHandle,
    note_path: String,
    days: u32,
) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let mut pulse = load_pulse_data(&cdir);

    let snooze_until = add_days(&today_str(), days as i64);
    pulse.snoozed.insert(note_path, snooze_until);

    save_pulse_data(&cdir, &pulse)
}

/// Dismiss a note from the review queue permanently.
#[tauri::command]
pub fn dismiss_note(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let mut pulse = load_pulse_data(&cdir);

    if !pulse.dismissed.contains(&note_path) {
        pulse.dismissed.push(note_path);
    }

    save_pulse_data(&cdir, &pulse)
}

/// Record that a note was visited (called from frontend on tab open).
#[tauri::command]
pub fn record_note_visit(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let mut pulse = load_pulse_data(&cdir);

    // Only update if not already reviewed today
    let today = today_str();
    if pulse.last_reviewed.get(&note_path).map(|d| d.as_str()) != Some(today.as_str()) {
        pulse.last_reviewed.insert(note_path, today);
    }

    save_pulse_data(&cdir, &pulse)
}

// ─── Internal helpers ───

fn load_pulse_data(cdir: &Path) -> ReviewPulseData {
    let path = cdir.join("review-pulse.json");
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(pulse) = serde_json::from_str(&data) {
                return pulse;
            }
        }
    }
    ReviewPulseData::default()
}

fn save_pulse_data(cdir: &Path, pulse: &ReviewPulseData) -> Result<(), String> {
    let path = cdir.join("review-pulse.json");
    let json = serde_json::to_string_pretty(pulse).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Failed to write review-pulse.json: {}", e))
}

fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn date_to_days(date_str: &str) -> i64 {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        d.signed_duration_since(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()).num_days()
    } else {
        0
    }
}

fn add_days(date_str: &str, days: i64) -> String {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        (d + chrono::Duration::days(days)).format("%Y-%m-%d").to_string()
    } else {
        date_str.to_string()
    }
}

fn scan_due_recursive(
    dir: &Path,
    pulse: &ReviewPulseData,
    today: &str,
    today_days: i64,
    tag_re: &Option<regex::Regex>,
    due: &mut Vec<DueNote>,
) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with('.') { continue; }
        if path.is_dir() {
            scan_due_recursive(&path, pulse, today, today_days, tag_re, due);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let path_str = path.to_string_lossy().to_string();
            // MIG-008 Step 4: review-pulse "due notes" use the human title.
            // Helper reads the file ONLY for canonical-named notes (skipped
            // for human-named ones), so non-canonical libraries pay no
            // extra cost; canonical libraries pay one read per note.
            let note_name = crate::libraries::note_display_name(&path, None);

            // Skip dismissed notes
            if pulse.dismissed.contains(&path_str) { continue; }

            // Skip snoozed notes
            if let Some(snooze_until) = pulse.snoozed.get(&path_str) {
                if snooze_until.as_str() > today { continue; }
            }

            let last_reviewed = pulse.last_reviewed.get(&path_str).cloned();
            let interval = pulse.intervals.get(&path_str).copied().unwrap_or(1);

            // Compute stratum (simple: use word count as proxy, or 2 as default)
            let stratum = 2u8; // Default; real stratum comes from frontend merge

            // Mode 1: Spaced Resurfacing
            if let Some(ref lr) = last_reviewed {
                let lr_days = date_to_days(lr);
                let due_days = lr_days + interval as i64;
                if today_days >= due_days {
                    let overdue = today_days - due_days;
                    due.push(DueNote {
                        note_path: path_str.clone(),
                        note_name: note_name.clone(),
                        reason: "interval_due".to_string(),
                        days_overdue: overdue,
                        stratum,
                        last_reviewed: Some(lr.clone()),
                    });
                    continue; // Don't double-count
                }
            } else {
                // Never reviewed — check if note is older than 1 day
                if let Ok(meta) = fs::metadata(&path) {
                    if let Ok(modified) = meta.modified() {
                        let age_secs = std::time::SystemTime::now()
                            .duration_since(modified)
                            .unwrap_or_default()
                            .as_secs();
                        if age_secs > 86400 {
                            due.push(DueNote {
                                note_path: path_str.clone(),
                                note_name: note_name.clone(),
                                reason: "never_reviewed".to_string(),
                                days_overdue: (age_secs / 86400) as i64,
                                stratum,
                                last_reviewed: None,
                            });
                            continue;
                        }
                    }
                }
            }

            // Mode 3: Mental Model Checkpoints (#assumption, #model)
            if let Some(ref re) = tag_re {
                if let Ok(content) = fs::read_to_string(&path) {
                    if re.is_match(&content) {
                        // Check if 30+ days since last review
                        let needs_checkpoint = if let Some(ref lr) = last_reviewed {
                            today_days - date_to_days(lr) >= 30
                        } else {
                            true
                        };
                        if needs_checkpoint {
                            due.push(DueNote {
                                note_path: path_str.clone(),
                                note_name: note_name.clone(),
                                reason: "checkpoint".to_string(),
                                days_overdue: 0,
                                stratum,
                                last_reviewed: last_reviewed.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// MIG-083 §A — corrected, pure scheduling logic (no I/O; unit-tested).
// These are the CORRECTED behaviours (Boss "fix all quirks", 2026-06-22):
// the documented 1·3·7·14·30 ladder; the tags_json checkpoint definition;
// and the Mode-2 staleness trigger-type set. Consumed by §B (write-time
// maintenance) + §D (the read). The table is created in search.rs init_db.
// ════════════════════════════════════════════════════════════════════════

/// One row of the derived `review_schedule` table (Mode 1/3). The Mode-2
/// "stale" lens is computed by a separate read-time JOIN (§D), not stored here.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleRow {
    pub path: String,
    pub reason: String,        // "never_reviewed" | "interval_due" | "checkpoint"
    pub due_days: i64,         // due date as days-since-epoch (date_to_days)
    pub is_checkpoint: bool,
    pub last_reviewed: Option<String>,
    pub stratum: i64,          // real maturity stratum (sky_nodes.stratum), 0 if unknown
}

/// MIG-083 — Mode-2 staleness fires ONLY on these load-bearing OUT-link types
/// (Boss 2026-06-22). Plain `associative` is excluded (the anti-noise filter).
pub const STALENESS_TRIGGER_TYPES: [&str; 5] =
    ["supports", "contradicts", "derives-from", "part-of", "supersedes"];

/// Does a link type trigger Mode-2 staleness for its SOURCE note?
pub fn is_staleness_trigger_type(link_type: &str) -> bool {
    STALENESS_TRIGGER_TYPES.contains(&link_type)
}

/// The corrected interval ladder: 1 → 3 → 7 → 14 → 30 (cap 30). Returns the
/// next step strictly above `prev` (so a fresh note's first interval is 1).
pub fn next_interval(prev: u32) -> u32 {
    const LADDER: [u32; 5] = [1, 3, 7, 14, 30];
    for &step in LADDER.iter() {
        if step > prev {
            return step;
        }
    }
    30
}

/// A note is a Mental-Model Checkpoint iff its `tags_json` (frontmatter + inline
/// `#` tags, already built by `index_note`) contains `assumption` or `model`.
/// (Boss decision: `tags_json` is the canonical checkpoint definition — the
/// superset that catches Properties-tagged checkpoints the old `#`-regex missed.)
pub fn is_checkpoint(tags_json: &str) -> bool {
    serde_json::from_str::<Vec<String>>(tags_json)
        .map(|tags| {
            tags.iter().any(|t| {
                let l = t.to_lowercase();
                l == "assumption" || l == "model"
            })
        })
        .unwrap_or(false)
}

/// Compute the (reason, due_days) for a note's Mode-1/3 schedule row.
/// Precedence: a reviewed checkpoint follows the 30-day re-confrontation
/// cadence; a reviewed non-checkpoint follows the ladder; an unreviewed note is
/// `never_reviewed`, due one day after its anchor (created/modified day).
pub fn compute_schedule_row(
    last_reviewed_day: Option<i64>,
    interval: u32,
    is_checkpoint: bool,
    anchor_day: i64,
) -> (String, i64) {
    match last_reviewed_day {
        Some(lr) if is_checkpoint => ("checkpoint".to_string(), lr + 30),
        Some(lr) => ("interval_due".to_string(), lr + interval.max(1) as i64),
        None => ("never_reviewed".to_string(), anchor_day + 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_is_1_3_7_14_30_capped() {
        assert_eq!(next_interval(0), 1, "fresh → 1");
        assert_eq!(next_interval(1), 3);
        assert_eq!(next_interval(3), 7);
        assert_eq!(next_interval(7), 14);
        assert_eq!(next_interval(14), 30);
        assert_eq!(next_interval(30), 30, "cap at 30");
        assert_eq!(next_interval(99), 30, "anything ≥30 caps at 30");
    }

    #[test]
    fn checkpoint_from_tags_json_both_sources() {
        assert!(is_checkpoint(r#"["assumption","x"]"#), "inline/frontmatter assumption");
        assert!(is_checkpoint(r#"["Model"]"#), "case-insensitive");
        assert!(!is_checkpoint(r#"["modeling","assumptions"]"#), "no partial match");
        assert!(!is_checkpoint(r#"[]"#));
        assert!(!is_checkpoint("not json"), "malformed → false, never panics");
    }

    #[test]
    fn staleness_trigger_types_exclude_associative() {
        for t in ["supports", "contradicts", "derives-from", "part-of", "supersedes"] {
            assert!(is_staleness_trigger_type(t), "{t} should trigger");
        }
        assert!(!is_staleness_trigger_type("associative"), "associative must NOT trigger (anti-noise)");
        assert!(!is_staleness_trigger_type("exemplifies"));
        assert!(!is_staleness_trigger_type("causes"));
    }

    #[test]
    fn schedule_row_precedence() {
        // reviewed non-checkpoint → ladder
        assert_eq!(compute_schedule_row(Some(100), 7, false, 0), ("interval_due".into(), 107));
        // reviewed checkpoint → 30-day cadence, regardless of interval
        assert_eq!(compute_schedule_row(Some(100), 7, true, 0), ("checkpoint".into(), 130));
        // never reviewed → due one day after the anchor
        assert_eq!(compute_schedule_row(None, 0, false, 200), ("never_reviewed".into(), 201));
        // never-reviewed checkpoint surfaces as never_reviewed first (checkpoint cadence starts post-review)
        assert_eq!(compute_schedule_row(None, 0, true, 200), ("never_reviewed".into(), 201));
    }
}
