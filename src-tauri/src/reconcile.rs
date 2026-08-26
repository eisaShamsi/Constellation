//! MIG-078 §A′.2 — Reconcile `note_meta` against disk (File-Over-App self-heal).
//!
//! The Map/OrgChart tree is now assembled from `note_meta` (MIG-078 §A′), so any
//! row whose `.md` file no longer exists on disk shows up as a *phantom* note.
//! The old disk-walk masked these because it only emitted notes it found on disk;
//! reading the index directly exposes the drift. Such drift accumulates from
//! out-of-app changes (a rename/delete via Explorer, git, Syncthing) and from
//! historical bugs that left orphan rows.
//!
//! This module removes those stale rows in the background, after first paint,
//! using the SAME canonical de-index path a normal delete uses
//! (`reindex_delete_note` → drops `note_links` + `note_meta`, fires the FTS /
//! sky triggers, runs CTSE term cleanup). `.md` files on disk remain the source
//! of truth; a stale row is just an index entry pointing at a file that is gone,
//! and a future re-index re-adds any note that actually exists.
//!
//! Scheduled by `ensure_search_db_ready` (runs once per universe-open). Operates
//! only on the ACTIVE universe's `note_meta`; child universes self-heal when they
//! are themselves the active universe.
//!
//! **MIG-097 — rename-drift RELOCATE (2026-07-07).** A rename writes the file
//! immediately (gated) but updates the index in a *detached, best-effort* tail
//! (§B2-4, to avoid a freeze on large libraries). On a busy 2 GB library that
//! tail can be starved/lost, and because gated renames deliberately suppress the
//! watcher, nothing heals it — the row is left at the OLD (now-dead) path with
//! the OLD name, while the file lives at a NEW path with the SAME `cid_cn`.
//! Boss-reproduced 2026-07-07 (Reviewer rename → row reverted to old name on
//! reopen, opening it hit the dead path → empty Dashboard; disk was correct).
//! Removing the dead row (the MIG-078 behaviour) would drop the note — AND its
//! review history / links — from the index until a future reindex. So this pass
//! now first tries to **relocate** each dead row to its current file, matched by
//! the stable `cid_cn`, preserving the row's aux data; only rows whose note is
//! genuinely gone (no file with that cid) fall back to removal.
//!
//! **Safety (Working Agreement #4 — never ship a risky bulk DB mutation):**
//!   1. A row is a candidate ONLY if it sits under a library root that is
//!      *currently accessible* (the root directory exists). If a drive is
//!      unmounted at boot, that library's rows match no accessible root and are
//!      skipped — never mass-touched.
//!   2. A hard **safety cap**: if the candidate set exceeds 10 % of all rows or
//!      200 rows (whichever is larger), the pass ABORTS without touching anything
//!      and logs a warning. A transient sync glitch that hides many files cannot
//!      cause a catastrophic purge/relocate; the few-row steady-state heal runs.
//!   3. The disk existence checks + the orphan walk run **lock-free** (the DB
//!      mutex is released while statting), so the scan never blocks user IPC.
//!   4. Relocation never overwrites an existing row (orphans have none by
//!      definition; guarded anyway) and runs in a transaction (all-or-nothing).

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::thread;
use tauri::Manager;

use crate::search::{extract_frontmatter_cid_cn, reindex_delete_note, reindex_single_note, SearchState};

/// Abort the pass if more than this fraction of all rows look stale.
const MAX_STALE_FRACTION: f64 = 0.10;
/// …or more than this many absolute rows (whichever bound is larger).
const MAX_STALE_ABSOLUTE: usize = 200;

/// PJ-207 §9 — **what this pass found, in numbers, for the user instead of the log.**
///
/// Every count here is the RESIDUAL: what is still true after this pass has healed
/// whatever it could. A file this run re-adopted is not reported as missing, because by
/// the time anyone reads the notice it is not. What survives is exactly what needs the
/// repair door (§11) — which is the only thing worth interrupting the user for.
///
/// **The pass was already computing two of these and telling nobody.** `stale` and
/// `orphans` have been derived on every launch since MIG-078; the only trace was a line
/// in `diagnostics.log`. On the Boss's live universe that line reads *"825 orphan files
/// (> cap 200) — skipping re-adopt"* — 825 notes absent from search, detected four times
/// on 2026-08-07 alone, surfaced never. PJ-223 is not an undetected defect; it is an
/// unreported one.
///
/// The two directions are deliberately separate fields and not one "out of sync" number.
/// They have different causes and different cures — a file the index has never seen means
/// a library was never walked; a row whose file is gone means a note was deleted or moved
/// outside the app — and §3 spent a whole step un-flattening exactly this kind of
/// collapsed outcome (`IndexOutcome`, `WalkTally`). Re-flattening it one step later would
/// be the same mistake with a shorter memory.
#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftReport {
    /// The file and its row both exist, and the mtimes disagree — the note was edited
    /// while Constellation was closed (or by something that suppressed the watcher).
    /// Compared with `==`, exactly as `index_note`'s own gate does, so this counts
    /// precisely the notes a repair would re-read: an mtime that moved BACKWARDS (a
    /// restore from backup, a `git checkout`) is drift too, and the repair fixes it.
    pub drifted: usize,
    /// A `.md` under one of our own libraries with no row in the index at all. Named for
    /// what was observed, not for a history a stat cannot see: this pass cannot know
    /// whether the file was never indexed or whether a prior pass removed its row.
    pub missing_from_index: usize,
    /// A row under an ACCESSIBLE own root whose file is gone. Rows under an inaccessible
    /// root are not counted — an unmounted drive is not a deleted note, which is the same
    /// distinction the healing loop refuses to touch them for.
    pub missing_on_disk: usize,
    /// Rows whose path belongs to a **linked universe's** library — the Charter W2-9
    /// copies §8 stopped creating and §13 may one day offer to remove.
    ///
    /// Counted by PATH against the federated root set, never by `library_name NOT IN
    /// (own)`. On the Boss's `Eisa Universe` those two differ by 69×: 9 rows genuinely
    /// belong to a linked universe, while 621 merely sit outside the own roots — 603 of
    /// them pointing at `E:\Cognitive Knowledge\…`, the pre-MIG-108 location, where no
    /// file exists any more. Reporting 621 as "duplicated from linked universes" would be
    /// a fabrication, and it would be a fabrication in fifteen languages.
    pub foreign_rows: usize,
    /// Files whose row agreed with the disk. Carried so the closing sum is checkable from
    /// the REPORT rather than only from inside the walk — the identity is
    /// `files_seen == unchanged + drifted + missing_from_index + files_unreadable`.
    /// (`WalkReport` ships its `unchanged` for the same reason.)
    pub unchanged: usize,
    /// `.md` files the walk visited, and rows the snapshot read — the two denominators.
    /// Without them a wrong count is indistinguishable from a right one; with them the
    /// identity above is checkable, which is the discipline `WalkTally`'s closing sum
    /// exists for.
    pub files_seen: usize,
    pub rows_seen: usize,
    /// False when any directory failed to list or the depth cap truncated the walk. **A
    /// sweep that could not look must never report "nothing changed"** — the whole point
    /// of this migration is that silence stops meaning success.
    pub walk_complete: bool,
    /// Directories that could not be listed, and files that could not be stat'ed. The
    /// second is tracked separately from `walk_complete` on purpose: `walk_complete`
    /// gates dead-row REMOVAL, and one unreadable file must not disarm that.
    pub dirs_unreadable: usize,
    pub files_unreadable: usize,
    /// PJ-369 — rows this pass classified as **stale phantoms**: their file is gone, the
    /// mount is provably live, they sit under no registered library and no linked universe,
    /// and they carry no earned link or review data. On the Boss's `Eisa Universe` this is
    /// 603 rows dragging 19,472 link edges, all pointing into `E:\Cognitive Knowledge`
    /// (a separate legacy universe whose content now lives in a Linked Universe).
    ///
    /// On his DAILY universe (`Eisa Cognitive Knowledge`) it is **0** — measured 2026-08-24,
    /// not assumed: all 8,031 rows sit under one of its 19 registered libraries, and the
    /// universe root is itself a library (`universe_notes`), so no row can fall outside.
    /// A zero here is the honest answer, not a broken count.
    ///
    /// **Counted, never acted on.** This pass is disk-first and only *visits* rows under a
    /// walked root; these are found by a separate classifier sweep that writes nothing. The
    /// removal is user-offered from Settings → Index (PJ-369 Step 4), because deleting index
    /// rows is exactly the class of operation this project requires to be visible.
    ///
    /// Deliberately **NOT** part of `has_findings()` — see the note there.
    pub stale_phantoms: usize,
}

impl DriftReport {
    /// Is there anything worth telling the user about? An all-zero report renders
    /// nothing — never a green "all clear", which would be noise on every launch.
    ///
    /// §11 note: the EMIT no longer gates on this (`record_drift_report` always emits so
    /// a post-repair rescan can replace stale counts with a clean report). This remains
    /// the CANONICAL definition of a finding — the frontend's `hasFindings`
    /// (`driftReport.ts`) mirrors it as the live render gate; the two must agree.
    ///
    /// **"I could not look" counts as a finding.** The first version of this asked only
    /// about the three drift counts, and the 2026-08-07 safety inspection caught what that
    /// means: a library with one unlistable folder — an ACL-denied directory, a OneDrive
    /// placeholder, a subtree past the depth cap — yields all three at zero, because the
    /// notes under it were never *seen*. The report would then have been suppressed, and
    /// silence is this feature's encoding of "all clear". An entire subtree missing from
    /// search would have been reported as a clean launch, by the very check written to end
    /// that. It contradicted the sentence on `walk_complete` two fields up.
    ///
    /// Keyed on the COUNTS rather than on `!walk_complete`, deliberately: `walk_complete`
    /// is `false` in `DriftReport::default()`, so a `!` test would make every default-
    /// constructed value look like a finding.
    /// **PJ-369 — `stale_phantoms` is deliberately absent from this test.** The notice band
    /// this gates offers "Repair now", and a repair cannot fix a phantom: the repair walks
    /// libraries and re-reads files, while a phantom's whole nature is that it lives under no
    /// library and has no file. Including it here would put a button in front of the user
    /// that provably cannot act on the thing it appears beneath — the "false door" the
    /// PJ-369 design attack named. Phantoms get their own sentence and their own control in
    /// Settings → Index; this stays the definition of *repairable* findings.
    pub fn has_findings(&self) -> bool {
        self.drifted > 0
            || self.missing_from_index > 0
            || self.missing_on_disk > 0
            || self.dirs_unreadable > 0
            || self.files_unreadable > 0
    }

