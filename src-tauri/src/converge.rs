//! PJ-207 §6 — **one place where derived views are rebuilt.**
//!
//! Constellation maintains five derived families write-time — a note's outgoing link
//! aggregates, the incoming (backlink) aggregates, Sky stratum/maturity, `tag_counts`,
//! and `review_schedule`. Each has a bulk recompute used when write-time maintenance
//! could not have run: after a full re-index, after an interrupted one, after a mass
//! path rewrite, after the link vocabulary changes.
//!
//! Before this module those recomputes were assembled **five different ways**:
//!
//! | Site | Families |
//! |---|---|
//! | `reconcile_filesystem`'s tail | 5 — outgoing, incoming, sky, tag_counts, review |
//! | `init_db`'s crash-marker healer | 3 — outgoing, incoming, sky |
//! | `mig108`'s post-move recompute | 1 — outgoing |
//! | `on_link_vocabulary_changed` | 2 — schedules two backfills |
//! | `incoming_links_backfill` | 1 — incoming |
//!
//! Five answers to "what does it mean for the derived views to be current." A sixth was
//! one careless commit away, and the differences were invisible: nothing in the code
//! said the boot healer runs three families while the reconcile tail runs five, so an
//! interrupted repair left `tag_counts` and `review_schedule` permanently stale with
//! nothing to notice (that gap is what §5's `derived_tail_pending` marker records, and
//! what [`after_interrupted_walk_at_boot`] now closes by running **all five**).
//!
//! ## Why a token, and why it is not decoration
//!
//! [`ConvergeKey`]'s field is a private unit, so **no code outside this module can
//! construct one** — and each of the five recomputes now requires a reference to it.
//! That is the enforcement: a sixth assembly cannot be written, because it cannot
//! obtain the argument.
//!
//! The obvious alternative — narrowing the recomputes to `pub(in crate::converge)` —
//! is not expressible in Rust: `pub(in path)` requires an ancestor module, and
//! `converge` is a sibling of `links_backfill`, `tag_counts` and `review`. Short of
//! moving all five function bodies in here (a far larger diff, and one that would
//! strand each family's tests away from its own module), the sealed token is the real
//! mechanism rather than a comment asking the next author to be careful.
//!
//! Tests construct one through [`ConvergeKey::for_test`], which exists only under
//! `cfg(test)` — in-crate tests keep exercising each family directly, while production
//! has exactly one door.
//!
//! ## The report is generated, never asserted
//!
//! [`ConvergeReport`] records, per family, what actually happened:
//! `Converged(n)` / `Skipped(reason)` / `Failed(msg)`. Three of the five are gated on a
//! back-fill stamp; those gates used to be silent `if` statements, so a run that
//! skipped three of five families was indistinguishable from one that ran all five.
//! They are now [`Skipped`](ConvergeOutcome::Skipped) with a reason — which is what
//! lets §11's repair report say what it did instead of claiming a whole repair.

use rusqlite::Connection;

/// Proof that a derived-view recompute is being driven by [`converge_derived_views`].
///
/// The unit field is private, so this type cannot be constructed outside this module.
/// Every bulk recompute takes one, which is what makes "there is exactly one assembly"
/// a fact the compiler enforces rather than a convention.
pub struct ConvergeKey(());

impl ConvergeKey {
    /// In-crate tests exercise each family's recompute directly — they are unit tests
    /// OF those families, not of the assembly. `cfg(test)` keeps this out of the
    /// shipped binary, so production still has exactly one door.
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        ConvergeKey(())
    }
}

/// Which derived families a caller needs brought current.
///
/// Named rather than boolean-per-family so a call site reads as an intent
/// ("everything a full walk invalidated") instead of a checklist someone can get
/// subtly wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Families {
    /// All five. What a full re-index — or the recovery from an interrupted one —
    /// invalidates.
    All,
    /// The three link-derived families: outgoing, incoming, sky. What a change to the
    /// link graph alone invalidates; note text, tags and review state are untouched.
    LinksOnly,
    /// Outgoing aggregates alone — a mass path rewrite that moved rows without
    /// changing any note's content or its links' targets.
    OutgoingOnly,
    /// Incoming aggregates alone — the §C.2a back-fill's own completion step.
    IncomingOnly,
}

