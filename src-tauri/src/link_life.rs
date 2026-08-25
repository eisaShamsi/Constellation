//! MIG-104 Slice 3 — the Earned-Life Ledger: the appender, the union reader, the contract.
//!
//! **The concept (the horse).** The traffic of the user's own mind — which links they walked,
//! what they came to trust, what they retired, and how a note changed over time — must live in
//! plain-text files they own, so the index can be thrown away and a note can be deleted
//! without the knowledge going with it.
//!
//! **Why an append.** Every other candidate design rewrote a file, and a rewriter holding a
//! stale or empty in-memory map writes an empty store — destroying exactly what it exists to
//! protect. An append has no such surface: a torn tail costs one line, and every earlier line
//! is immutable. This is the mechanism being *structurally incapable* of the failure rather
//! than disciplined against it.
//!
//! **Two streams, one appender — because the fold algebras differ and must never be confused:**
//!
//! | | `earned.jsonl` (+ snapshot) | `note-history.jsonl` |
//! |---|---|---|
//! | record is | a *fold target* | the payload itself |
//! | fold | `n` = **max**, decisions = latest; commutative + idempotent | **NEVER FOLDS, NEVER COMPACTS** |
//! | bounded by | earned-link count (33 live) | history events, forever |
//!
//! Folding the history stream would collapse a thought into a keystroke: the live rows
//! `hid` 8251/8252/8253 record `ma` → `mas` → `masadir`, a property being typed. `read_state`
//! is the ONLY fold implementation in this module, and it reads Stream A only;
//! `read_history_for` deliberately has none.
//!
//! **The one weakness of an append-only store is that it only grows — so Stream A is compacted**
//! (`maybe_compact`): past a byte threshold the folded state is written to
//! `earned.snapshot.jsonl` and the tail is renamed aside, never deleted. Load is then
//! `snapshot + tail`, both bounded. Stream B is structurally excluded from that machinery —
//! `maybe_compact` has no stream parameter — because a compactor that could reach it would
//! eventually be pointed at it.
//!
//! **Ordering is by `hid` (the source row ordinal), never by `at`** — 765 `captured_at` groups
//! collide across 1,536 live rows, with 2,066 order inversions.
//!
//! **Portability (Boss ruling Q5).** The Universe is portable across Windows and macOS, never
//! opened concurrently. Every key is cid-first with a Universe-relative, forward-slashed, NFC
//! path fallback; every line ends `\n`. A ledger written on one OS must read byte-correctly on
//! the other.
//!
//! **fsync is per-site, not uniform** (measured Slice 0: fsync 3,418 µs vs a plain append
//! 168 µs — 20×). Mandatory where the only other copy is about to be destroyed
//! (archive-before-purge) and for rare user decisions; a plain append for walk counters and the
//! continuous history mirror. See `tests/mig104/README.md`.

use std::io::Write;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

/// The two streams. One appender, two files — see the module docs for why they may never share
/// a fold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// `earned.jsonl` — link life. Folds.
    Earned,
    /// `note-history.jsonl` — a note's change history. Never folds.
    NoteHistory,
}

impl Stream {
    pub fn file_name(self) -> &'static str {
        match self {
            Stream::Earned => "earned.jsonl",
            Stream::NoteHistory => "note-history.jsonl",
        }
    }
}

/// `earned.snapshot.jsonl` — one line per earned link (plus one per note decision), current
/// folded state. Bounded by what the user has earned, never by how long they have been using the
/// app, which is what keeps the load bounded. Written by `maybe_compact`.
pub const SNAPSHOT_FILE: &str = "earned.snapshot.jsonl";

/// The store's directory, derived from the connection itself: `conn.path()`'s parent IS the
/// `.constellation` dir (Boss ruling Q1). This is why no writer needs a path threaded to it —
/// pinned by `tests_mig104_baseline::conn_path_parent_is_the_constellation_dir`.
pub fn store_dir(conn: &Connection) -> Option<PathBuf> {
    let p = conn.path()?;
    if p.is_empty() {
        return None; // in-memory connection (tests) — no store
    }
    Path::new(p).parent().map(|d| d.to_path_buf())
}

/// What a load found, so nothing is ever silently swallowed (§3.7, the corrupt-store contract).
#[derive(Debug, Default, Clone)]
pub struct LoadReport {
    /// Individual unparseable lines. Each costs ONE line — never the file — and is COUNTED.
    pub skipped_lines: usize,
    /// Set when the store was structurally unusable and renamed aside.
    pub corrupt_renamed_to: Option<PathBuf>,
    /// When true the caller must NOT write a fresh store: a blind overwrite would destroy the
    /// backup that was about to save the user. Requires acknowledgement first.
    pub refuse_write: bool,
    /// **The file EXISTS but could not be read at all** — set to the OS error. `None` covers both
    /// "read fine" and "genuinely absent"; only a caller that needs the difference must ask, and
    /// `archive_present` (`deleted_notes.rs`) already separates absent from empty.
    ///
    /// 2026-08-25 inspection, HIGH false-success. `read_lines` mapped EVERY `read_to_string`
    /// error to an empty Vec, so an archive that exists and cannot be decoded — a tear inside a
    /// multi-byte UTF-8 sequence, a Windows sharing violation from an antivirus or backup agent,
    /// an unhydrated cloud placeholder — arrived at the Deleted-notes surface as
    /// `{ notes: [], total: 0, unreadableLines: 0, archivePresent: true }`, which renders as
    /// *"The record exists and is empty — no removal has been recorded in this universe."*
    /// Asserted as fact, with no error and no cue, about the last surviving record of notes whose
    /// files are gone. The `unreadableLines` banner could not fire: it lives inside the
    /// `total > 0` branch. Absent stays a FACT; unreadable is now a different fact.
    pub unreadable_file: Option<String>,
}

/// One folded link-life record: the current state of one earned link.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Earned {
    /// Absolute traversal count. Folds by MAX — never a sum, never a decrement.
    pub n: i64,
    pub conf: Option<String>,
    pub status: Option<String>,
    /// Last-writer-wins timestamp of the newest record folded in.
    pub at: Option<String>,
    /// The target's NAME as last written — the only human-legible part of a line, and the
    /// fallback key when the target has no identity. Folded so the SNAPSHOT can re-emit it:
    /// a snapshot rebuilt from the key alone would read `{"to":"20260512T144233Z_NOTE_77C9"}`
    /// with no clue what that is, silently trading File-Over-App legibility for compaction.
    pub tn: Option<String>,
    /// Whether the record that supplied `at` was a SEEDED one (`"seed":1`) — i.e. whether this
    /// record's timestamp is derived rather than witnessed (Slice 5, Boss-found). Folded so the
    /// snapshot can carry the marker forward; without it, compaction would silently relabel every
    /// reconstructed timestamp as an observed one.
    pub at_seeded: bool,
}

/// One folded NOTE-level record. Today only review priority, which is a decision about a note
/// rather than about a link and therefore has no `(source, target)` key.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct NoteEarned {
    /// The review-priority override. **`-1` means CLEARED** — that is what `set_review_priority`
    /// writes for `None` (`review.rs`), and the restore maps it back to SQL `NULL`.
    pub p: i64,
    pub at: Option<String>,
    pub at_seeded: bool,
}

pub type FoldedMap = std::collections::HashMap<String, Earned>;
pub type NoteMap = std::collections::HashMap<String, NoteEarned>;

/// **Everything Stream A means, after folding — and the ONLY thing the snapshot is written from.**
///
/// That sentence is the whole safety argument for compaction: the compactor rewrites the store
/// from this value, so compaction is lossless **iff this type can re-express every record the
/// ledger accepts**. Anything a writer appends that this type cannot hold is data the compactor
/// would quietly drop — so a new record kind must land here in the same change that introduces it.
///
/// That is not hypothetical. Slice 4 shipped `priority` records that `set_review_priority`
/// appends **and fsyncs**, while the fold's key function required a target and therefore dropped
/// every one of them. Nothing consumed them, so nothing failed — until this slice, where a
/// fold-and-rewrite would have moved them out of the loaded store for good. Found and fixed here
/// (WA#6); `notes` is the field that holds them.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LedgerState {
    /// Earned link state, keyed `cid>TARGET_CID` or `cid>~target-name`.
    pub links: FoldedMap,
    /// Note-level decisions, keyed by the note's `cid_cn`.
    pub notes: NoteMap,
}

/// One note-history record, as archived. The record IS the payload — see the module docs.
#[derive(Debug, Clone, PartialEq)]
pub struct HistRecord {
    pub cid: String,
    /// The source row ordinal. THE ordering key — `at` collides constantly.
    pub hid: i64,
    pub at: i64,
    pub raw: String,
}

/// Serializes every operation that MUTATES the store's files against every other one.
///
/// ★ **The bug this exists to prevent — found by the safety inspection on the Slice-7 build, in
/// the slice's own new code.** `maybe_compact` folds the tail into a snapshot and then renames the
/// tail aside. Between those two steps it writes and fsyncs a multi-megabyte file — tens of
/// milliseconds — and `append` took no lock of any kind. Every record appended in that window was
/// moved into `earned.tail-<stamp>.jsonl`, which **nothing ever reads back** (that is exactly what
/// bounds the load). On Windows the rename even succeeds while an append handle is open
/// (`FILE_SHARE_DELETE`), so the handle keeps writing into the aside file.
///
/// The appenders are provably concurrent with a boot-thread compaction: `constellation_link_traverse`
/// (`search.rs`), `record_decision` (retire / restore / trust) and `set_review_priority`
/// (`review.rs`) all append **after deliberately dropping the DB guard**, from Tauri command
/// threads, at any moment of an interactive session.
///
/// **And the damage is worse than a missing line.** The restore treats the ledger as authoritative
/// for DECISIONS (confidence, retired/active, review priority). A decision lost this way is not
/// merely absent: on the next boot the fold still carries the *pre-decision* value, disagrees with
/// the DB, and **writes the old value back** — silently reversing a retirement or a priority the
/// user had set, while every step logs success. Walk counts self-heal (absolute `n`, max-fold), so
/// the permanent loss lands precisely on the data this migration exists to make durable.
///
/// My own comment in `link_life_restore` claimed this was impossible because compaction rides the
/// restore's thread. That reasoning covered restore-vs-compact and nothing else — a sequencing
/// argument mistaken for an exclusion argument. Corrected there too.
///
/// A poisoned lock must never stop the ledger writing: `into_inner()` rather than `unwrap()`.
static FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// MIG-111 Phase 0.3 (R5 · Architect condition 5) — the guard is TWO locks now.
///
/// The static mutex above orders THREADS within this process — and is all the original guard
/// was. It is invisible to a second Constellation instance, so a compaction over THERE could
/// still rename the tail out from under an append over HERE: the Slice-7 bug shape, across
/// the process boundary — the exact hazard the MIG-111 Option-B adversarial pass named A1,
/// promoted from latent to routine the moment writes cross universes (a parent appending
/// earned decisions into a linked universe's ledger while that universe is open elsewhere).
///
/// So the guard now ALSO takes a **blocking exclusive OS lock** on `<dir>/ledger.lock`
/// (fs4: `LockFileEx` on Windows, `flock` on Unix), taken AFTER the in-process mutex — our
/// own threads queue on the cheap mutex so at most one thread per process waits on the OS
/// lock. Hold times are the guard's existing ones (µs for an append, tens of ms for a
/// compaction), so blocking — not failing — is correct for the waiter.
///
/// Availability posture, same as the poison rule: the ledger must never stop writing. If the
/// lock FILE cannot even be opened (a permissions/anti-virus pathology — anywhere an append
/// could work, this open works too), we log loudly and proceed with the in-process lock only,
/// which is exactly yesterday's behaviour, not a new failure mode.
///
/// ★ **And the wait is BOUNDED — the correction the 0.3 inspection forced.** The first cut called
/// `lock_exclusive`, which is `LockFileEx` *without* `LOCKFILE_FAIL_IMMEDIATELY` / a blocking
/// `flock`: an unbounded park with no timeout facility in either OS API. Held inside the
/// process-global `FILE_LOCK`, one foreign holder stuck mid-hold (suspended under a crash dialog,
/// parked in a debugger, its rename stalled by an anti-virus scan) froze **every** ledger write in
/// this process forever — the awaiting command never returning, nothing surfaced. That traded a
/// rare lost record for a permanent silent hang, which is the worse bug: before 0.3 the wait was
/// bounded by this process's own threads, so the fix had made availability strictly worse.
///
/// So the OS lock is taken by `try_lock_exclusive` on a retry budget. The budget is set two orders
/// of magnitude above the guard's real hold times (µs for an append, tens of ms for a compaction),
/// so legitimate contention can never reach it; only a pathological holder can, and that holder now
/// costs a bounded wait and a loud line instead of the process. On exhaustion we take the SAME road
/// as an unopenable lock file — in-process exclusion only, pre-0.3 behaviour — because the aside
/// copy makes a lost append recoverable while a hung command is not.
struct LedgerGuard {
    _mutex: std::sync::MutexGuard<'static, ()>,
    os: Option<std::fs::File>, // exclusive OS lock; released on drop
}

impl LedgerGuard {
    /// Whether cross-process exclusion is actually in force for this operation.
    ///
    /// The fallback (below) is right for `append`: blocking there would hang a user's command,
    /// and an append that lands during a foreign compaction is still findable in the aside copy
    /// the compaction keeps. **That argument is circular for the compaction itself** — the rename
    /// is what CREATES the aside file, and nothing ever reads one back (that is exactly what
    /// bounds the load). So the destructive path asks this, and refuses rather than proceeding
    /// blind. It costs nothing: compaction is optional, `Refused` is a first-class outcome its
    /// caller already handles distinctly, and it re-runs every boot.
    fn cross_process(&self) -> bool {
        self.os.is_some()
    }
}

