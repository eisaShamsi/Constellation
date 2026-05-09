//! MIG-021v2 §1B' — Classifier candidate definitions, expanded to cover the
//! full two-axis taxonomy.
//!
//! Pre-existing `SOURCE_DEFINITIONS` (11 horizontal parents, ~150 words each,
//! battle-tested in §1B) is preserved and reused. New material added in v2:
//!
//! 1. `HORIZONTAL_LEAF_HINTS` — 41 short scholarly hints, one per sub-leaf
//!    from `docs/sources-of-knowledge-diagram.html`. Concatenated with the
//!    parent's full definition at runtime to produce the leaf's embedding text.
//!
//! 2. `build_classifier_candidates()` — runtime builder that walks both
//!    taxonomies and produces the unified `ClassifierCandidate` list (53
//!    horizontal + 222 vertical = ~275 entries). For vertical nodes (where
//!    the source chart provides only labels), the embedding text is built
//!    mechanically from `[en_label] ([ar_label] [tr]) — Branch X: [branch_name].
//!    Parent: [parent_label].` Per Plan §3 risk mitigation: no fabrication
//!    of philosophical content where the chart provides only a label.
//!
//! Embedding cost: ~275 definitions × one-time embedding (~10 sec on Eisa's
//! machine), cached in `tier1_embedding::HORIZONTAL_VECTORS` +
//! `VERTICAL_VECTORS`. Per-classification cost is unchanged.

use crate::sources::{horizontal_taxonomy, vertical_taxonomy};

/// One candidate the classifier embeds and ranks against. The `axis` field
/// distinguishes horizontal (sources) from vertical (content_type) so the
/// classifier can return parallel suggestion sets per axis.
#[derive(Debug, Clone)]
pub struct ClassifierCandidate {
    pub id: String,
    pub axis: ClassifierAxis,
    pub embedding_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassifierAxis {
    Horizontal,
    Vertical,
}

impl ClassifierAxis {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClassifierAxis::Horizontal => "horizontal",
            ClassifierAxis::Vertical => "vertical",
        }
    }
}

// ─── Horizontal parent definitions (11 — preserved from §1B) ──────────

