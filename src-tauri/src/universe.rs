// ─── Constellation Universe — Portable User-Owned Data Storage ───

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

// ─── Data Structures ───

/// Metadata stored inside each universe's .constellation/universe.json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseMeta {
    pub name: String,
    pub created: String,
    pub version: u32,
    #[serde(default)]
    pub children: Vec<String>,
    /// Relative folder name for universe-level notes (e.g., "كون عيسى")
    #[serde(default)]
    pub notes_folder: Option<String>,
}

/// Entry in the global registry (app_data_dir/universes.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created: String,
}

/// Global registry stored in app_data_dir.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UniverseRegistry {
    entries: Vec<UniverseEntry>,
    active_id: Option<String>,
}

/// Tauri managed state — holds the active universe path.
pub struct UniverseState {
    pub active_path: Mutex<Option<PathBuf>>,
    /// Batch-2 §B2-5 — serializes whole universe switches. While
    /// `set_active_universe` was SYNC, the single IPC dispatch thread
    /// serialized concurrent activations (main-window boot restore + the
    /// second screen both call it) for free; as `(async)` they would
    /// interleave the teardown/heal/migrate sequence. Every switch runs
    /// under this lock, with the already-active check re-run under it.
    pub switch_lock: Mutex<()>,
}

impl UniverseState {
    pub fn new() -> Self {
        Self {
            active_path: Mutex::new(None),
            switch_lock: Mutex::new(()),
        }
    }
}

// ─── .constellation/ Directory Helpers ───

/// Return the .constellation/ config directory inside a universe root.
pub fn constellation_dir(universe_root: &Path) -> PathBuf {
    universe_root.join(".constellation")
}

/// Return the .constellation/ directory for the active universe.
pub fn active_constellation_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = active_universe_dir(app)?;
    Ok(constellation_dir(&root))
}

/// Resolve the .constellation/ dir for an EXPLICIT universe root when given,
/// else fall back to the ambient active universe.
///
/// 2026-08-01 safety inspection (cross-window-clobber class): the active
/// pointer flips BEFORE the frontend switch handler runs (UniverseManager
/// awaits set_active_universe first), so an ambient-keyed save racing a
/// universe switch writes universe A's data into universe B's file. Same
/// treatment as the MIG-100 session.json commands (which take the explicit
/// root for exactly this reason), kept backward-compatible via Option so
/// existing call sites keep working until the frontend passes the root.
pub fn resolve_constellation_dir(
    app: &tauri::AppHandle,
    universe_root: Option<String>,
) -> Result<PathBuf, String> {
    match universe_root {
        Some(root) => Ok(constellation_dir(Path::new(&root))),
        None => active_constellation_dir(app),
    }
}

// ─── Registry Helpers ───

/// Path to the global universe registry: {app_data_dir}/universes.json
fn registry_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    Ok(app_dir.join("universes.json"))
}

/// **Use this on every path that will WRITE the registry back.** It surfaces an unreadable or
/// corrupt `universes.json` as an error instead of an empty registry, so `load → mutate → save`
/// cannot replace every Universe the user has with the one they just made.
///
/// `load_registry` below is the lenient read-only twin: it still falls back to empty (a
/// read-only consumer showing nothing is a display problem, not data loss) and is now
/// documented as forbidden on any write-back path.
fn load_registry_for_update(app: &tauri::AppHandle) -> Result<UniverseRegistry, String> {
    let path = registry_path(app)?;
    let empty = || UniverseRegistry { entries: vec![], active_id: None };
    match read_persisted_json::<UniverseRegistry>(&path) {
        Ok(Some(r)) => Ok(r),
        Ok(None) => Ok(empty()),
        // Transient — the registry is fine and simply unread. Refuse; a retry will succeed.
        Err(e @ PersistedError::Unreadable(_)) => Err(e.into()),
        // 2026-08-02 audit — the FIRST version of this fix returned Err here too, and that
        // blocked EVERY route into the app: create a universe, add an existing one, the
        // first-run setup, the migration escape hatch. Boot is lenient, so the user reached a
        // working-looking window where nothing could be done and no message explained why.
        // Refusing to overwrite must never become refusing to continue. Set the unusable file
        // aside — the bytes are preserved for recovery — and proceed from empty, which is now
        // TRUE rather than assumed.
        Err(PersistedError::Corrupt(msg)) => {
            eprintln!("[universe] {msg}");
            set_aside_corrupt(&path);
            Ok(empty())
        }
    }
}

/// READ-ONLY. Falls back to an empty registry when the file cannot be read or parsed — which
/// is safe to *display* and catastrophic to *save*. Never call this on a path that writes the
/// registry back; use `load_registry_for_update`.
fn load_registry(app: &tauri::AppHandle) -> UniverseRegistry {
    let path = match registry_path(app) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[universe] Failed to get registry path: {}", e);
            return UniverseRegistry { entries: vec![], active_id: None };
        }
    };
    if path.exists() {
        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[universe] Failed to read registry ({}): {}", path.display(), e);
                return UniverseRegistry { entries: vec![], active_id: None };
            }
        };
        serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("[universe] Corrupt registry JSON ({}): {}", path.display(), e);
            UniverseRegistry { entries: vec![], active_id: None }
        })
    } else {
        UniverseRegistry { entries: vec![], active_id: None }
    }
}

/// Safety Audit G6 — ATOMIC write for persisted-state files: write to a temp then
/// rename over the target. A plain `fs::write` truncates-then-writes, so a crash or
/// power loss mid-write leaves the file partial — and every loader here swallows the
/// parse error and falls back to empty (silently dropping the user's registry /
/// settings / workspaces / collections / property-types). The rename is atomic on
/// the same directory; a failed rename leaves the old file intact (never truncated).
/// PJ-187 — the temp name must be UNIQUE PER WRITE.
///
/// It used to be `<target>.tmp`: one fixed name, no lock. Every persisted-state file in the
/// app goes through this one function — the universe registry, `universe.json`, `settings.json`,
/// `workspaces.json`, the tab session, `collections.json`, `property-types.json` — so two
/// writers of the SAME file (two windows, or a settings save racing the session autosave)
/// both created, wrote and fsync'd *the same temp path*. Whoever renamed second could publish
/// the other's half-written bytes under the final name, and the loser's `remove_file` could
/// delete a temp the winner was still using. The failure mode is exactly what the temp+rename
/// dance exists to prevent, and every loader here **swallows the parse error and falls back to
/// empty** — so a corrupted registry does not error, it silently presents as "no universes",
/// "no collections", "no saved workspaces", and the next save writes that emptiness back.
///
/// `write_gate::atomic_write` already had the answer two files away — `.{stem}.{pid}-{n}.cnstmp`
/// — and `link_life.rs` states the rule outright: PJ-087, never reuse a fixed temp name.
static STATE_TMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) fn atomic_write(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    let stem = path.file_name().and_then(|n| n.to_str()).unwrap_or("state.json");
    let tmp = path.with_file_name(format!(
        ".{}.{}-{}.cnstmp",
        stem,
        std::process::id(),
        STATE_TMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    // MIG-100 / G6 hardening: fsync BEFORE the rename — otherwise power loss
    // can land the rename while the data blocks are still unflushed, leaving
    // a zero-length/garbage file under the FINAL name (the exact failure the
    // temp+rename dance exists to prevent; write_gate.rs documents the same
    // requirement for note writes).
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(contents)?;
        f.sync_all()?;
    }
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// **The read-side twin of `atomic_write`** — and the cure for the defect the comment above
/// has described, accurately and in writing, since the G6 audit: *"every loader here swallows
/// the parse error and falls back to empty … and the next save writes that emptiness back."*
/// That audit hardened the WRITE and left the READ exactly as described. Worse, an atomic
/// write GUARANTEES the destructive overwrite completes cleanly.
///
/// The distinction the old loaders could not make, and the whole point of this function:
///
/// * **Absent** — `Ok(None)`. Genuinely "none yet". Safe to treat as empty AND to write over.
/// * **Present and readable** — `Ok(Some(value))`.
/// * **Present but unreadable or unparseable** — `Err`. **The user's data is still on disk;
///   we merely failed to see it.** A caller that writes back here destroys it. One second of
///   a sync tool, antivirus or a backup job holding the file is an everyday event.
///
/// Any load → mutate → save path MUST use this and propagate the `Err`. Read-only consumers
/// may fall back to a default, but must never then persist that default.
/// (2026-08-02 consolidated triage, ranked concern #1 — 13 register entries, 4 files.)
/// Why a persisted read failed — the two cases need OPPOSITE handling, and collapsing them is
/// how the first version of this fix locked the user out of their own app.
#[derive(Debug)]
pub(crate) enum PersistedError {
    /// The file exists and we could not read it: a lock, a sharing violation, an I/O error.
    /// **Transient.** The data is intact; refusing is correct and it will work in a moment.
    /// Never set this file aside — there is nothing wrong with it.
    Unreadable(String),
    /// The file exists and its contents are not usable: unparseable, truncated, empty.
    /// **Permanent until something intervenes.** Refusing alone would strand the user forever,
    /// so a caller that would otherwise be blocked may set it aside and continue — preserving
    /// the bytes for recovery while unblocking the app.
    Corrupt(String),
}

impl PersistedError {
    pub(crate) fn message(&self) -> &str {
        match self {
            PersistedError::Unreadable(m) | PersistedError::Corrupt(m) => m,
        }
    }
}

impl From<PersistedError> for String {
    fn from(e: PersistedError) -> String {
        e.message().to_string()
    }
}

pub(crate) fn read_persisted_json<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Option<T>, PersistedError> {
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        // Only "not found" is trustworthy emptiness. Permission-denied, sharing violations
        // (the Windows AV/sync case) and I/O errors all mean the data is there and unread.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(PersistedError::Unreadable(format!(
                "Could not read {} ({}). Refusing to treat it as empty.",
                path.display(),
                e
            )))
        }
    };
    // A zero-length file is the classic half-written / truncated state. It is NOT "no data".
    if data.trim().is_empty() {
        return Err(PersistedError::Corrupt(format!(
            "{} is empty — refusing to treat a truncated file as no data.",
            path.display()
        )));
    }
    serde_json::from_str(&data).map(Some).map_err(|e| {
        PersistedError::Corrupt(format!(
            "Could not parse {} ({}). Refusing to overwrite it.",
            path.display(),
            e
        ))
    })
}

/// Preserve a file we cannot use, so refusing to overwrite it never becomes refusing to let the
/// user continue. Mirrors `libraries::back_up_corrupt_config` and the review-pulse rename-aside.
pub(crate) fn set_aside_corrupt(path: &Path) -> Option<PathBuf> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let aside = path.with_file_name(format!(
        "{}.corrupt-{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("state.json"),
        secs
    ));
    match std::fs::rename(path, &aside) {
        Ok(()) => {
            eprintln!("[persist] set aside unusable {} → {}", path.display(), aside.display());
            Some(aside)
        }
        Err(e) => {
            eprintln!("[persist] could NOT set aside {}: {}", path.display(), e);
            None
        }
    }
}

/// MIG-100 safety-inspection fix — best-effort ATOMIC persist for the
/// federation/migration heal sites that previously did `let _ = fs::write`
/// (non-atomic AND error-swallowed). A plain `fs::write` truncates-then-writes,
/// so a crash / power-loss / AV-lock mid-write leaves `universe.json` /
/// `libraries.json` PARTIAL — and every loader here swallows the parse error
/// and falls back to EMPTY, silently dropping the user's federation manifest
/// or library registrations. `atomic_write` (temp + fsync + rename) makes that
/// impossible. Best-effort by design (these heals are opportunistic), but a
/// failure is now LOGGED, never silently discarded.
fn persist_json_best_effort(path: &Path, json: &str) {
    if let Err(e) = atomic_write(path, json.as_bytes()) {
        eprintln!("[universe] Failed to persist {}: {}", path.display(), e);
    }
}

fn save_registry(app: &tauri::AppHandle, registry: &UniverseRegistry) -> Result<(), String> {
    let path = registry_path(app)?;
    let data = serde_json::to_string_pretty(registry).map_err(|e| e.to_string())?;
    atomic_write(&path, data.as_bytes()).map_err(|e| format!("Failed to save universe registry: {}", e))
}

// ─── Active Universe Helper ───

/// Get the active universe ROOT directory path from managed state.
pub fn active_universe_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let state = app.state::<UniverseState>();
    let lock = state.active_path.lock().map_err(|e| e.to_string())?;
    lock.clone().ok_or_else(|| "No active universe set.".to_string())
}

// ─── UUID Helper ───

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let random: u32 = (timestamp as u32).wrapping_mul(2654435761) ^ std::process::id();
    format!("{:x}{:04x}", timestamp, random & 0xFFFF)
}

// ─── Migration: old flat format → .constellation/ ───

/// Auto-migrate a universe from the old flat layout to .constellation/ subdirectory.
/// Called when setting an active universe.
fn migrate_to_constellation(universe_root: &Path) -> Result<(), String> {
    let cdir = constellation_dir(universe_root);
    let old_meta = universe_root.join("universe.json");

    // Only migrate if old format exists but .constellation/ does not
    if !old_meta.exists() || cdir.exists() {
        return Ok(());
    }

    eprintln!("[universe] Migrating {} to .constellation/ format", universe_root.display());

    fs::create_dir_all(&cdir)
        .map_err(|e| format!("Failed to create .constellation/: {}", e))?;
    fs::create_dir_all(cdir.join("bases"))
        .map_err(|e| format!("Failed to create .constellation/bases/: {}", e))?;

    // Move config files into .constellation/
    let files_to_move = [
        "universe.json",
        "settings.json",
        "bookmarks.json",
        "workspaces.json",
        "property-types.json",
        "collections.json",
        "workbench.json", // MIG-092: legacy — read_universe_collections adopts it
    ];
    for file in &files_to_move {
        let src = universe_root.join(file);
        if src.exists() {
            let dest = cdir.join(file);
            fs::rename(&src, &dest)
                .map_err(|e| format!("Failed to move {}: {}", file, e))?;
        }
    }

    // Rename vaults.json → libraries.json during migration
    let old_vaults = universe_root.join("vaults.json");
    if old_vaults.exists() {
        let dest = cdir.join("libraries.json");
        fs::rename(&old_vaults, &dest)
            .map_err(|e| format!("Failed to move vaults.json → libraries.json: {}", e))?;
    }

    // Move bases/ directory contents
    let old_bases = universe_root.join("bases");
    if old_bases.is_dir() {
        let new_bases = cdir.join("bases");
        if let Ok(entries) = fs::read_dir(&old_bases) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.is_file() {
                    let dest = new_bases.join(entry.file_name());
                    fs::rename(&src, &dest).ok();
                }
            }
        }
        // Remove old empty bases dir
        fs::remove_dir(&old_bases).ok();
    }

    eprintln!("[universe] Migration complete for {}", universe_root.display());
    Ok(())
}