impl Drop for LedgerGuard {
    fn drop(&mut self) {
        if let Some(f) = self.os.take() {
            // Explicitly fs4's, not std's inherent `unlock` (stable since 1.89, and it would win
            // the method lookup silently): the lock was taken through fs4, so it is released
            // through fs4 — the pair can never drift onto two different OS calls.
            let _ = fs4::FileExt::unlock(&f);
        }
    }
}

/// The retry budget for the OS lock. Two orders of magnitude above a compaction's hold, so no
/// legitimate contention reaches it; a pathological holder costs this much and no more.
const OS_LOCK_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);
const OS_LOCK_RETRY: std::time::Duration = std::time::Duration::from_millis(2);

/// Take the exclusive OS lock, waiting at most `OS_LOCK_BUDGET`.
///
/// Only *contention* is retried. Any other error — a bad handle, a filesystem that does not
/// implement locking — cannot be fixed by waiting, so it returns immediately rather than burning
/// the budget to reach the same fallback.
fn lock_within_budget(f: &std::fs::File, budget: std::time::Duration) -> Result<(), String> {
    use fs4::FileExt;
    let contended = fs4::lock_contended_error().raw_os_error();
    let deadline = std::time::Instant::now() + budget;
    loop {
        match f.try_lock_exclusive() {
            Ok(()) => return Ok(()),
            Err(e)
                if e.raw_os_error() == contended
                    || e.kind() == std::io::ErrorKind::WouldBlock =>
            {
                if std::time::Instant::now() >= deadline {
                    return Err(format!("still held by another process after {budget:?}"));
                }
                std::thread::sleep(OS_LOCK_RETRY);
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

/// Record that cross-process exclusion is OFF for this operation — **durably**.
///
/// `eprintln!` alone was how the first cut announced it, and this subsystem already knows better:
/// `link_life_restore` carries the rule in-code ("NEVER `eprintln!` a failure a user needs —
/// Windows GUI release builds send stderr nowhere"), written after the Boss's 2026-07-27 restore
/// reported *"0 of 34 written"* with 33 records unaccounted for and no reason recorded anywhere.
/// `main.rs` is `windows_subsystem = "windows"` in release, so that channel is not merely quiet,
/// it does not exist.
///
/// What is being announced is not a lost write — the degraded state IS pre-0.3 behaviour, which
/// ships on main today. It is that a documented loss path has been silently re-opened: a Universe
/// on a share whose filesystem cannot lock runs an entire session with the 0.3 protection off,
/// every command returning Ok. Erasing that evidence is what turns a detectable degradation into
/// an undetectable one, so the line goes to the durable sink the user can actually open.
fn degraded(dir: &Path, why: &str) {
    let msg = format!(
        "[link-life] {why} in {} — proceeding with in-process exclusion only (pre-0.3 behaviour): \
         a SECOND INSTANCE could now compact this ledger under an append here",
        dir.display()
    );
    // `dir` IS the `.constellation` dir (see `store_dir`), and `diag_log` takes the search.db path
    // so callers never need to know the Universe root — the same shape link_life_restore uses.
    // It mirrors to stderr itself for dev builds, so there is no second `eprintln!` here.
    crate::search::diag_log(&dir.join("search.db"), &msg);
}

fn file_guard(dir: &Path) -> LedgerGuard {
    let mutex = FILE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let os = match std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(dir.join("ledger.lock"))
    {
        Ok(f) => match lock_within_budget(&f, OS_LOCK_BUDGET) {
            Ok(()) => Some(f),
            Err(e) => {
                degraded(dir, &format!("ledger.lock could not be locked ({e})"));
                None
            }
        },
        Err(e) => {
            degraded(dir, &format!("ledger.lock could not be opened ({e})"));
            None
        }
    };
    LedgerGuard { _mutex: mutex, os }
}

/// Append lines to a stream. ONE `write_all` per line including its `\n`, in append mode, so a
/// concurrent reader always sees whole lines and a crash can only truncate the last one.
///
/// Deliberately does NOT fsync — see the module docs on the 20× cost. Call `fsync` explicitly
/// at the sites where the only other copy is about to be destroyed.
///
/// Takes `FILE_LOCK` so an append can never be in flight while `maybe_compact` is moving the tail
/// out from under it. ~168 µs held; the walk path is already fire-and-forget off the click path.
pub fn append(dir: &Path, s: Stream, lines: &[String]) -> Result<(), String> {
    if lines.is_empty() {
        return Ok(());
    }
    let _g = file_guard(dir);
    let path = dir.join(s.file_name());
    let mut h = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("ledger open {}: {e}", path.display()))?;
    for line in lines {
        let mut buf = line.clone();
        if !buf.ends_with('\n') {
            buf.push('\n');
        }
        h.write_all(buf.as_bytes())
            .map_err(|e| format!("ledger append {}: {e}", path.display()))?;
    }
    Ok(())
}

/// Force the stream's bytes to disk. Use at the archive-before-purge site and after a user
/// decision; NOT on the walk path (Slice 0 measured 3.4 ms vs 168 µs).
pub fn fsync(dir: &Path, s: Stream) -> Result<(), String> {
    // Under the same lock as `append`: opening the tail while a compaction is renaming it would
    // otherwise recreate an empty file and sync that instead.
    let _g = file_guard(dir);
    let path = dir.join(s.file_name());
    let h = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("ledger open for fsync {}: {e}", path.display()))?;
    h.sync_all()
        .map_err(|e| format!("ledger fsync {}: {e}", path.display()))
}

fn parse_str(v: &serde_json::Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(|s| s.to_string())
}

/// The key a link-life record folds under. cid-pair first (portable, survives a rename); the
/// target NAME only as a fallback for an unresolved target.
fn earned_key(v: &serde_json::Value) -> Option<String> {
    let cid = parse_str(v, "cid")?;
    let to = parse_str(v, "to").unwrap_or_default();
    if !to.is_empty() {
        return Some(format!("{cid}>{to}"));
    }
    let tn = parse_str(v, "tn").unwrap_or_default();
    if tn.is_empty() {
        return None;
    }
    Some(format!("{cid}>~{}", tn.to_lowercase()))
}

/// An un-acknowledged quarantine sitting beside the store, if there is one.
///
/// **Why this is an existence check and not a flag someone remembered to set.** `quarantine`
/// returns a `LoadReport` with `refuse_write = true`, and until this slice that was the ONLY thing
/// that ever set it — so the guard in `link_life_restore` (*"do NOT write a thing from a store we
/// could not read"*) read as a live protection while being structurally unable to fire: the reader
/// built its own fresh report, in which the flag was always `false`. Found while wiring the
/// compactor's identical guard, and fixed here rather than duplicated (LL-035: a claim that a
/// protection is active is a RUNTIME claim, and only something observable can establish it).
///
/// The observable fact is the file. `quarantine` renames the suspect store to
/// `earned.corrupt-<stamp>.jsonl` and never deletes it, so its presence IS the un-acknowledged
/// state, and **acknowledging is the user moving or deleting that file** — File-Over-App, no UI
/// required and no hidden flag to get out of step with the disk.
pub fn quarantine_pending(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        (name.starts_with("earned.corrupt-") && name.ends_with(".jsonl")).then(|| e.path())
    })
}

/// Read one JSONL file, skipping (and counting) unparseable lines. Never throws on content.
/// PJ-385 — the same line reader, for the delete-archive reader in `deleted_notes.rs`.
///
/// Exposed rather than copied: it is the one place that treats an absent file as a FACT (an
/// empty store, not an error) and charges one unreadable line to `skipped_lines` instead of
/// discarding the file. A second implementation would drift from those two rules, and both are
/// exactly what a reader of a destroyed note's only record must get right.
pub(crate) fn read_archive_lines(path: &Path, report: &mut LoadReport) -> Vec<serde_json::Value> {
    // Takes the ledger lock, like every other reader in this module. The 2026-08-25 inspection
    // found the first version skipping it: this file is APPENDED to by every delete, and a read
    // racing an append can observe a torn final line and charge it to `skipped_lines` — turning a
    // healthy archive into one that reports itself as partly unreadable, on the surface whose
    // whole job is to say whether the record is complete.
    let _g = path.parent().map(file_guard);
    read_lines(path, report)
}

fn read_lines(path: &Path, report: &mut LoadReport) -> Vec<serde_json::Value> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        // Absent is a FACT: an empty store, not an error. Anything ELSE is a failure to look,
        // and "I could not look" must never be returned as "there is nothing there" — see
        // `LoadReport::unreadable_file`. Callers that do not consult the field keep their old
        // behaviour exactly (an empty Vec); the one surface where the difference is the whole
        // point (`deleted_notes_list`) refuses instead.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            report.unreadable_file = Some(e.to_string());
            return Vec::new();
        }
    };
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => out.push(v),
            // ONE bad line costs one line. Counted, and surfaced by the caller.
            Err(_) => report.skipped_lines += 1,
        }
    }
    out
}

/// Load the whole Stream-A state: snapshot + tail, both bounded. **THE ONLY FOLD IN THIS MODULE.**
///
/// Idempotent by ARITHMETIC, not by rule: `n` is written absolute and folds by max, so a
/// duplicated region, a re-appended restored copy, or a "keep both" merge resolution all fold
/// to the same answer. Later records win for decisions.
///
/// **The snapshot is read FIRST and the tail second, and that order is the correctness argument
/// for compaction** — the tail is, by construction, everything appended *after* the snapshot was
/// written, so plain file order already means "later wins" for decisions. It also means a crash
/// between writing the snapshot and renaming the tail aside is harmless: both are read, and the
/// duplicated region folds to the same answer it had before.
///
/// ★ **The read is UNDER THE GUARD — the second correction the 0.3 inspection forced.** That
/// crash argument covers a *crash*, where nothing runs afterwards. It does not cover a
/// **concurrent** compaction, and the two reads are two separate `read_to_string` calls: a
/// compaction that persists the new snapshot and renames the tail aside *between* them leaves the
/// reader folding the OLD snapshot plus an absent tail — and an absent tail is a FACT here, an
/// empty store, not an error. Every record that lived in the tail simply vanishes from the fold,
/// silently, with the load report reading perfectly normal.
///
/// The damage is the same shape as the append race and lands on the same data: `link_life_restore`
/// treats the fold as authoritative for decisions, so a stale fold does not merely omit them — it
/// **writes the pre-decision values back**, un-retiring a link the user retired, re-imposing a
/// priority they cleared. Cross-process reachability is not hypothetical and not only MIG-111's
/// end state: the Phase 0.2 owner lock is still record-only, so two instances can hold the same
/// Universe today, and each boot runs restore → compact over this same directory.
///
/// The guard has to live HERE rather than at the callers, because "every reader takes it" is
/// exactly the kind of promise a new caller silently breaks. `maybe_compact` is the one caller
/// that already holds it, and it calls `read_state_locked` — the non-reentrancy is discharged by
/// there being two names, not by remembering.
pub fn read_state(dir: &Path) -> (LedgerState, LoadReport) {
    let _g = file_guard(dir);
    read_state_locked(dir)
}

/// `read_state`'s body, for the one caller that is already inside the guard. Never `pub`: the
/// unlocked read is a compaction implementation detail, not a reading mode anyone may choose.
fn read_state_locked(dir: &Path) -> (LedgerState, LoadReport) {
    let mut report = LoadReport::default();
    // The refusal must be OBSERVED, not remembered — see `quarantine_pending`. Every reader now
    // gets the same answer from the same fact on disk, so a guard that says "refuse" can actually
    // fire, and a test can put it in the state that makes it fire.
    if let Some(p) = quarantine_pending(dir) {
        report.corrupt_renamed_to = Some(p);
        report.refuse_write = true;
    }
    let mut state = LedgerState::default();
    // Snapshot first, then the tail — the tail is newer by construction.
    let mut all = read_lines(&dir.join(SNAPSHOT_FILE), &mut report);
    all.extend(read_lines(&dir.join(Stream::Earned.file_name()), &mut report));
    for v in &all {
        let t = parse_str(v, "t").unwrap_or_default();
        // Stream B records can never enter the fold, even if a file is concatenated by hand.
        if matches!(t.as_str(), "nh" | "nd" | "nr") {
            continue;
        }
        // A seeded line's timestamp is DERIVED, not witnessed (Slice 5). Carried, not dropped.
        let seeded = v.get("seed").is_some();

        // A note-level decision has no target and therefore no link key. Before this slice the
        // key function returned `None` here and the record vanished from the loaded state.
        if t == "priority" {
            let Some(cid) = parse_str(v, "cid").filter(|c| !c.is_empty()) else { continue };
            let e = state.notes.entry(cid).or_default();
            if let Some(p) = v.get("p").and_then(|x| x.as_i64()) {
                e.p = p; // a DECISION: latest wins, including the `-1` that means "cleared"
            }
            // NON-EMPTY only — see the identical guard on the link branch below for why.
            if let Some(at) = parse_str(v, "at").filter(|s| !s.is_empty()) {
                e.at = Some(at);
                e.at_seeded = seeded;
            }
            continue;
        }

        let Some(key) = earned_key(v) else { continue };
        let e = state.links.entry(key).or_default();
        if let Some(n) = v.get("n").and_then(|x| x.as_i64()) {
            e.n = e.n.max(n); // MAX, so a replay can never ratchet a count down
        }
        if let Some(c) = parse_str(v, "conf") {
            e.conf = Some(c);
        }
        match t.as_str() {
            "retire" => e.status = Some("archived".to_string()),
            "restore" => e.status = Some("active".to_string()),
            _ => {}
        }
        // An EXPLICIT status field, which is what a folded `state` line carries. `retire` /
        // `restore` remain the sugar a live writer emits; this is the general form the snapshot
        // needs so one line can express a link that was walked AND retired.
        if let Some(s) = parse_str(v, "status").filter(|s| !s.is_empty()) {
            e.status = Some(s);
        }
        if let Some(tn) = parse_str(v, "tn").filter(|s| !s.is_empty()) {
            e.tn = Some(tn);
        }
        // NON-EMPTY only, and this is what makes the snapshot round trip EXACTLY. A record with
        // no timestamp folds to `at: None`; the snapshot has to write the field anyway (the field
        // order is part of the format), so it writes `"at":""` — which, read back naively, would
        // fold to `Some("")` and make the state after a compaction differ from the state before.
        // Nothing user-visible would break (every consumer treats `""` as absent), which is
        // exactly why it would have gone unnoticed. Treating empty as absent on the way IN keeps
        // "compaction cannot change what the store means" literally true.
        if let Some(at) = parse_str(v, "at").filter(|s| !s.is_empty()) {
            e.at = Some(at);
            e.at_seeded = seeded;
        }
    }
    (state, report)
}

