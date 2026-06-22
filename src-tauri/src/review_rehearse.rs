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
    let rows: Vec<(String, String, i64)> = {
        let mut stmt = conn
            .prepare("SELECT path, COALESCE(tags_json,'[]'), COALESCE(modified,0) FROM note_meta")
            .map_err(|e| e.to_string())?;
        let it = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))
            .map_err(|e| e.to_string())?;
        it.filter_map(|x| x.ok()).collect()
    };
    for (path, tags_json, modified) in &rows {
        let stratum: i64 = conn
            .query_row("SELECT CAST(stratum AS INTEGER) FROM sky_nodes WHERE path = ?1", params![path], |r| r.get(0))
            .unwrap_or(0);
        let lr = pulse.last_reviewed.get(path).map(|s| s.as_str());
        let interval = pulse.intervals.get(path).copied().unwrap_or(0);
        let snoozed = pulse.snoozed.get(path).map(|s| s.as_str());
        let dismissed = pulse.dismissed.contains(path);
        crate::review::backfill_schedule_row(conn, path, tags_json, *modified, stratum, lr, interval, snoozed, dismissed, today)?;
    }
    conn.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at) VALUES ('review', 1, strftime('%s','now'))",
        [],
    )
    .map_err(|e| format!("stamp review: {}", e))?;
    Ok(rows.len() as i64)
}

/// The independent reference: recompute the universe-wide due-set in Rust loops from
/// the action state + `note_meta`, NOT from `review_schedule`.
fn reference_due_set(conn: &Connection, pulse: &crate::review::ReviewPulseData, today: &str, today_days: i64) -> Result<HashSet<(String, String)>, String> {
    let mut set: HashSet<(String, String)> = HashSet::new();

    // Lens 1: Mode 1/3 — one (reason, due_days) per note from pulse + note_meta.
    {
        let mut stmt = conn
            .prepare("SELECT path, COALESCE(tags_json,'[]'), COALESCE(modified,0) FROM note_meta")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let (path, tags_json, modified) = row;
            let is_cp = crate::review::is_checkpoint(&tags_json);
            let lr = pulse.last_reviewed.get(&path).map(|s| s.as_str());
            let interval = pulse.intervals.get(&path).copied().unwrap_or(0);
            let snoozed = pulse.snoozed.get(&path).map(|s| s.as_str());
            let dismissed = pulse.dismissed.contains(&path);
            let (reason, due_days) = crate::review::schedule_for(is_cp, modified, lr, interval, snoozed, dismissed, today);
            if reason != "dismissed" && due_days <= today_days {
                set.insert((path, reason));
            }
        }
    }

    // Lens 2: Mode 2 — staleness, by iterating load-bearing out-links in Rust.
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
            // source must be reviewed + not dismissed
            let lr = match pulse.last_reviewed.get(&src) {
                Some(d) => d,
                None => continue,
            };
            if pulse.dismissed.contains(&src) {
                continue;
            }
            let lr_day = crate::review::date_to_days(lr);
            // resolve dep + its content-change day
            let dep: Option<(Option<i64>, i64)> = conn
                .query_row(
                    "SELECT content_changed_at, COALESCE(modified,0) FROM note_meta WHERE cid_cn = ?1",
                    params![target_cid],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .ok();
            if let Some((cca, modified)) = dep {
                let dep_day = crate::review::secs_to_days(cca.unwrap_or(modified));
                if dep_day > lr_day {
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

    // Seed the Mode-2 fixture BEFORE the back-fill so the seeded review state lands
    // in review_schedule, and BOTH the indexed read + the reference see it.
    let seeded = seed_mode2_fixture(&conn, &mut pulse, today_days)?;

    let schedule_rows = backfill(&conn, &pulse, &today)?;

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

    // best-effort cleanup of the copy
    let _ = std::fs::remove_file(&copy);
    let _ = std::fs::remove_file(copy.with_extension("db-wal"));

    Ok(RehearseReport {
        note_count,
        schedule_rows,
        indexed_total: last.len(),
        lens1_count,
        lens2_count,
        query_ms_max,
        parity_ok,
        parity_only_indexed: only_indexed,
        parity_only_reference: only_reference,
        mode2_fixture,
    })
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
    eprintln!("query_due_notes max  : {:.2} ms  (target < 100 ms)", report.query_ms_max);
    eprintln!("parity indexed==ref  : {}", report.parity_ok);
    if !report.parity_ok {
        eprintln!("  only in indexed (≤20): {:?}", report.parity_only_indexed);
        eprintln!("  only in reference(≤20): {:?}", report.parity_only_reference);
    }
    eprintln!("mode-2 fixture       : {}", report.mode2_fixture);
    eprintln!("─────────────────────────────────────────────");

    assert!(report.parity_ok, "indexed read diverged from the corrected reference");
    assert!(report.query_ms_max < 100.0, "get_due_notes exceeded the 100 ms budget: {:.2} ms", report.query_ms_max);
    assert!(!report.mode2_fixture.starts_with("FAILED"), "Mode-2 staleness did not fire: {}", report.mode2_fixture);
}
