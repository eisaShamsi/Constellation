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
    /// Lowercased — the IDENTITY, used for every comparison and lookup.
    pub key: String,
    /// The spelling MOST of the members actually use (`Country`, not `country`).
    ///
    /// Identity is lowercased so `Country` and `country` are one field; but a template
    /// written from the lowercased identity would not match its own casts, and a
    /// case-mismatched frontmatter key spawns a DUPLICATE property in every note made
    /// from that template. So the mold is cut with the spelling the user actually
    /// writes. Rendering and the written file use this; nothing else does.
    pub display: String,
    /// Notes in this shape carrying the key.
    pub count: usize,
    /// `count / support` — 1.0 for a core field, lower for an optional one.
    pub fill: f64,
}

/// A recurring section heading, with the spelling the members actually use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShapeHeading {
    /// Lowercased identity.
    pub text: String,
    /// Modal original spelling — `Cast`, not `cast`. See `ShapeField::display`.
    pub display: String,
}

/// One example note. The PATH is the identity; the TITLE is what a human recognises.
///
/// Constellation's filenames are canonical (`YYYYMMDDTHHMMSSZ_NOTE_XXXX.md`), so a
/// basename tells the user nothing. For the kinds with no proposed name — 18 of 21 on
/// the real Universe — these titles are the densest recognition signal the surface has.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShapeExample {
    pub path: String,
    pub title: String,
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
    pub headings: Vec<ShapeHeading>,
    /// How many notes carry the whole core.
    pub support: usize,
    /// A few example notes — the evidence, so a proposal is always checkable.
    pub examples: Vec<ShapeExample>,
    /// A name read off the members (§4B), or `None` when the corpus does not
    /// contain one. `None` is a real answer, not a failure: on the real Universe the
    /// largest kind of all (679 notes sharing `born · died`) has no name anywhere in
    /// the user's material, because a person note is written in every library. The
    /// honest move is to show the evidence and let the user name it.
    pub proposed_name: Option<ProposedName>,
    /// Every candidate that cleared all three gates, best first — rank 1 is the
    /// proposal, the rest are the user's one-click alternates. Candidates that FAILED
    /// the bar never appear here: showing a rejected candidate as a clickable chip
    /// turns a rejection back into a suggestion with extra steps.
    pub name_candidates: Vec<ProposedName>,
}

/// A kind whose notes are this fraction already covered by a LARGER kind adds
/// nothing and is dropped — redundancy-aware selection over membership, not keys.
const MAX_MEMBER_OVERLAP: f64 = 0.75;

/// A heading must appear in this fraction of a shape's notes to be part of it.
const HEADING_QUORUM: f64 = 0.4;

/// The per-note input: everything the engine reads about one note.
///
/// The first three fields SHAPE the note (which kind it belongs to). The rest only
/// ever NAME it (§4B) — they are deliberately kept out of shape discovery, because
/// a folder or a tag says where a note was filed, not what kind of thing it is.
#[derive(Default)]
pub struct NoteFacts {
    pub path: String,
    pub property_keys: Vec<String>,
    pub headings: Vec<String>,
    /// The library this note belongs to — a container the user named.
    pub library: String,
    /// The note's title.
    pub title: String,
    /// Hashtags on the note.
    pub tags: Vec<String>,
    /// Frontmatter values as `(key, value)`. Noise keys are dropped here, not by
    /// the caller — see `NAME_MAX_VALUE_LEN` and `is_noise`.
    pub property_values: Vec<(String, String)>,
}

/// Where a naming candidate was found. These are the INDEPENDENT evidence families:
/// a token corroborated by two of them is far likelier to be the kind's real name
/// than one seen many times in a single family.
///
/// Independence is not free — it had to be engineered. `folder:` is *also* a
/// frontmatter property and the library name is *also* a path segment, so a naive
/// split double-counts one piece of evidence as two and manufactures agreement out
/// of nothing. Measured on the real Universe, that inflated 8 honest results to 18
/// mostly-wrong ones. `note_families` subtracts the overlaps.
pub const NAME_FAMILIES: [&str; 6] = ["library", "folder", "tag", "title", "value", "heading"];
const FAM_LIBRARY: usize = 0;
const FAM_FOLDER: usize = 1;
const FAM_TAG: usize = 2;
const FAM_TITLE: usize = 3;
const FAM_VALUE: usize = 4;
const FAM_HEADING: usize = 5;

/// Confidence level for the coverage interval — 95%.
const NAME_Z: f64 = 1.96;
/// A candidate must describe at least this share of the kind's notes — measured as
/// the LOWER BOUND of the interval, never the observed rate.
const NAME_MIN_COVERAGE_LB: f64 = 0.5;
/// ...and be at least this much commoner inside the kind than in the Universe at
/// large. THIS is what removes stopwords without a stopword list — see `rank_names`.
const NAME_MIN_LIFT_LB: f64 = 2.0;
/// ...and be corroborated by this many INDEPENDENT families.
const NAME_MIN_FAMILIES: usize = 2;
/// How many cleared candidates to keep: rank 1 is the proposal, the rest are the
/// user's one-click alternates and the collision fallback list.
const NAME_MAX_CANDIDATES: usize = 3;
/// A property value longer than this is prose, not a label.
const NAME_MAX_VALUE_LEN: usize = 40;

