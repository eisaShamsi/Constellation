//! Constellation Super Multilingual Search Engine — Phase 1: FTS5 + Structured Queries.
//!
//! Architecture:
//! - SQLite database at {universe}/.constellation/search.db
//! - FTS5 virtual table for lexical search (BM25 ranking)
//! - Metadata table for structured queries (properties, tags, wikilinks)
//! - Arabic Light10 stemming reused from libraries.rs
//! - Incremental indexing via file watcher events
//!
//! Phase 2 will add: ONNX embeddings, semantic search, RRF fusion.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

// ─── Types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: Option<String>,
    pub mode: String,           // "lexical" | "structured" | "hybrid" (future)
    pub filters: Option<SearchFilters>,
    pub limit: Option<u32>,
    pub include_snippet: Option<bool>,
    pub include_headings: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub properties: Option<Vec<PropertyFilter>>,
    pub tags: Option<Vec<String>>,
    pub wikilinks_to: Option<Vec<String>>,
    pub wikilinks_from: Option<Vec<String>>,
    pub library_names: Option<Vec<String>>,
    pub maturity: Option<Vec<String>>,
    pub path_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PropertyFilter {
    pub key: String,
    pub op: String,     // "=" | "!=" | "contains" | "is_empty"
    pub value: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub library_name: String,
    pub score: f64,
    pub match_type: String,  // "title" | "content" | "property" | "tag" | "wikilink"
    pub snippet: Option<String>,
    pub heading_breadcrumb: Option<Vec<String>>,
    pub modified: u64,
}

#[derive(Debug, Serialize)]
pub struct SearchIndexStats {
    pub note_count: u32,
    pub index_size_bytes: u64,
}

// ─── State ─────────────────────────────────────────────────────

pub struct SearchState {
    pub db: Mutex<Option<Connection>>,
}

impl SearchState {
    pub fn new() -> Self {
        SearchState { db: Mutex::new(None) }
    }
}

// ─── Database Setup ────────────────────────────────────────────

fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    Ok(cdir.join("search.db"))
}

fn init_db(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| format!("Failed to open search.db: {}", e))?;

    // Enable WAL mode for concurrent reads
    conn.execute_batch("PRAGMA journal_mode=WAL;").map_err(|e| e.to_string())?;

    // Create metadata table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS note_meta (
            path TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            library_name TEXT NOT NULL,
            modified INTEGER NOT NULL,
            content_hash TEXT,
            properties_json TEXT DEFAULT '{}',
            tags_json TEXT DEFAULT '[]',
            outgoing_links_json TEXT DEFAULT '[]',
            headings_json TEXT DEFAULT '[]',
            body_text TEXT DEFAULT ''
        );
    ").map_err(|e| format!("Failed to create note_meta: {}", e))?;

    // Create FTS5 virtual table for full-text search
    conn.execute_batch("
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            name,
            body_text,
            content=note_meta,
            content_rowid=rowid,
            tokenize='unicode61 remove_diacritics 2'
        );
    ").map_err(|e| format!("Failed to create notes_fts: {}", e))?;

    // Triggers to keep FTS in sync with note_meta
    conn.execute_batch("
        CREATE TRIGGER IF NOT EXISTS note_meta_ai AFTER INSERT ON note_meta BEGIN
            INSERT INTO notes_fts(rowid, name, body_text) VALUES (new.rowid, new.name, new.body_text);
        END;
        CREATE TRIGGER IF NOT EXISTS note_meta_ad AFTER DELETE ON note_meta BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, name, body_text) VALUES('delete', old.rowid, old.name, old.body_text);
        END;
        CREATE TRIGGER IF NOT EXISTS note_meta_au AFTER UPDATE ON note_meta BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, name, body_text) VALUES('delete', old.rowid, old.name, old.body_text);
            INSERT INTO notes_fts(rowid, name, body_text) VALUES (new.rowid, new.name, new.body_text);
        END;
    ").map_err(|e| format!("Failed to create FTS triggers: {}", e))?;

    // Indexes for structured queries
    conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_note_library ON note_meta(library_name);
        CREATE INDEX IF NOT EXISTS idx_note_modified ON note_meta(modified);
    ").map_err(|e| format!("Failed to create indexes: {}", e))?;

    Ok(conn)
}

