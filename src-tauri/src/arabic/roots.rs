//! The root inventory — the bag of dictionary-sanctioned (j-d-r / جذور)
//! from which every derivable Arabic word is built.
//!
//! The inventory has **two** jobs:
//!
//! 1. **Classification (static, no lookup).** Given a candidate sequence
//!    of 3 or 4 radical letters, decide which of the 8 `RootClass`
//!    shapes it belongs to. This is pure arithmetic on the letters — no
//!    dictionary required — and lets the generator apply the correct
//!    morphophonemic rules even for roots we've never seen.
//!
//! 2. **Whitelist (seed + corpus).** Applying every pattern to every
//!    possible letter triple produces a 17,000³ combinatorial explosion,
//!    most of which are nonsense. The whitelist filters the generator's
//!    output to only those (root, pattern) pairs whose root is a real
//!    Arabic jidr. M1f ships a hand-picked seed of ~120 high-frequency
//!    roots; the M1f-data milestone expands this to 7K from classical
//!    dictionaries (Lisān, Qāmūs) with CC-BY-SA-derived data files.
//!
//! # Storage
//!
//! The radical sequence is stored as a `Vec<char>` inside `Root.radicals`
//! (see `types.rs`). For lookup we convert to a hyphenated string key
//! (`"ك-ت-ب"`) to keep the `HashMap` simple and human-readable in debug
//! output.
//!
//! # Classification algorithm
//!
//! Given radicals `r[0..n]`:
//!
//! ```text
//!   if n == 3:
//!       if any r[i] is hamza-bearing  → HamzatedTriliteral
//!       elif r[1] == r[2]             → GeminatedTriliteral
//!       elif r[0] is weak (و/ي)       → AssimilatedTriliteral
//!       elif r[1] is weak             → HollowTriliteral
//!       elif r[2] is weak             → DefectiveTriliteral
//!       else                          → SoundTriliteral
//!   elif n == 4:
//!       if any r[i] is weak or hamza  → WeakQuadriliteral
//!       else                          → SoundQuadriliteral
//! ```
//!
//! Hamza beats everything because hamzated patterns collide with every
//! other weak-radical rule; treating it as its own class simplifies the
//! generator.

use super::types::{Root, RootClass};
use std::collections::HashMap;
use std::sync::OnceLock;

// ──────────────────────────────────────────────────────────────────────
// Classification
// ──────────────────────────────────────────────────────────────────────

/// Weak letters: ا و ي. These trigger hollow / defective / assimilated
/// rules in the generator depending on where they sit in the root.
#[inline]
pub fn is_weak(c: char) -> bool {
    matches!(c, 'ا' | 'و' | 'ي')
}

/// Any form of hamza — bare or on a carrier. A root with a hamza letter
/// is treated as `HamzatedTriliteral` regardless of whether its other
/// radicals are sound or weak.
#[inline]
pub fn is_hamza(c: char) -> bool {
    matches!(c, 'ء' | 'أ' | 'إ' | 'آ' | 'ؤ' | 'ئ')
}

/// Deterministic root-class classifier. Works for any letter triple or
/// quadruple — no dictionary lookup.
///
/// Returns `None` if the radical count is outside 3..=4 (only tri- and
/// quadriliterals are modeled; Arabic has a few rare 5-radical roots
/// but they're reducible to quadriliterals for our purposes).
pub fn classify(radicals: &[char]) -> Option<RootClass> {
    match radicals.len() {
        3 => Some(classify_triliteral(radicals)),
        4 => Some(classify_quadriliteral(radicals)),
        _ => None,
    }
}

fn classify_triliteral(r: &[char]) -> RootClass {
    if r.iter().any(|&c| is_hamza(c)) {
        return RootClass::HamzatedTriliteral;
    }
    // Gemination (ر[1] = r[2]) wins over other weak positioning because
    // the geminated form collapses the final two radicals into a shaddah.
    if r[1] == r[2] {
        return RootClass::GeminatedTriliteral;
    }
    if is_weak(r[0]) {
        return RootClass::AssimilatedTriliteral;
    }
    if is_weak(r[1]) {
        return RootClass::HollowTriliteral;
    }
    if is_weak(r[2]) {
        return RootClass::DefectiveTriliteral;
    }
    RootClass::SoundTriliteral
}

