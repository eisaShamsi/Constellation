//! MIG-104 Slice 5 — seed the Earned-Life Ledger from the existing index, once.
//!
//! **This is the moment the currently-at-risk data stops being single-copy.** Everything the user
//! earned before the write hooks existed — 33 links walked over months, and the confidence tiers
//! those walks produced — lives ONLY in `search.db` today. `search.db` is renameable-aside by the
//! schema gate, restorable from a stale backup, and 2 GB of it syncs badly. One pass over it puts
//! the earned half on disk in plain text, and from then on the hooks keep it current.
//!
//! **Idempotent by construction, not by luck.** Every record carries an ABSOLUTE `n` and the fold
//! takes the max, so running this twice, or three times, or against a store that already holds a
//! newer walk, converges on the same answer. The `schema_versions` stamp is therefore an
//! optimisation (don't do pointless work), not a correctness guard — which is the right way round:
//! a correctness guard that can be lost with a restored database is not a guard.
//!
//! **It records nothing it cannot key, and it SAYS how many it skipped.** A record whose source
//! note has no identity cannot ever be restored to anything, so writing it would be theatre. The
//! skip count is surfaced in the diagnostics line rather than swallowed.
//!
//! Deliberately NOT done here (both verified against the live data, both would cause harm):
//!   * **No force-stamping of missing identities.** `canonical::ensure_cid_cn` WRITES the note file.
//!     The cid-less notes are templates and `.trash` copies — stamping a template changes what
//!     every future note spawned from it emits. Zero live content notes lack an identity, so there
//!     is nothing to gain and a real side effect to cause.
//!   * **No recording of off-curve `weight` rows.** 236 rows carry weights `1 + ln(1+n)` cannot
//!     produce (decay residue). They are not earned data; seeding them would put 236 junk records
//!     in the durable store. `weight` is DERIVED on restore from `n`, which heals all 236 for free.

use rusqlite::Connection;
use std::time::Duration;
use tauri::Manager;

use crate::search::SearchState;

/// Bump to force a re-seed (e.g. if the record shape changes).
///
/// **v2 (2026-07-27)** — v1 shipped two defects the Boss found by reading his own ledger:
/// the target join fanned out on duplicate note names (38 earned links became 44 records, 6 of
/// them asserting links he never walked), and seeded decision timestamps were borrowed from
/// `last_traversed` without saying so. Both fixed; the bump makes the corrected pass re-run.
/// Re-running is safe by arithmetic (absolute `n` + max-fold), but the 6 spurious v1 records
/// key on identities the corrected pass never writes, so they cannot be folded away — the v1
/// `earned.jsonl` must be deleted before the re-seed, not merged with it.
pub(crate) const SCHEMA_VERSION: i64 = 2;

/// The ONE earned predicate — the single definition of "did the user earn anything here?".
///
/// Measured 2026-07-27 on the live index: **35 rows match; 2 are `structural` and excluded by
/// design (PJ-065 — structural edges carry no living-link apparatus), leaving 33 recordable**
/// across 25 source notes, out of 234,233 links (0.014%).
///
/// TWO PROHIBITIONS, both verified against the live data — do not "improve" this by adding them:
///   * **`weight <> 1.0`** — 236 rows carry values the earned curve cannot produce (115 at 0.526,
///     119 at 0.564, both with `traversal_count = 0`). Decay residue, not earned data.
///   * **`last_traversed <> ''`** — non-empty on ALL 234,233 rows, because `index_note` stamps it
///     at insert. It identifies nothing.
pub(crate) const EARNED_PREDICATE: &str = "traversal_count > 0 \
     OR status <> 'active' \
     OR (confidence IS NOT NULL AND confidence NOT IN ('hypothesis','structural'))";

/// True once the seed has run to completion and been stamped.
pub(crate) fn is_stamped(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT version FROM schema_versions WHERE module = 'link_life_backfill'",
        [],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        >= SCHEMA_VERSION
}

