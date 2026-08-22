//! MIG-108 — One Universe, One Location: the relocation engine.
//!
//! **Concept.** A universe is one directory that wholly contains its knowledge. This module
//! relocates every external own-library under the universe root — app-mediated, journaled,
//! snapshot-first (the Lightroom "Update Folder Location" shape; Architect doc §0/§5).
//!
//! Slice 1 (this file's first landing): the three foundations everything else stands on —
//!   · the **journal** — crash-safe, resumable record of intent and progress. The boot
//!     reconcile is NOT a net here (it hard-aborts above max(200, 10%) stale rows; at
//!     migration scale ~98% of the index goes stale for a moment — Architect H4), so the
//!     journal is the ONLY recovery mechanism and is written BEFORE each mutating step.
//!   · the **pre-flight classifier** — a pure function over the registry that decides, for
//!     every entry: skip (already under root), MOVE, COPY (Boss ruling D3), skip-foreign
//!     (another universe's root or child — Architect H6), or report-missing. Pure so every
//!     rule is testable without an AppHandle.
//!   · the **snapshot** — WAL-checkpointed copy of search.db (the system of record for the
//!     earned link data) + the path-bearing JSON stores, verified by reopening (H5).
//!
//! Slices 2+ add the move/rewrite phases; the thin `#[tauri::command]` wrappers arrive with
//! the proposal UI (Slice 4).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Phase-4 audit (MED, migration-path) — **the engine must not be guillotined mid-move.**
///
/// The PJ-103 graceful-close handshake holds the window for at most 5 s and then destroys
/// it. Mid-engine that lands the process between two directory moves, or inside a copy —
/// and while the journal makes it *recoverable* (that is exactly what the `started` /
/// `copied` sub-states are for), recoverable is not the same as acceptable when the user is
/// watching a screen that says "please keep Constellation open". A sentence of prose was
/// the ONLY thing standing between a stray click on the X and a half-moved universe.
///
/// So the close is REFUSED for as long as the engine holds the world open. This is not a
/// trap: a genuinely hung engine can still be killed from the OS, and that path is the
/// journaled crash the resume flow already handles.
static ENGINE_RUNNING: AtomicBool = AtomicBool::new(false);

/// True while a unification / restore is mutating the world. Read by the window-close
/// handler in `lib.rs`.
pub fn engine_is_running() -> bool {
    ENGINE_RUNNING.load(Ordering::SeqCst)
}

/// RAII so the flag clears on EVERY exit path — the `?` early-returns inside the engine
/// loop and a panic included. A flag left set would make the window permanently unclosable,
/// which would be a worse bug than the one this guard prevents.
struct RunningGuard;
impl RunningGuard {
    fn new() -> Self {
        ENGINE_RUNNING.store(true, Ordering::SeqCst);
        RunningGuard
    }
}
impl Drop for RunningGuard {
    fn drop(&mut self) {
        ENGINE_RUNNING.store(false, Ordering::SeqCst);
    }
}

// ─── Path normalization ─────────────────────────────────────────────────────────────────────
//
// One rule for every comparison in this module (H3): forward slashes, no trailing slash,
// lowercase, Unicode NFC. Stored paths are byte-inconsistent across writers (Rust `\` vs JS
// `/`; NFC is live on Arabic names), so the engine NEVER compares raw strings.

pub(crate) fn norm(p: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    p.nfc()
        .collect::<String>()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

/// Separator-bounded "is `p` under `root` (or equal)?" on normalized forms.
pub(crate) fn norm_under(p: &str, root: &str) -> bool {
    let (p, r) = (norm(p), norm(root));
    p == r || p.starts_with(&format!("{}/", r))
}

/// Best-effort same-volume check. Windows: compare drive/UNC prefixes; elsewhere: device ids.
/// `false` routes the entry to the copy+remove fallback — never a correctness risk, only cost.
pub(crate) fn same_volume(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        fn prefix(p: &Path) -> Option<String> {
            match p.components().next() {
                Some(std::path::Component::Prefix(pr)) => {
                    Some(pr.as_os_str().to_string_lossy().to_lowercase())
                }
                _ => None,
            }
        }
        match (prefix(a), prefix(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::MetadataExt;
        match (std::fs::metadata(a), std::fs::metadata(b)) {
            (Ok(ma), Ok(mb)) => ma.dev() == mb.dev(),
            _ => false,
        }
    }
}

// ─── Classification ─────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum EntryClass {
    /// Already under the root (incl. the universe_notes entry itself) — nothing to do.
    UnderRoot,
    /// External own library → same-volume rename (or copy+remove fallback) to `dest`.
    Move,
    /// Boss ruling D3 — content is COPIED under the root and the registration re-points to
    /// the copy; the original files stay where they are and are simply no longer registered.
    Copy,
    /// The entry is another registered universe's root, or sits under one, or is a resolved
    /// child-universe root (H6). NEVER moved — relocating it would corrupt registries this
    /// migration does not own. Skipped and reported.
    ForeignUniverse { reason: String },
    /// The registered path does not exist on disk. Skipped and reported.
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightEntry {
    pub library_id: String,
    pub library_name: String,
    pub old_path: String,
    pub class: EntryClass,
    /// Destination under the root (Move/Copy only), basename de-collided.
    pub dest: Option<String>,
    /// True when the old path and the root share a volume (rename vs copy+remove).
    pub same_volume: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub universe_root: String,
    pub entries: Vec<PreflightEntry>,
    /// Basenames that needed a numeric suffix at the destination (informational).
    pub decollided: Vec<String>,
}

impl PreflightReport {
    pub fn to_move(&self) -> impl Iterator<Item = &PreflightEntry> {
        self.entries
            .iter()
            .filter(|e| matches!(e.class, EntryClass::Move | EntryClass::Copy))
    }
}

/// The pure classifier. `copy_paths` = registered paths the caller wants COPIED rather than
/// moved (D3 — the Boss universe passes PJ-065-test-book); `foreign_roots` = every OTHER
/// registered universe root + every resolved child-universe root of ANY registered universe
/// (the caller assembles this from `registered_universe_roots` + the child resolvers,
/// EXCLUDING the active universe's own root, which is of course "under root", not foreign).
pub fn classify(
    universe_root: &str,
    libraries: &[crate::libraries::LibraryInfo],
    copy_paths: &[String],
    foreign_roots: &[String],
) -> PreflightReport {
    let copy_set: HashSet<String> = copy_paths.iter().map(|p| norm(p)).collect();
    let mut entries = Vec::new();
    let mut decollided = Vec::new();
    // Names already taken at the destination: existing fs entries at the root + destinations
    // this same plan has already claimed (two same-basename externals must not merge — H7).
    let mut taken: HashSet<String> = std::fs::read_dir(universe_root)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_lowercase()))
                .collect()
        })
        .unwrap_or_default();

    for lib in libraries {
        let is_under = norm_under(&lib.path, universe_root);
        let class = if is_under {
            EntryClass::UnderRoot
        } else if !Path::new(&lib.path).is_dir() {
            EntryClass::Missing
        } else if let Some(reason) = foreign_reason(&lib.path, foreign_roots) {
            EntryClass::ForeignUniverse { reason }
        } else if let Some(owner) = universe_manifest_at_or_above(Path::new(&lib.path)) {
            // The structural backstop. Reached only when `foreign_roots` did NOT already name
            // this path — i.e. exactly when the registry or a child manifest came back short.
            // Ahead of the Copy arm on purpose: copying a universe's files into another
            // universe as plain content is the same mangling as moving them, which is why
            // `bring_in_library` refuses it too.
            EntryClass::ForeignUniverse {
                reason: format!(
                    "is, or sits inside, a universe of its own: {}",
                    owner.display()
                ),
            }
        } else if copy_set.contains(&norm(&lib.path)) {
            EntryClass::Copy
        } else {
            EntryClass::Move
        };

        let dest = if matches!(class, EntryClass::Move | EntryClass::Copy) {
            let base = Path::new(&lib.path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| lib.name.clone());
            let (final_name, bumped) = free_name(&base, &taken);
            if bumped {
                decollided.push(final_name.clone());
            }
            taken.insert(final_name.to_lowercase());
            Some(
                Path::new(universe_root)
                    .join(&final_name)
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            None
        };

        entries.push(PreflightEntry {
            library_id: lib.id.clone(),
            library_name: lib.name.clone(),
            old_path: lib.path.clone(),
            same_volume: same_volume(Path::new(&lib.path), Path::new(universe_root)),
            class,
            dest,
        });
    }

    // Phase-4 audit — nested actionable entries: an outer library's move physically carries
    // an inner registered library with it, and the inner entry's own move then finds its
    // source gone. Rather than plan a corrupting sequence, the INNER entry is demoted to a
    // reported skip; the user resolves the nesting first (none exist on the Boss universe —
    // all 18 externals are siblings — but the guard is standing behaviour).
    let actionable_roots: Vec<(usize, String)> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e.class, EntryClass::Move | EntryClass::Copy))
        .map(|(i, e)| (i, e.old_path.clone()))
        .collect();
    for (i, path) in &actionable_roots {
        for (j, other) in &actionable_roots {
            if i != j && norm_under(path, other) && norm(path) != norm(other) {
                entries[*i].class = EntryClass::ForeignUniverse {
                    reason: format!("is nested inside another library being relocated ({})", other),
                };
                entries[*i].dest = None;
            }
        }
    }

    PreflightReport {
        universe_root: universe_root.to_string(),
        entries,
        decollided,
    }
}

/// **Does this directory carry a universe manifest?** Asks the DISK, not a report.
///
/// `foreign_reason` answers "is this inside a universe?" from `foreign_roots`, which the caller
/// assembles from `registered_universe_roots` → `load_registry` (universe.rs:138-161) — and
/// `load_registry` returns an EMPTY registry on a path error, a read failure, and a parse
/// failure alike. An empty registry means an empty foreign set, which means `foreign_reason`
/// answers `None` for everything, which means **every** external registered library falls
/// through to `EntryClass::Move`. One unreadable `universes.json` and the plan proposes
/// relocating other universes' content.
///
/// This is the structural backstop: a fact on disk cannot be lost by whichever reader degraded.
///
/// **`fs::metadata`, deliberately not `Path::exists()`.** `exists()` returns `false` both for
/// "absent" and for "present but the metadata could not be read" — reintroducing the very
/// lenient-reader defect one layer down, inside the window between the dialog's plan and the
/// plan `mig108_execute` recomputes. Here an unreadable manifest counts as PRESENT: the guard
/// is monotone toward refusing to relocate, and refusing to move something the user can see is
/// recoverable in a way that moving another universe's files is not.
pub(crate) fn carries_universe_manifest(dir: &Path) -> bool {
    for candidate in [dir.join(".constellation").join("universe.json"), dir.join("universe.json")] {
        match std::fs::metadata(&candidate) {
            Ok(_) => return true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return true, // cannot confirm absence ⇒ assume present
        }
    }
    false
}

/// The nearest directory at or above `path` that carries a universe manifest, if any.
/// Walks to the volume root: a library registered three folders deep inside another universe
/// is that universe's content just as much as its root is.
fn universe_manifest_at_or_above(path: &Path) -> Option<PathBuf> {
    let mut cur = Some(path);
    while let Some(d) = cur {
        if carries_universe_manifest(d) {
            return Some(d.to_path_buf());
        }
        cur = d.parent();
    }
    None
}

fn foreign_reason(path: &str, foreign_roots: &[String]) -> Option<String> {
    for root in foreign_roots {
        if norm(path) == norm(root) {
            return Some(format!("is another universe's root: {}", root));
        }
        if norm_under(path, root) {
            return Some(format!("sits inside another universe: {}", root));
        }
    }
    None
}

/// `base` if free at the root, else `base 2`, `base 3`, … (never silently merging two
/// libraries into one directory). Returns (name, was_bumped).
fn free_name(base: &str, taken: &HashSet<String>) -> (String, bool) {
    if !taken.contains(&base.to_lowercase()) {
        return (base.to_string(), false);
    }
    for n in 2..=9999 {
        let candidate = format!("{} {}", base, n);
        if !taken.contains(&candidate.to_lowercase()) {
            return (candidate, true);
        }
    }
    // 9999 same-named siblings: practically unreachable; caller surfaces the plan anyway.
    (format!("{} {}", base, uuid_suffix()), true)
}

fn uuid_suffix() -> String {
    format!("{:x}", std::process::id())
}

// ─── Journal ────────────────────────────────────────────────────────────────────────────────

pub const JOURNAL_FILE: &str = "mig108-journal.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Plan recorded; nothing mutated yet. Safe to discard.
    Planned,
    /// Snapshot taken and verified; fs still untouched. Safe to discard.
    Snapshotted,
    /// One or more directory moves may have happened (per-entry `moved` flags say which).
    Moving,
    /// Every Move/Copy entry's fs operation is complete; DB not yet rewritten.
    Moved,
    /// The DB transaction committed; JSON stores not yet all rewritten.
    DbRewritten,
    /// All JSON stores rewritten (per-store flags say which); trash not yet consolidated.
    JsonRewritten,
    /// Everything done and verified.
    Done,
    /// The in-tx verification failed and the DB was rolled back. fs moves stand (recorded
    /// per-entry); the resume path surfaces this to the user rather than guessing.
    VerifyFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub library_id: String,
    pub library_name: String,
    pub old_path: String,
    pub new_path: String,
    /// "move" | "copy"
    pub action: String,
    /// The fs operation for this entry completed.
    pub moved: bool,
    /// Phase-4 audit — the fs operation for this entry BEGAN. Journaled before the first
    /// byte moves, so a destination found on resume can be classified: started=true means
    /// it is OUR partial (safe to delete and redo — the source is the authority);
    /// started=false means a genuine collision (hard error, never deleted).
    #[serde(default)]
    pub started: bool,
    /// Phase-4 audit — for copy-based operations (copy-class and cross-volume moves): the
    /// copy completed and was count-verified; only the source removal (moves) remains.
    /// Distinguishes crash-mid-copy (partial DEST, delete+redo) from crash-mid-remove
    /// (partial SOURCE, complete dest — finishing the removal is the only safe direction).
    #[serde(default)]
    pub copied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub version: u32,
    pub universe_root: String,
    pub phase: Phase,
    pub snapshot_db: Option<String>,
    /// (store filename, backup path) pairs.
    pub json_backups: Vec<(String, String)>,
    /// JSON stores already rewritten in the JsonRewritten phase (idempotent resume).
    pub json_rewritten: Vec<String>,
    pub entries: Vec<JournalEntry>,
    /// Pre-move aggregates the verify phase must reproduce EXACTLY (I2).
    pub baseline: Option<Baseline>,
    /// Stage-B 2026-08-01 — WHY the last attempt stopped. The live failure journaled
    /// `VerifyFailed` and nothing else, so the reason had to be reconstructed afterwards from
    /// the user's data; a 45-minute rollback deserves to explain itself. Written on every
    /// verify refusal, cleared when a later attempt gets past it, and surfaced in the resume
    /// card so the user is told rather than left guessing.
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Baseline {
    pub note_meta_rows: i64,
    pub note_links_rows: i64,
    pub note_links_weight_sum: f64,
    pub review_schedule_rows: i64,
}

