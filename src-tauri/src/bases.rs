use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
// tauri::Manager unused — removed
// HashSet was used by §A for in-memory columns_detected dedup; §C replaced
// that with a SQL DISTINCT query so HashSet is no longer needed here.

// ─── Security ───

/// Validate that a file path is within a registered library or the active universe's bases directory.
fn validate_base_path(app: &tauri::AppHandle, file_path: &str) -> Result<(), String> {
    let target = fs::canonicalize(file_path)
        .or_else(|_| {
            // File may not exist yet (save); canonicalize parent
            Path::new(file_path).parent()
                .ok_or_else(|| "Invalid path".to_string())
                .and_then(|p| fs::canonicalize(p).map_err(|e| e.to_string()))
        })
        .map_err(|_| "Cannot resolve file path.".to_string())?;

    // Check if path is within the active universe directory
    if let Ok(universe_dir) = crate::universe::active_universe_dir(app) {
        if let Ok(canon_universe) = fs::canonicalize(&universe_dir) {
            if target.starts_with(&canon_universe) {
                return Ok(());
            }
        }
    }

    // Check if path is within any registered library
    let libraries = crate::libraries::load_libraries_pub(app);
    for lib in &libraries {
        if let Ok(canon_lib) = fs::canonicalize(&lib.path) {
            if target.starts_with(&canon_lib) {
                return Ok(());
            }
        }
    }

    Err("Path is outside of registered libraries and universe directory.".to_string())
}

// ─── Data Structures ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSource {
    #[serde(rename = "type")]
    pub source_type: String,   // "folder" | "tag" | "all"
    pub path: Option<String>,  // folder path (relative to library root)
    pub tag: Option<String>,   // tag filter
    #[serde(rename = "includeSubfolders", default = "default_true")]
    pub include_subfolders: bool,
    #[serde(rename = "selectedVaults", default)]
    pub selected_vaults: Vec<String>, // empty = all libraries; populated = only these library names
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub property: String,
    pub label: Option<String>,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_true")]
    pub visible: bool,
    pub direction: Option<String>, // per-column direction override
}

fn default_width() -> u32 { 150 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    pub property: String,
    pub operator: String, // "is" | "is_not" | "contains" | "not_contains" | "gt" | "lt" | "is_empty" | "is_not_empty"
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortRule {
    pub property: String,
    #[serde(default = "default_asc")]
    pub direction: String, // "asc" | "desc"
}

fn default_asc() -> String { "asc".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDefinition {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub name: String,
    pub source: BaseSource,
    #[serde(default)]
    pub columns: Vec<ColumnDef>,
    #[serde(default)]
    pub filters: Vec<FilterRule>,
    #[serde(default)]
    pub sorts: Vec<SortRule>,
    #[serde(default = "default_view")]
    pub view: String, // "table" | "card" | "list"
    #[serde(default = "default_auto")]
    pub direction: String, // "auto" | "rtl" | "ltr"
}

fn default_version() -> u32 { 1 }
fn default_view() -> String { "table".to_string() }
fn default_auto() -> String { "auto".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseRow {
    pub file_path: String,
    pub file_name: String,
    pub library_name: String,
    pub library_path: String,
    pub properties: HashMap<String, String>,
    pub modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseQueryResult {
    pub rows: Vec<BaseRow>,
    pub total_count: usize,
    pub query_time_ms: u64,
    pub columns_detected: Vec<String>, // auto-detected property keys from data
}

// ─── Frontmatter Parser ───

/// Parse YAML frontmatter from a markdown note into a HashMap.
/// Returns None if no valid frontmatter found.
pub fn parse_frontmatter(content: &str) -> Option<HashMap<String, String>> {
    if !content.starts_with("---") {
        return None;
    }
    let lines: Vec<&str> = content.lines().collect();
    let end_idx = lines.iter().skip(1).position(|l| l.trim() == "---")?;
    let end_idx = end_idx + 1; // offset from skip(1)

    let mut props = HashMap::new();
    let mut i = 1;
    while i < end_idx {
        let line = lines[i];
        if let Some(colon) = line.find(':') {
            let key = line[..colon].trim();
            // Skip indented lines (part of nested YAML)
            if key.is_empty() || line.starts_with(' ') || line.starts_with('\t') {
                i += 1;
                continue;
            }
            let mut value = line[colon + 1..].trim().to_string();

            // Handle multi-line list values (key:\n  - item1\n  - item2)
            if value.is_empty() && i + 1 < end_idx {
                let next = lines.get(i + 1).unwrap_or(&"");
                if next.trim_start().starts_with("- ") {
                    let mut items = Vec::new();
                    let mut j = i + 1;
                    while j < end_idx {
                        let item_line = lines[j].trim();
                        if item_line.starts_with("- ") {
                            let item = item_line[2..].trim();
                            let item = item.trim_matches('"').trim_matches('\'');
                            items.push(item.to_string());
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    value = items.join(", ");
                    i = j;
                    if !key.is_empty() {
                        props.insert(key.to_string(), value);
                    }
                    continue;
                }
            }

            // Strip surrounding quotes
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = value[1..value.len() - 1].to_string();
            }

            // Handle inline list [a, b, c]
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                let items: Vec<&str> = inner.split(',').map(|s| {
                    s.trim().trim_matches('"').trim_matches('\'')
                }).collect();
                value = items.join(", ");
            }

            if !key.is_empty() {
                props.insert(key.to_string(), value);
            }
        }
        i += 1;
    }

    Some(props)
}

// ─── Scanning ───

/// Recursively scan a directory for .md files and extract their frontmatter.
pub fn scan_folder(
    dir: &Path,
    library_name: &str,
    library_path: &str,
    include_subfolders: bool,
    rows: &mut Vec<BaseRow>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/folders
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            if include_subfolders {
                scan_folder(&path, library_name, library_path, true, rows);
            }
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let properties = parse_frontmatter(&content).unwrap_or_default();
            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);

            let file_name = name.trim_end_matches(".md").to_string();

            rows.push(BaseRow {
                file_path: path.to_string_lossy().to_string(),
                file_name,
                library_name: library_name.to_string(),
                library_path: library_path.to_string(),
                properties,
                modified,
            });
        }
    }
}

/// Scan notes filtered by tag across a library.
pub fn scan_by_tag(
    dir: &Path,
    library_name: &str,
    library_path: &str,
    tag: &str,
    rows: &mut Vec<BaseRow>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let tag_clean = tag.trim_start_matches('#').to_lowercase();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') { continue; }

        if path.is_dir() {
            scan_by_tag(&path, library_name, library_path, tag, rows);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let properties = parse_frontmatter(&content).unwrap_or_default();

            // Check if note has the tag in frontmatter or body
            let has_tag = {
                // Check frontmatter tags property
                let fm_tags = properties.get("tags").map(|t| t.to_lowercase()).unwrap_or_default();
                let has_fm = fm_tags.split(',').any(|t| t.trim() == tag_clean);
                // Check body for #tag
                let has_body = content.contains(&format!("#{}", tag_clean));
                has_fm || has_body
            };

            if !has_tag { continue; }

            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);

            rows.push(BaseRow {
                file_path: path.to_string_lossy().to_string(),
                file_name: name.trim_end_matches(".md").to_string(),
                library_name: library_name.to_string(),
                library_path: library_path.to_string(),
                properties,
                modified,
            });
        }
    }
}

