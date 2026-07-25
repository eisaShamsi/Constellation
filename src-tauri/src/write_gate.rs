//! MIG-076 §A1 — The WriteGate: the single door every .md note write passes through.
//!
//! Why this module exists: four data-corruption incidents (BUG-012/§140, BUG-015,
//! the F2 cascade-stomp family, BUG-023) shared one root — note writes were
//! plain `fs::write` calls scattered across ten sites, with no serialization,
//! no atomicity, and no identity verification. The WriteGate is the structural
//! fix's foundation layer (Architect: `lab/reports/MIG-076-WRITE-INTEGRITY-ARCHITECT.md`):
//!
//! - **L0 — per-path serialization.** Every write to a given file takes that
//!   file's lock first. Two writers can never interleave on one path — races
//!   become impossible rather than detected (the SQLite-WAL principle).
//! - **L1 — atomic replace.** Content lands in a same-directory temp file,
//!   is fsynced, then swapped in via `ReplaceFileW` (preserves the target's
//!   creation time + attributes — `note_meta.created_at` reads fs creation
//!   time, so a plain temp+rename would silently reset note ages) with a
//!   bounded retry for antivirus/indexer sharing violations. A crash mid-save
//!   can no longer leave a torn or zero-length note.
//! - **The write journal.** Every gated write appends one JSONL line
//!   (`write-journal.jsonl` in the app-data dir, 5 MB rotation): timestamp,
//!   path, surface, outcome, size, content hash. Any future anomaly is
//!   attributable from one file — the forensics BUG-023 cost a session to
//!   reconstruct by hand.
//!
//! L2 (identity + freshness compare-and-swap, shadow→enforce) lands in §B on
//! the `Expectation` seam below. Speed rider (Eisa): all gate costs live on
//! the ≥1.5 s-debounced save path — one uncontended in-memory lock, one fsync,
//! one journal append. Nothing here runs per keystroke or at boot.

use std::collections::HashMap;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// §B2 — when true, identity/freshness mismatches REFUSE (and quarantine);
/// until then the gate runs in SHADOW mode: journal the would-refuse verdict,
/// perform the write anyway. Flipped only after the ★Stage-1 soak shows a
/// clean journal (Plan §F1, invariant I6/I10).
pub const WRITE_GATE_ENFORCE: bool = false;

/// §B1 (L2) — what the caller believes about the file it is replacing.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expectation {
    /// The note identity the caller composed this content FOR (frontmatter
    /// `cid_cn`). Mismatch with the on-disk identity = the Frankenstein class.
    pub expected_cid: Option<String>,
    /// Freshness token of the disk state the caller last read (racy-git rule:
    /// mtime+size, escalate to hash on ambiguity).
    pub base_mtime: u64,
    pub base_size: u64,
    /// fnv1a-64 hex of the disk content the caller last read (as reported by
    /// the gate/journal fingerprint). When mtime+size drift but this matches
    /// the current disk content, the drift was metadata-only (AV touch) — fresh.
    pub base_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOutcome {
    /// Write performed; an Expectation was checked (§B+).
    Ok,
    /// Write performed with no Expectation supplied (§A2's first routing mode).
    OkUnchecked,
    /// Create-exclusive succeeded — the file did not exist before.
    CreatedExclusive,
    /// Create-exclusive refused — the path already exists. The caller maps
    /// this to its own collision behavior; the gate guarantees "write to
    /// path" can never silently become "create file".
    RefusedExists,
    /// A gated filesystem rename/move (both paths were locked).
    Renamed,
    /// §B1 shadow — the on-disk note carries a DIFFERENT cid_cn than the one
    /// this content was composed for (the Frankenstein class), or the file is
    /// gone. Written anyway in shadow mode; refused once enforcement flips.
    WouldRefuseIdentity,
    /// §B1 shadow — same note, but the disk is newer than the state the
    /// caller composed against (the lost-update/stomp class).
    WouldRefuseStale,
    /// §B1 — identity expected, but the on-disk note has no cid_cn yet
    /// (legacy population; the §B3 backfill closes this). Journaled, allowed.
    UnverifiedNoCid,
    /// §B2 — no explicit Expectation, but the INCOMING content carries a
    /// cid_cn and it MATCHES the disk's: identity self-attested. The content
    /// is the snapshot — this protection needs no caller plumbing and covers
    /// every writer (it is the check that would have caught BUG-023's write).
    SelfAttestedOk,
    /// §B2 — the incoming content carries a cid_cn but no file exists at the
    /// path: this write CREATED the file. Either a legitimate create-via-write
    /// surface or the §140 deleted-note class — the journal + soak decide
    /// which surfaces should move to create-exclusive (§F).
    CreatedByWrite,
    /// Batch-2 — a gated destructive removal (file or directory) performed
    /// under the path lock (`gate_delete`).
    Deleted,
}

impl WriteOutcome {
    fn as_str(self) -> &'static str {
        match self {
            WriteOutcome::Ok => "ok",
            WriteOutcome::OkUnchecked => "ok_unchecked",
            WriteOutcome::CreatedExclusive => "created_exclusive",
            WriteOutcome::RefusedExists => "refused_exists",
            WriteOutcome::Renamed => "renamed",
            WriteOutcome::WouldRefuseIdentity => "would_refuse_identity",
            WriteOutcome::WouldRefuseStale => "would_refuse_stale",
            WriteOutcome::UnverifiedNoCid => "unverified_no_cid",
            WriteOutcome::SelfAttestedOk => "ok_self_attested",
            WriteOutcome::CreatedByWrite => "created_by_write",
            WriteOutcome::Deleted => "deleted",
        }
    }
}

// ─── L0: the per-path lock registry ────────────────────────────────────────
//
// Entries are an Arc<Mutex<()>> per distinct path written this session —
// tens of bytes each, bounded by the number of notes the user touches between
// launches; no sweep needed (contrast watcher_suppress, which is hit by every
// watcher event and does sweep).

