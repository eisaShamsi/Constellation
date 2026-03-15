use std::fs;
use std::path::Path;
use std::time::Instant;
use serde::{Deserialize, Serialize};

use crate::bases::{
    BaseRow, FilterRule, SortRule,
    scan_folder, scan_by_tag, apply_filters, apply_sorts_fixed,
};

/// Dataview query types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum QueryType {
    #[serde(rename = "table")]
    Table,
    #[serde(rename = "list")]
    List,
    #[serde(rename = "task")]
    Task,
    #[serde(rename = "calendar")]
    Calendar,
}

/// Parsed dataview query from DQL syntax
#[derive(Debug, Serialize, Deserialize)]
pub struct DataviewQuery {
    pub query_type: QueryType,
    pub columns: Vec<String>,
    pub source_type: String,      // "folder", "tag", "all"
    pub source_value: String,     // folder path or tag name
    pub filters: Vec<FilterRule>,
    pub sorts: Vec<SortRule>,
    pub limit: Option<usize>,
    pub group_by: Option<String>,
}

/// Result of a dataview query execution
#[derive(Debug, Serialize, Deserialize)]
pub struct DataviewResult {
    pub query_type: String,
    pub rows: Vec<BaseRow>,
    pub columns: Vec<String>,
    pub total_count: usize,
    pub query_time_ms: u64,
    pub group_by: Option<String>,
    pub error: Option<String>,
}

/// Parse a DQL query string into a structured DataviewQuery.
///
/// Supported syntax:
///   TABLE prop1, prop2 FROM "folder/path" WHERE prop = "value" SORT prop ASC LIMIT 20
///   LIST FROM #tag WHERE status != "done"
///   TABLE prop FROM "folder" WHERE prop > "5" SORT prop DESC
///   TABLE FROM "" (all notes)
fn parse_dql(query: &str) -> Result<DataviewQuery, String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("Empty query".to_string());
    }

    // Tokenize: split by whitespace but preserve quoted strings
    let tokens = tokenize_dql(query);
    if tokens.is_empty() {
        return Err("Empty query".to_string());
    }

    // Parse query type
    let query_type = match tokens[0].to_uppercase().as_str() {
        "TABLE" => QueryType::Table,
        "LIST" => QueryType::List,
        "TASK" => QueryType::Task,
        "CALENDAR" => QueryType::Calendar,
        _ => return Err(format!("Unknown query type: {}. Use TABLE, LIST, TASK, or CALENDAR.", tokens[0])),
    };

    let mut idx = 1;
    let mut columns: Vec<String> = Vec::new();
    let mut source_type = "all".to_string();
    let mut source_value = String::new();
    let mut filters: Vec<FilterRule> = Vec::new();
    let mut sorts: Vec<SortRule> = Vec::new();
    let mut limit: Option<usize> = None;
    let mut group_by: Option<String> = None;

    // Parse columns (before FROM) — only for TABLE
    if matches!(query_type, QueryType::Table) {
        while idx < tokens.len() && tokens[idx].to_uppercase() != "FROM"
            && tokens[idx].to_uppercase() != "WHERE"
            && tokens[idx].to_uppercase() != "SORT"
            && tokens[idx].to_uppercase() != "LIMIT"
            && tokens[idx].to_uppercase() != "GROUP"
        {
            let col = tokens[idx].trim_matches(',').trim().to_string();
            if !col.is_empty() {
                columns.push(col);
            }
            idx += 1;
        }
    }

    // Parse FROM clause
    if idx < tokens.len() && tokens[idx].to_uppercase() == "FROM" {
        idx += 1;
        if idx < tokens.len() {
            let src = &tokens[idx];
            if src.starts_with('#') {
                source_type = "tag".to_string();
                source_value = src[1..].to_string();
            } else if src == "\"\"" || src.is_empty() {
                source_type = "all".to_string();
            } else {
                source_type = "folder".to_string();
                source_value = src.trim_matches('"').to_string();
            }
            idx += 1;
        }
    }

    // Parse WHERE clause
    if idx < tokens.len() && tokens[idx].to_uppercase() == "WHERE" {
        idx += 1;
        while idx < tokens.len()
            && tokens[idx].to_uppercase() != "SORT"
            && tokens[idx].to_uppercase() != "LIMIT"
            && tokens[idx].to_uppercase() != "GROUP"
        {
            // Skip AND/OR connectors (we treat all as AND for now)
            if tokens[idx].to_uppercase() == "AND" || tokens[idx].to_uppercase() == "OR" {
                idx += 1;
                continue;
            }

            // Parse: property operator value
            if idx + 2 < tokens.len() {
                let property = tokens[idx].clone();
                let op_raw = tokens[idx + 1].to_uppercase();
                let value = tokens[idx + 2].trim_matches('"').to_string();

                let operator = match op_raw.as_str() {
                    "=" | "==" | "IS" => "is".to_string(),
                    "!=" | "IS_NOT" => "is_not".to_string(),
                    ">" | "GT" => "gt".to_string(),
                    "<" | "LT" => "lt".to_string(),
                    ">=" | "GTE" => "gt".to_string(), // approximate
                    "<=" | "LTE" => "lt".to_string(), // approximate
                    "CONTAINS" => "contains".to_string(),
                    "NOT_CONTAINS" => "not_contains".to_string(),
                    _ => "contains".to_string(),
                };

                filters.push(FilterRule {
                    property,
                    operator,
                    value,
                });
                idx += 3;
            } else {
                // Handle unary operators: property IS_EMPTY / IS_NOT_EMPTY
                if idx + 1 < tokens.len() {
                    let property = tokens[idx].clone();
                    let op = tokens[idx + 1].to_uppercase();
                    if op == "IS_EMPTY" {
                        filters.push(FilterRule {
                            property,
                            operator: "is_empty".to_string(),
                            value: String::new(),
                        });
                        idx += 2;
                    } else if op == "IS_NOT_EMPTY" {
                        filters.push(FilterRule {
                            property,
                            operator: "is_not_empty".to_string(),
                            value: String::new(),
                        });
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                } else {
                    idx += 1;
                }
            }
        }
    }

    // Parse SORT clause
    if idx < tokens.len() && tokens[idx].to_uppercase() == "SORT" {
        idx += 1;
        while idx < tokens.len()
            && tokens[idx].to_uppercase() != "LIMIT"
            && tokens[idx].to_uppercase() != "GROUP"
        {
            let property = tokens[idx].clone();
            idx += 1;
            let direction = if idx < tokens.len() {
                let d = tokens[idx].to_uppercase();
                if d == "ASC" || d == "DESC" {
                    idx += 1;
                    d.to_lowercase()
                } else {
                    "asc".to_string()
                }
            } else {
                "asc".to_string()
            };
            // Skip comma separator
            if idx < tokens.len() && tokens[idx] == "," {
                idx += 1;
            }
            sorts.push(SortRule { property, direction });
        }
    }

    // Parse GROUP BY clause
    if idx < tokens.len() && tokens[idx].to_uppercase() == "GROUP" {
        idx += 1;
        if idx < tokens.len() && tokens[idx].to_uppercase() == "BY" {
            idx += 1;
        }
        if idx < tokens.len() {
            group_by = Some(tokens[idx].clone());
            idx += 1;
        }
    }

    // Parse LIMIT clause
    if idx < tokens.len() && tokens[idx].to_uppercase() == "LIMIT" {
        idx += 1;
        if idx < tokens.len() {
            if let Ok(n) = tokens[idx].parse::<usize>() {
                limit = Some(n);
            }
        }
    }

    Ok(DataviewQuery {
        query_type,
        columns,
        source_type,
        source_value,
        filters,
        sorts,
        limit,
        group_by,
    })
}

