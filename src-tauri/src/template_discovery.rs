//! MIG-103 §4 — RECOGNITION: discover the note shapes a Universe already contains.
//!
//! **Concept (the horse):** *the Template Studio does not invent templates — it
//! takes an impression from the casts already made.* A user who has written 32
//! philosopher notes already owns a philosopher template; it was never missing,
//! only never carved. This module finds those molds.
//!
//! **It reports, it does not predict.** Every shape returned is a statement about
//! what demonstrably recurs in the user's own notes, carried with its evidence
//! (how many notes, and which ones), so a proposal can always be checked.
//!
//! ## The algorithm, and how the real data shaped it
//!
//! Every step below was validated against a real 7,802-note Universe, and FOUR
//! designs were tried and discarded before this one. The audit that settled it:
//! `docs/concept-papers/MIG-103-Shape-Discovery-Algorithm-Audit.md`.
//!
//! 1. **Strip identity + provenance + system keys.** Fingerprinting on all
//!    properties gave 7,380 signatures for 7,802 notes — noise, because
//!    `cid_cn`/`created`/`title` are universal and `stage`/`maturity` sat on 98%.
//! 2. **Candidate cores** = recurring exact signatures, PLUS their pairwise
//!    intersections. Intersections matter because a family's core need not exist
//!    as a signature of its own: if every person note carries `born, died` plus
//!    something else, nothing has exactly `{born, died}` and the family shatters.
//! 3. **Keep MINIMAL cores** — a signature containing another is a richer variant,
//!    so the smaller absorbs it.
//! 4. **Drop redundant kinds by MEMBERSHIP overlap.** Minimality only sees key
//!    containment; it cannot tell that `{born, institutions}`, `{alma_mater,
//!    born}`, `{born, field}` and `{awards, born}` are one family sliced four
//!    ways. On the real corpus those crowded the top results (30 kinds); comparing
//!    who-belongs instead of which-keys collapses them to 21 while `{born,
//!    institutions}` survives on its own merits — an academic is not just a person.
//! 5. **Describe by FILL RATES, not a rigid set** — the audit's central finding,
//!    and what every mature system converged on independently.
//!
//! **Rejected, with evidence:** closed frequent itemsets (surfaced bare fields —
//! `born`, `country` — as if they were types); maximal itemsets (deleted every
//! type core); Jaccard clustering (mean 1.94 keys/note puts it in its documented
//! zero-overlap failure condition).
//!
//! Cost: one pass over `note_meta` (already maintained at write time — Rule 8),
//! a bounded candidate set, and set comparisons. Nothing at boot; on demand only.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};

/// Keys every note carries — they describe identity, never shape.
const IDENTITY_KEYS: &[&str] = &["cid_cn", "cid", "created", "kind", "title", "modified", "updated"];

/// Keys describing where a note CAME FROM rather than what it IS, plus the
/// system-assigned fields Constellation itself maintains (`stage`, `maturity` —
/// the Cognitive Engine writes these on essentially every note). Excluded so a
/// Universe does not collapse into one giant meaningless cluster.
const PROVENANCE_KEYS: &[&str] = &[
    "attribution", "cuniverse", "license", "source", "source_url", "folder", "library",
    "stage", "maturity",
];

/// Minimum notes sharing a shape before it is worth proposing. Deliberately low —
/// the concept paper's rule is "few, strong, well-evidenced", and an honest
/// "nothing recurs yet" beats a list of guesses.
pub const MIN_SUPPORT: usize = 3;

/// **A field on very nearly every note describes nothing.** Above this fraction
/// a candidate is a universal/system field, not a note type, and is dropped.
///
/// The general form of a bug the real Universe exposed: `stage` and `maturity`
/// sat on 7,595 of 7,802 notes (98%), so the top "shape" was `stage` — the
/// baseline wearing a template's costume. Naming those keys as noise fixes that
/// instance; this ratio fixes the CLASS, catching whatever universal property a
/// future Universe carries that nobody enumerated.
///
/// Set at 0.9, not lower, deliberately: a **legitimately dominant** shape must
/// survive. A Universe that is 80% article notes genuinely HAS an article shape,
/// and suppressing it because it is common would be the opposite error.
const MAX_SUPPORT_RATIO: f64 = 0.9;

/// Below this many shaped notes the ceiling is not applied at all — "on almost
/// every note" is not a meaningful statement about a handful of notes, and in a
/// small corpus a shape covering everything is simply the honest answer.
const MIN_CORPUS_FOR_RATIO: usize = 20;

