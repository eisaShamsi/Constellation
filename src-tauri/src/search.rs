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
use std::sync::{Arc, Mutex};
use tauri::Manager;

/// Schema version tracked via `PRAGMA user_version`.
///
/// Increment when the FTS5 tokenizer changes, the `notes_fts` / `notes_vocab`
/// schema changes, or any other setup in `init_db` requires a one-time
/// rebuild of derived data. On boot we drop + recreate the FTS5 chain if
/// the stored version is below this and then issue an `INSERT INTO
/// notes_fts(notes_fts) VALUES('rebuild')` so the new index populates
/// from the existing `note_meta` rows — no filesystem re-walk needed.
///
/// | version | change                                                       |
/// |--------:|--------------------------------------------------------------|
/// |       0 | legacy — notes_fts created with `tokenize='unicode61 ...'`   |
/// |       1 | custom Constellation tokenizer (Arabic Light10 + bigrams)    |
const FTS_SCHEMA_VERSION: i64 = 1;

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
    pub links_all: Option<Vec<String>>,     // incoming + outgoing combined
    pub typed_links: Option<Vec<TypedLinkFilter>>, // cognitive link type queries
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

#[derive(Debug, Deserialize)]
pub struct TypedLinkFilter {
    pub link_type: String,   // supports, contradicts, causes, etc.
    pub target: String,      // target note name
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

pub(crate) fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    Ok(cdir.join("search.db"))
}

/// Append a timestamped line to `<universe>/.constellation/diagnostics.log`.
///
/// Windows Tauri builds are compiled as GUI subsystem so `eprintln!` /
/// `println!` go nowhere even when launched from a terminal. Diagnostics
/// that must be visible after the fact (migration fired? which tokenizer
/// is active? how many `notes_vocab` rows?) therefore need a durable
/// sink the user can open in any editor. Takes the `search.db` path so
/// callers never need to know the Universe root.
///
/// Non-fatal: any failure is swallowed so diagnostics never break the
/// critical path. Also mirrored to `eprintln!` for dev builds where
/// stderr IS attached (e.g. `npm run tauri dev`).
pub(crate) fn diag_log(db_path: &Path, msg: &str) {
    // Still emit to stderr for dev builds and future console-subsystem binaries.
    eprintln!("{}", msg);
    let Some(parent) = db_path.parent() else { return; };
    let log_path = parent.join("diagnostics.log");
    let ts = {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    };
    let line = format!("[{}] {}\n", ts, msg);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        });
}

/// Register the custom Constellation FTS5 tokenizer on a connection.
///
/// Every connection that will run `MATCH` against `notes_fts` — or
/// `CREATE VIRTUAL TABLE ... tokenize='constellation'` — needs this
/// called once. Tokenizer registration is connection-local in SQLite
/// FTS5 (no global registry in the `bundled` build), so callers that
/// open their own connections (e.g. the read-only opens in
/// `libraries::read_index_entries` / `read_term_mentions`) must call
/// this before issuing queries.
///
/// Idempotent within a connection in the sense that repeated calls
/// with the same name register a second time (SQLite shadows the
/// earlier registration); but under normal flow each connection
/// should call this exactly once, right after opening.
pub(crate) fn register_fts5_tokenizer(conn: &mut Connection) -> Result<(), String> {
    let stopwords = Arc::new(crate::libraries::build_stopwords());
    crate::fts5_tokenizer::register_tokenizer::<
        crate::fts5_tokenizer::ConstellationTokenizer,
    >(
        conn,
        crate::fts5_tokenizer::ConstellationGlobal { stopwords },
        "constellation",
    )
}

