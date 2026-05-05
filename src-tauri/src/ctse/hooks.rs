//! CTSE write-time hooks (MIG-013 §1C).
//!
//! Maintains `term_vocab` incrementally as notes are saved or deleted.
//! Each save delta is computed from the old vs. new body_text using the
//! same FTS5 tokenizer that `notes_fts` uses (`fts5_tokenizer::tokenize_to_vec`),
//! so `term_vocab.term` stays byte-identical to the FTS5 token namespace.
//!
//! ## Design — Write-Time Derivation (CLAUDE.md Rule 8)
//!
//! `term_vocab` is a derived view of `note_meta.body_text`. The single
//! source of truth is the file on disk → its row in `note_meta`. Every
//! save updates `term_vocab` in lockstep with `note_meta` via this
//! module's hooks. There is no `populate_*` / `rebuild_*` command and
//! no boot-time re-walk of the corpus — the index is always current.
//!
//! ## Why both `total_count` AND `doc_count` are maintained
//!
//! - `total_count` — total occurrences across the corpus. Used by the
//!   backfill ordering (TF-IDF descending = rarest terms first) and by
//!   the Index panel for "popular terms" lists.
//! - `doc_count` — number of distinct notes containing the term. The
//!   IDF half of TF-IDF and a useful cardinality signal in its own right.
//!
//! Both are kept exact via per-save delta computation: the old body's
//! token-count map is differenced against the new body's, yielding a
//! signed delta per term.
//!
//! ## Fast-path concept resolution
//!
//! For each term that's NEW to `term_vocab` (didn't exist before this
//! save), we attempt an M11 fast-path lookup across all 15 supported
//! languages. On hit, `bridge_concept_id` is populated immediately
//! (microseconds — FST query, no ONNX). On miss, the column stays NULL
//! and the slow-path `ctse_run_backfill` Tauri command (§1C-4) picks
//! it up later via e5 inference.
//!
//! Bigram tokens (joined by `BIGRAM_SEP`, U+001F) are skipped from
//! resolution — the lexicon doesn't store bigrams, and a bigram sent
//! to the slow path returns noise.
//!
//! ## Hot-path invariants
//!
//! - **No ONNX** — write path must never call the embedding engine.
//!   Slow-path resolution is exclusively the backfill's job.
//! - **No allocations in the steady state** — stopwords are cached in
//!   a `OnceLock`, hash maps are sized from the token vec length.
//! - **Body cap** — pathological bodies (Wikipedia paste, hex blob)
//!   are clipped to 1 MiB on a UTF-8 boundary, mirroring the
//!   `BODY_CAP_BYTES` precedent set by the original Phase 1 commit.

use crate::lexicon::LexiconGraph;
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Per-note safety cap. Mirrors the constant used by the now-retired
/// Phase 1 `populate_term_vocab` bootstrap. A pathological note with
/// hundreds of MB of pasted text would otherwise freeze the worker;
/// the first 1 MiB is enough for any meaningful vocabulary signal.
const BODY_CAP_BYTES: usize = 1024 * 1024;

/// Process-wide stopword set. Built once on first hook invocation,
/// reused for every subsequent save. The set itself is small
/// (~hundreds of entries across 15 languages); building it is cheap
/// but doing it on every save would still be wasteful.
fn stopwords_cached() -> &'static HashSet<String> {
    static SW: OnceLock<HashSet<String>> = OnceLock::new();
    SW.get_or_init(crate::libraries::build_stopwords)
}

