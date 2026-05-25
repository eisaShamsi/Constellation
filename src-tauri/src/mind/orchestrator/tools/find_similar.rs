//! `find_similar` tool — semantic nearest-neighbour search over note
//! embeddings.
//!
//! Wraps `crate::search::constellation_search_similar` which reads the
//! reference note's stored embedding from `note_embeddings` and returns
//! the top-K most-cosine-similar notes (excluding the reference itself).
//!
//! Failure mode: if the reference note has no stored embedding (e.g.,
//! freshly-added note before the embedding pass runs), the underlying
//! fn errors with "Note has no embedding" — surfaced to the model as a
//! tool error.

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::search::constellation_search_similar;

#[derive(Deserialize)]
struct Args {
    /// Absolute file path of the reference note.
    path: String,
    /// Max neighbours. Hard-capped at 30.
    #[serde(default)]
    limit: Option<u32>,
}

pub fn schema() -> Value {
    json!({
        "name": "find_similar",
        "description": "Find notes semantically similar to a given reference note (via embeddings, not keywords). Use this when the user asks 'what notes are related to X' or 'find notes about the same topic as X'. Returns up to 30 nearest-neighbour notes ranked by cosine similarity, excluding the reference itself.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute file path of the reference note."
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 30,
                    "description": "Max neighbours (default 10, hard-cap 30)."
                }
            },
            "required": ["path"]
        }
    })
}

pub async fn run(app: AppHandle, args: Value) -> Result<Value, String> {
    let parsed: Args =
        serde_json::from_value(args).map_err(|e| format!("invalid args: {e}"))?;

    let path = parsed.path.clone();
    let limit = parsed.limit.unwrap_or(10).min(30);

    let results =
        tokio::task::spawn_blocking(move || constellation_search_similar(app, parsed.path, Some(limit)))
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
                "snippet": r.snippet,
            })
        })
        .collect();

    Ok(json!({
        "status": "ok",
        "reference_path": path,
        "result_count": json_results.len(),
        "results": json_results,
    }))
}