// ─── Indexing Pipeline ─────────────────────────────────────────

/// Parse frontmatter properties from YAML block.
fn parse_frontmatter(content: &str) -> (HashMap<String, String>, Vec<String>, String) {
    let mut properties = HashMap::new();
    let mut tags = Vec::new();
    let mut body = content.to_string();

    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let fm = &content[3..3 + end];
            body = content[3 + end + 3..].trim().to_string();

            let mut in_tags = false;
            for line in fm.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("tags:") {
                    in_tags = true;
                    // Inline tags: tags: [a, b] or tags: a, b
                    let val = trimmed[5..].trim();
                    if !val.is_empty() && !val.starts_with('[') {
                        for t in val.split(',') {
                            let t = t.trim().trim_matches(|c| c == '"' || c == '\'');
                            if !t.is_empty() { tags.push(t.to_lowercase()); }
                        }
                    }
                    continue;
                }
                if in_tags {
                    if trimmed.starts_with("- ") {
                        let tag = trimmed[2..].trim().to_lowercase();
                        if !tag.is_empty() { tags.push(tag); }
                    } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        in_tags = false;
                    }
                }
                if !in_tags && trimmed.contains(':') && !trimmed.starts_with('#') {
                    if let Some(idx) = trimmed.find(':') {
                        let key = trimmed[..idx].trim().to_string();
                        let val = trimmed[idx + 1..].trim().trim_matches('"').to_string();
                        if !key.is_empty() { properties.insert(key, val); }
                    }
                }
            }
        }
    }

    (properties, tags, body)
}

/// Extract outgoing wikilinks from note content.
fn extract_wikilinks(content: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]").unwrap();
    let mut links = Vec::new();
    for cap in re.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let target = m.as_str().trim().to_lowercase();
            if !target.is_empty() && !links.contains(&target) {
                links.push(target);
            }
        }
    }
    links
}

/// Extract headings from markdown content.
fn extract_headings(content: &str) -> Vec<String> {
    let mut headings = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim().to_string();
            if !heading.is_empty() { headings.push(heading); }
        }
    }
    headings
}

/// Strip markdown syntax for plain-text indexing.
fn strip_markdown(text: &str) -> String {
    let mut result = text.to_string();
    // Remove code blocks
    let code_re = regex::Regex::new(r"(?s)```.*?```").unwrap();
    result = code_re.replace_all(&result, " ").to_string();
    // Remove inline code
    let inline_re = regex::Regex::new(r"`[^`]+`").unwrap();
    result = inline_re.replace_all(&result, " ").to_string();
    // Remove wikilinks but keep text
    let wiki_re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]").unwrap();
    result = wiki_re.replace_all(&result, "$1").to_string();
    // Remove markdown links
    let link_re = regex::Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap();
    result = link_re.replace_all(&result, "$1").to_string();
    // Remove headings markers
    result = result.replace('#', " ");
    // Remove bold/italic markers
    result = result.replace("**", " ").replace("__", " ").replace('*', " ").replace('_', " ");
    result
}

/// Index a single note into the database.
fn index_note(conn: &Connection, note_path: &str, library_name: &str) -> Result<(), String> {
    let path = Path::new(note_path);
    if !path.exists() || path.extension().map(|e| e != "md").unwrap_or(true) {
        return Ok(());
    }

    let name = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let modified = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);

    // Check if already indexed with same modified time
    let existing_mod: Option<u64> = conn.query_row(
        "SELECT modified FROM note_meta WHERE path = ?1",
        params![note_path],
        |row| row.get(0),
    ).ok();

    if existing_mod == Some(modified) {
        return Ok(()); // Already up to date
    }

    let (properties, tags, body) = parse_frontmatter(&content);
    let wikilinks = extract_wikilinks(&content);
    let headings = extract_headings(&content);
    let plain_body = strip_markdown(&body);

    let props_json = serde_json::to_string(&properties).unwrap_or_default();
    let tags_json = serde_json::to_string(&tags).unwrap_or_default();
    let links_json = serde_json::to_string(&wikilinks).unwrap_or_default();
    let headings_json = serde_json::to_string(&headings).unwrap_or_default();

    // Upsert: delete old, insert new (triggers handle FTS sync)
    conn.execute("DELETE FROM note_meta WHERE path = ?1", params![note_path])
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO note_meta (path, name, library_name, modified, properties_json, tags_json, outgoing_links_json, headings_json, body_text)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![note_path, name, library_name, modified, props_json, tags_json, links_json, headings_json, plain_body],
    ).map_err(|e| format!("Failed to index note {}: {}", note_path, e))?;

    Ok(())
}

