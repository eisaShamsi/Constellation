//! Layer 1 — contextual normalizer.
//!
//! The normalizer is the **first** stage of the five-layer pipeline. Its
//! job is to reduce surface variation that is purely orthographic — things
//! an Arabic reader would consider the "same" word written differently —
//! while preserving every letter that carries morphological weight.
//!
//! # Design principles
//!
//! 1. **Lossless at the surface level.** The original input is returned
//!    verbatim on `Normalized.surface`. The FTS5 `surface` field stores
//!    exactly what the user typed; we never mutate that.
//! 2. **Tiered output.** The normalizer exposes multiple views:
//!    - `stripped` — tashkeel + tatweel removed. Always safe. This is the
//!      primary input to Layer 2 (protected list) and Layer 3 (FST).
//!    - `folded` — stripped **plus** aggressive Light10-style letter
//!      folding (alif / ya / ta-marbuta collapses). Used only as a final
//!      fallback in Layer 3 when strict matching fails.
//! 3. **Preserve hamza variants by default.** `أئمة` vs `اومة` are NOT
//!    the same word. Hamza-on-alif is a root letter. Only `folded` collapses.
//! 4. **Language-family detection.** We return a coarse `Script` tag so
//!    the caller (the FTS tokenizer) can route Persian / Urdu / Hebrew
//!    tokens to their own analyzer instead of the Arabic pipeline.
//!
//! # Why this matters for the Light10 bug
//!
//! Light10 over-strips `وائل` → `ائل` because it treats و as a prefix
//! regardless of position. The normalizer does **not** strip consonants
//! — that is Layer 3's job, guarded by the FST. The normalizer only
//! removes things no reader would consider part of the word: diacritics
//! and the elongation kashida.

use std::fmt;

// ──────────────────────────────────────────────────────────────────────
// Unicode ranges & character classification
// ──────────────────────────────────────────────────────────────────────

/// Arabic tashkeel marks (diacritics) that are always safe to strip.
///
/// Covers fatḥa / ḍamma / kasra, tanwīn, shadda, sukūn, maddah, and the
/// stand-alone hamza-above/below marks. The dagger alif (`U+0670`) is
/// included because it's a short-alif diacritic, not a full letter.
#[inline]
pub fn is_tashkeel(c: char) -> bool {
    matches!(c as u32,
        0x064B..=0x065F   // tanwīn, short vowels, shadda, sukūn, maddah, hamza marks
        | 0x0670          // superscript (dagger) alif
        | 0x06D6..=0x06ED // Qur'anic annotation marks
        | 0x08D3..=0x08FF // Extended-A marks
    )
}

/// The tatweel (kashida) — a typographic elongation with no linguistic
/// content. Always strip.
#[inline]
pub fn is_tatweel(c: char) -> bool {
    c == '\u{0640}'
}

/// Belongs to any Arabic-script Unicode block (includes Persian, Urdu,
/// Kurdish, Sindhi letters — not just Modern Standard Arabic).
#[inline]
pub fn is_arabic_block(c: char) -> bool {
    matches!(c as u32,
        0x0600..=0x06FF   // Arabic
        | 0x0750..=0x077F // Arabic Supplement
        | 0x08A0..=0x08FF // Arabic Extended-A
        | 0xFB50..=0xFDFF // Arabic Presentation Forms-A
        | 0xFE70..=0xFEFF // Arabic Presentation Forms-B
    )
}

/// Hebrew block — for language routing.
#[inline]
pub fn is_hebrew_block(c: char) -> bool {
    matches!(c as u32, 0x0590..=0x05FF | 0xFB1D..=0xFB4F)
}

/// Characters that are Persian/Urdu but not MSA. Used to flag tokens
/// that the Arabic pipeline should pass through to the per-language
/// analyzer instead of analyzing as Arabic.
#[inline]
pub fn is_persian_specific(c: char) -> bool {
    matches!(c,
        '\u{067E}' // پ peh
        | '\u{0686}' // چ tcheh
        | '\u{0698}' // ژ jeh
        | '\u{06A9}' // ک keheh (Persian kāf)
        | '\u{06AF}' // گ gaf
        | '\u{06C0}' // ۀ heh with yeh above
        | '\u{06CC}' // ی farsi yeh
    )
}

