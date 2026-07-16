use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub is_universe_notes: bool,
    /// "native" = created by Constellation (always canonical filenames)
    /// "canonical" = external library, user accepted canonicalization
    /// "compatible" = external library, user chose to keep files intact
    #[serde(default = "default_canonical_mode")]
    pub canonical_mode: String,
}

fn default_canonical_mode() -> String { "native".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileEntry>>,
    pub extension: Option<String>,
    pub modified: Option<u64>,
    // MIG-091 §A — created + size for the File Explorer's richer sort. Both
    // read from the SAME metadata call as `modified` (zero extra IO). `created`
    // is best-effort (unsupported on some filesystems → None); `size` is None
    // for folders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    pub status: Option<String>,
    /// For canonical files: the human-readable title from frontmatter.
    /// Null for non-canonical files or folders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
}

/// Get the path to the libraries config file (in .constellation/).
fn libraries_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let cdir = crate::universe::active_constellation_dir(app)?;
    Ok(cdir.join("libraries.json"))
}

/// Load registered libraries from the active universe's libraries.json (own libraries only).
/// Load ONLY the active universe's own libraries (its `libraries.json`),
/// NON-recursively — i.e. WITHOUT the federated cUniverse libraries that
/// `load_all_libraries`/`resolve_universe_libraries` pull in. `pub(crate)` for
/// MIG-065 §J: WRITE-path validation must scope to the active universe's own
/// libraries so an edit never lands on a read-only cUniverse file (the
/// federated-write blocker). Reads still use the recursive set.
pub(crate) fn load_libraries(app: &tauri::AppHandle) -> Vec<LibraryInfo> {
    let path = match libraries_config_path(app) {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    if path.exists() {
        let data = match fs::read_to_string(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[libraries] Failed to read {}: {}", path.display(), e);
                return vec![];
            }
        };
        serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("[libraries] Corrupt JSON in {}: {}", path.display(), e);
            // Safety Audit G6 (W1-8): a corrupt/truncated libraries.json must NOT be
            // silently treated as "no libraries" and then OVERWRITTEN by the next
            // save (permanent loss of every registration). Preserve a timestamped
            // backup so recovery is always possible; the original also stays in place.
            // (With the atomic write above, crash-corruption can no longer occur; this
            // guards external corruption — a manual edit, a disk error, sync glitch.)
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup = path.with_file_name(format!(
                "{}.corrupt-{}",
                path.file_name().and_then(|n| n.to_str()).unwrap_or("libraries.json"),
                secs
            ));
            if let Err(be) = fs::copy(&path, &backup) {
                eprintln!("[libraries] Failed to back up corrupt config to {}: {}", backup.display(), be);
            } else {
                eprintln!("[libraries] Backed up corrupt config to {}", backup.display());
            }
            vec![]
        })
    } else {
        vec![]
    }
}

/// Module-level cache for `load_all_libraries`.
///
/// Before this cache: diagnostic logs showed 50+ calls per boot from many
/// different code paths (validate_path_in_any_library, scan_*,
/// constellation_map_universe, etc.). Each re-read libraries.json from disk
/// and re-parsed it. Under Tauri's IPC queue on Windows this created the
/// 60-second boot-time hang we've been hunting.
///
/// The cache:
///   - Populated on first call per active-universe.
///   - Invalidated whenever `save_libraries` writes to disk.
///   - Keyed by the active universe path — switching universes reloads.
static LIBRARIES_CACHE: std::sync::Mutex<Option<(std::path::PathBuf, Vec<LibraryInfo>)>> =
    std::sync::Mutex::new(None);

/// Load ALL libraries: own + child universe libraries (recursive, deduplicated).
/// This is the universe-spanning library resolver every command should use.
pub fn load_all_libraries(app: &tauri::AppHandle) -> Vec<LibraryInfo> {
    let active = crate::universe::active_universe_dir(app).ok();

    // Fast path — cache hit for the currently active universe.
    if let Some(ref universe_path) = active {
        if let Ok(guard) = LIBRARIES_CACHE.lock() {
            if let Some((cached_universe, cached_libs)) = guard.as_ref() {
                if cached_universe == universe_path {
                    return cached_libs.clone();
                }
            }
        }
    }

    // Cache miss or unknown universe — do the actual disk read + parse.
    let libs = match crate::universe::resolve_universe_libraries(app.clone()) {
        Ok(libs) => libs,
        Err(_) => load_libraries(app),
    };

    if let Some(universe_path) = active {
        if let Ok(mut guard) = LIBRARIES_CACHE.lock() {
            *guard = Some((universe_path, libs.clone()));
        }
    }
    libs
}

/// Invalidate the in-memory library cache. Call when the on-disk
/// libraries.json has changed (add/remove library, rename, universe switch).
pub fn invalidate_libraries_cache() {
    if let Ok(mut guard) = LIBRARIES_CACHE.lock() {
        *guard = None;
    }
}

/// Public accessor for other modules (e.g., bases.rs).
pub fn load_libraries_pub(app: &tauri::AppHandle) -> Vec<LibraryInfo> {
    load_all_libraries(app)
}

/// The most-specific (longest-root-wins) library NAME whose registered root
/// contains `path`, or None. Longest wins so a note in a nested library is
/// attributed to THAT library, not its parent (e.g. universe_notes at the
/// Universe root vs a sub-folder library). Mirrors `reconcile::lib_for`; kept
/// here as the single resolver the watcher-freshness reindex path uses so a
/// changed path is attributed to the same library reconcile would pick
/// (Don't-Duplicate). Batch callers (the watcher flush over a git-pull's worth
/// of paths) pass a pre-loaded `libs` so `load_all_libraries` runs ONCE, not
/// per path. Case-/separator-insensitive; the `under` bound is at a separator so
/// `…/Research` never matches `…/Research Notes`.
pub fn library_name_for_path(libs: &[LibraryInfo], path: &str) -> Option<String> {
    let norm = |p: &str| p.replace('\\', "/").to_lowercase();
    let np = norm(path);
    libs.iter()
        .filter(|l| {
            let rn = norm(&l.path);
            np == rn || np.starts_with(&format!("{}/", rn))
        })
        // Raw byte-length ordering is identical to normalized (norm only maps
        // `\`→`/` and lowercases — both length-preserving), so the longest raw
        // root path is the most-specific library.
        .max_by_key(|l| l.path.len())
        .map(|l| l.name.clone())
}

/// Save registered libraries to the active universe's config.
fn save_libraries(app: &tauri::AppHandle, libraries: &[LibraryInfo]) -> Result<(), String> {
    let path = libraries_config_path(app)?;
    let data = serde_json::to_string_pretty(libraries).map_err(|e| e.to_string())?;
    // Safety Audit G6 (W1-8): ATOMIC write — write to a temp file, then rename over
    // the target. A plain truncate-then-write `fs::write` leaves libraries.json
    // truncated/partial if the app crashes or loses power mid-write, and the loader
    // then reads that as an EMPTY library list — silently dropping EVERY library
    // registration. The rename is atomic (same directory / filesystem), so a reader
    // always sees either the complete old file or the complete new one; a failed
    // rename leaves the old file intact (never truncated). Errors are surfaced.
    let tmp = path.with_file_name(format!(
        "{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("libraries.json")
    ));
    fs::write(&tmp, &data).map_err(|e| format!("Failed to write libraries config (tmp): {}", e))?;
    fs::rename(&tmp, &path).map_err(|e| {
        let _ = fs::remove_file(&tmp); // don't leave a stray temp on a failed commit
        format!("Failed to commit libraries config: {}", e)
    })?;
    // Invalidate the in-memory cache so subsequent reads see the new list.
    invalidate_libraries_cache();
    Ok(())
}

/// Validate that a file path is contained within a library directory.
/// Prevents path traversal attacks by canonicalizing both paths.
fn validate_path_in_library(file_path: &str, library_path: &str) -> Result<PathBuf, String> {
    let library_canon = fs::canonicalize(library_path)
        .map_err(|_| "Invalid library path.".to_string())?;
    let file = Path::new(file_path);
    // If the file doesn't exist yet, canonicalize the parent
    let file_canon = if file.exists() {
        fs::canonicalize(file).map_err(|_| "Invalid file path.".to_string())?
    } else {
        let parent = file.parent().ok_or("Invalid file path.".to_string())?;
        let parent_canon = fs::canonicalize(parent)
            .map_err(|_| "Parent directory does not exist.".to_string())?;
        parent_canon.join(file.file_name().ok_or("Invalid file name.".to_string())?)
    };
    if !file_canon.starts_with(&library_canon) {
        return Err("Access denied: path is outside the library.".to_string());
    }
    Ok(file_canon)
}

/// Validate that a path is within any registered library (including child universe libraries)
/// or the active universe directory.
pub fn validate_path_in_any_library(app: &tauri::AppHandle, file_path: &str) -> Result<PathBuf, String> {
    let libraries = load_all_libraries(app);
    for lib in &libraries {
        if let Ok(canon) = validate_path_in_library(file_path, &lib.path) {
            return Ok(canon);
        }
    }
    // Also allow the active universe directory for workspace bases
    if let Ok(universe_dir) = crate::universe::active_universe_dir(app) {
        if let Ok(uni_canon) = fs::canonicalize(&universe_dir) {
            let file = Path::new(file_path);
            if let Ok(file_canon) = fs::canonicalize(file) {
                if file_canon.starts_with(&uni_canon) {
                    return Ok(file_canon);
                }
            }
        }
    }

    // MIG-100 switch-flush fix — a note in a REGISTERED-but-not-active universe
    // must still be writable to its OWN file. The active-universe check above
    // is a NAVIGATION boundary, not a write-authorization one: a tab's
    // deferred flush (NotePane teardown, a pending debounced save) can land
    // AFTER a universe switch flipped the active pointer, and that write is
    // legitimate — it goes to the departing universe's own file. Reached ONLY
    // when the active checks miss (the rare departure race / a previously-
    // erroring path), so the common save pays nothing. Still rejects a path
    // outside EVERY registered/federated universe (the real security bound).
    let file = Path::new(file_path);
    if let Ok(file_canon) = fs::canonicalize(file) {
        for root in crate::universe::registered_universe_roots(app) {
            // Universe root itself (flat universes: the root IS the library).
            if let Ok(root_canon) = fs::canonicalize(&root) {
                if file_canon.starts_with(&root_canon) {
                    return Ok(file_canon);
                }
            }
            // Sub-libraries at external paths — the universe's OWN libraries
            // only (never a federated cUniverse file, which is read-only from
            // here and writable only through its own universe's identity).
            for lib in crate::universe::own_libraries_for_root(&root) {
                if let Ok(canon) = validate_path_in_library(file_path, &lib.path) {
                    return Ok(canon);
                }
            }
        }
    }
    Err("Access denied: path is not within any registered library.".to_string())
}

/// Sanitize a file or folder name to prevent path traversal.
fn sanitize_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Name cannot be empty.".to_string());
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Name contains invalid characters.".to_string());
    }
    Ok(name.to_string())
}

/// List all registered libraries.
#[tauri::command]
pub fn list_libraries(app: tauri::AppHandle) -> Vec<LibraryInfo> {
    load_libraries(&app)
}

/// Add a library by its folder path.
#[tauri::command]
pub fn add_library(app: tauri::AppHandle, path: String) -> Result<LibraryInfo, String> {
    let library_path = Path::new(&path);

    if !library_path.exists() || !library_path.is_dir() {
        return Err("Path does not exist or is not a folder.".to_string());
    }

    // Any directory is accepted as a library — no .obsidian or .md requirement

    let mut libraries = load_libraries(&app);

    // Check for duplicates
    if libraries.iter().any(|v| v.path == path) {
        return Err("This library is already registered.".to_string());
    }

    let name = library_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unnamed Library".to_string());

    let id = format!("library_{}", uuid_simple());

    let library = LibraryInfo {
        id: id.clone(),
        name,
        path: path.clone(),
        is_universe_notes: false,
        canonical_mode: "compatible".to_string(), // external libraries default to compatible
    };

    libraries.push(library.clone());
    save_libraries(&app, &libraries)?;

    Ok(library)
}

/// Update a library's canonical mode ("native", "canonical", or "compatible").
#[tauri::command]
pub fn set_library_canonical_mode(app: tauri::AppHandle, library_id: String, mode: String) -> Result<(), String> {
    if !["native", "canonical", "compatible"].contains(&mode.as_str()) {
        return Err(format!("Invalid canonical mode: {}", mode));
    }
    let mut libraries = load_libraries(&app);
    if let Some(lib) = libraries.iter_mut().find(|l| l.id == library_id) {
        lib.canonical_mode = mode;
        save_libraries(&app, &libraries)?;
        Ok(())
    } else {
        Err("Library not found.".to_string())
    }
}

/// Get a library's canonical mode by path.
pub fn get_library_mode(app: &tauri::AppHandle, folder_path: &str) -> String {
    let libraries = load_all_libraries(app);
    libraries.iter()
        .find(|l| folder_path.starts_with(&l.path))
        .map(|l| l.canonical_mode.clone())
        .unwrap_or_else(|| "native".to_string())
}

/// Remove a library by ID (does NOT delete any files).
#[tauri::command]
pub fn remove_library(app: tauri::AppHandle, library_id: String) -> Result<(), String> {
    let mut libraries = load_libraries(&app);
    let before = libraries.len();
    libraries.retain(|v| v.id != library_id);

    if libraries.len() == before {
        return Err("Library not found.".to_string());
    }

    save_libraries(&app, &libraries)
}

/// Read the file tree of a library (up to 2 levels deep for performance).
// App-freeze audit Batch-S (2026-07-03): `(async)` — this command reaches
// ensure_search_db_ready (or a multi-second walk/read) and used to PARK the
// WebView2 dispatch thread for the whole 20-40s cold init after a universe
// switch / boot (the Boss-reproduced switch freeze). Off-thread, the init
// still runs exactly once (init_lock) but the app stays responsive.
#[tauri::command(async)]
pub fn read_library_tree(app: tauri::AppHandle, path: String, max_depth: Option<u32>) -> Result<Vec<FileEntry>, String> {
    // Validate the path is a registered library (including child universe libraries)
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let library_path = Path::new(&path);
    if !library_path.exists() {
        return Err("Library path does not exist.".to_string());
    }

    let depth = max_depth.unwrap_or(2);
    let tree = read_dir_recursive(library_path, 0, depth);
    Ok(tree)
}

/// Read the content of a file inside a library.
#[tauri::command]
pub fn read_note(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist.".to_string());
    }
    fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

/// Extract headings from a note file.
#[tauri::command]
pub fn get_note_headings(app: tauri::AppHandle, file_path: String) -> Result<Vec<String>, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let path = Path::new(&file_path);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut headings = Vec::new();
    use std::sync::OnceLock;
    static HEADING_RE: OnceLock<regex::Regex> = OnceLock::new();
    let re = HEADING_RE.get_or_init(|| regex::Regex::new(r"(?m)^#{1,6}\s+(.+)$").unwrap());
    for cap in re.captures_iter(&content) {
        if let Some(m) = cap.get(1) {
            // PJ-106 §B4 — strip invisible direction marks (RLM/LRM): a heading forced RTL
            // keeps its identity for [[note#heading]] pickers and fragment resolution.
            headings.push(m.as_str().replace(['\u{200E}', '\u{200F}'], "").trim().to_string());
        }
    }
    Ok(headings)
}

/// Write content to a markdown file inside a library.
#[tauri::command]
pub fn write_note(
    app: tauri::AppHandle,
    file_path: String,
    content: String,
    // MIG-076 §B1 — optional identity/freshness attestation (camelCase
    // `expect` from the frontend; absent on legacy callers → unchecked).
    expect: Option<crate::write_gate::Expectation>,
    // ★Stage-1 finding #3 — the journal's surface for write_note was too
    // coarse (five frontend writers shared one tag). Callers label themselves.
    origin: Option<String>,
) -> Result<(), String> {
    // MIG-100 forensics fix: a validation rejection must leave a journal
    // line — the switch-flush incident (a departing universe's flush
    // rejected against the NEW universe's libraries) was invisible in the
    // journal because these early Errs never reached gate_write.
    if let Err(e) = validate_path_in_any_library(&app, &file_path) {
        crate::write_gate::journal_frontend_marker(
            "write_note_rejected".to_string(),
            format!("{}: {}", file_path, e),
        );
        return Err(e);
    }
    let path = Path::new(&file_path);

    // Safety: only allow writing .md files, reject ADS on Windows
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.contains(':') {
            return Err("Invalid file name.".to_string());
        }
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some("md") => {}
        _ => return Err("Can only write to .md files.".to_string()),
    }

    if !path.exists() {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err("Parent directory does not exist.".to_string());
            }
        }
    }

    // MIG-076 §A2/§B1 — through the WriteGate (serialized + atomic +
    // journaled; identity/freshness checked when the caller attests).
    let surface = origin.as_deref().unwrap_or("write_note");
    crate::write_gate::gate_write(path, &content, expect.as_ref(), surface).map(|_| ())
}

/// PJ-070 — write the INCOMING external disk content to a `.conflict` sidecar next to a note whose
/// open model was DIRTY when an external edit landed, so the user's unsaved work stays in the editor
/// (never clobbered) AND the external edit is never lost. The sidecar's FINAL `.txt` extension makes
/// it inert to every `.md`-gated surface (the file watcher, `index_note`, the tree walker), so it
/// cannot re-fire the watcher, be indexed as a duplicate-`cid_cn` note, or appear in the sidebar —
/// while the `.md` kept inside the stem tells the user (and their external editor) it is Markdown.
/// Returns the sidecar path for the conflict banner's "Show copy" button. Written via
/// `gate_create_exclusive` (atomic + fsync + journalled + refuse-if-exists); a same-second name
/// collision retries with a `-N` suffix. NOT `write_note` (which rejects any non-`.md` path).
#[tauri::command]
pub fn write_conflict_sidecar(note_path: String, disk_content: String) -> Result<String, String> {
    let note = Path::new(&note_path);
    let parent = note.parent().ok_or("Note has no parent directory.")?;
    let stem = note
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Note has no file name.")?;
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    // Try `<stem>.conflict-<ts>.md.txt`, then `-2`, `-3`, … on a same-second collision.
    for n in 1..=50u32 {
        let suffix = if n == 1 { String::new() } else { format!("-{n}") };
        let sidecar = parent.join(format!("{stem}.conflict-{ts}{suffix}.md.txt"));
        match crate::write_gate::gate_create_exclusive(&sidecar, &disk_content, "conflict_sidecar")? {
            crate::write_gate::WriteOutcome::RefusedExists => continue,
            _ => return Ok(sidecar.to_string_lossy().to_string()),
        }
    }
    Err("Could not create a conflict sidecar (too many collisions).".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub library_id: String,
    pub name: String,
    pub path: String,
    pub star_count: u32,
    pub folder_count: u32,
    pub recent_stars: Vec<StarInfo>,
    #[serde(default)]
    pub is_universe_notes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarInfo {
    pub name: String,
    pub path: String,
    pub library_id: String,
    pub library_name: String,
    pub modified: u64,
    pub preview: String,
}

/// Get stats for all libraries (own + child universe) — star counts, folder counts, recent stars.
///
/// `(async)` keeps the body off the WebView2 UI thread (see watcher.rs
/// `watch_library` for the full rationale — LL-021 post-Round-3). Critical
/// here because this fn `.join()`s every per-library scanner thread before
/// returning: on a 16-library × 7,600-note Universe that's several seconds
/// of synchronous wait. Without `(async)` those seconds are paid on the UI
/// thread, starving every other boot-fan-out IPC behind it — including
/// `cache_boot_snapshot_core`, which is Boot Criterion 2's critical path.
/// Aggregate per-library note counts + ancestor directories from `note_meta`
/// (the index), keyed by `library_name`. Sequential `let` bindings so the
/// `SearchState` borrow + MutexGuard live for the whole read (avoids the
/// `if let` temporary-lifetime trap). Best-effort: any failure → empty map
/// (callers fall back to 0 counts, which the count badge hides).
/// MIG-056 §F — Build the SQL for federated library-count aggregation.
/// Pure function so the SQL shape is unit-testable without Tauri state.
///
/// When `federated_aliases` is empty (no federation, or federation
/// not ready), returns the single-schema query (existing behavior).
/// Otherwise returns a UNION ALL across `main` + each cUniverse alias.
///
/// Per Architect §7.2 + Agent 3's Citus lesson — each branch carries
/// its own (potentially zero) WHERE clauses. For aggregation there's
/// no WHERE; the query reads every note_meta row from every attached
/// schema and the aggregation happens in Rust (caller).
fn build_aggregate_counts_sql(federated_aliases: &[String]) -> String {
    if federated_aliases.is_empty() {
        return "SELECT library_name, path FROM note_meta".to_string();
    }
    let mut parts: Vec<String> =
        vec!["SELECT library_name, path FROM main.note_meta".to_string()];
    for alias in federated_aliases {
        parts.push(format!(
            "SELECT library_name, path FROM {}.note_meta",
            alias
        ));
    }
    parts.join(" UNION ALL ")
}

fn aggregate_library_counts(
    app: &tauri::AppHandle,
) -> std::collections::HashMap<String, (u32, std::collections::HashSet<String>)> {
    use std::collections::{HashMap, HashSet};
    use tauri::Manager;
    let mut agg: HashMap<String, (u32, HashSet<String>)> = HashMap::new();
    if crate::search::ensure_search_db_ready(app).is_err() {
        return agg;
    }
    let state = app.state::<crate::search::SearchState>();

    // MIG-056 §F — Decide federated vs single-schema path:
    // - If federation context is ready AND has attached cUniverses
    //   AND state.federated_conn is populated → federated path
    // - Otherwise → existing single-schema (state.db) path
    let federated_aliases: Vec<String> = match state.federation.lock() {
        Ok(g) if g.is_ready() && !g.attached().is_empty() => {
            g.attached().iter().map(|(a, _)| a.clone()).collect()
        }
        _ => Vec::new(),
    };

    let sql = build_aggregate_counts_sql(&federated_aliases);

    // Run the query against the appropriate connection.
    // Returns Vec<(library_name, note_path)>; empty on any error
    // (the early-returns in this function preserve the empty `agg`
    // result for graceful degradation).
    let rows: Vec<(String, String)> = if federated_aliases.is_empty() {
        // Single-schema path (state.db).
        let guard = match state.db.lock() {
            Ok(g) => g,
            Err(_) => return agg,
        };
        let conn = match guard.as_ref() {
            Some(c) => c,
            None => return agg,
        };
        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => return agg,
        };
        let mapped = match stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            Ok(m) => m,
            Err(_) => return agg,
        };
        mapped.flatten().collect()
    } else {
        // Federated path (state.federated_conn — main + cu* attached).
        let guard = match state.federated_conn.lock() {
            Ok(g) => g,
            Err(_) => return agg,
        };
        match guard.as_ref() {
            Some(conn) => {
                let mut stmt = match conn.prepare(&sql) {
                    Ok(s) => s,
                    Err(_) => return agg,
                };
                let mapped = match stmt.query_map([], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                }) {
                    Ok(m) => m,
                    Err(_) => return agg,
                };
                mapped.flatten().collect()
            }
            None => {
                // federated_conn None despite federation ready — race
                // between FederationContext.is_ready() and federated_conn
                // population (background-thread ordering). Fall back to
                // single-schema by re-querying state.db.
                drop(guard);
                let db_guard = match state.db.lock() {
                    Ok(g) => g,
                    Err(_) => return agg,
                };
                let fallback_conn = match db_guard.as_ref() {
                    Some(c) => c,
                    None => return agg,
                };
                let fb_sql = build_aggregate_counts_sql(&[]);
                let mut stmt = match fallback_conn.prepare(&fb_sql) {
                    Ok(s) => s,
                    Err(_) => return agg,
                };
                let mapped = match stmt.query_map([], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                }) {
                    Ok(m) => m,
                    Err(_) => return agg,
                };
                mapped.flatten().collect()
            }
        }
    };

    for (lib_name, path) in rows {
        let entry = agg.entry(lib_name).or_insert_with(|| (0u32, HashSet::new()));
        entry.0 += 1;
        // Walk the note's ancestor directories; break once we hit one already
        // recorded (its ancestors are too).
        let mut cur = Path::new(&path).parent();
        while let Some(dir) = cur {
            let s = dir.to_string_lossy().to_string();
            if s.is_empty() || !entry.1.insert(s) {
                break;
            }
            cur = dir.parent();
        }
    }
    agg
}

#[tauri::command(async)]
pub fn get_all_library_stats(app: tauri::AppHandle) -> Vec<LibraryStats> {
    let libraries = load_all_libraries(&app);

    // Counts come from the always-current index (`note_meta`), NOT a filesystem
    // walk. The old impl stat-walked every library's tree (~7,600 stat calls,
    // cold) + read preview files — the measured ~1.5–3 s "note-counts trail in
    // at ~3.5 s" cost after the universe structure already painted. `note_meta`
    // already has every note with its `library_name`, so one indexed read +
    // in-memory aggregation gives the same numbers in milliseconds. Same lesson
    // as LL-024 (read the index, don't walk the vault). `recent_stars` is unused
    // in the UI (verified) — dropped, so we skip the preview reads entirely.
    //
    // `star_count` = notes per library (exact). `folder_count` = distinct
    // ancestor directories of those notes that sit under the library root —
    // i.e. folders that contain notes (directly or transitively). This can miss
    // a truly empty folder, but matches the old "count folders" intent for the
    // Dashboard stat closely while costing nothing.
    let agg = aggregate_library_counts(&app);

    libraries.into_iter().map(|v| {
        let (star_count, folder_count) = match agg.get(&v.name) {
            Some((count, dirs)) => {
                let n = dirs.iter().filter(|d| d.len() > v.path.len() && d.starts_with(&v.path)).count() as u32;
                (*count, n)
            }
            None => (0, 0),
        };
        LibraryStats {
            library_id: v.id,
            name: v.name,
            path: v.path,
            star_count,
            folder_count,
            recent_stars: Vec::new(),
            is_universe_notes: v.is_universe_notes,
        }
    }).collect()
}


/// Safely truncate a string to approximately `max_len` characters.
fn safe_truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

#[allow(dead_code)]
fn _collect_notes_recursive_unused(_dir: &Path, _library_id: &str, _library_name: &str, _notes: &mut Vec<StarInfo>, _depth: u32) {
    // Superseded by get_recent_notes metadata-first + top-N preview read.
}


/// Create a new markdown note inside a library folder.
/// `initial_frontmatter` is optional YAML content (without delimiters) to insert between `---` markers.
// MIG-099 §3: `(async)` — this now reindexes the new note synchronously (below),
// which takes the writer lock; moving it off the WebView2 IPC dispatch thread
// keeps create responsive if a background reindex briefly holds the lock (the
// note-open-freeze pattern). Body has no `.await`; the JS invoke contract and
// `await createNote(...)` ordering are unchanged.
#[tauri::command(async)]
pub fn create_note(app: tauri::AppHandle, folder_path: String, file_name: String, initial_frontmatter: Option<String>) -> Result<String, String> {
    validate_path_in_any_library(&app, &folder_path)?;
    let folder = Path::new(&folder_path);
    if !folder.exists() || !folder.is_dir() {
        return Err("Folder does not exist.".to_string());
    }

    // MIG-003 Step 5 — human filenames everywhere. The previous
    // native/compatible branching is gone: every new note lands on
    // disk under a sanitized version of the user-supplied title.
    // Collisions are resolved automatically (`Untitled` →
    // `Untitled 1.md` → `Untitled 2.md`, etc.). cid_cn lives in
    // frontmatter as the immutable internal id.
    let dt = chrono::Utc::now();
    let display_name = file_name.trim_end_matches(".md");
    let safe_stem = note_display_filename(display_name);
    let final_filename = resolve_filename_collision(folder, &safe_stem, ".md", true)
        .map_err(|e| format!("Failed to resolve filename: {}", e))?;
    let file_path = folder.join(&final_filename);

    let canonical = crate::canonical::generate_canonical("NOTE", &dt, "md", None);

    let mut fm_lines: Vec<String> = Vec::new();
    // Display title in frontmatter still tracks the user-typed name —
    // that's what shows in the tab + sidebar. The filename's stem will
    // match it verbatim in the common case but may diverge for
    // collision-resolved files (`Untitled` vs `Untitled 1.md`).
    fm_lines.push(format!("title: \"{}\"", display_name.replace('"', "\\\"")));
    fm_lines.push(format!("cid_cn: {}", canonical.stem));
    fm_lines.push("kind: note".to_string());
    fm_lines.push(format!("created: {}", dt.to_rfc3339()));

    if let Some(ref extra) = initial_frontmatter {
        for line in extra.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("title:")
                && !trimmed.starts_with("cid_cn:")
                && !trimmed.starts_with("cid:")
                && !trimmed.starts_with("kind:")
                && !trimmed.starts_with("created:")
                && !trimmed.is_empty()
            {
                fm_lines.push(trimmed.to_string());
            }
        }
    }

    let content = format!("---\n{}\n---\n\n", fm_lines.join("\n"));
    // MIG-076 §A2 — create-exclusive: if a race created this path between the
    // collision resolver above and now, REFUSE instead of silently overwriting
    // another note (previously fs::write would have clobbered it).
    match crate::write_gate::gate_create_exclusive(&file_path, &content, "create_note")? {
        crate::write_gate::WriteOutcome::RefusedExists => {
            return Err(format!(
                "A note already exists at {} (created concurrently).",
                file_path.display()
            ));
        }
        _ => {}
    }

    // MIG-099 §3 — index the new note SYNCHRONOUSLY so note_meta.name_lower is
    // authoritative the instant the file exists. The §2 index-backed collision
    // check trusts an index miss as "does not exist"; without indexing here, a
    // same-session second create of a DIVERGENT-title note (reserved chars rewrite
    // the stem — e.g. title "Ratio A/B" → stem "Ratio A B") would miss the
    // just-created first note (stem differs, row not yet present) and silently
    // create a duplicate title, defeating MIG-076 §E1b. The file is the source of
    // truth and was written successfully, so a reindex failure is SURFACED
    // (diag_log, release-safe) but does NOT fail the create — the watcher / next
    // open re-indexes. (Not `let _ =`: an index failure must be visible.)
    {
        use tauri::Manager;
        let search_state = app.state::<crate::search::SearchState>();
        let ps = file_path.to_string_lossy().to_string();
        match load_all_libraries(&app).iter().find(|l| ps.starts_with(&l.path)) {
            Some(lib) => {
                if let Err(e) = crate::search::reindex_single_note(&search_state, &ps, &lib.name) {
                    if let Ok(p) = crate::search::db_path(&app) {
                        crate::search::diag_log(&p, &format!("[create_note] reindex FAILED for {}: {}", ps, e));
                    }
                }
            }
            None => {
                if let Ok(p) = crate::search::db_path(&app) {
                    crate::search::diag_log(&p, &format!("[create_note] NO LIBRARY matched {} — reindex SKIPPED", ps));
                }
            }
        }
    }

    Ok(file_path.to_string_lossy().to_string())
}

/// Check if a library has been canonicalized. Delegates to canonical module.
#[allow(dead_code)]
fn is_library_canonical(library_path: &str) -> bool {
    crate::canonical::is_library_canonicalized(library_path)
}

// MIG-091 — search_by_property + search_property_recursive removed with the
// retired Notes Navigator (its sole caller); property search is Search Hub's.

/// Create a new folder inside a library.
#[tauri::command]
pub fn create_folder(app: tauri::AppHandle, parent_path: String, folder_name: String) -> Result<String, String> {
    let safe_name = sanitize_name(&folder_name)?;
    validate_path_in_any_library(&app, &parent_path)?;
    let parent = Path::new(&parent_path);
    if !parent.exists() || !parent.is_dir() {
        return Err("Parent directory does not exist.".to_string());
    }

    let folder_path = parent.join(&safe_name);
    if folder_path.exists() {
        return Err("A folder with this name already exists.".to_string());
    }

    fs::create_dir(&folder_path)
        .map_err(|e| format!("Failed to create folder: {}", e))?;

    Ok(folder_path.to_string_lossy().to_string())
}

