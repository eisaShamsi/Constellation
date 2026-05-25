//! `summarize` tool — note-level summary via NSC.
//!
//! Wraps `crate::nsc::compute_summary_for_note` which returns the
//! author's frontmatter summary (verbatim) if present, otherwise the
//! `> [!summary]` callout (verbatim), otherwise a generated extractive
//! summary. Per Architect §2.3 gap, folder/library-level summarization
//! is OUT of Phase 1 scope — the dispatcher accepts only single-note
//! paths and surfaces an explicit error for other shapes.

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::nsc::compute_summary_for_note;

#[derive(Deserialize)]
struct Args {
    /// Absolute file path of the note to summarize.
    path: String,
}

pub fn schema() -> Value {
    json!({
        "name": "summarize",
        "description": "Get a one-paragraph summary of a single note. Returns the author's frontmatter summary if present, otherwise an extractive summary of the body. Use this when you need a quick gist of a note without reading the full text. Per-note only — folder/library-level summaries are not yet supported.",
        "input_schema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute file path of the note to summarize."
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
    let summary = tokio::task::spawn_blocking(move || {
        compute_summary_for_note(&app, &parsed.path)
    })
    .await
    .map_err(|e| format!("spawn_blocking join error: {e}"))??;

    Ok(json!({
        "status": "ok",
        "path": path,
        "summary": summary.summary,
        "headline": summary.headline,
        "source": summary.source,
    }))
}