fn init_db(path: &Path) -> Result<Connection, String> {
    let mut conn = Connection::open(path).map_err(|e| format!("Failed to open search.db: {}", e))?;

    // Enable WAL mode for concurrent reads
    conn.execute_batch("PRAGMA journal_mode=WAL;").map_err(|e| e.to_string())?;

    // ─── Register the custom FTS5 tokenizer ──────────────────────────
    // Must happen BEFORE any `CREATE VIRTUAL TABLE ... tokenize='constellation'`
    // so SQLite can resolve the tokenizer name. Safe to call on a
    // connection that has never seen FTS5 — it only wires up an
    // in-memory pointer on the connection; no DB state changes.
    register_fts5_tokenizer(&mut conn)?;

    // ─── FTS schema migration ────────────────────────────────────────
    // Old databases have `notes_fts` created with
    //   tokenize='unicode61 remove_diacritics 2'
    // `CREATE VIRTUAL TABLE IF NOT EXISTS` below would NOT change an
    // existing table's tokenizer — it silently skips. So if the stored
    // `PRAGMA user_version` is below the current FTS schema version we
    // drop the FTS5 chain, let the CREATE statements below rebuild it
    // with the new tokenizer, and then issue a `rebuild` command to
    // repopulate it from `note_meta` (no filesystem walk needed — FTS5
    // re-indexes from the content table). See FTS_SCHEMA_VERSION above
    // for the version ledger.
    let stored_version: i64 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| format!("PRAGMA user_version failed: {}", e))?;
    let needs_fts_rebuild = stored_version < FTS_SCHEMA_VERSION;
    diag_log(path, &format!(
        "[search] init_db: PRAGMA user_version={} (target {}) — rebuild {}",
        stored_version,
        FTS_SCHEMA_VERSION,
        if needs_fts_rebuild { "NEEDED (dropping notes_fts/notes_vocab)" } else { "skipped (already current)" },
    ));
    if needs_fts_rebuild {
        // Drop notes_vocab first (it depends on notes_fts). IF EXISTS so
        // this is a no-op on fresh DBs.
        conn.execute_batch("
            DROP TABLE IF EXISTS notes_vocab;
            DROP TABLE IF EXISTS notes_fts;
        ").map_err(|e| format!("Failed to drop old FTS chain during migration: {}", e))?;
    }

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

    // Create FTS5 virtual table for full-text search.
    //
    // Uses the custom 'constellation' tokenizer (registered above) so
    // the stored tokens are already stemmed forms:
    //   * Arabic Light10 collapses the ~452k surface forms observed on
    //     a 7,600-note Arabic-heavy Universe to ~30-60k stems.
    //   * Multi-language stemmers (Persian / Hebrew / Cyrillic /
    //     Devanagari / German / Spanish / Portuguese / French / Turkish /
    //     English) each collapse their own inflections.
    //   * Bigrams are emitted as colocated tokens, joined by the
    //     `fts5_tokenizer::BIGRAM_SEP` sentinel byte.
    //   * `MATCH` queries are stemmed through the same tokenizer, so
    //     `MATCH 'الكتاب'` and `MATCH 'كتب'` both land on the stem 'كتب'.
    conn.execute_batch("
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            name,
            body_text,
            content=note_meta,
            content_rowid=rowid,
            tokenize='constellation'
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

    // ─── Index Panel vocabulary view ─────────────────────────────────────
    // fts5vocab exposes the sorted term dictionary that FTS5 already
    // maintains on-disk as triggers update `notes_fts`. Row mode:
    //   (term TEXT, doc INTEGER, cnt INTEGER)
    //   * doc — number of distinct notes containing the term
    //   * cnt — total occurrences across all notes
    // This replaces the custom index_terms/index_mentions/index_meta tables:
    // the Index panel reads directly from the already-maintained FTS5 index,
    // no separate tokenization or aggregation pass is needed, and edits to
    // notes update the vocab transparently via the existing FTS5 triggers.
    conn.execute_batch("
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_vocab USING fts5vocab(notes_fts, 'row');
    ").map_err(|e| format!("Failed to create notes_vocab: {}", e))?;

    // Indexes for structured queries
    conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_note_library ON note_meta(library_name);
        CREATE INDEX IF NOT EXISTS idx_note_modified ON note_meta(modified);
        CREATE INDEX IF NOT EXISTS idx_note_name ON note_meta(name);
    ").map_err(|e| format!("Failed to create indexes: {}", e))?;

    // Covering index for the boot-path projection:
    //   SELECT name, path, library_name FROM note_meta
    // Without this, SQLite does a full table scan and reads the wide
    // rows (body_text + *_json blobs, ~80 MB on a 7,600-note Universe)
    // just to project three narrow TEXT columns. With the covering
    // index, the planner does an index-only scan over ~200 KB of index
    // pages. Measured 2026-04-16: brings `read_notes` from 8021 ms to
    // low-millis on cold boot. See lab/boot-perf/boot-bundle-cold-start.md.
    //
    // `IF NOT EXISTS` + no version bump means this index is picked up
    // on the next app launch without deleting or rebuilding the user's
    // existing search.db.
    conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_note_boot_snapshot
            ON note_meta(name, path, library_name);
    ").map_err(|e| format!("Failed to create idx_note_boot_snapshot: {}", e))?;

    // ─── Living Link System (Knowledge Formulation) ─────────────────────
    // note_links: stores typed, directed, annotated links with lifecycle data.
    // Source of truth: LINK files on disk. This table is the fast index.
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS note_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_path TEXT NOT NULL,
            source_name TEXT NOT NULL,
            target_path TEXT,
            target_name TEXT NOT NULL,
            link_type TEXT NOT NULL DEFAULT 'relates',
            annotation TEXT DEFAULT '',
            confidence TEXT DEFAULT 'hypothesis',
            weight REAL DEFAULT 1.0,
            created TEXT DEFAULT '',
            last_traversed TEXT DEFAULT '',
            traversal_count INTEGER DEFAULT 0,
            library_name TEXT DEFAULT '',
            status TEXT DEFAULT 'active',
            UNIQUE(source_path, target_name, link_type)
        );
        CREATE INDEX IF NOT EXISTS idx_link_source ON note_links(source_path);
        CREATE INDEX IF NOT EXISTS idx_link_target ON note_links(target_name);
        CREATE INDEX IF NOT EXISTS idx_link_type ON note_links(link_type);
        CREATE INDEX IF NOT EXISTS idx_link_weight ON note_links(weight);
        CREATE INDEX IF NOT EXISTS idx_link_confidence ON note_links(confidence);
        CREATE INDEX IF NOT EXISTS idx_link_status ON note_links(status);
        CREATE INDEX IF NOT EXISTS idx_link_last_traversed ON note_links(last_traversed);
        CREATE INDEX IF NOT EXISTS idx_link_traversal_count ON note_links(traversal_count);
    ").map_err(|e| format!("Failed to create note_links: {}", e))?;

    // Drop any leftover tables from the aborted custom-index experiment
    // (2026-04-16). The Index panel now reads directly from the FTS5 vocab
    // virtual table `notes_vocab` above; these tables are no longer used.
    conn.execute_batch("
        DROP TABLE IF EXISTS index_mentions;
        DROP TABLE IF EXISTS index_terms;
        DROP TABLE IF EXISTS index_meta;
    ").map_err(|e| format!("Failed to drop obsolete index tables: {}", e))?;

    // ─── One-time FTS5 rebuild after tokenizer migration ─────────────
    // If we bumped past FTS_SCHEMA_VERSION above we dropped the old
    // `notes_fts` + `notes_vocab`. The `CREATE VIRTUAL TABLE IF NOT
    // EXISTS` statements above re-created them with the new tokenizer,
    // but empty — there's no content yet. `INSERT INTO notes_fts(notes_fts)
    // VALUES('rebuild')` walks the content table (`note_meta`) and
    // re-tokenizes every row through our custom pipeline, populating
    // the inverted index.
    //
    // This happens inline in `init_db`, which is called once per
    // Universe open. For the 7,600-note trial Universe this is expected
    // to complete in well under 10 seconds (FTS5 reads the content
    // table sequentially; our tokenizer is pure Rust stemming). If
    // measurement shows it above that threshold we'll move the rebuild
    // to a background task post-paint, per Rule 8's first-time
    // population guidance.
    //
    // A `wal_checkpoint(TRUNCATE)` afterwards prevents the large
    // transaction from bloating the WAL (learned the hard way — a
    // previous aborted streaming run left a 3.1 GB WAL that froze boot).
    if needs_fts_rebuild {
        let rebuild_start = std::time::Instant::now();
        conn.execute_batch("INSERT INTO notes_fts(notes_fts) VALUES('rebuild');")
            .map_err(|e| format!("Failed to rebuild notes_fts: {}", e))?;
        let rebuild_ms = rebuild_start.elapsed().as_millis();

        // Stamp the new schema version BEFORE checkpoint so that a crash
        // after checkpoint but before PRAGMA wouldn't trigger a spurious
        // second rebuild.
        conn.execute_batch(&format!("PRAGMA user_version = {};", FTS_SCHEMA_VERSION))
            .map_err(|e| format!("Failed to stamp user_version: {}", e))?;

        // Truncate WAL so the large rebuild transaction doesn't haunt
        // future boots. Ignore errors — this is hygiene, not correctness.
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");

        diag_log(path, &format!(
            "[search] notes_fts rebuilt with 'constellation' tokenizer in {} ms",
            rebuild_ms
        ));
    }

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
/// Applies Arabic normalization for consistent matching with title-based names.
fn extract_wikilinks(content: &str) -> Vec<String> {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"\[\[([^\]|]+?)(?:\|[^\]]+?)?\]\]").unwrap());
    let mut links = Vec::new();
    for cap in re.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let target = normalize_arabic_for_search(&m.as_str().trim().to_lowercase());
            if !target.is_empty() && !links.contains(&target) {
                links.push(target);
            }
        }
    }
    links
}

