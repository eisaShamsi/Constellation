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
    // MIG-084 §B — rich-Reviewer decision context (all from write-time note_meta columns,
    // Rule 8). incoming/outgoing = active-link counts; maturity = the named vocabulary
    // (seed/sapling/evergreen/canonical/wilting) derived via maturity::compute_state.
    pub incoming_count: i64,
    pub outgoing_count: i64,
    pub maturity: String,
    pub word_count: i64,                  // MIG-084 §F.2 — for the orphan diagnosis
    // MIG-084 §F.2 — the user OVERRIDE for review priority (None = "use the computed
    // score"; the frontend computes the effective priority from these signals via the
    // priorities.ts engine). 0..100 = an explicit override. Filled by the post-lens pass.
    pub priority_override: Option<i64>,
    // MIG-084 §F.2-fix — the note's CANONICAL reason (highest-precedence lens it appears
    // in: stale > fragile > orphan > checkpoint > interval_due > never_reviewed). The
    // PRIORITY engine reads THIS (not the per-row lens reason) so a multi-lens note has
    // ONE priority across all its rows AND matches the note tab. Filled by the post-pass.
    pub alarm_reason: Option<String>,
}

/// Get all notes due for review in a library.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
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
/// The library-scope WHERE fragment for bind placeholder `p` (e.g. "?2"). A note is
/// in scope iff its path begins with the (separator-trimmed) library prefix AND the
/// next char is a separator — so "/U/Lib" matches "/U/Lib/x.md" but NOT the sibling
/// "/U/Lib2/y.md" (review finding D). Matches either '/' or '\' (=char(92)) so it's
/// correct for POSIX and Windows path forms alike. An empty prefix ⇒ whole universe.
/// Single-sourced across both lenses so finding D's guard can't drift between them.
fn scope_clause(p: &str, col: &str) -> String {
    format!("({p} = '' OR (substr({col}, 1, length({p})) = {p} AND substr({col}, length({p}) + 1, 1) IN ('/', char(92))))")
}

/// MIG-083 — the Mode-2 staleness probe for ONE note (`?1` = its path): its active
/// load-bearing out-links whose dependency has a hash-confirmed content change,
/// most-consequential first (weight, then most-recent change, then id). Self-links
/// excluded. Single-sourced by both the Lens-2 read and `get_note_review_status` (§F).
fn stale_probe_sql() -> String {
    format!(
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
        types = staleness_types_sql(),
    )
}

/// Core of the staleness probe, against a CALLER-PREPARED statement (so the Lens-2
/// bulk read prepares [`stale_probe_sql`] ONCE and reuses it across the reviewed set).
/// Returns the MOST consequential changed load-bearing dependency `(link_type,
/// dep_name, dep_changed_local_day)` — content changed ≥ `grace` (min 1) LOCAL days
/// AFTER `last_reviewed` — or `None` (incl. a malformed `last_reviewed`, never day 0).
pub(crate) fn note_stale_status_with_stmt(
    probe: &mut rusqlite::Statement,
    source_path: &str,
    last_reviewed: &str,
    grace: i64,
) -> Result<Option<(String, String, i64)>, String> {
    let lr_day = match parse_day(last_reviewed) {
        Some(d) => d,
        None => return Ok(None),
    };
    let grace = grace.max(1);
    let mut rows = probe
        .query_map(rusqlite::params![source_path], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
        })
        .map_err(|e| format!("stale probe: {}", e))?;
    // Rows arrive most-consequential first; the first whose dependency changed ≥ grace
    // LOCAL days after the review is the answer.
    while let Some(Ok((link_type, dep_name, cca))) = rows.next() {
        let dep_day = local_day(cca);
        if dep_day - lr_day >= grace {
            return Ok(Some((link_type, dep_name, dep_day)));
        }
    }
    Ok(None)
}

