//! CTSE concept-based semantic search (MIG-013 §1D, query-time
//! expansion architecture).
//!
//! ## How it closes the cross-language loop
//!
//! 1. The user's query is embedded once via multilingual-e5-small.
//! 2. Cosine k-NN against the baked 20K M11 concept-vector matrix
//!    yields the top-K nearest concept IDs (each `c:…`).
//! 3. For each surviving concept ID we expand it back to its lemmas
//!    in every supported language using a process-wide
//!    `concept_id → [lemma]` index built once at boot from
//!    `LexiconGraph`. So `c:knowledge` expands to `["knowledge",
//!    "معرفة", "wissen", "savoir", …]`.
//! 4. Those lemmas are joined into a single FTS5 OR-clause MATCH
//!    against `notes_fts`. The FTS5 tokenizer stems every lemma the
//!    same way it stemmed the indexed bodies, so "knowledge" matches
//!    documents containing "knowledge", "knowl-", or any of the same
//!    concept's other-language lemmas.
//!
//! ## Why query-time expansion (not pre-computed `bridge_concept_id`)
//!
//! Mature search systems uniformly do **query-time concept/synonym
//! expansion**, not document-side concept tagging:
//!
//! - **Lucene/Elasticsearch**: `SynonymGraphFilter` (2017+) is
//!   recommended at query time over the deprecated index-time
//!   `SynonymFilter`, because index-time flattens token graphs and
//!   breaks phrase queries.
//! - **SQLite FTS5**: documented "Method 2 — query-time synonym
//!   expansion" is one of the three core synonym strategies.
//! - **CLIR (Cross-Language Information Retrieval)**: query
//!   translation is the canonical technique. The query is short and
//!   dynamic; the corpus is large and static.
//! - **Library platforms (Primo, Ex Libris)**: controlled-vocabulary
//!   expansion runs at query time so the index doesn't need to be
//!   re-tagged when the vocabulary changes.
//!
//! Constellation's CTSE follows the same pattern: M11 is the
//! controlled vocabulary; user queries get expanded into M11
//! concept IDs and then into multilingual lemmas; the FTS5 MATCH
//! against `notes_fts` is the same shape every other lexical search
//! uses. There is no per-term bridge column, no boot-time backfill,
//! no first-fill — adding a new M11 release picks up automatically
//! on the next query.
//!
//! ## Cost profile
//!
//! - Boot: build `concept_id → lemmas` map once via a single pass
//!   over `LexiconGraph::nodes`. ~10 ms on Boss's 200K-node M11,
//!   ~5 MB resident. Cached forever via `OnceLock`.
//! - Per query: ~50 ms e5 inference + ~5 ms cosine k-NN + sub-ms
//!   in-memory expansion + ~20–50 ms FTS5 MATCH.
//! - Frontend MUST debounce ≥300 ms (CLAUDE.md Rule 3).

use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use tauri::Manager;

use crate::bridge_vectors;
use crate::embeddings;
use crate::lexicon::LexiconGraph;
use crate::search::SearchState;

/// Top-K nearest concept count. Each concept expands to ~10–15
/// lemmas (all the languages M11 covers), so the OR-clause has
/// roughly K × 12 entries. K=10 lands ~120 lemmas — comfortably
/// within FTS5's parser budget.
const CONCEPT_TOP_K: usize = 10;

/// Minimum cosine score for a concept to be included. Below this
/// the concept is "too far" from the query and would dilute results
/// with semantic noise. Tunable via the request payload.
const DEFAULT_MIN_SCORE: f32 = 0.55;

/// Hard cap on lemma cardinality in the FTS5 OR-clause. 200 quoted
/// terms in one MATCH is comfortably handled by SQLite's parser;
/// going much higher risks "too many query terms" errors.
const LEMMA_CAP: usize = 200;

