//! Constellation Arabic Engine — core types.
//!
//! Design principle: **generative, not dictionary-bound**.
//! A word is modeled as `root × pattern + affixes`. Every surface form
//! is reproducible from its (root, pattern) pair; we never store the
//! full surface in a 40K-entry dictionary (Buckwalter's approach).
//!
//! All character data is UTF-8 Arabic. No Buckwalter transliteration
//! is used at the public API — we operate on native Arabic throughout.
//!
//! See `docs/CONSTELLATION-ARABIC-ENGINE.md` for the full spec.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 15 languages Constellation speaks.
///
/// This mirrors the locale codes in `src/lib/i18n/` exactly, so a single
/// `Lang` value round-trips between Rust FTS payloads and the Svelte
/// front-end without translation. Order matches the alphabetical order
/// of locale files for stable iteration in UI dropdowns.
///
/// Used by the `lexicon` module (multilingual bridge) and surfaces in
/// `Analysis.equivalents` as the HashMap key. Per the 2026-04-18 design
/// decisions: all 15 supported from day one, bidirectional lookup, both
/// a global settings default and a quick toggle in the search bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Lang {
    /// Arabic — العربية (RTL). Uses CAE (Constellation Arabic Engine).
    Ar,
    /// German — Deutsch.
    De,
    /// English.
    En,
    /// Spanish — Español.
    Es,
    /// Persian / Farsi — فارسی (RTL, shares script with Arabic).
    Fa,
    /// French — Français.
    Fr,
    /// Hebrew — עברית (RTL).
    He,
    /// Hindi — हिन्दी (Devanagari).
    Hi,
    /// Japanese — 日本語 (requires segmentation).
    Ja,
    /// Korean — 한국어.
    Ko,
    /// Portuguese — Português.
    Pt,
    /// Russian — Русский (Cyrillic).
    Ru,
    /// Turkish — Türkçe (agglutinative morphology).
    Tr,
    /// Urdu — اردو (RTL, Perso-Arabic script).
    Ur,
    /// Chinese — 中文 (requires segmentation).
    Zh,
}

impl Lang {
    /// Round-trips with the Constellation i18n locale codes.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ar => "ar",
            Lang::De => "de",
            Lang::En => "en",
            Lang::Es => "es",
            Lang::Fa => "fa",
            Lang::Fr => "fr",
            Lang::He => "he",
            Lang::Hi => "hi",
            Lang::Ja => "ja",
            Lang::Ko => "ko",
            Lang::Pt => "pt",
            Lang::Ru => "ru",
            Lang::Tr => "tr",
            Lang::Ur => "ur",
            Lang::Zh => "zh",
        }
    }

    /// Inverse of `code`. Returns `None` for unknown strings so callers
    /// can choose how to handle legacy payloads.
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "ar" => Lang::Ar,
            "de" => Lang::De,
            "en" => Lang::En,
            "es" => Lang::Es,
            "fa" => Lang::Fa,
            "fr" => Lang::Fr,
            "he" => Lang::He,
            "hi" => Lang::Hi,
            "ja" => Lang::Ja,
            "ko" => Lang::Ko,
            "pt" => Lang::Pt,
            "ru" => Lang::Ru,
            "tr" => Lang::Tr,
            "ur" => Lang::Ur,
            "zh" => Lang::Zh,
            _ => return None,
        })
    }

    /// Writing direction — affects rendering of the equivalents chip strip
    /// in search results.
    pub fn is_rtl(self) -> bool {
        matches!(self, Lang::Ar | Lang::Fa | Lang::He | Lang::Ur)
    }

    /// All 15 languages, in the canonical order. Useful for UI iteration.
    pub fn all() -> &'static [Lang] {
        &[
            Lang::Ar, Lang::De, Lang::En, Lang::Es, Lang::Fa, Lang::Fr,
            Lang::He, Lang::Hi, Lang::Ja, Lang::Ko, Lang::Pt, Lang::Ru,
            Lang::Tr, Lang::Ur, Lang::Zh,
        ]
    }
}

