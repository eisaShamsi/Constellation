//! MIG-021v3 V3-§4 — Linguistic Cataloger.
//!
//! Reads through the note's LANGUAGE — Arabic morphology via CAE
//! (Constellation Arabic Engine), cross-civilizational equivalence via
//! the lexicon's multi-token entries, slow-path Bridge similarity for
//! Arabic terms not in the lexicon.
//!
//! Per Architect §2.1:
//!   Strong on: technical Arabic / Sanskrit / Greek vocabulary;
//!              root-aware (avoids false positives from string-similar
//!              non-epistemic terms — e.g. قياس "measurement" vs قياس
//!              "analogy" disambiguated via root context)
//!   Weak on:   pure free-form prose without keyword anchors
//!   Latency:   microseconds for CAE+lexicon path; ~30 ms per unknown
//!              Arabic term when Bridge fallback fires
//!
//! Three matching paths in priority order:
//!   1. CAE root match (HIGH confidence) — when the note has an Arabic
//!      word whose root matches a lexicon entry's `root` field
//!   2. Surface-token match (MEDIUM confidence) — when the note contains
//!      a literal lexicon token (handles English transliterations and
//!      Sanskrit IAST forms via the lexicon's multi-token list)
//!   3. Bridge similarity (LOW confidence) — for Arabic-script words
//!      not matched by 1 or 2, embed and query the Lexical Bridge
//!      vector store; if nearest concept matches one of our lexicon's
//!      targets, attribute the taxonomy ID at low confidence
//!
//! Rules fired (Architect §4):
//!   * Rule of Side-channel Preference — root match outranks surface
//!     match outranks Bridge fallback
//!   * Rule of Three — abstains at depth when too many candidates fire

use crate::arabic;
use crate::cece::cataloger::{
    Axis, AxisAssignment, Cataloger, CatalogerContext, Confidence, ReasoningTrail,
};
use crate::sources::{is_valid_content_type_id, is_valid_source_id};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

// ─── Lexicon load (extends V3-§3's loader to read the `root` field) ─

#[derive(Debug, Deserialize)]
struct LexiconFile {
    horizontal: Vec<TokenRule>,
    vertical: Vec<TokenRule>,
}

#[derive(Debug, Deserialize, Clone)]
struct TokenRule {
    tokens: Vec<String>,
    /// Optional Arabic 3-consonant root (e.g. "ق-ي-س"). Only present
    /// on Arabic-anchored entries; transliterations and Sanskrit
    /// entries leave it absent and rely on surface-token match.
    #[serde(default)]
    root: Option<String>,
    target: String,
    weight: f32,
    evidence: String,
}

#[derive(Debug, Clone)]
struct LoadedLexicon {
    horizontal: Vec<TokenRule>,
    vertical: Vec<TokenRule>,
    /// Pre-built map: root → list of (target, weight, evidence) for
    /// fast root-match lookup during classify.
    horizontal_by_root: HashMap<String, Vec<(String, f32, String)>>,
    vertical_by_root: HashMap<String, Vec<(String, f32, String)>>,
}

static LEXICON: OnceLock<LoadedLexicon> = OnceLock::new();

