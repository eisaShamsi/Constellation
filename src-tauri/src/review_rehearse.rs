//! MIG-083 §D — the rehearsal harness. Proves the Rule-8 indexed Review-Pulse read
//! against a **COPY** of the live 7,611-note universe DB. It never touches the
//! original (secure-don't-muddle): it copies the file, migrates + back-fills the
//! copy, then rehearses.
//!
//! Run it:
//! ```text
//!   REVIEW_REHEARSE_DB="E:/Constellation Universes/Eisa Cognitive Knowledge/.constellation/search.db" \
//!     cargo test --lib review_rehearse -- --nocapture
//! ```
//! With the env var UNSET the test is a no-op, so a normal `cargo test` / CI run
//! never copies ~2 GB.
//!
//! It asserts three things:
//!   1. **Parity** — the indexed read (`review::query_due_notes_indexed`, reading
//!      `review_schedule` + the Mode-2 JOIN) equals an independent **reference**
//!      recomputed in Rust loops from the action state (`review-pulse.json`) +
//!      `note_meta`. Same set of `(path, reason)`.
//!   2. **Budget** — `query_due_notes_indexed` returns in **< 100 ms** on the full
//!      corpus.
//!   3. **Mode-2 fires** — a seeded real-graph fixture (a note with a load-bearing
//!      OUT-link whose dependency we mark as "changed today") surfaces as `stale`.
//!
//! Zero-`.md`-syscall is structural: `query_due_notes_indexed` touches only SQLite
//! (no `read_dir`/`metadata`/`read_to_string`). The latency corroborates it — a
//! 7,611-note filesystem walk could not complete in <100 ms.

#![cfg(test)]

use crate::review::STALENESS_TRIGGER_TYPES;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct RehearseReport {
    note_count: i64,
    schedule_rows: i64,
    indexed_total: usize,
    lens1_count: usize,
    lens2_count: usize,
    pre_seed_stale: usize, // stale count BEFORE seeding — must be 0 (touch-test, finding A)
    query_ms_max: f64,
    parity_ok: bool,
    parity_only_indexed: Vec<(String, String)>,
    parity_only_reference: Vec<(String, String)>,
    mode2_fixture: String, // "ok" | "n/a (no load-bearing link)" | "FAILED"
}

/// Days-since-2020 → unix secs at UTC midnight (mirrors `strftime('%s', date)`).
fn secs_of_day(days: i64) -> i64 {
    days * 86_400 + 1_577_836_800
}

/// Copy `src` (and its `-wal` sidecar, if present) to a temp file. The `-shm` is
/// intentionally NOT copied — SQLite regenerates it on open.
fn copy_db(src: &Path) -> Result<PathBuf, String> {
    let stamp = src
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dst = std::env::temp_dir().join(format!("review_rehearse_{}.db", stamp));
    std::fs::copy(src, &dst).map_err(|e| format!("copy {}: {}", src.display(), e))?;
    let wal_src = src.with_extension("db-wal");
    if wal_src.exists() {
        let _ = std::fs::copy(&wal_src, dst.with_extension("db-wal"));
    }
    Ok(dst)
}

/// Back-fill `review_schedule` on the copy from `note_meta` + the in-memory pulse,
/// then stamp `review` — exactly mirroring `review_backfill::run`, inline (no app
/// handle). Returns the row count.
fn backfill(conn: &Connection, pulse: &crate::review::ReviewPulseData, today: &str) -> Result<i64, String> {
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
        let stratum: i64 = conn
            .query_row("SELECT CAST(stratum AS INTEGER) FROM sky_nodes WHERE path = ?1", params![path], |r| r.get(0))
            .unwrap_or(0);
        let lr = pulse.last_reviewed.get(path).map(|s| s.as_str());
        let interval = pulse.intervals.get(path).copied().unwrap_or(0);
        let snoozed = pulse.snoozed.get(path).map(|s| s.as_str());
        let dismissed = pulse.dismissed.contains(path);
        crate::review::backfill_schedule_row(conn, path, tags_json, *modified, stratum, lr, interval, snoozed, dismissed, today)?;
        // Baseline content_hash exactly as review_backfill::process_batch does.
        conn.execute(
            "UPDATE note_meta SET content_hash = ?2 WHERE path = ?1 AND content_hash IS NULL",
            params![path, crate::review::content_hash(body_text)],
        )
        .map_err(|e| e.to_string())?;
    }
    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('review', 1, strftime('%s','now'))",
        [],
    )
    .map_err(|e| format!("stamp review: {}", e))?;
    Ok(rows.len() as i64)
}