fn is_noise(key: &str) -> bool {
    let k = key.to_lowercase();
    IDENTITY_KEYS.contains(&k.as_str()) || PROVENANCE_KEYS.contains(&k.as_str())
}

/// One property of a discovered shape, with how often it actually appears.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShapeField {
    pub key: String,
    /// Notes in this shape carrying the key.
    pub count: usize,
    /// `count / support` — 1.0 for a core field, lower for an optional one.
    pub fill: f64,
}

/// One recurring shape: a CORE every member carries, plus the optional tail, each
/// field reported with its fill rate.
///
/// **This representation is the audit's central finding** (`docs/concept-papers/
/// MIG-103-Shape-Discovery-Algorithm-Audit.md`). Storing a kind as a rigid SET of
/// keys is what made `{born,died}`, `{born,died,occupation}` and
/// `{born,died,predecessor,successor}` three unrelated "kinds"; storing it as
/// `{key → count}` dissolves the question — one kind, a hard core, an honest tail.
///
/// It is also what every mature system converged on independently: Wikipedia's
/// `Infobox person` has 142 parameters and **zero required**; MongoDB Compass
/// reports "present in 87% of documents"; quicktype merges records and marks the
/// difference optional. And counters merge associatively, so the whole surface can
/// later be maintained incrementally instead of rescanned (Rule 8).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredShape {
    /// Keys carried by EVERY note in this shape — what makes it this kind.
    pub core: Vec<String>,
    /// Every key seen across the shape's notes, core first, then by fill rate.
    pub fields: Vec<ShapeField>,
    /// Headings appearing in at least `HEADING_QUORUM` of the matching notes.
    pub headings: Vec<String>,
    /// How many notes carry the whole core.
    pub support: usize,
    /// A few example notes — the evidence, so a proposal is always checkable.
    pub examples: Vec<String>,
}

/// A kind whose notes are this fraction already covered by a LARGER kind adds
/// nothing and is dropped — redundancy-aware selection over membership, not keys.
const MAX_MEMBER_OVERLAP: f64 = 0.75;

/// A heading must appear in this fraction of a shape's notes to be part of it.
const HEADING_QUORUM: f64 = 0.4;

/// The per-note input: its property keys and heading texts, already extracted.
pub struct NoteFacts {
    pub path: String,
    pub property_keys: Vec<String>,
    pub headings: Vec<String>,
}

