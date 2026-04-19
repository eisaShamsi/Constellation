//! Boot cache — serves the UI's startup snapshot from SQLite (no filesystem walk).
//!
//! Why this module exists
//! ----------------------
//! Every PKM app that handles 10k+ notes (Obsidian, Logseq) persists parsed
//! metadata to a local cache and reads from it on boot; the filesystem is only
//! walked in the background to reconcile changes. Before this module, every
//! boot of Constellation walked the entire Universe via `scanLibraryLinks`,
//! `scanLibraryTags`, `scanLibraryIndex`, and `collect_library_notes` — each
//! re-reading every `.md` file. On a 7,600-note Universe that was ~30k full
//! file reads before the UI could do anything, which caused audible disk
//! thrashing and 4-minute "Not Responding" freezes.
//!
//! The `note_meta` table (populated incrementally by `search.rs`) already
//! stores everything the boot snapshot needs: name, path, library_name, mtime,
//! tags_json, outgoing_links_json. We just weren't reading from it on boot.
//! This module adds the boot-path reader.
//!
//! What the frontend gets
//! ----------------------
//! The boot snapshot is split into two commands so the heavy link/tag
//! payload doesn't block first paint (BOOT-BUDGET.md Criterion 2):
//!
//! * `cache_boot_snapshot_core` (awaited): { notes, is_cold }
//!     - `notes`: [{ name, path, library_name }] — populates file tree / Sight.
//!     - `is_cold`: true on first boot (cache empty) so the UI can show a
//!       one-time "Building index…" progress indicator.
//!
//! * `cache_boot_snapshot_graph` (deferred via requestIdleCallback): { links, tags }
//!     - `links`: [NoteLink] — populates Sky View graph, backlinks, Index.
//!     - `tags`: { tag: count } — populates tag browser, autocomplete.
//!     These views are never on the initial paint path, so shipping them
//!     after `boot:hydrated` frees ~8s of IPC + JS work off the critical path.
//!
//! * `cache_boot_snapshot` (back-compat shim): returns the combined shape
//!     above for any ambient caller (second screen, tests, future code).
//!
//! The expensive link-context and tag-mention line-by-line data are NOT
//! included here — they're only used in hover cards / Index view, which can
//! lazy-fetch after first paint.

use rusqlite::{params, Connection, OpenFlags};
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::libraries::NoteLink;

/// Open a READ-ONLY connection to search.db. Uses its own Connection — does
/// NOT touch SearchState.db's mutex. SQLite WAL mode (set in init_db) allows
/// unlimited concurrent readers, so this is free and unblocking.
///
/// Why: previously every cache read fought the same Mutex<Connection> used
/// by the filesystem-walking reconcile, so a long walk froze Search Hub and
/// backlinks for the duration. With a dedicated reader connection the two
/// operations never contend — the writer walks on its own connection, any
/// number of readers can query through their own.
fn open_reader(app: &tauri::AppHandle) -> Result<Connection, String> {
    let path = crate::search::db_path(app)?;
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = Connection::open_with_flags(&path, flags)
        .map_err(|e| format!("Failed to open search.db (read-only): {}", e))?;
    // WAL readers set busy_timeout so they wait briefly rather than fail
    // if the writer holds a lock at an exact moment of checkpoint.
    conn.busy_timeout(std::time::Duration::from_millis(500))
        .map_err(|e| e.to_string())?;
    Ok(conn)
}

#[derive(Debug, Serialize)]
pub struct BootNote {
    pub name: String,
    pub path: String,
    pub library_name: String,
}

/// Back-compat combined snapshot. Preserved so ambient callers (second screen,
/// tests, legacy code) keep working. No longer on the boot critical path —
/// the frontend now awaits `BootSnapshotCore` for first-paint hydration and
/// defers `BootSnapshotGraph` via `requestIdleCallback`.
#[derive(Debug, Serialize)]
pub struct BootSnapshot {
    pub notes: Vec<BootNote>,
    pub links: Vec<NoteLink>,
    pub tags: HashMap<String, u32>,
    pub is_cold: bool,
}