/// Single-note convenience wrapper: prepares the probe + delegates. Used by the §F
/// note-status tab (`get_note_review_status`) where there's exactly one note.
pub(crate) fn note_stale_status(
    conn: &rusqlite::Connection,
    source_path: &str,
    last_reviewed: &str,
    grace: i64,
) -> Result<Option<(String, String, i64)>, String> {
    let mut probe = conn.prepare(&stale_probe_sql()).map_err(|e| format!("stale probe prepare: {}", e))?;
    note_stale_status_with_stmt(&mut probe, source_path, last_reviewed, grace)
}

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
    // MIG-084 §B — deterministic "now" for maturity (UTC-midnight of today, same
    // day-frame as the schedule), so the maturity label is reproducible in tests.
    let now_secs = day_midnight_secs(today_days);
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
        // INNER JOIN: a row with no backing note_meta (an orphan — e.g. left by some
        // non-delete path) must NEVER surface as a phantom queue entry pointing at a
        // dead path (re-verify finding). No note_meta → not a note.
        let sql = format!(
            "SELECT rs.path, nm.name, rs.reason, rs.due_days, rs.stratum, rs.last_reviewed,
                    nm.incoming_count, nm.outgoing_count, nm.created_at, nm.modified
             FROM review_schedule rs
             JOIN note_meta nm ON nm.path = rs.path
             WHERE rs.due_days <= ?1
               AND rs.reason != 'dismissed'
               AND (rs.snoozed_until IS NULL OR rs.snoozed_until <= ?3)
               AND {scope}",
            scope = scope_clause("?2", "rs.path"),
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| format!("due lens-1 prepare: {}", e))?;
        let rows = stmt
            .query_map(rusqlite::params![today_days, library_path, today], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                    r.get::<_, Option<i64>>(8)?,
                    r.get::<_, i64>(9)?,
                ))
            })
            .map_err(|e| format!("due lens-1 query: {}", e))?;
        for row in rows.flatten() {
            let (path, name, reason, due_days, stratum, last_reviewed, inc, out, created_at, modified) = row;
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
                incoming_count: inc,
                outgoing_count: out,
                maturity: maturity_label(inc, created_at, modified, now_secs),
                word_count: 0, priority_override: None, alarm_reason: None, // filled by the §F.2 post-passes below
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
        let reviewed: Vec<(String, String, i64, String, i64, i64, Option<i64>, i64)> = {
            // NOTE (Boss 2026-06-22): snooze does NOT suppress the Stale lens — the two
            // lenses stay fully separate. Snooze hides a note from time-based "Due for
            // Review" (Lens-1) only; staleness is a distinct signal (a dependency
            // changed) and still surfaces while snoozed.
            let sql = format!(
                "SELECT rs.path, nm.name, rs.stratum, rs.last_reviewed,
                        nm.incoming_count, nm.outgoing_count, nm.created_at, nm.modified
                 FROM review_schedule rs
                 JOIN note_meta nm ON nm.path = rs.path
                 WHERE rs.last_reviewed IS NOT NULL
                   AND rs.reason != 'dismissed'
                   AND {scope}",
                scope = scope_clause("?1", "rs.path"),
            );
            let mut stmt = conn.prepare(&sql).map_err(|e| format!("due lens-2 reviewed prepare: {}", e))?;
            let rows = stmt
                .query_map(rusqlite::params![library_path], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, String>(3)?,
                        r.get::<_, i64>(4)?, r.get::<_, i64>(5)?, r.get::<_, Option<i64>>(6)?, r.get::<_, i64>(7)?))
                })
                .map_err(|e| format!("due lens-2 reviewed query: {}", e))?;
            rows.flatten().collect()
        };

        // Prepare the staleness probe ONCE and reuse it across the reviewed set
        // (`note_stale_status_with_stmt`) — single-sources the SQL + day-comparison
        // with `get_note_review_status` (§F) without re-preparing per note.
        let mut probe = conn.prepare(&stale_probe_sql()).map_err(|e| format!("lens-2 stale probe prepare: {}", e))?;
        for (path, name, stratum, last_reviewed, inc, out, created_at, modified) in reviewed {
            if let Some((link_type, dep_name, dep_day)) = note_stale_status_with_stmt(&mut probe, &path, &last_reviewed, grace)? {
                due.push(DueNote {
                    note_path: path,
                    note_name: name,
                    reason: "stale".to_string(),
                    days_overdue: (today_days - dep_day).max(0),
                    stratum: stratum.clamp(0, 255) as u8,
                    last_reviewed: Some(last_reviewed),
                    stale_trigger_name: Some(dep_name),
                    stale_trigger_type: Some(link_type),
                    stale_changed_on: Some(day_to_date(dep_day)),
                    incoming_count: inc,
                    outgoing_count: out,
                    maturity: maturity_label(inc, created_at, modified, now_secs),
                    word_count: 0, priority_override: None, alarm_reason: None, // filled by the §F.2 post-passes below
                });
            }
        }
    }

    // ── Lens: Orphan (MIG-084 §C) — a note with real content that NOTHING links to
    // yet: the shared UNREFERENCED predicate (crate::connectivity, MIG-094) AND this
    // surface's own substance floor (word_count > 20 — a per-surface lens, not baked
    // into the shared definition). An orphan is an ALARM ("connect me"), NEVER
    // disposable (Eisa 2026-06-23): surfaced regardless of review schedule, oldest-first
    // (days_overdue = age). A note dismissed from review is excluded (LEFT JOIN). All
    // from write-time note_meta columns — no FS walk (Rule 8). ──
    {
        let sql = format!(
            "SELECT nm.path, nm.name, nm.incoming_count, nm.outgoing_count, nm.created_at, nm.modified, rs.last_reviewed
             FROM note_meta nm
             LEFT JOIN review_schedule rs ON rs.path = nm.path
             WHERE {unreferenced} AND nm.word_count > 20
               AND (rs.reason IS NULL OR rs.reason != 'dismissed')
               AND {scope}",
            unreferenced = crate::connectivity::unreferenced_where("nm"),
            scope = scope_clause("?1", "nm.path"),
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| format!("orphan lens prepare: {}", e))?;
        let rows = stmt.query_map(rusqlite::params![library_path], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?, r.get::<_, i64>(5)?, r.get::<_, Option<String>>(6)?))
        }).map_err(|e| format!("orphan lens query: {}", e))?;
        for row in rows.flatten() {
            let (path, name, inc, out, created_at, modified, last_reviewed) = row;
            due.push(DueNote {
                note_path: path, note_name: name, reason: "orphan".to_string(),
                days_overdue: (now_secs - created_at.unwrap_or(modified)).max(0) / 86_400, // age, oldest-first
                stratum: 0,
                last_reviewed,
                stale_trigger_name: None, stale_trigger_type: None, stale_changed_on: None,
                incoming_count: inc, outgoing_count: out,
                maturity: maturity_label(inc, created_at, modified, now_secs),
                word_count: 0, priority_override: None, alarm_reason: None, // filled by the §F.2 post-passes below
            });
        }
    }

    // ── Lens: Fragile / single-point-of-failure (MIG-084 §C) — many notes depend on
    // this one but it rests on ≤1 `derives-from` support: the shared FRAGILE predicate
    // (crate::connectivity, MIG-094), reading the derives support from the write-time
    // outgoing_link_types_json map instead of a per-row note_links subquery (proven
    // occurrence-count-equivalent by the §2 parity test). "Shore me up." Most-depended-on
    // first (days_overdue = incoming_count). Dismissed-excluded. All note_meta (Rule 8). ──
    {
        let sql = format!(
            "SELECT nm.path, nm.name, nm.incoming_count, nm.outgoing_count, nm.created_at, nm.modified, rs.last_reviewed
             FROM note_meta nm
             LEFT JOIN review_schedule rs ON rs.path = nm.path
             WHERE {fragile}
               AND (rs.reason IS NULL OR rs.reason != 'dismissed')
               AND {scope}",
            fragile = crate::connectivity::fragile_where("nm"),
            scope = scope_clause("?1", "nm.path"),
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| format!("fragile lens prepare: {}", e))?;
        let rows = stmt.query_map(rusqlite::params![library_path], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?,
                r.get::<_, Option<i64>>(4)?, r.get::<_, i64>(5)?, r.get::<_, Option<String>>(6)?))
        }).map_err(|e| format!("fragile lens query: {}", e))?;
        for row in rows.flatten() {
            let (path, name, inc, out, created_at, modified, last_reviewed) = row;
            due.push(DueNote {
                note_path: path, note_name: name, reason: "fragile".to_string(),
                days_overdue: inc, // most-depended-on first
                stratum: 0,
                last_reviewed,
                stale_trigger_name: None, stale_trigger_type: None, stale_changed_on: None,
                incoming_count: inc, outgoing_count: out,
                maturity: maturity_label(inc, created_at, modified, now_secs),
                word_count: 0, priority_override: None, alarm_reason: None, // filled by the §F.2 post-passes below
            });
        }
    }

    if !due.is_empty() {
        // MIG-084 §F.2 — stamp each row with its priority OVERRIDE (NULL = use computed)
        // and word_count, in one scoped pass. The effective priority + the queue's
        // priority ordering are computed FRONTEND-side (priorities.ts) from these signals,
        // so the backend carries only the override, not a duplicated formula.
        // MIG-084 §G perf — fetch review_priority + word_count for ONLY the due notes
        // (by their paths), not a full note_meta scan. The old `WHERE {scope}` scanned all
        // in-scope rows (fat — they carry body_text) regardless of how few were due — ~480 ms
        // on the 7,660-note universe. Path is the PK, so this is N indexed lookups.
        let mut meta: std::collections::HashMap<String, (Option<i64>, i64)> = std::collections::HashMap::new();
        let mut paths: Vec<String> = due.iter().map(|d| d.note_path.clone()).collect();
        paths.sort();
        paths.dedup();
        let placeholders = std::iter::repeat("?").take(paths.len()).collect::<Vec<_>>().join(",");
        let sql = format!("SELECT path, review_priority, word_count FROM note_meta WHERE path IN ({})", placeholders);
        let mut stmt = conn.prepare(&sql).map_err(|e| format!("priority fetch prepare: {}", e))?;
        let params: Vec<&dyn rusqlite::ToSql> = paths.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        let rows = stmt
            .query_map(params.as_slice(), |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?, r.get::<_, i64>(2)?))
            })
            .map_err(|e| format!("priority fetch: {}", e))?;
        for row in rows.flatten() { meta.insert(row.0, (row.1, row.2)); }
        for d in due.iter_mut() {
            if let Some((ovr, wc)) = meta.get(&d.note_path) { d.priority_override = *ovr; d.word_count = *wc; }
        }

        // MIG-084 §F.2-fix — stamp each row with the note's CANONICAL reason (highest-
        // precedence lens it appears in), so a multi-lens note computes ONE priority across
        // all its rows AND matches get_note_review_status (the note tab). Precedence mirrors
        // the note tab's: stale > fragile > orphan > checkpoint > interval_due > never_reviewed.
        fn precedence(r: &str) -> u8 {
            match r {
                "stale" => 0, "fragile" => 1, "orphan" => 2,
                "checkpoint" => 3, "interval_due" => 4, "never_reviewed" => 5, _ => 6,
            }
        }
        let mut canonical: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for d in &due {
            let cur = canonical.entry(d.note_path.clone()).or_insert_with(|| d.reason.clone());
            if precedence(&d.reason) < precedence(cur) { *cur = d.reason.clone(); }
        }
        for d in due.iter_mut() { d.alarm_reason = canonical.get(&d.note_path).cloned(); }

        // MIG-084 §G (audit P2) — a note's PRIORITY must be identical across all its rows
        // AND match the note tab. alarm_reason unifies the reason, but `days_overdue` is
        // overloaded per lens (orphan = age, fragile = inbound-count, time-lenses = days-
        // since-due). The note tab's days_overdue is the SCHEDULE value (today-due), so adopt
        // a TIME-LENS row's days_overdue (never/interval_due/checkpoint/stale) as the note's
        // canonical days for ALL its rows; a note with no time-lens row (a pure orphan/
        // fragile) keeps its own. days_overdue display per row is unaffected (orphan/fragile
        // why-lines don't print it). This makes the per-row computedPriority converge.
        let time_lens = |r: &str| matches!(r, "never_reviewed" | "interval_due" | "checkpoint" | "stale");
        let mut canon_days: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for d in &due {
            if time_lens(&d.reason) {
                canon_days.entry(d.note_path.clone()).or_insert(d.days_overdue);
            }
        }
        for d in due.iter_mut() {
            if let Some(days) = canon_days.get(&d.note_path) { d.days_overdue = *days; }
        }
    }

    // Sort: higher stratum first, then more overdue (the legacy tie-break). The Reviewer
    // re-sorts by EFFECTIVE priority (override ?? computed) frontend-side — this is just a
    // stable initial order.
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
    // MIG-080 §F — Mode-2 staleness for THIS note (so the note-context tab answers
    // "is this note due OR stale?", not just due). is_stale + the triggering neighbour.
    pub is_stale: bool,
    pub stale_trigger_name: Option<String>,
    pub stale_trigger_type: Option<String>,
    pub stale_changed_on: Option<String>,
    // MIG-084 §F.2 — the priority OVERRIDE (None = use the computed score) + word_count,
    // so the note tab mirrors the Reviewer's computed-priority + prescription logic.
    pub priority_override: Option<i64>,
    pub word_count: i64,
    pub incoming_count: i64,
    pub outgoing_count: i64,
    pub maturity: String,
    pub days_overdue: i64,
    // MIG-084 §F.2 — the resolved lens reason (stale > fragile > orphan > schedule), so
    // the note tab feeds the SAME `reason` to the priority engine the Reviewer uses (the
    // schedule `reason` alone never carries stale/orphan/fragile). None = no alarm/schedule.
    pub alarm_reason: Option<String>,
}

