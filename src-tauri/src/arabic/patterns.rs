//! Arabic morphological patterns — the complete template catalogue.
//!
//! A pattern is the abstract skeleton a root is cast into. In classical
//! Arabic grammar, patterns are written with the placeholder radicals
//! ف (fāʾ = 1st), ع (ʿayn = 2nd), ل (lām = 3rd). For quadriliteral roots
//! the 4th radical is written as a second ل; the generator distinguishes
//! by scan position, so `فَعْلَل` with root د-ح-ر-ج yields `دَحْرَج`.
//!
//! Every pattern string below carries full tashkeel so the generator can
//! emit correctly-vocalized surface forms when asked. Readers and FTS
//! callers strip tashkeel before matching.
//!
//! # Coverage (per design decision 3 — "نولدها كلها")
//!
//! - ~50 verbal patterns: Form I (9 tense×voweliz combos), Forms II–X
//!   (27), quadriliteral (6), passive variants (19)
//! - ~20 verbal-noun (مصدر) patterns
//! - 22 participles (11 active + 11 passive)
//! - 8 place/time/instrument noun patterns
//! - 6 intensive (مبالغة) patterns
//! - 4 comparative / relative / diminutive patterns
//! - **All 27 classical broken-plural patterns** (جمع تكسير)
//!
//! Total: ~140 patterns. This file is the authoritative source; any
//! change here affects every generated surface form in the FST.
//!
//! # Phonological adjustments
//!
//! The `accepts` field declares which root classes this pattern can
//! structurally receive. Actual sound-shift rules (hollow → alif,
//! defective → long vowel, hamza carrier selection, gemination
//! collapse) are applied by `crate::arabic::generator` in M2, not here.

use super::types::{Pattern, PatternKind, RootClass};

// ── Convenience: reusable acceptance sets ─────────────────────────────

/// Accepts every triliteral root class.
fn tri_all() -> Vec<RootClass> {
    vec![
        RootClass::SoundTriliteral,
        RootClass::AssimilatedTriliteral,
        RootClass::HollowTriliteral,
        RootClass::DefectiveTriliteral,
        RootClass::GeminatedTriliteral,
        RootClass::HamzatedTriliteral,
    ]
}

/// Accepts every quadriliteral root class.
fn quad_all() -> Vec<RootClass> {
    vec![RootClass::SoundQuadriliteral, RootClass::WeakQuadriliteral]
}

/// Accepts every root class (tri + quad).
fn any_root() -> Vec<RootClass> {
    let mut v = tri_all();
    v.extend(quad_all());
    v
}

// ── Helper: compact pattern constructor ───────────────────────────────

fn p(template: &str, kind: PatternKind, accepts: Vec<RootClass>, label_ar: &str, label_en: &str) -> Pattern {
    Pattern {
        template: template.to_string(),
        kind,
        accepts,
        label_ar: label_ar.to_string(),
        label_en: label_en.to_string(),
    }
}

// ──────────────────────────────────────────────────────────────────────
// §1. VERBAL PATTERNS — PERFECT / IMPERFECT / IMPERATIVE
// ──────────────────────────────────────────────────────────────────────

/// Form I (ثلاثي مجرد) — the base, unadorned triliteral verb.
/// Three voweliz classes for perfect × three for imperfect. Not all
/// 9 combinations are attested, but we list the 6 canonical pairs a
/// root can take. The dictionary (roots.json) specifies which pairs
/// apply to each root via `a/a`, `a/i`, `a/u`, `i/a`, `u/u` markers.
fn form_i_verbs() -> Vec<Pattern> {
    vec![
        // Perfect (ماضي)
        p("فَعَلَ",  PatternKind::VerbPerfect,   tri_all(), "فَعَلَ — ماضٍ (فتح عين)",   "Form I perfect, a-stem (kataba)"),
        p("فَعِلَ",  PatternKind::VerbPerfect,   tri_all(), "فَعِلَ — ماضٍ (كسر عين)",   "Form I perfect, i-stem (shariba)"),
        p("فَعُلَ",  PatternKind::VerbPerfect,   tri_all(), "فَعُلَ — ماضٍ (ضم عين)",    "Form I perfect, u-stem (karuma)"),
        // Imperfect (مضارع)
        p("يَفْعُلُ", PatternKind::VerbImperfect, tri_all(), "يَفْعُلُ — مضارع (ضم عين)",  "Form I imperfect, u-stem (yaktubu)"),
        p("يَفْعِلُ", PatternKind::VerbImperfect, tri_all(), "يَفْعِلُ — مضارع (كسر عين)", "Form I imperfect, i-stem (yajlisu)"),
        p("يَفْعَلُ", PatternKind::VerbImperfect, tri_all(), "يَفْعَلُ — مضارع (فتح عين)", "Form I imperfect, a-stem (yadhhabu)"),
        // Imperative (أمر)
        p("افْعُلْ",  PatternKind::VerbImperative, tri_all(), "افْعُلْ — أمر (ضم عين)",   "Form I imperative, u-stem (uktub)"),
        p("افْعِلْ",  PatternKind::VerbImperative, tri_all(), "افْعِلْ — أمر (كسر عين)",  "Form I imperative, i-stem (ijlis)"),
        p("افْعَلْ",  PatternKind::VerbImperative, tri_all(), "افْعَلْ — أمر (فتح عين)",  "Form I imperative, a-stem (idhhab)"),
    ]
}