/// What one seeding pass did, so the diagnostics line can state it rather than imply it.
#[derive(Debug, Default, PartialEq)]
pub struct SeedReport {
    /// Rows the earned predicate matched.
    pub matched: usize,
    /// Records written.
    pub recorded: usize,
    /// Rows skipped because the SOURCE note has no identity — unkeyable, therefore unrestorable.
    pub skipped_no_source_cid: usize,
    /// Rows skipped because they are `structural` (no living-link apparatus by design).
    pub skipped_structural: usize,
}

/// Schedule the one-shot seed on a background thread, after paint. Silent no-op once stamped.
/// Mirrors `link_boot_index::maybe_schedule` — the proven shape: own thread, own connection,
/// failure non-fatal and logged.
pub fn maybe_schedule(app: tauri::AppHandle) {
    if !crate::link_life::EARNED_LEDGER_WRITE {
        return;
    }
    let state = app.state::<SearchState>();
    let needs_run = {
        let Ok(guard) = state.db.lock() else { return };
        let Some(conn) = guard.as_ref() else { return };
        !is_stamped(conn)
    };
    if !needs_run {
        return;
    }
    let app_bg = app.clone();
    std::thread::spawn(move || match run(&app_bg) {
        Ok(r) => diag(
            &app_bg,
            &format!(
                "[link_life_backfill] seeded the earned-life ledger: {} recorded of {} earned rows \
                 (skipped {} with no source identity, {} structural) — stamped",
                r.recorded, r.matched, r.skipped_no_source_cid, r.skipped_structural
            ),
        ),
        Err(e) => diag(&app_bg, &format!("[link_life_backfill] FAILED (non-fatal): {}", e)),
    });
}

/// Read the earned rows on a DEDICATED connection, append them, then stamp.
///
/// The stamp is written LAST and only on success, so an interrupted pass simply re-runs next boot
/// (and the fold makes the partial first pass harmless).
fn run(app: &tauri::AppHandle) -> Result<SeedReport, String> {
    let path = crate::search::db_path(app)?;
    let dir = path
        .parent()
        .ok_or("search.db has no parent dir")?
        .to_path_buf();

    let conn = Connection::open(&path).map_err(|e| format!("open link_life_backfill conn: {}", e))?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;

    // The store's .gitignore rides this pass — it is what makes the File-Over-App claim
    // operationally true, and it must exist before the first user ever syncs the folder.
    let _ = crate::link_life::ensure_gitignore(&dir);

    let report = seed(&conn, &dir)?;

    conn.execute(
        "INSERT INTO schema_versions (module, version, updated_at) VALUES ('link_life_backfill', ?1, ?2)
         ON CONFLICT(module) DO UPDATE SET version = excluded.version, updated_at = excluded.updated_at",
        rusqlite::params![SCHEMA_VERSION, chrono::Utc::now().timestamp()],
    )
    .map_err(|e| format!("stamp: {}", e))?;

    Ok(report)
}