/// Ensure the universe notes folder exists for existing universes (migration).
fn ensure_universe_notes_folder(universe_root: &Path) -> Result<(), String> {
    let cdir = constellation_dir(universe_root);
    let meta_path = cdir.join("universe.json");

    if !meta_path.exists() {
        return Ok(());
    }

    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    // ─── Migration: flatten nested universe folder ───
    // Old behavior created UniverseName/UniverseName/ (nested). New behavior uses
    // the universe root directly as the library (Obsidian-style flat).
    if let Some(ref folder_name) = meta.notes_folder {
        let nested_path = universe_root.join(folder_name);
        // If nested folder exists AND has the same name as the universe → migrate to flat
        if nested_path.is_dir() && folder_name == &meta.name {
            // Walk down the chain of same-name nesting: root/Name/Name/Name/...
            // Find the deepest level, then move everything up to root.
            let mut deepest = nested_path.clone();
            loop {
                let next = deepest.join(folder_name);
                if next.is_dir() { deepest = next; } else { break; }
            }
            eprintln!("[universe] Migrating nested folders to flat (deepest: {})", deepest.display());

            // Move contents from deepest level up to universe root
            if let Ok(entries) = fs::read_dir(&deepest) {
                for entry in entries.flatten() {
                    let src = entry.path();
                    let fname = entry.file_name();
                    // Skip if it's another same-name directory (already traversed)
                    if src.is_dir() && fname.to_string_lossy() == folder_name.as_str() { continue; }
                    let dest = universe_root.join(&fname);
                    if !dest.exists() {
                        let _ = fs::rename(&src, &dest);
                    }
                }
            }

            // Also move contents from intermediate levels (they may have files too)
            let mut level = nested_path.clone();
            while level != *universe_root {
                if let Ok(entries) = fs::read_dir(&level) {
                    for entry in entries.flatten() {
                        let src = entry.path();
                        let fname = entry.file_name();
                        if src.is_dir() && fname.to_string_lossy() == folder_name.as_str() { continue; }
                        let dest = universe_root.join(&fname);
                        if !dest.exists() {
                            let _ = fs::rename(&src, &dest);
                        }
                    }
                }
                let next = level.join(folder_name);
                if next.is_dir() { level = next; } else { break; }
            }

            // Remove empty nested folders bottom-up
            let mut cleanup = deepest.clone();
            while cleanup != *universe_root {
                let _ = fs::remove_dir(&cleanup); // only removes if empty
                if let Some(parent) = cleanup.parent() {
                    cleanup = parent.to_path_buf();
                } else { break; }
            }

            // Update metadata to flat (notes_folder = None)
            meta.notes_folder = None;
            if let Ok(json) = serde_json::to_string_pretty(&meta) {
                persist_json_best_effort(&meta_path, &json);
            }

            // Update libraries.json: point universe_notes library to root
            let libs_path = cdir.join("libraries.json");
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    for lib in &mut libs {
                        if lib.is_universe_notes {
                            lib.path = universe_root.to_string_lossy().to_string();
                        }
                    }
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        persist_json_best_effort(&libs_path, &json);
                    }
                }
            }
            eprintln!("[universe] Migration to flat structure complete");
            return Ok(());
        }

        // Non-matching subfolder (e.g. user renamed) — leave as-is, just ensure it exists
        if !nested_path.exists() {
            fs::create_dir_all(&nested_path)
                .map_err(|e| format!("Failed to create universe notes folder: {}", e))?;
        }
        // Ensure registered as library
        let libs_path = cdir.join("libraries.json");
        let folder_path_str = nested_path.to_string_lossy().to_string();
        if libs_path.exists() {
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    if !libs.iter().any(|l| l.is_universe_notes) {
                        let mut libs = libs;
                        libs.insert(0, crate::libraries::LibraryInfo {
                            id: format!("universe_notes_{}", uuid_simple()),
                            name: meta.name.clone(),
                            path: folder_path_str,
                            is_universe_notes: true,
                            canonical_mode: "native".to_string(),
                        });
                        if let Ok(json) = serde_json::to_string_pretty(&libs) {
                            persist_json_best_effort(&libs_path, &json);
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    // notes_folder is None → universe root IS the library (flat/Obsidian-style)
    // Just ensure it's registered as a library
    let libs_path = cdir.join("libraries.json");
    let root_path_str = universe_root.to_string_lossy().to_string();
    // A read or parse FAILURE must never be read as "there are no libraries".
    //
    // This block exists to auto-register the Universe root as its own library, and it
    // decides whether to write by asking whether the registry already contains such an
    // entry. Collapsing an I/O error into an empty Vec (`.ok()...unwrap_or_default()`)
    // answered "no entry" for a file we simply could not read — and then atomically
    // REPLACED the registry with a single entry, deleting every library the user had
    // registered, with no error and no backup. One transient lock (a sync client, an
    // AV scanner, a network drive) on any boot was enough, and this runs on EVERY
    // universe activation. (2026-07-21 inspection, APP-KILLER.)
    //
    // Absent is a fact; unreadable is an unknown. Only the fact may proceed.
    let mut libs: Vec<crate::libraries::LibraryInfo> = if libs_path.exists() {
        let data = fs::read_to_string(&libs_path).map_err(|e| {
            format!("Could not read {}: {e}. Refusing to touch it.", libs_path.display())
        })?;
        serde_json::from_str(&data).map_err(|e| {
            format!("Could not parse {}: {e}. Refusing to overwrite it.", libs_path.display())
        })?
    } else {
        vec![]
    };

    if !libs.iter().any(|l| l.is_universe_notes) {
        libs.insert(0, crate::libraries::LibraryInfo {
            id: format!("universe_notes_{}", uuid_simple()),
            name: meta.name.clone(),
            path: root_path_str,
            is_universe_notes: true,
            canonical_mode: "native".to_string(),
        });
        if let Ok(json) = serde_json::to_string_pretty(&libs) {
            persist_json_best_effort(&libs_path, &json);
        }
    }

    Ok(())
}

// ─── Library Resolution (Universe of Universes) ───

/// Recursively resolve all libraries accessible from a universe directory.
/// Collects own libraries + child universe libraries, deduplicated by path.
/// PJ-207 §8 — `pub(crate)` so the regression test can prove the two library sets
/// genuinely DIFFER on a federated universe using the real resolver, rather than
/// hand-building a "recursive set" that would keep passing after production stopped
/// producing one. (The `search.rs:338` mirror trap §1 deleted; not re-introduced in a test.)
pub(crate) fn resolve_libraries_recursive(universe_path: &Path, visited: &mut Vec<PathBuf>) -> Vec<crate::libraries::LibraryInfo> {
    // Prevent circular references
    if let Ok(canon) = fs::canonicalize(universe_path) {
        if visited.contains(&canon) {
            return vec![];
        }
        visited.push(canon);
    }

    let mut all_libraries: Vec<crate::libraries::LibraryInfo> = Vec::new();
    let cdir = constellation_dir(universe_path);

    // 1. Load own libraries from .constellation/libraries.json
    let libs_path = cdir.join("libraries.json");
    if libs_path.exists() {
        if let Ok(data) = fs::read_to_string(&libs_path) {
            if let Ok(libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&data) {
                all_libraries.extend(libs);
            }
        }
    } else {
        // Fallback: try old flat format (vaults.json at root)
        let old_path = universe_path.join("vaults.json");
        if old_path.exists() {
            if let Ok(data) = fs::read_to_string(&old_path) {
                if let Ok(libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&data) {
                    all_libraries.extend(libs);
                }
            }
        }
    }

    // 2. Load children from .constellation/universe.json and recurse
    let meta_path = cdir.join("universe.json");
    let meta_path = if meta_path.exists() { meta_path } else { universe_path.join("universe.json") };
    if meta_path.exists() {
        if let Ok(data) = fs::read_to_string(&meta_path) {
            if let Ok(meta) = serde_json::from_str::<UniverseMeta>(&data) {
                for child_path_str in &meta.children {
                    let child_canon = match fs::canonicalize(child_path_str) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    if child_canon.is_dir() {
                        let child_libs = resolve_libraries_recursive(&child_canon, visited);
                        all_libraries.extend(child_libs);
                    }
                }
            }
        }
    }

    // 3. Deduplicate by path
    let mut seen = std::collections::HashSet::new();
    all_libraries.retain(|v| seen.insert(v.path.clone()));

    all_libraries
}

/// Resolve the cUniverse children declared by `parent` into canonicalized
/// Universe-root `PathBuf`s.
///
/// Reads `<parent>/.constellation/universe.json` (falling back to
/// `<parent>/universe.json` for legacy layouts), decodes `UniverseMeta`,
/// and canonicalizes each `children` entry. Non-existent or non-directory
/// entries are silently skipped — a child Universe that was moved or
/// deleted shouldn't block the active Universe from booting.
///
/// Used by `set_active_universe` to feed
/// `arabic::overrides::activate_layered_for_universe` (M8b-v2), and a
/// natural hook point for any future layered-per-Universe surface (tag
/// browser federation, sky view merging, etc.).
pub(crate) fn resolve_child_universe_roots(parent: &Path) -> Vec<PathBuf> {
    let cdir = constellation_dir(parent);
    let meta_path = cdir.join("universe.json");
    let meta_path = if meta_path.exists() {
        meta_path
    } else {
        parent.join("universe.json")
    };
    if !meta_path.exists() {
        return Vec::new();
    }
    let Ok(data) = fs::read_to_string(&meta_path) else {
        return Vec::new();
    };
    let Ok(meta) = serde_json::from_str::<UniverseMeta>(&data) else {
        return Vec::new();
    };
    meta.children
        .iter()
        .filter_map(|s| fs::canonicalize(s).ok())
        .filter(|p| p.is_dir())
        .collect()
}

/// MIG-111 §1.2/A4 — **the same enumeration, fail-closed.**
///
/// `resolve_child_universe_roots` answers "no children" for three different situations:
/// there are none, the manifest could not be READ, and the manifest could not be PARSED.
/// For the sidebar surfaces that is tolerable — a group is missing from a list. For the
/// **owner resolver** it is not, and the reason is MIG-108: linked universes live UNDER the
/// active root, so a note inside one is inside the parent too. Lose the child from the
/// candidate set and `resolve_owner` does not refuse — it confidently returns the PARENT,
/// `is_active: true`.
///
/// **This is PRE-EMPTIVE, not a bug that shipped.** `resolve_owner` has no production caller
/// yet (`WriteScope::for_note` is its only caller, and nothing calls that), so no write has
/// ever taken the wrong branch. The hazard is what would happen the moment Phase 1.2 wires it
/// — the routed write would land in the wrong universe's database with every row count still
/// correct. Closing it before the wiring is the point; do not read this as a defect log. (`resolve_owner`'s own doc comment says it reads
/// fresh rather than from `load_all_libraries`' cache to avoid a degraded federation — and
/// then called a reader that degraded silently one layer down.)
///
/// Two deliberate differences from the lenient reader, neither of which invents a policy:
///
/// 1. **A manifest that exists but cannot be read or parsed is an ERROR**, not an empty list.
///    This is `read_persisted_json`'s standing doctrine — only "not found" is trustworthy
///    emptiness — applied to the file that defines the federation. A universe with no
///    manifest, or a manifest declaring no children, is still `Ok(vec![])`: zero cUniverses
///    is a complete, valid setup.
/// 2. **A declared child that cannot be canonicalized is KEPT, as its declared path**, not
///    dropped. Canonicalization here is de-duplication, not a membership test; dropping on
///    its failure removes a universe from the candidate set, which is the misroute above.
///    Keeping it can only make the set MORE complete — and if the folder really is gone, no
///    path resolves inside it anyway, while if it is merely unreachable the *next* precondition
///    (its `search.db` must already exist) refuses by name instead of writing elsewhere.
pub(crate) fn resolve_child_universe_roots_strict(
    parent: &Path,
) -> Result<Vec<PathBuf>, PersistedError> {
    let cdir = constellation_dir(parent);
    let meta_path = cdir.join("universe.json");
    let meta_path = if meta_path.exists() { meta_path } else { parent.join("universe.json") };

    let Some(meta) = read_persisted_json::<UniverseMeta>(&meta_path)
        .map_err(|e| name_the_universe(parent, e))?
    else {
        return Ok(Vec::new()); // no manifest ⇒ genuinely no declared children
    };

    let mut out = Vec::with_capacity(meta.children.len());
    for child in &meta.children {
        match fs::canonicalize(child) {
            Ok(p) => out.push(p),
            // The folder is genuinely gone — a universe that was deleted. Keep the DECLARED
            // path: nothing resolves inside a directory that does not exist, so keeping it
            // cannot misroute, while dropping it is exactly what removes a universe from the
            // candidate set.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => out.push(PathBuf::from(child)),
            // Anything else — permission denied, a sharing violation, an offline placeholder,
            // a disconnected share — means the directory IS there and we could not look at it.
            // `Unreadable`, so a caller can retry rather than treat it as permanent.
            Err(e) => {
                return Err(PersistedError::Unreadable(format!(
                    "A universe linked to \"{}\" could not be checked ({}: {e}). Refusing to treat it as absent. Nothing was changed.",
                    universe_label(parent),
                    child
                )))
            }
        }
    }
    Ok(out)
}

/// Name the universe in a persisted-read failure **while preserving which KIND it was.**
///
/// The first version of `resolve_child_universe_roots_strict` collapsed both variants into one
/// `String` — losing exactly the distinction `load_registry_for_update` makes twenty lines
/// above, under a comment recording why it exists: *"the FIRST version of this fix returned Err
/// here too, and that blocked EVERY route into the app … refusing to overwrite must never
/// become refusing to continue."* That scar was re-cut in a new file and the panel caught it.
///
/// The two are not the same decision downstream. `Unreadable` is usually transient (an
/// antivirus or sync tool holding the file for a moment) and the right response is to keep the
/// note dirty and retry. `Corrupt` is permanent until something intervenes, and retrying is
/// pointless. A caller handed one `String` cannot tell them apart.
fn name_the_universe(parent: &Path, e: PersistedError) -> PersistedError {
    let msg = format!(
        "Cannot determine which universes are linked to \"{}\" — {} Nothing was changed.",
        universe_label(parent),
        e.message()
    );
    match e {
        PersistedError::Unreadable(_) => PersistedError::Unreadable(msg),
        PersistedError::Corrupt(_) => PersistedError::Corrupt(msg),
    }
}

fn universe_label(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.display().to_string())
}

/// The recursive form of the fail-closed enumeration. Same cycle guard and same subtree
/// order as `resolve_child_universe_roots_recursive`; the difference is that an unreadable
/// manifest ANYWHERE in the tree aborts rather than silently pruning that branch.
pub(crate) fn resolve_child_universe_roots_recursive_strict(
    parent: &Path,
) -> Result<Vec<PathBuf>, PersistedError> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut visited: Vec<PathBuf> = Vec::new();
    let canon_parent = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    visited.push(canon_parent);
    let mut stack: Vec<PathBuf> = resolve_child_universe_roots_strict(parent)?;
    while let Some(root) = stack.pop() {
        let canon = fs::canonicalize(&root).unwrap_or_else(|_| root.clone());
        if visited.contains(&canon) {
            continue; // cycle / duplicate guard
        }
        visited.push(canon);
        for child in resolve_child_universe_roots_strict(&root)? {
            stack.push(child);
        }
        out.push(root);
    }
    Ok(out)
}

/// MIG-062 §B — recursively resolve ALL cUniverse roots in the federation
/// tree (direct children, their children, …), de-duplicated and cycle-guarded.
///
/// Matches the federated set CNS sees (MIG-061) so the filesystem-walk sidebar
/// surfaces (Five Acts, Workspace Bases) federate over exactly the same
/// universes. The active `parent` itself is NOT included — only descendants.
///
/// Cycle guard: canonicalized paths in `visited` prevent an A→B→A federation
/// loop (or a self-referencing universe.json) from spinning forever.
pub(crate) fn resolve_child_universe_roots_recursive(parent: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut visited: Vec<PathBuf> = Vec::new();
    let canon_parent = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    visited.push(canon_parent);
    let mut stack: Vec<PathBuf> = resolve_child_universe_roots(parent);
    while let Some(root) = stack.pop() {
        let canon = fs::canonicalize(&root).unwrap_or_else(|_| root.clone());
        if visited.contains(&canon) {
            continue; // cycle / duplicate guard
        }
        visited.push(canon);
        // Descend into this cUniverse's own children before recording it,
        // so the whole subtree is enumerated.
        for child in resolve_child_universe_roots(&root) {
            stack.push(child);
        }
        out.push(root);
    }
    out
}

/// MIG-062 — display name of the universe rooted at `root`, from its
/// universe.json `name`. Falls back to the directory name if the manifest
/// is missing/unreadable. Used to label federated cUniverse sub-groups in
/// the Five Acts / Workspace Bases sidebar sections.
pub(crate) fn universe_display_name(root: &Path) -> String {
    let cdir = constellation_dir(root);
    let meta_path = if cdir.join("universe.json").exists() {
        cdir.join("universe.json")
    } else {
        root.join("universe.json")
    };
    if let Ok(data) = fs::read_to_string(&meta_path) {
        if let Ok(meta) = serde_json::from_str::<UniverseMeta>(&data) {
            if !meta.name.is_empty() {
                return meta.name;
            }
        }
    }
    root.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("cUniverse")
        .to_string()
}

// ─── Tauri Commands ───

/// List all known universes from the registry, with the active one first.
#[tauri::command]
pub fn list_universes(app: tauri::AppHandle) -> Vec<UniverseEntry> {
    let registry = load_registry(&app);
    let active_id = registry.active_id.clone();
    let mut entries = registry.entries;
    // Sort: active universe first, so the frontend tries it first on startup
    if let Some(ref aid) = active_id {
        entries.sort_by(|a, b| {
            let a_active = a.id == *aid;
            let b_active = b.id == *aid;
            b_active.cmp(&a_active)
        });
    }
    entries
}

/// Create a new universe: .constellation/ directory structure + config files.
#[tauri::command]
pub fn create_universe(
    app: tauri::AppHandle,
    name: String,
    path: String,
) -> Result<UniverseEntry, String> {
    let universe_dir = Path::new(&path).join(&name);

    if universe_dir.exists() {
        return Err("A directory with this name already exists at the chosen location.".to_string());
    }

    // Create directory structure
    let cdir = constellation_dir(&universe_dir);
    fs::create_dir_all(&cdir)
        .map_err(|e| format!("Failed to create .constellation/ directory: {}", e))?;
    fs::create_dir_all(cdir.join("bases"))
        .map_err(|e| format!("Failed to create bases directory: {}", e))?;
    fs::create_dir_all(cdir.join("templates"))
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;

    // Universe root IS the notes folder (Obsidian-style flat structure).
    // No nested subfolder — notes go directly in the universe root.

    // Write universe.json into .constellation/
    let now = chrono::Local::now().to_rfc3339();
    let meta = UniverseMeta {
        name: name.clone(),
        created: now.clone(),
        version: 2,
        children: vec![],
        notes_folder: None, // None = universe root is the library (flat)
    };
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(cdir.join("universe.json"), &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // Register universe root as the notes library
    let notes_library = crate::libraries::LibraryInfo {
        id: format!("universe_notes_{}", uuid_simple()),
        name: name.clone(),
        path: universe_dir.to_string_lossy().to_string(), // root, not nested
        is_universe_notes: true,
        canonical_mode: "native".to_string(),
    };
    let libraries_json = serde_json::to_string_pretty(&vec![&notes_library]).map_err(|e| e.to_string())?;
    fs::write(cdir.join("libraries.json"), &libraries_json)
        .map_err(|e| format!("Failed to write libraries.json: {}", e))?;
    fs::write(cdir.join("bookmarks.json"), "[]")
        .map_err(|e| format!("Failed to write bookmarks.json: {}", e))?;
    fs::write(cdir.join("settings.json"), "{}")
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;
    fs::write(cdir.join("workspaces.json"), "[]")
        .map_err(|e| format!("Failed to write workspaces.json: {}", e))?;
    fs::write(cdir.join("property-types.json"), "{}")
        .map_err(|e| format!("Failed to write property-types.json: {}", e))?;
    fs::write(cdir.join("collections.json"), "[]")
        .map_err(|e| format!("Failed to write collections.json: {}", e))?;

    // Add to registry (path = universe ROOT, not .constellation/)
    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name: name.clone(),
        path: universe_dir.to_string_lossy().to_string(),
        created: now,
    };

    let mut registry = load_registry_for_update(&app)?;
    registry.entries.push(entry.clone());
    if registry.active_id.is_none() {
        registry.active_id = Some(entry.id.clone());
    }
    save_registry(&app, &registry)?;

    Ok(entry)
}

