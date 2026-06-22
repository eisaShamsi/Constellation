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
    // MIG-083 §D — Mode-2 staleness "why" (None for Mode-1/3 rows). The §F two-lens
    // reviewer renders "stale because {type} {name} changed on {date}" from these.
    pub stale_trigger_name: Option<String>, // the changed OUT-dependency's display name
    pub stale_trigger_type: Option<String>, // the load-bearing link type that carries it
    pub stale_changed_on: Option<String>,   // YYYY-MM-DD the dependency's content changed
}

/// Get all notes due for review in a library.
#[tauri::command]
pub fn get_due_notes(
    app: tauri::AppHandle,
    library_path: String,
    stale_grace_days: Option<i64>,
) -> Result<Vec<DueNote>, String> {
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    let today = today_str();
    let today_days = date_to_days(&today);
    let grace = stale_grace_days.unwrap_or(1); // default: strict next-day (Mode-2)

    // MIG-083 — Rule-8 read. Once the §C back-fill has built + stamped the write-time
    // `review_schedule` table, read it (an indexed SELECT ∪ the Mode-2 staleness JOIN)
    // — ZERO filesystem access, <100 ms on 7,600 notes.
    if let Some(state) = app.try_state::<crate::search::SearchState>() {
        if let Ok(guard) = state.db.lock() {
            if let Some(conn) = guard.as_ref() {
                if is_stamped(conn) {
                    return query_due_notes_indexed(conn, &library_path, &today, today_days, grace);
                }
            }
        }
    }

    // Unstamped — the write-time schedule isn't built yet (first boot of a MIG-083
    // build, before the post-paint back-fill stamps; or a never-built table). The
    // legacy full-FS-walk `scan_due_recursive` was REMOVED in §E once the indexed swap
    // was Boss-validated (it was the Rule-8 violation this migration existed to kill).
    // Kick the back-fill (idempotent — no-op if already running/stamped) and return an
    // empty list; the panel shows "All caught up" for the few seconds until it stamps,
    // then every read is the cheap indexed path.
    crate::review_backfill::maybe_schedule(app.clone());
    Ok(Vec::new())
}

