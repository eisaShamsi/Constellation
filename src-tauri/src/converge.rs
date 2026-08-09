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
///
/// PJ-207 §11 — `Serialize`, because this is what the repair door renders: the Settings
/// report shows each family's outcome verbatim, so a stamp-gated `Skipped` can never be
/// presented as a whole repair. Tagged (`kind` + `value`) so the frontend switches on
/// `kind` without parsing strings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
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
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConvergeReport {
    pub outgoing: ConvergeOutcome,
    pub incoming: ConvergeOutcome,
    pub sky: ConvergeOutcome,
    pub tag_counts: ConvergeOutcome,
    pub review: ConvergeOutcome,
    /// PJ-228 — the run gave way before it had asked for every family it wanted.
    /// Distinct from a family being `Skipped`: skipping is an ANSWER (the table is not
    /// in use), stopping is an ABSENCE of one. Only the second must keep a heal marker
    /// armed, which is why `all_ok()` cannot be the clear condition on its own.
    #[serde(default)]
    pub stopped: bool,
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
            stopped: false,
        }
    }

    /// The clear condition for a crash marker: every family the caller asked for
    /// answered, AND the run was not cut short.
    ///
    /// PJ-228 — `all_ok()` alone is not that condition. It is true for a run that gave
    /// way after one family, because the four it never reached are `Skipped`, and
    /// `Skipped` deliberately does not count as failure (an unstamped table is not in
    /// use; demanding `Converged` there would arm the marker forever). A background heal
    /// that yielded to a repair must therefore NOT clear the marker it was healing —
    /// otherwise the yield silently disarms the recovery it exists to perform.
    pub fn converged_fully(&self) -> bool {
        self.all_ok() && !self.stopped
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
    /// PJ-228 — asked BETWEEN families, never inside one.
    ///
    /// The families are absolute recomputes: `tag_counts` is one `DELETE` + one
    /// `INSERT … SELECT` in a single transaction, and the link families are single
    /// `UPDATE … SET col = (SELECT …)` statements. That is what makes stopping at a
    /// family boundary safe — every family is all-or-nothing, so a stopped run leaves
    /// each table either fully recomputed or completely untouched, never half-built.
    /// It is also what makes running this in the background survivable at all: a
    /// concurrent write-time ±delta either precedes the recompute's SELECT (and is
    /// counted) or follows its COMMIT (and applies to a correct base). **Do not make a
    /// family incremental without revisiting both properties.**
    ///
    /// `None` means "never stop", which is every pre-PJ-228 caller.
    pub stop: Option<&'a dyn Fn() -> bool>,
}

impl<'a> Ctx<'a> {
    /// The pre-PJ-228 context: a `.constellation` dir, and no way to be interrupted.
    pub fn new(constellation_dir: Option<&'a std::path::Path>) -> Self {
        Ctx { constellation_dir, stop: None }
    }

    fn should_stop(&self) -> bool {
        self.stop.map(|f| f()).unwrap_or(false)
    }
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

    // PJ-228 — the stop checkpoints. Between families ONLY: each family is an
    // all-or-nothing recompute, so a boundary is the one place a run can give way
    // without leaving a table half-built. `stopped` is what keeps the caller's crash
    // marker armed; the unreached families stay `Skipped(NOT_REQUESTED)`.
    macro_rules! checkpoint {
        () => {
            if ctx.should_stop() {
                report.stopped = true;
                return report;
            }
        };
    }

    // Before the FIRST family too: a stop already requested when the run begins (the app
    // is closing, a repair claimed the DB) must do no work at all, not one family's worth.
    checkpoint!();
    if want_outgoing {
        report.outgoing = match crate::links_backfill::recompute_all_outgoing(conn, _key) {
            Ok(n) => ConvergeOutcome::Converged(n),
            Err(e) => ConvergeOutcome::Failed(e.to_string()),
        };
    }

    checkpoint!();
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

    checkpoint!();
    if want_links {
        // Ungated and idempotent; a no-op on an empty sky_nodes (pre-backfill).
        report.sky = match crate::links_backfill::recompute_all_sky(conn, _key) {
            Ok(n) => ConvergeOutcome::Converged(n),
            Err(e) => ConvergeOutcome::Failed(e.to_string()),
        };
    }

