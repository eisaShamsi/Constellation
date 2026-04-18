//! Layer 2 — the protected list.
//!
//! The protected list is a hash-lookup table of words that the analyzer
//! must **never** decompose, no matter how tempting the surface shape
//! appears. A hit returns the verbatim surface as both `surface` and
//! `lemma` with `AnalysisOrigin::ProtectedList` and `confidence = 1.0`.
//!
//! # Why this layer exists
//!
//! The Light10 stemmer over-strips:
//!   - `وائل` → `ائل` (treats و as conjunction)
//!   - `إنترنت` → `نترن` (strips ن as imperfect prefix)
//!   - `محمد` → `مد` (strips م as participle prefix)
//!
//! Every one of these is a proper noun or loanword that carries no
//! morphological decomposition — there is no `root × pattern` analysis
//! because the word was not derived from an Arabic root. The fix is
//! brute-force: a curated list of ~20K entries drawn from Wikipedia
//! categories (CC BY-SA). The M1e milestone hand-picks 200 high-impact
//! entries so the pipeline behaves correctly on the common case while
//! the full corpus is assembled (M1g).
//!
//! # Matching strategy
//!
//! The analyzer queries the protected list using the **stripped** form
//! (tashkeel + tatweel removed) from the normalizer. This catches
//! `وَائِل` / `وائل` / `وائلَ` all as one entry — no sensitivity to the
//! writer's vowel choices. Hamza variants (`أ` / `إ` / `آ`) are
//! preserved because they are root letters; users who write `احمد`
//! without the hamza won't match the entry for `أحمد` — but the Layer 3
//! folded fallback will catch it with reduced confidence.
//!
//! # Extension
//!
//! Per قرار 2 (ا): all data embeds at compile time with `include_str!`,
//! so the binary still ships without external dependencies. A future
//! tiered mode would mmap a large FST off disk. For M1e we use a
//! compile-time `const` Rust array to keep the module self-contained
//! and debuggable; the switch to data files happens in M1g when the
//! list grows past ~1K entries.

use super::normalizer;
use super::types::{Analysis, AnalysisOrigin, Lang, PartOfSpeech};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

/// Why this word is protected. The category drives POS and origin-lang
/// defaults and surfaces in the analyzer's explanation UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtectedCategory {
    /// Arabic proper name (person): محمد، فاطمة، عمر، وائل.
    ProperNoun,
    /// Geographic proper noun: القاهرة، دمشق، اليمن.
    /// Geographic entries preserve the `ال` in the protected-list key.
    Place,
    /// Loanword from another language: إنترنت، كمبيوتر، بنك.
    Loanword,
    /// Common function word / particle too short or too irregular to
    /// decompose safely: هذا، ذلك، الذي، متى.
    Function,
}

/// One entry in the protected table.
#[derive(Debug, Clone)]
pub struct ProtectedEntry {
    /// Canonical surface (stripped of tashkeel, with hamza variants
    /// preserved). This is the lookup key.
    pub lemma: String,
    pub category: ProtectedCategory,
    /// Coarse POS — mostly Noun / ProperNoun / Foreign.
    pub pos: PartOfSpeech,
    /// For loanwords, the language of origin — lets the lexical bridge
    /// connect إنترنت ↔ "internet" automatically. `None` for native.
    pub origin_lang: Option<Lang>,
}

