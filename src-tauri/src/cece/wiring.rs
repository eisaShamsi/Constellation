//! MIG-021v3 V3-§8 — Orchestrator wiring.
//!
//! Builds the production Orchestrator: instantiates all six catalogers,
//! wires their injectable functions (embed / lookup / inference) to the
//! real backends (EmbeddingState for embeddings, SearchState for kNN
//! lookup + typed-neighbor lookup), registers each cataloger with its
//! cost tier so the orchestrator's cascade order is correct.
//!
//! Per Architect §9 + V3-§7 deferred decision: the Reasoning Cataloger's
//! InferenceFn is wired as None for now (llama-cpp-2 dep + Qwen3-4B GGUF
//! deferred to V3-§7.b). It abstains gracefully; the ensemble still runs
//! with five other catalogers.

use crate::cece::cataloger::TypedNeighbor;
use crate::cece::catalogers::graph::GraphCataloger;
use crate::cece::catalogers::linguistic::LinguisticCataloger;
use crate::cece::catalogers::reasoning::ReasoningCataloger;
use crate::cece::catalogers::semantic::{NeighborRecord, SemanticCataloger};
use crate::cece::catalogers::structural::StructuralCataloger;
use crate::cece::catalogers::user_authority::UserAuthorityCataloger;
use crate::cece::orchestrator::Orchestrator;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

/// Build a production-wired orchestrator with all six catalogers
/// registered in cost order. Called by `classifier_suggest_for_note`
/// once per request (cataloger init is cheap; the heavy lifting is
/// inside their classify() calls).
///
/// V3-§8.r2.b fix (audit Software Architecture #5): shares one
/// e5-small inference between Linguistic and Semantic via a
/// per-call `MemoizedEmbed` cache, instead of letting each cataloger
/// independently call `embed_fn`. Saves ~30 ms × 7K = ~3.5 minutes
/// on a full-Library scan when both catalogers want to embed the
/// same note text.
pub fn build_orchestrator(app: &AppHandle) -> Orchestrator {
    let mut o = Orchestrator::new();

    // Build the per-IPC embedding cache. Both Linguistic (Bridge slow-
    // path) and Semantic (kNN-blend) consult `embed` for the same note
    // text; without memoization that's 2 ONNX calls per note. The
    // cache lives for the lifetime of this orchestrator (one IPC).
    let app_for_embed = app.clone();
    let embed_cache: Arc<MemoizedEmbed> = Arc::new(MemoizedEmbed::new(Box::new(
        move |text: &str| embed_text(&app_for_embed, text),
    )));

    // ─── Cost 0: cheap, run always ───
    o.register(Arc::new(UserAuthorityCataloger::new()), 0);
    o.register(Arc::new(StructuralCataloger::new()), 0);

    // Linguistic uses the shared embedder for Bridge slow-path.
    let embed_for_linguistic = embed_cache.clone();
    let linguistic = crate::cece::catalogers::linguistic::LinguisticCataloger::with_embedder(
        Box::new(move |text: &str| embed_for_linguistic.embed(text)),
    );
    o.register(Arc::new(linguistic), 0);

    // ─── Cost 1: medium (DB queries) ───
    let app_for_graph = app.clone();
    let graph = GraphCataloger::with_lookup(Box::new(move |note_path: &str| {
        load_typed_neighbors(&app_for_graph, note_path)
    }));
    o.register(Arc::new(graph), 1);

    let embed_for_semantic = embed_cache.clone();
    let app_for_semantic_lookup = app.clone();
    let semantic = SemanticCataloger::with_io(
        Box::new(move |text: &str| embed_for_semantic.embed(text)),
        Box::new(move |query: &[f32], k: usize| {
            knn_classified_neighbors(&app_for_semantic_lookup, query, k)
        }),
    );
    o.register(Arc::new(semantic), 1);

    // ─── Cost 2: expensive (LLM inference) ───
    // Reasoning Cataloger has no inference fn yet (llama-cpp-2 deferred
    // to V3-§7.b). Abstains gracefully; ensemble still works.
    o.register(Arc::new(ReasoningCataloger::new()), 2);

    o
}

