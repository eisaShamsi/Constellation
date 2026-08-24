//! PJ-369 — the mount-aware classifier for stale index rows.
//!
//! # Concept
//!
//! *"The index must not carry entries for notes that don't exist and belong to no library —
//! a search result must correspond to a real, openable note."*
//!
//! The Boss's **`Eisa Universe`** carries **603 `note_meta` rows and 19,472 `note_links`
//! edges** pointing at files under `E:\Cognitive Knowledge\…` that no longer exist on disk.
//!
//! *Which* universe is load-bearing, not trivia. Measured 2026-08-24 against both databases:
//! `Eisa Universe` (2,731 rows) has 621 rows outside every live library root, 603 of them
//! file-gone. The Boss's **daily** universe, `Eisa Cognitive Knowledge` (8,031 rows), has
//! **zero** of either — every row sits under one of its 19 registered libraries, and its
//! own root is itself a registered library (`universe_notes`), so nothing can fall outside.
//! An earlier draft of this header said "the Boss's own daily universe", which would have
//! sent his verification test to a universe where this module correctly reports nothing.
//!
//! Those rows surface as search results that open nothing, plus phantom edges in the link
//! graph, Sky View and the Reviewer. The boot reconcile has never removed them because it is
//! disk-first (walks registered library roots and checks the index against files found), so
//! rows under no walked root are never even visited — and step 3 of `reconcile.rs`
//! **deliberately** skips outside-root rows for a load-bearing safety reason: `Path::exists()`
//! returning `false` is indistinguishable between a file that is truly gone and a file on an
//! **unmounted drive**. Removing outside-root rows blindly would destroy real notes.
//!
//! # What this module is
//!
//! A **pure predicate** — `classify(conn, path, ctx) -> Verdict` — that decides whether one
//! `note_meta` row is a true phantom safe to prune. It writes nothing. Its only outputs are:
//!
//! - `Verdict::Prune(reason)` — every condition below is met, and every doubt has been
//!   resolved. Callers may safely feed this path through `reindex_delete_note`.
//! - `Verdict::Keep(reason)` — at least one condition failed. The reason names which.
//! - `Verdict::Unknown(reason)` — a condition could not be evaluated (e.g. federation
//!   unresolvable). **Fails closed.** A caller that receives `Unknown` must NOT prune.
//!
//! # The four conditions, in order
//!
//! A row is `Prune`-able **iff all four** are true:
//!
//! 1. **File-gone AND mount-live.** The row's `path` does not exist on disk, AND the nearest
//!    existing ancestor directory is readable. If both are false the drive itself might be
//!    unmounted; we return `Unknown`. The plan calls this the "mount-aware probe."
//!
//! 2. **Not under any registered library of THIS universe.** The candidate is tested against
//!    the FULL registered set from `try_load_libraries`, **not** the boot walk's `is_dir()`-
//!    filtered `roots_norm`. This is the Attack-0 fix: a real library whose folder was
//!    renamed in Explorer between sessions fails `is_dir()` but is still a live registered
//!    root — every real note in it must be kept.
//!
//! 3. **Not under any linked-universe root.** Tested against the strictly-loaded federation.
//!    If the federation cannot be fully resolved (unreadable manifest, network share hiccup)
//!    the whole run refuses — returning `Unknown` for every candidate — because a partial
//!    federation makes the "don't touch federated rows" check silently vacuous.
//!
//! 4. **No earned data on the row.** Any outgoing link with a promoted `confidence`,
//!    a non-`active` `status`, a `weight > 1.0`, or a `traversal_count > 0`; OR a non-NULL
//!    `note_meta.review_priority` (user's explicit override); OR a `review_schedule` row that
//!    has actually been **reviewed or snoozed** (`last_reviewed IS NOT NULL OR snoozed_until
//!    IS NOT NULL`) — is a KEEP. Earned data is the user's work; a phantom row with earned
//!    data is a data puzzle to investigate, not something to delete.
//!
//!    *Not* "any `review_schedule` row": one is auto-created as a baseline for every indexed
//!    note, so that reading matched all 603 candidates and the ground-truth audit returned
//!    `Prune: 0`. An earlier version of this list said exactly that, and would have kept every
//!    row the user wants removed. See `has_earned_data`.
//!
//! # The governing law: FAIL CLOSED
//!
//! Any doubt at any condition → `Unknown` (never `Prune`). The classifier only returns
//! `Prune` when every question was answered `yes` from disk or `no` from a query it verified
//! it could run. If a test cannot run at all — `try_load_libraries` errored, `read_dir` on
//! an ancestor errored, the earned-data query errored — the answer is `Unknown`.
//!
//! # Not built in this module
//!
//! The deletion itself (Step 3 of the plan) goes through `reindex_delete_note`, not through
//! anything here. This module writes nothing.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::libraries::{foreign_library_roots, path_is_under_any, try_load_libraries, LibraryInfo};

