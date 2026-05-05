//! CTSE concept-based semantic search (MIG-013 §1D).
//!
//! Closes the read path: a user query is embedded once, mapped to its
//! nearest M11 concepts, expanded back to every term in `term_vocab`
//! that resolves to those concepts, and finally surfaced as note hits
//! via the existing `notes_fts` MATCH pipeline.
//!
//! ## Why this gives cross-language search "for free"
//!
//! Every M11 concept ships with its multilingual lemma set (English,
//! Arabic, Spanish, ...). At first-fill, Arabic terms like `معرفة`
//! and English terms like `knowledge` resolve to the same concept id.
//! When the user types "knowledge", the e5 embedding lands close to
//! that concept. The OR-clause built from term_vocab includes both
//! lemmas, so the FTS5 MATCH surfaces notes regardless of which
//! script they're written in.
//!
//! ## Cost profile
//!
//! - One e5 inference (~50 ms) per query.
//! - One cosine sweep over 20K × 384 floats (~5 ms).
//! - One `term_vocab` lookup (indexed, sub-ms).
//! - One FTS5 MATCH with an OR clause (linear in matched rows; bounded
//!   by `LIMIT`).
//!
//! Total: typical query lands in ~60–80 ms end-to-end. Frontend MUST
//! debounce ≥300 ms (CLAUDE.md Rule 3) to keep the IPC channel cool.

use rusqlite::{params, params_from_iter};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::bridge_vectors;
use crate::embeddings;
use crate::search::SearchState;

/// How many nearest concepts to consider per query. Each concept may
/// have ~5–20 terms in `term_vocab` mapping to it (multi-script
/// coverage), so the OR-clause cardinality is roughly K × 10.
const CONCEPT_TOP_K: usize = 10;

/// Minimum cosine score for a concept to be included. Below this, the
/// concept is "too far" from the query and would dilute results with
/// noise. Tunable via the request payload (`min_score`).
const DEFAULT_MIN_SCORE: f32 = 0.55;

/// Hard cap on the OR-clause cardinality to avoid pathological FTS5
/// query parses on rare multi-concept queries. 200 quoted terms in
/// one MATCH is comfortably handled by SQLite's parser.
const TERM_CAP: usize = 200;

#[derive(Debug, Serialize)]
pub struct ConceptSearchHit {
    pub path: String,
    pub name: String,
    pub library_name: String,
    /// FTS5 `snippet()` output with CHAR(2)/CHAR(3) markers around
    /// matched tokens. Frontend renders these as `<mark>` spans.
    pub snippet: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConceptSearchRequest {
    pub query: String,
    /// Optional library scope. `None` searches across every library
    /// in the active Universe (cross-library, cross-language).
    pub library_name: Option<String>,
    pub limit: Option<u32>,
    pub min_score: Option<f32>,
}

/// Embed the query, map to top-K M11 concepts, expand to term_vocab,
/// run an FTS5 OR-clause MATCH, return note hits with snippets.
///
/// Returns `Ok(Vec::new())` on empty/whitespace queries, on no
/// concept above the threshold, and on no terms mapped to the
/// surviving concepts. The caller (frontend) handles all three the
/// same way: render an empty result set.
#[tauri::command]
pub fn ctse_search_by_concept(
    app: tauri::AppHandle,
    request: ConceptSearchRequest,
) -> Result<Vec<ConceptSearchHit>, String> {
    let q = request.query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let limit = request.limit.unwrap_or(50).min(500) as i64;
    let min_score = request.min_score.unwrap_or(DEFAULT_MIN_SCORE);

    // Step 1 — embed the query (auto-inits engine on first call).
    let query_vec = embeddings::constellation_embed_text(app.clone(), q.to_string())?;

    // Step 2 — top-K concepts above threshold.
    let store = bridge_vectors::get();
    let top_concepts = store.nearest_concepts_k(&query_vec, CONCEPT_TOP_K);
    let concept_ids: Vec<String> = top_concepts
        .into_iter()
        .filter(|(_, score)| *score >= min_score)
        .filter_map(|(row, _)| store.concept_id(row).map(str::to_string))
        .collect();
    if concept_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Steps 3–5 share the SearchState lock.
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search database not initialized")?;

    // Step 3 — pull every term_vocab row whose bridge_concept_id is in
    // the surviving concept set. Skip bigrams (joined by U+001F /
    // CHAR(31)) — they aren't lexicon-resolvable so should never have
    // landed here in the first place, but defensive.
    let placeholders = concept_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let term_sql = format!(
        "SELECT term FROM term_vocab \
         WHERE bridge_concept_id IN ({}) \
           AND term NOT LIKE '%' || CHAR(31) || '%'",
        placeholders
    );
    let terms: Vec<String> = {
        let mut stmt = conn.prepare(&term_sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(concept_ids.iter()), |row| {
                row.get::<_, String>(0)
            })
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect()
    };
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    // Step 4 — build the FTS5 MATCH query. Each term is wrapped in
    // double quotes so the FTS5 query parser treats it as a phrase
    // (escapes special chars). Embedded `"` is doubled per FTS5
    // grammar. The OR clause is capped at `TERM_CAP` to keep the
    // parser in its happy path.
    let match_query: String = terms
        .iter()
        .take(TERM_CAP)
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" OR ");

    // Step 5 — FTS5 MATCH joined with note_meta. The library filter
    // is applied as a separate clause (note_meta.library_name is
    // indexed via `idx_note_library`, so the planner can intersect
    // efficiently).
    let mut hits = Vec::new();
    if let Some(library) = request.library_name.as_deref() {
        let sql = "SELECT nm.path, nm.name, nm.library_name, \
                          snippet(notes_fts, -1, CHAR(2), CHAR(3), '…', 12) \
                   FROM notes_fts \
                   JOIN note_meta nm ON nm.rowid = notes_fts.rowid \
                   WHERE notes_fts MATCH ?1 \
                     AND nm.library_name = ?2 \
                   ORDER BY rank \
                   LIMIT ?3";
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query(params![match_query, library, limit])
            .map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            hits.push(ConceptSearchHit {
                path: row.get(0).map_err(|e| e.to_string())?,
                name: row.get(1).map_err(|e| e.to_string())?,
                library_name: row.get(2).map_err(|e| e.to_string())?,
                snippet: row.get(3).ok(),
            });
        }
    } else {
        let sql = "SELECT nm.path, nm.name, nm.library_name, \
                          snippet(notes_fts, -1, CHAR(2), CHAR(3), '…', 12) \
                   FROM notes_fts \
                   JOIN note_meta nm ON nm.rowid = notes_fts.rowid \
                   WHERE notes_fts MATCH ?1 \
                   ORDER BY rank \
                   LIMIT ?2";
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query(params![match_query, limit])
            .map_err(|e| e.to_string())?;
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            hits.push(ConceptSearchHit {
                path: row.get(0).map_err(|e| e.to_string())?,
                name: row.get(1).map_err(|e| e.to_string())?,
                library_name: row.get(2).map_err(|e| e.to_string())?,
                snippet: row.get(3).ok(),
            });
        }
    }
    Ok(hits)
}
