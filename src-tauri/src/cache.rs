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
use tauri::Manager;

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
/// MIG-004 §9: alias mapping carried in the graph snapshot so the
/// frontend Backlinks / Outgoing / Map / Sight panels can resolve a
/// wikilink targeting an alias to the renamed note's current path.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteAliasOut {
    pub path: String,
    pub alias_lower: String,
}

#[derive(Debug, Serialize)]
pub struct BootSnapshotGraph {
    pub links: Vec<NoteLink>,
    pub tags: HashMap<String, u32>,
    /// MIG-004 §9: full alias table snapshot. Frontend builds an
    /// `alias_lower → path` map and an inverse `path → aliases[]` map.
    /// ~1.4k entries on the reference universe; trivial payload size.
    #[serde(default)]
    pub aliases: Vec<NoteAliasOut>,
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

/// Boot graph payload — **tags + aliases only** since MIG-079 §C.2b. The
/// 234k-row typed-link edge array used to be read here too (a full
/// `note_links` scan, ~11.3 s cold — the bulk of the cold `graph_ready`
/// cost); it is now deferred off boot to `cache_full_links` (lazy, behind
/// the frontend's `ensureFullLinks()` / `linksReady` guard). After §C.1 the
/// tag read is a `tag_counts` summary lookup (~ms) and aliases are ~1.4k
/// rows, so this command returns sub-second even cold. `links` is returned
/// as an empty vec for back-compat with the field's existing consumers.
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

    // MIG-061 §M — federate the graph payload across cUniverses.
    // Same pattern as cache_boot_snapshot_sky (§E + §L): get the
    // schema list, pick the right connection (bare for single-universe,
    // federated_conn for multi-schema), loop per schema and concatenate.
    // Per Boss principle: each universe's note_links / aliases / tags
    // are independent — no merge, just concatenation.
    let schemas = get_federated_schemas(&app);
    let is_federated = schemas.len() > 1;

    let t1 = Instant::now();
    let state = app.state::<crate::search::SearchState>();
    let bare_conn;
    let fed_guard;
    let conn: &Connection;
    if is_federated {
        fed_guard = state.federated_conn.lock()
            .map_err(|e| format!("federated_conn lock poisoned: {}", e))?;
        if let Some(c) = fed_guard.as_ref() {
            conn = c;
        } else {
            // federated_conn not yet ready — degrade to bare reader
            // (single-universe payload) for this call. Frontend will
            // re-invoke via the federation:ready event listener.
            timings.push(("federated_conn_none".into(), t1.elapsed().as_millis() as u64));
            let server_return_unix_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            return Ok(BootSnapshotGraph {
                aliases: Vec::new(),
                links: Vec::new(),
                tags: HashMap::new(),
                timings_ms: timings,
                server_return_unix_ms,
                server_start_unix_ms,
            });
        }
    } else {
        bare_conn = match open_reader(&app) {
            Ok(c) => c,
            Err(_) => {
                timings.push(("open_reader_err".into(), t1.elapsed().as_millis() as u64));
                let server_return_unix_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0);
                return Ok(BootSnapshotGraph {
                    aliases: Vec::new(),
                    links: Vec::new(),
                    tags: HashMap::new(),
                    timings_ms: timings,
                    server_return_unix_ms,
                    server_start_unix_ms,
                });
            }
        };
        conn = &bare_conn;
    }
    timings.push(("open_reader".into(), t1.elapsed().as_millis() as u64));

    // MIG-079 §C.2b — the 234k-row edge array is NO LONGER read at boot.
    // It was the ~11.3 s-cold cost on this command (a full `note_links`
    // scan, plus the now-removed cold-detection `COUNT(*)` per schema). Sky
    // View renders from the write-time `sky_*` payload
    // (`cache_boot_snapshot_sky`); the Backlinks/Outgoing COUNT badges read
    // the write-time `note_meta.incoming_count`/`outgoing_count` (§C.2a /
    // MIG-066). The full edge LIST (panel rows, the buildSkyData fallback,
    // live traversal chips) lazy-loads via the dedicated `cache_full_links`
    // command behind the frontend's memoized `ensureFullLinks()` + a
    // `linksReady` guard (idle pre-fetch right after `boot:graph-ready`).
    // The legacy untyped-links cold-detection now lives in `cache_full_links`.
    let links: Vec<NoteLink> = Vec::new();

    // MIG-061 §M — read tags per schema, accumulate counts into one map.
    let t4 = Instant::now();
    let mut tags: HashMap<String, u32> = HashMap::new();
    for schema in &schemas {
        read_tags_in_schema(conn, schema, &mut tags)?;
    }
    timings.push(("read_tags".into(), t4.elapsed().as_millis() as u64));

    // MIG-061 §M — read aliases per schema, concatenate.
    let t_alias = Instant::now();
    let mut aliases: Vec<NoteAliasOut> = Vec::new();
    for schema in &schemas {
        aliases.extend(read_aliases_in_schema(conn, schema)?);
    }
    timings.push(("read_aliases".into(), t_alias.elapsed().as_millis() as u64));

    let server_return_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Ok(BootSnapshotGraph { links, tags, aliases, timings_ms: timings, server_return_unix_ms, server_start_unix_ms })
}

/// MIG-079 §C.2b — the deferred edge list. Returns the full typed-link
/// `note_links` edge array (the payload `cache_boot_snapshot_graph` used to
/// carry). Invoked lazily off the boot critical path by the frontend's
/// memoized `ensureFullLinks()` (idle pre-fetch right after
/// `boot:graph-ready`, plus on first Backlinks/Outgoing/Graph open). The
/// scan itself is the same federated `read_links_in_schema` per schema +
/// the legacy `outgoing_links_json` fallback; §C.3's `idx_link_boot`
/// covering index keeps it index-only so the lazy scan reads leaf pages
/// only.
#[derive(Debug, Serialize)]
pub struct BootLinks {
    pub links: Vec<NoteLink>,
    pub timings_ms: Vec<(String, u64)>,
    pub server_return_unix_ms: u128,
    pub server_start_unix_ms: u128,
}

#[tauri::command]
pub fn cache_full_links(app: tauri::AppHandle) -> Result<BootLinks, String> {
    let server_start_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let mut timings: Vec<(String, u64)> = Vec::new();

    let t0 = Instant::now();
    let _ = crate::search::ensure_search_db_ready(&app);
    timings.push(("ensure_db".into(), t0.elapsed().as_millis() as u64));

    // Same federated-connection acquisition as `cache_boot_snapshot_graph`
    // / `cache_boot_snapshot_sky`: pick the bare reader for a single
    // universe, the attached `federated_conn` when cUniverses are present.
    let schemas = get_federated_schemas(&app);
    let is_federated = schemas.len() > 1;

    let t1 = Instant::now();
    let state = app.state::<crate::search::SearchState>();
    let bare_conn;
    let fed_guard;
    let conn: &Connection;
    if is_federated {
        fed_guard = state
            .federated_conn
            .lock()
            .map_err(|e| format!("federated_conn lock poisoned: {}", e))?;
        if let Some(c) = fed_guard.as_ref() {
            conn = c;
        } else {
            // Federation not yet attached — return empty; the frontend's
            // federation:ready handler force-re-invokes once it settles.
            timings.push(("federated_conn_none".into(), t1.elapsed().as_millis() as u64));
            let server_return_unix_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            return Ok(BootLinks { links: Vec::new(), timings_ms: timings, server_return_unix_ms, server_start_unix_ms });
        }
    } else {
        bare_conn = match open_reader(&app) {
            Ok(c) => c,
            Err(_) => {
                timings.push(("open_reader_err".into(), t1.elapsed().as_millis() as u64));
                let server_return_unix_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0);
                return Ok(BootLinks { links: Vec::new(), timings_ms: timings, server_return_unix_ms, server_start_unix_ms });
            }
        };
        conn = &bare_conn;
    }
    timings.push(("open_reader".into(), t1.elapsed().as_millis() as u64));

    // Per-schema typed-link read, concatenated (MIG-061 §M concatenation —
    // each universe's edges are independent).
    let t3 = Instant::now();
    let mut links: Vec<NoteLink> = Vec::new();
    for schema in &schemas {
        links.extend(read_links_in_schema(conn, schema)?);
    }
    timings.push(("read_links".into(), t3.elapsed().as_millis() as u64));

    // Legacy fallback: pre-typed-link indices have no `note_links` rows —
    // parse `outgoing_links_json` from `note_meta` when the typed table is
    // empty but notes exist (per schema).
    if links.is_empty() {
        let mut note_count: i64 = 0;
        for schema in &schemas {
            let sql = format!("SELECT COUNT(*) FROM {}.note_meta", schema);
            note_count += conn.query_row(&sql, params![], |row| row.get::<_, i64>(0)).unwrap_or(0);
        }
        if note_count > 0 {
            let t3b = Instant::now();
            for schema in &schemas {
                links.extend(read_untyped_links_fallback_in_schema(conn, schema)?);
            }
            timings.push(("read_untyped_links_fallback".into(), t3b.elapsed().as_millis() as u64));
        }
    }

    let server_return_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Ok(BootLinks { links, timings_ms: timings, server_return_unix_ms, server_start_unix_ms })
}

// ─── MIG-079 §C.2c — per-note link ROW queries ───────────────────────────
// The Backlinks/Outgoing panels need only ONE note's links, but today the
// frontend holds all 234k edges in a JS array and filters it per note (the
// in-memory-everything anti-pattern that froze scrolling). These commands
// return just the active note's rows from SQLite — a bounded, index-seeking
// lookup (the inverted-index "posting list" for the note's name): backlinks
// ride `idx_nl_tnl` (target_name_lower), outgoing rides `idx_link_source`.
// COUNT stays write-time (§C.2a); ROWS are a read-time indexed query (WA#5).
// Returns the SAME `NoteLink` shape `read_links_in_schema` does (context left
// empty/lazy), so the frontend `getBacklinks`/`getOutgoingLinks` sort+dedupe+
// tier logic is unchanged below the data source.