/// The seeding pass itself — separated from `run` so it is testable against a real `init_db`
/// connection with no AppHandle.
pub(crate) fn seed(conn: &Connection, dir: &std::path::Path) -> Result<SeedReport, String> {
    // ONE output row per earned link row — never more.
    //
    // Boss-found 2026-07-27: this was a `LEFT JOIN note_meta tgt ON LOWER(tgt.name) = ...`, and a
    // link whose target NAME is shared by several notes therefore fanned out into several records,
    // each asserting a different target identity. Measured on the live index: 38 earned rows became
    // 44 emitted, because 3 notes are named `السعودية`, 2 `فلسفة`, 2 `banana`, 2 `collision test`.
    // The extra records claim the user walked links they never walked, and on restore could hand a
    // count to the wrong link.
    //
    // The fix is not a better guess — it is a REFUSAL to guess. A correlated subquery resolves the
    // target identity ONLY when exactly one indexed note carries the name; when the name is
    // ambiguous it yields `''`, and the record keys on the target NAME instead. That is precisely
    // what the fold's name-key fallback exists for, and it never invents a link.
    let sql = format!(
        "SELECT l.source_path, l.target_name, l.traversal_count, l.confidence, l.status,
                COALESCE(src.cid_cn, '') AS src_cid,
                (SELECT CASE WHEN COUNT(*) = 1 THEN MAX(t.cid_cn) ELSE '' END
                   FROM note_meta t
                   WHERE LOWER(t.name) = LOWER(l.target_name) AND t.cid_cn != '') AS tgt_cid,
                (SELECT CASE WHEN COUNT(*) = 1 THEN MAX(t.name) ELSE l.target_name END
                   FROM note_meta t
                   WHERE LOWER(t.name) = LOWER(l.target_name) AND t.cid_cn != '') AS tgt_label,
                COALESCE(l.last_traversed, '') AS at
         FROM note_links l
         LEFT JOIN note_meta src ON src.path = l.source_path
         WHERE {EARNED_PREDICATE}"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare seed: {}", e))?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(2)?,               // traversal_count
                r.get::<_, Option<String>>(3)?,    // confidence
                r.get::<_, String>(4)?,            // status
                r.get::<_, String>(5)?,            // src_cid
                r.get::<_, String>(6)?,            // tgt_cid
                r.get::<_, String>(7)?,            // tgt_label
                r.get::<_, String>(8)?,            // last_traversed
            ))
        })
        .map_err(|e| format!("query seed: {}", e))?;

    let mut report = SeedReport::default();
    let mut lines: Vec<String> = Vec::new();

    for row in rows.flatten() {
        let (n, conf, status, src_cid, tgt_cid, tgt_label, at) = row;
        report.matched += 1;

        // Structural edges carry no living-link apparatus (PJ-065). The predicate cannot exclude
        // them by itself — a structural row can still have `traversal_count > 0` — so the filter
        // lives here, counted rather than silently dropped.
        if conf.as_deref() == Some("structural") {
            report.skipped_structural += 1;
            continue;
        }
        // A record whose SOURCE has no identity can never be restored to anything. Writing it
        // would be theatre; skipping it silently would be dishonest. Counted and surfaced.
        if src_cid.is_empty() {
            report.skipped_no_source_cid += 1;
            continue;
        }

        // Only a NON-derivable confidence is a decision worth a record; a tier that merely
        // restates the count is not (see link_life::is_derivable_tier).
        // The stamp is `last_traversed` — the only timestamp the index carries. For a WALK that is
        // truthful; for a decision it is not (there is no "when was this archived" column), which
        // the Boss's own data exposed: a Contested click at 09:21:25 seeded as 09:13:51, the walk's
        // time. The time cannot be made true, so every seeded line is MARKED as derived — a reader
        // can then tell a witnessed decision from a reconstructed one.
        let stamp = if at.is_empty() { chrono::Utc::now().to_rfc3339() } else { at };
        let seeded = crate::link_life::mark_seeded;
        if n > 0 {
            lines.push(seeded(crate::link_life::walk_line(&src_cid, &tgt_cid, &tgt_label, n, &stamp)));
        }
        if let Some(c) = conf.as_deref() {
            if c != "hypothesis" && !crate::link_life::is_derivable_tier(c, n) {
                lines.push(seeded(crate::link_life::trust_line(&src_cid, &tgt_cid, &tgt_label, c, &stamp)));
            }
        }
        if status == "archived" {
            lines.push(seeded(crate::link_life::retire_line(&src_cid, &tgt_cid, &tgt_label, &stamp)));
        }
        report.recorded += 1;
    }

    if !lines.is_empty() {
        crate::link_life::append(dir, crate::link_life::Stream::Earned, &lines)?;
        // fsync ONCE for the whole pass: this is the moment months of earned data becomes
        // durable, and it happens once per universe — the 3.4 ms is irrelevant here.
        crate::link_life::fsync(dir, crate::link_life::Stream::Earned)?;
    }
    Ok(report)
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

