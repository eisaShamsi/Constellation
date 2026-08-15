//! MIG-111 Phase 0.2 (R5) — **the per-universe OWNER LOCK.**
//!
//! One Constellation instance "owns" a universe while it has it ACTIVE. Ownership is held as
//! an **OS file lock** (`LockFileEx` on Windows, `flock` on macOS/Linux, via `fs4`), because
//! an OS lock has the two properties this migration needs and the retired probe lacked:
//!
//!  1. **It sees an IDLE holder.** The old `BEGIN EXCLUSIVE; ROLLBACK` probe against the
//!     child's `search.db` acquires only SQLite's WRITE lock — in WAL mode an instance that
//!     merely has the universe open, with no write in flight, does NOT hold that lock, so the
//!     probe answered "not open elsewhere" in precisely the routine case it existed to catch
//!     (a false NEGATIVE, proven by the MIG-111 adversarial pass and pinned by the
//!     two-process test below). The owner lock is held continuously for the whole session,
//!     idle or not.
//!  2. **It dies with the process.** A crashed holder's lock evaporates — there is no stale
//!     lock FILE to mis-trust, so "stale lock recovery" is simply: if the lock is acquirable,
//!     there is no live holder, whatever leftover metadata says.
//!
//! ## The two-file shape (adversarial finding H4)
//!
//! Windows `LockFileEx` locks are **mandatory**: an exclusive lock on a byte range blocks
//! other processes' READS of that range. A single lock-file that also carried the owner's
//! identity would therefore be unreadable at exactly the moment a refused prober needs to say
//! WHO holds it. So ownership is two files, both in `<universe root>/.constellation/`:
//!
//!  * **`owner.lock`** — zero-byte, never read, held with `try_lock_exclusive` for the
//!    session's lifetime. The lock IS the truth.
//!  * **`owner.info.json`** — freely readable metadata (`pid`, `hostname`, timestamps),
//!    written by the holder BEFORE acquiring and refreshed by `refresh_heartbeat`. Never
//!    locked. It is DIAGNOSTIC ONLY: the probe's acquirability decides liveness; the info
//!    file only supplies the words for the refusal message ("held by Constellation on
//!    ALSHAMSI-PC since 09:14"). A leftover info file with an acquirable lock is crash
//!    residue and is overwritten on the next acquire, never trusted.
//!
//! ## Identity
//!
//! A universe's lock identity is its **canonicalized root path** (`fs::canonicalize`), which
//! folds Windows case-insensitivity and `\\?\` prefixes; macOS NFC/NFD variance folds the
//! same way because the lock lives INSIDE the root — two spellings of one directory reach the
//! same `owner.lock` inode/file. Comparisons in this module go through `canon()` only.
//!
//! ## Scope of THIS step (0.2 — foundations, no user-visible change)
//!
//! The mechanism + wiring: the active universe's lock is acquired on activation
//! (`set_active_universe` / first boot resolve) and released on switch;
//! `federation::migrate::is_cuniverse_open_elsewhere` now consults the owner lock (and keeps
//! the SQLite probe only as a supplement for NON-Constellation tools holding the DB).
//! ENFORCEMENT — refusing routed writes to a universe owned elsewhere, the read-only note
//! presentation with the Boss-ruled persistent quiet line — is Phase 1.4's step, not this
//! one. If acquisition fails at activation today, behaviour is unchanged (two instances could
//! already do this before MIG-111); the state is recorded loudly so Phase 1 can flip
//! enforcement on a mechanism that is already proven.
use fs4::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerInfo {
    pub pid: u32,
    pub hostname: String,
    pub acquired_at: String,
    pub heartbeat_at: String,
}

/// A held ownership of one universe. Dropping releases the OS lock; the OS releases it on
/// crash regardless — that asymmetry (explicit release is a courtesy, crash-release is the
/// guarantee) is the whole reason this is an OS lock and not a marker file.
pub struct OwnerLock {
    file: File,
    root: PathBuf,
}

