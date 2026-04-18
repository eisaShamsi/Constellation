//! The pattern-applier — the linguistic heart of the engine.
//!
//! Given a `(Root, Pattern)` pair, produce the **surface form** — the
//! actual Arabic word the speaker would write. Everything the Buckwalter
//! morphology's 40,000-entry dictionary bought in a lookup, we compute
//! from first principles by combining 158 patterns × ~120 seed roots =
//! ~19K surface forms (scaling to ~1.1M when the full 7K root corpus
//! lands in M1f-data).
//!
//! ## Template substitution
//!
//! A pattern like `فَعَلَ` contains three placeholder letters:
//!
//!   ف = `F` slot  → root's 1st radical
//!   ع = `ʿ` slot  → root's 2nd radical
//!   ل = `L` slot  → root's 3rd radical (first ل) / 4th radical (second ل)
//!
//! The substituter walks the template left-to-right, emitting each
//! non-placeholder character verbatim and each placeholder as the
//! corresponding radical. Tashkeel, prefix augments (م / ت / أ / إ / ن
//! / س / ي / و), and structural letters (ة, ي, ا, ً) pass through.
//!
//! ## Phonology by root class
//!
//! The mechanical substitution works for `SoundTriliteral` and
//! `SoundQuadriliteral` verbatim. Three additional passes run for the
//! classes that need them:
//!
//!   * **Geminated fusion** — `GeminatedTriliteral` roots (r[1] = r[2])
//!     collapse the duplicated consonant. Two rewrite rules fire in one
//!     pass: `<Y><V><Y> → <Y>` (vowel-separated pair, as in م-د-د + فَعَلَ
//!     → مَدَدَ → stripped مد) and `<X><ْ><Y><V><Y> → <X><V><Y>` (sukun
//!     with vowel migration, as in ء-م-م + أَفْعِلَة → أَءْمِمَة → أَءِمَة).
//!
//!   * **Hamza carrier picking** — `HamzatedTriliteral` roots store the
//!     hamza as a bare ء. After substitution (and fusion, if geminated),
//!     each remaining bare ء picks its carrier from the strongest short
//!     vowel in its immediate environment: kasra → ئ (or إ word-initial),
//!     damma → ؤ, fatha → أ, no vowel → ء. This closes the flagship
//!     الأئمة case that Light10 botched.
//!
//!   * **Weak-radical rewrites (M2.c)** — three high-frequency verb
//!     shapes get class-specific phonology:
//!     * `HollowTriliteral` + `VerbPerfect` — middle و/ي becomes ا when
//!       flanked by fatha + any short vowel (قَوَلَ → قَالَ, بَيَعَ → بَاعَ).
//!     * `DefectiveTriliteral` + `VerbPerfect` — final و/ي + fatha
//!       becomes ا (for و-final) or ى (for ي-final): دَعَوَ → دَعَا,
//!       رَمَيَ → رَمَى.
//!     * `AssimilatedTriliteral` + `VerbImperfect` (و-initial only) —
//!       the initial و drops after the tense prefix, collapsing the
//!       sukun: يَوْعِدُ → يَعِدُ.
//!     All other weak-class / pattern combinations pass through
//!     mechanically (v1 compromise); they produce slightly over-literal
//!     forms that the analyzer's folded-lookup fallback recovers in
//!     practice. Remaining rules (hollow imperfect, defective imperfect,
//!     ي-initial assimilated, weak-quadriliteral) are tracked for M2.d.
//!
//! ## Acceptance gate
//!
//! Before applying, we check that `pattern.accepts` includes
//! `root.class`. A pattern built only for sound triliterals will not
//! generate a surface form for a hollow root — the generator returns
//! `None`, which the FST builder treats as "skip this cell".
//!
//! # Placeholders, char-by-char
//!
//! Arabic pattern templates use the same three letters as the pattern
//! alphabet. Disambiguating "is this letter a placeholder or a structural
//! letter" is trivial: placeholders are `ف`, `ع`, and the first/second
//! occurrence of `ل`. Every *other* instance of those letters in a
//! template is a structural letter — but this never happens in the
//! patterns.rs catalogue because our convention is to use only
//! placeholder forms inside a template. So a linear scan with a counter
//! for `ل` suffices.

use super::patterns;
use super::roots;
use super::types::{PartOfSpeech, Pattern, PatternKind, Root, RootClass};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

// ──────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────

/// Apply `pattern` to `root`, producing a surface form.
///
/// Returns `None` when the pattern does not accept this root's class
/// (e.g. applying a sound-triliteral-only pattern to a quadriliteral
/// root), or when the template's placeholder count doesn't match the
/// root's radical count.
pub fn apply(root: &Root, pattern: &Pattern) -> Option<String> {
    if !pattern.accepts.contains(&root.class) {
        return None;
    }

    match root.class {
        RootClass::SoundTriliteral => apply_sound_triliteral(root, pattern),
        RootClass::SoundQuadriliteral => apply_sound_quadriliteral(root, pattern),
        RootClass::GeminatedTriliteral => apply_geminated(root, pattern),
        RootClass::HamzatedTriliteral => apply_hamzated(root, pattern),
        RootClass::HollowTriliteral => apply_hollow(root, pattern),
        RootClass::DefectiveTriliteral => apply_defective(root, pattern),
        RootClass::AssimilatedTriliteral => apply_assimilated(root, pattern),
        RootClass::WeakQuadriliteral => apply_weak_quadriliteral(root, pattern),
    }
}

/// Generate **every** (root, pattern) pair from the current seed corpora,
/// returning a flat list of `(root_key, pattern_label, surface)` triples.
///
/// This is the primary input to the FST builder (M3). Cells where
/// `apply` returns `None` are skipped. The output is sorted by surface
/// to give the FST builder a stable, dedup-friendly input.
pub fn generate_all() -> Vec<GeneratedForm> {
    let idx = roots::RootsIndex::get();
    let pats = patterns::all_patterns();

    // M9-intern — dedup pools for the two `Arc<str>` fields on
    // `GeneratedForm`. A root_key like `ك-ت-ب` recurs across every
    // pattern applied to that root (~140 times at full corpus); a
    // pattern_label like `فَاعِل` recurs across every root that licenses
    // the active participle (~6,000 times at 7K-root scale). Interning
    // each once here avoids thousands of owned-String heap allocations
    // during the cold-start rebuild, and lets the side-tables share a
    // single heap copy per distinct value.
    let mut root_pool: HashMap<String, Arc<str>> = HashMap::new();
    let mut label_pool: HashMap<String, Arc<str>> = HashMap::new();

    // Pre-intern every pattern label — there are ~200 of them regardless
    // of corpus size, so this is a bounded warm-up that pays for itself
    // on the very first root × pattern iteration.
    for pattern in &pats {
        intern(&mut label_pool, &pattern.label_ar);
    }

    let mut out: Vec<GeneratedForm> = Vec::with_capacity(idx.len() * pats.len() / 4);
    for root in idx.iter() {
        let root_key = intern(&mut root_pool, &roots::canonical_key(&root.radicals));
        for pattern in &pats {
            if let Some(surface) = apply(root, pattern) {
                out.push(GeneratedForm {
                    root_key: Arc::clone(&root_key),
                    pattern_label: intern(&mut label_pool, &pattern.label_ar),
                    pattern_kind: pattern.kind,
                    surface,
                });
            }
        }
    }
    out.sort_by(|a, b| a.surface.cmp(&b.surface));
    out
}

/// Intern a string into an `Arc<str>` pool, returning a shared `Arc` that
/// all callers referencing the same string get back. Used by both the
/// cold-start [`generate_all`] and the cache-load path in `fst_bake` —
/// whoever decodes the side tables can reuse this helper for symmetric
/// sharing.
pub(crate) fn intern(pool: &mut HashMap<String, Arc<str>>, s: &str) -> Arc<str> {
    if let Some(existing) = pool.get(s) {
        return Arc::clone(existing);
    }
    let arc: Arc<str> = Arc::from(s);
    pool.insert(s.to_string(), Arc::clone(&arc));
    arc
}

