//! Constellation Map — CE Layer 2.
//!
//! Computes a hierarchical tree with knowledge weight, maturity, and stratum
//! per node for the radial sunburst visualization (Constellation Map).
//!
//! Pipeline: recursive filesystem walk → word count + link extraction →
//! inbound link aggregation → maturity/stratum inference → weight computation →
//! recursive tree with aggregated metrics.
//!
//! Based on the Constellation Map Concept Paper (April 2026).

use rusqlite;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

use crate::strata::strip_frontmatter_pub;

/// MIG-005 §1: read the `note_aliases` table into an in-memory
/// `alias_lower → canonical_path` map. Used by inbound-link aggregators so
/// a wikilink targeting a renamed note's old title (or any historical
/// alias) is counted toward the renamed note instead of being lost as a
/// broken link.
///
/// Failures (DB lock contended, table missing on first boot before MIG-004
/// schema upgrade, query error) all degrade to an empty map — the caller
/// sees pre-MIG-005 alias-blind behavior, which is correct for an empty
/// alias table anyway.
///
/// Mirrors the SELECT shape used by `cache.rs::cache_boot_snapshot_sky`
/// (MIG-004 §8), including the `ORDER BY path` deterministic-collision
/// tiebreak: when two notes legitimately share an alias, the
/// lexicographically-first path wins (same as the boot snapshot).
fn load_alias_map(app: &tauri::AppHandle) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let state = app.state::<crate::search::SearchState>();
    let guard = match state.db.lock() {
        Ok(g) => g,
        Err(_) => return map,
    };
    let conn = match guard.as_ref() {
        Some(c) => c,
        None => return map,
    };
    let mut stmt = match conn
        .prepare("SELECT alias_lower, path FROM note_aliases ORDER BY path")
    {
        Ok(s) => s,
        Err(_) => return map,
    };
    let rows = match stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    }) {
        Ok(rs) => rs,
        Err(_) => return map,
    };
    for r in rows.flatten() {
        // First-write-wins on collision (matches cache.rs §8 / §110).
        map.entry(r.0).or_insert(r.1);
    }
    map
}

/// MIG-077 perf — load note records from the indexed `note_meta` table instead
/// of reading every file on disk. The OrgChart/Map's per-file walk
/// (`collect_notes_recursive`) read the WHOLE universe (measured: 419 MB / 7,664
/// files ≈ 40–58 s with per-file AV scanning) on EVERY open AND every reload — a
/// Rule-8 violation. note_meta already holds word_count + outgoing links +
/// timestamps. Returns empty on any error, so callers fall back to the disk walk
/// (preserves correctness for federated child-universe libs not in this DB, and
/// is a no-op-safe degrade if the index is unavailable).
fn load_note_records(app: &tauri::AppHandle) -> Vec<NoteRecord> {
    let mut out: Vec<NoteRecord> = Vec::new();
    let state = app.state::<crate::search::SearchState>();
    let guard = match state.db.lock() { Ok(g) => g, Err(_) => return out };
    let conn = match guard.as_ref() { Some(c) => c, None => return out };
    load_note_records_from_conn(conn, &mut out);
    out
}

/// Load note records from a child universe's own search.db. Each child
/// universe maintains its own `note_meta` table; the active universe's
/// DB has no entries for them. Without this, the Map falls back to
/// `collect_notes_recursive` (read every .md file from disk) for child
/// universe libraries — measured 2+ minutes on Eisa's 7,600-note
/// universe with per-file antivirus scanning.
fn load_note_records_for_child_universe(cu_path: &str) -> Vec<NoteRecord> {
    let mut out: Vec<NoteRecord> = Vec::new();
    let db_path = std::path::Path::new(cu_path)
        .join(".constellation")
        .join("search.db");
    if !db_path.exists() { return out; }
    let conn = match rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(_) => return out,
    };
    let _ = conn.execute_batch("PRAGMA mmap_size=268435456;");
    load_note_records_from_conn(&conn, &mut out);
    out
}