/// What a probe learned about a universe's ownership.
#[derive(Debug)]
pub enum Ownership {
    /// No live holder (leftover info, if any, is crash residue).
    Free,
    /// A live holder exists elsewhere; `info` is best-effort (None if the info file is
    /// missing/corrupt — the LOCK decides liveness, the info only supplies words).
    HeldElsewhere { info: Option<OwnerInfo> },
}

fn canon(root: &Path) -> PathBuf {
    fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf())
}

fn lock_dir(root: &Path) -> PathBuf {
    root.join(".constellation")
}

fn lock_path(root: &Path) -> PathBuf {
    lock_dir(root).join("owner.lock")
}

fn info_path(root: &Path) -> PathBuf {
    lock_dir(root).join("owner.info.json")
}

fn now_iso() -> String {
    // Seconds precision is plenty for a diagnostic timestamp.
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn write_info(root: &Path) {
    let info = OwnerInfo {
        pid: std::process::id(),
        hostname: hostname_lossy(),
        acquired_at: now_iso(),
        heartbeat_at: now_iso(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&info) {
        // Best-effort by design: the info file is words, not truth. A failed write must
        // never fail the acquire — the refusal message just gets less specific.
        let _ = fs::write(info_path(root), json);
    }
}

fn hostname_lossy() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "this computer".to_string())
}

impl OwnerLock {
    /// Acquire ownership of `root`'s universe. Fails fast (never blocks) when a live holder
    /// exists elsewhere.
    pub fn acquire(root: &Path) -> Result<OwnerLock, Ownership> {
        let root = canon(root);
        if let Err(e) = fs::create_dir_all(lock_dir(&root)) {
            // No .constellation dir and none creatable — treat as free-but-unlockable;
            // callers log. (A universe without .constellation is not a universe yet.)
            eprintln!("[universe-lock] cannot prepare lock dir for {:?}: {}", root, e);
        }
        let file = match OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(lock_path(&root))
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[universe-lock] cannot open owner.lock for {:?}: {}", root, e);
                return Err(Ownership::Free);
            }
        };
        match file.try_lock_exclusive() {
            Ok(()) => {
                write_info(&root);
                Ok(OwnerLock { file, root })
            }
            Err(_) => Err(probe_info(&root)),
        }
    }

    /// Refresh the diagnostic heartbeat (the LOCK is the liveness truth; this only keeps the
    /// refusal message's "since …" honest across long sessions).
    pub fn refresh_heartbeat(&self) {
        if let Ok(txt) = fs::read_to_string(info_path(&self.root)) {
            if let Ok(mut info) = serde_json::from_str::<OwnerInfo>(&txt) {
                info.heartbeat_at = now_iso();
                if let Ok(json) = serde_json::to_string_pretty(&info) {
                    let _ = fs::write(info_path(&self.root), json);
                }
            }
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for OwnerLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
        // The info file is left in place deliberately: with the lock released it is inert
        // residue (the probe's acquirability overrides it), and keeping it means a crashed
        // OTHER instance's stale info never gets confused with ours — the next acquire
        // overwrites it wholesale.
    }
}

fn probe_info(root: &Path) -> Ownership {
    let info = fs::read_to_string(info_path(root))
        .ok()
        .and_then(|t| serde_json::from_str::<OwnerInfo>(&t).ok());
    Ownership::HeldElsewhere { info }
}

/// Is `root`'s universe owned by a LIVE holder elsewhere? Never blocks.
///
/// The mechanism: try a shared lock on `owner.lock`. The holder keeps an EXCLUSIVE lock for
/// its whole session, so shared-acquirable ⟺ no live holder — including the IDLE holder the
/// retired `BEGIN EXCLUSIVE` probe could not see.
pub fn probe(root: &Path) -> Ownership {
    let root = canon(root);
    let path = lock_path(&root);
    let file = match OpenOptions::new().read(true).open(&path) {
        // No lock file at all → nothing has ever owned it (or the universe pre-dates 0.2).
        Err(_) => return Ownership::Free,
        Ok(f) => f,
    };
    match file.try_lock_shared() {
        Ok(()) => {
            let _ = file.unlock();
            Ownership::Free
        }
        Err(_) => probe_info(&root),
    }
}