/// MIG-083 §D / MIG-080 §F — one note's Review-Pulse status: the O(1) `review_schedule`
/// PK lookup (Mode 1/3) PLUS the per-note Mode-2 staleness probe. Read-only, keyed by an
/// already-open note's path (no fs access; the frontend only asks for notes it opened).
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn get_note_review_status(
    app: tauri::AppHandle,
    note_path: String,
    stale_grace_days: Option<i64>,
) -> Result<NoteReviewStatus, String> {
    let grace = stale_grace_days.unwrap_or(1);
    if let Some(state) = app.try_state::<crate::search::SearchState>() {
        if let Ok(guard) = state.db.lock() {
            if let Some(conn) = guard.as_ref() {
                // §F.2 — the note's priority OVERRIDE (nullable; None = use computed) +
                // the signals the note tab needs to mirror the Reviewer's computed score
                // (incoming/outgoing counts + the maturity vocabulary). Present even for an
                // unscheduled orphan.
                let today_days = date_to_days(&today_str());
                let now_secs = day_midnight_secs(today_days);
                let (priority_override, word_count, incoming_count, outgoing_count, maturity, oltj) = conn
                    .query_row(
                        "SELECT review_priority, word_count, incoming_count, outgoing_count, created_at, modified, outgoing_link_types_json FROM note_meta WHERE path = ?1",
                        rusqlite::params![note_path],
                        |r| Ok((
                            r.get::<_, Option<i64>>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?,
                            maturity_label(r.get::<_, i64>(2)?, r.get::<_, Option<i64>>(4)?, r.get::<_, i64>(5)?, now_secs),
                            r.get::<_, String>(6)?,
                        )),
                    )
                    .unwrap_or((None, 0, 0, 0, "seed".to_string(), "{}".to_string()));
                // §F.2 — the connection-health lens flags via the shared connectivity
                // predicates (MIG-094) so alarm_reason matches the queue exactly. The
                // derives support is read from the write-time outgoing_link_types_json map
                // (same value as the old note_links COUNT(*) subquery — §2 parity test).
                let is_orphan = crate::connectivity::is_unreferenced(incoming_count) && word_count > 20;
                let is_fragile = crate::connectivity::is_fragile(
                    incoming_count,
                    crate::connectivity::derives_from_support(&oltj),
                );
                let row: Option<(String, i64, Option<String>, i64)> = conn
                    .query_row(
                        "SELECT reason, due_days, last_reviewed, is_checkpoint FROM review_schedule WHERE path = ?1",
                        rusqlite::params![note_path],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                    )
                    .ok();
                if let Some((reason, due_days, last_reviewed, is_cp)) = row {
                    // Mode-2: a reviewed, non-dismissed note can also be stale.
                    let stale = match (&last_reviewed, reason.as_str()) {
                        (Some(lr), r) if r != "dismissed" => note_stale_status(conn, &note_path, lr, grace)?,
                        _ => None,
                    };
                    return Ok(NoteReviewStatus {
                        never_reviewed: last_reviewed.is_none(),
                        reason: Some(reason.clone()),
                        due_days: Some(due_days),
                        last_reviewed,
                        is_checkpoint: is_cp != 0,
                        is_stale: stale.is_some(),
                        stale_trigger_type: stale.as_ref().map(|(t, _, _)| t.clone()),
                        stale_trigger_name: stale.as_ref().map(|(_, n, _)| n.clone()),
                        stale_changed_on: stale.as_ref().map(|(_, _, d)| day_to_date(*d)),
                        priority_override, word_count, incoming_count, outgoing_count, maturity,
                        days_overdue: (today_days - due_days).max(0),
                        // stale > fragile > orphan > the schedule reason. A DISMISSED note
                        // resolves to "dismissed" (NOT fragile/orphan): every get_due_notes
                        // lens excludes dismissed, so the note tab must not fabricate an alarm
                        // priority for a note the queue shows nowhere (audit §G drift).
                        alarm_reason: Some(if stale.is_some() { "stale".into() }
                            else if reason != "dismissed" && is_fragile { "fragile".into() }
                            else if reason != "dismissed" && is_orphan { "orphan".into() }
                            else { reason }),
                    });
                }
                // note_meta exists but no review_schedule row (e.g. an orphan): clean
                // never-reviewed status, still carrying the override + signals.
                return Ok(NoteReviewStatus {
                    reason: None, due_days: None, last_reviewed: None, never_reviewed: true,
                    is_checkpoint: false, is_stale: false, stale_trigger_name: None,
                    stale_trigger_type: None, stale_changed_on: None, priority_override, word_count,
                    incoming_count, outgoing_count, maturity, days_overdue: 0,
                    alarm_reason: if is_fragile { Some("fragile".into()) } else if is_orphan { Some("orphan".into()) } else { None },
                });
            }
        }
    }
    // No SearchState/conn at all (unstamped boot): a clean default.
    Ok(NoteReviewStatus {
        reason: None,
        due_days: None,
        last_reviewed: None,
        never_reviewed: true,
        is_checkpoint: false,
        is_stale: false,
        stale_trigger_name: None,
        stale_trigger_type: None,
        stale_changed_on: None,
        priority_override: None,
        word_count: 0,
        incoming_count: 0,
        outgoing_count: 0,
        maturity: "seed".to_string(),
        days_overdue: 0,
        alarm_reason: None,
    })
}

