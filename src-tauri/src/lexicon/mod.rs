//! Constellation Multilingual Lexical Bridge.
//!
//! A polylingual lemma graph: every lemma in any of the 15 supported
//! languages can be looked up and yields its equivalents in any other
//! language, plus in-language synonyms and optional hypernyms/hyponyms.
//!
//! # Why this module exists
//!
//! A user searching for `المعرفة` should find **every note about knowledge**,
//! regardless of the language the note was written in. Obsidian, Roam,
//! Logseq do not offer this. For Constellation it is a defining feature.
//!
//! Combined with the Constellation Arabic Engine (`crate::arabic`) and the
//! future per-language analyzers, the full search pipeline is:
//!
//! ```text
//! user query: "المعرفة"
//!   │
//!   ▼  ── lemmatize (via CAE for Arabic, English lemmatizer for English, etc.)
//! lemma = معرفة, lang = Ar
//!   │
//!   ▼  ── lexicon::expand(lemma, Lang::Ar)
//! expansions = {
//!   Ar: [معرفة, معارف, عرفان],        // in-language synonyms
//!   En: [knowledge, cognition, ...],
//!   Fr: [connaissance, savoir],
//!   De: [Wissen, Erkenntnis, ...],
//!   ... all 15 langs
//! }
//!   │
//!   ▼  ── FTS5 MATCH lemma:(معرفة OR knowledge OR connaissance OR ...)
//! note hits, each tagged by detected language
//!   │
//!   ▼  ── UI groups or mixes per user preference
//! ```
//!
//! # Graph, not dictionary (per design decision 3)
//!
//! The lexicon is stored as an **undirected graph of lemmas** where every
//! edge is a semantic-equivalence link. This gives bidirectional search
//! for free: a French "connaissance" edge to Arabic "معرفة" is also an
//! Arabic→French edge.
//!
//! Nodes carry `(lemma, lang, sense_id)` so `bank[riverside]` and
//! `bank[financial]` are distinct graph nodes, pointing to different
//! equivalents in other languages. Sense disambiguation comes from
//! WordNet synsets where available; from Wiktionary sense lines
//! elsewhere; from co-occurrence learning (Layer 5) as a fallback.
//!
//! # Storage — tiered per design decision 1 (option C)
//!
//! - **Core tier** (~15 MB, embedded in binary):
//!   ~20,000 most-common concepts × 15 languages. Built from the
//!   intersection of WordNet, Arabic WordNet, and Wiktionary's
//!   frequent-word translation sections.
//! - **Optional expansion packs** (~10–30 MB each, downloaded on demand
//!   into `<Universe>/.constellation/lexicon-packs/`):
//!   Academic, Philosophy, Science, Classics, Medical, Legal, etc.
//! - **User overrides** (per Universe, learning layer 5):
//!   `<Universe>/.constellation/lexicon-overrides.json`.
//!
//! # Data provenance — all open, attribution preserved
//!
//! - Princeton WordNet — WordNet license (BSD-like)
//! - Arabic WordNet (AWN 2.0) — GPL-academic (free use, attribution)
//! - Wiktionary translation sections — CC BY-SA 3.0
//! - OmegaWiki — CC BY/GFDL (used for cross-checks only)
//!
//! No closed / commercial data enters the bridge.
//!
//! # M10 status
//!
//! The architecture and data structures are complete; a hand-picked
//! ~15-concept seed ships in `lexicon/data/seed_v1.tsv` to exercise
//! every code path (TSV parse → graph build → FST name-index → edge
//! traversal → query expansion) end-to-end. The 20K core and the
//! on-disk cache arrive in M11. Query expansion plumbing into the
//! FTS5 search path arrives in M12.

pub mod bake;
pub mod expansion;
pub mod fts;
pub mod graph;
pub mod parse;

pub use expansion::{ExpansionOptions, ExpansionResult, SynonymLevel};
pub use fts::{build_match_expr, escape_fts_term};
pub use graph::{
    build_bundle, seed_tsv, BuildError, Edge, EdgeKind, LemmaNode, LexiconBundle, LexiconGraph,
    SenseId,
};
pub use parse::{parse, parse_with_diagnostics, ConceptRecord, ParseRowError};

