//! Arabic affixes — the clitic and functional morphology inventory.
//!
//! Arabic attaches many grammatical elements to the word as clitics
//! rather than separating them with spaces. Analyzing `فسيكتبونها`
//! requires peeling:
//!
//! ```text
//!   فَـ (conjunction "so")
//!   سَـ (future "will")
//!   يَـ (imperfect 3-masc)
//!   كتب (root stem, Form I imperfect)
//!   ـون (masculine plural)
//!   ـها (object pronoun "her/it")
//! ```
//!
//! Each affix is a separable morpheme with a defined **slot** (position
//! in the prefix / suffix stack) and a legal stacking rule (which slots
//! may legally appear outer / inner to it).
//!
//! The stacking model is what lets the analyzer reject bogus splits:
//! `وَأَـ` (conjunction then interrogative) is illegal because the
//! interrogative `أ` must sit outermost; `أَوَـ` (`أوفي` "and am I in?")
//! is legal because `أ` precedes `و` in slot order.
//!
//! # Coverage
//!
//! - 6 prefix slots × atomic surfaces → ~15 prefixes
//! - 3 suffix slots × many grammatical persons/numbers → ~50 suffixes
//! - Compound-aware: `لل` (ل + ال elision) is recognized as one atomic
//!   surface belonging to the `PrefixPreposition` slot with a pinned
//!   `PrefixDefinite` companion, saved separately under `compounds()`
//!   so the analyzer can try the fused surface before two-step peeling.
//!
//! # What's NOT here
//!
//! Nunation (tanwīn: `ـً`, `ـٍ`, `ـٌ`) and case vowels (`ـَ`, `ـِ`, `ـُ`)
//! are not modeled as affixes — they're tashkeel, stripped by the
//! normalizer (layer 1) before the analyzer runs.

use super::types::{Affix, AffixFunction, AffixSlot};

fn a(surface: &str, slot: AffixSlot, function: AffixFunction, allows_after: Vec<AffixSlot>) -> Affix {
    Affix {
        surface: surface.to_string(),
        slot,
        function,
        allows_after,
    }
}

// ──────────────────────────────────────────────────────────────────────
// §1. PREFIXES
// ──────────────────────────────────────────────────────────────────────
//
// Prefix slot order, from outermost to innermost (i.e. leftmost to
// rightmost as written):
//
//   [Interrogative] [Conjunction] [Future] [Preposition] [Definite] [Imperfect]
//
// The `allows_after` field on each affix lists the slots that may
// legally appear **to the outer side** (toward the start of the word)
// — so a definite-article `ال` allows conjunction / preposition /
// interrogative to have preceded it, but not another definite article.
// ──────────────────────────────────────────────────────────────────────

fn prefixes() -> Vec<Affix> {
    vec![
        // ── Interrogative hamza — أ (e.g. أتكتب؟ "do you write?") ──
        // Outermost prefix. Nothing may precede it.
        a("أ", AffixSlot::PrefixInterrogative, AffixFunction::InterrogativeHamza, vec![]),

        // ── Conjunctions — و / ف ──
        // May follow only the interrogative.
        a("و", AffixSlot::PrefixConjunction, AffixFunction::ConjunctionAnd, vec![AffixSlot::PrefixInterrogative]),
        a("ف", AffixSlot::PrefixConjunction, AffixFunction::ConjunctionThen, vec![AffixSlot::PrefixInterrogative]),

        // ── Future marker — س (سيذهب "he will go") ──
        // Only on imperfect verbs; may follow conjunction / interrogative.
        a("س", AffixSlot::PrefixFuture, AffixFunction::FutureSa, vec![
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
        ]),

        // ── Prepositions — ب / ك / ل ──
        // Fused to the following word; may follow conj/interrog.
        a("ب", AffixSlot::PrefixPreposition, AffixFunction::PrepositionBi, vec![
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
        ]),
        a("ك", AffixSlot::PrefixPreposition, AffixFunction::PrepositionKa, vec![
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
        ]),
        a("ل", AffixSlot::PrefixPreposition, AffixFunction::PrepositionLi, vec![
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
        ]),

        // ── Definite article — ال ──
        // May follow a preposition (bi + al → بال, ka + al → كال),
        // a conjunction (wa + al → وال), or an interrogative (أ + ال).
        a("ال", AffixSlot::PrefixDefinite, AffixFunction::DefiniteAl, vec![
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
            AffixSlot::PrefixPreposition,
        ]),

        // ── Imperfect verbal prefixes — ي / ت / ن / أ ──
        // Bind to imperfect verb stems. Cannot follow a preposition
        // or definite article (those go only on nouns).
        a("ي", AffixSlot::PrefixImperfect, AffixFunction::ImperfectYa, vec![
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
            AffixSlot::PrefixFuture,
        ]),
        a("ت", AffixSlot::PrefixImperfect, AffixFunction::ImperfectTa, vec![
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
            AffixSlot::PrefixFuture,
        ]),
        a("ن", AffixSlot::PrefixImperfect, AffixFunction::ImperfectNa, vec![
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
            AffixSlot::PrefixFuture,
        ]),
        a("أ", AffixSlot::PrefixImperfect, AffixFunction::ImperfectA, vec![
            AffixSlot::PrefixConjunction,
            AffixSlot::PrefixFuture,
        ]),
    ]
}