// ─── Filtering ───

pub fn apply_filters(rows: &mut Vec<BaseRow>, filters: &[FilterRule]) {
    for filter in filters {
        rows.retain(|row| {
            let value = if filter.property == "file_name" {
                Some(&row.file_name as &str)
            } else {
                row.properties.get(&filter.property).map(|s| s.as_str())
            };

            match filter.operator.as_str() {
                "is" => value.map(|v| v.to_lowercase() == filter.value.to_lowercase()).unwrap_or(false),
                "is_not" => value.map(|v| v.to_lowercase() != filter.value.to_lowercase()).unwrap_or(true),
                "contains" => value.map(|v| v.to_lowercase().contains(&filter.value.to_lowercase())).unwrap_or(false),
                "not_contains" => value.map(|v| !v.to_lowercase().contains(&filter.value.to_lowercase())).unwrap_or(true),
                "gt" => {
                    if let (Some(v), Ok(fv)) = (value, filter.value.parse::<f64>()) {
                        v.parse::<f64>().map(|nv| nv > fv).unwrap_or(false)
                    } else { false }
                },
                "lt" => {
                    if let (Some(v), Ok(fv)) = (value, filter.value.parse::<f64>()) {
                        v.parse::<f64>().map(|nv| nv < fv).unwrap_or(false)
                    } else { false }
                },
                "is_empty" => value.map(|v| v.is_empty()).unwrap_or(true),
                "is_not_empty" => value.map(|v| !v.is_empty()).unwrap_or(false),
                _ => true,
            }
        });
    }
}

// ─── Sorting ───

// ─── Tauri Commands ───

#[tauri::command]
pub fn parse_base_file(app: tauri::AppHandle, file_path: String) -> Result<BaseDefinition, String> {
    // Security: validate path is within a library or the active universe bases dir
    validate_base_path(&app, &file_path)?;

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read base file: {}", e))?;

    // Parse YAML
    serde_json::from_str::<BaseDefinition>(&content)
        .or_else(|_| {
            // Try parsing as YAML (simple key: value format)
            parse_base_yaml(&content)
        })
        .map_err(|e| format!("Failed to parse base file: {}", e))
}

/// Simple YAML-like parser for .base files.
/// For MVP, we use serde_json after converting YAML to JSON.
/// In production, add serde_yaml dependency.
fn parse_base_yaml(content: &str) -> Result<BaseDefinition, String> {
    // For now, try to parse as JSON first (the frontend will save as JSON)
    serde_json::from_str(content)
        .map_err(|e| format!("Invalid base file format: {}", e))
}

// ─── MIG-054 §A — SQL-backed query_base (Rule 8 compliance) ───
//
// Reads from note_meta.properties_json (write-time-derived frontmatter)
// instead of walking the filesystem. The cache the v1.4 Concept Paper §10.1
// mandate calls for already exists: note_meta is write-time-maintained by
// every upstream save / file-watcher / backfill path.
//
// Source types handled here (§A): "folder", "all", and legacy "vault" (the
// latter translated inline so existing .base files continue to work).
// Source type "tag" is implemented in §B via notes_fts MATCH; until then
// it returns an explicit error.
//
// 8 filter operators: is / is_not / contains / not_contains / gt / lt /
// is_empty / is_not_empty — all translated to json_extract expressions.
//
// columns_detected uses in-memory dedup in §A; §C upgrades to a json_each
// SQL query.

