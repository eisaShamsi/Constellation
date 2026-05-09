//! MIG-021v2 §1A' — Horizontal-axis taxonomy data.
//!
//! Source of truth: `docs/sources-of-knowledge-diagram.html` (Eisa-canonical,
//! interactive 3-level diagram). This file is the Rust mirror of that data
//! plus lookup helpers. The TypeScript mirror at
//! `src/lib/sources/horizontalTaxonomy.ts` ships the same shape for the
//! tree-picker frontend.
//!
//! Structure:
//!   - 1 implicit root (skipped — used only as conceptual container)
//!   - 11 parents (S1-S11) with tier metadata (Tier 1/2/3 acceptance)
//!   - 41 sub-leaves with traditional scholarly terms
//!   - 1 `unclassifiable` opt-out token (preserved from §1A; not in diagram)
//!   = 53 active horizontal IDs total
//!
//! ID scheme (per Plan §0 Q2):
//!   - Parents: kebab-case English label slug (perception / inference / …)
//!   - Leaves: parent + "/" + leaf slug (perception/external, inference/deductive)
//!   - The "/" separator is a YAML-safe character in scalar values
//!   - Existing 11 parent IDs are byte-identical to §1A's SOURCE_IDS — so
//!     legacy `sources:` data on disk still validates against this taxonomy

#[derive(Debug, Clone, Copy)]
pub struct HorizontalNode {
    /// Stable ID used in frontmatter and DB. kebab-case; "/" separates
    /// parent and leaf for sub-nodes.
    pub id: &'static str,
    /// English label (display).
    pub en: &'static str,
    /// Arabic label (display, RTL).
    pub ar: &'static str,
    /// Sanskrit/Pali transliteration where present in source diagram.
    pub tr: Option<&'static str>,
    /// Tier of acceptance: 1 = universally accepted, 2 = broadly accepted,
    /// 3 = school-specific or contested. 0 = leaf or special token (no tier).
    pub tier: u8,
    /// Parent's ID. None = top-level parent (one of S1-S11).
    pub parent_id: Option<&'static str>,
}