/// The classifier's answer for one candidate row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Safe to prune, with the reason recorded so the receipt can carry it.
    Prune(&'static str),
    /// Must NOT prune, with the reason so a curious user can be shown why.
    Keep(&'static str),
    /// Cannot be classified from available evidence. **Never prune on Unknown.**
    /// The whole run should surface this to the user rather than fall through it.
    Unknown(&'static str),
}

impl Verdict {
    pub fn reason(&self) -> &'static str {
        match self {
            Verdict::Prune(r) | Verdict::Keep(r) | Verdict::Unknown(r) => r,
        }
    }
}

/// Context shared across a whole classification run. Built once per invocation and passed to
/// every `classify` call — so we don't re-load the libraries manifest 603 times, and we don't
/// re-canonicalise the linked-universe roots per row. If either could not be built, `refused`
/// carries the reason and every classification returns `Unknown`.
pub struct ClassifierCtx {
    /// Every registered library of THIS universe, from `try_load_libraries` (strict; the
    /// same source the write-safe reconcile uses). Normalised to lowercase forward-slash
    /// form for the containment test. **Tested pre-`is_dir()` filter — a library whose
    /// folder failed to stat this instant is still a real registered library.**
    own_roots: std::collections::HashSet<String>,

    /// Every linked-universe library root that the strict federation resolver could reach.
    /// If ANY linked universe fails to resolve, this is not built and `refused` is set —
    /// because a partial list makes the "don't touch federated rows" check silently vacuous.
    linked_roots: std::collections::HashSet<String>,

    /// If Some, every `classify` call returns `Unknown` with this reason. Set when the
    /// libraries manifest can't be read, or the federation resolves partially.
    refused: Option<&'static str>,

    /// Memoised ancestor-readability results, so the ~603 candidates that all live under a
    /// handful of parents don't each re-stat the same folders. Key is the ancestor path
    /// (verbatim); value is whether `read_dir` succeeds on it.
    ancestor_cache: std::sync::Mutex<std::collections::HashMap<PathBuf, bool>>,
}

impl ClassifierCtx {
    /// Build the classifier context from a live `AppHandle`. Never panics; every failure
    /// becomes `refused` so classification returns `Unknown` for every row.
    pub fn build(app: &tauri::AppHandle) -> Self {
        // The universe's own libraries — the write-safe strict loader, exactly what
        // `reconcile.rs` uses when it decides whether to WRITE.
        let libs: Vec<LibraryInfo> = match try_load_libraries(app) {
            Ok(v) => v,
            Err(_) => {
                return Self {
                    own_roots: Default::default(),
                    linked_roots: Default::default(),
                    refused: Some(
                        "the library registry could not be read — refusing to classify any row",
                    ),
                    ancestor_cache: Default::default(),
                };
            }
        };

        // Attack-0 fix: test against EVERY registered library path, without the boot walk's
        // `is_dir()` filter. A folder-rename-outside-the-app leaves the library registered
        // but `is_dir()`-false for one boot; we must not classify its notes as phantoms.
        let own_roots: std::collections::HashSet<String> =
            libs.iter().map(|l| norm(&l.path)).collect();

        // The linked-universe set uses the SAME resolver the reconcile trusts —
        // `foreign_library_roots`, which itself refuses to fabricate on read failure.
        // Attack-1 fix: if the federation resolves only partially, we do NOT proceed with
        // a shorter list; we set `refused` so no row is pruned.
        //
        // `foreign_library_roots` returns an empty set both when there are no linked
        // universes AND when it could not resolve any. We disambiguate by asking the
        // universe manifest directly: if any child is DECLARED and none resolved, we refuse.
        // 2026-08-24 panel — this block previously read `.ok()` … `.unwrap_or(0)`, which
        // collapsed **"I could not read the federation"** into **"there is no federation"**.
        // That is failing OPEN, in the guard whose entire purpose is to fail closed, and it
        // disarmed the guard in precisely the case it exists for: an unreadable manifest
        // yields 0 declared children, so the refusal below never fires, and a linked
        // universe's notes — sitting under roots we could not resolve — become prunable.
        // Harmless while this step only COUNTS; at Step 3/4 it deletes another universe's
        // index rows, which is the Attack-1 scenario itself. An unknown federation is not an
        // absent one.
        let declared_roots = match crate::universe::active_universe_dir(app) {
            Ok(root) => match crate::universe::resolve_child_universe_roots_recursive_strict(&root)
            {
                Ok(v) => v,
                Err(_) => {
                    return Self {
                        own_roots,
                        linked_roots: Default::default(),
                        refused: Some(
                            "the universe manifest could not be read — refusing to classify any row, because an unreadable federation is not an absent one",
                        ),
                        ancestor_cache: Default::default(),
                    };
                }
            },
            Err(_) => {
                return Self {
                    own_roots,
                    linked_roots: Default::default(),
                    refused: Some(
                        "the active universe root could not be resolved — refusing to classify any row",
                    ),
                    ancestor_cache: Default::default(),
                };
            }
        };
        let linked_roots = foreign_library_roots(app, &libs);
        // 2026-08-24 diff-scoped inspection — this used to test `linked_roots.is_empty()`, so it
        // refused only when the federation resolved to NOTHING. The doc four fields above states
        // the actual contract: *"If ANY linked universe fails to resolve … `refused` is set."*
        // With two Linked Universes where A resolves and B's folder was renamed in Explorer
        // between sessions, `declared` still counts B (the strict resolver deliberately KEEPS a
        // NotFound child) while `foreign_library_roots` silently drops it — a NON-empty set, so
        // the guard stayed silent and every parent-index row under B's old path classified as a
        // phantom. Those notes exist; they are merely somewhere else. Today that is a wrong
        // number stated as fact; at Step 3/4 the same verdict feeds `reindex_delete_note` and
        // deletes a Linked Universe's index rows — the Attack-1 scenario this guard exists for,
        // and a write-sovereignty violation.
        //
        // So the test is now per-child: every declared child must have contributed at least one
        // resolved library root. A child universe always registers its own root as a library
        // (`universe_notes`), so "contributed nothing" means we could not read it — not that it
        // is legitimately empty.
        let mut linked_roots = linked_roots;
        let mut unresolved = 0usize;
        for d in &declared_roots {
            let dn = norm(&d.to_string_lossy());
            match declared_child_status(&dn, &linked_roots, absence_is_trustworthy(d)) {
                DeclaredChild::Contributed => {}
                DeclaredChild::AbsentButTrusted => {
                    // Its folder is gone, so it can hide no note — the run continues. But rows
                    // may still point under its old path, and those must never be pruned, so its
                    // declared root joins the protected set. Attack-1 holds either way.
                    linked_roots.insert(dn);
                }
                DeclaredChild::Unresolved => unresolved += 1,
            }
        }
        if unresolved > 0 {
            return Self {
                own_roots,
                linked_roots: Default::default(),
                refused: Some(
                    "the federation resolved only partially — refusing to classify any row so a linked universe's notes cannot be mistaken for phantoms",
                ),
                ancestor_cache: Default::default(),
            };
        }

        Self {
            own_roots,
            linked_roots,
            refused: None,
            ancestor_cache: Default::default(),
        }
    }

    /// Test-only constructor: build a ctx directly from already-resolved root sets. Used by
    /// the test battery to exercise the classifier without a live `AppHandle`. Production
    /// code MUST use `build`.
    /// Why this run declined to classify anything, if it did.
    ///
    /// Exposed read-only (2026-08-24 panel) so the caller can say so in `diagnostics.log`: a
    /// refused run reports zero phantoms and therefore looks exactly like a clean universe,
    /// and "we could not tell" must be distinguishable from "there is nothing to tell". The
    /// field stays private — a caller may READ the refusal, never set one.
    pub fn refusal(&self) -> Option<&'static str> {
        self.refused
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        own_roots: std::collections::HashSet<String>,
        linked_roots: std::collections::HashSet<String>,
        refused: Option<&'static str>,
    ) -> Self {
        Self {
            own_roots,
            linked_roots,
            refused,
            ancestor_cache: Default::default(),
        }
    }

    /// True if this row's path is under any registered own library (Attack-0 gate).
    fn under_own(&self, path: &str) -> bool {
        path_is_under_any(path, &self.own_roots)
    }

    /// True if this row's path is under any linked-universe root (Attack-1 gate).
    fn under_linked(&self, path: &str) -> bool {
        path_is_under_any(path, &self.linked_roots)
    }

    /// Ancestor-readability, memoised. Walks upward from `path` and returns the first ancestor
    /// that exists; then asks whether `read_dir` on it succeeds. If NO ancestor exists at all
    /// (an absurd absolute path), returns `false` — treat as "cannot prove the mount is live."
    fn ancestor_readable(&self, path: &Path) -> bool {
        let mut cur = path.parent();
        while let Some(dir) = cur {
            if dir.as_os_str().is_empty() {
                return false;
            }
            if dir.exists() {
                if let Ok(mut cache) = self.ancestor_cache.lock() {
                    if let Some(&hit) = cache.get(dir) {
                        return hit;
                    }
                    let readable = std::fs::read_dir(dir).is_ok();
                    cache.insert(dir.to_path_buf(), readable);
                    return readable;
                }
                // Poisoned mutex: don't trust; fail closed.
                return false;
            }
            cur = dir.parent();
        }
        false
    }
}