/// A typed link extracted from note content.
#[derive(Debug, Clone)]
struct TypedLink {
    target: String,       // target note name (lowercase)
    link_type: String,    // supports, contradicts, causes, etc.
    annotation: String,   // user's reasoning (from |annotation syntax)
}

/// Extract typed links from note content.
/// Matches: [[type::target|annotation]], [[type::target]], [[target|display]], [[target]]
fn extract_typed_links(content: &str) -> Vec<TypedLink> {
    use std::sync::OnceLock;
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    // Matches: [[optional_type::target|optional_annotation]]
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"\[\[(?:([a-zA-Z\-]+)::)?([^\]|]+?)(?:\|([^\]]*))?\]\]").unwrap()
    });
    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for cap in re.captures_iter(content) {
        let link_type = cap.get(1)
            .map(|m| m.as_str().to_lowercase())
            .unwrap_or_else(|| "relates".to_string());
        let target = cap.get(2)
            .map(|m| m.as_str().trim().to_lowercase())
            .unwrap_or_default();
        let annotation = cap.get(3)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        if target.is_empty() { continue; }
        let key = format!("{}::{}", link_type, target);
        if !seen.insert(key) { continue; }

        links.push(TypedLink { target, link_type, annotation });
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
/// Pre-compiled regex patterns for strip_markdown (compiled once, reused on every call).
fn strip_md_patterns() -> &'static [regex::Regex; 4] {
    use std::sync::OnceLock;
    static PATTERNS: OnceLock<[regex::Regex; 4]> = OnceLock::new();
    PATTERNS.get_or_init(|| [
        regex::Regex::new(r"(?s)```.*?```").unwrap(),               // code blocks
        regex::Regex::new(r"`[^`]+`").unwrap(),                     // inline code
        regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]").unwrap(), // wikilinks
        regex::Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap(),      // markdown links
    ])
}

fn strip_markdown(text: &str) -> String {
    let patterns = strip_md_patterns();
    let mut result = text.to_string();
    result = patterns[0].replace_all(&result, " ").to_string();
    result = patterns[1].replace_all(&result, " ").to_string();
    result = patterns[2].replace_all(&result, "$1").to_string();
    result = patterns[3].replace_all(&result, "$1").to_string();
    result = result.replace('#', " ");
    result = result.replace("**", " ").replace("__", " ").replace('*', " ").replace('_', " ");
    result
}