#[tauri::command]
pub fn query_base(
    app: tauri::AppHandle,
    definition: BaseDefinition,
    library_paths: Vec<(String, String)>, // (library_name, library_path) pairs
) -> Result<BaseQueryResult, String> {
    let start = Instant::now();

    // Filter libraries by selectedLibraries (empty = all libraries)
    let active_libs: Vec<String> = if definition.source.selected_vaults.is_empty() {
        library_paths.iter().map(|(n, _)| n.clone()).collect()
    } else {
        library_paths.iter()
            .filter(|(vname, _)| definition.source.selected_vaults.contains(vname))
            .map(|(n, _)| n.clone())
            .collect()
    };

    // Empty active_libs → empty result, no need to hit the DB
    if active_libs.is_empty() {
        return Ok(BaseQueryResult {
            rows: Vec::new(),
            total_count: 0,
            query_time_ms: start.elapsed().as_millis() as u64,
            columns_detected: Vec::new(),
        });
    }

    let library_path_map: HashMap<String, String> = library_paths.into_iter().collect();

    // Open the search DB
    let db_path = crate::search::db_path(&app)
        .map_err(|e| format!("Failed to resolve search DB path: {}", e))?;
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open search DB: {}", e))?;

    // Build the WHERE clause for the source type
    let (source_where, source_params) =
        build_source_where(&definition.source, &active_libs, &library_path_map)?;

    // Build filter clauses
    let (filter_sql, filter_params) = build_filter_clauses(&definition.filters);

    // Build ORDER BY
    let order_sql = build_order_clause(&definition.sorts);

    // Combine source + filter into a single WHERE expression — used by both the
    // main row query and the §C columns_detected query.
    let combined_where = if filter_sql.is_empty() {
        source_where.clone()
    } else {
        format!("{} AND {}", source_where, filter_sql)
    };
    let mut combined_params: Vec<String> = source_params;
    combined_params.extend(filter_params);

    // Build the main row query
    let mut sql = format!(
        "SELECT path, name, library_name, modified, properties_json FROM note_meta WHERE {}",
        combined_where
    );
    if !order_sql.is_empty() {
        sql.push(' ');
        sql.push_str(&order_sql);
    }

    // Execute
    let mut stmt = conn.prepare(&sql)
        .map_err(|e| format!("Failed to prepare query_base SQL: {}\nSQL: {}", e, sql))?;
    let row_iter = stmt.query_map(
        rusqlite::params_from_iter(combined_params.iter()),
        |row| {
            let path: String = row.get(0)?;
            let name: String = row.get(1)?;
            let library_name: String = row.get(2)?;
            let modified: i64 = row.get(3)?;
            let properties_json: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
            Ok((path, name, library_name, modified, properties_json))
        },
    ).map_err(|e| format!("Failed to execute query_base SQL: {}", e))?;

    let mut rows: Vec<BaseRow> = Vec::new();
    for row_result in row_iter {
        let (path, name, library_name, modified, properties_json) =
            row_result.map_err(|e| format!("Failed to read query_base row: {}", e))?;
        let properties = parse_properties_json(&properties_json);
        let library_path = library_path_map.get(&library_name).cloned().unwrap_or_default();
        rows.push(BaseRow {
            file_path: path,
            file_name: name.trim_end_matches(".md").to_string(),
            library_name,
            library_path,
            properties,
            modified: modified as u64,
        });
    }

    let total_count = rows.len();

    // §C — columns_detected via SQL json_each (replaces the in-memory HashSet dedup).
    // Operates on the same filtered row set as the main query (same WHERE), so the
    // distinct property keys returned are exactly those of the FILTERED notes.
    let columns_detected = detect_columns_sql(&conn, &combined_where, &combined_params)?;

    let query_time_ms = start.elapsed().as_millis() as u64;

    Ok(BaseQueryResult {
        rows,
        total_count,
        query_time_ms,
        columns_detected,
    })
}

// ─── §A helpers: SQL builders + JSON parsing ───

/// Parse a `note_meta.properties_json` TEXT value into a HashMap<String, String>.
/// Arrays are joined with ", " to match the old `parse_frontmatter` behavior; objects
/// are serialized back to JSON; null becomes empty string.
fn parse_properties_json(json_str: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();
    if json_str.is_empty() || json_str == "{}" {
        return props;
    }
    let value: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return props,
    };
    if let serde_json::Value::Object(map) = value {
        for (key, val) in map {
            let str_val = match val {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => String::new(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string().trim_matches('"').to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                serde_json::Value::Object(_) => val.to_string(),
            };
            props.insert(key, str_val);
        }
    }
    props
}

/// Build the WHERE clause for the source type. Returns (sql_fragment, params).
/// Params are bound for `library_name IN (?, ?, ...)`; path patterns are inlined
/// with single-quote escaping (LIKE patterns can't be parameterized cleanly).
fn build_source_where(
    source: &BaseSource,
    active_libs: &[String],
    library_paths: &HashMap<String, String>,
) -> Result<(String, Vec<String>), String> {
    let lib_placeholders: Vec<&str> = (0..active_libs.len()).map(|_| "?").collect();
    let lib_in = format!("library_name IN ({})", lib_placeholders.join(", "));

    match source.source_type.as_str() {
        "all" => Ok((lib_in, active_libs.to_vec())),
        "folder" => {
            let folder = source.path.as_deref().unwrap_or("");
            let sep = std::path::MAIN_SEPARATOR;
            let mut path_clauses: Vec<String> = Vec::new();
            for lib in active_libs {
                if let Some(lib_path) = library_paths.get(lib) {
                    // Construct the prefix: <lib_path><sep><folder><sep> (or <lib_path><sep> if folder empty)
                    let prefix = if folder.is_empty() {
                        format!("{}{}", lib_path, sep)
                    } else {
                        format!("{}{}{}{}", lib_path, sep, folder, sep)
                    };
                    // Escape single-quotes for SQL string literal
                    let escaped = prefix.replace('\'', "''");
                    if source.include_subfolders {
                        path_clauses.push(format!("path LIKE '{}%'", escaped));
                    } else {
                        // Direct children only — no further separator allowed after the prefix
                        path_clauses.push(format!(
                            "(path LIKE '{esc}%' AND path NOT LIKE '{esc}%{sep_esc}%')",
                            esc = escaped,
                            sep_esc = sep.to_string().replace('\'', "''")
                        ));
                    }
                }
            }
            let path_or = if path_clauses.is_empty() {
                "1=0".to_string()
            } else {
                format!("({})", path_clauses.join(" OR "))
            };
            Ok((format!("{} AND {}", lib_in, path_or), active_libs.to_vec()))
        }
        "tag" => {
            // MIG-054 §B — tag source via SQL.
            //
            // Two-pronged match (mirrors the old scan_by_tag):
            //   (a) Frontmatter tags_json (a JSON array maintained at write time).
            //       Match either "tag" or "#tag" form, case-insensitive, to be
            //       defensive about how upstream writers serialize tags.
            //   (b) Body text — search for literal "#<tag>" as a substring of
            //       body_text. body_text is Arabic-normalized at write time
            //       (per orientation §4.6 — tashkeel/tatweel removed); we apply
            //       the same normalization to the user's tag so multilingual
            //       hashtag matching works. Case-sensitive LIKE — mirrors the
            //       old implementation's `content.contains("#<tag_lower>")`.
            //
            // Note on FTS5: the Architect §5.4 considered notes_fts MATCH for
            // body-tag detection, but the default unicode61 tokenizer strips
            // the leading `#` (treats it as a separator). That would over-match
            // (every mention of the bare word, not just the #hashtag form).
            // The LIKE approach preserves OLD semantics exactly.
            let raw_tag = source
                .tag
                .as_deref()
                .unwrap_or("")
                .trim_start_matches('#');
            if raw_tag.is_empty() {
                return Err("Tag source type requires a tag value.".to_string());
            }
            let tag_lower = raw_tag.to_lowercase();
            let tag_normalized_lower =
                crate::arabic::normalizer::normalize_stripped(&tag_lower);

            let where_clause = format!(
                "{lib_in} AND (\
                 EXISTS (SELECT 1 FROM json_each(tags_json) \
                         WHERE LOWER(json_each.value) = ? OR LOWER(json_each.value) = ?) \
                 OR body_text LIKE ?\
                 )"
            );

            let mut params: Vec<String> = active_libs.to_vec();
            params.push(tag_lower.clone());
            params.push(format!("#{}", tag_lower));
            params.push(format!("%#{}%", tag_normalized_lower));

            Ok((where_clause, params))
        }
        "vault" => {
            // Legacy "vault" source type: translate inline to "all" + selected_libraries = [target].
            // §D retires the source-type literal entirely (rewrites on save); §A handles the
            // read-path so existing legacy .base files continue to work.
            let target = source
                .selected_vaults
                .first()
                .cloned()
                .or_else(|| source.path.clone())
                .unwrap_or_default();
            if target.is_empty() {
                return Err(
                    "Legacy 'vault' source type with no target library — base file is malformed."
                        .to_string(),
                );
            }
            Ok(("library_name = ?".to_string(), vec![target]))
        }
        other => Err(format!("Unknown source type: {}", other)),
    }
}