/// MIG-084 §F.2 — set (or CLEAR) a note's review-priority override. `Some(0..100)`
/// clamps + writes an explicit override; `None` writes NULL = "use the computed score"
/// (the Reset-to-computed action). A user-owned lever on `note_meta` (survives
/// re-indexing — index_note's conflict-update omits this column). The Reviewer detail
/// pane + the note's Review tab both call this.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn set_review_priority(
    app: tauri::AppHandle,
    note_path: String,
    priority: Option<i64>,
) -> Result<(), String> {
    let value: Option<i64> = priority.map(|p| p.clamp(0, 100));
    if let Some(state) = app.try_state::<crate::search::SearchState>() {
        if let Ok(guard) = state.db.lock() {
            if let Some(conn) = guard.as_ref() {
                conn.execute(
                    "UPDATE note_meta SET review_priority = ?2 WHERE path = ?1",
                    rusqlite::params![note_path, value],
                )
                .map_err(|e| format!("set_review_priority: {}", e))?;
                return Ok(());
            }
        }
    }
    Err("database not ready".to_string())
}

// Note-open-freeze class fix (2026-07-03): the three review actions each do a
// whole-file read-modify-write of review-pulse.json (load_pulse_data →
// save_pulse_data). As SYNC commands the single IPC dispatch thread serialized
// them; as `(async)` they can interleave on Tokio workers — two concurrent RMWs
// would lose the earlier write. PULSE_LOCK serializes the RMW critical section.
static PULSE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Mark a note as reviewed. Advances to the next interval on the 1·3·7·14·30 ladder.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn mark_reviewed(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let _pulse_guard = PULSE_LOCK.lock().map_err(|e| e.to_string())?;
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
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn snooze_note(
    app: tauri::AppHandle,
    note_path: String,
    days: u32,
) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let _pulse_guard = PULSE_LOCK.lock().map_err(|e| e.to_string())?;
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
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn dismiss_note(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let _pulse_guard = PULSE_LOCK.lock().map_err(|e| e.to_string())?;
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
        match fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(pulse) => return pulse,
                Err(e) => {
                    // 2026-07-25 inspection (PJ-140): a parse failure used to fall
                    // through to default() SILENTLY — a corrupt review-pulse.json
                    // discarded ALL of the user's review history (last_reviewed,
                    // intervals, snoozes; earned, lives only here) with no error and
                    // no recoverable copy. Now: back the bad file aside before
                    // starting fresh, so nothing is destroyed and it can be recovered.
                    // (Mirrors try_load_libraries' corrupt-registry contract.)
                    let aside = path.with_extension(format!(
                        "corrupt-{}.json",
                        chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
                    ));
                    let _ = fs::rename(&path, &aside);
                    eprintln!(
                        "[review] review-pulse.json unparseable ({e}); backed up to {} and starting fresh",
                        aside.display()
                    );
                }
            },
            Err(e) => {
                // A read error (lock, transient IO) is NOT corruption — do not
                // touch the file; just use defaults for this boot and retry next time.
                eprintln!("[review] review-pulse.json unreadable ({e}); using defaults this session");
            }
        }
    }
    ReviewPulseData::default()
}