/// Index a single note into the database.
fn index_note(conn: &Connection, note_path: &str, library_name: &str) -> Result<(), String> {
    let path = Path::new(note_path);
    if !path.exists() || path.extension().map(|e| e != "md").unwrap_or(true) {
        return Ok(());
    }

    let file_stem = path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // PERF: mtime-first gate. Previously we read the file into memory THEN
    // checked the cache — meaning every unchanged file was still read from
    // disk on every boot. On a 7,600-note Universe that's 7,600 wasted reads.
    // Now: stat the file, compare to cached mtime, read content only if stale.
    let modified = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);

    let existing_mod: Option<u64> = conn.query_row(
        "SELECT modified FROM note_meta WHERE path = ?1",
        params![note_path],
        |row| row.get(0),
    ).ok();

    if existing_mod == Some(modified) {
        return Ok(()); // Cache hit — no disk read needed.
    }

    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    let (properties, tags, body) = parse_frontmatter(&content);
    let wikilinks = extract_wikilinks(&content);
    let headings = extract_headings(&content);
    let plain_body = strip_markdown(&body);

    // Use frontmatter `title:` as the display name when available (supports canonical filenames).
    // Falls back to file stem for legacy (human-named) files.
    let name = properties.get("title")
        .filter(|t| !t.is_empty())
        .cloned()
        .unwrap_or_else(|| file_stem.clone());

    // Arabic normalization for FTS body text (Phase 4)
    // Normalize diacritics, Alef variants, Teh marbuta for consistent content search.
    // NOTE: name is stored ORIGINAL (not normalized) so it matches graph node IDs.
    // Arabic normalization for name matching happens at query time instead.
    let plain_body = normalize_arabic_for_search(&plain_body);

    let props_json = serde_json::to_string(&properties).unwrap_or_default();
    let tags_json = serde_json::to_string(&tags).unwrap_or_default();
    let links_json = serde_json::to_string(&wikilinks).unwrap_or_default();
    let headings_json = serde_json::to_string(&headings).unwrap_or_default();

    // Extract typed links for the living link system
    let typed_links = extract_typed_links(&content);
    let now = chrono::Utc::now().to_rfc3339();

    // Upsert: delete old, insert new (triggers handle FTS sync) — wrapped in transaction for atomicity
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        conn.execute("DELETE FROM note_meta WHERE path = ?1", params![note_path])
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO note_meta (path, name, library_name, modified, properties_json, tags_json, outgoing_links_json, headings_json, body_text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![note_path, name, library_name, modified, props_json, tags_json, links_json, headings_json, plain_body],
        ).map_err(|e| format!("Failed to index note {}: {}", note_path, e))?;

        // Populate note_links — preserve existing weight/traversal data on re-index
        // Step 1: Snapshot existing traversal data before deleting
        let mut preserved: std::collections::HashMap<String, (f64, String, i64, String, String)> =
            std::collections::HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT target_name, link_type, weight, last_traversed, traversal_count, confidence, created
                 FROM note_links WHERE source_path = ?1"
            ).map_err(|e| e.to_string())?;
            let rows = stmt.query_map(params![note_path], |row| {
                Ok((
                    row.get::<_, String>(0)?,  // target_name
                    row.get::<_, String>(1)?,  // link_type
                    row.get::<_, f64>(2)?,     // weight
                    row.get::<_, String>(3)?,  // last_traversed
                    row.get::<_, i64>(4)?,     // traversal_count
                    row.get::<_, String>(5)?,  // confidence
                    row.get::<_, String>(6)?,  // created
                ))
            }).map_err(|e| e.to_string())?;
            for row in rows {
                if let Ok((target, ltype, w, lt, tc, conf, created)) = row {
                    // Only preserve if link was actually traversed (tc > 0)
                    if tc > 0 || w != 1.0 {
                        let key = format!("{}::{}", ltype, target);
                        preserved.insert(key, (w, lt, tc, conf, created));
                    }
                }
            }
        }
        // Step 2: Delete and re-insert, restoring preserved data
        conn.execute("DELETE FROM note_links WHERE source_path = ?1", params![note_path])
            .map_err(|e| e.to_string())?;
        for tl in &typed_links {
            let key = format!("{}::{}", tl.link_type, tl.target);
            if let Some((w, lt, tc, conf, created)) = preserved.get(&key) {
                conn.execute(
                    "INSERT OR IGNORE INTO note_links (source_path, source_name, target_name, link_type, annotation, confidence, weight, created, last_traversed, traversal_count, library_name, status)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'active')",
                    params![note_path, name, tl.target, tl.link_type, tl.annotation, conf, w, created, lt, tc, library_name],
                ).map_err(|e| format!("Failed to index link: {}", e))?;
            } else {
                conn.execute(
                    "INSERT OR IGNORE INTO note_links (source_path, source_name, target_name, link_type, annotation, confidence, weight, created, last_traversed, traversal_count, library_name, status)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'hypothesis', 1.0, ?6, ?6, 0, ?7, 'active')",
                    params![note_path, name, tl.target, tl.link_type, tl.annotation, now, library_name],
                ).map_err(|e| format!("Failed to index link: {}", e))?;
            }
        }

        Ok(())
    })();
    match result {
        Ok(()) => { conn.execute_batch("COMMIT").map_err(|e| e.to_string())?; }
        Err(e) => { let _ = conn.execute_batch("ROLLBACK"); return Err(e); }
    }

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
            // Try exact match first, then partial (LIKE) for user-typed partial names
            let links: Option<String> = conn.query_row(
                "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                params![source_lower],
                |row| row.get(0),
            ).ok().or_else(|| {
                conn.query_row(
                    "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) LIKE '%' || ?1 || '%' LIMIT 1",
                    params![source_lower],
                    |row| row.get(0),
                ).ok()
            });
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

    // Mutual filters: notes that link to X AND X links back to them
    if let Some(targets) = &filters.mutual {
        for target in targets {
            let target_lower = target.to_lowercase();
            // Must link TO target
            conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target_lower.clone()));
            // AND this note must be in target's outgoing links (X links back)
            let links: Option<String> = conn.query_row(
                "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                params![target_lower],
                |row| row.get(0),
            ).ok().or_else(|| {
                conn.query_row(
                    "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) LIKE '%' || ?1 || '%' LIMIT 1",
                    params![target_lower],
                    |row| row.get(0),
                ).ok()
            });
            let back_targets: Vec<String> = links
                .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                .unwrap_or_default();
            if !back_targets.is_empty() {
                let placeholders: Vec<String> = back_targets.iter().map(|_| "LOWER(name) = ?".to_string()).collect();
                conditions.push(format!("({})", placeholders.join(" OR ")));
                for bt in &back_targets {
                    params_vec.push(Box::new(bt.clone()));
                }
            } else {
                // Target has no outgoing links → mutual is impossible → return nothing
                conditions.push("0 = 1".to_string());
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
    // Pre-compute incoming link targets in ONE pass (O(n) instead of O(n²))
    if filters.orphans.unwrap_or(false) {
        // No outgoing links
        conditions.push("(outgoing_links_json IS NULL OR outgoing_links_json = '[]')".to_string());

        // Build set of all notes that have incoming links (single scan of outgoing_links_json)
        let mut has_incoming: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Ok(mut stmt) = conn.prepare("SELECT outgoing_links_json FROM note_meta WHERE outgoing_links_json IS NOT NULL AND outgoing_links_json != '[]'") {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                for row in rows.flatten() {
                    if let Ok(targets) = serde_json::from_str::<Vec<String>>(&row) {
                        for t in targets {
                            has_incoming.insert(t);
                        }
                    }
                }
            }
        }

        // Use temp table for efficient SQL NOT IN check
        let _ = conn.execute("CREATE TEMP TABLE IF NOT EXISTS _incoming_targets (name TEXT PRIMARY KEY)", []);
        let _ = conn.execute("DELETE FROM _incoming_targets", []);
        if let Ok(mut ins) = conn.prepare("INSERT OR IGNORE INTO _incoming_targets (name) VALUES (?1)") {
            for name in &has_incoming {
                let _ = ins.execute(params![name]);
            }
        }
        conditions.push("LOWER(name) NOT IN (SELECT name FROM _incoming_targets)".to_string());
    }

    // Links-between filter: notes that link to BOTH X and Y
    if let Some(targets) = &filters.links_between {
        for target in targets {
            conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target.to_lowercase()));
        }
    }

    // Links-all filter: notes connected to X in either direction (incoming OR outgoing)
    // Results get match_type "wikilink" with snippet indicating direction
    if let Some(targets) = &filters.links_all {
        for target in targets {
            let target_lower = target.to_lowercase();
            // Get X's outgoing links (notes X links to)
            // Exact match first, then partial (LIKE) for user-typed partial names
            let outgoing: Vec<String> = conn.query_row(
                "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                params![target_lower],
                |row| row.get::<_, String>(0),
            ).ok()
            .or_else(|| {
                conn.query_row(
                    "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) LIKE '%' || ?1 || '%' LIMIT 1",
                    params![target_lower],
                    |row| row.get::<_, String>(0),
                ).ok()
            })
            .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
            .unwrap_or_default();

            // Build: (links TO X) OR (X links to this note)
            let mut sub_conditions: Vec<String> = Vec::new();
            // Incoming: notes that link to X
            sub_conditions.push("outgoing_links_json LIKE '%\"' || ? || '\"%'".to_string());
            params_vec.push(Box::new(target_lower.clone()));
            // Outgoing: notes that X links to
            for out_name in &outgoing {
                sub_conditions.push("LOWER(name) = ?".to_string());
                params_vec.push(Box::new(out_name.clone()));
            }
            conditions.push(format!("({})", sub_conditions.join(" OR ")));
            // Exclude X itself
            conditions.push("LOWER(name) != ?".to_string());
            params_vec.push(Box::new(target_lower));
        }
    }

    // Typed link filter: find notes that have a specific relationship to a target
    // e.g., "supports [[X]]" → find notes where source has link_type=supports to target=X
    if let Some(typed_links) = &filters.typed_links {
        for tl in typed_links {
            let target_lower = tl.target.to_lowercase();
            let link_type_lower = tl.link_type.to_lowercase();
            // Query note_links table for matching typed links
            let mut source_paths: Vec<String> = Vec::new();
            if let Ok(mut stmt) = conn.prepare(
                "SELECT DISTINCT source_path FROM note_links WHERE link_type = ?1 AND (LOWER(target_name) = ?2 OR LOWER(target_name) LIKE '%' || ?2 || '%') AND status = 'active'"
            ) {
                if let Ok(rows) = stmt.query_map(params![link_type_lower, target_lower], |row| row.get::<_, String>(0)) {
                    for row in rows.flatten() {
                        source_paths.push(row);
                    }
                }
            }
            if source_paths.is_empty() {
                // No matches — add impossible condition to return empty results
                conditions.push("1 = 0".to_string());
            } else {
                let placeholders: Vec<String> = source_paths.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!("path IN ({})", placeholders.join(",")));
                for sp in &source_paths {
                    params_vec.push(Box::new(sp.clone()));
                }
            }
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
    } else if filters.wikilinks_to.as_ref().map_or(false, |w| !w.is_empty()) {
        "links_to"
    } else if filters.wikilinks_from.as_ref().map_or(false, |w| !w.is_empty()) {
        "links_from"
    } else if filters.mutual.as_ref().map_or(false, |w| !w.is_empty()) {
        "mutual"
    } else if filters.links_between.as_ref().map_or(false, |w| !w.is_empty()) {
        "links_between"
    } else if filters.links_all.as_ref().map_or(false, |w| !w.is_empty()) {
        "links_all"
    } else if filters.mentions.as_ref().map_or(false, |m| !m.is_empty()) {
        "mentions"
    } else if filters.orphans.unwrap_or(false) {
        "orphan"
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

// ─── Link System Queries ──────────────────────────────────────

/// Get link statistics from the note_links table.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkStats {
    pub total_links: usize,
    pub by_type: std::collections::HashMap<String, usize>,
    pub by_confidence: std::collections::HashMap<String, usize>,
    pub with_annotation: usize,
    pub sample_links: Vec<serde_json::Value>,
}