/// One family's support for a proposed name — the evidence, shown to the user.
///
/// The raw counts are carried, not just the statistics, because the UI shows counts:
/// *"library **Film** — 64 of these 64 notes; 412 in your Universe"*. A rendered
/// confidence score would be invented — there is no ground truth here to measure one
/// against — and inventing one is exactly what the project forbids.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NameEvidence {
    /// One of `NAME_FAMILIES`.
    pub family: String,
    /// Notes in this kind carrying the token in this family.
    pub members_with: usize,
    /// Notes in this kind.
    pub members_total: usize,
    /// Notes in the whole Universe carrying it in this family (members included).
    pub corpus_with: usize,
    /// Notes in the whole Universe.
    pub corpus_total: usize,
    /// Wilson 95% lower bound of `members_with / members_total`. Ranking only.
    pub coverage_lb: f64,
    /// `coverage_lb / (corpus_with / corpus_total)`. Ranking only.
    pub lift_lb: f64,
}

/// A name PROPOSED for a discovered kind — never imposed. The user edits it.
///
/// `evidence` is not decoration. Constellation proposes from the user's own
/// material and shows its reasoning; a name with no visible basis is indistinguishable
/// from one we invented, which the project forbids outright.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProposedName {
    /// The user's own word, verbatim — never stemmed, singularized or translated.
    pub name: String,
    /// Every family that independently supports it, strongest first.
    pub evidence: Vec<NameEvidence>,
    /// Tie-breaker only; never rendered.
    pub score: f64,
}

/// Wilson score interval, lower bound, 95%.
///
/// The small-sample correction. A token on 3 of 3 notes is observed at 100% with
/// unbounded lift and no confidence whatever; dividing by the estimate's own
/// uncertainty is what fixes that. Same observed rate, different evidence volume:
/// `3/3 → 0.438`, `8/8 → 0.676`, `50/50 → 0.929`.
///
/// It is applied to the GATES, not merely to the score, and that ordering is the
/// point. Monroe, Colaresi & Quinn (2008) §3.2.6 on the alternative — a minimum
/// count threshold — - is blunt: it "simply removes the most problematic features
/// without resolving the issue", handing you the highest-lift token that barely
/// clears the floor. Gating on the lower bound instead makes the raw coverage a kind
/// must reach RISE automatically as the kind shrinks, with no count threshold
/// anywhere: at `m = 3` the maximum attainable bound is 0.438, so a three-note kind
/// can never be named — which is the correct outcome, derived rather than chosen.
fn wilson_lb(k: usize, n: usize) -> f64 {
    if n == 0 || k == 0 {
        return 0.0;
    }
    let nn = n as f64;
    let p = (k.min(n) as f64) / nn;
    let z2 = NAME_Z * NAME_Z;
    let centre = p + z2 / (2.0 * nn);
    let margin = NAME_Z * (((p * (1.0 - p)) + z2 / (4.0 * nn)) / nn).sqrt();
    ((centre - margin) / (1.0 + z2 / nn)).max(0.0)
}

/// Keys whose VALUE can never name a kind — it names the import, the container, or
/// the note's own identity.
///
/// Deliberately NOT `is_noise`: `kind`, `stage` and `maturity` are excluded from
/// shape discovery (there they are the baseline wearing a template's costume) but a
/// `kind: film` value is the single best naming signal that exists, so they belong
/// here. Over-inclusion is safe — `stage` and `maturity` sit on 98% of the real
/// Universe, so their values score a lift of about 1.0 and the contrast gate drops
/// them unaided.
fn is_value_noise(key: &str) -> bool {
    matches!(
        key,
        // provenance — names where the note came from
        "source" | "source_url" | "license" | "attribution" | "cuniverse"
        // containers — already the library and folder families
        | "folder" | "library"
        // identity — unique per note, or the title family again
        | "cid" | "cid_cn" | "created" | "modified" | "updated" | "title"
    )
}