/// A triliteral or quadriliteral Arabic root.
///
/// Stored as the three (or four) *radical letters* in order:
///   كتب → Root { radicals: ['ك','ت','ب'] }
///   دحرج → Root { radicals: ['د','ح','ر','ج'] }
///
/// `hamza` variants (أ/إ/آ/ئ/ؤ/ء) are preserved on the radical — a hamza
/// is a root letter, not a vowel. The normalizer never strips them.
///
/// `weak` letters (ا/و/ي) are marked so patterns can apply sound-shift
/// rules (e.g. قول + فاعل → قائل, where the و becomes ء).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Root {
    /// 3 or 4 radical letters in dictionary order.
    pub radicals: Vec<char>,
    /// Root classification — used by the generator to apply correct
    /// morphophonemic rules.
    pub class: RootClass,
    /// Optional semantic hint ("speaking", "writing", …) for future
    /// semantic search layers. Free text, not required for stemming.
    pub gloss: Option<String>,
}

/// Classification of root letter types — drives the generator.
///
/// This is the *structural* shape of the root, not its meaning. It tells
/// the pattern applicator which phonological rules fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RootClass {
    /// Triliteral with all strong consonants: كتب، ضرب، جلس.
    SoundTriliteral,
    /// Triliteral with a weak first radical (ا/و/ي): وعد، يسر.
    /// Arabic grammarians call this `مِثَال`.
    AssimilatedTriliteral,
    /// Triliteral with a weak second radical: قول، بيع.
    /// Arabic grammarians call this `أَجْوَف`.
    HollowTriliteral,
    /// Triliteral with a weak third radical: دعا، رمى.
    /// Arabic grammarians call this `نَاقِص`.
    DefectiveTriliteral,
    /// Triliteral with identical 2nd and 3rd radicals: مدّ (م-د-د), شكّ (ش-ك-ك).
    /// Arabic grammarians call this `مُضَعَّف`.
    GeminatedTriliteral,
    /// Triliteral with a hamza in any position: سأل، قرأ، أكل.
    /// Arabic grammarians call this `مَهْمُوز`.
    HamzatedTriliteral,
    /// Four-radical sound root: دحرج، زلزل.
    SoundQuadriliteral,
    /// Four-radical root with a weak letter (rare).
    WeakQuadriliteral,
}

/// A morphological pattern — the abstract template that a root is cast into.
///
/// Patterns are represented using the classical Arabic placeholders:
///   `ف` (F) = first radical
///   `ع` (ʿ) = second radical
///   `ل` (L) = third radical
///   `ل`-geminate (L) = fourth radical (for quadriliterals, we use a second `ل`)
///
/// So the pattern for `كاتب` (writer, active participle of Form I) is written
/// as `فَاعِل` and applied to root ك-ت-ب by substituting:
///   ف → ك, ع → ت, ل → ب
/// yielding `كاتب`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pattern {
    /// Canonical pattern string with placeholders (ف/ع/ل).
    /// Includes full tashkeel so the generator produces correctly-vocalized
    /// surface forms when asked; readers may strip for display.
    pub template: String,
    /// Classification: verbal form, nominal, broken plural, participle, …
    pub kind: PatternKind,
    /// Which root classes this pattern accepts.
    /// Some patterns only apply to sound roots; others handle weak letters.
    pub accepts: Vec<RootClass>,
    /// Human-readable name for diagnostics and the learning UI.
    /// Examples: "فاعل — اسم فاعل للثلاثي", "أفعال — جمع تكسير".
    pub label_ar: String,
    pub label_en: String,
}

