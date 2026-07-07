//! MIG-094 — the ONE home for Constellation's named structural-connectivity
//! predicates. PJ-069 Step 1 collapses five drifted "orphan" implementations and
//! four copy-pasted "fragile" ones into a small, graph-theory-grounded vocabulary,
//! each computed once from write-time `note_meta` columns (Rule 8 — no fs-walk, no
//! per-query re-count, no in-memory degree map). Surfaces that ask genuinely
//! different questions keep their own named predicate; surfaces that ask the SAME
//! question stop re-implementing it in ways that silently disagree.
//!
//! The three named concepts (Boss-ruled 2026-07-06, two-named-orphan-concepts):
//!
//! * **UNREFERENCED** — graph "source node" (in-degree 0): nothing points at this
//!   note, though it may link outward (a Map-of-Content). `incoming_count == 0`.
//!   Surfaces layer their OWN substance filter (e.g. `word_count > 20`) on top — the
//!   floor is a per-surface lens, never baked into the shared definition.
//! * **ISOLATED** — graph "isolated vertex" (degree 0): no links in EITHER direction.
//!   A strictly stronger question than UNREFERENCED. `incoming_count == 0 AND
//!   outgoing_count == 0`.
//! * **FRAGILE** — single-point-of-failure / under-corroborated authority (network
//!   science articulation-point / bus-factor; Toulmin weakly-warranted load-bearing
//!   claim): many notes depend on it, but it rests on ≤1 `derives-from` support.
//!   `incoming_count >= 5 AND derives_from_support <= 1`.
//!
//! `incoming_count` is the canonical alias-aware, DISTINCT-source, archived-excluded,
//! structural-lane-excluded column (incoming_links_backfill.rs + triggers).
//! `outgoing_count` is the DISTINCT-active-cognitive-edge column. The derives-from
//! support is read from `outgoing_link_types_json` — which is materialized as
//! `json_group_object(link_type, COUNT(*))` over active cognitive edges
//! (search.rs `outgoing_aggregate_assignments`), so `json_extract(...,
//! '$."derives-from"')` is occurrence-count-equivalent to the legacy
//! `SELECT COUNT(*) FROM note_links WHERE source_path=? AND link_type='derives-from'
//! AND status='active'` subquery (proven by the §2 parity test below).

// The SQL `WHERE` fragments are single-sourced through these builders: each
// concept's threshold / column / JSON-key exists as exactly ONE literal (the whole
// point of this module). Pass "" for bare unqualified columns, or a table alias
// (e.g. "nm") for a scan site that qualifies `note_meta`.

/// Column prefix for an optional table alias ("" → bare column, "nm" → "nm.").
fn col_prefix(alias: &str) -> String {
    if alias.is_empty() { String::new() } else { format!("{alias}.") }
}

/// SQL `WHERE` fragment for UNREFERENCED.
pub fn unreferenced_where(alias: &str) -> String {
    let p = col_prefix(alias);
    format!("{p}incoming_count = 0")
}

/// SQL `WHERE` fragment for ISOLATED.
pub fn isolated_where(alias: &str) -> String {
    let p = col_prefix(alias);
    format!("{p}incoming_count = 0 AND {p}outgoing_count = 0")
}

/// SQL `WHERE` fragment for FRAGILE — derives-from support from the write-time
/// `outgoing_link_types_json` map (no `note_links` subquery).
pub fn fragile_where(alias: &str) -> String {
    let p = col_prefix(alias);
    format!("{p}incoming_count >= 5 AND COALESCE(json_extract({p}outgoing_link_types_json, '$.\"derives-from\"'), 0) <= 1")
}

/// Row-predicate: UNREFERENCED (nothing points here).
#[inline]
pub fn is_unreferenced(incoming_count: i64) -> bool {
    incoming_count == 0
}

/// Row-predicate: ISOLATED (no links either direction).
#[inline]
pub fn is_isolated(incoming_count: i64, outgoing_count: i64) -> bool {
    incoming_count == 0 && outgoing_count == 0
}

/// Row-predicate: FRAGILE (many dependents, ≤1 derives-from support).
#[inline]
pub fn is_fragile(incoming_count: i64, derives_from_support: i64) -> bool {
    incoming_count >= 5 && derives_from_support <= 1
}

