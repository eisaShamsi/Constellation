//! Query-expansion types.
//!
//! When a user types a word into the search bar, the query flows through
//! the following expansion pipeline:
//!
//! 1. Lemmatize the query in its source language.
//! 2. Fetch the lemma's cross-lingual equivalents from the lexicon graph.
//! 3. Optionally add in-language synonyms (layer 2 — always on by default).
//! 4. Optionally add hypernyms / hyponyms (layer 3 — behind a toggle).
//! 5. Build an FTS5 `MATCH` expression covering all languages the user
//!    has enabled in settings (or filtered via the in-search toggle).
//!
//! The `ExpansionOptions` below carry user preferences; `ExpansionResult`
//! carries the full expanded set, broken down by relation type so the UI
//! can show "we also searched for: knowledge, cognition, connaissance, …".
//!
//! Per design decisions (2026-04-18):
//!   - All 15 languages supported, user picks which ones to include.
//!   - Synonyms on by default; hypernyms/hyponyms behind a toggle.
//!   - Bidirectional (the graph is undirected).
//!   - Results grouped or mixed, user's choice.
//!   - Settings default plus a quick in-search toggle.
//!   - Learning loop: user can mark an expansion "not for my Universe".

use crate::arabic::Lang;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Which tiers of synonym/hypernym relations to include in expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynonymLevel {
    /// No in-language synonyms; only cross-lingual translations.
    /// Used when the user wants precise searches and no semantic drift.
    None,
    /// Direct synonyms only (WordNet synset members). Default.
    /// Catches "knowledge ≈ cognition" but not "knowledge → science".
    Synonym,
    /// Synonyms plus one-hop hypernyms and hyponyms. Wider recall,
    /// lower precision. Useful for exploratory research.
    SynonymAndHypernyms,
}

impl Default for SynonymLevel {
    fn default() -> Self {
        SynonymLevel::Synonym
    }
}

/// User-controlled options for a single query expansion.
///
/// The default (`..Default::default()`) matches a new Universe's initial
/// settings: all 15 languages enabled, synonyms on, hypernyms off.
/// The search bar's quick toggle flips `enabled_langs` to
/// `{source_lang_only}` for a fast mono-lingual search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionOptions {
    /// Which target languages to expand into. If this set contains only
    /// the source language, the expansion is effectively mono-lingual
    /// but still benefits from in-language synonyms.
    pub enabled_langs: HashSet<Lang>,
    /// Which tiers of synonym/hypernym relations to include.
    pub synonym_level: SynonymLevel,
    /// Cap the number of equivalents per target language. Prevents
    /// runaway expansion when a lemma has many translations
    /// (e.g. "love" has ~40 in English alone across all senses).
    pub max_per_lang: usize,
}

impl Default for ExpansionOptions {
    fn default() -> Self {
        let mut langs = HashSet::with_capacity(15);
        for &l in Lang::all() {
            langs.insert(l);
        }
        Self {
            enabled_langs: langs,
            synonym_level: SynonymLevel::Synonym,
            max_per_lang: 8,
        }
    }
}

impl ExpansionOptions {
    /// Narrow the options to a single language — used by the in-search
    /// "🌐 off" quick toggle to disable cross-lingual expansion.
    pub fn mono(lang: Lang) -> Self {
        let mut langs = HashSet::with_capacity(1);
        langs.insert(lang);
        Self {
            enabled_langs: langs,
            synonym_level: SynonymLevel::None,
            max_per_lang: 0,
        }
    }
}

/// Result of expanding one query lemma through the lexical bridge.
///
/// Separated by relation type so the UI can explain the expansion to
/// the user ("we also searched for …") and offer per-expansion
/// refinement ("this translation is wrong for my Universe").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionResult {
    pub source_lemma: String,
    pub source_lang: Lang,
    /// Cross-lingual equivalents, keyed by target language.
    pub equivalents: HashMap<Lang, Vec<String>>,
    /// In-language synonyms, keyed by the language they're in.
    /// Usually just the source language, but for Arabic ↔ Hebrew it may
    /// be common for both languages to have synonyms inferred together.
    pub synonyms: HashMap<Lang, Vec<String>>,
    /// Hypernyms (broader concepts), included only when
    /// `synonym_level == SynonymAndHypernyms`.
    pub hypernyms: HashMap<Lang, Vec<String>>,
    /// Hyponyms (narrower concepts), included only when
    /// `synonym_level == SynonymAndHypernyms`.
    pub hyponyms: HashMap<Lang, Vec<String>>,
}

impl ExpansionResult {
    /// Flat list of all expansion terms paired with their language,
    /// ready to feed into FTS5 MATCH generation. Deduplicated.
    pub fn flat_terms(&self) -> Vec<(Lang, String)> {
        let mut out: Vec<(Lang, String)> = Vec::new();
        let mut seen: HashSet<(Lang, String)> = HashSet::new();
        let mut push = |lang: Lang, term: &str, out: &mut Vec<(Lang, String)>, seen: &mut HashSet<(Lang, String)>| {
            let key = (lang, term.to_string());
            if seen.insert(key.clone()) {
                out.push(key);
            }
        };
        push(self.source_lang, &self.source_lemma, &mut out, &mut seen);
        for (&lang, terms) in &self.equivalents {
            for t in terms { push(lang, t, &mut out, &mut seen); }
        }
        for (&lang, terms) in &self.synonyms {
            for t in terms { push(lang, t, &mut out, &mut seen); }
        }
        for (&lang, terms) in &self.hypernyms {
            for t in terms { push(lang, t, &mut out, &mut seen); }
        }
        for (&lang, terms) in &self.hyponyms {
            for t in terms { push(lang, t, &mut out, &mut seen); }
        }
        out
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_enables_all_languages() {
        let opts = ExpansionOptions::default();
        assert_eq!(opts.enabled_langs.len(), 15);
        assert!(matches!(opts.synonym_level, SynonymLevel::Synonym));
    }

    #[test]
    fn mono_mode_narrows_to_one() {
        let opts = ExpansionOptions::mono(Lang::Ar);
        assert_eq!(opts.enabled_langs.len(), 1);
        assert!(opts.enabled_langs.contains(&Lang::Ar));
        assert!(matches!(opts.synonym_level, SynonymLevel::None));
    }

    #[test]
    fn flat_terms_deduplicates_and_includes_source() {
        let mut r = ExpansionResult {
            source_lemma: "معرفة".to_string(),
            source_lang: Lang::Ar,
            equivalents: HashMap::new(),
            synonyms: HashMap::new(),
            hypernyms: HashMap::new(),
            hyponyms: HashMap::new(),
        };
        r.equivalents.insert(Lang::En, vec!["knowledge".to_string(), "cognition".to_string()]);
        r.synonyms.insert(Lang::Ar, vec!["معرفة".to_string(), "علم".to_string()]);
        let terms = r.flat_terms();
        // Expected: ("ar","معرفة"), ("en","knowledge"), ("en","cognition"), ("ar","علم").
        // The source-lemma synonym ("ar","معرفة") must not duplicate.
        assert_eq!(terms.len(), 4);
        assert!(terms.contains(&(Lang::Ar, "معرفة".to_string())));
        assert!(terms.contains(&(Lang::En, "knowledge".to_string())));
    }
}