/// The link half of `read_state`. A thin projection, deliberately NOT a second fold — one
/// implementation is what keeps the snapshot writer and every reader from ever disagreeing.
pub fn read_folded(dir: &Path) -> (FoldedMap, LoadReport) {
    let (state, report) = read_state(dir);
    (state.links, report)
}

/// Read a note's archived history, ordinal-ordered. **No fold** — every event survives.
pub fn read_history_for(dir: &Path, cid: &str) -> (Vec<HistRecord>, LoadReport) {
    let mut report = LoadReport::default();
    let mut out: Vec<HistRecord> = read_lines(&dir.join(Stream::NoteHistory.file_name()), &mut report)
        .into_iter()
        .filter(|v| parse_str(v, "cid").as_deref() == Some(cid))
        .filter(|v| parse_str(v, "t").as_deref() == Some("nh"))
        .map(|v| HistRecord {
            cid: cid.to_string(),
            hid: v.get("hid").and_then(|x| x.as_i64()).unwrap_or(0),
            at: v.get("at").and_then(|x| x.as_i64()).unwrap_or(0),
            raw: v.to_string(),
        })
        .collect();
    // By `hid`, NEVER by `at`: 765 `captured_at` groups collide across 1,536 live rows.
    // `sort_by_key` is stable, so equal hids keep file order.
    out.sort_by_key(|r| r.hid);
    (out, report)
}

/// The `.gitignore` that makes the File-Over-App claim operationally true (Boss decision #8).
///
/// Patterns are BY NAME, never the folder — that is the whole point. Excluding
/// `.constellation/` wholesale to skip the databases would exclude the earned data living in it,
/// in the same event it exists to survive. Measured: this list takes the live folder from
/// 2,836 MB to 38 KB, and `*.db` (not `search.db*`) is what also catches the orphaned 939 MB
/// `Constellation SV Test.db`.
pub const GITIGNORE_CONTENT: &str = "\
# Constellation — derived / machine state. NEVER exclude this folder wholesale:
# the earned-life ledger lives here and must travel with your notes (MIG-104).
*.db
*.db-wal
*.db-shm
boot-perf.*
diagnostics.log
sv-trace.log
mig108-journal.json
mig108-backup*/
";

/// Write the `.gitignore` once. NEVER overwrites — the user may have edited it.
pub fn ensure_gitignore(dir: &Path) -> Result<(), String> {
    let p = dir.join(".gitignore");
    if p.exists() {
        return Ok(());
    }
    std::fs::write(&p, GITIGNORE_CONTENT).map_err(|e| format!("write .gitignore: {e}"))
}

/// True when `name` is excluded by `GITIGNORE_CONTENT`. Kept beside the constant so the test
/// that asserts the live folder's contents cannot drift from the patterns.
pub fn gitignore_excludes(name: &str) -> bool {
    let n = name.to_lowercase();
    n.ends_with(".db")
        || n.ends_with(".db-wal")
        || n.ends_with(".db-shm")
        || n.starts_with("boot-perf.")
        || n == "diagnostics.log"
        || n == "sv-trace.log"
        || n == "mig108-journal.json"
        || n.starts_with("mig108-backup")
}

/// Fold Syncthing `.sync-conflict-*` copies back in, then remove them. Nearly free because the
/// Stream-A fold is already commutative and idempotent. Stream B is append-deduped by `hid`
/// rather than folded. Returns how many copies were adopted.
pub fn adopt_conflict_copies(dir: &Path) -> usize {
    let Ok(rd) = std::fs::read_dir(dir) else { return 0 };
    let mut adopted = 0usize;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.contains(".sync-conflict-") {
            continue;
        }
        let is_earned = name.starts_with("earned");
        let is_hist = name.starts_with("note-history");
        if !is_earned && !is_hist {
            continue;
        }
        let src = entry.path();
        let Ok(text) = std::fs::read_to_string(&src) else { continue };
        let target = if is_earned { Stream::Earned } else { Stream::NoteHistory };
        let lines: Vec<String> = text.lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_string()).collect();
        if append(dir, target, &lines).is_ok() {
            let _ = std::fs::remove_file(&src);
            adopted += 1;
        }
    }
    adopted
}

// ─── Slice 7: the snapshot + the compactor ───────────────────────────────────
//
// **The concept (the horse).** The cost of reading the ledger must be bounded by how much the
// user has EARNED, not by how long they have been using Constellation. Without this, a store that
// only ever grows is read in full on every boot forever — the one serious defect of an
// append-only design, and the reason a snapshot exists at all.
//
// **Why a byte threshold and never a timer.** An idle Universe must produce zero writes. A timer
// would rewrite a store nobody touched, which is both pointless I/O inside a watched folder and a
// recurring chance to corrupt a file that was perfectly fine.
//
// **The order is the safety argument, and it is the reverse of the intuitive one.** Write the new
// snapshot first, make it durable, and only THEN rename the tail aside:
//
//   1. build every line from the folded state          (pure, in memory)
//   2. write them to a UNIQUE temp in the same dir     (PJ-087: never a fixed `<name>.tmp`)
//   3. fsync the temp                                  (see below — this one is mandatory)
//   4. persist it over `earned.snapshot.jsonl`         (atomic rename within the volume)
//   5. rename `earned.jsonl` aside, NEVER delete it    (invariant #4)
//
// A crash at any point before 5 leaves snapshot + full tail, which the fold reads as the same
// state it already had — duplicated input, identical answer. A crash after 5 has the snapshot on
// disk holding everything the tail held. There is no window in which the state is only in a file
// that is not read.
//
// **Why step 3's fsync is not optional here, when the walk path's is.** The moment step 5 lands,
// the snapshot is the ONLY file the loader reads that contains the folded history — the
// renamed-aside tail is a safety copy, deliberately outside the load path so the load stays
// bounded. An unsynced snapshot plus a power loss would therefore cost the user real earned data
// that no automatic pass would recover. That is exactly the "the other copy is about to stop
// being read" test the fsync policy is written against.

/// Compaction fires when the TAIL alone reaches this. The snapshot is not counted: it is already
/// bounded by the number of earned links, which is what makes the whole scheme bounded.
///
/// 2 MB is ~10,000 records at the live line width (~200 B). Measured for scale, not guessed: the
/// live tail is **6,222 bytes** after seeding 33 earned links, so this Universe compacts for the
/// first time somewhere around its 10,000th recorded decision.
pub const COMPACT_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024;

/// What one compaction actually did — every number surfaced, none implied.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactReport {
    /// Lines written to the new snapshot (links + note decisions).
    pub lines: usize,
    /// Size of the tail that was folded in and renamed aside.
    pub tail_bytes: u64,
    /// Size of the snapshot that replaced it.
    pub snapshot_bytes: u64,
    /// Where the tail went. It is KEPT — this is a rename, never a delete.
    pub tail_renamed_to: PathBuf,
}

/// The three genuinely different outcomes, kept apart so a caller can never read "nothing to do"
/// as "done" or a refusal as a success (LL-035: a log line must be evidence, never intent).
#[derive(Debug, Clone, PartialEq)]
pub enum CompactOutcome {
    /// Under the threshold. **Nothing was read and nothing was written** — the idle case, and by
    /// far the common one.
    BelowThreshold { tail_bytes: u64 },
    /// Compaction was possible but REFUSED, with the reason. Never silently treated as success.
    Refused(&'static str),
    Compacted(CompactReport),
}

/// Pick a tail-aside name that cannot overwrite an existing one.
///
/// The stamp is second-granular, so two compactions inside one second would otherwise collide and
/// the second `rename` would silently destroy the first aside copy — turning "never delete" into
/// "usually never delete". Cheap to make structural.
fn unique_aside(dir: &Path, stamp: &str) -> PathBuf {
    let base = format!("earned.tail-{stamp}");
    let mut p = dir.join(format!("{base}.jsonl"));
    let mut i = 2;
    while p.exists() {
        p = dir.join(format!("{base}-{i}.jsonl"));
        i += 1;
    }
    p
}

/// Fold the tail into a bounded snapshot, once it is worth doing.
///
/// **There is no `Stream` parameter, and that is deliberate.** `note-history.jsonl` must NEVER be
/// compacted: its records ARE the payload, and folding two of them destroys the intermediate state
/// that is their entire value (the live rows `hid` 8251/8252/8253 record one property being typed,
/// `ma` → `mas` → `masadir`). If its size ever becomes a concern the only legal operation is time
/// segmentation into `note-history-<year>.jsonl`. Making the stream a parameter would turn that
/// prohibition into a code-review convention; leaving it out makes it a fact about the signature.
///
/// `stamp` is supplied by the caller — never a clock inside, so the tail-aside name is
/// deterministic under test (the same discipline `quarantine` follows).
pub fn maybe_compact(dir: &Path, stamp: &str) -> Result<CompactOutcome, String> {
    let tail = dir.join(Stream::Earned.file_name());
    // The threshold probe is deliberately OUTSIDE the lock: it is one `metadata()` call, it is
    // what runs on every boot of every Universe, and it must never contend with an appender.
    let tail_bytes = std::fs::metadata(&tail).map(|m| m.len()).unwrap_or(0);
    if tail_bytes < COMPACT_THRESHOLD_BYTES {
        return Ok(CompactOutcome::BelowThreshold { tail_bytes });
    }

    // ★ From here to the rename is ONE critical section, and it has to be: the whole operation is
    // "decide what the tail contains, then declare that content handled". An append landing
    // between those two halves is a record that no reader will ever see again. Held only on the
    // rare occasion the threshold is actually crossed — see `FILE_LOCK` for the failure it stops.
    let _g = file_guard(dir);
    // ★ And if that critical section is not actually exclusive across processes, we do not enter
    // it. See `LedgerGuard::cross_process` for why the append path's "proceed anyway" reasoning
    // does not carry here: this is the operation that moves records OUT of the load path.
    if !_g.cross_process() {
        return Ok(CompactOutcome::Refused(
            "cross-process exclusion unavailable — refusing to move the tail out of the load path \
             while another instance could be appending to it",
        ));
    }
    // Re-read the size under the lock. Between the probe and the lock, another compaction (or a
    // quarantine) may have already dealt with this tail.
    let tail_bytes = std::fs::metadata(&tail).map(|m| m.len()).unwrap_or(0);
    if tail_bytes < COMPACT_THRESHOLD_BYTES {
        return Ok(CompactOutcome::BelowThreshold { tail_bytes });
    }

    // The guard is already held — see `read_state`'s note on why the unlocked body has its own
    // name. Calling `read_state` here would deadlock on the non-reentrant mutex.
    let (state, load) = read_state_locked(dir);
    if load.refuse_write {
        // The store was structurally unusable and renamed aside. Writing a snapshot from a store
        // we could not read is how a compactor destroys the thing it exists to protect.
        return Ok(CompactOutcome::Refused(
            "store quarantined — refusing to snapshot from a store that could not be read",
        ));
    }
    if state.links.is_empty() && state.notes.is_empty() {
        // A megabytes-long tail that folds to nothing is not an empty store; it is a store we do
        // not understand. Replacing it with an empty snapshot would read as a successful
        // compaction while moving every byte out of the load path.
        return Ok(CompactOutcome::Refused(
            "the fold is empty while the tail is not — refusing to snapshot nothing over something",
        ));
    }

    let lines = snapshot_lines(&state);
    let snapshot = dir.join(SNAPSHOT_FILE);

    // Step 2 — a UNIQUE temp in the SAME directory (same volume, so the persist is a rename and
    // not a copy). PJ-087: never `universe::atomic_write`'s fixed `<name>.tmp`, which two writers
    // can collide on.
    let mut tmp = tempfile::Builder::new()
        .prefix("earned.snapshot.")
        .suffix(".tmp")
        .tempfile_in(dir)
        .map_err(|e| format!("compact temp in {}: {e}", dir.display()))?;
    for line in &lines {
        let mut buf = line.clone();
        if !buf.ends_with('\n') {
            buf.push('\n');
        }
        tmp.write_all(buf.as_bytes())
            .map_err(|e| format!("compact write: {e}"))?;
    }
    // Step 3 — mandatory here; see the note above the threshold constant.
    tmp.as_file()
        .sync_all()
        .map_err(|e| format!("compact fsync: {e}"))?;
    // Step 4 — atomic replace. If THIS fails the tail is untouched and the old snapshot still
    // stands: the store is exactly as it was, and the next boot simply tries again.
    let snapshot_bytes = tmp.as_file().metadata().map(|m| m.len()).unwrap_or(0);
    tmp.persist(&snapshot)
        .map_err(|e| format!("compact persist {}: {e}", snapshot.display()))?;

    // Step 5 — the tail is RENAMED, never deleted. Everything in it is now in the snapshot, but
    // "now in the snapshot" is a claim about code, and the aside copy is what makes the claim
    // recoverable if the claim is ever wrong.
    let dest = unique_aside(dir, stamp);
    std::fs::rename(&tail, &dest)
        .map_err(|e| format!("compact rename tail to {}: {e}", dest.display()))?;

    Ok(CompactOutcome::Compacted(CompactReport {
        lines: lines.len(),
        tail_bytes,
        snapshot_bytes,
        tail_renamed_to: dest,
    }))
}

/// The store is structurally unusable (not merely one bad line) → rename it aside and REFUSE to
/// write a fresh one until acknowledged. `stamp` is supplied by the caller (never `Date::now`
/// inside, so the name is deterministic in tests).
pub fn quarantine(dir: &Path, s: Stream, stamp: &str) -> LoadReport {
    // Renames the live store — same exclusion as compaction, but deliberately NOT the same
    // refusal. Compaction is optional and re-runs every boot, so refusing it when the OS lock is
    // unavailable costs nothing; quarantine exists to stop the app reading a store it could not
    // parse, and refusing THAT on a lockless filesystem would leave it reading the corruption.
    // (Reachable from tests only today — the corrupt-store path is not yet wired to a caller.)
    let _g = file_guard(dir);
    let src = dir.join(s.file_name());
    let dest = dir.join(format!("earned.corrupt-{stamp}.jsonl"));
    let mut report = LoadReport::default();
    if std::fs::rename(&src, &dest).is_ok() {
        report.corrupt_renamed_to = Some(dest);
    }
    // A blind overwrite would destroy the backup that was about to save the user.
    report.refuse_write = true;
    report
}

/// Slice 4 toggle. `false` = today's behaviour exactly, byte-for-byte: no file is created and
/// no writer runs. Kept as a `const` so the dead branch is compiled out entirely rather than
/// costing a check on the traverse path.
pub const EARNED_LEDGER_WRITE: bool = true;

// ─── Record builders ─────────────────────────────────────────────────────────
//
// The on-disk FORMAT lives here and nowhere else, so a writer cannot invent a variant and the
// format test has one thing to assert against.
//
// Field order is FIXED and meaningful — the file is meant to be read by a human in a text editor,
// where `v,t,cid,to,tn,n,at` reads as a sentence and the alphabetical `at,cid,n,t,tn,to,v` does
// not. `serde_json::json!` CANNOT deliver that: without the `preserve_order` feature its Map is a
// BTreeMap and it sorts keys. (The first cut of this module claimed otherwise in a comment; the
// format test caught it.) Enabling `preserve_order` globally would change every JSON write in the
// app, so the lines are built here with an explicit ordered writer instead — values still escaped
// by serde, so the output is always valid JSON.
//
// `cid` = the SOURCE note's identity; `to` = the TARGET's identity when resolvable; `tn` = the
// target's name, which is the fallback key AND the only human-legible part of the line.
//
// Q2 (Boss-ruled): the key is TYPE-FREE — `[[supports::X]]` and `[[derives-from::X]]` from one
// note fold to ONE record, because all four DB writers already match on source + target name and
// ignore `link_type`. Re-typing a link therefore keeps its earned history.

/// Write an ordered JSON object. Values are escaped by serde (so the line is always valid JSON);
/// only the ORDER is ours. `Val` keeps the call sites readable.
enum Val<'a> {
    S(&'a str),
    I(i64),
}

fn obj(fields: &[(&str, Val)]) -> String {
    let mut out = String::from("{");
    for (i, (k, v)) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&serde_json::to_string(k).unwrap_or_default());
        out.push(':');
        match v {
            Val::S(x) => out.push_str(&serde_json::to_string(x).unwrap_or_default()),
            Val::I(x) => out.push_str(&x.to_string()),
        }
    }
    out.push('}');
    out
}