/// What one family's recompute did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvergeOutcome {
    /// Ran, and touched `n` rows.
    Converged(usize),
    /// Deliberately not run, with the reason — never silently absent.
    Skipped(&'static str),
    /// Attempted and failed. Best-effort families do not abort the run, but a failure
    /// is never invisible.
    Failed(String),
}

impl ConvergeOutcome {
    pub fn is_failure(&self) -> bool {
        matches!(self, ConvergeOutcome::Failed(_))
    }
}

/// Per-family account of one convergence run.
#[derive(Debug, Clone)]
pub struct ConvergeReport {
    pub outgoing: ConvergeOutcome,
    pub incoming: ConvergeOutcome,
    pub sky: ConvergeOutcome,
    pub tag_counts: ConvergeOutcome,
    pub review: ConvergeOutcome,
}

/// The reason string for a family the caller did not ask for.
const NOT_REQUESTED: &str = "not requested";
/// The reason string for a family whose back-fill has not stamped yet — the table is
/// not in use, and the legacy live path is still the source of truth.
const NOT_STAMPED: &str = "back-fill not stamped";

impl ConvergeReport {
    fn new() -> Self {
        ConvergeReport {
            outgoing: ConvergeOutcome::Skipped(NOT_REQUESTED),
            incoming: ConvergeOutcome::Skipped(NOT_REQUESTED),
            sky: ConvergeOutcome::Skipped(NOT_REQUESTED),
            tag_counts: ConvergeOutcome::Skipped(NOT_REQUESTED),
            review: ConvergeOutcome::Skipped(NOT_REQUESTED),
        }
    }

    /// True when nothing the caller asked for failed. A `Skipped` family is not a
    /// failure — it is an answer.
    pub fn all_ok(&self) -> bool {
        !(self.outgoing.is_failure()
            || self.incoming.is_failure()
            || self.sky.is_failure()
            || self.tag_counts.is_failure()
            || self.review.is_failure())
    }

    /// Every family that failed, for a caller that has to report or retry.
    pub fn failures(&self) -> Vec<(&'static str, String)> {
        let mut out = Vec::new();
        for (name, o) in [
            ("outgoing", &self.outgoing),
            ("incoming", &self.incoming),
            ("sky", &self.sky),
            ("tag_counts", &self.tag_counts),
            ("review", &self.review),
        ] {
            if let ConvergeOutcome::Failed(m) = o {
                out.push((name, m.clone()));
            }
        }
        out
    }
}

/// What the review family needs and the others do not: the `.constellation` directory
/// holding `review-pulse.json`, and today's date. Only some callers have them.
pub struct Ctx<'a> {
    pub constellation_dir: Option<&'a std::path::Path>,
}

