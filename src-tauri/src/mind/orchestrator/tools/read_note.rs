//! `read_note` tool — read the full markdown content of a single note.
//!
//! Wraps `crate::libraries::read_note` which itself validates the path
//! against `validate_path_in_any_library` (a note path outside any
//! registered library is rejected with an error — this is the security
//! boundary that keeps the model from reading arbitrary files on disk).

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::libraries::read_note;

#[derive(Deserialize)]
struct Args {
    /// Absolute file path of the note (.md file).
    path: String,
}

pub fn schema() -> Value {
    json!({
        "name": "read_note",
        "description": "Read the full markdown content of a single note by its absolute file path. Use this after search_notes to get the full text of a result. The path must be inside one of the user's registered libraries — paths outside the Universe are rejected.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute file path of the note (a .md file inside a registered library)."
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
    let content = tokio::task::spawn_blocking(move || read_note(app, parsed.path))
        .await
        .map_err(|e| format!("spawn_blocking join error: {e}"))??;

    Ok(json!({
        "status": "ok",
        "path": path,
        "content": content,
        "length_chars": content.len(),
    }))
}