/// A traversal. `n` is written ABSOLUTE (never a delta) — that is what makes the fold idempotent.
pub fn walk_line(src_cid: &str, tgt_cid: &str, tgt_name: &str, n: i64, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("walk")),
        ("cid", Val::S(src_cid)),
        ("to", Val::S(tgt_cid)),
        ("tn", Val::S(tgt_name)),
        ("n", Val::I(n)),
        ("at", Val::S(at)),
    ])
}

/// A confidence judgment. Only ever a USER judgment — never the auto-tier derivable from `n`
/// (≥10 established, ≥3 evidence), because recording a derivable value would fill the ledger
/// with events that carry no decision.
pub fn trust_line(src_cid: &str, tgt_cid: &str, tgt_name: &str, conf: &str, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("trust")),
        ("cid", Val::S(src_cid)),
        ("to", Val::S(tgt_cid)),
        ("tn", Val::S(tgt_name)),
        ("conf", Val::S(conf)),
        ("at", Val::S(at)),
    ])
}

/// Retiring a link. Archival, not deletion — the wikilink deliberately stays in the note, which
/// is exactly why this must be durable: a rebuild from the notes alone would resurrect it.
pub fn retire_line(src_cid: &str, tgt_cid: &str, tgt_name: &str, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("retire")),
        ("cid", Val::S(src_cid)),
        ("to", Val::S(tgt_cid)),
        ("tn", Val::S(tgt_name)),
        ("at", Val::S(at)),
    ])
}

/// Un-retiring a link.
pub fn restore_line(src_cid: &str, tgt_cid: &str, tgt_name: &str, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("restore")),
        ("cid", Val::S(src_cid)),
        ("to", Val::S(tgt_cid)),
        ("tn", Val::S(tgt_name)),
        ("at", Val::S(at)),
    ])
}

/// A review-priority decision on a NOTE (no target).
pub fn priority_line(cid: &str, p: i64, at: &str) -> String {
    obj(&[
        ("v", Val::I(1)),
        ("t", Val::S("priority")),
        ("cid", Val::S(cid)),
        ("p", Val::I(p)),
        ("at", Val::S(at)),
    ])
}

/// A FOLDED link record — one line carrying the whole current state of one earned link. This is
/// the snapshot's vocabulary, and the only record type that is a *conclusion* rather than an
/// *event*: `walk` / `trust` / `retire` say what happened, `state` says where things stand.
///
/// `conf` and `status` are omitted when unset, so a link that was only ever walked still reads as
/// one short, obvious sentence in a text editor.
pub fn state_line(
    cid: &str,
    to: &str,
    tn: &str,
    n: i64,
    conf: Option<&str>,
    status: Option<&str>,
    at: &str,
) -> String {
    let mut f: Vec<(&str, Val)> = vec![
        ("v", Val::I(1)),
        ("t", Val::S("state")),
        ("cid", Val::S(cid)),
        ("to", Val::S(to)),
        ("tn", Val::S(tn)),
        ("n", Val::I(n)),
    ];
    if let Some(c) = conf {
        f.push(("conf", Val::S(c)));
    }
    if let Some(s) = status {
        f.push(("status", Val::S(s)));
    }
    f.push(("at", Val::S(at)));
    obj(&f)
}

/// Render the entire folded state as the lines of a snapshot.
///
/// **Sorted by key**, for three reasons that all matter more than the microseconds: a human
/// scanning the file gets a stable order, two compactions of the same state produce byte-identical
/// files (which is what `compaction_is_lossless` can then assert), and a git diff of the store
/// shows what actually changed instead of a rehashed `HashMap` order.
///
/// Every line must fold back to the key it was written from — pinned by
/// `every_snapshot_line_folds_back_to_its_own_key`. That is the property the whole slice rests on:
/// if it does not hold, compaction silently re-keys the user's earned history.
pub fn snapshot_lines(state: &LedgerState) -> Vec<String> {
    let mut out = Vec::with_capacity(state.links.len() + state.notes.len());

    let mut keys: Vec<&String> = state.links.keys().collect();
    keys.sort();
    for k in keys {
        let e = &state.links[k];
        let (cid, rest) = match k.split_once('>') {
            Some(p) => p,
            None => continue, // not a key this module ever writes
        };
        // Reconstruct the two halves the key encodes: identity-keyed carries the target's cid,
        // name-keyed carries `~name` and no identity at all.
        let (to, name_from_key) = match rest.strip_prefix('~') {
            Some(name) => ("", name),
            None => (rest, ""),
        };
        // Prefer the label as last written (real capitalisation); fall back to the key's lowered
        // copy. Either folds to the same key, because `earned_key` lowercases the name.
        let tn = e.tn.as_deref().unwrap_or(name_from_key);
        let line = state_line(
            cid,
            to,
            tn,
            e.n,
            e.conf.as_deref(),
            e.status.as_deref(),
            e.at.as_deref().unwrap_or(""),
        );
        out.push(if e.at_seeded { mark_seeded(line) } else { line });
    }

    let mut cids: Vec<&String> = state.notes.keys().collect();
    cids.sort();
    for cid in cids {
        let e = &state.notes[cid];
        let line = priority_line(cid, e.p, e.at.as_deref().unwrap_or(""));
        out.push(if e.at_seeded { mark_seeded(line) } else { line });
    }
    out
}

/// Mark a line as SEEDED rather than observed — its timestamp is derived, not witnessed.
///
/// MIG-104 Slice 5, Boss-found 2026-07-27: the back-fill has no "when was this decided" column to
/// read (`note_links` carries `created` and `last_traversed`, nothing else), so a seeded `trust` or
/// `retire` necessarily borrows `last_traversed`. The Boss's own data made the gap visible — a
/// Contested click at 09:21:25 was seeded as 09:13:51, the time of the walk. The timestamp cannot
/// be made true, so the RECORD says it is derived. A reader (human or restore) can then tell a
/// witnessed decision from a reconstructed one, and a future re-seed can be told apart from real
/// activity. Additive: readers that do not know the field ignore it.
pub fn mark_seeded(line: String) -> String {
    match line.rfind('}') {
        Some(i) => format!("{},\"seed\":1{}", &line[..i], &line[i..]),
        None => line,
    }
}

/// The confidence tier DERIVABLE from a traversal count — the same thresholds
/// `constellation_link_traverse` applies. Not stored in the ledger, because it is a function of
/// `n`; computed wherever it is needed, exactly like `weight`.
pub fn auto_tier(n: i64) -> &'static str {
    if n >= 10 {
        "established"
    } else if n >= 3 {
        "evidence"
    } else {
        "hypothesis"
    }
}

/// True when `conf` is merely the tier derivable from `n` — i.e. carries no user judgment and
/// must NOT be recorded.
pub fn is_derivable_tier(conf: &str, n: i64) -> bool {
    conf == auto_tier(n)
}

/// Rank a confidence tier so a restore can never DOWNGRADE a user's judgment to a derived one.
/// `contested` is a deliberate stance, not a rung on the ladder, so it outranks everything.
pub fn conf_rank(conf: &str) -> u8 {
    match conf {
        "contested" => 4,
        "established" => 3,
        "evidence" => 2,
        _ => 1,
    }
}

#[cfg(test)]
mod tests_0_3_cross_process {
    //! MIG-111 Phase 0.3 — the verification clause: two REAL processes, no line lost.
    use super::*;
    use std::path::PathBuf;

    /// The child body: appends numbered lines to the Earned tail as fast as it can until the
    /// stop marker vanishes. Ignored in normal runs; the parent invokes it explicitly.
    #[test]
    #[ignore = "child-process helper for the 0.3 two-process conservation test"]
    fn ledger_appender_child() {
        let (Ok(dir), Ok(stop)) = (
            std::env::var("CONSTELLATION_LEDGER_DIR"),
            std::env::var("CONSTELLATION_LEDGER_STOP"),
        ) else {
            return;
        };
        let dir = PathBuf::from(dir);
        let stop = PathBuf::from(stop);
        let mut i = 0u32;
        while stop.exists() && i < 5_000 {
            append(&dir, Stream::Earned, &[format!("CHILD-{i}")]).expect("child append");
            i += 1;
        }
        // Tell the parent how many we wrote.
        std::fs::write(dir.join("child.count"), i.to_string()).unwrap();
    }

