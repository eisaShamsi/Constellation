//! Rust-native ONNX embedding engine for semantic search.
//! Uses `ort` (ONNX Runtime) + `tokenizers` (HuggingFace) for 100% offline inference.
//! Model: multilingual-e5-small (384-dim, 100 languages).

use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use rusqlite::params;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tokenizers::Tokenizer;

// ─── State ────────────────────────────────────────────────────

pub struct EmbeddingState {
    pub engine: Mutex<Option<EmbeddingEngine>>,
    /// MIG-012 cancel flag for the term-embedding job. Lives on the
    /// per-app-instance state rather than as a process global so the
    /// scope is "this app" — one Universe at a time today, but
    /// future-proof if app instances ever multiplex.
    pub term_embed_cancel: std::sync::atomic::AtomicBool,
}

// ─── MIG-012 — f32 BLOB helpers (shared between note + term embeddings) ─

/// Pack an f32 vector into a little-endian byte BLOB suitable for
/// SQLite storage. Inverse of `blob_to_vec`. Used by both
/// `note_embeddings` and `term_embeddings` writers.
fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// Unpack a SQLite BLOB back into an f32 vector. Returns None if the
/// byte length isn't a multiple of 4 (corrupt row). Caller checks
/// length matches the expected dimensions.
fn blob_to_vec(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        out.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Some(out)
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

fn run_embedding(engine: &EmbeddingEngine, text: &str) -> Result<Vec<f32>, String> {
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
fn ensure_engine(app: &tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<EmbeddingState>();
    let guard = state.engine.lock().map_err(|e| e.to_string())?;
    if guard.is_some() { return Ok(()); }
    drop(guard); // release lock before init
    constellation_init_embeddings(app.clone())
        .map(|_| ())
}

/// Embed a single text (for query embedding in search).
/// Uses "query: " prefix for e5 models. Auto-inits engine if needed.
#[tauri::command]
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

/// Batch embed notes and store in the search database.
/// `force`: if true, re-embed even if already exists (for edited notes).
#[tauri::command]
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
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;

    let force = force.unwrap_or(false);
    let mut count = 0u32;
    for note in &notes {
        // Skip if already embedded (unless force re-embed)
        if !force {
            let existing: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM note_embeddings WHERE path = ?1",
                params![note.path],
                |row| row.get(0),
            ).unwrap_or(false);
            if existing { count += 1; continue; }
        }

        let text = format!("passage: {} {}", note.name.replace(".md", ""), note.content);
        match run_embedding(engine, &text) {
            Ok(embedding) => {
                let bytes = vec_to_blob(&embedding);
                let dims = embedding.len() as i32;
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
#[tauri::command]
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

// ─── MIG-012 — Index Search Engine: term-level embeddings ─────

use std::sync::atomic::Ordering;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TermEmbedProgress {
    pub processed: u32,
    pub total: u32,
    pub done: bool,
    pub cancelled: bool,
    /// MIG-012-fix-5: which phase the worker is in. Lets the frontend
    /// distinguish "Loading model" (slow on cold disk: ~10–30s for the
    /// ONNX model + tokenizer) from "Scanning vocabulary" (slow on
    /// large libraries: SQLite walks the FTS5 vocab segments and sorts)
    /// from the actual per-term embedding loop. Without this, the UI
    /// shows an indeterminate "Starting…" shimmer for both early
    /// phases and the user can't tell which is the bottleneck.
    /// Values: "loading-model" | "scanning-vocab" | "embedding".
    /// `None` on the per-term progress events for backward compat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TermSimilarity {
    pub term: String,
    pub score: f32,
}

/// Build (or refresh) the term_embeddings table by walking notes_vocab
/// and embedding every term with the same multilingual-e5-small model
/// used for note_embeddings.
///
/// `force` (default false): when false, skip terms already in the
/// table (resumable / incremental). When true, re-embed every term.
///
/// Emits Tauri events `term-embedding-progress` with `TermEmbedProgress`
/// payload every ~50 terms so the frontend can render a live progress
/// bar. The final event has `done: true`.
///
/// Cancellable via `cancel_term_embeddings`. Worker checks the global
/// `TERM_EMBED_CANCEL` flag per-term; on cancel, stops cleanly with a
/// `done: true, cancelled: true` event. Already-embedded terms remain.
///
/// Boss-approved Q2.C: lazy-on-first-semantic-search-toggle-on. The
/// frontend invokes this only when the toggle flips on AND
/// `embedding_status` reports `embedded_term_count == 0`.
#[tauri::command]
pub fn init_term_embeddings(
    app: tauri::AppHandle,
    force: Option<bool>,
) -> Result<(), String> {
    use rusqlite::params;

    let embed_state = app.state::<EmbeddingState>();
    embed_state.term_embed_cancel.store(false, Ordering::SeqCst);

    // MIG-012-fix-5 — phase-1 emit BEFORE ensure_engine so the user sees
    // "Loading model…" immediately. The ONNX model is ~120 MB and on
    // cold disk loads in tens of seconds; without this emit the user
    // would sit in indeterminate-shimmer for that whole window.
    let _ = app.emit(
        "term-embedding-progress",
        TermEmbedProgress {
            processed: 0,
            total: 0,
            done: false,
            cancelled: false,
            phase: Some("loading-model".into()),
        },
    );
    ensure_engine(&app)?;
    let force = force.unwrap_or(false);

    // Pull the term list with a SHORT-lived db lock so we can release
    // it before the long embed loop. Lock-per-iteration below avoids
    // holding SearchState for the full ~10–20 min job, which would
    // freeze every other read-only IPC against search.db.
    // MIG-012-fix-4 — embed-path threshold divergence from browse path.
    // `read_index_entries` (Index panel browse) uses `cnt >= 5` to
    // surface the long-tail vocabulary. The embed path needs a tighter
    // threshold because:
    //   (a) Embedding is ~50 ms/term; on a 7,600-note Universe with
    //       multi-script noise (SQLite LENGTH() counts BYTES, so
    //       1-character Arabic / Hebrew / CJK terms pass `LENGTH >= 2`),
    //       the cnt >= 5 corpus balloons to ~570k → ~8 hours of CPU.
    //   (b) Low-frequency terms make poor semantic anchors anyway —
    //       a 5-occurrence term hasn't accumulated enough context for
    //       the embedding model to capture meaning reliably; semantic
    //       neighbours of such terms are usually noise.
    //
    // `cnt >= 20` is empirically a reasonable threshold — drops the
    // 7,600-note corpus from ~573k to ~30-50k → 30-min rebuild instead
    // of 8 hours. The browse path stays at `cnt >= 5` so the Index
    // panel's term list is unaffected; only the semantic-search corpus
    // changes.
    //
    // Existing rows in `term_embeddings` from prior runs (with the old
    // threshold) are preserved — INSERT OR REPLACE on the embed path
    // doesn't DELETE. Search continues to use them. Future fix can
    // add a Settings knob (`project_term_embeddings_threshold_too_loose.md`)
    // for power users who want a different trade-off.
    // Phase 2 — scanning the FTS5 vocab. On large libraries this walks
    // millions of token-position rows + sorts; can take seconds.
    let _ = app.emit(
        "term-embedding-progress",
        TermEmbedProgress {
            processed: 0,
            total: 0,
            done: false,
            cancelled: false,
            phase: Some("scanning-vocab".into()),
        },
    );
    // MIG-012-fix-6 — open a fresh read-only connection for the vocab
    // scan, mirroring `read_index_entries` exactly. Boss-observed
    // 20-minute hangs on the prior shared-SearchState path; the Index
    // panel's `read_index_entries` runs the same shape SQL in ~350ms
    // via this pattern. Avoids any contention with the shared write
    // connection AND any shared-PRAGMA differences.
    //
    // Also drops ORDER BY — the embed loop doesn't care about term
    // order. fts5vocab walks the FTS5 dictionary in its native order
    // anyway; ORDER BY would force SQLite to materialize + sort.
    let all_terms: Vec<String> = {
        use rusqlite::{Connection, OpenFlags};
        let db_path = crate::search::db_path(&app)?;
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let mut conn = Connection::open_with_flags(&db_path, flags)
            .map_err(|e| format!("Failed to open search.db: {}", e))?;
        conn.busy_timeout(std::time::Duration::from_millis(500))
            .map_err(|e| e.to_string())?;
        crate::search::register_fts5_tokenizer(&mut conn)?;
        let mut stmt = conn
            .prepare("SELECT term FROM notes_vocab WHERE LENGTH(term) >= 2 AND cnt >= 20")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let collected: Vec<String> = rows.filter_map(|r| r.ok()).collect();
        collected
    };
    let total = all_terms.len() as u32;

    let mut processed = 0u32;
    let _ = app.emit(
        "term-embedding-progress",
        TermEmbedProgress { processed: 0, total, done: false, cancelled: false, phase: Some("embedding".into()) },
    );

    for term in &all_terms {
        if embed_state.term_embed_cancel.load(Ordering::SeqCst) {
            let _ = app.emit(
                "term-embedding-progress",
                TermEmbedProgress { processed, total, done: true, cancelled: true, phase: None },
            );
            return Ok(());
        }

        // Per-term: skip-if-existing check with a short-lived lock.
        if !force {
            let existing: bool = {
                let search_state = app.state::<crate::search::SearchState>();
                let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
                let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
                conn.query_row(
                    "SELECT COUNT(*) > 0 FROM term_embeddings WHERE term = ?1",
                    params![term],
                    |row| row.get(0),
                )
                .unwrap_or(false)
            };
            if existing {
                processed += 1;
                continue;
            }
        }

        // Bigram terms (sentinel \x1F) are stored unchanged in
        // notes_vocab; the display form is the space-joined version.
        // Embed the display form so the query embedding (clean text)
        // matches in the same space.
        let display = if term.as_bytes().contains(&crate::fts5_tokenizer::BIGRAM_SEP) {
            term.replace('\u{001F}', " ")
        } else {
            term.clone()
        };
        let prefixed = format!("passage: {}", display);

        // Embed: hold the engine lock ONLY for run_embedding. Other
        // consumers (note-embedding flow, future surfaces) get a clean
        // window between every term. Mutex acquire/release is microseconds
        // vs. tens-of-ms compute per term — negligible overhead.
        let embedding = {
            let engine_guard = embed_state.engine.lock().map_err(|e| e.to_string())?;
            let engine = engine_guard
                .as_ref()
                .ok_or("Embedding engine not initialized")?;
            match run_embedding(engine, &prefixed) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("[term_embeddings] Failed to embed {:?}: {}", term, e);
                    processed += 1;
                    continue;
                }
            }
        };

        // Insert: short-lived db lock per term. Other read-only IPCs
        // can fit between writes.
        {
            let search_state = app.state::<crate::search::SearchState>();
            let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
            let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
            let bytes = vec_to_blob(&embedding);
            let dims = embedding.len() as i32;
            let _ = conn.execute(
                "INSERT OR REPLACE INTO term_embeddings (term, embedding, dimensions, model_id, last_built) VALUES (?1, ?2, ?3, ?4, CAST(strftime('%s','now') AS INTEGER))",
                params![term, bytes, dims, "multilingual-e5-small"],
            );
        }

        processed += 1;
        if processed % 50 == 0 || processed == total {
            let _ = app.emit(
                "term-embedding-progress",
                TermEmbedProgress { processed, total, done: false, cancelled: false, phase: None },
            );
        }
    }

    let _ = app.emit(
        "term-embedding-progress",
        TermEmbedProgress { processed, total, done: true, cancelled: false, phase: None },
    );
    Ok(())
}

/// Cancel an in-flight term-embedding job. Sets the per-app cancel flag
/// on `EmbeddingState`; the worker breaks out at the next per-term check.
/// Idempotent: calling when no job is running is a no-op.
#[tauri::command]
pub fn cancel_term_embeddings(app: tauri::AppHandle) -> Result<(), String> {
    let embed_state = app.state::<EmbeddingState>();
    embed_state.term_embed_cancel.store(true, Ordering::SeqCst);
    Ok(())
}

/// MIG-012 §Build.3 — semantic search over term_embeddings.
///
/// Embed the query with the same model + prefix used at index time
/// (`query: ` for the query side; `passage: ` for the indexed terms),
/// load all term embeddings from SQLite, compute cosine similarity
/// (just a dot product since both vectors are L2-normalized), filter
/// by `threshold` (default 0.7), sort descending, take `top_k` (default
/// 50), return `Vec<TermSimilarity>`.
#[tauri::command]
pub fn search_terms_semantic(
    app: tauri::AppHandle,
    query: String,
    top_k: Option<u32>,
    threshold: Option<f32>,
) -> Result<Vec<TermSimilarity>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let top_k = top_k.unwrap_or(50).min(500) as usize;
    let threshold = threshold.unwrap_or(0.7);

    ensure_engine(&app)?;
    let engine_state = app.state::<EmbeddingState>();
    let engine_guard = engine_state.engine.lock().map_err(|e| e.to_string())?;
    let engine = engine_guard
        .as_ref()
        .ok_or("Embedding engine not initialized")?;

    let prefixed = format!("query: {}", q);
    let query_vec = run_embedding(engine, &prefixed)?;

    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;

    // Pull every term embedding. For 50k terms × 384 dims × 4 bytes
    // = 75 MB; not free but acceptable on the rare semantic query.
    // If this becomes hot, consider an in-memory cache or a Faiss
    // index. Defer until measured.
    let mut stmt = conn
        .prepare("SELECT term, embedding FROM term_embeddings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let term: String = row.get(0)?;
            let bytes: Vec<u8> = row.get(1)?;
            Ok((term, bytes))
        })
        .map_err(|e| e.to_string())?;

    let mut scored: Vec<TermSimilarity> = Vec::new();
    for r in rows.flatten() {
        let (term, bytes) = r;
        // Decode + length check. Both vectors are L2-normalized at write
        // time, so cosine similarity == dot product.
        let Some(term_vec) = blob_to_vec(&bytes) else { continue };
        if term_vec.len() != query_vec.len() {
            continue;
        }
        let score: f32 = query_vec
            .iter()
            .zip(term_vec.iter())
            .map(|(q, t)| q * t)
            .sum();
        if score >= threshold {
            scored.push(TermSimilarity { term, score });
        }
    }
    // Descending similarity, take top_k.
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);
    Ok(scored)
}

/// Status check for the term-embedding pipeline. Used by the frontend
/// to decide whether the toggle should fire `init_term_embeddings`
/// (Q2.C — only when count == 0) or skip it (already populated).
#[tauri::command]
pub fn term_embedding_status(app: tauri::AppHandle) -> Result<u32, String> {
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let count = if let Some(conn) = db_guard.as_ref() {
        conn.query_row("SELECT COUNT(*) FROM term_embeddings", [], |row| row.get::<_, u32>(0))
            .unwrap_or(0)
    } else {
        0
    };
    Ok(count)
}
