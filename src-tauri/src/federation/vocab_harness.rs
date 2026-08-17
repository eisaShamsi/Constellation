//! MIG-111 Phase 1.2 — **the H1 harness: one note, two vocabularies, aggregate VALUES diffed.**
//!
//! ## What this exists to catch
//!
//! `link_types::REGISTRY` is a process-global holding **the ACTIVE universe's** vocabulary
//! (`link_types.rs`), and **26 call sites across 11 files** read it through `snapshot()` — the
//! trigger DDL, the `index_note` parse chain, `maintain_incoming_after_save`, the sky write-time
//! maintenance, the rank-CASE and IN-list generators, both backfills' fingerprint gates.
//!
//! The moment the Router writes into a CHILD universe's database, every one of those computes with
//! the **parent's** vocabulary and stores the answer in the **child's** rows. The Architect's
//! adversarial pass named this H1, and named why it is nastier than it sounds:
//!
//! > the harness must diff aggregate VALUES, not just rows.
//!
//! Row counts survive a vocabulary mismatch untouched. The note still has one outgoing link; the
//! target still has one incoming. What changes is what those rows SAY — `link_type` collapses to
//! the null type, `incoming_link_types` loses a member, the typed aggregates disagree. A test that
//! counts rows reports health over a corrupted child universe, which is the same failure the whole
//! of Phase 0.4 kept teaching: **proving a property over the part you happened to look at.**
//!
//! ## What it gives Phase 1.2
//!
//! `aggregates_for` snapshots what a note's indexing actually PRODUCED — the values, not the
//! shape. `index_under_vocabulary` indexes through the real `init_db` + `index_note` with a chosen
//! vocabulary installed. Together they let 1.2 state its acceptance condition directly:
//!
//! * **RED today** — `two_vocabularies_disagree_on_the_same_note` proves the hazard is real: the
//!   identical note, indexed under two vocabularies, yields different aggregate VALUES while the
//!   row COUNTS stay identical. That is the bug, made visible before a line of routing exists.
//! * **GREEN when 1.2 lands** — a routed write into a child must produce the CHILD's values. The
//!   assertion is already written here (`routed_write_must_match_the_owners_vocabulary`), marked
//!   `#[ignore]` with the reason, and 1.2's definition of done is removing that attribute.
//!
//! ## H1b — what the harness found on its first run, before any routing existed
//!
//! The determinism check failed immediately: identical inputs, two different answers. The cause was
//! not the harness. `set_active` mutates a **process-global**, and `index_note` reads it at CALL
//! time — so a sibling test swapping the vocabulary mid-run changed what this one produced. Same
//! note, same connection, same database; different values.
//!
//! That rules out the obvious shape for 1.2 — *open the child's connection, `set_active` the
//! child's types, write, restore* — because the active universe's own debounced save, a backfill
//! tick, or the watcher lands inside that window and computes with whichever vocabulary is
//! installed at that instant. **A routed write must carry its vocabulary explicitly**, threaded
//! through the call or held per-connection, never by mutating shared state for a duration.
//! `a_vocabulary_swap_reaches_back_into_an_already_open_database` pins this so the tempting design
//! cannot pass unnoticed.
//!
//! The harness is committed BEFORE the implementation deliberately. A harness written afterwards
//! tends to describe what the code does; written first, it describes what the code must do — and
//! this one had already constrained the design before the design existed.

use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// What indexing a note actually produced — the VALUES, which is the whole point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aggregates {
    /// Rows only. Included so a test can SHOW that counts are equal while values differ —
    /// the trap this harness exists to expose.
    pub link_rows: i64,
    /// `(source, target, link_type)` for every edge, ordered — the values.
    pub edges: Vec<(String, String, String)>,
    /// `note_meta.incoming_count` per path, ordered.
    pub incoming_counts: Vec<(String, i64)>,
    /// `note_meta.incoming_link_types` per path, ordered — the aggregate a row count cannot see.
    pub incoming_types: Vec<(String, String)>,
}