    /// PJ-369 — is there anything to tell the user about phantoms? Separate from
    /// `has_findings` precisely because the remedy is different (a user-offered prune in
    /// Settings, not "Repair now").
    pub fn has_phantoms(&self) -> bool {
        self.stale_phantoms > 0
    }
}

/// What one reconcile pass did, and what it found. The healed counts and the report are
/// returned together because the report is only truthful net of the healing: reporting
/// "5 notes are missing from the index" about five files this very pass re-adopted would
/// be an alarm about work already done.
#[derive(Default)]
pub(crate) struct ReconcileOutcome {
    pub relocated: usize,
    pub readopted: usize,
    pub removed: usize,
    /// `None` when the pass stopped before it had an answer — no accessible roots, an
    /// empty index, or a universe switch mid-pass. An absent report is not a clean one,
    /// and the caller must not render it as one.
    pub report: Option<DriftReport>,
}

/// The walk's accumulator. Five out-parameters threaded through a recursive function is
/// how the sixth gets forgotten; one struct is also what lets the drift comparison happen
/// at the single point where the file is already stat'ed.
struct Walk {
    /// `.md` files with no row — the relocate/re-adopt candidates, with their `cid_cn`.
    orphans: Vec<(String, String)>,
    /// Dedupes across overlapping roots (universe_notes at the root + a nested library).
    seen: HashSet<String>,
    complete: bool,
    files_seen: usize,
    drifted: usize,
    unchanged: usize,
    dirs_unreadable: usize,
    files_unreadable: usize,
}

/// `complete` starts **true** and is cleared by evidence, so `Default` is hand-written
/// rather than derived: `#[derive(Default)]` would start it `false`, and every one of the
/// ten construction sites would have to remember `complete: true`. That is the same
/// "somebody forgets the sixth one" failure the struct was introduced to end, moved to a
/// new address — and a forgotten `true` is silent and expensive: it disables dead-row
/// removal AND tells the user the sweep could not look everywhere.
impl Default for Walk {
    fn default() -> Self {
        Self {
            orphans: Vec::new(),
            seen: HashSet::new(),
            complete: true,
            files_seen: 0,
            drifted: 0,
            unchanged: 0,
            dirs_unreadable: 0,
            files_unreadable: 0,
        }
    }
}

fn norm(p: &str) -> String {
    p.replace('\\', "/").to_lowercase()
}

/// `true` when `path` sits at or under `root` (bounded at a separator, so
/// "…/Research" never matches "…/Research Notes"). Both args already normalized.
fn under(path_norm: &str, root_norm: &str) -> bool {
    path_norm == root_norm || path_norm.starts_with(&format!("{}/", root_norm))
}

/// Schedule the reconcile on a background thread. Returns immediately.
/// Called from `ensure_search_db_ready` after the connection is live.
pub fn maybe_schedule(app: tauri::AppHandle) {
    thread::spawn(move || {
    // 2026-08-24 panel — the generation as it stood when this pass was scheduled. `run`
    // gates its own returns on the same value, but a switch can also land in the window
    // between `run` returning and the emit below, and the emit is what the user SEES.
    // Captured here rather than threaded out of `run` so the check cannot be forgotten by
    // a future early return inside it: whatever `run` decides, nothing is surfaced unless
    // the universe is still the one that was active when we started.
    let generation_at_start = crate::search::federation_generation_now(&app);
    match run(&app) {
        Ok(outcome) => {
            if outcome.relocated > 0 || outcome.readopted > 0 || outcome.removed > 0 {
                diag(
                    &app,
                    &format!(
                        "[reconcile] healed index drift: {} relocated + {} re-adopted (by cid_cn), {} removed (note truly gone)",
                        outcome.relocated, outcome.readopted, outcome.removed
                    ),
                );
            }
            // PJ-207 §9 — hand the residual to the surface that can act on it. This is the
            // whole step: the numbers below have been computed on every launch for months
            // and gone nowhere but `diagnostics.log`.
            if let Some(report) = outcome.report {
                diag(&app, &format!(
                    "[reconcile] drift check: {} changed on disk, {} not in the index, {} rows without a file, {} from a linked universe ({} files / {} rows seen, walk {}{}{})",
                    report.drifted, report.missing_from_index, report.missing_on_disk, report.foreign_rows,
                    report.files_seen, report.rows_seen,
                    if report.walk_complete { "complete" } else { "INCOMPLETE" },
                    if report.dirs_unreadable > 0 { format!(", {} folder(s) unreadable", report.dirs_unreadable) } else { String::new() },
                    if report.files_unreadable > 0 { format!(", {} file(s) unreadable", report.files_unreadable) } else { String::new() },
                ));
                // PJ-369 — its own line, not appended to the drift sentence: a phantom is a
                // different finding with a different remedy, and burying it inside a line
                // whose subject is "repairable drift" is how it would be misread as one.
                if report.stale_phantoms > 0 {
                    diag(&app, &format!(
                        // No "offered for removal in Settings → Index" here: that control does
                        // not exist until Step 4, and the user-facing sentence had the same
                        // claim removed for the same reason. A log line that promises a door
                        // which is not built misleads whoever reads the log next — including me.
                        "[reconcile] {} stale index entr{} point at notes that no longer exist and belong to no library — counted only, nothing was changed",
                        report.stale_phantoms,
                        if report.stale_phantoms == 1 { "y" } else { "ies" },
                    ));
                }
                if crate::index_repair::walk_may_proceed(
                    false,
                    crate::search::federation_generation_now(&app),
                    generation_at_start,
                ) {
                    crate::index_repair::record_drift_report(&app, report);
                } else {
                    diag(&app, "[reconcile] universe switched before the report could be shown — discarded (its numbers describe the universe just left).");
                }
            }
        }
        Err(e) => diag(&app, &format!("[reconcile] FAILED (non-fatal): {}", e)),
    }
    });
}

