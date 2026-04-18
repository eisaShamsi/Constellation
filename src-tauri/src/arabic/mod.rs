//! Constellation Arabic Engine (CAE)
//!
//! A ground-up Arabic morphological analyzer purpose-built for the
//! Constellation PKM. Replaces the Larkey Light10 surface stemmer with
//! a five-layer generative pipeline that understands root × pattern
//! morphology without relying on Buckwalter's 40,000-entry dictionary
//! or any GPL-encumbered data.
//!
//! ## Why a new engine
//!
//! Light10 over-strips common Arabic names (وائل → ائل) and cannot
//! decompose broken plurals (الأئمة → ائم). Buckwalter is GPL-v2 and
//! 40 MB RSS. Farasa / MADAMIRA require a JVM and tens of megabytes of
//! ML models. None are acceptable for an embedded, local-first PKM.
//!
//! ## Architecture — five layers
//!
//! Input (raw Arabic word)
//!    ↓
//! [L1 normalizer]        ← tashkeel / tatweel removal, hamza variants,
//!                          language detection; preserves surface form
//!    ↓
//! [L2 protected list]    ← ~20K proper nouns + loanwords (hash lookup).
//!                          Hit → return verbatim, no further analysis.
//!                          This is where `وائل` is saved.
//!    ↓
//! [L3 generative FST]    ← rolling-hash + FST over all (root × pattern)
//!                          combinations. Returns zero-or-more analyses.
//!                          This is where `الأئمة` decomposes to إمام/أمم.
//!    ↓
//! [L4 disambiguator]     ← ranks multiple analyses by corpus frequency,
//!                          context, and user-Universe history
//!    ↓
//! [L5 user overrides]    ← per-Universe learning layer
//!                          (`<Universe>/.constellation/arabic-overrides.json`)
//!    ↓
//! Output (Vec<Analysis>, best-ranked first)
//!
//! ## FTS5 integration
//!
//! Per قرار 1 (ج), the FTS schema stores three Arabic fields per token:
//!   - `surface` (display, what user typed)
//!   - `lemma` (dictionary headword, primary search key)
//!   - `root` (ك-ت-ب form, for root-based power search)
//!
//! ## License
//!
//! All linguistic data is derived from public-domain sources (Lisān al-ʿArab,
//! Qāmūs al-Muḥīṭ, classical grammar treatises) or CC BY-SA Wikipedia
//! extractions. No Buckwalter / BAMA / SAMA content is used. Constellation
//! retains its own license; the Arabic engine ships under the same terms.

pub mod types;
pub mod patterns;
pub mod affixes;
pub mod normalizer;
pub mod protected;
pub mod roots;
pub mod generator;
pub mod fst_index;
// pub mod analyzer;      // M4
// pub mod disambiguator; // M7
// pub mod overrides;     // M8

pub use types::{
    Affix, AffixFunction, AffixSlot, Analysis, AnalysisOrigin, Lang, PartOfSpeech, Pattern,
    PatternKind, Root, RootClass,
};

use std::collections::HashMap;