/// **The one body.** Bring the requested derived families current from `note_links` /
/// `note_meta`, reporting per family what happened.
///
/// Best-effort per family by design: a `tag_counts` failure must not prevent the review
/// schedule being rebuilt, and neither must abort a repair whose note index is already
/// correct. That is why each outcome is recorded rather than returned as an error —
/// the caller decides what a failure means for it.
pub fn converge_derived_views(
    conn: &Connection,
    _key: &ConvergeKey,
    families: Families,
    ctx: &Ctx<'_>,
) -> ConvergeReport {
    let mut report = ConvergeReport::new();

    let want_links = matches!(families, Families::All | Families::LinksOnly);
    let want_outgoing = want_links || matches!(families, Families::OutgoingOnly);
    let want_incoming = want_links || matches!(families, Families::IncomingOnly);
    let want_note_state = matches!(families, Families::All);

    if want_outgoing {
        report.outgoing = match crate::links_backfill::recompute_all_outgoing(conn, _key) {
            Ok(n) => ConvergeOutcome::Converged(n),
            Err(e) => ConvergeOutcome::Failed(e.to_string()),
        };
    }

    if want_incoming {
        // The stamp gate belongs to CONVERGENCE, not to the recompute.
        //
        // For a periodic self-heal (`All` / `LinksOnly`) it is right: before the §C.2a
        // back-fill stamps, the incoming columns are inert and reads fall back to
        // getBacklinks, so recomputing would write values nothing reads.
        //
        // For `IncomingOnly` it would be exactly wrong. That request comes from the
        // back-fill's own completion step — the caller that is ABOUT to stamp. The plan
        // said to route it through the gated path; doing so would have made the
        // back-fill a permanent no-op (it recomputes, THEN stamps, so `is_built` is
        // false at the moment it asks). A builder is not a healer.
        let gate_on_stamp = !matches!(families, Families::IncomingOnly);
        if !gate_on_stamp || crate::incoming_links_backfill::is_built(conn) {
            // Defensive: the back-fill builds this index; ensure it exists so the
            // recompute seeks. Normally a no-op.
            let _ = conn.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_nl_tnl ON note_links(target_name_lower, status);",
            );
            report.incoming = match crate::links_backfill::recompute_all_incoming(conn, _key) {
                Ok(n) => ConvergeOutcome::Converged(n),
                Err(e) => ConvergeOutcome::Failed(e.to_string()),
            };
        } else {
            report.incoming = ConvergeOutcome::Skipped(NOT_STAMPED);
        }
    }

    if want_links {
        // Ungated and idempotent; a no-op on an empty sky_nodes (pre-backfill).
        report.sky = match crate::links_backfill::recompute_all_sky(conn, _key) {
            Ok(n) => ConvergeOutcome::Converged(n),
            Err(e) => ConvergeOutcome::Failed(e.to_string()),
        };
    }

    if want_note_state {
        if crate::tag_counts::is_stamped(conn) {
            report.tag_counts = match run_tag_counts(conn, _key) {
                Ok(n) => ConvergeOutcome::Converged(n),
                Err(e) => ConvergeOutcome::Failed(e),
            };
        } else {
            report.tag_counts = ConvergeOutcome::Skipped(NOT_STAMPED);
        }

        if crate::review::is_stamped(conn) {
            report.review = match ctx.constellation_dir {
                Some(dir) => {
                    let pulse = crate::review::load_pulse_data(dir);
                    let today = crate::review::today_str();
                    // PJ-207 §5 — owns its own windowed transactions; never wrap it.
                    match crate::review::recompute_all_in(conn, &pulse, &today, _key) {
                        Ok(()) => ConvergeOutcome::Converged(0),
                        Err(e) => ConvergeOutcome::Failed(e),
                    }
                }
                // Honest rather than silent: the review family needs the universe's
                // `.constellation` dir to read review-pulse.json, and a caller that
                // does not have one (the boot healer works from a bare connection)
                // cannot converge it. Said, not skipped invisibly.
                None => ConvergeOutcome::Skipped("no constellation dir in context"),
            };
        } else {
            report.review = ConvergeOutcome::Skipped(NOT_STAMPED);
        }
    }

    report
}

/// `tag_counts::recompute_all_in` needs its own transaction (it is a DELETE + INSERT
/// pair that must be atomic against the write-time ± delta — see §4).
fn run_tag_counts(conn: &Connection, key: &ConvergeKey) -> Result<usize, String> {
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    match crate::tag_counts::recompute_all_in(conn, key) {
        Ok(n) => {
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
            Ok(n)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e.to_string())
        }
    }
}

// ─── The named entry points ──────────────────────────────────────────────────
//
// Each constructs the key internally. They exist so a call site states its INTENT —
// "after a repair run" — rather than picking a family set, which is exactly the choice
// that produced five divergent assemblies.

/// After a full walk: everything it invalidated.
pub fn after_repair_run(conn: &Connection, constellation_dir: Option<&std::path::Path>) -> ConvergeReport {
    converge_derived_views(
        conn,
        &ConvergeKey(()),
        Families::All,
        &Ctx { constellation_dir },
    )
}