fn load_note_records_from_conn(conn: &rusqlite::Connection, out: &mut Vec<NoteRecord>) {
    let mut stmt = match conn.prepare(
        "SELECT path, name, word_count, outgoing_links_json, modified, created_at FROM note_meta",
    ) {
        Ok(s) => s,
        Err(_) => return,
    };
    let rows = match stmt.query_map([], |r| {
        Ok((
            r.get::<_, Option<String>>(0)?.unwrap_or_default(),
            r.get::<_, Option<String>>(1)?.unwrap_or_default(),
            r.get::<_, Option<i64>>(2)?.unwrap_or(0) as u32,
            r.get::<_, Option<String>>(3)?.unwrap_or_default(),
            r.get::<_, Option<i64>>(4)?.unwrap_or(0) as u64,
            r.get::<_, Option<i64>>(5)?.unwrap_or(0) as u64,
        ))
    }) {
        Ok(rs) => rs,
        Err(_) => return,
    };
    for row in rows.flatten() {
        let (path, name, word_count, links_json, modified, created) = row;
        if path.is_empty() { continue; }
        let outgoing_links: Vec<String> = serde_json::from_str::<Vec<String>>(&links_json)
            .unwrap_or_default()
            .into_iter()
            .map(|s| match s.find("::") {
                Some(i) => s[i + 2..].trim().to_lowercase(),
                None => s.trim().to_lowercase(),
            })
            .filter(|t| !t.is_empty())
            .collect();
        let created = if created > 0 { created } else { modified };
        out.push(NoteRecord { path, name, word_count, outgoing_links, modified, created });
    }
}

/// A node in the Map tree — universe, child_universe, library, folder, or note.
#[derive(Debug, Clone, Serialize)]
pub struct MapNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub node_type: String, // "universe" | "child_universe" | "library" | "folder" | "note"
    pub weight: f64,
    pub note_count: u32,
    pub word_count: u32,
    pub link_count: u32,
    pub maturity: Option<String>,
    pub stratum: Option<u8>,
    pub modified: Option<u64>,
    pub children: Option<Vec<MapNode>>,
}

/// Internal record for a note, built during the first pass.
#[derive(Clone)]
struct NoteRecord {
    path: String,
    name: String,
    word_count: u32,
    outgoing_links: Vec<String>, // target note names (lowercase)
    modified: u64,               // unix timestamp
    created: u64,                // unix timestamp
}