/// Rich (~150-word) definitions for the 11 horizontal parents. Embedded
/// at app startup; one of the strongest-distinguishing inputs to the
/// embedding-similarity classifier. Order matches `classifiable_sources()`.
pub const SOURCE_DEFINITIONS: &[(&str, &str); 11] = &[
    (
        "perception",
        "Direct sensory contact with an object. Knowledge from the senses — what you saw, heard, touched, smelled, tasted, felt. First-hand observation of phenomena in the present moment. Empirical data gathered through your own sensory apparatus. Not deduced, not reported, not remembered — perceived directly. Examples: 'I saw the building collapse.' 'The water tasted salty.' 'I observed three crows on the wire at 7 AM.' 'The temperature reading was 23°C.' Includes structured observation: experimental data, fieldwork notes, measurements, direct visual or acoustic recording. Excludes inferences drawn from observations (those are inference); recollections of past observations (those are memory); descriptions of things seen by others (those are testimony). The Greek aisthēsis, Sunni Islamic al-hiss, Indian pratyaksa, Mohist qin zhi all name this same epistemic source.",
    ),
    (
        "inference",
        "Knowledge derived through reasoning from premises to conclusions. The note presents an argument, deduces, calculates, reasons logically from given facts to new claims. Mathematical proofs, syllogistic reasoning, statistical inference, causal analysis. Phrases like 'therefore,' 'it follows that,' 'consequently,' 'because of this,' 'we can conclude,' 'the implication is.' Includes both deductive (necessary conclusions) and inductive (probable conclusions) reasoning. Differs from perception (no direct sensory observation), from testimony (the conclusion is your own derivation, not a quoted source), from comparison (no analogical step), from postulation (not specifically inferring an unobserved fact to explain an observed one). The Sunni Islamic al-aql, Indian anumana, Mohist shuo zhi name this source. Logic, mathematics, theoretical analysis, deductive reasoning chains.",
    ),
    (
        "testimony",
        "Knowledge transmitted via reliable verbal report from another knower. Quoted statements, citations, references to what someone else said or wrote. The note's primary content is what someone else observed, claimed, or reported. Phrases like 'according to,' 'as X said,' 'the report states,' 'as documented by,' followed by a name or source. Citations to books, papers, individuals, or institutions. Single-source testimony — one named witness or document. Differs from mass-transmission (testimony from one source, not a convergent multi-witness chain). Includes formal academic citations, direct quotations, paraphrased reports, news reports, expert opinions, and 'I was told that' style notes. The Sunni Islamic al-khabar al-sadiq, Indian sabda, Mohist wen zhi all name this source. Documentary research, journalism, oral history, citation-based scholarship.",
    ),
    (
        "mass-transmission",
        "Knowledge from convergent reports by independent witnesses too numerous to collude on falsehood. The Sunni Islamic usul al-fiqh tradition's distinctive epistemic source — al-tawatur. The note cites multiple independent sources that all confirm the same fact, not because one quoted the other, but because each independently observed or reported. Used for facts established by overwhelming witness consensus: widely-attested historical events, mutawatir hadith chains, well-documented physical phenomena observed by many. Phrases like 'all sources agree,' 'uniformly attested,' 'convergent testimony,' 'established beyond doubt by multiple independent witnesses,' 'widely reported by independent observers.' Differs from testimony (single source); differs from inference (the conclusion isn't derived, it's directly attested by many independent witnesses). Yields necessary, certain knowledge in classical Sunni epistemology.",
    ),
    (
        "comparison",
        "Knowledge gained by analogy or similarity to a known object. The note's central move is 'X is like Y' — extending knowledge of a known thing to an unknown thing via structural similarity. Examples: legal qiyas (extending a ruling from a known case to a new analogous one); scientific reasoning by analogy ('the atom is like a tiny solar system'); pedagogical comparisons; metaphorical understanding. Phrases like 'similar to,' 'analogous to,' 'like X, this Y also,' 'by parallel reasoning,' 'the comparison suggests,' 'corresponds to.' Differs from inference (no logical derivation step); differs from postulation (no need to explain an observed fact); the validity rests on the similarity itself. The Sunni Islamic al-qiyas, Indian upamana name this source. Comparative law, comparative philology, structural parallels.",
    ),
    (
        "postulation",
        "Inference to the best explanation. The note posits an unobserved fact required to explain something observed. 'X must be true, because otherwise we cannot explain the observed Y.' Scientific hypothesis-formation, abductive reasoning, theoretical entities posited to make sense of data. Phrases like 'must be the case that,' 'the only way to explain this is,' 'this requires that,' 'we must assume,' 'the simplest explanation is,' 'the best explanation is,' 'this implies an underlying.' Differs from inference (which derives from given premises); postulation derives a NEW premise to make existing observations make sense. Common in scientific theorizing, detective reasoning, philosophical speculation, theoretical physics. The Indian arthapatti names this source. Hidden mechanisms, latent causes, theoretical postulates.",
    ),
    (
        "non-apprehension",
        "Knowledge of absence. The absence of perception itself yields knowledge that something is not there. Distinctive Indian Bhatta Mimamsa and Advaita Vedanta source — anupalabdhi. The note's claim is about something NOT existing or NOT happening. 'There is no jar in this room because I see no jar.' 'No evidence has been found of X.' 'The records show no entry for Y.' Negative observations, missing data, gaps documented as significant findings. Phrases like 'no evidence of,' 'no record of,' 'absent from,' 'no sign of,' 'fails to appear,' 'is missing,' 'we found nothing of,' 'silence in the records.' Differs from inference (the absence is directly perceived as significant, not deduced from premises). Used in archaeology, audit work, evidence law, archival research.",
    ),
    (
        "memory",
        "Recall of previously cognized content. The note's content is something the writer remembers from past experience — events, conversations, learnings, observations from earlier. Distinct from perception (which is present-tense observation); memory carries the past forward. Phrases like 'I remember when,' 'as I recall,' 'from my earlier experience,' 'back in [date],' 'I recall reading,' 'I learned earlier that,' 'in my memory,' 'I once saw,' 'years ago.' Includes biographical recollections, historical recall, recalled lessons, recalled observations, recalled conversations. Reliability of memory itself is a known epistemic concern; the note may flag uncertainty about the recollection. The Indian smrti names this source. Personal recollections, oral histories, recalled facts.",
    ),
    (
        "innate-disposition",
        "Pre-experiential cognitive endowment yielding direct knowledge of certain truths. Sunni Islamic fitrah, Greek nous as intuition of first principles, Confucian liangzhi, Mencian moral sprouts. The note expresses something the writer takes as known by the human mind without instruction or external evidence: basic moral intuitions ('causing unnecessary suffering is wrong'), elementary logical truths ('a thing cannot be both A and not-A'), aesthetic recognitions, basic mathematical intuitions, universal human responses. Phrases like 'self-evident,' 'obviously,' 'any human knows that,' 'the mind naturally recognizes,' 'intuitively,' 'as a matter of basic moral sense.' Differs from inference (no derivation steps); differs from perception (no sensory contact). Universal pre-experiential knowing — what every healthy human mind already knows.",
    ),
    (
        "inspiration",
        "Non-discursive apprehension claimed in mystical and spiritual traditions. The note records a flash of insight, a vision, a dream-content, an unbidden realization, or a creative breakthrough that did not come from systematic reasoning or external sources. Sufi-influenced epistemology recognizes al-ilham as a contested source of personal knowing. Phrases like 'it came to me,' 'I suddenly realized,' 'in a moment of insight,' 'the vision showed,' 'I dreamt that,' 'creative breakthrough,' 'epiphany,' 'spiritual insight,' 'mystical experience.' The contested status across traditions: mainstream Sunni kalam and analytic philosophy reject it as a public source; the user's adoption is personal. Differs from innate-disposition (which is universal pre-experiential) — inspiration is a particular event in a particular mind. Artistic creation, mystical experience, sudden insight.",
    ),
    (
        "revelation",
        "Communication from a divine source, transmitted through prophets, scripture, or recognized religious channels. The note quotes or interprets sacred text (Qur'an, Bible, Vedas, Talmud, Sunnah), or records knowledge attributed to prophetic transmission, or cites established religious doctrine derived from such sources. Phrases like 'as the Qur'an says,' 'scripture teaches,' 'the prophet said,' 'divine command,' 'revealed truth,' 'as prescribed in [sacred text],' 'sacred teaching,' 'God commanded,' 'in the holy book.' A primary epistemic source in Sunni Islam, Judaism, Christianity, Hindu Mimamsa, and other religious traditions; rejected by Carvaka and most secular epistemology. Distinguishable from testimony in that the chain ultimately points to a divine origin, not merely a human reporter. Religious teachings, scriptural exegesis, prophetic narrations.",
    ),
];