impl ProtectedEntry {
    /// Promote this entry into a full `Analysis` — the shape the
    /// analyzer pipeline emits to downstream consumers.
    pub fn to_analysis(&self, surface: &str) -> Analysis {
        Analysis {
            surface: surface.to_string(),
            lemma: self.lemma.clone(),
            root: String::new(), // no root — this is non-decomposable
            pattern_label: match self.category {
                ProtectedCategory::ProperNoun => "proper-noun".to_string(),
                ProtectedCategory::Place => "place".to_string(),
                ProtectedCategory::Loanword => "loanword".to_string(),
                ProtectedCategory::Function => "function-word".to_string(),
            },
            pos: self.pos,
            prefixes: Vec::new(),
            suffixes: Vec::new(),
            confidence: 1.0,
            origin: AnalysisOrigin::ProtectedList,
            equivalents: HashMap::new(),
            lang: Lang::Ar,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Seed data — the M1e 200-entry hand-picked list.
//
// Each row is (surface, category, pos, origin_lang).
// The surface must already be stripped of tashkeel. Hamza variants
// (أ إ آ ؤ ئ) are preserved. Names appear in their most common
// orthographic form.
//
// Selection criteria:
//   - Name / place / loanword whose prefix coincidentally matches an
//     Arabic clitic (و / أ / م / ال / ب / ك / ل) and would be over-stripped.
//   - High-frequency: the entry must appear in at least 1-in-10K words
//     of a modern Arabic corpus.
//   - Short enough that surface collision matters (≤ 6 letters, typically).
//
// The full 20K corpus comes in M1g from Wikipedia category extraction.
// ──────────────────────────────────────────────────────────────────────

type Seed = (&'static str, ProtectedCategory, PartOfSpeech, Option<Lang>);

const SEED: &[Seed] = &[
    // ── Proper nouns: people (masculine) — the critical وائل case and peers
    ("وائل",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("محمد",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("أحمد",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("علي",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("عمر",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("عثمان",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("يوسف",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("إبراهيم",   ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("إسماعيل",   ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("موسى",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("عيسى",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("يعقوب",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("داود",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("سليمان",    ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("خالد",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("حسن",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("حسين",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("عبدالله",   ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("عبدالرحمن", ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("طارق",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("ياسر",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("سامي",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("ماجد",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("فيصل",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("بدر",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("أسامة",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("أنس",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("زياد",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("رائد",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("صالح",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("مازن",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("نبيل",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),

    // ── Proper nouns: people (feminine)
    ("فاطمة",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("عائشة",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("خديجة",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("مريم",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("زينب",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("سارة",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("ليلى",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("نور",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("هدى",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("أمل",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("رنا",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("هند",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("دينا",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("رانيا",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("سلمى",      ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("إيمان",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("جميلة",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("منى",       ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),
    ("نجلاء",     ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun, None),

    // ── Places: countries
    ("السعودية",  ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("مصر",       ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("العراق",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("سوريا",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("لبنان",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الأردن",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("فلسطين",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("اليمن",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("عمان",      ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الكويت",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("البحرين",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("قطر",       ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الإمارات",  ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("المغرب",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الجزائر",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("تونس",      ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("ليبيا",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("السودان",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("موريتانيا", ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الصومال",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("جيبوتي",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),

    // ── Places: cities
    ("القاهرة",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("بغداد",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("دمشق",      ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("بيروت",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الرياض",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("جدة",       ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("مكة",       ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("المدينة",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("أبوظبي",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("دبي",       ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الدوحة",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الإسكندرية", ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الخرطوم",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الرباط",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("طرابلس",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("صنعاء",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("القدس",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("غزة",       ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("حلب",       ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("الموصل",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("البصرة",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),

    // ── Places: non-Arab world (common refs)
    ("أمريكا",    ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::En)),
    ("بريطانيا",  ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::En)),
    ("فرنسا",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Fr)),
    ("ألمانيا",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::De)),
    ("إسبانيا",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Es)),
    ("إيطاليا",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, None),
    ("روسيا",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Ru)),
    ("الصين",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Zh)),
    ("اليابان",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Ja)),
    ("كوريا",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Ko)),
    ("الهند",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Hi)),
    ("تركيا",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Tr)),
    ("إيران",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Fa)),
    ("باكستان",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Ur)),
    ("لندن",      ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::En)),
    ("باريس",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Fr)),
    ("برلين",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::De)),
    ("نيويورك",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::En)),
    ("موسكو",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Ru)),
    ("طوكيو",     ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Ja)),
    ("إسطنبول",   ProtectedCategory::Place, PartOfSpeech::ProperNoun, Some(Lang::Tr)),

    // ── Loanwords: technology / computing
    ("إنترنت",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("كمبيوتر",   ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("تلفزيون",   ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("تلفون",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("راديو",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("فيديو",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("موبايل",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("لابتوب",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("تابلت",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("سيرفر",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("باسورد",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("إيميل",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("فيسبوك",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("تويتر",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("يوتيوب",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("جوجل",      ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),

    // ── Loanwords: finance / business
    // (Italian-origin loanwords: we don't carry Italian in the 15-language
    // bridge, so they're tagged with `None` origin — they still benefit from
    // being protected against prefix stripping.)
    ("بنك",       ProtectedCategory::Loanword, PartOfSpeech::Foreign, None), /* Italian banca via Ottoman */
    ("فيزا",      ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("فاتورة",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, None), /* Italian fattura */
    ("دولار",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("يورو",      ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("شيك",       ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("بوليصة",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, None), /* Italian polizza */

    // ── Loanwords: transport
    ("أوتوبيس",   ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("تاكسي",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("سيارة",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, None), /* arabized */
    ("طيارة",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, None),
    ("قطار",      ProtectedCategory::Loanword, PartOfSpeech::Foreign, None),
    ("ميترو",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::Fr)),
    ("باص",       ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),

    // ── Loanwords: household / food
    ("تلفاز",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("سندوتش",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("بيتزا",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, None), /* Italian pizza */
    ("قهوة",      ProtectedCategory::Loanword, PartOfSpeech::Foreign, None),
    ("شاي",       ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::Zh)),
    ("شاورما",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::Tr)),
    ("بطاطس",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("طماطم",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),

    // ── Loanwords: scientific / medical
    ("كيميا",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, None),
    ("فيزياء",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, None),
    ("بيولوجيا",  ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("جيولوجيا",  ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("أكسجين",    ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("هيدروجين",  ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("فيتامين",   ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("فيروس",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("بكتيريا",   ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("إنزيم",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),
    ("هرمون",     ProtectedCategory::Loanword, PartOfSpeech::Foreign, Some(Lang::En)),

    // ── Common function words / particles (non-decomposable)
    ("هذا",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("هذه",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("هؤلاء",     ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("ذلك",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("تلك",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("أولئك",     ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("الذي",      ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("التي",      ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("الذين",     ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("اللواتي",   ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("اللاتي",    ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("متى",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("أين",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("كيف",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("لماذا",     ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("ماذا",      ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("أيضا",      ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("إذن",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("لكن",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("بل",        ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("قد",        ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("لقد",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("إن",        ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("إنما",      ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("كي",        ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("لكي",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("عندما",     ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("حيث",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("حتى",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
    ("فقط",       ProtectedCategory::Function, PartOfSpeech::Particle, None),
];

// ──────────────────────────────────────────────────────────────────────
// Loaded table — built once per process from `SEED`.
// ──────────────────────────────────────────────────────────────────────

/// Process-wide singleton keyed by the tashkeel-stripped surface.
static TABLE: OnceLock<HashMap<String, ProtectedEntry>> = OnceLock::new();

fn build_table() -> HashMap<String, ProtectedEntry> {
    let mut map = HashMap::with_capacity(SEED.len() + 16);
    for &(surface, category, pos, origin_lang) in SEED {
        // Defensive: the seed should already be stripped. We normalize
        // here to tolerate accidental diacritics in future edits.
        let lemma = normalizer::normalize_stripped(surface);
        map.insert(
            lemma.clone(),
            ProtectedEntry { lemma, category, pos, origin_lang },
        );
    }
    map
}

fn table() -> &'static HashMap<String, ProtectedEntry> {
    TABLE.get_or_init(build_table)
}

// ──────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────

/// Exact-match lookup on the tashkeel-stripped surface. This is the
/// primary entry point used by the analyzer pipeline (Layer 2).
///
/// Returns `None` if the word is not in the protected list — the caller
/// then falls through to Layer 3 (the generative FST).
pub fn lookup(stripped: &str) -> Option<&'static ProtectedEntry> {
    if stripped.is_empty() { return None; }
    table().get(stripped)
}

/// Loose lookup that also accepts the aggressive-folded form — used by
/// Layer 3's final fallback. Returns any entry whose stripped form
/// folds to the same value.
///
/// This is O(n) over the table, so it's only ever called after strict
/// matching has failed. For M1e (~200 entries) this is negligible; for
/// M1g (20K entries) it will be replaced with a reverse index on the
/// folded form.
pub fn lookup_folded(folded: &str) -> Vec<&'static ProtectedEntry> {
    if folded.is_empty() { return Vec::new(); }
    let mut hits = Vec::new();
    for entry in table().values() {
        if normalizer::fold_letters(&entry.lemma) == folded {
            hits.push(entry);
        }
    }
    hits
}

/// Number of protected entries loaded.
pub fn len() -> usize {
    table().len()
}

/// Iterate all entries — used by tests and by the settings UI's "show me
/// what's protected in my Universe" inspector.
pub fn iter() -> impl Iterator<Item = &'static ProtectedEntry> {
    table().values()
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── THE critical case ───────────────────────────────────────────

    #[test]
    fn wael_is_protected() {
        // The flagship fix: Light10 over-strips وائل → ائل. CAE must
        // return it verbatim.
        let entry = lookup("وائل").expect("وائل must be in protected list");
        assert_eq!(entry.lemma, "وائل");
        assert_eq!(entry.category, ProtectedCategory::ProperNoun);
        assert_eq!(entry.pos, PartOfSpeech::ProperNoun);
    }

    #[test]
    fn wael_analysis_has_max_confidence() {
        let entry = lookup("وائل").unwrap();
        let a = entry.to_analysis("وائل");
        assert_eq!(a.surface, "وائل");
        assert_eq!(a.lemma, "وائل");
        assert_eq!(a.confidence, 1.0);
        assert!(matches!(a.origin, AnalysisOrigin::ProtectedList));
        assert!(a.prefixes.is_empty());
        assert!(a.suffixes.is_empty());
        assert!(a.root.is_empty(), "proper nouns have no root");
    }

    // ── other proper nouns ──────────────────────────────────────────

    #[test]
    fn common_names_are_protected() {
        for name in ["محمد", "أحمد", "فاطمة", "عائشة", "خالد", "مريم"] {
            assert!(lookup(name).is_some(), "{name} should be protected");
        }
    }

    #[test]
    fn places_are_protected() {
        for place in ["القاهرة", "دمشق", "بغداد", "بيروت", "مكة"] {
            assert!(lookup(place).is_some(), "{place} should be protected");
        }
    }

    #[test]
    fn loanwords_are_protected_with_origin_lang() {
        let e = lookup("إنترنت").expect("إنترنت must be protected");
        assert_eq!(e.category, ProtectedCategory::Loanword);
        assert_eq!(e.pos, PartOfSpeech::Foreign);
        assert_eq!(e.origin_lang, Some(Lang::En));
    }

    // ── tashkeel resilience ─────────────────────────────────────────

    #[test]
    fn vocalized_surface_still_matches_after_normalization() {
        // The analyzer runs the normalizer first, so it'll call lookup
        // with the stripped form. Simulate that here.
        let stripped = normalizer::normalize_stripped("وَائِل");
        assert_eq!(stripped, "وائل");
        assert!(lookup(&stripped).is_some());
    }

    #[test]
    fn unknown_word_returns_none() {
        // A decomposable ordinary Arabic word should not be protected.
        assert!(lookup("المعرفة").is_none());
        assert!(lookup("الكتاب").is_none());
        assert!(lookup("يكتبون").is_none());
    }

    #[test]
    fn empty_lookup_is_none() {
        assert!(lookup("").is_none());
    }

    // ── folded lookup (loose) ───────────────────────────────────────

    #[test]
    fn folded_lookup_catches_alif_variant() {
        // User types احمد without the hamza — folded lookup should find أحمد.
        let folded = normalizer::fold_letters("احمد");
        let hits = lookup_folded(&folded);
        assert!(
            hits.iter().any(|e| e.lemma == "أحمد"),
            "folded lookup of {folded} should find أحمد"
        );
    }

    // ── table shape ─────────────────────────────────────────────────

    #[test]
    fn table_has_expected_size() {
        // Sanity check: ~200 hand-picked entries for M1e.
        let n = len();
        assert!(n >= 180, "expected at least 180 protected entries, got {n}");
        assert!(n <= 260, "protected table grew unexpectedly: {n} entries");
    }

    #[test]
    fn no_duplicate_lemmas_in_seed() {
        // Build the table from scratch and compare to a HashSet of seed
        // lemmas — any collisions would silently overwrite and hide bugs.
        let seed_lemmas: HashSet<String> = SEED
            .iter()
            .map(|(s, _, _, _)| normalizer::normalize_stripped(s))
            .collect();
        assert_eq!(
            seed_lemmas.len(),
            SEED.len(),
            "duplicate surface in SEED — check the const array"
        );
    }

    #[test]
    fn every_entry_has_nonempty_lemma() {
        for entry in iter() {
            assert!(!entry.lemma.is_empty());
        }
    }

    // ── category coverage ───────────────────────────────────────────

    #[test]
    fn every_category_has_entries() {
        let mut by_cat = HashMap::<ProtectedCategory, usize>::new();
        for entry in iter() {
            *by_cat.entry(entry.category).or_insert(0) += 1;
        }
        assert!(by_cat.get(&ProtectedCategory::ProperNoun).copied().unwrap_or(0) >= 30);
        assert!(by_cat.get(&ProtectedCategory::Place).copied().unwrap_or(0) >= 30);
        assert!(by_cat.get(&ProtectedCategory::Loanword).copied().unwrap_or(0) >= 30);
        assert!(by_cat.get(&ProtectedCategory::Function).copied().unwrap_or(0) >= 20);
    }
}
