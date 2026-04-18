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
//! tiered mode would mmap a large FST off disk. M1e started with a
//! compile-time `const` Rust array (~200 entries, self-contained and
//! debuggable); M1g/M1h switched to `protected_seed.tsv` once the list
//! grew past 1K entries — now matching the `roots_seed.tsv` pattern.

use super::normalizer;
use super::types::{Analysis, AnalysisOrigin, Lang, PartOfSpeech};
use std::collections::HashMap;
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
// Seed data — external TSV file loaded at startup (M1g/M1h).
//
// The seed corpus lives in `protected_seed.tsv` alongside this file and
// is embedded into the binary via `include_str!`. Each non-comment,
// non-blank line is `surface<TAB>category<TAB>origin_lang`:
//
//   surface     — Arabic surface form, tashkeel-stripped, hamza preserved.
//   category    — `proper` | `place` | `loanword` | `function`.
//   origin_lang — BCP-47 code from {ar, de, en, es, fa, fr, he, hi, ja,
//                 ko, pt, ru, tr, ur, zh}, or `-` for None.
//
// `PartOfSpeech` is derived from `category` (no redundant column):
//   proper/place → ProperNoun, loanword → Foreign, function → Particle.
//
// The file grows append-only; `build_table` applies first-write-wins on
// duplicate surfaces so reordering or pasting an already-present entry
// is always safe.
//
// Selection criteria (the bar for adding an entry):
//   - Name / place / loanword whose prefix coincidentally matches an
//     Arabic clitic (و / أ / م / ال / ب / ك / ل) and would be over-stripped.
//   - High-frequency: expected to appear in real user text.
//   - Non-decomposable: no (root × pattern) analysis exists.
//
// Sourcing: hand-curated from public-domain references. No BAMA /
// Buckwalter / SAMA / GPL data is used. Ramp target: 20K proper nouns
// + 2K loanwords from CC-BY-SA Wikipedia extraction (future milestone);
// v1 ships ~1200 high-impact entries.
// ──────────────────────────────────────────────────────────────────────

/// Raw TSV text embedded at compile time. Zero I/O at runtime — parse
/// happens lazily inside [`build_table`] on first `table()` call.
const PROTECTED_TSV: &str = include_str!("protected_seed.tsv");

/// Parse a BCP-47 language tag from the TSV's third column.
///
/// Returns `None` for the sentinel `-` (native Arabic or an origin we
/// don't carry in the 15-language bridge). Unknown tags also return
/// `None` — they're treated as "no known origin" rather than halting
/// the build, so typos in the TSV degrade gracefully instead of
/// disabling protection of a valid surface.
fn parse_origin_lang(tag: &str) -> Option<Lang> {
    match tag {
        "ar" => Some(Lang::Ar),
        "de" => Some(Lang::De),
        "en" => Some(Lang::En),
        "es" => Some(Lang::Es),
        "fa" => Some(Lang::Fa),
        "fr" => Some(Lang::Fr),
        "he" => Some(Lang::He),
        "hi" => Some(Lang::Hi),
        "ja" => Some(Lang::Ja),
        "ko" => Some(Lang::Ko),
        "pt" => Some(Lang::Pt),
        "ru" => Some(Lang::Ru),
        "tr" => Some(Lang::Tr),
        "ur" => Some(Lang::Ur),
        "zh" => Some(Lang::Zh),
        _ => None, // "-" or unknown
    }
}

/// Map a category tag to its `ProtectedCategory` + derived POS.
///
/// The POS is strictly 1:1 with the category (places and people are
/// both proper nouns; loanwords are always Foreign; function words are
/// always Particle), so storing POS as a separate column would just be
/// redundant noise the author could get wrong. Returns `None` on
/// unknown category tags so the caller can skip the row.
fn parse_category(tag: &str) -> Option<(ProtectedCategory, PartOfSpeech)> {
    match tag {
        "proper"   => Some((ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun)),
        "place"    => Some((ProtectedCategory::Place,      PartOfSpeech::ProperNoun)),
        "loanword" => Some((ProtectedCategory::Loanword,   PartOfSpeech::Foreign)),
        "function" => Some((ProtectedCategory::Function,   PartOfSpeech::Particle)),
        _ => None,
    }
}