#[tauri::command]
pub fn constellation_link_stats(app: tauri::AppHandle) -> Result<LinkStats, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let total_links: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active'", [], |r| r.get(0)
    ).unwrap_or(0);

    let mut by_type = std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT link_type, COUNT(*) FROM note_links WHERE status = 'active' GROUP BY link_type") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,usize>(1)?))) {
            for row in rows.flatten() { by_type.insert(row.0, row.1); }
        }
    }

    let mut by_confidence = std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT confidence, COUNT(*) FROM note_links WHERE status = 'active' GROUP BY confidence") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,usize>(1)?))) {
            for row in rows.flatten() { by_confidence.insert(row.0, row.1); }
        }
    }

    let with_annotation: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active' AND annotation != ''", [], |r| r.get(0)
    ).unwrap_or(0);

    let mut sample_links = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT source_name, target_name, link_type, annotation, confidence, weight FROM note_links WHERE status = 'active' ORDER BY weight DESC LIMIT 10"
    ) {
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok(serde_json::json!({
                "source": r.get::<_,String>(0)?,
                "target": r.get::<_,String>(1)?,
                "type": r.get::<_,String>(2)?,
                "annotation": r.get::<_,String>(3)?,
                "confidence": r.get::<_,String>(4)?,
                "weight": r.get::<_,f64>(5)?,
            }))
        }) {
            for row in rows.flatten() { sample_links.push(row); }
        }
    }

    Ok(LinkStats { total_links, by_type, by_confidence, with_annotation, sample_links })
}

/// Record a link traversal: user followed a link from source to target.
/// Updates last_traversed, increments traversal_count, recalculates weight.
/// Weight formula: 1.0 + ln(1 + traversal_count) — logarithmic, early traversals matter most.
#[tauri::command]
pub fn constellation_link_traverse(
    app: tauri::AppHandle,
    source_path: String,
    target_name: String,
) -> Result<serde_json::Value, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let now = chrono::Utc::now().to_rfc3339();
    let target_lower = target_name.to_lowercase();

    // Two-step: read current traversal_count, compute new weight in Rust, then update.
    // This avoids reliance on SQLite math functions (ln) which need SQLITE_ENABLE_MATH_FUNCTIONS.
    let mut stmt = conn.prepare(
        "SELECT id, traversal_count FROM note_links
         WHERE source_path = ?1 AND LOWER(target_name) = ?2"
    ).map_err(|e| e.to_string())?;
    let links: Vec<(i64, i64)> = stmt.query_map(params![source_path, target_lower], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
    }).map_err(|e| e.to_string())?
      .filter_map(|r| r.ok())
      .collect();

    let mut updated: usize = 0;
    for (id, tc) in &links {
        let new_tc = tc + 1;
        let new_weight = 1.0 + (1.0 + new_tc as f64).ln();
        conn.execute(
            "UPDATE note_links SET
                traversal_count = ?1,
                last_traversed = ?2,
                weight = ?3,
                status = CASE WHEN status = 'dormant' THEN 'active' ELSE status END
             WHERE id = ?4",
            params![new_tc, now, new_weight, id],
        ).map_err(|e| format!("Failed to record traversal: {}", e))?;
        updated += 1;
    }

    Ok(serde_json::json!({
        "updated": updated,
        "source": source_path,
        "target": target_name,
        "timestamp": now,
    }))
}

/// Find dormant links — links not traversed within the given threshold (default 90 days).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DormantLink {
    pub source_name: String,
    pub target_name: String,
    pub link_type: String,
    pub annotation: String,
    pub weight: f64,
    pub last_traversed: String,
    pub traversal_count: i64,
    pub days_dormant: i64,
}