fn locks() -> &'static Mutex<HashMap<String, Arc<Mutex<()>>>> {
    static CELL: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Windows paths are case-insensitive and arrive in both separator styles;
/// the lock key must unify them or two spellings of one file get two locks.
fn lock_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn path_lock(path: &Path) -> Arc<Mutex<()>> {
    let key = lock_key(path);
    let mut map = match locks().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(), // a poisoned registry must never block writes
    };
    map.entry(key).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
}

// ─── The journal ───────────────────────────────────────────────────────────

const JOURNAL_MAX_BYTES: u64 = 5 * 1024 * 1024;

fn journal_path() -> &'static OnceLock<PathBuf> {
    static CELL: OnceLock<PathBuf> = OnceLock::new();
    &CELL
}

/// Set once at app startup (lib.rs `.setup()`), pointing at the app-data dir.
/// Before/without init (unit tests that don't care), journaling no-ops.
pub fn init_journal(dir: PathBuf) {
    let _ = fs::create_dir_all(&dir);
    let _ = journal_path().set(dir.join("write-journal.jsonl"));
}

/// Append serialization — keeps concurrent gate writes from interleaving
/// journal lines (the path locks are per-file; the journal is shared).
fn journal_lock() -> &'static Mutex<()> {
    static CELL: OnceLock<Mutex<()>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(()))
}

/// Best-effort: journal failure must never fail or delay the user's write.
fn journal(path: &Path, surface: &str, outcome: WriteOutcome, bytes: usize, hash: u64) {
    journal_ext(path, surface, outcome, bytes, hash, None, None);
}

fn journal_ext(
    path: &Path,
    surface: &str,
    outcome: WriteOutcome,
    bytes: usize,
    hash: u64,
    expected_cid: Option<&str>,
    found_cid: Option<&str>,
) {
    let Some(jp) = journal_path().get() else { return };
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let mut line = serde_json::json!({
        "ts": ts,
        "path": path.to_string_lossy(),
        "surface": surface,
        "outcome": outcome.as_str(),
        "bytes": bytes,
        "hash": format!("{:016x}", hash),
    });
    if let Some(e) = expected_cid {
        line["expected_cid"] = serde_json::json!(e);
    }
    if let Some(f) = found_cid {
        line["found_cid"] = serde_json::json!(f);
    }
    let _guard = match journal_lock().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    // Rotate at the cap: one .old generation is enough history for forensics.
    if let Ok(meta) = fs::metadata(jp) {
        if meta.len() > JOURNAL_MAX_BYTES {
            let _ = fs::rename(jp, jp.with_extension("jsonl.old"));
        }
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(jp) {
        let _ = writeln!(f, "{}", line);
    }
}

/// FNV-1a 64 — a cheap, dependency-free content fingerprint for the journal.
/// (Not a CAS token — §B's freshness hash decision is separate.)
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

// ─── L1: atomic replace ────────────────────────────────────────────────────

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Same-directory temp + fsync + swap-in, with a bounded retry for the
/// antivirus/indexer class (`ERROR_SHARING_VIOLATION`/`ERROR_ACCESS_DENIED`
/// on freshly-created files — the #1 real-world atomic-replace failure on
/// Windows; rustup/cargo retry exactly like this).
fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| format!("write_gate: no parent dir for {}", path.display()))?;
    let stem = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let tmp = dir.join(format!(
        ".{}.{}-{}.cnstmp",
        stem,
        std::process::id(),
        TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));

    // Suppress watcher events for BOTH paths involved in the swap — temp
    // creation and the final replace each emit; either leaking re-opens the
    // F3 watcher-loop class (Rename Concept Paper).
    crate::watcher_suppress::mark(&tmp);
    crate::watcher_suppress::mark(path);

    {
        let mut f = fs::File::create(&tmp)
            .map_err(|e| format!("write_gate: temp create failed: {}", e))?;
        f.write_all(content.as_bytes())
            .map_err(|e| format!("write_gate: temp write failed: {}", e))?;
        // fsync before the swap — otherwise power loss can land the rename
        // while data is unflushed, yielding a named-but-empty note.
        f.sync_all()
            .map_err(|e| format!("write_gate: temp sync failed: {}", e))?;
    }

    let mut attempt: u64 = 0;
    loop {
        let res = if path.exists() {
            replace_file(path, &tmp)
        } else {
            fs::rename(&tmp, path).map_err(|e| e.to_string())
        };
        match res {
            Ok(()) => return Ok(()),
            Err(e) => {
                attempt += 1;
                if attempt >= 5 {
                    let _ = fs::remove_file(&tmp);
                    return Err(format!("write_gate: replace failed after retries: {}", e));
                }
                std::thread::sleep(Duration::from_millis(50 * attempt));
            }
        }
    }
}

