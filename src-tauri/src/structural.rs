//! PJ-065 §6 — lazy READ APIs for the structural (parent / table-of-contents) lane.
//!
//! Only DIRECT edges are stored (`note_links` rows, two inverse faces: `contains`
//! parent→child carrying the `seq` order, and `parent` child→parent). Ancestors
//! (the breadcrumb) and descendants (the outline) are computed **on read** here —
//! never stored (Rule 8 / the LL-XXX OOM lesson) — lazily, only on a user gesture
//! (the TOC panel, §7). Reads use the cached read-only connection; they never block
//! the writer and never touch note body content (Editor-Surface Gate).
//!
//! **Single-parent (D5) + acyclicity (D6) are resolved HERE, deterministically** —
//! NOT by index-order-dependent write-time rejection. A note's structural parent is:
//! (1) its OWN `parent:` declaration (authoritative), else (2) the smallest-path
//! `contains:` claim targeting it. A visited-set on resolved PATHS breaks any cycle
//! so a malformed (cyclic / multi-parent) graph renders cleanly and never hangs.
//!
//! MVP scope: structural edges live within one universe (the D5 ruling). Reads use
//! the primary universe connection; cUniverse federation of the spine is a follow-up.

use rusqlite::Connection;
use serde::Serialize;
use tauri::Manager;

/// One resolved structural neighbour (a child, or a breadcrumb ancestor).
#[derive(Serialize, Clone)]
pub struct StructuralNode {
    pub path: String,
    pub name: String,
    /// Order under its parent (the `contains` face). `None` = unordered (a child
    /// declared only via its own `parent:`, or a breadcrumb ancestor).
    pub seq: Option<i64>,
    /// True when THIS node appears under a parent that is NOT its resolved single
    /// parent — i.e. an overruled `contains:` claim (the D5 single-parent guard).
    /// The real parent won; this listing is surfaced (not silently dropped) so the
    /// user can fix the conflicting frontmatter. `false` for a real child.
    #[serde(default)]
    pub contested: bool,
    /// When `contested`, the name of the note that actually owns this child (its
    /// resolved parent) — for the panel's "belongs to X" notice.
    #[serde(default)]
    pub contested_owner: Option<String>,
}

/// A node in the descendant outline tree.
#[derive(Serialize, Clone)]
pub struct StructuralOutlineNode {
    pub path: String,
    pub name: String,
    pub seq: Option<i64>,
    pub children: Vec<StructuralOutlineNode>,
    /// True when this node's subtree was cut by the cycle / depth guard (the node
    /// is shown, but not re-expanded). Lets the panel flag a malformed loop.
    pub truncated: bool,
    /// Mirrors `StructuralNode::contested` — an overruled `contains:` claim, shown
    /// flagged and never re-expanded (its real subtree lives under its real parent).
    #[serde(default)]
    pub contested: bool,
    #[serde(default)]
    pub contested_owner: Option<String>,
}

/// Defensive bound on tree depth (cycles are already caught by the visited-set; this
/// also caps a pathological very-deep legitimate chain so a read can never run away).
const MAX_DEPTH: usize = 64;

/// Resolve a note NAME → (path, canonical name) via `note_meta` (case-folded).
/// `None` when the target is unresolved (an orphan reference).
fn resolve_name(conn: &Connection, name: &str) -> Option<(String, String)> {
    conn.query_row(
        "SELECT path, name FROM note_meta WHERE LOWER(name) = LOWER(?1) LIMIT 1",
        rusqlite::params![name],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )
    .ok()
}