#[tauri::command]
pub fn constellation_link_dormant(
    app: tauri::AppHandle,
    days_threshold: Option<u32>,
) -> Result<Vec<DormantLink>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let threshold = days_threshold.unwrap_or(90) as i64;

    let mut stmt = conn.prepare(
        "SELECT source_name, target_name, link_type, annotation, weight,
                last_traversed, traversal_count,
                CAST(julianday('now') - julianday(last_traversed) AS INTEGER) AS days_dormant
         FROM note_links
         WHERE status = 'active'
           AND last_traversed != ''
           AND julianday('now') - julianday(last_traversed) >= ?1
         ORDER BY days_dormant DESC
         LIMIT 200"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![threshold], |row| {
        Ok(DormantLink {
            source_name: row.get(0)?,
            target_name: row.get(1)?,
            link_type: row.get(2)?,
            annotation: row.get(3)?,
            weight: row.get(4)?,
            last_traversed: row.get(5)?,
            traversal_count: row.get(6)?,
            days_dormant: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ─── P4: Formulation Analysis (Knowledge Diagnostics) ────────

/// A formulation insight — one row from a diagnostic query.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FormulationInsight {
    pub source_name: String,
    pub target_name: String,
    pub link_type: String,
    pub annotation: String,
    pub weight: f64,
    pub confidence: String,
    pub traversal_count: i64,
    pub last_traversed: String,
    pub library_name: String,
}

/// Formulation analysis: diagnostic queries for intellectual life.
/// `query_type` determines which analysis runs:
///   - "strongest_evidence"  — top supports for a target, ranked by weight × confidence
///   - "weak_foundations"    — hypothesis links with high weight (building on sand)
///   - "tensions"           — contradicts links for a target
///   - "stagnating"         — high-weight links gone dormant
///   - "abandoned"          — archived links
///   - "emerging"           — hypothesis + growing weight (curiosity without proof)
///   - "bias_check"         — targets where supports >> contradicts (echo chambers)
///   - "most_connected"     — notes with most incoming typed links
///   - "knowledge_gaps"     — notes with outgoing links but few incoming (giving but not receiving)
#[tauri::command]
pub fn constellation_formulation_analysis(
    app: tauri::AppHandle,
    query_type: String,
    target: Option<String>,
) -> Result<Vec<FormulationInsight>, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let target_lower = target.as_deref().unwrap_or("").to_lowercase();
    let confidence_weight = |c: &str| -> f64 {
        match c { "established" => 3.0, "evidence" => 2.0, "hypothesis" => 1.0, "contested" => 0.5, _ => 1.0 }
    };

    match query_type.as_str() {
        "strongest_evidence" => {
            // Top supports for a target, ranked by weight × confidence multiplier
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE link_type = 'supports' AND status = 'active'
                 AND (?1 = '' OR LOWER(target_name) LIKE '%' || ?1 || '%')
                 ORDER BY weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            let mut results = query_insights(&mut stmt, &[&target_lower as &dyn rusqlite::types::ToSql])?;
            // Re-sort by weight × confidence
            results.sort_by(|a, b| {
                let sa = a.weight * confidence_weight(&a.confidence);
                let sb = b.weight * confidence_weight(&b.confidence);
                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
            });
            Ok(results)
        }
        "weak_foundations" => {
            // hypothesis links with high weight — building on uncertain ground
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE confidence = 'hypothesis' AND weight > 2.0 AND status = 'active'
                 ORDER BY weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[])
        }
        "tensions" => {
            // contradicts links for a target
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE link_type = 'contradicts' AND status = 'active'
                 AND (?1 = '' OR LOWER(target_name) LIKE '%' || ?1 || '%')
                 ORDER BY weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[&target_lower as &dyn rusqlite::types::ToSql])
        }
        "stagnating" => {
            // high-weight links gone dormant
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE status = 'dormant' AND weight > 2.0
                 ORDER BY weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[])
        }
        "abandoned" => {
            // archived links
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE status = 'archived'
                 ORDER BY last_traversed DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[])
        }
        "emerging" => {
            // hypothesis + recently traversed (curiosity without proof yet)
            let mut stmt = conn.prepare(
                "SELECT source_name, target_name, link_type, annotation, weight, confidence, traversal_count, last_traversed, library_name
                 FROM note_links WHERE confidence = 'hypothesis' AND traversal_count > 0 AND status = 'active'
                 ORDER BY traversal_count DESC, weight DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            query_insights(&mut stmt, &[])
        }
        "bias_check" => {
            // targets where supports count >> contradicts count
            let mut stmt = conn.prepare(
                "SELECT target_name,
                    SUM(CASE WHEN link_type = 'supports' THEN 1 ELSE 0 END) as support_count,
                    SUM(CASE WHEN link_type = 'contradicts' THEN 1 ELSE 0 END) as contradict_count
                 FROM note_links WHERE status = 'active' AND link_type IN ('supports', 'contradicts')
                 GROUP BY target_name
                 HAVING support_count > 0 AND contradict_count = 0
                 ORDER BY support_count DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |row| {
                Ok(FormulationInsight {
                    source_name: String::new(),
                    target_name: row.get(0)?,
                    link_type: "bias".to_string(),
                    annotation: format!("{} supports, {} contradicts", row.get::<_, i64>(1)?, row.get::<_, i64>(2)?),
                    weight: row.get::<_, i64>(1)? as f64,
                    confidence: String::new(),
                    traversal_count: 0,
                    last_traversed: String::new(),
                    library_name: String::new(),
                })
            }).map_err(|e| e.to_string())?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }
        "most_connected" => {
            // notes with most incoming typed links
            let mut stmt = conn.prepare(
                "SELECT target_name, COUNT(*) as cnt, GROUP_CONCAT(DISTINCT link_type) as types,
                        AVG(weight) as avg_weight
                 FROM note_links WHERE status = 'active'
                 GROUP BY target_name ORDER BY cnt DESC LIMIT 50"
            ).map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |row| {
                Ok(FormulationInsight {
                    source_name: String::new(),
                    target_name: row.get(0)?,
                    link_type: row.get::<_, String>(2).unwrap_or_default(),
                    annotation: format!("{} incoming links", row.get::<_, i64>(1)?),
                    weight: row.get::<_, f64>(3).unwrap_or(1.0),
                    confidence: String::new(),
                    traversal_count: row.get::<_, i64>(1)?,
                    last_traversed: String::new(),
                    library_name: String::new(),
                })
            }).map_err(|e| e.to_string())?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        }
        _ => Err(format!("Unknown formulation query: {}", query_type)),
    }
}

