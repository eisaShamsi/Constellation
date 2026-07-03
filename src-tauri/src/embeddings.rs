//! Rust-native ONNX embedding engine for semantic search.
//! Uses `ort` (ONNX Runtime) + `tokenizers` (HuggingFace) for 100% offline inference.
//! Model: multilingual-e5-small (384-dim, 100 languages).

use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use rusqlite::params;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tokenizers::Tokenizer;

// ─── State ────────────────────────────────────────────────────

pub struct EmbeddingState {
    pub engine: Mutex<Option<EmbeddingEngine>>,
    /// Cancel flag for long-running embedding jobs. Originally added
    /// for the MIG-012 term-embedding bootstrap; reused in MIG-013 §1C
    /// by `ctse::backfill::ctse_run_backfill` (the per-Universe slow-path
    /// concept-resolution job). The semantics are unchanged — flip true
    /// to request a graceful stop at the next safe checkpoint.
    pub term_embed_cancel: std::sync::atomic::AtomicBool,
}

// ─── f32 BLOB helper (used by note_embeddings) ────────────────

/// Pack an f32 vector into a little-endian byte BLOB suitable for
/// SQLite storage. Used by `constellation_embed_notes` when writing
/// note-level embeddings into the `note_embeddings` table.
fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

pub struct EmbeddingEngine {
    session: Mutex<Session>,
    tokenizer: Tokenizer,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingStatus {
    pub ready: bool,
    pub embedded_count: u32,
    pub model_loaded: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct NoteForEmbedding {
    pub path: String,
    pub name: String,
    pub content: String,
}

// ─── Engine ───────────────────────────────────────────────────

fn resolve_model_path(app: &tauri::AppHandle) -> Result<(PathBuf, PathBuf), String> {
    // Try bundled resource directory first (production builds)
    if let Ok(resource_dir) = app.path().resource_dir() {
        let model_path = resource_dir.join("models").join("model.onnx");
        let tokenizer_path = resource_dir.join("models").join("tokenizer.json");
        if model_path.exists() && tokenizer_path.exists() {
            return Ok((model_path, tokenizer_path));
        }
    }

    // Fallback: src-tauri/models/ directory (development mode)
    let dev_candidates = [
        PathBuf::from("models"),                           // relative to CWD (src-tauri/)
        PathBuf::from("src-tauri/models"),                 // relative to project root
        std::env::current_exe()                            // relative to exe location
            .unwrap_or_default()
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("models"),
    ];

    for dir in &dev_candidates {
        let model_path = dir.join("model.onnx");
        let tokenizer_path = dir.join("tokenizer.json");
        if model_path.exists() && tokenizer_path.exists() {
            eprintln!("[Embeddings] Found models at {:?}", dir);
            return Ok((model_path, tokenizer_path));
        }
    }

    Err("Model files not found. Expected model.onnx + tokenizer.json in models/ directory.".to_string())
}

fn init_engine(model_path: &PathBuf, tokenizer_path: &PathBuf) -> Result<EmbeddingEngine, String> {
    let session = Session::builder()
        .map_err(|e| format!("Failed to create session builder: {}", e))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| format!("Failed to set optimization: {}", e))?
        .with_intra_threads(2)
        .map_err(|e| format!("Failed to set threads: {}", e))?
        .commit_from_file(model_path)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    let tokenizer = Tokenizer::from_file(tokenizer_path)
        .map_err(|e| format!("Failed to load tokenizer: {}", e))?;

    Ok(EmbeddingEngine { session: Mutex::new(session), tokenizer })
}

// MIG-021 §1B: exposed pub(crate) so the classifier module can reuse
// the cached engine for source-classification without spinning up a
// second ONNX session. Caller must hold the EmbeddingState lock.
pub(crate) fn run_embedding(engine: &EmbeddingEngine, text: &str) -> Result<Vec<f32>, String> {
    // Truncate to ~512 tokens worth of text (char-safe for multi-byte UTF-8)
    let truncated = if text.len() > 2000 {
        let mut end = 2000;
        while end > 0 && !text.is_char_boundary(end) { end -= 1; }
        &text[..end]
    } else { text };

    let encoding = engine.tokenizer.encode(truncated, true)
        .map_err(|e| format!("Tokenization failed: {}", e))?;

    // Truncate to max 512 tokens (model's max sequence length)
    const MAX_TOKENS: usize = 512;
    let ids_raw = encoding.get_ids();
    let mask_raw = encoding.get_attention_mask();
    let type_raw = encoding.get_type_ids();
    let seq_len = ids_raw.len().min(MAX_TOKENS);

    let input_ids: Vec<i64> = ids_raw[..seq_len].iter().map(|&id| id as i64).collect();
    let attention_mask: Vec<i64> = mask_raw[..seq_len].iter().map(|&m| m as i64).collect();
    let token_type_ids: Vec<i64> = type_raw[..seq_len].iter().map(|&t| t as i64).collect();

    // ort v2: Tensor::from_array takes (shape_tuple, data_vec)
    let ids_tensor = Tensor::from_array(([1usize, seq_len], input_ids.into_boxed_slice()))
        .map_err(|e| format!("Tensor creation failed: {}", e))?;
    let mask_tensor = Tensor::from_array(([1usize, seq_len], attention_mask.clone().into_boxed_slice()))
        .map_err(|e| format!("Tensor creation failed: {}", e))?;
    let type_tensor = Tensor::from_array(([1usize, seq_len], token_type_ids.into_boxed_slice()))
        .map_err(|e| format!("Tensor creation failed: {}", e))?;

    // Lock session for mutable access (ort v2 requires &mut self for run)
    let mut session = engine.session.lock().map_err(|e| e.to_string())?;
    let outputs = session
        .run(ort::inputs![ids_tensor, mask_tensor, type_tensor])
        .map_err(|e| format!("Inference failed: {}", e))?;

    // ort v2: try_extract_tensor returns (&Shape, &[f32])
    let (shape, data) = outputs[0].try_extract_tensor::<f32>()
        .map_err(|e| format!("Output extraction failed: {}", e))?;

    if shape.len() != 3 {
        return Err(format!("Unexpected output shape: {:?}", shape));
    }
    let embed_dim = shape[2] as usize;

    // Mean pooling with attention mask
    let mut result = vec![0.0f32; embed_dim];
    let mut count = 0.0f32;
    for i in 0..seq_len {
        if attention_mask[i] == 1 {
            let offset = i * embed_dim; // data layout: [1, seq_len, embed_dim] flattened
            for j in 0..embed_dim {
                result[j] += data[offset + j];
            }
            count += 1.0;
        }
    }
    if count > 0.0 {
        for v in result.iter_mut() { *v /= count; }
    }

    // L2 normalize
    let norm: f32 = result.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in result.iter_mut() { *v /= norm; }
    }