impl Journal {
    pub fn new(universe_root: &str, report: &PreflightReport) -> Self {
        Journal {
            version: 1,
            universe_root: universe_root.to_string(),
            phase: Phase::Planned,
            snapshot_db: None,
            json_backups: Vec::new(),
            json_rewritten: Vec::new(),
            entries: report
                .to_move()
                .map(|e| JournalEntry {
                    library_id: e.library_id.clone(),
                    library_name: e.library_name.clone(),
                    old_path: e.old_path.clone(),
                    new_path: e.dest.clone().expect("Move/Copy entries carry a dest"),
                    action: match e.class {
                        EntryClass::Copy => "copy".into(),
                        _ => "move".into(),
                    },
                    moved: false,
                    started: false,
                    copied: false,
                })
                .collect(),
            baseline: None,
            last_error: None,
        }
    }

    pub fn path_for(constellation_dir: &Path) -> PathBuf {
        constellation_dir.join(JOURNAL_FILE)
    }

    /// Persist BEFORE the step it describes — the journal must always be ahead of reality
    /// on intent and behind it on completion, so a crash window is always recorded.
    pub fn save(&self, constellation_dir: &Path) -> Result<(), String> {
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        crate::universe::atomic_write(&Self::path_for(constellation_dir), data.as_bytes())
            .map_err(|e| format!("mig108 journal write failed: {}", e))
    }

    pub fn load(constellation_dir: &Path) -> Result<Option<Journal>, String> {
        let p = Self::path_for(constellation_dir);
        if !p.exists() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let j: Journal = serde_json::from_str(&raw)
            .map_err(|e| format!("mig108 journal unreadable ({}) — NOT deleting it; surface to the user", e))?;
        Ok(Some(j))
    }

    /// A journal that represents unfinished work (the boot resume signal).
    pub fn is_unfinished(&self) -> bool {
        !matches!(self.phase, Phase::Done)
    }

    /// Remove the journal — only ever called for Done or for the safe-to-discard phases
    /// (Planned/Snapshotted, where nothing was mutated).
    pub fn discard(constellation_dir: &Path) -> Result<(), String> {
        let p = Self::path_for(constellation_dir);
        if p.exists() {
            std::fs::remove_file(&p).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

// ─── Snapshot ───────────────────────────────────────────────────────────────────────────────

/// The path-bearing JSON stores the snapshot copies and the rewrite phase later touches
/// (Architect H12; json-state map ranking).
pub const JSON_STORES: &[&str] = &[
    "libraries.json",
    "review-pulse.json",
    "workspaces.json",
    "session.json",
    "session.prev.json",
    "collections.json",
    "settings.json",
    "universe.json",
    "bookmarks.json",
];

pub const BACKUP_DIR: &str = "mig108-backup";

/// WAL-checkpoint the live connection, copy search.db (+ any WAL/SHM sidecars), reopen the
/// copy read-only and verify row counts against the live DB; then copy the JSON stores.
/// Returns the updated journal fields (snapshot_db, json_backups, baseline).
///
/// H5: a naive fs::copy of a WAL database silently loses un-checkpointed frames — a corrupt
/// "backup" of the earned ledger, discovered only at restore time. TRUNCATE both flushes and
/// truncates the WAL so the main file alone is complete; sidecars are copied belt-and-braces.
pub fn take_snapshot(
    conn: &rusqlite::Connection,
    db_path: &Path,
    constellation_dir: &Path,
) -> Result<(String, Vec<(String, String)>, Baseline), String> {
    let backup_dir = constellation_dir.join(BACKUP_DIR);
    // Phase-4 audit (LOW, migration-path) — a second run used to copy straight over the first
    // run's backup, silently breaking the summary's promise that "the backup is kept until you
    // choose to remove it". Move the previous one aside instead. Exactly ONE generation is
    // kept: this is a multi-GB copy of the index, and an unbounded chain of them inside the
    // user's own universe folder would be its own defect.
    if backup_dir.exists()
        && std::fs::read_dir(&backup_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    {
        let prev = constellation_dir.join(format!("{}.prev", BACKUP_DIR));
        let _ = std::fs::remove_dir_all(&prev);
        if let Err(e) = std::fs::rename(&backup_dir, &prev) {
            // Never block the migration on backup housekeeping — but never pretend either.
            eprintln!("[mig108] could not set the previous backup aside ({e}); it will be replaced");
        }
    }
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| format!("wal_checkpoint failed: {}", e))?;

    let baseline = read_baseline(conn)?;

    // MIG-111 R11 note — this `fs::copy` is the AUDITED EXEMPTION to the live-WAL copy ban
    // (see `federation::migrate::backup_database`): the checkpoint above ran through the held
    // connection (emptying the WAL into the main file), and the block below VERIFIES the copy
    // opens with a matching baseline. Do not imitate this shape without all three parts.
    let db_backup = backup_dir.join("search.db.pre-mig108");
    std::fs::copy(db_path, &db_backup).map_err(|e| format!("db backup copy failed: {}", e))?;
    for ext in ["-wal", "-shm"] {
        let side = PathBuf::from(format!("{}{}", db_path.display(), ext));
        let dest = backup_dir.join(format!("search.db.pre-mig108{}", ext));
        // Phase-4 audit — a stale sidecar from an EARLIER run paired with a fresh main file
        // is a corrupt backup; remove first, copy only when the live sidecar is non-empty.
        let _ = std::fs::remove_file(&dest);
        if side.exists() && std::fs::metadata(&side).map(|m| m.len() > 0).unwrap_or(false) {
            std::fs::copy(&side, &dest).map_err(|e| format!("sidecar backup failed: {}", e))?;
        }
    }

    // Verify the copy is a complete, openable database with the same counts.
    {
        let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY;
        let check = rusqlite::Connection::open_with_flags(&db_backup, flags)
            .map_err(|e| format!("backup unopenable: {}", e))?;
        let b2 = read_baseline(&check)?;
        if b2 != baseline {
            return Err(format!(
                "backup verification failed: live {:?} vs backup {:?}",
                baseline, b2
            ));
        }
    }

    let mut json_backups = Vec::new();
    for store in JSON_STORES {
        let src = constellation_dir.join(store);
        if src.exists() {
            let dest = backup_dir.join(store);
            std::fs::copy(&src, &dest).map_err(|e| format!("{} backup failed: {}", store, e))?;
            json_backups.push((store.to_string(), dest.to_string_lossy().to_string()));
        }
    }

    Ok((db_backup.to_string_lossy().to_string(), json_backups, baseline))
}

pub fn read_baseline(conn: &rusqlite::Connection) -> Result<Baseline, String> {
    let q = |sql: &str| -> Result<i64, String> {
        conn.query_row(sql, [], |r| r.get(0)).map_err(|e| e.to_string())
    };
    Ok(Baseline {
        note_meta_rows: q("SELECT COUNT(*) FROM note_meta")?,
        note_links_rows: q("SELECT COUNT(*) FROM note_links")?,
        note_links_weight_sum: conn
            .query_row("SELECT COALESCE(SUM(weight), 0.0) FROM note_links", [], |r| r.get(0))
            .map_err(|e| e.to_string())?,
        review_schedule_rows: q("SELECT COUNT(*) FROM review_schedule")?,
    })
}

// ─── Tests ──────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::libraries::LibraryInfo;

    fn lib(id: &str, name: &str, path: &str) -> LibraryInfo {
        LibraryInfo {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            is_universe_notes: false,
            canonical_mode: "compatible".into(),
        }
    }

    fn scratch_universe() -> (tempfile::TempDir, String) {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("Universe");
        std::fs::create_dir_all(root.join(".constellation")).unwrap();
        (td, root.to_string_lossy().to_string())
    }

    // ── PJ-333 — the same concern, the OTHER surface (Boss-ruled 2026-08-22) ──

    /// **The hole, stated as the discriminator between the two predicates.**
    ///
    /// `bring_in_library` asked only `carries_universe_manifest(src)` — "is this folder ITSELF a
    /// universe?" — so a plain subfolder inside an UNREGISTERED universe passed every check and
    /// was relocated out of it. `classify` had already been given the upward walk by PJ-322; this
    /// asserts the two predicates disagree exactly where the defect lived, and that the walking
    /// one is the one that closes it.
    #[test]
    fn a_folder_inside_another_universe_is_seen_only_by_the_walking_predicate() {
        let td = tempfile::tempdir().unwrap();
        let other = td.path().join("Unregistered Universe");
        let inside = other.join("Research").join("Notes");
        std::fs::create_dir_all(&inside).unwrap();
        std::fs::create_dir_all(other.join(".constellation")).unwrap();
        std::fs::write(other.join(".constellation").join("universe.json"), b"{}").unwrap();

        assert!(
            !carries_universe_manifest(&inside),
            "THE HOLE: the folder itself carries no manifest, so the narrow predicate waves it \
             through — and this is the shape `bring_in_library` used to accept"
        );
        let owner = universe_manifest_at_or_above(&inside)
            .expect("the walking predicate must find the universe above it");
        assert_eq!(owner, other, "and it names WHICH universe, so the refusal can say so");
        drop(td);
    }

    /// The guard must not over-refuse: an ordinary folder with no universe anywhere above it
    /// stays bring-in-able, or the feature is dead.
    #[test]
    fn an_ordinary_folder_is_still_bring_in_able() {
        let td = tempfile::tempdir().unwrap();
        let plain = td.path().join("Just A Folder").join("Deeper");
        std::fs::create_dir_all(&plain).unwrap();
        assert!(
            universe_manifest_at_or_above(&plain).is_none(),
            "no manifest at or above ⇒ ordinary content, still ingestible"
        );
        drop(td);
    }

    // ── PJ-322 — the structural backstop (panel-ordered, 2026-08-20) ──
    //
    // `foreign_roots` is a REPORT, assembled from `load_registry`, which returns an EMPTY
    // registry on a path error, a read failure, and a parse failure alike (universe.rs:141-158).
    // These tests pass `&[]` for `foreign_roots` — which is exactly what a degraded read
    // produces — and assert the classifier still refuses to relocate another universe.

    #[test]
    fn a_universe_is_never_moved_even_when_the_registry_came_back_empty() {
        let (td, root) = scratch_universe();
        let other = td.path().join("Other Universe");
        std::fs::create_dir_all(other.join(".constellation")).unwrap();
        std::fs::write(other.join(".constellation").join("universe.json"), b"{}").unwrap();
        let libs = vec![lib("x", "Other", &other.to_string_lossy())];

        // The degraded case: NO foreign roots reported at all.
        let report = classify(&root, &libs, &[], &[]);
        let e = &report.entries[0];
        assert!(
            matches!(e.class, EntryClass::ForeignUniverse { .. }),
            "a universe root must never be Move, even with an empty foreign set; got {:?}",
            e.class
        );
        assert!(e.dest.is_none(), "and it must have no destination: {:?}", e.dest);
        drop(td);
    }

    #[test]
    fn a_library_nested_inside_another_universe_is_foreign_by_structure() {
        let (td, root) = scratch_universe();
        let other = td.path().join("Other Universe");
        std::fs::create_dir_all(other.join("Deep").join("Nested Lib")).unwrap();
        std::fs::write(other.join("universe.json"), b"{}").unwrap(); // root-level manifest form
        let libs = vec![lib(
            "x",
            "Nested",
            &other.join("Deep").join("Nested Lib").to_string_lossy(),
        )];

        let report = classify(&root, &libs, &[], &[]);
        assert!(
            matches!(report.entries[0].class, EntryClass::ForeignUniverse { .. }),
            "the walk must reach an ancestor's manifest; got {:?}",
            report.entries[0].class
        );
        drop(td);
    }

    /// The guard must not over-refuse: an ordinary external folder is still relocatable, or
    /// the whole unification stops working.
    #[test]
    fn an_ordinary_external_folder_is_still_moved() {
        let (td, root) = scratch_universe();
        let ext = td.path().join("Ordinary Lib");
        std::fs::create_dir_all(&ext).unwrap();
        let libs = vec![lib("x", "Ordinary", &ext.to_string_lossy())];
        let report = classify(&root, &libs, &[], &[]);
        assert_eq!(
            report.entries[0].class,
            EntryClass::Move,
            "no manifest anywhere above ⇒ ordinary content"
        );
        drop(td);
    }

    /// An explicit Copy choice does not override the structure. Copying a universe's files in
    /// as plain content is the same mangling as moving them — `bring_in_library` refuses it on
    /// the same grounds.
    #[test]
    fn an_explicit_copy_choice_cannot_override_the_structure() {
        let (td, root) = scratch_universe();
        let other = td.path().join("Other Universe");
        std::fs::create_dir_all(other.join(".constellation")).unwrap();
        std::fs::write(other.join(".constellation").join("universe.json"), b"{}").unwrap();
        let libs = vec![lib("x", "Other", &other.to_string_lossy())];
        let report = classify(&root, &libs, &[other.to_string_lossy().to_string()], &[]);
        assert!(
            matches!(report.entries[0].class, EntryClass::ForeignUniverse { .. }),
            "structure beats the copy list; got {:?}",
            report.entries[0].class
        );
        drop(td);
    }

    // ── classifier ──

    #[test]
    fn classifier_covers_every_class() {
        let (td, root) = scratch_universe();
        // under-root entry (the universe_notes shape: path == root)
        let under = root.clone();
        // external own library
        let ext = td.path().join("External Lib");
        std::fs::create_dir_all(&ext).unwrap();
        // copy-class (D3)
        let copyme = td.path().join("Repo Book");
        std::fs::create_dir_all(&copyme).unwrap();
        // foreign universe root + a library under it
        let foreign = td.path().join("Other Universe");
        std::fs::create_dir_all(foreign.join("Inside")).unwrap();
        // missing
        let missing = td.path().join("Gone");

        let libs = vec![
            lib("u", "Universe Notes", &under),
            lib("a", "External", &ext.to_string_lossy()),
            lib("b", "Book", &copyme.to_string_lossy()),
            lib("c", "ForeignRoot", &foreign.to_string_lossy()),
            lib("d", "InsideForeign", &foreign.join("Inside").to_string_lossy()),
            lib("e", "Missing", &missing.to_string_lossy()),
        ];
        let report = classify(
            &root,
            &libs,
            &[copyme.to_string_lossy().to_string()],
            &[foreign.to_string_lossy().to_string()],
        );

        let by_id = |id: &str| report.entries.iter().find(|e| e.library_id == id).unwrap();
        assert_eq!(by_id("u").class, EntryClass::UnderRoot);
        assert_eq!(by_id("a").class, EntryClass::Move);
        assert_eq!(by_id("b").class, EntryClass::Copy);
        assert!(matches!(by_id("c").class, EntryClass::ForeignUniverse { .. }));
        assert!(matches!(by_id("d").class, EntryClass::ForeignUniverse { .. }));
        assert_eq!(by_id("e").class, EntryClass::Missing);

        // Move/Copy destinations are flat under the root (Boss D1).
        assert_eq!(
            by_id("a").dest.as_deref().map(norm),
            Some(norm(&format!("{}/External Lib", root)))
        );
        assert!(by_id("b").dest.is_some());
        assert!(by_id("c").dest.is_none(), "foreign entries are never given a destination");
    }

    /// H7 — two externals sharing a basename must land in two directories, never merge.
    #[test]
    fn classifier_decollides_same_basenames() {
        let (td, root) = scratch_universe();
        let a = td.path().join("TreeA").join("Notes");
        let b = td.path().join("TreeB").join("Notes");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        // …and a folder named "Notes" ALREADY at the root (taken by the fs).
        std::fs::create_dir_all(Path::new(&root).join("Notes")).unwrap();

        let libs = vec![
            lib("a", "Notes A", &a.to_string_lossy()),
            lib("b", "Notes B", &b.to_string_lossy()),
        ];
        let report = classify(&root, &libs, &[], &[]);
        let dests: Vec<String> = report
            .entries
            .iter()
            .filter_map(|e| e.dest.clone())
            .map(|d| norm(&d))
            .collect();
        assert_eq!(dests.len(), 2);
        assert_ne!(dests[0], dests[1], "same-basename libraries must never merge");
        assert!(
            !dests.contains(&norm(&format!("{}/Notes", root))),
            "the fs-taken name is respected: {:?}",
            dests
        );
        assert_eq!(report.decollided.len(), 2);
    }

    /// Normalization: a stored backslash path under the root is UnderRoot even though the
    /// root is stored with forward slashes (H3's shape at classification time).
    #[test]
    fn classifier_normalizes_separators() {
        let (_td, root) = scratch_universe();
        let stored = format!("{}\\Sub", root.replace('/', "\\"));
        std::fs::create_dir_all(Path::new(&root).join("Sub")).unwrap();
        let libs = vec![lib("s", "Sub", &stored)];
        let report = classify(&root.replace('\\', "/"), &libs, &[], &[]);
        assert_eq!(report.entries[0].class, EntryClass::UnderRoot);
    }

    // ── journal ──

    #[test]
    fn journal_round_trips_and_signals_resume() {
        let (_td, root) = scratch_universe();
        let cdir = Path::new(&root).join(".constellation");

        let ext = Path::new(&root).parent().unwrap().join("Ext");
        std::fs::create_dir_all(&ext).unwrap();
        let libs = vec![lib("x", "Ext", &ext.to_string_lossy())];
        let report = classify(&root, &libs, &[], &[]);

        let mut j = Journal::new(&root, &report);
        assert_eq!(j.entries.len(), 1);
        assert_eq!(j.phase, Phase::Planned);
        j.save(&cdir).unwrap();

        let loaded = Journal::load(&cdir).unwrap().expect("journal exists");
        assert!(loaded.is_unfinished());
        assert_eq!(loaded.entries[0].action, "move");

        // Advance through the phases; each save is a full atomic rewrite.
        j.phase = Phase::Moving;
        j.entries[0].moved = true;
        j.phase = Phase::Moved;
        j.save(&cdir).unwrap();
        let loaded = Journal::load(&cdir).unwrap().unwrap();
        assert!(loaded.entries[0].moved);
        assert!(loaded.is_unfinished());

        j.phase = Phase::Done;
        j.save(&cdir).unwrap();
        assert!(!Journal::load(&cdir).unwrap().unwrap().is_unfinished());
        Journal::discard(&cdir).unwrap();
        assert!(Journal::load(&cdir).unwrap().is_none());
    }

    /// An unreadable journal is an ERROR to surface, never silently discarded — it is the
    /// only record of a possibly half-moved universe.
    #[test]
    fn corrupt_journal_is_surfaced_not_swallowed() {
        let (_td, root) = scratch_universe();
        let cdir = Path::new(&root).join(".constellation");
        std::fs::write(Journal::path_for(&cdir), b"{ not json").unwrap();
        let err = Journal::load(&cdir).unwrap_err();
        assert!(err.contains("NOT deleting"), "{}", err);
    }

    // ── snapshot ──

    fn seed_db(dir: &Path) -> (rusqlite::Connection, PathBuf) {
        let db = dir.join("search.db");
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE note_meta (path TEXT PRIMARY KEY);
             CREATE TABLE note_links (source_path TEXT, weight REAL);
             CREATE TABLE review_schedule (path TEXT PRIMARY KEY);
             INSERT INTO note_meta VALUES ('a.md'), ('b.md');
             INSERT INTO note_links VALUES ('a.md', 1.5), ('a.md', 2.0), ('b.md', 0.5);
             INSERT INTO review_schedule VALUES ('a.md');",
        )
        .unwrap();
        (conn, db)
    }

    #[test]
    fn snapshot_checkpoints_copies_and_verifies() {
        let (_td, root) = scratch_universe();
        let cdir = Path::new(&root).join(".constellation");
        let (conn, db_path) = seed_db(&cdir);

        let (backup, json_backups, baseline) = take_snapshot(&conn, &db_path, &cdir).unwrap();
        assert_eq!(baseline.note_meta_rows, 2);
        assert_eq!(baseline.note_links_rows, 3);
        assert!((baseline.note_links_weight_sum - 4.0).abs() < 1e-9);
        assert_eq!(baseline.review_schedule_rows, 1);
        assert!(Path::new(&backup).exists());
        // No JSON stores existed in this scratch — the list is allowed to be empty.
        assert!(json_backups.is_empty());

        // The backup is complete WITHOUT its WAL: rows written before the snapshot are in
        // the main file (checkpoint TRUNCATE), so a copy taken alone still holds them.
        let check =
            rusqlite::Connection::open_with_flags(&backup, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        let n: i64 = check.query_row("SELECT COUNT(*) FROM note_links", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 3);
    }

    #[test]
    fn snapshot_copies_the_json_stores_it_finds() {
        let (_td, root) = scratch_universe();
        let cdir = Path::new(&root).join(".constellation");
        std::fs::write(cdir.join("libraries.json"), b"[]").unwrap();
        std::fs::write(cdir.join("collections.json"), b"[]").unwrap();
        let (conn, db_path) = seed_db(&cdir);

        let (_b, json_backups, _base) = take_snapshot(&conn, &db_path, &cdir).unwrap();
        let names: Vec<&str> = json_backups.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"libraries.json"));
        assert!(names.contains(&"collections.json"));
        for (_, p) in &json_backups {
            assert!(Path::new(p).exists());
        }
    }
}

