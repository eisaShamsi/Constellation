//! Source-language detection from raw query text — M12-lang-detect.
//!
//! Needed by the M14 search wiring: the search hot path receives a raw
//! string from the user and needs to know which `Lang` to pass into
//! [`crate::lexicon::expand_to_match_expr`]. We can't ask the user to
//! tag every query — so we infer from the Unicode script ranges
//! present in the text.
//!
//! # Design
//!
//! Classification is a two-step decision:
//!
//! 1. **Count strong-script characters per family** (Arabic, Hebrew,
//!    Devanagari, Cyrillic, CJK-family, Latin). Digits, whitespace,
//!    punctuation, symbols, and script-less marks are ignored — they
//!    carry no language signal.
//! 2. **Pick the dominant family, then disambiguate within it** using
//!    script-exclusive characters:
//!    - **Arabic family** (Ar / Fa / Ur share the Arabic script):
//!      Urdu-distinctive letters (ٹ ڈ ڑ ں ے ۓ) → Ur. Else, Persian/Urdu
//!      letters not in Modern Standard Arabic (پ چ ژ گ ک ی) → Fa.
//!      Else Ar.
//!    - **CJK family** (Ja / Ko / Zh share the Han characters):
//!      any Hangul → Ko. Else any Hiragana / Katakana → Ja. Else Zh.
//!      Edge case: pure-Han Japanese text (e.g. "日本") is misclassified
//!      as Zh — unavoidable without a dictionary and acceptable because
//!      the lexicon expansion hits the same Han node in both cases.
//!    - **Latin family** (En / De / Es / Fr / Pt / Tr share the Latin
//!      alphabet): any Turkish-exclusive letter (ğ İ ı ş) → Tr. Else
//!      German ß → De. Else French œ → Fr. Else Spanish ñ → Es. Else
//!      Portuguese ã õ → Pt. Else En as pragmatic default.
//!
//! # Rationale for the Latin fallback
//!
//! Pure unaccented Latin text is genuinely ambiguous ("book" could be
//! English, Afrikaans, Dutch — none of which we care about here; "amor"
//! is Spanish / Portuguese / Latin). Returning En matches the user's
//! most-likely intent given Constellation's shipped languages and
//! preserves the rollback story — at worst the graph walks find no
//! expansions and we fall back to the un-expanded search.
//!
//! # Return value
//!
//! `None` signals the caller to skip lexicon expansion entirely — there
//! were no letter-bearing characters to classify. Numbers-only,
//! punctuation-only, and empty strings all return `None`. The caller
//! (M14 `lexical_search`) falls back to the plain FTS5 prefix match.

use crate::arabic::Lang;

