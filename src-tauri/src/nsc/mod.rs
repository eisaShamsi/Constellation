//! Note Summary Creator (NSC) — extractive, embedding-based TextRank.
//!
//! See `docs/Constellation-NSC-Concept-Paper-v1.0.md`. Summary precedence
//! (NSC only GENERATES when the author has NOT written one):
//!   1. author frontmatter summary (`summary`/`description`/`abstract`/`excerpt`)
//!   2. author `> [!summary]` / `[!abstract]` / `[!tldr]` callout in the body
//!   3. generated extractive summary:
//!        UAX#29 sentence split (+ fallback) → e5-small sentence embeddings
//!        → weighted PageRank (TextRank) → top-k sentences in document order.
//!
//! NSC is READ-ONLY on notes: it never writes a generated summary back
//! into the note file (File-Over-App). The proven-standard method (Eisa's
//! constraint, 2026-05-20): TextRank/PageRank over an embedding-cosine
//! sentence graph (Mihalcea & Tarau 2004) + Unicode UAX#29 segmentation.
//!
//! note_meta read vs. raw-file read: the cheap path (frontmatter + extractive)
//! uses the already-indexed `note_meta.body_text`. Detecting an author summary
//! *callout* needs the raw file — `body_text` is markdown-stripped AND Arabic-
//! normalized (tashkeel/tatweel removed), so it would lose the author's exact
//! wording. That one extra read happens ONLY on a cache miss for a note with no
//! frontmatter summary (notes with a frontmatter summary never read the file).

use std::hash::{Hash, Hasher};
use tauri::Manager;
use unicode_segmentation::UnicodeSegmentation;

/// How a summary was produced (mirrored to the frontend as `summary_source`).
pub const SOURCE_FRONTMATTER: &str = "frontmatter";
pub const SOURCE_CALLOUT: &str = "callout";
pub const SOURCE_EXTRACTIVE: &str = "extractive";
pub const SOURCE_OPENING: &str = "opening";

/// Sentences in a generated summary (top-k by TextRank centrality).
const TARGET_SENTENCES: usize = 3;
/// Drop sentence fragments shorter than this (chars) before ranking.
const MIN_SENTENCE_CHARS: usize = 16;
/// Opening-text fallback length (chars) for punctuation-less scripts.
const OPENING_CHARS: usize = 280;
/// MIG-040 crash-fix: bound per-note work. A very long note can otherwise be
/// split into hundreds of sentences and embedded in one batch, which can
/// exhaust / abort the ONNX runtime and crash the app. Cap both the body
/// length scanned and the number of sentences embedded/ranked.
const MAX_RANK_SENTENCES: usize = 40;
const MAX_BODY_CHARS: usize = 50_000;

/// Frontmatter keys that count as an author-written summary, priority order.
const FRONTMATTER_SUMMARY_KEYS: [&str; 4] = ["summary", "description", "abstract", "excerpt"];

/// Callout types that render with the 📋 summary icon (see
/// `src/lib/editor/calloutPlugin.ts` CALLOUT_ICONS). A body callout of one of
/// these types counts as an author-written summary.
const SUMMARY_CALLOUT_TYPES: [&str; 3] = ["summary", "abstract", "tldr"];

/// The result of summarizing a note.
#[derive(Debug, Clone)]
pub struct NoteSummary {
    pub summary: String,
    /// One of SOURCE_FRONTMATTER / SOURCE_EXTRACTIVE / SOURCE_OPENING.
    pub source: String,
}

// ─── Public entry point ────────────────────────────────────────────────

/// Get the summary for a note: the author's frontmatter summary if present,
/// else a generated extractive summary of the body. Reads `note_meta`
/// (`body_text` + `properties_json`) — no file I/O, no note-file writes.
pub fn compute_summary_for_note(app: &tauri::AppHandle, note_path: &str) -> Result<NoteSummary, String> {
    let (body_text, properties_json) = read_note_meta(app, note_path)?;
    summarize_from_parts(app, note_path, &body_text, properties_json.as_deref())
}