/// The independent reference: recompute the universe-wide due-set in Rust loops from
/// the action state + `note_meta`, NOT from `review_schedule`. Lens-1 is recomputed
/// INDEPENDENTLY of `schedule_for`/`compute_schedule_row` (review finding L) so a bug
/// in the production scheduling logic makes parity FAIL rather than passing on both
/// sides. `today_days` must be the LOCAL today (matches `local_day`).
fn reference_due_set(conn: &Connection, pulse: &crate::review::ReviewPulseData, today: &str, today_days: i64) -> Result<HashSet<(String, String)>, String> {
    let mut set: HashSet<(String, String)> = HashSet::new();
    let today_day = crate::review::parse_day(today).unwrap_or(today_days);

    // Lens 1: Mode 1/3 — INDEPENDENT recompute of the (reason, due_days) spec.
    {
        let mut stmt = conn
            .prepare("SELECT path, COALESCE(tags_json,'[]'), COALESCE(modified,0) FROM note_meta")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let (path, tags_json, modified) = row;
            if pulse.dismissed.contains(&path) {
                continue;
            }
            // Active snooze hides the note from Lens-1 too.
            if let Some(su) = pulse.snoozed.get(&path) {
                if su.as_str() > today {
                    continue;
                }
            }
            let is_cp = crate::review::is_checkpoint(&tags_json);
            let interval = pulse.intervals.get(&path).copied().unwrap_or(0).max(1);
            // (reason, due_days) recomputed inline — NOT via schedule_for/compute_schedule_row.
            let (reason, due_days) = match pulse.last_reviewed.get(&path) {
                Some(lr) => {
                    let d = crate::review::parse_day(lr).unwrap_or(0);
                    if is_cp { ("checkpoint", d + 30) } else { ("interval_due", d + interval as i64) }
                }
                None => ("never_reviewed", crate::review::secs_to_days(modified) + 1),
            };
            if due_days <= today_day {
                set.insert((path, reason.to_string()));
            }
        }
    }

    // Lens 2: Mode 2 — staleness, by iterating load-bearing out-links in Rust. Mirrors
    // the corrected impl: content_changed_at IS NOT NULL (no mtime fallback), local_day
    // comparison, self-link excluded, malformed last_reviewed skipped. Snooze does NOT
    // suppress the Stale lens (Boss 2026-06-22) — the lenses are fully separate.
    {
        let types_in = STALENESS_TRIGGER_TYPES.iter().map(|t| format!("'{}'", t)).collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT jl.source_path, jl.target_cid_cn FROM note_links jl
             WHERE jl.status='active' AND jl.link_type IN ({types})
               AND jl.target_cid_cn IS NOT NULL AND jl.target_cid_cn != ''",
            types = types_in
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let (src, target_cid) = row;
            let lr = match pulse.last_reviewed.get(&src) {
                Some(d) => d,
                None => continue,
            };
            if pulse.dismissed.contains(&src) {
                continue;
            }
            // (snooze does NOT suppress staleness — lenses are separate.)
            let lr_day = match crate::review::parse_day(lr) {
                Some(d) => d,
                None => continue, // malformed → skip
            };
            // resolve dep: its path (self-link exclusion) + content-change instant
            let dep: Option<(String, Option<i64>)> = conn
                .query_row(
                    "SELECT path, content_changed_at FROM note_meta WHERE cid_cn = ?1",
                    params![target_cid],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .ok();
            if let Some((dep_path, Some(cca))) = dep {
                if dep_path == src {
                    continue; // self-link
                }
                if crate::review::local_day(cca) > lr_day {
                    set.insert((src, "stale".to_string()));
                }
            }
        }
    }
    Ok(set)
}