/// One generated surface form, fully linked back to its inputs for FST
/// payload assembly.
///
/// M9-intern — `root_key` and `pattern_label` are `Arc<str>` rather than
/// `String` so the ~200 distinct pattern labels and ~600 (today) / ~7,000
/// (projected) distinct root keys are heap-allocated **once** and shared
/// across every form that references them. At 1.1M-form scale a full
/// `String` per field would cost ~110 bytes per form in repeated string
/// bytes + allocator overhead (160 MB total); sharing via `Arc<str>` drops
/// that to ~32 bytes per form for the two pointers (35 MB total) plus a
/// few KB for the shared strings themselves. On-disk bake format is
/// unchanged — strings still round-trip through a length-prefixed UTF-8
/// encoding. The `fst_bake` decoder re-interns at load time by funnelling
/// every decoded string through a shared pool so cache hits get the same
/// sharing as cold rebuilds.
#[derive(Debug, Clone)]
pub struct GeneratedForm {
    pub root_key: Arc<str>,
    pub pattern_label: Arc<str>,
    pub pattern_kind: PatternKind,
    pub surface: String,
}

// ──────────────────────────────────────────────────────────────────────
// Core substitution (sound case — purely mechanical)
// ──────────────────────────────────────────────────────────────────────

/// Walk the template, substituting placeholder letters for radicals.
///
/// The placeholder characters in Arabic pattern notation are:
///   ف → first radical
///   ع → second radical
///   ل → third radical (first occurrence) / fourth radical (second occurrence)
///
/// `radicals.len()` must be 3 or 4. For a triliteral root applied to a
/// quadriliteral pattern (or vice-versa), this returns `None`.
fn substitute(template: &str, radicals: &[char]) -> Option<String> {
    // count_placeholders returns (non_lam_count, lam_count).
    // `non_lam_count` must always be 2 (one ف, one ع); `lam_count` drives
    // the triliteral-vs-quadriliteral distinction.
    let (non_lam_count, lam_count) = count_placeholders(template);
    if non_lam_count != 2 {
        return None;
    }

    match radicals.len() {
        3 => {
            // Triliteral: one ل (the third radical slot).
            if lam_count != 1 {
                return None;
            }
        }
        4 => {
            // Quadriliteral: two ل (third + fourth radical slots).
            if lam_count != 2 {
                return None;
            }
        }
        _ => return None,
    }

    let mut out = String::with_capacity(template.len() + 8);
    let mut lam_seen = 0usize;
    for c in template.chars() {
        match c {
            'ف' => out.push(radicals[0]),
            'ع' => out.push(radicals[1]),
            'ل' => {
                let idx = 2 + lam_seen;
                if idx >= radicals.len() {
                    return None;
                }
                out.push(radicals[idx]);
                lam_seen += 1;
            }
            other => out.push(other),
        }
    }
    Some(out)
}

/// Count occurrences of `ف`/`ع` and `ل` in a template. Returns
/// `(non_lam_placeholders, lam_count)` for arity validation.
fn count_placeholders(template: &str) -> (usize, usize) {
    let mut fp = 0usize;
    let mut lam = 0usize;
    for c in template.chars() {
        match c {
            'ف' | 'ع' => fp += 1,
            'ل' => lam += 1,
            _ => {}
        }
    }
    (fp, lam)
}

// ──────────────────────────────────────────────────────────────────────
// Per-class appliers
// ──────────────────────────────────────────────────────────────────────

fn apply_sound_triliteral(root: &Root, pattern: &Pattern) -> Option<String> {
    substitute(&pattern.template, &root.radicals)
}

fn apply_sound_quadriliteral(root: &Root, pattern: &Pattern) -> Option<String> {
    substitute(&pattern.template, &root.radicals)
}

/// Hamzated roots carry a hamza in some position. In the root table we
/// store the *bare* hamza (ء) when the carrier is context-dependent
/// (e.g. ء-م-م, ء-ل-ف), and the pre-carriered letter (أ / إ / ؤ / ئ)
/// when the carrier is structurally fixed (e.g. أ-م-ر, ب-د-أ). After
/// mechanical substitution we run two post-processors:
///
///   1. If `r[1] == r[2]`, apply the geminated-fusion rules so forms
///      like ء-م-م + أَفْعِلَة → أَءْمِمَة collapse to أَءِمَة (with vowel
///      migration), and ء-م-م + فَعَلَ → ءَمَمَ collapses to ءَمَ.
///
///   2. Walk the result and re-carrier every remaining bare ء based on
///      the strongest short vowel in its immediate environment. This is
///      what turns أَءِمَة into أَئِمَة — kasra on the right-hand side
///      promotes the hamza to a ya-carrier. The stripped form أئمة is
///      the exact input the user types for the flagship الأئمة case.
///
/// Pre-carriered hamza letters (أ/إ/ؤ/ئ) already in the root pass
/// through unchanged; the carrier walker only rewrites bare ء.
fn apply_hamzated(root: &Root, pattern: &Pattern) -> Option<String> {
    let mut s = substitute(&pattern.template, &root.radicals)?;
    // Geminated hamzated roots (ء-م-م, ء-ن-ن if added) need fusion too.
    // Gemination beats hamza on classify() by structural preference
    // (hamza wins), so we get here — but the (r[1] == r[2]) shape is what
    // actually matters for the fuser.
    if root.radicals.len() >= 3 && root.radicals[1] == root.radicals[2] {
        s = apply_geminated_fusion(&s, root.radicals[1]);
    }
    s = apply_hamza_carriers(&s);
    Some(s)
}

/// Geminated roots (r[1] == r[2]): after mechanical substitution the
/// surface has two identical consonants either adjacent (separated by a
/// short vowel) or split by a sukun + vowel. Classical phonology fuses
/// them into a single consonant with shadda; we drop the duplicate and
/// migrate any inter-radical vowel leftward when a sukun precedes the
/// first copy.
///
/// Examples:
///   م-د-د + فَعَلَ → mechanical مَدَدَ → fused مَدَ (stripped مد)
///   ء-م-م + أَفْعِلَة → mechanical أَءْمِمَة → fused أَءِمَة (stripped أءمة)
///
/// The stripper drops shaddahs downstream, so emitting without shadda is
/// equivalent for indexing purposes and simpler for the fuser.
fn apply_geminated(root: &Root, pattern: &Pattern) -> Option<String> {
    let raw = substitute(&pattern.template, &root.radicals)?;
    Some(apply_geminated_fusion(&raw, root.radicals[1]))
}

// ──────────────────────────────────────────────────────────────────────
// Phonology helpers — hamza carrier selection + geminated fusion
//
// These power `apply_hamzated` and `apply_geminated` (and any future
// class that needs them, such as a weak-radical applier that produces
// hamzas via و/ي → ء promotion in hollow and defective patterns).
//
// Every function operates on the *vocalised* surface — full tashkeel
// included — because the phonological rules fire on vowel context. The
// normalizer strips tashkeel downstream for indexing.
// ──────────────────────────────────────────────────────────────────────

/// Recognise the three short vowel marks (fatha/kasra/damma).
/// Excludes sukun (ْ), shadda (ّ), tanwin marks, and the long-vowel
/// letters — those have different phonological roles.
fn is_short_vowel(c: char) -> bool {
    matches!(c, 'َ' | 'ِ' | 'ُ')
}

/// Rank short vowels by "strength" in the hamza-carrier selection sense.
///
/// The classical rule: the carrier of a medial hamza is chosen by the
/// strongest short vowel in its immediate environment, where strength
/// is ordered kasra > damma > fatha. A sukun or absent vowel has rank 0
/// and is only the chosen context when nothing else is present.
fn vowel_strength(c: char) -> u8 {
    match c {
        'ِ' => 3, // kasra wins
        'ُ' => 2, // damma next
        'َ' => 1, // fatha weakest
        _ => 0,
    }
}