// ═══ Slice 2 — move + rewrite + verify ══════════════════════════════════════════════════════

/// Component-wise path remap: if `stored` lies under `old_root` (compared NFC + case- +
/// separator-insensitively), rebuild it as `new_root` + the ORIGINAL raw suffix components.
///
/// Component-wise because NFC can change a component's LENGTH (an NFD-stored Arabic name has
/// more codepoints than its NFC form), so byte-offset prefix slicing is unsound (H3). The
/// suffix components are carried VERBATIM — the row keeps whatever separator/encoding
/// convention it had, only the root prefix changes, so every equality-keyed consumer that
/// found the row before finds it after.
pub(crate) fn remap_path(stored: &str, old_root: &str, new_root: &str) -> Option<String> {
    fn parts(p: &str) -> Vec<&str> {
        p.split(['/', '\\']).filter(|s| !s.is_empty()).collect()
    }
    fn norm_component(c: &str) -> String {
        use unicode_normalization::UnicodeNormalization;
        c.nfc().collect::<String>().to_lowercase()
    }
    let stored_parts = parts(stored);
    let old_parts = parts(old_root);
    if stored_parts.len() < old_parts.len() || old_parts.is_empty() {
        return None;
    }
    for (i, op) in old_parts.iter().enumerate() {
        if norm_component(stored_parts[i]) != norm_component(op) {
            return None;
        }
    }
    let mut out = std::path::PathBuf::from(new_root);
    for part in &stored_parts[old_parts.len()..] {
        out.push(part);
    }
    Some(out.to_string_lossy().to_string())
}

/// First (old_root, new_root) pair that remaps `stored`, applied. None = not under any moved root.
pub(crate) fn remap_any(stored: &str, pairs: &[(String, String)]) -> Option<String> {
    for (old, new) in pairs {
        if let Some(r) = remap_path(stored, old, new) {
            return Some(r);
        }
    }
    None
}

/// The BOOLEAN fast path for "is `stored` under any of these roots?".
///
/// Stage-A timing round 2: the in-tx verify alone took **205 s**, because it answered this
/// yes/no question by attempting the full component-splitting, per-component-NFC
/// `remap_path` against every row x every pair (~68M NFC normalizations across ~470k rows).
/// For a membership test, normalize each PREFIX once and each ROW once, then compare with a
/// separator-bounded starts_with. `remap_path` stays the authority for actual rewriting —
/// which only ever runs on the matched minority.
pub(crate) struct NormPrefixes(Vec<String>);

impl NormPrefixes {
    pub(crate) fn new<'a>(roots: impl Iterator<Item = &'a str>) -> Self {
        NormPrefixes(roots.map(norm).collect())
    }
    pub(crate) fn matches(&self, stored: &str) -> bool {
        let n = norm(stored);
        self.0.iter().any(|r| {
            n == *r
                || (n.len() > r.len()
                    && n.as_bytes()[r.len()] == b'/'
                    && n.starts_with(r.as_str()))
        })
    }
}

// ─── Move phase ─────────────────────────────────────────────────────────────────────────────

/// Execute every not-yet-moved entry's fs operation, journaling around each. Halts with Err
/// on the first failure (open handle, permissions) — the journal names exactly where; a
/// re-run resumes from the first unmoved entry (idempotent: `moved` entries are skipped, and
/// an entry whose new_path already exists while old_path is gone is adopted as moved).
pub fn run_move_phase(journal: &mut Journal, constellation_dir: &Path) -> Result<(), String> {
    journal.phase = Phase::Moving;
    journal.save(constellation_dir)?;

    for i in 0..journal.entries.len() {
        if journal.entries[i].moved {
            continue;
        }
        let (old_p, new_p, action, started, copied) = {
            let e = &journal.entries[i];
            (
                PathBuf::from(&e.old_path),
                PathBuf::from(&e.new_path),
                e.action.clone(),
                e.started,
                e.copied,
            )
        };
        // Phase-4 audit (the BLOCKER class) — adoption never GUESSES completeness:
        //   · a same-volume RENAME is atomic, so `source gone + dest present` proves done;
        //   · copy-based ops prove nothing by existence — a crash mid-copy leaves a partial
        //     dir that is_dir() happily accepts. Their proof is the journaled `copied` flag,
        //     set only after a count-verified copy.
        let same_vol = same_volume(&old_p, &new_p);
        let rename_done = action == "move" && same_vol && !old_p.exists() && new_p.is_dir();
        if rename_done {
            journal.entries[i].moved = true;
            journal.save(constellation_dir)?;
            continue;
        }

        // A destination present before we ever STARTED is a genuine collision — hard error,
        // never deleted. One we DID start (and whose copy never completed) is our own
        // partial: the source is intact and authoritative, so delete it and redo.
        if new_p.exists() && !started {
            return Err(format!(
                "mig108 move: destination already exists (and this entry never started): {}",
                new_p.display()
            ));
        }
        if new_p.exists() && started && !copied && old_p.exists() {
            std::fs::remove_dir_all(&new_p)
                .map_err(|e| format!("mig108: clearing our partial destination failed: {}", e))?;
        }

        journal.entries[i].started = true;
        journal.save(constellation_dir)?;

        if action == "move" && same_vol {
            crate::write_gate::gate_rename(&old_p, &new_p, "mig108_move")
                .map_err(|e| format!("mig108 move {} -> {}: {}", old_p.display(), new_p.display(), e))?;
        } else {
            // Copy-based (copy-class, or cross-volume move).
            if !copied {
                crate::libraries::copy_dir_recursive(&old_p, &new_p)
                    .map_err(|e| format!("mig108 copy {} -> {}: {}", old_p.display(), new_p.display(), e))?;
                // The Architect's promised completeness clause, now real: same file count
                // or the copy did not happen (symlinks are skipped on BOTH sides of the
                // count by the same walker rule, so they cannot skew it — their skipping
                // is logged rather than silent).
                let (src_n, dst_n) = (count_files(&old_p), count_files(&new_p));
                if src_n != dst_n {
                    return Err(format!(
                        "mig108 copy verify: {} files at source, {} at destination for {}",
                        src_n,
                        dst_n,
                        new_p.display()
                    ));
                }
                journal.entries[i].copied = true;
                journal.save(constellation_dir)?;
            }
            if action == "move" {
                // copied=true + old still present = finish the removal; the destination is
                // complete and verified, so this direction is the only safe one.
                if old_p.exists() {
                    std::fs::remove_dir_all(&old_p)
                        .map_err(|e| format!("mig108 cross-volume source removal: {}", e))?;
                }
            }
        }
        journal.entries[i].moved = true;
        journal.save(constellation_dir)?;
    }

    journal.phase = Phase::Moved;
    journal.save(constellation_dir)
}

/// File count for the copy-completeness verify. Skips symlinks/junctions with a LOG LINE —
/// the same rule `copy_dir_recursive` applies (PJ-140 #43), so the two sides of the compare
/// can never disagree about them, and their omission is visible rather than silent.
pub(crate) fn count_files(root: &Path) -> u64 {
    fn inner(dir: &Path, n: &mut u64) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            if e.file_type().map(|t| t.is_symlink()).unwrap_or(false) {
                eprintln!("[mig108] symlink skipped in copy/count: {}", e.path().display());
                continue;
            }
            let p = e.path();
            if p.is_dir() {
                inner(&p, n);
            } else {
                *n += 1;
            }
        }
    }
    let mut n = 0;
    inner(root, &mut n);
    n
}

// ─── DB rewrite phase ───────────────────────────────────────────────────────────────────────

/// The path-bearing tables the straggler sweep patrols after the per-note cascade: rows
/// under an old prefix with no note_meta parent (legacy orphans), or rows the cascade's
/// gates skipped (an unstamped review_schedule). Path-PK tables get a destination
/// pre-delete so the UPDATE can never abort on a phantom.
/// Stage-B failure 2026-08-01 — **this list is the single source of truth for BOTH the
/// repair and the check.** It was not, and that cost a live run.
///
/// The Phase-4 audit (I2) observed that the verify inspected only these five tables while
/// seven more aux tables also carry paths. I widened the VERIFY to all twelve and left the
/// SWEEP at five — so the check could see a stale row the repair could never reach. On the
/// Boss's universe that was 14 orphaned `note_embeddings` rows and 6 `note_body` rows (rows
/// whose parent `note_meta` row is gone, which is precisely what this sweep exists for and
/// precisely what the per-note cascade cannot see). The verify counted 20 stale rows,
/// refused the commit, and rolled back a 45-minute run — correctly, but for a defect I had
/// introduced that morning. A detector without its repair is not a safety feature.
///
/// The verify now iterates THIS list and nothing else, so the two cannot diverge again.
///
/// Third field = the destination pre-delete, needed when the path column is PK/UNIQUE so the
/// UPDATE can never abort on a phantom row already sitting at the new path. (The
/// destination-prefix purge earlier in the transaction normally clears those, so this is the
/// belt to that braces.) `sight_v3_layout`, `note_state_history` and `shape_history` key
/// MANY rows per path — deleting by destination path there would destroy siblings, so they
/// are false by construction, not by omission.
const SWEEP: &[(&str, &str, bool)] = &[
    ("note_links", "source_path", false),
    ("note_aliases", "path", false),
    ("sky_nodes", "path", true),
    ("sky_links", "source_path", false),
    ("review_schedule", "path", true),
    // The seven the audit named — now repaired, not merely detected.
    ("note_embeddings", "path", true),      // path TEXT PRIMARY KEY
    ("note_body", "path", true),            // path TEXT PRIMARY KEY
    ("note_summaries", "path", true),       // path TEXT PRIMARY KEY
    ("sources_suggestions", "note_path", true), // note_path TEXT PRIMARY KEY
    ("sight_v3_layout", "note_path", false),    // composite key — many rows per path
    ("note_state_history", "note_path", false), // history — many rows per path
    ("shape_history", "path", false),           // history — many rows per path
];

