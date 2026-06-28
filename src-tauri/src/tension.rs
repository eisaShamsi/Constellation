//! Tension Detector — Cognitive Engine Phase 4.
//!
//! Surfaces contradictions, orphan knowledge, structural gaps, and single
//! points of failure. Zero AI — pure graph topology analysis.
//!
//! 4 Detection Types:
//!   1. Contradictions  — notes linked with `contradicts`
//!   2. Orphans         — notes with 0 inbound links
//!   3. Structural gaps — tag-clusters with no cross-wikilinks
//!   4. Single points   — notes with 5+ inbound but ≤1 derives-from source
//!
//! Earned complexity: activates only when library has 50+ linked notes.
//!
//! MIG-075 §A2 — inputs are read from the DB (`note_meta` name/path/
//! word_count · `note_links` active rows · `json_each(tags_json)`)
//! instead of re-reading every .md per run (Perf Rule 8). The detection
//! algorithms are unchanged. Documented input deltas vs the retired fs
//! walk: tag coverage widens (tags_json carries every script + the
//! frontmatter `tags:` lists; the walk's regex was Latin+Arabic inline
//! only); contradiction rows are per-(source,target) pair without the
//! ×N occurrence multiplier (`note_links` stores one row per source ×
//! type × target); archived links are excluded (`status='active'`); and
//! word_count is the indexer's markdown-stripped count (the walk counted
//! raw tokens), so orphan severity tiers can shift at the margins.
//! The command is async so the scan never blocks the WebView2 UI thread.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tauri::Manager;

#[derive(Debug, Clone, Serialize)]
pub struct TensionReport {
    pub contradictions: Vec<TensionItem>,
    pub orphans: Vec<TensionItem>,
    pub structural_gaps: Vec<GapItem>,
    pub single_points: Vec<TensionItem>,
    pub total_linked_notes: usize,
    pub total_notes: usize,
    pub active: bool, // false if <50 linked notes
}