/// The ordered children of `parent_path` (name `parent_name`): the union of the two
/// faces. `contains` rows (source = parent, ordered by `seq`) come first; then
/// `parent` rows (notes naming this note as their parent), by name. Deduped by child
/// path — the seq-bearing `contains` row wins when a pair is declared on both sides.
fn children_of(conn: &Connection, parent_path: &str, parent_name: &str) -> Vec<StructuralNode> {
    use std::collections::HashSet;
    let mut out: Vec<StructuralNode> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // Face 1 — `contains` (parent's ordered child list). NULL seq sorts last.
    if let Ok(mut stmt) = conn.prepare(
        "SELECT target_name, seq FROM note_links \
         WHERE source_path = ?1 AND link_type = 'contains' AND status = 'active' \
         ORDER BY seq IS NULL, seq, target_name",
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![parent_path], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?))
        }) {
            for (cname, seq) in rows.flatten() {
                if let Some((cpath, cn)) = resolve_name(conn, &cname) {
                    if seen.insert(cpath.to_lowercase()) {
                        // D5 single-parent: a `contains:` child belongs here ONLY if THIS
                        // note is its resolved parent. If another claim wins (the child's own
                        // parent:, or a smaller-path container), surface this as a contested
                        // (overruled) listing — never silently drop it. (One extra parent_of
                        // resolve per contains-child; bounded by the TOC size, read-time only.)
                        let resolved = parent_of(conn, &cpath, &cn);
                        let (contested, owner) = match &resolved {
                            Some(p) if p.path.to_lowercase() != parent_path.to_lowercase() => {
                                (true, Some(p.name.clone()))
                            }
                            _ => (false, None),
                        };
                        out.push(StructuralNode { path: cpath, name: cn, seq, contested, contested_owner: owner });
                    }
                }
            }
        }
    }

    // Face 2 — `parent` (children that declared THIS note as their parent). Their own
    // `parent:` is authoritative, so THIS note is always their resolved parent (never contested).
    if let Ok(mut stmt) = conn.prepare(
        "SELECT source_path, source_name FROM note_links \
         WHERE target_name_lower = LOWER(?1) AND link_type = 'parent' AND status = 'active' \
         ORDER BY source_name",
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![parent_name], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            for (cpath, cname) in rows.flatten() {
                if seen.insert(cpath.to_lowercase()) {
                    out.push(StructuralNode { path: cpath, name: cname, seq: None, contested: false, contested_owner: None });
                }
            }
        }
    }
    out
}

/// The single DETERMINISTIC structural parent of `(note_path, note_name)`, or `None`.
/// Precedence (index-order-independent): (1) the note's OWN `parent:` face; (2) else
/// the smallest-path `contains:` claim targeting it.
fn parent_of(conn: &Connection, note_path: &str, note_name: &str) -> Option<StructuralNode> {
    // (1) The note's own `parent:` declaration — authoritative.
    if let Ok(pname) = conn.query_row(
        "SELECT target_name FROM note_links \
         WHERE source_path = ?1 AND link_type = 'parent' AND status = 'active' \
         ORDER BY target_name LIMIT 1",
        rusqlite::params![note_path],
        |r| r.get::<_, String>(0),
    ) {
        if let Some((ppath, pn)) = resolve_name(conn, &pname) {
            return Some(StructuralNode { path: ppath, name: pn, seq: None, contested: false, contested_owner: None });
        }
    }
    // (2) Else the smallest-path parent that `contains:` this note (deterministic tie-break).
    if let Ok((ppath, pn)) = conn.query_row(
        "SELECT source_path, source_name FROM note_links \
         WHERE target_name_lower = LOWER(?1) AND link_type = 'contains' AND status = 'active' \
         ORDER BY source_path LIMIT 1",
        rusqlite::params![note_name],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    ) {
        return Some(StructuralNode { path: ppath, name: pn, seq: None, contested: false, contested_owner: None });
    }
    None
}

fn descendants_rec(
    conn: &Connection,
    path: &str,
    name: &str,
    visited: &mut std::collections::HashSet<String>,
    depth: usize,
) -> Vec<StructuralOutlineNode> {
    if depth >= MAX_DEPTH {
        return Vec::new();
    }
    let mut out = Vec::new();
    for k in children_of(conn, path, name) {
        // An overruled `contains:` claim (D5): show it flagged, never re-expand — its
        // real subtree lives under its real parent. Surfaced, not silently dropped.
        if k.contested {
            out.push(StructuralOutlineNode {
                path: k.path,
                name: k.name,
                seq: k.seq,
                children: Vec::new(),
                truncated: false,
                contested: true,
                contested_owner: k.contested_owner,
            });
            continue;
        }
        let key = k.path.to_lowercase();
        if !visited.insert(key.clone()) {
            // Already on the current path → a cycle. Show the node, do not re-expand.
            out.push(StructuralOutlineNode {
                path: k.path,
                name: k.name,
                seq: k.seq,
                children: Vec::new(),
                truncated: true,
                contested: false,
                contested_owner: None,
            });
            continue;
        }
        let children = descendants_rec(conn, &k.path, &k.name, visited, depth + 1);
        // Backtrack: allow the same note to appear under a different branch (a DAG
        // reachable by two routes is still acyclic), while a true cycle on the active
        // path stays cut above.
        visited.remove(&key);
        out.push(StructuralOutlineNode {
            path: k.path,
            name: k.name,
            seq: k.seq,
            children,
            truncated: false,
            contested: false,
            contested_owner: None,
        });
    }
    out
}