/// Parse the TSV into an iterator of parsed rows. Skips blank lines and
/// anything starting with '#'. Rows with fewer than three tab-separated
/// columns are dropped silently — the file has a fixed column count, so
/// a short row indicates a hand-edit bug that tests will surface via
/// the corpus-size assertion.
fn parse_tsv(
    tsv: &str,
) -> impl Iterator<Item = (&str, ProtectedCategory, PartOfSpeech, Option<Lang>)> {
    tsv.lines().filter_map(|line| {
        let line = line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let mut cols = line.splitn(3, '\t');
        let surface = cols.next()?.trim();
        let category_tag = cols.next()?.trim();
        let origin_tag = cols.next()?.trim();
        if surface.is_empty() || category_tag.is_empty() {
            return None;
        }
        let (category, pos) = parse_category(category_tag)?;
        let origin_lang = parse_origin_lang(origin_tag);
        Some((surface, category, pos, origin_lang))
    })
}

/// Public accessor for the embedded seed text. Mirrors
/// [`super::roots::seed_tsv`] so future tools (lint, diff, inspector UI)
/// can reach the same bytes the parser sees.
pub fn seed_tsv() -> &'static str {
    PROTECTED_TSV
}

// ── Migration note ────────────────────────────────────────────────────
// The legacy in-source seed array (M1e) used this shape:
//
//   type Seed = (&'static str, ProtectedCategory, PartOfSpeech, Option<Lang>);
//   const SEED: &[Seed] = &[ ("وائل", ProperNoun, ProperNoun, None), ... ];
//
// All data now lives in `protected_seed.tsv`. The comment preserved so
// repo-wide greps for `type Seed` or `const SEED` land here instead of
// silently returning no hits and raising a "did we lose the data?" alarm.
// ──────────────────────────────────────────────────────────────────────


// ──────────────────────────────────────────────────────────────────────
// Loaded table — built once per process from the embedded TSV.
// ──────────────────────────────────────────────────────────────────────

/// Process-wide singleton keyed by the tashkeel-stripped surface.
static TABLE: OnceLock<HashMap<String, ProtectedEntry>> = OnceLock::new();