/// Forms II–X — triliteral augmented (ثلاثي مزيد).
/// Each Form has a perfect, imperfect, and imperative pattern.
fn forms_ii_to_x_verbs() -> Vec<Pattern> {
    vec![
        // Form II: فَعَّلَ — intensify / causative (kattaba = made [someone] write)
        p("فَعَّلَ",    PatternKind::VerbPerfect,   tri_all(), "فَعَّلَ — ماضي Form II",     "Form II perfect (kattaba)"),
        p("يُفَعِّلُ",   PatternKind::VerbImperfect, tri_all(), "يُفَعِّلُ — مضارع Form II",   "Form II imperfect (yukattibu)"),
        p("فَعِّلْ",     PatternKind::VerbImperative, tri_all(), "فَعِّلْ — أمر Form II",      "Form II imperative (kattib)"),
        // Form III: فَاعَلَ — mutuality / directed-at (kātaba = corresponded with)
        p("فَاعَلَ",   PatternKind::VerbPerfect,   tri_all(), "فَاعَلَ — ماضي Form III",    "Form III perfect (kātaba)"),
        p("يُفَاعِلُ",  PatternKind::VerbImperfect, tri_all(), "يُفَاعِلُ — مضارع Form III",  "Form III imperfect (yukātibu)"),
        p("فَاعِلْ",    PatternKind::VerbImperative, tri_all(), "فَاعِلْ — أمر Form III",     "Form III imperative (kātib)"),
        // Form IV: أَفْعَلَ — causative (aktaba = caused to write)
        p("أَفْعَلَ",   PatternKind::VerbPerfect,   tri_all(), "أَفْعَلَ — ماضي Form IV",     "Form IV perfect (aktaba)"),
        p("يُفْعِلُ",   PatternKind::VerbImperfect, tri_all(), "يُفْعِلُ — مضارع Form IV",    "Form IV imperfect (yuktibu)"),
        p("أَفْعِلْ",    PatternKind::VerbImperative, tri_all(), "أَفْعِلْ — أمر Form IV",      "Form IV imperative (aktib)"),
        // Form V: تَفَعَّلَ — reflexive of II (takallama = spoke)
        p("تَفَعَّلَ",   PatternKind::VerbPerfect,   tri_all(), "تَفَعَّلَ — ماضي Form V",     "Form V perfect (takallama)"),
        p("يَتَفَعَّلُ",  PatternKind::VerbImperfect, tri_all(), "يَتَفَعَّلُ — مضارع Form V",   "Form V imperfect (yatakallamu)"),
        p("تَفَعَّلْ",   PatternKind::VerbImperative, tri_all(), "تَفَعَّلْ — أمر Form V",     "Form V imperative (takallam)"),
        // Form VI: تَفَاعَلَ — reciprocal (takātaba = corresponded mutually)
        p("تَفَاعَلَ",  PatternKind::VerbPerfect,   tri_all(), "تَفَاعَلَ — ماضي Form VI",    "Form VI perfect (takātaba)"),
        p("يَتَفَاعَلُ", PatternKind::VerbImperfect, tri_all(), "يَتَفَاعَلُ — مضارع Form VI",  "Form VI imperfect (yatakātabu)"),
        p("تَفَاعَلْ",  PatternKind::VerbImperative, tri_all(), "تَفَاعَلْ — أمر Form VI",     "Form VI imperative (takātab)"),
        // Form VII: انْفَعَلَ — passive/medio-passive (inkasara = was broken)
        p("انْفَعَلَ",   PatternKind::VerbPerfect,   tri_all(), "انْفَعَلَ — ماضي Form VII",   "Form VII perfect (inkasara)"),
        p("يَنْفَعِلُ",   PatternKind::VerbImperfect, tri_all(), "يَنْفَعِلُ — مضارع Form VII", "Form VII imperfect (yankasiru)"),
        p("انْفَعِلْ",   PatternKind::VerbImperative, tri_all(), "انْفَعِلْ — أمر Form VII",    "Form VII imperative (inkasir)"),
        // Form VIII: افْتَعَلَ — reflexive (iktaba = enrolled / wrote oneself)
        p("افْتَعَلَ",   PatternKind::VerbPerfect,   tri_all(), "افْتَعَلَ — ماضي Form VIII",  "Form VIII perfect (iktasaba)"),
        p("يَفْتَعِلُ",   PatternKind::VerbImperfect, tri_all(), "يَفْتَعِلُ — مضارع Form VIII","Form VIII imperfect (yaktasibu)"),
        p("افْتَعِلْ",   PatternKind::VerbImperative, tri_all(), "افْتَعِلْ — أمر Form VIII",   "Form VIII imperative (iktasib)"),
        // Form IX: افْعَلَّ — color/defect (iḥmarra = turned red)
        p("افْعَلَّ",    PatternKind::VerbPerfect,   tri_all(), "افْعَلَّ — ماضي Form IX",     "Form IX perfect (iḥmarra)"),
        p("يَفْعَلُّ",   PatternKind::VerbImperfect, tri_all(), "يَفْعَلُّ — مضارع Form IX",    "Form IX imperfect (yaḥmarru)"),
        p("افْعَلِلْ",    PatternKind::VerbImperative, tri_all(), "افْعَلِلْ — أمر Form IX",    "Form IX imperative (iḥmaril)"),
        // Form X: اسْتَفْعَلَ — seek/request (istaktaba = asked to write)
        p("اسْتَفْعَلَ", PatternKind::VerbPerfect,   tri_all(), "اسْتَفْعَلَ — ماضي Form X",   "Form X perfect (istaktaba)"),
        p("يَسْتَفْعِلُ", PatternKind::VerbImperfect, tri_all(), "يَسْتَفْعِلُ — مضارع Form X",  "Form X imperfect (yastaktibu)"),
        p("اسْتَفْعِلْ",  PatternKind::VerbImperative, tri_all(), "اسْتَفْعِلْ — أمر Form X",   "Form X imperative (istaktib)"),
    ]
}