/// Minimal boot payload required to paint the sidebar / file tree / Sight.
/// Returns in low-millis even on a 7,600-note Universe because `note_meta` is
/// a flat SQLite table with no joins and the row projection is narrow
/// (name, path, library_name — three `TEXT` columns).
#[derive(Debug, Serialize)]
pub struct BootSnapshotCore {
    pub notes: Vec<BootNote>,
    pub is_cold: bool,
    /// Per-phase Rust-side wall-clock timings for cold-boot attribution.
    /// Ordered the same way the phases run. Shipped to
    /// `boot-perf.latest.json` so we can tell whether a slow cold boot
    /// lives in `ensure_db`, `open_reader`, or `read_notes`. See
    /// `lab/boot-perf/boot-bundle-cold-start.md`.
    pub timings_ms: Vec<(String, u64)>,
    /// Server-side `SystemTime::now()` at the moment the struct is
    /// returned from the Tauri command, expressed as milliseconds since
    /// the Unix epoch. Paired with a `Date.now()` capture on the JS side
    /// immediately after `invoke()` resolves, the delta isolates pure
    /// IPC transport + JSON deserialize cost — independent of any work
    /// the JS caller does with the payload afterwards. Diagnostic tool
    /// for the Criterion 2 22.5s mystery (boot-perf 2026-04-19).
    pub server_return_unix_ms: u128,
    /// Server-side `SystemTime::now()` at the VERY FIRST line of the command
    /// body — before any work. Paired with a JS-side `Date.now()` captured
    /// immediately before `invoke()`, the delta is pure dispatcher-queue
    /// time (how long Tauri held the request before starting execution).
    /// If `queue_ms` is large but `body_ms` (server_return - server_start)
    /// is small, the bottleneck is the blocking-pool scheduler, not the
    /// SQLite work. Second round of IPC-overhead instrumentation
    /// (boot-perf 2026-04-19).
    pub server_start_unix_ms: u128,
}

/// Heavy boot payload — the typed-link edge list plus aggregated tag counts.
/// Deferred to `requestIdleCallback` on the frontend so the ~656k-row link
/// table never blocks first paint on large Universes. Only consumed by Sky
/// View, backlinks panel, tag browser, and the Lens — none of which are on
/// the initial paint path.
#[derive(Debug, Serialize)]
pub struct BootSnapshotGraph {
    pub links: Vec<NoteLink>,
    pub tags: HashMap<String, u32>,
    /// Per-phase Rust-side wall-clock timings for graph-phase attribution.
    /// Same purpose / shape as `BootSnapshotCore::timings_ms`.
    pub timings_ms: Vec<(String, u64)>,
    /// Server-side Unix-epoch millisecond timestamp at struct construction.
    /// See `BootSnapshotCore::server_return_unix_ms` — same diagnostic use.
    pub server_return_unix_ms: u128,
    /// Server-side Unix-epoch millisecond timestamp at command-body entry.
    /// See `BootSnapshotCore::server_start_unix_ms` — same diagnostic use.
    pub server_start_unix_ms: u128,
}

/// Fast boot payload — just the notes list and a cold-cache flag. The
/// frontend awaits this before marking `boot:hydrated` / clearing the
/// "Building index…" splash. The heavy link-graph + tag-aggregation payload
/// is fetched separately via `cache_boot_snapshot_graph` after first paint.
///
/// On a 7,600-note Universe this query returns in low-millis because
/// `note_meta` is indexed and the row projection is three narrow `TEXT`
/// columns.
#[tauri::command]
pub fn cache_boot_snapshot_core(app: tauri::AppHandle) -> Result<BootSnapshotCore, String> {
    // Stamp command-body entry FIRST — before any work. Paired with a JS-side
    // `Date.now()` captured immediately before `invoke()`, the delta is pure
    // Tauri-dispatcher queue time. See `BootSnapshotCore::server_start_unix_ms`.
    let server_start_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let mut timings: Vec<(String, u64)> = Vec::new();

    // Phase 1: schema bootstrap (no-op on existing DB).
    let t0 = Instant::now();
    let _ = crate::search::ensure_search_db_ready(&app);
    timings.push(("ensure_db".into(), t0.elapsed().as_millis() as u64));

    // Phase 2: open a dedicated read-only connection. SQLite WAL mode lets
    // this coexist with the writer and with other readers — no mutex contention.
    let t1 = Instant::now();
    let conn = match open_reader(&app) {
        Ok(c) => c,
        Err(_) => {
            timings.push(("open_reader_err".into(), t1.elapsed().as_millis() as u64));
            let server_return_unix_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            return Ok(BootSnapshotCore {
                notes: Vec::new(),
                is_cold: true,
                timings_ms: timings,
                server_return_unix_ms,
                server_start_unix_ms,
            });
        }
    };
    timings.push(("open_reader".into(), t1.elapsed().as_millis() as u64));

    // Phase 3: the actual row scan — `SELECT name, path, library_name FROM note_meta`.
    // This is the suspect phase on cold boot; `note_meta` is a row-store with
    // wide columns (body_text, *_json) that force full-page reads.
    let t2 = Instant::now();
    let notes = read_notes(&conn)?;
    timings.push(("read_notes".into(), t2.elapsed().as_millis() as u64));

    let is_cold = notes.is_empty();

    let server_return_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Ok(BootSnapshotCore { notes, is_cold, timings_ms: timings, server_return_unix_ms, server_start_unix_ms })
}