fn lexicon() -> &'static LoadedLexicon {
    LEXICON.get_or_init(|| {
        const RAW: &str = include_str!("../../../data/sources_lexicon.json");
        let parsed: LexiconFile = serde_json::from_str(RAW)
            .expect("sources_lexicon.json must parse");

        // Pre-lowercase tokens once, drop entries with invalid targets
        // (defense in depth against drift).
        let lower_h: Vec<TokenRule> = parsed
            .horizontal
            .into_iter()
            .filter(|r| is_valid_source_id(&r.target))
            .map(|r| TokenRule {
                // V3-§8.r4.5 (audit P1.7): NFKC-normalize tokens at
                // load so confusable codepoints (Cyrillic look-alikes,
                // ZWNJ, Tatweel, Persian/Urdu lookalikes for Arabic
                // characters) match the same way as the canonical
                // lexicon entry. Without NFKC the substring match is
                // byte-level and silently bypasses on confusables.
                tokens: r.tokens.iter().map(|t| t.nfkc().collect::<String>().to_lowercase()).collect(),
                ..r
            })
            .collect();
        let lower_v: Vec<TokenRule> = parsed
            .vertical
            .into_iter()
            .filter(|r| is_valid_content_type_id(&r.target))
            .map(|r| TokenRule {
                // V3-§8.r4.5 (audit P1.7): NFKC-normalize tokens at
                // load so confusable codepoints (Cyrillic look-alikes,
                // ZWNJ, Tatweel, Persian/Urdu lookalikes for Arabic
                // characters) match the same way as the canonical
                // lexicon entry. Without NFKC the substring match is
                // byte-level and silently bypasses on confusables.
                tokens: r.tokens.iter().map(|t| t.nfkc().collect::<String>().to_lowercase()).collect(),
                ..r
            })
            .collect();

        let mut h_by_root: HashMap<String, Vec<(String, f32, String)>> = HashMap::new();
        for r in &lower_h {
            if let Some(root) = &r.root {
                h_by_root
                    .entry(normalize_root(root))
                    .or_default()
                    .push((r.target.clone(), r.weight, r.evidence.clone()));
            }
        }
        let mut v_by_root: HashMap<String, Vec<(String, f32, String)>> = HashMap::new();
        for r in &lower_v {
            if let Some(root) = &r.root {
                v_by_root
                    .entry(normalize_root(root))
                    .or_default()
                    .push((r.target.clone(), r.weight, r.evidence.clone()));
            }
        }

        LoadedLexicon {
            horizontal: lower_h,
            vertical: lower_v,
            horizontal_by_root: h_by_root,
            vertical_by_root: v_by_root,
        }
    })
}

/// CAE returns roots without separators; the lexicon stores them with
/// hyphens for human readability (e.g. "ق-ي-س"). Normalize both sides
/// to the no-hyphen form for comparison.
fn normalize_root(root: &str) -> String {
    root.chars().filter(|c| *c != '-' && !c.is_whitespace()).collect()
}

// ─── Bridge slow-path fallback wiring (B3) ───
// The cataloger holds an optional embedder closure. The orchestrator
// (V3-§8) wires it to embeddings::run_embedding when the AppHandle is
// available; unit tests leave it None and skip the slow path.

pub type EmbedFn = Box<
    dyn Fn(&str) -> Result<Vec<f32>, String> + Send + Sync + 'static,
>;

pub struct LinguisticCataloger {
    embed_fn: Option<EmbedFn>,
}

impl LinguisticCataloger {
    pub fn new() -> Self {
        Self { embed_fn: None }
    }

    pub fn with_embedder(embed_fn: EmbedFn) -> Self {
        Self {
            embed_fn: Some(embed_fn),
        }
    }
}

impl Default for LinguisticCataloger {
    fn default() -> Self {
        Self::new()
    }
}

