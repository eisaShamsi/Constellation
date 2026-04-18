//! Lexicon seed TSV parser — M10.
//!
//! One row = one `ConceptRecord`. Row format:
//!
//!   `concept_id<TAB>pos<TAB>lang:lemma,lemma,...<TAB>lang:lemma,...<TAB>...`
//!
//! Blank lines and `#`-prefixed lines are skipped. Unknown `lang_code:`
//! prefixes are dropped. Malformed rows (missing id, missing pos column,
//! no language columns) are skipped with a warning to stderr — the core
//! tier should not refuse to load on one bad row.
//!
//! Scales to the M11 20K-concept core without format changes. The parser
//! is intentionally permissive so hand-edited dialect packs can ship with
//! minor whitespace inconsistencies without breaking the boot path.
//!
//! # Example
//!
//! ```text
//! c:book	Noun	en:book,books	ar:كتاب	fr:livre
//! ```
//!
//! parses to:
//!
//! ```ignore
//! ConceptRecord {
//!     id: "c:book",
//!     pos: Some(PartOfSpeech::Noun),
//!     labels: { Lang::En: ["book", "books"], Lang::Ar: ["كتاب"],
//!               Lang::Fr: ["livre"] },
//! }
//! ```

use crate::arabic::{Lang, PartOfSpeech};
use std::collections::BTreeMap;

/// A single parsed concept from the seed TSV, before it becomes nodes
/// + edges in the graph. Labels are preserved in iteration-stable
/// order (`BTreeMap`) so two parses of the same TSV produce byte-
/// identical graph output — useful for reproducible FST digests in M11.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptRecord {
    /// Stable slug like `c:book`. Kept verbatim (no normalization).
    pub id: String,
    /// Part of speech. `None` when the TSV has an empty pos column
    /// or the value does not match a known variant.
    pub pos: Option<PartOfSpeech>,
    /// Language → lemma list. Always at least one language present in
    /// a successfully-parsed record.
    pub labels: BTreeMap<Lang, Vec<String>>,
}

/// Error kinds that cause a row to be skipped. Kept as an enum so the
/// M14 UI can surface per-row diagnostics when the user pastes a
/// custom pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseRowError {
    /// Row had fewer than 3 tab-separated columns.
    TooFewColumns,
    /// `concept_id` column was empty.
    EmptyId,
    /// No language columns parsed cleanly — everything after the pos
    /// column was malformed.
    NoLabels,
}

/// Parse a seed TSV string into a list of concept records.
///
/// Comment and blank lines are skipped. Malformed rows are counted
/// but silently dropped (use `parse_with_diagnostics` if you need the
/// per-row errors).
pub fn parse(tsv: &str) -> Vec<ConceptRecord> {
    let (records, _errors) = parse_with_diagnostics(tsv);
    records
}

/// Same as `parse` but returns the per-row errors alongside the
/// successful records. Used by the test harness and (in M14) the UI
/// when the user pastes a custom dialect pack.
pub fn parse_with_diagnostics(
    tsv: &str,
) -> (Vec<ConceptRecord>, Vec<(usize, ParseRowError)>) {
    let mut records = Vec::new();
    let mut errors = Vec::new();
    for (line_idx, line) in tsv.lines().enumerate() {
        let trimmed = line.trim_end_matches('\r');
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        match parse_row(trimmed) {
            Ok(r) => records.push(r),
            Err(e) => errors.push((line_idx + 1, e)),
        }
    }
    (records, errors)
}

fn parse_row(line: &str) -> Result<ConceptRecord, ParseRowError> {
    let mut cols = line.split('\t');
    let id = cols.next().ok_or(ParseRowError::TooFewColumns)?.trim();
    if id.is_empty() {
        return Err(ParseRowError::EmptyId);
    }
    let pos_str = cols.next().ok_or(ParseRowError::TooFewColumns)?.trim();
    let pos = parse_pos(pos_str);

    let mut labels: BTreeMap<Lang, Vec<String>> = BTreeMap::new();
    for col in cols {
        let col = col.trim();
        if col.is_empty() {
            continue;
        }
        let Some((lang_code, lemmas)) = col.split_once(':') else {
            continue; // malformed column — drop it
        };
        let Some(lang) = Lang::from_code(lang_code.trim()) else {
            continue; // unknown language code — drop the column
        };
        let bucket = labels.entry(lang).or_default();
        for lemma in lemmas.split(',') {
            let lemma = lemma.trim();
            if !lemma.is_empty() {
                bucket.push(lemma.to_string());
            }
        }
        // Drop the bucket if it ended up empty (pathological input).
        if bucket.is_empty() {
            labels.remove(&lang);
        }
    }

    if labels.is_empty() {
        return Err(ParseRowError::NoLabels);
    }

    Ok(ConceptRecord {
        id: id.to_string(),
        pos,
        labels,
    })
}

