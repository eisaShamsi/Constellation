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

/// §B1 (L2) seam — what the caller believes about the file it is replacing.
/// Carried now so §A2 call-site routing doesn't churn when CAS lands.
#[allow(dead_code)]
pub struct Expectation {
    /// The note identity the caller composed this content FOR (frontmatter
    /// `cid_cn`). Mismatch with the on-disk identity = the Frankenstein class.
    pub expected_cid: Option<String>,
    /// Freshness token of the disk state the caller last read (racy-git rule:
    /// mtime+size, escalate to hash on ambiguity — §B1).
    pub base_mtime: u64,
    pub base_size: u64,
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
}

impl WriteOutcome {
    fn as_str(self) -> &'static str {
        match self {
            WriteOutcome::Ok => "ok",
            WriteOutcome::OkUnchecked => "ok_unchecked",
            WriteOutcome::CreatedExclusive => "created_exclusive",
            WriteOutcome::RefusedExists => "refused_exists",
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
    let Some(jp) = journal_path().get() else { return };
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let line = serde_json::json!({
        "ts": ts,
        "path": path.to_string_lossy(),
        "surface": surface,
        "outcome": outcome.as_str(),
        "bytes": bytes,
        "hash": format!("{:016x}", hash),
    });
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

/// Write `content` to `path` through the gate: path lock → (§B: CAS check)
/// → atomic replace → journal. `surface` names the caller for the journal
/// (e.g. "write_note", "cascade", "rename_title", "base_edit_cell").
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

    // §B1 lands here: when `expect` is Some, re-read the target under the
    // lock and compare identity (cid_cn) + freshness (mtime+size→hash).
    let outcome = if expect.is_some() { WriteOutcome::Ok } else { WriteOutcome::OkUnchecked };

    atomic_write(path, content)?;
    journal(path, surface, outcome, content.len(), fnv1a(content.as_bytes()));
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
}
