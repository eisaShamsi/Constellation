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

    // 3. MIG-021v3 V3-§8 — CECE Cataloger Ensemble.
    //    Build the production-wired orchestrator (six catalogers,
    //    cost-ordered, with embed/lookup/inference functions wired
    //    to the real backends). Run the ensemble against this note.
    //
    //    The v2 three-tier classifier (tier1_rules + tier1_embedding)
    //    is preserved — its outputs are now consumed by the Linguistic,
    //    Structural, and Semantic catalogers via the wiring layer.
    //    See lab/reports/MIG-021v3-EPISTEMIC-CONTENT-ENGINE-ARCHITECT.md §6.
    let _ = text_for_classification; // suppress unused-warning; future:
                                      // pass title-prepended text into ctx if a
                                      // cataloger asks for it via context.
    let frontmatter_sources = crate::sources::extract_sources(&content);
    let frontmatter_content_type = crate::sources::extract_content_type(&content);
    let ctx = crate::cece::cataloger::CatalogerContext::new(
        note_path.clone(),
        content.clone(),
        frontmatter_sources,
        frontmatter_content_type,
    );

    let orchestrator = crate::cece::wiring::build_orchestrator(&app);

    // Two-pass run: cheap catalogers first, expensive only if cheaper
    // catalogers don't reach Unanimous. The closure synthesizes after
    // the cheap pass so the orchestrator can decide whether to spend
    // the Reasoning Cataloger budget.
    let reliability_for_early =
        crate::cece::reliability::ReliabilityProfile::default();
    let trails = orchestrator.run(&ctx, |trails_so_far| {
        let composite = crate::cece::synthesis::synthesize(
            trails_so_far.to_vec(),
            &reliability_for_early,
        );
        crate::cece::orchestrator::EarlyVerdict::from_decisions(
            &composite.horizontal,
            &composite.vertical,
        )
    });

    // Final synthesis with the per-Library reliability profile (looked
    // up by the note's containing Library; falls back to default when
    // the Library can't be resolved).
    let library_root = library_root_for_note(&app, &note_path);
    let reliability = library_root
        .as_ref()
        .map(|lr| crate::cece::reliability::load_or_default(lr))
        .unwrap_or_default();
    let composite =
        crate::cece::synthesis::synthesize(trails, &reliability);

    // 4. Build flat suggestions from the composite + persist with the
    //    composite trail in the suggestions_json blob (backward-
    //    compatible — old readers still parse the flat suggestions list).
    // V3-§8 fix-A: propagate the REAL ensemble weights from the
    // synthesis layer instead of hardcoded 0.85/0.50 constants.
    // primary_weight is normalized [0, 1] where 1.0 = winning weighted
    // vote; see_also_weights are normalized fractions of the primary.
    let mut suggestions: Vec<crate::sources::Suggestion> = Vec::new();
    if let Some(prim) = &composite.horizontal.primary {
        suggestions.push(crate::sources::Suggestion {
            source: prim.clone(),
            confidence: composite.horizontal.primary_weight,
            evidence: composite.composite_reasoning.clone(),
            axis: "horizontal".to_string(),
        });
        for (i, s) in composite.horizontal.see_also.iter().enumerate() {
            let w = composite
                .horizontal
                .see_also_weights
                .get(i)
                .copied()
                .unwrap_or(0.0);
            suggestions.push(crate::sources::Suggestion {
                source: s.clone(),
                confidence: w,
                evidence: "see also (competing cataloger vote)".to_string(),
                axis: "horizontal".to_string(),
            });
        }
    }
    if let Some(prim) = &composite.vertical.primary {
        suggestions.push(crate::sources::Suggestion {
            source: prim.clone(),
            confidence: composite.vertical.primary_weight,
            evidence: composite.composite_reasoning.clone(),
            axis: "vertical".to_string(),
        });
        for (i, s) in composite.vertical.see_also.iter().enumerate() {
            let w = composite
                .vertical
                .see_also_weights
                .get(i)
                .copied()
                .unwrap_or(0.0);
            suggestions.push(crate::sources::Suggestion {
                source: s.clone(),
                confidence: w,
                evidence: "see also (competing cataloger vote)".to_string(),
                axis: "vertical".to_string(),
            });
        }
    }

    // tier_used semantics carried over from v2 for backward compat:
    // 1 = only cheap catalogers contributed; 2 = expensive (Reasoning)
    // also voiced. Since Reasoning Cataloger abstains today (V3-§7
    // injection deferred), tier_used will always be 1 until V3-§7.b.
    let tier_used: i64 = if composite
        .catalogers_voiced
        .iter()
        .any(|c| c == "reasoning")
    {
        2
    } else {
        1
    };

    // 5. Persist: standard Suggestion list AND the composite trail
    //    blob. The composite is stored next to the suggestions in the
    //    same row so the Source Review UI can render both.
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state
        .db
        .lock()
        .map_err(|e| format!("DB lock: {}", e))?;
    let conn = db_guard
        .as_ref()
        .ok_or("Search database not initialized")?;
    write_suggestions_with_composite(conn, &note_path, &suggestions, tier_used, &composite)?;

    // 6. Return the record for immediate display.
    let composite_json = serde_json::to_string(&composite).ok();
    Ok(SuggestionRecord {
        note_path,
        suggestions,
        classifier_tier: tier_used,
        created_at: chrono::Utc::now().timestamp(),
        composite_json,
    })
}