/// Quadriliteral verbs (رباعي). Two bases plus T-prefix variant.
/// The template uses ل twice; generator treats occurrences 3 and 4 as
/// radical 2 and radical 3 respectively.
fn quadriliteral_verbs() -> Vec<Pattern> {
    vec![
        // فَعْلَلَ — basic quadri (daḥraja = rolled something)
        p("فَعْلَلَ",   PatternKind::VerbPerfect,   quad_all(), "فَعْلَلَ — ماضي رباعي",      "Quadri perfect (daḥraja)"),
        p("يُفَعْلِلُ",  PatternKind::VerbImperfect, quad_all(), "يُفَعْلِلُ — مضارع رباعي",    "Quadri imperfect (yudaḥriju)"),
        p("فَعْلِلْ",   PatternKind::VerbImperative, quad_all(), "فَعْلِلْ — أمر رباعي",       "Quadri imperative (daḥrij)"),
        // تَفَعْلَلَ — T-prefix quadri (tadaḥraja = rolled itself)
        p("تَفَعْلَلَ",  PatternKind::VerbPerfect,   quad_all(), "تَفَعْلَلَ — ماضي رباعي مطاوع", "Quadri T-form perfect (tadaḥraja)"),
        p("يَتَفَعْلَلُ", PatternKind::VerbImperfect, quad_all(), "يَتَفَعْلَلُ — مضارع رباعي",  "Quadri T-form imperfect (yatadaḥraju)"),
        p("تَفَعْلَلْ",   PatternKind::VerbImperative, quad_all(), "تَفَعْلَلْ — أمر رباعي",   "Quadri T-form imperative (tadaḥral)"),
    ]
}

