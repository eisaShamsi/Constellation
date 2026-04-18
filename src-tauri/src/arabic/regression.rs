//! M5 — regression corpus harness.
//!
//! A **regression corpus** is a held-out test set that pins the
//! analyzer's behaviour on a representative sample of inputs. It lives
//! in `regression_cases.tsv` next to this file (embedded via
//! `include_str!`, so there is no runtime I/O — the corpus rides along
//! inside the binary just like `roots_seed.tsv` and
//! `protected_seed.tsv`).
//!
//! # Why we need this
//!
//! The analyzer is about to gain two disruptive changes:
//!
//!   - **M6** — swap `stem_arabic_light10` in `fts5_tokenizer.rs` for
//!     `arabic::analyze`. Every FTS5 token on every note in every
//!     Universe now flows through this engine. A silent regression on
//!     `وائل` or `الأئمة` would re-poison the search index.
//!   - **M7** — the disambiguator will reorder multi-analysis results.
//!     If it picks the wrong analysis for `كاتب` (verb vs. active
//!     participle), downstream search tuples change.
//!
//! The unit tests in `mod.rs::tests` cover hand-picked flagship cases
//! (وائل, الأئمة, الكاتب, فكاتب). This corpus is the broader safety net:
//! 500 surfaces drawn from every origin + pattern + affix combination
//! the analyzer can produce, so a change that silently alters behaviour
//! on, say, "only imperfect-with-feminine-suffix verbs" shows up as a
//! concrete failing row instead of a vague end-user complaint three
//! months later.
//!
//! # Corpus file format
//!
//! Each non-comment, non-blank line is TAB-separated:
//!
//!   `surface<TAB>origin<TAB>lemma<TAB>root`
//!
//! Where:
//!   - **surface**  — the raw input to `analyze_best`.
//!   - **origin**   — one of `protected` | `generative` | `heuristic` |
//!                    `foreign` (matches `AnalysisOrigin`).
//!   - **lemma**    — expected `Analysis.lemma`, or `-` to skip the
//!                    lemma assertion (used for ambiguous generative
//!                    hits where `analyze_best`'s tiebreak isn't
//!                    stable across refactors).
//!   - **root**     — expected `Analysis.root` as the hyphen-joined
//!                    radical key (e.g. `ك-ت-ب`), or `-` for
//!                    root-less origins (proper nouns, foreign).
//!
//! Lines starting with `#` are comments. Blank lines are skipped.
//!
//! # Harness
//!
//! `run_corpus()` loads the embedded TSV, calls `analyze_best` for each
//! case, and returns a `CorpusReport { passed, failed: Vec<Failure> }`.
//! The `corpus_passes_with_full_score` integration test asserts
//! `failed.is_empty()` on every commit. A partial-score variant is
//! exposed so M6/M7 can be developed incrementally against the corpus
//! without blocking every commit.
//!
//! # Scoring policy
//!
//! The corpus is pass/fail — there is no confidence-level comparison in
//! v1. If the analyzer swaps origins (e.g. a word becomes protected
//! that was generative) the TSV row has to be updated explicitly. That
//! is deliberate: we want a human to notice the change and decide
//! whether it is an improvement or a regression. Silent drift is the
//! whole failure mode the corpus exists to prevent.

use super::{analyze_best, AnalysisOrigin};

/// Raw corpus text embedded at compile time. Zero I/O at runtime.
const CORPUS_TSV: &str = include_str!("regression_cases.tsv");

/// One parsed case from the TSV.
#[derive(Debug, Clone)]
pub struct Case {
    pub surface: String,
    pub origin: AnalysisOrigin,
    /// `None` means "don't assert" (the `-` sentinel in the TSV).
    pub expected_lemma: Option<String>,
    /// Same as above — `None` means "don't assert". Useful for multi-
    /// analysis surfaces where `analyze_best`'s tiebreak isn't stable.
    pub expected_root: Option<String>,
    /// Source line number (1-based) in the TSV — appears in failure
    /// messages so a broken case is trivially locatable.
    pub line: usize,
}

/// One failure captured by `run_corpus`. The `reason` string is the
/// human-readable diff; `case` is the offending row for context.
#[derive(Debug, Clone)]
pub struct Failure {
    pub case: Case,
    pub reason: String,
}

/// Aggregated corpus-run outcome.
#[derive(Debug, Clone)]
pub struct CorpusReport {
    pub passed: usize,
    pub failed: Vec<Failure>,
}

impl CorpusReport {
    pub fn total(&self) -> usize {
        self.passed + self.failed.len()
    }
    pub fn pass_rate(&self) -> f64 {
        if self.total() == 0 {
            return 1.0;
        }
        self.passed as f64 / self.total() as f64
    }
}