/// MIG-083 §D — the Rule-8 indexed read. Builds the due list from the write-time
/// `review_schedule` table (Mode 1/3) UNION the Mode-2 staleness JOIN, with **zero
/// filesystem access** (no `read_dir` / `metadata` / `read_to_string` / regex over
/// `.md`). Caller holds the DB lock and has verified [`is_stamped`].
///
/// `library_path` scopes to the same subtree the legacy scan walked. To avoid
/// sibling-library bleed-through ("/U/Lib" must NOT match "/U/Lib2"; review finding
/// D), the prefix is **separator-terminated** so the match lands on a path boundary.
/// An EMPTY `library_path` means "whole universe" (the rehearsal harness) → no scope
/// filter. `substr` is char-indexed in SQLite (correct for the multibyte Arabic root).
/// The two lenses are kept distinct (Boss: "two separate lenses, never merged into
/// one score") — a note can appear once per lens, each carrying its own `reason`.
pub(crate) fn query_due_notes_indexed(
    conn: &rusqlite::Connection,
    library_path: &str,
    today: &str,
    today_days: i64,
    stale_grace_days: i64,
) -> Result<Vec<DueNote>, String> {
    // Staleness grace period (Boss-configurable, minimum 1 day): a dependency must
    // have changed at least `grace` days AFTER the note's last review to flag it.
    // grace=1 == the strict next-day-onward default.
    let grace = stale_grace_days.max(1);
    let mut due: Vec<DueNote> = Vec::new();
    // Library scoping: a note is in-scope iff its path begins with library_path AND
    // the next char is a path separator — so "/U/Lib" matches "/U/Lib/x.md" but NOT
    // the sibling "/U/Lib2/y.md" (review finding D). Matches EITHER '/' or '\' (=char(92))
    // so it is correct whether note_meta stores POSIX or Windows separators — appending
    // one fixed separator would zero out the queue if the stored form differed. An empty
    // library_path means "whole universe" (the rehearsal harness) → match all. A trailing
    // separator on the input is trimmed so the boundary char lands on the real separator.
    let library_path = library_path.trim_end_matches(['/', '\\']);

    // ── Lens 1: Mode 1/3 — time-based resurfacing + checkpoints (indexed on due_days). ──
    {
        let mut stmt = conn
            .prepare(
                // INNER JOIN: a row with no backing note_meta (an orphan — e.g. left by
                // some non-delete path) must NEVER surface as a phantom queue entry
                // pointing at a dead path (re-verify finding). No note_meta → not a note.
                "SELECT rs.path, nm.name, rs.reason, rs.due_days, rs.stratum, rs.last_reviewed
                 FROM review_schedule rs
                 JOIN note_meta nm ON nm.path = rs.path
                 WHERE rs.due_days <= ?1
                   AND rs.reason != 'dismissed'
                   AND (rs.snoozed_until IS NULL OR rs.snoozed_until <= ?3)
                   AND (?2 = '' OR (substr(rs.path, 1, length(?2)) = ?2
                        AND substr(rs.path, length(?2) + 1, 1) IN ('/', char(92))))",
            )
            .map_err(|e| format!("due lens-1 prepare: {}", e))?;
        let rows = stmt
            .query_map(rusqlite::params![today_days, library_path, today], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|e| format!("due lens-1 query: {}", e))?;
        for row in rows.flatten() {
            let (path, name, reason, due_days, stratum, last_reviewed) = row;
            due.push(DueNote {
                note_path: path,
                note_name: name,
                reason,
                days_overdue: today_days - due_days,
                stratum: stratum.clamp(0, 255) as u8,
                last_reviewed,
                stale_trigger_name: None,
                stale_trigger_type: None,
                stale_changed_on: None,
            });
        }
    }

    // ── Lens 2: Mode 2 — staleness. A note is stale when a load-bearing OUT-dependency
    // (supports/contradicts/derives-from/part-of/supersedes; NOT associative) had its
    // CONTENT actually change (hash-confirmed — `content_changed_at IS NOT NULL`; we
    // do NOT fall back to file mtime, so a sync/touch/cid_cn/frontmatter save never
    // false-fires — review finding A) on a later LOCAL calendar day than this note's
    // last explicit review (`local_day` vs the local `last_reviewed` — finding F).
    // Resolution: note_links.target_cid_cn → note_meta.cid_cn (both UNIQUE-indexed —
    // the reliable join key; target_path is unset for freshly-indexed links). 1-hop;
    // self-links excluded (finding I). One row per stale note, citing its most
    // consequential changed dependency (highest weight, then most-recent change, then
    // jl.id for a stable tie-break — finding G). ──
    //
    // Structured as two steps (NOT one big JOIN) for a guaranteed query plan: the
    // single-JOIN form let SQLite drive from `note_links.status='active'` — scanning
    // ALL ~234k active links on a large universe (~200 ms) — because `last_reviewed`
    // is unindexed-looking to the planner on a freshly-built table. Instead: (1) fetch
    // the tiny reviewed set (the partial index idx_review_last_reviewed makes this
    // O(reviewed), not O(corpus)); (2) probe each note's out-links with a prepared
    // statement reused per note — every call rides idx_link_source. The day comparison
    // is done in Rust (`local_day`) so impl + the rehearsal reference share ONE
    // arithmetic (no SQLite-`/` vs `div_euclid` divergence — finding H).
    {
        let reviewed: Vec<(String, String, i64, String)> = {
            let mut stmt = conn
                .prepare(
                    // NOTE (Boss 2026-06-22): snooze does NOT suppress the Stale lens —
                    // the two lenses stay fully separate. Snooze hides a note from
                    // time-based "Due for Review" (Lens-1) only; staleness is a distinct
                    // signal (a dependency changed) and still surfaces while snoozed.
                    "SELECT rs.path, nm.name, rs.stratum, rs.last_reviewed
                     FROM review_schedule rs
                     JOIN note_meta nm ON nm.path = rs.path
                     WHERE rs.last_reviewed IS NOT NULL
                       AND rs.reason != 'dismissed'
                       AND (?1 = '' OR (substr(rs.path, 1, length(?1)) = ?1
                            AND substr(rs.path, length(?1) + 1, 1) IN ('/', char(92))))",
                )
                .map_err(|e| format!("due lens-2 reviewed prepare: {}", e))?;
            let rows = stmt
                .query_map(rusqlite::params![library_path], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, String>(3)?))
                })
                .map_err(|e| format!("due lens-2 reviewed query: {}", e))?;
            rows.flatten().collect()
        };

        let types_in = STALENESS_TRIGGER_TYPES
            .iter()
            .map(|t| format!("'{}'", t))
            .collect::<Vec<_>>()
            .join(",");
        // Per reviewed note: its load-bearing out-links whose dependency has a
        // hash-confirmed content change, most-consequential first. The day filter +
        // "first that beats last_reviewed" pick happen in Rust (local_day).
        let probe_sql = format!(
            "SELECT jl.link_type, COALESCE(dep.name, jl.target_name), dep.content_changed_at
             FROM note_links jl
             JOIN note_meta dep ON dep.cid_cn = jl.target_cid_cn
             WHERE jl.source_path = ?1
               AND jl.status = 'active'
               AND jl.link_type IN ({types})
               AND jl.target_cid_cn IS NOT NULL AND jl.target_cid_cn != ''
               AND dep.content_changed_at IS NOT NULL
               AND dep.path != ?1
             ORDER BY jl.weight DESC, dep.content_changed_at DESC, jl.id DESC",
            types = types_in,
        );
        let mut probe = conn.prepare(&probe_sql).map_err(|e| format!("due lens-2 probe prepare: {}", e))?;
        for (path, name, stratum, last_reviewed) in reviewed {
            // A malformed last_reviewed is SKIPPED (never bucketed to day 0).
            let lr_day = match parse_day(&last_reviewed) {
                Some(d) => d,
                None => continue,
            };
            let mut rows = probe
                .query_map(rusqlite::params![path], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
                })
                .map_err(|e| format!("due lens-2 probe query: {}", e))?;
            // Rows arrive most-consequential first; take the first whose dependency
            // changed on a later LOCAL day than the review (strict > → same-day safe).
            while let Some(Ok((link_type, dep_name, cca))) = rows.next() {
                let dep_day = local_day(cca);
                if dep_day - lr_day >= grace {
                    due.push(DueNote {
                        note_path: path.clone(),
                        note_name: name.clone(),
                        reason: "stale".to_string(),
                        days_overdue: (today_days - dep_day).max(0),
                        stratum: stratum.clamp(0, 255) as u8,
                        last_reviewed: Some(last_reviewed.clone()),
                        stale_trigger_name: Some(dep_name),
                        stale_trigger_type: Some(link_type),
                        stale_changed_on: Some(day_to_date(dep_day)),
                    });
                    break;
                }
            }
        }
    }

    // Sort: higher stratum first, then more overdue first (matches the legacy scan).
    due.sort_by(|a, b| b.stratum.cmp(&a.stratum).then(b.days_overdue.cmp(&a.days_overdue)));
    Ok(due)
}