/// Set the active universe by ID. Auto-migrates old format if needed.
// Note-open-freeze Batch-2 §B2-5 (2026-07-03): `(async)` — a universe switch
// waits on the DB writer lock (invalidate_search_state) and can sit behind a
// mid-flight backfill/reindex; as SYNC that wait froze the whole app. The body
// stays a blocking fn (std Mutex guards — no .await may cross them); the
// switch_lock below restores, off the main thread, the whole-switch
// serialization SYNC dispatch used to provide (double-checked with the §A
// already-active guard, which re-runs under the lock).
#[tauri::command(async)]
pub fn set_active_universe(app: tauri::AppHandle, id: String) -> Result<(), String> {
    // App-freeze audit R2 (2026-07-04): phase timing measured this command at
    // ~5ms end-to-end on a settled switch (journal `uswitch:*` markers, since
    // removed) — the switch command is NOT the felt lag. The residual "switch
    // back before it settles" contention is the DEPARTING universe's still-
    // running async boot warming, not this body; its own reproduce-first pass.
    // Batch-2 §B2-5 — one switch at a time (poison-tolerant like init_lock).
    let switch_state = app.state::<UniverseState>();
    let _switch_guard = switch_state
        .switch_lock
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // MIG-079 §A — idempotent activation guard. Re-activating the universe that is
    // ALREADY active is a no-op: do NOT tear down + rebuild the search DB. Without
    // this, the main window's boot restore AND the second screen both call this for
    // the SAME universe, and the second call's invalidate_search_state bumps the
    // federation generation → the in-flight init_db's connection is discarded and a
    // full SECOND init_db + boot-graph recompute runs (the observed double-init,
    // ~+34s). We compare the requested universe's registry path to the active path:
    // a genuine SWITCH has a different path and falls through; a cold start has
    // active_path = None and falls through. The search-DB lifecycle is owned by
    // ensure_search_db_ready independently, so skipping the redundant re-activation
    // never leaves the DB unopened.
    {
        let registry = load_registry(&app);
        if let Some(entry) = registry.entries.iter().find(|e| e.id == id) {
            let requested = PathBuf::from(&entry.path);
            let state = app.state::<UniverseState>();
            let already_active = state
                .active_path
                .lock()
                .map(|g| g.as_deref() == Some(requested.as_path()))
                .unwrap_or(false);
            if already_active {
                return Ok(());
            }
        }
    }
    let mut registry = load_registry_for_update(&app)?;

    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| "Universe not found in registry.".to_string())?
        .clone();

    let mut universe_path = PathBuf::from(&entry.path);
    if !universe_path.is_dir() {
        // Path doesn't exist — check if it was consolidated (CT\CT → CT)
        // by looking for .constellation in the parent directory
        let mut healed = false;
        if let Some(parent) = universe_path.parent() {
            if parent.is_dir() && constellation_dir(parent).join("universe.json").exists() {
                eprintln!("[universe] Healing stale path: {} → {}", universe_path.display(), parent.display());
                universe_path = parent.to_path_buf();
                // Update registry with healed path
                for e in &mut registry.entries {
                    if e.id == id { e.path = universe_path.to_string_lossy().to_string(); }
                }
                let _ = save_registry(&app, &registry);
                healed = true;
            }
        }
        if !healed {
            return Err(format!("Universe directory does not exist: {}", entry.path));
        }
    }

    // Auto-migrate old flat format to .constellation/
    migrate_to_constellation(&universe_path)?;

    // Ensure universe notes folder exists (migration for existing universes)
    ensure_universe_notes_folder(&universe_path)?;

    // ─── Migration: consolidate same-name parent nesting ───
    // If universe is at C:\Name\Name\ and parent is C:\Name\, move everything up
    // and update the registry to point to C:\Name\.
    let mut final_path = universe_path.clone();
    if let (Some(parent), Some(dir_name)) = (universe_path.parent(), universe_path.file_name()) {
        let parent_name = parent.file_name().map(|n| n.to_string_lossy().to_string());
        let this_name = dir_name.to_string_lossy().to_string();
        if parent_name.as_deref() == Some(&this_name) && parent.join(".constellation").exists() == false {
            // Parent has same name and no .constellation of its own → consolidate
            eprintln!("[universe] Consolidating nested universe: {} → {}", universe_path.display(), parent.display());
            // Move .constellation/ and all contents up to parent.
            // 2026-07-25 PJ-140 #33: log a failed move instead of swallowing it — the
            // critical entry is .constellation/ (universe.json, libraries.json,
            // settings.json, search.db). If IT fails to move, repointing the registry
            // at the parent would leave the universe pointing at a directory WITHOUT
            // its config (silent breakage). So the repoint below is gated on the move
            // actually landing.
            if let Ok(entries) = fs::read_dir(&universe_path) {
                for entry in entries.flatten() {
                    let src = entry.path();
                    let dest = parent.join(entry.file_name());
                    if !dest.exists() {
                        if let Err(e) = fs::rename(&src, &dest) {
                            eprintln!("[universe] Consolidation move failed for {}: {}", src.display(), e);
                        }
                    }
                }
            }
            // Only repoint if the critical config directory actually made it up. If it
            // did NOT, keep the nested path (final_path is already universe_path) so the
            // universe stays openable at its original, config-bearing location rather than
            // being repointed at a parent with no .constellation.
            if parent.join(".constellation").exists() {
                // Remove the now-empty nested directory
                let _ = fs::remove_dir(&universe_path);

                // Update registry path
                let parent_str = parent.to_string_lossy().to_string();
                for e in &mut registry.entries {
                    if e.id == id {
                        e.path = parent_str.clone();
                    }
                }
                // Update library paths in .constellation/libraries.json
                let cdir = constellation_dir(parent);
                let libs_path = cdir.join("libraries.json");
                if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                    if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                        for lib in &mut libs {
                            if lib.is_universe_notes {
                                lib.path = parent_str.clone();
                            }
                        }
                        if let Ok(json) = serde_json::to_string_pretty(&libs) {
                            persist_json_best_effort(&libs_path, &json);
                        }
                    }
                }
                final_path = parent.to_path_buf();
                eprintln!("[universe] Consolidation complete: {}", final_path.display());
            } else {
                eprintln!(
                    "[universe] Consolidation ABORTED: .constellation did not land at {} — keeping the nested path so the universe stays openable.",
                    parent.display()
                );
            }
        }
    }

    // ─── Fix stale library paths on every activation ───
    // If the universe was moved, libraries.json still has old absolute paths.
    // Fix is_universe_notes to point to current root, and resolve other stale paths.
    let cdir_fix = constellation_dir(&final_path);
    let libs_fix_path = cdir_fix.join("libraries.json");
    if libs_fix_path.exists() {
        if let Ok(libs_data) = fs::read_to_string(&libs_fix_path) {
            if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                let root_str = final_path.to_string_lossy().to_string();
                let mut changed = false;
                for lib in &mut libs {
                    if lib.is_universe_notes && lib.path != root_str {
                        eprintln!("[universe] Fixing universe notes path: {} → {}", lib.path, root_str);
                        lib.path = root_str.clone();
                        changed = true;
                    } else if !lib.is_universe_notes && !Path::new(&lib.path).exists() {
                        if let Some(folder_name) = Path::new(&lib.path).file_name() {
                            let candidate = final_path.join(folder_name);
                            if candidate.is_dir() {
                                let new_path = candidate.to_string_lossy().to_string();
                                eprintln!("[universe] Fixing library path: {} → {}", lib.path, new_path);
                                lib.path = new_path;
                                changed = true;
                            }
                        }
                    }
                }
                if changed {
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        persist_json_best_effort(&libs_fix_path, &json);
                    }
                }
            }
        }
    }


    // Update managed state
    let state = app.state::<UniverseState>();
    let mut lock = state.active_path.lock().map_err(|e| e.to_string())?;
    *lock = Some(final_path.clone());
    drop(lock); // release the pointer mutex before the (cheap, fail-fast) lock acquire
    // MIG-111 Phase 0.2 (R5) — take the OS owner lock for the newly-active universe
    // (releases the previous one). Record-only until Phase 1.4 flips enforcement.
    crate::universe_lock::activate(&final_path);

    // Invalidate the libraries cache — switching universes means the
    // libraries list is completely different now.
    crate::libraries::invalidate_libraries_cache();

    // MIG-055 §H audit hotfix — also invalidate the search-DB connection
    // so the next `ensure_search_db_ready` opens the DB at the NEW
    // universe's path. Surfaced by the §H migration-path audit (Scenario
    // 10): without this reset, `ensure_search_db_ready` early-returns on
    // `state.db.is_some()` and the new universe's
    // `init_five_acts_system_notes` is silently skipped until app
    // restart. Closes a pre-existing latent bug that MIG-055 was the
    // first feature to expose.
    crate::search::invalidate_search_state(&app);

    // M8b: load this Universe's Arabic user-override file into the
    // process-wide active store. Consumed by every subsequent FTS5
    // tokenizer call via `arabic::overrides::active()`. Errors are
    // logged but NOT propagated — a malformed overrides.json must not
    // prevent the user from switching Universes. The engine gracefully
    // falls back to no-overrides on error, and the Settings UI will
    // surface the parse error when the user opens the overrides panel.
    //
    // M8b-v2: also enumerate `UniverseMeta::children` so any cUniverse
    // child overrides are stacked under the parent's sovereign layer.
    // Lookup walks parent → children on normalize-miss; parent wins on
    // conflict. A non-federated Universe (no children) collapses to
    // the pre-v2 single-layer behaviour byte-for-byte.
    let child_universe_roots = resolve_child_universe_roots(&final_path);
    match crate::arabic::overrides::activate_layered_for_universe(
        &final_path,
        &child_universe_roots,
    ) {
        Ok(count) if count > 0 => {
            eprintln!(
                "[arabic] Loaded {} Arabic override(s) for Universe at {} ({} child Universe{} stacked)",
                count,
                final_path.display(),
                child_universe_roots.len(),
                if child_universe_roots.len() == 1 { "" } else { "s" },
            );
        }
        Ok(_) => {} // no overrides authored yet — common case, silent
        Err(e) => {
            eprintln!("[arabic] Failed to load overrides for Universe at {}: {}",
                      final_path.display(), e);
            // Install an empty store so any residual from a previous
            // active Universe doesn't leak across the switch.
            crate::arabic::overrides::clear_active();
        }
    }


    // Update registry
    registry.active_id = Some(id);
    save_registry(&app, &registry)?;

    Ok(())
}

/// Get the current active universe path.
#[tauri::command]
pub fn get_active_universe_path(app: tauri::AppHandle) -> Option<String> {
    let state = app.state::<UniverseState>();
    let lock = state.active_path.lock().ok()?;
    lock.as_ref().map(|p| p.to_string_lossy().to_string())
}

/// Remove a universe from the registry (does NOT delete files).
#[tauri::command]
pub fn remove_universe_from_registry(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut registry = load_registry_for_update(&app)?;
    registry.entries.retain(|e| e.id != id);
    if registry.active_id.as_deref() == Some(&id) {
        registry.active_id = registry.entries.first().map(|e| e.id.clone());
    }
    save_registry(&app, &registry)
}

/// Rename the active universe — updates registry, universe.json, notes folder, and library entry.
#[tauri::command]
pub fn rename_universe(app: tauri::AppHandle, new_name: String) -> Result<(), String> {
    let universe_dir = active_universe_dir(&app)?;
    let cdir = constellation_dir(&universe_dir);

    // 1. Read current universe.json
    let meta_path = cdir.join("universe.json");
    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    let old_name = meta.name.clone();

    // 2. Rename the notes folder on disk (only if a subfolder exists — legacy nested layout)
    if let Some(ref old_folder) = meta.notes_folder {
        let old_path = universe_dir.join(old_folder);
        let new_path = universe_dir.join(&new_name);
        if old_path.exists() && !new_path.exists() {
            fs::rename(&old_path, &new_path)
                .map_err(|e| format!("Failed to rename notes folder: {}", e))?;
        }
        // Update the library entry path
        let libs_path = cdir.join("libraries.json");
        if libs_path.exists() {
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    for lib in &mut libs {
                        if lib.is_universe_notes {
                            lib.name = new_name.clone();
                            lib.path = new_path.to_string_lossy().to_string();
                        }
                    }
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        persist_json_best_effort(&libs_path, &json);
                    }
                }
            }
        }
    } else {
        // Flat layout (notes_folder = None): just update the library name
        let libs_path = cdir.join("libraries.json");
        if libs_path.exists() {
            if let Ok(libs_data) = fs::read_to_string(&libs_path) {
                if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                    for lib in &mut libs {
                        if lib.is_universe_notes {
                            lib.name = new_name.clone();
                            // Path stays as universe root — no subfolder to rename
                        }
                    }
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        persist_json_best_effort(&libs_path, &json);
                    }
                }
            }
        }
    }

    // 3. Update universe.json (preserve notes_folder — None for flat, Some for legacy)
    meta.name = new_name.clone();
    // Don't overwrite notes_folder — keep None if flat, keep old value if legacy subfolder
    if meta.notes_folder.is_some() {
        meta.notes_folder = Some(new_name.clone());
    }
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    atomic_write(&meta_path, meta_json.as_bytes())
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // 4. Update global registry
    let mut registry = load_registry_for_update(&app)?;
    for entry in &mut registry.entries {
        if entry.path == universe_dir.to_string_lossy().to_string() {
            entry.name = new_name.clone();
        }
    }
    save_registry(&app, &registry)?;

    eprintln!("[universe] Renamed universe '{}' → '{}'", old_name, new_name);
    Ok(())
}

/// Open an existing universe directory (must contain .constellation/universe.json).
#[tauri::command]
pub fn open_existing_universe(app: tauri::AppHandle, path: String) -> Result<UniverseEntry, String> {
    let universe_dir = Path::new(&path);

    if !universe_dir.is_dir() {
        return Err("Path does not exist or is not a directory.".to_string());
    }

    // Auto-migrate if old flat format detected
    migrate_to_constellation(universe_dir)?;

    // Check for .constellation/universe.json (new format) or universe.json (fallback)
    let cdir = constellation_dir(universe_dir);
    let meta_path = if cdir.join("universe.json").exists() {
        cdir.join("universe.json")
    } else if universe_dir.join("universe.json").exists() {
        universe_dir.join("universe.json")
    } else {
        return Err("This folder does not contain a .constellation/ directory. It is not a valid Constellation universe.".to_string());
    };

    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    let mut registry = load_registry_for_update(&app)?;

    // Check for duplicates by path
    let canon = fs::canonicalize(universe_dir)
        .unwrap_or_else(|_| universe_dir.to_path_buf());
    for existing in &registry.entries {
        let existing_canon = fs::canonicalize(&existing.path)
            .unwrap_or_else(|_| PathBuf::from(&existing.path));
        if existing_canon == canon {
            // PJ-310 — REGISTER ONLY. Activation belongs to `set_active_universe`, the one door
            // that does it completely.
            //
            // This used to set the in-memory active pointer and take the OS folder lock here, and
            // that is exactly half a switch: `set_active_universe` also calls
            // `invalidate_libraries_cache` (:1051), `invalidate_search_state` (:1061) and
            // `activate_layered_for_universe` (:1077). Without the middle one,
            // `ensure_search_db_ready` early-returns on `state.db.is_some()`, so the connection to
            // the PREVIOUS universe's `search.db` stays open — the app believes it is in universe
            // B while every search and every index write goes to universe A's database. That is
            // the class MIG-111 exists to end, shipped and reachable from the Universe Manager.
            //
            // `create_universe`, the oldest and most-used door, already worked this way: it never
            // writes the pointer and leaves activation to its caller. This makes the registration
            // doors match the one that was already right, rather than inventing a new rule.
            //
            // `registry.active_id` + `save_registry` STAY: that is the user's durable intent, and
            // the correct door rewrites it anyway.
            registry.active_id = Some(existing.id.clone());
            save_registry(&app, &registry)?;
            return Ok(existing.clone());
        }
    }

    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name: meta.name.clone(),
        path: path.clone(),
        created: meta.created.clone(),
    };

    registry.entries.push(entry.clone());
    registry.active_id = Some(entry.id.clone());
    save_registry(&app, &registry)?;

    // ─── Fix library paths after universe move ───
    // libraries.json stores absolute paths from the old location. After a move,
    // update is_universe_notes library to point to the new universe root.
    // Also update any library whose old path no longer exists but whose folder
    // name exists under the new universe root.
    let libs_path = cdir.join("libraries.json");
    if libs_path.exists() {
        if let Ok(libs_data) = fs::read_to_string(&libs_path) {
            if let Ok(mut libs) = serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&libs_data) {
                let root_str = universe_dir.to_string_lossy().to_string();
                let mut changed = false;
                for lib in &mut libs {
                    let old_path = Path::new(&lib.path);
                    if lib.is_universe_notes {
                        // Universe notes library always points to root
                        if lib.path != root_str {
                            eprintln!("[universe] Fixing universe notes path: {} → {}", lib.path, root_str);
                            lib.path = root_str.clone();
                            changed = true;
                        }
                    } else if !old_path.exists() {
                        // Non-universe library with stale path — try to find it under new root
                        if let Some(folder_name) = old_path.file_name() {
                            let candidate = universe_dir.join(folder_name);
                            if candidate.is_dir() {
                                let new_path = candidate.to_string_lossy().to_string();
                                eprintln!("[universe] Fixing library path: {} → {}", lib.path, new_path);
                                lib.path = new_path;
                                changed = true;
                            }
                        }
                    }
                }
                if changed {
                    if let Ok(json) = serde_json::to_string_pretty(&libs) {
                        persist_json_best_effort(&libs_path, &json);
                    }
                }
            }
        }
    }

    // Ensure universe notes folder is registered
    ensure_universe_notes_folder(universe_dir)?;

    // PJ-310 — REGISTER ONLY; activation belongs to `set_active_universe`. See the note in the
    // already-registered branch above for why half a switch is worse than none.

    Ok(entry)
}