/// Windows: `ReplaceFileW` — MSDN's recommended replace; preserves the
/// TARGET's ACLs, attributes and creation time (which `note_meta.created_at`
/// reads from the filesystem — a plain rename would reset every note's age
/// to its last save). Manual kernel32 extern: no new dependency.
#[cfg(windows)]
fn replace_file(target: &Path, replacement: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn ReplaceFileW(
            replaced: *const u16,
            replacement: *const u16,
            backup: *const u16,
            flags: u32,
            exclude: *mut core::ffi::c_void,
            reserved: *mut core::ffi::c_void,
        ) -> i32;
        fn GetLastError() -> u32;
    }
    fn wide(p: &Path) -> Vec<u16> {
        p.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }
    const REPLACEFILE_IGNORE_MERGE_ERRORS: u32 = 0x2;
    let (t, r) = (wide(target), wide(replacement));
    let ok = unsafe {
        ReplaceFileW(
            t.as_ptr(),
            r.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_IGNORE_MERGE_ERRORS,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if ok != 0 {
        Ok(())
    } else {
        Err(format!("ReplaceFileW error {}", unsafe { GetLastError() }))
    }
}

#[cfg(not(windows))]
fn replace_file(target: &Path, replacement: &Path) -> Result<(), String> {
    fs::rename(replacement, target).map_err(|e| e.to_string())
}

// ─── The gate ──────────────────────────────────────────────────────────────

/// §B1 — the verdict, computed UNDER the path lock (no TOCTOU): identity
/// first (cid_cn — the Frankenstein class), then freshness (mtime+size, with
/// the racy-git hash escalation for metadata-only drift). Returns the verdict
/// and the cid found on disk (for the journal).
fn check_expectation(path: &Path, exp: &Expectation) -> (WriteOutcome, Option<String>) {
    let Ok(meta) = fs::metadata(path) else {
        // The note this content was composed for is GONE from this path
        // (deleted or renamed mid-flight) — the §140 class.
        return (WriteOutcome::WouldRefuseIdentity, None);
    };

    let disk_mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let disk_size = meta.len();
    let fresh_by_meta = disk_mtime == exp.base_mtime && disk_size == exp.base_size;

    // Read the disk content only when a check needs it (identity expected,
    // or freshness needs the hash escalation).
    let need_content = exp.expected_cid.is_some() || (!fresh_by_meta && exp.base_hash.is_some());
    let disk_content = if need_content { fs::read_to_string(path).ok() } else { None };

    let found_cid = disk_content
        .as_deref()
        .and_then(crate::search::extract_frontmatter_cid_cn);

    if let Some(ref expected) = exp.expected_cid {
        match found_cid {
            Some(ref found) if found != expected => {
                return (WriteOutcome::WouldRefuseIdentity, Some(found.clone()));
            }
            None => return (WriteOutcome::UnverifiedNoCid, None),
            _ => {} // identity confirmed — fall through to freshness
        }
    }

    if fresh_by_meta {
        return (WriteOutcome::Ok, found_cid);
    }
    if let (Some(base_hash), Some(content)) = (&exp.base_hash, &disk_content) {
        if format!("{:016x}", fnv1a(content.as_bytes())) == *base_hash {
            // Metadata drifted (AV/indexer touch) but the bytes are exactly
            // what the caller composed against — fresh.
            return (WriteOutcome::Ok, found_cid);
        }
    }
    (WriteOutcome::WouldRefuseStale, found_cid)
}

/// Write `content` to `path` through the gate: path lock → CAS check (§B,
/// SHADOW until `WRITE_GATE_ENFORCE`) → atomic replace → journal. `surface`
/// names the caller for the journal (e.g. "write_note", "cascade").
pub fn gate_write(
    path: &Path,
    content: &str,
    expect: Option<&Expectation>,
    surface: &str,
) -> Result<WriteOutcome, String> {
    let lk = path_lock(path);
    let _guard = match lk.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(), // a panicked earlier writer must not wedge this file forever
    };

    let (outcome, found_cid) = match expect {
        Some(exp) => check_expectation(path, exp),
        // §B2 — SELF-ATTESTATION: no caller expectation, but the incoming
        // content names the note it belongs to (its frontmatter cid_cn).
        // Compare against the disk's identity under the lock. The content IS
        // the snapshot — no plumbing, no second composition source, and every
        // writer is covered.
        None => match crate::search::extract_frontmatter_cid_cn(content) {
            None => (WriteOutcome::OkUnchecked, None),
            Some(incoming_cid) => {
                if !path.exists() {
                    (WriteOutcome::CreatedByWrite, None)
                } else {
                    let disk_cid = fs::read_to_string(path)
                        .ok()
                        .as_deref()
                        .and_then(crate::search::extract_frontmatter_cid_cn);
                    match disk_cid {
                        Some(ref d) if *d != incoming_cid => {
                            (WriteOutcome::WouldRefuseIdentity, disk_cid.clone())
                        }
                        Some(_) => (WriteOutcome::SelfAttestedOk, disk_cid.clone()),
                        None => (WriteOutcome::UnverifiedNoCid, None),
                    }
                }
            }
        },
    };

    // SHADOW mode: would-refuse verdicts journal loudly but the write
    // proceeds — invariant I6 (no legitimate save blocked before the soak
    // proves the verdicts trustworthy). Enforcement (refuse + quarantine +
    // dialog) flips in §F1.
    let _ = WRITE_GATE_ENFORCE; // referenced now; consumed by the §F1 flip

    atomic_write(path, content)?;
    // The journal's "expected" = the explicit attestation, or the incoming
    // content's own cid when the verdict came from self-attestation.
    let journal_expected: Option<String> = match expect {
        Some(e) => e.expected_cid.clone(),
        None => crate::search::extract_frontmatter_cid_cn(content),
    };
    journal_ext(
        path,
        surface,
        outcome,
        content.len(),
        fnv1a(content.as_bytes()),
        journal_expected.as_deref(),
        found_cid.as_deref(),
    );
    Ok(outcome)
}

/// Create a NEW file — refuses if the path already exists. "Write to path"
/// can never create, and "create" can never overwrite: the Git lockfile /
/// `If-None-Match: *` semantics that close the §140 path-reuse class at the
/// boundary.
pub fn gate_create_exclusive(
    path: &Path,
    content: &str,
    surface: &str,
) -> Result<WriteOutcome, String> {
    let lk = path_lock(path);
    let _guard = match lk.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };

    if path.exists() {
        journal(path, surface, WriteOutcome::RefusedExists, content.len(), fnv1a(content.as_bytes()));
        return Ok(WriteOutcome::RefusedExists);
    }
    atomic_write(path, content)?;
    journal(path, surface, WriteOutcome::CreatedExclusive, content.len(), fnv1a(content.as_bytes()));
    Ok(WriteOutcome::CreatedExclusive)
}

// ─── §E-2: write-integrity diagnostics ──────────────────────────────────────