/// One transaction: per-note proven cascade + straggler sweep + cache/cursor hygiene +
/// aggregate recompute + hard verification. COMMIT only when every invariant holds;
/// otherwise ROLLBACK and journal `VerifyFailed` (fs moves stand, recorded per entry —
/// the resume path surfaces the state instead of guessing).
pub fn run_db_rewrite(
    conn: &rusqlite::Connection,
    journal: &mut Journal,
    constellation_dir: &Path,
) -> Result<(), String> {
    // Stage-A finding B: COPY entries are included. The index rows key the note's IDENTITY,
    // and the REGISTRATION moves to the root copy — so the rows follow it; the retained
    // original becomes an unregistered plain folder, correctly invisible to the index. The
    // first version filtered copies out ("never indexed under the root" — wrong: they were
    // indexed at their OLD path), which orphaned 70 rows at Stage-A — and because the in-tx
    // verify used this same pair set, it was BLIND to them (LL-040: a verifier built from
    // the decision's own assumptions cannot catch the decision's error; the independent
    // rehearsal check with a wider net did).
    let pairs: Vec<(String, String)> = journal
        .entries
        .iter()
        .map(|e| (e.old_path.clone(), e.new_path.clone()))
        .collect();
    // Phase-4 audit — the SNAPSHOT-time baseline cannot gate the rewrite: legitimate boot
    // work in a crash-resume window (link_life_restore, review self-heal, reconcile
    // re-adoption) changes the counts, and a byte-equality check against a stale number
    // fails FOREVER. The invariant I2 actually means "this TRANSACTION loses nothing" — so
    // the baseline is captured INSIDE the transaction, before the first UPDATE. The
    // snapshot-time numbers remain in the journal for the report and the backup's own
    // verification, which is what they truly certify.
    journal
        .baseline
        .as_ref()
        .ok_or("mig108 db rewrite: journal carries no baseline (snapshot must run first)")?;

    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| format!("mig108 BEGIN failed: {}", e))?;

    let result = (|| -> Result<(), String> {
        conn.execute_batch("PRAGMA defer_foreign_keys = ON")
            .map_err(|e| e.to_string())?;
        // H2 — the ungated per-edge outgoing recompute is O(N^2) across a bulk rewrite
        // (+17 s measured at 216k links).
        crate::search::drop_outgoing_link_triggers(conn)?;

        // Phase-4 audit (BLOCKER 1) — purge DESTINATION-prefix rows before the cascade. In
        // a crash-resume window the boot reconcile can RE-ADOPT the already-moved files as
        // fresh rows at their NEW paths (default weights, no history — recomputable junk by
        // construction). Left in place they collide with the cascade's UPDATEs
        // (UNIQUE(source_path, target_name, link_type) has no pre-delete for note_links),
        // failing the rewrite deterministically on every retry. The REAL rows — the earned
        // ones — still sit at the OLD paths and are about to be remapped in. Purging the
        // destination prefixes is therefore both safe and what makes resume idempotent
        // against a dirty world. (The root library's own notes are under the ROOT but not
        // under any per-library destination dir, so they are untouchable by this.)
        let dest_prefixes = NormPrefixes::new(pairs.iter().map(|(_, n)| n.as_str()));
        let mut purged = 0usize;
        for (table, col) in [
            ("note_meta", "path"),
            ("note_links", "source_path"),
            ("note_aliases", "path"),
            ("note_embeddings", "path"),
            ("note_body", "path"),
            ("review_schedule", "path"),
            ("note_summaries", "path"),
            ("sources_suggestions", "note_path"),
            ("sight_v3_layout", "note_path"),
            ("note_state_history", "note_path"),
            ("shape_history", "path"),
            ("sky_nodes", "path"),
            ("sky_links", "source_path"),
        ] {
            let sql = format!("SELECT DISTINCT {c} FROM {t}", c = col, t = table);
            let stale: Vec<String> = match conn.prepare(&sql) {
                Ok(mut stmt) => stmt
                    .query_map([], |r| r.get::<_, String>(0))
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .filter(|p| dest_prefixes.matches(p))
                    .collect(),
                Err(e) if e.to_string().contains("no such table") => continue,
                Err(e) => return Err(e.to_string()),
            };
            for v in stale {
                conn.execute(&format!("DELETE FROM {t} WHERE {c} = ?1", t = table, c = col), [&v])
                    .map_err(|e| e.to_string())?;
                purged += 1;
            }
        }
        if purged > 0 {
            eprintln!(
                "[mig108] purged {} destination-prefix rows (crash-window re-adoption junk)",
                purged
            );
        }

        // The in-tx baseline is captured AFTER the junk purge, so it describes exactly the
        // set whose conservation this transaction must prove — the earned rows. (Captured
        // before the purge it counts the junk, and equality can never hold on a resume; the
        // re-adoption test caught precisely that ordering flaw in the first version.)
        let baseline = read_baseline(conn)?;
        // Stage-A finding C — the sky triggers came OFF too. The first version kept them
        // ("the proven cascade, exactly as every live single-note move runs it") — true at
        // single-note scale, and ~25 MINUTES at 7,800-notes-in-one-transaction scale: the
        // per-fire enrichment EXISTS probes and per-edge sky_links delete+reinsert multiply.
        // The straggler sweep below rewrites sky_nodes / sky_links / note_aliases directly
        // (proven by the fixture, which runs with these triggers absent). Recreation:
        // idempotent init_db — the command layer calls it post-commit for the live session,
        // and the next boot's init_db covers every crash window (the same self-heal the
        // rehearsal maker already relied on, proven live when the Boss's boot recreated 17).
        conn.execute_batch(
            "DROP TRIGGER IF EXISTS note_meta_sky_au;
             DROP TRIGGER IF EXISTS note_links_sky_ai;
             DROP TRIGGER IF EXISTS note_links_sky_ad;
             DROP TRIGGER IF EXISTS note_links_sky_au;",
        )
        .map_err(|e| format!("mig108: dropping sky triggers failed: {}", e))?;

        // The per-note proven cascade, enumerated Rust-side with normalized matching (H3) —
        // never SQL replace()/LIKE.
        let all_paths: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT path FROM note_meta")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };
        let t_loop = std::time::Instant::now();
        for old in &all_paths {
            if let Some(new) = remap_any(old, &pairs) {
                crate::libraries::migrate_note_db_paths(conn, old, &new);
            }
        }
        eprintln!("[mig108 timing] per-note cascade loop: {:.1}s", t_loop.elapsed().as_secs_f64());

        // Straggler sweep — universal end-state guarantee for rows the cascade cannot see.
        let t_sweep = std::time::Instant::now();
        let old_prefixes = NormPrefixes::new(pairs.iter().map(|(o, _)| o.as_str()));
        for (table, col, pre_delete) in SWEEP {
            let sql = format!("SELECT DISTINCT {c} FROM {t}", c = col, t = table);
            let stale: Vec<String> = match conn.prepare(&sql) {
                Ok(mut stmt) => stmt
                    .query_map([], |r| r.get::<_, String>(0))
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .filter(|p| old_prefixes.matches(p))
                    .collect(),
                // Lazily-created tables may not exist — the correct no-op.
                Err(e) if e.to_string().contains("no such table") => continue,
                Err(e) => return Err(e.to_string()),
            };
            for old in stale {
                let new = remap_any(&old, &pairs).expect("filtered above");
                if *pre_delete {
                    conn.execute(
                        &format!("DELETE FROM {t} WHERE {c} = ?1", t = table, c = col),
                        [&new],
                    )
                    .map_err(|e| e.to_string())?;
                }
                conn.execute(
                    &format!("UPDATE {t} SET {c} = ?2 WHERE {c} = ?1", t = table, c = col),
                    [&old, &new],
                )
                .map_err(|e| e.to_string())?;
            }
        }

        eprintln!("[mig108 timing] straggler sweep: {:.1}s", t_sweep.elapsed().as_secs_f64());

        // H11 hygiene — all tolerant of absent tables.
        let t_h = std::time::Instant::now();
        for sql in [
            "DELETE FROM sky_backfill_cursor",
            "DELETE FROM links_outgoing_backfill_cursor",
            "DELETE FROM review_backfill_cursor",
            "DELETE FROM note_body_backfill_cursor",
            "DELETE FROM sight_v3_layout",
            "DELETE FROM sight_v3_layout_cursor",
            "DELETE FROM sight_v3_graph_version",
            "DELETE FROM sight_v3_density_grid",
            "DELETE FROM link_stats_cache",
        ] {
            if let Err(e) = conn.execute_batch(sql) {
                if !e.to_string().contains("no such table") {
                    return Err(e.to_string());
                }
            }
        }
        eprintln!("[mig108 timing] cache/cursor hygiene: {:.1}s", t_h.elapsed().as_secs_f64());

        // Restore the aggregate machinery, then recompute once (the reconcile precedent).
        let t_rc = std::time::Instant::now();
        crate::search::create_outgoing_link_triggers(conn)?;
        // PJ-207 §6 — through the one assembly. This site ran ONE family by hand while
        // the reconcile tail ran five; the difference was correct (a path rewrite moves
        // rows without changing links or content) but invisible. Now it is named.
        let rep = crate::converge::after_mig108(conn);
        if let Some((_, msg)) = rep.failures().into_iter().next() {
            return Err(msg);
        }
        eprintln!("[mig108 timing] recompute_all_outgoing: {:.1}s", t_rc.elapsed().as_secs_f64());

        // ── Verification (I2) — inside the transaction, before COMMIT ──
        let t_v = std::time::Instant::now();
        let after = read_baseline(conn)?;
        if after != baseline {
            return Err(format!(
                "mig108 verify: aggregates diverged — before {:?}, after {:?}",
                baseline, after
            ));
        }
        // Stage-B failure 2026-08-01 — the verify iterates SWEEP and ONLY SWEEP. A separate
        // VERIFY_EXTRA list used to name seven tables the sweep never patrolled, which made
        // a permanently-unsatisfiable check: it could count stale rows that nothing in the
        // transaction was able to rewrite. Those seven now live in SWEEP itself, so every
        // table this loop checks is a table the sweep just repaired, by construction.
        let mut stale_left = 0i64;
        for (table, col, _) in SWEEP {
            let sql = format!("SELECT {c} FROM {t}", c = col, t = table);
            if let Ok(mut stmt) = conn.prepare(&sql) {
                let rows = stmt
                    .query_map([], |r| r.get::<_, String>(0))
                    .map_err(|e| e.to_string())?;
                stale_left += rows
                    .filter_map(|r| r.ok())
                    .filter(|p| old_prefixes.matches(p))
                    .count() as i64;
            }
        }
        {
            let mut stmt = conn.prepare("SELECT path FROM note_meta").map_err(|e| e.to_string())?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(|e| e.to_string())?;
            stale_left += rows
                .filter_map(|r| r.ok())
                .filter(|p| old_prefixes.matches(p))
                .count() as i64;
        }
        if stale_left != 0 {
            return Err(format!("mig108 verify: {} rows still under an old prefix", stale_left));
        }
        for e in &journal.entries {
            if !Path::new(&e.new_path).is_dir() {
                return Err(format!("mig108 verify: moved dir missing: {}", e.new_path));
            }
        }
        eprintln!("[mig108 timing] in-tx verify: {:.1}s", t_v.elapsed().as_secs_f64());
        Ok(())
    })();

    match result {
        Ok(()) => {
            let t_c = std::time::Instant::now();
            conn.execute_batch("COMMIT").map_err(|e| {
                let _ = conn.execute_batch("ROLLBACK");
                format!("mig108 COMMIT failed (deferred FK?): {}", e)
            })?;
            eprintln!("[mig108 timing] COMMIT: {:.1}s", t_c.elapsed().as_secs_f64());
            journal.last_error = None; // got past the verify — the old reason is history
            journal.phase = Phase::DbRewritten;
            journal.save(constellation_dir)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            journal.phase = Phase::VerifyFailed;
            // Stage-B 2026-08-01 — journal WHY, not just THAT. The live failure recorded only
            // the phase, so the cause had to be reconstructed from the user's data after the
            // fact. eprintln is not a record on a release GUI build: stderr goes nowhere.
            journal.last_error = Some(e.clone());
            eprintln!("[mig108] verify refused the commit — rolled back: {e}");
            journal.save(constellation_dir)?;
            Err(e)
        }
    }
}

// ─── JSON rewrite phase ─────────────────────────────────────────────────────────────────────

/// Deep-remap every string (object KEYS included — `folderTemplates` is keyed by absolute
/// folder path) that lies under a moved root. Uniform on purpose: in these stores, a string
/// under a moved library's old absolute path IS a reference to moved content — one rule,
/// no per-store field list to drift (H12).
pub(crate) fn remap_json_value(v: &mut serde_json::Value, pairs: &[(String, String)]) {
    match v {
        serde_json::Value::String(s) => {
            if let Some(new) = remap_any(s, pairs) {
                *s = new;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                remap_json_value(item, pairs);
            }
        }
        serde_json::Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for k in keys {
                if let Some(new_key) = remap_any(&k, pairs) {
                    if let Some(val) = map.remove(&k) {
                        map.insert(new_key, val);
                    }
                }
            }
            for (_, val) in map.iter_mut() {
                remap_json_value(val, pairs);
            }
        }
        _ => {}
    }
}

/// The stores the rewrite touches (superset of the snapshot list — session.prev.json is the
/// session reader's fallback and must not resurrect old paths).
pub const REWRITE_STORES: &[&str] = &[
    "libraries.json",
    "review-pulse.json",
    "workspaces.json",
    "session.json",
    "session.prev.json",
    "collections.json",
    "settings.json",
    "bookmarks.json",
];

pub fn run_json_rewrites(journal: &mut Journal, constellation_dir: &Path) -> Result<(), String> {
    let pairs: Vec<(String, String)> = journal
        .entries
        .iter()
        .map(|e| (e.old_path.clone(), e.new_path.clone()))
        .collect();

    for store in REWRITE_STORES {
        if journal.json_rewritten.iter().any(|s| s == store) {
            continue; // idempotent resume
        }
        let path = constellation_dir.join(store);
        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .map_err(|e| format!("mig108 {} read: {}", store, e))?;
            let mut v: serde_json::Value = serde_json::from_str(&raw)
                .map_err(|e| format!("mig108 {} parse: {}", store, e))?;
            remap_json_value(&mut v, &pairs);
            let out = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
            crate::universe::atomic_write(&path, out.as_bytes())
                .map_err(|e| format!("mig108 {} write: {}", store, e))?;
        }
        journal.json_rewritten.push(store.to_string());
        journal.save(constellation_dir)?;
    }

    journal.phase = Phase::JsonRewritten;
    journal.save(constellation_dir)
}

// ─── Orchestrator ───────────────────────────────────────────────────────────────────────────