// ─── Horizontal leaf hints (41 — short, scholarly) ───────────────────

/// Short hints (one sentence each) for the 41 horizontal sub-leaves.
/// Concatenated with the parent's full definition at runtime to produce
/// the leaf's embedding text. Drawn from the diagram's tri-script labels
/// + standard scholarly literature; no fabrication beyond what the labels
/// themselves already commit to.
pub const HORIZONTAL_LEAF_HINTS: &[(&str, &str)] = &[
    // S1 Perception
    ("perception/external", "Specifically external sensory perception of objects in the world (Indian bāhya pratyakṣa)."),
    ("perception/internal", "Specifically internal mental perception of one's own thoughts and feelings (mānasa pratyakṣa)."),
    ("perception/self", "Specifically reflexive self-awareness — knowledge of one's own knowing (svasaṃvedana / al-shu'ūr bi-l-dhāt)."),
    ("perception/extraordinary", "Specifically extraordinary perception — yogic or contemplative sensing beyond normal sensory range (yogaja / mushāhadah)."),

    // S2 Inference
    ("inference/deductive", "Specifically deductive inference — necessary conclusion drawn from given premises through formal logic (al-istinbāṭ al-burhānī)."),
    ("inference/inductive", "Specifically inductive inference — probable generalization derived from observed particulars (al-istiqrā')."),
    ("inference/abductive", "Specifically abductive inference — conjectural reasoning from effect to most-plausible cause."),
    ("inference/necessary", "Specifically necessary reason — a priori rational knowledge that the mind grasps as logically inescapable (al-ʿaql al-ḍarūrī)."),
    ("inference/speculative", "Specifically speculative reason — discursive reasoning toward conclusions not yet certain (al-ʿaql al-naẓarī)."),

    // S3 Testimony
    ("testimony/direct-witness", "Specifically direct first-person witness testimony from someone who was present at the event (al-shahādah al-mubāsharah)."),
    ("testimony/reported", "Specifically secondary reported testimony — witnessed by one party and relayed through a chain (al-khabar al-manqūl)."),
    ("testimony/authoritative", "Specifically testimony from a recognized reliable authority — āpta-vacana / khabar al-thiqah."),
    ("testimony/scriptural", "Specifically scripturally-grounded testimony — citation of religious sources as testimonial authority (al-naql al-shar'ī)."),

    // S4 Mass-transmission
    ("mass-transmission/verbal", "Specifically verbal mass-transmission — the transmitted text or wording is itself attested by overwhelming witnesses (tawātur lafẓī)."),
    ("mass-transmission/meaning", "Specifically meaning-based mass-transmission — the substance is uniformly attested even if wording varies (tawātur ma'nawī)."),
    ("mass-transmission/practical", "Specifically practical mass-transmission — a continuous community practice that serves as evidence (tawātur 'amalī)."),

    // S5 Comparison / Analogy
    ("comparison/ratio-legis", "Specifically analogy by shared underlying cause — the classical fiqh qiyās al-'illah."),
    ("comparison/indication", "Specifically analogy by indicative similarity — qiyās al-dilālah."),
    ("comparison/resemblance", "Specifically analogy by surface resemblance — qiyās al-shabah."),
    ("comparison/a-fortiori", "Specifically a fortiori analogy — qiyās al-awlā, where the new case is even more deserving of the ruling than the precedent."),

    // S6 Postulation / IBE
    ("postulation/from-perceived", "Specifically postulation from a perceived fact — positing an unobserved cause to explain something seen (dṛṣṭārthāpatti)."),
    ("postulation/from-heard", "Specifically postulation from a heard report — positing an unobserved fact to make sense of testimony (śrutārthāpatti)."),
    ("postulation/ibe", "Specifically inference to the best explanation — selecting among competing hypotheses the one that best accounts for the data."),

    // S7 Non-apprehension
    ("non-apprehension/prior", "Specifically prior absence — the non-existence of something before its production (prāgabhāva)."),
    ("non-apprehension/posterior", "Specifically posterior absence — the non-existence of something after its destruction (pradhvaṃsābhāva)."),
    ("non-apprehension/mutual", "Specifically mutual absence — two distinct things being not-the-other (anyonyābhāva)."),
    ("non-apprehension/absolute", "Specifically absolute absence — the unqualified non-existence of a thing in a locus across all time (atyantābhāva)."),

    // S8 Memory
    ("memory/recollection", "Specifically active recollection — the deliberate calling-to-mind of past content (al-tadhakkur)."),
    ("memory/recognition", "Specifically recognition — identifying a present object as one previously encountered (pratyabhijñā)."),
    ("memory/episodic", "Specifically episodic memory — recall of specific dated events from one's own life."),
    ("memory/semantic", "Specifically semantic memory — recall of general facts and meanings, undated and decontextualized."),

    // S9 Innate disposition
    ("innate-disposition/primordial", "Specifically primordial disposition — the Sunni fiṭrah, the human innate orientation toward truth and the divine."),
    ("innate-disposition/first-principles", "Specifically intuition of first principles — self-evident axioms the mind grasps without proof (badahiyāt al-ʿaql)."),
    ("innate-disposition/moral", "Specifically innate moral knowledge — Mencian liángzhī, the inborn sense of right and wrong."),
    ("innate-disposition/axioms", "Specifically self-evident axioms — necessary truths whose denial is incoherent (al-ʿulūm al-ḍarūriyyah)."),

    // S10 Inspiration
    ("inspiration/ilham", "Specifically ilhām — direct God-cast inspiration into the heart of a sincere seeker."),
    ("inspiration/kashf", "Specifically kashf — mystical unveiling, a sudden vision of unseen reality."),
    ("inspiration/dream-vision", "Specifically true dream-vision — al-ru'yā al-ṣādiqah, a veridical dream considered a fragment of prophecy."),

    // S11 Revelation
    ("revelation/recited", "Specifically recited revelation — the Qur'an itself, transmitted as both meaning and exact wording (al-waḥy al-matluww)."),
    ("revelation/non-recited", "Specifically non-recited revelation — the Sunnah, prophetic teachings transmitted as meaning rather than exact divine speech (al-waḥy ghayr al-matluww)."),
    ("revelation/modes-of-receiving", "Specifically the modes of receiving revelation — true dream, audible voice, angelic mediation, etc. (awjuh nuzūl al-waḥy)."),
];