/// Heal what can be healed, and report what cannot.
fn run(app: &tauri::AppHandle) -> Result<ReconcileOutcome, String> {
    // 1. Accessible library roots (name, path). If NONE are accessible (e.g. the
    //    universe drive is offline), do nothing — never touch rows on a bad mount.
    //
    //    PJ-207 §8 — the active universe's OWN libraries, never the federation-recursive
    //    set, and via the strict loader because this pass WRITES. Scoping the walk alone
    //    could not close Charter W2-9: this function draws its roots from the same list
    //    and RE-ADOPTS orphans at step 9, so removing a linked universe's rows without
    //    scoping here too would delete them and re-adopt them on every single launch —
    //    an oscillation that costs a ledger append and an fsync each time.
    //
    //    `?` rather than a silent empty list: an unreadable registry here previously
    //    produced zero roots, which reads as "nothing to reconcile" and returns success.
    let libs = crate::libraries::try_load_libraries(app)?;
    //    PJ-207 §8 (safety inspection, HIGH) — capture the universe generation up front.
    //    This pass runs on a spawned thread and holds NO guard of any kind: it computes
    //    its roots, its stale set and its orphan list from the universe that was active
    //    when it started, then writes through `state.db`, which a universe SWITCH
    //    replaces underneath it. The result is the very defect §8 exists to end, by a
    //    different door — the DEPARTED universe's `.md` files indexed into the NEWLY
    //    active universe's index. §7 built `federation_generation_now` and
    //    `walk_may_proceed` for exactly this and wired only the bulk walk to them; the
    //    boot reconcile was left behind. Same invariant (Architect Invariant 10: no write
    //    lands in a universe other than the one active when the run started), so it takes
    //    the same shared decision rather than a second copy of it.
    let generation = crate::search::federation_generation_now(app);
    let still_ours = || {
        crate::index_repair::walk_may_proceed(
            false, // this pass has no cancel channel; only the switch can stop it
            crate::search::federation_generation_now(app),
            generation,
        )
    };
    //    Roots belonging to a linked universe, for the orphan walk to SKIP. Narrowing the
    //    list above is not sufficient on its own — `collect_md` descends through every
    //    non-dot directory, so a cUniverse directory nested inside an own root (and
    //    `universe_notes`' root IS the Universe root) would still be reached, its files
    //    would still look like orphans, and step 9 would still re-adopt them.
    let foreign_roots = crate::libraries::foreign_library_roots(app, &libs);
    let roots: Vec<(String, String)> = libs
        .iter()
        .filter(|l| Path::new(&l.path).is_dir())
        .map(|l| (l.name.clone(), l.path.clone()))
        .collect();
    if roots.is_empty() {
        // No accessible root: nothing to reconcile, and nothing we could honestly report
        // either — every note is unreachable, which is a mount problem, not index drift.
        return Ok(ReconcileOutcome::default());
    }
    let roots_norm: Vec<String> = roots.iter().map(|(_, p)| norm(p)).collect();

    // 2. Snapshot (path, cid_cn, modified) under a brief lock, then release it.
    //    PJ-207 §9 — `modified` is the drift check's whole index-side input, and it is
    //    free here: this SELECT is already a full table scan (the `cid_cn` indexes are
    //    partial, so nothing covers it), and measured on the live database adding the
    //    column costs 25.0 ms against 26.0 ms without it. The row is faulted in either way.
    let state = app.state::<SearchState>();
    let rows: Vec<(String, String, u64)> = {
        let guard = state.db.lock().map_err(|e| e.to_string())?;
        let conn = guard.as_ref().ok_or("DB not initialized")?;
        let mut stmt = conn
            .prepare("SELECT path, COALESCE(cid_cn, ''), modified FROM note_meta")
            .map_err(|e| e.to_string())?;
        let r = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)? as u64))
            })
            .map_err(|e| e.to_string())?;
        r.flatten().collect()
    };
    let total = rows.len();
    if total == 0 {
        // Empty index — the initial reindex owns population. Reporting "every file on
        // disk is missing from the index" during a first run would be true and useless.
        return Ok(ReconcileOutcome::default());
    }
    let known: HashMap<String, u64> = rows.iter().map(|(p, _, m)| (norm(p), *m)).collect();

    // 3. Dead rows — LOCK-FREE per-path stat. Stat each note_meta path INDIVIDUALLY
    //    (never infer "dead" from a walk's completeness — a read_dir error on one
    //    subdir would then make its files look dead and get removed). Only rows
    //    under an accessible root are candidates (never touch a bad mount).
    let mut stale: Vec<(String, String)> = Vec::new();
    // MIG-112 §8 — rows whose note belongs to a universe NESTED inside this one. Collected in
    // the loop that already visits every row (no extra scan) and BEFORE `rows` is dropped.
    // `own_root` is the active universe's root — `universe_notes`' path IS that root. If it
    // cannot be resolved this stays empty and the pass below does nothing: a de-adoption driven
    // by a mis-resolved root would remove rows that ARE ours.
    let mig112_own_root: Option<std::path::PathBuf> = libs
        .iter()
        .find(|l| l.is_universe_notes)
        .map(|l| std::path::PathBuf::from(&l.path));
    let mut mig112_foreign: Vec<String> = Vec::new();
    // Memoised per DIRECTORY, not per note. `path_is_in_foreign_universe` walks a note's
    // ancestors doing up to two `fs::metadata` calls per level; called naively for all 8,031
    // rows of the daily universe that is tens of thousands of syscalls added to BOOT, which
    // CLAUDE.md's Rule 8 forbids ("no new feature may regress boot time"). Notes share
    // ancestors, so one verdict per distinct parent directory collapses it to a few hundred
    // checks. Bounded above by `own_root`: nothing at or above our own root can be foreign.
    let mut mig112_seen: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    // The roots the user has EXPLICITLY REGISTERED as libraries, excluding `universe_notes`
    // (whose path IS the universe root, so it claims everything and can discriminate nothing).
    //
    // 2026-08-26 safety inspection, CONFIRMED HIGH — this pass decided ownership from the
    // FILESYSTEM alone and never consulted `libraries.json`. A registered library whose folder
    // happens to sit inside another universe's directory would have had EVERY one of its notes
    // purged from `note_meta` and all eleven dependent tables at boot. `.md` files survive, but
    // `note_links.weight` / `traversal_count` / `last_traversed` / `confidence` / `status`,
    // `note_meta.review_priority` and `review_schedule` do NOT — CLAUDE.md's storage section
    // records `search.db` as their ONLY system of record, and `build_delete_archive` does not
    // carry them either. No live registry trips this today (all eight checked), but `add_library`
    // had no manifest check, so it was one "New Library" click away.
    //
    // **An explicit declaration beats a filesystem inference.** If the user registered it, it is
    // his, and this pass has no business removing it.
    let mig112_declared: Vec<String> = libs
        .iter()
        .filter(|l| !l.is_universe_notes)
        .map(|l| norm(&l.path))
        .collect();
    let mut foreign_rows = 0usize;
    // PJ-369 Step 2 — the phantom COUNT, computed inside the loop that already visits every
    // row so it costs no extra scan. Write-free: this only counts. The classifier context is
    // built once; if it cannot be built (unreadable registry, partially-resolved federation)
    // every classification returns `Unknown` and the count stays 0 — honest silence rather
    // than a number we cannot stand behind.
    let mut stale_phantoms = 0usize;
    // Rows the classifier declined to judge. Never surfaced as a count to the user — a doubt
    // is not a finding — but logged, so "the run refused" can be told apart from "there was
    // nothing to report" by anyone reading the diagnostics afterwards.
    let mut phantom_unknown = 0usize;
    let phantom_ctx = crate::phantom_prune::ClassifierCtx::build(app);
    // Deliberately the READ-ONLY reader, not a plain `Connection::open`: `SQLITE_OPEN_READ_ONLY`
    // + `query_only=ON` make "this counting pass cannot write" an invariant SQLite ENFORCES
    // rather than a promise the reviewer has to verify by reading every query. It also carries
    // the 5s `busy_timeout`, so a row is classified rather than silently skipped when the writer
    // holds the lock — without it the Boss would see a different number on every boot.
    // `.ok()` → `None` → the branch below never runs → count stays 0 → honest silence.
    let phantom_conn =
        crate::search::open_read_only_search_conn(&crate::search::db_path(app)?).ok();
    for (p, cid, _) in &rows {
        if p.is_empty() {
            continue;
        }
        let pn = norm(p);
        if !roots_norm.iter().any(|r| under(&pn, r)) {
            // PJ-207 §9 — outside every own root. Only the ones under a LINKED universe's
            // library are the Charter W2-9 class worth reporting; a row pointing at some
            // third place (an unmounted drive, a folder the user moved) is neither ours
            // nor theirs, and counting it as "duplicated from a linked universe" would
            // put a false number in front of the user. Still skipped for healing either
            // way — the `continue` below is unchanged and load-bearing (WA#4: never
            // mass-touch rows on a root we cannot see).
            if crate::libraries::path_is_under_any(p, &foreign_roots) {
                foreign_rows += 1;
            } else if let Some(pc) = phantom_conn.as_ref() {
                // PJ-369 — the rows this `continue` has always skipped are exactly the
                // phantom candidates: outside every own root AND not a linked universe's.
                // Classify (never act). Only a definite `Prune` verdict counts; `Keep` and
                // `Unknown` both leave the tally untouched, so a doubt is never a number.
                match crate::phantom_prune::classify(pc, p, &phantom_ctx) {
                    crate::phantom_prune::Verdict::Prune(_) => stale_phantoms += 1,
                    // 2026-08-24 panel — a REFUSED run and a genuinely clean universe both
                    // produced `0`, and rendered identically: no sentence. That makes "we
                    // could not tell" indistinguishable from "there is nothing to tell",
                    // which is the silence this whole migration exists to end. Counted here
                    // so the refusal is at least visible in `diagnostics.log`; the
                    // user-facing form belongs with Step 4's control and its receipt, and is
                    // recorded as owed rather than invented now.
                    crate::phantom_prune::Verdict::Unknown(_) => phantom_unknown += 1,
                    crate::phantom_prune::Verdict::Keep(_) => {}
                }
            }
            continue;
        }
        // MIG-112 §8 — a note inside a nested universe. Checked BEFORE the existence test:
        // these files DO exist, so the dead-row path can never see them, which is precisely why
        // they would otherwise stay frozen in this index forever.
        if let Some(root) = mig112_own_root.as_deref() {
            let parent = Path::new(p).parent().map(|d| d.to_path_buf());
            let foreign = match parent {
                None => false,
                Some(dir) => {
                    let key = norm(&dir.to_string_lossy());
                    match mig112_seen.get(&key) {
                        Some(v) => *v,
                        None => {
                            let v = crate::libraries::path_is_in_foreign_universe(&dir, root);
                            mig112_seen.insert(key, v);
                            v
                        }
                    }
                }
            };
            if foreign {
                // …unless the user DECLARED a library there. Checked on the row's own path, not
                // the memoised directory verdict, because the declaration is about this path.
                let pn = norm(p);
                let declared = mig112_declared
                    .iter()
                    .any(|r| pn == *r || pn.starts_with(&format!("{}/", r)));
                if declared {
                    diag(app, &format!(
                        "[reconcile] MIG-112: {} sits inside a nested universe but belongs to a REGISTERED library — kept. An explicit declaration beats a filesystem inference.",
                        p
                    ));
                } else {
                    mig112_foreign.push(p.clone());
                    continue;
                }
            }
        }
        if !Path::new(p).exists() {
            stale.push((p.clone(), cid.clone()));
        }
    }

    // 4. Orphan files — walk the accessible roots (lock-free) for `.md` files NOT in
    //    note_meta: the surviving half of a lost-tail rename whose dead row a prior
    //    reconcile already removed. Directory listing is cheap; frontmatter (the
    //    cid) is read only for orphans.
    let mut walk = Walk::default();
    for (_, root) in &roots {
        // Walk only TOP-LEVEL roots — skip a root nested under another (universe_notes
        // at the root + a sub-folder library): the parent walk already covers it, so
        // we don't read_dir the overlap twice. lib_for still attributes via ALL roots.
        let rn = norm(root);
        if roots.iter().any(|(_, other)| { let on = norm(other); on != rn && under(&rn, &on) }) {
            continue;
        }
        collect_md(Path::new(root), &known, &foreign_roots, &mut walk, 0);
    }
    let orphans = std::mem::take(&mut walk.orphans);
    let walk_complete = walk.complete;

    // PJ-207 §9 — the report as the walk found it. Healing below subtracts from it, so
    // what finally reaches the user is the residual.
    let report = DriftReport {
        drifted: walk.drifted,
        missing_from_index: orphans.len(),
        missing_on_disk: stale.len(),
        foreign_rows,
        unchanged: walk.unchanged,
        files_seen: walk.files_seen,
        rows_seen: total,
        walk_complete,
        dirs_unreadable: walk.dirs_unreadable,
        files_unreadable: walk.files_unreadable,
        stale_phantoms,
    };
    // 2026-08-24 panel — say so when the classifier declined. A refusal reports `0` phantoms
    // and therefore renders exactly like a clean universe; without this line there is no way,
    // afterwards, to tell which of the two happened. `refused` is whole-run (a partial
    // federation, an unreadable manifest); `phantom_unknown` is per-row.
    if phantom_unknown > 0 || phantom_ctx.refusal().is_some() {
        diag(app, &format!(
            "[reconcile] phantom classification INCOMPLETE — {} row(s) undecided{}. The phantom count below is a floor, not a total.",
            phantom_unknown,
            phantom_ctx.refusal().map(|r| format!("; run refused: {}", r)).unwrap_or_default(),
        ));
    }
    // The row snapshot is finished with — `known` holds the only part still needed, and
    // `rows` is ~1.5 MB of paths that would otherwise live through the walk (seconds, on a
    // cold disk) and the whole write phase below.
    let row_count = rows.len();
    drop(rows);

    // 4b. MIG-112 §8 — DE-ADOPT notes that belong to a universe nested inside this one.
    //
    //     Steps 1-7 stop the app REACHING them; nothing removes the rows already written, and
    //     without this pass those rows are strictly worse off than before the fence: every walk
    //     now skips them, the watcher's arms drop them, and step 8 above only removes rows whose
    //     FILE IS MISSING — which these are not. They would sit frozen at whatever body text
    //     they had, serving stale answers to search, Quick Switcher and backlinks forever, with
    //     no drift notice and no repair receipt. (2026-08-26 safety inspection, CONFIRMED.)
    //
    //     **POSITION IS LOAD-BEARING — this ran as step 8b and was DEAD CODE.** The second
    //     inspection pass caught it: two unconditional returns sit below (`stale.is_empty() &&
    //     orphans.is_empty()`, and the stale safety cap), and the state this pass exists to fix
    //     is exactly the state that takes the first of them. A nested-universe row can never
    //     enter `stale` — the MIG-112 arm in step 3 `continue`s before the `!exists()` test, and
    //     those files DO exist — and can never enter `orphans`, because `collect_md` skips the
    //     nested root via the same predicate. So whenever `mig112_foreign` is non-empty, both
    //     other sets get nothing from it, the clean-drift return fires, and the de-adopt never
    //     runs. It must therefore sit ABOVE those returns; it depends on neither set.
    //
    //     **No `.md` is deleted or touched.** The note keeps living in its own universe; only
    //     this universe's claim on it goes. Routed through `reindex_delete_note` because that is
    //     the only thing that clears all eleven dependent tables.
    let mut de_adopted = 0usize;
    {
        let foreign_rows_here = &mig112_foreign;

        // FAIL CLOSED on an implausible sweep. This pass can only ever fire on notes physically
        // inside a nested universe root, which is a rare accident (the MIG-108 unification made
        // three of them on the Boss's machine and that hole is closed) — so a large set means
        // the root resolved wrongly, not that the user has thousands of nested notes. Refuse and
        // say so rather than act on a number we cannot stand behind.
        let cap = std::cmp::max(50, row_count / 20);
        if foreign_rows_here.len() > cap {
            diag(app, &format!(
                "[reconcile] MIG-112 de-adopt REFUSED: {} of {} rows resolve to a nested universe (cap {}) — that is implausible, so the universe root is more likely wrong than the rows. Nothing removed.",
                foreign_rows_here.len(), row_count, cap
            ));
        } else {
            for p in foreign_rows_here {
                if !still_ours() {
                    // Same discipline as every other write loop here, and the same reason:
                    // a de-adopt landing after a switch would remove a row from a universe
                    // that was never the one this pass measured. No report either — its
                    // numbers describe the universe just left.
                    diag(app, "[reconcile] universe switched mid de-adopt — stopping.");
                    return Ok(ReconcileOutcome::default());
                }
                match reindex_delete_note(
                    &state,
                    p,
                    crate::search::DeleteCtx::new(crate::search::DeleteReason::ForeignUniverse),
                ) {
                    Ok(_) => de_adopted += 1,
                    // Never silent: a failed de-adopt leaves a frozen row, which is the exact
                    // state this pass exists to end.
                    Err(e) => diag(app, &format!("[reconcile] MIG-112 de-adopt failed for {}: {}", p, e)),
                }
            }
            if de_adopted > 0 {
                diag(app, &format!(
                    "[reconcile] MIG-112: de-adopted {} row(s) belonging to a nested universe — no .md file was touched.",
                    de_adopted
                ));
            }
        }
    }


    if stale.is_empty() && orphans.is_empty() {
        // Existence drift is clean — but MTIME drift may not be, and it is invisible to
        // this pass's healing (nothing at boot re-reads a changed file; that is the whole
        // premise of PJ-207). Returning early here without the report is how the check
        // would have stayed silent on the ONLY universe state it was written to catch.
        //
        // 2026-08-24 panel — but it must still be OUR report. The write-phase gate below
        // already refuses to surface a departed universe's numbers ("the same cross-universe
        // contamination §8 exists to prevent, in the notice instead of the index"); this
        // return published them anyway, and it is the path a universe with no stale or
        // orphan rows takes on EVERY boot. Concretely: the pass starts in a universe with
        // phantoms, the user switches to a clean one while it is still running, and the
        // clean universe displays the departed one's count. That is the wrong-universe
        // error this project has already made on paper this week; the software must not
        // make it too.
        if !still_ours() {
            diag(app, "[reconcile] universe switched mid-pass — no report (its numbers describe the universe just left).");
            return Ok(ReconcileOutcome::default());
        }
        return Ok(ReconcileOutcome { report: Some(report), ..Default::default() });
    }

    // 5. Safety caps (WA#4) — a suspiciously large set in EITHER direction means a
    //    transient mount/sync or a mid-initial-index race, not steady-state drift.
    let cap = MAX_STALE_ABSOLUTE.max((total as f64 * MAX_STALE_FRACTION) as usize);
    if stale.len() > cap {
        diag(app, &format!("[reconcile] ABORTED: {} of {} rows look stale (> cap {}). Refusing to touch — offline drive or sync in progress.", stale.len(), total, cap));
        // PJ-207 §9 — the report SURVIVES the abort, and this is the case that most needs
        // it. Refusing to act is correct here (WA#4), but refusing to act silently is what
        // left the Boss's 825 missing notes reported to a log file and nowhere else. The
        // cap decides what this pass may TOUCH; it does not decide what the user may KNOW.
        //
        // 2026-08-24 panel — same universe gate as the clean-drift return above. "What the
        // user may know" still means what they may know ABOUT THE UNIVERSE THEY ARE IN.
        if !still_ours() {
            diag(app, "[reconcile] universe switched mid-pass — no report (its numbers describe the universe just left).");
            return Ok(ReconcileOutcome::default());
        }
        return Ok(ReconcileOutcome { report: Some(report), ..Default::default() });
    }

    // 6. cid_cn → orphan path (first wins), for relocating a STILL-present dead row
    //    onto its current file.
    let mut orphan_by_cid: HashMap<String, String> = HashMap::new();
    for (p, cid) in &orphans {
        if !cid.is_empty() {
            orphan_by_cid.entry(cid.clone()).or_insert_with(|| p.clone());
        }
    }
    let mut consumed: HashSet<String> = HashSet::new(); // orphan paths taken by a relocate

    // 7. Relocate each dead row whose cid_cn has a live orphan file (preserves the
    //    row's aux data — review history, links). A relocate that FAILS is LEFT for
    //    next boot — NEVER falls to remove: falling to remove would destroy exactly
    //    the aux relocate exists to preserve, for a note that still exists. [audit]
    let mut relocated = 0usize;
    let mut relocate_failed = 0usize;
    let mut remove: Vec<String> = Vec::new();
    // Every write phase below is gated: the scan above was lock-free and can have taken
    // seconds on a large universe, which is ample room for a switch.
    if !still_ours() {
        diag(app, "[reconcile] universe switched mid-pass — writing nothing (the stale/orphan sets belong to the departed universe).");
        // No report either: every number in it describes the universe the user has just
        // left. Surfacing it against the newly-active one would be the same cross-universe
        // contamination §8 exists to prevent, in the notice instead of the index.
        return Ok(ReconcileOutcome::default());
    }
    for (dead, cid) in &stale {
        // Empty-cid rows have no identity to relocate by, so they land in
        // `remove`. PJ-153 (MIG-105 C6): the init_db boot healer now INJECTS
        // cid_cn into every knowledge note that lacks one (and it runs before
        // this reconcile — proven boot order), so the only rows that can still
        // arrive here empty are kind-template rows (empty BY DESIGN, MIG-TPL
        // §1 — a mold's identity IS its content; remove + re-adopt is lossless
        // for it) and genuinely-deleted notes. A knowledge note can no longer
        // be dropped here for lacking an identity.
        let target = if cid.is_empty() { None } else { orphan_by_cid.get(cid).cloned() };
        match target {
            Some(new_path) => {
                let res = {
                    let guard = state.db.lock().map_err(|e| e.to_string())?;
                    let conn = guard.as_ref().ok_or("DB not initialized")?;
                    relocate_row(conn, dead, &new_path)
                };
                match res {
                    Ok(()) => {
                        let np = norm(&new_path);
                        // Reindex the new path to refresh name/body (re-locks internally,
                        // so it runs AFTER the relocate lock is released).
                        if let Some(lib_name) = lib_for(&roots, &np) {
                            let _ = reindex_single_note(&state, &new_path, lib_name);
                        }
                        consumed.insert(np); // this orphan is the relocated row — don't re-adopt it
                        relocated += 1;
                    }
                    Err(e) => {
                        // PJ-151 (2026-07-26): this arm discarded the error for ~3 weeks
                        // while asserting "target busy/contended" — wrong in 100% of the
                        // 1,591 logged cases (live data shows NO row at any target path).
                        // Surface the REAL error so the failing class can be named; keep
                        // the dead row + its aux for retry next boot. Never fall to remove.
                        relocate_failed += 1;
                        if relocate_failed <= 20 {
                            let kind = match e {
                                // relocate_row's two sentinels, distinguished so the log
                                // says WHICH invariant stopped the heal.
                                rusqlite::Error::InvalidQuery => "target OCCUPIED (guard)",
                                rusqlite::Error::StatementChangedRows(0) => {
                                    "cascade moved NOTHING (see [migrate_note_db_paths] lines above)"
                                }
                                _ => "DB error",
                            };
                            diag(app, &format!(
                                "[reconcile] relocate FAILED ({kind}) {dead} -> {new_path}: {e:?} — kept for retry"
                            ));
                        }
                    }
                }
            }
            None => remove.push(dead.clone()), // no orphan with this cid — removal CANDIDATE
        }
    }
    if relocate_failed > 20 {
        diag(app, &format!(
            "[reconcile] …plus {} more relocate failures this boot (first 20 detailed above)",
            relocate_failed - 20
        ));
    }

    // 8. De-index the truly-gone — but ONLY when the walk was COMPLETE (an
    //    incomplete walk could hide a renamed note's moved file, turning a relocate
    //    into a destructive remove) AND a fresh re-stat still shows the file gone
    //    (guards a transient stat error that falsely marked a live note dead). Both
    //    guard against destroying review history for a note that isn't actually gone.
    //    [audit HIGH + MED]
    let mut removed = 0usize;
    // PJ-207 §9 — rows whose file turned out to be present after all. The row is correctly
    // KEPT, so nothing is missing on disk — but nothing increments `removed` either, and
    // without this the notice would report the note as still missing when it is not.
    let mut resurrected = 0usize;
    if walk_complete {
        for p in &remove {
            // Per-iteration, not just per-phase: a capped sweep can be 200 deletes, and a
            // delete landing in the wrong universe destroys a row that was never ours.
            if !still_ours() {
                diag(app, "[reconcile] universe switched mid-removal — stopping; the remaining phantoms are left for a clean pass.");
                return Ok(ReconcileOutcome { relocated, readopted: 0, removed, report: None });
            }
            if Path::new(p).exists() {
                resurrected += 1;
                continue; // transient stat earlier — the file is there; keep the row.
            }
            match reindex_delete_note(
                &state,
                p,
                crate::search::DeleteCtx::new(crate::search::DeleteReason::ReconcileGone),
            ) {
                Ok(_) => removed += 1,
                Err(e) => diag(app, &format!("[reconcile] failed to remove {}: {}", p, e)),
            }
        }
    } else if !remove.is_empty() {
        diag(app, &format!("[reconcile] walk INCOMPLETE (a subtree failed to list) — skipping {} removal(s) to protect aux; phantoms left for a clean pass.", remove.len()));
    }

    // 9. RE-ADOPT orphans NOT consumed by a relocate — index the file fresh. Its
    //    note_meta row was already deleted by a prior reconcile, so there was
    //    nothing to relocate; the file on disk is the source of truth (File-Over-
    //    App). Capped: a huge orphan set is a mid-initial-index race, not drift —
    //    the initial reindex owns that, so skip re-adopt there.
    let mut readopted = 0usize;
    let mut readopt_failed = 0usize;
    if orphans.len() <= cap {
        for (p, _cid) in &orphans {
            // The re-adopt tail is the one the inspection named: it INDEXES files, so a
            // switch here writes the departed universe's notes straight into the new
            // universe's `note_meta`/`notes_fts`.
            if !still_ours() {
                diag(app, "[reconcile] universe switched mid-re-adopt — stopping before writing another universe's notes into this index.");
                return Ok(ReconcileOutcome { relocated, readopted, removed, report: None });
            }
            let np = norm(p);
            if consumed.contains(&np) {
                continue;
            }
            if let Some(lib_name) = lib_for(&roots, &np) {
                match reindex_single_note(&state, p, lib_name) {
                    Ok(_) => readopted += 1,
                    Err(e) => {
                        // PJ-154 (2026-07-26): this Err was 100% silent — an orphan that
                        // can never index (e.g. a cid_cn UNIQUE collision with a dead row)
                        // stayed invisible to search with no trace. Surface it, bounded.
                        readopt_failed += 1;
                        if readopt_failed <= 20 {
                            diag(app, &format!("[reconcile] re-adopt FAILED {}: {}", p, e));
                        }
                    }
                }
            }
        }
    } else {
        diag(app, &format!("[reconcile] {} orphan files (> cap {}) — skipping re-adopt (a full reindex is the right tool).", orphans.len(), cap));
    }

    // PJ-151 (2026-07-26): an all-deferred boot used to be COMPLETELY invisible —
    // the (0,0,0) tuple looked like "nothing to do" while every relocate failed.
    // Any failure now forces a boot summary regardless of the healed counts.
    if relocate_failed > 0 || readopt_failed > 0 {
        diag(app, &format!(
            "[reconcile] boot summary: {} relocated, {} re-adopted, {} removed — {} relocate FAILURES, {} re-adopt failures (details above)",
            relocated, readopted, removed, relocate_failed, readopt_failed
        ));
    }

    let report = net_of_healing(report, relocated, readopted, removed + resurrected);

    Ok(ReconcileOutcome { relocated, readopted, removed, report: Some(report) })
}