/// Run (or resume) everything after planning: snapshot → moves → DB → JSON. Idempotent per
/// journal state; the caller (Slice 4's command) handles the freeze envelope and the trash
/// consolidation step (Slice 3) around it.
pub fn run_engine(
    conn: &rusqlite::Connection,
    db_path: &Path,
    journal: &mut Journal,
    constellation_dir: &Path,
) -> Result<(), String> {
    if matches!(journal.phase, Phase::Planned) {
        let (db_backup, json_backups, baseline) = take_snapshot(conn, db_path, constellation_dir)?;
        journal.snapshot_db = Some(db_backup);
        journal.json_backups = json_backups;
        journal.baseline = Some(baseline);
        journal.phase = Phase::Snapshotted;
        journal.save(constellation_dir)?;
    }
    if matches!(journal.phase, Phase::Snapshotted | Phase::Moving) {
        run_move_phase(journal, constellation_dir)?;
    }
    if matches!(journal.phase, Phase::Moved | Phase::VerifyFailed) {
        run_db_rewrite(conn, journal, constellation_dir)?;
    }
    if matches!(journal.phase, Phase::DbRewritten) {
        run_json_rewrites(journal, constellation_dir)?;
    }
    if matches!(journal.phase, Phase::JsonRewritten) {
        finish_with_trash_consolidation(journal, constellation_dir)?;
    }
    Ok(())
}

/// T step + Done — consolidate per-library trash into the root's (idempotent, so a resume
/// re-entering here is safe). The registry was just rewritten, so its paths are the NEW
/// locations; a missing/unreadable registry skips consolidation rather than guessing
/// (nothing is lost — the standalone pass can run any time).
fn finish_with_trash_consolidation(
    journal: &mut Journal,
    constellation_dir: &Path,
) -> Result<(), String> {
    let libs_file = constellation_dir.join("libraries.json");
    if let Ok(raw) = std::fs::read_to_string(&libs_file) {
        if let Ok(libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&raw) {
            let paths: Vec<String> = libs.iter().map(|l| l.path.clone()).collect();
            consolidate_trash(Path::new(&journal.universe_root), &paths)?;
        }
    }
    journal.phase = Phase::Done;
    journal.save(constellation_dir)
}

#[cfg(test)]
mod slice2_tests {
    use super::*;
    use crate::libraries::LibraryInfo;

    fn lib(id: &str, name: &str, path: &str) -> LibraryInfo {
        LibraryInfo {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            is_universe_notes: false,
            canonical_mode: "compatible".into(),
        }
    }

    // ── remap_path ──

    #[test]
    fn remap_preserves_the_raw_suffix_and_matches_across_conventions() {
        // Backslash-stored row, forward-slash root — matches, suffix verbatim.
        let got = remap_path("E:\\Old Tree\\Lib\\Sub\\Note.md", "E:/Old Tree/Lib", "E:/Root/Lib").unwrap();
        assert_eq!(norm(&got), norm("E:/Root/Lib/Sub/Note.md"));
        // Case-insensitive on the prefix.
        assert!(remap_path("e:/old tree/lib/n.md", "E:/Old Tree/Lib", "E:/R").is_some());
        // NOT under → None; sibling prefix must not match (separator-bounded by components).
        assert!(remap_path("E:/Old Tree/Library2/n.md", "E:/Old Tree/Lib", "E:/R").is_none());
    }

    #[test]
    fn remap_is_nfc_safe_on_arabic_components() {
        // NFD-stored component vs NFC root: same text, different codepoint counts.
        let nfc = "\u{0623}\u{062f}\u{0628}"; // أدب (NFC: U+0623 = hamza-on-alef precomposed)
        let nfd = "\u{0627}\u{0654}\u{062f}\u{0628}"; // alef + combining hamza + د + ب
        let stored = format!("E:/Tree/{}/Note.md", nfd);
        let root = format!("E:/Tree/{}", nfc);
        let got = remap_path(&stored, &root, "E:/Root/Adab").unwrap();
        assert_eq!(norm(&got), norm("E:/Root/Adab/Note.md"));
    }

    // ── the end-to-end fixture ──

    struct Fixture {
        _td: tempfile::TempDir,
        root: String,
        cdir: PathBuf,
        conn: rusqlite::Connection,
        db_path: PathBuf,
        ext_a: String, // moving external (ASCII)
        ext_b: String, // moving external (Arabic name)
        book: String,  // copy-class
    }