/// Lookup helper for a leaf hint.
fn leaf_hint(id: &str) -> Option<&'static str> {
    HORIZONTAL_LEAF_HINTS
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| *v)
}

/// Lookup helper for a parent's full definition.
fn parent_definition(id: &str) -> Option<&'static str> {
    SOURCE_DEFINITIONS
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| *v)
}

// ─── Runtime candidate builder ──────────────────────────────────────

/// Build the unified classifier candidate list. Called once at first
/// classifier invocation; results cached by the caller.
///
/// Composition:
/// - 11 horizontal parents → use rich SOURCE_DEFINITIONS verbatim
/// - 41 horizontal leaves → "{parent_def} Specifically: {leaf_hint}"
/// - 222 vertical nodes → mechanical "{en_label} ({ar_label}{ tr_suffix}). Branch X — {branch_label} taxonomy. Parent: {parent_label}."
///   (Per Plan §3 risk mitigation: no fabrication of philosophical content
///   where the source diagram provides only a label. The label + Arabic +
///   transliteration + parent-label give the embedding model enough signal
///   to differentiate at the BRANCH level reliably; LEAF-level accuracy is
///   bonus precision when confidence is high. Future PJ may enrich.)
///
/// Excludes the `unclassifiable` opt-out token (classifier never suggests it)
/// and the `epistemic-content` root (not a meaningful classification target).
pub fn build_classifier_candidates() -> Vec<ClassifierCandidate> {
    let mut out: Vec<ClassifierCandidate> =
        Vec::with_capacity(53 + 222);

    // Horizontal axis
    for node in horizontal_taxonomy::HORIZONTAL_NODES {
        if node.id == "unclassifiable" {
            continue; // never suggested
        }
        if node.parent_id.is_none() {
            // Top-level parent — use rich definition
            if let Some(def) = parent_definition(node.id) {
                out.push(ClassifierCandidate {
                    id: node.id.to_string(),
                    axis: ClassifierAxis::Horizontal,
                    embedding_text: def.to_string(),
                });
            }
        } else {
            // Leaf — combine parent's full definition + leaf hint
            let parent = node.parent_id.unwrap();
            let parent_def = parent_definition(parent).unwrap_or("");
            let hint = leaf_hint(node.id).unwrap_or("");
            let combined = format!("{} {}", parent_def, hint);
            out.push(ClassifierCandidate {
                id: node.id.to_string(),
                axis: ClassifierAxis::Horizontal,
                embedding_text: combined,
            });
        }
    }

    // Vertical axis — mechanical embedding text from labels + parent context
    for node in vertical_taxonomy::VERTICAL_NODES {
        if node.id == "epistemic-content" {
            continue; // root — not classifiable
        }
        let parent_label = node
            .parent_id
            .and_then(vertical_taxonomy::en_label)
            .unwrap_or("(none)");
        let branch_label = match node.branch {
            1 => "Sensory inputs",
            2 => "Symbolic entities",
            3 => "Semantic contents",
            4 => "Epistemic states",
            5 => "Higher-order constructs",
            _ => "(unknown)",
        };
        let text = format!(
            "{} ({}). Branch {} — {} taxonomy. Parent: {}.",
            node.en, node.ar, node.branch, branch_label, parent_label
        );
        out.push(ClassifierCandidate {
            id: node.id.to_string(),
            axis: ClassifierAxis::Vertical,
            embedding_text: text,
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_definitions_count() {
        assert_eq!(SOURCE_DEFINITIONS.len(), 11);
    }

    #[test]
    fn leaf_hints_count() {
        assert_eq!(HORIZONTAL_LEAF_HINTS.len(), 41);
    }

    #[test]
    fn every_horizontal_leaf_has_a_hint() {
        for n in horizontal_taxonomy::HORIZONTAL_NODES {
            if n.parent_id.is_some() {
                assert!(
                    leaf_hint(n.id).is_some(),
                    "missing hint for leaf `{}`", n.id
                );
            }
        }
    }

    #[test]
    fn build_candidates_covers_both_axes() {
        let candidates = build_classifier_candidates();
        let horizontal: Vec<_> = candidates
            .iter()
            .filter(|c| c.axis == ClassifierAxis::Horizontal)
            .collect();
        let vertical: Vec<_> = candidates
            .iter()
            .filter(|c| c.axis == ClassifierAxis::Vertical)
            .collect();
        // 11 parents + 41 leaves = 52 horizontal (excludes unclassifiable)
        assert_eq!(horizontal.len(), 52);
        // ~218 sub-nodes (excludes root)
        assert!(vertical.len() >= 215 && vertical.len() <= 225,
                "expected ~218 vertical, got {}", vertical.len());
    }

    #[test]
    fn parent_definitions_are_substantial() {
        for (id, def) in SOURCE_DEFINITIONS {
            assert!(
                def.split_whitespace().count() >= 100,
                "{} definition too short", id
            );
        }
    }
}