/// Backlink rows for one note in one schema: active edges whose lowercased
/// `target_name` is the note's name OR any alias (passed pre-lowercased).
/// `target_name_lower` is a VIRTUAL generated column indexed by `idx_nl_tnl`,
/// so `IN (...)` seeks. Defensive: a cUniverse on an older schema may lack the
/// column — skip it gracefully (Ok(empty)) rather than fail the whole query.
fn backlink_rows_in_schema(
    conn: &Connection,
    schema: &str,
    targets_lower: &[String],
) -> Result<Vec<NoteLink>, String> {
    if targets_lower.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = targets_lower.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    // PJ-065 — the structural (parent/TOC) lane never shows as a cognitive backlink
    // (the TOC panel is its only surface) and must not break the getBacklinks ==
    // incoming_count parity. Active since §5 (no-op only if the lane is ever un-registered).
    let sx = crate::link_types::snapshot().structural_not_in_clause("link_type");
    let sql = format!(
        "SELECT source_path, source_name, target_name, link_type, library_name, \
                weight, traversal_count, annotation, last_traversed, confidence \
         FROM {}.note_links \
         WHERE status != 'archived'{} AND target_name_lower IN ({})",
        schema, sx, placeholders
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Ok(Vec::new()), // older schema w/o target_name_lower — skip
    };
    let rows = stmt
        .query_map(rusqlite::params_from_iter(targets_lower.iter()), map_note_link_row)
        .map_err(|e| format!("query backlink rows ({}): {}", schema, e))?;
    let mut out = Vec::new();
    for r in rows.flatten() {
        out.push(r);
    }
    Ok(out)
}

/// Outgoing rows for one note in one schema: active edges whose `source_path`
/// is this note's path (seeks `idx_link_source`).
fn outgoing_rows_in_schema(
    conn: &Connection,
    schema: &str,
    source_path: &str,
) -> Result<Vec<NoteLink>, String> {
    // PJ-065 — exclude the structural (parent/TOC) lane from the cognitive
    // outgoing-links panel (the TOC panel is its surface). Active since §5.
    let sx = crate::link_types::snapshot().structural_not_in_clause("link_type");
    let sql = format!(
        "SELECT source_path, source_name, target_name, link_type, library_name, \
                weight, traversal_count, annotation, last_traversed, confidence \
         FROM {}.note_links \
         WHERE source_path = ? AND status != 'archived'{}",
        schema, sx
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Ok(Vec::new()),
    };
    let rows = stmt
        .query_map([source_path], map_note_link_row)
        .map_err(|e| format!("query outgoing rows ({}): {}", schema, e))?;
    let mut out = Vec::new();
    for r in rows.flatten() {
        out.push(r);
    }
    Ok(out)
}

/// Shared row → NoteLink projection for the per-note row queries. Mirrors
/// `read_links_in_schema` exactly (context lazy/empty).
fn map_note_link_row(row: &rusqlite::Row) -> rusqlite::Result<NoteLink> {
    let link_type: String = row.get(3)?;
    Ok(NoteLink {
        source_path: row.get(0)?,
        source_name: row.get(1)?,
        target: row.get(2)?,
        context: String::new(),
        library_name: row.get(4)?,
        link_type: if link_type.is_empty() { None } else { Some(link_type) },
        annotation: row.get(7)?,
        weight: row.get(5)?,
        traversal_count: row.get(6)?,
        last_traversed: row.get(8)?,
        confidence: row.get(9)?,
    })
}

/// MIG-079 §C.2c — backlink rows for the active note (federated). `aliases`
/// are the note's alias set (the frontend passes `notePathToAliases`); the
/// match is name + aliases, lowercased + de-duplicated. Replaces the frontend
/// `getBacklinks(allLibraryLinks, …)` array filter with a per-note indexed read.
#[tauri::command]
pub fn get_backlink_rows(
    app: tauri::AppHandle,
    note_name: String,
    aliases: Vec<String>,
) -> Result<Vec<NoteLink>, String> {
    let mut targets: Vec<String> = Vec::new();
    let primary = note_name.to_lowercase();
    if !primary.is_empty() {
        targets.push(primary);
    }
    for a in aliases {
        let al = a.to_lowercase();
        if !al.is_empty() && !targets.contains(&al) {
            targets.push(al);
        }
    }
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let _ = crate::search::ensure_search_db_ready(&app);
    let schemas = get_federated_schemas(&app);
    let is_federated = schemas.len() > 1;
    let state = app.state::<crate::search::SearchState>();
    if is_federated {
        let fed_guard = state.federated_conn.lock().map_err(|e| format!("federated_conn lock poisoned: {}", e))?;
        let conn = match fed_guard.as_ref() { Some(c) => c, None => return Ok(Vec::new()) };
        let mut out: Vec<NoteLink> = Vec::new();
        for schema in &schemas {
            out.extend(backlink_rows_in_schema(conn, schema, &targets)?);
        }
        return Ok(out);
    }
    // PJ-066 §C3 — single-schema: use the cached READ-ONLY reader connection (never waits on
    // the writer's lock, and no per-call connection open like the old `open_reader`).
    crate::search::with_read_conn(state.inner(), |conn| {
        let mut out: Vec<NoteLink> = Vec::new();
        for schema in &schemas {
            out.extend(backlink_rows_in_schema(conn, schema, &targets)?);
        }
        Ok(out)
    })
}

/// MIG-079 §C.2c — outgoing rows for the active note (federated). Replaces the
/// frontend `getOutgoingLinks(allLibraryLinks, notePath)` array filter.
#[tauri::command]
pub fn get_outgoing_rows(
    app: tauri::AppHandle,
    note_path: String,
) -> Result<Vec<NoteLink>, String> {
    if note_path.is_empty() {
        return Ok(Vec::new());
    }
    let _ = crate::search::ensure_search_db_ready(&app);
    let schemas = get_federated_schemas(&app);
    let is_federated = schemas.len() > 1;
    let state = app.state::<crate::search::SearchState>();
    if is_federated {
        let fed_guard = state.federated_conn.lock().map_err(|e| format!("federated_conn lock poisoned: {}", e))?;
        let conn = match fed_guard.as_ref() { Some(c) => c, None => return Ok(Vec::new()) };
        let mut out: Vec<NoteLink> = Vec::new();
        for schema in &schemas {
            out.extend(outgoing_rows_in_schema(conn, schema, &note_path)?);
        }
        return Ok(out);
    }
    // PJ-066 §C3 — single-schema: cached READ-ONLY reader (never waits on the writer).
    crate::search::with_read_conn(state.inner(), |conn| {
        let mut out: Vec<NoteLink> = Vec::new();
        for schema in &schemas {
            out.extend(outgoing_rows_in_schema(conn, schema, &note_path)?);
        }
        Ok(out)
    })
}

fn read_aliases(conn: &Connection) -> Result<Vec<NoteAliasOut>, String> {
    read_aliases_in_schema(conn, "main")
}

/// MIG-061 §M — schema-parameterized variant of read_aliases.
/// Defensive: cUniverses on older schemas may lack `note_aliases`.
/// Returns Ok(empty) for those — does not fail the whole graph read.
fn read_aliases_in_schema(conn: &Connection, schema: &str) -> Result<Vec<NoteAliasOut>, String> {
    // MIG-004 §10 audit-fix (4A-MED): ORDER BY path keeps the
    // serialized payload stable across boots.
    let sql = format!(
        "SELECT path, alias_lower FROM {}.note_aliases ORDER BY path",
        schema
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Ok(Vec::new()), // older schemas — skip gracefully
    };
    let rows = stmt
        .query_map([], |row| Ok(NoteAliasOut {
            path: row.get(0)?,
            alias_lower: row.get(1)?,
        }))
        .map_err(|e| format!("query aliases ({}): {}", schema, e))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("row aliases ({}): {}", schema, e))?);
    }
    Ok(out)
}

// ─── MIG-001 Step 8: pre-shaped Sky View payload ──────────────────────
// `cache_boot_snapshot_sky` reads sky_nodes + sky_links directly and
// returns a pre-shaped `{ nodes, links }` payload that the frontend can
// feed to GraphMindView without running buildSkyData(). Kills the 217k-
// edge JS iteration the old path paid on every boot.
//
// Gate: is_ready = schema_versions.sky >= SKY_SCHEMA_VERSION. The Step 5
// back-fill stamps this on completion. If the stamp is absent (mid-back-
// fill, or the user is on a fresh install where the back-fill hasn't
// finished yet), is_ready=false and the frontend falls back to the old
// buildSkyData path. Triggers continue populating sky_* for new writes
// so the back-fill and the new path coexist cleanly.

/// Shape matches the TypeScript `SkyNode` interface exactly so the
/// frontend can assign the response directly to `skyNodes` without a
/// transform step. `serde(rename_all = "camelCase")` handles the
/// snake_case → camelCase translation at serialize time.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyNodeOut {
    pub id: String,
    pub name: String,
    pub path: String,
    pub library_name: String,
    pub link_count: u32,
    pub outgoing_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stratum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maturity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

/// Matches TypeScript `SkyLink`. `source` and `target` are lowercase
/// names — same as what `buildSkyData` produced from the old path, so
/// downstream consumers (ego filter, Louvain, highlight) see the exact
/// same shape after the frontend swap in Step 9.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyLinkOut {
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootSnapshotSky {
    pub nodes: Vec<SkyNodeOut>,
    pub links: Vec<SkyLinkOut>,
    /// False when schema_versions.sky hasn't reached SKY_SCHEMA_VERSION
    /// yet — the frontend should fall back to buildSkyData() in that
    /// case. Happens mid-back-fill on first boot after the upgrade.
    pub is_ready: bool,
    pub timings_ms: Vec<(String, u64)>,
}