    fn build_fixture() -> Fixture {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("Universe");
        let cdir = root.join(".constellation");
        std::fs::create_dir_all(&cdir).unwrap();

        let ext_a = td.path().join("Tree").join("LibA");
        let ext_b = td.path().join("Tree").join("مكتبة عربية");
        let book = td.path().join("Repo").join("Book");
        for d in [&ext_a, &ext_b, &book] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::write(ext_a.join("a.md"), "alpha").unwrap();
        std::fs::write(ext_b.join("b.md"), "beta").unwrap();
        std::fs::write(book.join("k.md"), "kappa").unwrap();

        let db_path = cdir.join("search.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
             CREATE TABLE note_meta (
                 path TEXT PRIMARY KEY, name TEXT,
                 outgoing_count INTEGER DEFAULT 0, outgoing_link_types TEXT DEFAULT '',
                 outgoing_link_types_json TEXT DEFAULT '{}', outgoing_top_rank INTEGER DEFAULT 99
             );
             CREATE TABLE note_links (
                 source_path TEXT, target_name TEXT, link_type TEXT DEFAULT 'associative',
                 status TEXT DEFAULT 'active', weight REAL,
                 UNIQUE(source_path, target_name, link_type)
             );
             CREATE TABLE note_aliases (path TEXT, alias_lower TEXT, PRIMARY KEY (path, alias_lower));
             CREATE TABLE note_embeddings (path TEXT PRIMARY KEY);
             CREATE TABLE note_body (path TEXT PRIMARY KEY, body TEXT);
             CREATE TABLE review_schedule (path TEXT PRIMARY KEY, last_reviewed TEXT);
             CREATE TABLE note_summaries (
                 path TEXT PRIMARY KEY, summary TEXT,
                 FOREIGN KEY (path) REFERENCES note_meta(path) ON DELETE CASCADE
             );
             CREATE TABLE sources_suggestions (note_path TEXT PRIMARY KEY);
             CREATE TABLE sight_v3_layout (note_path TEXT);
             CREATE TABLE note_state_history (id INTEGER PRIMARY KEY AUTOINCREMENT, note_path TEXT);
             CREATE TABLE sky_nodes (path TEXT PRIMARY KEY);
             CREATE TABLE sky_links (source_path TEXT, target_name TEXT, link_type TEXT,
                 UNIQUE(source_path, target_name, link_type));",
        )
        .unwrap();

        // Seed: one note per moving library. LibA's row uses BACKSLASH separators; the Arabic
        // library's row is stored in a DIFFERENT normalization than the fs path to prove H3.
        let a_note = format!("{}\\a.md", ext_a.to_string_lossy().replace('/', "\\"));
        let b_note = format!("{}/b.md", ext_b.to_string_lossy());
        conn.execute("INSERT INTO note_meta (path, name) VALUES (?1, 'a')", [&a_note]).unwrap();
        conn.execute("INSERT INTO note_meta (path, name) VALUES (?1, 'b')", [&b_note]).unwrap();
        conn.execute(
            "INSERT INTO note_links (source_path, target_name, weight) VALUES (?1, 'b', 1.5), (?1, 'c', 2.5)",
            [&a_note],
        )
        .unwrap();
        conn.execute("INSERT INTO note_aliases VALUES (?1, 'alias-a')", [&a_note]).unwrap();
        conn.execute("INSERT INTO review_schedule VALUES (?1, '2026-07-01')", [&a_note]).unwrap();
        // The FK child that REFUSES a bare parent UPDATE without deferral (H1).
        conn.execute("INSERT INTO note_summaries VALUES (?1, 'sum')", [&a_note]).unwrap();
        conn.execute("INSERT INTO sky_nodes VALUES (?1)", [&a_note]).unwrap();
        conn.execute("INSERT INTO sky_links VALUES (?1, 'b', 'associative')", [&a_note]).unwrap();
        // Stage-A finding B — the COPY-class library's note was indexed at its OLD path too;
        // its rows must follow the registration to the root copy.
        let k_note = format!("{}/k.md", book.to_string_lossy());
        conn.execute("INSERT INTO note_meta (path, name) VALUES (?1, 'k')", [&k_note]).unwrap();
        conn.execute("INSERT INTO review_schedule VALUES (?1, '2026-07-02')", [&k_note]).unwrap();

        // JSON stores.
        std::fs::write(
            cdir.join("libraries.json"),
            serde_json::to_string_pretty(&vec![
                lib("u", "Universe", &root.to_string_lossy()),
                lib("a", "LibA", &ext_a.to_string_lossy()),
                lib("b", "Arabic", &ext_b.to_string_lossy()),
                lib("k", "Book", &book.to_string_lossy()),
            ])
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            cdir.join("review-pulse.json"),
            serde_json::json!({
                "last_reviewed": { a_note.clone(): "2026-07-01" },
                "snoozed": {}, "intervals": { a_note.clone(): 4 }, "dismissed": [b_note.clone()]
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            cdir.join("workspaces.json"),
            serde_json::json!([{ "name": "W", "tabs": [{ "path": a_note }], "activeTabPath": a_note }])
                .to_string(),
        )
        .unwrap();
        std::fs::write(
            cdir.join("settings.json"),
            serde_json::json!({
                "folderTemplates": { format!("{}/Ideas", ext_a.to_string_lossy()): "Idea" },
                "trashDestination": "local"
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            cdir.join("collections.json"),
            serde_json::json!([{ "id": "starred", "items": [{ "type": "note", "path": b_note }] }])
                .to_string(),
        )
        .unwrap();

        Fixture {
            _td: td,
            root: root.to_string_lossy().to_string(),
            cdir,
            conn,
            db_path,
            ext_a: ext_a.to_string_lossy().to_string(),
            ext_b: ext_b.to_string_lossy().to_string(),
            book: book.to_string_lossy().to_string(),
        }
    }

    fn plan(f: &Fixture) -> Journal {
        let libs = vec![
            lib("u", "Universe", &f.root),
            lib("a", "LibA", &f.ext_a),
            lib("b", "Arabic", &f.ext_b),
            lib("k", "Book", &f.book),
        ];
        let report = classify(&f.root, &libs, &[f.book.clone()], &[]);
        Journal::new(&f.root, &report)
    }

    #[test]
    fn end_to_end_moves_rewrites_and_verifies() {
        let f = build_fixture();
        let mut j = plan(&f);
        j.save(&f.cdir).unwrap();

        run_engine(&f.conn, &f.db_path, &mut j, &f.cdir).unwrap();
        assert_eq!(j.phase, Phase::Done);

        // fs — moves moved, copy-class original retained.
        assert!(Path::new(&f.root).join("LibA").join("a.md").is_file());
        assert!(Path::new(&f.root).join("مكتبة عربية").join("b.md").is_file());
        assert!(!Path::new(&f.ext_a).exists(), "moved source gone");
        assert!(Path::new(&f.book).join("k.md").is_file(), "copy-class original untouched");
        assert!(Path::new(&f.root).join("Book").join("k.md").is_file(), "copy landed");

        // db — nothing under old prefixes; earned data intact; FK child moved with parent.
        let count = |sql: &str| -> i64 { f.conn.query_row(sql, [], |r| r.get(0)).unwrap() };
        assert_eq!(count("SELECT COUNT(*) FROM note_meta"), 3);
        let weight: f64 = f
            .conn
            .query_row("SELECT SUM(weight) FROM note_links", [], |r| r.get(0))
            .unwrap();
        assert!((weight - 4.0).abs() < 1e-9);
        let paths: Vec<String> = {
            let mut st = f.conn.prepare("SELECT path FROM note_meta").unwrap();
            let v = st.query_map([], |r| r.get(0)).unwrap().filter_map(|r| r.ok()).collect();
            v
        };
        for p in &paths {
            assert!(
                norm_under(p, &f.root),
                "every note_meta row lives under the root now: {}",
                p
            );
        }
        let sum_path: String = f
            .conn
            .query_row("SELECT path FROM note_summaries", [], |r| r.get(0))
            .unwrap();
        assert!(norm_under(&sum_path, &f.root), "FK child moved with its parent");
        let sky: String = f.conn.query_row("SELECT path FROM sky_nodes", [], |r| r.get(0)).unwrap();
        assert!(norm_under(&sky, &f.root));
        let scheds: Vec<String> = {
            let mut st = f.conn.prepare("SELECT path FROM review_schedule").unwrap();
            let v = st.query_map([], |r| r.get(0)).unwrap().filter_map(|r| r.ok()).collect();
            v
        };
        for p in &scheds {
            assert!(norm_under(p, &f.root), "unstamped review rows caught by the sweep: {}", p);
        }
        // Stage-A finding B — the copy-class note's rows moved to the ROOT COPY's path,
        // even though the original files remain on disk at the old location.
        let k_now: String = f
            .conn
            .query_row("SELECT path FROM note_meta WHERE name = 'k'", [], |r| r.get(0))
            .unwrap();
        assert!(
            norm_under(&k_now, &f.root),
            "copy-class rows follow the REGISTRATION, not the retained original: {}",
            k_now
        );

        // json — keys AND values remapped in every store.
        let libs_txt = std::fs::read_to_string(f.cdir.join("libraries.json")).unwrap();
        assert!(!libs_txt.contains(&f.ext_a.replace('\\', "\\\\")), "libraries.json re-pointed");
        let settings: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(f.cdir.join("settings.json")).unwrap()).unwrap();
        let ft = settings["folderTemplates"].as_object().unwrap();
        assert_eq!(ft.len(), 1);
        assert!(
            norm_under(ft.keys().next().unwrap(), &f.root),
            "folderTemplates KEY remapped: {:?}",
            ft.keys().next()
        );
        let pulse: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(f.cdir.join("review-pulse.json")).unwrap()).unwrap();
        assert!(norm_under(pulse["last_reviewed"].as_object().unwrap().keys().next().unwrap(), &f.root));
        assert!(norm_under(pulse["dismissed"][0].as_str().unwrap(), &f.root));
        let ws: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(f.cdir.join("workspaces.json")).unwrap()).unwrap();
        assert!(norm_under(ws[0]["tabs"][0]["path"].as_str().unwrap(), &f.root));
        let coll: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(f.cdir.join("collections.json")).unwrap()).unwrap();
        assert!(norm_under(coll[0]["items"][0]["path"].as_str().unwrap(), &f.root));

        // The snapshot still holds the PRE-move truth.
        let backup = j.snapshot_db.as_ref().unwrap();
        let check = rusqlite::Connection::open_with_flags(
            backup,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        let old_paths: Vec<String> = {
            let mut st = check.prepare("SELECT path FROM note_meta").unwrap();
            let v = st.query_map([], |r| r.get(0)).unwrap().filter_map(|r| r.ok()).collect();
            v
        };
        assert!(old_paths.iter().any(|p| !norm_under(p, &f.root)), "backup keeps old paths");
    }

    #[test]
    fn interrupt_after_moves_resumes_to_done() {
        let f = build_fixture();
        let mut j = plan(&f);
        j.save(&f.cdir).unwrap();

        // Phase 1 run: snapshot + moves only (simulated crash before the DB rewrite).
        let (db_backup, json_backups, baseline) = take_snapshot(&f.conn, &f.db_path, &f.cdir).unwrap();
        j.snapshot_db = Some(db_backup);
        j.json_backups = json_backups;
        j.baseline = Some(baseline);
        j.phase = Phase::Snapshotted;
        j.save(&f.cdir).unwrap();
        run_move_phase(&mut j, &f.cdir).unwrap();
        drop(j); // "crash"

        // Boot: reload from disk, resume.
        let mut j2 = Journal::load(&f.cdir).unwrap().expect("journal survives");
        assert!(j2.is_unfinished());
        assert_eq!(j2.phase, Phase::Moved);
        run_engine(&f.conn, &f.db_path, &mut j2, &f.cdir).unwrap();
        assert_eq!(j2.phase, Phase::Done);

        let p: String = f.conn.query_row("SELECT path FROM note_summaries", [], |r| r.get(0)).unwrap();
        assert!(norm_under(&p, &f.root));
    }

    /// RED — a genuine verify failure (a moved directory missing at COMMIT time) must ROLL
    /// BACK the whole rewrite and journal VerifyFailed, leaving the DB byte-untouched.
    #[test]
    fn verify_failure_rolls_back_and_journals() {
        let f = build_fixture();
        let mut j = plan(&f);
        j.save(&f.cdir).unwrap();

        let (db_backup, json_backups, baseline) = take_snapshot(&f.conn, &f.db_path, &f.cdir).unwrap();
        j.snapshot_db = Some(db_backup);
        j.json_backups = json_backups;
        j.baseline = Some(baseline);
        j.phase = Phase::Snapshotted;
        j.save(&f.cdir).unwrap();
        run_move_phase(&mut j, &f.cdir).unwrap();

        // Sabotage the WORLD, not the bookkeeping: a moved destination vanishes.
        std::fs::remove_dir_all(Path::new(&f.root).join("LibA")).unwrap();

        let err = run_db_rewrite(&f.conn, &mut j, &f.cdir).unwrap_err();
        assert!(err.contains("moved dir missing"), "{}", err);
        assert_eq!(j.phase, Phase::VerifyFailed);

        let paths: Vec<String> = {
            let mut st = f.conn.prepare("SELECT path FROM note_meta").unwrap();
            let v = st.query_map([], |r| r.get(0)).unwrap().filter_map(|r| r.ok()).collect();
            v
        };
        assert!(paths.iter().all(|p| !norm_under(p, &f.root)), "db untouched after rollback");
    }

    /// Phase-4 audit (HIGH) — a STALE snapshot-time baseline must NOT wedge resume: boot
    /// healers legitimately write to the DB in a crash window, so the verify's baseline is
    /// captured IN the transaction. The old behaviour (byte-equality against snapshot time)
    /// failed forever after any such write.
    #[test]
    fn stale_snapshot_baseline_does_not_wedge_resume() {
        let f = build_fixture();
        let mut j = plan(&f);
        j.save(&f.cdir).unwrap();

        let (db_backup, json_backups, baseline) = take_snapshot(&f.conn, &f.db_path, &f.cdir).unwrap();
        j.snapshot_db = Some(db_backup);
        j.json_backups = json_backups;
        j.baseline = Some(baseline);
        j.phase = Phase::Snapshotted;
        j.save(&f.cdir).unwrap();
        run_move_phase(&mut j, &f.cdir).unwrap();

        // The crash-window healer: a row appears AFTER the snapshot, BEFORE the rewrite.
        let a_note = {
            let mut st = f.conn.prepare("SELECT path FROM note_meta LIMIT 1").unwrap();
            let v: String = st.query_map([], |r| r.get(0)).unwrap().next().unwrap().unwrap();
            v
        };
        f.conn
            .execute(
                "INSERT INTO note_links (source_path, target_name, weight) VALUES (?1, 'healer', 9.9)",
                [&a_note],
            )
            .unwrap();

        run_db_rewrite(&f.conn, &mut j, &f.cdir).unwrap();
        assert_eq!(j.phase, Phase::DbRewritten);
        // …and the healer's row was preserved and remapped with everything else.
        let n: i64 = f
            .conn
            .query_row("SELECT COUNT(*) FROM note_links WHERE target_name='healer'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    /// Phase-4 audit (BLOCKER 1) — reconcile can RE-ADOPT moved files as fresh rows at their
    /// NEW paths in the crash window. Those rows collide with the cascade's UPDATEs and used
    /// to fail the rewrite deterministically on every retry. The destination-prefix purge
    /// deletes the junk (recomputable by construction) and the EARNED rows win.
    #[test]
    fn crash_window_readoption_junk_is_purged_and_earned_rows_win() {
        let f = build_fixture();
        let mut j = plan(&f);
        j.save(&f.cdir).unwrap();

        let (db_backup, json_backups, baseline) = take_snapshot(&f.conn, &f.db_path, &f.cdir).unwrap();
        j.snapshot_db = Some(db_backup);
        j.json_backups = json_backups;
        j.baseline = Some(baseline);
        j.phase = Phase::Snapshotted;
        j.save(&f.cdir).unwrap();
        run_move_phase(&mut j, &f.cdir).unwrap();

        // Reconcile's fresh adoption at the NEW path: same identity, DEFAULT weight, and the
        // exact UNIQUE key the cascade's UPDATE will try to move the earned row onto.
        let new_a = Path::new(&f.root).join("LibA").join("a.md").to_string_lossy().to_string();
        f.conn.execute("INSERT INTO note_meta (path, name) VALUES (?1, 'a')", [&new_a]).unwrap();
        f.conn
            .execute(
                "INSERT INTO note_links (source_path, target_name, link_type, weight) VALUES (?1, 'b', 'associative', 1.0)",
                [&new_a],
            )
            .unwrap();

        run_db_rewrite(&f.conn, &mut j, &f.cdir).unwrap();
        assert_eq!(j.phase, Phase::DbRewritten);

        // The earned row (weight 1.5) survived at the new path; the junk (1.0) is gone.
        let w: f64 = f
            .conn
            .query_row(
                "SELECT weight FROM note_links WHERE source_path = ?1 AND target_name='b' AND link_type='associative'",
                [&new_a],
                |r| r.get(0),
            )
            .unwrap();
        assert!((w - 1.5).abs() < 1e-9, "the EARNED weight won, not the re-adopted default: {}", w);
    }

    /// Phase-4 audit (BLOCKER 3) — a partial copy-class copy must NEVER be adopted as done:
    /// resume deletes the partial (its source is the authority) and recopies, count-verified.
    #[test]
    fn partial_copy_is_deleted_and_redone_never_adopted() {
        let f = build_fixture();
        let mut j = plan(&f);
        j.save(&f.cdir).unwrap();
        let (db_backup, json_backups, baseline) = take_snapshot(&f.conn, &f.db_path, &f.cdir).unwrap();
        j.snapshot_db = Some(db_backup);
        j.json_backups = json_backups;
        j.baseline = Some(baseline);
        j.phase = Phase::Snapshotted;
        j.save(&f.cdir).unwrap();

        // Simulate the crash: the copy STARTED (journaled) and produced a partial dest —
        // a directory that exists but is missing the file.
        let idx = j.entries.iter().position(|e| e.action == "copy").expect("copy entry");
        let dest = PathBuf::from(&j.entries[idx].new_path);
        std::fs::create_dir_all(&dest).unwrap(); // partial: dir exists, k.md absent
        j.entries[idx].started = true;
        j.phase = Phase::Moving;
        j.save(&f.cdir).unwrap();

        run_move_phase(&mut j, &f.cdir).unwrap();
        assert!(dest.join("k.md").is_file(), "the partial was deleted and the copy REDONE");
        assert!(
            Path::new(&f.book).join("k.md").is_file(),
            "copy-class source remains untouched"
        );
        // A destination that was NEVER started stays a hard error (genuine collision).
    }

    /// **Stage-B failure 2026-08-01 — the live run that rolled back after 45 minutes.**
    ///
    /// The per-note cascade rewrites aux rows by walking `note_meta`, so a row whose parent
    /// note row is GONE is invisible to it — that is exactly what the straggler sweep is for.
    /// The Phase-4 audit had me widen the VERIFY to seven more aux tables without widening
    /// the SWEEP, producing a check that could count stale rows nothing could repair. On the
    /// Boss's universe: 14 orphaned `note_embeddings` rows + 6 `note_body` rows → 20 stale →
    /// COMMIT refused → full rollback, deterministically, on every retry.
    ///
    /// RED-proof: revert `SWEEP` to its five original entries and this test fails with
    /// "stale rows under an old prefix remain" — the exact live failure, in 20 milliseconds
    /// instead of 45 minutes.
    #[test]
    fn orphaned_aux_rows_are_rewritten_by_the_sweep_not_merely_detected() {
        let f = build_fixture();

        // Orphans: rows under a moving library whose note_meta parent does not exist. Real
        // universes accumulate these (a note deleted while its embedding row outlived it).
        let ghost_a = Path::new(&f.ext_a).join("ghost.md").to_string_lossy().to_string();
        let ghost_b = Path::new(&f.ext_a).join("ghost2.md").to_string_lossy().to_string();
        f.conn
            .execute("INSERT INTO note_embeddings (path) VALUES (?1)", [&ghost_a])
            .unwrap();
        f.conn
            .execute("INSERT INTO note_body (path, body) VALUES (?1, 'orphan')", [&ghost_b])
            .unwrap();
        assert_eq!(
            f.conn
                .query_row("SELECT COUNT(*) FROM note_meta WHERE path IN (?1, ?2)", [&ghost_a, &ghost_b], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0,
            "the fixture's orphans must genuinely have no parent note row"
        );

        let mut j = plan(&f);
        j.save(&f.cdir).unwrap();
        let (db_backup, json_backups, baseline) =
            take_snapshot(&f.conn, &f.db_path, &f.cdir).unwrap();
        j.snapshot_db = Some(db_backup);
        j.json_backups = json_backups;
        j.baseline = Some(baseline);
        j.phase = Phase::Snapshotted;
        j.save(&f.cdir).unwrap();
        run_move_phase(&mut j, &f.cdir).unwrap();

        // The whole point: this must SUCCEED. Before the fix it returned
        // "stale rows under an old prefix remain" and rolled back.
        run_db_rewrite(&f.conn, &mut j, &f.cdir).expect("orphans must be repairable, not just detectable");
        assert_eq!(j.phase, Phase::DbRewritten);

        // …and the orphans followed their library to the new root.
        let new_a = Path::new(&f.root).join("LibA").join("ghost.md").to_string_lossy().to_string();
        let new_b = Path::new(&f.root).join("LibA").join("ghost2.md").to_string_lossy().to_string();
        assert_eq!(
            f.conn
                .query_row("SELECT COUNT(*) FROM note_embeddings WHERE path = ?1", [&new_a], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1,
            "orphaned embedding row rewritten to the new path"
        );
        assert_eq!(
            f.conn
                .query_row("SELECT COUNT(*) FROM note_body WHERE path = ?1", [&new_b], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1,
            "orphaned body row rewritten to the new path"
        );
    }

    /// Phase-4 audit (HIGH) — the promised restore, proven: run to Moved, put everything
    /// back, and the world is byte-identical (fs at old paths, journal gone, DB untouched).
    #[test]
    fn restore_from_moved_puts_everything_back() {
        let f = build_fixture();
        let mut j = plan(&f);
        j.save(&f.cdir).unwrap();
        let (db_backup, json_backups, baseline) = take_snapshot(&f.conn, &f.db_path, &f.cdir).unwrap();
        j.snapshot_db = Some(db_backup);
        j.json_backups = json_backups;
        j.baseline = Some(baseline);
        j.phase = Phase::Snapshotted;
        j.save(&f.cdir).unwrap();
        run_move_phase(&mut j, &f.cdir).unwrap();
        assert_eq!(j.phase, Phase::Moved);

        // The restore body (the command minus AppHandle): reverse each entry, discard.
        for e in j.entries.iter().rev() {
            let old_p = Path::new(&e.old_path);
            let new_p = Path::new(&e.new_path);
            match e.action.as_str() {
                "copy" => {
                    if new_p.exists() {
                        std::fs::remove_dir_all(new_p).unwrap();
                    }
                }
                _ => {
                    if new_p.is_dir() && !old_p.exists() {
                        std::fs::rename(new_p, old_p).unwrap();
                    }
                }
            }
        }
        Journal::discard(&f.cdir).unwrap();

        assert!(Path::new(&f.ext_a).join("a.md").is_file(), "moved library back at its old path");
        assert!(Path::new(&f.ext_b).join("b.md").is_file());
        assert!(!Path::new(&f.root).join("LibA").exists(), "no residue at the root");
        assert!(!Path::new(&f.root).join("Book").exists(), "the copy was removed; source intact");
        assert!(Path::new(&f.book).join("k.md").is_file());
        assert!(Journal::load(&f.cdir).unwrap().is_none(), "journal discarded");
    }
}

// ═══ Slice 3 — trash consolidation ══════════════════════════════════════════════════════════

/// Move every top-level entry of each `<library>/.trash` into `<root>/.trash`, de-colliding
/// via the shared helper (names CAN clash across libraries and with suffixed names already at
/// the destination); remove each emptied source `.trash`. Idempotent — a universe with no
/// per-library trash is a no-op — and standalone: it also serves universes that never needed
/// relocation (the pre-MIG-108 scope setting left per-library trash behind).
///
/// Entries are moved as UNITS (a trashed FOLDER moves whole, with any attachments inside);
/// nothing is read or rewritten — dot-paths are invisible to the index by construction, so
/// no DB work accompanies this.
pub fn consolidate_trash(universe_root: &Path, library_paths: &[String]) -> Result<usize, String> {
    let root_trash = universe_root.join(".trash");
    let mut moved = 0usize;

    for lib in library_paths {
        let lib_path = Path::new(lib);
        // The root's own .trash is the destination, not a source.
        if norm(&lib_path.to_string_lossy()) == norm(&universe_root.to_string_lossy()) {
            continue;
        }
        let src_trash = lib_path.join(".trash");
        if !src_trash.is_dir() {
            continue;
        }
        if !root_trash.exists() {
            std::fs::create_dir_all(&root_trash).map_err(|e| e.to_string())?;
        }
        let entries: Vec<PathBuf> = std::fs::read_dir(&src_trash)
            .map_err(|e| format!("consolidate: read {} failed: {}", src_trash.display(), e))?
            .flatten()
            .map(|e| e.path())
            .collect();
        for entry in entries {
            match crate::libraries::trash_move_decolliding(&entry, &root_trash, "mig108_trash")? {
                crate::libraries::TrashMoveOutcome::Moved(_) => moved += 1,
                crate::libraries::TrashMoveOutcome::NotRenamed { error, .. } => {
                    // Cross-volume leftovers: copy+remove, never overwriting (fresh name).
                    let dest = crate::libraries::free_trash_name(
                        &entry,
                        &root_trash,
                        &root_trash.join(entry.file_name().ok_or("nameless trash entry")?),
                    )?;
                    if entry.is_dir() {
                        crate::libraries::copy_dir_recursive(&entry, &dest)?;
                        std::fs::remove_dir_all(&entry).map_err(|e| e.to_string())?;
                    } else {
                        std::fs::copy(&entry, &dest)
                            .map_err(|e| format!("consolidate fallback ({}): {} — rename had said: {}", entry.display(), e, error))?;
                        std::fs::remove_file(&entry).map_err(|e| e.to_string())?;
                    }
                    moved += 1;
                }
            }
        }
        // Source .trash is empty now — remove it so nothing recreates the two-trash state.
        if std::fs::read_dir(&src_trash).map(|mut d| d.next().is_none()).unwrap_or(false) {
            let _ = std::fs::remove_dir(&src_trash);
        }
    }
    Ok(moved)
}

// ─── Slice 4 — the command surface ──────────────────────────────────────────────────────────
//
// Thin wrappers; every decision lives in the tested engine above. The PROPOSAL is the
// contract (The Constellation Way): `mig108_preflight` is read-only and feeds the dialog;
// nothing mutates until the user's explicit `mig108_execute`, whose `copy_paths` carries the
// per-entry Move/Copy choices the dialog collected (Boss D2/D3 — entries default to Move and
// any can be flipped to Copy).

#[derive(Serialize)]
pub struct JournalState {
    pub phase: Phase,
    pub entries_total: usize,
    pub entries_moved: usize,
    pub universe_root: String,
    /// True when restore (put everything back) is a valid direction for this phase.
    pub restorable: bool,
    /// Why the last attempt stopped, if it did — shown in the resume card.
    pub last_error: Option<String>,
}

fn active_cdir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    crate::universe::active_constellation_dir(app)
}

/// The set of roots this migration must treat as **foreign** — other universes' content,
/// which it may never relocate.
///
/// **Fallible since PJ-322 (Boss decision 1, 2026-08-20).** Both readers underneath used to
/// answer "nothing" for "I could not look": `registered_universe_roots` via the lenient
/// `load_registry`, and `resolve_child_universe_roots_recursive` via the lenient child reader.
/// An empty answer here does not fail loudly — it makes `foreign_reason` return `None` for
/// everything, and every external registered library falls through `classify` to
/// `EntryClass::Move`. **A plan that moves directories must refuse rather than guess.**
///
/// This is the *reported* half of the guard. `classify` also asks the disk directly
/// (`universe_manifest_at_or_above`), which is what still holds if a reader degrades in a way
/// neither of these catches. Two independent guards on one concern, deliberately.
/// Carry the *kind* of failure across the IPC boundary, which is `Result<_, String>`.
///
/// **Found by the panel, 2026-08-20, in code written the same morning.** `universe.rs`
/// deliberately preserves `PersistedError::{Unreadable, Corrupt}` — with a comment explaining
/// that `Unreadable` is usually a transient lock where a retry succeeds, while `Corrupt` is
/// permanent and retrying is pointless — and then this function threw that distinction away one
/// call later with `.map_err(|e| e.message().to_string())`. The unify dialog consequently told
/// the user "this is usually temporary… Try again" about a file that will never repair itself.
///
/// A machine-readable prefix, not English prose: the frontend must not have to pattern-match a
/// translated sentence to decide which buttons are honest. The prefix is stripped before display
/// (`Mig108UnifyDialog.svelte`), so the user never sees it.
pub(crate) const REFUSAL_TRANSIENT: &str = "transient|";
pub(crate) const REFUSAL_DAMAGED: &str = "damaged|";

fn classify_refusal(e: crate::universe::PersistedError) -> String {
    match e {
        crate::universe::PersistedError::Unreadable(m) => format!("{REFUSAL_TRANSIENT}{m}"),
        crate::universe::PersistedError::Corrupt(m) => format!("{REFUSAL_DAMAGED}{m}"),
    }
}

/// **Safety inspection 2026-08-22 (MED, content-loss): a failed copy used to leave its debris
/// under the universe root — which IS a library.**
///
/// `copy_dir_recursive` propagates with `?` partway through (disk full, MAX_PATH on a deep
/// attachment tree, a permission-denied subfolder). The dialog then showed "Failed to copy file:
/// …" and the user reasonably concluded nothing had happened — while several hundred valid `.md`
/// files sat under the root, where the indexer and the watcher pick them up as real notes. A retry
/// de-collides to "Name 2", so the orphan persists as a duplicate set, indexed and searchable.
///
/// Best-effort cleanup: if the copy fails, remove what it wrote before returning the error, and
/// say in the message whether that succeeded. Never silently.
fn copy_or_clean_up(src: &Path, dest: &Path) -> Result<(), String> {
    match crate::libraries::copy_dir_recursive(src, dest) {
        Ok(()) => Ok(()),
        Err(e) => {
            let cleaned = std::fs::remove_dir_all(dest).is_ok();
            Err(if cleaned {
                format!("{e} — the partial copy was removed, so nothing was added to your universe.")
            } else {
                format!(
                    "{e} — and the partial copy at {} could NOT be removed. Delete that folder \
                     yourself before retrying, or its files will be indexed as real notes.",
                    dest.display()
                )
            })
        }
    }
}

/// The first symlink/junction anywhere under `root`, if any.
///
/// A reparse point is the one thing `copy_dir_recursive` drops **silently and by design** (it must
/// not follow one — a junction cycle recurses unboundedly). That makes its presence, not a file
/// count, the honest test of "was this copy complete?".
fn first_reparse_point(root: &Path) -> Option<std::path::PathBuf> {
    let rd = std::fs::read_dir(root).ok()?;
    for e in rd.flatten() {
        if e.file_type().map(|t| t.is_symlink()).unwrap_or(false) {
            return Some(e.path());
        }
        let p = e.path();
        if p.is_dir() {
            if let Some(found) = first_reparse_point(&p) {
                return Some(found);
            }
        }
    }
    None
}

fn assemble_foreign_roots(app: &tauri::AppHandle, own_root: &str) -> Result<Vec<String>, String> {
    let mut roots: Vec<String> = Vec::new();
    // `registered_universe_roots_strict` only ever refuses for the TRANSIENT reason — a corrupt
    // registry is set aside and treated as genuinely empty by `load_registry_for_update`, so it
    // returns `Ok`. Tagged explicitly rather than left bare, so the dialog never has to guess.
    for r in crate::universe::registered_universe_roots_strict(app)
        .map_err(|m| format!("{REFUSAL_TRANSIENT}{m}"))?
    {
        let rs = r.to_string_lossy().to_string();
        // Children of ANY registered universe are foreign content too (H6) — including the
        // active universe's own cUniverses.
        for c in crate::universe::resolve_child_universe_roots_recursive_strict(&r)
            .map_err(classify_refusal)?
        {
            roots.push(c.to_string_lossy().to_string());
        }
        if norm(&rs) != norm(own_root) {
            roots.push(rs);
        }
    }
    Ok(roots)
}

/// Read-only: classify the active universe for the proposal dialog. `copy_paths` lets the
/// dialog re-run the plan as the user flips entries between Move and Copy.
#[tauri::command(async)]
pub fn mig108_preflight(
    app: tauri::AppHandle,
    copy_paths: Option<Vec<String>>,
) -> Result<PreflightReport, String> {
    let root = crate::universe::active_universe_dir(&app)?
        .to_string_lossy()
        .to_string();
    let libs = crate::libraries::load_all_libraries(&app);
    let foreign = assemble_foreign_roots(&app, &root)?;
    Ok(classify(&root, &libs, &copy_paths.unwrap_or_default(), &foreign))
}

/// The unfinished-journal probe for boot (None = nothing to resume).
#[tauri::command(async)]
pub fn mig108_journal_state(app: tauri::AppHandle) -> Result<Option<JournalState>, String> {
    let cdir = active_cdir(&app)?;
    // Phase-4 audit — a CORRUPT journal (the only record of a possibly half-moved universe)
    // must reach the USER, not a dev-only console: propagate the error so the dialog shows
    // it instead of silently booting as if nothing happened.
    Ok(Journal::load(&cdir)?
        .filter(|j| j.is_unfinished())
        .map(|j| JournalState {
            entries_total: j.entries.len(),
            entries_moved: j.entries.iter().filter(|e| e.moved).count(),
            universe_root: j.universe_root.clone(),
            restorable: matches!(
                j.phase,
                Phase::Planned | Phase::Snapshotted | Phase::Moving | Phase::Moved | Phase::VerifyFailed
            ),
            last_error: j.last_error.clone(),
            phase: j.phase,
        }))
}

fn emit_phase(app: &tauri::AppHandle, phase: &str) {
    use tauri::Emitter;
    let _ = app.emit("mig108:progress", phase);
}

/// One engine phase per call (run_engine advances through all of them; this steps, so the
/// wrapper can emit progress between phases — the honest granularity, since the moves are
/// near-instant renames and the DB rewrite is one indivisible transaction).
fn run_engine_step(
    conn: &rusqlite::Connection,
    db_path: &Path,
    journal: &mut Journal,
    constellation_dir: &Path,
) -> Result<(), String> {
    match journal.phase {
        Phase::Planned => {
            let (b, jb, base) = take_snapshot(conn, db_path, constellation_dir)?;
            journal.snapshot_db = Some(b);
            journal.json_backups = jb;
            journal.baseline = Some(base);
            journal.phase = Phase::Snapshotted;
            journal.save(constellation_dir)
        }
        Phase::Snapshotted | Phase::Moving => run_move_phase(journal, constellation_dir),
        Phase::Moved | Phase::VerifyFailed => run_db_rewrite(conn, journal, constellation_dir),
        Phase::DbRewritten => run_json_rewrites(journal, constellation_dir),
        Phase::JsonRewritten => finish_with_trash_consolidation(journal, constellation_dir),
        Phase::Done => Ok(()),
    }
}

fn run_with_events(
    app: &tauri::AppHandle,
    journal: &mut Journal,
    cdir: &Path,
) -> Result<(), String> {
    use tauri::Manager;
    // Refuse the window close for the whole run (Phase-4 audit) — covers execute AND resume.
    let _running = RunningGuard::new();
    let state = app.state::<crate::search::SearchState>();
    let guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = guard.as_ref().ok_or("Search database not initialized")?;
    let db_path = crate::search::db_path(app).map_err(|e| e.to_string())?;

    // Stage-A findings A + C, both owed to the same fact: this process OUTLIVES the
    // migration (the post-Unify reload restarts only the webview). So the command layer
    // must (A) invalidate the process-lifetime registry + embed-index caches the engine's
    // direct libraries.json write bypassed — without this, read_library_tree's
    // Library-≠-Folder exclusion set still holds the PRE-migration paths and every
    // relocated library ALSO renders as a folder of the root (the Boss's screenshot) — and
    // (C) recreate the dropped triggers for the live session via idempotent init_db.
    let finish_in_process = |db_path: &Path| {
        crate::libraries::invalidate_libraries_cache();
        crate::embeds::invalidate_all_vault_indexes();
        if let Err(e) = crate::search::init_db(db_path) {
            // Non-fatal: the next boot's init_db recreates the triggers regardless; the
            // session merely runs without live sky maintenance until then. Surfaced loudly.
            eprintln!("[mig108] post-run init_db failed (triggers restored at next boot): {e}");
        }
    };

    let label = |p: &Phase| match p {
        Phase::Planned => "snapshot",
        Phase::Snapshotted | Phase::Moving => "moving",
        Phase::Moved | Phase::VerifyFailed => "rewriting",
        Phase::DbRewritten => "stores",
        Phase::JsonRewritten => "trash",
        Phase::Done => "done",
    };
    loop {
        emit_phase(app, label(&journal.phase));
        if matches!(journal.phase, Phase::Done) {
            finish_in_process(&db_path);
            return Ok(());
        }
        let before = journal.phase.clone();
        run_engine_step(conn, &db_path, journal, cdir)?;
        if journal.phase == before {
            return Err(format!("mig108: phase {:?} did not advance", before));
        }
    }
}

/// Execute the unification the user just approved. The frontend holds the freeze envelope
/// (dirty tabs flushed, second screen closed, watchers down) around this call and reloads
/// the window on success.
#[tauri::command(async)]
pub fn mig108_execute(app: tauri::AppHandle, copy_paths: Vec<String>) -> Result<(), String> {
    let cdir = active_cdir(&app)?;
    if let Some(j) = Journal::load(&cdir)? {
        if j.is_unfinished() {
            return Err("A previous unification is unfinished - resume it instead.".to_string());
        }
    }
    let report = mig108_preflight(app.clone(), Some(copy_paths))?;
    let mut journal = Journal::new(&report.universe_root, &report);
    if journal.entries.is_empty() {
        return Err("Nothing to unify.".to_string());
    }
    journal.save(&cdir)?;
    run_with_events(&app, &mut journal, &cdir)
}

/// Phase-4 audit — the promised rollback, now shipped. Valid for every phase where the DB
/// transaction never COMMITTED (Planned / Snapshotted / Moving / Moved / VerifyFailed —
/// VerifyFailed rolled back, so the DB is untouched): reverse each completed fs operation
/// (rename back; delete the root copy — its source was never touched; cross-volume mirrored
/// via the copied flag), then discard the journal. The snapshot stays on disk. After
/// DbRewritten the only safe direction is FORWARD (resume) — the command refuses, and the
/// dialog explains rather than offering an impossible button.
#[tauri::command(async)]
pub fn mig108_restore(app: tauri::AppHandle) -> Result<(), String> {
    let cdir = active_cdir(&app)?;
    // Restore moves directories back — the same mid-flight kill risk as the forward run.
    let _running = RunningGuard::new();
    let journal = Journal::load(&cdir)?.ok_or("No unification journal to restore")?;
    match journal.phase {
        Phase::Planned | Phase::Snapshotted | Phase::Moving | Phase::Moved | Phase::VerifyFailed => {}
        _ => {
            return Err(
                "The knowledge index has already been rewritten — finishing the unification is the only safe direction."
                    .to_string(),
            )
        }
    }
    // Reverse in reverse order (nesting-safe even though nested entries are refused).
    for e in journal.entries.iter().rev() {
        let old_p = Path::new(&e.old_path);
        let new_p = Path::new(&e.new_path);
        match e.action.as_str() {
            "copy" => {
                // The source was never modified; the copy (partial or complete) is ours.
                if new_p.exists() {
                    std::fs::remove_dir_all(new_p)
                        .map_err(|er| format!("restore: removing copy {} failed: {}", new_p.display(), er))?;
                }
            }
            _ => {
                if new_p.is_dir() && !old_p.exists() {
                    if same_volume(new_p, old_p) {
                        crate::write_gate::gate_rename(new_p, old_p, "mig108_restore")
                            .map_err(|er| format!("restore: rename-back {} failed: {}", new_p.display(), er))?;
                    } else {
                        crate::libraries::copy_dir_recursive(new_p, old_p)?;
                        std::fs::remove_dir_all(new_p).map_err(|er| er.to_string())?;
                    }
                } else if new_p.exists() && old_p.exists() {
                    // Our partial (started, never completed): the source is authoritative.
                    std::fs::remove_dir_all(new_p)
                        .map_err(|er| format!("restore: removing partial {} failed: {}", new_p.display(), er))?;
                }
            }
        }
    }
    Journal::discard(&cdir)?;
    crate::libraries::invalidate_libraries_cache();
    crate::embeds::invalidate_all_vault_indexes();
    Ok(())
}

/// Resume an unfinished run found at boot. Same engine, same journal.
#[tauri::command(async)]
pub fn mig108_resume(app: tauri::AppHandle) -> Result<(), String> {
    let cdir = active_cdir(&app)?;
    let mut journal = Journal::load(&cdir)?.ok_or("No unification journal to resume")?;
    if !journal.is_unfinished() {
        return Ok(());
    }
    run_with_events(&app, &mut journal, &cdir)
}

// ─── Slice 5 — bring an external folder in (the standing D2 flow) ──────────────────────────

/// Pure planning half: destination under the root, basename de-collided against the fs.
pub(crate) fn bring_in_dest(source: &str, universe_root: &str) -> Result<PathBuf, String> {
    let base = Path::new(source)
        .file_name()
        .ok_or("Cannot determine the folder's name")?
        .to_string_lossy()
        .to_string();
    let taken: HashSet<String> = std::fs::read_dir(universe_root)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_lowercase()))
                .collect()
        })
        .unwrap_or_default();
    let (name, _) = free_name(&base, &taken);
    Ok(Path::new(universe_root).join(name))
}

/// Boss D2 — "ask each time": the dialog collected Copy (default) or Move; this executes it.
/// Copy leaves the original untouched and unmanaged; Move takes ownership (same-volume
/// rename, cross-volume copy+remove). Either way the DESTINATION is registered — which
/// `add_library` accepts, because it is under the root by construction.
#[tauri::command(async)]
pub fn bring_in_library(
    app: tauri::AppHandle,
    source_path: String,
    mode: String,
) -> Result<crate::libraries::LibraryInfo, String> {
    let src = Path::new(&source_path);
    if !src.is_dir() {
        return Err("That folder does not exist.".to_string());
    }
    let root = crate::universe::active_universe_dir(&app)?
        .to_string_lossy()
        .to_string();
    if norm_under(&source_path, &root) {
        return Err("That folder is already inside the universe — use Add library.".to_string());
    }
    // H6 — never ingest another universe's root or child.
    let foreign = assemble_foreign_roots(&app, &root)?;
    if let Some(reason) = foreign_reason(&source_path, &foreign) {
        return Err(format!("That folder cannot be brought in: it {}.", reason));
    }
    // A registered external library is the MIGRATION's business, not this flow's — bringing
    // it in here would strand its index rows at the old path (the proposal dialog relocates
    // registered externals with the full rewrite).
    //
    // **Safety inspection 2026-08-22 (HIGH, silent-data-loss): this guard used to FAIL OPEN.**
    // It read `load_all_libraries`, the LENIENT reader, whose own doc says every caller must be
    // read-only — and it swallows both a read failure and a parse failure into an empty list,
    // then CACHES that empty answer for the process lifetime. An empty list makes the `any()`
    // below vacuously false, so the guard the user is relying on simply is not there, and a
    // registered external library gets relocated: its registry entry left pointing at a now-empty
    // folder, every index row stranded, and `add_library` afterwards reporting either success or
    // a registry error that reads as "nothing happened" — AFTER an irreversible move.
    //
    // The strict twin already exists. Two lines above, `assemble_foreign_roots` refuses on a
    // degraded read for exactly this reason (PJ-322, Boss decision 1: *a plan that moves
    // directories must refuse rather than guess*). This is the surviving instance of that class.
    let libs = crate::libraries::try_load_libraries(&app).map_err(|e| {
        format!(
            "Cannot check whether that folder is already a registered library ({e}). \
             Nothing was changed."
        )
    })?;
    if libs.iter().any(|l| norm(&l.path) == norm(&source_path)) {
        return Err(
            "That folder is a registered library — the unification proposal relocates it safely."
                .to_string(),
        );
    }

    // Phase-4 audit — a folder that IS a universe (registered or not) must be opened, not
    // ingested: swallowing its .constellation (search.db, universe.json, earned ledger)
    // into another universe as plain files is data mangling.
    //
    // **PJ-333 (Boss-ruled 2026-08-22) — this now WALKS UPWARD, and that is the change.**
    //
    // It used to ask `carries_universe_manifest(src)`: is this folder ITSELF a universe? That
    // left a hole the 2026-08-21 safety inspection confirmed. A second Constellation universe
    // sitting on disk but NOT in this install's registry — synced from another machine, or
    // removed from the list while its files were kept — cannot be named by
    // `assemble_foreign_roots`, so `foreign_reason` answers `None` for every path beneath it.
    // Pick a plain subfolder inside it, which carries no manifest of its own, and Bring-In →
    // Move succeeded: that universe's content was relocated OUT of it, with no error.
    //
    // `classify` was given the upward walk by PJ-322 because there the guard is monotone-safe.
    // The comment that stood here said widening THIS one was a user-facing behaviour change and
    // belonged to the Boss rather than a mid-cascade edit. It was, he ruled it, and here it is —
    // one concern, one predicate, both surfaces (the Whole-Ecosystem Fix Law).
    //
    // The message distinguishes the two conditions, because they call for different actions: a
    // folder that IS a universe should be OPENED; a folder INSIDE one should be moved from
    // within that universe, and the user is told which universe that is.
    if let Some(owner) = universe_manifest_at_or_above(src) {
        let owner_is_the_folder = norm(&owner.to_string_lossy()) == norm(&src.to_string_lossy());
        return Err(if owner_is_the_folder {
            "That folder is a universe of its own — open it from the universe switcher instead."
                .to_string()
        } else {
            format!(
                "That folder is inside another universe (\"{}\"), so bringing it in would take \
                 content out of that universe. Open that universe and move it from there instead.",
                owner
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| owner.display().to_string())
            )
        });
    }
    let dest = bring_in_dest(&source_path, &root)?;
    match mode.as_str() {
        "move" => {
            if same_volume(src, Path::new(&root)) {
                crate::write_gate::gate_rename(src, &dest, "bring_in")
                    .map_err(|e| format!("Move failed: {}", e))?;
            } else {
                copy_or_clean_up(src, &dest)?;

                // **Safety inspection 2026-08-22 (MED, content-loss): this delete used to be
                // unconditional.** `copy_dir_recursive` skips symlinks and junctions with a bare
                // `continue` and no log, so a junction'd subtree — an attachments folder, a shared
                // Research tree, an ordinary Windows layout — was not copied and was then
                // DESTROYED at the source, with `Ok` returned and nothing recorded.
                //
                // **The fix the inspection prescribed would not have caught its own scenario, and
                // that is worth stating.** It asked for `run_move_phase`'s src/dst file-count
                // compare — but `count_files` skips symlinks too, and by the same rule, so the
                // counts MATCH while the subtree is missing. The check would have passed and the
                // source would still have been deleted. Verified by reading both walkers.
                //
                // So the guard is on the thing that actually differs: if the source contains any
                // reparse point, the copy is by definition not a complete copy, and we keep the
                // original. Nothing is destroyed; the user is told exactly why and where both
                // copies are. The count compare is kept as a second, independent check for a
                // shortfall that is not symlink-related (an unreadable subdirectory, which
                // `count_files` swallows and `copy_dir_recursive` propagates).
                if let Some(link) = first_reparse_point(src) {
                    return Err(format!(
                        "Copied, but the original was NOT removed: {} contains a shortcut or \
                         junction ({}) that cannot be copied. Both copies are on disk — the new \
                         one at {} — so move or recreate that link yourself, then delete the \
                         original.",
                        src.display(),
                        link.display(),
                        dest.display()
                    ));
                }
                let (src_n, dst_n) = (count_files(src), count_files(&dest));
                if src_n != dst_n {
                    return Err(format!(
                        "Copied, but the original was NOT removed: {src_n} files at the source \
                         and {dst_n} at the destination. Both copies are on disk — the new one at \
                         {} — so nothing has been lost; compare them before deleting the original.",
                        dest.display()
                    ));
                }
                std::fs::remove_dir_all(src).map_err(|e| format!("Move cleanup failed: {}", e))?;
            }
        }
        _ => {
            copy_or_clean_up(src, &dest)?;
        }
    }
    crate::libraries::add_library(app, dest.to_string_lossy().to_string())
}