/// Seed a deterministic Mode-2 fixture on the copy: find a note with a load-bearing
/// OUT-link to an existing dependency, mark the dependency "changed today", and
/// review the source long ago — so it MUST surface as stale. Mutates the in-memory
/// pulse (reflected into review_schedule by the subsequent back-fill) + note_meta.
/// Returns the source path it seeded, or None if the graph has no such link.
fn seed_mode2_fixture(conn: &Connection, pulse: &mut crate::review::ReviewPulseData, today_days: i64) -> Result<Option<String>, String> {
    let types_in = STALENESS_TRIGGER_TYPES.iter().map(|t| format!("'{}'", t)).collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT jl.source_path, dep.path FROM note_links jl
         JOIN note_meta dep ON dep.cid_cn = jl.target_cid_cn
         WHERE jl.status='active' AND jl.link_type IN ({types})
           AND jl.target_cid_cn IS NOT NULL AND jl.target_cid_cn != ''
           AND jl.source_path != dep.path
         LIMIT 1",
        types = types_in
    );
    let found: Option<(String, String)> = conn
        .query_row(&sql, [], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .ok();
    if let Some((src, dep_path)) = found {
        // dependency "changed today"
        conn.execute(
            "UPDATE note_meta SET content_changed_at = ?2 WHERE path = ?1",
            params![dep_path, secs_of_day(today_days)],
        )
        .map_err(|e| e.to_string())?;
        // source reviewed long ago (in the action source → reflected by back-fill)
        pulse.last_reviewed.insert(src.clone(), "2020-01-02".to_string());
        pulse.dismissed.retain(|p| p != &src);
        pulse.snoozed.remove(&src);
        return Ok(Some(src));
    }
    Ok(None)
}

fn run(live_db: &Path) -> Result<RehearseReport, String> {
    let copy = copy_db(live_db)?;
    // The universe's .constellation/ is the parent of search.db → review-pulse.json lives there.
    let cdir = live_db.parent().ok_or("no parent for live_db")?.to_path_buf();
    let mut pulse = crate::review::load_pulse_data(&cdir);
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today_days = crate::review::date_to_days(&today);

    // Migrate the copy (adds content_changed_at via ensure_note_meta_review_columns;
    // creates review_schedule) exactly as boot would.
    let conn = crate::search::init_db(&copy)?;

    let note_count: i64 = conn.query_row("SELECT COUNT(*) FROM note_meta", [], |r| r.get(0)).unwrap_or(0);

    // FIRST back-fill (unseeded): baselines content_hash corpus-wide, leaves
    // content_changed_at NULL everywhere. TOUCH-TEST (review finding A): with no
    // recorded content change anywhere, the Stale lens MUST be empty — nothing fires
    // off a file mtime / touch. We assert this before seeding.
    let schedule_rows = backfill(&conn, &pulse, &today)?;
    let pre = crate::review::query_due_notes_indexed(&conn, "", &today, today_days)?;
    let pre_seed_stale = pre.iter().filter(|d| d.reason == "stale").count();

    // Now seed ONE real-graph fixture (one dep "changed today", one source reviewed
    // long ago) + re-back-fill so the seeded review state lands in review_schedule.
    let seeded = seed_mode2_fixture(&conn, &mut pulse, today_days)?;
    backfill(&conn, &pulse, &today)?;

    // ── Budget: time the indexed read (universe-wide: empty prefix matches all). ──
    let mut query_ms_max = 0.0_f64;
    let mut last: Vec<crate::review::DueNote> = Vec::new();
    for i in 0..6 {
        let t0 = std::time::Instant::now();
        last = crate::review::query_due_notes_indexed(&conn, "", &today, today_days)?;
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        if i > 0 {
            // ignore the warmup run (cold page cache)
            query_ms_max = query_ms_max.max(ms);
        }
    }
    let lens2_count = last.iter().filter(|d| d.reason == "stale").count();
    let lens1_count = last.len() - lens2_count;

    // ── Parity: indexed set vs the independent reference. ──
    let indexed_set: HashSet<(String, String)> = last.iter().map(|d| (d.note_path.clone(), d.reason.clone())).collect();
    let reference_set = reference_due_set(&conn, &pulse, &today, today_days)?;
    let only_indexed: Vec<(String, String)> = indexed_set.difference(&reference_set).cloned().take(20).collect();
    let only_reference: Vec<(String, String)> = reference_set.difference(&indexed_set).cloned().take(20).collect();
    let parity_ok = only_indexed.is_empty() && only_reference.is_empty();

    // ── Mode-2 fixture: the seeded source surfaces as stale. ──
    let mode2_fixture = match seeded {
        None => "n/a (no load-bearing link in corpus)".to_string(),
        Some(src) => {
            if indexed_set.contains(&(src.clone(), "stale".to_string())) {
                "ok".to_string()
            } else {
                format!("FAILED — {} did not surface as stale", src)
            }
        }
    };

    // best-effort cleanup of the copy. Drop the connection FIRST — Windows refuses
    // to delete a file with an open handle, which would silently leak the ~2 GB copy.
    drop(conn);
    let _ = std::fs::remove_file(&copy);
    let _ = std::fs::remove_file(copy.with_extension("db-wal"));
    let _ = std::fs::remove_file(copy.with_extension("db-shm"));

    Ok(RehearseReport {
        note_count,
        schedule_rows,
        indexed_total: last.len(),
        lens1_count,
        lens2_count,
        pre_seed_stale,
        query_ms_max,
        parity_ok,
        parity_only_indexed: only_indexed,
        parity_only_reference: only_reference,
        mode2_fixture,
    })
}