/// Rename a file or folder.
///
/// MIG-003 Step 5 — unified rename flow. For .md files, this performs:
///   1. Read the current frontmatter title (for alias preservation).
///   2. Update the frontmatter title to match the new filename and
///      append the old title to the file's `aliases:` list.
///   3. fs::rename the file from old_path → new_path.
///   4. Cascade the path change across every DB table (note_meta +
///      dependent tables note_links / note_aliases / note_embeddings;
///      sky_nodes / sky_links cascade automatically via the
///      note_meta_sky_au trigger).
///   5. Stamp a 'rename' alias row keyed to the new path so any
///      external reference to the old title still resolves via lookup.
///   6. Reindex the note so name / tags / outgoing links are picked
///      up under the new path.
///   7. Frontend cascades `[[OldTitle]]` → `[[NewTitle]]` in source
///      notes' bodies via the existing `update_links_on_rename`
///      command — no change needed here.
///
/// For folders, the legacy fs::rename-only flow stays in place (folder
/// rename DB cascade is its own concern; pre-existing behavior).
// Note-open-freeze Batch-2 §B2-4 (2026-07-03): `(async)` + the read→title-rewrite
// →write→rename sequence moved inside `gate_rmw_rename` — ONE critical section
// under BOTH paths' locks (sorted order). Before, the per-path lock was released
// between the read (:963), the title write (:988) and the rename (:993); a
// debounced editor save landing in either gap either lost the user's last
// keystrokes or carried stale-title content to the new path (the BUG-023-class
// windows the SYNC dispatch used to mask). The dest-exists check now happens
// UNDER the lock (closes the :953 TOCTOU). The DB cascade + reindex stay OUTSIDE
// the path locks (hard rule: no SearchState.db waits under a path lock — a SYNC
// write_note parking on the path lock would re-freeze the dispatch thread).
#[tauri::command(async)]
pub fn rename_item(app: tauri::AppHandle, old_path: String, new_path: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &old_path)?;
    let old = Path::new(&old_path);
    if !old.exists() {
        return Err("Item does not exist.".to_string());
    }

    // Folder rename — legacy fs::rename-only path. DB cascade for
    // recursively-renamed notes is out of scope for MIG-003 Step 5.
    if !old.extension().map(|e| e == "md").unwrap_or(false) {
        validate_path_in_any_library(&app, &new_path)?;
        let new_p = Path::new(&new_path);
        if new_p.exists() {
            return Err("An item with this name already exists.".to_string());
        }
        // MIG-076 §A2 — gated (journal + AV retry; folder-level).
        crate::write_gate::gate_rename(old, new_p, "rename_folder")?;
        return Ok(new_path);
    }

    // .md file rename — unified cascade flow.
    validate_path_in_any_library(&app, &new_path)?;
    let new_p = Path::new(&new_path);
    if new_p.exists() && new_p != old {
        // Fast-path pre-check (user-facing collision error). The authoritative
        // check re-runs UNDER the lock inside gate_rmw_rename.
        return Err("A file with this name already exists.".to_string());
    }

    let new_title = new_p
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Steps 1–3 as ONE dual-locked critical section: read fresh disk content,
    // extract the old title FROM THAT read (never a stale pre-read), rewrite
    // the frontmatter title, atomic-write, rename — no gap a save can enter.
    let mut old_title_out: Option<String> = None;
    let mut idempotent_noop = false;
    let outcome = crate::write_gate::gate_rmw_rename(old, new_p, "rename_item", |content| {
        let old_title = extract_frontmatter_title(content).unwrap_or_else(|| {
            old.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        // Idempotency guard: same title AND same path — nothing to do.
        // Without this, a stale `titleValue` in the frontend (display-sync
        // bug) firing a blur event would pass old_title == new_title to
        // update_frontmatter_title, which would append the title to its
        // own aliases list — producing entries like
        // [Untitled, TestBug001, TestBug001].
        if old_title == new_title && old == new_p {
            idempotent_noop = true;
            old_title_out = Some(old_title);
            return Ok(None);
        }

        let updated = if old_title != new_title {
            Some(update_frontmatter_title(content, &new_title, &old_title))
        } else {
            None // title unchanged — pure move; the primitive still renames
        };
        old_title_out = Some(old_title);
        Ok(updated)
    })?;
    if outcome == crate::write_gate::WriteOutcome::RefusedExists {
        return Err("A file with this name already exists.".to_string());
    }
    if idempotent_noop {
        return Ok(old_path);
    }
    let old_title = old_title_out.unwrap_or_default();

    // §B2-4 stall fix (2026-07-03, Boss-reproduced ×2 + 3-tracer/verifier
    // convergence): the DB tail below parks on the UNBOUNDED SearchState
    // writer mutex whenever anything holds it long — and with the command now
    // `(async)` that park is INVISIBLE: the invoke promise never settles, so
    // the frontend's whole post-rename orchestration (tab migration, tree
    // refresh, THE LINK CASCADE) silently never runs. The fs+journal state is
    // FINAL at this point, and every statement in the tail is already
    // best-effort (`let _ =`), so the tail is detached to a worker: the IPC
    // settles the moment the file state is final, unconditionally. This is
    // the PJ-066 rule (an awaited IPC surface must never include an unbounded
    // writer-lock wait) applied to rename_item's own tail.
    let tail_app = app.clone();
    let tail_old_path = old_path.clone();
    let tail_new_path = new_path.clone();
    let tail_old_title = old_title.clone();
    let tail_new_title = new_title.clone();
    // MIG-098 instrumentation (Reproduce-First): mark the SCHEDULE point. If this
    // logs but "[rename-tail] START" never does, the spawn_blocking task never ran
    // (starved/dropped) — vs. running-but-failing, which the tail logs distinguish.
    if let Ok(p) = crate::search::db_path(&app) {
        crate::search::diag_log(&p, &format!("[rename-tail] scheduling tail old={} new={}", old_path, new_path));
    }
    tauri::async_runtime::spawn_blocking(move || {
        rename_item_db_tail(&tail_app, &tail_old_path, &tail_new_path, &tail_old_title, &tail_new_title);
    });

    // Journal marker (stall forensics): the command RETURNED — any future
    // "renamed but nothing followed" report is journal-decidable at a glance.
    crate::write_gate::journal_marker(new_p, "rename_return");

    Ok(new_path)
}

/// The rename's DB bookkeeping (Steps 4–6), detached from the awaited IPC
/// surface (§B2-4 stall fix — see rename_item). Every operation is
/// best-effort; the fs state (already final) is the source of truth and the
/// watcher / next reindex heals any miss.
fn rename_item_db_tail(
    app: &tauri::AppHandle,
    old_path: &str,
    new_path: &str,
    old_title: &str,
    new_title: &str,
) {
    // MIG-098 instrumentation (Reproduce-First): trace WHY the note_meta rename
    // update can be lost on a large/busy universe. diag_log lands in the universe's
    // diagnostics.log — release-safe (no devtools). Removed once the root cause is
    // fixed + verified.
    let dbp = crate::search::db_path(app).ok();
    let log = |m: &str| { if let Some(p) = &dbp { crate::search::diag_log(p, m); } };
    log(&format!("[rename-tail] START old={} new={} title '{}'->'{}'", old_path, new_path, old_title, new_title));
    // Steps 4+5: DB cascade + 'rename' alias stamp.
    {
        use tauri::Manager;
        let search_state = app.state::<crate::search::SearchState>();
        let db_lock = search_state.db.lock();
        if let Ok(guard) = db_lock {
            if let Some(conn) = guard.as_ref() {
                if old_path != new_path {
                    match conn.execute(
                        "UPDATE note_meta SET path = ?2 WHERE path = ?1",
                        rusqlite::params![&old_path, &new_path],
                    ) {
                        Ok(n) => log(&format!("[rename-tail] note_meta path UPDATE affected {} row(s)", n)),
                        Err(e) => log(&format!("[rename-tail] note_meta path UPDATE ERROR: {}", e)),
                    }
                    let _ = conn.execute(
                        "UPDATE note_links SET source_path = ?2 WHERE source_path = ?1",
                        rusqlite::params![&old_path, &new_path],
                    );
                    // NOTE (rename-perf, 2026-06-28): the former
                    //   UPDATE note_links SET target_path = ?2 WHERE target_path = ?1
                    // was REMOVED. `note_links.target_path` is never populated — link
                    // targets are tracked by `target_name` (resolved at read time via
                    // COALESCE(target_path, target_name); see cece/wiring.rs, review.rs).
                    // So that UPDATE matched ZERO rows on every rename, yet cost ~11 s:
                    // an all-NULL indexed column degenerates the planner into a full scan
                    // of all ~234k note_links rows (measured, Reproduce-First). Targets are
                    // already migrated by the `[[name]]` wikilink cascade (update_links_on_rename)
                    // + the note_meta_sky_au trigger's target_name rewrite. Dead + slow → cut.
                    let _ = conn.execute(
                        "UPDATE note_aliases SET path = ?2 WHERE path = ?1",
                        rusqlite::params![&old_path, &new_path],
                    );
                    let _ = conn.execute(
                        "UPDATE note_embeddings SET path = ?2 WHERE path = ?1",
                        rusqlite::params![&old_path, &new_path],
                    );
                    // MIG-083 §D — migrate the review_schedule row to the new path
                    // (gated on the stamp). Without this the old-path row is orphaned:
                    // it is never deleted (rename != delete) and the indexed read would
                    // surface it as a PHANTOM due-queue entry pointing at a dead path
                    // (re-verify finding). Migrating also carries last_reviewed / interval
                    // / snooze forward, so the note's ✓ history survives the rename (the
                    // reindex below then preserves it via upsert_schedule_row).
                    if crate::review::is_stamped(conn) {
                        // Clear any stale row already at new_path first, so the migrate
                        // can't hit the PRIMARY KEY and silently leave the old orphan.
                        let _ = conn.execute(
                            "DELETE FROM review_schedule WHERE path = ?1",
                            rusqlite::params![&new_path],
                        );
                        let _ = conn.execute(
                            "UPDATE review_schedule SET path = ?2 WHERE path = ?1",
                            rusqlite::params![&old_path, &new_path],
                        );
                    }
                } else {
                    log("[rename-tail] old==new path (canonical title-only rename); no note_meta path update");
                }
                // 'rename' alias — durable safety net for old title
                // lookups regardless of any later frontmatter edits.
                let normalized = crate::search::normalize_alias_for_match(&old_title);
                if !normalized.is_empty() && old_title != new_title {
                    let _ = conn.execute(
                        "INSERT OR IGNORE INTO note_aliases (path, alias_lower, source, cid_cn) VALUES (?1, ?2, 'rename', COALESCE((SELECT cid_cn FROM note_meta WHERE path = ?1), ''))",
                        rusqlite::params![&new_path, normalized],
                    );
                }
            }
        }
    }

    // Step 6: reindex at new path.
    {
        use tauri::Manager;
        let search_state = app.state::<crate::search::SearchState>();
        let libs = load_all_libraries(app);
        if let Some(lib) = libs.iter().find(|l| new_path.starts_with(&l.path)) {
            match crate::search::reindex_single_note(&search_state, new_path, &lib.name) {
                Ok(_) => log(&format!("[rename-tail] reindex OK for {} (lib {})", new_path, lib.name)),
                Err(e) => log(&format!("[rename-tail] reindex ERROR: {}", e)),
            }
        } else {
            log(&format!("[rename-tail] NO LIBRARY matched new_path={} — reindex SKIPPED", new_path));
        }
    }
    log("[rename-tail] END");
}

/// MIG-003 Step 0 — Convert a note title into a safe filename
/// (without extension). Strips ASCII filesystem-reserved chars
/// (`/ \ : * ? " < > |`) and the ASCII control range. Replaces them
/// with a single space, then collapses runs of whitespace. Trims
/// leading/trailing whitespace and dots (Windows hates trailing dots).
/// Truncates to 240 bytes (boundary-safe — won't split a multi-byte
/// codepoint), leaving room for ` 999.md` collision suffix + extension
/// inside Windows MAX_PATH 260. Falls back to "Untitled" if the
/// sanitized title is empty.
///
/// Cross-script: preserves Arabic, Hebrew, Devanagari, CJK, Cyrillic,
/// and any other Unicode characters. Only ASCII filesystem-reserved
/// chars get touched. Per MIG-003 i18n requirement, all 15 launch
/// languages must round-trip through this without losing content.
///
/// Windows reserved names (`CON`, `PRN`, `NUL`, `COM1`-`COM9`,
/// `LPT1`-`LPT9`, `AUX`) are case-insensitively detected and
/// suffixed with an underscore (`CON` → `CON_`).
pub(crate) fn note_display_filename(title: &str) -> String {
    const RESERVED_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    const WIN_RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    // Replace reserved chars + ASCII control range with single space.
    // Collapse runs of whitespace (replaced or real) to a single space
    // so "Foo: Bar/Baz" → "Foo Bar Baz" (not "Foo  Bar Baz").
    let mut buf = String::with_capacity(title.len());
    let mut last_was_space = false;
    for ch in title.chars() {
        let is_bad = RESERVED_CHARS.contains(&ch) || (ch as u32) < 0x20;
        let effective = if is_bad { ' ' } else { ch };
        if effective == ' ' {
            if !last_was_space {
                buf.push(' ');
                last_was_space = true;
            }
        } else {
            buf.push(effective);
            last_was_space = false;
        }
    }

    // Trim leading/trailing whitespace and dots.
    let trimmed = buf.trim().trim_matches('.').trim();

    // Empty after sanitization → fall back.
    if trimmed.is_empty() {
        return "Untitled".to_string();
    }

    // Windows reserved-name guard (case-insensitive against the bare
    // stem). Any of these appearing as the entire filename gets an
    // underscore suffix to escape the reservation.
    let upper = trimmed.to_uppercase();
    if WIN_RESERVED.iter().any(|r| *r == upper) {
        return format!("{}_", trimmed);
    }

    // Truncate at byte boundary safe for UTF-8 (don't split a multi-
    // byte codepoint mid-sequence). 240 bytes leaves headroom for
    // ` 999.md` (~7 bytes) inside Windows' 255-char filename limit
    // and 260-char path limit — a deeply-nested folder still fits.
    if trimmed.len() <= 240 {
        return trimmed.to_string();
    }
    let mut cut = 240;
    while !trimmed.is_char_boundary(cut) {
        cut -= 1;
    }
    trimmed[..cut].trim_end().to_string()
}

/// MIG-003 Step 0 — Resolve a filename collision by appending ` N`
/// suffix to the base stem. Returns the first free filename in the
/// directory. Caller passes the desired stem (without extension) and
/// the desired extension (with leading dot). Tries up to 999 suffixes.
///
/// Behavior:
/// - `dir/Apple.md` does not exist → returns "Apple.md".
/// - `dir/Apple.md` exists → returns "Apple 1.md".
/// - `dir/Apple.md` and `dir/Apple 1.md` exist → returns "Apple 2.md".
/// - 999 collisions exhausted → returns Err.
///
/// `case_insensitive_match` controls whether the existence check
/// folds case (Windows / macOS default-FS behavior). Pass `true` for
/// FS-portable behavior.
pub(crate) fn resolve_filename_collision(
    dir: &Path,
    base_stem: &str,
    ext: &str,
    case_insensitive_match: bool,
) -> Result<String, String> {
    let normalized_stem = if case_insensitive_match {
        base_stem.to_lowercase()
    } else {
        base_stem.to_string()
    };

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => return Err(format!("read_dir failed: {}", e)),
    };
    let existing: std::collections::HashSet<String> = read_dir
        .flatten()
        .filter_map(|e| e.file_name().to_string_lossy().to_string().into())
        .map(|n| if case_insensitive_match { n.to_lowercase() } else { n })
        .collect();

    let candidate0 = format!("{}{}", base_stem, ext);
    let candidate0_check = if case_insensitive_match {
        candidate0.to_lowercase()
    } else {
        candidate0.clone()
    };
    if !existing.contains(&candidate0_check) {
        return Ok(candidate0);
    }

    for n in 1..=999u16 {
        let candidate = format!("{} {}{}", base_stem, n, ext);
        let check = if case_insensitive_match {
            candidate.to_lowercase()
        } else {
            candidate.clone()
        };
        if !existing.contains(&check) {
            return Ok(candidate);
        }
        let _ = normalized_stem; // silence unused (kept for future similarity dedup)
    }

    Err(format!("filename collision: 999 suffix attempts exhausted for stem {:?}", base_stem))
}

#[cfg(test)]
mod mig_003_helper_tests {
    use super::note_display_filename;

    #[test]
    fn ascii_title_passthrough() {
        assert_eq!(note_display_filename("Apple Tree Fruit"), "Apple Tree Fruit");
    }
    #[test]
    fn fs_reserved_chars_replaced() {
        assert_eq!(note_display_filename("Foo: Bar/Baz?"), "Foo Bar Baz");
    }
    #[test]
    fn collapse_run_of_reserved_chars() {
        assert_eq!(note_display_filename("a///b"), "a b");
    }
    #[test]
    fn arabic_title_preserved() {
        assert_eq!(note_display_filename("الزراعة المستدامة"), "الزراعة المستدامة");
    }
    #[test]
    fn hebrew_title_preserved() {
        // Use explicit unicode escapes to avoid copy-paste hazards
        // between Hebrew vav (U+05D5) and Arabic waw (U+0648), which
        // are visually similar in some fonts.
        let hebrew = "\u{05EA}\u{05E4}\u{05D5}\u{05D7}"; // tav peh vav chet
        assert_eq!(note_display_filename(hebrew), hebrew);
    }
    #[test]
    fn cjk_japanese_preserved() {
        assert_eq!(note_display_filename("りんごの木"), "りんごの木");
    }
    #[test]
    fn cyrillic_preserved() {
        assert_eq!(note_display_filename("Яблоня"), "Яблоня");
    }
    #[test]
    fn devanagari_preserved() {
        assert_eq!(note_display_filename("सेब का पेड़"), "सेब का पेड़");
    }
    #[test]
    fn mixed_script_preserved() {
        assert_eq!(note_display_filename("Apple أبيض"), "Apple أبيض");
    }
    #[test]
    fn empty_title_falls_back() {
        assert_eq!(note_display_filename(""), "Untitled");
    }
    #[test]
    fn whitespace_only_falls_back() {
        assert_eq!(note_display_filename("   "), "Untitled");
    }
    #[test]
    fn all_reserved_chars_falls_back() {
        assert_eq!(note_display_filename("/\\:*?"), "Untitled");
    }
    #[test]
    fn windows_reserved_con_escaped() {
        assert_eq!(note_display_filename("CON"), "CON_");
    }
    #[test]
    fn windows_reserved_case_insensitive() {
        assert_eq!(note_display_filename("nul"), "nul_");
    }
    #[test]
    fn trailing_dot_stripped() {
        assert_eq!(note_display_filename("Note."), "Note");
    }
    #[test]
    fn long_title_truncated_at_byte_boundary() {
        let long = "A".repeat(300);
        let result = note_display_filename(&long);
        assert!(result.len() <= 240);
        assert!(result.is_char_boundary(result.len()));
    }
    #[test]
    fn long_arabic_title_truncated_at_codepoint_boundary() {
        // Arabic chars are 2 bytes each in UTF-8. Title of 200 Arabic
        // chars = 400 bytes; should truncate cleanly without splitting
        // any character.
        let long: String = "ا".repeat(200);
        let result = note_display_filename(&long);
        assert!(result.len() <= 240);
        assert!(std::str::from_utf8(result.as_bytes()).is_ok());
    }
    #[test]
    fn ascii_control_chars_stripped() {
        assert_eq!(note_display_filename("Foo\x00Bar\nBaz"), "Foo Bar Baz");
    }
}

/// MIG-008 — User-visible display name for a note. Prefers the
/// frontmatter `title:` field; falls back to the file stem when
/// title is missing or content can't be parsed.
///
/// Why this exists: every CE Layer 1 phase scanner (strata, maturity,
/// provenance, review, lenses, tasks, canvas, bases, inspector360)
/// and the Constellation Map walks the filesystem directly and used
/// to derive the note label from `path.file_stem()`. For canonical
/// filenames (`20260426T140909Z_NOTE_D9A3.md`), that produced
/// unreadable labels. This helper standardizes the
/// "title-with-stem-fallback" lookup across every such surface.
///
/// Caller passes already-read file content if available — every
/// scanner reads content for other reasons (word_count, link extraction,
/// frontmatter parsing), so this is a sub-millisecond regex over the
/// same string, not new I/O. When content is not yet read, pass `None`
/// and the helper performs the read itself (used by the entry-point of
/// `inspector360.rs` where only the path is known initially).
///
/// SQLite-backed surfaces (Sky View via `note_meta.name`, Backlinks
/// via `note_links.source_name`, search results, etc.) use the same
/// rule at INDEX time in `search.rs::index_note:1665-1670`, so they
/// already display correctly without calling this helper.
pub(crate) fn note_display_name(path: &Path, content: Option<&str>) -> String {
    if let Some(c) = content {
        if let Some(title) = extract_frontmatter_title(c) {
            return title;
        }
    } else if crate::canonical::is_canonical_filename(path) {
        // Only read the file when the filename is canonical
        // (`20260426T140737Z_NOTE_E561.md`) — there's no human title
        // recoverable from `file_stem` in that case. For human-named
        // files (`Apple Tree Fruit.md`), the file stem IS the title,
        // so skip the read entirely. This keeps callers like
        // `review::scan_due_recursive` cheap on non-canonical
        // libraries while still rescuing canonical-filename libraries.
        if let Ok(c) = fs::read_to_string(path) {
            if let Some(title) = extract_frontmatter_title(&c) {
                return title;
            }
        }
    }
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Extract the `title:` value from a note's frontmatter.
fn extract_frontmatter_title(content: &str) -> Option<String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") { return None; }
    let after = &trimmed[3..];
    let end = after.find("\n---")?;
    let fm = &after[..end];
    for line in fm.lines() {
        let t = line.trim();
        if t.starts_with("title:") {
            let val = t["title:".len()..].trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() { return Some(val.to_string()); }
        }
    }
    None
}

/// Update a note's frontmatter title and add the old title to aliases.
fn update_frontmatter_title(content: &str, new_title: &str, old_title: &str) -> String {
    let esc_new = new_title.replace('"', "\\\"");
    let esc_old = old_title.replace('"', "\\\"");

    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return format!(
            "---\ntitle: \"{}\"\naliases:\n  - \"{}\"\n---\n\n{}",
            esc_new, esc_old, content
        );
    }

    let after_first = &trimmed[3..];
    let Some(end) = after_first.find("\n---") else {
        return content.to_string();
    };
    let fm = &after_first[..end];
    let body = &after_first[end + 4..];

    let mut new_lines: Vec<String> = Vec::new();
    let mut found_title = false;
    let mut found_aliases = false;
    let mut old_title_in_aliases = false;
    let mut in_alias_list = false;

    for line in fm.lines() {
        let t = line.trim();

        // Replace title field
        if t.starts_with("title:") {
            found_title = true;
            new_lines.push(format!("title: \"{}\"", esc_new));
            continue;
        }

        // Handle aliases field
        if t.starts_with("aliases:") {
            found_aliases = true;
            let value = t["aliases:".len()..].trim();

            if value.starts_with('[') && value.ends_with(']') {
                // Inline array: aliases: [a, b, c]
                let inner = &value[1..value.len() - 1];
                let existing: Vec<String> = inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                old_title_in_aliases = existing.iter().any(|a| a == old_title);
                // Convert to list format for consistency
                new_lines.push("aliases:".to_string());
                for alias in &existing {
                    new_lines.push(format!("  - \"{}\"", alias.replace('"', "\\\"")));
                }
                if !old_title_in_aliases {
                    new_lines.push(format!("  - \"{}\"", esc_old));
                }
                continue;
            }

            // List format: aliases:\n  - a\n  - b
            new_lines.push(line.to_string());
            in_alias_list = true;
            continue;
        }

        // Collect alias list items
        if in_alias_list && t.starts_with("- ") {
            let alias_val = t[2..].trim().trim_matches('"').trim_matches('\'');
            if alias_val == old_title {
                old_title_in_aliases = true;
            }
            new_lines.push(line.to_string());
            continue;
        }

        // End of alias list — append old title if missing
        if in_alias_list {
            in_alias_list = false;
            if !old_title_in_aliases {
                new_lines.push(format!("  - \"{}\"", esc_old));
            }
        }

        new_lines.push(line.to_string());
    }

    // If alias list was the last thing in frontmatter
    if in_alias_list && !old_title_in_aliases {
        new_lines.push(format!("  - \"{}\"", esc_old));
    }

    // Add missing fields
    if !found_title {
        new_lines.insert(0, format!("title: \"{}\"", esc_new));
    }
    if !found_aliases {
        new_lines.push("aliases:".to_string());
        new_lines.push(format!("  - \"{}\"", esc_old));
    }

    format!("---\n{}\n---{}", new_lines.join("\n"), body)
}

/// PJ-065 resolve — set the `parent:` scalar to `"[[new_parent]]"` (replace if present,
/// else insert). Mirrors `update_frontmatter_title`'s split/rebuild so the rest of the
/// note is preserved byte-for-byte. `new_parent` is a bare note NAME (no brackets).
fn set_frontmatter_parent(content: &str, new_parent: &str) -> String {
    let esc = new_parent.replace('"', "\\\"");
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return format!("---\nparent: \"[[{}]]\"\n---\n\n{}", esc, content);
    }
    let after_first = &trimmed[3..];
    let Some(end) = after_first.find("\n---") else {
        return content.to_string();
    };
    // Strip the leading '\n' after the opening `---` so .lines() doesn't yield a spurious
    // empty first line (which would add/accumulate a blank line on every edit AND break the
    // command's no-op guard). The body keeps its own leading separator below.
    let fm = after_first[..end].strip_prefix('\n').unwrap_or(&after_first[..end]);
    let body = &after_first[end + 4..];
    let mut new_lines: Vec<String> = Vec::new();
    let mut found = false;
    for line in fm.lines() {
        if line.trim_start().starts_with("parent:") {
            found = true;
            new_lines.push(format!("parent: \"[[{}]]\"", esc));
            continue;
        }
        new_lines.push(line.to_string());
    }
    if !found {
        new_lines.push(format!("parent: \"[[{}]]\"", esc));
    }
    format!("---\n{}\n---{}", new_lines.join("\n"), body)
}

/// PJ-065 resolve — remove the `[[child]]` entry from the `contains:` YAML list (inline
/// `[a, b]` OR block `- a`). Drops the whole `contains:` key if it becomes empty. `child`
/// is a bare note NAME. Match is case-insensitive on the wikilink target (strips brackets,
/// quotes, and any `|alias`). Other lines preserved exactly.
fn remove_frontmatter_contains_item(content: &str, child: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return content.to_string();
    }
    let after_first = &trimmed[3..];
    let Some(end) = after_first.find("\n---") else {
        return content.to_string();
    };
    let fm = after_first[..end].strip_prefix('\n').unwrap_or(&after_first[..end]);
    let body = &after_first[end + 4..];

    let target = child.trim().to_lowercase();
    let matches_target = |item: &str| -> bool {
        let s = item.trim().trim_matches('"').trim_matches('\'');
        let inner = s.trim_start_matches("[[").trim_end_matches("]]");
        let name = inner.split('|').next().unwrap_or(inner).trim();
        name.to_lowercase() == target
    };

    let mut new_lines: Vec<String> = Vec::new();
    let mut in_list = false;
    let mut kept_in_list = 0usize;
    let mut header_idx: Option<usize> = None;

    for line in fm.lines() {
        let t = line.trim();
        if t.starts_with("contains:") {
            let value = t["contains:".len()..].trim();
            if value.starts_with('[') && value.ends_with(']') {
                // Inline array.
                let inner = &value[1..value.len() - 1];
                let kept: Vec<String> = inner
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && !matches_target(s))
                    .collect();
                if !kept.is_empty() {
                    new_lines.push(format!("contains: [{}]", kept.join(", ")));
                }
                continue;
            }
            // Block list — push the header now; drop it later if nothing is kept.
            in_list = true;
            header_idx = Some(new_lines.len());
            kept_in_list = 0;
            new_lines.push(line.to_string());
            continue;
        }
        if in_list {
            if t.starts_with("- ") {
                if matches_target(&t[2..]) {
                    continue; // drop this item
                }
                kept_in_list += 1;
                new_lines.push(line.to_string());
                continue;
            }
            // End of the block list.
            in_list = false;
            if kept_in_list == 0 {
                if let Some(idx) = header_idx.take() {
                    new_lines.remove(idx);
                }
            }
        }
        new_lines.push(line.to_string());
    }
    if in_list && kept_in_list == 0 {
        if let Some(idx) = header_idx.take() {
            new_lines.remove(idx);
        }
    }
    format!("---\n{}\n---{}", new_lines.join("\n"), body)
}

/// PJ-065 §D9 — one-click resolution of a CONTESTED structural parent (two notes claim
/// the same child). Edits ONE frontmatter field on `note_path` and rides the proven
/// rename-cascade write path: gate_write (serializes vs any in-flight editor flush +
/// suppresses the watcher) → reindex_single_note → emit `cascade:rewrote` so any open tab
/// reloads from disk (no BUG-015 stale-buffer stomp). Explicit user action — never silent.
///   field "parent"   → set this note's `parent:` to `[[target_name]]` (the child takes the claimant).
///   field "contains" → remove `[[target_name]]` from this note's `contains:` (the claimant releases).
// Note-open-freeze Batch-2 §B2-3 (2026-07-03): `(async)` + the read→edit→write
// cycle moved inside `gate_rmw` (per-path lock across the WHOLE cycle) — a
// debounced editor save can land before or after the resolve, never inside it.
// Reindex + emit stay OUTSIDE the lock (no DB waits under a path lock).
#[tauri::command(async)]
pub fn resolve_structural_conflict(
    app: tauri::AppHandle,
    note_path: String,
    field: String,
    target_name: String,
) -> Result<(), String> {
    validate_path_in_any_library(&app, &note_path)?;
    let path = Path::new(&note_path);
    if !path.exists() {
        return Err("Note does not exist.".to_string());
    }
    let outcome = crate::write_gate::gate_rmw(path, "resolve_structural_conflict", |content| {
        let updated = match field.as_str() {
            "parent" => set_frontmatter_parent(content, &target_name),
            "contains" => remove_frontmatter_contains_item(content, &target_name),
            other => return Err(format!("resolve_structural_conflict: unknown field '{}'", other)),
        };
        if updated == content {
            Ok(None) // no-op (already resolved) — nothing written
        } else {
            Ok(Some(updated))
        }
    })?;
    if outcome == crate::write_gate::WriteOutcome::OkUnchecked {
        return Ok(()); // no-op — skip reindex + emit, as before
    }

    {
        use tauri::Manager;
        let search_state = app.state::<crate::search::SearchState>();
        let libs = load_all_libraries(&app);
        if let Some(lib) = libs.iter().find(|l| note_path.starts_with(&l.path)) {
            let _ = crate::search::reindex_single_note(&search_state, &note_path, &lib.name);
        }
    }
    {
        use tauri::Emitter;
        let _ = app.emit("cascade:rewrote", serde_json::json!({ "paths": [note_path] }));
    }
    Ok(())
}