/// Compute the Constellation Map tree for a library.
#[tauri::command]
pub fn constellation_map_data(
    app: tauri::AppHandle,
    library_path: String,
    max_depth: Option<u32>,
) -> Result<MapNode, String> {
    // Security: validate library access
    let libraries = crate::libraries::load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }

    let root = Path::new(&library_path);
    if !root.is_dir() {
        return Err("Library path is not a directory.".to_string());
    }

    let depth_limit = max_depth.unwrap_or(5);

    // Pass 1: Collect all notes with metadata
    let mut all_notes: Vec<NoteRecord> = Vec::new();
    collect_notes_recursive(root, &mut all_notes);

    // MIG-005 §1: load the alias resolution map once for this command.
    // Renamed notes still have wikilinks under their old titles in other
    // notes; without this lookup those wikilinks are silently dropped from
    // the renamed note's inbound count and the bubble shrinks visibly.
    let alias_to_path = load_alias_map(&app);

    // Build note name → path map for link resolution
    let mut name_to_path: HashMap<String, String> = HashMap::new();
    let mut path_to_name: HashMap<String, String> = HashMap::new();
    for note in &all_notes {
        let name_lower = note.name.to_lowercase();
        name_to_path.insert(name_lower.clone(), note.path.clone());
        path_to_name.insert(note.path.clone(), name_lower);
    }

    // Build inbound link count map keyed by canonical lowercased note name.
    // 3-tier resolution per cache.rs::read_sky_links_raw (MIG-004 §8):
    //   1. target is a current note name → count toward that name.
    //   2. target is an alias of a current note → resolve to canonical
    //      path → look up canonical name → count toward THAT name.
    //   3. unresolved (broken link) → skip; don't pollute inbound_map.
    let mut inbound_map: HashMap<String, usize> = HashMap::new();
    for note in &all_notes {
        for target in &note.outgoing_links {
            let canonical_name = if name_to_path.contains_key(target) {
                target.clone()
            } else if let Some(canonical_path) = alias_to_path.get(target) {
                match path_to_name.get(canonical_path) {
                    Some(n) => n.clone(),
                    None => continue,
                }
            } else {
                continue;
            };
            *inbound_map.entry(canonical_name).or_insert(0) += 1;
        }
    }

    // Build note metadata map: path_lower → (word_count, link_count, inbound, maturity, stratum, modified)
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // MIG-008 Step 1: trailing String is the display name (frontmatter title
    // with file_stem fallback) that build_tree threads into MapNode.name so
    // the sunburst's hover/breadcrumb labels never expose the canonical
    // filename to the user.
    let mut note_meta: HashMap<String, (u32, u32, String, Option<u8>, u64, String)> = HashMap::new();
    for note in &all_notes {
        let key = note.path.replace('\\', "/").to_lowercase();
        let inbound = *inbound_map.get(&note.name.to_lowercase()).unwrap_or(&0);

        // Maturity (reuse logic from maturity.rs)
        let days_since_created = (now_secs.saturating_sub(note.created)) / 86400;
        let days_since_modified = (now_secs.saturating_sub(note.modified)) / 86400;
        let maturity = compute_maturity(inbound, days_since_created, days_since_modified);

        // Simplified stratum (1-8 based on word count + links)
        let stratum = compute_simple_stratum(note.word_count, note.outgoing_links.len(), inbound);

        note_meta.insert(key, (
            note.word_count,
            note.outgoing_links.len() as u32,
            maturity,
            Some(stratum),
            note.modified,
            note.name.clone(),
        ));
    }

    // Pass 2: Build the MapNode tree
    let mut tree = build_tree(root, &note_meta, 0, depth_limit);
    tree.node_type = "library".to_string();

    Ok(tree)
}

/// MIG-078 §A′ — when true, the universe Map/OrgChart tree is assembled from
/// the indexed `note_meta` records (the folder hierarchy is derived from each
/// note's path, a materialized-path column) with NO filesystem walk. This
/// removes the `build_tree` read_dir/stat walk that made the OrgChart/Map open
/// take minutes on a large universe (the disk walk's cost was exposed when the
/// incidental `loadData()` FS-cache warm-up was removed). Flip to `false` to
/// fall back to the legacy disk-walk path for one rollback cycle. The legacy
/// `build_tree`/`collect_notes_recursive` remain in use by
/// `constellation_map_data` (single-library command), so this const only
/// switches the universe-level path.
const MAP_TREE_FROM_INDEX: bool = true;