/// At boot, when a walk is known to have died mid-flight.
///
/// **PJ-207 §5/§6 — this now runs all FIVE families, not three.** The pre-existing
/// healer ran outgoing/incoming/sky, because the marker it reads was written by the
/// trigger drop. But the same interrupted walk equally left `tag_counts` and
/// `review_schedule` unrecomputed, and those had no boot heal anywhere — so they stayed
/// stale until the next full repair, undetectably (the drift check compares `.md` files
/// to the index and cannot see a derived table stale against itself).
pub fn after_interrupted_walk_at_boot(
    conn: &Connection,
    constellation_dir: Option<&std::path::Path>,
) -> ConvergeReport {
    converge_derived_views(
        conn,
        &ConvergeKey(()),
        Families::All,
        &Ctx { constellation_dir },
    )
}

/// After MIG-108's mass path rewrite: rows moved, links and content did not.
pub fn after_mig108(conn: &Connection) -> ConvergeReport {
    converge_derived_views(
        conn,
        &ConvergeKey(()),
        Families::OutgoingOnly,
        &Ctx { constellation_dir: None },
    )
}

/// After the link vocabulary changed: the aggregates' SQL is built from the registry's
/// rank/type lists, so every link-derived value can shift without a single edge moving.
pub fn after_vocabulary_change(conn: &Connection) -> ConvergeReport {
    converge_derived_views(
        conn,
        &ConvergeKey(()),
        Families::LinksOnly,
        &Ctx { constellation_dir: None },
    )
}

/// The §C.2a incoming back-fill's own completion step — the initial BUILD of the
/// incoming aggregates, not a heal of them.
///
/// Deliberately ungated (see the gate note in [`converge_derived_views`]): this caller
/// recomputes and then stamps, so the stamp it would be gated on is the one it is about
/// to write.
pub fn after_incoming_backfill(conn: &Connection) -> ConvergeReport {
    converge_derived_views(
        conn,
        &ConvergeKey(()),
        Families::IncomingOnly,
        &Ctx { constellation_dir: None },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The gates that used to be invisible `if` statements now produce a REASON.
    /// Before this, a run that skipped three of five families was indistinguishable
    /// from one that ran all five — which is precisely how a partial repair got
    /// advertised as a whole one.
    #[test]
    fn an_unstamped_family_is_skipped_with_a_reason_never_converged_zero() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);
             CREATE TABLE note_meta (path TEXT PRIMARY KEY, tags_json TEXT NOT NULL DEFAULT '[]');
             CREATE TABLE tag_counts (tag TEXT PRIMARY KEY, n INTEGER NOT NULL DEFAULT 0);
             CREATE TABLE note_links (source_path TEXT, target_name TEXT, link_type TEXT, status TEXT, target_name_lower TEXT);
             CREATE TABLE review_schedule (path TEXT PRIMARY KEY, reason TEXT, due_days INTEGER);",
        )
        .unwrap();

        let report = converge_derived_views(
            &conn,
            &ConvergeKey::for_test(),
            Families::All,
            &Ctx { constellation_dir: None },
        );

        assert_eq!(
            report.tag_counts,
            ConvergeOutcome::Skipped(NOT_STAMPED),
            "an unstamped family must say WHY it did not run, never look like a run that found nothing",
        );
        assert_eq!(report.review, ConvergeOutcome::Skipped(NOT_STAMPED));
        assert_ne!(report.tag_counts, ConvergeOutcome::Converged(0));
    }

    /// A family the caller did not ask for is distinguishable from one that ran.
    #[test]
    fn a_family_not_requested_is_distinguishable_from_one_that_ran() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);
             CREATE TABLE note_meta (path TEXT PRIMARY KEY);
             CREATE TABLE note_links (source_path TEXT, target_name TEXT, link_type TEXT, status TEXT, target_name_lower TEXT);",
        )
        .unwrap();

        let report = converge_derived_views(
            &conn,
            &ConvergeKey::for_test(),
            Families::OutgoingOnly,
            &Ctx { constellation_dir: None },
        );
        assert_eq!(report.tag_counts, ConvergeOutcome::Skipped(NOT_REQUESTED));
        assert_eq!(report.sky, ConvergeOutcome::Skipped(NOT_REQUESTED));
        assert!(report.all_ok(), "skipping is not failing");
    }
}