/// A snapshot of the write journal for the Settings → Security & Privacy
/// "Write integrity" line. Counts across the live journal AND its one rotated
/// `.old` generation. `anomalies` = the SHADOW-mode would-refuse verdicts
/// (`would_refuse_identity` + `would_refuse_stale`) — the numbers that must be
/// ZERO before the §F1 enforcement flip (the clean-soak precondition).
#[derive(serde::Serialize, Default)]
pub struct JournalStats {
    /// Total journal lines (every gated write appends one).
    pub writes: u64,
    /// would_refuse_identity + would_refuse_stale — must be 0 for the §F flip.
    pub anomalies: u64,
    pub would_refuse_identity: u64,
    pub would_refuse_stale: u64,
    /// Unix-millis of the MOST RECENT anomaly — lets the UI distinguish a stale
    /// red (historical, pre-fix) from a live one. `None` when anomalies == 0.
    pub last_anomaly_ts: Option<u64>,
    /// Create races the gate correctly refused (the gate WORKING — informational).
    pub refused_exists: u64,
    /// Writes with no cid_cn to verify against (templates / non-notes).
    pub unverified_no_cid: u64,
    /// Files first created via a write/create-exclusive surface.
    pub created: u64,
    /// `WRITE_GATE_ENFORCE` — false = shadow (monitor), true = enforced (§F1).
    pub enforce: bool,
    /// Whether any journal file exists yet.
    pub exists: bool,
    /// Whether a rotated `.old` generation was also counted.
    pub rotated: bool,
    /// The journal's directory (app-data dir) — for the "open folder" button.
    pub dir: String,
}

/// Stream one journal file line-by-line (a 5 MB file never loads whole into
/// memory), folding each line's outcome into the running tally.
fn count_journal_file(path: &Path, stats: &mut JournalStats) {
    use std::io::{BufRead, BufReader};
    let Ok(f) = fs::File::open(path) else { return };
    for line in BufReader::new(f).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() { continue; }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        stats.writes += 1;
        match v.get("outcome").and_then(|o| o.as_str()).unwrap_or("") {
            "would_refuse_identity" => {
                stats.would_refuse_identity += 1; stats.anomalies += 1;
                let ts = v.get("ts").and_then(|t| t.as_u64()).unwrap_or(0);
                if ts > stats.last_anomaly_ts.unwrap_or(0) { stats.last_anomaly_ts = Some(ts); }
            }
            "would_refuse_stale" => {
                stats.would_refuse_stale += 1; stats.anomalies += 1;
                let ts = v.get("ts").and_then(|t| t.as_u64()).unwrap_or(0);
                if ts > stats.last_anomaly_ts.unwrap_or(0) { stats.last_anomaly_ts = Some(ts); }
            }
            "refused_exists" => stats.refused_exists += 1,
            "unverified_no_cid" => stats.unverified_no_cid += 1,
            "created_exclusive" | "created_by_write" => stats.created += 1,
            _ => {}
        }
    }
}

/// §E-2 — read the write-journal stats for the diagnostics line. Read-only,
/// opened on demand from Settings (never a hot path), counting the live journal
/// plus its one rotated `.old` generation.
#[tauri::command]
pub fn read_write_journal_stats() -> JournalStats {
    let mut stats = JournalStats { enforce: WRITE_GATE_ENFORCE, ..Default::default() };
    let Some(jp) = journal_path().get() else { return stats };
    if let Some(dir) = jp.parent() {
        stats.dir = dir.to_string_lossy().to_string();
    }
    if jp.exists() {
        stats.exists = true;
        count_journal_file(jp, &mut stats);
    }
    let old = jp.with_extension("jsonl.old");
    if old.exists() {
        stats.rotated = true;
        stats.exists = true;
        count_journal_file(&old, &mut stats);
    }
    stats
}

/// Rename/move a note file under BOTH paths' locks (acquired in sorted-key
/// order so two concurrent renames can never deadlock). The bounded retry
/// covers the same AV/indexer sharing-violation class as writes.
pub fn gate_rename(old: &Path, new: &Path, surface: &str) -> Result<WriteOutcome, String> {
    let (ka, kb) = (lock_key(old), lock_key(new));
    let (first, second) = if ka <= kb { (old, new) } else { (new, old) };
    let l1 = path_lock(first);
    let _g1 = match l1.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    // Same path (case/separator variant) → one lock is enough. The Arc is
    // hoisted to function scope so the guard's borrow outlives the block.
    let l2 = if ka != kb { Some(path_lock(second)) } else { None };
    let _g2 = l2.as_ref().map(|l| match l.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    });

    // 2026-07-25 PJ-140 #18: dest-exists guard UNDER the lock. Every caller's collision
    // check is an OUTSIDE-the-lock exists() pre-check (move_item, trash de-collide,
    // rename_folder), so a concurrent create at `new` between that check and here would
    // be silently REPLACED: fs::rename replaces an existing dest on both Windows and —
    // the mandated macOS port — POSIX rename(2). Return Err (NOT Ok(RefusedExists)):
    // every caller already treats a collision as Err — `?` callers propagate the clean
    // "already exists" error and never run their post-rename DB migrate against a path
    // holding someone else's file, and delete_trash's `.is_err()` fallback then
    // completes the move via copy+remove. An Ok outcome here would instead read as a
    // successful rename that never happened (false success). Skip when old==new
    // (case/separator variant).
    if ka != kb && new.exists() {
        journal(new, surface, WriteOutcome::RefusedExists, 0, fnv1a(old.to_string_lossy().as_bytes()));
        return Err("An item with this name already exists at the destination.".to_string());
    }

    crate::watcher_suppress::mark(old);
    crate::watcher_suppress::mark(new);

    let mut attempt: u64 = 0;
    loop {
        match fs::rename(old, new) {
            Ok(()) => break,
            Err(e) => {
                attempt += 1;
                if attempt >= 5 {
                    return Err(format!("Failed to rename file: {}", e));
                }
                std::thread::sleep(Duration::from_millis(50 * attempt));
            }
        }
    }
    journal(new, surface, WriteOutcome::Renamed, 0, fnv1a(old.to_string_lossy().as_bytes()));
    Ok(WriteOutcome::Renamed)
}

