//! MIG-092 §3 — Collections' hydration read (repurposed from the MIG-090
//! Workbench, `collections_hydrate`).
//!
//! ONE batched, read-only lookup for a collection's NOTE members: membership
//! keys (cid_cn preferred; path fallback for notes that lack a cid) → the live
//! row facts, all from write-time-maintained indexes (`note_meta` LEFT JOIN
//! `review_schedule`). Zero filesystem access; O(set size) via the partial
//! UNIQUE cid index + the path PK. `collections.json` stores membership ONLY —
//! this command is where every displayed fact is re-read on each surface
//! open / liveness event (the MIG-077 B1 stale-snapshot cure).
//!
//! Folder / saved-search members (unified from the former Bookmarks) are NOT
//! hydrated here — they carry no `note_meta` row; the frontend renders them
//! from their inline stored facts and never sends them to this command.
//!
//! Missing members (a key with no row — the note was deleted externally or
//! its universe detached) simply return no row; the frontend diffs sent keys
//! against returned rows and shows the honest "missing" standing.

use rusqlite::Connection;
use serde::Serialize;
use tauri::Manager;

/// One hydrated row. Review fields are honest empties (`None`/`false`) when
/// the MIG-083 review schema hasn't been stamped yet on this universe — the
/// same gate the Reviewer itself uses (`review::is_stamped`).
#[derive(Debug, Clone, Serialize)]
pub struct CollectionRow {
    /// The key the caller sent for this row (a cid_cn or a path) — lets the
    /// frontend re-associate rows with its membership items directly.
    pub key: String,
    pub path: String,
    pub cid_cn: String,
    pub name: String,
    pub library_name: String,
    pub modified: i64,
    pub word_count: i64,
    /// The note's own declared stage (frontmatter), or None when unstaged —
    /// the [forming] chip's honest source (concept: the user's declaration).
    pub stage: Option<String>,
    pub incoming_count: i64,
    pub outgoing_count: i64,
    /// The write-time `{type: count}` JSONs (MIG-066/079) — the [contested]
    /// chip reads `contradicts` out of these client-side.
    pub incoming_link_types_json: String,
    pub outgoing_link_types_json: String,
    pub review_reason: Option<String>,
    pub review_due: bool,
    pub snoozed: bool,
}

/// The SELECT shared by both key kinds. `{key_expr}` is the column the sent
/// key equals (cid_cn or path); the review LEFT JOIN is always safe — the
/// table exists unconditionally (CREATE IF NOT EXISTS in init_db), rows are
/// simply absent until the MIG-083 backfill stamps.
fn hydrate_sql(key_col: &str, placeholders: &str) -> String {
    format!(
        "SELECT nm.{key_col} AS key, nm.path, nm.cid_cn, nm.name, nm.library_name,
                nm.modified, nm.word_count,
                json_extract(nm.properties_json, '$.\"stage\"') AS stage,
                nm.incoming_count, nm.outgoing_count,
                nm.incoming_link_types_json, nm.outgoing_link_types_json,
                rs.reason, rs.due_days, rs.snoozed_until
         FROM note_meta nm
         LEFT JOIN review_schedule rs ON rs.path = nm.path
         WHERE nm.{key_col} IN ({placeholders}){cid_guard}",
        key_col = key_col,
        placeholders = placeholders,
        // The cid unique index is PARTIAL (WHERE cid_cn != '', the 2026-07-05
        // collision fix) — the explicit predicate is required for the planner
        // to use it with bound parameters.
        cid_guard = if key_col == "cid_cn" { " AND nm.cid_cn != ''" } else { "" },
    )
}