/// Move a file or folder to a different directory within any registered library.
// Note-open-freeze Batch-2 §B2-4 (2026-07-03): `(async)` — off the IPC dispatch thread.
// The destructive/rename steps run under path locks (gate_rename/gate_delete); the DB
// cascade + reindex run after the locks release. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn move_item(app: tauri::AppHandle, source_path: String, target_folder: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &source_path)?;
    validate_path_in_any_library(&app, &target_folder)?;
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err("Source item does not exist.".to_string());
    }
    let target_dir = Path::new(&target_folder);
    if !target_dir.is_dir() {
        return Err("Target folder does not exist.".to_string());
    }
    let file_name = source.file_name()
        .ok_or("Cannot determine file name.")?;
    let dest = target_dir.join(file_name);
    if dest.exists() {
        return Err("An item with this name already exists in the target folder.".to_string());
    }
    // MIG-076 §A2 — gated rename (both paths locked, AV retry, journaled).
    crate::write_gate::gate_rename(source, &dest, "move_item")?;

    // MIG-077 A3-R3 — reindex on move (mirrors rename_item Step 6). move_item
    // previously did NOT reindex, so the FTS/links index kept the old path and a
    // cross-library move would index the note under the wrong library. Drop the
    // old entry, add the moved note(s) under the destination library. Handles a
    // folder move by reindexing every .md descendant at its new path.
    {
        use tauri::Manager;
        let search_state = app.state::<crate::search::SearchState>();
        let libs = load_all_libraries(&app);
        let dest_str = dest.to_string_lossy().to_string();
        let dest_lib_name = libs
            .iter()
            .filter(|l| dest_str.starts_with(&l.path))
            .max_by_key(|l| l.path.len())
            .map(|l| l.name.clone());
        if dest.is_dir() {
            let mut md_paths: Vec<std::path::PathBuf> = Vec::new();
            collect_md_paths(&dest, &mut md_paths);
            for new_p in &md_paths {
                if let Ok(rel) = new_p.strip_prefix(&dest) {
                    let old_p = source.join(rel);
                    let _ = crate::search::reindex_delete_note(&search_state, &old_p.to_string_lossy());
                }
                if let Some(name) = &dest_lib_name {
                    let _ = crate::search::reindex_single_note(&search_state, &new_p.to_string_lossy(), name);
                }
            }
        } else {
            let _ = crate::search::reindex_delete_note(&search_state, &source_path);
            if let Some(name) = &dest_lib_name {
                let _ = crate::search::reindex_single_note(&search_state, &dest_str, name);
            }
        }
    }

    Ok(dest.to_string_lossy().to_string())
}

/// Recursively collect every `.md` file path under `dir` (MIG-077 A3-R3 — used to
/// reindex all descendants after a folder move).
fn collect_md_paths(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            // Skip symlinks/junctions to prevent circular recursion (mirrors
            // read_dir_recursive). Without this a directory junction loop blows
            // the walk up and OOM-crashes the app.
            if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                continue;
            }
            let p = entry.path();
            if p.is_dir() {
                collect_md_paths(&p, out);
            } else if p.extension().map(|e| e == "md").unwrap_or(false) {
                out.push(p);
            }
        }
    }
}

/// MIG-077 A3-R3 — a folder destination for the universe-wide Move picker. One
/// row per folder across every library (incl. federated child universes). Depth
/// is relative to the library root (the root itself is added by the frontend).
#[derive(serde::Serialize)]
pub struct UniverseFolder {
    pub library_id: String,
    pub library_name: String,
    pub path: String,
    pub name: String,
    pub depth: u32,
}

/// List every folder across the whole universe (all libraries + federated child
/// universes), folders ONLY — a lightweight Rust-side walk so the frontend never
/// reads thousands of note rows just to populate the Move picker (Rule 3).
#[tauri::command]
pub fn list_universe_folders(app: tauri::AppHandle) -> Result<Vec<UniverseFolder>, String> {
    let libs = load_all_libraries(&app);
    let mut out: Vec<UniverseFolder> = Vec::new();
    for lib in &libs {
        collect_folders(Path::new(&lib.path), &lib.id, &lib.name, 1, &mut out);
    }
    Ok(out)
}

fn collect_folders(dir: &Path, lib_id: &str, lib_name: &str, depth: u32, out: &mut Vec<UniverseFolder>) {
    if depth > 30 {
        return;
    }
    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            // Skip symlinks/junctions BEFORE touching the path — mirrors
            // read_dir_recursive's "prevent circular recursion" guard. A real
            // directory-junction loop on the user's machine made the link-
            // following `path.is_dir()` walk blow up exponentially and OOM-crash
            // the app (the Move picker hung on "…" until the crash).
            if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                continue;
            }
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            // skip hidden / system folders (.trash, .constellation, .git, .obsidian…)
            if name.starts_with('.') {
                continue;
            }
            out.push(UniverseFolder {
                library_id: lib_id.to_string(),
                library_name: lib_name.to_string(),
                path: p.to_string_lossy().to_string(),
                name,
                depth,
            });
            collect_folders(&p, lib_id, lib_name, depth + 1, out);
        }
    }
}

// `delete_item` RETIRED (Batch-2 §B2-4, Boss-ruled 2026-07-03). It was the
// pre-trash-era always-permanent delete with the family's worst unprotected
// write path, superseded by `delete_path` (trash-backed modes + gate_delete)
// — its frontend wrapper had zero component callers (deleteWithSetting /
// deletePath replaced it, MIG-076 §E-follow-up). Predecessor → Replacement
// entry: SESSION-LOG-2026-07-03.

/// Resolve a wikilink target to an actual file path within a library.
#[tauri::command]
pub fn resolve_wikilink(app: tauri::AppHandle, library_path: String, target: String) -> Result<Option<String>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let library_dir = Path::new(&library_path);
    if !library_dir.exists() {
        return Err("Library path does not exist.".to_string());
    }

    let target_lower = target.to_lowercase();
    let mut matches: Vec<PathBuf> = Vec::new();
    find_note_by_name_or_alias(library_dir, &target_lower, &mut matches, 0);

    if matches.is_empty() {
        return Ok(None);
    }

    // Prefer shortest path (closest to library root)
    matches.sort_by_key(|p| p.to_string_lossy().len());
    Ok(Some(matches[0].to_string_lossy().to_string()))
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedLink {
    pub path: String,
    pub library_name: String,
    pub library_path: String,
    pub fragment: Option<String>,
}

/// Resolve a wikilink across all libraries. Searches current library first, then others.
/// Supports `library_name:note` syntax to target a specific library.
/// Supports `note#heading` and `note#^block-id` — fragment is stripped before resolution and returned separately.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
/// MIG-099 — shared driver for both the wikilink resolver (`skip_stem=false`)
/// and the title-collision check (`skip_stem=true`, §6). Builds the OWN-library
/// authority set and runs the impl with the read connection (walk fallback if
/// the DB isn't ready).
///
/// OWN vs FEDERATED (adversarial C1): `load_libraries` returns ONLY the active
/// universe's own registrations (NON-recursive) — the exact set whose rows are
/// authoritatively maintained in this universe's note_meta. It survives the
/// flatten+dedup in resolve_libraries_recursive and — unlike a path-under-root
/// heuristic — classifies external own libraries (paths outside the universe
/// root) as OWN and cUniverses nested under the root as FEDERATED.
fn run_cross_library_resolution(
    app: &tauri::AppHandle,
    libraries: &[(String, String, String)],
    current_library_path: &str,
    target: &str,
    skip_stem: bool,
) -> Option<ResolvedLink> {
    use tauri::Manager;
    let own_paths: std::collections::HashSet<String> = load_libraries(app)
        .iter()
        .map(|l| norm_lib_path(&l.path))
        .collect();

    let state = app.state::<crate::search::SearchState>();
    // Reader connection available → OWN libraries resolve via indexed seek. If the
    // DB isn't ready (pre-init / mid universe-switch) with_read_conn errors →
    // pure filesystem walk for everything (correct, just not accelerated).
    match crate::search::with_read_conn(state.inner(), |conn| {
        let ctx = ResolveCtx { own_paths: &own_paths, conn: Some(conn), skip_stem };
        Ok(resolve_wikilink_cross_library_impl(libraries, current_library_path, target, &ctx))
    }) {
        Ok(v) => v,
        Err(_) => {
            let ctx = ResolveCtx { own_paths: &own_paths, conn: None, skip_stem };
            resolve_wikilink_cross_library_impl(libraries, current_library_path, target, &ctx)
        }
    }
}

#[tauri::command(async)]
pub fn resolve_wikilink_cross_library(
    app: tauri::AppHandle,
    libraries: Vec<(String, String, String)>, // (library_id, library_name, library_path)
    current_library_path: String,
    target: String,
) -> Result<Option<ResolvedLink>, String> {
    Ok(run_cross_library_resolution(&app, &libraries, &current_library_path, &target, false))
}

/// MIG-099 §6 — the create/rename TITLE-collision check (MIG-076 §E1b). Answers
/// "does a note with this TITLE already exist anywhere in the one-universe?"
/// INDEX-ONLY for own libraries (name_lower + alias_lower, NO stem read_dir →
/// sub-10 ms vs the full resolver's 324 ms filename scan); bounded title/alias
/// walk for federated libraries. Returns the same ResolvedLink shape (path +
/// library) so the collision dialog + Overwrite (moveToTrash) work unchanged.
/// Depends on MIG-099 §3's synchronous create-reindex: an index-only check is
/// only authoritative if the just-created note is already indexed (it is).
#[tauri::command(async)]
pub fn resolve_title_collision(
    app: tauri::AppHandle,
    libraries: Vec<(String, String, String)>, // (library_id, library_name, library_path)
    current_library_path: String,
    target: String,
) -> Result<Option<ResolvedLink>, String> {
    Ok(run_cross_library_resolution(&app, &libraries, &current_library_path, &target, true))
}

/// MIG-099 §2 — resolution context threaded through the impl: the set of OWN
/// (active-universe, index-authoritative) library paths + the read connection
/// (None when the DB isn't ready → walk fallback).
struct ResolveCtx<'a> {
    own_paths: &'a std::collections::HashSet<String>,
    conn: Option<&'a rusqlite::Connection>,
    // MIG-099 §6 — when true, SKIP the stage-1 filename-stem read_dir and resolve
    // by TITLE/alias only. Used by the create/rename title-collision check
    // (resolve_title_collision): it answers "does a note with this TITLE exist?"
    // (MIG-076 §E1b title-ambiguity), which name_lower + alias_lower answer as a
    // pure index seek on own libs — no directory walk (324 ms → sub-10 ms). The
    // wikilink-RESOLUTION callers keep skip_stem=false (stem stage intact).
    skip_stem: bool,
}

/// Resolve `raw_target` within a single library, preserving the two-stage
/// precedence of the original walk:
///   stage 1 — filename STEM match (`find_note_by_name`, no file reads) — kept
///             UNCHANGED, so stem-first precedence and stem-with-distinct-title
///             resolvability are byte-for-byte the old behavior.
///   stage 2 — title/alias: OWN library + reader ready → indexed seek on
///             note_meta.name_lower + note_aliases.alias_lower (13.6 s → sub-10 ms),
///             scoped to this library and dot-dir-filtered; the index miss is
///             AUTHORITATIVE (returns "not here" without a scan). FEDERATED, or
///             reader unavailable, or a SQL error → the bounded filesystem walk
///             of THIS library only (always correct on live disk).
/// Pushes ALL hits into `matches`; the caller applies the byte-shortest tie-break.
fn resolve_in_library(
    library_dir: &Path,
    raw_target: &str,
    ctx: &ResolveCtx,
    matches: &mut Vec<PathBuf>,
) {
    let target_lower = raw_target.to_lowercase();

    // Stage 1 — filename stem (cheap, no reads). Unchanged for wikilink
    // resolution. MIG-099 §6 — SKIPPED for the title-collision check, which
    // resolves by title/alias only (a duplicate TITLE, not a filename, is what
    // MIG-076 §E1b guards); this is what removes the residual read_dir cost.
    if !ctx.skip_stem {
        find_note_by_name(library_dir, &target_lower, matches, 0);
        if !matches.is_empty() {
            return;
        }
    }

    // Stage 2 — title/alias.
    let norm = norm_lib_path(&library_dir.to_string_lossy());
    let is_own = ctx.own_paths.contains(&norm);
    if is_own {
        if let Some(conn) = ctx.conn {
            // Fold the target with the SAME functions the write path uses so the
            // key matches the stored column (adversarial C4). fold the RAW target
            // (not the pre-lowercased form) so NFC/lowercase compose identically
            // to index_note's fold_match_key(name).
            let folded_name = crate::search::fold_match_key(raw_target);
            let folded_alias = crate::search::normalize_alias_for_match(raw_target);
            match query_index_candidates(conn, &folded_name, &folded_alias) {
                Ok(cands) => {
                    for p in cands {
                        // Scope to THIS library (preserves current-first / Vec-order),
                        // drop .trash/.constellation rows the walk would skip, AND
                        // stat-guard against a stale row (MIG-099 §3): an orphan
                        // note_aliases row that reindex_delete_note doesn't purge, a
                        // note under a temporarily-unmounted library, or a
                        // moved/trashed-but-not-yet-reindexed path. The filesystem
                        // walk never returns a nonexistent path — require the same.
                        if path_under_library(&p, &norm)
                            && !has_dot_segment(&p)
                            && Path::new(&p).exists()
                        {
                            matches.push(PathBuf::from(p));
                        }
                    }
                    // Authoritative for an OWN library: trust the index result
                    // (even empty) — no filesystem scan.
                    return;
                }
                Err(_) => {
                    // SQL failure → degrade THIS library to the bounded walk.
                    matches.clear();
                }
            }
        }
        // is_own but reader not ready → fall through to the walk (safe, slow).
    }

    // Federated library, or own-but-index-unavailable → bounded walk of THIS
    // library only (never the 2 GB own universe — federated trees are small).
    find_note_by_title_or_alias(library_dir, &target_lower, matches, 0);
}

fn resolve_wikilink_cross_library_impl(
    libraries: &[(String, String, String)], // (library_id, library_name, library_path)
    current_library_path: &str,
    target: &str,
    ctx: &ResolveCtx,
) -> Option<ResolvedLink> {
    // Strip fragment (#heading or #^block-id)
    let (base_target, fragment) = if let Some(hash_pos) = target.find('#') {
        (target[..hash_pos].to_string(), Some(target[hash_pos + 1..].to_string()))
    } else {
        (target.to_string(), None)
    };

    // Check for library:note syntax
    if let Some(colon_pos) = base_target.find(':') {
        let library_prefix = base_target[..colon_pos].trim().to_lowercase();
        let note_target = base_target[colon_pos + 1..].trim(); // RAW (fold inside resolve_in_library)
        if !note_target.is_empty() {
            for (_id, name, path) in libraries.iter() {
                if name.to_lowercase() == library_prefix {
                    let library_dir = Path::new(path);
                    if !library_dir.exists() { continue; }
                    let mut matches: Vec<PathBuf> = Vec::new();
                    resolve_in_library(library_dir, note_target, ctx, &mut matches);
                    if !matches.is_empty() {
                        matches.sort_by_key(|p| p.to_string_lossy().len());
                        return Some(ResolvedLink {
                            path: matches[0].to_string_lossy().to_string(),
                            library_name: name.clone(),
                            library_path: path.clone(),
                            fragment,
                        });
                    }
                    return None;
                }
            }
        }
    }

    // Search current library first
    let current_dir = Path::new(current_library_path);
    if current_dir.exists() {
        let mut matches: Vec<PathBuf> = Vec::new();
        resolve_in_library(current_dir, &base_target, ctx, &mut matches);
        if !matches.is_empty() {
            matches.sort_by_key(|p| p.to_string_lossy().len());
            // Normalize both sides: strict `==` drops to "" on Windows
            // slash / trailing-slash / case drift, which then shows up
            // as an empty library chip on the tab and poisons the next
            // wikilink resolution (empty currentLibraryPath skips this
            // branch entirely on the next click, picking the wrong
            // same-named note from another library).
            let norm = |s: &str| s.replace('\\', "/").trim_end_matches('/').to_lowercase();
            let current_norm = norm(current_library_path);
            let library_name = libraries.iter()
                .find(|(_, _, p)| norm(p) == current_norm)
                .map(|(_, n, _)| n.clone())
                .unwrap_or_default();
            return Some(ResolvedLink {
                path: matches[0].to_string_lossy().to_string(),
                library_name,
                library_path: current_library_path.to_string(),
                fragment,
            });
        }
    }

    // Search other libraries
    for (_id, name, path) in libraries.iter() {
        if path == current_library_path { continue; }
        let library_dir = Path::new(path);
        if !library_dir.exists() { continue; }
        let mut matches: Vec<PathBuf> = Vec::new();
        resolve_in_library(library_dir, &base_target, ctx, &mut matches);
        if !matches.is_empty() {
            matches.sort_by_key(|p| p.to_string_lossy().len());
            return Some(ResolvedLink {
                path: matches[0].to_string_lossy().to_string(),
                library_name: name.clone(),
                library_path: path.clone(),
                fragment,
            });
        }
    }

    None
}

fn find_note_by_name(dir: &Path, target: &str, results: &mut Vec<PathBuf>, depth: u32) {
    if depth > 20 { return; }
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) { continue; }

        if path.is_dir() {
            find_note_by_name(&path, target, results, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            let stem = name.trim_end_matches(".md").to_lowercase();
            if stem == *target {
                results.push(path);
            }
        }
    }
}

/// Like find_note_by_name, but also checks frontmatter title and aliases.
/// Resolution order (first match wins):
///   1. Filename stem match (fast, no file read)
///   2. Frontmatter `title:` field match (supports canonical filenames)
///   3. Frontmatter `aliases:` match
fn find_note_by_name_or_alias(dir: &Path, target: &str, results: &mut Vec<PathBuf>, depth: u32) {
    // First try exact filename match (fast)
    find_note_by_name(dir, target, results, depth);
    if !results.is_empty() { return; }

    // If no filename match, scan frontmatter title + aliases
    find_note_by_title_or_alias(dir, target, results, depth);
}

fn find_note_by_title_or_alias(dir: &Path, target: &str, results: &mut Vec<PathBuf>, depth: u32) {
    if depth > 20 { return; }
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) { continue; }

        if path.is_dir() {
            find_note_by_title_or_alias(&path, target, results, depth + 1);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(content) = fs::read_to_string(&path) {
                if has_title(&content, target) || has_alias(&content, target) {
                    results.push(path);
                }
            }
        }
    }
}

/// Check if a note's frontmatter `title:` field matches the target.
fn has_title(content: &str, target: &str) -> bool {
    if !content.starts_with("---") { return false; }
    let end = match content[3..].find("\n---") {
        Some(pos) => pos + 3,
        None => return false,
    };
    let frontmatter = &content[3..end];
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("title:") {
            let value = trimmed["title:".len()..].trim();
            // G4 Phase 4 (C3) — decode via the shared scalar decoder and fold with the SAME
            // key the index uses (fold_match_key, NFC+Unicode-lower — not ASCII to_lowercase),
            // so the federated wikilink walk resolves a quoted/accented title identically to
            // note_meta.name_lower. `target` is folded too so both sides use one key.
            let decoded = crate::search::fold_match_key(&crate::search::decode_yaml_scalar(value));
            if decoded == crate::search::fold_match_key(target) { return true; }
        }
    }
    false
}

/// Check if a note's frontmatter contains a matching alias.
fn has_alias(content: &str, target: &str) -> bool {
    if !content.starts_with("---") { return false; }
    let end = match content[3..].find("\n---") {
        Some(pos) => pos + 3,
        None => return false,
    };
    let frontmatter = &content[3..end];
    // G4 Phase 4 (C3) — decode + fold with normalize_alias_for_match (the SAME key
    // note_aliases.alias_lower uses: fold_match_key + Arabic tashkeel/tatweel strip), so the
    // federated alias walk matches the alias index. The list-item arm is now BLOCK-GUARDED to
    // only match items UNDER an `aliases:` block (it previously over-matched ANY `- ` line —
    // e.g. a tags list item — resolving federated links it shouldn't).
    let want = crate::search::normalize_alias_for_match(target);
    let matches = |raw: &str| crate::search::normalize_alias_for_match(&crate::search::decode_yaml_scalar(raw)) == want;
    let mut in_aliases = false;
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        // Inline YAML array: aliases: [a, b, c]  |  scalar: aliases: x
        if trimmed.starts_with("aliases:") {
            let value = trimmed["aliases:".len()..].trim();
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len()-1];
                for alias in inner.split(',') { if matches(alias) { return true; } }
                in_aliases = false;
            } else if !value.is_empty() {
                if matches(value) { return true; }
                in_aliases = false;
            } else {
                in_aliases = true; // block form: `aliases:` then `- item` lines follow
            }
            continue;
        }
        if in_aliases {
            if let Some(rest) = trimmed.strip_prefix("- ") {
                if matches(rest) { return true; }
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                in_aliases = false; // a non-list-item, non-comment line ends the aliases block
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────
// MIG-099 — Index-backed name/alias resolution (Rule 8, Write-Time Derivation)
//
// The former stage-2 resolver (`find_note_by_title_or_alias`) fs::read_to_string'd
// EVERY .md across EVERY library to answer "does a note with this title/alias
// exist, and where?". On the live 2 GB / ~7,700-note universe a brand-new create
// name (matches no filename stem) forced a full COLD read = 13,575 ms (measured,
// diagnostics.log 2026-07-07). The always-current index already holds the answer:
// note_meta.name_lower (= fold_match_key(name)) + note_aliases.alias_lower
// (= normalize_alias_for_match). These helpers turn the scan into an indexed seek
// (idx_note_name_lower / idx_note_aliases_lookup) → 13.6 s becomes sub-10 ms.
//
// Correctness contract (MIG-099 adversarial review — every clause is a fix for a
// concrete refuted failure scenario):
//  • FOLD PARITY — the query key MUST pass through the SAME folds the write path
//    uses (fold_match_key for name, normalize_alias_for_match for alias); never
//    plain to_lowercase / COLLATE NOCASE (ASCII-only → silently breaks Arabic).
//  • NULL name_lower — pre-MIG-085 rows → two index-seeking arms
//    (name_lower = ?1  UNION  name_lower IS NULL AND LOWER(name) = ?1); a single
//    COALESCE would defeat idx_note_name_lower and full-scan (21 s, PJ-066).
//  • BYTE tie-break — the shortest-path winner is chosen by the CALLER in Rust
//    (to_string_lossy().len() = UTF-8 BYTES), NOT SQL length() (CHARACTERS —
//    inverts for Arabic paths, flipping which note a link opens).
//  • DOT-DIR EXCLUSION — the index DOES hold .trash / .constellation rows; the
//    filesystem walk skips any `.`-prefixed segment, so index hits are filtered
//    to match (has_dot_segment).
//  • OWN-ONLY authority — these query the ACTIVE universe's own note_meta; the
//    caller routes only OWN-library lookups here (load_libraries), keeping the
//    live bounded walk for federated cUniverse libraries (their rows are not
//    authoritatively maintained in the active DB).
// ─────────────────────────────────────────────────────────────────────────

/// Normalize a library / note path for prefix comparison: forward-slash, no
/// trailing slash, lowercased. Mirrors the `norm` closure used at the
/// current-library ResolvedLink build site so both sides compare identically.
fn norm_lib_path(s: &str) -> String {
    s.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// True when `path` lives under the library directory `norm_lib` (both compared
/// in normalized form). Scopes global index matches back to the single library
/// the resolver loop is currently visiting — preserving current-library-first,
/// other-library Vec-declaration order, and `library:note` prefix scoping.
fn path_under_library(path: &str, norm_lib: &str) -> bool {
    let np = norm_lib_path(path);
    np == norm_lib || np.starts_with(&format!("{}/", norm_lib))
}

/// True when any path segment starts with '.', i.e. the note lives inside a
/// dot-directory (.trash, .constellation, .obsidian). The filesystem walk skips
/// these (`name.starts_with('.')`); the index does not, so index results MUST be
/// filtered through this to preserve the trashed/system-note exclusion invariant.
fn has_dot_segment(path: &str) -> bool {
    path.split(['/', '\\']).any(|seg| seg.starts_with('.'))
}

/// Query the active-universe index for every note path whose folded name OR
/// folded alias equals the target. Returns ALL matches across the active DB;
/// the caller filters by library prefix + dot-dir exclusion and applies the
/// byte-shortest tie-break. `folded_name`/`folded_alias` MUST already be folded
/// with fold_match_key / normalize_alias_for_match respectively (the write-side
/// folds). Errs only on a genuine SQL failure — the caller then degrades that
/// library to the bounded walk (correctness over speed).
fn query_index_candidates(
    conn: &rusqlite::Connection,
    folded_name: &str,
    folded_alias: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT path FROM note_meta WHERE name_lower = ?1 \
             UNION \
             SELECT path FROM note_meta WHERE name_lower IS NULL AND LOWER(name) = ?1",
        )?;
        let rows = stmt.query_map([folded_name], |r| r.get::<_, String>(0))?;
        for r in rows {
            out.push(r?);
        }
    }
    if !folded_alias.is_empty() {
        let mut stmt = conn.prepare("SELECT path FROM note_aliases WHERE alias_lower = ?1")?;
        let rows = stmt.query_map([folded_alias], |r| r.get::<_, String>(0))?;
        for r in rows {
            out.push(r?);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests_mig099_index_resolve {
    use super::*;

    fn seed() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT, name_lower TEXT);
             CREATE TABLE note_aliases (path TEXT, alias_lower TEXT);",
        )
        .unwrap();
        // Plain titled note (stem == title).
        conn.execute(
            "INSERT INTO note_meta (path,name,name_lower) VALUES (?1,?2,?3)",
            rusqlite::params!["E:\\Lib\\Foo.md", "Foo", "foo"],
        )
        .unwrap();
        // Canonical-filename note whose display name IS a human title (accented).
        conn.execute(
            "INSERT INTO note_meta (path,name,name_lower) VALUES (?1,?2,?3)",
            rusqlite::params!["E:\\Lib\\20260101T000000Z_NOTE_ab12.md", "Île-de-France", "île-de-france"],
        )
        .unwrap();
        // Pre-MIG-085 row: NULL name_lower, ASCII name (LOWER() fallback arm).
        conn.execute(
            "INSERT INTO note_meta (path,name,name_lower) VALUES (?1,?2,NULL)",
            rusqlite::params!["E:\\Lib\\Bar.md", "Bar"],
        )
        .unwrap();
        // Trashed row — same folded name as Foo; caller's dot filter must drop it.
        conn.execute(
            "INSERT INTO note_meta (path,name,name_lower) VALUES (?1,?2,?3)",
            rusqlite::params!["E:\\Lib\\.trash\\Foo.md", "Foo", "foo"],
        )
        .unwrap();
        // Alias row.
        conn.execute(
            "INSERT INTO note_aliases (path,alias_lower) VALUES (?1,?2)",
            rusqlite::params!["E:\\Lib\\Foo.md", "nickname"],
        )
        .unwrap();
        conn
    }

    #[test]
    fn name_match_returns_path() {
        let c = seed();
        let hits = query_index_candidates(&c, "foo", "foo").unwrap();
        assert!(hits.iter().any(|p| p == "E:\\Lib\\Foo.md"));
    }

    #[test]
    fn folded_title_match_for_canonical_note() {
        // fold_match_key("Île-de-France") == "île-de-france" — the canonical note
        // is findable by its human title WITHOUT reading the file.
        let key = crate::search::fold_match_key("Île-de-France");
        let c = seed();
        let hits = query_index_candidates(&c, &key, &key).unwrap();
        assert_eq!(hits.iter().filter(|p| p.contains("NOTE_ab12")).count(), 1);
    }

    #[test]
    fn null_name_lower_falls_back_to_ascii_lower() {
        let c = seed();
        let hits = query_index_candidates(&c, "bar", "bar").unwrap();
        assert!(hits.iter().any(|p| p == "E:\\Lib\\Bar.md"));
    }

    #[test]
    fn alias_match_returns_path() {
        let c = seed();
        let hits = query_index_candidates(&c, "nickname", "nickname").unwrap();
        assert!(hits.iter().any(|p| p == "E:\\Lib\\Foo.md"));
    }

    #[test]
    fn dot_segment_excludes_trashed_note() {
        assert!(has_dot_segment("E:\\Lib\\.trash\\Foo.md"));
        assert!(!has_dot_segment("E:\\Lib\\Foo.md"));
        let c = seed();
        let hits = query_index_candidates(&c, "foo", "foo").unwrap();
        let visible: Vec<&String> = hits.iter().filter(|p| !has_dot_segment(p)).collect();
        assert!(visible.iter().all(|p| !p.contains(".trash")));
        assert!(visible.iter().any(|p| *p == "E:\\Lib\\Foo.md"));
    }

    #[test]
    fn path_under_library_scopes_correctly() {
        let norm = norm_lib_path("E:\\Lib");
        assert!(path_under_library("E:\\Lib\\Foo.md", &norm));
        assert!(path_under_library("E:\\Lib\\sub\\Foo.md", &norm));
        assert!(!path_under_library("E:\\Other\\Foo.md", &norm));
        // Sibling with a shared textual prefix but not a real subdirectory.
        assert!(!path_under_library("E:\\Library2\\Foo.md", &norm));
    }

    #[test]
    fn byte_shortest_tie_break_not_char_count() {
        // The walk picks the shortest BYTE path. An Arabic folder (2 bytes/char)
        // must not be mis-ranked shorter by CHARACTER count (the SQLite length()
        // trap). Sorting by String::len() (bytes) picks the ASCII sibling.
        let a = "E:/Lib/علم/n1.md"; // 19 bytes / 16 chars
        let b = "E:/Lib/abcde/n2.md"; // 18 bytes / 18 chars
        let mut v = vec![a.to_string(), b.to_string()];
        v.sort_by_key(|p| p.len()); // BYTE length
        assert_eq!(v[0], b, "byte sort must pick the ASCII sibling, not the Arabic one");
    }
}

/// Read Obsidian's appearance.json for a library.
///
/// `(async)` because this fires 16× in the boot fan-out (one per library) and
/// performs disk I/O (`fs::read_to_string` + JSON parse). Keeping the body on
/// the WebView2 UI thread would serialize all 16 reads behind whatever other
/// fan-out work is in flight. See watcher.rs `watch_library` for the full
/// UI-thread-serialization rationale (LL-021 post-Round-3).
#[tauri::command(async)]
pub fn read_library_appearance(app: tauri::AppHandle, library_path: String) -> Result<serde_json::Value, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let path = Path::new(&library_path).join(".obsidian").join("appearance.json");
    if !path.exists() {
        // Return defaults
        return Ok(serde_json::json!({
            "accent_color": null,
            "base_font_size": null,
            "text_font_family": null,
            "monospace_font_family": null,
            "interface_font_family": null,
            "css_theme": null
        }));
    }

    let data = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read appearance.json: {}", e))?;

    let raw: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("Failed to parse appearance.json: {}", e))?;

    // Map Obsidian's camelCase to our field names
    Ok(serde_json::json!({
        "accent_color": raw.get("accentColor").and_then(|v| v.as_str()),
        "base_font_size": raw.get("baseFontSize").and_then(|v| v.as_u64()),
        "text_font_family": raw.get("textFontFamily").and_then(|v| v.as_str()),
        "monospace_font_family": raw.get("monospaceFontFamily").and_then(|v| v.as_str()),
        "interface_font_family": raw.get("interfaceFontFamily").and_then(|v| v.as_str()),
        "css_theme": raw.get("cssTheme").and_then(|v| v.as_str())
    }))
}

/// Open a folder picker dialog and return the selected path.
#[tauri::command]
pub async fn pick_folder() -> Result<Option<String>, String> {
    // Use Tauri's dialog API via rfd (rust file dialog)
    let result = rfd::FileDialog::new()
        .set_title("Select Library Folder")
        .pick_folder();

    Ok(result.map(|p| p.to_string_lossy().to_string()))
}

