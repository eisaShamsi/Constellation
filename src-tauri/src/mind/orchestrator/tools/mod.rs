//! Tool implementations the model can call during a chat turn.
//!
//! Phase 1 ships the full read-tool palette — six tools:
//!
//! - [`search_notes`] — keyword search via `search::constellation_search`
//! - [`read_note`] — file body via `libraries::read_note`
//! - [`find_similar`] — semantic neighbours via `search::constellation_search_similar`
//! - [`summarize`] — single-note summary via `nsc::compute_summary_for_note`
//! - [`list_recent`] — newest-first scan via `search::constellation_search_recent`
//! - [`graph_neighbors`] — typed-link BFS via `search::constellation_graph_neighbors`
//!
//! The first 4 landed in §A (their backing fns existed already); the
//! last 2 landed in §C after §B added `constellation_search_recent` +
//! `constellation_graph_neighbors` to `search.rs`.
//!
//! Each tool module follows the same shape:
//! - `pub fn schema() -> serde_json::Value` — the JSON-Schema the model sees
//! - `pub async fn run(app: AppHandle, args: Value) -> Result<Value, String>`
//!
//! Errors bubble up as `Err(String)`; the
//! [`crate::mind::orchestrator::dispatcher::RealToolDispatcher`] catches
//! them and turns them into `{ "status": "error", "error": "..." }`
//! tool results the model can see and recover from.
//!
//! NOTE: not every tool requires args (`list_recent` accepts zero). The
//! `required` field in the input_schema is therefore optional; the
//! palette test below checks presence-when-needed rather than always.

pub mod find_similar;
pub mod graph_neighbors;
pub mod list_recent;
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

/// The full Phase 1 read-tool palette — all 6 tools `RealToolDispatcher`
/// routes to.
pub fn ready_palette() -> Vec<ToolSchema> {
    vec![
        schema_value_to_tool_schema(search_notes::schema()),
        schema_value_to_tool_schema(read_note::schema()),
        schema_value_to_tool_schema(find_similar::schema()),
        schema_value_to_tool_schema(summarize::schema()),
        schema_value_to_tool_schema(list_recent::schema()),
        schema_value_to_tool_schema(graph_neighbors::schema()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_palette_has_six_distinct_tools() {
        let palette = ready_palette();
        assert_eq!(palette.len(), 6);
        let names: Vec<&str> = palette.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"search_notes"));
        assert!(names.contains(&"read_note"));
        assert!(names.contains(&"find_similar"));
        assert!(names.contains(&"summarize"));
        assert!(names.contains(&"list_recent"));
        assert!(names.contains(&"graph_neighbors"));
    }

    #[test]
    fn every_tool_has_nonempty_description_and_object_input_schema() {
        for tool in ready_palette() {
            assert!(!tool.description.is_empty(), "tool {} has empty description", tool.name);
            let schema = &tool.input_schema;
            assert_eq!(schema.get("type"), Some(&serde_json::json!("object")));
            assert!(schema.get("properties").is_some(), "tool {} missing properties", tool.name);
            // `required` is OPTIONAL — list_recent legitimately has no required args.
        }
    }
}