/// Helper: execute a prepared statement and collect FormulationInsight rows.
fn query_insights(
    stmt: &mut rusqlite::Statement,
    params: &[&dyn rusqlite::types::ToSql],
) -> Result<Vec<FormulationInsight>, String> {
    let rows = stmt.query_map(params, |row| {
        Ok(FormulationInsight {
            source_name: row.get(0)?,
            target_name: row.get(1)?,
            link_type: row.get(2)?,
            annotation: row.get(3)?,
            weight: row.get(4)?,
            confidence: row.get(5)?,
            traversal_count: row.get(6)?,
            last_traversed: row.get(7)?,
            library_name: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Apply weight decay to all active links.
/// Formula: weight = weight × 0.95^(months_since_last_traversal)
/// Only decays links not traversed in the last 30 days.
/// Also derives lifecycle stage from weight + traversal data.
#[tauri::command]
pub fn constellation_link_decay(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    // Step 1: Apply weight decay to links not traversed in 30+ days
    // Calculate months since last traversal for each link
    let mut decayed: usize = 0;
    let mut dormant_count: usize = 0;

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, weight, last_traversed, traversal_count, status
         FROM note_links
         WHERE status IN ('active', 'dormant')
           AND last_traversed != ''
           AND julianday('now') - julianday(last_traversed) >= 30"
    ) {
        let links: Vec<(i64, f64, String, i64, String)> = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
            ))
        }).map_err(|e| e.to_string())?
          .filter_map(|r| r.ok())
          .collect();

        for (id, weight, last_traversed, _tc, status) in &links {
            // Calculate months since last traversal
            let months = conn.query_row(
                "SELECT (julianday('now') - julianday(?1)) / 30.0",
                params![last_traversed],
                |row| row.get::<_, f64>(0),
            ).unwrap_or(0.0);

            if months < 1.0 { continue; }

            // Apply decay: weight × 0.95^months
            let new_weight = weight * (0.95_f64).powf(months);
            let new_weight = (new_weight * 1000.0).round() / 1000.0; // round to 3 decimals

            // Determine lifecycle status
            let new_status = if months >= 3.0 && *status == "active" {
                dormant_count += 1;
                "dormant"
            } else {
                status.as_str()
            };

            conn.execute(
                "UPDATE note_links SET weight = ?1, status = ?2 WHERE id = ?3",
                params![new_weight, new_status, id],
            ).map_err(|e| format!("Failed to decay link: {}", e))?;
            decayed += 1;
        }
    }

    // Step 2: Count lifecycle stage distribution
    let mut stages: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // Birth: traversal_count = 0, weight = 1.0
    let birth: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active' AND traversal_count = 0",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("birth".to_string(), birth);

    // Growth: traversal_count > 0, weight < 5.0
    let growth: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active' AND traversal_count > 0 AND weight < 5.0",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("growth".to_string(), growth);

    // Maturity: weight >= 5.0
    let maturity: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'active' AND weight >= 5.0",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("maturity".to_string(), maturity);

    // Dormancy
    let dormant: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'dormant'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("dormancy".to_string(), dormant);

    // Archived
    let archived: usize = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE status = 'archived'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    stages.insert("archived".to_string(), archived);

    Ok(serde_json::json!({
        "decayed": decayed,
        "new_dormant": dormant_count,
        "lifecycle": stages,
    }))
}

/// Update a link's confidence level.
#[tauri::command]
pub fn constellation_link_set_confidence(
    app: tauri::AppHandle,
    source_path: String,
    target_name: String,
    confidence: String,
) -> Result<(), String> {
    if !["hypothesis", "evidence", "established", "contested"].contains(&confidence.as_str()) {
        return Err(format!("Invalid confidence level: {}", confidence));
    }
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let target_lower = target_name.to_lowercase();
    conn.execute(
        "UPDATE note_links SET confidence = ?1 WHERE source_path = ?2 AND LOWER(target_name) = ?3",
        params![confidence, source_path, target_lower],
    ).map_err(|e| format!("Failed to update confidence: {}", e))?;

    Ok(())
}

/// Archive a link (soft delete — preserved in history).
#[tauri::command]
pub fn constellation_link_archive(
    app: tauri::AppHandle,
    source_path: String,
    target_name: String,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search DB not initialized")?;

    let target_lower = target_name.to_lowercase();
    conn.execute(
        "UPDATE note_links SET status = 'archived', weight = 0.0 WHERE source_path = ?1 AND LOWER(target_name) = ?2",
        params![source_path, target_lower],
    ).map_err(|e| format!("Failed to archive link: {}", e))?;

    Ok(())
}

// ─── Tauri Commands ────────────────────────────────────────────

/// Fast path: open the search DB (creating schema if absent) and place it in
/// state. Does NOT walk the filesystem. Safe to call from the boot path — on
/// a populated DB this is a millisecond-scale operation.
///
/// Previously `constellation_search_init` opened the DB AND walked every
/// library before putting the connection in state. That meant any concurrent
/// `cache_boot_snapshot` call saw `None` and reported a cold cache, defeating
/// the whole cache-first boot. Splitting this in two is what makes the
/// cache-first boot actually work on 2nd+ launches.
pub fn ensure_search_db_ready(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<SearchState>();
    {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Ok(());
        }
    }
    let path = db_path(app)?;
    let version_path = path.with_extension("version");
    let current_version = "7";
    let needs_rebuild = match std::fs::read_to_string(&version_path) {
        Ok(v) => v.trim() != current_version,
        Err(_) => true,
    };
    if needs_rebuild {
        let _ = std::fs::remove_file(&path);
    }
    let conn = init_db(&path)?;
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    *db = Some(conn);
    if needs_rebuild {
        let _ = std::fs::write(&version_path, current_version);
    }
    Ok(())
}

/// Walk every library and reindex changed files using a DEDICATED connection
/// (not the one in SearchState). SQLite's WAL mode allows concurrent readers,
/// so frontend queries through `state.db` continue working while this runs.
///
/// Runs in the caller's thread. `cache_reconcile` wraps this in
/// `std::thread::spawn` so it never blocks IPC.
pub fn reconcile_filesystem(app: &tauri::AppHandle) -> Result<SearchIndexStats, String> {
    // Make sure schema exists and state has the query connection.
    ensure_search_db_ready(app)?;

    let path = db_path(app)?;
    // Dedicated connection for the walk — does NOT touch state.db, so the
    // state's query connection stays available to frontend reads the whole
    // time. WAL mode (set in init_db) is what makes this safe.
    let mut walk_conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open search.db for reconcile: {}", e))?;
    walk_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|e| e.to_string())?;
    // Reconcile writes to note_meta; the FTS5 AFTER-INSERT/UPDATE
    // triggers tokenize body_text through the 'constellation' tokenizer.
    // Without registration here the trigger's INSERT INTO notes_fts
    // would fail on this connection with "no such tokenizer".
    register_fts5_tokenizer(&mut walk_conn)?;

    let libraries = crate::libraries::load_all_libraries(app);
    for lib in &libraries {
        index_library_recursive(&walk_conn, Path::new(&lib.path), &lib.name, 0);
    }

    let note_count: u32 = walk_conn.query_row(
        "SELECT COUNT(*) FROM note_meta", [], |row| row.get(0)
    ).unwrap_or(0);
    let index_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    Ok(SearchIndexStats { note_count, index_size_bytes: index_size })
}