/// Build a library MapNode from a library path — reusable for both top-level and child universe libs.
///
/// `alias_to_path` is loaded once per command at the call-site (universe-
/// level entry point) and threaded down so every library inherits the
/// same alias view — see MIG-005 §1.
fn build_library_node(
    lib_path: &str,
    lib_name: &str,
    depth_limit: u32,
    alias_to_path: &HashMap<String, String>,
    db_records: &[NoteRecord],
) -> Option<MapNode> {
    let root = Path::new(lib_path);
    if !root.is_dir() { return None; }

    // MIG-077 perf — prefer the indexed note_meta records (db_records) over reading
    // every file on disk. Filter to this library's subtree. Fall back to the disk
    // walk only when the DB has no rows for it (federated child-universe libs whose
    // notes live in another DB, or a cold/empty index) — preserving correctness.
    let lib_prefix = format!("{}/", lib_path.replace('\\', "/").to_lowercase());
    let mut all_notes: Vec<NoteRecord> = db_records
        .iter()
        .filter(|r| r.path.replace('\\', "/").to_lowercase().starts_with(&lib_prefix))
        .cloned()
        .collect();
    // MIG-078 §A′ — the disk-walk fallback only runs in legacy mode. In
    // index mode an empty filtered set means the library simply has no
    // indexed notes (shown empty), never a 7,600-file content read.
    if !MAP_TREE_FROM_INDEX && all_notes.is_empty() {
        collect_notes_recursive(root, &mut all_notes);
    }

    // Build name ↔ path maps for 3-tier alias resolution (mirrors
    // constellation_map_data above).
    let mut name_to_path: HashMap<String, String> = HashMap::new();
    let mut path_to_name: HashMap<String, String> = HashMap::new();
    for note in &all_notes {
        let name_lower = note.name.to_lowercase();
        name_to_path.insert(name_lower.clone(), note.path.clone());
        path_to_name.insert(note.path.clone(), name_lower);
    }

    let mut inbound_map: HashMap<String, usize> = HashMap::new();
    for note in &all_notes {
        for target in &note.outgoing_links {
            let canonical_name = if name_to_path.contains_key(target) {
                target.clone()
            } else if let Some(canonical_path) = alias_to_path.get(target) {
                match path_to_name.get(canonical_path) {
                    Some(n) => n.clone(),
                    None => continue,
                }
            } else {
                continue;
            };
            *inbound_map.entry(canonical_name).or_insert(0) += 1;
        }
    }

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // MIG-008 Step 1: trailing String is the display name (frontmatter title
    // with file_stem fallback) that build_tree threads into MapNode.name so
    // the sunburst's hover/breadcrumb labels never expose the canonical
    // filename to the user.
    let mut note_meta: HashMap<String, (u32, u32, String, Option<u8>, u64, String)> = HashMap::new();
    for note in &all_notes {
        let key = note.path.replace('\\', "/").to_lowercase();
        let inbound = *inbound_map.get(&note.name.to_lowercase()).unwrap_or(&0);
        let days_since_created = (now_secs.saturating_sub(note.created)) / 86400;
        let days_since_modified = (now_secs.saturating_sub(note.modified)) / 86400;
        let maturity = compute_maturity(inbound, days_since_created, days_since_modified);
        let stratum = compute_simple_stratum(note.word_count, note.outgoing_links.len(), inbound);
        note_meta.insert(key, (
            note.word_count,
            note.outgoing_links.len() as u32,
            maturity,
            Some(stratum),
            note.modified,
            note.name.clone(),
        ));
    }

    let mut tree = if MAP_TREE_FROM_INDEX {
        // MIG-078 §A′ — assemble from the indexed records; no filesystem walk.
        build_tree_from_records(lib_path, &all_notes, &note_meta, depth_limit)
    } else {
        build_tree(root, &note_meta, 0, depth_limit)
    };
    tree.name = lib_name.to_string();
    tree.node_type = "library".to_string();
    Some(tree)
}