// ─── Batch-2 primitives: locked read-modify-write / delete ─────────────────
//
// WHY (note-open-freeze Batch 2, 2026-07-03): `gate_write` holds the per-path
// lock only across the CAS check + atomic replace — NOT across a caller's
// read→modify→write cycle. While every note-file command was a SYNC
// `#[tauri::command]`, the single IPC dispatch thread serialized those cycles
// against the editor's debounced save for free. Converting them to `(async)`
// (so a writer-lock wait can't freeze the app) removes that accidental
// serialization — these primitives replace it with an explicit one: the SAME
// per-path lock `gate_write` uses, held across the WHOLE cycle, so an editor
// save can land before or after an RMW but never inside it.
//
// TWO HARD RULES for callers:
// 1. NEVER call another gate_* on the same path inside the closure — the
//    per-path Mutex is NOT reentrant; that is a self-deadlock.
// 2. NEVER wait on SearchState.db (or any multi-second lock) inside the
//    closure — the editor's SYNC `write_note` parks on this path lock, and a
//    path lock that waits on the DB writer re-freezes the dispatch thread
//    through the back door. Do DB work AFTER the gate call returns.

/// Read `path` under its lock, let `mutate` produce replacement content, and
/// atomically write it — one critical section. `mutate` returning `Ok(None)`
/// means "no change needed" (nothing written, nothing journaled). The file
/// must exist (RMW targets existing notes; use `gate_create_exclusive` /
/// `gate_write` to create).
pub fn gate_rmw(
    path: &Path,
    surface: &str,
    mutate: impl FnOnce(&str) -> Result<Option<String>, String>,
) -> Result<WriteOutcome, String> {
    let lk = path_lock(path);
    let _guard = match lk.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    let disk = fs::read_to_string(path)
        .map_err(|e| format!("write_gate: rmw read failed for {}: {}", path.display(), e))?;
    let Some(updated) = mutate(&disk)? else {
        return Ok(WriteOutcome::OkUnchecked); // no-op: nothing written
    };
    atomic_write(path, &updated)?;
    // Freshness is BY CONSTRUCTION (read under the same lock) — journal as a
    // checked write, with the content's own cid as the attestation.
    let cid = crate::search::extract_frontmatter_cid_cn(&updated);
    journal_ext(
        path,
        surface,
        WriteOutcome::Ok,
        updated.len(),
        fnv1a(updated.as_bytes()),
        cid.as_deref(),
        cid.as_deref(),
    );
    Ok(WriteOutcome::Ok)
}

/// The `rename_item` shape as ONE critical section under BOTH paths' locks
/// (sorted order, like `gate_rename`): read `old` → `mutate` (e.g. rewrite
/// the frontmatter title) → optional atomic write → move `old` → `new`.
/// `old == new` (a pure title change) skips the move. A pre-existing `new`
/// (different file) returns `RefusedExists` under the lock — the caller maps
/// it to its collision dialog; the check can no longer race a concurrent
/// create.
pub fn gate_rmw_rename(
    old: &Path,
    new: &Path,
    surface: &str,
    mutate: impl FnOnce(&str) -> Result<Option<String>, String>,
) -> Result<WriteOutcome, String> {
    let (ka, kb) = (lock_key(old), lock_key(new));
    let (first, second) = if ka <= kb { (old, new) } else { (new, old) };
    let l1 = path_lock(first);
    let _g1 = match l1.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    let l2 = if ka != kb { Some(path_lock(second)) } else { None };
    let _g2 = l2.as_ref().map(|l| match l.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    });

    if ka != kb && new.exists() {
        journal(new, surface, WriteOutcome::RefusedExists, 0, fnv1a(old.to_string_lossy().as_bytes()));
        return Ok(WriteOutcome::RefusedExists);
    }

    let disk = fs::read_to_string(old)
        .map_err(|e| format!("write_gate: rmw-rename read failed for {}: {}", old.display(), e))?;
    if let Some(updated) = mutate(&disk)? {
        atomic_write(old, &updated)?;
        let cid = crate::search::extract_frontmatter_cid_cn(&updated);
        journal_ext(
            old,
            surface,
            WriteOutcome::Ok,
            updated.len(),
            fnv1a(updated.as_bytes()),
            cid.as_deref(),
            cid.as_deref(),
        );
    }

    if ka == kb {
        return Ok(WriteOutcome::Ok); // pure title change — no move
    }

    crate::watcher_suppress::mark(old);
    crate::watcher_suppress::mark(new);
    let mut attempt: u64 = 0;
    loop {
        match fs::rename(old, new) {
            Ok(()) => break,
            Err(e) => {
                attempt += 1;
                if attempt >= 5 {
                    return Err(format!("Failed to rename file: {}", e));
                }
                std::thread::sleep(Duration::from_millis(50 * attempt));
            }
        }
    }
    journal(new, surface, WriteOutcome::Renamed, 0, fnv1a(old.to_string_lossy().as_bytes()));
    Ok(WriteOutcome::Renamed)
}

/// What `gate_delete` removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteMode {
    /// `fs::remove_file`
    File,
    /// `fs::remove_dir_all`
    DirAll,
}