/// MIG-083 §D — a note's Review-Pulse status (the §F note-context Review tab reads
/// this, O(1)). `reason`/`due_days` are None when the note has no schedule row yet
/// (unstamped, or not-yet-indexed) — the tab renders a clean "not scheduled" state.
#[derive(Debug, Clone, Serialize)]
pub struct NoteReviewStatus {
    pub reason: Option<String>,        // never_reviewed | interval_due | checkpoint | dismissed
    pub due_days: Option<i64>,         // due date as days-since-2020 (None if no row)
    pub last_reviewed: Option<String>, // ISO date of the last explicit ✓, or None
    pub never_reviewed: bool,          // true iff no explicit review has happened
    pub is_checkpoint: bool,           // a #assumption/#model mental-model checkpoint
}

/// MIG-083 §D — O(1) PK lookup of one note's review status. Read-only single-row
/// metadata fetch keyed by an already-open note's path (no fs access, no library
/// validation needed — the frontend only asks for notes it already opened).
#[tauri::command]
pub fn get_note_review_status(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<NoteReviewStatus, String> {
    if let Some(state) = app.try_state::<crate::search::SearchState>() {
        if let Ok(guard) = state.db.lock() {
            if let Some(conn) = guard.as_ref() {
                let row: Option<(String, i64, Option<String>, i64)> = conn
                    .query_row(
                        "SELECT reason, due_days, last_reviewed, is_checkpoint FROM review_schedule WHERE path = ?1",
                        rusqlite::params![note_path],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                    )
                    .ok();
                if let Some((reason, due_days, last_reviewed, is_cp)) = row {
                    return Ok(NoteReviewStatus {
                        never_reviewed: last_reviewed.is_none(),
                        reason: Some(reason),
                        due_days: Some(due_days),
                        last_reviewed,
                        is_checkpoint: is_cp != 0,
                    });
                }
            }
        }
    }
    // No row (unstamped, or the note isn't scheduled): a clean "never reviewed" status.
    Ok(NoteReviewStatus {
        reason: None,
        due_days: None,
        last_reviewed: None,
        never_reviewed: true,
        is_checkpoint: false,
    })
}

/// Mark a note as reviewed. Advances to the next interval on the 1·3·7·14·30 ladder.
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
    pulse.snoozed.insert(note_path.clone(), snooze_until.clone());

    save_pulse_data(&cdir, &pulse)?;
    // §B-2 — push the schedule row's due day out (Lens-1) + record snoozed_until
    // (Lens-2) so the read excludes it from BOTH lenses.
    sync_action_to_row(&app, |conn| review_row_snooze(conn, &note_path, &snooze_until));
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

pub(crate) fn date_to_days(date_str: &str) -> i64 {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        d.signed_duration_since(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()).num_days()
    } else {
        0
    }
}

