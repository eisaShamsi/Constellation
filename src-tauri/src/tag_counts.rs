//! MIG-079 §C.1 — write-time `tag_counts` summary (Boot WTD, CLAUDE.md Rule 8).
//!
//! The boot graph snapshot used to aggregate every note's `tags_json` on every
//! launch (`cache::read_tags`, a full `note_meta` scan — **5.6 s** measured on the
//! 7,653-note live universe, because each row drags the inline `body_text`). That
//! is a read-time recompute of a view that only changes when a note's tags change.
//!
//! This module maintains a persisted `tag_counts(tag, n)` summary that is current
//! at all times, so the boot read is an O(distinct-tags) table lookup (~ms):
//!
//! - **Write-time ±delta** — `index_note` (the SOLE writer of `note_meta.tags_json`)
//!   and `reindex_delete_note` call [`apply_delta`] inside their existing
//!   transaction, applying `new − old` per tag. O(tags-on-note); a save that does
//!   not touch tags is a no-op (the `old == new` fast path).
//! - **Gated on `schema_versions.tag_counts`** — the delta and the read-flip both
//!   activate only once the table is stamped. Before that, the legacy live scan is
//!   the source of truth (zero-risk rollout; an un-upgraded attached cUniverse
//!   simply keeps scanning).
//! - **One-shot atomic backfill** — [`maybe_schedule`] builds the whole table from
//!   `note_meta` in a SINGLE `json_each` aggregate inside ONE `IMMEDIATE`
//!   transaction, then stamps, then commits. Atomicity is deliberate: a counter is
//!   an *additive* aggregate, so a batched build racing live ±deltas could
//!   double-count. Building + stamping atomically means a save either commits
//!   before the backfill (its delta is gated-off, the aggregate sees the new
//!   `note_meta`) or after (table stamped, its delta applies) — both converge.
//!   Tags have no giant-row problem (unlike `note_body`'s 128 MB outlier), so the
//!   aggregate is one cheap statement; the ~6 s write-lock hold is one-time,
//!   background, post-paint, and WAL keeps boot READS unblocked throughout.
//! - **`reconcile_filesystem` recomputes authoritatively** — after a full re-index,
//!   the table is rebuilt from `note_meta` (the periodic self-heal), so any drift
//!   from any source is corrected.
//!
//! Multiset semantics MATCH `cache::read_tags_in_schema` EXACTLY: each occurrence
//! counts (duplicates included), empty strings are skipped. Proven byte-identical
//! to the live `read_tags` aggregate by the rehearsal harness (lab/tag-counts/).

use rusqlite::{params, Connection, TransactionBehavior};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri::Manager;

use crate::search::SearchState;

/// Bump to force a one-time rebuild on existing DBs (e.g. if the multiset
/// semantics ever change). Parallel to `LINKS_OUTGOING_SCHEMA_VERSION`.
pub(crate) const SCHEMA_VERSION: i64 = 1;

/// The shared aggregate SQL — `note_meta` → `tag_counts`. Used by BOTH the
/// backfill and `reconcile_filesystem`'s self-heal so the two population paths
/// can never drift (the links_backfill `recompute_range` pattern). The caller
/// owns the transaction.
///
/// `json_each` over a guard (`json_valid` ? tags_json : '[]') so a malformed row
/// contributes nothing — matching `read_tags`' `unwrap_or_default()`. `type='text'`
/// counts only string elements (Constellation always writes clean string arrays;
/// the guard is defensive). `value <> ''` mirrors `read_tags`' empty-skip.
pub(crate) fn recompute_all_in(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM tag_counts", [])?;
    let n = conn.execute(
        "INSERT INTO tag_counts(tag, n)
           SELECT je.value, COUNT(*)
           FROM note_meta,
                json_each(CASE WHEN json_valid(tags_json) THEN tags_json ELSE '[]' END) je
           WHERE je.type = 'text' AND je.value <> ''
           GROUP BY je.value",
        [],
    )?;
    Ok(n)
}

/// Parse a `tags_json` array into a per-tag occurrence multiset, EXACTLY as
/// `cache::read_tags_in_schema` counts: skip empty, count each occurrence
/// (duplicates included), malformed → empty.
pub(crate) fn tag_multiset(tags_json: &str) -> HashMap<String, i64> {
    let mut m: HashMap<String, i64> = HashMap::new();
    let arr: Vec<String> = serde_json::from_str(tags_json).unwrap_or_default();
    for t in arr {
        if !t.is_empty() {
            *m.entry(t).or_insert(0) += 1;
        }
    }
    m
}