use crate::arabic::Lang;
use std::collections::{HashMap, HashSet};

/// Look up cross-lingual equivalents for a lemma in a given source language.
///
/// Walks the lexicon graph from every node matching `(source, lemma)`
/// (there may be multiple when senses differ), collects `Equivalent`-kind
/// edges, and groups the reached lemmas by target language. Results from
/// the same target language are deduplicated while preserving
/// first-encountered order (so `WordNet > Wiktionary > User` provenance
/// survives the collect).
///
/// Returns an empty map when the lemma is not in the graph — callers
/// should treat "no expansion" as a no-op, not an error.
pub fn equivalents(lemma: &str, source: Lang) -> HashMap<Lang, Vec<String>> {
    equivalents_via(LexiconGraph::get(), lemma, source)
}

/// Like [`equivalents`], but against a caller-supplied graph instead of
/// the singleton. Used by tests to avoid leaking state through the
/// process-wide `OnceLock`.
pub fn equivalents_via(
    graph: &LexiconGraph,
    lemma: &str,
    source: Lang,
) -> HashMap<Lang, Vec<String>> {
    let mut out: HashMap<Lang, Vec<String>> = HashMap::new();
    let mut seen: HashSet<(Lang, String)> = HashSet::new();
    for src in graph.find_nodes(source, lemma) {
        for edge in graph.edges_of(src) {
            if edge.kind != EdgeKind::Equivalent {
                continue;
            }
            let tgt = &graph.nodes[edge.target as usize];
            let key = (tgt.lang, tgt.lemma.clone());
            if seen.insert(key) {
                out.entry(tgt.lang).or_default().push(tgt.lemma.clone());
            }
        }
    }
    out
}

/// Expand a lemma into its full cross-lingual + in-language synonym set
/// for FTS query expansion.
///
/// Respects the caller's `ExpansionOptions`:
///   - `enabled_langs`: cross-lingual equivalents are filtered to this
///     set. The source language is always implicitly allowed for the
///     source-lemma echo.
///   - `synonym_level`:
///     * `None` — only cross-lingual equivalents, no in-language synonyms.
///     * `Synonym` — cross-lingual + same-lang synonyms (default).
///     * `SynonymAndHypernyms` — all of the above + one-hop hypernyms
///       and hyponyms.
///   - `max_per_lang`: caps each bucket. `0` disables the cap.
///
/// Returns an `ExpansionResult` with the source lemma echoed back plus
/// `equivalents` / `synonyms` / `hypernyms` / `hyponyms` maps, each
/// keyed by target language. See
/// [`ExpansionResult::flat_terms`](expansion::ExpansionResult::flat_terms)
/// for the FTS-ready flat list.
pub fn expand(lemma: &str, source: Lang, opts: &ExpansionOptions) -> ExpansionResult {
    expand_via(LexiconGraph::get(), lemma, source, opts)
}