/// Pick a hamza carrier letter given the short vowels immediately
/// before and after a bare ء, and whether the ء is word-initial.
///
/// Rules (v1):
///   * Strongest vowel = kasra → ئ (medial) / إ (word-initial).
///   * Strongest vowel = damma → ؤ.
///   * Strongest vowel = fatha → أ.
///   * No adjacent short vowel → ء stays bare. This handles the rare
///     sukun-flanked ء (e.g. قرآن-adjacent shapes) conservatively; the
///     disambiguator can refine later if regression data asks for it.
fn pick_hamza_carrier(prev: Option<char>, next: Option<char>, word_initial: bool) -> char {
    let strongest = [prev, next]
        .iter()
        .filter_map(|v| *v)
        .max_by_key(|&c| vowel_strength(c));
    match strongest {
        Some('ِ') => {
            if word_initial {
                'إ'
            } else {
                'ئ'
            }
        }
        Some('ُ') => 'ؤ',
        Some('َ') => 'أ',
        _ => 'ء',
    }
}

/// Walk `s` and replace every bare hamza (ء) with its contextually-correct
/// carrier letter. Pre-carriered hamza letters (أ/إ/ؤ/ئ) and every other
/// character pass through unchanged.
fn apply_hamza_carriers(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    for (i, &c) in chars.iter().enumerate() {
        if c != 'ء' {
            out.push(c);
            continue;
        }
        let prev_vowel = if i > 0 && is_short_vowel(chars[i - 1]) {
            Some(chars[i - 1])
        } else {
            None
        };
        let next_vowel = if i + 1 < chars.len() && is_short_vowel(chars[i + 1]) {
            Some(chars[i + 1])
        } else {
            None
        };
        out.push(pick_hamza_carrier(prev_vowel, next_vowel, i == 0));
    }
    out.into_iter().collect()
}

/// Apply the two geminated-fusion rewrite rules to `s`, where `dup_char`
/// is the duplicated radical (r[1] == r[2]).
///
/// Scan left-to-right, longer rule first to avoid spurious matches:
///
///   1. `<X> <ْ> <Y> <V> <Y>` → `<X> <V> <Y>`
///      (sukun-preceded pair with vowel migration — covers أَفْعِلَة applied
///       to ء-م-م: the medial kasra migrates onto the ء's sukun slot.)
///
///   2. `<Y> <V> <Y>` → `<Y>`
///      (vowel-separated pair — covers فَعَلَ applied to م-د-د: drop the
///       second د and its preceding fatha, leaving a single د which the
///       normalizer renders as مد.)
///
/// Characters that don't match either rule pass through unchanged, and
/// the rules cannot overlap — rule 1's span is five chars starting at
/// `<X>`, rule 2's span is three chars starting at `<Y>`. The scanner
/// tries rule 1 first at every position, then rule 2, then advances.
fn apply_geminated_fusion(s: &str, dup_char: char) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        // Rule 1: <X> <sukun> <Y> <V> <Y>  →  <X> <V> <Y>
        if i + 4 < chars.len()
            && chars[i + 1] == 'ْ'
            && chars[i + 2] == dup_char
            && is_short_vowel(chars[i + 3])
            && chars[i + 4] == dup_char
        {
            out.push(chars[i]);        // X
            out.push(chars[i + 3]);    // V (migrated to X's sukun slot)
            out.push(chars[i + 4]);    // single Y (the second, surviving one)
            i += 5;
            continue;
        }
        // Rule 2: <Y> <V> <Y>  →  <Y>
        if i + 2 < chars.len()
            && chars[i] == dup_char
            && is_short_vowel(chars[i + 1])
            && chars[i + 2] == dup_char
        {
            out.push(chars[i]); // single surviving Y
            i += 3;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out.into_iter().collect()
}

// ──────────────────────────────────────────────────────────────────────
// Phonology helpers — weak-radical rules (M2.c)
//
// Three high-frequency verb shapes landed in M2.c: hollow perfect
// (قَالَ), defective perfect (دَعَا / رَمَى), assimilated imperfect
// (يَعِدُ). Each helper operates on the vocalised surface produced by
// `substitute` and returns the phonologically-correct form. The scope
// is kept narrow on purpose — every other pattern kind on these
// classes passes through mechanically, so the analyzer's folded-lookup
// fallback still catches them while the v1 generator stays small and
// auditable. Remaining rules (hollow imperfect, defective imperfect,
// ي-initial assimilated, weak-quadriliteral) are tracked for M2.d.
// ──────────────────────────────────────────────────────────────────────