/// The active `derives-from` support count, read from a note's
/// `outgoing_link_types_json` (`{"type":count}`). Returns 0 when the note has no
/// `derives-from` edges (the key is absent). Occurrence-count-equivalent to the
/// legacy `COUNT(*)` subquery — see `fragile_where` and the §2 parity test.
pub fn derives_from_support(outgoing_link_types_json: &str) -> i64 {
    // Cheap, allocation-light parse: the map is small ({type:count} over ≤9 cognitive
    // types). serde_json is already a dependency; use it for correctness.
    match serde_json::from_str::<serde_json::Value>(outgoing_link_types_json) {
        Ok(serde_json::Value::Object(map)) => map
            .get("derives-from")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_predicates() {
        // UNREFERENCED: incoming 0, regardless of outgoing.
        assert!(is_unreferenced(0));
        assert!(!is_unreferenced(1));
        // ISOLATED is strictly stronger: needs BOTH zero.
        assert!(is_isolated(0, 0));
        assert!(!is_isolated(0, 3)); // links out → UNREFERENCED but NOT ISOLATED
        assert!(!is_isolated(2, 0));
        // FRAGILE: many dependents, thin support.
        assert!(is_fragile(5, 0));
        assert!(is_fragile(9, 1));
        assert!(!is_fragile(4, 0)); // not enough dependents
        assert!(!is_fragile(9, 2)); // well-supported
    }

    #[test]
    fn derives_extract() {
        assert_eq!(derives_from_support("{}"), 0);
        assert_eq!(derives_from_support(r#"{"supports":3}"#), 0); // key absent
        assert_eq!(derives_from_support(r#"{"derives-from":1}"#), 1);
        assert_eq!(
            derives_from_support(r#"{"supports":2,"derives-from":4,"contradicts":1}"#),
            4
        );
        assert_eq!(derives_from_support("not json"), 0);
    }

    /// §2 build-gate — the JSON-map derives count MUST equal the legacy
    /// `COUNT(*) FROM note_links WHERE source_path=? AND link_type='derives-from'
    /// AND status='active'` subquery, or the FRAGILE `<=1` boundary shifts silently.
    /// Proven here on an in-memory fixture mirroring the real trigger's
    /// `json_group_object(link_type, COUNT(*))` materialization.
    #[test]
    fn fragile_json_equals_subquery() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, incoming_count INTEGER, outgoing_link_types_json TEXT);
             CREATE TABLE note_links (source_path TEXT, link_type TEXT, status TEXT);
             -- A: 6 dependents (set below), rests on ONE derives-from → FRAGILE.
             INSERT INTO note_links VALUES ('/a.md','derives-from','active');
             INSERT INTO note_links VALUES ('/a.md','supports','active');
             INSERT INTO note_links VALUES ('/a.md','derives-from','archived'); -- excluded (not active)
             -- B: two active derives-from → NOT fragile on the support side.
             INSERT INTO note_links VALUES ('/b.md','derives-from','active');
             INSERT INTO note_links VALUES ('/b.md','derives-from','active');
             -- C: zero derives-from.
             INSERT INTO note_links VALUES ('/c.md','contradicts','active');",
        )
        .unwrap();
        // Materialize the JSON map exactly as the real trigger does.
        conn.execute_batch(
            "INSERT INTO note_meta (path, incoming_count, outgoing_link_types_json) VALUES
             ('/a.md', 6, (SELECT COALESCE(json_group_object(link_type, cnt),'{}') FROM
                (SELECT link_type, COUNT(*) cnt FROM note_links WHERE source_path='/a.md' AND status='active' GROUP BY link_type))),
             ('/b.md', 7, (SELECT COALESCE(json_group_object(link_type, cnt),'{}') FROM
                (SELECT link_type, COUNT(*) cnt FROM note_links WHERE source_path='/b.md' AND status='active' GROUP BY link_type))),
             ('/c.md', 8, (SELECT COALESCE(json_group_object(link_type, cnt),'{}') FROM
                (SELECT link_type, COUNT(*) cnt FROM note_links WHERE source_path='/c.md' AND status='active' GROUP BY link_type)));",
        )
        .unwrap();

        // For every note: FRAGILE via the shared JSON helper == FRAGILE via the legacy subquery.
        let mut stmt = conn
            .prepare("SELECT path, incoming_count, outgoing_link_types_json FROM note_meta ORDER BY path")
            .unwrap();
        let rows: Vec<(String, i64, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for (path, inc, json) in rows {
            let subquery: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM note_links WHERE source_path=?1 AND link_type='derives-from' AND status='active'",
                    [&path],
                    |r| r.get(0),
                )
                .unwrap();
            let via_helper = is_fragile(inc, derives_from_support(&json));
            let via_subquery = inc >= 5 && subquery <= 1;
            assert_eq!(
                via_helper, via_subquery,
                "FRAGILE parity mismatch for {path}: helper={via_helper} subquery={via_subquery} (json={json}, subq_count={subquery})"
            );
        }
    }
}