/// Build the filter clauses for the WHERE. Returns (sql_fragment, params_in_order).
fn build_filter_clauses(filters: &[FilterRule]) -> (String, Vec<String>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new();

    for filter in filters {
        let prop_expr = property_sql_expression(&filter.property);

        let clause = match filter.operator.as_str() {
            "is" => {
                params.push(filter.value.to_lowercase());
                format!("LOWER({}) = ?", prop_expr)
            }
            "is_not" => {
                params.push(filter.value.to_lowercase());
                format!("(LOWER({}) IS NULL OR LOWER({}) <> ?)", prop_expr, prop_expr)
            }
            "contains" => {
                params.push(format!("%{}%", filter.value.to_lowercase()));
                format!("LOWER({}) LIKE ?", prop_expr)
            }
            "not_contains" => {
                params.push(format!("%{}%", filter.value.to_lowercase()));
                format!("({} IS NULL OR LOWER({}) NOT LIKE ?)", prop_expr, prop_expr)
            }
            "gt" => {
                params.push(filter.value.clone());
                format!("CAST({} AS REAL) > CAST(? AS REAL)", prop_expr)
            }
            "lt" => {
                params.push(filter.value.clone());
                format!("CAST({} AS REAL) < CAST(? AS REAL)", prop_expr)
            }
            "is_empty" => {
                format!("({} IS NULL OR {} = '')", prop_expr, prop_expr)
            }
            "is_not_empty" => {
                format!("({} IS NOT NULL AND {} <> '')", prop_expr, prop_expr)
            }
            _ => "1=1".to_string(), // unknown operator: no-op filter
        };
        clauses.push(clause);
    }

    let filter_sql = if clauses.is_empty() {
        String::new()
    } else {
        clauses.join(" AND ")
    };
    (filter_sql, params)
}

/// Build the ORDER BY clause for the given sorts.
fn build_order_clause(sorts: &[SortRule]) -> String {
    if sorts.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = sorts
        .iter()
        .map(|sort| {
            let prop_expr = property_sql_expression(&sort.property);
            let direction = if sort.direction == "desc" { "DESC" } else { "ASC" };
            // COLLATE NOCASE for case-insensitive string sort; numeric comparison falls out
            // of CAST in the WHERE path. Sort here is lexicographic; behavioral equivalence
            // matches the old apply_sorts_fixed which fell back to lowercased string compare
            // when numeric parsing failed.
            format!("{} COLLATE NOCASE {}", prop_expr, direction)
        })
        .collect();

    format!("ORDER BY {}", parts.join(", "))
}

/// §C — `columns_detected` via SQL json_each.
///
/// Runs a `SELECT DISTINCT key FROM note_meta, json_each(properties_json)
/// WHERE <combined WHERE>` against the same filter shape as the main query.
/// Returns property keys sorted ascending (same shape as the §A in-memory
/// dedup it replaces).
fn detect_columns_sql(
    conn: &Connection,
    where_sql: &str,
    where_params: &[String],
) -> Result<Vec<String>, String> {
    let sql = format!(
        "SELECT DISTINCT json_each.key \
         FROM note_meta, json_each(note_meta.properties_json) \
         WHERE {} \
         ORDER BY json_each.key",
        where_sql
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare detect_columns_sql: {}\nSQL: {}", e, sql))?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(where_params.iter()),
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| format!("Failed to execute detect_columns_sql: {}", e))?;

    let mut keys: Vec<String> = Vec::new();
    for row_result in rows {
        let key = row_result.map_err(|e| format!("Failed to read column key: {}", e))?;
        // Defensive: skip any non-string keys (json_each returns the column index for arrays;
        // properties_json is always an object so this should never trigger, but be safe).
        if !key.is_empty() {
            keys.push(key);
        }
    }
    Ok(keys)
}