// ──────────────────────────────────────────────────────────────────────
// Output types
// ──────────────────────────────────────────────────────────────────────

/// Coarse script classification — one of these per token. The FTS
/// tokenizer uses this to route tokens to per-language analyzers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    /// Pure Modern Standard Arabic (or a word containing only MSA letters).
    Arabic,
    /// Contains Persian-specific letters (گ ک پ چ ژ ی ۀ). Should route
    /// to the Persian analyzer when we add one; for now treated as
    /// Arabic with a flag.
    PersianFamily,
    /// Hebrew.
    Hebrew,
    /// Latin-script (Latin block + extensions). Punt to English pipeline.
    Latin,
    /// Anything else (CJK, Devanagari, emoji, mixed) — passthrough verbatim.
    Other,
    /// Empty / whitespace-only.
    Empty,
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Script::Arabic => "arabic",
            Script::PersianFamily => "persian-family",
            Script::Hebrew => "hebrew",
            Script::Latin => "latin",
            Script::Other => "other",
            Script::Empty => "empty",
        };
        f.write_str(s)
    }
}

/// The full normalizer output.
///
/// The caller typically uses:
///   - `surface` to store in FTS `surface` column (display).
///   - `stripped` as the key into the protected list (Layer 2) and the
///     FST (Layer 3). This is the **primary** working form.
///   - `folded` only as a last-resort fallback when nothing else matches.
///   - `script` to decide whether to even run the Arabic pipeline.
#[derive(Debug, Clone)]
pub struct Normalized {
    /// The original input, unchanged.
    pub surface: String,
    /// Tashkeel + tatweel stripped. Always safe.
    pub stripped: String,
    /// `stripped` plus aggressive letter folding (alif / ya / ta-marbuta).
    /// This is the old Light10 behavior — intentionally last-resort.
    pub folded: String,
    /// Script classification.
    pub script: Script,
    /// True if the stripped string contains any Persian-specific letters.
    /// When set, CAE should still attempt analysis but with low
    /// confidence — likely a loanword or cross-script token.
    pub has_persian_letters: bool,
}

// ──────────────────────────────────────────────────────────────────────
// Core transformations
// ──────────────────────────────────────────────────────────────────────

/// Remove tashkeel diacritics and the tatweel from a string.
///
/// This is the **only** unconditionally-safe transformation in Arabic
/// normalization — both an Arabic reader and every dictionary agree that
/// `كَتَبَ` and `كتب` are the same word.
pub fn strip_tashkeel_and_tatweel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if !is_tashkeel(c) && !is_tatweel(c) {
            out.push(c);
        }
    }
    out
}

/// Aggressive letter folding — the classical Light10 / fuzzy-match form.
///
/// Applies these collapses:
///
///   - ا / أ / إ / آ → ا   (unifies all alif variants)
///   - ى → ي             (alif maqṣūra → yāʾ)
///   - ة → ه             (tāʾ marbūṭa → hāʾ)
///   - ؤ / ئ → و / ي     (hamza-bearer collapse)
///
/// **Only** use this when strict matching (stripped form) has failed.
/// This loses information: `وَائِل` (stripped: `وائل`) and `وَايِل`
/// (stripped: `وايل`) fold to the same `وايل` — which is exactly the
/// Light10 bug we want to avoid in the common path.
pub fn fold_letters(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let folded = match c {
            'أ' | 'إ' | 'آ' | 'ٱ' => 'ا',
            'ى' => 'ي',
            'ة' => 'ه',
            'ؤ' => 'و',
            'ئ' => 'ي',
            _ => c,
        };
        out.push(folded);
    }
    out
}

