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
//! action state — re-read per batch, refusing rather than degrading (2026-08-08).

use crate::search::SearchState;
use rusqlite::{params, Connection};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};

const BATCH_SIZE: usize = 1000;
const INTER_BATCH_SLEEP_MS: u64 = 50;

/// One-shot guard. Unlike the other back-fills (scheduled once at boot), this one is
/// ALSO kicked lazily from `get_due_notes` on every unstamped read (§E) — without
/// this, repeated reads during the build window would spawn concurrent back-fill
/// threads racing on the same WAL. compare_exchange ensures exactly one runs.
static RUNNING: AtomicBool = AtomicBool::new(false);

/// Schedule the back-fill on a background thread. Returns immediately. No-op if
/// `review` is already stamped (the common steady-state case) or one is already running.
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
    // Claim the single run-slot; if another back-fill already holds it, do nothing.
    if RUNNING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }
    thread::spawn(move || {
        if let Err(e) = run(&app) {
            eprintln!("review_backfill: {}", e);
        }
        RUNNING.store(false, Ordering::SeqCst);
    });
}

fn run(app: &tauri::AppHandle) -> Result<u64, String> {
    let state = app.state::<SearchState>();
    {
        let mut guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_mut().ok_or("DB not initialized")?;
        ensure_cursor_table(conn)?;
    }

    // The per-universe action state (the source of truth for the rows this back-fill
    // writes). Re-read per BATCH, not once — 2026-08-08 §11 inspection, the same two
    // findings documented on `review::recompute_all_in`, and this surface is the more
    // dangerous of the two because it STAMPS on completion: a pulse that degraded to
    // defaults here would be baked in as "built" and never revisited until a repair.
    let cdir = crate::universe::active_constellation_dir(app)?;
    let today = crate::review::today_str();

    let mut last_path = read_cursor(&state.db)?;
    let mut total: u64 = 0;
    loop {
        let (n, new_last) = process_batch(&state.db, &cdir, &last_path, &today)?;
        if n == 0 {
            finalize(&state.db, &cdir, &today)?;
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
    cdir: &std::path::Path,
    last_path: &str,
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
        // Fresh inside the lock, and refusing on a read error — see the note in `run`.
        let pulse = crate::review::load_pulse_data_for_update(cdir)?;
        for (path, tags_json, modified, body_text) in &rows {
            crate::review::backfill_one(conn, path, tags_json, *modified, body_text, &pulse, today)?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => crate::converge::commit_or_rollback(conn)?,
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
/// PJ-207 §11 inspection (MED, index-divergence) — the back-fill's OWN window race,
/// which the per-batch pulse re-read narrows but cannot close.
///
/// While `review` is unstamped, `sync_action_to_row` is a deliberate no-op, so a
/// ✓ Reviewed / Snooze / Dismiss taken during the build writes `review-pulse.json` and
/// NOT the row — and the note's own Review tab is not stamp-gated, so the buttons are
/// live throughout. Re-reading the pulse each batch rescues a note only if the cursor
/// has not yet passed it; a note in an already-committed batch keeps `last_reviewed =
/// NULL`. `finalize` then stamps, and from that moment every row is re-derived from the
/// ROW rather than the pulse — so the note reads never-reviewed forever, resurfacing in
/// the Reviewer while the on-disk source of truth says it was reviewed, and nothing
/// self-heals short of a manual Repair. That is PJ-187's symptom, arrived at from a new
/// direction.
///
/// So the last thing before the stamp is a re-apply of the pulse's OWN action set —
/// bounded by how many decisions the user made, not by corpus size.
fn reapply_pulse_actions(
    conn: &Connection,
    cdir: &std::path::Path,
    today: &str,
) -> Result<usize, String> {
    let pulse = crate::review::load_pulse_data_for_update(cdir)?;
    let acted = pulse.acted_paths();
    if acted.is_empty() {
        return Ok(0);
    }
    let mut applied = 0usize;
    for path in acted {
        // Only notes that actually exist in the index — an action recorded against a
        // since-deleted note has no row to rebuild, and must not create one.
        let row: Option<(String, i64, String)> = conn
            .query_row(
                "SELECT COALESCE(tags_json,'[]'), COALESCE(modified,0), COALESCE(body_text,'') FROM note_meta WHERE path = ?1",
                params![&path],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok();
        let Some((tags_json, modified, body_text)) = row else { continue };
        crate::review::backfill_one(conn, &path, &tags_json, modified, &body_text, &pulse, today)?;
        applied += 1;
    }
    Ok(applied)
}

fn finalize(db: &Mutex<Option<Connection>>, cdir: &std::path::Path, today: &str) -> Result<(), String> {
    let guard = db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("DB not initialized")?;
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    let r = (|| -> Result<(), String> {
        // BEFORE the stamp, inside the same transaction: either both land or neither
        // does. Stamping a divergence as authoritative is the failure this prevents.
        reapply_pulse_actions(conn, cdir, today)?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('review', ?1, strftime('%s','now'))",
            params![crate::review::REVIEW_SCHEMA_VERSION],
        )
        .map_err(|e| format!("stamp review: {}", e))?;
        conn.execute("DELETE FROM review_backfill_cursor WHERE id = 1", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if r.is_ok() {
        crate::converge::commit_or_rollback(conn)?;
    } else {
        let _ = conn.execute_batch("ROLLBACK");
    }
    r
}

#[cfg(test)]
mod tests {
    //! PJ-207 §11 inspection (MED, index-divergence) — the pre-stamp re-apply.
    use super::reapply_pulse_actions;
    use rusqlite::Connection;

    fn db() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, tags_json TEXT DEFAULT '[]',
               modified INTEGER, body_text TEXT DEFAULT '', content_hash TEXT);
             CREATE TABLE sky_nodes (path TEXT PRIMARY KEY, stratum TEXT);
             CREATE TABLE review_schedule (path TEXT PRIMARY KEY, reason TEXT NOT NULL,
               due_days INTEGER NOT NULL, is_checkpoint INTEGER NOT NULL DEFAULT 0,
               last_reviewed TEXT, stratum INTEGER NOT NULL DEFAULT 0,
               interval INTEGER NOT NULL DEFAULT 0, snoozed_until TEXT);",
        )
        .unwrap();
        c
    }

    fn pulse_dir(tag: &str, json: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("constellation-backfill-{}", tag));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("review-pulse.json"), json).unwrap();
        d
    }

    /// The defect: while the back-fill is running, `review` is unstamped, so a
    /// ✓ Reviewed writes only `review-pulse.json`. If the cursor had already passed
    /// that note, its row kept `last_reviewed = NULL` — and the stamp then made the row
    /// authoritative forever. The re-apply runs immediately before the stamp so the
    /// user's decision is in the row when it becomes the source of truth.
    #[test]
    fn an_action_taken_during_the_backfill_is_applied_before_the_stamp() {
        let c = db();
        c.execute(
            "INSERT INTO note_meta (path, tags_json, modified, body_text) VALUES ('/lib/Passed.md','[]',0,'')",
            [],
        )
        .unwrap();
        // The row as the already-committed batch left it: never reviewed.
        c.execute(
            "INSERT INTO review_schedule (path, reason, due_days) VALUES ('/lib/Passed.md','never_reviewed',0)",
            [],
        )
        .unwrap();

        let d = pulse_dir(
            "acted-mid-run",
            r#"{"last_reviewed":{"/lib/Passed.md":"2026-06-21"},"intervals":{"/lib/Passed.md":7},"snoozed":{},"dismissed":[]}"#,
        );
        let applied = reapply_pulse_actions(&c, &d, "2026-06-22").unwrap();
        assert_eq!(applied, 1, "the one acted-on note is re-applied");

        let (reason, last): (String, Option<String>) = c
            .query_row(
                "SELECT reason, last_reviewed FROM review_schedule WHERE path='/lib/Passed.md'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(last.as_deref(), Some("2026-06-21"), "the review survives the stamp");
        assert_eq!(reason, "interval_due", "and the row is re-derived from it");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// An action recorded against a note that no longer exists must not conjure a row —
    /// that would be an orphan the sweep has to clean up later.
    #[test]
    fn an_action_on_a_note_that_no_longer_exists_creates_no_row() {
        let c = db();
        let d = pulse_dir(
            "acted-on-ghost",
            r#"{"last_reviewed":{"/lib/Ghost.md":"2026-06-21"},"intervals":{},"snoozed":{},"dismissed":[]}"#,
        );
        let applied = reapply_pulse_actions(&c, &d, "2026-06-22").unwrap();
        assert_eq!(applied, 0, "no note_meta row → nothing to re-apply");
        let n: i64 = c
            .query_row("SELECT COUNT(*) FROM review_schedule", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "and no phantom row was created");
        let _ = std::fs::remove_dir_all(&d);
    }
}