/// Discover the recurring shapes in a set of notes.
///
/// `max_shapes` caps the result so the caller never has to render a wall of weak
/// patterns; shapes are ranked by support, then by specificity (more properties
/// first), so the strongest and most descriptive surface first.
pub fn discover_shapes(notes: &[NoteFacts], max_shapes: usize) -> Vec<DiscoveredShape> {
    // 1. Reduce each note to its SEMANTIC property set.
    let sets: Vec<(usize, BTreeSet<String>)> = notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let s: BTreeSet<String> = n
                .property_keys
                .iter()
                .map(|k| k.to_lowercase())
                .filter(|k| !is_noise(k))
                .collect();
            (i, s)
        })
        .filter(|(_, s)| !s.is_empty())
        .collect();
    if sets.is_empty() {
        return Vec::new();
    }

    let ceiling = if sets.len() >= MIN_CORPUS_FOR_RATIO {
        ((sets.len() as f64) * MAX_SUPPORT_RATIO).ceil() as usize
    } else {
        usize::MAX // too small for "almost every note" to mean anything
    };

    // 2. Candidate CORES: exact signatures that recur. A core must carry at least
    //    two keys — one key is a FIELD, not a note kind (nobody wants a `born`
    //    template).
    let mut sig_counts: HashMap<BTreeSet<String>, usize> = HashMap::new();
    for (_, s) in &sets {
        *sig_counts.entry(s.clone()).or_insert(0) += 1;
    }
    let frequent: Vec<BTreeSet<String>> = sig_counts
        .into_iter()
        .filter(|(sig, n)| *n >= MIN_SUPPORT && sig.len() >= 2)
        .map(|(sig, _)| sig)
        .collect();

    // A family's core need NOT exist as a signature in its own right. If every
    // person note carries `born, died` PLUS something else, no note has exactly
    // `{born, died}` — and a candidate set built only from observed signatures
    // would miss the core and fragment the family all over again. (Our Universe
    // hid this: it happens to contain 146 plain `{born, died}` notes. A Universe
    // without them would have exposed it.) So pairwise intersections join the
    // candidate pool.
    //
    // This is NOT the closed-itemset pass that failed earlier: that one kept
    // subsets *alongside* their supersets, which is what surfaced bare fields.
    // Here every candidate still needs two keys, and step 3's minimality filter
    // keeps only the smallest core of each family — so intersections can only
    // MERGE fragments, never multiply them.
    let mut candidates: HashSet<BTreeSet<String>> = frequent.iter().cloned().collect();
    for i in 0..frequent.len() {
        for j in (i + 1)..frequent.len() {
            let inter: BTreeSet<String> = frequent[i].intersection(&frequent[j]).cloned().collect();
            if inter.len() >= 2 {
                candidates.insert(inter);
            }
        }
    }
    let candidates: Vec<BTreeSet<String>> = candidates.into_iter().collect();

    // 3. Keep only MINIMAL cores. A signature containing another candidate is a
    //    richer VARIANT of it, not a separate kind — so the smaller one wins and
    //    absorbs it. This is what turns twelve fragments of a person note into one
    //    kind with an optional tail.
    let cores: Vec<BTreeSet<String>> = candidates
        .iter()
        .filter(|c| !candidates.iter().any(|o| o.len() < c.len() && o.is_subset(c)))
        .cloned()
        .collect();

    // 4. Each core absorbs every note whose properties are a SUPERSET of it.
    let mut grouped: Vec<(BTreeSet<String>, Vec<usize>)> = cores
        .into_iter()
        .map(|core| {
            let members: Vec<usize> =
                sets.iter().filter(|(_, s)| core.is_subset(s)).map(|(i, _)| *i).collect();
            (core, members)
        })
        .filter(|(_, m)| m.len() >= MIN_SUPPORT && m.len() <= ceiling)
        .collect();

    // 4b. REDUNDANCY FILTER — drop a kind whose notes are already explained by a
    //     bigger one. Minimality (step 3) only catches cores that CONTAIN one
    //     another; it cannot see that `{born, institutions}`, `{alma_mater, born}`,
    //     `{born, field}` and `{awards, born}` are all the same person family
    //     sliced by a different second key. On the real Universe those crowded the
    //     top nine results. Comparing MEMBERSHIP instead of keys collapses them:
    //     30 kinds → 21, and the genuinely distinct ones survive (`born,
    //     institutions` keeps its place at 70% overlap — an academic really is not
    //     just a person). This is redundancy-aware pattern selection; without it,
    //     intersection-derived cores trade one kind of over-generation for another.
    grouped.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    let mut kept: Vec<(BTreeSet<String>, Vec<usize>)> = Vec::new();
    for (core, members) in grouped {
        let mine: HashSet<usize> = members.iter().copied().collect();
        let redundant = kept.iter().any(|(_, bigger)| {
            let shared = bigger.iter().filter(|i| mine.contains(i)).count();
            shared as f64 / mine.len() as f64 >= MAX_MEMBER_OVERLAP
        });
        if !redundant {
            kept.push((core, members));
        }
    }

    // 5. Describe each surviving group by FILL RATES rather than a rigid set.
    let mut shapes: Vec<DiscoveredShape> = kept
        .into_iter()
        .filter_map(|(core, members)| {
            let support = members.len();

            let mut field_counts: HashMap<String, usize> = HashMap::new();
            let mut heading_counts: HashMap<String, usize> = HashMap::new();
            for &i in &members {
                for k in notes[i].property_keys.iter().map(|k| k.to_lowercase()).filter(|k| !is_noise(k)) {
                    *field_counts.entry(k).or_insert(0) += 1;
                }
                let seen: HashSet<String> = notes[i]
                    .headings
                    .iter()
                    .map(|h| h.trim().to_lowercase())
                    .filter(|h| !h.is_empty())
                    .collect();
                for h in seen {
                    *heading_counts.entry(h).or_insert(0) += 1;
                }
            }

            let mut fields: Vec<ShapeField> = field_counts
                .into_iter()
                .map(|(key, count)| ShapeField { key, count, fill: count as f64 / support as f64 })
                .collect();
            // Core fields first (fill 1.0 by construction), then the tail by how
            // often it actually appears — the honest ordering.
            fields.sort_by(|a, b| {
                let ac = core.contains(&a.key);
                let bc = core.contains(&b.key);
                bc.cmp(&ac)
                    .then(b.count.cmp(&a.count))
                    .then(a.key.cmp(&b.key))
            });

            let quorum = ((support as f64) * HEADING_QUORUM).ceil() as usize;
            let mut headings: Vec<(String, usize)> =
                heading_counts.into_iter().filter(|(_, c)| *c >= quorum.max(2)).collect();
            headings.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

            Some(DiscoveredShape {
                core: core.iter().cloned().collect(),
                fields,
                headings: headings.into_iter().map(|(h, _)| h).take(12).collect(),
                support,
                examples: members.iter().take(5).map(|&i| notes[i].path.clone()).collect(),
            })
        })
        .collect();

    // 5. Rank: strongest support first; ties broken toward the more specific core.
    shapes.sort_by(|a, b| {
        b.support
            .cmp(&a.support)
            .then(b.core.len().cmp(&a.core.len()))
            .then(a.core.cmp(&b.core))
    });
    shapes.truncate(max_shapes);
    shapes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(path: &str, props: &[&str], heads: &[&str]) -> NoteFacts {
        NoteFacts {
            path: path.to_string(),
            property_keys: props.iter().map(|s| s.to_string()).collect(),
            headings: heads.iter().map(|s| s.to_string()).collect(),
        }
    }
    fn fill_of(s: &DiscoveredShape, key: &str) -> f64 {
        s.fields.iter().find(|f| f.key == key).map(|f| f.fill).unwrap_or(0.0)
    }

    #[test]
    fn identity_and_provenance_keys_are_never_a_shape() {
        let notes: Vec<NoteFacts> = (0..10)
            .map(|i| note(&format!("/n{i}.md"), &["cid_cn", "created", "kind", "title", "attribution", "license"], &[]))
            .collect();
        assert!(discover_shapes(&notes, 10).is_empty(), "noise-only notes have no shape");
    }

    #[test]
    fn constellation_system_fields_are_noise() {
        let notes: Vec<NoteFacts> = (0..8)
            .map(|i| note(&format!("/n{i}.md"), &["cid_cn", "stage", "maturity"], &[]))
            .collect();
        assert!(discover_shapes(&notes, 10).is_empty(), "stage/maturity describe no shape");
    }

    #[test]
    fn a_recurring_shape_is_discovered_with_its_evidence() {
        let notes: Vec<NoteFacts> = (0..6)
            .map(|i| note(&format!("/p{i}.md"), &["cid_cn", "born", "died"], &["Life", "Legacy"]))
            .collect();
        let shapes = discover_shapes(&notes, 10);
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].core, vec!["born", "died"]);
        assert_eq!(shapes[0].support, 6);
        assert_eq!(fill_of(&shapes[0], "born"), 1.0, "a core field is always present");
        assert!(shapes[0].headings.contains(&"life".to_string()));
        assert!(!shapes[0].examples.is_empty(), "a proposal must carry checkable evidence");
    }

    #[test]
    fn below_the_floor_nothing_is_proposed() {
        let notes = vec![
            note("/a.md", &["born", "died"], &[]),
            note("/b.md", &["born", "died"], &[]),
        ];
        assert!(discover_shapes(&notes, 10).is_empty(), "2 notes is under MIN_SUPPORT — silence beats a guess");
    }

    /// ★ THE AUDIT'S CENTRAL FINDING, in one test.
    ///
    /// A family that an exact-signature algorithm shatters into three unrelated
    /// "kinds" must come back as ONE kind: a hard core, and an optional tail
    /// carrying honest fill rates. This is the published treatment (parametric
    /// schema inference; Wikipedia's `Infobox person` has 142 parameters and zero
    /// required; MongoDB Compass reports "present in 87%").
    #[test]
    fn a_family_becomes_one_kind_with_an_optional_tail() {
        let mut notes = Vec::new();
        for i in 0..5 { notes.push(note(&format!("/plain{i}.md"), &["born", "died"], &[])); }
        for i in 0..4 { notes.push(note(&format!("/occ{i}.md"), &["born", "died", "occupation"], &[])); }
        for i in 0..3 { notes.push(note(&format!("/phil{i}.md"), &["born", "died", "school"], &[])); }

        let shapes = discover_shapes(&notes, 10);
        assert_eq!(shapes.len(), 1, "one family is ONE kind, not three");
        let s = &shapes[0];
        assert_eq!(s.core, vec!["born", "died"], "the core is what every member carries");
        assert_eq!(s.support, 12, "all twelve notes belong to it");
        assert_eq!(fill_of(s, "born"), 1.0);
        assert_eq!(fill_of(s, "died"), 1.0);
        assert!((fill_of(s, "occupation") - 4.0 / 12.0).abs() < 1e-9, "the tail reports its real fill rate");
        assert!((fill_of(s, "school") - 3.0 / 12.0).abs() < 1e-9);
    }

    /// Core fields lead; the optional tail follows by how often it truly appears.
    #[test]
    fn fields_are_ordered_core_first_then_by_fill() {
        let mut notes = Vec::new();
        for i in 0..6 { notes.push(note(&format!("/a{i}.md"), &["born", "died", "rare"], &[])); }
        for i in 0..6 { notes.push(note(&format!("/b{i}.md"), &["born", "died", "common"], &[])); }
        for i in 0..4 { notes.push(note(&format!("/c{i}.md"), &["born", "died", "common"], &[])); }
        let s = &discover_shapes(&notes, 10)[0];
        assert_eq!(s.fields[0].fill, 1.0);
        assert_eq!(s.fields[1].fill, 1.0);
        let common = s.fields.iter().position(|f| f.key == "common").unwrap();
        let rare = s.fields.iter().position(|f| f.key == "rare").unwrap();
        assert!(common < rare, "the commoner optional field is listed first");
    }

    #[test]
    fn a_lone_property_is_not_a_note_kind() {
        let bare: Vec<NoteFacts> = (0..6).map(|i| note(&format!("/b{i}.md"), &["aliases"], &[])).collect();
        assert!(discover_shapes(&bare, 10).is_empty(), "one field is a field, not a kind");
    }

    /// A field on very nearly every note is the baseline, not a template. In the
    /// real Universe `stage`/`maturity` sat on 7,595 of 7,802 notes.
    #[test]
    fn a_shape_on_almost_every_note_is_not_proposed() {
        let mut notes = Vec::new();
        for i in 0..30 { notes.push(note(&format!("/u{i}.md"), &["ubiq_a", "ubiq_b"], &[])); }
        for i in 0..4 { notes.push(note(&format!("/p{i}.md"), &["ubiq_a", "ubiq_b", "born", "died"], &[])); }
        let shapes = discover_shapes(&notes, 10);
        assert!(
            !shapes.iter().any(|s| s.core == vec!["ubiq_a", "ubiq_b"]),
            "a core on ~88% of notes is the baseline"
        );
    }

    #[test]
    fn distinct_shapes_are_ranked_by_support() {
        let mut notes = Vec::new();
        for i in 0..8 { notes.push(note(&format!("/film{i}.md"), &["country", "language"], &["Cast", "Plot"])); }
        for i in 0..3 { notes.push(note(&format!("/pub{i}.md"), &["publisher", "issn"], &[])); }
        let shapes = discover_shapes(&notes, 10);
        assert_eq!(shapes[0].support, 8, "the strongest shape leads");
        assert!(shapes[0].support >= shapes[shapes.len() - 1].support);
    }

    #[test]
    fn a_heading_below_quorum_is_not_part_of_the_shape() {
        let mut notes = Vec::new();
        for i in 0..10 { notes.push(note(&format!("/n{i}.md"), &["country", "region"], &["History"])); }
        notes.push(note("/odd.md", &["country", "region"], &["Weather"]));
        let s = &discover_shapes(&notes, 10)[0];
        assert!(s.headings.contains(&"history".to_string()));
        assert!(!s.headings.contains(&"weather".to_string()), "a one-off heading is not the shape");
    }

    /// The redundancy filter, which minimality cannot do. Two cores that share no
    /// key containment can still describe the SAME notes; the bigger one wins.
    #[test]
    fn a_kind_already_explained_by_a_bigger_one_is_dropped() {
        let mut notes = Vec::new();
        // 12 notes carry born+died+institutions — so {born,died} and
        // {born,institutions} both match them, sharing nearly all their members.
        for i in 0..12 { notes.push(note(&format!("/both{i}.md"), &["born", "died", "institutions"], &[])); }
        for i in 0..6 { notes.push(note(&format!("/plain{i}.md"), &["born", "died"], &[])); }

        let shapes = discover_shapes(&notes, 10);
        assert_eq!(shapes.len(), 1, "the smaller, fully-contained kind adds nothing");
        assert_eq!(shapes[0].core, vec!["born", "died"]);
        assert_eq!(shapes[0].support, 18);
        assert!(
            (fill_of(&shapes[0], "institutions") - 12.0 / 18.0).abs() < 1e-9,
            "the absorbed variant survives as an optional field with its real fill rate"
        );
    }

    #[test]
    fn max_shapes_caps_the_proposal_list() {
        let mut notes = Vec::new();
        for g in 0..10 {
            for i in 0..4 {
                let a: &'static str = Box::leak(format!("k{g}a").into_boxed_str());
                let b: &'static str = Box::leak(format!("k{g}b").into_boxed_str());
                notes.push(note(&format!("/g{g}n{i}.md"), &[a, b], &[]));
            }
        }
        assert!(discover_shapes(&notes, 3).len() <= 3);
    }
}