/// MIG-078 §A′ — Build a library's MapNode tree purely from indexed note
/// records (no filesystem walk). The folder hierarchy is derived from each
/// note's path (a materialized-path column in `note_meta`); subtree aggregates
/// roll up in one pass. Designed to produce output identical to `build_tree`
/// for the same note set, minus the disk I/O:
///   * depth cutoff mirrors `build_tree` (a note is included iff its
///     folder-depth from the library root is `< depth_limit`);
///   * children of each folder are sorted by their on-disk path segment
///     (folder name / note filename), mirroring `entries.sort_by(file_name)`;
///   * empty folders are excluded (a folder only exists if it contains a note);
///   * folder paths are reconstructed with the native separator (PathBuf),
///     and note paths are emitted verbatim from `note_meta`, matching the
///     native paths the disk walk produced.
fn build_tree_from_records(
    lib_path: &str,
    notes: &[NoteRecord],
    note_meta: &HashMap<String, (u32, u32, String, Option<u8>, u64, String)>,
    depth_limit: u32,
) -> MapNode {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    // Intermediate folder accumulator built from note paths.
    struct Folder {
        sub: BTreeMap<String, Folder>, // child folders keyed by path segment
        notes: Vec<(String, MapNode)>, // (filename segment, note node)
    }
    impl Folder {
        fn new() -> Self {
            Folder { sub: BTreeMap::new(), notes: Vec::new() }
        }
    }

    // Segment count of the library root — used to slice each note's path into
    // the part relative to the library (robust to case/separator differences,
    // and never panics on a UTF-8 boundary).
    let lib_seg_count = lib_path
        .split(|c| c == '/' || c == '\\')
        .filter(|s| !s.is_empty())
        .count();

    let mut root = Folder::new();

    for note in notes {
        let all_segs: Vec<&str> = note
            .path
            .split(|c| c == '/' || c == '\\')
            .filter(|s| !s.is_empty())
            .collect();
        if all_segs.len() <= lib_seg_count {
            continue; // not under this library (shouldn't happen post-filter)
        }
        let rel = &all_segs[lib_seg_count..];
        let folder_segs = &rel[..rel.len() - 1];
        let file_seg = rel[rel.len() - 1];

        // Depth cutoff identical to build_tree's `depth < max_depth` descent.
        if (folder_segs.len() as u32) >= depth_limit {
            continue;
        }

        let key = note.path.replace('\\', "/").to_lowercase();
        let (wc, lc, maturity, stratum, modified, note_name) = match note_meta.get(&key) {
            Some(t) => t.clone(),
            None => continue,
        };
        let weight = compute_weight(wc, lc, modified);
        let node = MapNode {
            name: note_name,
            path: note.path.clone(),
            is_dir: false,
            node_type: "note".to_string(),
            weight,
            note_count: 1,
            word_count: wc,
            link_count: lc,
            maturity: Some(maturity),
            stratum,
            modified: Some(modified),
            children: None,
        };

        let mut cur = &mut root;
        for seg in folder_segs {
            cur = cur.sub.entry((*seg).to_string()).or_insert_with(Folder::new);
        }
        cur.notes.push((file_seg.to_string(), node));
    }

    // Recursively materialize a Folder accumulator into a MapNode, rolling up
    // aggregates and sorting children by their on-disk segment name.
    fn convert(folder: &Folder, dir_path: &str, dir_name: &str) -> MapNode {
        let mut children: Vec<(String, MapNode)> = Vec::new();
        let mut total_weight: f64 = 0.0;
        let mut total_notes: u32 = 0;
        let mut total_words: u32 = 0;
        let mut total_links: u32 = 0;
        let mut latest_modified: u64 = 0;

        for (seg, sub) in &folder.sub {
            let mut child_pb = PathBuf::from(dir_path);
            child_pb.push(seg);
            let child_path = child_pb.to_string_lossy().to_string();
            let child = convert(sub, &child_path, seg);
            if child.note_count > 0 {
                total_weight += child.weight;
                total_notes += child.note_count;
                total_words += child.word_count;
                total_links += child.link_count;
                if child.modified.unwrap_or(0) > latest_modified {
                    latest_modified = child.modified.unwrap_or(0);
                }
                children.push((seg.clone(), child));
            }
        }
        for (seg, node) in &folder.notes {
            total_weight += node.weight;
            total_notes += node.note_count;
            total_words += node.word_count;
            total_links += node.link_count;
            if node.modified.unwrap_or(0) > latest_modified {
                latest_modified = node.modified.unwrap_or(0);
            }
            children.push((seg.clone(), node.clone()));
        }
        children.sort_by(|a, b| a.0.cmp(&b.0));
        let kids: Vec<MapNode> = children.into_iter().map(|(_, n)| n).collect();

        MapNode {
            name: dir_name.to_string(),
            path: dir_path.to_string(),
            is_dir: true,
            node_type: "folder".to_string(),
            weight: total_weight.max(0.1),
            note_count: total_notes,
            word_count: total_words,
            link_count: total_links,
            maturity: None,
            stratum: None,
            modified: if latest_modified > 0 { Some(latest_modified) } else { None },
            children: if kids.is_empty() { None } else { Some(kids) },
        }
    }

    let dir_name = Path::new(lib_path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| lib_path.to_string());
    convert(&root, lib_path, &dir_name)
}