/// Apply the write-time ±delta for one note: `tag_counts.n += new − old` per tag,
/// pruning any tag whose count falls to ≤ 0. Caller runs this INSIDE the note's
/// transaction (so the counter move is atomic with the `note_meta` write).
///
/// O(distinct tags touched). The `old == new` fast path makes a body-only save
/// (the common case) a no-op.
pub(crate) fn apply_delta(conn: &Connection, old_json: &str, new_json: &str) -> rusqlite::Result<()> {
    let old = tag_multiset(old_json);
    let new = tag_multiset(new_json);
    if old == new {
        return Ok(()); // tags unchanged — the overwhelmingly common save
    }
    let mut keys: HashSet<&String> = HashSet::new();
    keys.extend(old.keys());
    keys.extend(new.keys());

    let mut up = conn.prepare_cached(
        "INSERT INTO tag_counts(tag, n) VALUES (?1, ?2)
         ON CONFLICT(tag) DO UPDATE SET n = n + excluded.n",
    )?;
    let mut prune = conn.prepare_cached("DELETE FROM tag_counts WHERE tag = ?1 AND n <= 0")?;

    for k in keys {
        let delta = new.get(k).copied().unwrap_or(0) - old.get(k).copied().unwrap_or(0);
        if delta == 0 {
            continue;
        }
        up.execute(params![k, delta])?;
        if delta < 0 {
            prune.execute(params![k])?;
        }
    }
    Ok(())
}

/// True once the table is stamped current (main schema). The ±delta + read-flip
/// gate on this.
pub(crate) fn is_stamped(conn: &Connection) -> bool {
    is_stamped_in_schema(conn, "main")
}

/// Schema-qualified stamp check — the federated read-flip calls this per attached
/// universe. Any error (an old cUniverse with no `tag_counts` / no row) → false →
/// that schema falls back to the live scan.
pub(crate) fn is_stamped_in_schema(conn: &Connection, schema: &str) -> bool {
    let sql = format!(
        "SELECT version FROM {}.schema_versions WHERE module = 'tag_counts'",
        schema
    );
    conn.query_row(&sql, [], |r| r.get::<_, i64>(0)).unwrap_or(0) >= SCHEMA_VERSION
}

/// Schedule the one-shot backfill on a background thread. Returns immediately.
/// Silent no-op once stamped. Mirrors `note_body_backfill::maybe_schedule`.
pub fn maybe_schedule(app: tauri::AppHandle) {
    let state = app.state::<SearchState>();
    let needs_run = {
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(conn) = guard.as_ref() else {
            return;
        };
        !is_stamped(conn)
    };
    if !needs_run {
        return;
    }

    let app_bg = app.clone();
    std::thread::spawn(move || match run(&app_bg) {
        Ok(n) => diag(&app_bg, &format!("[tag_counts_backfill] completed: {} distinct tags", n)),
        Err(e) => diag(&app_bg, &format!("[tag_counts_backfill] FAILED (non-fatal): {}", e)),
    });
}

/// Build `tag_counts` from `note_meta` and stamp — atomically, on a DEDICATED
/// connection (the proven `reconcile_filesystem` walk_conn pattern), so the
/// IMMEDIATE write-lock hold never touches `state.db`'s mutex and WAL keeps boot
/// readers unblocked. Aggregate + stamp share ONE transaction → no race.
fn run(app: &tauri::AppHandle) -> Result<usize, String> {
    let path = crate::search::db_path(app)?;
    let mut conn = Connection::open(&path)
        .map_err(|e| format!("open tag_counts conn: {}", e))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| format!("pragma: {}", e))?;
    conn.busy_timeout(Duration::from_secs(30))
        .map_err(|e| format!("busy_timeout: {}", e))?;

    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| format!("begin immediate: {}", e))?;
    let n = recompute_all_in(&tx).map_err(|e| format!("recompute: {}", e))?;
    tx.execute(
        "INSERT OR REPLACE INTO schema_versions (module, version, updated_at)
         VALUES ('tag_counts', ?1, strftime('%s','now'))",
        params![SCHEMA_VERSION],
    )
    .map_err(|e| format!("stamp: {}", e))?;
    tx.commit().map_err(|e| format!("commit: {}", e))?;
    Ok(n)
}

fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