/// Link an existing Markdown folder as a universe.
/// Creates .constellation/ inside the folder and registers it as a single-library universe.
#[tauri::command]
pub fn link_library_as_universe(app: tauri::AppHandle, path: String) -> Result<UniverseEntry, String> {
    let library_dir = Path::new(&path);

    if !library_dir.is_dir() {
        return Err("Path does not exist or is not a directory.".to_string());
    }

    let cdir = constellation_dir(library_dir);

    // PJ-207 §15 — this gate asked the wrong question, and the create-path below ran on top
    // of a real Universe because of it.
    //
    // A universe still in the pre-`.constellation/` flat layout keeps `universe.json`,
    // `vaults.json`, `settings.json` and `collections.json` at its ROOT. The gate only asks
    // whether `.constellation/universe.json` exists, so for that universe it read false and
    // fell through to "create" — over the user's libraries, collections and settings.
    // `open_existing_universe` has always called `migrate_to_constellation` first for exactly
    // this reason; this door never did. The ordering also mattered: `migrate_to_constellation`
    // bails the moment `.constellation/` exists, so once this command created that directory
    // the legacy files were stranded at the root permanently and no later boot could adopt them.
    migrate_to_constellation(library_dir)?;

    // If .constellation/universe.json already exists (or the migration just put it there),
    // treat as "open existing"
    if cdir.join("universe.json").exists() {
        return open_existing_universe(app, path);
    }

    // Create .constellation/ inside the library folder
    fs::create_dir_all(&cdir)
        .map_err(|e| format!("Failed to create .constellation/ directory: {}", e))?;
    fs::create_dir_all(cdir.join("bases"))
        .map_err(|e| format!("Failed to create bases directory: {}", e))?;

    // Derive name from folder name
    let name = library_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let name = if name.is_empty() { "My Library".to_string() } else { name };

    let now = chrono::Local::now().to_rfc3339();

    // Write universe.json
    let meta = UniverseMeta {
        name: name.clone(),
        created: now.clone(),
        version: 2,
        children: vec![],
        notes_folder: None,
    };
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(cdir.join("universe.json"), &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // Register the folder itself as the sole library
    let lib_id = format!("library_{}", uuid_simple());
    let library_entry = crate::libraries::LibraryInfo {
        id: lib_id,
        name: name.clone(),
        path: path.clone(),
        is_universe_notes: false,
        canonical_mode: "compatible".to_string(), // linked external folder
    };
    // PJ-207 §15 — these writes were unconditional, and each of the five defaults was
    // additionally `.ok()`-ed. Whenever `.constellation/` was populated but its `universe.json`
    // was not where the gate above looks, this wrote a ONE-ENTRY `libraries.json` over a
    // registry that listed every library, and `"[]"`/`"{}"` over collections (every Starred
    // item), settings, workspaces and property types — then returned Ok, because even the I/O
    // error was discarded. The sibling `migrate_legacy_data` has guarded the identical writes
    // with `if !exists()` all along; this path never adopted it. A default belongs only where
    // there is genuinely nothing: never overwrite a file that is already on disk, and never
    // discard the error from a write we do perform.
    let libs_path = cdir.join("libraries.json");
    if !libs_path.exists() {
        let libs_json = serde_json::to_string_pretty(&vec![library_entry]).map_err(|e| e.to_string())?;
        atomic_write(&libs_path, libs_json.as_bytes())
            .map_err(|e| format!("Failed to write libraries.json: {}", e))?;
    }

    // Write empty data files — only where none exists yet.
    for (file, default) in [
        ("bookmarks.json", "[]"),
        ("settings.json", "{}"),
        ("workspaces.json", "[]"),
        ("property-types.json", "{}"),
        ("collections.json", "[]"),
    ] {
        let p = cdir.join(file);
        if !p.exists() {
            atomic_write(&p, default.as_bytes())
                .map_err(|e| format!("Failed to write {}: {}", file, e))?;
        }
    }

    // Register in global registry
    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name,
        path: path.clone(),
        created: now,
    };

    let mut registry = load_registry_for_update(&app)?;
    registry.entries.push(entry.clone());
    registry.active_id = Some(entry.id.clone());
    save_registry(&app, &registry)?;

    // PJ-310 — REGISTER ONLY; activation belongs to `set_active_universe`. Its caller
    // (`UniverseSetup.handleLinkLibrary`) already calls it immediately after this returns.

    Ok(entry)
}

/// Check if migration from legacy app_data_dir storage is needed.
#[tauri::command]
pub fn check_migration_needed(app: tauri::AppHandle) -> bool {
    let reg_path = match registry_path(&app) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let old_vaults = app
        .path()
        .app_data_dir()
        .map(|d| d.join("vaults.json"))
        .unwrap_or_default();
    !reg_path.exists() && old_vaults.exists()
}

/// Add a child universe path to the active universe's children array.
#[tauri::command]
pub fn add_child_universe(app: tauri::AppHandle, child_path: String) -> Result<(), String> {
    let cdir = active_constellation_dir(&app)?;
    let meta_path = cdir.join("universe.json");
    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    // Validate child path exists and has .constellation/universe.json (or old universe.json)
    let child_dir = Path::new(&child_path);
    let child_cdir = constellation_dir(child_dir);
    if !child_cdir.join("universe.json").exists() && !child_dir.join("universe.json").exists() {
        return Err("The selected path is not a valid universe.".to_string());
    }

    // Prevent adding self
    let universe_dir = active_universe_dir(&app)?;
    if let (Ok(self_canon), Ok(child_canon)) = (fs::canonicalize(&universe_dir), fs::canonicalize(child_dir)) {
        if self_canon == child_canon {
            return Err("A universe cannot be a child of itself.".to_string());
        }
    }

    if !meta.children.contains(&child_path) {
        meta.children.push(child_path);
    }

    let json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    atomic_write(&meta_path, json.as_bytes()).map_err(|e| format!("Failed to save universe.json: {}", e))?;
    // PJ-235 (panel condition) — linking a universe CHANGES the federated set, and
    // `LIBRARIES_CACHE` is warm from boot. Without this, `foreign_library_roots` — the set
    // `require_own_library` and every federation-aware walk boundary refuse by — stays EMPTY
    // for the rest of the session in which the user linked the universe, which is the
    // highest-risk session there is: the guard is inert exactly when it is first needed.
    crate::libraries::invalidate_libraries_cache();
    Ok(())
}

/// Remove a child universe path from the active universe's children array.
#[tauri::command]
pub fn remove_child_universe(app: tauri::AppHandle, child_path: String) -> Result<(), String> {
    let cdir = active_constellation_dir(&app)?;
    let meta_path = cdir.join("universe.json");
    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let mut meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    meta.children.retain(|c| c != &child_path);

    let json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    atomic_write(&meta_path, json.as_bytes()).map_err(|e| format!("Failed to save universe.json: {}", e))?;
    // PJ-235 — same invalidation as add_child_universe: the federated set changed.
    crate::libraries::invalidate_libraries_cache();
    Ok(())
}

/// Return the full merged library list for the active universe
/// (own + children, recursive, deduplicated).
#[tauri::command]
pub fn resolve_universe_libraries(app: tauri::AppHandle) -> Result<Vec<crate::libraries::LibraryInfo>, String> {
    let universe_dir = active_universe_dir(&app)?;
    let mut visited = Vec::new();
    Ok(resolve_libraries_recursive(&universe_dir, &mut visited))
}

/// MIG-100 — a SPECIFIC universe root's OWN libraries (its
/// `.constellation/libraries.json`, NON-recursive — deliberately WITHOUT the
/// federated cUniverse libraries, mirroring `load_libraries`' write-scope
/// discipline: an edit must never land on a read-only cUniverse file). Used
/// by write-path validation so a departing universe's note (a tab whose
/// deferred flush lands after a universe switch flipped the active pointer)
/// still validates against its OWN universe's libraries. Only reached on the
/// rare active-miss.
pub fn own_libraries_for_root(universe_root: &Path) -> Vec<crate::libraries::LibraryInfo> {
    let libs_path = constellation_dir(universe_root).join("libraries.json");
    match fs::read_to_string(&libs_path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => vec![],
    }
}

/// MIG-100 — the registered universe roots (registry entries only; the active
/// universe first via list ordering is irrelevant here). Used by the write
/// validator's cross-universe fallback.
pub fn registered_universe_roots(app: &tauri::AppHandle) -> Vec<PathBuf> {
    load_registry(app)
        .entries
        .into_iter()
        .map(|e| PathBuf::from(e.path))
        .collect()
}

/// PJ-322 — the same list, but **it never reports "no universes" when it means "I could not
/// look"** (Boss-approved 2026-08-20, decision 1).
///
/// `registered_universe_roots` goes through the lenient `load_registry`, which returns an empty
/// registry for a path error, a read failure, and a parse failure alike. That is correct for a
/// *display* consumer — a list showing nothing is a display problem. It is wrong for
/// `mig108::assemble_foreign_roots`, which uses this list to decide which folders are **another
/// universe's** and therefore must never be relocated: an empty answer there means *nothing is
/// foreign*, and every external registered library becomes a candidate to move.
///
/// This reuses `load_registry_for_update`'s split verbatim rather than inventing a third
/// policy: **`Unreadable` refuses** (transient — an antivirus or sync tool holding the file;
/// a retry will succeed), while **`Corrupt` sets the unusable file aside and proceeds from
/// empty** — which is then TRUE rather than assumed, and preserves the bytes for recovery.
/// That split is a recorded scar (see the comment on `load_registry_for_update`): the first
/// version returned `Err` for both and locked the user out of the app entirely. Refusing to
/// overwrite must never become refusing to continue.
pub fn registered_universe_roots_strict(app: &tauri::AppHandle) -> Result<Vec<PathBuf>, String> {
    Ok(load_registry_for_update(app)?
        .entries
        .into_iter()
        .map(|e| PathBuf::from(e.path))
        .collect())
}

/// Info about a child universe — name, path, and how many libraries it contributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildUniverseInfo {
    pub name: String,
    pub path: String,
    pub library_count: u32,
}

/// Return info about child universes of the active universe. Reads
/// `universe.json` and then, for each child, reads the child's universe.json
/// + libraries.json to count libraries — small files, but a handful of
/// synchronous filesystem round-trips per child.
///
/// Called on boot from `DashboardView.onMount` →
/// `loadDashboardData()` → `getChildUniverses()` → this command. Because
/// DashboardView mounts the instant `libraries.set(bundle.libraries)` fires
/// (before `cache_boot_snapshot_core` returns), a sync `#[tauri::command]`
/// binding would queue this work on the WebView2 UI thread and block the
/// core snapshot. Converting to `#[tauri::command(async)]` offloads dispatch
/// to Tokio workers — see `watcher.rs` docstring for the full chain, and
/// `libraries.rs::scan_library_tags` for the boot-fan-out context.
#[tauri::command(async)]
pub fn get_child_universes(app: tauri::AppHandle) -> Result<Vec<ChildUniverseInfo>, String> {
    let cdir = active_constellation_dir(&app)?;
    let meta_path = cdir.join("universe.json");

    if !meta_path.exists() {
        return Ok(vec![]);
    }

    let data = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read universe.json: {}", e))?;
    let meta: UniverseMeta = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse universe.json: {}", e))?;

    let mut children = Vec::new();
    for child_path_str in &meta.children {
        let child_path = Path::new(child_path_str);
        let child_cdir = constellation_dir(child_path);

        // Try .constellation/universe.json first, then old flat format
        let child_meta_path = if child_cdir.join("universe.json").exists() {
            child_cdir.join("universe.json")
        } else {
            child_path.join("universe.json")
        };

        let name = if child_meta_path.exists() {
            if let Ok(child_data) = fs::read_to_string(&child_meta_path) {
                if let Ok(child_meta) = serde_json::from_str::<UniverseMeta>(&child_data) {
                    child_meta.name
                } else {
                    child_path.file_name().unwrap_or_default().to_string_lossy().to_string()
                }
            } else {
                child_path.file_name().unwrap_or_default().to_string_lossy().to_string()
            }
        } else {
            child_path.file_name().unwrap_or_default().to_string_lossy().to_string()
        };

        // Count libraries in child
        let libs_path = if child_cdir.join("libraries.json").exists() {
            child_cdir.join("libraries.json")
        } else {
            child_path.join("vaults.json")
        };
        let library_count = if libs_path.exists() {
            if let Ok(vdata) = fs::read_to_string(&libs_path) {
                serde_json::from_str::<Vec<crate::libraries::LibraryInfo>>(&vdata)
                    .map(|v| v.len() as u32)
                    .unwrap_or(0)
            } else { 0 }
        } else { 0 };

        children.push(ChildUniverseInfo {
            name,
            path: child_path_str.clone(),
            library_count,
        });
    }

    Ok(children)
}

/// Read library list from a child universe path (reads its
/// `.constellation/libraries.json`). Small file, but DashboardView calls this
/// **once per child universe** in a sequential `for` loop after
/// `getChildUniverses` resolves (src/lib/components/DashboardView.svelte
/// loadDashboardData). Same UI-thread-serialization concern as
/// `get_child_universes` above — see that docstring + `watcher.rs` for full
/// rationale.
#[tauri::command(async)]
pub fn read_child_universe_libraries(_app: tauri::AppHandle, child_path: String) -> Result<Vec<crate::libraries::LibraryInfo>, String> {
    let cp = Path::new(&child_path);
    let cdir = constellation_dir(cp);

    let libs_path = if cdir.join("libraries.json").exists() {
        cdir.join("libraries.json")
    } else {
        cp.join("vaults.json")
    };

    if !libs_path.exists() {
        return Ok(vec![]);
    }

    let data = fs::read_to_string(&libs_path)
        .map_err(|e| format!("Failed to read libraries.json: {}", e))?;
    let libs: Vec<crate::libraries::LibraryInfo> = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse libraries.json: {}", e))?;
    Ok(libs)
}

// ─── Data File I/O Commands ───
// All data files live inside .constellation/

/// Read settings.json from the active universe (or an explicit root — the
/// session.json precedent, mirrored on the load side 2026-08-01).
#[tauri::command]
pub fn read_universe_settings(app: tauri::AppHandle, universe_root: Option<String>) -> Result<serde_json::Value, String> {
    let dir = resolve_constellation_dir(&app, universe_root)?;
    let path = dir.join("settings.json");
    if path.exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse settings: {}", e))
    } else {
        Ok(serde_json::Value::Object(serde_json::Map::new()))
    }
}

/// Save settings.json to the active universe.
// MIG-100 inspection fix (freeze class): (async) — these persisted-JSON saves
// now fsync (hardened atomic_write); a sync command would park the WebView2
// dispatch thread for the fsync (100ms–seconds on network/USB/AV-scanned
// disks). Same one-word fix as the read commands above (universe.rs Batch-S).
// 2026-08-01 safety inspection (cross-window-clobber class): optional explicit
// universe_root — the frontend writer is a 300ms debounce that can fire after
// set_active_universe flips the pointer; resolve_constellation_dir pins the
// write to the universe the save was composed in (session.json precedent).
#[tauri::command(async)]
pub fn save_universe_settings(app: tauri::AppHandle, settings: serde_json::Value, universe_root: Option<String>) -> Result<(), String> {
    let dir = resolve_constellation_dir(&app, universe_root)?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    atomic_write(&dir.join("settings.json"), json.as_bytes())
        .map_err(|e| format!("Failed to save settings: {}", e))
}

/// Read bookmarks.json from the active universe. MIG-092: READ-ONLY now — the
/// only reader is the one-time Bookmarks→Starred migration (loadCollections);
/// nothing writes bookmarks.json anymore (it is retained as a backup).
/// Missing file → empty array.
#[tauri::command]
pub fn read_universe_bookmarks(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = active_constellation_dir(&app)?;
    let path = dir.join("bookmarks.json");
    if path.exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read bookmarks: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse bookmarks: {}", e))
    } else {
        Ok(serde_json::Value::Array(vec![]))
    }
}

/// The ONE writer bookmarks.json is allowed to have: the first-run localStorage migration.
///
/// PJ-207 §15 — `UniverseSetup.migrateLocalStorage` has been invoking `save_universe_bookmarks`,
/// a command MIG-092 retired and which is registered nowhere. That leg therefore failed 100% of
/// the time into a `catch` that only reaches a console a release build does not have. The user's
/// legacy bookmarks stayed in localStorage — the store PJ-110 proved a leveldb orphan can wipe
/// wholesale — and the ONE-TIME Bookmarks→Starred adoption in `loadCollections` then spent its
/// only slot against an empty file (`CL.migrateBookmarks` seeds a Starred collection even from an
/// empty array, and `saveCollections` persists it, so it never re-runs).
///
/// MIG-092's ruling still stands: this is NOT a general writer. It refuses a bookmarks.json that
/// already holds anything, so it can never overwrite the retained backup, and it reads through
/// `read_persisted_json` so an unreadable file refuses instead of being taken for empty.
#[tauri::command(async)]
pub fn migrate_universe_bookmarks(app: tauri::AppHandle, bookmarks: serde_json::Value) -> Result<(), String> {
    let dir = active_constellation_dir(&app)?;
    let path = dir.join("bookmarks.json");
    match read_persisted_json::<serde_json::Value>(&path) {
        Ok(None) => {}
        Ok(Some(serde_json::Value::Array(a))) if a.is_empty() => {}
        Ok(Some(_)) => {
            return Err(format!(
                "{} already holds bookmarks — refusing to overwrite them.",
                path.display()
            ))
        }
        Err(e) => return Err(e.message().to_string()),
    }
    let json = serde_json::to_string_pretty(&bookmarks).map_err(|e| e.to_string())?;
    atomic_write(&path, json.as_bytes())
        .map_err(|e| format!("Failed to save bookmarks: {}", e))
}

/// Read workspaces.json from the active universe (or an explicit root — the
/// session.json precedent, mirrored on the load side 2026-08-01).
#[tauri::command]
pub fn read_universe_workspaces(app: tauri::AppHandle, universe_root: Option<String>) -> Result<serde_json::Value, String> {
    let dir = resolve_constellation_dir(&app, universe_root)?;
    let path = dir.join("workspaces.json");
    if path.exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read workspaces: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse workspaces: {}", e))
    } else {
        Ok(serde_json::Value::Array(vec![]))
    }
}