/// Count characters by script block.
fn classify_script(s: &str) -> (Script, bool) {
    let mut arabic = 0usize;
    let mut hebrew = 0usize;
    let mut latin = 0usize;
    let mut other = 0usize;
    let mut persian_specific = 0usize;

    for c in s.chars() {
        if c.is_whitespace() {
            continue;
        }
        if is_tashkeel(c) || is_tatweel(c) {
            continue;
        }
        if is_persian_specific(c) {
            persian_specific += 1;
            arabic += 1; // Persian letters live in the Arabic block
        } else if is_arabic_block(c) {
            arabic += 1;
        } else if is_hebrew_block(c) {
            hebrew += 1;
        } else if c.is_ascii_alphabetic()
            || matches!(c as u32, 0x00C0..=0x024F | 0x1E00..=0x1EFF)
        {
            latin += 1;
        } else if c.is_alphabetic() {
            other += 1;
        }
    }

    let total = arabic + hebrew + latin + other;
    if total == 0 {
        return (Script::Empty, false);
    }

    // Script wins by majority. Hebrew is unambiguous — one Hebrew letter
    // means the token is not Arabic. Latin majority → Latin. Arabic
    // majority → Arabic (flag Persian if any Persian-specific present).
    let script = if hebrew > arabic && hebrew > latin {
        Script::Hebrew
    } else if latin > arabic && latin > hebrew {
        Script::Latin
    } else if arabic > 0 {
        if persian_specific > 0 {
            Script::PersianFamily
        } else {
            Script::Arabic
        }
    } else {
        Script::Other
    };

    (script, persian_specific > 0)
}

// ──────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────

/// Primary entry point — run the normalizer on a single token.
///
/// Returns a `Normalized` value with four views of the same input:
/// raw surface, stripped (safe), folded (aggressive), plus the detected
/// script. The analyzer pipeline can then pick the appropriate view at
/// each layer.
pub fn normalize(s: &str) -> Normalized {
    let stripped = strip_tashkeel_and_tatweel(s);
    let folded = fold_letters(&stripped);
    let (script, has_persian_letters) = classify_script(&stripped);
    Normalized {
        surface: s.to_string(),
        stripped,
        folded,
        script,
        has_persian_letters,
    }
}

/// Convenience wrapper for call sites that only want the stripped form.
/// Saves allocating a `Normalized` when the caller has no use for it.
pub fn normalize_stripped(s: &str) -> String {
    strip_tashkeel_and_tatweel(s)
}