    Ok(result)
}

/// Batched inference. Replaces per-call ONNX overhead (~60-70 ms each)
/// with a single inference per batch of N inputs (~80 ms per batch of
/// 32 → ~400 inputs/sec). Used by `embed_passages_standalone` (the
/// MIG-013 §1A build-time helper). Same model, same prefixes, same
/// L2-normalization — just amortizes the inference fixed overhead.
///
/// Returns one Vec<f32> per input text in input order. The caller is
/// responsible for the "passage: " / "query: " prefix.
// MIG-040 (NSC): pub(crate) so the Note Summary Creator can batch-embed a
// note's sentences for extractive TextRank without a second ONNX session.
pub(crate) fn run_embedding_batch(engine: &EmbeddingEngine, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }

    // Tokenize each text. Truncate text bytes first (UTF-8-safe) so the
    // tokenizer doesn't choke on huge inputs, then truncate token
    // sequence to MAX_TOKENS. Track the per-batch max sequence length
    // so we know how much to pad shorter rows to.
    const MAX_TOKENS: usize = 512;
    let mut encoded = Vec::with_capacity(texts.len());
    let mut max_len = 1usize;
    for text in texts {
        let truncated: &str = if text.len() > 2000 {
            let mut end = 2000;
            while end > 0 && !text.is_char_boundary(end) {
                end -= 1;
            }
            &text[..end]
        } else {
            text.as_str()
        };
        let encoding = engine
            .tokenizer
            .encode(truncated, true)
            .map_err(|e| format!("Tokenization failed: {}", e))?;
        let len = encoding.get_ids().len().min(MAX_TOKENS);
        if len > max_len {
            max_len = len;
        }
        encoded.push(encoding);
    }

    // Pack into [batch_size, max_len] tensors. Shorter rows are padded
    // with token_id=0 + attention_mask=0 (the standard padding scheme).
    let batch_size = texts.len();
    let total = batch_size * max_len;
    let mut input_ids = vec![0i64; total];
    let mut attention_mask = vec![0i64; total];
    let mut token_type_ids = vec![0i64; total];
    for (i, encoding) in encoded.iter().enumerate() {
        let ids = encoding.get_ids();
        let mask = encoding.get_attention_mask();
        let types = encoding.get_type_ids();
        let len = ids.len().min(MAX_TOKENS);
        let row_off = i * max_len;
        for j in 0..len {
            input_ids[row_off + j] = ids[j] as i64;
            attention_mask[row_off + j] = mask[j] as i64;
            token_type_ids[row_off + j] = types[j] as i64;
        }
    }

    // Build tensors. The mask is also kept in scope below for the
    // mean-pool weighting; we clone the bytes used for the tensor.
    let mask_for_pool = attention_mask.clone();
    let ids_tensor = ort::value::Tensor::from_array(([batch_size, max_len], input_ids.into_boxed_slice()))
        .map_err(|e| format!("Tensor creation failed: {}", e))?;
    let mask_tensor = ort::value::Tensor::from_array(([batch_size, max_len], attention_mask.into_boxed_slice()))
        .map_err(|e| format!("Tensor creation failed: {}", e))?;
    let type_tensor = ort::value::Tensor::from_array(([batch_size, max_len], token_type_ids.into_boxed_slice()))
        .map_err(|e| format!("Tensor creation failed: {}", e))?;

    let mut session = engine.session.lock().map_err(|e| e.to_string())?;
    let outputs = session
        .run(ort::inputs![ids_tensor, mask_tensor, type_tensor])
        .map_err(|e| format!("Inference failed: {}", e))?;
    let (shape, data) = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Output extraction failed: {}", e))?;
    if shape.len() != 3 || shape[0] as usize != batch_size {
        return Err(format!("Unexpected output shape: {:?}", shape));
    }
    let embed_dim = shape[2] as usize;

    // Per-row mean pool with attention mask + L2-normalize (identical
    // to the single-row run_embedding pooling, just iterated per row).
    let mut results = Vec::with_capacity(batch_size);
    for b in 0..batch_size {
        let mut vec = vec![0.0f32; embed_dim];
        let mut count = 0.0f32;
        for s in 0..max_len {
            if mask_for_pool[b * max_len + s] == 1 {
                let offset = (b * max_len + s) * embed_dim;
                for j in 0..embed_dim {
                    vec[j] += data[offset + j];
                }
                count += 1.0;
            }
        }
        if count > 0.0 {
            for v in vec.iter_mut() {
                *v /= count;
            }
        }
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in vec.iter_mut() {
                *v /= norm;
            }
        }
        results.push(vec);
    }
    Ok(results)
}

