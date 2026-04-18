//! Layer 4 — disambiguator.
//!
//! Arabic is genuinely ambiguous. A single surface like `كاتب` arises from
//! multiple `(root × pattern)` combinations — at minimum the active
//! participle (Noun, Form I فَاعِل) and the Form III perfect verb
//! (كَاتَبَ "to correspond with"). The generative index faithfully produces
//! both; the FTS tokenizer and the Index UI need **one** lemma to write
//! into the `notes_fts` row, and the user deserves the same answer every
//! time regardless of FST serialization order.
//!
//! This module provides a deterministic, linguistically-informed ranking
//! that replaces the insertion-order tiebreak that `analyze_best` used
//! before M7. The ranking is pure: same input → same output — so the
//! regression corpus (`arabic::regression`) stays green on any origin
//! assertion and the Index UI shows stable lemma columns across restarts.
//!
//! ## V1 ranking key
//!
//! Analyses are ordered by the tuple:
//!
//!   1. **confidence** (higher first) — preserves the layer-1 vs layer-3
//!      dominance that `analyze()` encodes in the numeric confidence.
//!   2. **origin** (UserOverride → ProtectedList → GenerativeFst → SurfaceHeuristic) —
//!      breaks ties between confidence peers by trusting higher-authority
//!      layers first. In practice `origin` and `confidence` are correlated
//!      so this rarely flips an order, but when it does (e.g. a future M8
//!      UserOverride with confidence 0.85), the override wins.
//!   3. **POS rank** (ProperNoun → Noun → Adjective → Adverb → Verb → …) —
//!      for PKM context, nouns and named entities dominate notes. This is
//!      the key step for `كاتب`: the active participle (Noun) now wins
//!      deterministically over the Form III verb.
//!   4. **affix count** (fewer first) — a bare analysis is more direct
//!      evidence than one that peeled three prefixes. At equal everything
//!      else, prefer the simpler decomposition.
//!   5. **lemma** (alphabetic) — final deterministic tiebreak. Guarantees
//!      no two distinct analyses ever tie under `rank_analyses` — the
//!      first element is stable across refactors, OS-level hash RNG, and
//!      FST build order.
//!
//! ## Future v2 extensions (tracked, not shipping today)
//!
//! - **Corpus frequency**: count lemma occurrences in the user's Universe
//!   (via FTS5 vocab) and bias the rank toward more-seen forms. This is
//!   the "learn from the user's own writing" layer.
//! - **Context window**: use the 3-word window around the token at query
//!   time to pick between readings (e.g. `كاتب الرسالة` → Noun; `كاتب أخاه` → Verb).
//! - **User override**: once M8 ships, a UserOverride origin always wins
//!   regardless of other signals — that's already encoded in `origin_rank`.

use super::types::{Analysis, AnalysisOrigin, PartOfSpeech};

/// Authority rank of the origin layer. Lower is better.
///
/// `UserOverride` comes first because an explicit user choice must always
/// win over any engine-computed answer. `SurfaceHeuristic` is last because
/// it's the "we don't know" fallback — any real analysis beats it.
pub(crate) fn origin_rank(origin: AnalysisOrigin) -> u8 {
    match origin {
        AnalysisOrigin::UserOverride => 0,
        AnalysisOrigin::ProtectedList => 1,
        AnalysisOrigin::GenerativeFst => 2,
        AnalysisOrigin::SurfaceHeuristic => 3,
    }
}

/// POS rank for PKM context. Lower is better.
///
/// The ordering is opinionated: in notes, named entities and common nouns
/// dominate the interesting content. Verbs are action words — still common,
/// still indexable, but when a surface is ambiguous between noun and verb
/// readings, the noun reading is usually what the user meant. "كاتب" in a
/// note is almost always "a writer", not "to correspond with".
///
/// If a future user study shows a different distribution for their corpus,
/// the disambiguator's POS rank is the one place to adjust — the generator,
/// FST, and protected list stay untouched.
pub(crate) fn pos_rank(pos: PartOfSpeech) -> u8 {
    match pos {
        PartOfSpeech::ProperNoun => 0,
        PartOfSpeech::Noun => 1,
        PartOfSpeech::Adjective => 2,
        PartOfSpeech::Adverb => 3,
        PartOfSpeech::Verb => 4,
        PartOfSpeech::Particle => 5,
        PartOfSpeech::Foreign => 6,
        PartOfSpeech::Unknown => 7,
    }
}