/// What kind of morphological role this pattern fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternKind {
    /// Perfect-tense verb stem: فَعَلَ (كتبَ), فَعِلَ (شربَ), فَعُلَ (كرُمَ).
    VerbPerfect,
    /// Imperfect-tense verb stem: يَفْعُل، يَفْعِل، يَفْعَل.
    VerbImperfect,
    /// Imperative stem: افْعُلْ، افْعِلْ، افْعَلْ.
    VerbImperative,
    /// Verbal noun (مصدر): كِتَابَة، ضَرْب، قِرَاءَة.
    VerbalNoun,
    /// Active participle (اسم فاعل): فَاعِل، مُفْعِل، مُفَاعِل.
    ActiveParticiple,
    /// Passive participle (اسم مفعول): مَفْعُول، مُفْعَل، مُفَاعَل.
    PassiveParticiple,
    /// Derived nominal (اسم مشتق — place, time, instrument, intensity).
    DerivedNoun,
    /// Broken plural (جمع تكسير) — 27 patterns, all 27 generated per قرار 3.
    BrokenPlural,
    /// Diminutive (اسم تصغير): فُعَيْل، فُعَيْعِل.
    Diminutive,
    /// Relative adjective (اسم منسوب): فَعْلِيّ.
    Relative,
    /// Elative / comparative (اسم تفضيل): أَفْعَل.
    Elative,
    /// Feminine of a nominal form — pattern + tā' marbūṭa.
    Feminine,
}

/// An affix — either a prefix or a suffix.
///
/// Prefixes stack left-to-right as written, suffixes right-to-left.
/// Stacking rules are encoded as `allows_after` so the analyzer knows
/// `فسيكتبونها` = (ف)(س)(ي)(كتب)(ون)(ها) is legal but (س)(ف)(...) is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affix {
    /// Surface form (e.g. "و", "ال", "ها", "ون").
    pub surface: String,
    /// Which slot this affix occupies.
    pub slot: AffixSlot,
    /// Grammatical function (conjunction, definite article, object pronoun, …).
    pub function: AffixFunction,
    /// Which affix slots are allowed to precede this one.
    /// Encoded as a sorted whitelist; empty means "nothing before me".
    pub allows_after: Vec<AffixSlot>,
}

/// Ordered slots for prefix stacking (outer → inner to the root).
///
/// The full prefix block follows this order strictly:
///   [Conjunction][Interrogative][Future][Preposition][Definite][Tense]
/// A word like `أفبالكتاب` = أ(Interrogative) + ف(Conjunction) + ب(Preposition) + ال(Definite) + كتاب
/// would be rejected by a naive analyzer; the slot order tells us that
/// interrogative can precede conjunction, but not vice-versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AffixSlot {
    // ── prefix slots (outer → inner) ────────────────────────────
    /// Interrogative hamza: أ (أتكتب؟).
    PrefixInterrogative,
    /// Conjunction: و ف.
    PrefixConjunction,
    /// Future marker: س (سيكتب).
    PrefixFuture,
    /// Preposition: ب ك ل (بالكتاب، كالكتاب، للكتاب).
    PrefixPreposition,
    /// Definite article: ال، لل (لل = ل + ال elided).
    PrefixDefinite,
    /// Imperfect tense prefix on verbs: ي ت ن أ.
    PrefixImperfect,

    // ── suffix slots (inner → outer) ────────────────────────────
    /// Feminine marker: ة/ت (fused with stem — special handling).
    SuffixFeminine,
    /// Number/gender marker on verbs & nouns: و ت ن ا ي.
    /// Examples: كتبتُ، كتبوا، المسلمون، المسلمات.
    SuffixNumber,
    /// Pronominal suffix: ه ها هم هن ك كم كن ي نا.
    /// Example: كتابه، كتابها، كتابهم.
    SuffixPronoun,
}

/// Grammatical function of an affix — richer than the slot; used by the
/// disambiguator and the learning UI.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AffixFunction {
    InterrogativeHamza,
    ConjunctionAnd,    // و
    ConjunctionThen,   // ف
    FutureSa,          // س
    FutureSawfa,       // سوف (rare, whole word)
    PrepositionBi,     // ب
    PrepositionKa,     // ك
    PrepositionLi,     // ل
    DefiniteAl,        // ال
    ImperfectYa,       // ي (3rd masc)
    ImperfectTa,       // ت (3rd fem / 2nd masc)
    ImperfectNa,       // ن (1st plural)
    ImperfectA,        // أ (1st singular)
    FeminineTa,        // ة
    DualAlif,          // ان (kitābāni)
    DualYa,            // ين (kitābayni)
    SoundFemPlural,    // ات (muʾmināt)
    SoundMascPluralWaw,// ون
    SoundMascPluralYa, // ين
    FemSingularYa,     // ي  (nisba or 1st-sg possessive)
    PronounHu,         // ه
    PronounHa,         // ها
    PronounHum,        // هم
    PronounHunna,      // هن
    PronounKa,         // ك
    PronounKum,        // كم
    PronounKunna,      // كن
    PronounI,          // ي  (when unambiguously pronoun, e.g. كتابي)
    PronounNa,         // نا
    // Verbal suffixes
    VerbPastTuMasc,    // تَ
    VerbPastTuFem,     // تِ
    VerbPastTu1s,      // تُ
    VerbPastTum,       // تم
    VerbPastTunna,     // تن
    VerbPastNa,        // نا
    VerbPastU,         // وا
    VerbPastAt,        // ت (feminine 3rd sg)
    VerbImperfectMascPl, // ون (on verbs — يكتبون)
    VerbImperfectFemPl,  // ن
    VerbImperfectJussiveNo, // elided
}