/// The full horizontal taxonomy as a flat list. Order is canonical
/// (matches the diagram's reading order, top-to-bottom).
pub const HORIZONTAL_NODES: &[HorizontalNode] = &[
    // ── S1 Perception (Tier 1) ──────────────────────────────────────
    HorizontalNode {
        id: "perception",
        en: "Perception / Sensation",
        ar: "الحِسّ",
        tr: Some("pratyakṣa"),
        tier: 1,
        parent_id: None,
    },
    HorizontalNode {
        id: "perception/external",
        en: "External perception",
        ar: "الإدراك الخارجي",
        tr: Some("bāhya pratyakṣa"),
        tier: 0,
        parent_id: Some("perception"),
    },
    HorizontalNode {
        id: "perception/internal",
        en: "Internal perception",
        ar: "الإدراك الباطني",
        tr: Some("mānasa pratyakṣa"),
        tier: 0,
        parent_id: Some("perception"),
    },
    HorizontalNode {
        id: "perception/self",
        en: "Self-perception",
        ar: "الشُّعور بالذات",
        tr: Some("svasaṃvedana"),
        tier: 0,
        parent_id: Some("perception"),
    },
    HorizontalNode {
        id: "perception/extraordinary",
        en: "Extraordinary perception",
        ar: "الإدراك الفائق",
        tr: Some("yogaja / mushāhadah"),
        tier: 0,
        parent_id: Some("perception"),
    },

    // ── S2 Inference (Tier 1) ───────────────────────────────────────
    HorizontalNode {
        id: "inference",
        en: "Inference / Reason",
        ar: "العَقل",
        tr: Some("anumāna"),
        tier: 1,
        parent_id: None,
    },
    HorizontalNode {
        id: "inference/deductive",
        en: "Deductive inference",
        ar: "الاستنباط البُرهاني",
        tr: None,
        tier: 0,
        parent_id: Some("inference"),
    },
    HorizontalNode {
        id: "inference/inductive",
        en: "Inductive inference",
        ar: "الاستقراء",
        tr: None,
        tier: 0,
        parent_id: Some("inference"),
    },
    HorizontalNode {
        id: "inference/abductive",
        en: "Abductive inference",
        ar: "الاستدلال الافتراضي",
        tr: None,
        tier: 0,
        parent_id: Some("inference"),
    },
    HorizontalNode {
        id: "inference/necessary",
        en: "Necessary reason",
        ar: "العَقل الضَّروري",
        tr: None,
        tier: 0,
        parent_id: Some("inference"),
    },
    HorizontalNode {
        id: "inference/speculative",
        en: "Speculative reason",
        ar: "العَقل النَّظَري",
        tr: None,
        tier: 0,
        parent_id: Some("inference"),
    },

    // ── S3 Testimony (Tier 2) ───────────────────────────────────────
    HorizontalNode {
        id: "testimony",
        en: "Testimony",
        ar: "الخَبَر",
        tr: Some("śabda"),
        tier: 2,
        parent_id: None,
    },
    HorizontalNode {
        id: "testimony/direct-witness",
        en: "Direct witness testimony",
        ar: "الشَّهادة المباشرة",
        tr: None,
        tier: 0,
        parent_id: Some("testimony"),
    },
    HorizontalNode {
        id: "testimony/reported",
        en: "Reported testimony",
        ar: "الخَبَر المنقول",
        tr: None,
        tier: 0,
        parent_id: Some("testimony"),
    },
    HorizontalNode {
        id: "testimony/authoritative",
        en: "Authoritative testimony",
        ar: "خَبَر الثِّقة",
        tr: Some("āpta-vacana"),
        tier: 0,
        parent_id: Some("testimony"),
    },
    HorizontalNode {
        id: "testimony/scriptural",
        en: "Scriptural testimony",
        ar: "النَّقل الشَّرعي",
        tr: None,
        tier: 0,
        parent_id: Some("testimony"),
    },

    // ── S4 Mass-transmission (Tier 3) ───────────────────────────────
    HorizontalNode {
        id: "mass-transmission",
        en: "Mass-transmission",
        ar: "التَّواتُر",
        tr: None,
        tier: 3,
        parent_id: None,
    },
    HorizontalNode {
        id: "mass-transmission/verbal",
        en: "Verbal mass-transmission",
        ar: "تَواتُر لَفظي",
        tr: None,
        tier: 0,
        parent_id: Some("mass-transmission"),
    },
    HorizontalNode {
        id: "mass-transmission/meaning",
        en: "Meaning mass-transmission",
        ar: "تَواتُر مَعنوي",
        tr: None,
        tier: 0,
        parent_id: Some("mass-transmission"),
    },
    HorizontalNode {
        id: "mass-transmission/practical",
        en: "Practical mass-transmission",
        ar: "تَواتُر عَمَلي",
        tr: None,
        tier: 0,
        parent_id: Some("mass-transmission"),
    },

    // ── S5 Comparison (Tier 2) ──────────────────────────────────────
    HorizontalNode {
        id: "comparison",
        en: "Comparison / Analogy",
        ar: "القياس",
        tr: Some("upamāna"),
        tier: 2,
        parent_id: None,
    },
    HorizontalNode {
        id: "comparison/ratio-legis",
        en: "Analogy by ratio legis",
        ar: "قياس العِلَّة",
        tr: None,
        tier: 0,
        parent_id: Some("comparison"),
    },
    HorizontalNode {
        id: "comparison/indication",
        en: "Analogy by indication",
        ar: "قياس الدِّلالة",
        tr: None,
        tier: 0,
        parent_id: Some("comparison"),
    },
    HorizontalNode {
        id: "comparison/resemblance",
        en: "Analogy by resemblance",
        ar: "قياس الشَّبَه",
        tr: None,
        tier: 0,
        parent_id: Some("comparison"),
    },
    HorizontalNode {
        id: "comparison/a-fortiori",
        en: "A fortiori analogy",
        ar: "قياس الأَوْلى",
        tr: None,
        tier: 0,
        parent_id: Some("comparison"),
    },

    // ── S6 Postulation / IBE (Tier 3) ───────────────────────────────
    HorizontalNode {
        id: "postulation",
        en: "Postulation / IBE",
        ar: "الاستنباط الافتراضي",
        tr: Some("arthāpatti"),
        tier: 3,
        parent_id: None,
    },
    HorizontalNode {
        id: "postulation/from-perceived",
        en: "From perceived fact",
        ar: "من مُعطى مُشاهَد",
        tr: Some("dṛṣṭārthāpatti"),
        tier: 0,
        parent_id: Some("postulation"),
    },
    HorizontalNode {
        id: "postulation/from-heard",
        en: "From heard fact",
        ar: "من خَبَر مَسموع",
        tr: Some("śrutārthāpatti"),
        tier: 0,
        parent_id: Some("postulation"),
    },
    HorizontalNode {
        id: "postulation/ibe",
        en: "Inference to best explanation",
        ar: "الاستدلال على أفضل تفسير",
        tr: None,
        tier: 0,
        parent_id: Some("postulation"),
    },

    // ── S7 Non-apprehension (Tier 3) ────────────────────────────────
    HorizontalNode {
        id: "non-apprehension",
        en: "Non-apprehension",
        ar: "عَدَم الإدراك",
        tr: Some("anupalabdhi"),
        tier: 3,
        parent_id: None,
    },
    HorizontalNode {
        id: "non-apprehension/prior",
        en: "Prior absence",
        ar: "العَدَم السابِق",
        tr: Some("prāgabhāva"),
        tier: 0,
        parent_id: Some("non-apprehension"),
    },
    HorizontalNode {
        id: "non-apprehension/posterior",
        en: "Posterior absence",
        ar: "العَدَم اللَّاحِق",
        tr: Some("pradhvaṃsābhāva"),
        tier: 0,
        parent_id: Some("non-apprehension"),
    },
    HorizontalNode {
        id: "non-apprehension/mutual",
        en: "Mutual absence",
        ar: "العَدَم التَّبادُلي",
        tr: Some("anyonyābhāva"),
        tier: 0,
        parent_id: Some("non-apprehension"),
    },
    HorizontalNode {
        id: "non-apprehension/absolute",
        en: "Absolute absence",
        ar: "العَدَم المُطلَق",
        tr: Some("atyantābhāva"),
        tier: 0,
        parent_id: Some("non-apprehension"),
    },

    // ── S8 Memory (Tier 2) ──────────────────────────────────────────
    HorizontalNode {
        id: "memory",
        en: "Memory",
        ar: "الذاكرة",
        tr: Some("smṛti"),
        tier: 2,
        parent_id: None,
    },
    HorizontalNode {
        id: "memory/recollection",
        en: "Recollection",
        ar: "التَّذَكُّر",
        tr: None,
        tier: 0,
        parent_id: Some("memory"),
    },
    HorizontalNode {
        id: "memory/recognition",
        en: "Recognition",
        ar: "التَّعَرُّف",
        tr: Some("pratyabhijñā"),
        tier: 0,
        parent_id: Some("memory"),
    },
    HorizontalNode {
        id: "memory/episodic",
        en: "Episodic memory",
        ar: "ذاكرة الأحداث",
        tr: None,
        tier: 0,
        parent_id: Some("memory"),
    },
    HorizontalNode {
        id: "memory/semantic",
        en: "Semantic memory",
        ar: "ذاكرة المعاني",
        tr: None,
        tier: 0,
        parent_id: Some("memory"),
    },

    // ── S9 Innate disposition (Tier 2) ──────────────────────────────
    HorizontalNode {
        id: "innate-disposition",
        en: "Innate disposition / Intuition",
        ar: "الفِطرة / الحَدْس",
        tr: None,
        tier: 2,
        parent_id: None,
    },
    HorizontalNode {
        id: "innate-disposition/primordial",
        en: "Primordial disposition (Sunni)",
        ar: "الفِطرة",
        tr: None,
        tier: 0,
        parent_id: Some("innate-disposition"),
    },
    HorizontalNode {
        id: "innate-disposition/first-principles",
        en: "Intuition of first principles",
        ar: "بَدَهيات العَقل",
        tr: None,
        tier: 0,
        parent_id: Some("innate-disposition"),
    },
    HorizontalNode {
        id: "innate-disposition/moral",
        en: "Innate moral knowledge",
        ar: "المعرفة الأخلاقية الفطرية",
        tr: Some("liángzhī"),
        tier: 0,
        parent_id: Some("innate-disposition"),
    },
    HorizontalNode {
        id: "innate-disposition/axioms",
        en: "Self-evident axioms",
        ar: "العُلوم الضَّرورية",
        tr: None,
        tier: 0,
        parent_id: Some("innate-disposition"),
    },

    // ── S10 Inspiration (Tier 3) ────────────────────────────────────
    HorizontalNode {
        id: "inspiration",
        en: "Inspiration / Mystical apprehension",
        ar: "الإلهام / الكَشْف",
        tr: None,
        tier: 3,
        parent_id: None,
    },
    HorizontalNode {
        id: "inspiration/ilham",
        en: "Ilhām (inspiration)",
        ar: "الإلهام",
        tr: None,
        tier: 0,
        parent_id: Some("inspiration"),
    },
    HorizontalNode {
        id: "inspiration/kashf",
        en: "Kashf (unveiling)",
        ar: "الكَشْف",
        tr: None,
        tier: 0,
        parent_id: Some("inspiration"),
    },
    HorizontalNode {
        id: "inspiration/dream-vision",
        en: "True dream-vision",
        ar: "الرُّؤيا الصَّادِقة",
        tr: None,
        tier: 0,
        parent_id: Some("inspiration"),
    },

    // ── S11 Revelation (Tier 3) ─────────────────────────────────────
    HorizontalNode {
        id: "revelation",
        en: "Revelation",
        ar: "الوحي",
        tr: None,
        tier: 3,
        parent_id: None,
    },
    HorizontalNode {
        id: "revelation/recited",
        en: "Recited revelation (Quran)",
        ar: "الوحي المتلوّ",
        tr: None,
        tier: 0,
        parent_id: Some("revelation"),
    },
    HorizontalNode {
        id: "revelation/non-recited",
        en: "Non-recited revelation (Sunnah)",
        ar: "الوحي غير المتلوّ",
        tr: None,
        tier: 0,
        parent_id: Some("revelation"),
    },
    HorizontalNode {
        id: "revelation/modes-of-receiving",
        en: "Modes of receiving revelation",
        ar: "أوجُه نزول الوحي",
        tr: None,
        tier: 0,
        parent_id: Some("revelation"),
    },

    // ── Opt-out token (preserved from §1A; not in source diagram) ───
    HorizontalNode {
        id: "unclassifiable",
        en: "Unclassifiable",
        ar: "غير قابل للتصنيف",
        tr: None,
        tier: 0,
        parent_id: None,
    },
];