/// MIG-013 §1A — Build-time / standalone embedding helper.
///
/// Builds its own Session + Tokenizer from the given paths (no
/// AppHandle, no Tauri runtime), then chunks the input through the
/// same batched mean-pool + L2-normalize pipeline as
/// `run_embedding_batch`. Used by the offline
/// `build_concept_vectors` [[bin]] to embed M11's ~20K concepts at
/// build time and bake the result into the binary asset shipped
/// with the app.
///
/// Returns one Vec<f32> per input text in input order. The caller
/// is responsible for prepending "passage: " (for indexed content)
/// or "query: " (for queries) per the e5 model card.
///
/// `intra_threads` is honored verbatim (clamped to >=1); the build
/// helper passes physical-core count for max throughput. The
/// runtime engine still uses 2 (set in `init_engine`) to avoid
/// fighting Tauri's thread pool.
pub fn embed_passages_standalone(
    model_path: &std::path::Path,
    tokenizer_path: &std::path::Path,
    texts: &[String],
    intra_threads: usize,
    batch_size: usize,
) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let session = Session::builder()
        .map_err(|e| format!("Failed to create session builder: {}", e))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| format!("Failed to set optimization: {}", e))?
        .with_intra_threads(intra_threads.max(1))
        .map_err(|e| format!("Failed to set threads: {}", e))?
        .commit_from_file(model_path)
        .map_err(|e| format!("Failed to load model: {}", e))?;
    let tokenizer = Tokenizer::from_file(tokenizer_path)
        .map_err(|e| format!("Failed to load tokenizer: {}", e))?;
    let engine = EmbeddingEngine { session: Mutex::new(session), tokenizer };

    let bs = batch_size.max(1);
    let mut all = Vec::with_capacity(texts.len());
    for chunk in texts.chunks(bs) {
        let batch_in: Vec<String> = chunk.to_vec();
        let batch_out = run_embedding_batch(&engine, &batch_in)?;
        all.extend(batch_out);
    }
    Ok(all)
}