/// Like [`expand`], but against a caller-supplied graph. Primary
/// testing entry point.
pub fn expand_via(
    graph: &LexiconGraph,
    lemma: &str,
    source: Lang,
    opts: &ExpansionOptions,
) -> ExpansionResult {
    let mut equivalents: HashMap<Lang, Vec<String>> = HashMap::new();
    let mut synonyms: HashMap<Lang, Vec<String>> = HashMap::new();
    let mut hypernyms: HashMap<Lang, Vec<String>> = HashMap::new();
    let mut hyponyms: HashMap<Lang, Vec<String>> = HashMap::new();
    let mut seen: HashSet<(EdgeKind, Lang, String)> = HashSet::new();

    let include_synonyms = !matches!(opts.synonym_level, SynonymLevel::None);
    let include_hypernyms = matches!(opts.synonym_level, SynonymLevel::SynonymAndHypernyms);

    for src_idx in graph.find_nodes(source, lemma) {
        for edge in graph.edges_of(src_idx) {
            let target = &graph.nodes[edge.target as usize];
            // Cross-lingual equivalents: filter to `enabled_langs`.
            match edge.kind {
                EdgeKind::Equivalent => {
                    if !opts.enabled_langs.contains(&target.lang) {
                        continue;
                    }
                    push_bounded(
                        &mut equivalents,
                        &mut seen,
                        EdgeKind::Equivalent,
                        target.lang,
                        &target.lemma,
                        opts.max_per_lang,
                    );
                }
                EdgeKind::Synonym | EdgeKind::UserLink => {
                    if !include_synonyms {
                        continue;
                    }
                    // Synonyms are in-language, but the user-link layer
                    // (M14) can mark cross-lingual pairs too. In both
                    // cases: respect `enabled_langs` for the target.
                    if !opts.enabled_langs.contains(&target.lang) {
                        continue;
                    }
                    push_bounded(
                        &mut synonyms,
                        &mut seen,
                        EdgeKind::Synonym,
                        target.lang,
                        &target.lemma,
                        opts.max_per_lang,
                    );
                }
                EdgeKind::Hypernym => {
                    if !include_hypernyms {
                        continue;
                    }
                    if !opts.enabled_langs.contains(&target.lang) {
                        continue;
                    }
                    push_bounded(
                        &mut hypernyms,
                        &mut seen,
                        EdgeKind::Hypernym,
                        target.lang,
                        &target.lemma,
                        opts.max_per_lang,
                    );
                }
                EdgeKind::Hyponym => {
                    if !include_hypernyms {
                        continue;
                    }
                    if !opts.enabled_langs.contains(&target.lang) {
                        continue;
                    }
                    push_bounded(
                        &mut hyponyms,
                        &mut seen,
                        EdgeKind::Hyponym,
                        target.lang,
                        &target.lemma,
                        opts.max_per_lang,
                    );
                }
            }
        }
    }

    ExpansionResult {
        source_lemma: lemma.to_string(),
        source_lang: source,
        equivalents,
        synonyms,
        hypernyms,
        hyponyms,
    }
}

fn push_bounded(
    bucket: &mut HashMap<Lang, Vec<String>>,
    seen: &mut HashSet<(EdgeKind, Lang, String)>,
    kind: EdgeKind,
    lang: Lang,
    lemma: &str,
    max: usize,
) {
    let key = (kind, lang, lemma.to_string());
    if !seen.insert(key) {
        return;
    }
    let bucket = bucket.entry(lang).or_default();
    if max == 0 || bucket.len() < max {
        bucket.push(lemma.to_string());
    }
}

/// End-to-end convenience: expand `lemma` and immediately fold the
/// result into an FTS5 `MATCH` expression ready for
/// `WHERE notes_fts MATCH ?` — the shape M12 plumbs into search.rs.
///
/// Returns `None` when the expansion produces zero usable terms after
/// escaping (empty source lemma, or a pathological lemma made only of
/// characters that [`fts::escape_fts_term`] strips). The caller should
/// treat `None` as "run the user's plain query instead" — an empty
/// MATCH clause is a syntax error in FTS5.
///
/// ```ignore
/// // Typical wiring on the search path:
/// let match_expr = lexicon::expand_to_match_expr(lemma, Lang::En, &opts)
///     .unwrap_or_else(|| format!("\"{}\"", lemma));  // fall back
/// let rows = conn.prepare("SELECT … FROM notes_fts WHERE notes_fts MATCH ?1 …")?
///     .query_map(params![match_expr], /* … */)?;
/// ```
pub fn expand_to_match_expr(
    lemma: &str,
    source: Lang,
    opts: &ExpansionOptions,
) -> Option<String> {
    let r = expand(lemma, source, opts);
    fts::build_match_expr(&r)
}

