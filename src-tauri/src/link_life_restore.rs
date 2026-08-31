//! MIG-104 Slice 6 — the restore. **This is the whole point of the migration.**
//!
//! Everything *about* a note comes back from the note itself: its text, its links, its properties.
//! But the record of the user's READING — which links they walked, how often, what they came to
//! trust, what they retired — has until now lived only in `search.db`. After this pass it comes
//! back from `earned.jsonl` too, so losing or rebuilding the index costs nothing the user created.
//!
//! **`weight` is DERIVED, never restored.** `earned_link_weight(n) = 1 + ln(1 + n)`. A stored
//! weight is not earned data — it is a cache of an arithmetic function of `n`, and 236 live rows
//! carry values that function cannot produce (decay residue). Recomputing heals all 236 for free
//! and stops `index_note`'s live `weight != 1.0` clause treating them as earned forever.
//!
//! **Batched, because every `note_links` UPDATE is expensive.** It fires `note_links_sky_au`
//! (a DELETE + INSERT on the 234k-row `sky_links`) plus the outgoing-aggregate trigger pair's two
//! `note_meta` UPDATEs. Restoring 34 records must not become 34 × that, unbatched, on the writer
//! lock at boot.
//!
//! ## The rule the Boss's own data forced (2026-07-27)
//!
//! A record whose target could not be identified keys on the target NAME — the seed deliberately
//! refuses to guess between several notes sharing a name. Two DIFFERENT links from one source to
//! two DIFFERENT same-named notes therefore fold to ONE record (live: `banana` ×2).
//!
//! > **A name-keyed record may be restored ONLY when it resolves to exactly ONE `note_links` row.
//! > If several rows match, it is SKIPPED and reported — never distributed across links that may
//! > have earned different amounts.**
//!
//! Both live `banana` links carry `n = 1`, so writing 1 to both would be correct *today* — which is
//! exactly why the rule must be structural rather than left to a judgement call. If their counts
//! ever differ, a max-fold restore would hand the lower link walks it never earned. An
//! identity-keyed record is unambiguous and restores normally.

use rusqlite::Connection;
use std::time::Duration;

/// How many `note_links` UPDATEs per transaction. Each one fires the sky trigger (DELETE + INSERT
/// on 234k rows) and the outgoing-aggregate pair, so the batch bounds how long the writer lock is
/// held in one go while still amortising the transaction cost.
const BATCH: usize = 50;

/// What one restore pass did — every number surfaced, none implied.
#[derive(Debug, Default, PartialEq)]
pub struct RestoreReport {
    /// Folded records read from the ledger.
    pub records: usize,
    /// Rows whose earned state was written back.
    pub restored: usize,
    /// Records already in agreement with the DB — nothing written (the steady state).
    pub already_current: usize,
    /// Records whose link no longer exists in the index at all.
    pub no_matching_row: usize,
    /// **Name-keyed records that matched SEVERAL rows** — skipped by rule, never distributed.
    pub ambiguous_skipped: usize,
    /// Rows whose `weight` was off the earned curve and has been recomputed.
    pub weights_healed: usize,
    /// Unparseable ledger lines (each cost one line, never the file).
    pub skipped_lines: usize,
    /// Writes PLANNED. `planned > 0` with `restored == 0` means every batch FAILED — an
    /// ambiguity the first cut could not express, so "0 written" read as "nothing needed doing".
    pub planned: usize,
    /// The index held no links at all — a rebuild in progress, not an absence of earned data.
    /// Nothing is written and nothing is concluded; the next boot retries.
    pub index_not_ready: bool,
    /// Note-level review priorities written back.
    pub priorities_restored: usize,
    /// Priorities the DB already agreed with.
    pub priorities_already_current: usize,
    /// Priority records whose note is no longer in the index — or whose identity matches more
    /// than one row, which is skipped by the same rule that governs ambiguous link records.
    pub priorities_unresolved: usize,
}

/// Runs on EVERY boot — deliberately not stamped.
///
/// ★ THE BUG THIS FIXES (Boss-found 2026-07-27, live). The first cut gated this on a
/// `schema_versions` stamp like the one-shot backfills. On a REBUILT index the pass raced the
/// initial indexing, found that none of the 34 records had a link to attach to yet, reported
/// "34 no longer in the index" — and **stamped itself complete**. It would never have run again,
/// and the earned data would never have come back. Exactly the scenario the whole migration
/// exists for, defeated by the wrong gate.
///
/// The error was conceptual, not incidental: a **reconciler must not carry a migration's stamp.**
/// This pass is idempotent (it writes only where the DB disagrees, and reports `already_current`
/// otherwise) and cheap (bounded by earned-link count — 34, not 234,233), so running it every boot
/// is the correct shape. It is the same discipline `reconcile::maybe_schedule` already follows for
/// index-vs-disk drift, and for the same reason: the condition it repairs can recur at any time.
///
/// ⚠ **A `schema_versions` row named `link_life_restore` exists on databases that ran the first
/// cut** (the Boss's live DB carries `version = 1`, verified 2026-07-27). Nothing reads it and
/// nothing writes it any more. **Do not add a gate here that consults it** — that row is the
/// artefact of the exact mistake described above, not a signal, and reintroducing the gate would
/// reintroduce the bug on every database that already has the row.
pub fn maybe_schedule(app: tauri::AppHandle) {
    let app_bg = app.clone();
    std::thread::spawn(move || {
        match run(&app_bg) {
            Ok(r) if r.index_not_ready => diag(
                &app_bg,
                "[link_life_restore] index still rebuilding (0 links) — nothing concluded, will retry next boot",
            ),
            Ok(r) if r.records == 0 && r.priorities_restored == 0 => {} // no ledger yet: silent, this runs every boot
            Ok(r) if r.restored == 0 && r.priorities_restored == 0 && r.already_current == r.records => {} // steady state
            Ok(r) => diag(
                &app_bg,
                &format!(
                    "[link_life_restore] earned layer restored: {} of {} records written ({} already current, \
                     {} no longer in the index, {} ambiguous-skipped), {} weights healed, \
                     {} priorities restored ({} already current, {} unresolved), {} bad lines",
                    r.restored, r.records, r.already_current, r.no_matching_row,
                    r.ambiguous_skipped, r.weights_healed,
                    r.priorities_restored, r.priorities_already_current, r.priorities_unresolved,
                    r.skipped_lines
                ),
            ),
            Err(e) => diag(&app_bg, &format!("[link_life_restore] FAILED (non-fatal): {}", e)),
        }

        // Slice 7 — compaction rides THIS thread, strictly after the restore, so the restore
        // never reads a store that is being rewritten underneath it.
        //
        // ⚠ That is a SEQUENCING argument and it covers restore-vs-compact ONLY. An earlier draft
        // of this comment claimed the single thread made the race "impossible"; the safety
        // inspection showed that was an overclaim — the live appenders (`constellation_link_traverse`,
        // `record_decision`, `set_review_priority`) all write from Tauri command threads and are
        // completely unaffected by which thread compaction runs on. Exclusion against THEM is
        // `link_life::FILE_LOCK`, and it lives in the module that owns the files. Do not read this
        // comment as covering it.
        //
        // Everything here is file I/O with no DB lock held (PJ-066).
        match compact(&app_bg) {
            Ok(crate::link_life::CompactOutcome::Compacted(r)) => diag(
                &app_bg,
                &format!(
                    "[link_life] ledger compacted: {} lines snapshotted, tail {} B → snapshot {} B, \
                     tail kept at {}",
                    r.lines, r.tail_bytes, r.snapshot_bytes, r.tail_renamed_to.display()
                ),
            ),
            // A refusal is NOT a quiet success — it means the store needs a human.
            Ok(crate::link_life::CompactOutcome::Refused(why)) => diag(
                &app_bg,
                &format!("[link_life] compaction REFUSED — {why}"),
            ),
            // The overwhelmingly common case. Silent, because an idle store must cost nothing,
            // including a log line every boot.
            Ok(crate::link_life::CompactOutcome::BelowThreshold { .. }) => {}
            Err(e) => diag(&app_bg, &format!("[link_life] compaction FAILED (non-fatal): {}", e)),
        }
    });
}