fn query_rows(
    conn: &Connection,
    key_col: &str,
    keys: &[String],
    review_stamped: bool,
    today_days: i64,
    today: &str,
    out: &mut Vec<CollectionRow>,
) -> Result<(), String> {
    if keys.is_empty() {
        return Ok(());
    }
    let placeholders = std::iter::repeat("?")
        .take(keys.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = hydrate_sql(key_col, &placeholders);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("collections_hydrate prepare ({}): {}", key_col, e))?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(keys.iter()), |r| {
            Ok((
                r.get::<_, String>(0)?,          // key
                r.get::<_, String>(1)?,          // path
                r.get::<_, String>(2)?,          // cid_cn
                r.get::<_, String>(3)?,          // name
                r.get::<_, String>(4)?,          // library_name
                r.get::<_, i64>(5)?,             // modified
                r.get::<_, i64>(6)?,             // word_count
                r.get::<_, Option<String>>(7)?,  // stage
                r.get::<_, i64>(8)?,             // incoming_count
                r.get::<_, i64>(9)?,             // outgoing_count
                r.get::<_, Option<String>>(10)?, // incoming_link_types_json
                r.get::<_, Option<String>>(11)?, // outgoing_link_types_json
                r.get::<_, Option<String>>(12)?, // review reason
                r.get::<_, Option<i64>>(13)?,    // due_days
                r.get::<_, Option<String>>(14)?, // snoozed_until
            ))
        })
        .map_err(|e| format!("collections_hydrate query ({}): {}", key_col, e))?;

    for row in rows {
        let (
            key, path, cid_cn, name, library_name, modified, word_count, stage,
            incoming_count, outgoing_count, in_types, out_types, reason, due_days,
            snoozed_until,
        ) = row.map_err(|e| format!("collections_hydrate row: {}", e))?;
        // The Reviewer's exact due predicate (review.rs Lens 1): due_days has
        // arrived AND the note isn't snoozed past today. Honest false when the
        // review schema isn't stamped (rows absent → due_days None anyway).
        let snoozed = matches!(&snoozed_until, Some(s) if s.as_str() > today);
        let review_due = review_stamped
            && !snoozed
            && matches!(due_days, Some(d) if d <= today_days);
        out.push(CollectionRow {
            key,
            path,
            cid_cn,
            name,
            library_name,
            modified,
            word_count,
            stage,
            incoming_count,
            outgoing_count,
            incoming_link_types_json: in_types.unwrap_or_default(),
            outgoing_link_types_json: out_types.unwrap_or_default(),
            review_reason: if review_stamped { reason } else { None },
            review_due,
            snoozed,
        });
    }
    Ok(())
}