fn build_table() -> HashMap<String, ProtectedEntry> {
    // Rough estimate: average row is ~20 bytes of TSV text. Over-
    // allocating slightly is cheaper than rehashing mid-insert.
    let approx = PROTECTED_TSV.len() / 20;
    let mut map = HashMap::with_capacity(approx.max(256));
    for (surface, category, pos, origin_lang) in parse_tsv(PROTECTED_TSV) {
        // Defensive: the seed should already be stripped. We normalize
        // here to tolerate accidental diacritics in future edits, and to
        // keep the `vocalized_surface_still_matches` invariant exact.
        let lemma = normalizer::normalize_stripped(surface);
        if lemma.is_empty() {
            continue;
        }
        // First-write-wins — duplicates across sections (e.g. a name
        // that also appears under loanwords) are tolerated silently.
        // The first occurrence in the TSV is authoritative; the
        // follow-up is ignored. Matches `roots::RootsIndex::build`.
        map.entry(lemma.clone())
            .or_insert(ProtectedEntry { lemma, category, pos, origin_lang });
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
        // Sanity check: M1g/M1h shipped ~1200 entries drawn from
        // public-domain references. Upper bound is deliberately loose
        // (2000) so normal append-only growth doesn't break tests; the
        // next resize up is the M1g-data pass to 20K proper nouns, at
        // which point this bound gets retuned.
        let n = len();
        assert!(n >= 800, "expected at least 800 protected entries, got {n}");
        assert!(n <= 2000, "protected table grew unexpectedly: {n} entries");
    }

    #[test]
    fn tsv_parses_to_at_least_as_many_entries_as_the_table() {
        // The table applies first-write-wins on duplicate surfaces, so
        // `len()` is a lower bound on TSV row count. Any accidental
        // explosion of duplicate rows (e.g. a bad copy-paste) would
        // show up here as a wide gap — the test passes today with a
        // tight bound (≤ 1 duplicate per 100 rows) and flags a diff
        // bomb if the TSV ever gains significantly more dupes.
        let row_count = parse_tsv(PROTECTED_TSV).count();
        let table_size = len();
        assert!(
            row_count >= table_size,
            "TSV row count {row_count} is somehow less than loaded table size {table_size}"
        );
        let dupes = row_count - table_size;
        assert!(
            dupes * 100 <= row_count,
            "TSV has {dupes} duplicate rows out of {row_count} — dedupe before committing"
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
        // Minimums reflect the M1g/M1h ramp (proper ≥ 300, place ≥ 200,
        // loanword ≥ 300, function ≥ 50). Kept well below actual counts
        // so ordinary curation edits don't break the test.
        assert!(by_cat.get(&ProtectedCategory::ProperNoun).copied().unwrap_or(0) >= 300);
        assert!(by_cat.get(&ProtectedCategory::Place).copied().unwrap_or(0) >= 200);
        assert!(by_cat.get(&ProtectedCategory::Loanword).copied().unwrap_or(0) >= 300);
        assert!(by_cat.get(&ProtectedCategory::Function).copied().unwrap_or(0) >= 50);
    }

    // ── TSV-parser unit tests ───────────────────────────────────────

    #[test]
    fn parse_origin_lang_handles_sentinel_and_known_codes() {
        assert_eq!(parse_origin_lang("-"), None);
        assert_eq!(parse_origin_lang(""), None);
        assert_eq!(parse_origin_lang("xx"), None); // unknown ⇒ None, not panic
        assert_eq!(parse_origin_lang("en"), Some(Lang::En));
        assert_eq!(parse_origin_lang("zh"), Some(Lang::Zh));
        assert_eq!(parse_origin_lang("ar"), Some(Lang::Ar));
    }

    #[test]
    fn parse_category_rejects_unknown() {
        assert_eq!(parse_category("unknown-tag"), None);
        assert_eq!(parse_category(""), None);
        assert!(matches!(
            parse_category("proper"),
            Some((ProtectedCategory::ProperNoun, PartOfSpeech::ProperNoun))
        ));
        assert!(matches!(
            parse_category("loanword"),
            Some((ProtectedCategory::Loanword, PartOfSpeech::Foreign))
        ));
        assert!(matches!(
            parse_category("function"),
            Some((ProtectedCategory::Function, PartOfSpeech::Particle))
        ));
        assert!(matches!(
            parse_category("place"),
            Some((ProtectedCategory::Place, PartOfSpeech::ProperNoun))
        ));
    }

    #[test]
    fn parse_tsv_skips_comments_and_blanks() {
        let input = "\
# comment
\r
وائل\tproper\t-
\t\t
# another
محمد\tproper\t-
bad-row-only-two-cols\tproper
عمر\tproper\t-
";
        let rows: Vec<_> = parse_tsv(input).collect();
        assert_eq!(rows.len(), 3, "expected 3 valid rows, got {rows:#?}");
        assert_eq!(rows[0].0, "وائل");
        assert_eq!(rows[1].0, "محمد");
        assert_eq!(rows[2].0, "عمر");
    }

    #[test]
    fn parse_tsv_drops_unknown_category() {
        let input = "وائل\tproper\t-\nfoo\tgibberish\t-\n";
        let rows: Vec<_> = parse_tsv(input).collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "وائل");
    }

    #[test]
    fn seed_tsv_accessor_returns_embedded_bytes() {
        // The accessor must return the same slice `include_str!` gave
        // us — consumers in the lexical bridge may content-address the
        // list by hashing these bytes.
        let s = seed_tsv();
        assert!(!s.is_empty());
        assert!(s.contains("وائل"));
        assert!(s.contains("\tproper\t")); // at least one proper row
    }

    #[test]
    fn first_write_wins_on_duplicate_surface() {
        // If the TSV ever re-lists a surface under a different category,
        // the build keeps the first occurrence. Prove it.
        let input = "كريم\tproper\t-\nكريم\tloanword\ten\n";
        let mut map: HashMap<String, ProtectedEntry> = HashMap::new();
        for (surface, category, pos, origin_lang) in parse_tsv(input) {
            let lemma = normalizer::normalize_stripped(surface);
            map.entry(lemma.clone()).or_insert(ProtectedEntry {
                lemma,
                category,
                pos,
                origin_lang,
            });
        }
        assert_eq!(map.len(), 1);
        let e = map.get("كريم").unwrap();
        assert_eq!(e.category, ProtectedCategory::ProperNoun);
        assert_eq!(e.origin_lang, None);
    }
}