/// MIG-061 §A — returns the list of schema aliases to query for federated
/// sky data. Always includes `"main"` first; appends each attached cUniverse
/// alias (`"cu0"`, `"cu1"`, …) in attach order if federation is ready.
/// Empty cUniverse list → returns just `["main"]` (single-universe behavior,
/// identical to pre-MIG-061).
///
/// Q3 Option B (federated link resolution): the schema-order is the
/// deterministic tiebreak for name collisions across universes.
/// `path_to_idx` / `name_to_idx` are built in this order during the merge
/// pass; first-insert-wins → schema-order wins.
fn get_federated_schemas(app: &tauri::AppHandle) -> Vec<String> {
    let state = app.state::<crate::search::SearchState>();
    let mut schemas = vec!["main".to_string()];
    if let Ok(fed) = state.federation.lock() {
        if fed.is_ready() {
            for (alias, _path) in fed.attached() {
                schemas.push(alias.clone());
            }
        }
    }
    schemas
}

/// MIG-061 §D — federated readiness gate (Q4 Option A: all-or-nothing).
///
/// Returns `true` only if EVERY schema in `schemas` has stamped a
/// `sky_schema_version >= SKY_SCHEMA_VERSION`. If any schema's
/// back-fill hasn't completed, returns `false` — and the caller in §E
/// returns an empty snapshot with `is_ready=false`, which causes the
/// frontend to fall back to the existing `buildSkyData` legacy path.
///
/// Conservative by design: never returns partial data. The next call
/// (typically after a back-fill stamp event) re-checks all schemas.
fn is_federated_sky_ready(conn: &Connection, schemas: &[String]) -> bool {
    for schema in schemas {
        let sql = format!(
            "SELECT version FROM {}.schema_versions WHERE module = 'sky'",
            schema
        );
        let v: i64 = conn.query_row(&sql, [], |r| r.get(0)).unwrap_or(0);
        if v < crate::search::SKY_SCHEMA_VERSION as i64 {
            return false;
        }
    }
    true
}

/// Sky View snapshot from the persisted sky_* tables. Linear in rows,
/// no JS-side iteration, no IPC re-serialization of raw note_links.
///
/// **MIG-061 (federation):** queries every schema in
/// `get_federated_schemas()` — `main` plus every attached cUniverse
/// (`cu0`, `cu1`, …) — and merges the result. The merge:
///
/// 1. Resolves Q4 Option A readiness: if any schema lags on
///    sky_schema_version, returns is_ready=false (frontend falls back
///    to legacy `buildSkyData`).
/// 2. Concatenates nodes across all schemas.
/// 3. Builds `path_to_idx` / `name_to_idx` / `alias_to_path` from the
///    MERGED node set (Q3 Option B: federated link resolution; cross-
///    universe wikilinks resolve to whichever schema has the target,
///    deterministic schema-order winner via first-insert-wins).
/// 4. Concatenates links across all schemas, resolving each one against
///    the merged maps.
///
/// Single-universe (no cUniverses) behavior is byte-identical to the
/// pre-MIG-061 path: schemas=["main"], no extra overhead.
///
/// **MIG-079 §C.2d:** `async` (the §9.1 lever). The body is synchronous
/// rusqlite, but `#[tauri::command(async)]` makes Tauri run it on its
/// worker thread pool instead of the single IPC dispatch thread — so the
/// cold 234k-row `sky_links` scan (~11 s on the reference universe) no
/// longer monopolises that thread and stalls every other boot IPC behind
/// it (measured: an 11.4 s gap in the IPC arrival trace). With §C.2d the
/// read is also deferred off boot (lazy on first Sky-surface open + an
/// after-idle background warm-up) — and both the on-open load and the
/// warm-up rely on this async attribute to avoid freezing the app.
#[tauri::command(async)]
pub fn cache_boot_snapshot_sky(app: tauri::AppHandle) -> Result<BootSnapshotSky, String> {
    let mut timings: Vec<(String, u64)> = Vec::new();

    let t0 = Instant::now();
    let _ = crate::search::ensure_search_db_ready(&app);
    timings.push(("ensure_db".into(), t0.elapsed().as_millis() as u64));

    // MIG-061 §A — collect schemas to query.
    let schemas = get_federated_schemas(&app);
    let is_federated = schemas.len() > 1;

    // Pick the right connection:
    //   - Single-universe → bare open_reader (cheaper, no ATTACH state).
    //   - Federated      → the warm federated_conn (has all cu* ATTACHed).
    //
    // `state` lifetime: tauri's State<T> is reference-counted, so binding
    // it at function scope keeps the underlying `Arc<SearchState>` alive
    // for as long as `fed_guard` borrows from it.
    let t1 = Instant::now();
    let state = app.state::<crate::search::SearchState>();
    let bare_conn;
    let fed_guard;
    let conn: &Connection;
    if is_federated {
        fed_guard = state.federated_conn.lock()
            .map_err(|e| format!("federated_conn lock poisoned: {}", e))?;
        if let Some(c) = fed_guard.as_ref() {
            conn = c;
        } else {
            // Federation context advertised cUniverses but federated_conn
            // is None — context-not-ready race during a universe switch.
            // Return empty + is_ready=false; frontend falls back.
            timings.push(("open_reader".into(), t1.elapsed().as_millis() as u64));
            return Ok(BootSnapshotSky {
                nodes: Vec::new(),
                links: Vec::new(),
                is_ready: false,
                timings_ms: timings,
            });
        }
    } else {
        bare_conn = open_reader(&app)?;
        conn = &bare_conn;
    }
    timings.push(("open_reader".into(), t1.elapsed().as_millis() as u64));

    // MIG-061 §D — federated readiness gate (Q4 Option A: all-or-nothing).
    // Every schema must have stamped sky_schema_version >= SKY_SCHEMA_VERSION.
    // If any schema's back-fill is still in flight, return is_ready=false.
    if !is_federated_sky_ready(conn, &schemas) {
        return Ok(BootSnapshotSky {
            nodes: Vec::new(),
            links: Vec::new(),
            is_ready: false,
            timings_ms: timings,
        });
    }

    // Strategy: avoid SQL JOINs + GROUP BY subqueries (both were O(N×M)
    // on the target universe — read_nodes was 4.8s and read_links was
    // 2.9s with the JOIN pattern because SQLite materialized aggregate
    // temp tables for each query). Instead, stream three cheap scans
    // and aggregate in Rust. Result on the same 7.6k-node / 232k-link
    // universe: ~200-400ms total, roughly 20× faster.
    //
    // Query order matters: nodes first so we can build the
    // path→id / name→idx maps used to resolve link source strings
    // and accumulate incoming counts in a single links pass.
    //
    // MIG-061 federation: same strategy, repeated per schema. Each
    // per-schema `prepare` is cheap (~µs) on the warm connection;
    // the cost is dominated by row materialization.

    // MIG-061 §L — per-schema isolation (Q3 → Option A revised per Boss
    // principle: "the data of Universe A shouldn't be merged/integrated
    // with Universe B").
    //
    // Each cUniverse's sky_links resolve ONLY against its own sky_nodes
    // and its own note_aliases. A wikilink in cu0 that targets a name
    // existing in cu1 does NOT resolve cross-universe — it stays
    // unresolved (or falls back to lowercase id) just as it would if
    // cu0 were viewed standalone, detached from cu1.
    //
    // Strict invariant: cu0's behavior must be identical whether
    // standalone or attached as a cUniverse of B. Federation is a
    // read-side concatenation, never a runtime merge of resolution
    // state.
    //
    // Memory: per-schema maps are slightly larger total than a merged
    // map (HashMap overhead × N schemas), but only by HashMap-bucket
    // overhead — node count is the same. On a 25-universe setup this
    // is negligible (a few MB).
    let mut nodes: Vec<SkyNodeOut> = Vec::new();
    let mut links: Vec<SkyLinkOut> = Vec::new();

    let t2 = Instant::now();
    let t_alias = Instant::now();
    let t3 = Instant::now();
    for schema in &schemas {
        // 1. Load this schema's nodes, append to the merged nodes vec.
        let schema_start = nodes.len();
        nodes.extend(read_sky_nodes_raw_in_schema(conn, schema)?);

        // 2. Build per-schema maps from THIS schema's nodes only.
        //    Indices point into the merged `nodes` vec via the
        //    schema_start offset, so link_count / outgoing_count bumps
        //    in read_sky_links_raw_in_schema land on the correct rows.
        let schema_len = nodes.len() - schema_start;
        let mut path_to_idx: std::collections::HashMap<String, usize> =
            std::collections::HashMap::with_capacity(schema_len);
        let mut name_to_idx: std::collections::HashMap<String, usize> =
            std::collections::HashMap::with_capacity(schema_len);
        for i in schema_start..nodes.len() {
            path_to_idx.insert(nodes[i].path.clone(), i);
            name_to_idx.entry(nodes[i].name.clone()).or_insert(i);
        }

        // 3. Load this schema's note_aliases. Defensive: older
        //    cUniverses may lack the table — skip gracefully.
        let mut alias_to_path: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let alias_sql = format!(
            "SELECT alias_lower, path FROM {}.note_aliases ORDER BY path",
            schema
        );
        match conn.prepare(&alias_sql) {
            Ok(mut stmt) => {
                let rows = stmt
                    .query_map([], |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })
                    .map_err(|e| format!("query aliases ({}): {}", schema, e))?;
                for r in rows {
                    let (alias, path) = r.map_err(|e| format!("row aliases ({}): {}", schema, e))?;
                    alias_to_path.entry(alias).or_insert(path);
                }
            }
            Err(e) => {
                timings.push((format!("alias_skip:{}:{}", schema, e), 0));
            }
        }

        // 4. Read this schema's links + resolve them against THIS
        //    schema's maps only. Per Q3 Option A: no cross-universe
        //    resolution.
        links.extend(read_sky_links_raw_in_schema(
            conn,
            schema,
            &path_to_idx,
            &name_to_idx,
            &alias_to_path,
            &mut nodes,
        )?);
    }
    timings.push(("scan_nodes".into(), t2.elapsed().as_millis() as u64));
    timings.push(("scan_aliases".into(), t_alias.elapsed().as_millis() as u64));
    timings.push(("scan_links_and_counts".into(), t3.elapsed().as_millis() as u64));

    Ok(BootSnapshotSky { nodes, links, is_ready: true, timings_ms: timings })
}