#[derive(Debug, Clone, Serialize)]
pub struct TensionItem {
    pub note_name: String,
    pub note_path: String,
    pub severity: String, // "low" | "medium" | "high"
    pub detail: String,   // English fallback + the Rust-test oracle
    // MIG-080 §E — localization handle: the frontend renders the user-facing detail
    // from `detail_kind` + `detail_args` via $t (the hardcoded `detail` above was the
    // only un-i18n'd user string on the Health surface). kinds:
    // "contradicts" | "contradicted_by" | "orphan" | "single_point".
    pub detail_kind: String,
    pub detail_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GapItem {
    pub tag: String,
    pub notes: Vec<String>, // note names in the cluster (top 5, for display)
    pub severity: String,
    // MIG-080 §E — the FULL cluster membership (lower-cased note names). The
    // note-scoped Health tab tests whether the open note is in THIS gap against
    // this list — `notes` is truncated to 5 for display and must NOT be used for
    // membership (a note in a >5-member gap would be missed non-deterministically).
    pub member_names: Vec<String>,
}

struct NoteInfo {
    path: String,
    name: String,
    word_count: usize,
    outgoing: Vec<(String, Option<String>)>, // (target_name_lower, link_type)
    tags: HashSet<String>,
}

/// Detect knowledge tensions in a library.
#[tauri::command(async)]
pub fn detect_tensions(
    app: tauri::AppHandle,
    library_path: String,
    library_name: String,
) -> Result<TensionReport, String> {
    // Same access contract as the retired fs walk: cUniverse library
    // paths are not registered own-libraries and are refused — the
    // health tab keeps its honest `unavailable` state for them
    // (federated tension reads are the reserved MIG-063 family).
    crate::libraries::validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;

    // Phase 1: load the per-library inputs from the DB. Lock held for
    // the three reads only, released before any detection work.
    let notes = {
        let state = app.state::<crate::search::SearchState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conn = db.as_ref().ok_or("Search DB not initialized")?;
        load_notes_from_db(conn, &library_name)?
    };

    Ok(detect_from_notes(notes))
}

/// MIG-080 §E — whether the open note can be RELIABLY analyzed for note-scoped
/// Health. The library tension detection is name-keyed (links resolve by note
/// name), so two notes sharing a title collapse (last-wins) and the loser would
/// look "healthy"; and a not-yet-indexed note isn't in the analyzed set at all.
/// Two O(1) indexed lookups — NOT the full detection — so this is cheap to call
/// per note-switch alongside the cached library report.
#[derive(Debug, Clone, Serialize)]
pub struct NoteTensionStatus {
    pub indexed: bool,        // the note has a note_meta row (it was indexed)
    pub ambiguous_title: bool, // another note in the library shares its (case-insensitive) title
}

#[tauri::command(async)]
pub fn note_tension_status(
    app: tauri::AppHandle,
    library_name: String,
    note_path: String,
) -> Result<NoteTensionStatus, String> {
    let state = app.state::<crate::search::SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;
    compute_note_tension_status(conn, &library_name, &note_path)
}

/// Pure core of `note_tension_status` (testable against an in-memory DB).
fn compute_note_tension_status(
    conn: &rusqlite::Connection,
    library_name: &str,
    note_path: &str,
) -> Result<NoteTensionStatus, String> {
    let name: Option<String> = match conn.query_row(
        "SELECT name FROM note_meta WHERE path = ?1",
        rusqlite::params![note_path],
        |r| r.get::<_, String>(0),
    ) {
        Ok(n) => Some(n),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err(e.to_string()),
    };

    let ambiguous_title = if let Some(ref n) = name {
        let cnt: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM note_meta WHERE library_name = ?1 AND LOWER(name) = LOWER(?2)",
                rusqlite::params![library_name, n],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        cnt > 1
    } else {
        false
    };

    Ok(NoteTensionStatus {
        indexed: name.is_some(),
        ambiguous_title,
    })
}

/// Build the per-library `NoteInfo` map from the DB — the same shape the
/// retired `scan_notes_recursive` produced, minus the file reads.
fn load_notes_from_db(
    conn: &rusqlite::Connection,
    library_name: &str,
) -> Result<HashMap<String, NoteInfo>, String> {
    let mut notes: HashMap<String, NoteInfo> = HashMap::new();
    // path → name-lower key, for attaching note_links rows (keyed by
    // source_path) to their note entry.
    let mut path_to_key: HashMap<String, String> = HashMap::new();

    {
        let mut stmt = conn
            .prepare("SELECT name, path, word_count FROM note_meta WHERE library_name = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![library_name], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let (name, path, word_count) = row;
            let key = name.to_lowercase();
            path_to_key.insert(path.clone(), key.clone());
            // Duplicate titles within a library collapse to one entry
            // (last wins) — the fs walk had the same keyed-by-name
            // semantics.
            notes.insert(key, NoteInfo {
                path,
                name,
                word_count: word_count.max(0) as usize,
                outgoing: Vec::new(),
                tags: HashSet::new(),
            });
        }
    }

    {
        // PJ-065 — exclude the structural (parent/TOC) lane: tension's orphan / SPOF /
        // contradiction analysis is a cognitive instrument, so a node with ONLY
        // structural links reads as a cognitive orphan (correct). Active since §5.
        let sx = crate::link_types::snapshot().structural_not_in_clause("link_type");
        let mut stmt = conn
            .prepare(&format!(
                "SELECT source_path, target_name, link_type FROM note_links \
                 WHERE library_name = ?1 AND status = 'active'{}",
                sx
            ))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![library_name], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let (source_path, target_name, link_type) = row;
            if let Some(key) = path_to_key.get(&source_path) {
                if let Some(info) = notes.get_mut(key) {
                    info.outgoing.push((target_name.to_lowercase(), link_type));
                }
            }
        }
    }

    {
        let mut stmt = conn
            .prepare(
                "SELECT nm.name, je.value FROM note_meta nm, json_each(nm.tags_json) je \
                 WHERE nm.library_name = ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![library_name], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let (name, tag) = row;
            let tag = tag.trim().to_lowercase();
            if tag.is_empty() { continue; }
            if let Some(info) = notes.get_mut(&name.to_lowercase()) {
                info.tags.insert(tag);
            }
        }
    }