fn save_pulse_data(cdir: &Path, pulse: &ReviewPulseData) -> Result<(), String> {
    let path = cdir.join("review-pulse.json");
    let json = serde_json::to_string_pretty(pulse).map_err(|e| e.to_string())?;
    // Atomic write (fsync-before-rename) — a plain fs::write could leave a
    // truncated/garbage file on crash/power-loss, which the loader above would then
    // (correctly) treat as corrupt and set aside, losing the session's review state.
    crate::universe::atomic_write(&path, json.as_bytes())
        .map_err(|e| format!("Failed to write review-pulse.json: {}", e))
}

pub(crate) fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// The day-number epoch — all `*_days` values are counted from here.
fn epoch_2020() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()
}

pub(crate) fn date_to_days(date_str: &str) -> i64 {
    // Lenient variant: an unparseable date buckets to day 0 (preserves the legacy
    // Mode-1 contract). Mode-2 uses the strict [`parse_day`] instead (finding E).
    parse_day(date_str).unwrap_or(0)
}

/// Strict variant of [`date_to_days`]: `None` when the string isn't a valid
/// `YYYY-MM-DD`. Mode-2 uses this so a malformed `last_reviewed` is SKIPPED rather
/// than silently bucketed to day 0 (2020-01-01), which would make almost every
/// dependency look "changed after review" → spurious staleness (review finding E).
pub(crate) fn parse_day(date_str: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .ok()
        .map(|d| d.signed_duration_since(epoch_2020()).num_days())
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
        Some(dt) => dt.date_naive().signed_duration_since(epoch_2020()).num_days(),
        None => secs_to_days(secs),
    }
}

/// Unix seconds at UTC-midnight of a days-since-2020 value (2020-01-01 = 1577836800).
/// Used to give the Reviewer a deterministic `now` in the same day-frame as the
/// review schedule (vs. wall-clock, which would make tests non-deterministic).
pub(crate) fn day_midnight_secs(days: i64) -> i64 { days * 86_400 + 1_577_836_800 }