/// Parse a single origin tag. Returns `None` on unknown, which the
/// parser treats as "skip the row" rather than panicking.
fn parse_origin(tag: &str) -> Option<AnalysisOrigin> {
    match tag {
        "protected" => Some(AnalysisOrigin::ProtectedList),
        "generative" => Some(AnalysisOrigin::GenerativeFst),
        "heuristic" => Some(AnalysisOrigin::SurfaceHeuristic),
        // `foreign` is not its own origin variant — it's the surface-
        // heuristic path for non-Arabic scripts. We carry the sentinel
        // for readability in the TSV and map it to SurfaceHeuristic here.
        "foreign" => Some(AnalysisOrigin::SurfaceHeuristic),
        _ => None,
    }
}

/// Convert `-` sentinels into `None`; pass everything else through as
/// `Some(String)`. Separated out so tests can verify the sentinel
/// behaviour on its own.
fn parse_optional(cell: &str) -> Option<String> {
    if cell == "-" || cell.is_empty() {
        None
    } else {
        Some(cell.to_string())
    }
}

/// Parse the TSV into the case list. Invalid rows (wrong column count,
/// unknown origin) are silently skipped — they would surface as a
/// corpus-size-mismatch failure in `corpus_has_expected_size`.
pub fn parse_corpus(tsv: &str) -> Vec<Case> {
    let mut out = Vec::with_capacity(tsv.len() / 32);
    for (idx, raw_line) in tsv.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.splitn(4, '\t').collect();
        if cols.len() != 4 {
            continue;
        }
        let Some(origin) = parse_origin(cols[1].trim()) else {
            continue;
        };
        let surface = cols[0].trim();
        if surface.is_empty() {
            continue;
        }
        out.push(Case {
            surface: surface.to_string(),
            origin,
            expected_lemma: parse_optional(cols[2].trim()),
            expected_root: parse_optional(cols[3].trim()),
            line: line_no,
        });
    }
    out
}

/// Run every case through `analyze_best` and collect failures.
pub fn run_corpus() -> CorpusReport {
    let cases = parse_corpus(CORPUS_TSV);
    let mut report = CorpusReport {
        passed: 0,
        failed: Vec::new(),
    };
    for case in cases {
        match evaluate(&case) {
            Ok(()) => report.passed += 1,
            Err(reason) => report.failed.push(Failure { case, reason }),
        }
    }
    report
}

/// Evaluate a single case. Returns `Err(reason)` with a concise,
/// diff-style message on mismatch so the failing row is self-explanatory
/// in the test output.
fn evaluate(case: &Case) -> Result<(), String> {
    let a = analyze_best(&case.surface);
    // Origin must match — this is the single hardest property to regress
    // silently (changing it implies a pipeline shape change).
    if a.origin != case.origin {
        return Err(format!(
            "origin: expected {:?}, got {:?} (lemma={}, root={}, conf={:.2})",
            case.origin, a.origin, a.lemma, a.root, a.confidence
        ));
    }
    // Surface round-trip — the analyzer must preserve the input verbatim.
    if a.surface != case.surface {
        return Err(format!(
            "surface: expected {:?}, got {:?}",
            case.surface, a.surface
        ));
    }
    if let Some(exp) = &case.expected_lemma {
        if &a.lemma != exp {
            return Err(format!(
                "lemma: expected {:?}, got {:?}",
                exp, a.lemma
            ));
        }
    }
    if let Some(exp) = &case.expected_root {
        if &a.root != exp {
            return Err(format!(
                "root: expected {:?}, got {:?}",
                exp, a.root
            ));
        }
    }
    Ok(())
}