/// Public entry point for the engine.
///
/// Runs the layers we have implemented so far:
///
///   1. **Normalizer** — strip tashkeel + tatweel, classify script.
///      Non-Arabic scripts are returned verbatim as `Foreign`.
///   2. **Protected list** — hash lookup on the stripped surface. A hit
///      returns a single high-confidence `Analysis` and short-circuits.
///   3. **Generative index** — `fst_index::GenerativeFst` over all
///      (seed root × 158 patterns) combinations. Two-level lookup:
///      exact stripped (confidence 0.85) then folded (confidence 0.65).
///      BurntSushi [`fst::Map`] backed as of M3 — mmap-ready, shares
///      prefixes across Arabic morphology, orders of magnitude smaller
///      at scale than the HashMap-backed `generator::GenerativeIndex`.
///   3b. **Affix-stripping cascade** — if step 3 missed, enumerate every
///      legal `(prefix chain, stem, suffix chain)` split of the surface
///      via `affixes::peel` and look each stem up in the generative
///      index. Confidence 0.75 (stripped stem) or 0.55 (folded stem).
///      This is how الكاتب → ال + كاتب → root ك-ت-ب; likewise
///      كتابها → كتاب + ها.
///
/// Layers 4–5 (disambiguator + user overrides) are milestones M7/M8.
/// When nothing in Layers 1–3 hits, we fall back to a low-confidence
/// `SurfaceHeuristic` that hands the caller the normalized surface as
/// both lemma and root-less analysis — this preserves correctness for
/// the FTS tokenizer while keeping the wire format forward-compatible.
pub fn analyze(word: &str) -> Vec<Analysis> {
    if word.is_empty() {
        return Vec::new();
    }

    // ── Layer 1: normalize ──────────────────────────────────────────
    let norm = normalizer::normalize(word);

    // Non-Arabic scripts bypass the Arabic pipeline — they're passed
    // through unchanged, tagged Foreign. The lexical bridge will still
    // link them across languages at search time.
    match norm.script {
        normalizer::Script::Latin
        | normalizer::Script::Hebrew
        | normalizer::Script::Other => {
            return vec![Analysis {
                surface: word.to_string(),
                lemma: norm.stripped.clone(),
                root: String::new(),
                pattern_label: "non-arabic".to_string(),
                pos: PartOfSpeech::Foreign,
                prefixes: Vec::new(),
                suffixes: Vec::new(),
                confidence: 0.8,
                origin: AnalysisOrigin::SurfaceHeuristic,
                equivalents: HashMap::new(),
                lang: Lang::Ar,
            }];
        }
        normalizer::Script::Empty => return Vec::new(),
        normalizer::Script::Arabic | normalizer::Script::PersianFamily => {}
    }

    // ── Layer 2: protected list ─────────────────────────────────────
    if let Some(entry) = protected::lookup(&norm.stripped) {
        return vec![entry.to_analysis(word)];
    }

    // ── Layer 3: generative index (FST-backed as of M3) ─────────────
    //
    // The GenerativeFst is seeded from `generator::generate_all()` —
    // every (root × pattern) combination the linguistic core produces.
    // Hits here are real morphological analyses, so confidence is much
    // higher than the heuristic fallback.
    //
    // First try the exact stripped form (preserves hamza carriers,
    // alif/ya variants, tā' marbūṭa). If that misses, fall back to the
    // folded form — aggressive normalization that matches Light10's
    // tolerance but without its over-stripping damage.
    let idx = fst_index::GenerativeFst::get();

    let stripped_hits = idx.lookup(&norm.stripped);
    if !stripped_hits.is_empty() {
        return stripped_hits
            .iter()
            .map(|form| Analysis {
                surface: word.to_string(),
                lemma: form.surface.clone(),
                root: form.root_key.clone(),
                pattern_label: form.pattern_label.clone(),
                pos: generator::pos_for_kind(form.pattern_kind),
                prefixes: Vec::new(),
                suffixes: Vec::new(),
                confidence: 0.85,
                origin: AnalysisOrigin::GenerativeFst,
                equivalents: HashMap::new(),
                lang: Lang::Ar,
            })
            .collect();
    }

    let folded_hits = idx.lookup_folded(&norm.folded);
    if !folded_hits.is_empty() {
        return folded_hits
            .iter()
            .map(|form| Analysis {
                surface: word.to_string(),
                lemma: form.surface.clone(),
                root: form.root_key.clone(),
                pattern_label: form.pattern_label.clone(),
                pos: generator::pos_for_kind(form.pattern_kind),
                prefixes: Vec::new(),
                suffixes: Vec::new(),
                // Folded match is fuzzier — penalise confidence so the
                // disambiguator prefers any stripped-match neighbour.
                confidence: 0.65,
                origin: AnalysisOrigin::GenerativeFst,
                equivalents: HashMap::new(),
                lang: Lang::Ar,
            })
            .collect();
    }

    // ── Layer 3b: affix-stripping cascade ───────────────────────────
    //
    // The bare stripped/folded lookups missed. Now try every legal
    // decomposition of the stripped surface into (prefix chain + stem +
    // suffix chain), and look up each stem in the generative index.
    //
    // Example: الكاتب → peel ال → lookup كاتب → hit (Form I active
    // participle of ك-ت-ب). The Analysis carries the peeled prefix/suffix
    // lists so the UI/disambiguator knows the morphological decomposition.
    //
    // Confidence: 0.75 for stripped-stem peelings (lower than bare 0.85
    // because we did more work — more ways to be wrong), 0.55 for folded.
    let mut peel_analyses: Vec<Analysis> = Vec::new();
    for candidate in affixes::peel(&norm.stripped) {
        // Skip the degenerate (no-peeling) case — we already tried the
        // bare surface above, no point duplicating analyses.
        if candidate.prefixes.is_empty() && candidate.suffixes.is_empty() {
            continue;
        }

        // Try stripped remainder first.
        let stem_hits = idx.lookup(&candidate.remainder);
        let source_hits: Vec<_> = if !stem_hits.is_empty() {
            stem_hits.iter().map(|f| (f.clone(), 0.75)).collect()
        } else {
            // Fall back to folded remainder.
            let folded_remainder = normalizer::normalize_folded(&candidate.remainder);
            let folded_stem_hits = idx.lookup_folded(&folded_remainder);
            if folded_stem_hits.is_empty() {
                continue;
            }
            folded_stem_hits.iter().map(|f| (f.clone(), 0.55)).collect()
        };

        for (form, conf) in source_hits {
            peel_analyses.push(Analysis {
                surface: word.to_string(),
                lemma: form.surface.clone(),
                root: form.root_key.clone(),
                pattern_label: form.pattern_label.clone(),
                pos: generator::pos_for_kind(form.pattern_kind),
                // Analysis stores AffixFunction (semantic) not Affix
                // (surface) — the caller wants to know "this has a
                // definite article", not which spelling of it. Map:
                prefixes: candidate.prefixes.iter().map(|a| a.function.clone()).collect(),
                suffixes: candidate.suffixes.iter().map(|a| a.function.clone()).collect(),
                confidence: conf,
                origin: AnalysisOrigin::GenerativeFst,
                equivalents: HashMap::new(),
                lang: Lang::Ar,
            });
        }
    }

    if !peel_analyses.is_empty() {
        // Deduplicate on (root, pattern_label, prefix_chain, suffix_chain) —
        // the same remainder can come from multiple peelings that happen to
        // produce the same stripped form (e.g. with/without a و if و is the
        // first letter of the stem anyway).
        peel_analyses.sort_by(|a, b| {
            a.root
                .cmp(&b.root)
                .then_with(|| a.pattern_label.cmp(&b.pattern_label))
                .then_with(|| a.prefixes.len().cmp(&b.prefixes.len()))
                .then_with(|| a.suffixes.len().cmp(&b.suffixes.len()))
        });
        peel_analyses.dedup_by(|a, b| {
            a.root == b.root
                && a.pattern_label == b.pattern_label
                && a.prefixes.len() == b.prefixes.len()
                && a.suffixes.len() == b.suffixes.len()
        });
        return peel_analyses;
    }

    // ── Layer 4/5 (disambiguator, user overrides): not yet wired ────
    //
    // Fall back to surface heuristic: return the normalized surface as
    // both lemma and root-less analysis. Confidence 0.3 signals "we
    // looked but couldn't really analyze" — the disambiguator (M7) will
    // prefer higher-confidence index matches once Layer 4 lands.
    vec![Analysis {
        surface: word.to_string(),
        lemma: norm.stripped,
        root: String::new(),
        pattern_label: "heuristic".to_string(),
        pos: PartOfSpeech::Unknown,
        prefixes: Vec::new(),
        suffixes: Vec::new(),
        confidence: 0.3,
        origin: AnalysisOrigin::SurfaceHeuristic,
        equivalents: HashMap::new(),
        lang: Lang::Ar,
    }]
}