// ─── The process-wide holder for the ACTIVE universe's lock ─────────────────────────────────

static ACTIVE_OWNER: OnceLock<Mutex<Option<OwnerLock>>> = OnceLock::new();

fn active_owner() -> &'static Mutex<Option<OwnerLock>> {
    ACTIVE_OWNER.get_or_init(|| Mutex::new(None))
}

/// Called on universe activation (boot resolve + every switch): release the previous
/// universe's ownership, acquire the new one.
///
/// Phase 0.2 policy — RECORD, do not yet ENFORCE: if the new universe is owned elsewhere,
/// behaviour is exactly as before this step (two instances could already open one universe),
/// but the fact is logged loudly and the holder stays `None`, so Phase 1.4 can flip refusal
/// on a mechanism that has been running in the field since this commit.
pub fn activate(root: &Path) {
    let mut slot = match active_owner().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    *slot = None; // release the previous universe's lock FIRST (Drop unlocks)
    match OwnerLock::acquire(root) {
        Ok(lock) => {
            *slot = Some(lock);
        }
        Err(Ownership::HeldElsewhere { info }) => {
            let who = info
                .map(|i| format!("{} (pid {}), heartbeat {}", i.hostname, i.pid, i.heartbeat_at))
                .unwrap_or_else(|| "another Constellation window".to_string());
            eprintln!(
                "[universe-lock] NOT ENFORCED YET (MIG-111 Phase 1.4): {:?} is owned by {} — \
                 this instance proceeds as before, unlocked",
                root, who
            );
        }
        Err(Ownership::Free) => {
            // acquire() only returns Free on an fs error opening the lock file; already logged.
        }
    }
}

