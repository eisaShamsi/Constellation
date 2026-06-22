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
use tauri::Manager; // for app.try_state in the action-writer row-sync (§B-2)

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

    pulse.last_reviewed.insert(note_path.clone(), today.clone());

    // MIG-083 — the documented 1·3·7·14·30 ladder (cap 30), not the old doubling.
    let current = pulse.intervals.get(&note_path).copied().unwrap_or(0);
    let next = next_interval(current);
    pulse.intervals.insert(note_path.clone(), next);

    // Remove from snoozed if present
    pulse.snoozed.remove(&note_path);

    save_pulse_data(&cdir, &pulse)?;
    // §B-2 — cache the action into the schedule row (no-op until §C stamps).
    sync_action_to_row(&app, |conn| review_row_mark(conn, &note_path, &today, next));
    Ok(())
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
    let until_day = date_to_days(&snooze_until);
    pulse.snoozed.insert(note_path.clone(), snooze_until);

    save_pulse_data(&cdir, &pulse)?;
    // §B-2 — push the schedule row's due day out so the read excludes it.
    sync_action_to_row(&app, |conn| review_row_snooze(conn, &note_path, until_day));
    Ok(())
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
        pulse.dismissed.push(note_path.clone());
    }

    save_pulse_data(&cdir, &pulse)?;
    // §B-2 — mark the schedule row dismissed (persists across re-index).
    sync_action_to_row(&app, |conn| review_row_dismiss(conn, &note_path));
    Ok(())
}

// MIG-083 — `record_note_visit` REMOVED (Boss decision 2026-06-22: opening a
// note does NOT count as a review; only the explicit "✓ Reviewed" action sets
// last_reviewed, so "I re-confronted this held position" stays meaningful). It
// was registered but never called from the frontend.

// ─── Internal helpers ───

