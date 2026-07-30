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

    PreflightReport {
        universe_root: universe_root.to_string(),
        entries,
        decollided,
    }
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
                })
                .collect(),
            baseline: None,
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
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| format!("wal_checkpoint failed: {}", e))?;

    let baseline = read_baseline(conn)?;

    let db_backup = backup_dir.join("search.db.pre-mig108");
    std::fs::copy(db_path, &db_backup).map_err(|e| format!("db backup copy failed: {}", e))?;
    for ext in ["-wal", "-shm"] {
        let side = PathBuf::from(format!("{}{}", db_path.display(), ext));
        if side.exists() && std::fs::metadata(&side).map(|m| m.len() > 0).unwrap_or(false) {
            let dest = backup_dir.join(format!("search.db.pre-mig108{}", ext));
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