// ─── Tauri Commands ──────────────────────────────────────────

/// Initialize the embedding engine (load model + tokenizer).
#[tauri::command]
pub fn constellation_init_embeddings(app: tauri::AppHandle) -> Result<String, String> {
    let (model_path, tokenizer_path) = resolve_model_path(&app)?;
    let engine = init_engine(&model_path, &tokenizer_path)?;

    let state = app.state::<EmbeddingState>();
    let mut guard = state.engine.lock().map_err(|e| e.to_string())?;
    *guard = Some(engine);

    Ok("Embedding engine loaded".to_string())
}

/// Ensure the engine is loaded (lazy init on first use).
/// MIG-021 §1B: pub(crate) so the classifier module can ensure the
/// engine before classifying; lazy load preserves boot-perf budget.
pub(crate) fn ensure_engine(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<EmbeddingState>();
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    if guard.is_some() { return Ok(()); }
    drop(guard); // release lock before init
    constellation_init_embeddings(app.clone())
        .map(|_| ())
}

/// Embed a single text (for query embedding in search).
/// Uses "query: " prefix for e5 models. Auto-inits engine if needed.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn constellation_embed_text(
    app: tauri::AppHandle,
    text: String,
) -> Result<Vec<f32>, String> {
    ensure_engine(&app)?;
    let state = app.state::<EmbeddingState>();
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("Embedding engine not initialized")?;

    let prefixed = format!("query: {}", text);
    run_embedding(engine, &prefixed)
}

/// MIG-071 audit (OGA) — batch-embed arbitrary passages with the bundled local ONNX engine and
/// RETURN the vectors (not stored). Lets the Sky-View "compute semantic links" feature run fully
/// offline through the same local model as search, instead of @xenova/transformers (which fetched
/// the model from the HuggingFace CDN at runtime). Caller passes already-summarised note texts.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn constellation_embed_texts(
    app: tauri::AppHandle,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>, String> {
    ensure_engine(&app)?;
    let state = app.state::<EmbeddingState>();
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("Embedding engine not initialized")?;
    // Documents use the e5 "passage: " prefix (mirrors constellation_embed_notes).
    let prefixed: Vec<String> = texts.iter().map(|t| format!("passage: {}", t)).collect();
    run_embedding_batch(engine, &prefixed)
}