/// Iterate raw corpus text (for tests that want to count rows without
/// re-running the analyzer).
pub fn raw_corpus() -> &'static str {
    CORPUS_TSV
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Format parser unit tests ────────────────────────────────────

    #[test]
    fn parse_origin_handles_known_tags() {
        assert_eq!(parse_origin("protected"), Some(AnalysisOrigin::ProtectedList));
        assert_eq!(parse_origin("generative"), Some(AnalysisOrigin::GenerativeFst));
        assert_eq!(parse_origin("heuristic"), Some(AnalysisOrigin::SurfaceHeuristic));
        assert_eq!(parse_origin("foreign"), Some(AnalysisOrigin::SurfaceHeuristic));
    }

    #[test]
    fn parse_origin_rejects_unknown() {
        assert_eq!(parse_origin("wibble"), None);
        assert_eq!(parse_origin(""), None);
    }

    #[test]
    fn parse_optional_maps_dash_to_none() {
        assert_eq!(parse_optional("-"), None);
        assert_eq!(parse_optional(""), None);
        assert_eq!(parse_optional("كتاب"), Some("كتاب".to_string()));
    }

    #[test]
    fn parse_corpus_skips_comments_blanks_and_short_rows() {
        let input = "\
# header comment
\r
وائل\tprotected\tوائل\t-
# another comment
short\trow
الكاتب\tgenerative\tكاتب\tك-ت-ب
";
        let cases = parse_corpus(input);
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].surface, "وائل");
        assert_eq!(cases[0].origin, AnalysisOrigin::ProtectedList);
        assert_eq!(cases[0].expected_lemma, Some("وائل".to_string()));
        assert_eq!(cases[0].expected_root, None);
        assert_eq!(cases[1].surface, "الكاتب");
        assert_eq!(cases[1].origin, AnalysisOrigin::GenerativeFst);
        assert_eq!(cases[1].expected_root, Some("ك-ت-ب".to_string()));
    }

    #[test]
    fn parse_corpus_records_line_numbers() {
        let input = "# comment\n\n\nوائل\tprotected\tوائل\t-\n";
        let cases = parse_corpus(input);
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].line, 4);
    }

    // ── Corpus shape ────────────────────────────────────────────────

    #[test]
    fn corpus_has_expected_size() {
        let cases = parse_corpus(CORPUS_TSV);
        assert!(
            cases.len() >= 500,
            "regression corpus has {} cases, expected at least 500",
            cases.len()
        );
        assert!(
            cases.len() <= 2000,
            "regression corpus grew unexpectedly: {} cases — if this is \
             intentional, bump the upper bound",
            cases.len()
        );
    }

    #[test]
    fn corpus_covers_every_origin() {
        let cases = parse_corpus(CORPUS_TSV);
        let mut protected_n = 0usize;
        let mut generative_n = 0usize;
        let mut heuristic_n = 0usize;
        for c in &cases {
            match c.origin {
                AnalysisOrigin::ProtectedList => protected_n += 1,
                AnalysisOrigin::GenerativeFst => generative_n += 1,
                AnalysisOrigin::SurfaceHeuristic => heuristic_n += 1,
                AnalysisOrigin::UserOverride => {
                    // UserOverride is M8; no corpus rows for it yet.
                }
            }
        }
        assert!(
            protected_n >= 50,
            "expected at least 50 protected cases, got {protected_n}"
        );
        assert!(
            generative_n >= 100,
            "expected at least 100 generative cases, got {generative_n}"
        );
        assert!(
            heuristic_n >= 5,
            "expected at least 5 heuristic/foreign cases, got {heuristic_n}"
        );
    }

    #[test]
    fn corpus_has_unique_surfaces() {
        // Duplicate surfaces would mask failures (a later row's expected
        // values silently override an earlier row in diagnostic reports).
        let cases = parse_corpus(CORPUS_TSV);
        let mut seen = std::collections::HashSet::with_capacity(cases.len());
        for c in &cases {
            assert!(
                seen.insert(c.surface.clone()),
                "duplicate surface {:?} at line {}",
                c.surface,
                c.line
            );
        }
    }

    // ── Full-run scoring ────────────────────────────────────────────

    #[test]
    fn corpus_passes_with_full_score() {
        // Zero failures is the standing bar. When M7 (the disambiguator)
        // lands, any newly-wrong cases should be reviewed: either the
        // analyzer regressed (bug) or the expected row needs updating
        // (corpus drift). In either case, the human is looped in.
        let report = run_corpus();
        if !report.failed.is_empty() {
            let mut msg = format!(
                "corpus regression — {} / {} cases failed:\n",
                report.failed.len(),
                report.total()
            );
            for f in report.failed.iter().take(25) {
                msg.push_str(&format!(
                    "  line {:>4} {:>20}  —  {}\n",
                    f.case.line, f.case.surface, f.reason
                ));
            }
            if report.failed.len() > 25 {
                msg.push_str(&format!(
                    "  ... and {} more (showing first 25)\n",
                    report.failed.len() - 25
                ));
            }
            panic!("{msg}");
        }
        assert!(
            report.pass_rate() >= 0.999,
            "pass rate dropped to {:.3}",
            report.pass_rate()
        );
    }

    #[test]
    fn raw_corpus_accessor_returns_nonempty_tsv() {
        let s = raw_corpus();
        assert!(!s.is_empty());
        assert!(s.contains("\tprotected\t") || s.contains("\tgenerative\t"));
    }
}