/// Compute the Constellation Map tree for the entire universe (all libraries + child universes).
///
/// # Boot-path implication (Round 7 fix, 2026-04-19)
///
/// `+layout.svelte` keeps both the `<ConstellationMap>` overlay (line 4134)
/// and the `<OrgChart fullscreen>` overlay (line 4173) **always mounted**,
/// toggling visibility with a CSS class rather than `{#if}`. That pattern
/// is deliberate ("preserve drill-down state across navigation"), but it
/// means both components' `onMount` / `$effect` fire on every boot:
///
/// * `ConstellationMap.loadData()` → `invoke('constellation_map_universe')`
/// * `OrgChart.loadFullscreenData()` → `invoke('constellation_map_universe')`
///
/// With the sync `#[tauri::command]` binding, both calls ran inline on
/// the WebView2 UI thread. Round 6's arrival log showed the first call
/// held the dispatcher for 17.2 s and the second for 3.5 s — a 20.7-second
/// queue in front of `cache_boot_snapshot_core`. That fully explained
/// `core_queue_ms = 20,693`.
///
/// Converting to `#[tauri::command(async)]` routes through
/// `respond_async_serialized` → `tauri::async_runtime::spawn`, so each
/// dispatch runs on a Tokio worker. The body is unchanged — the map tree
/// is still computed with the same filesystem walk — but it no longer
/// serializes in front of the core snapshot.
///
/// Rule 8 follow-up (tracked separately): the correct long-term fix is
/// to (a) gate both overlays with `{#if}` so the walk only runs when the
/// user opens the Map/OrgChart, and (b) persist the derived map tree,
/// maintained by triggers on note-save, so even an explicit open is
/// instant. Neither is needed to close Criterion 2.
#[tauri::command(async)]
pub fn constellation_map_universe(
    app: tauri::AppHandle,
    universe_name: String,
    max_depth: Option<u32>,
) -> Result<MapNode, String> {
    let libraries = crate::libraries::load_all_libraries(&app);
    let depth_limit = max_depth.unwrap_or(5);

    // MIG-005 §1: load the alias resolution map once for the whole
    // universe walk. Threaded down into every per-library
    // build_library_node call so they all see the same alias view.
    let alias_to_path = load_alias_map(&app);

    // MIG-077 perf — load all note records from the index ONCE (was: each
    // build_library_node re-read every file on disk; measured tens of seconds on a
    // 7,600-note universe, on every open + every reload). Threaded into each
    // build_library_node; empty => that library falls back to the disk walk.
    let mut db_records = load_note_records(&app);

    // Get child universes
    let child_universes = crate::universe::get_child_universes(app.clone()).unwrap_or_default();

    // Load note records from each child universe's own search.db so their
    // libraries don't fall back to the slow disk walk.
    for cu in &child_universes {
        let cu_records = load_note_records_for_child_universe(&cu.path);
        db_records.extend(cu_records);
    }

    // Collect library paths that belong to child universes (to exclude from top-level)
    let mut child_lib_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Build child universe nodes
    let mut top_children: Vec<MapNode> = Vec::new();
    let mut total_weight: f64 = 0.0;
    let mut total_notes: u32 = 0;
    let mut total_words: u32 = 0;
    let mut total_links: u32 = 0;
    let mut latest_modified: u64 = 0;

    for cu in &child_universes {
        // Get libraries in this child universe
        let cu_libs = crate::universe::read_child_universe_libraries(app.clone(), cu.path.clone())
            .unwrap_or_default();

        let mut cu_lib_nodes: Vec<MapNode> = Vec::new();
        let mut cu_weight: f64 = 0.0;
        let mut cu_notes: u32 = 0;
        let mut cu_words: u32 = 0;
        let mut cu_links: u32 = 0;
        let mut cu_modified: u64 = 0;

        for lib in &cu_libs {
            child_lib_paths.insert(lib.path.replace('\\', "/").to_lowercase());
            if let Some(node) = build_library_node(&lib.path, &lib.name, depth_limit, &alias_to_path, &db_records) {
                cu_weight += node.weight;
                cu_notes += node.note_count;
                cu_words += node.word_count;
                cu_links += node.link_count;
                if node.modified.unwrap_or(0) > cu_modified {
                    cu_modified = node.modified.unwrap_or(0);
                }
                cu_lib_nodes.push(node);
            }
        }

        if !cu_lib_nodes.is_empty() {
            total_weight += cu_weight;
            total_notes += cu_notes;
            total_words += cu_words;
            total_links += cu_links;
            if cu_modified > latest_modified { latest_modified = cu_modified; }

            top_children.push(MapNode {
                name: cu.name.clone(),
                path: cu.path.clone(),
                is_dir: true,
                node_type: "child_universe".to_string(),
                weight: cu_weight.max(0.1),
                note_count: cu_notes,
                word_count: cu_words,
                link_count: cu_links,
                maturity: None,
                stratum: None,
                modified: if cu_modified > 0 { Some(cu_modified) } else { None },
                children: Some(cu_lib_nodes),
            });
        }
    }

    // Build top-level library nodes (excluding those in child universes)
    for lib in &libraries {
        let key = lib.path.replace('\\', "/").to_lowercase();
        if child_lib_paths.contains(&key) { continue; }

        if let Some(node) = build_library_node(&lib.path, &lib.name, depth_limit, &alias_to_path, &db_records) {
            total_weight += node.weight;
            total_notes += node.note_count;
            total_words += node.word_count;
            total_links += node.link_count;
            if node.modified.unwrap_or(0) > latest_modified {
                latest_modified = node.modified.unwrap_or(0);
            }
            top_children.push(node);
        }
    }

    if top_children.is_empty() {
        return Err("No libraries found.".to_string());
    }

    Ok(MapNode {
        name: universe_name,
        path: String::new(),
        is_dir: true,
        node_type: "universe".to_string(),
        weight: total_weight.max(0.1),
        note_count: total_notes,
        word_count: total_words,
        link_count: total_links,
        maturity: None,
        stratum: None,
        modified: if latest_modified > 0 { Some(latest_modified) } else { None },
        children: Some(top_children),
    })
}