// ─── Lookup helpers ────────────────────────────────────────────────

/// Returns true if `id` is a valid horizontal taxonomy node.
pub fn is_valid_id(id: &str) -> bool {
    HORIZONTAL_NODES.iter().any(|n| n.id == id)
}

/// All horizontal IDs (parents + leaves + opt-out token), in canonical order.
pub fn all_ids() -> Vec<&'static str> {
    HORIZONTAL_NODES.iter().map(|n| n.id).collect()
}

/// Tier metadata for a parent node. Returns None for leaves and the opt-out
/// token (they don't carry their own tier; their parent's tier applies for
/// classifier-fallback purposes — see `effective_tier`).
pub fn tier_for(id: &str) -> Option<u8> {
    HORIZONTAL_NODES
        .iter()
        .find(|n| n.id == id)
        .filter(|n| n.tier > 0)
        .map(|n| n.tier)
}

/// Tier that applies to this node when classifying — for leaves, returns
/// the parent's tier. For the opt-out token returns None. Used by the
/// tier-aware confidence fallback in §1B' (Plan §0 Q7).
pub fn effective_tier(id: &str) -> Option<u8> {
    if let Some(t) = tier_for(id) {
        return Some(t);
    }
    HORIZONTAL_NODES
        .iter()
        .find(|n| n.id == id)
        .and_then(|n| n.parent_id)
        .and_then(tier_for)
}