/// Summarize honoring author authorship first, generating only as a fallback:
///   1. frontmatter summary field (verbatim, from `properties_json`)
///   2. `> [!summary]`/`[!abstract]`/`[!tldr]` callout in the body (verbatim,
///      read from the raw file — see the callout note below)
///   3. generated extractive summary of the body
fn summarize_from_parts(
    app: &tauri::AppHandle,
    note_path: &str,
    body_text: &str,
    properties_json: Option<&str>,
) -> Result<NoteSummary, String> {
    // 1. Author's frontmatter summary.
    if let Some(fm) = frontmatter_summary(properties_json) {
        return Ok(NoteSummary { summary: fm, source: SOURCE_FRONTMATTER.to_string() });
    }
    // 2. Author's summary callout in the body. `body_text` is markdown-stripped
    //    AND Arabic-normalized, so we read the raw file to recover the callout
    //    and the author's exact wording (diacritics preserved). One read, only
    //    here — when the note has no frontmatter summary on a cache miss.
    if let Some(callout) = file_callout_summary(note_path) {
        return Ok(NoteSummary { summary: callout, source: SOURCE_CALLOUT.to_string() });
    }
    // 3. Generated extractive summary.
    summarize_body(app, body_text)
}

// ─── Frontmatter precedence ─────────────────────────────────────────────