/// Per-call embedding cache. Holds one closure that calls the real
/// embedder, plus a Mutex<Option<Vec<f32>>> result cache keyed by
/// text. We key by text-string to handle the (rare) case where two
/// catalogers want to embed *different* substrings of the same note;
/// in practice both Linguistic and Semantic embed the full note body
/// and we get one cache hit.
pub struct MemoizedEmbed {
    inner: Box<dyn Fn(&str) -> Result<Vec<f32>, String> + Send + Sync + 'static>,
    cache: Mutex<Option<(String, Vec<f32>)>>,
}

impl MemoizedEmbed {
    pub fn new(
        inner: Box<dyn Fn(&str) -> Result<Vec<f32>, String> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            inner,
            cache: Mutex::new(None),
        }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        // Recover from poisoning gracefully (audit P1.3 mitigation pattern).
        let mut guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((cached_text, cached_vec)) = guard.as_ref() {
            if cached_text == text {
                return Ok(cached_vec.clone());
            }
        }
        let v = (self.inner)(text)?;
        *guard = Some((text.to_string(), v.clone()));
        Ok(v)
    }
}

/// Embedding helper: ensures the e5-small ONNX engine is loaded, then
/// runs a single-text embedding. Returns Err on any failure (caller
/// abstains — never crashes the orchestrator).
fn embed_text(app: &AppHandle, text: &str) -> Result<Vec<f32>, String> {
    crate::embeddings::ensure_engine(app)?;
    let state = app.state::<crate::embeddings::EmbeddingState>();
    // V3-§8.r4.2 (audit P1.3): poison recovery on the embedding-engine
    // mutex too. The engine itself is just a Session + Tokenizer; no
    // in-memory invariant requires poisoning protection.
    let guard = state.engine.lock().unwrap_or_else(|e| e.into_inner());
    let engine = guard
        .as_ref()
        .ok_or("Embedding engine not initialized")?;
    crate::embeddings::run_embedding(engine, text)
}

/// kNN lookup for the Semantic Cataloger. Finds the top-k already-
/// classified neighbors by cosine similarity in the search DB's
/// note_embeddings + note_meta tables.
///
/// Implementation note: this loads ALL classified-note embeddings into
/// memory and computes cosine in Rust. Fine for vaults up to ~10k
/// classified notes (~12MB for 384-dim float32 embeddings); a future
/// optimization can switch to an ANN index. For first ship, brute-
/// force is correct + simple.
fn knn_classified_neighbors(
    app: &AppHandle,
    query: &[f32],
    k: usize,
) -> Result<Vec<NeighborRecord>, String> {
    crate::search::ensure_search_db_ready(app)?;
    let state = app.state::<crate::search::SearchState>();
    let guard = state.db.lock().unwrap_or_else(|e| e.into_inner());
    let conn = guard
        .as_ref()
        .ok_or("Search DB not initialized")?;

    // Pull every note that has BOTH an embedding AND at least one
    // sources or content_type assignment.
    let mut stmt = conn
        .prepare(
            r#"
            SELECT m.path, m.sources, m.content_type, e.embedding
            FROM note_meta m
            JOIN note_embeddings e ON e.path = m.path
            WHERE (m.sources IS NOT NULL AND m.sources != '' AND m.sources != '[]')
               OR (m.content_type IS NOT NULL AND m.content_type != '' AND m.content_type != '[]')
            "#,
        )
        .map_err(|e| format!("prepare knn: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,            // path
                row.get::<_, Option<String>>(1)?,    // sources (JSON array)
                row.get::<_, Option<String>>(2)?,    // content_type (JSON array)
                row.get::<_, Vec<u8>>(3)?,           // embedding (raw bytes)
            ))
        })
        .map_err(|e| format!("query knn: {}", e))?;

    let mut scored: Vec<(NeighborRecord, f32)> = Vec::new();
    for row in rows {
        let (path, sources_json, ct_json, emb_bytes) =
            row.map_err(|e| format!("knn row: {}", e))?;
        let neighbor_emb = bytes_to_f32_vec(&emb_bytes);
        if neighbor_emb.len() != query.len() {
            continue; // dimension mismatch — skip silently
        }
        let cosine = cosine_similarity(query, &neighbor_emb);
        let sources = parse_json_string_array(sources_json.as_deref());
        let content_type = parse_json_string_array(ct_json.as_deref());
        scored.push((
            NeighborRecord {
                note_path: path,
                sources,
                content_type,
                cosine,
            },
            cosine,
        ));
    }

    // Sort by cosine descending, take top k.
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    Ok(scored.into_iter().map(|(r, _)| r).collect())
}