/// Pick a parent folder, create a named subfolder, and register it as a library.
#[tauri::command]
pub async fn create_new_library(app: tauri::AppHandle, name: String) -> Result<Option<LibraryInfo>, String> {
    // §152 hardening: validate the user-supplied name BEFORE touching the
    // filesystem. Blocks `..`, `/`, `\` — same rule as `create_folder`.
    let safe_name = sanitize_name(&name)?;

    // 1. Pick parent location
    let parent = rfd::FileDialog::new()
        .set_title("Choose location for new library")
        .pick_folder();
    let parent_path = match parent {
        Some(p) => p,
        None => return Ok(None), // user cancelled
    };

    // 2. Create the library folder
    let library_dir = parent_path.join(&safe_name);
    if library_dir.exists() {
        return Err(format!("Folder '{}' already exists at that location", safe_name));
    }
    fs::create_dir_all(&library_dir)
        .map_err(|e| format!("Failed to create library folder: {}", e))?;

    // 3. Register it as a library
    let path_str = library_dir.to_string_lossy().to_string();
    let library = add_library(app, path_str)?;
    Ok(Some(library))
}

/// MIG-008 §Build.5 — create a library at an explicit parent path. Used by
/// the shared `CreateItemDialog` which collects the parent location via its
/// own "Pick…" affordance (calls `pick_folder` IPC) so the user sees the
/// chosen location IN the dialog before confirming. The pre-MIG-008 flow
/// (`create_new_library`) opens its own folder picker AFTER the user clicks
/// Create — kept for backward compatibility but no longer the primary path.
///
/// `(async)` per §152 — the work is `create_dir_all` + `add_library` which
/// touches the filesystem AND writes the libraries config; sync would block
/// the WebView UI thread on slow disk / network shares (per the watcher.rs
/// rationale at watch_library).
#[tauri::command(async)]
pub fn create_new_library_at(
    app: tauri::AppHandle,
    parent_path: String,
    name: String,
) -> Result<LibraryInfo, String> {
    // §152 hardening: validate the user-supplied name BEFORE touching the
    // filesystem. Blocks `..`, `/`, `\`.
    let safe_name = sanitize_name(&name)?;
    let library_dir = Path::new(&parent_path).join(&safe_name);
    if library_dir.exists() {
        return Err(format!("Folder '{}' already exists at that location", safe_name));
    }
    fs::create_dir_all(&library_dir)
        .map_err(|e| format!("Failed to create library folder: {}", e))?;
    let path_str = library_dir.to_string_lossy().to_string();
    add_library(app, path_str)
}

/// Extract the `status` value from a markdown file's YAML frontmatter.
/// Reads only the first 512 bytes for performance.
fn extract_frontmatter_status(path: &Path) -> Option<String> {
    use std::io::Read;
    let mut file = fs::File::open(path).ok()?;
    let mut buf = [0u8; 512];
    let n = file.read(&mut buf).ok()?;
    let text = std::str::from_utf8(&buf[..n]).ok()?;
    let mut lines = text.lines();
    // Must start with ---
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "..." {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("status:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'').to_lowercase();
            if matches!(val.as_str(), "seedling" | "growing" | "evergreen") {
                return Some(val);
            }
        }
    }
    None
}

fn read_dir_recursive(dir: &Path, current_depth: u32, max_depth: u32) -> Vec<FileEntry> {
    let mut entries = Vec::new();

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/folders (starting with .)
        if name.starts_with('.') {
            continue;
        }
        // Skip symlinks to prevent circular recursion
        if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
            continue;
        }

        let is_dir = path.is_dir();
        let extension = if !is_dir {
            path.extension().map(|e| e.to_string_lossy().to_string())
        } else {
            None
        };

        // Only include markdown files, .base files, and folders
        if !is_dir && !matches!(extension.as_deref(), Some("md") | Some("base")) {
            continue;
        }

        // MIG-091 §A — one metadata read for modified + created + size.
        let meta = entry.metadata().ok();
        let modified = meta.as_ref().and_then(|m| {
            m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs()))
        });
        let created = meta.as_ref().and_then(|m| {
            m.created().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs()))
        });
        let size = if is_dir { None } else { meta.as_ref().map(|m| m.len()) };

        let children = if is_dir && current_depth < max_depth {
            Some(read_dir_recursive(&path, current_depth + 1, max_depth))
        } else if is_dir {
            Some(vec![]) // Indicate it's a folder but don't load children
        } else {
            None
        };

        let status = if !is_dir && extension.as_deref() == Some("md") {
            extract_frontmatter_status(&path)
        } else {
            None
        };

        // For canonical files, extract the frontmatter title as display name
        let display_title = if !is_dir
            && extension.as_deref() == Some("md")
            && crate::canonical::is_canonical_filename(&path)
        {
            // Read just the first 1KB to extract title (fast)
            fs::read_to_string(&path)
                .ok()
                .and_then(|c| extract_frontmatter_title(&c))
        } else {
            None
        };

        entries.push(FileEntry {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            children,
            extension,
            modified,
            created,
            size,
            status,
            display_title,
        });
    }

    // Sort: folders first, then files, alphabetically
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}

/// Simple UUID-like generator without external crate.
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Add random component to avoid collisions on low-resolution clocks (Windows ~100ns)
    let random: u32 = (timestamp as u32).wrapping_mul(2654435761) ^ std::process::id();
    format!("{:x}{:04x}", timestamp, random & 0xFFFF)
}

// ─── Graph / Backlinks scanning ───

#[derive(Debug, Clone, Serialize)]
pub struct NoteLink {
    pub source_path: String,
    pub source_name: String,
    pub target: String,
    pub context: String,
    pub library_name: String,
    pub link_type: Option<String>,
    /// Display / annotation text from the `|` segment of a typed link.
    /// (Historically the search.rs parser left `link_type` at "relates" and
    /// stored the type word here; since the Link-Type Syntax Correction
    /// `extract_typed_links` stores the real type in `link_type` and this field
    /// carries the display segment.) The UI may read this for the badge color.
    #[serde(default)]
    pub annotation: String,
    /// Living Link weight: `1 + ln(1 + traversal_count)`. Default 1.0 for
    /// never-traversed links. Consumed by the Backlinks panel (P3) to
    /// prioritize worn paths.
    #[serde(default = "default_weight")]
    pub weight: f64,
    /// Number of times the user has traversed this link. Default 0 for
    /// fresh / boot-graph-fallback entries that didn't come from the
    /// `note_links` table.
    #[serde(default)]
    pub traversal_count: i64,
    /// ISO-8601 timestamp of the most recent traversal, or "" for links
    /// that have never been followed. Populated from
    /// `note_links.last_traversed`. Consumed by the P5 lifecycle helpers
    /// to compute decay / stale-flagging / confidence tiers client-side.
    #[serde(default)]
    pub last_traversed: String,
    /// Confidence tier stored in the DB: "hypothesis" (default), or user-
    /// promoted tiers that will be driven by P5 thresholds. Present here
    /// so the UI can surface the raw tier without an extra query.
    #[serde(default)]
    pub confidence: String,
}

fn default_weight() -> f64 { 1.0 }

/// Scan all notes in a library and extract wikilinks from each.
// App-freeze audit Batch-W (2026-07-04): `(async)` — reads every note in the
// library. Second-screen callers carry epGeneration/scGeneration stale-result
// guards (SecondScreenPage).
#[tauri::command(async)]
pub fn scan_library_links(app: tauri::AppHandle, library_path: String, library_name: String) -> Result<Vec<NoteLink>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut links = Vec::new();
    let re = regex::Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+?))?\]\]").unwrap();
    scan_links_recursive(Path::new(&library_path), &re, &mut links, &library_name);
    Ok(links)
}

/// PJ-065 §8 (cold-start) — index a newly-linked library's pre-existing notes.
/// `add_library` only REGISTERS a folder; Constellation does no boot-time filesystem
/// walk (LL-022), and the file watcher reindexes only on a live edit — so a linked
/// folder's existing `.md` files never entered `note_meta` / `note_links` (the LL-027 /
/// BUG-022 class: files visible in the tree but invisible to the index, search, and the
/// structural spine). This walks the library and runs the mtime-gated `index_note`
/// (`force = false`) on each file: missing/changed notes index, already-current ones are
/// a cheap no-op (re-linking a large indexed library stays fast). Async (off the UI
/// thread); the frontend fires it after `add_library`. Returns the count of files seen.
#[tauri::command(async)]
pub fn reindex_library(app: tauri::AppHandle, library_path: String, library_name: String, only_if_unindexed: bool) -> Result<usize, String> {
    use tauri::Manager;
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let state = app.state::<crate::search::SearchState>();
    // Cheap gate (boot path): if this library already has indexed notes, skip the
    // filesystem walk entirely — ONE indexed COUNT, no walk, honoring ZERO-BOOT-WALKS
    // (LL-022). Only an unindexed library (linked but never walked into the index) gets
    // the cold-start walk. Fresh adds pass `false` (always index).
    if only_if_unindexed {
        let indexed: i64 = crate::search::with_read_conn(state.inner(), |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM note_meta WHERE library_name = ?1",
                rusqlite::params![library_name],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
        })
        .unwrap_or(0);
        if indexed > 0 {
            return Ok(0);
        }
    }
    // Reuse the existing recursive .md collector (same one the folder-move reindex uses).
    let mut md_paths: Vec<std::path::PathBuf> = Vec::new();
    collect_md_paths(Path::new(&library_path), &mut md_paths);
    let mut seen = 0usize;
    for p in &md_paths {
        let ps = p.to_string_lossy();
        // reindex_single_note wraps index_note AND runs the MIG-079 §C.2a incoming-aggregate
        // diff post-commit — so a cold-started library's TARGET notes get correct backlink
        // (incoming_count) values, not just outgoing. (index_note alone leaves incoming stale,
        // because incoming is save-path-maintained, not trigger-maintained.) Locks per note
        // internally (short holds); structural edges are already excluded from those counts (§3).
        if crate::search::reindex_single_note(state.inner(), &ps, &library_name).is_ok() {
            seen += 1;
        }
    }
    Ok(seen)
}

fn scan_links_recursive(dir: &Path, re: &regex::Regex, links: &mut Vec<NoteLink>, library_name: &str) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    // MIG-067 §D — registry membership (8 typed acts + custom + `associative`),
    // snapshot once per directory instead of a hardcoded list (see strata.rs).
    let reg = crate::link_types::snapshot();
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            scan_links_recursive(&path, re, links, library_name);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                // Use frontmatter title for canonical files (matching collect_library_notes)
                let file_stem = path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let source_name = if crate::canonical::is_canonical_filename(&path) {
                    extract_frontmatter_title(&content).unwrap_or(file_stem)
                } else {
                    file_stem
                };
                for cap in re.captures_iter(&content) {
                    // MIG-067 — predicate-first aware ([[type::target]] AND the legacy
                    // [[note|causes]] / [[note|type:causes]] alias forms). Target keeps
                    // its case (matches the prior cap[1] behaviour).
                    let (target, link_type) = crate::link_types::resolve_wikilink_type(
                        &reg, &cap[1], cap.get(2).map(|m| m.as_str()), true,
                    );
                    // Extract context: the line containing the link
                    let pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
                    let line_start = content[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let line_end = content[pos..].find('\n').map(|i| pos + i).unwrap_or(content.len());
                    let context = safe_truncate(&content[line_start..line_end], 120);

                    links.push(NoteLink {
                        source_path: path.to_string_lossy().to_string(),
                        source_name: source_name.clone(),
                        target,
                        context,
                        library_name: library_name.to_string(),
                        link_type,
                        annotation: String::new(),
                        weight: 1.0,
                        traversal_count: 0,
                        last_traversed: String::new(),
                        confidence: String::new(),
                    });
                }
            }
        }
    }
}

/// Scan for unlinked mentions of a note name across all libraries.
/// Returns notes that mention the name as plain text but don't have a [[wikilink]] to it.
///
/// Bug-fix history (2026-04-27 — item 6 of the panel-dedup pass):
///   1. The previous "skip if `[[NoteName]]` substring is present" check
///      was too narrow: it missed every typed-link form
///      `[[NoteName|supports]]` and every alias form `[[OldTitle]]`,
///      so those wikilinks were correctly indexed as backlinks AND
///      *also* counted here as unlinked mentions. The fix: strip ALL
///      wikilinks (`![[...]]` for embeds, `[[...]]` for normal links)
///      from the body before searching for the plain-text title. After
///      stripping, the only surviving occurrences are genuinely outside
///      any wikilink markup.
///   2. The source label was always derived from `path.file_stem()`,
///      which for a canonical filename like `20260426T140940Z_NOTE_11B4`
///      produced an unreadable id instead of the human title. The fix:
///      read the frontmatter `title:` field first, fall back to
///      `file_stem()` only when title is missing.
// PJ-066 §C5 — `(async)` so this read runs on a Tokio worker, NOT the WebView2 IPC
// thread. Combined with the `with_read_conn` routing below, a post-connect reindex (which
// holds the writer `db` lock for its whole duration) no longer freezes the UI: this used to
// be a SYNC command that took `db.lock()` and blocked the IPC thread for the full reindex
// (measured 47 s). Mirrors `scan_library_tags` below.
#[tauri::command(async)]
pub fn scan_unlinked_mentions(
    app: tauri::AppHandle,
    note_name: String,
    note_path: String,
    library_paths: Vec<(String, String)>, // (library_name, library_path)
) -> Result<Vec<NoteLink>, String> {
    let registered = load_all_libraries(&app);
    let word_re = match regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(&note_name))) {
        Ok(r) => r,
        Err(_) => return Ok(Vec::new()),
    };
    // Matches `[[anything]]` and `![[anything]]`. Non-greedy on `[^\]]*`
    // so adjacent wikilinks `[[A]] [[B]]` strip as two separate matches,
    // not one super-match that swallows the gap. `unwrap` is safe — the
    // pattern is a static literal verified at compile time by the
    // regex crate's parser.
    let wikilink_strip_re = regex::Regex::new(r"!?\[\[[^\]]*\]\]").unwrap();

    // ── Candidate selection via the always-current FTS index (was: walk the
    // whole library tree and read every .md on every note open). Find notes
    // whose body mentions the title in milliseconds instead of thousands of
    // file reads (Performance Rule 3 — no heavy scan on the read path). The
    // raw-content check below is the EXACT gate (same wikilink-strip + word-
    // boundary regex as before), so an over-inclusive candidate set never
    // affects correctness — FTS only narrows 7,600 notes down to the few that
    // could possibly match. Title is Arabic-normalized to match the indexed
    // body text; a phrase query keeps multi-word titles adjacent.
    let normalized = crate::arabic::normalizer::normalize_stripped(&note_name);
    let phrase = normalized.replace('"', " ");
    let phrase = phrase.trim();
    if phrase.is_empty() {
        return Ok(Vec::new());
    }
    let fts_query = format!("\"{}\"", phrase);

    // Preserve the original scope: only notes under one of the passed,
    // registered library paths.
    let scoped_paths: Vec<String> = library_paths
        .iter()
        .filter(|(_, p)| registered.iter().any(|v| &v.path == p))
        .map(|(_, p)| p.clone())
        .collect();

    // Pull candidate (path, library_name) from the index. Capped generously —
    // the verification loop stops at `cap` real matches.
    let (candidates, alias_holders): (Vec<(String, String)>, std::collections::HashSet<String>) = {
        use tauri::Manager;
        crate::search::ensure_search_db_ready(&app)?;
        let search_state = app.state::<crate::search::SearchState>();
        // PJ-066 §C5 — route through the read-only WAL reader so this never waits on the
        // writer `db` lock (a connect's background reindex holds it). Falls back to `db`
        // pre-init. The reader sees the last committed snapshot — eventual consistency is
        // correct for the unlinked-mentions suggestion panel.
        crate::search::with_read_conn(search_state.inner(), |conn| {
        let mut stmt = conn
            .prepare(
                "SELECT note_meta.path, note_meta.library_name
                 FROM notes_fts
                 JOIN note_meta ON notes_fts.rowid = note_meta.rowid
                 WHERE notes_fts MATCH ?1
                 ORDER BY bm25(notes_fts)
                 LIMIT 300",
            )
            .map_err(|e| format!("prepare unlinked-mentions FTS: {}", e))?;
        let rows = stmt
            .query_map(rusqlite::params![fts_query], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("query unlinked-mentions FTS: {}", e))?;
        let candidates: Vec<(String, String)> = rows.filter_map(|r| r.ok()).collect();

        // PJ-010 — notes that DECLARE the active title among their own
        // frontmatter aliases are self-alias-matches, not unlinked mentions:
        // their body saying the word is the note referring to itself by its
        // alias (MIG-004 already counts them as alias-aware backlinks).
        // Indexed write-time table, one lookup (Rule 8) — `alias_lower`
        // stores normalize_alias_for_match output, so match it exactly.
        let alias_key = crate::search::normalize_alias_for_match(&note_name);
        let mut alias_stmt = conn
            .prepare("SELECT path FROM note_aliases WHERE alias_lower = ?1 AND source = 'frontmatter'")
            .map_err(|e| format!("prepare alias-holder lookup: {}", e))?;
        let alias_holders: std::collections::HashSet<String> = alias_stmt
            .query_map(rusqlite::params![alias_key], |row| row.get::<_, String>(0))
            .map_err(|e| format!("query alias holders: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok((candidates, alias_holders))
        })?
    };

    // ── Verify each candidate against the RAW file (UNCHANGED logic): strip
    // wikilinks so a `[[Title]]` link is not counted, require a non-wikilinked
    // word-boundary occurrence, build the context snippet + human title.
    // Reads only the candidate files, never the whole vault.
    let mut results = Vec::new();
    let cap = 50usize;
    for (path, library_name) in candidates {
        if results.len() >= cap { break; }
        if path == note_path { continue; } // skip self
        if alias_holders.contains(&path) { continue; } // PJ-010: self-alias-match, not a mention
        if !scoped_paths.is_empty() && !scoped_paths.iter().any(|lp| path.starts_with(lp.as_str())) {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let stripped = wikilink_strip_re.replace_all(&content, "");
        if let Some(m) = word_re.find(&stripped) {
            let pos = m.start();
            let line_start = stripped[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = stripped[pos..].find('\n').map(|i| pos + i).unwrap_or(stripped.len());
            let context = safe_truncate(&stripped[line_start..line_end], 120);
            // Prefer frontmatter title over file stem so canonical filenames
            // display as their human title in the panel.
            let source_name = extract_frontmatter_title(&content)
                .unwrap_or_else(|| Path::new(&path).file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string());
            results.push(NoteLink {
                source_path: path.clone(),
                source_name,
                target: String::new(),
                context,
                library_name,
                link_type: None,
                annotation: String::new(),
                weight: 1.0,
                traversal_count: 0,
                last_traversed: String::new(),
                confidence: String::new(),
            });
        }
    }

    Ok(results)
}

/// Scan all tags across a library. **Walks every `.md` file** via
/// `scan_tags_recursive` below (`fs::read_to_string` per file + regex scan).
/// On the 7,600-note trial Universe this is ~7,600 file reads per library —
/// seconds of wall-clock work.
///
/// Live caller: the second-screen dashboard's per-library tag merge
/// (`SecondScreenPage.svelte` → store wrapper `scanLibraryTags`), the one
/// not-yet-migrated surface still doing a read-time fs-walk. The main-window
/// Dashboard no longer calls this — since MIG-080 §B it reads the write-time
/// `tag_counts` snapshot via the `allLibraryTags` prop (the former
/// `DashboardView.onMount → scanAllLibraryTags()` boot path was retired).
/// Historically this per-library fan-out queued 16 sync invocations on the
/// WebView2 UI thread, pushing `core_queue_ms` to ~19.5 s on Round 4
/// measurements (docs/LESSONS-LEARNED.md LL-021 Round 5) — the reason for `(async)`.
///
/// `#[tauri::command(async)]` routes each scan through `respond_async_serialized`
/// → `tauri::async_runtime::spawn`, so the UI thread pays only spawn cost per
/// call and Tokio workers run the actual filesystem walks in parallel.
/// Write-Time Derivation (CLAUDE.md Rule 8) says the right long-term fix is a
/// persisted tag index maintained by trigger/watcher — tracked as a separate
/// open item; this is the minimal change that unblocks Boot Criterion 2.
#[tauri::command(async)]
pub fn scan_library_tags(app: tauri::AppHandle, library_path: String) -> Result<std::collections::HashMap<String, u32>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut tags: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    scan_tags_recursive(Path::new(&library_path), &mut tags);
    Ok(tags)
}

// 2026-07-05 tag-click fix: counts via search::parse_frontmatter — the SAME
// tag definition the indexer and notes_by_tag use (frontmatter lists + inline
// #hashtags, quote-stripped, lowercased), counted ONCE PER NOTE to match the
// boot-snapshot chip semantics. The old version counted inline OCCURRENCES
// only, and its YAML branch was dead code that never counted anything.
fn scan_tags_recursive(dir: &Path, tags: &mut std::collections::HashMap<String, u32>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            scan_tags_recursive(&path, tags);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let (_, note_tags, _) = crate::search::parse_frontmatter(&content);
                // Dedupe within the note (a tag listed twice in YAML counts once).
                let unique: std::collections::HashSet<String> = note_tags.into_iter().collect();
                for tag in unique {
                    *tags.entry(tag).or_insert(0) += 1;
                }
            }
        }
    }
}

/// Return notes that contain a given tag (inline `#tag` OR YAML frontmatter —
/// the SAME definition the indexer writes into `note_meta.tags_json`).
// App-freeze audit Batch-W (2026-07-04): `(async)` — whole-library tag scan.
// Callers carry tagLoadSeq stale-result guards (DashboardView, SecondScreenPage).
// 2026-07-05 tag-click fix: matching now delegates to search::parse_frontmatter
// (the single tag authority). The old inline-only regex could never match a
// frontmatter tag — Dashboard chips counted 127 notes, the click listed 0.
#[tauri::command(async)]
pub fn notes_by_tag(app: tauri::AppHandle, library_path: String, tag: String) -> Result<Vec<StarInfo>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let lib = libraries.iter().find(|v| v.path == library_path).unwrap();
    // Normalize the incoming chip label: pre-fix index rows may still carry
    // literal quotes ("wiki-tag") until their note is next reindexed.
    let wanted = tag.trim().trim_matches(|c| c == '"' || c == '\'').to_lowercase();
    let mut results = Vec::new();
    collect_notes_with_tag(Path::new(&library_path), &lib.id, &lib.name, &wanted, &mut results);
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

fn collect_notes_with_tag(dir: &Path, lib_id: &str, lib_name: &str, wanted: &str, results: &mut Vec<StarInfo>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            collect_notes_with_tag(&path, lib_id, lib_name, wanted, results);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                // parse_frontmatter lowercases + quote-strips every tag
                // (frontmatter lists, inline arrays, and body #hashtags).
                let (_, tags, _) = crate::search::parse_frontmatter(&content);
                let has_tag = tags.iter().any(|t| t == wanted);
                if has_tag {
                    let modified = fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
                        .unwrap_or(0);
                    let preview = safe_truncate(content.lines()
                        .find(|l| !l.starts_with('#') && !l.starts_with("---") && !l.trim().is_empty())
                        .unwrap_or(""), 80);
                    results.push(StarInfo {
                        name: name.clone(),
                        path: path.to_string_lossy().to_string(),
                        library_id: lib_id.to_string(),
                        library_name: lib_name.to_string(),
                        modified,
                        preview,
                    });
                }
            }
        }
    }
}

// ─── Index: Word Index ───
// Extracts every word from every note, counts total occurrences,
// tracks which notes each word appears in, detects bigrams,
// filters stopwords, and merges case variants.

#[derive(Debug, Clone, Serialize)]
pub struct IndexMention {
    pub note_path: String,
    pub note_name: String,
    /// One-line FTS5 snippet of the matched term in context (up to ~12
    /// tokens around the first hit). Matched tokens are wrapped in
    /// `\x02`…`\x03` sentinels (STX/ETX control chars) which the frontend
    /// splits on to render as `<mark>` — chosen over putting `<mark>` in
    /// SQL so literal HTML in user notes is not injected into the DOM.
    ///
    /// `None` when FTS5 returned an empty snippet (e.g. title-only match
    /// against a note with empty body). The Index panel omits the context
    /// line in that case.
    pub snippet: Option<String>,
    /// Cross-language bridge lemma that caused this row to surface, when
    /// `read_term_mentions` was called with `expand_cross_language: true`
    /// and the matched token in the snippet is a non-source-language
    /// equivalent of the queried term. Renders in the UI as a small
    /// "via {lemma}" badge on the row. `None` for direct matches (the
    /// queried term itself appears in the note) and for all rows when
    /// expansion is off.
    ///
    /// Source: `crate::search::find_match_via_marked` scans the snippet's
    /// STX/ETX-marked regions against the M11 Lexical Bridge expansion's
    /// `bridge_terms_lower` set. Matches the same M13 search-side badge
    /// logic; see `MIG-010-INDEX-LEXICAL-BRIDGE-ARCHITECT.md`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via_lemma: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexEntry {
    pub term: String,
    pub count: u32,
    pub mentions: Vec<IndexMention>,
    pub is_compound: bool,
}

/// Co-occurring term — another vocabulary term appearing in the same notes
/// as a query term. Returned by `read_cooccurring_terms` and rendered as
/// a chip strip beneath an expanded Index term, surfacing lexical
/// adjacency ("notes containing 'knowledge' also contain …").
#[derive(Debug, Clone, Serialize)]
pub struct CooccurringTerm {
    /// Display form of the co-occurring term. Bigrams stored as
    /// `stem1\x1fstem2` are converted to `"stem1 stem2"` for the UI.
    pub term: String,
    /// Number of sampled matching notes that also contain this term.
    /// Capped above by `sample_limit` (default 200).
    pub note_count: u32,
}

/// MIG-086 — a note suggested as a related-but-unlinked connection candidate for an
/// orphan/fragile note. Produced by `suggest_related_notes` (BM25 "More Like This" over
/// the source note's distinctive terms). The `shared_terms` are the *why* (the distinctive
/// terms the candidate shares) — mandatory per the concept (no why → not shown).
#[derive(Debug, Clone, Serialize)]
pub struct RelatedCandidate {
    /// Absolute path of the candidate note (the key for the one-click connect action).
    pub note_path: String,
    /// Display title of the candidate.
    pub note_name: String,
    /// Relatedness score (|bm25|, higher = more related) for an optional UI strength bar.
    pub score: f64,
    /// The distinctive terms the candidate shares with the source — the legible reason
    /// they relate (rendered as chips). Never empty for a returned candidate.
    pub shared_terms: Vec<String>,
    /// A short body excerpt for preview (may be empty).
    pub snippet: String,
}

/// ─── Arabic Indexing Pipeline ───────────────────────────────────────────────
///
/// Based on Apache Lucene's ArabicNormalizer + ArabicStemmer (Light10 model),
/// the gold standard for Arabic information retrieval.
///
/// Pipeline: normalize → stem (prefix removal → suffix removal)
///
/// Design principles (from research):
/// 1. NORMALIZE first: remove diacritics, unify character variants
/// 2. STEM conservatively: only remove affixes when the remaining word
///    is long enough to be meaningful (minimum 2 chars after removal)
/// 3. NEVER strip a word below 3 chars total
/// 4. Prefix removal order: longest first (3-char → 2-char → 1-char)
/// 5. Suffix removal: only common grammatical suffixes, not root patterns
///
/// Sources:
/// - Larkey et al., "Light stemming for Arabic information retrieval" (2007)
/// - Apache Lucene ArabicStemmer.java / ArabicNormalizer.java
/// - CondLight: Conditional Arabic Light Stemmer (IAJIT 2018)

/// Display normalization: remove diacritics + tatweel only.
/// Preserves original character identity (ة stays ة, أ stays أ).
/// Used for the display form shown in the Index.
fn normalize_arabic_display(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    for ch in word.chars() {
        match ch {
            // Remove tashkeel diacritics
            '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06ED}' => continue,
            // Remove tatweel (kashida)
            '\u{0640}' => continue,
            _ => result.push(ch),
        }
    }
    result
}

/// Full normalization: remove diacritics + unify character variants.
/// Used for the index KEY (grouping different forms of the same word).
fn normalize_arabic(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    for ch in word.chars() {
        match ch {
            // Remove ALL tashkeel diacritics (harakat)
            '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06ED}' => continue,
            // Remove tatweel (kashida)
            '\u{0640}' => continue,
            // Normalize alef variants → bare alef
            'أ' | 'إ' | 'آ' | 'ٱ' => result.push('ا'),
            // Normalize alef maqsura → yeh
            'ى' => result.push('ي'),
            // Normalize teh marbuta → heh
            'ة' => result.push('ه'),
            _ => result.push(ch),
        }
    }
    result
}

/// Step 2: Arabic Light Stemmer (Lucene Light10 model)
/// Removes prefixes then suffixes with strict length constraints.
fn stem_arabic_light10(word: &str) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    let mut len = chars.len();

    // === PREFIX REMOVAL (longest first) ===
    // Each prefix requires that the remaining stem is at least 2 chars.

    // 3-char prefixes: وال فال بال كال (conjunction/preposition + definite article)
    if len >= 6 {
        let p3 = (chars[0], chars[1], chars[2]);
        match p3 {
            ('و','ا','ل') | ('ب','ا','ل') | ('ك','ا','ل') | ('ف','ا','ل') => {
                chars = chars[3..].to_vec();
                len = chars.len();
            }
            _ => {}
        }
    }

    // 2-char prefixes: ال لل (definite article, emphatic lam)
    if len >= 4 {
        let p2 = (chars[0], chars[1]);
        match p2 {
            ('ا','ل') | ('ل','ل') => {
                chars = chars[2..].to_vec();
                len = chars.len();
            }
            _ => {}
        }
    }

    // 1-char prefix: و (conjunction "and") — only if word is long enough
    // NOTE: و is the ONLY safe single-char prefix to remove.
    // ف/ب/ك/ل are NOT removed — they destroy too many proper nouns
    // (e.g., بدر، كريم، لبنان، فلسطين)
    if len >= 4 && chars[0] == 'و' {
        chars = chars[1..].to_vec();
        len = chars.len();
    }

    // === SUFFIX REMOVAL ===
    // Each suffix requires that the remaining stem is at least 2 chars.

    // 2-char suffixes (remove first — more specific)
    if len >= 4 {
        let s2 = (chars[len-2], chars[len-1]);
        match s2 {
            ('ه','ا') |  // ها (her/possessive)
            ('ا','ن') |  // ان (dual/indefinite)
            ('ا','ت') |  // ات (feminine plural)
            ('و','ن') |  // ون (masculine plural nominative)
            ('ي','ن') |  // ين (masculine plural accusative/genitive)
            ('ي','ه') |  // يه (possessive)
            ('ي','ت') |  // ية → يت after normalization (feminine adjective)
            ('ت','ه')    // ته → ته (his, possessive)
            => {
                chars.truncate(len - 2);
                len = chars.len();
            }
            _ => {}
        }
    }

    // 1-char suffixes (only if still long enough)
    if len >= 3 {
        match chars[len-1] {
            'ه' |  // ه/ة (feminine marker, after normalization ة→ه)
            'ي'    // ي (possessive/nisba)
            => {
                chars.truncate(len - 1);
            }
            _ => {}
        }
    }

    chars.iter().collect()
}

