//! MIG-013 §1A — Build-time concept-vector baker.
//!
//! Reads M11's seed TSV (`src-tauri/src/lexicon/data/lexicon_v1.tsv`),
//! picks one canonical surface form per concept, embeds all surface
//! forms with multilingual-e5-small, and writes the resulting
//! 20K × 384 f32 matrix + concept-id table to:
//!
//!     `src-tauri/src/bridge_vectors/data/concept_vectors_v1.bin`
//!
//! Run from the `src-tauri/` directory:
//!
//! ```bash
//! cd src-tauri && cargo run --bin build_concept_vectors --release
//! ```
//!
//! ## Surface-form selection
//!
//! Priority order for the canonical lemma (best embedding-space
//! anchoring with multilingual-e5-small): **En > Zh > Es > Fr > De >
//! Ja > Ru > Pt > Ar > Ko > Hi > Tr > Fa > He > Ur**. English first
//! because e5 is dominantly English-trained; the long fallback chain
//! is for region-specific `ProperNoun` concepts that may lack
//! English (rare in M11 v1, but the chain ensures every concept gets
//! a vector).
//!
//! ## Asset layout
//!
//! See `src-tauri/src/bridge_vectors/mod.rs` for the format spec.
//!
//! ## Design notes
//!
//! - **No Tauri runtime**: this is a plain `[[bin]]` target; calls
//!   `constellation_lib::embeddings::embed_passages_standalone`
//!   which builds its own ONNX session without an `AppHandle`.
//! - **M11 zero-touch**: this binary only *reads* the lexicon TSV
//!   and the public `lexicon::parse` API. `git diff
//!   src-tauri/src/lexicon/` returns empty after this commit.
//! - **Reproducibility**: `BTreeMap<Lang, Vec<String>>` in
//!   `ConceptRecord` is iteration-stable; same TSV → byte-identical
//!   output (modulo ONNX runtime determinism — which e5 + opset 14
//!   provides on CPU).

use constellation_lib::arabic::Lang;
use constellation_lib::bridge_vectors::{ASSET_MAGIC, VECTOR_DIM};
use constellation_lib::embeddings::embed_passages_standalone;
use constellation_lib::lexicon::ConceptRecord;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

/// Lemma-language priority for canonical surface-form selection.
/// Front-loaded with high-resource languages where e5-small produces
/// the cleanest embeddings.
const LANG_PRIORITY: &[Lang] = &[
    Lang::En, Lang::Zh, Lang::Es, Lang::Fr, Lang::De, Lang::Ja,
    Lang::Ru, Lang::Pt, Lang::Ar, Lang::Ko, Lang::Hi, Lang::Tr,
    Lang::Fa, Lang::He, Lang::Ur,
];

/// Embedding batch size. 128 amortizes the e5-small ONNX fixed
/// overhead well on a 384-dim model while keeping peak tensor
/// memory under ~32 MB.
const BATCH_SIZE: usize = 128;