fn classify_quadriliteral(r: &[char]) -> RootClass {
    if r.iter().any(|&c| is_weak(c) || is_hamza(c)) {
        RootClass::WeakQuadriliteral
    } else {
        RootClass::SoundQuadriliteral
    }
}

// ──────────────────────────────────────────────────────────────────────
// String ↔ Root helpers
// ──────────────────────────────────────────────────────────────────────

/// Canonical hyphenated string key for a root: `['ك','ت','ب']` → `"ك-ت-ب"`.
pub fn canonical_key(radicals: &[char]) -> String {
    let mut s = String::with_capacity(radicals.len() * 3);
    for (i, c) in radicals.iter().enumerate() {
        if i > 0 { s.push('-'); }
        s.push(*c);
    }
    s
}

/// Parse a hyphenated root key back into radicals. Used when roots are
/// stored as strings in FTS payloads or override files.
pub fn parse_key(key: &str) -> Vec<char> {
    key.split('-').filter(|s| !s.is_empty()).flat_map(|s| s.chars()).collect()
}

/// Convenience: build a `Root` from a hyphenated key, auto-classifying.
/// Returns `None` if the key has fewer than 3 or more than 4 letters.
pub fn root_from_key(key: &str, gloss: Option<&str>) -> Option<Root> {
    let radicals = parse_key(key);
    let class = classify(&radicals)?;
    Some(Root {
        radicals,
        class,
        gloss: gloss.map(|s| s.to_string()),
    })
}

// ──────────────────────────────────────────────────────────────────────
// Seed — external TSV file loaded at startup (M1f + M1f-data).
//
// The seed corpus lives in `roots_seed.tsv` alongside this file and is
// embedded into the binary via `include_str!`. Each non-comment, non-
// blank line is `key<TAB>gloss`. The file grows append-only; `build()`
// applies first-write-wins on duplicate keys so reordering or pasting
// already-present entries is always safe.
//
// Sourcing: hand-curated from public-domain classical works (Lisān
// al-ʿArab, Qāmūs al-Muḥīṭ, Jāmiʿ al-Durūs) cross-referenced against
// modern CC-BY-SA corpus-frequency lists. No GPL / BAMA / SAMA data
// is used. Ramp target: 7K (M1f-data); v1 ships ~595 high-frequency
// MSA + Quranic roots.
// ──────────────────────────────────────────────────────────────────────

/// Raw TSV text embedded at compile time. Zero I/O at runtime — parse
/// happens lazily inside `RootsIndex::build()` on first `get()`.
const SEED_TSV: &str = include_str!("roots_seed.tsv");

/// Public accessor for the embedded seed text. Used by
/// [`crate::arabic::fst_bake`] to content-address the compiled FST cache
/// — editing `roots_seed.tsv` flips this string, which flips the cache's
/// `version_hash`, which orphans the old file so the next boot rebuilds
/// from scratch. A `pub fn` rather than a `pub const` because some
/// consumers may want to treat the seed as opaque bytes in the future.
pub fn seed_tsv() -> &'static str {
    SEED_TSV
}

/// Parse the TSV into an iterator of `(key, gloss)` pairs. Skips blank
/// lines and anything starting with '#'. Rows with fewer than two
/// tab-separated columns are dropped silently — the seed file has no
/// optional columns in v1 so a missing gloss is a file-level bug the
/// build should surface via the corpus-size assertion in tests.
fn parse_seed(tsv: &str) -> impl Iterator<Item = (&str, &str)> {
    tsv.lines().filter_map(|line| {
        let line = line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let mut cols = line.splitn(2, '\t');
        let key = cols.next()?.trim();
        let gloss = cols.next()?.trim();
        if key.is_empty() || gloss.is_empty() {
            return None;
        }
        Some((key, gloss))
    })
}

// ──────────────────────────────────────────────────────────────────────
// Index — built once per process from the seed.
// ──────────────────────────────────────────────────────────────────────