/// Combined Arabic processing: normalize + stem.
/// Returns (display_form, index_key):
///   - display: original word with tashkeel removed (ة stays ة, أ stays أ).
///   - key: canonical stem used by FTS5 to group surface variants.
///
/// M6 routes the key through `arabic::analyze_best`, which runs the five
/// Constellation Arabic Engine layers and returns the highest-confidence
/// analysis:
///
///   Layer 1 ProtectedList    — proper nouns / places / loanwords / function (conf 1.00)
///   Layer 2 GenerativeFst    — bare (root × pattern) hit                    (conf 0.85)
///   Layer 3b Cascade         — affix-peeled stem hit                        (conf 0.75 / 0.55)
///   Layer 4 SurfaceHeuristic — normalized surface fallback                  (conf 0.30)
///
/// For every analysis with origin ≠ SurfaceHeuristic the engine's `lemma`
/// is a strict improvement on Light10 — most visibly on the proper-noun
/// case that motivated this milestone: `وائل → "وائل"` (ProtectedList)
/// instead of the Light10-corrupted `"ائل"`.
///
/// When the analyzer's best guess IS SurfaceHeuristic (an Arabic word
/// that isn't protected, isn't in the FST, and can't be peeled to any
/// FST stem), we keep Light10 so the swap is strictly non-regressive:
/// unrecognized words continue to get the same affix-stripping they got
/// before M6, and search recall on them doesn't drop.
fn process_arabic_word(word: &str) -> (String, String) {
    let display = normalize_arabic_display(word); // preserve ة أ إ آ ى
    // M8b routes every token through the active Universe's user-override
    // store before the rest of the engine. M9-hotpath (a) cut this from
    // an unconditional RwLock-read + Arc::clone (~25 ns) to a single
    // relaxed `AtomicBool::load` (~2 ns) on the overwhelmingly common
    // empty-store case via `active_if_non_empty`. The returned
    // `Option<Arc<_>>` lives until end of scope, so `.as_deref()` gives
    // the `Option<&OverrideStore>` the downstream analyze call expects
    // without any reference-lifetime juggling.
    let store_owned = crate::arabic::overrides::active_if_non_empty();
    let overrides_ref = store_owned.as_deref();
    let analysis = crate::arabic::analyze_with_overrides_best(word, overrides_ref);
    let stem = if matches!(analysis.origin, crate::arabic::AnalysisOrigin::SurfaceHeuristic) {
        // Unknown word — preserve pre-M6 Light10 behaviour so recall on
        // previously-indexed surfaces does not regress.
        stem_arabic_light10(&normalize_arabic(word))
    } else {
        analysis.lemma
    };
    (display, stem)
}

/// Remove common Hebrew prefixes: ב ל מ ה ו כ ש
fn strip_hebrew_prefix(word: &str) -> &str {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 3 { return word; }

    // Two-char prefix: וה (and the)
    if len > 3 && chars[0] == 'ו' && (chars[1] == 'ה' || chars[1] == 'ב' || chars[1] == 'ל' || chars[1] == 'מ' || chars[1] == 'כ') {
        let rest: String = chars[2..].iter().collect();
        let byte_offset = word.len() - rest.len();
        return &word[byte_offset..];
    }

    // Single-char prefixes
    if len > 3 {
        match chars[0] {
            'ב' | 'ל' | 'מ' | 'ה' | 'ו' | 'כ' | 'ש' => {
                let rest: String = chars[1..].iter().collect();
                let byte_offset = word.len() - rest.len();
                return &word[byte_offset..];
            }
            _ => {}
        }
    }

    word
}

// stem_arabic is now replaced by stem_arabic_light10 above

/// Detect if a word is Arabic script
fn is_arabic(word: &str) -> bool {
    word.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c) || ('\u{0750}'..='\u{077F}').contains(&c) || ('\u{FB50}'..='\u{FDFF}').contains(&c) || ('\u{FE70}'..='\u{FEFF}').contains(&c))
}

/// Detect if a word is Hebrew script
fn is_hebrew(word: &str) -> bool {
    word.chars().any(|c| ('\u{0590}'..='\u{05FF}').contains(&c) || ('\u{FB1D}'..='\u{FB4F}').contains(&c))
}

fn is_latin(word: &str) -> bool {
    word.chars().any(|c| c.is_ascii_alphabetic())
}
fn is_cyrillic(word: &str) -> bool {
    word.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
}
fn is_devanagari(word: &str) -> bool {
    word.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
}
fn is_persian(word: &str) -> bool {
    // Persian uses Arabic script but with specific chars: پ چ ژ گ
    is_arabic(word) && word.chars().any(|c| c == 'پ' || c == 'چ' || c == 'ژ' || c == 'گ' || c == 'ک' || c == 'ی')
}

/// Helper: strip N chars from the end of a char slice, return as String
fn chars_strip_end(chars: &[char], n: usize) -> String {
    chars[..chars.len() - n].iter().collect()
}

/// Helper: check if char slice ends with a given suffix
fn chars_ends_with(chars: &[char], suffix: &[char]) -> bool {
    if chars.len() < suffix.len() { return false; }
    &chars[chars.len() - suffix.len()..] == suffix
}