/// Save workspaces.json to the active universe.
// MIG-100 inspection fix (freeze class): (async) — these persisted-JSON saves
// now fsync (hardened atomic_write); a sync command would park the WebView2
// dispatch thread for the fsync (100ms–seconds on network/USB/AV-scanned
// disks). Same one-word fix as the read commands above (universe.rs Batch-S).
// 2026-08-01 safety inspection (cross-window-clobber class): optional explicit
// universe_root — the frontend writer is fire-and-forget and can land after a
// universe switch; resolve_constellation_dir pins the write to the universe
// the save was composed in (session.json precedent).
#[tauri::command(async)]
pub fn save_universe_workspaces(app: tauri::AppHandle, workspaces: serde_json::Value, universe_root: Option<String>) -> Result<(), String> {
    let dir = resolve_constellation_dir(&app, universe_root)?;
    let json = serde_json::to_string_pretty(&workspaces).map_err(|e| e.to_string())?;
    atomic_write(&dir.join("workspaces.json"), json.as_bytes())
        .map_err(|e| format!("Failed to save workspaces: {}", e))
}

// ─── MIG-100 — Auto-session (session.json) ───
//
// The auto-restore-tabs snapshot. Deliberately a SEPARATE file from
// workspaces.json: this one is machine-written every ~1s of tab churn and
// disposable, while workspaces.json holds the user's named snapshots — a bug
// in the high-frequency path must never be able to clobber the precious file.
// Both commands take an EXPLICIT universe root instead of the ambient
// active_constellation_dir: the active pointer flips BEFORE the frontend
// switch handler runs (UniverseManager awaits set_active_universe first), so
// an ambient-keyed save racing a universe switch would write universe A's
// tabs into universe B's file.

/// Track which universe roots have had their session rotated this process
/// lifetime. `.prev` must be LAST LAUNCH's final state (the Firefox
/// previous.jsonlz4 pattern) — rotating on every save would make it a ~1s
/// stale sibling, propagating a bad snapshot into both generations within
/// seconds. Keyed per root so a mid-session universe switch still rotates
/// the other universe's file exactly once.
fn session_rotate_once(dir: &Path) -> bool {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static ROTATED: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    let set = ROTATED.get_or_init(|| Mutex::new(HashSet::new()));
    let key = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let mut guard = set.lock().unwrap_or_else(|p| p.into_inner());
    guard.insert(key)
}

/// Read the auto-session snapshot for an explicit universe root.
/// Missing/corrupt current generation → try session.prev.json → null.
/// Absence or corruption is NEVER an Err: a bad snapshot means "no session",
/// not a boot failure. (A missing current with a live .prev is the
/// crash-between-rotate-and-write window — .prev is the last good state.)
// (async): file reads on a slow/remote disk must not park the WebView2
// dispatch thread (same rationale as this file's other read commands).
#[tauri::command(async)]
pub fn read_universe_session(universe_root: String) -> Result<serde_json::Value, String> {
    let dir = constellation_dir(Path::new(&universe_root));
    for name in ["session.json", "session.prev.json"] {
        if let Ok(data) = fs::read_to_string(dir.join(name)) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                if !v.is_null() {
                    return Ok(v);
                }
            }
        }
    }
    Ok(serde_json::Value::Null)
}

/// Save (or delete) the auto-session snapshot for an explicit universe root.
/// `session: null` deletes BOTH generations — the toggle-off "stop
/// remembering" primitive. First save per launch rotates current → .prev.
// MIG-100 inspection fix (freeze class): (async) — these persisted-JSON saves
// now fsync (hardened atomic_write); a sync command would park the WebView2
// dispatch thread for the fsync (100ms–seconds on network/USB/AV-scanned
// disks). Same one-word fix as the read commands above (universe.rs Batch-S).
#[tauri::command(async)]
pub fn save_universe_session(universe_root: String, session: serde_json::Value) -> Result<(), String> {
    let dir = constellation_dir(Path::new(&universe_root));
    let current = dir.join("session.json");
    if session.is_null() {
        // Toggle-off "stop remembering": a deletion that FAILS must say so —
        // returning Ok while the tab list survives on disk would be a false
        // success (the frontend reverts the toggle on Err). Missing = fine.
        for name in ["session.json", "session.prev.json"] {
            match fs::remove_file(dir.join(name)) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("Failed to delete {}: {}", name, e)),
            }
        }
        return Ok(());
    }
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create .constellation: {}", e))?;
    if session_rotate_once(&dir) && current.exists() {
        // Best-effort: a failed rotation must not block the save — atomic_write
        // below still protects the current generation.
        let _ = fs::rename(&current, dir.join("session.prev.json"));
    }
    let json = serde_json::to_string(&session).map_err(|e| e.to_string())?;
    atomic_write(&current, json.as_bytes()).map_err(|e| format!("Failed to save session: {}", e))
}

/// MIG-092 §1 — read collections.json from the active universe (Collections'
/// membership: `[{id,name,created,items:[{type?,cid?,path,name?,library_name?,addedAt,done?}]}]`
/// — membership only; every displayed note fact is re-read from the index at
/// hydration, never cached here). Missing file → empty array, same contract as
/// bookmarks.
///
/// One-time migration: a legacy MIG-090 `workbench.json` (identical shape —
/// `type` simply defaults to `note` when absent) is adopted into
/// `collections.json` and retained as `workbench.json.migrated`. Idempotent:
/// runs only when `collections.json` does not yet exist, so once adopted it is
/// never re-read; the retained backup keeps the change reversible.
// (or an explicit root — the session.json precedent, mirrored on the load side
// 2026-08-01.)
/// Adopt a legacy `workbench.json` into `collections.json`, exactly once.
///
/// `Ok(None)` means there was nothing to adopt. **`Err` means there was something and we could not
/// read it — and that distinction is the whole point of this function existing.**
///
/// ## Why this is a separate, strict, parse-first function (PJ-281, tenth sweep)
///
/// The gate is `!collections.json.exists()`, so this runs ONCE per universe, ever. Two defects
/// turned that one shot into permanent loss of the user's entire Workbench collection set:
///
/// 1. **The read was swallowed.** `if let Ok(data) = read_to_string(&legacy)` dropped any transient
///    failure — an AV scanner or sync client holding the file for that one read, the exact Windows
///    sharing-violation case `read_persisted_json` was written to refuse — and fell through to
///    `Ok([])`. The frontend cannot tell that from a genuinely empty store: it seeds a Starred
///    collection and persists it, which CREATES `collections.json` and closes this gate forever.
///    `workbench.json` then sits on disk unadopted, with no error, no log, and no `.migrated`
///    marker. Routing through `read_persisted_json` makes the refusal structural — only NotFound
///    is trustworthy emptiness — and the frontend's error path (which does not seed, and disables
///    saves) already handles the `Err` correctly.
///
/// 2. **It wrote before it parsed.** The old order `atomic_write(legacy_bytes)` → rename →
///    `from_str` meant a CORRUPT `workbench.json` was copied verbatim into `collections.json`, the
///    legacy retired to `.migrated`, and only then did the parse fail. Every later boot then found
///    `collections.json` present (the gate closed), failed to parse it, and returned Err — the
///    user's collections gone AND collections unloadable at all. Parsing first makes the adopt
///    all-or-nothing: nothing is written unless the payload is known good.
///
/// Two earlier passes (Safety Audit G6 W1-9, then PJ-140) hardened the WRITE side of this same
/// block — atomic_write, fsync-before-rename, retire-only-on-commit — and both left the read.
fn adopt_legacy_collections(dir: &Path, path: &Path) -> Result<Option<serde_json::Value>, String> {
    let legacy = dir.join("workbench.json");
    let parsed: serde_json::Value = match read_persisted_json(&legacy) {
        // Genuinely absent — nothing to adopt, and the caller's empty answer is the truth.
        Ok(None) => return Ok(None),
        Ok(Some(v)) => v,
        Err(e) => {
            return Err(format!(
                "Legacy Workbench collections could not be read, so they were NOT adopted \
                 (they are still on disk and will be retried): {}",
                String::from(e)
            ))
        }
    };
    let json = serde_json::to_string_pretty(&parsed)
        .map_err(|e| format!("Failed to re-serialize legacy collections: {}", e))?;
    // Retire the legacy file ONLY if the adopt committed; otherwise it stays and next boot retries.
    match atomic_write(path, json.as_bytes()) {
        Ok(_) => {
            let _ = fs::rename(&legacy, dir.join("workbench.json.migrated"));
        }
        Err(e) => {
            eprintln!("[collections] legacy adoption failed (will retry next boot): {}", e);
        }
    }
    Ok(Some(parsed))
}

#[tauri::command]
pub fn read_universe_collections(app: tauri::AppHandle, universe_root: Option<String>) -> Result<serde_json::Value, String> {
    let dir = resolve_constellation_dir(&app, universe_root)?;
    let path = dir.join("collections.json");
    if !path.exists() {
        if let Some(adopted) = adopt_legacy_collections(&dir, &path)? {
            return Ok(adopted);
        }
        return Ok(serde_json::Value::Array(vec![]));
    }
    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read collections: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse collections: {}", e))
}

/// MIG-092 §1 — save collections.json to the active universe.
// MIG-100 inspection fix (freeze class): (async) — these persisted-JSON saves
// now fsync (hardened atomic_write); a sync command would park the WebView2
// dispatch thread for the fsync (100ms–seconds on network/USB/AV-scanned
// disks). Same one-word fix as the read commands above (universe.rs Batch-S).
// 2026-08-01 safety inspection (cross-window-clobber class): optional explicit
// universe_root — a collections save chained in universe A that lands after
// set_active_universe flips the pointer would write A's whole collections list
// over universe B's collections.json; resolve_constellation_dir pins the write
// to the universe the save was composed in (session.json precedent).
#[tauri::command(async)]
pub fn save_universe_collections(app: tauri::AppHandle, collections: serde_json::Value, universe_root: Option<String>) -> Result<(), String> {
    let dir = resolve_constellation_dir(&app, universe_root)?;
    let json = serde_json::to_string_pretty(&collections).map_err(|e| e.to_string())?;
    atomic_write(&dir.join("collections.json"), json.as_bytes())
        .map_err(|e| format!("Failed to save collections: {}", e))
}

/// Read property-types.json from the active universe (or an explicit root —
/// the session.json precedent, mirrored on the load side 2026-08-01).
#[tauri::command]
pub fn read_universe_property_types(app: tauri::AppHandle, universe_root: Option<String>) -> Result<serde_json::Value, String> {
    let dir = resolve_constellation_dir(&app, universe_root)?;
    let path = dir.join("property-types.json");
    if path.exists() {
        let data = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read property types: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse property types: {}", e))
    } else {
        Ok(serde_json::Value::Object(serde_json::Map::new()))
    }
}

/// Save property-types.json to the active universe.
// MIG-100 inspection fix (freeze class): (async) — these persisted-JSON saves
// now fsync (hardened atomic_write); a sync command would park the WebView2
// dispatch thread for the fsync (100ms–seconds on network/USB/AV-scanned
// disks). Same one-word fix as the read commands above (universe.rs Batch-S).
// 2026-08-01 safety inspection (cross-window-clobber class): optional explicit
// universe_root — the frontend writer is a 500ms debounce that can fire after
// set_active_universe flips the pointer; resolve_constellation_dir pins the
// write to the universe the save was composed in (session.json precedent).
#[tauri::command(async)]
pub fn save_universe_property_types(app: tauri::AppHandle, types: serde_json::Value, universe_root: Option<String>) -> Result<(), String> {
    let dir = resolve_constellation_dir(&app, universe_root)?;
    let json = serde_json::to_string_pretty(&types).map_err(|e| e.to_string())?;
    atomic_write(&dir.join("property-types.json"), json.as_bytes())
        .map_err(|e| format!("Failed to save property types: {}", e))
}

// ─── Legacy Migration ───

/// Migrate legacy data from app_data_dir to a new universe directory.
#[tauri::command]
pub fn migrate_legacy_data(app: tauri::AppHandle, name: String, universe_path: String) -> Result<UniverseEntry, String> {
    let universe_dir = PathBuf::from(&universe_path);
    let app_dir = app.path().app_data_dir()
        .map_err(|_| "Failed to get app data dir.".to_string())?;

    // Create universe directory structure with .constellation/
    let cdir = constellation_dir(&universe_dir);
    fs::create_dir_all(&cdir)
        .map_err(|e| format!("Failed to create .constellation/ directory: {}", e))?;
    fs::create_dir_all(cdir.join("bases"))
        .map_err(|e| format!("Failed to create bases directory: {}", e))?;

    // Copy vaults.json → .constellation/libraries.json
    let old_vaults = app_dir.join("vaults.json");
    if old_vaults.exists() {
        fs::copy(&old_vaults, cdir.join("libraries.json"))
            .map_err(|e| format!("Failed to copy vaults.json: {}", e))?;
    } else {
        fs::write(cdir.join("libraries.json"), "[]").ok();
    }

    // Copy bases directory contents
    let old_bases = app_dir.join("bases");
    if old_bases.is_dir() {
        let target_bases = cdir.join("bases");
        if let Ok(entries) = fs::read_dir(&old_bases) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.is_file() {
                    let dest = target_bases.join(entry.file_name());
                    fs::copy(&src, &dest).ok();
                }
            }
        }
    }

    // Write universe.json
    let now = chrono::Local::now().to_rfc3339();
    let name = if name.trim().is_empty() { "My Universe".to_string() } else { name };
    let meta = UniverseMeta {
        name: name.clone(),
        created: now.clone(),
        version: 2,
        children: vec![],
        notes_folder: None,
    };
    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    fs::write(cdir.join("universe.json"), &meta_json)
        .map_err(|e| format!("Failed to write universe.json: {}", e))?;

    // Write empty data files
    if !cdir.join("bookmarks.json").exists() {
        fs::write(cdir.join("bookmarks.json"), "[]").ok();
    }
    if !cdir.join("settings.json").exists() {
        fs::write(cdir.join("settings.json"), "{}").ok();
    }
    if !cdir.join("workspaces.json").exists() {
        fs::write(cdir.join("workspaces.json"), "[]").ok();
    }
    if !cdir.join("property-types.json").exists() {
        fs::write(cdir.join("property-types.json"), "{}").ok();
    }

    // Create registry with this universe
    let entry = UniverseEntry {
        id: format!("universe_{}", uuid_simple()),
        name,
        path: universe_dir.to_string_lossy().to_string(),
        created: now,
    };

    let registry = UniverseRegistry {
        entries: vec![entry.clone()],
        active_id: Some(entry.id.clone()),
    };
    save_registry(&app, &registry)?;

    // Set as active
    let state = app.state::<UniverseState>();
    let mut lock = state.active_path.lock().map_err(|e| e.to_string())?;
    *lock = Some(universe_dir.clone());
    crate::universe_lock::activate(&universe_dir); // MIG-111 0.2 (R5)

    Ok(entry)
}

/// Scaffold a starter PKM structure in a library folder.
/// Creates Atlas/, Calendar/, Efforts/, + (inbox), and a Welcome.md note.
#[tauri::command]
pub fn scaffold_starter_library(library_path: String) -> Result<(), String> {
    let root = Path::new(&library_path);
    if !root.exists() {
        return Err("Library path does not exist.".to_string());
    }

    let folders = ["Atlas", "Calendar", "Efforts", "+"];
    for folder in &folders {
        let dir = root.join(folder);
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create {}: {}", folder, e))?;
    }

    let welcome_path = root.join("Welcome.md");
    if !welcome_path.exists() {
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let content = format!(
            "---\ncreated: {}\nstatus: seedling\n---\n\n# Welcome to Constellation\n\nYour knowledge universe is ready. Here's a quick guide to get started:\n\n## Folder Structure\n\n- **Atlas** — Maps of Content, dashboards, and indexes\n- **Calendar** — Daily notes and time-based entries\n- **Efforts** — Active projects and tasks\n- **+** — Quick capture inbox (Ctrl+Shift+N)\n\n## Tips\n\n- Use `[[wikilinks]]` to connect your notes\n- Press `Ctrl+N` to create a new note\n- Press `Ctrl+Shift+N` to quick-capture into your inbox\n- Open the Star View to see your knowledge network\n\nHappy exploring!\n",
            now
        );
        // MIG-076 §A2 — gated.
        crate::write_gate::gate_write(&welcome_path, &content, None, "welcome_note")
            .map_err(|e| format!("Failed to write Welcome.md: {}", e))?;
    }

    Ok(())
}

// ─── Template Commands ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEntry {
    pub name: String,
    pub path: String,
}

/// Resolve the templates folder the user actually chose.
///
/// MIG-TPL §1 (2026-07-19) — THE DISCONNECTION FIX. Settings has always shown a "Template folder"
/// field (`appSettings.templateFolder`, default `"Templates"`), and NOTHING read it: both commands
/// below resolved `<universe>/.constellation/templates` unconditionally — a hidden directory the
/// UI never reveals and never creates. A Universe without that directory therefore had an
/// empty picker forever, with an empty-state message pointing at the placebo setting.
///
/// Now the frontend passes its setting and it is honoured. File-Over-App: a template is an ordinary
/// note the user owns, so it lives in a VISIBLE folder, not inside the app's private directory.
///
/// `folder` is interpreted as relative to the universe root (the common case, e.g. `"Templates"`),
/// or used as-is when absolute (the Settings folder-picker yields an absolute path). Empty falls
/// back to `"Templates"`. Traversal is refused: a relative folder may not escape the universe root.
fn resolve_templates_dir(app: &tauri::AppHandle, folder: Option<String>) -> Result<PathBuf, String> {
    let root = active_universe_dir(app)?;
    resolve_templates_dir_for_root(&root, folder.as_deref())
}

/// AppHandle-free core of `resolve_templates_dir`, factored out (PJ-153 /
/// MIG-105 C6) so the boot cid_cn healer in search.rs — which runs inside
/// `init_db`, before any AppHandle or frontend setting exists — can resolve
/// the SAME folder from the universe root + the persisted `templateFolder`
/// setting. Semantics must stay identical to the command path above: empty /
/// None falls back to "Templates"; absolute is used as-is; a relative folder
/// may not escape the universe root.
pub(crate) fn resolve_templates_dir_for_root(root: &Path, folder: Option<&str>) -> Result<PathBuf, String> {
    let raw = folder.unwrap_or_default().trim();
    let candidate = if raw.is_empty() {
        root.join("Templates")
    } else {
        let p = Path::new(raw);
        if p.is_absolute() { p.to_path_buf() } else { root.join(p) }
    };

    // Refuse `../` escapes for RELATIVE settings. An absolute path is the user's explicit choice
    // (they picked it in the folder browser) and is left alone.
    if !Path::new(raw).is_absolute() {
        let normalized = candidate
            .components()
            .filter(|c| !matches!(c, std::path::Component::CurDir))
            .collect::<PathBuf>();
        if normalized.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err("Template folder must not contain '..'".to_string());
        }
    }
    Ok(candidate)
}