/// Sort analyses in place, best first.
///
/// No-op on empty or single-element slices (sort is idempotent). After
/// this call, `analyses[0]` is the answer `analyze_best` hands to the
/// FTS tokenizer, and `analyses[1..]` are the alternatives the UI could
/// surface in a "did you mean" flyout.
///
/// Comparator avoids `partial_cmp` on `f32` (which returns `Option<Ordering>`);
/// instead we call `partial_cmp` once and fall back to `Equal` on NaN. In
/// practice confidence is always a clamped `f32` in `[0.0, 1.0]` set by
/// the generator, so NaN can only arise from future code-paths that forget
/// the invariant — and even then we degrade gracefully.
pub(crate) fn rank_analyses(analyses: &mut [Analysis]) {
    analyses.sort_by(|a, b| {
        // 1. Confidence desc.
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            // 2. Origin asc (UserOverride first).
            .then_with(|| origin_rank(a.origin).cmp(&origin_rank(b.origin)))
            // 3. POS rank asc (ProperNoun / Noun first).
            .then_with(|| pos_rank(a.pos).cmp(&pos_rank(b.pos)))
            // 4. Affix count asc (fewer peelings = more direct).
            .then_with(|| {
                let ca = a.prefixes.len() + a.suffixes.len();
                let cb = b.prefixes.len() + b.suffixes.len();
                ca.cmp(&cb)
            })
            // 5. Lemma alphabetic — deterministic final tiebreak.
            .then_with(|| a.lemma.cmp(&b.lemma))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arabic::types::{Analysis, Lang};
    use std::collections::HashMap;

    /// Helper: build a minimal `Analysis` for ranking tests. Fields that
    /// don't participate in the sort key are set to `Default`-equivalent
    /// sentinel values so the test's intent is obvious.
    fn mk(
        lemma: &str,
        confidence: f32,
        origin: AnalysisOrigin,
        pos: PartOfSpeech,
        n_prefixes: usize,
        n_suffixes: usize,
    ) -> Analysis {
        Analysis {
            surface: lemma.to_string(),
            lemma: lemma.to_string(),
            root: String::new(),
            pattern_label: String::new(),
            pos,
            prefixes: vec![super::super::types::AffixFunction::DefiniteAl; n_prefixes],
            suffixes: vec![super::super::types::AffixFunction::PronounHa; n_suffixes],
            confidence,
            origin,
            equivalents: HashMap::new(),
            lang: Lang::Ar,
        }
    }

    #[test]
    fn origin_rank_puts_user_override_first() {
        assert!(origin_rank(AnalysisOrigin::UserOverride) < origin_rank(AnalysisOrigin::ProtectedList));
        assert!(origin_rank(AnalysisOrigin::ProtectedList) < origin_rank(AnalysisOrigin::GenerativeFst));
        assert!(origin_rank(AnalysisOrigin::GenerativeFst) < origin_rank(AnalysisOrigin::SurfaceHeuristic));
    }

    #[test]
    fn pos_rank_puts_proper_noun_first_then_noun() {
        assert!(pos_rank(PartOfSpeech::ProperNoun) < pos_rank(PartOfSpeech::Noun));
        assert!(pos_rank(PartOfSpeech::Noun) < pos_rank(PartOfSpeech::Verb));
        assert!(pos_rank(PartOfSpeech::Verb) < pos_rank(PartOfSpeech::Particle));
    }

    #[test]
    fn rank_prefers_higher_confidence() {
        let mut v = vec![
            mk("low",  0.65, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
            mk("high", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Verb, 0, 0),
        ];
        rank_analyses(&mut v);
        assert_eq!(v[0].lemma, "high", "higher confidence must win regardless of POS");
    }

    #[test]
    fn rank_prefers_protected_over_fst_at_equal_confidence() {
        let mut v = vec![
            mk("fst",       0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
            mk("protected", 0.85, AnalysisOrigin::ProtectedList, PartOfSpeech::Noun, 0, 0),
        ];
        rank_analyses(&mut v);
        assert_eq!(v[0].lemma, "protected", "protected wins at equal confidence");
    }

    #[test]
    fn rank_prefers_user_override_over_everything_at_equal_confidence() {
        let mut v = vec![
            mk("protected", 0.85, AnalysisOrigin::ProtectedList, PartOfSpeech::Noun, 0, 0),
            mk("override",  0.85, AnalysisOrigin::UserOverride,  PartOfSpeech::Verb, 2, 1),
        ];
        rank_analyses(&mut v);
        assert_eq!(v[0].lemma, "override", "user override wins even with more affixes and worse POS");
    }

    #[test]
    fn rank_prefers_noun_over_verb_at_equal_confidence_and_origin() {
        // The كاتب case: same confidence, same origin, one is Noun (active
        // participle), the other is Verb (Form III perfect). PKM context
        // wants the noun reading.
        let mut v = vec![
            mk("katib-verb", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Verb, 0, 0),
            mk("katib-noun", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
        ];
        rank_analyses(&mut v);
        assert_eq!(v[0].lemma, "katib-noun", "noun reading wins the POS tiebreak");
    }

    #[test]
    fn rank_prefers_fewer_affixes_at_equal_everything_else() {
        let mut v = vec![
            mk("peeled", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 2, 1),
            mk("bare",   0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
        ];
        rank_analyses(&mut v);
        assert_eq!(v[0].lemma, "bare", "fewer affixes = more direct analysis, wins");
    }

    #[test]
    fn rank_is_alphabetic_at_full_tie() {
        let mut v = vec![
            mk("zebra", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
            mk("apple", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
        ];
        rank_analyses(&mut v);
        assert_eq!(v[0].lemma, "apple", "alphabetic lemma is the final deterministic tiebreak");
    }

    #[test]
    fn rank_is_idempotent() {
        let mut v = vec![
            mk("b", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Verb, 0, 0),
            mk("a", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
            mk("c", 0.65, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
        ];
        rank_analyses(&mut v);
        let once: Vec<String> = v.iter().map(|a| a.lemma.clone()).collect();
        rank_analyses(&mut v);
        let twice: Vec<String> = v.iter().map(|a| a.lemma.clone()).collect();
        assert_eq!(once, twice, "sorting twice must produce the same order");
    }

    #[test]
    fn rank_handles_empty_and_single_element_slices() {
        let mut empty: Vec<Analysis> = Vec::new();
        rank_analyses(&mut empty);
        assert!(empty.is_empty());

        let mut one = vec![mk("solo", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0)];
        rank_analyses(&mut one);
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].lemma, "solo");
    }

    #[test]
    fn rank_handles_nan_confidence_without_panic() {
        // Invariant violation (confidence should be clamped by the generator)
        // but we must degrade gracefully. NaN → partial_cmp returns None →
        // fallback Ordering::Equal — so the remaining keys decide.
        let mut v = vec![
            mk("nan", f32::NAN, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
            mk("ok",  0.85,     AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
        ];
        rank_analyses(&mut v); // must not panic
        // Order isn't pinned (NaN means "Equal" for confidence key; subsequent
        // keys are identical; final alphabetic wins → "nan" < "ok"). The
        // contract is just "doesn't panic".
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn rank_lower_confidence_never_overtakes_at_any_pos() {
        // Even if a 0.65-confidence entry is a Noun and a 0.85-confidence
        // entry is a Verb, the 0.85 must still come first. Confidence is
        // the dominant key — the rest only breaks ties.
        let mut v = vec![
            mk("0.65-noun", 0.65, AnalysisOrigin::GenerativeFst, PartOfSpeech::Noun, 0, 0),
            mk("0.85-verb", 0.85, AnalysisOrigin::GenerativeFst, PartOfSpeech::Verb, 0, 0),
        ];
        rank_analyses(&mut v);
        assert_eq!(v[0].lemma, "0.85-verb");
    }
}