/// Heavy boot payload — link edges + tag counts. Deferred to
/// `requestIdleCallback` so the ~656k-row payload never blocks first paint.
///
/// Reads the typed-link `note_links` table when populated (current indexer)
/// and falls back to the legacy `outgoing_links_json` blob in `note_meta`
/// for indices built before typed links existed.
#[tauri::command]
pub fn cache_boot_snapshot_graph(app: tauri::AppHandle) -> Result<BootSnapshotGraph, String> {
    // Stamp command-body entry FIRST. See `cache_boot_snapshot_core` for
    // rationale — this is the queue-time diagnostic.
    let server_start_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let mut timings: Vec<(String, u64)> = Vec::new();

    let t0 = Instant::now();
    let _ = crate::search::ensure_search_db_ready(&app);
    timings.push(("ensure_db".into(), t0.elapsed().as_millis() as u64));

    let t1 = Instant::now();
    let conn = match open_reader(&app) {
        Ok(c) => c,
        Err(_) => {
            timings.push(("open_reader_err".into(), t1.elapsed().as_millis() as u64));
            let server_return_unix_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            return Ok(BootSnapshotGraph {
                links: Vec::new(),
                tags: HashMap::new(),
                timings_ms: timings,
                server_return_unix_ms,
                server_start_unix_ms,
            });
        }
    };
    timings.push(("open_reader".into(), t1.elapsed().as_millis() as u64));

    // Detect cold cache by counting note_meta rows — if zero, the index
    // hasn't been built yet and the fallback is pointless.
    let t2 = Instant::now();
    let note_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM note_meta", params![], |row| row.get(0))
        .unwrap_or(0);
    timings.push(("count_notes".into(), t2.elapsed().as_millis() as u64));

    let t3 = Instant::now();
    let mut links = read_links(&conn)?;
    timings.push(("read_links".into(), t3.elapsed().as_millis() as u64));

    if links.is_empty() && note_count > 0 {
        let t3b = Instant::now();
        links = read_untyped_links_fallback(&conn)?;
        timings.push(("read_untyped_links_fallback".into(), t3b.elapsed().as_millis() as u64));
    }

    let t4 = Instant::now();
    let tags = read_tags(&conn)?;
    timings.push(("read_tags".into(), t4.elapsed().as_millis() as u64));

    let server_return_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Ok(BootSnapshotGraph { links, tags, timings_ms: timings, server_return_unix_ms, server_start_unix_ms })
}

/// Back-compat shim — merges `cache_boot_snapshot_core` + `_graph` into the
/// original single-response shape. Kept so ambient callers (second screen,
/// tests, any external invocation) keep working; no longer on the boot
/// critical path.
#[tauri::command]
pub fn cache_boot_snapshot(app: tauri::AppHandle) -> Result<BootSnapshot, String> {
    // Shim: the per-phase timings produced by the split commands are not
    // included in the merged shape. Ambient callers (second screen, tests)
    // don't consume them; only the boot-perf scorecard does, and the boot
    // path no longer goes through this shim.
    let core = cache_boot_snapshot_core(app.clone())?;
    let graph = cache_boot_snapshot_graph(app)?;
    Ok(BootSnapshot {
        notes: core.notes,
        links: graph.links,
        tags: graph.tags,
        is_cold: core.is_cold,
    })
}