/// MIG-061 §B — schema-parameterized variant of the sky_nodes scan.
///
/// `schema` is interpolated as-is into the SQL. Caller MUST pass either
/// the literal `"main"` or an alias from `get_federated_schemas` (which
/// only yields validated alphanumeric aliases from `federation::attach::
/// schema_alias`). Any other source would be a SQL-injection foothold.
///
/// Pre-MIG-061 callers used the wrapper `read_sky_nodes_raw(conn)` which
/// is now defined below as a thin shim that passes `"main"`.
fn read_sky_nodes_raw_in_schema(
    conn: &Connection,
    schema: &str,
) -> Result<Vec<SkyNodeOut>, String> {
    let sql = format!(
        "SELECT id, name, path, library_name, stratum, maturity, origin_type, created_at \
         FROM {}.sky_nodes",
        schema
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("prepare nodes ({}): {}", schema, e))?;
    let rows = stmt
        .query_map([], |row| {
            // MIG-061 §K — flexible stratum read.
            // The `stratum` column is declared TEXT in the schema
            // (search.rs:2210) but populated via STRATUM_SQL_EXPR which
            // computes an INTEGER (1-8). SQLite's loose typing means
            // SOME rows store the value as INTEGER class, others as TEXT.
            // The pre-MIG-061 code (and the previous MIG-061 §B revision)
            // used `row.get::<_, Option<i64>>(4)?` which fails on TEXT-
            // class rows with "Invalid column type Text at index: 4".
            //
            // Surfaced by the §J.3 diagnostic trace: the boot-path call
            // to cache_boot_snapshot_sky has been silently failing in
            // production all along; frontend fell back to `buildSkyData`
            // (the legacy path) which doesn't read sky_nodes at all.
            //
            // This read handles both storage classes by inspecting the
            // raw rusqlite::Value, parsing TEXT to i64 if needed.
            let stratum: Option<i64> = match row.get_ref(4)? {
                rusqlite::types::ValueRef::Null => None,
                rusqlite::types::ValueRef::Integer(i) => Some(i),
                rusqlite::types::ValueRef::Text(b) => std::str::from_utf8(b)
                    .ok()
                    .and_then(|s| s.parse::<i64>().ok()),
                rusqlite::types::ValueRef::Real(f) => Some(f as i64),
                rusqlite::types::ValueRef::Blob(_) => None,
            };
            Ok(SkyNodeOut {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                library_name: row.get(3)?,
                // Counts filled in during the links scan pass.
                link_count: 0,
                outgoing_count: 0,
                stratum,
                maturity: row.get::<_, Option<String>>(5)?,
                origin_type: row.get::<_, Option<String>>(6)?,
                created_at: row.get::<_, Option<i64>>(7)?,
            })
        })
        .map_err(|e| format!("query nodes ({}): {}", schema, e))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("row nodes ({}): {}", schema, e))?);
    }
    Ok(out)
}

/// Back-compat wrapper preserved for callers (and tests) that don't
/// need schema parameterization. Identical to pre-MIG-061 behavior.
#[allow(dead_code)] // Kept for back-compat / clarity; primary call site is §E
fn read_sky_nodes_raw(conn: &Connection) -> Result<Vec<SkyNodeOut>, String> {
    read_sky_nodes_raw_in_schema(conn, "main")
}

/// MIG-061 §C — schema-parameterized variant of the sky_links scan.
///
/// Same SQL-injection-safety rules as `read_sky_nodes_raw_in_schema`:
/// `schema` MUST be `"main"` or an alphanumeric alias from
/// `get_federated_schemas`.
///
/// Q3 Option B (federated link resolution): the `path_to_idx`,
/// `name_to_idx`, and `alias_to_path` maps passed in are built across
/// the MERGED node set (all schemas) by §E. So a link with
/// `target_name = "FooBar"` from cu0's sky_links resolves to whichever
/// schema has the FooBar node — first-insert-wins on cross-schema
/// name collision (schema-order winner: main > cu0 > cu1 > ...).
fn read_sky_links_raw_in_schema(
    conn: &Connection,
    schema: &str,
    path_to_idx: &std::collections::HashMap<String, usize>,
    name_to_idx: &std::collections::HashMap<String, usize>,
    alias_to_path: &std::collections::HashMap<String, String>,
    nodes_mut: &mut [SkyNodeOut],
) -> Result<Vec<SkyLinkOut>, String> {
    let sql = format!(
        "SELECT source_path, target_name, link_type FROM {}.sky_links",
        schema
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("prepare links ({}): {}", schema, e))?;
    let rows = stmt
        .query_map([], |row| {
            let source_path: String = row.get(0)?;
            let target_name: String = row.get(1)?;
            let link_type: String = row.get(2)?;
            Ok((source_path, target_name, link_type))
        })
        .map_err(|e| format!("query links ({}): {}", schema, e))?;

    // Reserve roughly the expected capacity to avoid reallocs. For the
    // target universe (232k links) this saves a handful of vec grows.
    let mut out: Vec<SkyLinkOut> = Vec::with_capacity(256 * 1024);

    for r in rows {
        let (source_path, target_name, link_type) = r.map_err(|e| format!("row links ({}): {}", schema, e))?;

        // Source id comes from the already-loaded node list via path.
        // Orphan edge (source_path not in sky_nodes) gets skipped — we
        // saw this happen rarely when a note was deleted mid-back-fill
        // before the AD trigger cascade ran to completion.
        let Some(&src_idx) = path_to_idx.get(&source_path) else {
            continue;
        };

        // Outgoing count bumped here (by source).
        nodes_mut[src_idx].outgoing_count += 1;

        // Target resolution (3-tier):
        //   1. name_to_idx hit  — wikilink targets the current name.
        //      Use the pre-lowercased `id` and bump link_count.
        //   2. alias_to_path hit (MIG-004 §8) — wikilink targets an
        //      alias. Resolve to canonical path → path_to_idx → id.
        //      Bumps link_count on the canonical row.
        //   3. Unresolved — orphan wikilink, target doesn't exist as a
        //      note or any alias. Fall back to an in-place ASCII
        //      downcase if possible, else unicode lowercase.
        //
        // The alias step is the rename fix: after a note renames,
        // wikilinks in other notes still target the OLD name (or any
        // historical alias). The alias table maps that old name back
        // to the renamed note's current path so the edge attaches to
        // the right node and link_count stays correct.
        let target = if let Some(&tgt_idx) = name_to_idx.get(&target_name) {
            nodes_mut[tgt_idx].link_count += 1;
            nodes_mut[tgt_idx].id.clone()
        } else if let Some(canonical_path) = alias_to_path.get(&target_name) {
            if let Some(&tgt_idx) = path_to_idx.get(canonical_path) {
                nodes_mut[tgt_idx].link_count += 1;
                nodes_mut[tgt_idx].id.clone()
            } else if target_name.is_ascii() {
                let mut s = target_name;
                s.make_ascii_lowercase();
                s
            } else {
                target_name.to_lowercase()
            }
        } else if target_name.is_ascii() {
            let mut s = target_name;
            s.make_ascii_lowercase();
            s
        } else {
            target_name.to_lowercase()
        };

        // Source id clone: same pattern — use the pre-lowercased id.
        // One String alloc per edge (clone of an existing ~32-byte
        // String). The ~232k edges on the target universe = ~7MB of
        // cloning, vs. ~7MB + 232k unicode-aware lowercase calls with
        // the naive loop.
        let source = nodes_mut[src_idx].id.clone();

        out.push(SkyLinkOut {
            source,
            target,
            link_type: if link_type.is_empty() { None } else { Some(link_type) },
        });
    }
    Ok(out)
}