/// The compaction pass. Pure file I/O against the store beside `search.db` — no connection, no
/// lock, nothing that can block a boot.
fn compact(app: &tauri::AppHandle) -> Result<crate::link_life::CompactOutcome, String> {
    // Compaction WRITES, so the Plan's promise that "toggle off = today's behaviour byte-for-byte"
    // has to cover it — a store left over from a run with writes ON must not be rewritten by a
    // build with them off. The const makes this branch vanish rather than cost a check.
    if !crate::link_life::EARNED_LEDGER_WRITE {
        return Ok(crate::link_life::CompactOutcome::BelowThreshold { tail_bytes: 0 });
    }
    let path = crate::search::db_path(app)?;
    let dir = path.parent().ok_or("search.db has no parent dir")?;
    // Second-granular UTC, so a tail-aside name is legible; `unique_aside` handles a collision.
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    crate::link_life::maybe_compact(dir, &stamp)
}

fn run(app: &tauri::AppHandle) -> Result<RestoreReport, String> {
    let path = crate::search::db_path(app)?;
    let dir = path.parent().ok_or("search.db has no parent dir")?.to_path_buf();
    let mut conn =
        Connection::open(&path).map_err(|e| format!("open link_life_restore conn: {}", e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("pragma: {}", e))?;

    // ★ THE CAUSE OF THE 2026-07-27 SILENT FAILURE — register the custom FTS5 tokenizer.
    //
    // `UPDATE note_links` is not a leaf write. It fires `note_links_outgoing_au`, which UPDATEs
    // `note_meta`, which fires `note_meta_au`, which writes `notes_fts` — an FTS5 table declared
    // with `tokenize = 'constellation'`. That tokenizer is registered PER CONNECTION (inside
    // `init_db`), so a bare `Connection::open` cannot service the write and every row failed with
    // `no such tokenizer: constellation`. All 33 planned writes were lost, silently.
    //
    // This module was cloned from `link_boot_index`, whose own docs state the precondition that
    // made ITS bare connection sufficient: *"CREATE INDEX is pure DDL — it fires NO row triggers —
    // so no FTS tokenizer registration is needed."* I copied the connection setup and not the
    // sentence explaining when it is safe. **A cloned pattern's PRECONDITIONS are part of the
    // pattern.** Any dedicated connection that writes a table with row triggers must register the
    // tokenizer; verified trigger chain above.
    crate::search::register_fts5_tokenizer(&mut conn)?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;

    restore(&conn, &dir)
}

/// Resolve a folded key back to the `note_links` rows it refers to.
///
/// Identity-keyed (`cid>TARGET_CID`) → the source note's path + the target note's NAME, which is
/// what `note_links` stores. Name-keyed (`cid>~name`) → the source path + that name.
fn rows_for_key(conn: &Connection, key: &str) -> Vec<i64> {
    let Some((src_cid, rest)) = key.split_once('>') else { return Vec::new() };
    let Ok(src_path) = conn.query_row(
        "SELECT path FROM note_meta WHERE cid_cn = ?1 AND cid_cn != '' LIMIT 1",
        rusqlite::params![src_cid],
        |r| r.get::<_, String>(0),
    ) else {
        return Vec::new(); // the source note itself is gone — nothing to restore onto
    };

    let target_lower: String = if let Some(name) = rest.strip_prefix('~') {
        name.to_string()
    } else {
        // Identity-keyed: find the target note's current NAME, so a target renamed since the
        // record was written still resolves (the identity is the durable half, the name is not).
        match conn.query_row(
            "SELECT LOWER(name) FROM note_meta WHERE cid_cn = ?1 AND cid_cn != '' LIMIT 1",
            rusqlite::params![rest],
            |r| r.get::<_, String>(0),
        ) {
            Ok(n) => n,
            Err(_) => return Vec::new(),
        }
    };

    let mut out = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id FROM note_links WHERE source_path = ?1 AND LOWER(target_name) = ?2",
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![src_path, target_lower], |r| r.get::<_, i64>(0)) {
            out.extend(rows.flatten());
        }
    }
    out
}

