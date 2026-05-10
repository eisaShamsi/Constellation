//! MIG-021v2 §1G2' — Tier 1 deterministic rules engine.
//!
//! Runs before the embedding classifier (now Tier 2 logically) and
//! resolves ~30-40% of notes for free, in microseconds:
//!
//!   1. **Frontmatter precedence** — if `sources:` or `content_type:`
//!      is already set in the YAML (manually via PropertyEditor), echo
//!      that with confidence 1.0. The classifier never overrides
//!      explicit user intent.
//!
//!   2. **Bilingual lexicon match** — the file `data/sources_lexicon.json`
//!      ships paired EN/AR/Sanskrit terms drawn from the horizontal &
//!      vertical taxonomies. Each entry maps a list of tokens to a
//!      target taxonomy ID + confidence weight. Substring match,
//!      case-insensitive (Unicode-aware via lowercase normalization).
//!
//!   3. **Regex pattern match** — citation forms (ISBN, DOI, URL,
//!      blockquote markers) → testimony / scriptural sub-leaves.
//!
//! Each match contributes to a per-axis score map; the top entries
//! per axis are returned as suggestions. If no axis has any rule hit
//! above the floor threshold, returns empty for that axis and the
//! caller falls through to Tier 2 embeddings.
//!
//! The lexicon file is loaded once on first use and cached in a
//! `OnceLock`. Per Performance Rule 6, no heavy parsing on the hot
//! path — the rules engine itself is just substring scans + regex.

use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::sources::{is_valid_content_type_id, is_valid_source_id, Suggestion};

// ─── Lexicon data structures (mirror sources_lexicon.json) ─────────

#[derive(Debug, Deserialize)]
struct LexiconFile {
    horizontal: Vec<TokenRule>,
    vertical: Vec<TokenRule>,
    regex_horizontal: Vec<RegexRule>,
}

#[derive(Debug, Deserialize, Clone)]
struct TokenRule {
    tokens: Vec<String>,
    target: String,
    weight: f32,
    evidence: String,
}

#[derive(Debug, Deserialize)]
struct RegexRuleRaw {
    pattern: String,
    target: String,
    weight: f32,
    evidence: String,
}

#[derive(Debug, Deserialize)]
#[serde(from = "RegexRuleRaw")]
struct RegexRule {
    pattern: Regex,
    target: String,
    weight: f32,
    evidence: String,
}

impl From<RegexRuleRaw> for RegexRule {
    fn from(raw: RegexRuleRaw) -> Self {
        // Compile-once: invalid patterns fall back to a never-match
        // regex so a malformed lexicon entry doesn't crash the loader.
        let pattern = Regex::new(&raw.pattern).unwrap_or_else(|_| Regex::new("$.^").unwrap());
        Self {
            pattern,
            target: raw.target,
            weight: raw.weight,
            evidence: raw.evidence,
        }
    }
}

#[derive(Debug)]
struct LoadedLexicon {
    horizontal_tokens: Vec<TokenRule>, // pre-lowercased tokens
    vertical_tokens: Vec<TokenRule>,
    regex_horizontal: Vec<RegexRule>,
}

static LEXICON: OnceLock<LoadedLexicon> = OnceLock::new();

fn lexicon() -> &'static LoadedLexicon {
    LEXICON.get_or_init(|| {
        // Bundled at compile time via include_str! so we don't depend
        // on the file being present at runtime.
        const RAW: &str = include_str!("../../data/sources_lexicon.json");
        let parsed: LexiconFile = serde_json::from_str(RAW)
            .expect("sources_lexicon.json must parse — fix the file or its schema");

        // Pre-lowercase tokens once + drop entries with invalid targets
        // (defense in depth against drift between lexicon & taxonomy).
        let lower_tokens = |rule: TokenRule| TokenRule {
            tokens: rule.tokens.iter().map(|t| t.to_lowercase()).collect(),
            ..rule
        };

        let horizontal_tokens = parsed
            .horizontal
            .into_iter()
            .filter(|r| is_valid_source_id(&r.target))
            .map(lower_tokens)
            .collect();
        let vertical_tokens = parsed
            .vertical
            .into_iter()
            .filter(|r| is_valid_content_type_id(&r.target))
            .map(lower_tokens)
            .collect();
        let regex_horizontal = parsed
            .regex_horizontal
            .into_iter()
            .filter(|r| is_valid_source_id(&r.target))
            .collect();

        LoadedLexicon {
            horizontal_tokens,
            vertical_tokens,
            regex_horizontal,
        }
    })
}

// ─── Public API ────────────────────────────────────────────────────

/// Outcome of Tier 1 classification. `tier_used` is 1 if any rule
/// produced suggestions; the caller falls through to Tier 2 only when
/// BOTH axes returned empty.
#[derive(Debug, Clone)]
pub struct Tier1Result {
    pub horizontal: Vec<Suggestion>,
    pub vertical: Vec<Suggestion>,
    /// True if the result came from frontmatter precedence (confidence 1.0,
    /// from explicit user setting). The frontend renders a different badge.
    pub from_frontmatter: bool,
}

impl Tier1Result {
    pub fn is_empty(&self) -> bool {
        self.horizontal.is_empty() && self.vertical.is_empty()
    }
}