#[cfg(test)]
mod slice5_tests {
    use super::*;

    #[test]
    fn ensure_under_active_root_is_the_one_location_law() {
        let root = "E:/Universe";
        assert!(crate::libraries::ensure_under_active_root("E:/Universe/Lib", root).is_ok());
        assert!(crate::libraries::ensure_under_active_root("E:\\Universe\\Lib", root).is_ok());
        assert!(crate::libraries::ensure_under_active_root("E:/Universe", root).is_ok());
        assert!(crate::libraries::ensure_under_active_root("E:/Elsewhere/Lib", root).is_err());
        // separator-bounded: a sibling sharing the prefix is OUTSIDE
        assert!(crate::libraries::ensure_under_active_root("E:/UniverseTwo/Lib", root).is_err());
    }

    #[test]
    fn bring_in_dest_decollides_against_the_root() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        std::fs::create_dir_all(root.join("Notes")).unwrap();
        let dest = bring_in_dest("E:/Anywhere/Notes", &root.to_string_lossy()).unwrap();
        assert_eq!(dest.file_name().unwrap().to_string_lossy(), "Notes 2");
        let dest2 = bring_in_dest("E:/Anywhere/Fresh", &root.to_string_lossy()).unwrap();
        assert_eq!(dest2.file_name().unwrap().to_string_lossy(), "Fresh");
    }
}

