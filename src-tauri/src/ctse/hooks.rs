//! CTSE write-time hooks (MIG-013 §1C, query-time-expansion variant).
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
//! ## What this hook does NOT do anymore
//!
//! Earlier §1C drafts also fast-path-resolved each new term to its
//! M11 concept ID and stored the result in `term_vocab.bridge_concept_id`
//! for use in cross-language search. **That column is now dead schema**
//! (left in place for forward-compat, but never read or written from
//! the hook). Cross-language search runs entirely at query time —
//! `ctse::search::ctse_search_by_concept` embeds the user query, finds
//! top-K M11 concepts, and expands them to multilingual lemmas in
//! memory. Pre-computing the term→concept map per save was a Working
//! Agreement #5 violation: the dominant industry pattern (Lucene
//! `SynonymGraphFilter`, SQLite FTS5 Method 2, CLIR query-translation,
//! Primo controlled-vocabulary expansion) all do query-time expansion,
//! not document-side concept tagging. Removing the per-save fast-path
//! call eliminates the bigram-explosion + slow-path-takes-hours
//! pathology entirely.
//!
//! What's left here is the bare ledger: maintain `(doc_count,
//! total_count)` per term so the Index panel and other consumers see
//! a current vocabulary view.
//!
//! ## Why both `total_count` AND `doc_count` are maintained
//!
//! - `total_count` — total occurrences across the corpus. Used by
//!   the Index panel for "popular terms" lists.
//! - `doc_count` — number of distinct notes containing the term. The
//!   IDF half of TF-IDF and a useful cardinality signal in its own right.
//!
//! Both are kept exact via per-save delta computation: the old body's
//! token-count map is differenced against the new body's, yielding a
//! signed delta per term.
//!
//! ## Hot-path invariants
//!
//! - **No ONNX, no concept lookup** — write path is purely local
//!   bookkeeping. Cross-language semantics live entirely on the read
//!   side.
//! - **No allocations in the steady state** — stopwords are cached in
//!   a `OnceLock`, hash maps are sized from the token vec length.
//! - **Body cap** — pathological bodies (Wikipedia paste, hex blob)
//!   are clipped to 1 MiB on a UTF-8 boundary, mirroring the
//!   `BODY_CAP_BYTES` precedent set by the original Phase 1 commit.

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
        // MIG-041: term_vocab stores SINGLE STEMS ONLY. Skip bigrams
        // (two stems joined by BIGRAM_SEP = 0x1F). They are redundant with
        // the FTS5 `notes_fts` index — which is what the Index panel,
        // phrase search, and Arabic matching read (`notes_vocab`) — and
        // nothing reads them from `term_vocab` (the query-time concept
        // expansion in `ctse::search` skips them on read, same predicate).
        // Writing them here only bloated the table (~90% of its rows).
        if t.as_bytes().contains(&crate::fts5_tokenizer::BIGRAM_SEP) {
            continue;
        }
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
fn apply_delta(conn: &Connection, delta: &HashMap<String, Delta>) -> Result<(), String> {
    if delta.is_empty() {
        return Ok(());
    }
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
        }
    }
    Ok(())
}

/// Hook fired after a single note has been (re)indexed. Computes the
/// delta from old to new body and applies it to `term_vocab`.
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
    apply_delta(conn, &delta)
}

/// Hook fired after a note has been deleted from `note_meta`. Subtracts
/// the deleted body's term contributions from `term_vocab`. Rows that
/// drop to zero `total_count` are kept as tombstones — the term may
/// reappear in a future save and re-counting from zero is correct.
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
    apply_delta(conn, &delta)
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

    fn count_row(conn: &Connection, term: &str) -> Option<(i64, i64)> {
        conn.query_row(
            "SELECT doc_count, total_count FROM term_vocab WHERE term = ?1",
            params![term],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .ok()
    }

    /// First-time index of a note inserts its tokens with the right
    /// counts. (No fast-path concept resolution anymore — that's a
    /// query-time concern in `ctse::search`.)
    #[test]
    fn first_index_inserts_with_correct_counts() {
        let conn = Connection::open_in_memory().unwrap();
        make_term_vocab(&conn);

        on_note_indexed(&conn, "note1.md", None, "book book knowledge").unwrap();

        let book = count_row(&conn, "book").expect("'book' should be inserted");
        assert_eq!(book.0, 1, "doc_count for 'book'");
        assert!(book.1 >= 2, "total_count for 'book' should be at least 2");

        let know = count_row(&conn, "knowledge").expect("'knowledge' should be inserted");
        assert_eq!(know.0, 1, "doc_count for 'knowledge'");
        assert!(know.1 >= 1, "total_count for 'knowledge' should be at least 1");
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

    /// MIG-041: `term_vocab` stores single stems only. Two same-script
    /// words form a bigram in the FTS5 token stream, but that bigram must
    /// NOT be written as a `term_vocab` row anymore — only the two stems.
    #[test]
    fn bigrams_are_not_written_to_term_vocab() {
        let conn = Connection::open_in_memory().unwrap();
        make_term_vocab(&conn);
        // "knowledge book" → stems "knowledge" + "book" (same script) would
        // emit the bigram "knowledge\x1Fbook" in the FTS5 token stream.
        on_note_indexed(&conn, "n.md", None, "knowledge book").unwrap();

        assert!(count_row(&conn, "knowledge").is_some(), "single stem 'knowledge' present");
        assert!(count_row(&conn, "book").is_some(), "single stem 'book' present");

        let bigram_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM term_vocab WHERE term LIKE '%' || CHAR(31) || '%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(bigram_rows, 0, "MIG-041: no bigram rows may be written to term_vocab");

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM term_vocab", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 2, "only the two single stems are stored");
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
    }
}