/// Index all notes in a library directory.
fn index_library_recursive(conn: &Connection, dir: &Path, library_name: &str, depth: u32) {
    if depth > 20 { return; }
    let read_dir = match std::fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            index_library_recursive(conn, &path, library_name, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let _ = index_note(conn, &path.to_string_lossy(), library_name);
        }
    }
}

// ─── Search Execution ──────────────────────────────────────────

/// Lexical search using FTS5 BM25 ranking.
fn lexical_search(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    // FTS5 query: match against name and body_text
    let fts_query = format!("{}*", query.replace('"', ""));

    let mut stmt = match conn.prepare(
        "SELECT note_meta.path, note_meta.name, note_meta.library_name, note_meta.modified,
                bm25(notes_fts, 10.0, 1.0) as score,
                snippet(notes_fts, 1, '<mark>', '</mark>', '...', 40) as snip
         FROM notes_fts
         JOIN note_meta ON notes_fts.rowid = note_meta.rowid
         WHERE notes_fts MATCH ?1
         ORDER BY score
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let results = stmt.query_map(params![fts_query, limit], |row| {
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: row.get::<_, f64>(4)?.abs(), // BM25 returns negative (lower = better)
            snippet: row.get(5).ok(),
            match_type: "content".to_string(),
            heading_breadcrumb: None,
        })
    }).ok();

    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

/// Structured filter search (properties, tags, wikilinks).
fn structured_search(conn: &Connection, filters: &SearchFilters, limit: u32) -> Vec<SearchResult> {
    let mut conditions = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    // Property filters
    if let Some(props) = &filters.properties {
        for pf in props {
            match pf.op.as_str() {
                "=" => {
                    conditions.push(format!("json_extract(properties_json, '$.{}') = ?", pf.key));
                    params_vec.push(Box::new(pf.value.clone().unwrap_or_default()));
                }
                "!=" => {
                    conditions.push(format!("(json_extract(properties_json, '$.{}') IS NULL OR json_extract(properties_json, '$.{}') != ?)", pf.key, pf.key));
                    params_vec.push(Box::new(pf.value.clone().unwrap_or_default()));
                }
                "contains" => {
                    conditions.push(format!("json_extract(properties_json, '$.{}') LIKE '%' || ? || '%'", pf.key));
                    params_vec.push(Box::new(pf.value.clone().unwrap_or_default()));
                }
                "is_empty" => {
                    conditions.push(format!("(json_extract(properties_json, '$.{}') IS NULL OR json_extract(properties_json, '$.{}') = '')", pf.key, pf.key));
                }
                _ => {}
            }
        }
    }

    // Tag filters
    if let Some(tags) = &filters.tags {
        for tag in tags {
            conditions.push("tags_json LIKE '%' || ? || '%'".to_string());
            params_vec.push(Box::new(tag.to_lowercase()));
        }
    }

    // Wikilink-to filters (find notes that link TO target)
    if let Some(targets) = &filters.wikilinks_to {
        for target in targets {
            conditions.push("outgoing_links_json LIKE '%' || ? || '%'".to_string());
            params_vec.push(Box::new(target.to_lowercase()));
        }
    }

    // Library filter
    if let Some(libs) = &filters.library_names {
        if !libs.is_empty() {
            let placeholders: Vec<String> = libs.iter().enumerate().map(|(_, _)| "?".to_string()).collect();
            conditions.push(format!("library_name IN ({})", placeholders.join(",")));
            for lib in libs {
                params_vec.push(Box::new(lib.clone()));
            }
        }
    }

    // Path prefix filter
    if let Some(prefix) = &filters.path_prefix {
        conditions.push("path LIKE ? || '%'".to_string());
        params_vec.push(Box::new(prefix.clone()));
    }

    if conditions.is_empty() {
        return Vec::new();
    }

    let where_clause = conditions.join(" AND ");
    let sql = format!(
        "SELECT path, name, library_name, modified FROM note_meta WHERE {} ORDER BY modified DESC LIMIT {}",
        where_clause, limit
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => { eprintln!("[Search] SQL error: {}", e); return Vec::new(); }
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let results = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            match_type: "structured".to_string(),
            snippet: None,
            heading_breadcrumb: None,
        })
    }).ok();

    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