/// The loaded root index. Keyed on `canonical_key(radicals)`.
pub struct RootsIndex {
    by_key: HashMap<String, Root>,
}

impl RootsIndex {
    /// Return a reference to the process-wide singleton.
    pub fn get() -> &'static RootsIndex {
        static CELL: OnceLock<RootsIndex> = OnceLock::new();
        CELL.get_or_init(RootsIndex::build)
    }

    fn build() -> RootsIndex {
        // Rough estimate: the TSV is ~30 bytes per row on average. Over-
        // allocating slightly is cheaper than rehashing mid-insert.
        let approx = SEED_TSV.len() / 24;
        let mut by_key = HashMap::with_capacity(approx.max(128));
        for (key, gloss) in parse_seed(SEED_TSV) {
            if let Some(root) = root_from_key(key, Some(gloss)) {
                // Canonicalize the key using the parsed radicals so any
                // equivalent spelling of the key (e.g. accidental spaces)
                // collapses to the same bucket.
                let k = canonical_key(&root.radicals);
                // First-write wins — duplicates across sections (e.g.
                // hamzated ق-ر-أ also appearing in the sound list) are
                // tolerated silently. The phonological class comes from
                // `classify()` on the radicals, not from the TSV section
                // heading, so mis-sectioned rows still classify correctly.
                by_key.entry(k).or_insert(root);
            }
        }
        RootsIndex { by_key }
    }

    /// Lookup by radicals.
    pub fn lookup(&self, radicals: &[char]) -> Option<&Root> {
        self.by_key.get(&canonical_key(radicals))
    }

    /// Lookup by canonical hyphenated key.
    pub fn lookup_key(&self, key: &str) -> Option<&Root> {
        self.by_key.get(key)
    }

    /// Is this root in the whitelist?
    pub fn contains(&self, radicals: &[char]) -> bool {
        self.by_key.contains_key(&canonical_key(radicals))
    }

    /// Total count of seeded roots.
    pub fn len(&self) -> usize {
        self.by_key.len()
    }

    /// Iterate every seeded root.
    pub fn iter(&self) -> impl Iterator<Item = &Root> {
        self.by_key.values()
    }

    /// All roots of a given class.
    pub fn by_class(&self, class: RootClass) -> Vec<&Root> {
        self.by_key.values().filter(|r| r.class == class).collect()
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── classification ──────────────────────────────────────────────

    #[test]
    fn sound_triliteral_classified_correctly() {
        assert_eq!(classify(&['ك', 'ت', 'ب']), Some(RootClass::SoundTriliteral));
        assert_eq!(classify(&['ع', 'ل', 'م']), Some(RootClass::SoundTriliteral));
    }

    #[test]
    fn hollow_triliteral_detected() {
        assert_eq!(classify(&['ق', 'و', 'ل']), Some(RootClass::HollowTriliteral));
        assert_eq!(classify(&['ب', 'ي', 'ع']), Some(RootClass::HollowTriliteral));
    }

    #[test]
    fn defective_triliteral_detected() {
        assert_eq!(classify(&['د', 'ع', 'و']), Some(RootClass::DefectiveTriliteral));
        assert_eq!(classify(&['ر', 'م', 'ي']), Some(RootClass::DefectiveTriliteral));
    }

    #[test]
    fn assimilated_triliteral_detected() {
        assert_eq!(classify(&['و', 'ع', 'د']), Some(RootClass::AssimilatedTriliteral));
        assert_eq!(classify(&['ي', 'س', 'ر']), Some(RootClass::AssimilatedTriliteral));
    }

    #[test]
    fn geminated_triliteral_detected() {
        assert_eq!(classify(&['م', 'د', 'د']), Some(RootClass::GeminatedTriliteral));
        assert_eq!(classify(&['ش', 'ك', 'ك']), Some(RootClass::GeminatedTriliteral));
    }

    #[test]
    fn hamzated_triliteral_detected() {
        assert_eq!(classify(&['أ', 'م', 'ر']), Some(RootClass::HamzatedTriliteral));
        assert_eq!(classify(&['س', 'أ', 'ل']), Some(RootClass::HamzatedTriliteral));
        assert_eq!(classify(&['ق', 'ر', 'أ']), Some(RootClass::HamzatedTriliteral));
    }

    #[test]
    fn hamza_beats_weak_and_gemination() {
        // س-أ-ل has a hamza + would be 'sound' otherwise: hamza wins.
        assert_eq!(classify(&['س', 'أ', 'ل']), Some(RootClass::HamzatedTriliteral));
        // ء-م-م has hamza + geminated: hamza wins.
        assert_eq!(classify(&['ء', 'م', 'م']), Some(RootClass::HamzatedTriliteral));
    }

    #[test]
    fn sound_quadriliteral_detected() {
        assert_eq!(classify(&['د', 'ح', 'ر', 'ج']), Some(RootClass::SoundQuadriliteral));
        assert_eq!(classify(&['ت', 'ر', 'ج', 'م']), Some(RootClass::SoundQuadriliteral));
    }

    #[test]
    fn weak_quadriliteral_detected() {
        // دحوج (hypothetical) — weak 3rd radical in a quad root.
        assert_eq!(classify(&['د', 'ح', 'و', 'ج']), Some(RootClass::WeakQuadriliteral));
    }

    #[test]
    fn biliterals_and_quintiliterals_return_none() {
        assert_eq!(classify(&['ك', 'ت']), None);
        assert_eq!(classify(&['ك', 'ت', 'ب', 'ر', 'ج']), None);
        assert_eq!(classify(&[]), None);
    }

    // ── canonical_key / parse_key roundtrip ─────────────────────────

    #[test]
    fn canonical_key_formats_with_hyphens() {
        assert_eq!(canonical_key(&['ك', 'ت', 'ب']), "ك-ت-ب");
        assert_eq!(canonical_key(&['د', 'ح', 'ر', 'ج']), "د-ح-ر-ج");
    }

    #[test]
    fn parse_key_inverts_canonical_key() {
        let radicals = vec!['ك', 'ت', 'ب'];
        let key = canonical_key(&radicals);
        let parsed = parse_key(&key);
        assert_eq!(parsed, radicals);
    }

    #[test]
    fn root_from_key_auto_classifies() {
        let r = root_from_key("ك-ت-ب", Some("writing")).unwrap();
        assert_eq!(r.class, RootClass::SoundTriliteral);
        assert_eq!(r.gloss.as_deref(), Some("writing"));

        let r = root_from_key("ء-م-م", Some("leading")).unwrap();
        assert_eq!(r.class, RootClass::HamzatedTriliteral); // hamza wins
    }

    // ── index ───────────────────────────────────────────────────────

    #[test]
    fn index_contains_ktb() {
        let idx = RootsIndex::get();
        let r = idx.lookup(&['ك', 'ت', 'ب']).expect("ك-ت-ب must be in seed");
        assert_eq!(r.class, RootClass::SoundTriliteral);
        assert_eq!(r.gloss.as_deref(), Some("writing"));
    }

    #[test]
    fn index_contains_aʾimma_root() {
        // The root of الأئمة is ء-م-م (to lead). This must be in the seed
        // so the generator can produce أَفْعِلَة → أَئِمَّة.
        let idx = RootsIndex::get();
        assert!(idx.contains(&['ء', 'م', 'م']),
            "ء-م-م (root of الأئمة) must be present for the flagship regression");
    }

    #[test]
    fn index_contains_quadriliterals() {
        let idx = RootsIndex::get();
        assert!(idx.contains(&['د', 'ح', 'ر', 'ج']));
        assert!(idx.contains(&['ت', 'ر', 'ج', 'م']));
    }

    #[test]
    fn index_has_reasonable_size() {
        let idx = RootsIndex::get();
        // M1f + M1f-data partial ships ~595 roots; the long-range target
        // is 7K. The lower bound guards against accidental file deletion
        // or a parser regression; the upper bound is generous to allow
        // append-only growth without forcing a test edit per add.
        assert!(
            idx.len() >= 500,
            "roots seed unexpectedly small: {} (expected ≥ 500)",
            idx.len()
        );
        assert!(
            idx.len() <= 10_000,
            "roots seed grew past ramp target: {} (revisit test cap)",
            idx.len()
        );
    }

    #[test]
    fn every_class_is_represented_in_seed() {
        let idx = RootsIndex::get();
        for class in [
            RootClass::SoundTriliteral,
            RootClass::HollowTriliteral,
            RootClass::DefectiveTriliteral,
            RootClass::AssimilatedTriliteral,
            RootClass::GeminatedTriliteral,
            RootClass::HamzatedTriliteral,
            RootClass::SoundQuadriliteral,
        ] {
            assert!(
                !idx.by_class(class).is_empty(),
                "no roots of class {:?} in seed — needed so generator test \
                 coverage hits every class",
                class
            );
        }
    }

    #[test]
    fn unknown_root_returns_none() {
        let idx = RootsIndex::get();
        assert!(idx.lookup(&['ف', 'غ', 'ذ']).is_none()); // not a real root
    }

    // ── TSV parser ──────────────────────────────────────────────────

    #[test]
    fn parse_seed_skips_comments_and_blank_lines() {
        let input = "\
# comment line
\n\
\n\
ك-ت-ب\twriting\n\
# another comment\n\
\n\
ع-ل-م\tknowing\n\
";
        let out: Vec<(&str, &str)> = parse_seed(input).collect();
        assert_eq!(out, vec![("ك-ت-ب", "writing"), ("ع-ل-م", "knowing")]);
    }

    #[test]
    fn parse_seed_trims_whitespace_around_columns() {
        let input = "  ك-ت-ب  \t  writing  \n";
        let out: Vec<(&str, &str)> = parse_seed(input).collect();
        assert_eq!(out, vec![("ك-ت-ب", "writing")]);
    }

    #[test]
    fn parse_seed_drops_rows_missing_gloss() {
        // A key-only row (no tab, no gloss) is a file-level bug — drop
        // silently; the corpus-size assertion will catch the regression.
        let input = "ك-ت-ب\nع-ل-م\tknowing\n";
        let out: Vec<(&str, &str)> = parse_seed(input).collect();
        assert_eq!(out, vec![("ع-ل-م", "knowing")]);
    }

    #[test]
    fn parse_seed_handles_crlf_line_endings() {
        // Windows editors may save \r\n — loader must not treat \r as part
        // of the gloss (which would break equality lookups downstream).
        let input = "ك-ت-ب\twriting\r\nع-ل-م\tknowing\r\n";
        let out: Vec<(&str, &str)> = parse_seed(input).collect();
        assert_eq!(out, vec![("ك-ت-ب", "writing"), ("ع-ل-م", "knowing")]);
    }

    #[test]
    fn seed_tsv_contains_flagship_roots() {
        // Smoke test against the embedded file: the highest-signal roots
        // for regression targets must be present.
        let idx = RootsIndex::get();
        for key in [
            "ك-ت-ب",    // basic sound triliteral
            "ء-م-م",    // ← الأئمة flagship
            "ق-و-ل",    // hollow M2.c
            "د-ع-و",    // defective M2.c
            "و-ع-د",    // assimilated M2.c
            "د-ح-ر-ج",  // sound quadriliteral
        ] {
            assert!(
                idx.lookup_key(key).is_some(),
                "seed TSV missing flagship root {key}"
            );
        }
    }

    // ── helpers ─────────────────────────────────────────────────────

    #[test]
    fn weak_letter_check_rejects_hamza() {
        // Hamzas are handled separately; they must not trigger the weak
        // branch or classification would miss the HamzatedTriliteral
        // fast-path.
        assert!(!is_weak('أ'));
        assert!(!is_weak('إ'));
        assert!(is_weak('ا')); // plain alif is weak (for rare assimilated cases)
        assert!(is_weak('و'));
        assert!(is_weak('ي'));
    }

    #[test]
    fn hamza_check_covers_all_variants() {
        for c in ['ء', 'أ', 'إ', 'آ', 'ؤ', 'ئ'] {
            assert!(is_hamza(c), "{c} should be recognized as hamza");
        }
        assert!(!is_hamza('ا'));
        assert!(!is_hamza('ك'));
    }
}