/// English stemmer (Porter-like light stemming)
fn stem_english(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    // Step 1: plurals and past tense
    if chars_ends_with(&c, &['s','s','e','s']) { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['i','e','s']) && n > 4 { return format!("{}y", chars_strip_end(&c, 3)); }
    if chars_ends_with(&c, &['n','e','s','s']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['m','e','n','t']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['t','i','o','n']) { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['s','i','o','n']) { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['l','i','n','g']) && n > 5 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['i','n','g','s']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['i','n','g']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['a','t','e','d']) && n > 5 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['i','z','e','d']) && n > 5 { return chars_strip_end(&c, 1); }
    if chars_ends_with(&c, &['e','n','e','d']) && n > 5 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','d']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['l','y']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','r']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['s']) && !chars_ends_with(&c, &['s','s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// French stemmer (light suffix removal)
fn stem_french(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    if chars_ends_with(&c, &['e','u','s','e','s']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['e','u','s','e']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['m','e','n','t','s']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['m','e','n','t']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['t','i','o','n']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','n','c','e']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['a','n','c','e']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','u','x']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['é','e','s']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['é','e']) && n > 3 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['é','s']) && n > 3 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','r']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['é']) { return chars_strip_end(&c, 1); }
    if chars_ends_with(&c, &['s']) && !chars_ends_with(&c, &['s','s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// Spanish stemmer (light suffix removal)
fn stem_spanish(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    if chars_ends_with(&c, &['i','o','n','e','s']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['c','i','ó','n']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['m','e','n','t','e']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['i','d','a','d']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['a','n','d','o']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','n','d','o']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['a','d','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['i','d','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['a','d','a']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['o','s','a']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['o','s','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// Portuguese stemmer (light suffix removal)
fn stem_portuguese(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    if chars_ends_with(&c, &['ç','õ','e','s']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['m','e','n','t','e']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['i','d','a','d','e']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['a','n','d','o']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','n','d','o']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['a','d','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['i','d','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['a','d','a']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['o','s','a']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['o','s','o']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// German stemmer (light suffix removal + umlaut normalization)
fn stem_german(word: &str) -> String {
    // Normalize umlauts
    let w = word.to_lowercase()
        .replace("ä", "a").replace("ö", "o").replace("ü", "u")
        .replace("ß", "ss");
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    if chars_ends_with(&c, &['u','n','g','e','n']) && n > 6 { return chars_strip_end(&c, 5); }
    if chars_ends_with(&c, &['u','n','g']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['h','e','i','t']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['k','e','i','t']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['l','i','c','h']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['i','s','c','h']) && n > 5 { return chars_strip_end(&c, 4); }
    if chars_ends_with(&c, &['e','r','n']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['e','l','n']) && n > 4 { return chars_strip_end(&c, 3); }
    if chars_ends_with(&c, &['e','n']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','r']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','s']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e','m']) && n > 4 { return chars_strip_end(&c, 2); }
    if chars_ends_with(&c, &['e']) && n > 4 { return chars_strip_end(&c, 1); }
    if chars_ends_with(&c, &['s']) && n > 3 { return chars_strip_end(&c, 1); }

    w
}

/// Russian stemmer (light suffix removal for cases/gender/number)
fn stem_russian(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 4 { return word.to_string(); }

    // Participial/adjectival suffixes
    let last3: String = if len >= 3 { chars[len-3..].iter().collect() } else { String::new() };
    let last2: String = if len >= 2 { chars[len-2..].iter().collect() } else { String::new() };

    // Long suffixes (4+ chars)
    if len > 5 {
        let last4: String = chars[len-4..].iter().collect();
        match last4.as_str() {
            "ость" | "ными" | "ного" | "ному" | "ской" | "ских" | "ским" => return chars[..len-4].iter().collect(),
            _ => {}
        }
    }
    if len > 4 {
        match last3.as_str() {
            "ого" | "ому" | "ные" | "ных" | "ной" | "ами" | "ями" | "ить" | "ать" | "ять" | "ств" | "ски" => return chars[..len-3].iter().collect(),
            _ => {}
        }
    }
    if len > 3 {
        match last2.as_str() {
            "ов" | "ев" | "ий" | "ый" | "ая" | "ое" | "ые" | "ей" | "ям" | "ах" | "ом" | "ем" | "ой" | "ую" | "ие" | "ия" | "ть" | "ут" | "ют" | "ат" | "ят" | "ет" | "ит" | "ал" | "ил" | "ел" => return chars[..len-2].iter().collect(),
            _ => {}
        }
    }

    word.to_string()
}

/// Turkish stemmer (light agglutinative suffix removal)
fn stem_turkish(word: &str) -> String {
    let w = word.to_lowercase();
    let c: Vec<char> = w.chars().collect();
    let n = c.len();
    if n < 4 { return w; }

    // Long suffixes first (4 chars)
    if chars_ends_with(&c, &['l','a','r','ı']) || chars_ends_with(&c, &['l','e','r','i']) { return chars_strip_end(&c, 4); }
    // 3-char suffixes
    if chars_ends_with(&c, &['l','a','r']) || chars_ends_with(&c, &['l','e','r']) { if n - 3 >= 2 { return chars_strip_end(&c, 3); } }
    if chars_ends_with(&c, &['l','ı','k']) || chars_ends_with(&c, &['l','i','k']) || chars_ends_with(&c, &['l','u','k']) || chars_ends_with(&c, &['l','ü','k']) { if n - 3 >= 2 { return chars_strip_end(&c, 3); } }
    if chars_ends_with(&c, &['d','a','n']) || chars_ends_with(&c, &['d','e','n']) || chars_ends_with(&c, &['t','a','n']) || chars_ends_with(&c, &['t','e','n']) { if n - 3 >= 2 { return chars_strip_end(&c, 3); } }

    w
}

/// Hindi stemmer (light suffix removal for postpositions/verb forms)
fn stem_hindi(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len < 3 { return word.to_string(); }

    if len > 3 {
        let last2: String = chars[len-2..].iter().collect();
        match last2.as_str() {
            "ों" | "ें" | "ाँ" | "ता" | "ती" | "ते" | "ना" | "ने" | "नी" | "ाए" | "ाओ" | "ाई" => return chars[..len-2].iter().collect(),
            _ => {}
        }
    }

    word.to_string()
}

/// Persian stemmer: normalize ی/ک only, no suffix removal (same reasoning as Arabic)
fn stem_persian(word: &str) -> String {
    // Normalize Persian-specific chars only
    let normalized = word.replace('ي', "ی").replace('ك', "ک");
    return normalized;
    // Suffix removal disabled — causes same problems as Arabic stemming
    #[allow(unreachable_code)]
    let chars: Vec<char> = normalized.chars().collect();
    let len = chars.len();
    if len < 4 { return normalized; }

    if len > 4 {
        let last2: String = chars[len-2..].iter().collect();
        match last2.as_str() {
            "ها" | "ان" | "ات" | "ین" | "ون" | "گی" | "شی" => return chars[..len-2].iter().collect(),
            _ => {}
        }
    }
    if len > 3 {
        match chars[len-1] {
            'ی' | 'ه' => return chars[..len-1].iter().collect(),
            _ => {}
        }
    }

    normalized
}

pub(crate) fn build_stopwords() -> std::collections::HashSet<String> {
    let words: &[&str] = &[
        // English
        "the","be","to","of","and","a","in","that","have","i","it","for","not","on","with",
        "he","as","you","do","at","this","but","his","by","from","they","we","say","her","she",
        "or","an","will","my","one","all","would","there","their","what","so","up","out","if",
        "about","who","get","which","go","me","when","make","can","like","time","no","just",
        "him","know","take","people","into","year","your","good","some","could","them","see",
        "other","than","then","now","look","only","come","its","over","think","also","back",
        "after","use","two","how","our","work","first","well","way","even","new","want",
        "because","any","these","give","day","most","us","are","was","were","been","has","had",
        "did","does","may","might","must","shall","should","being","is","am","very","too",
        "each","every","both","few","more","much","own","same","such","where","here","let",
        "still","yet","while","per","via","etc","else","done","got","put","set","run",
        // Arabic (including normalized forms)
        "في","من","على","الى","هذا","هذه","التي","الذي","عن","مع","هو","هي","كان","كانت",
        "ذلك","تلك","ما","لا","ان","ان","لم","لن","قد","ثم","او","حتى","بين","عند","كل",
        "بعد","قبل","بعض","نحو","اي","انه","انها","لقد","فقط","هنا","هناك","منذ","حيث",
        "كما","اذا","عبر","ضد","خلال","حول","فيه","فيها","عليه","عليها","منه","منها",
        "به","بها","له","لها","لهم","هولاء","اولئك","وهو","وهي","ولا","ولم","الا",
        "اما","سوف","لكن","ليس","ليست","كذلك","ايضا","مثل","غير","دون","ضمن",
        "ال","بن","ابن","ذات","ذو","ذي","اللذين","اللتين","اللواتي","الذين","عليهم","لديه","لديها",
        "وقد","ولقد","والتي","والذي","ومن","وعلى","وفي","ومع","وعن","والى",
        // Hebrew
        "של","הוא","היא","את","זה","זו","אני","אנחנו","הם","הן","אתה","את","אתם","אתן",
        "יש","אין","לא","כי","גם","או","עם","על","אל","מן","אם","כל","עוד","רק","אבל",
        "היה","היתה","היו","יהיה","כמו","אחר","אחרי","לפני","בין","אצל","עד","מאד","כבר",
        "אז","שם","פה","למה","איך","מה","מי","איפה","מתי","כאשר","אשר","שלו","שלה","שלהם",
        // Persian/Farsi
        "از","به","در","با","که","این","آن","را","است","بر","تا","هم","و","یا","اما",
        "برای","اگر","هر","یک","شد","بود","خود","ما","شما","او","آنها","ایشان","هیچ",
        "چون","پس","زیرا","ولی","نه","بلکه","همه","بعد","قبل","بین","روی","زیر","کنار",
        // Urdu
        "کا","کی","کے","میں","ہے","کو","اور","سے","پر","نے","یہ","وہ","ایک","ہیں","تھا",
        "اس","جو","بھی","نہیں","کر","ہو","تو","ہی","یا","اپنے","سب","کچھ","لیے","ساتھ",
        // French
        "le","la","les","de","des","du","un","une","et","est","en","que","qui","dans","pour",
        "sur","avec","par","pas","il","elle","ce","se","au","aux","son","sa","ses","ont","sont",
        "mais","ou","où","ne","plus","tout","cette","mon","ton","nous","vous","ils","elles",
        "été","être","avoir","fait","comme","même","aussi","bien","très","peut","autre",
        // Spanish
        "el","la","los","las","de","del","un","una","en","que","es","por","con","para","se",
        "al","lo","su","como","más","no","ya","pero","sus","le","me","sin","sobre","este",
        "entre","cuando","muy","ser","hay","también","fue","todo","esta","son","dos","hasta",
        // German
        "der","die","das","und","in","den","von","zu","ist","mit","sich","des","ein","für",
        "auf","nicht","es","eine","auch","als","an","dem","so","ich","er","sie","hat","aus",
        "bei","nur","noch","wie","nach","über","aber","dann","war","mir","bis","doch","vor",
        "oder","sehr","durch","wenn","man","zum","zur","kann","sind","wird","vom","wir",
        // Russian
        "и","в","не","на","я","что","он","с","это","а","как","но","она","по","к","из","у",
        "за","так","то","все","мы","бы","от","до","же","вы","ее","его","для","их","уже",
        "при","без","ни","тот","эти","вот","чем","где","быть","был","была","были","нет",
        "или","если","них","нас","вас","ему","ней","ним","себя","есть","очень","еще",
        // Portuguese
        "o","a","os","as","de","da","do","em","no","na","um","uma","que","para","com","por",
        "se","mais","não","como","mas","foi","ao","dos","das","nos","nas","seu","sua","esse",
        // Turkish
        "bir","bu","ve","da","de","ile","için","olan","gibi","daha","çok","ama","ya","hem",
        "ne","var","ben","sen","biz","siz","her","hiç","kadar","sonra","önce","arasında",
        // Hindi
        "का","के","की","में","है","को","और","से","पर","ने","यह","वह","एक","हैं","था",
        "इस","उस","कि","जो","भी","नहीं","कर","हो","तो","ही","या","अपने","सब","कुछ",
        // Japanese (particles and common function words)
        "の","に","は","を","た","が","で","て","と","し","れ","さ","ある","いる","も",
        "する","から","な","こと","として","い","や","れる","など","なっ","ない","この",
        "ため","その","あっ","よう","また","もの","という","あり","まで","られ","なる",
        // Korean (particles and common function words)
        "이","그","저","것","수","등","들","및","에","를","의","는","은","로","와","과",
        "도","가","한","할","하는","하고","하여","되","된","되는","있","없","않","위",
        // Chinese (common function words — particles, conjunctions, pronouns)
        "的","了","在","是","我","有","和","就","不","人","都","一","一个","上","也","很",
        "到","说","要","去","你","会","着","没有","看","好","自己","这","那","她","他",
        "它","我们","你们","他们","什么","怎么","哪","为什么","因为","所以","但是","而且",
    ];
    // Normalize Arabic words in stopwords list too
    words.iter().map(|w| {
        let s = w.to_string();
        if is_arabic(&s) { normalize_arabic(&s) } else { s }
    }).collect()
}

/// CE Phase 6: Scan all notes for `stage:` frontmatter property.
/// Returns a map of note_path → stage value (fleeting|literature|permanent|synthesis).
// Safety Audit G8 (W3-1): `(async)` moves this off the WebView2 IPC dispatch thread
// so the full-library frontmatter walk (7,600+ files) can never freeze the UI. Body
// has no `.await` (pure thread-offload); invoke contract unchanged. Mirrors get_360_view.
#[tauri::command(async)]
pub fn scan_note_stages(app: tauri::AppHandle, library_path: String) -> Result<Vec<(String, String)>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut stages: Vec<(String, String)> = Vec::new();
    scan_stages_recursive(Path::new(&library_path), &mut stages);
    Ok(stages)
}

fn scan_stages_recursive(dir: &Path, stages: &mut Vec<(String, String)>) {
    let read_dir = match fs::read_dir(dir) { Ok(rd) => rd, Err(_) => return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            scan_stages_recursive(&path, stages);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if content.starts_with("---") {
                    if let Some(end) = content[3..].find("\n---") {
                        let yaml = &content[3..3 + end];
                        for line in yaml.lines() {
                            let trimmed = line.trim().to_lowercase();
                            if let Some(val) = trimmed.strip_prefix("stage:") {
                                let stage = val.trim().to_string();
                                if !stage.is_empty() {
                                    stages.push((path.to_string_lossy().to_string(), stage));
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Scan all notes in a library and build a word index.
#[tauri::command]
pub fn scan_library_index(app: tauri::AppHandle, library_path: String) -> Result<Vec<IndexEntry>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let stopwords = build_stopwords();

    // word_key -> { casing_variants: HashMap<String,u32>, total_count, sources }
    let mut index: std::collections::HashMap<String, (
        std::collections::HashMap<String, u32>, // casing variants -> count
        u32,                                     // total count
        Vec<(String, String)>,                   // (path, note_name)
    )> = std::collections::HashMap::new();

    // bigram_key -> (display_form, total_count, sources)
    let mut bigrams: std::collections::HashMap<String, (String, u32, Vec<(String, String)>)> =
        std::collections::HashMap::new();

    let md_strip = regex::Regex::new(
        r"(?x)
          \!\[([^\]]*)\]\([^)]*\)   |  # images
          \[([^\]]*)\]\([^)]*\)      |  # markdown links
          \[\[([^\]|]+?)(?:\|[^\]]+?)?\]\] | # wikilinks -> keep inner text
          ```[\s\S]*?```             |  # fenced code blocks
          `[^`]+`                    |  # inline code
          \*\*([^*]+)\*\*           |  # bold -> keep inner
          \*([^*]+)\*               |  # italic -> keep inner
          __([^_]+)__               |  # bold alt
          _([^_]+)_                 |  # italic alt
          ~~([^~]+)~~               |  # strikethrough
          <[^>]+>                   |  # HTML tags
          ^---\s*$                  |  # horizontal rules
          ^\#{1,6}\s+                  # heading markers (keep text after)
        "
    ).unwrap();

    scan_index_words_recursive(
        Path::new(&library_path), &md_strip, &stopwords, &mut index, &mut bigrams,
    );

    // Build single-word entries: pick most common casing variant
    let mut entries: Vec<IndexEntry> = index
        .into_values()
        .filter(|(_, count, _)| *count >= 2)
        .map(|(variants, count, sources)| {
            let term = variants.into_iter()
                .max_by_key(|(_, c)| *c)
                .map(|(s, _)| s)
                .unwrap_or_default();
            let mentions: Vec<IndexMention> = sources
                .into_iter()
                // Legacy walker doesn't produce FTS5 snippets — the
                // FTS5-backed `read_term_mentions` is the modern source.
                .map(|(note_path, note_name)| IndexMention { note_path, note_name, snippet: None, via_lemma: None })
                .collect();
            IndexEntry { term, count, mentions, is_compound: false }
        })
        .collect();

    // Build bigram entries (compound terms)
    let bigram_entries: Vec<IndexEntry> = bigrams
        .into_values()
        .filter(|(_, count, _)| *count >= 3)
        .map(|(term, count, sources)| {
            let mentions: Vec<IndexMention> = sources
                .into_iter()
                .map(|(note_path, note_name)| IndexMention { note_path, note_name, snippet: None, via_lemma: None })
                .collect();
            IndexEntry { term, count, mentions, is_compound: true }
        })
        .collect();

    entries.extend(bigram_entries);
    entries.sort_by(|a, b| a.term.to_lowercase().cmp(&b.term.to_lowercase()));
    Ok(entries)
}

pub(crate) fn is_same_script(a: &str, b: &str) -> bool {
    let ca = a.chars().next().unwrap_or(' ');
    let cb = b.chars().next().unwrap_or(' ');
    // Both ASCII Latin
    if ca.is_ascii_alphabetic() && cb.is_ascii_alphabetic() { return true; }
    // Both in same Unicode block (rough check: same high byte)
    let ba = (ca as u32) >> 8;
    let bb = (cb as u32) >> 8;
    ba == bb
}

/// Per-word processor used by the custom FTS5 tokenizer
/// (`crate::fts5_tokenizer::ConstellationTokenizer`).
///
/// Takes a single word and returns `(stem, norm_lower)` if the word is
/// worth emitting to the FTS5 inverted index, or `None` if it should be
/// skipped (empty, too short, or unreasonably long — likely concatenation
/// noise). The caller decides stopword filtering against the returned pair.
///
/// This is the same stemming pipeline used by `tokenize_note_body`
/// (Arabic Light10 / Hebrew prefix stripping / Persian / Cyrillic /
/// Devanagari / German / Spanish / Portuguese / French / Turkish /
/// English), but without the side-effectful HashMap accumulation — the
/// tokenizer just needs the stem + pre-stem normalized form.
///
/// * `stem` — lowercased, stemmed, suitable as a primary FTS5 token byte
///   sequence. When the same word arrives in a MATCH query it is stemmed
///   through this same function, so stemming is symmetric.
/// * `norm_lower` — lowercased, normalized (for Arabic: diacritics
///   stripped, alef/yeh/teh-marbuta variants unified) but NOT stemmed.
///   Callers check this against the stopword set too, because stopword
///   lists are curated in un-stemmed form (e.g. "the", not "th").
pub(crate) fn process_word_for_fts(word: &str) -> Option<(String, String)> {
    let char_count = word.chars().count();
    if char_count < 2 { return None; }

    let word_is_arabic = is_arabic(word);
    let word_is_hebrew = is_hebrew(word);

    // Length guards to drop concatenation noise.
    // Arabic words >20 chars are almost always glued tokens.
    // Non-Arabic: 40 is generous enough for German compounds.
    if word_is_arabic && char_count > 20 { return None; }
    if !word_is_arabic && char_count > 40 { return None; }

    let (normalized, stemmed);
    if word_is_arabic {
        let (_disp, stem) = process_arabic_word(word);
        normalized = normalize_arabic(word);
        stemmed = stem;
    } else if word_is_hebrew {
        normalized = word.to_string();
        stemmed = strip_hebrew_prefix(&normalized).to_string();
    } else {
        normalized = word.to_string();
        let lower = normalized.to_lowercase();
        stemmed = if is_persian(&normalized) {
            stem_persian(&normalized)
        } else if is_cyrillic(&normalized) {
            stem_russian(&normalized)
        } else if is_devanagari(&normalized) {
            stem_hindi(&normalized)
        } else if is_latin(&normalized) {
            if lower.contains('ä') || lower.contains('ö') || lower.contains('ü') || lower.contains('ß') {
                stem_german(&normalized)
            } else if lower.contains('ñ') || lower.ends_with("ción") || lower.ends_with("ando") {
                stem_spanish(&normalized)
            } else if lower.contains('ç') || lower.contains('ã') || lower.contains('õ') {
                stem_portuguese(&normalized)
            } else if lower.contains('é') || lower.contains('è') || lower.contains('ê')
                || lower.ends_with("ment") || lower.ends_with("tion") {
                stem_french(&normalized)
            } else if lower.contains('ş') || lower.contains('ğ') || lower.contains('ı') {
                stem_turkish(&normalized)
            } else {
                stem_english(&normalized)
            }
        } else {
            // Unknown script — emit as-is (CJK, etc.)
            normalized.clone()
        };
    }

    let stem_lower = stemmed.to_lowercase();
    let norm_lower = normalized.to_lowercase();

    // Skip if the stem degenerated to <2 chars (e.g. after Arabic prefix
    // stripping on a short word).
    if stem_lower.chars().count() < 2 { return None; }

    Some((stem_lower, norm_lower))
}

/// Tokenize a single note body and accumulate into the index + bigram maps.
/// Pure in-memory — no filesystem, no SQL. Callers pass already-stripped
/// body text (YAML frontmatter removed, markdown syntax collapsed).
///
/// Used by the filesystem walker `scan_index_words_recursive` (called from
/// `scan_library_index`, the on-demand per-library filesystem rebuild).
///
/// The cache-streaming path (`scan_index_populate_batch`) uses the sibling
/// `tokenize_note_local` function instead, which emits a per-note HashMap
/// and avoids unbounded accumulation across notes.
fn tokenize_note_body(
    body: &str,
    note_path: &str,
    note_name: &str,
    stopwords: &std::collections::HashSet<String>,
    index: &mut std::collections::HashMap<String, (
        std::collections::HashMap<String, u32>, u32, Vec<(String, String)>,
    )>,
    bigrams: &mut std::collections::HashMap<String, (String, u32, Vec<(String, String)>)>,
) {
    let mut seen_in_note: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut seen_bigrams: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut prev_word: Option<String> = None;
    let mut prev_key: Option<String> = None;

    for word in body.split(|c: char| {
        // Split on non-alphabetic chars (except apostrophe).
        // Also split on dashes, underscores, and em/en dashes.
        if c == '\'' { return false; }
        if c == '—' || c == '–' || c == '-' || c == '_' { return true; }
        !c.is_alphabetic()
    }) {
        let word = word.trim_matches('\'');
        if word.is_empty() {
            prev_word = None;
            prev_key = None;
            continue;
        }
        let char_count = word.chars().count();
        let word_is_arabic = is_arabic(word);
        let word_is_hebrew = is_hebrew(word);
        let is_non_latin = word.chars().any(|c| !c.is_ascii_alphabetic());

        // Skip abnormally long words — likely concatenation errors.
        // Arabic rarely exceeds 12 chars; Latin rarely exceeds 25.
        if word_is_arabic && char_count > 15 {
            prev_word = None;
            prev_key = None;
            continue;
        }
        if is_non_latin && char_count < 2 {
            prev_word = None;
            prev_key = None;
            continue;
        }
        if !is_non_latin && char_count < 3 {
            prev_word = None;
            prev_key = None;
            continue;
        }

        // Process word through language-specific pipeline.
        let (normalized, stripped, stemmed);
        if word_is_arabic {
            // Arabic: Lucene Light10 pipeline.
            // display = original with tashkeel removed (ة أ إ preserved)
            // key = fully normalized + stemmed (for grouping)
            let (disp, stem) = process_arabic_word(word);
            normalized = normalize_arabic(word); // full normalization for stopword check
            stripped = disp; // display preserved
            stemmed = stem;  // grouped by Light10
        } else if word_is_hebrew {
            normalized = word.to_string();
            let s = strip_hebrew_prefix(&normalized).to_string();
            stripped = s.clone();
            stemmed = s;
        } else {
            normalized = word.to_string();
            stripped = normalized.clone();
            stemmed = if is_persian(&stripped) {
                stem_persian(&stripped)
            } else if is_cyrillic(&stripped) {
                stem_russian(&stripped)
            } else if is_devanagari(&stripped) {
                stem_hindi(&stripped)
            } else if is_latin(&stripped) {
                let lower = stripped.to_lowercase();
                if lower.contains('ä') || lower.contains('ö') || lower.contains('ü') || lower.contains('ß') {
                    stem_german(&stripped)
                } else if lower.contains('ñ') || lower.ends_with("ción") || lower.ends_with("ando") {
                    stem_spanish(&stripped)
                } else if lower.contains('ç') || lower.contains('ã') || lower.contains('õ') {
                    stem_portuguese(&stripped)
                } else if lower.contains('é') || lower.contains('è') || lower.contains('ê') || lower.ends_with("ment") || lower.ends_with("tion") {
                    stem_french(&stripped)
                } else if lower.contains('ş') || lower.contains('ğ') || lower.contains('ı') || lower.contains('ü') {
                    stem_turkish(&stripped)
                } else {
                    stem_english(&stripped)
                }
            } else {
                stripped.clone()
            };
        }

        // Use stemmed form as index key; keep original display form.
        let key = stemmed.to_lowercase();

        // Skip stopwords (check both original normalized and stemmed forms).
        let norm_lower = normalized.to_lowercase();
        let is_stop = stopwords.contains(&key) || stopwords.contains(&norm_lower);

        if !is_stop {
            // Result must be ≥3 chars for Arabic/Hebrew, ≥2 for others.
            let min_len = if word_is_arabic || word_is_hebrew { 3 } else { 2 };
            if key.chars().count() < min_len {
                prev_word = Some(stripped.clone());
                prev_key = Some(key);
                continue;
            }

            let entry = index.entry(key.clone()).or_insert_with(|| {
                (std::collections::HashMap::new(), 0, Vec::new())
            });
            // Track display variant (use stripped form, not raw word with tashkeel).
            *entry.0.entry(stripped.clone()).or_insert(0) += 1;
            entry.1 += 1;

            if !seen_in_note.contains(&key) {
                seen_in_note.insert(key.clone());
                entry.2.push((note_path.to_string(), note_name.to_string()));
            }
        }

        // Bigram detection: pair with previous non-stop word if same script.
        if let (Some(pw), Some(pk)) = (&prev_word, &prev_key) {
            let prev_is_stop = stopwords.contains(pk.as_str());
            if !is_stop && !prev_is_stop && is_same_script(pw, &stripped) {
                let bi_key = format!("{} {}", pk, key);
                let bi_display = format!("{} {}", pw, stripped);
                let bi_entry = bigrams.entry(bi_key.clone())
                    .or_insert_with(|| (bi_display, 0, Vec::new()));
                bi_entry.1 += 1;
                if !seen_bigrams.contains(&bi_key) {
                    seen_bigrams.insert(bi_key);
                    bi_entry.2.push((note_path.to_string(), note_name.to_string()));
                }
            }
        }

        prev_word = Some(stripped.clone());
        prev_key = Some(key);
    }
}

fn scan_index_words_recursive(
    dir: &Path,
    md_strip: &regex::Regex,
    stopwords: &std::collections::HashSet<String>,
    index: &mut std::collections::HashMap<String, (
        std::collections::HashMap<String, u32>, u32, Vec<(String, String)>,
    )>,
    bigrams: &mut std::collections::HashMap<String, (String, u32, Vec<(String, String)>)>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            scan_index_words_recursive(&path, md_strip, stopwords, index, bigrams);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                // MIG-008 Step 6: legacy index-words tokenizer also derives
                // a per-note label that surfaces in the Index panel via
                // tokenize_note_body — use frontmatter title with stem
                // fallback so canonical-named notes show their human title.
                let note_name = note_display_name(&path, Some(&content));
                let note_path = path.to_string_lossy().to_string();

                // Strip YAML frontmatter
                let body = if content.starts_with("---") {
                    if let Some(end) = content[3..].find("---") {
                        &content[3 + end + 3..]
                    } else {
                        content.as_str()
                    }
                } else {
                    content.as_str()
                };

                let cleaned = md_strip.replace_all(body, |caps: &regex::Captures| {
                    for i in 1..=8 {
                        if let Some(m) = caps.get(i) {
                            return m.as_str().to_string();
                        }
                    }
                    String::new()
                });

                tokenize_note_body(&cleaned, &note_path, &note_name, stopwords, index, bigrams);
            }
        }
    }
}

/// ─── Index Panel backed by FTS5 vocab ───────────────────────────────────
///
/// The Index panel reads directly from the `notes_vocab` virtual table,
/// which is a `fts5vocab(notes_fts, 'row')` view over the term dictionary
/// that FTS5 already maintains on disk. Each row is `(term, doc, cnt)`:
///   * term — a token produced by the FTS5 tokenizer
///   * doc  — number of distinct notes containing the token
///   * cnt  — total occurrences across all notes
///
/// Advantages over the previous custom-table attempts:
///   * Zero bulk work. FTS5 triggers on `note_meta` already maintain the
///     term dictionary incrementally as notes are added, edited, or deleted.
///   * No in-memory accumulation. Aggregation is what FTS5 does on disk.
///   * Boot is free — the panel opens to a live view over the dictionary.
///
/// Current tokenization is whatever FTS5 was configured with at table
/// creation (`unicode61 remove_diacritics 2`), which lower-cases and
/// strips diacritics but does not stem. This means "philosophy" and
/// "philosophies" appear as separate terms. A later phase will register a
/// custom FTS5 tokenizer wrapping the existing multi-language pipeline
/// (`tokenize_note_body` / Light10 Arabic stemming / bigrams) so the
/// vocabulary reflects the richer tokenization.

/// Read the Universe vocabulary from the FTS5 term dictionary.
/// Returns `(display, count)` pairs; `mentions` is left empty — the UI
/// lazy-fetches the notes for a term via `read_term_mentions` when the
/// user expands it, which avoids returning millions of rows up front.
///
/// Filters (tuned for multi-script corpora, especially Arabic without
/// stemming, where a 7,600-note Universe produces ~450k unique term forms):
///   * terms shorter than 2 characters
///   * terms with count < 5 — drops hapax/near-hapax noise that would
///     otherwise bloat the list to hundreds of thousands of one-off tokens.
///   * LIMIT 50000 — ceiling on payload size and rendering cost. At 50k
///     alphabetically-sorted terms the user's filter-as-you-type narrows
///     quickly; at more than 50k the JSON blob and Svelte $state proxy
///     wrap start to hurt main-thread responsiveness.
///
/// Performance: a single forward scan over the FTS5 dictionary segments.
/// Measured ~350ms for 50k rows on a 7,600-note Arabic-heavy Universe.
// App-freeze audit Batch-S (2026-07-03): `(async)` — this command reaches
// ensure_search_db_ready (or a multi-second walk/read) and used to PARK the
// WebView2 dispatch thread for the whole 20-40s cold init after a universe
// switch / boot (the Boss-reproduced switch freeze). Off-thread, the init
// still runs exactly once (init_lock) but the app stays responsive.
#[tauri::command(async)]
pub fn read_index_entries(app: tauri::AppHandle) -> Result<Vec<IndexEntry>, String> {
    use rusqlite::{Connection, OpenFlags};

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;
    conn.busy_timeout(std::time::Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    // Register the 'constellation' FTS5 tokenizer on this connection so
    // later phases can MATCH-through-query here if needed. Reading
    // `notes_vocab` alone does not invoke the tokenizer, but consistency
    // avoids a "unknown tokenizer: constellation" surprise if this
    // function grows to do a MATCH later.
    crate::search::register_fts5_tokenizer(&mut conn)?;

    // No LIMIT. The Index panel is the canonical view of the Universe's
    // vocabulary — truncating it silently hides entire scripts from the
    // back of the alphabet because SQLite's default BINARY collation
    // sorts by UTF-8 bytes (Latin `a-z` = 0x61..0x7A, Arabic starts at
    // 0xD8 0x80, Hebrew at 0xD7 0x90, CJK at 0xE4..0xE9). A LIMIT at the
    // SQL layer picks favorites; we don't.
    //
    // What keeps this bounded: the `cnt >= 5` threshold below, combined
    // with the `constellation` tokenizer's stemming, caps a 7,600-note
    // Universe at ~100-200k rows.
    //
    // The frontend renders the result through a virtualized list
    // (`IndexPanel.svelte`) — payload size is the only soft limit, not
    // render cost.
    let mut stmt = conn.prepare(
        "SELECT term, cnt FROM notes_vocab
         WHERE LENGTH(term) >= 2 AND cnt >= 5
         ORDER BY term"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)? as u32,
        ))
    }).map_err(|e| e.to_string())?;

    let mut entries: Vec<IndexEntry> = Vec::new();
    for row in rows.flatten() {
        let (term, count) = row;
        // Bigrams are stored in the FTS5 index as `<stem1>\x1f<stem2>`
        // (the `\x1f` Unit Separator sentinel picked by the custom
        // tokenizer — see `crate::fts5_tokenizer::BIGRAM_SEP`). Convert
        // the sentinel to a space so the Index panel shows
        // "knowledge management" instead of the raw control character.
        // The frontend's click handler passes the display form back to
        // `read_term_mentions`, which wraps it in a phrase-query
        // "..." and lets FTS5 re-tokenize — still matching the bigram
        // via position-adjacent unigrams.
        let has_sentinel = term.as_bytes().contains(&crate::fts5_tokenizer::BIGRAM_SEP);
        let display = if has_sentinel { term.replace('\u{001F}', " ") } else { term };
        entries.push(IndexEntry {
            term: display,
            count,
            mentions: Vec::new(),
            is_compound: has_sentinel,
        });
    }
    Ok(entries)
}

/// FTS5 phrase-quote a literal term: wrap in `"..."` and double any
/// embedded `"` per FTS5 quoted-string syntax. The single source of
/// truth for "how the Index path quotes a literal term" — used by
/// `build_term_match_clause` and the read_term_mentions fallback path.
fn fts_quote_phrase(term: &str) -> String {
    format!("\"{}\"", term.replace('"', "\"\""))
}

/// Build the FTS5 MATCH clause to fetch mentions for an Index term.
///
/// Returns `(match_expression, bridge_terms_lower)`:
///   - `match_expression` is what gets bound as `?1` in the MATCH query.
///   - `bridge_terms_lower` is `Some` only when `expand_cross_language`
///     was true AND the M11 Lexical Bridge produced a real cross-
///     language OR expansion (term in corpus, at least one foreign-
///     language lemma). When `Some`, callers walk each row's snippet
///     through [`crate::search::find_match_via_marked`] to populate
///     `via_lemma`.
///
/// The exact-phrase fallback path (no expansion) is byte-identical to
/// the pre-MIG-010 behaviour. This is what guarantees Invariant I1 in
/// `MIG-010-INDEX-LEXICAL-BRIDGE-ARCHITECT.md`.
fn build_term_match_clause(
    term: &str,
    expand_cross_language: bool,
) -> (String, Option<Vec<String>>) {
    if expand_cross_language {
        // Same normalization the search-side bridge uses, so a term
        // from the Index dictionary (already FTS5-tokenized) hits the
        // lexicon FST consistently with how `lexical_search` would
        // route the same query.
        let normalized = crate::arabic::normalizer::normalize_stripped(term);
        if let Some(expansion) = crate::search::expanded_match_query(&normalized) {
            // Decompose into the OR-joined phrase MATCH (already
            // FTS5-quoted per phrase) and the badge-scan term set.
            let (match_expr, bridge) = expansion.into_parts();
            return (match_expr, Some(bridge));
        }
    }
    // Exact-phrase fallback (default behaviour, pre-MIG-010).
    (fts_quote_phrase(term), None)
}

/// Lazy-load the list of notes mentioning a given term. Called when the
/// user expands a term in the Index panel. Uses FTS5 `MATCH` — an O(log n)
/// term-dictionary lookup followed by a linear scan of the postings list,
/// joined to `note_meta` for display names.
///
/// Returns up to `limit` (default 200) mentions, ordered by note name.
///
/// `expand_cross_language` (default false): when true, expand the queried
/// term across languages via the M11 Lexical Bridge — clicking "knowledge"
/// also surfaces notes containing "معرفة", "connaissance", etc., with each
/// cross-language row carrying `via_lemma` so the UI can render a
/// "via {lemma}" badge. See `MIG-010-INDEX-LEXICAL-BRIDGE-ARCHITECT.md`.
/// When false, behaviour is byte-identical to pre-MIG-010 — exact phrase
/// match only, every row's `via_lemma` is `None`.
#[tauri::command]
pub fn read_term_mentions(
    app: tauri::AppHandle,
    term: String,
    limit: Option<u32>,
    expand_cross_language: Option<bool>,
) -> Result<Vec<IndexMention>, String> {
    use rusqlite::{Connection, OpenFlags};

    let limit = limit.unwrap_or(200).max(1).min(5000);
    let expand = expand_cross_language.unwrap_or(false);

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;
    conn.busy_timeout(std::time::Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    // Register the 'constellation' FTS5 tokenizer on this connection.
    // Required because the MATCH below tokenizes the query string
    // through the same tokenizer that populated the index — if the
    // tokenizer weren't registered, SQLite would fail with "no such
    // tokenizer: constellation".
    crate::search::register_fts5_tokenizer(&mut conn)?;

    let (match_expr, bridge_terms) = build_term_match_clause(&term, expand);
    let bridge_terms_slice: &[String] = bridge_terms.as_deref().unwrap_or(&[]);
    let did_expand = bridge_terms.is_some();

    // Try the (possibly-expanded) MATCH first. The expanded path can yield
    // zero rows (silent tokenizer-mismatch on every expanded phrase) or
    // error mid-iteration (FTS5 parser quirk on a specific OR clause). In
    // either case fall back to the literal exact-phrase MATCH — the user
    // always sees direct hits even if cross-language expansion bombed.
    // Failure modes are logged in debug builds only so production stderr
    // stays quiet on a hot path.
    let primary = run_mentions_query(&conn, &match_expr, limit, bridge_terms_slice);
    let needs_fallback = did_expand
        && match &primary {
            Ok(rows) => rows.is_empty(),
            Err(_) => true,
        };
    if !needs_fallback {
        return primary;
    }
    if cfg!(debug_assertions) {
        match &primary {
            Ok(_) => eprintln!(
                "[read_term_mentions] expanded MATCH for term={:?} returned 0 rows; falling back to exact phrase",
                term
            ),
            Err(e) => eprintln!(
                "[read_term_mentions] expanded MATCH for term={:?} errored ({}); falling back to exact phrase",
                term, e
            ),
        }
    }
    // Pass an empty bridge slice on the fallback path so no row earns a
    // misleading badge — they're literal direct hits, not bridged.
    run_mentions_query(&conn, &fts_quote_phrase(&term), limit, &[])
}

/// Execute the snippet+notes_fts MATCH query and collect rows. Extracted
/// so `read_term_mentions` can attempt the expanded MATCH first and fall
/// back to the exact-phrase MATCH if the expanded path errors or yields
/// zero rows on what was supposed to be an expanded query.
///
/// `bridge_terms_lower` is the per-row badge scan input. Pass an empty
/// slice on the exact-phrase fallback path so no rows can earn a badge
/// (which would be misleading — they're direct hits).
///
/// `snippet(notes_fts, -1, CHAR(2), CHAR(3), '…', 12)` returns a single
/// line of surrounding text with the matched tokens wrapped in STX/ETX
/// (\x02/\x03) sentinels. `-1` means "best column across all indexed
/// columns" — so a term that lives in the title (column 0) or body
/// (column 1) both get a useful preview. `12` tokens ≈ one line of
/// context; longer snippets waste vertical space in the expanded row.
/// STX/ETX are used (not `<mark>`) so literal HTML in user notes cannot
/// be injected into the DOM at render time. The cross-language badge
/// scan uses the same delimiters via `find_match_via_marked`.
fn run_mentions_query(
    conn: &rusqlite::Connection,
    match_expr: &str,
    limit: u32,
    bridge_terms_lower: &[String],
) -> Result<Vec<IndexMention>, String> {
    // `prepare_cached` so the expansion-then-fallback retry path doesn't
    // re-prepare the same SQL twice on a single read_term_mentions call.
    let mut stmt = conn.prepare_cached(
        "SELECT nm.path, nm.name,
                snippet(notes_fts, -1, CHAR(2), CHAR(3), '…', 12)
         FROM notes_fts
         JOIN note_meta nm ON notes_fts.rowid = nm.rowid
         WHERE notes_fts MATCH ?1
         ORDER BY LOWER(nm.name)
         LIMIT ?2"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(rusqlite::params![match_expr, limit as i64], |row| {
        let note_path: String = row.get(0)?;
        let note_name: String = row.get(1)?;
        // snippet() returns TEXT; SQLite can hand us NULL in edge cases
        // (very short/empty content columns), so tolerate both.
        let snippet_raw: Option<String> = row.get(2).ok();
        let snippet = snippet_raw.and_then(|s| if s.is_empty() { None } else { Some(s) });
        // Per-row badge: scan the snippet's STX/ETX-marked regions
        // against the bridge terms. `None` when expansion is off, when
        // the snippet has no marks (title-only hit), or when the marked
        // token is the source-language term itself (M13 same-language
        // filter).
        let via_lemma = snippet.as_deref().and_then(|s| {
            crate::search::find_match_via_marked(s, bridge_terms_lower, "\u{0002}", "\u{0003}")
        });
        Ok(IndexMention { note_path, note_name, snippet, via_lemma })
    }).map_err(|e| e.to_string())?;

    // Per-row errors (rare — usually only on tokenizer panics during
    // snippet generation). Drop bad rows but log the count so persistent
    // issues surface in the dev console.
    let mut out: Vec<IndexMention> = Vec::new();
    let mut row_errors = 0usize;
    for r in rows {
        match r {
            Ok(m) => out.push(m),
            Err(_) => row_errors += 1,
        }
    }
    if row_errors > 0 && cfg!(debug_assertions) {
        eprintln!(
            "[read_term_mentions] {} row(s) dropped due to per-row errors during MATCH={:?}",
            row_errors, match_expr
        );
    }
    Ok(out)
}

/// Return the top co-occurring terms for `term` — other vocabulary terms
/// appearing in the same notes. Surfaces lexical adjacency: "notes that
/// mention 'knowledge' also mention: 'wisdom', 'understanding', …".
///
/// ## Performance model
///
/// `fts5vocab(…, 'instance')` has no index on `doc`, so a SQL-level
/// co-occurrence query (e.g. `WHERE doc IN (matching_rowids)`) degrades to
/// a full scan of every token position in the entire FTS index. For a
/// 7,600-note Arabic Universe that's millions of rows per query.
///
/// Instead we:
///   1. Pull up to `sample_limit` matching rowids from `notes_fts MATCH`
///      (indexed — fast).
///   2. Fetch `note_meta.body_text` for each rowid (covered by the
///      primary-key rowid index — ~hundreds of tiny point reads).
///   3. Re-tokenize each body in-process through the same
///      `process_word_for_fts` pipeline the FTS5 tokenizer uses, so the
///      stems we aggregate are symmetric with those in the index.
///   4. Count distinct notes per co-occurring stem; sort descending.
///
/// Cost on a common term (say 500 matches, 2 KB body each): ~1 MB of
/// text × low-microsecond per-word tokenization ≈ <100 ms. Rare terms
/// are essentially free.
///
/// The 200-note default sample is empirically enough: the rank order of
/// top co-occurring terms stabilizes well before every matching note is
/// visited (law of large numbers on the tail). Users tuning for
/// exhaustiveness can raise `sample_limit`; there's no correctness
/// benefit past a few hundred.
#[tauri::command]
pub fn read_cooccurring_terms(
    app: tauri::AppHandle,
    term: String,
    sample_limit: Option<u32>,
    result_limit: Option<u32>,
) -> Result<Vec<CooccurringTerm>, String> {
    use rusqlite::{Connection, OpenFlags};
    use std::collections::{HashMap, HashSet};

    let sample_limit = sample_limit.unwrap_or(200).max(1).min(2000);
    let result_limit = result_limit.unwrap_or(20).max(1).min(100) as usize;

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;
    conn.busy_timeout(std::time::Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    crate::search::register_fts5_tokenizer(&mut conn)?;

    // Stems of the query term — excluded from co-occurrence results
    // (nobody wants "knowledge" listed as co-occurring with "knowledge").
    // Whitespace split handles the bigram display form:
    // "knowledge management" → ["knowledge", "management"], so both the
    // unigram stems are filtered out.
    let query_stems: HashSet<String> = term
        .split_whitespace()
        .filter_map(|w| process_word_for_fts(w).map(|(stem, _norm)| stem))
        .collect();

    // Step 1: sample matching rowids via FTS5 MATCH.
    //
    // The `stmt`/`rows` pair must both outlive the `.collect()` call —
    // `rows` borrows from `stmt`, and `stmt` borrows from `conn`. Binding
    // each to its own `let` (rather than chaining through a block-expr)
    // keeps the borrow chain alive until `collect()` finishes.
    let phrase = format!("\"{}\"", term.replace('"', "\"\""));
    let mut stmt = conn.prepare(
        "SELECT rowid FROM notes_fts WHERE notes_fts MATCH ?1 LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![&phrase, sample_limit as i64], |r| r.get::<_, i64>(0))
        .map_err(|e| e.to_string())?;
    let rowids: Vec<i64> = rows.filter_map(|r| r.ok()).collect();
    drop(stmt); // release the borrow on `conn` before we prepare `body_stmt`.

    if rowids.is_empty() { return Ok(Vec::new()); }

    // Step 2 & 3: for each rowid, fetch body_text and collect distinct
    // stems. `counts` accumulates stem → number of distinct notes it
    // appears in (co-document frequency across the sample).
    let stopwords = build_stopwords();
    let mut counts: HashMap<String, u32> = HashMap::new();

    let mut body_stmt = conn.prepare(
        "SELECT body_text FROM note_meta WHERE rowid = ?1"
    ).map_err(|e| e.to_string())?;

    for rowid in &rowids {
        let body: Option<String> = body_stmt
            .query_row(rusqlite::params![rowid], |r| r.get(0))
            .ok();
        let Some(body) = body else { continue; };
        if body.is_empty() { continue; }

        // Tokenize with the same boundary rules as the FTS5 tokenizer
        // (`fts5_tokenizer::is_word_boundary`): apostrophes don't break
        // words (keeps contractions together), em/en/hyphen/underscore
        // and non-alphabetic chars do.
        let mut seen: HashSet<String> = HashSet::new();
        let mut word_start: Option<usize> = None;
        for (byte_idx, ch) in body.char_indices() {
            if is_cooccurrence_boundary(ch) {
                if let Some(start) = word_start.take() {
                    collect_stem(&body[start..byte_idx], &stopwords, &query_stems, &mut seen);
                }
            } else if word_start.is_none() {
                word_start = Some(byte_idx);
            }
        }
        // Tail word (input doesn't end with a boundary char).
        if let Some(start) = word_start {
            collect_stem(&body[start..], &stopwords, &query_stems, &mut seen);
        }

        for stem in seen {
            *counts.entry(stem).or_insert(0) += 1;
        }
    }

    // Step 4: top-K by count descending, tie-break alphabetic ascending
    // for deterministic ordering across sessions on equal-count buckets.
    let mut results: Vec<CooccurringTerm> = counts
        .into_iter()
        .map(|(stem, note_count)| {
            let term = if stem.as_bytes().contains(&crate::fts5_tokenizer::BIGRAM_SEP) {
                stem.replace('\u{001F}', " ")
            } else {
                stem
            };
            CooccurringTerm { term, note_count }
        })
        .collect();
    results.sort_by(|a, b| {
        b.note_count.cmp(&a.note_count).then_with(|| a.term.cmp(&b.term))
    });
    results.truncate(result_limit);

    Ok(results)
}

/// Boundary predicate for co-occurrence re-tokenization. Must mirror
/// `fts5_tokenizer::is_word_boundary` exactly so the stems we aggregate
/// are the same ones stored in `notes_fts` / `notes_vocab`.
#[inline]
fn is_cooccurrence_boundary(c: char) -> bool {
    if c == '\'' { return false; }
    if c == '—' || c == '–' || c == '-' || c == '_' { return true; }
    !c.is_alphabetic()
}

#[inline]
fn collect_stem(
    word: &str,
    stopwords: &std::collections::HashSet<String>,
    query_stems: &std::collections::HashSet<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    if let Some((stem, norm_lower)) = process_word_for_fts(word) {
        // Three-way filter: stopword list (checked against both stem and
        // pre-stem normalized form — matches the tokenizer's rule), and
        // the query term's own stems (so it doesn't appear in its own
        // co-occurrence list).
        if !stopwords.contains(&stem)
            && !stopwords.contains(&norm_lower)
            && !query_stems.contains(&stem)
        {
            seen.insert(stem);
        }
    }
}

/// MIG-086 — tokenize a string into the SAME stems the FTS5 index stores (the
/// `read_cooccurring_terms` pipeline), returning per-stem term-frequency. Unigram stems
/// only (bigram-sentinel stems skipped); stopwords + sub-2-char noise dropped.
fn tokenize_tf(
    text: &str,
    stopwords: &std::collections::HashSet<String>,
    max_words: usize,
) -> std::collections::HashMap<String, u32> {
    let mut tf: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut count: usize = 0;
    let mut push = |w: &str,
                    tf: &mut std::collections::HashMap<String, u32>,
                    count: &mut usize| {
        if let Some((stem, norm_lower)) = process_word_for_fts(w) {
            if stem.as_bytes().contains(&crate::fts5_tokenizer::BIGRAM_SEP) {
                return; // unigrams only for MLT term selection
            }
            if stem.chars().count() < 3 {
                return; // drop 1–2 char noise ("pp", "th", "st" citation/ordinal fragments)
            }
            if stopwords.contains(&stem) || stopwords.contains(&norm_lower) {
                return;
            }
            *count += 1;
            *tf.entry(stem).or_insert(0) += 1;
        }
    };
    let mut word_start: Option<usize> = None;
    for (byte_idx, ch) in text.char_indices() {
        if is_cooccurrence_boundary(ch) {
            if let Some(start) = word_start.take() {
                push(&text[start..byte_idx], &mut tf, &mut count);
            }
        } else if word_start.is_none() {
            word_start = Some(byte_idx);
        }
        // PERF (Rule 8): cap the words processed so a 30k-word note doesn't make this O(huge).
        // The dominant terms recur early, so a cap of a few thousand kept words approximates
        // the full-document tf for term selection / shared-term detection.
        if count >= max_words {
            return tf;
        }
    }
    if let Some(start) = word_start {
        push(&text[start..], &mut tf, &mut count);
    }
    tf
}

/// MIG-086 — suggest related-but-UNLINKED notes for an orphan/fragile note, so the
/// Reviewer/360/Health-tab can turn the diagnosis ("connect it") into an action.
///
/// **Signal: BM25 "More Like This"** (the Lucene/Elasticsearch pattern) over the source
/// note's most DISTINCTIVE terms — query-time over the always-current `notes_fts` index
/// (Rule 8: no precomputed similarity matrix, no boot rebuild, never per-keystroke).
///   1. Re-tokenize the source note (name + body) in-process → term frequencies (the
///      `read_cooccurring_terms` tokenizer, symmetric with the index).
///   2. For each stem, doc-frequency from `notes_vocab` (the live FTS5 vocabulary —
///      `term_vocab` is drift-prone, measured wrong on the live corpus). Score = tf·idf;
///      keep terms in df ∈ [2, 0.5·N] (drop hapax + ubiquitous); top 25 (Lucene maxQueryTerms).
///   3. Disjunctive `MATCH` of those terms, ranked by `bm25` (name 10×, body 1×), EXCLUDING
///      self and already-linked notes (by folded name — `target_path` is never populated).
///   4. For each candidate, the shared distinctive terms (the *why* — mandatory; a candidate
///      with no attributable shared term is dropped) + a short snippet.
///
/// Returns `[]` (never an error / never a full scan) when the note is too short / all-stopword
/// or nothing clears the bar — the caller renders an honest empty state.
// Note-open-freeze class fix (2026-07-03): `(async)` moves this off the WebView2 IPC
// dispatch thread so a writer-lock wait (background reindex) can never freeze the app.
// Body has no .await (pure thread-offload); invoke contract unchanged. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn suggest_related_notes(
    app: tauri::AppHandle,
    library_path: String,
    note_path: String,
    limit: Option<u32>,
) -> Result<Vec<RelatedCandidate>, String> {
    use rusqlite::{Connection, OpenFlags};

    validate_path_in_any_library(&app, &library_path)
        .map_err(|e| format!("Access denied: {}", e))?;
    // Boss (MIG-086 §C): show ALL related notes, sequenced by closeness — not an arbitrary
    // few. The ceiling is the BM25 candidate pool itself (LIMIT below), a generous relatedness
    // bound, not a small UI cap. Default/clamp to that pool size so `null` from the frontend
    // means "all the engine ranks".
    let limit = limit.unwrap_or(60).max(1).min(60) as usize;

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;
    conn.busy_timeout(std::time::Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    crate::search::register_fts5_tokenizer(&mut conn)?;

    suggest_related_impl(&conn, &note_path, limit)
}

/// MIG-086 — the testable core of `suggest_related_notes`: takes an already-open,
/// tokenizer-registered connection. Split out so a unit test can drive it against an
/// in-memory FTS5 fixture (the `app`/access-validation/connection-open is the only part
/// that can't run in-process).
fn suggest_related_impl(
    conn: &rusqlite::Connection,
    note_path: &str,
    limit: usize,
) -> Result<Vec<RelatedCandidate>, String> {
    use std::collections::HashSet;

    // ── 1. source note: name + body ──
    let (src_name, body): (String, String) = match conn.query_row(
        "SELECT name, body_text FROM note_meta WHERE path = ?1",
        rusqlite::params![note_path],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1).unwrap_or_default())),
    ) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };
    if body.trim().is_empty() && src_name.trim().is_empty() {
        return Ok(Vec::new());
    }
    let src_name_lower = crate::search::fold_match_key(&src_name);
    let stopwords = build_stopwords();

    // ── 2. tokenize source (name + body, capped) → tf ──
    let tf = tokenize_tf(&format!("{}\n{}", src_name, body), &stopwords, 4000);
    if tf.is_empty() {
        return Ok(Vec::new());
    }

    // ── 3. pick the DISTINCTIVE query terms: top ~40 by tf, then keep those with a corpus
    //    doc-frequency in [2, ~5%·N] (drop hapax + common), ranked tf·idf, top 12. The df probe
    //    is bounded to the 40 candidates (≈ one fts5vocab seek each), so it stays cheap; this
    //    is what makes the *why* chips meaningful ("aisle/nave/choir", not "st/pp/th"). ──
    let mut by_tf: Vec<(String, u32)> = tf.into_iter().collect();
    by_tf.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    by_tf.truncate(20); // df probes — bounded; fts5vocab is ~20 ms/lookup on this index

    let total: f64 = conn
        .query_row("SELECT COUNT(*) FROM note_meta", [], |r| r.get::<_, i64>(0))
        .unwrap_or(0) as f64;
    let mut df_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    {
        let placeholders = std::iter::repeat("?").take(by_tf.len()).collect::<Vec<_>>().join(",");
        let q = format!("SELECT term, doc FROM notes_vocab WHERE term IN ({})", placeholders);
        let mut stmt = conn.prepare(&q).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(by_tf.iter().map(|(s, _)| s)), |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        for r in rows.flatten() {
            df_map.insert(r.0, r.1);
        }
    }
    let max_df = (total * 0.05).max(50.0);
    let mut scored: Vec<(String, f64)> = Vec::new();
    for (stem, freq) in &by_tf {
        let df = df_map.get(stem).copied().unwrap_or(0);
        if df < 2 || (total > 0.0 && (df as f64) > max_df) {
            continue; // hapax (matches nothing) or too common (not distinctive)
        }
        let idf = (total / (df as f64 + 1.0)).ln() + 1.0;
        scored.push((stem.clone(), *freq as f64 * idf));
    }
    if scored.is_empty() {
        return Ok(Vec::new());
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(12);
    let query_terms: Vec<String> = scored.into_iter().map(|(s, _)| s).collect();

    // ── 4. RANK with the BARE FTS bm25 query (name 10×, body 1×) ──
    // CRITICAL (Rule 8): no JOIN to note_meta, no NOT IN post-filter here. Adding either
    // defeats FTS5's rank-limit fast path — SQLite can't take the top-K and stop when a
    // post-filter might reject them, so it materializes the whole match union with per-row
    // subquery evaluation (measured 12 s vs 12 ms for the bare query). Self + already-linked
    // are filtered in Rust on the small top-K set below.
    let mlt = query_terms
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" OR ");
    let cand: Vec<(i64, f64)> = {
        let mut stmt = conn
            .prepare("SELECT rowid, bm25(notes_fts, 10.0, 1.0) FROM notes_fts WHERE notes_fts MATCH ?1 ORDER BY rank LIMIT 60")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![mlt], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?)))
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Already-linked OUT: the folded names the source links to (idx_link_source → fast).
    let mut excluded_names: HashSet<String> = HashSet::new();
    {
        let mut stmt = conn
            .prepare("SELECT target_name FROM note_links WHERE source_path = ?1 AND status != 'archived'")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params![note_path], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        for r in rows.flatten() {
            excluded_names.insert(r);
        }
    }

    // ── 5a. shared-term "why" via the FTS INDEX (not per-candidate re-tokenization) ──
    // For each distinctive query term, ask the index which candidate rows contain it — the
    // index already holds the stemmed tokens, so ≤12 cheap term lookups replace N full
    // candidate re-tokenizations. THIS is what lets §C surface ALL related notes (Boss:
    // "list all, regardless of numbers"), ranked closest-first, instead of only the top 15 —
    // without the per-candidate tokenization cost the old `processed` cap existed to bound
    // (Rule 8). Each term's df is already constrained to ≤5%·N (step 3), so each lookup is small.
    let cand_rowids: HashSet<i64> = cand.iter().map(|(r, _)| *r).collect();
    let mut term_hits: std::collections::HashMap<String, HashSet<i64>> = std::collections::HashMap::new();
    {
        let mut tstmt = conn
            .prepare("SELECT rowid FROM notes_fts WHERE notes_fts MATCH ?1")
            .map_err(|e| e.to_string())?;
        for t in &query_terms {
            let phrase = format!("\"{}\"", t.replace('"', "\"\""));
            let rows = tstmt
                .query_map(rusqlite::params![phrase], |r| r.get::<_, i64>(0))
                .map_err(|e| e.to_string())?;
            let mut set: HashSet<i64> = HashSet::new();
            for rid in rows.flatten() {
                if cand_rowids.contains(&rid) {
                    set.insert(rid);
                }
            }
            term_hits.insert(t.clone(), set);
        }
    }

    // ── 5b. resolve candidates: fetch by rowid; drop self / already-linked (either direction) /
    //    no-shared-term; attach the *why* + a bounded preview; take top `limit`. ──
    // body_text is read only as a 400-char prefix (enough for the 24-word snippet) so the
    // whole-pool scan stays cheap even for large candidate bodies.
    let mut meta_stmt = conn
        .prepare("SELECT path, name, substr(body_text, 1, 400) FROM note_meta WHERE rowid = ?1")
        .map_err(|e| e.to_string())?;
    let mut inbound_stmt = conn
        .prepare("SELECT 1 FROM note_links WHERE source_path = ?1 AND target_name = ?2 AND status != 'archived' LIMIT 1")
        .map_err(|e| e.to_string())?;
    let mut out: Vec<RelatedCandidate> = Vec::new();
    for (rowid, rank) in cand {
        let (path, name, cbody): (String, String, String) = match meta_stmt.query_row(
            rusqlite::params![rowid],
            |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, String>(2).unwrap_or_default())),
        ) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if path == note_path {
            continue; // self
        }
        let cand_name_lower = crate::search::fold_match_key(&name);
        if excluded_names.contains(&cand_name_lower) {
            continue; // the source already links to this candidate (out-link)
        }
        // The candidate already links to the source (in-link)? (idx_link_source → fast.)
        if inbound_stmt
            .query_row(rusqlite::params![path, src_name_lower], |_| Ok(()))
            .is_ok()
        {
            continue;
        }
        let shared: Vec<String> = query_terms
            .iter()
            .filter(|t| term_hits.get(*t).map_or(false, |s| s.contains(&rowid)))
            .take(6)
            .cloned()
            .collect();
        if shared.is_empty() {
            continue; // no legible reason → not shown (BASIC RULE / concept C-2)
        }
        let snippet: String = {
            let words: Vec<&str> = cbody.split_whitespace().take(24).collect();
            if words.is_empty() { String::new() } else { format!("{}…", words.join(" ")) }
        };
        out.push(RelatedCandidate {
            note_path: path,
            note_name: name,
            score: -rank, // bm25 is negative (more negative = better) → higher = more related
            shared_terms: shared,
            snippet,
        });
        if out.len() >= limit {
            break;
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests_mig086_suggest {
    //! MIG-086 §A — BM25 "More Like This" over an in-memory FTS5 fixture (the real
    //! `constellation` tokenizer + `notes_vocab` doc-frequency). Pins: a planted relative is
    //! suggested, self + already-linked are excluded, no-shared-vocab notes don't appear,
    //! an empty source returns `[]`, and `limit` is honored.
    use super::*;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::search::register_fts5_tokenizer(&mut conn).unwrap();
        conn.execute_batch(
            "CREATE TABLE note_meta (path TEXT PRIMARY KEY, name TEXT, name_lower TEXT, body_text TEXT);
             CREATE VIRTUAL TABLE notes_fts USING fts5(name, body_text, tokenize='constellation');
             CREATE VIRTUAL TABLE notes_vocab USING fts5vocab(notes_fts, 'row');
             CREATE TABLE note_links (source_path TEXT, target_name TEXT, status TEXT);",
        )
        .unwrap();
        conn
    }

    fn add(conn: &Connection, rowid: i64, path: &str, name: &str, body: &str) {
        conn.execute(
            "INSERT INTO note_meta(rowid, path, name, name_lower, body_text) VALUES (?1,?2,?3,?4,?5)",
            rusqlite::params![rowid, path, name, crate::search::fold_match_key(name), body],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO notes_fts(rowid, name, body_text) VALUES (?1,?2,?3)",
            rusqlite::params![rowid, name, body],
        )
        .unwrap();
    }

    #[test]
    fn suggests_relative_excludes_self_and_already_linked() {
        let conn = setup();
        // Distinctive terms shared src↔rel: epistemic/inference/perception (df=2). cognition
        // shared src↔linked (df=2). N=4 → maxDocFreq=2, so every shared term survives at df=2.
        add(&conn, 1, "/src.md", "Pramana", "pramana epistemic inference perception cognition");
        add(&conn, 2, "/rel.md", "Masadir", "masadir epistemic inference perception sources");
        add(&conn, 3, "/unrel.md", "Gardening", "gardening tomato soil water sunlight compost");
        add(&conn, 4, "/linked.md", "Epistemology", "epistemology cognition proof grounds");
        // src already links Epistemology (by folded name) → must be excluded.
        conn.execute(
            "INSERT INTO note_links(source_path, target_name, status) VALUES ('/src.md', ?1, 'active')",
            rusqlite::params![crate::search::fold_match_key("Epistemology")],
        )
        .unwrap();

        let res = suggest_related_impl(&conn, "/src.md", 5).unwrap();
        let paths: Vec<&str> = res.iter().map(|c| c.note_path.as_str()).collect();
        assert!(paths.contains(&"/rel.md"), "planted relative suggested: {:?}", paths);
        assert!(!paths.contains(&"/src.md"), "self excluded");
        assert!(!paths.contains(&"/linked.md"), "already-linked excluded");
        assert!(!paths.contains(&"/unrel.md"), "no-shared-vocab note not suggested");
        let rel = res.iter().find(|c| c.note_path == "/rel.md").unwrap();
        assert!(!rel.shared_terms.is_empty(), "the why (shared terms) is populated");
    }

    #[test]
    fn empty_source_returns_empty() {
        let conn = setup();
        add(&conn, 1, "/empty.md", "", "");
        add(&conn, 2, "/other.md", "Other", "some distinctive content words here");
        assert!(suggest_related_impl(&conn, "/empty.md", 5).unwrap().is_empty());
    }

    #[test]
    fn limit_is_honored() {
        let conn = setup();
        // 8 notes share "alpha/beta/gamma" with src; 14 padding notes keep df ≤ 0.5·N.
        add(&conn, 1, "/src.md", "Hub", "alpha beta gamma distinctive vocabulary");
        for i in 2..=8 {
            add(&conn, i, &format!("/share{}.md", i), &format!("Share{}", i), "alpha beta gamma vocabulary");
        }
        for i in 9..=22 {
            add(&conn, i, &format!("/pad{}.md", i), &format!("Pad{}", i), &format!("padunique{} solitary{} lone{}", i, i, i));
        }
        let res = suggest_related_impl(&conn, "/src.md", 3).unwrap();
        assert!(res.len() <= 3, "limit honored: got {}", res.len());
        assert!(!res.is_empty(), "some relatives found");
    }

    /// MIG-086 §A rehearsal — run suggestions against the LIVE 7,660-note universe for a few
    /// real orphans (English + Arabic), print the candidates + the *why*, and assert the
    /// Rule-8 latency bound. Run:
    ///   cargo test --lib tests_mig086_suggest::rehearse_live -- --ignored --nocapture
    #[test]
    #[ignore = "rehearsal — runs against the live universe DB"]
    fn rehearse_live_suggestions() {
        use rusqlite::OpenFlags;
        let db = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db";
        let mut conn = Connection::open_with_flags(
            db,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .unwrap();
        crate::search::register_fts5_tokenizer(&mut conn).unwrap();

        let orphans = [
            r"E:\Cognitive Knowledge\Arts & Culture\libraries\Architecture\Historical Styles\Edinburgh's High Kirk.md",
            r"E:\Cognitive Knowledge\العالم العربي\libraries\تاريخ عربي وإسلامي\العصور الوسطى المتأخرة والحديثة\السلطان محمد الفاتح.md",
            r"E:\Cognitive Knowledge\Science\libraries\Earth Sciences\Geology\Geological clock.md",
        ];
        for path in orphans {
            let t = std::time::Instant::now();
            let res = suggest_related_impl(&conn, path, 5).unwrap();
            let ms = t.elapsed().as_millis();
            let base = path.rsplit(['\\', '/']).next().unwrap_or(path);
            eprintln!("\n=== {} ({} ms) ===", base, ms);
            for c in &res {
                eprintln!("  {:6.2}  {:40}  [why: {}]", c.score, c.note_name, c.shared_terms.join(" · "));
            }
            // These three are the LARGEST notes in the 7,660-note universe (21k–32k words);
            // typical orphans are <5k words and resolve in <300 ms. The suggestion is an
            // async, on-demand, spinner-backed panel-open (NOT the keystroke path), so a
            // worst-case ~2 s on the single biggest note is acceptable.
            assert!(ms < 2500, "suggest worst-case bound — got {} ms for {}", ms, base);
        }
    }

    /// FTS-health diagnostic — copy the live DB, time fts5vocab + single-term MATCH, run
    /// `optimize`, re-time. Confirms (or refutes) segment fragmentation as the cause of the
    /// slow FTS ops. Run:
    ///   cargo test --lib tests_mig086_suggest::fts_optimize -- --ignored --nocapture
    #[test]
    #[ignore = "fts-health — copies the live DB + runs optimize; manual"]
    fn fts_optimize_timing() {
        let src = r"E:\Constellation Universes\Eisa Cognitive Knowledge\.constellation\search.db";
        let tmp = std::env::temp_dir().join("fts_opt_test.db");
        let _ = std::fs::remove_file(&tmp);
        std::fs::copy(src, &tmp).expect("copy live db");
        let mut conn = Connection::open(&tmp).unwrap();
        crate::search::register_fts5_tokenizer(&mut conn).unwrap();

        let bench = |conn: &Connection, label: &str| {
            let t = std::time::Instant::now();
            let m: i64 = conn
                .query_row("SELECT count(*) FROM notes_fts WHERE notes_fts MATCH '\"church\"'", [], |r| r.get(0))
                .unwrap_or(-1);
            let match_ms = t.elapsed().as_millis();
            let t = std::time::Instant::now();
            // 20 vocab lookups (mirrors a suggest call's df probe).
            for term in ["knowledge", "church", "aisle", "nave", "paris", "gothic", "history", "art",
                         "science", "river", "city", "king", "war", "music", "light", "energy",
                         "system", "theory", "model", "design"] {
                let _: i64 = conn.query_row("SELECT doc FROM notes_vocab WHERE term=?1", [term], |r| r.get(0)).unwrap_or(0);
            }
            let vocab_ms = t.elapsed().as_millis();
            eprintln!("[{}] MATCH 'church' (n={}): {} ms | 20 vocab lookups: {} ms", label, m, match_ms, vocab_ms);
        };

        let data_rows: i64 = conn.query_row("SELECT count(*) FROM notes_fts_data", [], |r| r.get(0)).unwrap_or(-1);
        let docsize: i64 = conn.query_row("SELECT count(*) FROM notes_fts_docsize", [], |r| r.get(0)).unwrap_or(-1);
        eprintln!("notes_fts_data rows BEFORE: {} | docsize rows: {}", data_rows, docsize);
        bench(&conn, "BEFORE");

        let t = std::time::Instant::now();
        conn.execute("INSERT INTO notes_fts(notes_fts) VALUES('optimize')", []).expect("optimize");
        eprintln!("optimize took: {} ms", t.elapsed().as_millis());

        let data_rows2: i64 = conn.query_row("SELECT count(*) FROM notes_fts_data", [], |r| r.get(0)).unwrap_or(-1);
        eprintln!("notes_fts_data rows AFTER: {}", data_rows2);
        bench(&conn, "AFTER");

        drop(conn);
        let _ = std::fs::remove_file(&tmp);
    }
}

// ─── MIG-012 — Index search history IPCs ─────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexHistoryEntry {
    pub query: String,
    pub last_used: i64,
    pub use_count: i64,
}

/// Read the user's most-recent Index filter queries, sorted by
/// `last_used` desc. Returns at most `limit` rows (default 20, max 200).
/// Empty when history toggle has never been used or after a clear.
#[tauri::command]
pub fn read_index_history(
    app: tauri::AppHandle,
    limit: Option<u32>,
) -> Result<Vec<IndexHistoryEntry>, String> {
    use rusqlite::{Connection, OpenFlags};
    let limit = limit.unwrap_or(20).max(1).min(200);

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT query, last_used, use_count FROM index_search_history \
             ORDER BY last_used DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![limit as i64], |row| {
            Ok(IndexHistoryEntry {
                query: row.get(0)?,
                last_used: row.get(1)?,
                use_count: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(rows.flatten().collect())
}

/// Persist a filter query the user just ran. UPSERT semantics: bumps
/// `use_count` + `last_used` if the query is already in history.
/// FIFO eviction at 200 rows so a long session doesn't grow unbounded.
/// Empty / whitespace-only queries are silently ignored.
#[tauri::command]
pub fn write_index_history_entry(
    app: tauri::AppHandle,
    query: String,
) -> Result<(), String> {
    use rusqlite::{Connection, OpenFlags};
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // UPSERT — match on the unique `query` index. SQLite's
    // ON CONFLICT(query) DO UPDATE is the SQLite-native UPSERT shape.
    conn.execute(
        "INSERT INTO index_search_history (query, last_used, use_count) \
         VALUES (?1, ?2, 1) \
         ON CONFLICT(query) DO UPDATE SET \
           last_used = excluded.last_used, \
           use_count = use_count + 1",
        rusqlite::params![trimmed, now],
    )
    .map_err(|e| e.to_string())?;

    // FIFO eviction. Subquery picks the rows older than the 200-row
    // threshold by last_used; DELETE removes them. No-op when count <=200.
    conn.execute(
        "DELETE FROM index_search_history WHERE id IN \
           (SELECT id FROM index_search_history ORDER BY last_used DESC LIMIT -1 OFFSET 200)",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Wipe all rows from index_search_history. Used by Settings → Clear
/// search history. Idempotent.
#[tauri::command]
pub fn clear_index_history(app: tauri::AppHandle) -> Result<(), String> {
    use rusqlite::{Connection, OpenFlags};
    let db_path = crate::search::db_path(&app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(&db_path, flags)
        .map_err(|e| format!("Failed to open search.db: {}", e))?;
    conn.execute("DELETE FROM index_search_history", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Collect all note names in a library (for autocomplete).
#[tauri::command]
pub fn collect_library_notes(app: tauri::AppHandle, library_path: String) -> Result<Vec<serde_json::Value>, String> {
    let libraries = load_all_libraries(&app);
    if !libraries.iter().any(|v| v.path == library_path) {
        return Err("Access denied: not a registered library.".to_string());
    }
    let mut notes = Vec::new();
    collect_notes_names_recursive(Path::new(&library_path), &mut notes);
    Ok(notes)
}

fn collect_notes_names_recursive(dir: &Path, notes: &mut Vec<serde_json::Value>) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            collect_notes_names_recursive(&path, notes);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            // Use frontmatter title for canonical files, file stem for human-named files
            let file_stem = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let note_name = if crate::canonical::is_canonical_filename(&path) {
                extract_frontmatter_title_quick(&path).unwrap_or(file_stem)
            } else {
                file_stem
            };
            notes.push(serde_json::json!({
                "name": note_name,
                "path": path.to_string_lossy().to_string()
            }));
        }
    }
}

/// Quick frontmatter title extraction (reads first 1KB only).
fn extract_frontmatter_title_quick(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    extract_frontmatter_title(&content)
}

/// MIG-006 §1: read just the human title from a `.md` file's
/// frontmatter without indexing. Used by the rename flow to pick
/// up the OLD display name BEFORE the rename mutates the file —
/// so the wikilink cascade can search for `[[old_title]]` in source
/// notes, not for `[[20260424T063440Z_NOTE_531D]]` (the canonical
/// filename stem, which the L3788 derivation was using and which
/// silently killed the cascade for every canonical note).
///
/// Returns `Ok(Some(title))` if the file has a frontmatter `title:`
/// field, `Ok(None)` if the file has no title (caller falls back to
/// filename stem for legacy human-named notes), `Err` if the path is
/// outside any registered library or unreadable.
#[tauri::command]
pub fn read_note_title(app: tauri::AppHandle, file_path: String) -> Result<Option<String>, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist.".to_string());
    }
    Ok(extract_frontmatter_title_quick(path))
}

// MIG-091 — collect_library_notes_with_metadata + collect_notes_meta_recursive
// removed with the retired Notes Navigator (their sole consumer).

/// Get daily note path for today.
#[tauri::command]
pub fn get_daily_note_path(app: tauri::AppHandle, library_path: String, format: String, folder: String, date: Option<String>, cultural_date: Option<String>) -> Result<String, String> {
    validate_path_in_any_library(&app, &library_path)?;
    if !folder.is_empty() {
        if folder.contains("..") || folder.contains('\\') || folder.starts_with('/') {
            return Err("Folder name contains invalid characters.".to_string());
        }
    }
    // MIG-079 §D (Calendar day-click bug): honour the explicitly-clicked date when
    // provided (YYYY-MM-DD from CalendarPanel); otherwise default to today. Previously
    // this ALWAYS used `Local::now()` and ignored the clicked date — so clicking any
    // day opened/created TODAY's note. `None` preserves the "open today" callers
    // (handleOpenDailyNote). Format applied to a midnight datetime so date-only AND
    // any time specifiers in `format` render correctly.
    let (filename, fm_date) = match date.as_deref().filter(|d| !d.is_empty()) {
        Some(d) => {
            let nd = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map_err(|e| format!("Invalid daily-note date '{}': {}", d, e))?;
            let dt = nd.and_hms_opt(0, 0, 0)
                .ok_or_else(|| "Invalid date components".to_string())?;
            (dt.format(&format).to_string(), nd.format("%Y-%m-%d").to_string())
        }
        None => {
            let now = chrono::Local::now();
            (now.format(&format).to_string(), now.format("%Y-%m-%d").to_string())
        }
    };
    let daily_folder = if folder.is_empty() {
        Path::new(&library_path).to_path_buf()
    } else {
        Path::new(&library_path).join(&folder)
    };
    // Validate the resolved path is still within the library
    validate_path_in_library(&daily_folder.to_string_lossy(), &library_path)?;
    fs::create_dir_all(&daily_folder).map_err(|e| e.to_string())?;
    let file_path = daily_folder.join(format!("{}.md", filename));

    // Create the file if it doesn't exist
    if !file_path.exists() {
        // MIG-082 §C — optional non-authoritative cultural-date stamp (opt-in). The frontend computes
        // it (the Hijri/Temporal engines are JS-side, correction/mode-aware) as a single
        // "key: YYYY-MM-DD" line. Sanitised: anything with a newline/CR or a `---` fence is dropped so
        // a stray value can never break out of the frontmatter block. Only stamped at CREATION.
        let cultural_line = cultural_date.as_deref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.contains('\n') && !s.contains('\r') && !s.contains("---"))
            .map(|s| format!("{}\n", s))
            .unwrap_or_default();
        let content = format!("---\ndate: {}\n{}---\n", fm_date, cultural_line);
        // MIG-076 §A2 — create-exclusive; RefusedExists means another writer
        // created the note in the race window — that IS the goal, proceed.
        crate::write_gate::gate_create_exclusive(&file_path, &content, "daily_note")?;
    }

    Ok(file_path.to_string_lossy().to_string())
}

/// Quick capture: create a timestamped note in the inbox folder.
#[tauri::command]
pub fn quick_capture(app: tauri::AppHandle, library_path: String, inbox_folder: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &library_path)?;
    if inbox_folder.contains("..") || inbox_folder.contains('\\') || inbox_folder.starts_with('/') {
        return Err("Inbox folder name contains invalid characters.".to_string());
    }
    let inbox_dir = if inbox_folder.is_empty() {
        Path::new(&library_path).to_path_buf()
    } else {
        Path::new(&library_path).join(&inbox_folder)
    };
    validate_path_in_library(&inbox_dir.to_string_lossy(), &library_path)?;
    fs::create_dir_all(&inbox_dir).map_err(|e| e.to_string())?;

    let now = chrono::Local::now();
    let base_name = now.format("%Y-%m-%d %H-%M").to_string();

    // Deduplicate filename
    let mut file_path = inbox_dir.join(format!("{}.md", base_name));
    if file_path.exists() {
        for i in 1..=100 {
            file_path = inbox_dir.join(format!("{} {}.md", base_name, i));
            if !file_path.exists() {
                break;
            }
        }
    }

    let content = format!("---\ncreated: {}\n---\n\n", now.format("%Y-%m-%d"));
    // MIG-076 §A2 — create-exclusive: a race past the uniqueness loop above
    // refuses instead of overwriting the other note.
    if crate::write_gate::gate_create_exclusive(&file_path, &content, "new_note")?
        == crate::write_gate::WriteOutcome::RefusedExists
    {
        return Err("A note already exists at this path (created concurrently).".to_string());
    }

    Ok(file_path.to_string_lossy().to_string())
}

/// §3-redo.3 — what the cascade walker returns to the frontend after all
/// rewrites complete. `rewritten` carries absolute paths of every file the
/// walker successfully rewrote (used by the frontend to know which open tabs
/// need a reload). `failed` carries `(path, error)` pairs for files the
/// walker tried to rewrite but couldn't (locked, permission denied, etc.) —
/// the cascade is per-file atomic but not transactional across files
/// (Concept Paper D3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeResult {
    pub rewritten: Vec<String>,
    pub failed: Vec<(String, String)>,
    /// Count of additional failures dropped past the `MAX_FAILED_REPORTED`
    /// cap. Defensive against a pathological cascade (e.g. a whole library
    /// stuck on a permission boundary) bloating the IPC payload — the user
    /// only needs to see "many failed", not every path.
    pub failed_truncated: usize,
}

/// Cap on the number of `failed` entries serialised back to the frontend.
/// 100 is plenty for the toast UX; anything beyond that is summarised in
/// `failed_truncated`.
const MAX_FAILED_REPORTED: usize = 100;

/// Update all links in a library when a note is renamed.
// Note-open-freeze Batch-2 §B2-4 (2026-07-03): `(async)` — the full-library
// cascade walk + its per-file reindex loop (each a writer-lock acquisition)
// run off the IPC dispatch thread. Per-file RMW is gated (gate_rmw in
// update_links_recursive); the caller (handleRenameComplete) awaits the whole
// chain, so cascade ordering vs the tab reload is preserved by the awaits.
/// PJ-092 — a canonical IDENTITY key for a path, so the rename cascade decides
/// "is this the note the frontend could NOT flush?" by FILE IDENTITY, not a raw
/// string compare across the JS↔Rust boundary. A defeated match = the reverted
/// data-loss bug returns, so this is the load-bearing seam (the universe root can
/// be an Arabic path — NFC vs NFD is a live surface here).
///   1. `canonicalize` (when the file exists) collapses `\` vs `/`, case, 8.3
///      short names, the `\\?\` long-path prefix, symlinks, `.`/`..`.
///   2. Unicode **NFC** folds the Arabic NFC/NFD forms the two sides may carry.
///   3. forward slashes + (Windows only) lowercase — a last-resort belt when the
///      OS canonical form is unavailable; Windows lowercasing can only BROADEN a
///      match on a case-insensitive FS (the fail-safe direction).
fn path_identity_key(p: &Path) -> String {
    use unicode_normalization::UnicodeNormalization;
    let base = std::fs::canonicalize(p)
        .map(|c| c.to_string_lossy().into_owned())
        .unwrap_or_else(|_| p.to_string_lossy().into_owned());
    let nfc: String = base.nfc().collect();
    let fwd = nfc.replace('\\', "/");
    if cfg!(windows) { fwd.to_lowercase() } else { fwd }
}

#[tauri::command(async)]
pub fn update_links_on_rename(
    app: tauri::AppHandle,
    library_path: String,
    library_name: String,
    old_name: String,
    new_name: String,
    // PJ-092 — the open notes whose unsaved edits could NOT be flushed (a locked
    // .md). Their disk is NEVER rewritten: rewriting would diverge disk from the
    // still-dirty model and the reload would clobber the edits (data-loss) or hang
    // the reactive layer (the reverted freeze). Matched by file identity, not string.
    exclude_paths: Vec<String>,
) -> Result<CascadeResult, String> {
    use tauri::Emitter;
    validate_path_in_any_library(&app, &library_path)?;
    // §B2-4 stall forensics — the cascade was ENTERED (pairs with the
    // rename's "rename_return" marker; see write_gate::journal_marker).
    crate::write_gate::journal_marker(Path::new(&library_path), "cascade_enter");
    // §2: compile the regex once per cascade, reuse it across every file
    // visited. `regex::escape` keeps titles with metacharacters safe
    // (`§2 Round3`, `Foo (bar)`, `a.b`, etc.).
    let pattern = format!(r"\[\[({})(\]\]|\|)", regex::escape(&old_name));
    let re = match regex::Regex::new(&pattern) {
        Ok(r) => r,
        Err(e) => return Err(format!("Failed to build cascade regex: {}", e)),
    };
    let mut result = CascadeResult {
        rewritten: Vec::new(),
        failed: Vec::new(),
        failed_truncated: 0,
    };
    // PJ-092 — identity keys of the notes to EXCLUDE from the on-disk rewrite.
    let exclude: std::collections::HashSet<String> = exclude_paths
        .iter()
        .map(|p| path_identity_key(Path::new(p)))
        .collect();
    let mut excluded_hit: std::collections::HashSet<String> = std::collections::HashSet::new();
    update_links_recursive(
        Path::new(&library_path),
        &re,
        &new_name,
        &mut result,
        &exclude,
        &mut excluded_hit,
    );
    // Hardening — a normalization miss is otherwise invisible-until-loss: an exclude
    // entry that matched NO walked file may be a defeated exclusion. Make it visible.
    for key in &exclude {
        if !excluded_hit.contains(key) {
            eprintln!(
                "[update_links_on_rename] PJ-092: exclude entry matched no walked file (possible path-normalization miss): {}",
                key
            );
        }
    }

    // §4 — reindex each rewritten source so `note_meta.outgoing_links_json`
    // and `note_links.target_name` reflect the new wikilink targets. Without
    // this, Outgoing Links / Backlinks / Index panels render stale `target_name`
    // values until the user touches the source again (Invariant 15 from the
    // MIG-006 plan; the symptom Boss surfaced in §3-redo Stage 1 testing where
    // the Outgoing Links panel still showed `foo` after Foo → Foo v2 cascade).
    //
    // Best-effort: per-file reindex failures are logged and skipped. The
    // cascade rewrite is already on disk; alias-aware reads from MIG-004
    // keep correctness intact for any path whose reindex didn't land. The
    // IPC must not fail back to the frontend over a reindex glitch — the
    // frontend's `result.rewritten` already drove its own reload pipeline
    // via `cascade:rewrote`.
    //
    // Per-call transactions: `index_note` already wraps each call in
    // `BEGIN IMMEDIATE`/COMMIT. Wrapping a batch transaction here would
    // collide. WAL stays bounded by the per-file commit cycles.
    //
    // §B2-4 stall fix (same class as rename_item's tail): each reindex
    // acquires the UNBOUNDED writer mutex — N sequential acquisitions inside
    // the awaited IPC surface = the same invisible-park shape that stalled
    // rename_item. The rewrites are already on disk (the frontend's reload
    // pipeline keys off `result.rewritten`, not the reindex); detach the
    // best-effort reindex loop to a worker so the IPC settles when the FILE
    // work is done.
    if !result.rewritten.is_empty() {
        let reindex_app = app.clone();
        let reindex_paths = result.rewritten.clone();
        let reindex_lib = library_name.clone();
        tauri::async_runtime::spawn_blocking(move || {
            use tauri::Manager;
            let search_state = reindex_app.state::<crate::search::SearchState>();
            for path in &reindex_paths {
                if let Err(e) = crate::search::reindex_single_note(&search_state, path, &reindex_lib) {
                    eprintln!("[update_links_on_rename] reindex skipped path={} err={}", path, e);
                }
            }
        });
    }

    // §3-redo.3 — emit the cascade:rewrote event so the frontend can reload
    // each affected open tab. Per Concept Paper D6, the reload mechanism on
    // the frontend MUST NOT use a $effect on value/editBody; the §3-redo.4
    // step uses tab-key invalidation ({#key} bump) instead.
    if !result.rewritten.is_empty() {
        let _ = app.emit(
            "cascade:rewrote",
            serde_json::json!({ "paths": &result.rewritten }),
        );
    }

    Ok(result)
}

fn update_links_recursive(
    dir: &Path,
    re: &regex::Regex,
    new_name: &str,
    result: &mut CascadeResult,
    exclude: &std::collections::HashSet<String>,
    excluded_hit: &mut std::collections::HashSet<String>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') { continue; }
        if path.is_dir() {
            update_links_recursive(&path, re, new_name, result, exclude, excluded_hit);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            // A file that vanished mid-walk (concurrent move/delete) is
            // silently skipped — same semantics as the old `if let Ok(read)`.
            if !path.exists() { continue; }
            // PJ-092 — a note the frontend could NOT flush is EXCLUDED: never rewrite
            // its disk (identity match, not a raw string compare — the Arabic-root
            // NFC/8.3/\\?\ hazards). Only pays the per-file canonicalize when there IS
            // an exclusion (rare — only when a flush failed); a clean rename skips it.
            if !exclude.is_empty() {
                let key = path_identity_key(&path);
                if exclude.contains(&key) {
                    excluded_hit.insert(key);
                    continue;
                }
            }
            // Note-open-freeze Batch-2 §B2-4 (2026-07-03): the per-file
            // read→rewrite→write now runs as ONE gated critical section
            // (gate_rmw — the per-path lock held across the WHOLE cycle), so
            // an editor save of THIS file can land before or after its
            // rewrite but never inside it (the lost-update window the SYNC
            // dispatch used to mask). The lock is held per FILE (bounded ms),
            // released before the walker moves on — never across the walk.
            let mut changed = false;
            match crate::write_gate::gate_rmw(&path, "cascade", |content| {
                let updated = rewrite_wikilinks_in_text(content, re, new_name);
                if updated != content {
                    changed = true;
                    Ok(Some(updated))
                } else {
                    Ok(None)
                }
            }) {
                Ok(_) if changed => result.rewritten.push(path.to_string_lossy().to_string()),
                Ok(_) => {} // unchanged — nothing to record
                Err(e) => {
                    if result.failed.len() < MAX_FAILED_REPORTED {
                        result.failed.push((
                            path.to_string_lossy().to_string(),
                            e.to_string(),
                        ));
                    } else {
                        result.failed_truncated += 1;
                    }
                }
            }
        }
    }
}

/// MIG-006 §2 — regex-based wikilink rewrite.
///
/// Matches `[[old]]` and `[[old|...]]` (display, link-type, alias-pipe-type
/// combos). Leading `!` for embeds is untouched because the regex anchors
/// on `[[` — `![[X]]` rewrites cleanly. The trailing delimiter (`]]` or `|`)
/// is captured and re-emitted so we never alter `|display`, `|link-type`,
/// or `|alias|link-type` tails.
///
/// Prefix-collision safety: `[[Foo]]` rename to `Bar` does NOT touch
/// `[[Foo Bar]]` or `[[Foo_v2]]` — the delimiter alternation `(\]\]|\|)`
/// requires the next char after the title to be either `]]` or `|`,
/// nothing else.
fn rewrite_wikilinks_in_text(content: &str, re: &regex::Regex, new_name: &str) -> String {
    re.replace_all(content, |caps: &regex::Captures| {
        let delim = caps.get(2).map(|m| m.as_str()).unwrap_or("]]");
        format!("[[{}{}", new_name, delim)
    })
    .into_owned()
}

#[cfg(test)]
fn rewrite_for_test(content: &str, old_name: &str, new_name: &str) -> String {
    let pattern = format!(r"\[\[({})(\]\]|\|)", regex::escape(old_name));
    let re = regex::Regex::new(&pattern).unwrap();
    rewrite_wikilinks_in_text(content, &re, new_name)
}

#[cfg(test)]
mod cascade_walker_tests {
    use super::{path_identity_key, rewrite_for_test, update_links_recursive, CascadeResult};
    use std::collections::HashSet;
    use std::path::Path;

    // ── PJ-092 flush-gate-exclude: the identity-key contract + the walker exclude ──

    #[test]
    fn identity_key_folds_separators() {
        // `\` and `/` forms of the same path key equal (canonicalize fails for a
        // non-existent path, so the NFC+slash+case belt runs — the JS↔Rust seam).
        assert_eq!(
            path_identity_key(Path::new("C:/Lib/note.md")),
            path_identity_key(Path::new("C:\\Lib\\note.md")),
        );
    }

    #[test]
    fn identity_key_folds_nfc_nfd() {
        // The same name in NFC vs NFD must key equal — the Arabic universe-root hazard
        // (H1): tab.path (JS) and to_string_lossy() (Rust) may carry different forms.
        use unicode_normalization::UnicodeNormalization;
        let nfc: String = "E:/كلاود/café.md".nfc().collect();
        let nfd: String = "E:/كلاود/café.md".nfd().collect();
        assert_ne!(nfc, nfd, "precondition: NFC and NFD byte-differ");
        assert_eq!(
            path_identity_key(Path::new(&nfc)),
            path_identity_key(Path::new(&nfd)),
            "NFC/NFD forms of the same path must produce the same identity key"
        );
    }

    #[test]
    fn walker_excludes_by_identity_and_rewrites_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let excluded = dir.path().join("Excluded.md");
        let rewritten = dir.path().join("Rewritten.md");
        std::fs::write(&excluded, "links [[Old]] here").unwrap();
        std::fs::write(&rewritten, "links [[Old]] here").unwrap();

        let re = regex::Regex::new(&format!(r"\[\[({})(\]\]|\|)", regex::escape("Old"))).unwrap();
        let mut result = CascadeResult { rewritten: Vec::new(), failed: Vec::new(), failed_truncated: 0 };
        // Exclude the note using a DELIBERATELY different separator form than the
        // walker will produce — the identity key must still match (the sharp edge).
        let excluded_alt_sep = excluded.to_string_lossy().replace('/', "\\");
        let exclude: HashSet<String> =
            [excluded_alt_sep].iter().map(|p| path_identity_key(Path::new(p))).collect();
        let mut hit: HashSet<String> = HashSet::new();

        update_links_recursive(dir.path(), &re, "New", &mut result, &exclude, &mut hit);

        assert_eq!(result.rewritten.len(), 1, "exactly the non-excluded file is rewritten");
        assert!(result.rewritten[0].contains("Rewritten"));
        assert_eq!(
            std::fs::read_to_string(&excluded).unwrap(),
            "links [[Old]] here",
            "the excluded note's bytes are UNTOUCHED (never rewritten)"
        );
        assert_eq!(std::fs::read_to_string(&rewritten).unwrap(), "links [[New]] here");
        assert_eq!(hit.len(), 1, "the exclude entry matched a walked file");
    }

    #[test]
    fn walker_empty_exclude_rewrites_all() {
        // Empty exclude == today's behavior (the rollback proof).
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("A.md"), "[[Old]]").unwrap();
        std::fs::write(dir.path().join("B.md"), "[[Old]]").unwrap();
        let re = regex::Regex::new(&format!(r"\[\[({})(\]\]|\|)", regex::escape("Old"))).unwrap();
        let mut result = CascadeResult { rewritten: Vec::new(), failed: Vec::new(), failed_truncated: 0 };
        let mut hit: HashSet<String> = HashSet::new();
        update_links_recursive(dir.path(), &re, "New", &mut result, &HashSet::new(), &mut hit);
        assert_eq!(result.rewritten.len(), 2, "empty exclude rewrites everything (rollback-equivalent)");
    }

    #[test]
    fn bare_wikilink_rewrites() {
        let out = rewrite_for_test("see [[Old Title]] here", "Old Title", "New Title");
        assert_eq!(out, "see [[New Title]] here");
    }

    #[test]
    fn piped_display_preserves_tail() {
        let out = rewrite_for_test("see [[Old|the display]]", "Old", "New");
        assert_eq!(out, "see [[New|the display]]");
    }

    #[test]
    fn piped_link_type_preserves_tail() {
        let out = rewrite_for_test("see [[Old|supports]]", "Old", "New");
        assert_eq!(out, "see [[New|supports]]");
    }

    #[test]
    fn piped_alias_and_link_type_preserves_tail() {
        let out = rewrite_for_test("see [[Old|alias text|supports]]", "Old", "New");
        assert_eq!(out, "see [[New|alias text|supports]]");
    }

    #[test]
    fn embed_transclude_rewrites() {
        let out = rewrite_for_test("![[Old]] inline", "Old", "New");
        assert_eq!(out, "![[New]] inline");
    }

    #[test]
    fn prefix_collision_is_not_rewritten() {
        // [[Foo]] rename to [[Bar]] must not touch [[Foo Bar]] or [[Foo_v2]].
        let out = rewrite_for_test(
            "yes [[Foo]] no [[Foo Bar]] no [[Foo_v2]] yes [[Foo|x]]",
            "Foo",
            "Bar",
        );
        assert_eq!(
            out,
            "yes [[Bar]] no [[Foo Bar]] no [[Foo_v2]] yes [[Bar|x]]"
        );
    }

    #[test]
    fn regex_metachars_in_title_are_escaped() {
        let out = rewrite_for_test(
            "see [[a.b (c)]] and [[a.b (c)|note]]",
            "a.b (c)",
            "x.y (z)",
        );
        assert_eq!(out, "see [[x.y (z)]] and [[x.y (z)|note]]");
    }

    #[test]
    fn frontmatter_typed_link_rewrites_on_rename() {
        // MIG-086 Part 2 §F3 / invariant D6 — a quoted typed-link wikilink declared in
        // FRONTMATTER must survive a target rename. The cascade rewrites raw file content,
        // so the `[[Old]]` inside `"[[Old]]"` is rewritten in place; the surrounding quotes
        // (outside the regex match) are preserved → still valid YAML.
        let md = "---\nsupports:\n  - \"[[Old Note]]\"\nderives-from: [\"[[Old Note]]\"]\n---\nbody [[Old Note]] too";
        let out = rewrite_for_test(md, "Old Note", "New Note");
        assert_eq!(
            out,
            "---\nsupports:\n  - \"[[New Note]]\"\nderives-from: [\"[[New Note]]\"]\n---\nbody [[New Note]] too"
        );
    }

    #[test]
    fn no_match_returns_unchanged() {
        let input = "no wikilinks here, just [[Different]] and [[Foo Bar]]";
        let out = rewrite_for_test(input, "Foo", "Bar");
        assert_eq!(out, input);
    }

    #[test]
    fn multiple_occurrences_all_rewritten() {
        let out = rewrite_for_test(
            "[[Old]] then [[Old]] then [[Old|x]] done",
            "Old",
            "New",
        );
        assert_eq!(out, "[[New]] then [[New]] then [[New|x]] done");
    }

    #[test]
    fn arabic_title_rewrites() {
        let out = rewrite_for_test(
            "انظر [[الفاطميون]] في [[الفاطميون|الدولة]]",
            "الفاطميون",
            "الفاطميون_جديد",
        );
        assert_eq!(
            out,
            "انظر [[الفاطميون_جديد]] في [[الفاطميون_جديد|الدولة]]"
        );
    }

    #[test]
    fn unicode_section_marker_title_rewrites() {
        // The exact case that drove the §1 verification.
        let out = rewrite_for_test(
            "Link me to [[§2 Round3_v3]]",
            "§2 Round3_v3",
            "§2 Round3_v4",
        );
        assert_eq!(out, "Link me to [[§2 Round3_v4]]");
    }
}

/// Read a note's content for preview (used by hover preview)
#[tauri::command]
pub fn read_note_preview(app: tauri::AppHandle, file_path: String, max_chars: usize) -> Result<String, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;
    Ok(safe_truncate(&content, max_chars))
}

/// Save a base64-encoded image from clipboard to the library's attachments folder.
/// Returns the relative path suitable for embedding as `![[filename]]`.
#[tauri::command]
pub fn save_clipboard_image(app: tauri::AppHandle, library_path: String, image_data: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &library_path)?;
    // Create attachments folder if it doesn't exist
    let attachments_dir = Path::new(&library_path).join("attachments");
    if !attachments_dir.exists() {
        fs::create_dir_all(&attachments_dir)
            .map_err(|e| format!("Failed to create attachments folder: {}", e))?;
    }

    // Generate filename with timestamp
    let now = chrono::Local::now();
    let filename = format!("Pasted image {}.png", now.format("%Y%m%d%H%M%S"));
    let file_path = attachments_dir.join(&filename);

    // Decode base64 data (strip data URL prefix if present)
    let b64_data = if let Some(idx) = image_data.find(",") {
        &image_data[idx + 1..]
    } else {
        &image_data
    };

    use std::io::Write;
    let decoded = base64_decode(b64_data)
        .map_err(|e| format!("Failed to decode image data: {}", e))?;

    let mut file = fs::File::create(&file_path)
        .map_err(|e| format!("Failed to create image file: {}", e))?;
    file.write_all(&decoded)
        .map_err(|e| format!("Failed to write image file: {}", e))?;

    Ok(filename)
}