/// Back-compat wrapper. Identical to pre-MIG-061 behavior — reads only
/// from the bare `sky_links` table (which SQLite resolves to `main.sky_links`).
#[allow(dead_code)] // Kept for back-compat / clarity; primary call site is §E
fn read_sky_links_raw(
    conn: &Connection,
    path_to_idx: &std::collections::HashMap<String, usize>,
    name_to_idx: &std::collections::HashMap<String, usize>,
    alias_to_path: &std::collections::HashMap<String, String>,
    nodes_mut: &mut [SkyNodeOut],
) -> Result<Vec<SkyLinkOut>, String> {
    read_sky_links_raw_in_schema(conn, "main", path_to_idx, name_to_idx, alias_to_path, nodes_mut)
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
    let graph = cache_boot_snapshot_graph(app.clone())?;
    // MIG-079 §C.2b — `graph.links` is now empty (edges deferred); fetch the
    // full edge list explicitly so ambient callers of this back-compat shim
    // keep the original combined shape.
    let full = cache_full_links(app)?;
    Ok(BootSnapshot {
        notes: core.notes,
        links: full.links,
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
    read_links_in_schema(conn, "main")
}

/// MIG-061 §M — schema-parameterized variant of read_links.
/// Reads `{schema}.note_links`. Each cUniverse's note_links table is
/// independent — per Boss principle (Q3 Option A): the federated graph
/// payload is a pure concatenation of each universe's own links, with
/// no cross-universe merge or remapping. Detach a cUniverse and the
/// remaining universes' link data is unaffected.
fn read_links_in_schema(conn: &Connection, schema: &str) -> Result<Vec<NoteLink>, String> {
    let mut out = Vec::new();
    // PJ-065 — the federated full-links payload + boot BootLinks bundle stay
    // cognitive: the structural (parent/TOC) lane is served only by the dedicated
    // get_structural_* APIs, never the boot bundle (so boot-bundle size is
    // unchanged and frontend cognitive graph consumers never see it). Active since §5.
    let sx = crate::link_types::snapshot().structural_not_in_clause("link_type");
    let sql = format!(
        "SELECT source_path, source_name, target_name, link_type, library_name, \
                weight, traversal_count, annotation, last_traversed, confidence \
         FROM {}.note_links WHERE status = 'active'{}",
        schema, sx
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare links ({}): {}", schema, e))?;
    let rows = stmt
        .query_map([], |row| {
            let source_path: String = row.get(0)?;
            let source_name: String = row.get(1)?;
            let target: String = row.get(2)?;
            let link_type: String = row.get(3)?;
            let library_name: String = row.get(4)?;
            let weight: f64 = row.get(5)?;
            let traversal_count: i64 = row.get(6)?;
            let annotation: String = row.get(7)?;
            let last_traversed: String = row.get(8)?;
            let confidence: String = row.get(9)?;
            Ok(NoteLink {
                source_path,
                source_name,
                target,
                context: String::new(), // lazy — not needed at boot
                library_name,
                link_type: if link_type.is_empty() { None } else { Some(link_type) },
                annotation,
                weight,
                traversal_count,
                last_traversed,
                confidence,
            })
        })
        .map_err(|e| format!("query links ({}): {}", schema, e))?;
    for r in rows.flatten() {
        out.push(r);
    }
    Ok(out)
}

/// Fallback: parse outgoing_links_json from note_meta rows. Used when
/// note_links is empty but the index has notes — handles legacy indices.
fn read_untyped_links_fallback(conn: &Connection) -> Result<Vec<NoteLink>, String> {
    read_untyped_links_fallback_in_schema(conn, "main")
}

/// MIG-061 §M — schema-parameterized variant of the untyped-links fallback.
fn read_untyped_links_fallback_in_schema(conn: &Connection, schema: &str) -> Result<Vec<NoteLink>, String> {
    let mut out = Vec::new();
    let sql = format!(
        "SELECT path, name, library_name, outgoing_links_json FROM {}.note_meta",
        schema
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("prepare fallback ({}): {}", schema, e))?;
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
                annotation: String::new(),
                weight: 1.0,
                traversal_count: 0,
                last_traversed: String::new(),
                confidence: String::new(),
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
    read_tags_in_schema(conn, "main", &mut tags)?;
    Ok(tags)
}

/// MIG-061 §M — schema-parameterized variant of read_tags.
/// Accumulates tag counts INTO the supplied map so the caller can
/// federate across multiple schemas (counts add across universes —
/// e.g., #project appearing 5 times in main and 3 times in cu0 → 8).
fn read_tags_in_schema(
    conn: &Connection,
    schema: &str,
    tags: &mut HashMap<String, u32>,
) -> Result<(), String> {
    // MIG-079 §C.1 — write-time path: when this schema's `tag_counts` summary is
    // stamped current, read the O(distinct-tags) table (~ms) instead of scanning
    // every note's `tags_json` (5.6 s on the live universe — the inline body_text
    // makes each note_meta row read expensive). The table is maintained by the
    // ±delta in index_note/reindex_delete_note and rebuilt by reconcile, so it is
    // always current. Any read error falls through to the legacy scan, so a fresh
    // or un-upgraded (e.g. older attached cUniverse) schema is never wrong.
    if crate::tag_counts::is_stamped_in_schema(conn, schema) {
        let sql = format!("SELECT tag, n FROM {}.tag_counts WHERE n > 0", schema);
        let read = (|| -> Result<(), String> {
            let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))
                .map_err(|e| e.to_string())?;
            for r in rows.flatten() {
                let (tag, n) = r;
                if !tag.is_empty() && n > 0 {
                    *tags.entry(tag).or_insert(0) += n as u32;
                }
            }
            Ok(())
        })();
        if read.is_ok() {
            return Ok(());
        }
        // else: fall through to the legacy scan below.
    }

    let sql = format!("SELECT tags_json FROM {}.note_meta", schema);
    let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare tags ({}): {}", schema, e))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("query tags ({}): {}", schema, e))?;
    for r in rows.flatten() {
        let arr: Vec<String> = serde_json::from_str(&r).unwrap_or_default();
        for t in arr {
            if t.is_empty() {
                continue;
            }
            *tags.entry(t).or_insert(0) += 1;
        }
    }
    Ok(())
}

/// Persist the frontend's boot-perf scorecard to
/// `<universe>/.constellation/boot-perf.latest.json`. Read by the Settings →
/// Debug panel (and the lab harness) to display pass/fail status against
/// the `lab/boot-perf/BOOT-BUDGET.md` ship-gate criteria.
#[tauri::command]
pub fn write_boot_perf_report(app: tauri::AppHandle, report_json: String) -> Result<(), String> {
    let cdir = crate::universe::active_constellation_dir(&app)?;
    let _ = std::fs::create_dir_all(&cdir);
    // `latest` is overwritten each write (Settings → Debug + the lab harness read it).
    let path = cdir.join("boot-perf.latest.json");
    std::fs::write(&path, &report_json).map_err(|e| e.to_string())?;
    // MIG-079 §B — durable, APPEND-ONLY per-boot history (NEVER overwritten), so
    // every launch (cold + warm, many per session) is captured and no measurement
    // is lost to the latest.json overwrite. One compact JSON object per line — the
    // frontend sends `JSON.stringify(report)` (single line) → valid JSON Lines.
    // Best-effort: a history write must never fail the boot or the latest write.
    let history = cdir.join("boot-perf.history.jsonl");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&history) {
        use std::io::Write;
        let _ = f.write_all(report_json.as_bytes());
        let _ = f.write_all(b"\n");
    }
    Ok(())
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
        // MIG-073 §P3 — a completed reconcile walk is the bulk-link-change
        // settle point: note_links may have changed arbitrarily, so refresh
        // the Knowledge Health snapshot unconditionally. Already on the
        // walker thread; uses its own dedicated connection.
        crate::search::kh_cache_recompute_blocking(&app_clone, false);
    });
    Ok(())
}

/// MIG-067 — the boot-time, WALK-FREE counterpart to `cache_reconcile`. Ensures
/// the search DB connection is ready and fires the same `cache-reconciled` event
/// the frontend listens for (which loads incoming link counts and marks search
/// ready) — but WITHOUT `reconcile_filesystem`'s stat-every-
/// file walk. That walk is what the ZERO BOOT-TIME WALKS rule forbids on boot;
/// it belongs only to the live watcher or an explicit Settings → Rebuild Index.
/// (A §B-era boot `cache_reconcile` re-introduced the walk and was the audible
/// background thrash; this replaces it.)
#[tauri::command]
pub fn cache_mark_search_ready(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    crate::search::ensure_search_db_ready(&app)?;
    let _ = app.emit("cache-reconciled", serde_json::json!({
        "was_cold": false,
        "note_count": 0i64,
        "elapsed_ms": 0u64,
    }));
    // MIG-073 — first-time population of the Knowledge Health snapshot cache.
    // One-off backfill (Rule 8): a single COUNT on every later boot, the full
    // recompute only when the cache is empty. Spawned + dedicated connection —
    // never blocks the boot pipeline, so MIG-067's walk-free boot stays intact.
    crate::search::spawn_kh_cache_recompute(&app, true);
    Ok(())
}

