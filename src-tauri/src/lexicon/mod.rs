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

pub mod graph;
pub mod expansion;

pub use graph::{LemmaNode, LexiconGraph, SenseId};
pub use expansion::{ExpansionOptions, ExpansionResult, SynonymLevel};

use crate::arabic::Lang;
use std::collections::HashMap;

/// Look up cross-lingual equivalents for a lemma in a given source language.
///
/// This is the primary public API of the bridge. During early development
/// (pre-M11, before the lexicon data ships) this returns an empty map so
/// the analyzer and search layers can wire through without waiting on the
/// full dataset.
///
/// Replaced by the real graph lookup in M11.
pub fn equivalents(_lemma: &str, _source: Lang) -> HashMap<Lang, Vec<String>> {
    HashMap::new()
}

/// Expand a lemma into its full cross-lingual + in-language synonym set
/// for FTS query expansion.
///
/// Respects user settings (which languages are enabled, whether synonyms
/// are included, whether hypernyms/hyponyms are enabled per design
/// decision 2). The caller passes the current `ExpansionOptions`; the
/// bridge returns the expanded set as a flat `Vec<(Lang, String)>` ready
/// to feed into FTS MATCH.
///
/// Replaced by the real expander in M12.
pub fn expand(lemma: &str, source: Lang, _opts: &ExpansionOptions) -> ExpansionResult {
    ExpansionResult {
        source_lemma: lemma.to_string(),
        source_lang: source,
        equivalents: HashMap::new(),
        synonyms: HashMap::new(),
        hypernyms: HashMap::new(),
        hyponyms: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_empty_equivalents() {
        let m = equivalents("معرفة", Lang::Ar);
        assert!(m.is_empty());
    }

    #[test]
    fn stub_expand_preserves_source() {
        let opts = ExpansionOptions::default();
        let r = expand("knowledge", Lang::En, &opts);
        assert_eq!(r.source_lemma, "knowledge");
        assert_eq!(r.source_lang, Lang::En);
    }
}