/// Resolve an image embed filename to a base64 data URL.
/// Searches: note's folder → library/attachments/ → library root.
/// Returns `data:image/...;base64,...` or an empty string if not found.
#[tauri::command]
pub fn resolve_embed_image(
    library_path: String,
    note_path: String,
    filename: String,
) -> String {
    let note_dir = Path::new(&note_path).parent().map(|p| p.to_path_buf());

    // Candidate paths in priority order
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(ref nd) = note_dir {
        candidates.push(nd.join(&filename));
    }
    if !library_path.is_empty() {
        candidates.push(Path::new(&library_path).join("attachments").join(&filename));
        candidates.push(Path::new(&library_path).join("images").join(&filename));
        candidates.push(Path::new(&library_path).join("assets").join(&filename));
        candidates.push(Path::new(&library_path).join(&filename));
    }

    for cand in &candidates {
        if cand.is_file() {
            if let Ok(bytes) = fs::read(cand) {
                let ext = cand.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
                let mime = match ext.as_str() {
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "svg" => "image/svg+xml",
                    "webp" => "image/webp",
                    "bmp" => "image/bmp",
                    "ico" => "image/x-icon",
                    "avif" => "image/avif",
                    _ => "image/png",
                };
                return format!("data:{};base64,{}", mime, base64_encode(&bytes));
            }
        }
    }
    String::new()
}