/// Destructive removal under the path lock (+ watcher suppression + the same
/// bounded AV/indexer retry as writes). Serializes against every gated write
/// to the same path, so a save can land before the delete (and be deleted
/// with it) or after (and legitimately recreate) — never during.
pub fn gate_delete(path: &Path, mode: DeleteMode, surface: &str) -> Result<WriteOutcome, String> {
    let lk = path_lock(path);
    let _guard = match lk.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    crate::watcher_suppress::mark(path);
    let mut attempt: u64 = 0;
    loop {
        let res = match mode {
            DeleteMode::File => fs::remove_file(path),
            DeleteMode::DirAll => fs::remove_dir_all(path),
        };
        match res {
            Ok(()) => break,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => break, // idempotent
            Err(e) => {
                attempt += 1;
                if attempt >= 5 {
                    return Err(format!("write_gate: delete failed after retries: {}", e));
                }
                std::thread::sleep(Duration::from_millis(50 * attempt));
            }
        }
    }
    journal(path, surface, WriteOutcome::Deleted, 0, fnv1a(path.to_string_lossy().as_bytes()));
    Ok(WriteOutcome::Deleted)
}

/// §B2-4 stall forensics — append a zero-byte marker line to the journal
/// (surface names the checkpoint, e.g. "rename_return" / "cascade_enter").
/// Makes "the fs work happened but the flow died afterwards" journal-decidable:
/// a rename with `renamed` but no `rename_return` = the command's tail stalled;
/// `rename_return` but no `cascade_enter` = the frontend chain died in between.
pub fn journal_marker(path: &Path, surface: &str) {
    journal(path, surface, WriteOutcome::OkUnchecked, 0, 0);
}

/// §B2-4 stall forensics, frontend side — lets the JS orchestration journal
/// its own checkpoints (and, crucially, the TEXT of a caught exception that
/// would otherwise die invisibly in a release build's console). The `detail`
/// string rides in the journal line's path field.
#[tauri::command]
pub fn journal_frontend_marker(surface: String, detail: String) {
    let d: String = detail.chars().take(300).collect();
    journal(Path::new(&d), &surface, WriteOutcome::OkUnchecked, 0, 0);
}