/// Strict variant of [`date_to_days`]: `None` when the string isn't a valid
/// `YYYY-MM-DD`. Mode-2 uses this so a malformed `last_reviewed` is SKIPPED rather
/// than silently bucketed to day 0 (2020-01-01), which would make almost every
/// dependency look "changed after review" → spurious staleness (review finding E).
pub(crate) fn parse_day(date_str: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .ok()
        .map(|d| d.signed_duration_since(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()).num_days())
}

/// Unix seconds → the **local** calendar day (days-since-2020), via the OS timezone.
/// Mode-2 compares a dependency's change day against `last_reviewed` — which is
/// written in LOCAL time (`today_str` uses `chrono::Local`). A file mtime is an
/// absolute (UTC) instant, so bucketing it by UTC day skews ±1 against the local
/// review date near midnight in non-UTC zones (review finding F). Converting the
/// mtime to the local day makes both sides share one frame. Falls back to the UTC
/// day only if the timestamp is out of range.
pub(crate) fn local_day(secs: i64) -> i64 {
    use chrono::TimeZone;
    match chrono::Local.timestamp_opt(secs, 0).single() {
        Some(dt) => dt
            .date_naive()
            .signed_duration_since(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())
            .num_days(),
        None => secs_to_days(secs),
    }
}

