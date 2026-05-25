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
    let full_content = tokio::task::spawn_blocking(move || read_note(app, parsed.path))
        .await
        .map_err(|e| format!("spawn_blocking join error: {e}"))??;

    // Phase 1 §N round-2 crash fix: cap returned content to ~2000
    // chars (~500-700 tokens) so the tool result doesn't blow the
    // round-2 prompt envelope past Fanar's 8192-token context.
    // Truncation is char-aware (no mid-UTF-8 splits).
    const MAX_CHARS: usize = 2000;
    let (content, truncated) = if full_content.chars().count() > MAX_CHARS {
        let trimmed: String = full_content.chars().take(MAX_CHARS).collect();
        (trimmed + "\n\n…[content truncated]", true)
    } else {
        (full_content.clone(), false)
    };

    Ok(json!({
        "status": "ok",
        "path": path,
        "content": content,
        "length_chars": full_content.len(),
        "truncated": truncated,
    }))
}