#[cfg(test)]
mod tests_mig104_backfill {
    use super::*;
    use crate::link_life;

    /// A universe fixture with the REAL schema, so the predicate is exercised against production
    /// column definitions and defaults rather than a hand-rolled table.
    fn universe() -> (tempfile::TempDir, Connection) {
        let td = tempfile::tempdir().unwrap();
        let cdir = td.path().join(".constellation");
        std::fs::create_dir_all(&cdir).unwrap();
        let conn = crate::search::init_db(&cdir.join("search.db")).unwrap();
        (td, conn)
    }

    fn note(conn: &Connection, path: &str, name: &str, cid: &str) {
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, body_text, cid_cn)
             VALUES (?1, ?2, 'L', 0, '', ?3)",
            rusqlite::params![path, name, cid],
        )
        .unwrap();
    }

    fn link(conn: &Connection, src: &str, tgt: &str, n: i64, conf: &str, status: &str, w: f64) {
        conn.execute(
            "INSERT INTO note_links
               (source_path, source_name, target_name, link_type, library_name,
                traversal_count, confidence, status, weight, last_traversed, created)
             VALUES (?1,'S',?2,'associative','L',?3,?4,?5,?6,'2026-07-01T00:00:00Z','2026-07-01T00:00:00Z')",
            rusqlite::params![src, tgt, n, conf, status, w],
        )
        .unwrap();
    }

    #[test]
    fn the_predicate_matches_earned_rows_and_excludes_the_unearned() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        // EARNED, three different ways:
        link(&conn, "/a.md", "B", 3, "evidence", "active", 2.386); // walked
        link(&conn, "/a.md", "walked-once", 1, "hypothesis", "active", 1.693);
        link(&conn, "/a.md", "retired", 0, "hypothesis", "archived", 0.0); // a decision
        link(&conn, "/a.md", "contested-thing", 0, "contested", "active", 1.0); // a judgment
        // NOT earned — the shapes the two prohibitions exist to keep out:
        link(&conn, "/a.md", "untouched", 0, "hypothesis", "active", 1.0);
        link(&conn, "/a.md", "decay-residue", 0, "hypothesis", "active", 0.526); // off-curve weight
        link(&conn, "/a.md", "also-residue", 0, "hypothesis", "active", 0.564);

        let r = seed(&conn, td.path()).unwrap();
        assert_eq!(r.matched, 4, "only the four earned rows may match the predicate");
        assert_eq!(r.recorded, 4);
        let (map, rep) = link_life::read_folded(td.path());
        assert_eq!(rep.skipped_lines, 0);
        assert_eq!(map.len(), 4, "one folded record per earned link");
        // The off-curve weights are absent — they are decay residue, not earned data.
        assert!(!map.keys().any(|k| k.contains("residue")));
    }

    #[test]
    fn structural_rows_are_skipped_and_counted() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        // A structural edge that HAS been traversed — the predicate matches it, so the filter
        // must be in the loop, not the WHERE clause.
        link(&conn, "/a.md", "parent-toc", 2, "structural", "active", 2.09);
        let r = seed(&conn, td.path()).unwrap();
        assert_eq!((r.matched, r.skipped_structural, r.recorded), (1, 1, 0));
        assert!(link_life::read_folded(td.path()).0.is_empty());
    }

    #[test]
    fn orphan_source_rows_are_skipped_and_counted_never_written() {
        let (td, conn) = universe();
        // No note_meta row for the source at all — the live shape of the 2 orphans (one under a
        // retired library root, one whose note was deleted).
        link(&conn, "/gone.md", "B", 5, "evidence", "active", 2.79);
        let r = seed(&conn, td.path()).unwrap();
        assert_eq!((r.matched, r.skipped_no_source_cid, r.recorded), (1, 1, 0));
        assert!(
            link_life::read_folded(td.path()).0.is_empty(),
            "a record we cannot key is a record we can never restore — it must not be written"
        );
    }

    #[test]
    fn a_rerun_is_a_no_op_by_ARITHMETIC_not_by_the_stamp() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        link(&conn, "/a.md", "B", 4, "evidence", "active", 2.6);

        seed(&conn, td.path()).unwrap();
        let first = link_life::read_folded(td.path()).0;
        // Run it again WITHOUT consulting the stamp — the fold must absorb it.
        seed(&conn, td.path()).unwrap();
        seed(&conn, td.path()).unwrap();
        let third = link_life::read_folded(td.path()).0;
        assert_eq!(first, third, "absolute n + max-fold makes a re-seed converge, stamp or no stamp");
        assert_eq!(third.get("C_A>C_B").unwrap().n, 4);
    }

    #[test]
    fn a_newer_walk_already_in_the_store_is_never_ratcheted_down_by_the_seed() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        // The hooks (Slice 4) already recorded a newer, higher count than the DB row carries —
        // e.g. the DB was restored from an older backup. The seed must not undo that.
        link_life::append(td.path(), link_life::Stream::Earned,
            &[link_life::walk_line("C_A", "C_B", "B", 9, "2026-07-27T00:00:00Z")]).unwrap();
        link(&conn, "/a.md", "B", 4, "evidence", "active", 2.6);
        seed(&conn, td.path()).unwrap();
        assert_eq!(link_life::read_folded(td.path()).0.get("C_A>C_B").unwrap().n, 9);
    }

    #[test]
    fn an_archived_row_seeds_a_retire_so_a_rebuild_cannot_resurrect_it() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        link(&conn, "/a.md", "B", 7, "evidence", "archived", 0.0);
        seed(&conn, td.path()).unwrap();
        let e = link_life::read_folded(td.path()).0.get("C_A>C_B").cloned().unwrap();
        assert_eq!(e.status.as_deref(), Some("archived"), "the retirement decision must survive");
        assert_eq!(e.n, 7, "and so must the count it was earned with");
    }

    #[test]
    fn a_derivable_confidence_seeds_no_trust_record() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        // `evidence` at n=3 is exactly the auto-tier — no user judgment to record.
        link(&conn, "/a.md", "B", 3, "evidence", "active", 2.386);
        seed(&conn, td.path()).unwrap();
        let text = std::fs::read_to_string(td.path().join("earned.jsonl")).unwrap();
        assert!(text.contains("\"t\":\"walk\""));
        assert!(!text.contains("\"t\":\"trust\""), "the auto-tier is not a decision");
        // …but a NON-derivable one is.
        note(&conn, "/c.md", "C", "C_C");
        link(&conn, "/c.md", "B", 1, "established", "active", 1.693);
        seed(&conn, td.path()).unwrap();
        let text2 = std::fs::read_to_string(td.path().join("earned.jsonl")).unwrap();
        assert!(text2.contains("\"t\":\"trust\""), "a manual tier that outranks the count IS a decision");
    }

    /// BOSS-FOUND 2026-07-27, RED before the fix. A link whose target NAME is shared by several
    /// notes must produce exactly ONE record — and, because the name cannot identify which note
    /// was meant, that record must key on the NAME, not on a guessed identity. The old LEFT JOIN
    /// emitted one record per same-named note, each asserting a different target: 38 live earned
    /// rows became 44 lines, claiming walks that never happened.
    #[test]
    fn an_ambiguous_target_name_emits_ONE_record_and_refuses_to_guess_an_identity() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        // Three notes share one name — the live `السعودية` shape.
        note(&conn, "/lib1/x.md", "السعودية", "C_X1");
        note(&conn, "/lib2/x.md", "السعودية", "C_X2");
        note(&conn, "/lib3/x.md", "السعودية", "C_X3");
        link(&conn, "/a.md", "السعودية", 1, "hypothesis", "active", 1.693);

        let r = seed(&conn, td.path()).unwrap();
        assert_eq!(r.matched, 1, "ONE earned link row must yield ONE row — never one per same-named note");
        assert_eq!(r.recorded, 1);

        let text = std::fs::read_to_string(td.path().join("earned.jsonl")).unwrap();
        assert_eq!(text.lines().count(), 1, "exactly one line on disk");
        assert!(text.contains(r#""to":"""#), "an ambiguous name must NOT be resolved to a guessed identity");
        for wrong in ["C_X1", "C_X2", "C_X3"] {
            assert!(!text.contains(wrong), "must not assert a target it cannot identify: {wrong}");
        }
        // It still keys — by name, which is exactly what the fold's fallback exists for.
        let (map, _) = link_life::read_folded(td.path());
        assert_eq!(map.len(), 1);
        assert!(map.keys().next().unwrap().starts_with("C_A>~"));
    }

    /// The unambiguous case must still resolve to the identity — the refusal above is scoped to
    /// genuine ambiguity, not applied everywhere out of caution.
    #[test]
    fn a_unique_target_name_still_resolves_to_its_identity() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "Uniquely Named", "C_B");
        link(&conn, "/a.md", "uniquely named", 2, "hypothesis", "active", 2.09);
        seed(&conn, td.path()).unwrap();
        let (map, _) = link_life::read_folded(td.path());
        assert_eq!(map.get("C_A>C_B").unwrap().n, 2);
        // …and the label is the note's real title, not the lowercased link text.
        let text = std::fs::read_to_string(td.path().join("earned.jsonl")).unwrap();
        assert!(text.contains(r#""tn":"Uniquely Named""#));
    }

    /// Seeded lines are marked as DERIVED, because a seeded decision's timestamp is borrowed from
    /// `last_traversed` and is not when the decision happened (Boss-found: a Contested click at
    /// 09:21:25 seeded as 09:13:51). Live-recorded lines carry no marker.
    #[test]
    fn seeded_lines_are_marked_derived_and_stay_valid_json() {
        let (td, conn) = universe();
        note(&conn, "/a.md", "A", "C_A");
        note(&conn, "/b.md", "B", "C_B");
        link(&conn, "/a.md", "B", 5, "contested", "archived", 0.0);
        seed(&conn, td.path()).unwrap();
        let text = std::fs::read_to_string(td.path().join("earned.jsonl")).unwrap();
        for line in text.lines() {
            let v: serde_json::Value = serde_json::from_str(line).expect("still valid JSON");
            assert_eq!(v.get("seed").and_then(|x| x.as_i64()), Some(1), "every seeded line says so");
        }
        // The marker must not disturb the fold.
        let e = link_life::read_folded(td.path()).0.get("C_A>C_B").cloned().unwrap();
        assert_eq!((e.n, e.status.as_deref()), (5, Some("archived")));
        // A live line carries no marker — the two are distinguishable.
        let live = link_life::walk_line("C_A", "C_B", "B", 6, "2026-07-27T10:00:00Z");
        assert!(!live.contains("\"seed\""));
    }

    #[test]
    fn the_seed_writes_the_gitignore_so_the_store_is_never_excluded_with_the_databases() {
        let (td, _conn) = universe();
        crate::link_life::ensure_gitignore(td.path()).unwrap();
        let g = std::fs::read_to_string(td.path().join(".gitignore")).unwrap();
        assert!(g.contains("*.db"));
        // Assert the SEMANTIC, not a substring: "earned" appears legitimately in the file's
        // explanatory comment (which is the point — it tells a reader why the folder must not be
        // excluded wholesale). What matters is that no PATTERN matches a ledger file.
        for ledger in ["earned.jsonl", "earned.snapshot.jsonl", "note-history.jsonl"] {
            assert!(
                !crate::link_life::gitignore_excludes(ledger),
                "the ledger must travel with the user's notes: {ledger}"
            );
        }
    }
}