/// PJ-207 §9 — net the healing out, so the notice describes what is STILL wrong.
///
/// A relocate consumes one of each: a dead row moves onto an orphan file, so it fixes one
/// `missing_on_disk` **and** one `missing_from_index` (`consumed` holds exactly the
/// orphans it took, and it grows once per successful relocate). A removal fixes one dead
/// row; a re-adopt fixes one orphan.
///
/// `resolved_on_disk` is `removed + resurrected`: a row whose file turned out to be
/// present is not "removed", but it is emphatically no longer missing, and counting only
/// the removals would report it to the user as still gone.
///
/// `drifted` is untouched by all of it, deliberately: this pass heals EXISTENCE drift
/// only — it never compares content or mtime, which is the gap PJ-207 exists to close.
/// Every drifted note in the report is still drifted when the user reads it. (The two
/// sets cannot overlap anyway: drift requires a row AND a file, while every healing path
/// here acts on one of the two existing without the other.)
///
/// A pure function so the arithmetic can be pinned by a test. `run` cannot be: every
/// entry point into this module takes an `AppHandle` and the crate has no Tauri test
/// harness — the §8 lesson, and the reason the wiring is asserted against the source
/// instead.
fn net_of_healing(
    report: DriftReport,
    relocated: usize,
    readopted: usize,
    resolved_on_disk: usize,
) -> DriftReport {
    DriftReport {
        missing_on_disk: report.missing_on_disk.saturating_sub(relocated + resolved_on_disk),
        missing_from_index: report.missing_from_index.saturating_sub(relocated + readopted),
        ..report
    }
}