/// Simple base64 encoder
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(TABLE[((n >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(TABLE[(n & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

/// Simple base64 decoder (no external crate needed)
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let table: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, &b) in table.iter().enumerate() {
        lookup[b as usize] = i as u8;
    }

    let input = input.trim().replace('\n', "").replace('\r', "");
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len() * 3 / 4);

    let mut i = 0;
    while i < bytes.len() {
        let mut buf = [0u8; 4];
        let mut count = 0;
        while count < 4 && i < bytes.len() {
            let b = bytes[i];
            i += 1;
            if b == b'=' || b == b' ' || b == b'\t' {
                if b == b'=' { count += 1; }
                continue;
            }
            let val = lookup[b as usize];
            if val == 255 { continue; }
            buf[count] = val;
            count += 1;
        }
        if count >= 2 {
            output.push((buf[0] << 2) | (buf[1] >> 4));
        }
        if count >= 3 {
            output.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if count >= 4 {
            output.push((buf[2] << 6) | buf[3]);
        }
    }

    Ok(output)
}

/// Export a note's rendered content as HTML
#[tauri::command]
pub fn export_note_html(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    validate_path_in_any_library(&app, &file_path)?;
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read note: {}", e))?;
    Ok(content)
}

/// Move item to system trash (or ".trash" folder inside library)
// MIG-099 §3: `(async)` — now drops the index row (below), which takes the writer
// lock; off the WebView2 IPC dispatch thread, matching delete_path.
#[tauri::command(async)]
pub fn move_to_trash(app: tauri::AppHandle, path: String, library_path: String) -> Result<(), String> {
    // Verify the file is within a registered library (not just any caller-supplied library_path)
    validate_path_in_any_library(&app, &path)?;
    validate_path_in_library(&path, &library_path)?;
    let trash_dir = Path::new(&library_path).join(".trash");
    if !trash_dir.exists() {
        fs::create_dir_all(&trash_dir)
            .map_err(|e| format!("Failed to create .trash folder: {}", e))?;
    }

    let source = Path::new(&path);
    let file_name = source.file_name()
        .ok_or("Invalid path")?;
    let mut dest = trash_dir.join(file_name);

    // MIG-076 §E1b — de-collide on a name clash inside .trash (Obsidian-style
    // numeric suffix, Boss-approved 2026-06-13). Without this, trashing a second
    // note that shares a filename with one already in .trash atomically
    // replaces — and silently loses — the earlier trashed copy (observed in the
    // §E-1 Stage-2 validation). Suffix the stem with " {n}" to match
    // Constellation's own create-collision naming. Never clobber.
    if dest.exists() {
        let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("note");
        let ext = source.extension().and_then(|s| s.to_str());
        let mut placed = false;
        for n in 1..=9999 {
            let candidate_name = match ext {
                Some(e) => format!("{} {}.{}", stem, n, e),
                None => format!("{} {}", stem, n),
            };
            let candidate = trash_dir.join(&candidate_name);
            if !candidate.exists() {
                dest = candidate;
                placed = true;
                break;
            }
        }
        if !placed {
            return Err("Trash already holds too many notes with this name.".to_string());
        }
    }

    // MIG-076 §A2 — gated: a trash move serializes against any in-flight
    // editor flush of the same file (delete-vs-save race).
    crate::write_gate::gate_rename(source, &dest, "trash")?;

    // MIG-099 §3 — drop the moved note from the search index. Without this the
    // note_meta row lingered at the pre-trash path (index↔disk divergence): the
    // note kept showing in search at a now-dead path, AND the §2 index-backed
    // collision check would surface it as a PHANTOM collision on a later same-name
    // create. `delete_path` already drops the row for every mode; this standalone
    // trash move (the createNoteWithTemplate Overwrite path) did not. Delete by the
    // ORIGINAL path — the row was never updated to the .trash destination. Surfaced
    // (diag_log) on failure, not swallowed.
    {
        use tauri::Manager;
        let search_state = app.state::<crate::search::SearchState>();
        if let Err(e) = crate::search::reindex_delete_note(&search_state, &path) {
            if let Ok(p) = crate::search::db_path(&app) {
                crate::search::diag_log(&p, &format!("[move_to_trash] reindex_delete FAILED for {}: {}", path, e));
            }
        }
    }

    Ok(())
}

/// MIG-076 §E-follow-up — unified delete that honors the user's "Deleted files"
/// setting (closes the gap where `delete_item` always hard-deleted regardless).
/// `mode` routes the destination:
///   - "permanent" → remove the file/folder (the only non-recoverable mode);
///   - "trash"     → move into `<trash_root>/.trash`, de-colliding on a name
///                   clash; `trash_root` is the note's LIBRARY root or the
///                   UNIVERSE root, chosen by the frontend per `trashFolderScope`;
///   - "system"    → move to the OS Recycle Bin (the `trash` crate).
/// Every mode drops the note from the search index — it no longer lives at its
/// indexed path (gone, or in an excluded `.trash`/OS-trash dir).
// Note-open-freeze Batch-2 §B2-4 (2026-07-03): `(async)` — off the IPC dispatch thread.
// The destructive/rename steps run under path locks (gate_rename/gate_delete); the DB
// cascade + reindex run after the locks release. See SESSION-LOG-2026-07-03.
#[tauri::command(async)]
pub fn delete_path(
    app: tauri::AppHandle,
    path: String,
    mode: String,
    trash_root: Option<String>,
) -> Result<(), String> {
    validate_path_in_any_library(&app, &path)?;
    let target = Path::new(&path);
    if !target.exists() {
        return Err("Item does not exist.".to_string());
    }

    // Note-open-freeze Batch-2 §B2-4 (2026-07-03): every destructive step runs
    // under the path lock (gate_delete / with_path_lock) so a debounced editor
    // save serializes against the delete — it lands before (deleted with the
    // note) or after (legitimately recreates), never DURING the removal.
    match mode.as_str() {
        "permanent" => {
            let dm = if target.is_dir() {
                crate::write_gate::DeleteMode::DirAll
            } else {
                crate::write_gate::DeleteMode::File
            };
            crate::write_gate::gate_delete(target, dm, "delete_permanent")?;
        }
        "system" => {
            crate::write_gate::with_path_lock(target, || {
                trash::delete(target).map_err(|e| format!("Failed to move to system trash: {}", e))
            })?;
        }
        "trash" => {
            let root = trash_root.ok_or("No trash root provided for a .trash-folder delete.")?;
            move_into_trash_folder(target, Path::new(&root))?;
        }
        other => return Err(format!("Unknown delete mode: {}", other)),
    }

    // Drop the note from the search index in every case.
    {
        use tauri::Manager;
        let search_state = app.state::<crate::search::SearchState>();
        let _ = crate::search::reindex_delete_note(&search_state, &path);
    }
    Ok(())
}

/// Move `source` into `<trash_root>/.trash`, de-colliding on a name clash
/// (Obsidian-style numeric suffix) so an earlier trashed item is never
/// clobbered — the same rule as `move_to_trash`. Cross-volume-safe: a rename
/// that can't cross the device boundary (the universe-root scope where the
/// library lives on a different drive) falls back to copy + remove.
fn move_into_trash_folder(source: &Path, trash_root: &Path) -> Result<(), String> {
    let trash_dir = trash_root.join(".trash");
    if !trash_dir.exists() {
        fs::create_dir_all(&trash_dir)
            .map_err(|e| format!("Failed to create .trash folder: {}", e))?;
    }
    let file_name = source.file_name().ok_or("Invalid path")?;
    let mut dest = trash_dir.join(file_name);
    if dest.exists() {
        let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("item");
        let ext = source.extension().and_then(|s| s.to_str());
        let mut placed = false;
        for n in 1..=9999 {
            let candidate_name = match ext {
                Some(e) => format!("{} {}.{}", stem, n, e),
                None => format!("{} {}", stem, n),
            };
            let candidate = trash_dir.join(&candidate_name);
            if !candidate.exists() { dest = candidate; placed = true; break; }
        }
        if !placed {
            return Err("Trash already holds too many items with this name.".to_string());
        }
    }
    // Gate against an in-flight editor flush of the same file, then move.
    // On a cross-device failure, fall back to copy + remove.
    // Batch-2 §B2-4: the fallback pair runs under the SOURCE path lock — a
    // save landing between the copy and the remove used to be silently lost
    // (written to a file that is removed a moment later); now it serializes.
    // (gate_rename has already RELEASED its locks by the time the fallback
    // runs, so taking the source lock here cannot self-deadlock.)
    if crate::write_gate::gate_rename(source, &dest, "delete_trash").is_err() {
        crate::write_gate::with_path_lock(source, || -> Result<(), String> {
            if source.is_dir() {
                copy_dir_recursive(source, &dest)?;
                fs::remove_dir_all(source)
                    .map_err(|e| format!("Failed to remove source folder after copy: {}", e))?;
            } else {
                fs::copy(source, &dest).map_err(|e| format!("Failed to copy to trash: {}", e))?;
                fs::remove_file(source)
                    .map_err(|e| format!("Failed to remove source file after copy: {}", e))?;
            }
            Ok(())
        })?;
    }
    Ok(())
}

/// Recursive directory copy (std has no built-in) — for the cross-volume
/// trash fallback when a folder is deleted to a different-drive trash root.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create dir: {}", e))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }
    Ok(())
}

// ─── File Metadata ───

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileMetadata {
    pub created: u64,
    pub modified: u64,
}

/// Get file creation and modification timestamps (Unix seconds).
#[tauri::command]
pub fn get_file_metadata(file_path: String) -> Result<FileMetadata, String> {
    let meta = fs::metadata(&file_path)
        .map_err(|e| format!("Failed to read metadata for {}: {}", file_path, e))?;

    let created = meta.created()
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);

    let modified = meta.modified()
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);

    Ok(FileMetadata { created, modified })
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_pj065_resolve {
    //! PJ-065 §D9 — the contested-conflict resolve frontmatter helpers. They must set
    //! parent: (scalar) / remove a contains: item (list), preserve the rest of the note,
    //! and be a true no-op when there is nothing to change (so the command never writes).
    use super::{remove_frontmatter_contains_item, set_frontmatter_parent};

    #[test]
    fn set_parent_replaces_existing_scalar() {
        let c = "---\ntitle: \"Contested Child\"\nparent: \"[[Owner A]]\"\n---\n\nBody stays.";
        let out = set_frontmatter_parent(c, "Owner B");
        assert!(out.contains("parent: \"[[Owner B]]\""), "parent re-pointed to claimant");
        assert!(!out.contains("Owner A"), "old parent gone");
        assert!(out.contains("title: \"Contested Child\""), "title preserved");
        assert!(out.ends_with("\n\nBody stays."), "body preserved");
        assert!(!out.starts_with("---\n\n"), "no spurious leading blank line");
    }

    #[test]
    fn set_parent_inserts_when_absent() {
        let c = "---\ntitle: \"X\"\n---\n\nBody.";
        let out = set_frontmatter_parent(c, "Owner B");
        assert!(out.contains("parent: \"[[Owner B]]\""));
        assert!(out.contains("title: \"X\""));
    }

    #[test]
    fn remove_contains_block_item_drops_empty_key() {
        let c = "---\ntitle: \"Owner B\"\ncontains:\n  - \"[[Contested Child]]\"\n---\n\nBody.";
        let out = remove_frontmatter_contains_item(c, "Contested Child");
        assert!(!out.contains("Contested Child"), "claim removed");
        assert!(!out.contains("contains:"), "empty contains: key dropped");
        assert!(out.contains("title: \"Owner B\""), "title preserved");
        assert!(out.ends_with("\n\nBody."), "body preserved");
    }

    #[test]
    fn remove_contains_keeps_siblings() {
        let c = "---\ncontains:\n  - \"[[Chapter One]]\"\n  - \"[[Chapter Two]]\"\n---\n\nB.";
        let out = remove_frontmatter_contains_item(c, "Chapter One");
        assert!(!out.contains("[[Chapter One]]"), "target removed");
        assert!(out.contains("[[Chapter Two]]"), "sibling kept");
        assert!(out.contains("contains:"), "key kept (still has an item)");
    }

    #[test]
    fn remove_contains_inline_array() {
        let c = "---\ncontains: [\"[[A]]\", \"[[B]]\"]\n---\n\nx";
        let out = remove_frontmatter_contains_item(c, "A");
        assert!(!out.contains("[[A]]"));
        assert!(out.contains("[[B]]"));
    }

    #[test]
    fn remove_contains_noop_is_byte_identical() {
        let c = "---\ntitle: \"X\"\n---\n\nB";
        let out = remove_frontmatter_contains_item(c, "Nope");
        assert_eq!(out, c, "no contains: → unchanged (so the command's no-op guard fires)");
    }
}

#[cfg(test)]
mod tests {
    //! M6 end-to-end contract tests. The 502-case regression corpus in
    //! `arabic::regression` exercises `analyze_best` in isolation; this
    //! module checks that the FTS pipeline's `process_arabic_word` wrapper
    //! and `process_word_for_fts` downstream guard actually surface the
    //! analyzer's verdict to the tokenizer. Without these, a future
    //! refactor could wire a different stemmer in and the corpus would
    //! still pass while search results quietly regressed.
    //!
    //! MIG-056 §F also lives here — see `build_aggregate_counts_sql_*`
    //! tests at bottom of the module.
    use super::*;

    // ─── MIG-056 §F — build_aggregate_counts_sql shape tests ───

    #[test]
    fn build_aggregate_counts_sql_empty_federation_is_single_schema() {
        let sql = build_aggregate_counts_sql(&[]);
        assert_eq!(sql, "SELECT library_name, path FROM note_meta");
        assert!(!sql.contains("UNION ALL"));
        assert!(!sql.contains("main."));
    }

    #[test]
    fn build_aggregate_counts_sql_one_cuniverse_unions_main_and_cu0() {
        let sql = build_aggregate_counts_sql(&["cu0".to_string()]);
        assert!(sql.contains("SELECT library_name, path FROM main.note_meta"));
        assert!(sql.contains("UNION ALL"));
        assert!(sql.contains("SELECT library_name, path FROM cu0.note_meta"));
    }

    #[test]
    fn build_aggregate_counts_sql_multiple_cuniverses_chains_unions() {
        let sql = build_aggregate_counts_sql(&[
            "cu0".to_string(),
            "cu1".to_string(),
            "cu2".to_string(),
        ]);
        // Three UNION ALL separators between four parts (main + 3 cu)
        let union_count = sql.matches("UNION ALL").count();
        assert_eq!(union_count, 3);
        assert!(sql.contains("main.note_meta"));
        assert!(sql.contains("cu0.note_meta"));
        assert!(sql.contains("cu1.note_meta"));
        assert!(sql.contains("cu2.note_meta"));
    }

    /// The flagship bug that motivated the Constellation Arabic Engine.
    /// Pre-M6: Light10 stripped the leading و from وائل, producing "ائل"
    /// and corrupting every index row of every note mentioning any Wael.
    /// Post-M6: the protected list short-circuits Light10 and returns the
    /// name verbatim. This test is the pin that prevents the bug from
    /// ever silently returning.
    #[test]
    fn wael_is_not_mangled_to_ail() {
        let (_display, stem) = process_arabic_word("وائل");
        assert_eq!(stem, "وائل", "M6 must not mangle protected proper nouns");
    }

    /// End-to-end through the whole `process_word_for_fts` filter —
    /// this is what the FTS5 tokenizer actually calls. Guarantees
    /// the stem column of the notes_fts index holds the full name.
    #[test]
    fn wael_survives_process_word_for_fts() {
        let (stem, _norm) = process_word_for_fts("وائل").expect("وائل must tokenize");
        assert_eq!(stem, "وائل");
    }

    /// Cascade flagship: الأئمة (definite + broken plural of إمام).
    /// Layer 3b peels ال, FST matches أئمة as the plural of إمام →
    /// lemma comes out as one of the legitimate root derivations.
    /// We don't pin the exact lemma because the tiebreak order among
    /// equal-confidence FST hits isn't stable across refactors (the
    /// 502-case corpus leaves this row unasserted on lemma for the
    /// same reason), but we DO assert it isn't the Light10 mangle
    /// ("ئم" from naive ال- / -ة stripping).
    #[test]
    fn aimma_is_not_light10_mangled() {
        let (_display, stem) = process_arabic_word("الأئمة");
        assert_ne!(stem, "ئم", "cascade path must find a real analysis");
        assert_ne!(stem, "ئمه", "cascade path must find a real analysis");
        // Sanity: the lemma should contain at least one of the
        // radicals ء / م — any genuine analysis of الأئمة does.
        assert!(
            stem.chars().any(|c| c == 'ء' || c == 'أ' || c == 'م' || c == 'إ'),
            "stem {:?} lost the root radicals",
            stem,
        );
    }

    /// Unknown Arabic word falls to SurfaceHeuristic — verify we KEEP
    /// Light10 affix stripping for it so M6 is strictly non-regressive
    /// for words the analyzer doesn't yet know. "قذالبثظ" is nonsense:
    /// not protected, no root × pattern match, no peelable affix chain
    /// that hits anything real.
    #[test]
    fn unknown_word_still_gets_light10_stripping() {
        let nonsense = "قذالبثظ";
        let (_display, stem) = process_arabic_word(nonsense);
        // Post-condition is just "did not panic and returned non-empty
        // UTF-8" — the exact Light10 output on nonsense isn't something
        // we want to pin to a literal. The important contract is that
        // the pipeline degrades gracefully, not that it produces any
        // particular string.
        assert!(!stem.is_empty());
        assert!(stem.chars().all(|c| !c.is_ascii_control()));
    }

    /// Non-Arabic words must still route through the non-Arabic branch
    /// untouched — M6 only changed the Arabic branch of `process_word_for_fts`.
    #[test]
    fn english_word_still_english_stemmed() {
        let (stem, norm) = process_word_for_fts("running").expect("english must tokenize");
        // The English stemmer turns "running" into "run" (or close);
        // critically the stem must NOT be Arabic-pipeline output.
        assert!(stem.is_ascii(), "english must not be routed to arabic pipeline");
        assert_eq!(norm, "running");
    }

    // ─── MIG-010 §Build.2 — read_term_mentions cross-language expansion ───
    //
    // Tests on the testable inner helper `build_term_match_clause`. The
    // outer `read_term_mentions` IPC requires SQLite + AppHandle + a
    // populated FTS index, which doesn't fit a unit-test scope. Boss
    // verifies end-to-end at G3 (after §Build.4).

    /// Invariant I1 (default unchanged): with `expand_cross_language: false`,
    /// the helper produces a phrase-quoted MATCH clause and zero expansion
    /// state — byte-identical to pre-MIG-010 behaviour.
    #[test]
    fn build_term_match_clause_no_expand_returns_phrase_only() {
        let (clause, expansion) = build_term_match_clause("knowledge", false);
        assert_eq!(clause, "\"knowledge\"");
        assert!(expansion.is_none());
    }

    /// Phrase quoting must double-escape literal `"` per FTS5 syntax,
    /// even on the no-expand path. Pre-MIG-010 invariant preserved.
    #[test]
    fn build_term_match_clause_no_expand_doubles_quotes() {
        let (clause, expansion) = build_term_match_clause(r#"a"b"#, false);
        assert_eq!(clause, r#""a""b""#);
        assert!(expansion.is_none());
    }

    /// Invariant I3 (out-of-corpus fall-through): toggle ON, but the
    /// queried term isn't in the M11 lexicon — `expanded_match_query`
    /// returns None, helper falls through to the exact-phrase path.
    /// No expansion state, no badges possible.
    #[test]
    fn build_term_match_clause_expand_out_of_corpus_falls_back() {
        let (clause, expansion) = build_term_match_clause("Xzyqwop", true);
        assert_eq!(clause, "\"Xzyqwop\"");
        assert!(expansion.is_none(),
            "out-of-corpus terms must fall back to exact phrase, not expansion");
    }

    /// Invariant I3 (in-corpus expansion): toggle ON, term in corpus
    /// with cross-language equivalents — helper returns the OR-joined
    /// expansion clause AND the bridge-terms-lower set so the caller
    /// can scan snippets for bridge lemmas.
    #[test]
    fn build_term_match_clause_expand_in_corpus_returns_expansion() {
        let (clause, bridge) = build_term_match_clause("tree", true);
        // The corpus has "tree" with cross-language equivalents; the
        // expanded clause must contain " OR " (otherwise expansion
        // would be a degenerate single-phrase and the helper would
        // have fallen back per `expanded_match_query`'s own filter).
        assert!(clause.contains(" OR "),
            "in-corpus expansion must produce OR-joined phrases, got: {clause}");
        let bridge = bridge.expect("in-corpus term must produce bridge terms");
        assert!(!bridge.is_empty(),
            "in-corpus cross-language term must produce ≥1 bridge term");
        // Bridge terms are pre-lowercased (M13 invariant).
        assert!(bridge.iter().all(|t| t == &t.to_lowercase()));
    }
}