/// Recursively collect all .md notes with word count, links, and timestamps.
fn collect_notes_recursive(dir: &Path, notes: &mut Vec<NoteRecord>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }

        if path.is_dir() {
            collect_notes_recursive(&path, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                // MIG-008 Step 1: prefer the human title over the canonical
                // filename so the Map's hover labels read "Apple Tree Fruit"
                // instead of "20260426T140737Z_NOTE_E561".
                let note_name = crate::libraries::note_display_name(&path, Some(&content));

                // Strip frontmatter for word counting
                let body = strip_frontmatter_pub(&content);

                // Count words (split on whitespace)
                let word_count = body.split_whitespace().count() as u32;

                // Extract outgoing wikilinks
                let wiki_re = regex::Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
                let outgoing_links: Vec<String> = wiki_re
                    .captures_iter(body)
                    .map(|cap| cap[1].trim().to_lowercase())
                    .collect();

                // File timestamps
                let metadata = fs::metadata(&path).ok();
                let modified = metadata.as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let created = metadata.as_ref()
                    .and_then(|m| m.created().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(modified);

                notes.push(NoteRecord {
                    path: path.to_string_lossy().to_string(),
                    name: note_name,
                    word_count,
                    outgoing_links,
                    modified,
                    created,
                });
            }
        }
    }
}