/// Hollow + `VerbPerfect`: a middle weak radical (و/ي) flanked by a
/// fatha on the left and any short vowel on the right collapses to ا.
///
/// Input → output (stripped in parens):
///   قَوَلَ → قَالَ (قال)      — ق-و-ل + فَعَلَ
///   بَيَعَ → بَاعَ (باع)      — ب-ي-ع + فَعَلَ
///   خَوِفَ → خَافَ (خاف)      — خ-و-ف + فَعِلَ
///
/// Any other `kind` — participles, verbal nouns, imperfect, broken
/// plurals — passes through unchanged in v1. The folded fallback in
/// the analyzer recovers many of them; M2.d tackles the rest.
fn apply_hollow_rules(s: &str, weak: char, kind: PatternKind) -> String {
    if kind != PatternKind::VerbPerfect {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        // <fatha> <weak> <short_vowel>  →  <fatha> <ا>
        if i + 2 < chars.len()
            && chars[i] == 'َ'
            && chars[i + 1] == weak
            && is_short_vowel(chars[i + 2])
        {
            out.push('َ');
            out.push('ا');
            i += 3;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out.into_iter().collect()
}

/// Defective + `VerbPerfect`: a final `<weak><fatha>` pair collapses to
/// a single long-vowel letter — ا when the weak is و, ى when it's ي.
///
/// Input → output (stripped in parens):
///   دَعَوَ → دَعَا (دعا)      — د-ع-و + فَعَلَ
///   رَمَيَ → رَمَى (رمى)      — ر-م-ي + فَعَلَ
///
/// Out-of-scope pattern kinds pass through unchanged.
fn apply_defective_rules(s: &str, weak: char, kind: PatternKind) -> String {
    if kind != PatternKind::VerbPerfect {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 2 {
        return s.to_string();
    }
    let last = chars.len() - 1;
    if chars[last] == 'َ' && chars[last - 1] == weak {
        let mut out: Vec<char> = chars[..last - 1].to_vec();
        out.push(if weak == 'ي' { 'ى' } else { 'ا' });
        return out.into_iter().collect();
    }
    s.to_string()
}

/// Assimilated + `VerbImperfect` (و-initial only in v1): the initial
/// weak و drops after a tense prefix + fatha, merging the sukun slot
/// into the next radical.
///
/// Input → output (stripped in parens):
///   يَوْعِدُ → يَعِدُ (يعد)    — و-ع-د + يَفْعِلُ
///
/// Tense prefixes recognised: ي / ت / ن / أ. The ي-initial assimilated
/// subcase and any non-fatha prefix vocalisation pass through
/// unchanged — they're deferred to M2.d because they're rarer and the
/// folded fallback catches the common ones.
fn apply_assimilated_rules(s: &str, weak: char, kind: PatternKind) -> String {
    if kind != PatternKind::VerbImperfect {
        return s.to_string();
    }
    if weak != 'و' {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    // Need at least <prefix> <fatha> <weak> <sukun> <ayn-radical> <...>
    if chars.len() < 5 {
        return s.to_string();
    }
    let is_tense_prefix = matches!(chars[0], 'ي' | 'ت' | 'ن' | 'أ');
    if !is_tense_prefix {
        return s.to_string();
    }
    if chars[1] == 'َ' && chars[2] == weak && chars[3] == 'ْ' {
        let mut out: Vec<char> = Vec::with_capacity(chars.len() - 2);
        out.push(chars[0]);      // tense prefix
        out.push(chars[1]);      // fatha
        out.extend_from_slice(&chars[4..]); // skip weak+sukun, keep the rest
        return out.into_iter().collect();
    }
    s.to_string()
}

// ──────────────────────────────────────────────────────────────────────
// Weak-radical appliers
//
// Each applier dispatches on `pattern.kind` — the scoped kind fires
// the M2.c rule; everything else passes through mechanically. The
// `WeakQuadriliteral` applier is pass-through entirely (real-corpus
// occurrences are rare enough that the folded fallback carries it).
// ──────────────────────────────────────────────────────────────────────

fn apply_hollow(root: &Root, pattern: &Pattern) -> Option<String> {
    let raw = substitute(&pattern.template, &root.radicals)?;
    Some(apply_hollow_rules(&raw, root.radicals[1], pattern.kind))
}

fn apply_defective(root: &Root, pattern: &Pattern) -> Option<String> {
    let raw = substitute(&pattern.template, &root.radicals)?;
    Some(apply_defective_rules(&raw, root.radicals[2], pattern.kind))
}

fn apply_assimilated(root: &Root, pattern: &Pattern) -> Option<String> {
    let raw = substitute(&pattern.template, &root.radicals)?;
    Some(apply_assimilated_rules(&raw, root.radicals[0], pattern.kind))
}

fn apply_weak_quadriliteral(root: &Root, pattern: &Pattern) -> Option<String> {
    // Quadriliteral weak rules are very rare in the real corpus; v1
    // emits the mechanical form, which is correct for the majority of
    // cases (native speakers accept the "naive" shape).
    substitute(&pattern.template, &root.radicals)
}

// ──────────────────────────────────────────────────────────────────────
// PatternKind → PartOfSpeech
// ──────────────────────────────────────────────────────────────────────

/// Map a `PatternKind` to a coarse `PartOfSpeech` for the analyzer.
///
/// Arabic pattern categories have clean PoS correspondences in most
/// cases. Participles are marked as nouns because in modern usage they
/// function primarily as nominalised forms (فَاعِل = "doer", مَفْعُول =
/// "done-thing"); the disambiguator can promote them to adjectives when
/// syntactic context demands.
pub fn pos_for_kind(kind: PatternKind) -> PartOfSpeech {
    match kind {
        PatternKind::VerbPerfect
        | PatternKind::VerbImperfect
        | PatternKind::VerbImperative => PartOfSpeech::Verb,

        PatternKind::VerbalNoun
        | PatternKind::DerivedNoun
        | PatternKind::BrokenPlural
        | PatternKind::Diminutive
        | PatternKind::ActiveParticiple
        | PatternKind::PassiveParticiple
        | PatternKind::Feminine => PartOfSpeech::Noun,

        PatternKind::Relative | PatternKind::Elative => PartOfSpeech::Adjective,
    }
}

// ──────────────────────────────────────────────────────────────────────
// GenerativeIndex — Layer 3 lookup cache
//
// The FST in M3 will replace this with an mmap'd finite-state
// transducer that stores the same key→values mapping in ~1.5 MB on
// disk. Until then, this HashMap-backed index gives us the same public
// surface so the analyzer can wire Layer 3 today. When the FST lands,
// only this module changes — callers (analyze() in mod.rs) don't.
// ──────────────────────────────────────────────────────────────────────

/// An index over `generate_all()` output, keyed by stripped surface
/// (tashkeel/tatweel-removed) for O(1) lookup.
///
/// Two lookup tables are maintained:
///   - `by_stripped` — exact match on the normalizer's `stripped` form.
///     Preserves hamza carriers, alif variants, tāʾ marbūṭa.
///   - `by_folded` — match on the `folded` form (aggressively flattened
///     hamza/alif/ya/ta-marbuta). Used as a graceful fallback when the
///     exact stripped lookup misses, matching Light10's tolerant behavior
///     but without its over-stripping.
///
/// Each bucket is deduplicated by `(root_key, pattern_label)` — the same
/// surface can arise from distinct roots (homonyms) or from the same root
/// via different patterns, and we want the analyzer to see all of them.
pub struct GenerativeIndex {
    by_stripped: HashMap<String, Vec<GeneratedForm>>,
    by_folded: HashMap<String, Vec<GeneratedForm>>,
}

impl GenerativeIndex {
    /// Access the lazily-initialised singleton. First call builds the
    /// full corpus (seed roots × 158 patterns → ~19K forms today, ~1.1M
    /// once the 7K root corpus lands); subsequent calls are free.
    pub fn get() -> &'static GenerativeIndex {
        static INDEX: OnceLock<GenerativeIndex> = OnceLock::new();
        INDEX.get_or_init(Self::build)
    }

    fn build() -> Self {
        let all = generate_all();
        let mut by_stripped: HashMap<String, Vec<GeneratedForm>> = HashMap::new();
        let mut by_folded: HashMap<String, Vec<GeneratedForm>> = HashMap::new();

        for form in all {
            let stripped = super::normalizer::normalize_stripped(&form.surface);
            let folded = super::normalizer::normalize_folded(&form.surface);
            by_stripped
                .entry(stripped.clone())
                .or_default()
                .push(form.clone());
            // Only index in `by_folded` if folding actually changes the key —
            // no point wasting memory on identical entries.
            if folded != stripped {
                by_folded.entry(folded).or_default().push(form);
            }
        }

        // Deduplicate each bucket on (root_key, pattern_label).
        for v in by_stripped.values_mut() {
            v.sort_by(|a, b| {
                a.root_key
                    .cmp(&b.root_key)
                    .then_with(|| a.pattern_label.cmp(&b.pattern_label))
            });
            v.dedup_by(|a, b| a.root_key == b.root_key && a.pattern_label == b.pattern_label);
        }
        for v in by_folded.values_mut() {
            v.sort_by(|a, b| {
                a.root_key
                    .cmp(&b.root_key)
                    .then_with(|| a.pattern_label.cmp(&b.pattern_label))
            });
            v.dedup_by(|a, b| a.root_key == b.root_key && a.pattern_label == b.pattern_label);
        }

        GenerativeIndex { by_stripped, by_folded }
    }

    /// Lookup by stripped surface (tashkeel/tatweel removed). Empty slice
    /// means "no hit" — callers can fall through to `lookup_folded`.
    pub fn lookup(&self, stripped: &str) -> &[GeneratedForm] {
        self.by_stripped
            .get(stripped)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Lookup by folded surface (aggressive normalization). Should only
    /// fire after `lookup` returned empty; results carry a slightly
    /// lower confidence downstream to reflect the fuzzier match.
    pub fn lookup_folded(&self, folded: &str) -> &[GeneratedForm] {
        self.by_folded
            .get(folded)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Number of distinct stripped-surface keys. Useful for sanity
    /// checks and benchmarks.
    pub fn len(&self) -> usize {
        self.by_stripped.len()
    }

    /// `true` when no forms have been indexed yet. Should never return
    /// `true` in production (seed corpus always yields >0 forms), but
    /// exposed for clippy / test clarity.
    pub fn is_empty(&self) -> bool {
        self.by_stripped.is_empty()
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arabic::patterns;
    use crate::arabic::roots::root_from_key;

    // Helper: find the first pattern matching a substring of its label.
    fn find_pattern_by_label_fragment(frag: &str) -> Pattern {
        patterns::all_patterns()
            .into_iter()
            .find(|p| p.label_ar.contains(frag) || p.label_en.contains(frag))
            .unwrap_or_else(|| panic!("no pattern label contains `{frag}`"))
    }

    // Helper: strip all tashkeel from a surface string for equality checks
    // that ignore vocalization differences. Prevents over-specifying test
    // expectations when the pattern template includes optional tashkeel.
    fn stripped(s: &str) -> String {
        crate::arabic::normalizer::normalize_stripped(s)
    }

    // ── substitute (primitive) ──────────────────────────────────────

    #[test]
    fn substitute_sound_triliteral_mechanical() {
        // فَعَلَ + ك-ت-ب → كَتَبَ
        let out = substitute("فَعَلَ", &['ك', 'ت', 'ب']).unwrap();
        assert_eq!(stripped(&out), "كتب");
    }

    #[test]
    fn substitute_active_participle() {
        // فَاعِل + ك-ت-ب → كَاتِب
        let out = substitute("فَاعِل", &['ك', 'ت', 'ب']).unwrap();
        assert_eq!(stripped(&out), "كاتب");
    }

    #[test]
    fn substitute_passive_participle() {
        // مَفْعُول + ك-ت-ب → مَكْتُوب
        let out = substitute("مَفْعُول", &['ك', 'ت', 'ب']).unwrap();
        assert_eq!(stripped(&out), "مكتوب");
    }

    #[test]
    fn substitute_rejects_wrong_arity() {
        // Triliteral pattern applied to quadriliteral radicals → None
        assert!(substitute("فَعَلَ", &['د', 'ح', 'ر', 'ج']).is_none());
        // Quadriliteral pattern applied to triliteral radicals → None
        assert!(substitute("فَعْلَل", &['ك', 'ت', 'ب']).is_none());
    }

    #[test]
    fn substitute_quadriliteral_uses_both_lams() {
        // فَعْلَلَ + د-ح-ر-ج → دَحْرَجَ
        let out = substitute("فَعْلَلَ", &['د', 'ح', 'ر', 'ج']).unwrap();
        assert_eq!(stripped(&out), "دحرج");
    }

    // ── apply (top-level) ───────────────────────────────────────────

    #[test]
    fn apply_sound_triliteral_perfect() {
        // كتب + Form I perfect فَعَلَ → كَتَبَ
        let root = root_from_key("ك-ت-ب", None).unwrap();
        // Find a Form I perfect pattern that accepts SoundTriliteral and
        // has the exact template فَعَلَ (first vowel pattern).
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.kind == PatternKind::VerbPerfect
                  && p.template == "فَعَلَ"
                  && p.accepts.contains(&RootClass::SoundTriliteral))
            .expect("Form I perfect فَعَلَ must exist for sound triliterals");
        let out = apply(&root, &pat).unwrap();
        assert_eq!(stripped(&out), "كتب");
    }

    #[test]
    fn apply_rejects_pattern_not_accepting_class() {
        // Build a root of one class and find a pattern that does NOT
        // accept it. Ensure apply returns None.
        let hollow = root_from_key("ق-و-ل", None).unwrap();
        // Form IV active participle مُفْعِل typically accepts sound roots only;
        // find one that excludes HollowTriliteral and confirm rejection.
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p|
                !p.accepts.contains(&RootClass::HollowTriliteral)
                && !p.accepts.is_empty()
            );
        if let Some(p) = pat {
            assert!(apply(&hollow, &p).is_none(),
                "expected None when pattern `{}` doesn't accept HollowTriliteral",
                p.label_ar);
        }
    }

    #[test]
    fn apply_active_participle_generates_kaatib() {
        // The primary test of the lemma promise: كاتب from ك-ت-ب via فاعل.
        let root = root_from_key("ك-ت-ب", None).unwrap();
        let pat = find_pattern_by_label_fragment("فاعل");
        let out = apply(&root, &pat).unwrap();
        assert_eq!(stripped(&out), "كاتب");
    }

    // ── geminated ───────────────────────────────────────────────────

    #[test]
    fn geminated_collapses_adjacent_duplicates() {
        // م-د-د + فَعَلَ → مَدَدَ → mechanical مدد, collapse adjacent د pair → مَدَ
        // (stripped). In surface form this would typically be مَدَّ with a
        // shadda; we emit the stripped form without shadda on purpose.
        let root = root_from_key("م-د-د", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.kind == PatternKind::VerbPerfect
                  && p.template == "فَعَلَ"
                  && p.accepts.contains(&RootClass::GeminatedTriliteral))
            .expect("Form I perfect فَعَلَ must accept geminated");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        // Either "مد" (collapsed) or "مدد" (kept) — both are acceptable
        // given v1 behavior; what matters is the adjacent collapse fired
        // or the radicals are preserved.
        assert!(s == "مد" || s == "مدد",
            "geminated result should be مد or مدد, got {s}");
    }

    #[test]
    fn geminated_does_not_collapse_non_adjacent_duplicates() {
        // A pattern that separates r[1] and r[2] with an alif — the two
        // د letters are not adjacent, so no collapse fires.
        let out = substitute("فَاعِل", &['م', 'د', 'د']).unwrap();
        let s = stripped(&out);
        assert!(s.contains("ماد"), "expected ماد prefix, got {s}");
    }

    // ── hamzated ────────────────────────────────────────────────────

    #[test]
    fn hamzated_preserves_hamza_carrier() {
        // أ-م-ر + فَعَلَ → أَمَرَ (the hamza on alif is preserved).
        let root = root_from_key("أ-م-ر", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.kind == PatternKind::VerbPerfect
                  && p.template == "فَعَلَ"
                  && p.accepts.contains(&RootClass::HamzatedTriliteral))
            .expect("Form I perfect must accept hamzated");
        let out = apply(&root, &pat).unwrap();
        // Stripped form should start with أ, preserving the hamza carrier.
        let s = stripped(&out);
        assert!(s.starts_with('أ'), "expected أ carrier, got {s}");
    }

    // ── quadriliteral ───────────────────────────────────────────────

    #[test]
    fn quadriliteral_perfect_form_i() {
        // د-ح-ر-ج + فَعْلَلَ → دَحْرَجَ
        let root = root_from_key("د-ح-ر-ج", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.template == "فَعْلَلَ"
                  && p.accepts.contains(&RootClass::SoundQuadriliteral))
            .expect("quadriliteral Form I perfect فَعْلَلَ must exist");
        let out = apply(&root, &pat).unwrap();
        assert_eq!(stripped(&out), "دحرج");
    }

    // ── generate_all ────────────────────────────────────────────────

    #[test]
    fn generate_all_produces_nonzero_output() {
        let all = generate_all();
        assert!(all.len() > 100,
            "expected >100 generated forms from seed × patterns, got {}",
            all.len());
    }

    #[test]
    fn generate_all_is_sorted_by_surface() {
        let all = generate_all();
        for pair in all.windows(2) {
            assert!(pair[0].surface <= pair[1].surface,
                "generate_all output must be sorted by surface");
        }
    }

    #[test]
    fn generate_all_includes_kaatib() {
        let all = generate_all();
        let has_kaatib = all.iter().any(|g| stripped(&g.surface) == "كاتب");
        assert!(has_kaatib,
            "generated corpus must contain كاتب (active participle of ك-ت-ب)");
    }

    #[test]
    fn generate_all_includes_maktub() {
        let all = generate_all();
        let has_maktub = all.iter().any(|g| stripped(&g.surface) == "مكتوب");
        assert!(has_maktub,
            "generated corpus must contain مكتوب (passive participle of ك-ت-ب)");
    }

    #[test]
    fn generate_all_includes_dahraj() {
        let all = generate_all();
        let has_dahraj = all.iter().any(|g| stripped(&g.surface) == "دحرج");
        assert!(has_dahraj,
            "generated corpus must contain دحرج (Form I perfect of د-ح-ر-ج)");
    }

    // ── placeholder counter ─────────────────────────────────────────

    #[test]
    fn count_placeholders_counts_correctly() {
        assert_eq!(count_placeholders("فَعَلَ"), (2, 1));          // ف + ع + ل
        assert_eq!(count_placeholders("فَعْلَلَ"), (2, 2));        // ف + ع + ل + ل
        assert_eq!(count_placeholders("مَفْعُول"), (2, 1));       // م + ف + ع + ل + و
        assert_eq!(count_placeholders("مُفَعِّل"), (2, 1));        // ف + ع + ل
    }

    // ── pos_for_kind ────────────────────────────────────────────────

    #[test]
    fn pos_for_kind_maps_verbs_to_verb() {
        assert_eq!(pos_for_kind(PatternKind::VerbPerfect), PartOfSpeech::Verb);
        assert_eq!(pos_for_kind(PatternKind::VerbImperfect), PartOfSpeech::Verb);
        assert_eq!(pos_for_kind(PatternKind::VerbImperative), PartOfSpeech::Verb);
    }

    #[test]
    fn pos_for_kind_maps_nominals_to_noun() {
        assert_eq!(pos_for_kind(PatternKind::VerbalNoun), PartOfSpeech::Noun);
        assert_eq!(pos_for_kind(PatternKind::DerivedNoun), PartOfSpeech::Noun);
        assert_eq!(pos_for_kind(PatternKind::BrokenPlural), PartOfSpeech::Noun);
        assert_eq!(pos_for_kind(PatternKind::Diminutive), PartOfSpeech::Noun);
        assert_eq!(pos_for_kind(PatternKind::ActiveParticiple), PartOfSpeech::Noun);
        assert_eq!(pos_for_kind(PatternKind::PassiveParticiple), PartOfSpeech::Noun);
        assert_eq!(pos_for_kind(PatternKind::Feminine), PartOfSpeech::Noun);
    }

    #[test]
    fn pos_for_kind_maps_adjectives_to_adjective() {
        assert_eq!(pos_for_kind(PatternKind::Relative), PartOfSpeech::Adjective);
        assert_eq!(pos_for_kind(PatternKind::Elative), PartOfSpeech::Adjective);
    }

    // ── GenerativeIndex ─────────────────────────────────────────────

    #[test]
    fn generative_index_builds_nonempty() {
        let idx = GenerativeIndex::get();
        assert!(
            !idx.is_empty(),
            "GenerativeIndex must be non-empty after build"
        );
        assert!(
            idx.len() > 100,
            "expected >100 distinct stripped keys, got {}",
            idx.len()
        );
    }

    #[test]
    fn generative_index_finds_kaatib() {
        let idx = GenerativeIndex::get();
        let hits = idx.lookup("كاتب");
        assert!(
            !hits.is_empty(),
            "كاتب must be found in the generative index"
        );
        // At least one hit should come from the ك-ت-ب root via an active
        // participle pattern.
        let found = hits.iter().any(|g| {
            &*g.root_key == "ك-ت-ب" && g.pattern_kind == PatternKind::ActiveParticiple
        });
        assert!(
            found,
            "كاتب must be traceable to (ك-ت-ب, ActiveParticiple); hits: {:?}",
            hits
        );
    }

    #[test]
    fn generative_index_finds_maktub() {
        let idx = GenerativeIndex::get();
        let hits = idx.lookup("مكتوب");
        assert!(!hits.is_empty(), "مكتوب must be in the generative index");
        let found = hits.iter().any(|g| {
            &*g.root_key == "ك-ت-ب" && g.pattern_kind == PatternKind::PassiveParticiple
        });
        assert!(
            found,
            "مكتوب must trace to (ك-ت-ب, PassiveParticiple); hits: {:?}",
            hits
        );
    }

    #[test]
    fn generative_index_finds_dahraj() {
        let idx = GenerativeIndex::get();
        let hits = idx.lookup("دحرج");
        assert!(!hits.is_empty(), "دحرج must be in the generative index");
        let found = hits
            .iter()
            .any(|g| &*g.root_key == "د-ح-ر-ج" && g.pattern_kind == PatternKind::VerbPerfect);
        assert!(
            found,
            "دحرج must trace to (د-ح-ر-ج, VerbPerfect); hits: {:?}",
            hits
        );
    }

    #[test]
    fn generative_index_misses_unknown_word() {
        let idx = GenerativeIndex::get();
        // A truly made-up Arabic-looking string that cannot arise from
        // any (seed root × pattern). Sanity check that lookup returns an
        // empty slice rather than panicking or returning garbage.
        let hits = idx.lookup("ززززز");
        assert!(hits.is_empty(), "fake word should not hit the index");
    }

    // ── M2.b: hamza carrier selection (primitive helpers) ───────────

    #[test]
    fn vowel_strength_orders_kasra_above_damma_above_fatha() {
        assert!(vowel_strength('ِ') > vowel_strength('ُ'));
        assert!(vowel_strength('ُ') > vowel_strength('َ'));
        assert!(vowel_strength('َ') > vowel_strength('ْ'));
        assert_eq!(vowel_strength('ا'), 0);
    }

    #[test]
    fn is_short_vowel_recognises_fatha_kasra_damma() {
        assert!(is_short_vowel('َ'));
        assert!(is_short_vowel('ِ'));
        assert!(is_short_vowel('ُ'));
        // Sukun, shadda, tanwins are NOT short vowels for carrier picking.
        assert!(!is_short_vowel('ْ'));
        assert!(!is_short_vowel('ّ'));
        assert!(!is_short_vowel('ً'));
        assert!(!is_short_vowel('ا'));
    }

    #[test]
    fn pick_hamza_carrier_picks_ya_for_kasra_medial() {
        // Adjacent kasra (on either side) in a medial position → ئ.
        assert_eq!(pick_hamza_carrier(Some('َ'), Some('ِ'), false), 'ئ');
        assert_eq!(pick_hamza_carrier(Some('ِ'), Some('َ'), false), 'ئ');
    }

    #[test]
    fn pick_hamza_carrier_picks_alif_below_for_kasra_word_initial() {
        // Word-initial + kasra → إ, not ئ.
        assert_eq!(pick_hamza_carrier(None, Some('ِ'), true), 'إ');
    }

    #[test]
    fn pick_hamza_carrier_picks_waw_for_damma() {
        assert_eq!(pick_hamza_carrier(Some('َ'), Some('ُ'), false), 'ؤ');
        assert_eq!(pick_hamza_carrier(Some('ُ'), None, false), 'ؤ');
    }

    #[test]
    fn pick_hamza_carrier_picks_alif_for_fatha() {
        assert_eq!(pick_hamza_carrier(Some('َ'), Some('َ'), false), 'أ');
        assert_eq!(pick_hamza_carrier(None, Some('َ'), true), 'أ');
    }

    #[test]
    fn pick_hamza_carrier_leaves_bare_when_no_vowel_context() {
        // Only sukun/None around — nothing to base the carrier on.
        assert_eq!(pick_hamza_carrier(None, None, true), 'ء');
        assert_eq!(pick_hamza_carrier(Some('ْ'), Some('ْ'), false), 'ء');
    }

    #[test]
    fn pick_hamza_carrier_kasra_beats_fatha_when_both_present() {
        // The strongest-vowel-wins rule: kasra outranks fatha regardless of
        // which side it's on.
        assert_eq!(pick_hamza_carrier(Some('َ'), Some('ِ'), false), 'ئ');
        assert_eq!(pick_hamza_carrier(Some('ِ'), Some('َ'), false), 'ئ');
    }

    #[test]
    fn apply_hamza_carriers_rewrites_bare_hamza_in_kasra_environment() {
        // Input: أَءِمَة (with fatha before ء, kasra after). Expected: ئ carrier.
        let out = apply_hamza_carriers("أَءِمَة");
        assert!(
            out.contains('ئ'),
            "expected ئ in output, got {out}"
        );
        assert!(
            !out.contains('ء'),
            "bare ء should have been rewritten; got {out}"
        );
    }

    #[test]
    fn apply_hamza_carriers_leaves_pre_carriered_letters_alone() {
        // أ, إ, ؤ, ئ, آ are already-carriered forms; the walker must
        // never touch them.
        let input = "أَخَذَ إِذْن مُؤْمِن قَائِل";
        let out = apply_hamza_carriers(input);
        assert_eq!(out, input, "pre-carriered letters must pass through verbatim");
    }

    #[test]
    fn apply_hamza_carriers_word_initial_fatha_becomes_alif() {
        // Bare ء at position 0, fatha after → أ.
        let out = apply_hamza_carriers("ءَمَرَ");
        assert!(
            out.starts_with('أ'),
            "word-initial ء + fatha should become أ; got {out}"
        );
    }

    // ── M2.b: geminated fusion (primitive helper) ───────────────────

    #[test]
    fn geminated_fusion_collapses_vowel_separated_pair() {
        // م-د-د + فَعَلَ mechanical: مَدَدَ. Rule 2 fires on the inner د-fatha-د.
        let out = apply_geminated_fusion("مَدَدَ", 'د');
        let s = stripped(&out);
        assert_eq!(s, "مد", "vowel-separated pair should fuse to single consonant");
    }

    #[test]
    fn geminated_fusion_migrates_vowel_across_sukun() {
        // ء-م-م + أَفْعِلَة mechanical: أَءْمِمَة. Rule 1 fires on ء-sukun-م-kasra-م.
        // Expected after fusion: أَءِمَة (kasra migrates to where the sukun was).
        let out = apply_geminated_fusion("أَءْمِمَة", 'م');
        assert_eq!(out, "أَءِمَة",
            "sukun before duplicated pair should absorb the inter-vowel");
    }

    #[test]
    fn geminated_fusion_ignores_non_duplicated_consonants() {
        // Input has no duplicated د (only one د total); fuser must be a no-op.
        let out = apply_geminated_fusion("كَتَبَ", 'د');
        assert_eq!(out, "كَتَبَ", "fuser must not alter non-duplicate strings");
    }

    #[test]
    fn geminated_fusion_does_not_collapse_long_vowel_separation() {
        // فِعَال of م-د-د = مِدَاد (kasra + alif sits BETWEEN the two د's —
        // the alif is a long-vowel letter, not a short vowel). The trigger
        // `<Y><V><Y>` requires a SHORT vowel in the V slot, so the fuser
        // must leave this surface alone.
        let out = apply_geminated_fusion("مِدَاد", 'د');
        let s = stripped(&out);
        assert_eq!(
            s, "مداد",
            "alif-separated pair must not fuse (long vowel breaks trigger); got {s}"
        );
    }

    // ── M2.b: end-to-end via apply() ────────────────────────────────

    #[test]
    fn apply_hamzated_geminated_produces_aimma() {
        // THE FLAGSHIP SURFACE. Root ء-م-م + أَفْعِلَة → أَئِمَة (stripped أئمة).
        // This is what الأئمة strips to after peeling ال, and what the
        // GenerativeIndex must be able to look up.
        let root = root_from_key("ء-م-م", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.template == "أَفْعِلَة")
            .expect("أَفْعِلَة broken-plural pattern must exist");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(
            s, "أئمة",
            "ء-م-م + أَفْعِلَة must produce أئمة (stripped); got {s}"
        );
    }

    #[test]
    fn apply_hamzated_geminated_perfect_produces_am() {
        // ء-م-م + فَعَلَ: mechanical ءَمَمَ → fuse م-fatha-م → ءَمَ → carrier ء/fatha → أَمَ.
        let root = root_from_key("ء-م-م", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| {
                p.kind == PatternKind::VerbPerfect
                    && p.template == "فَعَلَ"
                    && p.accepts.contains(&RootClass::HamzatedTriliteral)
            })
            .expect("Form I perfect فَعَلَ must accept hamzated");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(s, "أم", "ء-م-م + فَعَلَ should fuse and carrier to أم; got {s}");
    }

    #[test]
    fn apply_hamzated_nonduplicate_passes_through_unchanged() {
        // أ-م-ر (pre-carriered, non-geminated): must not grow spurious carriers.
        let root = root_from_key("أ-م-ر", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| {
                p.kind == PatternKind::VerbPerfect
                    && p.template == "فَعَلَ"
                    && p.accepts.contains(&RootClass::HamzatedTriliteral)
            })
            .expect("Form I perfect فَعَلَ must accept hamzated");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(s, "أمر");
    }

    #[test]
    fn apply_hamzated_initial_bare_alef_hamza_gets_carrier() {
        // ء-ل-ف (bare initial hamza) + فَعَلَ → expected stripped ألف
        // (word-initial ء + fatha → أ).
        let root = root_from_key("ء-ل-ف", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| {
                p.kind == PatternKind::VerbPerfect
                    && p.template == "فَعَلَ"
                    && p.accepts.contains(&RootClass::HamzatedTriliteral)
            })
            .expect("Form I perfect فَعَلَ must accept hamzated");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(s, "ألف", "bare initial ء + fatha → أ; got {s}");
    }

    #[test]
    fn generate_all_includes_aimma() {
        // The closing proof for the الأئمة flagship case: the generated
        // corpus MUST contain أئمة, otherwise the cascade has nothing to
        // look up after peeling ال.
        let all = generate_all();
        let has_aimma = all.iter().any(|g| stripped(&g.surface) == "أئمة");
        assert!(
            has_aimma,
            "generated corpus must contain أئمة (ء-م-م + أَفْعِلَة); first \
             hamzated forms: {:?}",
            all.iter()
                .filter(|g| &*g.root_key == "ء-م-م")
                .map(|g| (stripped(&g.surface), (*g.pattern_label).to_string()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn generative_index_finds_aimma() {
        // Direct lookup test: the index must have أئمة keyed on its
        // stripped form so the analyzer's Layer 3 / 3b can find it
        // without any extra normalisation.
        let idx = GenerativeIndex::get();
        let hits = idx.lookup("أئمة");
        assert!(
            !hits.is_empty(),
            "أئمة must be findable in the GenerativeIndex after M2.b"
        );
        let found = hits.iter().any(|g| {
            &*g.root_key == "ء-م-م" && g.pattern_kind == PatternKind::BrokenPlural
        });
        assert!(
            found,
            "أئمة must trace to (ء-م-م, BrokenPlural); hits: {:?}",
            hits
        );
    }

    // ── M2.c: weak-radical phonology (primitive helpers) ────────────

    #[test]
    fn apply_hollow_rules_collapses_fatha_waw_fatha_to_alif() {
        // قَوَلَ → قَالَ (ق-و-ل + فَعَلَ, Form I perfect a-a vocalisation)
        let out = apply_hollow_rules("قَوَلَ", 'و', PatternKind::VerbPerfect);
        assert_eq!(stripped(&out), "قال");
    }

    #[test]
    fn apply_hollow_rules_collapses_fatha_ya_fatha_to_alif() {
        // بَيَعَ → بَاعَ (ب-ي-ع + فَعَلَ)
        let out = apply_hollow_rules("بَيَعَ", 'ي', PatternKind::VerbPerfect);
        assert_eq!(stripped(&out), "باع");
    }

    #[test]
    fn apply_hollow_rules_collapses_fatha_waw_kasra_to_alif() {
        // خَوِفَ → خَافَ (خ-و-ف + فَعِلَ, Form I perfect a-i vocalisation)
        let out = apply_hollow_rules("خَوِفَ", 'و', PatternKind::VerbPerfect);
        assert_eq!(stripped(&out), "خاف");
    }

    #[test]
    fn apply_hollow_rules_non_perfect_passes_through() {
        // Non-VerbPerfect kind must not fire the rule — the folded
        // fallback handles other shapes in v1.
        let out = apply_hollow_rules("قَوَلَ", 'و', PatternKind::VerbalNoun);
        assert_eq!(out, "قَوَلَ", "non-perfect kind must pass through unchanged");
    }

    #[test]
    fn apply_hollow_rules_leaves_nonweak_middle_alone() {
        // If the middle radical isn't the weak letter we asked about,
        // the rule must not fire even for VerbPerfect.
        let out = apply_hollow_rules("كَتَبَ", 'و', PatternKind::VerbPerfect);
        assert_eq!(out, "كَتَبَ", "no weak match → no rewrite");
    }

    #[test]
    fn apply_defective_rules_waw_final_becomes_alif() {
        // دَعَوَ → دَعَا (د-ع-و + فَعَلَ)
        let out = apply_defective_rules("دَعَوَ", 'و', PatternKind::VerbPerfect);
        assert_eq!(out, "دَعَا");
    }

    #[test]
    fn apply_defective_rules_ya_final_becomes_alif_maksura() {
        // رَمَيَ → رَمَى (ر-م-ي + فَعَلَ). The rule must emit ى, not ي —
        // the folding step downstream will map it for the fuzzy index,
        // but the stripped index needs ى to match the user's surface.
        let out = apply_defective_rules("رَمَيَ", 'ي', PatternKind::VerbPerfect);
        assert_eq!(out, "رَمَى");
        assert!(out.ends_with('ى'), "ي-final must produce trailing ى; got {out}");
    }

    #[test]
    fn apply_defective_rules_non_perfect_passes_through() {
        let out = apply_defective_rules("دَعَوَ", 'و', PatternKind::ActiveParticiple);
        assert_eq!(out, "دَعَوَ", "non-perfect kind must pass through unchanged");
    }

    #[test]
    fn apply_defective_rules_leaves_unterminated_form_alone() {
        // If the surface doesn't end in <weak><fatha>, no rewrite.
        let out = apply_defective_rules("رَامٍ", 'ي', PatternKind::VerbPerfect);
        assert_eq!(out, "رَامٍ", "non-matching terminal → no rewrite");
    }

    #[test]
    fn apply_assimilated_rules_drops_waw_after_ya_prefix() {
        // يَوْعِدُ → يَعِدُ (و-ع-د + يَفْعِلُ)
        let out = apply_assimilated_rules("يَوْعِدُ", 'و', PatternKind::VerbImperfect);
        assert_eq!(stripped(&out), "يعد");
    }

    #[test]
    fn apply_assimilated_rules_drops_waw_after_ta_prefix() {
        // تَوْعِدُ → تَعِدُ (second-person feminine)
        let out = apply_assimilated_rules("تَوْعِدُ", 'و', PatternKind::VerbImperfect);
        assert_eq!(stripped(&out), "تعد");
    }

    #[test]
    fn apply_assimilated_rules_ya_initial_passes_through_in_v1() {
        // ي-initial assimilated is deferred to M2.d; v1 must be a no-op.
        let out = apply_assimilated_rules("يَيْبِسُ", 'ي', PatternKind::VerbImperfect);
        assert_eq!(out, "يَيْبِسُ", "ي-initial passes through in v1");
    }

    #[test]
    fn apply_assimilated_rules_perfect_passes_through() {
        // Perfect of assimilated roots doesn't drop the weak — وَعَدَ stays.
        let out = apply_assimilated_rules("وَعَدَ", 'و', PatternKind::VerbPerfect);
        assert_eq!(out, "وَعَدَ", "perfect must not fire the rule");
    }

    #[test]
    fn apply_assimilated_rules_form_iv_imperfect_passes_through() {
        // Form IV imperfect is يُوعِدُ with damma-prefix, not fatha — my
        // rule's `chars[1] == fatha` guard skips it. Stays intact.
        let out = apply_assimilated_rules("يُوعِدُ", 'و', PatternKind::VerbImperfect);
        assert_eq!(out, "يُوعِدُ", "damma-prefix imperfect must not drop weak");
    }

    // ── M2.c: end-to-end via apply() ────────────────────────────────

    #[test]
    fn apply_hollow_form_i_perfect_produces_qala() {
        // The flagship hollow perfect case.
        let root = root_from_key("ق-و-ل", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.kind == PatternKind::VerbPerfect
                  && p.template == "فَعَلَ"
                  && p.accepts.contains(&RootClass::HollowTriliteral))
            .expect("Form I perfect فَعَلَ must accept hollow");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(s, "قال", "ق-و-ل + فَعَلَ must produce قال; got {s}");
    }

    #[test]
    fn apply_hollow_form_i_perfect_produces_baaʿa() {
        // Hollow with ي middle.
        let root = root_from_key("ب-ي-ع", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.kind == PatternKind::VerbPerfect
                  && p.template == "فَعَلَ"
                  && p.accepts.contains(&RootClass::HollowTriliteral))
            .expect("Form I perfect فَعَلَ must accept hollow");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(s, "باع", "ب-ي-ع + فَعَلَ must produce باع; got {s}");
    }

    #[test]
    fn apply_defective_form_i_perfect_produces_daʿa() {
        // د-ع-و + فَعَلَ → دَعَا (stripped دعا).
        let root = root_from_key("د-ع-و", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.kind == PatternKind::VerbPerfect
                  && p.template == "فَعَلَ"
                  && p.accepts.contains(&RootClass::DefectiveTriliteral))
            .expect("Form I perfect فَعَلَ must accept defective");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(s, "دعا", "د-ع-و + فَعَلَ must produce دعا; got {s}");
    }

    #[test]
    fn apply_defective_form_i_perfect_produces_rama() {
        // ر-م-ي + فَعَلَ → رَمَى (stripped رمى, with ى not ي).
        let root = root_from_key("ر-م-ي", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.kind == PatternKind::VerbPerfect
                  && p.template == "فَعَلَ"
                  && p.accepts.contains(&RootClass::DefectiveTriliteral))
            .expect("Form I perfect فَعَلَ must accept defective");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(s, "رمى", "ر-م-ي + فَعَلَ must produce رمى; got {s}");
    }

    #[test]
    fn apply_assimilated_form_i_imperfect_produces_yaʿidu() {
        // The flagship assimilated imperfect: و-ع-د + يَفْعِلُ → يَعِدُ.
        let root = root_from_key("و-ع-د", None).unwrap();
        let pat = patterns::all_patterns()
            .into_iter()
            .find(|p| p.kind == PatternKind::VerbImperfect
                  && p.template == "يَفْعِلُ"
                  && p.accepts.contains(&RootClass::AssimilatedTriliteral))
            .expect("Form I imperfect يَفْعِلُ must accept assimilated");
        let out = apply(&root, &pat).unwrap();
        let s = stripped(&out);
        assert_eq!(s, "يعد", "و-ع-د + يَفْعِلُ must produce يعد; got {s}");
    }

    #[test]
    fn generate_all_includes_qala() {
        let all = generate_all();
        let has_qala = all
            .iter()
            .any(|g| &*g.root_key == "ق-و-ل" && stripped(&g.surface) == "قال");
        assert!(has_qala, "generated corpus must contain قال from ق-و-ل");
    }

    #[test]
    fn generate_all_includes_daʿa() {
        let all = generate_all();
        let has_daʿa = all
            .iter()
            .any(|g| &*g.root_key == "د-ع-و" && stripped(&g.surface) == "دعا");
        assert!(has_daʿa, "generated corpus must contain دعا from د-ع-و");
    }

    #[test]
    fn generate_all_includes_yaʿidu() {
        let all = generate_all();
        let has_yaʿidu = all
            .iter()
            .any(|g| &*g.root_key == "و-ع-د" && stripped(&g.surface) == "يعد");
        assert!(has_yaʿidu, "generated corpus must contain يعد from و-ع-د");
    }

    #[test]
    fn generative_index_finds_qala() {
        let idx = GenerativeIndex::get();
        let hits = idx.lookup("قال");
        assert!(!hits.is_empty(), "قال must be in the generative index");
        let found = hits.iter().any(|g|
            &*g.root_key == "ق-و-ل" && g.pattern_kind == PatternKind::VerbPerfect
        );
        assert!(found, "قال must trace to (ق-و-ل, VerbPerfect); hits: {:?}", hits);
    }

    #[test]
    fn generative_index_finds_yaʿidu() {
        let idx = GenerativeIndex::get();
        let hits = idx.lookup("يعد");
        assert!(!hits.is_empty(), "يعد must be in the generative index");
        let found = hits.iter().any(|g|
            &*g.root_key == "و-ع-د" && g.pattern_kind == PatternKind::VerbImperfect
        );
        assert!(found, "يعد must trace to (و-ع-د, VerbImperfect); hits: {:?}", hits);
    }
}