/// Does THIS process currently hold `root`'s ownership? (Used by the migrate probe so a
/// parent never mistakes its OWN activation lock for a foreign holder.)
pub fn held_by_us(root: &Path) -> bool {
    let root = canon(root);
    match active_owner().lock() {
        Ok(g) => g.as_ref().map(|l| l.root == root).unwrap_or(false),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_root(t: &TempDir) -> PathBuf {
        let root = t.path().join("U");
        fs::create_dir_all(root.join(".constellation")).unwrap();
        root
    }

    #[test]
    fn acquire_then_probe_same_process_reports_held() {
        let t = TempDir::new().unwrap();
        let root = make_root(&t);
        let lock = OwnerLock::acquire(&root).expect("first acquire succeeds");
        // A second acquire in the SAME process must fail too (fs4 locks are per-handle).
        match OwnerLock::acquire(&root) {
            Err(Ownership::HeldElsewhere { info }) => {
                let info = info.expect("info file written by the holder");
                assert_eq!(info.pid, std::process::id());
            }
            other => panic!("expected HeldElsewhere, got {:?}", other.err()),
        }
        drop(lock);
        assert!(matches!(probe(&root), Ownership::Free), "released → Free");
    }

    #[test]
    fn probe_without_any_lock_file_is_free() {
        let t = TempDir::new().unwrap();
        let root = make_root(&t);
        assert!(matches!(probe(&root), Ownership::Free));
    }

    #[test]
    fn crash_residue_info_without_lock_is_free_and_overwritten_on_acquire() {
        let t = TempDir::new().unwrap();
        let root = make_root(&t);
        // Simulate a crashed holder: info file present, NO lock held.
        fs::write(
            info_path(&canon(&root)),
            r#"{"pid":99999,"hostname":"ghost","acquired_at":"x","heartbeat_at":"x"}"#,
        )
        .unwrap();
        assert!(
            matches!(probe(&root), Ownership::Free),
            "the LOCK decides liveness; leftover info is residue"
        );
        let _lock = OwnerLock::acquire(&root).expect("acquire over residue succeeds");
        let txt = fs::read_to_string(info_path(&canon(&root))).unwrap();
        let info: OwnerInfo = serde_json::from_str(&txt).unwrap();
        assert_eq!(info.pid, std::process::id(), "residue overwritten wholesale");
    }

    /// **The two-process proof (the 0.2 verification clause), and the red half of the pair:**
    /// a REAL second process holds the lock IDLE — no SQLite write in flight — and
    /// (a) the retired `BEGIN EXCLUSIVE` probe says "not open elsewhere" (the false negative
    ///     the adversarial pass certified),
    /// (b) the owner-lock probe says HELD, with the holder's info readable.
    ///
    /// The child is this same test binary re-invoked with `CONSTELLATION_LOCK_HOLDER` set
    /// (the `lock_holder_child` test below), which acquires, drops a ready-marker, and holds
    /// until the parent deletes the marker.
    #[test]
    fn two_processes_idle_holder_detected_where_old_probe_fails() {
        let t = TempDir::new().unwrap();
        let root = make_root(&t);
        // A search.db the OLD probe can be run against, idle (no writer).
        let db = root.join(".constellation").join("search.db");
        {
            let c = rusqlite::Connection::open(&db).unwrap();
            c.pragma_update(None, "journal_mode", "WAL").unwrap();
            c.execute("CREATE TABLE t (v)", []).unwrap();
        }

        let exe = std::env::current_exe().unwrap();
        let ready = root.join("holder.ready");
        let mut child = std::process::Command::new(&exe)
            .args(["universe_lock::tests::lock_holder_child", "--exact", "--nocapture", "--include-ignored"])
            .env("CONSTELLATION_LOCK_HOLDER", &root)
            .env("CONSTELLATION_LOCK_READY", &ready)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn child holder");

        // Wait for the child to hold the lock (marker appears), max ~10s.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !ready.exists() {
            assert!(std::time::Instant::now() < deadline, "child never became ready");
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // (a) THE RED HALF: the retired probe cannot see the idle holder.
        assert!(
            !crate::federation::migrate::sqlite_write_lock_held(&db),
            "BEGIN EXCLUSIVE must report NOT-held for an idle holder — the documented false \
             negative this module exists to fix (if this ever fails, SQLite's locking model \
             changed and the module doc must be revisited)"
        );

        // (b) THE GREEN HALF: the owner lock sees it, with words.
        match probe(&root) {
            Ownership::HeldElsewhere { info } => {
                let info = info.expect("holder wrote info");
                assert_eq!(info.pid, child.id(), "the info names the actual holder process");
            }
            Ownership::Free => panic!("owner-lock probe missed a live idle holder"),
        }

        // Release: delete the marker; the child exits and the lock evaporates.
        let _ = fs::remove_file(&ready);
        let _ = child.wait();
        assert!(matches!(probe(&root), Ownership::Free), "child exit releases the lock");
    }

    /// Not a test — the CHILD process body for the two-process proof above. Ignored in
    /// normal runs; the parent invokes it explicitly with `--include-ignored` and the env
    /// vars set. Without them it exits immediately.
    #[test]
    #[ignore = "child-process helper for two_processes_idle_holder_detected_where_old_probe_fails"]
    fn lock_holder_child() {
        let (Ok(root), Ok(ready)) = (
            std::env::var("CONSTELLATION_LOCK_HOLDER"),
            std::env::var("CONSTELLATION_LOCK_READY"),
        ) else {
            return;
        };
        let root = PathBuf::from(root);
        let ready = PathBuf::from(ready);
        let _lock = OwnerLock::acquire(&root).expect("child acquires");
        fs::write(&ready, b"held").unwrap();
        // Hold, IDLE, until the parent removes the marker (or 30s safety cap).
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while ready.exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}
