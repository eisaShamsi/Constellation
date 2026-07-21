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
//! ## The algorithm, and why it is this one
//!
//! Validated against a real 7,802-note Universe before any of it was written, and
//! the first two attempts failed in ways that shaped the design:
//!
//! 1. Fingerprinting on ALL property keys gave 7,380 distinct signatures for
//!    7,802 notes — pure noise, because `cid_cn`/`created`/`title` sit on every
//!    note and import provenance (`attribution`, `license`, `source_url`, …) sits
//!    on thousands. → strip both classes; they describe identity and origin, not
//!    shape.
//! 2. Exact-signature grouping then FRAGMENTED obvious families: `born,died` (146
//!    notes), `born,died,occupation` (34), `born,died,predecessor,successor` (39)
//!    and `born,died,era,school,…` (32) are plainly one shape with variants. →
//!    the unit is not the exact signature but the **closed frequent property
//!    set**: a set P whose support (notes whose properties ⊇ P) meets a floor, and
//!    which no strict superset matches with the SAME support. `born,died` is
//!    reported as the broad core; each richer variant is reported alongside it,
//!    because each has its own distinct support.
//!
//! Cost: one pass over `note_meta` (already maintained at write time — Rule 8), a
//! bounded candidate set, and set comparisons. Nothing runs at boot; discovery is
//! on demand.

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