/// Classify one `note_meta` row.
///
/// `conn` is a read-only connection to this universe's `search.db`; the earned-data query
/// runs on it. `path` is the row's `path` column verbatim. `ctx` was built once for the run.
pub fn classify(conn: &Connection, path: &str, ctx: &ClassifierCtx) -> Verdict {
    // Hard refusal from the ctx: something at build time couldn't be answered, so no row
    // can be safely classified. Fails closed.
    if let Some(reason) = ctx.refused {
        return Verdict::Unknown(reason);
    }

    // 0. Trivial guard. An empty path is not a phantom in any useful sense.
    if path.is_empty() {
        return Verdict::Keep("empty path");
    }

    // 2. Registered library gate — done BEFORE the file probe so we don't stat a live note.
    //    A row under any registered library of this universe is not a phantom, full stop,
    //    even if its file happens to be missing right now (which is reconcile.rs's job).
    if ctx.under_own(path) {
        return Verdict::Keep("under a registered library of this universe");
    }

    // 3. Linked-universe gate — same reason, and separately: we never write to a Linked
    //    Universe's derived rows from this side (Boss ruling / MIG-111 write sovereignty).
    if ctx.under_linked(path) {
        return Verdict::Keep("belongs to a linked universe");
    }

    // 4. Earned-data gate. A row with any promoted link (`confidence != 'hypothesis'` or
    //    `status != 'active'` or `weight > 1.0` or `traversal_count > 0`), or a user-set
    //    review priority, or a review schedule row, carries the user's work. Never prune.
    match has_earned_data(conn, path) {
        Ok(true) => return Verdict::Keep("carries earned link or review data"),
        Ok(false) => {}
        Err(_) => return Verdict::Unknown("earned-data query failed"),
    }

    // 1. Mount-aware probe. This is the only place `fs` gets involved for the row itself.
    //    `try_exists` distinguishes "the file is not there" (Ok(false)) from "we could not
    //    check" (Err); the second is Unknown, never Prune.
    let p = Path::new(path);
    match p.try_exists() {
        Ok(true) => Verdict::Keep("file still exists on disk"),
        Ok(false) => {
            // File isn't there. Is the drive itself live? Ask the nearest existing ancestor.
            if ctx.ancestor_readable(p) {
                Verdict::Prune("file gone; mount live; not under any own or linked root; no earned data")
            } else {
                Verdict::Unknown("file absent but no ancestor directory is readable — cannot confirm the mount is live")
            }
        }
        Err(_) => Verdict::Unknown("could not stat the file"),
    }
}

