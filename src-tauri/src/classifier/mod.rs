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

// MIG-022 §0 (audit F1, 2026-05-11) — `source_definitions`,
// `tier1_embedding`, and `tier1_rules` deleted as dead code. The v2
// three-tier classifier was replaced wholesale by the V3-§8 CECE
// 6-cataloger ensemble; the modules had no production callers (only
// their own internal tests). `scan_job` and `correction_log` remain
// reachable: scan_job hosts the active classifier_scan_* IPCs; the
// correction log is consumed by sources/ and cece/reliability.rs.
pub mod scan_job;
pub mod correction_log;

use crate::sources::{write_suggestions, SuggestionRecord};
use std::path::Path;
use tauri::Manager;

/// On-demand single-note classification.
///
/// Reads the note from disk, runs the CECE 6-cataloger ensemble,
/// writes the synthesized suggestions to the `sources_suggestions`
/// queue, returns the suggestion record for the frontend to surface
/// immediately.
///
/// Returns an error if the note can't be read, the embedding engine
/// fails to initialize, or the database write fails.
// Note-open-freeze Batch-2 §B2-2 (2026-07-03): `(async)` — off the IPC dispatch thread.
// Discovery-verified async-only-safe: DB-only / mutex-covered body, no note-file writes,
// all callers await. See SESSION-LOG-2026-07-03 (Architect findings).
#[tauri::command(async)]
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
    //    MIG-022 §0 (audit F1, 2026-05-11): the v2 three-tier
    //    classifier (tier1_rules + tier1_embedding + source_definitions)
    //    has been deleted; CECE replaced it wholesale. Each cataloger
    //    implements its own logic from scratch (Linguistic uses CAE
    //    roots directly, Structural uses regex rules directly, Semantic
    //    uses the embedding helper directly).
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

