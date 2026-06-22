//! MIG-083 §C — resumable post-paint back-fill of `review_schedule`.
//!
//! Mirrors `sky_backfill`: a background thread (scheduled by `maybe_schedule`,
//! never blocks boot), a `review_backfill_cursor` for crash-resume, 1000-row
//! batches with a 50 ms inter-batch sleep (so it doesn't starve the main thread
//! or saturate WAL), idempotent `INSERT OR REPLACE`, and a stamp of
//! `schema_versions.review = 1` ON COMPLETION — which is the single moment the
//! whole MIG-083 write-time machinery (the §B `index_note`/action hooks) flips
//! from inert to live. Reads stay on the legacy scan until §D swaps them.
//!
//! Sources, all already in the DB (no `.md` reads): `note_meta` (path/tags/
//! modified), `sky_nodes.stratum`, and the per-universe `review-pulse.json`
//! action state (loaded once).

use crate::search::SearchState;
use rusqlite::{params, Connection};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};

const BATCH_SIZE: usize = 1000;
const INTER_BATCH_SLEEP_MS: u64 = 50;

/// Schedule the back-fill on a background thread. Returns immediately. No-op if
/// `review` is already stamped (the common steady-state case).
pub fn maybe_schedule(app: tauri::AppHandle) {
    {
        let state = app.state::<SearchState>();
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        match guard.as_ref() {
            Some(conn) if crate::review::is_stamped(conn) => return, // already built
            None => return,
            _ => {}
        }
    }
    thread::spawn(move || {
        if let Err(e) = run(&app) {
            eprintln!("review_backfill: {}", e);
        }
    });
}

fn run(app: &tauri::AppHandle) -> Result<u64, String> {
    let state = app.state::<SearchState>();
    {
        let mut guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        ensure_cursor_table(conn)?;
    }

    // Load the per-universe action state ONCE (small JSON; the source of truth).
    let cdir = crate::universe::active_constellation_dir(app)?;
    let pulse = crate::review::load_pulse_data(&cdir);
    let today = crate::review::today_str();

    let mut last_path = read_cursor(&state.db)?;
    let mut total: u64 = 0;
    loop {
        let (n, new_last) = process_batch(&state.db, &last_path, &pulse, &today)?;
        if n == 0 {
            finalize(&state.db)?;
            let _ = app.emit("review-backfill-progress", serde_json::json!({ "done": true, "total": total }));
            return Ok(total);
        }
        total += n as u64;
        last_path = new_last;
        write_cursor(&state.db, &last_path)?;
        let _ = app.emit("review-backfill-progress", serde_json::json!({ "done": false, "total": total }));
        thread::sleep(Duration::from_millis(INTER_BATCH_SLEEP_MS));
    }
}

fn process_batch(
    db: &Mutex<Option<Connection>>,
    last_path: &str,
    pulse: &crate::review::ReviewPulseData,
    today: &str,
) -> Result<(usize, String), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;

    // body_text comes along so we can BASELINE note_meta.content_hash (MIG-083 §D,
    // review finding A): with a baseline in place, the first post-stamp save of a
    // dependency that is a mere touch (body unchanged) hashes equal → does NOT bump
    // content_changed_at → does NOT false-fire staleness. Hashing note_meta.body_text
    // (= the same plain_body index_note hashes) keeps the back-fill .md-read-free.
    let rows: Vec<(String, String, i64, String)> = {
        let mut stmt = conn
            .prepare("SELECT path, tags_json, modified, COALESCE(body_text,'') FROM note_meta WHERE path > ?1 ORDER BY path LIMIT ?2")
            .map_err(|e| e.to_string())?;
        let it = stmt
            .query_map(params![last_path, BATCH_SIZE as i64], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    r.get::<_, Option<i64>>(2)?.unwrap_or(0),
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        it.filter_map(|x| x.ok()).collect()
    };
    if rows.is_empty() {
        return Ok((0, last_path.to_string()));
    }
    let new_last = rows.last().unwrap().0.clone();

    // One transaction per batch: each row writes twice (the review_schedule
    // INSERT OR REPLACE + the content_hash baseline UPDATE); without this every
    // write auto-commits, so a 1000-row batch fsyncs ~2000 times. BEGIN IMMEDIATE
    // collapses that to one commit.
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        for (path, tags_json, modified, body_text) in &rows {
            let stratum: i64 = conn
                .query_row(
                    "SELECT CAST(stratum AS INTEGER) FROM sky_nodes WHERE path = ?1",
                    params![path],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            let last_reviewed = pulse.last_reviewed.get(path).map(|s| s.as_str());
            let interval = pulse.intervals.get(path).copied().unwrap_or(0);
            let snoozed = pulse.snoozed.get(path).map(|s| s.as_str());
            let dismissed = pulse.dismissed.contains(path);
            crate::review::backfill_schedule_row(
                conn, path, tags_json, *modified, stratum, last_reviewed, interval, snoozed, dismissed, today,
            )?;
            // Baseline the content hash (only if not already set — resume-safe; content_changed_at
            // stays NULL so nothing is "stale" until a real post-stamp body change bumps it).
            conn.execute(
                "UPDATE note_meta SET content_hash = ?2 WHERE path = ?1 AND content_hash IS NULL",
                params![path, crate::review::content_hash(body_text)],
            )
            .map_err(|e| format!("baseline content_hash {}: {}", path, e))?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => conn.execute_batch("COMMIT").map_err(|e| e.to_string())?,
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            return Err(e);
        }
    }
    Ok((rows.len(), new_last))
}

fn ensure_cursor_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS review_backfill_cursor (
            id        INTEGER PRIMARY KEY CHECK (id = 1),
            last_path TEXT
        );",
    )
    .map_err(|e| format!("review cursor create: {}", e))
}

fn read_cursor(db: &Mutex<Option<Connection>>) -> Result<String, String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;
    Ok(conn
        .query_row(
            "SELECT last_path FROM review_backfill_cursor WHERE id = 1",
            [],
            |r| r.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
        .unwrap_or_default())
}

fn write_cursor(db: &Mutex<Option<Connection>>, last_path: &str) -> Result<(), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;
    conn.execute(
        "INSERT OR REPLACE INTO review_backfill_cursor (id, last_path) VALUES (1, ?1)",
        params![last_path],
    )
    .map_err(|e| format!("review cursor write: {}", e))?;
    Ok(())
}

/// Drained → stamp `schema_versions.review` (flips the machinery live) + wipe the
/// cursor. Both in one txn so a crash can't half-stamp.
fn finalize(db: &Mutex<Option<Connection>>) -> Result<(), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    let r = (|| -> Result<(), String> {
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('review', 1, strftime('%s','now'))",
            [],
        )
        .map_err(|e| format!("stamp review: {}", e))?;
        conn.execute("DELETE FROM review_backfill_cursor WHERE id = 1", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if r.is_ok() {
        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
    } else {
        let _ = conn.execute_batch("ROLLBACK");
    }
    r
}
