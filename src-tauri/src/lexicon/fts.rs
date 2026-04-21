//! FTS5 MATCH expression generation for query expansion — M12.
//!
//! Takes an [`ExpansionResult`](crate::lexicon::ExpansionResult) and
//! produces a single `MATCH` clause the search engine can pass straight
//! to `WHERE notes_fts MATCH ?`. The expression is a disjunction of
//! phrase-quoted terms:
//!
//! ```text
//! "book" OR "books" OR "كتاب" OR "livre" OR "knowledge" OR …
//! ```
//!
//! # Why phrase queries, not bare tokens
//!
//! FTS5 treats unquoted uppercase `AND` / `OR` / `NOT` / `NEAR` as
//! operators. A raw user term that happens to collide ("the OR gate")
//! would parse as a query fragment and produce surprising results.
//! Wrapping every term in double quotes turns it into an opaque phrase
//! so a stray operator keyword inside a lemma (unlikely but possible at
//! M11 scale with 20K concepts × 15 langs) cannot change the query
//! shape.
//!
//! # Why no escaping of `"`
//!
//! FTS5's phrase syntax has no escape sequence — a `"` inside `"..."`
//! terminates the phrase early. A lemma containing `"` would therefore
//! silently corrupt the MATCH expression. Rather than try to work
//! around that with clever quoting, we strip `"` before wrapping. This
//! is a no-op at M11 scale (no supported language uses `"` as a lemma
//! character) and the behaviour is documented loudly so any future
//! pack with exotic characters cannot blow up at search time.
//!
//! # Why a separate helper module instead of an inline search.rs fn
//!
//! `search.rs` is where the SQL lives; it should know MATCH syntax but
//! not the lexicon. `lexicon::fts` is where the lexicon lives; it
//! should know MATCH syntax but not SQL. The two meet at
//! [`expand_to_match_expr`](crate::lexicon::expand_to_match_expr) — a
//! single end-to-end call that [`search.rs`] will reach for once the
//! M14 settings UI wires expansion into the user-facing lexical
//! path. Keeping the generator in the lexicon module means the tests
//! here cover every escaping / empty-input / edge-case concern without
//! needing a real search database.

use super::expansion::ExpansionResult;

/// Escape a single lemma for inclusion in an FTS5 phrase query.
///
/// Wraps the term in `"..."`. Strips interior double quotes and
/// control characters. Returns `None` when the result would be an
/// empty phrase (input was empty, all-whitespace, or entirely stripped
/// by the filter) so callers can fall back to their plain-query path
/// instead of producing `""` (which FTS5 treats as a syntax error).
///
/// ```ignore
/// assert_eq!(escape_fts_term("book"), Some("\"book\"".into()));
/// assert_eq!(escape_fts_term("  كتاب  "), Some("\"كتاب\"".into()));
/// assert_eq!(escape_fts_term(""), None);
/// ```
pub fn escape_fts_term(term: &str) -> Option<String> {
    // Strip double-quotes (FTS5 phrases have no escape syntax) and
    // control characters (null / tab / newline can confuse tokenizers
    // regardless of what the syntax says about them).
    let cleaned: String = term
        .chars()
        .filter(|c| *c != '"' && !c.is_control())
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(trimmed.len() + 2);
    out.push('"');
    out.push_str(trimmed);
    out.push('"');
    Some(out)
}