/// The ordered children of one note (one level). Lazy; user-gesture only.
// App-freeze audit Batch-S (2026-07-03): `(async)` — this command reaches
// ensure_search_db_ready (or a multi-second walk/read) and used to PARK the
// WebView2 dispatch thread for the whole 20-40s cold init after a universe
// switch / boot (the Boss-reproduced switch freeze). Off-thread, the init
// still runs exactly once (init_lock) but the app stays responsive.
#[tauri::command(async)]
pub fn get_structural_children(
    app: tauri::AppHandle,
    note_path: String,
    note_name: String,
) -> Result<Vec<StructuralNode>, String> {
    let _ = crate::search::ensure_search_db_ready(&app);
    let state = app.state::<crate::search::SearchState>();
    crate::search::with_read_conn(state.inner(), |conn| {
        Ok(children_of(conn, &note_path, &note_name))
    })
}

/// The breadcrumb: the deterministic single-parent chain from the root down to (but
/// not including) this note. Visited-set on paths breaks any cycle.
// App-freeze audit Batch-S (2026-07-03): `(async)` — this command reaches
// ensure_search_db_ready (or a multi-second walk/read) and used to PARK the
// WebView2 dispatch thread for the whole 20-40s cold init after a universe
// switch / boot (the Boss-reproduced switch freeze). Off-thread, the init
// still runs exactly once (init_lock) but the app stays responsive.
#[tauri::command(async)]
pub fn get_structural_ancestors(
    app: tauri::AppHandle,
    note_path: String,
    note_name: String,
) -> Result<Vec<StructuralNode>, String> {
    let _ = crate::search::ensure_search_db_ready(&app);
    let state = app.state::<crate::search::SearchState>();
    crate::search::with_read_conn(state.inner(), |conn| {
        use std::collections::HashSet;
        let mut chain: Vec<StructuralNode> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(note_path.to_lowercase());
        let mut cur_path = note_path.clone();
        let mut cur_name = note_name.clone();
        for _ in 0..MAX_DEPTH {
            match parent_of(conn, &cur_path, &cur_name) {
                Some(p) => {
                    if !visited.insert(p.path.to_lowercase()) {
                        break; // cycle — stop cleanly
                    }
                    cur_path = p.path.clone();
                    cur_name = p.name.clone();
                    chain.push(p);
                }
                None => break,
            }
        }
        chain.reverse(); // root-first for the breadcrumb
        Ok(chain)
    })
}

/// The descendant outline subtree of one note (recursive children, cycle/depth-bounded).
// App-freeze audit Batch-S (2026-07-03): `(async)` — this command reaches
// ensure_search_db_ready (or a multi-second walk/read) and used to PARK the
// WebView2 dispatch thread for the whole 20-40s cold init after a universe
// switch / boot (the Boss-reproduced switch freeze). Off-thread, the init
// still runs exactly once (init_lock) but the app stays responsive.
#[tauri::command(async)]
pub fn get_structural_descendants(
    app: tauri::AppHandle,
    note_path: String,
    note_name: String,
) -> Result<Vec<StructuralOutlineNode>, String> {
    let _ = crate::search::ensure_search_db_ready(&app);
    let state = app.state::<crate::search::SearchState>();
    crate::search::with_read_conn(state.inner(), |conn| {
        let mut visited = std::collections::HashSet::new();
        visited.insert(note_path.to_lowercase());
        Ok(descendants_rec(conn, &note_path, &note_name, &mut visited, 0))
    })
}