/// Initialize the search index — builds/rebuilds the SQLite database.
///
/// Kept for backward compatibility with callers that want the legacy
/// "open + walk" behavior. The boot path now uses `ensure_search_db_ready`
/// (instant) and `reconcile_filesystem` (on a background thread) separately.
#[tauri::command]
pub fn constellation_search_init(app: tauri::AppHandle) -> Result<SearchIndexStats, String> {
    ensure_search_db_ready(&app)?;
    reconcile_filesystem(&app)
}

/// Reindex a single note (called on file change).
#[tauri::command]
pub fn constellation_search_reindex(
    app: tauri::AppHandle,
    note_path: String,
    library_name: String,
) -> Result<(), String> {
    let state = app.state::<SearchState>();
    reindex_single_note(&state, &note_path, &library_name)
}

/// Delete a note from the search index + link table.
pub fn reindex_delete_note(state: &SearchState, note_path: &str) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db.as_ref() {
        let _ = conn.execute("DELETE FROM note_links WHERE source_path = ?1", params![note_path]);
        let _ = conn.execute("DELETE FROM note_meta WHERE path = ?1", params![note_path]);
    }
    Ok(())
}

/// Reindex a single note — callable from other modules without Tauri command overhead.
pub fn reindex_single_note(
    state: &SearchState,
    note_path: &str,
    library_name: &str,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db.as_ref() {
        index_note(conn, note_path, library_name)?;
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
            // Fallback: open the DB lazily. This used to call
            // constellation_search_init which WALKED every library on every
            // first-search — blocking the UI for seconds on a large Universe.
            // Now we only open the connection (cheap); the index is kept fresh
            // by the file watcher and by explicit Rebuild Index actions. If
            // the index is cold (empty), the search just returns no results.
            drop(db_guard);
            ensure_search_db_ready(&app)?;
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
                // Post-process links_all: tag each result with direction (↑ incoming / ↓ outgoing)
                if let Some(targets) = &filters.links_all {
                    for target in targets {
                        let target_lower = target.to_lowercase();
                        let outgoing: Vec<String> = conn.query_row(
                            "SELECT outgoing_links_json FROM note_meta WHERE LOWER(name) = ?1",
                            params![target_lower],
                            |row| row.get::<_, String>(0),
                        ).ok()
                        .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                        .unwrap_or_default();
                        let outgoing_set: std::collections::HashSet<String> = outgoing.into_iter().collect();

                        for r in results.iter_mut() {
                            let r_lower = r.name.to_lowercase();
                            let is_outgoing = outgoing_set.contains(&r_lower);
                            // Check if this result links TO target (incoming to target)
                            let is_incoming = r.snippet.as_ref()
                                .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
                                .map(|links| links.contains(&target_lower))
                                .unwrap_or(false);
                            // If we can't tell from snippet, check outgoing_links_json
                            let is_incoming = is_incoming || {
                                conn.query_row(
                                    "SELECT outgoing_links_json FROM note_meta WHERE path = ?1",
                                    params![r.path],
                                    |row| row.get::<_, String>(0),
                                ).ok()
                                .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                                .map(|links| links.contains(&target_lower))
                                .unwrap_or(false)
                            };

                            r.snippet = Some(if is_incoming && is_outgoing {
                                "↑↓".to_string()
                            } else if is_incoming {
                                "↑".to_string()
                            } else {
                                "↓".to_string()
                            });
                        }
                    }
                }
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
        // Two-pass approach: collect all (result, similarity) pairs first,
        // then apply dynamic threshold relative to top score.
        // e5-small produces compressed similarity ranges (0.72–0.88 typical),
        // so a fixed threshold fails — we need adaptive filtering.
        let mut all: Vec<(SearchResult, f32)> = Vec::new();
        for row in rows.flatten() {
            let (path, name, library_name, modified, embedding) = row;
            let sim = cosine_similarity(query_embedding, &embedding);
            if sim > 0.5 { // absolute floor — skip completely irrelevant
                all.push((SearchResult {
                    path, name, library_name, modified,
                    score: sim as f64,
                    match_type: "semantic".to_string(),
                    snippet: None,
                    heading_breadcrumb: None,
                }, sim));
            }
        }

        if !all.is_empty() {
            let top_score = all.iter().map(|(_, s)| *s).fold(f32::NEG_INFINITY, f32::max);
            // Dynamic threshold: within 3% of top score, minimum 0.75
            let dynamic_thresh = f32::max(0.75, top_score - 0.03);
            for (r, sim) in all {
                if sim >= dynamic_thresh {
                    scored.push((r, sim));
                }
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
            // Same lazy-open pattern as constellation_search — cheap DB open
            // only, never a filesystem walk. See that function for rationale.
            drop(db_guard);
            ensure_search_db_ready(&app)?;
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

        // Title search: try BOTH original AND normalized (name is stored original,
        // but user might type either form of Arabic)
        all_titles.extend(search_titles(conn, &raw_lower, limit));
        if normalized != raw_lower {
            all_titles.extend(search_titles(conn, &normalized, limit));
        }
        all_contents.extend(search_contents(conn, &normalized, limit));
        // Tags and wikilinks: search both original and normalized
        all_tags.extend(search_tags(conn, &raw_lower, limit));
        if normalized != raw_lower {
            all_tags.extend(search_tags(conn, &normalized, limit));
        }
        all_properties.extend(search_properties(conn, &raw_lower, limit));
        all_wikilinks.extend(search_wikilinks(conn, &raw_lower, limit));
        if normalized != raw_lower {
            all_wikilinks.extend(search_wikilinks(conn, &normalized, limit));
        }
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

/// Return incoming link counts for all notes from the search database.
#[tauri::command]
pub fn constellation_search_link_counts(
    app: tauri::AppHandle,
) -> Result<std::collections::HashMap<String, u32>, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => return Ok(std::collections::HashMap::new()),
    };

    // Initialize counts for all notes
    let mut counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut name_stmt = conn.prepare("SELECT LOWER(name) FROM note_meta")
        .map_err(|e| e.to_string())?;
    let names: Vec<String> = name_stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    for name in &names {
        counts.insert(name.clone(), 0);
    }

    // Scan all outgoing links and count targets
    let mut links_stmt = conn.prepare("SELECT outgoing_links_json FROM note_meta")
        .map_err(|e| e.to_string())?;
    let all_links: Vec<String> = links_stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    for links_json in &all_links {
        if let Ok(targets) = serde_json::from_str::<Vec<String>>(links_json) {
            for target in &targets {
                if let Some(count) = counts.get_mut(&target.to_lowercase()) {
                    *count += 1;
                }
            }
        }
    }

    Ok(counts)
}
