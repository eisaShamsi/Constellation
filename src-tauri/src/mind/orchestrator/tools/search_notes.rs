//! `search_notes` tool — keyword search across the active Universe.
//!
//! Wraps `crate::search::constellation_search` with a JSON-arg surface
//! the model can call. Default mode is `lexical` (FTS5 keyword search) —
//! the simplest + always-on path that doesn't require an embedding for
//! the query. Hybrid / semantic modes would need the query passed
//! through `LocalEmbeddingProvider` first; Phase 1 keeps it simple,
//! Phase 1.x may add semantic when the dispatcher gains an embedding
//! handle.

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::search::{constellation_search, SearchFilters, SearchRequest};

#[derive(Deserialize)]
struct Args {
    /// Keyword / phrase to search for.
    query: String,
    /// "lexical" | "structured" | "semantic" | "hybrid". Default: "lexical".
    #[serde(default)]
    mode: Option<String>,
    /// Restrict to these library names. Empty = all libraries.
    #[serde(default)]
    libraries: Option<Vec<String>>,
    /// Filter by tag(s).
    #[serde(default)]
    tags: Option<Vec<String>>,
    /// Max results. Hard-capped at 30 to keep tool-result payloads sane.
    #[serde(default)]
    limit: Option<u32>,
}

pub fn schema() -> Value {
    json!({
        "name": "search_notes",
        "description": "Search the active Universe for notes matching a keyword or phrase. Returns a list of matches with path, name, library, snippet, and modified-time. Use this tool first when answering questions that need information from the user's notes.",
        "input_schema": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The keyword or phrase to search for. Required."
                },
                "mode": {
                    "type": "string",
                    "enum": ["lexical", "structured", "semantic", "hybrid"],
                    "description": "Search mode. Default: lexical (FTS5 keyword)."
                },
                "libraries": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Restrict to these library names. Omit for all libraries."
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Filter by tag(s)."
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 30,
                    "description": "Max results (default 10, hard-cap 30)."
                }
            },
            "required": ["query"]
        }
    })
}

pub async fn run(app: AppHandle, args: Value) -> Result<Value, String> {
    let parsed: Args =
        serde_json::from_value(args).map_err(|e| format!("invalid args: {e}"))?;

    let mode = parsed.mode.unwrap_or_else(|| "lexical".to_string());
    let limit = parsed.limit.unwrap_or(10).min(30);

    let filters = if parsed.libraries.is_some() || parsed.tags.is_some() {
        Some(SearchFilters {
            properties: None,
            tags: parsed.tags,
            wikilinks_to: None,
            wikilinks_from: None,
            mutual: None,
            mentions: None,
            orphans: None,
            links_between: None,
            links_all: None,
            typed_links: None,
            library_names: parsed.libraries,
            maturity: None,
            path_prefix: None,
        })
    } else {
        None
    };

    let request = SearchRequest {
        query: Some(parsed.query),
        query_embedding: None,
        mode,
        filters,
        limit: Some(limit),
        include_snippet: Some(true),
        include_headings: Some(false),
    };

    let results = tokio::task::spawn_blocking(move || constellation_search(app, request))
        .await
        .map_err(|e| format!("spawn_blocking join error: {e}"))??;

    let json_results: Vec<Value> = results
        .into_iter()
        .map(|r| {
            json!({
                "name": r.name,
                "path": r.path,
                "library_name": r.library_name,
                "score": r.score,
                "match_type": r.match_type,
                "snippet": r.snippet,
                "modified": r.modified,
            })
        })
        .collect();

    Ok(json!({
        "status": "ok",
        "result_count": json_results.len(),
        "results": json_results,
    }))
}
