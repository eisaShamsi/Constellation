//! `graph_neighbors` tool — BFS over the typed-link graph starting at
//! one note.
//!
//! Wraps the §B helper `crate::search::constellation_graph_neighbors`.
//! Useful when the user asks "what does this note connect to?" or
//! "what supports / contradicts X?" — the result includes direction
//! ("outgoing" vs "incoming"), link type ("supports", "contradicts",
//! etc.), and distance from the root.
//!
//! Depth is clamped to [1, 3] by the underlying helper.

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::search::constellation_graph_neighbors;

#[derive(Deserialize)]
struct Args {
    /// Absolute file path of the root note.
    path: String,
    /// How many hops to traverse. Clamped to [1, 3]; default 1.
    #[serde(default)]
    depth: Option<u32>,
}

pub fn schema() -> Value {
    json!({
        "name": "graph_neighbors",
        "description": "Find notes connected to a given note through the typed-link graph (supports, contradicts, causes, etc.). Returns each neighbor with its link type, direction (outgoing = root → neighbor, incoming = neighbor → root), and distance. Use when the user asks 'what does X connect to?', 'what supports/contradicts X?', or 'what's related to X by reasoning?' (vs find_similar which uses embeddings).",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute file path of the root note."
                },
                "depth": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 3,
                    "description": "How many hops to traverse (default 1; max 3)."
                }
            },
            "required": ["path"]
        }
    })
}

pub async fn run(app: AppHandle, args: Value) -> Result<Value, String> {
    let parsed: Args =
        serde_json::from_value(args).map_err(|e| format!("invalid args: {e}"))?;

    let depth = parsed.depth.unwrap_or(1);
    let path = parsed.path.clone();

    let neighborhood = tokio::task::spawn_blocking(move || {
        constellation_graph_neighbors(&app, parsed.path, depth)
    })
    .await
    .map_err(|e| format!("spawn_blocking join error: {e}"))??;

    // GraphNeighborhood derives Serialize; flatten directly into the
    // tool result.
    Ok(json!({
        "status": "ok",
        "root_path": neighborhood.root_path,
        "root_name": neighborhood.root_name,
        "depth": neighborhood.depth,
        "neighbor_count": neighborhood.neighbors.len(),
        "neighbors": neighborhood.neighbors,
    }))
}