/// Get the path to the templates directory, creating it on first use.
#[tauri::command]
pub fn get_templates_dir(app: tauri::AppHandle, folder: Option<String>) -> Result<String, String> {
    let templates_dir = resolve_templates_dir(&app, folder)?;
    fs::create_dir_all(&templates_dir)
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;
    Ok(templates_dir.to_string_lossy().to_string())
}

/// List all .md template files in the user's templates directory.
///
/// Returns an empty list (not an error) when the folder does not exist yet — the picker turns that
/// into an actionable empty state naming the REAL folder and offering to create a template.
#[tauri::command]
pub fn list_templates(app: tauri::AppHandle, folder: Option<String>) -> Result<Vec<TemplateEntry>, String> {
    let templates_dir = resolve_templates_dir(&app, folder)?;
    if !templates_dir.exists() {
        return Ok(vec![]);
    }
    let mut templates = Vec::new();
    collect_templates_recursive(&templates_dir, &mut templates);
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
}

/// One-time, VISIBLE, LOSSLESS migration of any templates left in the old hidden directory.
///
/// COPY, never move-and-delete: if anything goes wrong the originals are still there. Existing
/// files at the destination are never overwritten. Returns the number of files copied so the
/// frontend can tell the user what happened rather than moving their notes silently.
#[tauri::command]
pub fn migrate_legacy_templates(app: tauri::AppHandle, folder: Option<String>) -> Result<usize, String> {
    let legacy = active_constellation_dir(&app)?.join("templates");
    if !legacy.exists() {
        return Ok(0);
    }
    let dest = resolve_templates_dir(&app, folder)?;
    if legacy == dest {
        return Ok(0);
    }
    fs::create_dir_all(&dest)
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;

    let mut copied = 0usize;
    let mut found = Vec::new();
    collect_templates_recursive(&legacy, &mut found);
    for entry in found {
        let src = Path::new(&entry.path);
        let file_name = match src.file_name() {
            Some(n) => n,
            None => continue,
        };
        let target = dest.join(file_name);
        if target.exists() {
            continue; // never clobber a template the user already has
        }
        if fs::copy(src, &target).is_ok() {
            copied += 1;
        }
    }
    Ok(copied)
}

// ─── MIG-103 §1 — "Save as Template": the impression-taking gesture ───

/// The THREE kinds of template a note can be saved as (Boss taxonomy, 2026-07-21).
/// Declared on the template file as `template_kind:` so the "use" side knows which
/// action to take (create / apply / insert).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateKind {
    /// Frontmatter + body → creating a note from it makes a whole note.
    Whole,
    /// Properties only, no body → applied to merge properties into the current note.
    Frontmatter,
    /// A body fragment, no properties → inserted at the cursor.
    Snippet,
}

impl TemplateKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "whole" | "note" => Some(Self::Whole),
            "frontmatter" | "properties" => Some(Self::Frontmatter),
            "snippet" => Some(Self::Snippet),
            _ => None,
        }
    }
    #[allow(dead_code)] // used by the §1 use-side (next increment) to read template_kind
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Whole => "whole",
            Self::Frontmatter => "frontmatter",
            Self::Snippet => "snippet",
        }
    }
}

/// The pure transform: note content → template content, for one of the three kinds.
///
/// **The R1 ruling (Boss-approved 2026-07-21), grounded in the standards research
/// (`docs/concept-papers/MIG-103-R1-Standards-and-Case-Studies.md`):** a template
/// is a FULL document whose type marker changes — the `.dotx`/`.ott` model. In
/// every kind, identity is stripped (the Boss caveat, and the anti-Evernote rule):
///
/// - **`cid_cn`/`created` removed**, **`title` = the template's name**, **`kind:
///   template`** — the type flip `file_kinds.rs` maps to `TMPL`.
/// - **`template_kind:`** declares which of the three this is, so "use" knows the
///   action without re-inferring it from shape.
///
/// The kinds differ in what they carry:
/// - **Whole** — properties (minus identity) + body, verbatim. Body bytes pass
///   through unchanged (the MIG-101 §A0 byte-splice discipline; CRLF preserved).
/// - **Frontmatter** — properties (minus identity), **body dropped**. The mold's
///   property skeleton, to be *applied* onto a note.
/// - **Snippet** — **body only**, no source properties. A reusable body fragment;
///   it still carries the minimal `kind: template` / `template_kind: snippet` /
///   `title` block so the file is identifiable as a template, but nothing of the
///   source note's own frontmatter.
pub(crate) fn template_content_from_note(
    content: &str,
    template_name: &str,
    kind: TemplateKind,
) -> String {
    let body = crate::bases::parse_frontmatter(content)
        .map(|_| {
            // Body = everything after the frontmatter block, byte-exact.
            crate::bases::frontmatter_span(content)
                .map(|(_, _, body_start)| content[body_start..].to_string())
                .unwrap_or_else(|| content.to_string())
        })
        .unwrap_or_else(|| content.to_string());

    match kind {
        TemplateKind::Whole => {
            let mut out = crate::bases::remove_frontmatter_property(content, "cid_cn");
            out = crate::bases::remove_frontmatter_property(&out, "cid");
            out = crate::bases::remove_frontmatter_property(&out, "created");
            out = crate::bases::update_frontmatter_property(&out, "kind", "template");
            out = crate::bases::update_frontmatter_property(&out, "template_kind", "whole");
            crate::bases::update_frontmatter_property(&out, "title", template_name)
        }
        TemplateKind::Frontmatter => {
            // Properties minus identity, and NO body.
            let Some((open_end, close_start, _)) = crate::bases::frontmatter_span(content) else {
                // No frontmatter to template — produce a bare properties template.
                return format!(
                    "---\nkind: template\ntemplate_kind: frontmatter\ntitle: {}\n---\n",
                    template_name
                );
            };
            let mut fm_only = content[..close_start].to_string();
            // The closing fence + a single trailing newline; drop the body entirely.
            let eol = if content[..open_end].ends_with("\r\n") { "\r\n" } else { "\n" };
            fm_only.push_str(&format!("---{eol}"));
            let mut out = crate::bases::remove_frontmatter_property(&fm_only, "cid_cn");
            out = crate::bases::remove_frontmatter_property(&out, "cid");
            out = crate::bases::remove_frontmatter_property(&out, "created");
            out = crate::bases::update_frontmatter_property(&out, "kind", "template");
            out = crate::bases::update_frontmatter_property(&out, "template_kind", "frontmatter");
            crate::bases::update_frontmatter_property(&out, "title", template_name)
        }
        TemplateKind::Snippet => snippet_template(template_name, &body),
    }
}

/// Build a snippet template from an arbitrary body fragment.
///
/// MIG-103 §1 (Boss request 2026-07-21): a snippet may be the note's WHOLE body
/// or just a **selected** word / sentence / paragraph — a snippet is a fragment,
/// so the user chooses its extent. Both paths land here so the produced file is
/// identical in shape either way: a minimal identifying block (no properties from
/// the source note — that is what makes it a snippet rather than a whole note),
/// then the fragment verbatim.
pub(crate) fn snippet_template(template_name: &str, body: &str) -> String {
    let eol = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let header = format!(
        "---{eol}kind: template{eol}template_kind: snippet{eol}title: {}{eol}---{eol}",
        template_name
    );
    format!("{header}{}", body.trim_start_matches(['\r', '\n']))
}

/// Windows-safe file stem for a template name (mirrors create_note's discipline).
fn sanitize_template_stem(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') { ' ' } else { c })
        .collect();
    let cleaned = cleaned.trim().trim_end_matches('.').to_string();
    if cleaned.is_empty() { "Template".to_string() } else { cleaned }
}