/// A complete morphological analysis of a surface word.
///
/// The analyzer returns `Vec<Analysis>` because Arabic is genuinely
/// ambiguous — `كتب` has at least three readings. The disambiguator
/// (Layer 4) ranks them; upstream FTS writes `lemma` to the index.
#[derive(Debug, Clone, Serialize)]
pub struct Analysis {
    /// The word exactly as it appeared in the source text (no normalization).
    /// This is what the Index displays per قرار 1 (ج): `surface + lemma + root`.
    pub surface: String,
    /// The dictionary headword — e.g. إمام for الأئمة, كتاب for وبالكتاب.
    /// This is the primary FTS indexing key.
    pub lemma: String,
    /// The root radicals, joined by U+002D HYPHEN-MINUS for storage:
    /// "ك-ت-ب", "ء-م-م". Secondary FTS index field for root-based search.
    pub root: String,
    /// Which pattern produced this surface from the root.
    pub pattern_label: String,
    /// Part-of-speech coarse tag (noun, verb, proper_noun, particle, foreign).
    pub pos: PartOfSpeech,
    /// Detected prefixes (in order of application, outer first).
    pub prefixes: Vec<AffixFunction>,
    /// Detected suffixes (in order of application, inner first).
    pub suffixes: Vec<AffixFunction>,
    /// Confidence 0.0–1.0. Set by the disambiguator; defaults to 1.0 for
    /// protected proper-noun hits and 0.5 for pure generative matches.
    pub confidence: f32,
    /// Origin layer that produced this analysis — for telemetry and the
    /// learning UI's "why was this chosen?" explanation.
    pub origin: AnalysisOrigin,
    /// Cross-lingual equivalents, populated by the Lexical Bridge
    /// (`crate::lexicon`). Key = target language; value = ordered list of
    /// equivalent lemmas in that language (most common first).
    ///
    /// Empty until the lexicon module resolves the lemma. Search uses
    /// this to expand queries: searching "المعرفة" matches English notes
    /// containing "knowledge", French notes containing "connaissance",
    /// etc. Populated for any analyzed word whose lemma has a bridge hit,
    /// regardless of the analyzed-word's source language.
    ///
    /// Skipped in JSON output when empty to keep FTS payloads compact.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub equivalents: HashMap<Lang, Vec<String>>,
    /// Which language produced this analysis. For CAE output this is
    /// always `Lang::Ar`; once per-language analyzers land, each fills
    /// its own value. Used by the UI to render a language badge.
    pub lang: Lang,
}

/// Which engine layer produced the analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisOrigin {
    /// Layer 2: matched the protected proper-noun / loanword list.
    ProtectedList,
    /// Layer 3: matched a (root, pattern) combination in the FST.
    GenerativeFst,
    /// Layer 3 fallback: no FST hit but surface rules gave a best guess.
    /// Used only when nothing else applies; confidence ≤ 0.3.
    SurfaceHeuristic,
    /// Layer 5: user-taught override in the current Universe.
    UserOverride,
}

/// Coarse part-of-speech tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    ProperNoun,
    Particle,
    /// Foreign / borrowed word (e.g. إنترنت, كمبيوتر). Treated as opaque;
    /// the engine returns it unchanged as both lemma and surface.
    Foreign,
    Unknown,
}