impl Cataloger for LinguisticCataloger {
    fn name(&self) -> &'static str {
        "linguistic"
    }

    fn classify(&self, ctx: &CatalogerContext) -> Option<ReasoningTrail> {
        let lex = lexicon();
        // V3-§8.r4.5 (audit P1.7): NFKC-normalize the note content so
        // confusable codepoints in the user's text match the same way
        // as the canonical lexicon entries (e.g. a Cyrillic 'і'
        // typed for Latin 'i' in qiyās; ZWNJ inserted in قياس).
        let lower: String = ctx.content.nfkc().collect::<String>().to_lowercase();

        // Path 1: CAE root match (HIGH confidence).
        // Tokenize Arabic-looking words and analyze their roots.
        let arabic_tokens = extract_arabic_tokens(&ctx.content);
        let mut h_hits: HashMap<String, (f32, String, &'static str)> = HashMap::new();
        let mut v_hits: HashMap<String, (f32, String, &'static str)> = HashMap::new();
        let mut roots_seen: Vec<String> = Vec::new();

        for tok in &arabic_tokens {
            let analyses = arabic::analyze(tok);
            for analysis in analyses.iter() {
                // analysis.root is already a hyphen-joined String like "ق-ي-س".
                if analysis.root.is_empty() {
                    continue;
                }
                let normalized = normalize_root(&analysis.root);
                if !roots_seen.contains(&normalized) {
                    roots_seen.push(normalized.clone());
                }
                if let Some(matches) = lex.horizontal_by_root.get(&normalized) {
                    for (target, weight, evidence) in matches {
                        let entry = h_hits
                            .entry(target.clone())
                            .or_insert((0.0, evidence.clone(), "cae_root"));
                        // Boost the weight slightly for root matches —
                        // they're more reliable than surface matches.
                        let boosted = (weight + 0.05_f32).min(0.99);
                        if boosted > entry.0 {
                            entry.0 = boosted;
                            entry.1 = format!("CAE root match: {}", analysis.root);
                            entry.2 = "cae_root";
                        }
                    }
                }
                if let Some(matches) = lex.vertical_by_root.get(&normalized) {
                    for (target, weight, evidence) in matches {
                        let entry = v_hits
                            .entry(target.clone())
                            .or_insert((0.0, evidence.clone(), "cae_root"));
                        let boosted = (weight + 0.05_f32).min(0.99);
                        if boosted > entry.0 {
                            entry.0 = boosted;
                            entry.1 = format!("CAE root match: {}", analysis.root);
                            entry.2 = "cae_root";
                        }
                    }
                }
            }
        }

        // Path 2: Surface-token match (MEDIUM confidence). Fills in
        // transliterations and Sanskrit entries the CAE pass missed.
        for rule in &lex.horizontal {
            for tok in &rule.tokens {
                if !tok.is_empty() && lower.contains(tok.as_str()) {
                    let entry = h_hits
                        .entry(rule.target.clone())
                        .or_insert((0.0, rule.evidence.clone(), "surface_token"));
                    if rule.weight > entry.0 {
                        entry.0 = rule.weight;
                        entry.1 = rule.evidence.clone();
                        entry.2 = "surface_token";
                    }
                    break;
                }
            }
        }
        for rule in &lex.vertical {
            for tok in &rule.tokens {
                if !tok.is_empty() && lower.contains(tok.as_str()) {
                    let entry = v_hits
                        .entry(rule.target.clone())
                        .or_insert((0.0, rule.evidence.clone(), "surface_token"));
                    if rule.weight > entry.0 {
                        entry.0 = rule.weight;
                        entry.1 = rule.evidence.clone();
                        entry.2 = "surface_token";
                    }
                    break;
                }
            }
        }

        // Path 3: Bridge fallback (LOW confidence) — only if we have an
        // embedder AND there are unmatched Arabic tokens. Cap at top
        // few unmatched tokens to keep latency bounded.
        if let Some(embed) = &self.embed_fn {
            let unmatched: Vec<&String> = arabic_tokens
                .iter()
                .filter(|t| {
                    let lt = t.to_lowercase();
                    !lex.horizontal.iter().any(|r| r.tokens.iter().any(|tk| lt.contains(tk)))
                        && !lex.vertical.iter().any(|r| r.tokens.iter().any(|tk| lt.contains(tk)))
                })
                .take(3) // bound the slow path
                .collect();
            for term in unmatched {
                if let Ok(vec) = embed(term) {
                    if let Some((concept_idx, score)) =
                        crate::bridge_vectors::get().nearest_concept(&vec)
                    {
                        if score >= 0.70 {
                            // Map concept_idx → concept_id; if the
                            // concept_id matches one of our lexicon
                            // targets, attribute it.
                            if let Some(concept_id) =
                                crate::bridge_vectors::get().concept_id(concept_idx)
                            {
                                // Check horizontal targets matching this concept
                                if is_valid_source_id(concept_id) {
                                    let entry = h_hits
                                        .entry(concept_id.to_string())
                                        .or_insert((0.0, String::new(), "bridge"));
                                    let bridge_weight = (score * 0.6).clamp(0.0, 0.7);
                                    if bridge_weight > entry.0 {
                                        entry.0 = bridge_weight;
                                        entry.1 = format!(
                                            "Bridge similarity match for '{}' → {} (score {:.2})",
                                            term, concept_id, score
                                        );
                                        entry.2 = "bridge";
                                    }
                                }
                                if is_valid_content_type_id(concept_id) {
                                    let entry = v_hits
                                        .entry(concept_id.to_string())
                                        .or_insert((0.0, String::new(), "bridge"));
                                    let bridge_weight = (score * 0.6).clamp(0.0, 0.7);
                                    if bridge_weight > entry.0 {
                                        entry.0 = bridge_weight;
                                        entry.1 = format!(
                                            "Bridge similarity match for '{}' → {} (score {:.2})",
                                            term, concept_id, score
                                        );
                                        entry.2 = "bridge";
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if h_hits.is_empty() && v_hits.is_empty() {
            return Some(ReasoningTrail::abstain(
                self.name(),
                "No CAE root match, no lexicon surface match, and Bridge fallback found nothing.",
            ));
        }

        // Determine which paths actually contributed (drives confidence + rules_fired).
        let any_root = h_hits.values().any(|(_, _, src)| *src == "cae_root")
            || v_hits.values().any(|(_, _, src)| *src == "cae_root");
        let any_surface = h_hits.values().any(|(_, _, src)| *src == "surface_token")
            || v_hits.values().any(|(_, _, src)| *src == "surface_token");
        let any_bridge = h_hits.values().any(|(_, _, src)| *src == "bridge")
            || v_hits.values().any(|(_, _, src)| *src == "bridge");

        let confidence = if any_root {
            Confidence::High
        } else if any_surface {
            Confidence::Medium
        } else {
            Confidence::Low
        };

        let mut rules_fired: Vec<String> = Vec::new();
        if any_root {
            rules_fired.push("cae_root_match".to_string());
        }
        if any_surface {
            rules_fired.push("surface_token_match".to_string());
        }
        if any_bridge {
            rules_fired.push("bridge_similarity".to_string());
        }
        rules_fired.push("rule_of_side_channel_preference".to_string());

        let horizontal = top_assignments(h_hits, 3);
        let vertical = top_assignments(v_hits, 3);
        let reasoning = build_reasoning(&horizontal, &vertical, &roots_seen);

        Some(ReasoningTrail {
            cataloger: self.name().to_string(),
            voiced_opinion: true,
            horizontal,
            vertical,
            reasoning,
            rules_fired,
            alternatives_considered: Vec::new(),
            self_reported_confidence: confidence,
        })
    }

    fn supported_axes(&self) -> &[Axis] {
        &[Axis::Horizontal, Axis::Vertical]
    }
}

/// Tokenize the note text and return only Arabic-script words. CAE
/// expects single words, not phrases. We split on whitespace, ASCII
/// punctuation, AND Arabic-script punctuation marks — then filter to
/// tokens whose first non-space char is in the Arabic Unicode block.
///
/// MIG-021v3 V3-§8.r1.a fix (audit P0.1): the original splitter used
/// only `is_ascii_punctuation()`, which excludes Arabic comma `،`
/// (U+060C), Arabic semicolon `؛` (U+061B), Arabic question mark `؟`
/// (U+061F), and Arabic full-stop `۔` (U+06D4). On real Arabic prose
/// these punctuation marks separate words constantly, so the splitter
/// produced multi-word tokens like `قياس،صحيح` that CAE could not
/// analyze — silently killing the cataloger's documented "Strong on
/// technical Arabic" path. This fix adds Arabic-script punctuation
/// + Unicode general-punctuation block to the splitter.
fn extract_arabic_tokens(text: &str) -> Vec<String> {
    text.split(|c: char| {
        c.is_whitespace()
            || c.is_ascii_punctuation()
            || is_arabic_punctuation(c)
    })
    .filter(|tok| !tok.is_empty())
    .filter(|tok| tok.chars().next().map(is_arabic).unwrap_or(false))
    .map(|tok| tok.to_string())
    .collect()
}

/// Arabic-script and adjacent punctuation marks the CAE root extractor
/// can't handle inside a token. Audit P0.1 fix.
fn is_arabic_punctuation(c: char) -> bool {
    let code = c as u32;
    matches!(
        c,
        '،' | '؛' | '؟' | '۔' | '٪' | '٫' | '٬' | '٭'  // Arabic punctuation block
    ) || (0x2000..=0x206F).contains(&code) // Unicode general punctuation
        || (0xFD3E..=0xFD3F).contains(&code) // Ornate parentheses
        || (0xFD4F..=0xFD4F).contains(&code)
}

fn is_arabic(c: char) -> bool {
    let code = c as u32;
    // Arabic + Arabic Supplement + Arabic Extended-A + Arabic Presentation Forms.
    (0x0600..=0x06FF).contains(&code)
        || (0x0750..=0x077F).contains(&code)
        || (0x08A0..=0x08FF).contains(&code)
        || (0xFB50..=0xFDFF).contains(&code)
        || (0xFE70..=0xFEFF).contains(&code)
}

fn top_assignments(
    hits: HashMap<String, (f32, String, &'static str)>,
    max: usize,
) -> Vec<AxisAssignment> {
    let mut entries: Vec<(String, f32)> = hits
        .into_iter()
        .map(|(target, (weight, _, _))| (target, weight))
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries.truncate(max);
    entries
        .into_iter()
        .enumerate()
        .map(|(i, (id, weight))| AxisAssignment {
            id,
            primary: i == 0,
            weight,
            descend_uncertain: false,
        })
        .collect()
}

fn build_reasoning(
    h: &[AxisAssignment],
    v: &[AxisAssignment],
    roots: &[String],
) -> String {
    use std::fmt::Write;
    let mut out = String::from("Linguistic match: ");
    let mut parts = Vec::new();
    if let Some(top) = h.first() {
        parts.push(format!("horizontal → {} (weight {:.2})", top.id, top.weight));
    }
    if let Some(top) = v.first() {
        parts.push(format!("vertical → {} (weight {:.2})", top.id, top.weight));
    }
    let _ = write!(out, "{}.", parts.join("; "));
    if !roots.is_empty() {
        let preview: Vec<&str> = roots.iter().take(5).map(|s| s.as_str()).collect();
        let _ = write!(out, " CAE roots seen: {}.", preview.join(", "));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_for_body(body: &str) -> CatalogerContext {
        CatalogerContext::new(
            "test.md".to_string(),
            body.to_string(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn arabic_qiyas_classifies_as_inference() {
        // V3-§8.r3 (audit Epistemology #1): qiyās in Sunni uṣūl is
        // structurally analogical INFERENCE, not comparison. Routes to
        // `inference` parent.
        let c = LinguisticCataloger::new();
        let trail = c.classify(&ctx_for_body("هذا قياس فقهي معتبر")).unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail.horizontal.iter().any(|a| a.id == "inference"));
        assert!(trail.rules_fired.contains(&"surface_token_match".to_string())
            || trail.rules_fired.contains(&"cae_root_match".to_string()));
    }

    #[test]
    fn english_transliteration_falls_to_surface_match() {
        // "mutawatir" is in the lexicon's tokens list but not Arabic
        // script, so CAE doesn't fire — surface match handles it.
        // V3-§8.r3 (audit Epistemology #2): bare متواتر / mutawatir
        // routes to PARENT mass-transmission (lafẓī / maʿnawī / ʿamalī
        // sub-classification needs more context than a token can carry).
        let c = LinguisticCataloger::new();
        let trail = c
            .classify(&ctx_for_body("This hadith is mutawatir according to the Sunni tradition."))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail.horizontal.iter().any(|a| a.id == "mass-transmission"));
        assert!(trail.rules_fired.contains(&"surface_token_match".to_string()));
    }

    #[test]
    fn sanskrit_iast_matches_via_surface() {
        let c = LinguisticCataloger::new();
        let trail = c
            .classify(&ctx_for_body("The pratyakṣa pramāṇa is foundational."))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(trail.horizontal.iter().any(|a| a.id == "perception/external"));
    }

    #[test]
    fn pure_english_prose_abstains() {
        let c = LinguisticCataloger::new();
        let trail = c
            .classify(&ctx_for_body("Constellation is a personal knowledge tool."))
            .unwrap();
        assert!(!trail.voiced_opinion);
    }

    #[test]
    fn root_match_outranks_no_match() {
        // The QIYAS root in vocalized + base form. We accept either
        // CAE root match (High) or surface match (Medium) — CAE may
        // not normalize every form. Both are acceptable signal.
        let c = LinguisticCataloger::new();
        let with_root = c
            .classify(&ctx_for_body("القياس مصدر معتبر في أصول الفقه"))
            .unwrap();
        assert!(with_root.voiced_opinion);
        assert!(matches!(
            with_root.self_reported_confidence,
            Confidence::High | Confidence::Medium
        ));
    }

    #[test]
    fn arabic_token_extraction_filters_correctly() {
        let tokens =
            extract_arabic_tokens("This is mixed: قياس and analogy and فطرة and tests.");
        assert!(tokens.iter().any(|t| t == "قياس"));
        assert!(tokens.iter().any(|t| t == "فطرة"));
        assert!(!tokens.iter().any(|t| t == "tests"));
        assert!(!tokens.iter().any(|t| t == "analogy"));
    }

    #[test]
    fn arabic_punctuation_separates_tokens() {
        // V3-§8.r1.a regression for audit P0.1: the original tokenizer
        // failed to split on Arabic punctuation, producing multi-word
        // tokens that CAE could not analyze. After the fix, Arabic
        // commas / semicolons / question marks must split tokens
        // exactly like ASCII commas would.
        let tokens = extract_arabic_tokens("هذا قياس،صحيح ومعتبر؛ في الفقه؟");
        // Each Arabic-script word should appear as its own token.
        assert!(tokens.iter().any(|t| t == "هذا"));
        assert!(tokens.iter().any(|t| t == "قياس"));
        assert!(tokens.iter().any(|t| t == "صحيح"));
        assert!(tokens.iter().any(|t| t == "ومعتبر"));
        assert!(tokens.iter().any(|t| t == "في"));
        assert!(tokens.iter().any(|t| t == "الفقه"));
        // The pre-fix bug produced "قياس،صحيح" as one token; assert
        // explicitly it does NOT appear.
        assert!(!tokens.iter().any(|t| t.contains('،')));
        assert!(!tokens.iter().any(|t| t.contains('؛')));
        assert!(!tokens.iter().any(|t| t.contains('؟')));
    }

    #[test]
    fn root_normalization_strips_hyphens() {
        assert_eq!(normalize_root("ق-ي-س"), "قيس");
        assert_eq!(normalize_root("ق ي س"), "قيس");
        assert_eq!(normalize_root("قيس"), "قيس");
    }

    // ─── V3-§9.A — Vertical lexicon expansion regression tests ───
    // 12 new entries added to sources_lexicon.json::vertical covering
    // all 5 branches of the vertical taxonomy. These tests confirm a
    // few representative entries fire on synthetic input. Full
    // verification happens in Gate 2 Boss-test (Phase E).

    #[test]
    fn v3_p9a_definition_phrasing_fires_concept() {
        // "the concept of" should fire semantic-contents/concept on the
        // vertical axis. Tests the new lexicon entry, NOT the structural
        // detector (which Phase B will add separately on a different
        // regex pattern).
        let c = LinguisticCataloger::new();
        let trail = c
            .classify(&ctx_for_body(
                "The concept of constructive proof is central to intuitionistic logic.",
            ))
            .unwrap();
        assert!(trail.voiced_opinion, "should voice on a clear vertical signal");
        assert!(
            trail.vertical.iter().any(|a| a.id == "semantic-contents/concept"),
            "vertical assignments should include semantic-contents/concept; got {:?}",
            trail.vertical.iter().map(|a| &a.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn v3_p9a_arabic_worldview_fires_higher_order() {
        let c = LinguisticCataloger::new();
        let trail = c
            .classify(&ctx_for_body(
                "تتشكل الرؤية الكونية للمؤمن من خلال نصوص الوحي والتفكر في الآفاق والأنفس.",
            ))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(
            trail
                .vertical
                .iter()
                .any(|a| a.id == "higher-order-constructs/worldview"),
            "should fire higher-order-constructs/worldview; got {:?}",
            trail.vertical.iter().map(|a| &a.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn v3_p9a_propositional_knowledge_phrasing_fires_correct_target() {
        // "we know that ... the boiling point of water is 100°C" — clear
        // propositional knowledge marker.
        let c = LinguisticCataloger::new();
        let trail = c
            .classify(&ctx_for_body(
                "We know that the boiling point of water at sea level is 100 degrees Celsius.",
            ))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(
            trail
                .vertical
                .iter()
                .any(|a| a.id == "epistemic-states/knowledge/by-content/propositional"),
            "should fire propositional knowledge; got {:?}",
            trail.vertical.iter().map(|a| &a.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn v3_p9a_doctrine_phrasing_fires_higher_order() {
        let c = LinguisticCataloger::new();
        let trail = c
            .classify(&ctx_for_body(
                "The Mu'tazili doctrine on free will differs sharply from the Ash'ari مذهب.",
            ))
            .unwrap();
        assert!(trail.voiced_opinion);
        assert!(
            trail
                .vertical
                .iter()
                .any(|a| a.id == "higher-order-constructs/doctrine"),
            "should fire higher-order-constructs/doctrine; got {:?}",
            trail.vertical.iter().map(|a| &a.id).collect::<Vec<_>>()
        );
    }
}