/// V3-§9.C.2 — Update per-cataloger reliability for both axes from
/// a single composite_json snapshot.
///
/// Boss-test 2026-05-11 Stage 3 surfaced a silent gap in V3-§9.C
/// wiring: when the frontend's Accept handler called sources_set_manual
/// + content_type_set_manual back-to-back, the second IPC found the
/// suggestion row already cleared by the first IPC's clear_suggestions
/// call, so its prior_composite was None and reliability never updated
/// on the second axis.
///
/// Fix: move the reliability update OUT of the per-axis IPCs and
/// into this dedicated dual-axis IPC. Callers with a composite snapshot
/// (the Source Review Accept handler; cece_resolve_disambiguation)
/// invoke this AFTER their writes. Single-pass updates both axes from
/// one snapshot — the suggestion row's status doesn't matter here
/// because the caller passes the JSON in directly.
///
/// `horizontal_pick` and `vertical_pick` are the user's final IDs on
/// each axis. Either may be empty (no update on that axis).
// App-freeze audit Batch-W (2026-07-04): `(async)` — the reliability RMW is
// file IO off the dispatch thread; RELIABILITY_LOCK (reliability.rs)
// serializes concurrent corrections.
#[tauri::command(async)]
pub fn cece_record_correction_for_card(
    app: tauri::AppHandle,
    note_path: String,
    composite_json: String,
    horizontal_pick: Vec<String>,
    vertical_pick: Vec<String>,
) -> Result<(), String> {
    crate::search::ensure_search_db_ready(&app)?;
    // Resolve the Library root the same way the in-IPC reliability
    // update used to. Returns None when the note isn't under any
    // registered Library — in that case there's nowhere to write
    // reliability data, so it's a no-op.
    let lib_root = match library_root_for_note(&app, &note_path) {
        Some(r) => r,
        None => return Ok(()),
    };
    if !horizontal_pick.is_empty() {
        crate::cece::reliability::update_reliability_from_correction(
            &lib_root,
            &composite_json,
            "horizontal",
            &horizontal_pick,
        );
    }
    if !vertical_pick.is_empty() {
        crate::cece::reliability::update_reliability_from_correction(
            &lib_root,
            &composite_json,
            "vertical",
            &vertical_pick,
        );
    }
    Ok(())
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
///
/// V3-§8.r7 Issue #2 (Boss-test 2026-05-10) — also auto-write the
/// OTHER axis's settled primary when that axis is not Split. Rationale:
/// the previous behavior was per-axis surgical, which lost data when
/// only one axis was Split. Example: Auteur theory had horizontal=Split
/// and vertical=Unanimous on `epistemic-states/illusion`; picking the
/// SOURCE chip wrote only `sources:` and silently discarded the
/// settled `epistemic-states/illusion` vertical value. Now we read
/// the composite_json before writing, identify the other axis's
/// primary if the other axis is not Split, and write both axes
/// in sequence.
///
/// MIG-040 bugfix (Eisa, 2026-05-20): when a card is Split on BOTH axes,
/// picking one axis used to clear the whole suggestion (card vanished) and
/// left the other axis unclassified. Now, if the OTHER axis is still Split,
/// the suggestion is re-inserted with the resolved axis marked settled, so
/// the card stays in place showing only the remaining axis. The command
/// returns the updated record (Some) when the card should stay, or None when
/// both axes are decided and the card should be removed.
// Note-open-freeze Batch-2 §B2-3 (2026-07-03): `(async)` — off the IPC dispatch thread.
// The note-file frontmatter rewrite inside runs through gate_rmw (whole read→rewrite→write
// under the per-path lock), so it can no longer interleave with a debounced editor save.
#[tauri::command(async)]
pub fn cece_resolve_disambiguation(
    app: tauri::AppHandle,
    note_path: String,
    axis: String,
    chosen_id: String,
) -> Result<Option<crate::sources::SuggestionRecord>, String> {
    // Snapshot the FULL prior suggestion BEFORE any write — set_manual clears
    // the row. We need composite_json (to detect whether the OTHER axis is
    // still Split + to re-insert) and suggestions / tier / created_at (to
    // re-insert preserving queue position). Best-effort: a missing row leaves
    // the snapshot None and the branches below no-op.
    let prior: Option<crate::sources::SuggestionRecord> = {
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state
            .db
            .lock()
            .map_err(|e| format!("DB lock: {}", e))?;
        let conn = db_guard
            .as_ref()
            .ok_or("Search database not initialized")?;
        crate::sources::read_suggestions(conn, &note_path).ok().flatten()
    };
    let composite_snapshot: Option<String> = prior.as_ref().and_then(|r| r.composite_json.clone());

    // Is the axis OTHER than the one being resolved ALSO Split (still awaiting
    // the user's pick)? If so, resolving THIS axis must NOT discard the card.
    let other_axis_still_split = composite_snapshot
        .as_deref()
        .map(|json| other_axis_needs_disambiguation(json, &axis))
        .unwrap_or(false);
    // The other axis's settled primary (None when the other axis is Split).
    let other_axis_settled: Option<String> = composite_snapshot
        .as_deref()
        .and_then(|json| extract_other_axis_settled(json, &axis));

    // Apply the user's pick on the disambiguated axis. This writes frontmatter
    // + the note_meta mirror + the correction log, AND clears the suggestion row.
    let chosen_id_for_reliability = chosen_id.clone();
    match axis.as_str() {
        "horizontal" => {
            crate::sources::sources_set_manual(app.clone(), note_path.clone(), vec![chosen_id.clone()])?;
        }
        "vertical" => {
            crate::sources::content_type_set_manual(app.clone(), note_path.clone(), vec![chosen_id.clone()])?;
        }
        other => return Err(format!("Unknown axis: {}", other)),
    }

    if other_axis_still_split {
        // Re-insert the suggestion with the resolved axis marked settled, so
        // the card stays in the queue showing ONLY the remaining (still-Split)
        // axis's chips. created_at is preserved so the card keeps its place.
        if let (Some(prior_rec), Some(snap)) = (prior.as_ref(), composite_snapshot.as_deref()) {
            let mutated = mark_axis_resolved(snap, &axis, &chosen_id);
            let suggestions_json = serde_json::to_string(&prior_rec.suggestions)
                .map_err(|e| format!("re-serialize suggestions: {}", e))?;
            let search_state = app.state::<crate::search::SearchState>();
            let db_guard = search_state.db.lock().map_err(|e| format!("DB lock: {}", e))?;
            let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
            conn.execute(
                "INSERT OR REPLACE INTO sources_suggestions
                 (note_path, suggestions_json, classifier_tier, created_at, composite_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![note_path, suggestions_json, prior_rec.classifier_tier, prior_rec.created_at, mutated],
            )
            .map_err(|e| format!("re-insert suggestion: {}", e))?;
        }

        // Reliability: record only the axis the user just resolved; the other
        // axis's correction is recorded when the user picks it.
        if let Some(blob) = composite_snapshot.clone() {
            let (h_pick, v_pick) = match axis.as_str() {
                "horizontal" => (vec![chosen_id_for_reliability.clone()], Vec::<String>::new()),
                "vertical" => (Vec::<String>::new(), vec![chosen_id_for_reliability.clone()]),
                _ => (Vec::<String>::new(), Vec::<String>::new()),
            };
            let _ = cece_record_correction_for_card(app.clone(), note_path.clone(), blob, h_pick, v_pick);
        }

        // Return the updated record so the frontend keeps + refreshes the card.
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state.db.lock().map_err(|e| format!("DB lock: {}", e))?;
        let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
        return Ok(crate::sources::read_suggestions(conn, &note_path).ok().flatten());
    }

    // ── Other axis already settled (or absent): original r7 behavior. ──
    // Co-write the other axis's settled primary, if it had one.
    let other_id_for_reliability = other_axis_settled.clone();
    if let Some(other_id) = other_axis_settled {
        match axis.as_str() {
            "horizontal" => {
                let _ = crate::sources::content_type_set_manual(app.clone(), note_path.clone(), vec![other_id]);
            }
            "vertical" => {
                let _ = crate::sources::sources_set_manual(app.clone(), note_path.clone(), vec![other_id]);
            }
            _ => {}
        }
    }

    // V3-§9.C.2 — reliability for BOTH axes from the pre-write snapshot.
    if let Some(blob) = composite_snapshot {
        let (h_pick, v_pick) = match axis.as_str() {
            "horizontal" => (
                vec![chosen_id_for_reliability],
                other_id_for_reliability.into_iter().collect::<Vec<_>>(),
            ),
            "vertical" => (
                other_id_for_reliability.into_iter().collect::<Vec<_>>(),
                vec![chosen_id_for_reliability],
            ),
            _ => (vec![], vec![]),
        };
        let _ = cece_record_correction_for_card(app, note_path, blob, h_pick, v_pick);
    }

    Ok(None)
}

/// Does the axis OTHER than `picked_axis` still need disambiguation (i.e. is
/// it also Split)? Used so resolving one axis of a both-Split card doesn't
/// discard the card before the user resolves the other axis.
fn other_axis_needs_disambiguation(composite_json: &str, picked_axis: &str) -> bool {
    let value: serde_json::Value = match serde_json::from_str(composite_json) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let other_key = match picked_axis {
        "horizontal" => "vertical",
        "vertical" => "horizontal",
        _ => return false,
    };
    value
        .get(other_key)
        .and_then(|o| o.get("needs_user_disambiguation_between"))
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
}

/// Return a copy of `composite_json` with `axis` marked resolved: its
/// `needs_user_disambiguation_between` cleared, `regime` settled, and
/// `primary` set to the user's pick. The other axis is untouched. On parse
/// failure, returns the input unchanged.
fn mark_axis_resolved(composite_json: &str, axis: &str, chosen_id: &str) -> String {
    let mut value: serde_json::Value = match serde_json::from_str(composite_json) {
        Ok(v) => v,
        Err(_) => return composite_json.to_string(),
    };
    let key = match axis {
        "horizontal" | "vertical" => axis,
        _ => return composite_json.to_string(),
    };
    if let Some(obj) = value.get_mut(key).and_then(|a| a.as_object_mut()) {
        obj.insert("needs_user_disambiguation_between".to_string(), serde_json::Value::Null);
        obj.insert("regime".to_string(), serde_json::Value::String("strong_majority".to_string()));
        obj.insert("primary".to_string(), serde_json::Value::String(chosen_id.to_string()));
    }
    serde_json::to_string(&value).unwrap_or_else(|_| composite_json.to_string())
}

/// V3-§8.r7 — Parse the composite JSON to extract the OTHER axis's
/// primary, but ONLY if that other axis is settled (not Split).
/// Used by `cece_resolve_disambiguation` to auto-write data the user
/// didn't have to pick.
///
/// Defensive: returns None on any parse error, missing field, or if
/// the other axis is Split (in which case the user has separate
/// chips for that axis and shouldn't have it auto-written).
fn extract_other_axis_settled(composite_json: &str, picked_axis: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(composite_json).ok()?;
    let other_key = match picked_axis {
        "horizontal" => "vertical",
        "vertical" => "horizontal",
        _ => return None,
    };
    let other = value.get(other_key)?;
    // Only auto-write if the other axis is NOT Split. Use the same
    // semantic check as the frontend's cardNeedsUserCall: a Split
    // axis is one with a populated needs_user_disambiguation_between
    // array.
    let needs_disambig = other
        .get("needs_user_disambiguation_between")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    if needs_disambig {
        return None;
    }
    let primary = other.get("primary")?.as_str()?;
    if primary.is_empty() {
        return None;
    }
    Some(primary.to_string())
}

#[cfg(test)]
mod r7_tests {
    use super::{extract_other_axis_settled, mark_axis_resolved, other_axis_needs_disambiguation};

    #[test]
    fn extracts_vertical_primary_when_horizontal_picked() {
        let json = r#"{
            "horizontal":{"primary":"testimony/authoritative","regime":"split","needs_user_disambiguation_between":["a","b"]},
            "vertical":{"primary":"epistemic-states/illusion","regime":"unanimous","needs_user_disambiguation_between":null}
        }"#;
        assert_eq!(
            extract_other_axis_settled(json, "horizontal"),
            Some("epistemic-states/illusion".to_string())
        );
    }

    #[test]
    fn returns_none_when_other_axis_is_also_split() {
        let json = r#"{
            "horizontal":{"primary":"a","regime":"split","needs_user_disambiguation_between":["a","b"]},
            "vertical":{"primary":"x","regime":"split","needs_user_disambiguation_between":["x","y"]}
        }"#;
        assert!(extract_other_axis_settled(json, "horizontal").is_none());
    }

    // MIG-040 — both-axes-split disambiguation bugfix helpers.
    #[test]
    fn other_axis_needs_disambiguation_detects_both_split() {
        let both = r#"{
            "horizontal":{"primary":"a","regime":"split","needs_user_disambiguation_between":["a","b"]},
            "vertical":{"primary":"x","regime":"split","needs_user_disambiguation_between":["x","y"]}
        }"#;
        // Picking either axis leaves the OTHER one still Split.
        assert!(other_axis_needs_disambiguation(both, "horizontal"));
        assert!(other_axis_needs_disambiguation(both, "vertical"));

        let only_h = r#"{
            "horizontal":{"primary":"a","regime":"split","needs_user_disambiguation_between":["a","b"]},
            "vertical":{"primary":"x","regime":"unanimous","needs_user_disambiguation_between":null}
        }"#;
        // Picking horizontal -> vertical is settled, not Split.
        assert!(!other_axis_needs_disambiguation(only_h, "horizontal"));
    }

    #[test]
    fn mark_axis_resolved_settles_only_the_picked_axis() {
        let both = r#"{
            "horizontal":{"primary":"a","regime":"split","needs_user_disambiguation_between":["a","b"]},
            "vertical":{"primary":"x","regime":"split","needs_user_disambiguation_between":["x","y"]}
        }"#;
        let out = mark_axis_resolved(both, "horizontal", "a");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        // Horizontal is now settled on the user's pick.
        assert!(v["horizontal"]["needs_user_disambiguation_between"].is_null());
        assert_eq!(v["horizontal"]["primary"], "a");
        assert_ne!(v["horizontal"]["regime"], "split");
        // Vertical is untouched — still Split, so the card keeps its chips.
        assert_eq!(
            v["vertical"]["needs_user_disambiguation_between"],
            serde_json::json!(["x", "y"])
        );
    }

    #[test]
    fn extracts_horizontal_primary_when_vertical_picked() {
        let json = r#"{
            "horizontal":{"primary":"testimony/authoritative","regime":"unanimous","needs_user_disambiguation_between":null},
            "vertical":{"primary":"x","regime":"split","needs_user_disambiguation_between":["x","y"]}
        }"#;
        assert_eq!(
            extract_other_axis_settled(json, "vertical"),
            Some("testimony/authoritative".to_string())
        );
    }

    #[test]
    fn returns_none_on_malformed_json() {
        assert!(extract_other_axis_settled("not json", "horizontal").is_none());
        assert!(extract_other_axis_settled("{}", "horizontal").is_none());
        assert!(extract_other_axis_settled(r#"{"horizontal":{}}"#, "vertical").is_none());
    }

    #[test]
    fn returns_none_when_other_axis_primary_is_null() {
        let json = r#"{
            "horizontal":{"primary":null,"regime":"unanimous","needs_user_disambiguation_between":null},
            "vertical":{"primary":"x","regime":"split","needs_user_disambiguation_between":["x"]}
        }"#;
        assert!(extract_other_axis_settled(json, "vertical").is_none());
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