// ─── Tauri Commands ────────────────────────────────────────────

/// Initialize the search index — builds/rebuilds the SQLite database.
#[tauri::command]
pub fn constellation_search_init(app: tauri::AppHandle) -> Result<SearchIndexStats, String> {
    let path = db_path(&app)?;
    let conn = init_db(&path)?;

    // Index all libraries
    let libraries = crate::libraries::load_all_libraries(&app);
    for lib in &libraries {
        index_library_recursive(&conn, Path::new(&lib.path), &lib.name, 0);
    }

    let note_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM note_meta", [], |row| row.get(0)
    ).unwrap_or(0);

    let index_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    // Store connection in app state
    let state = app.state::<SearchState>();
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    *db = Some(conn);

    Ok(SearchIndexStats { note_count, index_size_bytes: index_size })
}

/// Reindex a single note (called on file change).
#[tauri::command]
pub fn constellation_search_reindex(
    app: tauri::AppHandle,
    note_path: String,
    library_name: String,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db.as_ref() {
        index_note(conn, &note_path, &library_name)?;
    }
    Ok(())
}

/// Main search command — supports lexical, structured, and combined modes.
#[tauri::command]
pub fn constellation_search(
    app: tauri::AppHandle,
    request: SearchRequest,
) -> Result<Vec<SearchResult>, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;

    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => {
            // Fallback: initialize on first search
            drop(db_guard);
            constellation_search_init(app.clone())?;
            let state = app.state::<SearchState>();
            let db_guard = state.db.lock().map_err(|e| e.to_string())?;
            return match db_guard.as_ref() {
                Some(c) => execute_search(c, &request),
                None => Err("Search index not available".to_string()),
            };
        }
    };

    execute_search(conn, &request)
}

fn execute_search(conn: &Connection, request: &SearchRequest) -> Result<Vec<SearchResult>, String> {
    let limit = request.limit.unwrap_or(50);
    let mut results = Vec::new();

    match request.mode.as_str() {
        "lexical" => {
            if let Some(q) = &request.query {
                if !q.trim().is_empty() {
                    results = lexical_search(conn, q, limit);
                }
            }
        }
        "structured" => {
            if let Some(filters) = &request.filters {
                results = structured_search(conn, filters, limit);
            }
        }
        "hybrid" | _ => {
            // Combine lexical + structured
            let mut lexical_results = Vec::new();
            let mut structured_results = Vec::new();

            if let Some(q) = &request.query {
                if !q.trim().is_empty() {
                    lexical_results = lexical_search(conn, q, limit);
                }
            }

            if let Some(filters) = &request.filters {
                structured_results = structured_search(conn, filters, limit);
            }

            // Merge: deduplicate by path, prefer higher score
            let mut seen = std::collections::HashSet::new();
            for r in lexical_results {
                if seen.insert(r.path.clone()) { results.push(r); }
            }
            for r in structured_results {
                if seen.insert(r.path.clone()) { results.push(r); }
            }
        }
    }

    // Apply limit
    results.truncate(limit as usize);
    Ok(results)
}