/// Run Tier 1 against a note's content + already-extracted frontmatter.
///
/// `frontmatter_sources` and `frontmatter_content_type` are the values
/// already in the YAML (as extracted by `sources::extract_sources` /
/// `extract_content_type`). When non-empty, they take absolute precedence:
/// the result echoes them at confidence 1.0 and the lexicon/regex passes
/// are skipped for that axis.
pub fn classify_tier1(
    content: &str,
    frontmatter_sources: &[String],
    frontmatter_content_type: &[String],
) -> Tier1Result {
    let mut result = Tier1Result {
        horizontal: Vec::new(),
        vertical: Vec::new(),
        from_frontmatter: false,
    };

    // ── 1. Frontmatter precedence ──
    if !frontmatter_sources.is_empty() {
        for id in frontmatter_sources {
            if is_valid_source_id(id) {
                result.horizontal.push(Suggestion {
                    source: id.clone(),
                    confidence: 1.0,
                    evidence: "Set in frontmatter (manual)".to_string(),
                    axis: "horizontal".to_string(),
                });
            }
        }
        result.from_frontmatter = true;
    }
    if !frontmatter_content_type.is_empty() {
        for id in frontmatter_content_type {
            if is_valid_content_type_id(id) {
                result.vertical.push(Suggestion {
                    source: id.clone(),
                    confidence: 1.0,
                    evidence: "Set in frontmatter (manual)".to_string(),
                    axis: "vertical".to_string(),
                });
            }
        }
        result.from_frontmatter = true;
    }

    let lex = lexicon();
    let lower = content.to_lowercase();

    // ── 2. Horizontal lexicon + regex (skip if frontmatter set it) ──
    if result.horizontal.is_empty() {
        let mut hits: HashMap<String, (f32, String)> = HashMap::new();
        for rule in &lex.horizontal_tokens {
            for tok in &rule.tokens {
                if !tok.is_empty() && lower.contains(tok.as_str()) {
                    let entry = hits
                        .entry(rule.target.clone())
                        .or_insert((0.0, rule.evidence.clone()));
                    if rule.weight > entry.0 {
                        entry.0 = rule.weight;
                    }
                    break;
                }
            }
        }
        for rule in &lex.regex_horizontal {
            if rule.pattern.is_match(content) {
                let entry = hits
                    .entry(rule.target.clone())
                    .or_insert((0.0, rule.evidence.clone()));
                if rule.weight > entry.0 {
                    entry.0 = rule.weight;
                }
            }
        }
        result.horizontal = top_suggestions(hits, "horizontal", 3);
    }

    // ── 3. Vertical lexicon (skip if frontmatter set it) ──
    if result.vertical.is_empty() {
        let mut hits: HashMap<String, (f32, String)> = HashMap::new();
        for rule in &lex.vertical_tokens {
            for tok in &rule.tokens {
                if !tok.is_empty() && lower.contains(tok.as_str()) {
                    let entry = hits
                        .entry(rule.target.clone())
                        .or_insert((0.0, rule.evidence.clone()));
                    if rule.weight > entry.0 {
                        entry.0 = rule.weight;
                    }
                    break;
                }
            }
        }
        result.vertical = top_suggestions(hits, "vertical", 3);
    }

    result
}

fn top_suggestions(
    hits: HashMap<String, (f32, String)>,
    axis: &str,
    max: usize,
) -> Vec<Suggestion> {
    let mut entries: Vec<(String, f32, String)> = hits
        .into_iter()
        .map(|(target, (w, ev))| (target, w, ev))
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    entries.truncate(max);
    entries
        .into_iter()
        .map(|(target, weight, evidence)| Suggestion {
            source: target,
            confidence: weight,
            evidence,
            axis: axis.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_precedence_short_circuits() {
        let r = classify_tier1(
            "any body text",
            &["testimony".to_string()],
            &["epistemic-states/doubt".to_string()],
        );
        assert!(r.from_frontmatter);
        assert_eq!(r.horizontal.len(), 1);
        assert_eq!(r.horizontal[0].source, "testimony");
        assert_eq!(r.horizontal[0].confidence, 1.0);
        assert_eq!(r.vertical.len(), 1);
        assert_eq!(r.vertical[0].source, "epistemic-states/doubt");
    }

    #[test]
    fn arabic_tawatur_lexicon_hit() {
        let r = classify_tier1("الحديث متواتر عند جميع الأمة", &[], &[]);
        assert!(!r.from_frontmatter);
        assert!(r.horizontal.iter().any(|s| s.source == "mass-transmission/verbal"));
    }

    #[test]
    fn english_doubt_marker_hits_vertical() {
        let r = classify_tier1("I doubt that the moon landing happened in 1969.", &[], &[]);
        assert!(r.vertical.iter().any(|s| s.source == "epistemic-states/doubt"));
    }

    #[test]
    fn empty_returns_empty() {
        let r = classify_tier1("Constellation is a personal knowledge system.", &[], &[]);
        // No keywords match; expect empty so the caller can fall through to Tier 2.
        assert!(r.is_empty());
    }

    #[test]
    fn isbn_regex_hits_testimony_scriptural() {
        let r = classify_tier1("See ISBN 978-0-12-345678-9 for more.", &[], &[]);
        assert!(r.horizontal.iter().any(|s| s.source == "testimony/scriptural"));
    }
}