/// Detect the most likely source language of a raw query string.
///
/// Returns `None` when the string has no letter-bearing characters the
/// classifier recognises — callers should fall back to the plain
/// un-expanded search path.
///
/// # Examples
///
/// ```ignore
/// use crate::lexicon::detect_source_lang;
/// use crate::arabic::Lang;
///
/// assert_eq!(detect_source_lang("book"),    Some(Lang::En));
/// assert_eq!(detect_source_lang("كتاب"),    Some(Lang::Ar));
/// assert_eq!(detect_source_lang("کتاب"),    Some(Lang::Fa));  // Persian kaf
/// assert_eq!(detect_source_lang("لڑکی"),    Some(Lang::Ur));  // Urdu retroflex
/// assert_eq!(detect_source_lang("본"),      Some(Lang::Ko));
/// assert_eq!(detect_source_lang("ほん"),    Some(Lang::Ja));
/// assert_eq!(detect_source_lang("   !!!"), None);
/// ```
pub fn detect_source_lang(s: &str) -> Option<Lang> {
    // Strong-script counters.
    let mut arabic = 0usize;
    let mut hebrew = 0usize;
    let mut devanagari = 0usize;
    let mut cyrillic = 0usize;
    let mut han = 0usize;
    let mut hiragana = 0usize;
    let mut katakana = 0usize;
    let mut hangul = 0usize;
    let mut latin = 0usize;

    // Arabic-family disambiguators.
    let mut urdu_mark = false;
    let mut perso_arabic_mark = false;

    // Latin-family disambiguators.
    let mut turkish_mark = false;
    let mut german_mark = false;
    let mut french_mark = false;
    let mut spanish_mark = false;
    let mut portuguese_mark = false;

    for c in s.chars() {
        let u = c as u32;

        // Arabic script block + supplements + presentation forms.
        if (0x0600..=0x06FF).contains(&u)
            || (0x0750..=0x077F).contains(&u)
            || (0x08A0..=0x08FF).contains(&u)
            || (0xFB50..=0xFDFF).contains(&u)
            || (0xFE70..=0xFEFF).contains(&u)
        {
            arabic += 1;
            // Urdu-distinctive letters (retroflex / Urdu yeh variants).
            // These appear almost exclusively in Urdu (and Kashmiri /
            // Pashto which we don't ship). Strong signal.
            if matches!(
                u,
                0x0679 // ٹ
                | 0x0688 // ڈ
                | 0x0691 // ڑ
                | 0x06BA // ں
                | 0x06D2 // ے
                | 0x06D3 // ۓ
            ) {
                urdu_mark = true;
            }
            // Persian / Urdu shared distinctive letters — present in
            // Fa and Ur but absent from Modern Standard Arabic.
            if matches!(
                u,
                0x067E // پ
                | 0x0686 // چ
                | 0x0698 // ژ
                | 0x06AF // گ
                | 0x06A9 // ک (Persian kaf)
                | 0x06CC // ی (Persian yeh)
            ) {
                perso_arabic_mark = true;
            }
            continue;
        }

        // Hebrew script + alphabetic presentation forms.
        if (0x0590..=0x05FF).contains(&u) || (0xFB1D..=0xFB4F).contains(&u) {
            hebrew += 1;
            continue;
        }

        // Devanagari.
        if (0x0900..=0x097F).contains(&u) {
            devanagari += 1;
            continue;
        }

        // Cyrillic (main + supplement).
        if (0x0400..=0x04FF).contains(&u) || (0x0500..=0x052F).contains(&u) {
            cyrillic += 1;
            continue;
        }

        // CJK Unified Ideographs (main + Extension A). Extension B lives
        // in the supplementary plane but is rare enough in search
        // queries that we don't bother with the surrogate range.
        if (0x4E00..=0x9FFF).contains(&u) || (0x3400..=0x4DBF).contains(&u) {
            han += 1;
            continue;
        }

        // Hiragana.
        if (0x3040..=0x309F).contains(&u) {
            hiragana += 1;
            continue;
        }

        // Katakana (main + phonetic extensions).
        if (0x30A0..=0x30FF).contains(&u) || (0x31F0..=0x31FF).contains(&u) {
            katakana += 1;
            continue;
        }

        // Hangul Syllables + Jamo + Compatibility Jamo.
        if (0xAC00..=0xD7AF).contains(&u)
            || (0x1100..=0x11FF).contains(&u)
            || (0x3130..=0x318F).contains(&u)
        {
            hangul += 1;
            continue;
        }

        // Latin — Basic (A–Z, a–z) + Latin-1 Supplement accented letters
        // + Latin Extended-A/B. Excludes digits (0–9 sit below A in the
        // ASCII block; we already skip them because the Basic range
        // starts at 0x41).
        if (0x0041..=0x005A).contains(&u)
            || (0x0061..=0x007A).contains(&u)
            || (0x00C0..=0x024F).contains(&u)
        {
            latin += 1;
            // Turkish: dotted/dotless I + breve G + cedilla S.
            if matches!(
                u,
                0x011E // Ğ
                | 0x011F // ğ
                | 0x0130 // İ
                | 0x0131 // ı
                | 0x015E // Ş
                | 0x015F // ş
            ) {
                turkish_mark = true;
            }
            // German: sharp s. (ä/ö/ü overlap with Turkish — ambiguous
            // on their own.)
            if matches!(u, 0x00DF /* ß */) {
                german_mark = true;
            }
            // French: ligature oe (œ/Œ). Extremely distinctive.
            if matches!(u, 0x0152 /* Œ */ | 0x0153 /* œ */) {
                french_mark = true;
            }
            // Spanish: ñ + inverted ? ! (the inverted punctuation is
            // picked up even outside the Latin-letter check — see
            // below).
            if matches!(u, 0x00D1 /* Ñ */ | 0x00F1 /* ñ */) {
                spanish_mark = true;
            }
            // Portuguese: ã õ (tilded vowels). ã exists in Spanish too
            // for technical terms but is overwhelmingly Portuguese in
            // natural text.
            if matches!(
                u,
                0x00C3 // Ã
                | 0x00E3 // ã
                | 0x00D5 // Õ
                | 0x00F5 // õ
            ) {
                portuguese_mark = true;
            }
            continue;
        }

        // Inverted punctuation signals Spanish even without any
        // accented letters.
        if matches!(u, 0x00A1 /* ¡ */ | 0x00BF /* ¿ */) {
            spanish_mark = true;
            continue;
        }

        // Everything else (digits, spaces, ASCII punctuation, emoji)
        // carries no language signal — ignore.
    }

    let cjk = han + hiragana + katakana + hangul;

    // Pick the dominant family. `max_by_key` picks the first max on ties
    // following iteration order; we list Arabic / CJK before Latin so
    // a short mixed query favours the meatier script signal.
    let families = [
        (arabic, Family::Arabic),
        (hebrew, Family::Hebrew),
        (devanagari, Family::Devanagari),
        (cyrillic, Family::Cyrillic),
        (cjk, Family::Cjk),
        (latin, Family::Latin),
    ];

    let (max_count, winner) = families.iter().copied().max_by_key(|(n, _)| *n)?;
    if max_count == 0 {
        return None;
    }

    let lang = match winner {
        Family::Arabic => {
            if urdu_mark {
                Lang::Ur
            } else if perso_arabic_mark {
                Lang::Fa
            } else {
                Lang::Ar
            }
        }
        Family::Hebrew => Lang::He,
        Family::Devanagari => Lang::Hi,
        Family::Cyrillic => Lang::Ru,
        Family::Cjk => {
            if hangul > 0 {
                Lang::Ko
            } else if hiragana + katakana > 0 {
                Lang::Ja
            } else {
                Lang::Zh
            }
        }
        Family::Latin => {
            if turkish_mark {
                Lang::Tr
            } else if german_mark {
                Lang::De
            } else if french_mark {
                Lang::Fr
            } else if spanish_mark {
                Lang::Es
            } else if portuguese_mark {
                Lang::Pt
            } else {
                Lang::En
            }
        }
    };

    Some(lang)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    Arabic,
    Hebrew,
    Devanagari,
    Cyrillic,
    Cjk,
    Latin,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- empty / non-letter inputs -----------------------------------

    #[test]
    fn empty_string_returns_none() {
        assert_eq!(detect_source_lang(""), None);
    }

    #[test]
    fn whitespace_only_returns_none() {
        assert_eq!(detect_source_lang("    \t\n  "), None);
    }

    #[test]
    fn digits_only_returns_none() {
        assert_eq!(detect_source_lang("12345"), None);
    }

    #[test]
    fn ascii_punctuation_only_returns_none() {
        assert_eq!(detect_source_lang("!?.,;:-()[]{}"), None);
    }

    #[test]
    fn emoji_only_returns_none() {
        // Emoji live in the Symbols blocks — no language signal.
        assert_eq!(detect_source_lang("🚀🔥✨"), None);
    }

    // --- single-script happy paths -----------------------------------

    #[test]
    fn english_plain_ascii_is_en() {
        assert_eq!(detect_source_lang("knowledge"), Some(Lang::En));
        assert_eq!(detect_source_lang("book"), Some(Lang::En));
        assert_eq!(detect_source_lang("The Quick Brown Fox"), Some(Lang::En));
    }

    #[test]
    fn arabic_text_is_ar() {
        assert_eq!(detect_source_lang("كتاب"), Some(Lang::Ar));
        assert_eq!(detect_source_lang("المعرفة"), Some(Lang::Ar));
        // Diacritics (tashkeel) live in the Arabic block — still Ar.
        assert_eq!(detect_source_lang("كِتَابٌ"), Some(Lang::Ar));
    }

    #[test]
    fn hebrew_text_is_he() {
        assert_eq!(detect_source_lang("ספר"), Some(Lang::He));
        assert_eq!(detect_source_lang("שלום"), Some(Lang::He));
    }

    #[test]
    fn devanagari_text_is_hi() {
        assert_eq!(detect_source_lang("पुस्तक"), Some(Lang::Hi));
        assert_eq!(detect_source_lang("ज्ञान"), Some(Lang::Hi));
    }

    #[test]
    fn cyrillic_text_is_ru() {
        assert_eq!(detect_source_lang("книга"), Some(Lang::Ru));
        assert_eq!(detect_source_lang("Пушкин"), Some(Lang::Ru));
    }

    // --- Arabic family disambiguation --------------------------------

    #[test]
    fn persian_kaf_distinguishes_fa_from_ar() {
        // کتاب uses Persian kaf (U+06A9), not Arabic kaf (U+0643).
        assert_eq!(detect_source_lang("کتاب"), Some(Lang::Fa));
    }

    #[test]
    fn persian_pe_che_zhe_gaf_trigger_fa() {
        assert_eq!(detect_source_lang("پنج"), Some(Lang::Fa));
        assert_eq!(detect_source_lang("چای"), Some(Lang::Fa));
        assert_eq!(detect_source_lang("ژاله"), Some(Lang::Fa));
        assert_eq!(detect_source_lang("گل"), Some(Lang::Fa));
    }

    #[test]
    fn urdu_retroflex_distinguishes_ur_from_fa() {
        // ڑ (U+0691) is Urdu-specific. The rest of the word is Persian-
        // script — but the retroflex flags it as Urdu.
        assert_eq!(detect_source_lang("لڑکی"), Some(Lang::Ur));
    }

    #[test]
    fn urdu_yeh_barree_triggers_ur() {
        // ے (U+06D2) — Urdu yeh-barree, appended to words at end.
        assert_eq!(detect_source_lang("ہے"), Some(Lang::Ur));
    }

    #[test]
    fn urdu_noon_ghunna_triggers_ur() {
        // ں (U+06BA) — Urdu noon ghunna, marking nasalisation.
        assert_eq!(detect_source_lang("ماں"), Some(Lang::Ur));
    }

    // --- CJK family disambiguation -----------------------------------

    #[test]
    fn pure_han_is_zh() {
        // Pragmatic default — a Japanese user typing "日本" will
        // misclassify as Zh. Documented limitation.
        assert_eq!(detect_source_lang("本"), Some(Lang::Zh));
        assert_eq!(detect_source_lang("中文"), Some(Lang::Zh));
    }

    #[test]
    fn hiragana_triggers_ja() {
        assert_eq!(detect_source_lang("ほん"), Some(Lang::Ja));
        assert_eq!(detect_source_lang("ありがとう"), Some(Lang::Ja));
    }

    #[test]
    fn katakana_triggers_ja() {
        assert_eq!(detect_source_lang("コンピュータ"), Some(Lang::Ja));
    }

    #[test]
    fn mixed_han_and_kana_is_ja() {
        // Japanese mixes kanji + kana routinely — the kana is the
        // distinctive signal that flips the classifier away from Zh.
        assert_eq!(detect_source_lang("東京の本"), Some(Lang::Ja));
        assert_eq!(detect_source_lang("日本語のほん"), Some(Lang::Ja));
    }

    #[test]
    fn hangul_triggers_ko() {
        assert_eq!(detect_source_lang("책"), Some(Lang::Ko));
        assert_eq!(detect_source_lang("한국어"), Some(Lang::Ko));
    }

    #[test]
    fn hangul_wins_over_han_when_mixed() {
        // Korean sometimes includes Han characters in academic writing.
        // Hangul presence should still win.
        assert_eq!(detect_source_lang("한국의 文化"), Some(Lang::Ko));
    }

    // --- Latin family disambiguation ---------------------------------

    #[test]
    fn turkish_dotless_i_triggers_tr() {
        assert_eq!(detect_source_lang("İstanbul"), Some(Lang::Tr));
        assert_eq!(detect_source_lang("kitaplık"), Some(Lang::Tr));
    }

    #[test]
    fn turkish_breve_g_triggers_tr() {
        assert_eq!(detect_source_lang("yağmur"), Some(Lang::Tr));
    }

    #[test]
    fn turkish_cedilla_s_triggers_tr() {
        assert_eq!(detect_source_lang("güneş"), Some(Lang::Tr));
    }

    #[test]
    fn german_sharp_s_triggers_de() {
        assert_eq!(detect_source_lang("Straße"), Some(Lang::De));
        assert_eq!(detect_source_lang("groß"), Some(Lang::De));
    }

    #[test]
    fn french_oe_ligature_triggers_fr() {
        assert_eq!(detect_source_lang("cœur"), Some(Lang::Fr));
        assert_eq!(detect_source_lang("œuvre"), Some(Lang::Fr));
    }

    #[test]
    fn spanish_n_tilde_triggers_es() {
        assert_eq!(detect_source_lang("España"), Some(Lang::Es));
        assert_eq!(detect_source_lang("niño"), Some(Lang::Es));
    }

    #[test]
    fn spanish_inverted_punctuation_triggers_es() {
        // ¿ and ¡ alone carry enough signal even without letters.
        assert_eq!(detect_source_lang("¿Cómo estás?"), Some(Lang::Es));
    }

    #[test]
    fn portuguese_tilded_vowels_trigger_pt() {
        assert_eq!(detect_source_lang("não"), Some(Lang::Pt));
        assert_eq!(detect_source_lang("coração"), Some(Lang::Pt));
    }

    #[test]
    fn shared_accents_without_distinctive_marks_fall_to_en() {
        // ü appears in both German and Turkish; é appears in French /
        // Spanish / Portuguese / English loanwords. Without a
        // distinctive marker, pragmatic fallback is En.
        assert_eq!(detect_source_lang("café"), Some(Lang::En));
        assert_eq!(detect_source_lang("über"), Some(Lang::En));
    }

    // --- mixed-script precedence -------------------------------------

    #[test]
    fn dominant_script_wins_in_mixed_query() {
        // Mostly Arabic, one English word — Arabic should win.
        assert_eq!(
            detect_source_lang("المعرفة book"),
            Some(Lang::Ar),
        );
    }

    #[test]
    fn latin_wins_when_dominant() {
        // Mostly English, one Arabic word.
        assert_eq!(
            detect_source_lang("knowledge and كتاب on the shelf"),
            Some(Lang::En),
        );
    }

    #[test]
    fn arabic_with_digits_still_arabic() {
        // Digits don't contribute to any family count.
        assert_eq!(detect_source_lang("كتاب 2026"), Some(Lang::Ar));
    }
}