pub(crate) fn load_pulse_data(cdir: &Path) -> ReviewPulseData {
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

#[cfg(test)]
fn day_to_date(days: i64) -> String {
    let base = chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    (base + chrono::Duration::days(days)).format("%Y-%m-%d").to_string()
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

// ── §B — the derived-table gate + write-time maintenance (DB side) ──────────

/// Is the `review_schedule` table built + authoritative? (schema_versions.review)
/// Until stamped (by the §C back-fill), the legacy FS scan is the source of truth
/// and every maintenance hook below is skipped — so §B is INERT until §C.
pub fn is_stamped(conn: &rusqlite::Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'review'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .map(|v| v >= 1)
    .unwrap_or(false)
}

/// Unix seconds → days-since-2020-01-01 (the epoch `date_to_days` uses), so a
/// note's `modified` timestamp and a `YYYY-MM-DD` review date are comparable.
pub fn secs_to_days(secs: i64) -> i64 {
    const UNIX_2020_01_01: i64 = 1_577_836_800;
    (secs - UNIX_2020_01_01).div_euclid(86_400)
}

/// MIG-083 §D — the Mode-2 content-change signal. A stable FNV-1a 64-bit hash of
/// a note's body, hex-encoded. `index_note` stores this in `note_meta.content_hash`
/// and bumps `content_changed_at` ONLY when the hash differs from the stored one —
/// so a real body edit fires staleness for dependents, but a touch / sync / cid_cn /
/// frontmatter-only save (body unchanged) does NOT.
///
/// FNV-1a is chosen deliberately over `std`'s `DefaultHasher`: the hash is PERSISTED
/// to disk and compared across app restarts and Rust toolchain upgrades. The std
/// hasher's algorithm is explicitly "not specified … should not be relied upon over
/// releases", which would silently false-fire every dependent on the first save after
/// a toolchain bump. FNV-1a is a fixed, specified algorithm — same bytes, same hash,
/// forever.
pub fn content_hash(body: &str) -> String {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET_BASIS;
    for &b in body.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    format!("{:016x}", h)
}

/// Write-time maintenance of ONE note's Mode-1/3 schedule row, from data already
/// in hand at `index_note` (zero extra `.md` reads). Preserves the action-owned
/// fields (`last_reviewed`, `interval`) and a `dismissed` state across re-index;
/// recomputes the content-derived fields (`is_checkpoint`, `reason`, `due_days`,
/// `stratum`). Caller gates on [`is_stamped`].
pub fn upsert_schedule_row(
    conn: &rusqlite::Connection,
    path: &str,
    tags_json: &str,
    modified_secs: i64,
    stratum: i64,
) -> Result<(), String> {
    let existing: Option<(Option<String>, i64, String)> = conn
        .query_row(
            "SELECT last_reviewed, interval, reason FROM review_schedule WHERE path = ?1",
            rusqlite::params![path],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok();

    // A dismissed note stays dismissed across re-index (else it'd resurface on
    // the next save). Leave the row untouched.
    if let Some((_, _, ref reason)) = existing {
        if reason == "dismissed" {
            return Ok(());
        }
    }

    let (last_reviewed, interval): (Option<String>, u32) = existing
        .map(|(lr, iv, _)| (lr, iv.max(0) as u32))
        .unwrap_or((None, 0));
    let is_cp = is_checkpoint(tags_json);
    let anchor_day = secs_to_days(modified_secs);
    let lr_day = last_reviewed.as_ref().map(|d| date_to_days(d));
    let (reason, due_days) = compute_schedule_row(lr_day, interval, is_cp, anchor_day);

    conn.execute(
        "INSERT INTO review_schedule (path, reason, due_days, is_checkpoint, last_reviewed, stratum, interval)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(path) DO UPDATE SET
           reason        = excluded.reason,
           due_days      = excluded.due_days,
           is_checkpoint = excluded.is_checkpoint,
           stratum       = excluded.stratum",
        rusqlite::params![path, reason, due_days, is_cp as i64, last_reviewed, stratum, interval as i64],
    )
    .map_err(|e| format!("review_schedule upsert {}: {}", path, e))?;
    Ok(())
}

/// Drop a note's schedule row (on note deletion). Caller gates on [`is_stamped`].
pub fn delete_schedule_row(conn: &rusqlite::Connection, path: &str) -> Result<(), String> {
    conn.execute("DELETE FROM review_schedule WHERE path = ?1", rusqlite::params![path])
        .map_err(|e| format!("review_schedule delete {}: {}", path, e))?;
    Ok(())
}

// ── §B-2 — action-writer row-sync ───────────────────────────────────────────
// The row CACHES last_reviewed/interval; `upsert_schedule_row` reads them from
// the ROW (not review-pulse.json) to stay off the per-save hot path. So an
// explicit ✓/snooze/dismiss must write the row directly. No-op until `review` is
// stamped (the row doesn't exist before the §C back-fill anyway).

/// Run `f` against the search DB iff it's ready and `review` is stamped.
fn sync_action_to_row(
    app: &tauri::AppHandle,
    f: impl FnOnce(&rusqlite::Connection) -> Result<(), String>,
) {
    if let Some(state) = app.try_state::<crate::search::SearchState>() {
        if let Ok(db) = state.db.lock() {
            if let Some(conn) = db.as_ref() {
                if is_stamped(conn) {
                    let _ = f(conn);
                }
            }
        }
    }
}

/// ✓ Reviewed: cache the new last_reviewed + interval and recompute reason/due
/// (is_checkpoint is read from the row — it's content-derived, set by index_note).
fn review_row_mark(
    conn: &rusqlite::Connection,
    path: &str,
    last_reviewed: &str,
    interval: u32,
) -> Result<(), String> {
    let is_cp = conn
        .query_row(
            "SELECT is_checkpoint FROM review_schedule WHERE path = ?1",
            rusqlite::params![path],
            |r| r.get::<_, i64>(0),
        )
        .map(|v| v != 0)
        .unwrap_or(false);
    let (reason, due_days) =
        compute_schedule_row(Some(date_to_days(last_reviewed)), interval, is_cp, 0);
    conn.execute(
        "INSERT INTO review_schedule (path, reason, due_days, is_checkpoint, last_reviewed, stratum, interval)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)
         ON CONFLICT(path) DO UPDATE SET
           reason=excluded.reason, due_days=excluded.due_days,
           last_reviewed=excluded.last_reviewed, interval=excluded.interval",
        rusqlite::params![path, reason, due_days, is_cp as i64, last_reviewed, interval as i64],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Snooze: push the due day forward so the read (`due_days <= today`) excludes it.
fn review_row_snooze(conn: &rusqlite::Connection, path: &str, until_day: i64) -> Result<(), String> {
    conn.execute(
        "UPDATE review_schedule SET due_days = ?1 WHERE path = ?2",
        rusqlite::params![until_day, path],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Dismiss: mark the row dismissed (persists across re-index); insert if absent.
fn review_row_dismiss(conn: &rusqlite::Connection, path: &str) -> Result<(), String> {
    let changed = conn
        .execute(
            "UPDATE review_schedule SET reason = 'dismissed' WHERE path = ?1",
            rusqlite::params![path],
        )
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        conn.execute(
            "INSERT OR IGNORE INTO review_schedule (path, reason, due_days) VALUES (?1, 'dismissed', 0)",
            rusqlite::params![path],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// §C — populate ONE note's schedule row from the `review-pulse.json` action
/// state (the back-fill's per-note step; idempotent `INSERT OR REPLACE`). Unlike
/// the write-time upsert, this SETS `last_reviewed`/`interval` from the JSON
/// source of truth. `today` is passed in (not read) so it's deterministic to test.
pub fn backfill_schedule_row(
    conn: &rusqlite::Connection,
    path: &str,
    tags_json: &str,
    modified_secs: i64,
    stratum: i64,
    last_reviewed: Option<&str>,
    interval: u32,
    snoozed_until: Option<&str>,
    dismissed: bool,
    today: &str,
) -> Result<(), String> {
    let is_cp = is_checkpoint(tags_json);
    let (reason, due_days) = if dismissed {
        ("dismissed".to_string(), 0)
    } else {
        let lr_day = last_reviewed.map(date_to_days);
        let (r, mut d) = compute_schedule_row(lr_day, interval, is_cp, secs_to_days(modified_secs));
        // A still-active snooze pushes the due day to the snooze date (keep reason).
        if let Some(su) = snoozed_until {
            if su > today {
                d = date_to_days(su);
            }
        }
        (r, d)
    };
    conn.execute(
        "INSERT OR REPLACE INTO review_schedule (path, reason, due_days, is_checkpoint, last_reviewed, stratum, interval)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![path, reason, due_days, is_cp as i64, last_reviewed, stratum, interval as i64],
    )
    .map_err(|e| format!("review_schedule backfill {}: {}", path, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sched_db() -> rusqlite::Connection {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);
             CREATE TABLE review_schedule (path TEXT PRIMARY KEY, reason TEXT NOT NULL, due_days INTEGER NOT NULL,
               is_checkpoint INTEGER NOT NULL DEFAULT 0, last_reviewed TEXT, stratum INTEGER NOT NULL DEFAULT 0,
               interval INTEGER NOT NULL DEFAULT 0);",
        ).unwrap();
        c
    }
    fn row(c: &rusqlite::Connection, path: &str) -> (String, i64, i64) {
        c.query_row("SELECT reason, due_days, is_checkpoint FROM review_schedule WHERE path=?1",
            rusqlite::params![path], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap()
    }

    #[test]
    fn is_stamped_gate() {
        let c = sched_db();
        assert!(!is_stamped(&c), "unstamped by default");
        c.execute("INSERT INTO schema_versions (module, version) VALUES ('review', 1)", []).unwrap();
        assert!(is_stamped(&c));
    }

    #[test]
    fn upsert_new_note_is_never_reviewed() {
        let c = sched_db();
        let modified = 1_577_836_800 + 100 * 86_400; // day 100
        upsert_schedule_row(&c, "/n.md", "[]", modified, 3).unwrap();
        let (reason, due, is_cp) = row(&c, "/n.md");
        assert_eq!(reason, "never_reviewed");
        assert_eq!(due, 101, "anchor day 100 + 1");
        assert_eq!(is_cp, 0);
        assert_eq!(c.query_row("SELECT stratum FROM review_schedule WHERE path='/n.md'", [], |r| r.get::<_,i64>(0)).unwrap(), 3);
    }

    #[test]
    fn upsert_preserves_review_state_and_recomputes_checkpoint() {
        let c = sched_db();
        // a previously-reviewed row (last_reviewed day 200, interval 7)
        c.execute("INSERT INTO review_schedule (path, reason, due_days, is_checkpoint, last_reviewed, stratum, interval)
                   VALUES ('/n.md','interval_due',207,0,?1,2,7)",
            rusqlite::params![day_to_date(200)]).unwrap();
        // re-index after tagging it #assumption → becomes a checkpoint; last_reviewed/interval preserved
        upsert_schedule_row(&c, "/n.md", r#"["assumption"]"#, 0, 5).unwrap();
        let (reason, due, is_cp) = row(&c, "/n.md");
        assert_eq!(reason, "checkpoint");
        assert_eq!(is_cp, 1);
        assert_eq!(due, 230, "last_reviewed 200 + 30-day checkpoint cadence");
        let (lr, iv): (String, i64) = c.query_row("SELECT last_reviewed, interval FROM review_schedule WHERE path='/n.md'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(date_to_days(&lr), 200, "last_reviewed preserved");
        assert_eq!(iv, 7, "interval preserved");
    }

    #[test]
    fn upsert_leaves_dismissed_alone() {
        let c = sched_db();
        c.execute("INSERT INTO review_schedule (path, reason, due_days) VALUES ('/n.md','dismissed',0)", []).unwrap();
        upsert_schedule_row(&c, "/n.md", r#"["assumption"]"#, 0, 9).unwrap();
        let (reason, _, _) = row(&c, "/n.md");
        assert_eq!(reason, "dismissed", "a dismissed note is not resurrected by re-index");
    }

    #[test]
    fn delete_drops_the_row() {
        let c = sched_db();
        upsert_schedule_row(&c, "/n.md", "[]", 1_577_836_800, 0).unwrap();
        delete_schedule_row(&c, "/n.md").unwrap();
        assert_eq!(c.query_row("SELECT COUNT(*) FROM review_schedule", [], |r| r.get::<_,i64>(0)).unwrap(), 0);
    }

    #[test]
    fn row_mark_uses_ladder_and_checkpoint() {
        let c = sched_db();
        c.execute("INSERT INTO review_schedule (path, reason, due_days, is_checkpoint) VALUES ('/n.md','never_reviewed',5,0)", []).unwrap();
        review_row_mark(&c, "/n.md", &day_to_date(100), 1).unwrap(); // first ✓ → interval 1
        let (reason, due, _) = row(&c, "/n.md");
        assert_eq!(reason, "interval_due");
        assert_eq!(due, 101);
        // a checkpoint row marks on the 30-day cadence regardless of interval
        c.execute("INSERT INTO review_schedule (path, reason, due_days, is_checkpoint) VALUES ('/c.md','checkpoint',0,1)", []).unwrap();
        review_row_mark(&c, "/c.md", &day_to_date(200), 7).unwrap();
        assert_eq!(row(&c, "/c.md"), ("checkpoint".to_string(), 230, 1));
    }

    #[test]
    fn row_snooze_and_dismiss() {
        let c = sched_db();
        c.execute("INSERT INTO review_schedule (path, reason, due_days) VALUES ('/n.md','interval_due',100)", []).unwrap();
        review_row_snooze(&c, "/n.md", 150).unwrap();
        assert_eq!(c.query_row("SELECT due_days FROM review_schedule WHERE path='/n.md'", [], |r| r.get::<_,i64>(0)).unwrap(), 150);
        // dismiss existing + absent
        review_row_dismiss(&c, "/n.md").unwrap();
        assert_eq!(row(&c, "/n.md").0, "dismissed");
        review_row_dismiss(&c, "/absent.md").unwrap();
        assert_eq!(row(&c, "/absent.md").0, "dismissed", "dismiss persists even with no prior row");
    }

    #[test]
    fn backfill_sets_state_snooze_dismiss() {
        let c = sched_db();
        let today = "2026-06-22";
        let lr = day_to_date(100);
        backfill_schedule_row(&c, "/r.md", "[]", 0, 2, Some(&lr), 7, None, false, today).unwrap();
        assert_eq!(row(&c, "/r.md"), ("interval_due".to_string(), 107, 0));
        backfill_schedule_row(&c, "/d.md", "[]", 0, 0, None, 0, None, true, today).unwrap();
        assert_eq!(row(&c, "/d.md").0, "dismissed");
        // active snooze → due pushed to the snooze day, reason kept
        backfill_schedule_row(&c, "/s.md", "[]", 0, 0, Some(&lr), 3, Some("2099-01-01"), false, today).unwrap();
        assert_eq!(c.query_row("SELECT due_days FROM review_schedule WHERE path='/s.md'", [], |r| r.get::<_,i64>(0)).unwrap(), date_to_days("2099-01-01"));
    }

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

    #[test]
    fn content_hash_is_stable_and_distinguishes_real_changes() {
        // Empty body → the FNV-1a offset basis (the one vector we can assert by
        // construction; guards against an accidental algorithm change).
        assert_eq!(content_hash(""), "cbf29ce484222325");
        // Deterministic: same bytes, same hash (across calls → across restarts).
        assert_eq!(content_hash("The horse pulls the carriage."), content_hash("The horse pulls the carriage."));
        // A one-character body edit flips the hash → content_changed_at WILL bump.
        assert_ne!(content_hash("conviction"), content_hash("convictions"));
        // Whitespace IS content (a real edit) — but an identical re-save is a no-op.
        assert_ne!(content_hash("a b"), content_hash("a  b"));
        // 16 hex chars always (fixed-width, so a TEXT-column compare is exact).
        assert_eq!(content_hash("anything at all").len(), 16);
    }
}