/// Clip a string to at most `cap` bytes on a UTF-8 codepoint boundary.
fn clip_utf8(s: &str, cap: usize) -> &str {
    if s.len() <= cap {
        return s;
    }
    let mut end = cap;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Tokenize a body into per-term occurrence counts. Empty input → empty
/// map. Identical token namespace to `notes_fts` (same tokenizer call).
fn token_counts(body: &str) -> HashMap<String, u32> {
    let clipped = clip_utf8(body, BODY_CAP_BYTES);
    let tokens = crate::fts5_tokenizer::tokenize_to_vec(clipped, stopwords_cached());
    let mut counts: HashMap<String, u32> = HashMap::with_capacity(tokens.len());
    for t in tokens {
        *counts.entry(t).or_insert(0) += 1;
    }
    counts
}

/// Per-term delta for a single note save. Both fields are signed:
/// positive = increment, negative = decrement, zero = no change.
#[derive(Debug, Clone, Copy)]
struct Delta {
    /// Net change to `term_vocab.total_count` for this term.
    total: i32,
    /// Net change to `term_vocab.doc_count` for this term.
    /// `+1` if the term appears in the new body but not the old.
    /// `-1` if the term was in the old body but is gone from the new.
    /// `0` if the term is in both bodies (any count change goes to
    /// `total` only — `doc_count` is per-note presence, not occurrences).
    doc: i32,
}

/// Compute per-term delta from old vs. new body token counts.
fn compute_delta(
    old: &HashMap<String, u32>,
    new: &HashMap<String, u32>,
) -> HashMap<String, Delta> {
    let mut out: HashMap<String, Delta> = HashMap::new();
    let keys: HashSet<&String> = old.keys().chain(new.keys()).collect();
    for term in keys {
        let oc = old.get(term).copied().unwrap_or(0) as i32;
        let nc = new.get(term).copied().unwrap_or(0) as i32;
        let total = nc - oc;
        let doc = match (oc, nc) {
            (0, n) if n > 0 => 1,
            (o, 0) if o > 0 => -1,
            _ => 0,
        };
        if total != 0 || doc != 0 {
            out.insert(term.clone(), Delta { total, doc });
        }
    }
    out
}

/// Apply a delta map to `term_vocab` inside the caller's connection.
///
/// Returns the list of terms that were newly inserted (didn't exist in
/// `term_vocab` before this call). Used by the caller to drive
/// fast-path concept resolution on just the fresh additions.
fn apply_delta(conn: &Connection, delta: &HashMap<String, Delta>) -> Result<Vec<String>, String> {
    if delta.is_empty() {
        return Ok(Vec::new());
    }
    let mut new_terms: Vec<String> = Vec::new();

    let mut select_existing = conn
        .prepare("SELECT total_count FROM term_vocab WHERE term = ?1")
        .map_err(|e| format!("term_vocab existence prepare failed: {}", e))?;
    let mut update_stmt = conn
        .prepare(
            "UPDATE term_vocab \
             SET total_count = MAX(0, total_count + ?1), \
                 doc_count = MAX(0, doc_count + ?2) \
             WHERE term = ?3",
        )
        .map_err(|e| format!("term_vocab update prepare failed: {}", e))?;
    let mut insert_stmt = conn
        .prepare(
            "INSERT INTO term_vocab (term, doc_count, total_count, bridge_concept_id) \
             VALUES (?1, ?2, ?3, NULL)",
        )
        .map_err(|e| format!("term_vocab insert prepare failed: {}", e))?;

    for (term, d) in delta {
        let exists: bool = select_existing
            .query_row(params![term], |_| Ok(true))
            .ok()
            .unwrap_or(false);
        if exists {
            update_stmt
                .execute(params![d.total, d.doc, term])
                .map_err(|e| format!("term_vocab update failed for {term:?}: {}", e))?;
        } else if d.total > 0 {
            // Only insert when the net total is positive — a delete
            // path could produce a row that "should be subtracted from"
            // a non-existent row; skipping silently is correct because
            // the term wasn't present in the index anyway.
            let initial_doc = d.doc.max(0) as i64;
            insert_stmt
                .execute(params![term, initial_doc, d.total as i64])
                .map_err(|e| format!("term_vocab insert failed for {term:?}: {}", e))?;
            new_terms.push(term.clone());
        }
    }
    Ok(new_terms)
}

/// Apply fast-path concept resolution to a list of newly-inserted
/// terms. Slow-path (ONNX) resolution is deferred to the backfill.
/// Reuses the shared multi-language fast-path helper from
/// [`super::fast_path_concept_id`].
fn fast_path_resolve_new_terms(conn: &Connection, terms: &[String]) -> Result<(), String> {
    if terms.is_empty() {
        return Ok(());
    }
    let graph = LexiconGraph::get();
    let mut update = conn
        .prepare("UPDATE term_vocab SET bridge_concept_id = ?1 WHERE term = ?2")
        .map_err(|e| format!("term_vocab bridge update prepare failed: {}", e))?;
    for term in terms {
        if let Some(cid) = super::fast_path_concept_id(graph, term) {
            update
                .execute(params![cid, term])
                .map_err(|e| format!("term_vocab bridge update failed for {term:?}: {}", e))?;
        }
    }
    Ok(())
}

/// Hook fired after a single note has been (re)indexed. Computes the
/// delta from old to new body and applies it to `term_vocab`. New
/// terms get a fast-path concept lookup (M11 only — no ONNX).
///
/// `old_body` is `None` for first-time indexing (the path didn't exist
/// in `note_meta` before this save). In that case every token in the
/// new body is a "new" contribution.
///
/// Best-effort: errors are returned but the caller is expected to log
/// and continue. `term_vocab` is a derived view — a single-row failure
/// must not fail the underlying file save.
pub fn on_note_indexed(
    conn: &Connection,
    _note_path: &str,
    old_body: Option<&str>,
    new_body: &str,
) -> Result<(), String> {
    let old = old_body.map(token_counts).unwrap_or_default();
    let new = token_counts(new_body);
    let delta = compute_delta(&old, &new);
    let new_terms = apply_delta(conn, &delta)?;
    fast_path_resolve_new_terms(conn, &new_terms)?;
    Ok(())
}

/// Hook fired after a note has been deleted from `note_meta`. Subtracts
/// the deleted body's term contributions from `term_vocab`. Rows that
/// drop to zero `total_count` are kept as tombstones with their
/// `bridge_concept_id` intact — the term may reappear in a future save,
/// and re-resolving the concept on every revival is wasteful.
pub fn on_note_deleted(
    conn: &Connection,
    _note_path: &str,
    old_body: &str,
) -> Result<(), String> {
    let old = token_counts(old_body);
    let mut delta: HashMap<String, Delta> = HashMap::with_capacity(old.len());
    for (term, &oc) in &old {
        delta.insert(
            term.clone(),
            Delta {
                total: -(oc as i32),
                doc: -1,
            },
        );
    }
    apply_delta(conn, &delta)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn make_term_vocab(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE term_vocab (
                term TEXT PRIMARY KEY,
                doc_count INTEGER NOT NULL,
                total_count INTEGER NOT NULL,
                bridge_concept_id TEXT
            );",
        )
        .unwrap();
    }

    fn count_row(conn: &Connection, term: &str) -> Option<(i64, i64, Option<String>)> {
        conn.query_row(
            "SELECT doc_count, total_count, bridge_concept_id FROM term_vocab WHERE term = ?1",
            params![term],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .ok()
    }

    /// First-time index of a note inserts its tokens with the right
    /// counts and resolves M11-known terms via the fast path.
    #[test]
    fn first_index_inserts_and_fast_path_resolves() {
        let conn = Connection::open_in_memory().unwrap();
        make_term_vocab(&conn);

        on_note_indexed(&conn, "note1.md", None, "book book knowledge").unwrap();

        let book = count_row(&conn, "book").expect("'book' should be inserted");
        assert_eq!(book.0, 1, "doc_count for 'book'");
        assert!(book.1 >= 2, "total_count for 'book' should be at least 2");
        assert!(
            book.2.as_deref().map(|s| s.starts_with("c:")).unwrap_or(false),
            "fast path should resolve 'book' to a c: concept id; got {:?}",
            book.2
        );
    }

    /// A second save of the same note with the same body yields no row
    /// changes — delta is all zeros, no-op.
    #[test]
    fn idempotent_resave_yields_no_delta() {
        let conn = Connection::open_in_memory().unwrap();
        make_term_vocab(&conn);
        on_note_indexed(&conn, "n.md", None, "book").unwrap();
        let before = count_row(&conn, "book").unwrap();
        on_note_indexed(&conn, "n.md", Some("book"), "book").unwrap();
        let after = count_row(&conn, "book").unwrap();
        assert_eq!(before, after, "no-op resave must not change row");
    }

    /// Editing a note to add a token bumps total/doc; removing a token
    /// decrements them.
    #[test]
    fn edit_applies_signed_delta() {
        let conn = Connection::open_in_memory().unwrap();
        make_term_vocab(&conn);

        on_note_indexed(&conn, "n.md", None, "book book").unwrap();
        let after_first = count_row(&conn, "book").unwrap();
        assert_eq!(after_first.0, 1, "doc_count after first save");
        assert_eq!(after_first.1, 2, "total_count after first save");

        on_note_indexed(&conn, "n.md", Some("book book"), "book book book").unwrap();
        let after_edit = count_row(&conn, "book").unwrap();
        assert_eq!(after_edit.0, 1, "doc_count unchanged when term still present");
        assert_eq!(after_edit.1, 3, "total_count incremented by 1");

        on_note_indexed(&conn, "n.md", Some("book book book"), "knowledge").unwrap();
        let after_remove = count_row(&conn, "book");
        assert!(
            after_remove.map(|r| r.0 == 0 && r.1 == 0).unwrap_or(false),
            "book row should drop to zero after being removed from note"
        );
        let know = count_row(&conn, "knowledge").expect("'knowledge' inserted");
        assert_eq!(know.0, 1);
        assert_eq!(know.1, 1);
    }

    /// Deleting a note subtracts its term contributions and tombstones
    /// rows that drop to zero — they are NOT removed.
    #[test]
    fn delete_subtracts_and_tombstones() {
        let conn = Connection::open_in_memory().unwrap();
        make_term_vocab(&conn);
        on_note_indexed(&conn, "n.md", None, "book knowledge").unwrap();
        on_note_deleted(&conn, "n.md", "book knowledge").unwrap();

        let book = count_row(&conn, "book").expect("tombstone row remains");
        assert_eq!(book.0, 0);
        assert_eq!(book.1, 0);
        assert!(
            book.2.is_some(),
            "bridge_concept_id is preserved across delete"
        );
    }

    /// Bigram tokens (containing the FTS5 sentinel byte) skip fast-path
    /// resolution — bigrams are not lexicon-resolvable.
    #[test]
    fn bigram_tokens_stay_null_after_fast_path() {
        let conn = Connection::open_in_memory().unwrap();
        make_term_vocab(&conn);
        // Inject a synthetic bigram row directly so we don't rely on
        // tokenizer internals to produce one.
        let bigram = format!("foo\u{001f}bar");
        conn.execute(
            "INSERT INTO term_vocab (term, doc_count, total_count, bridge_concept_id) VALUES (?1, 1, 1, NULL)",
            params![bigram],
        )
        .unwrap();
        fast_path_resolve_new_terms(&conn, &[bigram.clone()]).unwrap();
        let row = count_row(&conn, &bigram).unwrap();
        assert!(row.2.is_none(), "bigram must remain unresolved (NULL)");
    }
}