/// Library-root resolution for the per-Library reliability JSON
/// lookup. Returns the longest-prefix-matching Library root path, or
/// None if the note isn't under any registered Library.
fn library_root_for_note(app: &tauri::AppHandle, note_path: &str) -> Option<String> {
    let libs = crate::libraries::list_libraries(app.clone());
    let pairs: Vec<(String, String)> = libs.into_iter().map(|l| (l.id, l.path)).collect();
    crate::classifier::correction_log::library_root_for_note(&pairs, note_path)
}

/// V3-§8.r1.f — Resolve a Sibling Disambiguation pick.
///
/// When the synthesis layer reports `regime: split` on either axis,
/// the Source Review UI surfaces the candidate IDs as radio chips.
/// The user picks one; the IPC writes that choice to the note's
/// frontmatter (via the existing `sources_set_manual` /
/// `content_type_set_manual` pipelines), logs the correction, and
/// clears the suggestion row.
///
/// `axis` is "horizontal" or "vertical"; `chosen_id` must be a valid
/// taxonomy ID for that axis (validated downstream by sources::*).
#[tauri::command]
pub fn cece_resolve_disambiguation(
    app: tauri::AppHandle,
    note_path: String,
    axis: String,
    chosen_id: String,
) -> Result<(), String> {
    match axis.as_str() {
        "horizontal" => {
            crate::sources::sources_set_manual(app, note_path, vec![chosen_id])
        }
        "vertical" => {
            crate::sources::content_type_set_manual(app, note_path, vec![chosen_id])
        }
        other => Err(format!("Unknown axis: {}", other)),
    }
}

/// V3-§8 extension of write_suggestions: persists the standard
/// Suggestion list AND the CECE composite reasoning trail so the
/// Source Review UI can render the per-cataloger badge cluster +
/// reasoning trail + Sibling Disambiguation prompts.
///
/// Storage shape: the composite is serialized as JSON and stored in
/// the `composite_json` column on `sources_suggestions`. v2-era rows
/// have NULL there; the SourceReview UI handles both cases.
///
/// V3-§8.r4.3 (audit P1.5): the column-add ALTER moved from this
/// hot path into `sources::ensure_sources_suggestions_table`
/// (called from `search::init_db` at boot). This IPC now assumes
/// the column exists. If it doesn't, the INSERT will fail loudly
/// with a real error — which is what we want; silent swallowing
/// would mask actual schema corruption.
fn write_suggestions_with_composite(
    conn: &rusqlite::Connection,
    note_path: &str,
    suggestions: &[crate::sources::Suggestion],
    tier_used: i64,
    composite: &crate::cece::synthesis::CompositeAssignment,
) -> Result<(), String> {
    let suggestions_json = serde_json::to_string(suggestions)
        .map_err(|e| format!("serialize suggestions: {}", e))?;
    let composite_json = serde_json::to_string(composite)
        .map_err(|e| format!("serialize composite: {}", e))?;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        r#"
        INSERT INTO sources_suggestions (note_path, suggestions_json, classifier_tier, created_at, composite_json)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(note_path) DO UPDATE SET
            suggestions_json = excluded.suggestions_json,
            classifier_tier = excluded.classifier_tier,
            created_at = excluded.created_at,
            composite_json = excluded.composite_json
        "#,
        rusqlite::params![note_path, suggestions_json, tier_used, now, composite_json],
    )
    .map_err(|e| format!("insert suggestion: {}", e))?;
    Ok(())
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
