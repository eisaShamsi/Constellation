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
//! `cache_boot_snapshot` returns, in a single IPC call:
//!   - `notes`: [{ name, path, library_name }] — populates file tree, Sky View, Sight
//!   - `links`: [NoteLink] — populates Sky View graph, backlinks, Index
//!   - `tags`: { tag: count } — populates tag browser, autocomplete
//!   - `is_cold`: true on first boot (cache empty) so the UI can show a
//!     one-time "Building index…" progress indicator.
//!
//! The expensive link-context and tag-mention line-by-line data are NOT
//! included here — they're only used in hover cards / Index view, which can
//! lazy-fetch after first paint.

use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use tauri::Manager;

use crate::libraries::NoteLink;
use crate::search::SearchState;

#[derive(Debug, Serialize)]
pub struct BootNote {
    pub name: String,
    pub path: String,
    pub library_name: String,
}

#[derive(Debug, Serialize)]
pub struct BootSnapshot {
    pub notes: Vec<BootNote>,
    pub links: Vec<NoteLink>,
    pub tags: HashMap<String, u32>,
    pub is_cold: bool,
}

/// Read the full boot snapshot from the SQLite cache. Pure in-memory query —
/// no filesystem I/O. Returns (notes, links, tags) for every library the
/// search index knows about.
///
/// `is_cold` is `true` iff `note_meta` is empty, which happens on a first
/// launch (or after the user clears the cache). The frontend uses this flag
/// to decide whether to show the one-time "Building index…" progress screen.
#[tauri::command]
pub fn cache_boot_snapshot(app: tauri::AppHandle) -> Result<BootSnapshot, String> {
    // CRITICAL: open the DB if state hasn't opened it yet. On 2nd+ boots the
    // search.db file already exists on disk with all our data, but nothing
    // has called constellation_search_init yet — so the state.db mutex is
    // empty and without this call we'd return is_cold: true despite the data
    // being right there. This was the bug in the initial cache-first boot:
    // the snapshot reported cold on every launch, defeating the whole point.
    let _ = crate::search::ensure_search_db_ready(&app);

    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;

    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => {
            return Ok(BootSnapshot {
                notes: Vec::new(),
                links: Vec::new(),
                tags: HashMap::new(),
                is_cold: true,
            });
        }
    };

    // ── Notes ───────────────────────────────────────────────────────
    let mut notes = Vec::new();
    {
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
    }

    let is_cold = notes.is_empty();

    // ── Links ───────────────────────────────────────────────────────
    // Read from the typed-link `note_links` table when available (populated by
    // index_note via extract_typed_links). Fall back to outgoing_links_json
    // from note_meta for notes that haven't been typed-link-indexed yet.
    let mut links = read_links(conn)?;
    if links.is_empty() && !is_cold {
        links = read_untyped_links_fallback(conn)?;
    }

    // ── Tags ────────────────────────────────────────────────────────
    let tags = read_tags(conn)?;

    Ok(BootSnapshot { notes, links, tags, is_cold })
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

/// Return true if the cache has any entries — used by the frontend to decide
/// whether to show the first-run "Building index…" progress UI.
#[tauri::command]
pub fn cache_is_populated(app: tauri::AppHandle) -> Result<bool, String> {
    let state = app.state::<SearchState>();
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    let conn = match db_guard.as_ref() {
        Some(c) => c,
        None => return Ok(false),
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