// ════════════════════════════════════════════════════════════════════
// MIG-061 §G — Federation tests for the sky_* reader path.
//
// These exercise the new schema-parameterized helpers and the federated
// readiness gate. They use temp-file SQLite (not :memory:) because the
// federation path requires ATTACH DATABASE on a populated file. Each
// test sets up:
//   - A "main" search.db with sky_nodes / sky_links / note_aliases /
//     schema_versions tables.
//   - For federated tests: one or more "cuN" search.db files, ATTACHed
//     to the main connection.
//
// Tests focus on the read-helper layer (read_sky_nodes_raw_in_schema,
// read_sky_links_raw_in_schema, is_federated_sky_ready). The top-level
// `cache_boot_snapshot_sky` is not directly testable here because it
// requires a Tauri AppHandle + SearchState; its behavior is covered
// indirectly via the helper tests + the §H Boss-test.
// ════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a fresh search.db at `path` with the sky_* + note_aliases
    /// + schema_versions tables, stamped at SKY_SCHEMA_VERSION (ready).
    /// Pre-populates with the given nodes + links.
    fn make_synthetic_sky_db(
        path: &std::path::Path,
        nodes: &[(&str, &str, &str, &str)], // (id, name, path, library_name)
        links: &[(&str, &str, &str)],       // (source_path, target_name, link_type)
        aliases: &[(&str, &str)],           // (alias_lower, target_path)
        stamp_ready: bool,
    ) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_versions (
                module TEXT PRIMARY KEY,
                version INTEGER NOT NULL
            );
            CREATE TABLE sky_nodes (
                path TEXT PRIMARY KEY,
                id TEXT NOT NULL,
                name TEXT NOT NULL,
                library_name TEXT NOT NULL,
                link_count INTEGER NOT NULL DEFAULT 0,
                outgoing_count INTEGER NOT NULL DEFAULT 0,
                stratum TEXT,
                maturity TEXT,
                origin_type TEXT,
                enrichment_dirty INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER,
                updated_at INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE sky_links (
                source_path TEXT NOT NULL,
                target_name TEXT NOT NULL,
                link_type TEXT NOT NULL DEFAULT '',
                weight REAL NOT NULL DEFAULT 0,
                count INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE note_aliases (
                path TEXT NOT NULL,
                alias_lower TEXT NOT NULL,
                added_at INTEGER NOT NULL DEFAULT 0,
                source TEXT NOT NULL DEFAULT 'frontmatter',
                PRIMARY KEY (path, alias_lower)
            );",
        )
        .unwrap();

        let version = if stamp_ready {
            crate::search::SKY_SCHEMA_VERSION as i64
        } else {
            (crate::search::SKY_SCHEMA_VERSION as i64) - 1
        };
        conn.execute(
            "INSERT INTO schema_versions (module, version) VALUES ('sky', ?1)",
            params![version],
        )
        .unwrap();

        for (id, name, p, lib) in nodes {
            conn.execute(
                "INSERT INTO sky_nodes (path, id, name, library_name) VALUES (?1, ?2, ?3, ?4)",
                params![p, id, name, lib],
            )
            .unwrap();
        }
        for (source, target, ltype) in links {
            // PJ-065 — structural (parent/TOC) edges never enter the sky graph
            // (Sky View is a cognitive surface). Active since §5.
            if crate::link_types::is_structural_type(&ltype) {
                continue;
            }
            conn.execute(
                "INSERT INTO sky_links (source_path, target_name, link_type) VALUES (?1, ?2, ?3)",
                params![source, target, ltype],
            )
            .unwrap();
        }
        for (alias, alias_path) in aliases {
            conn.execute(
                "INSERT INTO note_aliases (path, alias_lower, source) VALUES (?1, ?2, 'frontmatter')",
                params![alias_path, alias],
            )
            .unwrap();
        }
    }

    fn attach_as(conn: &Connection, db_path: &std::path::Path, alias: &str) {
        let uri = db_path.to_string_lossy().replace('\\', "/");
        let sql = format!("ATTACH DATABASE 'file:{}?mode=ro' AS {}", uri, alias);
        conn.execute(&sql, []).unwrap();
    }

    // §G.1 — schema="main" returns same as the back-compat shim.
    #[test]
    fn test_sky_nodes_raw_in_schema_main_only() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("search.db");
        make_synthetic_sky_db(
            &db,
            &[("a", "A", "/m/a.md", "Lib"), ("b", "B", "/m/b.md", "Lib")],
            &[],
            &[],
            true,
        );
        let conn = Connection::open(&db).unwrap();
        let result = read_sky_nodes_raw_in_schema(&conn, "main").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|n| n.id == "a"));
        assert!(result.iter().any(|n| n.id == "b"));
    }

    // §G.2 — reads from an ATTACHed cUniverse via cu0.sky_nodes.
    #[test]
    fn test_sky_nodes_raw_in_schema_attached_cu() {
        let main_dir = TempDir::new().unwrap();
        let cu_dir = TempDir::new().unwrap();
        let main_db = main_dir.path().join("search.db");
        let cu_db = cu_dir.path().join("search.db");
        make_synthetic_sky_db(&main_db, &[("m", "M", "/m/m.md", "Main")], &[], &[], true);
        make_synthetic_sky_db(&cu_db, &[("x", "X", "/cu/x.md", "ChildLib")], &[], &[], true);

        let conn = Connection::open(&main_db).unwrap();
        attach_as(&conn, &cu_db, "cu0");

        let cu_nodes = read_sky_nodes_raw_in_schema(&conn, "cu0").unwrap();
        assert_eq!(cu_nodes.len(), 1);
        assert_eq!(cu_nodes[0].name, "X");
        assert_eq!(cu_nodes[0].library_name, "ChildLib");
    }

    // §G.3 — Q4 Option A: single-universe ready returns true.
    #[test]
    fn test_federated_sky_ready_single_universe() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("search.db");
        make_synthetic_sky_db(&db, &[], &[], &[], true);
        let conn = Connection::open(&db).unwrap();
        assert!(is_federated_sky_ready(&conn, &["main".to_string()]));
    }

    // §G.4 — Q4 Option A: any unstamped schema returns false.
    #[test]
    fn test_federated_sky_ready_partial_unstamped() {
        let main_dir = TempDir::new().unwrap();
        let cu_dir = TempDir::new().unwrap();
        let main_db = main_dir.path().join("search.db");
        let cu_db = cu_dir.path().join("search.db");
        make_synthetic_sky_db(&main_db, &[], &[], &[], true);  // main READY
        make_synthetic_sky_db(&cu_db, &[], &[], &[], false);   // cu0 NOT READY

        let conn = Connection::open(&main_db).unwrap();
        attach_as(&conn, &cu_db, "cu0");

        assert!(
            !is_federated_sky_ready(&conn, &["main".to_string(), "cu0".to_string()]),
            "expected NOT ready because cu0 lags on sky_schema_version"
        );
    }

    // §G.5 (revised under §L) — Q3 Option A: links resolve within-universe.
    //
    // Per Boss principle ("data of Universe A shouldn't be merged/integrated
    // with Universe B"), each schema's sky_links resolve ONLY against that
    // same schema's sky_nodes. This test sets up two universes with their
    // own internal A→B links and verifies each resolves correctly + does
    // NOT cross universe boundaries.
    #[test]
    fn test_within_universe_link_resolution() {
        let main_dir = TempDir::new().unwrap();
        let cu_dir = TempDir::new().unwrap();
        let main_db = main_dir.path().join("search.db");
        let cu_db = cu_dir.path().join("search.db");
        // main: A → B (both in main).
        make_synthetic_sky_db(
            &main_db,
            &[
                ("a", "A", "/m/a.md", "Main"),
                ("b", "B", "/m/b.md", "Main"),
            ],
            &[("/m/a.md", "B", "")], // A → B internal link
            &[],
            true,
        );
        // cu0: X → Y (both in cu0).
        make_synthetic_sky_db(
            &cu_db,
            &[
                ("x", "X", "/cu/x.md", "ChildLib"),
                ("y", "Y", "/cu/y.md", "ChildLib"),
            ],
            &[("/cu/x.md", "Y", "")], // X → Y internal link
            &[],
            true,
        );

        let conn = Connection::open(&main_db).unwrap();
        attach_as(&conn, &cu_db, "cu0");

        // Per-schema resolution (matches §E §L logic).
        let mut nodes: Vec<SkyNodeOut> = Vec::new();
        let mut all_links: Vec<SkyLinkOut> = Vec::new();
        for schema in &["main", "cu0"] {
            let schema_start = nodes.len();
            nodes.extend(read_sky_nodes_raw_in_schema(&conn, schema).unwrap());
            let mut path_to_idx: HashMap<String, usize> = HashMap::new();
            let mut name_to_idx: HashMap<String, usize> = HashMap::new();
            for i in schema_start..nodes.len() {
                path_to_idx.insert(nodes[i].path.clone(), i);
                name_to_idx.entry(nodes[i].name.clone()).or_insert(i);
            }
            let alias_to_path: HashMap<String, String> = HashMap::new();
            all_links.extend(
                read_sky_links_raw_in_schema(
                    &conn,
                    schema,
                    &path_to_idx,
                    &name_to_idx,
                    &alias_to_path,
                    &mut nodes,
                )
                .unwrap(),
            );
        }

        // Both links should resolve, each within its own universe.
        assert_eq!(all_links.len(), 2);
        // main's A→B
        assert!(all_links.iter().any(|l| l.source == "a" && l.target == "b"));
        // cu0's X→Y
        assert!(all_links.iter().any(|l| l.source == "x" && l.target == "y"));
        // No cross-universe edges (no A→Y, no X→B).
        assert!(!all_links.iter().any(|l| l.source == "a" && l.target == "y"));
        assert!(!all_links.iter().any(|l| l.source == "x" && l.target == "b"));
    }

    // §G.6 (revised under §L) — Q3 Option A strict isolation:
    // when BOTH universes have a node named "Shared" and BOTH have a
    // link → "Shared", each link resolves to ITS OWN universe's Shared.
    // Per-schema resolution; no cross-universe collision.
    #[test]
    fn test_per_schema_link_isolation() {
        let main_dir = TempDir::new().unwrap();
        let cu_dir = TempDir::new().unwrap();
        let main_db = main_dir.path().join("search.db");
        let cu_db = cu_dir.path().join("search.db");
        // main has Shared + main_src; main has link main_src → Shared.
        make_synthetic_sky_db(
            &main_db,
            &[
                ("shared", "Shared", "/m/shared.md", "Main"),
                ("main_src", "MainSrc", "/m/main_src.md", "Main"),
            ],
            &[("/m/main_src.md", "Shared", "")],
            &[],
            true,
        );
        // cu0 has Shared + cu_src; cu0 has link cu_src → Shared.
        make_synthetic_sky_db(
            &cu_db,
            &[
                ("shared", "Shared", "/cu/shared.md", "ChildLib"),
                ("cu_src", "CuSrc", "/cu/cu_src.md", "ChildLib"),
            ],
            &[("/cu/cu_src.md", "Shared", "")],
            &[],
            true,
        );

        let conn = Connection::open(&main_db).unwrap();
        attach_as(&conn, &cu_db, "cu0");

        let mut nodes: Vec<SkyNodeOut> = Vec::new();
        for schema in &["main", "cu0"] {
            let schema_start = nodes.len();
            nodes.extend(read_sky_nodes_raw_in_schema(&conn, schema).unwrap());
            let mut path_to_idx: HashMap<String, usize> = HashMap::new();
            let mut name_to_idx: HashMap<String, usize> = HashMap::new();
            for i in schema_start..nodes.len() {
                path_to_idx.insert(nodes[i].path.clone(), i);
                name_to_idx.entry(nodes[i].name.clone()).or_insert(i);
            }
            let alias_to_path: HashMap<String, String> = HashMap::new();
            let _ = read_sky_links_raw_in_schema(
                &conn,
                schema,
                &path_to_idx,
                &name_to_idx,
                &alias_to_path,
                &mut nodes,
            )
            .unwrap();
        }

        // Both Shared nodes received their own universe's link.
        // No cross-universe leakage.
        let main_shared_idx = nodes.iter().position(|n| n.path == "/m/shared.md").unwrap();
        let cu_shared_idx = nodes.iter().position(|n| n.path == "/cu/shared.md").unwrap();
        assert_eq!(
            nodes[main_shared_idx].link_count, 1,
            "main's Shared receives exactly 1 incoming link (main_src → Shared)"
        );
        assert_eq!(
            nodes[cu_shared_idx].link_count, 1,
            "cu0's Shared receives exactly 1 incoming link (cu_src → Shared)"
        );
    }

    // §G.7 (revised under §L) — Per-schema aliases.
    // Same alias "foo" can exist in both main and cu0; each resolves
    // within its own universe. No global merge.
    #[test]
    fn test_per_schema_alias_isolation() {
        let main_dir = TempDir::new().unwrap();
        let cu_dir = TempDir::new().unwrap();
        let main_db = main_dir.path().join("search.db");
        let cu_db = cu_dir.path().join("search.db");
        // main: alias "foo" → /m/main.md (which has name "MainNote").
        // main has a link from main_src targeting "foo" (alias).
        make_synthetic_sky_db(
            &main_db,
            &[
                ("note_main", "MainNote", "/m/main.md", "Main"),
                ("main_src", "MainSrc", "/m/main_src.md", "Main"),
            ],
            &[("/m/main_src.md", "foo", "")],
            &[("foo", "/m/main.md")],
            true,
        );
        // cu0: alias "foo" → /cu/cu.md (which has name "CuNote").
        // cu0 has a link from cu_src targeting "foo".
        make_synthetic_sky_db(
            &cu_db,
            &[
                ("note_cu", "CuNote", "/cu/cu.md", "ChildLib"),
                ("cu_src", "CuSrc", "/cu/cu_src.md", "ChildLib"),
            ],
            &[("/cu/cu_src.md", "foo", "")],
            &[("foo", "/cu/cu.md")],
            true,
        );

        let conn = Connection::open(&main_db).unwrap();
        attach_as(&conn, &cu_db, "cu0");

        let mut nodes: Vec<SkyNodeOut> = Vec::new();
        for schema in &["main", "cu0"] {
            let schema_start = nodes.len();
            nodes.extend(read_sky_nodes_raw_in_schema(&conn, schema).unwrap());
            let mut path_to_idx: HashMap<String, usize> = HashMap::new();
            let mut name_to_idx: HashMap<String, usize> = HashMap::new();
            for i in schema_start..nodes.len() {
                path_to_idx.insert(nodes[i].path.clone(), i);
                name_to_idx.entry(nodes[i].name.clone()).or_insert(i);
            }
            let mut alias_to_path: HashMap<String, String> = HashMap::new();
            let alias_sql = format!(
                "SELECT alias_lower, path FROM {}.note_aliases ORDER BY path",
                schema
            );
            let mut stmt = conn.prepare(&alias_sql).unwrap();
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .unwrap();
            for r in rows {
                let (alias, path) = r.unwrap();
                alias_to_path.entry(alias).or_insert(path);
            }
            let _ = read_sky_links_raw_in_schema(
                &conn,
                schema,
                &path_to_idx,
                &name_to_idx,
                &alias_to_path,
                &mut nodes,
            )
            .unwrap();
        }

        // Each universe's MainNote/CuNote received its own link via the
        // alias "foo" resolved within its own scope.
        let main_note_idx = nodes.iter().position(|n| n.path == "/m/main.md").unwrap();
        let cu_note_idx = nodes.iter().position(|n| n.path == "/cu/cu.md").unwrap();
        assert_eq!(
            nodes[main_note_idx].link_count, 1,
            "main's MainNote receives the link via main's alias foo"
        );
        assert_eq!(
            nodes[cu_note_idx].link_count, 1,
            "cu0's CuNote receives the link via cu0's alias foo"
        );
    }

    // §G.6.legacy + §G.7.legacy — REMOVED under §L.
    //
    // The pre-§L tests asserted Option B semantics (first-insert-wins
    // across schemas for both name_to_idx and alias_to_path). Under §L
    // (Q3 Option A: per-schema isolation), there's no global merge —
    // each schema resolves only against its own maps. The legacy tests
    // were testing behavior that no longer exists; deleted rather than
    // adapted. The replacement tests above (test_within_universe_link_
    // resolution, test_per_schema_link_isolation, test_per_schema_alias_
    // isolation) cover Option A correctly.

    // §Q.1 (audit D5 fix) — federated read_links_in_schema concatenates
    // across schemas without merging. Each universe's note_links rows
    // appear independently in the combined result.
    #[test]
    fn test_federated_links_concatenate_across_schemas() {
        let main_dir = TempDir::new().unwrap();
        let cu_dir = TempDir::new().unwrap();
        let main_db = main_dir.path().join("search.db");
        let cu_db = cu_dir.path().join("search.db");

        // Build main DB with a note_links row.
        let conn_m = Connection::open(&main_db).unwrap();
        conn_m.execute_batch("
            CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER NOT NULL);
            CREATE TABLE note_links (
                source_path TEXT NOT NULL, source_name TEXT NOT NULL,
                target_name TEXT NOT NULL, link_type TEXT NOT NULL DEFAULT '',
                library_name TEXT NOT NULL DEFAULT '',
                weight REAL NOT NULL DEFAULT 0.0,
                traversal_count INTEGER NOT NULL DEFAULT 0,
                annotation TEXT NOT NULL DEFAULT '',
                last_traversed TEXT NOT NULL DEFAULT '',
                confidence TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'active'
            );
            INSERT INTO note_links (source_path, source_name, target_name, link_type, library_name)
                VALUES ('/m/a.md', 'A', 'B', 'supports', 'MainLib');
        ").unwrap();
        drop(conn_m);

        // Build cu0 DB with its own note_links row.
        let conn_c = Connection::open(&cu_db).unwrap();
        conn_c.execute_batch("
            CREATE TABLE schema_versions (module TEXT PRIMARY KEY, version INTEGER NOT NULL);
            CREATE TABLE note_links (
                source_path TEXT NOT NULL, source_name TEXT NOT NULL,
                target_name TEXT NOT NULL, link_type TEXT NOT NULL DEFAULT '',
                library_name TEXT NOT NULL DEFAULT '',
                weight REAL NOT NULL DEFAULT 0.0,
                traversal_count INTEGER NOT NULL DEFAULT 0,
                annotation TEXT NOT NULL DEFAULT '',
                last_traversed TEXT NOT NULL DEFAULT '',
                confidence TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'active'
            );
            INSERT INTO note_links (source_path, source_name, target_name, link_type, library_name)
                VALUES ('/cu/x.md', 'X', 'Y', 'derives-from', 'ChildLib');
        ").unwrap();
        drop(conn_c);

        let conn = Connection::open(&main_db).unwrap();
        attach_as(&conn, &cu_db, "cu0");

        let mut links: Vec<NoteLink> = Vec::new();
        links.extend(read_links_in_schema(&conn, "main").unwrap());
        links.extend(read_links_in_schema(&conn, "cu0").unwrap());

        // Both universes' links present, no merging, no resolution.
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|l| l.source_name == "A" && l.target == "B"));
        assert!(links.iter().any(|l| l.source_name == "X" && l.target == "Y"));
    }

    // §Q.2 (audit D5 fix) — federated read_tags_in_schema sums counts
    // across schemas. Tag "shared" appears in both main (2x) and cu0 (3x)
    // → final accumulated count is 5.
    #[test]
    fn test_federated_tags_sum_across_schemas() {
        let main_dir = TempDir::new().unwrap();
        let cu_dir = TempDir::new().unwrap();
        let main_db = main_dir.path().join("search.db");
        let cu_db = cu_dir.path().join("search.db");

        // main has 2 notes with tag "shared", 1 with "main-only".
        let conn_m = Connection::open(&main_db).unwrap();
        conn_m.execute_batch(r#"
            CREATE TABLE note_meta (path TEXT PRIMARY KEY, tags_json TEXT NOT NULL DEFAULT '[]');
            INSERT INTO note_meta VALUES ('/m/a.md', '["shared","main-only"]');
            INSERT INTO note_meta VALUES ('/m/b.md', '["shared"]');
        "#).unwrap();
        drop(conn_m);

        // cu0 has 3 notes with tag "shared", 1 with "cu-only".
        let conn_c = Connection::open(&cu_db).unwrap();
        conn_c.execute_batch(r#"
            CREATE TABLE note_meta (path TEXT PRIMARY KEY, tags_json TEXT NOT NULL DEFAULT '[]');
            INSERT INTO note_meta VALUES ('/cu/x.md', '["shared","cu-only"]');
            INSERT INTO note_meta VALUES ('/cu/y.md', '["shared"]');
            INSERT INTO note_meta VALUES ('/cu/z.md', '["shared"]');
        "#).unwrap();
        drop(conn_c);

        let conn = Connection::open(&main_db).unwrap();
        attach_as(&conn, &cu_db, "cu0");

        let mut tags: HashMap<String, u32> = HashMap::new();
        read_tags_in_schema(&conn, "main", &mut tags).unwrap();
        read_tags_in_schema(&conn, "cu0", &mut tags).unwrap();

        assert_eq!(tags.get("shared"), Some(&5), "shared should accumulate to 5 (2 + 3)");
        assert_eq!(tags.get("main-only"), Some(&1));
        assert_eq!(tags.get("cu-only"), Some(&1));
    }

    // §G.8 — Q2 Option C: id=lower(name) collisions tolerated, path
    // serves as the disambiguator.
    #[test]
    fn test_node_id_uniqueness_lower_name_collisions_tolerated() {
        let main_dir = TempDir::new().unwrap();
        let cu_dir = TempDir::new().unwrap();
        let main_db = main_dir.path().join("search.db");
        let cu_db = cu_dir.path().join("search.db");
        // Both universes have a note id="foobar", name="FooBar".
        make_synthetic_sky_db(
            &main_db,
            &[("foobar", "FooBar", "/m/foobar.md", "Main")],
            &[],
            &[],
            true,
        );
        make_synthetic_sky_db(
            &cu_db,
            &[("foobar", "FooBar", "/cu/foobar.md", "ChildLib")],
            &[],
            &[],
            true,
        );

        let conn = Connection::open(&main_db).unwrap();
        attach_as(&conn, &cu_db, "cu0");

        let mut nodes: Vec<SkyNodeOut> = Vec::new();
        nodes.extend(read_sky_nodes_raw_in_schema(&conn, "main").unwrap());
        nodes.extend(read_sky_nodes_raw_in_schema(&conn, "cu0").unwrap());

        // Both nodes present in the merged result (Q2 Option C: id
        // collisions tolerated, path disambiguates).
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes.iter().filter(|n| n.id == "foobar").count(), 2);
        // Paths are distinct → consumers can disambiguate via path.
        let paths: std::collections::HashSet<_> =
            nodes.iter().map(|n| n.path.as_str()).collect();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains("/m/foobar.md"));
        assert!(paths.contains("/cu/foobar.md"));
    }
}