/// Return the first non-empty author summary field from a `properties_json`
/// blob (the frontmatter dict `note_meta` already stores), or None.
pub(crate) fn frontmatter_summary(properties_json: Option<&str>) -> Option<String> {
    let json = properties_json?;
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let obj = value.as_object()?;
    for key in FRONTMATTER_SUMMARY_KEYS {
        if let Some(s) = obj.get(key).and_then(|v| v.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

// ─── Author summary callout (read from the raw file) ────────────────────

/// Read the raw note file and return the first summary-family callout's
/// verbatim body, or None. Best-effort: any read error → None (the caller
/// then falls back to extractive). Strips the YAML frontmatter so a `summary:`
/// frontmatter line is never mistaken for a callout.
fn file_callout_summary(note_path: &str) -> Option<String> {
    let content = std::fs::read_to_string(note_path).ok()?;
    let body = crate::strata::strip_frontmatter_pub(&content);
    body_callout_summary(body)
}

/// Find the first author-written summary callout in a note body and return its
/// verbatim text. Mirrors `src/lib/editor/calloutPlugin.ts`: a callout block is
/// a run of contiguous `>`-prefixed lines whose first line is `> [!type]`. We
/// take the body lines (everything after the title line) with the `>` prefix
/// removed; a single-line callout with no body falls back to its inline title
/// text. Returns None when the body has no summary-family callout.
pub(crate) fn body_callout_summary(body: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if let Some((ctype, inline)) = parse_callout_header(lines[i]) {
            if SUMMARY_CALLOUT_TYPES.contains(&ctype.as_str()) {
                // Collect contiguous body lines: `>`-prefixed, not a new header.
                let mut parts: Vec<String> = Vec::new();
                let mut j = i + 1;
                while j < lines.len() {
                    if !is_quote_line(lines[j]) || parse_callout_header(lines[j]).is_some() {
                        break;
                    }
                    let stripped = strip_quote_prefix(lines[j]);
                    if !stripped.trim().is_empty() {
                        parts.push(stripped);
                    }
                    j += 1;
                }
                let summary = collapse_ws(&parts.join(" "));
                if !summary.is_empty() {
                    return Some(summary);
                }
                // No body lines — single-line callout: use its inline title.
                let inline = collapse_ws(&inline);
                if !inline.is_empty() {
                    return Some(inline);
                }
                // Degenerate empty callout — skip past it, keep scanning.
                i = j;
                continue;
            }
        }
        i += 1;
    }
    None
}

/// Parse a callout header `> [!type]<+|->? <title>`. Returns (lowercased type,
/// inline title text after the marker) or None. Mirrors calloutPlugin.ts's
/// `/^>\s*\[!(\w+)\]([+-])?\s*/`, additionally tolerating leading spaces (the
/// Markdown blockquote allowance).
fn parse_callout_header(line: &str) -> Option<(String, String)> {
    let rest = line.trim_start_matches(' ').strip_prefix('>')?.trim_start();
    let rest = rest.strip_prefix("[!")?;
    let close = rest.find(']')?;
    let ctype = &rest[..close];
    if ctype.is_empty() || !ctype.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let after = &rest[close + 1..];
    let after = after.strip_prefix(['+', '-']).unwrap_or(after);
    Some((ctype.to_lowercase(), after.trim_start().to_string()))
}

/// True if the line is a blockquote line (`>` optional space). Mirrors the
/// `/^>\s?/` continuation test (same leading-space tolerance).
fn is_quote_line(line: &str) -> bool {
    line.trim_start_matches(' ').starts_with('>')
}

/// Remove the leading `>` and one optional following space from a body line.
fn strip_quote_prefix(line: &str) -> String {
    let s = line.trim_start_matches(' ');
    let s = s.strip_prefix('>').unwrap_or(s);
    s.strip_prefix(' ').unwrap_or(s).to_string()
}

/// Collapse internal whitespace runs to single spaces and trim the ends.
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ─── Extractive summarization ───────────────────────────────────────────

fn summarize_body(app: &tauri::AppHandle, body_text: &str) -> Result<NoteSummary, String> {
    // Bound the input first (MIG-040 crash-fix): truncate a very long body,
    // then split into sentences.
    let bounded = bound_body(body_text);
    let all = split_sentences(bounded);

    // Too few sentences to rank: return the opening text / joined verbatim.
    if all.len() <= TARGET_SENTENCES {
        if all.is_empty() {
            let open = opening_text(bounded);
            let source = if open.is_empty() { SOURCE_EXTRACTIVE } else { SOURCE_OPENING };
            return Ok(NoteSummary { summary: open, source: source.to_string() });
        }
        return Ok(NoteSummary { summary: all.join(" "), source: SOURCE_EXTRACTIVE.to_string() });
    }

    // Cap how many sentences we embed/rank — downsample evenly so the summary
    // still covers the whole note — so a long note can't build a huge batch.
    let sentences: Vec<String> = if all.len() > MAX_RANK_SENTENCES {
        downsample(&all, MAX_RANK_SENTENCES)
    } else {
        all
    };

    // Embed each sentence with the shared e5-small engine ("query: " prefix
    // for symmetric sentence-to-sentence similarity per the e5 model card).
    crate::embeddings::ensure_engine(app)?;
    let embeddings = {
        let state = app.state::<crate::embeddings::EmbeddingState>();
        let guard = state.engine.lock().map_err(|e| e.to_string())?;
        let engine = guard.as_ref().ok_or("Embedding engine not initialized")?;
        let prefixed: Vec<String> = sentences.iter().map(|s| format!("query: {}", s)).collect();
        crate::embeddings::run_embedding_batch(engine, &prefixed)?
    };

    let top = textrank_top_k(&embeddings, TARGET_SENTENCES);
    let summary = top.iter().map(|&i| sentences[i].as_str()).collect::<Vec<_>>().join(" ");
    Ok(NoteSummary { summary, source: SOURCE_EXTRACTIVE.to_string() })
}

/// Split text into sentences using the Unicode UAX#29 standard, with a
/// paragraph fallback for scripts that lack sentence punctuation (Thai,
/// Lao). Returns trimmed, non-trivial sentences in document order. If even
/// the fallback yields a single block, returns that one block (the caller
/// then uses opening-text).
pub(crate) fn split_sentences(text: &str) -> Vec<String> {
    // Primary: UAX#29 sentence boundaries (handles CJK / Arabic / Indic).
    let all: Vec<String> = text
        .split_sentence_bounds()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    // Prefer substantial sentences; relax if filtering leaves too few.
    let substantial: Vec<String> = all
        .iter()
        .filter(|s| s.chars().count() >= MIN_SENTENCE_CHARS)
        .cloned()
        .collect();
    let primary = if substantial.len() >= 2 { substantial } else { all };
    if primary.len() >= 2 {
        return primary;
    }

    // Fallback: paragraph / line breaks (punctuation-less scripts, or text
    // that is one long block to UAX#29).
    let paras: Vec<String> = text
        .split(['\n', '\r'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if paras.len() >= 2 {
        return paras;
    }

    primary
}

/// TextRank: rank sentences by weighted-PageRank centrality over a cosine-
/// similarity graph, return the indices of the top-k sentences IN ORIGINAL
/// DOCUMENT ORDER. `embeddings[i]` must be L2-normalized (so cosine = dot),
/// which `run_embedding_batch` guarantees.
pub(crate) fn textrank_top_k(embeddings: &[Vec<f32>], k: usize) -> Vec<usize> {
    let n = embeddings.len();
    if n == 0 {
        return Vec::new();
    }
    if n <= k {
        return (0..n).collect();
    }

    // Weighted adjacency = clamped cosine similarity, no self-loops.
    let mut weight = vec![vec![0f32; n]; n];
    let mut row_sum = vec![0f32; n];
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let sim = cosine(&embeddings[i], &embeddings[j]).max(0.0);
            weight[i][j] = sim;
            row_sum[i] += sim;
        }
    }

    // Power-iteration PageRank, damping d = 0.85 (TextRank, Mihalcea & Tarau).
    let d = 0.85f32;
    let base = (1.0 - d) / n as f32;
    let mut score = vec![1.0f32 / n as f32; n];
    for _ in 0..100 {
        let mut next = vec![base; n];
        for i in 0..n {
            if row_sum[i] > 0.0 {
                let share = d * score[i];
                for j in 0..n {
                    if i != j {
                        next[j] += share * (weight[i][j] / row_sum[i]);
                    }
                }
            } else {
                // Dangling node: distribute its mass uniformly.
                let share = d * score[i] / (n as f32 - 1.0);
                for j in 0..n {
                    if i != j {
                        next[j] += share;
                    }
                }
            }
        }
        let delta: f32 = score.iter().zip(&next).map(|(a, b)| (a - b).abs()).sum();
        score = next;
        if delta < 1e-6 {
            break;
        }
    }

    // Top-k by score, then restore original document order.
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| {
        score[b]
            .partial_cmp(&score[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut top: Vec<usize> = idx.into_iter().take(k).collect();
    top.sort_unstable();
    top
}

/// Cosine similarity of two L2-normalized vectors (= dot product).
fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// First ~OPENING_CHARS of the body (char-boundary safe), with an ellipsis
/// if truncated. The last-resort fallback for punctuation-less scripts.
fn opening_text(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.chars().count() <= OPENING_CHARS {
        return trimmed.to_string();
    }
    let mut end = trimmed
        .char_indices()
        .nth(OPENING_CHARS)
        .map(|(i, _)| i)
        .unwrap_or(trimmed.len());
    while end > 0 && !trimmed.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", trimmed[..end].trim_end())
}

/// Truncate the body (char-boundary safe) before sentence-splitting, so a
/// pathologically large note can't blow up the split + embedding work.
fn bound_body(body: &str) -> &str {
    if body.len() <= MAX_BODY_CHARS {
        return body;
    }
    let mut end = MAX_BODY_CHARS;
    while end > 0 && !body.is_char_boundary(end) {
        end -= 1;
    }
    &body[..end]
}

/// Evenly downsample `sentences` to at most `max`, preserving document order.
/// Keeps whole-note coverage while bounding the embedding batch + TextRank
/// matrix size.
fn downsample(sentences: &[String], max: usize) -> Vec<String> {
    let n = sentences.len();
    if n <= max || max == 0 {
        return sentences.to_vec();
    }
    (0..max).map(|i| sentences[i * n / max].clone()).collect()
}

// ─── note_meta read ─────────────────────────────────────────────────────

fn read_note_meta(app: &tauri::AppHandle, note_path: &str) -> Result<(String, Option<String>), String> {
    let search_state = app.state::<crate::search::SearchState>();
    let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
    let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
    conn.query_row(
        "SELECT body_text, properties_json FROM note_meta WHERE path = ?1",
        rusqlite::params![note_path],
        |r| {
            Ok((
                r.get::<_, Option<String>>(0)?.unwrap_or_default(),
                r.get::<_, Option<String>>(1)?,
            ))
        },
    )
    .map_err(|e| format!("note_meta read failed for {}: {}", note_path, e))
}

// ─── Summary cache + batched delivery (Rule 8 / no per-card IPC) ────────

/// One cached/computed summary for a note (serialized to the frontend).
#[derive(Debug, Clone, serde::Serialize)]
pub struct NoteSummaryEntry {
    pub path: String,
    pub summary: String,
    pub source: String,
}

/// Idempotent: create the `note_summaries` cache table. Called from
/// `search::init_db`. Keyed by note path; `content_hash` (over body_text)
/// drives invalidation when the note changes. Cascade-deletes with the note.
pub fn ensure_note_summaries_table(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS note_summaries (
            path TEXT PRIMARY KEY,
            summary TEXT NOT NULL,
            source TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (path) REFERENCES note_meta(path) ON DELETE CASCADE
        );",
    )?;
    Ok(())
}

/// Bump when the summarization ALGORITHM changes so cached summaries recompute.
/// The cached `content_hash` embeds this version, so a bump makes every stored
/// hash stale on next access (self-healing — no explicit cache wipe needed).
///   v2 (2026-05-20): author `> [!summary]` callouts now take precedence over a
///                    generated extractive summary (previously only frontmatter
///                    did) — every pre-v2 cached summary must recompute.
const NSC_ALGO_VERSION: &str = "v2";

/// Fast non-cryptographic hash of the body, for cache invalidation. Prefixed
/// with `NSC_ALGO_VERSION` so an algorithm change invalidates the whole cache.
fn body_hash(body: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    body.hash(&mut h);
    format!("{}:{:016x}", NSC_ALGO_VERSION, h.finish())
}

/// Batch get-or-compute summaries for a set of notes. Cache-first (keyed by
/// path + body content_hash); computes + caches misses; returns one entry per
/// note that has a non-empty summary. `#[command(async)]` so the (possibly
/// embedding-heavy) miss path runs off the UI thread (LL-021/LL-022). The
/// frontend calls this once with the visible cards' paths — zero per-card IPC.
#[tauri::command(async)]
pub fn nsc_get_summaries_for_notes(
    app: tauri::AppHandle,
    note_paths: Vec<String>,
) -> Result<Vec<NoteSummaryEntry>, String> {
    crate::search::ensure_search_db_ready(&app)?;
    let mut out = Vec::with_capacity(note_paths.len());
    for path in &note_paths {
        match get_or_compute_cached(&app, path) {
            Ok(Some(entry)) => out.push(entry),
            Ok(None) => {}
            Err(e) => eprintln!("[NSC] summary failed for {}: {}", path, e),
        }
    }
    Ok(out)
}

/// Single-note get-or-compute (cache-first). For on-demand refresh.
#[tauri::command(async)]
pub fn nsc_get_summary(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<Option<NoteSummaryEntry>, String> {
    crate::search::ensure_search_db_ready(&app)?;
    get_or_compute_cached(&app, &note_path)
}

/// Cache-first get-or-compute for one note. Reads note_meta once, checks the
/// cache against the current body hash, computes + caches on miss/stale.
fn get_or_compute_cached(
    app: &tauri::AppHandle,
    note_path: &str,
) -> Result<Option<NoteSummaryEntry>, String> {
    let (body_text, properties_json) = read_note_meta(app, note_path)?;
    let hash = body_hash(&body_text);

    // Cache hit (fresh)?
    {
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
        let cached: Option<(String, String, String)> = conn
            .query_row(
                "SELECT summary, source, content_hash FROM note_summaries WHERE path = ?1",
                rusqlite::params![note_path],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .ok();
        if let Some((summary, source, ch)) = cached {
            if ch == hash {
                return Ok(Some(NoteSummaryEntry {
                    path: note_path.to_string(),
                    summary,
                    source,
                }));
            }
        }
    }

    // Miss/stale: compute (frontmatter → callout → extractive). The DB lock
    // above is released; summarize_body locks the embedding engine (separate
    // mutex) — no nested same-lock, no deadlock.
    let result = summarize_from_parts(app, note_path, &body_text, properties_json.as_deref())?;
    if result.summary.trim().is_empty() {
        return Ok(None);
    }

    // Cache it (best-effort).
    {
        let now = chrono::Utc::now().timestamp();
        let search_state = app.state::<crate::search::SearchState>();
        let db_guard = search_state.db.lock().map_err(|e| e.to_string())?;
        let conn = db_guard.as_ref().ok_or("Search database not initialized")?;
        let _ = conn.execute(
            "INSERT OR REPLACE INTO note_summaries (path, summary, source, content_hash, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![note_path, result.summary, result.source, hash, now],
        );
    }

    Ok(Some(NoteSummaryEntry {
        path: note_path.to_string(),
        summary: result.summary,
        source: result.source,
    }))
}

// ─── Tests (pure functions — no ONNX engine needed) ─────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_english() {
        let s = split_sentences("The cat sat on the mat. The dog ran across the yard! Did the bird fly away?");
        assert!(s.len() >= 3, "english: got {:?}", s);
    }

    #[test]
    fn splits_arabic() {
        // Arabic full stop + Arabic question mark (؟).
        let s = split_sentences("هذا اختبار للغة العربية الجميلة. هل يعمل التقسيم بشكل صحيح؟ نعم إنه يعمل تماما.");
        assert!(s.len() >= 2, "arabic: got {:?}", s);
    }

    #[test]
    fn splits_chinese() {
        // CJK full-width period 。 and question mark ？
        let s = split_sentences("这是一个中文测试句子的例子。它能够正确地分割吗？是的它可以工作。");
        assert!(s.len() >= 2, "chinese: got {:?}", s);
    }

    #[test]
    fn splits_hindi() {
        // Devanagari danda (।) sentence terminator.
        let s = split_sentences("यह एक हिंदी परीक्षण वाक्य है। क्या यह सही ढंग से काम करता है? हाँ यह काम करता है।");
        assert!(s.len() >= 2, "hindi: got {:?}", s);
    }

    #[test]
    fn thai_no_punctuation_does_not_panic() {
        // Thai has no sentence punctuation; one block is acceptable (the
        // caller falls back to opening-text). Must not panic.
        let s = split_sentences("นี่คือประโยคภาษาไทยที่ไม่มีเครื่องหมายวรรคตอนเลยและยาวพอสมควร");
        assert!(!s.is_empty(), "thai: got {:?}", s);
    }

    #[test]
    fn textrank_returns_all_when_few() {
        let emb = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        assert_eq!(textrank_top_k(&emb, 3), vec![0, 1]);
    }

    #[test]
    fn textrank_drops_the_outlier() {
        // 0,1,2 form a tight cluster (mutually identical); 3 is an orthogonal
        // outlier connected to nothing. The three central sentences far
        // outscore the outlier (~0.32 vs ~0.04 per node), so top-3 = the
        // cluster, returned in original document order. (Two equal-size
        // cliques would TIE under PageRank — hence cluster-vs-outlier, not
        // cluster-vs-cluster, makes the assertion deterministic.)
        let emb = vec![
            vec![1.0, 0.0],
            vec![1.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];
        assert_eq!(textrank_top_k(&emb, 3), vec![0, 1, 2]);
    }

    #[test]
    fn frontmatter_summary_precedence() {
        assert_eq!(
            frontmatter_summary(Some(r#"{"summary":"Author wrote this.","description":"ignored"}"#)),
            Some("Author wrote this.".to_string())
        );
        // No summary key → description is next in priority.
        assert_eq!(
            frontmatter_summary(Some(r#"{"title":"x","description":"Use me."}"#)),
            Some("Use me.".to_string())
        );
        assert_eq!(frontmatter_summary(Some(r#"{"title":"x"}"#)), None);
        assert_eq!(frontmatter_summary(Some(r#"{"summary":"   "}"#)), None);
        assert_eq!(frontmatter_summary(None), None);
        assert_eq!(frontmatter_summary(Some("not json")), None);
    }

    #[test]
    fn callout_summary_basic() {
        // The image-1 case: `> [!summary]` (no inline title) + body lines.
        let body = "# Title\n\n> [!summary]\n> First sentence of the summary.\n> Second sentence too.\n\nMore body.";
        assert_eq!(
            body_callout_summary(body),
            Some("First sentence of the summary. Second sentence too.".to_string())
        );
    }

    #[test]
    fn callout_summary_arabic_verbatim_with_title() {
        // Arabic callout with an explicit title — title is a label, body wins.
        // Diacritics in the body must be preserved verbatim.
        let body = "> [!summary] ملخّص\n> الهَرَم الأكبر هو أكبر أهرام مصر.";
        assert_eq!(
            body_callout_summary(body),
            Some("الهَرَم الأكبر هو أكبر أهرام مصر.".to_string())
        );
    }

    #[test]
    fn callout_summary_aliases_and_fold_marker() {
        assert_eq!(
            body_callout_summary("> [!abstract]\n> Abstract text."),
            Some("Abstract text.".to_string())
        );
        assert_eq!(
            body_callout_summary("> [!tldr]-\n> Folded but present."),
            Some("Folded but present.".to_string())
        );
    }

    #[test]
    fn callout_summary_single_line_inline() {
        // No body lines → use the inline title text.
        assert_eq!(
            body_callout_summary("> [!summary] The whole summary on one line"),
            Some("The whole summary on one line".to_string())
        );
    }

    #[test]
    fn callout_summary_skips_non_summary_then_finds_summary() {
        let body = "> [!note]\n> Just a note.\n\n> [!summary]\n> The real summary.";
        assert_eq!(
            body_callout_summary(body),
            Some("The real summary.".to_string())
        );
    }

    #[test]
    fn callout_summary_none_when_absent() {
        assert_eq!(body_callout_summary("> [!note]\n> Only a note callout."), None);
        assert_eq!(body_callout_summary("Plain body, no callouts at all."), None);
        // Empty summary callout (no body, no inline) → None (falls to extractive).
        assert_eq!(body_callout_summary("> [!summary]\n\nUnquoted paragraph."), None);
    }

    #[test]
    fn opening_text_truncates_on_char_boundary() {
        let long = "a".repeat(500);
        let open = opening_text(&long);
        assert!(open.ends_with('…'));
        assert!(open.chars().count() <= OPENING_CHARS + 1);
    }

    #[test]
    fn downsample_bounds_count_and_preserves_order() {
        let s: Vec<String> = (0..100).map(|i| format!("{:03}", i)).collect();
        let d = downsample(&s, 40);
        assert_eq!(d.len(), 40);
        assert_eq!(d[0], "000");
        // Zero-padded so string order == numeric order: must be non-decreasing.
        for w in d.windows(2) {
            assert!(w[0] <= w[1]);
        }
        // No-op when already within the cap.
        let small = vec!["a".to_string(), "b".to_string()];
        assert_eq!(downsample(&small, 40), small);
    }
}
