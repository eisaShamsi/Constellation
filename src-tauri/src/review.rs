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