/// Typed-neighbor lookup for the Graph Cataloger. Joins note_links
/// with note_meta to surface each linked note's link type +
/// classifications.
fn load_typed_neighbors(
    app: &AppHandle,
    note_path: &str,
) -> Result<Vec<TypedNeighbor>, String> {
    crate::search::ensure_search_db_ready(app)?;
    let state = app.state::<crate::search::SearchState>();
    let guard = state.db.lock().unwrap_or_else(|e| e.into_inner());
    let conn = guard
        .as_ref()
        .ok_or("Search DB not initialized")?;

    // The note_links schema (search.rs:1679): source_path, target_path
    // (resolved when known), target_name (always present), link_type.
    // We look up both directions — this note as source AND as target —
    // since the semantic relationship informs both. Some links have
    // target_path NULL (unresolved wikilink); those skip the join with
    // note_meta and contribute no classification signal (filtered later).
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                COALESCE(l.target_path, l.target_name) AS neighbor_path,
                l.link_type,
                m.sources,
                m.content_type
            FROM note_links l
            LEFT JOIN note_meta m ON m.path = l.target_path
            WHERE l.source_path = ?1
            UNION ALL
            SELECT
                l.source_path AS neighbor_path,
                l.link_type,
                m.sources,
                m.content_type
            FROM note_links l
            LEFT JOIN note_meta m ON m.path = l.source_path
            WHERE l.target_path = ?1
            "#,
        )
        .map_err(|e| {
            format!("prepare typed-neighbors: {}", e)
        })?;

    let rows = stmt
        .query_map([note_path], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|e| format!("query typed-neighbors: {}", e))?;

    let mut out = Vec::new();
    for row in rows {
        let (neighbor_path, link_type, sources_json, ct_json) =
            row.map_err(|e| format!("typed-neighbor row: {}", e))?;
        out.push(TypedNeighbor {
            neighbor_path,
            link_type,
            neighbor_sources: parse_json_string_array(sources_json.as_deref()),
            neighbor_content_type: parse_json_string_array(ct_json.as_deref()),
        });
    }
    Ok(out)
}

// ─── Helpers ───────────────────────────────────────────────────────

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na < 1e-9 || nb < 1e-9 {
        0.0
    } else {
        dot / (na * nb)
    }
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    if bytes.len() % 4 != 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        out.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    out
}

/// Parse a JSON string-array out of a `note_meta.sources` /
/// `note_meta.content_type` cell, with diagnostic logging on
/// corruption (V3-§8.r4.7 — audit P2: silent dropping was leaving
/// Semantic Cataloger without signal with no log line).
fn parse_json_string_array(json: Option<&str>) -> Vec<String> {
    let Some(s) = json else { return Vec::new() };
    if s.is_empty() || s == "[]" {
        return Vec::new();
    }
    match serde_json::from_str::<Vec<String>>(s) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "[wiring] note_meta JSON-array parse failed (silently dropping for kNN/graph vote): {} — payload: {:?}",
                e,
                if s.len() > 200 { &s[..200] } else { s }
            );
            Vec::new()
        }
    }
}