/// Migrate a `note_meta` row + its path-keyed aux rows from `old` to `new` — a
/// lost-tail rename left the row at a dead path while the file moved to `new`
/// with the SAME cid_cn. Mirrors `rename_item_db_tail`'s path cascade; the caller
/// reindexes `new` afterward to refresh name/body. Transactional (all-or-nothing);
/// never overwrites an existing row at `new`.
fn relocate_row(conn: &rusqlite::Connection, old: &str, new: &str) -> rusqlite::Result<()> {
    let occupied: bool = conn
        .query_row("SELECT 1 FROM note_meta WHERE path = ?1", [new], |_| Ok(true))
        .unwrap_or(false);
    if occupied {
        return Err(rusqlite::Error::InvalidQuery);
    }
    conn.execute_batch("BEGIN IMMEDIATE")?;
    // PJ-149 B / Stage-0 C5 (2026-07-26): this was a DUPLICATE 5-table cascade that
    // had already drifted from the canonical one (no note_body/summaries/history/
    // layout/shape/suggestions — the relocated note's earned aux stayed stranded at
    // the dead path). Delegate to the ONE shared cascade so the two surfaces can
    // never drift again (the Whole-Ecosystem law). The helper's note_meta destination
    // pre-delete is a no-op here — the occupied-guard above already proved the
    // destination row-free. Accepted trade (build-spec §2-C5): per-statement error
    // propagation becomes logged-best-effort inside this still-atomic envelope.
    crate::libraries::migrate_note_db_paths(conn, old, new);
    // VERIFY, then report. The shared cascade is best-effort by contract (one failed
    // statement must never abort a user's rename), so it cannot signal failure to us —
    // and on 2026-07-26 that turned this function into a liar: FK enforcement refused
    // every parent-path UPDATE, the cascade logged and moved on, this returned Ok, and
    // reconcile reported "14 relocated" on a boot where NOTHING moved. A success this
    // function reports must be a fact it checked.
    let moved: bool = conn
        .query_row("SELECT 1 FROM note_meta WHERE path = ?1", [new], |_| Ok(true))
        .unwrap_or(false);
    if !moved {
        let _ = conn.execute_batch("ROLLBACK");
        return Err(rusqlite::Error::StatementChangedRows(0));
    }
    conn.execute_batch("COMMIT")?;
    Ok(())
}