#[derive(Debug, Serialize)]
pub struct ConceptSearchHit {
    pub path: String,
    pub name: String,
    pub library_name: String,
    /// FTS5 `snippet()` output with CHAR(2)/CHAR(3) markers around
    /// matched tokens. Frontend renders as `<mark>` spans.
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

/// Process-wide `concept_id → [lemma]` reverse index. Built lazily
/// on first call by walking `LexiconGraph::nodes` once. Each lemma
/// appears once per concept it belongs to (multilingual coverage —
/// "knowledge" and "معرفة" both land under `c:knowledge`).
///
/// Lemmas are stored as bare strings (no `Lang` tag) because the
/// FTS5 tokenizer doesn't care about source language — it stems
/// every input through the same pipeline. Including the language
/// would add memory without adding retrieval signal.
fn concept_lemmas() -> &'static HashMap<String, Vec<String>> {
    static CACHE: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let graph = LexiconGraph::get();
        let mut map: HashMap<String, Vec<String>> = HashMap::with_capacity(20_000);
        for node in &graph.nodes {
            map.entry(node.concept_id.clone())
                .or_default()
                .push(node.lemma.clone());
        }
        // De-duplicate per concept (rare but possible if the same
        // surface appears under multiple senses with the same
        // concept ID). Cheap — concept lemma lists are small.
        for lemmas in map.values_mut() {
            lemmas.sort();
            lemmas.dedup();
        }
        map
    })
}

/// Embed the query, map to top-K M11 concepts, expand to their
/// multilingual lemmas, run an FTS5 OR-clause MATCH, return note
/// hits with snippets.
///
/// Returns `Ok(Vec::new())` on empty/whitespace queries, on no
/// concept above the score threshold, and on no lemmas (a concept
/// with empty lemma list — vanishingly rare). The caller (frontend)
/// renders all three the same: empty result set.
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

    // Step 3 — expand each concept ID to its multilingual lemmas
    // via the in-memory map. Union + dedupe across concepts.
    let map = concept_lemmas();
    let mut lemma_set: std::collections::HashSet<String> =
        std::collections::HashSet::with_capacity(concept_ids.len() * 12);
    for cid in &concept_ids {
        if let Some(lemmas) = map.get(cid) {
            for lemma in lemmas {
                lemma_set.insert(lemma.clone());
            }
        }
    }
    if lemma_set.is_empty() {
        return Ok(Vec::new());
    }

    // Step 4 — build the FTS5 MATCH query. Each lemma is wrapped in
    // double quotes so the FTS5 query parser treats it as a phrase
    // (escaping special chars). Embedded `"` is doubled per FTS5
    // grammar. Cardinality cap protects the parser.
    let lemmas: Vec<String> = lemma_set.into_iter().take(LEMMA_CAP).collect();
    let match_query: String = lemmas
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" OR ");

    // Step 5 — FTS5 MATCH joined with note_meta. Optional library
    // filter uses `idx_note_library` for an indexed intersection
    // with the FTS5 result set.
    let state = app.state::<SearchState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_ref().ok_or("Search database not initialized")?;

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity: the lazy `concept_id → lemmas` map covers a known
    /// English lemma and an Arabic equivalent under the same
    /// concept ID. This exercises both the cache build and the
    /// cross-language coverage that the search path depends on.
    #[test]
    fn concept_lemmas_includes_book_in_multiple_languages() {
        let map = concept_lemmas();
        assert!(!map.is_empty(), "concept_lemmas map must not be empty");
        // Find any concept whose English lemma is "book".
        let book_concept = map
            .iter()
            .find(|(_, lemmas)| lemmas.iter().any(|l| l == "book"));
        assert!(
            book_concept.is_some(),
            "M11 should have a concept whose lemmas include 'book'"
        );
        let (_, lemmas) = book_concept.unwrap();
        // M11 ships multilingual coverage; assert at least one
        // non-Latin-alphabet lemma is present (Arabic, CJK, etc.).
        let has_non_latin = lemmas.iter().any(|l| {
            l.chars().any(|c| !c.is_ascii() && c.is_alphabetic())
        });
        assert!(
            has_non_latin,
            "expected at least one non-ASCII lemma alongside 'book'; got {lemmas:?}",
        );
    }
}