#[cfg(test)]
mod tests_c2c_per_note_rows {
    //! MIG-079 §C.2c Step-1 proof. The per-note row queries must return the SAME
    //! edge set the frontend getBacklinks/getOutgoingLinks array-filter produces.
    //! The in-memory test pins the match semantics; the ignored rehearsal proves
    //! it on a COPY of the live universe (the §C.1/§C.2a discipline):
    //!   C2C_REHEARSAL_DB="E:\\Backups\\Constellation\\rehearsal\\c2c.db" \
    //!   cargo test --release --lib tests_c2c_per_note_rows::rehearse -- --ignored --nocapture
    use super::*;

    #[test]
    fn backlink_and_outgoing_rows_match_filter_semantics() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note_links (
                source_path TEXT, source_name TEXT, target_name TEXT, link_type TEXT,
                library_name TEXT DEFAULT '', annotation TEXT DEFAULT '', weight REAL DEFAULT 1.0,
                traversal_count INTEGER DEFAULT 0, last_traversed TEXT DEFAULT '',
                confidence TEXT DEFAULT 'hypothesis', status TEXT DEFAULT 'active',
                target_name_lower TEXT GENERATED ALWAYS AS (LOWER(target_name)) VIRTUAL);
             CREATE INDEX idx_nl_tnl ON note_links(target_name_lower, status);
             CREATE INDEX idx_link_source ON note_links(source_path);
             INSERT INTO note_links(source_path,source_name,target_name,link_type,status) VALUES
               ('/S1.md','S1','Alpha','supports','active'),
               ('/S1.md','S1','al','','active'),
               ('/S2.md','S2','alpha','supports','active'),
               ('/S3.md','S3','Beta','causes','active'),
               ('/S4.md','S4','Alpha','supports','archived'),
               ('/A.md','Alpha','Gamma','relates','active'),
               ('/A.md','Alpha','Delta','relates','archived');",
        )
        .unwrap();
        // Backlinks to Alpha (name 'alpha' + alias 'al'): raw matched edges =
        // S1(name) + S1(alias) + S2(case-insensitive); archived S4 excluded.
        let targets = vec!["alpha".to_string(), "al".to_string()];
        let bl = backlink_rows_in_schema(&conn, "main", &targets).unwrap();
        assert_eq!(bl.len(), 3, "S1(name)+S1(alias)+S2; archived excluded");
        let srcs: Vec<&str> = bl.iter().map(|l| l.source_path.as_str()).collect();
        assert!(srcs.contains(&"/S1.md") && srcs.contains(&"/S2.md"));
        assert!(!srcs.contains(&"/S4.md"));
        // link_type empty string maps to None (matches read_links_in_schema).
        assert!(bl.iter().any(|l| l.link_type.as_deref() == Some("supports")));
        assert!(bl.iter().any(|l| l.link_type.is_none()));
        // Outgoing of A: Gamma active; Delta archived excluded.
        let og = outgoing_rows_in_schema(&conn, "main", "/A.md").unwrap();
        assert_eq!(og.len(), 1);
        assert_eq!(og[0].target, "Gamma");
    }

    #[test]
    #[ignore = "rehearsal — needs a live-DB copy via C2C_REHEARSAL_DB"]
    fn rehearse_rows_match_count_and_bruteforce() {
        use std::collections::HashSet;
        let db = std::env::var("C2C_REHEARSAL_DB").expect("set C2C_REHEARSAL_DB");
        let conn = Connection::open(&db).unwrap();

        // Load ALL active edges once for the independent brute-force oracle.
        let all: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare("SELECT source_path, LOWER(target_name) FROM note_links WHERE status != 'archived'")
                .unwrap();
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect()
        };
        eprintln!("[c2c-rehearsal] loaded {} active edges for the oracle", all.len());

        // Sample: the 40 biggest hubs + 40 mid notes (covers hub + leaf + aliased).
        let sample: Vec<(String, String, i64)> = {
            let mut stmt = conn
                .prepare("SELECT path, name, incoming_count FROM note_meta ORDER BY incoming_count DESC LIMIT 40")
                .unwrap();
            let mut v: Vec<(String, String, i64)> = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();
            let mut stmt2 = conn
                .prepare("SELECT path, name, incoming_count FROM note_meta WHERE incoming_count > 0 ORDER BY name LIMIT 40 OFFSET 1000")
                .unwrap();
            v.extend(
                stmt2
                    .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?)))
                    .unwrap()
                    .filter_map(|r| r.ok()),
            );
            v
        };

        let mut checked = 0usize;
        let mut count_mismatch = 0usize;
        let mut set_mismatch = 0usize;
        for (path, name, incoming) in &sample {
            // targets = lower(name) + aliases (note_aliases.alias_lower).
            let mut targets: Vec<String> = vec![name.to_lowercase()];
            {
                let mut astmt = conn.prepare("SELECT alias_lower FROM note_aliases WHERE path = ?").unwrap();
                let al: Vec<String> = astmt
                    .query_map([path], |r| r.get::<_, String>(0))
                    .unwrap()
                    .filter_map(|r| r.ok())
                    .collect();
                for a in al {
                    if !a.is_empty() && !targets.contains(&a) {
                        targets.push(a);
                    }
                }
            }
            let tset: HashSet<&String> = targets.iter().collect();

            // ACTUAL — the §C.2c query, deduped by source (getBacklinks dedupeBySource).
            let actual = backlink_rows_in_schema(&conn, "main", &targets).unwrap();
            let actual_srcs: HashSet<String> = actual.iter().map(|l| l.source_path.clone()).collect();

            // ORACLE — independent brute-force scan of all active edges.
            let oracle_srcs: HashSet<String> = all
                .iter()
                .filter(|(_, t)| tset.contains(t))
                .map(|(s, _)| s.clone())
                .collect();

            checked += 1;
            if actual_srcs != oracle_srcs {
                set_mismatch += 1;
                if set_mismatch <= 8 {
                    eprintln!("   SET MISMATCH {name}: query={} oracle={}", actual_srcs.len(), oracle_srcs.len());
                }
            }
            // Rows-tie-to-count: deduped sources == the §C.2a incoming_count
            // (which the §C.2a rehearsal already proved == getBacklinks).
            if actual_srcs.len() as i64 != *incoming {
                count_mismatch += 1;
                if count_mismatch <= 8 {
                    eprintln!("   COUNT MISMATCH {name}: query={} incoming_count={}", actual_srcs.len(), incoming);
                }
            }
        }
        eprintln!(
            "[c2c-rehearsal] checked {} notes — set_mismatch={} count_mismatch={}",
            checked, set_mismatch, count_mismatch
        );
        assert_eq!(set_mismatch, 0, "per-note query must equal the brute-force filter");
        assert_eq!(count_mismatch, 0, "deduped backlink sources must equal incoming_count");

        // Outgoing — distinct-target count must equal note_meta.outgoing_count
        // for the same sample (MIG-066 write-time aggregate).
        let mut og_mismatch = 0usize;
        for (path, name, _) in &sample {
            let outc: i64 = conn
                .query_row("SELECT outgoing_count FROM note_meta WHERE path = ?", [path], |r| r.get(0))
                .unwrap_or(-1);
            if outc < 0 {
                continue;
            }
            // get_outgoing_rows returns the active outgoing EDGES (status!='archived',
            // == status='active' on this all-active DB). outgoing_count (MIG-066) is
            // COUNT(*) of those edges. So edge-count must equal outgoing_count. (The
            // PANEL later dedupes by raw target — distinct targets ≤ edges — which is a
            // frontend concern, not this row query.) Independent oracle: brute-force
            // active edges from `all` for this source.
            let og = outgoing_rows_in_schema(&conn, "main", path).unwrap();
            let oracle = all.iter().filter(|(s, _)| s == path).count();
            if og.len() != oracle || og.len() as i64 != outc {
                og_mismatch += 1;
                if og_mismatch <= 8 {
                    eprintln!("   OUTGOING MISMATCH {name}: query_edges={} oracle_edges={} outgoing_count={}", og.len(), oracle, outc);
                }
            }
        }
        eprintln!("[c2c-rehearsal] outgoing edge-count mismatch={}", og_mismatch);
        assert_eq!(og_mismatch, 0, "outgoing edge count must equal the brute-force oracle AND outgoing_count");
        eprintln!("[c2c-rehearsal] PASS — per-note rows == filter == write-time counts");
    }
}