/// Translate a property name to the SQL expression that yields its value.
/// Special properties: file_name, modified. Everything else is a json_extract.
fn property_sql_expression(property: &str) -> String {
    if property == "file_name" {
        // Match the old behavioral semantics: file_name is the name without .md suffix.
        // SQLite has no clean suffix-strip; use CASE WHEN to handle the optional .md.
        "CASE WHEN name LIKE '%.md' THEN substr(name, 1, length(name) - 3) ELSE name END"
            .to_string()
    } else if property == "modified" {
        "modified".to_string()
    } else {
        // Use the JSON path '$.\"<prop>\"' form for safety — handles property names with spaces,
        // dots, special characters. Escape any embedded single-quotes and double-quotes.
        let escaped = property.replace('\'', "''").replace('"', "\\\"");
        format!("json_extract(properties_json, '$.\"{}\"')", escaped)
    }
}

/// Fixed sorting that handles owned strings properly.
pub fn apply_sorts_fixed(rows: &mut Vec<BaseRow>, sorts: &[SortRule]) {
    if sorts.is_empty() { return; }

    rows.sort_by(|a, b| {
        for sort in sorts {
            let av = if sort.property == "file_name" {
                a.file_name.clone()
            } else if sort.property == "modified" {
                a.modified.to_string()
            } else {
                a.properties.get(&sort.property).cloned().unwrap_or_default()
            };
            let bv = if sort.property == "file_name" {
                b.file_name.clone()
            } else if sort.property == "modified" {
                b.modified.to_string()
            } else {
                b.properties.get(&sort.property).cloned().unwrap_or_default()
            };

            // Try numeric comparison first
            let ord = match (av.parse::<f64>(), bv.parse::<f64>()) {
                (Ok(an), Ok(bn)) => an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal),
                _ => av.to_lowercase().cmp(&bv.to_lowercase()),
            };

            let ord = if sort.direction == "desc" { ord.reverse() } else { ord };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    });
}

#[tauri::command]
pub fn create_base(
    app: tauri::AppHandle,
    folder_path: String,
    file_name: String,
) -> Result<String, String> {
    // Validate folder is in a registered library
    let libraries = crate::libraries::load_libraries_pub(&app);
    let folder = Path::new(&folder_path);
    let canon_folder = fs::canonicalize(folder)
        .map_err(|_| "Folder does not exist.".to_string())?;
    let in_library = libraries.iter().any(|v| {
        fs::canonicalize(&v.path)
            .map(|vp| canon_folder.starts_with(vp))
            .unwrap_or(false)
    });
    if !in_library {
        return Err("Access denied: path is not within any registered library.".to_string());
    }
    if !folder.is_dir() {
        return Err("Folder does not exist.".to_string());
    }

    // Sanitize name
    let safe_name = file_name.trim().replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "");
    if safe_name.is_empty() {
        return Err("Invalid file name.".to_string());
    }

    let name = if safe_name.ends_with(".base") {
        safe_name
    } else {
        format!("{}.base", safe_name)
    };

    let file_path = folder.join(&name);
    if file_path.exists() {
        return Err("A file with this name already exists.".to_string());
    }

    // Build default BaseDefinition
    let display_name = name.trim_end_matches(".base").to_string();
    let definition = BaseDefinition {
        version: 1,
        name: display_name,
        source: BaseSource {
            source_type: "all".to_string(),
            path: None,
            tag: None,
            include_subfolders: true,
            selected_vaults: vec![],
        },
        columns: vec![],
        filters: vec![],
        sorts: vec![],
        view: "table".to_string(),
        direction: "auto".to_string(),
    };

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to create base file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_base_file(app: tauri::AppHandle, file_path: String, definition: BaseDefinition) -> Result<(), String> {
    // Security: validate path is within a library or the active universe bases dir
    validate_base_path(&app, &file_path)?;

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write base file: {}", e))
}