/// Build the recursive MapNode tree from the filesystem.
///
/// MIG-008 Step 1: the trailing `String` in each note_meta tuple is the
/// display name produced by `note_display_name` at scan time (frontmatter
/// title with file_stem fallback). Used for `MapNode.name` so the
/// Constellation Map never shows raw canonical filenames in tooltips or
/// breadcrumbs.
fn build_tree(
    dir: &Path,
    note_meta: &HashMap<String, (u32, u32, String, Option<u8>, u64, String)>,
    depth: u32,
    max_depth: u32,
) -> MapNode {
    let dir_name = dir.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.to_string_lossy().to_string());

    let mut children: Vec<MapNode> = Vec::new();
    let mut total_weight: f64 = 0.0;
    let mut total_notes: u32 = 0;
    let mut total_words: u32 = 0;
    let mut total_links: u32 = 0;
    let mut latest_modified: u64 = 0;

    if depth < max_depth {
        if let Ok(read_dir) = fs::read_dir(dir) {
            let mut entries: Vec<_> = read_dir.flatten().collect();
            entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

            for entry in entries {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') { continue; }

                if path.is_dir() {
                    let child = build_tree(&path, note_meta, depth + 1, max_depth);
                    total_weight += child.weight;
                    total_notes += child.note_count;
                    total_words += child.word_count;
                    total_links += child.link_count;
                    if child.modified.unwrap_or(0) > latest_modified {
                        latest_modified = child.modified.unwrap_or(0);
                    }
                    // Only include non-empty folders
                    if child.note_count > 0 {
                        children.push(child);
                    }
                } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    let key = path.to_string_lossy().replace('\\', "/").to_lowercase();

                    if let Some((wc, lc, maturity, stratum, modified, note_name)) = note_meta.get(&key) {
                        let note_name = note_name.clone();
                        let weight = compute_weight(*wc, *lc, *modified);
                        total_weight += weight;
                        total_notes += 1;
                        total_words += wc;
                        total_links += lc;
                        if *modified > latest_modified {
                            latest_modified = *modified;
                        }

                        children.push(MapNode {
                            name: note_name,
                            path: path.to_string_lossy().to_string(),
                            is_dir: false,
                            node_type: "note".to_string(),
                            weight,
                            note_count: 1,
                            word_count: *wc,
                            link_count: *lc,
                            maturity: Some(maturity.clone()),
                            stratum: *stratum,
                            modified: Some(*modified),
                            children: None,
                        });
                    }
                }
            }
        }
    }

    MapNode {
        name: dir_name,
        path: dir.to_string_lossy().to_string(),
        is_dir: true,
        node_type: "folder".to_string(),
        weight: total_weight.max(0.1),
        note_count: total_notes,
        word_count: total_words,
        link_count: total_links,
        maturity: None,
        stratum: None,
        modified: if latest_modified > 0 { Some(latest_modified) } else { None },
        children: if children.is_empty() { None } else { Some(children) },
    }
}

/// Compute knowledge weight for a single note.
fn compute_weight(word_count: u32, link_count: u32, modified: u64) -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days_since = (now.saturating_sub(modified)) as f64 / 86400.0;

    let word_score = (word_count as f64 / 500.0).min(5.0);
    let link_score = (link_count as f64 / 3.0).min(5.0);
    let recency_score = (1.0 - days_since / 365.0).max(0.0);

    (word_score + link_score + recency_score).max(0.1)
}

/// Maturity state (reuses logic from maturity.rs).
fn compute_maturity(inbound: usize, days_since_created: u64, days_since_modified: u64) -> String {
    if inbound >= 10 && days_since_modified >= 30 {
        return "canonical".to_string();
    }
    if inbound >= 4 && days_since_created >= 7 && days_since_modified >= 90 {
        return "wilting".to_string();
    }
    if inbound >= 4 && days_since_created >= 7 {
        return "evergreen".to_string();
    }
    if inbound >= 1 || days_since_created >= 2 {
        return "sapling".to_string();
    }
    "seed".to_string()
}

/// Simplified stratum (1-8) based on word count + link count.
fn compute_simple_stratum(word_count: u32, outgoing: usize, inbound: usize) -> u8 {
    let total_links = outgoing + inbound;
    if word_count > 2000 && total_links > 15 { return 8; }
    if word_count > 1500 && total_links > 10 { return 7; }
    if word_count > 1000 && total_links > 8 { return 6; }
    if word_count > 500 && total_links > 5 { return 5; }
    if word_count > 300 && total_links > 3 { return 4; }
    if word_count > 200 && total_links > 1 { return 3; }
    if word_count > 50 { return 2; }
    1
}
