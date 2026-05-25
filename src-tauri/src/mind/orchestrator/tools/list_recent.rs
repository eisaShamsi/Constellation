//! `list_recent` tool — return the most-recently-modified notes across
//! the active Universe.
//!
//! Wraps the §B helper `crate::search::constellation_search_recent`.
//! Useful when the user asks "what was I working on yesterday?" or
//! "show me my most recent thoughts on X" (paired with `search_notes`
//! to scope to a topic).

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::search::constellation_search_recent;

#[derive(Deserialize)]
struct Args {
    /// Unix-seconds threshold; only notes modified *after* this are
    /// returned. Omit / pass 0 for "no threshold" (everything sorted).
    #[serde(default)]
    since_unix_seconds: Option<u64>,
    /// Max results. Hard-capped at 100; default 20.
    #[serde(default)]
    limit: Option<u32>,
}

pub fn schema() -> Value {
    json!({
        "name": "list_recent",
        "description": "Return the most-recently-modified notes across the active Universe, newest first. Use when the user asks 'what was I working on lately?' or wants to see their recent activity. Optionally filter to notes modified after a given Unix timestamp.",
        "input_schema": {
            "type": "object",
            "properties": {
                "since_unix_seconds": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Unix timestamp (seconds). Only notes modified after this are returned. Omit for no threshold."
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 100,
                    "description": "Max results (default 20, hard-cap 100)."
                }
            }
        }
    })
}

pub async fn run(app: AppHandle, args: Value) -> Result<Value, String> {
    let parsed: Args =
        serde_json::from_value(args).map_err(|e| format!("invalid args: {e}"))?;

    let since = parsed.since_unix_seconds.unwrap_or(0);
    let limit = parsed.limit;

    let results =
        tokio::task::spawn_blocking(move || constellation_search_recent(&app, since, limit))
            .await
            .map_err(|e| format!("spawn_blocking join error: {e}"))??;

    let json_results: Vec<Value> = results
        .into_iter()
        .map(|r| {
            json!({
                "name": r.name,
                "path": r.path,
                "library_name": r.library_name,
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