/// Batch embed notes and store in the search database.
/// `force`: if true, re-embed even if already exists (for edited notes).
///
/// PJ-066 follow-up (2026-06-27): `#[tauri::command(async)]` — the e5 `run_embedding`
/// inference is CPU-bound and multi-second on a large note (~32s on the 533-link
/// "Ancient history" Wikipedia import, measured). As a SYNC command it ran on the
/// single IPC dispatch thread and FROZE the whole app for that duration on every
/// save. Async moves the inference onto Tauri's worker pool (the §C1 fix that the
/// reindex already got; this is the sibling that was missed). The invoke contract is
/// unchanged — the Promise still resolves on completion.
#[tauri::command(async)]
pub fn constellation_embed_notes(
    app: tauri::AppHandle,
    notes: Vec<NoteForEmbedding>,
    force: Option<bool>,
) -> Result<u32, String> {
    ensure_engine(&app)?;
    let state = app.state::<EmbeddingState>();
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or("Embedding engine not initialized")?;

    let search_state = app.state::<crate::search::SearchState>();

    let force = force.unwrap_or(false);
    let mut count = 0u32;
    for note in &notes {
        // MIG-076 §D (2026-06-13) — the search DB lock is now scoped to the
        // existence check and the INSERT only. PREVIOUSLY a single db_guard was
        // held across the whole loop INCLUDING run_embedding (the CPU-bound,
        // multi-second model inference): a save's background embed therefore
        // held the lock for the full inference and BLOCKED every other DB op —
        // notably a subsequent rename's reindex_single_note — for ~10s (the
        // "slow rename after edit" Boss saw). Inference now runs lock-free.

        // Skip if already embedded (unless force re-embed) — brief lock.
        if !force {
            let existing: bool = {
                let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
                let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
                conn.query_row(
                    "SELECT COUNT(*) > 0 FROM note_embeddings WHERE path = ?1",
                    params![note.path],
                    |row| row.get(0),
                ).unwrap_or(false)
            };
            if existing { count += 1; continue; }
        }

        // Model inference — NO DB lock held (the whole point of this scoping).
        let text = format!("passage: {} {}", note.name.replace(".md", ""), note.content);
        match run_embedding(engine, &text) {
            Ok(embedding) => {
                let bytes = vec_to_blob(&embedding);
                let dims = embedding.len() as i32;
                // Write — brief lock.
                let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
                let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
                conn.execute(
                    "INSERT OR REPLACE INTO note_embeddings (path, embedding, dimensions, model_id, cid_cn) VALUES (?1, ?2, ?3, ?4, (SELECT cid_cn FROM note_meta WHERE path = ?1))",
                    params![note.path, bytes, dims, "multilingual-e5-small"],
                ).map_err(|e| format!("DB write failed: {}", e))?;
                count += 1;
            }
            Err(e) => eprintln!("[Embeddings] Failed to embed {}: {}", note.name, e),
        }
    }

    Ok(count)
}

/// Get embedding status.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn constellation_embedding_status(
    app: tauri::AppHandle,
) -> Result<EmbeddingStatus, String> {
    let state = app.state::<EmbeddingState>();
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    let model_loaded = guard.is_some();

    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;

    let embedded_count = if let Some(conn) = db_guard.as_ref() {
        conn.query_row("SELECT COUNT(*) FROM note_embeddings", [], |row| row.get::<_, u32>(0))
            .unwrap_or(0)
    } else {
        0
    };

    Ok(EmbeddingStatus {
        ready: model_loaded,
        embedded_count,
        model_loaded,
    })
}