/// MIG-103 §1a — save an existing note as a template.
///
/// `content` carries the LIVE editor text when the note is open (the model is the
/// authority on an open note — MIG-076; reading disk here would snapshot a stale
/// cast). When absent, disk is read. The source must live inside the active
/// universe or a registered library; the destination is the visible templates
/// folder; collisions auto-suffix (`Name 1`, `Name 2`) exactly like create_note;
/// the write is create-exclusive so a race cannot clobber an existing template.
#[tauri::command(async)]
pub fn create_template(
    app: tauri::AppHandle,
    file_path: String,
    content: Option<String>,
    template_name: String,
    kind: String,
    // `snippet_text` (MIG-103 §1): when saving a SNIPPET the user may choose a
    // selected fragment instead of the whole body. Present = use this text
    // verbatim as the snippet; absent = fall back to the note's whole body.
    // Ignored for the other two kinds, whose extent is not a choice.
    snippet_text: Option<String>,
    folder: Option<String>,
) -> Result<String, String> {
    // MIG-111 §0.4 — the READ-scope check, because `file_path` is the SOURCE note, not the
    // destination. The template is written into `resolve_templates_dir`, inside the ACTIVE
    // universe, and is safe by construction.
    //
    // This was `validate_base_path`, which §0.4 taught to refuse linked universes — turning a
    // legitimate own-universe write into a refusal whenever the SOURCE note happened to live in a
    // linked one. Since MIG-108 nests those under the active root, "Save as template" on such a
    // note broke: the dialog closed, no file appeared, and the frontend swallowed the error into
    // a console that does not exist in release. Making a template FROM something you can read is
    // exactly what federation is for; the boundary belongs on where it lands, and that is already
    // guarded at the two writers that actually touch a template file.
    crate::libraries::validate_path_in_any_library(&app, &file_path)?;
    let template_kind = TemplateKind::parse(&kind)
        .ok_or_else(|| format!("Unknown template kind '{}'.", kind))?;
    let source = match content {
        Some(c) => c,
        None => fs::read_to_string(&file_path).map_err(|e| format!("Failed to read note: {}", e))?,
    };

    let stem = sanitize_template_stem(&template_name);
    let templated = match (template_kind, snippet_text.as_deref()) {
        (TemplateKind::Snippet, Some(sel)) if !sel.trim().is_empty() => snippet_template(&stem, sel),
        _ => template_content_from_note(&source, &stem, template_kind),
    };

    let dir = resolve_templates_dir(&app, folder)?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create templates directory: {}", e))?;

    // Collision-resolve, then write create-exclusive: the exists() probe is a
    // convenience; the gate is the guarantee (a concurrent create REFUSES, and
    // we advance to the next suffix rather than clobbering).
    for attempt in 0..100u32 {
        let candidate = if attempt == 0 {
            dir.join(format!("{stem}.md"))
        } else {
            dir.join(format!("{stem} {attempt}.md"))
        };
        if candidate.exists() {
            continue;
        }
        match crate::write_gate::gate_create_exclusive(&candidate, &templated, "create_template")? {
            crate::write_gate::WriteOutcome::RefusedExists => continue,
            _ => {
                let p = candidate.to_string_lossy().to_string();
                reindex_written_template(&app, &p, "create_template");
                return Ok(p);
            }
        }
    }
    Err("Could not find a free template name after 100 attempts.".to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// MIG-103 §4 Slice 2 — KEEP: turn a discovered kind into a real mold.
//
// The Studio proposes; the user decides; only then is anything written. So this
// is the ONLY place a discovered kind touches disk, and it does so under three
// Boss rulings (2026-07-22):
//   1. NEVER overwrite an existing template. On a name clash the user chooses —
//      rename, merge into the existing one, or cancel — so this returns a TYPED
//      error rather than silently suffixing the name. (`create_template` DOES
//      auto-suffix, and that is right for its gesture: saving a note as a
//      template is not naming a kind. Different act, different rule.)
//   2. Optional fields are opt-in. The caller sends exactly what the user ticked;
//      no threshold is applied here, because any threshold is a judgement the
//      data does not contain.
//   3. Undo is real. `undo_adopt_kind` trashes the file, but ONLY while it is
//      byte-identical to what was written — once the user has edited the mold it
//      is their work, not our transaction.
// ─────────────────────────────────────────────────────────────────────────────

/// Index a template we just wrote, mirroring `create_note`.
///
/// 2026-07-22 inspection. Template writes go through the gate, which SUPPRESSES the
/// watcher event, and none of them reindexed — so a freshly written template had no
/// `note_meta` row until the next boot's reconcile. That is not "templates are
/// excluded from search" (the frontend's `isTemplatePath` does that deliberately);
/// it is invisible-now-visible-after-restart, which is worse than either choice,
/// and it made a `[[Template Name]]` reference fail to resolve and offer to create
/// a NEW note — a silent duplicate.
///
/// The file is the source of truth and was written successfully, so an index failure
/// is SURFACED but never fails the write.
fn reindex_written_template(app: &tauri::AppHandle, path: &str, surface: &str) {
    use tauri::Manager;
    let search_state = app.state::<crate::search::SearchState>();
    // Canonical longest-root-wins resolver (2026-07-24 inspection) — a first-match
    // `starts_with` always returned the universe_notes root, filing every template
    // written into a nested sub-library under the wrong library_name.
    // PJ-254 — OWN libraries only. This feeds a reindex, and the FEDERATED resolver would
    // file a template written into a linked universe under THIS universe's index.
    match crate::libraries::owning_own_library_name(app, path) {
        Some(lib_name) => {
            if let Err(e) = crate::search::reindex_single_note(&search_state, path, &lib_name) {
                if let Ok(p) = crate::search::db_path(app) {
                    crate::search::diag_log(&p, &format!("[{surface}] reindex FAILED for {path}: {e}"));
                }
            }
        }
        None => {
            // A templates folder outside every library is a valid choice; nothing to index.
            if let Ok(p) = crate::search::db_path(app) {
                crate::search::diag_log(&p, &format!("[{surface}] NO LIBRARY matched {path} — reindex SKIPPED"));
            }
        }
    }
}

/// Returned when the chosen name is taken. The frontend matches on this exact
/// prefix to raise the three-way choice; a plain string error would be
/// indistinguishable from an I/O failure and would surface as a scary message.
pub const TEMPLATE_EXISTS: &str = "TEMPLATE_EXISTS:";

/// Build a mold from a discovered kind: properties with empty values, then a
/// section per recurring heading.
///
/// Values are deliberately EMPTY. A discovered kind is a shape, not content — the
/// fields say *what a note of this kind answers*, and every answer belongs to the
/// cast. Spellings arrive as the members actually write them (`Country`, not
/// `country`), which is why §4's `display` amendment exists: a mold cut with the
/// wrong casing spawns a duplicate property in every note made from it.
pub(crate) fn template_content_from_kind(
    name: &str,
    fields: &[String],
    headings: &[String],
    core: &[String],
) -> String {
    let mut out = String::from("---\n");
    out.push_str("kind: template\n");
    out.push_str("template_kind: whole\n");
    out.push_str(&format!("title: {name}\n"));
    // The kind this mold was cut from, so the Studio can recognise it again.
    //
    // Boss, 2026-07-23: "If I kept a kind… why is it there, as it hasn't been dealt
    // with?" — because the tick lived in memory and died with the session. The record
    // belongs in the FILE, not in app state: there it survives a restart, a sync, and
    // moving the Universe to another machine. File-Over-App applies to the mold exactly
    // as it does to the cast.
    if !core.is_empty() {
        let mut sorted: Vec<String> = core.iter().map(|k| k.to_lowercase()).collect();
        sorted.sort();
        out.push_str(&format!("from_kind: {}\n", sorted.join(" ")));
    }
    for f in fields {
        let key = f.trim();
        if key.is_empty() || key.contains(':') {
            continue; // a key carrying a colon would break the block
        }
        out.push_str(&format!("{key}:\n"));
    }
    out.push_str("---\n\n");
    for h in headings {
        let t = h.trim();
        if !t.is_empty() {
            out.push_str(&format!("## {t}\n\n"));
        }
    }
    out
}

/// KEEP — write the mold. Create-exclusive; never overwrites.
// PJ-066 §C5 — `(async)`: this reindexes, which takes the writer `db` lock, and a
// SYNC command holding it blocks the IPC thread and freezes the UI.
#[tauri::command(async)]
pub fn adopt_discovered_kind(
    app: tauri::AppHandle,
    name: String,
    fields: Vec<String>,
    headings: Vec<String>,
    // The kind's core keys — stamped into the mold so the Studio recognises it later.
    core: Vec<String>,
    folder: Option<String>,
) -> Result<String, String> {
    let stem = sanitize_template_stem(&name);
    let content = template_content_from_kind(&stem, &fields, &headings, &core);

    let dir = resolve_templates_dir(&app, folder)?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create templates directory: {}", e))?;
    let target = dir.join(format!("{stem}.md"));

    // The gate is the guarantee — a concurrent create refuses rather than clobbers.
    // The exists() probe only lets us report the clash before attempting the write.
    if target.exists() {
        return Err(format!("{TEMPLATE_EXISTS}{}", target.to_string_lossy()));
    }
    match crate::write_gate::gate_create_exclusive(&target, &content, "adopt_discovered_kind")? {
        crate::write_gate::WriteOutcome::RefusedExists => {
            Err(format!("{TEMPLATE_EXISTS}{}", target.to_string_lossy()))
        }
        _ => {
            let p = target.to_string_lossy().to_string();
            reindex_written_template(&app, &p, "adopt_discovered_kind");
            Ok(p)
        }
    }
}

/// Which discovered kinds already have a mold, read from the templates themselves.
///
/// The Studio calls this on open so a kept kind shows the name the user gave it rather
/// than pretending it was never dealt with. The source of truth is the `from_kind:` line
/// in each template — no app state, nothing to go stale, and it survives a restart.
#[derive(serde::Serialize)]
pub struct KeptKind {
    /// The kind's core keys, lowercased and space-joined — its signature.
    pub signature: String,
    pub name: String,
    pub path: String,
}

#[tauri::command(async)]
pub fn list_kept_kinds(
    app: tauri::AppHandle,
    folder: Option<String>,
) -> Result<Vec<KeptKind>, String> {
    let dir = resolve_templates_dir(&app, folder)?;
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(out); // no templates folder yet is not an error
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(true, |e| e != "md") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else { continue };
        let Some(props) = crate::bases::parse_frontmatter(&content) else { continue };
        let Some(sig) = props.get("from_kind") else { continue };
        let sig = sig.trim().to_lowercase();
        if sig.is_empty() {
            continue;
        }
        out.push(KeptKind {
            signature: sig,
            name: props.get("title").cloned().unwrap_or_else(|| {
                path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
            }),
            path: path.to_string_lossy().to_string(),
        });
    }
    Ok(out)
}

/// MERGE — add missing properties to an existing mold, changing nothing else.
///
/// Additive by construction: a key already present keeps its value (which may be a
/// default the user typed), the body is untouched, and no key is ever removed. The
/// user chose "add these fields to it", not "replace it".
// PJ-066 §C5 — `(async)`: this reindexes, which takes the writer `db` lock, and a
// SYNC command holding it blocks the IPC thread and freezes the UI.
#[tauri::command(async)]
pub fn merge_fields_into_template(
    app: tauri::AppHandle,
    template_path: String,
    fields: Vec<String>,
) -> Result<Vec<String>, String> {
    crate::bases::validate_base_path(&app, &template_path)?;

    // 2026-07-24 inspection. This was an UNGUARDED read-modify-write: read the file,
    // append fields, then `gate_write(..., expect = None)`. Studio-cut templates
    // deliberately carry no `cid_cn`, so the gate's self-attestation degraded to an
    // unconditional overwrite — an editor save landing between the read and the write
    // was silently discarded. `gate_rmw` makes read+write ONE critical section, so a
    // concurrent save can land before or after but never inside the window. (The
    // closure is pure string work: no `gate_*`, no DB lock inside — gate_rmw's two
    // hard rules.)
    let mut added: Vec<String> = Vec::new();
    let outcome = crate::write_gate::gate_rmw(
        Path::new(&template_path),
        "merge_fields_into_template",
        |original| {
            added.clear();
            let existing: std::collections::HashSet<String> =
                crate::bases::parse_frontmatter(original)
                    .map(|m| m.keys().map(|k| k.to_lowercase()).collect())
                    .unwrap_or_default();
            let mut out = original.to_string();
            for f in &fields {
                let key = f.trim();
                if key.is_empty() || key.contains(':') || existing.contains(&key.to_lowercase()) {
                    continue;
                }
                out = crate::bases::update_frontmatter_property(&out, key, "");
                added.push(key.to_string());
            }
            // Nothing to add — do not touch the file at all.
            Ok(if added.is_empty() { None } else { Some(out) })
        },
    )?;

    if added.is_empty() || outcome == crate::write_gate::WriteOutcome::OkUnchecked {
        return Ok(added);
    }
    reindex_written_template(&app, &template_path, "merge_fields_into_template");
    // The gate marks the path watcher-SUPPRESSED, so without this an OPEN template tab
    // keeps its pre-merge content and the next keystroke's save silently overwrites the
    // merged fields while the Studio's "merged: added X" message stands. Re-uses the
    // watcher's own event → the existing adopt path (clean model adopts, dirty model
    // keeps its work and sidecars the change).
    announce_disk_write(&app, &template_path);
    Ok(added)
}

/// Announce a Rust-side write so an OPEN note re-bases from disk instead of
/// overwriting it. Gated writes are watcher-suppressed by design (we must not treat
/// our own write as an external edit); this re-emits the watcher's own event so the
/// well-tested `adoptExternalChangeIntoTabs` path runs. Twin of
/// `sources::announce_frontmatter_write`.
fn announce_disk_write(app: &tauri::AppHandle, path: &str) {
    use tauri::Emitter;
    let _ = app.emit(
        "library-changed",
        serde_json::json!({ "libraryId": "", "paths": [path] }),
    );
}

/// UNDO — trash a mold that was just kept, ONLY if it is untouched since.
///
/// The guard is byte-equality with what we wrote. Once the user has edited the
/// mold it is their work and undo declines; an undo that silently discarded an
/// edit would be exactly the silent data loss this project hunts.
// PJ-066 §C5 — `(async)`: this reindexes, which takes the writer `db` lock, and a
// SYNC command holding it blocks the IPC thread and freezes the UI.
#[tauri::command(async)]
pub fn undo_adopt_kind(
    app: tauri::AppHandle,
    template_path: String,
    expected_content: String,
) -> Result<bool, String> {
    crate::bases::validate_base_path(&app, &template_path)?;
    let current = match fs::read_to_string(&template_path) {
        Ok(c) => c,
        Err(_) => return Ok(false), // already gone — nothing to undo, and not an error
    };
    if current != expected_content {
        return Ok(false); // edited since; leave it alone
    }
    // Trash, never delete. `move_to_trash` validates membership of a registered
    // library, and the universe root IS one (`universe_notes`), so the default
    // `<universe>/Templates` resolves. A templates folder pointed OUTSIDE the
    // universe returns an error here rather than being hard-deleted — the right
    // failure: refusing to undo is recoverable, an unrecoverable delete is not.
    let root = active_universe_dir(&app)?.to_string_lossy().to_string();
    crate::libraries::move_to_trash(app, template_path, root)?;
    Ok(true)
}

fn collect_templates_recursive(dir: &Path, templates: &mut Vec<TemplateEntry>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_templates_recursive(&path, templates);
        } else if path.extension().map_or(false, |ext| ext == "md") {
            // MIG-008 Step 6: template picker labels use frontmatter title
            // so a canonical-named template ("20260426T...NOTE_XXXX.md")
            // shows its human title in the picker.
            let name = crate::libraries::note_display_name(&path, None);
            templates.push(TemplateEntry {
                name,
                path: path.to_string_lossy().to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// PJ-187 — concurrent writers of the SAME state file must never share a temp path.
    ///
    /// Every persisted-state file (registry, universe.json, settings, workspaces, session,
    /// collections, property-types) is written through this one function. With the old fixed
    /// `<target>.tmp` name, two writers created and fsync'd the same temp, so one could publish
    /// the other's half-written bytes under the final name — and every loader here swallows a
    /// parse error and falls back to EMPTY, so the corruption presents as "you have no
    /// collections / no workspaces / no universes" and the next save writes that back.
    #[test]
    fn pj187_concurrent_state_writes_never_share_a_temp_name() {
        use std::sync::{Arc, Barrier};
        let dir = TempDir::new().unwrap();
        let target = Arc::new(dir.path().join("settings.json"));

        // Two threads writing DIFFERENT contents to the same path, released together.
        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();
        for (i, byte) in [b'A', b'B'].into_iter().enumerate() {
            let t = Arc::clone(&target);
            let b = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                let payload = vec![byte; 64 * 1024]; // large enough that the writes overlap
                b.wait();
                for _ in 0..25 {
                    atomic_write(&t, &payload).unwrap_or_else(|e| panic!("writer {i} failed: {e}"));
                }
            }));
        }
        for h in handles {
            h.join().expect("a writer panicked");
        }

        // The survivor must be ENTIRELY one writer's bytes — never a mixture, and never empty.
        let got = std::fs::read(&*target).expect("target must exist");
        assert!(!got.is_empty(), "target was left empty");
        let first = got[0];
        assert!(
            got.iter().all(|b| *b == first),
            "the published file interleaved two writers' bytes — a shared temp path",
        );
        assert_eq!(got.len(), 64 * 1024, "the published file was truncated");

        // And no temp files may be left behind.
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".cnstmp") || n.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp files left behind: {leftovers:?}");
    }

    // ── MIG-103 §4 Slice 2 — the mold a kept kind produces ──────────────────

    #[test]
    fn a_kept_kind_becomes_a_mold_with_empty_fields_and_its_sections() {
        let out = template_content_from_kind("film", &["country".into(), "language".into()], &["Cast".into(), "Plot".into()], &["country".into(), "language".into()]);

        assert!(out.starts_with("---
"));
        assert!(out.contains("kind: template
"));
        assert!(out.contains("template_kind: whole
"));
        assert!(out.contains("title: film
"));
        // EMPTY values: a kind is a shape, not content — every answer belongs to the cast.
        assert!(out.contains("country:
"), "{out}");
        assert!(out.contains("language:
"), "{out}");
        assert!(out.contains("## Cast
"));
        assert!(out.contains("## Plot
"));
        // No identity: a mold that carried one would stamp its birthday on every cast.
        assert!(!out.contains("cid_cn"));
        assert!(!out.contains("created:"));
    }

    /// The mold is cut with the spelling the member notes actually use. A
    /// case-mismatched key spawns a DUPLICATE property in every note made from it.
    #[test]
    fn the_mold_keeps_the_spelling_it_was_given() {
        let out = template_content_from_kind("Film", &["Country".into()], &["Production".into()], &["Country".into()]);
        assert!(out.contains("Country:
"), "{out}");
        assert!(!out.contains("country:
"), "must not lowercase the user's own key");
        assert!(out.contains("## Production
"));
    }

    #[test]
    fn a_kind_with_no_sections_still_makes_a_valid_mold() {
        let out = template_content_from_kind("person", &["born".into(), "died".into()], &[], &["born".into(), "died".into()]);
        assert_eq!(out.matches("---").count(), 2, "exactly one frontmatter block:
{out}");
        assert!(out.trim_end().ends_with("---"), "no stray sections:
{out}");
    }

    /// A key containing a colon would break the YAML block. Skipped, not escaped —
    /// a property name with a colon in it is not a property name.
    #[test]
    fn a_field_that_would_break_the_block_is_skipped() {
        let out = template_content_from_kind("x", &["good".into(), "bad: key".into(), "  ".into()], &[], &[]);
        assert!(out.contains("good:
"));
        assert!(!out.contains("bad"));
        let fm = out.split("---").nth(1).unwrap();
        for line in fm.lines().filter(|l| !l.trim().is_empty()) {
            assert_eq!(line.matches(':').count(), 1, "one colon per line: {line:?}");
        }
    }

    /// Write a `{root}/universe.json` (the fallback location read by
    /// `resolve_child_universe_roots`) with the given children.
    fn write_universe(root: &Path, name: &str, children: &[&Path]) {
        fs::create_dir_all(root).unwrap();
        let meta = UniverseMeta {
            name: name.to_string(),
            created: "2026-01-01".to_string(),
            version: 1,
            children: children
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            notes_folder: None,
        };
        fs::write(
            root.join("universe.json"),
            serde_json::to_string(&meta).unwrap(),
        )
        .unwrap();
    }

    fn canon_set(roots: &[PathBuf]) -> std::collections::HashSet<PathBuf> {
        roots
            .iter()
            .map(|p| fs::canonicalize(p).unwrap_or_else(|_| p.clone()))
            .collect()
    }

    // ─── MIG-100 §1 — auto-session IPC pair ───

    fn session_of(root: &Path) -> serde_json::Value {
        read_universe_session(root.to_string_lossy().to_string()).unwrap()
    }

    fn save_session(root: &Path, v: serde_json::Value) {
        save_universe_session(root.to_string_lossy().to_string(), v).unwrap();
    }

    #[test]
    fn session_missing_file_is_null_never_err() {
        let tmp = TempDir::new().unwrap();
        assert!(session_of(tmp.path()).is_null());
    }

    #[test]
    fn session_round_trip() {
        let tmp = TempDir::new().unwrap();
        let snap = serde_json::json!({"version": 1, "tabs": [{"path": "a.md"}]});
        save_session(tmp.path(), snap.clone());
        assert_eq!(session_of(tmp.path()), snap);
    }

    #[test]
    fn session_corrupt_current_falls_back_to_prev() {
        let tmp = TempDir::new().unwrap();
        let cdir = constellation_dir(tmp.path());
        fs::create_dir_all(&cdir).unwrap();
        let prev = serde_json::json!({"version": 1, "tabs": [{"path": "prev.md"}]});
        fs::write(cdir.join("session.prev.json"), serde_json::to_string(&prev).unwrap()).unwrap();
        fs::write(cdir.join("session.json"), "{ not json").unwrap();
        assert_eq!(session_of(tmp.path()), prev);
        // Both generations corrupt → null, never an Err.
        fs::write(cdir.join("session.prev.json"), "also { not json").unwrap();
        assert!(session_of(tmp.path()).is_null());
    }

    #[test]
    fn session_rotates_once_per_launch() {
        let tmp = TempDir::new().unwrap();
        let cdir = constellation_dir(tmp.path());
        fs::create_dir_all(&cdir).unwrap();
        // Simulate LAST launch's final state already on disk.
        let last_launch = serde_json::json!({"version": 1, "tabs": [{"path": "last-launch.md"}]});
        fs::write(cdir.join("session.json"), serde_json::to_string(&last_launch).unwrap()).unwrap();
        // First save of THIS launch rotates it into .prev …
        let v2 = serde_json::json!({"version": 1, "tabs": [{"path": "v2.md"}]});
        save_session(tmp.path(), v2.clone());
        // … later saves do NOT rotate again: .prev stays last-launch, not ~1s stale.
        let v3 = serde_json::json!({"version": 1, "tabs": [{"path": "v3.md"}]});
        save_session(tmp.path(), v3.clone());
        assert_eq!(session_of(tmp.path()), v3);
        let prev: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(cdir.join("session.prev.json")).unwrap()).unwrap();
        assert_eq!(prev, last_launch);
    }

    #[test]
    fn session_two_roots_stay_isolated() {
        let tmp_a = TempDir::new().unwrap();
        let tmp_b = TempDir::new().unwrap();
        let a = serde_json::json!({"version": 1, "tabs": [{"path": "a.md"}]});
        let b = serde_json::json!({"version": 1, "tabs": [{"path": "b.md"}]});
        save_session(tmp_a.path(), a.clone());
        save_session(tmp_b.path(), b.clone());
        assert_eq!(session_of(tmp_a.path()), a);
        assert_eq!(session_of(tmp_b.path()), b);
    }

    #[test]
    fn own_libraries_for_root_reads_non_recursive() {
        // MIG-100 switch-flush fix: a departing universe's own libraries must
        // resolve from its root regardless of which universe is active.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("UniverseA");
        let cdir = constellation_dir(&root);
        fs::create_dir_all(&cdir).unwrap();
        let libs = serde_json::json!([
            { "id": "lib1", "name": "A Notes", "path": root.to_string_lossy(),
              "color": "#111", "is_universe_notes": true },
            { "id": "lib2", "name": "Sub", "path": root.join("Sub").to_string_lossy(),
              "color": "#222", "is_universe_notes": false },
        ]);
        fs::write(cdir.join("libraries.json"), serde_json::to_string(&libs).unwrap()).unwrap();

        let resolved = own_libraries_for_root(&root);
        assert_eq!(resolved.len(), 2, "both own libraries resolve");
        assert!(resolved.iter().any(|l| l.name == "A Notes"));
        assert!(resolved.iter().any(|l| l.name == "Sub"));

        // A missing/blank universe root → empty, never a panic.
        assert!(own_libraries_for_root(&tmp.path().join("Nope")).is_empty());
    }

    #[test]
    fn session_null_deletes_both_generations() {
        let tmp = TempDir::new().unwrap();
        let cdir = constellation_dir(tmp.path());
        fs::create_dir_all(&cdir).unwrap();
        fs::write(cdir.join("session.json"), "{\"version\":1}").unwrap();
        fs::write(cdir.join("session.prev.json"), "{\"version\":1}").unwrap();
        save_session(tmp.path(), serde_json::Value::Null);
        assert!(!cdir.join("session.json").exists());
        assert!(!cdir.join("session.prev.json").exists());
        assert!(session_of(tmp.path()).is_null());
    }

    // MIG-062 §B — recursive enumeration covers the whole federation tree.
    #[test]
    fn recursive_roots_covers_full_federation_tree() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("A");
        let b = tmp.path().join("B");
        let c = tmp.path().join("C");
        let d = tmp.path().join("D");
        for p in [&a, &b, &c, &d] {
            fs::create_dir_all(p).unwrap();
        }
        // A -> B, C ; B -> D ; C and D are leaves.
        write_universe(&a, "A", &[&b, &c]);
        write_universe(&b, "B", &[&d]);
        write_universe(&c, "C", &[]);
        write_universe(&d, "D", &[]);

        let roots = resolve_child_universe_roots_recursive(&a);
        let set = canon_set(&roots);
        assert_eq!(set.len(), 3, "expected B, C, D — got {:?}", roots);
        assert!(set.contains(&fs::canonicalize(&b).unwrap()));
        assert!(set.contains(&fs::canonicalize(&c).unwrap()));
        assert!(set.contains(&fs::canonicalize(&d).unwrap()));
        // Parent A is NOT included.
        assert!(!set.contains(&fs::canonicalize(&a).unwrap()));
    }

    // ─── MIG-111 §1.2/A4 — the fail-closed federation enumeration ───────
    //
    // Real directories, real manifests. The misroute these prevent is not a lost
    // list item: under MIG-108 a linked universe lives UNDER the active root, so a
    // child dropped from the candidate set does not make `resolve_owner` refuse —
    // it makes it answer PARENT, confidently, and the write lands in the wrong
    // database. Each test therefore also demonstrates the misroute it stops.

    #[test]
    fn a_manifest_that_cannot_be_parsed_refuses_instead_of_reporting_no_children() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("A");
        let child = a.join("Linked"); // MIG-108: the child lives UNDER the parent
        fs::create_dir_all(&child).unwrap();
        write_universe(&a, "A", &[&child]);
        write_universe(&child, "Linked", &[]);

        // Sanity: intact, the child IS enumerated.
        assert_eq!(resolve_child_universe_roots_recursive_strict(&a).unwrap().len(), 1);

        // Now corrupt the parent's manifest, the way a half-written file or a sync
        // tool leaves it.
        fs::write(a.join("universe.json"), b"{\"name\": \"A\", \"childr").unwrap();

        // The lenient reader says "no children" — and THAT is the misroute:
        let lenient = resolve_child_universe_roots_recursive(&a);
        assert!(lenient.is_empty(), "precondition: the lenient reader loses the child");
        let misrouted = crate::federation::owner::resolve_owner_in(
            &child.join("note.md").to_string_lossy(),
            &a,
            &lenient,
        )
        .expect("the lenient path does not even refuse");
        assert!(
            misrouted.is_active,
            "THE BUG: with the child lost, the parent claims a note it does not own"
        );

        // The strict reader refuses, and names the universe it could not read.
        let err = resolve_child_universe_roots_recursive_strict(&a)
            .expect_err("an unparseable manifest is not an empty federation");
        assert!(
            matches!(err, PersistedError::Corrupt(_)),
            "an unparseable manifest is CORRUPT, not merely unreadable — retrying it is              pointless and the caller must be able to tell: {err:?}"
        );
        assert!(err.message().contains("A"), "the refusal must name the universe: {err:?}");
        assert!(err.message().contains("Nothing was changed"), "{err:?}");
    }

    #[test]
    fn a_manifest_that_cannot_be_read_refuses() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("A");
        fs::create_dir_all(&a).unwrap();
        // A directory where the manifest belongs: an I/O error that is not NotFound,
        // on Windows and Unix alike.
        fs::create_dir_all(a.join("universe.json")).unwrap();
        let err = resolve_child_universe_roots_strict(&a)
            .expect_err("unreadable is not 'no children'");
        assert!(
            matches!(err, PersistedError::Unreadable(_)),
            "a manifest we could not READ is transient — the caller may retry, so the variant              must survive: {err:?}"
        );
        assert!(err.message().contains("Nothing was changed"), "{err:?}");
    }

    #[test]
    fn no_manifest_at_all_is_zero_children_not_an_error() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("A");
        fs::create_dir_all(&a).unwrap();
        assert!(
            resolve_child_universe_roots_strict(&a).unwrap().is_empty(),
            "a universe with zero cUniverses is a complete, valid setup"
        );
    }

    #[test]
    fn a_declared_child_that_cannot_be_canonicalized_is_kept_not_dropped() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("A");
        let gone = a.join("Unreachable");
        fs::create_dir_all(&a).unwrap();
        write_universe(&a, "A", &[&gone]); // declared, but never created

        let roots = resolve_child_universe_roots_strict(&a).unwrap();
        assert_eq!(roots.len(), 1, "the declared child is kept: {roots:?}");
        assert_eq!(roots[0], gone, "kept as its DECLARED path, since it cannot be canonicalized");

        // Why it matters: keeping it means a path inside it still resolves to IT.
        let owner = crate::federation::owner::resolve_owner_in(
            &gone.join("note.md").to_string_lossy(),
            &a,
            &roots,
        )
        .unwrap();
        assert!(
            !owner.is_active,
            "the note belongs to the declared child, not to the parent that contains it"
        );
    }

    // MIG-062 §B — a federation cycle (A->B->A) must terminate, not loop.
    #[test]
    fn recursive_roots_handles_cycle() {
        let tmp = TempDir::new().unwrap();
        let a = tmp.path().join("A");
        let b = tmp.path().join("B");
        for p in [&a, &b] {
            fs::create_dir_all(p).unwrap();
        }
        write_universe(&a, "A", &[&b]);
        write_universe(&b, "B", &[&a]); // cycle back to A
        let roots = resolve_child_universe_roots_recursive(&a);
        let set = canon_set(&roots);
        // Terminates; B is included, A (parent) is guarded out.
        assert!(set.contains(&fs::canonicalize(&b).unwrap()));
        assert!(!set.contains(&fs::canonicalize(&a).unwrap()));
    }
}

#[cfg(test)]
mod tests_mig103_template {
    use super::{sanitize_template_stem, template_content_from_note, TemplateKind};

    const NOTE: &str = "---\ntitle: \"Zakat rulings\"\ncid_cn: 20260707T190416Z_NOTE_943A\nkind: note\ncreated: 2026-07-07T19:04:16+00:00\ntags:\n  - fiqh\nstage: seed\n---\n# Overview\nProse paragraph one.\n\n## Details\nMore prose here.\n";

    // ── Kind 1: WHOLE — properties (minus identity) + body verbatim ──
    #[test]
    fn whole_keeps_body_and_properties_strips_identity() {
        let t = template_content_from_note(NOTE, "Zakat Template", TemplateKind::Whole);
        assert!(!t.contains("cid_cn:"), "cid_cn must not survive into a template");
        assert!(!t.contains("created:"), "created must not survive into a template");
        assert!(t.contains("kind: template"), "the type flip IS templateness");
        assert!(t.contains("template_kind: whole"), "the kind is declared on the file");
        assert!(t.contains("title: Zakat Template"));
        assert!(t.contains("Prose paragraph one."));
        assert!(t.contains("More prose here."));
        assert!(t.contains("  - fiqh"), "nested frontmatter preserved");
        assert!(t.contains("stage: seed"));
    }

    /// The whole-note template must classify as TMPL through the EXISTING kind
    /// system. Written to a real temp file because classification is path-based.
    #[test]
    fn produced_template_classifies_as_tmpl() {
        let t = template_content_from_note(NOTE, "T", TemplateKind::Whole);
        let dir = std::env::temp_dir().join(format!("cnstl_mig103_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("probe.md");
        std::fs::write(&f, &t).unwrap();
        let mut registry = crate::file_kinds::KindRegistry::new(None);
        let kind = crate::file_kinds::classify_file(&f, &mut registry);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(kind, "TMPL", "kind: template must classify as TMPL");
    }

    // ── Kind 2: FRONTMATTER — properties (minus identity), NO body ──
    #[test]
    fn frontmatter_keeps_properties_drops_body() {
        let t = template_content_from_note(NOTE, "Book Props", TemplateKind::Frontmatter);
        assert!(t.contains("kind: template"));
        assert!(t.contains("template_kind: frontmatter"));
        assert!(t.contains("title: Book Props"));
        assert!(t.contains("stage: seed"), "the properties' VALUES are kept (it's a property mold)");
        assert!(t.contains("  - fiqh"));
        assert!(!t.contains("cid_cn:"));
        assert!(!t.contains("created:"));
        assert!(!t.contains("Prose paragraph one."), "a frontmatter template carries NO body");
        assert!(!t.contains("# Overview"), "not even headings");
    }

    #[test]
    fn frontmatter_from_bare_note_is_a_minimal_props_block() {
        let t = template_content_from_note("Just a body\n", "Empty Props", TemplateKind::Frontmatter);
        assert!(t.contains("template_kind: frontmatter"));
        assert!(t.contains("title: Empty Props"));
        assert!(!t.contains("Just a body"), "no body in a frontmatter template");
    }

    // ── Kind 3: SNIPPET — body only, none of the source's properties ──
    #[test]
    fn snippet_keeps_body_drops_source_properties() {
        let t = template_content_from_note(NOTE, "My Snippet", TemplateKind::Snippet);
        assert!(t.contains("kind: template"));
        assert!(t.contains("template_kind: snippet"));
        assert!(t.contains("title: My Snippet"));
        assert!(t.contains("# Overview"), "the body is the snippet");
        assert!(t.contains("Prose paragraph one."));
        assert!(!t.contains("stage: seed"), "the source's own properties are NOT in a snippet");
        assert!(!t.contains("  - fiqh"));
        assert!(!t.contains("cid_cn:"));
    }

    #[test]
    fn snippet_from_bare_note_wraps_the_body() {
        let t = template_content_from_note("Reusable text.\n", "Frag", TemplateKind::Snippet);
        assert!(t.contains("template_kind: snippet"));
        assert!(t.contains("Reusable text."));
    }

    /// Boss request 2026-07-21 — a snippet may be a SELECTED fragment (a word, a
    /// sentence, a paragraph) rather than the whole body. Both paths must produce
    /// an identically-shaped template file.
    #[test]
    fn snippet_from_a_selection_carries_only_the_fragment() {
        let t = super::snippet_template("Frag", "just this sentence.");
        assert!(t.contains("kind: template"));
        assert!(t.contains("template_kind: snippet"));
        assert!(t.contains("title: Frag"));
        assert!(t.contains("just this sentence."));
        assert!(!t.contains("# Overview"), "nothing but the fragment");
    }

    /// A selection snippet and a whole-body snippet must be the SAME SHAPE — the
    /// header is identical; only the fragment differs.
    #[test]
    fn selection_and_whole_body_snippets_share_one_shape() {
        let from_sel = super::snippet_template("X", "fragment text");
        let from_body = template_content_from_note("---\nstage: seed\n---\nfragment text\n", "X", TemplateKind::Snippet);
        let head = |s: &str| s.split("---").take(2).collect::<Vec<_>>().join("---");
        assert_eq!(head(&from_sel), head(&from_body), "snippet header must not depend on the source path");
        assert!(!from_body.contains("stage: seed"), "source properties never enter a snippet");
    }

    #[test]
    fn snippet_preserves_a_crlf_fragment() {
        let t = super::snippet_template("X", "line one\r\nline two");
        assert!(t.contains("line one\r\nline two"), "fragment bytes preserved");
    }

    /// MIG-101 §A0 discipline: a CRLF note yields a CRLF whole-note template with
    /// the body byte-identical.
    #[test]
    fn crlf_note_round_trips_in_whole_mode() {
        let crlf = NOTE.replace('\n', "\r\n");
        let t = template_content_from_note(&crlf, "T", TemplateKind::Whole);
        assert!(t.contains("# Overview\r\nProse paragraph one.\r\n"), "CRLF body was rewritten");
    }

    /// A note with no frontmatter still becomes a valid whole-note template.
    #[test]
    fn bare_note_gains_a_frontmatter_block() {
        let t = template_content_from_note("Just a body\n", "Bare", TemplateKind::Whole);
        assert!(t.contains("kind: template"));
        assert!(t.contains("template_kind: whole"));
        assert!(t.contains("title: Bare"));
        assert!(t.ends_with("Just a body\n"), "body must be preserved verbatim");
    }

    #[test]
    fn kind_parse_is_forgiving_and_closed() {
        assert_eq!(TemplateKind::parse("whole"), Some(TemplateKind::Whole));
        assert_eq!(TemplateKind::parse("Note"), Some(TemplateKind::Whole));
        assert_eq!(TemplateKind::parse("frontmatter"), Some(TemplateKind::Frontmatter));
        assert_eq!(TemplateKind::parse("snippet"), Some(TemplateKind::Snippet));
        assert_eq!(TemplateKind::parse("book"), None);
    }

    #[test]
    fn stem_sanitizes_windows_reserved_chars() {
        assert_eq!(sanitize_template_stem("A/B:C?"), "A B C");
        assert_eq!(sanitize_template_stem("  "), "Template");
        assert_eq!(sanitize_template_stem("Name."), "Name");
    }
}


#[cfg(test)]
mod tests_persisted_read_refuses_emptiness {
    //! Triage concern #1 (2026-08-02), ranked APP-KILLER: a loader that maps "I could not
    //! read this" onto "you have none", after which an ordinary save writes that emptiness
    //! over the user's real file.
    //!
    //! Verified live in `universe.rs` before the fix: `load_registry` returned an EMPTY
    //! registry on read failure AND on parse failure — both indistinguishable from the
    //! genuinely-absent case — and `create_universe` does load -> push -> save. One second of
    //! a sync tool holding `universes.json`, then one new Universe, and the registry contained
    //! only that new one. Every previously registered Universe: forgotten.
    //!
    //! The sting recorded for posterity: the G6 audit's own comment above `atomic_write`
    //! DESCRIBES this defect in writing, and that audit hardened the write while leaving the
    //! read exactly as described — which made the destructive overwrite atomic.
    use super::{read_persisted_json, PersistedError};
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Reg {
        entries: Vec<String>,
    }

    fn tmp(name: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("cns-persist-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn absent_is_the_only_trustworthy_emptiness() {
        let p = tmp("absent.json");
        let got: Option<Reg> = read_persisted_json(&p).expect("absent must not be an error");
        assert_eq!(got, None, "a file that was never created genuinely means 'none yet'");
    }

    #[test]
    fn a_readable_file_round_trips() {
        let p = tmp("ok.json");
        std::fs::write(&p, br#"{"entries":["a","b"]}"#).unwrap();
        let got: Option<Reg> = read_persisted_json(&p).unwrap();
        assert_eq!(got.unwrap().entries, vec!["a".to_string(), "b".to_string()]);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn corrupt_json_is_an_error_not_an_empty_registry() {
        // The everyday cause: a half-written file from a sync tool, or a manual edit.
        let p = tmp("corrupt.json");
        std::fs::write(&p, br#"{"entries":["a","#).unwrap();
        let got = read_persisted_json::<Reg>(&p);
        assert!(got.is_err(), "corrupt MUST NOT present as empty — that is the app-killer");
        assert!(
            matches!(got.as_ref().unwrap_err(), PersistedError::Corrupt(_)),
            "a corrupt file is recoverable-by-setting-aside, not a transient lock"
        );
        assert!(
            got.unwrap_err().message().contains("Refusing to overwrite"),
            "the error must say why, so the caller cannot mistake it for 'no data'"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn a_zero_length_file_is_truncation_not_absence() {
        // The classic torn-write state. `serde_json` would reject it anyway, but the message
        // must not read like a parse problem — an empty file means the data was there.
        let p = tmp("empty.json");
        std::fs::write(&p, b"").unwrap();
        let got = read_persisted_json::<Reg>(&p);
        assert!(got.is_err(), "an empty file is a truncated file, never 'no data'");
        assert!(matches!(got.as_ref().unwrap_err(), PersistedError::Corrupt(_)));
        assert!(got.unwrap_err().message().contains("truncated"));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn whitespace_only_is_also_truncation() {
        let p = tmp("ws.json");
        std::fs::write(&p, b"   \n\t\n").unwrap();
        assert!(read_persisted_json::<Reg>(&p).is_err());
        let _ = std::fs::remove_file(&p);
    }

    /// RED-proof, kept in the suite: the shape this replaced. If someone "simplifies"
    /// `read_persisted_json` back to a lenient fallback, this documents what breaks.
    #[test]
    fn the_old_lenient_shape_cannot_tell_the_two_apart() {
        let absent = tmp("legacy-absent.json");
        let corrupt = tmp("legacy-corrupt.json");
        std::fs::write(&corrupt, br#"{"entries":["a","#).unwrap();

        // Exactly what `load_registry` used to do for both cases.
        let legacy = |p: &std::path::Path| -> Vec<String> {
            let Ok(d) = std::fs::read_to_string(p) else { return vec![] };
            serde_json::from_str::<Reg>(&d).map(|r| r.entries).unwrap_or_default()
        };

        assert_eq!(legacy(&absent), Vec::<String>::new());
        assert_eq!(legacy(&corrupt), Vec::<String>::new());
        // Identical answers for "you have none" and "I could not read your data".
        // The save path that follows cannot possibly do the right thing.
        assert!(read_persisted_json::<Reg>(&absent).unwrap().is_none());
        assert!(read_persisted_json::<Reg>(&corrupt).is_err());
        let _ = std::fs::remove_file(&corrupt);
    }
}

/// PJ-281 (tenth sweep) — the legacy Workbench adoption is a ONE-SHOT gate, so every way it can
/// go wrong is permanent. These pin the two that were live.
#[cfg(test)]
mod tests_pj281_legacy_collections_adoption {
    use super::*;

    fn dirs() -> (tempfile::TempDir, PathBuf) {
        let t = tempfile::tempdir().unwrap();
        let target = t.path().join("collections.json");
        (t, target)
    }

    #[test]
    fn nothing_to_adopt_is_none_not_an_error() {
        let (t, target) = dirs();
        assert!(adopt_legacy_collections(t.path(), &target).unwrap().is_none());
        assert!(!target.exists(), "and nothing is created");
    }

    #[test]
    fn a_good_legacy_file_is_adopted_and_retired() {
        let (t, target) = dirs();
        std::fs::write(
            t.path().join("workbench.json"),
            br#"[{"id":"c1","name":"Reading","items":[{"path":"/a.md"}]}]"#,
        )
        .unwrap();
        let adopted = adopt_legacy_collections(t.path(), &target).unwrap().expect("adopted");
        assert_eq!(adopted[0]["name"], "Reading");
        assert!(target.exists(), "collections.json is written");
        assert!(t.path().join("workbench.json.migrated").exists(), "the legacy is retired, not deleted");
        assert!(!t.path().join("workbench.json").exists());
    }

    /// **The one that mattered.** An unreadable/truncated legacy file must be an ERROR, never
    /// `Ok(empty)`. The frontend seeds a Starred collection from an empty list and persists it,
    /// which creates collections.json and closes this gate FOREVER — the user's whole Workbench
    /// set gone, with no error anywhere. Only the Err keeps the gate open for the next boot.
    #[test]
    fn an_unreadable_legacy_file_refuses_rather_than_reporting_emptiness() {
        let (t, target) = dirs();
        std::fs::write(t.path().join("workbench.json"), b"").unwrap(); // the truncated case
        let err = adopt_legacy_collections(t.path(), &target).unwrap_err();
        assert!(err.contains("NOT adopted"), "the message must say it did not adopt: {err}");
        assert!(!target.exists(), "NOTHING may be written — that write is what closes the gate");
        assert!(t.path().join("workbench.json").exists(), "and the legacy file must survive");
    }

    /// The write-before-parse defect found in the same block: a corrupt legacy file used to be
    /// copied verbatim into collections.json and the legacy retired, and only THEN did the parse
    /// fail — leaving every later boot with a present-but-unparseable collections.json and no way
    /// back. Parse first: nothing is written unless the payload is known good.
    #[test]
    fn a_corrupt_legacy_file_is_never_copied_over_the_gate() {
        let (t, target) = dirs();
        std::fs::write(t.path().join("workbench.json"), b"{not json at all").unwrap();
        assert!(adopt_legacy_collections(t.path(), &target).is_err());
        assert!(!target.exists(), "a corrupt payload must not become the adopted file");
        assert!(t.path().join("workbench.json").exists(), "nor may the legacy be retired");
        assert!(!t.path().join("workbench.json.migrated").exists());
    }
}