/// MIG-084 §B — the named maturity vocabulary for a Reviewer row, derived at READ
/// time from the write-time `note_meta` columns (Rule 8) through the SHARED
/// [`crate::maturity::compute_state`] thresholds (one source of truth). `now_secs`
/// is UTC-midnight of today; a NULL `created_at` falls back to `modified`.
pub(crate) fn maturity_label(inbound: i64, created_at: Option<i64>, modified: i64, now_secs: i64) -> String {
    let created = created_at.unwrap_or(modified).max(0);
    let modified = modified.max(0);
    let dsc = ((now_secs - created).max(0) / 86_400) as u64;
    let dsm = ((now_secs - modified).max(0) / 86_400) as u64;
    crate::maturity::compute_state(inbound.max(0) as usize, dsc, dsm)
}

/// Days-since-2020-01-01 → `YYYY-MM-DD` (the inverse of `date_to_days`). Used to
/// render `stale_changed_on` for the Mode-2 lens.
fn day_to_date(days: i64) -> String {
    (epoch_2020() + chrono::Duration::days(days)).format("%Y-%m-%d").to_string()
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

/// The staleness trigger types as a SQL `IN (…)` list — `'supports','contradicts',…`.
/// Single-sourced for the Mode-2 probe + the rehearsal reference (the values are
/// fixed code constants, so the interpolation is injection-safe).
pub(crate) fn staleness_types_sql() -> String {
    STALENESS_TRIGGER_TYPES
        .iter()
        .map(|t| format!("'{}'", t))
        .collect::<Vec<_>>()
        .join(",")
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

/// The `review_schedule` schema/logic version. Bumping it makes [`is_stamped`] report
/// false until the back-fill re-runs + re-stamps — the sibling rollback/re-build path
/// (mirrors `SKY_SCHEMA_VERSION`): a future change to the scheduling logic bumps this,
/// the next boot rebuilds, and the reconcile self-heals (Architect I7).
pub const REVIEW_SCHEMA_VERSION: i64 = 1;

/// Is the `review_schedule` table built + authoritative at the current version?
/// (schema_versions.review ≥ [`REVIEW_SCHEMA_VERSION`]). Until stamped (by the §C
/// back-fill), `get_due_notes` returns empty + kicks the back-fill — so §B is INERT
/// until §C.
pub fn is_stamped(conn: &rusqlite::Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'review'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .map(|v| v >= REVIEW_SCHEMA_VERSION)
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

/// The shared per-note step of the §C back-fill, the §E reconcile recompute, and the
/// rehearsal harness: rebuild ONE note's `review_schedule` row from the data in hand
/// (real `stratum` from sky_nodes; action state from the pulse) + baseline its
/// `content_hash` from `body_text` if unset. Single-sourced so the three callers
/// can't drift (the rehearsal then exercises the REAL back-fill body).
pub(crate) fn backfill_one(
    conn: &rusqlite::Connection,
    path: &str,
    tags_json: &str,
    modified_secs: i64,
    body_text: &str,
    pulse: &ReviewPulseData,
    today: &str,
) -> Result<(), String> {
    let stratum: i64 = conn
        .query_row(
            "SELECT CAST(stratum AS INTEGER) FROM sky_nodes WHERE path = ?1",
            rusqlite::params![path],
            |r| r.get(0),
        )
        .unwrap_or(0);
    backfill_schedule_row(
        conn,
        path,
        tags_json,
        modified_secs,
        stratum,
        pulse.last_reviewed.get(path).map(|s| s.as_str()),
        pulse.intervals.get(path).copied().unwrap_or(0),
        pulse.snoozed.get(path).map(|s| s.as_str()),
        pulse.dismissed.contains(&path.to_string()),
        today,
    )?;
    // Baseline content_hash (only if unset — resume-safe; content_changed_at stays
    // NULL so nothing is "stale" until a real post-stamp body change bumps it).
    conn.execute(
        "UPDATE note_meta SET content_hash = ?2 WHERE path = ?1 AND content_hash IS NULL",
        rusqlite::params![path, content_hash(body_text)],
    )
    .map_err(|e| format!("baseline content_hash {}: {}", path, e))?;
    Ok(())
}

/// MIG-083 §E — the reconcile self-heal (Plan §C / Architect I1). Authoritatively
/// rebuild `review_schedule` from the just-reconciled `note_meta` + the per-universe
/// action state (`review-pulse.json`): sweep orphan rows (notes gone from disk),
/// refresh every row's content-derived fields (reason/due/stratum/is_checkpoint) and
/// re-anchor the action-owned fields (last_reviewed/interval/snooze/dismiss). So any
/// drift — an orphan, a stratum gone stale via a neighbour's link edit, a row missed
/// in a back-fill inter-batch window — self-heals on the periodic reconcile. Mirrors
/// `tag_counts::recompute_all_in`; the caller wraps it in one transaction.
pub fn recompute_all_in(
    conn: &rusqlite::Connection,
    pulse: &ReviewPulseData,
    today: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM review_schedule WHERE path NOT IN (SELECT path FROM note_meta)",
        [],
    )
    .map_err(|e| format!("review_schedule orphan sweep: {}", e))?;
    let rows: Vec<(String, String, i64, String)> = {
        let mut stmt = conn
            .prepare("SELECT path, COALESCE(tags_json,'[]'), COALESCE(modified,0), COALESCE(body_text,'') FROM note_meta")
            .map_err(|e| e.to_string())?;
        let it = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?, r.get::<_, String>(3)?)))
            .map_err(|e| e.to_string())?;
        it.filter_map(|x| x.ok()).collect()
    };
    for (path, tags_json, modified, body_text) in &rows {
        backfill_one(conn, path, tags_json, *modified, body_text, pulse, today)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests_pulse_durability {
    //! 2026-07-25 inspection (PJ-140): a corrupt review-pulse.json used to silently
    //! fall through to default(), discarding ALL review history with no recoverable
    //! copy. The loader now sets a corrupt file aside before starting fresh.
    use super::{load_pulse_data, save_pulse_data, ReviewPulseData};
    use std::fs;

    fn tmp(tag: &str) -> std::path::PathBuf {
        // Unique per test — Rust runs tests in parallel, so a shared dir name would
        // let one test's cleanup race another's write.
        let d = std::env::temp_dir().join(format!("cns-pulse-{}-{}", std::process::id(), tag));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn a_corrupt_pulse_is_backed_up_not_silently_discarded() {
        let d = tmp("corrupt");
        fs::write(d.join("review-pulse.json"), b"{ this is not valid json ]").unwrap();
        // Loading must NOT delete or overwrite the corrupt file in place; it must be
        // preserved under a .corrupt-*.json name so the history can be recovered.
        let loaded = load_pulse_data(&d);
        assert!(loaded.last_reviewed.is_empty(), "falls back to defaults for THIS session");
        let backed_up = fs::read_dir(&d).unwrap().filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy().contains(".corrupt-"));
        assert!(backed_up, "a corrupt pulse file must be set aside, never silently lost");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn a_valid_pulse_round_trips_atomically() {
        let d = tmp("roundtrip");
        let mut p = ReviewPulseData::default();
        p.last_reviewed.insert("Note.md".into(), "2026-07-25".into());
        p.intervals.insert("Note.md".into(), 14);
        save_pulse_data(&d, &p).unwrap();
        // No stray temp file left behind by the atomic write.
        let stray = fs::read_dir(&d).unwrap().filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy().ends_with(".tmp"));
        assert!(!stray, "atomic_write must not leave a .tmp file on success");
        let back = load_pulse_data(&d);
        assert_eq!(back.last_reviewed.get("Note.md").map(String::as_str), Some("2026-07-25"));
        assert_eq!(back.intervals.get("Note.md").copied(), Some(14));
        let _ = fs::remove_dir_all(&d);
    }
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
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT, cid_cn TEXT, modified INTEGER, content_changed_at INTEGER, content_hash TEXT, tags_json TEXT DEFAULT '[]', body_text TEXT DEFAULT '', incoming_count INTEGER NOT NULL DEFAULT 0, outgoing_count INTEGER NOT NULL DEFAULT 0, outgoing_link_types_json TEXT NOT NULL DEFAULT '{}', created_at INTEGER, word_count INTEGER NOT NULL DEFAULT 0, review_priority INTEGER);
             CREATE TABLE sky_nodes (path TEXT PRIMARY KEY, stratum TEXT);
             CREATE TABLE note_links (id INTEGER PRIMARY KEY AUTOINCREMENT, source_path TEXT, target_name TEXT, target_cid_cn TEXT, link_type TEXT, status TEXT DEFAULT 'active', weight REAL DEFAULT 1.0);
             CREATE TABLE review_schedule (path TEXT PRIMARY KEY, reason TEXT NOT NULL, due_days INTEGER NOT NULL,
               is_checkpoint INTEGER NOT NULL DEFAULT 0, last_reviewed TEXT, stratum INTEGER NOT NULL DEFAULT 0, interval INTEGER NOT NULL DEFAULT 0, snoozed_until TEXT);",
        ).unwrap();
        c
    }
    /// Unix seconds at UTC-midnight of a YYYY-MM-DD date (matches strftime('%s', d)).
    fn secs(date: &str) -> i64 { date_to_days(date) * 86_400 + 1_577_836_800 }

    #[test]
    fn due_notes_carry_connection_counts_and_maturity() {
        // MIG-084 §B — the queue enriches each row with the write-time connection
        // counts + the named maturity vocabulary (derived via maturity::compute_state).
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        // A canonical hub: 12 inbound, 4 outbound, untouched 30+ days.
        c.execute(
            "INSERT INTO note_meta (path,name,cid_cn,modified,incoming_count,outgoing_count,created_at) \
             VALUES ('/lib/H.md','Hub','CIDH',?1,12,4,?2)",
            rusqlite::params![secs("2026-01-01"), secs("2025-06-01")],
        ).unwrap();
        c.execute(
            "INSERT INTO review_schedule (path,reason,due_days,last_reviewed,stratum) \
             VALUES ('/lib/H.md','interval_due',?1,'2026-06-01',5)",
            rusqlite::params![today_days - 1],
        ).unwrap();
        let due = query_due_notes_indexed(&c, "/lib/", today, today_days, 1).unwrap();
        let h = due.iter().find(|d| d.note_path == "/lib/H.md").expect("hub is due");
        assert_eq!(h.incoming_count, 12);
        assert_eq!(h.outgoing_count, 4);
        assert_eq!(h.maturity, "canonical", "12 inbound + untouched 30+ days ⇒ canonical");
    }

    #[test]
    fn orphan_and_fragile_lenses_surface_connection_alarms() {
        // MIG-084 §C — the two connection-health lenses. Orphan = real content, zero
        // inbound. Fragile = many inbound, ≤1 derives-from support. Both from write-time
        // columns; dismissed notes excluded; tiny stubs (word_count ≤ 20) are NOT orphans.
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        let put = |path: &str, name: &str, inc: i64, wc: i64| {
            c.execute("INSERT INTO note_meta (path,name,cid_cn,modified,incoming_count,outgoing_count,created_at,word_count) \
                       VALUES (?1,?2,?3,?4,?5,1,?6,?7)",
                rusqlite::params![path, name, name, secs("2026-01-01"), inc, secs("2026-01-01"), wc]).unwrap();
        };
        put("/lib/Orphan.md", "Lonely", 0, 100);   // orphan: 0 inbound, real content
        put("/lib/Stub.md", "Tiny", 0, 5);          // NOT an orphan: too short
        put("/lib/Hub.md", "Fragile Hub", 8, 300);  // fragile candidate: 8 inbound
        put("/lib/Dismissed.md", "Hidden", 0, 100); // orphan but dismissed → excluded
        c.execute("INSERT INTO review_schedule (path,reason,due_days,stratum) VALUES ('/lib/Dismissed.md','dismissed',0,1)", []).unwrap();
        // Hub has only 1 derives-from out-link ⇒ single point of failure.
        c.execute("INSERT INTO note_links (source_path,target_name,target_cid_cn,link_type,status,weight) \
                   VALUES ('/lib/Hub.md','Some Dep','CIDX','derives-from','active',1.0)", []).unwrap();

        let due = query_due_notes_indexed(&c, "/lib/", today, today_days, 1).unwrap();
        let orphans: Vec<&str> = due.iter().filter(|d| d.reason == "orphan").map(|d| d.note_path.as_str()).collect();
        let fragile: Vec<&str> = due.iter().filter(|d| d.reason == "fragile").map(|d| d.note_path.as_str()).collect();
        assert_eq!(orphans, vec!["/lib/Orphan.md"], "only the real-content, non-dismissed orphan");
        assert_eq!(fragile, vec!["/lib/Hub.md"], "the 8-inbound, single-support hub is fragile");
    }

    #[test]
    fn multi_lens_note_gets_one_canonical_alarm_reason() {
        // MIG-084 §F.2-fix — a note in BOTH the never-reviewed lens AND the orphan lens
        // (0 inbound + real content, never ✓) gets ONE canonical reason on every row
        // (orphan > never_reviewed), so its priority is identical across rows + the note tab.
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        c.execute("INSERT INTO note_meta (path,name,cid_cn,modified,incoming_count,outgoing_count,word_count) \
                   VALUES ('/lib/N.md','Note N','CN',?1,0,1,120)", rusqlite::params![secs("2026-01-01")]).unwrap();
        c.execute("INSERT INTO review_schedule (path,reason,due_days,stratum) VALUES ('/lib/N.md','never_reviewed',?1,3)",
            rusqlite::params![today_days - 5]).unwrap();
        let due = query_due_notes_indexed(&c, "/lib/", today, today_days, 1).unwrap();
        let rows: Vec<&DueNote> = due.iter().filter(|d| d.note_path == "/lib/N.md").collect();
        assert_eq!(rows.len(), 2, "appears in both never_reviewed and orphan lenses");
        for r in &rows {
            assert_eq!(r.alarm_reason.as_deref(), Some("orphan"), "canonical reason is orphan on EVERY row");
        }
        // §G (audit P2) — canonical days_overdue: both rows adopt the time-lens (never_reviewed)
        // value (today - due_days = 5), so the per-row computed priority is identical.
        let days: Vec<i64> = rows.iter().map(|r| r.days_overdue).collect();
        assert_eq!(days, vec![5, 5], "all rows share the time-lens days_overdue (5)");
    }

    #[test]
    fn priority_override_is_carried_nullable() {
        // MIG-084 §F.2 — the backend carries the priority OVERRIDE (NULL = use computed;
        // the effective priority + the ranking are computed frontend-side). B has an
        // explicit override; A has none (NULL) → None reaches the DueNote, not a "50".
        let c = read_db();
        let today = "2026-06-22";
        let today_days = date_to_days(today);
        // A: no override (review_priority omitted → NULL). B: override 90.
        c.execute("INSERT INTO note_meta (path,name,cid_cn,modified) VALUES ('/lib/A.md','Alpha','CA',?1)",
            rusqlite::params![secs("2026-01-01")]).unwrap();
        c.execute("INSERT INTO note_meta (path,name,cid_cn,modified,review_priority) VALUES ('/lib/B.md','Beta','CB',?1,90)",
            rusqlite::params![secs("2026-01-01")]).unwrap();
        for p in ["/lib/A.md", "/lib/B.md"] {
            c.execute("INSERT INTO review_schedule (path,reason,due_days,last_reviewed,stratum) VALUES (?1,'interval_due',?2,'2026-06-01',3)",
                rusqlite::params![p, today_days - 1]).unwrap();
        }
        let due = query_due_notes_indexed(&c, "/lib/", today, today_days, 1).unwrap();
        assert_eq!(due.iter().find(|d| d.note_path == "/lib/A.md").unwrap().priority_override, None, "no override ⇒ None (use computed)");
        assert_eq!(due.iter().find(|d| d.note_path == "/lib/B.md").unwrap().priority_override, Some(90));
    }

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
    fn recompute_all_in_sweeps_orphans_and_rebuilds() {
        // §E reconcile self-heal (Plan §C / Architect I1, audit P1): an orphan row
        // (no backing note_meta) is swept, and every real note gets a fresh row.
        let c = read_db();
        let today = "2026-06-22";
        c.execute("INSERT INTO note_meta (path,name,cid_cn,modified,tags_json) VALUES ('/lib/Real.md','Real','CIDR',?1,'[]')",
            rusqlite::params![secs("2026-01-01")]).unwrap();
        // a stale orphan + a real-but-missing-row (back-fill-window gap)
        c.execute("INSERT INTO review_schedule (path,reason,due_days) VALUES ('/lib/Ghost.md','interval_due',0)", []).unwrap();

        recompute_all_in(&c, &ReviewPulseData::default(), today).unwrap();

        assert_eq!(c.query_row("SELECT COUNT(*) FROM review_schedule WHERE path='/lib/Ghost.md'", [], |r| r.get::<_,i64>(0)).unwrap(), 0,
            "orphan row (no note_meta) swept");
        let (reason, _): (String, i64) = c.query_row("SELECT reason, due_days FROM review_schedule WHERE path='/lib/Real.md'", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(reason, "never_reviewed", "real note (no pulse entry) rebuilt as never_reviewed");
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