fn main() -> Result<(), String> {
    let t0 = Instant::now();
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    println!("[build_concept_vectors] cwd = {}", cwd.display());

    // ─── 1. Read M11 TSV ─────────────────────────────────────────
    let tsv_path: PathBuf = ["src", "lexicon", "data", "lexicon_v1.tsv"].iter().collect();
    if !tsv_path.exists() {
        return Err(format!(
            "TSV not found at {}. Run from src-tauri/.", tsv_path.display()
        ));
    }
    let tsv = std::fs::read_to_string(&tsv_path)
        .map_err(|e| format!("Failed to read {}: {e}", tsv_path.display()))?;
    let records: Vec<ConceptRecord> = constellation_lib::lexicon::parse(&tsv);
    println!(
        "[build_concept_vectors] parsed {} concepts from {} ({} bytes) in {:.2}s",
        records.len(),
        tsv_path.display(),
        tsv.len(),
        t0.elapsed().as_secs_f32()
    );
    if records.is_empty() {
        return Err("No concepts parsed — TSV malformed?".into());
    }

    // ─── 2. Pick canonical surface form per concept ──────────────
    let t1 = Instant::now();
    let mut surface_forms: Vec<(String, String)> = Vec::with_capacity(records.len());
    let mut by_lang: [u32; 15] = [0; 15];
    let mut skipped_no_lemma: u32 = 0;
    for rec in &records {
        let canonical = pick_canonical_lemma(rec);
        match canonical {
            Some((lang, lemma)) => {
                by_lang[lang_idx(lang)] += 1;
                surface_forms.push((rec.id.clone(), format!("passage: {lemma}")));
            }
            None => {
                eprintln!(
                    "[build_concept_vectors] WARN concept {} has no usable lemma — skipping",
                    rec.id
                );
                skipped_no_lemma += 1;
            }
        }
    }
    println!(
        "[build_concept_vectors] selected {} surface forms (skipped {}) in {:.2}s",
        surface_forms.len(),
        skipped_no_lemma,
        t1.elapsed().as_secs_f32()
    );
    println!(
        "[build_concept_vectors] coverage by language: {}",
        LANG_PRIORITY
            .iter()
            .map(|l| format!("{l:?}={}", by_lang[lang_idx(*l)]))
            .collect::<Vec<_>>()
            .join(" ")
    );

    // ─── 3. Resolve model paths ──────────────────────────────────
    let model_path: PathBuf = ["models", "model.onnx"].iter().collect();
    let tokenizer_path: PathBuf = ["models", "tokenizer.json"].iter().collect();
    if !model_path.exists() || !tokenizer_path.exists() {
        return Err(format!(
            "Model files not found at {} / {}. \
             Run from src-tauri/ and ensure models/ contains model.onnx + tokenizer.json.",
            model_path.display(), tokenizer_path.display()
        ));
    }

    // ─── 4. Embed all surface forms ──────────────────────────────
    let texts: Vec<String> = surface_forms.iter().map(|(_, t)| t.clone()).collect();
    let t2 = Instant::now();
    let intra_threads = std::thread::available_parallelism()
        .map(|n| n.get().max(2))
        .unwrap_or(4);
    println!(
        "[build_concept_vectors] embedding {} passages, batch={}, intra_threads={} ...",
        texts.len(), BATCH_SIZE, intra_threads
    );
    let vectors = embed_passages_standalone(
        &model_path, &tokenizer_path, &texts, intra_threads, BATCH_SIZE,
    )?;
    let elapsed = t2.elapsed().as_secs_f32();
    let rate = texts.len() as f32 / elapsed.max(0.001);
    println!(
        "[build_concept_vectors] embedded {} passages in {:.1}s ({:.1} passages/sec)",
        vectors.len(), elapsed, rate
    );
    if vectors.len() != surface_forms.len() {
        return Err(format!(
            "Embedding count mismatch: expected {}, got {}",
            surface_forms.len(), vectors.len()
        ));
    }

    // ─── 5. Validate vectors ─────────────────────────────────────
    for (i, v) in vectors.iter().enumerate() {
        if v.len() != VECTOR_DIM {
            return Err(format!(
                "Vector {} has wrong dim {} (expected {})",
                i, v.len(), VECTOR_DIM
            ));
        }
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if !(0.99..=1.01).contains(&norm) {
            return Err(format!(
                "Vector {} (concept {}) has norm {:.4} (expected ~1.0). \
                 L2 normalization is non-negotiable for cosine k-NN.",
                i, surface_forms[i].0, norm
            ));
        }
    }
    println!("[build_concept_vectors] all {} vectors are 384-dim and L2-normalized", vectors.len());

    // ─── 6. Write asset ──────────────────────────────────────────
    let out_path: PathBuf = ["src", "bridge_vectors", "data", "concept_vectors_v1.bin"]
        .iter().collect();
    write_asset(&out_path, &surface_forms, &vectors)?;
    let bytes = std::fs::metadata(&out_path)
        .map_err(|e| e.to_string())?
        .len();
    println!(
        "[build_concept_vectors] wrote {} ({} bytes / {:.1} MB)",
        out_path.display(),
        bytes,
        bytes as f32 / 1024.0 / 1024.0
    );
    println!("[build_concept_vectors] total elapsed: {:.1}s", t0.elapsed().as_secs_f32());
    Ok(())
}

/// Pick the canonical surface form using the priority chain. Returns
/// the first lemma found in the highest-priority language present.
fn pick_canonical_lemma(rec: &ConceptRecord) -> Option<(Lang, &str)> {
    for &lang in LANG_PRIORITY {
        if let Some(lemmas) = rec.labels.get(&lang) {
            if let Some(first) = lemmas.iter().find(|l| !l.trim().is_empty()) {
                return Some((lang, first.as_str()));
            }
        }
    }
    None
}

/// Map a `Lang` variant to its priority-list index (0-based). Used
/// for the per-language coverage histogram in the build log.
fn lang_idx(lang: Lang) -> usize {
    LANG_PRIORITY.iter().position(|l| *l == lang).unwrap_or(0)
}

/// Serialize header + concept-id table + vector matrix to disk.
/// Layout is documented in `src-tauri/src/bridge_vectors/mod.rs`.
fn write_asset(
    path: &std::path::Path,
    surface_forms: &[(String, String)],
    vectors: &[Vec<f32>],
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let f = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut w = std::io::BufWriter::new(f);

    // Header
    w.write_all(ASSET_MAGIC).map_err(|e| e.to_string())?;
    let count = surface_forms.len() as u32;
    let dim = VECTOR_DIM as u32;
    w.write_all(&count.to_le_bytes()).map_err(|e| e.to_string())?;
    w.write_all(&dim.to_le_bytes()).map_err(|e| e.to_string())?;

    // Concept-id table: u16 LE byte_len + UTF-8 bytes per row
    let mut table_bytes: usize = 0;
    for (id, _) in surface_forms {
        let id_bytes = id.as_bytes();
        if id_bytes.len() > u16::MAX as usize {
            return Err(format!("concept id too long ({} bytes): {}", id_bytes.len(), id));
        }
        let len = id_bytes.len() as u16;
        w.write_all(&len.to_le_bytes()).map_err(|e| e.to_string())?;
        w.write_all(id_bytes).map_err(|e| e.to_string())?;
        table_bytes += 2 + id_bytes.len();
    }

    // Pad to 4-byte boundary so the f32 matrix is mmap-aligned
    let header_bytes = 8 + 4 + 4; // magic + count + dim
    let pre_matrix = header_bytes + table_bytes;
    let pad = (4 - (pre_matrix % 4)) % 4;
    if pad > 0 {
        w.write_all(&vec![0u8; pad]).map_err(|e| e.to_string())?;
    }

    // Vector matrix: count × dim × f32 LE, row-major
    for v in vectors {
        for x in v {
            w.write_all(&x.to_le_bytes()).map_err(|e| e.to_string())?;
        }
    }
    w.flush().map_err(|e| e.to_string())?;
    Ok(())
}
