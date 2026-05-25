//! Tool implementations the model can call during a chat turn.
//!
//! Phase 1 (§A) ships the four "ready" read tools that wrap existing
//! subsystem fns without needing any new code outside `mind/`:
//!
//! - [`search_notes`] — keyword search via `search::constellation_search`
//! - [`read_note`] — file body via `libraries::read_note`
//! - [`find_similar`] — semantic neighbours via `search::constellation_search_similar`
//! - [`summarize`] — single-note summary via `nsc::compute_summary_for_note`
//!
//! Phase 1 (§C) will add the remaining two tools after §B lands the
//! supporting `pub fn`s in `search.rs`:
//!
//! - `list_recent` — recently modified notes
//! - `graph_neighbors` — BFS over `note_links`
//!
//! Each tool module follows the same shape:
//! - `pub fn schema() -> serde_json::Value` — the JSON-Schema the model sees
//! - `pub async fn run(app: AppHandle, args: Value) -> Result<Value, String>`
//!
//! Errors bubble up as `Err(String)`; the [`crate::mind::orchestrator::dispatcher::RealToolDispatcher`]
//! catches them and turns them into `{ "status": "error", "error": "..." }`
//! tool results the model can see.

pub mod find_similar;
pub mod read_note;
pub mod search_notes;
pub mod summarize;

use crate::mind::provider::ToolSchema;
use serde_json::Value;

fn schema_value_to_tool_schema(v: Value) -> ToolSchema {
    // Each tool's schema() returns {"name", "description", "input_schema"}
    // matching the ToolSchema struct shape. Parse it.
    serde_json::from_value(v).expect("tool schema must deserialize into ToolSchema")
}

/// The 4 ready tools' schemas as a single palette. §C will extend this
/// to 6 once `list_recent` + `graph_neighbors` are wired.
pub fn ready_palette() -> Vec<ToolSchema> {
    vec![
        schema_value_to_tool_schema(search_notes::schema()),
        schema_value_to_tool_schema(read_note::schema()),
        schema_value_to_tool_schema(find_similar::schema()),
        schema_value_to_tool_schema(summarize::schema()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_palette_has_four_distinct_tools() {
        let palette = ready_palette();
        assert_eq!(palette.len(), 4);
        let names: Vec<&str> = palette.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"search_notes"));
        assert!(names.contains(&"read_note"));
        assert!(names.contains(&"find_similar"));
        assert!(names.contains(&"summarize"));
    }

    #[test]
    fn every_tool_has_nonempty_description_and_required_args() {
        for tool in ready_palette() {
            assert!(!tool.description.is_empty(), "tool {} has empty description", tool.name);
            let schema = &tool.input_schema;
            assert_eq!(schema.get("type"), Some(&serde_json::json!("object")));
            assert!(schema.get("properties").is_some(), "tool {} missing properties", tool.name);
            // Every Phase 1 tool requires at least one arg
            assert!(schema.get("required").is_some(), "tool {} missing required", tool.name);
        }
    }
}