    checkpoint!();
    if want_note_state {
        if crate::tag_counts::is_stamped(conn) {
            report.tag_counts = match run_tag_counts(conn, _key) {
                Ok(n) => ConvergeOutcome::Converged(n),
                Err(e) => ConvergeOutcome::Failed(e),
            };
        } else {
            report.tag_counts = ConvergeOutcome::Skipped(NOT_STAMPED);
        }

        checkpoint!();
        if crate::review::is_stamped(conn) {
            report.review = match ctx.constellation_dir {
                Some(dir) => {
                    let today = crate::review::today_str();
                    // PJ-207 §5 — owns its own windowed transactions; never wrap it.
                    // It also re-reads review-pulse.json per window (2026-08-08: this
                    // is a WRITE-BACK consumer of the pulse, so it must never be handed
                    // a snapshot that could be a degraded default, nor one that goes
                    // stale under a live ✓ Reviewed — see `recompute_all_in`'s doc).
                    match crate::review::recompute_all_in(conn, dir, &today, _key) {
                        Ok(n) => ConvergeOutcome::Converged(n),
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

/// COMMIT — and ROLL BACK if the COMMIT itself fails.
///
/// 2026-08-08 §11 inspection (HIGH). **SQLite leaves the transaction OPEN when a COMMIT
/// fails**; the application owes the explicit ROLLBACK. Four sites across this module
/// and the two `review*` modules returned the COMMIT error without one, and each had a
/// correct ROLLBACK on the arm right beside it — the error path everyone thinks about,
/// versus the one nobody does.
///
/// The worst instance sat under `review::with_busy_retry`: a busy COMMIT left the
/// transaction open, the retry re-ran `BEGIN IMMEDIATE`, and that failed with *"cannot
/// start a transaction within a transaction"* — a message the busy-matcher does not
/// recognise — so the pass returned Err with the transaction still open. On the boot
/// heal that connection is the one `init_db` is about to publish as `state.db`, which
/// would leave the app running its ENTIRE session inside an uncommitted transaction and
/// discarding every `search.db` write at exit. Per CLAUDE.md that file is today the only
/// home for the earned half of the Living Link Architecture, so "discarded at exit"
/// means traversal counts, weights, confidence promotions and archived links, gone,
/// with nothing surfaced.
///
/// One helper rather than four call-site fixes, so the next transaction added here
/// cannot quietly reintroduce the shape (the Whole-Ecosystem Fix Law).
pub(crate) fn commit_or_rollback(conn: &Connection) -> Result<(), String> {
    if let Err(e) = conn.execute_batch("COMMIT") {
        let _ = conn.execute_batch("ROLLBACK");
        return Err(e.to_string());
    }
    Ok(())
}

/// `tag_counts::recompute_all_in` needs its own transaction (it is a DELETE + INSERT
/// pair that must be atomic against the write-time ± delta — see §4).
fn run_tag_counts(conn: &Connection, key: &ConvergeKey) -> Result<usize, String> {
    conn.execute_batch("BEGIN IMMEDIATE").map_err(|e| e.to_string())?;
    match crate::tag_counts::recompute_all_in(conn, key) {
        Ok(n) => {
            commit_or_rollback(conn)?;
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
        &Ctx::new(constellation_dir),
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
/// **PJ-228 — it no longer runs at boot; it runs AFTER paint, and it can be stopped.**
///
/// The name changed with the behaviour. It used to be called from inside `init_db`,
/// before `state.db` was published, so every launch of a universe with an armed marker
/// waited on it (3,143 ms measured) with nothing on screen. It now runs on
/// `derived_heal`'s background thread with the status-bar strip, and `stop` is what lets
/// it give way to a repair — see [`ConvergeReport::converged_fully`] for why giving way
/// must NOT clear the caller's crash marker.
///
/// The key stays sealed in this module: `derived_heal` cannot construct a `ConvergeKey`,
/// so it comes through here, and there is still exactly one assembly.
pub fn heal_interrupted_walk(
    conn: &Connection,
    constellation_dir: Option<&std::path::Path>,
    stop: &dyn Fn() -> bool,
) -> ConvergeReport {
    converge_derived_views(
        conn,
        &ConvergeKey(()),
        Families::All,
        &Ctx { constellation_dir, stop: Some(stop) },
    )
}

/// After MIG-108's mass path rewrite: rows moved, links and content did not.
pub fn after_mig108(conn: &Connection) -> ConvergeReport {
    converge_derived_views(
        conn,
        &ConvergeKey(()),
        Families::OutgoingOnly,
        &Ctx::new(None),
    )
}

/// After the link vocabulary changed: the aggregates' SQL is built from the registry's
/// rank/type lists, so every link-derived value can shift without a single edge moving.
pub fn after_vocabulary_change(conn: &Connection) -> ConvergeReport {
    converge_derived_views(
        conn,
        &ConvergeKey(()),
        Families::LinksOnly,
        &Ctx::new(None),
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
        &Ctx::new(None),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-08-08 §11 inspection (HIGH). SQLite leaves the transaction OPEN when a
    /// COMMIT fails — the caller owes the ROLLBACK, and four sites here and in the two
    /// `review*` modules did not pay it. The consequence was not a lost window: the
    /// retry above one of them re-ran `BEGIN IMMEDIATE`, which failed with a message
    /// the busy-matcher does not recognise, and the connection — on the boot heal, the
    /// one about to be published as `state.db` — stayed transacted for the session.
    ///
    /// A COMMIT is made to fail portably with a DEFERRED foreign key: the violation is
    /// legal until COMMIT, which is exactly when SQLite refuses.
    #[test]
    fn a_failed_commit_leaves_no_transaction_open() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE parent (id INTEGER PRIMARY KEY);
             CREATE TABLE child (id INTEGER PRIMARY KEY, pid INTEGER
               REFERENCES parent(id) DEFERRABLE INITIALLY DEFERRED);",
        )
        .unwrap();

        conn.execute_batch("BEGIN IMMEDIATE").unwrap();
        conn.execute("INSERT INTO child (id, pid) VALUES (1, 999)", []).unwrap();
        assert!(!conn.is_autocommit(), "precondition: a transaction is open");

        let err = commit_or_rollback(&conn).expect_err("the deferred FK must refuse at COMMIT");
        assert!(
            err.to_lowercase().contains("foreign key"),
            "the COMMIT's own error is surfaced, not swallowed: {err}"
        );
        assert!(
            conn.is_autocommit(),
            "THE POINT: the failed COMMIT must leave the connection with no open transaction"
        );
        // And the connection is immediately usable again — the shape that broke was the
        // NEXT `BEGIN IMMEDIATE` failing with 'cannot start a transaction within a
        // transaction' and being mistaken for something other than a busy retry.
        conn.execute_batch("BEGIN IMMEDIATE").expect("a fresh transaction can start");
        conn.execute_batch("ROLLBACK").unwrap();
    }

    /// PJ-207 §11 inspection (HIGH, freeze-hang) — **how long is the boot heal, really?**
    ///
    /// `after_interrupted_walk_at_boot` runs all five families SYNCHRONOUSLY inside
    /// `init_db`, before `state.db` is published, with no `AppHandle` and therefore no
    /// progress surface. The inspection called it a freeze; the severity depends on a
    /// number nobody had measured, and a severity argued from a guess is not evidence.
    /// This measures it against a copy of a real universe.
    ///
    /// ```text
    ///   CONVERGE_TIME_DB="…/scratch-copy-of-search.db"     ///     cargo test --lib converge_boot_heal_cost -- --ignored --nocapture
    /// ```
    /// Unset → no-op, so a normal `cargo test` never touches a 180 MB file. It WRITES to
    /// the DB it is given (that is what converging is), so give it a COPY, never a live
    /// universe.
    #[test]
    #[ignore]
    fn converge_boot_heal_cost() {
        let Ok(db) = std::env::var("CONVERGE_TIME_DB") else {
            eprintln!("[converge-cost] CONVERGE_TIME_DB unset — skipping");
            return;
        };
        // Through the app's OWN initialiser, not a bare `Connection::open` — `init_db`
        // registers the custom `constellation` FTS tokenizer, without which three of the
        // five families abort on their first statement and the timing is meaningless.
        // This is also the faithful shape: `init_db` is exactly where the boot heal runs.
        let conn = crate::search::init_db(std::path::Path::new(&db)).expect("init the copy");
        let notes: i64 = conn
            .query_row("SELECT COUNT(*) FROM note_meta", [], |r| r.get(0))
            .unwrap_or(-1);
        let cdir = std::env::var("CONVERGE_TIME_CDIR").ok().map(std::path::PathBuf::from);

        let t = std::time::Instant::now();
        let report = converge_derived_views(
            &conn,
            &ConvergeKey::for_test(),
            Families::All,
            &Ctx::new(cdir.as_deref()),
        );
        let ms = t.elapsed().as_secs_f64() * 1000.0;

        eprintln!("[converge-cost] {} notes — ALL FIVE FAMILIES in {:.0} ms", notes, ms);
        for (family, outcome) in [
            ("outgoing", &report.outgoing),
            ("incoming", &report.incoming),
            ("sky", &report.sky),
            ("tag_counts", &report.tag_counts),
            ("review", &report.review),
        ] {
            eprintln!("[converge-cost]   {:<11} {:?}", family, outcome);
        }
    }

    /// PJ-228 — the distinction the background heal rests on. A run that gave way is
    /// NOT a run that succeeded, even though every family it never reached is `Skipped`
    /// and `Skipped` is deliberately not a failure. If the heal cleared its crash marker
    /// on `all_ok()`, yielding to a repair would silently disarm the very recovery the
    /// marker exists to trigger — and nothing would ever heal again, undetectably.
    #[test]
    fn a_stopped_run_is_not_a_converged_one_even_though_all_ok_is_true() {
        let mut r = ConvergeReport::new();
        r.outgoing = ConvergeOutcome::Converged(10);
        assert!(r.all_ok(), "unreached families are Skipped, which is not failure");
        assert!(r.converged_fully(), "a run that reached the end may clear the marker");

        r.stopped = true;
        assert!(r.all_ok(), "still 'no failures' — this is the trap");
        assert!(!r.converged_fully(), "but it must NOT clear the marker");

        // A failure keeps the marker armed whether or not the run was stopped.
        let mut f = ConvergeReport::new();
        f.tag_counts = ConvergeOutcome::Failed("boom".into());
        assert!(!f.all_ok());
        assert!(!f.converged_fully());
    }

    /// The stop closure is asked BETWEEN families, so a run that gives way leaves each
    /// table either fully recomputed or untouched — never half-built — and the families
    /// it never reached stay `Skipped`, not silently `Converged(0)`.
    #[test]
    fn stopping_happens_at_a_family_boundary_and_leaves_the_rest_unclaimed() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER, updated_at INTEGER);
             CREATE TABLE note_meta (path TEXT PRIMARY KEY, tags_json TEXT NOT NULL DEFAULT '[]',
               outgoing_count INTEGER NOT NULL DEFAULT 0, outgoing_link_types_json TEXT NOT NULL DEFAULT '{}',
               incoming_count INTEGER NOT NULL DEFAULT 0, incoming_link_types_json TEXT NOT NULL DEFAULT '{}');
             CREATE TABLE tag_counts (tag TEXT PRIMARY KEY, n INTEGER NOT NULL DEFAULT 0);
             CREATE TABLE note_links (source_path TEXT, target_name TEXT, link_type TEXT, status TEXT, target_name_lower TEXT);
             CREATE TABLE sky_nodes (path TEXT PRIMARY KEY, stratum TEXT);
             CREATE TABLE review_schedule (path TEXT PRIMARY KEY, reason TEXT, due_days INTEGER);",
        )
        .unwrap();

        // Stop immediately: not one family should run.
        let always = || true;
        let r = converge_derived_views(
            &conn,
            &ConvergeKey::for_test(),
            Families::All,
            &Ctx { constellation_dir: None, stop: Some(&always) },
        );
        assert!(r.stopped, "the run recorded that it gave way");
        assert!(!r.converged_fully());
        assert_eq!(r.outgoing, ConvergeOutcome::Skipped(NOT_REQUESTED), "never reached, never claimed");
        assert_eq!(r.review, ConvergeOutcome::Skipped(NOT_REQUESTED));

        // Never stop: the run completes and is allowed to clear a marker.
        let never = || false;
        let r2 = converge_derived_views(
            &conn,
            &ConvergeKey::for_test(),
            Families::All,
            &Ctx { constellation_dir: None, stop: Some(&never) },
        );
        assert!(!r2.stopped);
        assert!(r2.converged_fully(), "no failures and not stopped → may clear");
    }

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
            &Ctx::new(None),
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
            &Ctx::new(None),
        );
        assert_eq!(report.tag_counts, ConvergeOutcome::Skipped(NOT_REQUESTED));
        assert_eq!(report.sky, ConvergeOutcome::Skipped(NOT_REQUESTED));
        assert!(report.all_ok(), "skipping is not failing");
    }
}