/// Parent ID, or None for top-level parents and the opt-out token.
pub fn parent_of(id: &str) -> Option<&'static str> {
    HORIZONTAL_NODES
        .iter()
        .find(|n| n.id == id)
        .and_then(|n| n.parent_id)
}

/// Children IDs of `parent_id`, in canonical order.
pub fn children_of(parent_id: &str) -> Vec<&'static str> {
    HORIZONTAL_NODES
        .iter()
        .filter(|n| n.parent_id == Some(parent_id))
        .map(|n| n.id)
        .collect()
}

/// English label for a node.
pub fn en_label(id: &str) -> Option<&'static str> {
    HORIZONTAL_NODES.iter().find(|n| n.id == id).map(|n| n.en)
}

/// Arabic label for a node.
pub fn ar_label(id: &str) -> Option<&'static str> {
    HORIZONTAL_NODES.iter().find(|n| n.id == id).map(|n| n.ar)
}

/// Sanskrit/Pali transliteration where present.
pub fn transliteration(id: &str) -> Option<&'static str> {
    HORIZONTAL_NODES
        .iter()
        .find(|n| n.id == id)
        .and_then(|n| n.tr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxonomy_has_53_nodes() {
        // 11 parents + 41 leaves + 1 opt-out = 53
        assert_eq!(HORIZONTAL_NODES.len(), 53);
    }

    #[test]
    fn eleven_parents() {
        let parents: Vec<_> = HORIZONTAL_NODES
            .iter()
            .filter(|n| n.parent_id.is_none() && n.id != "unclassifiable")
            .collect();
        assert_eq!(parents.len(), 11);
    }

    #[test]
    fn forty_one_leaves() {
        let leaves: Vec<_> = HORIZONTAL_NODES
            .iter()
            .filter(|n| n.parent_id.is_some())
            .collect();
        assert_eq!(leaves.len(), 41);
    }

    #[test]
    fn tier_distribution_matches_diagram() {
        let tier_1: Vec<_> = HORIZONTAL_NODES
            .iter()
            .filter(|n| n.tier == 1)
            .collect();
        let tier_2: Vec<_> = HORIZONTAL_NODES
            .iter()
            .filter(|n| n.tier == 2)
            .collect();
        let tier_3: Vec<_> = HORIZONTAL_NODES
            .iter()
            .filter(|n| n.tier == 3)
            .collect();
        assert_eq!(tier_1.len(), 2, "Tier 1: Perception, Inference");
        assert_eq!(tier_2.len(), 4, "Tier 2: Testimony, Comparison, Memory, Innate-disposition");
        assert_eq!(tier_3.len(), 5, "Tier 3: Mass-transmission, Postulation, Non-apprehension, Inspiration, Revelation");
    }

    #[test]
    fn legacy_parent_ids_preserved_for_backward_compat() {
        // The 11 parent IDs MUST byte-match §1A's SOURCE_IDS so legacy
        // `sources:` data on disk still validates.
        let legacy = [
            "perception", "inference", "testimony", "mass-transmission",
            "comparison", "postulation", "non-apprehension", "memory",
            "innate-disposition", "inspiration", "revelation", "unclassifiable",
        ];
        for id in legacy {
            assert!(is_valid_id(id), "legacy ID `{}` not in taxonomy", id);
        }
    }

    #[test]
    fn parent_links_resolve() {
        // Every leaf's parent_id must exist as a top-level node.
        for leaf in HORIZONTAL_NODES.iter().filter(|n| n.parent_id.is_some()) {
            let p = leaf.parent_id.unwrap();
            assert!(is_valid_id(p), "leaf `{}` has unknown parent `{}`", leaf.id, p);
        }
    }

    #[test]
    fn effective_tier_walks_up_to_parent() {
        // Leaves inherit their parent's tier for classifier fallback.
        assert_eq!(effective_tier("perception/external"), Some(1));
        assert_eq!(effective_tier("revelation/recited"), Some(3));
        assert_eq!(effective_tier("memory/recollection"), Some(2));
        // Parents return their own tier.
        assert_eq!(effective_tier("inference"), Some(1));
        // Opt-out token has no tier.
        assert_eq!(effective_tier("unclassifiable"), None);
    }

    #[test]
    fn children_of_eleven_parents_sums_to_41() {
        let parents = [
            "perception", "inference", "testimony", "mass-transmission",
            "comparison", "postulation", "non-apprehension", "memory",
            "innate-disposition", "inspiration", "revelation",
        ];
        let total: usize = parents.iter().map(|p| children_of(p).len()).sum();
        assert_eq!(total, 41);
    }

    #[test]
    fn no_duplicate_ids() {
        let mut seen = std::collections::HashSet::new();
        for n in HORIZONTAL_NODES {
            assert!(seen.insert(n.id), "duplicate ID: {}", n.id);
        }
    }
}