/// One recurring shape, with the evidence that justifies proposing it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredShape {
    /// The property keys that define this shape, sorted.
    pub properties: Vec<String>,
    /// Headings appearing in at least `HEADING_QUORUM` of the matching notes.
    pub headings: Vec<String>,
    /// How many notes carry every property in `properties`.
    pub support: usize,
    /// A few example notes — the evidence, so a proposal is always checkable.
    pub examples: Vec<String>,
}

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

    // 2. Group by EXACT signature. A note kind is the whole shape a note has, not
    //    every subset of it.
    //
    //    This replaced a closed-frequent-itemset pass (all pairwise intersections,
    //    then drop non-closed sets). That version was more sophisticated and
    //    measurably WORSE on the real 7,802-note Universe: it surfaced every
    //    intermediate subset, so the top proposals came back as bare single fields
    //    — `aliases`, `born`, `died`, `country` — which are fields, not note kinds.
    //    Exact signatures give `born, died, era, main_interests, notable_ideas,
    //    region, school` (a philosopher) and `country, language` + cast/plot/
    //    production (a film). The family insight survives without the machinery:
    //    a philosopher note and a plain person note ARE different templates, so
    //    reporting the variants separately is correct, not fragmentation.
    let ceiling = if sets.len() >= MIN_CORPUS_FOR_RATIO {
        ((sets.len() as f64) * MAX_SUPPORT_RATIO).ceil() as usize
    } else {
        usize::MAX // too small for "almost every note" to mean anything
    };

    let mut groups: HashMap<BTreeSet<String>, Vec<usize>> = HashMap::new();
    for (i, s) in &sets {
        groups.entry(s.clone()).or_default().push(*i);
    }

    // 3. Score each group, attaching the evidence that justifies proposing it.
    let mut shapes: Vec<DiscoveredShape> = groups
        .into_iter()
        .filter(|(_, m)| m.len() >= MIN_SUPPORT && m.len() <= ceiling)
        .map(|(props, members)| {
            let mut counts: HashMap<String, usize> = HashMap::new();
            for &i in &members {
                let seen: HashSet<String> = notes[i]
                    .headings
                    .iter()
                    .map(|h| h.trim().to_lowercase())
                    .filter(|h| !h.is_empty())
                    .collect();
                for h in seen {
                    *counts.entry(h).or_insert(0) += 1;
                }
            }
            let quorum = ((members.len() as f64) * HEADING_QUORUM).ceil() as usize;
            let mut headings: Vec<(String, usize)> =
                counts.into_iter().filter(|(_, c)| *c >= quorum.max(2)).collect();
            headings.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            DiscoveredShape {
                properties: props.into_iter().collect(),
                headings: headings.into_iter().map(|(h, _)| h).take(12).collect(),
                support: members.len(),
                examples: members.iter().take(5).map(|&i| notes[i].path.clone()).collect(),
            }
        })
        // 4. A lone property with no recurring headings is a FIELD, not a note
        //    kind — nobody wants a "born" template. Two properties, or one plus a
        //    heading structure, is the floor for calling something a shape.
        .filter(|s| s.properties.len() >= 2 || !s.headings.is_empty())
        .collect();

    // 5. Rank: strongest support first; ties broken toward the more specific shape.
    shapes.sort_by(|a, b| {
        b.support
            .cmp(&a.support)
            .then(b.properties.len().cmp(&a.properties.len()))
            .then(a.properties.cmp(&b.properties))
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

    #[test]
    fn identity_and_provenance_keys_are_never_a_shape() {
        // Every note carries these; they describe no shape at all.
        let notes: Vec<NoteFacts> = (0..10)
            .map(|i| note(&format!("/n{i}.md"), &["cid_cn", "created", "kind", "title", "attribution", "license"], &[]))
            .collect();
        assert!(discover_shapes(&notes, 10).is_empty(), "noise-only notes have no shape");
    }

    #[test]
    fn a_recurring_property_set_is_discovered_with_its_evidence() {
        let notes: Vec<NoteFacts> = (0..6)
            .map(|i| note(&format!("/p{i}.md"), &["cid_cn", "born", "died"], &["Life", "Legacy"]))
            .collect();
        let shapes = discover_shapes(&notes, 10);
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].properties, vec!["born", "died"]);
        assert_eq!(shapes[0].support, 6);
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

    /// THE FAMILY CASE — and the design the real Universe corrected.
    ///
    /// An earlier version mined closed frequent itemsets so a family's CORE
    /// (`born,died`) was reported with the combined support of all its variants.
    /// Run against the real 7,802-note Universe that produced *worse* proposals:
    /// bare single fields (`born`, `died`, `country`) crowded the top, because
    /// every intermediate subset qualified.
    ///
    /// The corrected semantics, validated on that data: **each variant is its own
    /// note kind.** A plain person note, a person-in-office note and a philosopher
    /// note ARE different templates — reporting them separately is right, not
    /// fragmentation. The plain signature keeps only ITS own support.
    #[test]
    fn family_variants_are_each_their_own_kind() {
        let mut notes = Vec::new();
        for i in 0..5 { notes.push(note(&format!("/plain{i}.md"), &["born", "died"], &[])); }
        for i in 0..4 { notes.push(note(&format!("/occ{i}.md"), &["born", "died", "occupation"], &[])); }
        for i in 0..3 { notes.push(note(&format!("/phil{i}.md"), &["born", "died", "school", "era"], &["Philosophy"])); }

        let shapes = discover_shapes(&notes, 10);
        let plain = shapes.iter().find(|s| s.properties == vec!["born", "died"]).expect("the plain kind is a kind");
        assert_eq!(plain.support, 5, "a signature counts only the notes that actually have it");

        assert!(shapes.iter().any(|s| s.properties.contains(&"occupation".to_string()) && s.support == 4));
        assert!(shapes.iter().any(|s| s.properties.contains(&"school".to_string()) && s.support == 3));
        assert_eq!(shapes.len(), 3, "three variants, three kinds — no synthetic intermediate subsets");
    }

    /// A lone property is a FIELD, not a note kind — nobody wants a `born`
    /// template. Two properties, or one plus recurring headings, is the floor.
    #[test]
    fn a_lone_property_is_not_a_note_kind() {
        let bare: Vec<NoteFacts> = (0..6).map(|i| note(&format!("/b{i}.md"), &["aliases"], &[])).collect();
        assert!(discover_shapes(&bare, 10).is_empty(), "one field with no structure is not a kind");

        // …but one property WITH a recurring heading structure is.
        let structured: Vec<NoteFacts> =
            (0..6).map(|i| note(&format!("/c{i}.md"), &["country"], &["History"])).collect();
        let shapes = discover_shapes(&structured, 10);
        assert_eq!(shapes.len(), 1);
        assert!(shapes[0].headings.contains(&"history".to_string()));
    }

    /// One signature, one kind — no synthetic subsets of it.
    #[test]
    fn one_signature_yields_exactly_one_kind() {
        // Every note with `born` also has `died` — `born` alone adds nothing.
        let notes: Vec<NoteFacts> = (0..5).map(|i| note(&format!("/n{i}.md"), &["born", "died"], &[])).collect();
        let shapes = discover_shapes(&notes, 10);
        assert_eq!(shapes.len(), 1, "only the closed set survives");
        assert_eq!(shapes[0].properties, vec!["born", "died"]);
    }

    #[test]
    fn distinct_shapes_are_ranked_by_support() {
        let mut notes = Vec::new();
        for i in 0..8 { notes.push(note(&format!("/film{i}.md"), &["country", "language"], &["Cast", "Plot"])); }
        for i in 0..3 { notes.push(note(&format!("/pub{i}.md"), &["publisher", "issn"], &[])); }
        let shapes = discover_shapes(&notes, 10);
        assert!(shapes[0].support >= shapes[shapes.len() - 1].support);
        assert_eq!(shapes[0].support, 8, "the strongest shape leads");
    }

    #[test]
    fn a_heading_below_quorum_is_not_part_of_the_shape() {
        let mut notes = Vec::new();
        for i in 0..10 { notes.push(note(&format!("/n{i}.md"), &["country"], &["History"])); }
        notes.push(note("/odd.md", &["country"], &["Weather"]));
        let shapes = discover_shapes(&notes, 10);
        let s = &shapes[0];
        assert!(s.headings.contains(&"history".to_string()));
        assert!(!s.headings.contains(&"weather".to_string()), "a one-off heading is not the shape");
    }

    /// THE REAL-DATA CATCH — a property on nearly every note is the baseline, not
    /// a shape. In the Boss's Universe `stage`/`maturity` sat on 7,595 of 7,802
    /// notes and the algorithm proposed "stage" as its top template.
    #[test]
    fn a_property_on_almost_every_note_is_not_a_shape() {
        let mut notes = Vec::new();
        // 20 notes all carrying a universal field; 4 of them also form a real shape.
        for i in 0..20 {
            notes.push(note(&format!("/u{i}.md"), &["universal"], &[]));
        }
        for i in 0..4 {
            notes.push(note(&format!("/p{i}.md"), &["universal", "born", "died"], &[]));
        }
        let shapes = discover_shapes(&notes, 10);
        assert!(
            !shapes.iter().any(|s| s.properties == vec!["universal"]),
            "a field on ~83% of notes is the baseline, not a template"
        );
        assert!(
            shapes.iter().any(|s| s.properties.contains(&"born".to_string())),
            "the genuinely selective shape must still be found"
        );
    }

    /// The system-assigned Cognitive Engine fields are noise by name as well —
    /// belt to the ratio's braces.
    #[test]
    fn constellation_system_fields_are_noise() {
        let notes: Vec<NoteFacts> = (0..8)
            .map(|i| note(&format!("/n{i}.md"), &["cid_cn", "stage", "maturity"], &[]))
            .collect();
        assert!(discover_shapes(&notes, 10).is_empty(), "stage/maturity describe no shape");
    }

    #[test]
    fn max_shapes_caps_the_proposal_list() {
        let mut notes = Vec::new();
        for g in 0..10 {
            for i in 0..4 {
                notes.push(note(&format!("/g{g}n{i}.md"), &[Box::leak(format!("k{g}").into_boxed_str())], &[]));
            }
        }
        assert!(discover_shapes(&notes, 3).len() <= 3);
    }
}