    /// **Conservation across processes.** A real second process appends continuously while THIS
    /// process repeatedly renames the tail aside under the guard (`quarantine` — the same
    /// rename-under-append critical section as compaction, minus the fold). With the OS lock,
    /// every appended line must survive INTACT, EXACTLY ONCE, somewhere findable (current tail
    /// or an aside file) — no vanished lines, no torn fragments, no duplicates. Before 0.3 the
    /// two processes' guards were invisible to each other, so a rename could slide under an
    /// in-flight append (`FILE_SHARE_DELETE`) — the Slice-7 shape across the process boundary.
    #[test]
    fn two_process_append_vs_rename_loses_nothing() {
        let t = tempfile::TempDir::new().unwrap();
        let dir = t.path().to_path_buf();
        let stop = dir.join("keep-going");
        std::fs::write(&stop, b"1").unwrap();

        let exe = std::env::current_exe().unwrap();
        let mut child = std::process::Command::new(&exe)
            .args([
                "link_life::tests_0_3_cross_process::ledger_appender_child",
                "--exact",
                "--nocapture",
                "--include-ignored",
            ])
            .env("CONSTELLATION_LEDGER_DIR", &dir)
            .env("CONSTELLATION_LEDGER_STOP", &stop)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn appender child");

        // Interleave: this process appends its own lines AND renames the tail aside repeatedly,
        // all through the real public API (the same critical sections production uses).
        for k in 0..40u32 {
            append(&dir, Stream::Earned, &[format!("PARENT-{k}")]).expect("parent append");
            if k % 8 == 7 {
                let _ = quarantine(&dir, Stream::Earned, &format!("t{k}"));
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        // Stop the child, collect its count.
        std::fs::remove_file(&stop).unwrap();
        let _ = child.wait();
        let child_n: u32 = std::fs::read_to_string(dir.join("child.count"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert!(child_n > 0, "the child must actually have appended under contention");

        // Gather every line from the current tail + every aside file.
        let mut seen: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            if name == Stream::Earned.file_name() || name.starts_with("earned.corrupt-") {
                if let Ok(txt) = std::fs::read_to_string(&path) {
                    for line in txt.lines() {
                        *seen.entry(line.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        for i in 0..child_n {
            let key = format!("CHILD-{i}");
            assert_eq!(seen.get(&key), Some(&1), "child line {key} must survive exactly once");
        }
        for k in 0..40u32 {
            let key = format!("PARENT-{k}");
            assert_eq!(seen.get(&key), Some(&1), "parent line {key} must survive exactly once");
        }
        // And nothing torn: every surviving line is one of ours, whole.
        for (line, n) in &seen {
            assert_eq!(*n, 1, "duplicated line: {line}");
            assert!(
                line.starts_with("CHILD-") || line.starts_with("PARENT-"),
                "torn or foreign fragment in the ledger: {line:?}"
            );
        }
    }

    /// The child body for the READER test: takes the real guard, announces that it holds it,
    /// keeps it for `HOLD`, then releases. It performs the compaction's own two-step in that
    /// window — persist a new snapshot, rename the tail aside — so a reader that slipped inside
    /// would fold the old snapshot against an absent tail.
    #[test]
    #[ignore = "child-process helper for the 0.3 reader-exclusion test"]
    fn ledger_lock_holder_child() {
        let Ok(dir) = std::env::var("CONSTELLATION_LEDGER_DIR") else { return };
        let dir = PathBuf::from(dir);
        let _g = file_guard(&dir);
        std::fs::write(dir.join("child.holding"), b"1").unwrap();
        std::thread::sleep(HOLD);
        // A FAITHFUL compaction — the module's own fold and serializer, then the real two-step in
        // its dangerous order. Faithful matters: the snapshot it leaves behind holds everything
        // the tail held, so if the reader still ends up missing a record, the miss is the race
        // and not a lossy fixture.
        let (state, _) = read_state_locked(&dir);
        let mut text = String::new();
        for line in snapshot_lines(&state) {
            text.push_str(&line);
            text.push('\n');
        }
        std::fs::write(dir.join(SNAPSHOT_FILE), text).unwrap();
        let _ = std::fs::rename(
            dir.join(Stream::Earned.file_name()),
            dir.join("earned.tail-child.jsonl"),
        );
        std::fs::write(dir.join("child.done"), b"1").unwrap();
    }

    const HOLD: std::time::Duration = std::time::Duration::from_millis(500);

    /// **A reader may not walk into a compaction.** `read_state` reads two files; a foreign
    /// process that persists the new snapshot and renames the tail aside *between* them leaves
    /// the reader folding the old snapshot plus an absent tail — and the restore then writes
    /// those pre-decision values back over the user's decisions.
    ///
    /// RED before the fix: `read_state` took no lock at all, so this returned in microseconds
    /// while the child held the guard. The elapsed-time assertion IS the exclusion claim — the
    /// only thing that can make a two-file read atomic against another process.
    #[test]
    fn read_state_waits_for_a_foreign_holder() {
        let t = tempfile::TempDir::new().unwrap();
        let dir = t.path().to_path_buf();
        append(
            &dir,
            Stream::Earned,
            &[
                r#"{"v":1,"t":"walk","cid":"C_A","to":"C_B","n":7}"#.to_string(),
                r#"{"v":1,"t":"retire","cid":"C_A","to":"C_B","at":"2026-08-15T10:00:00Z"}"#
                    .to_string(),
            ],
        )
        .unwrap();

        let exe = std::env::current_exe().unwrap();
        let mut child = std::process::Command::new(&exe)
            .args([
                "link_life::tests_0_3_cross_process::ledger_lock_holder_child",
                "--exact",
                "--nocapture",
                "--include-ignored",
            ])
            .env("CONSTELLATION_LEDGER_DIR", &dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn lock-holder child");

        // Wait for the child to actually hold it — otherwise we would be timing nothing.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !dir.join("child.holding").exists() {
            assert!(std::time::Instant::now() < deadline, "child never took the lock");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let t0 = std::time::Instant::now();
        let (state, report) = read_state(&dir);
        let waited = t0.elapsed();
        let _ = child.wait();

        assert!(
            waited >= HOLD / 2,
            "read_state returned in {waited:?} while another process held the ledger guard — \
             the two-file read is not excluded against a concurrent compaction"
        );
        assert!(dir.join("child.done").exists(), "the child must have finished its rename first");
        // And having waited, the fold is whole: the decision that lived in the tail is present.
        assert_eq!(report.skipped_lines, 0);
        assert_eq!(
            state.links.get("C_A>C_B").and_then(|e| e.status.clone()).as_deref(),
            Some("archived"),
            "the retire record must survive the compaction the reader waited out"
        );
    }

    /// **Without cross-process exclusion, compaction REFUSES.** The append path proceeds when the
    /// OS lock is unavailable, and that is right — blocking would hang a user's command, and the
    /// appended record is still findable in the aside copy. The compaction is the operation that
    /// CREATES that aside copy, and nothing ever reads one back, so proceeding blind there is the
    /// one place the fallback could actually destroy a decision: a second instance appending a
    /// retire during the fold-to-rename window would have it moved out of the load path, and the
    /// next boot's restore would write the pre-decision value back over it.
    ///
    /// A directory named `ledger.lock` makes the lock file unopenable — the same `os: None` state
    /// a share with no lock support produces, deterministically.
    #[test]
    fn compaction_refuses_when_it_cannot_exclude_another_process() {
        let d = tempfile::TempDir::new().unwrap();
        // A tail well past the threshold, so the only thing that can stop it is the refusal.
        let mut lines = Vec::new();
        for i in 0..40_000 {
            lines.push(format!(
                r#"{{"v":1,"t":"walk","cid":"C_A","to":"C_B{i}","n":3}}"#
            ));
        }
        append(d.path(), Stream::Earned, &lines).unwrap();
        assert!(
            std::fs::metadata(d.path().join(Stream::Earned.file_name())).unwrap().len()
                >= COMPACT_THRESHOLD_BYTES,
            "the fixture must actually cross the threshold, or this proves nothing"
        );

        // The append above already created the lock FILE; replace it with a directory.
        let lock = d.path().join("ledger.lock");
        std::fs::remove_file(&lock).unwrap();
        std::fs::create_dir(&lock).unwrap();
        let out = maybe_compact(d.path(), "NOLOCK").unwrap();
        assert!(
            matches!(out, CompactOutcome::Refused(_)),
            "expected a refusal with the exclusion unavailable, got {out:?}"
        );
        // And the refusal is real: the tail is untouched, nothing moved out of the load path.
        assert!(d.path().join(Stream::Earned.file_name()).exists(), "the tail must still be live");
        assert!(!d.path().join(SNAPSHOT_FILE).exists(), "and no snapshot was written over it");
        assert_eq!(
            read_state(d.path()).0.links.len(),
            40_000,
            "every record is still in the load path"
        );
    }

    /// **Losing the OS lock must leave EVIDENCE ON DISK.** The first cut announced the fallback
    /// with `eprintln!` only — and `main.rs` is `windows_subsystem = "windows"` in release, so
    /// that channel does not exist for the user. A Universe on a share whose filesystem cannot
    /// lock would then run an entire session with the 0.3 protection off, every command returning
    /// Ok, with nothing anywhere recording that a documented loss path had been re-opened. This
    /// pins the durable sink, which is the whole content of the fix.
    #[test]
    fn the_degraded_fallback_is_recorded_where_a_human_can_read_it() {
        let t = tempfile::TempDir::new().unwrap();
        degraded(t.path(), "ledger.lock could not be opened (simulated)");
        let log = std::fs::read_to_string(t.path().join("diagnostics.log"))
            .expect("the fallback must reach diagnostics.log, not just stderr");
        assert!(log.contains("in-process exclusion only"), "it must say WHAT was lost: {log}");
        assert!(log.contains("SECOND INSTANCE"), "and why that matters: {log}");
    }

    /// **The wait is bounded.** A stuck foreign holder must cost a bounded wait and a loud line,
    /// never the process: the first 0.3 cut called `lock_exclusive`, an unbounded park held
    /// inside the process-global mutex, which would have frozen every ledger write here forever.
    #[test]
    fn a_held_lock_times_out_instead_of_parking_forever() {
        use fs4::FileExt;
        let t = tempfile::TempDir::new().unwrap();
        let path = t.path().join("ledger.lock");
        let holder = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        holder.lock_exclusive().unwrap();

        // A second HANDLE in this process is a genuine second lock owner to the OS.
        let waiter = std::fs::OpenOptions::new().read(true).write(true).open(&path).unwrap();
        let budget = std::time::Duration::from_millis(120);
        let t0 = std::time::Instant::now();
        let outcome = lock_within_budget(&waiter, budget);
        let waited = t0.elapsed();

        assert!(outcome.is_err(), "a held lock must not be reported as acquired");
        assert!(waited >= budget, "it must actually have waited its budget, not failed fast");
        assert!(
            waited < budget * 8,
            "it must give up near the budget, not park indefinitely (waited {waited:?})"
        );

        holder.unlock().unwrap();
        assert!(
            lock_within_budget(&waiter, budget).is_ok(),
            "and once free, the very next attempt takes it"
        );
    }
}

#[cfg(test)]
mod tests_mig104_link_life {
    use super::*;

    fn dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }
    fn line(json: serde_json::Value) -> String {
        json.to_string()
    }

    #[test]
    fn append_writes_exactly_one_line_with_lf() {
        let d = dir();
        append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":1}))]).unwrap();
        let raw = std::fs::read(d.path().join("earned.jsonl")).unwrap();
        assert_eq!(raw.iter().filter(|b| **b == b'\n').count(), 1);
        assert!(!raw.contains(&b'\r'), "no CRLF — the ledger must read byte-correctly on macOS");
        assert_eq!(*raw.last().unwrap(), b'\n');
    }

    /// 2026-08-25 inspection, HIGH false-success. `read_lines` mapped EVERY `read_to_string`
    /// error to an empty Vec, so "I could not open the file" and "the file is not there" became
    /// the same answer — and the Deleted-notes surface renders the second as *"The record exists
    /// and is empty — no removal has been recorded in this universe."* about an archive that may
    /// be the last surviving record of destroyed notes.
    ///
    /// A directory is used as the unreadable path because it is the one input that fails
    /// `read_to_string` with a NON-NotFound error on every platform this ships to, without
    /// needing permissions the test runner may not have. What is asserted is the DISTINCTION, so
    /// this cannot pass by accident: absent leaves the field `None`, unreadable sets it, and both
    /// still return an empty Vec so no existing caller changes behaviour.
    #[test]
    fn an_unreadable_store_is_not_reported_as_an_empty_one() {
        let dir = tempfile::tempdir().unwrap();

        // 1. Genuinely absent — a FACT, and must stay one.
        let mut absent = LoadReport::default();
        let out = read_archive_lines(&dir.path().join("note-history.jsonl"), &mut absent);
        assert!(out.is_empty());
        assert_eq!(absent.unreadable_file, None, "an absent store must not read as unreadable");

        // 2. Present and readable — the control. If this ever set the field, the assertion in (3)
        //    would be worthless because it could not have failed.
        let real = dir.path().join("real.jsonl");
        std::fs::write(&real, "{\"v\":1,\"t\":\"del\",\"cid\":\"c\"}
").unwrap();
        let mut ok = LoadReport::default();
        assert_eq!(read_archive_lines(&real, &mut ok).len(), 1);
        assert_eq!(ok.unreadable_file, None);

        // 3. Exists but cannot be read.
        let unreadable = dir.path().join("as_a_directory");
        std::fs::create_dir(&unreadable).unwrap();
        let mut bad = LoadReport::default();
        let out = read_archive_lines(&unreadable, &mut bad);
        assert!(out.is_empty(), "still empty — callers that ignore the field are unaffected");
        assert!(
            bad.unreadable_file.is_some(),
            "a store that exists and cannot be read must be distinguishable from an empty one;              without this, `deleted_notes_list` returns archivePresent=true with zero notes and              the UI states as fact that nothing was ever deleted"
        );
    }

    #[test]
    fn torn_tail_loses_only_the_last_line() {
        let d = dir();
        for n in 1..=3 {
            append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":n}))]).unwrap();
        }
        // Simulate a kill mid-append: truncate inside the final line.
        let p = d.path().join("earned.jsonl");
        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, &text[..text.len() - 8]).unwrap();
        let (map, report) = read_folded(d.path());
        assert_eq!(report.skipped_lines, 1, "exactly the torn line is lost, and it is COUNTED");
        assert_eq!(map.get("A>B").unwrap().n, 2, "every earlier record survives — an append cannot clobber");
    }

    #[test]
    fn fold_is_commutative_and_idempotent() {
        let mk = |ns: &[i64]| {
            let d = dir();
            for n in ns {
                append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":n}))]).unwrap();
            }
            read_folded(d.path()).0
        };
        assert_eq!(mk(&[3, 8, 5]), mk(&[5, 3, 8]), "order cannot change the answer");
        assert_eq!(mk(&[4]), mk(&[4, 4, 4]), "a duplicated region folds to one copy's answer");
    }

    #[test]
    fn max_fold_never_decreases_n() {
        let d = dir();
        for n in [9, 2, 1] {
            append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":n}))]).unwrap();
        }
        assert_eq!(read_folded(d.path()).0.get("A>B").unwrap().n, 9);
    }

    #[test]
    fn a_later_decision_wins_while_the_count_still_maxes() {
        let d = dir();
        append(d.path(), Stream::Earned, &[
            line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":9})),
            line(serde_json::json!({"v":1,"t":"retire","cid":"A","to":"B","at":"2026-07-18T08:20:02Z"})),
        ]).unwrap();
        let e = read_folded(d.path()).0.get("A>B").cloned().unwrap();
        assert_eq!(e.status.as_deref(), Some("archived"));
        assert_eq!(e.n, 9);
    }

    /// THE distinction the whole module is built around.
    #[test]
    fn history_never_folds() {
        let d = dir();
        // The real collision shape: same `captured_at`, ordinals out of file order.
        append(d.path(), Stream::NoteHistory, &[
            line(serde_json::json!({"v":1,"t":"nh","cid":"C","hid":8252,"at":1785131711000i64,"ev":{"to":"mas"}})),
            line(serde_json::json!({"v":1,"t":"nh","cid":"C","hid":8251,"at":1785131711000i64,"ev":{"to":"ma"}})),
            line(serde_json::json!({"v":1,"t":"nh","cid":"C","hid":8253,"at":1785131711000i64,"ev":{"to":"masadir"}})),
        ]).unwrap();
        let (recs, report) = read_history_for(d.path(), "C");
        assert_eq!(report.skipped_lines, 0);
        assert_eq!(recs.len(), 3, "a thought being typed must survive as three events, never folded to one");
        assert_eq!(recs.iter().map(|r| r.hid).collect::<Vec<_>>(), vec![8251, 8252, 8253],
            "ordered by the row ordinal — `at` is identical on all three");
        // And a history record can never leak into the link-life fold.
        assert!(read_folded(d.path()).0.is_empty());
    }

    #[test]
    fn unparseable_line_is_skipped_and_counted() {
        let d = dir();
        let p = d.path().join("earned.jsonl");
        std::fs::write(&p, "{\"v\":1,\"t\":\"walk\",\"cid\":\"A\",\"to\":\"B\",\"n\":1}\nnot json at all\n{\"v\":1,\"t\":\"walk\",\"cid\":\"A\",\"to\":\"B\",\"n\":5}\n").unwrap();
        let (map, report) = read_folded(d.path());
        assert_eq!(report.skipped_lines, 1);
        assert_eq!(map.get("A>B").unwrap().n, 5, "the good lines on BOTH sides of the bad one load");
    }

    #[test]
    fn structurally_corrupt_store_is_renamed_aside_and_refuses_fresh_write() {
        let d = dir();
        append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":1}))]).unwrap();
        let report = quarantine(d.path(), Stream::Earned, "2026-07-27T000000Z");
        let aside = report.corrupt_renamed_to.expect("renamed aside, never deleted");
        assert!(aside.exists(), "the suspect store is KEPT — it may be the only copy left");
        assert!(!d.path().join("earned.jsonl").exists());
        assert!(report.refuse_write, "must refuse a fresh write until acknowledged");
    }

    #[test]
    fn gitignore_excludes_every_db_in_the_live_folder_but_no_ledger() {
        for excluded in [
            "search.db", "search.db-wal", "search.db-shm",
            "Constellation SV Test.db", // the orphaned 939 MB one — `search.db*` would MISS it
            "boot-perf.latest.json", "boot-perf.history.jsonl",
            "diagnostics.log", "sv-trace.log",
            "mig108-journal.json", "mig108-backup", "mig108-backup.prev", // MIG-108 crash state — single-machine, never synced
        ] {
            assert!(gitignore_excludes(excluded), "must be excluded from sync: {excluded}");
        }
        for kept in ["earned.jsonl", "earned.snapshot.jsonl", "note-history.jsonl",
                     "settings.json", "libraries.json", "universe.json", "review-pulse.json"] {
            assert!(!gitignore_excludes(kept), "must TRAVEL with the user's notes: {kept}");
        }
    }

    #[test]
    fn ensure_gitignore_never_overwrites_the_users_edit() {
        let d = dir();
        ensure_gitignore(d.path()).unwrap();
        std::fs::write(d.path().join(".gitignore"), "# mine\n").unwrap();
        ensure_gitignore(d.path()).unwrap();
        assert_eq!(std::fs::read_to_string(d.path().join(".gitignore")).unwrap(), "# mine\n");
    }

    #[test]
    fn adopts_a_sync_conflict_copy_then_removes_it() {
        let d = dir();
        append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":2}))]).unwrap();
        let conflict = d.path().join("earned.sync-conflict-20260727-120000-ABCDEF.jsonl");
        std::fs::write(&conflict, line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"B","n":7})) + "\n").unwrap();
        assert_eq!(adopt_conflict_copies(d.path()), 1);
        assert!(!conflict.exists(), "the copy is folded in, then removed");
        assert_eq!(read_folded(d.path()).0.get("A>B").unwrap().n, 7, "the higher count wins by max-fold");
    }

    #[test]
    fn an_absent_store_is_a_fact_not_an_error() {
        let d = dir();
        let (map, report) = read_folded(d.path());
        assert!(map.is_empty());
        assert_eq!(report.skipped_lines, 0);
        assert!(!report.refuse_write, "absent is an empty store; only UNREADABLE refuses a write");
    }

    #[test]
    fn keys_are_os_portable() {
        // The fallback key path must never carry a drive letter or a backslash.
        let d = dir();
        append(d.path(), Stream::Earned, &[line(serde_json::json!({"v":1,"t":"walk","cid":"A","to":"","tn":"The Four Books","n":1}))]).unwrap();
        let (map, _) = read_folded(d.path());
        let k = map.keys().next().unwrap();
        assert!(!k.contains('\\') && !k.contains(':'), "no path separators or drive letters in a key: {k}");
        assert_eq!(k, "A>~the four books", "an unresolved target folds case-insensitively by name");
    }
}

#[cfg(test)]
mod tests_mig104_compact {
    //! MIG-104 Slice 7 — the snapshot + compactor.
    //!
    //! The property every test here circles: **compaction may change how the store is written,
    //! never what it means.** `read_state` before == `read_state` after, for every record kind
    //! the writers can emit.
    use super::*;

    fn dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    /// Fill the tail past the threshold with real records, so compaction has something to do.
    /// Returns how many distinct links were written.
    fn fill_past_threshold(d: &Path) -> usize {
        // ~143 B per line at these key widths, so 700 × 24 ≈ 2.4 MB clears the threshold with
        // margin. Distinct links, ascending counts — the shape a real store reaches after long use.
        let links = 700;
        let mut lines = Vec::new();
        for round in 1..=24 {
            for i in 0..links {
                lines.push(walk_line(
                    &format!("20260703T091101Z_NOTE_{i:04X}"),
                    &format!("20260512T144233Z_NOTE_{i:04X}"),
                    "the four books",
                    round,
                    "2026-07-03T09:11:05Z",
                ));
            }
        }
        append(d, Stream::Earned, &lines).unwrap();
        assert!(
            std::fs::metadata(d.join("earned.jsonl")).unwrap().len() >= COMPACT_THRESHOLD_BYTES,
            "fixture must actually cross the threshold or the test proves nothing"
        );
        links as usize
    }

    /// THE headline property: the store's MEANING is identical across a compaction.
    #[test]
    fn compaction_is_lossless() {
        let d = dir();
        let links = fill_past_threshold(d.path());
        // Every other record kind too, so "lossless" is not proven on walks alone.
        append(d.path(), Stream::Earned, &[
            trust_line("C_A", "C_B", "B", "contested", "2026-07-18T08:17:49Z"),
            retire_line("C_A", "C_C", "banana", "2026-07-18T08:20:02Z"),
            mark_seeded(walk_line("C_A", "C_D", "D", 5, "2026-07-01T00:00:00Z")),
            walk_line("C_A", "", "unresolved target", 2, "2026-07-02T00:00:00Z"),
            priority_line("C_NOTE", 2, "2026-07-18T08:21:00Z"),
        ]).unwrap();

        let (before, rep_before) = read_state(d.path());
        assert_eq!(rep_before.skipped_lines, 0);

        let out = maybe_compact(d.path(), "2026-07-27T120000Z").unwrap();
        let CompactOutcome::Compacted(r) = out else { panic!("expected a compaction, got {out:?}") };
        assert_eq!(r.lines, links + 4 + 1, "one line per link, plus the note decision");

        let (after, rep_after) = read_state(d.path());
        assert_eq!(rep_after.skipped_lines, 0, "the snapshot must be parseable in full");
        assert_eq!(before, after, "compaction changed the store's MEANING — that is the one thing it may not do");
        assert!(r.snapshot_bytes < r.tail_bytes, "…and it must actually be smaller");
    }

    /// Every kind of decision must survive, individually named so a failure says which one broke.
    #[test]
    fn every_record_kind_survives_a_compaction() {
        let d = dir();
        fill_past_threshold(d.path());
        append(d.path(), Stream::Earned, &[
            walk_line("C_S", "C_T", "Target Note", 9, "2026-07-01T00:00:00Z"),
            trust_line("C_S", "C_T", "Target Note", "contested", "2026-07-02T00:00:00Z"),
            retire_line("C_S", "C_T", "Target Note", "2026-07-03T00:00:00Z"),
            priority_line("C_S", 7, "2026-07-04T00:00:00Z"),
        ]).unwrap();
        maybe_compact(d.path(), "S").unwrap();

        let (st, _) = read_state(d.path());
        let e = st.links.get("C_S>C_T").expect("the link survived");
        assert_eq!(e.n, 9, "the walk count");
        assert_eq!(e.conf.as_deref(), Some("contested"), "the user's judgment");
        assert_eq!(e.status.as_deref(), Some("archived"), "the retirement — a walked AND retired link needs BOTH on one line");
        assert_eq!(e.tn.as_deref(), Some("Target Note"), "the human-legible label, with its real capitalisation");
        assert_eq!(st.notes.get("C_S").map(|n| n.p), Some(7), "the note-level decision");
    }

    /// Slice 4 appends `priority` records and fsyncs them. Before this slice the fold dropped
    /// every one (the key function required a target), so a fold-and-rewrite compactor would have
    /// moved them permanently out of the loaded store. RED without the `notes` map.
    #[test]
    fn a_note_priority_survives_the_fold_and_the_compaction() {
        let d = dir();
        append(d.path(), Stream::Earned, &[priority_line("C_NOTE", 3, "2026-07-18T08:21:00Z")]).unwrap();
        let (st, _) = read_state(d.path());
        assert_eq!(st.notes.get("C_NOTE").map(|n| n.p), Some(3), "a review priority is earned data, not noise");

        // …and the LAST decision wins, including clearing it (`-1`).
        append(d.path(), Stream::Earned, &[priority_line("C_NOTE", -1, "2026-07-19T00:00:00Z")]).unwrap();
        assert_eq!(read_state(d.path()).0.notes.get("C_NOTE").map(|n| n.p), Some(-1), "clearing a priority is itself a decision");

        fill_past_threshold(d.path());
        maybe_compact(d.path(), "S").unwrap();
        assert_eq!(read_state(d.path()).0.notes.get("C_NOTE").map(|n| n.p), Some(-1), "and it survives compaction");
    }

    /// The property the whole slice rests on. If a snapshot line folded to a DIFFERENT key than
    /// the record it was written from, compaction would silently re-key the user's history — the
    /// count would still be there, attached to the wrong link.
    #[test]
    fn every_snapshot_line_folds_back_to_its_own_key() {
        let d = dir();
        append(d.path(), Stream::Earned, &[
            walk_line("C_A", "C_B", "Capitalised Name", 3, "t1"),      // identity-keyed
            walk_line("C_A", "", "Banana", 4, "t2"),                    // name-keyed, mixed case
            walk_line("C_A", "", "قهوة", 1, "t3"),                      // name-keyed, non-Latin
            retire_line("C_A", "C_E", "E", "t4"),                       // a decision with n = 0
        ]).unwrap();
        let (state, _) = read_state(d.path());
        assert_eq!(state.links.len(), 4);

        for (key, line) in state.links.keys().cloned().collect::<Vec<_>>().iter().zip(snapshot_lines(&state)) {
            let _ = key; // ordering differs; the assertion below is over the whole set
            let v: serde_json::Value = serde_json::from_str(&line).expect("a snapshot line is valid JSON");
            let k = earned_key(&v).expect("a snapshot line must key");
            assert!(state.links.contains_key(&k), "snapshot line re-keyed to something the fold never had: {line}");
        }
        // And the round trip as a whole is an identity.
        let d2 = dir();
        append(d2.path(), Stream::Earned, &snapshot_lines(&state)).unwrap();
        assert_eq!(read_state(d2.path()).0, state, "writing the state and reading it back must be an identity");
    }

    /// Invariant #4. The tail holds the user's raw history; "it is all in the snapshot now" is a
    /// claim about code, and the aside copy is what makes that claim recoverable if it is wrong.
    #[test]
    fn compaction_renames_the_tail_never_deletes_it() {
        let d = dir();
        fill_past_threshold(d.path());
        let original = std::fs::read_to_string(d.path().join("earned.jsonl")).unwrap();

        let CompactOutcome::Compacted(r) = maybe_compact(d.path(), "2026-07-27T120000Z").unwrap()
            else { panic!("expected a compaction") };

        assert!(!d.path().join("earned.jsonl").exists(), "the tail is moved out of the load path…");
        assert!(r.tail_renamed_to.exists(), "…but it still EXISTS");
        assert_eq!(std::fs::read_to_string(&r.tail_renamed_to).unwrap(), original,
            "and it is byte-identical — a rename, not a rewrite");
        assert_eq!(r.tail_renamed_to.file_name().unwrap().to_string_lossy(),
            "earned.tail-2026-07-27T120000Z.jsonl");
    }

    /// The renamed-aside tail must sit OUTSIDE the load path — otherwise compaction bounds
    /// nothing and every boot re-reads the whole history it just archived.
    #[test]
    fn the_aside_tail_is_not_read_back_so_the_load_stays_bounded() {
        let d = dir();
        fill_past_threshold(d.path());
        maybe_compact(d.path(), "S").unwrap();
        // A second pass sees only the (small) snapshot: the tail is gone from the load path, so
        // there is nothing left to compact.
        assert!(matches!(maybe_compact(d.path(), "S2").unwrap(), CompactOutcome::BelowThreshold { tail_bytes: 0 }));
        let snap = std::fs::metadata(d.path().join(SNAPSHOT_FILE)).unwrap().len();
        assert!(snap < COMPACT_THRESHOLD_BYTES, "the bounded copy is what boot reads: {snap} bytes");
    }

    /// Two compactions in the same second must not let the second silently destroy the first
    /// aside copy — that would turn "never delete" into "usually never delete".
    #[test]
    fn two_compactions_in_one_second_keep_both_aside_copies() {
        let d = dir();
        fill_past_threshold(d.path());
        let CompactOutcome::Compacted(a) = maybe_compact(d.path(), "SAME").unwrap() else { panic!() };
        fill_past_threshold(d.path());
        let CompactOutcome::Compacted(b) = maybe_compact(d.path(), "SAME").unwrap() else { panic!() };
        assert_ne!(a.tail_renamed_to, b.tail_renamed_to);
        assert!(a.tail_renamed_to.exists() && b.tail_renamed_to.exists(), "both survive");
    }

    /// PJ-087 regression: a FIXED `<name>.tmp` is what two writers collide on. `tempfile_in`
    /// gives every attempt its own name, in the same directory so the persist stays a rename.
    #[test]
    fn temp_names_are_unique_and_land_in_the_same_directory() {
        let d = dir();
        let mk = || tempfile::Builder::new().prefix("earned.snapshot.").suffix(".tmp")
            .tempfile_in(d.path()).unwrap();
        let (a, b, c) = (mk(), mk(), mk());
        let names: std::collections::HashSet<_> =
            [&a, &b, &c].iter().map(|t| t.path().to_path_buf()).collect();
        assert_eq!(names.len(), 3, "a fixed temp name is the PJ-087 collision");
        assert_eq!(a.path().parent().unwrap(), d.path(),
            "same directory ⇒ same volume ⇒ persist is an atomic rename, not a copy");
    }

    /// An idle Universe must produce ZERO writes. A timer-driven compactor would rewrite a store
    /// nobody touched — pointless I/O inside a watched folder, and a fresh chance to corrupt a
    /// file that was fine.
    #[test]
    fn threshold_is_bytes_not_time_so_an_idle_store_is_never_rewritten() {
        let d = dir();
        append(d.path(), Stream::Earned, &[walk_line("C_A", "C_B", "B", 1, "t")]).unwrap();
        let before = std::fs::read(d.path().join("earned.jsonl")).unwrap();
        let stamp_before = std::fs::metadata(d.path().join("earned.jsonl")).unwrap().modified().unwrap();

        for i in 0..100 {
            let out = maybe_compact(d.path(), &format!("cycle-{i}")).unwrap();
            assert!(matches!(out, CompactOutcome::BelowThreshold { .. }), "cycle {i} wrote something");
        }
        assert_eq!(std::fs::read(d.path().join("earned.jsonl")).unwrap(), before);
        assert_eq!(std::fs::metadata(d.path().join("earned.jsonl")).unwrap().modified().unwrap(), stamp_before);
        assert!(!d.path().join(SNAPSHOT_FILE).exists(), "no snapshot is created for an idle store");
        // MIG-111 0.3 — `ledger.lock` (the cross-process guard's OS-lock file) is a permanent
        // resident of the dir, not a leftover; the census excludes it BY NAME so any OTHER new
        // file still fails this invariant.
        let residents: Vec<String> = std::fs::read_dir(d.path())
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
            .filter(|n| n != "ledger.lock")
            .collect();
        assert_eq!(residents.len(), 1, "no aside copies, no temps left behind: {residents:?}");
    }

    /// Stream B's records ARE the payload; folding two of them destroys the intermediate state
    /// that is their whole value. The compactor cannot be pointed at it — there is no parameter
    /// to point — and this asserts the file is untouched even when it is the larger of the two.
    #[test]
    fn note_history_is_never_compacted() {
        let d = dir();
        // The real shape: one property being typed, three events, one timestamp.
        let hist: Vec<String> = (8251..=8253).map(|hid| serde_json::json!({
            "v":1,"t":"nh","cid":"C","hid":hid,"at":1785131711000i64,"ev":{"to":"ma"}
        }).to_string()).collect();
        let mut bulk = hist.clone();
        for i in 0..40_000 {
            bulk.push(serde_json::json!({"v":1,"t":"nh","cid":"C","hid":30000+i,"at":1785131711000i64,"ev":{"x":i}}).to_string());
        }
        append(d.path(), Stream::NoteHistory, &bulk).unwrap();
        let hp = d.path().join("note-history.jsonl");
        let before = std::fs::read(&hp).unwrap();
        assert!(before.len() as u64 > COMPACT_THRESHOLD_BYTES, "the history file is over the threshold…");

        fill_past_threshold(d.path());
        maybe_compact(d.path(), "S").unwrap();

        assert_eq!(std::fs::read(&hp).unwrap(), before, "…and compaction must not have touched one byte of it");
        assert_eq!(read_history_for(d.path(), "C").0.len(), 40_003, "every event still individually there");
        assert!(!std::fs::read_dir(d.path()).unwrap().flatten()
            .any(|e| e.file_name().to_string_lossy().starts_with("note-history.tail-")),
            "no aside copy of the history stream may ever be produced");
    }

    /// The crash window. Between the snapshot landing and the tail being renamed aside, BOTH
    /// files are in the load path — which is exactly why the snapshot is written first.
    #[test]
    fn a_crash_between_the_snapshot_and_the_rename_loses_nothing() {
        let d = dir();
        fill_past_threshold(d.path());
        append(d.path(), Stream::Earned, &[
            trust_line("C_A", "C_B", "B", "contested", "t1"),
            priority_line("C_N", 4, "t2"),
        ]).unwrap();
        let (before, _) = read_state(d.path());

        // Reproduce the interrupted state: snapshot written, tail NOT yet renamed.
        let lines = snapshot_lines(&before);
        std::fs::write(d.path().join(SNAPSHOT_FILE),
            lines.iter().map(|l| format!("{l}\n")).collect::<String>()).unwrap();

        let (mid, rep) = read_state(d.path());
        assert_eq!(rep.skipped_lines, 0);
        assert_eq!(mid, before, "reading snapshot + the whole tail is the SAME state — duplicated input, identical answer");

        // Finishing the job afterwards is still an identity.
        maybe_compact(d.path(), "S").unwrap();
        assert_eq!(read_state(d.path()).0, before);
    }

    /// The documented revert path: `cat` the snapshot and every aside tail back into
    /// `earned.jsonl`, delete the snapshot. Tested, not just written in a commit message.
    #[test]
    fn the_revert_recipe_restores_a_single_pre_slice7_file() {
        let d = dir();
        fill_past_threshold(d.path());
        append(d.path(), Stream::Earned, &[
            retire_line("C_A", "C_B", "B", "t1"),
            priority_line("C_N", 1, "t2"),
        ]).unwrap();
        let (before, _) = read_state(d.path());
        maybe_compact(d.path(), "STAMP").unwrap();

        // The recipe, exactly as the commit message states it.
        let mut merged = std::fs::read_to_string(d.path().join(SNAPSHOT_FILE)).unwrap();
        for e in std::fs::read_dir(d.path()).unwrap().flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n.starts_with("earned.tail-") {
                merged.push_str(&std::fs::read_to_string(e.path()).unwrap());
                std::fs::remove_file(e.path()).unwrap();
            }
        }
        std::fs::write(d.path().join("earned.jsonl"), merged).unwrap();
        std::fs::remove_file(d.path().join(SNAPSHOT_FILE)).unwrap();

        let (after, rep) = read_state(d.path());
        assert_eq!(rep.skipped_lines, 0);
        assert_eq!(after, before, "a reverted store means exactly what it meant before Slice 7");
    }

    /// A store we could not read must never be the source of a rewrite — that is precisely how a
    /// compactor destroys the data it exists to protect. Driven through the REAL path: `quarantine`
    /// leaves an `earned.corrupt-*.jsonl` behind, and that file IS the un-acknowledged state.
    #[test]
    fn an_unacknowledged_quarantine_refuses_compaction() {
        let d = dir();
        fill_past_threshold(d.path());
        // The store went structurally unusable at some point and was renamed aside; the user has
        // not dealt with it yet, and a new tail has accrued since.
        std::fs::write(d.path().join("earned.corrupt-2026-07-27T000000Z.jsonl"), "\u{0}\u{0}garbage").unwrap();

        let out = maybe_compact(d.path(), "S").unwrap();
        assert!(matches!(out, CompactOutcome::Refused(_)), "expected a refusal, got {out:?}");
        assert!(!d.path().join(SNAPSHOT_FILE).exists(), "nothing may have been written…");
        assert!(d.path().join("earned.jsonl").exists(), "…and the tail is left exactly where it was");

        // Acknowledging is the user moving the file away. Then it compacts normally.
        std::fs::remove_file(d.path().join("earned.corrupt-2026-07-27T000000Z.jsonl")).unwrap();
        assert!(matches!(maybe_compact(d.path(), "S").unwrap(), CompactOutcome::Compacted(_)));
    }

    /// ★ The guard above could not fire before this slice. `refuse_write` was set ONLY inside
    /// `quarantine`, which hands back its own report — so every reader's report carried `false`,
    /// and `link_life_restore`'s *"do NOT write a thing from a store we could not read"* was a
    /// dead branch that read as a live protection. This pins the fix: the reader now OBSERVES the
    /// quarantine on disk (LL-035 — a protection being active is a runtime claim).
    #[test]
    fn the_reader_reports_a_pending_quarantine_so_the_refuse_guard_can_actually_fire() {
        let d = dir();
        append(d.path(), Stream::Earned, &[walk_line("C_A", "C_B", "B", 1, "t")]).unwrap();
        assert!(!read_state(d.path()).1.refuse_write, "a healthy store refuses nothing");

        let aside = quarantine(d.path(), Stream::Earned, "2026-07-27T000000Z")
            .corrupt_renamed_to.expect("renamed aside");
        let (_, report) = read_state(d.path());
        assert!(report.refuse_write, "the guard must fire from what is ON DISK, not from a flag someone passed along");
        assert_eq!(report.corrupt_renamed_to.as_deref(), Some(aside.as_path()), "…and it must name the file to deal with");

        std::fs::remove_file(&aside).unwrap();
        assert!(!read_state(d.path()).1.refuse_write, "acknowledging is moving the file away");
    }

    /// A megabytes-long tail that folds to nothing is not an empty store — it is a store we do not
    /// understand. Replacing it with an empty snapshot would read as a successful compaction while
    /// moving every byte out of the load path.
    #[test]
    fn a_tail_that_folds_to_nothing_refuses_rather_than_snapshotting_emptiness() {
        let d = dir();
        std::fs::write(d.path().join("earned.jsonl"), "x".repeat(COMPACT_THRESHOLD_BYTES as usize + 1)).unwrap();
        let out = maybe_compact(d.path(), "S").unwrap();
        assert!(matches!(out, CompactOutcome::Refused(_)), "expected a refusal, got {out:?}");
        assert!(!d.path().join(SNAPSHOT_FILE).exists());
        assert!(d.path().join("earned.jsonl").exists(), "the bytes we could not read are KEPT");
    }

    /// A refusal is not a success. Kept as three distinct outcomes so no caller can log
    /// "compacted" for a store it declined to touch (LL-035: a log line is evidence, not intent).
    #[test]
    fn the_three_outcomes_are_distinguishable() {
        let d = dir();
        assert!(matches!(maybe_compact(d.path(), "S").unwrap(), CompactOutcome::BelowThreshold { tail_bytes: 0 }));
        fill_past_threshold(d.path());
        assert!(matches!(maybe_compact(d.path(), "S").unwrap(), CompactOutcome::Compacted(_)));
    }

    /// A seeded record's timestamp is DERIVED, not witnessed (Slice 5, Boss-found). Compaction
    /// must not quietly relabel it as observed.
    #[test]
    fn the_seeded_marker_survives_compaction_and_a_live_record_clears_it() {
        let d = dir();
        fill_past_threshold(d.path());
        append(d.path(), Stream::Earned, &[
            mark_seeded(walk_line("C_SEED", "C_T", "T", 2, "2026-07-01T00:00:00Z")),
            walk_line("C_LIVE", "C_T", "T", 2, "2026-07-01T00:00:00Z"),
        ]).unwrap();
        maybe_compact(d.path(), "S").unwrap();

        let snap = std::fs::read_to_string(d.path().join(SNAPSHOT_FILE)).unwrap();
        let seeded_line = snap.lines().find(|l| l.contains("C_SEED")).unwrap();
        let live_line = snap.lines().find(|l| l.contains("C_LIVE")).unwrap();
        assert!(seeded_line.contains("\"seed\":1"), "a reconstructed timestamp must still say so");
        assert!(!live_line.contains("\"seed\""), "a witnessed one must not be marked as derived");
        assert!(serde_json::from_str::<serde_json::Value>(seeded_line).is_ok(), "still valid JSON");

        let (st, _) = read_state(d.path());
        assert!(st.links["C_SEED>C_T"].at_seeded);
        assert!(!st.links["C_LIVE>C_T"].at_seeded);
    }

    /// A snapshot line is meant to be read by a human in a text editor. If the label were dropped,
    /// a compacted store would be a wall of opaque identities — File-Over-App in name only.
    #[test]
    fn a_snapshot_line_is_still_legible_to_a_human() {
        let d = dir();
        fill_past_threshold(d.path());
        append(d.path(), Stream::Earned, &[
            walk_line("20260703T091101Z_NOTE_A1B2", "20260512T144233Z_NOTE_77C9", "the four books", 3, "2026-07-03T09:11:05Z"),
            retire_line("20260703T091101Z_NOTE_A1B2", "20260512T144233Z_NOTE_77C9", "the four books", "2026-07-18T08:20:02Z"),
        ]).unwrap();
        maybe_compact(d.path(), "S").unwrap();
        let snap = std::fs::read_to_string(d.path().join(SNAPSHOT_FILE)).unwrap();
        let l = snap.lines().find(|l| l.contains("A1B2")).unwrap();
        assert_eq!(l, r#"{"v":1,"t":"state","cid":"20260703T091101Z_NOTE_A1B2","to":"20260512T144233Z_NOTE_77C9","tn":"the four books","n":3,"status":"archived","at":"2026-07-18T08:20:02Z"}"#);
    }

    /// A store whose SNAPSHOT is large, so the write+fsync window between "decide what the tail
    /// contains" and "declare it handled" is wide. `fill_past_threshold` writes many rounds over
    /// few links and therefore snapshots to only 700 lines — fast to write, and the race window
    /// closes before an appender can reach it. The exclusion bug is about that window, so the
    /// fixture that tests it has to open it.
    fn fill_with_many_distinct_links(d: &Path) -> usize {
        let links = 20_000;
        let lines: Vec<String> = (0..links).map(|i| walk_line(
            &format!("20260703T091101Z_NOTE_{i:05}"),
            &format!("20260512T144233Z_NOTE_{i:05}"),
            "the four books", 1, "2026-07-03T09:11:05Z",
        )).collect();
        append(d, Stream::Earned, &lines).unwrap();
        assert!(std::fs::metadata(d.join("earned.jsonl")).unwrap().len() >= COMPACT_THRESHOLD_BYTES);
        links
    }

    /// ★ THE MECHANISM, deterministically. No threads, no timing: perform by hand exactly what an
    /// interleaved append does to a compaction — fold the store, THEN let a decision land, THEN
    /// finish the compaction from the stale fold. The record ends up in the aside tail, which no
    /// reader touches, so it is gone from the store while every step reported success.
    ///
    /// This test does not exercise the guard (the next one does); it pins WHY the guard exists, so
    /// that a future change which reopens the window has something that explains the damage.
    #[test]
    fn a_stale_fold_strands_a_decision_in_the_aside_tail() {
        let d = dir();
        fill_past_threshold(d.path());

        // 1. What the compactor read.
        let (stale, _) = read_state(d.path());
        // 2. The user retires a link while the snapshot is being written and fsync'd.
        append(d.path(), Stream::Earned,
            &[retire_line("C_LATE", "C_TGT", "target", "2026-07-27T10:00:00Z")]).unwrap();
        assert!(read_state(d.path()).0.links.contains_key("C_LATE>C_TGT"), "it IS in the store now");
        // 3. The compactor finishes from its stale fold and renames the tail aside.
        std::fs::write(d.path().join(SNAPSHOT_FILE),
            snapshot_lines(&stale).iter().map(|l| format!("{l}\n")).collect::<String>()).unwrap();
        let aside = d.path().join("earned.tail-STALE.jsonl");
        std::fs::rename(d.path().join("earned.jsonl"), &aside).unwrap();

        // The decision is gone from the store — and the file it is in is never read back.
        let (after, _) = read_state(d.path());
        assert!(!after.links.contains_key("C_LATE>C_TGT"),
            "this is the damage the exclusion prevents");
        assert!(std::fs::read_to_string(&aside).unwrap().contains("C_LATE"),
            "the bytes survive on disk — but nothing in the app will ever look at them again");
    }

    /// ★ THE SAFETY-INSPECTION FINDING (2026-07-27), reproduced. A decision appended WHILE a
    /// compaction is in flight was moved into the aside tail — which nothing ever reads back — so
    /// the record vanished from the store with every step reporting success.
    ///
    /// Worse than a missing line: the restore treats the ledger as authoritative for decisions, so
    /// on the NEXT boot the fold still carries the pre-decision value, disagrees with the DB, and
    /// **writes the old value back** — silently un-retiring a link or reverting a priority.
    ///
    /// RED without `FILE_LOCK`: verified by removing the guard from `maybe_compact`, which loses
    /// records on essentially every run (the compaction writes ~2.4 MB while the appender runs).
    #[test]
    fn a_decision_appended_during_a_compaction_is_never_lost() {
        let d = dir();
        fill_with_many_distinct_links(d.path());
        let path = d.path().to_path_buf();

        // The live shape: a user retiring links from command threads while the boot thread
        // compacts. The appender runs until compaction has RETURNED, so its writes are guaranteed
        // to span the read→write→fsync→rename window rather than racing it — the first cut used a
        // fixed count and passed by timing luck even with the exclusion removed, which would have
        // shipped a regression test that could not see the regression.
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        use std::sync::Arc;
        let stop = Arc::new(AtomicBool::new(false));
        let written = Arc::new(AtomicUsize::new(0));
        let (s2, w2) = (stop.clone(), written.clone());
        let appender = std::thread::spawn(move || {
            while !s2.load(Ordering::Relaxed) {
                let i = w2.load(Ordering::Relaxed);
                append(&path, Stream::Earned, &[retire_line(
                    &format!("C_DECISION_{i:05}"), &format!("C_T_{i:05}"), "t", "2026-07-27T10:00:00Z",
                )]).unwrap();
                w2.store(i + 1, Ordering::Relaxed);
            }
        });
        let out = maybe_compact(d.path(), "RACE").unwrap();
        stop.store(true, Ordering::Relaxed);
        appender.join().unwrap();
        assert!(matches!(out, CompactOutcome::Compacted(_)), "the fixture must actually compact");
        let n = written.load(Ordering::Relaxed);
        assert!(n > 0, "the appender must have written during the compaction");

        // Every single decision must still be in the LOADED state — not merely on disk somewhere.
        let (state, _) = read_state(d.path());
        let missing: Vec<usize> = (0..n)
            .filter(|i| !state.links.contains_key(&format!("C_DECISION_{i:05}>C_T_{i:05}")))
            .collect();
        assert!(missing.is_empty(), "{} of {n} decisions were moved out of the load path: {:?}",
            missing.len(), &missing[..missing.len().min(10)]);
        for i in 0..n {
            let e = &state.links[&format!("C_DECISION_{i:05}>C_T_{i:05}")];
            assert_eq!(e.status.as_deref(), Some("archived"), "record {i} lost its decision");
        }
    }

    /// The same exclusion, from the other side: a compaction must not start while an append is
    /// mid-flight, or on Windows the rename succeeds under the open handle (`FILE_SHARE_DELETE`)
    /// and the appender keeps writing into a file nobody reads.
    #[test]
    fn appends_and_compaction_never_interleave() {
        let d = dir();
        fill_with_many_distinct_links(d.path());
        let path = d.path().to_path_buf();
        let handles: Vec<_> = (0..4).map(|t| {
            let p = path.clone();
            std::thread::spawn(move || {
                for i in 0..50 {
                    append(&p, Stream::Earned,
                        &[walk_line(&format!("C_T{t}_{i:02}"), "C_X", "x", 1, "t")]).unwrap();
                }
            })
        }).collect();
        maybe_compact(d.path(), "RACE2").unwrap();
        for h in handles { h.join().unwrap(); }

        let (state, report) = read_state(d.path());
        assert_eq!(report.skipped_lines, 0, "no line may be torn by an interleaved write");
        for t in 0..4 {
            for i in 0..50 {
                assert!(state.links.contains_key(&format!("C_T{t}_{i:02}>C_X")),
                    "thread {t} record {i} was lost");
            }
        }
    }

    /// A record with no timestamp must round-trip as *no timestamp*. The snapshot writes the
    /// field regardless (field order is part of the format), so `""` on the way out has to read
    /// back as absent on the way in — otherwise compaction changes the state it just wrote.
    #[test]
    fn a_record_with_no_timestamp_round_trips_as_absent_not_as_empty_string() {
        let d = dir();
        // A hand-written / truncated line, which is the shape that actually produces this.
        std::fs::write(d.path().join("earned.jsonl"),
            "{\"v\":1,\"t\":\"walk\",\"cid\":\"C_A\",\"to\":\"C_B\",\"tn\":\"B\",\"n\":3}\n\
             {\"v\":1,\"t\":\"priority\",\"cid\":\"C_N\",\"p\":2}\n").unwrap();
        let (before, _) = read_state(d.path());
        assert_eq!(before.links["C_A>C_B"].at, None);
        assert_eq!(before.notes["C_N"].at, None);

        // Write the snapshot and read it back — the identity compaction relies on.
        let d2 = dir();
        append(d2.path(), Stream::Earned, &snapshot_lines(&before)).unwrap();
        assert_eq!(read_state(d2.path()).0, before, "an absent timestamp must not become an empty one");
    }

    /// Sorted output: a human gets a stable order, and two compactions of the same state produce
    /// byte-identical files instead of a rehashed `HashMap` order.
    #[test]
    fn the_snapshot_is_deterministic_across_runs() {
        let d = dir();
        append(d.path(), Stream::Earned, &[
            walk_line("C_C", "C_X", "x", 1, "t"), walk_line("C_A", "C_X", "x", 1, "t"),
            walk_line("C_B", "C_X", "x", 1, "t"), priority_line("C_Z", 1, "t"),
        ]).unwrap();
        let (st, _) = read_state(d.path());
        let a = snapshot_lines(&st);
        assert_eq!(a, snapshot_lines(&read_state(d.path()).0), "same state ⇒ byte-identical snapshot");
        let cids: Vec<&str> = a.iter().filter(|l| l.contains("\"state\""))
            .map(|l| if l.contains("C_A") { "A" } else if l.contains("C_B") { "B" } else { "C" }).collect();
        assert_eq!(cids, vec!["A", "B", "C"], "sorted by key");
    }
}

#[cfg(test)]
mod tests_mig104_hooks {
    //! MIG-104 Slice 4 — the record FORMAT and the two write ORDERS. The commands themselves
    //! need an AppHandle, so these pin the parts that carry the design: the line shapes, the
    //! type-free key of Q2, the derivable-tier suppression, and the decision order's contract
    //! that a failed append must stop the DB change.
    use super::*;

    #[test]
    fn walk_line_carries_an_absolute_count_and_fixed_field_order() {
        let l = walk_line("C_SRC", "C_TGT", "the four books", 3, "2026-07-27T09:11:05Z");
        assert_eq!(
            l,
            r#"{"v":1,"t":"walk","cid":"C_SRC","to":"C_TGT","tn":"the four books","n":3,"at":"2026-07-27T09:11:05Z"}"#,
            "field order is part of the contract — a human reads this file in a text editor"
        );
    }

    /// Q2, Boss-ruled: the key is TYPE-FREE, so re-typing a link keeps its earned history.
    #[test]
    fn type_variants_of_one_pair_produce_one_ledger_key() {
        let d = tempfile::tempdir().unwrap();
        // The real live shape: one note linking to `the four books` twice, as `supports` and as
        // `derives-from`, one click each. The DB writers match on source + target name and ignore
        // link_type, so both are ONE link in the user's terms.
        append(d.path(), Stream::Earned, &[
            walk_line("C_ISLAM", "C_BOOKS", "the four books", 1, "2026-07-27T09:00:00Z"),
            walk_line("C_ISLAM", "C_BOOKS", "the four books", 2, "2026-07-27T09:05:00Z"),
        ]).unwrap();
        let (map, _) = read_folded(d.path());
        assert_eq!(map.len(), 1, "the two typed variants must fold to ONE record");
        assert_eq!(map.get("C_ISLAM>C_BOOKS").unwrap().n, 2);
    }

    #[test]
    fn an_unresolved_target_still_keys_and_survives_the_fold() {
        let d = tempfile::tempdir().unwrap();
        append(d.path(), Stream::Earned, &[walk_line("C_SRC", "", "banana", 4, "2026-07-27T09:00:00Z")]).unwrap();
        let (map, _) = read_folded(d.path());
        assert_eq!(map.get("C_SRC>~banana").unwrap().n, 4, "a broken link's earned history is still recorded");
    }

    /// The auto-tier must never be recorded: it carries no user judgment and is derivable from
    /// the count, so recording it would fill the ledger with decisions nobody made.
    #[test]
    fn auto_tier_promotion_writes_no_trust_event() {
        assert!(is_derivable_tier("hypothesis", 1));
        assert!(is_derivable_tier("evidence", 3));
        assert!(is_derivable_tier("evidence", 9));
        assert!(is_derivable_tier("established", 10));
        // A USER judgment is never derivable — `contested` has no count that produces it…
        assert!(!is_derivable_tier("contested", 0));
        assert!(!is_derivable_tier("contested", 50));
        // …and neither is a manual pick that outranks the count.
        assert!(!is_derivable_tier("established", 3));
        assert!(!is_derivable_tier("evidence", 1));
    }

    #[test]
    fn retire_then_restore_reconstructs_in_order_from_the_ledger_alone() {
        let d = tempfile::tempdir().unwrap();
        append(d.path(), Stream::Earned, &[
            walk_line("C_A", "C_B", "b", 7, "2026-07-27T09:00:00Z"),
            retire_line("C_A", "C_B", "b", "2026-07-27T09:01:00Z"),
        ]).unwrap();
        assert_eq!(read_folded(d.path()).0.get("C_A>C_B").unwrap().status.as_deref(), Some("archived"));
        append(d.path(), Stream::Earned, &[restore_line("C_A", "C_B", "b", "2026-07-27T09:02:00Z")]).unwrap();
        let e = read_folded(d.path()).0.get("C_A>C_B").cloned().unwrap();
        assert_eq!(e.status.as_deref(), Some("active"), "the LAST decision wins");
        assert_eq!(e.n, 7, "and the earned count is untouched by either decision");
    }

    /// The decision order's whole point: if the record cannot be made durable the DB must not
    /// change. `append` returning Err is what the command propagates instead of proceeding.
    #[test]
    fn a_failed_append_is_an_error_the_caller_must_not_swallow() {
        // A path that cannot be a directory → open() fails → Err, not a silent Ok.
        let d = tempfile::tempdir().unwrap();
        let not_a_dir = d.path().join("file-not-dir");
        std::fs::write(&not_a_dir, b"x").unwrap();
        let r = append(&not_a_dir, Stream::Earned, &[retire_line("C_A", "C_B", "b", "2026-07-27T09:00:00Z")]);
        assert!(r.is_err(), "the decision path relies on this Err to abort the DB change");
    }

    #[test]
    fn priority_line_has_no_target() {
        let l = priority_line("C_A", 2, "2026-07-27T09:00:00Z");
        assert_eq!(l, r#"{"v":1,"t":"priority","cid":"C_A","p":2,"at":"2026-07-27T09:00:00Z"}"#);
        assert!(!l.contains("\"to\""), "a review priority is about a NOTE, not a link");
    }

    #[test]
    fn the_toggle_off_means_no_file_is_ever_created() {
        // Documents the contract; the const is compiled in, so this asserts the shape callers use.
        let d = tempfile::tempdir().unwrap();
        if !EARNED_LEDGER_WRITE {
            assert!(!d.path().join(Stream::Earned.file_name()).exists());
        }
        // With writes ON (the shipped default) an append creates the file on first use.
        append(d.path(), Stream::Earned, &[walk_line("C", "D", "d", 1, "t")]).unwrap();
        assert!(d.path().join("earned.jsonl").exists());
    }
}