/// The restore pass. Separated from `run` so it is testable with no AppHandle.
pub(crate) fn restore(conn: &Connection, dir: &std::path::Path) -> Result<RestoreReport, String> {
    let (state, load) = crate::link_life::read_state(dir);
    let folded = &state.links;
    let mut report = RestoreReport {
        records: folded.len(),
        skipped_lines: load.skipped_lines,
        ..Default::default()
    };
    // Never draw a conclusion from an index that has not been populated yet. On a REBUILT database
    // this pass can start while indexing is still running, and every record would look like a link
    // that no longer exists. Report the state honestly and leave the work for the next boot — which
    // is safe precisely because this pass is unstamped and re-runs.
    let links_total: i64 = conn
        .query_row("SELECT COUNT(*) FROM note_links", [], |r| r.get(0))
        .unwrap_or(0);
    if links_total == 0 {
        report.index_not_ready = true;
        return Ok(report);
    }
    if load.refuse_write {
        // The store was structurally unusable and has been renamed aside. Do NOT write a thing
        // from a store we could not read — that is how a restore destroys what it was protecting.
        return Ok(report);
    }

    // Plan the writes first, entirely read-only, so the write transactions are short.
    struct Write {
        id: i64,
        n: i64,
        conf: Option<String>,
        status: String,
        at: String,
        weight: f64,
    }
    let mut writes: Vec<Write> = Vec::new();

    for (key, e) in folded {
        let ids = rows_for_key(conn, key);
        if ids.is_empty() {
            report.no_matching_row += 1;
            continue;
        }
        // THE RULE (see the module docs): a name-keyed record that matches several rows is skipped,
        // never distributed. One folded count cannot be handed to links that may have earned
        // different amounts.
        let name_keyed = key.split_once('>').map(|(_, r)| r.starts_with('~')).unwrap_or(false);
        if name_keyed && ids.len() > 1 {
            report.ambiguous_skipped += 1;
            continue;
        }
        for id in ids {
            let cur: Option<(i64, Option<String>, String, f64)> = conn
                .query_row(
                    "SELECT traversal_count, confidence, status, weight FROM note_links WHERE id = ?1",
                    rusqlite::params![id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
                .ok();
            let Some((cur_n, cur_conf, cur_status, cur_w)) = cur else { continue };

            // Never ratchet DOWN: the DB may hold a walk newer than the ledger's last flush.
            let n = cur_n.max(e.n);
            // Confidence: a RECORDED tier is a user judgment and wins. When none was recorded,
            // the tier is DERIVABLE from the count — exactly like `weight` — so derive it rather
            // than preserve whatever the rebuilt row happens to carry. (Caught by
            // `db_loss_round_trip`: a link with 7 walks came back `hypothesis`, because
            // `evidence` at n>=3 is derivable and therefore deliberately never recorded.)
            // Never DOWNGRADE: if the row already carries a higher-ranked judgment than the
            // derived tier, keep it — mirroring traverse's CASE WHEN preservation.
            let conf = match e.conf.clone() {
                Some(c) => Some(c),
                None => {
                    let derived = crate::link_life::auto_tier(n);
                    match cur_conf.as_deref() {
                        Some(cur) if crate::link_life::conf_rank(cur)
                            > crate::link_life::conf_rank(derived) => cur_conf.clone(),
                        _ => Some(derived.to_string()),
                    }
                }
            };
            let status = e.status.clone().unwrap_or(cur_status.clone());
            // DERIVED, never restored — this is also the weight heal.
            let weight = crate::search::earned_link_weight(n);
            let at = e.at.clone().unwrap_or_default();

            let differs = n != cur_n
                || conf != cur_conf
                || status != cur_status
                || (weight - cur_w).abs() > 1e-9;
            if !differs {
                report.already_current += 1;
                continue;
            }
            if (weight - cur_w).abs() > 1e-9 && n == cur_n {
                report.weights_healed += 1;
            }
            writes.push(Write { id, n, conf, status, at, weight });
        }
    }

    // ── Note-level decisions ────────────────────────────────────────────────────────────────
    //
    // Review priority is a decision the user made about a NOTE, so it has no `(source, target)`
    // key and never entered the link plan above. Slice 4 has been appending and fsyncing these
    // records since it shipped; nothing has ever read them back. The Plan's Slice 6 says
    // "restores review priority too" and the code did not — found while building the compactor,
    // because a fold-and-rewrite would have moved the records out of the loaded store for good.
    //
    // The ledger is authoritative here, and that is a property of the WRITE order rather than a
    // preference: `set_review_priority` appends and fsyncs BEFORE it touches the DB, so the file
    // can never be behind. (Contrast the walk count, which is DB-first and therefore max-folded.)
    struct NoteWrite {
        path: String,
        p: Option<i64>,
    }
    let mut note_writes: Vec<NoteWrite> = Vec::new();
    for (cid, ne) in &state.notes {
        if cid.is_empty() {
            continue;
        }
        // Exactly ONE row, by the same rule that governs an ambiguous link record: a decision is
        // never distributed across notes it might not belong to.
        let mut paths: Vec<String> = Vec::new();
        if let Ok(mut stmt) =
            conn.prepare("SELECT path FROM note_meta WHERE cid_cn = ?1 AND cid_cn != '' LIMIT 2")
        {
            if let Ok(rows) = stmt.query_map(rusqlite::params![cid], |r| r.get::<_, String>(0)) {
                paths.extend(rows.flatten());
            }
        }
        if paths.len() != 1 {
            report.priorities_unresolved += 1;
            continue;
        }
        // `-1` is what `set_review_priority` writes for "cleared" — it maps back to SQL NULL.
        let want: Option<i64> = if ne.p < 0 { None } else { Some(ne.p) };
        let cur: Option<i64> = conn
            .query_row(
                "SELECT review_priority FROM note_meta WHERE path = ?1",
                rusqlite::params![&paths[0]],
                |r| r.get(0),
            )
            .unwrap_or(None);
        if cur == want {
            report.priorities_already_current += 1;
            continue;
        }
        note_writes.push(NoteWrite { path: paths.remove(0), p: want });
    }

    report.planned = writes.len() + note_writes.len();

    // Apply in batches. Every UPDATE fires the sky trigger (DELETE + INSERT over 234k rows) and the
    // outgoing-aggregate pair, so an unbatched loop would hold the writer lock far too long at boot.
    for chunk in writes.chunks(BATCH) {
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("begin restore batch: {}", e))?;
        let mut ok = true;
        for w in chunk {
            let res = conn.execute(
                "UPDATE note_links SET traversal_count = ?2, confidence = ?3, status = ?4,
                        weight = ?5, last_traversed = CASE WHEN ?6 = '' THEN last_traversed ELSE ?6 END
                 WHERE id = ?1",
                rusqlite::params![w.id, w.n, w.conf, w.status, w.weight, w.at],
            );
            if let Err(e) = res {
                // NEVER eprintln! a failure a user needs: Windows GUI release builds send stderr
                // nowhere (search.rs documents this in-code). This exact swallow is why the Boss's
                // 2026-07-27 restore reported "0 of 34 written" with 33 records unaccounted for and
                // no reason recorded anywhere. Log the row AND the values, so one boot names it.
                let msg = format!(
                    "[link_life_restore] row {} UPDATE FAILED: {e} — (n={}, conf={:?}, status={}, w={:.4}, at={:?})",
                    w.id, w.n, w.conf, w.status, w.weight, w.at
                );
                eprintln!("{msg}");
                if let Some(p) = conn.path() {
                    if !p.is_empty() {
                        crate::search::diag_log(std::path::Path::new(p), &msg);
                    }
                }
                ok = false;
                break;
            }
        }
        if ok {
            conn.execute_batch("COMMIT").map_err(|e| format!("commit restore batch: {}", e))?;
            report.restored += chunk.len();
        } else {
            let _ = conn.execute_batch("ROLLBACK");
        }
    }

    // Batched because `UPDATE note_meta` fires `sight_v6_layout_invalidate_au`, which is
    // UNGUARDED (`AFTER UPDATE ON note_meta`, no `WHEN`) and deletes the note's cached layout row
    // on every single update. Harmless — it is a derived cache — but it is per-row work.
    //
    // It does NOT reach `notes_fts`: `note_meta_au` carries `WHEN OLD.name IS NOT NEW.name OR
    // OLD.body_text IS NOT NEW.body_text`, and a review-priority write changes neither. Verified
    // 2026-07-27 by performing exactly this UPDATE on the live database from a bare connection
    // with no tokenizer registered — it succeeded. Stated precisely because the first version of
    // this comment claimed the opposite and would have taught the next reader a false mechanism:
    // the tokenizer this module registers is required by the `note_links` writes above (LL-036),
    // not by this loop.
    for chunk in note_writes.chunks(BATCH) {
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("begin priority batch: {}", e))?;
        let mut ok = true;
        for w in chunk {
            if let Err(e) = conn.execute(
                "UPDATE note_meta SET review_priority = ?2 WHERE path = ?1",
                rusqlite::params![w.path, w.p],
            ) {
                let msg = format!(
                    "[link_life_restore] review_priority UPDATE FAILED for {}: {e} — (p={:?})",
                    w.path, w.p
                );
                eprintln!("{msg}");
                if let Some(p) = conn.path() {
                    if !p.is_empty() {
                        crate::search::diag_log(std::path::Path::new(p), &msg);
                    }
                }
                ok = false;
                break;
            }
        }
        if ok {
            conn.execute_batch("COMMIT").map_err(|e| format!("commit priority batch: {}", e))?;
            report.priorities_restored += chunk.len();
        } else {
            let _ = conn.execute_batch("ROLLBACK");
        }
    }

    Ok(report)
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

#[cfg(test)]
mod tests_mig104_restore {
    use super::*;
    use crate::{link_life, link_life_backfill};

    fn universe() -> (tempfile::TempDir, Connection) {
        let td = tempfile::tempdir().unwrap();
        let cdir = td.path().join(".constellation");
        std::fs::create_dir_all(&cdir).unwrap();
        let conn = crate::search::init_db(&cdir.join("search.db")).unwrap();
        (td, conn)
    }
    fn store(td: &tempfile::TempDir) -> std::path::PathBuf {
        td.path().join(".constellation")
    }
    fn note(conn: &Connection, path: &str, name: &str, cid: &str) {
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, body_text, cid_cn)
             VALUES (?1, ?2, 'L', 0, '', ?3)",
            rusqlite::params![path, name, cid],
        ).unwrap();
    }
    fn link(conn: &Connection, src: &str, tgt: &str, n: i64, conf: &str, status: &str, w: f64) {
        conn.execute(
            "INSERT INTO note_links
               (source_path, source_name, target_name, link_type, library_name,
                traversal_count, confidence, status, weight, last_traversed, created)
             VALUES (?1,'S',?2,'associative','L',?3,?4,?5,?6,'2026-07-01T00:00:00Z','2026-07-01T00:00:00Z')",
            rusqlite::params![src, tgt, n, conf, status, w],
        ).unwrap();
    }
    fn row(conn: &Connection, src: &str, tgt: &str) -> (i64, Option<String>, String, f64) {
        conn.query_row(
            "SELECT traversal_count, confidence, status, weight FROM note_links
             WHERE source_path=?1 AND LOWER(target_name)=LOWER(?2)",
            rusqlite::params![src, tgt],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        ).unwrap()
    }

    /// THE HEADLINE: total index loss, then everything the user earned comes back from the file.
    #[test]
    fn db_loss_round_trip_restores_every_earned_value() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        note(&conn, "/c.md", "C", "C_C");
        note(&conn, "/d.md", "D", "C_D");
        // Earned three different ways: a walked link, a user judgment, a retirement.
        link(&conn, "/a.md", "B", 7, "evidence", "active", 3.079);
        link(&conn, "/a.md", "C", 2, "contested", "active", 2.09);
        link(&conn, "/a.md", "D", 4, "evidence", "archived", 0.0);
        link_life_backfill::seed(&conn, &s).unwrap();

        // TOTAL LOSS of the earned layer: the links come back from the note text as brand new.
        conn.execute("DELETE FROM note_links", []).unwrap();
        link(&conn, "/a.md", "B", 0, "hypothesis", "active", 1.0);
        link(&conn, "/a.md", "C", 0, "hypothesis", "active", 1.0);
        link(&conn, "/a.md", "D", 0, "hypothesis", "active", 1.0);

        let r = restore(&conn, &s).unwrap();
        assert_eq!(r.records, 3);
        assert_eq!(r.restored, 3, "all three earned links must be written back");
        assert_eq!(r.ambiguous_skipped, 0);

        let (n, c, st, w) = row(&conn, "/a.md", "B");
        assert_eq!((n, c.as_deref(), st.as_str()), (7, Some("evidence"), "active"));
        assert!((w - crate::search::earned_link_weight(7)).abs() < 1e-9, "weight is DERIVED from n");
        let (n, c, _, _) = row(&conn, "/a.md", "C");
        assert_eq!((n, c.as_deref()), (2, Some("contested")), "a user judgment survives index loss");
        let (n, _, st, w) = row(&conn, "/a.md", "D");
        assert_eq!((n, st.as_str()), (4, "archived"), "a RETIRED link stays retired — the wikilink is still in the note");
        assert!((w - crate::search::earned_link_weight(4)).abs() < 1e-9);
    }

    /// BOSS-FOUND CONSTRAINT (the `banana` shape): a name-keyed record matching SEVERAL rows must
    /// be skipped and reported — never distributed. RED without the rule: both links would take
    /// the folded max, and the one that earned less would gain walks it never made.
    #[test]
    fn an_ambiguous_name_keyed_record_is_skipped_never_distributed() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        // Two notes share the name, so the seed refuses to identify the target…
        note(&conn, "/lib1/banana.md", "Banana", "C_B1");
        note(&conn, "/lib2/banana.md", "Banana", "C_B2");
        // …and the source has TWO links to that name, with DIFFERENT earned counts.
        link(&conn, "/a.md", "Banana", 5, "hypothesis", "active", 2.79);
        link(&conn, "/a.md", "banana", 1, "hypothesis", "active", 1.693);
        link_life_backfill::seed(&conn, &s).unwrap();

        // The ledger holds ONE name-keyed record folded to the max (5).
        let (folded, _) = link_life::read_folded(&s);
        assert_eq!(folded.len(), 1);
        assert_eq!(folded.values().next().unwrap().n, 5);

        // Wipe the earned layer, then restore.
        conn.execute("UPDATE note_links SET traversal_count = 0, weight = 1.0", []).unwrap();
        let r = restore(&conn, &s).unwrap();
        assert_eq!(r.ambiguous_skipped, 1, "the ambiguous record must be SKIPPED and counted");
        assert_eq!(r.restored, 0, "nothing may be written");
        let counts: Vec<i64> = conn
            .prepare("SELECT traversal_count FROM note_links ORDER BY id").unwrap()
            .query_map([], |r| r.get(0)).unwrap().flatten().collect();
        assert_eq!(counts, vec![0, 0], "neither link may be handed a count it might not have earned");
    }

    /// An UNAMBIGUOUS name-keyed record (only one matching row) restores normally — the rule is
    /// scoped to genuine ambiguity, not applied everywhere out of caution.
    #[test]
    fn an_unambiguous_name_keyed_record_still_restores() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        // Target has no identity at all → the record must key on the name.
        link(&conn, "/a.md", "Nowhere Note", 6, "hypothesis", "active", 2.95);
        link_life_backfill::seed(&conn, &s).unwrap();
        conn.execute("UPDATE note_links SET traversal_count = 0, weight = 1.0", []).unwrap();
        let r = restore(&conn, &s).unwrap();
        assert_eq!((r.restored, r.ambiguous_skipped), (1, 0));
        assert_eq!(row(&conn, "/a.md", "Nowhere Note").0, 6);
    }

    #[test]
    fn restore_is_idempotent_and_the_second_pass_writes_nothing() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        link(&conn, "/a.md", "B", 3, "evidence", "active", 2.386);
        link_life_backfill::seed(&conn, &s).unwrap();
        conn.execute("UPDATE note_links SET traversal_count = 0, weight = 1.0", []).unwrap();

        let first = restore(&conn, &s).unwrap();
        assert_eq!(first.restored, 1);
        let second = restore(&conn, &s).unwrap();
        assert_eq!(second.restored, 0, "a second pass must write nothing");
        assert_eq!(second.already_current, 1, "…and must say the row is already current");
    }

    /// The DB may hold a walk NEWER than the ledger's last flush (the ledger is append-only but the
    /// process could have died before flushing). The restore must never ratchet a count DOWN.
    #[test]
    fn a_newer_db_count_is_never_ratcheted_down_by_an_older_ledger() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        link(&conn, "/a.md", "B", 2, "hypothesis", "active", 2.09);
        link_life_backfill::seed(&conn, &s).unwrap();
        // The user kept reading after the seed; the DB is ahead.
        conn.execute("UPDATE note_links SET traversal_count = 9", []).unwrap();
        restore(&conn, &s).unwrap();
        assert_eq!(row(&conn, "/a.md", "B").0, 9, "the higher count wins — never a downgrade");
    }

    /// The weight heal: the 236-row shape (a weight the earned curve cannot produce) is recomputed,
    /// and rows already on the curve are NOT written.
    #[test]
    fn the_weight_heal_fixes_off_curve_rows_and_touches_nothing_else() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        note(&conn, "/c.md", "C", "C_C");
        // Off the curve — the live decay-residue shape, with a real earned count.
        link(&conn, "/a.md", "B", 1, "contested", "active", 0.526);
        // Already exactly on the curve for its count.
        link(&conn, "/a.md", "C", 3, "contested", "active", crate::search::earned_link_weight(3));
        link_life_backfill::seed(&conn, &s).unwrap();

        let r = restore(&conn, &s).unwrap();
        let (_, _, _, w_b) = row(&conn, "/a.md", "B");
        assert!((w_b - crate::search::earned_link_weight(1)).abs() < 1e-9, "off-curve weight recomputed");
        assert_eq!(r.weights_healed, 1, "exactly the off-curve row counts as a heal");
        assert_eq!(r.already_current, 1, "the on-curve row is left alone");
    }

    /// A record whose link no longer exists is counted, not an error — the user may have deleted
    /// that wikilink since. NOTE the fixture keeps another link alive: an index with ZERO links is
    /// a different condition entirely (a rebuild in progress) and is handled by the guard below.
    #[test]
    fn a_record_whose_link_is_gone_is_counted_not_fatal() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        note(&conn, "/c.md", "C", "C_C");
        link(&conn, "/a.md", "B", 4, "evidence", "active", 2.6);
        link(&conn, "/a.md", "C", 2, "evidence", "active", 2.09);
        link_life_backfill::seed(&conn, &s).unwrap();
        // The user deleted ONE wikilink; the index is otherwise healthy.
        conn.execute("DELETE FROM note_links WHERE LOWER(target_name) = 'b'", []).unwrap();
        let r = restore(&conn, &s).unwrap();
        assert!(!r.index_not_ready, "one missing link is not a rebuild in progress");
        assert_eq!(r.no_matching_row, 1, "the vanished link's record is counted");
    }

    /// ★ THE BOSS-FOUND BUG (live, 2026-07-27). On a REBUILT index this pass can start while
    /// indexing is still running. Every record then looks like a link that no longer exists — and
    /// the first cut *stamped itself complete* on exactly that reading, so the earned data would
    /// never have come back. Two guards now: it draws NO conclusion from an unpopulated index, and
    /// it carries no stamp at all, so the next boot retries.
    #[test]
    fn an_index_still_rebuilding_concludes_nothing_and_writes_nothing() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        link(&conn, "/a.md", "B", 7, "evidence", "active", 3.079);
        link_life_backfill::seed(&conn, &s).unwrap();

        // The state the Boss hit: the ledger is full, the index is not yet populated.
        conn.execute("DELETE FROM note_links", []).unwrap();
        let r = restore(&conn, &s).unwrap();
        assert!(r.index_not_ready, "an empty index must be reported as NOT READY…");
        assert_eq!(r.no_matching_row, 0, "…never as 'the links are gone'");
        assert_eq!(r.restored, 0);

        // …and once indexing finishes, a later pass restores everything. This is only possible
        // because the pass is unstamped: a stamp here would have made the loss permanent.
        link(&conn, "/a.md", "B", 0, "hypothesis", "active", 1.0);
        let r2 = restore(&conn, &s).unwrap();
        assert!(!r2.index_not_ready);
        assert_eq!(r2.restored, 1, "the retry after the rebuild is what saves the earned data");
        assert_eq!(row(&conn, "/a.md", "B").0, 7);
    }

    /// An ABSENT store is a fact — an empty store, not an error — so this is a clean no-op.
    /// (Renamed from `an_unreadable_store_writes_nothing`, which is a different condition
    /// entirely and is now covered by the test below.)
    #[test]
    fn an_absent_store_writes_nothing() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        link(&conn, "/a.md", "B", 3, "evidence", "active", 2.386);
        let r = restore(&conn, &s).unwrap();
        assert_eq!((r.records, r.restored), (0, 0));
        assert_eq!(row(&conn, "/a.md", "B").0, 3, "the DB is left exactly as it was");
    }

    /// ★ SLICE-7 FINDING. The `refuse_write` guard above the write planner could not fire: the
    /// flag was set ONLY inside `link_life::quarantine`, which returns its own report, while the
    /// reader always built a fresh one in which it was `false`. A dead branch that read as a live
    /// protection — the LL-035 shape ("X is inactive" is a runtime claim). The reader now observes
    /// the quarantine file on disk, so the guard is real; this test puts the store in the state
    /// that fires it and asserts nothing is written.
    #[test]
    fn a_quarantined_store_writes_nothing() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        link(&conn, "/a.md", "B", 6, "evidence", "active", 2.95);
        link_life_backfill::seed(&conn, &s).unwrap();
        conn.execute("UPDATE note_links SET traversal_count = 0, weight = 1.0", []).unwrap();

        // The store went structurally unusable and was renamed aside; the user has not dealt
        // with it yet. The DB is the survivor (§3.7) and must not be written from a store we
        // could not read.
        std::fs::write(s.join("earned.corrupt-2026-07-27T000000Z.jsonl"), "\u{0}garbage").unwrap();
        let r = restore(&conn, &s).unwrap();
        assert_eq!(r.restored, 0, "a quarantined store must produce NO writes");
        assert_eq!(row(&conn, "/a.md", "B").0, 0, "the DB is left exactly as it was");

        // Acknowledging is moving the file away — then the restore proceeds normally.
        std::fs::remove_file(s.join("earned.corrupt-2026-07-27T000000Z.jsonl")).unwrap();
        assert_eq!(restore(&conn, &s).unwrap().restored, 1);
        assert_eq!(row(&conn, "/a.md", "B").0, 6);
    }

    /// ★ SLICE-7 FINDING — the Plan's Slice 6 says "restores review priority too", and the code
    /// did not. `set_review_priority` has been appending and fsyncing `priority` records since
    /// Slice 4, and nothing ever read one back: the fold's key function required a target, so
    /// every one was silently dropped. Losing `search.db` therefore still cost the user every
    /// review priority they had set — in the one pass whose whole job is that it must not.
    #[test]
    fn a_review_priority_survives_losing_the_index() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        link(&conn, "/a.md", "B", 1, "hypothesis", "active", 1.693);
        conn.execute("UPDATE note_meta SET review_priority = 5 WHERE path = '/a.md'", []).unwrap();
        // The record the live command writes, file-first and fsync'd, before touching the DB.
        link_life::append(&s, link_life::Stream::Earned,
            &[link_life::priority_line("C_A", 5, "2026-07-27T09:00:00Z")]).unwrap();

        // Total loss of the index: the note comes back from the file with no priority at all.
        conn.execute("UPDATE note_meta SET review_priority = NULL", []).unwrap();

        let r = restore(&conn, &s).unwrap();
        assert_eq!(r.priorities_restored, 1, "the decision must come back");
        let p: Option<i64> = conn.query_row(
            "SELECT review_priority FROM note_meta WHERE path = '/a.md'", [], |r| r.get(0)).unwrap();
        assert_eq!(p, Some(5));

        // Idempotent, like everything else in this pass.
        let again = restore(&conn, &s).unwrap();
        assert_eq!((again.priorities_restored, again.priorities_already_current), (0, 1));
    }

    /// `-1` is what `set_review_priority` writes for "cleared". Clearing is itself a decision and
    /// must restore as SQL NULL — not as the number -1, and not by being ignored.
    #[test]
    fn a_cleared_priority_restores_as_null_not_as_minus_one() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        link(&conn, "/a.md", "B", 1, "hypothesis", "active", 1.693);
        link_life::append(&s, link_life::Stream::Earned, &[
            link_life::priority_line("C_A", 5, "2026-07-27T09:00:00Z"),
            link_life::priority_line("C_A", -1, "2026-07-27T10:00:00Z"), // the user cleared it
        ]).unwrap();
        conn.execute("UPDATE note_meta SET review_priority = 5 WHERE path = '/a.md'", []).unwrap();

        let r = restore(&conn, &s).unwrap();
        assert_eq!(r.priorities_restored, 1);
        let p: Option<i64> = conn.query_row(
            "SELECT review_priority FROM note_meta WHERE path = '/a.md'", [], |r| r.get(0)).unwrap();
        assert_eq!(p, None, "the LAST decision was to clear it");
    }

    /// The same refusal that governs an ambiguous link record: a decision is never applied to a
    /// note we cannot identify uniquely.
    #[test]
    fn a_priority_for_a_note_that_is_gone_is_counted_not_applied() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        link(&conn, "/a.md", "B", 1, "hypothesis", "active", 1.693);
        link_life::append(&s, link_life::Stream::Earned,
            &[link_life::priority_line("C_VANISHED", 4, "2026-07-27T09:00:00Z")]).unwrap();
        let r = restore(&conn, &s).unwrap();
        assert_eq!((r.priorities_restored, r.priorities_unresolved), (0, 1));
    }

    /// ★ THE 2026-07-27 REGRESSION TEST, and an indictment of the other tests in this module.
    ///
    /// Every test here builds its DB with `init_db`, which registers the custom FTS5 tokenizer —
    /// so the fixture was MORE CAPABLE than production, where `run()` opens a bare
    /// `Connection::open`. All 51 tests passed while the live restore lost all 33 writes to
    /// `no such tokenizer: constellation`.
    ///
    /// This test reproduces PRODUCTION's connection: raw open, no tokenizer. It asserts the
    /// precondition is REAL (the write fails without registration) and that registering it — what
    /// `run()` now does — makes the restore work. If someone removes that call, this goes red.
    #[test]
    fn a_bare_connection_cannot_write_note_links_and_registering_the_tokenizer_fixes_it() {
        let td = tempfile::tempdir().unwrap();
        let cdir = td.path().join(".constellation");
        std::fs::create_dir_all(&cdir).unwrap();
        let dbp = cdir.join("search.db");
        {
            let conn = crate::search::init_db(&dbp).unwrap();
            note(&conn, "/a.md", "A", "C_A");
            note(&conn, "/b.md", "B", "C_B");
            link(&conn, "/a.md", "B", 6, "evidence", "active", 2.95);
            link_life_backfill::seed(&conn, &cdir).unwrap();
            conn.execute("UPDATE note_links SET traversal_count = 0, weight = 1.0", []).unwrap();
        } // closed — the tokenizer registration died with that connection

        // PRODUCTION's shape: a bare connection, exactly what `run()` opens.
        let mut bare = Connection::open(&dbp).unwrap();
        bare.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;").unwrap();

        // The precondition is real: the write reaches notes_fts through
        // note_links_outgoing_au -> note_meta -> note_meta_au, and cannot be serviced.
        let unregistered = bare.execute(
            "UPDATE note_links SET traversal_count = 6, weight = 2.95 WHERE source_path = '/a.md'", [],
        );
        let msg = unregistered.map(|_| String::new()).unwrap_err().to_string();
        assert!(
            msg.contains("tokenizer"),
            "a bare connection must fail on the FTS trigger chain — that is the whole precondition; got: {msg}"
        );

        // What run() now does — after which the restore lands.
        crate::search::register_fts5_tokenizer(&mut bare).unwrap();
        let r = restore(&bare, &cdir).unwrap();
        assert_eq!(r.planned, 1, "one write planned");
        assert_eq!(r.restored, 1, "and APPLIED — the tokenizer is what makes the write serviceable");
        assert_eq!(row(&bare, "/a.md", "B").0, 6);
    }

    /// An identity-keyed record must still resolve after the TARGET has been renamed — the identity
    /// is the durable half of the key, the name is not.
    #[test]
    fn an_identity_keyed_record_survives_a_target_rename() {
        let (td, conn) = universe();
        let s = store(&td);
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "Old Title", "C_B");
        link(&conn, "/a.md", "Old Title", 5, "evidence", "active", 2.79);
        link_life_backfill::seed(&conn, &s).unwrap();

        // The target is renamed; the wikilink (and so note_links.target_name) follows.
        conn.execute("UPDATE note_meta SET name = 'New Title' WHERE cid_cn = 'C_B'", []).unwrap();
        conn.execute("UPDATE note_links SET target_name = 'New Title', traversal_count = 0, weight = 1.0", []).unwrap();

        let r = restore(&conn, &s).unwrap();
        assert_eq!(r.restored, 1, "the identity resolves the record to the renamed target");
        assert_eq!(row(&conn, "/a.md", "New Title").0, 5);
    }

    /// PJ-435 — the earned layer must survive the universe MOVING, not only the index dying in
    /// place. `store_dir` derives the store from the CONNECTION's own path, so wherever
    /// `search.db` sits, `earned.jsonl` is read beside it — this test pins that property by
    /// physically renaming the whole universe directory between seed and restore, then restoring
    /// through the shipping function at the NEW location.
    ///
    /// It also pins the two measured CASUALTIES, so the boundary is executable rather than prose
    /// (CLAUDE.md "Storage — WHAT SURVIVES LOSING THE INDEX", corrected 2026-08-29):
    ///   * a link's `created` date is NOT in the store and does NOT come back;
    ///   * a `review_schedule` row is keyed on an absolute path and is orphaned by the move.
    /// If either assertion ever fails because the gap was CLOSED — update that CLAUDE.md section
    /// in the same commit, per its own closing instruction, and then this test.
    #[test]
    fn the_earned_layer_survives_the_universe_moving() {
        let root = tempfile::tempdir().unwrap();
        let old_u = root.path().join("old").join("Universe");
        let new_u = root.path().join("elsewhere").join("Universe");
        std::fs::create_dir_all(old_u.join(".constellation")).unwrap();

        let old_store = old_u.join(".constellation");
        {
            let conn = crate::search::init_db(&old_store.join("search.db")).unwrap();
            conn.execute(
                "INSERT INTO note_meta (path, name, library_name, modified, body_text, cid_cn)
                 VALUES (?1, 'A', 'L', 0, '', 'C_A')",
                rusqlite::params![old_u.join("a.md").to_string_lossy()],
            ).unwrap();
            conn.execute(
                "INSERT INTO note_meta (path, name, library_name, modified, body_text, cid_cn)
                 VALUES (?1, 'B', 'L', 0, '', 'C_B')",
                rusqlite::params![old_u.join("b.md").to_string_lossy()],
            ).unwrap();
            conn.execute(
                "INSERT INTO note_links
                   (source_path, source_name, target_name, link_type, library_name,
                    traversal_count, confidence, status, weight, last_traversed, created)
                 VALUES (?1, 'A', 'B', 'associative', 'L',
                    5, 'evidence', 'active', 2.79, '2026-07-01T00:00:00Z', '2024-03-01T00:00:00Z')",
                rusqlite::params![old_u.join("a.md").to_string_lossy()],
            ).unwrap();
            // A review rhythm keyed, as shipped, on the ABSOLUTE path.
            conn.execute(
                "INSERT INTO review_schedule (path, reason, due_days, is_checkpoint)
                 VALUES (?1, 'r', 3, 0)",
                rusqlite::params![old_u.join("a.md").to_string_lossy()],
            ).unwrap();
            link_life_backfill::seed(&conn, &old_store).unwrap();
        } // connection CLOSED before the move — Windows will not rename an open database's dir

        // THE MOVE — the user drags the folder somewhere else entirely.
        std::fs::create_dir_all(new_u.parent().unwrap()).unwrap();
        std::fs::rename(&old_u, &new_u).unwrap();
        let new_store = new_u.join(".constellation");
        assert!(new_store.join("earned.jsonl").exists(), "the store travelled with the folder");

        let conn = crate::search::init_db(&new_store.join("search.db")).unwrap();
        // A rebuild at the new location: re-indexing rewrites note_meta to the NEW paths (the
        // cid-keyed self-heal, or a fresh insert — either way the cids are unchanged), and the
        // link returns from the note text as brand new — fresh count, fresh confidence, and a
        // FRESH birth date. (This is what a Full re-read does to all 234,917 of the Boss's
        // links, which is why the docs now warn against it.) The first version of this test
        // skipped the note_meta rewrite and got restored=0: the restore resolves a cid to its
        // CURRENT path, so an inconsistent fixture matched nothing — the failure was the
        // fixture's, and it is kept in this comment because it demonstrates the matching is
        // genuinely identity-first.
        conn.execute(
            "UPDATE note_meta SET path = ?1 WHERE cid_cn = 'C_A'",
            rusqlite::params![new_u.join("a.md").to_string_lossy()],
        ).unwrap();
        conn.execute(
            "UPDATE note_meta SET path = ?1 WHERE cid_cn = 'C_B'",
            rusqlite::params![new_u.join("b.md").to_string_lossy()],
        ).unwrap();
        conn.execute("DELETE FROM note_links", []).unwrap();
        conn.execute(
            "INSERT INTO note_links
               (source_path, source_name, target_name, link_type, library_name,
                traversal_count, confidence, status, weight, last_traversed, created)
             VALUES (?1, 'A', 'B', 'associative', 'L',
                0, 'hypothesis', 'active', 1.0, '', '2026-08-30T00:00:00Z')",
            rusqlite::params![new_u.join("a.md").to_string_lossy()],
        ).unwrap();

        // CONTROL — a restore aimed at the OLD location must find nothing. If this half ever
        // passes with records > 0 the positive half below proves nothing (a check that cannot
        // fail is not a check).
        let ghost = restore(&conn, &old_store).unwrap();
        assert_eq!(ghost.records, 0, "the old location is empty — the store MOVED");

        // THE POINT — restored from beside the database, at the new location.
        let r = restore(&conn, &new_store).unwrap();
        assert_eq!(r.records, 1);
        assert_eq!(r.restored, 1, "what the user earned follows the folder");

        let (n, conf, status, w, created): (i64, String, String, f64, String) = conn
            .query_row(
                "SELECT traversal_count, confidence, status, weight, created FROM note_links",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap();
        assert_eq!((n, conf.as_str(), status.as_str()), (5, "evidence", "active"));
        assert!((w - crate::search::earned_link_weight(5)).abs() < 1e-9);

        // THE TWO CASUALTIES, pinned as facts rather than left as prose:
        assert_eq!(
            created, "2026-08-30T00:00:00Z",
            "the link's BIRTH DATE is not in the store and did not come back — if this failed \
             because the store now carries it, close the loop: CLAUDE.md storage section, then here"
        );
        let review_path: String = conn
            .query_row("SELECT path FROM review_schedule", [], |row| row.get(0))
            .unwrap();
        assert!(
            review_path.starts_with(&old_u.to_string_lossy().to_string()),
            "the review row still points at the OLD location — orphaned by the move, exactly as \
             measured on the Boss's universe (8,033 rows keyed by absolute path)"
        );
    }
}