/// Convenience wrapper for call sites that want the aggressive fold
/// (equivalent to classic Light10 preprocessing, minus the prefix/suffix
/// strip). Available for the Layer 3 fuzzy fallback.
pub fn normalize_folded(s: &str) -> String {
    fold_letters(&strip_tashkeel_and_tatweel(s))
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── strip_tashkeel_and_tatweel ──────────────────────────────────

    #[test]
    fn tashkeel_is_stripped() {
        // كَتَبَ → كتب
        assert_eq!(strip_tashkeel_and_tatweel("كَتَبَ"), "كتب");
        // bismillāh al-raḥmān al-raḥīm with full tashkeel
        assert_eq!(
            strip_tashkeel_and_tatweel("بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ"),
            "بسم الله الرحمن الرحيم"
        );
    }

    #[test]
    fn tatweel_is_stripped() {
        // كــتــاب → كتاب
        assert_eq!(strip_tashkeel_and_tatweel("كــتــاب"), "كتاب");
    }

    #[test]
    fn shadda_and_tanwin_are_stripped() {
        // مَدَّ (shadda) → مد
        assert_eq!(strip_tashkeel_and_tatweel("مَدَّ"), "مد");
        // كتابٌ (tanwīn) → كتاب
        assert_eq!(strip_tashkeel_and_tatweel("كتابٌ"), "كتاب");
    }

    #[test]
    fn dagger_alif_is_stripped() {
        // هَٰذَا → هذا
        assert_eq!(strip_tashkeel_and_tatweel("هَٰذَا"), "هذا");
        assert_eq!(strip_tashkeel_and_tatweel("ذَٰلِكَ"), "ذلك");
    }

    #[test]
    fn ascii_passes_through_stripping() {
        assert_eq!(strip_tashkeel_and_tatweel("hello"), "hello");
        assert_eq!(strip_tashkeel_and_tatweel(""), "");
    }

    // ── critical: the وائل case ─────────────────────────────────────

    #[test]
    fn wael_survives_stripping() {
        // This is THE critical test. Light10 over-strips this to ائل
        // because it treats و as a prefix. The normalizer must NOT
        // touch consonants — only tashkeel/tatweel — so stripped stays
        // character-identical when there's no diacritic to remove.
        assert_eq!(strip_tashkeel_and_tatweel("وائل"), "وائل");
        let n = normalize("وائل");
        assert_eq!(n.surface, "وائل");
        assert_eq!(n.stripped, "وائل");
        // Folded collapses ئ → ي but preserves every consonant position.
        // CRUCIALLY, the و is still there.
        assert_eq!(n.folded, "وايل");
    }

    // ── fold_letters ────────────────────────────────────────────────

    #[test]
    fn alif_variants_fold_to_plain_alif() {
        assert_eq!(fold_letters("أكل"), "اكل");
        assert_eq!(fold_letters("إسلام"), "اسلام");
        assert_eq!(fold_letters("آيات"), "ايات");
        assert_eq!(fold_letters("ٱلله"), "الله");
    }

    #[test]
    fn alif_maqsura_folds_to_ya() {
        assert_eq!(fold_letters("موسى"), "موسي");
        assert_eq!(fold_letters("إلى"), "الي");
    }

    #[test]
    fn ta_marbuta_folds_to_ha() {
        assert_eq!(fold_letters("مدرسة"), "مدرسه");
    }

    #[test]
    fn folding_is_idempotent() {
        let once = fold_letters("الأئمة");
        let twice = fold_letters(&once);
        assert_eq!(once, twice, "folding must be a fixed point on its own output");
    }

    // ── normalize (full pipeline) ───────────────────────────────────

    #[test]
    fn normalize_populates_all_views() {
        let n = normalize("كَتَبَ");
        assert_eq!(n.surface, "كَتَبَ");
        assert_eq!(n.stripped, "كتب");
        assert_eq!(n.folded, "كتب");
        assert_eq!(n.script, Script::Arabic);
        assert!(!n.has_persian_letters);
    }

    #[test]
    fn normalize_empty_string() {
        let n = normalize("");
        assert_eq!(n.surface, "");
        assert_eq!(n.stripped, "");
        assert_eq!(n.folded, "");
        assert_eq!(n.script, Script::Empty);
    }

    #[test]
    fn aʾimma_case_normalizes_but_does_not_decompose() {
        // الأئمة stripped should retain all letters; Layer 3 is the one
        // that decomposes it. Folded reveals that أ and ئ are hamza
        // bearers over plain alif / ya.
        let n = normalize("الأئمة");
        assert_eq!(n.stripped, "الأئمة");
        assert_eq!(n.folded, "الايمه");
    }

    // ── script detection ────────────────────────────────────────────

    #[test]
    fn arabic_token_is_detected_as_arabic() {
        assert_eq!(normalize("كتاب").script, Script::Arabic);
        assert_eq!(normalize("المعرفة").script, Script::Arabic);
    }

    #[test]
    fn persian_token_is_detected_as_persian_family() {
        // گربه (cat in Persian) contains گ which is Persian-specific.
        let n = normalize("گربه");
        assert_eq!(n.script, Script::PersianFamily);
        assert!(n.has_persian_letters);
    }

    #[test]
    fn latin_token_is_detected_as_latin() {
        assert_eq!(normalize("hello").script, Script::Latin);
        assert_eq!(normalize("café").script, Script::Latin);
    }

    #[test]
    fn hebrew_token_is_detected_as_hebrew() {
        assert_eq!(normalize("שלום").script, Script::Hebrew);
    }

    #[test]
    fn empty_token_is_empty_script() {
        assert_eq!(normalize("").script, Script::Empty);
        assert_eq!(normalize("   ").script, Script::Empty);
    }

    // ── convenience wrappers ────────────────────────────────────────

    #[test]
    fn convenience_wrappers_match_full_normalize() {
        let input = "كَتَبَ";
        let n = normalize(input);
        assert_eq!(normalize_stripped(input), n.stripped);
        assert_eq!(normalize_folded(input), n.folded);
    }

    // ── stability / regression ──────────────────────────────────────

    #[test]
    fn stripping_preserves_char_order() {
        // Non-tashkeel characters must appear in the same order.
        let s = "مَرَّحْباً";
        let stripped = strip_tashkeel_and_tatweel(s);
        // Expected: "مرحبا" (tanwīn ً stripped, shadda/fatḥa/sukūn stripped)
        assert_eq!(stripped, "مرحبا");
    }

    #[test]
    fn stripping_handles_mixed_script() {
        // Code-switching: "كتب Hello الكتاب"
        let s = "كَتَبَ Hello الكِتَاب";
        assert_eq!(strip_tashkeel_and_tatweel(s), "كتب Hello الكتاب");
    }
}