/// MIG-092 §3 — hydrate a collection's note members. `(async)` per the
/// near-universal rule (touches `state.db`); ms-class (indexed, O(set), capped)
/// so the writer lock is held only briefly — the same access shape as
/// `get_due_notes`. Returns whatever resolves; the frontend derives "missing"
/// from the diff. Folder/search members are never sent here.
#[tauri::command(async)]
pub fn collections_hydrate(
    app: tauri::AppHandle,
    cids: Vec<String>,
    paths: Vec<String>,
) -> Result<Vec<CollectionRow>, String> {
    if cids.len() + paths.len() > 512 {
        return Err("collections_hydrate: key cap exceeded (512)".to_string());
    }
    let today = crate::review::today_str();
    let today_days = crate::review::date_to_days(&today);

    let mut out: Vec<CollectionRow> = Vec::new();
    if let Some(state) = app.try_state::<crate::search::SearchState>() {
        if let Ok(guard) = state.db.lock() {
            if let Some(conn) = guard.as_ref() {
                let stamped = crate::review::is_stamped(conn);
                query_rows(conn, "cid_cn", &cids, stamped, today_days, &today, &mut out)?;
                query_rows(conn, "path", &paths, stamped, today_days, &today, &mut out)?;
            }
        }
    }
    // Conn absent (init still running) → empty; the surface re-hydrates on the
    // next open / liveness event — the get_due_notes graceful shape.
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                name TEXT NOT NULL DEFAULT '',
                library_name TEXT NOT NULL DEFAULT '',
                modified INTEGER NOT NULL DEFAULT 0,
                word_count INTEGER NOT NULL DEFAULT 0,
                properties_json TEXT NOT NULL DEFAULT '{}',
                cid_cn TEXT NOT NULL DEFAULT '',
                incoming_count INTEGER NOT NULL DEFAULT 0,
                outgoing_count INTEGER NOT NULL DEFAULT 0,
                incoming_link_types_json TEXT NOT NULL DEFAULT '{}',
                outgoing_link_types_json TEXT NOT NULL DEFAULT '{}'
             );
             CREATE UNIQUE INDEX idx_note_meta_cid_cn ON note_meta(cid_cn) WHERE cid_cn != '';
             CREATE TABLE review_schedule (
                path TEXT PRIMARY KEY,
                reason TEXT NOT NULL,
                due_days INTEGER NOT NULL,
                is_checkpoint INTEGER NOT NULL DEFAULT 0,
                last_reviewed TEXT,
                stratum INTEGER NOT NULL DEFAULT 0,
                interval INTEGER NOT NULL DEFAULT 0,
                snoozed_until TEXT
             );",
        )
        .unwrap();
        conn
    }

    fn seed(conn: &Connection, path: &str, cid: &str, stage: Option<&str>) {
        let props = match stage {
            Some(s) => format!("{{\"stage\": \"{}\"}}", s),
            None => "{}".to_string(),
        };
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, word_count, properties_json, cid_cn)
             VALUES (?1, ?2, 'Lib', 100, 10, ?3, ?4)",
            rusqlite::params![path, path, props, cid],
        )
        .unwrap();
    }

    #[test]
    fn resolves_by_cid_and_path_and_reports_missing_by_omission() {
        let conn = test_db();
        seed(&conn, "a.md", "CID_A", Some("growth"));
        seed(&conn, "b.md", "", None); // external note, no cid yet
        let today = crate::review::today_str();
        let today_days = crate::review::date_to_days(&today);

        let mut out = Vec::new();
        query_rows(&conn, "cid_cn", &vec!["CID_A".into(), "CID_GONE".into()], true, today_days, &today, &mut out).unwrap();
        query_rows(&conn, "path", &vec!["b.md".into()], true, today_days, &today, &mut out).unwrap();

        assert_eq!(out.len(), 2, "the unknown cid returns no row (missing by omission)");
        assert_eq!(out[0].key, "CID_A");
        assert_eq!(out[0].stage.as_deref(), Some("growth"));
        assert_eq!(out[1].path, "b.md");
        assert!(out[1].stage.is_none());
    }

    #[test]
    fn due_predicate_matches_reviewer_semantics_and_unstamped_is_honest() {
        let conn = test_db();
        seed(&conn, "due.md", "CID_D", None);
        seed(&conn, "snoozed.md", "CID_S", None);
        let today = crate::review::today_str();
        let today_days = crate::review::date_to_days(&today);
        conn.execute(
            "INSERT INTO review_schedule (path, reason, due_days) VALUES ('due.md', 'interval_due', ?1)",
            rusqlite::params![today_days - 1],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO review_schedule (path, reason, due_days, snoozed_until) VALUES ('snoozed.md', 'interval_due', ?1, '9999-12-31')",
            rusqlite::params![today_days - 1],
        )
        .unwrap();

        let mut stamped = Vec::new();
        query_rows(&conn, "cid_cn", &vec!["CID_D".into(), "CID_S".into()], true, today_days, &today, &mut stamped).unwrap();
        let due = stamped.iter().find(|r| r.path == "due.md").unwrap();
        let snz = stamped.iter().find(|r| r.path == "snoozed.md").unwrap();
        assert!(due.review_due && due.review_reason.as_deref() == Some("interval_due"));
        assert!(!snz.review_due && snz.snoozed, "snoozed-past-today is not due");

        let mut unstamped = Vec::new();
        query_rows(&conn, "cid_cn", &vec!["CID_D".into()], false, today_days, &today, &mut unstamped).unwrap();
        assert!(
            !unstamped[0].review_due && unstamped[0].review_reason.is_none(),
            "unstamped review schema → honest empties, never a wrong chip"
        );
    }
}
