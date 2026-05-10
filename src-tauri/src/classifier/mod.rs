//! MIG-021 §1B — Epistemic Classifier (Tier 1: e5-small embedding-similarity).
//!
//! Reads a note's content, embeds it via the existing `multilingual-e5-small`
//! ONNX runtime (already shipped for semantic search; reused here at zero
//! additional bundle cost), computes cosine similarity to each of the 11
//! cached source-definition vectors, returns the top-N as suggestions.
//!
//! Tier 2 (Qwen3-1.7B + llama.cpp) is built in §1H and will live in a
//! sibling `tier2_llm.rs` file inside this module. Tier 1 ships first
//! and works on Day 1 with no extra requirements.
//!
//! Anchored against:
//!   docs/Constellation-Sight-Concept-Paper-v2.0.md §8
//!   lab/reports/MIG-021-EPISTEMIC-CLASSIFIER-PLAN.md §1B

mod source_definitions;
mod tier1_embedding;
pub mod scan_job;
// MIG-021v2 §1G2' — Tier 1 deterministic rules + correction log.
pub mod tier1_rules;
pub mod correction_log;

use crate::sources::{write_suggestions, SuggestionRecord};
use std::path::Path;
use tauri::Manager;

/// On-demand single-note classification.
///
/// Reads the note from disk, runs Tier 1 classification, writes the top-3
/// suggestions to the `sources_suggestions` queue, returns the
/// suggestion record for the frontend to surface immediately.
///
/// Returns an error if the note can't be read, the embedding engine
/// fails to initialize, or the database write fails.
#[tauri::command]
pub fn classifier_suggest_for_note(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<SuggestionRecord, String> {
    crate::search::ensure_search_db_ready(&app)?;

    // 1. Read note content.
    let path = Path::new(&note_path);
    if !path.exists() {
        return Err(format!("Note not found: {}", note_path));
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", note_path, e))?;

    // 2. Extract title + body for classification (per Plan §0 Q3:
    // title carries strong signal; classify on title + body concatenated).
    let (title, body) = extract_title_and_body(&content);
    let text_for_classification = if title.is_empty() {
        body
    } else {
        format!("{}\n\n{}", title, body)
    };

    // 3. MIG-021v2 §1G2' — Tier 1 deterministic rules first.
    //    Read frontmatter sources / content_type for precedence check.
    let frontmatter_sources = crate::sources::extract_sources(&content);
    let frontmatter_content_type = crate::sources::extract_content_type(&content);
    let tier1 = tier1_rules::classify_tier1(
        &content,
        &frontmatter_sources,
        &frontmatter_content_type,
    );

    // Combine Tier 1 hits across both axes.
    let mut suggestions: Vec<crate::sources::Suggestion> = Vec::new();
    suggestions.extend(tier1.horizontal.iter().cloned());
    suggestions.extend(tier1.vertical.iter().cloned());

    // 4. Fall through to Tier 2 (embeddings) for any axis Tier 1 left empty.
    //    This is the heart of the three-tier routing — if rules covered
    //    one axis but not the other, we only spend embedding cost on the
    //    uncovered axis. Today tier1_embedding::classify always runs for
    //    both; we filter its output to fill the gap so the cheaper path
    //    is preserved as much as possible.
    let needs_horizontal = tier1.horizontal.is_empty();
    let needs_vertical = tier1.vertical.is_empty();
    let tier_used: i64 = if tier1.from_frontmatter {
        1
    } else if !needs_horizontal && !needs_vertical {
        1
    } else if !needs_horizontal || !needs_vertical {
        // Mixed — at least one axis needed Tier 2. We tag the record at
        // tier 2 so the badge reflects "model" rather than "rule" when
        // the user looks at it; the per-suggestion provenance comes
        // from the evidence string.
        2
    } else {
        2
    };

    if needs_horizontal || needs_vertical {
        let tier2 = tier1_embedding::classify(&app, &text_for_classification)?;
        for s in tier2 {
            if s.axis == "horizontal" && needs_horizontal {
                suggestions.push(s);
            } else if s.axis == "vertical" && needs_vertical {
                suggestions.push(s);
            }
        }
    }

    // 5. Write suggestions to queue (overwrites any prior entry).
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state
        .db
        .lock()
        .map_err(|e| format!("DB lock: {}", e))?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;
    write_suggestions(conn, &note_path, &suggestions, tier_used)?;

    // 6. Return the record for immediate display.
    Ok(SuggestionRecord {
        note_path,
        suggestions,
        classifier_tier: tier_used,
        created_at: chrono::Utc::now().timestamp(),
    })
}

/// Internal: split a note into (title, body) where title is the
/// frontmatter `title:` field (or the file stem), and body is the
/// frontmatter-stripped content.
///
/// Body is truncated to the first 2000 chars per Plan §0 Q4 (Tier 1
/// uses ~512-token e5-small window; 2000 chars is a safe upper bound
/// covering most knowledge-note lengths).
fn extract_title_and_body(content: &str) -> (String, String) {
    let mut title = String::new();
    let mut body = content.to_string();

    if content.starts_with("---") {
        if let Some(end) = content[3..].find("\n---") {
            let frontmatter = &content[3..3 + end];
            for line in frontmatter.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("title:") {
                    title = rest.trim().trim_matches('"').trim_matches('\'').to_string();
                    break;
                }
            }
            let body_start = 3 + end + 4;
            body = content[body_start..].trim().to_string();
        }
    }

    // Truncate body to 2000 chars (char-boundary safe for UTF-8).
    if body.len() > 2000 {
        let mut end = 2000;
        while end > 0 && !body.is_char_boundary(end) {
            end -= 1;
        }
        body.truncate(end);
    }

    (title, body)
}

// Re-exports kept private for now; §1H Tier-2 wrapper will surface
// what it needs when it lands. Tests inside this module can reach
// the children directly via super::source_definitions / super::tier1_embedding.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_title_and_body_handles_no_frontmatter() {
        let (t, b) = extract_title_and_body("just body text");
        assert_eq!(t, "");
        assert_eq!(b, "just body text");
    }

    #[test]
    fn extract_title_and_body_pulls_title_from_frontmatter() {
        let content = "---\ntitle: Foo\n---\n\nbody here";
        let (t, b) = extract_title_and_body(content);
        assert_eq!(t, "Foo");
        assert_eq!(b, "body here");
    }

    #[test]
    fn extract_title_and_body_truncates_long_body() {
        let long_body = "a".repeat(3000);
        let content = format!("---\ntitle: T\n---\n\n{}", long_body);
        let (_, b) = extract_title_and_body(&content);
        assert!(b.len() <= 2000);
    }
}