/// The spelling most members actually wrote, or the lowercase identity if unknown.
/// Ties break toward the alphabetically-first spelling so the result is deterministic.
fn modal_spelling(seen: Option<&HashMap<String, usize>>, fallback: &str) -> String {
    seen.and_then(|m| {
        m.iter()
            .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
            .map(|(spelling, _)| spelling.clone())
    })
    .unwrap_or_else(|| fallback.to_string())
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

    // 5. Describe each surviving group by FILL RATES rather than a rigid set, and
    //    read a name off its members (§4B). The corpus counts are built once — every
    //    lift is measured against the whole Universe, which is exactly what makes a
    //    stopword list unnecessary.
    let name_corpus = build_name_corpus(notes);
    let mut shapes: Vec<DiscoveredShape> = kept
        .into_iter()
        .filter_map(|(core, members)| {
            let support = members.len();

            let mut field_counts: HashMap<String, usize> = HashMap::new();
            let mut heading_counts: HashMap<String, usize> = HashMap::new();
            // lowercase identity -> {original spelling -> times seen}, so the mold can
            // be cut with the spelling the members actually use.
            let mut field_spellings: HashMap<String, HashMap<String, usize>> = HashMap::new();
            let mut heading_spellings: HashMap<String, HashMap<String, usize>> = HashMap::new();
            for &i in &members {
                for raw in notes[i].property_keys.iter() {
                    let k = raw.to_lowercase();
                    if is_noise(&k) {
                        continue;
                    }
                    *field_counts.entry(k.clone()).or_insert(0) += 1;
                    *field_spellings.entry(k).or_default().entry(raw.clone()).or_insert(0) += 1;
                }
                let mut seen: HashSet<String> = HashSet::new();
                for raw in notes[i].headings.iter() {
                    let h = raw.trim().to_lowercase();
                    if h.is_empty() || !seen.insert(h.clone()) {
                        continue; // once per note, however often it repeats inside it
                    }
                    *heading_counts.entry(h.clone()).or_insert(0) += 1;
                    *heading_spellings
                        .entry(h)
                        .or_default()
                        .entry(raw.trim().to_string())
                        .or_insert(0) += 1;
                }
            }

            let mut fields: Vec<ShapeField> = field_counts
                .into_iter()
                .map(|(key, count)| ShapeField {
                    display: modal_spelling(field_spellings.get(&key), &key),
                    key,
                    count,
                    fill: count as f64 / support as f64,
                })
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
                headings: headings
                    .into_iter()
                    .take(12)
                    .map(|(h, _)| ShapeHeading {
                        display: modal_spelling(heading_spellings.get(&h), &h),
                        text: h,
                    })
                    .collect(),
                support,
                examples: members
                    .iter()
                    .take(5)
                    .map(|&i| ShapeExample {
                        path: notes[i].path.clone(),
                        title: notes[i].title.clone(),
                    })
                    .collect(),
                proposed_name: None, // set by resolve_name_collisions, below
                name_candidates: rank_names(&name_corpus, &members),
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

    // Names are settled LAST, and only among the shapes the user will actually see —
    // a kind that fell below `max_shapes` must never claim a visible kind's name.
    let mut key_df: HashMap<String, usize> = HashMap::new();
    for (_, s) in &sets {
        for k in s {
            *key_df.entry(k.clone()).or_insert(0) += 1;
        }
    }
    resolve_name_collisions(&mut shapes, &key_df);
    shapes
}

// ─────────────────────────────────────────────────────────────────────────────
// §4B — NAMING: reading a kind's name off its own members
//
// Discovery answers "these 84 notes are the same kind of thing." It cannot answer
// "what kind?" — `{country, language}` does not spell FILM. The name has to come
// from the notes themselves, in the user's own words, and it has to be a PROPOSAL
// the user edits, never a decision.
//
// The method is differential (cluster-relative) labelling: rank a candidate not by
// how often it appears inside the kind, but by how much likelier it is inside than
// outside. Measured on the real Universe, plain frequency proposes `source=Wikipedia`
// (100% of every kind) and `the` (44% of film titles); the same candidates score
// lift 1.0 and 1.3 and vanish without anyone writing a stopword list — which is the
// only approach that can work when the corpus is fifteen languages.
//
// Lift alone then over-rewards rarity: the film kind's highest-lift token is the
// imported maintenance tag `template-film-date-with-1-release-date` (69x). What
// separates the real name from that is CORROBORATION — `film` is independently the
// library name, a tag, a title word and a property value; the maintenance tag exists
// in one family only. Requiring two independent families is what makes this precise.
// ─────────────────────────────────────────────────────────────────────────────

/// Maps token text to a dense id, so the per-note candidate sets are `u32`s rather
/// than a few million `String` clones on a large Universe.
#[derive(Default)]
struct Interner {
    ids: HashMap<String, u32>,
    text: Vec<String>,
}

impl Interner {
    fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.ids.get(s) {
            return id;
        }
        let id = self.text.len() as u32;
        self.ids.insert(s.to_string(), id);
        self.text.push(s.to_string());
        id
    }
}

/// Every candidate token in one piece of text: the WHOLE trimmed string, plus each
/// word inside it.
///
/// The whole string matters for more than tidiness. Word-splitting assumes a script
/// that separates words with spaces; Chinese and Japanese do not, so a library named
/// 映画 yields no words at all and would be invisible to a token-only candidate set.
/// Keeping the intact string means a container the user named is always a candidate
/// in every script. (Language-First by Design, at the algorithm level.)
fn name_tokens(text: &str, out: &mut Vec<String>) {
    let whole = text.trim().to_lowercase();
    if whole.is_empty() {
        return;
    }
    let n = whole.chars().count();
    if (2..=NAME_MAX_VALUE_LEN).contains(&n) {
        out.push(whole.clone());
    }
    // Two characters, not three. A three-character floor is a Latin-script
    // assumption: `カサブランカ (映画)` splits into `カサブランカ` and `映画`, and a
    // three-char rule silently drops the half that carries the meaning. Two-character
    // words are the norm in CJK. Admitting English `of`/`in` costs nothing — their
    // base rate is near 1.0, so the contrast gate rejects them at a lift of ~1. The
    // length rule was doing a job the lift gate already does better, in every language.
    for w in whole.split(|c: char| !c.is_alphanumeric()) {
        if w.chars().count() >= 2 && !w.chars().all(|c| c.is_numeric()) {
            out.push(w.to_string());
        }
    }
}

/// One note's candidate tokens, split into the six INDEPENDENT families.
///
/// The two subtractions at the end are the load-bearing part: without them `folder`
/// and `value` agree whenever a note carries a `folder:` property, and `library` and
/// `folder` agree always, so a single fact masquerades as corroboration. Measured on
/// the real Universe that inflated 8 honest results into 18 mostly-wrong ones.
fn note_families(n: &NoteFacts, interner: &mut Interner) -> [Vec<u32>; 6] {
    let mut raw: [Vec<String>; 6] = Default::default();

    name_tokens(&n.library, &mut raw[FAM_LIBRARY]);
    let norm = n.path.replace('\\', "/");
    let mut segs: Vec<&str> = norm.split('/').collect();
    segs.pop(); // the file name is the title, not a folder
    for seg in segs {
        name_tokens(seg, &mut raw[FAM_FOLDER]);
    }
    for tag in &n.tags {
        name_tokens(tag.trim_start_matches('#').trim_matches('"'), &mut raw[FAM_TAG]);
    }
    name_tokens(&n.title, &mut raw[FAM_TITLE]);
    for (k, v) in &n.property_values {
        // A provenance value (`source: Wikipedia`) names the import, not the kind;
        // a container value (`folder: History`) is the container family again.
        if is_value_noise(&k.to_lowercase()) || v.chars().count() > NAME_MAX_VALUE_LEN {
            continue;
        }
        name_tokens(v, &mut raw[FAM_VALUE]);
    }
    for h in &n.headings {
        name_tokens(h, &mut raw[FAM_HEADING]);
    }

    let mut out: [Vec<u32>; 6] = Default::default();
    for f in 0..6 {
        out[f] = raw[f].iter().map(|t| interner.intern(t)).collect();
        out[f].sort_unstable();
        out[f].dedup();
    }
    // Independence repair — a token already counted by a container family is not
    // additionally "evidence" because it also appears as a property value.
    let lib: HashSet<u32> = out[FAM_LIBRARY].iter().copied().collect();
    out[FAM_FOLDER].retain(|t| !lib.contains(t));
    let container: HashSet<u32> =
        out[FAM_LIBRARY].iter().chain(out[FAM_FOLDER].iter()).copied().collect();
    out[FAM_VALUE].retain(|t| !container.contains(t));
    out
}

/// Corpus-wide counts per family — the denominator of every lift.
struct NameCorpus {
    per_note: Vec<[Vec<u32>; 6]>,
    base: [Vec<u32>; 6],
    text: Vec<String>,
    total: usize,
}

fn build_name_corpus(notes: &[NoteFacts]) -> NameCorpus {
    let mut interner = Interner::default();
    let per_note: Vec<[Vec<u32>; 6]> =
        notes.iter().map(|n| note_families(n, &mut interner)).collect();
    let size = interner.text.len();
    let mut base: [Vec<u32>; 6] = Default::default();
    for f in 0..6 {
        base[f] = vec![0; size];
    }
    for fams in &per_note {
        for f in 0..6 {
            for &t in &fams[f] {
                base[f][t as usize] += 1;
            }
        }
    }
    NameCorpus { per_note, base, text: interner.text, total: notes.len() }
}

/// Propose a name for one kind, or `None`.
///
/// `None` is not a shortfall. `{born, died}` is the largest kind in the real Universe
/// (679 notes) and has no name in it: its notes sit 20% in Philosophy, 15% in Film,
/// 13% in Literature — because a person note gets written wherever the user is
/// working. Its strongest candidate is the imported tag token `deaths` (88%, 10x),
/// supported by one family. Proposing "Deaths" would be worse than proposing nothing:
/// a prefilled wrong answer anchors, a blank field asks.
fn rank_names(corpus: &NameCorpus, members: &[usize]) -> Vec<ProposedName> {
    let m = members.len();
    if m == 0 || corpus.total == 0 {
        return Vec::new();
    }
    let mut counts: HashMap<(usize, u32), usize> = HashMap::new();
    for &i in members {
        for f in 0..6 {
            for &t in &corpus.per_note[i][f] {
                *counts.entry((f, t)).or_insert(0) += 1;
            }
        }
    }

    // token -> every family clearing BOTH bars
    let mut hits: HashMap<u32, Vec<NameEvidence>> = HashMap::new();
    for ((f, t), k) in counts {
        let coverage_lb = wilson_lb(k, m);
        if coverage_lb < NAME_MIN_COVERAGE_LB {
            continue; // GATE 1 — describes the kind
        }
        let corpus_with = corpus.base[f][t as usize] as usize;
        if corpus_with == 0 {
            continue;
        }
        // The base rate INCLUDES the members, so `corpus_with >= k` and lift is capped
        // at `corpus_total / m` by construction — exclusivity can never yield an
        // infinite lift, and the reward for it shrinks as the kind grows. That is the
        // regularisation the rare-item literature asks for, with no constant to pick.
        let base = corpus_with as f64 / corpus.total as f64;
        let lift_lb = coverage_lb / base;
        if lift_lb < NAME_MIN_LIFT_LB {
            continue; // GATE 2 — commoner here than everywhere
        }
        hits.entry(t).or_default().push(NameEvidence {
            family: NAME_FAMILIES[f].to_string(),
            members_with: k,
            members_total: m,
            corpus_with,
            corpus_total: corpus.total,
            coverage_lb,
            lift_lb,
        });
    }

    let mut out: Vec<ProposedName> = hits
        .into_iter()
        .filter(|(_, fams)| fams.len() >= NAME_MIN_FAMILIES) // GATE 3 — corroborated
        .map(|(t, mut fams)| {
            let score: f64 = fams.iter().map(|e| e.coverage_lb * e.lift_lb.ln()).sum();
            fams.sort_by(|a, b| {
                b.lift_lb.partial_cmp(&a.lift_lb).unwrap_or(std::cmp::Ordering::Equal)
            });
            ProposedName { name: corpus.text[t as usize].clone(), evidence: fams, score }
        })
        .collect();

    // Corroborating families FIRST; the score only breaks ties. This ordering is
    // load-bearing, not cosmetic: no scalar statistic can separate
    // `template-film-date-with-1-release-date` from `film`, because the maintenance
    // tag is MAXIMALLY discriminative — the same importer emitted both it and the
    // frontmatter keys the kind was discovered from. They are two fingerprints of one
    // process, and only independence of evidence tells them apart.
    out.sort_by(|a, b| {
        b.evidence
            .len()
            .cmp(&a.evidence.len())
            .then(b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.name.cmp(&b.name))
    });
    out.truncate(NAME_MAX_CANDIDATES);
    out
}

/// Two kinds can legitimately propose the same word. The bigger one keeps it; the
/// other walks its own ranked list, and only if every alternate is taken does it
/// qualify rank 1 with the core key that is RAREST in the Universe — a word the user
/// actually wrote. So two person-ish kinds become `person` and `person · institutions`,
/// never `person` and `person 2`: a numeric suffix makes an identifier unique, which
/// is not this name's job. Recognition is.
fn resolve_name_collisions(shapes: &mut [DiscoveredShape], key_df: &HashMap<String, usize>) {
    let mut order: Vec<usize> = (0..shapes.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(shapes[i].support));
    let mut taken: HashSet<String> = HashSet::new();
    for i in order {
        let cands = shapes[i].name_candidates.clone();
        if cands.is_empty() {
            continue;
        }
        if let Some(free) = cands.iter().find(|c| !taken.contains(&c.name)) {
            taken.insert(free.name.clone());
            shapes[i].proposed_name = Some(free.clone());
            continue;
        }
        let mut core = shapes[i].core.clone();
        core.sort_by_key(|k| *key_df.get(k).unwrap_or(&0));
        let mut name = format!("{} · {}", cands[0].name, core[0]);
        if taken.contains(&name) && core.len() > 1 {
            name = format!("{} · {} · {}", cands[0].name, core[0], core[1]);
        }
        taken.insert(name.clone());
        shapes[i].proposed_name = Some(ProposedName { name, ..cands[0].clone() });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §4 SURFACE — the IPC boundary
// ─────────────────────────────────────────────────────────────────────────────

/// Everything the recognition panel needs, in one read.
///
/// Read-only and on demand — never at boot. Rule 8 (Write-Time Derivation) forbids
/// re-walking the Universe to build a derived view, and this does not: `note_meta` is
/// already maintained on the write path, so this is one indexed pass over data that is
/// always current, not a rebuild. If the panel ever becomes something the user leaves
/// open, the counters here merge associatively and can be maintained incrementally.
#[tauri::command]
pub fn discover_template_shapes(
    state: tauri::State<crate::search::SearchState>,
    max_shapes: Option<usize>,
) -> Result<Vec<DiscoveredShape>, String> {
    // PJ-066 §C3 — the READ-ONLY reader connection, so a multi-second scan can never
    // wait on (or hold) the writer's lock and freeze the app.
    let notes = crate::search::with_read_conn(state.inner(), |conn| {
        let mut stmt = conn
            .prepare(
                "SELECT path, name, library_name, properties_json, tags_json, headings_json \
                 FROM note_meta",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(5)?.unwrap_or_default(),
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut out: Vec<NoteFacts> = Vec::new();
        for row in rows {
            let (path, title, library, props, tags, heads) = row.map_err(|e| e.to_string())?;
            let property_values = parse_property_values(&props);
            out.push(NoteFacts {
                property_keys: property_values.iter().map(|(k, _)| k.clone()).collect(),
                headings: parse_headings(&heads),
                tags: serde_json::from_str::<Vec<String>>(&tags).unwrap_or_default(),
                property_values,
                path,
                title,
                library,
            });
        }
        Ok(out)
    })?;

    Ok(discover_shapes(&notes, max_shapes.unwrap_or(40)))
}

/// `properties_json` → `(key, value)` pairs. A non-string value keeps its key (the key
/// is what SHAPES a note) but contributes no naming token, which is correct: a list or a
/// map is not a label.
fn parse_property_values(json: &str) -> Vec<(String, String)> {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .map(|o| {
            o.into_iter()
                .map(|(k, v)| (k, v.as_str().map(String::from).unwrap_or_default()))
                .collect()
        })
        .unwrap_or_default()
}

/// `headings_json` → heading texts. Tolerates both the object form (`{"text": …}`) and a
/// bare string array, because both shapes exist in the wild index.
fn parse_headings(json: &str) -> Vec<String> {
    serde_json::from_str::<Vec<serde_json::Value>>(json)
        .unwrap_or_default()
        .iter()
        .map(|h| {
            h.get("text")
                .and_then(|t| t.as_str())
                .map(String::from)
                .unwrap_or_else(|| h.as_str().unwrap_or("").to_string())
        })
        .filter(|h| !h.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Does this shape carry the given heading (by lowercase identity)?
    fn has_heading(s: &DiscoveredShape, text: &str) -> bool {
        s.headings.iter().any(|h| h.text == text)
    }

    fn note(path: &str, props: &[&str], heads: &[&str]) -> NoteFacts {
        NoteFacts {
            path: path.to_string(),
            property_keys: props.iter().map(|s| s.to_string()).collect(),
            headings: heads.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    /// A note that also carries the §4B naming inputs.
    fn named_note(
        path: &str,
        props: &[&str],
        library: &str,
        title: &str,
        tags: &[&str],
        values: &[(&str, &str)],
    ) -> NoteFacts {
        NoteFacts {
            path: path.to_string(),
            property_keys: props.iter().map(|s| s.to_string()).collect(),
            headings: Vec::new(),
            library: library.to_string(),
            title: title.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            property_values: values
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
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
        assert!(has_heading(&shapes[0], "life"));
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
        assert!(has_heading(s, "history"));
        assert!(!has_heading(s, "weather"), "a one-off heading is not the shape");
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

    // ── §4B naming ──────────────────────────────────────────────────────────

    /// 60 unrelated notes, so a lift measured against them means something.
    fn filler() -> Vec<NoteFacts> {
        (0..60)
            .map(|i| {
                named_note(
                    &format!("/Misc/n{i}.md"),
                    &["colour", "shape"],
                    "Misc",
                    &format!("The thing {i}"),
                    &["note"],
                    &[],
                )
            })
            .collect()
    }

    fn film_corpus() -> Vec<NoteFacts> {
        let mut notes = filler();
        for i in 0..10 {
            notes.push(named_note(
                &format!("/Film/History/m{i}.md"),
                &["country", "language"],
                "Film",
                &format!("The Matrix {i} (film)"),
                // The imported maintenance tag: unique to these notes, so its LIFT is
                // the highest in the kind — and it is junk.
                &["template-film-date-with-1-release-date", "note"],
                &[("content_type", "film")],
            ));
        }
        notes
    }

    fn named(shapes: &[DiscoveredShape], core: &[&str]) -> Option<ProposedName> {
        shapes
            .iter()
            .find(|s| s.core.iter().map(String::as_str).collect::<Vec<_>>() == core)
            .and_then(|s| s.proposed_name.clone())
    }

    /// THE HEADLINE CASE. Three candidates describe every note in the kind:
    ///   `film`    — library, a tag, the titles          → 3 families
    ///   `history` — the folder                          → 1 family
    ///   the maintenance tag — highest lift of the three → 1 family
    /// Ranking by frequency picks a stopword; ranking by lift picks the maintenance
    /// tag; ranking by CORROBORATION picks the name. This is the real Universe's
    /// film kind reproduced in miniature.
    #[test]
    fn corroboration_beats_lift_and_beats_frequency() {
        let shapes = discover_shapes(&film_corpus(), 20);
        let name = named(&shapes, &["country", "language"]).expect("the film kind is nameable");

        assert_eq!(name.name, "film");
        assert!(name.evidence.len() >= 2, "a name must be corroborated: {:?}", name.evidence);
        let fams: Vec<&str> = name.evidence.iter().map(|e| e.family.as_str()).collect();
        assert!(fams.contains(&"library") && fams.contains(&"tag"));
    }

    /// The folder is where a note was FILED, which is not what it IS. `history`
    /// covers 100% of the kind at the same lift as `film` — and loses, because
    /// nothing else in the user's material agrees with it.
    #[test]
    fn a_single_family_never_names_a_kind() {
        let name = named(&discover_shapes(&film_corpus(), 20), &["country", "language"]).unwrap();
        assert_ne!(name.name, "history");
        assert_ne!(name.name, "template-film-date-with-1-release-date");
    }

    /// No stopword list, in any language: `the` and `note` are on every note in the
    /// corpus, so their lift is 1.0 and they cannot clear the bar. This is the only
    /// mechanism that survives a fifteen-language Universe.
    #[test]
    fn corpus_universal_tokens_are_suppressed_without_a_stopword_list() {
        let shapes = discover_shapes(&film_corpus(), 20);
        for s in &shapes {
            if let Some(n) = &s.proposed_name {
                assert!(n.name != "the" && n.name != "note", "stopword proposed: {}", n.name);
            }
        }
    }

    /// When the corpus holds no name for a kind, we say so. This is the real
    /// `{born, died}` case: strong, repeated, single-family evidence and nothing
    /// that agrees with it.
    #[test]
    fn a_kind_with_no_name_in_the_corpus_proposes_nothing() {
        let mut notes = filler();
        for i in 0..10 {
            notes.push(named_note(
                &format!("/Misc/p{i}.md"),
                &["born", "died"],
                "Misc",           // same container as the filler — no signal
                &format!("Person {i}"),
                &["1889-deaths"], // one family, high lift, still not a name
                &[],
            ));
        }
        let shapes = discover_shapes(&notes, 20);
        assert!(
            named(&shapes, &["born", "died"]).is_none(),
            "single-family evidence must abstain, not guess"
        );
    }

    /// The library name is also a path segment. Counting it twice manufactures
    /// agreement out of one fact — on the real Universe that turned 8 honest
    /// results into 18 mostly-wrong ones.
    #[test]
    fn the_same_fact_seen_twice_is_not_corroboration() {
        let mut notes = filler();
        for i in 0..10 {
            notes.push(named_note(
                &format!("/Recipes/r{i}.md"), // path segment == library name
                &["cook_time", "servings"],
                "Recipes",
                &format!("Dish {i}"),
                &["note"],
                &[("folder", "Recipes")], // and a property that says it a third time
            ));
        }
        assert!(
            named(&discover_shapes(&notes, 20), &["cook_time", "servings"]).is_none(),
            "library + its own path segment + its own folder property is ONE family"
        );
    }

    /// Word-splitting assumes spaces between words. Japanese has none, so a
    /// token-only candidate set would make a library named 映画 unnameable.
    #[test]
    fn a_name_in_a_script_without_word_spaces_is_still_proposable() {
        let mut notes = filler();
        for i in 0..10 {
            notes.push(named_note(
                &format!("/映画/f{i}.md"),
                &["country", "language"],
                "映画",
                &format!("作品 {i}"),
                &["映画", "note"],
                &[],
            ));
        }
        let name = named(&discover_shapes(&notes, 20), &["country", "language"]).unwrap();
        assert_eq!(name.name, "映画");
    }

    /// The proposal is the user's own word, unaltered. Singularising "Recipes" to
    /// "Recipe" needs English morphology; applying it to fifteen languages produces
    /// nonsense, and inventing a word the user never wrote is forbidden outright.
    /// They can edit it — that is the whole contract.
    #[test]
    fn the_proposed_name_is_the_users_word_verbatim() {
        let mut notes = filler();
        for i in 0..10 {
            notes.push(named_note(
                &format!("/Kitchen/r{i}.md"),
                &["cook_time", "servings"],
                "Recipes",
                &format!("Recipes for dish {i}"),
                &["note"],
                &[],
            ));
        }
        let name = named(&discover_shapes(&notes, 20), &["cook_time", "servings"]).unwrap();
        assert_eq!(name.name, "recipes", "no stemming, no singularisation, no translation");
    }

    /// Every proposal carries the evidence that produced it, because a name with no
    /// visible basis is indistinguishable from one we invented.
    #[test]
    fn every_proposal_carries_checkable_evidence() {
        let name = named(&discover_shapes(&film_corpus(), 20), &["country", "language"]).unwrap();
        for e in &name.evidence {
            assert!(NAME_FAMILIES.contains(&e.family.as_str()));
            assert!(e.coverage_lb >= NAME_MIN_COVERAGE_LB);
            assert!(e.lift_lb >= NAME_MIN_LIFT_LB);
            // The UI renders these COUNTS, never the statistics.
            assert!(e.members_with > 0 && e.members_with <= e.members_total);
            assert!(e.corpus_with >= e.members_with, "the base rate includes the members");
            assert!(e.corpus_total >= e.corpus_with);
        }
        // Strongest evidence first, so the UI can show one line and be honest.
        assert!(name.evidence.windows(2).all(|w| w[0].lift_lb >= w[1].lift_lb));
    }

    /// The abstention floor is DERIVED, not chosen. At three members the Wilson lower
    /// bound of a perfect 3-of-3 is 0.438, below the 0.50 gate — so the smallest
    /// discoverable kind can never be named, however exclusive its evidence. Without
    /// this, three notes sharing an accident would be handed a confident name.
    #[test]
    fn the_smallest_kind_can_never_be_named_however_exclusive_its_evidence() {
        assert!(wilson_lb(3, 3) < NAME_MIN_COVERAGE_LB, "3/3 must not clear the gate");
        assert!(wilson_lb(4, 4) >= NAME_MIN_COVERAGE_LB, "4/4 must clear it");

        let mut notes = filler();
        for i in 0..3 {
            notes.push(named_note(
                &format!("/Zither/z{i}.md"),
                &["tuning", "strings"],
                "Zither",           // perfectly exclusive: 3 of 3, nothing else has it
                &format!("Zither {i}"),
                &["zither"],
                &[("kind", "zither")],
            ));
        }
        let shapes = discover_shapes(&notes, 20);
        assert_eq!(
            named(&shapes, &["strings", "tuning"]),
            None,
            "3 of 3 is 100% observed and still not enough evidence"
        );
    }

    /// Two kinds can want the same word. The bigger keeps it; the other is qualified
    /// by a key the user actually wrote — never by a number, which would make the
    /// name unique without making it recognisable.
    #[test]
    fn a_name_collision_is_qualified_by_the_users_own_key() {
        let mut notes = filler();
        for i in 0..12 {
            notes.push(named_note(
                &format!("/Film/a{i}.md"),
                &["country", "language"],
                "Film",
                &format!("Feature {i} film"),
                &["film"],
                &[],
            ));
        }
        for i in 0..8 {
            notes.push(named_note(
                &format!("/Film/b{i}.md"),
                &["country", "language", "cinematographer"],
                "Film",
                &format!("Short {i} film"),
                &["film"],
                &[],
            ));
        }
        let shapes = discover_shapes(&notes, 20);
        let names: Vec<String> = shapes
            .iter()
            .filter_map(|s| s.proposed_name.as_ref().map(|p| p.name.clone()))
            .collect();
        let uniq: HashSet<&String> = names.iter().collect();
        assert_eq!(names.len(), uniq.len(), "no two kinds may share a proposed name");
        for n in &names {
            assert!(!n.chars().last().unwrap().is_numeric(), "no numeric suffixes: {n}");
        }
    }

    /// Candidates that failed the bar are never offered — not as a proposal, and not
    /// as a chip. Every alternate the user can click has itself cleared all three gates.
    #[test]
    fn alternates_are_only_ever_candidates_that_passed() {
        for s in &discover_shapes(&film_corpus(), 20) {
            assert!(s.name_candidates.len() <= NAME_MAX_CANDIDATES);
            for c in &s.name_candidates {
                assert!(c.evidence.len() >= NAME_MIN_FAMILIES);
            }
            if s.name_candidates.is_empty() {
                assert!(s.proposed_name.is_none(), "no candidates means no proposal");
            }
        }
    }

    /// Run the whole engine over a REAL Universe and print what it proposes.
    ///
    /// Four designs for this engine passed their unit tests and were still wrong;
    /// each one died against real notes. So the real-data check is kept as a test
    /// rather than a throwaway script — ignored by default, run on demand:
    ///
    /// ```text
    /// CONSTELLATION_REAL_DB="…/.constellation/search.db" \
    ///   cargo test --lib template_discovery::tests::real_universe -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "needs a real Universe; set CONSTELLATION_REAL_DB"]
    fn real_universe_shapes_and_names() {
        let Ok(db) = std::env::var("CONSTELLATION_REAL_DB") else {
            eprintln!("set CONSTELLATION_REAL_DB to a search.db path");
            return;
        };
        let conn = rusqlite::Connection::open_with_flags(
            &db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .expect("open the Universe read-only");

        let t_read = std::time::Instant::now();
        let mut stmt = conn
            .prepare(
                "SELECT path, name, library_name, properties_json, tags_json, headings_json \
                 FROM note_meta",
            )
            .expect("note_meta");
        let notes: Vec<NoteFacts> = stmt
            .query_map([], |r| {
                let props: String = r.get::<_, Option<String>>(3)?.unwrap_or_default();
                let tags: String = r.get::<_, Option<String>>(4)?.unwrap_or_default();
                let heads: String = r.get::<_, Option<String>>(5)?.unwrap_or_default();
                let pv: Vec<(String, String)> =
                    serde_json::from_str::<serde_json::Value>(&props)
                        .ok()
                        .and_then(|v| v.as_object().cloned())
                        .map(|o| {
                            o.into_iter()
                                .map(|(k, v)| {
                                    (k, v.as_str().map(String::from).unwrap_or_default())
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                Ok(NoteFacts {
                    path: r.get(0)?,
                    property_keys: pv.iter().map(|(k, _)| k.clone()).collect(),
                    headings: serde_json::from_str::<Vec<serde_json::Value>>(&heads)
                        .unwrap_or_default()
                        .iter()
                        .map(|h| {
                            h.get("text")
                                .and_then(|t| t.as_str())
                                .map(String::from)
                                .unwrap_or_else(|| h.as_str().unwrap_or("").to_string())
                        })
                        .collect(),
                    library: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    title: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    tags: serde_json::from_str::<Vec<String>>(&tags).unwrap_or_default(),
                    property_values: pv,
                })
            })
            .expect("query")
            .filter_map(Result::ok)
            .collect();

        let t0 = std::time::Instant::now();
        println!("[read+parse] {:?} for {} notes", t_read.elapsed(), notes.len());
        let shapes = discover_shapes(&notes, 40);
        let elapsed = t0.elapsed();
        println!(
            "\n{} notes → {} kinds  [discovery {:?}]\n",
            notes.len(),
            shapes.len(),
            elapsed
        );
        let mut n_named = 0;
        for s in &shapes {
            let core = s.core.join(" · ");
            let spellings: Vec<&str> = s.fields.iter().take(4).map(|f| f.display.as_str()).collect();
            let heads: Vec<&str> = s.headings.iter().take(4).map(|h| h.display.as_str()).collect();
            let ex: Vec<&str> = s.examples.iter().take(2).map(|e| e.title.as_str()).collect();
            match &s.proposed_name {
                Some(p) => {
                    n_named += 1;
                    let ev: Vec<String> = p
                        .evidence
                        .iter()
                        .map(|e| {
                            format!(
                                "{} {}/{} of {}",
                                e.family, e.members_with, e.members_total, e.corpus_with
                            )
                        })
                        .collect();
                    println!("{:5}  {:44} → {:16} [{}]", s.support, core, p.name, ev.join(", "));
                }
                None => println!("{:5}  {:44} → (ask the user)", s.support, core),
            }
            if !spellings.is_empty() {
                println!("          fields: {:?}  headings: {:?}  eg: {:?}", spellings, heads, ex);
            }
        }
        println!("\nnamed {n_named}/{}", shapes.len());
        assert!(!shapes.is_empty(), "a real Universe must yield shapes");
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