/// Tokenize DQL query string, preserving quoted strings as single tokens
fn tokenize_dql(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        if ch == '"' {
            if in_quotes {
                // End of quoted token
                tokens.push(current.clone());
                current.clear();
                in_quotes = false;
            } else {
                // Start of quoted token — push any pending token first
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
                in_quotes = true;
            }
        } else if ch.is_whitespace() && !in_quotes {
            if !current.trim().is_empty() {
                tokens.push(current.trim().to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }
    tokens
}

/// Execute a Dataview query against the given libraries.
///
/// `query_text` is the raw DQL query string.
/// `library_paths` is a list of (library_name, library_path) tuples.
#[tauri::command]
pub fn execute_dataview_query(
    _app: tauri::AppHandle,
    query_text: String,
    library_paths: Vec<(String, String)>,
) -> DataviewResult {
    let start = Instant::now();

    // Parse the DQL query
    let query = match parse_dql(&query_text) {
        Ok(q) => q,
        Err(e) => {
            return DataviewResult {
                query_type: "error".to_string(),
                rows: vec![],
                columns: vec![],
                total_count: 0,
                query_time_ms: start.elapsed().as_millis() as u64,
                group_by: None,
                error: Some(e),
            };
        }
    };

    // Collect rows using the same scan functions as bases
    let mut rows: Vec<BaseRow> = Vec::new();

    match query.source_type.as_str() {
        "folder" => {
            for (library_name, library_path) in &library_paths {
                let full_path = Path::new(library_path).join(&query.source_value);
                // Validate the resolved path stays within the library (prevents path traversal)
                let library_canon = match fs::canonicalize(library_path) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let full_path_canon = match fs::canonicalize(&full_path) {
                    Ok(p) => p,
                    Err(_) => continue, // Path doesn't exist or is invalid
                };
                if !full_path_canon.starts_with(&library_canon) {
                    continue; // Silently skip: path escapes library boundary
                }
                scan_folder(&full_path_canon, library_name, library_path, true, &mut rows);
            }
        }
        "tag" => {
            for (library_name, library_path) in &library_paths {
                let vp = Path::new(library_path);
                scan_by_tag(vp, library_name, library_path, &query.source_value, &mut rows);
            }
        }
        _ => {
            // "all" — scan all libraries
            for (library_name, library_path) in &library_paths {
                let vp = Path::new(library_path);
                scan_folder(vp, library_name, library_path, true, &mut rows);
            }
        }
    }

    let total_count = rows.len();

    // Apply filters
    if !query.filters.is_empty() {
        apply_filters(&mut rows, &query.filters);
    }

    // Apply sorts
    if !query.sorts.is_empty() {
        apply_sorts_fixed(&mut rows, &query.sorts);
    }

    // Apply limit
    if let Some(limit) = query.limit {
        rows.truncate(limit);
    }

    // Detect columns: use explicit columns if provided, otherwise auto-detect
    let columns = if !query.columns.is_empty() {
        query.columns.clone()
    } else {
        // Auto-detect all property keys
        let mut keys: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for row in &rows {
            for key in row.properties.keys() {
                if seen.insert(key.clone()) {
                    keys.push(key.clone());
                }
            }
        }
        keys.sort();
        keys
    };

    let query_type_str = match query.query_type {
        QueryType::Table => "table",
        QueryType::List => "list",
        QueryType::Task => "task",
        QueryType::Calendar => "calendar",
    };

    DataviewResult {
        query_type: query_type_str.to_string(),
        rows,
        columns,
        total_count,
        query_time_ms: start.elapsed().as_millis() as u64,
        group_by: query.group_by,
        error: None,
    }
}
