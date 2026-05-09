//! MIG-021 §1B — The 11 source definitions used as embedding-classification
//! anchors. Drawn from the Universal Epistemic Content Taxonomy
//! (`docs/epistemic-content-taxonomy.md`) + Concept Paper v2.0 §7.1.
//!
//! Each definition is ~120-180 words, rich with semantic cues that
//! e5-small can distinguish (textual phrases, examples, contrasts with
//! adjacent sources). Embedded as compile-time constants — at app
//! startup the classifier embeds each definition once and caches the
//! 11 × 384-dim vectors in `tier1_embedding::SOURCE_VECTORS`.
//!
//! Per Plan §0 Q2: ~150 words per source, English canonical (the
//! e5-small model is multilingual and embeds English source-defs
//! into a shared space that aligns with non-English note content).
//! If accuracy is poor on Arabic-heavy notes, we may add bilingual
//! definition pairs in a future revision.

/// `(source_id, canonical English definition)` pairs for the 11
/// classifiable sources. Order matches `crate::sources::SOURCE_IDS[0..11]`.
/// `unclassifiable` is intentionally absent — it is an opt-out token
/// the classifier never suggests.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definitions_match_source_ids_count() {
        // 11 classifiable sources; 'unclassifiable' is the 12th opt-out token
        // (NOT in SOURCE_DEFINITIONS by design — classifier never suggests it).
        assert_eq!(SOURCE_DEFINITIONS.len(), 11);
        let ids: Vec<&str> = SOURCE_DEFINITIONS.iter().map(|(id, _)| *id).collect();
        let canonical = crate::sources::classifiable_sources();
        assert_eq!(ids, canonical);
    }

    #[test]
    fn definitions_are_substantial() {
        // Each definition should be at least 100 words (per Plan §0 Q2).
        for (id, def) in SOURCE_DEFINITIONS {
            let word_count = def.split_whitespace().count();
            assert!(
                word_count >= 100,
                "Definition for {} has only {} words; needs ≥100 for embedding-classification accuracy",
                id,
                word_count
            );
        }
    }

    #[test]
    fn definitions_are_not_too_long() {
        // Under 250 words each — keeps total embedding cost reasonable
        // and avoids drowning the distinctive cues in noise.
        for (id, def) in SOURCE_DEFINITIONS {
            let word_count = def.split_whitespace().count();
            assert!(
                word_count <= 250,
                "Definition for {} has {} words; over 250 may dilute embedding signal",
                id,
                word_count
            );
        }
    }
}