/// Build an OR-joined FTS5 MATCH expression from an expansion result.
///
/// Walks [`ExpansionResult::flat_terms`] (which already deduplicates
/// and echoes the source lemma), escapes each term via
/// [`escape_fts_term`], drops any that come back `None`, and joins the
/// survivors with ` OR `. Further deduplication on the escaped string
/// catches the case where two terms differ only in characters the
/// escape pass strips.
///
/// Returns `None` when the expansion produces zero usable terms — the
/// caller should treat that as "fall back to the user's raw query"
/// rather than passing an empty MATCH clause (which FTS5 errors on).
///
/// Preserves insertion order from `flat_terms`: the source lemma
/// appears first, then equivalents, then synonyms, then hypernyms,
/// then hyponyms. Stable ordering helps with test assertions and
/// makes the generated query legible in diagnostics logs.
pub fn build_match_expr(expansion: &ExpansionResult) -> Option<String> {
    let terms = expansion.flat_terms();
    let mut escaped: Vec<String> = Vec::with_capacity(terms.len());
    for (_lang, term) in terms {
        let Some(e) = escape_fts_term(&term) else {
            continue;
        };
        if !escaped.contains(&e) {
            escaped.push(e);
        }
    }
    if escaped.is_empty() {
        None
    } else {
        Some(escaped.join(" OR "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arabic::Lang;
    use crate::lexicon::parse::parse;
    use crate::lexicon::{expand_via, ExpansionOptions, LexiconGraph, SynonymLevel};
    use std::collections::HashMap;

    // ── escape_fts_term ───────────────────────────────────────────

    #[test]
    fn escape_wraps_in_double_quotes() {
        assert_eq!(escape_fts_term("book"), Some("\"book\"".to_string()));
    }

    #[test]
    fn escape_trims_surrounding_whitespace() {
        assert_eq!(escape_fts_term("  book  "), Some("\"book\"".to_string()));
    }

    #[test]
    fn escape_strips_interior_double_quotes() {
        // A term with `"` inside would corrupt the phrase if not filtered.
        assert_eq!(
            escape_fts_term("the \"book\""),
            Some("\"the book\"".to_string())
        );
    }

    #[test]
    fn escape_strips_control_characters() {
        // Tab / newline inside a term drop out; the rest is preserved.
        assert_eq!(escape_fts_term("book\n"), Some("\"book\"".to_string()));
        assert_eq!(escape_fts_term("a\tb"), Some("\"ab\"".to_string()));
    }

    #[test]
    fn escape_returns_none_on_empty() {
        assert_eq!(escape_fts_term(""), None);
    }

    #[test]
    fn escape_returns_none_on_whitespace_only() {
        assert_eq!(escape_fts_term("   "), None);
    }

    #[test]
    fn escape_returns_none_when_filter_strips_everything() {
        // A term that's only double-quotes and control chars collapses to empty.
        assert_eq!(escape_fts_term("\"\""), None);
        assert_eq!(escape_fts_term("\n\t"), None);
    }

    #[test]
    fn escape_preserves_arabic_script() {
        assert_eq!(escape_fts_term("كتاب"), Some("\"كتاب\"".to_string()));
    }

    #[test]
    fn escape_preserves_multi_word_phrases() {
        // Multi-word lemmas (proper nouns, compound terms) survive intact.
        // FTS5 phrase queries are exactly the right shape for these.
        assert_eq!(
            escape_fts_term("New York"),
            Some("\"New York\"".to_string())
        );
    }

    #[test]
    fn escape_preserves_internal_whitespace() {
        // Only leading / trailing whitespace gets trimmed; internal
        // whitespace stays as-is so phrase matching works.
        assert_eq!(
            escape_fts_term("  a  b  "),
            Some("\"a  b\"".to_string())
        );
    }

    // ── build_match_expr ──────────────────────────────────────────

    fn tiny_graph() -> LexiconGraph {
        let tsv = "\
c:book\tNoun\ten:book,books\tar:كتاب\tfr:livre
c:knowledge\tNoun\ten:knowledge,cognition\tar:معرفة\tfr:connaissance
";
        LexiconGraph::from_records(parse(tsv)).unwrap()
    }

    #[test]
    fn build_contains_source_lemma_and_translations() {
        let g = tiny_graph();
        let r = expand_via(&g, "book", Lang::En, &ExpansionOptions::default());
        let expr = build_match_expr(&r).expect("non-empty expansion");
        assert!(expr.contains("\"book\""), "missing source: {}", expr);
        assert!(expr.contains("\"كتاب\""), "missing arabic: {}", expr);
        assert!(expr.contains("\"livre\""), "missing french: {}", expr);
        assert!(expr.contains(" OR "));
    }

    #[test]
    fn build_includes_in_language_synonyms_at_default_level() {
        let g = tiny_graph();
        let r = expand_via(&g, "book", Lang::En, &ExpansionOptions::default());
        let expr = build_match_expr(&r).unwrap();
        // `books` is a same-concept same-lang synonym of `book`.
        assert!(expr.contains("\"books\""), "missing synonym: {}", expr);
    }

    #[test]
    fn build_omits_synonyms_when_level_is_none() {
        let g = tiny_graph();
        let mut opts = ExpansionOptions::default();
        opts.synonym_level = SynonymLevel::None;
        let r = expand_via(&g, "book", Lang::En, &opts);
        let expr = build_match_expr(&r).unwrap();
        assert!(!expr.contains("\"books\""), "synonyms leaked: {}", expr);
        // Equivalents still present.
        assert!(expr.contains("\"book\""));
        assert!(expr.contains("\"كتاب\""));
    }

    #[test]
    fn build_source_lemma_appears_only_once() {
        let g = tiny_graph();
        let r = expand_via(&g, "book", Lang::En, &ExpansionOptions::default());
        let expr = build_match_expr(&r).unwrap();
        let count = expr.matches("\"book\"").count();
        assert_eq!(count, 1, "source lemma duplicated: {}", expr);
    }

    #[test]
    fn build_returns_none_on_fully_empty_expansion() {
        // An all-empty ExpansionResult (source_lemma="") drops out entirely
        // because escape_fts_term returns None on the empty source.
        let r = ExpansionResult {
            source_lemma: String::new(),
            source_lang: Lang::En,
            equivalents: HashMap::new(),
            synonyms: HashMap::new(),
            hypernyms: HashMap::new(),
            hyponyms: HashMap::new(),
        };
        assert!(build_match_expr(&r).is_none());
    }

    #[test]
    fn build_returns_single_term_when_lemma_absent_from_graph() {
        // Unknown lemma: expand_via still echoes the source lemma. The
        // MATCH expression is then just the one phrase — a valid query
        // that behaves identically to today's un-expanded search.
        let g = tiny_graph();
        let r = expand_via(&g, "xyzzy", Lang::En, &ExpansionOptions::default());
        let expr = build_match_expr(&r).unwrap();
        assert_eq!(expr, "\"xyzzy\"");
    }

    #[test]
    fn build_respects_enabled_langs_filter() {
        // Restrict target langs to En only. Arabic / French must drop.
        let g = tiny_graph();
        let mut opts = ExpansionOptions::default();
        opts.enabled_langs.clear();
        opts.enabled_langs.insert(Lang::En);
        let r = expand_via(&g, "book", Lang::En, &opts);
        let expr = build_match_expr(&r).unwrap();
        assert!(!expr.contains("\"كتاب\""), "arabic leaked: {}", expr);
        assert!(!expr.contains("\"livre\""), "french leaked: {}", expr);
        assert!(expr.contains("\"book\""));
        assert!(expr.contains("\"books\""), "en synonym missing: {}", expr);
    }

    #[test]
    fn build_from_arabic_source_reaches_all_targets() {
        let g = tiny_graph();
        let r = expand_via(&g, "كتاب", Lang::Ar, &ExpansionOptions::default());
        let expr = build_match_expr(&r).unwrap();
        assert!(expr.contains("\"كتاب\""), "source missing: {}", expr);
        assert!(expr.contains("\"book\""), "en missing: {}", expr);
        assert!(expr.contains("\"livre\""), "fr missing: {}", expr);
    }

    #[test]
    fn build_term_with_quotes_would_have_been_dropped_individually() {
        // A synthetic ExpansionResult that forces the de-dup branch to
        // collapse two logically-distinct terms whose escaped forms
        // coincide after the quote-stripping pass.
        let mut r = ExpansionResult {
            source_lemma: "book".to_string(),
            source_lang: Lang::En,
            equivalents: HashMap::new(),
            synonyms: HashMap::new(),
            hypernyms: HashMap::new(),
            hyponyms: HashMap::new(),
        };
        r.synonyms.insert(
            Lang::En,
            vec!["\"book\"".to_string(), "book".to_string()],
        );
        let expr = build_match_expr(&r).unwrap();
        // Both synonyms escape down to `"book"`, which matches the
        // already-present source term — so the final expression is
        // just the single phrase.
        assert_eq!(expr, "\"book\"");
    }

    #[test]
    fn build_uses_or_separator_between_distinct_terms() {
        let g = tiny_graph();
        let r = expand_via(&g, "knowledge", Lang::En, &ExpansionOptions::default());
        let expr = build_match_expr(&r).unwrap();
        // At minimum: knowledge + cognition (syn) + معرفة + connaissance.
        // So the `OR` separator must appear at least three times.
        let or_count = expr.matches(" OR ").count();
        assert!(
            or_count >= 3,
            "expected at least 3 OR separators, got {}: {}",
            or_count,
            expr
        );
    }
}