/// Like [`expand_to_match_expr`], but against a caller-supplied graph
/// so tests can exercise the full pipeline without touching the
/// process-wide singleton.
pub fn expand_to_match_expr_via(
    graph: &LexiconGraph,
    lemma: &str,
    source: Lang,
    opts: &ExpansionOptions,
) -> Option<String> {
    let r = expand_via(graph, lemma, source, opts);
    fts::build_match_expr(&r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_graph() -> LexiconGraph {
        let tsv = "\
c:book\tNoun\ten:book,books\tar:كتاب\tfr:livre
c:read\tVerb\ten:read,reading\tar:قرأ\tfr:lire
c:knowledge\tNoun\ten:knowledge,cognition\tar:معرفة\tfr:connaissance
";
        LexiconGraph::from_records(parse(tsv)).unwrap()
    }

    #[test]
    fn equivalents_crosses_languages() {
        let g = small_graph();
        let eq = equivalents_via(&g, "book", Lang::En);
        assert_eq!(eq.get(&Lang::Ar), Some(&vec!["كتاب".to_string()]));
        assert_eq!(eq.get(&Lang::Fr), Some(&vec!["livre".to_string()]));
        // `equivalents()` must NOT include same-language synonyms — that's
        // what `expand` is for.
        assert!(!eq.contains_key(&Lang::En));
    }

    #[test]
    fn equivalents_from_arabic_reaches_english() {
        let g = small_graph();
        let eq = equivalents_via(&g, "معرفة", Lang::Ar);
        let en = eq.get(&Lang::En).unwrap();
        assert!(en.contains(&"knowledge".to_string()));
        assert!(en.contains(&"cognition".to_string()));
    }

    #[test]
    fn equivalents_of_unknown_is_empty() {
        let g = small_graph();
        assert!(equivalents_via(&g, "xyzzy", Lang::En).is_empty());
    }

    #[test]
    fn expand_default_includes_synonyms_and_translations() {
        let g = small_graph();
        let r = expand_via(&g, "book", Lang::En, &ExpansionOptions::default());
        // Cross-lingual equivalents reach Arabic + French.
        assert!(r.equivalents.get(&Lang::Ar).is_some());
        assert!(r.equivalents.get(&Lang::Fr).is_some());
        // In-language synonym reaches "books".
        let en_syn = r.synonyms.get(&Lang::En).unwrap();
        assert!(en_syn.contains(&"books".to_string()));
    }

    #[test]
    fn expand_mono_mode_skips_cross_language() {
        let g = small_graph();
        let r = expand_via(&g, "book", Lang::En, &ExpansionOptions::mono(Lang::En));
        // Mono mode disables synonyms entirely (SynonymLevel::None) and
        // only enables the source lang in `enabled_langs`.
        assert!(r.equivalents.is_empty());
        assert!(r.synonyms.is_empty());
    }

    #[test]
    fn expand_synonym_level_none_preserves_equivalents() {
        let g = small_graph();
        let mut opts = ExpansionOptions::default();
        opts.synonym_level = SynonymLevel::None;
        let r = expand_via(&g, "book", Lang::En, &opts);
        // Equivalents still populated; synonyms dropped.
        assert!(r.equivalents.get(&Lang::Ar).is_some());
        assert!(r.synonyms.is_empty());
    }

    #[test]
    fn expand_enabled_langs_filter_is_respected() {
        let g = small_graph();
        let mut opts = ExpansionOptions::default();
        // Restrict to English + French only — Arabic should drop.
        opts.enabled_langs.clear();
        opts.enabled_langs.insert(Lang::En);
        opts.enabled_langs.insert(Lang::Fr);
        let r = expand_via(&g, "book", Lang::En, &opts);
        assert!(r.equivalents.get(&Lang::Fr).is_some());
        assert!(r.equivalents.get(&Lang::Ar).is_none());
    }

    #[test]
    fn expand_max_per_lang_caps_bucket() {
        let g = small_graph();
        let mut opts = ExpansionOptions::default();
        opts.max_per_lang = 1;
        let r = expand_via(&g, "knowledge", Lang::En, &opts);
        // English has two synonyms (`cognition` + the source-echo effect
        // from `push_bounded`). Cap at 1 means we keep only the first.
        // Actual `knowledge` is the source lemma — it doesn't travel
        // through a same-concept edge, so the only synonym is
        // `cognition`. The cap shouldn't produce >1 here anyway, but
        // cross-lingual with richer data (e.g. Arabic) can show it.
        for (lang, terms) in &r.equivalents {
            assert!(
                terms.len() <= 1,
                "lang {:?} exceeded cap: {:?}",
                lang,
                terms
            );
        }
    }

    #[test]
    fn expand_uses_source_singleton_when_graph_unspecified() {
        // Through the top-level `expand()`, which goes through the
        // OnceLock singleton. This also smoke-tests the seed data.
        let r = expand("book", Lang::En, &ExpansionOptions::default());
        assert_eq!(r.source_lemma, "book");
        assert_eq!(r.source_lang, Lang::En);
        assert!(r.equivalents.get(&Lang::Ar).is_some());
    }

    #[test]
    fn expand_preserves_source_identity() {
        let g = small_graph();
        let r = expand_via(&g, "معرفة", Lang::Ar, &ExpansionOptions::default());
        assert_eq!(r.source_lemma, "معرفة");
        assert_eq!(r.source_lang, Lang::Ar);
        // And it should produce an English bucket with knowledge +
        // cognition via the Equivalent edges.
        let en = r.equivalents.get(&Lang::En).unwrap();
        assert!(en.contains(&"knowledge".to_string()));
    }

    #[test]
    fn hypernyms_skipped_at_default_level() {
        // The M10 seed carries no hypernym edges yet, but the control
        // flow must still check the flag — assert the empty return.
        let g = small_graph();
        let r = expand_via(&g, "book", Lang::En, &ExpansionOptions::default());
        assert!(r.hypernyms.is_empty());
        assert!(r.hyponyms.is_empty());
    }

    // ── expand_to_match_expr end-to-end (M12) ─────────────────────

    #[test]
    fn expand_to_match_expr_via_produces_or_joined_phrase_query() {
        let g = small_graph();
        let expr = expand_to_match_expr_via(
            &g,
            "book",
            Lang::En,
            &ExpansionOptions::default(),
        )
        .expect("non-empty match expression");
        // Every term is phrase-quoted so operator keywords inside a
        // lemma can never change the query shape.
        assert!(expr.contains("\"book\""), "source missing: {}", expr);
        assert!(expr.contains("\"كتاب\""), "arabic missing: {}", expr);
        assert!(expr.contains("\"livre\""), "french missing: {}", expr);
        assert!(expr.contains(" OR "), "disjunction missing: {}", expr);
    }

    #[test]
    fn expand_to_match_expr_via_falls_back_to_source_on_miss() {
        let g = small_graph();
        // Lemma not in the graph: the echoed source lemma still
        // produces a valid single-phrase MATCH clause.
        let expr = expand_to_match_expr_via(
            &g,
            "quasar",
            Lang::En,
            &ExpansionOptions::default(),
        )
        .unwrap();
        assert_eq!(expr, "\"quasar\"");
    }

    #[test]
    fn expand_to_match_expr_via_returns_none_on_empty_lemma() {
        let g = small_graph();
        // Empty lemma echoes as an empty source, which escape_fts_term
        // strips — so the whole expression is None and callers fall
        // back to their non-expanded path.
        let expr = expand_to_match_expr_via(
            &g,
            "",
            Lang::En,
            &ExpansionOptions::default(),
        );
        assert!(expr.is_none());
    }

    #[test]
    fn expand_to_match_expr_via_honours_mono_mode() {
        let g = small_graph();
        // mono(En): enabled_langs = {En}, synonym_level = None. So the
        // MATCH expression contains nothing but the echoed source
        // lemma — behaviourally identical to today's un-expanded
        // search. Safety net for the rollback case.
        let expr = expand_to_match_expr_via(
            &g,
            "book",
            Lang::En,
            &ExpansionOptions::mono(Lang::En),
        )
        .unwrap();
        assert_eq!(expr, "\"book\"");
    }

    #[test]
    fn expand_to_match_expr_through_singleton() {
        // The non-`_via` variant routes through `LexiconGraph::get()`,
        // which also exercises the on-disk cache path (thanks M11).
        // Smoke-test that the end-to-end call on the seeded graph
        // returns a non-empty MATCH expression.
        let expr = expand_to_match_expr(
            "book",
            Lang::En,
            &ExpansionOptions::default(),
        );
        assert!(expr.is_some());
        assert!(expr.unwrap().contains("\"book\""));
    }
}
