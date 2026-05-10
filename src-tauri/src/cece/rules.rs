//! MIG-021v3 V3-§1 — Cataloger Rules registry.
//!
//! The five Architect §4 rules are encoded declaratively in
//! `data/cataloger_rules.json` and consulted by name from each cataloger.
//! No rule logic lives in this file — only the loader + lookup.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RuleId {
    Authority,
    Application,
    Three,
    SideChannel,
    AuthorityControl,
}

impl RuleId {
    pub fn as_str(self) -> &'static str {
        match self {
            RuleId::Authority => "authority",
            RuleId::Application => "application",
            RuleId::Three => "three",
            RuleId::SideChannel => "side_channel",
            RuleId::AuthorityControl => "authority_control",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleSpec {
    pub id: RuleId,
    pub name: String,
    pub description: String,
    /// Cataloger short names this rule applies to (empty = all).
    pub applies_to_catalogers: Vec<String>,
    /// 1.0 = highest priority. Used by synthesis to break ties when two
    /// rules of different priorities both fire.
    pub signal_priority: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct RulesFile {
    rules: Vec<RuleSpec>,
}

static RULES: OnceLock<Vec<RuleSpec>> = OnceLock::new();

fn rules_loaded() -> &'static [RuleSpec] {
    RULES
        .get_or_init(|| {
            const RAW: &str = include_str!("../../data/cataloger_rules.json");
            let parsed: RulesFile = serde_json::from_str(RAW)
                .expect("cataloger_rules.json must parse — fix the file or its schema");
            parsed.rules
        })
        .as_slice()
}

/// Look up a rule by its identifier. Panics if the rule isn't in the
/// declarative file (programmer error — the enum and the file must
/// stay in sync; the loader assertion catches this on first call).
pub fn rule(id: RuleId) -> &'static RuleSpec {
    rules_loaded()
        .iter()
        .find(|r| r.id == id)
        .expect("RuleId variant present in code but missing from cataloger_rules.json")
}

/// Returns true if the rule applies to a given cataloger.
pub fn rule_applies_to(id: RuleId, cataloger: &str) -> bool {
    let r = rule(id);
    r.applies_to_catalogers.is_empty() || r.applies_to_catalogers.iter().any(|c| c == cataloger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rule_variants_present_in_json() {
        // Force the loader to run + verify every enum variant has an entry.
        for id in [
            RuleId::Authority,
            RuleId::Application,
            RuleId::Three,
            RuleId::SideChannel,
            RuleId::AuthorityControl,
        ] {
            let r = rule(id);
            assert_eq!(r.id, id);
            assert!(!r.name.is_empty());
            assert!(!r.description.is_empty());
        }
    }

    #[test]
    fn applies_to_logic_works() {
        // Rule of Authority applies to user_authority and to all others
        // implicitly (they defer to it). Default JSON should have empty
        // applies_to (= all).
        assert!(rule_applies_to(RuleId::Authority, "linguistic"));
        assert!(rule_applies_to(RuleId::Authority, "user_authority"));
    }
}