/// Project `note_meta` → `BootNote`. Single prepared statement, one scan.
fn read_notes(conn: &Connection) -> Result<Vec<BootNote>, String> {
    let mut notes = Vec::new();
    let mut stmt = conn
        .prepare("SELECT name, path, library_name FROM note_meta")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(BootNote {
                name: row.get(0)?,
                path: row.get(1)?,
                library_name: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows.flatten() {
        notes.push(r);
    }
    Ok(notes)
}

/// Read all links from the typed note_links table.
fn read_links(conn: &Connection) -> Result<Vec<NoteLink>, String> {
    let mut out = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT source_path, source_name, target_name, link_type, library_name
             FROM note_links WHERE status = 'active'",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let source_path: String = row.get(0)?;
            let source_name: String = row.get(1)?;
            let target: String = row.get(2)?;
            let link_type: String = row.get(3)?;
            let library_name: String = row.get(4)?;
            Ok(NoteLink {
                source_path,
                source_name,
                target,
                context: String::new(), // lazy — not needed at boot
                library_name,
                link_type: if link_type.is_empty() { None } else { Some(link_type) },
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows.flatten() {
        out.push(r);
    }
    Ok(out)
}

/// Fallback: parse outgoing_links_json from note_meta rows. Used when
/// note_links is empty but the index has notes — handles legacy indices.
fn read_untyped_links_fallback(conn: &Connection) -> Result<Vec<NoteLink>, String> {
    let mut out = Vec::new();
    let mut stmt = conn
        .prepare("SELECT path, name, library_name, outgoing_links_json FROM note_meta")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    for r in rows.flatten() {
        let (path, name, library_name, json) = r;
        let targets: Vec<String> = serde_json::from_str(&json).unwrap_or_default();
        for target in targets {
            out.push(NoteLink {
                source_path: path.clone(),
                source_name: name.clone(),
                target,
                context: String::new(),
                library_name: library_name.clone(),
                link_type: None,
            });
        }
    }
    Ok(out)
}

/// Aggregate tag counts across the entire index by parsing tags_json arrays.
/// O(n) over notes; a single SQL scan. For a 7,600-note Universe this
/// completes in low-millis on modern hardware.
fn read_tags(conn: &Connection) -> Result<HashMap<String, u32>, String> {
    let mut tags: HashMap<String, u32> = HashMap::new();
    let mut stmt = conn
        .prepare("SELECT tags_json FROM note_meta")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    for r in rows.flatten() {
        let arr: Vec<String> = serde_json::from_str(&r).unwrap_or_default();
        for t in arr {
            if t.is_empty() {
                continue;
            }
            *tags.entry(t).or_insert(0) += 1;
        }
    }
    Ok(tags)
}

/// Persist the frontend's boot-perf scorecard to
/// `<universe>/.constellation/boot-perf.latest.json`. Read by the Settings →
/// Debug panel (and the lab harness) to display pass/fail status against
/// the `lab/boot-perf/BOOT-BUDGET.md` ship-gate criteria.
#[tauri::command]
pub fn write_boot_perf_report(app: tauri::AppHandle, report_json: String) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let _ = std::fs::create_dir_all(&cdir);
    let path = cdir.join("boot-perf.latest.json");
    std::fs::write(&path, report_json).map_err(|e| e.to_string())
}

/// Read the most recent boot-perf report — used by Settings → Debug.
#[tauri::command]
pub fn read_boot_perf_report(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let path = cdir.join("boot-perf.latest.json");
    if !path.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(&path).map(Some).map_err(|e| e.to_string())
}

/// Return true if the cache has any entries — used by the frontend to decide
/// whether to show the first-run "Building index…" progress UI.
#[tauri::command]
pub fn cache_is_populated(app: tauri::AppHandle) -> Result<bool, String> {
    // Use the dedicated reader to avoid contending with any in-flight walk.
    let conn = match open_reader(&app) {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM note_meta", params![], |row| row.get(0))
        .unwrap_or(0);
    Ok(count > 0)
}

/// Reconcile the cache against the filesystem in the background: walk every
/// library, call the search indexer for each .md file (mtime-gated — unchanged
/// files are skipped without reading). Emits `cache-reconciled` when done so
/// the frontend can refresh any stats that came from the cache.
///
/// This is effectively a thin wrapper around `constellation_search_init` that
/// makes the reconcile step an explicit, named step in the boot pipeline.
#[tauri::command]
pub fn cache_reconcile(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    // Ensure the schema exists + state's query connection is ready BEFORE
    // we spawn the walk thread. This makes cache_boot_snapshot calls that
    // arrive during the walk succeed (they see the populated DB) instead of
    // racing the walker.
    crate::search::ensure_search_db_ready(&app)?;

    let app_clone = app.clone();
    std::thread::spawn(move || {
        let started = std::time::Instant::now();
        // reconcile_filesystem uses a DEDICATED connection — the state
        // connection stays free for concurrent frontend queries thanks to
        // SQLite WAL mode.
        let result = crate::search::reconcile_filesystem(&app_clone);
        let (note_count, was_cold) = match result {
            Ok(stats) => (stats.note_count, stats.note_count == 0),
            Err(_) => (0, true),
        };
        let _ = app_clone.emit("cache-reconciled", serde_json::json!({
            "was_cold": was_cold,
            "note_count": note_count,
            "elapsed_ms": started.elapsed().as_millis() as u64,
        }));
    });
    Ok(())
}