#[cfg(test)]
mod tests {
    //! Pins the three population paths against each other on the bundled SQLite:
    //! the per-note ±delta ([`apply_delta`]), the bulk aggregate ([`recompute_all_in`]),
    //! and the legacy multiset count ([`tag_multiset`] — the `read_tags` semantics).
    //! If any two could drift, the boot count would diverge from what the user sees.
    use super::*;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, tags_json TEXT NOT NULL DEFAULT '[]', body_text TEXT NOT NULL DEFAULT '');
             CREATE TABLE tag_counts (tag TEXT PRIMARY KEY, n INTEGER NOT NULL DEFAULT 0);
             CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);",
        )
        .unwrap();
        conn
    }

    fn counts(conn: &Connection) -> HashMap<String, i64> {
        let mut stmt = conn.prepare("SELECT tag, n FROM tag_counts WHERE n > 0").unwrap();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))).unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    /// The legacy `read_tags` aggregate, computed in Rust over note_meta — the
    /// ground truth the table must equal.
    fn live_aggregate(conn: &Connection) -> HashMap<String, i64> {
        let mut stmt = conn.prepare("SELECT tags_json FROM note_meta").unwrap();
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).unwrap();
        let mut m: HashMap<String, i64> = HashMap::new();
        for tj in rows.map(|r| r.unwrap()) {
            for (k, v) in tag_multiset(&tj) {
                *m.entry(k).or_insert(0) += v;
            }
        }
        m
    }

    /// PJ-207 §4 — the covering index `idx_note_meta_tags(path, tags_json)` is a PURE
    /// COST change: it must not move a single count.
    ///
    /// Why an index needs a semantics test at all: `recompute_all_in` is a
    /// `json_each` join with a `GROUP BY`, and the index changes which rows the
    /// planner walks and in what order. The counts must be byte-identical, which is
    /// the plan's verification clause stated as an assertion.
    ///
    /// Measured on a byte copy of the real 1.89 GB universe before this shipped:
    /// `EXPLAIN QUERY PLAN` becomes `SCAN note_meta USING COVERING INDEX
    /// idx_note_meta_tags`, because the query wants `tags_json` alone while a table
    /// scan drags 270.3 MB of row payload (259.5 MB of it `body_text`) against the
    /// index's 1.6 MB — 167x fewer bytes, which is what made the rebuild 13.0 s cold.
    #[test]
    fn the_covering_index_changes_the_plan_and_not_one_count() {
        let conn = db();
        // A corpus with every shape the aggregate has to handle: duplicates within a
        // note, a tag shared across notes, an empty list, a malformed value, and a
        // non-ASCII tag (the Boss's universe is bilingual).
        for (p, t) in [
            ("/a.md", r#"["alpha","beta","alpha"]"#),
            ("/b.md", r#"["beta","معرفة"]"#),
            ("/c.md", "[]"),
            ("/d.md", "not json"),
            ("/e.md", r#"["معرفة","gamma"]"#),
        ] {
            conn.execute("INSERT INTO note_meta(path, tags_json) VALUES (?1, ?2)", params![p, t])
                .unwrap();
        }

        // Bulk filler so the planner has a corpus worth indexing. On a 5-row table a
        // scan is genuinely cheaper and SQLite will rightly refuse the index — asserting
        // plan selection at that scale would pin the environment, not the fix. Each
        // filler row also carries a fat `body_text`, which is the whole reason the table
        // scan is expensive in production.
        {
            let mut st = conn
                .prepare("INSERT INTO note_meta(path, tags_json, body_text) VALUES (?1, ?2, ?3)")
                .unwrap();
            let filler = "x".repeat(2_000);
            for i in 0..2_000 {
                st.execute(params![
                    format!("/bulk/{i}.md"),
                    format!(r#"["t{}","shared"]"#, i % 50),
                    filler,
                ])
                .unwrap();
            }
        }

        recompute_all_in(&conn).unwrap();
        let without_index = counts(&conn);
        assert_eq!(without_index, live_aggregate(&conn), "baseline must match ground truth");

        conn.execute_batch(
            "CREATE INDEX idx_note_meta_tags ON note_meta(path, tags_json); ANALYZE;",
        )
        .unwrap();

        recompute_all_in(&conn).unwrap();
        assert_eq!(
            counts(&conn),
            without_index,
            "the covering index is a cost change only — not one count may move",
        );

        // NOTE, stated rather than faked: this test does NOT assert plan selection.
        // `note_meta` here has three columns, so `(path, tags_json)` is very nearly the
        // whole table and SQLite correctly declines the index — it buys nothing at this
        // shape. Plan selection is a property of the REAL 30-column table, and is
        // asserted where it can actually be observed:
        // `the_covering_index_is_chosen_on_a_real_corpus` below. Asserting it here
        // would pin the fixture, not the fix.
    }

    /// PJ-207 §4 — the gate the plan actually set: **the planner must choose the
    /// covering index.** An index nobody uses buys nothing.
    ///
    /// Ignored by default, like `rehearse_against_live_copy` above and for the same
    /// reason: it needs a real corpus. The unit test above cannot show this, because a
    /// three-column fixture makes the index nearly the whole table.
    ///
    /// Run it against a **byte copy** of a real universe (never the live file):
    /// ```text
    /// TAG_COUNTS_REHEARSAL_DB=E:/_pj207-scratch/search.db \
    ///   cargo test --lib the_covering_index_is_chosen_on_a_real_corpus -- --ignored --nocapture
    /// ```
    /// Verified 2026-08-03 on a copy of the 1.89 GB / 7,824-note universe:
    /// `SCAN note_meta USING COVERING INDEX idx_note_meta_tags`.
    #[test]
    #[ignore = "rehearsal — needs a real-corpus DB copy via TAG_COUNTS_REHEARSAL_DB"]
    fn the_covering_index_is_chosen_on_a_real_corpus() {
        let db = std::env::var("TAG_COUNTS_REHEARSAL_DB").expect("set TAG_COUNTS_REHEARSAL_DB");
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_note_meta_tags ON note_meta(path, tags_json);",
        )
        .unwrap();

        let plan: String = {
            let mut st = conn
                .prepare(
                    "EXPLAIN QUERY PLAN SELECT je.value, COUNT(*) FROM note_meta, \
                     json_each(CASE WHEN json_valid(tags_json) THEN tags_json ELSE '[]' END) je \
                     WHERE je.type = 'text' AND je.value <> '' GROUP BY je.value",
                )
                .unwrap();
            let rows = st.query_map([], |r| r.get::<_, String>(3)).unwrap();
            rows.map(|r| r.unwrap()).collect::<Vec<_>>().join(" | ")
        };
        eprintln!("[rehearsal] plan: {plan}");
        assert!(
            plan.contains("idx_note_meta_tags"),
            "the planner must use the covering index — without it §4 buys nothing; plan was: {plan}",
        );

        let t = std::time::Instant::now();
        recompute_all_in(&conn).unwrap();
        eprintln!("[rehearsal] recompute_all_in with the index: {:?}", t.elapsed());
    }

    #[test]
    fn multiset_matches_read_tags_semantics() {
        // empty skipped; duplicates counted; malformed → empty.
        assert_eq!(tag_multiset(r#"["a","b","a"]"#), HashMap::from([("a".into(), 2), ("b".into(), 1)]));
        assert_eq!(tag_multiset(r#"["a","","a"]"#), HashMap::from([("a".into(), 2)]));
        assert_eq!(tag_multiset("not json"), HashMap::new());
        assert_eq!(tag_multiset("[]"), HashMap::new());
        // non-string-element array → serde drops the whole array (Vec<String> fails).
        assert_eq!(tag_multiset(r#"["a", 1]"#), HashMap::new());
    }

    #[test]
    fn delta_add_remove_rename_and_prune() {
        let conn = db();
        // New note: [] -> [a,b,a]
        apply_delta(&conn, "[]", r#"["a","b","a"]"#).unwrap();
        assert_eq!(counts(&conn), HashMap::from([("a".into(), 2), ("b".into(), 1)]));
        // Edit: drop one 'a', add 'c': [a,b,a] -> [a,b,c]
        apply_delta(&conn, r#"["a","b","a"]"#, r#"["a","b","c"]"#).unwrap();
        assert_eq!(counts(&conn), HashMap::from([("a".into(), 1), ("b".into(), 1), ("c".into(), 1)]));
        // Remove 'b' entirely: it must be PRUNED (not left at 0).
        apply_delta(&conn, r#"["a","b","c"]"#, r#"["a","c"]"#).unwrap();
        assert_eq!(counts(&conn), HashMap::from([("a".into(), 1), ("c".into(), 1)]));
        let zero: i64 = conn
            .query_row("SELECT COUNT(*) FROM tag_counts WHERE n <= 0", [], |r| r.get(0))
            .unwrap();
        assert_eq!(zero, 0, "tags that fall to zero are pruned, never left as dead rows");
    }

    #[test]
    fn delta_no_op_when_tags_unchanged() {
        let conn = db();
        apply_delta(&conn, "[]", r#"["x"]"#).unwrap();
        let before = counts(&conn);
        // Same tags (body-only edit) — must touch nothing.
        apply_delta(&conn, r#"["x"]"#, r#"["x"]"#).unwrap();
        assert_eq!(counts(&conn), before);
    }

    #[test]
    fn backfill_equals_sum_of_deltas_and_is_idempotent() {
        // Seed a corpus with duplicates, empties, a malformed row, and unicode.
        let conn = db();
        for (p, t) in [
            ("/1.md", r#"["philosophy","فلسفة","philosophy"]"#),
            ("/2.md", r#"["فلسفة",""]"#),
            ("/3.md", r#"[]"#),
            ("/4.md", r#"["living-people"]"#),
        ] {
            conn.execute("INSERT INTO note_meta(path, tags_json) VALUES (?1, ?2)", params![p, t]).unwrap();
        }

        // Path 1: bulk aggregate.
        recompute_all_in(&conn).unwrap();
        let by_aggregate = counts(&conn);

        // It must equal the legacy live scan EXACTLY.
        assert_eq!(by_aggregate, live_aggregate(&conn));
        assert_eq!(
            by_aggregate,
            HashMap::from([("philosophy".into(), 2), ("فلسفة".into(), 2), ("living-people".into(), 1)])
        );

        // Path 2: build the SAME table purely from per-note deltas (from empty).
        conn.execute("DELETE FROM tag_counts", []).unwrap();
        let mut stmt = conn.prepare("SELECT tags_json FROM note_meta").unwrap();
        let all: Vec<String> = stmt.query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();
        for tj in &all {
            apply_delta(&conn, "[]", tj).unwrap();
        }
        assert_eq!(counts(&conn), by_aggregate, "delta-from-empty == bulk aggregate");

        // Idempotent: a second aggregate over the same corpus yields the same table.
        recompute_all_in(&conn).unwrap();
        assert_eq!(counts(&conn), by_aggregate);
    }

    /// MIG-079 §C.1 rehearsal (mandatory item a) — run the REAL `recompute_all_in`
    /// against a COPY of the live universe DB and assert `tag_counts` equals the
    /// live `read_tags` aggregate EXACTLY (the serde target dumped by
    /// lab/tag-counts/analyze-live-tags.py). Proves the production aggregate SQL
    /// is byte-identical to the read path it replaces, on the real 7,653-note
    /// corpus — not a re-typed Python approximation.
    ///
    /// Run:
    ///   TAG_COUNTS_REHEARSAL_DB="E:\Backups\Constellation\rehearsal\search-copy.db" \
    ///   TAG_COUNTS_TARGET_JSON="..\lab\tag-counts\live-read-tags-target.json" \
    ///   cargo test --lib tag_counts::tests::rehearse_against_live_copy -- --ignored --nocapture
    #[test]
    #[ignore = "rehearsal — needs a live-DB copy via TAG_COUNTS_REHEARSAL_DB"]
    fn rehearse_against_live_copy() {
        let db = std::env::var("TAG_COUNTS_REHEARSAL_DB").expect("set TAG_COUNTS_REHEARSAL_DB");
        let target = std::env::var("TAG_COUNTS_TARGET_JSON").expect("set TAG_COUNTS_TARGET_JSON");

        let conn = Connection::open(&db).unwrap();
        // init_db creates this on a real boot; the copy predates the migration.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tag_counts (tag TEXT PRIMARY KEY, n INTEGER NOT NULL DEFAULT 0);",
        )
        .unwrap();

        let t = std::time::Instant::now();
        let distinct = recompute_all_in(&conn).unwrap();
        eprintln!("[rehearsal] recompute_all_in: {} distinct tags in {:?}", distinct, t.elapsed());

        let built = counts(&conn);
        let expected: HashMap<String, i64> =
            serde_json::from_str(&std::fs::read_to_string(&target).unwrap()).unwrap();

        // Exact set + count equality, with readable diffs on failure.
        let only_built: Vec<_> = built.keys().filter(|k| !expected.contains_key(*k)).take(10).collect();
        let only_expected: Vec<_> = expected.keys().filter(|k| !built.contains_key(*k)).take(10).collect();
        let count_diffs: Vec<_> = built
            .iter()
            .filter(|(k, v)| expected.get(*k).map_or(false, |e| e != *v))
            .take(10)
            .map(|(k, v)| (k.clone(), *v, expected[k]))
            .collect();
        eprintln!(
            "[rehearsal] built distinct={} occ={}  expected distinct={} occ={}",
            built.len(),
            built.values().sum::<i64>(),
            expected.len(),
            expected.values().sum::<i64>(),
        );
        eprintln!("[rehearsal] only-in-built(≤10): {:?}", only_built);
        eprintln!("[rehearsal] only-in-expected(≤10): {:?}", only_expected);
        eprintln!("[rehearsal] count-diffs(≤10): {:?}", count_diffs);

        assert_eq!(built, expected, "tag_counts must equal the live read_tags aggregate EXACTLY");
        eprintln!("[rehearsal] PASS — tag_counts == live read_tags aggregate, byte-for-byte");
    }
}