/// Convenience: returns the single best analysis (highest confidence),
/// or a verbatim stub if the analyzer produced nothing.
///
/// This is the primary hook for the FTS5 tokenizer, which does not care
/// about ambiguity — it just wants `(surface, lemma, root)` to index.
pub fn analyze_best(word: &str) -> Analysis {
    let mut analyses = analyze(word);
    if analyses.is_empty() {
        return Analysis {
            surface: word.to_string(),
            lemma: word.to_string(),
            root: String::new(),
            pattern_label: "none".to_string(),
            pos: PartOfSpeech::Unknown,
            prefixes: Vec::new(),
            suffixes: Vec::new(),
            confidence: 0.0,
            origin: AnalysisOrigin::SurfaceHeuristic,
            equivalents: HashMap::new(),
            lang: Lang::Ar,
        };
    }
    analyses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    analyses.into_iter().next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── critical: the وائل case now goes through the protected list ─

    #[test]
    fn wael_flows_through_protected_layer() {
        // This is the flagship bug that motivated the whole engine.
        // Light10 returned ائل. Our pipeline must return وائل with
        // ProtectedList origin and full confidence.
        let a = analyze_best("وائل");
        assert_eq!(a.surface, "وائل");
        assert_eq!(a.lemma, "وائل");
        assert_eq!(a.confidence, 1.0);
        assert!(matches!(a.origin, AnalysisOrigin::ProtectedList));
        assert_eq!(a.pos, PartOfSpeech::ProperNoun);
    }

    #[test]
    fn vocalized_wael_also_flows_through_protected_layer() {
        // User typed fully vocalized "وَائِل" — normalizer strips the
        // tashkeel, protected layer matches the bare form.
        let a = analyze_best("وَائِل");
        assert_eq!(a.surface, "وَائِل", "surface preserves user's vocalization");
        assert_eq!(a.lemma, "وائل", "lemma is the stripped canonical form");
        assert!(matches!(a.origin, AnalysisOrigin::ProtectedList));
    }

    #[test]
    fn alaimma_flagship_resolves_through_cascade() {
        // THE flagship case. Light10 returned ائم (broken-plural
        // decomposition failure). Our five-layer pipeline must:
        //   1. Normalize الأئمة.
        //   2. Fail the protected-list lookup (it's not a proper noun).
        //   3. Fail the bare generative lookup (strip doesn't include ال).
        //   4. Peel ال in Layer 3b → remainder أئمة.
        //   5. Look up أئمة in the generative index → hit on
        //      (ء-م-م, أَفْعِلَة broken plural) thanks to the M2.b
        //      geminated-fusion + hamza-carrier pipeline.
        //   6. Return the analysis with root ء-م-م, origin GenerativeFst,
        //      and the definite-article prefix recorded.
        let analyses = analyze("الأئمة");
        let found = analyses.iter().any(|a| {
            a.root == "ء-م-م"
                && a.prefixes.contains(&AffixFunction::DefiniteAl)
                && matches!(a.origin, AnalysisOrigin::GenerativeFst)
        });
        assert!(
            found,
            "expected (ال + أئمة → ء-م-م, broken-plural) analysis; got {:?}",
            analyses
                .iter()
                .map(|a| (
                    a.prefixes.clone(),
                    a.root.clone(),
                    a.pattern_label.clone(),
                    format!("{:?}", a.origin),
                ))
                .collect::<Vec<_>>()
        );
        // Surface must round-trip verbatim — no user-visible mutation.
        let best = analyze_best("الأئمة");
        assert_eq!(best.surface, "الأئمة");
    }

    // ── Layer 3b (affix cascade) regression ─────────────────────────

    #[test]
    fn definite_prefix_cascade_resolves_alkatib() {
        // الكاتب is ال (definite article) + كاتب (active participle of
        // ك-ت-ب). The cascade must peel ال and route the remainder
        // through Layer 3 to recover the root.
        let analyses = analyze("الكاتب");
        let found = analyses.iter().any(|a| {
            a.root == "ك-ت-ب"
                && a.prefixes.contains(&AffixFunction::DefiniteAl)
                && matches!(a.origin, AnalysisOrigin::GenerativeFst)
        });
        assert!(
            found,
            "expected (ال + كاتب → ك-ت-ب) cascade analysis; got {:?}",
            analyses
                .iter()
                .map(|a| (
                    a.prefixes.clone(),
                    a.root.clone(),
                    a.suffixes.clone(),
                    format!("{:?}", a.origin),
                ))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn conjunction_prefix_cascade_resolves_fakatib() {
        // فكاتب → ف + كاتب (conjunction "then" + active participle).
        let analyses = analyze("فكاتب");
        let found = analyses.iter().any(|a| {
            a.root == "ك-ت-ب"
                && a.prefixes.contains(&AffixFunction::ConjunctionThen)
                && matches!(a.origin, AnalysisOrigin::GenerativeFst)
        });
        assert!(found, "expected ف + كاتب cascade; got {:?}", analyses);
    }

    #[test]
    fn cascade_confidence_is_below_bare_hit() {
        // A bare Layer 3 hit (e.g. كاتب) must always score higher than
        // the same root recovered through peeling (e.g. الكاتب). This
        // invariant lets the disambiguator prefer direct matches.
        let bare = analyze_best("كاتب");
        let peeled = analyze_best("الكاتب");
        assert!(
            bare.confidence > peeled.confidence,
            "bare {} should be higher confidence than peeled {}",
            bare.confidence, peeled.confidence
        );
    }

    #[test]
    fn cascade_preserves_surface_verbatim() {
        // No matter how much peeling happens internally, the returned
        // Analysis.surface must be exactly what the user typed.
        let a = analyze_best("الكاتب");
        assert_eq!(a.surface, "الكاتب");
    }

    #[test]
    fn cascade_does_not_regress_non_peelable_words() {
        // كاتب on its own must still take the bare (prefix-free) path,
        // NOT be "peeled" into (ك + اتب) or similar spurious splits.
        let a = analyze_best("كاتب");
        assert!(
            a.prefixes.is_empty() && a.suffixes.is_empty(),
            "bare كاتب must not acquire phantom affixes; got prefixes={:?} suffixes={:?}",
            a.prefixes, a.suffixes
        );
    }

    // ── Layer 3 (generative) regression ─────────────────────────────

    #[test]
    fn bare_generative_hit_returns_generative_origin() {
        // كاتب (stripped) is genuinely ambiguous — it maps to both the
        // active participle (Noun) كَاتِب and the Form III perfect verb
        // كَاتَبَ ("to correspond with"). The analyzer returns ALL hits;
        // analyze_best picks first among equal-confidence results. The
        // guarantees worth asserting here are:
        //   1. origin is GenerativeFst (Layer 3 fired)
        //   2. root is ك-ت-ب (the structural discovery)
        //   3. confidence is high (not heuristic)
        // PoS is left to the disambiguator (M7) to pick via context.
        let a = analyze_best("كاتب");
        assert!(
            matches!(a.origin, AnalysisOrigin::GenerativeFst),
            "expected GenerativeFst origin for كاتب, got {:?}",
            a.origin
        );
        assert_eq!(a.root, "ك-ت-ب", "كاتب → root ك-ت-ب");
        assert!(
            a.confidence >= 0.8,
            "stripped generative hit should be high-confidence, got {}",
            a.confidence
        );
    }

    #[test]
    fn katib_ambiguity_surfaces_both_pos() {
        // Stronger version of the above: analyze() must return multiple
        // analyses for كاتب (at minimum the active-participle Noun and
        // the Form III Verb), so the disambiguator has real alternatives
        // to rank.
        let analyses = analyze("كاتب");
        let has_noun = analyses.iter().any(|a| a.pos == PartOfSpeech::Noun);
        let has_verb = analyses.iter().any(|a| a.pos == PartOfSpeech::Verb);
        assert!(
            has_noun && has_verb,
            "كاتب should yield both Noun and Verb analyses; got {:?}",
            analyses.iter().map(|a| a.pos).collect::<Vec<_>>()
        );
    }

    #[test]
    fn generative_hit_preserves_surface_for_display() {
        // The engine must always return `surface` = what the user typed,
        // verbatim. Normalization only happens internally for lookup —
        // the display contract is "show the user their own text".
        let a = analyze_best("كَاتِب");
        assert_eq!(
            a.surface, "كَاتِب",
            "surface must preserve user's vocalization"
        );
        assert!(matches!(a.origin, AnalysisOrigin::GenerativeFst));
    }

    #[test]
    fn generative_hit_returns_all_analyses() {
        // A single surface form can arise from multiple (root, pattern)
        // combinations. analyze() returns ALL of them, the disambiguator
        // (M7) picks between them at query time.
        let analyses = analyze("كاتب");
        assert!(
            !analyses.is_empty(),
            "كاتب must produce at least one analysis"
        );
        for a in &analyses {
            assert!(matches!(a.origin, AnalysisOrigin::GenerativeFst));
        }
    }

    #[test]
    fn passive_participle_flows_through_generative_layer() {
        // مكتوب is the passive participle of ك-ت-ب via مَفْعُول. Like
        // كاتب, it must be traceable through Layer 3 to its root.
        let a = analyze_best("مكتوب");
        assert!(
            matches!(a.origin, AnalysisOrigin::GenerativeFst),
            "expected GenerativeFst origin for مكتوب, got {:?}",
            a.origin
        );
        assert_eq!(a.root, "ك-ت-ب");
    }

    #[test]
    fn quadriliteral_verb_flows_through_generative_layer() {
        // دحرج is the Form I perfect of the quadriliteral root
        // د-ح-ر-ج — the classic "to roll" verb. Demonstrates that the
        // pipeline handles 4-radical roots, not just triliterals.
        let a = analyze_best("دحرج");
        assert!(
            matches!(a.origin, AnalysisOrigin::GenerativeFst),
            "expected GenerativeFst origin for دحرج, got {:?}",
            a.origin
        );
        assert_eq!(a.root, "د-ح-ر-ج");
        assert_eq!(a.pos, PartOfSpeech::Verb);
    }

    #[test]
    fn english_token_is_foreign() {
        let a = analyze_best("Hello");
        assert_eq!(a.surface, "Hello");
        assert_eq!(a.pos, PartOfSpeech::Foreign);
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        assert!(analyze("").is_empty());
    }

    #[test]
    fn whitespace_input_returns_empty_vec() {
        // Normalizer classifies whitespace as Empty → bail out.
        assert!(analyze("   ").is_empty());
    }

    #[test]
    fn common_names_are_protected() {
        for name in ["محمد", "فاطمة", "القاهرة", "إنترنت"] {
            let a = analyze_best(name);
            assert!(
                matches!(a.origin, AnalysisOrigin::ProtectedList),
                "{name} should come from protected list, got {:?}",
                a.origin
            );
            assert_eq!(a.confidence, 1.0);
        }
    }

    #[test]
    fn loanword_has_foreign_pos() {
        let a = analyze_best("إنترنت");
        assert_eq!(a.pos, PartOfSpeech::Foreign);
    }

    #[test]
    fn non_arabic_letters_bypass_arabic_pipeline() {
        // Latin and Hebrew scripts should not be run through the Arabic
        // analyzer — they're routed back as Foreign tokens.
        for token in ["café", "שלום", "hello"] {
            let a = analyze_best(token);
            assert_eq!(a.pos, PartOfSpeech::Foreign, "{token} should be Foreign");
        }
    }
}