/// Days-since-2020-01-01 → `YYYY-MM-DD` (the inverse of `date_to_days`). Used to
/// render `stale_changed_on` for the Mode-2 lens.
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
    let existing: Option<(Option<String>, i64, String, Option<String>)> = conn
        .query_row(
            "SELECT last_reviewed, interval, reason, snoozed_until FROM review_schedule WHERE path = ?1",
            rusqlite::params![path],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .ok();

    // A dismissed note stays dismissed across re-index (else it'd resurface on
    // the next save). Leave the row untouched.
    if let Some((_, _, ref reason, _)) = existing {
        if reason == "dismissed" {
            return Ok(());
        }
    }

    let (last_reviewed, interval, snoozed_until): (Option<String>, u32, Option<String>) = existing
        .map(|(lr, iv, _, su)| (lr, iv.max(0) as u32, su))
        .unwrap_or((None, 0, None));
    let is_cp = is_checkpoint(tags_json);
    let anchor_day = secs_to_days(modified_secs);
    let lr_day = last_reviewed.as_ref().map(|d| date_to_days(d));
    let (reason, mut due_days) = compute_schedule_row(lr_day, interval, is_cp, anchor_day);

    // Preserve an ACTIVE snooze across re-index (review finding E #7): without this,
    // a save (or a rename-cascade re-index) recomputes due_days and silently drops
    // the snooze. Keep due_days at the snooze day so Lens-1 stays hidden; the
    // snoozed_until column itself is preserved by NOT touching it in DO UPDATE.
    let today = today_str();
    if let Some(ref su) = snoozed_until {
        if su.as_str() > today.as_str() {
            due_days = date_to_days(su);
        }
    }

    conn.execute(
        "INSERT INTO review_schedule (path, reason, due_days, is_checkpoint, last_reviewed, stratum, interval, snoozed_until)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(path) DO UPDATE SET
           reason        = excluded.reason,
           due_days      = excluded.due_days,
           is_checkpoint = excluded.is_checkpoint,
           stratum       = excluded.stratum",
        rusqlite::params![path, reason, due_days, is_cp as i64, last_reviewed, stratum, interval as i64, snoozed_until],
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
    // ✓ Reviewed clears any active snooze (reviewing IS engaging with the note).
    conn.execute(
        "INSERT INTO review_schedule (path, reason, due_days, is_checkpoint, last_reviewed, stratum, interval, snoozed_until)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, NULL)
         ON CONFLICT(path) DO UPDATE SET
           reason=excluded.reason, due_days=excluded.due_days,
           last_reviewed=excluded.last_reviewed, interval=excluded.interval, snoozed_until=NULL",
        rusqlite::params![path, reason, due_days, is_cp as i64, last_reviewed, interval as i64],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Snooze: push the due day forward (Lens-1 excludes via `due_days <= today`) AND
/// record `snoozed_until` so the Mode-2 Stale lens also hides it — snooze = "not
/// now" across BOTH lenses (review findings C #3/#10).
fn review_row_snooze(conn: &rusqlite::Connection, path: &str, snooze_until: &str) -> Result<(), String> {
    let until_day = date_to_days(snooze_until);
    conn.execute(
        "UPDATE review_schedule SET due_days = ?1, snoozed_until = ?2 WHERE path = ?3",
        rusqlite::params![until_day, snooze_until, path],
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
/// Pure core of the back-fill: a note's `(reason, due_days)` given its action state
/// (`review-pulse.json`) + content. Shared by [`backfill_schedule_row`] (the write
/// path) and the §D rehearsal reference (the read-side recompute) so the two can
/// never drift — same spec, one implementation.
pub fn schedule_for(
    is_checkpoint: bool,
    modified_secs: i64,
    last_reviewed: Option<&str>,
    interval: u32,
    snoozed_until: Option<&str>,
    dismissed: bool,
    today: &str,
) -> (String, i64) {
    if dismissed {
        return ("dismissed".to_string(), 0);
    }
    let lr_day = last_reviewed.map(date_to_days);
    let (r, mut d) = compute_schedule_row(lr_day, interval, is_checkpoint, secs_to_days(modified_secs));
    // A still-active snooze pushes the due day to the snooze date (keep reason).
    if let Some(su) = snoozed_until {
        if su > today {
            d = date_to_days(su);
        }
    }
    (r, d)
}

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
    let (reason, due_days) =
        schedule_for(is_cp, modified_secs, last_reviewed, interval, snoozed_until, dismissed, today);
    conn.execute(
        "INSERT OR REPLACE INTO review_schedule (path, reason, due_days, is_checkpoint, last_reviewed, stratum, interval, snoozed_until)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![path, reason, due_days, is_cp as i64, last_reviewed, stratum, interval as i64, snoozed_until],
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
               interval INTEGER NOT NULL DEFAULT 0, snoozed_until TEXT);",
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
        let su = day_to_date(150);
        review_row_snooze(&c, "/n.md", &su).unwrap();
        let (dd, snz): (i64, Option<String>) = c.query_row("SELECT due_days, snoozed_until FROM review_schedule WHERE path='/n.md'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(dd, 150, "due pushed to the snooze day (Lens-1 hides it)");
        assert_eq!(snz.as_deref(), Some(su.as_str()), "snoozed_until recorded (Lens-2 hides it)");
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

    /// Minimal slice of the real schema needed by `query_due_notes_indexed`.
    fn read_db() -> rusqlite::Connection {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT, cid_cn TEXT, modified INTEGER, content_changed_at INTEGER);
             CREATE TABLE note_links (id INTEGER PRIMARY KEY AUTOINCREMENT, source_path TEXT, target_name TEXT, target_cid_cn TEXT, link_type TEXT, status TEXT DEFAULT 'active', weight REAL DEFAULT 1.0);
             CREATE TABLE review_schedule (path TEXT PRIMARY KEY, reason TEXT NOT NULL, due_days INTEGER NOT NULL,
               is_checkpoint INTEGER NOT NULL DEFAULT 0, last_reviewed TEXT, stratum INTEGER NOT NULL DEFAULT 0, interval INTEGER NOT NULL DEFAULT 0, snoozed_until TEXT);",
        ).unwrap();
        c
    }
    /// Unix seconds at UTC-midnight of a YYYY-MM-DD date (matches strftime('%s', d)).
    fn secs(date: &str) -> i64 { date_to_days(date) * 86_400 + 1_577_836_800 }

    #[test]
    fn indexed_read_two_lenses_scope_and_filters() {
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);

        // note_meta: A/B/C/E/G in the library; D/F/H dependencies; Z outside the library.
        let nm = |p: &str, n: &str, cid: &str, modified: i64, cca: Option<i64>| (p.to_string(), n.to_string(), cid.to_string(), modified, cca);
        for (p, n, cid, m, cca) in [
            nm("/lib/A.md", "Alpha", "CIDA", secs("2026-01-01"), None),
            nm("/lib/B.md", "Beta", "CIDB", secs("2026-01-01"), None),
            nm("/lib/C.md", "Gamma", "CIDC", secs("2026-01-01"), None),
            nm("/lib/E.md", "Epsilon", "CIDE", secs("2026-01-01"), None),
            nm("/lib/G.md", "Gee", "CIDG", secs("2026-01-01"), None),
            nm("/lib/D.md", "Delta-dep", "CIDD", secs("2026-01-01"), Some(secs("2026-06-10"))), // changed AFTER C's review
            nm("/lib/F.md", "Foxtrot-dep", "CIDF", secs("2026-01-01"), Some(secs("2026-06-10"))),
            nm("/lib/H.md", "Hotel-dep", "CIDH", secs("2026-05-01"), None),                      // NULL cca → falls back to modified (BEFORE review)
            nm("/other/Z.md", "Zulu", "CIDZ", secs("2026-01-01"), None),                          // outside the library
        ] {
            c.execute("INSERT INTO note_meta (path,name,cid_cn,modified,content_changed_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![p, n, cid, m, cca]).unwrap();
        }
        // review_schedule: A due (lens 1); B dismissed (excluded); C/E/G reviewed but NOT due
        // (so any appearance is purely lens 2); Z due but out-of-library.
        for (p, reason, due, lr) in [
            ("/lib/A.md", "interval_due", today_days - 1, Some("2026-06-01")),
            ("/lib/B.md", "dismissed", 0i64, None),
            ("/lib/C.md", "interval_due", today_days + 100, Some("2026-06-01")),
            ("/lib/E.md", "interval_due", today_days + 100, Some("2026-06-01")),
            ("/lib/G.md", "interval_due", today_days + 100, Some("2026-06-01")),
            ("/other/Z.md", "interval_due", today_days - 1, Some("2026-06-01")),
        ] {
            c.execute("INSERT INTO review_schedule (path,reason,due_days,last_reviewed,stratum) VALUES (?1,?2,?3,?4,3)",
                rusqlite::params![p, reason, due, lr]).unwrap();
        }
        // links: C→D derives-from (load-bearing) ; E→F associative (excluded) ; G→H derives-from (dep unchanged since review)
        for (src, tname, tcid, lt) in [
            ("/lib/C.md", "Delta-dep", "CIDD", "derives-from"),
            ("/lib/E.md", "Foxtrot-dep", "CIDF", "associative"),
            ("/lib/G.md", "Hotel-dep", "CIDH", "derives-from"),
        ] {
            c.execute("INSERT INTO note_links (source_path,target_name,target_cid_cn,link_type,status,weight) VALUES (?1,?2,?3,?4,'active',2.0)",
                rusqlite::params![src, tname, tcid, lt]).unwrap();
        }

        let due = query_due_notes_indexed(&c, "/lib/", today, today_days, 1).unwrap();
        let got: std::collections::HashSet<(String, String)> =
            due.iter().map(|d| (d.note_path.clone(), d.reason.clone())).collect();

        // Lens 1: only A is due. Lens 2: only C is stale (E associative-excluded; G dep
        // unchanged-since-review via COALESCE→modified; Z out-of-library; B dismissed).
        assert_eq!(got.len(), 2, "exactly A(due) + C(stale); got {:?}", got);
        assert!(got.contains(&("/lib/A.md".into(), "interval_due".into())));
        assert!(got.contains(&("/lib/C.md".into(), "stale".into())));
        assert!(!got.iter().any(|(p, _)| p == "/other/Z.md"), "library scope excludes /other");

        // The stale row explains itself.
        let c_row = due.iter().find(|d| d.note_path == "/lib/C.md").unwrap();
        assert_eq!(c_row.stale_trigger_type.as_deref(), Some("derives-from"));
        assert_eq!(c_row.stale_trigger_name.as_deref(), Some("Delta-dep"));
        assert_eq!(c_row.stale_changed_on.as_deref(), Some("2026-06-10"));
    }

    #[test]
    fn indexed_read_dedups_to_most_consequential_dependency() {
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        // One source S reviewed 2026-06-01, with TWO changed load-bearing deps of
        // different weight. Mode 2 must surface S ONCE, citing the heavier link.
        for (p, n, cid, cca) in [
            ("/lib/S.md", "Source", "CIDS", None),
            ("/lib/Light.md", "Light", "CIDL", Some(secs("2026-06-05"))),
            ("/lib/Heavy.md", "Heavy", "CIDH", Some(secs("2026-06-05"))),
        ] {
            c.execute("INSERT INTO note_meta (path,name,cid_cn,modified,content_changed_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![p, n, cid, secs("2026-01-01"), cca]).unwrap();
        }
        c.execute("INSERT INTO review_schedule (path,reason,due_days,last_reviewed,stratum) VALUES ('/lib/S.md','interval_due',?1,'2026-06-01',1)",
            rusqlite::params![today_days + 100]).unwrap();
        c.execute("INSERT INTO note_links (source_path,target_name,target_cid_cn,link_type,status,weight) VALUES ('/lib/S.md','Light','CIDL','supports','active',1.0)", []).unwrap();
        c.execute("INSERT INTO note_links (source_path,target_name,target_cid_cn,link_type,status,weight) VALUES ('/lib/S.md','Heavy','CIDH','supports','active',5.0)", []).unwrap();

        let due = query_due_notes_indexed(&c, "/lib/", today, today_days, 1).unwrap();
        let stale: Vec<_> = due.iter().filter(|d| d.reason == "stale").collect();
        assert_eq!(stale.len(), 1, "S surfaces once, not once-per-dep");
        assert_eq!(stale[0].stale_trigger_name.as_deref(), Some("Heavy"), "cites the heaviest link");
    }

    #[test]
    fn snooze_hides_from_due_not_from_stale() {
        // Boss 2026-06-22: the lenses are SEPARATE. Snooze hides a note from time-based
        // "Due for Review" (Lens-1) but NOT from "Stale" (Lens-2) — staleness is a
        // distinct signal. S is reviewed + due-by-interval + snoozed into the future,
        // AND has a changed load-bearing dep → must appear ONLY as stale, not as due.
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        for (p, n, cid, cca) in [
            ("/lib/S.md", "Snoozed", "CIDS", None),
            ("/lib/Dep.md", "Dep", "CIDD", Some(secs("2026-06-10"))),
        ] {
            c.execute("INSERT INTO note_meta (path,name,cid_cn,modified,content_changed_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![p, n, cid, secs("2026-01-01"), cca]).unwrap();
        }
        // due_days in the past (would be due) BUT snoozed_until in the future.
        c.execute("INSERT INTO review_schedule (path,reason,due_days,last_reviewed,stratum,snoozed_until) VALUES ('/lib/S.md','interval_due',?1,'2026-06-01',2,'2099-01-01')",
            rusqlite::params![today_days - 5]).unwrap();
        c.execute("INSERT INTO note_links (source_path,target_name,target_cid_cn,link_type,status,weight) VALUES ('/lib/S.md','Dep','CIDD','supports','active',2.0)", []).unwrap();

        let due = query_due_notes_indexed(&c, "/lib/", today, today_days, 1).unwrap();
        let s_reasons: Vec<&str> = due.iter().filter(|d| d.note_path == "/lib/S.md").map(|d| d.reason.as_str()).collect();
        assert_eq!(s_reasons, vec!["stale"],
            "snoozed note must be HIDDEN from Due (Lens-1) but STILL shown as Stale (Lens-2); got {:?}", s_reasons);
    }

    #[test]
    fn stale_grace_period_gates_by_days() {
        // Boss 2026-06-22: a configurable grace period (min 1). A dependency must have
        // changed at least `grace` days after the review to flag stale.
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        // S reviewed 2026-06-01; dep changed 2026-06-15 (~14 days later — wide enough
        // that a ±1-day timezone shift in local_day can't flip the assertions).
        for (p, n, cid, cca) in [
            ("/lib/S.md", "S", "CIDS", None),
            ("/lib/D.md", "D", "CIDD", Some(secs("2026-06-15"))),
        ] {
            c.execute("INSERT INTO note_meta (path,name,cid_cn,modified,content_changed_at) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![p, n, cid, secs("2026-01-01"), cca]).unwrap();
        }
        c.execute("INSERT INTO review_schedule (path,reason,due_days,last_reviewed,stratum) VALUES ('/lib/S.md','interval_due',?1,'2026-06-01',1)",
            rusqlite::params![today_days + 100]).unwrap();
        c.execute("INSERT INTO note_links (source_path,target_name,target_cid_cn,link_type,status,weight) VALUES ('/lib/S.md','D','CIDD','supports','active',1.0)", []).unwrap();

        let stale = |grace: i64| query_due_notes_indexed(&c, "/lib/", today, today_days, grace).unwrap().iter().any(|d| d.reason == "stale");
        assert!(stale(1), "grace 1: a ~14-day-later change is stale");
        assert!(stale(5), "grace 5: still stale");
        assert!(!stale(30), "grace 30: a ~14-day-later change is NOT yet stale");
    }

    #[test]
    fn library_scope_excludes_sibling_prefix() {
        // "/U/Lib" must NOT match the sibling "/U/Lib2" (review finding D).
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        for (p, n) in [("/U/Lib/a.md", "A"), ("/U/Lib2/b.md", "B"), ("/U/Library/c.md", "C")] {
            c.execute("INSERT INTO note_meta (path,name,cid_cn,modified) VALUES (?1,?2,?2,0)", rusqlite::params![p, n]).unwrap();
            c.execute("INSERT INTO review_schedule (path,reason,due_days,stratum) VALUES (?1,'never_reviewed',?2,1)",
                rusqlite::params![p, today_days - 1]).unwrap();
        }
        let due = query_due_notes_indexed(&c, "/U/Lib", today, today_days, 1).unwrap();
        let paths: Vec<&str> = due.iter().map(|d| d.note_path.as_str()).collect();
        assert_eq!(paths, vec!["/U/Lib/a.md"], "only the real child; siblings /U/Lib2 + /U/Library excluded");
    }

    #[test]
    fn indexed_read_excludes_orphan_rows() {
        // A due review_schedule row with NO backing note_meta (an orphan, e.g. left by
        // a rename before §D's migration) must NOT surface as a phantom queue entry.
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        c.execute("INSERT INTO review_schedule (path,reason,due_days,stratum) VALUES ('/lib/ghost.md','never_reviewed',?1,1)",
            rusqlite::params![today_days - 1]).unwrap();
        let due = query_due_notes_indexed(&c, "/lib/", today, today_days, 1).unwrap();
        assert!(due.is_empty(), "orphan row (no note_meta) must not surface; got {:?}", due.iter().map(|d| &d.note_path).collect::<Vec<_>>());
    }

    #[test]
    fn upsert_preserves_active_snooze_across_reindex() {
        // review finding E (#7): a re-index (save / rename-cascade) must NOT drop a snooze.
        let c = sched_db();
        let far = "2099-01-01";
        c.execute("INSERT INTO review_schedule (path,reason,due_days,is_checkpoint,last_reviewed,stratum,interval,snoozed_until)
                   VALUES ('/n.md','interval_due',?1,0,?2,2,7,?3)",
            rusqlite::params![date_to_days(far), day_to_date(100), far]).unwrap();
        upsert_schedule_row(&c, "/n.md", "[]", 0, 5).unwrap(); // simulate re-index
        let (dd, snz): (i64, Option<String>) = c.query_row(
            "SELECT due_days, snoozed_until FROM review_schedule WHERE path='/n.md'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(snz.as_deref(), Some(far), "snooze preserved across re-index");
        assert_eq!(dd, date_to_days(far), "due_days kept at the snooze day (not reset to lr+interval=107)");
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