/// The most-specific (longest-path) accessible library whose root contains the
/// normalized path `np`, or None. Longest wins so a note in a nested library is
/// attributed to THAT library, not its parent (e.g. universe_notes at the root).
fn lib_for<'a>(roots: &'a [(String, String)], np: &str) -> Option<&'a str> {
    roots
        .iter()
        .filter(|(_, rp)| under(np, &norm(rp)))
        .max_by_key(|(_, rp)| rp.len())
        .map(|(name, _)| name.as_str())
}

/// Recursively walk `.md` files under `dir`, pushing `(path, cid_cn)` for files
/// NOT in `known` (note_meta) to `orphans` (→ relocate a surviving dead row, or
/// re-adopt). Frontmatter (for the cid) is read only for orphan files. Skips
/// hidden entries (`.trash`, `.constellation`).
///
/// `seen` dedupes across OVERLAPPING roots (universe_notes at the root + a nested
/// registered library) so a file is visited once. `complete` is set false on ANY
/// read_dir error or depth cutoff — the caller must NOT remove dead rows from an
/// incomplete walk (a hidden subtree could hold a renamed note's moved file, and
/// removing its row would destroy aux the walk simply failed to surface). [audit]
///
/// PJ-207 §8 — `foreign` holds the roots of libraries owned by a LINKED universe. A
/// directory in that set is skipped WITHOUT clearing `complete`: `complete` means "the
/// walk saw everything it was meant to see", and a linked universe's notes were never
/// among them. Clearing it would permanently disable dead-row removal for any universe
/// with a federated child — turning a scope fix into a durability regression.
/// PJ-207 §9 — `known` carries each row's stored `modified` so the mtime comparison
/// happens HERE, at the one point in the whole boot where the file has already been
/// stat'ed. That is the entire cost argument for the drift check: measured in this code,
/// on the Boss's own hardware (`E:` is a USB mechanical disk, not the SSD the boot budget
/// assumed), the comparison costs **+4 to +10 ms on 7,964 files** — because on Windows the
/// timestamps arrive with the directory listing and no extra syscall is made. A second
/// walker to answer the same question would have paid for the whole traversal again, which
/// is what the Whole-Ecosystem Fix Law is for.
///
/// The same insight is why classification uses `entry.file_type()` rather than
/// `path.is_dir()`. `Path::is_dir()` is `fs::metadata`, which on Windows opens a handle per
/// entry; `file_type()` is already in hand. A symlink falls back to `is_dir()`, so junction
/// traversal is bit-for-bit what it was.
///
/// **Measured, warm, by `pj207_s9_drift_cost` in this file:**
///
/// | tree | before (`is_dir`, no drift check) | after (`file_type` + drift check) |
/// |---|---|---|
/// | 7,964 `.md` | 252–260 ms | **17–19 ms** |
/// | 2,094 `.md` | 207–219 ms | **17–18 ms** |
///
/// So the step that added a per-file comparison made the boot walk **~14× faster**, and the
/// honest figure for §9's cost is *negative*: about 200–240 ms cheaper than the walk it
/// replaces, every launch. Cold, on the Boss's USB mechanical disk, the same traversal was
/// 3.5–8.7 s — which is what that 14× is worth in practice.
fn collect_md(
    dir: &Path,
    known: &HashMap<String, u64>,
    foreign: &HashSet<String>,
    walk: &mut Walk,
    depth: u32,
) {
    if depth > 20 {
        // Truncated — a deeper file is unseen; don't trust removal. Counted as unreadable
        // too: this IS a directory we did not list, and `has_findings` keys on the count
        // so that a walk which could not look everywhere can never render as "all clear".
        walk.complete = false;
        walk.dirs_unreadable += 1;
        return;
    }
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => {
            walk.complete = false; // this subtree is unseen; don't trust removal for it.
            walk.dirs_unreadable += 1;
            return;
        }
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if entry_is_dir(&entry, &path) {
            // A linked universe's root nested under ours — not our notes, not our orphans.
            if crate::libraries::is_walk_boundary(&path, foreign) {
                continue;
            }
            collect_md(&path, known, foreign, walk, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let ps = path.to_string_lossy().to_string();
            let pn = norm(&ps);
            if !walk.seen.insert(pn.clone()) {
                continue; // already visited via an overlapping root
            }
            walk.files_seen += 1;
            match known.get(&pn) {
                // Orphan — read its cid_cn (empty for a cid-free note).
                None => {
                    let cid = std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|c| extract_frontmatter_cid_cn(&c))
                        .unwrap_or_default();
                    walk.orphans.push((ps, cid));
                }
                Some(&stored) => match entry_mtime(&entry) {
                    Some(m) if m == stored => walk.unchanged += 1,
                    Some(_) => walk.drifted += 1,
                    // A file we could not stat is a file we cannot judge. It is neither
                    // unchanged nor drifted, and saying either would be inventing an
                    // answer — the honest report is that one file could not be read.
                    None => walk.files_unreadable += 1,
                },
            }
        }
    }
}

/// Is this entry a directory to descend into? The cheap test, with the exact old
/// behaviour preserved for the one case where the two differ.
///
/// `DirEntry::file_type()` comes from the directory enumeration — free. `Path::is_dir()`
/// is `fs::metadata`, a handle open per entry, and it was ~95% of this walk's cost. They
/// disagree only on symlinks: `is_dir()` follows one and reports the TARGET, `file_type()`
/// reports the link. A directory symlink or junction inside a library was descended into
/// before this change, so it must still be — hence the fallback rather than a bare
/// `ft.is_dir()`, which would silently stop walking it.
fn entry_is_dir(entry: &std::fs::DirEntry, path: &Path) -> bool {
    match entry.file_type() {
        Ok(ft) if !ft.is_symlink() => ft.is_dir(),
        _ => path.is_dir(),
    }
}

/// The entry's mtime, in `note_meta.modified`'s units.
///
/// `DirEntry::metadata()` is the cheap call — on Windows the directory enumeration
/// already carried the timestamps, so this costs no syscall at all, which is why the
/// drift check is ~2 ms on 8,000 files. But it deliberately does **not** follow symlinks,
/// while `index_note` stats through `fs::metadata` and therefore stores the TARGET's
/// mtime. Left unhandled, every symlinked note would read as permanently drifted and the
/// notice would nag about a note nothing can fix. The slow path is taken only for an
/// actual symlink, which in a note library is approximately never.
fn entry_mtime(entry: &std::fs::DirEntry) -> Option<u64> {
    let md = entry.metadata().ok()?;
    if md.file_type().is_symlink() {
        return std::fs::metadata(entry.path()).ok().as_ref().and_then(crate::search::mtime_secs);
    }
    crate::search::mtime_secs(&md)
}

/// Write a line to the universe's diagnostics log (mirrors `links_backfill::diag`).
fn diag(app: &tauri::AppHandle, msg: &str) {
    if let Ok(path) = crate::search::db_path(app) {
        crate::search::diag_log(&path, msg);
    }
}

#[cfg(test)]
mod tests_mig112_deadopt_position {
    //! MIG-112 §8 — the de-adopt must run BEFORE `run`'s early returns.
    //!
    //! ## The bug this exists to catch, which shipped past every other gate
    //!
    //! The de-adopt was first written as step **8b**, below two unconditional returns —
    //! `if stale.is_empty() && orphans.is_empty()` and the stale safety cap. It compiled, all
    //! 1,581 tests passed, and it was **dead code on every boot that mattered**, because the
    //! state it exists to fix is exactly the state that takes the first return:
    //!
    //! * a nested-universe row can never enter `stale` — step 3's MIG-112 arm `continue`s
    //!   before the `!Path::exists()` test, and those files DO exist; and
    //! * it can never enter `orphans` — `collect_md` skips the nested root via the same
    //!   predicate.
    //!
    //! So whenever there is something to de-adopt, both other sets are empty, `run` returns,
    //! and nothing happens. Caught by the second safety-inspection pass, not by a test — hence
    //! this one.
    //!
    //! ## Why a SOURCE-ORDER test rather than a behavioural one
    //!
    //! `run` needs an `AppHandle`, so this module's other tests drive the helpers instead and
    //! none of them can see placement. This asserts the one property that failed, using the
    //! `body_of` pattern already established in `libraries.rs`. Comments are stripped first:
    //! a test that can be satisfied by writing a sentence is not a test.