#[cfg(test)]
mod tests {
    //! PJ-065 §6 — the read-time resolution the critics asked to pin: deterministic
    //! single-parent precedence + tie-break, the two-face children union with seq
    //! order, and the cycle guard that renders a malformed loop without hanging.
    use super::*;
    use rusqlite::Connection;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT);
             CREATE TABLE note_links (source_path TEXT, source_name TEXT, target_name TEXT,
                link_type TEXT, status TEXT, seq INTEGER,
                target_name_lower TEXT GENERATED ALWAYS AS (LOWER(target_name)) VIRTUAL);",
        )
        .unwrap();
        conn
    }
    fn note(conn: &Connection, path: &str, name: &str) {
        conn.execute("INSERT INTO note_meta(path,name) VALUES (?1,?2)", rusqlite::params![path, name]).unwrap();
    }
    fn edge(conn: &Connection, sp: &str, sn: &str, tgt: &str, lt: &str, seq: Option<i64>) {
        conn.execute(
            "INSERT INTO note_links(source_path,source_name,target_name,link_type,status,seq) VALUES (?1,?2,?3,?4,'active',?5)",
            rusqlite::params![sp, sn, tgt, lt, seq],
        ).unwrap();
    }

    #[test]
    fn children_union_contains_by_seq_then_parent_face() {
        let conn = db();
        note(&conn, "/book.md", "Book");
        note(&conn, "/c1.md", "Ch1");
        note(&conn, "/c2.md", "Ch2");
        note(&conn, "/c3.md", "Ch3");
        // Book contains Ch2(seq2), Ch1(seq1) — declared out of order; seq drives order.
        edge(&conn, "/book.md", "Book", "Ch2", "contains", Some(2));
        edge(&conn, "/book.md", "Book", "Ch1", "contains", Some(1));
        // Ch3 declares parent: Book (the other face) — appended after the ordered list.
        edge(&conn, "/c3.md", "Ch3", "Book", "parent", None);
        let kids = children_of(&conn, "/book.md", "Book");
        let names: Vec<&str> = kids.iter().map(|k| k.name.as_str()).collect();
        assert_eq!(names, vec!["Ch1", "Ch2", "Ch3"], "contains by seq (1,2), then the parent-face child");
        assert_eq!(kids[0].seq, Some(1));
    }

    #[test]
    fn parent_own_declaration_wins_over_contains_claim() {
        let conn = db();
        note(&conn, "/x.md", "X");
        note(&conn, "/a.md", "A");
        note(&conn, "/b.md", "B");
        edge(&conn, "/x.md", "X", "A", "parent", None); // X says its parent is A
        edge(&conn, "/b.md", "B", "X", "contains", Some(1)); // B claims to contain X
        let p = parent_of(&conn, "/x.md", "X").unwrap();
        assert_eq!(p.name, "A", "the child's own parent: declaration is authoritative");
    }

    #[test]
    fn parent_contains_tie_breaks_by_smallest_path() {
        let conn = db();
        note(&conn, "/y.md", "Y");
        note(&conn, "/zeta.md", "Zeta");
        note(&conn, "/alpha.md", "Alpha");
        // Y has no own parent; two notes claim contains: Y → smallest path wins, deterministically.
        edge(&conn, "/zeta.md", "Zeta", "Y", "contains", Some(1));
        edge(&conn, "/alpha.md", "Alpha", "Y", "contains", Some(1));
        let p = parent_of(&conn, "/y.md", "Y").unwrap();
        assert_eq!(p.path, "/alpha.md", "deterministic smallest-path tie-break (index-order-independent)");
    }

    #[test]
    fn contested_contains_claim_is_flagged_not_silently_dropped() {
        let conn = db();
        note(&conn, "/oa.md", "Owner A");
        note(&conn, "/ob.md", "Owner B");
        note(&conn, "/cc.md", "Contested Child");
        edge(&conn, "/cc.md", "Contested Child", "Owner A", "parent", None); // child's own parent: A wins
        edge(&conn, "/ob.md", "Owner B", "Contested Child", "contains", Some(1)); // B's competing claim
        // Owner A (the resolved parent) shows it as a REAL child, not contested.
        let a_kids = children_of(&conn, "/oa.md", "Owner A");
        assert_eq!(a_kids.len(), 1);
        assert_eq!(a_kids[0].name, "Contested Child");
        assert!(!a_kids[0].contested, "the real parent lists it as a normal child");
        // Owner B (the losing claimant) shows it FLAGGED contested → Owner A (surfaced, not dropped).
        let b_kids = children_of(&conn, "/ob.md", "Owner B");
        assert_eq!(b_kids.len(), 1, "the overruled claim is surfaced, never silently dropped");
        assert!(b_kids[0].contested, "Owner B's claim lost → flagged");
        assert_eq!(b_kids[0].contested_owner.as_deref(), Some("Owner A"));
    }

    #[test]
    fn descendants_cycle_is_truncated_never_hangs() {
        let conn = db();
        note(&conn, "/a.md", "A");
        note(&conn, "/b.md", "B");
        edge(&conn, "/a.md", "A", "B", "contains", Some(1)); // A contains B
        edge(&conn, "/b.md", "B", "A", "contains", Some(1)); // B contains A — a 2-cycle
        let mut visited = std::collections::HashSet::new();
        visited.insert("/a.md".to_string());
        let tree = descendants_rec(&conn, "/a.md", "A", &mut visited, 0);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "B");
        assert_eq!(tree[0].children.len(), 1, "B's child A is shown");
        assert!(tree[0].children[0].truncated, "the loop-closing A is not re-expanded — no hang");
    }
}