    Ok(notes)
}

/// The four detections — unchanged from the pre-MIG-075 walk; pure so
/// the fixture tests can exercise it without an AppHandle.
fn detect_from_notes(notes: HashMap<String, NoteInfo>) -> TensionReport {
    let note_names: HashSet<String> = notes.keys().cloned().collect();

    // Phase 2: Build inbound map + each note's OUTGOING derives-from count.
    let mut inbound_count: HashMap<String, usize> = HashMap::new();
    let mut inbound_sources: HashMap<String, HashSet<String>> = HashMap::new();
    // MIG-085 — a single point of failure is a note MANY depend on that itself rests on
    // ≤1 derives-from support (OUTGOING). Count the note's own outgoing derives-from, to
    // match the Reviewer's fragile lens + inspector360's SPOF (one canonical definition).
    let mut out_derives: HashMap<String, usize> = HashMap::new();

    for (key, info) in &notes {
        for (target, link_type) in &info.outgoing {
            if link_type.as_deref() == Some("derives-from") {
                *out_derives.entry(key.clone()).or_insert(0) += 1;
            }
            if note_names.contains(target) {
                *inbound_count.entry(target.clone()).or_insert(0) += 1;
                inbound_sources.entry(target.clone()).or_default().insert(info.name.clone());
            }
        }
    }

    let total_notes = notes.len();
    let total_linked = inbound_count.len();

    // Earned complexity check
    if total_linked < 50 {
        return TensionReport {
            contradictions: vec![],
            orphans: vec![],
            structural_gaps: vec![],
            single_points: vec![],
            total_linked_notes: total_linked,
            total_notes,
            active: false,
        };
    }

    // Detection 1: Contradictions — ONE row per (source, target) pair
    // (Boss-approved dedupe, 2026-06-10). The §A2 DB re-source made the
    // input row-unique per (source, type, target), so the old ×N
    // occurrence counter became unreachable and was removed in /simplify;
    // the map stays as the defensive pair-dedupe + stable-sort anchor.
    // key: (source_path, target_lower) → (source_name, target_name, target_path)
    let mut contradiction_pairs: HashMap<(String, String), (String, String, String)> =
        HashMap::new();
    for info in notes.values() {
        for (target, link_type) in &info.outgoing {
            if link_type.as_deref() == Some("contradicts") {
                if let Some(target_info) = notes.get(target) {
                    contradiction_pairs
                        .entry((info.path.clone(), target.clone()))
                        .or_insert_with(|| {
                            (
                                info.name.clone(),
                                target_info.name.clone(),
                                target_info.path.clone(),
                            )
                        });
                }
            }
        }
    }
    // Stable order by source name (HashMap order is random per run; the
    // panel should not reshuffle on every open).
    let mut pair_rows: Vec<((String, String), (String, String, String))> =
        contradiction_pairs.into_iter().collect();
    pair_rows.sort_by(|a, b| a.1 .0.cmp(&b.1 .0));
    // MIG-080 §E — emit BOTH perspectives so the note-scoped Health tab surfaces the
    // tension whether the open note is the SOURCE (it contradicts X) or the TARGET
    // (it is contradicted by Y) of the `contradicts` link. A contradicted note is
    // NOT healthy. (`detect_tensions` is now consumed only by the note-scoped filter,
    // so the per-pair duplication is sliced back apart per note.)
    let mut contradictions: Vec<TensionItem> = Vec::new();
    for ((source_path, _), (source_name, target_name, target_path)) in pair_rows {
        // A note that `contradicts`-links itself has no meaningful "contradicted by"
        // counterpart — emit one row, not two near-identical ones.
        let is_self = source_path == target_path;
        contradictions.push(TensionItem {
            note_name: source_name.clone(),
            note_path: source_path,
            severity: "high".to_string(),
            detail: format!("contradicts \"{}\"", target_name),
            detail_kind: "contradicts".to_string(),
            detail_args: vec![target_name.clone()],
        });
        if !is_self {
            contradictions.push(TensionItem {
                note_name: target_name,
                note_path: target_path,
                severity: "high".to_string(),
                detail: format!("contradicted by \"{}\"", source_name),
                detail_kind: "contradicted_by".to_string(),
                detail_args: vec![source_name],
            });
        }
    }

    // Detection 2: Orphans (0 inbound links, has content)
    let mut orphans: Vec<TensionItem> = Vec::new();
    for info in notes.values() {
        let inbound = inbound_count.get(&info.name.to_lowercase()).copied().unwrap_or(0);
        if inbound == 0 && info.word_count > 20 {
            let severity = if info.word_count > 500 { "high" }
                else if info.word_count > 100 { "medium" }
                else { "low" };
            orphans.push(TensionItem {
                note_name: info.name.clone(),
                note_path: info.path.clone(),
                severity: severity.to_string(),
                detail: format!("{} words, no inbound links", info.word_count),
                detail_kind: "orphan".to_string(),
                detail_args: vec![info.word_count.to_string()],
            });
        }
    }
    // Sort orphans: high severity first
    orphans.sort_by(|a, b| {
        let ord = |s: &str| match s { "high" => 0, "medium" => 1, _ => 2 };
        ord(&a.severity).cmp(&ord(&b.severity))
    });

    // Detection 3: Structural gaps (tag-clusters without cross-links)
    let mut structural_gaps: Vec<GapItem> = Vec::new();
    // Group notes by tag
    let mut tag_notes: HashMap<String, Vec<String>> = HashMap::new();
    for info in notes.values() {
        for tag in &info.tags {
            tag_notes.entry(tag.clone()).or_default().push(info.name.to_lowercase());
        }
    }
    // Find tags with 3+ notes where notes don't link to each other
    for (tag, members) in &tag_notes {
        if members.len() < 3 { continue; }
        // Check how many cross-links exist within this tag group
        let member_set: HashSet<&String> = members.iter().collect();
        let mut cross_links = 0;
        for member in members {
            if let Some(info) = notes.get(member) {
                for (target, _) in &info.outgoing {
                    if member_set.contains(target) && target != member {
                        cross_links += 1;
                    }
                }
            }
        }
        // If fewer than 20% of possible links exist, it's a gap
        let possible = members.len() * (members.len() - 1);
        if possible > 0 && cross_links * 5 < possible {
            // Deterministic display: sort the resolved names, then take 5 (the
            // HashMap-iteration order of `members` is otherwise random per run).
            let mut all_names: Vec<String> = members.iter()
                .filter_map(|m| notes.get(m).map(|i| i.name.clone()))
                .collect();
            all_names.sort();
            let note_names: Vec<String> = all_names.iter().take(5).cloned().collect();
            // MIG-080 §E — full membership (lower-cased keys) for note-scoped filtering.
            let mut member_names = members.clone();
            member_names.sort();
            member_names.dedup();
            structural_gaps.push(GapItem {
                tag: tag.clone(),
                notes: note_names,
                severity: if members.len() >= 8 { "high".to_string() }
                    else if members.len() >= 5 { "medium".to_string() }
                    else { "low".to_string() },
                member_names,
            });
        }
    }
    structural_gaps.sort_by(|a, b| b.notes.len().cmp(&a.notes.len()));
    structural_gaps.truncate(20); // limit to top 20 gaps

    // Detection 4: Single points of failure
    let mut single_points: Vec<TensionItem> = Vec::new();
    for (name_lower, sources) in &inbound_sources {
        if sources.len() >= 5 {
            // OUTGOING derives-from — what THIS note rests on (matches the Reviewer/360).
            let derives_count = out_derives.get(name_lower).copied().unwrap_or(0);
            if derives_count <= 1 {
                if let Some(info) = notes.get(name_lower) {
                    single_points.push(TensionItem {
                        note_name: info.name.clone(),
                        note_path: info.path.clone(),
                        severity: if sources.len() >= 10 { "high".to_string() }
                            else { "medium".to_string() },
                        detail: format!("{} notes depend on this; it rests on only {} support", sources.len(), derives_count),
                        detail_kind: "single_point".to_string(),
                        detail_args: vec![sources.len().to_string(), derives_count.to_string()],
                    });
                }
            }
        }
    }

    TensionReport {
        contradictions,
        orphans,
        structural_gaps,
        single_points,
        total_linked_notes: total_linked,
        total_notes,
        active: true,
    }
}