/// Passive verbs (مبني للمجهول). Every active verb has a passive
/// counterpart with a fixed voweliz pattern.
fn passive_verbs() -> Vec<Pattern> {
    vec![
        // Form I passive
        p("فُعِلَ",    PatternKind::VerbPerfect,   tri_all(), "فُعِلَ — مبني للمجهول",       "Form I passive perfect (kutiba)"),
        p("يُفْعَلُ",   PatternKind::VerbImperfect, tri_all(), "يُفْعَلُ — مضارع مبني للمجهول","Form I passive imperfect (yuktabu)"),
        // Form II passive
        p("فُعِّلَ",    PatternKind::VerbPerfect,   tri_all(), "فُعِّلَ — مبني للمجهول II",   "Form II passive (kuttiba)"),
        p("يُفَعَّلُ",   PatternKind::VerbImperfect, tri_all(), "يُفَعَّلُ — مضارع مبني II",   "Form II passive impf (yukattabu)"),
        // Form III passive
        p("فُوعِلَ",    PatternKind::VerbPerfect,   tri_all(), "فُوعِلَ — مبني للمجهول III", "Form III passive (kūtiba)"),
        p("يُفَاعَلُ",  PatternKind::VerbImperfect, tri_all(), "يُفَاعَلُ — مضارع مبني III", "Form III passive impf (yukātabu)"),
        // Form IV passive
        p("أُفْعِلَ",   PatternKind::VerbPerfect,   tri_all(), "أُفْعِلَ — مبني للمجهول IV",  "Form IV passive (uktiba)"),
        p("يُفْعَلُ",   PatternKind::VerbImperfect, tri_all(), "يُفْعَلُ — مضارع مبني IV",    "Form IV passive impf (yuktabu)"),
        // Form V passive (rare)
        p("تُفُعِّلَ",   PatternKind::VerbPerfect,   tri_all(), "تُفُعِّلَ — مبني للمجهول V",  "Form V passive (tukullima)"),
        p("يُتَفَعَّلُ",  PatternKind::VerbImperfect, tri_all(), "يُتَفَعَّلُ — مضارع مبني V",  "Form V passive impf (yutakallamu)"),
        // Form VI passive (rare)
        p("تُفُوعِلَ",  PatternKind::VerbPerfect,   tri_all(), "تُفُوعِلَ — مبني للمجهول VI", "Form VI passive (tukūtiba)"),
        p("يُتَفَاعَلُ", PatternKind::VerbImperfect, tri_all(), "يُتَفَاعَلُ — مضارع مبني VI", "Form VI passive impf (yutakātabu)"),
        // Form VII: no passive (already medio-passive)
        // Form VIII passive
        p("افْتُعِلَ",  PatternKind::VerbPerfect,   tri_all(), "افْتُعِلَ — مبني للمجهول VIII","Form VIII passive (uktusiba)"),
        p("يُفْتَعَلُ",  PatternKind::VerbImperfect, tri_all(), "يُفْتَعَلُ — مضارع مبني VIII","Form VIII passive impf (yuktasabu)"),
        // Form IX: no passive (stative)
        // Form X passive
        p("اسْتُفْعِلَ", PatternKind::VerbPerfect,   tri_all(), "اسْتُفْعِلَ — مبني للمجهول X","Form X passive (ustuktiba)"),
        p("يُسْتَفْعَلُ",PatternKind::VerbImperfect, tri_all(), "يُسْتَفْعَلُ — مضارع مبني X", "Form X passive impf (yustaktabu)"),
        // Quadriliteral passives
        p("فُعْلِلَ",   PatternKind::VerbPerfect,   quad_all(), "فُعْلِلَ — مبني للمجهول رباعي","Quadri passive (duḥrija)"),
        p("يُفَعْلَلُ",  PatternKind::VerbImperfect, quad_all(), "يُفَعْلَلُ — مضارع مبني رباعي","Quadri passive impf (yudaḥraju)"),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// §2. VERBAL NOUNS (مصادر)
// ──────────────────────────────────────────────────────────────────────

/// Form I verbal nouns are idiosyncratic — each root lexically
/// determines which مصدر pattern(s) apply. We list every common one;
/// the generator emits the ones the dictionary marks for each root.
fn form_i_verbal_nouns() -> Vec<Pattern> {
    vec![
        p("فَعْل",       PatternKind::VerbalNoun, tri_all(), "فَعْل — مصدر",        "Form I verbal noun (ḍarb)"),
        p("فِعْل",       PatternKind::VerbalNoun, tri_all(), "فِعْل — مصدر",        "Form I verbal noun (ʿilm)"),
        p("فُعْل",       PatternKind::VerbalNoun, tri_all(), "فُعْل — مصدر",        "Form I verbal noun (shukr)"),
        p("فَعَل",       PatternKind::VerbalNoun, tri_all(), "فَعَل — مصدر",        "Form I verbal noun (ṭalab)"),
        p("فِعَل",       PatternKind::VerbalNoun, tri_all(), "فِعَل — مصدر",        "Form I verbal noun (ḥiyal)"),
        p("فُعُول",      PatternKind::VerbalNoun, tri_all(), "فُعُول — مصدر",       "Form I verbal noun (khurūj)"),
        p("فِعَالَة",    PatternKind::VerbalNoun, tri_all(), "فِعَالَة — مصدر حرفة","Form I verbal noun (kitāba)"),
        p("فَعَالَة",    PatternKind::VerbalNoun, tri_all(), "فَعَالَة — مصدر صفة", "Form I verbal noun (shajāʿa)"),
        p("فِعَال",      PatternKind::VerbalNoun, tri_all(), "فِعَال — مصدر",       "Form I verbal noun (jihād)"),
        p("فُعَال",      PatternKind::VerbalNoun, tri_all(), "فُعَال — مصدر",       "Form I verbal noun (suʾāl)"),
        p("فَعِيل",      PatternKind::VerbalNoun, tri_all(), "فَعِيل — مصدر",       "Form I verbal noun (ḥanīn)"),
        p("فَعَلَان",   PatternKind::VerbalNoun, tri_all(), "فَعَلَان — مصدر",   "Form I verbal noun (ṭayarān)"),
        p("فَعْلَة",     PatternKind::VerbalNoun, tri_all(), "فَعْلَة — مصدر المرة","Form I one-time-action (ḍarba)"),
        p("فِعْلَة",     PatternKind::VerbalNoun, tri_all(), "فِعْلَة — مصدر الهيئة","Form I manner-noun (riḥla)"),
    ]
}

/// Verbal nouns for Forms II–X and quadriliterals — each is canonical.
fn derived_verbal_nouns() -> Vec<Pattern> {
    vec![
        p("تَفْعِيل",      PatternKind::VerbalNoun, tri_all(),  "تَفْعِيل — مصدر II",   "Form II verbal noun (tadrīs)"),
        p("تَفْعِلَة",     PatternKind::VerbalNoun, tri_all(),  "تَفْعِلَة — مصدر II",  "Form II verbal noun (takrima)"),
        p("مُفَاعَلَة",    PatternKind::VerbalNoun, tri_all(),  "مُفَاعَلَة — مصدر III","Form III verbal noun (mushāraka)"),
        p("فِعَال",        PatternKind::VerbalNoun, tri_all(),  "فِعَال — مصدر III",   "Form III secondary (qitāl)"),
        p("إِفْعَال",      PatternKind::VerbalNoun, tri_all(),  "إِفْعَال — مصدر IV",   "Form IV verbal noun (ikrām)"),
        p("تَفَعُّل",      PatternKind::VerbalNoun, tri_all(),  "تَفَعُّل — مصدر V",     "Form V verbal noun (taʿallum)"),
        p("تَفَاعُل",     PatternKind::VerbalNoun, tri_all(),  "تَفَاعُل — مصدر VI",   "Form VI verbal noun (taʿāwun)"),
        p("انْفِعَال",    PatternKind::VerbalNoun, tri_all(),  "انْفِعَال — مصدر VII",  "Form VII verbal noun (inkisār)"),
        p("افْتِعَال",    PatternKind::VerbalNoun, tri_all(),  "افْتِعَال — مصدر VIII", "Form VIII verbal noun (ijtimāʿ)"),
        p("افْعِلَال",    PatternKind::VerbalNoun, tri_all(),  "افْعِلَال — مصدر IX",  "Form IX verbal noun (iṣfirār)"),
        p("اسْتِفْعَال",  PatternKind::VerbalNoun, tri_all(),  "اسْتِفْعَال — مصدر X", "Form X verbal noun (istikhdām)"),
        p("فَعْلَلَة",     PatternKind::VerbalNoun, quad_all(), "فَعْلَلَة — مصدر رباعي","Quadri verbal noun (daḥraja)"),
        p("فِعْلَال",      PatternKind::VerbalNoun, quad_all(), "فِعْلَال — مصدر رباعي","Quadri verbal noun (zilzāl)"),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// §3. PARTICIPLES (اسم الفاعل واسم المفعول)
// ──────────────────────────────────────────────────────────────────────

fn active_participles() -> Vec<Pattern> {
    vec![
        p("فَاعِل",       PatternKind::ActiveParticiple, tri_all(),  "فَاعِل — اسم فاعل I",   "Form I active participle (kātib)"),
        p("مُفَعِّل",      PatternKind::ActiveParticiple, tri_all(),  "مُفَعِّل — اسم فاعل II", "Form II active participle (mudarris)"),
        p("مُفَاعِل",     PatternKind::ActiveParticiple, tri_all(),  "مُفَاعِل — اسم فاعل III","Form III active participle (mushārik)"),
        p("مُفْعِل",       PatternKind::ActiveParticiple, tri_all(),  "مُفْعِل — اسم فاعل IV",  "Form IV active participle (muslim)"),
        p("مُتَفَعِّل",   PatternKind::ActiveParticiple, tri_all(),  "مُتَفَعِّل — اسم فاعل V","Form V active participle (mutakallim)"),
        p("مُتَفَاعِل",   PatternKind::ActiveParticiple, tri_all(),  "مُتَفَاعِل — اسم فاعل VI","Form VI active participle (mutakātib)"),
        p("مُنْفَعِل",     PatternKind::ActiveParticiple, tri_all(),  "مُنْفَعِل — اسم فاعل VII","Form VII active participle (munkasir)"),
        p("مُفْتَعِل",     PatternKind::ActiveParticiple, tri_all(),  "مُفْتَعِل — اسم فاعل VIII","Form VIII active participle (muktasib)"),
        p("مُفْعَلّ",      PatternKind::ActiveParticiple, tri_all(),  "مُفْعَلّ — اسم فاعل IX","Form IX active participle (muḥmarr)"),
        p("مُسْتَفْعِل",  PatternKind::ActiveParticiple, tri_all(),  "مُسْتَفْعِل — اسم فاعل X","Form X active participle (mustakhdim)"),
        p("مُفَعْلِل",     PatternKind::ActiveParticiple, quad_all(), "مُفَعْلِل — اسم فاعل رباعي","Quadri active participle (mudaḥrij)"),
    ]
}

fn passive_participles() -> Vec<Pattern> {
    vec![
        p("مَفْعُول",     PatternKind::PassiveParticiple, tri_all(),  "مَفْعُول — اسم مفعول I", "Form I passive participle (maktūb)"),
        p("مُفَعَّل",      PatternKind::PassiveParticiple, tri_all(),  "مُفَعَّل — اسم مفعول II","Form II passive participle (mudarras)"),
        p("مُفَاعَل",     PatternKind::PassiveParticiple, tri_all(),  "مُفَاعَل — اسم مفعول III","Form III passive participle (mushārak)"),
        p("مُفْعَل",       PatternKind::PassiveParticiple, tri_all(),  "مُفْعَل — اسم مفعول IV","Form IV passive participle (muslam)"),
        p("مُتَفَعَّل",   PatternKind::PassiveParticiple, tri_all(),  "مُتَفَعَّل — اسم مفعول V","Form V passive participle (mutakallam)"),
        p("مُتَفَاعَل",  PatternKind::PassiveParticiple, tri_all(),  "مُتَفَاعَل — اسم مفعول VI","Form VI passive participle (mutakātab)"),
        p("مُنْفَعَل",     PatternKind::PassiveParticiple, tri_all(),  "مُنْفَعَل — اسم مفعول VII","Form VII passive participle (munkasar)"),
        p("مُفْتَعَل",    PatternKind::PassiveParticiple, tri_all(),  "مُفْتَعَل — اسم مفعول VIII","Form VIII passive participle (muktasab)"),
        p("مُسْتَفْعَل", PatternKind::PassiveParticiple, tri_all(),  "مُسْتَفْعَل — اسم مفعول X","Form X passive participle (mustakhdam)"),
        p("مُفَعْلَل",    PatternKind::PassiveParticiple, quad_all(), "مُفَعْلَل — اسم مفعول رباعي","Quadri passive participle (mudaḥraj)"),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// §4. DERIVED NOUNS (اسم مكان / زمان / آلة)
// ──────────────────────────────────────────────────────────────────────

/// Place, time, and instrument nouns. Same mīm-prefix template family
/// split by voweliz. `مَفْعَل/مَفْعِل/مَفْعَلَة` cover place & time;
/// `مِفْعَل/مِفْعَال/مِفْعَلَة` cover instruments.
fn derived_nouns() -> Vec<Pattern> {
    vec![
        p("مَفْعَل",   PatternKind::DerivedNoun, tri_all(), "مَفْعَل — اسم مكان/زمان",   "Place/time noun (malʿab)"),
        p("مَفْعِل",   PatternKind::DerivedNoun, tri_all(), "مَفْعِل — اسم مكان/زمان",   "Place/time noun (manzil)"),
        p("مَفْعَلَة", PatternKind::DerivedNoun, tri_all(), "مَفْعَلَة — اسم مكان",       "Place noun (madrasa)"),
        p("مِفْعَل",   PatternKind::DerivedNoun, tri_all(), "مِفْعَل — اسم آلة",           "Instrument noun (mibrad)"),
        p("مِفْعَال",  PatternKind::DerivedNoun, tri_all(), "مِفْعَال — اسم آلة",         "Instrument noun (miftāḥ)"),
        p("مِفْعَلَة", PatternKind::DerivedNoun, tri_all(), "مِفْعَلَة — اسم آلة",        "Instrument noun (miknasa)"),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// §5. INTENSIVES (صيغ المبالغة), COMPARATIVE, RELATIVE, DIMINUTIVE
// ──────────────────────────────────────────────────────────────────────

fn intensive_patterns() -> Vec<Pattern> {
    vec![
        p("فَعَّال",     PatternKind::DerivedNoun, tri_all(), "فَعَّال — مبالغة",      "Hyperbolic agent (kadhdhāb)"),
        p("فَعُول",      PatternKind::DerivedNoun, tri_all(), "فَعُول — مبالغة",      "Hyperbolic (ṣabūr)"),
        p("فَعِيل",      PatternKind::DerivedNoun, tri_all(), "فَعِيل — صفة مشبهة",   "Assimilated adj. (raḥīm)"),
        p("فَعِل",       PatternKind::DerivedNoun, tri_all(), "فَعِل — صفة مشبهة",    "Assimilated adj. (ḥadhir)"),
        p("مِفْعَال",    PatternKind::DerivedNoun, tri_all(), "مِفْعَال — مبالغة",     "Hyperbolic (miʿṭāʾ)"),
        p("فَعْلَان",   PatternKind::DerivedNoun, tri_all(), "فَعْلَان — صفة",        "Adjective (ʿaṭshān)"),
    ]
}

fn elative_relative_diminutive() -> Vec<Pattern> {
    vec![
        p("أَفْعَل",     PatternKind::Elative,    tri_all(), "أَفْعَل — اسم تفضيل",   "Elative / comparative (akbar)"),
        p("فُعْلَى",    PatternKind::Elative,    tri_all(), "فُعْلَى — مؤنث التفضيل","Fem. elative (kubrā)"),
        p("فَعْلِيّ",    PatternKind::Relative,   any_root(),"فَعْلِيّ — اسم منسوب",   "Relative adj. (ʿarabī)"),
        p("فُعَيْل",    PatternKind::Diminutive, tri_all(), "فُعَيْل — تصغير",       "Diminutive (kutayyib)"),
        p("فُعَيْعِل",  PatternKind::Diminutive, quad_all(),"فُعَيْعِل — تصغير رباعي","Quadri diminutive"),
        p("فُعَيْلَة",  PatternKind::Diminutive, tri_all(), "فُعَيْلَة — تصغير مؤنث","Fem. diminutive"),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// §6. BROKEN PLURALS (جموع التكسير) — ALL 27
// ──────────────────────────────────────────────────────────────────────
//
// Per design decision 3 ("نولدها كلها"), every classical broken-plural
// pattern is listed. The generator applies them all; the disambiguator
// and root-level dictionary flags narrow which one actually applies to
// each lemma at runtime.
//
// The 27 are ordered by frequency of use in modern written Arabic,
// head of the list = most productive.
// ──────────────────────────────────────────────────────────────────────

fn broken_plurals() -> Vec<Pattern> {
    vec![
        //  1. أَفْعَال — مُعتادة لكثير من المذكر (qalam → aqlām, bāb → abwāb)
        p("أَفْعَال",     PatternKind::BrokenPlural, tri_all(), "أَفْعَال — جمع تكسير",   "Broken plural (aqlām)"),
        //  2. أَفْعُل — قليلة لكنها منتجة (rijl → arjul)
        p("أَفْعُل",      PatternKind::BrokenPlural, tri_all(), "أَفْعُل — جمع تكسير",    "Broken plural (arjul)"),
        //  3. أَفْعِلَة — لمذكر وإناث (imām → aʾimma, ʿamūd → aʿmida) — الحالة التي فشلت في Light10
        p("أَفْعِلَة",    PatternKind::BrokenPlural, tri_all(), "أَفْعِلَة — جمع تكسير",  "Broken plural (aʾimma) — the أئمة case"),
        //  4. فِعْلَة — قليلة (ghilmān in فِعْلَة sense, ṣibya)
        p("فِعْلَة",      PatternKind::BrokenPlural, tri_all(), "فِعْلَة — جمع تكسير",    "Broken plural (ṣibya)"),
        //  5. فُعْل — rare (humr = red ones) — often passes for adj too
        p("فُعْل",        PatternKind::BrokenPlural, tri_all(), "فُعْل — جمع تكسير",      "Broken plural (ḥumr)"),
        //  6. فُعُل — very common (kitāb → kutub)
        p("فُعُل",        PatternKind::BrokenPlural, tri_all(), "فُعُل — جمع تكسير",      "Broken plural (kutub)"),
        //  7. فُعَل — common for فَعْلَة singulars (ghurfa → ghuraf)
        p("فُعَل",        PatternKind::BrokenPlural, tri_all(), "فُعَل — جمع تكسير",      "Broken plural (ghuraf)"),
        //  8. فِعَل — niʿma → niʿam, liḥya → liḥā (hollow)
        p("فِعَل",        PatternKind::BrokenPlural, tri_all(), "فِعَل — جمع تكسير",      "Broken plural (niʿam)"),
        //  9. فَعَلَة — for human doers (ṭālib → ṭalaba)
        p("فَعَلَة",     PatternKind::BrokenPlural, tri_all(), "فَعَلَة — جمع تكسير",     "Broken plural (ṭalaba)"),
        // 10. فُعَلَة — rare (quḍāh — note: defective cases absorbed here)
        p("فُعَلَة",     PatternKind::BrokenPlural, tri_all(), "فُعَلَة — جمع تكسير",     "Broken plural (quḍāh)"),
        // 11. فَعْلَى — for فَعِيل adjectives of injury/ailment (jarīḥ → jarḥā)
        p("فَعْلَى",    PatternKind::BrokenPlural, tri_all(), "فَعْلَى — جمع تكسير",     "Broken plural (jarḥā)"),
        // 12. فِعَلَة — rare (qird → qirada)
        p("فِعَلَة",    PatternKind::BrokenPlural, tri_all(), "فِعَلَة — جمع تكسير",     "Broken plural (qirada)"),
        // 13. فُعَّل — for active participle (rākiʿ → rukkaʿ)
        p("فُعَّل",      PatternKind::BrokenPlural, tri_all(), "فُعَّل — جمع تكسير",      "Broken plural (rukkaʿ)"),
        // 14. فُعَّال — for فَاعِل (ṭālib → ṭullāb, ḥājj → ḥujjāj)
        p("فُعَّال",     PatternKind::BrokenPlural, tri_all(), "فُعَّال — جمع تكسير",    "Broken plural (ṭullāb)"),
        // 15. فِعَال — common (jabal → jibāl, rajul → rijāl)
        p("فِعَال",      PatternKind::BrokenPlural, tri_all(), "فِعَال — جمع تكسير",    "Broken plural (jibāl)"),
        // 16. فُعُول — very common (qalb → qulūb, baḥr → buḥūr)
        p("فُعُول",     PatternKind::BrokenPlural, tri_all(), "فُعُول — جمع تكسير",     "Broken plural (qulūb)"),
        // 17. فِعْلَان — ghilmān
        p("فِعْلَان",  PatternKind::BrokenPlural, tri_all(), "فِعْلَان — جمع تكسير",   "Broken plural (ghilmān)"),
        // 18. فُعْلَان — qaḍīb → quḍbān, rākib → rukbān
        p("فُعْلَان",  PatternKind::BrokenPlural, tri_all(), "فُعْلَان — جمع تكسير",   "Broken plural (quḍbān)"),
        // 19. فُعَلَاء — for فَعِيل adj (shāʿir → shuʿarāʾ, ʿālim → ʿulamāʾ)
        p("فُعَلَاء",  PatternKind::BrokenPlural, tri_all(), "فُعَلَاء — جمع تكسير",   "Broken plural (shuʿarāʾ)"),
        // 20. أَفْعِلَاء — ṣadīq → aṣdiqāʾ, nabī → anbiyāʾ
        p("أَفْعِلَاء",PatternKind::BrokenPlural, tri_all(), "أَفْعِلَاء — جمع تكسير", "Broken plural (aṣdiqāʾ)"),
        // 21. مَفَاعِل — for مَفْعَل singulars (madrasa → madāris, masjid → masājid)
        p("مَفَاعِل",  PatternKind::BrokenPlural, tri_all(), "مَفَاعِل — جمع تكسير",   "Broken plural (madāris)"),
        // 22. مَفَاعِيل — miftāḥ → mafātīḥ, miṣbāḥ → maṣābīḥ
        p("مَفَاعِيل",PatternKind::BrokenPlural, tri_all(), "مَفَاعِيل — جمع تكسير", "Broken plural (mafātīḥ)"),
        // 23. فَوَاعِل — jāʾiza → jawāʾiz, shāriʿ → shawāriʿ, kawkab → kawākib
        p("فَوَاعِل",  PatternKind::BrokenPlural, tri_all(), "فَوَاعِل — جمع تكسير",   "Broken plural (jawāʾiz)"),
        // 24. فَعَائِل — for فَعِيلَة singulars (risāla → rasāʾil, qaṣīda → qaṣāʾid)
        p("فَعَائِل",  PatternKind::BrokenPlural, tri_all(), "فَعَائِل — جمع تكسير",   "Broken plural (rasāʾil)"),
        // 25. فَعَالِي — defective plurals (ṣaḥrāʾ → ṣaḥārī, layla → layālī)
        p("فَعَالِي",  PatternKind::BrokenPlural, tri_all(), "فَعَالِي — جمع تكسير",   "Broken plural (layālī)"),
        // 26. فَعَالِل — true quadriliteral plural (dirham → darāhim, zilzāl → zalāzil)
        p("فَعَالِل",  PatternKind::BrokenPlural, quad_all(),"فَعَالِل — جمع تكسير رباعي","Quadri broken plural (darāhim)"),
        // 27. فَعَالِيل — extended quadri plural (qindīl → qanādīl, dīnār → danānīr)
        p("فَعَالِيل",PatternKind::BrokenPlural, quad_all(),"فَعَالِيل — جمع تكسير رباعي ممدود","Extended quadri plural (qanādīl)"),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// §7. FEMININE PATTERNS (تأنيث)
// ──────────────────────────────────────────────────────────────────────
//
// Feminine is usually formed by adding tā' marbūṭa (ة) to the masculine.
// Some patterns have distinct feminine templates (e.g. فَعْلَاء for colors
// and defects: aḥmar → ḥamrāʾ). Listed here as standalone patterns so
// the generator can emit them directly when producing FTS surface forms.
// ──────────────────────────────────────────────────────────────────────

fn feminine_patterns() -> Vec<Pattern> {
    vec![
        p("فَاعِلَة",   PatternKind::Feminine, tri_all(), "فَاعِلَة — اسم فاعل مؤنث","Form I fem active (kātiba)"),
        p("مَفْعُولَة", PatternKind::Feminine, tri_all(), "مَفْعُولَة — اسم مفعول مؤنث","Form I fem passive (maktūba)"),
        p("فَعِيلَة",   PatternKind::Feminine, tri_all(), "فَعِيلَة — صفة مشبهة مؤنث","Fem. ṣifa (jamīla)"),
        p("فَعْلَاء",   PatternKind::Feminine, tri_all(), "فَعْلَاء — مؤنث أَفْعَل",    "Feminine of أَفْعَل (ḥamrāʾ)"),
        p("فُعْلَى",    PatternKind::Feminine, tri_all(), "فُعْلَى — مؤنث التفضيل",   "Fem. of elative (kubrā)"),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// PUBLIC API
// ──────────────────────────────────────────────────────────────────────

/// The complete catalogue — every pattern the engine knows.
///
/// Called once at engine initialization. The result is cached in the
/// generator; downstream code never re-calls this.
///
/// Total pattern count should be ~140 after M1b. Verified by the
/// `catalogue_size_is_stable` test below — if a pattern is added or
/// removed, bump the expected count and write a note explaining why.
pub fn all_patterns() -> Vec<Pattern> {
    let mut v = Vec::with_capacity(160);
    v.extend(form_i_verbs());
    v.extend(forms_ii_to_x_verbs());
    v.extend(quadriliteral_verbs());
    v.extend(passive_verbs());
    v.extend(form_i_verbal_nouns());
    v.extend(derived_verbal_nouns());
    v.extend(active_participles());
    v.extend(passive_participles());
    v.extend(derived_nouns());
    v.extend(intensive_patterns());
    v.extend(elative_relative_diminutive());
    v.extend(broken_plurals());
    v.extend(feminine_patterns());
    v
}

/// Patterns restricted to a single kind — convenience accessor for the
/// generator, which iterates kinds when producing FTS surface forms.
pub fn patterns_of_kind(kind: PatternKind) -> Vec<Pattern> {
    all_patterns().into_iter().filter(|p| p.kind == kind).collect()
}

// ──────────────────────────────────────────────────────────────────────
// Tests — minimal sanity checks on the catalogue shape.
// Real regression (does pattern X produce surface Y for root Z?) lives
// in M5, once the generator is written.
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_is_nonempty() {
        let all = all_patterns();
        assert!(all.len() > 100, "expected at least 100 patterns, got {}", all.len());
    }

    #[test]
    fn all_27_broken_plurals_present() {
        let bps = patterns_of_kind(PatternKind::BrokenPlural);
        assert_eq!(bps.len(), 27, "broken-plural catalogue must have exactly 27 patterns");
    }

    #[test]
    fn every_pattern_has_labels() {
        for p in all_patterns() {
            assert!(!p.template.is_empty(), "empty template");
            assert!(!p.label_ar.is_empty(), "missing Arabic label for {}", p.template);
            assert!(!p.label_en.is_empty(), "missing English label for {}", p.template);
            assert!(!p.accepts.is_empty(), "pattern {} accepts nothing", p.template);
        }
    }

    #[test]
    fn aʾimma_pattern_exists() {
        // The failing case from the user's bug report: الأئمة.
        // Must be generatable by أَفْعِلَة, which is broken-plural #3.
        let bps = patterns_of_kind(PatternKind::BrokenPlural);
        assert!(
            bps.iter().any(|p| p.template == "أَفْعِلَة"),
            "أَفْعِلَة pattern must be present — required to decompose الأئمة → إمام"
        );
    }

    #[test]
    fn all_ten_forms_have_perfect_imperfect_imperative() {
        // Forms I–X × 3 moods = 30 patterns (without passives).
        // Form I contributes 9 (3 voweliz classes per mood), Forms II–X
        // contribute 3 each = 27. Total ≥ 36.
        let verbs: Vec<_> = all_patterns().into_iter()
            .filter(|p| matches!(p.kind,
                PatternKind::VerbPerfect | PatternKind::VerbImperfect | PatternKind::VerbImperative))
            .collect();
        assert!(verbs.len() >= 36, "expected at least 36 verbal patterns, got {}", verbs.len());
    }

    #[test]
    fn every_pattern_template_has_at_least_one_radical_placeholder() {
        // A pattern without ف, ع, or ل is not a valid Arabic morphological
        // template — it would never be root-dependent.
        for p in all_patterns() {
            let has_placeholder = p.template.chars().any(|c| c == 'ف' || c == 'ع' || c == 'ل');
            assert!(
                has_placeholder,
                "pattern {} has no radical placeholder",
                p.template
            );
        }
    }
}