/// Read the aggregates out of a database. Everything is ordered so two snapshots compare by
/// value rather than by whatever order SQLite happened to return.
pub fn aggregates_for(conn: &Connection, root: &Path) -> rusqlite::Result<Aggregates> {
    // Paths are recorded RELATIVE to the harness directory. Absolute temp paths differ between
    // runs, which would make every values-diff report a difference that is only the clock.
    let root_n = root.to_string_lossy().replace('\\', "/").to_lowercase();
    let strip = move |p: String| -> String {
        let n = p.replace('\\', "/").to_lowercase();
        n.strip_prefix(&format!("{root_n}/")).map(|s| s.to_string()).unwrap_or(n)
    };
    let link_rows: i64 = conn.query_row("SELECT COUNT(*) FROM note_links", [], |r| r.get(0))?;

    let mut st = conn.prepare(
        "SELECT source_path, target_name, COALESCE(link_type, '') FROM note_links \
         ORDER BY source_path, target_name, link_type",
    )?;
    let edges = st
        .query_map([], |r| Ok((strip(r.get(0)?), r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut st = conn.prepare(
        "SELECT path, COALESCE(incoming_count, 0) FROM note_meta ORDER BY path",
    )?;
    let incoming_counts = st
        .query_map([], |r| Ok((strip(r.get(0)?), r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut st = conn.prepare(
        "SELECT path, COALESCE(incoming_link_types, '') FROM note_meta ORDER BY path",
    )?;
    let incoming_types = st
        .query_map([], |r| Ok((strip(r.get(0)?), r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(Aggregates { link_rows, edges, incoming_counts, incoming_types })
}

/// Custom link-type deltas, as `link-types.json` would supply them.
pub fn deltas(ids: &[&str]) -> Vec<crate::link_types::LinkTypeDef> {
    ids.iter()
        .enumerate()
        .map(|(i, id)| crate::link_types::LinkTypeDef {
            id: (*id).to_string(),
            label: (*id).to_string(),
            parent: None,
            color: "#888888".to_string(),
            order: 100 + i as i64,
            builtin: false,
            emoji: None,
            desc: None,
            structural: false,
        })
        .collect()
}

/// Index `notes` into a fresh database with `vocabulary` installed, and return what it produced.
///
/// Drives the REAL `init_db` + `index_note`, not a stand-in: the point is to observe what the
/// production path computes when the vocabulary underneath it differs.
/// Serialises harness runs. `link_types::REGISTRY` is a **process-global**, so two runs installing
/// different vocabularies at once clobber each other — see `a_vocabulary_swap_reaches_back_into_an_
/// already_open_database` for why that is a finding about Phase 1.2 and not merely test hygiene.
pub(crate) static HARNESS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// **PJ-304 — restore the process-global on the way out, including on panic.**
///
/// This harness was committed to constrain Phase 1.2's design against LL-047, and it
/// introduced that very hazard into the suite. `set_active` was called and **never undone**,
/// so the first harness test to run left a 9-type vocabulary installed for *every subsequent
/// test in the process* — not a race window, permanent contamination, with test scheduling
/// order deciding whether it bit.
///
/// It bit two tests that assert the empty-sentinel rank `9` — which is
/// `cognitive_ids().len() + 1`, correct only for the seeds-only registry. A custom type makes
/// it 10. (The 1/2/4 ranks are unaffected: a custom type sorts *after* the seeds, so only the
/// sentinel moves.) Measured on pristine `main` at commit 857530f5, before any Phase 1.2
/// change: `links_backfill::tests::backfill_populates_existing_rows` and
/// `search::tests_mig066_outgoing::outgoing_aggregates_maintained_by_triggers`, failing
/// together roughly 1 run in 6.
///
/// Restoring on `Drop` is an interim measure and is honestly weaker than a fix: it shrinks the
/// exposure from "the rest of the process" to "the duration of this call", but a non-harness
/// test running concurrently can still read the mutated global. That residue is exactly what
/// LL-047 says cannot be closed while the vocabulary is ambient — **Stage A removes it
/// structurally**, by threading the vocabulary so this harness stops calling `set_active` at
/// all. Delete this guard then.
struct RestoreVocabulary;

impl Drop for RestoreVocabulary {
    fn drop(&mut self) {
        // Seeds-only is the registry's own default (`LinkTypeRegistry::seeds_only`, which
        // `cell()` initialises with), so this restores the state every other test assumes.
        crate::link_types::set_active(Vec::new());
    }
}

pub fn index_under_vocabulary(
    dir: &Path,
    vocabulary: Vec<crate::link_types::LinkTypeDef>,
    notes: &[(&str, &str)],
) -> rusqlite::Result<Aggregates> {
    let _serial = HARNESS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _restore = RestoreVocabulary;
    crate::link_types::set_active(vocabulary);
    let conn = crate::search::init_db(&dir.join("search.db")).expect("init_db");
    for (name, body) in notes {
        let path = dir.join(name);
        std::fs::write(&path, body).expect("write note");
        crate::search::index_note(&conn, &path.to_string_lossy(), "harness", true)
            .expect("index_note");
    }
    // ★ The maintenance pass, which `index_note` alone does not reach — and which is precisely
    // what H1 is about. `maintain_incoming_after_save` recomputes the incoming aggregates using
    // the vocabulary in force, so a routed write with the WRONG vocabulary lands its answer here.
    // An empty "old" signature means "everything about this note is new", forcing a full recompute.
    let empty = std::collections::HashSet::new();
    for (name, _) in notes {
        let path = dir.join(name);
        crate::search::maintain_incoming_after_save(&conn, &path.to_string_lossy(), &empty, "", &empty)
            .expect("maintain_incoming_after_save");
    }
    aggregates_for(&conn, dir)
}

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "constellation_mig111_h1_{}_{}",
        tag,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&d).expect("tmp dir");
    d
}

#[cfg(test)]
mod tests_mig111_12_h1_harness {
    use super::*;

    /// A note whose link uses a type only ONE of the two vocabularies knows.
    const NOTES: &[(&str, &str)] = &[
        ("Source.md", "A claim, and a link: [[refutes::Target|because of X]]\n"),
        ("Target.md", "The target.\n"),
    ];

    /// **THE HAZARD, MADE VISIBLE.** The identical note indexed under two vocabularies produces
    /// the same NUMBER of rows and different VALUES in them.
    ///
    /// This is why the plan demands values be diffed: after a routed write with the wrong
    /// vocabulary, every count in the child universe still looks right. `refutes` is a custom type
    /// in one universe and unknown in the other, so the SAME `[[refutes::Target]]` is a typed
    /// cognitive edge in one and collapses in the other — and only the values say so.
    #[test]
    fn two_vocabularies_disagree_on_the_same_note() {
        let with = index_under_vocabulary(&tmp_dir("with"), deltas(&["refutes"]), NOTES).unwrap();
        let without = index_under_vocabulary(&tmp_dir("without"), deltas(&[]), NOTES).unwrap();

        assert_eq!(
            with.link_rows, without.link_rows,
            "row COUNTS are identical — which is exactly why counting rows cannot detect this"
        );
        assert_ne!(
            (with.edges.clone(), with.incoming_types.clone()),
            (without.edges.clone(), without.incoming_types.clone()),
            "the VALUES must differ, or this harness is not observing the vocabulary at all\n\
             with:    {:?}\n{:?}\nwithout: {:?}\n{:?}",
            with.edges, with.incoming_types, without.edges, without.incoming_types
        );
    }

    /// **H1b — found by this harness, on its first run: the vocabulary is read at INDEX time from
    /// a process-global, so it is not a property of the database being written.**
    ///
    /// The determinism test failed on the first run with identical inputs, because a sibling test
    /// called `set_active` while this one was mid-index. Same note, same database, two answers.
    ///
    /// That is not test hygiene — it is a **constraint on Phase 1.2**. It rules out the obvious
    /// implementation of "use the owner's vocabulary": open the child's connection, `set_active`
    /// the child's types, write, `set_active` back. Any concurrent operation — the active
    /// universe's own debounced save, a backfill tick, the watcher — lands in the window and
    /// computes with whichever vocabulary happened to be installed. The corruption is silent and
    /// row counts stay right, so nothing surfaces it.
    ///
    /// **A routed write must therefore carry its vocabulary explicitly** (threaded through the
    /// call, or held per-connection) rather than by mutating shared global state for the duration.
    /// This test pins the hazard so that a future "just swap the global" cannot pass unnoticed.
    #[test]
    fn a_vocabulary_swap_reaches_back_into_an_already_open_database() {
        let _serial = HARNESS_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // PJ-304 — this test calls `set_active` directly, so it carries the same guard.
        let _restore = RestoreVocabulary;
        let dir = tmp_dir("swap");

        // Open the database with the FULL vocabulary in force — as a routed pool would, having
        // just installed the owner's types.
        crate::link_types::set_active(deltas(&["refutes"]));
        let conn = crate::search::init_db(&dir.join("search.db")).expect("init_db");
        for (name, body) in NOTES {
            std::fs::write(dir.join(name), body).expect("write note");
        }

        // Something else swaps the global before the write lands. Nothing about the connection,
        // the note, or the database changed.
        crate::link_types::set_active(deltas(&[]));

        for (name, _) in NOTES {
            let p = dir.join(name).to_string_lossy().to_string();
            crate::search::index_note(&conn, &p, "harness", true).expect("index_note");
        }
        let got = aggregates_for(&conn, &dir).unwrap();

        assert!(
            got.edges.iter().any(|(_, t, ty)| t.contains("refutes::") || ty == "associative"),
            "the swap DID reach the write — the vocabulary is read at index time, not owned by \
             the connection. If this ever fails, the coupling changed and 1.2's design premise \
             must be re-checked. Got: {:?}",
            got.edges
        );
    }

    /// The harness must be deterministic, or a values-diff means nothing.
    #[test]
    fn the_same_vocabulary_twice_gives_the_same_values() {
        let a = index_under_vocabulary(&tmp_dir("det_a"), deltas(&["refutes"]), NOTES).unwrap();
        let b = index_under_vocabulary(&tmp_dir("det_b"), deltas(&["refutes"]), NOTES).unwrap();
        assert_eq!(a, b, "the same inputs must produce the same aggregates");
    }

    /// **PHASE 1.2's ACCEPTANCE CONDITION, written before the code exists.**
    ///
    /// A routed write into a child universe must compute with the CHILD's vocabulary — so
    /// indexing the child's note through the Router must equal indexing it under the child's own
    /// vocabulary, and must NOT equal indexing it under the parent's.
    ///
    /// Ignored until the routed context pool lands. **Removing the `#[ignore]` is what "1.2 is
    /// done" means** — not a claim in a commit message.
    #[test]
    #[ignore = "MIG-111 Phase 1.2 — the routed context pool does not exist yet; this is its red→green"]
    fn routed_write_must_match_the_owners_vocabulary() {
        let child_vocab = deltas(&["refutes"]);
        let parent_vocab = deltas(&["exemplifies"]);

        let expected = index_under_vocabulary(&tmp_dir("child"), child_vocab, NOTES).unwrap();
        let wrong = index_under_vocabulary(&tmp_dir("parent"), parent_vocab, NOTES).unwrap();
        assert_ne!(expected, wrong, "the two vocabularies must genuinely differ for this to test anything");

        // TODO(1.2): index through the routed pool with the parent ACTIVE and the note owned by
        // the child, then:
        //     assert_eq!(routed, expected, "a routed write must use the OWNER's vocabulary");
        //     assert_ne!(routed, wrong,     "and never the active universe's");
        panic!("Phase 1.2 not implemented — this test is the acceptance condition, not a passing claim");
    }
}
