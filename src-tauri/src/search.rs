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
    pub query_embedding: Option<Vec<f32>>,  // pre-computed embedding for semantic search
    pub mode: String,           // "lexical" | "structured" | "semantic" | "hybrid"
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
    pub mutual: Option<Vec<String>>,
    pub mentions: Option<Vec<String>>,
    pub orphans: Option<bool>,
    pub links_between: Option<Vec<String>>,  // exactly 2 targets
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

    // Create embeddings table for semantic search (Phase 2)
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS note_embeddings (
            path TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            dimensions INTEGER NOT NULL DEFAULT 384,
            model_id TEXT DEFAULT 'all-MiniLM-L6-v2'
        );
    ").map_err(|e| format!("Failed to create note_embeddings: {}", e))?;

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
                    if !val.is_empty() {
                        // Strip brackets for [a, b] format
                        let val = val.trim_start_matches('[').trim_end_matches(']');
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

    // Also extract inline #hashtags from body text
    let tag_re = regex::Regex::new(r"(?:^|\s)#([\w\p{L}\p{N}_/-]+)").unwrap();
    for cap in tag_re.captures_iter(&body) {
        if let Some(m) = cap.get(1) {
            let tag = m.as_str().trim().to_lowercase();
            if !tag.is_empty() && !tags.contains(&tag) {
                tags.push(tag);
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

/// Arabic text normalization for consistent FTS matching.
/// Removes diacritics, normalizes Alef variants, Teh marbuta, Alef maqsura.
fn normalize_arabic_for_search(text: &str) -> String {
    text.chars().filter_map(|c| {
        // Remove diacritics (tashkeel)
        if ('\u{064B}'..='\u{065F}').contains(&c) || c == '\u{0670}'
            || ('\u{06D6}'..='\u{06ED}').contains(&c) { return None; }
        // Remove tatweel
        if c == '\u{0640}' { return None; }
        // Normalize Alef variants → ا
        if c == 'أ' || c == 'إ' || c == 'آ' || c == '\u{0671}' { return Some('ا'); }
        // Alef maqsura → ي
        if c == 'ى' { return Some('ي'); }
        // Teh marbuta → ه
        if c == 'ة' { return Some('ه'); }
        Some(c)
    }).collect()
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

    // Arabic normalization for better FTS matching (Phase 4)
    // Normalize diacritics, Alef variants, Teh marbuta for consistent indexing
    let plain_body = normalize_arabic_for_search(&plain_body);
    let name = normalize_arabic_for_search(&name);

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
    // Normalize query for Arabic consistency (same normalization as indexed text)
    let normalized = normalize_arabic_for_search(query);
    let fts_query = format!("{}*", normalized.replace('"', ""));

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

    let query_lower = normalized.to_lowercase();

    let results = stmt.query_map(params![fts_query, limit], |row| {
        let name: String = row.get(1)?;
        let name_lower = name.to_lowercase();
        let title_hit = name_lower.contains(&query_lower);
        let snippet: Option<String> = row.get(5).ok();
        let body_hit = snippet.as_ref().map_or(false, |s| s.contains("<mark>"));

        let match_type = if title_hit && body_hit {
            "title".to_string() // prioritize title when both match
        } else if title_hit {
            "title".to_string()
        } else {
            "content".to_string()
        };

        Ok(SearchResult {
            path: row.get(0)?,
            name,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: row.get::<_, f64>(4)?.abs(),
            snippet,
            match_type,
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

    // Tag filters — JSON-quoted match for exact tag element
    if let Some(tags) = &filters.tags {
        for tag in tags {
            conditions.push("tags_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(tag.to_lowercase()));
        }
    }

    // Wikilink-to filters (find notes that link TO target) — JSON-quoted exact match
    if let Some(targets) = &filters.wikilinks_to {
        for target in targets {
            conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target.to_lowercase()));
        }
    }

    // Wikilink-from filters: find notes that X links TO (outgoing links of X)
    // This is a two-step query: first find X's outgoing links, then return those notes
    let mut from_targets: Vec<String> = Vec::new();
    if let Some(sources) = &filters.wikilinks_from {
        for source in sources {
            let source_lower = source.to_lowercase();
            // Find the note named `source` and read its outgoing_links_json
            let links: Option<String> = conn.query_row(
                "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                params![source_lower],
                |row| row.get(0),
            ).ok();
            if let Some(links_json) = links {
                if let Ok(targets) = serde_json::from_str::<Vec<String>>(&links_json) {
                    from_targets.extend(targets);
                }
            }
        }
    }
    if !from_targets.is_empty() {
        let placeholders: Vec<String> = from_targets.iter().map(|_| "LOWER(name) = ?".to_string()).collect();
        conditions.push(format!("({})", placeholders.join(" OR ")));
        for t in &from_targets {
            params_vec.push(Box::new(t.clone()));
        }
    }

    // Mutual filters: notes that link to X AND X links to them
    if let Some(targets) = &filters.mutual {
        for target in targets {
            let target_lower = target.to_lowercase();
            // Must link TO target
            conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target_lower.clone()));
            // AND target must link back (find target's outgoing links, filter to those)
            let links: Option<String> = conn.query_row(
                "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                params![target_lower],
                |row| row.get(0),
            ).ok();
            if let Some(links_json) = links {
                if let Ok(back_targets) = serde_json::from_str::<Vec<String>>(&links_json) {
                    if !back_targets.is_empty() {
                        let placeholders: Vec<String> = back_targets.iter().map(|_| "LOWER(name) = ?".to_string()).collect();
                        conditions.push(format!("({})", placeholders.join(" OR ")));
                        for bt in &back_targets {
                            params_vec.push(Box::new(bt.clone()));
                        }
                    }
                }
            }
        }
    }

    // Mentions filter: notes that contain X's name in body but do NOT have [[X]] wikilink
    if let Some(names) = &filters.mentions {
        for name in names {
            let name_lower = name.to_lowercase();
            conditions.push("LOWER(body_text) LIKE '%' || ? || '%'".to_string());
            params_vec.push(Box::new(name_lower.clone()));
            conditions.push("outgoing_links_json NOT LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(name_lower.clone()));
            // Exclude the note itself
            conditions.push("LOWER(name) != ?".to_string());
            params_vec.push(Box::new(name_lower));
        }
    }

    // Orphans filter: notes with no incoming or outgoing links
    if filters.orphans.unwrap_or(false) {
        // No outgoing links
        conditions.push("(outgoing_links_json IS NULL OR outgoing_links_json = '[]')".to_string());
        // No incoming links — use JSON-quoted match to avoid substring false positives
        // Links stored as: ["note1","note2"] — match "\"name\"" for exact element
        conditions.push("NOT EXISTS (SELECT 1 FROM note_meta AS other WHERE other.outgoing_links_json LIKE '%\"' || LOWER(note_meta.name) || '\"%' AND other.path != note_meta.path)".to_string());
    }

    // Links-between filter: notes that link to BOTH X and Y
    if let Some(targets) = &filters.links_between {
        for target in targets {
            conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
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

    // Determine the dominant filter type for match_type coloring
    let dominant_type = if filters.tags.as_ref().map_or(false, |t| !t.is_empty()) {
        "tag"
    } else if filters.wikilinks_to.as_ref().map_or(false, |w| !w.is_empty())
           || filters.wikilinks_from.as_ref().map_or(false, |w| !w.is_empty())
           || filters.mutual.as_ref().map_or(false, |w| !w.is_empty())
           || filters.links_between.as_ref().map_or(false, |w| !w.is_empty()) {
        "wikilink"
    } else if filters.mentions.as_ref().map_or(false, |m| !m.is_empty()) {
        "content"
    } else if filters.orphans.unwrap_or(false) {
        "structured"
    } else if filters.properties.as_ref().map_or(false, |p| !p.is_empty()) {
        "property"
    } else {
        "structured"
    };

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
    let mt = dominant_type.to_string();

    let results = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            match_type: mt.clone(),
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

    // Schema v2: force full reindex to pick up bracket-format tags + inline hashtags
    // Check for version marker; if missing or outdated, delete and rebuild
    let version_path = path.with_extension("version");
    let current_version = "3"; // v3: exact JSON-quoted tag/link matching, inline hashtags, bracket tags
    let needs_rebuild = match std::fs::read_to_string(&version_path) {
        Ok(v) => v.trim() != current_version,
        Err(_) => true,
    };
    if needs_rebuild {
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::write(&version_path, current_version);
    }

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
    let limit = if request.limit.unwrap_or(0) == 0 { 100000 } else { request.limit.unwrap() };
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
        "semantic" => {
            // Semantic-only search using stored embeddings
            if let Some(q_embedding) = request.query_embedding.as_ref() {
                results = semantic_search(conn, q_embedding, limit);
            }
        }
        "hybrid" | _ => {
            // Full hybrid: RRF fusion of lexical + semantic + structured
            let mut lexical_results = Vec::new();
            let mut semantic_results = Vec::new();
            let mut structured_results = Vec::new();

            if let Some(q) = &request.query {
                if !q.trim().is_empty() {
                    lexical_results = lexical_search(conn, q, limit * 2);
                }
            }

            if let Some(q_embedding) = request.query_embedding.as_ref() {
                semantic_results = semantic_search(conn, q_embedding, limit * 2);
            }

            if let Some(filters) = &request.filters {
                structured_results = structured_search(conn, filters, limit);
            }

            // RRF fusion: score(d) = Σ 1/(k + rank_i(d)), k=60
            if !lexical_results.is_empty() || !semantic_results.is_empty() {
                results = rrf_fuse(lexical_results, semantic_results, 60);
            }

            // Merge structured results (they're filter-based, not ranked)
            let seen: std::collections::HashSet<String> = results.iter().map(|r| r.path.clone()).collect();
            for r in structured_results {
                if !seen.contains(&r.path) { results.push(r); }
            }
        }
    }

    results.truncate(limit as usize);
    Ok(results)
}

// ─── Semantic Search ───────────────────────────────────────────

/// Cosine similarity between two float vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() { return 0.0; }
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

/// Search for notes similar to a query embedding using cosine similarity.
fn semantic_search(conn: &Connection, query_embedding: &[f32], limit: u32) -> Vec<SearchResult> {
    // Load all embeddings and compute similarity
    let mut stmt = match conn.prepare(
        "SELECT e.path, m.name, m.library_name, m.modified, e.embedding, e.dimensions
         FROM note_embeddings e
         JOIN note_meta m ON e.path = m.path"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut scored: Vec<(SearchResult, f32)> = Vec::new();

    let rows = stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let name: String = row.get(1)?;
        let library_name: String = row.get(2)?;
        let modified: u64 = row.get(3)?;
        let embedding_blob: Vec<u8> = row.get(4)?;
        let dimensions: usize = row.get::<_, u32>(5)? as usize;

        // Convert blob to f32 vector (safe: skip malformed blobs)
        if embedding_blob.len() % 4 != 0 || embedding_blob.len() / 4 < dimensions {
            return Ok(("".to_string(), "".to_string(), "".to_string(), 0, Vec::new()));
        }
        let embedding: Vec<f32> = embedding_blob
            .chunks_exact(4)
            .take(dimensions)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok((path, name, library_name, modified, embedding))
    }).ok();

    if let Some(rows) = rows {
        for row in rows.flatten() {
            let (path, name, library_name, modified, embedding) = row;
            let sim = cosine_similarity(query_embedding, &embedding);
            if sim > 0.3 { // minimum threshold
                scored.push((SearchResult {
                    path, name, library_name, modified,
                    score: sim as f64,
                    match_type: "semantic".to_string(),
                    snippet: None,
                    heading_breadcrumb: None,
                }, sim));
            }
        }
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit as usize);
    scored.into_iter().map(|(r, _)| r).collect()
}

/// Reciprocal Rank Fusion: merges two ranked result lists.
/// RRF_score(d) = Σ 1/(k + rank_i(d))
fn rrf_fuse(list_a: Vec<SearchResult>, list_b: Vec<SearchResult>, k: u32) -> Vec<SearchResult> {
    let mut scores: HashMap<String, (f64, SearchResult)> = HashMap::new();

    for (rank, r) in list_a.into_iter().enumerate() {
        let rrf = 1.0 / (k as f64 + rank as f64 + 1.0);
        scores.entry(r.path.clone()).or_insert((0.0, r)).0 += rrf;
    }

    for (rank, r) in list_b.into_iter().enumerate() {
        let rrf = 1.0 / (k as f64 + rank as f64 + 1.0);
        let path = r.path.clone();
        let mt = r.match_type.clone();
        let entry = scores.entry(path).or_insert((0.0, r));
        entry.0 += rrf;
        if entry.1.match_type == "content" && mt == "semantic" {
            entry.1.match_type = "hybrid".to_string();
        }
    }

    let mut fused: Vec<SearchResult> = scores.into_iter().map(|(_, (score, mut r))| {
        r.score = score;
        r
    }).collect();

    fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    fused
}

// ─── Embedding Storage Commands ────────────────────────────────

/// Store a pre-computed embedding vector for a note (called from JS semantic engine).
#[tauri::command]
pub fn constellation_search_store_embedding(
    app: tauri::AppHandle,
    note_path: String,
    embedding: Vec<f32>,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db.as_ref() {
        let dimensions = embedding.len() as u32;
        // Convert f32 vec to blob (little-endian bytes)
        let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        conn.execute(
            "INSERT OR REPLACE INTO note_embeddings (path, embedding, dimensions) VALUES (?1, ?2, ?3)",
            params![note_path, blob, dimensions],
        ).map_err(|e| format!("Failed to store embedding: {}", e))?;
    }
    Ok(())
}

/// Find notes semantically similar to a given note.
#[tauri::command]
pub fn constellation_search_similar(
    app: tauri::AppHandle,
    note_path: String,
    limit: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search index not initialized")?;

    // Get the note's embedding
    let embedding_blob: Vec<u8> = conn.query_row(
        "SELECT embedding FROM note_embeddings WHERE path = ?1",
        params![note_path],
        |row| row.get(0),
    ).map_err(|_| "Note has no embedding".to_string())?;

    let query_embedding: Vec<f32> = embedding_blob
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    let mut results = semantic_search(conn, &query_embedding, limit.unwrap_or(20));
    // Remove the query note itself from results
    results.retain(|r| r.path != note_path);
    Ok(results)
}

// ─── Universal Categorized Search ─────────────────────────────

#[derive(Debug, Serialize)]
pub struct UniversalSearchResponse {
    pub titles: Vec<SearchResult>,
    pub contents: Vec<SearchResult>,
    pub tags: Vec<SearchResult>,
    pub properties: Vec<SearchResult>,
    pub wikilinks: Vec<SearchResult>,
    pub semantic: Vec<SearchResult>,
}

#[tauri::command]
pub fn constellation_search_universal(
    app: tauri::AppHandle,
    query: String,
    query_embedding: Option<Vec<f32>>,
    limit: Option<u32>,
) -> Result<UniversalSearchResponse, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;

    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => {
            drop(db_guard);
            constellation_search_init(app.clone())?;
            let state = app.state::<SearchState>();
            let db_guard = state.db.lock().map_err(|e| e.to_string())?;
            return match db_guard.as_ref() {
                Some(c) => execute_universal_search(c, &query, query_embedding.as_deref(), if limit.unwrap_or(0) == 0 { 100000 } else { limit.unwrap() }),
                None => Err("Search index not available".to_string()),
            };
        }
    };

    execute_universal_search(conn, &query, query_embedding.as_deref(), if limit.unwrap_or(0) == 0 { 100000 } else { limit.unwrap() })
}

/// Split query by comma variants: , (Latin) ، (Arabic) 、(CJK)
fn split_multi_terms(query: &str) -> Vec<String> {
    query.split(|c| c == ',' || c == '،' || c == '、')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Deduplicate results by path, keeping highest score
fn dedup_results(mut results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut seen = std::collections::HashMap::new();
    let mut deduped = Vec::new();
    for r in results.drain(..) {
        let entry = seen.entry(r.path.clone()).or_insert(0.0_f64);
        if r.score > *entry {
            *entry = r.score;
            deduped.retain(|existing: &SearchResult| existing.path != r.path);
            deduped.push(r);
        }
    }
    deduped
}

fn execute_universal_search(conn: &Connection, query: &str, query_embedding: Option<&[f32]>, limit: u32) -> Result<UniversalSearchResponse, String> {
    let terms = split_multi_terms(query);

    let mut all_titles = Vec::new();
    let mut all_contents = Vec::new();
    let mut all_tags = Vec::new();
    let mut all_properties = Vec::new();
    let mut all_wikilinks = Vec::new();

    for term in &terms {
        let normalized = normalize_arabic_for_search(term);
        let raw_lower = term.to_lowercase();

        all_titles.extend(search_titles(conn, &normalized, limit));
        all_contents.extend(search_contents(conn, &normalized, limit));
        all_tags.extend(search_tags(conn, &raw_lower, limit));
        all_properties.extend(search_properties(conn, &raw_lower, limit));
        all_wikilinks.extend(search_wikilinks(conn, &raw_lower, limit));
    }

    // 6. SEMANTIC — cosine similarity on stored embeddings (if query embedding provided)
    let semantic = if let Some(qe) = query_embedding {
        let mut results = semantic_search(conn, qe, limit);
        results.truncate(limit as usize);
        results
    } else {
        Vec::new()
    };

    // Deduplicate and truncate
    let mut titles = dedup_results(all_titles); titles.truncate(limit as usize);
    let mut contents = dedup_results(all_contents); contents.truncate(limit as usize);
    let mut tags = dedup_results(all_tags); tags.truncate(limit as usize);
    let mut properties = dedup_results(all_properties); properties.truncate(limit as usize);
    let mut wikilinks = dedup_results(all_wikilinks); wikilinks.truncate(limit as usize);

    Ok(UniversalSearchResponse { titles, contents, tags, properties, wikilinks, semantic })
}

fn search_titles(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    let fts_query = format!("name:{}*", query.replace('"', ""));
    let mut stmt = match conn.prepare(
        "SELECT note_meta.path, note_meta.name, note_meta.library_name, note_meta.modified,
                bm25(notes_fts, 10.0, 0.0) as score
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
            score: row.get::<_, f64>(4)?.abs(),
            snippet: None,
            match_type: "title".to_string(),
            heading_breadcrumb: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn search_contents(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    let fts_query = format!("body_text:{}*", query.replace('"', ""));
    let mut stmt = match conn.prepare(
        "SELECT note_meta.path, note_meta.name, note_meta.library_name, note_meta.modified,
                bm25(notes_fts, 0.0, 1.0) as score,
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
            score: row.get::<_, f64>(4)?.abs(),
            snippet: row.get(5).ok(),
            match_type: "content".to_string(),
            heading_breadcrumb: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn search_tags(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    // Use JSON-quoted match for exact tag: "tagname" in the JSON array
    // This avoids substring false positives (e.g., "id" matching "video")
    let mut stmt = match conn.prepare(
        "SELECT path, name, library_name, modified, tags_json FROM note_meta
         WHERE tags_json LIKE '%\"' || ?1 || '\"%'
         ORDER BY modified DESC
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let results = stmt.query_map(params![query, limit], |row| {
        let tags_json: String = row.get(4)?;
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            snippet: Some(tags_json),
            match_type: "tag".to_string(),
            heading_breadcrumb: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn search_properties(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    let mut stmt = match conn.prepare(
        "SELECT path, name, library_name, modified, properties_json FROM note_meta
         WHERE properties_json LIKE '%' || ?1 || '%'
         ORDER BY modified DESC
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let results = stmt.query_map(params![query, limit], |row| {
        let props_json: String = row.get(4)?;
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            snippet: Some(props_json),
            match_type: "property".to_string(),
            heading_breadcrumb: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn search_wikilinks(conn: &Connection, query: &str, limit: u32) -> Vec<SearchResult> {
    let mut stmt = match conn.prepare(
        "SELECT path, name, library_name, modified, outgoing_links_json FROM note_meta
         WHERE outgoing_links_json LIKE '%\"' || ?1 || '\"%'
         ORDER BY modified DESC
         LIMIT ?2"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let results = stmt.query_map(params![query, limit], |row| {
        let links_json: String = row.get(4)?;
        Ok(SearchResult {
            path: row.get(0)?,
            name: row.get(1)?,
            library_name: row.get(2)?,
            modified: row.get(3)?,
            score: 1.0,
            snippet: Some(links_json),
            match_type: "wikilink".to_string(),
            heading_breadcrumb: None,
        })
    }).ok();
    results.map(|rows| rows.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}