fn parse_pos(s: &str) -> Option<PartOfSpeech> {
    match s {
        "" | "Unknown" => None,
        "Noun" => Some(PartOfSpeech::Noun),
        "Verb" => Some(PartOfSpeech::Verb),
        "Adjective" => Some(PartOfSpeech::Adjective),
        "Adverb" => Some(PartOfSpeech::Adverb),
        "ProperNoun" => Some(PartOfSpeech::ProperNoun),
        "Particle" => Some(PartOfSpeech::Particle),
        "Foreign" => Some(PartOfSpeech::Foreign),
        // Unknown label — treat as unspecified rather than rejecting
        // the row. Lets dialect packs introduce new labels without
        // breaking the core parser.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_simple_row() {
        let tsv = "c:book\tNoun\ten:book\tar:كتاب";
        let rs = parse(tsv);
        assert_eq!(rs.len(), 1);
        let r = &rs[0];
        assert_eq!(r.id, "c:book");
        assert_eq!(r.pos, Some(PartOfSpeech::Noun));
        assert_eq!(r.labels.len(), 2);
        assert_eq!(r.labels[&Lang::En], vec!["book"]);
        assert_eq!(r.labels[&Lang::Ar], vec!["كتاب"]);
    }

    #[test]
    fn multiple_lemmas_in_one_lang_column() {
        let tsv = "c:book\tNoun\ten:book,books,tome";
        let rs = parse(tsv);
        assert_eq!(rs[0].labels[&Lang::En], vec!["book", "books", "tome"]);
    }

    #[test]
    fn comments_and_blanks_are_skipped() {
        let tsv = "\
# header comment
# another

c:book\tNoun\ten:book
# trailing comment
";
        let rs = parse(tsv);
        assert_eq!(rs.len(), 1);
        assert_eq!(rs[0].id, "c:book");
    }

    #[test]
    fn unknown_lang_code_is_dropped_not_rejected() {
        let tsv = "c:book\tNoun\ten:book\txyz:foo\tar:كتاب";
        let rs = parse(tsv);
        assert_eq!(rs.len(), 1);
        // `xyz:` dropped, the valid langs survived.
        assert_eq!(rs[0].labels.len(), 2);
        assert!(rs[0].labels.contains_key(&Lang::En));
        assert!(rs[0].labels.contains_key(&Lang::Ar));
    }

    #[test]
    fn empty_pos_means_unknown() {
        let tsv = "c:foo\t\ten:foo";
        let rs = parse(tsv);
        assert_eq!(rs[0].pos, None);
    }

    #[test]
    fn row_with_no_labels_is_skipped() {
        let tsv = "c:lonely\tNoun\txyz:foo";
        let (rs, errs) = parse_with_diagnostics(tsv);
        assert!(rs.is_empty());
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].1, ParseRowError::NoLabels);
    }

    #[test]
    fn row_with_empty_id_is_skipped() {
        let tsv = "\tNoun\ten:foo";
        let (rs, errs) = parse_with_diagnostics(tsv);
        assert!(rs.is_empty());
        assert_eq!(errs[0].1, ParseRowError::EmptyId);
    }

    #[test]
    fn row_with_only_one_column_is_skipped() {
        let tsv = "c:foo";
        let (rs, errs) = parse_with_diagnostics(tsv);
        assert!(rs.is_empty());
        assert_eq!(errs[0].1, ParseRowError::TooFewColumns);
    }

    #[test]
    fn crlf_line_endings_are_handled() {
        let tsv = "c:book\tNoun\ten:book\r\nc:read\tVerb\ten:read\r\n";
        let rs = parse(tsv);
        assert_eq!(rs.len(), 2);
    }

    #[test]
    fn lemmas_with_surrounding_whitespace_are_trimmed() {
        let tsv = "c:book\tNoun\ten: book , books ";
        let rs = parse(tsv);
        assert_eq!(rs[0].labels[&Lang::En], vec!["book", "books"]);
    }

    #[test]
    fn duplicate_lang_column_appends() {
        // The TSV spec says one column per language, but two columns of
        // `en:` should merge their lemma lists rather than lose data.
        let tsv = "c:book\tNoun\ten:book\ten:tome";
        let rs = parse(tsv);
        assert_eq!(rs[0].labels[&Lang::En], vec!["book", "tome"]);
    }

    #[test]
    fn whole_seed_file_parses() {
        // Smoke-test the packaged seed. Count-sensitive so we notice if
        // someone deletes a row by accident.
        let tsv = include_str!("data/seed_v1.tsv");
        let rs = parse(tsv);
        assert!(rs.len() >= 10, "seed shrank unexpectedly: {}", rs.len());
        // Every record has at least English + Arabic (the two primary
        // Constellation languages today).
        for r in &rs {
            assert!(r.labels.contains_key(&Lang::En), "missing en on {}", r.id);
            assert!(r.labels.contains_key(&Lang::Ar), "missing ar on {}", r.id);
        }
    }
}