// ─── MIG-075 §A2 tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests_mig075_tension {
    //! Pins the DB-sourced tension pipeline: the loader's three reads +
    //! library scoping, and each detection on fixtures — including the
    //! documented deltas (per-pair contradictions without ×N; active-only).
    use super::*;
    use rusqlite::Connection;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                 path TEXT PRIMARY KEY, name TEXT, library_name TEXT,
                 word_count INTEGER DEFAULT 0, tags_json TEXT DEFAULT '[]');
             CREATE TABLE note_links (
                 source_path TEXT, target_name TEXT, link_type TEXT,
                 status TEXT DEFAULT 'active', library_name TEXT);",
        )
        .unwrap();
        conn
    }

    fn add_note(conn: &Connection, lib: &str, name: &str, words: i64, tags: &str) {
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, word_count, tags_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![format!("/{lib}/{name}.md"), name, lib, words, tags],
        )
        .unwrap();
    }

    fn add_link(conn: &Connection, lib: &str, source: &str, target: &str, lt: &str, status: &str) {
        conn.execute(
            "INSERT INTO note_links (source_path, target_name, link_type, status, library_name)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![format!("/{lib}/{source}.md"), target.to_lowercase(), lt, status, lib],
        )
        .unwrap();
    }

    /// A 51-note chain n0→n1→…→n50 gives 50 distinct inbound targets —
    /// exactly the earned-complexity floor.
    fn add_chain(conn: &Connection, lib: &str) {
        for i in 0..=50 {
            add_note(conn, lib, &format!("n{i}"), 30, "[]");
        }
        for i in 0..50 {
            add_link(conn, lib, &format!("n{i}"), &format!("n{}", i + 1), "associative", "active");
        }
    }

    #[test]
    fn loader_scopes_by_library_and_reads_all_three_inputs() {
        let conn = mem_db();
        add_note(&conn, "A", "Alpha", 25, r#"["philosophy","فلسفة"]"#);
        add_note(&conn, "B", "Other", 25, r#"["noise"]"#);
        add_link(&conn, "A", "Alpha", "beta", "supports", "active");
        add_link(&conn, "A", "Alpha", "gone", "supports", "archived"); // excluded
        add_link(&conn, "B", "Other", "alpha", "supports", "active"); // other library
        let notes = load_notes_from_db(&conn, "A").unwrap();
        assert_eq!(notes.len(), 1, "library-scoped");
        let alpha = notes.get("alpha").unwrap();
        assert_eq!(alpha.outgoing.len(), 1, "active-only outgoing");
        assert!(alpha.tags.contains("philosophy") && alpha.tags.contains("فلسفة"),
            "tags from json_each incl. non-Latin: {:?}", alpha.tags);
    }

    #[test]
    fn gate_inactive_below_50_linked() {
        let conn = mem_db();
        add_note(&conn, "A", "x", 30, "[]");
        add_note(&conn, "A", "y", 30, "[]");
        add_link(&conn, "A", "x", "y", "associative", "active");
        let report = detect_from_notes(load_notes_from_db(&conn, "A").unwrap());
        assert!(!report.active);
        assert_eq!(report.total_linked_notes, 1);
    }

    #[test]
    fn contradiction_is_one_row_per_pair_without_multiplier() {
        let conn = mem_db();
        add_chain(&conn, "A");
        add_link(&conn, "A", "n0", "n5", "contradicts", "active");
        let report = detect_from_notes(load_notes_from_db(&conn, "A").unwrap());
        assert!(report.active);
        // MIG-080 §E — TWO rows per pair: the SOURCE ("contradicts") and the TARGET
        // ("contradicted by"), so a contradicted note also surfaces the tension in its
        // note-scoped Health tab. Still NO ×N occurrence multiplier (the old bug).
        assert_eq!(report.contradictions.len(), 2);
        let source = report
            .contradictions
            .iter()
            .find(|r| r.note_name == "n0")
            .expect("source row");
        assert!(source.detail.contains("contradicts"), "{}", source.detail);
        assert_eq!(source.detail_kind, "contradicts");
        let target = report
            .contradictions
            .iter()
            .find(|r| r.note_name == "n5")
            .expect("target row");
        assert!(target.detail.contains("contradicted by"), "{}", target.detail);
        assert_eq!(target.detail_kind, "contradicted_by");
        for row in &report.contradictions {
            assert!(!row.detail.contains('×'), "no occurrence multiplier: {}", row.detail);
        }
    }

    #[test]
    fn orphan_gap_and_single_point_detected() {
        let conn = mem_db();
        add_chain(&conn, "A");
        // Orphan: >20 words, zero inbound.
        add_note(&conn, "A", "Lonely", 120, "[]");
        // Gap: three notes sharing a tag, no cross-links among them.
        for name in ["g1", "g2", "g3"] {
            add_note(&conn, "A", name, 30, r#"["cluster"]"#);
        }
        // Single point: 5 inbound sources, 0 derives-from.
        add_note(&conn, "A", "Hub", 30, "[]");
        for i in 10..15 {
            add_link(&conn, "A", &format!("n{i}"), "hub", "supports", "active");
        }
        let report = detect_from_notes(load_notes_from_db(&conn, "A").unwrap());
        assert!(report.active);
        assert!(report.orphans.iter().any(|o| o.note_name == "Lonely"), "orphan found");
        assert!(report.structural_gaps.iter().any(|g| g.tag == "cluster"), "tag-cluster gap found");
        assert!(report.single_points.iter().any(|s| s.note_name == "Hub"), "SPOF found");
    }

    #[test]
    fn note_tension_status_indexed_and_ambiguous() {
        let conn = mem_db();
        add_note(&conn, "A", "Unique", 30, "[]"); // /A/Unique.md
        // Two notes sharing a (case-insensitive) title within library A, distinct paths:
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, word_count, tags_json) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params!["/A/dup1.md", "Twin", "A", 30, "[]"],
        ).unwrap();
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, word_count, tags_json) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params!["/A/dup2.md", "twin", "A", 30, "[]"], // case-insensitive collision
        ).unwrap();
        // Same title in a DIFFERENT library must NOT count as ambiguous within A:
        add_note(&conn, "B", "Unique", 30, "[]");

        let absent = compute_note_tension_status(&conn, "A", "/A/missing.md").unwrap();
        assert!(!absent.indexed && !absent.ambiguous_title, "absent path → not indexed");

        let unique = compute_note_tension_status(&conn, "A", "/A/Unique.md").unwrap();
        assert!(unique.indexed && !unique.ambiguous_title, "unique title, cross-lib twin ignored");

        let dup = compute_note_tension_status(&conn, "A", "/A/dup1.md").unwrap();
        assert!(dup.indexed && dup.ambiguous_title, "shared title within library → ambiguous");
    }

    #[test]
    fn self_contradiction_emits_single_row() {
        let conn = mem_db();
        add_chain(&conn, "A");
        add_link(&conn, "A", "n0", "n0", "contradicts", "active"); // self-link
        let report = detect_from_notes(load_notes_from_db(&conn, "A").unwrap());
        // One row (the "contradicts" source perspective), NOT a duplicate "contradicted by".
        let n0_rows: Vec<_> = report.contradictions.iter().filter(|r| r.note_name == "n0").collect();
        assert_eq!(n0_rows.len(), 1, "self-contradiction is one row, not two");
        assert_eq!(n0_rows[0].detail_kind, "contradicts");
    }
}