/// MIG-083 §D — build + verify the "Review Demo" scratch universe so the Boss can
/// SEE Mode-2 staleness live without touching Cognitive Knowledge. Indexes the two
/// notes (Claim --derives-from--> Evidence), seeds Claim reviewed 10 days ago +
/// Evidence changed today, and asserts Claim surfaces as `stale`. Leaves the DB so
/// the app preserves it on open (upsert keeps last_reviewed; the baselined content
/// hash keeps content_changed_at). Run:
///   REVIEW_DEMO_DIR="E:/Constellation Universes/Review Demo" cargo test --release build_review_demo -- --nocapture
#[test]
fn build_review_demo() {
    let dir = match std::env::var("REVIEW_DEMO_DIR") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("build_review_demo: SKIPPED (set REVIEW_DEMO_DIR)");
            return;
        }
    };
    // Normalize to the OS-native (backslash) form so the pre-built note_meta.path
    // matches what the app will store when it walks the universe on open (else the
    // app's re-index creates fresh rows and our seeded ones orphan).
    let dir = dir.replace('/', "\\");
    let root = Path::new(&dir);
    let db = root.join(".constellation").join("search.db");
    let _ = std::fs::remove_file(&db);
    let _ = std::fs::remove_file(db.with_extension("db-wal"));
    let _ = std::fs::remove_file(db.with_extension("db-shm"));
    let conn = crate::search::init_db(&db).expect("init_db");

    // Index the TARGET first so the source's typed link resolves its target_cid_cn
    // (a forward link to a not-yet-indexed note would resolve to NULL → not stale).
    crate::search::index_note(&conn, &root.join("Evidence.md").to_string_lossy(), "Review Demo", true).expect("index Evidence");
    crate::search::index_note(&conn, &root.join("Claim.md").to_string_lossy(), "Review Demo", true).expect("index Claim");

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today_days = crate::review::date_to_days(&today);
    backfill(&conn, &crate::review::ReviewPulseData::default(), &today).expect("backfill");

    let claim_path: String = conn.query_row("SELECT path FROM note_meta WHERE name='Claim'", [], |r| r.get(0)).expect("Claim row");
    let ev_path: String = conn.query_row("SELECT path FROM note_meta WHERE name='Evidence'", [], |r| r.get(0)).expect("Evidence row");

    // Claim reviewed 10 days ago, interval 30 → due_days far future (NOT interval-due,
    // so the ONLY reason it surfaces is staleness).
    let lr = "2026-06-12";
    conn.execute(
        "UPDATE review_schedule SET last_reviewed=?2, interval=30, reason='interval_due', due_days=?3, snoozed_until=NULL WHERE path=?1",
        rusqlite::params![claim_path, lr, crate::review::date_to_days(lr) + 30],
    ).unwrap();
    // Evidence "changed today" (content_hash already baselined → the app's re-index on
    // open will see an identical hash and NOT bump, preserving this).
    conn.execute("UPDATE note_meta SET content_changed_at=?2 WHERE path=?1", rusqlite::params![ev_path, secs_of_day(today_days)]).unwrap();

    // Rewrite review-pulse.json with the VERIFIED path (so a later ✓ stays consistent).
    let _ = std::fs::write(
        root.join(".constellation").join("review-pulse.json"),
        format!("{{\n  \"last_reviewed\": {{ {:?}: \"{}\" }},\n  \"snoozed\": {{}},\n  \"intervals\": {{ {:?}: 30 }},\n  \"dismissed\": []\n}}", claim_path, lr, claim_path),
    );

    let due = crate::review::query_due_notes_indexed(&conn, "", &today, today_days).unwrap();
    eprintln!("── Review Demo built ({} due) ──", due.len());
    for d in &due {
        eprintln!("  {} [{}] trigger={:?} changed_on={:?}", d.note_name, d.reason, d.stale_trigger_name, d.stale_changed_on);
    }
    let claim_stale = due.iter().any(|d| d.note_name == "Claim" && d.reason == "stale");
    drop(conn);
    assert!(claim_stale, "Claim must surface as stale — demo not ready");
    eprintln!("✓ Demo ready: open the 'Review Demo' universe → Review panel → Claim shows 🥀 stale.");
}