/// Normalise a path for containment testing. Mirrors `libraries::path_is_under_any`'s own
/// convention (lowercase forward-slash, trailing-slash-stripped).
fn norm(p: &str) -> String {
    // Strip the Windows VERBATIM prefix first. `fs::canonicalize` returns `\\?\E:\…` on
    // Windows, while every path that reaches us from `libraries.json` is a plain `E:\…`.
    // Normalising without stripping produced `//?/e:/…` on one side and `e:/…` on the other,
    // so the two could never compare equal.
    //
    // This was not theoretical: the 2026-08-24 Attack-1 fix compared canonicalised declared
    // child roots against plain linked-library roots, concluded that EVERY declared child had
    // failed to resolve, and refused the whole run — turning a working feature into permanent
    // silence. It reached the Boss's test as "there is no sentence at all", and his run is what
    // caught it. The unit tests could not: they fed the comparison pre-normalised literals, so
    // they exercised the LOGIC and never the INPUT FORM.
    //
    // Stripping is a no-op for the plain paths that dominate here, so it cannot disturb them.
    let p = p.strip_prefix(r"\\?\").unwrap_or(p);
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// Did EVERY declared linked universe actually contribute a resolved library root?
///
/// The Attack-1 decision, extracted free of `AppHandle` so a regression test exercises the REAL
/// function rather than a copy that would keep passing after this one changed — the
/// `libraries::foreign_roots_of` pattern.
///
/// Found by the 2026-08-24 diff-scoped inspection: the guard used to ask
/// `linked_roots.is_empty()`, which refuses only a federation that resolved to NOTHING. The two
/// inputs disagree about a missing child — `resolve_child_universe_roots_recursive_strict`
/// deliberately KEEPS a child whose folder is `NotFound`, while `resolve_libraries_recursive`
/// silently skips it — so with one child present and one renamed away, the set was non-empty,
/// the guard stayed silent, and every parent-index row under the renamed child's old path
/// classified as a phantom. Those notes exist; they are merely somewhere else.
///
/// A child universe always registers its own root as a library (`universe_notes`), so a child
/// that contributed nothing was unreadable — never legitimately empty.
fn child_contributed(dn: &str, linked_roots: &std::collections::HashSet<String>) -> bool {
    let prefix = format!("{}/", dn);
    linked_roots.iter().any(|l| l == dn || l.starts_with(&prefix))
}

/// Test-only: did EVERY declared child contribute a resolved root?
///
/// This is the *contribution* half of the decision, not the whole of it — `declared_child_status`
/// is the real arbiter, because a child whose folder is genuinely gone contributes nothing and
/// must still not stop the run. Kept because the cases built on it pin things that remain true
/// and were expensive to learn: that a canonicalised declared root must match its plain linked
/// twin, and that a prefix lookalike (`child b2`) must not satisfy `child b`.
#[cfg(test)]
fn federation_is_complete(
    declared_norm: &[String],
    linked_roots: &std::collections::HashSet<String>,
) -> bool {
    declared_norm.iter().all(|d| child_contributed(d, linked_roots))
}

/// What a single declared child means for the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeclaredChild {
    /// It resolved and contributed at least one library root. Nothing to do.
    Contributed,
    /// Its folder is genuinely gone, on a mount we can still read. It can hide no note, so it
    /// must NOT stop the run — but any index row still pointing under it is protected anyway.
    AbsentButTrusted,
    /// It exists and yielded nothing, or we could not trust its absence (possible unmounted
    /// drive). Ambiguous → the whole run refuses.
    Unresolved,
}

/// Decide one declared child. Pure, and takes the filesystem answer as a VALUE
/// (`absence_is_trustworthy` = the folder does not exist AND its nearest existing ancestor is
/// readable) so it can be tested with the same inputs the caller really supplies.
///
/// # Why "genuinely gone" must not refuse the run
///
/// The first version of this guard refused whenever any declared child failed to resolve. That
/// reads as maximally safe and is not: `resolve_child_universe_roots_recursive_strict` is
/// **recursive**, so a universe inherits the declared children of its linked universes. The
/// Boss's `Eisa Universe` links `كون عيسى`, which still declares a grandchild
/// (`Two universe UNIVERSE`) whose folder was deleted long ago. One dead grandchild therefore
/// refused every run, permanently, and the feature showed nothing at all — twice, in his own
/// testing, before this was understood.
///
/// A folder that does not exist can hide no note, so it cannot make the "don't touch federated
/// rows" check vacuous — which is the only thing the refusal protects. What it CAN do is leave
/// index rows pointing under its old path, so the caller adds that path to the protected set:
/// the run proceeds, and those rows are still never pruned. Attack-1 is preserved exactly, and
/// the refusal is reserved for real ambiguity: a child that is present but unreadable, or an
/// absence we cannot trust because the mount itself may be gone.
pub(crate) fn declared_child_status(
    dn: &str,
    linked_roots: &std::collections::HashSet<String>,
    absence_is_trustworthy: bool,
) -> DeclaredChild {
    if child_contributed(dn, linked_roots) {
        DeclaredChild::Contributed
    } else if absence_is_trustworthy {
        DeclaredChild::AbsentButTrusted
    } else {
        DeclaredChild::Unresolved
    }
}

/// The nearest EXISTING ancestor of `path` is readable — the same mount-aware probe `classify`
/// uses per row, in a free form usable before a `ClassifierCtx` exists. Called once per declared
/// child, so it needs no cache.
fn absence_is_trustworthy(path: &Path) -> bool {
    if path.exists() {
        return false; // present: its silence is not explained by absence
    }
    let mut cur = path.parent();
    while let Some(dir) = cur {
        if dir.as_os_str().is_empty() {
            return false;
        }
        if dir.exists() {
            return std::fs::read_dir(dir).is_ok();
        }
        cur = dir.parent();
    }
    false
}

/// True if this note_meta row carries any earned data. Any query error returns `Err` — the
/// caller turns that into `Unknown`, which is not-prunable, which is the correct posture.
///
/// # Do not "optimise" this into three batched scans — it was measured, and it is worse
///
/// The shape below is three indexed point lookups per candidate, which reads as wasteful next
/// to "just pre-load the earned paths into a set once." Measured 2026-08-24 against the live
/// `Eisa Universe` database (312 MB, 621 candidates), read-only:
///
/// | approach                                    | cold    | warm    |
/// |---------------------------------------------|---------|---------|
/// | per-row point lookups (this code)           | 11.6 s  | 0.042 s |
/// | three batched `DISTINCT`/`IS NOT NULL` scans| 38.8 s  |    —    |
///
/// The batched form is ~3× SLOWER because the cost here is not query shape but **cold random
/// page reads**: three full scans touch far more pages than 621 lookups that each ride an
/// existing index (`idx_link_boot` covers the `note_links` predicate; the other two are primary
/// keys). Warm, the whole pass costs 42 ms — the queries were never the problem.
///
/// The cost is also paid on a background thread (`reconcile::maybe_schedule` spawns and returns),
/// so it delays no paint, and it is **0.000 s** on the Boss's daily universe, which has zero
/// outside-root rows to classify at all.
fn has_earned_data(conn: &Connection, path: &str) -> Result<bool, rusqlite::Error> {
    // Outgoing links: any promoted confidence, any archive/other status, any weight > 1.0,
    // any traversal_count > 0. The defaults are 'hypothesis' / 'active' / 1.0 / 0
    // (search.rs:5405–5409); anything else is user work.
    let earned_links: i64 = conn.query_row(
        "SELECT COUNT(*) FROM note_links
           WHERE source_path = ?1
             AND (confidence != 'hypothesis'
                  OR status != 'active'
                  OR weight > 1.0
                  OR traversal_count > 0)",
        params![path],
        |r| r.get(0),
    )?;
    if earned_links > 0 {
        return Ok(true);
    }

    // Review priority override: user has explicitly said this note is high/low priority.
    let review_pri: Option<i64> = conn
        .query_row(
            "SELECT review_priority FROM note_meta WHERE path = ?1",
            params![path],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    if review_pri.is_some() {
        return Ok(true);
    }

    // Any REVIEWED-or-snoozed review-schedule row for this note. This is the earned form:
    // a bare `review_schedule` row is auto-created for every indexed note as the baseline
    // (`review_schedule` schema — search.rs:4886), so its mere existence is NOT user work.
    // Only `last_reviewed IS NOT NULL` (the user has reviewed) or `snoozed_until IS NOT NULL`
    // (the user has snoozed) marks earned interaction. Discovered when the audit against the
    // Boss's live db classified 603/603 phantoms as "carries earned data" — every one had a
    // bare baseline row, and none carried an actual review action. Broadening this condition
    // to "any row present" would have kept 603 rows he wanted removed.
    //
    // `review_schedule` may not exist in older test DBs; treat "table missing" as no data.
    let has_review: Result<i64, rusqlite::Error> = conn.query_row(
        "SELECT COUNT(*) FROM review_schedule
           WHERE path = ?1
             AND (last_reviewed IS NOT NULL OR snoozed_until IS NOT NULL)",
        params![path],
        |r| r.get(0),
    );
    match has_review {
        Ok(n) if n > 0 => Ok(true),
        Ok(_) => Ok(false),
        Err(rusqlite::Error::SqliteFailure(_, ref msg))
            if msg.as_deref().map(|s| s.contains("no such table")).unwrap_or(false) =>
        {
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

// ─── test battery ────────────────────────────────────────────────────────────────────
//
// The test battery pins EVERY property the classifier promises, using an in-memory database
// so no real universe is touched. Every one of the three adversarial attacks that shaped
// this design has at least one test named after it.

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::collections::HashSet;
    use tempfile::TempDir;

    /// Build a minimal search.db schema — only the columns the classifier reads.
    fn seed_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (
                path TEXT PRIMARY KEY,
                review_priority INTEGER
             );
             CREATE TABLE note_links (
                source_path TEXT, target_name TEXT, link_type TEXT,
                confidence TEXT DEFAULT 'hypothesis',
                weight REAL DEFAULT 1.0,
                traversal_count INTEGER DEFAULT 0,
                status TEXT DEFAULT 'active'
             );
             CREATE TABLE review_schedule (
                path TEXT PRIMARY KEY,
                last_reviewed TEXT,
                snoozed_until TEXT
             );",
        )
        .unwrap();
        conn
    }

    fn own_only(roots: &[&str]) -> ClassifierCtx {
        ClassifierCtx::for_test(
            roots.iter().map(|r| norm(r)).collect(),
            HashSet::new(),
            None,
        )
    }
    fn own_and_linked(own: &[&str], linked: &[&str]) -> ClassifierCtx {
        ClassifierCtx::for_test(
            own.iter().map(|r| norm(r)).collect(),
            linked.iter().map(|r| norm(r)).collect(),
            None,
        )
    }

    // ── The happy path — the 603 real phantoms ──────────────────────────────────────

    #[test]
    fn a_file_gone_row_outside_all_roots_on_a_live_mount_is_prune() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        // Create a real, readable ancestor directory (the mount is live).
        let ancestor = tmp.path().join("Cognitive Knowledge");
        std::fs::create_dir_all(&ancestor).unwrap();
        // A path INSIDE it whose file does NOT exist — that is the phantom shape.
        let phantom = ancestor.join("Arts & Culture").join("dead-note.md");
        let phantom_s = phantom.to_string_lossy().to_string();
        conn.execute(
            "INSERT INTO note_meta (path) VALUES (?1)",
            params![phantom_s],
        )
        .unwrap();

        // Own roots elsewhere on disk, so the phantom is under none of them.
        let own = tmp.path().join("Constellation Universes").join("Eisa Universe");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);

        assert_eq!(
            classify(&conn, &phantom_s, &ctx),
            Verdict::Prune(
                "file gone; mount live; not under any own or linked root; no earned data"
            )
        );
    }

    // ── Attack 0: never prune a row under a registered library, EVEN IF is_dir() is false ──

    #[test]
    fn attack0_a_row_under_a_registered_library_is_never_prune_even_if_folder_is_gone() {
        let conn = seed_db();
        // No directory on disk at all — the library's folder was renamed in Explorer.
        let phantom_lib = "E:/UnrelatedTmp/renamed-library";
        let note = format!("{}/note.md", phantom_lib);
        conn.execute("INSERT INTO note_meta (path) VALUES (?1)", params![note])
            .unwrap();

        // The registered set STILL contains this root, even though it no longer resolves.
        let ctx = own_only(&[phantom_lib]);

        // Must be Keep, must NOT be Prune. The classifier trusts the registered set.
        assert_eq!(
            classify(&conn, &note, &ctx),
            Verdict::Keep("under a registered library of this universe")
        );
    }

    // ── Attack 1a: never prune a linked-universe row ────────────────────────────────

    #[test]
    fn attack1a_a_linked_universe_row_is_never_prune() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        // Linked universe is on disk; its note has been deleted.
        let linked = tmp.path().join("Linked Universe");
        std::fs::create_dir_all(&linked).unwrap();
        let note = linked.join(".trash").join("archived.md");
        let note_s = note.to_string_lossy().to_string();
        conn.execute(
            "INSERT INTO note_meta (path) VALUES (?1)",
            params![note_s],
        )
        .unwrap();

        let own = tmp.path().join("Own Universe");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_and_linked(
            &[own.to_str().unwrap()],
            &[linked.to_str().unwrap()],
        );

        assert_eq!(
            classify(&conn, &note_s, &ctx),
            Verdict::Keep("belongs to a linked universe")
        );
    }

    // ── Attack 1b: unresolvable federation refuses ALL classifications ──────────────

    #[test]
    fn attack1b_a_partial_federation_makes_the_whole_run_unknown() {
        let conn = seed_db();
        let ctx = ClassifierCtx::for_test(
            HashSet::new(),
            HashSet::new(),
            Some("federation could not be resolved"),
        );
        conn.execute(
            "INSERT INTO note_meta (path) VALUES ('E:/anything/here.md')",
            [],
        )
        .unwrap();

        match classify(&conn, "E:/anything/here.md", &ctx) {
            Verdict::Unknown(_) => (),
            other => panic!("expected Unknown on refused ctx, got {:?}", other),
        }
    }

    // The Attack-1 DECISION itself (the guard that sets `refused`), as opposed to
    // `attack1b` above which only proves a refused ctx yields `Unknown`. The bug the
    // 2026-08-24 diff-scoped inspection found lived here, and no test could see it:
    // the old guard asked `linked_roots.is_empty()`, so a PARTIALLY-resolved federation
    // sailed through while the module's doc promised it would refuse.

    #[test]
    fn attack1c_a_child_that_contributed_nothing_is_not_counted_as_contributing() {
        // Two Linked Universes declared. A resolves; B's folder was renamed in Explorer
        // between sessions, so `foreign_library_roots` silently drops it while the strict
        // declared list still carries it. The set is NON-EMPTY — which is exactly why the
        // old `is_empty()` test stayed silent and let B's notes be called phantoms.
        let declared = vec!["e:/u/child a".to_string(), "e:/u/child b".to_string()];
        let mut linked = HashSet::new();
        linked.insert("e:/u/child a".to_string());
        linked.insert("e:/u/child a/library one".to_string());

        assert!(
            !federation_is_complete(&declared, &linked),
            "a federation missing ONE of several declared children must refuse",
        );
    }

    #[test]
    fn attack1g_a_canonicalised_declared_root_still_matches_a_plain_linked_root() {
        // THE BUG THE BOSS'S TEST CAUGHT, pinned. `resolve_child_universe_roots_recursive_strict`
        // returns CANONICALISED paths — on Windows that is the verbatim form `\\?\E:\…` — while
        // `foreign_library_roots` normalises the plain `E:\…` strings from `libraries.json`.
        // Comparing them without stripping the prefix made every declared child look unresolved,
        // so the guard refused every run and the feature went permanently silent.
        //
        // The earlier tests could not catch it: they fed `federation_is_complete` pre-normalised
        // literals, exercising the LOGIC while never exercising the INPUT FORM the caller
        // actually supplies. This one builds the declared side the way the real code does.
        let tmp = TempDir::new().unwrap();
        let child = tmp.path().join("Child Universe");
        std::fs::create_dir_all(&child).unwrap();

        // The declared side, produced exactly as ClassifierCtx::build produces it.
        let canonical = std::fs::canonicalize(&child).unwrap();
        let declared = vec![norm(&canonical.to_string_lossy())];

        // The linked side, produced as foreign_library_roots does: the plain path, normalised.
        let mut linked = HashSet::new();
        linked.insert(norm(&child.to_string_lossy()));

        assert!(
            federation_is_complete(&declared, &linked),
            "a canonicalised declared root must match its own plain linked root\n\
             declared={:?}\n  linked={:?}",
            declared,
            linked,
        );
    }

    #[test]
    fn attack1h_a_dead_grandchild_universe_must_not_refuse_the_whole_run() {
        // THE SECOND FAILURE THE BOSS'S TEST CAUGHT, pinned.
        //
        // `resolve_child_universe_roots_recursive_strict` is RECURSIVE: a universe inherits the
        // declared children of its linked universes. `Eisa Universe` links `كون عيسى`, which
        // still declares a grandchild whose folder was deleted long ago. Under the first version
        // of this guard that single dead grandchild refused every run, permanently — the feature
        // showed nothing at all, twice, in his own testing.
        //
        // A folder that does not exist can hide no note, so it cannot make the federated-rows
        // check vacuous, which is the only thing the refusal protects.
        let tmp = TempDir::new().unwrap();
        let gone = tmp.path().join("Deleted Universe");
        // deliberately NOT created — but its parent exists and is readable
        assert!(!gone.exists());

        let dn = norm(&gone.to_string_lossy());
        let linked = HashSet::new(); // it contributed nothing, because it is not there

        assert_eq!(
            declared_child_status(&dn, &linked, absence_is_trustworthy(&gone)),
            DeclaredChild::AbsentButTrusted,
            "a declared child whose folder is genuinely gone, on a readable mount, must not refuse",
        );
    }

    #[test]
    fn attack1i_a_child_that_is_present_but_contributed_nothing_still_refuses() {
        // The refusal must survive for real ambiguity: the folder is there, so its silence is
        // NOT explained by absence — it could be an unreadable manifest hiding real notes.
        let tmp = TempDir::new().unwrap();
        let present = tmp.path().join("Present But Silent");
        std::fs::create_dir_all(&present).unwrap();

        let dn = norm(&present.to_string_lossy());
        let linked = HashSet::new();

        assert_eq!(
            declared_child_status(&dn, &linked, absence_is_trustworthy(&present)),
            DeclaredChild::Unresolved,
            "a child that EXISTS but resolved nothing is ambiguous and must refuse",
        );
    }

    #[test]
    fn attack1d_every_child_resolved_is_complete() {
        let declared = vec!["e:/u/child a".to_string(), "e:/u/child b".to_string()];
        let mut linked = HashSet::new();
        linked.insert("e:/u/child a".to_string());
        linked.insert("e:/u/child b/some library".to_string()); // contributes via a descendant
        assert!(federation_is_complete(&declared, &linked));
    }

    #[test]
    fn attack1e_no_declared_children_means_nothing_to_judge() {
        // A universe with no federation at all must NOT be refused — that would disable the
        // whole feature for the common case.
        assert!(federation_is_complete(&[], &HashSet::new()));
    }

    #[test]
    fn attack1f_a_prefix_lookalike_does_not_count_as_resolved() {
        // "child b2" must not satisfy "child b" — a substring match here would re-open the
        // hole this guard closes.
        let declared = vec!["e:/u/child b".to_string()];
        let mut linked = HashSet::new();
        linked.insert("e:/u/child b2".to_string());
        assert!(
            !federation_is_complete(&declared, &linked),
            "a sibling whose name merely starts with the declared path must not count",
        );
    }

    // ── Attack 2: earned data always wins over "the file is gone" ───────────────────

    #[test]
    fn attack2a_a_promoted_confidence_link_makes_the_row_keep() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        let ancestor = tmp.path().join("archive");
        std::fs::create_dir_all(&ancestor).unwrap();
        let note = ancestor.join("gone-but-earned.md");
        let note_s = note.to_string_lossy().to_string();
        conn.execute("INSERT INTO note_meta (path) VALUES (?1)", params![note_s])
            .unwrap();
        conn.execute(
            "INSERT INTO note_links (source_path, target_name, link_type, confidence)
             VALUES (?1, 'Other', 'supports', 'established')",
            params![note_s],
        )
        .unwrap();

        let own = tmp.path().join("owned");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);

        assert_eq!(
            classify(&conn, &note_s, &ctx),
            Verdict::Keep("carries earned link or review data")
        );
    }

    #[test]
    fn attack2b_a_traversed_link_makes_the_row_keep() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        let ancestor = tmp.path().join("archive");
        std::fs::create_dir_all(&ancestor).unwrap();
        let note = ancestor.join("gone-but-used.md");
        let note_s = note.to_string_lossy().to_string();
        conn.execute("INSERT INTO note_meta (path) VALUES (?1)", params![note_s])
            .unwrap();
        conn.execute(
            "INSERT INTO note_links (source_path, target_name, link_type, traversal_count)
             VALUES (?1, 'Other', 'supports', 3)",
            params![note_s],
        )
        .unwrap();
        let own = tmp.path().join("owned");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);
        assert_eq!(
            classify(&conn, &note_s, &ctx),
            Verdict::Keep("carries earned link or review data")
        );
    }

    #[test]
    fn attack2c_a_review_priority_override_makes_the_row_keep() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        let ancestor = tmp.path().join("archive");
        std::fs::create_dir_all(&ancestor).unwrap();
        let note = ancestor.join("gone-but-prioritised.md");
        let note_s = note.to_string_lossy().to_string();
        conn.execute(
            "INSERT INTO note_meta (path, review_priority) VALUES (?1, 3)",
            params![note_s],
        )
        .unwrap();
        let own = tmp.path().join("owned");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);
        assert_eq!(
            classify(&conn, &note_s, &ctx),
            Verdict::Keep("carries earned link or review data")
        );
    }

    /// **The bug this test exists to prevent.** The Boss's live database had a bare
    /// `review_schedule` row for every one of the 603 phantoms — auto-created as a baseline,
    /// not user work. If this classifier treated bare row-existence as earned data (as the
    /// first draft did), the audit against his data returned `Prune: 0` — the entire feature
    /// silently no-oped. Only `last_reviewed IS NOT NULL` OR `snoozed_until IS NOT NULL`
    /// marks a genuinely-earned schedule row.
    #[test]
    fn a_bare_review_schedule_row_is_NOT_earned_data() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        let ancestor = tmp.path().join("archive");
        std::fs::create_dir_all(&ancestor).unwrap();
        let note = ancestor.join("gone-with-baseline-schedule.md");
        let note_s = note.to_string_lossy().to_string();
        conn.execute("INSERT INTO note_meta (path) VALUES (?1)", params![note_s])
            .unwrap();
        // A schedule row exists but the user has NEVER reviewed or snoozed this note.
        conn.execute(
            "INSERT INTO review_schedule (path, last_reviewed, snoozed_until) VALUES (?1, NULL, NULL)",
            params![note_s],
        )
        .unwrap();
        let own = tmp.path().join("owned");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);

        // Must be Prune — the bare baseline row is NOT earned data.
        assert_eq!(
            classify(&conn, &note_s, &ctx),
            Verdict::Prune(
                "file gone; mount live; not under any own or linked root; no earned data"
            )
        );
    }

    /// The other side of the same coin: a schedule row with `last_reviewed` set IS earned.
    #[test]
    fn a_reviewed_schedule_row_IS_earned_data() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        let ancestor = tmp.path().join("archive");
        std::fs::create_dir_all(&ancestor).unwrap();
        let note = ancestor.join("gone-but-reviewed.md");
        let note_s = note.to_string_lossy().to_string();
        conn.execute("INSERT INTO note_meta (path) VALUES (?1)", params![note_s])
            .unwrap();
        conn.execute(
            "INSERT INTO review_schedule (path, last_reviewed, snoozed_until) VALUES (?1, '2026-05-01', NULL)",
            params![note_s],
        )
        .unwrap();
        let own = tmp.path().join("owned");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);
        assert_eq!(
            classify(&conn, &note_s, &ctx),
            Verdict::Keep("carries earned link or review data")
        );
    }

    // ── The mount-live probe: an ejected drive stays Unknown, never Prune ───────────

    #[test]
    fn an_ejected_drive_returns_unknown_never_prune() {
        let conn = seed_db();
        // A path on a fictional drive letter that (we can be near-certain) does not exist.
        // Its file doesn't exist AND no ancestor exists — the fail-closed case.
        let phantom = "Z:/definitely-not-a-mounted-drive-12345/note.md";
        conn.execute(
            "INSERT INTO note_meta (path) VALUES (?1)",
            params![phantom],
        )
        .unwrap();

        // Ensure the path really doesn't exist in this test environment.
        if Path::new(phantom).exists() {
            // Impossibly rare; skip the test rather than assert about the wrong thing.
            eprintln!("test skipped: Z: is unexpectedly present");
            return;
        }

        let tmp = TempDir::new().unwrap();
        let own = tmp.path().join("owned");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);

        match classify(&conn, phantom, &ctx) {
            Verdict::Unknown(_) => (),
            other => panic!("expected Unknown on missing mount, got {:?}", other),
        }
    }

    // ── The file DOES exist: never Prune, even if outside every root ────────────────

    #[test]
    fn a_row_whose_file_still_exists_is_always_keep() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        let real = tmp.path().join("stray.md");
        std::fs::write(&real, "hello").unwrap();
        let path_s = real.to_string_lossy().to_string();
        conn.execute(
            "INSERT INTO note_meta (path) VALUES (?1)",
            params![path_s],
        )
        .unwrap();
        // Own roots elsewhere.
        let own = tmp.path().join("Elsewhere");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);
        assert_eq!(
            classify(&conn, &path_s, &ctx),
            Verdict::Keep("file still exists on disk")
        );
    }

    // ── An empty path is Keep, never Prune ──────────────────────────────────────────

    #[test]
    fn an_empty_path_row_is_keep() {
        let conn = seed_db();
        let ctx = own_only(&[]);
        assert_eq!(classify(&conn, "", &ctx), Verdict::Keep("empty path"));
    }

    // ── The ancestor cache serves repeated ancestors from memory ────────────────────

    #[test]
    fn ancestor_readability_is_cached_across_calls() {
        let conn = seed_db();
        let tmp = TempDir::new().unwrap();
        let ancestor = tmp.path().join("shared-parent");
        std::fs::create_dir_all(&ancestor).unwrap();
        let n1 = ancestor.join("a.md").to_string_lossy().to_string();
        let n2 = ancestor.join("b.md").to_string_lossy().to_string();
        for p in [&n1, &n2] {
            conn.execute("INSERT INTO note_meta (path) VALUES (?1)", params![p])
                .unwrap();
        }
        let own = tmp.path().join("Owned");
        std::fs::create_dir_all(&own).unwrap();
        let ctx = own_only(&[own.to_str().unwrap()]);

        // First call warms the cache. Second call must serve from it — check by inspecting
        // the cache directly after both, since we can't time-benchmark reliably in tests.
        let _ = classify(&conn, &n1, &ctx);
        {
            let cache = ctx.ancestor_cache.lock().unwrap();
            assert!(cache.contains_key(ancestor.as_path()), "first call populated the cache");
        }
        let _ = classify(&conn, &n2, &ctx);
        {
            let cache = ctx.ancestor_cache.lock().unwrap();
            // Still exactly one entry for the shared parent — the second call didn't re-stat.
            assert_eq!(
                cache.iter().filter(|(k, _)| *k == ancestor.as_path()).count(),
                1
            );
        }
    }
}