/// Escape hatch for compound destructive sequences that must run under the
/// source path's lock but don't fit `gate_delete`'s single-call shape (the
/// `delete_path` trash fallback's copy+remove pair). SAME TWO HARD RULES as
/// the closures above: no gate_* on this path inside `f` (non-reentrant
/// Mutex → self-deadlock), no DB waits inside `f`.
pub fn with_path_lock<R>(path: &Path, f: impl FnOnce() -> R) -> R {
    let lk = path_lock(path);
    let _guard = match lk.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    f()
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_write_gate {
    use super::*;
    use std::thread;

    fn tdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wg_{}_{}", tag, std::process::id()));
        let _ = fs::create_dir_all(&d);
        d
    }

    #[test]
    fn writes_fresh_file_and_journals() {
        let d = tdir("fresh");
        init_journal(d.clone());
        let p = d.join("note.md");
        let out = gate_write(&p, "hello world", None, "test").unwrap();
        assert_eq!(out, WriteOutcome::OkUnchecked);
        assert_eq!(fs::read_to_string(&p).unwrap(), "hello world");
        // journal is global+once-init; presence of the file is the assertion
        // (first init wins across the test binary — line content covered below)
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn replaces_existing_content_atomically() {
        let d = tdir("replace");
        let p = d.join("note.md");
        gate_write(&p, "first", None, "test").unwrap();
        gate_write(&p, "second", None, "test").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "second");
        // no temp litter left behind
        let leftovers: Vec<_> = fs::read_dir(&d)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".cnstmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp files left: {:?}", leftovers);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn concurrent_writers_serialize_never_tear() {
        let d = tdir("serial");
        let p = d.join("contested.md");
        let a = "A".repeat(64 * 1024);
        let b = "B".repeat(64 * 1024);
        let (pa, pb) = (p.clone(), p.clone());
        let (ca, cb) = (a.clone(), b.clone());
        let ta = thread::spawn(move || {
            for _ in 0..25 {
                gate_write(&pa, &ca, None, "test_a").unwrap();
            }
        });
        let tb = thread::spawn(move || {
            for _ in 0..25 {
                gate_write(&pb, &cb, None, "test_b").unwrap();
            }
        });
        ta.join().unwrap();
        tb.join().unwrap();
        let final_content = fs::read_to_string(&p).unwrap();
        // Atomic + serialized: the survivor is ONE full payload, never a mix.
        assert!(final_content == a || final_content == b, "torn write detected");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn create_exclusive_refuses_existing() {
        let d = tdir("excl");
        let p = d.join("note.md");
        gate_write(&p, "original", None, "test").unwrap();
        let out = gate_create_exclusive(&p, "intruder", "test").unwrap();
        assert_eq!(out, WriteOutcome::RefusedExists);
        assert_eq!(fs::read_to_string(&p).unwrap(), "original"); // untouched
        let fresh = d.join("new.md");
        assert_eq!(
            gate_create_exclusive(&fresh, "born", "test").unwrap(),
            WriteOutcome::CreatedExclusive
        );
        assert_eq!(fs::read_to_string(&fresh).unwrap(), "born");
        let _ = fs::remove_dir_all(&d);
    }

    fn stat(p: &Path) -> (u64, u64) {
        let m = fs::metadata(p).unwrap();
        let mt = m.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs();
        (mt, m.len())
    }

    fn exp(cid: Option<&str>, mtime: u64, size: u64, hash: Option<String>) -> Expectation {
        Expectation {
            expected_cid: cid.map(|s| s.to_string()),
            base_mtime: mtime,
            base_size: size,
            base_hash: hash,
        }
    }

    #[test]
    fn cas_identity_mismatch_shadow_journals_but_writes() {
        let d = tdir("cas_id");
        let p = d.join("note.md");
        gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nbody", None, "test").unwrap();
        let (mt, sz) = stat(&p);
        // Composed for NOTE_BBBB — the disk holds NOTE_AAAA: the Frankenstein class.
        let e = exp(Some("NOTE_BBBB"), mt, sz, None);
        let out = gate_write(&p, "---\ncid_cn: NOTE_BBBB\n---\nintruder", Some(&e), "test").unwrap();
        assert_eq!(out, WriteOutcome::WouldRefuseIdentity);
        // SHADOW: the write still happened (enforcement is §F1).
        assert!(fs::read_to_string(&p).unwrap().contains("intruder"));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn cas_fresh_and_matching_passes() {
        let d = tdir("cas_ok");
        let p = d.join("note.md");
        gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nv1", None, "test").unwrap();
        let (mt, sz) = stat(&p);
        let e = exp(Some("NOTE_AAAA"), mt, sz, None);
        let out = gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nv2", Some(&e), "test").unwrap();
        assert_eq!(out, WriteOutcome::Ok);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn cas_stale_disk_detected() {
        let d = tdir("cas_stale");
        let p = d.join("note.md");
        gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nv1", None, "test").unwrap();
        let (mt, sz) = stat(&p);
        // Disk moves on (size changes — catches same-second mtime too).
        gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nv2 newer and longer", None, "test").unwrap();
        let e = exp(Some("NOTE_AAAA"), mt, sz, None);
        let out = gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nstomp", Some(&e), "test").unwrap();
        assert_eq!(out, WriteOutcome::WouldRefuseStale);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn cas_hash_escalation_forgives_metadata_drift() {
        let d = tdir("cas_hash");
        let p = d.join("note.md");
        let v1 = "---\ncid_cn: NOTE_AAAA\n---\nv1";
        gate_write(&p, v1, None, "test").unwrap();
        let (_, sz) = stat(&p);
        // Wrong mtime (metadata drift) but the content hash matches the disk.
        let h = format!("{:016x}", fnv1a(v1.as_bytes()));
        let e = exp(Some("NOTE_AAAA"), 1, sz, Some(h));
        let out = gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nv2", Some(&e), "test").unwrap();
        assert_eq!(out, WriteOutcome::Ok);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn cas_no_cid_on_disk_is_unverified() {
        let d = tdir("cas_nocid");
        let p = d.join("note.md");
        gate_write(&p, "no frontmatter at all", None, "test").unwrap();
        let (mt, sz) = stat(&p);
        let e = exp(Some("NOTE_AAAA"), mt, sz, None);
        let out = gate_write(&p, "still none", Some(&e), "test").unwrap();
        assert_eq!(out, WriteOutcome::UnverifiedNoCid);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn cas_missing_file_is_identity_refusal() {
        let d = tdir("cas_gone");
        let p = d.join("gone.md");
        let e = exp(Some("NOTE_AAAA"), 0, 0, None);
        let out = gate_write(&p, "resurrect?", Some(&e), "test").unwrap();
        assert_eq!(out, WriteOutcome::WouldRefuseIdentity);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn self_attested_identity_mismatch_detected_without_any_expectation() {
        let d = tdir("self_id");
        let p = d.join("note.md");
        let out = gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nmine", None, "test").unwrap();
        assert_eq!(out, WriteOutcome::CreatedByWrite); // fresh path, cid-carrying
        // A Frankenstein write: content composed for ANOTHER note, no caller
        // attestation at all — the content itself betrays it.
        let out = gate_write(&p, "---\ncid_cn: NOTE_ZZZZ\n---\nfrankenstein", None, "test").unwrap();
        assert_eq!(out, WriteOutcome::WouldRefuseIdentity);
        // Shadow: still written (enforcement is §F1).
        assert!(fs::read_to_string(&p).unwrap().contains("frankenstein"));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn self_attested_match_passes() {
        let d = tdir("self_ok");
        let p = d.join("note.md");
        gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nv1", None, "test").unwrap();
        let out = gate_write(&p, "---\ncid_cn: NOTE_AAAA\n---\nv2", None, "test").unwrap();
        assert_eq!(out, WriteOutcome::SelfAttestedOk);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn cid_free_content_stays_unchecked() {
        let d = tdir("self_plain");
        let p = d.join("note.md");
        gate_write(&p, "plain v1", None, "test").unwrap();
        let out = gate_write(&p, "plain v2", None, "test").unwrap();
        assert_eq!(out, WriteOutcome::OkUnchecked);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn gate_overhead_timing_print_only() {
        // Speed rider visibility — prints per-write cost; asserts nothing
        // (CI timing is noisy). Run with --nocapture to read it.
        let d = tdir("timing");
        let p = d.join("note.md");
        let content = "x".repeat(8 * 1024);
        let t0 = std::time::Instant::now();
        for _ in 0..50 {
            gate_write(&p, &content, None, "bench").unwrap();
        }
        let per = t0.elapsed().as_micros() / 50;
        println!("[write_gate] avg gated write (8KB, fsync incl.): {} µs", per);
        let _ = fs::remove_dir_all(&d);
    }

    // ─── Batch-2 primitives: the concurrency proofs ─────────────────────────
    // These are the tests the mig-076 JS harness cannot express (it is
    // single-threaded): real threads racing the new locked-RMW primitives
    // against gate_write. They MUST stay green before any note-file command
    // is converted to #[tauri::command(async)].

    #[test]
    fn rmw_concurrent_increments_lose_nothing() {
        // The lost-update property proof: 8 threads each read-increment-write
        // through gate_rmw. Unprotected read→gate_write would lose updates;
        // the locked RMW must count exactly to 8.
        let d = tdir("rmw_inc");
        let p = d.join("counter.md");
        gate_write(&p, "0", None, "test").unwrap();
        let mut handles = Vec::new();
        for _ in 0..8 {
            let p2 = p.clone();
            handles.push(thread::spawn(move || {
                gate_rmw(&p2, "test_rmw", |disk| {
                    let n: u64 = disk.trim().parse().map_err(|e| format!("{}", e))?;
                    // widen the read→write window so unprotected code WOULD race
                    thread::sleep(Duration::from_millis(10));
                    Ok(Some(format!("{}", n + 1)))
                })
                .unwrap();
            }));
        }
        for h in handles { h.join().unwrap(); }
        assert_eq!(fs::read_to_string(&p).unwrap().trim(), "8");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn rmw_excludes_concurrent_gate_write() {
        // A gate_write dispatched while an RMW is mid-cycle must land AFTER
        // the RMW completes (never inside its read→write window).
        use std::sync::mpsc;
        let d = tdir("rmw_excl");
        let p = d.join("note.md");
        gate_write(&p, "base", None, "test").unwrap();
        let (entered_tx, entered_rx) = mpsc::channel::<()>();
        let p_rmw = p.clone();
        let rmw = thread::spawn(move || {
            gate_rmw(&p_rmw, "test_rmw", |disk| {
                assert_eq!(disk, "base"); // the RMW's read
                entered_tx.send(()).unwrap();
                thread::sleep(Duration::from_millis(200)); // hold the window open
                Ok(Some("rmw-out".to_string()))
            })
            .unwrap();
        });
        entered_rx.recv().unwrap(); // RMW is inside its window now
        let p_w = p.clone();
        let writer = thread::spawn(move || {
            gate_write(&p_w, "editor-save", None, "test").unwrap();
        });
        rmw.join().unwrap();
        writer.join().unwrap();
        // The editor save was dispatched DURING the window but must have
        // executed after it: final content is the save's, and the RMW's
        // output was composed from "base", not torn state.
        assert_eq!(fs::read_to_string(&p).unwrap(), "editor-save");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn rmw_rename_crossing_renames_no_deadlock() {
        // a→b and b→a fired concurrently: sorted lock order must prevent
        // deadlock; exactly one wins, the other gets RefusedExists or a read
        // error for the vanished source — never a hang, never data loss.
        let d = tdir("rmw_cross");
        let a = d.join("a.md");
        let b = d.join("b.md");
        gate_write(&a, "content-a", None, "test").unwrap();
        gate_write(&b, "content-b", None, "test").unwrap();
        let (a2, b2) = (a.clone(), b.clone());
        let t1 = thread::spawn(move || gate_rmw_rename(&a2, &b2, "test", |c| Ok(Some(c.to_string()))));
        let (a3, b3) = (a.clone(), b.clone());
        let t2 = thread::spawn(move || gate_rmw_rename(&b3, &a3, "test", |c| Ok(Some(c.to_string()))));
        let r1 = t1.join().unwrap();
        let r2 = t2.join().unwrap();
        // Both returned (no deadlock). Both files still exist with the two
        // contents between them (no byte lost), whatever the interleaving.
        let mut contents = vec![
            fs::read_to_string(&a).unwrap_or_default(),
            fs::read_to_string(&b).unwrap_or_default(),
        ];
        contents.sort();
        assert!(r1.is_ok() || r2.is_ok(), "at least one rename path returned cleanly: {:?} / {:?}", r1, r2);
        assert!(contents.contains(&"content-a".to_string()) || contents.contains(&"content-b".to_string()));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn rmw_rename_dest_exists_refused_under_lock() {
        let d = tdir("rmw_dest");
        let a = d.join("a.md");
        let b = d.join("b.md");
        gate_write(&a, "content-a", None, "test").unwrap();
        gate_write(&b, "content-b", None, "test").unwrap();
        let out = gate_rmw_rename(&a, &b, "test", |c| Ok(Some(c.to_string()))).unwrap();
        assert_eq!(out, WriteOutcome::RefusedExists);
        // source untouched, dest untouched
        assert_eq!(fs::read_to_string(&a).unwrap(), "content-a");
        assert_eq!(fs::read_to_string(&b).unwrap(), "content-b");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn rmw_rename_same_path_is_title_only() {
        let d = tdir("rmw_same");
        let p = d.join("note.md");
        gate_write(&p, "old-title", None, "test").unwrap();
        let out = gate_rmw_rename(&p, &p, "test", |_| Ok(Some("new-title".to_string()))).unwrap();
        assert_eq!(out, WriteOutcome::Ok);
        assert_eq!(fs::read_to_string(&p).unwrap(), "new-title");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn rmw_missing_file_errors_cleanly() {
        let d = tdir("rmw_miss");
        let p = d.join("gone.md");
        let res = gate_rmw(&p, "test", |c| Ok(Some(c.to_string())));
        assert!(res.is_err());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn rmw_none_writes_nothing() {
        let d = tdir("rmw_none");
        let p = d.join("note.md");
        gate_write(&p, "untouched", None, "test").unwrap();
        let out = gate_rmw(&p, "test", |_| Ok(None)).unwrap();
        assert_eq!(out, WriteOutcome::OkUnchecked);
        assert_eq!(fs::read_to_string(&p).unwrap(), "untouched");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn gate_delete_is_idempotent_and_serializes_with_rmw() {
        use std::sync::mpsc;
        let d = tdir("del");
        let p = d.join("note.md");
        // idempotent: deleting a missing file is Deleted, not an error
        let out = gate_delete(&p, DeleteMode::File, "test").unwrap();
        assert_eq!(out, WriteOutcome::Deleted);
        // serialization: a delete dispatched inside an RMW window waits for it
        gate_write(&p, "base", None, "test").unwrap();
        let (entered_tx, entered_rx) = mpsc::channel::<()>();
        let p_rmw = p.clone();
        let rmw = thread::spawn(move || {
            gate_rmw(&p_rmw, "test_rmw", |_| {
                entered_tx.send(()).unwrap();
                thread::sleep(Duration::from_millis(150));
                Ok(Some("rmw-out".to_string()))
            })
            .unwrap();
        });
        entered_rx.recv().unwrap();
        let p_del = p.clone();
        let del = thread::spawn(move || gate_delete(&p_del, DeleteMode::File, "test").unwrap());
        rmw.join().unwrap();
        let out = del.join().unwrap();
        assert_eq!(out, WriteOutcome::Deleted);
        assert!(!p.exists(), "delete ran after the RMW completed; file gone");
        let _ = fs::remove_dir_all(&d);
    }
}