#[cfg(test)]
mod slice3_tests {
    use super::*;

    #[test]
    fn consolidation_moves_decollides_and_removes_empty_sources() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path().join("Universe");
        let lib_a = root.join("LibA");
        let lib_b = root.join("LibB");
        for d in [&lib_a, &lib_b] {
            std::fs::create_dir_all(d.join(".trash")).unwrap();
        }
        // Same-named trashed note in BOTH libraries + a trashed FOLDER with an attachment.
        std::fs::write(lib_a.join(".trash").join("Note.md"), "FROM A").unwrap();
        std::fs::write(lib_b.join(".trash").join("Note.md"), "FROM B").unwrap();
        let folder = lib_a.join(".trash").join("Old Folder");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("pic.png"), "img").unwrap();
        // A suffixed name ALREADY at the destination.
        std::fs::create_dir_all(root.join(".trash")).unwrap();
        std::fs::write(root.join(".trash").join("Note 1.md"), "ALREADY HERE").unwrap();

        let libs = vec![
            root.to_string_lossy().to_string(),
            lib_a.to_string_lossy().to_string(),
            lib_b.to_string_lossy().to_string(),
        ];
        let moved = consolidate_trash(&root, &libs).unwrap();
        assert_eq!(moved, 3);

        let bodies: Vec<String> = std::fs::read_dir(root.join(".trash"))
            .unwrap()
            .flatten()
            .filter(|e| e.path().is_file())
            .map(|e| std::fs::read_to_string(e.path()).unwrap())
            .collect();
        assert!(bodies.contains(&"FROM A".to_string()));
        assert!(bodies.contains(&"FROM B".to_string()));
        assert!(bodies.contains(&"ALREADY HERE".to_string()), "pre-existing entry never clobbered");
        assert!(
            root.join(".trash").join("Old Folder").join("pic.png").is_file(),
            "a trashed folder moves as a unit, attachments included"
        );
        assert!(!lib_a.join(".trash").exists(), "emptied source .trash removed");
        assert!(!lib_b.join(".trash").exists());

        // Idempotent: a second run is a clean no-op.
        assert_eq!(consolidate_trash(&root, &libs).unwrap(), 0);
    }
}

#[cfg(test)]
mod rehearsal_harness {
    //! Slice-6 — the mechanical rehearsal: run the REAL engine against a scratch universe
    //! built by `lab/tools/mig108_make_rehearsal.py`, headless, timed, invariants asserted.
    //!
    //! Deliberately `#[ignore]`d: it touches a caller-supplied path on the real filesystem.
    //! Drive it explicitly:
    //!
    //!   MIG108_REHEARSAL_ROOT="E:\...\MIG108 Rehearsal" \
    //!   MIG108_COPY_BASENAMES="PJ-065-test-book" \
    //!   cargo test --lib mig108_rehearsal_run -- --ignored --nocapture
    use super::*;

    #[test]
    #[ignore]
    fn mig108_rehearsal_run() {
        let root = match std::env::var("MIG108_REHEARSAL_ROOT") {
            Ok(r) => r,
            Err(_) => {
                eprintln!("MIG108_REHEARSAL_ROOT not set — skipping");
                return;
            }
        };
        let copy_basenames: Vec<String> = std::env::var("MIG108_COPY_BASENAMES")
            .unwrap_or_default()
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();
        let cdir = Path::new(&root).join(".constellation");
        let db_path = cdir.join("search.db");
        let libs: Vec<crate::libraries::LibraryInfo> = serde_json::from_str(
            &std::fs::read_to_string(cdir.join("libraries.json")).unwrap(),
        )
        .unwrap();
        let copy_paths: Vec<String> = libs
            .iter()
            .filter(|l| {
                Path::new(&l.path)
                    .file_name()
                    .map(|n| copy_basenames.contains(&n.to_string_lossy().to_lowercase()))
                    .unwrap_or(false)
            })
            .map(|l| l.path.clone())
            .collect();

        let report = classify(&root, &libs, &copy_paths, &[]);
        let actionable = report.to_move().count();
        println!("rehearsal: {} actionable entries ({} copy)", actionable, copy_paths.len());
        assert!(actionable > 0, "nothing to rehearse — is this scratch already unified?");

        let mut journal = Journal::new(&root, &report);
        journal.save(&cdir).unwrap();

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let t0 = std::time::Instant::now();
        loop {
            let before = journal.phase.clone();
            if matches!(before, Phase::Done) { break; }
            let tp = std::time::Instant::now();
            run_engine_step(&conn, &db_path, &mut journal, &cdir).unwrap();
            println!(
                "rehearsal: phase {:?} -> {:?} in {:.1}s",
                before, journal.phase, tp.elapsed().as_secs_f64()
            );
        }
        let took = t0.elapsed();
        println!("rehearsal: engine completed in {:.1}s", took.as_secs_f64());
        assert_eq!(journal.phase, Phase::Done);

        // The independent wider-net check (LL-040: never only the engine's own verify):
        // zero rows under ANY journal old path, copy-class included.
        let mut stale = 0i64;
        let old_prefixes = NormPrefixes::new(journal.entries.iter().map(|e| e.old_path.as_str()));
        for (table, col, _) in SWEEP {
            if let Ok(mut stmt) = conn.prepare(&format!("SELECT {c} FROM {t}", c = col, t = table)) {
                stale += stmt
                    .query_map([], |r| r.get::<_, String>(0))
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .filter(|p| old_prefixes.matches(p))
                    .count() as i64;
            }
        }
        {
            let mut stmt = conn.prepare("SELECT path FROM note_meta").unwrap();
            stale += stmt
                .query_map([], |r| r.get::<_, String>(0))
                .unwrap()
                .filter_map(|r| r.ok())
                .filter(|p| old_prefixes.matches(p))
                .count() as i64;
        }
        assert_eq!(stale, 0, "wider-net stale check must be ZERO (copy-class included)");
        println!("rehearsal: wider-net stale rows = 0 — PASS");
    }
}