#[test]
fn review_rehearse_live() {
    let db = match std::env::var("REVIEW_REHEARSE_DB") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("review_rehearse_live: SKIPPED (set REVIEW_REHEARSE_DB to a live search.db to run)");
            return;
        }
    };
    let report = run(Path::new(&db)).expect("rehearsal run failed");
    eprintln!("──────── MIG-083 §D rehearsal report ────────");
    eprintln!("notes in corpus      : {}", report.note_count);
    eprintln!("review_schedule rows : {}", report.schedule_rows);
    eprintln!("due (indexed total)  : {}  (lens-1 {} + lens-2/stale {})", report.indexed_total, report.lens1_count, report.lens2_count);
    eprintln!("touch-test (pre-seed): {} stale  (must be 0 — no staleness off a touch/mtime)", report.pre_seed_stale);
    eprintln!("query_due_notes max  : {:.2} ms  (target < 100 ms)", report.query_ms_max);
    eprintln!("parity indexed==ref  : {}", report.parity_ok);
    if !report.parity_ok {
        eprintln!("  only in indexed (≤20): {:?}", report.parity_only_indexed);
        eprintln!("  only in reference(≤20): {:?}", report.parity_only_reference);
    }
    eprintln!("mode-2 fixture       : {}", report.mode2_fixture);
    eprintln!("─────────────────────────────────────────────");

    assert_eq!(report.pre_seed_stale, 0, "TOUCH-TEST FAILED: {} notes were stale with NO recorded content change (firing off mtime — finding A)", report.pre_seed_stale);
    assert!(report.parity_ok, "indexed read diverged from the corrected reference");
    assert!(report.query_ms_max < 100.0, "get_due_notes exceeded the 100 ms budget: {:.2} ms", report.query_ms_max);
    assert!(!report.mode2_fixture.starts_with("FAILED"), "Mode-2 staleness did not fire: {}", report.mode2_fixture);
}