    fn run_body_without_comments() -> String {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("reconcile.rs");
        let all = std::fs::read_to_string(&p).expect("read reconcile.rs");
        // Cut EVERY test module off before searching. The first cut of this helper did not, and
        // searched for a function signature that also appears in this test's own source — so it
        // matched ITS OWN line, extracted a "body" that was really the test module, found no
        // de-adopt in it and reported the production code broken. Same family as the comment
        // trap `libraries.rs::body_of` records: a structural test must not be able to see itself.
        let prod = &all[..all.find("#[cfg(test)]").unwrap_or(all.len())];
        let start = prod
            .find("fn run(app: &tauri::AppHandle)")
            .expect("reconcile::run was renamed — re-derive this test against the new name");
        prod[start..]
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("
")
    }

    #[test]
    fn the_de_adopt_runs_before_the_clean_drift_early_return() {
        let body = run_body_without_comments();
        let de_adopt = body
            .find("DeleteReason::ForeignUniverse")
            .expect("the MIG-112 de-adopt is gone from run() — if it moved, move this test with it");
        let clean_return = body
            .find("if stale.is_empty() && orphans.is_empty()")
            .expect("the clean-drift early return is gone — re-derive this test against what replaced it");
        assert!(
            de_adopt < clean_return,
            "MIG-112 REGRESSION: the de-adopt sits AFTER the clean-drift early return, so it is              dead code on exactly the boots where it is needed. A nested-universe row never              enters `stale` (its file exists) and never enters `orphans` (the walk skips it), so              that return fires and the de-adopt never runs. Move it back above the return."
        );
    }

    #[test]
    fn the_de_adopt_also_runs_before_the_safety_cap_return() {
        let body = run_body_without_comments();
        let de_adopt = body.find("DeleteReason::ForeignUniverse").expect("de-adopt missing");
        let cap_return = body
            .find("if stale.len() > cap")
            .expect("the stale safety-cap return is gone — re-derive this test");
        assert!(
            de_adopt < cap_return,
            "MIG-112 REGRESSION: an unrelated stale-row spike would skip the de-adopt entirely"
        );
    }

    #[test]
    fn the_de_adopt_is_still_guarded_by_the_universe_switch_check() {
        let body = run_body_without_comments();
        let start = body.find("DeleteReason::ForeignUniverse").expect("de-adopt missing");
        let window_start = body[..start].rfind("still_ours()").unwrap_or(0);
        assert!(
            window_start > 0 && start - window_start < 1200,
            "the de-adopt lost its `still_ours()` guard — a removal landing after a universe              switch would drop a row from a universe this pass never measured"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Minimal schema covering every table `relocate_row` migrates. (`is_stamped`
    /// returns false without `schema_versions`, so `review_schedule` is skipped.)
    fn schema(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT, cid_cn TEXT);
             CREATE TABLE note_links (source_path TEXT, target_name TEXT);
             CREATE TABLE note_aliases (path TEXT, alias_lower TEXT);
             CREATE TABLE note_embeddings (path TEXT, vec BLOB);",
        )
        .unwrap();
    }

    /// MIG-097 — a lost-tail rename leaves the row at a dead path; relocating it to
    /// the note's current file (by cid_cn) must migrate note_meta + every aux row,
    /// preserving the stable cid_cn (and thus review history / links).
    #[test]
    fn relocate_row_migrates_note_and_aux_by_path() {
        let conn = Connection::open_in_memory().unwrap();
        schema(&conn);
        let dead = "E:/lib/التجربة الثانية_إعادة تسمية.md";
        let new = "E:/lib/التجربة الثانية ن2.md";
        conn.execute("INSERT INTO note_meta(path,name,cid_cn) VALUES (?1,'التجربة الثانية','CID8878')", [dead]).unwrap();
        conn.execute("INSERT INTO note_links(source_path,target_name) VALUES (?1,'Foo')", [dead]).unwrap();
        conn.execute("INSERT INTO note_aliases(path,alias_lower) VALUES (?1,'x')", [dead]).unwrap();
        conn.execute("INSERT INTO note_embeddings(path,vec) VALUES (?1, x'00')", [dead]).unwrap();

        relocate_row(&conn, dead, new).unwrap();

        let cnt_dead: i64 = conn.query_row("SELECT COUNT(*) FROM note_meta WHERE path=?1", [dead], |r| r.get(0)).unwrap();
        let cnt_new: i64 = conn.query_row("SELECT COUNT(*) FROM note_meta WHERE path=?1", [new], |r| r.get(0)).unwrap();
        assert_eq!(cnt_dead, 0, "dead note_meta row removed");
        assert_eq!(cnt_new, 1, "note_meta relocated to the current file");
        let cid: String = conn.query_row("SELECT cid_cn FROM note_meta WHERE path=?1", [new], |r| r.get(0)).unwrap();
        assert_eq!(cid, "CID8878", "stable cid_cn preserved across relocate");
        let lnk: String = conn.query_row("SELECT source_path FROM note_links", [], |r| r.get(0)).unwrap();
        assert_eq!(lnk, new, "note_links.source_path migrated");
        let al: String = conn.query_row("SELECT path FROM note_aliases", [], |r| r.get(0)).unwrap();
        assert_eq!(al, new, "note_aliases.path migrated");
        let em: String = conn.query_row("SELECT path FROM note_embeddings", [], |r| r.get(0)).unwrap();
        assert_eq!(em, new, "note_embeddings.path migrated");
    }

    /// Never overwrite an existing row (orphans have none by definition; guard it).
    /// A refused relocate must leave BOTH rows intact (no data loss).
    #[test]
    fn relocate_row_refuses_occupied_target() {
        let conn = Connection::open_in_memory().unwrap();
        schema(&conn);
        conn.execute("INSERT INTO note_meta(path,name,cid_cn) VALUES ('a.md','A','C1')", []).unwrap();
        conn.execute("INSERT INTO note_meta(path,name,cid_cn) VALUES ('b.md','B','C2')", []).unwrap();
        assert!(relocate_row(&conn, "a.md", "b.md").is_err(), "must refuse an occupied target");
        let cnt: i64 = conn.query_row("SELECT COUNT(*) FROM note_meta", [], |r| r.get(0)).unwrap();
        assert_eq!(cnt, 2, "no row lost on refused relocate");
    }

    /// The disk walk pushes (path, cid) for files NOT in `known` to `orphans`
    /// (→ relocate / re-adopt), skipping already-indexed files. This is what lets
    /// the reconcile recover a note whose dead row a prior pass already removed.
    #[test]
    fn collect_md_finds_orphans_skips_indexed() {
        let dir = std::env::temp_dir().join(format!("mig098_md_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let orphan = dir.join("relocated note.md");
        std::fs::write(&orphan, "---\ntitle: Relocated\ncid_cn: CIDNEW\nkind: note\n---\nbody").unwrap();
        let known_file = dir.join("already indexed.md");
        std::fs::write(&known_file, "---\ntitle: Known\ncid_cn: CIDOLD\nkind: note\n---\nbody").unwrap();

        let mut known = HashMap::new();
        known.insert(norm(&known_file.to_string_lossy()), mtime_of(&known_file));
        let mut walk = Walk::default();
        collect_md(&dir, &known, &HashSet::new(), &mut walk, 0);

        assert!(walk.complete, "a clean walk reports complete");
        assert_eq!(walk.orphans.len(), 1, "only the unindexed file is an orphan");
        assert_eq!(walk.orphans[0].1, "CIDNEW", "orphan carries its cid_cn for relocate/re-adopt");
        assert_eq!(norm(&walk.orphans[0].0), norm(&orphan.to_string_lossy()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PJ-207 §8 — the oscillation this step exists to prevent, at its source.
    ///
    /// Step 9 RE-ADOPTS every orphan the walk finds. So if a linked universe's directory
    /// sits under one of our roots, its notes are orphans here, and removing their rows
    /// would only mean re-adopting them on the next launch — forever. The foreign root is
    /// skipped, and `complete` deliberately stays true: a subtree we were never meant to
    /// see is not a subtree we failed to read, and clearing the flag would disable
    /// dead-row removal for every federated universe.
    #[test]
    fn collect_md_skips_a_linked_universes_root_without_marking_the_walk_incomplete() {
        let dir = std::env::temp_dir().join(format!("pj207s8_md_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let foreign_dir = dir.join("Linked Child");
        std::fs::create_dir_all(&foreign_dir).unwrap();
        std::fs::write(dir.join("mine.md"), "---\ntitle: Mine\ncid_cn: CIDMINE\n---\nzarquon").unwrap();
        std::fs::write(foreign_dir.join("theirs.md"), "---\ntitle: Theirs\ncid_cn: CIDTHEIRS\n---\nblorptide").unwrap();

        let mut foreign = HashSet::new();
        foreign.insert(norm(&foreign_dir.to_string_lossy()));

        let mut walk = Walk::default();
        collect_md(&dir, &HashMap::new(), &foreign, &mut walk, 0);

        assert!(walk.complete, "an excluded subtree is not an unreadable one — removal stays enabled");
        assert_eq!(walk.orphans.len(), 1, "only our own note is an orphan to re-adopt");
        assert_eq!(walk.orphans[0].1, "CIDMINE", "and it is ours, not the linked universe's");
        assert_eq!(walk.files_seen, 1, "and the linked universe's file is not even counted");

        // Without the exclusion the same walk reaches it — the state before this step.
        let mut walk2 = Walk::default();
        collect_md(&dir, &HashMap::new(), &HashSet::new(), &mut walk2, 0);
        assert_eq!(walk2.orphans.len(), 2, "REPRODUCTION: unscoped, the linked note is an orphan to re-adopt");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The file's mtime in `note_meta.modified`'s units — through the SAME function the
    /// indexer stores it with, so a test cannot pass on a definition production doesn't use.
    fn mtime_of(p: &Path) -> u64 {
        crate::search::mtime_secs(&std::fs::metadata(p).unwrap()).unwrap()
    }

    /// PJ-207 §9 — **the reproduction, as a test.** A note edited while Constellation was
    /// closed leaves the file's mtime ahead of the row's, and nothing at boot notices.
    ///
    /// The recipe from the reproduction record (`PJ-207-REPRODUCTION-2026-08-03.md` §2),
    /// in miniature: two indexed notes, one of which has moved on disk since its row was
    /// written. Exactly one is drift; the other must NOT be, or the notice cries wolf on
    /// every launch about 7,800 unchanged notes.
    #[test]
    fn a_note_edited_while_the_app_was_closed_is_counted_as_drift() {
        let dir = std::env::temp_dir().join(format!("pj207s9_drift_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let edited = dir.join("edited outside.md");
        let untouched = dir.join("untouched.md");
        std::fs::write(&edited, "---\ncid_cn: CIDA\n---\nvandrasil").unwrap();
        std::fs::write(&untouched, "---\ncid_cn: CIDB\n---\nquiet").unwrap();

        let mut known = HashMap::new();
        // The edited note's row remembers an OLDER mtime — what the index held before the
        // external edit. The untouched note's row agrees with its file.
        known.insert(norm(&edited.to_string_lossy()), mtime_of(&edited) - 4_735_509);
        known.insert(norm(&untouched.to_string_lossy()), mtime_of(&untouched));

        let mut walk = Walk::default();
        collect_md(&dir, &known, &HashSet::new(), &mut walk, 0);

        assert_eq!(walk.drifted, 1, "the externally-edited note is drift");
        assert_eq!(walk.unchanged, 1, "and the untouched one is NOT — the notice must not cry wolf");
        assert_eq!(walk.orphans.len(), 0, "both are indexed, so neither is an orphan");
        assert_eq!(walk.files_seen, 2);
        assert_eq!(walk.files_unreadable, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PJ-207 §9 / PJ-223 — **a file the index has never seen is not a file that changed.**
    ///
    /// On the Boss's live universe this is 825 notes, 798 of them in `Constellation PKM`,
    /// a registered own library: on disk, absent from the index, and therefore absent from
    /// search. The plan's three-counter report had no field for them; with the obvious
    /// implementation of `drifted` (`existing_mod == Some(m)` against a row that does not
    /// exist) all 825 would have been reported as "changed while Constellation was
    /// closed", which is false for every one of them — they never changed, they were
    /// never read. This test is the difference between those two sentences.
    #[test]
    fn a_file_the_index_has_never_seen_is_counted_apart_from_one_that_changed() {
        let dir = std::env::temp_dir().join(format!("pj207s9_unseen_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let never_indexed = dir.join("never indexed.md");
        let indexed = dir.join("indexed.md");
        std::fs::write(&never_indexed, "---\ncid_cn: CIDX\n---\nunseen").unwrap();
        std::fs::write(&indexed, "---\ncid_cn: CIDY\n---\nseen").unwrap();

        let mut known = HashMap::new();
        known.insert(norm(&indexed.to_string_lossy()), mtime_of(&indexed));

        let mut walk = Walk::default();
        collect_md(&dir, &known, &HashSet::new(), &mut walk, 0);

        assert_eq!(walk.orphans.len(), 1, "the never-indexed file is missing FROM THE INDEX");
        assert_eq!(walk.drifted, 0, "and it did NOT change — reporting it as changed is a false sentence");
        assert_eq!(walk.unchanged, 1, "the indexed, unmoved note");
        assert_eq!(
            walk.files_seen,
            walk.unchanged + walk.drifted + walk.orphans.len(),
            "the books close: every file seen is unchanged, drifted, or missing from the index"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A directory that cannot be listed makes the walk INCOMPLETE and is counted — a
    /// sweep that could not look must never be rendered as "nothing changed". The depth
    /// cutoff is the other way in, and it is the one with no `read_dir` error to notice.
    #[test]
    fn a_walk_that_could_not_look_everywhere_says_so() {
        let mut walk = Walk::default();
        collect_md(Path::new("Z:/no/such/directory/pj207s9"), &HashMap::new(), &HashSet::new(), &mut walk, 0);
        assert!(!walk.complete, "an unreadable root is not an empty one");
        assert_eq!(walk.dirs_unreadable, 1, "and it is counted, not merely flagged");

        let mut deep = Walk::default();
        collect_md(Path::new("."), &HashMap::new(), &HashSet::new(), &mut deep, 21);
        assert!(!deep.complete, "past the depth cap the walk is truncated, so removal must not be trusted");
    }

    /// PJ-207 §9 — the notice describes what is STILL wrong after this pass healed what it
    /// could. Reporting five re-adopted files as missing would be an alarm about work the
    /// same launch already finished; a relocate fixes BOTH directions at once, because it
    /// moves one dead row onto one orphan file.
    #[test]
    fn the_report_is_what_survived_the_healing() {
        let found = DriftReport {
            drifted: 19,
            missing_from_index: 10,
            missing_on_disk: 7,
            foreign_rows: 9,
            files_seen: 2094,
            rows_seen: 1890,
            walk_complete: true,
            ..DriftReport::default()
        };
        // 2 relocates (each fixes one of each), 5 re-adopts, 3 removals.
        let net = net_of_healing(found, 2, 5, 3);
        assert_eq!(net.missing_from_index, 3, "10 orphans − 2 relocated − 5 re-adopted");
        assert_eq!(net.missing_on_disk, 2, "7 dead rows − 2 relocated − 3 removed");
        assert_eq!(net.drifted, 19, "this pass never re-reads a changed file, so drift survives it untouched");
        assert_eq!(net.foreign_rows, 9, "and nothing here removes a linked universe's row");

        // Healing can only ever fix what was found; the counts must never wrap.
        let over = net_of_healing(found, 40, 40, 40);
        assert_eq!(over.missing_from_index, 0);
        assert_eq!(over.missing_on_disk, 0);
    }

    /// A launch that finds nothing wrong says nothing at all. `has_findings` is what keeps
    /// a green "all clear" banner off the screen on every boot — and `foreign_rows` is
    /// deliberately NOT a finding: linked-universe copies are a state of the index §13 may
    /// one day offer to tidy, not a reason to interrupt someone opening their notes.
    ///
    /// **The last two assertions are the 2026-08-07 safety inspection's finding.** A
    /// library with one unlistable folder produces all three drift counts at zero — the
    /// notes under it were never seen — so a `has_findings` asking only about those three
    /// suppressed the report, and silence is how this feature says "all clear". A whole
    /// subtree absent from search would have been rendered as a clean launch.
    #[test]
    fn a_clean_report_renders_nothing_but_a_walk_that_could_not_look_is_not_clean() {
        assert!(!DriftReport::default().has_findings());
        assert!(!DriftReport { foreign_rows: 621, ..DriftReport::default() }.has_findings());
        assert!(DriftReport { drifted: 1, ..DriftReport::default() }.has_findings());
        assert!(DriftReport { missing_from_index: 1, ..DriftReport::default() }.has_findings());
        assert!(DriftReport { missing_on_disk: 1, ..DriftReport::default() }.has_findings());
        assert!(
            DriftReport { dirs_unreadable: 1, ..DriftReport::default() }.has_findings(),
            "a folder that could not be listed hides every note under it — reporting that \
             launch as clean is the silent failure this whole migration exists to end"
        );
        assert!(DriftReport { files_unreadable: 1, ..DriftReport::default() }.has_findings());
    }

    /// PJ-207 §M6 — **what the drift check actually costs**, in the shipped code, against
    /// a real note tree. Not run by the suite; it needs a corpus and a cold cache.
    ///
    /// ```text
    /// PJ207_S9_TREE="E:\Constellation Universes\Eisa Universe" \
    ///   cargo test --release pj207_s9_drift_cost -- --ignored --nocapture
    /// ```
    ///
    /// It prints two numbers: the walk as it shipped BEFORE this step (no per-file stat)
    /// and the walk with the drift comparison folded in. The difference is §9's entire
    /// cost, because the traversal itself was already happening on every launch.
    ///
    /// The baseline closure below mirrors `collect_md` minus the stat. A mirror is
    /// forbidden in a test that ASSERTS behaviour — that is the `search.rs:338` trap §1
    /// deleted — but this one asserts nothing; it exists to produce a baseline number, and
    /// a baseline that drifted from production would show up as a nonsensical delta.
    #[test]
    #[ignore]
    fn pj207_s9_drift_cost() {
        let Ok(root) = std::env::var("PJ207_S9_TREE") else {
            println!("set PJ207_S9_TREE to a universe root to measure");
            return;
        };
        fn bare(dir: &Path, n: &mut usize, depth: u32) {
            if depth > 20 { return; }
            let Ok(rd) = std::fs::read_dir(dir) else { return };
            for e in rd.flatten() {
                let p = e.path();
                if e.file_name().to_string_lossy().starts_with('.') { continue; }
                if p.is_dir() { bare(&p, n, depth + 1); }
                else if p.extension().map(|x| x == "md").unwrap_or(false) { *n += 1; }
            }
        }
        // Every file present in `known` with a matching mtime, so the measured walk takes
        // the stat-and-compare path for all of them — the worst case, and the steady state.
        let mut known: HashMap<String, u64> = HashMap::new();
        {
            let mut w = Walk::default();
            collect_md(Path::new(&root), &known, &HashSet::new(), &mut w, 0);
            for (p, _) in &w.orphans {
                if let Ok(md) = std::fs::metadata(p) {
                    if let Some(m) = crate::search::mtime_secs(&md) { known.insert(norm(p), m); }
                }
            }
        }
        for trial in 0..5 {
            let t0 = std::time::Instant::now();
            let mut n = 0usize;
            bare(Path::new(&root), &mut n, 0);
            let before = t0.elapsed();

            let t1 = std::time::Instant::now();
            let mut w = Walk::default();
            collect_md(Path::new(&root), &known, &HashSet::new(), &mut w, 0);
            let after = t1.elapsed();

            println!(
                "trial {trial}: {n} md · walk-only {:>8.1} ms · with drift check {:>8.1} ms · \
                 delta {:>+7.1} ms · unchanged {} drifted {} orphans {} unreadable {}/{}",
                before.as_secs_f64() * 1000.0,
                after.as_secs_f64() * 1000.0,
                (after.as_secs_f64() - before.as_secs_f64()) * 1000.0,
                w.unchanged, w.drifted, w.orphans.len(), w.dirs_unreadable, w.files_unreadable,
            );
        }
    }

    /// `lib_for` attributes a path to the MOST-SPECIFIC (longest) containing root,
    /// so a note in a nested library isn't mis-attributed to the universe_notes root.
    #[test]
    fn lib_for_prefers_the_nested_library() {
        let roots = vec![
            ("universe_notes".to_string(), "E:/U".to_string()),
            ("Nested".to_string(), "E:/U/Nested".to_string()),
        ];
        assert_eq!(lib_for(&roots, &norm("E:/U/Nested/note.md")), Some("Nested"));
        assert_eq!(lib_for(&roots, &norm("E:/U/top.md")), Some("universe_notes"));
        assert_eq!(lib_for(&roots, &norm("E:/Other/x.md")), None);
    }
}