/// Compound surface forms: the analyzer tries these as single atomic
/// prefixes before falling back to two-step peeling. Each entry is
/// (surface, components-in-order).
///
/// Right now the only true fusion is `لل = ل + ال` (preposition + article
/// with elision of the alif of the article). We encode this so that a
/// word like `للكتاب` ("for the book") is recognized in one step.
fn compound_prefixes() -> Vec<(String, Vec<AffixFunction>)> {
    vec![
        (
            "لل".to_string(),
            vec![AffixFunction::PrepositionLi, AffixFunction::DefiniteAl],
        ),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// §2. SUFFIXES
// ──────────────────────────────────────────────────────────────────────
//
// Suffix slot order, from innermost to outermost (closest to the root,
// moving away). As written on the page, innermost is to the left of the
// suffix block, outermost is to the right:
//
//   stem [Feminine?] [Number/Gender] [Pronoun]
//
// # Verbal vs. nominal
//
// Verbal and nominal suffixes share slots but carry distinct functions:
//
//   - After a perfect verb:  past-person markers (تُ, تَ, نا, وا, etc.)
//   - After an imperfect verb: subject-agreement (ون, ين, ان, ن)
//   - After a noun:  dual / plural markers + pronoun possessives
//
// The analyzer uses the POS of the identified stem (coming from the
// generator / FST) to choose which suffixes are plausible at each slot.
// ──────────────────────────────────────────────────────────────────────

fn suffixes() -> Vec<Affix> {
    vec![
        // ── Feminine marker — ة ──
        // Innermost. Followed by nothing outer is the common case (مدرسة),
        // but can be followed by a pronoun (مدرستها — with ة sometimes
        // surfacing as ت before pronoun; normalizer handles the variant).
        a("ة", AffixSlot::SuffixFeminine, AffixFunction::FeminineTa, vec![]),

        // ── Number / gender markers ──
        // These replace the feminine marker when applied (a plural noun
        // drops its singular ة and takes ات / ون / ين / ان instead).
        // The `allows_after` encodes: a Number suffix may follow a
        // Feminine suffix only in the rare cases where both coexist
        // (e.g. some dialects / frozen expressions).
        a("ان", AffixSlot::SuffixNumber, AffixFunction::DualAlif, vec![AffixSlot::SuffixFeminine]),
        a("ين", AffixSlot::SuffixNumber, AffixFunction::DualYa, vec![AffixSlot::SuffixFeminine]),
        a("ات", AffixSlot::SuffixNumber, AffixFunction::SoundFemPlural, vec![AffixSlot::SuffixFeminine]),
        a("ون", AffixSlot::SuffixNumber, AffixFunction::SoundMascPluralWaw, vec![AffixSlot::SuffixFeminine]),

        // Verbal perfect endings — go in SuffixNumber slot because they
        // similarly convey person/number on the verb.
        a("تُ", AffixSlot::SuffixNumber, AffixFunction::VerbPastTu1s, vec![]),
        a("تَ", AffixSlot::SuffixNumber, AffixFunction::VerbPastTuMasc, vec![]),
        a("تِ", AffixSlot::SuffixNumber, AffixFunction::VerbPastTuFem, vec![]),
        a("نا", AffixSlot::SuffixNumber, AffixFunction::VerbPastNa, vec![]),
        a("تم", AffixSlot::SuffixNumber, AffixFunction::VerbPastTum, vec![]),
        a("تن", AffixSlot::SuffixNumber, AffixFunction::VerbPastTunna, vec![]),
        a("وا", AffixSlot::SuffixNumber, AffixFunction::VerbPastU, vec![]),
        a("ت", AffixSlot::SuffixNumber, AffixFunction::VerbPastAt, vec![]),

        // Verbal imperfect endings — plural/dual markers on يكتبون, تكتبين, يكتبان.
        // Surface collision with nominal plurals is resolved by POS of the stem.
        a("ون", AffixSlot::SuffixNumber, AffixFunction::VerbImperfectMascPl, vec![]),
        a("ن",  AffixSlot::SuffixNumber, AffixFunction::VerbImperfectFemPl, vec![]),

        // ── Pronominal suffixes ──
        // Outermost. Follow feminine, number, or attach directly to stem.
        // Order inside `allows_after`: any inner suffix is legal to precede.
        a("ي",   AffixSlot::SuffixPronoun, AffixFunction::PronounI, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("كَ",  AffixSlot::SuffixPronoun, AffixFunction::PronounKa, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("ك",   AffixSlot::SuffixPronoun, AffixFunction::PronounKa, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("كم",  AffixSlot::SuffixPronoun, AffixFunction::PronounKum, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("كن",  AffixSlot::SuffixPronoun, AffixFunction::PronounKunna, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("ه",   AffixSlot::SuffixPronoun, AffixFunction::PronounHu, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("ها",  AffixSlot::SuffixPronoun, AffixFunction::PronounHa, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("هم",  AffixSlot::SuffixPronoun, AffixFunction::PronounHum, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("هن",  AffixSlot::SuffixPronoun, AffixFunction::PronounHunna, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
        a("نا",  AffixSlot::SuffixPronoun, AffixFunction::PronounNa, vec![
            AffixSlot::SuffixFeminine, AffixSlot::SuffixNumber,
        ]),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// §3. PEELING — enumerate all legal (prefixes, remainder, suffixes)
// decompositions of a stripped surface.
//
// This is the cascade that turns الأئمة into (ال + أئمة), فسيكتبونها
// into (ف + س + ي + كتب + ون + ها), etc. It does NOT evaluate whether
// the remainder is a real stem — that's Layer 3's job. The peeler's
// contract is simply: given a surface, return every way to split it
// into legal affix chains plus a middle, deduplicated.
// ──────────────────────────────────────────────────────────────────────

use std::sync::OnceLock;

/// Slot order for prefixes, outermost → innermost (left → right in surface).
/// A chain is valid iff for every position `i`, every slot at positions
/// `0..i` appears in `chain[i].allows_after`.
const PREFIX_SLOT_ORDER: &[AffixSlot] = &[
    AffixSlot::PrefixInterrogative,
    AffixSlot::PrefixConjunction,
    AffixSlot::PrefixFuture,
    AffixSlot::PrefixPreposition,
    AffixSlot::PrefixDefinite,
    AffixSlot::PrefixImperfect,
];

/// Slot order for suffixes, innermost → outermost (left → right after the stem).
const SUFFIX_SLOT_ORDER: &[AffixSlot] = &[
    AffixSlot::SuffixFeminine,
    AffixSlot::SuffixNumber,
    AffixSlot::SuffixPronoun,
];

/// One legal decomposition of a surface form into affix chains plus a
/// middle "remainder" that Layer 3 must look up.
///
/// The empty prefix / empty suffix / original-surface case is included
/// so callers can use `peel()` as a uniform iterator; the analyzer
/// dedupes against its earlier bare-surface lookup.
#[derive(Debug, Clone)]
pub struct PeelCandidate {
    /// Prefixes in outer→inner order (the order they appear written).
    pub prefixes: Vec<Affix>,
    /// The middle of the word after peeling — what gets looked up.
    pub remainder: String,
    /// Suffixes in inner→outer order (left-to-right as written after stem).
    pub suffixes: Vec<Affix>,
}

/// Is this prefix chain valid under `allows_after` stacking rules?
fn is_valid_chain(chain: &[&Affix]) -> bool {
    for i in 0..chain.len() {
        let affix = chain[i];
        for outer in chain.iter().take(i) {
            if !affix.allows_after.contains(&outer.slot) {
                return false;
            }
        }
    }
    true
}

/// Enumerate all valid prefix chains, each being a list of 0..=N affixes
/// where slots appear in `PREFIX_SLOT_ORDER` and every affix's
/// `allows_after` is satisfied by the slots already chosen.
///
/// The empty chain is included (first entry) so callers can treat
/// "no prefix" as just another peeling option.
fn enumerate_chains(slot_order: &[AffixSlot], pool: &[Affix]) -> Vec<Vec<Affix>> {
    // For each slot, collect all affixes belonging to it. Then recurse:
    // at slot `i`, choose either "none" or "one of the affixes in slot i".
    let per_slot: Vec<Vec<&Affix>> = slot_order
        .iter()
        .map(|&slot| pool.iter().filter(|a| a.slot == slot).collect::<Vec<_>>())
        .collect();

    let mut out: Vec<Vec<Affix>> = vec![Vec::new()];

    for group in &per_slot {
        let mut next: Vec<Vec<Affix>> = Vec::with_capacity(out.len() * (group.len() + 1));
        for existing in &out {
            // Option A: skip this slot.
            next.push(existing.clone());

            // Option B: add one affix from this slot, if the chain stays valid.
            for candidate in group {
                let mut extended = existing.clone();
                extended.push((*candidate).clone());
                let refs: Vec<&Affix> = extended.iter().collect();
                if is_valid_chain(&refs) {
                    next.push(extended);
                }
            }
        }
        out = next;
    }

    out
}

fn prefix_chains_cached() -> &'static [Vec<Affix>] {
    static CACHE: OnceLock<Vec<Vec<Affix>>> = OnceLock::new();
    CACHE
        .get_or_init(|| enumerate_chains(PREFIX_SLOT_ORDER, &prefixes()))
        .as_slice()
}

fn suffix_chains_cached() -> &'static [Vec<Affix>] {
    static CACHE: OnceLock<Vec<Vec<Affix>>> = OnceLock::new();
    CACHE
        .get_or_init(|| enumerate_chains(SUFFIX_SLOT_ORDER, &suffixes()))
        .as_slice()
}

/// Minimum remainder length (in chars) to consider a peeling worth
/// looking up. Shorter than this and the remainder is almost certainly
/// not a stem — we'd generate a flood of spurious candidates.
///
/// Most Arabic triliteral surface forms have at least 3 characters in
/// their bare stem (e.g. كَتَبَ → strip tashkeel → كتب). A length-2
/// remainder would only make sense for heavily-assimilated defective
/// stems (e.g. رأى → رأ), and we'd rather miss those than flood the
/// disambiguator with noise.
const MIN_REMAINDER_CHARS: usize = 3;

/// Enumerate every legal `(prefix_chain, remainder, suffix_chain)`
/// split of `stripped`. The empty-prefix / empty-suffix combination
/// (i.e. the original surface unchanged) is included.
///
/// Runs in `O((P + C) × S)` where P and S are the atomic prefix/suffix
/// chain counts (both cached static singletons) and C is the number of
/// compound-surface prefixes. Compounds (currently just `لل` = li + al
/// with alif-elision) are tried as extra prefix surfaces that map to
/// their constituent atomic affixes — this is how `للكتاب` recovers
/// both prepositional and definite markers in one step.
///
/// The cascade is NOT responsible for checking whether the remainder is
/// a real stem; that's the generative-index caller's job.
pub fn peel(stripped: &str) -> Vec<PeelCandidate> {
    let prefix_chains = prefix_chains_cached();
    let suffix_chains = suffix_chains_cached();
    let mut out: Vec<PeelCandidate> = Vec::with_capacity(16);

    // Pre-compose atomic-prefix / atomic-suffix surface strings.
    let prefix_surfaces: Vec<String> = prefix_chains
        .iter()
        .map(|chain| chain.iter().map(|a| a.surface.as_str()).collect::<String>())
        .collect();
    let suffix_surfaces: Vec<String> = suffix_chains
        .iter()
        .map(|chain| chain.iter().map(|a| a.surface.as_str()).collect::<String>())
        .collect();

    // Expand atomic prefix chains with compound forms so we can try
    // each compound as one atomic "super-prefix" whose semantic content
    // is the sequence of AffixFunctions listed in compound_forms().
    let atomic_pool: Vec<Affix> = prefixes();
    let compound_prefix_chains: Vec<(String, Vec<Affix>)> = compound_prefixes()
        .into_iter()
        .filter_map(|(surface, functions)| {
            // Map each function back to its Affix row (uniquely identified
            // by function in the current catalogue).
            let chain: Option<Vec<Affix>> = functions
                .iter()
                .map(|f| atomic_pool.iter().find(|a| a.function == *f).cloned())
                .collect();
            chain.map(|c| (surface, c))
        })
        .collect();

    let try_peel = |surface: &str,
                    prefix_chain: Vec<Affix>,
                    out: &mut Vec<PeelCandidate>| {
        let mid = match stripped.strip_prefix(surface) {
            Some(m) => m,
            None => return,
        };
        for (si, s_surface) in suffix_surfaces.iter().enumerate() {
            let remainder = match mid.strip_suffix(s_surface.as_str()) {
                Some(r) => r,
                None => continue,
            };
            if remainder.chars().count() < MIN_REMAINDER_CHARS {
                continue;
            }
            out.push(PeelCandidate {
                prefixes: prefix_chain.clone(),
                remainder: remainder.to_string(),
                suffixes: suffix_chains[si].clone(),
            });
        }
    };

    // Atomic prefix chains.
    for (pi, p_surface) in prefix_surfaces.iter().enumerate() {
        try_peel(p_surface.as_str(), prefix_chains[pi].clone(), &mut out);
    }

    // Compound prefix surfaces.
    for (c_surface, c_chain) in &compound_prefix_chains {
        try_peel(c_surface.as_str(), c_chain.clone(), &mut out);
    }

    out
}

// ──────────────────────────────────────────────────────────────────────
// PUBLIC API
// ──────────────────────────────────────────────────────────────────────

/// All atomic affixes (prefixes + suffixes). Compounds are returned
/// separately via [`compound_forms`].
pub fn all_affixes() -> Vec<Affix> {
    let mut v = prefixes();
    v.extend(suffixes());
    v
}

/// Prefixes only, in slot order.
pub fn all_prefixes() -> Vec<Affix> {
    prefixes()
}

/// Suffixes only, in slot order.
pub fn all_suffixes() -> Vec<Affix> {
    suffixes()
}

/// Compound prefix surfaces (currently just `لل`). Returned as
/// `(surface, sequence-of-atomic-functions)`.
pub fn compound_forms() -> Vec<(String, Vec<AffixFunction>)> {
    compound_prefixes()
}

/// All affixes belonging to a given slot, for analyzer lookup.
pub fn affixes_in_slot(slot: AffixSlot) -> Vec<Affix> {
    all_affixes().into_iter().filter(|a| a.slot == slot).collect()
}

// ──────────────────────────────────────────────────────────────────────
// Tests — shape-only checks. Actual peeling correctness is verified
// in the analyzer test suite (M5).
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_and_suffix_slots_are_disjoint() {
        // A prefix must never appear in a suffix slot and vice-versa.
        let pref_slots = [
            AffixSlot::PrefixInterrogative,
            AffixSlot::PrefixConjunction,
            AffixSlot::PrefixFuture,
            AffixSlot::PrefixPreposition,
            AffixSlot::PrefixDefinite,
            AffixSlot::PrefixImperfect,
        ];
        let suf_slots = [
            AffixSlot::SuffixFeminine,
            AffixSlot::SuffixNumber,
            AffixSlot::SuffixPronoun,
        ];
        for p in prefixes() {
            assert!(pref_slots.contains(&p.slot), "prefix {} in wrong slot {:?}", p.surface, p.slot);
        }
        for s in suffixes() {
            assert!(suf_slots.contains(&s.slot), "suffix {} in wrong slot {:?}", s.surface, s.slot);
        }
    }

    #[test]
    fn definite_article_is_present() {
        // Necessary for decomposing الكتاب → ال + كتاب, الأئمة → ال + أئمة.
        assert!(prefixes().iter().any(|p| p.surface == "ال" && p.slot == AffixSlot::PrefixDefinite));
    }

    #[test]
    fn compound_lil_is_present() {
        // للكتاب → لل + كتاب. Must be recognized as a single compound.
        let comps = compound_forms();
        assert!(comps.iter().any(|(s, _)| s == "لل"));
    }

    #[test]
    fn interrogative_has_no_outer_peer() {
        // أ is the outermost prefix — nothing legally precedes it.
        let interr: Vec<_> = prefixes()
            .into_iter()
            .filter(|p| p.slot == AffixSlot::PrefixInterrogative)
            .collect();
        assert!(!interr.is_empty());
        for p in interr {
            assert!(p.allows_after.is_empty(), "interrogative should have empty allows_after");
        }
    }

    #[test]
    fn pronoun_ha_is_present_for_feminine_pronoun() {
        // Used heavily: كتابها, أرضها, قاتلوها.
        assert!(suffixes().iter().any(|s|
            s.surface == "ها" && s.slot == AffixSlot::SuffixPronoun
            && matches!(s.function, AffixFunction::PronounHa)
        ));
    }

    #[test]
    fn all_affixes_count_is_in_expected_range() {
        // Sanity floor: if someone accidentally deletes half the file,
        // this catches it. Real regression is in M5.
        let n = all_affixes().len();
        assert!(n >= 30, "expected at least 30 affixes, got {}", n);
        assert!(n <= 80, "unexpectedly large affix set: {}", n);
    }

    // ── peeler tests ────────────────────────────────────────────────

    #[test]
    fn peel_returns_bare_word_as_empty_chain() {
        // For a word with no affixes, the peeler must include the
        // trivial (empty, full-word, empty) candidate — downstream
        // Layer 3 will look the word up unchanged.
        let cands = peel("كاتب");
        assert!(
            cands.iter().any(|c|
                c.prefixes.is_empty() && c.suffixes.is_empty() && c.remainder == "كاتب"
            ),
            "bare word كاتب must yield a (empty, كاتب, empty) peeling; got {:?}",
            cands
        );
    }

    #[test]
    fn peel_strips_definite_article_from_alaimma() {
        // The flagship decomposition: الأئمة → ال + أئمة.
        let cands = peel("الأئمة");
        let has_al_split = cands.iter().any(|c| {
            c.prefixes.iter().any(|a| a.surface == "ال" && a.slot == AffixSlot::PrefixDefinite)
                && c.remainder == "أئمة"
                && c.suffixes.is_empty()
        });
        assert!(
            has_al_split,
            "expected ال + أئمة peeling; got {:?}",
            cands.iter().map(|c| (
                c.prefixes.iter().map(|a| a.surface.as_str()).collect::<String>(),
                c.remainder.clone(),
                c.suffixes.iter().map(|a| a.surface.as_str()).collect::<String>(),
            )).collect::<Vec<_>>()
        );
    }

    #[test]
    fn peel_strips_conjunction_fa_from_fakaatib() {
        // فكاتب → ف + كاتب (conjunction "then" + active participle).
        let cands = peel("فكاتب");
        let has_fa_split = cands.iter().any(|c| {
            c.prefixes.iter().any(|a| a.surface == "ف" && a.slot == AffixSlot::PrefixConjunction)
                && c.remainder == "كاتب"
                && c.suffixes.is_empty()
        });
        assert!(
            has_fa_split,
            "expected ف + كاتب peeling; got {:?}",
            cands
        );
    }

    #[test]
    fn peel_strips_suffix_ha_from_katabaha() {
        // كتابها → كتاب + ها (pronoun "her book").
        let cands = peel("كتابها");
        let has_ha_split = cands.iter().any(|c| {
            c.prefixes.is_empty()
                && c.remainder == "كتاب"
                && c.suffixes.iter().any(|a| a.surface == "ها")
        });
        assert!(
            has_ha_split,
            "expected (empty, كتاب, ها) peeling; got {:?}",
            cands
        );
    }

    #[test]
    fn peel_short_remainders_are_dropped() {
        // The minimum remainder is 3 chars. A word like "اف" would
        // try to peel as ا (no — not a valid prefix) or as bare ("اف"
        // is only 2 chars). Confirm the length guard keeps us out.
        let cands = peel("اف");
        for c in &cands {
            assert!(
                c.remainder.chars().count() >= MIN_REMAINDER_CHARS,
                "peeler should not emit remainders shorter than {} chars; got remainder {:?}",
                MIN_REMAINDER_CHARS,
                c.remainder
            );
        }
    }

    #[test]
    fn peel_respects_stacking_rules() {
        // No prefix chain in the peeler's output should ever violate
        // `allows_after` — e.g. a definite article `ال` must never
        // appear OUTSIDE (before) a conjunction `و`/`ف`.
        for c in peel("والكتاب") {
            for i in 0..c.prefixes.len() {
                let affix = &c.prefixes[i];
                for outer in c.prefixes.iter().take(i) {
                    assert!(
                        affix.allows_after.contains(&outer.slot),
                        "illegal stacking: {} after {} (prefix chain {:?})",
                        affix.surface, outer.surface, c.prefixes
                    );
                }
            }
        }
    }

    #[test]
    fn peel_returns_multiple_candidates_for_ambiguous_prefix() {
        // و is both a conjunction AND can appear inside other words as
        // a literal و. The peeler must return BOTH the "peel و" and
        // "don't peel" candidates so the generative index can validate.
        let cands = peel("وكتاب");
        let with_wa = cands
            .iter()
            .any(|c| c.prefixes.iter().any(|a| a.surface == "و"));
        let without_wa = cands
            .iter()
            .any(|c| c.prefixes.is_empty() && c.remainder == "وكتاب");
        assert!(with_wa, "must include peeling with و stripped");
        assert!(without_wa, "must include peeling with و retained");
    }

    #[test]
    fn peel_handles_compound_lil() {
        // للكتاب = li + al (with alif elision) + كتاب. The atomic chain
        // [PrepositionLi, DefiniteAl] would produce surface "لال", not
        // "لل" — so the peeler must try the compound surface "لل" as a
        // super-prefix, yielding the same pair of AffixFunctions.
        let cands = peel("للكتاب");
        let has_compound = cands.iter().any(|c| {
            c.remainder == "كتاب"
                && c.suffixes.is_empty()
                && c.prefixes.len() == 2
                && c.prefixes[0].function == AffixFunction::PrepositionLi
                && c.prefixes[1].function == AffixFunction::DefiniteAl
        });
        assert!(
            has_compound,
            "expected compound لل peeling into (li + al + كتاب); got {:?}",
            cands
                .iter()
                .map(|c| (
                    c.prefixes.iter().map(|a| a.surface.clone()).collect::<Vec<_>>(),
                    c.remainder.clone(),
                    c.suffixes.iter().map(|a| a.surface.clone()).collect::<Vec<_>>(),
                ))
                .collect::<Vec<_>>()
        );
    }
}