#[tauri::command]
pub fn update_note_property(
    app: tauri::AppHandle,
    file_path: String,
    key: String,
    value: String,
) -> Result<(), String> {
    // Security: validate path is in a library
    let libraries = crate::libraries::load_libraries_pub(&app);
    let in_library = libraries.iter().any(|v| {
        fs::canonicalize(&file_path).ok()
            .and_then(|fp| fs::canonicalize(&v.path).ok().map(|vp| fp.starts_with(vp)))
            .unwrap_or(false)
    });
    if !in_library {
        return Err("Access denied: file is not in a registered library.".to_string());
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;

    let new_content = update_frontmatter_property(&content, &key, &value);

    fs::write(&file_path, new_content)
        .map_err(|e| format!("Failed to write note: {}", e))
}

/// Update or insert a single property in a note's YAML frontmatter.
fn update_frontmatter_property(content: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if !content.starts_with("---") {
        // No frontmatter — create one
        let mut result = format!("---\n{}: {}\n---\n", key, format_yaml_value(value));
        result.push_str(content);
        return result;
    }

    let end_idx = lines.iter().skip(1).position(|l| l.trim() == "---");
    let end_idx = match end_idx {
        Some(i) => i + 1,
        None => {
            // Malformed frontmatter — prepend new one
            let mut result = format!("---\n{}: {}\n---\n", key, format_yaml_value(value));
            result.push_str(content);
            return result;
        }
    };

    // Check if property already exists
    let mut found = false;
    let mut new_lines: Vec<String> = Vec::new();
    new_lines.push("---".to_string());

    let mut i = 1;
    while i < end_idx {
        let line = lines[i];
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim();
            if !k.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                if k == key {
                    // Replace existing value
                    new_lines.push(format!("{}: {}", key, format_yaml_value(value)));
                    found = true;
                    // Skip any continuation lines (multi-line list)
                    i += 1;
                    while i < end_idx && (lines[i].starts_with("  - ") || lines[i].starts_with("  ")) {
                        if lines[i].trim().starts_with("- ") {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
        }
        new_lines.push(line.to_string());
        i += 1;
    }

    if !found {
        new_lines.push(format!("{}: {}", key, format_yaml_value(value)));
    }

    new_lines.push("---".to_string());

    // Append body (everything after frontmatter)
    for line in &lines[end_idx + 1..] {
        new_lines.push(line.to_string());
    }

    new_lines.join("\n")
}

// ─── Workspace-level Base Storage ───

/// Get the workspace bases directory: {active_universe}/.constellation/bases/
fn workspace_bases_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    let bases_dir = cdir.join("bases");
    fs::create_dir_all(&bases_dir).map_err(|e| format!("Failed to create bases dir: {}", e))?;
    Ok(bases_dir)
}

/// Workspace base entry returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceBaseEntry {
    pub id: String,        // file stem (e.g. "My Research")
    pub name: String,      // display name from definition
    pub path: String,      // full file path
    pub modified: u64,     // last modified timestamp
}

#[tauri::command]
pub fn list_workspace_bases(app: tauri::AppHandle) -> Result<Vec<WorkspaceBaseEntry>, String> {
    let dir = workspace_bases_dir(&app)?;
    let mut entries = Vec::new();

    let read = fs::read_dir(&dir).map_err(|e| format!("Failed to read workspace bases: {}", e))?;
    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "base").unwrap_or(false) {
            let id = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // Try to read the name from the definition
            let name = fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str::<BaseDefinition>(&c).ok())
                .map(|d| d.name)
                .unwrap_or_else(|| id.clone());

            let modified = entry.metadata()
                .and_then(|m| m.modified())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                .unwrap_or(0);

            entries.push(WorkspaceBaseEntry {
                id,
                name,
                path: path.to_string_lossy().to_string(),
                modified,
            });
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

#[tauri::command]
pub fn create_workspace_base(
    app: tauri::AppHandle,
    file_name: String,
) -> Result<String, String> {
    let dir = workspace_bases_dir(&app)?;

    // Sanitize name
    let safe_name = file_name.trim().replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "");
    if safe_name.is_empty() {
        return Err("Invalid file name.".to_string());
    }

    let name = if safe_name.ends_with(".base") {
        safe_name
    } else {
        format!("{}.base", safe_name)
    };

    let file_path = dir.join(&name);
    if file_path.exists() {
        return Err("A base with this name already exists.".to_string());
    }

    let display_name = name.trim_end_matches(".base").to_string();
    let definition = BaseDefinition {
        version: 1,
        name: display_name,
        source: BaseSource {
            source_type: "all".to_string(),
            path: None,
            tag: None,
            include_subfolders: true,
            selected_vaults: vec![],
        },
        columns: vec![],
        filters: vec![],
        sorts: vec![],
        view: "table".to_string(),
        direction: "auto".to_string(),
    };

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, &content)
        .map_err(|e| format!("Failed to create workspace base: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn save_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
    definition: BaseDefinition,
) -> Result<(), String> {
    // Validate the path is inside the workspace bases directory
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Invalid workspace bases directory.".to_string())?;
    // For new files that don't exist yet, canonicalize the parent directory and
    // append only the filename — avoids raw-path starts_with bypass via ".." components.
    let canon_target = if target.exists() {
        fs::canonicalize(target)
            .map_err(|_| "Invalid target path.".to_string())?
    } else {
        let parent = target.parent().ok_or("Invalid target path.".to_string())?;
        let canon_parent = fs::canonicalize(parent)
            .map_err(|_| "Parent directory does not exist.".to_string())?;
        let fname = target.file_name().ok_or("Invalid file name.".to_string())?;
        canon_parent.join(fname)
    };

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    let content = serde_json::to_string_pretty(&definition)
        .map_err(|e| format!("Failed to serialize base: {}", e))?;
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write workspace base: {}", e))
}

#[tauri::command]
pub fn delete_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<(), String> {
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);

    // Validate path is inside workspace bases directory
    let canon_target = fs::canonicalize(target)
        .map_err(|_| "File does not exist.".to_string())?;
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Workspace directory not found.".to_string())?;

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete workspace base: {}", e))
}

#[tauri::command]
pub fn parse_workspace_base(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<BaseDefinition, String> {
    let bases_dir = workspace_bases_dir(&app)?;
    let target = Path::new(&file_path);

    // Validate path is inside workspace bases directory
    let canon_target = fs::canonicalize(target)
        .map_err(|_| "File does not exist.".to_string())?;
    let canon_dir = fs::canonicalize(&bases_dir)
        .map_err(|_| "Workspace directory not found.".to_string())?;

    if !canon_target.starts_with(&canon_dir) {
        return Err("Access denied: path is not within workspace bases directory.".to_string());
    }

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read workspace base: {}", e))?;

    serde_json::from_str::<BaseDefinition>(&content)
        .map_err(|e| format!("Failed to parse workspace base: {}", e))
}

/// Format a value for YAML output.
/// Quotes strings that contain special characters.
fn format_yaml_value(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }
    // Check if value needs quoting
    if value.contains(':') || value.contains('#') || value.contains('\'')
        || value.contains('"') || value.contains('\n') || value.starts_with(' ')
        || value.ends_with(' ') || value.starts_with('[') || value.starts_with('{')
    {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

// ─── MIG-054 §A — Unit tests for the SQL-builder helpers ───

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_properties_json ──

    #[test]
    fn parse_properties_json_empty() {
        assert_eq!(parse_properties_json("").len(), 0);
        assert_eq!(parse_properties_json("{}").len(), 0);
    }

    #[test]
    fn parse_properties_json_string_value() {
        let m = parse_properties_json(r#"{"status":"in-progress"}"#);
        assert_eq!(m.get("status"), Some(&"in-progress".to_string()));
    }

    #[test]
    fn parse_properties_json_number_value() {
        let m = parse_properties_json(r#"{"pages":42}"#);
        assert_eq!(m.get("pages"), Some(&"42".to_string()));
    }

    #[test]
    fn parse_properties_json_bool_value() {
        let m = parse_properties_json(r#"{"done":true}"#);
        assert_eq!(m.get("done"), Some(&"true".to_string()));
    }

    #[test]
    fn parse_properties_json_null_value() {
        let m = parse_properties_json(r#"{"author":null}"#);
        assert_eq!(m.get("author"), Some(&"".to_string()));
    }

    #[test]
    fn parse_properties_json_array_value() {
        let m = parse_properties_json(r#"{"tags":["a","b","c"]}"#);
        assert_eq!(m.get("tags"), Some(&"a, b, c".to_string()));
    }

    #[test]
    fn parse_properties_json_arabic_keys_and_values() {
        let m = parse_properties_json(r#"{"العنوان":"عيسى","العمر":42}"#);
        assert_eq!(m.get("العنوان"), Some(&"عيسى".to_string()));
        assert_eq!(m.get("العمر"), Some(&"42".to_string()));
    }

    #[test]
    fn parse_properties_json_invalid_returns_empty() {
        assert_eq!(parse_properties_json("not json").len(), 0);
    }

    // ── property_sql_expression ──

    #[test]
    fn property_sql_expression_file_name() {
        let s = property_sql_expression("file_name");
        assert!(s.contains("substr(name"));
        assert!(s.contains(".md"));
    }

    #[test]
    fn property_sql_expression_modified() {
        assert_eq!(property_sql_expression("modified"), "modified");
    }

    #[test]
    fn property_sql_expression_regular() {
        let s = property_sql_expression("status");
        assert!(s.contains("json_extract"));
        assert!(s.contains("status"));
    }

    #[test]
    fn property_sql_expression_with_double_quote() {
        let s = property_sql_expression("foo\"bar");
        assert!(s.contains("foo\\\"bar"));
    }

    #[test]
    fn property_sql_expression_arabic() {
        let s = property_sql_expression("عنوان");
        assert!(s.contains("عنوان"));
    }

    // ── build_filter_clauses ──

    #[test]
    fn build_filter_clauses_empty() {
        let (sql, params) = build_filter_clauses(&[]);
        assert_eq!(sql, "");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn build_filter_clauses_is() {
        let filters = vec![FilterRule {
            property: "status".to_string(),
            operator: "is".to_string(),
            value: "Done".to_string(),
        }];
        let (sql, params) = build_filter_clauses(&filters);
        assert!(sql.contains("LOWER("));
        assert!(sql.contains("= ?"));
        assert_eq!(params, vec!["done".to_string()]);
    }

    #[test]
    fn build_filter_clauses_is_not() {
        let filters = vec![FilterRule {
            property: "status".to_string(),
            operator: "is_not".to_string(),
            value: "Done".to_string(),
        }];
        let (sql, params) = build_filter_clauses(&filters);
        assert!(sql.contains("IS NULL OR"));
        assert!(sql.contains("<> ?"));
        assert_eq!(params, vec!["done".to_string()]);
    }

    #[test]
    fn build_filter_clauses_contains() {
        let filters = vec![FilterRule {
            property: "tags".to_string(),
            operator: "contains".to_string(),
            value: "Aristotle".to_string(),
        }];
        let (sql, params) = build_filter_clauses(&filters);
        assert!(sql.contains("LIKE ?"));
        assert_eq!(params, vec!["%aristotle%".to_string()]);
    }

    #[test]
    fn build_filter_clauses_gt() {
        let filters = vec![FilterRule {
            property: "pages".to_string(),
            operator: "gt".to_string(),
            value: "100".to_string(),
        }];
        let (sql, params) = build_filter_clauses(&filters);
        assert!(sql.contains("CAST"));
        assert!(sql.contains("> CAST"));
        assert_eq!(params, vec!["100".to_string()]);
    }

    #[test]
    fn build_filter_clauses_lt() {
        let filters = vec![FilterRule {
            property: "pages".to_string(),
            operator: "lt".to_string(),
            value: "100".to_string(),
        }];
        let (sql, params) = build_filter_clauses(&filters);
        assert!(sql.contains("< CAST"));
        assert_eq!(params, vec!["100".to_string()]);
    }

    #[test]
    fn build_filter_clauses_is_empty_no_param() {
        let filters = vec![FilterRule {
            property: "status".to_string(),
            operator: "is_empty".to_string(),
            value: "".to_string(),
        }];
        let (sql, params) = build_filter_clauses(&filters);
        assert!(sql.contains("IS NULL"));
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn build_filter_clauses_is_not_empty_no_param() {
        let filters = vec![FilterRule {
            property: "status".to_string(),
            operator: "is_not_empty".to_string(),
            value: "".to_string(),
        }];
        let (sql, params) = build_filter_clauses(&filters);
        assert!(sql.contains("IS NOT NULL"));
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn build_filter_clauses_multiple_anded() {
        let filters = vec![
            FilterRule {
                property: "status".to_string(),
                operator: "is".to_string(),
                value: "done".to_string(),
            },
            FilterRule {
                property: "tags".to_string(),
                operator: "contains".to_string(),
                value: "aristotle".to_string(),
            },
        ];
        let (sql, params) = build_filter_clauses(&filters);
        assert!(sql.contains(" AND "));
        assert_eq!(params.len(), 2);
    }

    // ── build_order_clause ──

    #[test]
    fn build_order_clause_empty() {
        assert_eq!(build_order_clause(&[]), "");
    }

    #[test]
    fn build_order_clause_single_desc() {
        let sorts = vec![SortRule {
            property: "modified".to_string(),
            direction: "desc".to_string(),
        }];
        let sql = build_order_clause(&sorts);
        assert!(sql.starts_with("ORDER BY"));
        assert!(sql.contains("modified"));
        assert!(sql.contains("DESC"));
    }

    #[test]
    fn build_order_clause_default_asc() {
        let sorts = vec![SortRule {
            property: "file_name".to_string(),
            direction: "asc".to_string(),
        }];
        let sql = build_order_clause(&sorts);
        assert!(sql.contains("ASC"));
    }

    #[test]
    fn build_order_clause_multi() {
        let sorts = vec![
            SortRule {
                property: "stratum".to_string(),
                direction: "desc".to_string(),
            },
            SortRule {
                property: "file_name".to_string(),
                direction: "asc".to_string(),
            },
        ];
        let sql = build_order_clause(&sorts);
        assert!(sql.contains(","));
        assert!(sql.contains("DESC"));
        assert!(sql.contains("ASC"));
    }

    // ── build_source_where ──

    fn test_lib_paths() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("Lib1".to_string(), format!("{sep}path{sep}to{sep}Lib1", sep = std::path::MAIN_SEPARATOR));
        m.insert("Lib2".to_string(), format!("{sep}path{sep}to{sep}Lib2", sep = std::path::MAIN_SEPARATOR));
        m
    }

    #[test]
    fn build_source_where_all() {
        let source = BaseSource {
            source_type: "all".to_string(),
            path: None,
            tag: None,
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string(), "Lib2".to_string()];
        let (sql, params) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        assert!(sql.starts_with("library_name IN"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn build_source_where_folder_subfolders_included() {
        let source = BaseSource {
            source_type: "folder".to_string(),
            path: Some("Projects".to_string()),
            tag: None,
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let (sql, _) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        assert!(sql.contains("library_name IN"));
        assert!(sql.contains("path LIKE"));
        assert!(sql.contains("Projects"));
        // No NOT LIKE restriction when subfolders included
        assert!(!sql.contains("NOT LIKE"));
    }

    #[test]
    fn build_source_where_folder_no_subfolders() {
        let source = BaseSource {
            source_type: "folder".to_string(),
            path: Some("Projects".to_string()),
            tag: None,
            include_subfolders: false,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let (sql, _) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        // When subfolders excluded, the NOT LIKE restriction appears
        assert!(sql.contains("NOT LIKE"));
    }

    #[test]
    fn build_source_where_tag_sql_shape() {
        let source = BaseSource {
            source_type: "tag".to_string(),
            path: None,
            tag: Some("aristotle".to_string()),
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let (sql, params) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        assert!(sql.contains("library_name IN"));
        assert!(sql.contains("json_each(tags_json)"));
        assert!(sql.contains("body_text LIKE ?"));
        // Three tag-related params after the library_name params:
        //   [Lib1, "aristotle", "#aristotle", "%#aristotle%"]
        assert_eq!(params.len(), 4);
        assert_eq!(params[0], "Lib1");
        assert_eq!(params[1], "aristotle");
        assert_eq!(params[2], "#aristotle");
        assert_eq!(params[3], "%#aristotle%");
    }

    #[test]
    fn build_source_where_tag_strips_leading_hash_in_user_input() {
        let source = BaseSource {
            source_type: "tag".to_string(),
            path: None,
            tag: Some("#aristotle".to_string()),
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let (_, params) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        assert_eq!(params[1], "aristotle"); // The leading # has been stripped
        assert_eq!(params[2], "#aristotle"); // The #-prefixed form is built fresh
    }

    #[test]
    fn build_source_where_tag_case_normalizes_to_lower() {
        let source = BaseSource {
            source_type: "tag".to_string(),
            path: None,
            tag: Some("Aristotle".to_string()),
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let (_, params) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        assert_eq!(params[1], "aristotle"); // lower-cased
    }

    #[test]
    fn build_source_where_tag_arabic_normalizes() {
        // Arabic tag with tashkeel — the normalize_stripped() pass should strip the diacritic
        // for the body_text LIKE param (since body_text was Arabic-normalized at write time)
        // while preserving the raw form for frontmatter tags_json matching.
        let source = BaseSource {
            source_type: "tag".to_string(),
            path: None,
            tag: Some("الْإمارات".to_string()), // tashkeel sukun on lam
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let (_, params) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        // Frontmatter params preserve the user's original form (Arabic was lowercased no-op):
        assert_eq!(params[1], "الْإمارات");
        // Body param has the diacritic stripped:
        assert_eq!(params[3], "%#الإمارات%");
    }

    #[test]
    fn build_source_where_tag_empty_returns_error() {
        let source = BaseSource {
            source_type: "tag".to_string(),
            path: None,
            tag: Some("".to_string()),
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let result = build_source_where(&source, &active_libs, &test_lib_paths());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires a tag"));
    }

    #[test]
    fn build_source_where_tag_only_hash_returns_error() {
        // "#" alone (no actual tag) → after trim_start_matches('#'), empty → error
        let source = BaseSource {
            source_type: "tag".to_string(),
            path: None,
            tag: Some("#".to_string()),
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let result = build_source_where(&source, &active_libs, &test_lib_paths());
        assert!(result.is_err());
    }

    #[test]
    fn build_source_where_legacy_vault_from_path() {
        let source = BaseSource {
            source_type: "vault".to_string(),
            path: Some("Lib1".to_string()),
            tag: None,
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let (sql, params) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        assert!(sql.contains("library_name = ?"));
        assert_eq!(params, vec!["Lib1".to_string()]);
    }

    #[test]
    fn build_source_where_legacy_vault_selected_wins_over_path() {
        let source = BaseSource {
            source_type: "vault".to_string(),
            path: Some("Lib1".to_string()),
            tag: None,
            include_subfolders: true,
            selected_vaults: vec!["Lib2".to_string()],
        };
        let active_libs = vec!["Lib1".to_string(), "Lib2".to_string()];
        let (_, params) = build_source_where(&source, &active_libs, &test_lib_paths()).unwrap();
        assert_eq!(params, vec!["Lib2".to_string()]); // selected_vaults wins
    }

    #[test]
    fn build_source_where_unknown_returns_error() {
        let source = BaseSource {
            source_type: "unknown_type".to_string(),
            path: None,
            tag: None,
            include_subfolders: true,
            selected_vaults: Vec::new(),
        };
        let active_libs = vec!["Lib1".to_string()];
        let result = build_source_where(&source, &active_libs, &test_lib_paths());
        assert!(result.is_err());
    }
}
